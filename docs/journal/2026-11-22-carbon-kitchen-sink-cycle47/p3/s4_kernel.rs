//! Cycle 47 P3 — Track 1 Experimental `S4` & `S5` Kernel over verbatim `cand_r5_head/endpoint.rs`.
//! Reuses `cand_r5_head`'s exact `S1/S2/S3` (`endpoint.rs` & `outq` Pass 1) so every `S4` and `S5`
//! substitute is measured apples-to-apples against `cand_r5_head` and `cand_r5_nobi`.

#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/endpoint.rs"]
mod endpoint;

/// Ranks a chunk holds
const CHUNK: usize = 512;
/// A chunk's bytes
const BYTES: usize = 2 * CHUNK;
/// No chunk
const NONE: u32 = u32::MAX;
/// Ranks an epoch holds
const EPOCH: u32 = 1 << 16;

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; BYTES]>,
    cuts: Vec<[u32; 256]>,
}

impl Spill {
    fn scratch_bytes(&self) -> usize {
        4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
    }
}

struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    pt: [u64; 256],
    limit: u32,
    over: bool,
}

struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a> Queues<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
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

    fn over(&self) -> bool {
        self.over
    }

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    #[inline(always)]
    fn thin_of(&self, a: usize) -> u32 {
        (self.pt[a] >> 32) as u32
    }

    fn extract_thin(&self) -> [u32; 256] {
        std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
    }

    #[inline(always)]
    fn push_thin(&mut self, a: usize, m: u32, w: u32) {
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

struct Reader<'a> {
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
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
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

    #[inline(always)]
    fn wire_xor_nomark(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32) {
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
    fn wire_xor(
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

    /// Pure forward wire (16x unrolled, verbatim `cand_r5_nobi`).
    #[inline(always)]
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
    #[inline(always)]
    fn unpack(w: &[u32]) -> Self {
        Strip {
            from: w[0],
            to: w[1],
            width: w[2] >> 8,
            byte: (w[2] & 0xFF) as u8,
        }
    }
}

#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S4Mode {
    /// Verbatim `cand_r5_head`: `edges.fill(0)` + per-byte `wire_xor` + `emit_bidirectional`
    HeadVerbatim,
    /// Epoch-filtered 16x unrolled `wire_xor` on `a == mb` (eliminates scalar `wire_xor` on non-mark epochs, +0 B WS)
    HeadEpoch16x,
    /// Verbatim `cand_r5_nobi`: pure `wire` + single-cursor `emit`
    NobiVerbatim,
    /// Zero-Skip 2-pass Galois/Frobenius L1D/L2-tiled monotone `OutQ16` reverse sweep (`tile` divides 65536)
    TileRev(usize),
    /// Single-pass `v`-tiled `OutQ16` sweep (with `edges.fill(0)`, reads `OutQ16` once)
    SinglePassTileV(usize),
    /// Zero-Skip 2-pass using `L1` software write-combining buffers (32 buckets x 32 pairs in stack)
    L1BufScatter,
    /// Two-pass zero-skip: Pass 1 pure `nobi::wire` + Pass 2 `OutQ16` untiled reverse scatter (isolates `fill(0)` + pure-store `edges[u]` vs tiling)
    ZeroSkipUntiled,
    /// Non-XOR 2-cursor Dual-Forward (`K=2`), pure `nobi::wire` (zero reverse scatter!) + `stage` buffer
    DualFwd,
    /// Non-XOR 4-cursor Quad-Forward (`K=4`, Whitworth 1897 chord-stitched), pure `nobi::wire` + `stage` buffers
    QuadFwd,
}

#[derive(Default)]
pub struct StageTimes {
    pub s1_s3_us: f64,
    pub s4_us: f64,
    pub s5_us: f64,
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    predecessor: Option<u32>,
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    /// Reusable staging buffers for DualFwd / QuadFwd only (0 capacity for all XOR/Nobi modes!)
    stage1: Vec<u8>,
    stage2: Vec<u8>,
    stage3: Vec<u8>,
}

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
pub const LIMIT: u32 = 16;

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
        First {
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

struct Pass<'a> {
    queues: Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
        if width <= limit {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece(c, x, w, keep, p)
        });
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
            if self.ns < self.cap_strips {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3)
                        .max(2 * self.edges.len())
                        .max(1024)
                        .min(self.cap_strips.saturating_mul(3));
                    self.edges.reserve_exact(need - self.edges.len());
                    self.edges.resize(need, 0);
                }
                let thin_a = self.queues.thin_of(a);
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    thin_a + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
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

/// Active-byte cursor for Galois/Frobenius L1D/L2-tiled monotone `OutQ16` sweep.
#[derive(Clone, Copy)]
struct TileCursor {
    a: u8,
    id: u32,
    at: u16,
    read: u32,
    epoch: u16,
    cut: u32,
    band_idx: u32,
    band_end: u32,
    u_cur: u32,
    u_left: u32,
    pack: u32,
    peek_v: u32,
}

impl TileCursor {
    #[inline(always)]
    fn init(
        a: usize,
        bands: &[Band],
        b_start: usize,
        b_end: usize,
        out: &[u8],
        spill: &Spill,
        heads: &Heads,
        bits: u32,
    ) -> Option<Self> {
        if b_start >= b_end {
            return None;
        }
        let b0 = bands[b_start];
        let mut c = TileCursor {
            a: a as u8,
            id: heads.heads[a],
            at: 0,
            read: 0,
            epoch: 0,
            cut: spill.cuts.first().map_or(u32::MAX, |ct| ct[a]),
            band_idx: b_start as u32 + 1,
            band_end: b_end as u32,
            u_cur: b0.dst(),
            u_left: b0.end(),
            pack: b0.code() << bits,
            peek_v: u32::MAX,
        };
        c.refresh_peek(out, spill, heads.in_out);
        Some(c)
    }

    #[inline(always)]
    fn refresh_peek(&mut self, out: &[u8], spill: &Spill, in_out: u32) {
        if self.u_left == 0 {
            self.peek_v = u32::MAX;
            return;
        }
        while self.read == self.cut {
            self.epoch += 1;
            self.cut = spill
                .cuts
                .get(self.epoch as usize)
                .map_or(u32::MAX, |ct| ct[self.a as usize]);
        }
        if self.at as usize == CHUNK {
            self.id = spill.next[self.id as usize];
            self.at = 0;
        }
        let chunk: &[u8] = if self.id < in_out {
            let base = self.id as usize * BYTES;
            &out[base..base + BYTES]
        } else {
            &spill.ovf[(self.id - in_out) as usize][..]
        };
        let off = 2 * self.at as usize;
        let lo = u16::from_le_bytes([chunk[off], chunk[off + 1]]) as u32;
        self.peek_v = (u32::from(self.epoch) << 16) | lo;
    }

    #[inline(always)]
    fn advance_band(&mut self, bands: &[Band], bits: u32) -> bool {
        if self.band_idx < self.band_end {
            let b = bands[self.band_idx as usize];
            self.band_idx += 1;
            self.u_cur = b.dst();
            self.u_left = b.end();
            self.pack = b.code() << bits;
            true
        } else {
            self.peek_v = u32::MAX;
            false
        }
    }
}

/// Pass 2 of `TileRev`: L1D/L2-tiled reverse-edge accumulation `edges[v] ^= u` over monotone `OutQ16`.
/// Requires `tile` to be a power of 2 dividing 65536 (so each tile lies strictly within one epoch).
#[inline(never)]
fn tile_rev_pass(
    edges: &mut [u32],
    bands: &[Band],
    cursors: &[usize; 256],
    out: &[u8],
    spill: &Spill,
    heads: &Heads,
    m: usize,
    tile: usize,
    mark: u32,
    mb: u8,
) -> u32 {
    debug_assert!(tile.is_power_of_two() && tile <= EPOCH as usize);
    let mut active = [TileCursor {
        a: 0,
        id: NONE,
        at: 0,
        read: 0,
        epoch: 0,
        cut: u32::MAX,
        band_idx: 0,
        band_end: 0,
        u_cur: 0,
        u_left: 0,
        pack: 0,
        peek_v: u32::MAX,
    }; 256];
    let mut n_act = 0usize;
    for a in 0..256 {
        let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
        if let Some(c) = TileCursor::init(a, bands, cursors[a], end, out, spill, heads, 0) {
            active[n_act] = c;
            n_act += 1;
        }
    }
    let in_out = heads.in_out;
    let mut predecessor = mark;
    let mut tile_lo = 0usize;
    while tile_lo < m {
        let tile_hi = (tile_lo + tile).min(m);
        let tile_hi_u32 = tile_hi as u32;
        let tile_epoch = (tile_lo >> 16) as u16;
        let tile_hi_lo16 = if (tile_hi & 0xFFFF) == 0 {
            65_536u32
        } else {
            (tile_hi & 0xFFFF) as u32
        };
        let tile_lo_16 = (tile_lo & 0xFFFF) as u32;
        let dst_tile = &mut edges[tile_lo..tile_hi];
        let mark_here = mark >= tile_lo as u32 && mark < tile_hi_u32;

        for cur in &mut active[..n_act] {
            if cur.peek_v >= tile_hi_u32 {
                continue;
            }
            let check_mark = mark_here && cur.a == mb;
            let mark_lo16 = mark & 0xFFFF;
            loop {
                if cur.epoch > tile_epoch || cur.peek_v >= tile_hi_u32 {
                    break;
                }
                while cur.read == cur.cut {
                    cur.epoch += 1;
                    cur.cut = spill
                        .cuts
                        .get(cur.epoch as usize)
                        .map_or(u32::MAX, |ct| ct[cur.a as usize]);
                }
                if cur.epoch > tile_epoch {
                    cur.refresh_peek(out, spill, in_out);
                    break;
                }
                if cur.at as usize == CHUNK {
                    cur.id = spill.next[cur.id as usize];
                    cur.at = 0;
                }
                let avail = (CHUNK - cur.at as usize)
                    .min((cur.cut - cur.read) as usize)
                    .min(cur.u_left as usize);
                let chunk: &[u8] = if cur.id < in_out {
                    let base = cur.id as usize * BYTES;
                    &out[base..base + BYTES]
                } else {
                    &spill.ovf[(cur.id - in_out) as usize][..]
                };
                let bytes = &chunk[2 * cur.at as usize..2 * (cur.at as usize + avail)];
                let mut i = 0usize;
                let mut u = cur.u_cur;

                if !check_mark {
                    // Monotonicity fast-path: checking v7 < tile_hi_lo16 guarantees v0..v7 < tile_hi_lo16!
                    while i + 8 <= avail {
                        let b = &bytes[2 * i..2 * i + 16];
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let v7 = (q1 >> 48) as u32;
                        if v7 >= tile_hi_lo16 {
                            break;
                        }
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let v0 = (q0 as u32 & 0xFFFF) - tile_lo_16;
                        let v1 = ((q0 >> 16) as u32 & 0xFFFF) - tile_lo_16;
                        let v2 = ((q0 >> 32) as u32 & 0xFFFF) - tile_lo_16;
                        let v3 = ((q0 >> 48) as u32) - tile_lo_16;
                        let v4 = (q1 as u32 & 0xFFFF) - tile_lo_16;
                        let v5 = ((q1 >> 16) as u32 & 0xFFFF) - tile_lo_16;
                        let v6 = ((q1 >> 32) as u32 & 0xFFFF) - tile_lo_16;
                        let v7s = v7 - tile_lo_16;
                        dst_tile[v0 as usize] ^= u;
                        dst_tile[v1 as usize] ^= u + 1;
                        dst_tile[v2 as usize] ^= u + 2;
                        dst_tile[v3 as usize] ^= u + 3;
                        dst_tile[v4 as usize] ^= u + 4;
                        dst_tile[v5 as usize] ^= u + 5;
                        dst_tile[v6 as usize] ^= u + 6;
                        dst_tile[v7s as usize] ^= u + 7;
                        u += 8;
                        i += 8;
                    }
                    let mut hit_tile_end = false;
                    while i < avail {
                        let lo = u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]) as u32;
                        if lo >= tile_hi_lo16 {
                            hit_tile_end = true;
                            cur.peek_v = (u32::from(cur.epoch) << 16) | lo;
                            break;
                        }
                        dst_tile[(lo - tile_lo_16) as usize] ^= u;
                        u += 1;
                        i += 1;
                    }
                    cur.at += i as u16;
                    cur.read += i as u32;
                    cur.u_cur = u;
                    cur.u_left -= i as u32;
                    if hit_tile_end {
                        break;
                    }
                } else {
                    let mut hit_tile_end = false;
                    while i < avail {
                        let lo = u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]) as u32;
                        if lo >= tile_hi_lo16 {
                            hit_tile_end = true;
                            cur.peek_v = (u32::from(cur.epoch) << 16) | lo;
                            break;
                        }
                        if lo == mark_lo16 {
                            predecessor = u;
                        }
                        dst_tile[(lo - tile_lo_16) as usize] ^= u;
                        u += 1;
                        i += 1;
                    }
                    cur.at += i as u16;
                    cur.read += i as u32;
                    cur.u_cur = u;
                    cur.u_left -= i as u32;
                    if hit_tile_end {
                        break;
                    }
                }

                if cur.u_left == 0 {
                    if !cur.advance_band(bands, 0) {
                        break;
                    }
                }
                cur.refresh_peek(out, spill, in_out);
            }
        }
        tile_lo = tile_hi;
    }
    predecessor
}

/// Single-pass `v`-tiled `OutQ16` sweep (with `edges.fill(0)`): reads `OutQ16` ONCE,
/// keeping `edges[v]` inside the `tile`-sized L1D/L2 slice.
#[inline(never)]
fn single_pass_tile_v(
    edges: &mut [u32],
    bands: &[Band],
    cursors: &[usize; 256],
    out: &[u8],
    spill: &Spill,
    heads: &Heads,
    m: usize,
    bits: u32,
    tile: usize,
    mark: u32,
    mb: u8,
) -> u32 {
    edges.fill(0);
    let mut active = [TileCursor {
        a: 0,
        id: NONE,
        at: 0,
        read: 0,
        epoch: 0,
        cut: u32::MAX,
        band_idx: 0,
        band_end: 0,
        u_cur: 0,
        u_left: 0,
        pack: 0,
        peek_v: u32::MAX,
    }; 256];
    let mut n_act = 0usize;
    for a in 0..256 {
        let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
        if let Some(c) = TileCursor::init(a, bands, cursors[a], end, out, spill, heads, bits) {
            active[n_act] = c;
            n_act += 1;
        }
    }
    let in_out = heads.in_out;
    let mut predecessor = mark;
    let mut tile_lo = 0usize;
    while tile_lo < m {
        let tile_hi = (tile_lo + tile).min(m);
        let tile_hi_u32 = tile_hi as u32;
        let tile_epoch = (tile_lo >> 16) as u16;
        let tile_hi_lo16 = if (tile_hi & 0xFFFF) == 0 {
            65_536u32
        } else {
            (tile_hi & 0xFFFF) as u32
        };
        let hi_base = u32::from(tile_epoch) << 16;
        let mark_here = mark >= tile_lo as u32 && mark < tile_hi_u32;

        for cur in &mut active[..n_act] {
            if cur.peek_v >= tile_hi_u32 {
                continue;
            }
            let check_mark = mark_here && cur.a == mb;
            let mark_lo16 = mark & 0xFFFF;
            loop {
                if cur.epoch > tile_epoch || cur.peek_v >= tile_hi_u32 {
                    break;
                }
                while cur.read == cur.cut {
                    cur.epoch += 1;
                    cur.cut = spill
                        .cuts
                        .get(cur.epoch as usize)
                        .map_or(u32::MAX, |ct| ct[cur.a as usize]);
                }
                if cur.epoch > tile_epoch {
                    cur.refresh_peek(out, spill, in_out);
                    break;
                }
                if cur.at as usize == CHUNK {
                    cur.id = spill.next[cur.id as usize];
                    cur.at = 0;
                }
                let avail = (CHUNK - cur.at as usize)
                    .min((cur.cut - cur.read) as usize)
                    .min(cur.u_left as usize);
                let chunk: &[u8] = if cur.id < in_out {
                    let base = cur.id as usize * BYTES;
                    &out[base..base + BYTES]
                } else {
                    &spill.ovf[(cur.id - in_out) as usize][..]
                };
                let bytes = &chunk[2 * cur.at as usize..2 * (cur.at as usize + avail)];
                let pack = cur.pack;
                let mut i = 0usize;
                let mut u = cur.u_cur;
                let mut hit_tile_end = false;
                while i < avail {
                    let lo = u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]) as u32;
                    if lo >= tile_hi_lo16 {
                        hit_tile_end = true;
                        cur.peek_v = hi_base | lo;
                        break;
                    }
                    let v = hi_base | lo;
                    if check_mark && lo == mark_lo16 {
                        predecessor = u;
                    }
                    edges[u as usize] ^= pack | v;
                    edges[v as usize] ^= u;
                    u += 1;
                    i += 1;
                }
                cur.at += i as u16;
                cur.read += i as u32;
                cur.u_cur = u;
                cur.u_left -= i as u32;
                if hit_tile_end {
                    break;
                }
                if cur.u_left == 0 {
                    if !cur.advance_band(bands, bits) {
                        break;
                    }
                }
                cur.refresh_peek(out, spill, in_out);
            }
        }
        tile_lo = tile_hi;
    }
    predecessor
}

/// Zero-Skip L1D software write-combining buffer scatter (32 buckets x 32 pairs = 8 KiB on stack).
/// Pass 1 writes `edges[u] = pack | v` cleanly (no `edges.fill(0)`), then Pass 2 scans `OutQ16`
/// buffering `(v, u)` into 32 tile-aligned buckets (each covering `8192` edges = 32 KiB) and
/// flushing 32 pairs at a time into `edges[v] ^= u`.
#[inline(never)]
fn l1_buf_scatter_pass(
    edges: &mut [u32],
    bands: &[Band],
    cursors: &[usize; 256],
    out: &[u8],
    spill: &Spill,
    heads: &Heads,
    m: usize,
    mark: u32,
    mb: usize,
) -> u32 {
    const NBUCK: usize = 32;
    const BSHIFT: u32 = 13; // 8192 entries per bucket (32 KiB)
    const BSIZE: usize = 32;
    let _ = m;
    let mut bufs = [[(0u32, 0u32); BSIZE]; NBUCK];
    let mut lens = [0usize; NBUCK];
    let mut predecessor = mark;

    for a in 0..256 {
        let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
        let mut q = Reader::new(out, spill, heads, a);
        for &band in &bands[cursors[a]..end] {
            let mut u = band.dst();
            let mut left = band.end() as usize;
            while left > 0 {
                while q.read == q.cut {
                    q.epoch += 1;
                    q.cut = spill
                        .cuts
                        .get(q.epoch as usize)
                        .map_or(u32::MAX, |c| c[a]);
                }
                if q.at == CHUNK {
                    q.id = spill.next[q.id as usize];
                    q.at = 0;
                }
                let take = left
                    .min(CHUNK - q.at)
                    .min((q.cut - q.read) as usize);
                let chunk: &[u8] = if q.id < heads.in_out {
                    let at = q.id as usize * BYTES;
                    &out[at..at + BYTES]
                } else {
                    &spill.ovf[(q.id - heads.in_out) as usize][..]
                };
                let hi = q.epoch << 16;
                let check_mark = a == mb;
                for s in chunk[2 * q.at..2 * (q.at + take)].chunks_exact(2) {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if check_mark && v == mark {
                        predecessor = u;
                    }
                    let b = ((v >> BSHIFT) as usize).min(NBUCK - 1);
                    let len = lens[b];
                    bufs[b][len] = (v, u);
                    if len + 1 == BSIZE {
                        for &(vv, uu) in &bufs[b] {
                            edges[vv as usize] ^= uu;
                        }
                        lens[b] = 0;
                    } else {
                        lens[b] = len + 1;
                    }
                    u += 1;
                }
                q.at += take;
                q.read += take as u32;
                left -= take;
            }
        }
    }
    for b in 0..NBUCK {
        for &(vv, uu) in &bufs[b][..lens[b]] {
            edges[vv as usize] ^= uu;
        }
    }
    predecessor
}

/// Zero-Skip untiled 2-pass: Pass 1 already wrote `edges[u] = pack | v`; Pass 2 does untiled `edges[v] ^= u`.
#[inline(never)]
fn untiled_rev_pass(
    edges: &mut [u32],
    bands: &[Band],
    cursors: &[usize; 256],
    out: &[u8],
    spill: &Spill,
    heads: &Heads,
    mark: u32,
    mb: usize,
) -> u32 {
    let mut predecessor = mark;
    for a in 0..256 {
        let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
        let mut q = Reader::new(out, spill, heads, a);
        for &band in &bands[cursors[a]..end] {
            let mut u = band.dst();
            let mut left = band.end() as usize;
            while left > 0 {
                while q.read == q.cut {
                    q.epoch += 1;
                    q.cut = spill
                        .cuts
                        .get(q.epoch as usize)
                        .map_or(u32::MAX, |c| c[a]);
                }
                if q.at == CHUNK {
                    q.id = spill.next[q.id as usize];
                    q.at = 0;
                }
                let take = left
                    .min(CHUNK - q.at)
                    .min((q.cut - q.read) as usize);
                let chunk: &[u8] = if q.id < heads.in_out {
                    let at = q.id as usize * BYTES;
                    &out[at..at + BYTES]
                } else {
                    &spill.ovf[(q.id - heads.in_out) as usize][..]
                };
                let hi = q.epoch << 16;
                let bytes = &chunk[2 * q.at..2 * (q.at + take)];
                if a != mb {
                    let mut i = 0usize;
                    while i + 8 <= take {
                        let b = &bytes[2 * i..2 * i + 16];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        edges[(hi | (q0 as u32 & 0xFFFF)) as usize] ^= u;
                        edges[(hi | ((q0 >> 16) as u32 & 0xFFFF)) as usize] ^= u + 1;
                        edges[(hi | ((q0 >> 32) as u32 & 0xFFFF)) as usize] ^= u + 2;
                        edges[(hi | ((q0 >> 48) as u32)) as usize] ^= u + 3;
                        edges[(hi | (q1 as u32 & 0xFFFF)) as usize] ^= u + 4;
                        edges[(hi | ((q1 >> 16) as u32 & 0xFFFF)) as usize] ^= u + 5;
                        edges[(hi | ((q1 >> 32) as u32 & 0xFFFF)) as usize] ^= u + 6;
                        edges[(hi | ((q1 >> 48) as u32)) as usize] ^= u + 7;
                        u += 8;
                        i += 8;
                    }
                    while i < take {
                        let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                        edges[v as usize] ^= u;
                        u += 1;
                        i += 1;
                    }
                } else {
                    for s in bytes.chunks_exact(2) {
                        let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                        if v == mark {
                            predecessor = u;
                        }
                        edges[v as usize] ^= u;
                        u += 1;
                    }
                }
                q.at += take;
                q.read += take as u32;
                left -= take;
            }
        }
    }
    predecessor
}

impl Kernel {
    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32, cap_strips: usize) -> Pass<'a> {
        Pass {
            queues: Queues::new(out, &mut self.spill, limit),
            edges: &mut self.edges,
            ns: 0,
            cap_strips,
            f: First::default(),
        }
    }

    /// Run with explicit `mode` and `bi_min_m` (production uses `65_536`; tests use `2` to force all modes on small cases!).
    #[inline(never)]
    pub fn run_mode(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        mode: S4Mode,
        bi_min_m: u32,
        out: &mut Vec<u8>,
    ) -> (Counts, StageTimes) {
        let t0 = std::time::Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], u32::MAX, usize::MAX);
        {
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
                    pass.take(c, pos, width, p, LIMIT);
                    pos += width;
                }
            }
        }
        let ns = pass.ns;
        let mut f = pass.f;
        f.thin = pass.queues.extract_thin();
        let heads = pass.queues.heads();
        assert!(!pass.queues.over());

        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
            mark_byte,
            runs,
        } = f;
        let mut stats = Counts {
            n,
            runs,
            retained: m as usize,
            thick: ns,
            ..Counts::default()
        };
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(thick);
        let need = self.arena_words(ns, m as usize);
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
        let cursors = self.induced(ns, n as u32, &thin, &first, &mut write, &mut stats);
        stats.bands = self.bands.len();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        let t1 = std::time::Instant::now();

        // === S4: Wiring ===
        self.predecessor = None;
        let use_bi = !wide && m >= bi_min_m;
        match mode {
            S4Mode::HeadVerbatim if use_bi => {
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut predecessor = mark;
                let mb = usize::from(mark_byte);
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
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
            }
            S4Mode::HeadEpoch16x if use_bi => {
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut predecessor = mark;
                let mb = usize::from(mark_byte);
                let mark_epoch = mark >> 16;
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
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
                            let span = band.end() as usize;
                            if queue.epoch != mark_epoch && queue.read + span as u32 <= queue.cut {
                                queue.wire_xor_nomark(
                                    edges,
                                    band.dst() as usize,
                                    span,
                                    band.code() << bits,
                                );
                            } else {
                                queue.wire_xor(
                                    edges,
                                    band.dst() as usize,
                                    span,
                                    band.code() << bits,
                                    mark,
                                    &mut predecessor,
                                );
                            }
                        }
                    }
                }
                self.predecessor = Some(predecessor);
            }
            S4Mode::TileRev(tile) if use_bi => {
                // Pass 1: pure forward wire (`edges[u] = pack | v`, ZERO `edges.fill(0)`)
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        queue.wire(&mut self.edges[dst..dst + span], b.code() << bits);
                    }
                }
                // Pass 2: Galois/Frobenius L1D/L2-tiled reverse sweep `edges[v] ^= u`
                let edges = &mut self.edges[..m as usize];
                let pred = tile_rev_pass(
                    edges,
                    &self.bands,
                    &cursors,
                    &out[..],
                    &self.spill,
                    &heads,
                    m as usize,
                    tile,
                    mark,
                    mark_byte,
                );
                self.predecessor = Some(pred);
            }
            S4Mode::SinglePassTileV(tile) if use_bi => {
                let edges = &mut self.edges[..m as usize];
                let pred = single_pass_tile_v(
                    edges,
                    &self.bands,
                    &cursors,
                    &out[..],
                    &self.spill,
                    &heads,
                    m as usize,
                    bits,
                    tile,
                    mark,
                    mark_byte,
                );
                self.predecessor = Some(pred);
            }
            S4Mode::L1BufScatter if use_bi => {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        queue.wire(&mut self.edges[dst..dst + span], b.code() << bits);
                    }
                }
                let edges = &mut self.edges[..m as usize];
                let pred = l1_buf_scatter_pass(
                    edges,
                    &self.bands,
                    &cursors,
                    &out[..],
                    &self.spill,
                    &heads,
                    m as usize,
                    mark,
                    usize::from(mark_byte),
                );
                self.predecessor = Some(pred);
            }
            S4Mode::ZeroSkipUntiled if use_bi => {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        queue.wire(&mut self.edges[dst..dst + span], b.code() << bits);
                    }
                }
                let edges = &mut self.edges[..m as usize];
                let pred = untiled_rev_pass(
                    edges,
                    &self.bands,
                    &cursors,
                    &out[..],
                    &self.spill,
                    &heads,
                    mark,
                    usize::from(mark_byte),
                );
                self.predecessor = Some(pred);
            }
            _ => {
                // NobiVerbatim, DualFwd, QuadFwd, or fallback when wide / m < bi_min_m
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = Reader::new(&out[..], &self.spill, &heads, a);
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
        }
        let t2 = std::time::Instant::now();

        // === S5: Emit ===
        if let Some(before) = self.predecessor {
            emit_bidirectional(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                before,
                bits,
                out,
            );
        } else if mode == S4Mode::DualFwd && !wide && m >= bi_min_m && m >= 2 {
            emit_dualfwd(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
                &mut self.stage1,
            );
        } else if mode == S4Mode::QuadFwd && !wide && m >= bi_min_m && m >= 4 {
            emit_quadfwd(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
                &mut self.stage1,
                &mut self.stage2,
                &mut self.stage3,
            );
        } else {
            emit(
                &self.edges[..m as usize],
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
        let t3 = std::time::Instant::now();

        (
            stats,
            StageTimes {
                s1_s3_us: t1.duration_since(t0).as_secs_f64() * 1e6,
                s4_us: t2.duration_since(t1).as_secs_f64() * 1e6,
                s5_us: t3.duration_since(t2).as_secs_f64() * 1e6,
            },
        )
    }

    /// Diagnostic helper: inspect band `dst` order and `edges.fill(0)` alone cost.
    pub fn scratch_bytes(&self) -> usize {
        use std::mem::size_of;
        self.spill.scratch_bytes()
            + size_of::<Band>() * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * self.edges.capacity()
            + 4 * self.extra.capacity()
            + 16 * self.inline16.capacity()
            + self.lens.capacity()
            + self.endpoint.scratch_bytes()
            + self.stage1.capacity()
            + self.stage2.capacity()
            + self.stage3.capacity()
    }

    pub fn band_locality_stats(&self) -> (usize, usize, usize) {
        let b = self.bands.len();
        let mut mono_pairs = 0usize;
        let mut cl_aligned = 0usize;
        for w in self.bands.windows(2) {
            if w[1].dst() >= w[0].dst() + w[0].end() {
                mono_pairs += 1;
            }
        }
        for band in &self.bands {
            if (band.dst() & 15) == 0 {
                cl_aligned += 1;
            }
        }
        (b, mono_pairs, cl_aligned)
    }

    /// Run `Nobi` wiring to inspect pure forward `edges[0..m)` and measure:
    /// (1) Dihedral involution failure rate (`rho(j) = m-1-j` conjugating `pi` to `pi^{-1}`),
    /// (2) Whitworth (1897) empirical max-arc ratio for `K=2` (theory 0.7500) and `K=4` (theory 0.5208).
    pub fn whitworth_and_dihedral_stats(&mut self, records: &[u16], n: usize, p: u32) -> (f64, f64, f64) {
        let mut out = Vec::new();
        let (stats, _) = self.run_mode(records, n, p, S4Mode::NobiVerbatim, 65_536, &mut out);
        let m = stats.retained;
        let bits = (u32::BITS - (m as u32 - 1).leading_zeros()).max(1);
        let mask = (1u32 << bits) - 1;
        let edges = &self.edges[..m];
        // Dihedral test: does `rho(j) = m - 1 - j` satisfy `rho(pi(rho(pi(j)))) == j`?
        let mut dih_fail = 0usize;
        for j in 0..m {
            let pj = (edges[j] & mask) as usize;
            let rpj = m - 1 - pj;
            let prpj = (edges[rpj] & mask) as usize;
            let rprpj = m - 1 - prpj;
            if rprpj != j {
                dih_fail += 1;
            }
        }
        let dih_fail_pct = 100.0 * dih_fail as f64 / m as f64;
        // Walk full cycle from mark=0 (or actual mark) to record positions of `q = (0 + k*m/K) % m`
        // Find mark by scanning? Any vertex on the single cycle gives exact arc lengths for equidistant rank anchors!
        let mut pos = vec![usize::MAX; m];
        let mut cur = 0usize;
        let mut cyc = 0usize;
        while pos[cur] == usize::MAX {
            pos[cur] = cyc;
            cur = (edges[cur] & mask) as usize;
            cyc += 1;
        }
        let a2 = [0usize, m / 2];
        let mut p2: Vec<usize> = a2.iter().filter_map(|&x| (pos[x] != usize::MAX).then_some(pos[x])).collect();
        p2.sort_unstable();
        let max_arc2 = if p2.len() == 2 {
            (p2[1] - p2[0]).max(cyc - (p2[1] - p2[0])) as f64 / cyc as f64
        } else {
            1.0
        };
        let a4 = [0usize, m / 4, m / 2, (3 * m) / 4];
        let mut p4: Vec<usize> = a4.iter().filter_map(|&x| (pos[x] != usize::MAX).then_some(pos[x])).collect();
        p4.sort_unstable();
        let max_arc4 = if p4.len() == 4 {
            let mut mx = cyc - (p4[3] - p4[0]);
            for w in p4.windows(2) {
                mx = mx.max(w[1] - w[0]);
            }
            mx as f64 / cyc as f64
        } else {
            1.0
        };
        (dih_fail_pct, max_arc2, max_arc4)
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

/// Non-XOR 2-cursor Dual-Forward walk (`K=2`, zero reverse scatter!).
/// Exact on all primitive, periodic, and malformed multi-cycle columns (matches forward `FL`).
#[allow(clippy::too_many_arguments)]
fn emit_dualfwd(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
    stage1: &mut Vec<u8>,
) {
    let m = edges.len();
    out.resize(n + INLINE, 0);
    if stage1.len() < n + INLINE {
        let need = (n + INLINE) - stage1.len();
        stage1.reserve_exact(need);
        stage1.resize(n + INLINE, 0);
    }
    let mask = (1 << bits) - 1;
    let a0 = mark;
    let a1 = ((mark as usize + m / 2) % m) as u32;
    let (mut j0, mut j1) = (a0, a1);
    let (mut at0, mut at1) = (0usize, 0usize);
    let (mut hit0, mut hit1) = (u8::MAX, u8::MAX);
    let (mut act0, mut act1) = (true, true);

    // Phase 1: lockstep 2-way MLP while both cursors are active
    for _ in 0..m {
        if !(act0 && act1) {
            break;
        }
        let e0 = edges[j0 as usize];
        let e1 = edges[j1 as usize];
        let c0 = (e0 >> bits) as usize;
        let c1 = (e1 >> bits) as usize;
        j0 = e0 & mask;
        j1 = e1 & mask;

        let l0 = lens[c0];
        if l0 != LONG {
            out[at0..at0 + INLINE].copy_from_slice(&inline[c0]);
            at0 += usize::from(l0);
        } else {
            let w = long_word(&inline[c0], spelling);
            let take = w.len().min(n - at0);
            out[at0..at0 + take].copy_from_slice(&w[..take]);
            at0 += take;
        }
        let l1 = lens[c1];
        if l1 != LONG {
            stage1[at1..at1 + INLINE].copy_from_slice(&inline[c1]);
            at1 += usize::from(l1);
        } else {
            let w = long_word(&inline[c1], spelling);
            let take = w.len().min(n - at1);
            stage1[at1..at1 + take].copy_from_slice(&w[..take]);
            at1 += take;
        }

        if j0 == a0 {
            hit0 = 0;
            act0 = false;
        } else if j0 == a1 {
            hit0 = 1;
            act0 = false;
        } else if at0 >= n {
            act0 = false;
        }
        if j1 == a0 {
            hit1 = 0;
            act1 = false;
        } else if j1 == a1 {
            hit1 = 1;
            act1 = false;
        } else if at1 >= n {
            act1 = false;
        }
    }

    // Phase 2: finish whichever single cursor is still needed!
    // If cursor 0 already returned to `a0` (disjoint cycle) or hit `n`, cursor 1 is unneeded!
    if act0 {
        for _ in 0..m {
            let e0 = edges[j0 as usize];
            let c0 = (e0 >> bits) as usize;
            j0 = e0 & mask;
            let l0 = lens[c0];
            if l0 != LONG {
                out[at0..at0 + INLINE].copy_from_slice(&inline[c0]);
                at0 += usize::from(l0);
            } else {
                let w = long_word(&inline[c0], spelling);
                let take = w.len().min(n - at0);
                out[at0..at0 + take].copy_from_slice(&w[..take]);
                at0 += take;
            }
            if j0 == a0 {
                hit0 = 0;
                break;
            } else if j0 == a1 {
                hit0 = 1;
                break;
            } else if at0 >= n {
                break;
            }
        }
    } else if hit0 == 1 && act1 && at0 < n {
        for _ in 0..m {
            let e1 = edges[j1 as usize];
            let c1 = (e1 >> bits) as usize;
            j1 = e1 & mask;
            let l1 = lens[c1];
            if l1 != LONG {
                stage1[at1..at1 + INLINE].copy_from_slice(&inline[c1]);
                at1 += usize::from(l1);
            } else {
                let w = long_word(&inline[c1], spelling);
                let take = w.len().min(n - at1);
                stage1[at1..at1 + take].copy_from_slice(&w[..take]);
                at1 += take;
            }
            if j1 == a0 {
                hit1 = 0;
                break;
            } else if j1 == a1 {
                hit1 = 1;
                break;
            } else if at0 + at1 >= n {
                break;
            }
        }
    }

    // Stitch
    let mut lo = at0.min(n);
    if hit0 == 1 && lo < n {
        let take = at1.min(n - lo);
        out[lo..lo + take].copy_from_slice(&stage1[..take]);
        lo += take;
        let _ = hit1;
    }
    out.truncate(lo);
    repeat(n, out);
}

/// Non-XOR 4-cursor Quad-Forward walk (`K=4`, Whitworth 1897 order-statistic chord stitching).
/// Zero reverse scatter; exact on all primitive, periodic, and malformed multi-cycle columns.
#[allow(clippy::too_many_arguments)]
fn emit_quadfwd(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
    stage1: &mut Vec<u8>,
    stage2: &mut Vec<u8>,
    stage3: &mut Vec<u8>,
) {
    let m = edges.len();
    out.resize(n + INLINE, 0);
    for st in [&mut *stage1, &mut *stage2, &mut *stage3] {
        if st.len() < n + INLINE {
            let need = (n + INLINE) - st.len();
            st.reserve_exact(need);
            st.resize(n + INLINE, 0);
        }
    }
    let mask = (1 << bits) - 1;
    let a = [
        mark,
        ((mark as usize + m / 4) % m) as u32,
        ((mark as usize + m / 2) % m) as u32,
        ((mark as usize + (3 * m) / 4) % m) as u32,
    ];
    let mut j = a;
    let mut at = [0usize; 4];
    let mut hit = [u8::MAX; 4];
    let mut act = [true; 4];
    let mut n_act = 4usize;

    #[inline(always)]
    fn step_one(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        bits: u32,
        mask: u32,
        a: &[u32; 4],
        cur: &mut u32,
        pos: &mut usize,
        hit_slot: &mut u8,
        active: &mut bool,
        n_act: &mut usize,
        buf: &mut [u8],
    ) {
        let e = edges[*cur as usize];
        let c = (e >> bits) as usize;
        let next = e & mask;
        *cur = next;
        let l = lens[c];
        if l != LONG {
            buf[*pos..*pos + INLINE].copy_from_slice(&inline[c]);
            *pos += usize::from(l);
        } else {
            let w = long_word(&inline[c], spelling);
            let take = w.len().min(n - *pos);
            buf[*pos..*pos + take].copy_from_slice(&w[..take]);
            *pos += take;
        }
        if next == a[0] {
            *hit_slot = 0;
            *active = false;
            *n_act -= 1;
        } else if next == a[1] {
            *hit_slot = 1;
            *active = false;
            *n_act -= 1;
        } else if next == a[2] {
            *hit_slot = 2;
            *active = false;
            *n_act -= 1;
        } else if next == a[3] {
            *hit_slot = 3;
            *active = false;
            *n_act -= 1;
        } else if *pos >= n {
            *active = false;
            *n_act -= 1;
        }
    }

    for _ in 0..m {
        if n_act == 0 {
            break;
        }
        if act[0] {
            step_one(
                edges, spelling, inline, lens, n, bits, mask, &a,
                &mut j[0], &mut at[0], &mut hit[0], &mut act[0], &mut n_act, out,
            );
        }
        if act[1] {
            step_one(
                edges, spelling, inline, lens, n, bits, mask, &a,
                &mut j[1], &mut at[1], &mut hit[1], &mut act[1], &mut n_act, stage1,
            );
        }
        if act[2] {
            step_one(
                edges, spelling, inline, lens, n, bits, mask, &a,
                &mut j[2], &mut at[2], &mut hit[2], &mut act[2], &mut n_act, stage2,
            );
        }
        if act[3] {
            step_one(
                edges, spelling, inline, lens, n, bits, mask, &a,
                &mut j[3], &mut at[3], &mut hit[3], &mut act[3], &mut n_act, stage3,
            );
        }
        // Early stop if anchor 0 already closed its cycle back to 0 without needing remaining active cursors
        if !act[0] && hit[0] == 0 {
            break;
        }
    }

    // Stitch along chain starting at 0
    let mut lo = at[0].min(n);
    let mut cur_seg = 0usize;
    let mut seen = 1u8;
    while lo < n {
        let nxt = hit[cur_seg];
        if nxt == 0 || nxt >= 4 || (seen & (1 << nxt)) != 0 {
            break;
        }
        seen |= 1 << nxt;
        let src: &[u8] = match nxt {
            1 => &stage1[..at[1]],
            2 => &stage2[..at[2]],
            3 => &stage3[..at[3]],
            _ => unreachable!(),
        };
        let take = src.len().min(n - lo);
        out[lo..lo + take].copy_from_slice(&src[..take]);
        lo += take;
        cur_seg = nxt as usize;
    }
    out.truncate(lo);
    repeat(n, out);
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}
