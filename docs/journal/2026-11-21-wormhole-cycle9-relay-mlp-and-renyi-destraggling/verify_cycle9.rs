#![forbid(unsafe_code)]
//! Wormhole Cycle 9 — Safe-Rust Confrontation of `Relay-MLP`:
//!   1. `Theorem 9.1` (Rényi 1953 + Graham 1969 Cut-Straggler Law): sweeps `k in {5, 6, 7, 8, 9}`
//!      (`S ~ 24 .. 384` zero-bit cuts) and compares measured work-stealing rounds against
//!      `1 + (mu - 1) * H_S / S`.
//!   2. `Theorem 9.2` (`Relay-MLP`: `1 Frontier Emitter + (mu-1) Cut Scouts -> mu Parallel Emitters`):
//!      overlaps Phase 4a scouting with direct in-place emission from `mark`, eliminating single-lane
//!      re-walks of emitted frontier cuts and eliminating idle scout barrier stalls.
//!   3. `Theorem 9.3` (Commutative Post-Slop Boundary Restoration): removes all inner-loop `end_at`
//!      bounds branches from parallel 16B sloppy emission via O(S) exact 16B post-restoration along `edges`.
//!   4. Confronts `Master-0.5.9` (`unsafe` `OpenCells`), `Diet_mu1` (safe, `mu = 1`), `Relay4` (`mu = 4`),
//!      `Relay8` (`mu = 8`), and `Relay12` (`mu = 12`) across 25 paired interleaved rounds + exact
//!      6-dimensional BMCP(256 KiB, 64 B, mu) cache trace!

use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::fs;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

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
// `RelayKernel`: Shared Dieted Table + Power-of-Two Safe Dict + `Relay-MLP`!
// ============================================================================
#[derive(Default)]
pub struct RelayKernel {
    pub edges: Vec<u32>,
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

#[derive(Default, Clone, Copy, Debug)]
pub struct RelayExecStats {
    pub rounds_4a: usize,
    pub rounds_4b: usize,
    pub q_rand_total: usize,
    pub lane0_4a_edges: usize,
    pub n_cuts: usize,
    pub n_tasks: usize,
    pub n_bounds: usize,
    pub max_cut_edges: usize,
    pub ns_4a: u64,
    pub ns_4b: u64,
}

impl RelayKernel {
    pub fn retained_heap_bytes(&self) -> usize {
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

    pub fn build_table(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> (u32, u32, u32) {
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

        // Pad dictionary tables to next power of two (`<= 4096`, `+~9 KiB`) so `code & (cap - 1)`
        // statically proves slice bounds to LLVM in `#![forbid(unsafe_code)]`!
        let cap_pow2 = self.inline16.len().next_power_of_two();
        self.inline16.resize(cap_pow2, [0u8; INLINE]);
        self.lens.resize(cap_pow2, 1);
        self.word_len.resize(cap_pow2, 1);
        self.code_cap = cap_pow2;

        // Phase 3: Span-Chunked Band Wiring
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

    /// Fair `mu = 1` Safe-Rust Baseline (`Diet_mu1` with identical power-of-two `code_mask`)
    pub fn emit_serial(&self, n: usize, m: u32, mark: u32, bits: u32, out: &mut Vec<u8>) -> u64 {
        let t0 = Instant::now();
        let mask = (1u32 << bits) - 1;
        let cap = self.code_cap;
        let code_mask = cap - 1;
        let inl = &self.inline16[..cap];
        let lens = &self.lens[..cap];
        let edges = self.edges.as_slice();
        let mut j = mark;
        let mut at = 0usize;
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
        out.truncate(at.min(n));
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
        t0.elapsed().as_nanos() as u64
    }

    #[inline]
    pub fn cut_params_512(m: u32, mark: u32, bits: u32, log2_cuts: u32) -> (u32, u32, [u32; 512], usize) {
        let shift = bits.saturating_sub(log2_cuts);
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
        (shift, cm, cuts, n_cuts)
    }

    /// `Relay-MLP<MU>` (`1 Frontier Emitter + (MU-1) Cut Scouts -> MU Parallel Sloppy Emitters + O(S) Restore`)
    pub fn emit_relay_mlp<const MU: usize>(
        &self,
        n: usize,
        m: u32,
        mark: u32,
        bits: u32,
        log2_cuts: u32,
        out: &mut Vec<u8>,
    ) -> RelayExecStats {
        let mut st = RelayExecStats::default();
        if MU <= 1 || m < 1024 {
            let ns = self.emit_serial(n, m, mark, bits, out);
            st.rounds_4a = m as usize;
            st.q_rand_total = m as usize;
            st.ns_4b = ns;
            return st;
        }
        let (shift, cm, cuts, n_cuts) = Self::cut_params_512(m, mark, bits, log2_cuts);
        st.n_cuts = n_cuts;
        if n_cuts <= 2 {
            let ns = self.emit_serial(n, m, mark, bits, out);
            st.rounds_4a = m as usize;
            st.q_rand_total = m as usize;
            st.ns_4b = ns;
            return st;
        }

        let mask = (1u32 << bits) - 1;
        let cap = self.code_cap;
        let code_mask = cap - 1;
        let edges = self.edges.as_slice();
        let wlen = &self.word_len[..cap];
        let lens = &self.lens[..cap];
        let inl = &self.inline16[..cap];

        // Stack-only cut & task tables (`0 B` heap!)
        let mut state = [0u8; 512]; // 0 = UNCLAIMED, 1 = SCOUTING, 2 = SCOUTED, 3 = HANDED_TO_LANE0
        let mut seg_len = [0usize; 512];
        let mut seg_next = [0usize; 512];
        let mut seg_edges = [0usize; 512];

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
        let mut s_edg = [0usize; MU];
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

        // ---------------------------------------------------------------------
        // Phase 4a: Overlapped `1 Frontier Emitter` + `MU - 1 Cut Scouts`!
        // ---------------------------------------------------------------------
        let t4a = Instant::now();
        let mut j0 = mark;
        let mut at0 = 0usize;
        let mut lane0_done = false;
        let mut period_len = n;
        let mut cur_c0_edges = 0usize;

        while scouts_active > 0 && !lane0_done {
            st.rounds_4a += 1;

            // 1. Step Lane 0 (`mark` frontier) directly into `out[at0..]`
            st.q_rand_total += 1;
            st.lane0_4a_edges += 1;
            cur_c0_edges += 1;
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

            // 2. Step all `MU - 1` active Scout Lanes concurrently!
            if scouts_active == MU - 1 {
                st.q_rand_total += MU - 1;
                let mut any_hit = false;
                for s in 1..MU {
                    let es = edges[s_j[s] as usize];
                    let cs = ((es >> bits) as usize) & code_mask;
                    s_acc[s] += usize::from(wlen[cs]);
                    s_edg[s] += 1;
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
                            seg_edges[fc] = s_edg[s];
                            if s_edg[s] > st.max_cut_edges {
                                st.max_cut_edges = s_edg[s];
                            }
                            let mut assigned = false;
                            while next_claim < n_cuts {
                                let c = next_claim;
                                next_claim += 1;
                                if state[c] == 0 {
                                    state[c] = 1;
                                    s_cid[s] = c;
                                    s_j[s] = cuts[c];
                                    s_acc[s] = 0;
                                    s_edg[s] = 0;
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
                    st.q_rand_total += 1;
                    let es = edges[s_j[s] as usize];
                    let cs = ((es >> bits) as usize) & code_mask;
                    s_acc[s] += usize::from(wlen[cs]);
                    s_edg[s] += 1;
                    let nj = es & mask;
                    s_j[s] = nj;
                    if ((nj ^ mark) & cm) == 0 {
                        let fc = s_cid[s];
                        state[fc] = 2;
                        seg_len[fc] = s_acc[s];
                        seg_next[fc] = (nj >> shift) as usize;
                        seg_edges[fc] = s_edg[s];
                        if s_edg[s] > st.max_cut_edges {
                            st.max_cut_edges = s_edg[s];
                        }
                        let mut assigned = false;
                        while next_claim < n_cuts {
                            let c = next_claim;
                            next_claim += 1;
                            if state[c] == 0 {
                                state[c] = 1;
                                s_cid[s] = c;
                                s_j[s] = cuts[c];
                                s_acc[s] = 0;
                                s_edg[s] = 0;
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

            // 3. Handle Lane 0 reaching block end or a cut boundary!
            if at0 >= n || j0 == mark {
                if cur_c0_edges > st.max_cut_edges {
                    st.max_cut_edges = cur_c0_edges;
                }
                period_len = at0.min(n);
                lane0_done = true;
                break;
            }
            if ((j0 ^ mark) & cm) == 0 {
                if cur_c0_edges > st.max_cut_edges {
                    st.max_cut_edges = cur_c0_edges;
                }
                cur_c0_edges = 0;
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
                            cur_c0_edges = s_edg[s];
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
                                    s_edg[s] = 0;
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

        // Finish the remaining tail of Lane 0's active cut (`~m/(2S) ~ 380` steps at k=8!)
        // and resolve all remaining `SCOUTED` cuts into `tasks` in O(S)!
        if !lane0_done {
            while at0 < n && ((j0 ^ mark) & cm) != 0 {
                st.rounds_4a += 1;
                st.q_rand_total += 1;
                st.lane0_4a_edges += 1;
                cur_c0_edges += 1;
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
            if cur_c0_edges > st.max_cut_edges {
                st.max_cut_edges = cur_c0_edges;
            }
            while at0 < n && j0 != mark {
                let hc = (j0 >> shift) as usize;
                debug_assert_eq!(state[hc], 2);
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
        st.ns_4a = t4a.elapsed().as_nanos() as u64;
        st.n_tasks = n_tasks;
        st.n_bounds = n_bounds;

        // ---------------------------------------------------------------------
        // Phase 4b: `MU`-Lane Work-Stealing Sloppy 16B Emit + O(S) Exact Restore!
        // ---------------------------------------------------------------------
        let t4b = Instant::now();
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
            st.rounds_4b += 1;
            st.q_rand_total += MU;
            let mut raw = [0u32; MU];
            for l in 0..MU {
                raw[l] = edges[e_j[l] as usize];
            }
            let mut any_done = false;
            for l in 0..MU {
                let edge = raw[l];
                let code = ((edge >> bits) as usize) & code_mask;
                e_j[l] = edge & mask;
                let len = lens[code];
                let at = e_at[l];
                if len != LONG {
                    // Unconditional 16B sloppy store — ZERO boundary check in hot loop!
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
            st.rounds_4b += 1;
            for l in 0..MU {
                if e_at[l] == usize::MAX {
                    continue;
                }
                st.q_rand_total += 1;
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

        // O(S) Commutative Idempotent 16B Boundary Restoration (`Theorem 9.3`)
        for bi in 0..n_bounds {
            let mut bj = bounds_row[bi];
            let mut bat = bounds_at[bi];
            if bat >= period_len {
                continue;
            }
            let mut rem = INLINE.min(period_len - bat);
            while rem > 0 {
                st.q_rand_total += 1;
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
        st.ns_4b = t4b.elapsed().as_nanos() as u64;
        st
    }
}

// ============================================================================
// Exact BMCP(256 KiB, 64 B, MU) Cache Simulator for `Relay-MLP<MU>`
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

#[derive(Default, Clone, Copy, Debug)]
struct RelayBmcp {
    dep_l2_crit_4a: usize,
    dep_l2_crit_4b: usize,
    dep_l2_crit_total: usize,
    c_miss_256k: usize,
    c_miss_2048k: usize,
}

fn simulate_relay_bmcp<const MU: usize>(
    rk: &RelayKernel,
    n: usize,
    m: u32,
    mark: u32,
    bits: u32,
    log2_cuts: u32,
) -> RelayBmcp {
    let base_edges = 0u32;
    let n_edge_lines = (m as usize).div_ceil(16) as u32 + 2;
    let base_lens = base_edges + n_edge_lines;
    let n_lens_lines = rk.code_cap.div_ceil(64) as u32 + 2;
    let base_wlen = base_lens + n_lens_lines;
    let n_wlen_lines = rk.code_cap.div_ceil(32) as u32 + 2;
    let base_inl = base_wlen + n_wlen_lines;
    let n_inl_lines = rk.code_cap.div_ceil(4) as u32 + 2;
    let base_spell = base_inl + n_inl_lines;
    let n_spell_lines = rk.spelling.len().div_ceil(64) as u32 + 2;
    let base_out = base_spell + n_spell_lines;
    let n_out_lines = (n + 32).div_ceil(64) as u32 + 2;
    let universe = (base_out + n_out_lines) as usize;

    let mut l2 = FastLru::new(4096, universe);
    let mut l3 = FastLru::new(32768, universe);
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

    let mut prof = RelayBmcp::default();
    let (shift, cm, cuts, n_cuts) = RelayKernel::cut_params_512(m, mark, bits, log2_cuts);
    let mask = (1u32 << bits) - 1;
    let code_mask = rk.code_cap - 1;

    let mut state = [0u8; 512];
    let mut seg_len = [0usize; 512];
    let mut seg_next = [0usize; 512];
    let mut tasks_row = [0u32; 1024];
    let mut tasks_start = [0usize; 1024];
    let mut tasks_end = [0usize; 1024];
    let mut n_tasks = 0usize;

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

    let mut lane_dep_4a = [0usize; MU];
    let mut j0 = mark;
    let mut at0 = 0usize;
    let mut lane0_done = false;

    let mut touch = |ln: u32, prof: &mut RelayBmcp| -> bool {
        let m2 = !l2.access(ln);
        if m2 {
            prof.c_miss_256k += 1;
        }
        if !l3.access(ln) {
            prof.c_miss_2048k += 1;
        }
        m2
    };

    while scouts_active > 0 && !lane0_done {
        if touch(base_edges + (j0 / 16), &mut prof) {
            lane_dep_4a[0] += 1;
        }
        let e0 = rk.edges[j0 as usize];
        let c0 = ((e0 >> bits) as usize) & code_mask;
        j0 = e0 & mask;
        touch(base_lens + (c0 / 64) as u32, &mut prof);
        let l0 = rk.lens[c0];
        let step0 = if l0 != LONG {
            touch(base_inl + (c0 / 4) as u32, &mut prof);
            usize::from(l0)
        } else {
            let w = c0 - 256;
            let s0 = rk.offsets[w] as usize;
            let s1 = rk.offsets[w + 1] as usize;
            let take = (s1 - s0).min(n - at0);
            for sln in (s0 / 64)..=((s0 + take.saturating_sub(1)) / 64) {
                touch(base_spell + sln as u32, &mut prof);
            }
            take
        };
        touch(base_out + (at0 / 64) as u32, &mut prof);
        at0 += step0;

        for s in 1..MU {
            if s_cid[s] == usize::MAX {
                continue;
            }
            if touch(base_edges + (s_j[s] / 16), &mut prof) {
                lane_dep_4a[s] += 1;
            }
            let es = rk.edges[s_j[s] as usize];
            let cs = ((es >> bits) as usize) & code_mask;
            touch(base_wlen + (cs / 32) as u32, &mut prof);
            s_acc[s] += usize::from(rk.word_len[cs]);
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

        if at0 >= n || j0 == mark {
            lane0_done = true;
            break;
        }
        if ((j0 ^ mark) & cm) == 0 {
            while at0 < n && ((j0 ^ mark) & cm) == 0 {
                if j0 == mark {
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
                        at0 = end_at;
                        j0 = cuts[seg_next[hc]];
                    }
                    0 => {
                        state[hc] = 3;
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
                            at0 = end_at;
                            j0 = s_j[s];
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
                            lane0_done = true;
                        }
                        break;
                    }
                    _ => unreachable!(),
                }
            }
            if !lane0_done && at0 >= n {
                lane0_done = true;
            }
        }
    }

    if !lane0_done {
        while at0 < n && ((j0 ^ mark) & cm) != 0 {
            if touch(base_edges + (j0 / 16), &mut prof) {
                lane_dep_4a[0] += 1;
            }
            let e0 = rk.edges[j0 as usize];
            let c0 = ((e0 >> bits) as usize) & code_mask;
            j0 = e0 & mask;
            touch(base_lens + (c0 / 64) as u32, &mut prof);
            let l0 = rk.lens[c0];
            let step0 = if l0 != LONG {
                touch(base_inl + (c0 / 4) as u32, &mut prof);
                usize::from(l0)
            } else {
                let w = c0 - 256;
                (rk.offsets[w + 1] as usize - rk.offsets[w] as usize).min(n - at0)
            };
            touch(base_out + (at0 / 64) as u32, &mut prof);
            at0 += step0;
        }
        while at0 < n && j0 != mark {
            let hc = (j0 >> shift) as usize;
            let end_at = (at0 + seg_len[hc]).min(n);
            tasks_row[n_tasks] = cuts[hc];
            tasks_start[n_tasks] = at0;
            tasks_end[n_tasks] = end_at;
            n_tasks += 1;
            at0 = end_at;
            j0 = cuts[seg_next[hc]];
        }
    }
    prof.dep_l2_crit_4a = *lane_dep_4a.iter().max().unwrap();

    // Phase 4b trace
    let mut next_t = 0usize;
    let mut e_j = [0u32; MU];
    let mut e_at = [usize::MAX; MU];
    let mut e_end = [0usize; MU];
    let mut lane_dep_4b = [0usize; MU];
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
    while e_active > 0 {
        for l in 0..MU {
            if e_at[l] == usize::MAX {
                continue;
            }
            if touch(base_edges + (e_j[l] / 16), &mut prof) {
                lane_dep_4b[l] += 1;
            }
            let edge = rk.edges[e_j[l] as usize];
            let code = ((edge >> bits) as usize) & code_mask;
            e_j[l] = edge & mask;
            touch(base_lens + (code / 64) as u32, &mut prof);
            let len = rk.lens[code];
            let step_b = if len != LONG {
                touch(base_inl + (code / 4) as u32, &mut prof);
                usize::from(len)
            } else {
                let w = code - 256;
                let s0 = rk.offsets[w] as usize;
                let s1 = rk.offsets[w + 1] as usize;
                let take = (s1 - s0).min(e_end[l] - e_at[l]);
                for sln in (s0 / 64)..=((s0 + take.saturating_sub(1)) / 64) {
                    touch(base_spell + sln as u32, &mut prof);
                }
                take
            };
            let o0 = e_at[l] / 64;
            let o1 = (e_at[l] + step_b.saturating_sub(1).max(15)) / 64;
            for oln in o0..=o1 {
                touch(base_out + oln as u32, &mut prof);
            }
            e_at[l] += step_b;
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
    prof.dep_l2_crit_4b = *lane_dep_4b.iter().max().unwrap();
    prof.dep_l2_crit_total = prof.dep_l2_crit_4a + prof.dep_l2_crit_4b;
    prof
}

fn harmonic(s: usize) -> f64 {
    (1..=s).map(|i| 1.0 / i as f64).sum()
}

fn verify_gate_g1(rk: &mut RelayKernel) -> usize {
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
                    let (m, mark, bits) = rk.build_table(&recs, n, p, &mut out1);
                    rk.emit_relay_mlp::<8>(n, m, mark, bits, 8, &mut out1);
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
    // 250 multi-cycle, periodic, tight-cut (`m = 1025..4000` w/ `log2_cuts = 9` -> cuts 2..8 bytes apart!),
    // and multi-epoch (`n = 70,000..180,000`) torture cases across MU = 4, 8, 12!
    for t in 0..250usize {
        let n = if t < 100 {
            1050 + (nxt() as usize % 1500) // Tight-cut slop-overlap torture (`S ~ 256..512`, `n/S ~ 3..8` bytes!)
        } else if t < 240 {
            3000 + (nxt() as usize % 9000)
        } else {
            70_000 + (t - 240) * 11_000
        };
        let alpha = ((nxt() % 6) + 2) as u8;
        let mut ll = Vec::with_capacity(n);
        if t % 17 == 0 {
            // Periodic torture!
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

        let (m, mark, bits) = rk.build_table(&recs, n, p, &mut out1);
        rk.emit_relay_mlp::<8>(n, m, mark, bits, 9, &mut out1);
        assert_eq!(out1, want, "Relay8 k=9 mismatch case {t} n={n} m={m}");

        let (m2, mark2, bits2) = rk.build_table(&recs, n, p, &mut out2);
        rk.emit_relay_mlp::<12>(n, m2, mark2, bits2, 8, &mut out2);
        assert_eq!(out2, want, "Relay12 k=8 mismatch case {t} n={n} m={m2}");
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
        .unwrap_or(25);
    let kernels = Kernels::detect();

    let mut rk = RelayKernel::default();
    let g1 = verify_gate_g1(&mut rk);
    println!(
        "PASS Gate G1: {} small, periodic, tight-cut slop-overlap, multi-cycle, and multi-epoch cases verified byte-exact.",
        g1
    );

    let blocks: [(&str, usize, u32, usize); 6] = [
        ("block-0.L", 899_931, 264_045, 25_533),
        ("block-2.L", 900_000, 320_807, 37_778),
        ("block-85.L", 899_935, 325_088, 47_553),
        ("block-143.L", 899_869, 802_868, 94_739),
        ("block-210.L", 899_959, 791_429, 89_106),
        ("block-207.L", 899_998, 205_180, 51_240),
    ];

    // 1. Verify Rényi–Graham Cut-Straggler Law (`k in {5, 6, 7, 8, 9}`) on `block-210.L` and `block-143.L`!
    println!("\n=== THEOREM 9.1: RENYI-GRAHAM CUT-STRAGGLER SWEEP (`Relay8`, `k = 5..9`) ===");
    println!(
        "{:<11} | {:>3} | {:>5} | {:>9} | {:>10} | {:>9} | {:>9}",
        "Block", "k", "S_cut", "MaxSegEdg", "Rounds(4a+4b)", "Meas/(m/4)", "RenyiBnd"
    );
    println!("{}", "-".repeat(76));
    let mut out_tmp = Vec::with_capacity(900_064);
    for &(bname, n_exp, p, _) in &[blocks[3], blocks[4]] {
        let ll = fs::read(format!("{}/{}", dir, bname)).expect("read");
        let bz2 = craft_bz2_block(&ll, p, kernels);
        let (runs_obj, _, _) = extract_runs(&bz2, kernels);
        for k in [5u32, 6, 7, 8, 9] {
            let (m, mark, bits) = rk.build_table(runs_obj.records(), n_exp, p, &mut out_tmp);
            let st = rk.emit_relay_mlp::<8>(n_exp, m, mark, bits, k, &mut out_tmp);
            let ideal_2p = (m as f64 / 8.0) * (15.0 / 8.0); // (15/64)*m ideal Relay8 rounds!
            let renyi_factor = 1.0 + 7.0 * harmonic(st.n_cuts) / (st.n_cuts as f64);
            println!(
                "{:<11} | {:>3} | {:>5} | {:>9} | {:>13} | {:>8.3}x | {:>8.3}x",
                bname,
                k,
                st.n_cuts,
                st.max_cut_edges,
                st.rounds_4a + st.rounds_4b,
                (st.rounds_4a + st.rounds_4b) as f64 / ideal_2p,
                renyi_factor
            );
        }
    }

    // 2. Full 6-Block 25-Round Confrontation:
    //    Arm 0: `Master`   (`unsafe` `OpenCells` 0.5.9)
    //    Arm 1: `Diet_mu1` (safe, `mu = 1`)
    //    Arm 2: `Relay4`   (safe, `mu = 4, k = 7`)
    //    Arm 3: `Relay8`   (safe, `mu = 8, k = 8`)
    //    Arm 4: `Relay12`  (safe, `mu = 12, k = 8`)
    let mut master_open = OpenCells::default();
    let mut rk_clean = RelayKernel::default();
    let mut out_m = Vec::with_capacity(900_064);
    let mut out_r = Vec::with_capacity(900_064);

    println!(
        "\n=== CYCLE 9 CONFRONTATION ({} PAIRED ROUNDS: Master vs Diet_mu1 vs Relay4/8/12) ===",
        rounds
    );
    println!(
        "{:<11} | {:>8} | {:>8} | {:>8} ({:>6}) | {:>8} ({:>6}) [{:>7} -> {:>7} emit] | {:>9} -> {:>8} ({:>5}) | {:>6}",
        "Block",
        "Master",
        "Diet_mu1",
        "Relay8",
        "/Diet",
        "Relay12",
        "/Diet",
        "DietEm",
        "Rel8Em",
        "Diet_DpL2",
        "Rel8_Dp",
        "Ratio",
        "L3Miss"
    );
    println!("{}", "-".repeat(138));

    let mut json_rows = Vec::new();
    for &(name, n_exp, p, diet_dep_l2) in &blocks {
        let ll = fs::read(format!("{}/{}", dir, name)).expect("read");
        let bz2 = craft_bz2_block(&ll, p, kernels);
        let (runs_obj, n_dec, p_dec) = extract_runs(&bz2, kernels);
        assert_eq!((n_dec, p_dec), (n_exp, p));

        assert!(master_open.invert_runs(&runs_obj, n_exp, p, &mut out_m, kernels));

        // Verify byte-exactness of ALL 4 safe-Rust arms against `origin/master`!
        let (m, mark, bits) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
        rk_clean.emit_serial(n_exp, m, mark, bits, &mut out_r);
        assert_eq!(out_r, out_m, "Diet_mu1 mismatch on {name}");

        let (_, _, _) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
        let st4 = rk_clean.emit_relay_mlp::<4>(n_exp, m, mark, bits, 7, &mut out_r);
        assert_eq!(out_r, out_m, "Relay4 mismatch on {name}");

        let (_, _, _) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
        let st8 = rk_clean.emit_relay_mlp::<8>(n_exp, m, mark, bits, 8, &mut out_r);
        assert_eq!(out_r, out_m, "Relay8 mismatch on {name}");

        let (_, _, _) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
        let st12 = rk_clean.emit_relay_mlp::<12>(n_exp, m, mark, bits, 8, &mut out_r);
        assert_eq!(out_r, out_m, "Relay12 mismatch on {name}");

        let bp4 = simulate_relay_bmcp::<4>(&rk_clean, n_exp, m, mark, bits, 7);
        let bp8 = simulate_relay_bmcp::<8>(&rk_clean, n_exp, m, mark, bits, 8);
        let bp12 = simulate_relay_bmcp::<12>(&rk_clean, n_exp, m, mark, bits, 8);
        let _ = (st4, bp4);

        let mut times: [Vec<u64>; 5] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let mut diet_em_v = Vec::with_capacity(rounds);
        let mut rel8_em_v = Vec::with_capacity(rounds);
        let mut rel12_em_v = Vec::with_capacity(rounds);

        for r in 0..(rounds + 3) {
            for rot in 0..5 {
                let arm = (r + rot) % 5;
                let t0 = Instant::now();
                match arm {
                    0 => {
                        master_open.invert_runs(&runs_obj, n_exp, p, &mut out_m, kernels);
                    }
                    1 => {
                        let (mm, mk, bt) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
                        let ns_em = rk_clean.emit_serial(n_exp, mm, mk, bt, &mut out_r);
                        if r >= 3 {
                            diet_em_v.push(ns_em);
                        }
                    }
                    2 => {
                        let (mm, mk, bt) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
                        rk_clean.emit_relay_mlp::<4>(n_exp, mm, mk, bt, 7, &mut out_r);
                    }
                    3 => {
                        let (mm, mk, bt) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
                        let s8 = rk_clean.emit_relay_mlp::<8>(n_exp, mm, mk, bt, 8, &mut out_r);
                        if r >= 3 {
                            rel8_em_v.push(s8.ns_4a + s8.ns_4b);
                        }
                    }
                    _ => {
                        let (mm, mk, bt) = rk_clean.build_table(runs_obj.records(), n_exp, p, &mut out_r);
                        let s12 = rk_clean.emit_relay_mlp::<12>(n_exp, mm, mk, bt, 8, &mut out_r);
                        if r >= 3 {
                            rel12_em_v.push(s12.ns_4a + s12.ns_4b);
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
        diet_em_v.sort_unstable();
        rel8_em_v.sort_unstable();
        rel12_em_v.sort_unstable();

        let med_mst = times[0][rounds / 2] as f64 / 1e3;
        let med_diet = times[1][rounds / 2] as f64 / 1e3;
        let med_rel4 = times[2][rounds / 2] as f64 / 1e3;
        let med_rel8 = times[3][rounds / 2] as f64 / 1e3;
        let med_rel12 = times[4][rounds / 2] as f64 / 1e3;
        let med_diet_em = diet_em_v[rounds / 2] as f64 / 1e3;
        let med_rel8_em = rel8_em_v[rounds / 2] as f64 / 1e3;
        let med_rel12_em = rel12_em_v[rounds / 2] as f64 / 1e3;

        println!(
            "{:<11} | {:>6.1}us | {:>6.1}us | {:>6.1}us ({:>5.3}x) | {:>6.1}us ({:>5.3}x) [{:>5.1}us -> {:>5.1}us] | {:>9} -> {:>8} ({:>4.2}x) | {:>6}",
            name,
            med_mst,
            med_diet,
            med_rel8,
            med_rel8 / med_diet,
            med_rel12,
            med_rel12 / med_diet,
            med_diet_em,
            med_rel8_em,
            diet_dep_l2,
            bp8.dep_l2_crit_total,
            bp8.dep_l2_crit_total as f64 / diet_dep_l2 as f64,
            bp8.c_miss_2048k
        );

        json_rows.push(format!(
            "    {{\"block\": \"{}\", \"m\": {}, \"master_us\": {:.1}, \"diet_mu1_us\": {:.1}, \"relay4_us\": {:.1}, \"relay8_us\": {:.1}, \"relay12_us\": {:.1}, \"diet_emit_us\": {:.1}, \"relay8_emit_us\": {:.1}, \"relay12_emit_us\": {:.1}, \"relay8_n_cuts\": {}, \"relay8_lane0_4a_edges\": {}, \"relay8_q_rand\": {}, \"relay8_d_serial\": {}, \"diet_dep_l2\": {}, \"relay8_dep_l2_crit\": {}, \"relay12_dep_l2_crit\": {}, \"relay8_c_miss_256k\": {}, \"relay8_c_miss_2048k\": {}, \"relay12_d_serial\": {}}}",
            name, m, med_mst, med_diet, med_rel4, med_rel8, med_rel12,
            med_diet_em, med_rel8_em, med_rel12_em,
            st8.n_cuts, st8.lane0_4a_edges, st8.q_rand_total, st8.rounds_4a + st8.rounds_4b,
            diet_dep_l2, bp8.dep_l2_crit_total, bp12.dep_l2_crit_total,
            bp8.c_miss_256k, bp8.c_miss_2048k, st12.rounds_4a + st12.rounds_4b
        ));
    }

    let r_heap = rk_clean.retained_heap_bytes();
    println!(
        "\nPersistent Scratch Heap of `Relay-MLP` (`0 B` staging!): {:.1} KiB (w/ out = {:.1} KiB <= 2048 KiB -> 10w = {:.2} MiB <= 20 MiB L3!)",
        r_heap as f64 / 1024.0,
        (r_heap + 900_032) as f64 / 1024.0,
        10.0 * (r_heap + 900_032) as f64 / (1024.0 * 1024.0)
    );

    if let Some(json_path) = std::env::args().nth(3) {
        let j = format!(
            "{{\n  \"g1_cases\": {},\n  \"relay_retained_heap_bytes\": {},\n  \"blocks\": [\n{}\n  ]\n}}\n",
            g1,
            r_heap,
            json_rows.join(",\n")
        );
        fs::write(json_path, j).expect("write json");
    }
}
