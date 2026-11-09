//! A mutation fuzz loop against libbzip2: valid streams (several shapes, levels, crafted
//! blocks, several streams) mutated (bits flipped, bytes set, cut, spliced, inserted,
//! deleted), each decoded by libbzip2 and by the decoders under test (tests/common's
//! `candidates`: this crate's, sequential on both kernel sets, and now and then parallel;
//! any other in-process decoder behind `Candidate`). Errors must be errors: never different bytes, never bytes
//! where libbzip2 fails, never a panic. Where libbzip2 decodes what this decoder refuses
//! (a randomised block, an oversubscribed code past its alphabet), the case is counted by
//! its error.
//!
//! The pipeline cases (tests/common's `pipeline_candidates`: the shipped default, three
//! pipelined teams of three with park and small, forced through each default entry point,
//! and one such team): multi-block inputs (`pipeline_corpus`), some edits aimed at a later
//! block's header and tables or the stream's end, and some left valid. Each is held to the
//! plain parallel decoder exactly (the bytes before any error, and the error's kind, block
//! and bit) as well as to libbzip2, and counted as reaching the pipeline when a leader took
//! a job while its helpers inverted another block (`pipeline_counts`).
//!
//! The run-fed cases (MODE run-fed, built with `--features run-fed`: tests/common's
//! `run_fed_candidates`, `StreamDecoder::pipelined_run_fed` at each sort, several plans and
//! both kernel sets, research: docs/journal/2026-11-09-run-fed-leader.md): the pipeline
//! cases' checks on `run_fed_corpus` (the pipeline corpus and crafted run-heavy streams the
//! records' rule takes), each case's reach counted from the run-fed phases (blocks by
//! records and by the column, on the team and alone, and the rebuilds after a refused
//! check, a bug: any is a failure).
//!
//! Usage: fuzz SECONDS [SEED] [OUT_DIR for the inputs of any failure, or -] [MODE]
//! MODE: mixed (the default: every 8th case a pipeline case), classic, pipeline or run-fed

#[path = "../tests/common/mod.rs"]
mod common;

use common::*;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(60);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let out_dir = args.get(3).cloned().filter(|d| d != "-");
    let mode = args.get(4).map_or("mixed", |s| s.as_str());
    let every = match mode {
        "mixed" => 8,
        "classic" => 0,
        "pipeline" | "run-fed" => 1,
        m => panic!("MODE is mixed, classic, pipeline or run-fed, not {m}"),
    };
    let run_fed = mode == "run-fed";
    assert!(
        !run_fed || cfg!(feature = "run-fed"),
        "MODE run-fed needs the build's --features run-fed"
    );
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);

    let mut corpus: Vec<Vec<u8>> = Vec::new();
    for (_, data) in samples(&mut rng) {
        let small = &data[..data.len().min(150_000)];
        for level in [1, 9] {
            corpus.push(compress(small, level));
        }
    }
    corpus.push(
        [
            compress(b"one", 9),
            compress(b"", 2),
            compress(&rng.bytes(5000), 1),
        ]
        .concat(),
    );
    for n in [5usize, 300, 70_000] {
        let ll: Vec<u8> = (0..n).map(|_| b"abc"[rng.below(3) as usize]).collect();
        let p = rng.below(n as u64) as u32;
        let z = craft(9, &[Spec::of(ll, p)], None);
        if oracle(&z).is_ok() {
            corpus.push(z);
        }
    }
    let cands = candidates();
    let pipes = if run_fed {
        run_fed_pipes()
    } else {
        pipeline_candidates()
    };
    let pipe_corpus: Vec<(Vec<u8>, Vec<u64>)> = if every == 0 {
        Vec::new()
    } else {
        let c = if run_fed {
            run_fed_corpus(&mut rng)
        } else {
            pipeline_corpus(&mut rng)
        };
        c.into_iter()
            .map(|(_, z)| {
                let starts = block_starts(&z);
                (z, starts)
            })
            .collect()
    };
    let mut pipe_stats = vec![PipeStats::default(); pipes.len()];
    let mut reach = vec![Reach::default(); pipes.len()];
    let deadline = Instant::now() + Duration::from_secs(secs);
    let (mut cases, mut bad) = (0u64, 0u64);
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    while Instant::now() < deadline {
        if every != 0 && (cases + 1) % every == 0 {
            cases += 1;
            let i = (cases / every) as usize % pipes.len();
            let before = phases_now();
            let outcome = pipeline_case(
                &pipes[i],
                &mut pipe_stats[i],
                &pipe_corpus,
                &corpus,
                &mut rng,
                &out_dir,
                cases,
                &mut bad,
            );
            if run_fed {
                let d: Vec<u64> = phases_now()
                    .iter()
                    .zip(&before)
                    .map(|(a, b)| a - b)
                    .collect();
                reach[i].add(&d);
                if d[REBUILT] > 0 {
                    bad += 1;
                    let why = format!(
                        "{}: {} blocks rebuilt after a refused check",
                        pipes[i].name(),
                        d[REBUILT]
                    );
                    // (the case is the seed's: rerun at it to get its input)
                    eprintln!("case {cases}: {why}");
                }
            }
            *outcomes.entry(format!("pipeline: {outcome}")).or_default() += 1;
            continue;
        }
        let base = &corpus[rng.below(corpus.len() as u64) as usize];
        let mut t = base.clone();
        let edits = 1 + rng.below(4);
        for _ in 0..edits {
            mutate(&mut t, &corpus, &mut rng, &near_start);
        }
        cases += 1;
        let want = oracle(&t);
        // (each candidate in turn; tests/common's last, the parallel one, is slower to
        // start, so it takes only every 16th case)
        let c = if cases % 16 == 0 {
            &cands[cands.len() - 1]
        } else {
            &cands[cases as usize % (cands.len() - 1).max(1)]
        };
        let got = match try_decode(c.as_ref(), &t) {
            Ok(r) => r,
            Err(_) => {
                bad += 1;
                report(&out_dir, &t, cases, &format!("{}: panic", c.name()));
                continue;
            }
        };
        let outcome = match (&want, &got) {
            (Ok(a), Ok(b)) if a == b => "both decode, same bytes".to_string(),
            (Ok(_), Ok(_)) => {
                bad += 1;
                report(
                    &out_dir,
                    &t,
                    cases,
                    &format!("{}: different bytes", c.name()),
                );
                "DIFFERENT BYTES".into()
            }
            (Err(_), Ok(_)) => {
                bad += 1;
                report(
                    &out_dir,
                    &t,
                    cases,
                    &format!("{}: libbzip2 fails, this decodes", c.name()),
                );
                "LIBBZIP2 FAILS, THIS DECODES".into()
            }
            (Err(_), Err(e)) => format!("both fail ({e})"),
            (Ok(_), Err(e)) => format!("libbzip2 decodes, this refuses ({e})"),
        };
        *outcomes.entry(outcome).or_default() += 1;
    }
    println!("{cases} cases in {secs} s (seed {seed}, mode {mode}), {bad} failures");
    for (o, n) in &outcomes {
        println!("{n:>9}  {o}");
    }
    if every != 0 {
        println!(
            "pipeline cases: candidate, cases, cases reaching the pipeline (a job taken \
             overlapped) of those countable, cases with an overlapped job's error, \
             overlapped jobs in all"
        );
        for (p, st) in pipes.iter().zip(&pipe_stats) {
            let reach = if st.countable == 0 {
                "not countable (the decoder is not in reach; the constructor is the counted one's)"
                    .to_string()
            } else {
                format!(
                    "{} of {} reached, {} with an overlapped error, {} overlapped jobs",
                    st.reached, st.countable, st.overlap_error_cases, st.overlapped_jobs
                )
            };
            println!("{:>9}  {}: {reach}", st.cases, p.name());
        }
    }
    if run_fed {
        println!(
            "run-fed reach: candidate, then its cases' blocks staged that the records' rule \
             takes, the team's blocks by records and by the column, the small blocks by \
             records and by the column, the cases with a team block by records, and the \
             blocks rebuilt after a refused check (a bug: 0)"
        );
        for (p, r) in pipes.iter().zip(&reach) {
            println!(
                "{:>9}  {}: rule {}, team records {} column {}, small records {} column {}, \
                 cases with team records {}, rebuilt {}",
                r.cases,
                p.name(),
                r.sum[RULE],
                r.sum[TEAM_RECORDS],
                r.sum[TEAM_COLUMN],
                r.sum[SMALL_RECORDS],
                r.sum[SMALL_COLUMN],
                r.team_record_cases,
                r.sum[REBUILT]
            );
        }
    }
    std::process::exit(if bad == 0 { 0 } else { 1 });
}

/// The run-fed phases a case's reach is counted from, by index in `phases_now`
const PHASES: [&str; 6] = [
    "rule_blocks",
    "team_records",
    "team_column",
    "small_records",
    "small_column",
    "fallbacks",
];
const RULE: usize = 0;
const TEAM_RECORDS: usize = 1;
const TEAM_COLUMN: usize = 2;
const SMALL_RECORDS: usize = 3;
const SMALL_COLUMN: usize = 4;
const REBUILT: usize = 5;

/// The run-fed phases now (`PHASES`; this process's totals, one decode at a time here, so
/// a case's are the difference across it); zeros without the feature
fn phases_now() -> Vec<u64> {
    #[cfg(feature = "run-fed")]
    {
        let all = mibz2::parallel::run_fed_phases();
        PHASES
            .iter()
            .map(|n| all.iter().find(|p| p.0 == *n).expect("a run-fed phase").2)
            .collect()
    }
    #[cfg(not(feature = "run-fed"))]
    {
        vec![0; PHASES.len()]
    }
}

/// A run-fed candidate's reach over the run
#[derive(Clone, Default)]
struct Reach {
    cases: u64,
    sum: [u64; PHASES.len()],
    team_record_cases: u64,
}

impl Reach {
    fn add(&mut self, d: &[u64]) {
        self.cases += 1;
        for (s, v) in self.sum.iter_mut().zip(d) {
            *s += v;
        }
        self.team_record_cases += (d[TEAM_RECORDS] > 0) as u64;
    }
}

#[cfg(feature = "run-fed")]
fn run_fed_pipes() -> Vec<Pipelined> {
    run_fed_candidates()
}

#[cfg(not(feature = "run-fed"))]
fn run_fed_pipes() -> Vec<Pipelined> {
    Vec::new()
}

/// A pipelined candidate's counts over the run
#[derive(Clone, Default)]
struct PipeStats {
    cases: u64,
    countable: u64,
    reached: u64,
    overlap_error_cases: u64,
    overlapped_jobs: u64,
}

/// One pipeline case: a multi-block input, edited (or not), decoded by `p`, by the plain
/// parallel decoder and by libbzip2; the outcome's class
#[allow(clippy::too_many_arguments)]
fn pipeline_case(
    p: &Pipelined,
    st: &mut PipeStats,
    pipe_corpus: &[(Vec<u8>, Vec<u64>)],
    corpus: &[Vec<u8>],
    rng: &mut Rng,
    out_dir: &Option<String>,
    case: u64,
    bad: &mut u64,
) -> String {
    let (base, starts) = &pipe_corpus[rng.below(pipe_corpus.len() as u64) as usize];
    let mut t = base.clone();
    // (one case in eight valid; else one or two edits, aimed at a block's start (its
    // header, tables and first symbols: a block after the first where there is one, so
    // that the blocks before it are in flight), at the stream's end, or anywhere)
    let later = if starts.len() > 1 {
        &starts[1..]
    } else {
        &starts[..]
    };
    let edits = if rng.below(8) == 0 {
        0
    } else {
        1 + rng.below(2)
    };
    let pick = |rng: &mut Rng, n: usize| match rng.below(3) {
        0 if !later.is_empty() => {
            let s = later[rng.below(later.len() as u64) as usize] / 8;
            (s as usize + rng.below(120) as usize).min(n - 1)
        }
        1 => n - 1 - (rng.below(16) as usize).min(n - 1),
        _ => rng.below(n as u64) as usize,
    };
    for _ in 0..edits {
        mutate(&mut t, corpus, rng, &pick);
    }
    st.cases += 1;
    let want = oracle(&t);
    let (plain_out, plain_res) = plain_parallel(&t);
    let got = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.run(&t))) {
        Ok(r) => r,
        Err(_) => {
            *bad += 1;
            report(out_dir, &t, case, &format!("{}: panic", p.name()));
            return "PANIC".into();
        }
    };
    if let Some((jobs, errs)) = got.counts {
        st.countable += 1;
        st.reached += (jobs > 0) as u64;
        st.overlap_error_cases += (errs > 0) as u64;
        st.overlapped_jobs += jobs;
    }
    let same_as_plain =
        got.res == plain_res && (got.out == plain_out || (!p.partial() && got.res.is_err()));
    if !same_as_plain {
        *bad += 1;
        report(
            out_dir,
            &t,
            case,
            &format!(
                "{}: not the plain decoder's: {:?} after {} bytes, the plain {:?} after {}",
                p.name(),
                got.res,
                got.out.len(),
                plain_res,
                plain_out.len()
            ),
        );
        return "NOT THE PLAIN DECODER'S BYTES OR ERROR".into();
    }
    match (&want, &got.res) {
        (Ok(a), Ok(())) if *a == got.out => "both decode, same bytes".into(),
        (Ok(_), Ok(())) => {
            *bad += 1;
            report(out_dir, &t, case, &format!("{}: different bytes", p.name()));
            "DIFFERENT BYTES".into()
        }
        (Err(_), Ok(())) => {
            *bad += 1;
            let why = format!("{}: libbzip2 fails, this decodes", p.name());
            report(out_dir, &t, case, &why);
            "LIBBZIP2 FAILS, THIS DECODES".into()
        }
        (Err(_), Err(e)) => format!("both fail ({}), as the plain decoder", kind_name(e)),
        (Ok(_), Err(e)) => format!(
            "libbzip2 decodes, this refuses ({}), as the plain decoder",
            kind_name(e)
        ),
    }
}

/// An edit's position in n bytes: mostly near the start (the headers and tables, where the
/// checks are)
fn near_start(rng: &mut Rng, n: usize) -> usize {
    if rng.below(2) == 0 {
        rng.below(n.min(400) as u64) as usize
    } else {
        rng.below(n as u64) as usize
    }
}

/// One edit of `t`, at positions `pos` picks (of `t.len()`, at least 1)
fn mutate(
    t: &mut Vec<u8>,
    corpus: &[Vec<u8>],
    rng: &mut Rng,
    pos: &dyn Fn(&mut Rng, usize) -> usize,
) {
    if t.is_empty() {
        t.push(rng.next() as u8);
        return;
    }
    match rng.below(8) {
        0..=2 => {
            let i = pos(rng, t.len());
            t[i] ^= 1 << rng.below(8);
        }
        3 => {
            let i = pos(rng, t.len());
            t[i] = [0, 0xff, 0x7f, 0x80, rng.next() as u8][rng.below(5) as usize];
        }
        4 => {
            let n = rng.below(t.len() as u64 + 1) as usize;
            t.truncate(n);
        }
        5 => {
            let i = pos(rng, t.len());
            let n = 1 + rng.below(8) as usize;
            for _ in 0..n {
                t.insert(i.min(t.len()), rng.next() as u8);
            }
        }
        6 => {
            let i = pos(rng, t.len());
            let n = (1 + rng.below(8) as usize).min(t.len() - i);
            t.drain(i..i + n);
        }
        _ => {
            let o = &corpus[rng.below(corpus.len() as u64) as usize];
            let i = rng.below(t.len() as u64) as usize;
            let j = rng.below(o.len() as u64) as usize;
            t.truncate(i);
            t.extend_from_slice(&o[j..]);
        }
    }
}

fn report(dir: &Option<String>, t: &[u8], case: u64, why: &str) {
    eprintln!("case {case}: {why}");
    if let Some(d) = dir {
        let _ = std::fs::create_dir_all(d);
        let _ = std::fs::write(format!("{d}/case-{case}.bz2"), t);
    }
}
