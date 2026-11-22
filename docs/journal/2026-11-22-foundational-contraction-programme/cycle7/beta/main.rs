#![forbid(unsafe_code)]
//! Same-binary paired interleaved A/B + A/A benchmark of `BijectiveOpen` vs `v0.5.12 OpenCells`
//! across all 6 SHA-256 authenticated Wikidata blocks!

pub use mibz2::{bwt, runs, Kernels};

#[path = "/tmp/fcp-b/bijective_open_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod candidate;

use bwt::{InverseBwt, OpenCells};
use candidate::BijectiveOpen;
use mibz2::block::Symbols;
use runs::Runs;
use std::time::Instant;

struct BlockSpec {
    file: &'static str,
    n: usize,
    p: u32,
}

const BLOCKS: &[BlockSpec] = &[
    BlockSpec { file: "block-0.L",   n: 899_931, p: 653_171 },
    BlockSpec { file: "block-2.L",   n: 900_000, p: 423_885 },
    BlockSpec { file: "block-85.L",  n: 899_935, p: 363_590 },
    BlockSpec { file: "block-143.L", n: 899_869, p: 660_732 },
    BlockSpec { file: "block-210.L", n: 899_959, p: 583_550 },
    BlockSpec { file: "block-207.L", n: 899_998, p: 774_491 },
];

fn build_runs(l: &[u8], orig_ptr: u32) -> Runs {
    let mut used = [false; 256];
    for &b in l { used[b as usize] = true; }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 { if used[b] { seq_to_unseq[n_in_use] = b as u8; n_in_use += 1; } }
    let mut mtf: Vec<u8> = (0..n_in_use as u8).collect();
    let mut syms: Vec<u16> = Vec::with_capacity(l.len() / 2);
    let mut zrun = 0usize;
    let flush_zrun = |z: &mut usize, syms: &mut Vec<u16>| {
        let mut v = *z;
        while v > 0 { v -= 1; syms.push((v & 1) as u16); v >>= 1; }
        *z = 0;
    };
    for &byte in l {
        let mut idx = 0usize;
        while seq_to_unseq[mtf[idx] as usize] != byte { idx += 1; }
        if idx == 0 { zrun += 1; } else {
            flush_zrun(&mut zrun, &mut syms);
            let val = mtf[idx]; mtf.copy_within(0..idx, 1); mtf[0] = val;
            syms.push((idx + 1) as u16);
        }
    }
    flush_zrun(&mut zrun, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);
    let sym = Symbols { crc: 0, orig_ptr, seq_to_unseq, syms, eob, max_len: 900_000 };
    let mut r = Runs::default();
    assert_eq!(runs::front(&sym, &mut r, Kernels::scalar()).expect("front"), l.len());
    r
}

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 { vals[m / 2] } else { 0.5 * (vals[m / 2 - 1] + vals[m / 2]) }
}

fn main() {
    let rounds = 21usize;
    let warmup = 3usize;

    let mut loaded = Vec::new();
    let mut open_ref = OpenCells::default();
    let mut bij_ref = BijectiveOpen::default();
    let (mut out_a, mut out_b) = (Vec::new(), Vec::new());

    for spec in BLOCKS {
        let raw = std::fs::read(format!("/tmp/mibz2-blocks/{}", spec.file)).unwrap();
        let r = build_runs(&raw, spec.p);
        assert!(open_ref.invert_runs(&r, spec.n, spec.p, &mut out_a, Kernels::scalar()));
        assert!(bij_ref.invert_runs(&r, spec.n, spec.p, &mut out_b, Kernels::scalar()));
        assert_eq!(out_a, out_b, "PARITY MISMATCH on {}!", spec.file);
        loaded.push((spec, r));
    }
    println!("Verified 100% byte-exact parity (BijectiveOpen == v0.5.12 OpenCells) on all 6 authenticated blocks!\n");

    let (mut open_a, mut open_a2, mut bij_b) = (OpenCells::default(), OpenCells::default(), BijectiveOpen::default());
    let (mut o1, mut o2, mut o3) = (Vec::with_capacity(900_032), Vec::with_capacity(900_032), Vec::with_capacity(900_032));

    for _ in 0..warmup {
        for (spec, r) in &loaded {
            open_a.invert_runs(r, spec.n, spec.p, &mut o1, Kernels::scalar());
            open_a2.invert_runs(r, spec.n, spec.p, &mut o2, Kernels::scalar());
            bij_b.invert_runs(r, spec.n, spec.p, &mut o3, Kernels::scalar());
        }
    }

    let mut ta = vec![vec![0.0; rounds]; loaded.len()];
    let mut ta2 = vec![vec![0.0; rounds]; loaded.len()];
    let mut tb = vec![vec![0.0; rounds]; loaded.len()];

    for rnd in 0..rounds {
        for (bi, (spec, r)) in loaded.iter().enumerate() {
            if rnd % 2 == 0 {
                let s = Instant::now(); open_a.invert_runs(r, spec.n, spec.p, &mut o1, Kernels::scalar()); ta[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); bij_b.invert_runs(r, spec.n, spec.p, &mut o3, Kernels::scalar()); tb[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a2.invert_runs(r, spec.n, spec.p, &mut o2, Kernels::scalar()); ta2[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
            } else {
                let s = Instant::now(); bij_b.invert_runs(r, spec.n, spec.p, &mut o3, Kernels::scalar()); tb[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a2.invert_runs(r, spec.n, spec.p, &mut o2, Kernels::scalar()); ta2[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a.invert_runs(r, spec.n, spec.p, &mut o1, Kernels::scalar()); ta[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
            }
        }
    }

    println!("=== SAME-BINARY PAIRED INTERLEAVED A/B + A/A (21 rounds, 1T, lto=fat, cgu=1) ===");
    println!("{:<12} | {:>9} | {:>9} | {:>8} | {:>13} | {:>8}", "Block", "Open (ms)", "Open2(ms)", "A/A", "BijectiveOpen", "B/A");
    let (mut sa, mut sa2, mut sb) = (0.0, 0.0, 0.0);
    for (bi, (spec, _)) in loaded.iter().enumerate() {
        let ma = median(&mut ta[bi]); let ma2 = median(&mut ta2[bi]); let mb = median(&mut tb[bi]);
        sa += ma; sa2 += ma2; sb += mb;
        println!("{:<12} | {:>9.3} | {:>9.3} | {:>7.3}x | {:>13.3} | {:>7.3}x", spec.file, ma, ma2, ma2 / ma, mb, mb / ma);
    }
    println!("---------------------------------------------------------------------------");
    println!("{:<12} | {:>9.3} | {:>9.3} | {:>7.3}x | {:>13.3} | {:>7.3}x", "6-BLK SUM", sa, sa2, sa2 / sa, sb, sb / sa);
    println!("\nBijectiveOpen peak scratch = {} KiB ({:.2} MiB)", bij_b.scratch_kib(), bij_b.scratch_kib() as f64 / 1024.0);
}
