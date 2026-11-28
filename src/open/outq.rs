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
//! **Safety:** no `unsafe`; every index is bounds-checked (or statically proved via fixed-size
//! arrays and `std::cell::Cell`).

#![forbid(unsafe_code)]

use std::cell::Cell;

/// Ranks a chunk holds on the shipped path (and its power-of-2 chunk stride)
pub(super) const CHUNK: usize = 512;
/// Power-of-2 chunk stride (ranks) on the shipped path
pub(super) const STRIDE: usize = 512;
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

/// Shipped queues (`CHUNK` usable ranks per `2 * STRIDE` byte chunk)
pub(super) type Queues<'a> = QueuesOf<'a, CHUNK, STRIDE>;

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

/// Register-resident cursor over one byte's `OutQ16` queue.
///
/// `cur` holds the remaining little-endian `u16` bytes within the current chunk and current epoch.
/// Refilling takes values and returns a new `CursorState` by value so LLVM keeps `cur` and
/// `epoch` in registers across bands without spilling a `&mut Self` pointer to the stack.
#[derive(Clone, Copy)]
struct CursorState<'a> {
    cur: &'a [u8],
    id: u32,
    at_end: usize,
    read_end: u32,
    epoch: u32,
    cut: u32,
}

#[inline(always)]
fn cursor_init<'a, const C: usize, const STRIDE: usize>(
    out: &'a [u8],
    spill: &'a Spill,
    heads: &Heads,
    a: usize,
) -> CursorState<'a> {
    let bytes = 2 * STRIDE;
    let id = heads.heads[a];
    let cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
    if id == NONE {
        return CursorState {
            cur: &[],
            id,
            at_end: 0,
            read_end: 0,
            epoch: 0,
            cut,
        };
    }
    let mut epoch = 0u32;
    let mut cur_cut = cut;
    while cur_cut == 0 {
        epoch += 1;
        cur_cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
    }
    let take = C.min(cur_cut as usize);
    let chunk: &'a [u8] = if id < heads.in_out {
        let base = id as usize * bytes;
        &out[base..base + 2 * take]
    } else {
        let base = (id - heads.in_out) as usize * bytes;
        &spill.ovf[base..base + 2 * take]
    };
    CursorState {
        cur: chunk,
        id,
        at_end: take,
        read_end: take as u32,
        epoch,
        cut: cur_cut,
    }
}

/// Cold refill invoked only when `cur` is exhausted (at a chunk or epoch boundary).
#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn cursor_refill<'a, const C: usize, const STRIDE: usize>(
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    mut id: u32,
    mut at: usize,
    read: u32,
    mut epoch: u32,
    mut cut: u32,
) -> CursorState<'a> {
    let bytes = 2 * STRIDE;
    while read == cut {
        epoch += 1;
        cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
    }
    if at == C {
        id = spill.next[id as usize];
        at = 0;
    }
    let take = (C - at).min((cut - read) as usize);
    let chunk: &'a [u8] = if id < in_out {
        let base = id as usize * bytes + 2 * at;
        &out[base..base + 2 * take]
    } else {
        let base = (id - in_out) as usize * bytes + 2 * at;
        &spill.ovf[base..base + 2 * take]
    };
    CursorState {
        cur: chunk,
        id,
        at_end: at + take,
        read_end: read + take as u32,
        epoch,
        cut,
    }
}

#[inline(always)]
fn fill_fwd_8(dst: &mut [u32], src: &[u8], hi: u32) {
    let s: &[u8; 16] = src[..16].try_into().unwrap();
    let d: &mut [u32; 8] = (&mut dst[..8]).try_into().unwrap();
    for i in 0..8 {
        d[i] = hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
    }
}

#[inline(always)]
fn fill_fwd_4(dst: &mut [u32], src: &[u8], hi: u32) {
    let s: &[u8; 8] = src[..8].try_into().unwrap();
    let d: &mut [u32; 4] = (&mut dst[..4]).try_into().unwrap();
    for i in 0..4 {
        d[i] = hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
    }
}

/// Branch-minimized fixed-width `8/4/2/1` overlapping forward filler.
///
/// Eliminates LLVM's multi-tier `>= 32` / `% 32` / `% 8` / scalar-tail branch cascade across the
/// ~79% of Open bands with `span < 32`. Writes are idempotent, so overlapping the final 8-wide or
/// 4-wide vector handles any tail without a scalar loop.
#[inline(always)]
fn fill_fwd_overlap(dst: &mut [u32], src: &[u8], hi: u32) {
    let n = dst.len();
    if n >= 8 {
        let mut i = 0;
        let limit = n - 8;
        while i < limit {
            fill_fwd_8(&mut dst[i..i + 8], &src[2 * i..2 * i + 16], hi);
            i += 8;
        }
        fill_fwd_8(&mut dst[limit..n], &src[2 * limit..2 * n], hi);
    } else if n >= 4 {
        fill_fwd_4(&mut dst[0..4], &src[0..8], hi);
        let tail = n - 4;
        fill_fwd_4(&mut dst[tail..n], &src[2 * tail..2 * n], hi);
    } else if n >= 2 {
        let v0 = hi | u32::from(u16::from_le_bytes([src[0], src[1]]));
        let v1 = hi | u32::from(u16::from_le_bytes([src[2], src[3]]));
        dst[0] = v0;
        dst[1] = v1;
        if n == 3 {
            dst[2] = hi | u32::from(u16::from_le_bytes([src[4], src[5]]));
        }
    } else if n == 1 {
        dst[0] = hi | u32::from(u16::from_le_bytes([src[0], src[1]]));
    }
}

/// 4× unrolled zero-bounds-check XOR scatter for rows guaranteed not to hit `mark`.
///
/// Both `dst_cells` (`edges[dst..dst+len]`) and `ep` (`edges[epoch<<16..][..65536]`) borrow the
/// same `edges` buffer safely via `Cell<u32>`. Indexing `ep[lo as usize]` where `lo: u16` and
/// `ep: &[Cell<u32>; 65536]` proves every index in bounds statically, removing both per-element
/// bounds checks in safe Rust while preserving exact sequential semantics when `u == v`.
#[inline(always)]
fn cell_xor_4x(
    dst_cells: &[Cell<u32>],
    ep: &[Cell<u32>; 65536],
    src: &[u8],
    mut u: u32,
    hi_pack: u32,
) {
    let mut dchunks = dst_cells.chunks_exact(4);
    let mut schunks = src.chunks_exact(8);
    for (dc, sc) in (&mut dchunks).zip(&mut schunks) {
        let w = u64::from_le_bytes([sc[0], sc[1], sc[2], sc[3], sc[4], sc[5], sc[6], sc[7]]);
        let lo0 = (w & 0xFFFF) as u32;
        dc[0].set(dc[0].get() ^ (hi_pack | lo0));
        let e0 = &ep[lo0 as usize];
        e0.set(e0.get() ^ u);
        let lo1 = ((w >> 16) & 0xFFFF) as u32;
        dc[1].set(dc[1].get() ^ (hi_pack | lo1));
        let e1 = &ep[lo1 as usize];
        e1.set(e1.get() ^ (u + 1));
        let lo2 = ((w >> 32) & 0xFFFF) as u32;
        dc[2].set(dc[2].get() ^ (hi_pack | lo2));
        let e2 = &ep[lo2 as usize];
        e2.set(e2.get() ^ (u + 2));
        let lo3 = (w >> 48) as u32;
        dc[3].set(dc[3].get() ^ (hi_pack | lo3));
        let e3 = &ep[lo3 as usize];
        e3.set(e3.get() ^ (u + 3));
        u += 4;
    }
    for (dc, sc) in dchunks
        .remainder()
        .iter()
        .zip(schunks.remainder().chunks_exact(2))
    {
        let lo = u32::from(u16::from_le_bytes([sc[0], sc[1]]));
        dc.set(dc.get() ^ (hi_pack | lo));
        let e = &ep[lo as usize];
        e.set(e.get() ^ u);
        u += 1;
    }
}

/// 4× unrolled XOR scatter for the trailing partial epoch (`m & 0xFFFF != 0` when `edges` has no
/// slack past `m`). Outlined cold so the hot full-epoch loop keeps full register availability.
#[cold]
#[inline(never)]
fn cell_xor_4x_tail(
    dst_cells: &[Cell<u32>],
    ep: &[Cell<u32>],
    src: &[u8],
    mut u: u32,
    hi_pack: u32,
) {
    let mut dchunks = dst_cells.chunks_exact(4);
    let mut schunks = src.chunks_exact(8);
    for (dc, sc) in (&mut dchunks).zip(&mut schunks) {
        let w = u64::from_le_bytes([sc[0], sc[1], sc[2], sc[3], sc[4], sc[5], sc[6], sc[7]]);
        let lo0 = (w & 0xFFFF) as u32;
        let lo1 = ((w >> 16) & 0xFFFF) as u32;
        let lo2 = ((w >> 32) & 0xFFFF) as u32;
        let lo3 = (w >> 48) as u32;
        dc[0].set(dc[0].get() ^ (hi_pack | lo0));
        let e0 = &ep[lo0 as usize];
        e0.set(e0.get() ^ u);
        dc[1].set(dc[1].get() ^ (hi_pack | lo1));
        let e1 = &ep[lo1 as usize];
        e1.set(e1.get() ^ (u + 1));
        dc[2].set(dc[2].get() ^ (hi_pack | lo2));
        let e2 = &ep[lo2 as usize];
        e2.set(e2.get() ^ (u + 2));
        dc[3].set(dc[3].get() ^ (hi_pack | lo3));
        let e3 = &ep[lo3 as usize];
        e3.set(e3.get() ^ (u + 3));
        u += 4;
    }
    for (dc, sc) in dchunks
        .remainder()
        .iter()
        .zip(schunks.remainder().chunks_exact(2))
    {
        let lo = u32::from(u16::from_le_bytes([sc[0], sc[1]]));
        dc.set(dc.get() ^ (hi_pack | lo));
        let e = &ep[lo as usize];
        e.set(e.get() ^ u);
        u += 1;
    }
}

/// Rare-epoch XOR scatter for the single `(mark_byte, mark_epoch)` slice that contains `mark`.
#[cold]
#[inline(never)]
fn cell_xor_mark(
    dst_cells: &[Cell<u32>],
    ep: &[Cell<u32>],
    src: &[u8],
    mut u: u32,
    hi_pack: u32,
    mark_lo: u16,
    predecessor: &mut u32,
) {
    for (dc, sc) in dst_cells.iter().zip(src.chunks_exact(2)) {
        let lo = u16::from_le_bytes([sc[0], sc[1]]);
        if lo == mark_lo {
            *predecessor = u;
        }
        let lo32 = u32::from(lo);
        dc.set(dc.get() ^ (hi_pack | lo32));
        let e = &ep[lo32 as usize];
        e.set(e.get() ^ u);
        u += 1;
    }
}

/// Window of `edges` for the current 65,536-row epoch: `Full` proves every `u16` index in bounds
/// statically; `Tail` handles a block's final partial epoch without allocating any padding.
#[derive(Clone, Copy)]
enum EpochSlice<'e> {
    Full(&'e [Cell<u32>; 65536]),
    Tail(&'e [Cell<u32>]),
}

#[inline(always)]
fn epoch_slice(cells: &[Cell<u32>], ep_off: usize) -> EpochSlice<'_> {
    if ep_off + 65_536 <= cells.len() {
        EpochSlice::Full(cells[ep_off..ep_off + 65_536].try_into().unwrap())
    } else {
        EpochSlice::Tail(&cells[ep_off..])
    }
}

/// A byte's queue read in order for bidirectional XOR wiring, holding a register-resident chunk
/// cursor and the current epoch's window into `edges`.
pub(super) struct XorReaderOf<'a, 'e, const C: usize, const STRIDE: usize = CHUNK> {
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    st: CursorState<'a>,
    cells: &'e [Cell<u32>],
    ep_off: usize,
    ep: EpochSlice<'e>,
    is_mark_byte: bool,
    mark_epoch: u32,
    mark_lo: u16,
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn wire_xor_slow<'a, 'e, const C: usize, const STRIDE: usize>(
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    cells: &'e [Cell<u32>],
    is_mark_byte: bool,
    mark_epoch: u32,
    mark_lo: u16,
    mut st: CursorState<'a>,
    mut ep_off: usize,
    mut ep: EpochSlice<'e>,
    mut dst: usize,
    mut left: usize,
    pack: u32,
    predecessor: &mut u32,
) -> (CursorState<'a>, usize, EpochSlice<'e>) {
    while left > 0 {
        if st.cur.is_empty() {
            st = cursor_refill::<C, STRIDE>(
                out,
                spill,
                in_out,
                a,
                st.id,
                st.at_end,
                st.read_end,
                st.epoch,
                st.cut,
            );
            let new_off = (st.epoch as usize) << 16;
            if new_off != ep_off {
                ep_off = new_off;
                ep = epoch_slice(cells, new_off);
            }
        }
        let take = left.min(st.cur.len() / 2);
        let (src, rest) = st.cur.split_at(take * 2);
        st.cur = rest;
        let hi_pack = pack | (st.epoch << 16);
        let dst_cells = &cells[dst..dst + take];
        if !is_mark_byte || st.epoch != mark_epoch {
            match ep {
                EpochSlice::Full(e) => cell_xor_4x(dst_cells, e, src, dst as u32, hi_pack),
                EpochSlice::Tail(e) => cell_xor_4x_tail(dst_cells, e, src, dst as u32, hi_pack),
            }
        } else {
            let ep_slice: &[Cell<u32>] = match ep {
                EpochSlice::Full(e) => &e[..],
                EpochSlice::Tail(e) => e,
            };
            cell_xor_mark(
                dst_cells,
                ep_slice,
                src,
                dst as u32,
                hi_pack,
                mark_lo,
                predecessor,
            );
        }
        dst += take;
        left -= take;
    }
    (st, ep_off, ep)
}

#[cold]
#[inline(never)]
fn wire_fwd_slow<'a, const C: usize, const STRIDE: usize>(
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    mut st: CursorState<'a>,
    mut dst: &mut [u32],
    pack: u32,
) -> CursorState<'a> {
    while !dst.is_empty() {
        if st.cur.is_empty() {
            st = cursor_refill::<C, STRIDE>(
                out,
                spill,
                in_out,
                a,
                st.id,
                st.at_end,
                st.read_end,
                st.epoch,
                st.cut,
            );
        }
        let take = dst.len().min(st.cur.len() / 2);
        let (src, rest) = st.cur.split_at(take * 2);
        st.cur = rest;
        let (now, drest) = dst.split_at_mut(take);
        fill_fwd_overlap(now, src, pack | (st.epoch << 16));
        dst = drest;
    }
    st
}

impl<'a, 'e, const C: usize, const STRIDE: usize> XorReaderOf<'a, 'e, C, STRIDE> {
    /// Byte `a`'s queue for XOR wiring into `cells` (whose length must be at least `m`).
    #[inline(always)]
    pub(super) fn new(
        out: &'a [u8],
        spill: &'a Spill,
        heads: &Heads,
        a: usize,
        cells: &'e [Cell<u32>],
        mark: u32,
        mark_byte: u8,
    ) -> Self {
        let st = cursor_init::<C, STRIDE>(out, spill, heads, a);
        let ep_off = (st.epoch as usize) << 16;
        let ep = epoch_slice(cells, ep_off);
        XorReaderOf {
            out,
            spill,
            in_out: heads.in_out,
            a,
            st,
            cells,
            ep_off,
            ep,
            is_mark_byte: a == usize::from(mark_byte),
            mark_epoch: mark >> 16,
            mark_lo: (mark & 0xFFFF) as u16,
        }
    }

    /// Each arrow from row `u` to `v` contributes `v` to `u`'s low bits and `u` to `v`'s low bits
    /// by XOR. The high bits of `u` hold its word code. Save the incoming neighbor of `mark` to
    /// start the walk in either direction.
    #[inline(always)]
    pub(super) fn wire_xor(&mut self, dst: usize, left: usize, pack: u32, predecessor: &mut u32) {
        let bytes_need = left * 2;
        if bytes_need <= self.st.cur.len() {
            let (src, rest) = self.st.cur.split_at(bytes_need);
            self.st.cur = rest;
            let hi_pack = pack | (self.st.epoch << 16);
            let dst_cells = &self.cells[dst..dst + left];
            if !self.is_mark_byte || self.st.epoch != self.mark_epoch {
                match self.ep {
                    EpochSlice::Full(ep) => cell_xor_4x(dst_cells, ep, src, dst as u32, hi_pack),
                    EpochSlice::Tail(ep) => {
                        cell_xor_4x_tail(dst_cells, ep, src, dst as u32, hi_pack)
                    }
                }
            } else {
                let ep_slice: &[Cell<u32>] = match self.ep {
                    EpochSlice::Full(ep) => &ep[..],
                    EpochSlice::Tail(ep) => ep,
                };
                cell_xor_mark(
                    dst_cells,
                    ep_slice,
                    src,
                    dst as u32,
                    hi_pack,
                    self.mark_lo,
                    predecessor,
                );
            }
        } else {
            let (st, ep_off, ep) = wire_xor_slow::<C, STRIDE>(
                self.out,
                self.spill,
                self.in_out,
                self.a,
                self.cells,
                self.is_mark_byte,
                self.mark_epoch,
                self.mark_lo,
                self.st,
                self.ep_off,
                self.ep,
                dst,
                left,
                pack,
                predecessor,
            );
            self.st = st;
            self.ep_off = ep_off;
            self.ep = ep;
        }
    }
}

/// A byte's queue read in order for forward wiring, holding a register-resident chunk cursor.
pub(super) struct ReaderOf<'a, const C: usize, const STRIDE: usize = CHUNK> {
    out: &'a [u8],
    spill: &'a Spill,
    in_out: u32,
    a: usize,
    st: CursorState<'a>,
}

/// Shipped reader (`CHUNK` usable ranks per `2 * STRIDE` byte chunk)
#[cfg(test)]
pub(super) type Reader<'a> = ReaderOf<'a, CHUNK, STRIDE>;

impl<'a, const C: usize, const STRIDE: usize> ReaderOf<'a, C, STRIDE> {
    /// Byte `a`'s queue, from its first rank
    #[inline(always)]
    pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        ReaderOf {
            out,
            spill,
            in_out: heads.in_out,
            a,
            st: cursor_init::<C, STRIDE>(out, spill, heads, a),
        }
    }

    /// The next `dst.len()` ranks, each with `pack` OR'd into the high bits, into `dst`.
    #[inline(always)]
    pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let bytes_need = dst.len() * 2;
        if bytes_need <= self.st.cur.len() {
            let (src, rest) = self.st.cur.split_at(bytes_need);
            self.st.cur = rest;
            fill_fwd_overlap(dst, src, pack | (self.st.epoch << 16));
        } else {
            self.st = wire_fwd_slow::<C, STRIDE>(
                self.out,
                self.spill,
                self.in_out,
                self.a,
                self.st,
                dst,
                pack,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build queues from a synthetic rank sequence and verify both `ReaderOf::wire` and
    /// `XorReaderOf::wire_xor` against a plain element-by-element reference.
    fn check_wiring_case<const C: usize, const S: usize>(
        m: u32,
        out_bytes: usize,
        mark: u32,
        force_self_loop: bool,
    ) {
        let mut out = vec![0u8; out_bytes];
        let mut spill = Spill::default();
        let mut queues = QueuesOf::<C, S>::new(&mut out, &mut spill, u32::MAX);
        let mut thin = [0u32; 256];
        let mut per_byte: [Vec<u32>; 256] = std::array::from_fn(|_| Vec::new());
        let mut mark_byte = 0u8;

        let mut pos = 0u32;
        let mut rng = 0x9E37_79B9u32 ^ m ^ mark.rotate_left(7);
        while pos < m {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let a = if force_self_loop {
                0usize
            } else {
                ((rng >> 24) as usize) % 6
            };
            let w = (1 + ((rng >> 16) % 16)).min(m - pos);
            queues.push(a, pos, w, &thin);
            for r in pos..pos + w {
                per_byte[a].push(r);
                if r == mark {
                    mark_byte = a as u8;
                }
            }
            thin[a] += w;
            pos += w;
        }
        let heads = queues.heads();
        if out_bytes <= 2 * (2 * S) && m >= 2 * C as u32 {
            assert!(
                spill.overflow() > 0,
                "case C={C}/S={S} m={m}, out_bytes={out_bytes} must exercise overflow chunks"
            );
        }

        // Partition each byte's ranks across varied band spans (including 0, 1..17, 511..513, 1007..1009).
        let span_pat = [
            0usize, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 511, 512, 513, 1007, 1008, 1009,
        ];
        let mut bands: [Vec<(usize, usize, u32)>; 256] = std::array::from_fn(|_| Vec::new());
        let mut dst_cursor = 0usize;
        let mut pi = 0usize;
        let bits = if m <= 1 {
            1
        } else {
            (u32::BITS - (m - 1).leading_zeros()).max(1)
        };
        for a in 0..256 {
            let mut rem = per_byte[a].len();
            while rem > 0 {
                let sp = span_pat[pi % span_pat.len()].min(rem);
                pi += 1;
                let code = ((pi as u32) % 63) + 1;
                let pack = code << bits;
                bands[a].push((dst_cursor, sp, pack));
                dst_cursor += sp;
                rem -= sp;
            }
        }
        assert_eq!(dst_cursor, m as usize);

        // Reference forward and XOR outputs.
        let mut ref_fwd = vec![0xDEAD_BEEFu32; m as usize];
        let mut ref_xor = vec![0u32; m as usize];
        let mut ref_pred = mark;
        for a in 0..256 {
            let mut idx = 0usize;
            for &(dst, sp, pack) in &bands[a] {
                for i in 0..sp {
                    let u = dst + i;
                    let v = per_byte[a][idx + i];
                    ref_fwd[u] = pack | v;
                    if v == mark {
                        ref_pred = u as u32;
                    }
                    ref_xor[u] ^= pack | v;
                    ref_xor[v as usize] ^= u as u32;
                }
                idx += sp;
            }
        }

        // Test forward ReaderOf::wire.
        let mut got_fwd = vec![0xDEAD_BEEFu32; m as usize];
        for (a, byte_bands) in bands.iter().enumerate() {
            if byte_bands.is_empty() {
                continue;
            }
            let mut r = ReaderOf::<C, S>::new(&out, &spill, &heads, a);
            for &(dst, sp, pack) in byte_bands {
                r.wire(&mut got_fwd[dst..dst + sp], pack);
            }
        }
        assert_eq!(
            got_fwd, ref_fwd,
            "forward wire mismatch for C={C}/S={S} m={m}"
        );

        // Test XorReaderOf::wire_xor on exact-length `edges` (len == m, 0 extra bytes, exercising
        // EpochSlice::Tail on partial epochs) AND with slack past `m` (exercising EpochSlice::Full
        // and proving trailing slack is untouched).
        if m >= 1 {
            let extras: &[usize] = if cfg!(miri) { &[0] } else { &[0, 65_536] };
            for &extra in extras {
                let len = m as usize + extra;
                let mut got_xor = vec![0xA5A5_A5A5u32; len];
                got_xor[..m as usize].fill(0);
                let cells = Cell::from_mut(&mut got_xor[..]).as_slice_of_cells();
                let mut got_pred = mark;
                for (a, byte_bands) in bands.iter().enumerate() {
                    if byte_bands.is_empty() {
                        continue;
                    }
                    let mut r =
                        XorReaderOf::<C, S>::new(&out, &spill, &heads, a, cells, mark, mark_byte);
                    for &(dst, sp, pack) in byte_bands {
                        r.wire_xor(dst, sp, pack, &mut got_pred);
                    }
                }
                assert_eq!(
                    got_pred, ref_pred,
                    "predecessor mismatch for C={C}/S={S} m={m}, mark={mark}, extra={extra}"
                );
                assert_eq!(
                    &got_xor[..m as usize],
                    &ref_xor[..],
                    "wire_xor mismatch for C={C}/S={S} m={m}, mark={mark}, extra={extra}"
                );
                if extra > 0 {
                    assert!(
                        got_xor[m as usize..].iter().all(|&w| w == 0xA5A5_A5A5),
                        "slack words beyond m were touched for C={C}/S={S} m={m}, extra={extra}"
                    );
                }
            }
        }
    }

    #[test]
    fn wire_and_wire_xor_small_and_span_boundaries() {
        for &m in &[
            0u32, 1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 511, 512, 513, 1007, 1008, 1009,
        ] {
            let mark = if m == 0 { 0 } else { m / 2 };
            check_wiring_case::<512, 512>(m, 4096, mark, false);
            check_wiring_case::<1008, 1024>(m, 8192, mark, false);
            if m >= 1 {
                // Force identity u == v self-loops on every row.
                check_wiring_case::<512, 512>(m, 4096, 0, true);
                check_wiring_case::<512, 512>(m, 4096, m - 1, true);
                check_wiring_case::<1008, 1024>(m, 8192, 0, true);
                check_wiring_case::<1008, 1024>(m, 8192, m - 1, true);
            }
        }
    }

    #[test]
    fn wire_and_wire_xor_overflow_and_epoch_boundaries() {
        // Multi-chunk overflow (only 1 chunk fits in `out`).
        check_wiring_case::<512, 512>(2500, 1024, 0, false);
        check_wiring_case::<512, 512>(2500, 1024, 2499, false);
        check_wiring_case::<512, 512>(2500, 0, 1234, false);
        check_wiring_case::<1008, 1024>(3500, 2048, 0, false);
        check_wiring_case::<1008, 1024>(3500, 2048, 3499, false);
        check_wiring_case::<1008, 1024>(3500, 0, 1234, false);

        // Exact EPOCH = 65536 boundaries and multi-epoch cuts, with mark on every edge position.
        for &mark in &[0u32, 1, 65_534, 65_535, 65_536, 65_537, 69_999] {
            check_wiring_case::<512, 512>(70_000, 160_000, mark, false);
            check_wiring_case::<1008, 1024>(70_000, 160_000, mark, false);
        }
        // Multi-epoch with heavy overflow.
        check_wiring_case::<512, 512>(66_000, 2048, 65_536, false);
        check_wiring_case::<512, 512>(65_535, 140_000, 65_534, false);
        check_wiring_case::<512, 512>(65_536, 140_000, 65_535, false);
        check_wiring_case::<1008, 1024>(66_000, 4096, 65_536, false);
        check_wiring_case::<1008, 1024>(65_535, 160_000, 65_534, false);
        check_wiring_case::<1008, 1024>(65_536, 160_000, 65_535, false);
    }

    #[test]
    fn miri_outq_wire_and_wire_xor() {
        // Exercise ReaderOf::wire (overlapping 8/4/2/1), XorReaderOf (Tail + self-loops + overflow)
        // on both 512/512 and guarded 1008/1024 chunk layouts.
        for &m in &[0u32, 1, 2, 3, 7, 8, 9, 15, 16, 17] {
            let mark = if m == 0 { 0 } else { m / 2 };
            check_wiring_case::<512, 512>(m, 1024, mark, false);
            check_wiring_case::<1008, 1024>(m, 2048, mark, false);
        }
        check_wiring_case::<512, 512>(9, 1024, 0, true);
        check_wiring_case::<512, 512>(550, 1024, 275, false);
        check_wiring_case::<1008, 1024>(9, 2048, 0, true);
        check_wiring_case::<1008, 1024>(1050, 2048, 525, false);

        // Exercise XorReaderOf's EpochSlice::Full (&[Cell<u32>; 65536]) path directly under Miri.
        let mut out = vec![0u8; 4 * STRIDE];
        let mut spill = Spill::default();
        let mut q = Queues::new(&mut out, &mut spill, u32::MAX);
        let mut thin = [0u32; 256];
        q.push(1, 0, 16, &thin);
        thin[1] += 16;
        q.push(1, 16, 1, &thin);
        let heads = q.heads();

        let mut edges = vec![0u32; 65_536];
        let cells = Cell::from_mut(&mut edges[..]).as_slice_of_cells();
        let mut pred = 5u32;
        // Byte 1 != mark_byte 0 -> hits cell_xor_4x on EpochSlice::Full with u == v self-loops.
        let mut r = XorReaderOf::<CHUNK, STRIDE>::new(&out, &spill, &heads, 1, cells, 5, 0);
        r.wire_xor(0, 17, 7 << 16, &mut pred);
        for (i, &w) in edges[..17].iter().enumerate() {
            assert_eq!(w, (7 << 16) | (i as u32) ^ (i as u32));
        }
        // Byte 1 == mark_byte 1 -> hits cell_xor_mark on EpochSlice::Full.
        edges[..17].fill(0);
        let cells = Cell::from_mut(&mut edges[..]).as_slice_of_cells();
        let mut r = XorReaderOf::<CHUNK, STRIDE>::new(&out, &spill, &heads, 1, cells, 5, 1);
        r.wire_xor(0, 17, 3 << 16, &mut pred);
        assert_eq!(pred, 5);
        for &w in &edges[..17] {
            assert_eq!(w, 3 << 16);
        }
    }
}
