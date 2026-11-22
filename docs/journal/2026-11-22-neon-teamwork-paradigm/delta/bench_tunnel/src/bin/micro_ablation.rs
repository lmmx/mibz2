#![forbid(unsafe_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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

const INLINE: usize = 16;
const LONG: u8 = 255;
const TUNNEL: u8 = 254;
const WIDE_BIT: u32 = 1u32 << 31;
const CODE_MASK: u32 = !WIDE_BIT;
const SENTINEL: u32 = 0x7FFF_FFE0; // bit 31 = 0, never matches any real edge +/- 15

struct Snap {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    edges: Vec<u32>,
    edges_wide: Vec<u32>, // bit 31 set iff row u is in a 1-step strip of width >= 2 (and != mark)
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

fn build_snap(l: &[u8], p: u32) -> Snap {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l { counts[b as usize] += 1; }
    let mut first = [0u32; 256];
    let mut s = 0u32;
    for c in 0..256 { first[c] = s; s += counts[c]; }
    let mut cur = first;
    let mut psi = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let j = cur[b as usize];
        cur[b as usize] += 1;
        psi[j as usize] = i as u32;
    }
    let mut f_col = vec![0u8; n];
    for c in 0..256 {
        let st = first[c] as usize;
        let en = if c == 255 { n } else { first[c + 1] as usize };
        for x in st..en { f_col[x] = c as u8; }
    }
    let mut is_ret = vec![false; n];
    let mut i = 0;
    while i < n {
        let c = l[i];
        let st = i;
        i += 1;
        while i < n && l[i] == c { i += 1; }
        let w = i - st;
        if w <= 16 {
            for x in st..i { is_ret[x] = true; }
        } else if (p as usize) >= st && (p as usize) < i {
            is_ret[p as usize] = true;
        }
    }
    let mut ret_rows = Vec::new();
    let mut row_to_rank = vec![u32::MAX; n];
    for x in 0..n {
        if is_ret[x] {
            row_to_rank[x] = ret_rows.len() as u32;
            ret_rows.push(x as u32);
        }
    }
    let m = ret_rows.len();
    let mark = row_to_rank[p as usize];
    let bits = (u32::BITS - (m as u32).leading_zeros()).max(1);
    let mut spelling = Vec::new();
    let mut offsets = vec![0u32];
    let mut inline16: Vec<[u8; INLINE]> = (0..=255u8).map(|c| { let mut v = [0u8; INLINE]; v[0] = c; v }).collect();
    let mut lens = vec![1u8; 256];
    let mut word_map = HashMap::<Vec<u8>, u32>::new();
    let mut edges = vec![0u32; m + 1];
    let mut buf = Vec::new();
    for (u, &row) in ret_rows.iter().enumerate() {
        buf.clear();
        let mut x = row as usize;
        loop {
            buf.push(f_col[x]);
            x = psi[x] as usize;
            if is_ret[x] { break; }
        }
        let next = row_to_rank[x];
        let code = if buf.len() == 1 {
            buf[0] as u32
        } else if let Some(&id) = word_map.get(&buf) {
            id
        } else {
            let id = (256 + offsets.len() - 1) as u32;
            spelling.extend_from_slice(&buf);
            offsets.push(spelling.len() as u32);
            let mut v = [0u8; INLINE];
            let h = buf.len().min(INLINE);
            v[..h].copy_from_slice(&buf[..h]);
            inline16.push(v);
            lens.push(if buf.len() <= INLINE { buf.len() as u8 } else { LONG });
            word_map.insert(buf.clone(), id);
            id
        };
        edges[u] = (code << bits) | next;
    }
    edges[m] = SENTINEL;

    // Build edges_wide: bit 31 = 1 iff u != mark and u is coupled with u-1 or u+1
    let mut edges_wide = edges.clone();
    let mut u = 0usize;
    while u < m {
        if u == mark as usize {
            u += 1;
            continue;
        }
        let e = edges[u];
        let mut w = 1usize;
        while u + w < m && (u + w) != mark as usize && edges[u + w] == e.wrapping_add(w as u32) {
            w += 1;
        }
        if w >= 2 {
            for k in 0..w {
                edges_wide[u + k] |= WIDE_BIT;
            }
        }
        u += w;
    }
    edges_wide[m] = SENTINEL;

    Snap { n, m, mark, bits, edges, edges_wide, spelling, offsets, inline16, lens }
}

fn emit_plain(snap: &Snap, edges: &[u32], out: &mut Vec<u8>) {
    out.resize(snap.n + INLINE, 0);
    let mut at = 0usize;
    let mask = (1u32 << snap.bits) - 1;
    let mut j = snap.mark;
    for _ in 0..snap.m {
        let edge = edges[j as usize];
        let code = (edge >> snap.bits) as usize;
        j = edge & mask;
        let len = snap.lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&snap.inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
            let take = word.len().min(snap.n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == snap.mark || at >= snap.n { break; }
    }
    out.truncate(at.min(snap.n));
}

/// Ablation 1: Plain emit + ONLY neighbor check (never tracks, never writes!)
fn emit_check_only(snap: &Snap, edges: &[u32], out: &mut Vec<u8>) -> usize {
    out.resize(snap.n + INLINE, 0);
    let mut at = 0usize;
    let mask = (1u32 << snap.bits) - 1;
    let mut j = snap.mark;
    let mut hits = 0usize;
    for _ in 0..snap.m {
        let u = j as usize;
        let edge = edges[u];
        let code = (edge >> snap.bits) as usize;
        let has = (edges[u.saturating_sub(1)] == edge.wrapping_sub(1))
                | (edges[u + 1] == edge.wrapping_add(1));
        hits += usize::from(has);
        j = edge & mask;
        let len = snap.lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&snap.inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
            let take = word.len().min(snap.n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == snap.mark || at >= snap.n { break; }
    }
    out.truncate(at.min(snap.n));
    hits
}

/// Ablation 2: WideBit LazyTunnel! Width-1 rows check ONLY `(e as i32) < 0` (ZERO neighbor loads!).
fn emit_widebit_lazy(
    snap: &Snap,
    edges: &mut [u32],
    lens: &mut Vec<u8>,
    tunnels: &mut Vec<(u32, u32)>,
    min_steps: u32,
    out: &mut Vec<u8>,
) {
    let m = snap.m;
    let n = snap.n;
    let bits = snap.bits;
    let mask = (1u32 << bits) - 1;
    let mark_idx = snap.mark as usize;
    let tunnel_base = snap.lens.len();
    let max_codes = (CODE_MASK >> bits).saturating_sub(2) as usize;

    lens.clear();
    lens.extend_from_slice(&snap.lens);
    tunnels.clear();
    out.resize(n + INLINE, 0);

    let mark_edge = edges[mark_idx];
    edges[mark_idx] = SENTINEL;
    edges[m] = SENTINEL;

    let mut at = 0usize;
    let mut j = {
        let code = ((mark_edge & CODE_MASK) >> bits) as usize;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&snap.inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        (mark_edge & mask) as usize
    };

    if j == mark_idx || at >= n {
        out.truncate(at.min(n));
        return;
    }

    let max_s: usize = 15;
    let mut rem = m - 1;

    'outer: while rem > 0 {
        rem -= 1;
        let u = j;
        let e = edges[u];
        let is_wide = (e as i32) < 0;
        let e_clean = e & CODE_MASK;
        let code = (e_clean >> bits) as usize;
        let next = (e_clean & mask) as usize;

        if !is_wide {
            // FAST WIDTH-1 OR TUNNEL-HIT PATH: ZERO NEIGHBOR LOADS!
            let len = lens[code];
            if len <= INLINE as u8 {
                out[at..at + INLINE].copy_from_slice(&snap.inline16[code]);
                at += usize::from(len);
            } else if len == TUNNEL {
                let (out_pos, byte_len) = tunnels[code - tunnel_base];
                let src = out_pos as usize;
                let take = (byte_len as usize).min(n - at);
                out.copy_within(src..src + take, at);
                at += take;
            } else {
                let w = code - 256;
                let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            j = next;
            if j == mark_idx || at >= n { break 'outer; }
            continue;
        }

        // WIDE_BIT is set -> enter tunnel tracking!
        let step_start_at = at;
        let len = lens[code];
        if len <= INLINE as u8 {
            out[at..at + INLINE].copy_from_slice(&snap.inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        j = next;
        if j == mark_idx || at >= n { break 'outer; }

        let max_l = max_s.min(u).min(next);
        let mut t_left = 0usize;
        while t_left < max_l && edges[u - t_left - 1] == e.wrapping_sub((t_left + 1) as u32) {
            t_left += 1;
        }
        let max_r = max_s.min(m - 1 - u).min(m - 1 - next);
        let mut t_right = 0usize;
        while t_right < max_r && edges[u + t_right + 1] == e.wrapping_add((t_right + 1) as u32) {
            t_right += 1;
        }
        if t_left + t_right == 0 {
            continue;
        }
        let t_start_u = u;
        let t_start_out = step_start_at;
        let mut t_steps = 1u32;

        while rem > 0 {
            rem -= 1;
            let cu = j;
            let ce = edges[cu];
            let cwide = (ce as i32) < 0;
            let ce_clean = ce & CODE_MASK;
            let ccode = (ce_clean >> bits) as usize;
            let cnext = (ce_clean & mask) as usize;

            if !cwide {
                // Tunnel ended at cu (either width-1 or existing tunnel hit)!
                if t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                    let tid_code = lens.len() as u32;
                    lens.push(TUNNEL);
                    tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                    let shifted = tid_code << bits; // WIDE_BIT = 0!
                    for d in 1..=t_left {
                        let src = t_start_u - d;
                        if (edges[src] as i32) < 0 {
                            edges[src] = shifted | ((cu - d) as u32);
                        }
                    }
                    for d in 1..=t_right {
                        let src = t_start_u + d;
                        if (edges[src] as i32) < 0 {
                            edges[src] = shifted | ((cu + d) as u32);
                        }
                    }
                }
                let clen = lens[ccode];
                if clen <= INLINE as u8 {
                    out[at..at + INLINE].copy_from_slice(&snap.inline16[ccode]);
                    at += usize::from(clen);
                } else if clen == TUNNEL {
                    let (out_pos, byte_len) = tunnels[ccode - tunnel_base];
                    let src = out_pos as usize;
                    let take = (byte_len as usize).min(n - at);
                    out.copy_within(src..src + take, at);
                    at += take;
                } else {
                    let w = ccode - 256;
                    let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
                    let take = word.len().min(n - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    at += take;
                }
                j = cnext;
                if j == mark_idx || at >= n { break 'outer; }
                break;
            }

            let lim_l = t_left.min(cu).min(cnext);
            let mut nl = 0usize;
            while nl < lim_l && edges[cu - nl - 1] == ce.wrapping_sub((nl + 1) as u32) {
                nl += 1;
            }
            let lim_r = t_right.min(m - 1 - cu).min(m - 1 - cnext);
            let mut nr = 0usize;
            while nr < lim_r && edges[cu + nr + 1] == ce.wrapping_add((nr + 1) as u32) {
                nr += 1;
            }
            if nl + nr == 0 {
                if t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                    let tid_code = lens.len() as u32;
                    lens.push(TUNNEL);
                    tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                    let shifted = tid_code << bits;
                    for d in 1..=t_left {
                        let src = t_start_u - d;
                        if (edges[src] as i32) < 0 {
                            edges[src] = shifted | ((cu - d) as u32);
                        }
                    }
                    for d in 1..=t_right {
                        let src = t_start_u + d;
                        if (edges[src] as i32) < 0 {
                            edges[src] = shifted | ((cu + d) as u32);
                        }
                    }
                }
                let clen = lens[ccode];
                if clen <= INLINE as u8 {
                    out[at..at + INLINE].copy_from_slice(&snap.inline16[ccode]);
                    at += usize::from(clen);
                } else {
                    let w = ccode - 256;
                    let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
                    let take = word.len().min(n - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    at += take;
                }
                j = cnext;
                if j == mark_idx || at >= n { break 'outer; }
                break;
            }
            if (nl < t_left || nr < t_right) && t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                let tid_code = lens.len() as u32;
                lens.push(TUNNEL);
                tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                let shifted = tid_code << bits;
                for d in (nl + 1)..=t_left {
                    let src = t_start_u - d;
                    if (edges[src] as i32) < 0 {
                        edges[src] = shifted | ((cu - d) as u32);
                    }
                }
                for d in (nr + 1)..=t_right {
                    let src = t_start_u + d;
                    if (edges[src] as i32) < 0 {
                        edges[src] = shifted | ((cu + d) as u32);
                    }
                }
            }
            t_left = nl;
            t_right = nr;
            t_steps += 1;
            let clen = lens[ccode];
            if clen <= INLINE as u8 {
                out[at..at + INLINE].copy_from_slice(&snap.inline16[ccode]);
                at += usize::from(clen);
            } else {
                let w = ccode - 256;
                let word = &snap.spelling[snap.offsets[w] as usize..snap.offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            j = cnext;
            if j == mark_idx || at >= n { break 'outer; }
        }
    }
    out.truncate(at.min(n));
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let mut snaps = Vec::new();
    let mut expecteds = Vec::new();
    for b in BLOCKS {
        let l = fs::read(dir.join(b.file)).expect("read");
        let s = build_snap(&l, b.p);
        let mut out = Vec::new();
        emit_plain(&s, &s.edges, &mut out);
        // Verify widebit_lazy matches emit_plain
        let mut ew = s.edges_wide.clone();
        let mut lens = Vec::new();
        let mut tuns = Vec::new();
        let mut out2 = Vec::new();
        emit_widebit_lazy(&s, &mut ew, &mut lens, &mut tuns, 4, &mut out2);
        assert_eq!(out2, out, "WideBit LazyTunnel mismatch on {}", b.file);
        snaps.push(s);
        expecteds.push(out);
    }

    let mut t_plain = Vec::new();
    let mut t_chk = Vec::new();
    let mut t_wb3 = Vec::new();
    let mut t_wb4 = Vec::new();
    let mut out = Vec::with_capacity(900_016);
    let mut edges_work = Vec::with_capacity(210_000);
    let mut lens_work = Vec::with_capacity(8192);
    let mut tuns_work = Vec::with_capacity(4096);
    let mut sink = 0usize;

    // Hot-cache (L2-warm, copying edges_work right before each emit so cache state is 100% IDENTICAL across all 4 arms!)
    for r in 0..17 {
        let (mut sp, mut sc, mut s3, mut s4) = (0.0, 0.0, 0.0, 0.0);
        for s in &snaps {
            edges_work.clear();
            edges_work.extend_from_slice(&s.edges);
            let t0 = Instant::now();
            emit_plain(s, &edges_work, &mut out);
            sp += t0.elapsed().as_secs_f64();
            sink ^= out[0] as usize;

            edges_work.clear();
            edges_work.extend_from_slice(&s.edges);
            let t0 = Instant::now();
            sink ^= emit_check_only(s, &edges_work, &mut out);
            sc += t0.elapsed().as_secs_f64();

            edges_work.clear();
            edges_work.extend_from_slice(&s.edges_wide);
            let t0 = Instant::now();
            emit_widebit_lazy(s, &mut edges_work, &mut lens_work, &mut tuns_work, 3, &mut out);
            s3 += t0.elapsed().as_secs_f64();
            sink ^= out[0] as usize;

            edges_work.clear();
            edges_work.extend_from_slice(&s.edges_wide);
            let t0 = Instant::now();
            emit_widebit_lazy(s, &mut edges_work, &mut lens_work, &mut tuns_work, 4, &mut out);
            s4 += t0.elapsed().as_secs_f64();
            sink ^= out[0] as usize;
        }
        if r >= 2 {
            t_plain.push(sp * 1000.0);
            t_chk.push(sc * 1000.0);
            t_wb3.push(s3 * 1000.0);
            t_wb4.push(s4 * 1000.0);
        }
    }

    let mp = median(t_plain);
    let mc = median(t_chk);
    let m3 = median(t_wb3);
    let m4 = median(t_wb4);
    println!("=== L2-WARM EMIT MICRO-ABLATION (6 BLOCKS SUMMED, IDENTICAL CACHE STATE, 15 ROUNDS, sink={}) ===", sink & 1);
    println!("  emit_plain (Open L=16):                 {:6.3} ms (1.000x)", mp);
    println!("  emit_check_only (2-neighbor check):     {:6.3} ms ({:.3}x, +{:.3} ms pure neighbor-check cost)", mc, mc / mp, mc - mp);
    println!("  emit_widebit_lazy (ms=3, ZERO w1 chk):  {:6.3} ms ({:.3}x, {:+.3} ms)", m3, m3 / mp, m3 - mp);
    println!("  emit_widebit_lazy (ms=4, ZERO w1 chk):  {:6.3} ms ({:.3}x, {:+.3} ms)", m4, m4 / mp, m4 - mp);
}
