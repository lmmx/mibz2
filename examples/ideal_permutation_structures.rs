#![forbid(unsafe_code)]
//! Ideal permutation structures diagnostic: measure consumer value of alternative
//! legal first-return / contraction / conjugate representations before searching
//! for a constructor.
//!
//! Smooth-zone diagnostic for claim `ideal-permutation-structures`
//! (`docs/journal/2026-11-24-algorithm-level/ideal-permutation-structures/`).
//! Zero edits to `src/`, zero `unsafe`.
//!
//! Usage:
//!   ideal_permutation_structures part_a
//!   ideal_permutation_structures exact  [FILE.bz2]
//!   ideal_permutation_structures bench  FILE.bz2 [--repeats N]
//!   ideal_permutation_structures all    FILE.bz2 [--repeats N]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs, endpoint.rs, outq.rs)
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
const INLINE: usize = 16;
const EDGE12_INLINE: usize = 12;
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

// Pre-registered train / held-out split on the 233 production-admitted wikidata blocks
const TRAIN_SPLIT_BLOCKS: usize = 117;

// Cache-eviction buffer for cold/streaming condition (64 MiB > 45 MiB L3)
const COLD_FLUSH_BYTES: usize = 64 * 1024 * 1024;

// Stage-1 i9-10900K reference anchors (wikidata x4, 1 worker)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_GATE_MS: f64 = I9_WALL_1W_MS * 0.03; // 39.18 ms
const I9_A6A_MS: f64 = 200.3;
const I9_A6B_MS: f64 = 216.0;
const I9_A6C_MS: f64 = 161.4;
const I9_C_BASE_MS: f64 = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS; // 577.7 ms
const I9_E_BASE_MS: f64 = 490.3;
const I9_CE_BASE_MS: f64 = I9_C_BASE_MS + I9_E_BASE_MS; // 1068.0 ms
const I9_8W_E_SCALE: f64 = 69.5 / 490.3; // character-stage1 k=8 emission share model

// ---------------------------------------------------------------------------
// Production Open constructor mirror (LIMIT=16, exact src/open.rs mirror)
// ---------------------------------------------------------------------------

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

#[inline(always)]
fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0; INLINE];
    slot[..4].copy_from_slice(&start.to_le_bytes());
    slot[4..8].copy_from_slice(&end.to_le_bytes());
    slot
}

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
    let end = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
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

#[inline(always)]
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

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
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
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
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

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
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

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
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
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        if width <= limit {
            self.piece::<QUEUE>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<QUEUE>(c, pos, width, false, p);
        } else {
            self.take_marked::<QUEUE>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<QUEUE>(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece<const QUEUE: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            if QUEUE {
                self.queues.push(a, f.m, w, &f.thin);
            }
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
struct ConstructNs {
    a6a: u64,
    a6b: u64,
    a6c: u64,
}

impl ConstructNs {
    fn total(&self) -> u64 {
        self.a6a + self.a6b + self.a6c
    }
    fn add(&mut self, o: ConstructNs) {
        self.a6a += o.a6a;
        self.a6b += o.a6b;
        self.a6c += o.a6c;
    }
}

#[derive(Default)]
struct OpenBuilder {
    stack: Vec<u32>,
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    predecessor: Option<u32>,
}

impl OpenBuilder {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
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
        let mut widest = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            if x < s.from {
                widest = widest.max(s.from - x);
                let id = e.node(s.from - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
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
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
            if x < s.to {
                widest = widest.max(s.to - x);
                let id = e.node(s.to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = s.to + s.width;
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
            grow(&mut self.edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        assert!(e.nodes as usize <= most);
        e.assign_bottom(m);
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = self.edges.split_at_mut(4 * ns);
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
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = self.bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                let dst = e.bottom(id) + in_node;
                let code = dictionary.code(&mut e, node_word, a as u8);
                self.bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        cursors
    }

    fn construct_timed(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        allow_bidi: bool,
        scratch_out: &mut Vec<u8>,
    ) -> (ConstructNs, usize, u32, u32) {
        let mut ns_t = ConstructNs::default();
        let t0 = Instant::now();
        scratch_out.resize(n as usize + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut scratch_out[..], &mut self.spill, u32::MAX),
            edges: &mut self.edges,
            ns: 0,
            cap_strips: usize::MAX,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, limit);
        });
        let ns = pass.ns;
        let (f, heads) = (pass.f, pass.queues.heads());
        let t1 = Instant::now();
        ns_t.a6a = t1.duration_since(t0).as_nanos() as u64;

        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        let first = starts(all);
        let mut write = starts(f.thick);
        let most = Self::nodes_most(ns);
        let wide_links = most > NARROW_NODES;
        let stride = if wide_links {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        let need = (4 * ns + (stride + RULE) * most).max(f.m as usize);
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
        let cursors = if wide_links {
            self.induced_with::<Wide>(ns, n, f.m, &f.thin)
        } else {
            self.induced_with::<Narrow>(ns, n, f.m, &f.thin)
        };
        let t2 = Instant::now();
        ns_t.a6b = t2.duration_since(t1).as_nanos() as u64;

        let m = f.m;
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
        if allow_bidi && !wide && m >= 65_536 {
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut predecessor = f.mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = Reader::new(scratch_out, &self.spill, &heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor(
                        edges,
                        band.dst() as usize,
                        (band.end() - k) as usize,
                        band.code() << bits,
                        f.mark,
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
                let mut queue = Reader::new(scratch_out, &self.spill, &heads, a);
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
        let t3 = Instant::now();
        ns_t.a6c = t3.duration_since(t2).as_nanos() as u64;
        (ns_t, m as usize, f.mark, bits)
    }

    fn snapshot_dict_rep(&self, n: usize, m: usize, mark: u32, bits: u32) -> DictReturnMap {
        DictReturnMap {
            n,
            m,
            mark,
            predecessor: self.predecessor,
            bits,
            edges: self.edges[..m].to_vec(),
            extra: self.extra.clone(),
            inline16: self.inline16.clone(),
            lens: self.lens.clone(),
            spelling: self.spelling.clone(),
            lanes: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Prebuilt representation interfaces & real charged consumers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Lane4 {
    starts: [u32; 4],
    byte_offs: [usize; 4],
    ends: [usize; 4],
    steps: [usize; 4],
}

#[derive(Clone)]
struct DictReturnMap {
    n: usize,
    m: usize,
    mark: u32,
    predecessor: Option<u32>,
    bits: u32,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    lanes: Option<Lane4>,
}

impl DictReturnMap {
    fn word_bytes(&self, code: usize) -> &[u8] {
        let len = self.lens[code];
        if len != LONG {
            &self.inline16[code][..usize::from(len)]
        } else {
            long_word(&self.inline16[code], &self.spelling)
        }
    }

    fn edges_bytes(&self) -> usize {
        4 * (self.edges.len() + self.extra.len()) + if self.lanes.is_some() { 48 } else { 0 }
    }

    fn dict_bytes(&self) -> usize {
        self.inline16.len() * INLINE + self.lens.len() + self.spelling.len()
    }

    fn rep_bytes(&self) -> usize {
        self.edges_bytes() + self.dict_bytes()
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct Edge16 {
    /// low 24 bits: next vertex index (< 2^20 <= 900,000); high 8 bits: len (1..=12 or 255 = LONG)
    meta: u32,
    /// inline word bytes (len <= 12), or (start: u32 LE, end: u32 LE) into `spelling` if len == 255
    payload: [u8; EDGE12_INLINE],
}

#[derive(Clone)]
struct Edge12ReturnMap {
    n: usize,
    m: usize,
    mark: u32,
    edges: Vec<Edge16>,
    spelling: Vec<u8>,
    lanes: Option<Lane4>,
    one_use_inline_edges: usize,
    total_inline_edges: usize,
}

impl Edge12ReturnMap {
    fn edges_bytes(&self) -> usize {
        self.edges.len() * std::mem::size_of::<Edge16>() + if self.lanes.is_some() { 48 } else { 0 }
    }

    fn dict_bytes(&self) -> usize {
        self.spelling.len()
    }

    fn rep_bytes(&self) -> usize {
        self.edges_bytes() + self.dict_bytes()
    }
}

#[inline(always)]
fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

/// Exact production `emit` (forward) from src/open.rs.
#[inline(never)]
fn consume_dict_fwd(rep: &DictReturnMap, out: &mut Vec<u8>) {
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = rep.mark;
    let mut j = mark;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
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

/// Exact production `emit_bidirectional` / `emit` from src/open.rs.
#[inline(never)]
fn consume_dict_bidi(rep: &DictReturnMap, out: &mut Vec<u8>) {
    let Some(before) = rep.predecessor else {
        consume_dict_fwd(rep, out);
        return;
    };
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = rep.mark;
    let edges = &rep.edges;
    let spelling = &rep.spelling;
    let inline = &rep.inline16;
    let lens = &rep.lens;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
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

/// 4-lane interleaved forward consumer on `DictReturnMap`.
/// Every lane charges every `edges[j]` load, `lens[code]` load, and output store,
/// with boundary-safe guards preventing any lane overrun into the next lane.
#[inline(never)]
fn consume_dict_4lane(rep: &DictReturnMap, out: &mut Vec<u8>) {
    let Some(l4) = rep.lanes else {
        consume_dict_fwd(rep, out);
        return;
    };
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;

    let (mut j0, mut j1, mut j2, mut j3) = (l4.starts[0], l4.starts[1], l4.starts[2], l4.starts[3]);
    let (mut a0, mut a1, mut a2, mut a3) = (
        l4.byte_offs[0],
        l4.byte_offs[1],
        l4.byte_offs[2],
        l4.byte_offs[3],
    );
    let (e0, e1, e2, e3) = (l4.ends[0], l4.ends[1], l4.ends[2], l4.ends[3]);
    let min_steps = l4.steps[0]
        .min(l4.steps[1])
        .min(l4.steps[2])
        .min(l4.steps[3]);

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn emit_step(
        edge: u32,
        idx: u32,
        bits: u32,
        extra: &[u32],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
        at: &mut usize,
        end: usize,
        out: &mut [u8],
    ) {
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[idx as usize]
        } as usize;
        let len = lens[code];
        if len != LONG && *at + INLINE <= end {
            out[*at..*at + INLINE].copy_from_slice(&inline16[code]);
            *at += usize::from(len);
        } else {
            let word = if len == LONG {
                long_word(&inline16[code], spelling)
            } else {
                &inline16[code][..usize::from(len)]
            };
            let take = word.len().min(end - *at);
            out[*at..*at + take].copy_from_slice(&word[..take]);
            *at += take;
        }
    }

    for _ in 0..min_steps {
        let ed0 = edges[j0 as usize];
        let ed1 = edges[j1 as usize];
        let ed2 = edges[j2 as usize];
        let ed3 = edges[j3 as usize];
        let (oj0, oj1, oj2, oj3) = (j0, j1, j2, j3);
        j0 = ed0 & mask;
        j1 = ed1 & mask;
        j2 = ed2 & mask;
        j3 = ed3 & mask;

        emit_step(
            ed0, oj0, bits, extra, inline16, lens, spelling, &mut a0, e0, out,
        );
        emit_step(
            ed1, oj1, bits, extra, inline16, lens, spelling, &mut a1, e1, out,
        );
        emit_step(
            ed2, oj2, bits, extra, inline16, lens, spelling, &mut a2, e2, out,
        );
        emit_step(
            ed3, oj3, bits, extra, inline16, lens, spelling, &mut a3, e3, out,
        );
    }

    // Finish any tail steps (at most 3 steps total across lanes)
    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let cur = rem_j[lane];
            let ed = edges[cur as usize];
            rem_j[lane] = ed & mask;
            emit_step(
                ed,
                cur,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                &mut rem_a[lane],
                l4.ends[lane],
                out,
            );
        }
    }
    out.truncate(n);
}

/// Real forward consumer on co-located 16-byte `Edge16` return map.
#[inline(never)]
fn consume_edge12_fwd(rep: &Edge12ReturnMap, out: &mut Vec<u8>) {
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mark = rep.mark;
    let mut j = mark;
    let edges = &rep.edges;
    let spelling = &rep.spelling;
    for _ in 0..edges.len() {
        let e = edges[j as usize];
        j = e.meta & 0x00FF_FFFF;
        let len = (e.meta >> 24) as u8;
        if len != LONG {
            out[at..at + EDGE12_INLINE].copy_from_slice(&e.payload);
            at += usize::from(len);
        } else {
            let start = u32::from_le_bytes([e.payload[0], e.payload[1], e.payload[2], e.payload[3]])
                as usize;
            let end = u32::from_le_bytes([e.payload[4], e.payload[5], e.payload[6], e.payload[7]])
                as usize;
            let word = &spelling[start..end];
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

/// 4-lane interleaved forward consumer on `Edge16` return map.
#[inline(never)]
fn consume_edge12_4lane(rep: &Edge12ReturnMap, out: &mut Vec<u8>) {
    let Some(l4) = rep.lanes else {
        consume_edge12_fwd(rep, out);
        return;
    };
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let edges = &rep.edges;
    let spelling = &rep.spelling;

    let (mut j0, mut j1, mut j2, mut j3) = (l4.starts[0], l4.starts[1], l4.starts[2], l4.starts[3]);
    let (mut a0, mut a1, mut a2, mut a3) = (
        l4.byte_offs[0],
        l4.byte_offs[1],
        l4.byte_offs[2],
        l4.byte_offs[3],
    );
    let (e0, e1, e2, e3) = (l4.ends[0], l4.ends[1], l4.ends[2], l4.ends[3]);
    let min_steps = l4.steps[0]
        .min(l4.steps[1])
        .min(l4.steps[2])
        .min(l4.steps[3]);

    #[inline(always)]
    fn emit_e16(e: Edge16, spelling: &[u8], at: &mut usize, end: usize, out: &mut [u8]) {
        let len = (e.meta >> 24) as u8;
        if len != LONG && *at + EDGE12_INLINE <= end {
            out[*at..*at + EDGE12_INLINE].copy_from_slice(&e.payload);
            *at += usize::from(len);
        } else if len != LONG {
            let take = usize::from(len).min(end - *at);
            out[*at..*at + take].copy_from_slice(&e.payload[..take]);
            *at += take;
        } else {
            let start = u32::from_le_bytes([e.payload[0], e.payload[1], e.payload[2], e.payload[3]])
                as usize;
            let stop = u32::from_le_bytes([e.payload[4], e.payload[5], e.payload[6], e.payload[7]])
                as usize;
            let word = &spelling[start..stop];
            let take = word.len().min(end - *at);
            out[*at..*at + take].copy_from_slice(&word[..take]);
            *at += take;
        }
    }

    for _ in 0..min_steps {
        let ed0 = edges[j0 as usize];
        let ed1 = edges[j1 as usize];
        let ed2 = edges[j2 as usize];
        let ed3 = edges[j3 as usize];
        j0 = ed0.meta & 0x00FF_FFFF;
        j1 = ed1.meta & 0x00FF_FFFF;
        j2 = ed2.meta & 0x00FF_FFFF;
        j3 = ed3.meta & 0x00FF_FFFF;

        emit_e16(ed0, spelling, &mut a0, e0, out);
        emit_e16(ed1, spelling, &mut a1, e1, out);
        emit_e16(ed2, spelling, &mut a2, e2, out);
        emit_e16(ed3, spelling, &mut a3, e3, out);
    }

    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let ed = edges[rem_j[lane] as usize];
            rem_j[lane] = ed.meta & 0x00FF_FFFF;
            emit_e16(ed, spelling, &mut rem_a[lane], l4.ends[lane], out);
        }
    }
    out.truncate(n);
}

// ---------------------------------------------------------------------------
// Offline construction of alternative representation families
// ---------------------------------------------------------------------------

fn compute_lanes_dict(rep: &DictReturnMap) -> Option<Lane4> {
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mut cycle_verts = Vec::with_capacity(rep.m);
    let mut cycle_offs = Vec::with_capacity(rep.m);
    let mut j = rep.mark;
    let mut at = 0usize;
    for _ in 0..rep.m {
        cycle_verts.push(j);
        cycle_offs.push(at);
        let edge = rep.edges[j as usize];
        let code = if rep.extra.is_empty() {
            edge >> bits
        } else {
            rep.extra[j as usize]
        } as usize;
        at += rep.word_bytes(code).len();
        j = edge & mask;
        if j == rep.mark || at >= rep.n {
            break;
        }
    }
    if j != rep.mark || at != rep.n || cycle_verts.len() < 64 {
        return None;
    }
    let total_steps = cycle_verts.len();
    let s1 = total_steps / 4;
    let s2 = total_steps / 2;
    let s3 = (3 * total_steps) / 4;
    let starts = [
        cycle_verts[0],
        cycle_verts[s1],
        cycle_verts[s2],
        cycle_verts[s3],
    ];
    let byte_offs = [
        cycle_offs[0],
        cycle_offs[s1],
        cycle_offs[s2],
        cycle_offs[s3],
    ];
    let ends = [byte_offs[1], byte_offs[2], byte_offs[3], rep.n + INLINE];
    let steps = [s1, s2 - s1, s3 - s2, total_steps - s3];
    Some(Lane4 {
        starts,
        byte_offs,
        ends,
        steps,
    })
}

fn compute_lanes_edge12(rep: &Edge12ReturnMap) -> Option<Lane4> {
    let mut cycle_verts = Vec::with_capacity(rep.m);
    let mut cycle_offs = Vec::with_capacity(rep.m);
    let mut j = rep.mark;
    let mut at = 0usize;
    for _ in 0..rep.m {
        cycle_verts.push(j);
        cycle_offs.push(at);
        let e = rep.edges[j as usize];
        let len = (e.meta >> 24) as u8;
        let wlen = if len != LONG {
            usize::from(len)
        } else {
            let start = u32::from_le_bytes([e.payload[0], e.payload[1], e.payload[2], e.payload[3]])
                as usize;
            let stop = u32::from_le_bytes([e.payload[4], e.payload[5], e.payload[6], e.payload[7]])
                as usize;
            stop - start
        };
        at += wlen;
        j = e.meta & 0x00FF_FFFF;
        if j == rep.mark || at >= rep.n {
            break;
        }
    }
    if j != rep.mark || at != rep.n || cycle_verts.len() < 64 {
        return None;
    }
    let total_steps = cycle_verts.len();
    let s1 = total_steps / 4;
    let s2 = total_steps / 2;
    let s3 = (3 * total_steps) / 4;
    let starts = [
        cycle_verts[0],
        cycle_verts[s1],
        cycle_verts[s2],
        cycle_verts[s3],
    ];
    let byte_offs = [
        cycle_offs[0],
        cycle_offs[s1],
        cycle_offs[s2],
        cycle_offs[s3],
    ];
    let ends = [byte_offs[1], byte_offs[2], byte_offs[3], rep.n + INLINE];
    let steps = [s1, s2 - s1, s3 - s2, total_steps - s3];
    Some(Lane4 {
        starts,
        byte_offs,
        ends,
        steps,
    })
}

/// Extract unpacked `(next, code)` arrays from a forward `DictReturnMap`.
fn unpack_fwd_graph(rep: &DictReturnMap) -> (Vec<u32>, Vec<u32>) {
    let m = rep.m;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mut next = Vec::with_capacity(m);
    let mut code = Vec::with_capacity(m);
    for u in 0..m {
        let e = rep.edges[u];
        next.push(e & mask);
        code.push(if rep.extra.is_empty() {
            e >> bits
        } else {
            rep.extra[u]
        });
    }
    (next, code)
}

/// Family 2: Conjugate row-renumbering `sigma o T o sigma^-1`.
/// `sigma[old_u] = new_u` (bijection on `0..m`).
fn conjugate_dict_rep(rep: &DictReturnMap, sigma: &[u32]) -> DictReturnMap {
    let m = rep.m;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let wide = !rep.extra.is_empty();
    let mut new_edges = vec![0u32; m];
    let mut new_extra = if wide { vec![0u32; m] } else { Vec::new() };
    for old_u in 0..m {
        let e = rep.edges[old_u];
        let old_v = (e & mask) as usize;
        let c = if wide { rep.extra[old_u] } else { e >> bits };
        let new_u = sigma[old_u] as usize;
        let new_v = sigma[old_v];
        if wide {
            new_edges[new_u] = new_v;
            new_extra[new_u] = c;
        } else {
            new_edges[new_u] = (c << bits) | new_v;
        }
    }
    DictReturnMap {
        n: rep.n,
        m,
        mark: sigma[rep.mark as usize],
        predecessor: None,
        bits,
        edges: new_edges,
        extra: new_extra,
        inline16: rep.inline16.clone(),
        lens: rep.lens.clone(),
        spelling: rep.spelling.clone(),
        lanes: None,
    }
}

/// Family 2a: Structure-derived conjugation sorting vertices by `(code_freq_rank, code, old_u)`.
fn build_f2a_conj_code_order(base: &DictReturnMap) -> DictReturnMap {
    let m = base.m;
    let (_, codes) = unpack_fwd_graph(base);
    let mut freq = vec![0u32; base.lens.len()];
    for &c in &codes {
        freq[c as usize] += 1;
    }
    let mut order: Vec<u32> = (0..m as u32).collect();
    order.sort_unstable_by(|&a, &b| {
        let ca = codes[a as usize] as usize;
        let cb = codes[b as usize] as usize;
        freq[cb]
            .cmp(&freq[ca])
            .then_with(|| ca.cmp(&cb))
            .then_with(|| a.cmp(&b))
    });
    let mut sigma = vec![0u32; m];
    for (new_u, &old_u) in order.iter().enumerate() {
        sigma[old_u as usize] = new_u as u32;
    }
    conjugate_dict_rep(base, &sigma)
}

/// Family 2b: Offline traversal-order layout oracle (`0 -> 1 -> ... -> m-1 -> 0` along marked cycle).
/// Explicitly labelled as an upper bound on conjugation layout.
fn build_f2b_conj_cycle_oracle(base: &DictReturnMap) -> DictReturnMap {
    let m = base.m;
    let (next, _) = unpack_fwd_graph(base);
    let mut sigma = vec![u32::MAX; m];
    let mut cursor = 0u32;
    // Marked cycle first
    let mut cur = base.mark as usize;
    while sigma[cur] == u32::MAX {
        sigma[cur] = cursor;
        cursor += 1;
        cur = next[cur] as usize;
    }
    // Any remaining cycles (for multicycle non-BWT inputs)
    for start in 0..m {
        let mut c = start;
        while sigma[c] == u32::MAX {
            sigma[c] = cursor;
            cursor += 1;
            c = next[c] as usize;
        }
    }
    assert_eq!(cursor as usize, m);
    conjugate_dict_rep(base, &sigma)
}

/// Family 1: Frequency-gated RePair / digram contraction on the return-map graph.
/// Every contraction removes a vertex `v != mark` and creates a compound word with
/// reuse count `>= tau` (zero 1-use contractions!). Preserves ascending BWT row-rank
/// order of surviving vertices `R' <= R`.
fn build_f1_repair(base: &DictReturnMap, tau: u32, max_word_len: usize) -> DictReturnMap {
    const NONE: u32 = u32::MAX;
    let m = base.m;
    let mark = base.mark;
    let (mut next, mut code) = unpack_fwd_graph(base);
    let mut prev = vec![0u32; m];
    for u in 0..m {
        prev[next[u] as usize] = u as u32;
    }
    let mut alive = vec![true; m];

    let orig_dict_len = base.lens.len();
    let mut word_len: Vec<u32> = (0..orig_dict_len)
        .map(|c| base.word_bytes(c).len() as u32)
        .collect();
    // Grammar rules for newly created codes `c >= orig_dict_len`: `(left_code, right_code)`
    let mut merges: Vec<(u32, u32)> = Vec::new();

    struct PairRec {
        a: u32,
        b: u32,
        count: u32,
        head: u32,
        suppressed_at: u32,
    }

    let mut pairs: Vec<PairRec> = Vec::new();
    let mut pair_map: HashMap<u64, u32> = HashMap::with_capacity(m / 2 + 64);
    let mut occ_pair = vec![NONE; m];
    let mut occ_prev = vec![NONE; m];
    let mut occ_next = vec![NONE; m];
    let mut heap: BinaryHeap<(u32, u32)> = BinaryHeap::new();

    let remove_occ = |u: u32,
                      pairs: &mut [PairRec],
                      occ_pair: &mut [u32],
                      occ_prev: &mut [u32],
                      occ_next: &mut [u32]| {
        let ui = u as usize;
        let pid = occ_pair[ui];
        if pid == NONE {
            return;
        }
        let p = occ_prev[ui];
        let nx = occ_next[ui];
        if p == NONE {
            pairs[pid as usize].head = nx;
        } else {
            occ_next[p as usize] = nx;
        }
        if nx != NONE {
            occ_prev[nx as usize] = p;
        }
        occ_pair[ui] = NONE;
        occ_prev[ui] = NONE;
        occ_next[ui] = NONE;
        pairs[pid as usize].count -= 1;
    };

    let add_occ = |u: u32,
                   alive: &[bool],
                   next: &[u32],
                   code: &[u32],
                   word_len: &[u32],
                   pairs: &mut Vec<PairRec>,
                   pair_map: &mut HashMap<u64, u32>,
                   occ_pair: &mut [u32],
                   occ_prev: &mut [u32],
                   occ_next: &mut [u32],
                   heap: &mut BinaryHeap<(u32, u32)>| {
        let ui = u as usize;
        if !alive[ui] {
            return;
        }
        let v = next[ui];
        if v == mark || v == u {
            return;
        }
        let ca = code[ui];
        let cb = code[v as usize];
        let combined = word_len[ca as usize] as usize + word_len[cb as usize] as usize;
        if max_word_len > 0 && combined > max_word_len {
            return;
        }
        let key = (u64::from(ca) << 32) | u64::from(cb);
        let pid = match pair_map.get(&key) {
            Some(&id) => id,
            None => {
                let id = pairs.len() as u32;
                pairs.push(PairRec {
                    a: ca,
                    b: cb,
                    count: 0,
                    head: NONE,
                    suppressed_at: 0,
                });
                pair_map.insert(key, id);
                id
            }
        };
        let rec = &mut pairs[pid as usize];
        let old_head = rec.head;
        occ_pair[ui] = pid;
        occ_prev[ui] = NONE;
        occ_next[ui] = old_head;
        if old_head != NONE {
            occ_prev[old_head as usize] = u;
        }
        rec.head = u;
        rec.count += 1;
        if rec.count >= tau && rec.count > rec.suppressed_at {
            heap.push((rec.count, pid));
        }
    };

    for u in 0..m as u32 {
        add_occ(
            u,
            &alive,
            &next,
            &code,
            &word_len,
            &mut pairs,
            &mut pair_map,
            &mut occ_pair,
            &mut occ_prev,
            &mut occ_next,
            &mut heap,
        );
    }

    let mut visited_self = vec![0u32; m];
    let mut self_gen = 0u32;

    while let Some((cnt, pid)) = heap.pop() {
        let cur_cnt = pairs[pid as usize].count;
        if cnt != cur_cnt {
            if cur_cnt >= tau && cur_cnt > pairs[pid as usize].suppressed_at {
                heap.push((cur_cnt, pid));
            }
            continue;
        }
        if cur_cnt < tau {
            break;
        }
        let a = pairs[pid as usize].a;
        let b = pairs[pid as usize].b;

        // For `a == b`, verify at least `tau` non-overlapping occurrences exist before creating a rule.
        if a == b {
            self_gen += 1;
            let mut non_overlap = 0u32;
            let mut cur = pairs[pid as usize].head;
            while cur != NONE {
                let v = next[cur as usize];
                if visited_self[cur as usize] != self_gen && visited_self[v as usize] != self_gen {
                    visited_self[cur as usize] = self_gen;
                    visited_self[v as usize] = self_gen;
                    non_overlap += 1;
                }
                cur = occ_next[cur as usize];
            }
            if non_overlap < tau {
                pairs[pid as usize].suppressed_at = cur_cnt;
                continue;
            }
        }

        let new_code = word_len.len() as u32;
        let new_len = word_len[a as usize] + word_len[b as usize];
        word_len.push(new_len);
        merges.push((a, b));

        while pairs[pid as usize].head != NONE {
            let u = pairs[pid as usize].head;
            let v = next[u as usize];
            let p = prev[u as usize];
            let w = next[v as usize];
            if p != u && p != v {
                remove_occ(p, &mut pairs, &mut occ_pair, &mut occ_prev, &mut occ_next);
            }
            remove_occ(u, &mut pairs, &mut occ_pair, &mut occ_prev, &mut occ_next);
            remove_occ(v, &mut pairs, &mut occ_pair, &mut occ_prev, &mut occ_next);

            alive[v as usize] = false;
            next[u as usize] = w;
            prev[w as usize] = u;
            code[u as usize] = new_code;

            if p != u && p != v {
                add_occ(
                    p,
                    &alive,
                    &next,
                    &code,
                    &word_len,
                    &mut pairs,
                    &mut pair_map,
                    &mut occ_pair,
                    &mut occ_prev,
                    &mut occ_next,
                    &mut heap,
                );
            }
            add_occ(
                u,
                &alive,
                &next,
                &code,
                &word_len,
                &mut pairs,
                &mut pair_map,
                &mut occ_pair,
                &mut occ_prev,
                &mut occ_next,
                &mut heap,
            );
        }
    }

    // Compact surviving vertices `R'` in ascending original BWT row-rank order
    // and materialize only used dictionary entries densely (zero dead entries / zero slack).
    let mut old_to_new = vec![NONE; m];
    let mut surv_old = Vec::with_capacity(m);
    for (u, &is_alive) in alive.iter().enumerate() {
        if is_alive {
            old_to_new[u] = surv_old.len() as u32;
            surv_old.push(u as u32);
        }
    }
    let new_m = surv_old.len();
    let new_mark = old_to_new[mark as usize];
    assert_ne!(new_mark, NONE);

    // Expand any code recursively via iterative stack
    let expand_code = |c: u32, out_buf: &mut Vec<u8>| {
        out_buf.clear();
        let mut st = vec![c];
        while let Some(id) = st.pop() {
            if (id as usize) < orig_dict_len {
                out_buf.extend_from_slice(base.word_bytes(id as usize));
            } else {
                let (l, r) = merges[id as usize - orig_dict_len];
                st.push(r);
                st.push(l);
            }
        }
    };

    let mut code_remap = vec![NONE; word_len.len()];
    let mut inline16: Vec<[u8; INLINE]> = Vec::new();
    let mut lens: Vec<u8> = Vec::new();
    let mut spelling: Vec<u8> = Vec::new();
    let mut content_dedup: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut tmp = Vec::new();

    let mut new_codes = Vec::with_capacity(new_m);
    for &ou in &surv_old {
        let old_c = code[ou as usize] as usize;
        let mapped = if code_remap[old_c] != NONE {
            code_remap[old_c]
        } else {
            expand_code(old_c as u32, &mut tmp);
            let cid = if let Some(&existing) = content_dedup.get(&tmp) {
                existing
            } else {
                let id = lens.len() as u32;
                if tmp.len() <= INLINE {
                    let mut slot = [0u8; INLINE];
                    slot[..tmp.len()].copy_from_slice(&tmp);
                    inline16.push(slot);
                    lens.push(tmp.len() as u8);
                } else {
                    let start = spelling.len() as u32;
                    spelling.extend_from_slice(&tmp);
                    let end = spelling.len() as u32;
                    inline16.push(long_slot(start, end));
                    lens.push(LONG);
                }
                content_dedup.insert(tmp.clone(), id);
                id
            };
            code_remap[old_c] = cid;
            cid
        };
        new_codes.push(mapped);
    }

    pack_dict_return_map(
        base.n,
        new_m,
        new_mark,
        &surv_old,
        &next,
        &old_to_new,
        &new_codes,
        inline16,
        lens,
        spelling,
    )
}

#[allow(clippy::too_many_arguments)]
fn pack_dict_return_map(
    n: usize,
    new_m: usize,
    new_mark: u32,
    surv_old: &[u32],
    next: &[u32],
    old_to_new: &[u32],
    new_codes: &[u32],
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
) -> DictReturnMap {
    let bits = (u32::BITS - (new_m.max(1) as u32 - 1).leading_zeros()).max(1);
    let wide = lens.len().saturating_sub(1) > (u32::MAX >> bits) as usize;
    let mut edges = Vec::with_capacity(new_m);
    let mut extra = if wide {
        Vec::with_capacity(new_m)
    } else {
        Vec::new()
    };
    for (ni, &ou) in surv_old.iter().enumerate() {
        let nv = old_to_new[next[ou as usize] as usize];
        let c = new_codes[ni];
        if wide {
            edges.push(nv);
            extra.push(c);
        } else {
            edges.push((c << bits) | nv);
        }
    }
    DictReturnMap {
        n,
        m: new_m,
        mark: new_mark,
        predecessor: None,
        bits,
        edges,
        extra,
        inline16,
        lens,
        spelling,
        lanes: None,
    }
}

/// Family 3a: Greedy `<= 16 B` short-word chain contraction capped at `d_max` dictionary entries,
/// through the standard dictionary-indexed `emit` interface, preserving ascending row-rank order.
fn build_f3a_dict_capped_fuse(base: &DictReturnMap, d_max: usize) -> DictReturnMap {
    let m = base.m;
    let mark = base.mark;
    let (mut next, orig_codes) = unpack_fwd_graph(base);
    let mut alive = vec![true; m];

    // Build initial compacted dictionary from used entries in `base`
    let mut inline16: Vec<[u8; INLINE]> = Vec::new();
    let mut lens: Vec<u8> = Vec::new();
    let mut spelling: Vec<u8> = Vec::new();
    let mut word_to_id: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut base_remap = vec![u32::MAX; base.lens.len()];
    let mut codes = vec![0u32; m];

    for u in 0..m {
        let oc = orig_codes[u] as usize;
        if base_remap[oc] == u32::MAX {
            let wb = base.word_bytes(oc);
            let id = if let Some(&ex) = word_to_id.get(wb) {
                ex
            } else {
                let nid = lens.len() as u32;
                if wb.len() <= INLINE {
                    let mut slot = [0u8; INLINE];
                    slot[..wb.len()].copy_from_slice(wb);
                    inline16.push(slot);
                    lens.push(wb.len() as u8);
                } else {
                    let start = spelling.len() as u32;
                    spelling.extend_from_slice(wb);
                    inline16.push(long_slot(start, spelling.len() as u32));
                    lens.push(LONG);
                }
                word_to_id.insert(wb.to_vec(), nid);
                nid
            };
            base_remap[oc] = id;
        }
        codes[u] = base_remap[oc];
    }

    // Greedily contract consecutive inline edges along every cycle subject to `lens.len() < d_max`
    let mut visited = vec![false; m];
    let mut buf = Vec::with_capacity(INLINE);
    for start_idx in 0..=m {
        let start = if start_idx == 0 {
            mark as usize
        } else {
            start_idx - 1
        };
        if !alive[start] || visited[start] {
            continue;
        }
        let mut u = start;
        while alive[u] && !visited[u] {
            visited[u] = true;
            loop {
                let v = next[u] as usize;
                if v == mark as usize || v == u || !alive[v] || visited[v] {
                    break;
                }
                let lu = lens[codes[u] as usize];
                let lv = lens[codes[v] as usize];
                if lu == LONG || lv == LONG {
                    break;
                }
                let comb = usize::from(lu) + usize::from(lv);
                if comb > INLINE {
                    break;
                }
                buf.clear();
                buf.extend_from_slice(&inline16[codes[u] as usize][..usize::from(lu)]);
                buf.extend_from_slice(&inline16[codes[v] as usize][..usize::from(lv)]);
                let cid = if let Some(&ex) = word_to_id.get(&buf) {
                    ex
                } else if lens.len() < d_max {
                    let nid = lens.len() as u32;
                    let mut slot = [0u8; INLINE];
                    slot[..comb].copy_from_slice(&buf);
                    inline16.push(slot);
                    lens.push(comb as u8);
                    word_to_id.insert(buf.clone(), nid);
                    nid
                } else {
                    break;
                };
                codes[u] = cid;
                next[u] = next[v];
                alive[v] = false;
            }
            u = next[u] as usize;
        }
    }

    // Compact surviving vertices in ascending original BWT row-rank order and drop unused dict entries
    let mut old_to_new = vec![u32::MAX; m];
    let mut surv_old = Vec::with_capacity(m);
    for (u, &is_alive) in alive.iter().enumerate() {
        if is_alive {
            old_to_new[u] = surv_old.len() as u32;
            surv_old.push(u as u32);
        }
    }
    let new_m = surv_old.len();
    let new_mark = old_to_new[mark as usize];

    let mut used_remap = vec![u32::MAX; lens.len()];
    let mut c_inline16 = Vec::new();
    let mut c_lens = Vec::new();
    let mut c_spelling = Vec::new();
    let mut new_codes = Vec::with_capacity(new_m);

    for &ou in &surv_old {
        let old_c = codes[ou as usize] as usize;
        let mapped = if used_remap[old_c] != u32::MAX {
            used_remap[old_c]
        } else {
            let nid = c_lens.len() as u32;
            let len = lens[old_c];
            if len != LONG {
                c_inline16.push(inline16[old_c]);
                c_lens.push(len);
            } else {
                let w = long_word(&inline16[old_c], &spelling);
                let s = c_spelling.len() as u32;
                c_spelling.extend_from_slice(w);
                c_inline16.push(long_slot(s, c_spelling.len() as u32));
                c_lens.push(LONG);
            }
            used_remap[old_c] = nid;
            nid
        };
        new_codes.push(mapped);
    }

    pack_dict_return_map(
        base.n,
        new_m,
        new_mark,
        &surv_old,
        &next,
        &old_to_new,
        &new_codes,
        c_inline16,
        c_lens,
        c_spelling,
    )
}

/// Family 3b/3c: Contract consecutive short words (`<= 12 B`) into co-located 16-byte `Edge16`
/// entries, preserving ascending original BWT row-rank order of surviving vertices.
/// Also used for actual post-Open linear materialisation timing (`mat_f3b_post_open`).
fn build_edge12_from_dict(base: &DictReturnMap, audit_one_use: bool) -> Edge12ReturnMap {
    let m = base.m;
    let mark = base.mark;
    let (next, codes) = unpack_fwd_graph(base);

    // Linear O(m) cycle-respecting contraction: greedily absorb `v = next[u]` into `u`
    // whenever `v != mark && v != u && len(u) + len(v) <= 12`.
    let mut alive = vec![true; m];
    let mut visited = vec![false; m];
    let mut out_next = next.clone();
    let mut out_len = vec![0u8; m];
    let mut out_pay = vec![[0u8; EDGE12_INLINE]; m];

    // Map base long words into deduplicated compact spelling
    let mut spelling: Vec<u8> = Vec::new();
    let mut long_remap: Vec<(u32, u32)> = vec![(u32::MAX, 0); base.lens.len()];
    let mut get_long_span = |c: usize, spelling: &mut Vec<u8>| -> (u32, u32) {
        if long_remap[c].0 != u32::MAX {
            return long_remap[c];
        }
        let w = base.word_bytes(c);
        let s = spelling.len() as u32;
        spelling.extend_from_slice(w);
        let e = spelling.len() as u32;
        long_remap[c] = (s, e);
        (s, e)
    };

    for start_idx in 0..=m {
        let start = if start_idx == 0 {
            mark as usize
        } else {
            start_idx - 1
        };
        if visited[start] || !alive[start] {
            continue;
        }
        let mut u = start;
        while !visited[u] {
            visited[u] = true;
            let cu = codes[u] as usize;
            let lu = base.lens[cu];
            let wlen_u = if lu != LONG {
                usize::from(lu)
            } else {
                base.word_bytes(cu).len()
            };
            if wlen_u <= EDGE12_INLINE {
                let mut cur_len = wlen_u;
                let mut pay = [0u8; EDGE12_INLINE];
                pay[..wlen_u].copy_from_slice(&base.inline16[cu][..wlen_u]);
                let mut v = next[u] as usize;
                while v != mark as usize && v != u && !visited[v] {
                    let cv = codes[v] as usize;
                    let lv = base.lens[cv];
                    if lv == LONG {
                        break;
                    }
                    let wlen_v = usize::from(lv);
                    if cur_len + wlen_v > EDGE12_INLINE {
                        break;
                    }
                    pay[cur_len..cur_len + wlen_v].copy_from_slice(&base.inline16[cv][..wlen_v]);
                    cur_len += wlen_v;
                    alive[v] = false;
                    visited[v] = true;
                    v = next[v] as usize;
                }
                out_next[u] = v as u32;
                out_len[u] = cur_len as u8;
                out_pay[u] = pay;
                u = v;
            } else {
                let (s, e) = get_long_span(cu, &mut spelling);
                out_next[u] = next[u];
                out_len[u] = LONG;
                let mut pay = [0u8; EDGE12_INLINE];
                pay[..4].copy_from_slice(&s.to_le_bytes());
                pay[4..8].copy_from_slice(&e.to_le_bytes());
                out_pay[u] = pay;
                u = next[u] as usize;
            }
        }
    }

    // Compact surviving vertices in ascending original BWT row-rank order
    let mut old_to_new = vec![u32::MAX; m];
    let mut new_m = 0usize;
    for (u, &is_alive) in alive.iter().enumerate() {
        if is_alive {
            old_to_new[u] = new_m as u32;
            new_m += 1;
        }
    }
    let new_mark = old_to_new[mark as usize];
    assert_ne!(new_mark, u32::MAX);

    let mut edges = Vec::with_capacity(new_m);
    for u in 0..m {
        if alive[u] {
            let nv = old_to_new[out_next[u] as usize];
            debug_assert!(nv < (1 << 24));
            edges.push(Edge16 {
                meta: (u32::from(out_len[u]) << 24) | nv,
                payload: out_pay[u],
            });
        }
    }

    let (one_use_inline_edges, total_inline_edges) = if audit_one_use {
        let mut counts: HashMap<(u8, [u8; EDGE12_INLINE]), u32> = HashMap::new();
        let mut total_inl = 0usize;
        for e in &edges {
            let len = (e.meta >> 24) as u8;
            if len != LONG {
                total_inl += 1;
                *counts.entry((len, e.payload)).or_insert(0) += 1;
            }
        }
        let one_use = counts.values().filter(|&&c| c == 1).count();
        (one_use, total_inl)
    } else {
        (0, 0)
    };

    Edge12ReturnMap {
        n: base.n,
        m: new_m,
        mark: new_mark,
        edges,
        spelling,
        lanes: None,
        one_use_inline_edges,
        total_inline_edges,
    }
}

// ---------------------------------------------------------------------------
// Structural metrics & write amplification accounting
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct ArmMetrics {
    blocks: usize,
    total_n: usize,
    total_m: usize,
    total_visits: usize,
    edges_bytes: usize,
    dict_bytes: usize,
    rep_bytes: usize,
    literal_visits: usize,
    inline_visits: usize,
    long_visits: usize,
    max_word_len: usize,
    wide_edges_blocks: usize,
    four_lane_blocks: usize,
    bytes_written_out: usize,
    one_use_inline_edges: usize,
    total_inline_edges: usize,
    word_lens_sample: Vec<usize>,
}

impl ArmMetrics {
    fn add_dict(&mut self, rep: &DictReturnMap, has_lanes: bool) {
        self.blocks += 1;
        self.total_n += rep.n;
        self.total_m += rep.m;
        self.edges_bytes += rep.edges_bytes();
        self.dict_bytes += rep.dict_bytes();
        self.rep_bytes += rep.rep_bytes();
        if !rep.extra.is_empty() {
            self.wide_edges_blocks += 1;
        }
        if has_lanes && rep.lanes.is_some() {
            self.four_lane_blocks += 1;
        }
        let bits = rep.bits;
        let mask = (1u32 << bits) - 1;
        let mut prev = rep.predecessor;
        let mut j = rep.mark;
        let mut at = 0usize;
        for _ in 0..rep.m {
            let edge = rep.edges[j as usize];
            let code = if rep.extra.is_empty() {
                edge >> bits
            } else {
                rep.extra[j as usize]
            } as usize;
            let len = rep.lens[code];
            let wlen = if len != LONG {
                self.inline_visits += 1;
                if len == 1 {
                    self.literal_visits += 1;
                }
                self.bytes_written_out += INLINE;
                usize::from(len)
            } else {
                self.long_visits += 1;
                let wl = long_word(&rep.inline16[code], &rep.spelling).len();
                let take = wl.min(rep.n - at);
                self.bytes_written_out += take;
                wl
            };
            self.total_visits += 1;
            self.max_word_len = self.max_word_len.max(wlen);
            if self.total_visits % 64 == 0 {
                self.word_lens_sample.push(wlen);
            }
            at += wlen;
            j = if let Some(p) = prev {
                let nx = (edge & mask) ^ p;
                prev = Some(j);
                nx
            } else {
                edge & mask
            };
            if j == rep.mark || at >= rep.n {
                break;
            }
        }
    }

    fn add_edge12(&mut self, rep: &Edge12ReturnMap, has_lanes: bool) {
        self.blocks += 1;
        self.total_n += rep.n;
        self.total_m += rep.m;
        self.edges_bytes += rep.edges_bytes();
        self.dict_bytes += rep.dict_bytes();
        self.rep_bytes += rep.rep_bytes();
        self.one_use_inline_edges += rep.one_use_inline_edges;
        self.total_inline_edges += rep.total_inline_edges;
        if has_lanes && rep.lanes.is_some() {
            self.four_lane_blocks += 1;
        }
        let mut j = rep.mark;
        let mut at = 0usize;
        for _ in 0..rep.m {
            let e = rep.edges[j as usize];
            let len = (e.meta >> 24) as u8;
            let wlen = if len != LONG {
                self.inline_visits += 1;
                if len == 1 {
                    self.literal_visits += 1;
                }
                self.bytes_written_out += EDGE12_INLINE;
                usize::from(len)
            } else {
                self.long_visits += 1;
                let s = u32::from_le_bytes([e.payload[0], e.payload[1], e.payload[2], e.payload[3]])
                    as usize;
                let stop =
                    u32::from_le_bytes([e.payload[4], e.payload[5], e.payload[6], e.payload[7]])
                        as usize;
                let wl = stop - s;
                let take = wl.min(rep.n - at);
                self.bytes_written_out += take;
                wl
            };
            self.total_visits += 1;
            self.max_word_len = self.max_word_len.max(wlen);
            if self.total_visits % 64 == 0 {
                self.word_lens_sample.push(wlen);
            }
            at += wlen;
            j = e.meta & 0x00FF_FFFF;
            if j == rep.mark || at >= rep.n {
                break;
            }
        }
    }

    fn p50_word_len(&mut self) -> usize {
        if self.word_lens_sample.is_empty() {
            return 0;
        }
        self.word_lens_sample.sort_unstable();
        self.word_lens_sample[self.word_lens_sample.len() / 2]
    }
}

// ---------------------------------------------------------------------------
// Corpus loading & synthetic BWT generation
// ---------------------------------------------------------------------------

struct BlockInput {
    stream_idx: usize,
    n: usize,
    orig_ptr: u32,
    records: Vec<u16>,
    walked: Vec<u8>,
}

fn load_wikidata(path: &str) -> (Vec<BlockInput>, Vec<BlockInput>) {
    let data = std::fs::read(path).expect("read bz2 file");
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::default();

    let mut admitted_blocks = Vec::new();
    let mut non_admitted_blocks = Vec::new();
    let mut probe_edges = Vec::new();
    let mut probe_spill = Spill::default();
    let mut probe_out = Vec::new();
    let mut idx = 0usize;

    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        let recs = records_of(&ll);
        let admitted = if let Some(cap) = open_cap(recs.len(), n, b.orig_ptr, false) {
            if cap == Cap::NONE {
                true
            } else {
                probe_out.resize(n + INLINE, 0);
                let mut pass = Pass {
                    queues: Queues::new(&mut probe_out, &mut probe_spill, cap.m),
                    edges: &mut probe_edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                maximal(&recs, |c, pos, width| {
                    pass.take::<false>(c, pos, width, b.orig_ptr, PROD_LIMIT);
                });
                !pass.queues.over && pass.f.m <= cap.m && pass.ns <= cap.strips
            }
        } else {
            false
        };
        let mut walked = Vec::new();
        build(&ll, &counts, &mut tt);
        walk(&tt, b.orig_ptr, &mut walked);
        let blk = BlockInput {
            stream_idx: idx,
            n,
            orig_ptr: b.orig_ptr,
            records: recs,
            walked,
        };
        if admitted {
            admitted_blocks.push(blk);
        } else {
            non_admitted_blocks.push(blk);
        }
        idx += 1;
        syms = b.syms;
    }
    (admitted_blocks, non_admitted_blocks)
}

/// Forward BWT of a text `t` -> `(L, orig_ptr)` matching bzip2's exact cyclic rotation sort.
fn forward_bwt(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    assert!(n > 0);
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = t[(a + k) % n];
            let cb = t[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut orig_ptr = 0u32;
    for (i, &r) in rots.iter().enumerate() {
        if r == 0 {
            orig_ptr = i as u32;
        }
        l.push(t[(r + n - 1) % n]);
    }
    (l, orig_ptr)
}

fn block_from_column(l: &[u8], orig_ptr: u32, stream_idx: usize) -> BlockInput {
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut tt = TVector::default();
    let mut walked = Vec::new();
    build(l, &counts, &mut tt);
    walk(&tt, orig_ptr, &mut walked);
    BlockInput {
        stream_idx,
        n: l.len(),
        orig_ptr,
        records: records_of(l),
        walked,
    }
}

// ---------------------------------------------------------------------------
// Part A: Exact finite search over all 2^(n-1) legal first-return subsets
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SubsetPoint {
    m: usize,
    visits: usize,
    distinct_words: usize,
    dict_payload_bytes: usize,
    rep_bytes: usize,
    max_word_len: usize,
}

fn compute_fl(l: &[u8]) -> Vec<usize> {
    let n = l.len();
    let mut counts = [0usize; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0usize; 256];
    let mut acc = 0usize;
    for c in 0..256 {
        starts[c] = acc;
        acc += counts[c];
    }
    let mut lf = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        lf[i] = starts[b as usize];
        starts[b as usize] += 1;
    }
    let mut fl = vec![0usize; n];
    for (i, &to) in lf.iter().enumerate() {
        fl[to] = i;
    }
    fl
}

fn eval_subset(l: &[u8], fl: &[usize], p: usize, mask: u32, expected: &[u8]) -> SubsetPoint {
    let n = l.len();
    let mut x = p;
    let mut visits = 0usize;
    let mut distinct: HashSet<Vec<u8>> = HashSet::new();
    let mut max_word_len = 0usize;
    let mut out = Vec::with_capacity(n);
    let mut cur_word = Vec::new();

    loop {
        cur_word.clear();
        loop {
            x = fl[x];
            cur_word.push(l[x]);
            if ((mask >> x) & 1) != 0 {
                break;
            }
        }
        visits += 1;
        max_word_len = max_word_len.max(cur_word.len());
        out.extend_from_slice(&cur_word);
        distinct.insert(cur_word.clone());
        if x == p || out.len() >= n {
            break;
        }
    }
    out.truncate(n);
    repeat(n, &mut out);
    assert_eq!(out, expected, "Part A exactness mismatch on mask={mask:#x}");

    let dict_payload_bytes: usize = distinct
        .iter()
        .filter(|w| w.len() > 1)
        .map(|w| w.len())
        .sum();
    let m = mask.count_ones() as usize;
    SubsetPoint {
        m,
        visits,
        distinct_words: distinct.len(),
        dict_payload_bytes,
        rep_bytes: 4 * m + dict_payload_bytes,
        max_word_len,
    }
}

fn width_limit_mask(l: &[u8], p: usize, limit: usize) -> u32 {
    let mut mask = 1u32 << p;
    let mut i = 0usize;
    while i < l.len() {
        let b = l[i];
        let mut j = i + 1;
        while j < l.len() && l[j] == b {
            j += 1;
        }
        if j - i <= limit {
            for k in i..j {
                mask |= 1u32 << k;
            }
        }
        i = j;
    }
    mask
}

fn cmd_part_a() {
    println!("=== PART A: EXACT FINITE SEARCH OVER ALL 2^(n-1) LEGAL FIRST-RETURN SUBSETS ===");

    let (l_json, p_json) = forward_bwt(b"{\"a\":1,\"a\":2,\"a\":3}"); // n=19? Let's keep n<=18:
    let _ = (l_json, p_json);
    let instances: Vec<(&str, Vec<u8>, u32)> = vec![
        {
            let (l, p) = forward_bwt(b"{\"a\":1,\"a\":2,\"a\":3"); // n=18, repeated JSON keys
            ("small_json_rep", l, p)
        },
        {
            let (l, p) = forward_bwt(b"\"P31\":\"Q5\",\"P31\":"); // n=17, Wikidata-like
            ("small_wikidata_micro", l, p)
        },
        {
            let (l, p) = forward_bwt(b"abcdefghijk12345"); // n=16, dense unique
            ("small_dense", l, p)
        },
        {
            let (l, p) = forward_bwt(b"abacabacabacabac"); // n=16, 4 periodic cycles of len 4
            ("small_periodic", l, p)
        },
        {
            // Explicit arbitrary non-BWT multicycle column (n=16): check cycle structure
            let l = b"babccabacabccaba".to_vec();
            ("small_multicycle", l, 0)
        },
        {
            let (l, p) = forward_bwt(b"aaaabbbbccccdddde"); // n=17, run-heavy
            ("small_run_heavy", l, p)
        },
    ];

    for (name, l, p_u32) in &instances {
        let n = l.len();
        let p = *p_u32 as usize;
        assert!(n <= 18);
        let fl = compute_fl(l);
        // Compute cycle count of LF
        let mut seen = vec![false; n];
        let mut cycle_lens = Vec::new();
        for start in 0..n {
            if !seen[start] {
                let mut x = start;
                let mut len = 0;
                while !seen[x] {
                    seen[x] = true;
                    x = fl[x];
                    len += 1;
                }
                cycle_lens.push(len);
            }
        }
        let mut counts = [0u32; 256];
        for &b in l {
            counts[b as usize] += 1;
        }
        let mut tt = TVector::default();
        let mut expected = Vec::new();
        build(l, &counts, &mut tt);
        walk(&tt, *p_u32, &mut expected);

        // Exhaustive enumeration of all 2^(n-1) legal subsets containing p
        let total_subsets = 1u32 << (n - 1);
        let mut best_by_visits: Vec<Option<SubsetPoint>> = vec![None; n + 1];
        let mut best_rep_overall: Option<SubsetPoint> = None;
        let mut best_rep_nontranscript: Option<SubsetPoint> = None; // visits >= 2
        let mut best_inl16_rep: Option<SubsetPoint> = None;

        for idx in 0..total_subsets {
            let low = idx & ((1u32 << p) - 1);
            let high = idx >> p;
            let mask = low | (1u32 << p) | (high << (p + 1));
            let pt = eval_subset(l, &fl, p, mask, &expected);

            let slot = &mut best_by_visits[pt.visits];
            if slot.as_ref().map_or(true, |cur| {
                (pt.dict_payload_bytes, pt.rep_bytes, pt.max_word_len)
                    < (cur.dict_payload_bytes, cur.rep_bytes, cur.max_word_len)
            }) {
                *slot = Some(pt.clone());
            }
            if best_rep_overall.as_ref().map_or(true, |cur| {
                (pt.rep_bytes, pt.visits) < (cur.rep_bytes, cur.visits)
            }) {
                best_rep_overall = Some(pt.clone());
            }
            if pt.visits >= 2
                && best_rep_nontranscript.as_ref().map_or(true, |cur| {
                    (pt.rep_bytes, pt.visits) < (cur.rep_bytes, cur.visits)
                })
            {
                best_rep_nontranscript = Some(pt.clone());
            }
            if pt.max_word_len <= 16
                && pt.visits >= 2
                && best_inl16_rep.as_ref().map_or(true, |cur| {
                    (pt.rep_bytes, pt.visits) < (cur.rep_bytes, cur.visits)
                })
            {
                best_inl16_rep = Some(pt.clone());
            }
        }

        // Extract exact Pareto frontier of (visits, dict_payload_bytes)
        let mut pareto: Vec<SubsetPoint> = Vec::new();
        let mut min_dict_so_far = usize::MAX;
        for pt in best_by_visits.iter().flatten() {
            if pt.dict_payload_bytes < min_dict_so_far {
                min_dict_so_far = pt.dict_payload_bytes;
                pareto.push(pt.clone());
            }
        }

        let runs = records_of(l).len();
        println!(
            "\n--- Instance: {} (n={}, r={}, p={}, cycles={:?}, subsets_checked={} ALL exact) ---",
            name, n, runs, p, cycle_lens, total_subsets
        );
        println!("  Width-based Open LIMIT choices:");
        for lim in [1usize, 2, 4, 8, 16] {
            let wm = width_limit_mask(l, p, lim);
            let wp = eval_subset(l, &fl, p, wm, &expected);
            let pareto_at_same_dict = pareto
                .iter()
                .find(|q| q.dict_payload_bytes <= wp.dict_payload_bytes)
                .map(|q| q.visits)
                .unwrap_or(wp.visits);
            println!(
                "    LIMIT={:>2}: m={:>2} visits={:>2} distinct={:>2} dict_pay={:>2}B rep={:>3}B max_w={:>2} | exact_min_visits_at_<=dict={:>2}",
                lim,
                wp.m,
                wp.visits,
                wp.distinct_words,
                wp.dict_payload_bytes,
                wp.rep_bytes,
                wp.max_word_len,
                pareto_at_same_dict
            );
        }
        println!("  Exact Pareto frontier (visits, dict_payload_bytes):");
        for pt in &pareto {
            println!(
                "    visits={:>2} m={:>2} dict_pay={:>2}B rep={:>3}B distinct={:>2} max_w={:>2}",
                pt.visits,
                pt.m,
                pt.dict_payload_bytes,
                pt.rep_bytes,
                pt.distinct_words,
                pt.max_word_len
            );
        }
        if let Some(best) = &best_rep_nontranscript {
            println!(
                "  Min rep_bytes (visits>=2): visits={} m={} dict_pay={}B rep={}B max_w={}",
                best.visits, best.m, best.dict_payload_bytes, best.rep_bytes, best.max_word_len
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Synthetic + non-admitted exactness stress suite
// ---------------------------------------------------------------------------

fn build_synthetic_stress_blocks() -> Vec<(&'static str, BlockInput)> {
    let mut out = Vec::new();
    // 1. Dense high-entropy valid BWT (n=4096)
    let mut rng = 0x9E37_79B9_7F4A_7C15u64;
    let mut dense_t = Vec::with_capacity(4096);
    for _ in 0..4096 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        dense_t.push(b'a' + (rng % 26) as u8);
    }
    let (l, p) = forward_bwt(&dense_t);
    out.push(("synth_dense_4k", block_from_column(&l, p, 0)));

    // 2. Periodic valid BWTs
    let mut per1 = Vec::new();
    for _ in 0..256 {
        per1.extend_from_slice(b"abac");
    }
    let (l, p) = forward_bwt(&per1);
    out.push(("synth_periodic_abac256", block_from_column(&l, p, 1)));

    let mut per2 = Vec::new();
    for _ in 0..512 {
        per2.extend_from_slice(b"aaab");
    }
    let (l, p) = forward_bwt(&per2);
    out.push(("synth_periodic_aaab512", block_from_column(&l, p, 2)));

    let mut per3 = Vec::new();
    for _ in 0..500 {
        per3.extend_from_slice(b"ab");
    }
    let (l, p) = forward_bwt(&per3);
    out.push(("synth_periodic_ab500", block_from_column(&l, p, 3)));

    // 3. Multicycle arbitrary non-BWT columns
    let mut mc1 = Vec::with_capacity(64);
    for i in 0..64 {
        mc1.push(b"abcde"[(i * 7 + (i / 3)) % 5]);
    }
    out.push(("synth_multicycle_64", block_from_column(&mc1, 13, 4)));

    let mut mc2 = Vec::with_capacity(4096);
    for i in 0..4096 {
        mc2.push(b"0123456789abcdef"[(i * 31 + (i >> 3)) % 16]);
    }
    out.push(("synth_multicycle_4k", block_from_column(&mc2, 777, 5)));

    // 4. Run-heavy structured BWT (n=4096)
    let mut rh = Vec::with_capacity(4096);
    for k in 0..64 {
        let ch = b'A' + (k % 8) as u8;
        for _ in 0..50 {
            rh.push(ch);
        }
        rh.extend_from_slice(format!(",\"id\":\"Q{:04}\"", k).as_bytes());
    }
    rh.truncate(4096);
    let (l, p) = forward_bwt(&rh);
    out.push(("synth_run_heavy_4k", block_from_column(&l, p, 6)));

    // 5. Edge cases
    out.push(("synth_edge_n1", block_from_column(b"Z", 0, 7)));
    out.push(("synth_edge_n2_p0", block_from_column(b"ba", 0, 8)));
    out.push(("synth_edge_n2_p1", block_from_column(b"ba", 1, 9)));
    out.push(("synth_edge_const64", block_from_column(&[b'x'; 64], 31, 10)));

    out
}

fn verify_all_families_on_block(
    name: &str,
    blk: &BlockInput,
    builder: &mut OpenBuilder,
    scratch: &mut Vec<u8>,
    out: &mut Vec<u8>,
) {
    // 1. Shipped bidi
    let (_, m_b, mark_b, bits_b) = builder.construct_timed(
        &blk.records,
        blk.n as u32,
        blk.orig_ptr,
        PROD_LIMIT,
        true,
        scratch,
    );
    let bidi = builder.snapshot_dict_rep(blk.n, m_b, mark_b, bits_b);
    consume_dict_bidi(&bidi, out);
    assert_eq!(out, &blk.walked, "MISMATCH ctrl_shipped_bidi on {name}");

    // 2. Shipped fwd
    let (_, m_f, mark_f, bits_f) = builder.construct_timed(
        &blk.records,
        blk.n as u32,
        blk.orig_ptr,
        PROD_LIMIT,
        false,
        scratch,
    );
    let fwd = builder.snapshot_dict_rep(blk.n, m_f, mark_f, bits_f);
    consume_dict_fwd(&fwd, out);
    assert_eq!(out, &blk.walked, "MISMATCH ctrl_shipped_fwd on {name}");

    // 3. Shipped fwd 4-lane
    let mut fwd4 = fwd.clone();
    fwd4.lanes = compute_lanes_dict(&fwd4);
    consume_dict_4lane(&fwd4, out);
    assert_eq!(
        out, &blk.walked,
        "MISMATCH ctrl_shipped_fwd_4lane on {name}"
    );

    // 4. F1a & F1b (RePair)
    let f1a = build_f1_repair(&fwd, 4, INLINE);
    consume_dict_fwd(&f1a, out);
    assert_eq!(out, &blk.walked, "MISMATCH F1a_repair_inl16 on {name}");

    let f1b = build_f1_repair(&fwd, 4, 0);
    consume_dict_fwd(&f1b, out);
    assert_eq!(out, &blk.walked, "MISMATCH F1b_repair_unb on {name}");

    // 5. F2a & F2b (Conjugation)
    let f2a = build_f2a_conj_code_order(&fwd);
    consume_dict_fwd(&f2a, out);
    assert_eq!(out, &blk.walked, "MISMATCH F2a_conj_code_order on {name}");

    let f2b = build_f2b_conj_cycle_oracle(&fwd);
    consume_dict_fwd(&f2b, out);
    assert_eq!(out, &blk.walked, "MISMATCH F2b_conj_cycle_oracle on {name}");

    // 6. F3a, F3b, F3c
    let f3a = build_f3a_dict_capped_fuse(&fwd, 4096);
    consume_dict_fwd(&f3a, out);
    assert_eq!(out, &blk.walked, "MISMATCH F3a_dict_capped_fuse on {name}");

    let f3b = build_edge12_from_dict(&fwd, false);
    consume_edge12_fwd(&f3b, out);
    assert_eq!(out, &blk.walked, "MISMATCH F3b_inline_edge12 on {name}");

    let f3c = build_edge12_from_dict(&f1b, false);
    consume_edge12_fwd(&f3c, out);
    assert_eq!(out, &blk.walked, "MISMATCH F3c_repair_edge12 on {name}");

    // 7. F4a, F4b, F4c (4-lane contracted)
    let mut f4a = f1b.clone();
    f4a.lanes = compute_lanes_dict(&f4a);
    consume_dict_4lane(&f4a, out);
    assert_eq!(out, &blk.walked, "MISMATCH F4a_repair_unb_4lane on {name}");

    let mut f4b = f3b.clone();
    f4b.lanes = compute_lanes_edge12(&f4b);
    consume_edge12_4lane(&f4b, out);
    assert_eq!(out, &blk.walked, "MISMATCH F4b_edge12_4lane on {name}");

    let mut f4c = f3c.clone();
    f4c.lanes = compute_lanes_edge12(&f4c);
    consume_edge12_4lane(&f4c, out);
    assert_eq!(
        out, &blk.walked,
        "MISMATCH F4c_repair_edge12_4lane on {name}"
    );
}

fn cmd_exactness(bz2_path: Option<&str>) {
    println!("\n=== EXACTNESS STRESS SUITE (SYNTHETIC + FORCED NON-ADMITTED CONTROLS) ===");
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut out = Vec::new();

    let synth = build_synthetic_stress_blocks();
    for (name, blk) in &synth {
        verify_all_families_on_block(name, blk, &mut builder, &mut scratch, &mut out);
    }
    println!(
        "  Synthetic stress cases verified 100% byte-exact across all 13 arms: {}/{}",
        synth.len(),
        synth.len()
    );

    if let Some(path) = bz2_path {
        let (_, non_admitted) = load_wikidata(path);
        let mut nontrivial = 0usize;
        for blk in &non_admitted {
            let label = format!("forced_non_admitted_blk_{}", blk.stream_idx);
            verify_all_families_on_block(&label, blk, &mut builder, &mut scratch, &mut out);
            if blk.n >= 1024 {
                nontrivial += 1;
            }
        }
        println!(
            "  Forced non-admitted wikidata controls verified 100% byte-exact across all 13 arms: {}/{} (of which {} have n >= 1024)",
            non_admitted.len(),
            non_admitted.len(),
            nontrivial
        );
    }
}

// ---------------------------------------------------------------------------
// Part B: Train selection, held-out warm & cold consumer benchmarks,
//         separate offline search & actual materialisation accounting
// ---------------------------------------------------------------------------

enum PrebuiltArm {
    DictBidi(Vec<DictReturnMap>),
    DictFwd(Vec<DictReturnMap>),
    Dict4Lane(Vec<DictReturnMap>),
    Edge12Fwd(Vec<Edge12ReturnMap>),
    Edge12_4Lane(Vec<Edge12ReturnMap>),
}

struct ArmBundle {
    name: &'static str,
    offline_ms: f64,
    data: PrebuiltArm,
    metrics: ArmMetrics,
}

impl ArmBundle {
    fn len(&self) -> usize {
        match &self.data {
            PrebuiltArm::DictBidi(v) | PrebuiltArm::DictFwd(v) | PrebuiltArm::Dict4Lane(v) => {
                v.len()
            }
            PrebuiltArm::Edge12Fwd(v) | PrebuiltArm::Edge12_4Lane(v) => v.len(),
        }
    }

    #[inline(always)]
    fn consume_one(&self, idx: usize, out: &mut Vec<u8>) {
        match &self.data {
            PrebuiltArm::DictBidi(v) => consume_dict_bidi(&v[idx], out),
            PrebuiltArm::DictFwd(v) => consume_dict_fwd(&v[idx], out),
            PrebuiltArm::Dict4Lane(v) => consume_dict_4lane(&v[idx], out),
            PrebuiltArm::Edge12Fwd(v) => consume_edge12_fwd(&v[idx], out),
            PrebuiltArm::Edge12_4Lane(v) => consume_edge12_4lane(&v[idx], out),
        }
    }
}

fn flush_caches(buf: &mut [u8]) {
    let mut sum = 0u64;
    for chunk in buf.chunks_exact_mut(64) {
        chunk[0] = chunk[0].wrapping_add(1);
        sum = sum.wrapping_add(u64::from(chunk[0]));
    }
    black_box(sum);
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

fn build_open_controls(blocks: &[BlockInput]) -> (Vec<DictReturnMap>, Vec<DictReturnMap>) {
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut bidis = Vec::with_capacity(blocks.len());
    let mut fwds = Vec::with_capacity(blocks.len());
    for b in blocks {
        let (_, mb, markb, bitsb) = builder.construct_timed(
            &b.records,
            b.n as u32,
            b.orig_ptr,
            PROD_LIMIT,
            true,
            &mut scratch,
        );
        bidis.push(builder.snapshot_dict_rep(b.n, mb, markb, bitsb));
        let (_, mf, markf, bitsf) = builder.construct_timed(
            &b.records,
            b.n as u32,
            b.orig_ptr,
            PROD_LIMIT,
            false,
            &mut scratch,
        );
        fwds.push(builder.snapshot_dict_rep(b.n, mf, markf, bitsf));
    }
    (bidis, fwds)
}

fn measure_warm_pass(arm: &ArmBundle, out: &mut Vec<u8>) -> f64 {
    let mut total_ns = 0u64;
    for i in 0..arm.len() {
        // Untimed warm-up of block i's exact working set + `out`
        arm.consume_one(i, out);
        black_box(&*out);
        let t0 = Instant::now();
        arm.consume_one(i, out);
        total_ns += t0.elapsed().as_nanos() as u64;
        black_box(&*out);
    }
    total_ns as f64 / 1e6
}

fn measure_cold_pass(arm: &ArmBundle, flush_buf: &mut [u8], out: &mut Vec<u8>) -> f64 {
    flush_caches(flush_buf);
    let t0 = Instant::now();
    for i in 0..arm.len() {
        arm.consume_one(i, out);
        black_box(&*out);
    }
    t0.elapsed().as_nanos() as f64 / 1e6
}

fn compute_metrics_dict(reps: &[DictReturnMap], has_lanes: bool) -> ArmMetrics {
    let mut m = ArmMetrics::default();
    for r in reps {
        m.add_dict(r, has_lanes);
    }
    m
}

fn compute_metrics_edge12(reps: &[Edge12ReturnMap], has_lanes: bool) -> ArmMetrics {
    let mut m = ArmMetrics::default();
    for r in reps {
        m.add_edge12(r, has_lanes);
    }
    m
}

fn cmd_bench(path: &str, repeats: usize) {
    let (admitted, _) = load_wikidata(path);
    assert_eq!(
        admitted.len(),
        233,
        "expected 233 Open-admitted wikidata blocks"
    );
    let train = &admitted[..TRAIN_SPLIT_BLOCKS];
    let held_out = &admitted[TRAIN_SPLIT_BLOCKS..];
    let train_rows: usize = train.iter().map(|b| b.n).sum();
    let held_rows: usize = held_out.iter().map(|b| b.n).sum();
    println!(
        "\n=== PART B: BOUNDED OFFLINE SEARCH & HELD-OUT BENCHMARK (repeats={}) ===",
        repeats
    );
    println!(
        "Split: train = {} blocks ({} rows), held-out = {} blocks ({} rows)",
        train.len(),
        train_rows,
        held_out.len(),
        held_rows
    );

    // -----------------------------------------------------------------------
    // B1. Train-set parameter sweep (frozen selection rule: fastest warm consumer
    //     subject to rep_bytes <= ctrl_shipped_fwd rep_bytes)
    // -----------------------------------------------------------------------
    println!(
        "\n--- B1. Train-set parameter sweep ({} blocks) ---",
        train.len()
    );
    let (_, train_fwds) = build_open_controls(train);
    let train_fwd_metrics = compute_metrics_dict(&train_fwds, false);
    let train_fwd_rep_bytes = train_fwd_metrics.rep_bytes;
    let mut out = Vec::new();
    let mut flush_buf = vec![0u8; COLD_FLUSH_BYTES];

    let train_fwd_arm = ArmBundle {
        name: "train_ctrl_fwd",
        offline_ms: 0.0,
        data: PrebuiltArm::DictFwd(train_fwds.clone()),
        metrics: train_fwd_metrics,
    };
    let mut train_fwd_ms = Vec::new();
    for _ in 0..3 {
        train_fwd_ms.push(measure_warm_pass(&train_fwd_arm, &mut out));
    }
    let (tfwd_med, _, _) = median_min_max(&train_fwd_ms);
    println!(
        "  train_ctrl_fwd: warm_med={:.2}ms visits={} rep_MiB={:.2} dict_KiB/b={:.1}",
        tfwd_med,
        train_fwd_arm.metrics.total_visits,
        train_fwd_rep_bytes as f64 / (1024.0 * 1024.0),
        train_fwd_arm.metrics.dict_bytes as f64 / (1024.0 * train.len() as f64),
    );

    // Sweep F1a (RePair <= 16 B) across tau in {2, 4, 8, 16}
    let mut best_f1a_tau = 4u32;
    let mut best_f1a_ms = f64::INFINITY;
    for &tau in &[2u32, 4, 8, 16] {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = train_fwds
            .iter()
            .map(|r| build_f1_repair(r, tau, INLINE))
            .collect();
        let off_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        let arm = ArmBundle {
            name: "f1a",
            offline_ms: off_ms,
            data: PrebuiltArm::DictFwd(reps),
            metrics: met,
        };
        // Exact check on train
        for (i, b) in train.iter().enumerate() {
            arm.consume_one(i, &mut out);
            assert_eq!(out, b.walked, "train F1a mismatch tau={tau} blk={i}");
        }
        let mut ms = Vec::new();
        for _ in 0..3 {
            ms.push(measure_warm_pass(&arm, &mut out));
        }
        let (med, _, _) = median_min_max(&ms);
        let compact_ok = arm.metrics.rep_bytes <= train_fwd_rep_bytes;
        println!(
            "  F1a_repair_inl16 tau={:>2}: warm={:>6.2}ms ({:.3}x) visits={:>9} rep_MiB={:>5.2} (compact={}) dict_KiB/b={:>5.1} wideE={} off={:.0}ms",
            tau,
            med,
            med / tfwd_med,
            arm.metrics.total_visits,
            arm.metrics.rep_bytes as f64 / (1024.0 * 1024.0),
            compact_ok,
            arm.metrics.dict_bytes as f64 / (1024.0 * train.len() as f64),
            arm.metrics.wide_edges_blocks,
            off_ms,
        );
        if compact_ok && med < best_f1a_ms {
            best_f1a_ms = med;
            best_f1a_tau = tau;
        }
    }

    // Sweep F1b (RePair unbounded) across tau in {2, 4, 8, 16}
    let mut best_f1b_tau = 4u32;
    let mut best_f1b_ms = f64::INFINITY;
    for &tau in &[2u32, 4, 8, 16] {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = train_fwds
            .iter()
            .map(|r| build_f1_repair(r, tau, 0))
            .collect();
        let off_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        let arm = ArmBundle {
            name: "f1b",
            offline_ms: off_ms,
            data: PrebuiltArm::DictFwd(reps),
            metrics: met,
        };
        for (i, b) in train.iter().enumerate() {
            arm.consume_one(i, &mut out);
            assert_eq!(out, b.walked, "train F1b mismatch tau={tau} blk={i}");
        }
        let mut ms = Vec::new();
        for _ in 0..3 {
            ms.push(measure_warm_pass(&arm, &mut out));
        }
        let (med, _, _) = median_min_max(&ms);
        let compact_ok = arm.metrics.rep_bytes <= train_fwd_rep_bytes;
        println!(
            "  F1b_repair_unb   tau={:>2}: warm={:>6.2}ms ({:.3}x) visits={:>9} rep_MiB={:>5.2} (compact={}) dict_KiB/b={:>5.1} wideE={} off={:.0}ms",
            tau,
            med,
            med / tfwd_med,
            arm.metrics.total_visits,
            arm.metrics.rep_bytes as f64 / (1024.0 * 1024.0),
            compact_ok,
            arm.metrics.dict_bytes as f64 / (1024.0 * train.len() as f64),
            arm.metrics.wide_edges_blocks,
            off_ms,
        );
        if compact_ok && med < best_f1b_ms {
            best_f1b_ms = med;
            best_f1b_tau = tau;
        }
    }

    // Sweep F3a (dict-capped <=16B fuse) across D_max in {4096, 16384}
    let mut best_f3a_dmax = 4096usize;
    let mut best_f3a_ms = f64::INFINITY;
    for &d_max in &[4096usize, 16384] {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = train_fwds
            .iter()
            .map(|r| build_f3a_dict_capped_fuse(r, d_max))
            .collect();
        let off_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        let arm = ArmBundle {
            name: "f3a",
            offline_ms: off_ms,
            data: PrebuiltArm::DictFwd(reps),
            metrics: met,
        };
        for (i, b) in train.iter().enumerate() {
            arm.consume_one(i, &mut out);
            assert_eq!(out, b.walked, "train F3a mismatch d_max={d_max} blk={i}");
        }
        let mut ms = Vec::new();
        for _ in 0..3 {
            ms.push(measure_warm_pass(&arm, &mut out));
        }
        let (med, _, _) = median_min_max(&ms);
        let compact_ok = arm.metrics.rep_bytes <= train_fwd_rep_bytes;
        println!(
            "  F3a_dict_fuse  d_max={:>5}: warm={:>6.2}ms ({:.3}x) visits={:>9} rep_MiB={:>5.2} (compact={}) dict_KiB/b={:>5.1} wideE={} off={:.0}ms",
            d_max,
            med,
            med / tfwd_med,
            arm.metrics.total_visits,
            arm.metrics.rep_bytes as f64 / (1024.0 * 1024.0),
            compact_ok,
            arm.metrics.dict_bytes as f64 / (1024.0 * train.len() as f64),
            arm.metrics.wide_edges_blocks,
            off_ms,
        );
        if med < best_f3a_ms {
            best_f3a_ms = med;
            best_f3a_dmax = d_max;
        }
    }

    println!(
        "  ==> Frozen train winners: F1a tau={}, F1b tau={}, F3a d_max={}",
        best_f1a_tau, best_f1b_tau, best_f3a_dmax
    );
    drop(train_fwd_arm);
    drop(train_fwds);

    // -----------------------------------------------------------------------
    // B2. Actual materialisation timings on held-out blocks (5 repeats)
    // -----------------------------------------------------------------------
    println!(
        "\n--- B2. Actual materialisation cost on held-out ({} blocks, 5 repeats) ---",
        held_out.len()
    );
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut mat_bidi_runs = Vec::new();
    let mut mat_fwd_runs = Vec::new();
    for _ in 0..5 {
        let mut tb = ConstructNs::default();
        for b in held_out {
            let (ns, _, _, _) = builder.construct_timed(
                &b.records,
                b.n as u32,
                b.orig_ptr,
                PROD_LIMIT,
                true,
                &mut scratch,
            );
            tb.add(ns);
        }
        mat_bidi_runs.push(tb);

        let mut tf = ConstructNs::default();
        for b in held_out {
            let (ns, _, _, _) = builder.construct_timed(
                &b.records,
                b.n as u32,
                b.orig_ptr,
                PROD_LIMIT,
                false,
                &mut scratch,
            );
            tf.add(ns);
        }
        mat_fwd_runs.push(tf);
    }
    let bidi_tot_ms: Vec<f64> = mat_bidi_runs
        .iter()
        .map(|c| c.total() as f64 / 1e6)
        .collect();
    let bidi_a6c_ms: Vec<f64> = mat_bidi_runs.iter().map(|c| c.a6c as f64 / 1e6).collect();
    let fwd_tot_ms: Vec<f64> = mat_fwd_runs
        .iter()
        .map(|c| c.total() as f64 / 1e6)
        .collect();
    let fwd_a6c_ms: Vec<f64> = mat_fwd_runs.iter().map(|c| c.a6c as f64 / 1e6).collect();
    let (mb_med, mb_min, mb_max) = median_min_max(&bidi_tot_ms);
    let (mf_med, mf_min, mf_max) = median_min_max(&fwd_tot_ms);
    let (ba6c_med, _, _) = median_min_max(&bidi_a6c_ms);
    let (fa6c_med, _, _) = median_min_max(&fwd_a6c_ms);
    let a6c_fwd_ratio = fa6c_med / ba6c_med;
    let i9_c_fwd_proj = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS * a6c_fwd_ratio;

    println!(
        "  mat_shipped_bidi (A6a+b+c_xor): {:.2}ms ({:.2}..{:.2}), A6c={:.2}ms",
        mb_med, mb_min, mb_max, ba6c_med
    );
    println!(
        "  mat_shipped_fwd  (A6a+b+c_fwd): {:.2}ms ({:.2}..{:.2}), A6c={:.2}ms (A6c ratio={:.3}x -> i9 C_fwd proj={:.1}ms)",
        mf_med, mf_min, mf_max, fa6c_med, a6c_fwd_ratio, i9_c_fwd_proj
    );

    // -----------------------------------------------------------------------
    // B3. Prebuild all 13 arms on held-out blocks & verify 100% byte-exactness
    // -----------------------------------------------------------------------
    let t_open = Instant::now();
    let (ho_bidis, ho_fwds) = build_open_controls(held_out);
    let open_build_ms = t_open.elapsed().as_secs_f64() * 1000.0;

    // Also measure actual post-Open linear materialisation of F3b (5 repeats)
    let mut mat_f3b_ms = Vec::new();
    for _ in 0..5 {
        let t0 = Instant::now();
        for r in &ho_fwds {
            let e12 = build_edge12_from_dict(r, false);
            black_box(e12.m);
        }
        mat_f3b_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    let (mf3b_med, mf3b_min, mf3b_max) = median_min_max(&mat_f3b_ms);
    println!(
        "  mat_f3b_post_open (linear O(m) Edge16 pass on top of fwd): {:.2}ms ({:.2}..{:.2}) -> total C_fwd+pass={:.2}ms ({:.3}x bidi C)",
        mf3b_med,
        mf3b_min,
        mf3b_max,
        mf_med + mf3b_med,
        (mf_med + mf3b_med) / mb_med
    );

    // Build all held-out arms
    let mut arms: Vec<ArmBundle> = Vec::new();

    // Controls
    arms.push(ArmBundle {
        name: "ctrl_shipped_bidi",
        offline_ms: open_build_ms / 2.0,
        metrics: compute_metrics_dict(&ho_bidis, false),
        data: PrebuiltArm::DictBidi(ho_bidis),
    });
    arms.push(ArmBundle {
        name: "ctrl_shipped_fwd",
        offline_ms: open_build_ms / 2.0,
        metrics: compute_metrics_dict(&ho_fwds, false),
        data: PrebuiltArm::DictFwd(ho_fwds.clone()),
    });
    {
        let t0 = Instant::now();
        let mut reps = ho_fwds.clone();
        for r in &mut reps {
            r.lanes = compute_lanes_dict(r);
        }
        let off = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, true);
        arms.push(ArmBundle {
            name: "ctrl_shipped_fwd_4lane",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::Dict4Lane(reps),
        });
    }

    // Family 1
    {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = ho_fwds
            .iter()
            .map(|r| build_f1_repair(r, best_f1a_tau, INLINE))
            .collect();
        let off = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        arms.push(ArmBundle {
            name: "F1a_repair_inl16",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::DictFwd(reps),
        });
    }
    let f1b_reps: Vec<DictReturnMap>;
    let f1b_off_ms: f64;
    {
        let t0 = Instant::now();
        f1b_reps = ho_fwds
            .iter()
            .map(|r| build_f1_repair(r, best_f1b_tau, 0))
            .collect();
        f1b_off_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&f1b_reps, false);
        arms.push(ArmBundle {
            name: "F1b_repair_unb",
            offline_ms: f1b_off_ms,
            metrics: met,
            data: PrebuiltArm::DictFwd(f1b_reps.clone()),
        });
    }

    // Family 2
    {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = ho_fwds.iter().map(build_f2a_conj_code_order).collect();
        let off = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        arms.push(ArmBundle {
            name: "F2a_conj_code_order",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::DictFwd(reps),
        });
    }
    {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = ho_fwds.iter().map(build_f2b_conj_cycle_oracle).collect();
        let off = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        arms.push(ArmBundle {
            name: "F2b_conj_cycle_oracle",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::DictFwd(reps),
        });
    }

    // Family 3
    {
        let t0 = Instant::now();
        let reps: Vec<DictReturnMap> = ho_fwds
            .iter()
            .map(|r| build_f3a_dict_capped_fuse(r, best_f3a_dmax))
            .collect();
        let off = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, false);
        arms.push(ArmBundle {
            name: "F3a_dict_capped_fuse",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::DictFwd(reps),
        });
    }
    let f3b_reps: Vec<Edge12ReturnMap>;
    let f3b_off_ms: f64;
    {
        let t0 = Instant::now();
        f3b_reps = ho_fwds
            .iter()
            .map(|r| build_edge12_from_dict(r, true))
            .collect();
        f3b_off_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_edge12(&f3b_reps, false);
        arms.push(ArmBundle {
            name: "F3b_inline_edge12",
            offline_ms: f3b_off_ms,
            metrics: met,
            data: PrebuiltArm::Edge12Fwd(f3b_reps.clone()),
        });
    }
    let f3c_reps: Vec<Edge12ReturnMap>;
    let f3c_off_ms: f64;
    {
        let t0 = Instant::now();
        f3c_reps = f1b_reps
            .iter()
            .map(|r| build_edge12_from_dict(r, true))
            .collect();
        f3c_off_ms = f1b_off_ms + t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_edge12(&f3c_reps, false);
        arms.push(ArmBundle {
            name: "F3c_repair_edge12",
            offline_ms: f3c_off_ms,
            metrics: met,
            data: PrebuiltArm::Edge12Fwd(f3c_reps.clone()),
        });
    }

    // Family 4 (4-lane + contracted)
    {
        let t0 = Instant::now();
        let mut reps = f1b_reps;
        for r in &mut reps {
            r.lanes = compute_lanes_dict(r);
        }
        let off = f1b_off_ms + t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_dict(&reps, true);
        arms.push(ArmBundle {
            name: "F4a_repair_unb_4lane",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::Dict4Lane(reps),
        });
    }
    {
        let t0 = Instant::now();
        let mut reps = f3b_reps;
        for r in &mut reps {
            r.lanes = compute_lanes_edge12(r);
        }
        let off = f3b_off_ms + t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_edge12(&reps, true);
        arms.push(ArmBundle {
            name: "F4b_edge12_4lane",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::Edge12_4Lane(reps),
        });
    }
    {
        let t0 = Instant::now();
        let mut reps = f3c_reps;
        for r in &mut reps {
            r.lanes = compute_lanes_edge12(r);
        }
        let off = f3c_off_ms + t0.elapsed().as_secs_f64() * 1000.0;
        let met = compute_metrics_edge12(&reps, true);
        arms.push(ArmBundle {
            name: "F4c_repair_edge12_4l",
            offline_ms: off,
            metrics: met,
            data: PrebuiltArm::Edge12_4Lane(reps),
        });
    }
    drop(ho_fwds);

    // Byte-exactness check on EVERY held-out block x EVERY arm before timing
    for arm in &arms {
        for (i, b) in held_out.iter().enumerate() {
            arm.consume_one(i, &mut out);
            assert_eq!(
                out, b.walked,
                "HELD-OUT BYTE MISMATCH arm={} block={}",
                arm.name, b.stream_idx
            );
        }
    }
    println!(
        "\n# Held-out exactness verified: 100% byte-identical to bwt::walk on all {} blocks x {} arms",
        held_out.len(),
        arms.len()
    );

    // -----------------------------------------------------------------------
    // B4. Structural metrics table on held-out blocks
    // -----------------------------------------------------------------------
    println!(
        "\n--- B4. Structural & representation table (held-out, {} blocks, {} bytes) ---",
        held_out.len(),
        held_rows
    );
    println!(
        "{:<24} {:>10} {:>6} {:>7} {:>7} {:>7} {:>6} {:>4} {:>5} {:>6} {:>6} {:>5} {:>5} {:>7} {:>8}",
        "arm",
        "visits",
        "vs_fwd",
        "edg_K/b",
        "dct_K/b",
        "rep_K/b",
        "mean_w",
        "p50",
        "max_w",
        "lit%",
        "long%",
        "wAmp",
        "wideE",
        "1use%",
        "off_ms"
    );
    let base_visits = arms[1].metrics.total_visits as f64;
    let nb = held_out.len() as f64;
    for arm in &mut arms {
        let p50 = arm.metrics.p50_word_len();
        let m = &arm.metrics;
        let mean_w = m.total_n as f64 / m.total_visits as f64;
        let lit_pct = 100.0 * m.literal_visits as f64 / m.total_visits as f64;
        let long_pct = 100.0 * m.long_visits as f64 / m.total_visits as f64;
        let w_amp = m.bytes_written_out as f64 / m.total_n as f64;
        let one_use_str = if m.total_inline_edges > 0 {
            format!(
                "{:>6.1}%",
                100.0 * m.one_use_inline_edges as f64 / m.total_inline_edges as f64
            )
        } else {
            "    n/a".to_string()
        };
        println!(
            "{:<24} {:>10} {:>5.2}x {:>7.1} {:>7.1} {:>7.1} {:>6.2} {:>4} {:>5} {:>5.1}% {:>5.1}% {:>5.2} {:>5} {:>7} {:>8.0}",
            arm.name,
            m.total_visits,
            m.total_visits as f64 / base_visits,
            m.edges_bytes as f64 / (1024.0 * nb),
            m.dict_bytes as f64 / (1024.0 * nb),
            m.rep_bytes as f64 / (1024.0 * nb),
            mean_w,
            p50,
            m.max_word_len,
            lit_pct,
            long_pct,
            w_amp,
            m.wide_edges_blocks,
            one_use_str,
            arm.offline_ms,
        );
    }

    // -----------------------------------------------------------------------
    // B5. Interleaved consumer timings: WARM (in-situ) & COLD (streaming)
    // -----------------------------------------------------------------------
    let mut warm_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut cold_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];

    for r in 0..repeats {
        for (ai, arm) in arms.iter().enumerate() {
            let wm = measure_warm_pass(arm, &mut out);
            warm_ms[ai].push(wm);
        }
        for (ai, arm) in arms.iter().enumerate() {
            let cm = measure_cold_pass(arm, &mut flush_buf, &mut out);
            cold_ms[ai].push(cm);
        }
        eprintln!("  [held-out bench] repeat {}/{} done", r + 1, repeats);
    }

    let print_timing_table = |label: &str, data: &[Vec<f64>]| {
        println!(
            "\n--- B5. Consumer timings: {} (held-out {} blocks, {} bytes, {} interleaved repeats) ---",
            label,
            held_out.len(),
            held_rows,
            repeats
        );
        println!(
            "{:<24} {:>19} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>6}",
            "arm",
            "ms med (min..max)",
            "ns/row",
            "ns/vis",
            "vs_bidi",
            "vs_fwd",
            "vs_4ln",
            "E_i9",
            "dE_bidi",
            "dE_fwd",
            "Cmax_b",
            "Cmax_f",
            "dC_ext",
            "8w_dE"
        );
        let (bidi_med, _, _) = median_min_max(&data[0]);
        let (fwd_med, _, _) = median_min_max(&data[1]);
        let (l4_med, _, _) = median_min_max(&data[2]);
        let e_fwd_i9 = I9_E_BASE_MS * (fwd_med / bidi_med);

        for (ai, arm) in arms.iter().enumerate() {
            let (med, mn, mx) = median_min_max(&data[ai]);
            let ns_row = med * 1e6 / held_rows as f64;
            let ns_vis = med * 1e6 / arm.metrics.total_visits as f64;
            let r_bidi: Vec<f64> = (0..repeats).map(|k| data[ai][k] / data[0][k]).collect();
            let r_fwd: Vec<f64> = (0..repeats).map(|k| data[ai][k] / data[1][k]).collect();
            let r_4ln: Vec<f64> = (0..repeats).map(|k| data[ai][k] / data[2][k]).collect();
            let (rb, _, _) = median_min_max(&r_bidi);
            let (rf, _, _) = median_min_max(&r_fwd);
            let (r4, _, _) = median_min_max(&r_4ln);

            let e_i9 = I9_E_BASE_MS * (med / bidi_med);
            let de_bidi = I9_E_BASE_MS - e_i9;
            let de_fwd = e_fwd_i9 - e_i9;
            let cmax_base = I9_CE_BASE_MS - e_i9 - I9_GATE_MS;
            let cmax_fwd = (i9_c_fwd_proj + e_fwd_i9) - e_i9 - I9_GATE_MS;
            let dc_ext = de_fwd - I9_GATE_MS;
            let w8_de = de_fwd * I9_8W_E_SCALE;

            println!(
                "{:<24} {:>6.2} ({:>5.2}..{:>5.2}) {:>7.3} {:>7.2} {:>6.3}x {:>6.3}x {:>6.3}x | {:>7.1} {:>+7.1} {:>+7.1} {:>7.1} {:>7.1} {:>+7.1} {:>+6.1}",
                arm.name,
                med,
                mn,
                mx,
                ns_row,
                ns_vis,
                rb,
                rf,
                r4,
                e_i9,
                de_bidi,
                de_fwd,
                cmax_base,
                cmax_fwd,
                dc_ext,
                w8_de
            );
        }
        let _ = l4_med;
    };

    print_timing_table("WARM (in-situ per-block L1/L2/L3 warm)", &warm_ms);
    print_timing_table("COLD (64 MiB flush before each arm pass)", &cold_ms);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage:\n  {} part_a\n  {} exact [FILE.bz2]\n  {} bench FILE.bz2 [--repeats N]\n  {} all FILE.bz2 [--repeats N]",
            args[0], args[0], args[0], args[0]
        );
        std::process::exit(1);
    }
    let cmd = args[1].as_str();
    let mut repeats = 7usize;
    let mut file: Option<&str> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("invalid repeats");
            i += 2;
        } else {
            file = Some(args[i].as_str());
            i += 1;
        }
    }

    match cmd {
        "part_a" => cmd_part_a(),
        "exact" => cmd_exactness(file),
        "bench" => cmd_bench(file.expect("FILE.bz2 required"), repeats),
        "all" => {
            cmd_part_a();
            let f = file.expect("FILE.bz2 required");
            cmd_exactness(Some(f));
            cmd_bench(f, repeats);
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            std::process::exit(1);
        }
    }
}
