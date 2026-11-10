//! The backward table, a check and a price (docs/journal/2026-11-10-backward-table.md): the
//! inverse BWT's table in the LF direction, filled in one sequential pass from the last
//! column's runs, and walked backward. Nothing in the decoder calls it.
//!
//! ```text
//! lf_fill FILE.bz2 [ROUNDS]
//! ```
//!
//! **The table.** P[i] = L[i] | LF(i) << 8, with LF(i) = C(L[i]) + (the copies of L[i] before
//! row i). A run of L at rows p..p + len, of byte c with k copies of c before p, is the entries
//! (C(c) + k + j) << 8 | c: an arithmetic progression, written at the next len slots. So the
//! table is filled in the order the runs arrive, with one counter a byte and no sort.
//!
//! **The walk.** From `orig_ptr`, i ← LF(i), emitting L[i]: the block's bytes last to first.
//!
//! It checks, on every block, that the bytes written backward equal `bwt::walk`'s, and times
//! the fill (from a run list made before timing) against `bwt::build` (from the column), in
//! decode order with the arms alternating.

use std::time::Instant;

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

struct Block {
    ll: Vec<u8>,
    counts: [u32; 256],
    orig_ptr: u32,
    /// the maximal runs of `ll`: byte and length
    runs: Vec<(u8, u32)>,
}

/// The table from the runs: one pass, each run's entries consecutive
fn fill(runs: &[(u8, u32)], counts: &[u32; 256], p: &mut Vec<u32>) {
    let mut next = [0u32; 256];
    let mut cum = 0u32;
    for c in 0..256 {
        next[c] = cum;
        cum += counts[c];
    }
    p.clear();
    p.reserve(cum as usize);
    for &(c, len) in runs {
        let base = next[c as usize];
        next[c as usize] = base + len;
        p.extend((base..base + len).map(|row| row << 8 | c as u32));
    }
}

/// `fill` with each run written 8 entries at a time into slack the next run overwrites: no
/// loop over a short run's rows
fn fill_wide(runs: &[(u8, u32)], counts: &[u32; 256], p: &mut Vec<u32>) {
    let mut next = [0u32; 256];
    let mut cum = 0u32;
    for c in 0..256 {
        next[c] = cum;
        cum += counts[c];
    }
    let n = cum as usize;
    p.clear();
    p.reserve(n + 8);
    let base8: [u32; 8] = std::array::from_fn(|j| (j as u32) << 8);
    let mut at = 0usize;
    for &(c, len) in runs {
        let base = next[c as usize];
        next[c as usize] = base + len;
        let mut e = base << 8 | c as u32;
        let end = at + len as usize;
        assert!(end <= n);
        while at < end {
            let w: [u32; 8] = std::array::from_fn(|j| e + base8[j]);
            // SAFETY: at < end <= n, and n + 8 entries are reserved
            unsafe { (p.as_mut_ptr().add(at) as *mut [u32; 8]).write_unaligned(w) };
            at += 8;
            e += 8 << 8;
        }
        at = end;
    }
    assert_eq!(at, n);
    // SAFETY: every slot below n was written by the runs, which tile it in order
    unsafe { p.set_len(n) };
}

/// The plain backward walk: n bytes, written last to first
fn walk_back(p: &[u32], orig_ptr: u32, out: &mut Vec<u8>) {
    let n = p.len();
    out.clear();
    out.resize(n, 0);
    let mut i = orig_ptr as usize;
    for t in (0..n).rev() {
        let e = p[i];
        out[t] = e as u8;
        i = (e >> 8) as usize;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).expect("read");
    let rounds: usize = args.get(2).map_or(5, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut syms, mut bs) = (Vec::new(), Vec::new());
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let mut runs: Vec<(u8, u32)> = Vec::new();
        for &c in &ll {
            match runs.last_mut() {
                Some((d, len)) if *d == c => *len += 1,
                _ => runs.push((c, 1)),
            }
        }
        bs.push(Block {
            ll,
            counts,
            orig_ptr: b.orig_ptr,
            runs,
        });
        syms = b.syms;
    }
    // the check: every block's bytes, backward, are bwt::walk's
    let (mut tt, mut p, mut want, mut got) = (TVector::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut rows, mut runs) = (0usize, 0usize);
    for (i, b) in bs.iter().enumerate() {
        if b.ll.is_empty() {
            continue;
        }
        build(&b.ll, &b.counts, &mut tt);
        want.clear();
        walk(&tt, b.orig_ptr, &mut want);
        fill(&b.runs, &b.counts, &mut p);
        assert_eq!(p.len(), b.ll.len());
        walk_back(&p, b.orig_ptr, &mut got);
        assert!(want == got, "block {i} differs");
        rows += b.ll.len();
        runs += b.runs.len();
    }
    println!(
        "{} blocks, {rows} rows, {runs} runs ({:.4} a row): the backward walk's bytes are bwt::walk's",
        bs.len(),
        runs as f64 / rows as f64
    );
    // the price: the fill from runs against the build from the column, decode order
    {
        // the wide fill writes the same table
        let mut q = Vec::new();
        for b in &bs {
            fill(&b.runs, &b.counts, &mut p);
            fill_wide(&b.runs, &b.counts, &mut q);
            assert!(p == q);
        }
    }
    let mut best = [f64::MAX; 3];
    for round in 0..rounds {
        for a in 0..3 {
            let arm = (a + round) % 3;
            let t0 = Instant::now();
            for b in &bs {
                match arm {
                    0 => {
                        build(&b.ll, &b.counts, &mut tt);
                        std::hint::black_box(&tt);
                    }
                    1 => {
                        fill(&b.runs, &b.counts, &mut p);
                        std::hint::black_box(&p);
                    }
                    _ => {
                        fill_wide(&b.runs, &b.counts, &mut p);
                        std::hint::black_box(&p);
                    }
                }
            }
            let ns = t0.elapsed().as_nanos() as f64 / rows as f64;
            best[arm] = best[arm].min(ns);
            println!(
                "round {round} {}: {ns:.3} ns a row",
                [
                    "bwt::build, from the column  ",
                    "fill, from the runs (LF)     ",
                    "fill, 8 entries a store (LF) "
                ][arm]
            );
        }
    }
    println!(
        "fastest rounds: build {:.3}, fill {:.3}, wide fill {:.3} ns a row ({:.2} of the build)",
        best[0],
        best[1],
        best[2],
        best[2] / best[0]
    );
}
