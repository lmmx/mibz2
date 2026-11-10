//! The pair matrix: one decode stage measured on one CPU while a chosen load runs without
//! pause beside it, on the stage's SMT sibling (the same core) or on a CPU of another core,
//! each condition paired by round against the stage alone. A diagnostic, from the Actions
//! tab only (sweep.yml, `tool: pair`): which kind of sibling slows the walk, and whether a
//! walk beside a compute stage costs the core less than a walk beside a walk. Its figures
//! are the run's summary and artifact, never recorded; the decoder's A/B decides changes.
//! What it decides between, and the rule that reads it, are registered in
//! docs/journal/2026-11-05-pair-matrix.md.
//!
//! ```text
//! pair FILE.bz2 [ROUNDS] [GHZ] [--json OUT.json] [--blocks N] [--cpus M,SIBLING,CROSS] [--seed N]
//! ```
//!
//! **The CPUs.** Those this process may run on, grouped by physical core: by sysfs's
//! `thread_siblings_list`, checked against each CPU's `core_id` and `physical_package_id`
//! (by those ids where a siblings list can't be read). The measured stage runs on the first
//! CPU of the first core with two CPUs (M); the sibling conditions' load on that core's
//! next CPU (S); the cross-core conditions' load on the first CPU of another core (X); this
//! process's main thread, which only waits, on a CPU none of them uses, where there is one.
//! With no SMT (every core one CPU, as on many VMs) the sibling conditions are refused, with
//! a message saying so, and the cross-core and alone conditions run. `--cpus` names the
//! three CPUs instead (`-` for none), checked against the topology. Where `--cpus` is not
//! given and the other core has two CPUs, odd rounds swap the cores (CI: M 0, S 1, X 2 in
//! even rounds; M 2, S 3, X 0 in odd ones), so the guest's housekeeping CPU is the
//! measured one in half the rounds; every condition is still paired within its round.
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
//! **The loads**, on the load's CPU, started with the measured thread behind a barrier (the
//! measured thread then sleeps 2 ms before its first block, so the load is already running)
//! and run without pause until the measured decode ends, then one unit more. Beside every
//! stage:
//! - `walk`: the room walk of the load's own block (the largest of the blocks measured, in
//!   its own table), again and again;
//! - `chain`: the plain walk's one dependent chain (one line in flight: `bwt::walk`'s step)
//!   over the same block's table, built here in the library's layout and its bytes checked
//!   against `bwt::walk`'s, 4096 steps a unit;
//! - `alu`: an L1-resident, uop-heavy integer loop with no memory (x86_64: 12 independent
//!   adds and xors an iteration over six registers, more than the core's ALUs issue);
//! - `mtf`, `build`: MTF + RLE2, or the T-vector build, of the load block, again and again;
//! - `stream`: a sequential read of a 64 MB buffer, 1 MiB a unit: larger than CI's 32 MB
//!   L3, so it evicts the measured walk's table (the DRAM and L3-eviction control);
//! - `decode`: whole decodes of the measured blocks, a block a unit, from a cursor that
//!   runs on from condition to condition: today's mixed sibling, a 4-copy run's.
//!
//! Beside the walk only (the stage the penalty is in), the loads that separate the rival
//! mechanisms of docs/journal/2026-11-05-corun-placement.md:
//! - `pause`: `spin_loop()` (x86 PAUSE) 4096 times a unit: an active sibling with an empty
//!   window;
//! - `l1store`: four store streams over a 16 KiB buffer (8-byte volatile stores, no loads),
//!   256 KiB a unit: the store queue and commit, nothing past L1;
//! - `chase32l2`: 32 independent chains, loads only, over a 256 KiB single-cycle
//!   permutation: lines in flight that hit L2 (256 KiB: L2-resident on every CI model,
//!   Zen 3's 512 KiB included, which the 0.75 MB first proposed is not; 5-8x the L1D);
//! - `chase32l3`: 32 chains over the load block's table (the chain's 3.6 MB), started 1/32
//!   of its one cycle apart, loads only: the walk's lines in flight without its staging
//!   stores and assembly. Neither chase writes its table; each keeps its 32 positions in an
//!   L1-resident array (no x86_64 register file holds 32), so a step is a table load, the
//!   position's load and its store, forwarded to the next step;
//! - `stream8`: the sequential read over an 8 MB buffer: L3-resident, above every CI L2;
//! - `aluduty`: the ALU loop in bursts with short idles, C2's short-idle premise: 20 us of
//!   ALU work, then a 20 us sleep (nanosleep, timer slack 1 ns), so the CPU's idle path
//!   (halt, or the guest's poll) runs between bursts; the busy share and period achieved
//!   are reported;
//! - `together`: the same block started together, C21's premise: the load decodes the
//!   measured blocks in step with the measured decode, the two walks of each block released
//!   by one rendezvous, so both siblings walk the same block at once (each its own table);
//!   once the measured decode's blocks are done it decodes on freely. It has no alone of its
//!   own, so no load speed.
//!
//! **The other core busy.** For the stages alone, for walk|walk, walk|mtf, walk|build,
//! mtf|walk, mtf|mtf, build|walk, build|build, walk|decode, mtf|decode and build|decode at
//! the sibling, the condition runs again with unmeasured full-decode loops pinned to both
//! CPUs of the other core (CI's 4-thread row). Their ratios are to the stage alone with the
//! machine otherwise idle, as phases' 4-copy factors are.
//!
//! **The anchor.** walk|decode at the sibling with the other core busy is the 4-copy
//! decode's walk. Its fastest-of-rounds ratio (phases' statistic) must land within 0.15 of
//! the phases 4-copy factor on the same model (x2.12 on the 7763, x2.22 on the 9V74, x2.23 on
//! the 9V45; x1.61 on the 8370C, the walk as one); the report's ANCHOR line says PASS or
//! FAIL, and on FAIL the instrument or its alone is off and no reading is made.
//!
//! **The load's own throughput** while the stage is timed: the load logs the time and its
//! work done at each unit's end; each unit is split into its time inside the measured
//! stage's windows and outside them, and the load's rate inside is fitted by least squares
//! (a unit's work = rate inside x time inside + rate outside x time outside). That undoes
//! the smearing of units as long as a window (the walk, MTF, build and decode loads take a
//! block a unit); the rate interpolated within each unit is kept beside it. Against the load
//! alone on S (M idle, a 0.3 s window), paired by round, it is the load's speed; the pair's
//! yield is the stage's speed (its time alone over its time) plus the load's: 2 when neither
//! slows the other, 1 when the core only takes turns. A load unit more than 1 ms longer than
//! its work at the median time a unit of work is counted as long (the load likely
//! preempted).
//!
//! **Fixed work.** For walk|walk, walk|mtf, walk|build, mtf|mtf, build|build and
//! decode|decode at the sibling, both siblings get the same fixed work, released by one
//! barrier: the walk, MTF or build of the load block 96 times (each side its own buffers and
//! table, L3-warm as the loads'), or whole decodes of 64 blocks spread through the file.
//! Each side's finish and the makespan are reported, so the pair's throughput including its
//! tail (one sibling done, the other running alone) is measured: its yield is the two
//! sides' times alone (each work alone on M, S idle) over the makespan. mtf|mtf and
//! build|build make the measured mixed-against-same-type ratio. About 1.1 s a round on a
//! 7763.
//!
//! **Rounds.** Every condition once a round. The conditions are grouped by stage (the loads
//! alone, and the fixed work, a group each); each round shuffles the groups' order and the
//! order within each group (seeded: `--seed`, else from the clock; the seed is in the JSON),
//! and each stage's group begins and ends with the stage alone, so each condition is paired
//! with the mean of its round's two alones. A stage's ratio to alone is the geometric mean
//! of the rounds' ratios with its 95% interval (a t-interval on the logs, as the sweep's),
//! with the median-of-rounds ratio, the fastest-of-rounds ratio (phases'), and the
//! geometric mean over the rounds with no involuntary context switch in the condition or
//! its alones beside it. At least 7 rounds for a reading (the documented dispatch runs 9);
//! fewer are a dry run, and say so.
//!
//! **Around every condition** the run records each CPU's busy jiffies (/proc/stat: user,
//! nice, system, irq and softirq; steal apart) and interrupts (/proc/interrupts), and flags a
//! condition whose measured core's other CPU, idle by design (a stage alone, beside a
//! cross-core load, or a load alone's M), was more than 1% busy (the resolution is a jiffy,
//! 10 ms: one jiffy in a 0.6 s decode flags). The JSON's head names the CPU model, kernel,
//! cpuidle driver and governor, transparent hugepages, /proc/cmdline and the GitHub run.
//!
//! **Cost**, a round with SMT and another core: 75 measured decodes (28 beside a sibling
//! load, 28 beside a cross-core load, 10 beside a sibling load with the other core busy, 3
//! alone with it busy, 6 alone: each stage twice), 13 load-alone windows of 0.3 s and 10
//! fixed-work runs (about 1.1 s in all). On a 7763 a decode alone takes about 0.6 s (4.27 +
//! 1.98 + 2.16 c/B and RLE1 and the CRC, over 209.7 MB at 3.24 GHz), 0.85-1.0 s beside a
//! sibling: about 60-70 s a round, 9-11 min at 9 rounds (an estimate; the report prints
//! the rounds' measured times).
//!
//! **Not built: the kernel axis.** The window-light walks (the AVX2 gather walk, the
//! one-store-a-step room walk) as measured stages are on research branches not yet merged
//! (`research/window-gather`, `research/window-onestore`); the plain walk as a stage would
//! cost about 2.6 s a decode. They join the matrix when they land.

use std::hint::{black_box, spin_loop};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Barrier;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

const USAGE: &str = "usage: pair FILE.bz2 [ROUNDS] [GHZ] [--json OUT.json] [--blocks N] [--cpus M,SIBLING,CROSS] [--seed N]";
/// Rounds for a reading: fewer are a dry run (the documented dispatch runs 9)
const MIN_ROUNDS: usize = 7;
/// The measured thread's sleep after the barrier, before its first block: the load is
/// running by then
const RAMP: Duration = Duration::from_millis(2);
/// A load alone: its window
const LOAD_ALONE: Duration = Duration::from_millis(300);
/// The chain load's steps a unit
const CHAIN_STEPS: usize = 4096;
/// The ALU load's iterations a unit
const ALU_ITERS: u64 = 1 << 16;
/// The stream loads' buffers, and their bytes a unit
const STREAM_BYTES: usize = 64 << 20;
const STREAM8_BYTES: usize = 8 << 20;
const STREAM_CHUNK: usize = 1 << 20;
/// The pause load's pauses a unit
const PAUSES: u64 = 4096;
/// The store load's buffer (16 KiB of words) and its passes over it a unit (256 KiB)
const L1_WORDS: usize = 2048;
const L1_PASSES: usize = 16;
/// The L2-resident chase's table: 256 KiB of entries
const L2_ENTRIES: usize = 1 << 16;
/// The chases' lanes, and each lane's steps a unit
const LANES: usize = 32;
const CHASE_STEPS: usize = 1024;
/// The duty-cycled ALU load: its burst, its sleep, and the iterations between clock reads
const DUTY_ON: Duration = Duration::from_micros(20);
const DUTY_OFF: Duration = Duration::from_micros(20);
const DUTY_CHUNK: u64 = 1024;
/// A load unit longer than its work at the median time a unit of work by more than this
/// (ns) is counted as long
const LONG_UNIT_NS: f64 = 1e6;
/// An idle-by-design CPU busier than this share of a condition is flagged
const IDLE_BUSY: f64 = 0.01;
/// Fixed work: the load block's stage this many times, and whole decodes of this many blocks
const FIXED_REPS: usize = 96;
const FIXED_DECODE_BLOCKS: usize = 64;
/// The anchor: walk|decode at the sibling, the other core busy, within this of phases'
/// 4-copy factor on the model
const ANCHOR_BAND: f64 = 0.15;
/// The phases 4-copy walk factors by model (a substring of /proc/cpuinfo's model name)
const ANCHORS: [(&str, f64, &str); 4] = [
    ("EPYC 7763", 2.12, "phases 42011789918, 4 copies"),
    ("EPYC 9V74", 2.22, "phases 42020095581, 4 copies"),
    ("EPYC 9V45", 2.23, "run 41989592674, 4 copies"),
    ("8370C", 1.61, "phases 42008210159, 4 copies, the walk as one"),
];

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
    /// The sibling loads run again with the other core busy
    fn busy_loads(self) -> &'static [Load] {
        match self {
            Kernel::Walk => &[Load::Walk, Load::Mtf, Load::Build, Load::Decode],
            Kernel::Mtf => &[Load::Walk, Load::Mtf, Load::Decode],
            Kernel::Build => &[Load::Walk, Load::Build, Load::Decode],
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
    Decode,
    Pause,
    L1Store,
    Chase32L2,
    Chase32L3,
    Stream8,
    AluDuty,
    Together,
}

const LOADS: [Load; 14] = [
    Load::Walk,
    Load::Chain,
    Load::Alu,
    Load::Mtf,
    Load::Build,
    Load::Stream,
    Load::Decode,
    Load::Pause,
    Load::L1Store,
    Load::Chase32L2,
    Load::Chase32L3,
    Load::Stream8,
    Load::AluDuty,
    Load::Together,
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
            Load::Decode => "decode",
            Load::Pause => "pause",
            Load::L1Store => "l1store",
            Load::Chase32L2 => "chase32l2",
            Load::Chase32L3 => "chase32l3",
            Load::Stream8 => "stream8",
            Load::AluDuty => "aluduty",
            Load::Together => "together",
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
            Load::Decode => "whole decodes of the blocks, a block a unit",
            Load::Pause => "PAUSE (spin_loop), 4096 a unit",
            Load::L1Store => "4 store streams over 16 KiB, no loads, 256 KiB a unit",
            Load::Chase32L2 => "32 chains, loads only, over a 256 KiB single cycle",
            Load::Chase32L3 => "32 chains, loads only, over the load block's 3.6 MB table",
            Load::Stream8 => "a sequential read of an 8 MB buffer, 1 MiB a unit",
            Load::AluDuty => "the ALU loop, 20 us bursts with 20 us sleeps",
            Load::Together => "decodes in step with the measured one, each walk started together",
        }
    }
    /// What its work counts
    fn unit(self) -> &'static str {
        match self {
            Load::Alu | Load::AluDuty => "iteration",
            Load::Pause => "pause",
            Load::Chase32L2 | Load::Chase32L3 => "load",
            _ => "byte",
        }
    }
    /// A rate (work a ns) as the tables show it
    fn show_rate(self, rate: f64) -> String {
        if !rate.is_finite() {
            return "n/a".into();
        }
        match self.unit() {
            "iteration" => format!("{:.1} ns/kit", 1e3 / rate),
            "pause" => format!("{:.2} ns/pause", 1.0 / rate),
            "load" => format!("{:.3} ns/load", 1.0 / rate),
            _ => format!("{:.3} ns/B", 1.0 / rate),
        }
    }
    /// Beside every stage; else beside the walk only
    fn every_stage(self) -> bool {
        matches!(
            self,
            Load::Walk
                | Load::Chain
                | Load::Alu
                | Load::Mtf
                | Load::Build
                | Load::Stream
                | Load::Decode
        )
    }
    /// Measured alone on S for its speed's baseline (the together load's work is tied to
    /// the measured decode, so it has none)
    fn has_alone(self) -> bool {
        self != Load::Together
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

/// A measured stage beside a load (both: on M and on the load's CPU), a stage alone (no
/// load), or a load alone (no stage: on S with M idle); `busy`: the other core's CPUs run
/// unmeasured decodes; `again`: a stage alone's second run, at its group's end
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Condition {
    kernel: Option<Kernel>,
    load: Option<Load>,
    place: Place,
    busy: bool,
    again: bool,
}

impl Condition {
    fn new(kernel: Option<Kernel>, load: Option<Load>, place: Place, busy: bool) -> Self {
        Condition {
            kernel,
            load,
            place,
            busy,
            again: false,
        }
    }
    fn id(&self) -> String {
        format!(
            "{}|{}|{}{}{}",
            self.kernel.map_or("-", Kernel::name),
            self.load.map_or("none", Load::name),
            self.place.name(),
            if self.busy { "+busy" } else { "" },
            if self.again { "#2" } else { "" }
        )
    }
    fn where_(&self) -> String {
        format!(
            "{}{}{}",
            self.place.name(),
            if self.busy { ", other core busy" } else { "" },
            if self.again { " (again, last)" } else { "" }
        )
    }
}

/// Every condition a round: for each stage, alone, beside each of its loads on the sibling
/// and on the other core (where each exists), with the other core busy (where it exists),
/// and alone again; then each load alone
fn conditions(sibling: bool, cross: bool, busy: bool) -> Vec<Condition> {
    let mut v = Vec::new();
    for k in KERNELS {
        v.push(Condition::new(Some(k), None, Place::Alone, false));
        for l in LOADS {
            if !(l.every_stage() || k == Kernel::Walk) {
                continue;
            }
            for (on, place) in [(sibling, Place::Sibling), (cross, Place::Cross)] {
                if on {
                    v.push(Condition::new(Some(k), Some(l), place, false));
                }
            }
        }
        if busy {
            v.push(Condition::new(Some(k), None, Place::Alone, true));
            if sibling {
                for &l in k.busy_loads() {
                    v.push(Condition::new(Some(k), Some(l), Place::Sibling, true));
                }
            }
        }
        v.push(Condition {
            again: true,
            ..Condition::new(Some(k), None, Place::Alone, false)
        });
    }
    for l in LOADS {
        if l.has_alone() {
            v.push(Condition::new(None, Some(l), Place::Alone, false));
        }
    }
    v
}

fn find(
    conds: &[Condition],
    kernel: Option<Kernel>,
    load: Option<Load>,
    place: Place,
    busy: bool,
) -> Option<usize> {
    conds.iter().position(|c| {
        c.kernel == kernel && c.load == load && c.place == place && c.busy == busy && !c.again
    })
}

/// A stage's two alones (first and last in its group)
fn alones(conds: &[Condition], k: Kernel) -> (Option<usize>, Option<usize>) {
    (
        find(conds, Some(k), None, Place::Alone, false),
        conds
            .iter()
            .position(|c| c.kernel == Some(k) && c.load.is_none() && c.again),
    )
}

/// The fixed-work kinds: a stage of the load block FIXED_REPS times, or whole decodes of a
/// block subset
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Work {
    Walk,
    Mtf,
    Build,
    Decode,
}

const WORKS: [Work; 4] = [Work::Walk, Work::Mtf, Work::Build, Work::Decode];

impl Work {
    fn name(self) -> &'static str {
        match self {
            Work::Walk => "walk",
            Work::Mtf => "mtf",
            Work::Build => "build",
            Work::Decode => "decode",
        }
    }
}

/// Fixed work on M, and on S where given, released together
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Fixed {
    m: Work,
    s: Option<Work>,
}

impl Fixed {
    fn id(&self) -> String {
        format!(
            "fixed:{}|{}",
            self.m.name(),
            self.s.map_or("none", Work::name)
        )
    }
}

/// Each work alone on M, then (with a sibling) the pairs
fn fixed_list(sibling: bool) -> Vec<Fixed> {
    let mut v: Vec<Fixed> = WORKS.iter().map(|&m| Fixed { m, s: None }).collect();
    if sibling {
        for (m, s) in [
            (Work::Walk, Work::Walk),
            (Work::Walk, Work::Mtf),
            (Work::Walk, Work::Build),
            (Work::Mtf, Work::Mtf),
            (Work::Build, Work::Build),
            (Work::Decode, Work::Decode),
        ] {
            v.push(Fixed { m, s: Some(s) });
        }
    }
    v
}

fn find_fixed(fixed: &[Fixed], m: Work, s: Option<Work>) -> Option<usize> {
    fixed.iter().position(|f| f.m == m && f.s == s)
}

/// splitmix64
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// Below `n` (n > 0)
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
}

/// A round's step: a condition, or a fixed-work run
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Cond(usize),
    Fixed(usize),
}

/// A round's order: the groups (each stage's conditions, the loads alone, the fixed work)
/// in a shuffled order, each shuffled within, a stage's group between its two alones
fn order(conds: &[Condition], nfixed: usize, rng: &mut Rng) -> Vec<Slot> {
    let mut groups: Vec<Vec<Slot>> = Vec::new();
    for k in KERNELS {
        let (first, last) = alones(conds, k);
        let mut mid: Vec<Slot> = (0..conds.len())
            .filter(|&i| conds[i].kernel == Some(k) && Some(i) != first && Some(i) != last)
            .map(Slot::Cond)
            .collect();
        shuffle(&mut mid, rng);
        let mut g: Vec<Slot> = first.map(Slot::Cond).into_iter().collect();
        g.extend(mid);
        g.extend(last.map(Slot::Cond));
        groups.push(g);
    }
    let mut alone: Vec<Slot> = (0..conds.len())
        .filter(|&i| conds[i].kernel.is_none())
        .map(Slot::Cond)
        .collect();
    shuffle(&mut alone, rng);
    groups.push(alone);
    let mut fixed: Vec<Slot> = (0..nfixed).map(Slot::Fixed).collect();
    shuffle(&mut fixed, rng);
    groups.push(fixed);
    groups.retain(|g| !g.is_empty());
    shuffle(&mut groups, rng);
    groups.into_iter().flatten().collect()
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

/// The CPUs a round uses: the measured stage's, its SMT sibling's and a CPU of another
/// core's (each where it exists), a CPU none of them uses for the main thread, and the
/// other core's CPUs (at most two) for the background decodes
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cpus {
    measured: usize,
    sibling: Option<usize>,
    cross: Option<usize>,
    spare: Option<usize>,
    busy: [Option<usize>; 2],
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
    fn busy_cpus(&self) -> Vec<usize> {
        self.busy.iter().flatten().copied().collect()
    }
    fn all(&self) -> Vec<usize> {
        let mut v: Vec<usize> = [Some(self.measured), self.sibling, self.cross]
            .into_iter()
            .flatten()
            .chain(self.busy_cpus())
            .collect();
        v.sort();
        v.dedup();
        v
    }
    fn show(&self) -> String {
        let c = |x: Option<usize>| x.map_or("-".to_string(), |c| c.to_string());
        format!(
            "M {}, S {}, X {}, busy {}, main {}",
            self.measured,
            c(self.sibling),
            c(self.cross),
            if self.busy_cpus().is_empty() {
                "-".to_string()
            } else {
                self.busy_cpus()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            },
            c(self.spare)
        )
    }
}

fn spare(cores: &[Vec<usize>], used: [Option<usize>; 3]) -> Option<usize> {
    cores
        .iter()
        .flatten()
        .copied()
        .find(|&c| !used.contains(&Some(c)))
}

/// The background decodes' CPUs: the cross CPU's core's, at most two
fn busy_of(cores: &[Vec<usize>], cross: Option<usize>) -> [Option<usize>; 2] {
    let core = cross.and_then(|x| cores.iter().find(|k| k.contains(&x)));
    match core {
        Some(k) => [k.first().copied(), k.get(1).copied()],
        None => [None, None],
    }
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
        busy: busy_of(cores, cross),
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
        busy: busy_of(cores, cross),
    })
}

/// The odd rounds' CPUs: the measured core and the cross core swapped (CI: M 2, S 3, X 0),
/// checked as `--cpus` is; None where the other core has no second CPU, or the swap would
/// leave the main thread without a CPU of its own where it had one
fn swapped(cores: &[Vec<usize>], c: &Cpus) -> Option<Cpus> {
    c.sibling?;
    let x = c.cross?;
    let other = cores.iter().find(|k| k.contains(&x))?;
    if other.len() < 2 {
        return None;
    }
    let s = parse_cpus(&format!("{},{},{}", other[0], other[1], c.measured), cores).ok()?;
    (s.spare.is_some() == c.spare.is_some()).then_some(s)
}

extern "C" {
    fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
    fn sched_getcpu() -> i32;
    fn prctl(option: i32, ...) -> i32;
}

/// prctl's PR_SET_TIMERSLACK
const PR_SET_TIMERSLACK: i32 = 29;

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

/// The calling thread's sleeps end as near their time as the timer allows (timer slack
/// 1 ns, not the default 50 us), for the duty-cycled load's short idles
fn fine_timer_slack() {
    // SAFETY: PR_SET_TIMERSLACK takes one unsigned long and changes only this thread's slack
    let _ = unsafe { prctl(PR_SET_TIMERSLACK, 1u64) };
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

// ---- Each CPU's use around a condition ----

/// /proc/stat's per-CPU lines: (cpu, [busy, steal, total]) jiffies, busy being user, nice,
/// system, irq and softirq, total every field to steal
fn parse_stat(s: &str) -> Vec<(usize, [u64; 3])> {
    s.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("cpu")?;
            let (id, fields) = rest.split_once(' ')?;
            let cpu: usize = id.parse().ok()?;
            let f: Vec<u64> = fields
                .split_whitespace()
                .map(|x| x.parse().unwrap_or(0))
                .collect();
            let g = |i: usize| f.get(i).copied().unwrap_or(0);
            // user nice system idle iowait irq softirq steal
            let busy = g(0) + g(1) + g(2) + g(5) + g(6);
            Some((cpu, [busy, g(7), (0..8).map(g).sum()]))
        })
        .collect()
}

/// /proc/interrupts summed over its rows, by CPU: (cpu, count); rows with fewer counts
/// than CPUs (ERR, MIS) left out
fn parse_interrupts(s: &str) -> Vec<(usize, u64)> {
    let mut lines = s.lines();
    let Some(head) = lines.next() else {
        return Vec::new();
    };
    let cpus: Vec<usize> = head
        .split_whitespace()
        .filter_map(|t| t.strip_prefix("CPU")?.parse().ok())
        .collect();
    let mut sum = vec![0u64; cpus.len()];
    for l in lines {
        let Some((_, rest)) = l.split_once(':') else {
            continue;
        };
        let counts: Vec<u64> = rest
            .split_whitespace()
            .take(cpus.len())
            .map_while(|t| t.parse().ok())
            .collect();
        if counts.len() == cpus.len() {
            for (s, c) in sum.iter_mut().zip(counts) {
                *s += c;
            }
        }
    }
    cpus.into_iter().zip(sum).collect()
}

#[derive(Clone, Debug, Default)]
struct Snap {
    stat: Vec<(usize, [u64; 3])>,
    irq: Vec<(usize, u64)>,
}

fn snap() -> Snap {
    let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
    Snap {
        stat: parse_stat(&read("/proc/stat")),
        irq: parse_interrupts(&read("/proc/interrupts")),
    }
}

/// A CPU's use over a condition
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct CpuUse {
    cpu: usize,
    busy: u64,
    steal: u64,
    total: u64,
    irqs: u64,
}

/// Each CPU's use between two snapshots, for the CPUs in both (`cpus`, where given, only)
fn used(a: &Snap, b: &Snap, cpus: &[usize]) -> Vec<CpuUse> {
    b.stat
        .iter()
        .filter(|(c, _)| cpus.is_empty() || cpus.contains(c))
        .filter_map(|&(cpu, y)| {
            let x = a.stat.iter().find(|(c, _)| *c == cpu)?.1;
            let irq = |s: &Snap| s.irq.iter().find(|(c, _)| *c == cpu).map(|x| x.1);
            Some(CpuUse {
                cpu,
                busy: y[0].saturating_sub(x[0]),
                steal: y[1].saturating_sub(x[1]),
                total: y[2].saturating_sub(x[2]),
                irqs: match (irq(a), irq(b)) {
                    (Some(p), Some(q)) => q.saturating_sub(p),
                    _ => 0,
                },
            })
        })
        .collect()
}

/// `cpu`'s busy share in `u` (NaN where it isn't there or no time passed)
fn busy_share(u: &[CpuUse], cpu: usize) -> f64 {
    match u.iter().find(|x| x.cpu == cpu) {
        Some(x) if x.total > 0 => x.busy as f64 / x.total as f64,
        _ => f64::NAN,
    }
}

/// The measured core's CPU that is idle by design in a condition: the sibling, for a stage
/// alone or beside a cross-core load; M, for a load alone; none beside a sibling load
fn idle_cpu(c: &Condition, cpus: &Cpus) -> Option<usize> {
    match (c.kernel, c.place) {
        (None, _) => Some(cpus.measured),
        (Some(_), Place::Sibling) => None,
        (Some(_), _) => cpus.sibling,
    }
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

/// A decoding thread's buffers, kept from block to block and condition to condition
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

/// The together load's rendezvous: each side's walks begun so far
#[derive(Default)]
struct Rendezvous {
    m: AtomicU64,
    s: AtomicU64,
}

/// One side of the rendezvous
struct Meet<'a> {
    mine: &'a AtomicU64,
    theirs: &'a AtomicU64,
    stop: &'a AtomicBool,
}

impl Meet<'_> {
    /// Wait (spinning) until the other side has come as many times as this one; false if
    /// `stop` is set first (the other side has ended)
    fn meet(&self) -> bool {
        let k = self.mine.fetch_add(1, Ordering::AcqRel) + 1;
        while self.theirs.load(Ordering::Acquire) < k {
            if self.stop.load(Ordering::Acquire) {
                return false;
            }
            spin_loop();
        }
        true
    }
}

/// One block as `decode_block` runs it with `Segmented` (the walk in place, over `ll`):
/// `kernel`'s stage timed into `windows` (none for a load), the walk preceded by `meet`
/// where given; its CRC checked. Returns its bytes, or None if the rendezvous was abandoned.
fn decode_block(
    b: &Symbols,
    k: Kernels,
    d: &mut Decode,
    kernel: Option<Kernel>,
    epoch: Instant,
    windows: &mut Vec<(u64, u64)>,
    meet: Option<&Meet>,
) -> Option<usize> {
    let len = timed(kernel == Some(Kernel::Mtf), epoch, windows, || {
        mtf::mtf(b, &mut d.ll, &mut d.counts, k).unwrap()
    });
    assert!((b.orig_ptr as usize) < len, "a block's orig_ptr");
    timed(kernel == Some(Kernel::Build), epoch, windows, || {
        build(&d.ll, &d.counts, &mut d.tt)
    });
    if let Some(m) = meet {
        if !m.meet() {
            return None;
        }
    }
    timed(kernel == Some(Kernel::Walk), epoch, windows, || {
        walk_segments_room(&mut d.tt, b.orig_ptr, &mut d.ll, &mut d.seg)
    });
    d.out.clear();
    rle::unrle(&d.ll, &mut d.out, k).unwrap();
    assert_eq!(crc::crc32(&d.out, k), b.crc, "a block's CRC");
    Some(len)
}

/// One decode of `blocks`, `kernel`'s stage timed; returns the bytes walked
fn decode(
    blocks: &[Symbols],
    k: Kernels,
    kernel: Kernel,
    d: &mut Decode,
    epoch: Instant,
    windows: &mut Vec<(u64, u64)>,
    meet: Option<&Meet>,
) -> usize {
    blocks
        .iter()
        .map(|b| {
            decode_block(b, k, d, Some(kernel), epoch, windows, meet)
                .expect("the together load ended before the measured decode")
        })
        .sum()
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

/// A single cycle through `n` slots (Sattolo's shuffle, seeded), each entry its successor's
/// slot << 8, as the chain's table holds its links
fn cycle_table(n: usize, seed: u64) -> Vec<u32> {
    assert!(n < 1 << 24, "a cycle's slots fit 24 bits");
    let mut next: Vec<u32> = (0..n as u32).collect();
    let mut rng = Rng(seed);
    for i in (1..n).rev() {
        let j = rng.below(i);
        next.swap(i, j);
    }
    next.into_iter().map(|x| x << 8).collect()
}

/// LANES entries of `t`'s cycle (through slot `from`), spaced len/LANES steps apart
fn lanes_on(t: &[u32], from: usize) -> [u32; LANES] {
    let gap = (t.len() / LANES).max(1);
    let mut p = t[from];
    let mut lanes = [0u32; LANES];
    for l in &mut lanes {
        *l = p;
        p = chain(t, p, gap);
    }
    lanes
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

/// `steps` steps of each of 32 chains through `t` (each entry its successor's slot << 8):
/// table loads only, 32 lines in flight; the lanes' positions are kept in `lanes`
///
/// # Safety
/// Every entry's slot (entry >> 8) is below t.len(), and so is every lane's
#[inline(never)]
unsafe fn chase32(t: &[u32], lanes: &mut [u32; LANES], steps: usize) {
    let mut p = *lanes;
    for _ in 0..steps {
        for x in &mut p {
            // SAFETY: the caller's: each slot read is below t.len()
            *x = unsafe { *t.get_unchecked((*x >> 8) as usize) };
        }
    }
    *lanes = p;
}

/// Whether every entry of `t`, and every lane, names a slot of `t`
fn links_in_bounds(t: &[u32], lanes: &[u32]) -> bool {
    t.iter()
        .chain(lanes)
        .all(|&e| ((e >> 8) as usize) < t.len())
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

/// `passes` passes of four store streams (each over a quarter of `buf`), 8-byte volatile
/// stores of `v` plus the pass, no loads
#[inline(never)]
fn stores(buf: &mut [u64], passes: usize, v: u64) {
    let q = buf.len() / 4;
    let (a, rest) = buf.split_at_mut(q);
    let (b, rest) = rest.split_at_mut(q);
    let (c, d) = rest.split_at_mut(q);
    for pass in 0..passes {
        let v = v.wrapping_add(pass as u64);
        for i in 0..q {
            // SAFETY: each a valid, aligned, exclusive reference into `buf`
            unsafe {
                std::ptr::write_volatile(&mut a[i], v);
                std::ptr::write_volatile(&mut b[i], v);
                std::ptr::write_volatile(&mut c[i], v);
                std::ptr::write_volatile(&mut d[i], v);
            }
        }
    }
}

/// The duty-cycled load's bursts and sleeps over a condition (ns)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Duty {
    on: u64,
    off: u64,
    periods: u64,
}

impl Duty {
    /// The share of its time the load was busy
    fn share(&self) -> f64 {
        if self.on + self.off == 0 {
            f64::NAN
        } else {
            self.on as f64 / (self.on + self.off) as f64
        }
    }
    /// The mean period (us)
    fn period_us(&self) -> f64 {
        if self.periods == 0 {
            f64::NAN
        } else {
            (self.on + self.off) as f64 / self.periods as f64 * 1e-3
        }
    }
}

/// A load thread's inputs and buffers, made once (the stream buffers only where `streams`)
struct Loads<'a> {
    k: Kernels,
    block: &'a Symbols,
    blocks: &'a [Symbols],
    n: usize,
    /// the load block's table, for the room walk, and the same in this file's copy, with
    /// the chain's place in it and the L3 chase's lanes
    tt: TVector,
    chain: Vec<u32>,
    p: u32,
    lanes3: [u32; LANES],
    /// the L2-resident cycle and its lanes
    l2: Vec<u32>,
    lanes2: [u32; LANES],
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
    stream8: Vec<u64>,
    at8: usize,
    st: Vec<u64>,
    stv: u64,
    /// the decode load's buffers and its next block
    dec: Decode,
    next: usize,
    duty: Duty,
}

/// A buffer of `bytes` written through, so every page is its own (not the zero page)
fn stream_buffer(bytes: usize) -> Vec<u64> {
    (0..bytes as u64 / 8)
        .map(|i| i.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
        .collect()
}

impl<'a> Loads<'a> {
    /// The loads over `block`, each checked: the room walk's and the chain's bytes against
    /// the plain walk's, every chase link in bounds; the decode load starting at block
    /// `next`
    fn new(
        block: &'a Symbols,
        blocks: &'a [Symbols],
        k: Kernels,
        streams: bool,
        next: usize,
    ) -> Self {
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
        let lanes3 = lanes_on(&t, block.orig_ptr as usize);
        let l2 = cycle_table(L2_ENTRIES, 0x5eed);
        let lanes2 = lanes_on(&l2, 0);
        assert!(links_in_bounds(&t, &lanes3) && links_in_bounds(&l2, &lanes2));
        let mut btt = TVector::new();
        build(&ll, &counts, &mut btt);
        let (stream, stream8) = if streams {
            (stream_buffer(STREAM_BYTES), stream_buffer(STREAM8_BYTES))
        } else {
            (Vec::new(), Vec::new())
        };
        Loads {
            k,
            block,
            blocks,
            n,
            tt,
            p: t[block.orig_ptr as usize],
            chain: t,
            lanes3,
            l2,
            lanes2,
            seg,
            out,
            bll: ll.clone(),
            bcounts: counts,
            ll,
            counts,
            btt,
            stream,
            at: 0,
            stream8,
            at8: 0,
            st: vec![0u64; L1_WORDS],
            stv: 1,
            dec: Decode::new(),
            next: next % blocks.len().max(1),
            duty: Duty::default(),
        }
    }

    /// Block `i` decoded with the decode load's buffers; its bytes
    fn decode_at(&mut self, i: usize) -> u64 {
        let mut none = Vec::new();
        let b = &self.blocks[i];
        decode_block(b, self.k, &mut self.dec, None, Instant::now(), &mut none, None).unwrap()
            as u64
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
            Load::Stream8 => {
                let words = STREAM_CHUNK / 8;
                black_box(sum(&self.stream8[self.at8 * words..(self.at8 + 1) * words]));
                self.at8 = (self.at8 + 1) % (STREAM8_BYTES / STREAM_CHUNK);
                STREAM_CHUNK as u64
            }
            Load::Decode | Load::Together => {
                let i = self.next;
                self.next = (i + 1) % self.blocks.len();
                self.decode_at(i)
            }
            Load::Pause => {
                for _ in 0..PAUSES {
                    spin_loop();
                }
                PAUSES
            }
            Load::L1Store => {
                stores(&mut self.st, L1_PASSES, self.stv);
                self.stv = self.stv.wrapping_add(L1_PASSES as u64);
                (L1_PASSES * L1_WORDS * 8) as u64
            }
            Load::Chase32L2 => {
                // SAFETY: the cycle's links and lanes were checked in bounds (new)
                unsafe { chase32(&self.l2, &mut self.lanes2, CHASE_STEPS) };
                (CHASE_STEPS * LANES) as u64
            }
            Load::Chase32L3 => {
                // SAFETY: the table's links and lanes were checked in bounds (new)
                unsafe { chase32(&self.chain, &mut self.lanes3, CHASE_STEPS) };
                (CHASE_STEPS * LANES) as u64
            }
            Load::AluDuty => {
                let t0 = Instant::now();
                let mut it = 0;
                while t0.elapsed() < DUTY_ON {
                    alu(DUTY_CHUNK);
                    it += DUTY_CHUNK;
                }
                let t1 = Instant::now();
                std::thread::sleep(DUTY_OFF);
                let t2 = Instant::now();
                self.duty.on += (t1 - t0).as_nanos() as u64;
                self.duty.off += (t2 - t1).as_nanos() as u64;
                self.duty.periods += 1;
                it
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
    if load == Load::AluDuty {
        fine_timer_slack();
    }
    st.duty = Duty::default();
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

/// The together load: the measured blocks decoded in step with the measured decode, each
/// walk begun at the rendezvous; then decodes on freely until `stop`, one unit more
fn run_together(
    st: &mut Loads,
    meet: &Meet,
    stop: &AtomicBool,
    epoch: Instant,
    log: &mut Vec<(u64, u64)>,
) {
    log.clear();
    let mut done = 0;
    log.push((ns(epoch), 0));
    let mut none = Vec::new();
    let blocks = st.blocks;
    for b in blocks {
        match decode_block(b, st.k, &mut st.dec, None, epoch, &mut none, Some(meet)) {
            Some(n) => done += n as u64,
            None => return,
        }
        log.push((ns(epoch), done));
    }
    loop {
        let last = stop.load(Ordering::Acquire);
        done += st.unit(Load::Decode);
        log.push((ns(epoch), done));
        if last {
            break;
        }
    }
}

/// A background decoder's buffers and its next block
struct Background {
    d: Decode,
    next: usize,
}

/// Unmeasured decodes, a block at a time, until `stop`
fn run_background(bg: &mut Background, blocks: &[Symbols], k: Kernels, stop: &AtomicBool) {
    let mut none = Vec::new();
    let epoch = Instant::now();
    while !stop.load(Ordering::Acquire) {
        decode_block(&blocks[bg.next], k, &mut bg.d, None, epoch, &mut none, None);
        bg.next = (bg.next + 1) % blocks.len();
    }
}

/// Sets the flag when dropped, so the other threads stop even if this one panics
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

/// `split`'s units more than LONG_UNIT_NS longer than their work takes at the median time a
/// unit of work
fn long_units(units: &[(f64, f64, f64)]) -> u32 {
    let per: Vec<f64> = units
        .iter()
        .filter(|u| u.2 > 0.0)
        .map(|u| (u.0 + u.1) / u.2)
        .collect();
    let m = median(&per);
    if !m.is_finite() {
        return 0;
    }
    units
        .iter()
        .filter(|u| u.0 + u.1 - u.2 * m > LONG_UNIT_NS)
        .count() as u32
}

/// One condition in one round
#[derive(Clone, Debug)]
struct Sample {
    /// the measured stage's time, summed over its windows (s); NaN for a load alone
    stage: f64,
    /// the load's rate inside the windows (work a ns): fitted, and interpolated; NaN with no
    /// load, or where the load's log didn't cover the windows
    fit: f64,
    interp: f64,
    /// the load's units over a ms longer than their work's median time
    long_units: u32,
    /// involuntary context switches during the condition: the measured thread's (or the
    /// timekeeper's), the load's
    m_preempt: u64,
    s_preempt: u64,
    /// each CPU's use over the condition
    cpu: Vec<CpuUse>,
    /// the duty-cycled load's bursts and sleeps
    duty: Duty,
}

impl Default for Sample {
    fn default() -> Self {
        Sample {
            stage: f64::NAN,
            fit: f64::NAN,
            interp: f64::NAN,
            long_units: 0,
            m_preempt: 0,
            s_preempt: 0,
            cpu: Vec::new(),
            duty: Duty::default(),
        }
    }
}

/// One fixed-work run in one round: each side's time from its release to its end, and the
/// makespan from the first release to the last end (s); NaN for a side that did nothing
#[derive(Clone, Debug)]
struct FixedSample {
    m: f64,
    s: f64,
    span: f64,
    m_preempt: u64,
    s_preempt: u64,
    cpu: Vec<CpuUse>,
}

impl Default for FixedSample {
    fn default() -> Self {
        FixedSample {
            m: f64::NAN,
            s: f64::NAN,
            span: f64::NAN,
            m_preempt: 0,
            s_preempt: 0,
            cpu: Vec::new(),
        }
    }
}

/// What a run needs, besides its buffers
struct Ctx<'a> {
    blocks: &'a [Symbols],
    k: Kernels,
    /// the fixed decode's blocks
    subset: Vec<usize>,
}

/// The buffers, kept from condition to condition: the measured decode's, the load's (S),
/// the fixed work's on M, the background decoders', the windows and the load's log
struct State<'a> {
    d: Decode,
    loads: Loads<'a>,
    mine: Loads<'a>,
    background: Vec<Background>,
    windows: Vec<(u64, u64)>,
    log: Vec<(u64, u64)>,
}

/// One condition: the load (if any) pinned to its CPU, the background decoders (if busy)
/// to the other core's, and the measured decode (if any) to M, started behind a barrier;
/// for a load alone, this thread keeps its window
fn run(cond: &Condition, cx: &Ctx, cpus: &Cpus, st: &mut State) -> Sample {
    let stop = AtomicBool::new(false);
    let bg: Vec<usize> = if cond.busy {
        cpus.busy_cpus()
    } else {
        Vec::new()
    };
    let barrier = Barrier::new(1 + usize::from(cond.load.is_some()) + bg.len());
    let rv = Rendezvous::default();
    let State {
        d,
        loads,
        background,
        windows,
        log,
        ..
    } = st;
    windows.clear();
    log.clear();
    let (mut m_preempt, mut s_preempt) = (0, 0);
    let before = snap();
    let epoch = Instant::now();
    std::thread::scope(|s| {
        for (&cpu, b) in bg.iter().zip(background.iter_mut()) {
            let (stop, barrier, blocks, k) = (&stop, &barrier, cx.blocks, cx.k);
            s.spawn(move || {
                pin(cpu);
                let _stop = StopOnDrop(stop);
                barrier.wait();
                run_background(b, blocks, k, stop);
            });
        }
        let load = cond.load.map(|l| {
            let cpu = cpus.of(cond.place).unwrap();
            let (stop, barrier, loads, log, rv) = (&stop, &barrier, &mut *loads, &mut *log, &rv);
            s.spawn(move || {
                pin(cpu);
                let _stop = StopOnDrop(stop);
                let p = preempted();
                barrier.wait();
                if l == Load::Together {
                    let meet = Meet {
                        mine: &rv.s,
                        theirs: &rv.m,
                        stop,
                    };
                    run_together(loads, &meet, stop, epoch, log);
                } else {
                    run_load(l, loads, stop, epoch, log);
                }
                assert_eq!(on_cpu(), cpu as i32, "the load left its CPU");
                preempted() - p
            })
        });
        match cond.kernel {
            Some(kernel) => {
                let (stop, barrier, d, w, rv) = (&stop, &barrier, &mut *d, &mut *windows, &rv);
                let (blocks, k, m) = (cx.blocks, cx.k, cpus.measured);
                let together = cond.load == Some(Load::Together);
                let h = s.spawn(move || {
                    let _stop = StopOnDrop(stop);
                    pin(m);
                    let p = preempted();
                    barrier.wait();
                    std::thread::sleep(RAMP);
                    let meet = together.then_some(Meet {
                        mine: &rv.m,
                        theirs: &rv.s,
                        stop,
                    });
                    decode(blocks, k, kernel, d, epoch, w, meet.as_ref());
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
    let after = snap();
    let mut sample = Sample {
        m_preempt,
        s_preempt,
        cpu: used(&before, &after, &[]),
        ..Sample::default()
    };
    if cond.kernel.is_some() {
        sample.stage = windows.iter().map(|&(s, e)| (e - s) as f64).sum::<f64>() * 1e-9;
    }
    if cond.load.is_some() && covers(log, windows) {
        let units = split(log, windows);
        (sample.fit, sample.interp) = rates(&units);
        sample.long_units = long_units(&units);
    }
    if cond.load == Some(Load::AluDuty) {
        sample.duty = loads.duty;
    }
    sample
}

/// One side's fixed work
fn fixed_work(w: Work, st: &mut Loads, subset: &[usize]) {
    match w {
        Work::Walk | Work::Mtf | Work::Build => {
            let load = match w {
                Work::Walk => Load::Walk,
                Work::Mtf => Load::Mtf,
                _ => Load::Build,
            };
            for _ in 0..FIXED_REPS {
                st.unit(load);
            }
        }
        Work::Decode => {
            for &i in subset {
                st.decode_at(i);
            }
        }
    }
}

/// One side of a fixed-work run, pinned and released by the barrier: (start, end) ns since
/// `epoch`, and its involuntary context switches
fn fixed_side(
    cpu: usize,
    w: Work,
    st: &mut Loads,
    subset: &[usize],
    barrier: &Barrier,
    epoch: Instant,
) -> (u64, u64, u64) {
    pin(cpu);
    let p = preempted();
    barrier.wait();
    let a = ns(epoch);
    fixed_work(w, st, subset);
    let b = ns(epoch);
    assert_eq!(on_cpu(), cpu as i32, "a fixed-work thread left its CPU");
    (a, b, preempted() - p)
}

/// One fixed-work run: M's work on M (the fixed side's buffers), S's on S (the load's)
fn run_fixed(f: &Fixed, cx: &Ctx, cpus: &Cpus, st: &mut State) -> FixedSample {
    let barrier = Barrier::new(1 + usize::from(f.s.is_some()));
    let State { loads, mine, .. } = st;
    let subset = &cx.subset[..];
    let before = snap();
    let epoch = Instant::now();
    let (m, s) = std::thread::scope(|sc| {
        let barrier = &barrier;
        let hs = f.s.map(|w| {
            let cpu = cpus.sibling.unwrap();
            let loads = &mut *loads;
            sc.spawn(move || fixed_side(cpu, w, loads, subset, barrier, epoch))
        });
        let (cpu, w, mine) = (cpus.measured, f.m, &mut *mine);
        let hm = sc.spawn(move || fixed_side(cpu, w, mine, subset, barrier, epoch));
        (hm.join().unwrap(), hs.map(|h| h.join().unwrap()))
    });
    let after = snap();
    let secs = |a: u64, b: u64| (b - a) as f64 * 1e-9;
    let (lo, hi) = match s {
        Some(s) => (m.0.min(s.0), m.1.max(s.1)),
        None => (m.0, m.1),
    };
    FixedSample {
        m: secs(m.0, m.1),
        s: s.map_or(f64::NAN, |s| secs(s.0, s.1)),
        span: secs(lo, hi),
        m_preempt: m.2,
        s_preempt: s.map_or(0, |s| s.2),
        cpu: used(&before, &after, &[]),
    }
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

fn least(xs: &[f64]) -> f64 {
    finite(xs.iter().copied())
        .into_iter()
        .fold(f64::NAN, f64::min)
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
fn geo(xs: &[f64]) -> Ci {
    let logs: Vec<f64> = xs.iter().filter(|&&x| x > 0.0).map(|x| x.ln()).collect();
    let (m, lo, hi) = mean_ci(&logs);
    (m.exp(), lo.exp(), hi.exp())
}

/// A stage's alone by round: the mean of its group's two alones (either where the other is
/// missing)
fn alone_by_round(conds: &[Condition], samples: &[Vec<Sample>], k: Kernel) -> Vec<f64> {
    let (a, b) = alones(conds, k);
    samples
        .iter()
        .map(|r| {
            let v = finite([a, b].into_iter().flatten().map(|i| r[i].stage));
            if v.is_empty() {
                f64::NAN
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        })
        .collect()
}

/// A stage's ratio to its alone by round, at a condition
fn ratio_by_round(conds: &[Condition], samples: &[Vec<Sample>], ci: usize) -> Vec<f64> {
    match conds[ci].kernel {
        Some(k) => alone_by_round(conds, samples, k)
            .iter()
            .zip(samples)
            .map(|(a, r)| r[ci].stage / a)
            .collect(),
        None => vec![f64::NAN; samples.len()],
    }
}

/// Whether a round's condition was preempted: its threads, or (for a stage) its alones'
fn round_preempted(conds: &[Condition], round: &[Sample], ci: usize) -> bool {
    let p = |i: usize| round[i].m_preempt + round[i].s_preempt > 0;
    p(ci)
        || conds[ci].kernel.is_some_and(|k| {
            let (a, b) = alones(conds, k);
            a.is_some_and(p) || b.is_some_and(p)
        })
}

/// The pairing factors, each a stage's ratio to its alone at the sibling: the walk beside
/// the walk (ww), beside X (wx), X beside the walk (xw), X beside X (xx), and each beside a
/// decoding sibling, today's mix (wd, xd)
#[derive(Clone, Copy, Debug, PartialEq)]
struct Factors {
    ww: f64,
    wx: f64,
    xw: f64,
    xx: f64,
    wd: f64,
    xd: f64,
}

/// The core's time for both siblings' walk (alone time `w` each) and stage X (`x` each):
/// (mixed, same-type, today's mix). Mixed: one sibling walks while the other runs X (at
/// `wx` and `xw`) until one kind runs out, the rest beside its own kind. Same-type: walk
/// beside walk (`ww`), then X beside X (`xx`). Today's mix: each sibling's walk and X
/// beside a decoding sibling (`wd`, `xd`), the two siblings in parallel.
fn pairing(w: f64, x: f64, f: &Factors) -> (f64, f64, f64) {
    let tau = (2.0 * w * f.wx).min(2.0 * x * f.xw);
    let rest_w = (2.0 * w - tau / f.wx).max(0.0);
    let rest_x = (2.0 * x - tau / f.xw).max(0.0);
    let mixed = tau + rest_w * f.ww / 2.0 + rest_x * f.xx / 2.0;
    (mixed, w * f.ww + x * f.xx, w * f.wd + x * f.xd)
}

/// The pairing model by round for the walk with stage `x` at the sibling, the other core
/// idle or `busy`: mixed over today's mix, mixed over same-type, and the six factors'
/// medians (ww, wx, xw, xx, wd, xd)
fn pairing_of(
    conds: &[Condition],
    samples: &[Vec<Sample>],
    x: Kernel,
    busy: bool,
) -> Option<(Ci, Ci, [f64; 6])> {
    let lx = if x == Kernel::Mtf {
        Load::Mtf
    } else {
        Load::Build
    };
    let r = |k: Kernel, l: Load| {
        find(conds, Some(k), Some(l), Place::Sibling, busy).map(|i| ratio_by_round(conds, samples, i))
    };
    let f = [
        r(Kernel::Walk, Load::Walk)?,
        r(Kernel::Walk, lx)?,
        r(x, Load::Walk)?,
        r(x, lx)?,
        r(Kernel::Walk, Load::Decode)?,
        r(x, Load::Decode)?,
    ];
    let (aw, ax) = (
        alone_by_round(conds, samples, Kernel::Walk),
        alone_by_round(conds, samples, x),
    );
    let (mut today, mut same) = (Vec::new(), Vec::new());
    for i in 0..samples.len() {
        let fs = Factors {
            ww: f[0][i],
            wx: f[1][i],
            xw: f[2][i],
            xx: f[3][i],
            wd: f[4][i],
            xd: f[5][i],
        };
        let (m, s, t) = pairing(aw[i], ax[i], &fs);
        today.push(m / t);
        same.push(m / s);
    }
    Some((geo(&today), geo(&same), f.map(|v| median(&v))))
}

/// A condition's figures over the rounds
#[derive(Clone, Debug)]
struct Row {
    cond: Condition,
    /// the stage's median c/B (at the reported clock) and ns/B
    cb: f64,
    nsb: f64,
    /// the stage's time over its time alone (the mean of its round's two): paired by round
    /// (geometric mean), median over median, fastest over fastest, and paired over the
    /// rounds not preempted (and how many)
    ratio: Ci,
    ratio_median: f64,
    ratio_fastest: f64,
    ratio_clean: Ci,
    clean_rounds: usize,
    /// the load's median rate inside the windows (work a ns: fitted), and interpolated
    rate: f64,
    rate_interp: f64,
    /// the load's rate over its rate alone, paired by round
    speed: Ci,
    /// the stage's speed plus the load's, by round
    pair_yield: Ci,
    /// the load's long units, and the context switches, summed over the rounds
    long_units: u32,
    m_preempt: u64,
    s_preempt: u64,
    /// rounds whose idle-by-design CPU was over 1% busy, and its largest share
    idle_flagged: usize,
    idle_max: f64,
    /// the duty-cycled load's busy share and period (us), median round
    duty: f64,
    period_us: f64,
}

/// Each condition's figures, from samples[round][condition] and each round's CPUs: `n`
/// bytes walked a decode
fn summarise(
    conds: &[Condition],
    samples: &[Vec<Sample>],
    placements: &[Cpus],
    n: usize,
    ghz: f64,
) -> Vec<Row> {
    let rounds = samples.len();
    let per = |ci: usize, f: &dyn Fn(&Sample) -> f64| -> Vec<f64> {
        samples.iter().map(|round| f(&round[ci])).collect()
    };
    conds
        .iter()
        .enumerate()
        .map(|(ci, c)| {
            let stage = per(ci, &|s| s.stage);
            let ratio = ratio_by_round(conds, samples, ci);
            let alone = c.kernel.map(|k| alone_by_round(conds, samples, k));
            let (ratio_median, ratio_fastest) = match &alone {
                Some(a) => (median(&stage) / median(a), least(&stage) / least(a)),
                None => (f64::NAN, f64::NAN),
            };
            let clean: Vec<f64> = (0..rounds)
                .filter(|&r| !round_preempted(conds, &samples[r], ci))
                .map(|r| ratio[r])
                .collect();
            let rate = per(ci, &|s| if c.kernel.is_some() { s.fit } else { s.interp });
            let speed: Vec<f64> = match c
                .load
                .filter(|l| l.has_alone())
                .and_then(|l| find(conds, None, Some(l), Place::Alone, false))
            {
                Some(a) => rate
                    .iter()
                    .zip(per(a, &|s| s.interp))
                    .map(|(x, a)| x / a)
                    .collect(),
                None => vec![f64::NAN; rounds],
            };
            let pair_yield: Vec<f64> = match (c.kernel, c.load) {
                (Some(_), Some(_)) => ratio.iter().zip(&speed).map(|(r, s)| 1.0 / r + s).collect(),
                (Some(_), None) => ratio.iter().map(|r| 1.0 / r).collect(),
                _ => vec![f64::NAN; rounds],
            };
            let idle: Vec<f64> = (0..rounds)
                .map(|r| {
                    idle_cpu(c, &placements[r])
                        .map_or(f64::NAN, |cpu| busy_share(&samples[r][ci].cpu, cpu))
                })
                .collect();
            Row {
                cond: *c,
                cb: median(&stage) * ghz * 1e9 / n as f64,
                nsb: median(&stage) * 1e9 / n as f64,
                ratio: geo(&ratio),
                ratio_median,
                ratio_fastest,
                clean_rounds: finite(clean.iter().copied()).len(),
                ratio_clean: geo(&clean),
                rate: median(&rate),
                rate_interp: median(&per(ci, &|s| s.interp)),
                speed: geo(&speed),
                pair_yield: mean_ci(&pair_yield),
                long_units: samples.iter().map(|r| r[ci].long_units).sum(),
                m_preempt: samples.iter().map(|r| r[ci].m_preempt).sum(),
                s_preempt: samples.iter().map(|r| r[ci].s_preempt).sum(),
                idle_flagged: idle.iter().filter(|&&x| x > IDLE_BUSY).count(),
                idle_max: finite(idle.iter().copied())
                    .into_iter()
                    .fold(f64::NAN, f64::max),
                duty: median(&per(ci, &|s| s.duty.share())),
                period_us: median(&per(ci, &|s| s.duty.period_us())),
            }
        })
        .collect()
}

/// A stage's ratio to alone at condition `a` over its ratio at `b`, by round; and the share
/// of `b`'s excess over alone that `a` has
fn contrast(conds: &[Condition], samples: &[Vec<Sample>], a: usize, b: usize) -> (Ci, Ci) {
    let (ra, rb) = (
        ratio_by_round(conds, samples, a),
        ratio_by_round(conds, samples, b),
    );
    let ratio: Vec<f64> = ra.iter().zip(&rb).map(|(x, y)| x / y).collect();
    let share: Vec<f64> = ra
        .iter()
        .zip(&rb)
        .map(|(x, y)| (x - 1.0) / (y - 1.0))
        .collect();
    (geo(&ratio), mean_ci(&share))
}

/// Whether a ratio's 95% interval excludes 1 (its excess over alone excludes 0)
fn resolved(r: Ci) -> bool {
    r.1 > 1.0 || r.2 < 1.0
}

/// The anchor: walk|decode at the sibling with the other core busy, against phases' 4-copy
/// factor on this model
#[derive(Clone, Debug)]
struct Anchor {
    ran: bool,
    fastest: f64,
    geo: Ci,
    reference: Option<(f64, &'static str)>,
    pass: Option<bool>,
}

fn anchor_ref(model: &str) -> Option<(f64, &'static str)> {
    ANCHORS
        .iter()
        .find(|a| model.contains(a.0))
        .map(|a| (a.1, a.2))
}

fn anchor(conds: &[Condition], rows: &[Row], model: &str) -> Anchor {
    let i = find(
        conds,
        Some(Kernel::Walk),
        Some(Load::Decode),
        Place::Sibling,
        true,
    );
    let reference = anchor_ref(model);
    match i {
        Some(i) => {
            let f = rows[i].ratio_fastest;
            Anchor {
                ran: true,
                fastest: f,
                geo: rows[i].ratio,
                reference,
                pass: reference.map(|(r, _)| (f - r).abs() <= ANCHOR_BAND),
            }
        }
        None => Anchor {
            ran: false,
            fastest: f64::NAN,
            geo: (f64::NAN, f64::NAN, f64::NAN),
            reference,
            pass: None,
        },
    }
}

/// A fixed-work run's figures over the rounds
#[derive(Clone, Debug)]
struct FixedRow {
    fixed: Fixed,
    /// median times (s): M's side, S's, the makespan
    m: f64,
    s: f64,
    span: f64,
    /// the share of the makespan one side ran alone (median round)
    tail: f64,
    /// the two sides' times alone over the makespan, by round
    pair_yield: Ci,
    m_preempt: u64,
    s_preempt: u64,
    idle_flagged: usize,
}

/// A work's time alone on M, by round
fn fixed_alone(fixed: &[Fixed], fs: &[Vec<FixedSample>], w: Work) -> Vec<f64> {
    match find_fixed(fixed, w, None) {
        Some(i) => fs.iter().map(|r| r[i].m).collect(),
        None => vec![f64::NAN; fs.len()],
    }
}

fn summarise_fixed(
    fixed: &[Fixed],
    fs: &[Vec<FixedSample>],
    placements: &[Cpus],
) -> Vec<FixedRow> {
    fixed
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let col = |g: &dyn Fn(&FixedSample) -> f64| -> Vec<f64> {
                fs.iter().map(|r| g(&r[i])).collect()
            };
            let am = fixed_alone(fixed, fs, f.m);
            let yld: Vec<f64> = (0..fs.len())
                .map(|r| {
                    let alone = am[r]
                        + f.s
                            .map_or(0.0, |w| fixed_alone(fixed, fs, w)[r]);
                    alone / fs[r][i].span
                })
                .collect();
            let tail = col(&|x| (x.span - x.m.min(x.s)) / x.span);
            let idle = (0..fs.len())
                .filter(|&r| {
                    f.s.is_none()
                        && placements[r]
                            .sibling
                            .is_some_and(|c| busy_share(&fs[r][i].cpu, c) > IDLE_BUSY)
                })
                .count();
            FixedRow {
                fixed: *f,
                m: median(&col(&|x| x.m)),
                s: median(&col(&|x| x.s)),
                span: median(&col(&|x| x.span)),
                tail: if f.s.is_some() {
                    median(&tail)
                } else {
                    f64::NAN
                },
                pair_yield: mean_ci(&yld),
                m_preempt: fs.iter().map(|r| r[i].m_preempt).sum(),
                s_preempt: fs.iter().map(|r| r[i].s_preempt).sum(),
                idle_flagged: idle,
            }
        })
        .collect()
}

/// The measured pairing: twice walk|x's makespan over walk|walk's plus x|x's, by round (the
/// core's time for 2 walks and 2 x's mixed against same-type, the tails included)
fn fixed_pairing(fixed: &[Fixed], fs: &[Vec<FixedSample>], x: Work) -> Option<Ci> {
    let wx = find_fixed(fixed, Work::Walk, Some(x))?;
    let ww = find_fixed(fixed, Work::Walk, Some(Work::Walk))?;
    let xx = find_fixed(fixed, x, Some(x))?;
    let v: Vec<f64> = fs
        .iter()
        .map(|r| 2.0 * r[wx].span / (r[ww].span + r[xx].span))
        .collect();
    Some(geo(&v))
}

fn ci3(x: Ci, digits: usize) -> String {
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

fn num(x: f64, digits: usize) -> String {
    if x.is_finite() {
        format!("{x:.digits$}")
    } else {
        "n/a".into()
    }
}

fn pct3(x: Ci) -> String {
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

/// The runner, as the JSON's head names it
#[derive(Clone, Debug, Default)]
struct Env {
    model: String,
    kernel: String,
    cmdline: String,
    cpuidle_driver: String,
    cpuidle_governor: String,
    thp_enabled: String,
    thp_defrag: String,
    run_id: Option<String>,
    run_attempt: Option<String>,
    sha: Option<String>,
}

fn read_env() -> Env {
    let read = |p: &str| {
        std::fs::read_to_string(p)
            .map(|s| s.trim().to_string())
            .ok()
    };
    let or = |x: Option<String>| x.unwrap_or_else(|| "unreadable".into());
    let idle = "/sys/devices/system/cpu/cpuidle";
    Env {
        model: read("/proc/cpuinfo")
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.split_once(':'))
                    .map(|(_, v)| v.trim().to_string())
            })
            .unwrap_or_else(|| "unknown".into()),
        kernel: or(read("/proc/sys/kernel/osrelease")),
        cmdline: or(read("/proc/cmdline")),
        cpuidle_driver: or(read(&format!("{idle}/current_driver"))),
        cpuidle_governor: or(read(&format!("{idle}/current_governor"))
            .or_else(|| read(&format!("{idle}/current_governor_ro")))),
        thp_enabled: or(read("/sys/kernel/mm/transparent_hugepage/enabled")),
        thp_defrag: or(read("/sys/kernel/mm/transparent_hugepage/defrag")),
        run_id: std::env::var("GITHUB_RUN_ID").ok(),
        run_attempt: std::env::var("GITHUB_RUN_ATTEMPT").ok(),
        sha: std::env::var("GITHUB_SHA").ok(),
    }
}

/// What a run was, for the report's head and the JSON's
struct Head<'a> {
    path: &'a str,
    blocks: usize,
    n: usize,
    ghz: f64,
    rounds: usize,
    seed: u64,
    topo: &'a Topology,
    placements: &'a [Cpus],
    swap: bool,
    load_block: (usize, usize),
    subset: usize,
    round_s: &'a [f64],
    env: &'a Env,
}

/// Everything summarised, for the report and the JSON
struct Summary<'a> {
    conds: &'a [Condition],
    samples: &'a [Vec<Sample>],
    rows: Vec<Row>,
    fixed: &'a [Fixed],
    fsamples: &'a [Vec<FixedSample>],
    frows: Vec<FixedRow>,
    anchor: Anchor,
}

/// The contrasts for a stage: for each of its sibling loads, (load, vs beside the walk
/// (None for the walk itself), the share of the walk sibling's excess (walk stage only,
/// and only where that excess is resolved), sibling over cross-core)
#[allow(clippy::type_complexity)]
fn contrasts_of(sm: &Summary, k: Kernel) -> Vec<(Load, Option<Ci>, Option<Ci>, Option<Ci>)> {
    let (conds, samples) = (sm.conds, sm.samples);
    let Some(ww) = find(conds, Some(k), Some(Load::Walk), Place::Sibling, false) else {
        return Vec::new();
    };
    let walk_resolved = resolved(sm.rows[ww].ratio);
    LOADS
        .iter()
        .filter_map(|&l| {
            let i = find(conds, Some(k), Some(l), Place::Sibling, false)?;
            let (vs, share) = if l == Load::Walk {
                (None, None)
            } else {
                let (r, s) = contrast(conds, samples, i, ww);
                (
                    Some(r),
                    (k == Kernel::Walk && walk_resolved).then_some(s),
                )
            };
            let x = find(conds, Some(k), Some(l), Place::Cross, false)
                .map(|j| contrast(conds, samples, i, j).0);
            Some((l, vs, share, x))
        })
        .collect()
}

/// The markdown summary
fn report(h: &Head, sm: &Summary) -> String {
    use std::fmt::Write;
    let (conds, rows) = (sm.conds, &sm.rows);
    let mut o = String::new();
    let first = h.placements[0];
    let _ = writeln!(o, "## Pair matrix: {}", h.path);
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "{}: {} blocks, {:.1} MB walked a decode; c/B of the walk's output at {} GHz reported; {} rounds of {} conditions and {} fixed-work runs, each round's order shuffled (seed {}).",
        h.env.model,
        h.blocks,
        h.n as f64 / 1e6,
        h.ghz,
        h.rounds,
        conds.len(),
        sm.fixed.len(),
        h.seed
    );
    if h.rounds < MIN_ROUNDS {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**A dry run:** fewer than {MIN_ROUNDS} rounds; its intervals are not for decisions."
        );
    }
    let _ = writeln!(o);
    let a = &sm.anchor;
    let dry = if h.rounds < MIN_ROUNDS {
        " (a dry run: not a reading)"
    } else {
        ""
    };
    let line = if !a.ran {
        "**ANCHOR: NOT RUN.** walk|decode at the sibling with the other core busy needs SMT and another core; no reading is made from this run.".to_string()
    } else {
        let got = format!(
            "walk beside a decoding sibling, the other core busy: x{} fastest of rounds (paired geometric mean x{})",
            num(a.fastest, 2),
            ci3(a.geo, 2)
        );
        match (a.reference, a.pass) {
            (Some((r, src)), Some(true)) => format!(
                "**ANCHOR: PASS.** {got}; phases' 4-copy walk factor on this model x{r:.2} ({src}); difference {:+.2}, within {ANCHOR_BAND}.",
                a.fastest - r
            ),
            (Some((r, src)), _) => format!(
                "**ANCHOR: FAIL.** {got}; phases' 4-copy walk factor on this model x{r:.2} ({src}); difference {}, beyond {ANCHOR_BAND}: the instrument or its alone is off, and no reading is made from this run.",
                num(a.fastest - r, 2)
            ),
            (None, _) => format!(
                "**ANCHOR: NO REFERENCE.** {got}; no phases 4-copy factor is recorded for {}, so it is not judged.",
                h.env.model
            ),
        }
    };
    let _ = writeln!(o, "{line}{dry}");
    let _ = writeln!(o);
    let agree = match h.topo.ids_agree {
        Some(true) => "; core_id and physical_package_id agree",
        Some(false) => "; **core_id or physical_package_id disagree with them**",
        None => "",
    };
    let _ = writeln!(
        o,
        "CPUs by core ({}{agree}): {}. Even rounds: {}.{}",
        h.topo.source,
        h.topo.show(),
        first.show(),
        if h.swap {
            format!(
                " Odd rounds, the cores swapped: {}.",
                h.placements[1].show()
            )
        } else {
            " The cores are not swapped between rounds.".to_string()
        }
    );
    if first.sibling.is_none() {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**No SMT here:** no core has two CPUs this process may use, so the sibling conditions are refused and not run. The cross-core and alone conditions ran."
        );
    }
    if first.cross.is_none() {
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "**One core here:** no CPU on another core, so the cross-core and other-core-busy conditions are not run."
        );
    }
    let _ = writeln!(o);
    let decodes = conds.iter().filter(|c| c.kernel.is_some()).count();
    let count = |f: &dyn Fn(&Condition) -> bool| conds.iter().filter(|c| f(c)).count();
    let mut rs = h.round_s.to_vec();
    rs.sort_by(f64::total_cmp);
    let _ = writeln!(
        o,
        "A round: {decodes} measured decodes ({} beside a sibling load, {} beside a cross-core load, {} beside a sibling load with the other core busy, {} alone with it busy, {} alone), {} load-alone windows of {:.1} s, {} fixed-work runs. The rounds took {} s (median {}).",
        count(&|c| c.place == Place::Sibling && !c.busy),
        count(&|c| c.place == Place::Cross),
        count(&|c| c.place == Place::Sibling && c.busy),
        count(&|c| c.kernel.is_some() && c.place == Place::Alone && c.busy),
        count(&|c| c.kernel.is_some() && c.place == Place::Alone && !c.busy),
        count(&|c| c.kernel.is_none()),
        LOAD_ALONE.as_secs_f64(),
        sm.fixed.len(),
        h.round_s
            .iter()
            .map(|s| format!("{s:.1}"))
            .collect::<Vec<_>>()
            .join(", "),
        num(median(&rs), 1)
    );
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "The load block: block {} of the file, {} bytes. Loads alone ran on S, M idle. The fixed decode: {} blocks spread through the file.",
        h.load_block.0, h.load_block.1, h.subset
    );
    // the idle path, and interrupts on M and S
    let irq = |cpu: &dyn Fn(&Cpus) -> Option<usize>| -> f64 {
        let v: Vec<f64> = sm
            .samples
            .iter()
            .zip(h.placements)
            .flat_map(|(r, p)| {
                r.iter().filter_map(move |s| {
                    let c = cpu(p)?;
                    let u = s.cpu.iter().find(|x| x.cpu == c)?;
                    if u.total == 0 {
                        return None;
                    }
                    Some(u.irqs as f64 / (u.total as f64 / 100.0))
                })
            })
            .collect();
        median(&v)
    };
    let flagged: usize = rows.iter().map(|r| r.idle_flagged).sum::<usize>()
        + sm.frows.iter().map(|r| r.idle_flagged).sum::<usize>();
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "Interrupts, median over conditions: {} a second on M, {} on S. Idle path: cpuidle {} / {}; transparent hugepages {} (defrag {}). Conditions whose idle-by-design CPU was over 1% busy: {flagged} (flagged in the tables).",
        num(irq(&|p| Some(p.measured)), 0),
        num(irq(&|p| p.sibling), 0),
        h.env.cpuidle_driver,
        h.env.cpuidle_governor,
        h.env.thp_enabled,
        h.env.thp_defrag
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "### The loads alone");
    let _ = writeln!(o);
    let _ = writeln!(o, "| load | what | rate (median) | units over 1 ms long | flags |");
    let _ = writeln!(o, "|---|---|---|---|---|");
    for r in rows.iter().filter(|r| r.cond.kernel.is_none()) {
        let l = r.cond.load.unwrap();
        let mut flags = flags_of(r);
        if l == Load::AluDuty {
            flags.insert(
                0,
                format!(
                    "achieved: {}% busy, {} us periods",
                    num(100.0 * r.duty, 0),
                    num(r.period_us, 1)
                ),
            );
        }
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} |",
            l.name(),
            l.about(),
            l.show_rate(r.rate),
            r.long_units,
            flags.join("; ")
        );
    }
    for k in KERNELS {
        let _ = writeln!(o);
        let _ = writeln!(o, "### {}: {}, on M", k.name(), k.title());
        let _ = writeln!(o);
        let _ = writeln!(
            o,
            "| load | where | c/B | ns/B | x alone (95%) | median | fastest | not preempted (rounds) | load speed (95%) | pair yield (95%) | flags |"
        );
        let _ = writeln!(o, "|---|---|---|---|---|---|---|---|---|---|---|");
        for r in rows.iter().filter(|r| r.cond.kernel == Some(k)) {
            let mut flags = flags_of(r);
            if r.cond.load == Some(Load::AluDuty) {
                flags.insert(
                    0,
                    format!(
                        "load {}% busy, {} us periods",
                        num(100.0 * r.duty, 0),
                        num(r.period_us, 1)
                    ),
                );
            }
            let _ = writeln!(
                o,
                "| {} | {} | {:.2} | {:.3} | {} | {} | {} | {} ({}) | {} | {} | {} |",
                r.cond.load.map_or("none", Load::name),
                r.cond.where_(),
                r.cb,
                r.nsb,
                ci3(r.ratio, 2),
                num(r.ratio_median, 2),
                num(r.ratio_fastest, 2),
                ci3(r.ratio_clean, 2),
                r.clean_rounds,
                ci3(r.speed, 2),
                ci3(r.pair_yield, 2),
                flags.join("; ")
            );
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "### The other core busy");
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "Each condition with unmeasured decodes on both CPUs of the other core: its ratio to the stage alone on an idle machine (as phases' 4-copy factors), and to the stage alone with the other core busy (the sibling's own cost there)."
    );
    let _ = writeln!(o);
    let mut busy_lines = Vec::new();
    for k in KERNELS {
        let Some(ab) = find(conds, Some(k), None, Place::Alone, true) else {
            continue;
        };
        for (i, c) in conds.iter().enumerate() {
            if c.kernel != Some(k) || !c.busy {
                continue;
            }
            let own = if i == ab {
                "1".to_string()
            } else {
                ci3(contrast(conds, sm.samples, i, ab).0, 2)
            };
            busy_lines.push(format!(
                "| {} | {} | {} | {own} |",
                k.name(),
                c.load.map_or("none (alone)", Load::name),
                ci3(rows[i].ratio, 2)
            ));
        }
    }
    if busy_lines.is_empty() {
        let _ = writeln!(o, "None: no other core.");
    } else {
        let _ = writeln!(o, "| stage | sibling load | x alone, idle machine (95%) | x alone, other core busy (95%) |");
        let _ = writeln!(o, "|---|---|---|---|");
        for l in busy_lines {
            let _ = writeln!(o, "{l}");
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "### Contrasts, paired by round");
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "Each stage beside a load on its sibling, against the same stage beside the walk on its sibling (the ratio of the two ratios to alone), and against the same load on the other core. The share of the walk sibling's excess over alone that this load's is: the walk stage only, and only where the walk sibling's excess has a 95% interval excluding 0."
    );
    let _ = writeln!(o);
    let mut lines = Vec::new();
    for k in KERNELS {
        for (l, vs, share, x) in contrasts_of(sm, k) {
            let show = |c: Option<Ci>, d: usize, none: &str| c.map_or(none.to_string(), |c| ci3(c, d));
            let share_none = if k != Kernel::Walk || l == Load::Walk {
                ""
            } else {
                "n/a: the walk sibling's excess unresolved"
            };
            lines.push(format!(
                "| {} | {} | {} | {} | {} |",
                k.name(),
                l.name(),
                show(vs, 3, "(itself)"),
                show(share, 2, share_none),
                show(x, 3, "n/a")
            ));
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
        "The core's time for both siblings' walk and the other stage (each stage's time alone, this run's), mixed (one sibling walks while the other runs the other stage, until one kind runs out, the rest beside its own kind), against today's mix (each stage beside a decoding sibling: the |decode rows' factors), and against walk-beside-walk placement (the walk beside the walk, then the other beside its own kind: the worst case, not today's). Below 0: a co-run-aware placement would take that much less of the core for those two stages. An ideal: the factors fixed, the stages free to be reordered (a decode orders a block's stages)."
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "| with | other core | walk beside walk | walk beside it | it beside the walk | it beside itself | walk beside a decode | it beside a decode | mixed vs today's mix (95%) | mixed vs walk-beside-walk placement, worst case (95%) |");
    let _ = writeln!(o, "|---|---|---|---|---|---|---|---|---|---|");
    for busy in [false, true] {
        for x in [Kernel::Mtf, Kernel::Build] {
            let other = if busy { "busy" } else { "idle" };
            match pairing_of(conds, sm.samples, x, busy) {
                Some((today, same, f)) => {
                    let _ = writeln!(
                        o,
                        "| {} | {other} | {} | {} | {} | {} | {} | {} | {} | {} |",
                        x.name(),
                        num(f[0], 2),
                        num(f[1], 2),
                        num(f[2], 2),
                        num(f[3], 2),
                        num(f[4], 2),
                        num(f[5], 2),
                        pct3(today),
                        pct3(same)
                    );
                }
                None => {
                    let _ = writeln!(
                        o,
                        "| {} | {other} | n/a (not run here) | | | | | | | |",
                        x.name()
                    );
                }
            }
        }
    }
    let _ = writeln!(o);
    let _ = writeln!(o, "### Fixed work: the pair's makespan, its tail included");
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "Both siblings given the same fixed work, released together: the walk, MTF or build of the load block {FIXED_REPS} times (each side its own buffers), or whole decodes of the {} blocks. Each side's time from its release, the makespan, the share of the makespan one side ran alone, and the pair's yield: the two sides' works' times alone on M (S idle) over the makespan (2 when neither slows the other, 1 when the core only takes turns).",
        h.subset
    );
    let _ = writeln!(o);
    let _ = writeln!(o, "| on M | on S | M (ms) | S (ms) | makespan (ms) | tail share | pair yield (95%) | flags |");
    let _ = writeln!(o, "|---|---|---|---|---|---|---|---|");
    for r in &sm.frows {
        let mut flags = Vec::new();
        if r.m_preempt + r.s_preempt > 0 {
            flags.push(format!("preempted {}+{}", r.m_preempt, r.s_preempt));
        }
        if r.idle_flagged > 0 {
            flags.push(format!("S over 1% busy in {} rounds", r.idle_flagged));
        }
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            r.fixed.m.name(),
            r.fixed.s.map_or("none", Work::name),
            num(r.m * 1e3, 1),
            num(r.s * 1e3, 1),
            num(r.span * 1e3, 1),
            num(r.tail, 2),
            ci3(r.pair_yield, 2),
            flags.join("; ")
        );
    }
    let _ = writeln!(o);
    for x in [Work::Mtf, Work::Build] {
        let v = fixed_pairing(sm.fixed, sm.fsamples, x);
        let _ = writeln!(
            o,
            "- Measured pairing, {}: 2 x makespan(walk|{0}) over makespan(walk|walk) + makespan({0}|{0}): {}.",
            x.name(),
            v.map_or("n/a (no sibling)".into(), pct3)
        );
    }
    let _ = writeln!(o);
    let _ = writeln!(
        o,
        "c/B and ns/B: the stage's time summed over its windows, the median round. x alone: its time over its round's alone (the mean of the stage's two alones, first and last in its group), the geometric mean of the rounds' ratios; median: the median round's over the median alone; fastest: the fastest round's over the fastest alone (phases' statistic); not preempted: the geometric mean over the rounds with no involuntary context switch in the condition or its alones. Load speed: its rate inside the stage's windows (fitted) over its rate alone. Pair yield: the stage's speed (1 / x alone) plus the load's; 2 when neither slows the other. Flags: load units more than 1 ms longer than their work's median time; involuntary context switches (measured + load threads, all rounds); the idle-by-design CPU of the measured core over 1% busy (/proc/stat, a jiffy's resolution)."
    );
    o
}

/// A row's flags
fn flags_of(r: &Row) -> Vec<String> {
    let mut flags = Vec::new();
    if r.long_units > 0 && r.cond.kernel.is_some() {
        flags.push(format!("{} load units over 1 ms long", r.long_units));
    }
    if r.m_preempt + r.s_preempt > 0 {
        flags.push(format!("preempted {}+{}", r.m_preempt, r.s_preempt));
    }
    if r.idle_flagged > 0 {
        flags.push(format!(
            "idle CPU over 1% busy in {} rounds (up to {}%)",
            r.idle_flagged,
            num(100.0 * r.idle_max, 1)
        ));
    }
    flags
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

fn jci(x: Ci) -> String {
    jarr([jnum(x.0), jnum(x.1), jnum(x.2)])
}

fn jopt(x: Option<usize>) -> String {
    x.map_or("null".into(), |c| c.to_string())
}

fn jostr(x: &Option<String>) -> String {
    x.as_deref().map_or("null".into(), jstr)
}

fn jcpus(c: &Cpus) -> String {
    format!(
        "{{\"measured\":{},\"sibling\":{},\"cross\":{},\"spare\":{},\"busy\":{}}}",
        c.measured,
        jopt(c.sibling),
        jopt(c.cross),
        jopt(c.spare),
        jarr(c.busy_cpus().iter().map(|x| x.to_string()))
    )
}

fn juse(u: &[CpuUse]) -> String {
    jarr(u.iter().map(|x| {
        format!(
            "{{\"cpu\":{},\"busy\":{},\"steal\":{},\"total\":{},\"irqs\":{}}}",
            x.cpu, x.busy, x.steal, x.total, x.irqs
        )
    }))
}

/// Everything as one JSON object: the run and its runner, each condition's samples by round
/// and its summary, the contrasts, the pairing, the fixed work and the anchor
fn json(h: &Head, sm: &Summary, orders: &[Vec<String>]) -> String {
    let (conds, rows, samples) = (sm.conds, &sm.rows, sm.samples);
    let e = h.env;
    let env = format!(
        "{{\"cpu_model\":{},\"kernel\":{},\"cmdline\":{},\"cpuidle_driver\":{},\"cpuidle_governor\":{},\"thp_enabled\":{},\"thp_defrag\":{},\"github_run_id\":{},\"github_run_attempt\":{},\"github_sha\":{}}}",
        jstr(&e.model),
        jstr(&e.kernel),
        jstr(&e.cmdline),
        jstr(&e.cpuidle_driver),
        jstr(&e.cpuidle_governor),
        jstr(&e.thp_enabled),
        jstr(&e.thp_defrag),
        jostr(&e.run_id),
        jostr(&e.run_attempt),
        jostr(&e.sha)
    );
    let topo = format!(
        "{{\"cores\":{},\"source\":{},\"ids_agree\":{},\"smt\":{},\"swap\":{},\"rounds\":{}}}",
        jarr(h.topo.cores.iter().map(|c| jarr(c.iter().map(|x| x.to_string())))),
        jstr(h.topo.source),
        h.topo.ids_agree.map_or("null".into(), |b| b.to_string()),
        h.topo.smt(),
        h.swap,
        jarr(h.placements.iter().map(jcpus))
    );
    let per = |ci: usize, f: &dyn Fn(&Sample) -> String| jarr(samples.iter().map(|r| f(&r[ci])));
    let cs = conds.iter().zip(rows).enumerate().map(|(ci, (c, r))| {
        format!(
            "{{\"id\":{},\"kernel\":{},\"load\":{},\"placement\":{},\"busy\":{},\"again\":{},\"load_unit\":{},\
             \"cpu_load\":{},\"stage_s\":{},\"load_fit_per_ns\":{},\"load_interp_per_ns\":{},\"load_long_units\":{},\
             \"m_preempt\":{},\"s_preempt\":{},\"duty\":{},\"cpu_use\":{},\
             \"summary\":{{\"cb\":{},\"ns_b\":{},\"ratio\":{},\"ratio_median\":{},\"ratio_fastest\":{},\"ratio_not_preempted\":{},\
             \"rounds_not_preempted\":{},\"load_rate_per_ns\":{},\"load_rate_interp_per_ns\":{},\"load_speed\":{},\"pair_yield\":{},\
             \"load_long_units\":{},\"idle_cpu_over_1pct_rounds\":{},\"idle_cpu_busy_max\":{},\"duty_busy_share\":{},\"duty_period_us\":{}}}}}",
            jstr(&c.id()),
            c.kernel.map_or("null".into(), |k| jstr(k.name())),
            c.load.map_or("null".into(), |l| jstr(l.name())),
            jstr(c.place.name()),
            c.busy,
            c.again,
            c.load.map_or("null".into(), |l| jstr(l.unit())),
            jarr(h.placements.iter().map(|p| jopt(c.load.and_then(|_| p.of(c.place))))),
            per(ci, &|s| jnum(s.stage)),
            per(ci, &|s| jnum(s.fit)),
            per(ci, &|s| jnum(s.interp)),
            per(ci, &|s| s.long_units.to_string()),
            per(ci, &|s| s.m_preempt.to_string()),
            per(ci, &|s| s.s_preempt.to_string()),
            if c.load == Some(Load::AluDuty) {
                per(ci, &|s| format!("{{\"on_ns\":{},\"off_ns\":{},\"periods\":{}}}", s.duty.on, s.duty.off, s.duty.periods))
            } else {
                "null".into()
            },
            per(ci, &|s| juse(&s.cpu)),
            jnum(r.cb),
            jnum(r.nsb),
            jci(r.ratio),
            jnum(r.ratio_median),
            jnum(r.ratio_fastest),
            jci(r.ratio_clean),
            r.clean_rounds,
            jnum(r.rate),
            jnum(r.rate_interp),
            jci(r.speed),
            jci(r.pair_yield),
            r.long_units,
            r.idle_flagged,
            jnum(r.idle_max),
            jnum(r.duty),
            jnum(r.period_us),
        )
    });
    let mut contrasts = Vec::new();
    for k in KERNELS {
        for (l, vs, share, x) in contrasts_of(sm, k) {
            if let Some(r) = vs {
                contrasts.push(format!(
                    "{{\"kernel\":{},\"load\":{},\"against\":\"walk_sibling\",\"ratio\":{},\"excess_share\":{}}}",
                    jstr(k.name()),
                    jstr(l.name()),
                    jci(r),
                    share.map_or("null".into(), jci)
                ));
            }
            if let Some(r) = x {
                contrasts.push(format!(
                    "{{\"kernel\":{},\"load\":{},\"against\":\"cross\",\"ratio\":{},\"excess_share\":null}}",
                    jstr(k.name()),
                    jstr(l.name()),
                    jci(r)
                ));
            }
        }
    }
    let mut pairing = Vec::new();
    for busy in [false, true] {
        for x in [Kernel::Mtf, Kernel::Build] {
            if let Some((today, same, f)) = pairing_of(conds, samples, x, busy) {
                pairing.push(format!(
                    "{{\"with\":{},\"other_core_busy\":{busy},\"mixed_over_today\":{},\"mixed_over_walk_beside_walk\":{},\"f_ww\":{},\"f_wx\":{},\"f_xw\":{},\"f_xx\":{},\"f_wd\":{},\"f_xd\":{}}}",
                    jstr(x.name()),
                    jci(today),
                    jci(same),
                    jnum(f[0]),
                    jnum(f[1]),
                    jnum(f[2]),
                    jnum(f[3]),
                    jnum(f[4]),
                    jnum(f[5])
                ));
            }
        }
    }
    let fixed = sm.fixed.iter().zip(&sm.frows).enumerate().map(|(i, (f, r))| {
        let col = |g: &dyn Fn(&FixedSample) -> String| jarr(sm.fsamples.iter().map(|x| g(&x[i])));
        format!(
            "{{\"id\":{},\"m\":{},\"s\":{},\"m_s\":{},\"s_s\":{},\"span_s\":{},\"m_preempt\":{},\"s_preempt\":{},\"cpu_use\":{},\
             \"summary\":{{\"m_s\":{},\"s_s\":{},\"span_s\":{},\"tail_share\":{},\"pair_yield\":{}}}}}",
            jstr(&f.id()),
            jstr(f.m.name()),
            f.s.map_or("null".into(), |w| jstr(w.name())),
            col(&|x| jnum(x.m)),
            col(&|x| jnum(x.s)),
            col(&|x| jnum(x.span)),
            col(&|x| x.m_preempt.to_string()),
            col(&|x| x.s_preempt.to_string()),
            col(&|x| juse(&x.cpu)),
            jnum(r.m),
            jnum(r.s),
            jnum(r.span),
            jnum(r.tail),
            jci(r.pair_yield)
        )
    });
    let fpair = [Work::Mtf, Work::Build].into_iter().filter_map(|x| {
        fixed_pairing(sm.fixed, sm.fsamples, x).map(|v| {
            format!(
                "{{\"with\":{},\"mixed_over_same\":{}}}",
                jstr(x.name()),
                jci(v)
            )
        })
    });
    let a = &sm.anchor;
    let anchor = format!(
        "{{\"condition\":\"walk|decode|sibling+busy\",\"ran\":{},\"ratio_fastest\":{},\"ratio\":{},\"reference\":{},\"source\":{},\"band\":{ANCHOR_BAND},\"pass\":{}}}",
        a.ran,
        jnum(a.fastest),
        jci(a.geo),
        a.reference.map_or("null".into(), |r| jnum(r.0)),
        a.reference.map_or("null".into(), |r| jstr(r.1)),
        a.pass.map_or("null".into(), |p| p.to_string())
    );
    format!(
        "{{\"tool\":\"pair\",\"file\":{},\"env\":{env},\"blocks\":{},\"bytes\":{},\"ghz\":{},\"rounds\":{},\"dry_run\":{},\"seed\":{},\
         \"round_s\":{},\"order\":{},\"topology\":{topo},\"load_block\":{{\"index\":{},\"bytes\":{}}},\"fixed_decode_blocks\":{},\
         \"anchor\":{anchor},\"conditions\":{},\"contrasts\":{},\"pairing\":{},\"fixed\":{},\"fixed_pairing\":{}}}\n",
        jstr(h.path),
        h.blocks,
        h.n,
        jnum(h.ghz),
        h.rounds,
        h.rounds < MIN_ROUNDS,
        h.seed,
        jarr(h.round_s.iter().map(|&s| jnum(s))),
        jarr(orders.iter().map(|o| jarr(o.iter().map(|s| jstr(s))))),
        h.load_block.0,
        h.load_block.1,
        h.subset,
        jarr(cs),
        jarr(contrasts),
        jarr(pairing),
        jarr(fixed),
        jarr(fpair),
    )
}

// ---- main ----

fn usage(msg: &str) -> ! {
    eprintln!("pair: {msg}\n{USAGE}");
    std::process::exit(2);
}

/// The fixed decode's blocks: at most FIXED_DECODE_BLOCKS, spread evenly through the file
fn subset(blocks: usize) -> Vec<usize> {
    if blocks <= FIXED_DECODE_BLOCKS {
        (0..blocks).collect()
    } else {
        (0..FIXED_DECODE_BLOCKS)
            .map(|i| i * blocks / FIXED_DECODE_BLOCKS)
            .collect()
    }
}

fn main() {
    let mut pos = Vec::new();
    let (mut json_out, mut max_blocks, mut cpus_arg, mut seed_arg) = (None, None, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .unwrap_or_else(|| usage(&format!("{name} needs a value")))
        };
        match a.as_str() {
            "--json" => json_out = Some(value("--json")),
            "--blocks" => {
                let v = value("--blocks");
                max_blocks = Some(
                    v.parse::<usize>()
                        .ok()
                        .filter(|&m| m >= 1)
                        .unwrap_or_else(|| usage(&format!("--blocks {v}: a count of at least 1"))),
                )
            }
            "--cpus" => cpus_arg = Some(value("--cpus")),
            "--seed" => {
                let v = value("--seed");
                seed_arg = Some(
                    v.parse::<u64>()
                        .unwrap_or_else(|_| usage(&format!("--seed {v}: a number"))),
                )
            }
            _ => pos.push(a),
        }
    }
    let Some(path) = pos.first().cloned() else {
        usage("FILE.bz2 is required")
    };
    let rounds: usize = pos.get(1).map_or(9, |s| {
        s.parse()
            .ok()
            .filter(|r| (2..=31).contains(r))
            .unwrap_or_else(|| usage(&format!("ROUNDS {s}: 2 to 31")))
    });
    let ghz: f64 = pos.get(2).map_or(2.8, |s| {
        s.parse()
            .ok()
            .filter(|g: &f64| *g > 0.0)
            .unwrap_or_else(|| usage(&format!("GHZ {s}: a clock in GHz")))
    });
    let seed = seed_arg.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64)
    });

    let topo = read_topology();
    let chosen = match &cpus_arg {
        Some(s) => parse_cpus(s, &topo.cores),
        None => choose(&topo.cores),
    };
    let cpus = chosen.unwrap_or_else(|e| {
        eprintln!("pair: {e}");
        std::process::exit(2);
    });
    let alt = if cpus_arg.is_none() {
        swapped(&topo.cores, &cpus)
    } else {
        None
    };
    if cpus.sibling.is_none() {
        eprintln!(
            "pair: no SMT here (CPUs by core: {}), so the sibling conditions are refused; the cross-core and alone conditions run",
            topo.show()
        );
    }
    // each CPU pinned to once before the matrix, so a refusal stops the run here, not with
    // a condition's other thread waiting at its barrier
    let mut every: Vec<usize> = cpus.all();
    every.extend(alt.map(|a| a.all()).unwrap_or_default());
    every.sort();
    every.dedup();
    for c in every {
        std::thread::spawn(move || pin(c))
            .join()
            .expect("pinning a thread");
    }
    if let Some(c) = cpus.spare {
        pin(c);
    }
    let env = read_env();

    let data = std::fs::read(&path).unwrap_or_else(|e| usage(&format!("{path}: {e}")));
    let k = Kernels::detect();
    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }
    if let Some(m) = max_blocks {
        blocks.truncate(m);
    }
    if blocks.is_empty() {
        usage(&format!("{path}: no blocks"));
    }

    // The check (each block's CRC) and the bytes walked; the load block, the largest
    let mut d = Decode::new();
    let mut windows = Vec::new();
    let n = decode(
        &blocks,
        k,
        Kernel::Walk,
        &mut d,
        Instant::now(),
        &mut windows,
        None,
    );
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let mut lb = (0, 0);
    for (i, b) in blocks.iter().enumerate() {
        let len = mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
        if len > lb.1 {
            lb = (i, len);
        }
    }
    let nb = blocks.len();
    let cx = Ctx {
        blocks: &blocks,
        k,
        subset: subset(nb),
    };
    let mut st = State {
        d,
        loads: Loads::new(&blocks[lb.0], &blocks, k, true, nb / 2),
        mine: Loads::new(&blocks[lb.0], &blocks, k, false, 0),
        background: (1..=2)
            .map(|j| Background {
                d: Decode::new(),
                next: j * nb / 3,
            })
            .collect(),
        windows,
        log: Vec::with_capacity(1 << 17),
    };
    st.windows.reserve(4 * nb);

    let conds = conditions(
        cpus.sibling.is_some(),
        cpus.cross.is_some(),
        !cpus.busy_cpus().is_empty(),
    );
    let fixed = fixed_list(cpus.sibling.is_some());
    let placements: Vec<Cpus> = (0..rounds)
        .map(|r| match alt {
            Some(a) if r % 2 == 1 => a,
            _ => cpus,
        })
        .collect();
    eprintln!(
        "pair: {}; {nb} blocks, {:.1} MB walked; {} conditions and {} fixed-work runs a round, {rounds} rounds, seed {seed}; even rounds {}{}",
        env.model,
        n as f64 / 1e6,
        conds.len(),
        fixed.len(),
        cpus.show(),
        alt.map_or(String::new(), |a| format!(", odd rounds {}", a.show()))
    );
    let mut rng = Rng(seed);
    let mut samples = vec![vec![Sample::default(); conds.len()]; rounds];
    let mut fsamples = vec![vec![FixedSample::default(); fixed.len()]; rounds];
    let (mut orders, mut round_s) = (Vec::new(), Vec::new());
    for r in 0..rounds {
        let t = Instant::now();
        let c = placements[r];
        if let Some(sp) = c.spare {
            pin(sp);
        }
        let ord = order(&conds, fixed.len(), &mut rng);
        for slot in &ord {
            match *slot {
                Slot::Cond(i) => samples[r][i] = run(&conds[i], &cx, &c, &mut st),
                Slot::Fixed(i) => fsamples[r][i] = run_fixed(&fixed[i], &cx, &c, &mut st),
            }
        }
        orders.push(
            ord.iter()
                .map(|s| match *s {
                    Slot::Cond(i) => conds[i].id(),
                    Slot::Fixed(i) => fixed[i].id(),
                })
                .collect::<Vec<_>>(),
        );
        round_s.push(t.elapsed().as_secs_f64());
        eprintln!(
            "pair: round {}/{rounds}, {:.1} s",
            r + 1,
            t.elapsed().as_secs_f64()
        );
    }

    let rows = summarise(&conds, &samples, &placements, n, ghz);
    let frows = summarise_fixed(&fixed, &fsamples, &placements);
    let sm = Summary {
        conds: &conds,
        samples: &samples,
        anchor: anchor(&conds, &rows, &env.model),
        rows,
        fixed: &fixed,
        fsamples: &fsamples,
        frows,
    };
    let head = Head {
        path: &path,
        blocks: nb,
        n,
        ghz,
        rounds,
        seed,
        topo: &topo,
        placements: &placements,
        swap: alt.is_some(),
        load_block: lb,
        subset: cx.subset.len(),
        round_s: &round_s,
        env: &env,
    };
    let a = &sm.anchor;
    eprintln!(
        "pair: ANCHOR {}: walk|decode|sibling+busy x{} fastest of rounds, reference {}",
        match a.pass {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None if a.ran => "NO REFERENCE",
            None => "NOT RUN",
        },
        num(a.fastest, 2),
        a.reference
            .map_or("none".to_string(), |r| format!("x{:.2} ({})", r.0, r.1))
    );
    print!("{}", report(&head, &sm));
    if let Some(p) = json_out {
        std::fs::write(&p, json(&head, &sm, &orders)).unwrap();
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
                spare: Some(3),
                busy: [Some(2), Some(3)],
            }
        );
        assert_eq!((c.alone(), c.of(Place::Cross)), (1, Some(2)));
        assert_eq!(c.all(), vec![0, 1, 2, 3]);
        // interleaved siblings: M's sibling is CPU 4, the other core's first CPU is 1
        let c = choose(&[vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare, c.busy),
            (0, Some(4), Some(1), Some(5), [Some(1), Some(5)])
        );
        // no SMT: no sibling, the cross-core conditions only; loads alone on the cross CPU;
        // one background decoder (the other core has one CPU)
        let c = choose(&[vec![0], vec![1], vec![2], vec![3]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare, c.busy),
            (0, None, Some(1), Some(2), [Some(1), None])
        );
        assert_eq!(c.alone(), 1);
        // an SMT core that isn't the first
        let c = choose(&[vec![0], vec![1, 2]]).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare),
            (1, Some(2), Some(0), None)
        );
        // one core of two siblings: no cross-core CPU, no background
        let c = choose(&[vec![0, 1]]).unwrap();
        assert_eq!((c.sibling, c.cross, c.busy), (Some(1), None, [None, None]));
        // one CPU: nothing to run a load on
        assert!(choose(&[vec![0]]).is_err());
    }

    #[test]
    fn cpus_named() {
        let ci = vec![vec![0, 1], vec![2, 3]];
        let c = parse_cpus("1,0,3", &ci).unwrap();
        assert_eq!(
            (c.measured, c.sibling, c.cross, c.spare, c.busy),
            (1, Some(0), Some(3), Some(2), [Some(2), Some(3)])
        );
        let c = parse_cpus("2,-,0", &ci).unwrap();
        assert_eq!((c.sibling, c.cross, c.busy), (None, Some(0), [Some(0), Some(1)]));
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
    fn odd_rounds_swap_the_cores() {
        let ci = vec![vec![0, 1], vec![2, 3]];
        let a = swapped(&ci, &choose(&ci).unwrap()).unwrap();
        assert_eq!(
            a,
            Cpus {
                measured: 2,
                sibling: Some(3),
                cross: Some(0),
                spare: Some(1),
                busy: [Some(0), Some(1)],
            }
        );
        // interleaved: M 1, S 5, X 0 (the first core's first CPU)
        let il = vec![vec![0, 4], vec![1, 5], vec![2, 6], vec![3, 7]];
        let a = swapped(&il, &choose(&il).unwrap()).unwrap();
        assert_eq!((a.measured, a.sibling, a.cross), (1, Some(5), Some(0)));
        // no SMT, one core, or another core of one CPU: no swap
        let ns = vec![vec![0], vec![1], vec![2], vec![3]];
        assert!(swapped(&ns, &choose(&ns).unwrap()).is_none());
        let one = vec![vec![0, 1]];
        assert!(swapped(&one, &choose(&one).unwrap()).is_none());
        let lop = vec![vec![0, 1], vec![2]];
        assert!(swapped(&lop, &choose(&lop).unwrap()).is_none());
        // the swap would leave the main thread without a CPU of its own
        let three = vec![vec![0, 1], vec![2, 3], vec![4]];
        let c = choose(&three).unwrap();
        assert_eq!(c.spare, Some(3));
        assert_eq!(swapped(&three, &c).unwrap().spare, Some(1));
    }

    #[test]
    fn the_condition_list() {
        let all = conditions(true, true, true);
        // walk: alone twice, 14 loads at 2 places, alone busy, 4 sibling loads busy; MTF and
        // the build: alone twice, 7 loads at 2 places, alone busy, 3 busy; 13 loads alone
        assert_eq!(all.len(), (2 + 28 + 1 + 4) + 2 * (2 + 14 + 1 + 3) + 13);
        assert_eq!(all.iter().filter(|c| c.kernel.is_some()).count(), 75);
        let ids: Vec<String> = all.iter().map(Condition::id).collect();
        for (i, id) in ids.iter().enumerate() {
            assert!(!ids[..i].contains(id), "{id} twice");
        }
        for k in KERNELS {
            let (a, b) = alones(&all, k);
            assert!(a.is_some() && b.is_some() && a != b);
            for l in LOADS {
                let on = find(&all, Some(k), Some(l), Place::Sibling, false).is_some();
                assert_eq!(on, l.every_stage() || k == Kernel::Walk, "{k:?} {l:?}");
                assert_eq!(on, find(&all, Some(k), Some(l), Place::Cross, false).is_some());
            }
            assert!(find(&all, Some(k), None, Place::Alone, true).is_some());
            for &l in k.busy_loads() {
                assert!(find(&all, Some(k), Some(l), Place::Sibling, true).is_some());
            }
        }
        // the anchor is there
        assert!(find(&all, Some(Kernel::Walk), Some(Load::Decode), Place::Sibling, true).is_some());
        for l in LOADS {
            assert_eq!(
                find(&all, None, Some(l), Place::Alone, false).is_some(),
                l != Load::Together
            );
        }
        // a stage has a load unless alone; a condition has a stage or a load
        for c in &all {
            assert!(c.kernel.is_some() || c.load.is_some());
            if c.kernel.is_some() {
                assert_eq!(c.load.is_none(), c.place == Place::Alone, "{}", c.id());
            }
            assert!(!c.again || (c.load.is_none() && !c.busy));
        }
        assert_eq!(ids[0], "walk|none|alone");
        assert!(ids.contains(&"walk|decode|sibling+busy".to_string()));
        assert!(ids.contains(&"mtf|none|alone#2".to_string()));
        // no SMT: no sibling conditions, but the busy alones where another core exists
        let x = conditions(false, true, true);
        assert!(x.iter().all(|c| c.place != Place::Sibling));
        assert_eq!(x.len(), (2 + 14 + 1) + 2 * (2 + 7 + 1) + 13);
        // one core: no cross-core or busy conditions
        let o = conditions(true, false, false);
        assert!(o.iter().all(|c| c.place != Place::Cross && !c.busy));
        assert_eq!(o.len(), (2 + 14) + 2 * (2 + 7) + 13);
    }

    #[test]
    fn the_rounds_order() {
        let conds = conditions(true, true, true);
        let fixed = fixed_list(true);
        let mut rng = Rng(7);
        let a = order(&conds, fixed.len(), &mut rng);
        let b = order(&conds, fixed.len(), &mut rng);
        // every slot once
        for o in [&a, &b] {
            assert_eq!(o.len(), conds.len() + fixed.len());
            for i in 0..conds.len() {
                assert_eq!(o.iter().filter(|&&s| s == Slot::Cond(i)).count(), 1);
            }
            for i in 0..fixed.len() {
                assert_eq!(o.iter().filter(|&&s| s == Slot::Fixed(i)).count(), 1);
            }
            // each stage's group contiguous, between its two alones
            for k in KERNELS {
                let at: Vec<usize> = o
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| matches!(s, Slot::Cond(i) if conds[*i].kernel == Some(k)))
                    .map(|(p, _)| p)
                    .collect();
                assert_eq!(at.last().unwrap() - at[0] + 1, at.len(), "{k:?} contiguous");
                let (first, last) = alones(&conds, k);
                assert_eq!(o[at[0]], Slot::Cond(first.unwrap()));
                assert_eq!(o[*at.last().unwrap()], Slot::Cond(last.unwrap()));
            }
        }
        // rounds differ; the same seed gives the same orders
        assert_ne!(a, b);
        let mut again = Rng(7);
        assert_eq!(order(&conds, fixed.len(), &mut again), a);
        assert_eq!(order(&conds, fixed.len(), &mut again), b);
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
    }

    #[test]
    fn long_units_are_by_absolute_excess() {
        // 50 us units, one of them 2 ms: one long unit; a 0.9 ms one is not
        let mut units = vec![(50e3, 0.0, 100.0); 20];
        units.push((2e6, 0.0, 100.0));
        units.push((0.9e6, 0.0, 100.0));
        assert_eq!(long_units(&units), 1);
        // a 3 ms unit of three times the work at 1 ms a work unit is not long
        let units = vec![(1e6, 0.0, 1.0), (1e6, 0.0, 1.0), (3e6, 0.0, 3.0)];
        assert_eq!(long_units(&units), 0);
        assert_eq!(long_units(&[]), 0);
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
        assert_eq!(least(&[3.0, f64::NAN, 1.5, 2.0]), 1.5);
        assert!(least(&[f64::NAN]).is_nan());
        assert!(resolved((1.2, 1.1, 1.3)) && resolved((0.8, 0.7, 0.9)));
        assert!(!resolved((1.1, 0.9, 1.3)) && !resolved((1.1, f64::NAN, f64::NAN)));
    }

    #[test]
    fn the_pairing_model() {
        let f = |ww, wx, xw, xx, wd, xd| Factors {
            ww,
            wx,
            xw,
            xx,
            wd,
            xd,
        };
        // nothing slows anything: every placement takes the same core time
        let (m, s, t) = pairing(1.0, 1.0, &f(1.0, 1.0, 1.0, 1.0, 1.0, 1.0));
        assert!((m - 2.0).abs() < 1e-12 && (s - 2.0).abs() < 1e-12 && (t - 2.0).abs() < 1e-12);
        // every sibling costs the same (2x): ties, whatever the work's split (walk 1, X 0.5:
        // mixed 2 while X lasts, then the walk's rest beside itself, 1 x 2 / 2)
        let (m, s, t) = pairing(1.0, 0.5, &f(2.0, 2.0, 2.0, 2.0, 2.0, 2.0));
        assert!((m - 3.0).abs() < 1e-12 && (s - 3.0).abs() < 1e-12 && (t - 3.0).abs() < 1e-12);
        // worked by hand: walk 1 and X 1 alone; mixed runs 2.4 (the walk beside X, 2 x 1.2)
        // while X advances 2.4 / 1.4 = 12/7, leaving 2/7 of X beside itself at 1.4 / 2:
        // 2.4 + 0.2 = 2.6. Same-type 2 + 1.4 = 3.4; today 1.8 + 1.3 = 3.1.
        let (m, s, t) = pairing(1.0, 1.0, &f(2.0, 1.2, 1.4, 1.4, 1.8, 1.3));
        assert!((m - 2.6).abs() < 1e-12, "{m}");
        assert!((s - 3.4).abs() < 1e-12 && (t - 3.1).abs() < 1e-12);
        // with the walk dearer beside a walk than beside today's decode (ww > wd, as P
        // predicts), the same-type denominator overstates the lever against today's mix
        assert!(m / s < m / t && m / t < 1.0);
        // only where today's factors equal the same-type ones do the two agree
        let (m, s, t) = pairing(1.0, 1.0, &f(2.0, 1.2, 1.4, 1.4, 2.0, 1.4));
        assert!((s - t).abs() < 1e-12 && (m / s - m / t).abs() < 1e-12);
        // a walk beside anything else costs more than beside a walk: mixed loses
        let (m, s, _) = pairing(1.0, 1.0, &f(1.5, 2.0, 2.0, 1.5, 1.7, 1.7));
        assert!(m > s);
    }

    fn sample(stage: f64, rate: f64) -> Sample {
        Sample {
            stage,
            fit: rate,
            interp: rate,
            ..Sample::default()
        }
    }

    #[test]
    fn the_summary_arithmetic() {
        let cpus = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let conds = conditions(true, true, true);
        // every stage 1 s alone (first alone 0.9, last 1.1 in round 0) and 2 s beside a
        // load; every load at rate 1 alone, 0.5 beside
        let mut samples: Vec<Vec<Sample>> = (0..3)
            .map(|_| {
                conds
                    .iter()
                    .map(|c| match (c.kernel, c.load) {
                        (Some(_), None) if !c.busy => sample(1.0, f64::NAN),
                        (Some(_), None) => sample(1.5, f64::NAN),
                        (Some(_), Some(_)) => sample(2.0, 0.5),
                        (None, _) => sample(f64::NAN, 1.0),
                    })
                    .collect()
            })
            .collect();
        let (a, b) = alones(&conds, Kernel::Walk);
        samples[0][a.unwrap()].stage = 0.9;
        samples[0][b.unwrap()].stage = 1.1;
        // a preempted round 2 at walk|alu|sibling, slower there
        let wa = find(&conds, Some(Kernel::Walk), Some(Load::Alu), Place::Sibling, false).unwrap();
        samples[2][wa].stage = 4.0;
        samples[2][wa].m_preempt = 1;
        let placements = vec![cpus; 3];
        let rows = summarise(&conds, &samples, &placements, 1_000_000_000, 3.0);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let r = &rows[find(&conds, Some(Kernel::Walk), Some(Load::Mtf), Place::Sibling, false).unwrap()];
        assert!(near(r.ratio.0, 2.0) && near(r.speed.0, 0.5) && near(r.pair_yield.0, 1.0));
        // 2 s over 1e9 bytes: 2 ns/B, 6 c/B at 3 GHz
        assert!(near(r.nsb, 2.0) && near(r.cb, 6.0));
        assert!(near(r.ratio_median, 2.0) && near(r.ratio_fastest, 2.0));
        // the preempted round: in the geometric mean, out of the clean one; the fastest and
        // the median are the other rounds'
        let p = &rows[wa];
        assert!(near(p.ratio.0, (2.0f64 * 2.0 * 4.0).powf(1.0 / 3.0)));
        assert!(near(p.ratio_clean.0, 2.0) && p.clean_rounds == 2);
        assert!(near(p.ratio_median, 2.0) && near(p.ratio_fastest, 2.0));
        assert_eq!((p.m_preempt, p.s_preempt), (1, 0));
        // the alones: each against their round's mean
        let a0 = &rows[a.unwrap()];
        assert!(near(a0.ratio_median, 1.0));
        // a busy condition: against the idle machine's alone
        let bz = &rows[find(&conds, Some(Kernel::Walk), None, Place::Alone, true).unwrap()];
        assert!(near(bz.ratio.0, 1.5));
        let l = &rows[find(&conds, None, Some(Load::Chain), Place::Alone, false).unwrap()];
        assert!(near(l.rate, 1.0) && l.ratio.0.is_nan());
        // the together load has no alone, so no speed
        let t = &rows[find(&conds, Some(Kernel::Walk), Some(Load::Together), Place::Sibling, false).unwrap()];
        assert!(near(t.ratio.0, 2.0) && t.speed.0.is_nan());
        // every sibling the same: contrasts at 1, the walk's excess all there; pairing ties
        let ww = find(&conds, Some(Kernel::Walk), Some(Load::Walk), Place::Sibling, false).unwrap();
        let wm = find(&conds, Some(Kernel::Walk), Some(Load::Mtf), Place::Sibling, false).unwrap();
        let (vs, share) = contrast(&conds, &samples, wm, ww);
        assert!(near(vs.0, 1.0) && near(share.0, 1.0));
        let (today, same, f) = pairing_of(&conds, &samples, Kernel::Mtf, false).unwrap();
        assert!(near(today.0, 1.0) && near(same.0, 1.0) && f.iter().all(|&f| near(f, 2.0)));
        assert!(pairing_of(&conds, &samples, Kernel::Build, true).is_some());
        // no SMT: no pairing table
        let conds = conditions(false, true, true);
        let samples = vec![vec![sample(1.0, 1.0); conds.len()]; 2];
        assert!(pairing_of(&conds, &samples, Kernel::Build, false).is_none());
    }

    #[test]
    fn the_share_needs_a_resolved_walk_excess() {
        let cpus = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let conds = conditions(true, true, false);
        let mk = |walk: [f64; 3]| -> Vec<Vec<Sample>> {
            (0..3)
                .map(|r| {
                    conds
                        .iter()
                        .map(|c| match (c.kernel, c.load) {
                            (Some(_), None) => sample(1.0, f64::NAN),
                            (Some(Kernel::Walk), Some(Load::Walk)) if c.place == Place::Sibling => {
                                sample(walk[r], 1.0)
                            }
                            (Some(_), Some(_)) => sample(1.5, 1.0),
                            (None, _) => sample(f64::NAN, 1.0),
                        })
                        .collect()
                })
                .collect()
        };
        let resolved_case = mk([2.0, 2.1, 2.2]);
        let unresolved = mk([0.8, 1.2, 1.0]);
        for (samples, has_share) in [(resolved_case, true), (unresolved, false)] {
            let rows = summarise(&conds, &samples, &[cpus; 3], 1000, 3.0);
            let sm = Summary {
                conds: &conds,
                samples: &samples,
                anchor: anchor(&conds, &rows, "x"),
                rows,
                fixed: &[],
                fsamples: &[],
                frows: Vec::new(),
            };
            let walk = contrasts_of(&sm, Kernel::Walk);
            // the walk itself: no self-contrast, but its sibling-vs-cross
            let me = walk.iter().find(|x| x.0 == Load::Walk).unwrap();
            assert!(me.1.is_none() && me.2.is_none() && me.3.is_some());
            let alu = walk.iter().find(|x| x.0 == Load::Alu).unwrap();
            assert!(alu.1.is_some());
            assert_eq!(alu.2.is_some(), has_share);
            // other stages: never a share
            assert!(contrasts_of(&sm, Kernel::Mtf).iter().all(|x| x.2.is_none()));
            assert_eq!(walk.len(), 14);
            assert_eq!(contrasts_of(&sm, Kernel::Build).len(), 7);
        }
    }

    #[test]
    fn the_anchor() {
        assert_eq!(
            anchor_ref("AMD EPYC 7763 64-Core Processor").map(|r| r.0),
            Some(2.12)
        );
        assert_eq!(
            anchor_ref("AMD EPYC 9V74 80-Core Processor").map(|r| r.0),
            Some(2.22)
        );
        assert_eq!(anchor_ref("AMD EPYC 9V45 96-Core Processor").map(|r| r.0), Some(2.23));
        assert_eq!(
            anchor_ref("Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz").map(|r| r.0),
            Some(1.61)
        );
        assert!(anchor_ref("Intel(R) Xeon(R) Platinum 8573C").is_none());
        let cpus = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let conds = conditions(true, true, true);
        let i = find(&conds, Some(Kernel::Walk), Some(Load::Decode), Place::Sibling, true).unwrap();
        for (fastest, pass) in [(2.20, Some(true)), (2.30, Some(false)), (1.98, Some(true)), (1.95, Some(false))] {
            let samples: Vec<Vec<Sample>> = (0..3)
                .map(|r| {
                    conds
                        .iter()
                        .enumerate()
                        .map(|(j, c)| match c.kernel {
                            Some(_) if c.load.is_none() && !c.busy => sample(1.0, f64::NAN),
                            Some(_) if j == i => sample(fastest + 0.1 * r as f64, 1.0),
                            Some(_) => sample(1.5, 1.0),
                            None => sample(f64::NAN, 1.0),
                        })
                        .collect()
                })
                .collect();
            let rows = summarise(&conds, &samples, &[cpus; 3], 1000, 3.0);
            let a = anchor(&conds, &rows, "AMD EPYC 7763 64-Core Processor");
            assert!(a.ran && (a.fastest - fastest).abs() < 1e-9);
            assert_eq!(a.pass, pass, "{fastest}");
            let a = anchor(&conds, &rows, "an unknown CPU");
            assert!(a.ran && a.pass.is_none());
        }
        let conds = conditions(false, true, true);
        let rows = summarise(&conds, &vec![vec![sample(1.0, 1.0); conds.len()]; 2], &[cpus; 2], 1000, 3.0);
        assert!(!anchor(&conds, &rows, "AMD EPYC 7763").ran);
    }

    #[test]
    fn the_fixed_work_arithmetic() {
        let fixed = fixed_list(true);
        assert_eq!(fixed.len(), 10);
        assert!(fixed_list(false).iter().all(|f| f.s.is_none()));
        // alone: walk 1 s, MTF 0.5, build 0.5, decode 2; walk|mtf: M 1.2, S 0.6, makespan 1.2;
        // walk|walk 2 a side; mtf|mtf 0.8 a side
        let time = |f: &Fixed| -> (f64, f64) {
            match (f.m, f.s) {
                (Work::Walk, None) => (1.0, f64::NAN),
                (Work::Mtf | Work::Build, None) => (0.5, f64::NAN),
                (Work::Decode, None) => (2.0, f64::NAN),
                (Work::Walk, Some(Work::Walk)) => (2.0, 2.0),
                (Work::Walk, Some(_)) => (1.2, 0.6),
                (Work::Mtf, Some(_)) | (Work::Build, Some(_)) => (0.8, 0.8),
                (Work::Decode, Some(_)) => (3.0, 3.0),
            }
        };
        let fs: Vec<Vec<FixedSample>> = (0..2)
            .map(|_| {
                fixed
                    .iter()
                    .map(|f| {
                        let (m, s) = time(f);
                        FixedSample {
                            m,
                            s,
                            span: if s.is_nan() { m } else { m.max(s) },
                            ..FixedSample::default()
                        }
                    })
                    .collect()
            })
            .collect();
        let cpus = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let rows = summarise_fixed(&fixed, &fs, &[cpus; 2]);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let row = |m, s| &rows[find_fixed(&fixed, m, s).unwrap()];
        // walk|mtf: (1 + 0.5) / 1.2; the tail half the makespan
        let r = row(Work::Walk, Some(Work::Mtf));
        assert!(near(r.pair_yield.0, 1.5 / 1.2) && near(r.tail, 0.5) && near(r.span, 1.2));
        // walk|walk: 2 / 2, no tail
        let r = row(Work::Walk, Some(Work::Walk));
        assert!(near(r.pair_yield.0, 1.0) && near(r.tail, 0.0));
        // decode|decode: 4 / 3
        assert!(near(row(Work::Decode, Some(Work::Decode)).pair_yield.0, 4.0 / 3.0));
        // alone: yield 1, no tail
        let r = row(Work::Walk, None);
        assert!(near(r.pair_yield.0, 1.0) && r.tail.is_nan());
        // measured pairing: 2 x 1.2 / (2 + 0.8)
        assert!(near(fixed_pairing(&fixed, &fs, Work::Mtf).unwrap().0, 2.4 / 2.8));
        assert!(fixed_pairing(&fixed_list(false), &fs, Work::Mtf).is_none());
        assert_eq!(subset(10), (0..10).collect::<Vec<_>>());
        let s = subset(431);
        assert_eq!((s.len(), s[0], s[1], s[63]), (64, 0, 6, 424));
    }

    #[test]
    fn proc_parsing() {
        let stat = "cpu  10 0 5 100 0 1 1 2 0 0\ncpu0 4 0 2 50 0 1 0 1 0 0\ncpu1 6 0 3 50 0 0 1 1 0 0\nintr 123\n";
        assert_eq!(
            parse_stat(stat),
            vec![(0, [7, 1, 58]), (1, [10, 1, 61])]
        );
        let irq = "           CPU0       CPU1\n  0:         10          2   IO-APIC   2-edge      timer\nLOC:       100        300   Local timer interrupts\nERR:          0\nMIS:          0\n";
        assert_eq!(parse_interrupts(irq), vec![(0, 110), (1, 302)]);
        assert!(parse_interrupts("").is_empty());
        let a = Snap {
            stat: vec![(0, [7, 1, 58]), (1, [10, 1, 61])],
            irq: vec![(0, 110), (1, 302)],
        };
        let b = Snap {
            stat: vec![(0, [9, 1, 158]), (1, [10, 1, 161])],
            irq: vec![(0, 150), (1, 303)],
        };
        let u = used(&a, &b, &[]);
        assert_eq!(
            u[0],
            CpuUse {
                cpu: 0,
                busy: 2,
                steal: 0,
                total: 100,
                irqs: 40
            }
        );
        assert!((busy_share(&u, 0) - 0.02).abs() < 1e-12 && busy_share(&u, 1) == 0.0);
        assert!(busy_share(&u, 5).is_nan());
        assert_eq!(used(&a, &b, &[1]).len(), 1);
        // the idle-by-design CPU
        let c = choose(&[vec![0, 1], vec![2, 3]]).unwrap();
        let alone = Condition::new(Some(Kernel::Walk), None, Place::Alone, false);
        let cross = Condition::new(Some(Kernel::Walk), Some(Load::Alu), Place::Cross, false);
        let sib = Condition::new(Some(Kernel::Walk), Some(Load::Alu), Place::Sibling, true);
        let la = Condition::new(None, Some(Load::Alu), Place::Alone, false);
        assert_eq!(
            [idle_cpu(&alone, &c), idle_cpu(&cross, &c), idle_cpu(&sib, &c), idle_cpu(&la, &c)],
            [Some(1), Some(1), None, Some(0)]
        );
    }

    #[test]
    fn json_values() {
        assert_eq!(jnum(f64::NAN), "null");
        assert_eq!(jnum(1.5), "1.5");
        assert_eq!(jstr("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
        assert_eq!(jci((1.0, f64::INFINITY, 2.0)), "[1,null,2]");
        assert_eq!(jostr(&None), "null");
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
    fn the_chases() {
        // the L2 table is one cycle through every slot
        let n = 4096;
        let t = cycle_table(n, 3);
        let mut seen = vec![false; n];
        let mut p = t[0];
        for _ in 0..n {
            let s = (p >> 8) as usize;
            assert!(!seen[s], "slot {s} twice before the cycle closed");
            seen[s] = true;
            p = t[s];
        }
        assert!(seen.iter().all(|&x| x) && p == t[0]);
        // the lanes start 1/32 of the cycle apart, and step as the plain chain does
        let mut lanes = lanes_on(&t, 0);
        assert!(links_in_bounds(&t, &lanes));
        assert_eq!(lanes[1], chain(&t, lanes[0], n / LANES));
        let start = lanes;
        // SAFETY: the links and lanes are in bounds (checked above)
        unsafe { chase32(&t, &mut lanes, 100) };
        for (l, s) in lanes.iter().zip(start) {
            assert_eq!(*l, chain(&t, s, 100));
        }
        // a block's table: the lanes stay on its one cycle
        let ll = b"banana_bandana_cabana".to_vec();
        let mut counts = [0u32; 256];
        for &c in &ll {
            counts[c as usize] += 1;
        }
        let bt = table(&ll, &counts);
        let mut lanes = lanes_on(&bt, 0);
        assert!(links_in_bounds(&bt, &lanes));
        let start = lanes;
        // SAFETY: as above
        unsafe { chase32(&bt, &mut lanes, 7) };
        for (l, s) in lanes.iter().zip(start) {
            assert_eq!(*l, chain(&bt, s, 7));
        }
        assert!(!links_in_bounds(&[5 << 8], &[]));
    }

    #[test]
    fn the_loads_run_their_units() {
        alu(1);
        alu(1000);
        let words: Vec<u64> = (1..=16).collect();
        assert_eq!(sum(&words), (0..8).fold(0, |x, j| x ^ ((j + 1) + (j + 9))));
        let mut buf = vec![0u64; L1_WORDS];
        stores(&mut buf, 3, 10);
        assert!(buf.iter().all(|&w| w == 12));
        let d = Duty {
            on: 20_000,
            off: 30_000,
            periods: 2,
        };
        assert!((d.share() - 0.4).abs() < 1e-12 && (d.period_us() - 25.0).abs() < 1e-12);
        assert!(Duty::default().share().is_nan());
    }

    /// A few blocks of a real stream, from libbzip2
    fn blocks_of(data: &[u8], level: u32) -> Vec<Symbols> {
        use std::io::Write;
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
        e.write_all(data).unwrap();
        let z = e.finish().unwrap();
        let mut sc = Scanner::new(SliceSource(&z));
        let mut v = Vec::new();
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
            v.push(symbols);
        }
        v
    }

    #[test]
    fn the_decoding_loads() {
        // three blocks of 100k-level text
        let text: Vec<u8> = (0..250_000u32)
            .map(|i| b"the walk beside the walk "[(i as usize * 7 + (i as usize >> 9)) % 25])
            .collect();
        let blocks = blocks_of(&text, 1);
        assert!(blocks.len() >= 2);
        let k = Kernels::detect();
        let mut st = Loads::new(&blocks[0], &blocks, k, false, 1);
        // the decode load runs on through the blocks, wrapping
        let total: u64 = (0..blocks.len() + 1).map(|_| st.unit(Load::Decode)).sum();
        assert!(total > 0);
        // every load's unit runs (the streams' buffers are empty here)
        for l in LOADS {
            if !matches!(l, Load::Stream | Load::Stream8) {
                assert!(st.unit(l) > 0, "{l:?}");
            }
        }
        assert!(st.duty.periods >= 1 && st.duty.on > 0);
        // the fixed work over a subset
        fixed_work(Work::Decode, &mut st, &[0, 1]);
        // the together load and the measured decode, in step: both finish
        let stop = AtomicBool::new(false);
        let rv = Rendezvous::default();
        let epoch = Instant::now();
        let (mut d, mut w, mut log) = (Decode::new(), Vec::new(), Vec::new());
        std::thread::scope(|s| {
            let (stop, rv, st, log) = (&stop, &rv, &mut st, &mut log);
            let h = s.spawn(move || {
                let meet = Meet {
                    mine: &rv.s,
                    theirs: &rv.m,
                    stop,
                };
                run_together(st, &meet, stop, epoch, log);
            });
            let guard = StopOnDrop(stop);
            let meet = Meet {
                mine: &rv.m,
                theirs: &rv.s,
                stop,
            };
            let n = decode(&blocks, k, Kernel::Walk, &mut d, epoch, &mut w, Some(&meet));
            assert!(n > 0);
            drop(guard);
            h.join().unwrap();
        });
        assert_eq!(rv.m.load(Ordering::Relaxed), blocks.len() as u64);
        assert_eq!(rv.s.load(Ordering::Relaxed), blocks.len() as u64);
        assert!(covers(&log, &w));
    }

    #[test]
    fn a_rendezvous_given_up() {
        // the other side never comes: stop releases the waiter
        let (a, b, stop) = (AtomicU64::new(0), AtomicU64::new(0), AtomicBool::new(false));
        std::thread::scope(|s| {
            let h = s.spawn(|| {
                Meet {
                    mine: &a,
                    theirs: &b,
                    stop: &stop,
                }
                .meet()
            });
            std::thread::sleep(Duration::from_millis(5));
            stop.store(true, Ordering::Release);
            assert!(!h.join().unwrap());
        });
    }
}
