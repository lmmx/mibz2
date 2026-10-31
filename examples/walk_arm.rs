//! The room walk's research arm against the shipped walk, paired, on a file's blocks, on one
//! thread (docs/journal/2026-10-31-walk-unroll-protocol.md §5). A diagnostic trial, not the
//! decoder's A/B.
//!
//! ```text
//! walk_arm FILE.bz2 [ROUNDS] [GHZ]
//! ```
//!
//! Three arms, each the complete walk (setup, passes, tail and assembly) at 32 lanes,
//! prefetch into L1 and 512 segments: A, the shipped `walk_segments_room_with_prefetch`; B,
//! the arm `walk_segments_room_unrolled_with_prefetch`; A2, the shipped walk again (the A/A
//! null). Each arm's output is checked against the plain walk on every block first.
//!
//! A round decodes the file once for each arm, as `decode` does on one thread (block by
//! block: MTF, the build, the walk timed, RLE1), so each walk meets the caches as a decode
//! leaves them. Round r runs the arms in permutation r mod 6 of (A, B, A2). An arm's round
//! is its walk time summed over the blocks. Printed: one `ROUND` line a round (seconds per
//! arm), then a `SUMMARY` JSON line: each arm's median c/B at GHZ and ns/B, and B/A and
//! A2/A as geometric means of the paired per-round ratios with two-sided 95% t-intervals
//! on the logs.

use std::time::Instant;

use mibz2::bwt::{
    build, walk, walk_segments_room_unrolled_with_prefetch, walk_segments_room_with_prefetch,
    SegScratch, TVector, PF_L1,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, rle, Kernels};

type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize);

const SEGMENTS: usize = 512;
const ARMS: [&str; 3] = ["A", "B", "A2"];
const PERMS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

// Two-sided 95% t quantiles for 1 to 30 degrees of freedom
const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: walk_arm FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(20, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();
    let walks: [Walk; 3] = [
        walk_segments_room_with_prefetch::<32, PF_L1>,
        walk_segments_room_unrolled_with_prefetch::<32, PF_L1>,
        walk_segments_room_with_prefetch::<32, PF_L1>,
    ];

    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }

    let (mut ll, mut freq) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out, mut bytes) = (Vec::new(), Vec::new(), Vec::new());
    let mut n = 0usize;
    for (bi, b) in blocks.iter().enumerate() {
        mtf::mtf(b, &mut ll, &mut freq, k).unwrap();
        build(&ll, &freq, &mut tt);
        walk(&tt, b.orig_ptr, &mut plain);
        n += plain.len();
        for (a, f) in walks.iter().enumerate() {
            f(&mut tt, b.orig_ptr, &mut out, &mut seg, SEGMENTS);
            assert!(
                out == plain,
                "block {bi}: arm {} differs from the plain walk",
                ARMS[a]
            );
        }
    }
    println!(
        "CHECK\t{} blocks\t{n} bytes\tevery arm equals the plain walk",
        blocks.len()
    );

    let mut times = vec![[0f64; 3]; rounds];
    for (r, round) in times.iter_mut().enumerate() {
        for &a in &PERMS[r % 6] {
            for b in &blocks {
                mtf::mtf(b, &mut ll, &mut freq, k).unwrap();
                build(&ll, &freq, &mut tt);
                let t = Instant::now();
                walks[a](&mut tt, b.orig_ptr, &mut out, &mut seg, SEGMENTS);
                round[a] += t.elapsed().as_secs_f64();
                bytes.clear();
                rle::unrle(&out, &mut bytes, k).unwrap();
            }
        }
        println!(
            "ROUND\t{r}\t{:?}\t{:.6}\t{:.6}\t{:.6}",
            PERMS[r % 6],
            round[0],
            round[1],
            round[2]
        );
    }

    let median = |a: usize| {
        let mut v: Vec<f64> = times.iter().map(|t| t[a]).collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let paired = |a: usize| {
        let logs: Vec<f64> = times.iter().map(|t| (t[a] / t[0]).ln()).collect();
        let m = logs.iter().sum::<f64>() / rounds as f64;
        let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (rounds - 1) as f64;
        let h = T95[rounds - 2] * (var / rounds as f64).sqrt();
        (m.exp(), (m - h).exp(), (m + h).exp())
    };
    let arm = |a: usize| {
        let s = median(a);
        format!(
            "\"{}\": {{\"c_per_B\": {:.4}, \"ns_per_B\": {:.4}}}",
            ARMS[a],
            s * ghz * 1e9 / n as f64,
            s * 1e9 / n as f64
        )
    };
    let ratio = |a: usize| {
        let (m, lo, hi) = paired(a);
        format!("{{\"ratio\": {m:.5}, \"lower95\": {lo:.5}, \"upper95\": {hi:.5}}}")
    };
    println!(
        "SUMMARY\t{{\"file\": \"{path}\", \"blocks\": {}, \"bytes\": {n}, \"rounds\": {rounds}, \
         \"ghz\": {ghz}, \"arms\": {{{}, {}, {}}}, \"B/A\": {}, \"A2/A\": {}}}",
        blocks.len(),
        arm(0),
        arm(1),
        arm(2),
        ratio(1),
        ratio(2)
    );
}
