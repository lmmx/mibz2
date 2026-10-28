//! Research only, not the decoder: a block's inverse BWT (its T-vector's build and walk) run
//! by several cores that share its one T-vector (the knee pilot's cooperative inversion,
//! lmmx/mibz2#39, #65). The decoder today inverts a block on one core, so N workers keep N
//! T-vectors live, and on the 10-core workstation throughput falls past the 4 workers whose
//! T-vectors fill ~0.7 of the L3 (docs/journal/2026-10-28-workstation-scaling.md). With k
//! cores a block, N × k cores keep N T-vectors live.
//!
//! ```text
//! coop block FILE.bz2 [PASSES] [K,...]     one block at a time on k cores (default 1,2,3,4)
//! coop grid FILE.bz2 [ROUNDS] [ARM,...]    N blocks at once on k cores each: ARM is NxK
//!                                          (cooperative), wNxK (the same walk, the build
//!                                          on one core), pNxK (N production workers; K 1)
//!                                          or sNxK (libsais, with --cfg libsais)
//! ```
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
//! each inverting every Nth block in turn on its own k threads, timed from a common start
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

use mibz2::bwt::{build, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

const START: u32 = 1 << 31;
const LINK: u32 = !START;
const LANES: usize = 32;
const SEGMENTS_A_THREAD: usize = 512;

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
}

fn to_words(t: Task) -> (u64, u64) {
    // SAFETY: a fat pointer is two words; it is rebuilt only while the task lives (run waits)
    let w: [u64; 2] = unsafe { std::mem::transmute(t as *const (dyn Fn(usize) + Sync)) };
    (w[0], w[1])
}

impl Pool {
    fn new(k: usize) -> Pool {
        let epoch = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicUsize::new(0));
        let fat = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let fat2 = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let hs = (1..k)
            .map(|idx| {
                let (epoch, done, fat, fat2) =
                    (epoch.clone(), done.clone(), fat.clone(), fat2.clone());
                std::thread::spawn(move || {
                    let mut seen = 0;
                    loop {
                        // spin a while (the next job is microseconds away), then park, so
                        // the pools of the arms not running leave their cores alone
                        let mut spins = 0u32;
                        let e = loop {
                            let e = epoch.load(Ordering::Acquire);
                            if e != seen {
                                break e;
                            }
                            spins += 1;
                            if spins < 50_000 {
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
        }
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
) {
    let ns = starts.len();
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
            i += 1;
        }
    }
}

/// The cooperative walk of `c.tt` on the pool's threads: `walk`'s output into `out`
fn coop_walk(orig_ptr: u32, pool: &Pool, c: &mut Coop, out: &mut Vec<u8>) {
    let n = c.tt.len();
    let r0 = c.tt[orig_ptr as usize] >> 8;
    // 16 segments a lane, as the room walk has (512 for its 32): k threads' lanes need k
    // times the segments, or the walk waits on its tail's few longest chains
    random_rows(n, SEGMENTS_A_THREAD * pool.k - 1, &mut c.starts);
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
    pool.run(&|_| unsafe { traverse(t.get(), starts, bufs, next, &taken) });
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
}

fn blocks(data: &[u8]) -> Vec<Block> {
    let kern = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut v = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&symbols, &mut ll, &mut counts, kern).unwrap();
        v.push(Block {
            ll,
            counts,
            orig_ptr: symbols.orig_ptr,
        });
    }
    v
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
    fn new(kind: char, k: usize) -> Inv {
        match kind {
            'p' => Inv::Prod(TVector::new(), SegScratch::default()),
            #[cfg(libsais)]
            's' => Inv::Sais(
                unsafe { libsais_unbwt_create_ctx_omp(k as i32) },
                vec![],
                vec![],
                vec![],
            ),
            'w' => Inv::Coop(Pool::new(k), Box::default(), Some(Pool::new(1))),
            _ => Inv::Coop(Pool::new(k), Box::default(), None),
        }
    }

    /// The block's inversion into `out`; the build's and the walk's seconds
    fn invert(&mut self, b: &Block, out: &mut Vec<u8>) -> (f64, f64) {
        let s = Instant::now();
        match self {
            Inv::Prod(tv, seg) => {
                build(&b.ll, &b.counts, tv);
                let m = s.elapsed().as_secs_f64();
                walk_segments_room(tv, b.orig_ptr, out, seg);
                (m, s.elapsed().as_secs_f64() - m)
            }
            Inv::Coop(pool, c, one) => {
                coop_build(&b.ll, &b.counts, one.as_ref().unwrap_or(pool), c);
                let m = s.elapsed().as_secs_f64();
                coop_walk(b.orig_ptr, pool, c, out);
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

/// `block`: each block inverted in decode order by every arm in turn
fn block_mode(bs: &[Block], passes: usize, ks: &[usize]) {
    let sais = cfg!(libsais);
    let mut arms: Vec<(String, Inv)> = vec![("prod (room walk, 1 core)".into(), Inv::new('p', 1))];
    arms.extend(ks.iter().map(|&k| (format!("coop {k}"), Inv::new('c', k))));
    if sais {
        arms.extend(ks.iter().map(|&k| (format!("sais {k}"), Inv::new('s', k))));
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
                let (m, w) = arms[a].1.invert(b, &mut got);
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
    let ns = |s: f64| s * 1e9 / bytes as f64;
    println!(
        "{} blocks, {:.1} MB (BWT bytes), fastest of {passes} passes",
        bs.len(),
        bytes as f64 / 1e6
    );
    println!("| arm | build ns/B | walk ns/B | inversion MB/s | vs prod | vs its 1 thread |");
    println!("|---|--:|--:|--:|--:|--:|");
    let tot = |i: usize| best[i].0 + best[i].1;
    for i in 0..na {
        let one = if i == 0 {
            0
        } else if i <= ks.len() {
            1
        } else {
            1 + ks.len()
        };
        println!(
            "| {} | {} | {:.2} | {:.0} | {:.2}x | {:.2}x |",
            arms[i].0,
            if arms[i].0.starts_with("sais") {
                "(in walk)".into()
            } else {
                format!("{:.2}", ns(best[i].0))
            },
            ns(best[i].1),
            bytes as f64 / tot(i) / 1e6,
            tot(0) / tot(i),
            tot(one) / tot(i)
        );
    }
}

/// `grid`: the T-vector phase with N blocks in flight on k cores each
fn grid_mode(bs: &[Block], rounds: usize, specs: &[String]) {
    let mut want_hash = Vec::with_capacity(bs.len());
    let mut inv = Inv::new('p', 1);
    let mut out = Vec::new();
    for b in bs {
        inv.invert(b, &mut out);
        want_hash.push(fnv(&out));
    }
    drop(inv);
    let bytes: usize = bs.iter().map(|b| b.ll.len()).sum();
    let parse = |s: &str| -> (char, usize, usize) {
        let (kind, rest) = match s.as_bytes()[0] {
            b'p' => ('p', &s[1..]),
            b's' => ('s', &s[1..]),
            b'w' => ('w', &s[1..]),
            _ => ('c', s),
        };
        let (n, k) = rest.split_once('x').unwrap_or((rest, "1"));
        (kind, n.parse().unwrap(), k.parse().unwrap())
    };
    let arms: Vec<(char, usize, usize)> = specs.iter().map(|s| parse(s)).collect();
    assert!(
        cfg!(libsais) || arms.iter().all(|a| a.0 != 's'),
        "s arms need --cfg libsais"
    );
    let mut results: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    // each round's build and walk seconds, a block worker's (the workers' sum over N)
    let mut phases: Vec<Vec<(f64, f64)>> = vec![Vec::new(); arms.len()];
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
            let (kind, n, k) = arms[a];
            let mut workers: Vec<Inv> = (0..n).map(|_| Inv::new(kind, k)).collect();
            let go = std::sync::Barrier::new(n + 1);
            let wall = std::thread::scope(|sc| {
                let hs: Vec<_> = workers
                    .iter_mut()
                    .enumerate()
                    .map(|(w, inv)| {
                        let (go, want_hash) = (&go, &want_hash);
                        sc.spawn(move || {
                            let mut out = Vec::new();
                            let (mut tb, mut tw) = (0.0, 0.0);
                            go.wait();
                            let mut i = w;
                            while i < bs.len() {
                                let (b, w) = inv.invert(&bs[i], &mut out);
                                (tb, tw) = (tb + b, tw + w);
                                if check {
                                    assert_eq!(fnv(&out), want_hash[i], "block {i} differs");
                                }
                                i += n;
                            }
                            (tb, tw)
                        })
                    })
                    .collect();
                go.wait();
                let s = Instant::now();
                let (mut tb, mut tw) = (0.0, 0.0);
                for h in hs {
                    let (b, w) = h.join().unwrap();
                    (tb, tw) = (tb + b, tw + w);
                }
                (s.elapsed().as_secs_f64(), tb / n as f64, tw / n as f64)
            });
            let (wall, tb, tw) = wall;
            if check {
                eprintln!("checked {}: every block's output the room walk's", specs[a]);
                continue;
            }
            results[a].push(bytes as f64 / wall / 1e6);
            phases[a].push((tb, tw));
            if let Some(f) = raw.as_mut() {
                use std::io::Write;
                writeln!(
                    f,
                    r#"{{"round": {round}, "arm": "{}", "kind": "{kind}", "blocks_live": {n}, "cores_a_block": {k}, "wall_s": {wall:.6}, "mb_s": {:.3}, "build_s_a_worker": {tb:.6}, "walk_s_a_worker": {tw:.6}}}"#,
                    specs[a],
                    bytes as f64 / wall / 1e6
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
        "{} blocks, {:.1} MB, {rounds} rounds; the T-vector phase only (build + walk)",
        bs.len(),
        bytes as f64 / 1e6
    );
    println!(
        "| arm | blocks live | cores | MB/s, median | vs {0}, paired mean (min-max) | build MB/s | walk MB/s | walk vs {0}, paired mean |",
        specs[0]
    );
    println!("|---|--:|--:|--:|--:|--:|--:|--:|");
    // a phase's throughput: the bytes over a worker's seconds in it (each of the N workers
    // inverts 1/N of the bytes, all at once), so arms of different N compare
    for (a, r) in results.iter().enumerate() {
        let (_, n, k) = arms[a];
        let ratios: Vec<f64> = r.iter().zip(base).map(|(x, y)| x / y).collect();
        let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
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
            "| {} | {n} | {} | {:.0} | {mean:.3} ({lo:.3}-{hi:.3}) | {} | {:.0} | {:.3} |",
            specs[a],
            n * k,
            med(r),
            if arms[a].0 == 's' {
                "(in walk)".to_string()
            } else {
                format!("{:.0}", med(&build))
            },
            med(&walk),
            wr.iter().sum::<f64>() / wr.len() as f64
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    let data =
        std::fs::read(args.get(2).expect("coop block|grid FILE.bz2 ...")).expect("a .bz2 file");
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
            block_mode(&bs, passes, &ks);
        }
        "grid" => {
            let rounds = args.get(3).map_or(4, |s| s.parse().unwrap());
            let specs: Vec<String> = args
                .get(4)
                .map_or("p4,p2,4x1,2x2,1x4", |s| s)
                .split(',')
                .map(String::from)
                .collect();
            grid_mode(&bs, rounds, &specs);
        }
        _ => panic!("coop block|grid FILE.bz2 ..."),
    }
}
