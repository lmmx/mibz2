//! The run-built pair table's diagnostic (docs/journal/2026-11-09-pair-from-runs.md):
//! #34's pair table built from the last column's run intersections, against #34's per-row
//! builder and against production's one-hop path, on one thread, paired by round.
//! Research only, behind the `pair-runs` feature; the decoder is unchanged.
//!
//! ```text
//! pair_runs FILE.bz2 [ROUNDS] [GHZ]   (ROUNDS 2-31, default 15, or 0 to validate only;
//!                                      GHZ the nominal clock for c/B, default 2.8)
//! env PAIR_RUNS_RAW   a file for the raw paired timings (CSV), one line a round, arm
//!                     and file
//!     PAIR_RUNS_HEAD  the head under test (40 hex digits), echoed and checked
//!     PAIR_RUNS_SPEC  bench/wikidata.json: the first file's size and decoded size are
//!                     checked against its corpus_bytes and decoded_bytes
//! ```
//!
//! The arms, each over every block in decode order, from the block's last column (made
//! by `mtf::mtf`, untimed, once):
//! - **(a), the build alone,** on the blocks the pair walk segments (n ≥ MIN_SEGMENTED):
//!   - `row`: #34's `build_pairs` and `pair_start` (its table, LF rows, first row);
//!   - `run`: `build_pairs_runs` (the same three, from the run intersections);
//!   - `one`: production's one-hop `bwt::build`, for reference (no LF rows, no first row).
//! - **(b), the whole entry point,** `InverseBwt::invert_in_place` over a fresh copy of the
//!   last column (the copy untimed), on every block:
//!   - `P`: production's `Segmented` (`build_one_pass`, then the 32-lane room walk);
//!   - `R`: `pair_runs::Pairs` with the run-built table and #34's consumer, on the blocks
//!     whose sampled run density is at most `pair_runs::ADMIT_DENSITY` (0.15); any other
//!     block goes to production's `Segmented` (the admission, fixed before timing);
//!   - `O`: the same with #34's own builder (the #34 kernel, a reference).
//!
//!   After each, untimed: RLE1 and the block's CRC, checked.
//!
//! The arms' order rotates by round. Each ratio is paired by round: the geometric mean over
//! rounds and its 95% t-interval; below 1 the first is cheaper.
//!
//! **Validity**, checked on every block before any timing, a failure a panic: the two
//! builders' words, LF rows (of 512 rows) and first row are identical, and the words are
//! ψ² by definition (from `bwt::build`'s table); the cells are fewer than 2r; the three
//! entry points' outputs are identical and give the block's CRC; the first file's whole
//! output is `mibz2::decode`'s. Then every timed pass checks every block's CRC. The run's
//! own validity (VALID) further needs every interval present, every total finite and
//! positive, the head, and the corpus identity; the workflow adds the corpus's SHA-256 and
//! each child's exit status. This process exits 3 when its part is not VALID.

#[path = "../tests/common/mod.rs"]
mod common;

use std::fmt::Write as _;
use std::time::Instant;

use mibz2::block::Symbols;
use mibz2::bwt::{build, InverseBwt, Segmented, TVector, MIN_SEGMENTED};
use mibz2::pair_runs::{
    build_pairs, build_pairs_runs, is_psi2, pair_start, setup, spare_bits, walk, Build,
    PairScratch, PairTable, Pairs, RunScratch, SpareBits,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

/// The production walk's segment count (`bwt`'s FLAGGED_SEGMENTS), for the validation's
/// direct pair walk
const FLAGGED: usize = 512;
const BUILD_ARMS: [&str; 3] = ["row", "run", "one"];
const WHOLE_ARMS: [&str; 3] = ["P", "R", "O"];
/// A round's seconds, summed over the blocks: (a)'s arms, then (b)'s
type Round = [f64; 6];
type Interval = (f64, f64, f64);

/// A block, as the timed arms take it
struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
    crc: u32,
}

/// 512 sorted distinct rows below n, as the walk's starts are (xorshift, seeded by n)
fn rows_for(n: usize) -> Vec<u32> {
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    let mut v: Vec<u32> = (0..511)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % n as u64) as u32
        })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// The arms' buffers, kept from block to block as a worker keeps them
#[derive(Default)]
struct Bufs {
    /// each builder its own table, as a worker keeps one (a shared one would be grown by
    /// one arm and reallocated by the other)
    t: PairTable,
    t_runs: PairTable,
    rs: RunScratch,
    lf: Vec<u32>,
    tt: TVector,
    io: Vec<u8>,
    out: Vec<u8>,
    prod: Segmented,
    runs: Pairs,
    rows: Pairs,
}

impl Bufs {
    fn new() -> Self {
        Bufs {
            runs: Pairs::new(Build::Runs),
            rows: Pairs::new(Build::Rows),
            ..Bufs::default()
        }
    }
}

/// (a)'s arm `arm` on block `b`
fn build_arm(arm: usize, b: &Block, rows: &[u32], x: &mut Bufs) -> u32 {
    match arm {
        0 => {
            build_pairs(&b.ll, &b.counts, &mut x.t, rows, &mut x.lf);
            pair_start(&b.ll, &x.t, b.orig_ptr)
        }
        1 => build_pairs_runs(
            &b.ll,
            &b.counts,
            &mut x.t_runs,
            &mut x.rs,
            rows,
            &mut x.lf,
            b.orig_ptr,
        )
        .expect("the counts are the column's own"),
        _ => {
            build(&b.ll, &b.counts, &mut x.tt);
            x.tt.len() as u32
        }
    }
}

/// (b)'s arm `arm` on block `b`, over x.io (a fresh copy of the column, made by the caller)
fn whole_arm(arm: usize, b: &Block, x: &mut Bufs) {
    let ok = match arm {
        0 => x.prod.invert_in_place(&mut x.io, &b.counts, b.orig_ptr),
        1 => x.runs.invert_in_place(&mut x.io, &b.counts, b.orig_ptr),
        _ => x.rows.invert_in_place(&mut x.io, &b.counts, b.orig_ptr),
    };
    assert!(ok, "in place");
}

fn check_crc(b: &Block, x: &mut Bufs, k: Kernels) {
    x.out.clear();
    rle::unrle(&x.io, &mut x.out, k).unwrap();
    assert_eq!(crc::crc32(&x.out, k), b.crc, "a block's CRC");
}

/// Totals over a file's validation
#[derive(Default)]
struct Tally {
    blocks: usize,
    paired: usize,
    rows: usize,
    paired_rows: usize,
    runs: usize,
    cells: usize,
    spare: SpareBits,
    flagged_blocks_max: usize,
}

/// Every block checked (the module doc's validity), before any timing; returns the
/// decoded bytes
fn validate(blocks: &[Block], k: Kernels, tally: &mut Tally) -> Vec<u8> {
    let mut x = Bufs::new();
    let mut walk_x = PairScratch::default();
    let (mut a, mut lfa) = (PairTable::default(), Vec::new());
    let mut decoded = Vec::new();
    for (i, b) in blocks.iter().enumerate() {
        let n = b.ll.len();
        tally.blocks += 1;
        tally.rows += n;
        if n >= MIN_SEGMENTED {
            let rows = rows_for(n);
            build_pairs(&b.ll, &b.counts, &mut a, &rows, &mut lfa);
            let r0a = pair_start(&b.ll, &a, b.orig_ptr);
            let r0b = build_arm(1, b, &rows, &mut x);
            assert!(a.words() == x.t_runs.words(), "block {i}: the words");
            assert_eq!(lfa, x.lf, "block {i}: the LF rows");
            assert_eq!(r0a, r0b, "block {i}: the first row");
            assert!(x.rs.cells < 2 * x.rs.runs, "block {i}: cells");
            assert!(
                is_psi2(&x.t_runs, &b.ll, &b.counts),
                "block {i}: the words are ψ²"
            );
            tally.paired += 1;
            tally.paired_rows += n;
            tally.runs += x.rs.runs;
            tally.cells += x.rs.cells;
            let c = spare_bits(&x.t_runs, r0b);
            let s = &mut tally.spare;
            s.n += c.n;
            s.dir_bytes += c.dir_bytes;
            s.old_bytes += c.old_bytes;
            s.flagged_blocks += c.flagged_blocks;
            s.flagged_rows += c.flagged_rows;
            s.steps += c.steps;
            s.flagged_steps += c.flagged_steps;
            s.ends_iters += c.ends_iters;
            tally.flagged_blocks_max = tally.flagged_blocks_max.max(c.flagged_blocks);
        }
        let mut outs = Vec::new();
        for arm in 0..3 {
            x.io.clear();
            x.io.extend_from_slice(&b.ll);
            whole_arm(arm, b, &mut x);
            outs.push(x.io.clone());
        }
        assert!(outs[0] == outs[1], "block {i}: R's output is P's");
        assert!(outs[0] == outs[2], "block {i}: O's output is P's");
        // and the run-built pair walk on every block it can segment, admitted or not
        if n >= MIN_SEGMENTED {
            let mut w = Vec::new();
            let r0 = setup(
                Build::Runs,
                &mut walk_x,
                &b.ll,
                &b.counts,
                b.orig_ptr,
                FLAGGED,
            );
            walk::<32, true>(&mut walk_x, r0, &mut w);
            assert!(
                outs[0] == w,
                "block {i}: the run-built pair walk's output is P's"
            );
        }
        check_crc(b, &mut x, k);
        decoded.extend_from_slice(&x.out);
    }
    decoded
}

fn ratio(rounds: &[Round], num: usize, den: usize) -> Option<Interval> {
    let logs: Vec<f64> = rounds.iter().map(|r| (r[num] / r[den]).ln()).collect();
    if logs.len() < 2 || logs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let m = logs.iter().sum::<f64>() / logs.len() as f64;
    let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (logs.len() - 1) as f64;
    let h = T95[logs.len() - 2] * (var / logs.len() as f64).sqrt();
    let iv = (m.exp(), (m - h).exp(), (m + h).exp());
    (iv.0.is_finite() && iv.1.is_finite() && iv.2.is_finite()).then_some(iv)
}

/// A paired difference's mean and 95% interval, in c/B
fn diff(rounds: &[Round], a: usize, b: usize, cb: &dyn Fn(f64) -> f64) -> Option<Interval> {
    let d: Vec<f64> = rounds.iter().map(|r| cb(r[a]) - cb(r[b])).collect();
    if d.len() < 2 || d.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let m = d.iter().sum::<f64>() / d.len() as f64;
    let var = d.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (d.len() - 1) as f64;
    let h = T95[d.len() - 2] * (var / d.len() as f64).sqrt();
    Some((m, m - h, m + h))
}

/// A number from a flat JSON object's top level (bench/wikidata.json), by key
fn json_number(text: &str, key: &str) -> Option<u64> {
    let at = text.find(&format!("\"{key}\""))?;
    let rest = text[at + key.len() + 2..].trim_start().strip_prefix(':')?;
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn fmt(r: Option<Interval>) -> String {
    match r {
        Some((g, lo, hi)) => format!("{g:.4} ({lo:.4}, {hi:.4})"),
        None => "MISSING".into(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: pair_runs FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(15, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!(
        rounds == 0 || (2..=31).contains(&rounds),
        "ROUNDS: 0, or 2 to 31"
    );
    let k = Kernels::detect();
    let head = std::env::var("PAIR_RUNS_HEAD").unwrap_or_default();
    let spec = std::env::var("PAIR_RUNS_SPEC")
        .ok()
        .map(|p| std::fs::read_to_string(p).unwrap());
    let mut raw = String::from("file,round,arm,seconds\n");
    let mut problems: Vec<String> = Vec::new();

    let head_ok = head.len() == 40 && head.bytes().all(|b| b.is_ascii_hexdigit());
    if !head_ok {
        problems.push(format!("head {head:?} is not 40 hex digits"));
    }
    let mut files = vec![(path.clone(), std::fs::read(path).unwrap())];
    for (name, data) in common::corpora(1) {
        files.push((format!("synthetic {name}"), common::compress(&data, 9)));
    }
    println!(
        "pair_runs: head {head}; {rounds} rounds, kernels {k:?}, c/B of output at {ghz} GHz; \
         ratios are arm over arm, paired by round, geometric mean (95% interval); below 1 \
         the first is cheaper"
    );
    // [file] = (name, Q1, M1 ratio, M1 c/B difference, Q2, O/P, R/O)
    let mut readings = Vec::new();
    for (fi, (name, data)) in files.iter().enumerate() {
        let mut blocks = Vec::new();
        let mut sc = Scanner::new(SliceSource(data));
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
            let b: Symbols = symbols;
            let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
            mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            blocks.push(Block {
                ll,
                counts,
                orig_ptr: b.orig_ptr,
                crc: b.crc,
            });
        }
        let mut tally = Tally::default();
        let decoded = validate(&blocks, k, &mut tally);
        if fi == 0 {
            assert!(
                decoded == mibz2::decode(data).unwrap(),
                "the output differs from decode's"
            );
            if let Some(spec) = &spec {
                let want = (
                    json_number(spec, "corpus_bytes"),
                    json_number(spec, "decoded_bytes"),
                );
                let got = (Some(data.len() as u64), Some(decoded.len() as u64));
                if want != got {
                    problems.push(format!("corpus identity: {got:?}, the spec says {want:?}"));
                }
                println!(
                    "corpus identity: {} bytes, {} decoded; spec {want:?}",
                    data.len(),
                    decoded.len()
                );
            } else {
                problems.push("no PAIR_RUNS_SPEC: corpus identity unchecked".into());
            }
        }
        let n = decoded.len();
        drop(decoded);
        let sp = &tally.spare;
        println!();
        println!(
            "{name}: {} blocks, {:.1} MB of output, {} rows; paired {} blocks ({} rows): r/n {:.4}, \
             cells/n {:.4}, cells/r {:.3} (all < 2r)",
            tally.blocks,
            n as f64 / 1e6,
            tally.rows,
            tally.paired,
            tally.paired_rows,
            tally.runs as f64 / tally.paired_rows.max(1) as f64,
            tally.cells as f64 / tally.paired_rows.max(1) as f64,
            tally.cells as f64 / tally.runs.max(1) as f64,
        );
        println!(
            "  spare-bit census (not built): directory {} B against the old lookup's {} B \
             ({:.0} and {:.0} a block); flagged blocks {} (max {} in a block of rows, bound 31), \
             distinct flagged rows {} ({:.3}%); one-chain walk: {} steps, {:.3}% on flagged \
             rows; the old lookup's correction loop {:.3} iterations a step",
            sp.dir_bytes,
            sp.old_bytes,
            sp.dir_bytes as f64 / tally.paired.max(1) as f64,
            sp.old_bytes as f64 / tally.paired.max(1) as f64,
            sp.flagged_blocks,
            tally.flagged_blocks_max,
            sp.flagged_rows,
            100.0 * sp.flagged_rows as f64 / sp.n.max(1) as f64,
            sp.steps,
            100.0 * sp.flagged_steps as f64 / sp.steps.max(1) as f64,
            sp.ends_iters as f64 / sp.steps.max(1) as f64,
        );
        if rounds == 0 {
            continue;
        }

        let rows: Vec<Vec<u32>> = blocks.iter().map(|b| rows_for(b.ll.len().max(1))).collect();
        let mut times: Vec<Round> = vec![[0f64; 6]; rounds];
        let mut x = Bufs::new();
        (x.runs.taken, x.runs.declined) = (0, 0);
        for (r, round) in times.iter_mut().enumerate() {
            for j in 0..3 {
                let arm = (j + r) % 3;
                for (b, rw) in blocks.iter().zip(&rows) {
                    if b.ll.len() < MIN_SEGMENTED {
                        continue;
                    }
                    let s = Instant::now();
                    let v = build_arm(arm, b, rw, &mut x);
                    round[arm] += s.elapsed().as_secs_f64();
                    std::hint::black_box(v);
                }
            }
            for j in 0..3 {
                let arm = (j + r) % 3;
                for b in &blocks {
                    x.io.clear();
                    x.io.extend_from_slice(&b.ll);
                    let s = Instant::now();
                    whole_arm(arm, b, &mut x);
                    round[3 + arm] += s.elapsed().as_secs_f64();
                    check_crc(b, &mut x, k);
                }
            }
            for (a, an) in BUILD_ARMS.iter().chain(&WHOLE_ARMS).enumerate() {
                let _ = writeln!(raw, "{name},{r},{an},{:.9}", round[a]);
            }
        }
        let cb = |s: f64| s * ghz * 1e9 / n as f64;
        let cb_paired = |s: f64| s * ghz * 1e9 / tally.paired_rows.max(1) as f64;
        let median = |a: usize| {
            let mut v: Vec<f64> = times.iter().map(|r| r[a]).collect();
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        for (a, t) in times.iter().flat_map(|r| r.iter().enumerate()) {
            // (the build arms run only on the blocks the pair walk segments)
            if a < 3 && tally.paired == 0 {
                continue;
            }
            if !(t.is_finite() && *t > 0.0) {
                problems.push(format!("{name}: arm {a} has a total of {t}"));
            }
        }
        println!(
            "  (a) build, c/B of the paired blocks' rows, median: row {:.3}, run {:.3}, one {:.3}",
            cb_paired(median(0)),
            cb_paired(median(1)),
            cb_paired(median(2))
        );
        let (taken, declined) = (x.runs.taken, x.runs.declined);
        println!(
            "  (b) whole entry point, c/B of output, median: P {:.3}, R {:.3}, O {:.3}; R took \
             {:.0} blocks a round by the pair walk, declined {:.0} (admission)",
            cb(median(3)),
            cb(median(4)),
            cb(median(5)),
            taken as f64 / rounds as f64,
            declined as f64 / rounds as f64
        );
        let q1 = ratio(&times, 1, 0);
        let m1 = ratio(&times, 1, 2);
        let m1d = if tally.paired > 0 {
            diff(&times, 1, 2, &cb_paired)
        } else {
            None
        };
        let q2 = ratio(&times, 4, 3);
        let op = ratio(&times, 5, 3);
        let ro = ratio(&times, 4, 5);
        println!("  Q1 R_build  run / row build              {}", fmt(q1));
        println!("  M1          run / one-hop build          {}", fmt(m1));
        match m1d {
            Some((m, lo, hi)) => {
                println!("  M1          run − one-hop build, c/B     {m:.3} ({lo:.3}, {hi:.3})")
            }
            None => println!("  M1          run − one-hop build, c/B     n/a (no paired blocks)"),
        }
        println!("  Q2 R_whole  R / P (run pair vs production) {}", fmt(q2));
        println!("  ref         O / P (#34 vs production)    {}", fmt(op));
        println!("  ref         R / O (run vs row, whole)    {}", fmt(ro));
        let paired_file = tally.paired > 0;
        for (label, r) in [("Q1", q1), ("M1", m1), ("Q2", q2), ("O/P", op), ("R/O", ro)] {
            let need = paired_file || matches!(label, "Q2");
            if need && r.is_none() {
                problems.push(format!("{name}: interval {label} missing"));
            }
        }
        readings.push((name.clone(), q1, m1d, q2));
    }

    if let Ok(p) = std::env::var("PAIR_RUNS_RAW") {
        std::fs::write(&p, &raw).unwrap();
        println!();
        println!(
            "raw paired timings: {p} ({} lines)",
            raw.lines().count() - 1
        );
    }
    if rounds == 0 {
        println!();
        println!("validation only: every block valid");
        return;
    }

    // The registration's readings (docs/journal/2026-11-09-pair-from-runs.md), computed
    // mechanically; UNJUDGED unless this is the registered run
    println!();
    println!("readings (UNJUDGED unless this is the registered run):");
    let (_, q1, m1d, q2) = &readings[0];
    if let Some((g, lo, hi)) = *q1 {
        let r = if g <= 0.50 && hi < 0.60 {
            "POSITIVE: the run intersections remove most of the old build's cost"
        } else if lo > 0.80 {
            "NEGATIVE STOP: the run-built pair table saves under 20% of the old build"
        } else {
            "INCONCLUSIVE (no re-run without a new registration)"
        };
        println!("  Q1 R_build {g:.4} ({lo:.4}, {hi:.4}); positive: <= 0.50 and hi < 0.60; stop: lo > 0.80");
        println!("  Q1 reading: {r}");
    }
    if let Some((m, lo, hi)) = *m1d {
        let r = if hi <= 1.0 {
            "MET: within one-pass + 1 c/B (the archive's reopening bound, here timed)"
        } else if lo > 1.0 {
            "NOT MET"
        } else {
            "UNRESOLVED"
        };
        println!("  M1 run − one-hop {m:.3} c/B ({lo:.3}, {hi:.3}): {r}");
    }
    let worst = readings[1..]
        .iter()
        .filter_map(|r| r.3.map(|x| (r.0.clone(), x.0)))
        .fold((String::new(), 0f64), |a, b| if b.1 > a.1 { b } else { a });
    if let Some((g, lo, hi)) = *q2 {
        let r = if g <= 0.95 && hi < 1.0 && worst.1 <= 1.05 {
            "POSITIVE: a production prototype may be registered (bench.yml's A/B decides)"
        } else if lo > 1.0 {
            "NEGATIVE STOP: the run-built pair with #34's consumer is slower than production"
        } else if g <= 0.95 && hi < 1.0 {
            "STOP: a synthetic corpus regresses by more than 5%"
        } else {
            "INCONCLUSIVE (no re-run without a new registration)"
        };
        println!("  Q2 R_whole {g:.4} ({lo:.4}, {hi:.4}); positive: <= 0.95, hi < 1, worst synthetic <= 1.05; stop: lo > 1");
        println!("  Q2 worst synthetic R_whole {:.4} ({})", worst.1, worst.0);
        println!("  Q2 reading: {r}");
    }
    println!();
    if problems.is_empty() {
        println!("VALID (this process's part): every interval present, totals finite and positive, head and corpus identity checked");
    } else {
        println!("NOT VALID:");
        for p in &problems {
            println!("  - {p}");
        }
        std::process::exit(3);
    }
}
