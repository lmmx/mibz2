//! Research only, not the decoder: a block's inverse BWT (its T-vector's build and walk) run
//! by several cores that share its one T-vector (the knee pilot's cooperative inversion,
//! lmmx/mibz2#39, #65). The decoder today inverts a block on one core, so N workers keep N
//! T-vectors live, and on the 10-core workstation throughput falls past the 4 workers whose
//! T-vectors fill ~0.7 of the L3 (docs/journal/2026-10-28-workstation-scaling.md). With k
//! cores a block, N × k cores keep N T-vectors live.
//!
//! ```text
//! coop block FILE.bz2 [PASSES] [K,...]     one block at a time on k cores (default 1,2,3,4)
//! coop mtf FILE.bz2 [ROUNDS]               the MTF stage alone, one core: production's
//!                                          against `mtf_cp` and `mtf_cp2`, which also
//!                                          make the count checkpoints (#78's stage 2 and
//!                                          its spill fix), paired by round
//! coop grid FILE.bz2 [ROUNDS] [ARM,...]    N blocks at once on k cores each: ARM is NxK
//!                                          (cooperative), wNxK (the same walk, the build
//!                                          on one core), pNxK (N production workers; K 1)
//!                                          or sNxK (libsais, with --cfg libsais), rNxK (the
//!                                          cooperative arm, its walk in room passes: #77),
//!                                          qNxK (the r arm, its build given each range's
//!                                          counts instead of counting them: #78's bound),
//!                                          nNxK (the q arm with its checkpoints made by its
//!                                          own MTF, `mtf_cp`: #78's stage 2; with
//!                                          COOP_WITH_MTF=1 only), mNxK and fNxK (the
//!                                          n arm, its MTF `mtf_cp2` or `mtf_cp2p`:
//!                                          #78's spill fix; COOP_WITH_MTF=1 only),
//!                                          xNxK (the q arm, its build in one pass, the
//!                                          scatter taking L's byte from L: #81);
//!                                          @S sets
//!                                          a cooperative arm's segments a block (4x1@1024:
//!                                          one core a block, with 4x2's segments)
//! ```
//!
//! COOP_CPUS (a comma list) pins every thread to its own CPU of it: in `block`, thread j of a
//! pool to the j-th; in `grid`, block worker w's k threads to the w-th k of them (a libsais
//! worker to those k together, which its OpenMP team inherits). Where each thread ran is
//! checked. `block` adds the equal-segment controls (one core walking k threads' segments)
//! and reports each cooperative walk's balance: its busiest thread's share of the bytes, the
//! last thread's start and the spread of the threads' ends.
//!
//! The cooperative inversion, on a pool of k threads kept for the run (the caller one of
//! them; the others spin between jobs, then park):
//! - **build:** the rows cut into k ranges (a multiple of 16 rows each); each thread copies
//!   its range of the last column into the table and counts its bytes; then, from each
//!   range's counts' prefix sums, each links its rows into its own slots of every bucket,
//!   with the production build's 16-row run stores. The slots are disjoint, so are the
//!   writes;
//! - **walk:** 16 segments a lane flagged once (the room walk's xorshift rows, plus the one
//!   the output begins in); each thread's 32 lanes take segments from one shared counter
//!   and write only those, through their own pointers, the table read-only; then the flags
//!   cleared and the segments joined in the output's order.
//!
//! `block` inverts each block of the file in decode order, with the production inversion
//! (mibz2's build and room walk, one core: `prod`) first, then the cooperative one at each
//! k, then (with libsais) libsais's on an OpenMP context of k threads, the arms' order
//! rotated block to block; build and walk timed apart, each summed over a pass, the fastest
//! of the passes kept. Every output is checked against prod's, byte for byte.
//!
//! `grid` measures the T-vector phase as a decoder runs it: every block's last column made
//! first (the scanner and MTF are outside the timing), then for each arm N block workers,
//! each taking the next block from one counter and inverting it on its own k threads, timed from a common start
//! to the last block's end: the arm's throughput in decoded bytes a second. The arms run
//! in an order rotated and reversed by the round. Each block's output is checked by its hash.
//! Each arm's build and walk are timed apart too, as phase throughputs (the bytes over a
//! worker's seconds in the phase), so a slow build and an ineffective walk are told apart;
//! with COOP_JSONL set, every round's figures are appended to that file.
//! Run it under `taskset` with one CPU of each physical core, so that N × k threads are on
//! N × k cores.

#![allow(unexpected_cfgs)] // (`--cfg libsais`, set by bench/coop.sh)
#![allow(clippy::needless_range_loop, clippy::manual_memcpy)] // (a research kernel, indexed as written)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};

const START: u32 = 1 << 31;
const LINK: u32 = !START;
const LANES: usize = 32;
const SEGMENTS_A_THREAD: usize = 512;
/// The stride of a block's count checkpoints (#78): a `q` arm's build ranges start and end
/// on multiples of it, so each range's counts are two checkpoints' difference
const CHECKPOINT: usize = 16 << 10;

#[cfg(libsais)]
#[link(name = "libsais_omp", kind = "static")]
extern "C" {
    fn libsais_unbwt_create_ctx_omp(threads: i32) -> *mut std::ffi::c_void;
    fn libsais_unbwt_cyclic_ctx(
        ctx: *const std::ffi::c_void,
        t: *const u8,
        u: *mut u8,
        a: *mut i32,
        n: i32,
        freq: *const i32,
        i: i32,
    ) -> i32;
}
#[cfg(libsais)]
#[link(name = "gomp")]
extern "C" {}

/// The room walk's starts: `m` xorshift rows seeded by n
fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
    rows.clear();
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    rows.extend((0..m).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as u32
    }));
}

#[inline(always)]
fn prefetch(t: *const u32, row: u32) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a prefetch never faults
    unsafe {
        core::arch::x86_64::_mm_prefetch::<{ core::arch::x86_64::_MM_HINT_T0 }>(
            t.wrapping_add(row as usize) as *const i8,
        )
    }
    #[cfg(not(target_arch = "x86_64"))]
    let _ = (t, row);
}

/// The calling thread pinned to the CPUs `cpus` (sched_setaffinity on itself): false if
/// refused, or not Linux
fn pin(cpus: &[usize]) -> bool {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        let mut mask = [0u64; 16]; // a cpu_set_t: 1024 CPUs
        for &c in cpus {
            mask[c / 64] |= 1 << (c % 64);
        }
        unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) == 0 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpus;
        false
    }
}

/// The CPU the calling thread is running on (-1 where unknown)
fn cpu_now() -> i64 {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn sched_getcpu() -> i32;
        }
        unsafe { sched_getcpu() as i64 }
    }
    #[cfg(not(target_os = "linux"))]
    -1
}

/// A task for a pool: called once on each of its k threads with the thread's index
type Task<'a> = &'a (dyn Fn(usize) + Sync);

/// k - 1 threads kept for the run (the caller is the k-th): a persistent pool, as a decoder
/// would keep, so no block pays a spawn
struct Pool {
    k: usize,
    epoch: Arc<AtomicUsize>,
    done: Arc<AtomicUsize>,
    // the posted task's fat pointer, as two words
    fat: Arc<std::sync::atomic::AtomicU64>,
    fat2: Arc<std::sync::atomic::AtomicU64>,
    hs: Vec<std::thread::JoinHandle<()>>,
    // each helper's CPU, if pinned, and the CPU it was last seen on (after a task)
    cpus: Option<Vec<usize>>,
    seen: Arc<Vec<std::sync::atomic::AtomicI64>>,
}

fn to_words(t: Task) -> (u64, u64) {
    // SAFETY: a fat pointer is two words; it is rebuilt only while the task lives (run waits)
    let w: [u64; 2] = unsafe { std::mem::transmute(t as *const (dyn Fn(usize) + Sync)) };
    (w[0], w[1])
}

impl Pool {
    /// With `cpus`, thread j (the caller is 0) is pinned to `cpus[j]`
    fn new(k: usize, cpus: Option<&[usize]>) -> Pool {
        let seen: Arc<Vec<std::sync::atomic::AtomicI64>> = Arc::new(
            (0..k)
                .map(|_| std::sync::atomic::AtomicI64::new(-1))
                .collect(),
        );
        let cpus = cpus.map(|c| c[..k].to_vec());
        let epoch = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicUsize::new(0));
        let fat = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let fat2 = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let hs = (1..k)
            .map(|idx| {
                let (epoch, done, fat, fat2) =
                    (epoch.clone(), done.clone(), fat.clone(), fat2.clone());
                let (cpu, seen_cpu) = (cpus.as_ref().map(|c| c[idx]), seen.clone());
                std::thread::spawn(move || {
                    if let Some(c) = cpu {
                        assert!(pin(&[c]), "could not pin a helper to CPU {c}");
                    }
                    let mut seen = 0;
                    loop {
                        // spin briefly (within an arm the next job is microseconds away),
                        // then park, so the pools of the arms not running leave their cores
                        // (and any SMT siblings) alone
                        let mut spins = 0u32;
                        let e = loop {
                            let e = epoch.load(Ordering::Acquire);
                            if e != seen {
                                break e;
                            }
                            spins += 1;
                            if spins < 2_000 {
                                std::hint::spin_loop();
                            } else {
                                std::thread::park();
                            }
                        };
                        if e == usize::MAX {
                            return;
                        }
                        seen = e;
                        let w = [fat.load(Ordering::Acquire), fat2.load(Ordering::Acquire)];
                        // SAFETY: the task outlives the epoch it was posted in (run waits)
                        let f: *const (dyn Fn(usize) + Sync) = unsafe { std::mem::transmute(w) };
                        unsafe { (*f)(idx) };
                        seen_cpu[idx].store(cpu_now(), Ordering::Relaxed);
                        done.fetch_add(1, Ordering::AcqRel);
                    }
                })
            })
            .collect();
        Pool {
            k,
            epoch,
            done,
            fat,
            fat2,
            hs,
            cpus,
            seen,
        }
    }

    /// Whether every helper that has run was on the CPU it was pinned to (true unpinned)
    fn placement_ok(&self) -> bool {
        let Some(c) = &self.cpus else { return true };
        (1..self.k).all(|j| {
            let s = self.seen[j].load(Ordering::Relaxed);
            s == -1 || s == c[j] as i64
        })
    }

    /// `f` on every thread of the pool, the caller's index 0; returns when all have ended
    fn run(&self, f: Task) {
        if self.k == 1 {
            return f(0);
        }
        let (a, b) = to_words(f);
        self.done.store(0, Ordering::Relaxed);
        self.fat.store(a, Ordering::Release);
        self.fat2.store(b, Ordering::Release);
        self.epoch.fetch_add(1, Ordering::AcqRel);
        for h in &self.hs {
            h.thread().unpark();
        }
        f(0);
        while self.done.load(Ordering::Acquire) != self.k - 1 {
            std::hint::spin_loop();
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.epoch.store(usize::MAX, Ordering::Release);
        for h in &self.hs {
            h.thread().unpark();
        }
        for h in self.hs.drain(..) {
            h.join().unwrap();
        }
    }
}

/// A raw pointer passed to the pool's threads, each using only the part it owns
struct P<T>(*mut T);
impl<T> Clone for P<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for P<T> {}
unsafe impl<T> Send for P<T> {}
unsafe impl<T> Sync for P<T> {}
impl<T> P<T> {
    // (a method, so that closures capture the whole P, which is Sync, not its field)
    fn get(&self) -> *mut T {
        self.0
    }
}

/// The cooperative inversion's buffers, kept from block to block
#[derive(Default)]
struct Coop {
    tt: Vec<u32>,
    hist: Vec<[u32; 256]>,
    starts: Vec<u32>,
    bufs: Vec<Vec<u8>>,
    next: Vec<usize>,
    /// a block's segments, if not 512 a thread (the equal-segment controls)
    segments: Option<usize>,
    /// the walk in room passes (`coop_walk_room`, the `r` and `q` arms), else `coop_walk`
    room: bool,
    /// the build given each range's counts from the block's checkpoints
    /// (`coop_build_given`, the `q` arms), else `coop_build`, which counts them
    given: bool,
    /// the given build in one pass, the scatter taking L's byte from L (`coop_build_fused`,
    /// the `x` arms: #81)
    fused: bool,
    /// each thread's bytes walked in the last block, and the walk's imbalance summed over
    /// blocks (the busiest thread's bytes over the threads' mean) with its count
    share: Vec<std::sync::atomic::AtomicU64>,
    imbalance: (f64, usize),
    /// each thread's start and end in the last block's walk, in ns from the walk's post;
    /// summed over blocks: the last thread's start (the helpers' wake-up lag) and the spread
    /// of the threads' ends (the tail where some wait), in µs
    times: Vec<(std::sync::atomic::AtomicU64, std::sync::atomic::AtomicU64)>,
    sync_us: (f64, f64),
}

#[inline(always)]
fn read_u64(b: &[u8], i: usize) -> u64 {
    u64::from_ne_bytes(b[i..i + 8].try_into().unwrap())
}

/// The T-vector of `ll` built on the pool's k threads (`build`'s links, exactly)
fn coop_build(ll: &[u8], counts: &[u32; 256], pool: &Pool, c: &mut Coop) {
    let (n, k) = (ll.len(), pool.k);
    // every entry is written in the first task before any is read: the length only
    if c.tt.len() != n {
        c.tt.resize(n, 0);
    }
    c.hist.resize(k, [0; 256]);
    let range = |j: usize| {
        let a = (n * j / k) & !15;
        let b = if j + 1 == k {
            n
        } else {
            (n * (j + 1) / k) & !15
        };
        a..b
    };
    let (tt, hist) = (P(c.tt.as_mut_ptr()), P(c.hist.as_mut_ptr()));
    pool.run(&|j| {
        let r = range(j);
        let h = unsafe { &mut *hist.get().add(j) };
        let dst = unsafe { std::slice::from_raw_parts_mut(tt.get().add(r.start), r.len()) };
        for (d, &c) in dst.iter_mut().zip(&ll[r.clone()]) {
            *d = c as u32;
        }
        if k == 1 {
            *h = *counts;
            return;
        }
        // four counters a byte, so a run of one byte (most of Wikidata's last column) is
        // not one chain of dependent increments
        let mut h4 = [[0u32; 256]; 4];
        let src = &ll[r];
        let mut q = src.chunks_exact(4);
        for c in &mut q {
            h4[0][c[0] as usize] += 1;
            h4[1][c[1] as usize] += 1;
            h4[2][c[2] as usize] += 1;
            h4[3][c[3] as usize] += 1;
        }
        for &c in q.remainder() {
            h4[0][c as usize] += 1;
        }
        for b in 0..256 {
            h[b] = h4[0][b] + h4[1][b] + h4[2][b] + h4[3][b];
        }
    });
    // each range's first slot in each bucket: the bucket's start, then the earlier ranges'
    let mut cum = 0u32;
    let mut starts = [0u32; 256];
    for b in 0..256 {
        starts[b] = cum;
        cum += counts[b];
    }
    for j in 0..k {
        for b in 0..256 {
            let h = c.hist[j][b];
            c.hist[j][b] = starts[b];
            starts[b] += h;
        }
    }
    pool.run(&|j| {
        let r = range(j);
        let ftab = unsafe { &mut *hist.get().add(j) };
        let t = |k: usize| unsafe { &mut *tt.get().add(k) };
        let mut i = r.start;
        while i + 16 <= r.end {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let cb = ll[i] as usize;
                let s = ftab[cb] as usize;
                let base = (i as u32) << 8;
                for jj in 0..16 {
                    let e = t(s + jj);
                    *e = (*e & 0xff) | (base + ((jj as u32) << 8));
                }
                ftab[cb] += 16;
            } else {
                for ii in i..i + 16 {
                    let cb = ll[ii] as usize;
                    let e = t(ftab[cb] as usize);
                    *e = (*e & 0xff) | (ii as u32) << 8;
                    ftab[cb] += 1;
                }
            }
            i += 16;
        }
        for ii in i..r.end {
            let cb = ll[ii] as usize;
            let e = t(ftab[cb] as usize);
            *e = (*e & 0xff) | (ii as u32) << 8;
            ftab[cb] += 1;
        }
    });
}

/// `coop_build` given each range's counts, the one change of #78's first experiment: the
/// ranges start and end on multiples of `CHECKPOINT`, so each range's counts are the
/// difference of two of the block's checkpoints (`checkpoints[i]`, the counts of
/// `ll[..i * CHECKPOINT]`; the block's own counts at its end), and the first pass only
/// copies. The checkpoints are made outside the timing: this bounds what removing the
/// recount buys, it is not what producing them would cost
fn coop_build_given(
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    pool: &Pool,
    c: &mut Coop,
) {
    let (n, k) = (ll.len(), pool.k);
    // every entry is written in the first task before any is read: the length only
    if c.tt.len() != n {
        c.tt.resize(n, 0);
    }
    c.hist.resize(k, [0; 256]);
    // a range's ends: k equal shares, each rounded to the nearest checkpoint
    let edge = |j: usize| {
        if j == k {
            n
        } else {
            ((n * j / k + CHECKPOINT / 2) / CHECKPOINT * CHECKPOINT).min(n)
        }
    };
    let range = |j: usize| edge(j)..edge(j + 1);
    let at = |e: usize| {
        if e == n {
            counts
        } else {
            &checkpoints[e / CHECKPOINT]
        }
    };
    let (tt, hist) = (P(c.tt.as_mut_ptr()), P(c.hist.as_mut_ptr()));
    pool.run(&|j| {
        let r = range(j);
        let h = unsafe { &mut *hist.get().add(j) };
        let dst = unsafe { std::slice::from_raw_parts_mut(tt.get().add(r.start), r.len()) };
        for (d, &c) in dst.iter_mut().zip(&ll[r.clone()]) {
            *d = c as u32;
        }
        let (lo, hi) = (at(r.start), at(r.end));
        for b in 0..256 {
            h[b] = hi[b] - lo[b];
        }
    });
    // each range's first slot in each bucket: the bucket's start, then the earlier ranges'
    let mut cum = 0u32;
    let mut starts = [0u32; 256];
    for b in 0..256 {
        starts[b] = cum;
        cum += counts[b];
    }
    for j in 0..k {
        for b in 0..256 {
            let h = c.hist[j][b];
            c.hist[j][b] = starts[b];
            starts[b] += h;
        }
    }
    pool.run(&|j| {
        let r = range(j);
        let ftab = unsafe { &mut *hist.get().add(j) };
        let t = |k: usize| unsafe { &mut *tt.get().add(k) };
        let mut i = r.start;
        while i + 16 <= r.end {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let cb = ll[i] as usize;
                let s = ftab[cb] as usize;
                let base = (i as u32) << 8;
                for jj in 0..16 {
                    let e = t(s + jj);
                    *e = (*e & 0xff) | (base + ((jj as u32) << 8));
                }
                ftab[cb] += 16;
            } else {
                for ii in i..i + 16 {
                    let cb = ll[ii] as usize;
                    let e = t(ftab[cb] as usize);
                    *e = (*e & 0xff) | (ii as u32) << 8;
                    ftab[cb] += 1;
                }
            }
            i += 16;
        }
        for ii in i..r.end {
            let cb = ll[ii] as usize;
            let e = t(ftab[cb] as usize);
            *e = (*e & 0xff) | (ii as u32) << 8;
            ftab[cb] += 1;
        }
    });
}

/// `coop_build_given` in one pass (#81's cooperative test): each range's first slot in
/// each bucket is the bucket's start plus the counts before the range, which is the
/// block's checkpoint at the range's start, so no pass counts, copies or sums before the
/// scatter, and the threads need no barrier between. The scatter writes each entry whole,
/// `ll[s] | i << 8` at slot s, taking the byte from L (the 256 cursors' own streams), as
/// #93 does on one core; nothing reads the table back. Every slot is written once (each
/// range fills its own slots of every bucket), so the table is only sized, and a kept
/// table's stale words are all overwritten. The table equals `coop_build_given`'s word
/// for word (`fused_check`, before any timing)
fn coop_build_fused(
    ll: &[u8],
    counts: &[u32; 256],
    checkpoints: &[[u32; 256]],
    pool: &Pool,
    c: &mut Coop,
) {
    let (n, k) = (ll.len(), pool.k);
    if c.tt.len() != n {
        c.tt.resize(n, 0);
    }
    let edge = |j: usize| {
        if j == k {
            n
        } else {
            ((n * j / k + CHECKPOINT / 2) / CHECKPOINT * CHECKPOINT).min(n)
        }
    };
    let at = |e: usize| {
        if e == n {
            counts
        } else {
            &checkpoints[e / CHECKPOINT]
        }
    };
    let mut starts = [0u32; 256];
    let mut cum = 0u32;
    for b in 0..256 {
        starts[b] = cum;
        cum += counts[b];
    }
    let tt = P(c.tt.as_mut_ptr());
    pool.run(&|j| {
        let r = edge(j)..edge(j + 1);
        let before = at(r.start);
        let mut ftab = [0u32; 256];
        for b in 0..256 {
            ftab[b] = starts[b] + before[b];
        }
        // SAFETY: slot s < n (the counts are the block's own), and the ranges' slots are
        // disjoint, so each is written by one thread
        let t = |s: usize| unsafe { &mut *tt.get().add(s) };
        let mut i = r.start;
        while i + 16 <= r.end {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let cb = ll[i] as usize;
                let s = ftab[cb] as usize;
                let base = (i as u32) << 8;
                for (jj, &l) in ll[s..s + 16].iter().enumerate() {
                    *t(s + jj) = l as u32 | (base + ((jj as u32) << 8));
                }
                ftab[cb] += 16;
            } else {
                for ii in i..i + 16 {
                    let cb = ll[ii] as usize;
                    let s = ftab[cb] as usize;
                    *t(s) = ll[s] as u32 | (ii as u32) << 8;
                    ftab[cb] += 1;
                }
            }
            i += 16;
        }
        for ii in i..r.end {
            let cb = ll[ii] as usize;
            let s = ftab[cb] as usize;
            *t(s) = ll[s] as u32 | (ii as u32) << 8;
            ftab[cb] += 1;
        }
    });
}

/// `coop_build_fused`'s table against `coop_build_given`'s, word for word, on every block
/// at k = 1, 2, 3 and 4 threads, the fused build each time into a table kept from the
/// block before (so a stale word left unwritten would show), untimed
fn fused_check(bs: &[Block]) {
    let mut words = 0usize;
    for k in 1..=4 {
        let pool = Pool::new(k, None);
        let (mut given, mut fused) = (Coop::default(), Coop::default());
        for (i, b) in bs.iter().enumerate() {
            coop_build_given(&b.ll, &b.counts, &b.checkpoints, &pool, &mut given);
            coop_build_fused(&b.ll, &b.counts, &b.checkpoints, &pool, &mut fused);
            assert!(
                fused.tt == given.tt,
                "block {i}, k {k}: the fused table differs"
            );
            words += b.ll.len();
        }
    }
    eprintln!(
        "fused check: {words} words, every block at k = 1-4: the fused table the given build's"
    );
}

struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

/// One thread's share of the walk: segments taken from `taken` until none are left, each
/// walked to the next start into its own buffer. A lane writes through its own pointer and
/// touches its buffer's header only when it takes, grows or ends the segment, as the room
/// walk does (other threads' segments' headers share its cache lines)
///
/// SAFETY: the pointers are valid for the block; each segment is taken once (fetch_add), so
/// its buffer and successor have one writer; the table is only read
unsafe fn traverse(
    t: *const u32,
    starts: &[u32],
    bufs: P<Vec<u8>>,
    next: P<usize>,
    taken: &AtomicUsize,
) -> u64 {
    let ns = starts.len();
    let mut walked = 0u64;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let take = |seg: usize| -> Lane {
        let b = &mut *bufs.get().add(seg);
        let base = b.as_mut_ptr();
        let q = *t.add(starts[seg] as usize) & LINK;
        *base = q as u8;
        prefetch(t, q >> 8);
        Lane {
            row: q >> 8,
            seg,
            p: base.add(1),
            end: base.add(b.capacity()),
        }
    };
    let grow = |l: &mut Lane| {
        let b = &mut *bufs.get().add(l.seg);
        let len = l.p.offset_from(b.as_ptr()) as usize;
        b.set_len(len);
        b.reserve(b.capacity());
        l.p = b.as_mut_ptr().add(len);
        l.end = b.as_mut_ptr().add(b.capacity());
    };
    let finish = |l: &Lane| {
        let b = &mut *bufs.get().add(l.seg);
        b.set_len(l.p.offset_from(b.as_ptr()) as usize);
        *next.get().add(l.seg) = seg_of(l.row);
    };
    let mut live: Vec<Lane> = Vec::with_capacity(LANES);
    while live.len() < LANES {
        let s = taken.fetch_add(1, Ordering::Relaxed);
        if s >= ns {
            break;
        }
        live.push(take(s));
        walked += 1;
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = *t.add(l.row as usize);
            if q & START != 0 {
                finish(l);
                let s = taken.fetch_add(1, Ordering::Relaxed);
                if s < ns {
                    *l = take(s);
                    walked += 1;
                    i += 1;
                } else {
                    live.swap_remove(i);
                }
                continue;
            }
            l.row = q >> 8;
            prefetch(t, l.row);
            if l.p == l.end {
                grow(l);
            }
            *l.p = q as u8;
            l.p = l.p.add(1);
            walked += 1;
            i += 1;
        }
    }
    walked
}

/// The cooperative walk of `c.tt` on the pool's threads: `walk`'s output into `out`
fn coop_walk(orig_ptr: u32, pool: &Pool, c: &mut Coop, out: &mut Vec<u8>) {
    let n = c.tt.len();
    let r0 = c.tt[orig_ptr as usize] >> 8;
    // 16 segments a lane, as the room walk has (512 for its 32): k threads' lanes need k
    // times the segments, or the walk waits on its tail's few longest chains
    let segments = c.segments.unwrap_or(SEGMENTS_A_THREAD * pool.k);
    random_rows(n, segments - 1, &mut c.starts);
    c.starts.push(r0);
    c.starts.sort_unstable();
    c.starts.dedup();
    for &r in c.starts.iter() {
        c.tt[r as usize] |= START;
    }
    let ns = c.starts.len();
    let cap = 2 * n / ns + 64;
    c.bufs.resize_with(ns.max(c.bufs.len()), Vec::new);
    for b in &mut c.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    c.next.clear();
    c.next.resize(ns, usize::MAX);
    let taken = AtomicUsize::new(0);
    let (t, starts) = (P(c.tt.as_mut_ptr()), &c.starts[..]);
    let (bufs, next) = (P(c.bufs.as_mut_ptr()), P(c.next.as_mut_ptr()));
    let k = pool.k;
    c.share.resize_with(k, Default::default);
    c.times.resize_with(k, Default::default);
    let (share, times) = (&c.share, &c.times);
    let posted = Instant::now();
    pool.run(&|j| {
        times[j]
            .0
            .store(posted.elapsed().as_nanos() as u64, Ordering::Relaxed);
        let w = unsafe { traverse(t.get(), starts, bufs, next, &taken) };
        times[j]
            .1
            .store(posted.elapsed().as_nanos() as u64, Ordering::Relaxed);
        share[j].store(w, Ordering::Relaxed);
    });
    if k > 1 {
        let w: Vec<f64> = c
            .share
            .iter()
            .map(|x| x.load(Ordering::Relaxed) as f64)
            .collect();
        let mean = w.iter().sum::<f64>() / k as f64;
        c.imbalance.0 += w.iter().cloned().fold(0.0, f64::max) / mean;
        c.imbalance.1 += 1;
        let us = |x: &std::sync::atomic::AtomicU64| x.load(Ordering::Relaxed) as f64 / 1e3;
        let last_start = c.times.iter().map(|t| us(&t.0)).fold(0.0, f64::max);
        let ends: Vec<f64> = c.times.iter().map(|t| us(&t.1)).collect();
        let spread = ends.iter().cloned().fold(0.0, f64::max)
            - ends.iter().cloned().fold(f64::INFINITY, f64::min);
        c.sync_us = (c.sync_us.0 + last_start, c.sync_us.1 + spread);
    }
    for &r in c.starts.iter() {
        c.tt[r as usize] &= LINK;
    }
    out.clear();
    out.reserve(n);
    let mut seg = c.starts.binary_search(&r0).unwrap();
    while out.len() < n {
        let b = &c.bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = c.next[seg];
    }
}

/// `traverse` in room passes, as the production room walk runs (`room_segments` in
/// mibz2::bwt), the one change of the #77 experiment: each pass is `room` rounds of every
/// lane, `room` the fewest bytes any lane's buffer has spare (and no more than `cap` - 1, a
/// buffer taken mid-pass's spare after its first byte), so a lane writes its byte at its
/// base plus the pass's offset, with no per-step pointer update or end test. The lanes are
/// an array, not a Vec, and a thread's bytes are counted once a segment, at its end, not
/// once a step. A lane whose segment ends mid-pass takes the next from `taken` and is
/// rebased so that base + offset is its new buffer; when none is left the pass ends there,
/// and the lanes still walking finish in `traverse`'s loop. Exactly `traverse`'s segments.
///
/// SAFETY: as `traverse`; every buffer has at least `cap` bytes of capacity
unsafe fn traverse_room(
    t: *const u32,
    starts: &[u32],
    bufs: P<Vec<u8>>,
    next: P<usize>,
    taken: &AtomicUsize,
    cap: usize,
) -> u64 {
    let ns = starts.len();
    let mut walked = 0u64;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let take = |seg: usize| -> Lane {
        let b = &mut *bufs.get().add(seg);
        let base = b.as_mut_ptr();
        let q = *t.add(starts[seg] as usize) & LINK;
        *base = q as u8;
        prefetch(t, q >> 8);
        Lane {
            row: q >> 8,
            seg,
            p: base.add(1),
            end: base.add(b.capacity()),
        }
    };
    let grow = |l: &mut Lane| {
        let b = &mut *bufs.get().add(l.seg);
        let len = l.p.offset_from(b.as_ptr()) as usize;
        b.set_len(len);
        b.reserve(b.capacity());
        l.p = b.as_mut_ptr().add(len);
        l.end = b.as_mut_ptr().add(b.capacity());
    };
    // the segment's length, its first byte included (`traverse` counts it at the take)
    let finish = |l: &Lane| -> u64 {
        let b = &mut *bufs.get().add(l.seg);
        let len = l.p.offset_from(b.as_ptr()) as usize;
        b.set_len(len);
        *next.get().add(l.seg) = seg_of(l.row);
        len as u64
    };
    let mut lanes: [Lane; LANES] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < LANES {
        let s = taken.fetch_add(1, Ordering::Relaxed);
        if s >= ns {
            break;
        }
        lanes[active] = take(s);
        active += 1;
    }
    if active == LANES {
        'full: loop {
            let mut room = cap - 1;
            for l in lanes.iter_mut() {
                if l.p == l.end {
                    grow(l);
                }
                room = room.min(l.end.offset_from(l.p) as usize);
            }
            let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
            for off in 0..room {
                for k in 0..LANES {
                    let l = &mut lanes[k];
                    let q = *t.add(l.row as usize);
                    if q & START != 0 {
                        l.p = base[k].wrapping_add(off);
                        walked += finish(l);
                        let s = taken.fetch_add(1, Ordering::Relaxed);
                        if s >= ns {
                            // the lanes before this one wrote at `off`, those after it not yet
                            for (j, lj) in lanes.iter_mut().enumerate() {
                                if j < k {
                                    lj.p = base[j].wrapping_add(off + 1);
                                } else if j > k {
                                    lj.p = base[j].wrapping_add(off);
                                }
                            }
                            break 'full;
                        }
                        *l = take(s);
                        // its next byte, at offset off + 1, goes at its p
                        base[k] = l.p.wrapping_sub(off + 1);
                        continue;
                    }
                    l.row = q >> 8;
                    prefetch(t, l.row);
                    *base[k].wrapping_add(off) = q as u8;
                }
            }
            for (l, b) in lanes.iter_mut().zip(base) {
                l.p = b.wrapping_add(room);
            }
        }
    }
    // the lanes still walking (a finished lane's successor is set): `traverse`'s loop, with
    // a byte to spare in each (a pass can end with a buffer exactly full)
    let mut live: Vec<Lane> = Vec::with_capacity(LANES);
    for l in lanes.into_iter().take(active) {
        if *next.get().add(l.seg) == usize::MAX {
            live.push(l);
        }
    }
    for l in live.iter_mut() {
        if l.p == l.end {
            grow(l);
        }
    }
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            let q = *t.add(l.row as usize);
            if q & START != 0 {
                walked += finish(l);
                let s = taken.fetch_add(1, Ordering::Relaxed);
                if s < ns {
                    *l = take(s);
                    i += 1;
                } else {
                    live.swap_remove(i);
                }
                continue;
            }
            l.row = q >> 8;
            prefetch(t, l.row);
            *l.p = q as u8;
            l.p = l.p.add(1);
            if l.p == l.end {
                grow(l);
            }
            i += 1;
        }
    }
    walked
}

/// `coop_walk`, its threads' walks in room passes (`traverse_room`): the `r` arms
fn coop_walk_room(orig_ptr: u32, pool: &Pool, c: &mut Coop, out: &mut Vec<u8>) {
    let n = c.tt.len();
    let r0 = c.tt[orig_ptr as usize] >> 8;
    // 16 segments a lane, as the room walk has (512 for its 32): k threads' lanes need k
    // times the segments, or the walk waits on its tail's few longest chains
    let segments = c.segments.unwrap_or(SEGMENTS_A_THREAD * pool.k);
    random_rows(n, segments - 1, &mut c.starts);
    c.starts.push(r0);
    c.starts.sort_unstable();
    c.starts.dedup();
    for &r in c.starts.iter() {
        c.tt[r as usize] |= START;
    }
    let ns = c.starts.len();
    let cap = 2 * n / ns + 64;
    c.bufs.resize_with(ns.max(c.bufs.len()), Vec::new);
    for b in &mut c.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    c.next.clear();
    c.next.resize(ns, usize::MAX);
    let taken = AtomicUsize::new(0);
    let (t, starts) = (P(c.tt.as_mut_ptr()), &c.starts[..]);
    let (bufs, next) = (P(c.bufs.as_mut_ptr()), P(c.next.as_mut_ptr()));
    let k = pool.k;
    c.share.resize_with(k, Default::default);
    c.times.resize_with(k, Default::default);
    let (share, times) = (&c.share, &c.times);
    let posted = Instant::now();
    pool.run(&|j| {
        times[j]
            .0
            .store(posted.elapsed().as_nanos() as u64, Ordering::Relaxed);
        let w = unsafe { traverse_room(t.get(), starts, bufs, next, &taken, cap) };
        times[j]
            .1
            .store(posted.elapsed().as_nanos() as u64, Ordering::Relaxed);
        share[j].store(w, Ordering::Relaxed);
    });
    if k > 1 {
        let w: Vec<f64> = c
            .share
            .iter()
            .map(|x| x.load(Ordering::Relaxed) as f64)
            .collect();
        let mean = w.iter().sum::<f64>() / k as f64;
        c.imbalance.0 += w.iter().cloned().fold(0.0, f64::max) / mean;
        c.imbalance.1 += 1;
        let us = |x: &std::sync::atomic::AtomicU64| x.load(Ordering::Relaxed) as f64 / 1e3;
        let last_start = c.times.iter().map(|t| us(&t.0)).fold(0.0, f64::max);
        let ends: Vec<f64> = c.times.iter().map(|t| us(&t.1)).collect();
        let spread = ends.iter().cloned().fold(0.0, f64::max)
            - ends.iter().cloned().fold(f64::INFINITY, f64::min);
        c.sync_us = (c.sync_us.0 + last_start, c.sync_us.1 + spread);
    }
    for &r in c.starts.iter() {
        c.tt[r as usize] &= LINK;
    }
    out.clear();
    out.reserve(n);
    let mut seg = c.starts.binary_search(&r0).unwrap();
    while out.len() < n {
        let b = &c.bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = c.next[seg];
    }
}

/// A block's last column, its byte counts and primary index
struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
    /// the counts of `ll[..i * CHECKPOINT]` for each i with i * CHECKPOINT <= its length,
    /// made here, outside every timing (the `q` arms' build reads them: #78's bound)
    checkpoints: Vec<[u32; 256]>,
    /// its MTF/RLE2 symbols, for the arms that time the MTF stage (COOP_WITH_MTF, `mtf`)
    symbols: Symbols,
}

fn blocks(data: &[u8]) -> Vec<Block> {
    let kern = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        let mut checkpoints = vec![[0u32; 256]];
        let mut cum = [0u32; 256];
        for chunk in ll.chunks(CHECKPOINT) {
            for &c in chunk {
                cum[c as usize] += 1;
            }
            if chunk.len() == CHECKPOINT {
                checkpoints.push(cum);
            }
        }
        assert_eq!(cum, counts, "the checkpoints' counts are the block's");
        v.push(Block {
            ll,
            counts,
            orig_ptr: symbols.orig_ptr,
            checkpoints,
            symbols,
        });
    }
    v
}

/// Production's MTF stage (`mibz2::mtf`'s AVX2 kernel, copied unchanged) that also makes the
/// block's count checkpoints, #78's stage 2: `checkpoints[i]` the counts of
/// `ll[..i * CHECKPOINT]`, as `blocks` makes them. The kernel already tests each byte's
/// position against the block's limit, and each run's end against it; here that limit is
/// the next checkpoint or the block's end, whichever is nearer, so the test that finds a
/// checkpoint is the one the kernel makes anyway. At a checkpoint the counts are copied
/// (1 KiB every `CHECKPOINT` bytes), and a run that crosses one is split there.
///
/// SAFETY: as `mibz2::mtf`'s kernel: AVX2 must be present
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_cp(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    checkpoints.clear();
    checkpoints.push([0; 256]);
    // the next checkpoint's position, and the limits the byte and run tests compare with:
    // a byte goes to the slow path at n >= byte_limit, a run when it would reach run_limit
    let mut next = CHECKPOINT;
    let mut byte_limit = max.min(next);
    let mut run_limit = (max + 1).min(next);
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                result = Err(ErrorKind::RunTooLong);
                break;
            }
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            let c = s2u[*base as usize];
            if n + run as usize >= run_limit {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break;
                }
                // the run reaches one or more checkpoints: its part up to each, then the copy
                let mut left = run as usize;
                while n + left >= next {
                    let part = next - n;
                    counts[c as usize] += part as u32;
                    std::ptr::write_bytes(out.add(n), c, part);
                    (n, left) = (next, left - part);
                    checkpoints.push(*counts);
                    next += CHECKPOINT;
                }
                counts[c as usize] += left as u32;
                std::ptr::write_bytes(out.add(n), c, left);
                n += left;
                byte_limit = max.min(next);
                run_limit = (max + 1).min(next);
            } else {
                counts[c as usize] += run;
                std::ptr::write_bytes(out.add(n), c, run as usize);
                n += run as usize;
            }
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= byte_limit {
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            // n == next: the bytes before this one complete a checkpoint
            checkpoints.push(*counts);
            next += CHECKPOINT;
            byte_limit = max.min(next);
            run_limit = (max + 1).min(next);
        }
        let nn = (s - 1) as usize;
        let v = *base.add(nn);
        if nn < 32 {
            let old = _mm256_load_si256(base as *const __m256i);
            let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
            _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
        } else {
            std::ptr::copy(base, base.add(1), nn);
        }
        *base = v;
        let c = s2u[v as usize];
        counts[c as usize] += 1;
        *out.add(n) = c;
        n += 1;
    }
    // a block that ends on a checkpoint completes it
    if result.is_ok() && n == next {
        checkpoints.push(*counts);
    }
    ll.set_len(n);
    result.map(|()| n)
}

/// `mtf_cp`'s checkpoints in the form that keeps production's hot loop (#78's spill fix,
/// the coop journal's §15). `mtf_cp` makes them in the loop itself, and its added live
/// state (the next checkpoint, two limits, the checkpoint vector) pushed the byte count
/// and the end-of-block symbol out of registers (§14.4). Here production's loop runs
/// unchanged but for its limit, the nearer of the block's end and the next checkpoint, a
/// value fixed for the loop as production's `max` is. Where it stops at that limit, the
/// outer loop, entered once every `CHECKPOINT` bytes, tells the block's end (production's
/// error) from a checkpoint, copies the counts, splits a run that crosses one, and takes
/// the symbol it stopped at. The byte step is production's, written once (`byte!`).
///
/// `PIN` (`mtf_cp2::<true>`, `mtf_cp2p`): a compiler fence after the list's new front is
/// stored, which emits no instruction. Without it, LLVM schedules that store after the
/// count's increment, where production and `mtf_cp` have it before. The next symbol's 32-byte
/// list load overlaps that byte store, so it can't be forwarded and waits for its commit,
/// which waits for every older instruction. So that order may put the `s2u` load and the
/// count's read-modify-write on the loop-carried chain (§15.1). The fence keeps production's order.
///
/// SAFETY: as `mibz2::mtf`'s kernel: AVX2 must be present
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_cp2<const PIN: bool>(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    checkpoints.clear();
    checkpoints.push([0; 256]);
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    // production's byte step: symbol $s's byte moved to the front and written at n
    macro_rules! byte {
        ($s:expr) => {{
            let nn = ($s - 1) as usize;
            let v = *base.add(nn);
            if nn < 32 {
                let old = _mm256_load_si256(base as *const __m256i);
                let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
                _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            } else {
                std::ptr::copy(base, base.add(1), nn);
            }
            *base = v;
            if PIN {
                std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            }
            let c = s2u[v as usize];
            counts[c as usize] += 1;
            *out.add(n) = c;
            n += 1;
        }};
    }
    let mut next = CHECKPOINT;
    let mut syms = b.syms.iter();
    'block: loop {
        let limit = max.min(next);
        // production's loop, `max` replaced by `limit`; `at` the symbol it stopped before
        let mut at = None;
        for &s in syms.by_ref() {
            if s <= 1 {
                if weight >= MAX_RUN_WEIGHT {
                    result = Err(ErrorKind::RunTooLong);
                    break 'block;
                }
                run += (s as u32 + 1) * weight;
                weight <<= 1;
                continue;
            }
            if run != 0 {
                if n + run as usize > limit {
                    at = Some(s);
                    break;
                }
                let c = s2u[*base as usize];
                counts[c as usize] += run;
                std::ptr::write_bytes(out.add(n), c, run as usize);
                n += run as usize;
                (run, weight) = (0, 1);
            }
            if s == b.eob {
                break 'block;
            }
            if n >= limit {
                at = Some(s);
                break;
            }
            byte!(s);
        }
        // the symbols ran out with no end-of-block symbol: production's loop ends there too
        let Some(s) = at else { break 'block };
        if run != 0 {
            // a run past the limit: past the block's end, production's error; else it crosses
            // one or more checkpoints, written up to each, then the copy
            if n + run as usize > max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
            let c = s2u[*base as usize];
            let mut left = run as usize;
            while n + left >= next {
                let part = next - n;
                counts[c as usize] += part as u32;
                std::ptr::write_bytes(out.add(n), c, part);
                (n, left) = (next, left - part);
                checkpoints.push(*counts);
                next += CHECKPOINT;
            }
            counts[c as usize] += left as u32;
            std::ptr::write_bytes(out.add(n), c, left);
            n += left;
            (run, weight) = (0, 1);
            // then the symbol after the run, as production's loop takes it (n < next here)
            if s == b.eob {
                break 'block;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
        } else {
            // a byte at the limit: at the block's end, production's error; else n == next,
            // and the bytes before it complete a checkpoint
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
            checkpoints.push(*counts);
            next += CHECKPOINT;
        }
        byte!(s);
    }
    // a block that ends on a checkpoint completes it
    if result.is_ok() && n == next {
        checkpoints.push(*counts);
    }
    ll.set_len(n);
    result.map(|()| n)
}

/// A worker's placeholder for the symbols it never reads (its own block's are in `bs`)
fn empty_symbols() -> Symbols {
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: [0; 256],
        syms: Vec::new(),
        eob: 0,
        max_len: 0,
    }
}

/// `mibz2::mtf`'s limit on a run's weight, as its kernel has it
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

/// The three MTF kernels on one block's symbols with `max_len` set to `max`: each one's
/// result, and its output where it succeeds, must be production's; both checkpoint kernels'
/// checkpoints must be `want`'s (the counts of every CHECKPOINT bytes of production's ll)
fn mtf_agree(sym: &Symbols, max: usize, want: Option<&[[u32; 256]]>, what: &str) -> bool {
    let kern = Kernels::detect();
    let s = Symbols {
        crc: sym.crc,
        orig_ptr: sym.orig_ptr,
        seq_to_unseq: sym.seq_to_unseq,
        syms: sym.syms.clone(),
        eob: sym.eob,
        max_len: max,
    };
    let (mut ll0, mut c0) = (Vec::new(), [0u32; 256]);
    let r0 = mtf::mtf(&s, &mut ll0, &mut c0, kern);
    type Kernel = unsafe fn(
        &Symbols,
        &mut Vec<u8>,
        &mut [u32; 256],
        &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind>;
    for (name, f) in [
        ("mtf_cp", mtf_cp as Kernel),
        ("mtf_cp2", mtf_cp2::<false>),
        ("mtf_cp2p", mtf_cp2::<true>),
    ] {
        let (mut ll, mut c, mut cps) = (Vec::new(), [0u32; 256], Vec::new());
        // SAFETY: AVX2, asserted by the caller
        let r = unsafe { f(&s, &mut ll, &mut c, &mut cps) };
        assert_eq!(
            r, r0,
            "{what}, max {max}: {name}'s result differs from production's"
        );
        if r0.is_ok() {
            assert!(
                ll == ll0 && c == c0,
                "{what}, max {max}: {name}'s ll or counts differ"
            );
            let mut cum = [0u32; 256];
            let mut own = vec![cum];
            for chunk in ll0.chunks(CHECKPOINT) {
                chunk.iter().for_each(|&x| cum[x as usize] += 1);
                if chunk.len() == CHECKPOINT {
                    own.push(cum);
                }
            }
            assert!(cps == own, "{what}, max {max}: {name}'s checkpoints differ");
            if let Some(w) = want {
                assert!(
                    cps == w,
                    "{what}: {name}'s checkpoints differ from the block's"
                );
            }
        }
    }
    r0.is_ok()
}

/// Both checkpoint kernels against production's on `bs` and on libbzip2-compressed edge
/// cases, each block also with its limit cut to its length less one, to its last whole
/// checkpoint (the block's end and a checkpoint the same limit) and raised to its next:
/// every result, error kind, output and checkpoint list the same
fn mtf_check(bs: &[Block]) {
    let mut x = 0x9e37_79b9_7f4a_7c15u64;
    let mut rnd = |m: usize| -> Vec<u8> {
        (0..m)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect()
    };
    let cp = CHECKPOINT;
    let mut zeros_then = vec![0u8; 2 * cp];
    zeros_then.push(7);
    let cases: Vec<(String, Vec<u8>)> = vec![
        ("one byte".into(), vec![b'x']),
        ("two bytes".into(), b"ab".to_vec()),
        ("a run of 1 MB".into(), vec![0u8; 1 << 20]),
        ("2 checkpoints of zeros, then a byte".into(), zeros_then),
        ("period 3".into(), b"abc".repeat(300_000)),
        ("random, 3 checkpoints".into(), rnd(3 * cp)),
        ("random, 3 checkpoints less 1".into(), rnd(3 * cp - 1)),
        ("random, 3 checkpoints and 1".into(), rnd(3 * cp + 1)),
        ("random 2 MB".into(), rnd(2 << 20)),
        (
            "runs of 1-300".into(),
            (0..4000u32)
                .flat_map(|i| {
                    std::iter::repeat((i % 7) as u8 + b'a').take(1 + (i * 37 % 300) as usize)
                })
                .collect(),
        ),
    ];
    let (mut checked, mut refused) = (0usize, 0usize);
    let mut one = |sym: &Symbols, n: usize, want: Option<&[[u32; 256]]>, what: &str| {
        let mut limits = vec![sym.max_len, n, n.div_ceil(cp) * cp];
        if n > 0 {
            limits.push(n - 1);
        }
        if n > cp && n % cp != 0 {
            limits.push(n / cp * cp);
        }
        for m in limits {
            checked += 1;
            refused += !mtf_agree(sym, m, want.filter(|_| m >= n), what) as usize;
        }
    };
    for (name, data) in &cases {
        for level in [1, 9] {
            let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
            std::io::Write::write_all(&mut e, data).unwrap();
            for b in blocks(&e.finish().unwrap()) {
                one(
                    &b.symbols,
                    b.ll.len(),
                    Some(&b.checkpoints),
                    &format!("{name}, level {level}"),
                );
            }
        }
    }
    for (i, b) in bs.iter().enumerate() {
        one(
            &b.symbols,
            b.ll.len(),
            Some(&b.checkpoints),
            &format!("block {i}"),
        );
    }
    println!(
        "mtf check: {checked} runs of each kernel ({refused} refused as production refuses them, with its error), over {} blocks and {} edge cases at levels 1 and 9: every result, ll, counts and checkpoint list production's",
        bs.len(),
        cases.len()
    );
}

/// `mtf`: the MTF stage alone on one core: production's kernel, `mtf_cp`, `mtf_cp2` and
/// `mtf_cp2p` over every block each round, in an order rotated by round, after `mtf_check`
fn mtf_mode(bs: &[Block], rounds: usize, cpus: Option<&[usize]>) {
    assert!(
        std::arch::is_x86_feature_detected!("avx2"),
        "mtf_cp and mtf_cp2 are AVX2"
    );
    if let Some(c) = cpus {
        assert!(pin(&c[..1]), "could not pin");
    }
    mtf_check(bs);
    let kern = Kernels::detect();
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    let (mut ll, mut counts, mut cps) = (Vec::new(), [0u32; 256], Vec::new());
    // the kernels, timed in an order rotated by the round and reversed every other cycle,
    // so each takes each place equally over every 8 rounds
    const NAMES: [&str; 4] = [
        "production (`mibz2::mtf`)",
        "`mtf_cp` (§14)",
        "`mtf_cp2` (§15, the spill fix)",
        "`mtf_cp2p` (§15, the fix, production's store order pinned)",
    ];
    let mut mb: [Vec<f64>; 4] = Default::default();
    for r in 0..rounds {
        let mut order = [0usize, 1, 2, 3];
        order.rotate_left(r % 4);
        if (r / 4) % 2 == 1 {
            order.reverse();
        }
        let mut got = [0f64; 4];
        for k in order {
            let s = Instant::now();
            for b in bs {
                // SAFETY (the checkpoint kernels): AVX2, asserted above
                match k {
                    0 => mtf::mtf(&b.symbols, &mut ll, &mut counts, kern),
                    1 => unsafe { mtf_cp(&b.symbols, &mut ll, &mut counts, &mut cps) },
                    2 => unsafe { mtf_cp2::<false>(&b.symbols, &mut ll, &mut counts, &mut cps) },
                    _ => unsafe { mtf_cp2::<true>(&b.symbols, &mut ll, &mut counts, &mut cps) },
                }
                .unwrap();
            }
            got[k] = bytes as f64 / s.elapsed().as_secs_f64() / 1e6;
        }
        eprintln!(
            "round {r} mtf: production {:.0} MB/s, mtf_cp {:.0}, mtf_cp2 {:.0}, mtf_cp2p {:.0}",
            got[0], got[1], got[2], got[3]
        );
        for k in 0..4 {
            mb[k].push(got[k]);
        }
    }
    let med = |v: &[f64]| {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let cell = |a: usize, base: usize| {
        let ratios: Vec<f64> = mb[a].iter().zip(&mb[base]).map(|(x, y)| x / y).collect();
        let (m, h) = interval(&ratios);
        let (lo, hi) = ratios
            .iter()
            .fold((f64::INFINITY, 0f64), |(l, h), &x| (l.min(x), h.max(x)));
        let won = ratios.iter().filter(|&&x| x > 1.0).count();
        format!("{m:.4} ±{h:.4} ({lo:.4}-{hi:.4}), {won}/{}", ratios.len())
    };
    let ncp: usize = bs.iter().map(|b| b.checkpoints.len()).sum();
    let most = bs.iter().map(|b| b.checkpoints.len()).max().unwrap_or(0);
    let mean_block = bytes as f64 / bs.len() as f64;
    println!(
        "{} blocks, {:.1} MB, {rounds} rounds, one core{}; every block's ll, counts and checkpoints checked",
        bs.len(),
        bytes as f64 / 1e6,
        if cpus.is_some() { ", pinned" } else { "" }
    );
    println!(
        "| kernel | MB/s, median | against production, paired: mean ±95% (min-max), rounds faster | against `mtf_cp`, paired | against `mtf_cp2`, paired |"
    );
    println!("|---|--:|--:|--:|--:|");
    println!("| {} | {:.0} | 1 | | |", NAMES[0], med(&mb[0]));
    println!(
        "| {} | {:.0} | {} | 1 | |",
        NAMES[1],
        med(&mb[1]),
        cell(1, 0)
    );
    println!(
        "| {} | {:.0} | {} | {} | 1 |",
        NAMES[2],
        med(&mb[2]),
        cell(2, 0),
        cell(2, 1)
    );
    println!(
        "| {} | {:.0} | {} | {} | {} |",
        NAMES[3],
        med(&mb[3]),
        cell(3, 0),
        cell(3, 1),
        cell(3, 2)
    );
    println!();
    println!(
        "Checkpoints: {ncp} in all ({:.1} a block, at most {most}), {} KiB a block at most and {:.1} KiB at the mean; the block's own table and last column are {:.0} KiB at the mean ({:.2}% more)",
        ncp as f64 / bs.len() as f64,
        most * 1024 / 1024,
        ncp as f64 / bs.len() as f64,
        mean_block * 5.0 / 1024.0,
        100.0 * (ncp as f64 / bs.len() as f64 * 1024.0) / (mean_block * 5.0)
    );
}

fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| {
        (h ^ x as u64).wrapping_mul(0x100_0000_01b3)
    })
}

/// An inversion method, kept by one block worker
enum Inv {
    Prod(TVector, SegScratch),
    /// the cooperative walk on the first pool; the build on it too, or on the second (one
    /// core: `w` arms, so a slow parallel build can't hide what the shared walk does)
    Coop(Pool, Box<Coop>, Option<Pool>),
    #[cfg(libsais)]
    Sais(*mut std::ffi::c_void, Vec<u8>, Vec<i32>, Vec<i32>),
}
unsafe impl Send for Inv {}

impl Inv {
    /// `kind`'s inversion on k threads (pinned to `cpus`, one each, when given), with `segments`
    /// a block (the cooperative ones; else 512 a thread)
    fn new(kind: char, k: usize, cpus: Option<&[usize]>, segments: Option<usize>) -> Inv {
        let coop = || {
            Box::new(Coop {
                segments,
                room: matches!(kind, 'r' | 'q' | 'n' | 'm' | 'f' | 'x'),
                given: matches!(kind, 'q' | 'n' | 'm' | 'f' | 'x'),
                fused: kind == 'x',
                ..Default::default()
            })
        };
        match kind {
            'p' => Inv::Prod(TVector::new(), SegScratch::default()),
            #[cfg(libsais)]
            's' => Inv::Sais(
                unsafe { libsais_unbwt_create_ctx_omp(k as i32) },
                vec![],
                vec![],
                vec![],
            ),
            'w' => Inv::Coop(Pool::new(k, cpus), coop(), Some(Pool::new(1, None))),
            _ => Inv::Coop(Pool::new(k, cpus), coop(), None),
        }
    }

    /// The cooperative walk's mean imbalance (busiest thread's bytes over the mean), if any
    fn imbalance(&self) -> Option<f64> {
        match self {
            Inv::Coop(_, c, _) if c.imbalance.1 > 0 => Some(c.imbalance.0 / c.imbalance.1 as f64),
            _ => None,
        }
    }

    /// The cooperative walk's mean helper wake-up lag and end spread a block, in µs
    fn sync_us(&self) -> Option<(f64, f64)> {
        match self {
            Inv::Coop(_, c, _) if c.imbalance.1 > 0 => {
                let n = c.imbalance.1 as f64;
                Some((c.sync_us.0 / n, c.sync_us.1 / n))
            }
            _ => None,
        }
    }

    /// Whether the pool's helpers ran where they were pinned
    fn placement_ok(&self) -> bool {
        match self {
            Inv::Coop(p, _, _) => p.placement_ok(),
            _ => true,
        }
    }

    /// The block's inversion into `out`; the build's and the walk's seconds
    fn invert(&mut self, b: &Block, out: &mut Vec<u8>) -> (f64, f64) {
        self.invert_with(b, &b.checkpoints, out)
    }

    /// `invert`, a `q` or `n` arm's build reading `checkpoints` (the block's own, made
    /// outside the timing, or those its worker's `mtf_cp` made in it)
    fn invert_with(
        &mut self,
        b: &Block,
        checkpoints: &[[u32; 256]],
        out: &mut Vec<u8>,
    ) -> (f64, f64) {
        let s = Instant::now();
        match self {
            Inv::Prod(tv, seg) => {
                build(&b.ll, &b.counts, tv);
                let m = s.elapsed().as_secs_f64();
                walk_segments_room(tv, b.orig_ptr, out, seg);
                (m, s.elapsed().as_secs_f64() - m)
            }
            Inv::Coop(pool, c, one) => {
                if c.fused {
                    coop_build_fused(&b.ll, &b.counts, checkpoints, pool, c);
                } else if c.given {
                    coop_build_given(&b.ll, &b.counts, checkpoints, pool, c);
                } else {
                    coop_build(&b.ll, &b.counts, one.as_ref().unwrap_or(pool), c);
                }
                let m = s.elapsed().as_secs_f64();
                if c.room {
                    coop_walk_room(b.orig_ptr, pool, c, out);
                } else {
                    coop_walk(b.orig_ptr, pool, c, out);
                }
                (m, s.elapsed().as_secs_f64() - m)
            }
            #[cfg(libsais)]
            Inv::Sais(ctx, u, a, freq) => {
                let n = b.ll.len();
                freq.clear();
                freq.extend(b.counts.iter().map(|&c| c as i32));
                u.resize(n, 0);
                a.resize(n + 1, 0);
                let r = unsafe {
                    libsais_unbwt_cyclic_ctx(
                        *ctx,
                        b.ll.as_ptr(),
                        u.as_mut_ptr(),
                        a.as_mut_ptr(),
                        n as i32,
                        freq.as_ptr(),
                        b.orig_ptr as i32,
                    )
                };
                assert_eq!(r, 0);
                std::mem::swap(u, out);
                (0.0, s.elapsed().as_secs_f64())
            }
        }
    }
}

/// The 97.5% quantile of Student's t with `df` degrees of freedom (a 95% two-sided interval)
fn t975(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    if df == 0 {
        f64::NAN
    } else if df <= 30 {
        T[df - 1]
    } else {
        1.96 + 2.4 / df as f64
    }
}

/// The mean of `x` and its 95% half-width
fn interval(x: &[f64]) -> (f64, f64) {
    let n = x.len();
    let m = x.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, f64::NAN);
    }
    let v = x.iter().map(|y| (y - m) * (y - m)).sum::<f64>() / (n - 1) as f64;
    (m, t975(n - 1) * (v / n as f64).sqrt())
}

/// `block`: each block inverted in decode order by every arm in turn. With `cpus`, the
/// caller is pinned to the first and each pool's thread j to the j-th (libsais's OpenMP
/// team, which inherits its caller's CPUs, is given the first k for its call)
fn block_mode(bs: &[Block], passes: usize, ks: &[usize], cpus: Option<&[usize]>) {
    if let Some(c) = cpus {
        assert!(
            c.len() >= *ks.iter().max().unwrap(),
            "fewer CPUs than threads"
        );
        assert!(pin(&c[..1]), "could not pin the caller");
    }
    // (name, inversion, its own one-thread arm, its k)
    let mut arms: Vec<(String, Inv, usize, usize)> = vec![(
        "prod (room walk, 1 core)".into(),
        Inv::new('p', 1, None, None),
        0,
        1,
    )];
    let one = arms.len();
    for &k in ks {
        arms.push((format!("coop {k}"), Inv::new('c', k, cpus, None), one, k));
    }
    // the equal-segment controls: one core walking k threads' segments, so that what k cores
    // buy is told from what the finer cut costs or buys
    for &k in ks.iter().filter(|&&k| k > 1) {
        let seg = SEGMENTS_A_THREAD * k;
        arms.push((
            format!("coop 1, {seg} segments (coop {k}'s)"),
            Inv::new('c', 1, cpus, Some(seg)),
            one,
            1,
        ));
    }
    if cfg!(libsais) {
        let one = arms.len();
        for &k in ks {
            arms.push((format!("sais {k}"), Inv::new('s', k, None, None), one, k));
        }
    }
    let na = arms.len();
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    let mut best = vec![(f64::INFINITY, f64::INFINITY); na];
    let (mut want, mut got) = (Vec::new(), Vec::new());
    for pass in 0..passes {
        let mut t = vec![(0.0, 0.0); na];
        for (nb, b) in bs.iter().enumerate() {
            let (m, w) = arms[0].1.invert(b, &mut want);
            t[0].0 += m;
            t[0].1 += w;
            for j in 0..na - 1 {
                let a = 1 + (j + nb) % (na - 1);
                let sais = arms[a].0.starts_with("sais");
                if let (true, Some(c)) = (sais, cpus) {
                    pin(&c[..arms[a].3]);
                }
                let (m, w) = arms[a].1.invert(b, &mut got);
                if let (true, Some(c)) = (sais, cpus) {
                    pin(&c[..1]);
                }
                t[a].0 += m;
                t[a].1 += w;
                assert!(
                    got == want,
                    "{} differs from the room walk on block {nb}",
                    arms[a].0
                );
            }
        }
        for (b, x) in best.iter_mut().zip(&t) {
            *b = (b.0.min(x.0), b.1.min(x.1));
        }
        eprintln!(
            "pass {pass}: prod build {:.3} s, walk {:.3} s",
            t[0].0, t[0].1
        );
    }
    let placed = arms.iter().all(|a| a.1.placement_ok());
    let ns = |s: f64| s * 1e9 / bytes as f64;
    println!(
        "{} blocks, {:.1} MB (BWT bytes), fastest of {passes} passes; {}",
        bs.len(),
        bytes as f64 / 1e6,
        match (cpus, placed) {
            (None, _) => "threads not pinned".to_string(),
            (Some(c), true) => format!(
                "thread j pinned to CPU {:?}[j], every helper seen there",
                &c[..*ks.iter().max().unwrap()]
            ),
            (Some(_), false) => "PINNED, BUT A HELPER WAS SEEN ON ANOTHER CPU".to_string(),
        }
    );
    println!("| arm | build ns/B | walk ns/B | inversion MB/s | vs prod | vs its 1 thread | walk: busiest thread's bytes / mean | walk: last start, end spread (µs a block) |");
    println!("|---|--:|--:|--:|--:|--:|--:|--:|");
    let tot = |i: usize| best[i].0 + best[i].1;
    for i in 0..na {
        println!(
            "| {} | {} | {:.2} | {:.0} | {:.2}x | {:.2}x | {} | {} |",
            arms[i].0,
            if arms[i].0.starts_with("sais") {
                "(in walk)".into()
            } else {
                format!("{:.2}", ns(best[i].0))
            },
            ns(best[i].1),
            bytes as f64 / tot(i) / 1e6,
            tot(0) / tot(i),
            tot(arms[i].2) / tot(i),
            arms[i]
                .1
                .imbalance()
                .map_or("".into(), |x| format!("{x:.3}")),
            arms[i]
                .1
                .sync_us()
                .map_or("".into(), |(l, e)| format!("{l:.0}, {e:.0}"))
        );
    }
}

/// An arm of the grid: its kind, N blocks in flight, k cores a block, and its segments a
/// block if set (`4x1@1024`)
#[derive(Clone, Copy)]
struct Arm {
    kind: char,
    n: usize,
    k: usize,
    segments: Option<usize>,
}

fn parse_arm(s: &str) -> Arm {
    let (kind, rest) = match s.as_bytes()[0] {
        b'p' => ('p', &s[1..]),
        b's' => ('s', &s[1..]),
        b'w' => ('w', &s[1..]),
        b'r' => ('r', &s[1..]),
        b'q' => ('q', &s[1..]),
        b'n' => ('n', &s[1..]),
        b'm' => ('m', &s[1..]),
        b'f' => ('f', &s[1..]),
        b'x' => ('x', &s[1..]),
        _ => ('c', s),
    };
    let (rest, segments) = match rest.split_once('@') {
        Some((r, g)) => (r, Some(g.parse().unwrap())),
        None => (rest, None),
    };
    let (n, k) = rest.split_once('x').unwrap_or((rest, "1"));
    Arm {
        kind,
        n: n.parse().unwrap(),
        k: k.parse().unwrap(),
        segments,
    }
}

/// `grid`: the T-vector phase with N blocks in flight on k cores each. With `cpus`, block
/// worker w's k threads are pinned to cpus[w k ..][..k], one each (a libsais worker to the k
/// together, its OpenMP team inheriting them), and where each was seen is checked
fn grid_mode(bs: &[Block], rounds: usize, specs: &[String], cpus: Option<&[usize]>) {
    // COOP_WITH_MTF=1: each worker makes its block's last column from the symbols in the
    // timing, as a decoder does (#78's stage 2): production's MTF, or an n arm's `mtf_cp`
    let with_mtf = std::env::var("COOP_WITH_MTF").is_ok_and(|v| v == "1");
    let kern = Kernels::detect();
    let mut want_hash = Vec::with_capacity(bs.len());
    let mut inv = Inv::new('p', 1, None, None);
    let mut out = Vec::new();
    for b in bs {
        inv.invert(b, &mut out);
        want_hash.push(fnv(&out));
    }
    drop(inv);
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    let arms: Vec<Arm> = specs.iter().map(|s| parse_arm(s)).collect();
    if arms.iter().any(|a| a.kind == 'x') {
        fused_check(bs);
    }
    assert!(
        cfg!(libsais) || arms.iter().all(|a| a.kind != 's'),
        "s arms need --cfg libsais"
    );
    assert!(
        with_mtf || arms.iter().all(|a| !matches!(a.kind, 'n' | 'm' | 'f')),
        "n, m and f arms need COOP_WITH_MTF=1"
    );
    assert!(
        !with_mtf || arms.iter().all(|a| a.kind != 's'),
        "s arms (libsais) don't take symbols: not with COOP_WITH_MTF=1"
    );
    assert!(
        arms.iter().all(|a| !matches!(a.kind, 'n' | 'm' | 'f'))
            || std::arch::is_x86_feature_detected!("avx2"),
        "n, m and f arms' MTF kernels are AVX2"
    );
    if let Some(c) = cpus {
        for (a, s) in arms.iter().zip(specs) {
            assert!(
                a.n * a.k <= c.len(),
                "{s} needs {} CPUs, {} given",
                a.n * a.k,
                c.len()
            );
        }
    }
    let mut results: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    // each round's build and walk seconds, a block worker's (the workers' sum over N)
    let mut phases: Vec<Vec<(f64, f64)>> = vec![Vec::new(); arms.len()];
    // every timed round's, for their medians (not the last round's: one disturbed round at
    // the end would stand for the arm)
    let mut imbalance: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut misplaced = vec![0usize; arms.len()];
    let mut sync_us: Vec<Vec<(f64, f64)>> = vec![Vec::new(); arms.len()];
    let mut raw = std::env::var("COOP_JSONL").ok().map(|p| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .unwrap()
    });
    // round -1 checks every arm's every block by its hash, untimed; the timed rounds after it
    // run the same code without the hash, which the workers would otherwise share unevenly
    // (an arm with fewer blocks in flight would hash on fewer threads)
    for round in -1..rounds as i64 {
        let check = round < 0;
        let r = round.max(0) as usize;
        let mut order: Vec<usize> = (0..arms.len()).collect();
        order.rotate_left(r % arms.len());
        if (r / arms.len()) % 2 == 1 {
            order.reverse();
        }
        for &a in &order {
            let Arm {
                kind,
                n,
                k,
                segments,
            } = arms[a];
            let group = |w: usize| cpus.map(|c| &c[w * k..(w + 1) * k]);
            let mut workers: Vec<Inv> = (0..n)
                .map(|w| Inv::new(kind, k, group(w), segments))
                .collect();
            let go = std::sync::Barrier::new(n + 1);
            // the blocks taken in order from one counter, as the decoder's workers take jobs
            // from one queue, so a slow worker doesn't keep a fixed share
            let next_block = AtomicUsize::new(0);
            let (wall, busy, placed) = std::thread::scope(|sc| {
                let hs: Vec<_> = workers
                    .iter_mut()
                    .enumerate()
                    .map(|(w, inv)| {
                        let (go, want_hash, g, next_block) =
                            (&go, &want_hash, group(w), &next_block);
                        sc.spawn(move || {
                            if let Some(g) = g {
                                assert!(
                                    pin(if kind == 's' { g } else { &g[..1] }),
                                    "could not pin a worker"
                                );
                            }
                            let mut out = Vec::new();
                            let (mut tb, mut tw, mut tm) = (0.0, 0.0, 0.0);
                            // with the MTF stage timed: this worker's own last column and
                            // counts (and an n arm's checkpoints), made from the symbols
                            let mut mine = Block {
                                ll: Vec::new(),
                                counts: [0; 256],
                                orig_ptr: 0,
                                checkpoints: Vec::new(),
                                symbols: empty_symbols(),
                            };
                            go.wait();
                            loop {
                                let i = next_block.fetch_add(1, Ordering::Relaxed);
                                if i >= bs.len() {
                                    break;
                                }
                                let (b, w) = if with_mtf {
                                    let s = Instant::now();
                                    let sym = &bs[i].symbols;
                                    if kind == 'n' {
                                        // SAFETY: mtf_cp needs AVX2, asserted at the start
                                        unsafe {
                                            mtf_cp(
                                                sym,
                                                &mut mine.ll,
                                                &mut mine.counts,
                                                &mut mine.checkpoints,
                                            )
                                        }
                                        .unwrap();
                                    } else if matches!(kind, 'm' | 'f') {
                                        // SAFETY: mtf_cp2 needs AVX2, asserted at the start
                                        let (ll, c, cps) =
                                            (&mut mine.ll, &mut mine.counts, &mut mine.checkpoints);
                                        unsafe {
                                            if kind == 'm' {
                                                mtf_cp2::<false>(sym, ll, c, cps)
                                            } else {
                                                mtf_cp2::<true>(sym, ll, c, cps)
                                            }
                                        }
                                        .unwrap();
                                    } else {
                                        mtf::mtf(sym, &mut mine.ll, &mut mine.counts, kern)
                                            .unwrap();
                                    }
                                    mine.orig_ptr = sym.orig_ptr;
                                    tm += s.elapsed().as_secs_f64();
                                    if check {
                                        assert!(
                                            !matches!(kind, 'n' | 'm' | 'f')
                                                || mine.checkpoints == bs[i].checkpoints,
                                            "block {i}: {kind}'s MTF checkpoints differ"
                                        );
                                    }
                                    // a q arm's checkpoints are still the free ones (its bound
                                    // with the MTF stage in the timing); an n, m or f arm's are its own
                                    let cps = if matches!(kind, 'n' | 'm' | 'f') {
                                        &mine.checkpoints
                                    } else {
                                        &bs[i].checkpoints
                                    };
                                    inv.invert_with(&mine, cps, &mut out)
                                } else {
                                    inv.invert(&bs[i], &mut out)
                                };
                                (tb, tw) = (tb + b, tw + w);
                                if check {
                                    assert_eq!(fnv(&out), want_hash[i], "block {i} differs");
                                }
                            }
                            let here = cpu_now();
                            let placed = g.map_or(true, |g| g.iter().any(|&c| c as i64 == here));
                            (tb, tw, tm, placed)
                        })
                    })
                    .collect();
                go.wait();
                let s = Instant::now();
                let ends: Vec<(f64, f64, f64, bool)> =
                    hs.into_iter().map(|h| h.join().unwrap()).collect();
                (
                    s.elapsed().as_secs_f64(),
                    ends.clone(),
                    ends.iter().all(|e| e.3),
                )
            });
            let placed = placed && workers.iter().all(|w| w.placement_ok());
            if check {
                assert!(
                    cpus.is_none() || placed,
                    "{}: a thread was seen off its CPU",
                    specs[a]
                );
                eprintln!("checked {}: every block's output the room walk's", specs[a]);
                continue;
            }
            misplaced[a] += !placed as usize;
            let tb = busy.iter().map(|b| b.0).sum::<f64>() / n as f64;
            let tw = busy.iter().map(|b| b.1).sum::<f64>() / n as f64;
            let tm = busy.iter().map(|b| b.2).sum::<f64>() / n as f64;
            // the busiest and idlest block worker's build + walk seconds: wall less the
            // busiest is what no worker's phases account for (startup, the last block's tail)
            let totals = busy.iter().map(|b| b.0 + b.1 + b.2);
            let bmax = totals.clone().fold(0.0, f64::max);
            let bmin = totals.fold(f64::INFINITY, f64::min);
            results[a].push(bytes as f64 / wall / 1e6);
            phases[a].push((tb, tw));
            let imb = workers
                .iter()
                .filter_map(|w| w.imbalance())
                .reduce(f64::max);
            imbalance[a].extend(imb);
            let sync = workers
                .iter()
                .filter_map(|w| w.sync_us())
                .reduce(|x, y| (x.0.max(y.0), x.1.max(y.1)));
            sync_us[a].extend(sync);
            if let Some(f) = raw.as_mut() {
                use std::io::Write;
                writeln!(
                    f,
                    r#"{{"round": {round}, "arm": "{}", "kind": "{kind}", "blocks_live": {n}, "cores_a_block": {k}, "segments": {}, "pinned": {}, "placement_ok": {placed}, "wall_s": {wall:.6}, "mb_s": {:.3}, "build_s_a_worker": {tb:.6}, "walk_s_a_worker": {tw:.6}, "mtf_s_a_worker": {tm:.6}, "busy_max_s": {bmax:.6}, "busy_min_s": {bmin:.6}, "unaccounted_s": {:.6}, "walk_imbalance": {}, "walk_last_start_us": {}, "walk_end_spread_us": {}}}"#,
                    specs[a],
                    segments.map_or("null".into(), |g| g.to_string()),
                    cpus.is_some(),
                    bytes as f64 / wall / 1e6,
                    wall - bmax,
                    imb.map_or("null".into(), |x| format!("{x:.4}")),
                    sync.map_or("null".into(), |s| format!("{:.1}", s.0)),
                    sync.map_or("null".into(), |s| format!("{:.1}", s.1))
                )
                .unwrap();
            }
            eprintln!(
                "round {round} {}: {:.0} MB/s",
                specs[a],
                bytes as f64 / wall / 1e6
            );
        }
    }
    let med = |v: &[f64]| {
        let mut v = v.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    // paired against the first arm, round by round
    let base = &results[0];
    println!(
        "{} blocks, {:.1} MB, {rounds} timed rounds; {}, not the decoder; {}",
        bs.len(),
        bytes as f64 / 1e6,
        if with_mtf {
            "the MTF stage and the T-vector phase (MTF + build + walk; MTF in no column below)"
        } else {
            "the T-vector phase only (build + walk)"
        },
        if cpus.is_some() {
            "each thread pinned to its own CPU, checked"
        } else {
            "threads not pinned"
        }
    );
    println!(
        "| arm | blocks live | cores | MB/s, median | vs {0}, paired: mean ±95% (min-max) | build MB/s | walk MB/s | walk vs {0}, paired | walk: busiest thread's bytes / mean, median | walk: last start, end spread (µs a block), medians | rounds misplaced |",
        specs[0]
    );
    println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    // a phase's throughput: the bytes over a worker's seconds in it (each of the N workers
    // inverts 1/N of the bytes, all at once), so arms of different N compare
    for (a, r) in results.iter().enumerate() {
        let Arm { kind, n, k, .. } = arms[a];
        let ratios: Vec<f64> = r.iter().zip(base).map(|(x, y)| x / y).collect();
        let (mean, h) = interval(&ratios);
        let (lo, hi) = ratios
            .iter()
            .fold((f64::INFINITY, 0f64), |(l, h), &x| (l.min(x), h.max(x)));
        let mbs = |t: f64| bytes as f64 / t / 1e6;
        let build: Vec<f64> = phases[a].iter().map(|p| mbs(p.0)).collect();
        let walk: Vec<f64> = phases[a].iter().map(|p| mbs(p.1)).collect();
        let wr: Vec<f64> = phases[a]
            .iter()
            .zip(&phases[0])
            .map(|(p, q)| q.1 / p.1)
            .collect();
        println!(
            "| {} | {n} | {} | {:.0} | {mean:.3} ±{h:.3} ({lo:.3}-{hi:.3}) | {} | {:.0} | {:.3} | {} | {} | {} |",
            specs[a],
            n * k,
            med(r),
            if kind == 's' { "(in walk)".to_string() } else { format!("{:.0}", med(&build)) },
            med(&walk),
            interval(&wr).0,
            if imbalance[a].is_empty() {
                String::new()
            } else {
                format!("{:.3}", med(&imbalance[a]))
            },
            if sync_us[a].is_empty() {
                String::new()
            } else {
                let l: Vec<f64> = sync_us[a].iter().map(|s| s.0).collect();
                let e: Vec<f64> = sync_us[a].iter().map(|s| s.1).collect();
                format!("{:.0}, {:.0}", med(&l), med(&e))
            },
            misplaced[a]
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    let data =
        std::fs::read(args.get(2).expect("coop block|grid FILE.bz2 ...")).expect("a .bz2 file");
    // COOP_CPUS: the CPUs to pin to, one a thread (bench/coop.sh: one of each physical core)
    let cpus: Option<Vec<usize>> = std::env::var("COOP_CPUS")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').map(|c| c.parse().unwrap()).collect());
    let bs = blocks(&data);
    match mode {
        "block" => {
            let passes = args.get(3).map_or(3, |s| s.parse().unwrap());
            let ks: Vec<usize> = args
                .get(4)
                .map_or("1,2,3,4", |s| s)
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect();
            block_mode(&bs, passes, &ks, cpus.as_deref());
        }
        "mtf" => {
            let rounds = args.get(3).map_or(20, |s| s.parse().unwrap());
            mtf_mode(&bs, rounds, cpus.as_deref());
        }
        "grid" => {
            let rounds = args.get(3).map_or(4, |s| s.parse().unwrap());
            let specs: Vec<String> = args
                .get(4)
                .map_or("p4,p2,4x1,2x2,1x4", |s| s)
                .split(',')
                .map(String::from)
                .collect();
            grid_mode(&bs, rounds, &specs, cpus.as_deref());
        }
        _ => panic!("coop block|mtf|grid FILE.bz2 ..."),
    }
}
