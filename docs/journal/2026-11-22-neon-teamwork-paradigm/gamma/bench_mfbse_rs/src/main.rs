#![forbid(unsafe_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use std::fs;
use std::path::Path;
use std::time::Instant;

// ---------- Pure Safe-Rust SHA-256 (zero external crates) ----------
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        let vals = [a, b, c, d, e, f, g, hh];
        for i in 0..8 {
            h[i] = h[i].wrapping_add(vals[i]);
        }
    }
    let mut s = String::with_capacity(64);
    for x in h {
        s.push_str(&format!("{x:08x}"));
    }
    s
}

// ---------- Build genuine mibz2::runs::Runs from raw BWT column L ----------
fn runs_from_column(col: &[u8], orig_ptr: u32) -> Runs {
    let mut present = [false; 256];
    for &b in col {
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
    let mut mtf: Vec<u8> = seq_to_unseq[..n_in_use].to_vec();
    let mut syms: Vec<u16> = Vec::with_capacity(col.len() / 2);
    let mut zero_run: usize = 0;

    let flush_zeros = |zr: &mut usize, syms: &mut Vec<u16>| {
        let mut z = *zr;
        while z > 0 {
            if (z & 1) == 1 {
                syms.push(0); // RUNA
                z = (z - 1) >> 1;
            } else {
                syms.push(1); // RUNB
                z = (z - 2) >> 1;
            }
        }
        *zr = 0;
    };

    for &b in col {
        let idx = mtf.iter().position(|&x| x == b).unwrap();
        if idx == 0 {
            zero_run += 1;
        } else {
            flush_zeros(&mut zero_run, &mut syms);
            mtf.remove(idx);
            mtf.insert(0, b);
            syms.push((idx + 1) as u16);
        }
    }
    flush_zeros(&mut zero_run, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);

    let block_syms = Symbols {
        crc: 0,
        orig_ptr,
        seq_to_unseq,
        syms,
        eob,
        max_len: col.len().max(900_000),
    };
    let mut runs = Runs::default();
    let decoded_n = front_scalar(&block_syms, &mut runs).expect("front_scalar failed");
    assert_eq!(decoded_n, col.len());

    // Verify runs.records() expands bit-for-bit to col
    let mut check_pos = 0usize;
    for &rec in runs.records() {
        let byte = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        assert!(col[check_pos..check_pos + len].iter().all(|&x| x == byte));
        check_pos += len;
    }
    assert_eq!(check_pos, col.len());
    runs
}

// ---------- Packed 8-byte MF-BSE Engine (safe Rust, zero unsafe) ----------
// Packing for m <= 1_048_575 (< 2^20):
//   Edge (u64): u:20 | (v:20 << 20) | (w:24 << 40)
//   Req  (u64): u:20 | (src_k:12 << 20) | (cid:20 << 32)
//   ChainNode (u64): x:20 | (off:24 << 20) | (meta_pad)
//   Reply (u64): cid:32 | (ppos:32 << 32)
#[inline(always)]
fn pack_edge(u: u32, v: u32, w: u32) -> u64 {
    (u as u64) | ((v as u64) << 20) | ((w as u64) << 40)
}
#[inline(always)]
fn unpack_edge(e: u64) -> (u32, u32, u32) {
    (
        (e & 0xF_FFFF) as u32,
        ((e >> 20) & 0xF_FFFF) as u32,
        (e >> 40) as u32,
    )
}
#[inline(always)]
fn pack_req(u: u32, src_k: u32, cid: u32) -> u64 {
    (u as u64) | ((src_k as u64) << 20) | ((cid as u64) << 32)
}
#[inline(always)]
fn unpack_req(r: u64) -> (u32, u32, u32) {
    (
        (r & 0xF_FFFF) as u32,
        ((r >> 20) & 0xFFF) as u32,
        (r >> 32) as u32,
    )
}

#[derive(Default)]
struct MfbseScratch {
    fwd_pool: Vec<u64>,
    fwd_len: Vec<u32>,
    req_bkt: Vec<Vec<u64>>,
    node_x: Vec<u32>,
    node_off: Vec<u32>,
    meta_start: Vec<u32>,
    meta_len: Vec<u32>,
    reply_pool: Vec<u64>,
    reply_len: Vec<u32>,
    in_u: Vec<u32>,
    in_w: Vec<u32>,
    out_v: Vec<u32>,
    out_w: Vec<u32>,
    pos: Vec<u32>,
}

struct MfbseMetrics {
    shortcuts: usize,
    edges_read: usize,
    scratch_bytes: usize,
    cycle_len: u32,
}

impl MfbseScratch {
    fn prepare(&mut self, m: usize, k_bkts: usize) -> (usize, usize) {
        let s = (m + k_bkts - 1) / k_bkts;
        let cap_e = 2 * s + 2;
        let fwd_tot = k_bkts * cap_e;
        if self.fwd_pool.len() < fwd_tot {
            self.fwd_pool.resize(fwd_tot, 0);
        }
        self.fwd_len.clear();
        self.fwd_len.resize(k_bkts, 0);

        if self.req_bkt.len() < k_bkts + 1 {
            self.req_bkt.resize_with(k_bkts + 1, Vec::new);
        }
        for k in 0..=k_bkts {
            self.req_bkt[k].clear();
            let want = if k == k_bkts { 16 } else { s + 16 };
            let cap = self.req_bkt[k].capacity();
            if cap < want {
                self.req_bkt[k].reserve(want - cap);
            }
        }
        let ks = k_bkts * s;
        if self.node_x.len() < ks {
            self.node_x.resize(ks, 0);
            self.node_off.resize(ks, 0);
            self.meta_start.resize(ks, 0);
            self.meta_len.resize(ks, 0);
            self.reply_pool.resize(ks, 0);
        }
        self.reply_len.clear();
        self.reply_len.resize(k_bkts, 0);
        if self.in_u.len() < s {
            self.in_u.resize(s, 0);
            self.in_w.resize(s, 0);
            self.out_v.resize(s, 0);
            self.out_w.resize(s, 0);
        }
        if self.pos.len() < m {
            self.pos.resize(m, 0);
        }
        (s, cap_e)
    }

    fn bytes_allocated(&self) -> usize {
        let req_b: usize = self.req_bkt.iter().map(|v| v.capacity() * 8).sum();
        self.fwd_pool.capacity() * 8
            + self.fwd_len.capacity() * 4
            + req_b
            + self.node_x.capacity() * 4
            + self.node_off.capacity() * 4
            + self.meta_start.capacity() * 4
            + self.meta_len.capacity() * 4
            + self.reply_pool.capacity() * 8
            + self.reply_len.capacity() * 4
            + (self.in_u.capacity() + self.in_w.capacity() + self.out_v.capacity() + self.out_w.capacity()) * 4
            + self.pos.capacity() * 4
    }

    #[inline(always)]
    fn push_edge(&mut self, u: u32, v: u32, w: u32, p_idx: u32, s: u32, k_bkts: u32, cap_e: usize) {
        let eu = if u == p_idx { k_bkts } else { u / s };
        let ev = if v == p_idx { k_bkts } else { v / s };
        let mut t = eu.min(ev);
        if t == k_bkts {
            t = k_bkts - 1;
        }
        let ti = t as usize;
        let idx = self.fwd_len[ti] as usize;
        self.fwd_pool[ti * cap_e + idx] = pack_edge(u, v, w);
        self.fwd_len[ti] = (idx + 1) as u32;
    }

    fn solve(&mut self, m: usize, p_idx: u32, k_bkts: usize, s: usize, cap_e: usize) -> MfbseMetrics {
        let su = s as u32;
        let ku = k_bkts as u32;
        let mu = m as u32;
        let mut shortcuts = 0usize;
        let mut edges_read = 0usize;
        let mut p_cycle_len = 0u32;

        // Forward bucket-order Schur elimination
        for k in 0..k_bkts {
            let lo = (k * s) as u32;
            let hi = (lo + su).min(mu);
            let sz = hi.saturating_sub(lo) as usize;
            let ne = self.fwd_len[k] as usize;
            edges_read += ne;

            for i in 0..sz {
                self.in_u[i] = u32::MAX;
            }
            let base_e = k * cap_e;
            for i in 0..ne {
                let (u, v, w) = unpack_edge(self.fwd_pool[base_e + i]);
                if u == p_idx && v == p_idx {
                    p_cycle_len = w;
                    continue;
                }
                if u >= lo && u < hi && u != p_idx {
                    let ui = (u - lo) as usize;
                    self.out_v[ui] = v;
                    self.out_w[ui] = w;
                }
                if v >= lo && v < hi && v != p_idx {
                    let vi = (v - lo) as usize;
                    self.in_u[vi] = u;
                    self.in_w[vi] = w;
                }
            }

            let base_n = k * s;
            let mut n_nodes = 0u32;
            let mut n_chains = 0u32;

            for idx in 0..sz {
                let x = lo + idx as u32;
                if x == p_idx {
                    continue;
                }
                let u = self.in_u[idx];
                if u == u32::MAX {
                    continue;
                }
                if !(u >= lo && u < hi && u != p_idx) {
                    let mut cur_x = x;
                    let mut acc_w = self.in_w[idx];
                    let c_start = n_nodes;
                    let w_end;
                    loop {
                        let cidx = (cur_x - lo) as usize;
                        self.node_x[base_n + n_nodes as usize] = cur_x;
                        self.node_off[base_n + n_nodes as usize] = acc_w;
                        n_nodes += 1;
                        let nxt = self.out_v[cidx];
                        acc_w += self.out_w[cidx];
                        if nxt >= lo && nxt < hi && nxt != p_idx {
                            cur_x = nxt;
                        } else {
                            w_end = nxt;
                            break;
                        }
                    }
                    let cid = n_chains;
                    n_chains += 1;
                    self.meta_start[base_n + cid as usize] = c_start;
                    self.meta_len[base_n + cid as usize] = n_nodes - c_start;

                    let bu = if u == p_idx { k_bkts } else { (u / su) as usize };
                    self.req_bkt[bu].push(pack_req(u, k as u32, cid));

                    shortcuts += 1;
                    if u == p_idx && w_end == p_idx {
                        p_cycle_len = acc_w;
                    } else {
                        let eu = if u == p_idx { ku } else { u / su };
                        let ew = if w_end == p_idx { ku } else { w_end / su };
                        let t = eu.min(ew) as usize;
                        let pos_t = self.fwd_len[t] as usize;
                        self.fwd_pool[t * cap_e + pos_t] = pack_edge(u, w_end, acc_w);
                        self.fwd_len[t] = (pos_t + 1) as u32;
                    }
                }
            }
        }

        // Backward zero-miss req/reply substitution
        self.pos[..m].fill(u32::MAX);
        self.pos[p_idx as usize] = 0;
        for &rq in &self.req_bkt[k_bkts] {
            let (_, sk, cid) = unpack_req(rq);
            let ski = sk as usize;
            let ri = self.reply_len[ski] as usize;
            self.reply_pool[ski * s + ri] = cid as u64; // ppos = 0
            self.reply_len[ski] = (ri + 1) as u32;
        }

        for k in (0..k_bkts).rev() {
            let lo = (k * s) as u32;
            let hi = (lo + su).min(mu);
            let sz = hi.saturating_sub(lo) as usize;
            let base_n = k * s;

            // Reuse self.in_u[0..sz] as L1-resident local_pos
            self.in_u[..sz].fill(u32::MAX);
            if p_idx >= lo && p_idx < hi {
                self.in_u[(p_idx - lo) as usize] = 0;
            }
            let nr = self.reply_len[k] as usize;
            for i in 0..nr {
                let rep = self.reply_pool[base_n + i];
                let cid = (rep & 0xFFFF_FFFF) as usize;
                let ppos = (rep >> 32) as u32;
                if ppos == u32::MAX {
                    continue;
                }
                let st = self.meta_start[base_n + cid] as usize;
                let ln = self.meta_len[base_n + cid] as usize;
                for j in 0..ln {
                    let x = self.node_x[base_n + st + j];
                    let off = self.node_off[base_n + st + j];
                    self.in_u[(x - lo) as usize] = ppos + off;
                }
            }
            self.pos[lo as usize..hi as usize].copy_from_slice(&self.in_u[..sz]);

            for &rq in &self.req_bkt[k] {
                let (u, sk, cid) = unpack_req(rq);
                let upos = self.in_u[(u - lo) as usize];
                let ski = sk as usize;
                let ri = self.reply_len[ski] as usize;
                self.reply_pool[ski * s + ri] = (cid as u64) | ((upos as u64) << 32);
                self.reply_len[ski] = (ri + 1) as u32;
            }
        }

        MfbseMetrics {
            shortcuts,
            edges_read,
            scratch_bytes: self.bytes_allocated(),
            cycle_len: p_cycle_len,
        }
    }
}

// ---------- Arm 2: Standalone MF-BSE directly from I0 (ZERO Rauzy-Veech!) ----------
#[derive(Default)]
struct StandaloneMfbse {
    mf: MfbseScratch,
    col_l: Vec<u8>,
}

impl StandaloneMfbse {
    fn invert_runs(&mut self, runs: &Runs, n: usize, p: u32, k_bkts: usize, out: &mut Vec<u8>) -> MfbseMetrics {
        let recs = runs.records();
        let mut counts = [0u32; 256];
        for &rec in recs {
            counts[(rec >> 8) as usize] += u32::from(rec & 255);
        }
        let mut cur = [0u32; 256];
        let mut acc = 0u32;
        for c in 0..256 {
            cur[c] = acc;
            acc += counts[c];
        }
        let (s, cap_e) = self.mf.prepare(n, k_bkts);
        if self.col_l.len() < n {
            self.col_l.resize(n, 0);
        }
        let su = s as u32;
        let ku = k_bkts as u32;
        let mut row = 0u32;
        for &rec in recs {
            let sym = (rec >> 8) as u8;
            let len = u32::from(rec & 255);
            let mut dst = cur[sym as usize];
            cur[sym as usize] = dst + len;
            self.col_l[row as usize..(row + len) as usize].fill(sym);
            for _ in 0..len {
                self.mf.push_edge(row, dst, 1, p, su, ku, cap_e);
                row += 1;
                dst += 1;
            }
        }
        let mut met = self.mf.solve(n, p, k_bkts, s, cap_e);
        met.scratch_bytes += self.col_l.capacity();

        // Emit output bytes directly from pos[0..n)
        out.resize(n, 0);
        let cl = met.cycle_len as usize;
        if cl == n {
            for i in 0..n {
                let rank = self.mf.pos[i] as usize;
                out[n - 1 - rank] = self.col_l[i];
            }
        } else {
            let mut cyc = vec![0u8; cl];
            for i in 0..n {
                let rank = self.mf.pos[i];
                if rank != u32::MAX {
                    cyc[rank as usize] = self.col_l[i];
                }
            }
            for t in 0..n {
                out[n - 1 - t] = cyc[t % cl];
            }
        }
        met
    }
}

// ---------- Arm 1: Hybrid Open (LIMIT=16 thick-strip elimination) + MF-BSE ----------
#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}
#[derive(Clone, Copy)]
struct Tile {
    end: u32,
    delta: u32,
    symbol: u16, // 0..255 byte, 256 terminal (retained)
}
#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32, // 0..255 single byte, >=256 word index (code - 256)
}

const LIMIT: u32 = 16;
const TERMINAL: u16 = 256;
const PAGE: u32 = 128;

#[derive(Default)]
struct HybridOpenMfbse {
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    thin_queues: Vec<Vec<u32>>,
    word_code: Vec<u32>,
    mf: MfbseScratch,
}

impl HybridOpenMfbse {
    fn trace_gap(&mut self, from: u32, to: u32, c: u8, k: &mut u32) {
        let mut x0 = from;
        while x0 < to {
            let mut x = x0;
            let mut width = to - x0;
            let start = self.spelling.len();
            let exit = loop {
                let mut ti = self.directory[(x / PAGE) as usize] as usize;
                while self.tiles[ti].end <= x {
                    ti += 1;
                }
                let t = self.tiles[ti];
                width = width.min(t.end - x);
                x = x.wrapping_add(t.delta);
                if t.symbol == TERMINAL {
                    break x;
                }
                self.spelling.push(t.symbol as u8);
            };
            let height = self.spelling.len() - start;
            let code = if height == 0 {
                u32::from(c)
            } else {
                // Reverse traced bytes so word is in FORWARD text order, with `c` (L[u]) at the end!
                self.spelling[start..].reverse();
                self.spelling.push(c);
                let idx = self.offsets.len() - 1;
                self.offsets.push(self.spelling.len() as u32);
                256 + idx as u32
            };
            self.bands.push(Band {
                end: *k + width,
                delta: exit.wrapping_sub(*k),
                code,
            });
            *k += width;
            x0 += width;
        }
    }

    fn invert_runs(&mut self, runs: &Runs, n: usize, p: u32, k_bkts: usize, out: &mut Vec<u8>) -> (usize, MfbseMetrics) {
        let recs = runs.records();
        if self.thin_queues.len() < 256 {
            self.thin_queues.resize_with(256, Vec::new);
        }
        for q in &mut self.thin_queues {
            q.clear();
        }
        self.strips.clear();
        let mut all = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m = 0u32;
        let mut mark = 0u32;

        // Pass 1: Maximal run scan -> thin queues (in retained rank order) + thick strips
        let mut pos = 0u32;
        let mut i = 0usize;
        while i < recs.len() {
            let rec = recs[i];
            let c = (rec >> 8) as u8;
            let mut w = u32::from(rec & 255);
            i += 1;
            while i < recs.len() && (recs[i] >> 8) as u8 == c {
                w += u32::from(recs[i] & 255);
                i += 1;
            }
            let a = c as usize;
            if w <= LIMIT {
                if p.wrapping_sub(pos) < w {
                    mark = m + (p - pos);
                }
                let q = &mut self.thin_queues[a];
                for r in m..m + w {
                    q.push(r);
                }
                m += w;
            } else if p.wrapping_sub(pos) >= w {
                self.strips.push(Strip {
                    from: pos,
                    to: all[a],
                    width: w,
                    byte: c,
                });
                thick[a] += 1;
            } else {
                // Split thick run around p
                if pos < p {
                    self.strips.push(Strip {
                        from: pos,
                        to: all[a],
                        width: p - pos,
                        byte: c,
                    });
                    thick[a] += 1;
                }
                mark = m;
                self.thin_queues[a].push(m);
                m += 1;
                if p + 1 < pos + w {
                    self.strips.push(Strip {
                        from: p + 1,
                        to: all[a] + (p - pos) + 1,
                        width: (pos + w) - (p + 1),
                        byte: c,
                    });
                    thick[a] += 1;
                }
            }
            all[a] += w;
            pos += w;
        }
        assert_eq!(pos as usize, n);

        // Build tile directory + image-ordered strips
        let mut first = [0u32; 256];
        let mut write = [0u32; 256];
        let (mut fa, mut ft) = (0u32, 0u32);
        for a in 0..256 {
            first[a] = fa;
            fa += all[a];
            write[a] = ft;
            ft += thick[a];
        }
        self.image_order.resize(self.strips.len(), 0);
        self.tiles.clear();
        let (mut last_x, mut ret_x) = (0u32, 0u32);
        for (si, st) in self.strips.iter_mut().enumerate() {
            let a = st.byte as usize;
            st.to += first[a];
            self.image_order[write[a] as usize] = si as u32;
            write[a] += 1;
            if st.from > last_x {
                self.tiles.push(Tile {
                    end: st.from,
                    delta: ret_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
                ret_x += st.from - last_x;
            }
            last_x = st.from + st.width;
            self.tiles.push(Tile {
                end: last_x,
                delta: st.to.wrapping_sub(st.from),
                symbol: u16::from(st.byte),
            });
        }
        if last_x < n as u32 {
            self.tiles.push(Tile {
                end: n as u32,
                delta: ret_x.wrapping_sub(last_x),
                symbol: TERMINAL,
            });
        }
        self.directory.clear();
        let mut ti = 0usize;
        let mut x_pg = 0u32;
        while x_pg < n as u32 {
            while self.tiles[ti].end <= x_pg {
                ti += 1;
            }
            self.directory.push(ti as u32);
            x_pg += PAGE;
        }

        // Trace bands across gaps between thick strips in F-order
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let mut cursors = [0usize; 256];
        let (mut k_cur, mut si) = (0u32, 0usize);
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let mut x = first[a];
            while si < write[a] as usize {
                let s = self.strips[self.image_order[si] as usize];
                self.trace_gap(x, s.to, a as u8, &mut k_cur);
                x = s.to + s.width;
                si += 1;
            }
            self.trace_gap(x, first[a] + all[a], a as u8, &mut k_cur);
        }
        assert_eq!(k_cur, m);

        // Emit m retained edges (u -> exit, word_len) DIRECTLY into MF-BSE buckets!
        let m_usize = m as usize;
        let (s, cap_e) = self.mf.prepare(m_usize, k_bkts);
        if self.word_code.len() < m_usize {
            self.word_code.resize(m_usize, 0);
        }
        let su = s as u32;
        let ku = k_bkts as u32;
        let mut kb = 0u32;
        for a in 0..256 {
            let b_end = if a + 1 < 256 { cursors[a + 1] } else { self.bands.len() };
            let q = &self.thin_queues[a];
            let mut qi = 0usize;
            for b in &self.bands[cursors[a]..b_end] {
                let span = (b.end - kb) as usize;
                let code = b.code;
                let wlen = if code < 256 {
                    1u32
                } else {
                    let wi = (code - 256) as usize;
                    self.offsets[wi + 1] - self.offsets[wi]
                };
                let dst_base = kb.wrapping_add(b.delta);
                for off in 0..span {
                    let u = q[qi + off];
                    let v = dst_base + off as u32;
                    self.word_code[u as usize] = code;
                    self.mf.push_edge(u, v, wlen, mark, su, ku, cap_e);
                }
                qi += span;
                kb = b.end;
            }
        }

        // Solve exact global output positions pos[0..m) via MF-BSE (Q=0 outside L1!)
        let mut met = self.mf.solve(m_usize, mark, k_bkts, s, cap_e);
        let q_bytes: usize = self.thin_queues.iter().map(|v| v.capacity() * 4).sum();
        met.scratch_bytes += self.strips.capacity() * std::mem::size_of::<Strip>()
            + self.image_order.capacity() * 4
            + self.tiles.capacity() * std::mem::size_of::<Tile>()
            + self.directory.capacity() * 4
            + self.bands.capacity() * std::mem::size_of::<Band>()
            + self.spelling.capacity()
            + self.offsets.capacity() * 4
            + q_bytes
            + self.word_code.capacity() * 4;

        // Emit bytes using exact pos[0..m) (forward write-only scatter, zero dependent read chain!)
        out.resize(n, 0);
        let cl = met.cycle_len as usize;
        if cl == n {
            for u in 0..m_usize {
                let p_off = self.mf.pos[u] as usize;
                let code = self.word_code[u];
                if code < 256 {
                    out[n - p_off - 1] = code as u8;
                } else {
                    let wi = (code - 256) as usize;
                    let s0 = self.offsets[wi] as usize;
                    let s1 = self.offsets[wi + 1] as usize;
                    let wlen = s1 - s0;
                    out[n - p_off - wlen..n - p_off].copy_from_slice(&self.spelling[s0..s1]);
                }
            }
        } else {
            let mut cyc = vec![0u8; cl];
            for u in 0..m_usize {
                let p_off = self.mf.pos[u];
                if p_off == u32::MAX {
                    continue;
                }
                let po = p_off as usize;
                let code = self.word_code[u];
                if code < 256 {
                    cyc[cl - po - 1] = code as u8;
                } else {
                    let wi = (code - 256) as usize;
                    let s0 = self.offsets[wi] as usize;
                    let s1 = self.offsets[wi + 1] as usize;
                    let wlen = s1 - s0;
                    cyc[cl - po - wlen..cl - po].copy_from_slice(&self.spelling[s0..s1]);
                }
            }
            let mut pos_out = n;
            while pos_out >= cl {
                pos_out -= cl;
                out[pos_out..pos_out + cl].copy_from_slice(&cyc);
            }
            if pos_out > 0 {
                out[..pos_out].copy_from_slice(&cyc[cl - pos_out..]);
            }
        }

        (m_usize, met)
    }
}

struct BlockSpec {
    name: &'static str,
    n: usize,
    p: u32,
    r: usize,
    sha256: &'static str,
}

const BLOCKS: &[BlockSpec] = &[
    BlockSpec {
        name: "block-0.L",
        n: 899931,
        p: 264045,
        r: 51184,
        sha256: "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67",
    },
    BlockSpec {
        name: "block-2.L",
        n: 900000,
        p: 320807,
        r: 54532,
        sha256: "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174",
    },
    BlockSpec {
        name: "block-85.L",
        n: 899935,
        p: 325088,
        r: 79375,
        sha256: "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8",
    },
    BlockSpec {
        name: "block-143.L",
        n: 899869,
        p: 802868,
        r: 108335,
        sha256: "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2",
    },
    BlockSpec {
        name: "block-210.L",
        n: 899959,
        p: 791429,
        r: 128732,
        sha256: "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd",
    },
    BlockSpec {
        name: "block-207.L",
        n: 899998,
        p: 205180,
        r: 87111,
        sha256: "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099",
    },
];

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[vals.len() / 2]
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let kernels = Kernels::detect();
    let mut loaded: Vec<(&BlockSpec, Runs)> = Vec::new();

    println!("=== 1. Verifying SHA-256 & Exactness against Master mibz2::bwt::OpenCells (0.5.11) ===");
    let mut open_ref = OpenCells::default();
    let mut hybrid = HybridOpenMfbse::default();
    let mut standalone = StandaloneMfbse::default();
    let mut out_open = Vec::new();
    let mut out_hyb = Vec::new();
    let mut out_std = Vec::new();

    for spec in BLOCKS {
        let data = fs::read(dir.join(spec.name)).expect("read block");
        assert_eq!(data.len(), spec.n);
        let h = sha256_hex(&data);
        assert_eq!(h, spec.sha256, "SHA-256 mismatch on {}", spec.name);
        let runs = runs_from_column(&data, spec.p);

        assert!(open_ref.invert_runs(&runs, spec.n, spec.p, &mut out_open, kernels));
        let (m16, hyb_met) = hybrid.invert_runs(&runs, spec.n, spec.p, 64, &mut out_hyb);
        assert_eq!(out_hyb, out_open, "Hybrid-Open-MF-BSE byte mismatch on {}!", spec.name);

        let std_met = standalone.invert_runs(&runs, spec.n, spec.p, 256, &mut out_std);
        assert_eq!(out_std, out_open, "Standalone-MF-BSE byte mismatch on {}!", spec.name);

        println!(
            "  {:<12} SHA256=OK | n={:6} r={:6} m16={:6} | Hyb: exact OK, Sc/m={:.3}, M={:.2}MiB | Std: exact OK, Sc/n={:.3}, M={:.2}MiB",
            spec.name,
            spec.n,
            spec.r,
            m16,
            hyb_met.shortcuts as f64 / m16 as f64,
            hyb_met.scratch_bytes as f64 / (1024.0 * 1024.0),
            std_met.shortcuts as f64 / spec.n as f64,
            std_met.scratch_bytes as f64 / (1024.0 * 1024.0),
        );
        loaded.push((spec, runs));
    }

    // ---------- 2. Single-Thread Paired Interleaved A/A & A/B Benchmark ----------
    let rounds_1t = 11usize;
    println!("\n=== 2. Single-Thread Paired Interleaved Benchmark (median of {rounds_1t} rounds, same binary) ===");
    println!(
        "{:<12} {:>9} {:>9} {:>7} | {:>9} {:>7} | {:>9} {:>7}",
        "Block", "Open(A)", "Open(A')", "A'/A", "HybMFBSE", "Hyb/Op", "StdMFBSE", "Std/Op"
    );
    println!("{}", "-".repeat(86));

    let mut open_a = OpenCells::default();
    let mut open_b = OpenCells::default();
    let mut hyb_k = HybridOpenMfbse::default();
    let mut std_k = StandaloneMfbse::default();
    let mut obuf_a = Vec::with_capacity(900_032);
    let mut obuf_b = Vec::with_capacity(900_032);
    let mut obuf_h = Vec::with_capacity(900_032);
    let mut obuf_s = Vec::with_capacity(900_032);

    // Warmup all kernels on all blocks
    for (spec, runs) in &loaded {
        open_a.invert_runs(runs, spec.n, spec.p, &mut obuf_a, kernels);
        open_b.invert_runs(runs, spec.n, spec.p, &mut obuf_b, kernels);
        hyb_k.invert_runs(runs, spec.n, spec.p, 64, &mut obuf_h);
        std_k.invert_runs(runs, spec.n, spec.p, 256, &mut obuf_s);
    }

    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_h = 0.0;
    let mut sum_s = 0.0;

    for (spec, runs) in &loaded {
        let mut ta = Vec::with_capacity(rounds_1t);
        let mut tb = Vec::with_capacity(rounds_1t);
        let mut th = Vec::with_capacity(rounds_1t);
        let mut ts = Vec::with_capacity(rounds_1t);
        for r in 0..rounds_1t {
            if r % 2 == 0 {
                let t0 = Instant::now();
                open_a.invert_runs(runs, spec.n, spec.p, &mut obuf_a, kernels);
                ta.push(t0.elapsed().as_secs_f64() * 1000.0);

                let t1 = Instant::now();
                open_b.invert_runs(runs, spec.n, spec.p, &mut obuf_b, kernels);
                tb.push(t1.elapsed().as_secs_f64() * 1000.0);

                let t2 = Instant::now();
                hyb_k.invert_runs(runs, spec.n, spec.p, 64, &mut obuf_h);
                th.push(t2.elapsed().as_secs_f64() * 1000.0);

                let t3 = Instant::now();
                std_k.invert_runs(runs, spec.n, spec.p, 256, &mut obuf_s);
                ts.push(t3.elapsed().as_secs_f64() * 1000.0);
            } else {
                let t3 = Instant::now();
                std_k.invert_runs(runs, spec.n, spec.p, 256, &mut obuf_s);
                ts.push(t3.elapsed().as_secs_f64() * 1000.0);

                let t2 = Instant::now();
                hyb_k.invert_runs(runs, spec.n, spec.p, 64, &mut obuf_h);
                th.push(t2.elapsed().as_secs_f64() * 1000.0);

                let t1 = Instant::now();
                open_b.invert_runs(runs, spec.n, spec.p, &mut obuf_b, kernels);
                tb.push(t1.elapsed().as_secs_f64() * 1000.0);

                let t0 = Instant::now();
                open_a.invert_runs(runs, spec.n, spec.p, &mut obuf_a, kernels);
                ta.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let ma = median(&mut ta);
        let mb = median(&mut tb);
        let mh = median(&mut th);
        let ms = median(&mut ts);
        sum_a += ma;
        sum_b += mb;
        sum_h += mh;
        sum_s += ms;
        println!(
            "{:<12} {:7.2}ms {:7.2}ms {:6.3}x | {:7.2}ms {:6.2}x | {:7.2}ms {:6.2}x",
            spec.name,
            ma,
            mb,
            mb / ma,
            mh,
            mh / ma,
            ms,
            ms / ma
        );
    }
    println!("{}", "-".repeat(86));
    println!(
        "{:<12} {:7.2}ms {:7.2}ms {:6.3}x | {:7.2}ms {:6.2}x | {:7.2}ms {:6.2}x",
        "6-block sum",
        sum_a,
        sum_b,
        sum_b / sum_a,
        sum_h,
        sum_h / sum_a,
        sum_s,
        sum_s / sum_a
    );

    // ---------- 3. 4-Worker Concurrent Interleaved Benchmark (shared L2/L3 contention!) ----------
    let waves_4t = 7usize;
    let iters_per_worker = 2usize; // each of 4 workers decodes all 6 blocks x 2 = 48 block decodes per wave
    println!("\n=== 3. 4-Worker Concurrent Benchmark ({waves_4t} interleaved waves, 48 block decodes/wave) ===");

    let run_4w_open = |loaded: &[(&BlockSpec, Runs)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|sc| {
            for wid in 0..4usize {
                sc.spawn(move || {
                    let mut k_open = OpenCells::default();
                    let mut out = Vec::with_capacity(900_032);
                    for _ in 0..iters_per_worker {
                        for b in 0..loaded.len() {
                            let (spec, runs) = &loaded[(b + wid) % loaded.len()];
                            k_open.invert_runs(runs, spec.n, spec.p, &mut out, kernels);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };

    let run_4w_hyb = |loaded: &[(&BlockSpec, Runs)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|sc| {
            for wid in 0..4usize {
                sc.spawn(move || {
                    let mut k_hyb = HybridOpenMfbse::default();
                    let mut out = Vec::with_capacity(900_032);
                    for _ in 0..iters_per_worker {
                        for b in 0..loaded.len() {
                            let (spec, runs) = &loaded[(b + wid) % loaded.len()];
                            k_hyb.invert_runs(runs, spec.n, spec.p, 64, &mut out);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };

    let run_4w_std = |loaded: &[(&BlockSpec, Runs)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|sc| {
            for wid in 0..4usize {
                sc.spawn(move || {
                    let mut k_std = StandaloneMfbse::default();
                    let mut out = Vec::with_capacity(900_032);
                    for _ in 0..iters_per_worker {
                        for b in 0..loaded.len() {
                            let (spec, runs) = &loaded[(b + wid) % loaded.len()];
                            k_std.invert_runs(runs, spec.n, spec.p, 256, &mut out);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };

    // Warmup
    let _ = run_4w_open(&loaded);
    let _ = run_4w_hyb(&loaded);

    let mut w_oa = Vec::with_capacity(waves_4t);
    let mut w_ob = Vec::with_capacity(waves_4t);
    let mut w_hy = Vec::with_capacity(waves_4t);
    let mut w_st = Vec::with_capacity(waves_4t);

    for w in 0..waves_4t {
        if w % 2 == 0 {
            w_oa.push(run_4w_open(&loaded));
            w_ob.push(run_4w_open(&loaded));
            w_hy.push(run_4w_hyb(&loaded));
            w_st.push(run_4w_std(&loaded));
        } else {
            w_st.push(run_4w_std(&loaded));
            w_hy.push(run_4w_hyb(&loaded));
            w_ob.push(run_4w_open(&loaded));
            w_oa.push(run_4w_open(&loaded));
        }
    }
    let moa = median(&mut w_oa);
    let mob = median(&mut w_ob);
    let mhy = median(&mut w_hy);
    let mst = median(&mut w_st);
    println!("  OpenCells (A)        : {moa:7.2} ms  (1.000x)");
    println!("  OpenCells (A' ctrl)  : {mob:7.2} ms  ({:.3}x A/A control)", mob / moa);
    println!("  Hybrid-Open-MF-BSE   : {mhy:7.2} ms  ({:.2}x vs OpenCells)", mhy / moa);
    println!("  Standalone-MF-BSE    : {mst:7.2} ms  ({:.2}x vs OpenCells)", mst / moa);
}
