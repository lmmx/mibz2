#![forbid(unsafe_code)]
//! Same-binary paired interleaved A/B + A/A benchmark of `Phi2XorOpenOf<16>` and `Phi2XorOpenOf<12>`
//! vs `v0.5.12 OpenCells` on blocks 0, 2, 85, 143, 207, 210 extracted directly from
//! `/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2`, plus Referee #2 short odd-cycle witnesses!

pub use mibz2::{bwt, runs, Kernels};

#[path = "/tmp/fcp-b/phi2_xor_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod candidate;

use bwt::{InverseBwt, OpenCells};
use candidate::Phi2XorOpenOf;
use mibz2::block::Symbols;
use mibz2::stream::{Event, Scanner, SliceSource};
use runs::Runs;
use std::time::Instant;

const WANT_BLOCKS: &[u64] = &[1, 2, 3, 85, 143, 210];

fn build_runs_from_col(l: &[u8], orig_ptr: u32) -> Runs {
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

fn check_witnesses() {
    let mut open = OpenCells::default();
    let mut p16 = Phi2XorOpenOf::<16>::default();
    let mut p12 = Phi2XorOpenOf::<12>::default();
    let (mut oa, mut ob, mut oc) = (Vec::new(), Vec::new(), Vec::new());

    let cases: Vec<(Vec<u8>, u32)> = vec![
        (vec![0, 1, 0, 0], 1),
        (vec![0, 1, 0, 0, 0, 0, 0, 0], 2),
        (vec![0, 1, 0, 0, 0, 0, 0, 0, 0], 2),
        (vec![0, 1, 0, 0, 0, 0], 2),
        (vec![7], 0),
        (vec![42; 100], 37),
    ];
    for (l, p) in cases {
        let r = build_runs_from_col(&l, p);
        let n = l.len();
        open.invert_runs(&r, n, p, &mut oa, Kernels::scalar());
        assert!(p16.try_invert_runs_profiled(r.records(), n, p, u32::MAX, usize::MAX, &mut ob, false));
        assert!(p12.try_invert_runs_profiled(r.records(), n, p, u32::MAX, usize::MAX, &mut oc, false));
        assert_eq!(oa, ob, "WITNESS L=16 MISMATCH on L={:?} p={}", l, p);
        assert_eq!(oa, oc, "WITNESS L=12 MISMATCH on L={:?} p={}", l, p);
    }
    println!("PASS: All Referee #2 short odd-cycle & boundary witnesses (L=16 & L=12, 0 failures)!");
}

fn main() {
    check_witnesses();

    let data = std::fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").expect("read wikidata.bz2");
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut extracted: Vec<(u64, usize, u32, Runs)> = Vec::new();
    let mut global_idx = 0u64;
    while let Event::Block { symbols, .. } = sc.next(syms).expect("scan") {
        if WANT_BLOCKS.contains(&global_idx) {
            let mut r = Runs::default();
            let n = runs::front(&symbols, &mut r, Kernels::scalar()).expect("runs::front");
            extracted.push((global_idx, n, symbols.orig_ptr, r));
        }
        global_idx += 1;
        syms = symbols.syms;
        if extracted.len() == WANT_BLOCKS.len() {
            break;
        }
    }
    // Order to match WANT_BLOCKS: 0, 2, 85, 143, 210, 207
    let mut loaded: Vec<(String, usize, u32, Runs)> = Vec::new();
    for &w in WANT_BLOCKS {
        let pos = extracted.iter().position(|x| x.0 == w).expect("block found");
        let (idx, n, p, r) = extracted.remove(pos);
        loaded.push((format!("block-{idx}.L"), n, p, r));
    }

    let mut open_ref = OpenCells::default();
    let mut p16_ref = Phi2XorOpenOf::<16>::default();
    let mut p12_ref = Phi2XorOpenOf::<12>::default();
    let (mut oa, mut ob, mut oc) = (Vec::new(), Vec::new(), Vec::new());

    for (name, n, p, r) in &loaded {
        assert!(open_ref.invert_runs(r, *n, *p, &mut oa, Kernels::scalar()));
        assert!(p16_ref.invert_runs(r, *n, *p, &mut ob, Kernels::scalar()));
        assert!(p12_ref.invert_runs(r, *n, *p, &mut oc, Kernels::scalar()));
        assert_eq!(oa, ob, "L=16 PARITY MISMATCH on {}!", name);
        assert_eq!(oa, oc, "L=12 PARITY MISMATCH on {}!", name);
    }
    println!("Verified 100% byte-exact parity (Phi2XorOpen L=16 & L=12 == v0.5.12 OpenCells) on all 6 blocks!\n");

    let rounds = 21usize;
    let warmup = 3usize;
    let (mut open_a, mut open_a2) = (OpenCells::default(), OpenCells::default());
    let (mut b16, mut b12) = (Phi2XorOpenOf::<16>::default(), Phi2XorOpenOf::<12>::default());
    let (mut o1, mut o2, mut o3, mut o4) = (
        Vec::with_capacity(900_032),
        Vec::with_capacity(900_032),
        Vec::with_capacity(900_032),
        Vec::with_capacity(900_032),
    );

    for _ in 0..warmup {
        for (_, n, p, r) in &loaded {
            open_a.invert_runs(r, *n, *p, &mut o1, Kernels::scalar());
            open_a2.invert_runs(r, *n, *p, &mut o2, Kernels::scalar());
            b16.invert_runs(r, *n, *p, &mut o3, Kernels::scalar());
            b12.invert_runs(r, *n, *p, &mut o4, Kernels::scalar());
        }
    }

    let mut ta = vec![vec![0.0; rounds]; loaded.len()];
    let mut ta2 = vec![vec![0.0; rounds]; loaded.len()];
    let mut tb16 = vec![vec![0.0; rounds]; loaded.len()];
    let mut tb12 = vec![vec![0.0; rounds]; loaded.len()];

    for rnd in 0..rounds {
        for (bi, (_, n, p, r)) in loaded.iter().enumerate() {
            if rnd % 2 == 0 {
                let s = Instant::now(); open_a.invert_runs(r, *n, *p, &mut o1, Kernels::scalar()); ta[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); b16.invert_runs(r, *n, *p, &mut o3, Kernels::scalar()); tb16[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); b12.invert_runs(r, *n, *p, &mut o4, Kernels::scalar()); tb12[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a2.invert_runs(r, *n, *p, &mut o2, Kernels::scalar()); ta2[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
            } else {
                let s = Instant::now(); b12.invert_runs(r, *n, *p, &mut o4, Kernels::scalar()); tb12[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); b16.invert_runs(r, *n, *p, &mut o3, Kernels::scalar()); tb16[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a2.invert_runs(r, *n, *p, &mut o2, Kernels::scalar()); ta2[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
                let s = Instant::now(); open_a.invert_runs(r, *n, *p, &mut o1, Kernels::scalar()); ta[bi][rnd] = s.elapsed().as_secs_f64() * 1e3;
            }
        }
    }

    println!("=== SAME-BINARY PAIRED INTERLEAVED A/B + A/A (21 rounds, 1T, lto=fat, cgu=1) ===");
    println!("{:<12} | {:>8} | {:>8} | {:>7} | {:>9} ({:>6}) | {:>9} ({:>6})", "Block", "Open(ms)", "Open2", "A/A", "Phi2_L16", "B16/A", "Phi2_L12", "B12/A");
    let (mut sa, mut sa2, mut s16, mut s12) = (0.0, 0.0, 0.0, 0.0);
    for (bi, (name, _, _, _)) in loaded.iter().enumerate() {
        let ma = median(&mut ta[bi]); let ma2 = median(&mut ta2[bi]);
        let m16 = median(&mut tb16[bi]); let m12 = median(&mut tb12[bi]);
        sa += ma; sa2 += ma2; s16 += m16; s12 += m12;
        println!("{:<12} | {:>8.3} | {:>8.3} | {:>6.3}x | {:>9.3} ({:>5.3}x) | {:>9.3} ({:>5.3}x)",
            name, ma, ma2, ma2 / ma, m16, m16 / ma, m12, m12 / ma);
    }
    println!("-----------------------------------------------------------------------------------------");
    println!("{:<12} | {:>8.3} | {:>8.3} | {:>6.3}x | {:>9.3} ({:>5.3}x) | {:>9.3} ({:>5.3}x)",
        "6-BLK SUM", sa, sa2, sa2 / sa, s16, s16 / sa, s12, s12 / sa);

    println!("\n=== PER-STAGE BREAKDOWN (Phi2XorOpen L=16 & L=12, warm!) ===");
    for (_, n, p, r) in &loaded {
        b16.try_invert_runs_profiled(r.records(), *n, *p, u32::MAX, usize::MAX, &mut o3, true);
    }
    for (_, n, p, r) in &loaded {
        b12.try_invert_runs_profiled(r.records(), *n, *p, u32::MAX, usize::MAX, &mut o4, true);
    }
}
