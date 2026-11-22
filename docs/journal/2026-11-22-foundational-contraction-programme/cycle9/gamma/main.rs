#![forbid(unsafe_code)]

pub use mibz2::{bwt, runs, Kernels};

#[path = "../../thick_morse_open_candidate.rs"]
pub mod candidate;

use candidate::ThickMorseOpen;
use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use std::fs;
use std::time::Instant;

struct BlockSpec {
    file: &'static str,
    n: usize,
    p: u32,
}

const BLOCKS: [BlockSpec; 6] = [
    BlockSpec { file: "block-0.L", n: 899_931, p: 264_045 },
    BlockSpec { file: "block-2.L", n: 900_000, p: 320_807 },
    BlockSpec { file: "block-85.L", n: 899_935, p: 325_088 },
    BlockSpec { file: "block-143.L", n: 899_869, p: 802_868 },
    BlockSpec { file: "block-210.L", n: 899_959, p: 791_429 },
    BlockSpec { file: "block-207.L", n: 899_998, p: 205_180 },
];

fn build_runs_from_column(ll: &[u8], p: u32, k: Kernels) -> Runs {
    let mut present = [false; 256];
    for &b in ll {
        present[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut mtf_list = [0u8; 256];
    let mut alpha = 0usize;
    for b in 0..256usize {
        if present[b] {
            seq_to_unseq[alpha] = b as u8;
            mtf_list[alpha] = b as u8;
            alpha += 1;
        }
    }
    let mut syms: Vec<u16> = Vec::with_capacity(ll.len() / 2);
    let mut zrun = 0usize;
    let flush_zrun = |z: &mut usize, out: &mut Vec<u16>| {
        let mut r = *z;
        while r > 0 {
            if (r & 1) == 1 {
                out.push(0);
                r = (r - 1) >> 1;
            } else {
                out.push(1);
                r = (r - 2) >> 1;
            }
        }
        *z = 0;
    };
    for &b in ll {
        if mtf_list[0] == b {
            zrun += 1;
        } else {
            flush_zrun(&mut zrun, &mut syms);
            let mut pos = 1usize;
            while mtf_list[pos] != b {
                pos += 1;
            }
            for j in (1..=pos).rev() {
                mtf_list[j] = mtf_list[j - 1];
            }
            mtf_list[0] = b;
            syms.push((pos + 1) as u16);
        }
    }
    flush_zrun(&mut zrun, &mut syms);
    let eob = (alpha + 1) as u16;
    syms.push(eob);
    let symbols = Symbols {
        crc: 0,
        orig_ptr: p,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };
    let mut runs = Runs::default();
    let n = front(&symbols, &mut runs, k).expect("front failed");
    assert_eq!(n, ll.len());
    runs
}

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 { vals[m / 2] } else { 0.5 * (vals[m / 2 - 1] + vals[m / 2]) }
}

fn main() {
    let k = Kernels::detect();
    let mut loaded: Vec<(&BlockSpec, Runs, Vec<u8>)> = Vec::new();
    for spec in &BLOCKS {
        let path = format!("/tmp/mibz2-blocks/{}", spec.file);
        let ll = fs::read(&path).expect("read block");
        assert_eq!(ll.len(), spec.n);
        let runs = build_runs_from_column(&ll, spec.p, k);
        let mut ref_out = Vec::new();
        let mut open = OpenCells::default();
        assert!(open.invert_runs(&runs, spec.n, spec.p, &mut ref_out, k));
        assert_eq!(ref_out.len(), spec.n);
        loaded.push((spec, runs, ref_out));
    }

    println!("=== 1. STEADY-STATE PER-STAGE BREAKDOWN & EXACT PARITY (W = 16, 12, 10, 8, 6, 4; PLAIN vs THICK-MORSE R1) ===");
    let configs = [
        ("Open W=16", 16, false),
        ("TM-R1 W=16", 16, true),
        ("Open W=12", 12, false),
        ("TM-R1 W=12", 12, true),
        ("Open W=10", 10, false),
        ("TM-R1 W=10", 10, true),
        ("Open W=8", 8, false),
        ("TM-R1 W=8", 8, true),
        ("Open W=6", 6, false),
        ("TM-R1 W=6", 6, true),
        ("Open W=4", 4, false),
        ("TM-R1 W=4", 4, true),
    ];

    let mut out = Vec::with_capacity(900_064);
    for &(name, w, tm) in &configs {
        let mut kern = ThickMorseOpen::with_config(w, tm);
        for _ in 0..3 {
            for (spec, runs, _) in &loaded {
                assert!(kern.invert_runs(runs, spec.n, spec.p, &mut out, k));
            }
        }
        let (mut sm, mut sraw, mut sres, mut s1, mut stm, mut s23, mut s4, mut sem) =
            (0usize, 0usize, 0usize, 0u64, 0u64, 0u64, 0u64, 0u64);
        let mut max_cd = 0usize;
        for (spec, runs, ref_out) in &loaded {
            let ok = kern.run_profiled(runs.records(), spec.n, spec.p, &mut out);
            assert!(ok && out == *ref_out, "PARITY FAIL on {} ({name})", spec.file);
            let s = kern.last_stats;
            sm += s.m_ret;
            sraw += s.ns_raw;
            sres += s.ns_res;
            max_cd = max_cd.max(s.codes_total);
            s1 += s.ns_s1;
            stm += s.ns_tm;
            s23 += s.ns_s23;
            s4 += s.ns_s4;
            sem += s.ns_emit;
        }
        let tot_ms = (s1 + stm + s23 + s4 + sem) as f64 / 1e6;
        println!(
            "  {:11} | m={:6} ns={:5}->{:5} maxCd={:4} | S1={:.2} TM={:.2} S23={:.2} S4={:.2} Emit={:.2} => {:5.2}m | PeakM={:4} KiB",
            name, sm, sraw, sres, max_cd,
            s1 as f64 / 1e6, stm as f64 / 1e6, s23 as f64 / 1e6, s4 as f64 / 1e6, sem as f64 / 1e6,
            tot_ms, kern.allocated_scratch_bytes() / 1024
        );
    }

    println!("\n=== 2. SAME-BINARY PAIRED INTERLEAVED A/B + A/A BENCHMARK vs v0.5.12 OpenCells (1T, 25 ROUNDS) ===");
    let mut open_a = OpenCells::default();
    let mut open_a2 = OpenCells::default();
    let mut w16_tm = ThickMorseOpen::with_config(16, true);
    let mut w10_pl = ThickMorseOpen::with_config(10, false);
    let mut w10_tm = ThickMorseOpen::with_config(10, true);
    let mut w8_pl = ThickMorseOpen::with_config(8, false);
    let mut w8_tm = ThickMorseOpen::with_config(8, true);

    for _ in 0..3 {
        for (spec, runs, _) in &loaded {
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            w16_tm.invert_runs(runs, spec.n, spec.p, &mut out, k);
            w10_pl.invert_runs(runs, spec.n, spec.p, &mut out, k);
            w10_tm.invert_runs(runs, spec.n, spec.p, &mut out, k);
            w8_pl.invert_runs(runs, spec.n, spec.p, &mut out, k);
            w8_tm.invert_runs(runs, spec.n, spec.p, &mut out, k);
        }
    }

    let rounds = 25usize;
    let mut t_oa = vec![vec![0.0; rounds]; 6];
    let mut t_oa2 = vec![vec![0.0; rounds]; 6];
    let mut t_16t = vec![vec![0.0; rounds]; 6];
    let mut t_10p = vec![vec![0.0; rounds]; 6];
    let mut t_10t = vec![vec![0.0; rounds]; 6];
    let mut t_8p = vec![vec![0.0; rounds]; 6];
    let mut t_8t = vec![vec![0.0; rounds]; 6];

    for r in 0..rounds {
        for (bi, (spec, runs, _)) in loaded.iter().enumerate() {
            let t0 = Instant::now(); open_a.invert_runs(runs, spec.n, spec.p, &mut out, k); t_oa[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); w16_tm.invert_runs(runs, spec.n, spec.p, &mut out, k); t_16t[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); w10_pl.invert_runs(runs, spec.n, spec.p, &mut out, k); t_10p[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); w10_tm.invert_runs(runs, spec.n, spec.p, &mut out, k); t_10t[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); w8_pl.invert_runs(runs, spec.n, spec.p, &mut out, k); t_8p[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); w8_tm.invert_runs(runs, spec.n, spec.p, &mut out, k); t_8t[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
            let t0 = Instant::now(); open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k); t_oa2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
        }
    }

    println!(
        "{:11} | {:>7} {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5}",
        "Block", "Open(A)", "OpenA'", "A/A", "TM-W16", "xOp", "Pl-W10", "xOp", "TM-W10", "xOp", "Pl-W8", "xOp", "TM-W8", "xOp"
    );
    println!("{}", "-".repeat(106));
    let (mut soa, mut soa2, mut s16t, mut s10p, mut s10t, mut s8p, mut s8t) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for (bi, (spec, _, _)) in loaded.iter().enumerate() {
        let moa = median(&mut t_oa[bi]); let moa2 = median(&mut t_oa2[bi]);
        let m16t = median(&mut t_16t[bi]); let m10p = median(&mut t_10p[bi]); let m10t = median(&mut t_10t[bi]);
        let m8p = median(&mut t_8p[bi]); let m8t = median(&mut t_8t[bi]);
        soa += moa; soa2 += moa2; s16t += m16t; s10p += m10p; s10t += m10t; s8p += m8p; s8t += m8t;
        println!(
            "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3}",
            spec.file, moa, moa2, moa2/moa, m16t, m16t/moa, m10p, m10p/moa, m10t, m10t/moa, m8p, m8p/moa, m8t, m8t/moa
        );
    }
    println!("{}", "-".repeat(106));
    println!(
        "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3}",
        "6-BLK SUM", soa, soa2, soa2/soa, s16t, s16t/soa, s10p, s10p/soa, s10t, s10t/soa, s8p, s8p/soa, s8t, s8t/soa
    );
    println!(
        "\nSteady-State Peak Allocated Scratch: TM-W16={}K | Pl-W10={}K | TM-W10={}K | Pl-W8={}K | TM-W8={}K",
        w16_tm.allocated_scratch_bytes() / 1024,
        w10_pl.allocated_scratch_bytes() / 1024,
        w10_tm.allocated_scratch_bytes() / 1024,
        w8_pl.allocated_scratch_bytes() / 1024,
        w8_tm.allocated_scratch_bytes() / 1024,
    );
}
