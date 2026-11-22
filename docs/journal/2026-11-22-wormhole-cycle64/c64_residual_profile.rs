use std::fs;
use std::time::Instant;

use mibz2::bits::Bits;
use mibz2::block::{read_item, Item};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::{ErrorKind, Kernels, Scratch};

mod c64_stages {
    #![forbid(unsafe_code)]
    use super::*;
    pub type Run = (u32, u8, u8);
    const TILE: usize = 64 << 10;

    pub struct FastAnchorFinder { bm: [u64; 1024] }
    impl FastAnchorFinder {
        pub fn new() -> Self {
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let mut bm = [0u64; 1024];
            for a in 0..256usize {
                for b in 0..256usize {
                    let v = ((a as u64) << 56) | (BLOCK_MAGIC << 8) | (b as u64);
                    for shift in 0..8u32 {
                        let sv = v >> shift;
                        for pos in 0..5usize {
                            let pair = ((sv >> (8 * pos)) & 0xFFFF) as usize;
                            bm[pair >> 6] |= 1u64 << (pair & 63);
                        }
                    }
                }
            }
            for &(x, y) in &[(b'B', b'Z'), (b'Z', b'h')] {
                let idx = ((x as usize) << 8) | (y as usize);
                bm[idx >> 6] |= 1u64 << (idx & 63);
            }
            Self { bm }
        }
        pub fn scan(&self, win: &[u8], out: &mut Vec<(u64, u8)>) {
            out.clear();
            if win.len() < 10 { return; }
            let (mut level, mut last_bit) = (0u8, -1i64);
            if win[0] == b'B' && win[1] == b'Z' && win[2] == b'h' && (b'1'..=b'9').contains(&win[3]) {
                level = win[3] - b'0';
            }
            let bm = &self.bm;
            let (mut p, n) = (0usize, win.len());
            const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
            let mut verify = |center: usize| {
                let lo = center.saturating_sub(4);
                let hi = (center + 2).min(n.saturating_sub(4));
                for i in lo..=hi {
                    if i + 4 <= n && win[i] == b'B' && win[i+1] == b'Z' && win[i+2] == b'h' && (b'1'..=b'9').contains(&win[i+3]) {
                        level = win[i+3] - b'0';
                    }
                    if level == 0 || i + 6 >= n { continue; }
                    let w = ((win[i] as u64) << 48) | ((win[i+1] as u64) << 40) | ((win[i+2] as u64) << 32)
                        | ((win[i+3] as u64) << 24) | ((win[i+4] as u64) << 16) | ((win[i+5] as u64) << 8) | (win[i+6] as u64);
                    for sh in 0u32..8 {
                        if ((w >> (8 - sh)) & 0xFFFF_FFFF_FFFF) == BLOCK_MAGIC {
                            let bit = (i as u64) * 8 + sh as u64;
                            if (bit as i64) > last_bit { out.push((bit, level)); last_bit = bit as i64; }
                        }
                    }
                }
            };
            while p + 14 <= n {
                let s = &win[p..p + 14];
                let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
                let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
                let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
                let i3 = ((s[12] as usize) << 8) | (s[13] as usize);
                if (((bm[i0>>6]>>(i0&63)) | (bm[i1>>6]>>(i1&63)) | (bm[i2>>6]>>(i2&63)) | (bm[i3>>6]>>(i3&63))) & 1) != 0 {
                    if ((bm[i0>>6]>>(i0&63)) & 1) != 0 { verify(p); }
                    if ((bm[i1>>6]>>(i1&63)) & 1) != 0 { verify(p+4); }
                    if ((bm[i2>>6]>>(i2&63)) & 1) != 0 { verify(p+8); }
                    if ((bm[i3>>6]>>(i3&63)) & 1) != 0 { verify(p+12); }
                }
                p += 16;
            }
            while p + 2 <= n {
                let idx = ((win[p] as usize) << 8) | (win[p + 1] as usize);
                if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 { verify(p); }
                p += 4;
            }
        }
    }

    #[inline]
    pub fn find_runs_swar(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let (n, mut p) = (rle.len(), 0usize);
        while p + 8 <= n {
            let w = u64::from_le_bytes(rle[p..p + 8].try_into().unwrap());
            let d = w ^ (w >> 8);
            let z = !(((d & 0x7F7F_7F7F_7F7F_7F7F) + 0x7F7F_7F7F_7F7F_7F7F) | d) & 0x8080_8080_8080_8080;
            let m = z & (z >> 8) & (z >> 16) & 0x0000_00FF_FFFF_FFFF;
            if m == 0 { p += 5; } else {
                let idx = p + (m.trailing_zeros() >> 3) as usize;
                let Some(&extra) = rle.get(idx + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((idx as u32, rle[idx], extra));
                p = idx + 5;
            }
        }
        while p + 3 < n {
            let c = rle[p];
            if rle[p+1] == c && rle[p+2] == c && rle[p+3] == c {
                let Some(&extra) = rle.get(p + 4) else { return Err(ErrorKind::RunUnfinished); };
                runs.push((p as u32, c, extra));
                p += 5;
            } else { p += 1; }
        }
        Ok(())
    }

    pub fn unrle_zs52(buf: &mut Vec<u8>, runs: &mut Vec<Run>, k: Kernels) -> Result<(u32, usize, usize), ErrorKind> {
        let n = buf.len();
        find_runs_swar(buf, runs)?;
        if runs.is_empty() { return Ok((mibz2::crc::crc32(buf, k), 0, 0)); }
        let (mut grown, mut max_end) = (0isize, n);
        for &(j, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            let e = (j as usize + 5) as isize + grown;
            if e as usize > max_end { max_end = e as usize; }
        }
        let final_len = (n as isize + grown) as usize;
        if final_len > max_end { max_end = final_len; }
        let grew_over_32 = if max_end > n + n / 32 + 64 { 1 } else { 0 };
        let grew_over_16 = if max_end > n + n / 16 + 64 { 1 } else { 0 };
        if max_end > buf.len() {
            if max_end > buf.capacity() { buf.reserve_exact(max_end - buf.len()); }
            buf.resize(max_end, 0);
        }
        let (mut crc_s, mut crc_at) = (!0u32, 0usize);
        let crc_flush = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE { *s = mibz2::crc::update_with(*s, &buf[*at..*at + TILE], k); *at += TILE; }
        };
        let (rlen, mut idx, mut src_pos, mut g) = (runs.len(), 0usize, 0usize, 0isize);
        while idx < rlen {
            let (j0, c0, e0) = runs[idx];
            let (j0u, g_after0) = (j0 as usize, g + (e0 as isize - 1));
            if g <= 0 && g_after0 <= 0 {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                let dst_run = (j0u as isize + g) as usize;
                let rl = 4 + e0 as usize;
                buf[dst_run..dst_run + rl].fill(c0);
                src_pos = j0u + 5; g = g_after0; idx += 1;
                crc_flush(&mut crc_s, &mut crc_at, buf, dst_run + rl);
            } else {
                let dst_lit = (src_pos as isize + g) as usize;
                if j0u > src_pos && dst_lit != src_pos { buf.copy_within(src_pos..j0u, dst_lit); }
                crc_flush(&mut crc_s, &mut crc_at, buf, (j0u as isize + g) as usize);
                let (hill_start, mut cur_g, mut k_i, mut closed) = (idx, g_after0, idx + 1, false);
                while k_i < rlen {
                    let ng = cur_g + (runs[k_i].2 as isize - 1);
                    if ng == 0 { closed = true; break; }
                    cur_g = ng; k_i += 1;
                }
                let (hill_end, mut se, mut de, next_src, next_g) = if closed {
                    let jk = runs[k_i].0 as usize;
                    (k_i + 1, jk + 5, jk + 5, jk + 5, 0isize)
                } else { (rlen, n, (n as isize + cur_g) as usize, n, cur_g) };
                for r_i in (hill_start..hill_end).rev() {
                    let (rj, rc, re) = runs[r_i];
                    let (rju, lit_s) = (rj as usize, rj as usize + 5);
                    if se > lit_s {
                        let ll = se - lit_s; de -= ll;
                        if de != lit_s { buf.copy_within(lit_s..se, de); }
                    }
                    let rl = 4 + re as usize; de -= rl; buf[de..de + rl].fill(rc); se = rju;
                }
                let hill_dst_end = if closed { runs[k_i].0 as usize + 5 } else { (n as isize + cur_g) as usize };
                crc_flush(&mut crc_s, &mut crc_at, buf, hill_dst_end);
                idx = hill_end; src_pos = next_src; g = next_g;
            }
        }
        if src_pos < n {
            let dst_lit = (src_pos as isize + g) as usize;
            if dst_lit != src_pos { buf.copy_within(src_pos..n, dst_lit); }
        }
        buf.truncate(final_len);
        crc_s = mibz2::crc::update_with(crc_s, &buf[crc_at..], k);
        Ok((!crc_s, grew_over_32, grew_over_16))
    }
}

fn profile_corpus(name: &str, path: &str, rounds: usize, k: Kernels) {
    let data = fs::read(path).unwrap();
    let finder = c64_stages::FastAnchorFinder::new();
    let mut cands = Vec::new();
    let (mut t_scan, mut t_read, mut t_front, mut t_bwt, mut t_rle, mut t_col) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (mut n_blks, mut n_runs_route, mut n_col_route, mut over32, mut over16, mut out_bytes) = (0usize, 0usize, 0usize, 0usize, 0usize, 0u64);

    for r in 0..rounds {
        let ts0 = Instant::now();
        finder.scan(&data, &mut cands);
        t_scan += ts0.elapsed().as_secs_f64() * 1e3;

        let mut syms = Vec::new();
        let mut runs = Runs::default();
        let mut inv = OpenCells::default();
        let mut rle_runs = Vec::new();
        let mut sc = Scratch::<OpenCells>::default();
        let mut out = Vec::with_capacity(1 << 20);
        if r == 0 { n_blks = 0; n_runs_route = 0; n_col_route = 0; over32 = 0; over16 = 0; out_bytes = 0; }

        for &(bit, level) in &cands {
            let tr0 = Instant::now();
            let mut br = Bits::at(&data, bit);
            let res = read_item(&mut br, level as u32, std::mem::take(&mut syms));
            t_read += tr0.elapsed().as_secs_f64() * 1e3;
            let Ok(Item::Block(b)) = res else { continue; };
            if r == 0 { n_blks += 1; }
            if inv.reads_runs() && runs::takes_runs(&b) {
                if r == 0 { n_runs_route += 1; }
                let tf0 = Instant::now();
                let n = runs::front(&b, &mut runs, k).unwrap();
                t_front += tf0.elapsed().as_secs_f64() * 1e3;

                let tb0 = Instant::now();
                let done = inv.invert_runs(&runs, n, b.orig_ptr, &mut out, k);
                assert!(done);
                t_bwt += tb0.elapsed().as_secs_f64() * 1e3;

                let tu0 = Instant::now();
                let (crc, g32, g16) = c64_stages::unrle_zs52(&mut out, &mut rle_runs, k).unwrap();
                t_rle += tu0.elapsed().as_secs_f64() * 1e3;
                assert_eq!(crc, b.crc);
                if r == 0 { over32 += g32; over16 += g16; out_bytes += out.len() as u64; }
            } else {
                if r == 0 { n_col_route += 1; }
                let tc0 = Instant::now();
                out.clear();
                let crc = mibz2::decode_block(&b, k, &mut sc, &mut out).unwrap();
                assert_eq!(crc, b.crc);
                t_col += tc0.elapsed().as_secs_f64() * 1e3;
                if r == 0 { out_bytes += out.len() as u64; }
            }
            syms = b.syms;
        }
    }
    let rf = rounds as f64;
    let (s, rd, fr, bw, rl, co) = (t_scan/rf, t_read/rf, t_front/rf, t_bwt/rf, t_rle/rf, t_col/rf);
    let tot = s + rd + fr + bw + rl + co;
    println!("\n=== {name} ({n_blks} blks: {n_runs_route} runs / {n_col_route} col, {:.2} MB out, {rounds} rd avg) ===", out_bytes as f64 / 1e6);
    println!("  S0 AnchorScan (serial): {:>7.2} ms ({:>5.1}%)", s, 100.0 * s / tot);
    println!("  S1 read_item  (par):    {:>7.2} ms ({:>5.1}%)", rd, 100.0 * rd / tot);
    println!("  S2 runs::front (par):   {:>7.2} ms ({:>5.1}%)", fr, 100.0 * fr / tot);
    println!("  S3 OpenCells   (par):   {:>7.2} ms ({:>5.1}%)", bw, 100.0 * bw / tot);
    println!("  S4 ZeroShift52+CRC(par):{:>7.2} ms ({:>5.1}%)  [grew >n/32: {over32}, >n/16: {over16}]", rl, 100.0 * rl / tot);
    if co > 0.01 { println!("  S3+4 ColRoute  (par):   {:>7.2} ms ({:>5.1}%)", co, 100.0 * co / tot); }
    println!("  TOTAL 1T CPU:           {:>7.2} ms (100.0%)  |  Parallelizable share: {:.2}%", tot, 100.0 * (tot - s) / tot);
}

fn main() {
    let k = Kernels::detect();
    println!("=== CYCLE 64: POST-#739 + PR4 + ZEROSHIFT52 RESIDUAL 1T STAGE PROFILE ===");
    profile_corpus("wikidata-l9", "/tmp/wormhole_r6/wikidata_431b_210m.bz2", 5, k);
    profile_corpus("wikidata-l1", "/tmp/wormhole_r6/wikidata_l1_36m.bz2", 10, k);
    profile_corpus("hetero-l9", "/tmp/wormhole_r6/hetero_60b.bz2", 5, k);
    profile_corpus("random-l9", "/tmp/wormhole_r6/random_l9_11m.bz2", 10, k);
}
