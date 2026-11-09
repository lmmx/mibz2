//! The walk's mechanisms taken apart on a file's blocks, on one thread: the interleaved
//! chains (the lane count), the software prefetch (none, into L1, into L2, non-temporal),
//! and the scheduling (the room walk's passes against the flagged walk's per-step
//! bookkeeping), each setting's output checked against the plain walk's. A diagnostic, for
//! choosing settings to try: the CI benchmark (the whole decoder, A/B) decides what is
//! faster.
//!
//! ```text
//! sweep FILE.bz2 [ROUNDS] [GHZ]
//! ```
//!
//! Each round decodes the file once at each setting, as `decode` does on one thread: block
//! by block, MTF, the build, the walk and RLE1, only the walk timed, so each walk comes
//! after the stages and the walk of the block before at the same setting, in the caches'
//! state a decode leaves (not after the same block walked moments before at the same
//! setting, which repeats find faster than any decode: up to a quarter at 512 segments on
//! one CPU, so that settings sharing the default's layout looked faster than the rest). The
//! settings' order is rotated round by round. A setting's round is its time summed over
//! the blocks. Against the default (the settings `Segmented` uses), each round's ratio is
//! paired, and the geometric mean of the rounds' ratios is given with its 95% interval (a
//! t-interval on the logs). Each setting's output is checked against the plain walk's
//! first, block by block.
//!
//! A block shorter than `MIN_SEGMENTED` (a stream's last; the bench corpus has 198) is
//! walked plainly at every setting, as the decoder walks it (`walk_fast`): segmented, it
//! would charge each setting's segment count on blocks of tens of rows, which no decode pays
//! (`docs/journal/2026-11-05-segment-cost.md`).
//!
//! Where the kernel lets a process count its own events (Linux on x86_64, a PMU exposed to
//! the machine, `perf_event_paranoid` at 2 or below), one more round, untimed, counts each
//! setting's instructions, cycles, last-level cache misses and back-end stall cycles, as
//! figures a byte of output: what a setting costs in work against what it waits for. Where
//! it doesn't, the table says so and has no counter columns.

use std::time::Instant;

use mibz2::bwt::{
    build, walk, walk_segments_flagged_with_prefetch, walk_segments_room_with_prefetch, SegScratch,
    TVector, MIN_SEGMENTED, PF_L1, PF_L2, PF_NTA, PF_OFF,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, rle, Kernels};

type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize);

struct Setting {
    walk: &'static str,
    lanes: usize,
    pf: u8,
    segments: usize,
    f: Walk,
}

impl Setting {
    /// The setting's walk of a block as the decoder would walk it: a block shorter than
    /// `MIN_SEGMENTED` plainly, whatever the setting (`walk_fast`), the rest segmented
    fn run(&self, tt: &mut TVector, orig_ptr: u32, out: &mut Vec<u8>, seg: &mut SegScratch) {
        if tt.len() < MIN_SEGMENTED {
            walk(tt, orig_ptr, out)
        } else {
            (self.f)(tt, orig_ptr, out, seg, self.segments)
        }
    }
}

macro_rules! settings {
    ($v:ident, $segments:expr; $($l:literal),*) => {
        $(
            for (pf, room, flagged) in [
                (PF_OFF, walk_segments_room_with_prefetch::<$l, PF_OFF> as Walk, walk_segments_flagged_with_prefetch::<$l, PF_OFF> as Walk),
                (PF_L1, walk_segments_room_with_prefetch::<$l, PF_L1>, walk_segments_flagged_with_prefetch::<$l, PF_L1>),
                (PF_L2, walk_segments_room_with_prefetch::<$l, PF_L2>, walk_segments_flagged_with_prefetch::<$l, PF_L2>),
                (PF_NTA, walk_segments_room_with_prefetch::<$l, PF_NTA>, walk_segments_flagged_with_prefetch::<$l, PF_NTA>),
            ] {
                $v.push(Setting { walk: "room", lanes: $l, pf, segments: $segments, f: room });
                $v.push(Setting { walk: "flagged", lanes: $l, pf, segments: $segments, f: flagged });
            }
        )*
    };
}

// The settings `Segmented` uses (bwt.rs: walk_segments_room, FLAGGED_LANES, PREFETCH,
// FLAGGED_SEGMENTS)
const DEFAULT: (&str, usize, u8, usize) = ("room", 32, PF_L1, 512);

// Two-sided 95% t quantiles for 1 to 30 degrees of freedom
const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

fn pf_name(pf: u8) -> &'static str {
    match pf {
        PF_OFF => "none",
        PF_L1 => "L1",
        PF_L2 => "L2",
        _ => "NTA",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: sweep FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(5, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    // The factorial at 512 segments, then the default's segment count varied alone
    let mut settings = Vec::new();
    settings!(settings, 512; 8, 12, 16, 24, 32, 48, 64);
    for segments in [256, 1024, 2048] {
        settings.push(Setting {
            walk: "room",
            lanes: 32,
            pf: PF_L1,
            segments,
            f: walk_segments_room_with_prefetch::<32, PF_L1>,
        });
    }
    let default = settings
        .iter()
        .position(|s| (s.walk, s.lanes, s.pf, s.segments) == DEFAULT)
        .unwrap();
    let mut counters = pmu::Counters::open();

    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }

    // times[round][setting]: seconds summed over the blocks; counts[setting]: the
    // counters' totals over the untimed round
    let mut times = vec![vec![0f64; settings.len()]; rounds];
    let mut counts = vec![[0u64; pmu::EVENTS]; settings.len()];
    // each setting's group time enabled and running, summed
    let mut cover = vec![(0u64, 0u64); settings.len()];
    let (mut ll, mut freq) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out, mut bytes) = (Vec::new(), Vec::new(), Vec::new());
    let mut n = 0usize;
    for (bi, b) in blocks.iter().enumerate() {
        mtf::mtf(b, &mut ll, &mut freq, k).unwrap();
        build(&ll, &freq, &mut tt);
        walk(&tt, b.orig_ptr, &mut plain);
        n += plain.len();
        for s in &settings {
            s.run(&mut tt, b.orig_ptr, &mut out, &mut seg);
            assert!(
                out == plain,
                "block {bi}: the {} walk, {} lanes, prefetch {}, {} segments",
                s.walk,
                s.lanes,
                pf_name(s.pf),
                s.segments
            );
        }
    }

    // A decode of the file at one setting, as `decode` runs it on one thread; `time` wraps
    // the walk alone
    let mut decode = |s: &Setting, time: &mut dyn FnMut(&mut dyn FnMut())| {
        for b in &blocks {
            mtf::mtf(b, &mut ll, &mut freq, k).unwrap();
            build(&ll, &freq, &mut tt);
            time(&mut || s.run(&mut tt, b.orig_ptr, &mut out, &mut seg));
            bytes.clear();
            rle::unrle(&out, &mut bytes, k).unwrap();
        }
    };
    for (r, round) in times.iter_mut().enumerate() {
        for j in 0..settings.len() {
            let si = (j + r) % settings.len();
            decode(&settings[si], &mut |f| {
                let t = Instant::now();
                f();
                round[si] += t.elapsed().as_secs_f64();
            });
        }
    }
    if let Some(c) = counters.as_mut() {
        for (si, s) in settings.iter().enumerate() {
            decode(s, &mut |f| {
                c.start();
                f();
                let (v, enabled, running) = c.stop();
                for (total, v) in counts[si].iter_mut().zip(v) {
                    *total += v;
                }
                cover[si].0 += enabled;
                cover[si].1 += running;
            });
        }
    }

    let cb = |secs: f64| secs * ghz * 1e9 / n as f64;
    let median = |si: usize| {
        let mut v: Vec<f64> = times.iter().map(|round| round[si]).collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let paired = |si: usize| {
        let logs: Vec<f64> = times
            .iter()
            .map(|round| (round[si] / round[default]).ln())
            .collect();
        let m = logs.iter().sum::<f64>() / rounds as f64;
        let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (rounds - 1) as f64;
        let h = T95[rounds - 2] * (var / rounds as f64).sqrt();
        (m.exp() - 1.0, (m - h).exp() - 1.0, (m + h).exp() - 1.0)
    };
    println!(
        "{path}: {} blocks, {:.1} MB of output; the walk alone at {ghz} GHz, {rounds} rounds, \
         against the {} walk, {} lanes, prefetch {}, {} segments",
        blocks.len(),
        n as f64 / 1e6,
        DEFAULT.0,
        DEFAULT.1,
        pf_name(DEFAULT.2),
        DEFAULT.3
    );
    match &counters {
        Some(c) => println!(
            "counters (one untimed round, one group): {}; a byte of output, misses a kB, \
             unscaled; \"counted\" is the share of the walks' time the group was on the PMU: \
             below 100% it was multiplexed out, and that row's counts are exploratory",
            c.names().join(", ")
        ),
        None => println!("counters: unavailable here ({})", pmu::why()),
    }
    print!("walk     lanes  prefetch  segments    c/B  change (95% interval)        ");
    if counters.is_some() {
        print!("  ins/B  cyc/B  LLC miss/kB  back-end stall  counted");
    }
    println!();
    let mut order: Vec<usize> = (0..settings.len()).collect();
    order.sort_by(|&a, &b| median(a).total_cmp(&median(b)));
    for si in order {
        let s = &settings[si];
        let (c, lo, hi) = paired(si);
        let mark = if si == default {
            "the default"
        } else if hi < 0.0 {
            "faster"
        } else if lo > 0.0 {
            "slower"
        } else {
            ""
        };
        print!(
            "{:7}  {:5}  {:8}  {:8}  {:5.2}  {:+6.1}% ({:+.1}%, {:+.1}%) {mark:11}",
            s.walk,
            s.lanes,
            pf_name(s.pf),
            s.segments,
            cb(median(si)),
            100.0 * c,
            100.0 * lo,
            100.0 * hi
        );
        if let Some(c) = &counters {
            // unscaled counts over the time the group ran, rated over the bytes that time
            // covered (all of them, where it was never multiplexed out)
            let (enabled, running) = cover[si];
            if running == 0 {
                print!("  {:>52}", "not counted (the group never ran)");
            } else {
                let counted = running as f64 / enabled as f64;
                let per = |i: usize| counts[si][i] as f64 / (n as f64 * counted);
                let or_na = |i: usize, v: String| if c.has(i) { v } else { "n/a".to_string() };
                print!(
                    "  {:5.2}  {:5.2}  {:>11}  {:>14}  {:6.1}%",
                    per(0),
                    per(1),
                    or_na(2, format!("{:.2}", 1e3 * per(2))),
                    or_na(3, format!("{:5.1}%", 100.0 * per(3) / per(1))),
                    100.0 * counted
                );
            }
        }
        println!();
    }
}

/// The process's own hardware counters (instructions, cycles, LLC misses, back-end stalls),
/// shared with examples/run_fed_stage.rs
#[path = "support/pmu.rs"]
mod pmu;
