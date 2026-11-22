#![forbid(unsafe_code)]
#![allow(dead_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

mod open_local;

use open_local::{Kernel as LocalKernel, ZeroRvMode};

struct BlockMeta {
    file: &'static str,
    n: usize,
    p: u32,
}

const BLOCKS: &[BlockMeta] = &[
    BlockMeta { file: "block-0.L",   n: 899931, p: 264045 },
    BlockMeta { file: "block-2.L",   n: 900000, p: 320807 },
    BlockMeta { file: "block-85.L",  n: 899935, p: 325088 },
    BlockMeta { file: "block-143.L", n: 899869, p: 802868 },
    BlockMeta { file: "block-207.L", n: 899998, p: 205180 },
    BlockMeta { file: "block-210.L", n: 899959, p: 791429 },
];

fn encode_symbols(l: &[u8], p: u32) -> Symbols {
    let mut present = [false; 256];
    for &b in l { present[b as usize] = true; }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 { if present[b] { seq_to_unseq[n_in_use] = b as u8; n_in_use += 1; } }
    let mut mtf = [0u8; 256];
    mtf[..n_in_use].copy_from_slice(&seq_to_unseq[..n_in_use]);
    let mut syms = Vec::with_capacity(l.len() / 2);
    let mut zr = 0usize;
    let flush = |zr: &mut usize, syms: &mut Vec<u16>| {
        let mut r = *zr;
        while r > 0 {
            if r & 1 == 1 { syms.push(0); r = (r - 1) >> 1; }
            else { syms.push(1); r = (r - 2) >> 1; }
        }
        *zr = 0;
    };
    for &b in l {
        let pos = mtf[..n_in_use].iter().position(|&x| x == b).unwrap();
        if pos == 0 { zr += 1; }
        else {
            flush(&mut zr, &mut syms);
            syms.push((pos + 1) as u16);
            for k in (1..=pos).rev() { mtf[k] = mtf[k - 1]; }
            mtf[0] = b;
        }
    }
    flush(&mut zr, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);
    Symbols { crc: 0, orig_ptr: p, seq_to_unseq, syms, eob, max_len: 900_000 }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

struct LoadedBlock {
    name: &'static str,
    n: usize,
    p: u32,
    runs: Runs,
    expected: Vec<u8>,
}

#[derive(Clone, Copy)]
enum Arm {
    ShippedA,
    ShippedAControl,
    LocalPlain,
    ZeroRv(ZeroRvMode),
}

impl Arm {
    fn name(self) -> String {
        match self {
            Arm::ShippedA => "OpenCells(shipped-A)".into(),
            Arm::ShippedAControl => "OpenCells(shipped-A')".into(),
            Arm::LocalPlain => "OpenLocal(L=16)".into(),
            Arm::ZeroRv(ZeroRvMode::Exact { min_w }) => format!("ZeroRV-Exact(w>={})", min_w),
            Arm::ZeroRv(ZeroRvMode::Contained { min_w }) => format!("ZeroRV-Cont(w>={})", min_w),
        }
    }
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let kernels = Kernels::detect();
    let mut blocks = Vec::new();
    let mut shipped = OpenCells::default();
    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).unwrap();
        let syms = encode_symbols(&l, b.p);
        let mut runs = Runs::default();
        front_scalar(&syms, &mut runs).unwrap();
        let mut expected = Vec::new();
        assert!(shipped.invert_runs(&runs, b.n, b.p, &mut expected, kernels));
        blocks.push(LoadedBlock { name: b.file, n: b.n, p: b.p, runs, expected });
    }

    let arms = [
        Arm::ShippedA,
        Arm::ShippedAControl,
        Arm::LocalPlain,
        Arm::ZeroRv(ZeroRvMode::Exact { min_w: 2 }),
        Arm::ZeroRv(ZeroRvMode::Exact { min_w: 3 }),
        Arm::ZeroRv(ZeroRvMode::Exact { min_w: 4 }),
        Arm::ZeroRv(ZeroRvMode::Exact { min_w: 6 }),
        Arm::ZeroRv(ZeroRvMode::Contained { min_w: 2 }),
        Arm::ZeroRv(ZeroRvMode::Contained { min_w: 3 }),
        Arm::ZeroRv(ZeroRvMode::Contained { min_w: 4 }),
    ];

    // 1. Exact correctness check on all 6 blocks + small exhaustive check!
    println!("=== CORRECTNESS CHECK (ALL ARMS x 6 AUTHENTICATED BLOCKS) ===");
    for &arm in &arms {
        let mut sk = OpenCells::default();
        let mut lk = LocalKernel::default();
        let mut out = Vec::new();
        let mut sum_elim = 0usize;
        let mut sum_tuns = 0usize;
        for b in &blocks {
            match arm {
                Arm::ShippedA | Arm::ShippedAControl => {
                    assert!(sk.invert_runs(&b.runs, b.n, b.p, &mut out, kernels));
                }
                Arm::LocalPlain => {
                    lk.run_staged(b.runs.records(), b.n, b.p, 16, &mut out);
                }
                Arm::ZeroRv(mode) => {
                    let (_, _, elim, tuns) = lk.run_zerorv_staged(b.runs.records(), b.n, b.p, 16, mode, &mut out);
                    sum_elim += elim;
                    sum_tuns += tuns;
                }
            }
            assert_eq!(out, b.expected, "Output mismatch for {} on {}", arm.name(), b.name);
        }
        if sum_elim > 0 {
            println!("  [PASS] {:<24} (eliminated {} rows = {:.1}% of m_16, {} tunnels)",
                arm.name(), sum_elim, 100.0 * sum_elim as f64 / 895391.0, sum_tuns);
        } else {
            println!("  [PASS] {}", arm.name());
        }
    }

    // 2. 1T interleaved benchmark (15 rounds, 6 blocks summed, with full stage breakdown!)
    let rounds_1t = 15;
    println!("\n=== 1T BENCHMARK ({} interleaved rounds, 6 blocks summed, median ms) ===", rounds_1t);
    println!("{:<24} | {:>7} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6}",
        "Arm", "Total", "vs A", "1stP", "Induce", "TunRec", "Wire", "Patch", "Emit");
    println!("{}", "-".repeat(94));

    let mut tot_1t: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut stg_1t: Vec<[Vec<f64>; 6]> = (0..arms.len()).map(|_| std::array::from_fn(|_| Vec::new())).collect();
    let mut per_blk_1t: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); blocks.len()]; arms.len()];

    let mut sa = OpenCells::default();
    let mut sac = OpenCells::default();
    let mut lks: Vec<LocalKernel> = (0..arms.len()).map(|_| LocalKernel::default()).collect();
    let mut out = Vec::with_capacity(900_016);

    for r in 0..(rounds_1t + 2) {
        for (ai, &arm) in arms.iter().enumerate() {
            let mut sum_tot = 0.0;
            let mut sum_s = [0.0; 6];
            for (bi, b) in blocks.iter().enumerate() {
                let dt = match arm {
                    Arm::ShippedA => {
                        let t0 = Instant::now();
                        sa.invert_runs(&b.runs, b.n, b.p, &mut out, kernels);
                        t0.elapsed().as_secs_f64() * 1000.0
                    }
                    Arm::ShippedAControl => {
                        let t0 = Instant::now();
                        sac.invert_runs(&b.runs, b.n, b.p, &mut out, kernels);
                        t0.elapsed().as_secs_f64() * 1000.0
                    }
                    Arm::LocalPlain => {
                        let (_, s5) = lks[ai].run_staged(b.runs.records(), b.n, b.p, 16, &mut out);
                        // Map 5 stages -> [1stP, Induce, 0, Wire, Inline, Emit]
                        sum_s[0] += s5[0] * 1000.0;
                        sum_s[1] += s5[1] * 1000.0;
                        sum_s[3] += s5[2] * 1000.0;
                        sum_s[4] += s5[3] * 1000.0;
                        sum_s[5] += s5[4] * 1000.0;
                        s5.iter().sum::<f64>() * 1000.0
                    }
                    Arm::ZeroRv(mode) => {
                        let (_, s6, _, _) = lks[ai].run_zerorv_staged(b.runs.records(), b.n, b.p, 16, mode, &mut out);
                        for k in 0..6 { sum_s[k] += s6[k] * 1000.0; }
                        s6.iter().sum::<f64>() * 1000.0
                    }
                };
                sum_tot += dt;
                if r >= 2 {
                    per_blk_1t[ai][bi].push(dt);
                }
            }
            if r >= 2 {
                tot_1t[ai].push(sum_tot);
                for k in 0..6 { stg_1t[ai][k].push(sum_s[k]); }
            }
        }
    }

    let base_1t = median(tot_1t[0].clone());
    for (ai, &arm) in arms.iter().enumerate() {
        let t = median(tot_1t[ai].clone());
        match arm {
            Arm::ShippedA | Arm::ShippedAControl => {
                println!("{:<24} | {:>7.3} | {:>5.3}x | {:>6} | {:>6} | {:>6} | {:>6} | {:>6} | {:>6}",
                    arm.name(), t, t / base_1t, "-", "-", "-", "-", "-", "-");
            }
            _ => {
                println!("{:<24} | {:>7.3} | {:>5.3}x | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3} | {:>6.3}",
                    arm.name(), t, t / base_1t,
                    median(stg_1t[ai][0].clone()),
                    median(stg_1t[ai][1].clone()),
                    median(stg_1t[ai][2].clone()),
                    median(stg_1t[ai][3].clone()),
                    median(stg_1t[ai][4].clone()),
                    median(stg_1t[ai][5].clone()));
            }
        }
    }

    println!("\nPer-block 1T median ms (Shipped-A vs Local16 vs ZeroRV-Exact(w>=3) vs ZeroRV-Cont(w>=3) vs ZeroRV-Cont(w>=2)):");
    println!("{:<12} | {:>7} | {:>7} | {:>7} ({:>5}) | {:>7} ({:>5}) | {:>7} ({:>5})",
        "Block", "ShipA", "Loc16", "Ex(w3)", "vsA", "Co(w3)", "vsA", "Co(w2)", "vsA");
    for (bi, b) in blocks.iter().enumerate() {
        let a = median(per_blk_1t[0][bi].clone());
        let l16 = median(per_blk_1t[2][bi].clone());
        let ex3 = median(per_blk_1t[4][bi].clone());
        let co3 = median(per_blk_1t[8][bi].clone());
        let co2 = median(per_blk_1t[7][bi].clone());
        println!("{:<12} | {:>7.3} | {:>7.3} | {:>7.3} ({:>4.2}x) | {:>7.3} ({:>4.2}x) | {:>7.3} ({:>4.2}x)",
            b.name, a, l16, ex3, ex3/a, co3, co3/a, co2, co2/a);
    }

    // 3. 4W Concurrent Benchmark!
    let blocks_arc = Arc::new(blocks);
    let rounds_4w = 15;
    let passes_per_worker = 3;
    println!("\n=== 4W CONCURRENT BENCHMARK ({} interleaved rounds, 4 threads x 3 passes x 6 blocks = 72 blocks/round, median wall ms) ===", rounds_4w);
    let mut tot_4w: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    for r in 0..(rounds_4w + 2) {
        for (ai, &arm) in arms.iter().enumerate() {
            let barrier = Arc::new(Barrier::new(4));
            let mut handles = Vec::with_capacity(4);
            for tid in 0..4usize {
                let blks = Arc::clone(&blocks_arc);
                let bar = Arc::clone(&barrier);
                handles.push(thread::spawn(move || {
                    let mut sk = OpenCells::default();
                    let mut lk = LocalKernel::default();
                    let mut out = Vec::with_capacity(900_016);
                    let b0 = &blks[tid % blks.len()];
                    match arm {
                        Arm::ShippedA | Arm::ShippedAControl => { sk.invert_runs(&b0.runs, b0.n, b0.p, &mut out, kernels); }
                        Arm::LocalPlain => { lk.run_staged(b0.runs.records(), b0.n, b0.p, 16, &mut out); }
                        Arm::ZeroRv(mode) => { lk.run_zerorv_staged(b0.runs.records(), b0.n, b0.p, 16, mode, &mut out); }
                    }
                    bar.wait();
                    let t0 = Instant::now();
                    for pass in 0..passes_per_worker {
                        for k in 0..blks.len() {
                            let b = &blks[(k + tid + pass) % blks.len()];
                            match arm {
                                Arm::ShippedA | Arm::ShippedAControl => { sk.invert_runs(&b.runs, b.n, b.p, &mut out, kernels); }
                                Arm::LocalPlain => { lk.run_staged(b.runs.records(), b.n, b.p, 16, &mut out); }
                                Arm::ZeroRv(mode) => { lk.run_zerorv_staged(b.runs.records(), b.n, b.p, 16, mode, &mut out); }
                            }
                        }
                    }
                    (t0.elapsed().as_secs_f64() * 1000.0, out[0])
                }));
            }
            let mut max_dt = 0.0f64;
            for h in handles {
                let (dt, _) = h.join().unwrap();
                if dt > max_dt { max_dt = dt; }
            }
            if r >= 2 { tot_4w[ai].push(max_dt); }
        }
    }
    let base_4w = median(tot_4w[0].clone());
    println!("{:<24} | {:>9} | {:>7}", "Arm", "Wall ms", "vs A");
    println!("{}", "-".repeat(46));
    for (ai, &arm) in arms.iter().enumerate() {
        let m = median(tot_4w[ai].clone());
        println!("{:<24} | {:>9.3} | {:>6.3}x", arm.name(), m, m / base_4w);
    }
}
