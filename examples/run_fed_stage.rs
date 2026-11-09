//! The run-fed stage's diagnostic (docs/journal/2026-11-09-run-fed-stage.md): the MTF
//! stage and the T-vector build as one stage, today's against the run records', on one
//! thread, in decode order, phase by phase, paired by round. Research only, behind the
//! `run-fed` feature; the decoder is unchanged.
//!
//! ```text
//! run_fed_stage FILE.bz2 [ROUNDS] [GHZ]   (ROUNDS 2-31, default 15; GHZ the nominal clock
//!                                          for cycles a byte of output, default 2.8)
//! ```
//!
//! The arms, each a decode of every block in decode order:
//! - **A, the column** (`Plan::Workers`' stage, CI's): `mtf::mtf` into L, then `bwt::build`;
//! - **C, the column with checkpoints** (the pipelined leader's MTF): `mtf::mtf_checkpoints`,
//!   then `bwt::build`. Its MTF row is what the pipelined team's leader runs today;
//! - **B, the run records:** on a block `run_stage::takes_runs` accepts, `mtf::mtf_runs`
//!   (emit), `run_stage::sort`, `run_stage::fill`; on any other, the fallback, A's stage;
//! - **D, the floor** (a mechanism probe, no candidate): A's MTF, then `run_stage::floor`,
//!   the table's n entries stored in order and nothing else (phase `floor`), then
//!   `bwt::build` over the buffer the floor left resident (phase `warm bld`). The floor
//!   against A's build bounds what any build storing the same table can save; the warm
//!   build against A's tells whether the table's write misses bind the build.
//!
//! Each block's stage is timed phase by phase as the decode meets it; the walk, RLE1 and the
//! CRC then run untimed on the arm's own table (the CRC checked), so each arm's stage meets
//! the caches as a decode leaves them. The phases (each arm's figure is its sum over the
//! blocks in a round):
//! - `front`: MTF (A, C) or emit (B), on the blocks B takes by records;
//! - `sort`: B's sort (0 for A and C);
//! - `build`: `bwt::build` (A, C) or B's fill, on those blocks;
//! - `fallback`: the whole stage on the blocks B leaves to the column (A's stage in every
//!   arm, C's with checkpoints).
//!
//! The arms' order rotates by round. Each ratio is paired by round (arm over arm, below 1
//! the first is cheaper): the geometric mean over the rounds and its 95% t-interval. Where
//! the process may count its own events, one more round of each arm, untimed, counts each
//! phase's instructions, cycles, LLC misses and back-end stalls.
//!
//! **Validity**, checked on every block before any timing, a failure a panic: B's table is
//! A's word for word, from the records and through `run_stage::build`; B's counts are A's;
//! its records are exactly L's maximal runs (closed by n), its cells fewer than 2r; the
//! walk of B's table, then RLE1, gives the block's stored CRC; and the first file's whole
//! output is `mibz2::decode`'s. Then each timed pass checks each block's CRC.
//!
//! The files: the one given (Wikidata on CI), then the test corpora's synthetic inputs
//! (tests/common's `corpora(1)`, compressed at level 9), each reported on its own, for the
//! fallback's cost where the records don't pay. The readings are computed against the
//! registration's thresholds, and labelled UNJUDGED: only the registered run is judged.

#[path = "../tests/common/mod.rs"]
mod common;
#[path = "support/pmu.rs"]
mod pmu;

use std::time::Instant;

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk_segments_room, SegScratch, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, run_stage, Kernels};

const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

const PHASES: [&str; 4] = ["front", "sort", "build", "fallback"];
/// D's phases 1 and 2: the floor, then the build over the table the floor left resident
const PHASES_D: [&str; 4] = ["front", "floor", "warm bld", "fallback"];
const ARMS: [&str; NARMS] = ["A column", "B records", "C column+ckpt", "D floor"];
const NARMS: usize = 4;
/// A round's times: [arm][phase], seconds summed over the blocks
type Round = [[f64; 4]; NARMS];
/// A paired ratio's geometric mean and 95% interval
type Interval = (f64, f64, f64);

/// One arm's buffers, kept from block to block as a worker keeps them
struct Bufs {
    ll: Vec<u8>,
    counts: [u32; 256],
    ck: Vec<[u32; 256]>,
    recs: Vec<u32>,
    s: run_stage::Scratch,
    tt: TVector,
    seg: SegScratch,
    walked: Vec<u8>,
    out: Vec<u8>,
}

impl Default for Bufs {
    fn default() -> Self {
        Bufs {
            ll: Vec::new(),
            counts: [0; 256],
            ck: Vec::new(),
            recs: Vec::new(),
            s: run_stage::Scratch::default(),
            tt: TVector::new(),
            seg: SegScratch::default(),
            walked: Vec::new(),
            out: Vec::new(),
        }
    }
}

/// Block `b`'s stage on arm `arm`, each phase wrapped by `time(phase, ..)`; then the walk,
/// RLE1 and the CRC, untimed, the CRC checked
fn stage(
    arm: usize,
    b: &Symbols,
    x: &mut Bufs,
    k: Kernels,
    time: &mut dyn FnMut(usize, &mut dyn FnMut()),
) {
    let runs = run_stage::takes_runs(b);
    let Bufs {
        ll,
        counts,
        ck,
        recs,
        s,
        tt,
        ..
    } = x;
    match (arm, runs) {
        (1, true) => {
            let mut n = 0;
            time(0, &mut || n = mtf::mtf_runs(b, recs, counts, k).unwrap());
            let mut ok = false;
            time(1, &mut || ok = run_stage::sort(recs, counts, n, s));
            assert!(ok, "the records tile the block");
            time(2, &mut || run_stage::fill(recs, s, n, tt, k));
        }
        (3, true) => {
            time(0, &mut || {
                mtf::mtf(b, ll, counts, k).unwrap();
            });
            time(1, &mut || run_stage::floor(ll.len(), tt, k));
            time(2, &mut || build(ll, counts, tt));
        }
        (_, true) => {
            time(0, &mut || {
                if arm == 0 {
                    mtf::mtf(b, ll, counts, k).unwrap();
                } else {
                    mtf::mtf_checkpoints(b, ll, counts, ck, k).unwrap();
                }
            });
            time(2, &mut || build(ll, counts, tt));
        }
        (_, false) => time(3, &mut || {
            if arm == 2 {
                mtf::mtf_checkpoints(b, ll, counts, ck, k).unwrap();
            } else {
                mtf::mtf(b, ll, counts, k).unwrap();
            }
            build(ll, counts, tt);
        }),
    }
    walk_segments_room(&mut x.tt, b.orig_ptr, &mut x.walked, &mut x.seg);
    x.out.clear();
    rle::unrle(&x.walked, &mut x.out, k).unwrap();
    assert_eq!(crc::crc32(&x.out, k), b.crc, "a block's CRC");
}

/// Every block checked (the module doc's validity), before any timing; returns the decoded
/// bytes and (blocks by records, rows by records, records, the fallback's blocks, rows,
/// cells by records)
fn validate(blocks: &[Symbols], k: Kernels) -> (Vec<u8>, [usize; 6]) {
    let (mut a, mut x) = (Bufs::default(), Bufs::default());
    let mut decoded = Vec::new();
    let mut tally = [0usize; 6];
    for (i, b) in blocks.iter().enumerate() {
        let n = mtf::mtf(b, &mut a.ll, &mut a.counts, k).unwrap();
        build(&a.ll, &a.counts, &mut a.tt);
        let rn = mtf::mtf_runs(b, &mut x.recs, &mut x.counts, k).unwrap();
        assert_eq!(
            (rn, x.counts),
            (n, a.counts),
            "block {i}: the records' length, counts"
        );
        let mut want = Vec::new();
        for (j, &c) in a.ll.iter().enumerate() {
            if j == 0 || c != a.ll[j - 1] {
                want.push((j as u32) << 8 | c as u32);
            }
        }
        want.push((n as u32) << 8);
        assert!(
            x.recs == want,
            "block {i}: the records are L's maximal runs"
        );
        assert!(
            run_stage::sort(&x.recs, &x.counts, n, &mut x.s),
            "block {i}: sort"
        );
        run_stage::fill(&x.recs, &x.s, n, &mut x.tt, k);
        assert!(
            x.tt.as_slice() == a.tt.as_slice(),
            "block {i}: the run-fed table"
        );
        // and the stage as B runs it, by whichever path
        let (bn, by_runs) = run_stage::build(
            b,
            &mut x.recs,
            &mut x.s,
            &mut x.ll,
            &mut x.counts,
            &mut x.tt,
            k,
        )
        .unwrap();
        assert_eq!(bn, n);
        assert!(
            x.tt.as_slice() == a.tt.as_slice(),
            "block {i}: the stage's table"
        );
        if by_runs {
            tally[0] += 1;
            tally[1] += n;
            tally[2] += x.recs.len() - 1;
        } else {
            tally[3] += 1;
        }
        tally[4] += n;
        if by_runs {
            let c = cells(&x.recs, &x.s.fo);
            let r = x.recs.len() - 1;
            assert!(n == 0 || c < 2 * r, "block {i}: {c} cells, {r} runs");
            tally[5] += c;
        }
        walk_segments_room(&mut x.tt, b.orig_ptr, &mut x.walked, &mut x.seg);
        x.out.clear();
        rle::unrle(&x.walked, &mut x.out, k).unwrap();
        assert_eq!(crc::crc32(&x.out, k), b.crc, "block {i}: the CRC");
        decoded.extend_from_slice(&x.out);
    }
    (decoded, tally)
}

/// A file's figures for the readings
struct Reading {
    name: String,
    stage: Option<Interval>,
    lead: Option<Interval>,
    floor: Option<Interval>,
    warm: Option<Interval>,
    fill_per_cell: f64,
    cells_per_row: f64,
}

/// The cells of a block's sweep: the distinct starts of its F intervals and its L runs
fn cells(recs: &[u32], fo: &[[u32; 2]]) -> usize {
    let (mut i, mut j, mut c) = (0, 0, 0);
    let (r, f) = (recs.len() - 1, fo.len() - 1);
    while i < r || j < f {
        let x = if i < r { recs[i] >> 8 } else { u32::MAX };
        let y = if j < f { fo[j][0] } else { u32::MAX };
        let m = x.min(y);
        i += (x == m) as usize;
        j += (y == m) as usize;
        c += 1;
    }
    c
}

fn ratio(
    rounds: &[Round],
    num: &dyn Fn(&Round) -> f64,
    den: &dyn Fn(&Round) -> f64,
) -> Option<Interval> {
    let logs: Vec<f64> = rounds.iter().map(|r| (num(r) / den(r)).ln()).collect();
    if logs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let m = logs.iter().sum::<f64>() / logs.len() as f64;
    let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (logs.len() - 1) as f64;
    let h = T95[logs.len() - 2] * (var / logs.len() as f64).sqrt();
    Some((m.exp(), (m - h).exp(), (m + h).exp()))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: run_fed_stage FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(15, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");
    let k = Kernels::detect();
    let mut counters = pmu::Counters::open();

    let mut files = vec![(path.clone(), std::fs::read(path).unwrap())];
    for (name, data) in common::corpora(1) {
        files.push((format!("synthetic {name}"), common::compress(&data, 9)));
    }
    println!(
        "run_fed_stage: {rounds} rounds, kernels {k:?}, c/B of output at {ghz} GHz; ratios are arm over \
         arm, paired by round, geometric mean (95% interval); below 1 the first is cheaper"
    );
    match &counters {
        Some(c) => println!(
            "counters: {} (one untimed round an arm)",
            c.names().join(", ")
        ),
        None => println!("counters: unavailable here ({})", pmu::why()),
    }
    let mut readings = Vec::new();
    for (fi, (name, data)) in files.iter().enumerate() {
        let mut blocks = Vec::new();
        let mut sc = Scanner::new(SliceSource(data));
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
            blocks.push(symbols);
        }
        let (decoded, tally) = validate(&blocks, k);
        if fi == 0 {
            assert!(
                decoded == mibz2::decode(data).unwrap(),
                "the output differs from decode's"
            );
        }
        let n = decoded.len();
        drop(decoded);

        // times[round][arm][phase], seconds summed over the blocks
        let mut times: Vec<Round> = vec![[[0f64; 4]; NARMS]; rounds];
        let mut bufs: [Bufs; NARMS] = Default::default();
        for (r, round) in times.iter_mut().enumerate() {
            for j in 0..NARMS {
                let arm = (j + r) % NARMS;
                let t = &mut round[arm];
                for b in &blocks {
                    stage(arm, b, &mut bufs[arm], k, &mut |p, f| {
                        let s = Instant::now();
                        f();
                        t[p] += s.elapsed().as_secs_f64();
                    });
                }
            }
        }
        // counts[arm][phase][event], and the group's time enabled and running
        let mut counts = [[[0u64; pmu::EVENTS]; 4]; NARMS];
        let mut cover = [[(0u64, 0u64); 4]; NARMS];
        if let Some(c) = counters.as_mut() {
            for arm in 0..NARMS {
                for b in &blocks {
                    stage(arm, b, &mut bufs[arm], k, &mut |p, f| {
                        c.start();
                        f();
                        let (v, e, rn) = c.stop();
                        for (t, v) in counts[arm][p].iter_mut().zip(v) {
                            *t += v;
                        }
                        cover[arm][p].0 += e;
                        cover[arm][p].1 += rn;
                    });
                }
            }
        }

        println!();
        println!(
            "{name}: {} blocks, {:.1} MB of output; by records {} blocks ({:.1}% of rows), r/n \
             {:.4} there; fallback {} blocks",
            blocks.len(),
            n as f64 / 1e6,
            tally[0],
            100.0 * tally[1] as f64 / tally[4] as f64,
            tally[2] as f64 / tally[1].max(1) as f64,
            tally[3]
        );
        let cr = |s: f64| s * ghz * 1e9 / n as f64;
        let median = |arm: usize, p: usize| {
            let mut v: Vec<f64> = times.iter().map(|r| r[arm][p]).collect();
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        print!("  {:14}", "c/B, median");
        for p in PHASES {
            print!("  {p:>8}");
        }
        println!("     stage");
        for (arm, an) in ARMS.iter().enumerate() {
            if arm == 3 {
                print!("  {:14}", "");
                for p in PHASES_D {
                    print!("  {p:>8}");
                }
                println!();
            }
            print!("  {an:14}");
            for p in 0..4 {
                print!("  {:8.3}", cr(median(arm, p)));
            }
            let mut v: Vec<f64> = times.iter().map(|r| r[arm].iter().sum()).collect();
            v.sort_by(f64::total_cmp);
            println!("  {:8.3}", cr(v[v.len() / 2]));
        }
        if let Some(c) = &counters {
            println!(
                "  counters a byte of output, each phase: ins, cyc, IPC, LLC misses a kB, \
                 back-end stall %, counted % (below 100 multiplexed: exploratory)"
            );
            for (arm, an) in ARMS.iter().enumerate() {
                let names = if arm == 3 { PHASES_D } else { PHASES };
                for (p, pn) in names.iter().enumerate() {
                    let (e, rn) = cover[arm][p];
                    if rn == 0 {
                        continue;
                    }
                    let counted = rn as f64 / e as f64;
                    let per = |i: usize| counts[arm][p][i] as f64 / (n as f64 * counted);
                    let na = |i: usize, v: String| if c.has(i) { v } else { "n/a".into() };
                    println!(
                        "    {an:14} {pn:8}  {:6.3}  {:6.3}  {:5.2}  {:>8}  {:>6}  {:5.1}",
                        per(0),
                        per(1),
                        per(0) / per(1),
                        na(2, format!("{:.3}", 1e3 * per(2))),
                        na(3, format!("{:.1}", 100.0 * per(3) / per(1))),
                        100.0 * counted
                    );
                }
            }
        }
        let total = |a: usize| move |r: &Round| r[a].iter().sum::<f64>();
        let ph = |a: usize, ps: &'static [usize]| {
            move |r: &Round| ps.iter().map(|&p| r[a][p]).sum::<f64>()
        };
        let rows: [(&str, Option<Interval>); 8] = [
            (
                "R_stage  B/A, the whole stage (primary)",
                ratio(&times, &total(1), &total(0)),
            ),
            (
                "R_front  B emit / A MTF, records' blocks",
                ratio(&times, &ph(1, &[0]), &ph(0, &[0])),
            ),
            (
                "R_build  B sort+fill / A build",
                ratio(&times, &ph(1, &[1, 2]), &ph(0, &[2])),
            ),
            (
                "R_lead   B emit+sort / C MTF+ckpt",
                ratio(&times, &ph(1, &[0, 1]), &ph(2, &[0])),
            ),
            (
                "R_emit   B emit / C MTF+ckpt",
                ratio(&times, &ph(1, &[0]), &ph(2, &[0])),
            ),
            (
                "R_fb     B/A on the fallback's blocks",
                ratio(&times, &ph(1, &[3]), &ph(0, &[3])),
            ),
            (
                "R_floor  D floor / A build",
                ratio(&times, &ph(3, &[1]), &ph(0, &[2])),
            ),
            (
                "R_warm   D build, table resident / A build",
                ratio(&times, &ph(3, &[2]), &ph(0, &[2])),
            ),
        ];
        for (label, r) in &rows {
            match r {
                Some((g, lo, hi)) => println!("  {label:42} {g:.4} ({lo:.4}, {hi:.4})"),
                None => println!("  {label:42} n/a (no such blocks)"),
            }
        }
        readings.push(Reading {
            name: name.clone(),
            stage: rows[0].1,
            lead: rows[3].1,
            floor: rows[6].1,
            warm: rows[7].1,
            fill_per_cell: median(1, 2) * ghz * 1e9 / tally[5].max(1) as f64,
            cells_per_row: tally[5] as f64 / tally[1].max(1) as f64,
        });
    }

    // The registration's readings (docs/journal/2026-11-09-run-fed-stage.md), computed
    // mechanically; UNJUDGED unless this is the registered run
    println!();
    println!("readings (UNJUDGED unless this is the registered run):");
    let w = &readings[0];
    let (g, lo, hi) = w.stage.expect("the first file has blocks");
    let worst = readings[1..]
        .iter()
        .filter_map(|r| r.stage.map(|x| (r.name.clone(), x.0)))
        .fold((String::new(), 0f64), |a, b| if b.1 > a.1 { b } else { a });
    let fallback_ok = worst.1 <= 1.02;
    let q1 = if g <= 0.90 && hi < 1.0 && fallback_ok {
        "FEASIBLE: a one-worker production prototype may be registered"
    } else if lo > 0.95 {
        "STOP: no one-worker stage saving of 5% or more"
    } else if !fallback_ok {
        "STOP: the fallback costs more than 2% on a synthetic corpus"
    } else {
        "STOP, INCONCLUSIVE: no saving shown to the gate (no re-run without a new registration)"
    };
    let q2 = match w.lead {
        Some((g, _, hi)) if g <= 0.90 && hi < 1.0 => {
            "LEADER LEAD: the next question is a pipelined prototype, registered on its own"
        }
        Some((_, lo, _)) if lo > 0.95 => "NO LEADER LEAD",
        Some(_) => "NO LEADER LEAD, INCONCLUSIVE",
        None => "n/a",
    };
    println!("  Q1 R_stage {g:.4} ({lo:.4}, {hi:.4}); gate: geomean <= 0.90 and hi < 1");
    println!(
        "  Q1 worst synthetic corpus R_stage {:.4} ({}); gate: geomean <= 1.02",
        worst.1, worst.0
    );
    println!("  Q1 reading: {q1}");
    if let Some((g, lo, hi)) = w.lead {
        println!("  Q2 R_lead {g:.4} ({lo:.4}, {hi:.4}); gate: geomean <= 0.90 and hi < 1");
    }
    println!("  Q2 reading: {q2}");
    let tell = |r: Option<Interval>, t: f64, yes: &str, no: &str| match r {
        Some((g, ..)) if g >= t => format!("{g:.3}: {yes}"),
        Some((g, ..)) => format!("{g:.3}: {no}"),
        None => "n/a".into(),
    };
    println!(
        "  M1 R_warm {}",
        tell(
            w.warm,
            0.95,
            "the build is not bound by its table's write misses",
            "the table's write misses cost the build"
        )
    );
    println!(
        "  M2 R_floor {}",
        tell(
            w.floor,
            0.80,
            "the build is near its stores' floor",
            "the build's cost is mostly above its stores' floor"
        )
    );
    println!(
        "  M3 fill {:.1} cycles a cell (cells/row {:.4}): the iterators' chain if near a load's \
         latency plus a few",
        w.fill_per_cell, w.cells_per_row
    );
}
