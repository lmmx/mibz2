#![forbid(unsafe_code)]
//! Wormhole Cycle 8 — Safe-Rust Confrontation of Zero-Bit Xor-Mask Cuts (`1P-Bounded256K` vs `2P-WS`)
//! Against `origin/master` (`d425d77`, `mibz2 0.5.9`) `OpenCells` and Fast SWAR `DietedOpen`:
//!   - Arm 0: `Master-0.5.9` (`mibz2::bwt::OpenCells` compiled from `origin/master`)
//!   - Arm 1: `DietedOpen`   (`mu = 1`: SWAR64 `OutQ16` in dead `out` + `Chaitin-Node16` in `edges` + `[u32;3]` coloring)
//!   - Arm 2: `1P-Stage256K` (`mu = 4`: 1-Pass with L3-safe `256 KiB` staging cap — tests Theorem 8.2(1) stall law!)
//!   - Arm 3: `2P-WS8x32`    (`mu = 8, S = 32`: 2-Pass Work-Stealing Zero-Copy MLP, `0 B` staging heap — Theorem 8.2(2)!)
//!
//! Computes BOTH exact BMCP(256 KiB, 64 B, mu) 6-dimensional metrics (including true parallel round-by-round
//! `D_past_L2` simulation across all `mu` concurrent lanes!) AND 21-round paired interleaved safe-Rust timings!

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

// ============================================================================
// Exact `.bz2` Stream Synthesizer for `origin/master` `OpenCells` Parity
// ============================================================================
#[derive(Default)]
struct BitWriter {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}
impl BitWriter {
    fn put(&mut self, v: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.acc = (self.acc << 1) | ((v >> i) & 1);
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.n = 0;
            }
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.put(0, 8 - self.n);
        }
        self.out
    }
}

fn plain_invert(ll: &[u8], orig_ptr: u32) -> Vec<u8> {
    let n = ll.len();
    let mut counts = [0usize; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut cum = [0usize; 256];
    let mut s = 0;
    for i in 0..256 {
        cum[i] = s;
        s += counts[i];
    }
    let mut psi = vec![0u32; n];
    for (i, &c) in ll.iter().enumerate() {
        psi[cum[c as usize]] = i as u32;
        cum[c as usize] += 1;
    }
    let mut p = psi[orig_ptr as usize];
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(ll[p as usize]);
        p = psi[p as usize];
    }
    out
}

fn mtf_symbols(ll: &[u8], used: &[u8]) -> Vec<u16> {
    let mut list: Vec<u8> = used.to_vec();
    let mut syms = Vec::with_capacity(ll.len());
    let mut run = 0u32;
    let flush = |run: &mut u32, syms: &mut Vec<u16>| {
        if *run > 0 {
            let mut z = *run - 1;
            loop {
                syms.push((z & 1) as u16);
                if z < 2 {
                    break;
                }
                z = (z - 2) / 2;
            }
            *run = 0;
        }
    };
    for &c in ll {
        let j = list.iter().position(|&x| x == c).unwrap();
        if j == 0 {
            run += 1;
            continue;
        }
        flush(&mut run, &mut syms);
        list.remove(j);
        list.insert(0, c);
        syms.push(j as u16 + 1);
    }
    flush(&mut run, &mut syms);
    syms.push(used.len() as u16 + 1);
    syms
}

fn craft_bz2_block(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Vec<u8> {
    let pre_rle1 = plain_invert(ll, orig_ptr);
    let mut unrled = Vec::with_capacity(pre_rle1.len());
    mibz2::rle::unrle(&pre_rle1, &mut unrled, kernels).expect("valid RLE1");
    let crc = mibz2::crc::crc32(&unrled, kernels);

    let mut seen = [false; 256];
    for &c in ll {
        seen[c as usize] = true;
    }
    let used: Vec<u8> = (0..=255u8).filter(|&c| seen[c as usize]).collect();
    let syms = mtf_symbols(ll, &used);
    let alpha = used.len() + 2;
    let code_len = (usize::BITS - (alpha - 1).leading_zeros()).max(1);

    let mut w = BitWriter::default();
    for &b in b"BZh9" {
        w.put(b as u64, 8);
    }
    w.put(0x3141_5926_5359, 48);
    w.put(crc as u64, 32);
    w.put(0, 1);
    w.put(orig_ptr as u64, 24);

    let mut in16 = 0u64;
    for &c in &used {
        in16 |= 0x8000 >> (c / 16);
    }
    w.put(in16, 16);
    for i in 0..16 {
        if in16 & (0x8000 >> i) != 0 {
            let mut word = 0u64;
            for &c in &used {
                if (c / 16) as u64 == i {
                    word |= 0x8000 >> (c % 16);
                }
            }
            w.put(word, 16);
        }
    }
    let n_sel = syms.len().div_ceil(50).max(1);
    w.put(2, 3);
    w.put(n_sel as u64, 15);
    for _ in 0..n_sel {
        w.put(0, 1);
    }
    for _ in 0..2 {
        w.put(code_len as u64, 5);
        for _ in 0..alpha {
            w.put(0, 1);
        }
    }
    for &s in &syms {
        w.put(s as u64, code_len);
    }
    w.put(0x1772_4538_5090, 48);
    w.put(crc as u64, 32);
    w.finish()
}

fn extract_runs(bz2: &[u8], kernels: Kernels) -> (Runs, usize, u32) {
    let mut sc = Scanner::new(SliceSource(bz2));
    let mut runs = Runs::default();
    match sc.next(Vec::new()).expect("valid bz2") {
        Event::Block { symbols, .. } => {
            let p = symbols.orig_ptr;
            let n = runs::front(&symbols, &mut runs, kernels).expect("front ok");
            (runs, n, p)
        }
        _ => panic!("expected block"),
    }
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

// ============================================================================
// Fast SWAR64 `DietedOpen` Shared Table Builder + Pluggable Emit Modes!
// ============================================================================
#[derive(Default)]
pub struct DietedKernel {
    pub edges: Vec<u32>,
    pub triples: Vec<[u32; 3]>,
    pub spelling: Vec<u8>,
    pub offsets: Vec<u32>,
    pub inline16: Vec<[u8; INLINE]>,
    pub lens: Vec<u8>,
    pub word_len: Vec<u16>, // Exact u16 byte length per code (<= 8 KiB, 100% L1d-resident!)
    pub chunk_next: Vec<u16>,
    pub ovf_q: Vec<[u16; 512]>,
    pub stk: Vec<u32>,
    pub stage_256k: Vec<u8>, // Used ONLY by Arm 2 (`1P-Stage256K`)
}

impl DietedKernel {
    pub fn base_heap_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.triples.capacity() * 12
            + self.spelling.capacity()
            + self.offsets.capacity() * 4
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.word_len.capacity() * 2
            + self.chunk_next.capacity() * 2
            + self.ovf_q.capacity() * 1024
            + self.stk.capacity() * 4
    }

    #[inline]
    fn ensure_exact_u32(v: &mut Vec<u32>, need: usize) {
        if v.capacity() < need {
            v.reserve_exact(need - v.len());
        }
        if v.len() < need {
            v.resize(need, 0);
        }
    }

    /// Phases 1-3: Fast SWAR64 `OutQ16` + `Chaitin-Node16` in `edges` + Band-Span Wiring!
    /// Returns `(m, mark, bits)`.
    pub fn build_table(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u32, u32, u32) {
        if out.capacity() < n + INLINE {
            out.reserve_exact(n + INLINE - out.len());
        }
        if out.len() < n + INLINE {
            out.resize(n + INLINE, 0);
        }
        let c_max = out.len() / 1024;
        self.ovf_q.clear();
        self.chunk_next.clear();
        self.triples.clear();

        let mut q_head = [u16::MAX; 256];
        let mut q_cur = [u16::MAX; 256];
        let mut q_ic = [512u16; 256];
        // Max epochs for m <= 900,000 is 14 (`900,000 >> 16 = 13`): fixed stack array, 0 heap allocs!
        let mut ep_thresh = [[u32::MAX; 16]; 256];
        let mut ep_cnt = [0u8; 256];
        let mut cur_epoch = 0u32;

        const SPLAT: u64 = 0x0001_0001_0001_0001;
        const IOTA0: u64 = 0x0003_0002_0001_0000;
        const IOTA4: u64 = 0x0007_0006_0005_0004;
        const IOTA8: u64 = 0x000B_000A_0009_0008;
        const IOTA12: u64 = 0x000F_000E_000D_000C;

        let push_slow = |c: usize,
                         m_start: u32,
                         w: u32,
                         out_buf: &mut [u8],
                         ovf_q: &mut Vec<[u16; 512]>,
                         q_head: &mut [u16; 256],
                         q_cur: &mut [u16; 256],
                         q_ic: &mut [u16; 256],
                         chunk_next: &mut Vec<u16>| {
            let mut rem = w;
            let mut r_val = m_start;
            while rem > 0 {
                let ic = q_ic[c] as usize;
                if ic >= 512 {
                    let new_cid = chunk_next.len() as u16;
                    chunk_next.push(u16::MAX);
                    if new_cid as usize >= c_max {
                        ovf_q.push([0u16; 512]);
                    }
                    if q_head[c] == u16::MAX {
                        q_head[c] = new_cid;
                    } else {
                        chunk_next[q_cur[c] as usize] = new_cid;
                    }
                    q_cur[c] = new_cid;
                    q_ic[c] = 0;
                    continue;
                }
                let take = rem.min((512 - ic) as u32);
                let cid = q_cur[c] as usize;
                if cid < c_max {
                    let base = cid * 1024 + ic * 2;
                    for d in 0..take as usize {
                        let b = (((r_val + d as u32) & 0xFFFF) as u16).to_le_bytes();
                        out_buf[base + 2 * d] = b[0];
                        out_buf[base + 2 * d + 1] = b[1];
                    }
                } else {
                    let slot = &mut ovf_q[cid - c_max];
                    for d in 0..take as usize {
                        slot[ic + d] = ((r_val + d as u32) & 0xFFFF) as u16;
                    }
                }
                q_ic[c] += take as u16;
                r_val += take;
                rem -= take;
            }
        };

        let mut all = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m = 0u32;
        let mut mark = 0u32;
        let mut pos = 0u32;
        let mut i = 0usize;
        let nrecs = records.len();

        while i < nrecs {
            let rec = records[i];
            let c = (rec >> 8) as usize;
            let mut w = u32::from(rec & 255);
            i += 1;
            while i < nrecs && (records[i] >> 8) as usize == c {
                w += u32::from(records[i] & 255);
                i += 1;
            }
            let end = pos + w;
            if w <= LIMIT {
                if p.wrapping_sub(pos) < w {
                    mark = m + (p - pos);
                }
                let ic = q_ic[c] as usize;
                let cid = q_cur[c] as usize;
                let lo = m & 0xFFFF;
                // 99%+ SWAR64 fast path: fits in current in-`out` chunk AND does not cross 64K epoch!
                if ic <= 496 && cid < c_max && (m >> 16) == cur_epoch && lo <= 0xFFF0 {
                    let base = cid * 1024 + ic * 2;
                    let r64 = (lo as u64).wrapping_mul(SPLAT);
                    let dst = &mut out[base..base + 32];
                    dst[0..8].copy_from_slice(&r64.wrapping_add(IOTA0).to_le_bytes());
                    dst[8..16].copy_from_slice(&r64.wrapping_add(IOTA4).to_le_bytes());
                    dst[16..24].copy_from_slice(&r64.wrapping_add(IOTA8).to_le_bytes());
                    dst[24..32].copy_from_slice(&r64.wrapping_add(IOTA12).to_le_bytes());
                    q_ic[c] = (ic + w as usize) as u16;
                    thin[c] += w;
                    m += w;
                } else {
                    let mut left = w;
                    let mut cur_m = m;
                    while left > 0 {
                        let ep = cur_m >> 16;
                        if ep > cur_epoch {
                            cur_epoch = ep;
                            for s in 0..256usize {
                                let idx = ep_cnt[s] as usize;
                                ep_thresh[s][idx] = thin[s];
                                ep_cnt[s] += 1;
                            }
                        }
                        let room_ep = 65536 - (cur_m & 0xFFFF);
                        let step = left.min(room_ep);
                        push_slow(
                            c,
                            cur_m,
                            step,
                            out,
                            &mut self.ovf_q,
                            &mut q_head,
                            &mut q_cur,
                            &mut q_ic,
                            &mut self.chunk_next,
                        );
                        thin[c] += step;
                        cur_m += step;
                        left -= step;
                    }
                    m = cur_m;
                }
                all[c] += w;
            } else if p.wrapping_sub(pos) >= w {
                self.triples.push([pos, all[c], (w << 8) | c as u32]);
                thick[c] += 1;
                all[c] += w;
            } else {
                if pos < p {
                    let lw = p - pos;
                    self.triples.push([pos, all[c], (lw << 8) | c as u32]);
                    thick[c] += 1;
                    all[c] += lw;
                }
                mark = m;
                let ep = m >> 16;
                if ep > cur_epoch {
                    cur_epoch = ep;
                    for s in 0..256usize {
                        let idx = ep_cnt[s] as usize;
                        ep_thresh[s][idx] = thin[s];
                        ep_cnt[s] += 1;
                    }
                }
                push_slow(
                    c,
                    m,
                    1,
                    out,
                    &mut self.ovf_q,
                    &mut q_head,
                    &mut q_cur,
                    &mut q_ic,
                    &mut self.chunk_next,
                );
                thin[c] += 1;
                m += 1;
                all[c] += 1;
                if p + 1 < end {
                    let rw = end - p - 1;
                    self.triples.push([p + 1, all[c], (rw << 8) | c as u32]);
                    thick[c] += 1;
                    all[c] += rw;
                }
            }
            pos = end;
        }

        // Phase 2: Direct-Linked Node16 inside `self.edges` (with `reserve_exact`)
        let nstrips = self.triples.len();
        let max_nodes = 3 * nstrips + 4;
        assert!(max_nodes < 65535);
        let bot_base = 4 * max_nodes;
        let img_base = 5 * max_nodes;
        let rules_base = img_base + nstrips;
        let init_u32 = (rules_base + 4 * max_nodes).max(m as usize + 4);
        Self::ensure_exact_u32(&mut self.edges, init_u32);

        let mut first = [0u32; 256];
        let mut write = [0u32; 256];
        let (mut acc0, mut acc1) = (0u32, 0u32);
        for a in 0..256 {
            first[a] = acc0;
            acc0 += all[a];
            write[a] = acc1;
            acc1 += thick[a];
        }
        for si in 0..nstrips {
            let a = (self.triples[si][2] & 255) as usize;
            self.triples[si][1] += first[a];
            let slot = write[a] as usize;
            write[a] += 1;
            self.edges[img_base + slot] = si as u32;
        }

        let set_node = |ed: &mut [u32], id: u16, width: u32, word: u32| {
            let b = (id as usize) * 4;
            ed[b] = 0;
            ed[b + 1] = 0;
            ed[b + 2] = width;
            ed[b + 3] = word;
        };
        let get_prev = |ed: &[u32], id: u16, side: usize| -> u16 {
            let v = ed[(id as usize) * 4];
            if side == 0 { v as u16 } else { (v >> 16) as u16 }
        };
        let set_prev = |ed: &mut [u32], id: u16, side: usize, val: u16| {
            let b = (id as usize) * 4;
            if side == 0 {
                ed[b] = (ed[b] & 0xFFFF_0000) | u32::from(val);
            } else {
                ed[b] = (ed[b] & 0x0000_FFFF) | (u32::from(val) << 16);
            }
        };
        let get_next = |ed: &[u32], id: u16, side: usize| -> u16 {
            let v = ed[(id as usize) * 4 + 1];
            if side == 0 { v as u16 } else { (v >> 16) as u16 }
        };
        let set_next = |ed: &mut [u32], id: u16, side: usize, val: u16| {
            let b = (id as usize) * 4 + 1;
            if side == 0 {
                ed[b] = (ed[b] & 0xFFFF_0000) | u32::from(val);
            } else {
                ed[b] = (ed[b] & 0x0000_FFFF) | (u32::from(val) << 16);
            }
        };

        set_node(&mut self.edges, 0, 0, 0);
        let mut nnodes = 1u16;
        for si in 0..nstrips {
            let w = self.triples[si][2] >> 8;
            let byte = self.triples[si][2] & 255;
            set_node(&mut self.edges, nnodes, w, byte + 1);
            nnodes += 1;
        }

        let app = |ed: &mut [u32], side: usize, ht: &mut (u16, u16), id: u16| {
            if ht.0 == 0 {
                set_prev(ed, id, side, 0);
                set_next(ed, id, side, 0);
                *ht = (id, id);
            } else {
                set_next(ed, ht.1, side, id);
                set_prev(ed, id, side, ht.1);
                set_next(ed, id, side, 0);
                ht.1 = id;
            }
        };

        let (mut t0, mut o0, mut t1, mut o1) = ((0u16, 0u16), (0u16, 0u16), (0u16, 0u16), (0u16, 0u16));
        let mut x = 0u32;
        for si in 0..nstrips {
            let from = self.triples[si][0];
            let w = self.triples[si][2] >> 8;
            if x < from {
                let id = nnodes;
                nnodes += 1;
                set_node(&mut self.edges, id, from - x, 0);
                app(&mut self.edges, 0, &mut o0, id);
                app(&mut self.edges, 1, &mut t1, id);
            }
            app(&mut self.edges, 0, &mut o0, (si + 1) as u16);
            x = from + w;
        }
        if (x as usize) < n {
            let id = nnodes;
            nnodes += 1;
            set_node(&mut self.edges, id, n as u32 - x, 0);
            app(&mut self.edges, 0, &mut o0, id);
            app(&mut self.edges, 1, &mut t1, id);
        }

        x = 0;
        for k in 0..nstrips {
            let si = self.edges[img_base + k] as usize;
            let to = self.triples[si][1];
            let w = self.triples[si][2] >> 8;
            if x < to {
                let id = nnodes;
                nnodes += 1;
                set_node(&mut self.edges, id, to - x, 0);
                app(&mut self.edges, 1, &mut o1, id);
                app(&mut self.edges, 0, &mut t0, id);
            }
            app(&mut self.edges, 1, &mut o1, (si + 1) as u16);
            x = to + w;
        }
        if (x as usize) < n {
            let id = nnodes;
            set_node(&mut self.edges, id, n as u32 - x, 0);
            app(&mut self.edges, 1, &mut o1, id);
            app(&mut self.edges, 0, &mut t0, id);
        }

        for (side, t_ht, o_ht) in [(0usize, t0, o0), (1usize, t1, o1)] {
            if t_ht.0 != 0 && o_ht.0 != 0 {
                set_next(&mut self.edges, 0, side, t_ht.0);
                set_prev(&mut self.edges, t_ht.0, side, 0);
                set_next(&mut self.edges, t_ht.1, side, o_ht.0);
                set_prev(&mut self.edges, o_ht.0, side, t_ht.1);
                set_next(&mut self.edges, o_ht.1, side, 0);
                set_prev(&mut self.edges, 0, side, o_ht.1);
            } else if t_ht.0 != 0 {
                set_next(&mut self.edges, 0, side, t_ht.0);
                set_prev(&mut self.edges, t_ht.0, side, 0);
                set_next(&mut self.edges, t_ht.1, side, 0);
                set_prev(&mut self.edges, 0, side, t_ht.1);
            } else if o_ht.0 != 0 {
                set_next(&mut self.edges, 0, side, o_ht.0);
                set_prev(&mut self.edges, o_ht.0, side, 0);
                set_next(&mut self.edges, o_ht.1, side, 0);
                set_prev(&mut self.edges, 0, side, o_ht.1);
            }
        }

        let cut_node = |ed: &mut [u32], id: u16, side: usize| {
            let prv = get_prev(ed, id, side);
            let nxt = get_next(ed, id, side);
            set_next(ed, prv, side, nxt);
            set_prev(ed, nxt, side, prv);
        };
        let ins_after = |ed: &mut [u32], at: u16, id: u16, side: usize| {
            let nxt = get_next(ed, at, side);
            set_prev(ed, id, side, at);
            set_next(ed, id, side, nxt);
            set_next(ed, at, side, id);
            set_prev(ed, nxt, side, id);
        };

        let mut nrules = 0usize;
        let mut rem = n as u32;
        while rem > 0 {
            let nx = get_prev(&self.edges, 0, 0);
            let ny = get_prev(&self.edges, 0, 1);
            let aw = self.edges[(nx as usize) * 4 + 2];
            let bw = self.edges[(ny as usize) * 4 + 2];
            let w = aw.min(bw);
            rem -= w;
            if nx == ny {
                cut_node(&mut self.edges, nx, 0);
                cut_node(&mut self.edges, nx, 1);
                continue;
            }
            let lw = self.edges[(nx as usize) * 4 + 3];
            let rw = self.edges[(ny as usize) * 4 + 3];
            let wj = if lw == 0 {
                rw
            } else if rw == 0 {
                lw
            } else {
                let ll = if lw <= 256 { 1 } else { self.edges[rules_base + (lw as usize - 257) * 4 + 2] };
                let rl = if rw <= 256 { 1 } else { self.edges[rules_base + (rw as usize - 257) * 4 + 2] };
                let rb = rules_base + nrules * 4;
                if self.edges.len() < rb + 4 {
                    self.edges.resize(rb + 260, 0);
                }
                self.edges[rb] = lw;
                self.edges[rb + 1] = rw;
                self.edges[rb + 2] = ll + rl;
                self.edges[rb + 3] = u32::MAX;
                let id = (nrules + 257) as u32;
                nrules += 1;
                id
            };
            if aw <= bw {
                cut_node(&mut self.edges, nx, 0);
                ins_after(&mut self.edges, ny, nx, 0);
                self.edges[(nx as usize) * 4 + 3] = wj;
                if aw == bw {
                    cut_node(&mut self.edges, ny, 0);
                    cut_node(&mut self.edges, ny, 1);
                } else {
                    self.edges[(ny as usize) * 4 + 2] -= w;
                }
            } else {
                cut_node(&mut self.edges, ny, 1);
                ins_after(&mut self.edges, nx, ny, 1);
                self.edges[(ny as usize) * 4 + 3] = wj;
                self.edges[(nx as usize) * 4 + 2] -= w;
            }
        }

        let mut id = get_next(&self.edges, 0, 1);
        x = 0;
        while id != 0 {
            self.edges[bot_base + id as usize] = x;
            x += self.edges[(id as usize) * 4 + 2];
            id = get_next(&self.edges, id, 1);
        }

        self.triples.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        if self.inline16.len() < 256 {
            self.inline16.clear();
            for c in 0..=255u8 {
                let mut v = [0u8; INLINE];
                v[0] = c;
                self.inline16.push(v);
            }
            self.lens.clear();
            self.lens.resize(256, 1);
            self.word_len.clear();
            self.word_len.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
        self.word_len.truncate(256);

        let mut cursors = [0usize; 257];
        id = get_next(&self.edges, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
        let mut nwords = 0usize;
        for a in 0..256usize {
            cursors[a] = self.triples.len();
            let end = k + thin[a];
            while k < end {
                let nw = self.edges[(id as usize) * 4 + 2];
                let width = (end - k).min(nw - in_node);
                let dst = self.edges[bot_base + id as usize] + in_node;
                let wd = self.edges[(id as usize) * 4 + 3];
                let code = if wd == 0 {
                    a as u32
                } else {
                    let o0 = self.spelling.len();
                    self.stk.clear();
                    self.stk.push(wd);
                    while let Some(cur) = self.stk.pop() {
                        if cur <= 256 {
                            self.spelling.push((cur - 1) as u8);
                        } else {
                            let rb = rules_base + (cur as usize - 257) * 4;
                            let first_off = self.edges[rb + 3];
                            if first_off == u32::MAX {
                                self.edges[rb + 3] = self.spelling.len() as u32;
                                self.stk.push(self.edges[rb + 1]);
                                self.stk.push(self.edges[rb]);
                            } else {
                                let f = first_off as usize;
                                let l = self.edges[rb + 2] as usize;
                                self.spelling.extend_from_within(f..f + l);
                            }
                        }
                    }
                    self.spelling.push(a as u8);
                    self.offsets.push(self.spelling.len() as u32);
                    let wl = self.spelling.len() - o0;
                    let mut v = [0u8; INLINE];
                    let head = wl.min(INLINE);
                    v[..head].copy_from_slice(&self.spelling[o0..o0 + head]);
                    self.inline16.push(v);
                    self.lens.push(if wl <= INLINE { wl as u8 } else { LONG });
                    self.word_len.push(wl.min(u16::MAX as usize) as u16);
                    nwords += 1;
                    (255 + nwords) as u32
                };
                self.triples.push([k + width, dst.wrapping_sub(k), code]);
                in_node += width;
                k += width;
                if in_node == nw {
                    id = get_next(&self.edges, id, 0);
                    in_node = 0;
                }
            }
        }
        cursors[256] = self.triples.len();

        // Phase 3: Fast Span-Chunked Band Wiring (hoisting chunk & epoch checks OUT of inner loop!)
        let bits = if m <= 1 { 1 } else { 32 - (m - 1).leading_zeros() };
        k = 0;
        for a in 0..256usize {
            let mut cid = q_head[a] as usize;
            let mut ic = 0usize;
            let mut consumed = 0u32;
            let mut ep_idx = 0usize;
            let n_ep = ep_cnt[a] as usize;
            let mut next_ep = if n_ep > 0 { ep_thresh[a][0] } else { u32::MAX };
            let mut high_add = 0u32;

            for bi in cursors[a]..cursors[a + 1] {
                let b_end = self.triples[bi][0];
                let mut dst = k.wrapping_add(self.triples[bi][1]) as usize;
                let pack = self.triples[bi][2] << bits;
                let mut rem_span = (b_end - k) as usize;
                while rem_span > 0 {
                    while consumed == next_ep {
                        high_add += 65536;
                        ep_idx += 1;
                        next_ep = if ep_idx < n_ep { ep_thresh[a][ep_idx] } else { u32::MAX };
                    }
                    if ic == 512 {
                        cid = self.chunk_next[cid] as usize;
                        ic = 0;
                    }
                    let room_chunk = 512 - ic;
                    let room_ep = (next_ep - consumed) as usize;
                    let step = rem_span.min(room_chunk).min(room_ep);
                    let pack_hi = pack + high_add;
                    let ed_dst = &mut self.edges[dst..dst + step];
                    if cid < c_max {
                        let base = cid * 1024 + ic * 2;
                        let src_bytes = &out[base..base + step * 2];
                        for d in 0..step {
                            let lo = u16::from_le_bytes([src_bytes[2 * d], src_bytes[2 * d + 1]]);
                            ed_dst[d] = pack_hi | u32::from(lo);
                        }
                    } else {
                        let slot = &self.ovf_q[cid - c_max][ic..ic + step];
                        for d in 0..step {
                            ed_dst[d] = pack_hi | u32::from(slot[d]);
                        }
                    }
                    dst += step;
                    ic += step;
                    consumed += step as u32;
                    rem_span -= step;
                }
                k = b_end;
            }
        }
        self.edges.truncate(m as usize);
        (m, mark, bits)
    }

    /// Arm 1 (`mu = 1`): Standard Serial 1-Lane Emit
    pub fn emit_serial(&self, n: usize, m: u32, mark: u32, bits: u32, out: &mut Vec<u8>) {
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
        let mut at = 0usize;
        for _ in 0..m {
            let edge = self.edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            let len = self.lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&self.inline16[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    /// Zero-Bit Xor-Mask Cut Geometry: `((j ^ mark) & cm) == 0`
    #[inline]
    pub fn cut_params(m: u32, mark: u32, bits: u32, log2_cuts: u32) -> (u32, u32, [u32; 64], usize) {
        let shift = bits.saturating_sub(log2_cuts);
        let cm = (1u32 << shift) - 1;
        let rem_low = mark & cm;
        let mut cuts = [0u32; 64];
        let mut n_cuts = 0usize;
        let stride = cm + 1;
        let mut q = 0u32;
        while let Some(c_row) = q.checked_mul(stride).map(|v| v + rem_low) {
            if c_row >= m || n_cuts >= 64 {
                break;
            }
            cuts[n_cuts] = c_row;
            n_cuts += 1;
            q += 1;
        }
        (shift, cm, cuts, n_cuts)
    }

    /// Arm 2 (`1P-Stage256K`, `mu = 4`, `S = 32` cuts):
    /// Lane 0 emits directly at `out[0..]` from `mark`; Lanes 1..3 stage into a `256 KiB`
    /// L3-safe slab pool (`32` x `8 KiB` slabs, recycled as soon as Lane 0 flushes a ready cut!).
    /// Returns `(total_steps, active_aux_steps)` to directly measure Theorem 8.2(1)'s stall fraction!
    pub fn emit_1p_bounded256k(
        &mut self,
        n: usize,
        m: u32,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) -> (usize, usize) {
        if m < 1024 {
            self.emit_serial(n, m, mark, bits, out);
            return (m as usize, 0);
        }
        const CAP_STAGE: usize = 256 * 1024;
        if self.stage_256k.len() < CAP_STAGE {
            self.stage_256k.resize(CAP_STAGE, 0);
        }

        let (shift, cm, cuts, n_cuts) = Self::cut_params(m, mark, bits, 4); // up to 16 cuts
        if n_cuts <= 1 {
            self.emit_serial(n, m, mark, bits, out);
            return (m as usize, 0);
        }
        let mark_cid = (mark >> shift) as usize;
        let mask = (1u32 << bits) - 1;

        // Track staged cuts: 15 non-mark cuts share `CAP_STAGE` (`~17 KiB` max per cut before stall!)
        let per_slot = (CAP_STAGE / n_cuts).saturating_sub(32);
        let mut slot_len = [0usize; 64];
        let mut slot_next_cid = [usize::MAX; 64];
        let mut slot_done = [false; 64];
        let mut slot_stalled_j = [u32::MAX; 64];
        let mut claimed = [false; 64];
        claimed[mark_cid] = true;

        let mut next_claim = 0usize;
        let claim_cut = |claimed: &mut [bool; 64], next_claim: &mut usize| -> usize {
            while *next_claim < n_cuts {
                let c = *next_claim;
                *next_claim += 1;
                if !claimed[c] {
                    claimed[c] = true;
                    return c;
                }
            }
            usize::MAX
        };

        let mut aux_cid = [usize::MAX; 3];
        let mut aux_j = [0u32; 3];
        for l in 0..3 {
            let c = claim_cut(&mut claimed, &mut next_claim);
            aux_cid[l] = c;
            if c != usize::MAX {
                aux_j[l] = cuts[c];
            }
        }

        let mut j0 = mark;
        let mut at0 = 0usize;
        let mut rounds = 0usize;
        let mut active_aux_ops = 0usize;

        while at0 < n {
            rounds += 1;
            assert!(
                rounds <= (n + m as usize) * 2 + 64,
                "1P-Stage256K loop: n={n} m={m} mark={mark} j0={j0} at0={at0} n_cuts={n_cuts}"
            );
            // Step Lane 0 (`mark` frontier) directly into `out[at0..]`
            let e0 = self.edges[j0 as usize];
            let c0 = (e0 >> bits) as usize;
            j0 = e0 & mask;
            let l0 = self.lens[c0];
            if l0 != LONG {
                out[at0..at0 + INLINE].copy_from_slice(&self.inline16[c0]);
                at0 += usize::from(l0);
            } else {
                let w = c0 - 256;
                let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                let take = word.len().min(n - at0);
                out[at0..at0 + take].copy_from_slice(&word[..take]);
                at0 += take;
            }

            // Step up to 3 auxiliary lanes in parallel ONLY while their `per_slot` 256K-share has room!
            for l in 0..3 {
                let cid = aux_cid[l];
                if cid == usize::MAX {
                    continue;
                }
                let cur_len = slot_len[cid];
                let ej = self.edges[aux_j[l] as usize];
                let code = (ej >> bits) as usize;
                let need = usize::from(self.word_len[code]).max(INLINE);
                if cur_len + need > per_slot {
                    // STALLED by the 256 KiB L3-safe staging ceiling!
                    slot_stalled_j[cid] = aux_j[l];
                    aux_cid[l] = usize::MAX;
                    continue;
                }
                active_aux_ops += 1;
                let nj = ej & mask;
                aux_j[l] = nj;
                let base = cid * (CAP_STAGE / n_cuts) + cur_len;
                let ln = self.lens[code];
                if ln != LONG {
                    self.stage_256k[base..base + INLINE].copy_from_slice(&self.inline16[code]);
                    slot_len[cid] = cur_len + usize::from(ln);
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    self.stage_256k[base..base + word.len()].copy_from_slice(word);
                    slot_len[cid] = cur_len + word.len();
                }
                if ((nj ^ mark) & cm) == 0 {
                    slot_done[cid] = true;
                    slot_next_cid[cid] = (nj >> shift) as usize;
                    let nxt = claim_cut(&mut claimed, &mut next_claim);
                    aux_cid[l] = nxt;
                    if nxt != usize::MAX {
                        aux_j[l] = cuts[nxt];
                    }
                }
            }

            // Did Lane 0 hit a cut? Splice any completed or partially staged segment!
            while at0 < n && ((j0 ^ mark) & cm) == 0 {
                if j0 == mark {
                    out.truncate(at0.min(n));
                    while out.len() < n {
                        let take = out.len().min(n - out.len());
                        out.extend_from_within(..take);
                    }
                    return (rounds, active_aux_ops);
                }
                let hit_cid = (j0 >> shift) as usize;
                // If an active lane is currently walking `hit_cid`, stop that lane and hand off to Lane 0!
                for l in 0..3 {
                    if aux_cid[l] == hit_cid {
                        slot_stalled_j[hit_cid] = aux_j[l];
                        let nxt = claim_cut(&mut claimed, &mut next_claim);
                        aux_cid[l] = nxt;
                        if nxt != usize::MAX {
                            aux_j[l] = cuts[nxt];
                        }
                    }
                }
                let buffered = slot_len[hit_cid];
                if buffered > 0 {
                    let base = hit_cid * (CAP_STAGE / n_cuts);
                    let take = buffered.min(n - at0);
                    out[at0..at0 + take].copy_from_slice(&self.stage_256k[base..base + take]);
                    at0 += take;
                    slot_len[hit_cid] = 0;
                }
                if slot_done[hit_cid] {
                    j0 = cuts[slot_next_cid[hit_cid]];
                } else if slot_stalled_j[hit_cid] != u32::MAX {
                    j0 = slot_stalled_j[hit_cid];
                    break;
                } else {
                    // Never started (all aux lanes stalled on other cuts!)
                    claimed[hit_cid] = true;
                    break;
                }
            }
            if j0 == mark {
                break;
            }
        }
        out.truncate(at0.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        (rounds, active_aux_ops)
    }

    /// Arm 3 (`2P-WS8x32`: Two-Pass Work-Stealing Zero-Copy MLP, `mu = 8`, `S = 32` cuts, `0 B` heap staging!)
    /// Returns `(rounds_4a, rounds_4b)` (`us_4a` and `us_4b` also measurable).
    pub fn emit_2p_ws8(&self, n: usize, m: u32, mark: u32, bits: u32, out: &mut Vec<u8>) -> (usize, usize, u64, u64) {
        if m < 2048 {
            self.emit_serial(n, m, mark, bits, out);
            return (m as usize, 0, 0, 0);
        }
        let (shift, cm, cuts, n_cuts) = Self::cut_params(m, mark, bits, 5); // 17..=32 cuts!
        if n_cuts <= 2 {
            self.emit_serial(n, m, mark, bits, out);
            return (m as usize, 0, 0, 0);
        }
        let mask = (1u32 << bits) - 1;
        let edges = self.edges.as_slice();
        let wlen = self.word_len.as_slice();

        // ---------------------------------------------------------------------
        // Sub-pass 4a: 8-Lane Branchless Length-Only Work-Stealing Cut Discovery
        // ---------------------------------------------------------------------
        let t4a = Instant::now();
        let mut seg_len = [0usize; 64];
        let mut seg_next = [0usize; 64];
        let mut next_cid = 0usize;

        let mut lane_cid = [usize::MAX; 8];
        let mut lane_j = [0u32; 8];
        let mut lane_acc = [0usize; 8];
        let mut active = 0usize;
        for l in 0..8 {
            if next_cid < n_cuts {
                let c = next_cid;
                next_cid += 1;
                lane_cid[l] = c;
                lane_j[l] = cuts[c];
                active += 1;
            }
        }

        let mut rounds_4a = 0usize;
        while active == 8 {
            rounds_4a += 1;
            let e0 = edges[lane_j[0] as usize];
            let e1 = edges[lane_j[1] as usize];
            let e2 = edges[lane_j[2] as usize];
            let e3 = edges[lane_j[3] as usize];
            let e4 = edges[lane_j[4] as usize];
            let e5 = edges[lane_j[5] as usize];
            let e6 = edges[lane_j[6] as usize];
            let e7 = edges[lane_j[7] as usize];

            lane_acc[0] += wlen[(e0 >> bits) as usize] as usize;
            lane_acc[1] += wlen[(e1 >> bits) as usize] as usize;
            lane_acc[2] += wlen[(e2 >> bits) as usize] as usize;
            lane_acc[3] += wlen[(e3 >> bits) as usize] as usize;
            lane_acc[4] += wlen[(e4 >> bits) as usize] as usize;
            lane_acc[5] += wlen[(e5 >> bits) as usize] as usize;
            lane_acc[6] += wlen[(e6 >> bits) as usize] as usize;
            lane_acc[7] += wlen[(e7 >> bits) as usize] as usize;

            let n0 = e0 & mask;
            let n1 = e1 & mask;
            let n2 = e2 & mask;
            let n3 = e3 & mask;
            let n4 = e4 & mask;
            let n5 = e5 & mask;
            let n6 = e6 & mask;
            let n7 = e7 & mask;
            lane_j[0] = n0;
            lane_j[1] = n1;
            lane_j[2] = n2;
            lane_j[3] = n3;
            lane_j[4] = n4;
            lane_j[5] = n5;
            lane_j[6] = n6;
            lane_j[7] = n7;

            if (((n0 ^ mark) & cm) == 0)
                | (((n1 ^ mark) & cm) == 0)
                | (((n2 ^ mark) & cm) == 0)
                | (((n3 ^ mark) & cm) == 0)
                | (((n4 ^ mark) & cm) == 0)
                | (((n5 ^ mark) & cm) == 0)
                | (((n6 ^ mark) & cm) == 0)
                | (((n7 ^ mark) & cm) == 0)
            {
                for l in 0..8 {
                    if ((lane_j[l] ^ mark) & cm) == 0 {
                        let finished_c = lane_cid[l];
                        seg_len[finished_c] = lane_acc[l];
                        seg_next[finished_c] = (lane_j[l] >> shift) as usize;
                        if next_cid < n_cuts {
                            let nc = next_cid;
                            next_cid += 1;
                            lane_cid[l] = nc;
                            lane_j[l] = cuts[nc];
                            lane_acc[l] = 0;
                        } else {
                            lane_cid[l] = usize::MAX;
                            active -= 1;
                        }
                    }
                }
            }
        }
        while active > 0 {
            rounds_4a += 1;
            for l in 0..8 {
                if lane_cid[l] == usize::MAX {
                    continue;
                }
                let e = edges[lane_j[l] as usize];
                lane_acc[l] += wlen[(e >> bits) as usize] as usize;
                let nj = e & mask;
                lane_j[l] = nj;
                if ((nj ^ mark) & cm) == 0 {
                    let finished_c = lane_cid[l];
                    seg_len[finished_c] = lane_acc[l];
                    seg_next[finished_c] = (nj >> shift) as usize;
                    if next_cid < n_cuts {
                        let nc = next_cid;
                        next_cid += 1;
                        lane_cid[l] = nc;
                        lane_j[l] = cuts[nc];
                        lane_acc[l] = 0;
                    } else {
                        lane_cid[l] = usize::MAX;
                        active -= 1;
                    }
                }
            }
        }
        let ns_4a = t4a.elapsed().as_nanos() as u64;

        // ---------------------------------------------------------------------
        // O(S) Stack Prefix-Sum Along `mark`'s Cycle (filters multi-cycle columns!)
        // ---------------------------------------------------------------------
        let t4b = Instant::now();
        let mark_cid = (mark >> shift) as usize;
        let mut cyc_cuts = [0usize; 64];
        let mut cyc_start = [0usize; 64];
        let mut cyc_end = [0usize; 64];
        let mut n_cyc = 0usize;
        let mut cur_c = mark_cid;
        let mut prefix_at = 0usize;
        loop {
            if prefix_at >= n || n_cyc >= n_cuts {
                break;
            }
            let sl = seg_len[cur_c];
            let end_at = (prefix_at + sl).min(n);
            cyc_cuts[n_cyc] = cur_c;
            cyc_start[n_cyc] = prefix_at;
            cyc_end[n_cyc] = end_at;
            n_cyc += 1;
            prefix_at = end_at;
            cur_c = seg_next[cur_c];
            if cur_c == mark_cid {
                break;
            }
        }

        // ---------------------------------------------------------------------
        // Sub-pass 4b: 8-Lane Work-Stealing Zero-Copy In-Place Emit Into `out`!
        // ---------------------------------------------------------------------
        let mut next_seg = 0usize;
        let mut e_j = [0u32; 8];
        let mut e_at = [usize::MAX; 8];
        let mut e_end = [0usize; 8];
        let mut e_active = 0usize;
        for l in 0..8 {
            if next_seg < n_cyc {
                let s_idx = next_seg;
                next_seg += 1;
                e_j[l] = cuts[cyc_cuts[s_idx]];
                e_at[l] = cyc_start[s_idx];
                e_end[l] = cyc_end[s_idx];
                e_active += 1;
            }
        }

        let lens = self.lens.as_slice();
        let inl = self.inline16.as_slice();
        let mut rounds_4b = 0usize;

        while e_active == 8 {
            rounds_4b += 1;
            let v0 = edges[e_j[0] as usize];
            let v1 = edges[e_j[1] as usize];
            let v2 = edges[e_j[2] as usize];
            let v3 = edges[e_j[3] as usize];
            let v4 = edges[e_j[4] as usize];
            let v5 = edges[e_j[5] as usize];
            let v6 = edges[e_j[6] as usize];
            let v7 = edges[e_j[7] as usize];

            let raw = [v0, v1, v2, v3, v4, v5, v6, v7];
            for l in 0..8 {
                let edge = raw[l];
                let code = (edge >> bits) as usize;
                e_j[l] = edge & mask;
                let len = lens[code];
                let at = e_at[l];
                let end_at = e_end[l];
                if len != LONG && at + INLINE <= end_at {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
                    e_at[l] = at + usize::from(len);
                } else if len != LONG {
                    let take = usize::from(len).min(end_at - at);
                    out[at..at + take].copy_from_slice(&inl[code][..take]);
                    e_at[l] = at + take;
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(end_at - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    e_at[l] = at + take;
                }
                if e_at[l] >= end_at {
                    if next_seg < n_cyc {
                        let s_idx = next_seg;
                        next_seg += 1;
                        e_j[l] = cuts[cyc_cuts[s_idx]];
                        e_at[l] = cyc_start[s_idx];
                        e_end[l] = cyc_end[s_idx];
                    } else {
                        e_active -= 1;
                        e_at[l] = usize::MAX;
                    }
                }
            }
        }
        while e_active > 0 {
            rounds_4b += 1;
            for l in 0..8 {
                if e_at[l] == usize::MAX {
                    continue;
                }
                let edge = edges[e_j[l] as usize];
                let code = (edge >> bits) as usize;
                e_j[l] = edge & mask;
                let len = lens[code];
                let at = e_at[l];
                let end_at = e_end[l];
                if len != LONG && at + INLINE <= end_at {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
                    e_at[l] = at + usize::from(len);
                } else if len != LONG {
                    let take = usize::from(len).min(end_at - at);
                    out[at..at + take].copy_from_slice(&inl[code][..take]);
                    e_at[l] = at + take;
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(end_at - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    e_at[l] = at + take;
                }
                if e_at[l] >= end_at {
                    if next_seg < n_cyc {
                        let s_idx = next_seg;
                        next_seg += 1;
                        e_j[l] = cuts[cyc_cuts[s_idx]];
                        e_at[l] = cyc_start[s_idx];
                        e_end[l] = cyc_end[s_idx];
                    } else {
                        e_active -= 1;
                        e_at[l] = usize::MAX;
                    }
                }
            }
        }
        out.truncate(prefix_at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        let ns_4b = t4b.elapsed().as_nanos() as u64;
        (rounds_4a, rounds_4b, ns_4a, ns_4b)
    }
}

// ============================================================================
// Exact Round-by-Round Parallel L2-Miss Depth (`D_past_M2`) Simulator for `2P-WS8`
// ============================================================================
struct FastLru {
    cap: usize,
    size: usize,
    head: u32,
    tail: u32,
    prev: Vec<u32>,
    next: Vec<u32>,
    present: Vec<bool>,
}
impl FastLru {
    fn new(cap: usize, universe: usize) -> Self {
        Self {
            cap,
            size: 0,
            head: u32::MAX,
            tail: u32::MAX,
            prev: vec![u32::MAX; universe],
            next: vec![u32::MAX; universe],
            present: vec![false; universe],
        }
    }
    fn access(&mut self, line: u32) -> bool {
        let x = line as usize;
        if self.present[x] {
            if self.head != line {
                let p = self.prev[x];
                let n = self.next[x];
                if p != u32::MAX {
                    self.next[p as usize] = n;
                }
                if n != u32::MAX {
                    self.prev[n as usize] = p;
                } else {
                    self.tail = p;
                }
                self.prev[x] = u32::MAX;
                self.next[x] = self.head;
                self.prev[self.head as usize] = line;
                self.head = line;
            }
            true
        } else {
            if self.size == self.cap {
                let ev = self.tail as usize;
                self.present[ev] = false;
                let p = self.prev[ev];
                self.tail = p;
                if p != u32::MAX {
                    self.next[p as usize] = u32::MAX;
                } else {
                    self.head = u32::MAX;
                }
                self.size -= 1;
            }
            self.present[x] = true;
            self.prev[x] = u32::MAX;
            self.next[x] = self.head;
            if self.head != u32::MAX {
                self.prev[self.head as usize] = line;
            } else {
                self.tail = line;
            }
            self.head = line;
            self.size += 1;
            false
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Ws8BmcpProfile {
    q_rand_4a: usize,
    q_rand_4b: usize,
    d_serial_4a: usize,
    d_serial_4b: usize,
    d_past_l2_crit_4a: usize,
    d_past_l2_crit_4b: usize,
    d_past_l2_rounds_4a: usize,
    d_past_l2_rounds_4b: usize,
    c_miss_256k: usize,
    c_miss_2048k: usize,
    n_cuts: usize,
    n_cyc_cuts: usize,
}

/// Counts exact BMCP costs for `2P-WS8` starting from the identical end-of-Phase-3
/// L2 (`256 KiB`) / L3 (`2,048 KiB` 10-worker slice) state used in Cycle 6's `UPPER` trace!
fn simulate_2p_ws8_dep_l2(dk: &DietedKernel, n: usize, m: u32, mark: u32, bits: u32) -> Ws8BmcpProfile {
    let base_edges = 0u32;
    let n_edge_lines = (m as usize).div_ceil(16) as u32 + 2;
    let base_lens = base_edges + n_edge_lines;
    let n_lens_lines = dk.lens.len().div_ceil(64) as u32 + 2;
    let base_wlen = base_lens + n_lens_lines;
    let n_wlen_lines = dk.word_len.len().div_ceil(32) as u32 + 2;
    let base_inl = base_wlen + n_wlen_lines;
    let n_inl_lines = dk.inline16.len().div_ceil(4) as u32 + 2;
    let base_spell = base_inl + n_inl_lines;
    let n_spell_lines = dk.spelling.len().div_ceil(64) as u32 + 2;
    let base_out = base_spell + n_spell_lines;
    let n_out_lines = (n + 32).div_ceil(64) as u32 + 2;
    let universe = (base_out + n_out_lines) as usize;

    let mut l2 = FastLru::new(4096, universe); // 256 KiB L2
    let mut l3 = FastLru::new(32768, universe); // 2048 KiB L3 slice (10 workers on 20 MiB L3)

    // Identical end-of-Phase-3 cache state as Cycle 6 `profile_emit_caches`:
    for ln in 0..n_out_lines.min((n.div_ceil(64)) as u32) {
        l3.access(base_out + ln);
    }
    for ln in 0..n_inl_lines - 2 {
        l2.access(base_inl + ln);
        l3.access(base_inl + ln);
    }
    for ln in 0..n_lens_lines - 2 {
        l2.access(base_lens + ln);
        l3.access(base_lens + ln);
    }
    for ln in 0..n_wlen_lines - 2 {
        l2.access(base_wlen + ln);
        l3.access(base_wlen + ln);
    }
    for ln in 0..(m as usize).div_ceil(16) as u32 {
        l2.access(base_edges + ln);
        l3.access(base_edges + ln);
    }

    let mut prof = Ws8BmcpProfile::default();
    let (shift, cm, cuts, n_cuts) = DietedKernel::cut_params(m, mark, bits, 5);
    prof.n_cuts = n_cuts;
    let mask = (1u32 << bits) - 1;

    // Pass 4a trace
    let mut seg_len = [0usize; 64];
    let mut seg_next = [0usize; 64];
    let mut next_cid = 0usize;
    let mut lane_cid = [usize::MAX; 8];
    let mut lane_j = [0u32; 8];
    let mut lane_acc = [0usize; 8];
    let mut lane_dep_misses_4a = [0usize; 8];
    let mut active = 0usize;
    for l in 0..8 {
        if next_cid < n_cuts {
            let c = next_cid;
            next_cid += 1;
            lane_cid[l] = c;
            lane_j[l] = cuts[c];
            active += 1;
        }
    }
    while active > 0 {
        prof.d_serial_4a += 1;
        let mut round_miss = false;
        for l in 0..8 {
            if lane_cid[l] == usize::MAX {
                continue;
            }
            prof.q_rand_4a += 1;
            let eln = base_edges + (lane_j[l] / 16);
            if !l2.access(eln) {
                lane_dep_misses_4a[l] += 1;
                prof.c_miss_256k += 1;
                round_miss = true;
            }
            if !l3.access(eln) {
                prof.c_miss_2048k += 1;
            }
            let e = dk.edges[lane_j[l] as usize];
            let code = (e >> bits) as usize;
            let wln = base_wlen + (code / 32) as u32;
            if !l2.access(wln) {
                prof.c_miss_256k += 1;
            }
            if !l3.access(wln) {
                prof.c_miss_2048k += 1;
            }
            lane_acc[l] += dk.word_len[code] as usize;
            let nj = e & mask;
            lane_j[l] = nj;
            if ((nj ^ mark) & cm) == 0 {
                let fc = lane_cid[l];
                seg_len[fc] = lane_acc[l];
                seg_next[fc] = (nj >> shift) as usize;
                if next_cid < n_cuts {
                    let nc = next_cid;
                    next_cid += 1;
                    lane_cid[l] = nc;
                    lane_j[l] = cuts[nc];
                    lane_acc[l] = 0;
                } else {
                    lane_cid[l] = usize::MAX;
                    active -= 1;
                }
            }
        }
        if round_miss {
            prof.d_past_l2_rounds_4a += 1;
        }
    }
    prof.d_past_l2_crit_4a = *lane_dep_misses_4a.iter().max().unwrap();

    // Prefix sum
    let mark_cid = (mark >> shift) as usize;
    let mut cyc_cuts = [0usize; 64];
    let mut cyc_start = [0usize; 64];
    let mut cyc_end = [0usize; 64];
    let mut n_cyc = 0usize;
    let mut cur_c = mark_cid;
    let mut prefix_at = 0usize;
    while prefix_at < n && n_cyc < n_cuts {
        let end_at = (prefix_at + seg_len[cur_c]).min(n);
        cyc_cuts[n_cyc] = cur_c;
        cyc_start[n_cyc] = prefix_at;
        cyc_end[n_cyc] = end_at;
        n_cyc += 1;
        prefix_at = end_at;
        cur_c = seg_next[cur_c];
        if cur_c == mark_cid {
            break;
        }
    }
    prof.n_cyc_cuts = n_cyc;

    // Pass 4b trace
    let mut next_seg = 0usize;
    let mut e_j = [0u32; 8];
    let mut e_at = [usize::MAX; 8];
    let mut e_end = [0usize; 8];
    let mut lane_dep_misses_4b = [0usize; 8];
    let mut e_active = 0usize;
    for l in 0..8 {
        if next_seg < n_cyc {
            let s = next_seg;
            next_seg += 1;
            e_j[l] = cuts[cyc_cuts[s]];
            e_at[l] = cyc_start[s];
            e_end[l] = cyc_end[s];
            e_active += 1;
        }
    }
    while e_active > 0 {
        prof.d_serial_4b += 1;
        let mut round_miss = false;
        for l in 0..8 {
            if e_at[l] == usize::MAX {
                continue;
            }
            prof.q_rand_4b += 1;
            let eln = base_edges + (e_j[l] / 16);
            if !l2.access(eln) {
                lane_dep_misses_4b[l] += 1;
                prof.c_miss_256k += 1;
                round_miss = true;
            }
            if !l3.access(eln) {
                prof.c_miss_2048k += 1;
            }
            let edge = dk.edges[e_j[l] as usize];
            let code = (edge >> bits) as usize;
            e_j[l] = edge & mask;
            let lln = base_lens + (code / 64) as u32;
            if !l2.access(lln) {
                prof.c_miss_256k += 1;
            }
            if !l3.access(lln) {
                prof.c_miss_2048k += 1;
            }
            let len = dk.lens[code];
            let step_b = if len != LONG {
                let iln = base_inl + (code / 4) as u32;
                if !l2.access(iln) {
                    prof.c_miss_256k += 1;
                }
                if !l3.access(iln) {
                    prof.c_miss_2048k += 1;
                }
                usize::from(len).min(e_end[l] - e_at[l])
            } else {
                let w = code - 256;
                let s0 = dk.offsets[w] as usize;
                let s1 = dk.offsets[w + 1] as usize;
                let take = (s1 - s0).min(e_end[l] - e_at[l]);
                for sln in (s0 / 64)..=((s0 + take.saturating_sub(1)) / 64) {
                    let g = base_spell + sln as u32;
                    if !l2.access(g) {
                        prof.c_miss_256k += 1;
                    }
                    if !l3.access(g) {
                        prof.c_miss_2048k += 1;
                    }
                }
                take
            };
            let o0 = e_at[l] / 64;
            let o1 = (e_at[l] + step_b.saturating_sub(1).max(15)) / 64;
            for oln in o0..=o1 {
                let g = base_out + oln as u32;
                if !l2.access(g) {
                    prof.c_miss_256k += 1;
                }
                if !l3.access(g) {
                    prof.c_miss_2048k += 1;
                }
            }
            e_at[l] += step_b;
            if e_at[l] >= e_end[l] {
                if next_seg < n_cyc {
                    let s = next_seg;
                    next_seg += 1;
                    e_j[l] = cuts[cyc_cuts[s]];
                    e_at[l] = cyc_start[s];
                    e_end[l] = cyc_end[s];
                } else {
                    e_at[l] = usize::MAX;
                    e_active -= 1;
                }
            }
        }
        if round_miss {
            prof.d_past_l2_rounds_4b += 1;
        }
    }
    prof.d_past_l2_crit_4b = *lane_dep_misses_4b.iter().max().unwrap();
    prof
}

fn verify_gate_g1(dk: &mut DietedKernel) -> usize {
    let mut out1 = Vec::new();
    let mut out2 = Vec::new();
    let mut cases = 0usize;
    for (alpha, max_n) in [(2u32, 9usize), (3u32, 6usize)] {
        for n in 1..=max_n {
            for code in 0..alpha.pow(n as u32) {
                let mut tmp = code;
                let ll: Vec<u8> = (0..n)
                    .map(|_| {
                        let d = (tmp % alpha) as u8;
                        tmp /= alpha;
                        d
                    })
                    .collect();
                let recs = records_of(&ll);
                for p in 0..n as u32 {
                    let want = plain_invert(&ll, p);
                    let (m, mark, bits) = dk.build_table(&recs, n, p, &mut out1);
                    dk.emit_2p_ws8(n, m, mark, bits, &mut out1);
                    assert_eq!(out1, want);
                    cases += 1;
                }
            }
        }
    }
    let mut rng = 0xD1B5_4A32_D192_ED03u64;
    let mut nxt = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    // Test medium/large multi-cycle & multi-epoch inputs (`n = 2500 .. 180,000`) where `m >= 2048` activates `2P-WS8`!
    for t in 0..200usize {
        let n = if t < 190 {
            2500 + (nxt() as usize % 8000)
        } else {
            70_000 + (t - 190) * 12_000
        };
        let alpha = ((nxt() % 6) + 2) as u8;
        let mut ll = Vec::with_capacity(n);
        while ll.len() < n {
            let sym = (nxt() as u8) % alpha;
            let w = if nxt() % 5 == 0 {
                ((nxt() % 40) + 17) as usize
            } else {
                ((nxt() % 12) + 1) as usize
            };
            let take = w.min(n - ll.len());
            ll.resize(ll.len() + take, sym);
        }
        let p = (nxt() as usize % n) as u32;
        let want = plain_invert(&ll, p);
        let recs = records_of(&ll);
        let (m, mark, bits) = dk.build_table(&recs, n, p, &mut out1);
        dk.emit_2p_ws8(n, m, mark, bits, &mut out1);
        assert_eq!(out1, want, "2P-WS8 mismatch case {t} n={n} m={m}");

        let (m2, mark2, bits2) = dk.build_table(&recs, n, p, &mut out2);
        dk.emit_1p_bounded256k(n, m2, mark2, bits2, &mut out2);
        assert_eq!(out2, want, "1P-Stage256K mismatch case {t} n={n} m={m2}");
        cases += 1;
    }
    cases
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/antigravity/admitted-bwt".to_string());
    let rounds: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(21);
    let kernels = Kernels::detect();

    let mut dk = DietedKernel::default();
    let g1 = verify_gate_g1(&mut dk);
    println!("PASS Gate G1: {} small, multi-cycle, and multi-epoch cases verified byte-exact.", g1);

    let blocks: [(&str, usize, u32, usize); 6] = [
        ("block-0.L", 899_931, 264_045, 25_533),
        ("block-2.L", 900_000, 320_807, 37_778),
        ("block-85.L", 899_935, 325_088, 47_553),
        ("block-143.L", 899_869, 802_868, 94_739),
        ("block-210.L", 899_959, 791_429, 89_106),
        ("block-207.L", 899_998, 205_180, 51_240),
    ];

    let mut master_open = OpenCells::default();
    let mut dk_clean = DietedKernel::default();
    let mut out_m = Vec::with_capacity(900_032);
    let mut out_d = Vec::with_capacity(900_032);

    println!(
        "\n=== CYCLE 8 CONFRONTATION ({} PAIRED ROUNDS VS `origin/master` `OpenCells` + BMCP TRACE) ===",
        rounds
    );
    println!(
        "{:<11} | {:>8} | {:>8} ({:>6}) | {:>8} ({:>6}) [4a/4b us] | {:>9} -> {:>9} ({:>5}) | {:>7} | {:>6}",
        "Block",
        "Master",
        "Diet_mu1",
        "/Mst",
        "2P-WS8",
        "/Mst",
        "Diet_DpL2",
        "WS8_Crit",
        "Ratio",
        "L3_Miss",
        "1P_Aux"
    );
    println!("{}", "-".repeat(124));

    let mut json_rows = Vec::new();
    for &(name, n_exp, p, diet_dep_l2) in &blocks {
        let ll = fs::read(format!("{}/{}", dir, name)).expect("read");
        let bz2 = craft_bz2_block(&ll, p, kernels);
        let (runs_obj, n_dec, p_dec) = extract_runs(&bz2, kernels);
        assert_eq!((n_dec, p_dec), (n_exp, p));

        assert!(master_open.invert_runs(&runs_obj, n_exp, p, &mut out_m, kernels));

        // Verify byte-exactness of Arm 1, Arm 2, Arm 3 against `origin/master` OpenCells!
        let (m, mark, bits) = dk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_d);
        dk_clean.emit_serial(n_exp, m, mark, bits, &mut out_d);
        assert_eq!(out_d, out_m, "Arm 1 Diet_mu1 mismatch on {name}");

        let (_, _, _) = dk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_d);
        let (r1p, aux1p) = dk_clean.emit_1p_bounded256k(n_exp, m, mark, bits, &mut out_d);
        assert_eq!(out_d, out_m, "Arm 2 1P-Stage256K mismatch on {name}");
        let aux_cov_pct = 100.0 * aux1p as f64 / m as f64;
        let _ = r1p;

        let (_, _, _) = dk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_d);
        let (r4a, r4b, _, _) = dk_clean.emit_2p_ws8(n_exp, m, mark, bits, &mut out_d);
        assert_eq!(out_d, out_m, "Arm 3 2P-WS8 mismatch on {name}");

        let bp = simulate_2p_ws8_dep_l2(&dk_clean, n_exp, m, mark, bits);
        let ws8_dpl2_crit = bp.d_past_l2_crit_4a + bp.d_past_l2_crit_4b;
        let ws8_dpl2_rounds = bp.d_past_l2_rounds_4a + bp.d_past_l2_rounds_4b;

        // Paired rotated timing across Master, Diet_mu1, and 2P-WS8
        let mut times = [Vec::with_capacity(rounds), Vec::with_capacity(rounds), Vec::with_capacity(rounds)];
        let mut s4a_vec = Vec::with_capacity(rounds);
        let mut s4b_vec = Vec::with_capacity(rounds);
        for r in 0..(rounds + 3) {
            for rot in 0..3 {
                let arm = (r + rot) % 3;
                let t0 = Instant::now();
                match arm {
                    0 => {
                        master_open.invert_runs(&runs_obj, n_exp, p, &mut out_m, kernels);
                    }
                    1 => {
                        let (mm, mk, bt) = dk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_d);
                        dk_clean.emit_serial(n_exp, mm, mk, bt, &mut out_d);
                    }
                    _ => {
                        let (mm, mk, bt) = dk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_d);
                        let (_, _, n4a, n4b) = dk_clean.emit_2p_ws8(n_exp, mm, mk, bt, &mut out_d);
                        if r >= 3 {
                            s4a_vec.push(n4a);
                            s4b_vec.push(n4b);
                        }
                    }
                }
                let dt = t0.elapsed().as_nanos() as u64;
                if r >= 3 {
                    times[arm].push(dt);
                }
            }
        }
        for v in &mut times {
            v.sort_unstable();
        }
        s4a_vec.sort_unstable();
        s4b_vec.sort_unstable();
        let med_mst = times[0][rounds / 2] as f64 / 1e3;
        let med_diet = times[1][rounds / 2] as f64 / 1e3;
        let med_ws8 = times[2][rounds / 2] as f64 / 1e3;
        let med_4a = s4a_vec[rounds / 2] as f64 / 1e3;
        let med_4b = s4b_vec[rounds / 2] as f64 / 1e3;

        println!(
            "{:<11} | {:>6.1}us | {:>6.1}us ({:>5.3}x) | {:>6.1}us ({:>5.3}x) [{:>4.0}/{:>4.0}] | {:>9} -> {:>9} ({:>4.2}x) | {:>7} | {:>5.1}%",
            name,
            med_mst,
            med_diet,
            med_diet / med_mst,
            med_ws8,
            med_ws8 / med_mst,
            med_4a,
            med_4b,
            diet_dep_l2,
            ws8_dpl2_crit,
            ws8_dpl2_crit as f64 / diet_dep_l2 as f64,
            bp.c_miss_2048k,
            aux_cov_pct
        );

        json_rows.push(format!(
            "    {{\"block\": \"{}\", \"m\": {}, \"n_cuts\": {}, \"n_cyc_cuts\": {}, \"master_us\": {:.1}, \"diet_mu1_us\": {:.1}, \"ws8_us\": {:.1}, \"ws8_pass4a_us\": {:.1}, \"ws8_pass4b_us\": {:.1}, \"q_rand_total\": {}, \"rounds_4a\": {}, \"rounds_4b\": {}, \"d_serial_total\": {}, \"diet_dep_l2\": {}, \"ws8_dep_l2_crit_4a\": {}, \"ws8_dep_l2_crit_4b\": {}, \"ws8_dep_l2_crit_total\": {}, \"ws8_dep_l2_rounds_total\": {}, \"ws8_c_miss_256k\": {}, \"ws8_c_miss_2048k\": {}, \"stage1p_aux_coverage_pct\": {:.2}}}",
            name, m, bp.n_cuts, bp.n_cyc_cuts, med_mst, med_diet, med_ws8, med_4a, med_4b,
            bp.q_rand_4a + bp.q_rand_4b, r4a, r4b, r4a + r4b,
            diet_dep_l2, bp.d_past_l2_crit_4a, bp.d_past_l2_crit_4b, ws8_dpl2_crit,
            ws8_dpl2_rounds, bp.c_miss_256k, bp.c_miss_2048k, aux_cov_pct
        ));
    }

    // Clear stage_256k (only Arm 2 used it) and print exact persistent heap of Diet_mu1 / 2P-WS8!
    dk_clean.stage_256k = Vec::new();
    let ws8_heap = dk_clean.base_heap_bytes();
    println!(
        "\nPersistent Scratch Heap of `Diet_mu1` & `2P-WS8` (`0 B` staging!): {:.1} KiB (w/ out = {:.1} KiB <= 2048 KiB -> 10w = {:.1} MiB <= 20 MiB L3!)",
        ws8_heap as f64 / 1024.0,
        (ws8_heap + 900_016) as f64 / 1024.0,
        10.0 * (ws8_heap + 900_016) as f64 / (1024.0 * 1024.0)
    );

    if let Some(json_path) = std::env::args().nth(3) {
        let j = format!(
            "{{\n  \"g1_cases\": {},\n  \"ws8_retained_heap_bytes\": {},\n  \"blocks\": [\n{}\n  ]\n}}\n",
            g1,
            ws8_heap,
            json_rows.join(",\n")
        );
        fs::write(json_path, j).expect("write json");
    }
}
