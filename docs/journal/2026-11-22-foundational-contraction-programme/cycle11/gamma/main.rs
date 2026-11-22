#![forbid(unsafe_code)]

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::collections::HashSet;
use std::fs;

fn rle1_decoded_len(src: &[u8]) -> usize {
    let mut out_len = 0usize;
    let mut i = 0usize;
    let n = src.len();
    let mut prev: u16 = 256;
    let mut run = 0usize;
    while i < n {
        let b = src[i];
        i += 1;
        out_len += 1;
        if u16::from(b) == prev {
            run += 1;
            if run == 4 {
                if i < n {
                    out_len += usize::from(src[i]);
                    i += 1;
                }
                prev = 256;
                run = 0;
            }
        } else {
            prev = u16::from(b);
            run = 1;
        }
    }
    out_len
}

fn main() {
    let k = Kernels::detect();
    let bz2 = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").expect("read wikidata.bz2");
    let mut scanner = Scanner::new(SliceSource(&bz2));
    let mut syms = Vec::new();
    let mut runs = Runs::default();
    let mut open = OpenCells::default();
    let mut ibwt_buf = Vec::with_capacity(900_064);

    let mut total_blocks = 0usize;
    let mut total_ibwt_bytes = 0u64;
    let mut total_rle1_bytes = 0u64;
    let mut rle1_blocks_gt1pct = 0usize;
    let mut max_rle1_ratio = 1.0f64;

    // Upstream MTF/RLE2 symbol counts across all 431 blocks
    let mut tot_syms = 0u64;
    let mut tot_runab = 0u64;
    let mut tot_mtf1 = 0u64;
    let mut tot_mtf2 = 0u64;
    let mut tot_mtf3p = 0u64;

    // Inter-block neighbor similarity (consecutive full >=800KB blocks, first 20 pairs!)
    let mut prev_hist: Option<[u32; 256]> = None;
    let mut prev_n: usize = 0;
    let mut prev_s16: Option<HashSet<[u8; 16]>> = None;
    let mut prev_s32: Option<HashSet<[u8; 32]>> = None;
    let mut pair_cnt = 0usize;
    let mut sum_l1 = 0.0f64;
    let mut sum_ov16 = 0.0f64;
    let mut sum_ov32 = 0.0f64;

    let mut win_high_runab_thick = 0u64;
    let mut win_high_runab_tot = 0u64;
    let mut win_low_runab_thin = 0u64;
    let mut win_low_runab_tot = 0u64;
    let mut full_analyzed = 0usize;

    loop {
        match scanner.next(syms).expect("scan") {
            Event::Done => break,
            Event::Block { symbols, .. } => {
                total_blocks += 1;
                let eob = symbols.eob;
                for &s in &symbols.syms {
                    if s == eob {
                        break;
                    }
                    tot_syms += 1;
                    match s {
                        0 | 1 => tot_runab += 1,
                        2 => tot_mtf1 += 1,
                        3 => tot_mtf2 += 1,
                        _ => tot_mtf3p += 1,
                    }
                }

                let n = front(&symbols, &mut runs, k).expect("front");
                total_ibwt_bytes += n as u64;
                let ok = open.invert_runs(&runs, n, symbols.orig_ptr, &mut ibwt_buf, k);
                assert!(ok && ibwt_buf.len() == n);

                let n_rle = rle1_decoded_len(&ibwt_buf);
                total_rle1_bytes += n_rle as u64;
                if n > 0 {
                    let ratio = n_rle as f64 / n as f64;
                    if ratio > max_rle1_ratio {
                        max_rle1_ratio = ratio;
                    }
                    if ratio > 1.01 {
                        rle1_blocks_gt1pct += 1;
                    }
                }

                // 50-symbol window correlation with thick vs thin rows (first 20 full blocks)
                if n >= 800_000 && full_analyzed < 20 {
                    full_analyzed += 1;
                    // Walk symbols in 50-symbol chunks, computing exact thick/thin rows produced per chunk!
                    let sy = &symbols.syms;
                    let mut idx = 0usize;
                    let mut cur_run = 0usize;
                    while idx < sy.len() && sy[idx] != eob {
                        let end = (idx + 50).min(sy.len());
                        let mut runab_cnt = 0usize;
                        let mut thick_rows = 0u64;
                        let mut thin_rows = 0u64;
                        let mut z_acc = 0usize;
                        let mut z_bit = 1usize;
                        for &s in &sy[idx..end] {
                            if s == eob { break; }
                            if s <= 1 {
                                runab_cnt += 1;
                                z_acc += (s as usize + 1) * z_bit;
                                z_bit <<= 1;
                            } else {
                                if z_acc > 0 {
                                    let w = z_acc + cur_run;
                                    if w > 16 { thick_rows += w as u64; } else { thin_rows += w as u64; }
                                    z_acc = 0; z_bit = 1;
                                } else if cur_run > 0 {
                                    thin_rows += cur_run as u64;
                                }
                                cur_run = 1;
                            }
                        }
                        if z_acc > 0 {
                            let w = z_acc + cur_run;
                            if w > 16 { thick_rows += w as u64; } else { thin_rows += w as u64; }
                            cur_run = 0;
                        }
                        if runab_cnt >= 25 {
                            win_high_runab_thick += thick_rows;
                            win_high_runab_tot += thick_rows + thin_rows;
                        } else {
                            win_low_runab_thin += thin_rows;
                            win_low_runab_tot += thick_rows + thin_rows;
                        }
                        idx = end;
                    }
                }

                // Neighbor-block similarity on consecutive full >=800KB blocks (first 20 pairs)
                if n >= 800_000 && pair_cnt < 20 {
                    let mut hist = [0u32; 256];
                    for &b in &ibwt_buf {
                        hist[b as usize] += 1;
                    }
                    // Sample every 16th 16B & 32B window to keep runtime < 1s!
                    let mut s16: HashSet<[u8; 16]> = HashSet::with_capacity(n / 16);
                    let mut s32: HashSet<[u8; 32]> = HashSet::with_capacity(n / 16);
                    let mut pos = 0usize;
                    while pos + 32 <= n {
                        let a16: [u8; 16] = ibwt_buf[pos..pos + 16].try_into().unwrap();
                        let a32: [u8; 32] = ibwt_buf[pos..pos + 32].try_into().unwrap();
                        s16.insert(a16);
                        s32.insert(a32);
                        pos += 16;
                    }
                    if let (Some(ph), Some(ps16), Some(ps32)) = (&prev_hist, &prev_s16, &prev_s32) {
                        let mut l1 = 0.0f64;
                        for b in 0..256 {
                            let p1 = ph[b] as f64 / prev_n as f64;
                            let p2 = hist[b] as f64 / n as f64;
                            l1 += (p1 - p2).abs();
                        }
                        let mut hit16 = 0usize;
                        let mut hit32 = 0usize;
                        let mut tot_w = 0usize;
                        let mut q = 0usize;
                        while q + 32 <= n {
                            let a16: [u8; 16] = ibwt_buf[q..q + 16].try_into().unwrap();
                            let a32: [u8; 32] = ibwt_buf[q..q + 32].try_into().unwrap();
                            if ps16.contains(&a16) { hit16 += 1; }
                            if ps32.contains(&a32) { hit32 += 1; }
                            tot_w += 1;
                            q += 16;
                        }
                        sum_l1 += l1;
                        sum_ov16 += hit16 as f64 / tot_w as f64;
                        sum_ov32 += hit32 as f64 / tot_w as f64;
                        pair_cnt += 1;
                    }
                    prev_hist = Some(hist);
                    prev_n = n;
                    prev_s16 = Some(s16);
                    prev_s32 = Some(s32);
                }

                syms = symbols.syms;
            }
        }
    }

    println!("=== 431-BLOCK `corpora/wikidata.bz2` UPSTREAM STREAM & POST-IBWT RLE1 AUDIT ===");
    println!("  Total blocks = {}", total_blocks);
    println!("  Total IBWT bytes = {} ({:.2} MB) -> Post-RLE1 bytes = {} ({:.2} MB), ratio = {:.5}x (blocks >1% exp: {}, max ratio = {:.3}x)",
        total_ibwt_bytes, total_ibwt_bytes as f64 / 1e6,
        total_rle1_bytes, total_rle1_bytes as f64 / 1e6,
        total_rle1_bytes as f64 / total_ibwt_bytes as f64,
        rle1_blocks_gt1pct, max_rle1_ratio);
    println!("  Upstream MTF+RLE2 syms = {} ({:.2}M): RUNA/RUNB = {:.1}%, mtf==1 = {:.1}%, mtf==2 = {:.1}%, mtf>=3 = {:.1}%",
        tot_syms, tot_syms as f64 / 1e6,
        100.0 * tot_runab as f64 / tot_syms as f64,
        100.0 * tot_mtf1 as f64 / tot_syms as f64,
        100.0 * tot_mtf2 as f64 / tot_syms as f64,
        100.0 * tot_mtf3p as f64 / tot_syms as f64);
    println!("  50-sym window predictor (20 full blks): high-RUNAB (>=25/50) -> {:.1}% thick rows; low-RUNAB (<25/50) -> {:.1}% thin rows",
        100.0 * win_high_runab_thick as f64 / win_high_runab_tot.max(1) as f64,
        100.0 * win_low_runab_thin as f64 / win_low_runab_tot.max(1) as f64);
    println!("  Neighbor-block similarity ({} consecutive 900KB pairs): mean hist L1 = {:.4} (TV = {:.4}), 16B overlap = {:.1}%, 32B overlap = {:.1}%",
        pair_cnt, sum_l1 / pair_cnt as f64, 0.5 * sum_l1 / pair_cnt as f64,
        100.0 * sum_ov16 / pair_cnt as f64, 100.0 * sum_ov32 / pair_cnt as f64);
}
