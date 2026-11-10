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
//! The service cases (MODE service; tests/common's `service_candidates`: the service core
//! as the default at 10 threads on its target, through each default entry point with the
//! machine's facts substituted, and explicit shapes): the pipeline cases' inputs and edits,
//! held to the plain parallel decoder and libbzip2 the same way, and counted from the
//! service core's own counts, read only after the completion fence (every hub thread
//! ended): cases with a block inverted on a team's pool, with a block finished, and with
//! more blocks prepared (finished) than the decoder has front (back) sets, so those sets
//! were reused.
//!
//! The team cases (MODE teams): the plain teams, `bwt::Cooperative`, forced by their plan
//! (`StreamDecoder::planned`: no default selects them below 6 physical cores, so no other
//! mode runs them on a small machine). The pipeline cases' inputs and edits, held to the
//! plain parallel decoder and libbzip2 the same way. Each case's blocks are also read here,
//! apart from the decoder, and counted by the route the decoder takes them by, which is a
//! function of the block alone: by its run records on the team's pool (a block
//! `runs::takes_runs` admits, of `bwt::MIN_COOP` rows or more: `Cooperative::invert_runs`'s
//! fill by shares and walk backward), by its records on one core (an admitted block below
//! that), or by its column either way.
//!
//! Usage: fuzz SECONDS [SEED] [OUT_DIR for the inputs of any failure, or -] [MODE]
//! MODE: mixed (the default: every 8th case a pipeline case), classic, pipeline, service or
//! teams, or cells (forced cells plus production admission; reach is asserted).

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
    if mode == "cells" {
        cells_fuzz(secs, seed, out_dir);
        return;
    }
    let every = match mode {
        "mixed" => 8,
        "classic" => 0,
        "pipeline" | "service" | "teams" => 1,
        m => panic!("MODE is mixed, classic, pipeline, service or teams, not {m}"),
    };
    let service = mode == "service";
    let teams = mode == "teams";
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
    let pipes = pipeline_candidates();
    let pipe_corpus: Vec<(Vec<u8>, Vec<u64>)> = if every == 0 {
        Vec::new()
    } else {
        pipeline_corpus(&mut rng)
            .into_iter()
            .map(|(_, z)| {
                let starts = block_starts(&z);
                (z, starts)
            })
            .collect()
    };
    let mut pipe_stats = vec![PipeStats::default(); pipes.len()];
    let svcs = service_candidates();
    let mut svc_stats = vec![SvcStats::default(); svcs.len()];
    let team_cands = team_candidates();
    let mut team_stats = vec![TeamStats::default(); team_cands.len()];
    let deadline = Instant::now() + Duration::from_secs(secs);
    let (mut cases, mut bad) = (0u64, 0u64);
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    while Instant::now() < deadline {
        if teams {
            cases += 1;
            let i = cases as usize % team_cands.len();
            let outcome = team_case(
                &team_cands[i],
                &mut team_stats[i],
                &pipe_corpus,
                &corpus,
                &mut rng,
                &out_dir,
                cases,
                &mut bad,
            );
            *outcomes.entry(format!("teams: {outcome}")).or_default() += 1;
            continue;
        }
        if service {
            cases += 1;
            let i = cases as usize % svcs.len();
            let outcome = service_case(
                &svcs[i],
                &mut svc_stats[i],
                &pipe_corpus,
                &corpus,
                &mut rng,
                &out_dir,
                cases,
                &mut bad,
            );
            *outcomes.entry(format!("service: {outcome}")).or_default() += 1;
            continue;
        }
        if every != 0 && (cases + 1) % every == 0 {
            cases += 1;
            let i = (cases / every) as usize % pipes.len();
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
    if teams {
        println!(
            "team cases: candidate, cases; cases with a block by records on the team's pool, \
             by records on one core, by the column on the pool, by the column on one core; \
             cases with an error; then the blocks by each route, and the blocks whose front \
             stage fails, admitted to the records and not"
        );
        for (c, st) in team_cands.iter().zip(&team_stats) {
            println!(
                "{:>9}  {}: records pooled {}, records alone {}, column pooled {}, column \
                 alone {}, errors {}; blocks {} / {} / {} / {}; front errors {} / {}",
                st.cases,
                c.name,
                st.with[0],
                st.with[1],
                st.with[2],
                st.with[3],
                st.errors,
                st.blocks[0],
                st.blocks[1],
                st.blocks[2],
                st.blocks[3],
                st.front_errors[0],
                st.front_errors[1]
            );
        }
    } else if service {
        println!(
            "service cases (counted after the completion fence): candidate, cases, with a \
             block on a team's pool, with a block finished, with front sets reused, with \
             back sets reused, with an error"
        );
        for (c, st) in svcs.iter().zip(&svc_stats) {
            println!(
                "{:>9}  {}: pooled {}, finished {}, fronts reused {}, backs reused {}, errors {}",
                st.cases,
                c.name(),
                st.pooled,
                st.finished,
                st.front_reuse,
                st.back_reuse,
                st.errors
            );
        }
    } else if every != 0 {
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
    std::process::exit(if bad == 0 { 0 } else { 1 });
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

/// A service candidate's counts over the run (cases, each counted once)
#[derive(Clone, Default)]
struct SvcStats {
    cases: u64,
    pooled: u64,
    finished: u64,
    front_reuse: u64,
    back_reuse: u64,
    errors: u64,
}

/// A plain-team decoder forced by its plan: `teams` teams of `k`, on kernels `kernels`
struct TeamCand {
    name: String,
    teams: usize,
    k: usize,
    kernels: mibz2::Kernels,
}

/// The plain teams to fuzz: each team size `StreamDecoder::planned` runs (2, 3, 4), one
/// team and two, and one on the portable kernels (the scalar front and fill)
fn team_candidates() -> Vec<TeamCand> {
    let detect = mibz2::Kernels::detect();
    let mut v: Vec<TeamCand> = [(1, 2), (2, 2), (1, 3), (1, 4)]
        .into_iter()
        .map(|(teams, k)| TeamCand {
            name: format!("{teams} plain team(s) of {k}"),
            teams,
            k,
            kernels: detect,
        })
        .collect();
    v.push(TeamCand {
        name: "1 plain team of 3, scalar kernels".into(),
        teams: 1,
        k: 3,
        kernels: mibz2::Kernels::scalar(),
    });
    v
}

/// A team candidate's counts over the run: the cases, those with a block by each route (by
/// records on the pool, by records alone, by the column on the pool, by the column alone),
/// those with an error, the blocks by each route, and the blocks whose front stage fails
/// (its move-to-front's error, or a primary index past the block): admitted ones, where
/// the records' front is the one to fail, and the others
#[derive(Clone, Default)]
struct TeamStats {
    cases: u64,
    with: [u64; 4],
    errors: u64,
    blocks: [u64; 4],
    front_errors: [u64; 2],
}

/// The blocks of `data` that scan and whose move-to-front stage succeeds, counted by the
/// route a plain team takes each by (`TeamStats`'s order), up to the input's first error:
/// read here, with the decoder's own selector and bound, not reported by the decoder
fn team_routes(data: &[u8]) -> ([u64; 4], [u64; 2]) {
    use mibz2::stream::{Event, Scanner, SliceSource};
    let (mut by, mut front_errors) = ([0u64; 4], [0u64; 2]);
    let mut sc = Scanner::new(SliceSource(data));
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let mut syms = Vec::new();
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        let records = mibz2::runs::takes_runs(&b);
        let n = match mibz2::mtf::mtf(&b, &mut ll, &mut counts, mibz2::Kernels::scalar()) {
            Ok(n) if (b.orig_ptr as usize) < n => n,
            _ => {
                front_errors[!records as usize] += 1;
                break;
            }
        };
        let pooled = n >= mibz2::bwt::MIN_COOP;
        by[match (records, pooled) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        }] += 1;
        syms = b.syms;
    }
    (by, front_errors)
}

/// One team case: as `service_case`, for a plain-team candidate
#[allow(clippy::too_many_arguments)]
fn team_case(
    p: &TeamCand,
    st: &mut TeamStats,
    pipe_corpus: &[(Vec<u8>, Vec<u64>)],
    corpus: &[Vec<u8>],
    rng: &mut Rng,
    out_dir: &Option<String>,
    case: u64,
    bad: &mut u64,
) -> String {
    use mibz2::parallel::Plan;
    let (base, starts) = &pipe_corpus[rng.below(pipe_corpus.len() as u64) as usize];
    let t = edited(base, starts, corpus, rng);
    st.cases += 1;
    let want = oracle(&t);
    let (plain_out, plain_res) = plain_parallel(&t);
    let run = || {
        let plan = Plan::Teams {
            teams: p.teams,
            k: p.k,
        };
        let dec = mibz2::StreamDecoder::planned(plan, 2 * p.teams, p.kernels);
        drive(&dec, &t, 1 << 20, 1 << 16)
    };
    let (out, res) = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
        Ok(r) => r,
        Err(_) => {
            *bad += 1;
            report(out_dir, &t, case, &format!("{}: panic", p.name));
            return "PANIC".into();
        }
    };
    let (by, front_errors) = team_routes(&t);
    for (i, &n) in by.iter().enumerate() {
        st.with[i] += (n > 0) as u64;
        st.blocks[i] += n;
    }
    st.front_errors[0] += front_errors[0];
    st.front_errors[1] += front_errors[1];
    st.errors += res.is_err() as u64;
    if res != plain_res || out != plain_out {
        *bad += 1;
        report(
            out_dir,
            &t,
            case,
            &format!(
                "{}: not the plain decoder's: {:?} after {} bytes, the plain {:?} after {}",
                p.name,
                res,
                out.len(),
                plain_res,
                plain_out.len()
            ),
        );
        return "NOT THE PLAIN DECODER'S BYTES OR ERROR".into();
    }
    match (&want, &res) {
        (Ok(a), Ok(())) if *a == out => "both decode, same bytes".into(),
        (Ok(_), Ok(())) => {
            *bad += 1;
            report(out_dir, &t, case, &format!("{}: different bytes", p.name));
            "DIFFERENT BYTES".into()
        }
        (Err(_), Ok(())) => {
            *bad += 1;
            let why = format!("{}: libbzip2 fails, this decodes", p.name);
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

/// One service case: as `pipeline_case`, for a service candidate
#[allow(clippy::too_many_arguments)]
fn service_case(
    p: &Serviced,
    st: &mut SvcStats,
    pipe_corpus: &[(Vec<u8>, Vec<u64>)],
    corpus: &[Vec<u8>],
    rng: &mut Rng,
    out_dir: &Option<String>,
    case: u64,
    bad: &mut u64,
) -> String {
    let (base, starts) = &pipe_corpus[rng.below(pipe_corpus.len() as u64) as usize];
    let t = edited(base, starts, corpus, rng);
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
    let c = got.counts;
    st.pooled += (c.pooled > 0) as u64;
    st.finished += (c.finished > 0) as u64;
    st.front_reuse += (c.prepared > got.fronts) as u64;
    st.back_reuse += (c.finished > got.backs) as u64;
    st.errors += got.res.is_err() as u64;
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

/// A multi-block input, edited as the pipeline and service cases edit it: one case in
/// eight valid; else one or two edits, aimed at a block's start (its header, tables and
/// first symbols: a block after the first where there is one, so that the blocks before
/// it are in flight), at the stream's end, or anywhere
fn edited(base: &[u8], starts: &[u64], corpus: &[Vec<u8>], rng: &mut Rng) -> Vec<u8> {
    let mut t = base.to_vec();
    let later = if starts.len() > 1 {
        &starts[1..]
    } else {
        starts
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
    t
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

fn cells_fuzz(secs: u64, seed: u64, out_dir: Option<String>) {
    use mibz2::bwt::CellsCounts;
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
    let mut corpus = cells::witnesses();
    for (_, bytes) in samples(&mut rng) {
        corpus.push(compress(&bytes[..bytes.len().min(150_000)], 9));
    }
    let (mut prod, mut forced) = (CellsCounts::default(), CellsCounts::default());
    let mut cases = 0;
    // Mandatory witnesses run even for a zero-second smoke check.
    for z in &corpus {
        let (p, f) = cells::check(z, mibz2::Kernels::detect());
        cells::add(&mut prod, p);
        cells::add(&mut forced, f);
        cases += 1;
    }
    cells::assert_reach(prod, forced);
    let deadline = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < deadline {
        let mut z = if cases % 3 == 0 {
            let n = 1 + rng.below(1000) as usize;
            let alphabet = 1 + rng.below(256);
            let ll = (0..n).map(|_| rng.below(alphabet) as u8).collect();
            craft(9, &[Spec::of(ll, rng.below(n as u64) as u32)], None)
        } else {
            corpus[rng.below(corpus.len() as u64) as usize].clone()
        };
        if cases % 4 != 0 {
            mutate(&mut z, &corpus, &mut rng, &near_start);
        }
        cases += 1;
        let k = if cases % 2 == 0 {
            mibz2::Kernels::scalar()
        } else {
            mibz2::Kernels::detect()
        };
        match std::panic::catch_unwind(|| cells::check(&z, k)) {
            Ok((p, f)) => {
                cells::add(&mut prod, p);
                cells::add(&mut forced, f);
            }
            Err(_) => {
                report(&out_dir, &z, cases, "cells differential failure");
                panic!("cells case {cases}, seed {seed}");
            }
        }
    }
    cells::assert_reach(prod, forced);
    println!("{cases} cells cases, seed {seed}, production {prod:?}, forced {forced:?}");
}
