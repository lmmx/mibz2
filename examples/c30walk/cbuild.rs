//! The construction-cost collector (docs/journal/2026-11-03-compact30-construction-protocol.md,
//! draft 3): compact30's direct constructor against production's build, both arms in this
//! one process.
//!
//!     c30walk cbuild OUT.jsonl CORPUS.bz2 --rounds R --cpus A,B,C,D [--dry-run]
//!         [--peak-rss BYTES] [--inject KIND]
//!     c30walk cbuild-rss OUT.json CORPUS.bz2 --arm P|C --cpus A,B,C,D
//!
//! **The arms** (§2), each timed around one call:
//! - **P:** `p_build`, which calls production's public `mibz2::bwt::build`;
//! - **C:** `c_build`, which calls `Ctor::build::<false>` (`ctor.rs`, unchanged), natural
//!   configuration.
//!
//! Both wrappers are `#[inline(never)]`, so each timed function is one symbol whose listing
//! G3 checks. Each window holds exactly the call: the builder's clears, validation,
//! allocation and growth, scan, writes, promotion, ordering, padding and `set_len`.
//!
//! **The window,** in both regimes: the monotonic clock, the thread CPU clock, the thread's
//! rusage (faults), the call, then rusage, CPU, monotonic. The calibration's empty windows
//! make the same calls.
//!
//! **The schedule** (§3): an untimed preamble, then round −1 and the measured rounds, then
//! the postflight.
//! - **The preamble:**
//!   - a sizing pass (last columns only, one block at a time);
//!   - the memory preflight;
//!   - the decode with production's output;
//!   - the oracle images;
//!   - the one-pass precondition;
//!   - the untimed counter pass;
//!   - the calibration.
//! - **Each round** runs four passes, each under both arms in the round's order: W1 warm,
//!   W4 warm, W1 cold, W4 cold.
//! - **W4** is four pinned workers, dealt the blocks by the snake. Each step: preparation, the
//!   spinning start barrier, the timed build, the completion barrier, then the untimed check
//!   and record.
//! - **Warm** reuses each worker's arenas; **cold** drops them in the preparation, outside the
//!   window.
//!
//! **The checks:**
//! - **G6, every build:** C returned true and its arrays equal the oracle image byte for
//!   byte; P's table equals its reference.
//! - **G7, round −1's W1 warm pass:**
//!   - C's record arena's spare capacity is poisoned before its build, and the record says
//!     how much was poisoned and whether the poisoned allocation survived the build (it
//!     doesn't when the constructor had to grow it); a separate untimed pass before round −1
//!     builds every block into a preallocated, poisoned arena, for universal coverage;
//!   - P's table is checked against the counting-scatter definition, then becomes the
//!     block's reference;
//!   - both arms' tables are walked by their unchanged walkers against production's output,
//!     and checked again after the walk.
//!
//! **Reading production's table:** through `TVector::as_slice` and `TVector::capacity`,
//! the two read-only accessors report 195 authorises in `src/bwt.rs`. Nothing writes to
//! production's table outside its builder: an injected P fault corrupts a copy of the
//! expected values instead.
//!
//! **A capture consumes a dry run's evidence** (`--dry-run-evidence DIR`, report 197): the
//! directory of a completed dry run, which the collector verifies before anything else
//! (`verify_evidence`). The capture's memory requirement is max(estimate, the verified
//! peak), plus the registered margin.

use super::ctor::{Ctor, Stats};
use super::{
    blocks, cpu_now, fail, json_str, phi, pin, thread_cpu_ns, Counts, Image, RawG, Rec,
    SpinBarrier, C30, FAIL_PATH, ROWS,
};
use mibz2::bwt::{SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;

const FULL: usize = 899_000;
const CALIBRATION_N: usize = 20_000;
const NO_SUCH_CPU: usize = 1023;
const GIB: u64 = 1 << 30;
/// What the estimate adds for the binary, stacks, threads and allocator slack
const FIXED_OVERHEAD: u64 = 64 << 20;

/// The P arm's timed call: production's public builder
#[inline(never)]
fn p_build(ll: &[u8], counts: &[u32; 256], tt: &mut TVector) {
    mibz2::bwt::build(ll, counts, tt)
}

/// The C arm's timed call: the accepted constructor, natural configuration
#[inline(never)]
fn c_build(
    ctor: &mut Ctor,
    ll: &[u8],
    counts: &[u32; 256],
    rec: &mut Vec<Rec>,
    raw: &mut Vec<RawG>,
) -> bool {
    ctor.build::<false>(ll, counts, 0, rec, raw)
}

/// The calling thread's (minor, major) faults so far (`getrusage(RUSAGE_THREAD)`)
#[inline(always)]
fn faults() -> (i64, i64) {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn getrusage(who: i32, usage: *mut [i64; 18]) -> i32;
        }
        const RUSAGE_THREAD: i32 = 1;
        let mut u = [0i64; 18];
        // SAFETY: struct rusage is two timevals then fourteen longs: 18 i64 on x86-64 Linux
        if unsafe { getrusage(RUSAGE_THREAD, &mut u) } != 0 {
            fail("clock", "getrusage(RUSAGE_THREAD) failed".into());
        }
        // ru_minflt, ru_majflt
        (u[8], u[9])
    }
    #[cfg(not(target_os = "linux"))]
    {
        (0, 0)
    }
}

/// The process's major faults so far (`getrusage(RUSAGE_SELF)`)
fn self_majflt() -> i64 {
    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn getrusage(who: i32, usage: *mut [i64; 18]) -> i32;
        }
        let mut u = [0i64; 18];
        // SAFETY: as `faults`
        unsafe { getrusage(0, &mut u) };
        u[9]
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// A timed window's readings
#[derive(Clone, Copy, Default)]
struct Win {
    start: u64,
    end: u64,
    cpu_start: u64,
    cpu_end: u64,
    minflt: i64,
    majflt: i64,
}

/// The window around `f`: monotonic, CPU, rusage, `f`, rusage, CPU, monotonic
#[inline(always)]
fn window<R>(epoch: Instant, f: impl FnOnce() -> R) -> (R, Win) {
    let w0 = Instant::now();
    let c0 = thread_cpu_ns();
    let r0 = faults();
    let r = f();
    let r1 = faults();
    let c1 = thread_cpu_ns();
    let w1 = Instant::now();
    let ns = |t: Instant| t.duration_since(epoch).as_nanos() as u64;
    (
        r,
        Win {
            start: ns(w0),
            end: ns(w1),
            cpu_start: c0,
            cpu_end: c1,
            minflt: r1.0 - r0.0,
            majflt: r1.1 - r0.1,
        },
    )
}

/// The clock calls' cost: `n` empty windows on each CPU, pinned
fn calibrate(cpus: &[usize], n: usize, epoch: Instant) -> String {
    let mut parts = Vec::new();
    for &cpu in cpus {
        let (mut cw, mut ww): (Vec<u64>, Vec<u64>) = std::thread::scope(|sc| {
            sc.spawn(|| {
                if !pin(cpu) {
                    fail("pin", format!("calibration: could not pin to CPU {cpu}"));
                }
                let (mut cw, mut ww) = (Vec::with_capacity(n), Vec::with_capacity(n));
                for _ in 0..n {
                    let ((), w) = window(epoch, || {});
                    cw.push(w.cpu_end - w.cpu_start);
                    ww.push(w.end - w.start);
                }
                (cw, ww)
            })
            .join()
            .unwrap()
        });
        cw.sort_unstable();
        ww.sort_unstable();
        let q = |v: &[u64], f: f64| v[((v.len() - 1) as f64 * f) as usize];
        let qs = |v: &[u64]| {
            format!(
                "[{},{},{},{},{}]",
                q(v, 0.0),
                q(v, 0.5),
                q(v, 0.9),
                q(v, 0.99),
                q(v, 1.0)
            )
        };
        parts.push(format!(
            "{{\"cpu\":{cpu},\"n\":{n},\"cpu_ns\":{},\"wall_ns\":{}}}",
            qs(&cw),
            qs(&ww)
        ));
    }
    parts.join(",")
}

// ---------------------------------------------------------------------------------------
// Memory: the estimate, the preflight and the postflight (§3, G9)
// ---------------------------------------------------------------------------------------

fn proc_kv(path: &str, key: &str) -> Option<u64> {
    let s = std::fs::read_to_string(path).ok()?;
    for line in s.lines() {
        let mut it = line.split_whitespace();
        if it.next()?.trim_end_matches(':') == key {
            let v: u64 = it.next()?.parse().ok()?;
            let unit = it.next();
            return Some(if unit == Some("kB") { v * 1024 } else { v });
        }
    }
    None
}

/// /proc/pressure/memory: absent (no file), read, or unreadable (the file exists but can't
/// be read or parsed). Unreadable never counts as no pressure.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Psi {
    Absent,
    Read(f64, f64),
    Unreadable,
}

fn psi() -> Psi {
    let path = std::path::Path::new("/proc/pressure/memory");
    if !path.exists() {
        return Psi::Absent;
    }
    let Ok(s) = std::fs::read_to_string(path) else {
        return Psi::Unreadable;
    };
    let get = |kind: &str| -> Option<f64> {
        let l = s.lines().find(|l| l.starts_with(kind))?;
        let f = l.split_whitespace().find(|f| f.starts_with("avg10="))?;
        f["avg10=".len()..].parse().ok()
    };
    match (get("some"), get("full")) {
        (Some(a), Some(b)) => Psi::Read(a, b),
        _ => Psi::Unreadable,
    }
}

/// The memory state. Every source that Linux always has (/proc/meminfo's three fields,
/// /proc/vmstat's pswpin and pswpout) is required: a missing or unparseable one is
/// recorded as null and makes the state unreadable, never zero.
struct MemState {
    available: Option<u64>,
    swap_total: Option<u64>,
    swap_free: Option<u64>,
    psi: Psi,
    pswpin: Option<u64>,
    pswpout: Option<u64>,
    rss: Option<u64>,
    hwm: Option<u64>,
}

fn opt(v: Option<u64>) -> String {
    v.map_or("null".into(), |x| x.to_string())
}

impl MemState {
    fn read() -> MemState {
        MemState {
            available: proc_kv("/proc/meminfo", "MemAvailable"),
            swap_total: proc_kv("/proc/meminfo", "SwapTotal"),
            swap_free: proc_kv("/proc/meminfo", "SwapFree"),
            psi: psi(),
            pswpin: proc_kv("/proc/vmstat", "pswpin"),
            pswpout: proc_kv("/proc/vmstat", "pswpout"),
            rss: proc_kv("/proc/self/status", "VmRSS"),
            hwm: proc_kv("/proc/self/status", "VmHWM"),
        }
    }
    /// Whether every required source was read
    fn readable(&self) -> bool {
        self.available.is_some()
            && self.swap_total.is_some()
            && self.swap_free.is_some()
            && self.pswpin.is_some()
            && self.pswpout.is_some()
            && self.psi != Psi::Unreadable
    }
    fn swap_used(&self) -> Option<u64> {
        Some(self.swap_total?.saturating_sub(self.swap_free?))
    }
    /// No pressure: PSI read as zero, or absent (recorded as such)
    fn pressure_free(&self) -> bool {
        match self.psi {
            Psi::Absent => true,
            Psi::Read(a, b) => a == 0.0 && b == 0.0,
            Psi::Unreadable => false,
        }
    }
    fn json(&self) -> String {
        let (status, some, full) = match self.psi {
            Psi::Absent => ("absent", "null".to_string(), "null".to_string()),
            Psi::Read(a, b) => ("read", a.to_string(), b.to_string()),
            Psi::Unreadable => ("unreadable", "null".to_string(), "null".to_string()),
        };
        format!(
            "\"readable\":{},\"mem_available\":{},\"swap_total\":{},\"swap_free\":{},\"swap_used\":{},\"psi_status\":\"{status}\",\"psi_present\":{},\"psi_some_avg10\":{some},\"psi_full_avg10\":{full},\"pswpin\":{},\"pswpout\":{},\"vm_rss\":{},\"vm_hwm\":{}",
            self.readable(),
            opt(self.available),
            opt(self.swap_total),
            opt(self.swap_free),
            opt(self.swap_used()),
            self.psi != Psi::Absent,
            opt(self.pswpin),
            opt(self.pswpout),
            opt(self.rss),
            opt(self.hwm)
        )
    }
}

/// The natural RAW bound of a block (as `Ctor::build` reserves it)
fn raw_bound(n: usize, counts: &[u32; 256]) -> usize {
    if n == 0 {
        return 0;
    }
    let g = n.div_ceil(ROWS);
    let sigma = counts.iter().filter(|&&c| c > 0).count();
    g.min(sigma * (phi(n - 1) as usize / 65_534) + sigma - 1)
}

/// The capture process's estimated peak memory, from the blocks' sizes and counts, and its
/// parts (§3)
fn estimate(sizes: &[(usize, [u32; 256])]) -> (u64, String) {
    let c_bytes = |n: usize, c: &[u32; 256]| -> u64 {
        let g = n.div_ceil(ROWS) as u64;
        64 * (g + 1) + 128 * raw_bound(n, c) as u64 + 8 * g.div_ceil(64)
    };
    let total: u64 = sizes.iter().map(|(n, _)| *n as u64).sum();
    let nmax = sizes.iter().map(|(n, _)| *n).max().unwrap_or(0) as u64;
    let cmax = sizes.iter().map(|(n, c)| c_bytes(*n, c)).max().unwrap_or(0);
    let inputs = 2 * total + 1024 * sizes.len() as u64;
    let images: u64 = sizes.iter().map(|(n, c)| c_bytes(*n, c)).sum();
    let references = 4 * total;
    // per worker: warm P, warm C, one cold arena at a time
    let workers = 4 * (4 * nmax + cmax + (4 * nmax).max(cmax));
    // worker 0's round -1 walks: the walkers' segment buffers and outputs
    let walk_scratch = 3 * (2 * nmax + (64 << 10)) + 2 * nmax;
    // the oracle's transient (Ψ, F, classes, its own arrays) and the counter pass's arena
    let oracle = 5 * nmax
        + nmax.div_ceil(30)
        + 64 * (nmax.div_ceil(30) + 1)
        + 128 * nmax.div_ceil(30)
        + cmax;
    // the decode's transients (production's inverter: table, segments, symbols)
    let decode = 8 * nmax;
    let sum =
        inputs + images + references + workers + walk_scratch + oracle + decode + FIXED_OVERHEAD;
    let parts = format!(
        "{{\"inputs\":{inputs},\"oracle_images\":{images},\"p_references\":{references},\"worker_arenas\":{workers},\"walk_scratch\":{walk_scratch},\"oracle_transient\":{oracle},\"decode_transient\":{decode},\"fixed\":{FIXED_OVERHEAD}}}"
    );
    (sum, parts)
}

// ---------------------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------------------

/// Each block's last column and counts, decoded one block at a time (no inversion)
fn last_columns(data: &[u8], keep: bool) -> Vec<(Vec<u8>, [u32; 256])> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut v = Vec::new();
    let mut ll = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        let mut counts = [0u32; 256];
        mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        if keep {
            v.push((std::mem::take(&mut ll), counts));
        } else {
            v.push((vec![0u8; 0], counts));
            // the size alone: kept in the counts' sum
        }
        syms = symbols.syms;
    }
    v
}

/// The registered four-worker assignment (the traversal protocol's snake)
fn snake(rows: &[usize]) -> [Vec<usize>; 4] {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(rows[i]), i));
    let mut lists: [Vec<usize>; 4] = Default::default();
    for (k, chunk) in order.chunks(4).enumerate() {
        for (j, &b) in chunk.iter().enumerate() {
            lists[if k % 2 == 0 { j } else { 3 - j }].push(b);
        }
    }
    lists
}

fn bytes<T>(v: &[T]) -> &[u8] {
    // SAFETY: Rec and RawG have no padding; u32 has none
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

/// The allocator's usable size behind an allocation of `cap` elements at `p` (0 when
/// there is none, or off glibc)
fn usable_at<T>(p: *const T, cap: usize) -> u64 {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        extern "C" {
            fn malloc_usable_size(p: *mut std::ffi::c_void) -> usize;
        }
        if cap == 0 || std::mem::size_of::<T>() == 0 {
            return 0;
        }
        // SAFETY: the pointer of a live, nonempty allocation of the system allocator
        unsafe { malloc_usable_size(p as *mut std::ffi::c_void) as u64 }
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        let _ = (p, cap);
        0
    }
}

fn usable<T>(v: &Vec<T>) -> u64 {
    usable_at(v.as_ptr(), v.capacity())
}

/// Poison a record arena's spare capacity (G7a)
fn poison(rec: &mut Vec<Rec>) {
    rec.clear();
    for s in rec.spare_capacity_mut() {
        s.write(Rec {
            f: [0xA5A5; 30],
            h: 0xA5A5_A5A5,
        });
    }
}

// ---------------------------------------------------------------------------------------
// The workers
// ---------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Arm {
    P,
    C,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Regime {
    Warm,
    Cold,
}

fn arm_name(a: Arm) -> &'static str {
    if a == Arm::P {
        "P"
    } else {
        "C"
    }
}

fn regime_name(r: Regime) -> &'static str {
    if r == Regime::Warm {
        "warm"
    } else {
        "cold"
    }
}

/// One arm's arenas
#[derive(Default)]
struct Arenas {
    tt: TVector,
    ctor: Ctor,
    rec: Vec<Rec>,
    raw: Vec<RawG>,
}

impl Arenas {
    /// (capacity bytes, allocator usable bytes) of the arm's table. P: the T-vector. C:
    /// `rec` + `raw` only; the constructor's bitmap is private to it and is not observed
    /// (its requested size is recorded separately).
    fn caps(&self, arm: Arm) -> (u64, u64) {
        match arm {
            Arm::P => (
                4 * self.tt.capacity() as u64,
                usable_at(self.tt.as_slice().as_ptr(), self.tt.capacity()),
            ),
            Arm::C => (
                64 * self.rec.capacity() as u64 + 128 * self.raw.capacity() as u64,
                usable(&self.rec) + usable(&self.raw),
            ),
        }
    }
}

/// A worker: its warm arenas (kept across blocks and rounds), its cold arenas (fresh for
/// each build), and the round −1 walkers' scratch
#[derive(Default)]
struct Worker {
    warm: Arenas,
    cold: Arenas,
    walk: C30,
    seg: SegScratch,
    out: Vec<u8>,
    /// round −1's G7 poison: the records of capacity poisoned, and the allocation's address
    poison_cap: usize,
    poison_ptr: usize,
}

/// A fault injected on purpose (dry runs only)
#[derive(Clone, Copy, PartialEq)]
enum Inject {
    None,
    /// flip one entry of this worker's first table in round −1's warm pass of this mode
    /// and arm, after the completion barrier, before the check
    Output {
        w4: bool,
        arm: Arm,
        worker: usize,
    },
    /// pin this worker to a CPU that doesn't exist, in round −1's warm pass of this mode
    Pin {
        w4: bool,
        worker: usize,
    },
    /// give C counts that are not L's own in this worker's first round −1 warm build
    Reject {
        w4: bool,
        worker: usize,
    },
    /// fail the memory preflight
    Preflight,
}

fn parse_inject(s: &str) -> Inject {
    let p: Vec<&str> = s.split(':').collect();
    let w4 = |m: &str| match m {
        "w1" => false,
        "w4" => true,
        _ => panic!("--inject: mode w1 or w4"),
    };
    match p.as_slice() {
        ["output", m, a, k] => Inject::Output {
            w4: w4(m),
            arm: if *a == "P" { Arm::P } else { Arm::C },
            worker: k.parse().unwrap(),
        },
        ["pin", m, k] => Inject::Pin {
            w4: w4(m),
            worker: k.parse().unwrap(),
        },
        ["reject", m, k] => Inject::Reject {
            w4: w4(m),
            worker: k.parse().unwrap(),
        },
        ["preflight"] => Inject::Preflight,
        _ => panic!(
            "--inject output:MODE:ARM:WORKER, pin:MODE:WORKER, reject:MODE:WORKER or preflight"
        ),
    }
}

/// The experiment's shared, read-only state
struct Exp<'a> {
    ll: &'a [Vec<u8>],
    counts: &'a [[u32; 256]],
    op: &'a [u32],
    want: &'a [Vec<u8>],
    imgs: &'a [Image],
    /// P's reference tables, set in round −1's first P pass (W1 warm)
    refs: &'a Mutex<Vec<Vec<u32>>>,
    epoch: Instant,
    inject: Inject,
    /// round −1's G7 records
    check0: &'a Mutex<Vec<String>>,
}

/// What one build's step context is
#[derive(Clone, Copy)]
struct Step {
    round: i64,
    regime: Regime,
    w4: bool,
    arm: Arm,
    worker: usize,
    step: usize,
    block: usize,
}

impl Worker {
    /// The untimed preparation before the start barrier. Cold: the previous arenas are
    /// dropped. Round −1's W1 warm C build: its record arena is poisoned (G7a).
    fn prepare(&mut self, s: &Step) {
        if s.regime == Regime::Cold {
            self.cold = Arenas::default();
        } else if s.round == -1 && !s.w4 && s.arm == Arm::C {
            poison(&mut self.warm.rec);
            self.poison_cap = self.warm.rec.capacity();
            self.poison_ptr = self.warm.rec.as_ptr() as usize;
        }
    }

    /// The timed build
    fn build(&mut self, s: &Step, e: &Exp) -> (bool, Win) {
        let b = s.block;
        let a = match s.regime {
            Regime::Warm => &mut self.warm,
            Regime::Cold => &mut self.cold,
        };
        let ll = e.ll[b].as_slice();
        let mut counts = e.counts[b];
        if let Inject::Reject { w4, worker } = e.inject {
            if s.round == -1
                && s.regime == Regime::Warm
                && s.arm == Arm::C
                && s.w4 == w4
                && s.worker == worker
                && s.step == 0
            {
                // the same total, one count moved: not L's own counts
                let x = ll[0] as usize;
                counts[x] -= 1;
                counts[(x + 1) % 256] += 1;
            }
        }
        match s.arm {
            Arm::P => {
                let ((), w) = window(e.epoch, || p_build(ll, &counts, &mut a.tt));
                (true, w)
            }
            Arm::C => window(e.epoch, || {
                c_build(&mut a.ctor, ll, &counts, &mut a.rec, &mut a.raw)
            }),
        }
    }

    /// The untimed check after the completion barrier (G6, and G7 in round −1's W1 warm
    /// pass). Ends the run on any mismatch.
    fn check(&mut self, s: &Step, ok: bool, e: &Exp) {
        let b = s.block;
        let what = format!(
            "round {} {} {} arm {} worker {} step {} block {b}",
            s.round,
            regime_name(s.regime),
            if s.w4 { "w4" } else { "w1" },
            arm_name(s.arm),
            s.worker,
            s.step
        );
        // an injected fault (dry runs only): C's own table is corrupted (the harness owns
        // it); for P, a copy of the expected values is, since production's table has no
        // mutable access
        let mut corrupt_expected = false;
        if let Inject::Output { w4, arm, worker } = e.inject {
            if s.round == -1
                && s.regime == Regime::Warm
                && s.w4 == w4
                && s.arm == arm
                && s.worker == worker
                && s.step == 0
            {
                match arm {
                    Arm::C => self.warm.rec[0].f[0] ^= 1,
                    Arm::P => corrupt_expected = true,
                }
            }
        }
        let g7 = s.round == -1 && s.regime == Regime::Warm && !s.w4;
        let a = match s.regime {
            Regime::Warm => &mut self.warm,
            Regime::Cold => &mut self.cold,
        };
        match s.arm {
            Arm::C => {
                if !ok {
                    fail(
                        "rejection",
                        format!("{what}: the constructor rejected the block"),
                    );
                }
                let img = &e.imgs[b];
                if bytes(&a.rec) != bytes(&img.rec) || bytes(&a.raw) != bytes(&img.raw) {
                    fail(
                        "output",
                        format!("{what}: compact30's arrays differ from the oracle image"),
                    );
                }
                if g7 {
                    // G7b, c: the unchanged walker on the arrays, then the arrays again
                    std::mem::swap(&mut self.walk.rec, &mut a.rec);
                    std::mem::swap(&mut self.walk.raw, &mut a.raw);
                    self.walk.n = e.ll[b].len();
                    let mut d = Counts::default();
                    self.walk
                        .walk::<false>(e.op[b], false, &mut self.out, &mut d);
                    let walk_ok = self.out == e.want[b];
                    std::mem::swap(&mut self.walk.rec, &mut a.rec);
                    std::mem::swap(&mut self.walk.raw, &mut a.raw);
                    let restored =
                        bytes(&a.rec) == bytes(&img.rec) && bytes(&a.raw) == bytes(&img.raw);
                    if !walk_ok || !restored {
                        fail(
                            "output",
                            format!("{what}: walker output {walk_ok}, restored {restored}"),
                        );
                    }
                    // what the poison actually covered: the capacity poisoned, and whether that
                    // allocation survived the build (a growing constructor replaces it)
                    let survived = self.poison_cap > 0
                        && a.rec.as_ptr() as usize == self.poison_ptr
                        && self.poison_cap >= a.rec.len();
                    e.check0.lock().unwrap().push(format!(
                        "{{\"kind\":\"check0\",\"arm\":\"C\",\"block\":{b},\"poisoned_capacity\":{},\"needed\":{},\"poison_survived\":{survived},\"exact\":true,\"walk\":true,\"restored\":true}}",
                        self.poison_cap,
                        a.rec.len()
                    ));
                }
            }
            Arm::P => {
                if g7 {
                    // the counting-scatter definition, independently of production's code
                    let t = a.tt.as_slice();
                    let ll = &e.ll[b];
                    let mut ftab = [0usize; 256];
                    let mut cum = 0;
                    for c in 0..256 {
                        ftab[c] = cum;
                        cum += e.counts[b][c] as usize;
                    }
                    let mut def = t.len() == ll.len();
                    for (i, &c) in ll.iter().enumerate() {
                        let sl = ftab[c as usize];
                        ftab[c as usize] += 1;
                        let mut expected = ll[sl] as u32 | (i as u32) << 8;
                        if corrupt_expected && i == 0 {
                            expected ^= 1 << 8;
                        }
                        def &= t.get(sl) == Some(&expected);
                    }
                    let reference = t.to_vec();
                    walk_segments(&mut a.tt, e.op[b], &mut self.out, &mut self.seg);
                    let walk_ok = self.out == e.want[b];
                    let restored = a.tt.as_slice() == reference.as_slice();
                    if !(def && walk_ok && restored) {
                        fail(
                            "output",
                            format!("{what}: P's table: definition {def}, walk {walk_ok}, restored {restored}"),
                        );
                    }
                    e.refs.lock().unwrap()[b] = reference;
                    e.check0.lock().unwrap().push(format!(
                        "{{\"kind\":\"check0\",\"arm\":\"P\",\"block\":{b},\"definition\":true,\"walk\":true,\"restored\":true}}"
                    ));
                } else {
                    let refs = e.refs.lock().unwrap();
                    let differs = if corrupt_expected {
                        let mut expected = refs[b].clone();
                        expected[0] ^= 1 << 8;
                        a.tt.as_slice() != expected.as_slice()
                    } else {
                        a.tt.as_slice() != refs[b].as_slice()
                    };
                    if refs[b].len() != a.tt.len() || differs {
                        fail(
                            "output",
                            format!("{what}: production's table differs from its reference"),
                        );
                    }
                }
            }
        }
    }
}

/// Production's walk on P's table, as the decoder calls it
fn walk_segments(tt: &mut TVector, op: u32, out: &mut Vec<u8>, s: &mut SegScratch) {
    mibz2::bwt::walk_segments_room(tt, op, out, s)
}

#[allow(clippy::too_many_arguments)]
fn record(
    s: &Step,
    rows: usize,
    cpu: (i64, i64),
    prep: (u64, u64),
    w: &Win,
    check: (u64, u64),
    caps: (u64, u64),
) -> String {
    format!(
        "{{\"kind\":\"build\",\"round\":{},\"regime\":\"{}\",\"mode\":\"{}\",\"arm\":\"{}\",\"worker\":{},\"step\":{},\"block\":{},\"rows\":{rows},\"cpu_before\":{},\"cpu_after\":{},\"prep_start\":{},\"prep_end\":{},\"start\":{},\"end\":{},\"cpu_start\":{},\"cpu_end\":{},\"minflt\":{},\"majflt\":{},\"check_start\":{},\"check_end\":{},\"cap_bytes\":{},\"usable_bytes\":{},\"cap_scope\":\"{}\",\"bitmap_req_bytes\":{},\"exact\":true}}",
        s.round,
        regime_name(s.regime),
        if s.w4 { "w4" } else { "w1" },
        arm_name(s.arm),
        s.worker,
        s.step,
        s.block,
        cpu.0,
        cpu.1,
        prep.0,
        prep.1,
        w.start,
        w.end,
        w.cpu_start,
        w.cpu_end,
        w.minflt,
        w.majflt,
        check.0,
        check.1,
        caps.0,
        caps.1,
        if s.arm == Arm::P { "tt" } else { "rec+raw" },
        if s.arm == Arm::P { 0 } else { 8 * rows.div_ceil(ROWS).div_ceil(64) }
    )
}

fn pin_or_fail(cpu: usize, what: &str) {
    if !pin(cpu) {
        fail("pin", format!("{what}: could not pin to CPU {cpu}"));
    }
}

/// One W1 pass: worker 0 on the first CPU, every block in corpus order
fn pass_w1(
    w: &mut Worker,
    round: i64,
    regime: Regime,
    arm: Arm,
    cpu: usize,
    e: &Exp,
) -> Vec<String> {
    let ns = |t: Instant| t.duration_since(e.epoch).as_nanos() as u64;
    std::thread::scope(|sc| {
        sc.spawn(|| {
            let cpu = match e.inject {
                Inject::Pin { w4: false, .. } if round == -1 && regime == Regime::Warm => {
                    NO_SUCH_CPU
                }
                _ => cpu,
            };
            pin_or_fail(cpu, "W1 worker 0");
            let mut v = Vec::with_capacity(e.ll.len());
            for b in 0..e.ll.len() {
                let s = Step {
                    round,
                    regime,
                    w4: false,
                    arm,
                    worker: 0,
                    step: b,
                    block: b,
                };
                let p0 = ns(Instant::now());
                w.prepare(&s);
                let p1 = ns(Instant::now());
                let c0 = cpu_now();
                let (ok, win) = w.build(&s, e);
                let c1 = cpu_now();
                let k0 = ns(Instant::now());
                w.check(&s, ok, e);
                let k1 = ns(Instant::now());
                let caps = match regime {
                    Regime::Warm => w.warm.caps(arm),
                    Regime::Cold => w.cold.caps(arm),
                };
                v.push(record(
                    &s,
                    e.ll[b].len(),
                    (c0, c1),
                    (p0, p1),
                    &win,
                    (k0, k1),
                    caps,
                ));
            }
            if regime == Regime::Cold {
                w.cold = Arenas::default();
            }
            v
        })
        .join()
        .unwrap()
    })
}

/// One W4 pass: four pinned workers in lockstep by step
fn pass_w4(
    ws: &mut [Worker],
    lists: &[Vec<usize>; 4],
    round: i64,
    regime: Regime,
    arm: Arm,
    cpus: &[usize],
    e: &Exp,
) -> Vec<String> {
    let steps = lists.iter().map(|l| l.len()).max().unwrap();
    let start = SpinBarrier::new(4);
    let done = std::sync::Barrier::new(4);
    let ns = |t: Instant| t.duration_since(e.epoch).as_nanos() as u64;
    let r: Vec<Vec<String>> = std::thread::scope(|sc| {
        let hs: Vec<_> = ws
            .iter_mut()
            .enumerate()
            .map(|(k, w)| {
                let (list, start, done) = (&lists[k], &start, &done);
                sc.spawn(move || {
                    let cpu = match e.inject {
                        Inject::Pin { w4: true, worker }
                            if worker == k && round == -1 && regime == Regime::Warm =>
                        {
                            NO_SUCH_CPU
                        }
                        _ => cpus[k],
                    };
                    pin_or_fail(cpu, &format!("W4 worker {k}"));
                    let mut v = Vec::with_capacity(list.len());
                    for step in 0..steps {
                        let item = list.get(step).map(|&b| {
                            let s = Step {
                                round,
                                regime,
                                w4: true,
                                arm,
                                worker: k,
                                step,
                                block: b,
                            };
                            let p0 = ns(Instant::now());
                            w.prepare(&s);
                            let p1 = ns(Instant::now());
                            (s, p0, p1, cpu_now())
                        });
                        start.wait();
                        let built = item.map(|(s, ..)| w.build(&s, e));
                        done.wait();
                        let c1 = cpu_now();
                        if let (Some((s, p0, p1, c0)), Some((ok, win))) = (item, built) {
                            let k0 = ns(Instant::now());
                            w.check(&s, ok, e);
                            let k1 = ns(Instant::now());
                            let caps = match regime {
                                Regime::Warm => w.warm.caps(arm),
                                Regime::Cold => w.cold.caps(arm),
                            };
                            v.push(record(
                                &s,
                                e.ll[s.block].len(),
                                (c0, c1),
                                (p0, p1),
                                &win,
                                (k0, k1),
                                caps,
                            ));
                        }
                    }
                    if regime == Regime::Cold {
                        w.cold = Arenas::default();
                    }
                    v
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    r.into_iter().flatten().collect()
}

fn append(path: &str, lines: &[String]) {
    let mut f = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    for l in lines {
        writeln!(f, "{l}").unwrap();
    }
}

fn arg_cpus(s: &str) -> Vec<usize> {
    s.split(',').map(|c| c.parse().unwrap()).collect()
}

// ---------------------------------------------------------------------------------------
// Identity: SHA-256, the machine, and a minimal JSON reader for the dry run's evidence
// ---------------------------------------------------------------------------------------

/// SHA-256 (FIPS 180-4), for the corpus, this binary and the evidence files
fn sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (t, word) in chunk.chunks(4).enumerate() {
            w[t] = u32::from_be_bytes(word.try_into().unwrap());
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for t in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (x, y) in h.iter_mut().zip(v) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

fn sha256_file(path: &str) -> Option<String> {
    std::fs::read(path).ok().map(|d| sha256(&d))
}

fn read_trim(path: &str) -> String {
    std::fs::read_to_string(path).map_or_else(|_| "unreadable".into(), |s| s.trim().to_string())
}

/// The machine's identity, as a canonical JSON object: hostname, machine-id, kernel, the
/// CPU model, the online CPUs and the installed memory. A capture must match its dry run's.
fn machine() -> String {
    let model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|m| m.trim().to_string())
        })
        .unwrap_or_else(|| "unreadable".into());
    format!(
        "{{\"hostname\":{},\"machine_id\":{},\"kernel\":{},\"cpu_model\":{},\"cpus_online\":{},\"mem_total\":{}}}",
        json_str(&read_trim("/proc/sys/kernel/hostname")),
        json_str(&read_trim("/etc/machine-id")),
        json_str(&read_trim("/proc/sys/kernel/osrelease")),
        json_str(&model),
        json_str(&read_trim("/sys/devices/system/cpu/online")),
        opt(proc_kv("/proc/meminfo", "MemTotal"))
    )
}

/// A JSON value, as the evidence files hold them
#[derive(Clone, Debug, PartialEq)]
enum J {
    Null,
    Bool(bool),
    Num(f64, String),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(v) => v.iter().find(|(x, _)| x == k).map(|(_, y)| y),
            _ => None,
        }
    }
    fn u64(&self) -> Option<u64> {
        match self {
            J::Num(_, raw) => raw.parse().ok(),
            _ => None,
        }
    }
    fn str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
    /// The canonical text of a value (objects in their order), for equality across files
    fn canon(&self) -> String {
        match self {
            J::Null => "null".into(),
            J::Bool(b) => b.to_string(),
            J::Num(_, raw) => raw.clone(),
            J::Str(s) => json_str(s),
            J::Arr(v) => format!("[{}]", v.iter().map(J::canon).collect::<Vec<_>>().join(",")),
            J::Obj(v) => format!(
                "{{{}}}",
                v.iter()
                    .map(|(k, x)| format!("{}:{}", json_str(k), x.canon()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

/// A strict JSON parser (RFC 8259 values; no trailing input)
fn parse_json(s: &str) -> Result<J, String> {
    fn ws(b: &[u8], i: &mut usize) {
        while *i < b.len() && matches!(b[*i], b' ' | b'\t' | b'\n' | b'\r') {
            *i += 1;
        }
    }
    fn val(b: &[u8], i: &mut usize, depth: usize) -> Result<J, String> {
        if depth > 64 {
            return Err("too deep".into());
        }
        ws(b, i);
        match b.get(*i) {
            Some(b'{') => {
                *i += 1;
                let mut v = Vec::new();
                ws(b, i);
                if b.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Ok(J::Obj(v));
                }
                loop {
                    ws(b, i);
                    let J::Str(k) = string(b, i)? else {
                        return Err("key".into());
                    };
                    ws(b, i);
                    if b.get(*i) != Some(&b':') {
                        return Err("colon".into());
                    }
                    *i += 1;
                    v.push((k, val(b, i, depth + 1)?));
                    ws(b, i);
                    match b.get(*i) {
                        Some(b',') => *i += 1,
                        Some(b'}') => {
                            *i += 1;
                            return Ok(J::Obj(v));
                        }
                        _ => return Err("object".into()),
                    }
                }
            }
            Some(b'[') => {
                *i += 1;
                let mut v = Vec::new();
                ws(b, i);
                if b.get(*i) == Some(&b']') {
                    *i += 1;
                    return Ok(J::Arr(v));
                }
                loop {
                    v.push(val(b, i, depth + 1)?);
                    ws(b, i);
                    match b.get(*i) {
                        Some(b',') => *i += 1,
                        Some(b']') => {
                            *i += 1;
                            return Ok(J::Arr(v));
                        }
                        _ => return Err("array".into()),
                    }
                }
            }
            Some(b'"') => string(b, i),
            Some(b't') if b[*i..].starts_with(b"true") => {
                *i += 4;
                Ok(J::Bool(true))
            }
            Some(b'f') if b[*i..].starts_with(b"false") => {
                *i += 5;
                Ok(J::Bool(false))
            }
            Some(b'n') if b[*i..].starts_with(b"null") => {
                *i += 4;
                Ok(J::Null)
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                let st = *i;
                *i += 1;
                while *i < b.len()
                    && matches!(b[*i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    *i += 1;
                }
                let raw = std::str::from_utf8(&b[st..*i]).map_err(|_| "number")?;
                let f: f64 = raw.parse().map_err(|_| format!("number {raw}"))?;
                Ok(J::Num(f, raw.to_string()))
            }
            _ => Err(format!("unexpected input at {}", *i)),
        }
    }
    fn string(b: &[u8], i: &mut usize) -> Result<J, String> {
        if b.get(*i) != Some(&b'"') {
            return Err("string".into());
        }
        *i += 1;
        let mut out = String::new();
        loop {
            let c = *b.get(*i).ok_or("unterminated string")?;
            *i += 1;
            match c {
                b'"' => return Ok(J::Str(out)),
                b'\\' => {
                    let e = *b.get(*i).ok_or("escape")?;
                    *i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'u' => {
                            let h = std::str::from_utf8(b.get(*i..*i + 4).ok_or("escape")?)
                                .map_err(|_| "escape")?;
                            let cp = u32::from_str_radix(h, 16).map_err(|_| "escape")?;
                            *i += 4;
                            out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        }
                        _ => return Err("escape".into()),
                    }
                }
                _ => {
                    // the rest of a UTF-8 sequence is copied byte for byte
                    let st = *i - 1;
                    let mut en = *i;
                    while en < b.len() && b[en] != b'"' && b[en] != b'\\' {
                        en += 1;
                    }
                    out.push_str(std::str::from_utf8(&b[st..en]).map_err(|_| "utf-8")?);
                    *i = en;
                }
            }
        }
    }
    let b = s.as_bytes();
    let mut i = 0;
    let v = val(b, &mut i, 0)?;
    ws(b, &mut i);
    if i != b.len() {
        return Err("trailing input".into());
    }
    Ok(v)
}

/// What this capture is, for comparison with its dry run's evidence
struct Own {
    corpus: String,
    corpus_sha256: String,
    binary_sha256: String,
    cpus: Vec<usize>,
    machine: String,
    rows: u64,
}

/// A verified dry run
struct Evidence {
    peak: u64,
    commit: String,
    jsonl_sha256: String,
    analysis_sha256: String,
    manifest_sha256: String,
}

/// Verify a completed dry run's directory (report 197): its records, its analysis and its
/// manifest, against each other and against this capture. Every condition is required;
/// the first that fails is returned.
fn verify_evidence(dir: &str, own: &Own) -> Result<Evidence, String> {
    let path = |f: &str| format!("{dir}/{f}");
    let read = |f: &str| -> Result<String, String> {
        std::fs::read_to_string(path(f)).map_err(|e| format!("{f}: {e}"))
    };
    let (jl, an, mf) = (
        read("c30-construction.jsonl")?,
        read("c30-analysis.json")?,
        read("MANIFEST.json")?,
    );
    let (jl_sha, an_sha, mf_sha) = (
        sha256(jl.as_bytes()),
        sha256(an.as_bytes()),
        sha256(mf.as_bytes()),
    );
    let man = parse_json(&mf).map_err(|e| format!("MANIFEST.json: {e}"))?;
    let ana = parse_json(&an).map_err(|e| format!("c30-analysis.json: {e}"))?;
    let need = |c: bool, what: &str| -> Result<(), String> {
        if c {
            Ok(())
        } else {
            Err(what.to_string())
        }
    };
    // the manifest: a passed run of a clean commit, these files, this binary, this corpus
    need(
        man.get("exit_status").and_then(J::u64) == Some(0),
        "the dry run's exit status is not 0",
    )?;
    let src = man.get("source").ok_or("the manifest has no source")?;
    let commit = src
        .get("commit")
        .and_then(J::str)
        .ok_or("the manifest has no commit")?
        .to_string();
    need(
        src.get("tracked_changes") == Some(&J::Arr(vec![])),
        "the dry run's source had tracked changes",
    )?;
    need(
        man.get("binary")
            .and_then(|b| b.get("sha256"))
            .and_then(J::str)
            == Some(own.binary_sha256.as_str()),
        "the dry run's binary is not this binary (a stale or different build)",
    )?;
    let corpora = match man.get("corpora") {
        Some(J::Arr(v)) => v.clone(),
        _ => vec![],
    };
    need(
        corpora.len() == 1
            && corpora[0].get("sha256").and_then(J::str) == Some(own.corpus_sha256.as_str()),
        "the dry run's corpus is not this corpus",
    )?;
    let files = match man.get("files") {
        Some(J::Arr(v)) => v.clone(),
        _ => vec![],
    };
    for (f, sha) in [
        ("c30-construction.jsonl", &jl_sha),
        ("c30-analysis.json", &an_sha),
    ] {
        need(
            files.iter().any(|x| {
                x.get("path").and_then(J::str) == Some(f)
                    && x.get("sha256").and_then(J::str) == Some(sha.as_str())
            }),
            &format!("{f} does not match the manifest's checksum"),
        )?;
    }
    // the analysis: a dry run whose schedule and every gate it checks passed
    need(
        ana.get("dry_run") == Some(&J::Bool(true)),
        "the analysis is not a dry run's",
    )?;
    let all_true = |o: Option<&J>| match o {
        Some(J::Obj(v)) => !v.is_empty() && v.iter().all(|(_, x)| *x == J::Bool(true)),
        _ => false,
    };
    need(
        all_true(ana.get("schedule")),
        "the dry run's schedule checks did not all pass",
    )?;
    let gates = ana.get("gates");
    for g in [
        "G3_identity",
        "G4_allocation",
        "G6_exact_output",
        "G7_round_minus_one",
        "G8_one_pass",
        "G9_memory",
        "G10_major_faults",
    ] {
        need(
            gates.and_then(|x| x.get(g)) == Some(&J::Bool(true)),
            &format!("the dry run's {g} did not pass"),
        )?;
    }
    need(
        ana.get("failures").is_none(),
        "the dry run recorded a failure",
    )?;
    // the records: a dry run of this corpus, binary, machine and CPU selection, completed,
    // with its peak
    let mut meta = None;
    let mut post = None;
    for line in jl.lines() {
        let r = parse_json(line).map_err(|e| format!("c30-construction.jsonl: {e}"))?;
        match r.get("kind").and_then(J::str) {
            Some("meta") => meta = Some(r),
            Some("postflight") => post = Some(r),
            Some("failure") => return Err("the dry run's records hold a failure".into()),
            _ => {}
        }
    }
    let meta = meta.ok_or("the dry run's records have no meta record")?;
    let post = post.ok_or("the dry run's records have no postflight (it did not complete)")?;
    need(
        meta.get("dry_run") == Some(&J::Bool(true)),
        "the records are not a dry run's",
    )?;
    need(
        meta.get("inject") == Some(&J::Null),
        "the dry run carried an injected fault",
    )?;
    need(
        meta.get("corpus").and_then(J::str) == Some(own.corpus.as_str()),
        "the dry run's corpus name differs",
    )?;
    need(
        meta.get("corpus_sha256").and_then(J::str) == Some(own.corpus_sha256.as_str()),
        "the dry run's corpus hash differs",
    )?;
    need(
        meta.get("binary_sha256").and_then(J::str) == Some(own.binary_sha256.as_str()),
        "the dry run's recorded binary differs",
    )?;
    need(
        meta.get("rows").and_then(J::u64) == Some(own.rows),
        "the dry run's rows differ",
    )?;
    let cpus = format!("{:?}", own.cpus).replace(' ', "");
    need(
        meta.get("cpus").map(J::canon) == Some(cpus),
        "the dry run's CPU selection differs from this capture's",
    )?;
    let mine = parse_json(&own.machine).map_err(|e| format!("own machine identity: {e}"))?;
    need(
        meta.get("machine") == Some(&mine),
        "the dry run ran on another machine (or one since changed)",
    )?;
    let peak = post
        .get("vm_hwm")
        .and_then(J::u64)
        .ok_or("the dry run's peak RSS is missing")?;
    need(
        ana.get("memory")
            .and_then(|m| m.get("peak_rss_vm_hwm"))
            .and_then(J::u64)
            == Some(peak),
        "the analysis's peak RSS is not the records' postflight peak",
    )?;
    // a plausibility floor: a dry run held every block's last column and production's
    // output, two bytes a row, so a smaller peak cannot be a real measurement
    need(peak >= 2 * own.rows, &format!("the dry run's peak RSS, {peak} bytes, is below the 2 bytes a row its inputs alone need"))?;
    Ok(Evidence {
        peak,
        commit,
        jsonl_sha256: jl_sha,
        analysis_sha256: an_sha,
        manifest_sha256: mf_sha,
    })
}

/// `cbuild`: the registered schedule (§3), or its dry run
pub fn main(args: &[String]) {
    let out = args[0].clone();
    let file = &args[1];
    let (mut rounds, mut cpus, mut dry) = (20i64, Vec::new(), false);
    let (mut inject, mut inject_arg) = (Inject::None, String::new());
    let (mut evidence_dir, mut verify_only) = (None::<String>, false);
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--rounds" => {
                rounds = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--cpus" => {
                cpus = arg_cpus(&args[i + 1]);
                i += 1;
            }
            "--dry-run" => dry = true,
            "--dry-run-evidence" => {
                evidence_dir = Some(args[i + 1].clone());
                i += 1;
            }
            "--verify-evidence-only" => verify_only = true,
            "--inject" => {
                inject = parse_inject(&args[i + 1]);
                inject_arg = args[i + 1].clone();
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    assert!(
        inject == Inject::None || dry,
        "--inject is for dry runs only"
    );
    if dry {
        rounds = 2;
    }
    assert!(
        cpus.len() == 4,
        "--cpus needs four CPUs, each a physical core"
    );
    std::fs::File::create(&out).unwrap();
    FAIL_PATH.set(out.clone()).unwrap();
    std::panic::set_hook(Box::new(|info| fail("panic", info.to_string())));
    let name = std::path::Path::new(file)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let data = std::fs::read(file).unwrap();
    let corpus_sha256 = sha256(&data);
    let binary_sha256 = sha256_file("/proc/self/exe").unwrap_or_else(|| "unreadable".into());
    let machine = machine();

    // the sizing pass, before any large allocation
    let sizes: Vec<(usize, [u32; 256])> = last_columns(&data, false)
        .into_iter()
        .map(|(_, c)| (c.iter().map(|&x| x as usize).sum(), c))
        .collect();
    let total_rows: u64 = sizes.iter().map(|(n, _)| *n as u64).sum();

    // a capture consumes a verified dry run (G9, report 197); a dry run takes none
    let evidence = match (dry, &evidence_dir) {
        (true, Some(_)) => fail("evidence", "a dry run takes no --dry-run-evidence".into()),
        (true, None) => None,
        (false, None) => fail(
            "evidence",
            "a capture needs --dry-run-evidence DIR, a completed dry run's directory".into(),
        ),
        (false, Some(d)) => {
            let own = Own {
                corpus: name.clone(),
                corpus_sha256: corpus_sha256.clone(),
                binary_sha256: binary_sha256.clone(),
                cpus: cpus.clone(),
                machine: machine.clone(),
                rows: total_rows,
            };
            match verify_evidence(d, &own) {
                Ok(e) => Some((d.clone(), e)),
                Err(why) => fail("evidence", format!("{d}: {why}")),
            }
        }
    };
    let peak = evidence.as_ref().map_or(0, |(_, e)| e.peak);
    let evidence_json = evidence.as_ref().map_or("null".to_string(), |(d, e)| {
        format!(
            "{{\"dir\":{},\"peak_rss\":{},\"commit\":{},\"jsonl_sha256\":\"{}\",\"analysis_sha256\":\"{}\",\"manifest_sha256\":\"{}\"}}",
            json_str(d),
            e.peak,
            json_str(&e.commit),
            e.jsonl_sha256,
            e.analysis_sha256,
            e.manifest_sha256
        )
    });

    // the memory preflight
    let (est, parts) = estimate(&sizes);
    let required = est.max(peak);
    let margin = (2 * GIB).max(required / 4);
    let m0 = MemState::read();
    let majflt0 = self_majflt();
    let passed = inject != Inject::Preflight
        && m0.readable()
        && m0.available.is_some_and(|a| a >= required + margin)
        && m0.swap_used() == Some(0)
        && m0.pressure_free();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"preflight\",\"estimate\":{est},\"estimate_parts\":{parts},\"dry_run_evidence\":{evidence_json},\"verified_peak\":{peak},\"required\":{required},\"margin\":{margin},{},\"self_majflt\":{majflt0},\"injected\":{},\"passed\":{passed}}}",
            m0.json(),
            inject == Inject::Preflight
        )],
    );
    if !passed {
        fail(
            "preflight",
            format!(
                "MemAvailable {} against required {required} + margin {margin}; swap used {}; pressure {:?}; sources readable {}{}",
                opt(m0.available),
                opt(m0.swap_used()),
                m0.psi,
                m0.readable(),
                if inject == Inject::Preflight { " (injected)" } else { "" }
            ),
        );
    }
    if verify_only {
        append(
            &out,
            &[format!(
                "{{\"kind\":\"evidence_verified\",\"dry_run_evidence\":{evidence_json},\"required\":{required}}}"
            )],
        );
        println!("cbuild: the dry run's evidence is verified; nothing was built (--verify-evidence-only)");
        return;
    }

    // the inputs and production's output (untimed, once)
    let bs = blocks(&data);
    drop(data);
    let n_blocks = bs.len();
    assert_eq!(
        n_blocks,
        sizes.len(),
        "the sizing pass and the decode agree"
    );
    let mut ll = Vec::with_capacity(n_blocks);
    let mut counts = Vec::with_capacity(n_blocks);
    let mut op = Vec::with_capacity(n_blocks);
    let mut want = Vec::with_capacity(n_blocks);
    for (l, c, o, w) in bs {
        ll.push(l);
        counts.push(c);
        op.push(o);
        want.push(w);
    }
    let rows: Vec<usize> = ll.iter().map(|l| l.len()).collect();

    // the one-pass precondition (G8)
    let mut pre = Vec::with_capacity(n_blocks);
    for b in 0..n_blocks {
        let mut h = [0u32; 256];
        for &x in &ll[b] {
            h[x as usize] += 1;
        }
        let (hist_ok, n_ok) = (h == counts[b], rows[b] < 1 << 24);
        pre.push(format!(
            "{{\"kind\":\"onepass\",\"block\":{b},\"rows\":{},\"histogram_equal\":{hist_ok},\"below_2_24\":{n_ok}}}",
            rows[b]
        ));
        if !(hist_ok && n_ok) {
            append(&out, &pre);
            fail(
                "onepass",
                format!("block {b}: the one-pass precondition fails"),
            );
        }
    }

    // the oracle images (untimed, once), and the untimed counter pass (build::<true>)
    let mut imgs = Vec::with_capacity(n_blocks);
    let mut braw = Vec::with_capacity(n_blocks);
    let mut ctrs = Vec::with_capacity(n_blocks);
    {
        let mut ctor = Ctor::default();
        let (mut rec, mut raw) = (Vec::new(), Vec::new());
        for b in 0..n_blocks {
            let mut o = C30::default();
            o.build(&ll[b], &counts[b], 0);
            ctor.st = Stats::default();
            assert!(ctor.build::<true>(&ll[b], &counts[b], 0, &mut rec, &mut raw));
            assert!(bytes(&rec) == bytes(&o.rec) && bytes(&raw) == bytes(&o.raw));
            let s = &ctor.st;
            ctrs.push(format!(
                "{{\"kind\":\"counters\",\"block\":{b},\"rows\":{},\"rows_complete\":{},\"rows_fragment\":{},\"rows_scalar\":{},\"runs\":{},\"run_rows\":{},\"wide_promotions\":{},\"reconstructed_fields\":{},\"raw_groups\":{},\"raw_moved\":{},\"raw_cycles\":{}}}",
                s.rows,
                s.rows_complete,
                s.rows_fragment,
                s.rows_scalar,
                s.runs,
                s.run_rows,
                s.wide_promotions,
                s.reconstructed_fields,
                s.raw_groups,
                s.raw_moved,
                s.raw_cycles
            ));
            braw.push(o.raw.len());
            imgs.push(Image {
                rec: o.rec,
                raw: o.raw,
            });
        }
    }

    // the untimed poison pass (G7): every block built by `build::<false>` into one arena
    // preallocated for the largest block and filled with poison, so that every record's
    // initialisation is shown, apart from the measured arms (which are not preallocated)
    let mut poisoned = Vec::with_capacity(n_blocks);
    {
        let gmax = rows.iter().map(|n| n.div_ceil(ROWS)).max().unwrap_or(0);
        let mut ctor = Ctor::default();
        let (mut rec, mut raw): (Vec<Rec>, Vec<RawG>) = (Vec::with_capacity(gmax + 1), Vec::new());
        for b in 0..n_blocks {
            poison(&mut rec);
            let (cap, ptr) = (rec.capacity(), rec.as_ptr() as usize);
            let ok = ctor.build::<false>(&ll[b], &counts[b], 0, &mut rec, &mut raw);
            let survived = rec.as_ptr() as usize == ptr && cap >= rec.len();
            let exact =
                ok && bytes(&rec) == bytes(&imgs[b].rec) && bytes(&raw) == bytes(&imgs[b].raw);
            poisoned.push(format!(
                "{{\"kind\":\"poisoncheck\",\"block\":{b},\"rows\":{},\"poisoned_capacity\":{cap},\"needed\":{},\"poison_survived\":{survived},\"exact\":{exact}}}",
                rows[b],
                rec.len()
            ));
            if !(survived && exact) {
                append(&out, &poisoned);
                fail(
                    "output",
                    format!("block {b}: the poison pass: survived {survived}, exact {exact}"),
                );
            }
        }
    }

    let lists = snake(&rows);
    let wrows: Vec<usize> = lists
        .iter()
        .map(|l| l.iter().map(|&b| rows[b]).sum())
        .collect();
    let total: usize = rows.iter().sum();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"meta\",\"corpus\":{name:?},\"blocks\":{n_blocks},\"rows\":{total},\"full_blocks\":{},\"rounds\":{rounds},\"dry_run\":{dry},\"inject\":{},\"cpus\":{cpus:?},\"regimes\":[\"warm\",\"cold\"],\"w4_lists\":{lists:?},\"w4_rows\":{wrows:?},\"block_rows\":{rows:?},\"block_raw_groups\":{braw:?},\"memory_estimate\":{est},\"corpus_sha256\":\"{corpus_sha256}\",\"binary_sha256\":\"{binary_sha256}\",\"machine\":{machine}}}",
            rows.iter().filter(|&&n| n >= FULL).count(),
            if inject == Inject::None { "null".to_string() } else { json_str(&inject_arg) },
        )],
    );
    append(&out, &pre);
    append(&out, &ctrs);
    append(&out, &poisoned);

    let epoch = Instant::now();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"calibration\",\"when\":\"before\",\"cpus\":[{}]}}",
            calibrate(&cpus, CALIBRATION_N, epoch)
        )],
    );
    let refs = Mutex::new(vec![Vec::new(); n_blocks]);
    let check0 = Mutex::new(Vec::new());
    let e = Exp {
        ll: &ll,
        counts: &counts,
        op: &op,
        want: &want,
        imgs: &imgs,
        refs: &refs,
        epoch,
        inject,
        check0: &check0,
    };
    let mut ws: Vec<Worker> = (0..4).map(|_| Worker::default()).collect();
    for round in -1..rounds {
        let arms = if round.rem_euclid(2) == 0 {
            [Arm::P, Arm::C]
        } else {
            [Arm::C, Arm::P]
        };
        for (regime, w4) in [
            (Regime::Warm, false),
            (Regime::Warm, true),
            (Regime::Cold, false),
            (Regime::Cold, true),
        ] {
            for &arm in &arms {
                let recs = if w4 {
                    pass_w4(&mut ws, &lists, round, regime, arm, &cpus, &e)
                } else {
                    pass_w1(&mut ws[0], round, regime, arm, cpus[0], &e)
                };
                append(&out, &recs);
            }
        }
        if round == -1 {
            let mut c = check0.lock().unwrap();
            append(&out, &c);
            c.clear();
        }
        println!(
            "cbuild: round {round} done{}",
            if dry { " (dry run)" } else { "" }
        );
    }
    append(
        &out,
        &[format!(
            "{{\"kind\":\"calibration\",\"when\":\"after\",\"cpus\":[{}]}}",
            calibrate(&cpus, CALIBRATION_N, epoch)
        )],
    );
    let m1 = MemState::read();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"postflight\",\"estimate\":{est},\"required\":{required},{},\"self_majflt\":{},\"pswpin_delta\":{},\"pswpout_delta\":{}}}",
            m1.json(),
            self_majflt(),
            delta(m0.pswpin, m1.pswpin),
            delta(m0.pswpout, m1.pswpout)
        )],
    );
}

/// A counter's increase as JSON, or null when either reading is missing
fn delta(a: Option<u64>, b: Option<u64>) -> String {
    match (a, b) {
        (Some(a), Some(b)) => (b as i64 - a as i64).to_string(),
        _ => "null".into(),
    }
}

/// `cbuild-rss`: one arm's W4 warm pass in a fresh process holding only the last columns
/// and counts; the growth of the peak RSS over the post-load baseline (§5)
pub fn rss(args: &[String]) {
    let out = &args[0];
    let file = &args[1];
    let (mut arm, mut cpus) = (None, Vec::new());
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--arm" => {
                arm = Some(if args[i + 1] == "P" { Arm::P } else { Arm::C });
                i += 1;
            }
            "--cpus" => {
                cpus = arg_cpus(&args[i + 1]);
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    let arm = arm.expect("--arm P or C");
    assert!(cpus.len() == 4);
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let data = std::fs::read(file).unwrap();
    let corpus_sha256 = sha256(&data);
    let cols = last_columns(&data, true);
    drop(data);
    let rows: Vec<usize> = cols.iter().map(|(l, _)| l.len()).collect();
    let lists = snake(&rows);
    let base_rss = proc_kv("/proc/self/status", "VmRSS");
    let hwm_load = proc_kv("/proc/self/status", "VmHWM");
    let reset = std::fs::write("/proc/self/clear_refs", "5").is_ok();
    let hwm_reset = proc_kv("/proc/self/status", "VmHWM");
    let start = SpinBarrier::new(4);
    let done = std::sync::Barrier::new(4);
    let steps = lists.iter().map(|l| l.len()).max().unwrap();
    let mut ws: Vec<Arenas> = (0..4).map(|_| Arenas::default()).collect();
    std::thread::scope(|sc| {
        for (k, a) in ws.iter_mut().enumerate() {
            let (list, start, done, cols, cpus) = (&lists[k], &start, &done, &cols, &cpus);
            sc.spawn(move || {
                pin_or_fail(cpus[k], &format!("rss worker {k}"));
                for step in 0..steps {
                    start.wait();
                    if let Some(&b) = list.get(step) {
                        let (ll, counts) = (&cols[b].0, &cols[b].1);
                        match arm {
                            Arm::P => p_build(ll, counts, &mut a.tt),
                            Arm::C => {
                                assert!(c_build(&mut a.ctor, ll, counts, &mut a.rec, &mut a.raw))
                            }
                        }
                    }
                    done.wait();
                }
            });
        }
    });
    let hwm = proc_kv("/proc/self/status", "VmHWM");
    let caps: u64 = ws.iter().map(|a| a.caps(arm).0).sum();
    let growth = match (hwm, base_rss) {
        (Some(h), Some(b)) => (h as i64 - b as i64).to_string(),
        _ => "null".into(),
    };
    std::fs::write(
        out,
        format!(
            "{{\"kind\":\"rss\",\"arm\":\"{}\",\"corpus\":{:?},\"corpus_sha256\":\"{corpus_sha256}\",\"blocks\":{},\"cpus\":{cpus:?},\"pid\":{},\"started_unix_ns\":{started},\"baseline_rss\":{},\"hwm_after_load\":{},\"hwm_reset\":{reset},\"hwm_after_reset\":{},\"hwm_after_pass\":{},\"growth_over_baseline\":{growth},\"arena_capacity_bytes\":{caps},\"capacity_scope\":\"{}\"}}\n",
            arm_name(arm),
            std::path::Path::new(file).file_name().unwrap().to_string_lossy(),
            rows.len(),
            std::process::id(),
            opt(base_rss),
            opt(hwm_load),
            opt(hwm_reset),
            opt(hwm),
            if arm == Arm::P { "tt" } else { "rec+raw" }
        ),
    )
    .unwrap();
    println!("cbuild-rss: arm {} done", arm_name(arm));
}
