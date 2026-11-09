//! A mutation fuzz loop against libbzip2: valid streams (several shapes, levels, crafted
//! blocks, several streams) mutated (bits flipped, bytes set, cut, spliced, inserted,
//! deleted), each decoded by libbzip2 and by the decoders under test (tests/common's
//! `candidates`: this crate's, sequential on both kernel sets, and now and then parallel;
//! any other in-process decoder behind `Candidate`). Errors must be errors: never different bytes, never bytes
//! where libbzip2 fails, never a panic. Where libbzip2 decodes what this decoder refuses
//! (a randomised block, an oversubscribed code past its alphabet), the case is counted by
//! its error.
//!
//! The service cases (tests/common's `service_candidates`: the service core,
//! `StreamDecoder::service`, research, at svc2's plan of 10 threads and other shapes, on
//! both kernel sets, pushed or through `BzReader::parallel_service`): multi-block inputs
//! (`multi_block_corpus`), some edits aimed at a later block's header and tables or the
//! stream's end, some left valid; each driven at random write and read sizes, and one in
//! eight cancelled (or its reader dropped) after a random number of bytes. Each is held to
//! the plain parallel decoder exactly (the bytes before any error, and the error's kind,
//! block and bit; a cancelled case's bytes a prefix of its bytes) as well as to libbzip2.
//! Built with `--features trace`, each service case also counts what it reached, from the
//! decoder's own marks: a pooled build and walk (a `mibz2-coop` helper's share), a service
//! worker's finishing and its assembly of a walked block's segments, and the reuse of a
//! back set and a front set (more blocks taken than the hub has of each).
//!
//! Usage: fuzz SECONDS [SEED] [OUT_DIR for the inputs of any failure, or -] [MODE]
//! MODE: mixed (the default: every 8th case a service case), classic, or service

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
        "service" => 1,
        m => panic!("MODE is mixed, classic or service, not {m}"),
    };
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
    let svcs = service_candidates();
    let svc_corpus: Vec<(Vec<u8>, Vec<u64>)> = if every == 0 {
        Vec::new()
    } else {
        multi_block_corpus(&mut rng)
            .into_iter()
            .map(|(_, z)| {
                let starts = block_starts(&z);
                (z, starts)
            })
            .collect()
    };
    let mut svc_stats = vec![SvcStats::default(); svcs.len()];
    let deadline = Instant::now() + Duration::from_secs(secs);
    let (mut cases, mut bad) = (0u64, 0u64);
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    while Instant::now() < deadline {
        if every != 0 && (cases + 1) % every == 0 {
            cases += 1;
            let i = (cases / every) as usize % svcs.len();
            let outcome = service_case(
                &svcs[i],
                &mut svc_stats[i],
                &svc_corpus,
                &corpus,
                &mut rng,
                &out_dir,
                cases,
                &mut bad,
            );
            *outcomes.entry(format!("service: {outcome}")).or_default() += 1;
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
            "service cases: candidate, cases (cancelled), and of those counted (a trace \
             build), cases reaching a pooled build, a pooled walk, a service worker's \
             finishing, its assembly of segments, a back set reused, a front set reused"
        );
        for (c, st) in svcs.iter().zip(&svc_stats) {
            let reach = if st.counted == 0 {
                "not counted (build with --features trace)".to_string()
            } else {
                format!(
                    "of {}: build {}, walk {}, finish {}, assemble {}, back reuse {}, \
                     front reuse {}, prepared queue past teams {} (longest {})",
                    st.counted,
                    st.pooled_build,
                    st.pooled_walk,
                    st.finished,
                    st.assembled,
                    st.back_reuse,
                    st.front_reuse,
                    st.beyond_teams,
                    st.max_prepared
                )
            };
            println!(
                "{:>9}  {} ({} cancelled): {reach}",
                st.cases,
                c.name(),
                st.cancelled
            );
        }
    }
    std::process::exit(if bad == 0 { 0 } else { 1 });
}

/// A service candidate's counts over the run: its cases, those cancelled, and of those
/// counted (a trace build) the cases that reached each path
#[derive(Clone, Default)]
struct SvcStats {
    cases: u64,
    cancelled: u64,
    counted: u64,
    pooled_build: u64,
    pooled_walk: u64,
    finished: u64,
    assembled: u64,
    back_reuse: u64,
    front_reuse: u64,
    /// cases whose prepared queue grew past `teams` blocks (the lookahead is an admission
    /// threshold, not a cap: D045), and the longest it grew in any case
    beyond_teams: u64,
    max_prepared: u64,
}

/// What one service run reached, from the decoder's marks (`trace`): helpers' shares of a
/// pooled build and walk, a service worker's finishing and its assembly of segments, and
/// blocks taken by the teams and prepared by the service, against the hub's back and front
/// sets (`2 x teams` and `2 x teams + services`)
#[cfg(feature = "trace")]
fn reached(c: &Serviced, st: &mut SvcStats, traces: &[mibz2::trace::ThreadTrace]) {
    use mibz2::trace::Kind;
    let count = |prefix: &str, kind: Kind| -> u64 {
        traces
            .iter()
            .filter(|t| t.name.starts_with(prefix))
            .map(|t| t.marks.iter().filter(|m| m.kind == kind).count() as u64)
            .sum()
    };
    let teams_got: u64 = traces
        .iter()
        .filter(|t| !t.name.starts_with("mibz2-svc") && !t.name.starts_with("mibz2-coop"))
        .map(|t| t.marks.iter().filter(|m| m.kind == Kind::TeamGot).count() as u64)
        .sum();
    let longest = traces
        .iter()
        .flat_map(|t| t.marks.iter())
        .filter(|m| m.kind == Kind::Prepared)
        .map(|m| m.len)
        .max()
        .unwrap_or(0);
    st.beyond_teams += (longest > c.teams as u64) as u64;
    st.max_prepared = st.max_prepared.max(longest);
    st.counted += 1;
    st.pooled_build += (count("mibz2-coop", Kind::BuildStart) > 0) as u64;
    st.pooled_walk += (count("mibz2-coop", Kind::WalkStart) > 0) as u64;
    st.finished += (count("mibz2-svc-", Kind::RleStart) > 0) as u64;
    st.assembled += (count("mibz2-svc-", Kind::AssembleStart) > 0) as u64;
    st.back_reuse += (teams_got > 2 * c.teams as u64) as u64;
    st.front_reuse +=
        (count("mibz2-svc-", Kind::JobGot) > (2 * c.teams + c.services) as u64) as u64;
}

/// One service case: a multi-block input, edited (or not), decoded by `c` at random write
/// and read sizes (now and then cancelled), by the plain parallel decoder and by libbzip2;
/// the outcome's class
#[allow(clippy::too_many_arguments)]
fn service_case(
    c: &Serviced,
    st: &mut SvcStats,
    svc_corpus: &[(Vec<u8>, Vec<u64>)],
    corpus: &[Vec<u8>],
    rng: &mut Rng,
    out_dir: &Option<String>,
    case: u64,
    bad: &mut u64,
) -> String {
    let (base, starts) = &svc_corpus[rng.below(svc_corpus.len() as u64) as usize];
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
    let sizes = [509usize, 4093, 65_536, 100_003, 1 << 20];
    let how = Drive {
        chunk: sizes[rng.below(sizes.len() as u64) as usize],
        read: sizes[rng.below(sizes.len() as u64) as usize],
        cancel_after: (rng.below(8) == 0).then(|| rng.below(3_000_000) as usize),
    };
    st.cases += 1;
    st.cancelled += how.cancel_after.is_some() as u64;
    let want = oracle(&t);
    let (plain_out, plain_res) = plain_parallel(&t);
    #[cfg(feature = "trace")]
    mibz2::trace::set_level(1);
    let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.run(&t, how)));
    #[cfg(feature = "trace")]
    {
        mibz2::trace::set_level(0);
        let traces = mibz2::trace::take();
        if got.is_ok() {
            reached(c, st, &traces);
        }
    }
    let (out, res) = match got {
        Ok(r) => r,
        Err(_) => {
            *bad += 1;
            report(out_dir, &t, case, &format!("{}: panic", c.name()));
            return "PANIC".into();
        }
    };
    let cancelled = how.cancel_after.is_some()
        && matches!(&res, Err(e) if e.kind == mibz2::ErrorKind::Cancelled);
    let fail = |why: String, bad: &mut u64| {
        *bad += 1;
        report(
            out_dir,
            &t,
            case,
            &format!(
                "{} {how:?}: {why}: {:?} after {} bytes, the plain {:?} after {}",
                c.name(),
                res,
                out.len(),
                plain_res,
                plain_out.len()
            ),
        );
    };
    if cancelled {
        if !plain_out.starts_with(&out) {
            fail(
                "cancelled, not a prefix of the plain decoder's bytes".into(),
                bad,
            );
            return "CANCELLED, NOT A PREFIX OF THE PLAIN DECODER'S BYTES".into();
        }
        return "cancelled, a prefix of the plain decoder's bytes".into();
    }
    if res != plain_res || out != plain_out {
        fail("not the plain decoder's".into(), bad);
        return "NOT THE PLAIN DECODER'S BYTES OR ERROR".into();
    }
    match (&want, &res) {
        (Ok(a), Ok(())) if *a == out => "both decode, same bytes".into(),
        (Ok(_), Ok(())) => {
            fail("different bytes from libbzip2".into(), bad);
            "DIFFERENT BYTES".into()
        }
        (Err(_), Ok(())) => {
            fail("libbzip2 fails, this decodes".into(), bad);
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
