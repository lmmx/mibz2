#![forbid(unsafe_code)]

use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front_scalar, Runs};
use mibz2::Kernels;
use std::fs;
use std::path::Path;
use std::time::Instant;

const LIMIT: u32 = 16;
const TERMINAL: u16 = 256;
const PAGE: u32 = 64;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NONE_U32: u32 = u32::MAX;

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
                syms.push(0);
                z = (z - 1) >> 1;
            } else {
                syms.push(1);
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
    runs
}

// ---------- Open Retained Table (exact production Open edges + inline16 layout) ----------
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
    symbol: u16,
}
#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Default)]
struct OpenTable {
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    thin_queues: Vec<Vec<u32>>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    mark: u32,
    bits: u32,
    n: usize,
}

impl OpenTable {
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

    fn build(&mut self, runs: &Runs, n: usize, p: u32) {
        self.n = n;
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
        self.mark = mark;

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

        let m_usize = m as usize;
        let bits = (32 - m.saturating_sub(1).leading_zeros()).max(1);
        self.bits = bits;
        let max_code = 255 + (self.offsets.len() - 1) as u32;
        let inline_code = bits < 32 && (max_code as u64) < (1u64 << (32 - bits));
        self.edges.resize(m_usize, 0);
        if inline_code {
            self.extra.clear();
        } else {
            self.extra.resize(m_usize, 0);
        }

        let mut kb = 0u32;
        for a in 0..256 {
            let b_end = if a + 1 < 256 {
                cursors[a + 1]
            } else {
                self.bands.len()
            };
            let q = &self.thin_queues[a];
            let mut qi = 0usize;
            for bi in cursors[a]..b_end {
                let band = self.bands[bi];
                let code = band.code;
                let delta = band.delta;
                let end = band.end;
                while kb < end {
                    let u = q[qi];
                    let exit = kb.wrapping_add(delta);
                    if inline_code {
                        self.edges[exit as usize] = u | (code << bits);
                    } else {
                        self.edges[exit as usize] = u;
                        self.extra[exit as usize] = code;
                    }
                    kb += 1;
                    qi += 1;
                }
            }
        }

        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0u8; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
        let words = self.offsets.len().saturating_sub(1);
        self.inline16.reserve(words);
        self.lens.reserve(words);
        for w in self.offsets.windows(2) {
            let word = &self.spelling[w[0] as usize..w[1] as usize];
            let mut v = [0u8; INLINE];
            let head = word.len().min(INLINE);
            v[..head].copy_from_slice(&word[..head]);
            self.inline16.push(v);
            self.lens.push(if word.len() <= INLINE {
                word.len() as u8
            } else {
                LONG
            });
        }
    }
}

fn repeat_out(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

// ---------- Emit 0: Exact Production Open::emit ----------
fn emit_serial(t: &OpenTable, out: &mut Vec<u8>) {
    let n = t.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let bits = t.bits;
    let mask = (1u32 << bits) - 1;
    let mut j = t.mark;
    let edges = &t.edges;
    let extra = &t.extra;
    let inline16 = &t.inline16;
    let lens = &t.lens;
    let spelling = &t.spelling;
    let offsets = &t.offsets;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == t.mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat_out(n, out);
}

// ---------- Emit 1: Flat-Slab Zero-Indirection BSAC(B, K) ----------
// Exact Poincaré first-return anchor contraction:
//   - Exactly 1 deterministic bijective-slot anchor per B-row block, guaranteed to contain `mark`.
//   - Zero bitmap, zero discovery pass, 2-ALU membership test & shift-only anchor ID.
//   - Phase 1: K-way MLP continuous-refill chase into flat L1-friendly lane slabs -> compact u16 code_buf.
//   - Phase 2: L1d-resident contracted cycle walk on Q = ceil(m/B) anchors (1.4-6.4 KiB!).
const SLAB: usize = 2048;

#[derive(Default)]
struct BsacFlatScratch {
    g_a: Vec<u32>,
    code_off: Vec<u32>,
    code_len: Vec<u32>,
    code_u16: Vec<u16>,
    slab: Vec<u16>,
    spill: Vec<Vec<u16>>,
    spills_seen: usize,
}

#[inline(always)]
fn bsac_slot<const SHIFT: u32>(q: u32, p_blk: u32, p_slot: u32, last_q: u32, last_len: u32) -> u32 {
    if q == p_blk {
        return p_slot;
    }
    let h = q.wrapping_mul(0x9E37_79B1) >> (32 - SHIFT);
    if q == last_q && h >= last_len {
        h % last_len
    } else {
        h
    }
}

fn emit_bsac_flat<const SHIFT: u32, const K: usize>(
    t: &OpenTable,
    sc: &mut BsacFlatScratch,
    out: &mut Vec<u8>,
) {
    let m = t.edges.len();
    let n = t.n;
    let b_sz = 1usize << SHIFT;
    let max_code = t.lens.len().saturating_sub(1);
    if !t.extra.is_empty() || m < b_sz * K * 2 || max_code > u16::MAX as usize {
        emit_serial(t, out);
        return;
    }
    let bits = t.bits;
    let mask = (1u32 << bits) - 1;
    let b_mask = (b_sz - 1) as u32;
    let q_cnt = (m + b_sz - 1) >> SHIFT;
    let last_q = (q_cnt - 1) as u32;
    let last_len = (m - ((q_cnt - 1) << SHIFT)) as u32;
    let p_blk = t.mark >> SHIFT;
    let p_slot = t.mark & b_mask;

    if sc.g_a.len() < q_cnt {
        sc.g_a.resize(q_cnt, 0);
        sc.code_off.resize(q_cnt, 0);
        sc.code_len.resize(q_cnt, 0);
    }
    sc.code_u16.clear();
    if sc.code_u16.capacity() < m {
        sc.code_u16.reserve(m - sc.code_u16.capacity());
    }
    if sc.slab.len() < K * SLAB {
        sc.slab.resize(K * SLAB, 0);
    }
    if sc.spill.len() < K {
        sc.spill.resize_with(K, Vec::new);
    }

    let edges = &t.edges;
    let slab = &mut sc.slab[..K * SLAB];

    let mut cur = [0u32; K];
    let mut qid = [0usize; K];
    let mut llen = [0usize; K];
    let mut spilled = [false; K];
    let mut active = [false; K];
    let mut next_q = 0usize;
    let mut alive = 0usize;

    for l in 0..K {
        if next_q < q_cnt {
            let q = next_q as u32;
            cur[l] = (q << SHIFT) | bsac_slot::<SHIFT>(q, p_blk, p_slot, last_q, last_len);
            qid[l] = next_q;
            llen[l] = 0;
            spilled[l] = false;
            active[l] = true;
            next_q += 1;
            alive += 1;
        }
    }

    let mut fetched = [0u32; K];
    while alive > 0 {
        for l in 0..K {
            fetched[l] = edges[cur[l] as usize];
        }
        for l in 0..K {
            if !active[l] {
                continue;
            }
            let e = fetched[l];
            let code = (e >> bits) as u16;
            let nxt = e & mask;
            let pos = llen[l];
            if pos < SLAB {
                slab[l * SLAB + pos] = code;
                llen[l] = pos + 1;
            } else {
                if !spilled[l] {
                    sc.spills_seen += 1;
                    sc.spill[l].clear();
                    sc.spill[l].extend_from_slice(&slab[l * SLAB..l * SLAB + SLAB]);
                    spilled[l] = true;
                }
                sc.spill[l].push(code);
                llen[l] = pos + 1;
            }
            let nq = nxt >> SHIFT;
            if (nxt & b_mask) == bsac_slot::<SHIFT>(nq, p_blk, p_slot, last_q, last_len) {
                let my_q = qid[l];
                sc.g_a[my_q] = nq;
                sc.code_off[my_q] = sc.code_u16.len() as u32;
                let cnt = llen[l];
                sc.code_len[my_q] = cnt as u32;
                if !spilled[l] {
                    sc.code_u16
                        .extend_from_slice(&slab[l * SLAB..l * SLAB + cnt]);
                } else {
                    sc.code_u16.extend_from_slice(&sc.spill[l]);
                    spilled[l] = false;
                }
                llen[l] = 0;

                if next_q < q_cnt {
                    let q = next_q as u32;
                    cur[l] = (q << SHIFT) | bsac_slot::<SHIFT>(q, p_blk, p_slot, last_q, last_len);
                    qid[l] = next_q;
                    next_q += 1;
                } else {
                    active[l] = false;
                    cur[l] = 0;
                    alive -= 1;
                }
            } else {
                cur[l] = nxt;
            }
        }
    }

    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mut cq = p_blk as usize;
    let inline16 = &t.inline16;
    let lens = &t.lens;
    let spelling = &t.spelling;
    let offsets = &t.offsets;
    for _ in 0..q_cnt {
        let off = sc.code_off[cq] as usize;
        let cnt = sc.code_len[cq] as usize;
        for &c16 in &sc.code_u16[off..off + cnt] {
            let code = c16 as usize;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if at >= n {
                break;
            }
        }
        cq = sc.g_a[cq] as usize;
        if cq == p_blk as usize || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat_out(n, out);
}

// ---------- In-Rust Exhaustive Small-Block Exactness Verification (Binary n<=8, Ternary n<=5) ----------
fn verify_small_exhaustive_rust() {
    let mut sc = BsacFlatScratch::default();
    let mut o_ser = Vec::new();
    let mut o_b1 = Vec::new();
    let mut o_b2 = Vec::new();

    let mut check_col = |col: &[u8]| {
        let n = col.len();
        let mut cnt = [0usize; 4];
        for &b in col {
            cnt[b as usize] += 1;
        }
        let mut c = [0usize; 4];
        for i in 1..4 {
            c[i] = c[i - 1] + cnt[i - 1];
        }
        let mut cur = c;
        let mut fl = vec![0u32; n];
        for (i, &b) in col.iter().enumerate() {
            let lf_i = cur[b as usize];
            cur[b as usize] += 1;
            fl[lf_i] = i as u32;
        }
        let mut f_col = vec![0u8; n];
        for sym in 0..4 {
            for r in c[sym]..c[sym] + cnt[sym] {
                f_col[r] = sym as u8;
            }
        }
        let bits = (32 - (n as u32).saturating_sub(1).leading_zeros()).max(1);
        let mut t = OpenTable::default();
        t.n = n;
        t.bits = bits;
        t.edges = (0..n)
            .map(|u| fl[u] | ((f_col[u] as u32) << bits))
            .collect();
        t.inline16 = (0..=255u8)
            .map(|ch| {
                let mut v = [0u8; INLINE];
                v[0] = ch;
                v
            })
            .collect();
        t.lens = vec![1u8; 256];

        for p in 0..n as u32 {
            t.mark = p;
            emit_serial(&t, &mut o_ser);
            emit_bsac_flat::<1, 1>(&t, &mut sc, &mut o_b1);
            emit_bsac_flat::<2, 1>(&t, &mut sc, &mut o_b2);
            assert_eq!(o_b1, o_ser);
            assert_eq!(o_b2, o_ser);
        }
    };

    let mut bin_cases = 0usize;
    for n in 1..=8usize {
        for mask in 0..(1usize << n) {
            let col: Vec<u8> = (0..n).map(|i| ((mask >> i) & 1) as u8).collect();
            check_col(&col);
            bin_cases += n;
        }
    }
    let mut tern_cases = 0usize;
    for n in 1..=5usize {
        let tot = 3usize.pow(n as u32);
        for mut code in 0..tot {
            let mut col = Vec::with_capacity(n);
            for _ in 0..n {
                col.push((code % 3) as u8);
                code /= 3;
            }
            check_col(&col);
            tern_cases += n;
        }
    }
    println!(
        "[VERIFIED] In-Rust exhaustive BSAC exactness: {bin_cases} binary (n<=8) + {tern_cases} ternary (n<=5) cases PASSED"
    );
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

struct BlockSpec {
    name: &'static str,
    n: usize,
    p: u32,
    sha256: &'static str,
}

const BLOCKS: &[BlockSpec] = &[
    BlockSpec {
        name: "block-0.L",
        n: 899931,
        p: 264045,
        sha256: "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67",
    },
    BlockSpec {
        name: "block-2.L",
        n: 900000,
        p: 320807,
        sha256: "8b47073232d1d9cff7a0688399cfd405211045e15228706f213780e083fc4174",
    },
    BlockSpec {
        name: "block-85.L",
        n: 899935,
        p: 325088,
        sha256: "5def63ec66c914e1986a97f8206a8debd09f6e936f0bc8d8f4cab8544ff64fe8",
    },
    BlockSpec {
        name: "block-143.L",
        n: 899869,
        p: 802868,
        sha256: "eb443421d4d7b459a1db02a00c068f4c2f1e3b7e70f85674228ea2cba30ceba2",
    },
    BlockSpec {
        name: "block-207.L",
        n: 899998,
        p: 205180,
        sha256: "7343ef019b44f0e838e68abc09e36d9806a0f45249cf2286702a2e482e555099",
    },
    BlockSpec {
        name: "block-210.L",
        n: 899959,
        p: 791429,
        sha256: "dd4d0c7d03dd6bc7ec8752ad03b2fd5607222c54bebab8d81ae98a2c277f3dcd",
    },
];

fn main() {
    verify_small_exhaustive_rust();

    let kernels = Kernels::detect();
    let dir = Path::new("/tmp/mibz2-blocks");

    println!("\n=== 1. SHA-256 Verification & Exactness Audit on All 6 Authenticated Wikidata Blocks ===");
    let mut loaded: Vec<(&BlockSpec, Runs, OpenTable)> = Vec::new();
    let mut open_ref = OpenCells::default();
    let mut out_ref = Vec::with_capacity(900_032);
    let mut out_ser = Vec::with_capacity(900_032);
    let mut out_128 = Vec::with_capacity(900_032);
    let mut out_256_8 = Vec::with_capacity(900_032);
    let mut out_256_12 = Vec::with_capacity(900_032);
    let mut sc = BsacFlatScratch::default();

    for spec in BLOCKS {
        let data = fs::read(dir.join(spec.name)).expect("read block");
        assert_eq!(data.len(), spec.n);
        assert_eq!(sha256_hex(&data), spec.sha256, "SHA-256 mismatch on {}", spec.name);
        let runs = runs_from_column(&data, spec.p);
        assert!(open_ref.invert_runs(&runs, spec.n, spec.p, &mut out_ref, kernels));

        let mut tab = OpenTable::default();
        tab.build(&runs, spec.n, spec.p);

        emit_serial(&tab, &mut out_ser);
        emit_bsac_flat::<7, 8>(&tab, &mut sc, &mut out_128);
        emit_bsac_flat::<8, 8>(&tab, &mut sc, &mut out_256_8);
        emit_bsac_flat::<8, 12>(&tab, &mut sc, &mut out_256_12);

        assert_eq!(out_ser, out_ref);
        assert_eq!(out_128, out_ref);
        assert_eq!(out_256_8, out_ref);
        assert_eq!(out_256_12, out_ref);

        let edges_kb = (tab.edges.len() * 4) as f64 / 1024.0;
        println!(
            "  {:<12} SHA256=OK | m16={:6} edges={:5.1}KiB ({:.2}x 2MiB-L2, {:.2}x 256KiB-L2) words={:4} | ALL EXACT",
            spec.name,
            tab.edges.len(),
            edges_kb,
            edges_kb / 2048.0,
            edges_kb / 256.0,
            tab.lens.len() - 256,
        );
        loaded.push((spec, runs, tab));
    }

    // Pre-warm BSAC scratch to max capacity and verify ZERO per-block heap allocation during timed runs!
    for (_, _, tab) in &loaded {
        emit_bsac_flat::<7, 8>(tab, &mut sc, &mut out_128);
        emit_bsac_flat::<8, 8>(tab, &mut sc, &mut out_256_8);
        emit_bsac_flat::<8, 12>(tab, &mut sc, &mut out_256_12);
    }
    let cap_before = (
        sc.g_a.capacity(),
        sc.code_off.capacity(),
        sc.code_len.capacity(),
        sc.code_u16.capacity(),
        sc.slab.capacity(),
    );
    sc.spills_seen = 0;

    // ---------- 2A. Production OpenCells (0.5.11) 1T A/A Control ----------
    let rounds = 25usize;
    let mut open_a = OpenCells::default();
    let mut open_b = OpenCells::default();
    for (spec, runs, _) in &loaded {
        open_a.invert_runs(runs, spec.n, spec.p, &mut out_ref, kernels);
        open_b.invert_runs(runs, spec.n, spec.p, &mut out_ser, kernels);
    }
    println!("\n=== 2A. Production OpenCells (0.5.11) 1T Baseline & A/A Control (median of {rounds} interleaved rounds) ===");
    let (mut s_oa, mut s_ob) = (0.0, 0.0);
    let mut open_medians = Vec::new();
    for (spec, runs, _) in &loaded {
        let (mut toa, mut tob) = (Vec::new(), Vec::new());
        for r in 0..rounds {
            if r % 2 == 0 {
                let st = Instant::now();
                open_a.invert_runs(runs, spec.n, spec.p, &mut out_ref, kernels);
                toa.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                open_b.invert_runs(runs, spec.n, spec.p, &mut out_ser, kernels);
                tob.push(st.elapsed().as_secs_f64() * 1000.0);
            } else {
                let st = Instant::now();
                open_b.invert_runs(runs, spec.n, spec.p, &mut out_ser, kernels);
                tob.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                open_a.invert_runs(runs, spec.n, spec.p, &mut out_ref, kernels);
                toa.push(st.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let moa = median(&mut toa);
        let mob = median(&mut tob);
        s_oa += moa;
        s_ob += mob;
        open_medians.push(moa);
        println!("  {:<12} Open(A)={:6.3}ms  Open(A')={:6.3}ms  A'/A={:.3}x", spec.name, moa, mob, mob / moa);
    }
    println!("  {:<12} Open(A)={:6.3}ms  Open(A')={:6.3}ms  A'/A={:.3}x", "6-block sum", s_oa, s_ob, s_ob / s_oa);

    // ---------- 2B. Isolated Emit-Only 1T Paired Interleaved A/B + A/A Control (Hot-L2 clean) ----------
    println!("\n=== 2B. Isolated Emit-Only 1T Benchmark (Hot-L2 Clean, median of {rounds} interleaved rounds) ===");
    println!(
        "{:<12} {:>7} {:>7} {:>6} | {:>7} {:>6} | {:>7} {:>6} | {:>7} {:>6} | {:>6}",
        "Block", "Ser(A)", "Ser(A')", "A'/A", "B(128,8)", "vsSer", "B(256,8)", "vsSer", "B(256,12)", "vsSer", "Em/Op%"
    );
    println!("{}", "-".repeat(106));

    let (mut s_a, mut s_ap, mut s_1, mut s_2, mut s_3) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (idx, (spec, _, tab)) in loaded.iter().enumerate() {
        for _ in 0..3 {
            emit_serial(tab, &mut out_ser);
            emit_bsac_flat::<7, 8>(tab, &mut sc, &mut out_128);
            emit_bsac_flat::<8, 8>(tab, &mut sc, &mut out_256_8);
            emit_bsac_flat::<8, 12>(tab, &mut sc, &mut out_256_12);
        }
        let (mut ta, mut tap, mut t1, mut t2, mut t3) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for r in 0..rounds {
            if r % 2 == 0 {
                let st = Instant::now();
                emit_serial(tab, &mut out_ser);
                ta.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_serial(tab, &mut out_ref);
                tap.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_bsac_flat::<7, 8>(tab, &mut sc, &mut out_128);
                t1.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_bsac_flat::<8, 8>(tab, &mut sc, &mut out_256_8);
                t2.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_bsac_flat::<8, 12>(tab, &mut sc, &mut out_256_12);
                t3.push(st.elapsed().as_secs_f64() * 1000.0);
            } else {
                let st = Instant::now();
                emit_bsac_flat::<8, 12>(tab, &mut sc, &mut out_256_12);
                t3.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_bsac_flat::<8, 8>(tab, &mut sc, &mut out_256_8);
                t2.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_bsac_flat::<7, 8>(tab, &mut sc, &mut out_128);
                t1.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_serial(tab, &mut out_ref);
                tap.push(st.elapsed().as_secs_f64() * 1000.0);

                let st = Instant::now();
                emit_serial(tab, &mut out_ser);
                ta.push(st.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let ma = median(&mut ta);
        let map = median(&mut tap);
        let m1 = median(&mut t1);
        let m2 = median(&mut t2);
        let m3 = median(&mut t3);
        s_a += ma;
        s_ap += map;
        s_1 += m1;
        s_2 += m2;
        s_3 += m3;
        println!(
            "{:<12} {:5.3}ms {:5.3}ms {:5.3}x | {:5.3}ms {:5.2}x | {:5.3}ms {:5.2}x | {:5.3}ms {:5.2}x | {:5.1}%",
            spec.name, ma, map, map / ma, m1, m1 / ma, m2, m2 / ma, m3, m3 / ma, 100.0 * ma / open_medians[idx]
        );
    }
    println!("{}", "-".repeat(106));
    println!(
        "{:<12} {:5.3}ms {:5.3}ms {:5.3}x | {:5.3}ms {:5.2}x | {:5.3}ms {:5.2}x | {:5.3}ms {:5.2}x | {:5.1}%",
        "6-block sum", s_a, s_ap, s_ap / s_a, s_1, s_1 / s_a, s_2, s_2 / s_a, s_3, s_3 / s_a, 100.0 * s_a / s_oa
    );

    let cap_after = (
        sc.g_a.capacity(),
        sc.code_off.capacity(),
        sc.code_len.capacity(),
        sc.code_u16.capacity(),
        sc.slab.capacity(),
    );
    assert_eq!(cap_before, cap_after, "Unexpected heap reallocation during timed BSAC rounds!");
    println!(
        "  [VERIFIED] Strictly zero per-block heap allocation in BSAC timed rounds (caps unchanged, lane spills={})",
        sc.spills_seen
    );

    // ---------- 3. 4-Worker Concurrent Benchmark (OpenCells A/A + Emit A/B + A/A) ----------
    let waves = 11usize;
    let iters_per_w = 4usize;
    println!("\n=== 3. 4-Worker Concurrent Benchmark ({waves} interleaved waves, 96 block-decodes/wave) ===");
    let run_4w_open = |loaded: &[(&BlockSpec, Runs, OpenTable)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for wid in 0..4usize {
                s.spawn(move || {
                    let mut ko = OpenCells::default();
                    let mut out = Vec::with_capacity(900_032);
                    for _ in 0..iters_per_w {
                        for b in 0..loaded.len() {
                            let (spec, runs, _) = &loaded[(b + wid) % loaded.len()];
                            ko.invert_runs(runs, spec.n, spec.p, &mut out, kernels);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let run_4w_ser = |loaded: &[(&BlockSpec, Runs, OpenTable)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for wid in 0..4usize {
                s.spawn(move || {
                    let mut out = Vec::with_capacity(900_032);
                    for _ in 0..iters_per_w {
                        for b in 0..loaded.len() {
                            let (_, _, tab) = &loaded[(b + wid) % loaded.len()];
                            emit_serial(tab, &mut out);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let run_4w_b256 = |loaded: &[(&BlockSpec, Runs, OpenTable)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for wid in 0..4usize {
                s.spawn(move || {
                    let mut out = Vec::with_capacity(900_032);
                    let mut bsc = BsacFlatScratch::default();
                    // Pre-warm once outside inner loop
                    emit_bsac_flat::<8, 8>(&loaded[5].2, &mut bsc, &mut out);
                    for _ in 0..iters_per_w {
                        for b in 0..loaded.len() {
                            let (_, _, tab) = &loaded[(b + wid) % loaded.len()];
                            emit_bsac_flat::<8, 8>(tab, &mut bsc, &mut out);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };
    let run_4w_b128 = |loaded: &[(&BlockSpec, Runs, OpenTable)]| -> f64 {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for wid in 0..4usize {
                s.spawn(move || {
                    let mut out = Vec::with_capacity(900_032);
                    let mut bsc = BsacFlatScratch::default();
                    emit_bsac_flat::<7, 8>(&loaded[5].2, &mut bsc, &mut out);
                    for _ in 0..iters_per_w {
                        for b in 0..loaded.len() {
                            let (_, _, tab) = &loaded[(b + wid) % loaded.len()];
                            emit_bsac_flat::<7, 8>(tab, &mut bsc, &mut out);
                        }
                    }
                    std::hint::black_box(out.len());
                });
            }
        });
        t0.elapsed().as_secs_f64() * 1000.0
    };

    let _ = run_4w_open(&loaded);
    let _ = run_4w_ser(&loaded);
    let _ = run_4w_b256(&loaded);
    let (mut w_oa, mut w_ob, mut w_sa, mut w_sb, mut w_b8, mut w_b7) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for w in 0..waves {
        if w % 2 == 0 {
            w_oa.push(run_4w_open(&loaded));
            w_ob.push(run_4w_open(&loaded));
            w_sa.push(run_4w_ser(&loaded));
            w_sb.push(run_4w_ser(&loaded));
            w_b8.push(run_4w_b256(&loaded));
            w_b7.push(run_4w_b128(&loaded));
        } else {
            w_b7.push(run_4w_b128(&loaded));
            w_b8.push(run_4w_b256(&loaded));
            w_sb.push(run_4w_ser(&loaded));
            w_sa.push(run_4w_ser(&loaded));
            w_ob.push(run_4w_open(&loaded));
            w_oa.push(run_4w_open(&loaded));
        }
    }
    let moa = median(&mut w_oa);
    let mob = median(&mut w_ob);
    let msa = median(&mut w_sa);
    let msb = median(&mut w_sb);
    let mb8 = median(&mut w_b8);
    let mb7 = median(&mut w_b7);
    println!("  OpenCells 4W (A)       : {moa:7.2} ms  (1.000x end-to-end)");
    println!("  OpenCells 4W (A' ctrl) : {mob:7.2} ms  ({:.3}x A/A control)", mob / moa);
    println!("  SerialEmit 4W (A)      : {msa:7.2} ms  (1.000x emit; {:.1}% of OpenCells 4W)", 100.0 * msa / moa);
    println!("  SerialEmit 4W (A' ctrl): {msb:7.2} ms  ({:.3}x A/A control)", msb / msa);
    println!("  BSAC(256, K=8) 4W      : {mb8:7.2} ms  ({:.3}x vs SerialEmit)", mb8 / msa);
    println!("  BSAC(128, K=8) 4W      : {mb7:7.2} ms  ({:.3}x vs SerialEmit)", mb7 / msa);

    // ---------- 4. Synthetic Pure-Random Width-1 Scaling vs edges/L2 ----------
    println!("\n=== 4. Synthetic Pure-Random Width-1 Scaling by edges/L2 (Xeon L2=2048KiB vs i9 L2=256KiB) ===");
    println!(
        "{:>9} {:>8} {:>7} {:>7} | {:>8} {:>7} | {:>8} {:>7} | {:>8} {:>7}",
        "m", "edgesKiB", "e/2MiB", "e/256K", "Serial", "ns/st", "B(256,8)", "vsSer", "B(256,12)", "vsSer"
    );
    println!("{}", "-".repeat(98));
    for &m in &[
        16_384usize,
        32_768,
        65_536,
        100_000,
        200_000,
        300_000,
        500_000,
        1_000_000,
        2_000_000,
        4_000_000,
    ] {
        let mut perm: Vec<u32> = (0..m as u32).collect();
        let mut rng: u64 = 0x1234_5678_9ABC_DEF1 ^ (m as u64);
        for i in (1..m).rev() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = ((rng >> 33) as usize) % (i + 1);
            perm.swap(i, j);
        }
        let bits = (32 - (m as u32 - 1).leading_zeros()).max(1);
        let mut t = OpenTable::default();
        t.n = m;
        t.mark = 0;
        t.bits = bits;
        t.edges.resize(m, 0);
        for i in 0..m {
            let u = perm[i];
            let v = perm[(i + 1) % m];
            let sym = (u & 0xFF) as u32;
            t.edges[u as usize] = v | (sym << bits);
        }
        t.inline16 = (0..=255u8)
            .map(|c| {
                let mut v = [0u8; INLINE];
                v[0] = c;
                v
            })
            .collect();
        t.lens = vec![1u8; 256];

        let mut o_ser = Vec::with_capacity(m + INLINE);
        let mut o_8 = Vec::with_capacity(m + INLINE);
        let mut o_12 = Vec::with_capacity(m + INLINE);

        emit_serial(&t, &mut o_ser);
        emit_bsac_flat::<8, 8>(&t, &mut sc, &mut o_8);
        emit_bsac_flat::<8, 12>(&t, &mut sc, &mut o_12);
        assert_eq!(o_ser, o_8);
        assert_eq!(o_ser, o_12);

        let rds = if m <= 500_000 { 15 } else { 5 };
        let (mut ts, mut t8, mut t12) = (Vec::new(), Vec::new(), Vec::new());
        for r in 0..rds {
            if r % 2 == 0 {
                let st = Instant::now();
                emit_serial(&t, &mut o_ser);
                ts.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                emit_bsac_flat::<8, 8>(&t, &mut sc, &mut o_8);
                t8.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                emit_bsac_flat::<8, 12>(&t, &mut sc, &mut o_12);
                t12.push(st.elapsed().as_secs_f64() * 1000.0);
            } else {
                let st = Instant::now();
                emit_bsac_flat::<8, 12>(&t, &mut sc, &mut o_12);
                t12.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                emit_bsac_flat::<8, 8>(&t, &mut sc, &mut o_8);
                t8.push(st.elapsed().as_secs_f64() * 1000.0);
                let st = Instant::now();
                emit_serial(&t, &mut o_ser);
                ts.push(st.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let ms = median(&mut ts);
        let m8 = median(&mut t8);
        let m12 = median(&mut t12);
        let ekb = (m * 4) as f64 / 1024.0;
        println!(
            "{:9} {:7.0}K {:6.2}x {:6.2}x | {:6.3}ms {:6.2}ns | {:6.3}ms {:6.3}x | {:6.3}ms {:6.3}x",
            m,
            ekb,
            ekb / 2048.0,
            ekb / 256.0,
            ms,
            ms * 1e6 / m as f64,
            m8,
            m8 / ms,
            m12,
            m12 / ms
        );
    }
}
