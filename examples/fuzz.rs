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
//! Usage: fuzz SECONDS [SEED] [OUT_DIR for the inputs of any failure, or -] [MODE]
//! MODE: mixed (the default: every 8th case a pipeline case), classic, pipeline or service

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
        "pipeline" | "service" => 1,
        m => panic!("MODE is mixed, classic, pipeline or service, not {m}"),
    };
    let service = mode == "service";
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
    let deadline = Instant::now() + Duration::from_secs(secs);
    let (mut cases, mut bad) = (0u64, 0u64);
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    while Instant::now() < deadline {
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
    if service {
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
