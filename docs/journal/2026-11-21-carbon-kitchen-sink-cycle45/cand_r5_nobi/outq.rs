//! The thin queues in the block's output buffer (the Mechanisms line's `OutQ16`,
//! `docs/journal/2026-11-21-carbon-cycle16-swar-outq16.md` on `research/carbon-word-transport`).
//!
//! The first pass queues each byte's retained ranks; the wiring reads them; only then does the
//! walk write the block's bytes into `out`. So the queues live in `out` itself, which the walk
//! writes over: no buffer of their own, and the cache lines they use are the ones the walk is
//! about to write. (The line's ablation: the same chunks in a buffer of their own lost the
//! gain, so the reuse of `out`'s lines, not the chunking, is what it buys.)
//!
//! **The layout:** `out` (its `n + 16` bytes) is cut into chunks of `CHUNK` ranks, two bytes a
//! rank. Each byte's queue is a list of chunks, taken in turn as the queues fill, linked by
//! `Spill::next`; a chunk past the end of `out` is an overflow chunk in `Spill::ovf` (none on a
//! full block of up to about 318,000 retained rows; every chunk on a block under 1 KiB).
//!
//! **The ranks in 16 bits:** a queue keeps a rank's low 16 bits. Its high bits, the rank's
//! epoch, are recovered from `Spill::cuts`: when the retained rows reach a multiple of 2^16
//! (a run is split there), every byte's queue length is recorded, so the entries of byte `a`'s
//! queue before `cuts[e][a]` are of epochs up to `e`, and the queue's entries are read in order.
//!
//! **Safety:** no `unsafe`; every index is bounds-checked.

/// Ranks a chunk holds
const CHUNK: usize = 512;
/// A chunk's bytes
const BYTES: usize = 2 * CHUNK;
/// No chunk
const NONE: u32 = u32::MAX;
/// Ranks an epoch holds
const EPOCH: u32 = 1 << 16;

/// The queues' state kept between blocks: the chunks' links, the overflow chunks and the
/// epochs' cuts (on the admitted blocks, a few KiB in all)
#[derive(Default)]
pub(super) struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; BYTES]>,
    cuts: Vec<[u32; 256]>,
    /// (test only) the chunks the last block's first pass took
    #[cfg(test)]
    pub(super) taken: u32,
}

impl Spill {
    /// (test only) the scratch it keeps, in bytes
    #[cfg(test)]
    pub(super) fn scratch(&self) -> usize {
        4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
    }

    /// (test only) the overflow chunks the last block took
    #[cfg(test)]
    pub(super) fn overflow(&self) -> usize {
        self.ovf.len()
    }
}

/// The queues while the first pass writes them
pub(super) struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    /// Packed `(thin[a] << 32) | (tail << 10) | fill` (`u32::MAX` in low 32 before first chunk)
    pt: [u64; 256],
    limit: u32,
    over: bool,
}

/// Where the queues are, once written: each byte's first chunk
pub(super) struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a> Queues<'a> {
    /// Empty queues in `out`, whose bytes are free until the walk, for at most `limit` ranks
    pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
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

    /// Whether a push went past the queues' limit (the block is refused)
    pub(super) fn over(&self) -> bool {
        self.over
    }

    /// (test only) the chunks taken
    #[cfg(test)]
    pub(super) fn chunks(&self) -> u32 {
        self.chunks
    }

    pub(super) fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    /// Queue the `w` ranks from `m` for byte `a`, whose queue length (before them) is
    /// `thin[a]`. A thin run has at most 16 rows, so the common case is one fixed block of 16
    /// ranks written (an iota from `m`), its first `w` kept: within the byte's last chunk, in
    /// `out`, and within an epoch. Anything else takes `push_slow`.
    #[inline(always)]
    pub(super) fn thin_of(&self, a: usize) -> u32 {
        (self.pt[a] >> 32) as u32
    }

    pub(super) fn extract_thin(&self) -> [u32; 256] {
        std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
    }

    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
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

    /// `push` across a chunk's end or an epoch's, into an overflow chunk, or of more than 16
    /// ranks (a limit above 16, in the tests): rank by rank. Past the queues' `limit` it sets
    /// `over` and writes nothing: every chunk is taken here, so the queues take none once the
    /// block is refused, and `push` itself pays nothing for the limit (only a chunk's end
    /// brings it here, and its fast case writes within a chunk already taken).
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

/// A byte's queue read in order: its chunk, the rank within it, the ranks read, and the
/// epoch with the queue length at which it ends
pub(super) struct Reader<'a> {
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

impl<'a> Reader<'a> {
    /// Byte `a`'s queue, from its first rank
    pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
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

    /// Each arrow from row u to v contributes v to u's low bits and u to
    /// v's low bits by XOR. The high bits of u hold its word code. Save the
    /// incoming neighbor of the mark to start the walk in either direction.

    #[inline(always)]
    pub(super) fn wire_xor_nomark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
    ) {
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
    pub(super) fn wire_xor(
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

    /// The next `span` ranks, each with `pack` added, into `dst`
    #[inline(always)]
    pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            // the common case: one chunk of `out`, one epoch
            let hi = pack | (self.epoch << 16);
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            let mut i = 0usize;
            while i + 16 <= span {
                let b = &bytes[2 * i..2 * i + 32];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                let d = &mut dst[i..i + 16];
                d[0] = hi | (q0 as u32 & 0xFFFF);  d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF); d[3] = hi | ((q0 >> 48) as u32);
                d[4] = hi | (q1 as u32 & 0xFFFF);  d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF); d[7] = hi | ((q1 >> 48) as u32);
                d[8] = hi | (q2 as u32 & 0xFFFF);  d[9] = hi | ((q2 >> 16) as u32 & 0xFFFF);
                d[10] = hi | ((q2 >> 32) as u32 & 0xFFFF); d[11] = hi | ((q2 >> 48) as u32);
                d[12] = hi | (q3 as u32 & 0xFFFF); d[13] = hi | ((q3 >> 16) as u32 & 0xFFFF);
                d[14] = hi | ((q3 >> 32) as u32 & 0xFFFF); d[15] = hi | ((q3 >> 48) as u32);
                i += 16;
            }
            if i + 8 <= span {
                let b = &bytes[2 * i..2 * i + 16];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let d = &mut dst[i..i + 8];
                d[0] = hi | (q0 as u32 & 0xFFFF);
                d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF);
                d[3] = hi | ((q0 >> 48) as u32);
                d[4] = hi | (q1 as u32 & 0xFFFF);
                d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF);
                d[7] = hi | ((q1 >> 48) as u32);
                i += 8;
            }
            while i < span {
                dst[i] = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                i += 1;
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

impl Spill { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity() } }
