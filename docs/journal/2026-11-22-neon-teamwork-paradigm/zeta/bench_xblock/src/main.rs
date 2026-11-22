mod open_xblock;

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{BzReader, Kernels};
use open_xblock::{AaOpenCells, LocalFusedCells, XbOpenKernel, XbSlp16kCells, XbSlp8kCells};
use std::fs;
use std::io::{Cursor, Read};
use std::time::Instant;

fn plain_inverse_bwt(l: &[u8], p: usize) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0usize; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut c = [0usize; 256];
    let mut tot = 0;
    for a in 0..256 {
        c[a] = tot;
        tot += counts[a];
    }
    let mut occ = c;
    let mut phi = vec![0usize; n];
    let mut f = vec![0u8; n];
    for (i, &b) in l.iter().enumerate() {
        let j = occ[b as usize];
        occ[b as usize] += 1;
        phi[j] = i;
        f[j] = b;
    }
    let mut out = Vec::with_capacity(n);
    let mut row = p;
    for _ in 0..n {
        out.push(f[row]);
        row = phi[row];
    }
    out
}

fn encode_records(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let c = l[i];
        let mut w = 0usize;
        while i < l.len() && l[i] == c && w < 127 {
            w += 1;
            i += 1;
        }
        recs.push(((c as u16) << 8) | (w as u16));
    }
    recs
}

fn exhaustive_verify_xbslp() {
    // Stress-test with tiny cache CAP=16 (triggers cache fill, local fallback, and resets across blocks!)
    // AND normal CAP=8192 across ALL binary n<=8 and ternary n<=5, ALL marks p!
    let mut k_aa = XbOpenKernel::<0, 0>::default();
    let mut k_tiny = XbOpenKernel::<2, 16>::default();
    let mut k_8k = XbOpenKernel::<2, 8192>::default();
    let mut out0 = Vec::new();
    let mut out1 = Vec::new();
    let mut out2 = Vec::new();
    let cap_none = open_xblock::Cap::NONE;

    let mut cases = 0usize;
    for &(sigma, max_n) in &[(2usize, 8usize), (3usize, 5usize)] {
        for n in 1..=max_n {
            let total = sigma.pow(n as u32);
            let mut col = vec![0u8; n];
            for mut code in 0..total {
                for j in (0..n).rev() {
                    col[j] = (code % sigma) as u8;
                    code /= sigma;
                }
                let recs = encode_records(&col);
                for p in 0..n {
                    let expected = plain_inverse_bwt(&col, p);
                    let ok0 = k_aa.try_run(&recs, n, p as u32, cap_none, &mut out0);
                    let ok1 = k_tiny.try_run(&recs, n, p as u32, cap_none, &mut out1);
                    let ok2 = k_8k.try_run(&recs, n, p as u32, cap_none, &mut out2);
                    assert!(ok0 && ok1 && ok2);
                    assert_eq!(out0, expected, "AA mismatch on n={} p={}", n, p);
                    assert_eq!(out1, expected, "XbSlp(CAP=16) mismatch on n={} p={}", n, p);
                    assert_eq!(out2, expected, "XbSlp(CAP=8192) mismatch on n={} p={}", n, p);
                    cases += 1;
                }
            }
        }
    }
    println!(
        "[EXHAUSTIVE VERIFY] AA, XbSlp(CAP=16 stress) & XbSlp(CAP=8192) passed {cases} cases (binary n<=8, ternary n<=5, all marks, periodic & multi-cycle) with 0 failures."
    );
}

fn decode_whole<I: InverseBwt + 'static>(data: &[u8], threads: usize) -> (u64, usize, u64) {
    let t0 = Instant::now();
    let mut r = BzReader::parallel_workers_with::<I>(
        Cursor::new(data.to_vec()),
        threads,
        4 * threads,
        Kernels::detect(),
    );
    let mut buf = vec![0u8; 1 << 20];
    let mut total = 0usize;
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    loop {
        let n = r.read(&mut buf).expect("decode");
        if n == 0 {
            break;
        }
        total += n;
        for chunk in buf[..n].chunks_exact(8) {
            let v = u64::from_le_bytes(chunk.try_into().unwrap());
            hash = (hash ^ v).wrapping_mul(0x1000_0000_01b3);
        }
    }
    (t0.elapsed().as_nanos() as u64, total, hash)
}

fn stats_ratio(arm: &[u64], base: &[u64]) -> (f64, f64, f64) {
    let logs: Vec<f64> = arm
        .iter()
        .zip(base.iter())
        .map(|(&a, &b)| (a as f64 / b as f64).ln())
        .collect();
    let n = logs.len() as f64;
    let mean = logs.iter().sum::<f64>() / n;
    let var = logs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let se = (var / n).sqrt();
    let gmean = mean.exp();
    // t_0.975 for df=7 is 2.365
    let lo = (mean - 2.365 * se).exp();
    let hi = (mean + 2.365 * se).exp();
    (gmean, lo, hi)
}

fn median_ms(xs: &[u64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_unstable();
    v[v.len() / 2] as f64 / 1e6
}

fn main() {
    exhaustive_verify_xbslp();

    let bz_path = "/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2";
    let data = fs::read(bz_path).expect("read wikidata.bz2");
    let kernels = Kernels::detect();

    // Pre-extract all 233 large admitted blocks for kernel-only benchmark
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut blocks: Vec<(Runs, usize, u32)> = Vec::new();
    loop {
        match scanner.next(syms).unwrap() {
            Event::Done => break,
            Event::Block { symbols, .. } => {
                if runs::takes_runs(&symbols) {
                    let mut r = Runs::default();
                    let n = runs::front(&symbols, &mut r, kernels).unwrap();
                    if n >= 65_536 {
                        blocks.push((r, n, symbols.orig_ptr));
                    }
                }
                syms = symbols.syms;
            }
        }
    }
    println!("Loaded {} large blocks from wikidata.bz2 for kernel-only A/B.", blocks.len());

    // Verify exact byte-for-byte identity on all 233 blocks in kernel-only mode
    {
        let mut k_ref = OpenCells::default();
        let mut k_aa = AaOpenCells::default();
        let mut k_lf = LocalFusedCells::default();
        let mut k_8k = XbSlp8kCells::default();
        let mut k_16k = XbSlp16kCells::default();
        let mut o_ref = Vec::new();
        let mut o_aa = Vec::new();
        let mut o_lf = Vec::new();
        let mut o_8k = Vec::new();
        let mut o_16k = Vec::new();
        for (idx, (r, n, p)) in blocks.iter().enumerate() {
            assert!(k_ref.invert_runs(r, *n, *p, &mut o_ref, kernels));
            assert!(k_aa.invert_runs(r, *n, *p, &mut o_aa, kernels));
            assert!(k_lf.invert_runs(r, *n, *p, &mut o_lf, kernels));
            assert!(k_8k.invert_runs(r, *n, *p, &mut o_8k, kernels));
            assert!(k_16k.invert_runs(r, *n, *p, &mut o_16k, kernels));
            assert_eq!(o_aa, o_ref, "AA mismatch block {}", idx);
            assert_eq!(o_lf, o_ref, "LocalFused mismatch block {}", idx);
            assert_eq!(o_8k, o_ref, "XbSlp8k mismatch block {}", idx);
            assert_eq!(o_16k, o_ref, "XbSlp16k mismatch block {}", idx);
        }
        println!("[KERNEL VERIFY] All 233 large blocks match production OpenCells byte-for-byte across AA, LocalFused, XbSlp8k, XbSlp16k.");
    }

    // 1. KERNEL-ONLY INTERLEAVED A/B + A/A (8 rounds, rotated arm order)
    let rounds = 8usize;
    let arm_names = ["Ref(0.5.11)", "AA(port)", "LocalFused", "XbSlp8k", "XbSlp16k"];
    let mut k_times = vec![vec![0u64; rounds]; 5];
    let mut out = Vec::new();
    for r in 0..rounds {
        let mut order: Vec<usize> = (0..5).map(|i| (i + r) % 5).collect();
        if r % 2 == 1 {
            order.reverse();
        }
        for &arm in &order {
            let t0 = Instant::now();
            match arm {
                0 => {
                    let mut k = OpenCells::default();
                    for (runs, n, p) in &blocks {
                        k.invert_runs(runs, *n, *p, &mut out, kernels);
                    }
                }
                1 => {
                    let mut k = AaOpenCells::default();
                    for (runs, n, p) in &blocks {
                        k.invert_runs(runs, *n, *p, &mut out, kernels);
                    }
                }
                2 => {
                    let mut k = LocalFusedCells::default();
                    for (runs, n, p) in &blocks {
                        k.invert_runs(runs, *n, *p, &mut out, kernels);
                    }
                }
                3 => {
                    let mut k = XbSlp8kCells::default();
                    for (runs, n, p) in &blocks {
                        k.invert_runs(runs, *n, *p, &mut out, kernels);
                    }
                }
                4 => {
                    let mut k = XbSlp16kCells::default();
                    for (runs, n, p) in &blocks {
                        k.invert_runs(runs, *n, *p, &mut out, kernels);
                    }
                }
                _ => unreachable!(),
            }
            k_times[arm][r] = t0.elapsed().as_nanos() as u64;
        }
    }
    println!("\n=== BENCHMARK 1: KERNEL-ONLY (233 large blocks in stream order, 1T, {} interleaved rounds) ===", rounds);
    for a in 0..5 {
        let (gr, lo, hi) = stats_ratio(&k_times[a], &k_times[0]);
        let (ga, alo, ahi) = stats_ratio(&k_times[a], &k_times[1]);
        println!(
            "  {:12}: median={:6.2} ms | vs Ref={:.4}x [{:.4}, {:.4}] | vs AA={:.4}x [{:.4}, {:.4}]",
            arm_names[a],
            median_ms(&k_times[a]),
            gr, lo, hi,
            ga, alo, ahi
        );
    }

    // 2. WHOLE-DECODER END-TO-END (BzReader::parallel_workers_with, all 431 blocks, 209.7 MB) at 1T and 4W!
    for &threads in &[1usize, 4usize] {
        let mut w_times = vec![vec![0u64; rounds]; 5];
        let mut ref_hash = 0u64;
        for r in 0..rounds {
            let mut order: Vec<usize> = (0..5).map(|i| (i + r) % 5).collect();
            if r % 2 == 1 {
                order.reverse();
            }
            for &arm in &order {
                let (ns, bytes, h) = match arm {
                    0 => decode_whole::<OpenCells>(&data, threads),
                    1 => decode_whole::<AaOpenCells>(&data, threads),
                    2 => decode_whole::<LocalFusedCells>(&data, threads),
                    3 => decode_whole::<XbSlp8kCells>(&data, threads),
                    4 => decode_whole::<XbSlp16kCells>(&data, threads),
                    _ => unreachable!(),
                };
                assert_eq!(bytes, 209_700_196);
                if ref_hash == 0 {
                    ref_hash = h;
                } else {
                    assert_eq!(h, ref_hash, "Output hash mismatch for arm {}!", arm_names[arm]);
                }
                w_times[arm][r] = ns;
            }
        }
        println!(
            "\n=== BENCHMARK 2: WHOLE DECODER (wikidata.bz2, 431 blocks, 209.7 MB, {}W, {} interleaved rounds) ===",
            threads, rounds
        );
        for a in 0..5 {
            let (gr, lo, hi) = stats_ratio(&w_times[a], &w_times[0]);
            let (ga, alo, ahi) = stats_ratio(&w_times[a], &w_times[1]);
            println!(
                "  {:12}: median={:6.2} ms | vs Ref={:.4}x [{:.4}, {:.4}] | vs AA={:.4}x [{:.4}, {:.4}]",
                arm_names[a],
                median_ms(&w_times[a]),
                gr, lo, hi,
                ga, alo, ahi
            );
        }
    }
}
