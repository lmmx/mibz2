#![forbid(unsafe_code)]

pub use mibz2::{bwt, runs, Kernels};

#[path = "../../bijective_r1_open_candidate.rs"]
pub mod candidate;

use candidate::{BijectiveR1Open, R1Mode};
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

    println!("=== 1. STEADY-STATE PER-STAGE BREAKDOWN & EXACT PARITY (6 WIKIDATA BLOCKS) ===");
    let configs = [
        ("EmbOpen (ctrl)", R1Mode::BaselineOpen),
        ("FOrder-R1-W2", R1Mode::FOrderW2),
        ("FOrder-W2+W1h3", R1Mode::FOrderW2AndW1H3),
    ];

    let mut out = Vec::with_capacity(900_064);
    for &(name, mode) in &configs {
        let mut kern = BijectiveR1Open::with_mode(mode);
        for _ in 0..3 {
            for (spec, runs, _) in &loaded {
                assert!(kern.invert_runs(runs, spec.n, spec.p, &mut out, k));
            }
        }
        let (mut sm0, mut smw, mut sch, mut s1, mut s23, mut sr1, mut s4, mut sem) =
            (0usize, 0usize, 0usize, 0u64, 0u64, 0u64, 0u64, 0u64);
        let mut max_cd = 0usize;
        for (spec, runs, ref_out) in &loaded {
            let ok = kern.run_profiled(runs.records(), spec.n, spec.p, &mut out);
            assert!(ok && out == *ref_out, "PARITY FAIL on {} ({name})", spec.file);
            let s = kern.last_stats;
            sm0 += s.m_open;
            smw += s.m_walk_steps;
            sch += s.r1_chains;
            max_cd = max_cd.max(s.codes_total);
            s1 += s.ns_s1;
            s23 += s.ns_s23;
            sr1 += s.ns_r1;
            s4 += s.ns_s4;
            sem += s.ns_emit;
        }
        let tot_ms = (s1 + s23 + sr1 + s4 + sem) as f64 / 1e6;
        println!(
            "  {:15} | m={:6} ({:5.1}% sav) | ch={:5} maxCd={:5} | S1={:.2} S23={:.2} R1={:.2} S4={:.2} Emit={:.2} => {:.2}m | PeakM={:4} KiB",
            name,
            smw,
            100.0 * (sm0 - smw) as f64 / sm0 as f64,
            sch,
            max_cd,
            s1 as f64 / 1e6,
            s23 as f64 / 1e6,
            sr1 as f64 / 1e6,
            s4 as f64 / 1e6,
            sem as f64 / 1e6,
            tot_ms,
            kern.allocated_scratch_bytes() / 1024,
        );
    }

    println!("\n=== 2. SAME-BINARY PAIRED INTERLEAVED A/B + A/A BENCHMARK vs v0.5.12 OpenCells (1T, 21 ROUNDS) ===");
    let mut open_a = OpenCells::default();
    let mut open_a2 = OpenCells::default();
    let mut emb_op = BijectiveR1Open::with_mode(R1Mode::BaselineOpen);
    let mut f_w2 = BijectiveR1Open::with_mode(R1Mode::FOrderW2);
    let mut f_w1 = BijectiveR1Open::with_mode(R1Mode::FOrderW2AndW1H3);

    for _ in 0..3 {
        for (spec, runs, _) in &loaded {
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            emb_op.invert_runs(runs, spec.n, spec.p, &mut out, k);
            f_w2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            f_w1.invert_runs(runs, spec.n, spec.p, &mut out, k);
        }
    }

    let rounds = 21usize;
    let mut t_oa = vec![vec![0.0; rounds]; 6];
    let mut t_oa2 = vec![vec![0.0; rounds]; 6];
    let mut t_eo = vec![vec![0.0; rounds]; 6];
    let mut t_w2 = vec![vec![0.0; rounds]; 6];
    let mut t_w1 = vec![vec![0.0; rounds]; 6];

    for r in 0..rounds {
        for (bi, (spec, runs, _)) in loaded.iter().enumerate() {
            let t0 = Instant::now();
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            emb_op.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_eo[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            f_w2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_w2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            f_w1.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_w1[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
        }
    }

    println!(
        "{:11} | {:>7} {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5}",
        "Block", "Open(A)", "OpenA'", "A/A", "EmbOp", "xOp", "F-R1W2", "xOp", "F-W2h3", "xOp"
    );
    println!("{}", "-".repeat(78));
    let (mut soa, mut soa2, mut seo, mut sw2, mut sw1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (bi, (spec, _, _)) in loaded.iter().enumerate() {
        let moa = median(&mut t_oa[bi]);
        let moa2 = median(&mut t_oa2[bi]);
        let meo = median(&mut t_eo[bi]);
        let mw2 = median(&mut t_w2[bi]);
        let mw1 = median(&mut t_w1[bi]);
        soa += moa; soa2 += moa2; seo += meo; sw2 += mw2; sw1 += mw1;
        println!(
            "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x",
            spec.file, moa, moa2, moa2 / moa, meo, meo / moa, mw2, mw2 / moa, mw1, mw1 / moa
        );
    }
    println!("{}", "-".repeat(78));
    println!(
        "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x",
        "6-BLK SUM", soa, soa2, soa2 / soa, seo, seo / soa, sw2, sw2 / soa, sw1, sw1 / soa
    );
    println!(
        "\nSteady-State Peak Allocated Scratch: EmbOp={}K | F-R1W2={}K | F-W2h3={}K",
        emb_op.allocated_scratch_bytes() / 1024,
        f_w2.allocated_scratch_bytes() / 1024,
        f_w1.allocated_scratch_bytes() / 1024,
    );
}
