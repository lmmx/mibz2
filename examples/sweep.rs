//! The walk's lane count, segment count and prefetch, swept on a file's blocks on one
//! thread: `walk_segments_room_with::<L, PF>` at each setting, every output checked against
//! the plain walk's. A diagnostic, for choosing settings to try: the CI benchmark (the whole
//! decoder, A/B) decides what is faster.
//!
//! ```text
//! sweep FILE.bz2 [ROUNDS] [GHZ]
//! ```
//!
//! Each block's T-vector is built once, then each round walks it at every setting, the
//! settings' order rotated block by block so no setting always runs first after the build.
//! A setting's round is its time summed over the blocks. Against the default (the
//! settings `Segmented` uses), each round's ratio is paired, and the geometric mean of the
//! rounds' ratios is given with its 95% interval (a t-interval on the logs).

use std::time::Instant;

use mibz2::bwt::{build, walk, walk_segments_room_with, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize);

macro_rules! walks {
    ($($l:literal),*) => {
        [$(
            ($l, true, walk_segments_room_with::<$l, true> as Walk),
            ($l, false, walk_segments_room_with::<$l, false> as Walk),
        )*]
    };
}

const SEGMENTS: [usize; 4] = [256, 512, 1024, 2048];
// The settings `Segmented` uses (bwt.rs: FLAGGED_LANES, PREFETCH, FLAGGED_SEGMENTS)
const DEFAULT: (usize, bool, usize) = (32, true, 512);

// Two-sided 95% t quantiles for 1 to 30 degrees of freedom
const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: sweep FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(5, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    let lanes = walks!(8, 16, 24, 32, 40, 48, 64);
    let settings: Vec<(usize, bool, usize, Walk)> = lanes
        .iter()
        .flat_map(|&(l, pf, f)| SEGMENTS.iter().map(move |&s| (l, pf, s, f)))
        .collect();
    let default = settings
        .iter()
        .position(|&(l, pf, s, _)| (l, pf, s) == DEFAULT)
        .unwrap();

    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }

    // times[round][setting]: seconds summed over the blocks
    let mut times = vec![vec![0f64; settings.len()]; rounds];
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out) = (Vec::new(), Vec::new());
    let mut n = 0usize;
    for (bi, b) in blocks.iter().enumerate() {
        mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut plain);
        n += plain.len();
        for &(l, pf, s, f) in &settings {
            f(&mut tt, b.orig_ptr, &mut out, &mut seg, s);
            assert!(
                out == plain,
                "block {bi}: {l} lanes, prefetch {pf}, {s} segments"
            );
        }
        for (r, round) in times.iter_mut().enumerate() {
            for j in 0..settings.len() {
                let si = (j + bi + r) % settings.len();
                let (_, _, s, f) = settings[si];
                let t = Instant::now();
                f(&mut tt, b.orig_ptr, &mut out, &mut seg, s);
                round[si] += t.elapsed().as_secs_f64();
            }
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
         against {} lanes, prefetch {}, {} segments",
        blocks.len(),
        n as f64 / 1e6,
        DEFAULT.0,
        DEFAULT.1,
        DEFAULT.2
    );
    println!("lanes  prefetch  segments    c/B  change (95% interval)");
    let mut order: Vec<usize> = (0..settings.len()).collect();
    order.sort_by(|&a, &b| median(a).total_cmp(&median(b)));
    for si in order {
        let (l, pf, s, _) = settings[si];
        let (c, lo, hi) = paired(si);
        let mark = if si == default {
            "  (the default)"
        } else if hi < 0.0 {
            "  faster"
        } else if lo > 0.0 {
            "  slower"
        } else {
            ""
        };
        println!(
            "{l:5}  {:8}  {s:8}  {:5.2}  {:+6.1}% ({:+.1}%, {:+.1}%){mark}",
            if pf { "yes" } else { "no" },
            cb(median(si)),
            100.0 * c,
            100.0 * lo,
            100.0 * hi
        );
    }
}
