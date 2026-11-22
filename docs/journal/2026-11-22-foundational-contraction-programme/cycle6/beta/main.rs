#![forbid(unsafe_code)]
//! Same-binary paired interleaved A/B + A/A benchmark of `SelThinSq` vs `v0.5.12 OpenCells`
//! across all 6 authenticated Wikidata blocks (/tmp/mibz2-blocks/block-*.L),
//! plus exact byte-for-byte parity verification, A/A noise control, and PeakM scratch audit.

pub use mibz2::{bwt, runs, Kernels};

#[path = "/tmp/fcp-b/selthinsq_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod candidate;

use bwt::{InverseBwt, OpenCells};
use candidate::SelThinSq;
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

use mibz2::block::Symbols;

/// Build exact production 127-capped `Runs` from raw L bytes via MTF+RLE2 -> `runs::front`.
fn build_runs(l: &[u8], orig_ptr: u32) -> Runs {
    let mut used = [false; 256];
    for &b in l {
        used[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 {
        if used[b] {
            seq_to_unseq[n_in_use] = b as u8;
            n_in_use += 1;
        }
    }
    let mut mtf: Vec<u8> = (0..n_in_use as u8).collect();
    let mut syms: Vec<u16> = Vec::with_capacity(l.len() / 2);
    let mut zrun = 0usize;

    let flush_zrun = |z: &mut usize, syms: &mut Vec<u16>| {
        let mut v = *z;
        while v > 0 {
            v -= 1;
            syms.push((v & 1) as u16); // RUNA = 0, RUNB = 1
            v >>= 1;
        }
        *z = 0;
    };

    for &byte in l {
        let mut idx = 0usize;
        while seq_to_unseq[mtf[idx] as usize] != byte {
            idx += 1;
        }
        if idx == 0 {
            zrun += 1;
        } else {
            flush_zrun(&mut zrun, &mut syms);
            let val = mtf[idx];
            mtf.copy_within(0..idx, 1);
            mtf[0] = val;
            syms.push((idx + 1) as u16);
        }
    }
    flush_zrun(&mut zrun, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);

    let sym = Symbols {
        crc: 0,
        orig_ptr,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };
    let mut r = Runs::default();
    let got_n = runs::front(&sym, &mut r, Kernels::scalar()).expect("runs::front");
    assert_eq!(got_n, l.len());
    r
}

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 {
        vals[m / 2]
    } else {
        0.5 * (vals[m / 2 - 1] + vals[m / 2])
    }
}

fn main() {
    let rounds: usize = 21;
    let warmup: usize = 3;

    // Load all 6 blocks & verify exact parity between OpenCells and SelThinSq!
    let mut loaded: Vec<(&BlockSpec, Runs, Vec<u8>)> = Vec::new();
    let mut open_ref = OpenCells::default();
    let mut sel_ref = SelThinSq::default();
    let mut out_a = Vec::new();
    let mut out_b = Vec::new();

    for spec in BLOCKS {
        let path = format!("/tmp/mibz2-blocks/{}", spec.file);
        let raw = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert_eq!(raw.len(), spec.n);
        let r = build_runs(&raw, spec.p);
        let ok_a = open_ref.invert_runs(&r, spec.n, spec.p, &mut out_a, Kernels::scalar());
        let ok_b = sel_ref.invert_runs(&r, spec.n, spec.p, &mut out_b, Kernels::scalar());
        assert!(ok_a && ok_b);
        assert_eq!(out_a, out_b, "PARITY MISMATCH on {}!", spec.file);
        loaded.push((spec, r, out_a.clone()));
    }
    println!("Verified 100% byte-exact parity (SelThinSq == v0.5.12 OpenCells) on all 6 authenticated blocks!\n");

    let mut open_a = OpenCells::default();
    let mut open_a2 = OpenCells::default();
    let mut sel_b = SelThinSq::default();
    let mut out1 = Vec::with_capacity(900_032);
    let mut out2 = Vec::with_capacity(900_032);
    let mut out3 = Vec::with_capacity(900_032);

    // Warmup
    for _ in 0..warmup {
        for (spec, r, _) in &loaded {
            open_a.invert_runs(r, spec.n, spec.p, &mut out1, Kernels::scalar());
            open_a2.invert_runs(r, spec.n, spec.p, &mut out2, Kernels::scalar());
            sel_b.invert_runs(r, spec.n, spec.p, &mut out3, Kernels::scalar());
        }
    }

    let mut t_open = vec![vec![0.0f64; rounds]; loaded.len()];
    let mut t_open2 = vec![vec![0.0f64; rounds]; loaded.len()];
    let mut t_sel = vec![vec![0.0f64; rounds]; loaded.len()];

    // Paired interleaved A / B / A2 (alternating order per round to cancel thermal drift!)
    for rnd in 0..rounds {
        for (bi, (spec, r, _)) in loaded.iter().enumerate() {
            if rnd % 2 == 0 {
                let s0 = Instant::now();
                open_a.invert_runs(r, spec.n, spec.p, &mut out1, Kernels::scalar());
                t_open[bi][rnd] = s0.elapsed().as_secs_f64() * 1e3;

                let s1 = Instant::now();
                sel_b.invert_runs(r, spec.n, spec.p, &mut out3, Kernels::scalar());
                t_sel[bi][rnd] = s1.elapsed().as_secs_f64() * 1e3;

                let s2 = Instant::now();
                open_a2.invert_runs(r, spec.n, spec.p, &mut out2, Kernels::scalar());
                t_open2[bi][rnd] = s2.elapsed().as_secs_f64() * 1e3;
            } else {
                let s1 = Instant::now();
                sel_b.invert_runs(r, spec.n, spec.p, &mut out3, Kernels::scalar());
                t_sel[bi][rnd] = s1.elapsed().as_secs_f64() * 1e3;

                let s2 = Instant::now();
                open_a2.invert_runs(r, spec.n, spec.p, &mut out2, Kernels::scalar());
                t_open2[bi][rnd] = s2.elapsed().as_secs_f64() * 1e3;

                let s0 = Instant::now();
                open_a.invert_runs(r, spec.n, spec.p, &mut out1, Kernels::scalar());
                t_open[bi][rnd] = s0.elapsed().as_secs_f64() * 1e3;
            }
        }
    }

    println!("=== SAME-BINARY PAIRED INTERLEAVED A/B + A/A (21 rounds, 1T, lto=fat, cgu=1) ===");
    println!("{:<12} | {:>9} | {:>9} | {:>8} | {:>9} | {:>8}", "Block", "Open (ms)", "Open2(ms)", "A/A", "SelThinSq", "B/A");
    let mut sum_a = 0.0;
    let mut sum_a2 = 0.0;
    let mut sum_b = 0.0;
    for (bi, (spec, _, _)) in loaded.iter().enumerate() {
        let ma = median(&mut t_open[bi]);
        let ma2 = median(&mut t_open2[bi]);
        let mb = median(&mut t_sel[bi]);
        sum_a += ma;
        sum_a2 += ma2;
        sum_b += mb;
        println!(
            "{:<12} | {:>9.3} | {:>9.3} | {:>7.3}x | {:>9.3} | {:>7.3}x",
            spec.file, ma, ma2, ma2 / ma, mb, mb / ma
        );
    }
    println!("-----------------------------------------------------------------------");
    println!(
        "{:<12} | {:>9.3} | {:>9.3} | {:>7.3}x | {:>9.3} | {:>7.3}x",
        "6-BLK SUM", sum_a, sum_a2, sum_a2 / sum_a, sum_b, sum_b / sum_a
    );
    println!("\nSelThinSq peak scratch = {} KiB ({:.2} MiB)", sel_b.scratch_kib(), sel_b.scratch_kib() as f64 / 1024.0);
}
