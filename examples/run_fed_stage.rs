//! The run-fed stage's diagnostic (docs/journal/2026-11-09-run-fed-stage.md): the MTF
//! stage and the T-vector build as one stage, today's against the run records', on one
//! thread, in decode order, paired by round. Research only, behind the `run-fed` feature;
//! the decoder is unchanged.
//!
//! ```text
//! run_fed_stage FILE.bz2 [ROUNDS] [GHZ]   (ROUNDS 2-31, default 15; GHZ the nominal clock
//!                                          for the nominal cycles below, default 2.8)
//! ```
//!
//! With `RUN_FED_RECORD=PATH` it writes the run's admission record there, as JSON: the
//! source identity (`RUN_FED_HEAD`, as the caller gives it), the rounds, kernels, host and
//! affinity, each corpus's SHA-256, size and path counts, and every round's raw times, arm
//! by arm, for `bench/run_fed_stage.py verify` to judge independently. It is written only
//! once every correctness check has passed.
//!
//! The arms, each a decode of every block in decode order:
//! - **A, the column** (`Plan::Workers`' stage, CI's): `mtf::mtf` into L, `bwt::build`;
//! - **C, the column with checkpoints** (the pipelined leader's MTF): `mtf::mtf_checkpoints`,
//!   then `bwt::build`;
//! - **B, the run records:** `run_stage::build`, the entry point itself: its selector
//!   `takes_runs`, then on a block it accepts `mtf::mtf_runs` (emit), `run_stage::sort`,
//!   `run_stage::fill`, and on any other the fallback, A's stage;
//! - **D, the store reference** (a mechanism probe, no candidate): A's MTF, then
//!   `run_stage::floor` (the table's n entries stored in order, nothing else), then
//!   `bwt::build` over the buffer it pre-touched.
//!
//! **Two sweeps a round,** each over every arm in a rotated order, each block's stage
//! followed by the walk, RLE1 and the CRC, untimed, on the arm's own table, the walk written
//! over the arm's `ll` as `Segmented::invert_in_place` writes it (so each arm keeps the
//! incumbent's buffers: no separate walk buffer):
//! 1. **the entry sweep:** each block's whole stage in one clock bracket, the arm's entry
//!    point as a decode calls it (B's `run_stage::build`, selector, checks and dispatch
//!    included). Summed by the block's path (the records' blocks, the fallback's), these
//!    are the judged figures: `R_stage`, `R_acc` and `R_fb`. Every arm has one bracket a
//!    block, so the clock's own cost is the same in each;
//! 2. **the phase sweep,** for attribution only: each phase in its own bracket (`front`:
//!    MTF or emit; `sort`; `build`: build or fill; `fallback`: the other blocks' stage).
//!    B takes three brackets on a records' block, A two, so the sum of phases is not the
//!    entry point's cost: `R_phases` against `R_stage` shows the difference.
//!
//! Each ratio is paired by round (arm over arm, below 1 the first is cheaper): the
//! geometric mean over the rounds and its 95% t-interval. Where the process may count its
//! own events, one more phase sweep, untimed, counts each phase's instructions, cycles,
//! LLC misses and back-end stalls.
//!
//! **Validity**, checked on every block before any timing, a failure a panic (and then no
//! record): B's table is A's word for word, from the records and through
//! `run_stage::build`; B's counts are A's; its records are exactly L's maximal runs (closed
//! by n), its cells fewer than 2r; the walk of B's table, then RLE1, gives the block's
//! stored CRC; and the first file's whole output is `mibz2::decode`'s. Then every timed
//! pass checks each block's CRC, and B's path is `takes_runs`'s.
//!
//! The files: the one given (Wikidata on CI), then the test corpora's synthetic inputs
//! (tests/common's `corpora(1)`, compressed at level 9; the text corpus is read from the
//! checkout's sources, so its bytes are the head's). The readings printed here are
//! UNJUDGED: only the verifier's, on the registered run's record, are judged.

#[path = "../tests/common/mod.rs"]
mod common;
#[path = "support/pmu.rs"]
mod pmu;
#[path = "support/sha256.rs"]
mod sha256;

use std::fmt::Write as _;
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
/// D's phases 1 and 2: the store reference, then the build over the buffer it touched
const PHASES_D: [&str; 4] = ["front", "floor", "pre-t bld", "fallback"];
const ARMS: [&str; NARMS] = ["A column", "B records", "C column+ckpt", "D floor"];
const NARMS: usize = 4;
/// A round's phase times: [arm][phase], seconds summed over the blocks
type Round = [[f64; 4]; NARMS];
/// A round's entry times: [arm][path], the records' blocks then the fallback's
type Entry = [[f64; 2]; NARMS];
/// A paired ratio's geometric mean and 95% interval
type Interval = (f64, f64, f64);

/// One arm's buffers, kept from block to block as a worker keeps them; the walk goes over
/// `ll`, as production's in-place inversion writes it
struct Bufs {
    ll: Vec<u8>,
    counts: [u32; 256],
    ck: Vec<[u32; 256]>,
    recs: Vec<u32>,
    s: run_stage::Scratch,
    tt: TVector,
    seg: SegScratch,
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
            out: Vec::new(),
        }
    }
}

/// The walk of the arm's table over its `ll`, RLE1 and the CRC, untimed, the CRC checked
fn walk_check(b: &Symbols, x: &mut Bufs, k: Kernels) {
    walk_segments_room(&mut x.tt, b.orig_ptr, &mut x.ll, &mut x.seg);
    x.out.clear();
    rle::unrle(&x.ll, &mut x.out, k).unwrap();
    assert_eq!(crc::crc32(&x.out, k), b.crc, "a block's CRC");
}

/// Block `b`'s whole stage on arm `arm`, by its entry point; returns whether B took the
/// records' path (false for the other arms)
fn entry(arm: usize, b: &Symbols, x: &mut Bufs, k: Kernels) -> bool {
    match arm {
        0 => {
            mtf::mtf(b, &mut x.ll, &mut x.counts, k).unwrap();
            build(&x.ll, &x.counts, &mut x.tt);
            false
        }
        1 => {
            let Bufs {
                ll,
                counts,
                recs,
                s,
                tt,
                ..
            } = x;
            run_stage::build(b, recs, s, ll, counts, tt, k).unwrap().1
        }
        2 => {
            mtf::mtf_checkpoints(b, &mut x.ll, &mut x.counts, &mut x.ck, k).unwrap();
            build(&x.ll, &x.counts, &mut x.tt);
            false
        }
        _ => {
            mtf::mtf(b, &mut x.ll, &mut x.counts, k).unwrap();
            run_stage::floor(x.ll.len(), &mut x.tt, k);
            build(&x.ll, &x.counts, &mut x.tt);
            false
        }
    }
}

/// Block `b`'s stage on arm `arm`, each phase wrapped by `time(phase, ..)`, for attribution;
/// then the walk, RLE1 and the CRC, untimed
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
            // the scratch's borrow moved into the bracket, so the `Sorted` outlives it
            let (rr, mut ss): (&[u32], _) = (recs, Some(s));
            let mut sorted = None;
            time(1, &mut || {
                sorted = ss.take().and_then(|s| run_stage::sort(rr, counts, n, s))
            });
            let t = sorted.expect("the records tile the block");
            time(2, &mut || run_stage::fill(&t, tt, k));
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
    walk_check(b, x, k);
}

/// A file's path counts, from the validation pass
#[derive(Default)]
struct Tally {
    blocks: usize,
    rows: usize,
    blocks_runs: usize,
    rows_runs: usize,
    records: usize,
    blocks_fallback: usize,
    cells: usize,
    /// the u32 slots B's AVX2 fill stores on the records' blocks, overstores included
    slots: usize,
}

/// Every block checked (the module doc's validity), before any timing; returns the decoded
/// bytes and the path counts
fn validate(blocks: &[Symbols], k: Kernels) -> (Vec<u8>, Tally) {
    let (mut a, mut x) = (Bufs::default(), Bufs::default());
    let mut decoded = Vec::new();
    let mut tally = Tally {
        blocks: blocks.len(),
        ..Tally::default()
    };
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
        let (c, slots) = {
            let t = run_stage::sort(&x.recs, &x.counts, n, &mut x.s)
                .unwrap_or_else(|| panic!("block {i}: sort"));
            run_stage::fill(&t, &mut x.tt, k);
            cells(t.recs(), t.fo())
        };
        assert!(
            x.tt.as_slice() == a.tt.as_slice(),
            "block {i}: the run-fed table"
        );
        let r = x.recs.len() - 1;
        assert!(n == 0 || c < 2 * r, "block {i}: {c} cells, {r} runs");
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
        assert_eq!(by_runs, run_stage::takes_runs(b), "block {i}: the path");
        assert!(
            x.tt.as_slice() == a.tt.as_slice(),
            "block {i}: the stage's table"
        );
        if by_runs {
            tally.blocks_runs += 1;
            tally.rows_runs += n;
            tally.records += r;
            tally.cells += c;
            tally.slots += slots;
        } else {
            tally.blocks_fallback += 1;
        }
        tally.rows += n;
        walk_check(b, &mut x, k);
        decoded.extend_from_slice(&x.out);
    }
    (decoded, tally)
}

/// The cells of a block's sweep (the distinct starts of its F intervals and its L runs),
/// and the u32 slots the AVX2 fill stores for them: 8 for each started 8 of a cell's rows
fn cells(recs: &[u32], fo: &[[u32; 2]]) -> (usize, usize) {
    let (mut i, mut j, mut c, mut slots) = (0, 0, 0, 0);
    let (r, f) = (recs.len() - 1, fo.len() - 1);
    let n = recs[r] >> 8;
    let mut start = 0u32;
    while i < r || j < f {
        let x = if i < r { recs[i] >> 8 } else { u32::MAX };
        let y = if j < f { fo[j][0] } else { u32::MAX };
        let m = x.min(y);
        if c > 0 {
            slots += (m - start).div_ceil(8) as usize * 8;
        }
        start = m;
        i += (x == m) as usize;
        j += (y == m) as usize;
        c += 1;
    }
    if c > 0 {
        slots += (n - start).div_ceil(8) as usize * 8;
    }
    (c, slots)
}

fn interval(logs: &[f64]) -> Option<Interval> {
    if logs.len() < 2 || logs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let m = logs.iter().sum::<f64>() / logs.len() as f64;
    let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (logs.len() - 1) as f64;
    let h = T95[logs.len() - 2] * (var / logs.len() as f64).sqrt();
    Some((m.exp(), (m - h).exp(), (m + h).exp()))
}

/// The paired ratio over the rounds; None where a round's figure is absent (0) or not
/// finite: the path has no blocks, or its times are missing
fn ratio<R>(rounds: &[R], num: &dyn Fn(&R) -> f64, den: &dyn Fn(&R) -> f64) -> Option<Interval> {
    let logs: Vec<f64> = rounds.iter().map(|r| (num(r) / den(r)).ln()).collect();
    interval(&logs)
}

/// The JSON for a float: finite, or the record is refused as it is written
fn num(x: f64) -> String {
    assert!(x.is_finite(), "a time that is not finite: {x}");
    format!("{x:e}")
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// One line of /proc/self/status or /proc/cpuinfo, or "unknown"
fn proc_field(path: &str, key: &str) -> String {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

/// A file's record: its identity, its path counts, its rounds' arm orders and raw times
struct FileRecord {
    name: String,
    bytes: usize,
    sha256: String,
    tally: Tally,
    order: Vec<[usize; NARMS]>,
    entry: Vec<Entry>,
    phases: Vec<Round>,
}

/// The run's record (`Run`'s identity, the files' records), as JSON
struct Run<'a> {
    head: Option<&'a str>,
    rounds: usize,
    ghz: f64,
    k: Kernels,
    host: String,
    affinity: String,
}

fn record(run: &Run, files: &[FileRecord]) -> String {
    let Run {
        head,
        rounds,
        ghz,
        k,
        ..
    } = *run;
    let mut o = String::new();
    let _ = write!(
        o,
        "{{\"schema\": \"run_fed_stage/1\", \"head\": {}, \"rounds\": {rounds}, \
         \"kernels\": {{\"avx2\": {}, \"clmul\": {}}}, \"ghz\": {}, \"host\": {}, \
         \"affinity\": {}, \"correctness\": \"passed\", \"arms\": [{}], \
         \"entry_paths\": [\"records\", \"fallback\"], \"phases\": [{}], \"files\": [",
        head.map_or("null".into(), json_str),
        k.avx2,
        k.clmul,
        num(ghz),
        json_str(&run.host),
        json_str(&run.affinity),
        ARMS.map(json_str).join(", "),
        PHASES.map(json_str).join(", "),
    );
    for (i, f) in files.iter().enumerate() {
        let t = &f.tally;
        let _ = write!(
            o,
            "{}\n  {{\"name\": {}, \"bytes\": {}, \"sha256\": \"{}\", \"blocks\": {}, \
             \"rows\": {}, \"blocks_runs\": {}, \"rows_runs\": {}, \"records\": {}, \
             \"blocks_fallback\": {}, \"cells\": {}, \"slots\": {},\n   \"order\": [{}],\n   \
             \"entry\": [{}],\n   \"phases\": [{}]}}",
            if i == 0 { "" } else { "," },
            json_str(&f.name),
            f.bytes,
            f.sha256,
            t.blocks,
            t.rows,
            t.blocks_runs,
            t.rows_runs,
            t.records,
            t.blocks_fallback,
            t.cells,
            t.slots,
            f.order
                .iter()
                .map(|r| format!("{r:?}"))
                .collect::<Vec<_>>()
                .join(", "),
            f.entry
                .iter()
                .map(|r| format!(
                    "[{}]",
                    r.iter()
                        .map(|a| format!("[{}, {}]", num(a[0]), num(a[1])))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join(",\n    "),
            f.phases
                .iter()
                .map(|r| format!(
                    "[{}]",
                    r.iter()
                        .map(|a| format!("[{}]", a.map(num).join(", ")))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join(",\n    "),
        );
    }
    o.push_str("\n]}\n");
    o
}

/// A file's figures for the readings
struct Reading {
    name: String,
    blocks_fallback: usize,
    stage: Option<Interval>,
    fb: Option<Interval>,
    lead: Option<Interval>,
    floor: Option<Interval>,
    warm: Option<Interval>,
    fill_nominal_per_cell: f64,
    fill_pmu_per_cell: Option<f64>,
    cells_per_row: f64,
    slots_per_row: f64,
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
    let head = std::env::var("RUN_FED_HEAD").ok();
    let record_path = std::env::var("RUN_FED_RECORD").ok();

    let mut files = vec![(path.clone(), std::fs::read(path).unwrap())];
    for (name, data) in common::corpora(1) {
        files.push((format!("synthetic {name}"), common::compress(&data, 9)));
    }
    println!(
        "run_fed_stage: {rounds} rounds, kernels {k:?}, head {}; nominal c/B of output (wall \
         time x {ghz} GHz, not counted cycles); ratios are arm over arm, paired by round, \
         geometric mean (95% interval); below 1 the first is cheaper",
        head.as_deref().unwrap_or("not given")
    );
    match &counters {
        Some(c) => println!(
            "counters: {} (one untimed phase sweep an arm)",
            c.names().join(", ")
        ),
        None => println!("counters: unavailable here ({})", pmu::why()),
    }
    let mut readings = Vec::new();
    let mut recs_out = Vec::new();
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
        let paths: Vec<bool> = blocks.iter().map(run_stage::takes_runs).collect();

        let mut entry_t: Vec<Entry> = vec![[[0f64; 2]; NARMS]; rounds];
        let mut times: Vec<Round> = vec![[[0f64; 4]; NARMS]; rounds];
        let mut order = Vec::with_capacity(rounds);
        let mut bufs: [Bufs; NARMS] = Default::default();
        for r in 0..rounds {
            let ord: [usize; NARMS] = std::array::from_fn(|j| (j + r) % NARMS);
            order.push(ord);
            // the entry sweep: one bracket a block, the judged figures
            for &arm in &ord {
                let x = &mut bufs[arm];
                for (b, &runs) in blocks.iter().zip(&paths) {
                    let s = Instant::now();
                    let by_runs = entry(arm, b, x, k);
                    entry_t[r][arm][!runs as usize] += s.elapsed().as_secs_f64();
                    assert!(arm != 1 || by_runs == runs, "B's path is takes_runs'");
                    walk_check(b, x, k);
                }
            }
            // the phase sweep: attribution
            for &arm in &ord {
                let t = &mut times[r][arm];
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
            "{name}: {} blocks, {:.1} MB of output, {} bytes, SHA-256 {}; by records {} blocks \
             ({:.1}% of rows), r/n {:.4} there, fill slots/row {:.4}; fallback {} blocks",
            blocks.len(),
            n as f64 / 1e6,
            data.len(),
            sha256::hex(data),
            tally.blocks_runs,
            100.0 * tally.rows_runs as f64 / tally.rows.max(1) as f64,
            tally.records as f64 / tally.rows_runs.max(1) as f64,
            tally.slots as f64 / tally.rows_runs.max(1) as f64,
            tally.blocks_fallback
        );
        let cr = |s: f64| s * ghz * 1e9 / n.max(1) as f64;
        let median = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let med_ph = |arm: usize, p: usize| median(&mut times.iter().map(|r| r[arm][p]).collect());
        print!("  {:14}", "nominal c/B");
        print!("  {:>9}", "entry");
        for p in PHASES {
            print!("  {p:>9}");
        }
        println!("  phase sum");
        for (arm, an) in ARMS.iter().enumerate() {
            if arm == 3 {
                print!("  {:14}  {:>9}", "", "");
                for p in PHASES_D {
                    print!("  {p:>9}");
                }
                println!();
            }
            print!("  {an:14}");
            print!(
                "  {:9.3}",
                cr(median(
                    &mut entry_t.iter().map(|r| r[arm][0] + r[arm][1]).collect()
                ))
            );
            for p in 0..4 {
                print!("  {:9.3}", cr(med_ph(arm, p)));
            }
            println!(
                "  {:9.3}",
                cr(median(
                    &mut times.iter().map(|r| r[arm].iter().sum()).collect()
                ))
            );
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
                        "    {an:14} {pn:9}  {:6.3}  {:6.3}  {:5.2}  {:>8}  {:>6}  {:5.1}",
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
        let etot = |a: usize| move |r: &Entry| r[a][0] + r[a][1];
        let epath = |a: usize, p: usize| move |r: &Entry| r[a][p];
        let total = |a: usize| move |r: &Round| r[a].iter().sum::<f64>();
        let ph = |a: usize, ps: &'static [usize]| {
            move |r: &Round| ps.iter().map(|&p| r[a][p]).sum::<f64>()
        };
        let rows: [(&str, Option<Interval>); 10] = [
            (
                "R_stage  B/A, the entry point, every block (judged)",
                ratio(&entry_t, &etot(1), &etot(0)),
            ),
            (
                "R_acc    B/A, the entry point, records' blocks",
                ratio(&entry_t, &epath(1, 0), &epath(0, 0)),
            ),
            (
                "R_fb     B/A, the entry point, fallback's blocks",
                ratio(&entry_t, &epath(1, 1), &epath(0, 1)),
            ),
            (
                "R_phases B/A, the phases' sum (attribution)",
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
                "R_floor  D store reference / A build",
                ratio(&times, &ph(3, &[1]), &ph(0, &[2])),
            ),
            (
                "R_warm   D build after pre-touch / A build",
                ratio(&times, &ph(3, &[2]), &ph(0, &[2])),
            ),
        ];
        for (label, r) in &rows {
            match r {
                Some((g, lo, hi)) => println!("  {label:52} {g:.4} ({lo:.4}, {hi:.4})"),
                None => println!("  {label:52} n/a (no such blocks, or times absent)"),
            }
        }
        let (e, rn) = cover[1][2];
        let fill_pmu_per_cell = counters
            .as_ref()
            .filter(|c| c.has(1) && rn > 0 && rn == e && tally.cells > 0)
            .map(|_| counts[1][2][1] as f64 / tally.cells as f64);
        readings.push(Reading {
            name: name.clone(),
            blocks_fallback: tally.blocks_fallback,
            stage: rows[0].1,
            fb: rows[2].1,
            lead: rows[6].1,
            floor: rows[8].1,
            warm: rows[9].1,
            fill_nominal_per_cell: med_ph(1, 2) * ghz * 1e9 / tally.cells.max(1) as f64,
            fill_pmu_per_cell,
            cells_per_row: tally.cells as f64 / tally.rows_runs.max(1) as f64,
            slots_per_row: tally.slots as f64 / tally.rows_runs.max(1) as f64,
        });
        recs_out.push(FileRecord {
            name: name.clone(),
            bytes: data.len(),
            sha256: sha256::hex(data),
            tally,
            order,
            entry: entry_t,
            phases: times,
        });
    }

    if let Some(p) = &record_path {
        let run = Run {
            head: head.as_deref(),
            rounds,
            ghz,
            k,
            host: proc_field("/proc/cpuinfo", "model name"),
            affinity: proc_field("/proc/self/status", "Cpus_allowed_list"),
        };
        std::fs::write(p, record(&run, &recs_out)).unwrap();
        println!();
        println!("record: {p}");
    }

    // The registration's readings (docs/journal/2026-11-09-run-fed-stage.md), computed as
    // the verifier computes them, but UNJUDGED: only `bench/run_fed_stage.py verify` on the
    // registered run's record judges. A required interval that is absent is INVALID here,
    // never a pass
    println!();
    println!("readings (UNJUDGED: the verifier judges the registered run's record):");
    let w = &readings[0];
    let mut missing: Vec<String> = Vec::new();
    for r in &readings {
        if r.stage.is_none() {
            missing.push(format!("{}: R_stage", r.name));
        }
        if r.blocks_fallback > 0 && r.fb.is_none() {
            missing.push(format!("{}: R_fb", r.name));
        }
    }
    if !missing.is_empty() {
        println!(
            "  INVALID: required intervals absent: {}",
            missing.join("; ")
        );
        return;
    }
    let (g, lo, hi) = w.stage.unwrap();
    let worst = |f: &dyn Fn(&Reading) -> Option<f64>, from: usize| {
        readings[from..]
            .iter()
            .filter_map(|r| f(r).map(|x| (r.name.clone(), x)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    };
    let s_syn = worst(&|r| r.stage.map(|x| x.0), 1).unwrap();
    let s_fb = worst(&|r| r.fb.map(|x| x.0), 0);
    let screens_ok = s_syn.1 <= 1.02 && !matches!(s_fb, Some((_, x)) if x > 1.02);
    let q1 = if g <= 0.90 && hi < 1.0 && screens_ok {
        "FEASIBLE: a one-worker production prototype may be registered (screens are point \
         estimates, not a noninferiority claim)"
    } else if lo > 0.95 {
        "STOP: no one-worker stage saving of 5% or more shown on this runner (an operational \
         pause, not proof of no effect)"
    } else if g <= 0.90 && hi < 1.0 {
        "STOP: a screen failed (an operational pause, not proof of harm)"
    } else {
        "STOP, INCONCLUSIVE: no saving shown to the gate (an operational pause, not proof of \
         no effect; no re-run without a new registration)"
    };
    println!("  Q1 R_stage {g:.4} ({lo:.4}, {hi:.4}); gate: geomean <= 0.90 and hi < 1");
    println!(
        "  Q1 screen, synthetic whole stage: worst R_stage geomean {:.4} ({}); screen <= 1.02",
        s_syn.1, s_syn.0
    );
    match &s_fb {
        Some((nm, x)) => println!(
            "  Q1 screen, fallback (entry point, selector charged): worst R_fb geomean {x:.4} \
             ({nm}); screen <= 1.02"
        ),
        None => println!("  Q1 screen, fallback: no corpus has fallback blocks (vacuous)"),
    }
    println!("  Q1 reading: {q1}");
    let q2 = match w.lead {
        Some((g, _, hi)) if g <= 0.90 && hi < 1.0 => "LEADER LEAD (exploratory)",
        Some((_, lo, _)) if lo > 0.95 => "NO LEADER LEAD (exploratory)",
        Some(_) => "NO LEADER LEAD, INCONCLUSIVE (exploratory)",
        None => "n/a",
    };
    if let Some((g, lo, hi)) = w.lead {
        println!("  Q2 R_lead {g:.4} ({lo:.4}, {hi:.4}), phases, exploratory");
    }
    println!("  Q2 reading: {q2}");
    let show = |r: Option<Interval>| match r {
        Some((g, lo, hi)) => format!("{g:.4} ({lo:.4}, {hi:.4})"),
        None => "n/a".into(),
    };
    println!(
        "  M1 R_warm {} (exploratory: a pre-touch contrast; it does not isolate cold write \
         misses)",
        show(w.warm)
    );
    println!(
        "  M2 R_floor {} (exploratory: against a store-only reference, not a floor every \
         build obeys; B stores {:.4} slots a row with its overstores)",
        show(w.floor),
        w.slots_per_row
    );
    println!(
        "  M3 fill {:.1} nominal cycles a cell (wall time x {ghz} GHz, not counted cycles); \
         counted: {}; cells/row {:.4} (exploratory)",
        w.fill_nominal_per_cell,
        w.fill_pmu_per_cell
            .map_or("n/a (no full-coverage cycle counter)".into(), |c| format!(
                "{c:.1} cycles a cell"
            )),
        w.cells_per_row
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_and_slots() {
        // three one-row cells: 3 cells, 24 slots for a 3-word table
        let recs = [
            b'a' as u32,
            1 << 8 | b'b' as u32,
            2 << 8 | b'c' as u32,
            3 << 8,
        ];
        let fo = [[0, 0], [1, 1], [2, 2], [3, 0]];
        assert_eq!(cells(&recs, &fo), (3, 24));
        // one run of 20: one cell, 24 slots
        assert_eq!(cells(&[b'a' as u32, 20 << 8], &[[0, 0], [20, 0]]), (1, 24));
    }

    /// The verifier's fixture (bench/fixtures/run_fed_stage.json), written by `record`
    /// itself, so the verifier's tests read this writer's format: a registered-shape run
    /// whose times are a formula (B 0.85 of A on Wikidata, about 1 elsewhere), no
    /// measurement. `RUN_FED_FIXTURE=write` rewrites it; otherwise it must match
    #[test]
    fn the_verifiers_fixture() {
        let names = [
            "corpora/wikidata.bz2",
            "synthetic text",
            "synthetic random",
            "synthetic one byte",
            "synthetic pattern",
            "synthetic runs",
            "synthetic records",
        ];
        let rounds = 15;
        let files: Vec<FileRecord> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let b_over_a = if i == 0 { 0.85 } else { 1.0 };
                let (blocks_runs, blocks_fallback) =
                    [(400, 31), (3, 0), (0, 3), (1, 0), (1, 0), (3, 0), (0, 3)][i];
                let jitter =
                    |r: usize, a: usize| 1.0 + 0.004 * (((r * 7 + a * 3 + i) % 5) as f64 - 2.0);
                let on = |c: usize| if c > 0 { 1.0 } else { 0.0 };
                FileRecord {
                    name: name.to_string(),
                    bytes: 1000 + i,
                    sha256: if i == 0 {
                        "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915".into()
                    } else {
                        format!("{:064x}", i)
                    },
                    tally: Tally {
                        blocks: blocks_runs + blocks_fallback,
                        rows: 900_000 * (blocks_runs + blocks_fallback),
                        blocks_runs,
                        rows_runs: 900_000 * blocks_runs,
                        records: 50_000 * blocks_runs,
                        blocks_fallback,
                        cells: 90_000 * blocks_runs,
                        slots: 1_100_000 * blocks_runs,
                    },
                    order: (0..rounds)
                        .map(|r| std::array::from_fn(|j| (j + r) % NARMS))
                        .collect(),
                    entry: (0..rounds)
                        .map(|r| {
                            std::array::from_fn(|a| {
                                let f = if a == 1 { b_over_a } else { 1.0 } * jitter(r, a);
                                [
                                    0.5 * f * on(blocks_runs),
                                    0.25 * jitter(r, a) * on(blocks_fallback),
                                ]
                            })
                        })
                        .collect(),
                    phases: (0..rounds)
                        .map(|r| {
                            std::array::from_fn(|a| {
                                let (j, run, fb) =
                                    (jitter(r, a), on(blocks_runs), on(blocks_fallback));
                                match a {
                                    1 => [
                                        0.2 * j * run,
                                        0.05 * j * run,
                                        0.2 * j * run,
                                        0.25 * j * fb,
                                    ],
                                    3 => [
                                        0.25 * j * run,
                                        0.1 * j * run,
                                        0.25 * j * run,
                                        0.25 * j * fb,
                                    ],
                                    _ => [0.25 * j * run, 0.0, 0.25 * j * run, 0.25 * j * fb],
                                }
                            })
                        })
                        .collect(),
                }
            })
            .collect();
        let run = Run {
            head: Some("0123456789abcdef0123456789abcdef01234567"),
            rounds,
            ghz: 2.8,
            k: Kernels {
                avx2: true,
                clmul: true,
            },
            host: "Fixture CPU".into(),
            affinity: "1".into(),
        };
        let got = record(&run, &files);
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/fixtures/run_fed_stage.json"
        );
        if std::env::var("RUN_FED_FIXTURE").as_deref() == Ok("write") {
            std::fs::write(path, &got).unwrap();
        }
        assert!(
            std::fs::read_to_string(path).unwrap() == got,
            "bench/fixtures/run_fed_stage.json is not `record`'s output: RUN_FED_FIXTURE=write \
             rewrites it"
        );
    }

    #[test]
    fn absent_rounds_are_no_interval() {
        let ok = [1.0f64, 1.1, 0.9];
        assert!(ratio(&ok, &|x| *x, &|_| 1.0).is_some());
        assert!(ratio(&[1.0, 0.0, 1.0], &|x| *x, &|_| 1.0).is_none());
        assert!(ratio(&[1.0, 1.0], &|x| *x, &|_| 0.0).is_none());
        assert!(ratio(&[1.0, f64::NAN], &|x| *x, &|_| 1.0).is_none());
    }
}
