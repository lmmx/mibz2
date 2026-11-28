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

/// Ranks a chunk holds on the shipped path (and its power-of-2 chunk stride)
const CHUNK: usize = 512;
/// Usable ranks per chunk on the `mtf-fold` path: leaves 16 guard ranks (32 B) at the end of each
/// `2 * FOLD_STRIDE` byte chunk so any `w <= 16` thin run with `f + w <= FOLD_CHUNK` writes 32 B
/// safely inside the chunk without hitting `push_slow` (0 overflow chunks on Wikidata)
#[cfg(feature = "mtf-fold")]
pub(super) const FOLD_CHUNK: usize = 1008;
/// Power-of-2 chunk stride (ranks) on the `mtf-fold` path
#[cfg(feature = "mtf-fold")]
pub(super) const FOLD_STRIDE: usize = 1024;
/// No chunk
const NONE: u32 = u32::MAX;
/// Ranks an epoch holds
const EPOCH: u32 = 1 << 16;
/// High bit of `fill[a]` marking an overflow (or not-yet-allocated) chunk when `HAS_GUARD` is
/// enabled: folds `tail < in_out` into the single `f.wrapping_add(w) <= C` check on the fast path
const OVF_BIT: u32 = 1 << 31;

/// The queues' state kept between blocks: the chunks' links, the overflow chunks and the
/// epochs' cuts (on the admitted blocks, a few KiB in all)
#[derive(Default)]
pub(super) struct Spill {
    next: Vec<u32>,
    ovf: Vec<u8>,
    cuts: Vec<[u32; 256]>,
    /// (test only) the chunks the last block's first pass took
    #[cfg(test)]
    pub(super) taken: u32,
    /// (test only) overflow chunks taken by the last block
    #[cfg(test)]
    ovf_chunks: usize,
}

impl Spill {
    /// (test only) the scratch it keeps, in bytes
    #[cfg(test)]
    pub(super) fn scratch(&self) -> usize {
        4 * self.next.capacity() + self.ovf.capacity() + 1024 * self.cuts.capacity()
    }

    /// (test only) the overflow chunks the last block took
    #[cfg(test)]
    pub(super) fn overflow(&self) -> usize {
        self.ovf_chunks
    }
}

/// The queues while the first pass writes them (`C` usable ranks per `2 * STRIDE` byte chunk;
/// `512/512` on the shipped path, `C/STRIDE` with a 16-rank guard zone under `mtf-fold`)
pub(super) struct QueuesOf<'a, const C: usize, const STRIDE: usize = CHUNK> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    /// chunks `out` holds
    in_out: u32,
    /// chunks taken
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    /// ranks in each byte's last chunk (`C` before its first; bit 31 set on overflow chunks when
    /// `HAS_GUARD` holds)
    fill: [u32; 256],
    /// the retained ranks the block may have (`Cap::m`)
    limit: u32,
    /// (sticky) a push went past `limit`: the block is refused, and no chunk is taken after
    over: bool,
}

/// Shipped queues (`CHUNK = 512` usable ranks per 1024 B chunk)
pub(super) type Queues<'a> = QueuesOf<'a, CHUNK, CHUNK>;

/// Where the queues are, once written: each byte's first chunk
pub(super) struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a, const C: usize, const STRIDE: usize> QueuesOf<'a, C, STRIDE> {
    const BYTES: usize = 2 * STRIDE;
    const HAS_GUARD: bool = C + 16 <= STRIDE;

    /// Empty queues in `out`, whose bytes are free until the walk, for at most `limit` ranks
    pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        #[cfg(test)]
        {
            spill.ovf_chunks = 0;
        }
        let init_fill = if Self::HAS_GUARD {
            OVF_BIT | (C as u32)
        } else {
            C as u32
        };
        QueuesOf {
            in_out: (out.len() / Self::BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [init_fill; 256],
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
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        if Self::HAS_GUARD {
            let f = self.fill[a];
            let next_f = f.wrapping_add(w);
            let lo = m & (EPOCH - 1);
            if w <= 16 && next_f <= C as u32 && lo + w < EPOCH {
                let at = self.tails[a] as usize * Self::BYTES + 2 * (f as usize);
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = next_f;
                return;
            }
            self.push_slow(a, m, w, thin);
            return;
        }
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= C && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * Self::BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }

    /// Guard-zone `push` for thin runs (`1 <= w <= 16`) inside `mtf-fold`'s batch flush loop.
    /// Because `OVF_BIT` is set in `fill[a]` before the first chunk and on overflow chunks,
    /// `f.wrapping_add(w) <= C` proves `f < C < OVF_BIT`, `tails[a] < in_out`, and `f + 16 <= STRIDE`
    /// in a single unsigned compare. When `NO_EPOCH` is true, the caller proved the entire batch
    /// cannot reach an epoch boundary, eliminating the epoch check as well.
    #[cfg(feature = "mtf-fold")]
    #[inline(always)]
    pub(super) fn push_thin_fast<const NO_EPOCH: bool>(
        &mut self,
        a: usize,
        m: u32,
        w: u32,
        thin: &[u32; 256],
    ) {
        debug_assert!(Self::HAS_GUARD && w <= 16);
        let f = self.fill[a];
        let next_f = f.wrapping_add(w);
        let epoch_ok = NO_EPOCH || ((m & (EPOCH - 1)) + w < EPOCH);
        if next_f <= C as u32 && epoch_ok {
            let at = self.tails[a] as usize * Self::BYTES + 2 * (f as usize);
            let lo = m as u16;
            let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = next_f;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }

    /// `push` across a chunk's end or an epoch's, into an overflow chunk, or of more than 16
    /// ranks (a limit above 16, in the tests): rank by rank. Past the queues' `limit` it sets
    /// `over` and writes nothing: every chunk is taken here, so the queues take none once the
    /// block is refused, and `push` itself pays nothing for the limit (only a chunk's end
    /// brings it here, and its fast case writes within a chunk already taken).
    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            let f_cur = if Self::HAS_GUARD {
                (self.fill[a] & !OVF_BIT) as usize
            } else {
                self.fill[a] as usize
            };
            if f_cur == C {
                self.take_chunk(a);
            }
            let f = if Self::HAS_GUARD {
                (self.fill[a] & !OVF_BIT) as usize
            } else {
                self.fill[a] as usize
            };
            let lo = m & (EPOCH - 1);
            // Write at most 4 ranks (8 bytes) per outer step without an inner loop so
            // `-C target-cpu=native` never auto-vectorizes to 256-bit YMM or emits `vzeroupper`.
            let take = left.min((C - f) as u32).min(EPOCH - lo).min(4);
            let chunk = self.chunk_mut(self.tails[a]);
            let dst = &mut chunk[2 * f..][..2 * (take as usize)];
            let mut v = lo as u16;
            let mut it = dst.chunks_exact_mut(2);
            if let Some(d) = it.next() {
                d.copy_from_slice(&v.to_le_bytes());
                v = v.wrapping_add(1);
            }
            if let Some(d) = it.next() {
                d.copy_from_slice(&v.to_le_bytes());
                v = v.wrapping_add(1);
            }
            if let Some(d) = it.next() {
                d.copy_from_slice(&v.to_le_bytes());
                v = v.wrapping_add(1);
            }
            if let Some(d) = it.next() {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] += take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                // the epoch's end: every queue's length, this byte's with the ranks so far
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }

    /// A new last chunk for byte `a`
    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out {
            let need = ((id - self.in_out) as usize + 1) * Self::BYTES;
            if self.spill.ovf.len() < need {
                self.spill.ovf.resize(need, 0);
            }
            #[cfg(test)]
            {
                self.spill.ovf_chunks = self.spill.ovf_chunks.max((id - self.in_out) as usize + 1);
            }
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE);
        }
        self.spill.next[id as usize] = NONE;
        if self.heads[a] == NONE {
            self.heads[a] = id;
        } else {
            self.spill.next[self.tails[a] as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = if Self::HAS_GUARD && id >= self.in_out {
            OVF_BIT
        } else {
            0
        };
    }

    fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
        if id < self.in_out {
            let at = id as usize * Self::BYTES;
            &mut self.out[at..at + Self::BYTES]
        } else {
            let at = (id - self.in_out) as usize * Self::BYTES;
            &mut self.spill.ovf[at..at + Self::BYTES]
        }
    }
}

/// A byte's queue read in order: its chunk, the rank within it, the ranks read, and the
/// epoch with the queue length at which it ends
pub(super) struct ReaderOf<'a, const C: usize, const STRIDE: usize = CHUNK> {
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

/// Shipped reader (`CHUNK = 512` usable ranks per 1024 B chunk)
#[cfg(test)]
pub(super) type Reader<'a> = ReaderOf<'a, CHUNK, CHUNK>;

impl<'a, const C: usize, const STRIDE: usize> ReaderOf<'a, C, STRIDE> {
    const BYTES: usize = 2 * STRIDE;

    /// Byte `a`'s queue, from its first rank
    pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        ReaderOf {
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
    pub(super) fn wire_xor(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        predecessor: &mut u32,
    ) {
        if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * Self::BYTES + 2 * self.at;
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
            if self.at == C {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = left.min(C - self.at).min((self.cut - self.read) as usize);
            let chunk: &[u8] = if self.id < self.in_out {
                let at = self.id as usize * Self::BYTES;
                &self.out[at..at + Self::BYTES]
            } else {
                let at = (self.id - self.in_out) as usize * Self::BYTES;
                &self.spill.ovf[at..at + Self::BYTES]
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
        if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
            // the common case: one chunk of `out`, one epoch
            let hi = pack | (self.epoch << 16);
            let at = self.id as usize * Self::BYTES + 2 * self.at;
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
            if self.at == C {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = dst
                .len()
                .min(C - self.at)
                .min((self.cut - self.read) as usize);
            let chunk: &[u8] = if self.id < self.in_out {
                let at = self.id as usize * Self::BYTES;
                &self.out[at..at + Self::BYTES]
            } else {
                let at = (self.id - self.in_out) as usize * Self::BYTES;
                &self.spill.ovf[at..at + Self::BYTES]
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
