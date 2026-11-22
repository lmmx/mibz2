#![forbid(unsafe_code)]

mod open_local;

use mibz2::block::Symbols;
use mibz2::runs::{front_scalar, Runs};
use open_local::Kernel as LocalKernel;
use std::fs;
use std::path::Path;

struct BlockMeta { file: &'static str, n: usize, p: u32 }

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

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let mut raw_blocks = Vec::new();
    let mut runs_vec = Vec::new();
    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).unwrap();
        let syms = encode_symbols(&l, b.p);
        let mut runs = Runs::default();
        front_scalar(&syms, &mut runs).unwrap();
        raw_blocks.push(l);
        runs_vec.push((b.file, b.n, b.p, runs));
    }

    // 1. 5-Stage breakdown for LIMIT = 16, 12, 8, 6, 4, 3, 2, 1!
    println!("=== 5-STAGE BREAKDOWN OF OPEN (6 BLOCKS SUMMED, 11 ROUNDS MEDIAN ms) ===");
    println!("{:<6} | {:>7} | {:>6} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} | {:>8}",
        "LIMIT", "m_ret", "thick", "1stPass", "Induce", "Wire", "Inline", "Emit", "Total");
    println!("{}", "-".repeat(86));
    let mut k = LocalKernel::default();
    let mut out = Vec::with_capacity(900_016);
    for &lim in &[16u32, 12, 8, 6, 4, 3, 2, 1] {
        let mut v = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vtot = Vec::new();
        let (mut sm, mut sthick) = (0, 0);
        for r in 0..13 {
            let mut s = [0.0; 5];
            sm = 0; sthick = 0;
            for (_, n, p, runs) in &runs_vec {
                let (c, st) = k.run_staged(runs.records(), *n, *p, lim, &mut out);
                sm += c.retained;
                sthick += c.thick;
                for i in 0..5 { s[i] += st[i] * 1000.0; }
            }
            if r >= 2 {
                let tot: f64 = s.iter().sum();
                for i in 0..5 { v[i].push(s[i]); }
                vtot.push(tot);
            }
        }
        println!("{:<6} | {:>7} | {:>6} | {:>7.3} | {:>7.3} | {:>7.3} | {:>7.3} | {:>7.3} | {:>8.3}",
            lim, sm, sthick,
            median(v[0].clone()), median(v[1].clone()), median(v[2].clone()),
            median(v[3].clone()), median(v[4].clone()), median(vtot));
    }

    // 2. Band & Thin-Run Tunnel Anatomy (6 blocks summed)
    println!("\n=== BAND-LEVEL & RUN-LEVEL TUNNEL ANATOMY (6 BLOCKS SUMMED) ===");
    let mut tot_bands = 0usize;
    let mut bands_in_single_thin_run = 0usize;
    let mut bands_in_single_thin_run_rows = 0usize;
    let mut bands_literal = 0usize;
    let mut bands_literal_rows = 0usize;
    let mut bands_word = 0usize;
    let mut bands_word_rows = 0usize;

    let mut run_cnt_w = [0usize; 17];
    let mut run_rows_w = [0usize; 17];
    let mut run_exact_match_w = [0usize; 17];   // LF(R_i) == R_j (exact same start & width!)
    let mut run_contained_in_thin_w = [0usize; 17]; // LF(R_i) ⊆ some thin run R_j (w_j <= 16)
    let mut run_contained_in_any_w = [0usize; 17];  // LF(R_i) ⊆ ANY maximal run (thin or thick!)

    for (bi, l) in raw_blocks.iter().enumerate() {
        let n = l.len();
        let p = BLOCKS[bi].p as usize;
        // Build Open L=16 bands
        let (_, _, _, ref runs) = runs_vec[bi];
        k.run_staged(runs.records(), n, p as u32, 16, &mut out);
        let bands = k.get_bands_info();

        // Build maximal runs of L and retained run_id per retained row
        let mut counts = [0u32; 256];
        for &b in l { counts[b as usize] += 1; }
        let mut first = [0u32; 256];
        let mut s = 0u32;
        for c in 0..256 { first[c] = s; s += counts[c]; }

        let mut max_runs: Vec<(u8, usize, usize, usize)> = Vec::new(); // (byte, from, to, width)
        let mut cur_c = first;
        let mut row_to_maxrun = vec![0u32; n];
        let mut i = 0;
        while i < n {
            let c = l[i];
            let st = i;
            i += 1;
            while i < n && l[i] == c { i += 1; }
            let w = i - st;
            let to = cur_c[c as usize] as usize;
            cur_c[c as usize] += w as u32;
            let rid = max_runs.len() as u32;
            for x in st..i { row_to_maxrun[x] = rid; }
            max_runs.push((c, st, to, w));
        }

        // Retained ranks -> thin run id
        let mut ret_run_id = Vec::new();
        for (rid, &(_, st, _, w)) in max_runs.iter().enumerate() {
            if w <= 16 {
                for _ in 0..w { ret_run_id.push(rid as u32); }
            } else if p >= st && p < st + w {
                ret_run_id.push(u32::MAX);
            }
        }

        // Check bands!
        for &(dst, span, code) in &bands {
            tot_bands += 1;
            if code < 256 {
                bands_literal += 1;
                bands_literal_rows += span as usize;
            } else {
                bands_word += 1;
                bands_word_rows += span as usize;
            }
            if span >= 2 {
                let r0 = ret_run_id[dst as usize];
                if r0 != u32::MAX {
                    let r1 = ret_run_id[(dst + span - 1) as usize];
                    if r0 == r1 {
                        bands_in_single_thin_run += 1;
                        bands_in_single_thin_run_rows += span as usize;
                    }
                }
            }
        }

        // Check raw thin runs w=2..16 under LF!
        for &(_, _from, to, w) in &max_runs {
            if w <= 16 {
                run_cnt_w[w] += 1;
                run_rows_w[w] += w;
                if w >= 2 {
                    let r_start = row_to_maxrun[to] as usize;
                    let r_end = row_to_maxrun[to + w - 1] as usize;
                    if r_start == r_end {
                        run_contained_in_any_w[w] += 1;
                        let (_, tgt_from, _, tgt_w) = max_runs[r_start];
                        if tgt_w <= 16 {
                            run_contained_in_thin_w[w] += 1;
                            if tgt_from == to && tgt_w == w {
                                run_exact_match_w[w] += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("  Total Open(L=16) bands: {} | literal bands: {} ({} rows) | word bands: {} ({} rows)",
        tot_bands, bands_literal, bands_literal_rows, bands_word, bands_word_rows);
    println!("  Bands (span>=2) landing ENTIRELY inside a single thin run: {} bands ({} rows = {:.2}% of m_16!)",
        bands_in_single_thin_run, bands_in_single_thin_run_rows,
        100.0 * bands_in_single_thin_run_rows as f64 / 895391.0);

    println!("\n  Thin-run width anatomy (w=2..16, raw LF in L):");
    println!("  {:>3} | {:>7} | {:>7} | {:>12} | {:>12} | {:>12}",
        "w", "runs", "rows", "LF⊆any_run", "LF⊆thin_run", "LF==thin_run");
    let (mut sr, mut srow, mut sany, mut sthin, mut sexact) = (0,0,0,0,0);
    for w in 2..=16 {
        sr += run_cnt_w[w]; srow += run_rows_w[w];
        sany += run_contained_in_any_w[w]; sthin += run_contained_in_thin_w[w]; sexact += run_exact_match_w[w];
        println!("  {:>3} | {:>7} | {:>7} | {:>7} ({:4.1}%) | {:>7} ({:4.1}%) | {:>7} ({:4.1}%)",
            w, run_cnt_w[w], run_rows_w[w],
            run_contained_in_any_w[w], 100.0 * run_contained_in_any_w[w] as f64 / run_cnt_w[w].max(1) as f64,
            run_contained_in_thin_w[w], 100.0 * run_contained_in_thin_w[w] as f64 / run_cnt_w[w].max(1) as f64,
            run_exact_match_w[w], 100.0 * run_exact_match_w[w] as f64 / run_cnt_w[w].max(1) as f64);
    }
    println!("  ALL | {:>7} | {:>7} | {:>7} ({:4.1}%) | {:>7} ({:4.1}%) | {:>7} ({:4.1}%)",
        sr, srow,
        sany, 100.0 * sany as f64 / sr as f64,
        sthin, 100.0 * sthin as f64 / sr as f64,
        sexact, 100.0 * sexact as f64 / sr as f64);
}
