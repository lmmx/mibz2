#![forbid(unsafe_code)]

mod open_local;

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use open_local::{EmitMode, Kernel as LocalKernel};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

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

/// Encode raw BWT column L into authentic bzip2 MTF+RLE2 Symbols so `mibz2::runs::front_scalar`
/// produces the exact `Runs` struct that production `OpenCells::invert_runs` consumes.
fn encode_symbols(l: &[u8], p: u32) -> Symbols {
    let mut present = [false; 256];
    for &b in l {
        present[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 {
        if present[b] {
            seq_to_unseq[n_in_use] = b as u8;
            n_in_use += 1;
        }
    }
    let mut mtf = [0u8; 256];
    mtf[..n_in_use].copy_from_slice(&seq_to_unseq[..n_in_use]);

    let mut syms: Vec<u16> = Vec::with_capacity(l.len() / 2);
    let mut zero_run = 0usize;
    let flush_zeros = |zr: &mut usize, syms: &mut Vec<u16>| {
        let mut r = *zr;
        while r > 0 {
            if r & 1 == 1 {
                syms.push(0); // RUNA
                r = (r - 1) >> 1;
            } else {
                syms.push(1); // RUNB
                r = (r - 2) >> 1;
            }
        }
        *zr = 0;
    };

    for &b in l {
        let pos = mtf[..n_in_use].iter().position(|&x| x == b).unwrap();
        if pos == 0 {
            zero_run += 1;
        } else {
            flush_zeros(&mut zero_run, &mut syms);
            syms.push((pos + 1) as u16);
            // Move to front
            for k in (1..=pos).rev() {
                mtf[k] = mtf[k - 1];
            }
            mtf[0] = b;
        }
    }
    flush_zeros(&mut zero_run, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);

    Symbols {
        crc: 0,
        orig_ptr: p,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    }
}

struct LoadedBlock {
    name: &'static str,
    n: usize,
    p: u32,
    runs: Runs,
    expected: Vec<u8>,
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

#[derive(Clone, Copy)]
enum Arm {
    ShippedA,
    ShippedAControl,
    LocalPlain(u32),
    LazyTunnel { min_steps: u32, sub: bool },
}

impl Arm {
    fn name(self) -> String {
        match self {
            Arm::ShippedA => "OpenCells(shipped-A)".to_string(),
            Arm::ShippedAControl => "OpenCells(shipped-A')".to_string(),
            Arm::LocalPlain(l) => format!("OpenLocal(L={})", l),
            Arm::LazyTunnel { min_steps, sub } => {
                format!("LazyTunnel(ms={},sub={})", min_steps, if sub { "T" } else { "F" })
            }
        }
    }
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let kernels = Kernels::detect();
    println!("=== PREPARING 6 AUTHENTICATED BLOCKS ===");
    let mut blocks = Vec::new();
    let mut shipped = OpenCells::default();

    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).expect("read block");
        assert_eq!(l.len(), b.n);
        let syms = encode_symbols(&l, b.p);
        let mut runs = Runs::default();
        let n_dec = front_scalar(&syms, &mut runs).expect("front_scalar");
        assert_eq!(n_dec, b.n);

        // Verify records reconstruct L identically!
        let mut recon = Vec::with_capacity(b.n);
        for &rec in runs.records() {
            let c = (rec >> 8) as u8;
            let len = (rec & 255) as usize;
            recon.resize(recon.len() + len, c);
        }
        assert_eq!(recon, l, "Runs reconstruction mismatch on {}", b.file);

        let mut expected = Vec::new();
        assert!(shipped.invert_runs(&runs, b.n, b.p, &mut expected, kernels));
        assert_eq!(expected.len(), b.n);
        println!("  Loaded {} (n={}, p={}, records={})", b.file, b.n, b.p, runs.records().len());
        blocks.push(LoadedBlock {
            name: b.file,
            n: b.n,
            p: b.p,
            runs,
            expected,
        });
    }

    let arms = [
        Arm::ShippedA,
        Arm::ShippedAControl,
        Arm::LocalPlain(16),
        Arm::LazyTunnel { min_steps: 3, sub: true },
        Arm::LazyTunnel { min_steps: 4, sub: true },
        Arm::LazyTunnel { min_steps: 4, sub: false },
        Arm::LazyTunnel { min_steps: 6, sub: true },
        Arm::LazyTunnel { min_steps: 8, sub: true },
        Arm::LocalPlain(8),
        Arm::LocalPlain(4),
        Arm::LocalPlain(1),
    ];

    // 1. Exact correctness check across all arms and all 6 blocks!
    println!("\n=== CORRECTNESS CHECK (ALL ARMS x 6 BLOCKS) ===");
    for &arm in &arms {
        let mut s_k = OpenCells::default();
        let mut l_k = LocalKernel::default();
        let mut out = Vec::new();
        for b in &blocks {
            match arm {
                Arm::ShippedA | Arm::ShippedAControl => {
                    assert!(s_k.invert_runs(&b.runs, b.n, b.p, &mut out, kernels));
                }
                Arm::LocalPlain(lim) => {
                    l_k.run_timed(b.runs.records(), b.n, b.p, lim, EmitMode::Plain, &mut out);
                }
                Arm::LazyTunnel { min_steps, sub } => {
                    l_k.run_timed(
                        b.runs.records(),
                        b.n,
                        b.p,
                        16,
                        EmitMode::LazyTunnel {
                            min_steps,
                            sub_tunnels: sub,
                        },
                        &mut out,
                    );
                }
            }
            assert_eq!(out, b.expected, "Output mismatch for {} on {}", arm.name(), b.name);
        }
        println!("  [PASS] {}", arm.name());
    }

    // 2. Single-thread (1T) interleaved benchmark!
    let rounds_1t = 15;
    println!("\n=== 1T BENCHMARK ({} interleaved rounds, 6 blocks summed, median ms) ===", rounds_1t);
    let mut totals_1t: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut constr_1t: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut emit_1t: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
    let mut per_blk_1t: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); blocks.len()]; arms.len()];

    // Pre-allocate one reusable scratch per arm
    let mut shipped_a = OpenCells::default();
    let mut shipped_ac = OpenCells::default();
    let mut locals: Vec<LocalKernel> = (0..arms.len()).map(|_| LocalKernel::default()).collect();
    let mut out = Vec::with_capacity(900_016);

    // Warmup 2 rounds
    for r in 0..(rounds_1t + 2) {
        for (ai, &arm) in arms.iter().enumerate() {
            let mut sum_tot = 0.0;
            let mut sum_c = 0.0;
            let mut sum_e = 0.0;
            for (bi, b) in blocks.iter().enumerate() {
                let (tot, c_t, e_t) = match arm {
                    Arm::ShippedA => {
                        let t0 = Instant::now();
                        shipped_a.invert_runs(&b.runs, b.n, b.p, &mut out, kernels);
                        let dt = t0.elapsed().as_secs_f64();
                        (dt, 0.0, 0.0)
                    }
                    Arm::ShippedAControl => {
                        let t0 = Instant::now();
                        shipped_ac.invert_runs(&b.runs, b.n, b.p, &mut out, kernels);
                        let dt = t0.elapsed().as_secs_f64();
                        (dt, 0.0, 0.0)
                    }
                    Arm::LocalPlain(lim) => {
                        let (_, c_t, e_t) = locals[ai].run_timed(
                            b.runs.records(),
                            b.n,
                            b.p,
                            lim,
                            EmitMode::Plain,
                            &mut out,
                        );
                        (c_t + e_t, c_t, e_t)
                    }
                    Arm::LazyTunnel { min_steps, sub } => {
                        let (_, c_t, e_t) = locals[ai].run_timed(
                            b.runs.records(),
                            b.n,
                            b.p,
                            16,
                            EmitMode::LazyTunnel {
                                min_steps,
                                sub_tunnels: sub,
                            },
                            &mut out,
                        );
                        (c_t + e_t, c_t, e_t)
                    }
                };
                sum_tot += tot;
                sum_c += c_t;
                sum_e += e_t;
                if r >= 2 {
                    per_blk_1t[ai][bi].push(tot * 1000.0);
                }
            }
            if r >= 2 {
                totals_1t[ai].push(sum_tot * 1000.0);
                constr_1t[ai].push(sum_c * 1000.0);
                emit_1t[ai].push(sum_e * 1000.0);
            }
        }
    }

    let base_1t = median(totals_1t[0].clone());
    let local16_emit_1t = median(emit_1t[2].clone());
    println!("{:<28} | {:>8} | {:>7} | {:>8} | {:>8} | {:>8}",
        "Arm", "Total ms", "vs A", "Const ms", "Emit ms", "Emit/L16");
    println!("{}", "-".repeat(82));
    for (ai, &arm) in arms.iter().enumerate() {
        let tot = median(totals_1t[ai].clone());
        let c = median(constr_1t[ai].clone());
        let e = median(emit_1t[ai].clone());
        if c == 0.0 {
            println!("{:<28} | {:>8.3} | {:>6.3}x | {:>8} | {:>8} | {:>8}",
                arm.name(), tot, tot / base_1t, "-", "-", "-");
        } else {
            println!("{:<28} | {:>8.3} | {:>6.3}x | {:>8.3} | {:>8.3} | {:>7.3}x",
                arm.name(), tot, tot / base_1t, c, e, e / local16_emit_1t);
        }
    }

    println!("\nPer-block 1T median ms (OpenCells-A vs OpenLocal-16 vs LazyTunnel ms=4,sub=T vs ms=6,sub=T):");
    println!("{:<12} | {:>8} | {:>8} | {:>8} ({:>6}) | {:>8} ({:>6})",
        "Block", "ShippedA", "Local16", "LT(4,T)", "vs A", "LT(6,T)", "vs A");
    for (bi, b) in blocks.iter().enumerate() {
        let a = median(per_blk_1t[0][bi].clone());
        let l16 = median(per_blk_1t[2][bi].clone());
        let lt4 = median(per_blk_1t[4][bi].clone());
        let lt6 = median(per_blk_1t[6][bi].clone());
        println!("{:<12} | {:>8.3} | {:>8.3} | {:>8.3} ({:>5.3}x) | {:>8.3} ({:>5.3}x)",
            b.name, a, l16, lt4, lt4 / a, lt6, lt6 / a);
    }

    // 3. 4-Worker (4W) concurrent benchmark!
    let blocks_arc = Arc::new(blocks);
    let rounds_4w = 15;
    let passes_per_worker = 3; // each of 4 workers decodes all 6 blocks 3 times = 18 blocks/worker, 72 blocks/round
    println!("\n=== 4W CONCURRENT BENCHMARK ({} interleaved rounds, 4 threads x {} passes x 6 blocks = 72 blocks/round, median wall ms) ===",
        rounds_4w, passes_per_worker);

    let arms_4w = [
        Arm::ShippedA,
        Arm::ShippedAControl,
        Arm::LocalPlain(16),
        Arm::LazyTunnel { min_steps: 3, sub: true },
        Arm::LazyTunnel { min_steps: 4, sub: true },
        Arm::LazyTunnel { min_steps: 4, sub: false },
        Arm::LazyTunnel { min_steps: 6, sub: true },
        Arm::LazyTunnel { min_steps: 8, sub: true },
    ];
    let mut totals_4w: Vec<Vec<f64>> = vec![Vec::new(); arms_4w.len()];

    for r in 0..(rounds_4w + 2) {
        for (ai, &arm) in arms_4w.iter().enumerate() {
            let barrier = Arc::new(Barrier::new(4));
            let mut handles = Vec::with_capacity(4);
            for tid in 0..4usize {
                let blks = Arc::clone(&blocks_arc);
                let bar = Arc::clone(&barrier);
                handles.push(thread::spawn(move || {
                    let mut sk = OpenCells::default();
                    let mut lk = LocalKernel::default();
                    let mut out = Vec::with_capacity(900_016);
                    // One warmup block so thread scratch is allocated before timed barrier
                    let b0 = &blks[tid % blks.len()];
                    match arm {
                        Arm::ShippedA | Arm::ShippedAControl => {
                            sk.invert_runs(&b0.runs, b0.n, b0.p, &mut out, kernels);
                        }
                        Arm::LocalPlain(lim) => {
                            lk.run_timed(b0.runs.records(), b0.n, b0.p, lim, EmitMode::Plain, &mut out);
                        }
                        Arm::LazyTunnel { min_steps, sub } => {
                            lk.run_timed(b0.runs.records(), b0.n, b0.p, 16, EmitMode::LazyTunnel { min_steps, sub_tunnels: sub }, &mut out);
                        }
                    }
                    bar.wait();
                    let t0 = Instant::now();
                    for pass in 0..passes_per_worker {
                        for k in 0..blks.len() {
                            let b = &blks[(k + tid + pass) % blks.len()];
                            match arm {
                                Arm::ShippedA | Arm::ShippedAControl => {
                                    sk.invert_runs(&b.runs, b.n, b.p, &mut out, kernels);
                                }
                                Arm::LocalPlain(lim) => {
                                    lk.run_timed(b.runs.records(), b.n, b.p, lim, EmitMode::Plain, &mut out);
                                }
                                Arm::LazyTunnel { min_steps, sub } => {
                                    lk.run_timed(b.runs.records(), b.n, b.p, 16, EmitMode::LazyTunnel { min_steps, sub_tunnels: sub }, &mut out);
                                }
                            }
                        }
                    }
                    let dt = t0.elapsed().as_secs_f64();
                    (dt, out[0])
                }));
            }
            let mut max_dt = 0.0f64;
            for h in handles {
                let (dt, _) = h.join().unwrap();
                if dt > max_dt {
                    max_dt = dt;
                }
            }
            if r >= 2 {
                totals_4w[ai].push(max_dt * 1000.0);
            }
        }
    }

    let base_4w = median(totals_4w[0].clone());
    println!("{:<28} | {:>10} | {:>8}", "Arm", "Wall ms", "vs A");
    println!("{}", "-".repeat(52));
    for (ai, &arm) in arms_4w.iter().enumerate() {
        let med = median(totals_4w[ai].clone());
        println!("{:<28} | {:>10.3} | {:>7.3}x", arm.name(), med, med / base_4w);
    }
}
