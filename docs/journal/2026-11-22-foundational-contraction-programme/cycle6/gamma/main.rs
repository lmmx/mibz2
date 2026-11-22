#![forbid(unsafe_code)]

pub use mibz2::{bwt, runs, Kernels};

#[path = "../../tube_open_candidate.rs"]
pub mod candidate;

use candidate::{TubeMode, TubeOpen};
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
        ("EmbOpen (ctrl)", TubeMode::BaselineOpen, 1),
        ("TubeUniV6 t=1", TubeMode::TubeUniV6, 1),
        ("TubeUniV6 t=2", TubeMode::TubeUniV6, 2),
        ("TubeUniV6 t=4", TubeMode::TubeUniV6, 4),
        ("TubeUniV6 t=8", TubeMode::TubeUniV6, 8),
        ("TubeBiXor t=2", TubeMode::TubeBiXorV6, 2),
        ("TubeBiXor t=4", TubeMode::TubeBiXorV6, 4),
    ];

    let mut out = Vec::with_capacity(900_064);
    for &(name, mode, tau) in &configs {
        let mut kern = TubeOpen::with_config(mode, tau);
        for _ in 0..2 {
            for (spec, runs, _) in &loaded {
                assert!(kern.invert_runs(runs, spec.n, spec.p, &mut out, k));
            }
        }
        let (mut sm0, mut smt, mut sh, mut st, mut s1, mut s23, mut s4, mut stub, mut sem) =
            (0usize, 0usize, 0usize, 0usize, 0u64, 0u64, 0u64, 0u64, 0u64);
        let mut max_cd = 0usize;
        for (spec, runs, ref_out) in &loaded {
            let ok = kern.run_profiled(runs.records(), spec.n, spec.p, &mut out);
            assert!(ok && out == *ref_out, "PARITY FAIL on {} ({name})", spec.file);
            let s = kern.last_stats;
            sm0 += s.m_open;
            smt += s.m_tube_steps;
            sh += s.tube_hops;
            st += s.tubes_applied;
            max_cd = max_cd.max(s.codes_total);
            s1 += s.ns_s1;
            s23 += s.ns_s23;
            s4 += s.ns_s4;
            stub += s.ns_tube;
            sem += s.ns_emit;
        }
        let tot_ms = (s1 + s23 + s4 + stub + sem) as f64 / 1e6;
        println!(
            "  {:15} | m={:6} ({:5.1}% sav) | hops={:5} tubes={:5} maxCd={:4} | S1={:.2} S23={:.2} S4={:.2} Tube={:.2} Emit={:.2} => {:.2}m | PeakM={:4} KiB",
            name,
            smt,
            100.0 * (sm0 - smt) as f64 / sm0 as f64,
            sh,
            st,
            max_cd,
            s1 as f64 / 1e6,
            s23 as f64 / 1e6,
            s4 as f64 / 1e6,
            stub as f64 / 1e6,
            sem as f64 / 1e6,
            tot_ms,
            kern.allocated_scratch_bytes() / 1024,
        );
    }

    println!("\n=== 2. SAME-BINARY PAIRED INTERLEAVED A/B + A/A BENCHMARK vs v0.5.12 OpenCells (1T, 21 ROUNDS) ===");
    let mut open_a = OpenCells::default();
    let mut open_a2 = OpenCells::default();
    let mut emb_open = TubeOpen::with_config(TubeMode::BaselineOpen, 1);
    let mut uni_t2 = TubeOpen::with_config(TubeMode::TubeUniV6, 2);
    let mut uni_t4 = TubeOpen::with_config(TubeMode::TubeUniV6, 4);
    let mut bix_t2 = TubeOpen::with_config(TubeMode::TubeBiXorV6, 2);
    let mut bix_t4 = TubeOpen::with_config(TubeMode::TubeBiXorV6, 4);

    for _ in 0..3 {
        for (spec, runs, _) in &loaded {
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            emb_open.invert_runs(runs, spec.n, spec.p, &mut out, k);
            uni_t2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            uni_t4.invert_runs(runs, spec.n, spec.p, &mut out, k);
            bix_t2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            bix_t4.invert_runs(runs, spec.n, spec.p, &mut out, k);
        }
    }

    let rounds = 21usize;
    let mut t_oa = vec![vec![0.0; rounds]; 6];
    let mut t_oa2 = vec![vec![0.0; rounds]; 6];
    let mut t_eo = vec![vec![0.0; rounds]; 6];
    let mut t_u2 = vec![vec![0.0; rounds]; 6];
    let mut t_u4 = vec![vec![0.0; rounds]; 6];
    let mut t_b2 = vec![vec![0.0; rounds]; 6];
    let mut t_b4 = vec![vec![0.0; rounds]; 6];

    for r in 0..rounds {
        for (bi, (spec, runs, _)) in loaded.iter().enumerate() {
            let t0 = Instant::now();
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            emb_open.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_eo[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            uni_t2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_u2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            uni_t4.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_u4[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            bix_t2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_b2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            bix_t4.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_b4[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
        }
    }

    println!(
        "{:11} | {:>7} {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5}",
        "Block", "Open(A)", "OpenA'", "A/A", "EmbOp", "xOp", "Uni-t2", "xOp", "Uni-t4", "xOp", "BiX-t2", "xOp", "BiX-t4", "xOp"
    );
    println!("{}", "-".repeat(106));
    let (mut soa, mut soa2, mut seo, mut su2, mut su4, mut sb2, mut sb4) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for (bi, (spec, _, _)) in loaded.iter().enumerate() {
        let moa = median(&mut t_oa[bi]);
        let moa2 = median(&mut t_oa2[bi]);
        let meo = median(&mut t_eo[bi]);
        let mu2 = median(&mut t_u2[bi]);
        let mu4 = median(&mut t_u4[bi]);
        let mb2 = median(&mut t_b2[bi]);
        let mb4 = median(&mut t_b4[bi]);
        soa += moa; soa2 += moa2; seo += meo; su2 += mu2; su4 += mu4; sb2 += mb2; sb4 += mb4;
        println!(
            "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x",
            spec.file, moa, moa2, moa2 / moa, meo, meo / moa, mu2, mu2 / moa, mu4, mu4 / moa, mb2, mb2 / moa, mb4, mb4 / moa
        );
    }
    println!("{}", "-".repeat(106));
    println!(
        "{:11} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x | {:6.3}m {:4.2}x",
        "6-BLK SUM", soa, soa2, soa2 / soa, seo, seo / soa, su2, su2 / soa, su4, su4 / soa, sb2, sb2 / soa, sb4, sb4 / soa
    );
    println!(
        "\nSteady-State Peak Allocated Scratch: EmbOp={}K | Uni-t2={}K | Uni-t4={}K | BiX-t2={}K | BiX-t4={}K",
        emb_open.allocated_scratch_bytes() / 1024,
        uni_t2.allocated_scratch_bytes() / 1024,
        uni_t4.allocated_scratch_bytes() / 1024,
        bix_t2.allocated_scratch_bytes() / 1024,
        bix_t4.allocated_scratch_bytes() / 1024,
    );
}
