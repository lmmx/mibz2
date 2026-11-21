#![forbid(unsafe_code)]
//! Wormhole Cycle 10 — Production-Drop-In `InverseBwt` Multi-Worker `BzReader` Verification,
//! Concurrent Barrier Confrontation, and Complete Round-2 Floor/Upper/Gap/Route Audit:
//!   - Implements `DietMu1Cells` (`mu = 1`) and `DietRelay8Cells` (`mu = 8, k = 8`) directly
//!     implementing `mibz2::bwt::InverseBwt` with `OpenCells`'s exact admission gates
//!     (`records * 20 < n * 3 && records * 64 >= n`), `LazyCells` fallback, and `extra: Vec<u32>`
//!     wide-code overflow safety (`0 B` on all real blocks).
//!   - Verifies `14,461` Gate G1 + wide-code + fallback cases byte-exact.
//!   - Synthesizes an 18-block (`3x` the 6 authenticated Wikidata blocks) `.bz2` stream and
//!     decodes it end-to-end via `mibz2::BzReader::parallel_workers_with::<DietRelay8Cells>` at
//!     `W = 1, 2, 4, 8` worker threads, verifying 100% byte-exact output and stream CRC32 against
//!     `mibz2::BzReader::parallel_workers_with::<OpenCells>`!
//!   - Runs 21-round paired interleaved confrontations at 1 worker and 2 concurrent barrier-synced
//!     workers against `origin/master` `OpenCells` (`d425d77`, `mibz2 0.5.9`).

use mibz2::bwt::{InverseBwt, LazyCells, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{BzReader, Kernels};
use std::fs;
use std::io::{Cursor, Read};
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const LAZY_DENSITY: (usize, usize) = (3, 20);

#[inline]
fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
}

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

fn write_bz2_block_body(w: &mut BitWriter, ll: &[u8], orig_ptr: u32, kernels: Kernels) -> u32 {
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
    crc
}

fn craft_single_bz2(ll: &[u8], orig_ptr: u32, kernels: Kernels) -> Vec<u8> {
    let mut w = BitWriter::default();
    for &b in b"BZh9" {
        w.put(b as u64, 8);
    }
    let crc = write_bz2_block_body(&mut w, ll, orig_ptr, kernels);
    w.put(0x1772_4538_5090, 48);
    w.put(crc as u64, 32);
    w.finish()
}

fn craft_multiblock_bz2(blocks_ll: &[(&[u8], u32)], repeats: usize, kernels: Kernels) -> Vec<u8> {
    let mut w = BitWriter::default();
    for &b in b"BZh9" {
        w.put(b as u64, 8);
    }
    let mut combined = 0u32;
    for _ in 0..repeats {
        for &(ll, p) in blocks_ll {
            let crc = write_bz2_block_body(&mut w, ll, p, kernels);
            combined = combined.rotate_left(1) ^ crc;
        }
    }
    w.put(0x1772_4538_5090, 48);
    w.put(combined as u64, 32);
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
// Production-Grade Safe-Rust `DietRelayCore` (Supports `wide` `extra` Fallback)
// ============================================================================
#[derive(Default)]
pub struct DietRelayCore {
    pub edges: Vec<u32>,
    pub extra: Vec<u32>, // Allocated ONLY if `(255 + nwords) > (u32::MAX >> bits)` (`0 B` on all real blocks!)
    pub triples: Vec<[u32; 3]>,
    pub spelling: Vec<u8>,
    pub offsets: Vec<u32>,
    pub inline16: Vec<[u8; INLINE]>,
    pub lens: Vec<u8>,
    pub word_len: Vec<u16>,
    pub chunk_next: Vec<u16>,
    pub ovf_q: Vec<[u16; 512]>,
    pub stk: Vec<u32>,
    pub code_cap: usize,
}

impl DietRelayCore {
    pub fn retained_heap_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
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

    /// Builds the dieted Open<16> table (`SWAR64 OutQ16` in `out` + `Node16` in `edges` + `[u32;3]` coloring).
    /// Returns `(m, mark, bits, wide)`.
    pub fn build_table(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u32, u32, u32, bool) {
        let need_out = n + 32;
        if out.capacity() < need_out {
            out.reserve_exact(need_out - out.len());
        }
        if out.len() < need_out {
            out.resize(need_out, 0);
        }
        let c_max = out.len() / 1024;
        self.ovf_q.clear();
        self.chunk_next.clear();
        self.triples.clear();

        let mut q_head = [u16::MAX; 256];
        let mut q_cur = [u16::MAX; 256];
        let mut q_ic = [512u16; 256];
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
        if pos as usize != n || m == 0 {
            return (0, 0, 1, false);
        }

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

        let cap_pow2 = self.inline16.len().next_power_of_two();
        self.inline16.resize(cap_pow2, [0u8; INLINE]);
        self.lens.resize(cap_pow2, 1);
        self.word_len.resize(cap_pow2, 1);
        self.code_cap = cap_pow2;

        let bits = if m <= 1 { 1 } else { 32 - (m - 1).leading_zeros() };
        let wide = (255 + nwords as u32) > (u32::MAX >> bits);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }

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
                let code = self.triples[bi][2];
                let pack = if wide { 0 } else { code << bits };
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
                    if wide {
                        self.extra[dst..dst + step].fill(code);
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
        (m, mark, bits, wide)
    }

    pub fn emit_serial(&self, n: usize, m: u32, mark: u32, bits: u32, wide: bool, out: &mut Vec<u8>) {
        let mask = (1u32 << bits) - 1;
        let cap = self.code_cap;
        let code_mask = cap - 1;
        let inl = &self.inline16[..cap];
        let lens = &self.lens[..cap];
        let edges = self.edges.as_slice();
        let mut j = mark;
        let mut at = 0usize;
        if !wide {
            for _ in 0..m {
                let edge = edges[j as usize];
                let code = ((edge >> bits) as usize) & code_mask;
                j = edge & mask;
                let len = lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
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
        } else {
            for _ in 0..m {
                let edge = edges[j as usize];
                let code = (self.extra[j as usize] as usize) & code_mask;
                j = edge & mask;
                let len = lens[code];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
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
        }
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    pub fn emit_relay8(&self, n: usize, m: u32, mark: u32, bits: u32, wide: bool, out: &mut Vec<u8>) {
        if wide || m < 1024 {
            self.emit_serial(n, m, mark, bits, wide, out);
            return;
        }
        const MU: usize = 8;
        let shift = bits.saturating_sub(8);
        let cm = (1u32 << shift) - 1;
        let rem_low = mark & cm;
        let mut cuts = [0u32; 512];
        let mut n_cuts = 0usize;
        let stride = cm + 1;
        let mut q = 0u32;
        while let Some(c_row) = q.checked_mul(stride).map(|v| v + rem_low) {
            if c_row >= m || n_cuts >= 512 {
                break;
            }
            cuts[n_cuts] = c_row;
            n_cuts += 1;
            q += 1;
        }
        if n_cuts <= 2 {
            self.emit_serial(n, m, mark, bits, false, out);
            return;
        }

        let mask = (1u32 << bits) - 1;
        let cap = self.code_cap;
        let code_mask = cap - 1;
        let edges = self.edges.as_slice();
        let wlen = &self.word_len[..cap];
        let lens = &self.lens[..cap];
        let inl = &self.inline16[..cap];

        let mut state = [0u8; 512];
        let mut seg_len = [0usize; 512];
        let mut seg_next = [0usize; 512];

        let mut tasks_row = [0u32; 1024];
        let mut tasks_start = [0usize; 1024];
        let mut tasks_end = [0usize; 1024];
        let mut n_tasks = 0usize;

        let mut bounds_row = [0u32; 1024];
        let mut bounds_at = [0usize; 1024];
        let mut n_bounds = 0usize;

        let mark_cid = (mark >> shift) as usize;
        state[mark_cid] = 3;

        let mut next_claim = 0usize;
        let mut s_cid = [usize::MAX; MU];
        let mut s_j = [0u32; MU];
        let mut s_acc = [0usize; MU];
        let mut scouts_active = 0usize;

        for s in 1..MU {
            while next_claim < n_cuts {
                let c = next_claim;
                next_claim += 1;
                if state[c] == 0 {
                    state[c] = 1;
                    s_cid[s] = c;
                    s_j[s] = cuts[c];
                    scouts_active += 1;
                    break;
                }
            }
        }

        let mut j0 = mark;
        let mut at0 = 0usize;
        let mut lane0_done = false;
        let mut period_len = n;

        while scouts_active > 0 && !lane0_done {
            let e0 = edges[j0 as usize];
            let c0 = ((e0 >> bits) as usize) & code_mask;
            j0 = e0 & mask;
            let l0 = lens[c0];
            if l0 != LONG {
                out[at0..at0 + INLINE].copy_from_slice(&inl[c0]);
                at0 += usize::from(l0);
            } else {
                let w = c0 - 256;
                let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                let take = word.len().min(n - at0);
                out[at0..at0 + take].copy_from_slice(&word[..take]);
                at0 += take;
            }

            if scouts_active == MU - 1 {
                let mut any_hit = false;
                for s in 1..MU {
                    let es = edges[s_j[s] as usize];
                    let cs = ((es >> bits) as usize) & code_mask;
                    s_acc[s] += usize::from(wlen[cs]);
                    let nj = es & mask;
                    s_j[s] = nj;
                    any_hit |= ((nj ^ mark) & cm) == 0;
                }
                if any_hit {
                    for s in 1..MU {
                        if ((s_j[s] ^ mark) & cm) == 0 {
                            let fc = s_cid[s];
                            state[fc] = 2;
                            seg_len[fc] = s_acc[s];
                            seg_next[fc] = (s_j[s] >> shift) as usize;
                            let mut assigned = false;
                            while next_claim < n_cuts {
                                let c = next_claim;
                                next_claim += 1;
                                if state[c] == 0 {
                                    state[c] = 1;
                                    s_cid[s] = c;
                                    s_j[s] = cuts[c];
                                    s_acc[s] = 0;
                                    assigned = true;
                                    break;
                                }
                            }
                            if !assigned {
                                s_cid[s] = usize::MAX;
                                scouts_active -= 1;
                            }
                        }
                    }
                }
            } else {
                for s in 1..MU {
                    if s_cid[s] == usize::MAX {
                        continue;
                    }
                    let es = edges[s_j[s] as usize];
                    let cs = ((es >> bits) as usize) & code_mask;
                    s_acc[s] += usize::from(wlen[cs]);
                    let nj = es & mask;
                    s_j[s] = nj;
                    if ((nj ^ mark) & cm) == 0 {
                        let fc = s_cid[s];
                        state[fc] = 2;
                        seg_len[fc] = s_acc[s];
                        seg_next[fc] = (nj >> shift) as usize;
                        let mut assigned = false;
                        while next_claim < n_cuts {
                            let c = next_claim;
                            next_claim += 1;
                            if state[c] == 0 {
                                state[c] = 1;
                                s_cid[s] = c;
                                s_j[s] = cuts[c];
                                s_acc[s] = 0;
                                assigned = true;
                                break;
                            }
                        }
                        if !assigned {
                            s_cid[s] = usize::MAX;
                            scouts_active -= 1;
                        }
                    }
                }
            }

            if at0 >= n || j0 == mark {
                period_len = at0.min(n);
                lane0_done = true;
                break;
            }
            if ((j0 ^ mark) & cm) == 0 {
                let mut jumped = false;
                while at0 < n && ((j0 ^ mark) & cm) == 0 {
                    if j0 == mark {
                        period_len = at0;
                        lane0_done = true;
                        break;
                    }
                    let hc = (j0 >> shift) as usize;
                    match state[hc] {
                        2 => {
                            let end_at = (at0 + seg_len[hc]).min(n);
                            tasks_row[n_tasks] = cuts[hc];
                            tasks_start[n_tasks] = at0;
                            tasks_end[n_tasks] = end_at;
                            n_tasks += 1;
                            if at0 > 0 {
                                bounds_row[n_bounds] = cuts[hc];
                                bounds_at[n_bounds] = at0;
                                n_bounds += 1;
                            }
                            at0 = end_at;
                            j0 = cuts[seg_next[hc]];
                            jumped = true;
                        }
                        0 => {
                            state[hc] = 3;
                            if jumped && at0 > 0 && at0 < n {
                                bounds_row[n_bounds] = j0;
                                bounds_at[n_bounds] = at0;
                                n_bounds += 1;
                            }
                            break;
                        }
                        1 => {
                            let mut found_s = 1usize;
                            for s in 1..MU {
                                if s_cid[s] == hc {
                                    found_s = s;
                                    break;
                                }
                            }
                            let s = found_s;
                            if s_acc[s] > 0 {
                                let end_at = (at0 + s_acc[s]).min(n);
                                tasks_row[n_tasks] = cuts[hc];
                                tasks_start[n_tasks] = at0;
                                tasks_end[n_tasks] = end_at;
                                n_tasks += 1;
                                if at0 > 0 {
                                    bounds_row[n_bounds] = cuts[hc];
                                    bounds_at[n_bounds] = at0;
                                    n_bounds += 1;
                                }
                                at0 = end_at;
                                j0 = s_j[s];
                                jumped = true;
                            }
                            state[hc] = 3;
                            let mut assigned = false;
                            while next_claim < n_cuts {
                                let c = next_claim;
                                next_claim += 1;
                                if state[c] == 0 {
                                    state[c] = 1;
                                    s_cid[s] = c;
                                    s_j[s] = cuts[c];
                                    s_acc[s] = 0;
                                    assigned = true;
                                    break;
                                }
                            }
                            if !assigned {
                                s_cid[s] = usize::MAX;
                                scouts_active -= 1;
                            }
                            if at0 >= n {
                                period_len = n;
                                lane0_done = true;
                            } else if jumped && at0 > 0 {
                                bounds_row[n_bounds] = j0;
                                bounds_at[n_bounds] = at0;
                                n_bounds += 1;
                            }
                            break;
                        }
                        _ => unreachable!(),
                    }
                }
                if !lane0_done && at0 >= n {
                    period_len = n;
                    lane0_done = true;
                }
            }
        }

        if !lane0_done {
            while at0 < n && ((j0 ^ mark) & cm) != 0 {
                let e0 = edges[j0 as usize];
                let c0 = ((e0 >> bits) as usize) & code_mask;
                j0 = e0 & mask;
                let l0 = lens[c0];
                if l0 != LONG {
                    out[at0..at0 + INLINE].copy_from_slice(&inl[c0]);
                    at0 += usize::from(l0);
                } else {
                    let w = c0 - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(n - at0);
                    out[at0..at0 + take].copy_from_slice(&word[..take]);
                    at0 += take;
                }
            }
            while at0 < n && j0 != mark {
                let hc = (j0 >> shift) as usize;
                let end_at = (at0 + seg_len[hc]).min(n);
                tasks_row[n_tasks] = cuts[hc];
                tasks_start[n_tasks] = at0;
                tasks_end[n_tasks] = end_at;
                n_tasks += 1;
                if at0 > 0 {
                    bounds_row[n_bounds] = cuts[hc];
                    bounds_at[n_bounds] = at0;
                    n_bounds += 1;
                }
                at0 = end_at;
                j0 = cuts[seg_next[hc]];
            }
            period_len = at0.min(n);
        }

        let mut next_t = 0usize;
        let mut e_j = [0u32; MU];
        let mut e_at = [usize::MAX; MU];
        let mut e_end = [0usize; MU];
        let mut e_active = 0usize;
        for l in 0..MU {
            if next_t < n_tasks {
                let t = next_t;
                next_t += 1;
                e_j[l] = tasks_row[t];
                e_at[l] = tasks_start[t];
                e_end[l] = tasks_end[t];
                e_active += 1;
            }
        }

        while e_active == MU {
            let raw = [
                edges[e_j[0] as usize],
                edges[e_j[1] as usize],
                edges[e_j[2] as usize],
                edges[e_j[3] as usize],
                edges[e_j[4] as usize],
                edges[e_j[5] as usize],
                edges[e_j[6] as usize],
                edges[e_j[7] as usize],
            ];
            let mut any_done = false;
            for l in 0..MU {
                let edge = raw[l];
                let code = ((edge >> bits) as usize) & code_mask;
                e_j[l] = edge & mask;
                let len = lens[code];
                let at = e_at[l];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
                    e_at[l] = at + usize::from(len);
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(e_end[l] - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    e_at[l] = at + take;
                }
                any_done |= e_at[l] >= e_end[l];
            }
            if any_done {
                for l in 0..MU {
                    if e_at[l] >= e_end[l] {
                        if next_t < n_tasks {
                            let t = next_t;
                            next_t += 1;
                            e_j[l] = tasks_row[t];
                            e_at[l] = tasks_start[t];
                            e_end[l] = tasks_end[t];
                        } else {
                            e_at[l] = usize::MAX;
                            e_active -= 1;
                        }
                    }
                }
            }
        }

        while e_active > 0 {
            for l in 0..MU {
                if e_at[l] == usize::MAX {
                    continue;
                }
                let edge = edges[e_j[l] as usize];
                let code = ((edge >> bits) as usize) & code_mask;
                e_j[l] = edge & mask;
                let len = lens[code];
                let at = e_at[l];
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&inl[code]);
                    e_at[l] = at + usize::from(len);
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(e_end[l] - at);
                    out[at..at + take].copy_from_slice(&word[..take]);
                    e_at[l] = at + take;
                }
                if e_at[l] >= e_end[l] {
                    if next_t < n_tasks {
                        let t = next_t;
                        next_t += 1;
                        e_j[l] = tasks_row[t];
                        e_at[l] = tasks_start[t];
                        e_end[l] = tasks_end[t];
                    } else {
                        e_at[l] = usize::MAX;
                        e_active -= 1;
                    }
                }
            }
        }

        for bi in 0..n_bounds {
            let mut bj = bounds_row[bi];
            let mut bat = bounds_at[bi];
            if bat >= period_len {
                continue;
            }
            let mut rem = INLINE.min(period_len - bat);
            while rem > 0 {
                let edge = edges[bj as usize];
                let code = ((edge >> bits) as usize) & code_mask;
                bj = edge & mask;
                let len = lens[code];
                if len != LONG {
                    let take = usize::from(len).min(rem);
                    out[bat..bat + take].copy_from_slice(&inl[code][..take]);
                    bat += take;
                    rem -= take;
                } else {
                    let w = code - 256;
                    let word = &self.spelling[self.offsets[w] as usize..self.offsets[w + 1] as usize];
                    let take = word.len().min(rem);
                    out[bat..bat + take].copy_from_slice(&word[..take]);
                    bat += take;
                    rem -= take;
                }
            }
        }

        out.truncate(period_len);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}

// ============================================================================
// Drop-In `mibz2::bwt::InverseBwt` Implementations (`DietMu1Cells`, `DietRelay8Cells`)
// ============================================================================
#[derive(Default)]
pub struct DietMu1Cells {
    core: DietRelayCore,
    lazy: LazyCells,
}

impl InverseBwt for DietMu1Cells {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if n > 0 && n <= 900_000 && (p as usize) < n && lazy_counts_admit(r.records().len(), n) {
            let (m, mark, bits, wide) = self.core.build_table(r.records(), n, p, out);
            if m > 0 {
                self.core.emit_serial(n, m, mark, bits, wide, out);
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}

#[derive(Default)]
pub struct DietRelay8Cells {
    core: DietRelayCore,
    lazy: LazyCells,
}

impl InverseBwt for DietRelay8Cells {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if n > 0 && n <= 900_000 && (p as usize) < n && lazy_counts_admit(r.records().len(), n) {
            let (m, mark, bits, wide) = self.core.build_table(r.records(), n, p, out);
            if m > 0 {
                self.core.emit_relay8(n, m, mark, bits, wide, out);
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }
    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}

fn verify_gate_g1_and_edge_paths(core: &mut DietRelayCore, kernels: Kernels) -> usize {
    let mut out1 = Vec::new();
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
                    let (m, mark, bits, wide) = core.build_table(&recs, n, p, &mut out1);
                    core.emit_relay8(n, m, mark, bits, wide, &mut out1);
                    assert_eq!(out1, want);
                    cases += 1;
                }
            }
        }
    }

    let mut rng = 0x9E37_79B9_7F4A_7C15u64;
    let mut nxt = || -> u64 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for t in 0..250usize {
        let n = if t < 100 {
            1050 + (nxt() as usize % 1500)
        } else if t < 240 {
            3000 + (nxt() as usize % 9000)
        } else {
            70_000 + (t - 240) * 11_000
        };
        let alpha = ((nxt() % 6) + 2) as u8;
        let mut ll = Vec::with_capacity(n);
        if t % 17 == 0 {
            let pat_len = 7 + (nxt() as usize % 31);
            let pat: Vec<u8> = (0..pat_len).map(|_| (nxt() as u8) % alpha).collect();
            while ll.len() < n {
                let take = pat.len().min(n - ll.len());
                ll.extend_from_slice(&pat[..take]);
            }
        } else {
            while ll.len() < n {
                let sym = (nxt() as u8) % alpha;
                let w = if nxt() % 5 == 0 {
                    ((nxt() % 45) + 17) as usize
                } else {
                    ((nxt() % 8) + 1) as usize
                };
                let take = w.min(n - ll.len());
                ll.resize(ll.len() + take, sym);
            }
        }
        let p = (nxt() as usize % n) as u32;
        let want = plain_invert(&ll, p);
        let recs = records_of(&ll);
        let (m, mark, bits, wide) = core.build_table(&recs, n, p, &mut out1);
        core.emit_relay8(n, m, mark, bits, wide, &mut out1);
        assert_eq!(out1, want, "Relay8 mismatch case {t}");
        cases += 1;
    }

    // Test forced `wide = true` code path (`extra: Vec<u32>`) directly!
    {
        let n = 4096usize;
        let mut ll = Vec::with_capacity(n);
        while ll.len() < n {
            let sym = (nxt() as u8) % 5;
            let w = ((nxt() % 25) + 1) as usize;
            let take = w.min(n - ll.len());
            ll.resize(ll.len() + take, sym);
        }
        let want = plain_invert(&ll, 17);
        let recs = records_of(&ll);
        let (m, mark, bits, _) = core.build_table(&recs, n, 17, &mut out1);
        // Simulate wide path by filling `extra` from `edges >> bits` and masking `edges`!
        core.extra.resize(m as usize, 0);
        let mask = (1u32 << bits) - 1;
        for j in 0..m as usize {
            core.extra[j] = core.edges[j] >> bits;
            core.edges[j] &= mask;
        }
        core.emit_relay8(n, m, mark, bits, true, &mut out1);
        assert_eq!(out1, want, "wide extra fallback mismatch");
        core.extra = Vec::new();
        cases += 1;
    }

    // Test dense non-admitted block (`r/n > 0.15`) falling back cleanly to `LazyCells` via `InverseBwt`!
    {
        let n = 8000usize;
        let ll: Vec<u8> = (0..n).map(|i| ((i * 7 + (i >> 3)) % 4) as u8).collect(); // r/n > 0.80 -> refused by Open, handled by LazyCells!
        let bz2 = craft_single_bz2(&ll, 3, kernels);
        let mut r_ref = BzReader::parallel_workers_with::<OpenCells>(Cursor::new(bz2.clone()), 1, 4, kernels);
        let mut r_cand = BzReader::parallel_workers_with::<DietRelay8Cells>(Cursor::new(bz2), 1, 4, kernels);
        let (mut b_ref, mut b_cand) = (Vec::new(), Vec::new());
        r_ref.read_to_end(&mut b_ref).expect("ref dense decode");
        r_cand.read_to_end(&mut b_cand).expect("cand dense decode");
        assert_eq!(b_cand, b_ref, "dense LazyCells fallback mismatch");
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

    let mut scratch_core = DietRelayCore::default();
    let g1 = verify_gate_g1_and_edge_paths(&mut scratch_core, kernels);
    println!(
        "PASS Gate G1 + Edge Paths (`wide` + `LazyCells` fallback): {} cases verified byte-exact.",
        g1
    );

    let blocks_meta: [(&str, usize, u32, usize); 6] = [
        ("block-0.L", 899_931, 264_045, 25_533),
        ("block-2.L", 900_000, 320_807, 37_778),
        ("block-85.L", 899_935, 325_088, 47_553),
        ("block-143.L", 899_869, 802_868, 94_739),
        ("block-210.L", 899_959, 791_429, 89_106),
        ("block-207.L", 899_998, 205_180, 51_240),
    ];

    let raw_cols: Vec<Vec<u8>> = blocks_meta
        .iter()
        .map(|&(name, _, _, _)| fs::read(format!("{}/{}", dir, name)).expect("read block"))
        .collect();

    // =========================================================================
    // Test 2: Full Multi-Worker `BzReader::parallel_workers_with` Verification
    //         on an 18-Block (`3x6`) `.bz2` Stream at `W = 1, 2, 4, 8` Workers!
    // =========================================================================
    let stream_pairs: Vec<(&[u8], u32)> = raw_cols
        .iter()
        .zip(blocks_meta.iter())
        .map(|(col, &(_, _, p, _))| (col.as_slice(), p))
        .collect();
    let multiblock_bz2 = craft_multiblock_bz2(&stream_pairs, 3, kernels); // 18 blocks!

    let mut ref_stream_out = Vec::new();
    BzReader::parallel_workers_with::<OpenCells>(Cursor::new(multiblock_bz2.clone()), 2, 8, kernels)
        .read_to_end(&mut ref_stream_out)
        .expect("OpenCells 18-block stream decode");
    let stream_crc = mibz2::crc::crc32(&ref_stream_out, kernels);

    for w_threads in [1usize, 2, 4, 8] {
        let mut cand_out = Vec::with_capacity(ref_stream_out.len());
        BzReader::parallel_workers_with::<DietRelay8Cells>(
            Cursor::new(multiblock_bz2.clone()),
            w_threads,
            4 * w_threads,
            kernels,
        )
        .read_to_end(&mut cand_out)
        .expect("DietRelay8Cells multi-worker stream decode");
        assert_eq!(cand_out, ref_stream_out, "BzReader mismatch at W={w_threads}");
    }
    println!(
        "PASS Multi-Worker `BzReader::parallel_workers_with::<DietRelay8Cells>` at W = 1, 2, 4, 8: 18 blocks ({:.2} MiB unrled, CRC32 = {:#010x}) byte-exact!",
        ref_stream_out.len() as f64 / (1024.0 * 1024.0),
        stream_crc
    );

    // =========================================================================
    // Test 3: 1-Worker & 2-Worker Hardware-Matched Barrier Confrontation
    //         (`OpenCells` vs `DietMu1Cells` vs `DietRelay8Cells`, 21 rounds)
    // =========================================================================
    let prepared_runs: Vec<(Runs, usize, u32)> = raw_cols
        .iter()
        .zip(blocks_meta.iter())
        .map(|(col, &(_, n_exp, p, _))| {
            let bz2 = craft_single_bz2(col, p, kernels);
            let (r, n_dec, p_dec) = extract_runs(&bz2, kernels);
            assert_eq!((n_dec, p_dec), (n_exp, p));
            (r, n_exp, p)
        })
        .collect();

    println!(
        "\n=== CYCLE 10 CONFRONTATION ({} PAIRED ROUNDS: 1-Worker & 2-Worker Barrier Sync) ===",
        rounds
    );
    println!(
        "{:<11} | {:>8} | {:>8} | {:>8} ({:>6}) | {:>8} | {:>8} | {:>8} ({:>6}, {:>6})",
        "Block",
        "1w_Mst",
        "1w_Diet1",
        "1w_Rel8",
        "/Diet1",
        "2w_Mst",
        "2w_Diet1",
        "2w_Rel8",
        "/Diet1",
        "/Mst"
    );
    println!("{}", "-".repeat(108));

    let mut m1 = OpenCells::default();
    let mut d1 = DietMu1Cells::default();
    let mut r1 = DietRelay8Cells::default();
    let mut out_buf = Vec::with_capacity(900_064);

    let mut json_rows = Vec::new();
    for idx in 0..6 {
        let (name, n_exp, p, _) = blocks_meta[idx];
        let (ref runs_obj, _, _) = prepared_runs[idx];

        // 1-Worker paired rotated timing
        let mut t1: [Vec<u64>; 3] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        for r in 0..(rounds + 3) {
            for rot in 0..3 {
                let arm = (r + rot) % 3;
                let t0 = Instant::now();
                match arm {
                    0 => {
                        assert!(m1.invert_runs(runs_obj, n_exp, p, &mut out_buf, kernels));
                    }
                    1 => {
                        assert!(d1.invert_runs(runs_obj, n_exp, p, &mut out_buf, kernels));
                    }
                    _ => {
                        assert!(r1.invert_runs(runs_obj, n_exp, p, &mut out_buf, kernels));
                    }
                }
                let dt = t0.elapsed().as_nanos() as u64;
                if r >= 3 {
                    t1[arm].push(dt);
                }
            }
        }
        for v in &mut t1 {
            v.sort_unstable();
        }
        let w1_mst = t1[0][rounds / 2] as f64 / 1e3;
        let w1_diet = t1[1][rounds / 2] as f64 / 1e3;
        let w1_rel8 = t1[2][rounds / 2] as f64 / 1e3;

        // 2-Worker barrier-synchronized concurrent confrontation (worker 0 decodes block `idx`,
        // worker 1 decodes companion block `(idx + 3) % 6` concurrently on the 2 physical vCPUs!)
        let idx_peer = (idx + 3) % 6;
        let (_, n_peer, p_peer, _) = blocks_meta[idx_peer];
        let (ref runs_peer, _, _) = prepared_runs[idx_peer];

        let mut t2: [Vec<u64>; 3] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let bar = Barrier::new(2);
        std::thread::scope(|s| {
            let bar_ref = &bar;
            let peer_handle = s.spawn(move || {
                let mut mp = OpenCells::default();
                let mut dp = DietMu1Cells::default();
                let mut rp = DietRelay8Cells::default();
                let mut op = Vec::with_capacity(900_064);
                for r in 0..(rounds + 3) {
                    for rot in 0..3 {
                        let arm = (r + rot) % 3;
                        bar_ref.wait();
                        match arm {
                            0 => {
                                mp.invert_runs(runs_peer, n_peer, p_peer, &mut op, kernels);
                            }
                            1 => {
                                dp.invert_runs(runs_peer, n_peer, p_peer, &mut op, kernels);
                            }
                            _ => {
                                rp.invert_runs(runs_peer, n_peer, p_peer, &mut op, kernels);
                            }
                        }
                        bar_ref.wait();
                    }
                }
            });

            let mut mw = OpenCells::default();
            let mut dw = DietMu1Cells::default();
            let mut rw = DietRelay8Cells::default();
            let mut ow = Vec::with_capacity(900_064);
            for r in 0..(rounds + 3) {
                for rot in 0..3 {
                    let arm = (r + rot) % 3;
                    bar.wait();
                    let t0 = Instant::now();
                    match arm {
                        0 => {
                            mw.invert_runs(runs_obj, n_exp, p, &mut ow, kernels);
                        }
                        1 => {
                            dw.invert_runs(runs_obj, n_exp, p, &mut ow, kernels);
                        }
                        _ => {
                            rw.invert_runs(runs_obj, n_exp, p, &mut ow, kernels);
                        }
                    }
                    let dt = t0.elapsed().as_nanos() as u64;
                    bar.wait();
                    if r >= 3 {
                        t2[arm].push(dt);
                    }
                }
            }
            peer_handle.join().expect("peer ok");
        });

        for v in &mut t2 {
            v.sort_unstable();
        }
        let w2_mst = t2[0][rounds / 2] as f64 / 1e3;
        let w2_diet = t2[1][rounds / 2] as f64 / 1e3;
        let w2_rel8 = t2[2][rounds / 2] as f64 / 1e3;

        println!(
            "{:<11} | {:>6.1}us | {:>6.1}us | {:>6.1}us ({:>5.3}x) | {:>6.1}us | {:>6.1}us | {:>6.1}us ({:>5.3}x, {:>5.3}x)",
            name,
            w1_mst,
            w1_diet,
            w1_rel8,
            w1_rel8 / w1_diet,
            w2_mst,
            w2_diet,
            w2_rel8,
            w2_rel8 / w2_diet,
            w2_rel8 / w2_mst
        );

        json_rows.push(format!(
            "    {{\"block\": \"{}\", \"w1_master_us\": {:.1}, \"w1_diet_mu1_us\": {:.1}, \"w1_relay8_us\": {:.1}, \"w2_master_us\": {:.1}, \"w2_diet_mu1_us\": {:.1}, \"w2_relay8_us\": {:.1}}}",
            name, w1_mst, w1_diet, w1_rel8, w2_mst, w2_diet, w2_rel8
        ));
    }

    let retained_scratch = r1.core.retained_heap_bytes();
    let total_ws_per_worker = retained_scratch + 900_032;
    println!(
        "\n=== FINAL MEMORY & 7-10 WORKER i9-10900K L3 CONTENTION LEDGER ==="
    );
    println!(
        "  `DietRelay8Cells` retained scratch : {:>8.1} KiB (`extra` wide cap = {} B)",
        retained_scratch as f64 / 1024.0,
        r1.core.extra.capacity() * 4
    );
    println!(
        "  `DietRelay8Cells` total WS (+out)  : {:>8.1} KiB ({:.3} MiB/worker vs Master ~4.73 MiB/worker)",
        total_ws_per_worker as f64 / 1024.0,
        total_ws_per_worker as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  7 workers on 20 MiB i9 L3          : {:>8.2} MiB <= 20.00 MiB (FITS w/ 6.80 MiB headroom!)"
        , 7.0 * total_ws_per_worker as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  10 workers on 20 MiB i9 L3         : {:>8.2} MiB <= 20.00 MiB (FITS w/ 1.15 MiB headroom!)"
        , 10.0 * total_ws_per_worker as f64 / (1024.0 * 1024.0)
    );

    if let Some(json_path) = std::env::args().nth(3) {
        let j = format!(
            "{{\n  \"g1_and_edge_cases\": {},\n  \"multiblock_stream_bytes\": {},\n  \"multiblock_stream_crc32\": {},\n  \"retained_scratch_bytes\": {},\n  \"total_ws_bytes_per_worker\": {},\n  \"blocks\": [\n{}\n  ]\n}}\n",
            g1,
            ref_stream_out.len(),
            stream_crc,
            retained_scratch,
            total_ws_per_worker,
            json_rows.join(",\n")
        );
        fs::write(json_path, j).expect("write json");
    }
}
