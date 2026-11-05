//! The pair matrix: one decode stage measured on one CPU while a chosen load runs without
//! pause beside it, on the stage's SMT sibling (the same core) or on a CPU of another core,
//! each condition paired by round against the stage alone. A diagnostic, from the Actions
//! tab only (sweep.yml, `tool: pair`): which kind of sibling slows the walk, and whether a
//! walk beside a compute stage costs the core less than a walk beside a walk. Its figures
//! are the run's summary and artifact, never recorded; the decoder's A/B decides changes.
//!
//! ```text
//! pair FILE.bz2 [ROUNDS] [GHZ] [--json OUT.json] [--blocks N] [--cpus M,SIBLING,CROSS]
//! ```
//!
//! **The CPUs.** Those this process may run on, grouped by physical core: by sysfs's
//! `thread_siblings_list`, checked against each CPU's `core_id` and `physical_package_id`
//! (by those ids where a siblings list can't be read). The measured stage runs on the first
//! CPU of the first core with two CPUs (M); the sibling conditions' load on that core's
//! next CPU; the cross-core conditions' load on the first CPU of another core; this
//! process's main thread, which only waits, on a CPU none of them uses, where there is one.
//! With no SMT (every core one CPU, as on many VMs) the sibling conditions are refused, with
//! a message saying so, and the cross-core and alone conditions run. `--cpus` names the
//! three CPUs instead (`-` for none), checked against the topology.
//!
//! **The measured stages**, on M: a decode of the file's blocks as `decode_block` runs one
//! with the default kernel (`Segmented`): MTF + RLE2 (`mtf::mtf`), the T-vector build
//! (`bwt::build`), the room walk (`bwt::walk_segments_room`, the entry point `Segmented`,
//! the sweep's default setting and phases' "as one" row use, its output over `ll` as
//! `invert_in_place` writes it), RLE1 and the CRC (checked), only the measured stage timed.
//! So each stage meets the caches as a decode leaves them, as in examples/sweep.rs and
//! examples/phases.rs. A stage's figure is its time summed over the blocks: cycles a byte of
//! the walk's output at the reported clock (the sweep's c/B), and ns a byte.
//!
//! **The loads**, on the load's CPU (S), started with the measured thread behind a barrier
//! (the measured thread then sleeps 2 ms before its first block, so the load is already
//! running) and run without pause until the measured decode ends, then one unit more:
//! - `walk`: the room walk (`walk_segments_room`) of the load's own block (the largest of
//!   the blocks measured, in its own table), again and again;
//! - `chain`: the plain walk's one dependent chain (one line in flight: `bwt::walk`'s step)
//!   over the same block's table, built here in the library's layout and its bytes checked
//!   against `bwt::walk`'s, 4096 steps a unit so its progress is seen finely;
//! - `alu`: an L1-resident, uop-heavy integer loop with no memory (x86_64: 12 independent
//!   adds and xors an iteration over six registers, more than the core's ALUs issue);
//! - `mtf`: MTF + RLE2 of the load block, again and again;
//! - `build`: the T-vector build of the load block, again and again;
//! - `stream`: a sequential read of a 64 MB buffer, 1 MiB a unit (bandwidth; the hardware
//!   prefetcher's few misses in flight per byte).
//!
//! **The load's own throughput** while the stage is timed: the load logs the time and its
//! work done at each unit's end; each unit is split into its time inside the measured
//! stage's windows and outside them, and the load's rate inside is fitted by least squares
//! (a unit's work = rate inside x time inside + rate outside x time outside). That undoes
//! the smearing of units as long as a window (the walk, MTF and build loads take a block a
//! unit); the rate interpolated within each unit is kept beside it. Against the load alone
//! on S (M idle, a 0.3 s window), paired by round, it is the load's speed; the pair's yield
//! is the stage's speed (its time alone over its time) plus the load's: 2 when neither slows
//! the other, 1 when the core only takes turns.
//!
//! **Rounds.** Every condition once a round, their order rotated a step each round (as the
//! sweep's settings), so each condition is paired by round with its stage alone: the
//! geometric mean of the rounds' ratios, with its 95% interval (a t-interval on the logs, as
//! the sweep's). At least 5 rounds for a reading; fewer are a dry run, and say so.
//!
//! **What it separates.** If the walk's SMT penalty is core-side (issue, or the partitioned
//! queues), any busy sibling costs it: the ALU loop about as much as a walk. If it is
//! memory-side (miss handling, lines in flight), only a memory-heavy sibling costs: the
//! walk and the stream, not the ALU loop, and the one-line chain little. Cross-core
//! conditions bound what the shared L3 and memory add. The pairing table asks whether a
//! co-run-aware placement could pay: the core's time for both siblings' walk and MTF (or
//! build), one sibling walking while the other runs the other stage, against each beside
//! its own kind; an ideal, with the four sibling ratios fixed and the stages free to be
//! reordered.

use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Barrier;
use std::time::{Duration, Instant};

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

/// The measured thread's sleep after the barrier, before its first block: the load is
/// running by then
const RAMP: Duration = Duration::from_millis(2);
/// A load alone: its window
const LOAD_ALONE: Duration = Duration::from_millis(300);
/// The chain load's steps a unit
const CHAIN_STEPS: usize = 4096;
/// The ALU load's iterations a unit
const ALU_ITERS: u64 = 1 << 16;
/// The stream load's buffer, and its bytes a unit
const STREAM_BYTES: usize = 64 << 20;
const STREAM_CHUNK: usize = 1 << 20;
/// A load unit this many times its median is flagged (the load was likely preempted)
const LONG_UNIT: f64 = 5.0;

// Two-sided 95% t quantiles for 1 to 30 degrees of freedom
const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

// ---- The conditions ----

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kernel {
    Walk,
    Mtf,
    Build,
}

const KERNELS: [Kernel; 3] = [Kernel::Walk, Kernel::Mtf, Kernel::Build];

impl Kernel {
    fn name(self) -> &'static str {
        match self {
            Kernel::Walk => "walk",
            Kernel::Mtf => "mtf",
            Kernel::Build => "build",
        }
    }
    fn title(self) -> &'static str {
        match self {
            Kernel::Walk => "the room walk (bwt::walk_segments_room, in place)",
            Kernel::Mtf => "MTF + RLE2 (mtf::mtf)",
            Kernel::Build => "the T-vector build (bwt::build)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Load {
    Walk,
    Chain,
    Alu,
    Mtf,
    Build,
    Stream,
}

const LOADS: [Load; 6] = [
    Load::Walk,
    Load::Chain,
    Load::Alu,
    Load::Mtf,
    Load::Build,
    Load::Stream,
];

impl Load {
    fn name(self) -> &'static str {
        match self {
            Load::Walk => "walk",
            Load::Chain => "chain",
            Load::Alu => "alu",
            Load::Mtf => "mtf",
            Load::Build => "build",
            Load::Stream => "stream",
        }
    }
    fn about(self) -> &'static str {
        match self {
            Load::Walk => "the room walk of the load block, again and again",
            Load::Chain => "the plain walk's one chain (1 line in flight), 4096 steps a unit",
            Load::Alu => "an L1-resident integer loop, 12 independent ALU ops an iteration",
            Load::Mtf => "MTF + RLE2 of the load block, again and again",
            Load::Build => "the T-vector build of the load block, again and again",
            Load::Stream => "a sequential read of a 64 MB buffer, 1 MiB a unit",
        }
    }
    /// What its work counts: bytes (of the block, or read), or iterations
    fn unit(self) -> &'static str {
        match self {
            Load::Alu => "iteration",
            _ => "byte",
        }
    }
    /// A rate (work a ns) as the table shows it: ns a byte, or ns a thousand iterations
    fn show_rate(self, rate: f64) -> String {
        if !rate.is_finite() {
            return "n/a".into();
        }
        match self {
            Load::Alu => format!("{:.1} ns/kit", 1e3 / rate),
            _ => format!("{:.3} ns/B", 1.0 / rate),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Place {
    Alone,
    Sibling,
    Cross,
}

impl Place {
    fn name(self) -> &'static str {
        match self {
            Place::Alone => "alone",
            Place::Sibling => "sibling",
            Place::Cross => "cross",
        }
    }
}

/// A measured stage beside a load (both: on M and on S), a stage alone (no load), or a
/// load alone (no stage: on S with M idle)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Condition {
    kernel: Option<Kernel>,
    load: Option<Load>,
    place: Place,
}

impl Condition {
    fn id(&self) -> String {
        format!(
            "{}|{}|{}",
            self.kernel.map_or("-", Kernel::name),
            self.load.map_or("none", Load::name),
            self.place.name()
        )
    }
}

/// Every condition a round: each stage alone, then beside each load on the sibling and on
/// the other core (where each exists); then each load alone
fn conditions(sibling: bool, cross: bool) -> Vec<Condition> {
    let mut v = Vec::new();
    for k in KERNELS {
        v.push(Condition {
            kernel: Some(k),
            load: None,
            place: Place::Alone,
        });
        for l in LOADS {
            for (on, place) in [(sibling, Place::Sibling), (cross, Place::Cross)] {
                if on {
                    v.push(Condition {
                        kernel: Some(k),
                        load: Some(l),
                        place,
                    });
                }
            }
        }
    }
    for l in LOADS {
        v.push(Condition {
            kernel: None,
            load: Some(l),
            place: Place::Alone,
        });
    }
    v
}

/// Round r's order of the conditions: the list rotated r steps, as the sweep's settings
fn order(r: usize, len: usize) -> Vec<usize> {
    (0..len).map(|j| (j + r) % len).collect()
}

fn find(
    conds: &[Condition],
    kernel: Option<Kernel>,
    load: Option<Load>,
    place: Place,
) -> Option<usize> {
    conds
        .iter()
        .position(|c| c.kernel == kernel && c.load == load && c.place == place)
}

// ---- The CPUs ----

/// A CPU list ("0-3,6"), as sysfs and /proc write them
fn cpu_list(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in s.trim().split(',').filter(|p| !p.is_empty()) {
        match part.split_once('-') {
            Some((a, b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<usize>(), b.parse::<usize>()) {
                    out.extend(a..=b);
                }
            }
            None => out.extend(part.parse::<usize>().ok()),
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
struct Topology {
    /// The allowed CPUs grouped by physical core, each core's CPUs in order, the cores in
    /// order of their first CPU
    cores: Vec<Vec<usize>>,
    /// What grouped them: "thread_siblings_list", "core_id" (the siblings unreadable) or
    /// "none" (each CPU a core of its own)
    source: &'static str,
    /// Whether every core's CPUs share one (physical_package_id, core_id) and no two cores
    /// do; None where an id can't be read or the ids grouped them
    ids_agree: Option<bool>,
}

impl Topology {
    fn smt(&self) -> bool {
        self.cores.iter().any(|c| c.len() >= 2)
    }
    fn show(&self) -> String {
        self.cores
            .iter()
            .map(|c| {
                c.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// The allowed CPUs grouped by core: by each CPU's siblings where every CPU's can be read,
/// else by (package, core) ids where every CPU's can, else each CPU alone
fn group(
    allowed: &[usize],
    siblings: impl Fn(usize) -> Option<Vec<usize>>,
    ids: impl Fn(usize) -> Option<(u32, u32)>,
) -> Topology {
    let sib: Option<Vec<Vec<usize>>> = allowed.iter().map(|&c| siblings(c)).collect();
    let id: Option<Vec<(u32, u32)>> = allowed.iter().map(|&c| ids(c)).collect();
    let mut cores: Vec<Vec<usize>> = Vec::new();
    let source = match (&sib, &id) {
        (Some(sib), _) => {
            for (&c, s) in allowed.iter().zip(sib) {
                let mut core: Vec<usize> =
                    s.iter().copied().filter(|x| allowed.contains(x)).collect();
                if !core.contains(&c) {
                    core = vec![c];
                }
                core.sort();
                if !cores.contains(&core) {
                    cores.push(core);
                }
            }
            "thread_siblings_list"
        }
        (None, Some(id)) => {
            let mut keys: Vec<(u32, u32)> = Vec::new();
            for (&c, &k) in allowed.iter().zip(id) {
                match keys.iter().position(|&x| x == k) {
                    Some(i) => cores[i].push(c),
                    None => {
                        keys.push(k);
                        cores.push(vec![c]);
                    }
                }
            }
            cores.iter_mut().for_each(|c| c.sort());
            "core_id"
        }
        (None, None) => {
            cores = allowed.iter().map(|&c| vec![c]).collect();
            "none"
        }
    };
    cores.sort();
    let ids_agree = match (source, &id) {
        ("thread_siblings_list", Some(id)) => {
            let key = |c: usize| id[allowed.iter().position(|&x| x == c).unwrap()];
            let firsts: Vec<(u32, u32)> = cores.iter().map(|c| key(c[0])).collect();
            let within = cores.iter().all(|c| c.iter().all(|&x| key(x) == key(c[0])));
            let distinct = firsts
                .iter()
                .enumerate()
                .all(|(i, k)| !firsts[..i].contains(k));
            Some(within && distinct)
        }
        _ => None,
    };
    Topology {
        cores,
        source,
        ids_agree,
    }
}

/// This process's CPUs and their topology, from /proc and sysfs
fn read_topology() -> Topology {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let allowed = status
        .lines()
        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
        .map(cpu_list)
        .unwrap_or_else(|| {
            (0..std::thread::available_parallelism().map_or(1, |n| n.get())).collect()
        });
    let topo = |c: usize, f: &str| {
        std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{c}/topology/{f}")).ok()
    };
    group(
        &allowed,
        |c| topo(c, "thread_siblings_list").map(|s| cpu_list(&s)),
        |c| {
            let p = topo(c, "physical_package_id")?.trim().parse().ok()?;
            let k = topo(c, "core_id")?.trim().parse().ok()?;
            Some((p, k))
        },
    )
}

/// The CPUs a run uses: the measured stage's, its SMT sibling's and a CPU of another core's
/// (each where it exists), and a CPU none of them uses for the main thread
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cpus {
    measured: usize,
    sibling: Option<usize>,
    cross: Option<usize>,
    spare: Option<usize>,
}

impl Cpus {
    /// Where a load runs alone: the sibling's CPU, else the cross-core one
    fn alone(&self) -> usize {
        self.sibling.or(self.cross).unwrap()
    }
    fn of(&self, place: Place) -> Option<usize> {
        match place {
            Place::Sibling => self.sibling,
            Place::Cross => self.cross,
            Place::Alone => Some(self.alone()),
        }
    }
}

fn spare(cores: &[Vec<usize>], used: [Option<usize>; 3]) -> Option<usize> {
    cores
        .iter()
        .flatten()
        .copied()
        .find(|&c| !used.contains(&Some(c)))
}

/// The default choice: M the first CPU of the first core with two CPUs (else of the first
/// core), its sibling that core's next CPU, the cross-core CPU the first of the first other
/// core
fn choose(cores: &[Vec<usize>]) -> Result<Cpus, String> {
    let all: usize = cores.iter().map(Vec::len).sum();
    if all < 2 {
        return Err(format!(
            "{all} CPU allowed: a load needs a CPU of its own beside the measured stage's"
        ));
    }
    let home = cores.iter().position(|c| c.len() >= 2).unwrap_or(0);
    let measured = cores[home][0];
    let sibling = cores[home].get(1).copied();
    let cross = (0..cores.len()).find(|&i| i != home).map(|i| cores[i][0]);
    Ok(Cpus {
        measured,
        sibling,
        cross,
        spare: spare(cores, [Some(measured), sibling, cross]),
    })
}

/// `--cpus M,SIBLING,CROSS` (`-` for none), checked: M allowed, SIBLING on M's core, CROSS
/// on another, at least one of the two
fn parse_cpus(s: &str, cores: &[Vec<usize>]) -> Result<Cpus, String> {
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err(format!("--cpus {s}: M,SIBLING,CROSS (- for none)"));
    }
    let cpu = |x: &str| -> Result<Option<usize>, String> {
        if x == "-" {
            Ok(None)
        } else {
            x.parse()
                .map(Some)
                .map_err(|_| format!("--cpus: {x:?} is not a CPU"))
        }
    };
    let core_of = |c: usize| cores.iter().position(|k| k.contains(&c));
    let measured = cpu(parts[0])?.ok_or("--cpus: M is required")?;
    let home = core_of(measured).ok_or(format!("--cpus: CPU {measured} is not allowed here"))?;
    let sibling = cpu(parts[1])?;
    if let Some(x) = sibling {
        if x == measured || core_of(x) != Some(home) {
            return Err(format!(
                "--cpus: CPU {x} is not CPU {measured}'s SMT sibling"
            ));
        }
    }
    let cross = cpu(parts[2])?;
    if let Some(x) = cross {
        match core_of(x) {
            None => return Err(format!("--cpus: CPU {x} is not allowed here")),
            Some(c) if c == home => {
                return Err(format!("--cpus: CPU {x} is on CPU {measured}'s core"))
            }
            _ => {}
        }
    }
    if sibling.is_none() && cross.is_none() {
        return Err("--cpus: no CPU for a load".into());
    }
    Ok(Cpus {
        measured,
        sibling,
        cross,
        spare: spare(cores, [Some(measured), sibling, cross]),
    })
}

extern "C" {
    fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
    fn sched_getcpu() -> i32;
}

/// Pin the calling thread to `cpu`, and check that it runs there
fn pin(cpu: usize) {
    assert!(cpu < 1024, "CPU {cpu}: beyond a 1024-bit CPU set");
    let mut mask = [0u64; 16];
    mask[cpu / 64] |= 1 << (cpu % 64);
    // SAFETY: a cpu_set_t of 1024 bits, read by the kernel; pid 0 is the calling thread
    let ok = unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) } == 0;
    assert!(ok, "could not pin a thread to CPU {cpu}");
    assert_eq!(
        on_cpu(),
        cpu as i32,
        "pinned to CPU {cpu}, running elsewhere"
    );
}

fn on_cpu() -> i32 {
    // SAFETY: no arguments; glibc's
    unsafe { sched_getcpu() }
}

/// The calling thread's involuntary context switches so far (0 where unreadable)
fn preempted() -> u64 {
    std::fs::read_to_string("/proc/thread-self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("nonvoluntary_ctxt_switches:"))
                .and_then(|v| v.trim().parse().ok())
        })
        .unwrap_or(0)
}

// ---- The measured decode ----

fn ns(epoch: Instant) -> u64 {
    epoch.elapsed().as_nanos() as u64
}

/// `f`, its window (start, end; ns since `epoch`) pushed onto `windows` if `on`
fn timed<T>(on: bool, epoch: Instant, windows: &mut Vec<(u64, u64)>, f: impl FnOnce() -> T) -> T {
    if !on {
        return f();
    }
    let start = ns(epoch);
    let v = f();
    windows.push((start, ns(epoch)));
    v
}

/// The measured thread's buffers, kept from block to block and condition to condition
struct Decode {
    ll: Vec<u8>,
    counts: [u32; 256],
    tt: TVector,
    seg: SegScratch,
    out: Vec<u8>,
}

impl Decode {
    fn new() -> Self {
        Decode {
            ll: Vec::new(),
            counts: [0; 256],
            tt: TVector::new(),
            seg: SegScratch::default(),
            out: Vec::new(),
        }
    }
}

/// One decode of `blocks` as `decode_block` runs it with `Segmented` (the walk in place,
/// over `ll`), `kernel`'s stage timed, each block's CRC checked; returns the bytes walked
fn decode(
    blocks: &[Symbols],
    k: Kernels,
    kernel: Kernel,
    d: &mut Decode,
    epoch: Instant,
    windows: &mut Vec<(u64, u64)>,
) -> usize {
    let mut n = 0;
    for b in blocks {
        let len = timed(kernel == Kernel::Mtf, epoch, windows, || {
            mtf::mtf(b, &mut d.ll, &mut d.counts, k).unwrap()
        });
        assert!((b.orig_ptr as usize) < len, "a block's orig_ptr");
        timed(kernel == Kernel::Build, epoch, windows, || {
            build(&d.ll, &d.counts, &mut d.tt)
        });
        timed(kernel == Kernel::Walk, epoch, windows, || {
            walk_segments_room(&mut d.tt, b.orig_ptr, &mut d.ll, &mut d.seg)
        });
        d.out.clear();
        rle::unrle(&d.ll, &mut d.out, k).unwrap();
        assert_eq!(crc::crc32(&d.out, k), b.crc, "a block's CRC");
        n += len;
    }
    n
}

// ---- The loads ----

/// The T-vector of `ll` (`counts` its bytes' counts) in `bwt::build`'s layout (slot s holds
/// ll[s] | i << 8, i the row whose byte took slot s), built plainly here for the chain
fn table(ll: &[u8], counts: &[u32; 256]) -> Vec<u32> {
    let mut ftab = [0u32; 256];
    let mut cum = 0u32;
    for (f, &c) in ftab.iter_mut().zip(counts) {
        *f = cum;
        cum += c;
    }
    let mut t = vec![0u32; ll.len()];
    for (i, &c) in ll.iter().enumerate() {
        let s = ftab[c as usize] as usize;
        t[s] = ll[s] as u32 | (i as u32) << 8;
        ftab[c as usize] += 1;
    }
    t
}

/// `steps` steps of `bwt::walk`'s chain from entry `p`, each load's row the last entry's
/// link: one line in flight; returns the entry reached (the chain is the block's one cycle,
/// so it can be stepped on forever)
#[inline(never)]
fn chain(t: &[u32], mut p: u32, steps: usize) -> u32 {
    for _ in 0..steps {
        p = t[(p >> 8) as usize];
    }
    p
}

/// `iters` iterations of six registers' independent chains of adds and xors: 12 ALU ops an
/// iteration, two on each chain, so the loop asks for 6 ops a cycle; no memory
#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn alu(iters: u64) {
    assert!(iters > 0);
    // SAFETY: registers only (no memory, no stack); the count is at least 1, so `dec`
    // reaches 0
    unsafe {
        std::arch::asm!(
            "2:",
            "add {a}, {k}",
            "xor {b}, {k}",
            "add {c}, {k}",
            "xor {d}, {k}",
            "add {e}, {k}",
            "xor {f}, {k}",
            "xor {a}, {k}",
            "add {b}, {k}",
            "xor {c}, {k}",
            "add {d}, {k}",
            "xor {e}, {k}",
            "add {f}, {k}",
            "dec {n}",
            "jnz 2b",
            n = inout(reg) iters => _,
            k = in(reg) 0x9e37_79b9_7f4a_7c15u64,
            a = inout(reg) 1u64 => _,
            b = inout(reg) 2u64 => _,
            c = inout(reg) 3u64 => _,
            d = inout(reg) 4u64 => _,
            e = inout(reg) 5u64 => _,
            f = inout(reg) 6u64 => _,
            options(nomem, nostack),
        );
    }
}

/// The same in Rust where there is no x86_64 form (its registers spilled each iteration)
#[cfg(not(target_arch = "x86_64"))]
#[inline(never)]
fn alu(iters: u64) {
    const K: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut r = [1u64, 2, 3, 4, 5, 6];
    for _ in 0..iters {
        for x in &mut r {
            *x = x.wrapping_add(K) ^ K;
        }
        r = black_box(r);
    }
}

/// A sequential read of `words`, summed in 8 lanes
#[inline(never)]
fn sum(words: &[u64]) -> u64 {
    let mut acc = [0u64; 8];
    for c in words.chunks_exact(8) {
        for (a, &w) in acc.iter_mut().zip(c) {
            *a = a.wrapping_add(w);
        }
    }
    acc.iter().fold(0, |x, &y| x ^ y)
}

/// The loads' inputs and buffers, made once
struct Loads<'a> {
    k: Kernels,
    block: &'a Symbols,
    n: usize,
    /// the load block's table, for the room walk, and the same in this file's copy, with
    /// the chain's place in it
    tt: TVector,
    chain: Vec<u32>,
    p: u32,
    seg: SegScratch,
    out: Vec<u8>,
    /// MTF's output; the build's input (MTF's, kept) and output
    ll: Vec<u8>,
    counts: [u32; 256],
    bll: Vec<u8>,
    bcounts: [u32; 256],
    btt: TVector,
    stream: Vec<u64>,
    at: usize,
}

impl<'a> Loads<'a> {
    /// The loads over `block`, each checked: the room walk's and the chain's bytes against
    /// the plain walk's
    fn new(block: &'a Symbols, k: Kernels) -> Self {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        let n = mtf::mtf(block, &mut ll, &mut counts, k).unwrap();
        let mut tt = TVector::new();
        build(&ll, &counts, &mut tt);
        let (mut plain, mut out, mut seg) = (Vec::new(), Vec::new(), SegScratch::default());
        walk(&tt, block.orig_ptr, &mut plain);
        walk_segments_room(&mut tt, block.orig_ptr, &mut out, &mut seg);
        assert!(
            out == plain,
            "the load block: the room walk against the plain walk"
        );
        let t = table(&ll, &counts);
        let mut p = t[block.orig_ptr as usize];
        let stepped: Vec<u8> = (0..n)
            .map(|_| {
                p = chain(&t, p, 1);
                p as u8
            })
            .collect();
        assert!(
            stepped == plain,
            "the load block: the chain against the plain walk"
        );
        let mut btt = TVector::new();
        build(&ll, &counts, &mut btt);
        // written through, so every page is its own (not the zero page)
        let stream: Vec<u64> = (0..STREAM_BYTES as u64 / 8)
            .map(|i| i.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
            .collect();
        Loads {
            k,
            block,
            n,
            tt,
            p: t[block.orig_ptr as usize],
            chain: t,
            seg,
            out,
            bll: ll.clone(),
            bcounts: counts,
            ll,
            counts,
            btt,
            stream,
            at: 0,
        }
    }

    /// One unit of `load`; returns its work
    fn unit(&mut self, load: Load) -> u64 {
        match load {
            Load::Walk => {
                walk_segments_room(
                    &mut self.tt,
                    self.block.orig_ptr,
                    &mut self.out,
                    &mut self.seg,
                );
                self.n as u64
            }
            Load::Chain => {
                self.p = chain(&self.chain, self.p, CHAIN_STEPS);
                CHAIN_STEPS as u64
            }
            Load::Alu => {
                alu(ALU_ITERS);
                ALU_ITERS
            }
            Load::Mtf => {
                mtf::mtf(self.block, &mut self.ll, &mut self.counts, self.k).unwrap();
                self.n as u64
            }
            Load::Build => {
                build(&self.bll, &self.bcounts, &mut self.btt);
                self.n as u64
            }
            Load::Stream => {
                let words = STREAM_CHUNK / 8;
                black_box(sum(&self.stream[self.at * words..(self.at + 1) * words]));
                self.at = (self.at + 1) % (STREAM_BYTES / STREAM_CHUNK);
                STREAM_CHUNK as u64
            }
        }
    }
}

/// `load` until `stop`, then one unit more, so the log covers the measured thread's last
/// window; the start, then each unit's end, pushed onto `log` (ns since `epoch`, work so
/// far)
fn run_load(
    load: Load,
    st: &mut Loads,
    stop: &AtomicBool,
    epoch: Instant,
    log: &mut Vec<(u64, u64)>,
) {
    log.clear();
    let mut done = 0;
    log.push((ns(epoch), 0));
    loop {
        let last = stop.load(Ordering::Acquire);
        done += st.unit(load);
        log.push((ns(epoch), done));
        if last {
            break;
        }
    }
}

/// Sets the flag when dropped, so a load stops even if the thread timing it panics
struct StopOnDrop<'a>(&'a AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

// ---- A condition's figures ----

/// Each unit of the load's log (its start, then each unit's end, with the work so far)
/// against the windows: (ns inside them, ns outside, work), for every unit that ends after
/// the first window starts and starts before the last ends
fn split(log: &[(u64, u64)], windows: &[(u64, u64)]) -> Vec<(f64, f64, f64)> {
    let (Some(first), Some(last)) = (windows.first(), windows.last()) else {
        return Vec::new();
    };
    let (lo, hi) = (first.0, last.1);
    let mut out = Vec::new();
    let mut w = 0;
    for u in log.windows(2) {
        let ((a, wa), (b, wb)) = (u[0], u[1]);
        if b <= lo || a >= hi {
            continue;
        }
        while w < windows.len() && windows[w].1 <= a {
            w += 1;
        }
        let mut inside = 0u64;
        for &(s, e) in windows[w..].iter().take_while(|x| x.0 < b) {
            inside += e.min(b).saturating_sub(s.max(a));
        }
        out.push((inside as f64, (b - a - inside) as f64, (wb - wa) as f64));
    }
    out
}

/// Whether the log runs from before the first window to after the last
fn covers(log: &[(u64, u64)], windows: &[(u64, u64)]) -> bool {
    match (log.first(), log.last(), windows.first(), windows.last()) {
        (Some(a), Some(b), Some(s), Some(e)) => a.0 <= s.0 && b.0 >= e.1,
        _ => false,
    }
}

/// The load's rate inside the windows (work a ns), from `split`'s units: fitted (a unit's
/// work = rate inside x ns inside + rate outside x ns outside, least squares; with no time
/// outside, the rate inside alone), and interpolated (each unit's work shared in proportion
/// to its time); NaN where it can't be told
fn rates(units: &[(f64, f64, f64)]) -> (f64, f64) {
    let (mut xx, mut xy, mut yy, mut wx, mut wy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let (mut work_in, mut time_in) = (0.0, 0.0);
    for &(x, y, w) in units {
        xx += x * x;
        xy += x * y;
        yy += y * y;
        wx += w * x;
        wy += w * y;
        if x + y > 0.0 {
            work_in += w * x / (x + y);
        }
        time_in += x;
    }
    let interp = if time_in > 0.0 {
        work_in / time_in
    } else {
        f64::NAN
    };
    let det = xx * yy - xy * xy;
    let fit = if xx == 0.0 {
        f64::NAN
    } else if yy == 0.0 {
        wx / xx
    } else if det > 1e-9 * xx * yy {
        (wx * yy - wy * xy) / det
    } else {
        f64::NAN
    };
    (if fit > 0.0 { fit } else { f64::NAN }, interp)
}

/// The longest unit over the median, among `split`'s units
fn spread(units: &[(f64, f64, f64)]) -> f64 {
    let mut d: Vec<f64> = units.iter().map(|u| u.0 + u.1).collect();
    if d.is_empty() {
        return f64::NAN;
    }
    d.sort_by(f64::total_cmp);
    d[d.len() - 1] / d[d.len() / 2]
}

/// One condition in one round
#[derive(Clone, Copy, Debug)]
struct Sample {
    /// the measured stage's time, summed over its windows (s); NaN for a load alone
    stage: f64,
    /// the load's rate inside the windows (work a ns): fitted, and interpolated; NaN with no
    /// load, or where the load's log didn't cover the windows
    fit: f64,
    interp: f64,
    /// the load's longest unit over its median
    spread: f64,
    /// involuntary context switches during the condition: the measured thread's (or the
    /// timekeeper's), the load's
    m_preempt: u64,
    s_preempt: u64,
}

impl Default for Sample {
    fn default() -> Self {
        Sample {
            stage: f64::NAN,
            fit: f64::NAN,
            interp: f64::NAN,
            spread: f64::NAN,
            m_preempt: 0,
            s_preempt: 0,
        }
    }
}

/// What a condition's run needs, besides its buffers
struct Ctx<'a> {
    blocks: &'a [Symbols],
    k: Kernels,
    cpus: Cpus,
}

/// One condition: the load (if any) pinned to its CPU and the measured decode (if any) to
/// M, started behind a barrier; for a load alone, this thread keeps its window
fn run(
    cond: &Condition,
    cx: &Ctx,
    d: &mut Decode,
    loads: &mut Loads,
    windows: &mut Vec<(u64, u64)>,
    log: &mut Vec<(u64, u64)>,
) -> Sample {
    let stop = AtomicBool::new(false);
    let barrier = Barrier::new(1 + usize::from(cond.load.is_some()));
    let epoch = Instant::now();
    windows.clear();
    log.clear();
    let (mut m_preempt, mut s_preempt) = (0, 0);
    std::thread::scope(|s| {
        let load = cond.load.map(|l| {
            let cpu = cx.cpus.of(cond.place).unwrap();
            let (stop, barrier, loads, log) = (&stop, &barrier, &mut *loads, &mut *log);
            s.spawn(move || {
                pin(cpu);
                let p = preempted();
                barrier.wait();
                run_load(l, loads, stop, epoch, log);
                assert_eq!(on_cpu(), cpu as i32, "the load left its CPU");
                preempted() - p
            })
        });
        match cond.kernel {
            Some(kernel) => {
                let (stop, barrier, d, w) = (&stop, &barrier, &mut *d, &mut *windows);
                let (blocks, k, m) = (cx.blocks, cx.k, cx.cpus.measured);
                let h = s.spawn(move || {
                    let _stop = StopOnDrop(stop);
                    pin(m);
                    let p = preempted();
                    barrier.wait();
                    std::thread::sleep(RAMP);
                    decode(blocks, k, kernel, d, epoch, w);
                    assert_eq!(on_cpu(), m as i32, "the measured thread left its CPU");
                    preempted() - p
                });
                m_preempt = h.join().unwrap();
            }
            None => {
                let _stop = StopOnDrop(&stop);
                let p = preempted();
                barrier.wait();
                std::thread::sleep(RAMP);
                let start = ns(epoch);
                std::thread::sleep(LOAD_ALONE);
                windows.push((start, ns(epoch)));
                m_preempt = preempted() - p;
            }
        }
        if let Some(h) = load {
            s_preempt = h.join().unwrap();
        }
    });
    let mut sample = Sample {
        m_preempt,
        s_preempt,
        ..Sample::default()
    };
    if cond.kernel.is_some() {
        sample.stage = windows.iter().map(|&(s, e)| (e - s) as f64).sum::<f64>() * 1e-9;
    }
    if cond.load.is_some() && covers(log, windows) {
        let units = split(log, windows);
        (sample.fit, sample.interp) = rates(&units);
        sample.spread = spread(&units);
    }
    sample
}

// ---- The summary ----

fn finite(xs: impl IntoIterator<Item = f64>) -> Vec<f64> {
    xs.into_iter().filter(|x| x.is_finite()).collect()
}

fn median(xs: &[f64]) -> f64 {
    let mut v = finite(xs.iter().copied());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// The mean of `xs` (the finite ones) with its two-sided 95% t-interval: (mean, lo, hi);
/// the interval NaN with fewer than 2
fn mean_ci(xs: &[f64]) -> (f64, f64, f64) {
    let v = finite(xs.iter().copied());
    let n = v.len();
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let m = v.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, f64::NAN, f64::NAN);
    }
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64;
    let h = T95[(n - 2).min(T95.len() - 1)] * (var / n as f64).sqrt();
    (m, m - h, m + h)
}

/// A figure with its 95% interval: (estimate, low, high)
type Ci = (f64, f64, f64);

/// The geometric mean of positive ratios with its 95% interval (a t-interval on the logs)
fn geo(xs: &[f64]) -> (f64, f64, f64) {
    let logs: Vec<f64> = xs.iter().filter(|&&x| x > 0.0).map(|x| x.ln()).collect();
    let (m, lo, hi) = mean_ci(&logs);
    (m.exp(), lo.exp(), hi.exp())
}

/// The core's time for both siblings' walk (alone time `w` each) and stage X (`x` each),
/// mixed (one sibling walks while the other runs X, at the ratios to alone `f_wx`, walk
/// beside X, and `f_xw`, X beside the walk, until one kind runs out; the rest beside its
/// own kind) over same-type (walk beside walk at `f_ww`, then X beside X at `f_xx`): below
/// 1, the mixed placement takes less of the core
fn pairing(w: f64, x: f64, f_ww: f64, f_wx: f64, f_xw: f64, f_xx: f64) -> f64 {
    let same = w * f_ww + x * f_xx;
    let tau = (2.0 * w * f_wx).min(2.0 * x * f_xw);
    let rest_w = (2.0 * w - tau / f_wx).max(0.0);
    let rest_x = (2.0 * x - tau / f_xw).max(0.0);
    let mixed = tau + rest_w * f_ww / 2.0 + rest_x * f_xx / 2.0;
    mixed / same
}

/// A condition's figures over the rounds
#[derive(Clone, Debug)]
struct Row {
    cond: Condition,
    cpu_load: Option<usize>,
    /// the stage's median c/B (at the reported clock) and ns/B
    cb: f64,
    nsb: f64,
    /// the stage's time over its time alone, paired by round
    ratio: (f64, f64, f64),
    /// the load's median rate inside the windows (work a ns: fitted), and interpolated
    rate: f64,
    rate_interp: f64,
    /// the load's rate over its rate alone, paired by round
    speed: (f64, f64, f64),
    /// the stage's speed plus the load's, by round
    pair_yield: (f64, f64, f64),
    /// the load's longest unit over its median, the most in any round; context switches
    /// summed over the rounds
    spread: f64,
    m_preempt: u64,
    s_preempt: u64,
}

/// Each condition's figures, from samples[round][condition]: `n` bytes walked a decode
fn summarise(
    conds: &[Condition],
    samples: &[Vec<Sample>],
    cpus: &Cpus,
    n: usize,
    ghz: f64,
) -> Vec<Row> {
    let per = |ci: usize, f: &dyn Fn(&Sample) -> f64| -> Vec<f64> {
        samples.iter().map(|round| f(&round[ci])).collect()
    };
    conds
        .iter()
        .enumerate()
        .map(|(ci, c)| {
            let stage = per(ci, &|s| s.stage);
            let rate = per(ci, &|s| if c.kernel.is_some() { s.fit } else { s.interp });
            let ratio: Vec<f64> = match c
                .kernel
                .and_then(|k| find(conds, Some(k), None, Place::Alone))
            {
                Some(a) => stage
                    .iter()
                    .zip(per(a, &|s| s.stage))
                    .map(|(x, a)| x / a)
                    .collect(),
                None => vec![f64::NAN; samples.len()],
            };
            let speed: Vec<f64> = match c
                .load
                .and_then(|l| find(conds, None, Some(l), Place::Alone))
            {
                Some(a) => rate
                    .iter()
                    .zip(per(a, &|s| s.interp))
                    .map(|(x, a)| x / a)
                    .collect(),
                None => vec![f64::NAN; samples.len()],
            };
            let pair_yield: Vec<f64> = match (c.kernel, c.load) {
                (Some(_), Some(_)) => ratio.iter().zip(&speed).map(|(r, s)| 1.0 / r + s).collect(),
                (Some(_), None) => ratio.iter().map(|r| 1.0 / r).collect(),
                _ => vec![f64::NAN; samples.len()],
            };
            let spread = per(ci, &|s| s.spread);
            Row {
                cond: *c,
                cpu_load: c.load.and_then(|_| cpus.of(c.place)),
                cb: median(&stage) * ghz * 1e9 / n as f64,
                nsb: median(&stage) * 1e9 / n as f64,
                ratio: geo(&ratio),
                rate: median(&rate),
                rate_interp: median(&per(ci, &|s| s.interp)),
                speed: geo(&speed),
                pair_yield: mean_ci(&pair_yield),
                spread: finite(spread).into_iter().fold(f64::NAN, f64::max),
                m_preempt: samples.iter().map(|r| r[ci].m_preempt).sum(),
                s_preempt: samples.iter().map(|r| r[ci].s_preempt).sum(),
            }
        })
        .collect()
}

/// A stage's ratio to alone beside load `a.0` at `a.1` over its ratio beside `b.0` at
/// `b.1`, by round; and the share of the second's excess over alone that the first has
fn contrast(
    conds: &[Condition],
    samples: &[Vec<Sample>],
    kernel: Kernel,
    a: (Load, Place),
    b: (Load, Place),
) -> Option<(Ci, Ci)> {
    let alone = find(conds, Some(kernel), None, Place::Alone)?;
    let ia = find(conds, Some(kernel), Some(a.0), a.1)?;
    let ib = find(conds, Some(kernel), Some(b.0), b.1)?;
    let (mut ratio, mut share) = (Vec::new(), Vec::new());
    for round in samples {
        let (fa, fb) = (
            round[ia].stage / round[alone].stage,
            round[ib].stage / round[alone].stage,
        );
        ratio.push(fa / fb);
        share.push((fa - 1.0) / (fb - 1.0));
    }
    Some((geo(&ratio), mean_ci(&share)))
}

/// The pairing model by round for the walk with stage `x`, at the sibling: the mixed
/// placement's core time over same-type, and the four ratios' medians (ww, wx, xw, xx)
fn pairing_of(conds: &[Condition], samples: &[Vec<Sample>], x: Kernel) -> Option<(Ci, [f64; 4])> {
    let (lw, lx) = (
        Load::Walk,
        if x == Kernel::Mtf {
            Load::Mtf
        } else {
            Load::Build
        },
    );
    let at = |k: Kernel, l: Option<Load>, p: Place| find(conds, Some(k), l, p);
    let (aw, ax) = (
        at(Kernel::Walk, None, Place::Alone)?,
        at(x, None, Place::Alone)?,
    );
    let ww = at(Kernel::Walk, Some(lw), Place::Sibling)?;
    let wx = at(Kernel::Walk, Some(lx), Place::Sibling)?;
    let xw = at(x, Some(lw), Place::Sibling)?;
    let xx = at(x, Some(lx), Place::Sibling)?;
    let mut by_round = Vec::new();
    let mut fs = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for round in samples {
        let (w, xa) = (round[aw].stage, round[ax].stage);
        let f = [
            round[ww].stage / w,
            round[wx].stage / w,
            round[xw].stage / xa,
            round[xx].stage / xa,
        ];
        by_round.push(pairing(w, xa, f[0], f[1], f[2], f[3]));
        for (v, f) in fs.iter_mut().zip(f) {
            v.push(f);
        }
    }
    Some((geo(&by_round), fs.map(|v| median(&v))))
}

fn ci3(x: (f64, f64, f64), digits: usize) -> String {
    if !x.0.is_finite() {
        return "n/a".into();
    }
    if !x.1.is_finite() {
        return format!("{:.*}", digits, x.0);
    }
    format!(
        "{:.*} ({:.*}, {:.*})",
        digits, x.0, digits, x.1, digits, x.2
    )
}

fn pct3(x: (f64, f64, f64)) -> String {
    if !x.0.is_finite() {
        return "n/a".into();
    }
    if !x.1.is_finite() {
        return format!("{:+.1}%", 100.0 * (x.0 - 1.0));
    }
    format!(
        "{:+.1}% ({:+.1}%, {:+.1}%)",
        100.0 * (x.0 - 1.0),
        100.0 * (x.1 - 1.0),
        100.0 * (x.2 - 1.0)
    )
}

/// What a run was, for the report's head and the JSON's
struct Head<'a> {
    path: &'a str,
    blocks: usize,
    n: usize,
    ghz: f64,
    rounds: usize,
    topo: &'a Topology,
    cpus: Cpus,
    load_block: (usize, usize),
}

/// The markdown summary
fn report(h: &Head, conds: &[Condition], rows: &[Row], samples: &[Vec<Sample>]) -> String {
    use std::fmt::Write;
    let mut o = String::new();
    let cpu = |c: Option<usize>| c.map_or("none".to_string(), |c| format!("CPU {c}"));
    let _ = writeln!(o, "## Pair matrix: {}", h.path);
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "{} blocks, {:.1} MB walked a decode; c/B of the walk's output at {} GHz reported; {} rounds of {} conditions, their order rotated a step a round.",
        h.blocks,
        h.n as f64 / 1e6,
        h.ghz,
        h.rounds,
        conds.len()
    );
    if h.rounds < 5 {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**A dry run:** fewer than 5 rounds; its intervals are not for decisions."
        );
    }
    let _ = writeln!(o);
    let agree = match h.topo.ids_agree {
        Some(true) => "; core_id and physical_package_id agree",
        Some(false) => "; **core_id or physical_package_id disagree with them**",
        None => "",
    };
    let _ = writeln!(
        o,
        "CPUs by core ({}{agree}): {}. The measured stage on CPU {}; the sibling load on {}; the cross-core load on {}; the main thread on {}.",
        h.topo.source,
        h.topo.show(),
        h.cpus.measured,
        cpu(h.cpus.sibling),
        cpu(h.cpus.cross),
        cpu(h.cpus.spare)
    );
    if h.cpus.sibling.is_none() {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**No SMT here:** no core has two CPUs this process may use, so the sibling conditions are refused and not run. The cross-core and alone conditions ran."
        );
    }
    if h.cpus.cross.is_none() {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**One core here:** no CPU on another core, so the cross-core conditions are not run."
        );
    }
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "The load block: block {} of the file, {} bytes. Loads alone ran on CPU {}, CPU {} idle.",
        h.load_block.0,
        h.load_block.1,
        h.cpus.alone(),
        h.cpus.measured
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "### The loads alone");
    let _ = writeln!(o);
    let _ = writeln!(o, "| load | what | rate (median) | longest unit / median |");
    let _ = writeln!(o, "|---|---|---|---|");
    for r in rows.iter().filter(|r| r.cond.kernel.is_none()) {
        let l = r.cond.load.unwrap();
        let _ = writeln!(
            o,
            "| {} | {} | {} | {:.1} |",
            l.name(),
            l.about(),
            l.show_rate(r.rate),
            r.spread
        );
    }
    for k in KERNELS {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "### {}: {}, on CPU {}",
            k.name(),
            k.title(),
            h.cpus.measured
        );
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "| load | where | c/B | ns/B | x alone (95%) | load rate | load speed (95%) | pair yield (95%) | flags |"
        );
        let _ = writeln!(o, "|---|---|---|---|---|---|---|---|---|");
        for r in rows.iter().filter(|r| r.cond.kernel == Some(k)) {
            let mut flags = Vec::new();
            if r.spread > LONG_UNIT {
                flags.push(format!("a load unit {:.0}x its median", r.spread));
            }
            if r.m_preempt + r.s_preempt > 0 {
                flags.push(format!("preempted {}+{}", r.m_preempt, r.s_preempt));
            }
            let (load, rate) = match r.cond.load {
                Some(l) => (l.name(), l.show_rate(r.rate)),
                None => ("none", String::new()),
            };
            let place = match r.cpu_load {
                Some(c) => format!("{} (CPU {c})", r.cond.place.name()),
                None => r.cond.place.name().to_string(),
            };
            let _ = writeln!(
                o,
                "| {load} | {place} | {:.2} | {:.3} | {} | {rate} | {} | {} | {} |",
                r.cb,
                r.nsb,
                ci3(r.ratio, 2),
                ci3(r.speed, 2),
                ci3(r.pair_yield, 2),
                flags.join("; ")
            );
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "### Contrasts, paired by round");
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "Each stage beside a load on its sibling, against the same stage beside the walk on its sibling (the ratio of the two ratios to alone; and the share of the walk sibling's excess over alone that this load's is), and against the same load on the other core."
    );
    let _ = writeln!(o);
    let mut lines = Vec::new();
    for k in KERNELS {
        for l in LOADS {
            let vs = contrast(
                conds,
                samples,
                k,
                (l, Place::Sibling),
                (Load::Walk, Place::Sibling),
            );
            let x = contrast(conds, samples, k, (l, Place::Sibling), (l, Place::Cross));
            if vs.is_none() && x.is_none() {
                continue;
            }
            let (a, b) = vs.map_or(("n/a".into(), "n/a".into()), |(r, s)| {
                (ci3(r, 3), ci3(s, 2))
            });
            let c = x.map_or("n/a".into(), |(r, _)| ci3(r, 3));
            lines.push(format!("| {} | {} | {a} | {b} | {c} |", k.name(), l.name()));
        }
    }
    if lines.is_empty() {
        let _ = writeln!(o, "None: no sibling conditions ran.");
    } else {
        let _ = writeln!(o, "| stage | load | vs beside the walk (95%) | share of the walk's excess (95%) | sibling vs cross (95%) |");
        let _ = writeln!(o, "|---|---|---|---|---|");
        for l in lines {
            let _ = writeln!(o, "{l}");
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "### Pairing the walk with a compute stage, at the sibling"
    );
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "The core's time for both siblings' walk and the other stage (each stage's time alone, this run's), mixed (one sibling walks while the other runs the other stage, until one kind runs out, the rest beside its own kind) over same-type (walk beside walk, then the other beside its own kind). Below 0: a co-run-aware placement would take that much less of the core for those two stages. An ideal: the four ratios fixed, the stages free to be reordered (a decode orders a block's stages)."
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "| with | walk beside walk | walk beside it | it beside the walk | it beside itself | mixed vs same-type (95%) |");
    let _ = writeln!(o, "|---|---|---|---|---|---|");
    for x in [Kernel::Mtf, Kernel::Build] {
        match pairing_of(conds, samples, x) {
            Some((g, f)) => {
                let _ = writeln!(
                    o,
                    "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {} |",
                    x.name(),
                    f[0],
                    f[1],
                    f[2],
                    f[3],
                    pct3(g)
                );
            }
            None => {
                let _ = writeln!(o, "| {} | n/a (no sibling conditions) | | | | |", x.name());
            }
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "c/B and ns/B: the stage's time summed over its windows, the median round. x alone: its time over its time alone, the geometric mean of the rounds' ratios. Load rate: inside the stage's windows (fitted), the median round. Load speed: that rate over the load's alone. Pair yield: the stage's speed (1 / x alone) plus the load's; 2 when neither slows the other. Flags: a load unit far longer than its median (the load likely preempted), and involuntary context switches (measured + load threads, all rounds)."
    );
    o
}

fn jnum(x: f64) -> String {
    if x.is_finite() {
        format!("{x}")
    } else {
        "null".into()
    }
}

fn jstr(s: &str) -> String {
    let mut o = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn jarr(xs: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", xs.into_iter().collect::<Vec<_>>().join(","))
}

fn jci(x: (f64, f64, f64)) -> String {
    jarr([jnum(x.0), jnum(x.1), jnum(x.2)])
}

fn jopt(x: Option<usize>) -> String {
    x.map_or("null".into(), |c| c.to_string())
}

/// Everything as one JSON object: the run, each condition's samples by round and its
/// summary, the contrasts and the pairing
fn json(h: &Head, conds: &[Condition], rows: &[Row], samples: &[Vec<Sample>]) -> String {
    let topo = format!(
        "{{\"cores\":{},\"source\":{},\"ids_agree\":{},\"smt\":{},\"measured\":{},\"sibling\":{},\"cross\":{},\"spare\":{},\"load_alone\":{}}}",
        jarr(h.topo.cores.iter().map(|c| jarr(c.iter().map(|x| x.to_string())))),
        jstr(h.topo.source),
        h.topo.ids_agree.map_or("null".into(), |b| b.to_string()),
        h.topo.smt(),
        h.cpus.measured,
        jopt(h.cpus.sibling),
        jopt(h.cpus.cross),
        jopt(h.cpus.spare),
        h.cpus.alone()
    );
    let per = |ci: usize, f: &dyn Fn(&Sample) -> String| jarr(samples.iter().map(|r| f(&r[ci])));
    let cs = conds.iter().zip(rows).enumerate().map(|(ci, (c, r))| {
        format!(
            "{{\"id\":{},\"kernel\":{},\"load\":{},\"placement\":{},\"cpu_measured\":{},\"cpu_load\":{},\"load_unit\":{},\
             \"stage_s\":{},\"load_fit_per_ns\":{},\"load_interp_per_ns\":{},\"load_spread\":{},\"m_preempt\":{},\"s_preempt\":{},\
             \"summary\":{{\"cb\":{},\"ns_b\":{},\"ratio\":{},\"load_rate_per_ns\":{},\"load_rate_interp_per_ns\":{},\"load_speed\":{},\"pair_yield\":{},\"load_spread_max\":{}}}}}",
            jstr(&c.id()),
            c.kernel.map_or("null".into(), |k| jstr(k.name())),
            c.load.map_or("null".into(), |l| jstr(l.name())),
            jstr(c.place.name()),
            c.kernel.map_or("null".into(), |_| h.cpus.measured.to_string()),
            jopt(r.cpu_load),
            c.load.map_or("null".into(), |l| jstr(l.unit())),
            per(ci, &|s| jnum(s.stage)),
            per(ci, &|s| jnum(s.fit)),
            per(ci, &|s| jnum(s.interp)),
            per(ci, &|s| jnum(s.spread)),
            per(ci, &|s| s.m_preempt.to_string()),
            per(ci, &|s| s.s_preempt.to_string()),
            jnum(r.cb),
            jnum(r.nsb),
            jci(r.ratio),
            jnum(r.rate),
            jnum(r.rate_interp),
            jci(r.speed),
            jci(r.pair_yield),
            jnum(r.spread),
        )
    });
    let mut contrasts = Vec::new();
    for k in KERNELS {
        for l in LOADS {
            for (vs, b) in [
                ("walk_sibling", (Load::Walk, Place::Sibling)),
                ("cross", (l, Place::Cross)),
            ] {
                if let Some((r, s)) = contrast(conds, samples, k, (l, Place::Sibling), b) {
                    // the share of the walk sibling's excess (meaningless against cross)
                    let share = if vs == "cross" { "null".into() } else { jci(s) };
                    contrasts.push(format!(
                        "{{\"kernel\":{},\"load\":{},\"against\":{},\"ratio\":{},\"excess_share\":{share}}}",
                        jstr(k.name()),
                        jstr(l.name()),
                        jstr(vs),
                        jci(r),
                    ));
                }
            }
        }
    }
    let pairing = [Kernel::Mtf, Kernel::Build].into_iter().filter_map(|x| {
        pairing_of(conds, samples, x).map(|(g, f)| {
            format!(
                "{{\"with\":{},\"mixed_over_same\":{},\"f_ww\":{},\"f_wx\":{},\"f_xw\":{},\"f_xx\":{}}}",
                jstr(x.name()),
                jci(g),
                jnum(f[0]),
                jnum(f[1]),
                jnum(f[2]),
                jnum(f[3])
            )
        })
    });
    format!(
        "{{\"tool\":\"pair\",\"file\":{},\"blocks\":{},\"bytes\":{},\"ghz\":{},\"rounds\":{},\"dry_run\":{},\"topology\":{topo},\
         \"load_block\":{{\"index\":{},\"bytes\":{}}},\"conditions\":{},\"contrasts\":{},\"pairing\":{}}}\n",
        jstr(h.path),
        h.blocks,
        h.n,
        jnum(h.ghz),
        h.rounds,
        h.rounds < 5,
        h.load_block.0,
        h.load_block.1,
        jarr(cs),
        jarr(contrasts),
        jarr(pairing),
    )
}

// ---- main ----

fn main() {
    let usage = "usage: pair FILE.bz2 [ROUNDS] [GHZ] [--json OUT.json] [--blocks N] [--cpus M,SIBLING,CROSS]";
    let mut pos = Vec::new();
    let (mut json_out, mut max_blocks, mut cpus_arg) = (None, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--json" => json_out = Some(args.next().expect(usage)),
            "--blocks" => {
                max_blocks = Some(args.next().expect(usage).parse::<usize>().expect(usage))
            }
            "--cpus" => cpus_arg = Some(args.next().expect(usage)),
            _ => pos.push(a),
        }
    }
    let path = pos.first().expect(usage).clone();
    let rounds: usize = pos.get(1).map_or(5, |s| s.parse().expect(usage));
    let ghz: f64 = pos.get(2).map_or(2.8, |s| s.parse().expect(usage));
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");

    let topo = read_topology();
    let chosen = match &cpus_arg {
        Some(s) => parse_cpus(s, &topo.cores),
        None => choose(&topo.cores),
    };
    let cpus = chosen.unwrap_or_else(|e| {
        eprintln!("pair: {e}");
        std::process::exit(2);
    });
    if cpus.sibling.is_none() {
        eprintln!(
            "pair: no SMT here (CPUs by core: {}), so the sibling conditions are refused; the cross-core and alone conditions run",
            topo.show()
        );
    }
    // each CPU pinned to once before the matrix, so a refusal stops the run here, not with
    // a condition's other thread waiting at its barrier
    for c in [Some(cpus.measured), cpus.sibling, cpus.cross]
        .into_iter()
        .flatten()
    {
        std::thread::spawn(move || pin(c))
            .join()
            .expect("pinning a thread");
    }
    if let Some(c) = cpus.spare {
        pin(c);
    }

    let data = std::fs::read(&path).unwrap();
    let k = Kernels::detect();
    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }
    if let Some(m) = max_blocks {
        assert!(m >= 1, "--blocks: at least 1");
        blocks.truncate(m);
    }
    assert!(!blocks.is_empty(), "{path}: no blocks");

    // The check (each block's CRC) and the bytes walked; the load block, the largest
    let (mut d, mut windows) = (Decode::new(), Vec::new());
    let n = decode(
        &blocks,
        k,
        Kernel::Walk,
        &mut d,
        Instant::now(),
        &mut windows,
    );
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let mut lb = (0, 0);
    for (i, b) in blocks.iter().enumerate() {
        let len = mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
        if len > lb.1 {
            lb = (i, len);
        }
    }
    let mut loads = Loads::new(&blocks[lb.0], k);
    let mut log = Vec::with_capacity(1 << 17);
    windows.reserve(4 * blocks.len());

    let conds = conditions(cpus.sibling.is_some(), cpus.cross.is_some());
    eprintln!(
        "pair: {} blocks, {:.1} MB walked; {} conditions a round, {rounds} rounds; measured on CPU {}, sibling {:?}, cross {:?}",
        blocks.len(),
        n as f64 / 1e6,
        conds.len(),
        cpus.measured,
        cpus.sibling,
        cpus.cross
    );
    let cx = Ctx {
        blocks: &blocks,
        k,
        cpus,
    };
    let mut samples = vec![vec![Sample::default(); conds.len()]; rounds];
    for (r, round) in samples.iter_mut().enumerate() {
        let t = Instant::now();
        for ci in order(r, conds.len()) {
            round[ci] = run(&conds[ci], &cx, &mut d, &mut loads, &mut windows, &mut log);
        }
        eprintln!(
            "pair: round {}/{rounds}, {:.1} s",
            r + 1,
            t.elapsed().as_secs_f64()
        );
    }

    let head = Head {
        path: &path,
        blocks: blocks.len(),
        n,
        ghz,
        rounds,
        topo: &topo,
        cpus,
        load_block: lb,
    };
    let rows = summarise(&conds, &samples, &cpus, n, ghz);
    print!("{}", report(&head, &conds, &rows, &samples));
    if let Some(p) = json_out {
        std::fs::write(&p, json(&head, &conds, &rows, &samples)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ci_layout(c: usize) -> Option<Vec<usize>> {
        Some(if c < 2 { vec![0, 1] } else { vec![2, 3] })
    }
    fn ci_ids(c: usize) -> Option<(u32, u32)> {
        Some((0, (c / 2) as u32))
    }

    #[test]
    fn cpu_lists() {
        assert_eq!(cpu_list("0-3,6\n"), vec![0, 1, 2, 3, 6]);
        assert_eq!(cpu_list("5"), vec![5]);
        assert_eq!(cpu_list(""), Vec::<usize>::new());
    }

    #[test]
    fn topology_groups_by_siblings_and_checks_the_ids() {
        // CI's runners: 4 CPUs on 2 cores, siblings 0-1 | 2-3, core_id 0 0 1 1
        let t = group(&[0, 1, 2, 3], ci_layout, ci_ids);
        assert_eq!(t.cores, vec![vec![0, 1], vec![2, 3]]);
        assert_eq!(t.source, "thread_siblings_list");
        assert_eq!(t.ids_agree, Some(true));
        assert!(t.smt());
        // ids that contradict the siblings lists are reported
        let t = group(&[0, 1, 2, 3], ci_layout, |c| Some((0, c as u32)));
        assert_eq!(t.ids_agree, Some(false));
        // siblings interleaved (0,4 | 1,5 | ...), as many Intel parts number them
        let t = group(
            &(0..8).collect::<Vec<_>>(),
            |c| Some(vec![c % 4, c % 4 + 4]),
            |c| Some((0, (c % 4) as u32)),
        );
        assert_eq!(
            t.cores,
            vec![vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]]
        );
        assert_eq!(t.ids_agree, Some(true));
        // a VM without SMT: one CPU a core
        let t = group(&[0, 1, 2, 3], |c| Some(vec![c]), |c| Some((0, c as u32)));
        assert_eq!(t.cores, vec![vec![0], vec![1], vec![2], vec![3]]);
        assert!(!t.smt());
        // a CPU set that allows one sibling of each core
        let t = group(&[0, 2], ci_layout, ci_ids);
        assert_eq!(t.cores, vec![vec![0], vec![2]]);
        // siblings unreadable: by the ids; nothing readable: each CPU alone
        let t = group(&[0, 1, 2, 3], |_| None, ci_ids);
        assert_eq!(
            (t.cores.clone(), t.source),
            (vec![vec![0, 1], vec![2, 3]], "core_id")
        );
        let t = group(&[0, 1], |_| None, |_| None);
        assert_eq!((t.cores, t.source), (vec![vec![0], vec![1]], "none"));
    }

    #[test]
    fn the_cpus_chosen() {
        let ci = vec![vec![0, 1], vec![2, 3]];
        let c = choose(&ci).unwrap();
        assert_eq!(
            c,
            Cpus {
                measured: 0,
                sibling: Some(1),
                cross: Some(2),
                spare: Some(3)
            }
        );
        assert_eq!((c.alone(), c.of(Place::Cross)), (1, Some(2)));
        // interleaved siblings: M's sibling is CPU 4, the other core's first CPU is 1
        let c = choose(&[vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare),
            (0, Some(4), Some(1), Some(5))
        );
        // no SMT: no sibling, the cross-core conditions only; loads alone on the cross CPU
        let c = choose(&[vec![0], vec![1], vec![2], vec![3]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare),
            (0, None, Some(1), Some(2))
        );
        assert_eq!(c.alone(), 1);
        // an SMT core that isn't the first
        let c = choose(&[vec![0], vec![1, 2]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare),
            (1, Some(2), Some(0), None)
        );
        // one core of two siblings: no cross-core CPU
        let c = choose(&[vec![0, 1]]).unwrap();
        assert_eq!((c.sibling, c.cross), (Some(1), None));
        // one CPU: nothing to run a load on
        assert!(choose(&[vec![0]]).is_err());
    }

    #[test]
    fn cpus_named() {
        let ci = vec![vec![0, 1], vec![2, 3]];
        let c = parse_cpus("1,0,3", &ci).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare),
            (1, Some(0), Some(3), Some(2))
        );
        let c = parse_cpus("2,-,0", &ci).unwrap();
        assert_eq!((c.sibling, c.cross), (None, Some(0)));
        assert!(
            parse_cpus("0,2,3", &ci).is_err(),
            "a sibling on another core"
        );
        assert!(parse_cpus("0,0,2", &ci).is_err(), "M as its own sibling");
        assert!(
            parse_cpus("0,1,1", &ci).is_err(),
            "a cross-core CPU on M's core"
        );
        assert!(parse_cpus("0,-,-", &ci).is_err(), "no load CPU");
        assert!(parse_cpus("7,-,2", &ci).is_err(), "M not allowed");
        assert!(parse_cpus("0,1", &ci).is_err(), "three fields");
    }

    #[test]
    fn the_condition_list() {
        let all = conditions(true, true);
        // each stage alone, beside 6 loads at 2 places; each load alone
        assert_eq!(all.len(), 3 * (1 + 6 * 2) + 6);
        let ids: Vec<String> = all.iter().map(Condition::id).collect();
        for (i, id) in ids.iter().enumerate() {
            assert!(!ids[..i].contains(id), "{id} twice");
        }
        for k in KERNELS {
            assert!(find(&all, Some(k), None, Place::Alone).is_some());
            for l in LOADS {
                assert!(find(&all, Some(k), Some(l), Place::Sibling).is_some());
                assert!(find(&all, Some(k), Some(l), Place::Cross).is_some());
            }
        }
        for l in LOADS {
            assert!(find(&all, None, Some(l), Place::Alone).is_some());
        }
        // a stage has a load unless alone; a condition has a stage or a load
        for c in &all {
            assert!(c.kernel.is_some() || c.load.is_some());
            if c.kernel.is_some() {
                assert_eq!(c.load.is_none(), c.place == Place::Alone, "{}", c.id());
            }
        }
        assert_eq!(ids[0], "walk|none|alone");
        assert_eq!(ids[1], "walk|walk|sibling");
        assert_eq!(ids[2], "walk|walk|cross");
        // no SMT: no sibling conditions
        let x = conditions(false, true);
        assert_eq!(x.len(), 3 * (1 + 6) + 6);
        assert!(x.iter().all(|c| c.place != Place::Sibling));
        assert_eq!(conditions(true, false).len(), 3 * (1 + 6) + 6);
    }

    #[test]
    fn rounds_rotate_the_order() {
        let len = 7;
        let mut seen = vec![vec![0; len]; len];
        for r in 0..len {
            let o = order(r, len);
            let mut s = o.clone();
            s.sort();
            assert_eq!(s, (0..len).collect::<Vec<_>>(), "a permutation");
            for (pos, &c) in o.iter().enumerate() {
                seen[c][pos] += 1;
            }
        }
        assert!(
            seen.iter().flatten().all(|&x| x == 1),
            "each condition in each place once"
        );
    }

    #[test]
    fn units_against_windows() {
        // units [0,10) [10,20) [20,40) of work 100 each; windows [5,15) and [25,30)
        let log = [(0, 0), (10, 100), (20, 200), (40, 300)];
        let w = [(5, 15), (25, 30)];
        assert!(covers(&log, &w));
        assert_eq!(
            split(&log, &w),
            vec![(5.0, 5.0, 100.0), (5.0, 5.0, 100.0), (5.0, 15.0, 100.0)]
        );
        // a log that starts after the first window, or ends before the last
        assert!(!covers(&log[1..], &w));
        assert!(!covers(&log[..3], &w));
        // units wholly outside the span are left out
        let log = [(0, 0), (2, 10), (4, 20), (14, 120), (50, 220)];
        assert_eq!(split(&log, &[(5, 10)]), vec![(5.0, 5.0, 100.0)]);
    }

    #[test]
    fn the_load_rate_inside_the_windows() {
        // a rate of 2 inside and 1 outside, units straddling the windows: the fit finds 2,
        // interpolation is smeared towards 1
        let units = [
            (10.0, 0.0, 20.0),
            (5.0, 5.0, 15.0),
            (0.0, 10.0, 10.0),
            (2.0, 8.0, 12.0),
        ];
        let (fit, interp) = rates(&units);
        assert!((fit - 2.0).abs() < 1e-12, "{fit}");
        assert!(interp < 2.0 && interp > 1.0, "{interp}");
        // one window covering every unit (a load alone): both the plain rate
        let (fit, interp) = rates(&[(10.0, 0.0, 30.0), (10.0, 0.0, 30.0)]);
        assert_eq!((fit, interp), (3.0, 3.0));
        // collinear units can't be told apart: no fit
        let (fit, interp) = rates(&[(5.0, 5.0, 10.0), (5.0, 5.0, 10.0)]);
        assert!(fit.is_nan());
        assert_eq!(interp, 1.0);
        assert!(rates(&[]).0.is_nan());
        assert_eq!(
            spread(&[(1.0, 0.0, 1.0), (1.0, 1.0, 1.0), (0.0, 10.0, 1.0)]),
            5.0
        );
    }

    #[test]
    fn the_intervals() {
        let (m, lo, hi) = geo(&[2.0, 2.0, 2.0]);
        assert!((m - 2.0).abs() < 1e-12 && (lo - 2.0).abs() < 1e-12 && (hi - 2.0).abs() < 1e-12);
        // logs 0 and ln 4: mean ln 2, sd ln 4 / sqrt 2, se ln 2, t(1) 12.706
        let (m, lo, hi) = geo(&[1.0, 4.0]);
        let h = 12.706 * 2f64.ln();
        assert!((m - 2.0).abs() < 1e-12);
        assert!((lo / (2f64.ln() - h).exp() - 1.0).abs() < 1e-12);
        assert!((hi / (2f64.ln() + h).exp() - 1.0).abs() < 1e-12);
        let (m, lo, hi) = mean_ci(&[1.0, 2.0, 3.0, f64::NAN]);
        let h = 4.303 * (1.0f64 / 3.0).sqrt();
        assert!(
            (m - 2.0).abs() < 1e-12
                && (lo - (2.0 - h)).abs() < 1e-12
                && (hi - (2.0 + h)).abs() < 1e-12
        );
        let (m, lo, _) = mean_ci(&[5.0]);
        assert!(m == 5.0 && lo.is_nan());
        assert_eq!(median(&[3.0, f64::NAN, 1.0, 2.0]), 2.0);
    }

    #[test]
    fn the_pairing_model() {
        // nothing slows anything: the placements tie
        assert!((pairing(1.0, 1.0, 1.0, 1.0, 1.0, 1.0) - 1.0).abs() < 1e-12);
        assert!((pairing(2.0, 1.0, 1.0, 1.0, 1.0, 1.0) - 1.0).abs() < 1e-12);
        // every sibling costs the same: the placements tie
        assert!((pairing(1.0, 0.5, 2.0, 2.0, 2.0, 2.0) - 1.0).abs() < 1e-12);
        // walk beside walk 2x, the other beside its own 1.4x; walk beside it 1.2x, it beside
        // the walk 1.4x, equal work: same 3.4, mixed 2.4 + 0.2 = 2.6
        assert!((pairing(1.0, 1.0, 2.0, 1.2, 1.4, 1.4) - 2.6 / 3.4).abs() < 1e-12);
        // a walk beside anything else costs more than beside a walk: mixed loses
        assert!(pairing(1.0, 1.0, 1.5, 2.0, 2.0, 1.5) > 1.0);
    }

    fn sample(stage: f64, rate: f64) -> Sample {
        Sample {
            stage,
            fit: rate,
            interp: rate,
            spread: 1.0,
            ..Sample::default()
        }
    }

    #[test]
    fn the_summary_arithmetic() {
        let cpus = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let conds = conditions(true, true);
        // every stage 1 s alone and 2 s beside a load; every load at rate 1 alone, 0.5 beside
        let samples: Vec<Vec<Sample>> = (0..3)
            .map(|_| {
                conds
                    .iter()
                    .map(|c| match (c.kernel, c.load) {
                        (Some(_), None) => sample(1.0, f64::NAN),
                        (Some(_), Some(_)) => sample(2.0, 0.5),
                        (None, _) => sample(f64::NAN, 1.0),
                    })
                    .collect()
            })
            .collect();
        let rows = summarise(&conds, &samples, &cpus, 1_000_000_000, 3.0);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let r = &rows[find(&conds, Some(Kernel::Walk), Some(Load::Alu), Place::Sibling).unwrap()];
        assert!(near(r.ratio.0, 2.0) && near(r.speed.0, 0.5) && near(r.pair_yield.0, 1.0));
        // 2 s over 1e9 bytes: 2 ns/B, 6 c/B at 3 GHz
        assert!(near(r.nsb, 2.0) && near(r.cb, 6.0));
        assert_eq!(r.cpu_load, Some(1));
        let a = &rows[find(&conds, Some(Kernel::Mtf), None, Place::Alone).unwrap()];
        assert!(near(a.ratio.0, 1.0) && near(a.pair_yield.0, 1.0) && a.speed.0.is_nan());
        let x = &rows[find(
            &conds,
            Some(Kernel::Build),
            Some(Load::Stream),
            Place::Cross,
        )
        .unwrap()];
        assert_eq!(x.cpu_load, Some(2));
        let l = &rows[find(&conds, None, Some(Load::Chain), Place::Alone).unwrap()];
        assert!(near(l.rate, 1.0) && l.ratio.0.is_nan());
        // every sibling the same: contrasts at 1, the walk's excess all there; pairing ties
        let (vs, share) = contrast(
            &conds,
            &samples,
            Kernel::Walk,
            (Load::Alu, Place::Sibling),
            (Load::Walk, Place::Sibling),
        )
        .unwrap();
        assert!(near(vs.0, 1.0) && near(share.0, 1.0));
        let (g, f) = pairing_of(&conds, &samples, Kernel::Mtf).unwrap();
        assert!(near(g.0, 1.0) && f.iter().all(|&f| near(f, 2.0)));
        // no SMT: no pairing table, no sibling contrast
        let conds = conditions(false, true);
        let samples = vec![vec![sample(1.0, 1.0); conds.len()]; 2];
        assert!(pairing_of(&conds, &samples, Kernel::Build).is_none());
        assert!(contrast(
            &conds,
            &samples,
            Kernel::Walk,
            (Load::Alu, Place::Sibling),
            (Load::Walk, Place::Sibling)
        )
        .is_none());
    }

    #[test]
    fn json_values() {
        assert_eq!(jnum(f64::NAN), "null");
        assert_eq!(jnum(1.5), "1.5");
        assert_eq!(jstr("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
        assert_eq!(jci((1.0, f64::INFINITY, 2.0)), "[1,null,2]");
    }

    #[test]
    fn the_plain_table_and_chain() {
        // a small last column: the layout holds for any
        let ll = b"nnbaaa".to_vec();
        let mut counts = [0u32; 256];
        for &c in &ll {
            counts[c as usize] += 1;
        }
        let t = table(&ll, &counts);
        // every slot written once: the links are a permutation of the rows
        let mut links: Vec<u32> = t.iter().map(|e| e >> 8).collect();
        links.sort();
        assert_eq!(links, (0..6).collect::<Vec<u32>>());
        // and each slot's byte is ll's at that slot
        assert!(t.iter().enumerate().all(|(s, e)| *e as u8 == ll[s]));
        // the chain's step is one table read
        assert_eq!(chain(&t, t[0], 1), t[(t[0] >> 8) as usize]);
    }

    #[test]
    fn the_loads_run_their_units() {
        alu(1);
        alu(1000);
        let words: Vec<u64> = (1..=16).collect();
        assert_eq!(sum(&words), (0..8).fold(0, |x, j| x ^ ((j + 1) + (j + 9))));
    }
}
