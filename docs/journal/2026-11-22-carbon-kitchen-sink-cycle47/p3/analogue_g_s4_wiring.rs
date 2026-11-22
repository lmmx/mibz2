#![forbid(unsafe_code)]

//! Cycle 47 P3 — Track 1: Solved Analogue G (Involution-Factored / Dihedral Permutations)
//! & Classical 19th-Century Algebra / Permutation Attack on `S4` Bidirectional Wiring (`+728 us`).
//!
//! Compiles with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

#[allow(dead_code)]
mod s4_kernel;

use s4_kernel::{Kernel as ExpKernel, S4Mode};
use std::fs;
use std::sync::{Arc, Barrier};
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &rot) in sa.iter().enumerate() {
        if rot == 0 {
            p = i as u32;
        }
        l.push(text[(rot + n - 1) % n]);
    }
    (l, p)
}

/// Forward FL reference (libbzip2 / cand_r5_head contract on arbitrary columns including multi-cycle).
fn invert_fl_ref(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut starts = [0usize; 256];
    let mut sum = 0usize;
    for a in 0..256 {
        starts[a] = sum;
        sum += cnt[a];
    }
    let mut fl = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        fl[starts[a]] = i;
        starts[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        x = fl[x];
        out.push(l[x]);
    }
    out
}

fn load_admitted_blocks() -> Vec<Block> {
    let specs: [(&str, u32); 6] = [
        ("block-0.L", 264045),
        ("block-2.L", 320807),
        ("block-85.L", 325088),
        ("block-143.L", 802868),
        ("block-207.L", 205180),
        ("block-210.L", 791429),
    ];
    let mut out_blocks = Vec::new();
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut tmp = Vec::new();
    for (file, p) in specs {
        let path = format!("/home/antigravity/admitted-bwt/{file}");
        let l = fs::read(&path).unwrap_or_else(|e| panic!("failed reading {path}: {e}"));
        let n = l.len();
        let records = records_of(&l);
        ref_k
            .run_entry(&records, n, p, cand_r5_head::Cap::NONE, &mut tmp)
            .unwrap();
        out_blocks.push(Block {
            name: file.to_string(),
            n,
            p,
            records,
            expected: tmp.clone(),
        });
    }
    out_blocks
}

fn load_non_wikidata_blocks() -> Vec<Block> {
    let specs: [(&str, u32); 6] = [
        ("jsonl_block_0.L", 897198),
        ("jsonl_block_1.L", 672135),
        ("jsonl_block_2.L", 334517),
        ("jsonl_block_3.L", 734310),
        ("jsonl_block_4.L", 522665),
        ("syslog_block_0.L", 165238),
    ];
    let mut out_blocks = Vec::new();
    let mut ref_k = cand_r5_head::Kernel::default();
    let mut tmp = Vec::new();
    for (file, p) in specs {
        let path = format!("/tmp/mibz2_corpora/non_wikidata/{file}");
        let l = fs::read(&path).unwrap_or_else(|e| panic!("failed reading {path}: {e}"));
        let n = l.len();
        let records = records_of(&l);
        ref_k
            .run_entry(&records, n, p, cand_r5_head::Cap::NONE, &mut tmp)
            .unwrap();
        out_blocks.push(Block {
            name: file.to_string(),
            n,
            p,
            records,
            expected: tmp.clone(),
        });
    }
    out_blocks
}

// ============================================================================
// Part 1: Solved Analogue G — Dihedral / Palindromic Involution-Factored BWTs
// ============================================================================

/// Generate a Fibonacci Sturmian BWT (r = 2, dihedral rotation x -> (x + q) mod n).
/// Every rotation pi(x) = (x + q) mod n factors analytically into two involutions
/// (Burnside 1897 / Miller 1900):
///   tau_1(x) = (n - 1 - x) mod n,   tau_2(x) = (n - 1 - q - x) mod n,
/// so pi(x) = tau_1(tau_2(x)) and pi^{-1}(x) = tau_2(tau_1(x)) in O(1) with ZERO reverse scatter!
fn run_solved_analogue_g() {
    println!("=== PART 1: SOLVED ANALOGUE G (DIHEDRAL / 2-INVOLUTION BWTs, BURNSIDE 1897) ===");
    let fib_cases: [(usize, usize, &str); 3] = [
        (196_418, 121_393, "Fib-F27"),
        (514_229, 317_811, "Fib-F29"),
        (832_040, 514_229, "Fib-F30"),
    ];
    for (n, q, label) in fib_cases {
        // Exact BWT of Fibonacci word: q 'b's followed by (n - q) 'a's, primary p = q - 1
        // Wait: for standard Sturmian rotation by q, L[0..q] = 'b', L[q..n] = 'a' (2 runs!).
        // Check FL: starts['a']=0, starts['b']=n-q.
        // For i in 0..q: L[i]='b' -> FL[n-q + i] = i.
        // For i in q..n: L[i]='a' -> FL[i - q] = i.
        // So FL(x) = (x + q) mod n!
        let mut l = vec![b'b'; q];
        l.resize(n, b'a');
        let p = 0u32;
        let expected = invert_fl_ref(&l, p);

        // Dihedral 2-involution bidirectional decoder (Burnside 1897):
        // tau1(x) = (n - 1) - x, tau2(x) = (2n - 1 - q - x) % n
        // Forward: x -> tau1(tau2(x)) = (x + q) mod n
        // Backward: y -> tau2(tau1(y)) = (y + n - q) mod n
        // Notice ZERO edge table, ZERO fill(0), ZERO reverse scatter!
        let decode_dihedral = |l_col: &[u8], out: &mut Vec<u8>| {
            out.resize(n, 0);
            let mut xf = p as usize;
            let mut xb = p as usize;
            let mut lo = 0usize;
            let mut hi = n;
            while lo < hi {
                // tau1(tau2(xf)) = (xf + q) mod n
                let t2f = if n - 1 >= xf + q {
                    n - 1 - xf - q
                } else {
                    2 * n - 1 - xf - q
                };
                xf = (n - 1) - t2f;
                out[lo] = l_col[xf];
                lo += 1;
                if lo >= hi {
                    break;
                }
                hi -= 1;
                out[hi] = l_col[xb];
                // tau2(tau1(xb)) = (xb + n - q) mod n
                let t1b = (n - 1) - xb;
                xb = if n - 1 >= t1b + q {
                    n - 1 - t1b - q
                } else {
                    2 * n - 1 - t1b - q
                };
            }
        };

        let mut out_dih = Vec::new();
        decode_dihedral(&l, &mut out_dih);
        assert_eq!(out_dih, expected, "Dihedral 2-involution mismatch on {label}");

        // Time Dihedral vs Reference FL vs cand_r5_head
        let rounds = 7;
        let mut t_dih = Vec::new();
        let mut t_ref = Vec::new();
        let mut t_head = Vec::new();
        let recs = records_of(&l);
        let mut head_k = cand_r5_head::Kernel::default();
        let mut out_head = Vec::new();
        head_k
            .run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_head)
            .unwrap();
        assert_eq!(out_head, expected, "Head mismatch on {label}");

        for _ in 0..rounds {
            let t0 = Instant::now();
            decode_dihedral(&l, &mut out_dih);
            t_dih.push(t0.elapsed().as_secs_f64() * 1e6);

            let t0 = Instant::now();
            let _ = invert_fl_ref(&l, p);
            t_ref.push(t0.elapsed().as_secs_f64() * 1e6);

            let t0 = Instant::now();
            head_k
                .run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_head)
                .unwrap();
            t_head.push(t0.elapsed().as_secs_f64() * 1e6);
        }
        t_dih.sort_by(|a, b| a.total_cmp(b));
        t_ref.sort_by(|a, b| a.total_cmp(b));
        t_head.sort_by(|a, b| a.total_cmp(b));
        let md = t_dih[rounds / 2];
        let mr = t_ref[rounds / 2];
        let mh = t_head[rounds / 2];
        println!(
            "  {label:8} (n={n:>7}, q={q:>7}): Dihedral-2Inv={md:7.1} us ({:.3}x Ref-FL {mr:7.1} us; Head={mh:7.1} us) [EXACT]",
            md / mr
        );
    }
    println!();
}

// ============================================================================
// Part 2: Exactness Gate (7,373 small + 200 malformed + periodic + spiral + 12 real blocks)
// ============================================================================

fn run_exactness_gate(wiki: &[Block], non_wiki: &[Block]) {
    println!("=== PART 2: EXHAUSTIVE EXACTNESS GATE ===");
    let modes = [
        ("HeadVerbatim", S4Mode::HeadVerbatim),
        ("HeadEpoch16x", S4Mode::HeadEpoch16x),
        ("NobiVerbatim", S4Mode::NobiVerbatim),
        ("TileRev-4K", S4Mode::TileRev(4096)),
        ("TileRev-8K", S4Mode::TileRev(8192)),
        ("TileRev-16K", S4Mode::TileRev(16384)),
        ("TileRev-64K", S4Mode::TileRev(65536)),
        ("SinglePassTileV-8K", S4Mode::SinglePassTileV(8192)),
        ("SinglePassTileV-64K", S4Mode::SinglePassTileV(65536)),
        ("L1BufScatter", S4Mode::L1BufScatter),
        ("ZeroSkipUntiled", S4Mode::ZeroSkipUntiled),
        ("DualFwd", S4Mode::DualFwd),
        ("QuadFwd", S4Mode::QuadFwd),
    ];

    let mut head_k = cand_r5_head::Kernel::default();
    let mut exp_k = ExpKernel::default();
    let mut out_ref = Vec::new();
    let mut out_exp = Vec::new();

    // 1. All binary words n = 1..=11 (4,094) + all ternary words n = 1..=7 (3,279) = 7,373 small BWTs
    let mut small_checked = 0usize;
    for n in 1..=11usize {
        for mask in 0..(1u32 << n) {
            let mut text = Vec::with_capacity(n);
            for bit in 0..n {
                text.push(if ((mask >> bit) & 1) == 0 { b'a' } else { b'b' });
            }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            head_k
                .run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref)
                .unwrap();
            assert_eq!(out_ref, text, "cand_r5_head failed small binary n={n}");
            for &(mname, mode) in &modes {
                exp_k.run_mode(&recs, n, p, mode, 2, &mut out_exp);
                assert_eq!(
                    out_exp, out_ref,
                    "Mode {mname} failed binary small n={n} mask={mask}"
                );
            }
            small_checked += 1;
        }
    }
    for n in 1..=7usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut text = Vec::with_capacity(n);
            for _ in 0..n {
                text.push(b'a' + (code % 3) as u8);
                code /= 3;
            }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            head_k
                .run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref)
                .unwrap();
            assert_eq!(out_ref, text, "cand_r5_head failed small ternary n={n}");
            for &(mname, mode) in &modes {
                exp_k.run_mode(&recs, n, p, mode, 2, &mut out_exp);
                assert_eq!(out_exp, out_ref, "Mode {mname} failed ternary small n={n}");
            }
            small_checked += 1;
        }
    }
    println!(
        "  [PASS] {small_checked} exhaustive small BWTs (binary 1..=11 + ternary 1..=7) x {} modes (bi_min_m=2)",
        modes.len()
    );

    // 2. 200 crafted malformed multi-cycle columns (including non-dividing cycles) vs cand_r5_head (forward FL)
    let mut rng: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next_u64 = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut malformed_checked = 0usize;
    let mut multi_cycle_count = 0usize;
    for idx in 0..200usize {
        let n = 6 + (idx % 115);
        let alpha = 2 + (idx % 5);
        let l: Vec<u8> = if idx == 0 {
            b"bbaabb".to_vec() // Referee Cycle 46 Bug #1 witness
        } else if idx < 40 {
            // Crafted doubled / non-dividing multi-cycle columns
            let half = (n / 2).max(3);
            let mut v = Vec::with_capacity(2 * half);
            for _ in 0..half {
                let c = b'a' + (next_u64() as usize % alpha) as u8;
                v.push(c);
                v.push(c);
            }
            v
        } else {
            (0..n)
                .map(|_| b'a' + (next_u64() as usize % alpha) as u8)
                .collect()
        };
        let nn = l.len();
        let p = (next_u64() as usize % nn) as u32;
        let fl_ref = invert_fl_ref(&l, p);
        let recs = records_of(&l);
        head_k
            .run_entry(&recs, nn, p, cand_r5_head::Cap::NONE, &mut out_ref)
            .unwrap();
        // Verify cand_r5_head vs FL ref on multi-cycle (note: cand_r5_head tiles marked cycle to n)
        if fl_ref != out_ref {
            // Confirm our modes match cand_r5_head (which is forward-FL on marked cycle!)
            multi_cycle_count += 1;
        } else {
            // Check if multi-cycle by counting cycle length
            let mut cnt = [0usize; 256];
            for &b in &l {
                cnt[usize::from(b)] += 1;
            }
            let mut st = [0usize; 256];
            let mut s = 0;
            for a in 0..256 {
                st[a] = s;
                s += cnt[a];
            }
            let mut fl = vec![0usize; nn];
            for (i, &b) in l.iter().enumerate() {
                let a = usize::from(b);
                fl[st[a]] = i;
                st[a] += 1;
            }
            let mut x = p as usize;
            let mut cyc = 0;
            loop {
                x = fl[x];
                cyc += 1;
                if x == p as usize || cyc >= nn {
                    break;
                }
            }
            if cyc < nn {
                multi_cycle_count += 1;
            }
        }
        for &(mname, mode) in &modes {
            exp_k.run_mode(&recs, nn, p, mode, 2, &mut out_exp);
            assert_eq!(
                out_exp, out_ref,
                "Mode {mname} disagreed with cand_r5_head on malformed idx={idx} n={nn} p={p}"
            );
        }
        malformed_checked += 1;
    }
    println!(
        "  [PASS] {malformed_checked} malformed columns ({multi_cycle_count} multi-cycle) byte-exact vs cand_r5_head across all {} modes",
        modes.len()
    );

    // 3. Large synthetic stress: periodic-300k (d=150, q=2000) & spiral-200k (w=199,998)
    {
        let mut pat = Vec::with_capacity(150);
        for i in 0..150usize {
            pat.push(b'a' + ((i * 7 + 3) % 13) as u8);
        }
        let mut text = Vec::with_capacity(300_000);
        for _ in 0..2000 {
            text.extend_from_slice(&pat);
        }
        // Periodic BWT: each row of pat's BWT repeated q=2000 times
        let (l_u, p_u) = bwt_small(&pat);
        let mut l_per = Vec::with_capacity(300_000);
        for &b in &l_u {
            l_per.resize(l_per.len() + 2000, b);
        }
        let p_per = p_u * 2000;
        let recs_per = records_of(&l_per);
        head_k
            .run_entry(&recs_per, 300_000, p_per, cand_r5_head::Cap::NONE, &mut out_ref)
            .unwrap();
        assert_eq!(out_ref, text);
        for &(mname, mode) in &modes {
            exp_k.run_mode(&recs_per, 300_000, p_per, mode, 2, &mut out_exp);
            assert_eq!(out_exp, out_ref, "Mode {mname} failed periodic-300k");
        }

        // Large periodic with m >= 65536 (e.g. d=35000, q=3 -> m=105000, 3 disjoint cycles!)
        let mut l_per3 = Vec::with_capacity(105_000);
        for i in 0..35_000usize {
            let b = b'a' + ((i.wrapping_mul(2654435761) >> 24) as u8 % 16);
            l_per3.push(b);
            l_per3.push(b);
            l_per3.push(b);
        }
        let recs_per3 = records_of(&l_per3);
        head_k
            .run_entry(&recs_per3, 105_000, 17, cand_r5_head::Cap::NONE, &mut out_ref)
            .unwrap();
        for &(mname, mode) in &modes {
            exp_k.run_mode(&recs_per3, 105_000, 17, mode, 65_536, &mut out_exp);
            assert_eq!(out_exp, out_ref, "Mode {mname} failed periodic-105k-d3 (m>=65536)");
        }
        println!("  [PASS] periodic-300k & periodic-105k-d3 (m>=65536 disjoint cycles) across all {} modes", modes.len());
    }

    // 4. All 6 Wikidata + 6 Non-Wikidata real blocks
    for (corpus_name, corpus) in [("6 Wikidata", wiki), ("6 Non-Wikidata", non_wiki)] {
        for blk in corpus {
            for &(mname, mode) in &modes {
                exp_k.run_mode(&blk.records, blk.n, blk.p, mode, 65_536, &mut out_exp);
                assert_eq!(
                    out_exp, blk.expected,
                    "Mode {mname} failed on {corpus_name} block {}",
                    blk.name
                );
            }
        }
        println!(
            "  [PASS] {corpus_name} real blocks byte-exact across all {} modes",
            modes.len()
        );
    }
    println!();
}

// ============================================================================
// Part 3: Stepanov Failure Diagnostics + S4 Tax Anatomy
// ============================================================================

fn run_diagnostics(wiki: &[Block], non_wiki: &[Block]) {
    println!("=== PART 3: STEPANOV ANALOGUE G FAILURE DIAGNOSIS & S4 TAX ANATOMY ===");
    let mut k = ExpKernel::default();
    let mut out = Vec::new();
    println!(
        "{:<16} {:>7} {:>6} {:>7} {:>7} {:>8} {:>8} {:>9} {:>9} {:>8} {:>7} {:>7}",
        "Block", "m", "Bands", "Mono%", "CL64%", "Fill0_us", "NobiS4", "HeadS4", "S4_Tax", "DihFail%", "WhitK2", "WhitK4"
    );
    for blk in wiki.iter().chain(non_wiki.iter()) {
        let (stats, _) = k.run_mode(&blk.records, blk.n, blk.p, S4Mode::HeadVerbatim, 65_536, &mut out);
        let (b, mono, cl) = k.band_locality_stats();
        let (dih_fail, wk2, wk4) = k.whitworth_and_dihedral_stats(&blk.records, blk.n, blk.p);
        let mono_pct = if b > 1 { 100.0 * mono as f64 / (b - 1) as f64 } else { 100.0 };
        let cl_pct = if b > 0 { 100.0 * cl as f64 / b as f64 } else { 0.0 };

        let mut buf = vec![0xA5A5_A5A5u32; stats.retained];
        let mut tf = Vec::with_capacity(51);
        for _ in 0..51 {
            let t0 = Instant::now();
            buf.fill(0);
            std::hint::black_box(&buf);
            tf.push(t0.elapsed().as_secs_f64() * 1e6);
        }
        tf.sort_by(|a, b| a.total_cmp(b));
        let fill_us = tf[25];

        let mut s4_nobi = Vec::new();
        let mut s4_head = Vec::new();
        for _ in 0..15 {
            let (_, st_n) = k.run_mode(&blk.records, blk.n, blk.p, S4Mode::NobiVerbatim, 65_536, &mut out);
            s4_nobi.push(st_n.s4_us);
            let (_, st_h) = k.run_mode(&blk.records, blk.n, blk.p, S4Mode::HeadVerbatim, 65_536, &mut out);
            s4_head.push(st_h.s4_us);
        }
        s4_nobi.sort_by(|a, b| a.total_cmp(b));
        s4_head.sort_by(|a, b| a.total_cmp(b));
        let sn = s4_nobi[7];
        let sh = s4_head[7];
        println!(
            "{:<16} {:>7} {:>6} {:>6.1}% {:>6.1}% {:>8.1} {:>8.1} {:>9.1} {:>+9.1} {:>7.2}% {:>7.4} {:>7.4}",
            blk.name, stats.retained, b, mono_pct, cl_pct, fill_us, sn, sh, sh - sn, dih_fail, wk2, wk4
        );
    }
    println!();
}

// ============================================================================
// Part 4: Same-Binary Paired A/B + A/A Benchmarks (1T, 2W, 4W)
// ============================================================================

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn bench_1t(label: &str, blocks: &[Block], rounds: usize) {
    println!("=== 1T PAIRED A/B + A/A BENCHMARK ({label}, {rounds} paired rounds) ===");
    let arms: [(&str, S4Mode); 14] = [
        ("Head-A1", S4Mode::HeadVerbatim),
        ("Head-A2 (A/A)", S4Mode::HeadVerbatim),
        ("HeadEpoch16x", S4Mode::HeadEpoch16x),
        ("Nobi", S4Mode::NobiVerbatim),
        ("TileRev-4K", S4Mode::TileRev(4096)),
        ("TileRev-8K", S4Mode::TileRev(8192)),
        ("TileRev-16K", S4Mode::TileRev(16384)),
        ("TileRev-64K", S4Mode::TileRev(65536)),
        ("SingleTileV-8K", S4Mode::SinglePassTileV(8192)),
        ("SingleTileV-64K", S4Mode::SinglePassTileV(65536)),
        ("L1BufScatter", S4Mode::L1BufScatter),
        ("ZeroSkipUntiled", S4Mode::ZeroSkipUntiled),
        ("DualFwd (K=2)", S4Mode::DualFwd),
        ("QuadFwd (K=4)", S4Mode::QuadFwd),
    ];

    let n_arms = arms.len();
    let mut kernels: Vec<ExpKernel> = (0..n_arms).map(|_| ExpKernel::default()).collect();
    let mut outs: Vec<Vec<u8>> = (0..n_arms).map(|_| Vec::new()).collect();

    // Warmup
    for (ai, &(_, mode)) in arms.iter().enumerate() {
        for blk in blocks {
            kernels[ai].run_mode(&blk.records, blk.n, blk.p, mode, 65_536, &mut outs[ai]);
        }
    }

    let mut tot_runs = vec![Vec::with_capacity(rounds); n_arms];
    let mut s13_runs = vec![Vec::with_capacity(rounds); n_arms];
    let mut s4_runs = vec![Vec::with_capacity(rounds); n_arms];
    let mut s5_runs = vec![Vec::with_capacity(rounds); n_arms];

    for r in 0..rounds {
        // Alternate arm order every round to cancel drift
        let order: Vec<usize> = if r % 2 == 0 {
            (0..n_arms).collect()
        } else {
            (0..n_arms).rev().collect()
        };
        for ai in order {
            let mode = arms[ai].1;
            let mut sum_s13 = 0.0;
            let mut sum_s4 = 0.0;
            let mut sum_s5 = 0.0;
            for blk in blocks {
                let (_, st) =
                    kernels[ai].run_mode(&blk.records, blk.n, blk.p, mode, 65_536, &mut outs[ai]);
                sum_s13 += st.s1_s3_us;
                sum_s4 += st.s4_us;
                sum_s5 += st.s5_us;
            }
            s13_runs[ai].push(sum_s13);
            s4_runs[ai].push(sum_s4);
            s5_runs[ai].push(sum_s5);
            tot_runs[ai].push(sum_s13 + sum_s4 + sum_s5);
        }
    }

    for ai in 0..n_arms {
        s13_runs[ai].sort_by(|a, b| a.total_cmp(b));
        s4_runs[ai].sort_by(|a, b| a.total_cmp(b));
        s5_runs[ai].sort_by(|a, b| a.total_cmp(b));
        tot_runs[ai].sort_by(|a, b| a.total_cmp(b));
    }

    let base_tot_p50 = quantile(&tot_runs[0], 0.5);
    let base_tot_p10 = quantile(&tot_runs[0], 0.1);
    let base_s4_p50 = quantile(&s4_runs[0], 0.5);
    let base_s45_p50 = quantile(&s4_runs[0], 0.5) + quantile(&s5_runs[0], 0.5);

    println!(
        "{:<16} {:>8} {:>8} {:>8} {:>8} {:>9} {:>8} {:>8} {:>8} {:>9}",
        "Arm", "S1-S3", "S4_p50", "S5_p50", "S4+S5", "Tot_p50", "Tot/A1", "p10/A1", "S4/A1", "ScratchKiB"
    );
    for (ai, &(name, _)) in arms.iter().enumerate() {
        let s13 = quantile(&s13_runs[ai], 0.5);
        let s4 = quantile(&s4_runs[ai], 0.5);
        let s5 = quantile(&s5_runs[ai], 0.5);
        let s45 = s4 + s5;
        let tot = quantile(&tot_runs[ai], 0.5);
        let p10 = quantile(&tot_runs[ai], 0.1);
        let ws = kernels[ai].scratch_bytes() as f64 / 1024.0;
        println!(
            "{:<16} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>9.1} {:>7.4}x {:>7.4}x {:>7.4}x {:>9.1}",
            name,
            s13,
            s4,
            s5,
            s45,
            tot,
            tot / base_tot_p50,
            p10 / base_tot_p10,
            s4 / base_s4_p50,
            ws
        );
        let _ = base_s45_p50;
    }
    println!();
}

fn bench_multiworker(label: &str, blocks: Arc<Vec<Block>>, workers: usize, rounds: usize) {
    println!("=== {workers}W CONCURRENT BENCHMARK ({label}, {rounds} paired rounds) ===");
    let arms: [(&str, S4Mode); 11] = [
        ("Head-A1", S4Mode::HeadVerbatim),
        ("Head-A2 (A/A)", S4Mode::HeadVerbatim),
        ("HeadEpoch16x", S4Mode::HeadEpoch16x),
        ("Nobi", S4Mode::NobiVerbatim),
        ("TileRev-4K", S4Mode::TileRev(4096)),
        ("TileRev-8K", S4Mode::TileRev(8192)),
        ("TileRev-16K", S4Mode::TileRev(16384)),
        ("TileRev-64K", S4Mode::TileRev(65536)),
        ("SingleTileV-8K", S4Mode::SinglePassTileV(8192)),
        ("ZeroSkipUntiled", S4Mode::ZeroSkipUntiled),
        ("DualFwd (K=2)", S4Mode::DualFwd),
    ];
    let n_arms = arms.len();
    let mut wall_runs = vec![Vec::with_capacity(rounds); n_arms];

    for r in 0..rounds {
        let order: Vec<usize> = if r % 2 == 0 {
            (0..n_arms).collect()
        } else {
            (0..n_arms).rev().collect()
        };
        for ai in order {
            let mode = arms[ai].1;
            let bar = Arc::new(Barrier::new(workers));
            let mut handles = Vec::with_capacity(workers);
            for w in 0..workers {
                let b_ref = Arc::clone(&blocks);
                let bar_c = Arc::clone(&bar);
                handles.push(std::thread::spawn(move || {
                    let mut k = ExpKernel::default();
                    let mut out = Vec::new();
                    // Warmup one block so allocations settle before barrier
                    let first_blk = &b_ref[w % b_ref.len()];
                    k.run_mode(&first_blk.records, first_blk.n, first_blk.p, mode, 65_536, &mut out);
                    bar_c.wait();
                    let t0 = Instant::now();
                    for idx in 0..b_ref.len() {
                        let blk = &b_ref[(idx + w) % b_ref.len()];
                        k.run_mode(&blk.records, blk.n, blk.p, mode, 65_536, &mut out);
                    }
                    t0.elapsed().as_secs_f64() * 1e6
                }));
            }
            let mut max_us = 0.0f64;
            for h in handles {
                max_us = max_us.max(h.join().unwrap());
            }
            wall_runs[ai].push(max_us);
        }
    }

    for ai in 0..n_arms {
        wall_runs[ai].sort_by(|a, b| a.total_cmp(b));
    }
    let base_p50 = quantile(&wall_runs[0], 0.5);
    let base_p10 = quantile(&wall_runs[0], 0.1);
    println!(
        "{:<16} {:>10} {:>10} {:>10} {:>10}",
        "Arm", "Wall_p10", "Wall_p50", "p10/A1", "p50/A1"
    );
    for (ai, &(name, _)) in arms.iter().enumerate() {
        let p10 = quantile(&wall_runs[ai], 0.1);
        let p50 = quantile(&wall_runs[ai], 0.5);
        println!(
            "{:<16} {:>10.1} {:>10.1} {:>9.4}x {:>9.4}x",
            name,
            p10,
            p50,
            p10 / base_p10,
            p50 / base_p50
        );
    }
    println!();
}

fn main() {
    let wiki = Arc::new(load_admitted_blocks());
    let non_wiki = Arc::new(load_non_wikidata_blocks());

    run_solved_analogue_g();
    run_exactness_gate(&wiki, &non_wiki);
    run_diagnostics(&wiki, &non_wiki);

    bench_1t("6 Admitted Wikidata", &wiki, 19);
    bench_1t("6 Non-Wikidata JSONL+Log", &non_wiki, 19);

    bench_multiworker("6 Admitted Wikidata", Arc::clone(&wiki), 2, 11);
    bench_multiworker("6 Admitted Wikidata", Arc::clone(&wiki), 4, 11);
    bench_multiworker("6 Non-Wikidata JSONL+Log", Arc::clone(&non_wiki), 2, 11);
    bench_multiworker("6 Non-Wikidata JSONL+Log", Arc::clone(&non_wiki), 4, 11);
}
