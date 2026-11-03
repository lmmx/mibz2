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
//!   - C's arena is poisoned before its build;
//!   - P's table is checked against the counting-scatter definition, then becomes the
//!     block's reference;
//!   - both arms' tables are walked by their unchanged walkers against production's output,
//!     and checked again after the walk.
//!
//! **Reading production's table:** production's `TVector` is a newtype over `Vec<u32>`
//! with no public accessor, and `src/` may not change. The harness reads it through a
//! layout reinterpretation (`tv`), guarded at compile time (size and alignment) and at
//! every read (the length). That relies on the newtype's layout, which Rust doesn't
//! guarantee; the write-up records it. It is used only outside every timed window.

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

const _: () = {
    assert!(std::mem::size_of::<TVector>() == std::mem::size_of::<Vec<u32>>());
    assert!(std::mem::align_of::<TVector>() == std::mem::align_of::<Vec<u32>>());
};

/// Production's table as a slice (harness only, untimed; see the module comment)
fn tv(t: &TVector) -> &Vec<u32> {
    // SAFETY: TVector is `struct TVector(pub(crate) Vec<u32>)`, a single-field newtype of the
    // same size and alignment (checked above); its length is checked against the public
    // `len()` on every read
    let v = unsafe { &*(t as *const TVector as *const Vec<u32>) };
    assert_eq!(
        v.len(),
        t.len(),
        "the T-vector reinterpretation disagrees with len()"
    );
    v
}

/// The same, mutable: used only by a dry run's injected fault
fn tv_mut(t: &mut TVector) -> &mut Vec<u32> {
    let n = t.len();
    // SAFETY: as `tv`
    let v = unsafe { &mut *(t as *mut TVector as *mut Vec<u32>) };
    assert_eq!(v.len(), n);
    v
}

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

/// /proc/pressure/memory's (some avg10, full avg10), if the file exists
fn psi() -> Option<(f64, f64)> {
    let s = std::fs::read_to_string("/proc/pressure/memory").ok()?;
    let get = |kind: &str| -> Option<f64> {
        let l = s.lines().find(|l| l.starts_with(kind))?;
        let f = l.split_whitespace().find(|f| f.starts_with("avg10="))?;
        f["avg10=".len()..].parse().ok()
    };
    Some((get("some")?, get("full")?))
}

/// The memory state, as a JSON fragment, and whether it shows swap in use or pressure
struct MemState {
    available: u64,
    swap_total: u64,
    swap_free: u64,
    psi: Option<(f64, f64)>,
    pswpin: u64,
    pswpout: u64,
    rss: u64,
    hwm: u64,
}

impl MemState {
    fn read() -> MemState {
        MemState {
            available: proc_kv("/proc/meminfo", "MemAvailable").unwrap_or(0),
            swap_total: proc_kv("/proc/meminfo", "SwapTotal").unwrap_or(0),
            swap_free: proc_kv("/proc/meminfo", "SwapFree").unwrap_or(0),
            psi: psi(),
            pswpin: proc_kv("/proc/vmstat", "pswpin").unwrap_or(0),
            pswpout: proc_kv("/proc/vmstat", "pswpout").unwrap_or(0),
            rss: proc_kv("/proc/self/status", "VmRSS").unwrap_or(0),
            hwm: proc_kv("/proc/self/status", "VmHWM").unwrap_or(0),
        }
    }
    fn swap_used(&self) -> u64 {
        self.swap_total.saturating_sub(self.swap_free)
    }
    fn pressure_free(&self) -> bool {
        self.psi.is_none_or(|(s, f)| s == 0.0 && f == 0.0)
    }
    fn json(&self) -> String {
        format!(
            "\"mem_available\":{},\"swap_total\":{},\"swap_free\":{},\"swap_used\":{},\"psi_present\":{},\"psi_some_avg10\":{},\"psi_full_avg10\":{},\"pswpin\":{},\"pswpout\":{},\"vm_rss\":{},\"vm_hwm\":{}",
            self.available,
            self.swap_total,
            self.swap_free,
            self.swap_used(),
            self.psi.is_some(),
            self.psi.map_or("null".into(), |p| p.0.to_string()),
            self.psi.map_or("null".into(), |p| p.1.to_string()),
            self.pswpin,
            self.pswpout,
            self.rss,
            self.hwm
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

fn usable<T>(v: &Vec<T>) -> u64 {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        extern "C" {
            fn malloc_usable_size(p: *mut std::ffi::c_void) -> usize;
        }
        if v.capacity() == 0 {
            return 0;
        }
        // SAFETY: a live allocation of the system allocator
        unsafe { malloc_usable_size(v.as_ptr() as *mut std::ffi::c_void) as u64 }
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        let _ = v;
        0
    }
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
    /// (capacity bytes, allocator usable bytes) of the arm's table
    fn caps(&self, arm: Arm) -> (u64, u64) {
        match arm {
            Arm::P => {
                let v = tv(&self.tt);
                (4 * v.capacity() as u64, usable(v))
            }
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
                    Arm::P => tv_mut(&mut self.warm.tt)[0] ^= 1 << 8,
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
                    e.check0.lock().unwrap().push(format!(
                        "{{\"kind\":\"check0\",\"arm\":\"C\",\"block\":{b},\"poisoned\":true,\"exact\":true,\"walk\":true,\"restored\":true}}"
                    ));
                }
            }
            Arm::P => {
                if g7 {
                    // the counting-scatter definition, independently of production's code
                    let t = tv(&a.tt);
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
                        def &= t.get(sl) == Some(&(ll[sl] as u32 | (i as u32) << 8));
                    }
                    let reference = t.clone();
                    walk_segments(&mut a.tt, e.op[b], &mut self.out, &mut self.seg);
                    let walk_ok = self.out == e.want[b];
                    let restored = *tv(&a.tt) == reference;
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
                    if refs[b].len() != a.tt.len() || *tv(&a.tt) != refs[b] {
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
        "{{\"kind\":\"build\",\"round\":{},\"regime\":\"{}\",\"mode\":\"{}\",\"arm\":\"{}\",\"worker\":{},\"step\":{},\"block\":{},\"rows\":{rows},\"cpu_before\":{},\"cpu_after\":{},\"prep_start\":{},\"prep_end\":{},\"start\":{},\"end\":{},\"cpu_start\":{},\"cpu_end\":{},\"minflt\":{},\"majflt\":{},\"check_start\":{},\"check_end\":{},\"cap_bytes\":{},\"usable_bytes\":{},\"exact\":true}}",
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
        caps.1
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

/// `cbuild`: the registered schedule (§3), or its dry run
pub fn main(args: &[String]) {
    let out = args[0].clone();
    let file = &args[1];
    let (mut rounds, mut cpus, mut dry) = (20i64, Vec::new(), false);
    let (mut inject, mut inject_arg, mut peak_rss) = (Inject::None, String::new(), 0u64);
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
            "--peak-rss" => {
                peak_rss = args[i + 1].parse().unwrap();
                i += 1;
            }
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

    // the sizing pass and the memory preflight, before any large allocation
    let sizes: Vec<(usize, [u32; 256])> = last_columns(&data, false)
        .into_iter()
        .map(|(_, c)| (c.iter().map(|&x| x as usize).sum(), c))
        .collect();
    let (est, parts) = estimate(&sizes);
    let required = est.max(peak_rss);
    let margin = (2 * GIB).max(required / 4);
    let m0 = MemState::read();
    let majflt0 = self_majflt();
    let passed = inject != Inject::Preflight
        && m0.available >= required + margin
        && m0.swap_used() == 0
        && m0.pressure_free();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"preflight\",\"estimate\":{est},\"estimate_parts\":{parts},\"peak_rss_arg\":{peak_rss},\"required\":{required},\"margin\":{margin},{},\"self_majflt\":{majflt0},\"injected\":{},\"passed\":{passed}}}",
            m0.json(),
            inject == Inject::Preflight
        )],
    );
    if !passed {
        fail(
            "preflight",
            format!(
                "MemAvailable {} against required {required} + margin {margin}; swap used {}; pressure {:?}{}",
                m0.available,
                m0.swap_used(),
                m0.psi,
                if inject == Inject::Preflight { " (injected)" } else { "" }
            ),
        );
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

    let lists = snake(&rows);
    let wrows: Vec<usize> = lists
        .iter()
        .map(|l| l.iter().map(|&b| rows[b]).sum())
        .collect();
    let total: usize = rows.iter().sum();
    append(
        &out,
        &[format!(
            "{{\"kind\":\"meta\",\"corpus\":{name:?},\"blocks\":{n_blocks},\"rows\":{total},\"full_blocks\":{},\"rounds\":{rounds},\"dry_run\":{dry},\"inject\":{},\"cpus\":{cpus:?},\"regimes\":[\"warm\",\"cold\"],\"w4_lists\":{lists:?},\"w4_rows\":{wrows:?},\"block_rows\":{rows:?},\"block_raw_groups\":{braw:?},\"memory_estimate\":{est}}}",
            rows.iter().filter(|&&n| n >= FULL).count(),
            if inject == Inject::None { "null".to_string() } else { json_str(&inject_arg) },
        )],
    );
    append(&out, &pre);
    append(&out, &ctrs);

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
            m1.pswpin as i64 - m0.pswpin as i64,
            m1.pswpout as i64 - m0.pswpout as i64
        )],
    );
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
    let data = std::fs::read(file).unwrap();
    let cols = last_columns(&data, true);
    drop(data);
    let rows: Vec<usize> = cols.iter().map(|(l, _)| l.len()).collect();
    let lists = snake(&rows);
    let base_rss = proc_kv("/proc/self/status", "VmRSS").unwrap_or(0);
    let hwm_load = proc_kv("/proc/self/status", "VmHWM").unwrap_or(0);
    let reset = std::fs::write("/proc/self/clear_refs", "5").is_ok();
    let hwm_reset = proc_kv("/proc/self/status", "VmHWM").unwrap_or(0);
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
    let hwm = proc_kv("/proc/self/status", "VmHWM").unwrap_or(0);
    let caps: u64 = ws.iter().map(|a| a.caps(arm).0).sum();
    std::fs::write(
        out,
        format!(
            "{{\"kind\":\"rss\",\"arm\":\"{}\",\"corpus\":{:?},\"blocks\":{},\"cpus\":{cpus:?},\"baseline_rss\":{base_rss},\"hwm_after_load\":{hwm_load},\"hwm_reset\":{reset},\"hwm_after_reset\":{hwm_reset},\"hwm_after_pass\":{hwm},\"growth_over_baseline\":{},\"arena_capacity_bytes\":{caps}}}\n",
            arm_name(arm),
            std::path::Path::new(file).file_name().unwrap().to_string_lossy(),
            rows.len(),
            hwm as i64 - base_rss as i64
        ),
    )
    .unwrap();
    println!("cbuild-rss: arm {} done", arm_name(arm));
}
