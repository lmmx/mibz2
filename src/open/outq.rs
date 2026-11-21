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
    /// chunks `out` holds
    in_out: u32,
    /// chunks taken
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    /// ranks in each byte's last chunk (`CHUNK` before its first)
    fill: [u32; 256],
    /// the retained ranks the block may have (`Cap::m`)
    limit: u32,
    /// (sticky) a push went past `limit`: the block is refused, and no chunk is taken after
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
        Queues {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
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
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
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
            if self.fill[a] as usize == CHUNK {
                self.take_chunk(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = self.chunk_mut(self.tails[a]);
            for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                .chunks_exact_mut(2)
                .enumerate()
            {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
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
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; BYTES]);
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
        self.fill[a] = 0;
    }

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

    /// The next `span` ranks, each with `pack` added, into `dst`
    #[inline(always)]
    pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            // the common case: one chunk of `out`, one epoch
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
