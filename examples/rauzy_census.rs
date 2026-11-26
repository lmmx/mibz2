//! Rauzy census: A6b time split and induction-order comparison on the production Open kernel's
//! auxiliary interval exchange (docs/journal/2026-11-24-algorithm-level/rauzy-census/).
//!
//! Characterisation only; zero changes to `src/`, zero `unsafe`.
//!
//! Usage:
//!   rauzy_census split  FILE.bz2 [--forced] [--repeats N]
//!   rauzy_census orders FILE.bz2 [--forced] [--repeats N] [--every K] [--per-block]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::collections::{BTreeSet, BinaryHeap, HashMap};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs, src/open/endpoint.rs,
// src/open/outq.rs, src/cells_lazy.rs verbatim)
// ---------------------------------------------------------------------------

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const RULE: usize = 4;
const NARROW_NODES: usize = 1 << 16;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const CHUNK: usize = 512;
const CHUNK_BYTES: usize = 2 * CHUNK;
const NONE_CHUNK: u32 = u32::MAX;
const EPOCH: u32 = 1 << 16;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cap {
    m: u32,
    strips: usize,
}

impl Cap {
    const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };

    fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
}

fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
}

fn open_cap(records: usize, n: usize, p: u32, forced: bool) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if forced || lazy_counts_admit(records, n) {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::with_capacity(l.len() / 4 + 16);
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

#[inline(always)]
fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
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

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0;
    counts.map(|w| {
        let start = total;
        total += w;
        start
    })
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
    }
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

// ---------------------------------------------------------------------------
// OutQ16 queues (mirroring src/open/outq.rs)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; CHUNK_BYTES]>,
    cuts: Vec<[u32; 256]>,
}

struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
    limit: u32,
    over: bool,
}

#[derive(Clone, Copy)]
struct Heads {
    heads: [u32; 256],
    in_out: u32,
}

impl<'a> Queues<'a> {
    fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
        spill.ovf.clear();
        spill.cuts.clear();
        Queues {
            in_out: (out.len() / CHUNK_BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE_CHUNK; 256],
            tails: [NONE_CHUNK; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
        }
    }

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }

    #[inline(always)]
    fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }

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
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }

    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; CHUNK_BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE_CHUNK);
        }
        self.spill.next[id as usize] = NONE_CHUNK;
        if self.heads[a] == NONE_CHUNK {
            self.heads[a] = id;
        } else {
            self.spill.next[self.tails[a] as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = 0;
    }

    fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
        if id < self.in_out {
            let at = id as usize * CHUNK_BYTES;
            &mut self.out[at..at + CHUNK_BYTES]
        } else {
            &mut self.spill.ovf[(id - self.in_out) as usize][..]
        }
    }
}

struct QueueReader<'a> {
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

impl<'a> QueueReader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        QueueReader {
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
            let at = self.id as usize * CHUNK_BYTES + 2 * self.at;
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
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
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
    fn wire(&mut self, dst: &mut [u32], pack: u32) {
        let span = dst.len();
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = pack | (self.epoch << 16);
            let at = self.id as usize * CHUNK_BYTES + 2 * self.at;
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
                let at = self.id as usize * CHUNK_BYTES;
                &self.out[at..at + CHUNK_BYTES]
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

// ---------------------------------------------------------------------------
// Endpoint Links & Elim (mirroring src/open/endpoint.rs, plus left / two-sided
// / Zorich / division orders on the exact same arena)
// ---------------------------------------------------------------------------

trait Links {
    const STRIDE: usize;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
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
}

impl Links for Wide {
    const STRIDE: usize = 6;
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
}

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

/// Sentinel id separating `terminals[side]` (the retained prefix `[0, m)`) from
/// `order[side]` (the eliminated suffix `[m, m + n)`) in two-ended orders.
const MID: u32 = 1;

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
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
    }
    #[inline(always)]
    fn set_bottom(&mut self, id: u32, x: u32) {
        self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
    }

    fn assign_bottom(&mut self, rows: u32, end_sentinel: u32) {
        let mut first = L::next(self.a, 0, 1);
        let mut last = L::prev(self.a, end_sentinel, 1);
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
        if first != end_sentinel && first != 0 {
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

    /// Close `0 -> first -> MID -> second -> 0` so `[0, m)` is `(0..MID)` and
    /// `[m, m + n)` is `(MID..0)`.
    fn close_with_mid(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        if first.1 != 0 {
            L::set_next(self.a, prev, side, first.0);
            L::set_prev(self.a, first.0, side, prev);
            prev = first.1;
        }
        L::set_next(self.a, prev, side, MID);
        L::set_prev(self.a, MID, side, prev);
        prev = MID;
        if second.1 != 0 {
            L::set_next(self.a, prev, side, second.0);
            L::set_prev(self.a, second.0, side, prev);
            prev = second.1;
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
    fn pop_head_mid(&mut self, id: u32, side: usize) {
        let next = L::next(self.a, id, side);
        L::set_prev(self.a, next, side, MID);
        L::set_next(self.a, MID, side, next);
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
    fn insert_before(&mut self, at: u32, id: u32, side: usize) {
        let prev = L::prev(self.a, at, side);
        L::set_prev(self.a, id, side, prev);
        L::set_next(self.a, id, side, at);
        L::set_next(self.a, prev, side, id);
        L::set_prev(self.a, at, side, id);
    }

    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule(w);
            let tag = self.a[r + 3];
            if tag >> 28 == 0 {
                tag >> 24
            } else {
                self.a[r + 2]
            }
        }
    }

    #[inline(always)]
    fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = self.rule(id);
        u128::from(self.a[r])
            | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64)
            | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let (left, right) = (left & ID_MASK, right & ID_MASK);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len(left) + self.len(right)
        };
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
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

    /// Production right-end elimination (exact mirror of src/open/endpoint.rs).
    fn eliminate_right<const SPIRALS: bool>(
        &mut self,
        remaining: &mut u32,
        steps: &mut usize,
        contracted: &mut usize,
        divisions: &mut usize,
    ) -> bool {
        let (mut left, mut taken, mut csteps, mut divs) =
            (*remaining, *steps, *contracted, *divisions);
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
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    taken += q as usize;
                    csteps += 1;
                    divs += 1;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    taken += q as usize;
                    csteps += 1;
                    divs += 1;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            csteps += 1;
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
        (*remaining, *steps, *contracted, *divisions) = (left, taken, csteps, divs);
        done
    }

    /// Left-end elimination at the `MID` sentinel (`[m, m+n)` consumed left to right).
    fn eliminate_left<const SPIRALS: bool>(
        &mut self,
        remaining: &mut u32,
        steps: &mut usize,
        contracted: &mut usize,
        divisions: &mut usize,
    ) -> bool {
        let (mut left, mut taken, mut csteps, mut divs) =
            (*remaining, *steps, *contracted, *divisions);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::next(self.a, MID, 0), L::next(self.a, MID, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::next(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    taken += q as usize;
                    csteps += 1;
                    divs += 1;
                    continue;
                }
                if aw < bw && L::next(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    assert!(q * aw < bw && q * aw <= left);
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    taken += q as usize;
                    csteps += 1;
                    divs += 1;
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            csteps += 1;
            if x == y {
                self.pop_head_mid(x, 0);
                self.pop_head_mid(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_head_mid(x, 0);
                self.insert_before(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.pop_head_mid(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.pop_head_mid(y, 1);
                self.insert_before(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }
        };
        (*remaining, *steps, *contracted, *divisions) = (left, taken, csteps, divs);
        done
    }

    /// Two-sided elimination: inspects both right `(prev(0))` and left `(next(MID))`.
    /// `ALT = false`: equal/drop first, else larger cell `min(aw, bw)`.
    /// `ALT = true`: strictly alternates right and left.
    fn eliminate_two_sided<const SPIRALS: bool, const ALT: bool>(
        &mut self,
        remaining: &mut u32,
        steps: &mut usize,
        contracted: &mut usize,
        divisions: &mut usize,
    ) -> bool {
        let (mut left, mut taken, mut csteps, mut divs) =
            (*remaining, *steps, *contracted, *divisions);
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (xr, yr) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (xl, yl) = (L::next(self.a, MID, 0), L::next(self.a, MID, 1));
            let (awr, bwr) = (self.width(xr), self.width(yr));
            let (awl, bwl) = (self.width(xl), self.width(yl));
            let take_left = if ALT {
                csteps & 1 == 1
            } else if xr == yr || awr == bwr {
                false
            } else if xl == yl || awl == bwl {
                true
            } else {
                awl.min(bwl) > awr.min(bwr)
            };
            if !take_left {
                let (x, y, aw, bw) = (xr, yr, awr, bwr);
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        taken += q as usize;
                        csteps += 1;
                        divs += 1;
                        continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word);
                        self.set_width(y, bw - q * aw);
                        left -= q * aw;
                        taken += q as usize;
                        csteps += 1;
                        divs += 1;
                        continue;
                    }
                }
                let w = aw.min(bw);
                left -= w;
                taken += 1;
                csteps += 1;
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
            } else {
                let (x, y, aw, bw) = (xl, yl, awl, bwl);
                if SPIRALS && x != y {
                    if aw > bw && L::next(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        taken += q as usize;
                        csteps += 1;
                        divs += 1;
                        continue;
                    }
                    if aw < bw && L::next(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word);
                        self.set_width(y, bw - q * aw);
                        left -= q * aw;
                        taken += q as usize;
                        csteps += 1;
                        divs += 1;
                        continue;
                    }
                }
                let w = aw.min(bw);
                left -= w;
                taken += 1;
                csteps += 1;
                if x == y {
                    self.pop_head_mid(x, 0);
                    self.pop_head_mid(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_head_mid(x, 0);
                    self.insert_before(y, x, 0);
                    self.set_word(x, word);
                    if aw == bw {
                        self.cut(y, 0);
                        self.pop_head_mid(y, 1);
                    } else {
                        self.set_width(y, bw - w);
                    }
                } else {
                    self.pop_head_mid(y, 1);
                    self.insert_before(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                }
            }
        };
        (*remaining, *steps, *contracted, *divisions) = (left, taken, csteps, divs);
        done
    }

    /// Full Zorich-accelerated right end: batches an entire same-winner streak in one
    /// chain splice (keeping the winner in registers) and contracts same-pair spirals
    /// by integer division + squaring.
    fn eliminate_zorich(
        &mut self,
        remaining: &mut u32,
        raw_steps: &mut usize,
        zorich_steps: &mut usize,
        divisions: &mut usize,
    ) -> bool {
        let (mut left, mut raw, mut zsteps, mut divs) =
            (*remaining, *raw_steps, *zorich_steps, *divisions);
        let room = SPIRAL_RULES * RULE;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (mut aw, mut bw) = (self.width(x), self.width(y));
            if x == y {
                left -= aw;
                raw += 1;
                zsteps += 1;
                self.pop_tail(x, 0);
                self.pop_tail(x, 1);
                continue;
            }
            if aw == bw {
                let word = self.join(self.word(x), self.word(y));
                left -= aw;
                raw += 1;
                zsteps += 1;
                self.pop_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                self.cut(y, 0);
                self.pop_tail(y, 1);
                continue;
            }
            if aw > bw {
                // Top streak: `x` stays winner while `aw > width(cur)` and `cur != x`.
                let xw = self.word(x);
                if L::prev(self.a, y, 1) == x {
                    let q = (aw - 1) / bw;
                    let head = if q == 1 { xw } else { self.power(xw, q) };
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    raw += q as usize;
                    zsteps += 1;
                    divs += usize::from(q >= 2);
                    continue;
                }
                // Batch distinct losers `y_1, y_2, ...` from tail of side 1.
                // Reversed chain `y_k -> ... -> y_1` is spliced after `x` on side 1.
                let nx = L::next(self.a, x, 1);
                let mut cur = y;
                let first_loser = y;
                let mut last_loser = y;
                let mut prev_tail = L::prev(self.a, y, 1);
                let mut prev_in_chain = nx;
                let mut count = 0usize;
                let mut need_grow = false;
                while cur != x && cur != 0 {
                    let cw = self.width(cur);
                    if aw <= cw {
                        break;
                    }
                    let next_tail = L::prev(self.a, cur, 1);
                    if next_tail == x && count > 0 {
                        // Stop batch before hitting a spiral with `x`; outer loop handles it.
                        break;
                    }
                    if self.top + RULE > self.a.len() {
                        need_grow = true;
                        break;
                    }
                    let word = self.join(xw, self.word(cur));
                    self.set_word(cur, word);
                    aw -= cw;
                    left -= cw;
                    raw += 1;
                    count += 1;
                    // Link `cur` before `prev_in_chain` (building reversed chain).
                    L::set_next(self.a, cur, 1, prev_in_chain);
                    if prev_in_chain != nx {
                        L::set_prev(self.a, prev_in_chain, 1, cur);
                    }
                    last_loser = cur;
                    prev_in_chain = cur;
                    prev_tail = next_tail;
                    cur = next_tail;
                }
                assert!(count > 0);
                // Splice `x -> last_loser -> ... -> first_loser -> nx` and `prev_tail -> 0`.
                L::set_next(self.a, prev_tail, 1, 0);
                L::set_prev(self.a, 0, 1, prev_tail);
                L::set_prev(self.a, last_loser, 1, x);
                L::set_next(self.a, x, 1, last_loser);
                L::set_prev(self.a, nx, 1, first_loser);
                self.set_width(x, aw);
                zsteps += 1;
                if need_grow {
                    break false;
                }
            } else {
                // Bottom streak: `y` stays winner while `bw > width(cur)` and `cur != y`.
                let yw = self.word(y);
                if L::prev(self.a, x, 0) == y {
                    let q = (bw - 1) / aw;
                    let tail = if q == 1 { yw } else { self.power(yw, q) };
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word);
                    self.set_width(y, bw - q * aw);
                    left -= q * aw;
                    raw += q as usize;
                    zsteps += 1;
                    divs += usize::from(q >= 2);
                    continue;
                }
                let ny = L::next(self.a, y, 0);
                let mut cur = x;
                let first_loser = x;
                let mut last_loser = x;
                let mut prev_tail = L::prev(self.a, x, 0);
                let mut prev_in_chain = ny;
                let mut count = 0usize;
                let mut need_grow = false;
                while cur != y && cur != 0 {
                    let cw = self.width(cur);
                    if bw <= cw {
                        break;
                    }
                    let next_tail = L::prev(self.a, cur, 0);
                    if next_tail == y && count > 0 {
                        break;
                    }
                    if self.top + RULE > self.a.len() {
                        need_grow = true;
                        break;
                    }
                    let word = self.join(self.word(cur), yw);
                    self.set_word(cur, word);
                    bw -= cw;
                    left -= cw;
                    raw += 1;
                    count += 1;
                    L::set_next(self.a, cur, 0, prev_in_chain);
                    if prev_in_chain != ny {
                        L::set_prev(self.a, prev_in_chain, 0, cur);
                    }
                    last_loser = cur;
                    prev_in_chain = cur;
                    prev_tail = next_tail;
                    cur = next_tail;
                }
                assert!(count > 0);
                L::set_next(self.a, prev_tail, 0, 0);
                L::set_prev(self.a, 0, 0, prev_tail);
                L::set_prev(self.a, last_loser, 0, y);
                L::set_next(self.a, y, 0, last_loser);
                L::set_prev(self.a, ny, 0, first_loser);
                self.set_width(y, bw);
                zsteps += 1;
                if need_grow {
                    break false;
                }
            }
        };
        (*remaining, *raw_steps, *zorich_steps, *divisions) = (left, raw, zsteps, divs);
        done
    }

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let id = word & ID_MASK;
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(id, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
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
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        let start = self.spelling.len();
        let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
        let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
        if self.contents.is_empty() {
            return self.insert(entry, len);
        }
        let bytes = if len == LONG {
            &self.spelling[start..]
        } else {
            &entry[..len as usize]
        };
        let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
        let old = self.contents[index] as usize;
        let same = old != 0
            && self.lens[old] == len
            && if len == LONG {
                long_word(&self.inline16[old], self.spelling) == bytes
            } else {
                self.inline16[old] == entry
            };
        let code = if same {
            if len == LONG {
                if memoized {
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
            }
            old as u32
        } else {
            let code = self.insert(entry, len);
            self.contents[index] = code;
            code
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
    let (low, high, middle) = if len == LONG {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
    } else {
        let value = u128::from_le_bytes(*entry);
        (value as u64, (value >> 64) as u64, 0)
    };
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

// ---------------------------------------------------------------------------
// Coordinate-tracked Open auxiliary IET for MONODROMY's 1.279 order and its
// fast flat-array online counterpart
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum GeoPolicy {
    /// Free steps wherever ready, else largest cell (MONODROMY §5.6 1.279 order).
    FreeThenLargest,
    /// Free steps wherever ready, else rightmost coincidence.
    FreeThenRight,
    /// Link-only ("weak") free test, else largest cell (MONODROMY §5.6 1.302 order).
    WeakThenLargest,
}

/// Rule arena shared by coordinate-tracked orders (same 4-word rule encoding as production).
struct RuleArena {
    a: Vec<u32>,
}

impl RuleArena {
    fn new(cap_rules: usize) -> Self {
        RuleArena {
            a: Vec::with_capacity(cap_rules * RULE),
        }
    }

    fn clear(&mut self) {
        self.a.clear();
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = (w - 257) as usize * RULE;
            let tag = self.a[r + 3];
            if tag >> 28 == 0 {
                tag >> 24
            } else {
                self.a[r + 2]
            }
        }
    }

    #[inline(always)]
    fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = (id - 257) as usize * RULE;
        u128::from(self.a[r])
            | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64)
            | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = (self.a.len() / RULE) as u32 + 257;
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let (left, right) = (left & ID_MASK, right & ID_MASK);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len(left) + self.len(right)
        };
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload(left)
                | (self.short_payload(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a.extend_from_slice(&rule);
        (len.min(SATURATED) << ID_BITS) | id
    }
}

struct GeoResult {
    /// Surviving nodes in `[0, m)` sorted by `ts` (side-0 order): `(width, bs, word)`.
    surviving: Vec<(u32, u32, u32)>,
    steps: usize,
    floor: usize,
    rules: usize,
    setup_ns: u64,
    steps_ns: u64,
}

fn run_geo_on_open(
    strips_raw: &[u32],
    image_order: &[u32],
    ns: usize,
    n: u32,
    m: u32,
    policy: GeoPolicy,
    rules: &mut RuleArena,
) -> GeoResult {
    let t0 = Instant::now();
    rules.clear();
    let max_nodes = 3 * ns + 3;
    let mut ts = Vec::with_capacity(max_nodes);
    let mut bs = Vec::with_capacity(max_nodes);
    let mut lam = Vec::with_capacity(max_nodes);
    let mut word = Vec::with_capacity(max_nodes);
    let mut top_end: HashMap<u32, u32> = HashMap::with_capacity(max_nodes * 2);
    let mut bot_end: HashMap<u32, u32> = HashMap::with_capacity(max_nodes * 2);

    // Strip nodes: ids 0..ns (placed in E = [m, m + n)).
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        ts.push(m + s.from);
        bs.push(m + s.to);
        lam.push(s.width);
        word.push(literal(s.byte));
        top_end.insert(m + s.from + s.width, i as u32);
        bot_end.insert(m + s.to + s.width, i as u32);
    }

    // Domain gaps: side 0 in E = [m, m+n), side 1 in R = [0, m).
    let mut x = 0u32;
    let mut bpos = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        if x < s.from {
            let w = s.from - x;
            let id = lam.len() as u32;
            ts.push(m + x);
            bs.push(bpos);
            lam.push(w);
            word.push(0);
            top_end.insert(m + x + w, id);
            bot_end.insert(bpos + w, id);
            bpos += w;
        }
        x = s.from + s.width;
    }
    if x < n {
        let w = n - x;
        let id = lam.len() as u32;
        ts.push(m + x);
        bs.push(bpos);
        lam.push(w);
        word.push(0);
        top_end.insert(m + x + w, id);
        bot_end.insert(bpos + w, id);
        bpos += w;
    }
    assert_eq!(bpos, m);

    // Image gaps: side 1 in E = [m, m+n), side 0 in R = [0, m).
    x = 0;
    let mut tpos = 0u32;
    for &i in image_order.iter() {
        let idx = i as usize;
        let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
        if x < s.to {
            let w = s.to - x;
            let id = lam.len() as u32;
            ts.push(tpos);
            bs.push(m + x);
            lam.push(w);
            word.push(0);
            top_end.insert(tpos + w, id);
            bot_end.insert(m + x + w, id);
            tpos += w;
        }
        x = s.to + s.width;
    }
    if x < n {
        let w = n - x;
        let id = lam.len() as u32;
        ts.push(tpos);
        bs.push(m + x);
        lam.push(w);
        word.push(0);
        top_end.insert(tpos + w, id);
        bot_end.insert(m + x + w, id);
        tpos += w;
    }
    assert_eq!(tpos, m);

    // Exact proved floor r2_E = |{top_end > m} U {bot_end > m}|.
    let mut coin: BTreeSet<u32> = BTreeSet::new();
    let mut e_bounds = HashMap::<u32, ()>::with_capacity(max_nodes * 2);
    for &c in top_end.keys() {
        if c > m {
            e_bounds.insert(c, ());
            if bot_end.contains_key(&c) {
                coin.insert(c);
            }
        }
    }
    for &c in bot_end.keys() {
        if c > m {
            e_bounds.insert(c, ());
        }
    }
    let floor = e_bounds.len();

    let weak = policy == GeoPolicy::WeakThenLargest;
    let use_heap = matches!(
        policy,
        GeoPolicy::FreeThenLargest | GeoPolicy::WeakThenLargest
    );

    let at =
        |c: u32, top_end: &HashMap<u32, u32>, bot_end: &HashMap<u32, u32>| -> Option<(u32, u32)> {
            if c <= m {
                return None;
            }
            match (top_end.get(&c), bot_end.get(&c)) {
                (Some(&a), Some(&b)) => Some((a, b)),
                _ => None,
            }
        };

    let reducing = |c: u32,
                    top_end: &HashMap<u32, u32>,
                    bot_end: &HashMap<u32, u32>,
                    ts: &[u32],
                    bs: &[u32],
                    lam: &[u32]|
     -> Option<(bool, u32)> {
        let (a, b) = at(c, top_end, bot_end)?;
        let (ai, bi) = (a as usize, b as usize);
        let (la, lb) = (lam[ai], lam[bi]);
        if a == b || la == lb {
            return Some((true, la));
        }
        if la > lb {
            let p = bs[ai] + la - lb;
            let ok = if p < m {
                true
            } else {
                top_end.contains_key(&p) && (!weak || top_end.contains_key(&(bs[ai] + la)))
            };
            Some((ok, lb))
        } else {
            let p = ts[bi] + lb - la;
            let ok = if p < m {
                true
            } else {
                bot_end.contains_key(&p) && (!weak || bot_end.contains_key(&(ts[bi] + lb)))
            };
            Some((ok, la))
        }
    };

    let mut ready: Vec<u32> = coin.iter().copied().collect();
    let mut waiting: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut heap: BinaryHeap<(u32, u32)> = BinaryHeap::new();
    if use_heap {
        for &c in &coin {
            if let Some((_, cell)) = reducing(c, &top_end, &bot_end, &ts, &bs, &lam) {
                heap.push((cell, c));
            }
        }
    }
    let setup_ns = t0.elapsed().as_nanos() as u64;

    let t1 = Instant::now();
    let mut remaining = n;
    let mut steps = 0usize;
    let mut not_reducing = 0usize;
    let mut fresh = [0u32; 3];

    while remaining > 0 {
        let mut pick = None;
        while let Some(c) = ready.pop() {
            match reducing(c, &top_end, &bot_end, &ts, &bs, &lam) {
                Some((true, _)) => {
                    pick = Some(c);
                    break;
                }
                Some((false, _)) => {
                    let (a, b) = at(c, &top_end, &bot_end).unwrap();
                    let (la, lb) = (lam[a as usize], lam[b as usize]);
                    let p = if la > lb {
                        bs[a as usize] + la - lb
                    } else {
                        ts[b as usize] + lb - la
                    };
                    waiting.entry(p).or_default().push(c);
                }
                None => {}
            }
        }
        let c = match pick {
            Some(c) => c,
            None => {
                if !use_heap {
                    *coin.iter().next_back().unwrap()
                } else {
                    loop {
                        let (k0, c) = heap.pop().unwrap();
                        let Some((_, k1)) = reducing(c, &top_end, &bot_end, &ts, &bs, &lam) else {
                            continue;
                        };
                        if k1 != k0 {
                            heap.push((k1, c));
                            continue;
                        }
                        break c;
                    }
                }
            }
        };

        let (a, b) = at(c, &top_end, &bot_end).unwrap();
        let (ai, bi) = (a as usize, b as usize);
        let (la, lb) = (lam[ai], lam[bi]);
        let w = la.min(lb);
        remaining -= w;
        steps += 1;
        coin.remove(&c);

        let nfresh: usize;
        if a == b {
            top_end.remove(&c);
            bot_end.remove(&c);
            lam[ai] = 0;
            fresh[0] = c - la;
            nfresh = 1;
        } else {
            let nw = rules.join(word[ai], word[bi]);
            if la > lb {
                word[bi] = nw;
                top_end.remove(&c);
                bot_end.remove(&c);
                top_end.insert(c - lb, a);
                let p = bs[ai] + la - lb;
                if p > m && !top_end.contains_key(&p) {
                    not_reducing += 1;
                }
                lam[ai] = la - lb;
                bs[bi] = p;
                bot_end.insert(p + lb, b);
                bot_end.insert(p, a);
                fresh = [c - lb, p, p + lb];
                nfresh = 3;
            } else if la < lb {
                word[ai] = nw;
                top_end.remove(&c);
                bot_end.remove(&c);
                bot_end.insert(c - la, b);
                let p = ts[bi] + lb - la;
                if p > m && !bot_end.contains_key(&p) {
                    not_reducing += 1;
                }
                lam[bi] = lb - la;
                ts[ai] = p;
                top_end.insert(p + la, a);
                top_end.insert(p, b);
                fresh = [c - la, p, p + la];
                nfresh = 3;
            } else {
                // la == lb: mirror production (keep `a`, kill `b`, or keep `b`? keep `b`!)
                word[bi] = nw;
                top_end.remove(&c);
                bot_end.remove(&c);
                lam[ai] = 0;
                bs[bi] = bs[ai];
                bot_end.insert(bs[ai] + la, b);
                fresh[0] = c - la;
                fresh[1] = bs[ai] + la;
                nfresh = 2;
            }
        }

        for &p in &fresh[..nfresh] {
            if p > m {
                if at(p, &top_end, &bot_end).is_some() {
                    coin.insert(p);
                    ready.push(p);
                    if use_heap {
                        let (_, cell) = reducing(p, &top_end, &bot_end, &ts, &bs, &lam).unwrap();
                        heap.push((cell, p));
                    }
                } else {
                    coin.remove(&p);
                }
                if let Some(v) = waiting.remove(&p) {
                    ready.extend(v);
                }
            }
        }
    }
    assert_eq!(steps, floor + not_reducing);

    // Extract surviving nodes in R = [0, m) sorted by ts.
    let mut surviving: Vec<(u32, u32, u32)> = Vec::with_capacity(top_end.len());
    for (&end_t, &id) in &top_end {
        let i = id as usize;
        let w = lam[i];
        if w > 0 {
            debug_assert!(end_t <= m && bs[i] < m);
            surviving.push((ts[i], w, id));
        }
    }
    surviving.sort_unstable_by_key(|&(t, _, _)| t);
    let out_nodes: Vec<(u32, u32, u32)> = surviving
        .into_iter()
        .map(|(_, w, id)| (w, bs[id as usize], word[id as usize]))
        .collect();
    let steps_ns = t1.elapsed().as_nanos() as u64;

    GeoResult {
        surviving: out_nodes,
        steps,
        floor,
        rules: rules.a.len() / RULE,
        setup_ns,
        steps_ns,
    }
}

/// Fast flat-array online implementation of MONODROMY's free-first + log2-bucketed
/// largest-cell order (no `HashMap`, no `BTreeSet`, no `BinaryHeap`) to measure the
/// practical cost floor of online scheduling.
struct FlatMap {
    keys: Vec<u32>,
    vals: Vec<u32>,
    mask: usize,
}

const EMPTY_KEY: u32 = u32::MAX;
const TOMB_KEY: u32 = u32::MAX - 1;

impl FlatMap {
    fn new(expected: usize) -> Self {
        let cap = (expected * 4).max(64).next_power_of_two();
        FlatMap {
            keys: vec![EMPTY_KEY; cap],
            vals: vec![0; cap],
            mask: cap - 1,
        }
    }

    fn reset(&mut self, expected: usize) {
        let cap = (expected * 4).max(64).next_power_of_two();
        if self.keys.len() < cap {
            self.keys.resize(cap, EMPTY_KEY);
            self.vals.resize(cap, 0);
        }
        self.keys[..cap].fill(EMPTY_KEY);
        self.mask = cap - 1;
    }

    #[inline(always)]
    fn hash(k: u32) -> usize {
        (k.wrapping_mul(0x9e37_79b9) >> 1) as usize
    }

    #[inline(always)]
    fn get(&self, k: u32) -> u32 {
        let mut idx = Self::hash(k) & self.mask;
        loop {
            let cur = self.keys[idx];
            if cur == k {
                return self.vals[idx];
            }
            if cur == EMPTY_KEY {
                return u32::MAX;
            }
            idx = (idx + 1) & self.mask;
        }
    }

    #[inline(always)]
    fn insert(&mut self, k: u32, v: u32) {
        let mut idx = Self::hash(k) & self.mask;
        let mut tomb = usize::MAX;
        loop {
            let cur = self.keys[idx];
            if cur == k {
                self.vals[idx] = v;
                return;
            }
            if cur == EMPTY_KEY {
                let put = if tomb != usize::MAX { tomb } else { idx };
                self.keys[put] = k;
                self.vals[put] = v;
                return;
            }
            if cur == TOMB_KEY && tomb == usize::MAX {
                tomb = idx;
            }
            idx = (idx + 1) & self.mask;
        }
    }

    #[inline(always)]
    fn remove(&mut self, k: u32) {
        let mut idx = Self::hash(k) & self.mask;
        loop {
            let cur = self.keys[idx];
            if cur == k {
                self.keys[idx] = TOMB_KEY;
                return;
            }
            if cur == EMPTY_KEY {
                return;
            }
            idx = (idx + 1) & self.mask;
        }
    }
}

struct FlatOnlineScratch {
    top_end: FlatMap,
    bot_end: FlatMap,
    ts: Vec<u32>,
    bs: Vec<u32>,
    lam: Vec<u32>,
    word: Vec<u32>,
    ready: Vec<u32>,
    buckets: [Vec<u32>; 21],
    surviving: Vec<(u32, u32, u32)>,
}

impl Default for FlatOnlineScratch {
    fn default() -> Self {
        FlatOnlineScratch {
            top_end: FlatMap::new(1024),
            bot_end: FlatMap::new(1024),
            ts: Vec::new(),
            bs: Vec::new(),
            lam: Vec::new(),
            word: Vec::new(),
            ready: Vec::new(),
            buckets: std::array::from_fn(|_| Vec::new()),
            surviving: Vec::new(),
        }
    }
}

/// Fast flat-table online scheduler: O(1) open-addressed boundary maps + 21 log2(cell)
/// buckets (no waiting map, no BTreeSet, no BinaryHeap).
fn run_flat_online_on_open(
    strips_raw: &[u32],
    image_order: &[u32],
    ns: usize,
    n: u32,
    m: u32,
    rules: &mut RuleArena,
    sc: &mut FlatOnlineScratch,
) -> GeoResult {
    let t0 = Instant::now();
    rules.clear();
    let max_nodes = 3 * ns + 3;
    sc.ts.clear();
    sc.bs.clear();
    sc.lam.clear();
    sc.word.clear();
    sc.ready.clear();
    for b in &mut sc.buckets {
        b.clear();
    }
    sc.top_end.reset(max_nodes * 2);
    sc.bot_end.reset(max_nodes * 2);

    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        sc.ts.push(m + s.from);
        sc.bs.push(m + s.to);
        sc.lam.push(s.width);
        sc.word.push(literal(s.byte));
        sc.top_end.insert(m + s.from + s.width, i as u32);
        sc.bot_end.insert(m + s.to + s.width, i as u32);
    }
    let mut x = 0u32;
    let mut bpos = 0u32;
    for i in 0..ns {
        let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
        if x < s.from {
            let w = s.from - x;
            let id = sc.lam.len() as u32;
            sc.ts.push(m + x);
            sc.bs.push(bpos);
            sc.lam.push(w);
            sc.word.push(0);
            sc.top_end.insert(m + x + w, id);
            sc.bot_end.insert(bpos + w, id);
            bpos += w;
        }
        x = s.from + s.width;
    }
    if x < n {
        let w = n - x;
        let id = sc.lam.len() as u32;
        sc.ts.push(m + x);
        sc.bs.push(bpos);
        sc.lam.push(w);
        sc.word.push(0);
        sc.top_end.insert(m + x + w, id);
        sc.bot_end.insert(bpos + w, id);
        bpos += w;
    }
    assert_eq!(bpos, m);

    x = 0;
    let mut tpos = 0u32;
    for &i in image_order.iter() {
        let idx = i as usize;
        let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
        if x < s.to {
            let w = s.to - x;
            let id = sc.lam.len() as u32;
            sc.ts.push(tpos);
            sc.bs.push(m + x);
            sc.lam.push(w);
            sc.word.push(0);
            sc.top_end.insert(tpos + w, id);
            sc.bot_end.insert(m + x + w, id);
            tpos += w;
        }
        x = s.to + s.width;
    }
    if x < n {
        let w = n - x;
        let id = sc.lam.len() as u32;
        sc.ts.push(tpos);
        sc.bs.push(m + x);
        sc.lam.push(w);
        sc.word.push(0);
        sc.top_end.insert(tpos + w, id);
        sc.bot_end.insert(m + x + w, id);
        tpos += w;
    }
    assert_eq!(tpos, m);

    // Seed initial coincidences into `ready` (for free check) and log2 buckets.
    let mut bucket_mask: u32 = 0;
    let nodes = sc.lam.len();
    for i in 0..nodes {
        let c = sc.ts[i] + sc.lam[i];
        if c > m {
            let b = sc.bot_end.get(c);
            if b != u32::MAX {
                sc.ready.push(c);
                let cell = sc.lam[i].min(sc.lam[b as usize]);
                let bk = (31 - cell.leading_zeros()).min(20) as usize;
                sc.buckets[bk].push(c);
                bucket_mask |= 1 << bk;
            }
        }
    }
    let setup_ns = t0.elapsed().as_nanos() as u64;

    let t1 = Instant::now();
    let mut remaining = n;
    let mut steps = 0usize;
    let mut not_reducing = 0usize;
    let mut fresh = [0u32; 3];

    while remaining > 0 {
        let mut pick = u32::MAX;
        while let Some(c) = sc.ready.pop() {
            let a = sc.top_end.get(c);
            if a == u32::MAX {
                continue;
            }
            let b = sc.bot_end.get(c);
            if b == u32::MAX {
                continue;
            }
            let (la, lb) = (sc.lam[a as usize], sc.lam[b as usize]);
            let is_free = if a == b || la == lb {
                true
            } else if la > lb {
                let p = sc.bs[a as usize] + la - lb;
                p < m || sc.top_end.get(p) != u32::MAX
            } else {
                let p = sc.ts[b as usize] + lb - la;
                p < m || sc.bot_end.get(p) != u32::MAX
            };
            if is_free {
                pick = c;
                break;
            }
        }
        let c = if pick != u32::MAX {
            pick
        } else {
            // Pick from the highest non-empty log2(cell) bucket.
            let mut found = u32::MAX;
            while bucket_mask != 0 {
                let bk = (31 - bucket_mask.leading_zeros()) as usize;
                if let Some(cand) = sc.buckets[bk].pop() {
                    let a = sc.top_end.get(cand);
                    if a == u32::MAX {
                        continue;
                    }
                    let b = sc.bot_end.get(cand);
                    if b == u32::MAX {
                        continue;
                    }
                    let cell = sc.lam[a as usize].min(sc.lam[b as usize]);
                    let real_bk = (31 - cell.leading_zeros()).min(20) as usize;
                    if real_bk < bk {
                        sc.buckets[real_bk].push(cand);
                        bucket_mask |= 1 << real_bk;
                        continue;
                    }
                    found = cand;
                    break;
                } else {
                    bucket_mask &= !(1 << bk);
                }
            }
            assert_ne!(found, u32::MAX);
            found
        };

        let a = sc.top_end.get(c);
        let b = sc.bot_end.get(c);
        let (ai, bi) = (a as usize, b as usize);
        let (la, lb) = (sc.lam[ai], sc.lam[bi]);
        let w = la.min(lb);
        remaining -= w;
        steps += 1;

        let nfresh: usize;
        if a == b {
            sc.top_end.remove(c);
            sc.bot_end.remove(c);
            sc.lam[ai] = 0;
            fresh[0] = c - la;
            nfresh = 1;
        } else {
            let nw = rules.join(sc.word[ai], sc.word[bi]);
            if la > lb {
                sc.word[bi] = nw;
                sc.top_end.remove(c);
                sc.bot_end.remove(c);
                sc.top_end.insert(c - lb, a);
                let p = sc.bs[ai] + la - lb;
                if p > m && sc.top_end.get(p) == u32::MAX {
                    not_reducing += 1;
                }
                sc.lam[ai] = la - lb;
                sc.bs[bi] = p;
                sc.bot_end.insert(p + lb, b);
                sc.bot_end.insert(p, a);
                fresh = [c - lb, p, p + lb];
                nfresh = 3;
            } else if la < lb {
                sc.word[ai] = nw;
                sc.top_end.remove(c);
                sc.bot_end.remove(c);
                sc.bot_end.insert(c - la, b);
                let p = sc.ts[bi] + lb - la;
                if p > m && sc.bot_end.get(p) == u32::MAX {
                    not_reducing += 1;
                }
                sc.lam[bi] = lb - la;
                sc.ts[ai] = p;
                sc.top_end.insert(p + la, a);
                sc.top_end.insert(p, b);
                fresh = [c - la, p, p + la];
                nfresh = 3;
            } else {
                sc.word[bi] = nw;
                sc.top_end.remove(c);
                sc.bot_end.remove(c);
                sc.lam[ai] = 0;
                sc.bs[bi] = sc.bs[ai];
                sc.bot_end.insert(sc.bs[ai] + la, b);
                fresh[0] = c - la;
                fresh[1] = sc.bs[ai] + la;
                nfresh = 2;
            }
        }

        for &p in &fresh[..nfresh] {
            if p > m {
                let ta = sc.top_end.get(p);
                if ta != u32::MAX {
                    let tb = sc.bot_end.get(p);
                    if tb != u32::MAX {
                        sc.ready.push(p);
                        let cell = sc.lam[ta as usize].min(sc.lam[tb as usize]);
                        let bk = (31 - cell.leading_zeros()).min(20) as usize;
                        sc.buckets[bk].push(p);
                        bucket_mask |= 1 << bk;
                    }
                }
            }
        }
    }

    sc.surviving.clear();
    for i in 0..nodes {
        let w = sc.lam[i];
        if w > 0 {
            sc.surviving.push((sc.ts[i], w, i as u32));
        }
    }
    sc.surviving.sort_unstable_by_key(|&(t, _, _)| t);
    let out_nodes: Vec<(u32, u32, u32)> = sc
        .surviving
        .iter()
        .map(|&(_, w, id)| (w, sc.bs[id as usize], sc.word[id as usize]))
        .collect();
    let steps_ns = t1.elapsed().as_nanos() as u64;

    GeoResult {
        surviving: out_nodes,
        steps,
        floor: steps - not_reducing,
        rules: rules.a.len() / RULE,
        setup_ns,
        steps_ns,
    }
}

// ---------------------------------------------------------------------------
// OpenMirror: mirrors production Kernel with fine-grained A6b sub-phase timers
// and pluggable induction orders
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
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
            }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
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
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[derive(Default, Clone, Copy)]
struct SplitTimings {
    a6a_ns: u64,
    a6b_pre_ns: u64,
    a6b_nodes_ns: u64,
    a6b_steps_ns: u64,
    a6b_dict_ns: u64,
    a6c_ns: u64,
    a7_ns: u64,
}

impl SplitTimings {
    fn add(&mut self, o: &SplitTimings) {
        self.a6a_ns += o.a6a_ns;
        self.a6b_pre_ns += o.a6b_pre_ns;
        self.a6b_nodes_ns += o.a6b_nodes_ns;
        self.a6b_steps_ns += o.a6b_steps_ns;
        self.a6b_dict_ns += o.a6b_dict_ns;
        self.a6c_ns += o.a6c_ns;
        self.a7_ns += o.a7_ns;
    }
    fn a6b_setup_ns(&self) -> u64 {
        self.a6b_pre_ns + self.a6b_nodes_ns
    }
    fn a6b_total_ns(&self) -> u64 {
        self.a6b_pre_ns + self.a6b_nodes_ns + self.a6b_steps_ns + self.a6b_dict_ns
    }
    fn c_total_ns(&self) -> u64 {
        self.a6a_ns + self.a6b_total_ns() + self.a6c_ns
    }
}

#[derive(Default, Clone, Copy)]
struct BlockCounts {
    n: usize,
    runs: usize,
    retained: usize,
    thick: usize,
    open_nodes: usize,
    steps: usize,
    contracted_steps: usize,
    divisions: usize,
    floor: usize,
    rules: usize,
    bands: usize,
    words: usize,
    spirals: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Order {
    RightProd,
    Left,
    TwoSidedEqMax,
    TwoSidedAlt,
    RightDiv,
    RightZorich,
    Monodromy1279,
    MonodromyFreeRight,
    MonodromyWeakMax,
    OnlineFlat,
}

impl Order {
    fn name(self) -> &'static str {
        match self {
            Order::RightProd => "right_prod",
            Order::Left => "left",
            Order::TwoSidedEqMax => "two_sided_eq_max",
            Order::TwoSidedAlt => "two_sided_alt",
            Order::RightDiv => "right_div",
            Order::RightZorich => "right_zorich",
            Order::Monodromy1279 => "monodromy_1279",
            Order::MonodromyFreeRight => "monodromy_free_right",
            Order::MonodromyWeakMax => "monodromy_weak_max",
            Order::OnlineFlat => "online_flat",
        }
    }

    fn uses_mid(self) -> bool {
        matches!(
            self,
            Order::Left | Order::TwoSidedEqMax | Order::TwoSidedAlt
        )
    }

    fn is_geo(self) -> bool {
        matches!(
            self,
            Order::Monodromy1279
                | Order::MonodromyFreeRight
                | Order::MonodromyWeakMax
                | Order::OnlineFlat
        )
    }
}

#[derive(Default)]
struct OpenMirror {
    stack: Vec<u32>,
    predecessor: Option<u32>,
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    geo_rules: Option<RuleArena>,
    flat_scratch: FlatOnlineScratch,
}

impl OpenMirror {
    fn nodes_most(ns: usize, extra_sentinel: bool) -> usize {
        3 * ns + 3 + usize::from(extra_sentinel)
    }

    fn arena_words(ns: usize, m: usize, extra_sentinel: bool) -> usize {
        let most = Self::nodes_most(ns, extra_sentinel);
        let stride = if most > NARROW_NODES {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * most).max(m)
    }

    /// Run a single block through the Open pipeline with the chosen induction order.
    /// If `TIME_SPLIT = true`, records fine-grained sub-phase timings; if `false`,
    /// skips internal timers (for overhead verification).
    #[allow(clippy::too_many_arguments)]
    fn run_block<const TIME_SPLIT: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        order: Order,
        out: &mut Vec<u8>,
    ) -> Option<(BlockCounts, SplitTimings)> {
        let n32 = n as u32;
        let mut tm = SplitTimings::default();

        // --- A6a: first pass ---
        let t_a6a = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        out.resize(n + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut out[..], &mut self.spill, cap.m),
            edges: &mut self.edges,
            ns: 0,
            cap_strips: cap.strips,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take(c, pos, width, p, LIMIT);
        });
        let ns = pass.ns;
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over);
        if let Some(t) = t_a6a {
            tm.a6a_ns = t.elapsed().as_nanos() as u64;
        }
        if over || f.m > cap.m || ns > cap.strips {
            return None;
        }

        // --- A6b pre-setup ---
        let t_pre = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
            runs,
        } = f;
        let mut stats = BlockCounts {
            n,
            runs,
            retained: m as usize,
            thick: ns,
            ..BlockCounts::default()
        };
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n32);
        let first = starts(all);
        let mut write = starts(thick);
        let extra_sentinel = order.uses_mid();
        let need = Self::arena_words(ns, m as usize, extra_sentinel);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
        let image_order = &mut rest_edges[..ns];
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let a = usize::from(byte);
            sw[1] += first[a];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        if let Some(t) = t_pre {
            tm.a6b_pre_ns = t.elapsed().as_nanos() as u64;
        }

        // --- A6b endpoint (nodes + steps + dict) ---
        let cursors = if order.is_geo() {
            self.induced_geo(ns, n32, &thin, order, &mut stats, &mut tm)
        } else {
            let most = Self::nodes_most(ns, extra_sentinel);
            if most > NARROW_NODES {
                self.induced_arena::<Wide, TIME_SPLIT>(
                    ns, n32, &thin, most, order, &mut stats, &mut tm,
                )
            } else {
                self.induced_arena::<Narrow, TIME_SPLIT>(
                    ns, n32, &thin, most, order, &mut stats, &mut tm,
                )
            }
        };
        stats.bands = self.bands.len();

        // --- A6c: retained-edge wiring ---
        let t_a6c = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        self.predecessor = None;
        if !wide && m >= 65_536 {
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut predecessor = mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = QueueReader::new(out, &self.spill, &heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor(
                        edges,
                        band.dst() as usize,
                        (band.end() - k) as usize,
                        band.code() << bits,
                        mark,
                        &mut predecessor,
                    );
                    k = band.end();
                }
            }
            self.predecessor = Some(predecessor);
        } else {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = QueueReader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = bend;
                }
            }
        }
        if let Some(t) = t_a6c {
            tm.a6c_ns = t.elapsed().as_nanos() as u64;
        }

        // --- A7: emission walk ---
        let t_a7 = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
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
        if let Some(t) = t_a7 {
            tm.a7_ns = t.elapsed().as_nanos() as u64;
        }

        Some((stats, tm))
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_arena<L: Links, const TIME_SPLIT: bool>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        most: usize,
        order: Order,
        stats: &mut BlockCounts,
        tm: &mut SplitTimings,
    ) -> [usize; 256] {
        let t_nodes = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        let use_mid = order.uses_mid();
        let OpenMirror {
            stack,
            bands,
            spelling,
            edges,
            inline16,
            lens,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= 4 * ns + nodes_words);
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0); // sentinel 0
        if use_mid {
            e.node(0, 0); // sentinel MID = 1
        }
        let strip_offset = e.nodes;
        let mut widest = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        let (mut terminals, mut ord) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            if x < s.from {
                widest = widest.max(s.from - x);
                let id = e.node(s.from - x, 0);
                e.append_to(&mut ord[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut ord[0], i as u32 + strip_offset, 0);
            x = s.from + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut ord[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
            if x < s.to {
                widest = widest.max(s.to - x);
                let id = e.node(s.to - x, 0);
                e.append_to(&mut ord[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut ord[1], i + strip_offset, 1);
            x = s.to + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut ord[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        stats.open_nodes = e.nodes as usize - strip_offset as usize;
        for side in 0..2 {
            if use_mid {
                e.close_with_mid(side, terminals[side], ord[side]);
            } else {
                e.close(side, terminals[side], ord[side]);
            }
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        if let Some(t) = t_nodes {
            tm.a6b_nodes_ns = t.elapsed().as_nanos() as u64;
        }

        // --- A6b induction steps ---
        let t_steps = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        let (mut remaining, mut steps, mut contracted, mut divisions) = (n, 0usize, 0usize, 0usize);
        let spirals = widest > CERTIFIED_WIDTH;
        stats.spirals = spirals;
        loop {
            let finished = match order {
                Order::RightProd => {
                    if spirals {
                        e.eliminate_right::<true>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    } else {
                        e.eliminate_right::<false>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    }
                }
                Order::Left => {
                    if spirals {
                        e.eliminate_left::<true>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    } else {
                        e.eliminate_left::<false>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    }
                }
                Order::TwoSidedEqMax => {
                    if spirals {
                        e.eliminate_two_sided::<true, false>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    } else {
                        e.eliminate_two_sided::<false, false>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    }
                }
                Order::TwoSidedAlt => {
                    if spirals {
                        e.eliminate_two_sided::<true, true>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    } else {
                        e.eliminate_two_sided::<false, true>(
                            &mut remaining,
                            &mut steps,
                            &mut contracted,
                            &mut divisions,
                        )
                    }
                }
                Order::RightDiv => e.eliminate_right::<true>(
                    &mut remaining,
                    &mut steps,
                    &mut contracted,
                    &mut divisions,
                ),
                Order::RightZorich => {
                    e.eliminate_zorich(&mut remaining, &mut steps, &mut contracted, &mut divisions)
                }
                _ => unreachable!(),
            };
            if finished {
                break;
            }
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, 4 * ns + top + SPIRAL_RULES * RULE);
            e = Elim {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        stats.steps = steps;
        stats.contracted_steps = contracted;
        stats.divisions = divisions;
        stats.rules = (e.top - e.base) / RULE;
        if let Some(t) = t_steps {
            tm.a6b_steps_ns = t.elapsed().as_nanos() as u64;
        }

        // --- A6b dictionary / expansion ---
        let t_dict = if TIME_SPLIT {
            Some(Instant::now())
        } else {
            None
        };
        let end_sentinel = if use_mid { MID } else { 0 };
        e.assign_bottom(stats.retained as u32, end_sentinel);
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail
        };
        let (pairs, contents) = if cache.len() >= CACHE_WORDS {
            cache[..CACHE_WORDS].fill(0);
            cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
        } else {
            (&mut [][..], &mut [][..])
        };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16,
            lens,
            spelling,
            stack,
        };
        let mut cursors = [0; 256];
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                if node_word != 0 {
                    stats.words += 1;
                }
                let code = dictionary.code(&mut e, node_word, a as u8);
                bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, end_sentinel);
        if let Some(t) = t_dict {
            tm.a6b_dict_ns = t.elapsed().as_nanos() as u64;
        }
        cursors
    }

    fn induced_geo(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        order: Order,
        stats: &mut BlockCounts,
        tm: &mut SplitTimings,
    ) -> [usize; 256] {
        let mut rules = self
            .geo_rules
            .take()
            .unwrap_or_else(|| RuleArena::new(ns * 4 + 64));
        let (strips_raw, rest) = self.edges.split_at(3 * ns);
        let image_order = &rest[..ns];
        let m = stats.retained as u32;
        let res = match order {
            Order::Monodromy1279 => run_geo_on_open(
                strips_raw,
                image_order,
                ns,
                n,
                m,
                GeoPolicy::FreeThenLargest,
                &mut rules,
            ),
            Order::MonodromyFreeRight => run_geo_on_open(
                strips_raw,
                image_order,
                ns,
                n,
                m,
                GeoPolicy::FreeThenRight,
                &mut rules,
            ),
            Order::MonodromyWeakMax => run_geo_on_open(
                strips_raw,
                image_order,
                ns,
                n,
                m,
                GeoPolicy::WeakThenLargest,
                &mut rules,
            ),
            Order::OnlineFlat => run_flat_online_on_open(
                strips_raw,
                image_order,
                ns,
                n,
                m,
                &mut rules,
                &mut self.flat_scratch,
            ),
            _ => unreachable!(),
        };
        tm.a6b_nodes_ns = res.setup_ns;
        tm.a6b_steps_ns = res.steps_ns;
        stats.steps = res.steps;
        stats.contracted_steps = res.steps;
        stats.floor = res.floor;
        stats.rules = res.rules;
        stats.open_nodes = 0;

        // Dictionary / band emission over `res.surviving` using the exact same Dictionary.
        let t_dict = Instant::now();
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        if self.edges.len() < CACHE_WORDS {
            self.edges.resize(CACHE_WORDS, 0);
        }
        self.edges[..CACHE_WORDS].fill(0);
        let (pairs, contents) = self.edges[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
        let rules_len = rules.a.len();
        let mut e = Elim::<Narrow> {
            a: &mut rules.a,
            base: 0,
            top: rules_len,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        begin_inline(
            if n >= 65_536 { 1536 } else { 0 },
            &mut self.inline16,
            &mut self.lens,
        );
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16: &mut self.inline16,
            lens: &mut self.lens,
            spelling: &mut self.spelling,
            stack: &mut self.stack,
        };
        let mut cursors = [0; 256];
        let mut si = 0usize;
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let (node_width, node_bottom, node_word) = res.surviving[si];
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = node_bottom + in_node;
                if node_word != 0 {
                    stats.words += 1;
                }
                let code = dictionary.code(&mut e, node_word, a as u8);
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    si += 1;
                    in_node = 0;
                }
            }
        }
        assert_eq!(si, res.surviving.len());
        tm.a6b_dict_ns = t_dict.elapsed().as_nanos() as u64;
        self.geo_rules = Some(rules);
        cursors
    }
}

// ---------------------------------------------------------------------------
// Emitters (mirroring src/open.rs)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Corpus loading & summary helpers
// ---------------------------------------------------------------------------

struct BlockInput {
    index: usize,
    n: usize,
    orig_ptr: u32,
    records: Vec<u16>,
    cap: Cap,
    walked: Vec<u8>,
}

fn load_blocks(
    path: &str,
    forced: bool,
    every: usize,
    keep_walked: bool,
) -> (Vec<BlockInput>, usize, usize) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::new();
    let mut blocks = Vec::new();
    let mut total_blocks = 0usize;
    let mut total_rows = 0usize;
    let mut admitted_idx = 0usize;

    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        total_blocks += 1;
        total_rows += n;
        let recs = records_of(&ll);
        if let Some(cap) = open_cap(recs.len(), n, b.orig_ptr, forced) {
            if admitted_idx % every == 0 {
                let mut walked = Vec::new();
                if keep_walked {
                    build(&ll, &counts, &mut tt);
                    walk(&tt, b.orig_ptr, &mut walked);
                }
                blocks.push(BlockInput {
                    index: total_blocks - 1,
                    n,
                    orig_ptr: b.orig_ptr,
                    records: recs,
                    cap,
                    walked,
                });
            }
            admitted_idx += 1;
        }
        syms = b.syms;
    }
    (blocks, total_blocks, total_rows)
}

fn median_min_max(xs: &[f64]) -> (f64, f64, f64) {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let med = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    (med, v[0], v[v.len() - 1])
}

fn cmd_split(path: &str, forced: bool, repeats: usize) {
    let (blocks, total_blocks, total_rows) = load_blocks(path, forced, 1, true);
    println!(
        "# split: {path} (forced={forced}, repeats={repeats}, total_blocks={total_blocks}, total_rows={total_rows}, pre_admitted={})",
        blocks.len()
    );
    if blocks.is_empty() {
        println!("NO_ADMITTED_BLOCKS");
        return;
    }

    let mut kernel = OpenMirror::default();
    let mut out = Vec::new();

    // Repeat 0: verify byte-identity against bwt::walk + collect exact counts.
    let mut open_blocks = 0usize;
    let mut open_rows = 0usize;
    let mut sum_runs = 0usize;
    let mut sum_retained = 0usize;
    let mut sum_thick = 0usize;
    let mut sum_open_nodes = 0usize;
    let mut sum_steps = 0usize;
    let mut sum_rules = 0usize;
    let mut sum_bands = 0usize;
    let mut sum_words = 0usize;
    let mut spiral_blocks = 0usize;

    for b in &blocks {
        if let Some((st, _)) = kernel.run_block::<false>(
            &b.records,
            b.n,
            b.orig_ptr,
            b.cap,
            Order::RightProd,
            &mut out,
        ) {
            assert_eq!(out, b.walked, "mismatch on block {}", b.index);
            open_blocks += 1;
            open_rows += st.n;
            sum_runs += st.runs;
            sum_retained += st.retained;
            sum_thick += st.thick;
            sum_open_nodes += st.open_nodes;
            sum_steps += st.steps;
            sum_rules += st.rules;
            sum_bands += st.bands;
            sum_words += st.words;
            spiral_blocks += usize::from(st.spirals);
        }
    }
    if open_blocks == 0 {
        println!("NO_OPEN_BLOCKS_AFTER_CAP");
        return;
    }

    println!(
        "counts: open_blocks={open_blocks}/{total_blocks} open_rows={open_rows} runs={sum_runs} retained={sum_retained} thick_strips={sum_thick} open_nodes={sum_open_nodes} steps={sum_steps} rules={sum_rules} bands={sum_bands} words={sum_words} spiral_blocks={spiral_blocks} verified=OK"
    );
    println!(
        "ratios: r/n={:.4} retained/n={:.4} steps/run={:.4} steps/open_node={:.4} steps/row={:.5}",
        sum_runs as f64 / open_rows as f64,
        sum_retained as f64 / open_rows as f64,
        sum_steps as f64 / sum_runs as f64,
        if sum_open_nodes > 0 {
            sum_steps as f64 / sum_open_nodes as f64
        } else {
            0.0
        },
        sum_steps as f64 / open_rows as f64,
    );

    // Timed repeats (split-instrumented) + uninstrumented overhead check.
    let mut split_runs = Vec::with_capacity(repeats);
    let mut uninst_ns = Vec::with_capacity(repeats);
    let rows_f = open_rows as f64;

    for _ in 0..repeats {
        let mut tot = SplitTimings::default();
        for b in &blocks {
            if let Some((_, tm)) = kernel.run_block::<true>(
                &b.records,
                b.n,
                b.orig_ptr,
                b.cap,
                Order::RightProd,
                &mut out,
            ) {
                tot.add(&tm);
            }
        }
        split_runs.push(tot);

        let t0 = Instant::now();
        for b in &blocks {
            let _ = kernel.run_block::<false>(
                &b.records,
                b.n,
                b.orig_ptr,
                b.cap,
                Order::RightProd,
                &mut out,
            );
        }
        uninst_ns.push(t0.elapsed().as_nanos() as f64 / rows_f);
    }

    fn extract_stats(runs: &[SplitTimings], f: impl Fn(&SplitTimings) -> f64) -> (f64, f64, f64) {
        let v: Vec<f64> = runs.iter().map(f).collect();
        median_min_max(&v)
    }
    let a6a = extract_stats(&split_runs, |t| t.a6a_ns as f64 / rows_f);
    let pre = extract_stats(&split_runs, |t| t.a6b_pre_ns as f64 / rows_f);
    let nodes = extract_stats(&split_runs, |t| t.a6b_nodes_ns as f64 / rows_f);
    let setup = extract_stats(&split_runs, |t| t.a6b_setup_ns() as f64 / rows_f);
    let steps = extract_stats(&split_runs, |t| t.a6b_steps_ns as f64 / rows_f);
    let dict = extract_stats(&split_runs, |t| t.a6b_dict_ns as f64 / rows_f);
    let a6b = extract_stats(&split_runs, |t| t.a6b_total_ns() as f64 / rows_f);
    let a6c = extract_stats(&split_runs, |t| t.a6c_ns as f64 / rows_f);
    let c_tot = extract_stats(&split_runs, |t| t.c_total_ns() as f64 / rows_f);
    let a7 = extract_stats(&split_runs, |t| t.a7_ns as f64 / rows_f);
    let open_tot = extract_stats(&split_runs, |t| (t.c_total_ns() + t.a7_ns) as f64 / rows_f);
    let uninst = median_min_max(&uninst_ns);

    let sh_pre = extract_stats(&split_runs, |t| {
        100.0 * t.a6b_pre_ns as f64 / t.a6b_total_ns().max(1) as f64
    });
    let sh_nodes = extract_stats(&split_runs, |t| {
        100.0 * t.a6b_nodes_ns as f64 / t.a6b_total_ns().max(1) as f64
    });
    let sh_setup = extract_stats(&split_runs, |t| {
        100.0 * t.a6b_setup_ns() as f64 / t.a6b_total_ns().max(1) as f64
    });
    let sh_steps = extract_stats(&split_runs, |t| {
        100.0 * t.a6b_steps_ns as f64 / t.a6b_total_ns().max(1) as f64
    });
    let sh_dict = extract_stats(&split_runs, |t| {
        100.0 * t.a6b_dict_ns as f64 / t.a6b_total_ns().max(1) as f64
    });
    let ns_per_step = extract_stats(&split_runs, |t| {
        if sum_steps > 0 {
            t.a6b_steps_ns as f64 / sum_steps as f64
        } else {
            0.0
        }
    });

    println!("| part | ns/row median (min..max) | share of A6b % (min..max) |");
    println!("|---|---:|---:|");
    println!(
        "| A6a (first pass) | {:.4} ({:.4}..{:.4}) | — |",
        a6a.0, a6a.1, a6a.2
    );
    println!(
        "| A6b-setup total | {:.4} ({:.4}..{:.4}) | {:.2}% ({:.2}..{:.2}%) |",
        setup.0, setup.1, setup.2, sh_setup.0, sh_setup.1, sh_setup.2
    );
    println!(
        "|   A6b-pre (image_order) | {:.4} ({:.4}..{:.4}) | {:.2}% ({:.2}..{:.2}%) |",
        pre.0, pre.1, pre.2, sh_pre.0, sh_pre.1, sh_pre.2
    );
    println!(
        "|   A6b-nodes (list build) | {:.4} ({:.4}..{:.4}) | {:.2}% ({:.2}..{:.2}%) |",
        nodes.0, nodes.1, nodes.2, sh_nodes.0, sh_nodes.1, sh_nodes.2
    );
    println!(
        "| A6b-steps (induction) | {:.4} ({:.4}..{:.4}) | {:.2}% ({:.2}..{:.2}%) |",
        steps.0, steps.1, steps.2, sh_steps.0, sh_steps.1, sh_steps.2
    );
    println!(
        "| A6b-dict (expand+dedup) | {:.4} ({:.4}..{:.4}) | {:.2}% ({:.2}..{:.2}%) |",
        dict.0, dict.1, dict.2, sh_dict.0, sh_dict.1, sh_dict.2
    );
    println!(
        "| A6b total | {:.4} ({:.4}..{:.4}) | 100.00% |",
        a6b.0, a6b.1, a6b.2
    );
    println!(
        "| A6c (wiring) | {:.4} ({:.4}..{:.4}) | — |",
        a6c.0, a6c.1, a6c.2
    );
    println!(
        "| C total (A6a+b+c) | {:.4} ({:.4}..{:.4}) | — |",
        c_tot.0, c_tot.1, c_tot.2
    );
    println!(
        "| A7 (emit walk) | {:.4} ({:.4}..{:.4}) | — |",
        a7.0, a7.1, a7.2
    );
    println!(
        "| Open total (C+E) | {:.4} ({:.4}..{:.4}) | uninst {:.4} ({:.4}..{:.4}), ratio {:.4} |",
        open_tot.0,
        open_tot.1,
        open_tot.2,
        uninst.0,
        uninst.1,
        uninst.2,
        open_tot.0 / uninst.0
    );
    println!(
        "step_cost: {:.2} ns/step ({:.2}..{:.2}), A6b/C={:.2}%, A6b/(C+E)={:.2}%",
        ns_per_step.0,
        ns_per_step.1,
        ns_per_step.2,
        100.0 * a6b.0 / c_tot.0,
        100.0 * a6b.0 / open_tot.0
    );
}

fn cmd_orders(path: &str, forced: bool, repeats: usize, every: usize, per_block: bool) {
    let (blocks, total_blocks, _) = load_blocks(path, forced, every, true);
    println!(
        "# orders: {path} (forced={forced}, repeats={repeats}, every={every}, sampled_blocks={}/{total_blocks})",
        blocks.len()
    );
    if blocks.is_empty() {
        println!("NO_ADMITTED_BLOCKS");
        return;
    }

    let orders = [
        Order::RightProd,
        Order::Left,
        Order::TwoSidedEqMax,
        Order::TwoSidedAlt,
        Order::RightDiv,
        Order::RightZorich,
        Order::Monodromy1279,
        Order::MonodromyFreeRight,
        Order::MonodromyWeakMax,
        Order::OnlineFlat,
    ];

    // First pass: check production baseline step counts per block and verify byte-identity.
    let mut kernel = OpenMirror::default();
    let mut out = Vec::new();
    let mut prod_steps_per_block = Vec::with_capacity(blocks.len());
    let mut active_blocks = Vec::new();
    for b in &blocks {
        if let Some((st, _)) = kernel.run_block::<false>(
            &b.records,
            b.n,
            b.orig_ptr,
            b.cap,
            Order::RightProd,
            &mut out,
        ) {
            assert_eq!(out, b.walked, "prod baseline mismatch on block {}", b.index);
            prod_steps_per_block.push(st.steps);
            active_blocks.push(b);
        }
    }
    if active_blocks.is_empty() {
        println!("NO_OPEN_BLOCKS_AFTER_CAP");
        return;
    }

    let open_rows: usize = active_blocks.iter().map(|b| b.n).sum();
    let rows_f = open_rows as f64;
    let mut total_runs = 0usize;
    let mut total_open_nodes = 0usize;
    let mut total_floor = 0usize;

    // Collect baseline ns/row for ratio reporting.
    let mut base_a6b_med = 0.0f64;
    let mut base_steps_med = 0.0f64;

    println!(
        "| order | steps | steps/run | steps/node | contracted/run | divs | rules | bands | steps ns/row (min..max) | A6b ns/row (min..max) | ns/step | A6b ratio | verified |"
    );
    println!("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|");

    for &ord in &orders {
        let mut tot_steps = 0usize;
        let mut tot_contracted = 0usize;
        let mut tot_divs = 0usize;
        let mut tot_rules = 0usize;
        let mut tot_bands = 0usize;
        let mut tot_floor = 0usize;
        let mut all_ok = true;

        // Pass 0: verification + exact counts (+ optional per-block dump).
        let mut per_blk_info = Vec::new();
        for (bi, b) in active_blocks.iter().enumerate() {
            let (st, tm) = kernel
                .run_block::<true>(&b.records, b.n, b.orig_ptr, b.cap, ord, &mut out)
                .expect("active block must stay admitted");
            let ok = out == b.walked;
            if !ok {
                all_ok = false;
                eprintln!(
                    "MISMATCH order={} block={} (n={})",
                    ord.name(),
                    b.index,
                    b.n
                );
            }
            if ord == Order::RightProd {
                assert_eq!(st.steps, prod_steps_per_block[bi]);
                total_runs += st.runs;
                total_open_nodes += st.open_nodes;
            }
            if ord == Order::Monodromy1279 {
                total_floor += st.floor;
            }
            tot_steps += st.steps;
            tot_contracted += st.contracted_steps;
            tot_divs += st.divisions;
            tot_rules += st.rules;
            tot_bands += st.bands;
            tot_floor += st.floor;
            if per_block {
                per_blk_info.push((
                    b.index,
                    b.n,
                    st.runs,
                    st.steps,
                    st.contracted_steps,
                    tm.a6b_steps_ns,
                    tm.a6b_total_ns(),
                    ok,
                ));
            }
        }
        assert!(all_ok, "order {} failed byte-identity check", ord.name());

        // Timed repeats.
        let mut steps_ns_v = Vec::with_capacity(repeats);
        let mut a6b_ns_v = Vec::with_capacity(repeats);
        let mut ns_step_v = Vec::with_capacity(repeats);
        for _ in 0..repeats {
            let mut tot = SplitTimings::default();
            for b in &active_blocks {
                let (_, tm) = kernel
                    .run_block::<true>(&b.records, b.n, b.orig_ptr, b.cap, ord, &mut out)
                    .unwrap();
                tot.add(&tm);
            }
            steps_ns_v.push(tot.a6b_steps_ns as f64 / rows_f);
            a6b_ns_v.push(tot.a6b_total_ns() as f64 / rows_f);
            let denom = if ord == Order::RightZorich || ord == Order::RightDiv {
                tot_contracted.max(1) as f64
            } else {
                tot_steps.max(1) as f64
            };
            ns_step_v.push(tot.a6b_steps_ns as f64 / denom);
        }
        let s_mm = median_min_max(&steps_ns_v);
        let a_mm = median_min_max(&a6b_ns_v);
        let ns_st = median_min_max(&ns_step_v);
        if ord == Order::RightProd {
            base_a6b_med = a_mm.0;
            base_steps_med = s_mm.0;
        }
        let _ = (tot_floor, base_steps_med);
        println!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {} | {} | {} | {:.4} ({:.4}..{:.4}) | {:.4} ({:.4}..{:.4}) | {:.2} | {:.3}x | {} |",
            ord.name(),
            tot_steps,
            tot_steps as f64 / total_runs.max(1) as f64,
            if total_open_nodes > 0 {
                tot_steps as f64 / total_open_nodes as f64
            } else {
                0.0
            },
            tot_contracted as f64 / total_runs.max(1) as f64,
            tot_divs,
            tot_rules,
            tot_bands,
            s_mm.0,
            s_mm.1,
            s_mm.2,
            a_mm.0,
            a_mm.1,
            a_mm.2,
            ns_st.0,
            a_mm.0 / base_a6b_med,
            if all_ok { "exact" } else { "FAIL" }
        );

        if per_block {
            for (idx, n, runs, st, cst, sns, ans, ok) in per_blk_info {
                println!(
                    "  blk {idx}: n={n} runs={runs} steps/run={:.4} csteps/run={:.4} steps_ns/row={:.4} a6b_ns/row={:.4} ok={ok}",
                    st as f64 / runs.max(1) as f64,
                    cst as f64 / runs.max(1) as f64,
                    sns as f64 / n as f64,
                    ans as f64 / n as f64,
                );
            }
        }
    }

    println!(
        "floor (r2_E on Open's auxiliary IET): {} ({:.4}/run, {:.4}/open_node), open_rows={}, runs={}, open_nodes={}",
        total_floor,
        total_floor as f64 / total_runs.max(1) as f64,
        if total_open_nodes > 0 {
            total_floor as f64 / total_open_nodes as f64
        } else {
            0.0
        },
        open_rows,
        total_runs,
        total_open_nodes
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage:\n  {} split  FILE.bz2 [--forced] [--repeats N]\n  {} orders FILE.bz2 [--forced] [--repeats N] [--every K] [--per-block]",
            args[0], args[0]
        );
        std::process::exit(2);
    }
    let cmd = args[1].as_str();
    let path = args[2].as_str();
    let mut forced = false;
    let mut repeats = 5usize;
    let mut every = 1usize;
    let mut per_block = false;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--forced" => forced = true,
            "--per-block" => per_block = true,
            "--repeats" => {
                i += 1;
                repeats = args[i].parse().expect("repeats");
            }
            "--every" => {
                i += 1;
                every = args[i].parse().expect("every");
            }
            other => panic!("unknown arg: {other}"),
        }
        i += 1;
    }
    match cmd {
        "split" => cmd_split(path, forced, repeats),
        "orders" => cmd_orders(path, forced, repeats, every, per_block),
        _ => panic!("unknown command: {cmd}"),
    }
}
