#![forbid(unsafe_code)]

//! Cycle 48 P2 — Analogue I: Run-Stream / Zero-Spill Streaming Queue Monoid (`FQ4` & `Obligation 5`)
//! Concrete `OutQ` & `S1` memory-traffic substitutes benchmarked directly over verbatim
//! `cand_r5_head/endpoint.rs`, with verbatim `cand_r5_head::Kernel` A/A ground truth:
//!   (a) OutQ-ChunkSweep: CHUNK in {64, 128, 256, 512} (128B / 256B / 512B / 1024B per chunk)
//!   (b) OutQ-RunPair:    Compact thin-run queues:
//!         - RunPair-E16K (2-bit width tag + 14-bit rank: w<=3 -> 2B, w=4..16 -> 4B; CHUNK=128 & 512)
//!         - RunPair-E4K  (4-bit width + 12-bit rank: ALL w=1..16 -> 2B/run; CHUNK=128)
//!         - RunPair-E32K (1-bit tag + 15-bit rank: w=1 -> 2B, w>=2 -> 4B; CHUNK=128)
//!   (c) Uncapped-S1:
//!         - Uncapped-U32   (&[u32] `(byte << 24) | len`, zero CAP=127 splits)
//!         - Uncapped-Esc16 (&[u16] bit-7 escape for len>=128, zero while-loop, smaller than CAP=127)

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 1. Parameterized OutQChunk<CHUNK, BYTES> (Exact cand_r5_head generalization)
// ============================================================================
mod outq_chunk {
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    pub struct Spill<const BYTES: usize> {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
        pub last_chunks: u32,
        pub last_ovf: usize,
        pub last_cuts: usize,
    }

    impl<const BYTES: usize> Default for Spill<BYTES> {
        fn default() -> Self {
            Self {
                next: Vec::new(),
                ovf: Vec::new(),
                cuts: Vec::new(),
                last_chunks: 0,
                last_ovf: 0,
                last_cuts: 0,
            }
        }
    }

    impl<const BYTES: usize> Spill<BYTES> {
        pub fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    pub struct Queues<'a, const CHUNK: usize, const BYTES: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill<BYTES>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        pt: [u64; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const CHUNK: usize, const BYTES: usize> Queues<'a, CHUNK, BYTES> {
        const SHIFT: u32 = CHUNK.trailing_zeros() + 1;
        const MASK: u32 = (BYTES - 1) as u32;

        pub fn new(out: &'a mut [u8], spill: &'a mut Spill<BYTES>, limit: u32) -> Self {
            debug_assert_eq!(BYTES, 2 * CHUNK);
            debug_assert!(CHUNK.is_power_of_two() && CHUNK >= 32);
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Self {
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

        #[inline(always)]
        pub fn thin_of(&self, a: usize) -> u32 {
            (self.pt[a] >> 32) as u32
        }

        pub fn extract_thin(&self) -> [u32; 256] {
            std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
        }

        pub fn finish_stats(&mut self) {
            self.spill.last_chunks = self.chunks;
            self.spill.last_ovf = self.spill.ovf.len();
            self.spill.last_cuts = self.spill.cuts.len();
        }

        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }

        #[inline(always)]
        pub fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & Self::MASK) as usize;
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
                if p == u32::MAX || (p & Self::MASK) as usize == CHUNK {
                    self.take_chunk_pt(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & Self::MASK) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(p >> Self::SHIFT);
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
                self.spill.next[((self.pt[a] as u32) >> Self::SHIFT) as usize] = id;
            }
            self.pt[a] = hi | u64::from(id << Self::SHIFT);
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

    pub struct Reader<'a, const CHUNK: usize, const BYTES: usize> {
        out: &'a [u8],
        spill: &'a Spill<BYTES>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a, const CHUNK: usize, const BYTES: usize> Reader<'a, CHUNK, BYTES> {
        pub fn new(out: &'a [u8], spill: &'a Spill<BYTES>, heads: &Heads, a: usize) -> Self {
            Self {
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

        #[inline(always)]
        pub fn wire_xor_nomark(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
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
                if i + 8 <= span {
                    let b = &bytes[2 * i..2 * i + 16];
                    let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                    let v0 = hi | (q0 as u32 & 0xFFFF);
                    let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q0 >> 48) as u32);
                    let v4 = hi | (q1 as u32 & 0xFFFF);
                    let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                    let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                    let v7 = hi | ((q1 >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 8];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                    edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                    i += 8;
                }
                if i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                    let v0 = hi | (q as u32 & 0xFFFF);
                    let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                    let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                    let v3 = hi | ((q >> 48) as u32);
                    let u0 = dst_start + i;
                    let eu = &mut edges[u0..u0 + 4];
                    eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
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
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
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
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)]
                    .chunks_exact(2)
                    .enumerate()
                {
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
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * BYTES + 2 * self.at;
                for (e, s) in dst
                    .iter_mut()
                    .zip(self.out[at..at + 2 * span].chunks_exact(2))
                {
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
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = dst
                    .len()
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                for (e, s) in now
                    .iter_mut()
                    .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = rest;
            }
        }
    }
}

// ============================================================================
// 2. Compact Thin-Run Queue `OutQRunPair<MODE, CHUNK, BYTES>`
//    MODE = 0: E16K (EPOCH=16384; w in 1..=3 -> 1 u16, w in 4..=16 -> 2 u16)
//    MODE = 1: E4K  (EPOCH=4096;  ALL w in 1..=16 -> 1 u16 `((w-1)<<12)|lo`)
//    MODE = 2: E32K (EPOCH=32768; w==1 -> 1 u16, w in 2..=16 -> 2 u16)
//    Invariant: no 2-u16 token ever straddles a chunk or epoch boundary!
// ============================================================================
mod outq_runpair {
    const NONE: u32 = u32::MAX;

    pub const MODE_E16K: u8 = 0;
    pub const MODE_E4K: u8 = 1;
    pub const MODE_E32K: u8 = 2;

    const fn epoch_shift(mode: u8) -> u32 {
        match mode {
            MODE_E16K => 14,
            MODE_E4K => 12,
            _ => 15,
        }
    }

    const fn epoch_size(mode: u8) -> u32 {
        1u32 << epoch_shift(mode)
    }

    const fn epoch_mask(mode: u8) -> u32 {
        epoch_size(mode) - 1
    }

    pub struct Spill<const BYTES: usize> {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
        pub last_chunks: u32,
        pub last_words: usize,
        pub last_ovf: usize,
        pub last_cuts: usize,
    }

    impl<const BYTES: usize> Default for Spill<BYTES> {
        fn default() -> Self {
            Self {
                next: Vec::new(),
                ovf: Vec::new(),
                cuts: Vec::new(),
                last_chunks: 0,
                last_words: 0,
                last_ovf: 0,
                last_cuts: 0,
            }
        }
    }

    impl<const BYTES: usize> Spill<BYTES> {
        pub fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    pub struct Queues<'a, const MODE: u8, const CHUNK: usize, const BYTES: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill<BYTES>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        /// Packed `(qwords[a] << 32) | (tail << SHIFT) | fill_words`
        pt: [u64; 256],
        thin_rows: [u32; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const MODE: u8, const CHUNK: usize, const BYTES: usize> Queues<'a, MODE, CHUNK, BYTES> {
        const SHIFT: u32 = CHUNK.trailing_zeros() + 1;
        const MASK: u32 = (BYTES - 1) as u32;
        const EPOCH: u32 = epoch_size(MODE);
        const EMASK: u32 = epoch_mask(MODE);

        pub fn new(out: &'a mut [u8], spill: &'a mut Spill<BYTES>, limit: u32) -> Self {
            debug_assert_eq!(BYTES, 2 * CHUNK);
            spill.ovf.clear();
            spill.cuts.clear();
            let in_out = (out.len() / BYTES) as u32;
            let out = &mut out[..in_out as usize * BYTES];
            Self {
                in_out,
                out,
                spill,
                chunks: 0,
                heads: [NONE; 256],
                pt: [u32::MAX as u64; 256],
                thin_rows: [0; 256],
                limit,
                over: false,
            }
        }

        #[inline(always)]
        pub fn thin_of(&self, a: usize) -> u32 {
            self.thin_rows[a]
        }

        pub fn extract_thin(&self) -> [u32; 256] {
            self.thin_rows
        }

        pub fn finish_stats(&mut self) {
            self.spill.last_chunks = self.chunks;
            self.spill.last_words = self.pt.iter().map(|&v| (v >> 32) as usize).sum();
            self.spill.last_ovf = self.spill.ovf.len();
            self.spill.last_cuts = self.spill.cuts.len();
        }

        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }

        #[inline(always)]
        pub fn push_thin(&mut self, a: usize, m: u32, w: u32) {
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & Self::MASK) as usize;
            let lo = m & Self::EMASK;
            if MODE == MODE_E4K {
                if (w <= 16) & (f < CHUNK) & (lo < Self::EPOCH - 16) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 2) {
                        let tok = (((w - 1) << 12) | lo) as u16;
                        dst.copy_from_slice(&tok.to_le_bytes());
                        self.pt[a] = v + ((1u64 << 32) | 1u64);
                        self.thin_rows[a] += w;
                        return;
                    }
                }
            } else if MODE == MODE_E16K {
                if (w <= 16) & (f <= CHUNK - 2) & (lo < Self::EPOCH - 16) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 4) {
                        if w <= 3 {
                            let tok = (((w - 1) << 14) | lo) as u16;
                            dst[0..2].copy_from_slice(&tok.to_le_bytes());
                            self.pt[a] = v + ((1u64 << 32) | 1u64);
                        } else {
                            let t0 = (0xC000 | lo) as u16;
                            let t1 = w as u16;
                            dst[0..2].copy_from_slice(&t0.to_le_bytes());
                            dst[2..4].copy_from_slice(&t1.to_le_bytes());
                            self.pt[a] = v + ((2u64 << 32) | 2u64);
                        }
                        self.thin_rows[a] += w;
                        return;
                    }
                }
            } else {
                // MODE_E32K
                if (w <= 16) & (f <= CHUNK - 2) & (lo < Self::EPOCH - 16) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 4) {
                        if w == 1 {
                            let tok = lo as u16;
                            dst[0..2].copy_from_slice(&tok.to_le_bytes());
                            self.pt[a] = v + ((1u64 << 32) | 1u64);
                        } else {
                            let t0 = (0x8000 | lo) as u16;
                            let t1 = w as u16;
                            dst[0..2].copy_from_slice(&t0.to_le_bytes());
                            dst[2..4].copy_from_slice(&t1.to_le_bytes());
                            self.pt[a] = v + ((2u64 << 32) | 2u64);
                        }
                        self.thin_rows[a] += w;
                        return;
                    }
                }
            }
            self.push_slow(a, m, w);
        }

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, mut m: u32, w: u32) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let p = self.pt[a] as u32;
                if p == u32::MAX || (p & Self::MASK) as usize == CHUNK {
                    self.take_chunk(a);
                }
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & Self::MASK) as usize;
                let slots_left = CHUNK - f;
                let lo = m & Self::EMASK;
                let mut take = left.min(16).min(Self::EPOCH - lo);

                // Guarantee no 2-u16 token ever straddles a chunk boundary:
                if MODE == MODE_E16K && slots_left == 1 && take >= 4 {
                    take = 3;
                } else if MODE == MODE_E32K && slots_left == 1 && take >= 2 {
                    take = 1;
                }

                let chunk = self.chunk_mut(p >> Self::SHIFT);
                let words_written: u64 = if MODE == MODE_E4K {
                    let tok = (((take - 1) << 12) | lo) as u16;
                    chunk[2 * f..2 * f + 2].copy_from_slice(&tok.to_le_bytes());
                    1
                } else if MODE == MODE_E16K {
                    if take <= 3 {
                        let tok = (((take - 1) << 14) | lo) as u16;
                        chunk[2 * f..2 * f + 2].copy_from_slice(&tok.to_le_bytes());
                        1
                    } else {
                        let t0 = (0xC000 | lo) as u16;
                        let t1 = take as u16;
                        chunk[2 * f..2 * f + 2].copy_from_slice(&t0.to_le_bytes());
                        chunk[2 * f + 2..2 * f + 4].copy_from_slice(&t1.to_le_bytes());
                        2
                    }
                } else if take == 1 {
                    let tok = lo as u16;
                    chunk[2 * f..2 * f + 2].copy_from_slice(&tok.to_le_bytes());
                    1
                } else {
                    let t0 = (0x8000 | lo) as u16;
                    let t1 = take as u16;
                    chunk[2 * f..2 * f + 2].copy_from_slice(&t0.to_le_bytes());
                    chunk[2 * f + 2..2 * f + 4].copy_from_slice(&t1.to_le_bytes());
                    2
                };

                self.pt[a] = v + ((words_written << 32) | words_written);
                self.thin_rows[a] += take;
                m += take;
                left -= take;
                if m % Self::EPOCH == 0 {
                    let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk(&mut self, a: usize) {
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
                self.spill.next[((self.pt[a] as u32) >> Self::SHIFT) as usize] = id;
            }
            self.pt[a] = hi | u64::from(id << Self::SHIFT);
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

    pub struct Reader<'a, const MODE: u8, const CHUNK: usize, const BYTES: usize> {
        out: &'a [u8],
        spill: &'a Spill<BYTES>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read_words: u32,
        epoch: u32,
        cut: u32,
        rem_v: u32,
        rem_w: usize,
    }

    impl<'a, const MODE: u8, const CHUNK: usize, const BYTES: usize> Reader<'a, MODE, CHUNK, BYTES> {
        const ESHIFT: u32 = epoch_shift(MODE);

        pub fn new(out: &'a [u8], spill: &'a Spill<BYTES>, heads: &Heads, a: usize) -> Self {
            Self {
                out,
                spill,
                in_out: heads.in_out,
                a,
                id: heads.heads[a],
                at: 0,
                read_words: 0,
                epoch: 0,
                cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
                rem_v: 0,
                rem_w: 0,
            }
        }

        #[inline(always)]
        pub fn wire_xor_nomark(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
        ) {
            if self.rem_w > 0 {
                let take = self.rem_w.min(left);
                let v0 = self.rem_v as usize;
                for k in 0..take {
                    let u = dst_start + k;
                    let v = v0 + k;
                    edges[u] ^= pack | (v as u32);
                    edges[v] ^= u as u32;
                }
                self.rem_v += take as u32;
                self.rem_w -= take;
                dst_start += take;
                left -= take;
                if left == 0 {
                    return;
                }
            }
            while left > 0 {
                while self.read_words == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let avail = (CHUNK - self.at).min((self.cut - self.read_words) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << Self::ESHIFT;
                let mut pos = self.at;
                let end_pos = self.at + avail;

                while left > 0 && pos < end_pos {
                    let tok = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                    pos += 1;
                    if MODE == MODE_E16K {
                        if tok < 0x4000 {
                            let v = hi | u32::from(tok);
                            edges[dst_start] ^= pack | v;
                            edges[v as usize] ^= dst_start as u32;
                            dst_start += 1;
                            left -= 1;
                        } else {
                            let tag = tok >> 14;
                            let v = hi | u32::from(tok & 0x3FFF);
                            let w = if tag < 3 {
                                (tag + 1) as usize
                            } else {
                                let w16 = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                                pos += 1;
                                w16 as usize
                            };
                            let take = w.min(left);
                            let v0 = v as usize;
                            for k in 0..take {
                                let u = dst_start + k;
                                let vv = v0 + k;
                                edges[u] ^= pack | (vv as u32);
                                edges[vv] ^= u as u32;
                            }
                            dst_start += take;
                            left -= take;
                            if w > take {
                                self.rem_v = v + take as u32;
                                self.rem_w = w - take;
                            }
                        }
                    } else if MODE == MODE_E4K {
                        if tok < 0x1000 {
                            let v = hi | u32::from(tok);
                            edges[dst_start] ^= pack | v;
                            edges[v as usize] ^= dst_start as u32;
                            dst_start += 1;
                            left -= 1;
                        } else {
                            let w = ((tok >> 12) + 1) as usize;
                            let v = hi | u32::from(tok & 0x0FFF);
                            let take = w.min(left);
                            let v0 = v as usize;
                            for k in 0..take {
                                let u = dst_start + k;
                                let vv = v0 + k;
                                edges[u] ^= pack | (vv as u32);
                                edges[vv] ^= u as u32;
                            }
                            dst_start += take;
                            left -= take;
                            if w > take {
                                self.rem_v = v + take as u32;
                                self.rem_w = w - take;
                            }
                        }
                    } else if tok < 0x8000 {
                        let v = hi | u32::from(tok);
                        edges[dst_start] ^= pack | v;
                        edges[v as usize] ^= dst_start as u32;
                        dst_start += 1;
                        left -= 1;
                    } else {
                        let v = hi | u32::from(tok & 0x7FFF);
                        let w = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]) as usize;
                        pos += 1;
                        let take = w.min(left);
                        let v0 = v as usize;
                        for k in 0..take {
                            let u = dst_start + k;
                            let vv = v0 + k;
                            edges[u] ^= pack | (vv as u32);
                            edges[vv] ^= u as u32;
                        }
                        dst_start += take;
                        left -= take;
                        if w > take {
                            self.rem_v = v + take as u32;
                            self.rem_w = w - take;
                        }
                    }
                }
                self.read_words += (pos - self.at) as u32;
                self.at = pos;
            }
        }

        #[inline(always)]
        pub fn wire_xor(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.rem_w > 0 {
                let take = self.rem_w.min(left);
                let v0 = self.rem_v;
                if mark.wrapping_sub(v0) < take as u32 {
                    *predecessor = (dst_start as u32) + (mark - v0);
                }
                for k in 0..take {
                    let u = dst_start + k;
                    let v = v0 as usize + k;
                    edges[u] ^= pack | (v as u32);
                    edges[v] ^= u as u32;
                }
                self.rem_v += take as u32;
                self.rem_w -= take;
                dst_start += take;
                left -= take;
                if left == 0 {
                    return;
                }
            }
            while left > 0 {
                while self.read_words == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let avail = (CHUNK - self.at).min((self.cut - self.read_words) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << Self::ESHIFT;
                let mut pos = self.at;
                let end_pos = self.at + avail;

                while left > 0 && pos < end_pos {
                    let tok = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                    pos += 1;
                    let (v, w) = if MODE == MODE_E16K {
                        let tag = tok >> 14;
                        let v = hi | u32::from(tok & 0x3FFF);
                        let w = if tag < 3 {
                            (tag + 1) as usize
                        } else {
                            let w16 = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                            pos += 1;
                            w16 as usize
                        };
                        (v, w)
                    } else if MODE == MODE_E4K {
                        let w = ((tok >> 12) + 1) as usize;
                        let v = hi | u32::from(tok & 0x0FFF);
                        (v, w)
                    } else if tok < 0x8000 {
                        (hi | u32::from(tok), 1usize)
                    } else {
                        let v = hi | u32::from(tok & 0x7FFF);
                        let w = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]) as usize;
                        pos += 1;
                        (v, w)
                    };
                    let take = w.min(left);
                    if mark.wrapping_sub(v) < take as u32 {
                        *predecessor = (dst_start as u32) + (mark - v);
                    }
                    let v0 = v as usize;
                    for k in 0..take {
                        let u = dst_start + k;
                        let vv = v0 + k;
                        edges[u] ^= pack | (vv as u32);
                        edges[vv] ^= u as u32;
                    }
                    dst_start += take;
                    left -= take;
                    if w > take {
                        self.rem_v = v + take as u32;
                        self.rem_w = w - take;
                    }
                }
                self.read_words += (pos - self.at) as u32;
                self.at = pos;
            }
        }

        #[inline(always)]
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let mut off = 0usize;
            let mut left = dst.len();
            if self.rem_w > 0 {
                let take = self.rem_w.min(left);
                let mut val = pack | self.rem_v;
                for e in &mut dst[off..off + take] {
                    *e = val;
                    val += 1;
                }
                self.rem_v += take as u32;
                self.rem_w -= take;
                off += take;
                left -= take;
                if left == 0 {
                    return;
                }
            }
            while left > 0 {
                while self.read_words == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let avail = (CHUNK - self.at).min((self.cut - self.read_words) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << Self::ESHIFT;
                let mut pos = self.at;
                let end_pos = self.at + avail;

                while left > 0 && pos < end_pos {
                    let tok = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                    pos += 1;
                    let (v, w) = if MODE == MODE_E16K {
                        let tag = tok >> 14;
                        let v = hi | u32::from(tok & 0x3FFF);
                        let w = if tag < 3 {
                            (tag + 1) as usize
                        } else {
                            let w16 = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]);
                            pos += 1;
                            w16 as usize
                        };
                        (v, w)
                    } else if MODE == MODE_E4K {
                        let w = ((tok >> 12) + 1) as usize;
                        let v = hi | u32::from(tok & 0x0FFF);
                        (v, w)
                    } else if tok < 0x8000 {
                        (hi | u32::from(tok), 1usize)
                    } else {
                        let v = hi | u32::from(tok & 0x7FFF);
                        let w = u16::from_le_bytes([chunk[2 * pos], chunk[2 * pos + 1]]) as usize;
                        pos += 1;
                        (v, w)
                    };
                    let take = w.min(left);
                    let mut val = pack | v;
                    for e in &mut dst[off..off + take] {
                        *e = val;
                        val += 1;
                    }
                    off += take;
                    left -= take;
                    if w > take {
                        self.rem_v = v + take as u32;
                        self.rem_w = w - take;
                    }
                }
                self.read_words += (pos - self.at) as u32;
                self.at = pos;
            }
        }
    }
}

// ============================================================================
// 3. `ikernel` wrapping VERBATIM `cand_r5_head/endpoint.rs`
// ============================================================================
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head"]
mod ikernel {
    use super::{outq_chunk, outq_runpair};
    use std::time::Instant;

    #[allow(dead_code)]
    mod endpoint;

    pub const LIMIT: u32 = 16;
    pub const INLINE: usize = 16;
    pub const LONG: u8 = u8::MAX;

    #[derive(Default, Clone, Copy, Debug)]
    pub struct StageTimes {
        pub s1_us: f64,
        pub s23_us: f64,
        pub s4_us: f64,
        pub s5_us: f64,
    }

    impl StageTimes {
        pub fn total_us(&self) -> f64 {
            self.s1_us + self.s23_us + self.s4_us + self.s5_us
        }
    }

    #[derive(Default, Clone, Copy, Debug)]
    #[allow(dead_code)]
    pub struct Counts {
        pub open_nodes: usize,
        pub steps: usize,
        pub rules: usize,
        pub n: usize,
        pub runs: usize,
        pub retained: usize,
        pub thick: usize,
        pub bands: usize,
        pub words: usize,
        pub copies: usize,
        pub traced: usize,
        pub thick_area: usize,
        pub unit_work: usize,
        pub longest: usize,
        pub table_bytes: usize,
        pub spirals: bool,
    }

    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    pub struct Strip {
        pub from: u32,
        pub to: u32,
        pub width: u32,
        pub byte: u8,
    }

    impl Strip {
        #[inline(always)]
        #[allow(dead_code)]
        pub fn unpack(w: &[u32]) -> Self {
            Strip {
                from: w[0],
                to: w[1],
                width: w[2] >> 8,
                byte: (w[2] & 0xFF) as u8,
            }
        }
    }

    #[derive(Clone, Copy)]
    pub struct Band(u64);

    impl Band {
        #[inline(always)]
        pub fn new(end: u32, dst: u32, code: u32) -> Self {
            Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
        }
        #[inline(always)]
        pub fn end(self) -> u32 {
            (self.0 & 0xF_FFFF) as u32
        }
        #[inline(always)]
        pub fn dst(self) -> u32 {
            ((self.0 >> 20) & 0xF_FFFF) as u32
        }
        #[inline(always)]
        pub fn code(self) -> u32 {
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
        runs: usize,
    }

    impl Default for First {
        fn default() -> Self {
            Self {
                thick_rows: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                m: 0,
                mark: 0,
                mark_byte: 0,
                runs: 0,
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

    pub fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
        let mut slot = [0; INLINE];
        slot[..4].copy_from_slice(&start.to_le_bytes());
        slot[4..8].copy_from_slice(&end.to_le_bytes());
        slot
    }

    pub fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
        &spelling[start..end]
    }

    pub fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
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

    #[inline(always)]
    fn split(
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
        mut visit: impl FnMut(u8, u32, u32, bool),
    ) {
        let end = pos + width;
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                visit(c, pos, p - pos, false);
            }
            visit(c, p, 1, true);
            if p + 1 < end {
                visit(c, p + 1, end - p - 1, false);
            }
        } else {
            visit(c, pos, width, width <= limit);
        }
    }

    pub enum RunInput<'a> {
        Cap127(&'a [u16]),
        UncappedU32(&'a [u32]),
        UncappedEsc16(&'a [u16]),
    }

    #[derive(Default)]
    pub struct Spills {
        pub spill_c64: outq_chunk::Spill<128>,
        pub spill_c128: outq_chunk::Spill<256>,
        pub spill_c256: outq_chunk::Spill<512>,
        pub spill_c512: outq_chunk::Spill<1024>,
        pub spill_rp128: outq_runpair::Spill<256>,
        pub spill_rp512: outq_runpair::Spill<1024>,
    }

    impl Spills {
        fn chunk_mut<const BYTES: usize>(&mut self) -> &mut outq_chunk::Spill<BYTES> {
            let any_mut: &mut dyn std::any::Any = match BYTES {
                128 => &mut self.spill_c64,
                256 => &mut self.spill_c128,
                512 => &mut self.spill_c256,
                1024 => &mut self.spill_c512,
                _ => unreachable!("unsupported chunk BYTES"),
            };
            any_mut.downcast_mut().expect("spill type match")
        }

        fn chunk_ref<const BYTES: usize>(&self) -> &outq_chunk::Spill<BYTES> {
            let any_ref: &dyn std::any::Any = match BYTES {
                128 => &self.spill_c64,
                256 => &self.spill_c128,
                512 => &self.spill_c256,
                1024 => &self.spill_c512,
                _ => unreachable!("unsupported chunk BYTES"),
            };
            any_ref.downcast_ref().expect("spill type match")
        }

        fn rp_mut<const BYTES: usize>(&mut self) -> &mut outq_runpair::Spill<BYTES> {
            let any_mut: &mut dyn std::any::Any = match BYTES {
                256 => &mut self.spill_rp128,
                1024 => &mut self.spill_rp512,
                _ => unreachable!("unsupported rp BYTES"),
            };
            any_mut.downcast_mut().expect("rp spill type match")
        }

        fn rp_ref<const BYTES: usize>(&self) -> &outq_runpair::Spill<BYTES> {
            let any_ref: &dyn std::any::Any = match BYTES {
                256 => &self.spill_rp128,
                1024 => &self.spill_rp512,
                _ => unreachable!("unsupported rp BYTES"),
            };
            any_ref.downcast_ref().expect("rp spill type match")
        }
    }

    #[derive(Default)]
    pub struct Kernel {
        endpoint: endpoint::Endpoint,
        predecessor: Option<u32>,
        pub spills: Spills,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        offsets: Vec<u32>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        pub force_wide: bool,
    }

    #[inline(always)]
    fn push_strip(edges: &mut Vec<u32>, ns: &mut usize, x: u32, thin_a: u32, thick_rows_a: u32, w: u32, c: u8) {
        let at = 3 * *ns;
        if edges.len() < at + 3 {
            let need = (at + 3).max(2 * edges.len()).max(1024);
            edges.reserve_exact(need - edges.len());
            edges.resize(need, 0);
        }
        edges[at..at + 3].copy_from_slice(&[x, thin_a + thick_rows_a, (w << 8) | u32::from(c)]);
        *ns += 1;
    }

    macro_rules! feed_input {
        ($input:expr, $p:expr, $pass_piece:expr, $pass_take:expr) => {
            match $input {
                RunInput::Cap127(records) => {
                    let mut pos = 0u32;
                    let mut i = 0usize;
                    let nrec = records.len();
                    while i < nrec {
                        let rec = records[i];
                        let c = (rec >> 8) as u8;
                        let mut width = u32::from(rec & 255);
                        i += 1;
                        if width <= LIMIT {
                            $pass_piece(c, pos, width, true, $p);
                            pos += width;
                        } else {
                            if width == 127 {
                                while i < nrec && (records[i] >> 8) as u8 == c {
                                    width += u32::from(records[i] & 255);
                                    i += 1;
                                }
                            }
                            $pass_take(c, pos, width, $p);
                            pos += width;
                        }
                    }
                }
                RunInput::UncappedU32(runs) => {
                    let mut pos = 0u32;
                    for &rec in runs {
                        let c = (rec >> 24) as u8;
                        let width = rec & 0x00FF_FFFF;
                        if width <= LIMIT {
                            $pass_piece(c, pos, width, true, $p);
                        } else {
                            $pass_take(c, pos, width, $p);
                        }
                        pos += width;
                    }
                }
                RunInput::UncappedEsc16(esc) => {
                    let mut pos = 0u32;
                    let mut i = 0usize;
                    let nrec = esc.len();
                    while i < nrec {
                        let rec = esc[i];
                        let c = (rec >> 8) as u8;
                        let mut width = u32::from(rec & 255);
                        i += 1;
                        if width <= LIMIT {
                            $pass_piece(c, pos, width, true, $p);
                            pos += width;
                        } else {
                            if width >= 128 {
                                width = ((width & 127) << 16) | u32::from(esc[i]);
                                i += 1;
                            }
                            $pass_take(c, pos, width, $p);
                            pos += width;
                        }
                    }
                }
            }
        };
    }

    impl Kernel {
        #[inline(never)]
        pub fn run_chunk<const CHUNK: usize, const BYTES: usize, const TIMED: bool>(
            &mut self,
            input: RunInput<'_>,
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
        ) -> StageTimes {
            let mut st = StageTimes::default();
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            out.resize(n + INLINE, 0);

            let mut f = First::default();
            let mut ns = 0usize;
            let heads = {
                let spill: &mut outq_chunk::Spill<BYTES> = self.spills.chunk_mut::<BYTES>();
                let mut queues = outq_chunk::Queues::<CHUNK, BYTES>::new(&mut out[..], spill, u32::MAX);
                let edges = &mut self.edges;

                let mut piece = |c: u8, x: u32, w: u32, keep: bool, p_row: u32| {
                    let a = usize::from(c);
                    if keep {
                        if p_row.wrapping_sub(x) < w {
                            f.mark = f.m + (p_row - x);
                            f.mark_byte = c;
                        }
                        queues.push_thin(a, f.m, w);
                        f.m += w;
                    } else {
                        let thin_a = queues.thin_of(a);
                        push_strip(edges, &mut ns, x, thin_a, f.thick_rows[a], w, c);
                        f.thick[a] += 1;
                        f.thick_rows[a] += w;
                    }
                };
                feed_input!(
                    input,
                    p,
                    |c, x, w, keep, p_row| piece(c, x, w, keep, p_row),
                    |c, pos, width, p_row: u32| {
                        f.runs += 1;
                        if p_row.wrapping_sub(pos) >= width {
                            piece(c, pos, width, false, p_row);
                        } else {
                            split(c, pos, width, p_row, LIMIT, |cc, xx, ww, kk| {
                                piece(cc, xx, ww, kk, p_row);
                            });
                        }
                    }
                );
                f.thin = queues.extract_thin();
                queues.finish_stats();
                queues.heads()
            };
            if let Some(t) = t0 {
                st.s1_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t1 = if TIMED { Some(Instant::now()) } else { None };
            let (cursors, mark, bits, wide) = self.run_s23(&f, ns, n as u32);
            if let Some(t) = t1 {
                st.s23_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t2 = if TIMED { Some(Instant::now()) } else { None };
            let m = f.m;
            self.predecessor = None;
            let spill_ref: &outq_chunk::Spill<BYTES> = self.spills.chunk_ref::<BYTES>();
            if !wide && m >= 65_536 {
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut predecessor = mark;
                let mb = usize::from(f.mark_byte);
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq_chunk::Reader::<CHUNK, BYTES>::new(&out[..], spill_ref, &heads, a);
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
                self.predecessor = Some(predecessor);
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq_chunk::Reader::<CHUNK, BYTES>::new(&out[..], spill_ref, &heads, a);
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
            if let Some(t) = t2 {
                st.s4_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t3 = if TIMED { Some(Instant::now()) } else { None };
            self.walk(n, m as usize, mark, bits, out);
            if let Some(t) = t3 {
                st.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            st
        }

        #[inline(never)]
        pub fn run_runpair<const MODE: u8, const CHUNK: usize, const BYTES: usize, const TIMED: bool>(
            &mut self,
            input: RunInput<'_>,
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
        ) -> StageTimes {
            let mut st = StageTimes::default();
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            out.resize(n + INLINE, 0);

            let mut f = First::default();
            let mut ns = 0usize;
            let heads = {
                let spill: &mut outq_runpair::Spill<BYTES> = self.spills.rp_mut::<BYTES>();
                let mut queues = outq_runpair::Queues::<MODE, CHUNK, BYTES>::new(&mut out[..], spill, u32::MAX);
                let edges = &mut self.edges;

                let mut piece = |c: u8, x: u32, w: u32, keep: bool, p_row: u32| {
                    let a = usize::from(c);
                    if keep {
                        if p_row.wrapping_sub(x) < w {
                            f.mark = f.m + (p_row - x);
                            f.mark_byte = c;
                        }
                        queues.push_thin(a, f.m, w);
                        f.m += w;
                    } else {
                        let thin_a = queues.thin_of(a);
                        push_strip(edges, &mut ns, x, thin_a, f.thick_rows[a], w, c);
                        f.thick[a] += 1;
                        f.thick_rows[a] += w;
                    }
                };
                feed_input!(
                    input,
                    p,
                    |c, x, w, keep, p_row| piece(c, x, w, keep, p_row),
                    |c, pos, width, p_row: u32| {
                        f.runs += 1;
                        if p_row.wrapping_sub(pos) >= width {
                            piece(c, pos, width, false, p_row);
                        } else {
                            split(c, pos, width, p_row, LIMIT, |cc, xx, ww, kk| {
                                piece(cc, xx, ww, kk, p_row);
                            });
                        }
                    }
                );
                f.thin = queues.extract_thin();
                queues.finish_stats();
                queues.heads()
            };
            if let Some(t) = t0 {
                st.s1_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t1 = if TIMED { Some(Instant::now()) } else { None };
            let (cursors, mark, bits, wide) = self.run_s23(&f, ns, n as u32);
            if let Some(t) = t1 {
                st.s23_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t2 = if TIMED { Some(Instant::now()) } else { None };
            let m = f.m;
            self.predecessor = None;
            let spill_ref: &outq_runpair::Spill<BYTES> = self.spills.rp_ref::<BYTES>();
            if !wide && m >= 65_536 {
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut predecessor = mark;
                let mb = usize::from(f.mark_byte);
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq_runpair::Reader::<MODE, CHUNK, BYTES>::new(&out[..], spill_ref, &heads, a);
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
                self.predecessor = Some(predecessor);
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq_runpair::Reader::<MODE, CHUNK, BYTES>::new(&out[..], spill_ref, &heads, a);
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
            if let Some(t) = t2 {
                st.s4_us = t.elapsed().as_secs_f64() * 1e6;
            }

            let t3 = if TIMED { Some(Instant::now()) } else { None };
            self.walk(n, m as usize, mark, bits, out);
            if let Some(t) = t3 {
                st.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            st
        }

        fn run_s23(&mut self, f: &First, ns: usize, n: u32) -> ([usize; 256], u32, u32, bool) {
            let mut stats = Counts {
                n: n as usize,
                runs: f.runs,
                retained: f.m as usize,
                thick: ns,
                ..Counts::default()
            };
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            assert!(f.m > 0 && all.iter().sum::<u32>() == n);
            let first = starts(all);
            let mut write = starts(f.thick);
            let need = self.arena_words(ns, f.m as usize);
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
            self.offsets.clear();
            let cursors = self.induced(ns, n, &f.thin, &first, &mut write, &mut stats);
            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let wide = self.force_wide || (self.lens.len() - 1 > (u32::MAX >> bits) as usize);
            if self.edges.len() < f.m as usize {
                self.edges.resize(f.m as usize, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(f.m as usize, 0);
            }
            (cursors, f.mark, bits, wide)
        }

        pub fn ws_bytes(&self, spill_bytes: usize) -> usize {
            use std::mem::size_of;
            let bands = size_of::<Band>() * self.bands.capacity();
            let spelling = self.spelling.capacity();
            let offsets = 4 * self.offsets.capacity();
            let edges = 4 * self.edges.capacity();
            let extra = 4 * self.extra.capacity();
            let inline16 = INLINE * self.inline16.capacity();
            let lens = self.lens.capacity();
            let ep = self.endpoint.scratch_bytes();
            spill_bytes + bands + spelling + offsets + edges + extra + inline16 + lens + ep
        }

        fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
            if let Some(before) = self.predecessor {
                emit_bidirectional(
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
                return;
            }
            emit(
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
    }

    #[allow(clippy::too_many_arguments)]
    fn emit(
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

    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional(
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

    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }
}

// ============================================================================
// 4. Corpora, Encoders (Cap127 / UncappedU32 / UncappedEsc16), & Solved Analogue I
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    uncapped_u32: Vec<u32>,
    uncapped_esc16: Vec<u16>,
    expected: Vec<u8>,
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

fn encode_uncapped_from_records(records: &[u16]) -> (Vec<u32>, Vec<u16>) {
    let mut u32s = Vec::with_capacity(records.len());
    let mut esc = Vec::with_capacity(records.len());
    let mut i = 0usize;
    let nrec = records.len();
    while i < nrec {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        if width == 127 {
            while i < nrec && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
        }
        u32s.push((u32::from(c) << 24) | width);
        if width <= 127 {
            esc.push((u16::from(c) << 8) | (width as u16));
        } else {
            debug_assert!(width < (1 << 23));
            esc.push((u16::from(c) << 8) | 0x80 | ((width >> 16) as u16));
            esc.push((width & 0xFFFF) as u16);
        }
    }
    (u32s, esc)
}

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        let b = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        l.resize(l.len() + len, b);
    }
    assert_eq!(l.len(), n);
    l
}

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let pos = cur[b as usize] as usize;
        fl[pos] = ((i as u32) << 8) | u32::from(b);
        cur[b as usize] += 1;
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

fn bwt_cyclic(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<u32> = t.iter().map(|&b| u32::from(b)).collect();
    let mut tmp = vec![0u32; n];
    let mut cnt = vec![0usize; n.max(256)];
    let mut sa2 = vec![0usize; n];
    for &b in t {
        cnt[b as usize] += 1;
    }
    let mut sum = 0usize;
    for c in cnt.iter_mut().take(256) {
        let v = *c;
        *c = sum;
        sum += v;
    }
    for i in 0..n {
        let b = t[i] as usize;
        sa[cnt[b]] = i;
        cnt[b] += 1;
    }
    let mut classes = 1u32;
    tmp[sa[0]] = 0;
    for i in 1..n {
        if t[sa[i]] != t[sa[i - 1]] {
            classes += 1;
        }
        tmp[sa[i]] = classes - 1;
    }
    std::mem::swap(&mut rank, &mut tmp);

    let mut h = 1usize;
    while h < n && (classes as usize) < n {
        for i in 0..n {
            let v = sa[i];
            sa2[i] = if v >= h { v - h } else { v + n - h };
        }
        let c_len = classes as usize;
        cnt[..c_len].fill(0);
        for &idx in &sa2 {
            cnt[rank[idx] as usize] += 1;
        }
        let mut s = 0usize;
        for c in cnt.iter_mut().take(c_len) {
            let v = *c;
            *c = s;
            s += v;
        }
        for &idx in &sa2 {
            let r = rank[idx] as usize;
            sa[cnt[r]] = idx;
            cnt[r] += 1;
        }
        tmp[sa[0]] = 0;
        let mut new_classes = 1u32;
        for i in 1..n {
            let cur = sa[i];
            let prev = sa[i - 1];
            let cur2 = if cur + h < n { cur + h } else { cur + h - n };
            let prev2 = if prev + h < n {
                prev + h
            } else {
                prev + h - n
            };
            if rank[cur] != rank[prev] || rank[cur2] != rank[prev2] {
                new_classes += 1;
            }
            tmp[cur] = new_classes - 1;
        }
        std::mem::swap(&mut rank, &mut tmp);
        classes = new_classes;
        h <<= 1;
    }
    let mut l = vec![0u8; n];
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
            l[i] = t[n - 1];
        } else {
            l[i] = t[pos - 1];
        }
    }
    (l, p)
}

/// Genuine Solved Analogue I BWTs:
///   - `ZeroSpill`: sigma = 12 <= 16, wide thin runs (w ~ 4..12, m/r_thin ~ 4.5..6), L1D frontier <= 12 KiB, 0 ovf!
///   - `Overflow`:  sigma = 12 <= 16, high m (~477k on 900k) where OutQ16 overflows `out[..]` (+81 KiB ovf)
///                  while RunPair-E16K stays 100% zero-spill inside `out[..]`!
fn build_solved_analogue_i() -> Vec<Block> {
    let mut res = Vec::new();
    let alpha = b"abcdefghijkl";
    let next_u = |s: &mut u64| -> usize {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 33) as usize
    };

    // (1) AnaI-ZeroSpill-300k & 900k: 20B repeated keys (avg ~7 copies) + 12B template -> rich w=4..12 runs!
    for &(name, n, seed) in &[
        ("AnaI-ZS-300k", 300_000usize, 0x511D_0001u64),
        ("AnaI-ZS-900k", 900_000usize, 0x511D_0002u64),
    ] {
        let mut rng = seed;
        let n_keys = n / 240;
        let mut keys = Vec::with_capacity(n_keys);
        for _ in 0..n_keys {
            let mut k = [0u8; 16];
            for b in &mut k {
                *b = alpha[4 + (next_u(&mut rng) % 8)];
            }
            keys.push(k);
        }
        let tmpl = b"abcdabdcabcd";
        let mut text = Vec::with_capacity(n);
        while text.len() < n {
            let k = keys[next_u(&mut rng) % n_keys];
            for &ch in tmpl {
                if text.len() < n {
                    text.push(ch);
                }
            }
            for &ch in &k {
                if text.len() < n {
                    text.push(ch);
                }
            }
        }
        let (l, p) = bwt_cyclic(&text);
        let records = records_of(&l);
        let (u32s, esc) = encode_uncapped_from_records(&records);
        let expected = forward_fl_invert(&l, p);
        assert_eq!(expected, text);
        res.push(Block {
            name: name.to_string(),
            n,
            p,
            records,
            uncapped_u32: u32s,
            uncapped_esc16: esc,
            expected,
        });
    }

    // (2) AnaI-Ovf-900k: m > n/2 (OutQ16 spills 81 chunks to ovf; RunPair stays zero-spill!)
    {
        let n = 900_000usize;
        let mut rng = 0xA11CE_0002u64;
        let n_keys = n / 120;
        let mut keys = Vec::with_capacity(n_keys);
        for _ in 0..n_keys {
            let mut k = [0u8; 8];
            for b in &mut k {
                *b = alpha[4 + (next_u(&mut rng) % 8)];
            }
            keys.push(k);
        }
        let tmpl = b"abcdabdc";
        let mut text = Vec::with_capacity(n);
        while text.len() < n {
            let k = keys[next_u(&mut rng) % n_keys];
            for &ch in tmpl {
                if text.len() < n {
                    text.push(ch);
                }
            }
            for &ch in &k {
                if text.len() < n {
                    text.push(ch);
                }
            }
        }
        let (l, p) = bwt_cyclic(&text);
        let records = records_of(&l);
        let (u32s, esc) = encode_uncapped_from_records(&records);
        let expected = forward_fl_invert(&l, p);
        assert_eq!(expected, text);
        res.push(Block {
            name: "AnaI-Ovf-900k".to_string(),
            n,
            p,
            records,
            uncapped_u32: u32s,
            uncapped_esc16: esc,
            expected,
        });
    }
    res
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
            let (u32s, esc) = encode_uncapped_from_records(&records);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
                uncapped_u32: u32s,
                uncapped_esc16: esc,
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
        let (u32s, esc) = encode_uncapped_from_records(&records);
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
            uncapped_u32: u32s,
            uncapped_esc16: esc,
            expected,
        });
    }
    blocks
}

// ============================================================================
// 5. Hard Exactness Suite (5,500+ Small + 250 Malformed FL + Epoch/Chunk Stress + All Corpora)
// ============================================================================

fn run_all_arms_exact(
    r5: &mut cand_r5_head::Kernel,
    ik: &mut ikernel::Kernel,
    l: &[u8],
    p: u32,
    tag: &str,
) {
    let recs = records_of(l);
    let (u32s, esc) = encode_uncapped_from_records(&recs);
    let n = l.len();
    let fl_exp = forward_fl_invert(l, p);
    let mut out_r5 = Vec::new();
    let mut out = Vec::new();
    r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_r5)
        .unwrap();
    assert_eq!(out_r5, fl_exp, "cand_r5_head vs FL mismatch on {tag}");

    // ChunkSweep: 64, 128, 256, 512
    ik.run_chunk::<64, 128, false>(ikernel::RunInput::Cap127(&recs), n, p, &mut out);
    assert_eq!(out, out_r5, "Chunk-64 mismatch on {tag}");
    ik.run_chunk::<128, 256, false>(ikernel::RunInput::Cap127(&recs), n, p, &mut out);
    assert_eq!(out, out_r5, "Chunk-128 mismatch on {tag}");
    ik.run_chunk::<256, 512, false>(ikernel::RunInput::Cap127(&recs), n, p, &mut out);
    assert_eq!(out, out_r5, "Chunk-256 mismatch on {tag}");
    ik.run_chunk::<512, 1024, false>(ikernel::RunInput::Cap127(&recs), n, p, &mut out);
    assert_eq!(out, out_r5, "Chunk-512 mismatch on {tag}");

    // RunPair: E16K (C128 & C512), E4K (C128), E32K (C128)
    ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, false>(
        ikernel::RunInput::Cap127(&recs),
        n,
        p,
        &mut out,
    );
    assert_eq!(out, out_r5, "RunPair-E16K-C128 mismatch on {tag}");
    ik.run_runpair::<{ outq_runpair::MODE_E16K }, 512, 1024, false>(
        ikernel::RunInput::Cap127(&recs),
        n,
        p,
        &mut out,
    );
    assert_eq!(out, out_r5, "RunPair-E16K-C512 mismatch on {tag}");
    ik.run_runpair::<{ outq_runpair::MODE_E4K }, 128, 256, false>(
        ikernel::RunInput::Cap127(&recs),
        n,
        p,
        &mut out,
    );
    assert_eq!(out, out_r5, "RunPair-E4K-C128 mismatch on {tag}");
    ik.run_runpair::<{ outq_runpair::MODE_E32K }, 128, 256, false>(
        ikernel::RunInput::Cap127(&recs),
        n,
        p,
        &mut out,
    );
    assert_eq!(out, out_r5, "RunPair-E32K-C128 mismatch on {tag}");

    // Uncapped-S1: U32 and Esc16
    ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedU32(&u32s), n, p, &mut out);
    assert_eq!(out, out_r5, "Uncapped-U32 mismatch on {tag}");
    ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedEsc16(&esc), n, p, &mut out);
    assert_eq!(out, out_r5, "Uncapped-Esc16 mismatch on {tag}");
}

fn count_fl_cycles(l: &[u8]) -> usize {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0usize; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let pos = cur[b as usize] as usize;
        fl[pos] = i;
        cur[b as usize] += 1;
    }
    let mut vis = vec![false; n];
    let mut cyc = 0usize;
    for i in 0..n {
        if !vis[i] {
            cyc += 1;
            let mut x = i;
            while !vis[x] {
                vis[x] = true;
                x = fl[x];
            }
        }
    }
    cyc
}

fn run_exactness_suite(
    solved: &[Block],
    wiki6: &[Block],
    nonwiki6: &[Block],
    code4: &[Block],
    w233: &[Block],
) {
    println!("================================================================================");
    println!("EXACTNESS VERIFICATION SUITE (ALL 10 ARMS vs cand_r5_head & forward-FL)");
    println!("================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut ik = ikernel::Kernel::default();

    let mut small_cnt = 0usize;
    for n in 1..=10usize {
        for mask in 0..(1u32 << n) {
            let mut l = Vec::with_capacity(n);
            for bit in 0..n {
                l.push(if (mask >> bit) & 1 == 1 { b'b' } else { b'a' });
            }
            run_all_arms_exact(&mut r5, &mut ik, &l, 0, "bin-p0");
            small_cnt += 1;
            if n >= 2 {
                run_all_arms_exact(&mut r5, &mut ik, &l, (n - 1) as u32, "bin-plast");
                small_cnt += 1;
            }
        }
    }

    let mut rng = 0xC48_0002_DEAD_BEEFu64;
    let next_u = |s: &mut u64| -> usize {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 33) as usize
    };
    for trial in 0..1600usize {
        let sigma = 2 + (next_u(&mut rng) % 8);
        let n_runs = 4 + (next_u(&mut rng) % 24);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = b'a' + (next_u(&mut rng) % sigma) as u8;
            let w = if next_u(&mut rng) % 4 == 0 {
                17 + (next_u(&mut rng) % 140)
            } else {
                1 + (next_u(&mut rng) % 16)
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u(&mut rng) % l.len()) as u32;
        run_all_arms_exact(&mut r5, &mut ik, &l, p, &format!("synth-thick-{trial}"));
        small_cnt += 1;
    }
    println!("  PASS: {small_cnt} small exhaustive + thick-strip + ovf cases 100% BYTE-EXACT across ALL 10 arms.");

    // 250+ malformed multi-cycle / pure-thick-cycle columns
    let mut mal_cnt = 0usize;
    let mut mal_multicyc = 0usize;
    for &w in &[17usize, 20, 64, 128, 257] {
        let mut l = Vec::new();
        l.push(b'z');
        l.resize(l.len() + w, b'b');
        l.resize(l.len() + w, b'a');
        let mc = count_fl_cycles(&l) > 1;
        for p in [0u32, 1, w as u32, (2 * w) as u32] {
            run_all_arms_exact(&mut r5, &mut ik, &l, p, "thick-cycle-2");
            mal_cnt += 1;
            mal_multicyc += usize::from(mc);
        }
        let mut l2 = Vec::new();
        l2.resize(w, b'a');
        l2.push(b'b');
        l2.resize(l2.len() + w, b'c');
        let mc2 = count_fl_cycles(&l2) > 1;
        for p in [0u32, w as u32, (w + 1) as u32] {
            run_all_arms_exact(&mut r5, &mut ik, &l2, p, "thick-self-loop");
            mal_cnt += 1;
            mal_multicyc += usize::from(mc2);
        }
    }
    for trial in 0..225usize {
        let mut l = Vec::new();
        let n_runs = 6 + (next_u(&mut rng) % 24);
        let sigma = 2 + (next_u(&mut rng) % 6);
        for _ in 0..n_runs {
            let b = b'A' + (next_u(&mut rng) % sigma) as u8;
            let w = match next_u(&mut rng) % 5 {
                0 => 17,
                1 => 127,
                2 => 128,
                3 => 1 + (next_u(&mut rng) % 16),
                _ => 17 + (next_u(&mut rng) % 160),
            };
            l.resize(l.len() + w, b);
        }
        let mc = count_fl_cycles(&l) > 1;
        let p = (next_u(&mut rng) % l.len()) as u32;
        run_all_arms_exact(&mut r5, &mut ik, &l, p, &format!("malformed-{trial}"));
        mal_cnt += 1;
        mal_multicyc += usize::from(mc);
    }
    println!(
        "  PASS: {mal_cnt} malformed columns ({mal_multicyc} verified multi-cycle FL) 100% BYTE-EXACT vs cand_r5_head & forward-FL."
    );

    // Epoch & chunk boundary adversarial stress (m ~ 70,000 crossing 4K, 16K, 32K, 64K + mark inside w=16 run!)
    {
        let mut l = Vec::with_capacity(72_000);
        let mut s = 0x1234_5678_9ABC_DEF0u64;
        while l.len() < 70_000 {
            let b = b'a' + (next_u(&mut s) % 4) as u8;
            let w = 1 + (next_u(&mut s) % 16);
            l.resize(l.len() + w, b);
        }
        for &p in &[0u32, 4095, 4096, 16383, 16384, 32767, 32768, 65535, 65536] {
            run_all_arms_exact(&mut r5, &mut ik, &l, p, &format!("epoch-stress-p{p}"));
        }
        let mut spiral = vec![b'a'; 200_000];
        spiral[0] = b'c';
        spiral[199_999] = b'b';
        run_all_arms_exact(&mut r5, &mut ik, &spiral, 0, "spiral-200k");
        println!("  PASS: 9 epoch-boundary (4K/16K/32K/64K) + chunk-straddle + spiral-200k stress cases 100% BYTE-EXACT.");
    }

    // Force-wide check + Solved + 6 Admitted + 6 Non-Wikidata + 4 Code/Docs
    let mut out = Vec::new();
    ik.force_wide = true;
    ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, false>(
        ikernel::RunInput::Cap127(&wiki6[0].records),
        wiki6[0].n,
        wiki6[0].p,
        &mut out,
    );
    assert_eq!(out, wiki6[0].expected, "RunPair-E16K force_wide mismatch");
    ik.force_wide = false;

    for blk in solved
        .iter()
        .chain(wiki6.iter())
        .chain(nonwiki6.iter())
        .chain(code4.iter())
    {
        ik.run_chunk::<64, 128, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Chunk-64 mismatch on {}", blk.name);
        ik.run_chunk::<128, 256, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Chunk-128 mismatch on {}", blk.name);
        ik.run_chunk::<256, 512, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Chunk-256 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Chunk-512 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, blk.expected, "RunPair-E16K-C128 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E16K }, 512, 1024, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, blk.expected, "RunPair-E16K-C512 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E4K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, blk.expected, "RunPair-E4K-C128 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E32K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, blk.expected, "RunPair-E32K-C128 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedU32(&blk.uncapped_u32), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Uncapped-U32 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedEsc16(&blk.uncapped_esc16), blk.n, blk.p, &mut out);
        assert_eq!(out, blk.expected, "Uncapped-Esc16 mismatch on {}", blk.name);
    }
    println!(
        "  PASS: {}/{} Solved + {}/{} Admitted Wiki + {}/{} Non-Wiki + {}/{} Code/Docs + force_wide 100% BYTE-EXACT across ALL 10 arms.",
        solved.len(), solved.len(), wiki6.len(), wiki6.len(), nonwiki6.len(), nonwiki6.len(), code4.len(), code4.len()
    );

    // All 233 full wikidata.bz2 blocks vs verbatim cand_r5_head across ALL 10 arms
    let mut out_r5 = Vec::new();
    for blk in w233 {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_r5)
            .unwrap();
        ik.run_chunk::<64, 128, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Chunk-64 mismatch on {}", blk.name);
        ik.run_chunk::<128, 256, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Chunk-128 mismatch on {}", blk.name);
        ik.run_chunk::<256, 512, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Chunk-256 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Chunk-512 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, out_r5, "RunPair-E16K-C128 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E16K }, 512, 1024, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, out_r5, "RunPair-E16K-C512 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E4K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, out_r5, "RunPair-E4K-C128 mismatch on {}", blk.name);
        ik.run_runpair::<{ outq_runpair::MODE_E32K }, 128, 256, false>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            &mut out,
        );
        assert_eq!(out, out_r5, "RunPair-E32K-C128 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedU32(&blk.uncapped_u32), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Uncapped-U32 mismatch on {}", blk.name);
        ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedEsc16(&blk.uncapped_esc16), blk.n, blk.p, &mut out);
        assert_eq!(out, out_r5, "Uncapped-Esc16 mismatch on {}", blk.name);
    }
    println!(
        "  PASS: {}/{} Full wikidata.bz2 blocks 100% BYTE-EXACT vs cand_r5_head across ALL 10 arms.",
        w233.len(), w233.len()
    );
}

// ============================================================================
// 6. Runtime OutQ Traffic Verification + Paired 1T Stage & 1T/2W/4W Multi-Worker Benchmarks
// ============================================================================

const ARM_NAMES: [&str; 11] = [
    "R5H-Ref",      // 0: verbatim cand_r5_head::Kernel
    "Chunk-512",    // 1: A/A control (CHUNK=512, 1024B)
    "Chunk-256",    // 2: CHUNK=256 (512B)
    "Chunk-128",    // 3: CHUNK=128 (256B)
    "Chunk-64",     // 4: CHUNK=64  (128B)
    "RP-E16K-C128", // 5: RunPair E16K + CHUNK=128
    "RP-E16K-C512", // 6: RunPair E16K + CHUNK=512 (isolates encoding from chunk size!)
    "RP-E4K-C128",  // 7: RunPair E4K (fixed 2B/run) + CHUNK=128
    "RP-E32K-C128", // 8: RunPair E32K + CHUNK=128
    "Uncap-U32",    // 9: Uncapped u32 `(c<<24)|len` + CHUNK=512
    "Uncap-Esc16",  // 10: Uncapped Esc16 `&[u16]` + CHUNK=512
];

fn run_arm_timed(
    arm: usize,
    r5: &mut cand_r5_head::Kernel,
    ik: &mut ikernel::Kernel,
    blk: &Block,
    out: &mut Vec<u8>,
) -> ikernel::StageTimes {
    match arm {
        0 => {
            let (c, e) = r5.run_stages(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, out);
            ikernel::StageTimes {
                s1_us: 0.0,
                s23_us: c,
                s4_us: 0.0,
                s5_us: e,
            }
        }
        1 => ik.run_chunk::<512, 1024, true>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out),
        2 => ik.run_chunk::<256, 512, true>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out),
        3 => ik.run_chunk::<128, 256, true>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out),
        4 => ik.run_chunk::<64, 128, true>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out),
        5 => ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, true>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            out,
        ),
        6 => ik.run_runpair::<{ outq_runpair::MODE_E16K }, 512, 1024, true>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            out,
        ),
        7 => ik.run_runpair::<{ outq_runpair::MODE_E4K }, 128, 256, true>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            out,
        ),
        8 => ik.run_runpair::<{ outq_runpair::MODE_E32K }, 128, 256, true>(
            ikernel::RunInput::Cap127(&blk.records),
            blk.n,
            blk.p,
            out,
        ),
        9 => ik.run_chunk::<512, 1024, true>(ikernel::RunInput::UncappedU32(&blk.uncapped_u32), blk.n, blk.p, out),
        _ => ik.run_chunk::<512, 1024, true>(ikernel::RunInput::UncappedEsc16(&blk.uncapped_esc16), blk.n, blk.p, out),
    }
}

fn run_arm_untimed(
    arm: usize,
    r5: &mut cand_r5_head::Kernel,
    ik: &mut ikernel::Kernel,
    blk: &Block,
    out: &mut Vec<u8>,
) {
    match arm {
        0 => {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, out)
                .unwrap();
        }
        1 => {
            ik.run_chunk::<512, 1024, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out);
        }
        2 => {
            ik.run_chunk::<256, 512, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out);
        }
        3 => {
            ik.run_chunk::<128, 256, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out);
        }
        4 => {
            ik.run_chunk::<64, 128, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, out);
        }
        5 => {
            ik.run_runpair::<{ outq_runpair::MODE_E16K }, 128, 256, false>(
                ikernel::RunInput::Cap127(&blk.records),
                blk.n,
                blk.p,
                out,
            );
        }
        6 => {
            ik.run_runpair::<{ outq_runpair::MODE_E16K }, 512, 1024, false>(
                ikernel::RunInput::Cap127(&blk.records),
                blk.n,
                blk.p,
                out,
            );
        }
        7 => {
            ik.run_runpair::<{ outq_runpair::MODE_E4K }, 128, 256, false>(
                ikernel::RunInput::Cap127(&blk.records),
                blk.n,
                blk.p,
                out,
            );
        }
        8 => {
            ik.run_runpair::<{ outq_runpair::MODE_E32K }, 128, 256, false>(
                ikernel::RunInput::Cap127(&blk.records),
                blk.n,
                blk.p,
                out,
            );
        }
        9 => {
            ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedU32(&blk.uncapped_u32), blk.n, blk.p, out);
        }
        _ => {
            ik.run_chunk::<512, 1024, false>(ikernel::RunInput::UncappedEsc16(&blk.uncapped_esc16), blk.n, blk.p, out);
        }
    }
}

fn print_runtime_traffic(blocks: &[Block], label: &str) {
    println!("\n--- Measured Runtime OutQ Traffic, Spill & Total Working Set — {label} ({} blocks) ---", blocks.len());
    let mut out = Vec::new();
    let tot_n: usize = blocks.iter().map(|b| b.n).sum();
    let nb = blocks.len() as f64;

    {
        let mut r5 = cand_r5_head::Kernel::default();
        for blk in blocks {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out)
                .unwrap();
        }
        let (ws_tot, dict_l1, _) = r5.scratch_report();
        println!(
            "  {:<13} | (verbatim cand_r5_head baseline)                                                    | tot_ws={:>7.1} KiB (dict_l1={:.1} KiB)",
            "R5H-Ref",
            ws_tot as f64 / 1024.0,
            dict_l1 as f64 / 1024.0
        );
    }

    macro_rules! report_chunk {
        ($name:expr, $ch:literal, $by:literal, $field:ident) => {{
            let mut ik = ikernel::Kernel::default();
            let (mut sum_ch, mut sum_ovf, mut sum_cuts) = (0usize, 0usize, 0usize);
            for blk in blocks {
                ik.run_chunk::<$ch, $by, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
                sum_ch += ik.spills.$field.last_chunks as usize;
                sum_ovf += ik.spills.$field.last_ovf;
                sum_cuts += ik.spills.$field.last_cuts;
            }
            let alloc_b = sum_ch * $by;
            let sp_b = ik.spills.$field.scratch_bytes();
            let ws_b = ik.ws_bytes(sp_b);
            println!(
                "  {:<13} | chunks/blk={:>6.1} | out_alloc/blk={:>6.1} KiB (S1+S4={:.3} B/B) | ovf={:>5} | cuts/blk={:>5.1} | spill={:>6.1}K | tot_ws={:>7.1} KiB",
                $name,
                sum_ch as f64 / nb,
                alloc_b as f64 / (1024.0 * nb),
                2.0 * alloc_b as f64 / tot_n as f64,
                sum_ovf,
                sum_cuts as f64 / nb,
                sp_b as f64 / 1024.0,
                ws_b as f64 / 1024.0
            );
        }};
    }
    report_chunk!("Chunk-512", 512, 1024, spill_c512);
    report_chunk!("Chunk-256", 256, 512, spill_c256);
    report_chunk!("Chunk-128", 128, 256, spill_c128);
    report_chunk!("Chunk-64", 64, 128, spill_c64);

    macro_rules! report_rp {
        ($name:expr, $mode:expr, $ch:literal, $by:literal, $field:ident) => {{
            let mut ik = ikernel::Kernel::default();
            let (mut sum_ch, mut sum_w, mut sum_ovf, mut sum_cuts) = (0usize, 0usize, 0usize, 0usize);
            for blk in blocks {
                ik.run_runpair::<$mode, $ch, $by, false>(ikernel::RunInput::Cap127(&blk.records), blk.n, blk.p, &mut out);
                sum_ch += ik.spills.$field.last_chunks as usize;
                sum_w += ik.spills.$field.last_words;
                sum_ovf += ik.spills.$field.last_ovf;
                sum_cuts += ik.spills.$field.last_cuts;
            }
            let alloc_b = sum_ch * $by;
            let useful_b = 2 * sum_w;
            let sp_b = ik.spills.$field.scratch_bytes();
            let ws_b = ik.ws_bytes(sp_b);
            println!(
                "  {:<13} | chunks/blk={:>6.1} | out_alloc/blk={:>6.1} KiB (use={:>5.1}K, S1+S4={:.3} B/B) | ovf={:>5} | cuts/blk={:>5.1} | spill={:>6.1}K | tot_ws={:>7.1} KiB",
                $name,
                sum_ch as f64 / nb,
                alloc_b as f64 / (1024.0 * nb),
                useful_b as f64 / (1024.0 * nb),
                2.0 * alloc_b as f64 / tot_n as f64,
                sum_ovf,
                sum_cuts as f64 / nb,
                sp_b as f64 / 1024.0,
                ws_b as f64 / 1024.0
            );
        }};
    }
    report_rp!("RP-E16K-C128", { outq_runpair::MODE_E16K }, 128, 256, spill_rp128);
    report_rp!("RP-E16K-C512", { outq_runpair::MODE_E16K }, 512, 1024, spill_rp512);
    report_rp!("RP-E4K-C128", { outq_runpair::MODE_E4K }, 128, 256, spill_rp128);
    report_rp!("RP-E32K-C128", { outq_runpair::MODE_E32K }, 128, 256, spill_rp128);
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!("\n====================================================================================================");
    println!("1T STAGE BREAKDOWN (PAIRED {rounds} ROUNDS, min [p50] us sum across {} blocks) — {label}", blocks.len());
    println!("====================================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut ik = ikernel::Kernel::default();
    let mut out = Vec::new();

    for blk in blocks {
        for arm in 0..ARM_NAMES.len() {
            run_arm_untimed(arm, &mut r5, &mut ik, blk, &mut out);
        }
    }

    let mut s1_min = [0.0f64; 11];
    let mut s23_min = [0.0f64; 11];
    let mut s4_min = [0.0f64; 11];
    let mut s5_min = [0.0f64; 11];
    let mut tot_min = [0.0f64; 11];
    let mut tot_p50 = [0.0f64; 11];

    for blk in blocks {
        let mut v_s1: [Vec<f64>; 11] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let mut v_s23: [Vec<f64>; 11] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let mut v_s4: [Vec<f64>; 11] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let mut v_s5: [Vec<f64>; 11] = std::array::from_fn(|_| Vec::with_capacity(rounds));
        let mut v_tot: [Vec<f64>; 11] = std::array::from_fn(|_| Vec::with_capacity(rounds));

        for r in 0..rounds {
            for step in 0..11 {
                let arm = if r % 2 == 0 { step } else { 10 - step };
                let st = run_arm_timed(arm, &mut r5, &mut ik, blk, &mut out);
                v_s1[arm].push(st.s1_us);
                v_s23[arm].push(st.s23_us);
                v_s4[arm].push(st.s4_us);
                v_s5[arm].push(st.s5_us);
                v_tot[arm].push(st.total_us());
            }
        }
        let stat = |v: &mut Vec<f64>| -> (f64, f64) {
            v.sort_by(|a, b| a.total_cmp(b));
            (v[0], v[v.len() / 2])
        };
        for arm in 0..11 {
            s1_min[arm] += stat(&mut v_s1[arm]).0;
            s23_min[arm] += stat(&mut v_s23[arm]).0;
            s4_min[arm] += stat(&mut v_s4[arm]).0;
            s5_min[arm] += stat(&mut v_s5[arm]).0;
            let (mn, md) = stat(&mut v_tot[arm]);
            tot_min[arm] += mn;
            tot_p50[arm] += md;
        }
    }

    let base_s1 = s1_min[1];
    let base_s4 = s4_min[1];
    let base_s14 = s1_min[1] + s4_min[1];
    let base_tot = tot_min[0];
    let aa_tot = tot_min[1];
    let base_p50 = tot_p50[0];
    println!(
        "{:<13} | {:>8} ({:>6}) | {:>7} | {:>8} ({:>6}) | {:>8} ({:>6}) | {:>8} | {:>8} ({:>6} R5, {:>6} C512) | p50 ({:>6} R5)",
        "Arm", "S1 us", "vsC512", "S23 us", "S4 us", "vsC512", "S1+S4", "vsC512", "S5 us", "Min us", "vs", "vs", "vs"
    );
    for arm in 0..11 {
        let s14 = s1_min[arm] + s4_min[arm];
        if arm == 0 {
            println!(
                "{:<13} | {:>8} ({:>6}) | {:>7.1} | {:>8} ({:>6}) | {:>8} ({:>6}) | {:>8.1} | {:>8.1} ({:>5.4}x R5, {:>5.4}x C512) | {:>8.1} ({:>5.4}x)",
                ARM_NAMES[arm], "—", "—", s23_min[arm], "—", "—", "—", "—", s5_min[arm], tot_min[arm], 1.0, tot_min[arm] / aa_tot, tot_p50[arm], 1.0
            );
        } else {
            println!(
                "{:<13} | {:>8.1} ({:>5.3}x) | {:>7.1} | {:>8.1} ({:>5.3}x) | {:>8.1} ({:>5.3}x) | {:>8.1} | {:>8.1} ({:>5.4}x R5, {:>5.4}x C512) | {:>8.1} ({:>5.4}x)",
                ARM_NAMES[arm],
                s1_min[arm],
                s1_min[arm] / base_s1,
                s23_min[arm],
                s4_min[arm],
                s4_min[arm] / base_s4,
                s14,
                s14 / base_s14,
                s5_min[arm],
                tot_min[arm],
                tot_min[arm] / base_tot,
                tot_min[arm] / aa_tot,
                tot_p50[arm],
                tot_p50[arm] / base_p50
            );
        }
    }
}

fn bench_multiworker(
    blocks: &[Block],
    label: &str,
    arms: &[usize],
    rounds: usize,
    iters_1w: usize,
    iters_mw: usize,
) {
    println!("\n--- Multi-Worker (1T / 2W / 4W, paired alternating {rounds} rounds, min / p50 wall ms) — {label} ---");
    for &workers in &[1usize, 2, 4] {
        let mut results: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); arms.len()];
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for r in 0..rounds {
            for idx in 0..arms.len() {
                let ai = if r % 2 == 0 { idx } else { arms.len() - 1 - idx };
                let arm = arms[ai];
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut ik = ikernel::Kernel::default();
                            let mut out = Vec::new();
                            for blk in blocks {
                                run_arm_untimed(arm, &mut r5, &mut ik, blk, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_arm_untimed(arm, &mut r5, &mut ik, blk, &mut out);
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
                results[ai].push(t_wall);
            }
        }
        let mut mins = Vec::with_capacity(arms.len());
        let mut meds = Vec::with_capacity(arms.len());
        for v in &mut results {
            v.sort_by(|a, b| a.total_cmp(b));
            mins.push(v[0]);
            meds.push(v[v.len() / 2]);
        }
        let base_min = mins[0];
        let base_med = meds[0];
        print!("  {}W min: ", workers);
        for (i, &arm) in arms.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{}={:.2}ms ({:.4}x)", ARM_NAMES[arm], mins[i], mins[i] / base_min);
        }
        println!();
        print!("  {}W p50: ", workers);
        for (i, &arm) in arms.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{}={:.2}ms ({:.4}x)", ARM_NAMES[arm], meds[i], meds[i] / base_med);
        }
        println!();
    }
}

fn resolve_corpus_path(rel: &str) -> Option<String> {
    let candidates = [
        format!("/tmp/mibz2_corpora/{rel}"),
        format!("/home/antigravity/mibz2_corpora_cache/{rel}"),
    ];
    for p in candidates {
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}

fn main() {
    let bench_only = std::env::args().any(|a| a == "--bench-only");
    let solved = build_solved_analogue_i();
    let wiki6 = load_six_admitted();
    let nonwiki_names = [
        "jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0",
    ];
    let nonwiki_path = resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin")
        .expect("non_wikidata_blocks.bin not found");
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false, !bench_only);

    let code4 = if let Some(code_path) = resolve_corpus_path("non_wikidata/code_docs_blocks.bin") {
        let code_names = ["code-0", "code-1", "code-2", "code-3"];
        load_bin_blocks(&code_path, Some(&code_names), false, !bench_only)
    } else {
        Vec::new()
    };

    let w431_path = resolve_corpus_path("wikidata_431_blocks.bin")
        .expect("wikidata_431_blocks.bin not found");
    let w233 = load_bin_blocks(&w431_path, None, true, false);

    if !bench_only {
        run_exactness_suite(&solved, &wiki6, &nonwiki6, &code4, &w233);
    }

    print_runtime_traffic(&solved, "Solved Analogue I (ZS-300k/900k + Ovf-900k)");
    print_runtime_traffic(&wiki6, "6 Admitted Wikidata Blocks");
    print_runtime_traffic(&w233, "233 Full wikidata.bz2 Blocks");
    print_runtime_traffic(&nonwiki6, "6 Real Non-Wikidata Blocks");
    if !code4.is_empty() {
        print_runtime_traffic(&code4, "4 Real Code & Docs Blocks (code_docs_blocks.bin)");
    }

    bench_stage_1t(&solved, "Solved Analogue I (3 blocks)", 9);
    bench_stage_1t(&wiki6, "6 Admitted Wikidata Blocks", 13);
    bench_stage_1t(&nonwiki6, "6 Real Non-Wikidata Blocks", 13);
    if !code4.is_empty() {
        bench_stage_1t(&code4, "4 Real Code & Docs Blocks", 9);
    }

    // Multi-worker (1T / 2W / 4W) across ALL key arms in two clean groups
    let chunk_arms = [0usize, 1, 2, 3, 4];
    let rp_uncap_arms = [0usize, 1, 5, 6, 7, 8, 9, 10];

    bench_multiworker(&solved, "Solved Analogue I (ChunkSweep)", &chunk_arms, 7, 2, 1);
    bench_multiworker(&solved, "Solved Analogue I (RunPair + Uncapped)", &rp_uncap_arms, 7, 2, 1);

    bench_multiworker(&wiki6, "6 Admitted Wikidata (ChunkSweep)", &chunk_arms, 7, 3, 2);
    bench_multiworker(&wiki6, "6 Admitted Wikidata (RunPair + Uncapped)", &rp_uncap_arms, 7, 3, 2);

    bench_multiworker(&nonwiki6, "6 Non-Wikidata (ChunkSweep)", &chunk_arms, 7, 3, 2);
    bench_multiworker(&nonwiki6, "6 Non-Wikidata (RunPair + Uncapped)", &rp_uncap_arms, 7, 3, 2);

    if !code4.is_empty() {
        bench_multiworker(&code4, "4 Code & Docs (ChunkSweep)", &chunk_arms, 5, 2, 1);
        bench_multiworker(&code4, "4 Code & Docs (RunPair + Uncapped)", &rp_uncap_arms, 5, 2, 1);
    }

    // Full 233 wikidata.bz2 multi-worker shootout!
    let w233_arms = [0usize, 1, 2, 3, 4, 5, 7, 9, 10];
    bench_multiworker(&w233, "233 Full wikidata.bz2 (All Key Arms)", &w233_arms, 3, 1, 1);
}
