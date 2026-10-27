//! A mutation fuzz loop against libbzip2: valid streams (several shapes, levels, crafted
//! blocks, several streams) mutated (bits flipped, bytes set, cut, spliced, inserted,
//! deleted), each decoded by libbzip2 and by this decoder (sequential on both kernel sets,
//! and now and then parallel). Errors must be errors: never different bytes, never bytes
//! where libbzip2 fails, never a panic. Where libbzip2 decodes what this decoder refuses
//! (a randomised block, an oversubscribed code past its alphabet), the case is counted by
//! its error.
//!
//! Usage: fuzz SECONDS [SEED] [OUT_DIR for the inputs of any failure]

#[path = "../tests/common/mod.rs"]
mod common;

use bunzip::{Kernels, ErrorKind};
use common::*;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(60);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let out_dir = args.get(3).cloned();
    let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);

    let mut corpus: Vec<Vec<u8>> = Vec::new();
    for (_, data) in samples(&mut rng) {
        let small = &data[..data.len().min(150_000)];
        for level in [1, 9] {
            corpus.push(compress(small, level));
        }
    }
    corpus.push([compress(b"one", 9), compress(b"", 2), compress(&rng.bytes(5000), 1)].concat());
    for n in [5usize, 300, 70_000] {
        let ll: Vec<u8> = (0..n).map(|_| b"abc"[rng.below(3) as usize]).collect();
        let p = rng.below(n as u64) as u32;
        let z = craft(9, &[Spec::of(ll, p)], None);
        if oracle(&z).is_ok() {
            corpus.push(z);
        }
    }
    let ks = [Kernels::scalar(), Kernels::detect()];
    let deadline = Instant::now() + Duration::from_secs(secs);
    let (mut cases, mut bad) = (0u64, 0u64);
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    while Instant::now() < deadline {
        let base = &corpus[rng.below(corpus.len() as u64) as usize];
        let mut t = base.clone();
        let edits = 1 + rng.below(4);
        for _ in 0..edits {
            mutate(&mut t, &corpus, &mut rng);
        }
        cases += 1;
        let want = oracle(&t);
        let k = ks[(cases % 2) as usize];
        let got = match ours(&t, k) {
            Ok(r) => r,
            Err(_) => {
                bad += 1;
                report(&out_dir, &t, cases, "panic");
                continue;
            }
        };
        if cases % 16 == 0 {
            let p = parallel(&t, 1 + rng.below(4) as usize, 1 + rng.below(6) as usize, 1 + rng.below(70_000) as usize,
                             1 + rng.below(100_000) as usize, k);
            let same = match (&p, &got) {
                (Ok(a), Ok(b)) => a == b,
                (Err(a), Err(b)) => a.kind == b.kind,
                _ => false,
            };
            if !same {
                bad += 1;
                report(&out_dir, &t, cases, "parallel differs from sequential");
            }
        }
        let outcome = match (&want, &got) {
            (Ok(a), Ok(b)) if a == b => "both decode, same bytes".to_string(),
            (Ok(_), Ok(_)) => {
                bad += 1;
                report(&out_dir, &t, cases, "different bytes");
                "DIFFERENT BYTES".into()
            }
            (Err(_), Ok(_)) => {
                bad += 1;
                report(&out_dir, &t, cases, "libbzip2 fails, this decodes");
                "LIBBZIP2 FAILS, THIS DECODES".into()
            }
            (Err(_), Err(e)) => format!("both fail ({})", short(&e.kind)),
            (Ok(_), Err(e)) => format!("libbzip2 decodes, this refuses ({})", short(&e.kind)),
        };
        *outcomes.entry(outcome).or_default() += 1;
    }
    println!("{cases} cases in {secs} s (seed {seed}), {bad} failures");
    for (o, n) in &outcomes {
        println!("{n:>9}  {o}");
    }
    std::process::exit(if bad == 0 { 0 } else { 1 });
}

fn short(k: &ErrorKind) -> String {
    let s = format!("{k:?}");
    s.split([' ', '{', '(']).next().unwrap_or("").to_string()
}

fn mutate(t: &mut Vec<u8>, corpus: &[Vec<u8>], rng: &mut Rng) {
    if t.is_empty() {
        t.push(rng.next() as u8);
        return;
    }
    // (mostly near the start: the headers and tables, where the checks are)
    let pos = |rng: &mut Rng, n: usize| {
        if rng.below(2) == 0 {
            rng.below(n.min(400) as u64) as usize
        } else {
            rng.below(n as u64) as usize
        }
    };
    match rng.below(8) {
        0 | 1 | 2 => {
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
