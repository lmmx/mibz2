#![forbid(unsafe_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::Runs;
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

// Re-export so candidate.rs (`use crate::bwt`, `use crate::runs`, `use crate::Kernels`) compiles unchanged!
pub use mibz2::{bwt, runs};

#[path = "../../morse_ap4_candidate.rs"]
mod candidate;

struct BlockData {
    name: &'static str,
    n: usize,
    p: u32,
    runs: Runs,
}

/// Encode raw BWT last column `l` into MTF+RLE2 `Symbols` and call `runs::front_scalar`
/// to obtain the exact production `Runs` representation!
fn runs_from_l(l: &[u8], orig_ptr: u32) -> Runs {
    let mut used = [false; 256];
    for &b in l {
        used[usize::from(b)] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut unseq_to_seq = [0u8; 256];
    let mut alpha = 0usize;
    for b in 0..256 {
        if used[b] {
            seq_to_unseq[alpha] = b as u8;
            unseq_to_seq[b] = alpha as u8;
            alpha += 1;
        }
    }
    let mut mtf: Vec<u8> = (0..alpha as u8).collect();
    let mut syms = Vec::with_capacity(l.len() / 2 + 16);
    let mut zrun = 0usize;
    let flush_z = |zrun: &mut usize, syms: &mut Vec<u16>| {
        let mut z = *zrun;
        *zrun = 0;
        while z > 0 {
            if z & 1 == 1 {
                syms.push(0); // RUNA
                z = (z - 1) >> 1;
            } else {
                syms.push(1); // RUNB
                z = (z - 2) >> 1;
            }
        }
    };
    for &b in l {
        let s = unseq_to_seq[usize::from(b)];
        let pos = mtf.iter().position(|&x| x == s).unwrap();
        if pos == 0 {
            zrun += 1;
        } else {
            flush_z(&mut zrun, &mut syms);
            mtf.copy_within(0..pos, 1);
            mtf[0] = s;
            syms.push((pos + 1) as u16);
        }
    }
    flush_z(&mut zrun, &mut syms);
    let eob = (alpha + 1) as u16;
    syms.push(eob);

    let blk = Symbols {
        crc: 0,
        orig_ptr,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };
    let mut r = Runs::default();
    let n_got = runs::front_scalar(&blk, &mut r).expect("front_scalar");
    assert_eq!(n_got, l.len());
    r
}

fn load_block(name: &'static str, n: usize, p: u32) -> BlockData {
    let path = format!("/tmp/mibz2-blocks/{name}");
    let l = fs::read(&path).expect("read block");
    assert_eq!(l.len(), n);
    let r = runs_from_l(&l, p);
    BlockData {
        name,
        n,
        p,
        runs: r,
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let specs = [
        ("block-0.L", 899_931, 264_045u32),
        ("block-2.L", 900_000, 320_807),
        ("block-85.L", 899_935, 325_088),
        ("block-143.L", 899_869, 802_868),
        ("block-210.L", 899_959, 791_429),
        ("block-207.L", 899_998, 205_180),
    ];
    let blocks: Vec<BlockData> = specs
        .iter()
        .map(|&(name, n, p)| load_block(name, n, p))
        .collect();

    let mut open_a = OpenCells::default();
    let mut open_b = OpenCells::default();
    let mut morse = candidate::Candidate::default();
    let mut out_open = Vec::with_capacity(900_064);
    let mut out_morse = Vec::with_capacity(900_064);
    let k = Kernels::detect();

    for b in &blocks {
        assert!(open_a.invert_runs(&b.runs, b.n, b.p, &mut out_open, k));
        assert!(morse.invert_runs(&b.runs, b.n, b.p, &mut out_morse, k));
        assert_eq!(out_open, out_morse, "EXACTNESS MISMATCH on {}", b.name);
        assert!(open_b.invert_runs(&b.runs, b.n, b.p, &mut out_open, k));
    }
    println!("EXACT PARITY PASS: 6/6 authenticated Wikidata blocks match v0.5.12 OpenCells byte-for-byte!");

    let rounds = 21usize;
    let mut aa_a = vec![Vec::with_capacity(rounds); 6];
    let mut aa_b = vec![Vec::with_capacity(rounds); 6];
    for r in 0..rounds {
        for (bi, b) in blocks.iter().enumerate() {
            if r % 2 == 0 {
                let t0 = Instant::now();
                open_a.invert_runs(&b.runs, b.n, b.p, &mut out_open, k);
                aa_a[bi].push(t0.elapsed().as_secs_f64() * 1e3);
                let t1 = Instant::now();
                open_b.invert_runs(&b.runs, b.n, b.p, &mut out_morse, k);
                aa_b[bi].push(t1.elapsed().as_secs_f64() * 1e3);
            } else {
                let t1 = Instant::now();
                open_b.invert_runs(&b.runs, b.n, b.p, &mut out_morse, k);
                aa_b[bi].push(t1.elapsed().as_secs_f64() * 1e3);
                let t0 = Instant::now();
                open_a.invert_runs(&b.runs, b.n, b.p, &mut out_open, k);
                aa_a[bi].push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }

    let mut ab_open = vec![Vec::with_capacity(rounds); 6];
    let mut ab_morse = vec![Vec::with_capacity(rounds); 6];
    for r in 0..rounds {
        for (bi, b) in blocks.iter().enumerate() {
            if r % 2 == 0 {
                let t0 = Instant::now();
                open_a.invert_runs(&b.runs, b.n, b.p, &mut out_open, k);
                ab_open[bi].push(t0.elapsed().as_secs_f64() * 1e3);
                let t1 = Instant::now();
                morse.invert_runs(&b.runs, b.n, b.p, &mut out_morse, k);
                ab_morse[bi].push(t1.elapsed().as_secs_f64() * 1e3);
            } else {
                let t1 = Instant::now();
                morse.invert_runs(&b.runs, b.n, b.p, &mut out_morse, k);
                ab_morse[bi].push(t1.elapsed().as_secs_f64() * 1e3);
                let t0 = Instant::now();
                open_a.invert_runs(&b.runs, b.n, b.p, &mut out_open, k);
                ab_open[bi].push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }

    println!("\n=== SAME-BINARY PAIRED INTERLEAVED A/B + A/A (21 ROUNDS, LTO=FAT, CGU=1, 1T, v0.5.12) ===");
    println!("block       | Open(ms) | Morse(ms) |  ratio  | A/A ctrl");
    println!("------------+----------+-----------+---------+---------");
    let (mut sum_o, mut sum_m, mut sum_aa_a, mut sum_aa_b) = (0.0, 0.0, 0.0, 0.0);
    for (bi, b) in blocks.iter().enumerate() {
        let mo = median(ab_open[bi].clone());
        let mm = median(ab_morse[bi].clone());
        let ma = median(aa_a[bi].clone());
        let mb = median(aa_b[bi].clone());
        sum_o += mo;
        sum_m += mm;
        sum_aa_a += ma;
        sum_aa_b += mb;
        println!(
            "{:<11} | {:8.3} | {:9.3} | {:6.3}x | {:6.3}x",
            b.name,
            mo,
            mm,
            mm / mo,
            mb / ma
        );
    }
    println!("------------+----------+-----------+---------+---------");
    println!(
        "SUM 6 BLKS  | {:8.3} | {:9.3} | {:6.3}x | {:6.3}x",
        sum_o,
        sum_m,
        sum_m / sum_o,
        sum_aa_b / sum_aa_a
    );

    println!("\n=== PER-STAGE BREAKDOWN OF MorseAp4Open (MEDIAN OF 7) ===");
    println!("block       |   S1  S2+S3a AP4_fnd AP4_cmp S4_xor  Emit |  Sum  | m_crit (/m16)   | codes (wide) | PeakM");
    println!("------------+-------------------------------------------+-------+-----------------+--------------+------");
    let (mut ss1, mut ss2, mut saf, mut sac, mut ss4, mut sem) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (mut sm16, mut smc) = (0u64, 0u64);
    for b in &blocks {
        let mut s1s = vec![]; let mut s2s = vec![]; let mut afs = vec![];
        let mut acs = vec![]; let mut s4s = vec![]; let mut ems = vec![];
        let mut last = candidate::StageStats::default();
        for _ in 0..7 {
            last = morse.profile_invert_runs(&b.runs, b.n, b.p, &mut out_morse);
            s1s.push(last.s1_ms); s2s.push(last.s2_s3a_ms); afs.push(last.ap4_find_ms);
            acs.push(last.ap4_comp_ms); s4s.push(last.s4_xor_ms); ems.push(last.emit_ms);
        }
        let (v1, v2, vf, vc, v4, ve) = (median(s1s), median(s2s), median(afs), median(acs), median(s4s), median(ems));
        ss1 += v1; ss2 += v2; saf += vf; sac += vc; ss4 += v4; sem += ve;
        sm16 += u64::from(last.m_16); smc += u64::from(last.m_crit);
        println!(
            "{:<11} | {:5.2}  {:5.2}   {:5.2}   {:5.2}  {:5.2} {:5.2} | {:5.2} | {:6} ({:5.3}m) | {:5} ({:5}) | {:4}K",
            b.name, v1, v2, vf, vc, v4, ve, v1+v2+vf+vc+v4+ve,
            last.m_crit, last.m_crit as f64 / last.m_16 as f64,
            last.codes, last.wide, last.peak_bytes / 1024
        );
    }
    println!("------------+-------------------------------------------+-------+-----------------+--------------+------");
    println!(
        "SUM 6 BLKS  | {:5.2}  {:5.2}   {:5.2}   {:5.2}  {:5.2} {:5.2} | {:5.2} | {:6} ({:5.3}m) |",
        ss1, ss2, saf, sac, ss4, sem, ss1+ss2+saf+sac+ss4+sem, smc, smc as f64 / sm16 as f64
    );
}
