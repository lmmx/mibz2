#![forbid(unsafe_code)]

//! Cycle 48 (P1) — Analogue H: Sub-32-Bit Edge Permutations (`Edges24-Pack4`, `Edges24-Byte4`,
//! `Edges24-Split`, `Edges18-BandOnly`, and Solved Analogue H `Edges16`) over `cand_r5_head/`.
//!
//! Attacking the 817 KB (`798.0 KiB`) `edges` working set (84.4% of `cand_r5_head`'s 945.8 KiB
//! scratch) and the `#145` 10T DRAM/L3 memory-bandwidth plateau (`4.91 B/B` chaingrind DRAM).
//!
//! Exactness gates:
//!   - 5,500+ small valid & synthetic thick-strip cases (vs `cand_r5_head` & `forward_fl_invert`)
//!   - 250+ malformed multi-cycle & pure-thick-cycle columns (incl. `m >= 65,536` bidirectional!)
//!   - Forced fallbacks (`force_fallback`, `m > 262,144`, `|D| = 1`, `spiral-200k` `LONG=255` + `COLD=254`)
//!   - Solved Analogue H (`AnaH-Compact300k`, `AnaH-CompactBi`, `AnaH-D1-60k`)
//!   - 6 Admitted Wikidata blocks
//!   - All 233 Full `wikidata.bz2` blocks
//!   - 6 Real Non-Wikidata blocks (`jsonl-0..4`, `syslog-0`)

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const COLD: u8 = 254;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const PAGE_SHIFT: u32 = 7; // 128-row Dirichlet page directory

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EdgeMode {
    R5Mirror32,
    Pack4,
    Byte4,
    Split,
    BandOnly18,
}

#[derive(Default, Clone, Copy, Debug)]
struct StageTimes {
    s1_us: f64,
    s23_us: f64,
    s3b_us: f64,
    s4_us: f64,
    s5_us: f64,
}

impl StageTimes {
    fn total_us(&self) -> f64 {
        self.s1_us + self.s23_us + self.s3b_us + self.s4_us + self.s5_us
    }
}

// ============================================================================
// 1. OutQ16 (verbatim cand_r5_head Queues/Spill + 24-bit / 18-bit wire paths)
// ============================================================================

mod outq_h {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub struct Spill {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub struct Queues<'a> {
        out: &'a mut [u8],
        spill: &'a mut Spill,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        pt: [u64; 256],
        limit: u32,
        over: bool,
    }

    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    impl<'a> Queues<'a> {
        pub fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Queues {
                in_out,
                out,
                spill,
                chunks: 0,
                heads: [NONE; 256],
                pt: [u32::MAX as u64; 256],
                limit,
                over: false,
            }
        }
        pub fn over(&self) -> bool {
            self.over
        }
        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }
        #[inline(always)]
        pub fn thin_of(&self, a: usize) -> u32 {
            (self.pt[a] >> 32) as u32
        }
        pub fn extract_thin(&self) -> [u32; 256] {
            std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
        }
        #[inline(always)]
        pub fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                let at = p as usize + f;
                if let Some(dst) = self.out.get_mut(at..at + 32) {
                    let lo = m as u16;
                    let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                    for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&x.to_le_bytes());
                    }
                    let wu = u64::from(w);
                    self.pt[a] = v + ((wu << 32) | wu);
                    return;
                }
            }
            self.push_slow_pt(a, m, w);
        }
        #[cold]
        #[inline(never)]
        fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & 1023) as usize == CHUNK {
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> 10);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let tu = u64::from(take);
                self.pt[a] = v + ((tu << 32) | tu);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    self.spill.cuts.push(cut);
                }
            }
        }
        fn take_chunk_pt(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push([0; BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE);
            }
            self.spill.next[id as usize] = NONE;
            let hi = self.pt[a] & !0xFFFF_FFFFu64;
            if self.heads[a] == NONE {
                self.heads[a] = id;
            } else {
                self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id;
            }
            self.pt[a] = hi | u64::from(id << 10);
        }
        #[cold]
        #[inline(never)]
        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    pub struct Reader<'a> {
        out: &'a [u8],
        spill: &'a Spill,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    #[inline(always)]
    pub fn xor24_pack4(w: &mut [u32], idx: usize, val24: u32) {
        let bo = 3 * idx;
        let wi = bo >> 2;
        let sh = (bo & 3) << 3;
        let v64 = u64::from(val24) << sh;
        w[wi] ^= v64 as u32;
        w[wi + 1] ^= (v64 >> 32) as u32;
    }

    #[inline(always)]
    pub fn xor24_byte4(b: &mut [u8], idx: usize, val24: u32) {
        let p = 3 * idx;
        let cur = u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]]);
        b[p..p + 4].copy_from_slice(&(cur ^ val24).to_le_bytes());
    }

    #[inline(always)]
    pub fn xor18_pack(w: &mut [u32], idx: usize, val18: u32) {
        let bit = 18 * idx;
        let wi = bit >> 5;
        let sh = bit & 31;
        let v64 = u64::from(val18) << sh;
        w[wi] ^= v64 as u32;
        w[wi + 1] ^= (v64 >> 32) as u32;
    }

    impl<'a> Reader<'a> {
        pub fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
            Reader {
                out,
                spill,
                in_out: heads.in_out,
                a,
                id: heads.heads[a],
                at: 0,
                read: 0,
                epoch: 0,
                cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }

        // --- Verbatim 32-bit wire methods (R5H-A2 mirror & fallback) ---
        #[inline(always)]
        pub fn wire_xor_nomark(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 16 <= span {
                    let b = &bytes[2 * i..2 * i + 32];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                    let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let v8 = hi | (q2 as u32 & 0xFFFF);
                    let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                    let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                    let v11 = hi | ((q2 >> 48) as u32);
                    let v12 = hi | (q3 as u32 & 0xFFFF);
                    let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                    let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                    let v15 = hi | ((q3 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 16];
                    eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                    eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                    eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                    edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                    edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                    edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                    edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                    edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                    i += 16;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        #[inline(always)]
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * BYTES + 2 * self.at;
                for (e, s) in dst.iter_mut().zip(self.out[at..at + 2 * span].chunks_exact(2)) {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = dst.len().min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                for (e, s) in now.iter_mut().zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2)) {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = rest;
            }
        }

        // --- Edges24-Pack4 (Vec<u32>, 4 x 24b in 3 x u32) ---
        #[inline(always)]
        pub fn wire_xor_pack4_nomark(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack24: u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                // Peel to 4-row alignment so sequential writes use 3 x u32 per 4 edges!
                while i < span && ((dst_start + i) & 3) != 0 {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    xor24_pack4(edges, u, pack24 | v);
                    xor24_pack4(edges, v as usize, u as u32);
                    i += 1;
                }
                while i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let e0 = pack24 | v0;
                    let e1 = pack24 | v1;
                    let e2 = pack24 | v2;
                    let e3 = pack24 | v3;
                    let u0 = dst_start + i;
                    let wi = (3 * u0) >> 2;
                    let ew = &mut edges[wi..wi + 3];
                    ew[0] ^= e0 | (e1 << 24);
                    ew[1] ^= (e1 >> 8) | (e2 << 16);
                    ew[2] ^= (e2 >> 16) | (e3 << 8);
                    xor24_pack4(edges, v0 as usize, u0 as u32);
                    xor24_pack4(edges, v1 as usize, (u0 + 1) as u32);
                    xor24_pack4(edges, v2 as usize, (u0 + 2) as u32);
                    xor24_pack4(edges, v3 as usize, (u0 + 3) as u32);
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    xor24_pack4(edges, u, pack24 | v);
                    xor24_pack4(edges, v as usize, u as u32);
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_pack4_slow(edges, dst_start, span, pack24, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor_pack4(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack24: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    xor24_pack4(edges, u, pack24 | v);
                    xor24_pack4(edges, v as usize, u as u32);
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_pack4_slow(edges, dst_start, span, pack24, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_pack4_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack24: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    xor24_pack4(edges, u, pack24 | v);
                    xor24_pack4(edges, v as usize, u as u32);
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        /// Unidirectional Pack4 (m < 65,536, zero reverse scatter; assumes zeroed edges!)
        #[inline(always)]
        pub fn wire_uni_pack4(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack24: u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                let bytes = &chunk[2 * self.at..2 * (self.at + take)];
                let mut i = 0usize;
                while i < take && ((dst_start + i) & 3) != 0 {
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    xor24_pack4(edges, dst_start + i, pack24 | v);
                    i += 1;
                }
                while i + 4 <= take {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let e0 = pack24 | hi | (q as u32 & 0xFFFF);
                    let e1 = pack24 | hi | ((q >> 16) as u32 & 0xFFFF);
                    let e2 = pack24 | hi | ((q >> 32) as u32 & 0xFFFF);
                    let e3 = pack24 | hi | ((q >> 48) as u32);
                    let wi = (3 * (dst_start + i)) >> 2;
                    let ew = &mut edges[wi..wi + 3];
                    ew[0] |= e0 | (e1 << 24);
                    ew[1] |= (e1 >> 8) | (e2 << 16);
                    ew[2] |= (e2 >> 16) | (e3 << 8);
                    i += 4;
                }
                while i < take {
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    xor24_pack4(edges, dst_start + i, pack24 | v);
                    i += 1;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        // --- Edges24-Byte4 (Vec<u8>, 3 B/row unaligned 4-byte RMW) ---
        #[inline(always)]
        pub fn wire_xor_byte4_nomark(
            &mut self,
            edges: &mut [u8],
            dst_start: usize,
            span: usize,
            pack24: u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut i = 0usize;
                while i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let e0 = pack24 | v0;
                    let e1 = pack24 | v1;
                    let e2 = pack24 | v2;
                    let e3 = pack24 | v3;
                    let u0 = dst_start + i;
                    let p0 = 3 * u0;
                    let w0 = u32::from_le_bytes([edges[p0], edges[p0 + 1], edges[p0 + 2], edges[p0 + 3]])
                        ^ (e0 | (e1 << 24));
                    let w1 = u32::from_le_bytes([edges[p0 + 4], edges[p0 + 5], edges[p0 + 6], edges[p0 + 7]])
                        ^ ((e1 >> 8) | (e2 << 16));
                    let w2 = u32::from_le_bytes([edges[p0 + 8], edges[p0 + 9], edges[p0 + 10], edges[p0 + 11]])
                        ^ ((e2 >> 16) | (e3 << 8));
                    edges[p0..p0 + 4].copy_from_slice(&w0.to_le_bytes());
                    edges[p0 + 4..p0 + 8].copy_from_slice(&w1.to_le_bytes());
                    edges[p0 + 8..p0 + 12].copy_from_slice(&w2.to_le_bytes());
                    xor24_byte4(edges, v0 as usize, u0 as u32);
                    xor24_byte4(edges, v1 as usize, (u0 + 1) as u32);
                    xor24_byte4(edges, v2 as usize, (u0 + 2) as u32);
                    xor24_byte4(edges, v3 as usize, (u0 + 3) as u32);
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    xor24_byte4(edges, u, pack24 | v);
                    xor24_byte4(edges, v as usize, u as u32);
                    i += 1;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_byte4_slow(edges, dst_start, span, pack24, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor_byte4(
            &mut self,
            edges: &mut [u8],
            dst_start: usize,
            span: usize,
            pack24: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    xor24_byte4(edges, u, pack24 | v);
                    xor24_byte4(edges, v as usize, u as u32);
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_byte4_slow(edges, dst_start, span, pack24, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_byte4_slow(
            &mut self,
            edges: &mut [u8],
            mut dst_start: usize,
            mut left: usize,
            pack24: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    xor24_byte4(edges, u, pack24 | v);
                    xor24_byte4(edges, v as usize, u as u32);
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        pub fn wire_uni_byte4(
            &mut self,
            edges: &mut [u8],
            mut dst_start: usize,
            mut left: usize,
            pack24: u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    let p = 3 * (dst_start + i);
                    let val = pack24 | v;
                    edges[p] = val as u8;
                    edges[p + 1] = (val >> 8) as u8;
                    edges[p + 2] = (val >> 16) as u8;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        // --- Edges24-Split (Vec<u16> low-16 + Vec<u8> high-2-next | hot_code) ---
        #[inline(always)]
        pub fn wire_xor_split_nomark(
            &mut self,
            lo: &mut [u16],
            hi_arr: &mut [u8],
            dst_start: usize,
            span: usize,
            pack_hi: u8,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let ep = self.epoch as u8;
                let ep_u32 = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let uh_xor = pack_hi ^ ep;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let vlo = u16::from_le_bytes([s[0], s[1]]);
                    let v = (ep_u32 | u32::from(vlo)) as usize;
                    lo[u] ^= vlo;
                    hi_arr[u] ^= uh_xor;
                    lo[v] ^= u as u16;
                    let u_hi = (u >> 16) as u8;
                    if u_hi != 0 {
                        hi_arr[v] ^= u_hi;
                    }
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_split_slow(lo, hi_arr, dst_start, span, pack_hi, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor_split(
            &mut self,
            lo: &mut [u16],
            hi_arr: &mut [u8],
            dst_start: usize,
            span: usize,
            pack_hi: u8,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let ep = self.epoch as u8;
                let ep_u32 = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let uh_xor = pack_hi ^ ep;
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let vlo = u16::from_le_bytes([s[0], s[1]]);
                    let v = ep_u32 | u32::from(vlo);
                    if v == mark {
                        before = u as u32;
                    }
                    lo[u] ^= vlo;
                    hi_arr[u] ^= uh_xor;
                    lo[v as usize] ^= u as u16;
                    let u_hi = (u >> 16) as u8;
                    if u_hi != 0 {
                        hi_arr[v as usize] ^= u_hi;
                    }
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_split_slow(lo, hi_arr, dst_start, span, pack_hi, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_split_slow(
            &mut self,
            lo: &mut [u16],
            hi_arr: &mut [u8],
            mut dst_start: usize,
            mut left: usize,
            pack_hi: u8,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let ep = self.epoch as u8;
                let ep_u32 = self.epoch << 16;
                let uh_xor = pack_hi ^ ep;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let vlo = u16::from_le_bytes([s[0], s[1]]);
                    let v = ep_u32 | u32::from(vlo);
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    lo[u] ^= vlo;
                    hi_arr[u] ^= uh_xor;
                    lo[v as usize] ^= u as u16;
                    let u_hi = (u >> 16) as u8;
                    if u_hi != 0 {
                        hi_arr[v as usize] ^= u_hi;
                    }
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        pub fn wire_uni_split(
            &mut self,
            lo: &mut [u16],
            hi_arr: &mut [u8],
            mut dst_start: usize,
            mut left: usize,
            pack_hi: u8,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let ep = self.epoch as u8;
                let uh = pack_hi | ep;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    lo[dst_start + i] = u16::from_le_bytes([s[0], s[1]]);
                    hi_arr[dst_start + i] = uh;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        // --- Edges18-BandOnly (Vec<u32>, 18 bits/row next_xor, 0 code bits!) ---
        #[inline(always)]
        pub fn wire_xor_e18_nomark(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    xor18_pack(edges, u, v);
                    xor18_pack(edges, v as usize, u as u32);
                }
                self.at += span;
                self.read += span as u32;
            } else {
                let mut dummy = 0u32;
                self.wire_xor_e18_slow(edges, dst_start, span, u32::MAX, &mut dummy);
            }
        }

        #[inline(always)]
        pub fn wire_xor_e18(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    xor18_pack(edges, u, v);
                    xor18_pack(edges, v as usize, u as u32);
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_e18_slow(edges, dst_start, span, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_e18_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    xor18_pack(edges, u, v);
                    xor18_pack(edges, v as usize, u as u32);
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        pub fn wire_uni_e18(&mut self, edges: &mut [u32], mut dst_start: usize, mut left: usize) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(CHUNK - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    xor18_pack(edges, dst_start + i, v);
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        /// Solved Analogue H degenerate `|D| == 1` (`Edges16`, 2 bytes/row!)
        pub fn wire_uni_e16(&mut self, dst: &mut [u16]) {
            let mut off = 0usize;
            while off < dst.len() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self.spill.cuts.get(self.epoch as usize).map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = (dst.len() - off)
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                for (d, s) in dst[off..off + take]
                    .iter_mut()
                    .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
                {
                    *d = u16::from_le_bytes([s[0], s[1]]);
                }
                self.at += take;
                self.read += take as u32;
                off += take;
            }
        }
    }
}

// ============================================================================
// 2. Verbatim S1-S3a Core (matching cand_r5_head bit-for-bit) + Analogue H S3b/S4/S5
// ============================================================================

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    #[inline(always)]
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
    }
}

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            mark_byte: 0,
        }
    }
}

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    if inline16.len() < 256 {
        inline16.clear();
        inline16.extend((0..=255u8).map(|c| {
            let mut v = [0; INLINE];
            v[0] = c;
            v
        }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8)
            .rotate_left(29)
            .wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

trait Links {
    const STRIDE: usize;
    const IS_NARROW: bool;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
    const BOTTOM: usize = 1;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
    const BOTTOM: usize = 2;
    #[inline(always)]
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    #[inline(always)]
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    #[inline(always)]
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

#[inline(always)]
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | u32::from(byte)
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links> Elim<'_, L> {
    #[inline(always)]
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    #[inline(always)]
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    #[inline(always)]
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    #[inline(always)]
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }
    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1);
            let prev = L::prev(self.a, last, 1);
            let width = self.width(first);
            hi -= self.width(last);
            self.set_bottom(first, lo);
            self.set_bottom(last, hi);
            lo += width;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            self.set_bottom(first, lo);
            lo += self.width(first);
        }
        assert_eq!(lo, hi);
    }
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes;
        self.nodes += 1;
        self.set_width(id, width);
        self.set_word(id, word);
        id
    }
    #[inline(always)]
    fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 {
            *head = id;
        } else {
            L::set_next(self.a, *tail, side, id);
        }
        L::set_prev(self.a, id, side, *tail);
        *tail = id;
    }
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }
    #[inline(always)]
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }
    #[inline(always)]
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }
    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + (id - 65_793) as usize * RULE
    }
    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 2 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }
    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum == 2 {
                return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + 65_793;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }
    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base);
        }
    }
    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_tail(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        *remaining = left;
        done
    }
    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(word, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

struct Dictionary<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let uid = id + 1 + (u32::from(g == 2) << 8);
        let key = (uid << 8) | u32::from(byte);
        let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
        let slot = bucket * 2;
        if self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }
    fn code_miss<L: Links>(
        &mut self,
        e: &mut Elim<L>,
        handle: u32,
        id: u32,
        byte: u8,
        key: u32,
        slot: usize,
    ) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            let code = if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                ent & 0x00FF_FFFF
            } else {
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                c
            };
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            return code;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        let code = if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            ent & 0x00FF_FFFF
        } else {
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }
    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

struct HKernel {
    spill: outq_h::Spill,
    stack: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    // Analogue H compact structures
    edges_bytes: Vec<u8>,
    edges_lo: Vec<u16>,
    edges_hi: Vec<u8>,
    // Hot dictionary (4.25 KiB L1D) + cold-band interval table
    hot_inline: [[u8; INLINE]; 256],
    hot_lens: [u8; 256],
    freq: Vec<u32>,
    ranked: Vec<u64>,
    code_to_hot: Vec<u8>,
    cold_bands: Vec<u32>, // packed `(last_dst << 14) | orig_code`
    cold_dir: Vec<u16>,   // Dirichlet page-128 directory
    force_fallback: bool,
}

impl Default for HKernel {
    fn default() -> Self {
        HKernel {
            spill: outq_h::Spill::default(),
            stack: Vec::new(),
            bands: Vec::new(),
            spelling: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            edges_bytes: Vec::new(),
            edges_lo: Vec::new(),
            edges_hi: Vec::new(),
            hot_inline: [[0; INLINE]; 256],
            hot_lens: [0; 256],
            freq: Vec::new(),
            ranked: Vec::new(),
            code_to_hot: Vec::new(),
            cold_bands: Vec::new(),
            cold_dir: Vec::new(),
            force_fallback: false,
        }
    }
}

struct Pass<'a> {
    queues: outq_h::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIMIT {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p);
        }
    }
    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        let end = pos + width;
        if pos < p {
            self.piece(c, pos, p - pos, false, p);
        }
        self.piece(c, p, 1, true, p);
        if p + 1 < end {
            self.piece(c, p + 1, end - p - 1, false, p);
        }
    }
    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }
            self.queues.push_thin(a, f.m, w);
            f.m += w;
        } else {
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                self.edges.reserve_exact(need - self.edges.len());
                self.edges.resize(need, 0);
            }
            let thin_a = self.queues.thin_of(a);
            self.edges[at..at + 3].copy_from_slice(&[
                x,
                thin_a + f.thick_rows[a],
                (w << 8) | u32::from(c),
            ]);
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[inline(always)]
fn get24_pack4(w: &[u32], u: usize) -> u32 {
    let bo = 3 * u;
    let wi = bo >> 2;
    let sh = (bo & 3) << 3;
    let pair = u64::from(w[wi]) | (u64::from(w[wi + 1]) << 32);
    ((pair >> sh) as u32) & 0x00FF_FFFF
}

#[inline(always)]
fn get24_byte4(b: &[u8], u: usize) -> u32 {
    let p = 3 * u;
    u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]]) & 0x00FF_FFFF
}

#[inline(always)]
fn get18_pack(w: &[u32], u: usize) -> u32 {
    let bit = 18 * u;
    let wi = bit >> 5;
    let sh = bit & 31;
    let pair = u64::from(w[wi]) | (u64::from(w[wi + 1]) << 32);
    ((pair >> sh) as u32) & 0x3_FFFF
}

#[inline(always)]
fn lookup_cold_band(cold_bands: &[u32], cold_dir: &[u16], u: u32) -> usize {
    let mut idx = cold_dir[(u >> PAGE_SHIFT) as usize] as usize;
    let key = u << 14;
    while cold_bands[idx] < key {
        idx += 1;
    }
    (cold_bands[idx] & 0x3FFF) as usize
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

impl HKernel {
    fn scratch_report(&self, mode: EdgeMode) -> (usize, String) {
        use std::mem::size_of;
        let q = self.spill.scratch_bytes();
        let bands = size_of::<Band>() * self.bands.capacity();
        let spelling = self.spelling.capacity();
        let edges = 4 * self.edges.capacity();
        let extra = 4 * self.extra.capacity();
        let inline16 = 16 * self.inline16.capacity();
        let lens = self.lens.capacity();
        let ep = 4 * self.stack.capacity();
        let ebytes = self.edges_bytes.capacity();
        let esplit = 2 * self.edges_lo.capacity() + self.edges_hi.capacity();
        let aux = 4 * self.cold_bands.capacity()
            + 2 * self.cold_dir.capacity()
            + 4 * self.freq.capacity()
            + 8 * self.ranked.capacity()
            + self.code_to_hot.capacity();
        let hot_fixed = if mode == EdgeMode::R5Mirror32 || mode == EdgeMode::BandOnly18 {
            0
        } else {
            16 * 256 + 256
        };
        let tot = q + bands + spelling + edges + extra + inline16 + lens + ep + ebytes + esplit + aux + hot_fixed;
        let s = format!(
            "tot={:.1}KiB (edges_u32={:.1}K ebytes={:.1}K esplit={:.1}K cold+aux={:.1}K hot={:.1}K bands={:.1}K spell={:.1}K dict={:.1}K q={:.1}K)",
            tot as f64 / 1024.0,
            edges as f64 / 1024.0,
            ebytes as f64 / 1024.0,
            esplit as f64 / 1024.0,
            aux as f64 / 1024.0,
            hot_fixed as f64 / 1024.0,
            bands as f64 / 1024.0,
            spelling as f64 / 1024.0,
            (inline16 + lens) as f64 / 1024.0,
            q as f64 / 1024.0,
        );
        (tot, s)
    }

    /// Active S4+S5 working set bytes touched during wiring & emit for the last block
    fn active_s45_bytes(&self, mode: EdgeMode, m: usize) -> usize {
        if self.force_fallback || mode == EdgeMode::R5Mirror32 || m > (1 << 18) {
            return 4 * m + 4 * self.extra.len();
        }
        let cold = 4 * self.cold_bands.len() + 2 * self.cold_dir.len();
        match mode {
            EdgeMode::R5Mirror32 => 4 * m + 4 * self.extra.len(),
            EdgeMode::Pack4 => ((3 * m) >> 2) * 4 + 8 + cold,
            EdgeMode::Byte4 => 3 * m + 4 + cold,
            EdgeMode::Split => 2 * m + m + cold,
            EdgeMode::BandOnly18 => ((18 * m) >> 5) * 4 + 8 + cold,
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        most: usize,
    ) -> [usize; 256] {
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= 4 * ns + nodes_words);
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let idx = i as usize;
            let to = strips_raw[3 * idx + 1];
            let width = strips_raw[3 * idx + 2] >> 8;
            if x < to {
                widest = widest.max(to - x);
                let id = e.node(to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = to + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
            e = Elim {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        e.assign_bottom(m);
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS { header } else { tail };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);
        if cache.len() < CACHE_WORDS {
            let mut cursors = [0; 256];
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(
                            node_word,
                            a as u8,
                            &mut self.stack,
                            &mut self.spelling,
                        );
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    self.bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }
        cache[..CACHE_WORDS].fill(0);
        let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
        let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
        let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16: &mut self.inline16,
            lens: &mut self.lens,
            spelling: &mut self.spelling,
            stack: &mut self.stack,
        };
        let mut cursors = [0; 256];
        let mut id = L::next(e.a, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let width = (end - k).min(node_width - in_node);
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let c = dictionary.code(&mut e, node_word, a as u8);
                    last_w = node_word;
                    last_c = c;
                    c
                };
                self.bands.push(Band::new(width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = next_id;
                    in_node = 0;
                    if id != 0 {
                        (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    }
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }

    /// Step H3'+H4': Pareto-Zipf top-K heavy-hitter selection (Hoare Quickselect O(|D|))
    /// + Dirichlet page-128 Cold-Band Interval Table (`(last_dst << 14) | orig_code`).
    /// Returns `(has_cold, single_code_opt)` where `single_code_opt = Some(c)` iff `used_codes == 1`.
    fn build_hot_and_cold(
        &mut self,
        m: usize,
        hot_bits: u32,
    ) -> Option<(bool, Option<usize>)> {
        let dict_len = self.lens.len();
        if m > (1 << 18) || dict_len > (1 << 14) {
            return None;
        }
        self.freq.clear();
        self.freq.resize(dict_len, 0);
        for &b in &self.bands {
            self.freq[b.code() as usize] += b.end();
        }
        self.ranked.clear();
        for (c, &f) in self.freq.iter().enumerate() {
            if f > 0 {
                // Pack `(freq << 16) | code` for fast descending selection
                self.ranked.push((u64::from(f) << 16) | (c as u64));
            }
        }
        let used = self.ranked.len();
        if used == 1 {
            let only_code = (self.ranked[0] & 0xFFFF) as usize;
            self.cold_bands.clear();
            self.cold_dir.clear();
            return Some((false, Some(only_code)));
        }
        let max_slots = 1usize << hot_bits;
        let esc = (max_slots - 1) as u8;
        let need_escape = used > max_slots;
        let num_hot = if need_escape { max_slots - 1 } else { used };

        if need_escape {
            // Hoare Quickselect (1961): O(|D|) partition placing top `num_hot` in `[0..num_hot]`
            self.ranked
                .select_nth_unstable_by(num_hot - 1, |a, b| b.cmp(a));
        }

        self.code_to_hot.clear();
        self.code_to_hot.resize(dict_len, esc);
        for h in 0..num_hot {
            let orig = (self.ranked[h] & 0xFFFF) as usize;
            self.code_to_hot[orig] = h as u8;
            self.hot_inline[h] = self.inline16[orig];
            self.hot_lens[h] = self.lens[orig];
        }
        if need_escape {
            self.hot_lens[usize::from(esc)] = COLD;
        }

        self.cold_bands.clear();
        self.cold_dir.clear();
        if need_escape {
            for &b in &self.bands {
                let c = b.code();
                if self.code_to_hot[c as usize] == esc {
                    let last = b.dst() + b.end() - 1;
                    self.cold_bands.push((last << 14) | c);
                }
            }
            if self.cold_bands.len() >= usize::from(u16::MAX) {
                return None;
            }
            self.cold_bands.sort_unstable();
            let num_pages = (m + (1 << PAGE_SHIFT) - 1) >> PAGE_SHIFT;
            self.cold_dir.resize(num_pages, 0);
            let mut cur = 0usize;
            let len = self.cold_bands.len();
            for pg in 0..num_pages {
                let key = ((pg as u32) << PAGE_SHIFT) << 14;
                while cur < len && self.cold_bands[cur] < key {
                    cur += 1;
                }
                self.cold_dir[pg] = cur as u16;
            }
        }
        Some((need_escape, None))
    }

    /// Build full Dirichlet page-128 band directory for `Edges18-BandOnly` (all bands).
    fn build_all_bands_dir(&mut self, m: usize) -> bool {
        let dict_len = self.lens.len();
        if m > (1 << 18) || dict_len > (1 << 14) || self.bands.len() >= usize::from(u16::MAX) {
            return false;
        }
        self.cold_bands.clear();
        for &b in &self.bands {
            let last = b.dst() + b.end() - 1;
            self.cold_bands.push((last << 14) | b.code());
        }
        self.cold_bands.sort_unstable();
        let num_pages = (m + (1 << PAGE_SHIFT) - 1) >> PAGE_SHIFT;
        self.cold_dir.clear();
        self.cold_dir.resize(num_pages, 0);
        let mut cur = 0usize;
        let len = self.cold_bands.len();
        for pg in 0..num_pages {
            let key = ((pg as u32) << PAGE_SHIFT) << 14;
            while cur < len && self.cold_bands[cur] < key {
                cur += 1;
            }
            self.cold_dir[pg] = cur as u16;
        }
        true
    }

    #[inline(never)]
    fn run<const TIMED: bool>(
        &mut self,
        mode: EdgeMode,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (usize, StageTimes) {
        let mut tm = StageTimes::default();
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);
        let (ns, f, heads) = {
            let mut pass = Pass {
                queues: outq_h::Queues::new(&mut out[..], &mut self.spill, u32::MAX),
                edges: &mut self.edges,
                ns: 0,
                f: First::default(),
            };
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= LIMIT {
                    pass.piece(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take(c, pos, width, p);
                    pos += width;
                }
            }
            let ns = pass.ns;
            let mut f = pass.f;
            f.thin = pass.queues.extract_thin();
            assert!(!pass.queues.over());
            (ns, f, pass.queues.heads())
        };
        if let Some(t0) = t_s1 {
            tm.s1_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s23 = if TIMED { Some(Instant::now()) } else { None };
        let m = f.m as usize;
        let all: [u32; 256] = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(f.thick);
        let most = 3 * ns + 3;
        let wide_nodes = most > NARROW_NODES;
        let stride = if wide_nodes { Wide::STRIDE } else { Narrow::STRIDE };
        // Crucial Analogue H persistent-WS sizing:
        // In Pack4/BandOnly18, edges needs only 0.75*m / 0.5625*m words instead of m words!
        let min_edge_words = match mode {
            EdgeMode::R5Mirror32 => m,
            EdgeMode::Pack4 => ((3 * m) >> 2) + 2,
            EdgeMode::Byte4 | EdgeMode::Split => 0,
            EdgeMode::BandOnly18 => ((18 * m) >> 5) + 2,
        };
        let need = (4 * ns + (stride + RULE) * most).max(min_edge_words);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        self.bands.clear();
        if n >= 65_536 && self.bands.capacity() < 8192 {
            self.bands.reserve_exact(8192);
        }
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        let cursors = if wide_nodes {
            self.induced_with::<Wide>(ns, n as u32, f.m, &f.thin, &first, &mut write, most)
        } else {
            self.induced_with::<Narrow>(ns, n as u32, f.m, &f.thin, &first, &mut write, most)
        };
        if let Some(t0) = t_s23 {
            tm.s23_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let use_fallback = self.force_fallback || mode == EdgeMode::R5Mirror32 || bits > 18;

        // S3b: Build hot/cold tables for compact modes
        let t_s3b = if TIMED { Some(Instant::now()) } else { None };
        let compact_bits = bits.max(16); // clamp to {16, 17, 18} -> hot_bits in {8, 7, 6}
        let hot_bits = 24 - compact_bits;
        let prep_ok = if use_fallback {
            None
        } else if mode == EdgeMode::BandOnly18 {
            if self.build_all_bands_dir(m) {
                Some((true, None))
            } else {
                None
            }
        } else {
            self.build_hot_and_cold(m, hot_bits)
        };
        if let Some(t0) = t_s3b {
            tm.s3b_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        // Unconditional exact 32-bit fallback (R5Mirror32 OR guard triggered)
        let Some((_has_cold, single_code)) = prep_ok else {
            self.run_s45_r5_32::<TIMED>(n, m, f.mark, f.mark_byte, bits, &cursors, &heads, out, &mut tm);
            return (m, tm);
        };

        // Solved Analogue H degenerate `|D| == 1` (`Edges16`, 2 bytes/row!)
        if let Some(only_code) = single_code {
            if m <= 65_536 && mode != EdgeMode::BandOnly18 {
                let t_s4 = if TIMED { Some(Instant::now()) } else { None };
                if self.edges_lo.len() < m {
                    let add = m - self.edges_lo.len();
                    self.edges_lo.reserve_exact(add);
                    self.edges_lo.resize(m, 0);
                }
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let dst = b.dst() as usize;
                        let span = b.end() as usize;
                        queue.wire_uni_e16(&mut self.edges_lo[dst..dst + span]);
                    }
                }
                if let Some(t0) = t_s4 {
                    tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                let t_s5 = if TIMED { Some(Instant::now()) } else { None };
                emit_e16_single(
                    &self.edges_lo[..m],
                    only_code,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    out,
                );
                if let Some(t0) = t_s5 {
                    tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                return (m, tm);
            }
        }

        // Dispatch compact S4 + S5 by mode and compile-time `compact_bits ∈ {16, 17, 18}`
        match mode {
            EdgeMode::Pack4 => {
                let t_s4 = if TIMED { Some(Instant::now()) } else { None };
                let words_need = ((3 * m) >> 2) + 2;
                if self.edges.len() < words_need {
                    let add = words_need - self.edges.len();
                    self.edges.reserve_exact(add);
                    self.edges.resize(words_need, 0);
                }
                let edges = &mut self.edges[..words_need];
                edges.fill(0);
                let bi = m >= 65_536;
                let mut predecessor = f.mark;
                if bi {
                    let mb = usize::from(f.mark_byte);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        if a != mb {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = u32::from(self.code_to_hot[b.code() as usize]);
                                queue.wire_xor_pack4_nomark(
                                    edges,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << compact_bits,
                                );
                            }
                        } else {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = u32::from(self.code_to_hot[b.code() as usize]);
                                queue.wire_xor_pack4(
                                    edges,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << compact_bits,
                                    f.mark,
                                    &mut predecessor,
                                );
                            }
                        }
                    }
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        for &b in &self.bands[cursors[a]..end] {
                            let hc = u32::from(self.code_to_hot[b.code() as usize]);
                            queue.wire_uni_pack4(
                                edges,
                                b.dst() as usize,
                                b.end() as usize,
                                hc << compact_bits,
                            );
                        }
                    }
                }
                if let Some(t0) = t_s4 {
                    tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                let t_s5 = if TIMED { Some(Instant::now()) } else { None };
                if bi {
                    match compact_bits {
                        17 => emit_bi_pack4::<17>(
                            &self.edges[..words_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        18 => emit_bi_pack4::<18>(
                            &self.edges[..words_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        _ => emit_bi_pack4::<16>(
                            &self.edges[..words_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                    }
                } else {
                    emit_uni_pack4::<16>(
                        &self.edges[..words_need],
                        &self.hot_inline,
                        &self.hot_lens,
                        &self.cold_bands,
                        &self.cold_dir,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        out,
                    );
                }
                if let Some(t0) = t_s5 {
                    tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
                }
            }
            EdgeMode::Byte4 => {
                let t_s4 = if TIMED { Some(Instant::now()) } else { None };
                let bytes_need = 3 * m + 4;
                if self.edges_bytes.len() < bytes_need {
                    let add = bytes_need - self.edges_bytes.len();
                    self.edges_bytes.reserve_exact(add);
                    self.edges_bytes.resize(bytes_need, 0);
                }
                let edges = &mut self.edges_bytes[..bytes_need];
                let bi = m >= 65_536;
                let mut predecessor = f.mark;
                if bi {
                    edges.fill(0);
                    let mb = usize::from(f.mark_byte);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        if a != mb {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = u32::from(self.code_to_hot[b.code() as usize]);
                                queue.wire_xor_byte4_nomark(
                                    edges,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << compact_bits,
                                );
                            }
                        } else {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = u32::from(self.code_to_hot[b.code() as usize]);
                                queue.wire_xor_byte4(
                                    edges,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << compact_bits,
                                    f.mark,
                                    &mut predecessor,
                                );
                            }
                        }
                    }
                } else {
                    // Unidirectional: direct byte writes, only clear trailing pad!
                    edges[3 * m..bytes_need].fill(0);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        for &b in &self.bands[cursors[a]..end] {
                            let hc = u32::from(self.code_to_hot[b.code() as usize]);
                            queue.wire_uni_byte4(
                                edges,
                                b.dst() as usize,
                                b.end() as usize,
                                hc << compact_bits,
                            );
                        }
                    }
                }
                if let Some(t0) = t_s4 {
                    tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                let t_s5 = if TIMED { Some(Instant::now()) } else { None };
                if bi {
                    match compact_bits {
                        17 => emit_bi_byte4::<17>(
                            &self.edges_bytes[..bytes_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        18 => emit_bi_byte4::<18>(
                            &self.edges_bytes[..bytes_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        _ => emit_bi_byte4::<16>(
                            &self.edges_bytes[..bytes_need],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                    }
                } else {
                    emit_uni_byte4::<16>(
                        &self.edges_bytes[..bytes_need],
                        &self.hot_inline,
                        &self.hot_lens,
                        &self.cold_bands,
                        &self.cold_dir,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        out,
                    );
                }
                if let Some(t0) = t_s5 {
                    tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
                }
            }
            EdgeMode::Split => {
                let t_s4 = if TIMED { Some(Instant::now()) } else { None };
                if self.edges_lo.len() < m {
                    let add = m - self.edges_lo.len();
                    self.edges_lo.reserve_exact(add);
                    self.edges_lo.resize(m, 0);
                }
                if self.edges_hi.len() < m {
                    let add = m - self.edges_hi.len();
                    self.edges_hi.reserve_exact(add);
                    self.edges_hi.resize(m, 0);
                }
                let lo = &mut self.edges_lo[..m];
                let hi_arr = &mut self.edges_hi[..m];
                let hi_shift = compact_bits - 16; // 0, 1, or 2
                let bi = m >= 65_536;
                let mut predecessor = f.mark;
                if bi {
                    lo.fill(0);
                    hi_arr.fill(0);
                    let mb = usize::from(f.mark_byte);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        if a != mb {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = self.code_to_hot[b.code() as usize];
                                queue.wire_xor_split_nomark(
                                    lo,
                                    hi_arr,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << hi_shift,
                                );
                            }
                        } else {
                            for &b in &self.bands[cursors[a]..end] {
                                let hc = self.code_to_hot[b.code() as usize];
                                queue.wire_xor_split(
                                    lo,
                                    hi_arr,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    hc << hi_shift,
                                    f.mark,
                                    &mut predecessor,
                                );
                            }
                        }
                    }
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        for &b in &self.bands[cursors[a]..end] {
                            let hc = self.code_to_hot[b.code() as usize];
                            queue.wire_uni_split(
                                lo,
                                hi_arr,
                                b.dst() as usize,
                                b.end() as usize,
                                hc << hi_shift,
                            );
                        }
                    }
                }
                if let Some(t0) = t_s4 {
                    tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                let t_s5 = if TIMED { Some(Instant::now()) } else { None };
                if bi {
                    match hi_shift {
                        1 => emit_bi_split::<1>(
                            &self.edges_lo[..m],
                            &self.edges_hi[..m],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        2 => emit_bi_split::<2>(
                            &self.edges_lo[..m],
                            &self.edges_hi[..m],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                        _ => emit_bi_split::<0>(
                            &self.edges_lo[..m],
                            &self.edges_hi[..m],
                            &self.hot_inline,
                            &self.hot_lens,
                            &self.cold_bands,
                            &self.cold_dir,
                            &self.spelling,
                            &self.inline16,
                            &self.lens,
                            n,
                            m,
                            f.mark,
                            predecessor,
                            out,
                        ),
                    }
                } else {
                    emit_uni_split(
                        &self.edges_lo[..m],
                        &self.edges_hi[..m],
                        &self.hot_inline,
                        &self.hot_lens,
                        &self.cold_bands,
                        &self.cold_dir,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        out,
                    );
                }
                if let Some(t0) = t_s5 {
                    tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
                }
            }
            EdgeMode::BandOnly18 => {
                let t_s4 = if TIMED { Some(Instant::now()) } else { None };
                let words_need = ((18 * m) >> 5) + 2;
                if self.edges.len() < words_need {
                    let add = words_need - self.edges.len();
                    self.edges.reserve_exact(add);
                    self.edges.resize(words_need, 0);
                }
                let edges = &mut self.edges[..words_need];
                edges.fill(0);
                let bi = m >= 65_536;
                let mut predecessor = f.mark;
                if bi {
                    let mb = usize::from(f.mark_byte);
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        if a != mb {
                            for &b in &self.bands[cursors[a]..end] {
                                queue.wire_xor_e18_nomark(edges, b.dst() as usize, b.end() as usize);
                            }
                        } else {
                            for &b in &self.bands[cursors[a]..end] {
                                queue.wire_xor_e18(
                                    edges,
                                    b.dst() as usize,
                                    b.end() as usize,
                                    f.mark,
                                    &mut predecessor,
                                );
                            }
                        }
                    }
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq_h::Reader::new(out, &self.spill, &heads, a);
                        for &b in &self.bands[cursors[a]..end] {
                            queue.wire_uni_e18(edges, b.dst() as usize, b.end() as usize);
                        }
                    }
                }
                if let Some(t0) = t_s4 {
                    tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
                }
                let t_s5 = if TIMED { Some(Instant::now()) } else { None };
                if bi {
                    emit_bi_e18(
                        &self.edges[..words_need],
                        &self.cold_bands,
                        &self.cold_dir,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        predecessor,
                        out,
                    );
                } else {
                    emit_uni_e18(
                        &self.edges[..words_need],
                        &self.cold_bands,
                        &self.cold_dir,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        out,
                    );
                }
                if let Some(t0) = t_s5 {
                    tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
                }
            }
            EdgeMode::R5Mirror32 => unreachable!(),
        }

        (m, tm)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_s45_r5_32<const TIMED: bool>(
        &mut self,
        n: usize,
        m: usize,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        cursors: &[usize; 256],
        heads: &outq_h::Heads,
        out: &mut Vec<u8>,
        tm: &mut StageTimes,
    ) {
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            let add = m - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        let mut pred_opt = None;
        if !wide && m >= 65_536 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq_h::Reader::new(out, &self.spill, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }
            pred_opt = Some(predecessor);
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq_h::Reader::new(out, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                }
            }
        }
        if let Some(t0) = t_s4 {
            tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
        }
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = pred_opt {
            emit_bi_r5_32(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                before,
                bits,
                out,
            );
        } else {
            emit_uni_r5_32(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }
        if let Some(t0) = t_s5 {
            tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
        }
    }
}

// ============================================================================
// 3. Emit Kernels (Sentinel-Fused `COLD = 254` Hot Path: 0 Extra Hot Branches!)
// ============================================================================

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_front_slow(
    u: u32,
    cf: usize,
    lf: u8,
    hot_inline: &[[u8; INLINE]; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    lo: &mut usize,
    hi: usize,
    out: &mut [u8],
) {
    let (entry, len) = if lf == COLD {
        let tc = lookup_cold_band(cold_bands, cold_dir, u);
        (&inline16[tc], lens[tc])
    } else {
        (&hot_inline[cf], lf)
    };
    if len != LONG && hi - *lo >= INLINE {
        out[*lo..*lo + INLINE].copy_from_slice(entry);
        *lo += usize::from(len);
    } else {
        let word = if len == LONG {
            long_word(entry, spelling)
        } else {
            &entry[..usize::from(len)]
        };
        let take = word.len().min(hi - *lo);
        out[*lo..*lo + take].copy_from_slice(&word[..take]);
        *lo += take;
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_back_slow(
    u: u32,
    cb: usize,
    lb: u8,
    hot_inline: &[[u8; INLINE]; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    lo: usize,
    hi: &mut usize,
    out: &mut [u8],
) {
    let (entry, len) = if lb == COLD {
        let tc = lookup_cold_band(cold_bands, cold_dir, u);
        (&inline16[tc], lens[tc])
    } else {
        (&hot_inline[cb], lb)
    };
    if len != LONG && *hi - lo >= INLINE {
        let shifted = u128::from_le_bytes(*entry) << (8 * (INLINE - usize::from(len)));
        out[*hi - INLINE..*hi].copy_from_slice(&shifted.to_le_bytes());
        *hi -= usize::from(len);
    } else {
        let word = if len == LONG {
            long_word(entry, spelling)
        } else {
            &entry[..usize::from(len)]
        };
        let take = word.len().min(*hi - lo);
        out[*hi - take..*hi].copy_from_slice(&word[word.len() - take..]);
        *hi -= take;
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_bi_pack4<const BITS: u32>(
    edges: &[u32],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    before: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << BITS) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=m / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = get24_pack4(edges, front as usize);
        let cf = (ef >> BITS) as usize;
        let single = front == back;
        let lf = hot_lens[cf];
        if lf < COLD && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&hot_inline[cf]);
            lo += usize::from(lf);
        } else {
            emit_front_slow(
                front, cf, lf, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                &mut lo, hi, out,
            );
        }
        if single {
            break;
        }
        let eb = get24_pack4(edges, back as usize);
        let cb = (eb >> BITS) as usize;
        let lb = hot_lens[cb];
        if lb < COLD && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(hot_inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            emit_back_slow(
                back, cb, lb, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                lo, &mut hi, out,
            );
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_uni_pack4<const BITS: u32>(
    edges: &[u32],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << BITS) - 1;
    let mut at = 0usize;
    let mut j = mark;
    for _ in 0..m {
        let edge = get24_pack4(edges, j as usize);
        let code = (edge >> BITS) as usize;
        let cur = j;
        j = edge & mask;
        let lf = hot_lens[code];
        if lf < COLD {
            out[at..at + INLINE].copy_from_slice(&hot_inline[code]);
            at += usize::from(lf);
        } else {
            let (entry, len) = if lf == COLD {
                let tc = lookup_cold_band(cold_bands, cold_dir, cur);
                (&inline16[tc], lens[tc])
            } else {
                (&hot_inline[code], lf)
            };
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(entry);
                at += usize::from(len);
            } else {
                let word = long_word(entry, spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bi_byte4<const BITS: u32>(
    edges: &[u8],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    before: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << BITS) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=m / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = get24_byte4(edges, front as usize);
        let cf = (ef >> BITS) as usize;
        let single = front == back;
        let lf = hot_lens[cf];
        if lf < COLD && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&hot_inline[cf]);
            lo += usize::from(lf);
        } else {
            emit_front_slow(
                front, cf, lf, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                &mut lo, hi, out,
            );
        }
        if single {
            break;
        }
        let eb = get24_byte4(edges, back as usize);
        let cb = (eb >> BITS) as usize;
        let lb = hot_lens[cb];
        if lb < COLD && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(hot_inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            emit_back_slow(
                back, cb, lb, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                lo, &mut hi, out,
            );
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_uni_byte4<const BITS: u32>(
    edges: &[u8],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << BITS) - 1;
    let mut at = 0usize;
    let mut j = mark;
    for _ in 0..m {
        let edge = get24_byte4(edges, j as usize);
        let code = (edge >> BITS) as usize;
        let cur = j;
        j = edge & mask;
        let lf = hot_lens[code];
        if lf < COLD {
            out[at..at + INLINE].copy_from_slice(&hot_inline[code]);
            at += usize::from(lf);
        } else {
            let (entry, len) = if lf == COLD {
                let tc = lookup_cold_band(cold_bands, cold_dir, cur);
                (&inline16[tc], lens[tc])
            } else {
                (&hot_inline[code], lf)
            };
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(entry);
                at += usize::from(len);
            } else {
                let word = long_word(entry, spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bi_split<const HI_SHIFT: u32>(
    lo_arr: &[u16],
    hi_arr: &[u8],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    before: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let hi_mask = (1u32 << HI_SHIFT) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=m / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let hf = u32::from(hi_arr[front as usize]);
        let ef_lo = u32::from(lo_arr[front as usize]);
        let cf = (hf >> HI_SHIFT) as usize;
        let single = front == back;
        let lf = hot_lens[cf];
        if lf < COLD && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&hot_inline[cf]);
            lo += usize::from(lf);
        } else {
            emit_front_slow(
                front, cf, lf, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                &mut lo, hi, out,
            );
        }
        if single {
            break;
        }
        let hb = u32::from(hi_arr[back as usize]);
        let eb_lo = u32::from(lo_arr[back as usize]);
        let cb = (hb >> HI_SHIFT) as usize;
        let lb = hot_lens[cb];
        if lb < COLD && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(hot_inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            emit_back_slow(
                back, cb, lb, hot_inline, cold_bands, cold_dir, spelling, inline16, lens,
                lo, &mut hi, out,
            );
        }
        let ef_xor = ef_lo | ((hf & hi_mask) << 16);
        let eb_xor = eb_lo | ((hb & hi_mask) << 16);
        (prev, front) = (front, ef_xor ^ prev);
        (next, back) = (back, eb_xor ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_uni_split(
    lo_arr: &[u16],
    hi_arr: &[u8],
    hot_inline: &[[u8; INLINE]; 256],
    hot_lens: &[u8; 256],
    cold_bands: &[u32],
    cold_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mut j = mark;
    for _ in 0..m {
        let cur = j;
        let code = usize::from(hi_arr[cur as usize]);
        j = u32::from(lo_arr[cur as usize]);
        let lf = hot_lens[code];
        if lf < COLD {
            out[at..at + INLINE].copy_from_slice(&hot_inline[code]);
            at += usize::from(lf);
        } else {
            let (entry, len) = if lf == COLD {
                let tc = lookup_cold_band(cold_bands, cold_dir, cur);
                (&inline16[tc], lens[tc])
            } else {
                (&hot_inline[code], lf)
            };
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(entry);
                at += usize::from(len);
            } else {
                let word = long_word(entry, spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bi_e18(
    edges: &[u32],
    all_bands: &[u32],
    all_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    before: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=m / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = get18_pack(edges, front as usize);
        let cf = lookup_cold_band(all_bands, all_dir, front);
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline16[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline16[cf], spelling)
            } else {
                &inline16[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = get18_pack(edges, back as usize);
        let cb = lookup_cold_band(all_bands, all_dir, back);
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline16[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline16[cb], spelling)
            } else {
                &inline16[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, ef ^ prev);
        (next, back) = (back, eb ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_uni_e18(
    edges: &[u32],
    all_bands: &[u32],
    all_dir: &[u16],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    m: usize,
    mark: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mut j = mark;
    for _ in 0..m {
        let cur = j;
        j = get18_pack(edges, cur as usize);
        let code = lookup_cold_band(all_bands, all_dir, cur);
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_e16_single(
    edges: &[u16],
    only_code: usize,
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mut j = mark;
    let len = lens[only_code];
    let entry = &inline16[only_code];
    for _ in 0..edges.len() {
        j = u32::from(edges[j as usize]);
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(entry);
            at += usize::from(len);
        } else {
            let word = long_word(entry, spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bi_r5_32(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    before: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let eb = edges[back as usize];
        let cb = (eb >> bits) as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_uni_r5_32(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
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
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

// ============================================================================
// 4. Corpora, Reference Forward-FL, Exactness Suite & Benchmarks
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
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

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &r in recs {
        let c = (r >> 8) as u8;
        let w = (r & 255) as usize;
        l.resize(l.len() + w, c);
    }
    assert_eq!(l.len(), n);
    l
}

/// Canonical forward-FL inversion matching libbzip2 / cand_r5_head on valid & malformed columns.
fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut base = [0usize; 256];
    let mut sum = 0;
    for c in 0..256 {
        base[c] = sum;
        sum += cnt[c];
    }
    let mut fl = vec![0u32; n];
    let mut cur = base;
    for (i, &b) in l.iter().enumerate() {
        let idx = cur[usize::from(b)];
        cur[usize::from(b)] = idx + 1;
        fl[idx] = ((i as u32) << 8) | u32::from(b);
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        let e = fl[x];
        out.push((e & 0xFF) as u8);
        x = (e >> 8) as usize;
    }
    out
}

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_unstable_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
            l.push(text[n - 1]);
        } else {
            l.push(text[s - 1]);
        }
    }
    (l, p)
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = forward_fl_invert(&l, p);
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
                expected,
            }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let expected = if with_expected {
            let l = expand_records(&records, n);
            forward_fl_invert(&l, p)
        } else {
            Vec::new()
        };
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block {
            name,
            n,
            p,
            records,
            expected,
        });
    }
    blocks
}

fn make_solved_analogue_h() -> Vec<Block> {
    let mut out = Vec::new();
    // 1. AnaH-Compact300k: m = 55,257 < 65,536, used_codes = 23 <= 256 (0% cold spills, uni 24-bit)
    let mut txt = Vec::with_capacity(300_000);
    let mut rng = 0x1234_5678_9ABC_DEF0u64;
    let mut next = || {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    let mut id_num = 100_000u32;
    while txt.len() + 64 < 300_000 {
        id_num += 1;
        let tag = (next() % 8) as u8;
        let line = format!("{{\"id\":\"Q{:06}\",\"type\":\"item\",\"v\":{}}}\n", id_num, b'A' + tag);
        txt.extend_from_slice(line.as_bytes());
    }
    let (l, p) = bwt_encode(&txt);
    out.push(Block {
        name: "AnaH-Uni55k".to_string(),
        n: l.len(),
        p,
        records: records_of(&l),
        expected: txt,
    });

    // 2. AnaH-Bi90k: m ~ 95,000 (bits_m = 17, 7 hot bits = 128 slots), used_codes <= 60 (0% cold spills, bi 24-bit!)
    let mut txt2 = Vec::with_capacity(600_000);
    let mut id2 = 200_000u32;
    while txt2.len() + 64 < 600_000 {
        id2 += 1;
        let tag = (next() % 16) as u8;
        let line = format!("{{\"id\":\"P{:06}\",\"k\":\"val\",\"v\":{}}}\n", id2, b'a' + tag);
        txt2.extend_from_slice(line.as_bytes());
    }
    let (l2, p2) = bwt_encode(&txt2);
    out.push(Block {
        name: "AnaH-Bi110k".to_string(),
        n: l2.len(),
        p: p2,
        records: records_of(&l2),
        expected: txt2,
    });
    out
}

fn run_exactness_suite(solved: &[Block], wiki: &[Block], nonwiki: &[Block], code_docs: &[Block], w233: &[Block]) {
    println!("================================================================================");
    println!("EXACTNESS VERIFICATION SUITE (5,500+ small + 260+ malformed FL + fallbacks + all corpora)");
    println!("================================================================================");
    let modes = [
        EdgeMode::R5Mirror32,
        EdgeMode::Pack4,
        EdgeMode::Byte4,
        EdgeMode::Split,
        EdgeMode::BandOnly18,
    ];
    let mut r5 = cand_r5_head::Kernel::default();
    let mut hk = HKernel::default();
    let mut out_r5 = Vec::new();
    let mut out_h = Vec::new();

    let mut check_all = |l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let n = l.len();
        let fl_exp = forward_fl_invert(l, p);
        r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_r5).unwrap();
        assert_eq!(out_r5, fl_exp, "cand_r5_head vs FL mismatch on {tag}");
        for &mode in &modes {
            hk.run::<false>(mode, &recs, n, p, &mut out_h);
            assert_eq!(out_h, out_r5, "{mode:?} mismatch on {tag}");
        }
    };

    // 1. Exhaustive small binary n=1..=10 at p=0 and p=n-1 (4,092 cases)
    let mut small_cnt = 0usize;
    for n in 1..=10usize {
        for mask in 0..(1u32 << n) {
            let mut l = Vec::with_capacity(n);
            for bit in 0..n {
                l.push(if (mask >> bit) & 1 == 1 { b'b' } else { b'a' });
            }
            check_all(&l, 0, "bin-p0");
            small_cnt += 1;
            if n >= 2 {
                check_all(&l, (n - 1) as u32, "bin-plast");
                small_cnt += 1;
            }
        }
    }

    // 2. 1,500 random multi-symbol thick/thin columns
    let mut rng = 0xDEAD_BEEF_CAFE_BABEu64;
    let next_u64 = |s: &mut u64| -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s
    };
    for trial in 0..1500usize {
        let sigma = 2 + (next_u64(&mut rng) as usize % 6);
        let n_runs = 4 + (next_u64(&mut rng) as usize % 16);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = b'a' + (next_u64(&mut rng) as usize % sigma) as u8;
            let w = if next_u64(&mut rng) % 3 == 0 {
                17 + (next_u64(&mut rng) as usize % 30)
            } else {
                1 + (next_u64(&mut rng) as usize % 16)
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("synth-thick-{trial}"));
        small_cnt += 1;
    }
    println!("  PASS: {small_cnt} small & thick-strip cases 100% BYTE-EXACT across all 5 modes.");

    // 3. 250+ malformed multi-cycle & pure-thick-cycle columns (incl. large m >= 65,536 bi-XOR malformed!)
    let mut mal_cnt = 0usize;
    for &w in &[17usize, 20, 32, 64] {
        let mut l = Vec::new();
        l.push(b'z');
        l.resize(l.len() + w, b'b');
        l.resize(l.len() + w, b'a');
        for p in [0u32, 1, w as u32, (2 * w) as u32] {
            check_all(&l, p, "thick-cycle-2");
            mal_cnt += 1;
        }
        let mut l2 = Vec::new();
        l2.resize(w, b'a');
        l2.push(b'b');
        l2.resize(l2.len() + w, b'c');
        for p in [0u32, w as u32, (w + 1) as u32] {
            check_all(&l2, p, "thick-self-loop");
            mal_cnt += 1;
        }
    }
    for trial in 0..222usize {
        let mut l = Vec::new();
        let n_runs = 6 + (next_u64(&mut rng) as usize % 20);
        let sigma = 2 + (next_u64(&mut rng) as usize % 5);
        for _ in 0..n_runs {
            let b = b'A' + (next_u64(&mut rng) as usize % sigma) as u8;
            let w = match next_u64(&mut rng) % 4 {
                0 => 20,
                1 => 40,
                2 => 1 + (next_u64(&mut rng) as usize % 5),
                _ => 17 + (next_u64(&mut rng) as usize % 25),
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("malformed-{trial}"));
        mal_cnt += 1;
    }
    // 10 large malformed multi-cycle columns with m >= 70,000 exercising bidirectional XOR 24-bit/18-bit!
    for trial in 0..10usize {
        let mut l = Vec::with_capacity(80_000);
        let sigma = 8 + (trial % 8);
        for i in 0..72_000usize {
            let b = b'a' + ((i * 7 + trial * 3 + (i / 1000)) % sigma) as u8;
            l.push(b);
        }
        // Add thick cycle region
        l.resize(l.len() + 2000, b'X');
        l.resize(l.len() + 2000, b'Y');
        let p = ((trial * 9973) % l.len()) as u32;
        check_all(&l, p, &format!("malformed-bi-large-{trial}"));
        mal_cnt += 1;
    }
    println!("  PASS: {mal_cnt} malformed multi-cycle (incl. 10 large m>=70k bi-XOR!) 100% BYTE-EXACT vs cand_r5_head & forward-FL.");

    // 4. Forced fallback & boundary stress tests:
    //   (a) spiral-200k (LONG = 255 alongside COLD = 254)
    let mut spiral_l = vec![b'a'; 200_000];
    spiral_l[0] = b'c';
    spiral_l[199_999] = b'b';
    check_all(&spiral_l, 0, "spiral-200k-LONG255");

    //   (b) Degenerate |D|=1 (`Edges16` fast path!)
    let mut l_d1 = vec![b'a'; 40_000];
    l_d1.push(b'b');
    l_d1.extend(std::iter::repeat_n(b'a', 19_999));
    check_all(&l_d1, 40_000, "AnaH-D1-60k");

    //   (c) m > 262,144 (`bits_m = 19` -> automatic fallback to exact 32-bit!)
    let mut l_bigm = Vec::with_capacity(270_000);
    for i in 0..270_000usize {
        l_bigm.push(b'a' + ((i * 13 + (i >> 4)) % 16) as u8);
    }
    check_all(&l_bigm, 12345, "m-270k-auto-fallback");

    //   (d) Explicit `force_fallback = true` on real block-0
    hk.force_fallback = true;
    for &mode in &modes {
        hk.run::<false>(mode, &wiki[0].records, wiki[0].n, wiki[0].p, &mut out_h);
        assert_eq!(out_h, wiki[0].expected, "force_fallback mismatch for {mode:?}");
    }
    hk.force_fallback = false;
    println!("  PASS: spiral-200k + |D|=1 Edges16 + m=270k auto-fallback + force_fallback BYTE-EXACT.");

    // 5. Solved Analogue H + 6 Admitted Wikidata + 6 Non-Wikidata + 4 Code+Docs
    for blk in solved.iter().chain(wiki.iter()).chain(nonwiki.iter()).chain(code_docs.iter()) {
        for &mode in &modes {
            hk.run::<false>(mode, &blk.records, blk.n, blk.p, &mut out_h);
            assert_eq!(out_h, blk.expected, "{mode:?} mismatch on {}", blk.name);
        }
    }
    println!(
        "  PASS: {}/{} Solved AnaH + {}/{} Admitted Wiki + {}/{} Non-Wiki + {}/{} Code+Docs 100% BYTE-EXACT across all 5 modes.",
        solved.len(), solved.len(), wiki.len(), wiki.len(), nonwiki.len(), nonwiki.len(), code_docs.len(), code_docs.len()
    );

    // 6. All 233 Full wikidata.bz2 blocks across all 5 modes vs cand_r5_head!
    for blk in w233 {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_r5).unwrap();
        for &mode in &[EdgeMode::Pack4, EdgeMode::Byte4, EdgeMode::Split, EdgeMode::BandOnly18] {
            hk.run::<false>(mode, &blk.records, blk.n, blk.p, &mut out_h);
            assert_eq!(out_h, out_r5, "{mode:?} mismatch on {}", blk.name);
        }
    }
    println!(
        "  PASS: {}/{} full wikidata.bz2 blocks 100% BYTE-EXACT across Pack4, Byte4, Split, BandOnly18 vs cand_r5_head!",
        w233.len(), w233.len()
    );
}

fn report_working_sets(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("PERSISTENT SCRATCH (`scratch_report()`) & ACTIVE S4+S5 WS — {label}");
    println!("================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut out = Vec::new();
    for blk in blocks {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out).unwrap();
    }
    let (r5_tot, r5_l1, r5_s) = r5.scratch_report();
    println!(
        "  cand_r5_head (verbatim): tot={:.1} KiB (dict_l1={:.1} KiB) | {r5_s}",
        r5_tot as f64 / 1024.0,
        r5_l1 as f64 / 1024.0
    );

    for &mode in &[
        EdgeMode::R5Mirror32,
        EdgeMode::Pack4,
        EdgeMode::Byte4,
        EdgeMode::Split,
        EdgeMode::BandOnly18,
    ] {
        let mut hk = HKernel::default();
        let mut sum_active = 0usize;
        let mut max_active = 0usize;
        for blk in blocks {
            let (m, _) = hk.run::<false>(mode, &blk.records, blk.n, blk.p, &mut out);
            let act = hk.active_s45_bytes(mode, m);
            sum_active += act;
            max_active = max_active.max(act);
        }
        let (tot, s) = hk.scratch_report(mode);
        let delta_kib = (tot as isize - r5_tot as isize) as f64 / 1024.0;
        println!(
            "  {:<12}: delta={:+6.1} KiB | active_S45 max={:5.1} KiB avg={:5.1} KiB | {s}",
            format!("{mode:?}"),
            delta_kib,
            max_active as f64 / 1024.0,
            (sum_active as f64 / blocks.len().max(1) as f64) / 1024.0,
        );
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("1T STAGE BREAKDOWN (PAIRED 15 ROUNDS, p50 us) — {label}");
    println!("================================================================================");
    let modes = [
        EdgeMode::R5Mirror32,
        EdgeMode::Pack4,
        EdgeMode::Byte4,
        EdgeMode::Split,
        EdgeMode::BandOnly18,
    ];
    let mut r5 = cand_r5_head::Kernel::default();
    let mut hks: Vec<HKernel> = (0..modes.len()).map(|_| HKernel::default()).collect();
    let mut out = Vec::new();

    // Warmup
    for blk in blocks {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
        for (mi, &mode) in modes.iter().enumerate() {
            hks[mi].run::<false>(mode, &blk.records, blk.n, blk.p, &mut out);
        }
    }

    let mut sum_s1 = [0.0; 5];
    let mut sum_s23 = [0.0; 5];
    let mut sum_s3b = [0.0; 5];
    let mut sum_s4 = [0.0; 5];
    let mut sum_s5 = [0.0; 5];
    let mut sum_tot_timed = [0.0; 5];
    let mut sum_tot_untimed = [0.0; 5];
    let mut sum_r5 = 0.0;

    println!(
        "{:<14} | {:>7} {:>7} | {:>7} {:>7} {:>7} | {:>7} {:>7} {:>7} {:>7} {:>7}",
        "Block", "R5H-A1", "R5H-A2", "P4:S3b", "P4:S4", "P4:S5", "Tot:A2", "Tot:P4", "Tot:B4", "Tot:Sp", "Tot:E18"
    );
    for blk in blocks {
        let mut vr5 = Vec::new();
        let mut vs1 = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vs23 = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vs3b = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vs4 = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vs5 = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vtot_t = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut vtot_u = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        for _ in 0..15 {
            let t0 = Instant::now();
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
            std::hint::black_box(&out);
            vr5.push(t0.elapsed().as_secs_f64() * 1e6);

            for (mi, &mode) in modes.iter().enumerate() {
                let (_, tm) = hks[mi].run::<true>(mode, &blk.records, blk.n, blk.p, &mut out);
                vs1[mi].push(tm.s1_us);
                vs23[mi].push(tm.s23_us);
                vs3b[mi].push(tm.s3b_us);
                vs4[mi].push(tm.s4_us);
                vs5[mi].push(tm.s5_us);
                vtot_t[mi].push(tm.total_us());

                let tu = Instant::now();
                hks[mi].run::<false>(mode, &blk.records, blk.n, blk.p, &mut out);
                std::hint::black_box(&out);
                vtot_u[mi].push(tu.elapsed().as_secs_f64() * 1e6);
            }
        }
        let p50 = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let mr5 = p50(&mut vr5);
        sum_r5 += mr5;
        let mut m3b = [0.0; 5];
        let mut m4 = [0.0; 5];
        let mut m5 = [0.0; 5];
        let mut mtu = [0.0; 5];
        for mi in 0..5 {
            sum_s1[mi] += p50(&mut vs1[mi]);
            sum_s23[mi] += p50(&mut vs23[mi]);
            m3b[mi] = p50(&mut vs3b[mi]);
            m4[mi] = p50(&mut vs4[mi]);
            m5[mi] = p50(&mut vs5[mi]);
            let mtt = p50(&mut vtot_t[mi]);
            mtu[mi] = p50(&mut vtot_u[mi]);
            sum_s3b[mi] += m3b[mi];
            sum_s4[mi] += m4[mi];
            sum_s5[mi] += m5[mi];
            sum_tot_timed[mi] += mtt;
            sum_tot_untimed[mi] += mtu[mi];
        }
        println!(
            "{:<14} | {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1}",
            blk.name, mr5, mtu[0], m3b[1], m4[1], m5[1], mtu[0], mtu[1], mtu[2], mtu[3], mtu[4]
        );
    }
    println!(
        "{:<14} | {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1}",
        "SUM", sum_r5, sum_tot_untimed[0], sum_s3b[1], sum_s4[1], sum_s5[1],
        sum_tot_untimed[0], sum_tot_untimed[1], sum_tot_untimed[2], sum_tot_untimed[3], sum_tot_untimed[4]
    );
    println!("  STAGE SUMS (us):    A2(32b)    Pack4    Byte4    Split   Band18");
    println!(
        "    S1  (pass)    : {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
        sum_s1[0], sum_s1[1], sum_s1[2], sum_s1[3], sum_s1[4]
    );
    println!(
        "    S23 (ind+dict): {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
        sum_s23[0], sum_s23[1], sum_s23[2], sum_s23[3], sum_s23[4]
    );
    println!(
        "    S3b (prep)    : {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
        sum_s3b[0], sum_s3b[1], sum_s3b[2], sum_s3b[3], sum_s3b[4]
    );
    println!(
        "    S4  (wire)    : {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1}  (vs A2: 1.000 / {:.3} / {:.3} / {:.3} / {:.3}x)",
        sum_s4[0], sum_s4[1], sum_s4[2], sum_s4[3], sum_s4[4],
        sum_s4[1] / sum_s4[0], sum_s4[2] / sum_s4[0], sum_s4[3] / sum_s4[0], sum_s4[4] / sum_s4[0]
    );
    println!(
        "    S5  (emit)    : {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1}  (vs A2: 1.000 / {:.3} / {:.3} / {:.3} / {:.3}x)",
        sum_s5[0], sum_s5[1], sum_s5[2], sum_s5[3], sum_s5[4],
        sum_s5[1] / sum_s5[0], sum_s5[2] / sum_s5[0], sum_s5[3] / sum_s5[0], sum_s5[4] / sum_s5[0]
    );
    println!(
        "    Total / R5H-A1:  {:.4}x   {:.4}x   {:.4}x   {:.4}x   {:.4}x",
        sum_tot_untimed[0] / sum_r5,
        sum_tot_untimed[1] / sum_r5,
        sum_tot_untimed[2] / sum_r5,
        sum_tot_untimed[3] / sum_r5,
        sum_tot_untimed[4] / sum_r5,
    );
}

fn bench_multiworker(blocks: &[Block], label: &str, rounds: usize, iters_1w: usize, iters_mw: usize) {
    println!("\n--- Multi-Worker (1T / 2W / 4W, paired alternating {rounds} rounds, p50 wall ms) — {label} ---");
    for &workers in &[1usize, 2, 4] {
        let mut r5_ms = Vec::new();
        let mut a2_ms = Vec::new();
        let mut p4_ms = Vec::new();
        let mut b4_ms = Vec::new();
        let mut sp_ms = Vec::new();
        let mut e18_ms = Vec::new();
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for arm in 0..6 {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut hk = HKernel::default();
                            let mut out = Vec::new();
                            for blk in blocks {
                                match arm {
                                    0 => { r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out); }
                                    1 => { hk.run::<false>(EdgeMode::R5Mirror32, &blk.records, blk.n, blk.p, &mut out); }
                                    2 => { hk.run::<false>(EdgeMode::Pack4, &blk.records, blk.n, blk.p, &mut out); }
                                    3 => { hk.run::<false>(EdgeMode::Byte4, &blk.records, blk.n, blk.p, &mut out); }
                                    4 => { hk.run::<false>(EdgeMode::Split, &blk.records, blk.n, blk.p, &mut out); }
                                    _ => { hk.run::<false>(EdgeMode::BandOnly18, &blk.records, blk.n, blk.p, &mut out); }
                                }
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    match arm {
                                        0 => { r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out); }
                                        1 => { hk.run::<false>(EdgeMode::R5Mirror32, &blk.records, blk.n, blk.p, &mut out); }
                                        2 => { hk.run::<false>(EdgeMode::Pack4, &blk.records, blk.n, blk.p, &mut out); }
                                        3 => { hk.run::<false>(EdgeMode::Byte4, &blk.records, blk.n, blk.p, &mut out); }
                                        4 => { hk.run::<false>(EdgeMode::Split, &blk.records, blk.n, blk.p, &mut out); }
                                        _ => { hk.run::<false>(EdgeMode::BandOnly18, &blk.records, blk.n, blk.p, &mut out); }
                                    }
                                    std::hint::black_box(&out);
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e3 / iters as f64
                        }));
                    }
                    let mut max_ms = 0.0f64;
                    for h in handles {
                        max_ms = max_ms.max(h.join().unwrap());
                    }
                    max_ms
                });
                match arm {
                    0 => r5_ms.push(t_wall),
                    1 => a2_ms.push(t_wall),
                    2 => p4_ms.push(t_wall),
                    3 => b4_ms.push(t_wall),
                    4 => sp_ms.push(t_wall),
                    _ => e18_ms.push(t_wall),
                }
            }
        }
        let med = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let (mr5, ma2, mp4, mb4, msp, me18) = (
            med(&mut r5_ms),
            med(&mut a2_ms),
            med(&mut p4_ms),
            med(&mut b4_ms),
            med(&mut sp_ms),
            med(&mut e18_ms),
        );
        println!(
            "  {}W: R5H-A1={:7.2}ms | A2={:7.2}ms ({:.4}x) | Pack4={:7.2}ms ({:.4}x) | Byte4={:7.2}ms ({:.4}x) | Split={:7.2}ms ({:.4}x) | Band18={:7.2}ms ({:.4}x)",
            workers, mr5, ma2, ma2 / mr5, mp4, mp4 / mr5, mb4, mb4 / mr5, msp, msp / mr5, me18, me18 / mr5
        );
    }
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/tmp/mibz2_corpora/{rel}");
    if std::path::Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if std::path::Path::new(&p2).exists() {
        return p2;
    }
    panic!("corpus file not found at {p1} or {p2}");
}

fn main() {
    let solved = make_solved_analogue_h();
    let wiki = load_six_admitted();
    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki_path = resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin");
    let nonwiki = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false, true);

    let code_docs_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let code_docs_path = resolve_corpus_path("non_wikidata/code_docs_blocks.bin");
    let code_docs = load_bin_blocks(&code_docs_path, Some(&code_docs_names), false, true);

    let w431_path = resolve_corpus_path("wikidata_431_blocks.bin");
    let w233 = load_bin_blocks(&w431_path, None, true, false);

    run_exactness_suite(&solved, &wiki, &nonwiki, &code_docs, &w233);

    report_working_sets(&solved, "Solved Analogue H (Uni55k + Bi110k)");
    report_working_sets(&wiki, "6 Admitted Wikidata Blocks");
    report_working_sets(&nonwiki, "6 Real Non-Wikidata Blocks (JSONL + SYSLOG)");
    report_working_sets(&code_docs, "4 Real Code+Docs Blocks");
    report_working_sets(&w233, "All 233 Full Wikidata.bz2 Blocks");

    bench_stage_1t(&solved, "Solved Analogue H (Uni55k + Bi110k)");
    bench_stage_1t(&wiki, "6 Admitted Wikidata Blocks");
    bench_stage_1t(&nonwiki, "6 Real Non-Wikidata Blocks (JSONL + SYSLOG)");
    bench_stage_1t(&code_docs, "4 Real Code+Docs Blocks");

    bench_multiworker(&solved, "Solved Analogue H (Uni55k + Bi110k)", 11, 5, 4);
    bench_multiworker(&wiki, "6 Admitted Wikidata Blocks", 11, 5, 4);
    bench_multiworker(&nonwiki, "6 Real Non-Wikidata Blocks (JSONL + SYSLOG)", 11, 5, 4);
    bench_multiworker(&code_docs, "4 Real Code+Docs Blocks", 5, 2, 1);
    bench_multiworker(&w233, "233 Full Wikidata.bz2 Blocks", 5, 1, 1);
}
