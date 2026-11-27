#![forbid(unsafe_code)]
//! Smooth-zone probe for claim `forward-rle1-crc-fusion`:
//! Upstream RLE1 expansion and tiled PCLMULQDQ CRC32 fusion into forward BWT emission
//! (1-lane inline FSM + pre-flagged dictionary, 1-lane L1d tile scan, 4-lane inline FSM
//! & L1d tile scan with GF(2) crc32_combine vs lane stitching, and interaction with
//! contracted return maps F1b/F4a).
//!
//! Zero `src/` edits, `#![forbid(unsafe_code)]`.
//!
//! Usage:
//!   cargo run --release --example forward_rle1_crc_fusion -- corpora/wikidata.bz2 [--ab-dir DIR] [--repeats N] [--skip-f1b]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, ErrorKind, Kernels};
use std::collections::{BinaryHeap, HashMap};
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open & RLE1 constants
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
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

const SHIPPED_TILE: usize = 64 << 10;
const TRAIN_SPLIT_BLOCKS: usize = 117;
const COLD_FLUSH_BYTES: usize = 64 * 1024 * 1024;

// Stage-1 i9-10900K 1W reference anchors (wikidata x4, 1,724 blocks, 838.80 MB)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_GATE_3PCT_MS: f64 = I9_WALL_1W_MS * 0.03; // 39.18 ms
const I9_E_BIDI_1W_MS: f64 = 490.3;
const I9_A9_1W_MS: f64 = 70.5; // midpoint of 66.6-74.4 ms (cc8ba137 / c71c4a56)
const I9_EA9_BIDI_1W_MS: f64 = I9_E_BIDI_1W_MS + I9_A9_1W_MS; // 560.8 ms

type Run = (u32, u8, u8);

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

// ---------------------------------------------------------------------------
// GF(2) CRC-32/BZIP2 combination (for backward in-place expansion with tiled CRC)
// ---------------------------------------------------------------------------

struct CrcCombine {
    /// `pow_byte[k]` is the 32x32 GF(2) matrix multiplying a CRC register by
    /// `x^(8 * 2^k) mod P(x)` in CRC-32/BZIP2's non-reflected MSB-first basis.
    pow_byte: [[u32; 32]; 32],
}

impl CrcCombine {
    fn new() -> Self {
        let mut step1 = [0u32; 32];
        for (j, col) in step1.iter_mut().enumerate() {
            let v = 1u32 << j;
            *col = if v & 0x8000_0000 != 0 {
                (v << 1) ^ crc::POLY
            } else {
                v << 1
            };
        }
        // Square 3 times: 1 bit -> 2 -> 4 -> 8 bits (1 byte)
        let mut byte1 = Self::mat_sq(&step1);
        byte1 = Self::mat_sq(&byte1);
        byte1 = Self::mat_sq(&byte1);

        let mut pow_byte = [[0u32; 32]; 32];
        pow_byte[0] = byte1;
        for k in 1..32 {
            pow_byte[k] = Self::mat_sq(&pow_byte[k - 1]);
        }
        Self { pow_byte }
    }

    #[inline]
    fn mat_vec(mat: &[u32; 32], mut v: u32) -> u32 {
        let mut sum = 0u32;
        let mut i = 0;
        while v != 0 {
            if v & 1 != 0 {
                sum ^= mat[i];
            }
            v >>= 1;
            i += 1;
        }
        sum
    }

    fn mat_sq(a: &[u32; 32]) -> [u32; 32] {
        let mut out = [0u32; 32];
        for (j, col) in out.iter_mut().enumerate() {
            *col = Self::mat_vec(a, a[j]);
        }
        out
    }

    /// Combine `c0 = update_with(init, a)` and `c1 = update_with(0, b)` where `len1 = b.len()`.
    #[inline]
    fn combine(&self, mut c0: u32, c1: u32, mut len1: usize) -> u32 {
        let mut k = 0;
        while len1 != 0 {
            if len1 & 1 != 0 {
                c0 = Self::mat_vec(&self.pow_byte[k], c0);
            }
            len1 >>= 1;
            k += 1;
        }
        c0 ^ c1
    }
}

// ---------------------------------------------------------------------------
// Safe 4-equal-byte scanners (bounds-check-free auto-vec 32B, SWAR u64, scalar)
// ---------------------------------------------------------------------------

/// Bounds-check-free safe 32-byte chunked scanner for 4 consecutive equal bytes.
/// Under `-C target-cpu=native` LLVM compiles the inner loop to the exact AVX2 sequence
/// (`vmovdqu` + 3x `vpcmpeqb` + 2x `vpand` + `vpmovmskb`) of `src/rle.rs::find_runs_avx2`;
/// under baseline x86-64 it compiles to 16-byte SSE2 (`pcmpeqb` + `pand` + `pmovmskb`).
#[inline]
fn find_next_run_autovec32(s: &[u8]) -> Option<usize> {
    let n = s.len();
    if n >= 35 {
        let m = (n - 3) & !31;
        let c0 = s[..m].chunks_exact(32);
        let c1 = s[1..m + 1].chunks_exact(32);
        let c2 = s[2..m + 2].chunks_exact(32);
        let c3 = s[3..m + 3].chunks_exact(32);
        for (chunk_idx, (((a, b), c), d)) in c0.zip(c1).zip(c2).zip(c3).enumerate() {
            let mut any = 0u8;
            for idx in 0..32 {
                any |= ((a[idx] == b[idx]) & (a[idx] == c[idx]) & (a[idx] == d[idx])) as u8;
            }
            if any != 0 {
                for idx in 0..32 {
                    if a[idx] == b[idx] && a[idx] == c[idx] && a[idx] == d[idx] {
                        return Some(chunk_idx * 32 + idx);
                    }
                }
            }
        }
        return find_next_run_scalar(s, m);
    }
    find_next_run_scalar(s, 0)
}

#[inline]
fn find_next_run_scalar(s: &[u8], mut i: usize) -> Option<usize> {
    let n = s.len();
    while i + 3 < n {
        let c = s[i];
        if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_runs_with(
    rle: &[u8],
    runs: &mut Vec<Run>,
    scanner: fn(&[u8]) -> Option<usize>,
) -> Result<(), ErrorKind> {
    runs.clear();
    let n = rle.len();
    let mut i = 0usize;
    while i + 3 < n {
        match scanner(&rle[i..]) {
            None => break,
            Some(rel) => {
                let j = i + rel;
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, rle[j], extra));
                i = j + 5;
            }
        }
    }
    Ok(())
}

#[inline]
fn find_runs_fast(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
    find_runs_with(rle, runs, find_next_run_autovec32)
}

// ---------------------------------------------------------------------------
// Arm 1: Faithful mirror of shipped `src/rle.rs::unrle_in_place`
// ---------------------------------------------------------------------------

fn shipped_unrle_in_place(
    buf: &mut Vec<u8>,
    runs: &mut Vec<Run>,
    stage: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let n = buf.len();
    if n > u32::MAX as usize {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(crc::crc32(buf, k));
    }
    find_runs_fast(buf, runs)?;
    let Some(&(first, _, _)) = runs.first() else {
        return Ok(crc::crc32(buf, k));
    };
    let (mut grown, mut shift) = (0isize, 0isize);
    for &(_, _, extra) in runs.iter() {
        grown += extra as isize - 1;
        shift = shift.max(grown);
    }
    let (shift, len) = (shift as usize, (n as isize + grown) as usize);
    if shift > n {
        std::mem::swap(buf, stage);
        buf.clear();
        rle::unrle(stage, buf, k)?;
        return Ok(crc::crc32(buf, k));
    }
    let first = first as usize;
    if shift > 0 {
        buf.reserve_exact(shift);
        buf.resize(n + shift, 0);
        buf.copy_within(first..n, first + shift);
    }
    let (mut s, mut at) = (!0u32, 0usize);
    let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
        while end - *at >= SHIPPED_TILE {
            *s = crc::update_with(*s, &buf[*at..*at + SHIPPED_TILE], k);
            *at += SHIPPED_TILE;
        }
    };
    tiles(&mut s, &mut at, buf, first);
    let (mut src, mut dst) = (first + shift, first);
    for &(j, c, extra) in runs.iter() {
        let from = j as usize + shift;
        if from > src {
            if dst != src {
                buf.copy_within(src..from, dst);
            }
            dst += from - src;
        }
        let run = 4 + extra as usize;
        buf[dst..dst + run].fill(c);
        dst += run;
        src = from + 5;
        tiles(&mut s, &mut at, buf, dst);
    }
    if n + shift > src && dst != src {
        buf.copy_within(src..n + shift, dst);
    }
    buf.truncate(len);
    s = crc::update_with(s, &buf[at..], k);
    Ok(!s)
}

// ---------------------------------------------------------------------------
// Fused forward BWT emission + RLE1 expansion + tiled PCLMULQDQ CRC32
// ---------------------------------------------------------------------------

const WM_HAS_RUN: u8 = 0x10;
const WM_ALL_SAME: u8 = 0x20;
const WM_TAIL_MASK: u8 = 0x0F;

#[derive(Clone, Copy)]
struct WordMeta {
    head_byte: u8,
    tail_byte: u8,
    /// Leading run length of `head_byte` (1..=3 if `!has_run`, 0 if `has_run`)
    head_run: u8,
    /// bits 0..3: `tail_run` (1..=3); bit 4: `WM_HAS_RUN`; bit 5: `WM_ALL_SAME`
    tail_and_flags: u8,
}

struct FusedDict {
    rep: DictReturnMap,
    word_meta: Vec<WordMeta>,
    open_literals: bool,
    meta_words_with_run: usize,
    meta_multi_words: usize,
}

#[inline]
fn compute_word_meta_one(w: &[u8]) -> WordMeta {
    debug_assert!(!w.is_empty());
    let head_byte = w[0];
    let tail_byte = w[w.len() - 1];
    if w.len() == 1 {
        return WordMeta {
            head_byte,
            tail_byte,
            head_run: 1,
            tail_and_flags: 1 | WM_ALL_SAME,
        };
    }
    let mut prev = head_byte;
    let mut run = 1u8;
    let mut head_run = 1u8;
    let mut in_head = true;
    for &b in &w[1..] {
        if b == prev {
            run += 1;
            if in_head {
                head_run = run;
            }
            if run == 4 {
                return WordMeta {
                    head_byte,
                    tail_byte,
                    head_run: 0,
                    tail_and_flags: WM_HAS_RUN,
                };
            }
        } else {
            in_head = false;
            prev = b;
            run = 1;
        }
    }
    let all_same = usize::from(head_run) == w.len();
    WordMeta {
        head_byte,
        tail_byte,
        head_run,
        tail_and_flags: run | if all_same { WM_ALL_SAME } else { 0 },
    }
}

fn build_word_meta_into(
    rep: &DictReturnMap,
    open_literals: bool,
    out: &mut Vec<WordMeta>,
) -> (usize, usize) {
    let n_codes = rep.lens.len();
    out.clear();
    out.reserve(n_codes);
    let start = if open_literals {
        for c in 0..=255u8 {
            out.push(WordMeta {
                head_byte: c,
                tail_byte: c,
                head_run: 1,
                tail_and_flags: 1 | WM_ALL_SAME,
            });
        }
        256
    } else {
        0
    };
    let mut with_run = 0usize;
    let mut multi = 0usize;
    for code in start..n_codes {
        let w = rep.word_bytes(code);
        let m = compute_word_meta_one(w);
        if w.len() > 1 {
            multi += 1;
            if (m.tail_and_flags & WM_HAS_RUN) != 0 {
                with_run += 1;
            }
        }
        out.push(m);
    }
    (multi, with_run)
}

fn check_open_literals(rep: &DictReturnMap) -> bool {
    if rep.lens.len() < 256 {
        return false;
    }
    for c in 0..256 {
        if rep.lens[c] != 1 || rep.inline16[c][0] != c as u8 {
            return false;
        }
    }
    true
}

fn make_fused_dict(mut rep: DictReturnMap, with_lanes: bool) -> FusedDict {
    if with_lanes {
        rep.lanes = compute_lanes_dict(&rep);
    }
    let open_literals = check_open_literals(&rep);
    let mut word_meta = Vec::new();
    let (meta_multi_words, meta_words_with_run) =
        build_word_meta_into(&rep, open_literals, &mut word_meta);
    FusedDict {
        rep,
        word_meta,
        open_literals,
        meta_words_with_run,
        meta_multi_words,
    }
}

#[derive(Clone, Copy, Default)]
struct FusionStats {
    boundary_straddles: usize,
    overflow_fallbacks: usize,
    no_lanes_fallbacks: usize,
    lanes_grown_zero: usize,
    lanes_total: usize,
    stitch_bytes_moved: u64,
    out_bytes_written: u64,
}

#[derive(Clone, Copy)]
struct LaneRanges {
    ranges: [(usize, usize); 4],
    n_ranges: usize,
    four_vecs: bool,
}

impl LaneRanges {
    fn single(len: usize) -> Self {
        Self {
            ranges: [(0, len), (0, 0), (0, 0), (0, 0)],
            n_ranges: 1,
            four_vecs: false,
        }
    }

    fn from_four_vecs(lens: [usize; 4]) -> Self {
        Self {
            ranges: [(0, lens[0]), (0, lens[1]), (0, lens[2]), (0, lens[3])],
            n_ranges: 4,
            four_vecs: true,
        }
    }

    fn total_len(&self) -> usize {
        self.ranges[..self.n_ranges]
            .iter()
            .map(|&(s, e)| e - s)
            .sum()
    }

    fn materialize(&self, sc: &Scratch) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.total_len());
        if self.four_vecs {
            for (l, &(s, e)) in self.ranges[..4].iter().enumerate() {
                v.extend_from_slice(&sc.lanes[l][s..e]);
            }
        } else {
            for &(s, e) in &self.ranges[..self.n_ranges] {
                v.extend_from_slice(&sc.out[s..e]);
            }
        }
        v
    }
}

// ---------------------------------------------------------------------------
// Exact RLE1 helpers for inline FSM and L1d-tile scan/expand
// ---------------------------------------------------------------------------

#[inline]
fn leading_eq(s: &[u8], byte: u8) -> usize {
    let mut c = 0usize;
    let lim = s.len().min(4);
    while c < lim && s[c] == byte {
        c += 1;
    }
    c
}

#[inline]
fn trailing_run_nonempty(s: &[u8]) -> (u8, u8) {
    let len = s.len();
    let last = s[len - 1];
    let mut r = 1usize;
    let lim = len.min(4);
    while r < lim && s[len - 1 - r] == last {
        r += 1;
    }
    (last, r as u8)
}

#[inline]
fn check_boundary_straddle(exit_byte: u8, exit_len: u8, first4: &[u8], first4_len: usize) -> bool {
    if exit_len == 4 {
        return true;
    }
    if exit_len == 0 || first4_len == 0 {
        return false;
    }
    if first4[0] != exit_byte {
        return false;
    }
    usize::from(exit_len) + leading_eq(&first4[..first4_len], exit_byte) >= 4
}

#[inline(never)]
#[cold]
#[allow(clippy::too_many_arguments)]
fn feed_rle_bytes_vec(
    word: &[u8],
    out: &mut Vec<u8>,
    at: &mut usize,
    run_byte: &mut u8,
    run_len: &mut u8,
    crc_state: &mut u32,
    crc_at: &mut usize,
    tile: usize,
    k: Kernels,
) {
    for &b in word {
        if *run_len == 4 {
            let extra = b as usize;
            if extra > 0 {
                let end = *at + extra;
                if end <= out.len() {
                    out[*at..end].fill(*run_byte);
                } else {
                    let in_bounds = out.len().saturating_sub(*at);
                    if in_bounds > 0 {
                        out[*at..*at + in_bounds].fill(*run_byte);
                    }
                    out.resize(end, *run_byte);
                }
                *at = end;
            }
            *run_len = 0;
        } else {
            if *at < out.len() {
                out[*at] = b;
            } else {
                out.push(b);
            }
            *at += 1;
            *run_len = if b == *run_byte { *run_len + 1 } else { 1 };
            *run_byte = b;
        }
        if *at - *crc_at >= tile {
            let flush = ((*at - *crc_at) / tile) * tile;
            *crc_state = crc::update_with(*crc_state, &out[*crc_at..*crc_at + flush], k);
            *crc_at += flush;
        }
    }
}

#[inline(never)]
#[cold]
#[allow(clippy::too_many_arguments)]
fn feed_rle_bytes_slice(
    word: &[u8],
    out: &mut [u8],
    at: &mut usize,
    run_byte: &mut u8,
    run_len: &mut u8,
    crc_state: &mut u32,
    crc_at: &mut usize,
    tile: usize,
    k: Kernels,
) -> bool {
    for &b in word {
        if *run_len == 4 {
            let extra = b as usize;
            if *at + extra + INLINE > out.len() {
                return false;
            }
            if extra > 0 {
                out[*at..*at + extra].fill(*run_byte);
                *at += extra;
            }
            *run_len = 0;
        } else {
            if *at + INLINE > out.len() {
                return false;
            }
            out[*at] = b;
            *at += 1;
            *run_len = if b == *run_byte { *run_len + 1 } else { 1 };
            *run_byte = b;
        }
        if *at - *crc_at >= tile {
            let flush = ((*at - *crc_at) / tile) * tile;
            *crc_state = crc::update_with(*crc_state, &out[*crc_at..*crc_at + flush], k);
            *crc_at += flush;
        }
    }
    true
}

#[inline(always)]
fn vec_fill_run(out: &mut Vec<u8>, dst: &mut usize, count: usize, byte: u8) {
    let end = *dst + count;
    if end <= out.len() {
        out[*dst..end].fill(byte);
    } else {
        let in_bounds = out.len().saturating_sub(*dst);
        if in_bounds > 0 {
            out[*dst..*dst + in_bounds].fill(byte);
        }
        out.resize(end, byte);
    }
    *dst = end;
}

#[inline(always)]
fn vec_copy_lit(out: &mut Vec<u8>, dst: &mut usize, src: &[u8]) {
    let end = *dst + src.len();
    if end <= out.len() {
        out[*dst..end].copy_from_slice(src);
    } else {
        let in_bounds = out.len().saturating_sub(*dst);
        if in_bounds > 0 {
            out[*dst..*dst + in_bounds].copy_from_slice(&src[..in_bounds]);
        }
        out.extend_from_slice(&src[in_bounds..]);
    }
    *dst = end;
}

/// Process a newly emitted raw pre-RLE1 tile `out[raw_start..*at]` in place (Vec version,
/// growing `out` without zero-fill overhead), updating `at`, `(run_byte, run_len)`, and `crc_state`.
#[inline]
#[allow(clippy::too_many_arguments)]
fn process_l1_tile_vec(
    out: &mut Vec<u8>,
    raw_start: usize,
    at: &mut usize,
    scratch: &mut Vec<u8>,
    run_byte: &mut u8,
    run_len: &mut u8,
    crc_state: &mut u32,
    crc_at: &mut usize,
    crc_tile: usize,
    k: Kernels,
) {
    let raw_end = *at;
    let raw_len = raw_end - raw_start;
    if raw_len == 0 {
        return;
    }
    let raw = &out[raw_start..raw_end];
    let boundary_ok = *run_len != 4
        && (*run_len == 0
            || raw[0] != *run_byte
            || (usize::from(*run_len) + leading_eq(raw, *run_byte)) < 4);
    if boundary_ok {
        match find_next_run_autovec32(raw) {
            None => {
                let (last, tr) = trailing_run_nonempty(raw);
                if usize::from(tr) == raw_len && *run_len > 0 && last == *run_byte {
                    *run_len += tr;
                } else {
                    *run_byte = last;
                    *run_len = tr;
                }
                if raw_end - *crc_at >= crc_tile {
                    *crc_state = crc::update_with(*crc_state, &out[*crc_at..raw_end], k);
                    *crc_at = raw_end;
                }
                return;
            }
            Some(first_rel) => {
                // Fast zero-scratch in-place compaction: bytes `raw_start..raw_start+first_rel`
                // are already in place; compact directly inside `out` while `dst <= src`.
                let mut src = raw_start + first_rel;
                let mut dst = src;
                let mut spilled = false;
                loop {
                    let c = out[src];
                    if src + 4 < raw_end {
                        let extra = out[src + 4] as usize;
                        let out_run = 4 + extra;
                        let next_src = src + 5;
                        if dst + out_run <= next_src {
                            out[dst..dst + out_run].fill(c);
                            dst += out_run;
                            src = next_src;
                        } else {
                            // Growth overtakes unread input: spill ONLY suffix `out[src..raw_end]`.
                            scratch.clear();
                            scratch.extend_from_slice(&out[src..raw_end]);
                            *run_len = 0;
                            spilled = true;
                            break;
                        }
                    } else {
                        // Run of 4 at exact end of tile; count byte arrives in next tile.
                        out[dst..dst + 4].fill(c);
                        dst += 4;
                        *run_byte = c;
                        *run_len = 4;
                        break;
                    }
                    if src >= raw_end {
                        *run_len = 0;
                        break;
                    }
                    match find_next_run_autovec32(&out[src..raw_end]) {
                        None => {
                            let rem = raw_end - src;
                            let (last, tr) = trailing_run_nonempty(&out[src..raw_end]);
                            if dst != src {
                                out.copy_within(src..raw_end, dst);
                            }
                            dst += rem;
                            *run_byte = last;
                            *run_len = tr;
                            break;
                        }
                        Some(r2) => {
                            if r2 > 0 {
                                if dst != src {
                                    out.copy_within(src..src + r2, dst);
                                }
                                dst += r2;
                                src += r2;
                            }
                        }
                    }
                }
                if !spilled {
                    *at = dst;
                    if dst - *crc_at >= crc_tile {
                        *crc_state = crc::update_with(*crc_state, &out[*crc_at..dst], k);
                        *crc_at = dst;
                    }
                    return;
                }
                // Finish the spilled suffix in `scratch` starting at `dst` (`*run_len == 0`)
                let mut pos = 0usize;
                let slen = scratch.len();
                while pos < slen {
                    match find_next_run_autovec32(&scratch[pos..]) {
                        None => {
                            vec_copy_lit(out, &mut dst, &scratch[pos..]);
                            let (last, tr) = trailing_run_nonempty(&scratch[pos..]);
                            *run_byte = last;
                            *run_len = tr;
                            break;
                        }
                        Some(rel) => {
                            if rel > 0 {
                                vec_copy_lit(out, &mut dst, &scratch[pos..pos + rel]);
                            }
                            let j = pos + rel;
                            let c = scratch[j];
                            if j + 4 < slen {
                                let extra = scratch[j + 4] as usize;
                                vec_fill_run(out, &mut dst, 4 + extra, c);
                                pos = j + 5;
                            } else {
                                vec_fill_run(out, &mut dst, 4, c);
                                *run_byte = c;
                                *run_len = 4;
                                break;
                            }
                        }
                    }
                }
                *at = dst;
                if dst - *crc_at >= crc_tile {
                    *crc_state = crc::update_with(*crc_state, &out[*crc_at..dst], k);
                    *crc_at = dst;
                }
                return;
            }
        }
    }

    // Rare boundary-run case (`!boundary_ok`): process via scratch.
    scratch.clear();
    scratch.extend_from_slice(raw);
    let mut dst = raw_start;
    let mut pos = 0usize;
    let slen = scratch.len();

    if *run_len == 4 {
        let extra = scratch[0] as usize;
        if extra > 0 {
            vec_fill_run(out, &mut dst, extra, *run_byte);
        }
        *run_len = 0;
        pos = 1;
    }
    while pos < slen && *run_len > 0 {
        let b = scratch[pos];
        if b == *run_byte {
            if dst < out.len() {
                out[dst] = b;
            } else {
                out.push(b);
            }
            dst += 1;
            *run_len += 1;
            pos += 1;
            if *run_len == 4 {
                if pos < slen {
                    let extra = scratch[pos] as usize;
                    if extra > 0 {
                        vec_fill_run(out, &mut dst, extra, *run_byte);
                    }
                    *run_len = 0;
                    pos += 1;
                }
                break;
            }
        } else {
            *run_len = 0;
            break;
        }
    }

    // Bulk loop: invariant `*run_len == 0` if `pos < slen`
    while pos < slen {
        match find_next_run_autovec32(&scratch[pos..]) {
            None => {
                vec_copy_lit(out, &mut dst, &scratch[pos..]);
                let (last, tr) = trailing_run_nonempty(&scratch[pos..]);
                *run_byte = last;
                *run_len = tr;
                break;
            }
            Some(rel) => {
                if rel > 0 {
                    vec_copy_lit(out, &mut dst, &scratch[pos..pos + rel]);
                }
                let j = pos + rel;
                let c = scratch[j];
                if j + 4 < slen {
                    let extra = scratch[j + 4] as usize;
                    vec_fill_run(out, &mut dst, 4 + extra, c);
                    pos = j + 5;
                } else {
                    vec_fill_run(out, &mut dst, 4, c);
                    *run_byte = c;
                    *run_len = 4;
                    break;
                }
            }
        }
    }

    *at = dst;
    if dst - *crc_at >= crc_tile {
        *crc_state = crc::update_with(*crc_state, &out[*crc_at..dst], k);
        *crc_at = dst;
    }
}

/// Fixed-slice version of `process_l1_tile_vec` for 4-lane disjoint slices.
/// Returns `false` if expansion would exceed `out.len() - INLINE` (overflow fallback trigger).
#[inline]
#[allow(clippy::too_many_arguments)]
fn process_l1_tile_slice(
    out: &mut [u8],
    raw_start: usize,
    at: &mut usize,
    scratch: &mut Vec<u8>,
    run_byte: &mut u8,
    run_len: &mut u8,
    crc_state: &mut u32,
    k: Kernels,
) -> bool {
    let raw_end = *at;
    let raw_len = raw_end - raw_start;
    if raw_len == 0 {
        return true;
    }
    let raw = &out[raw_start..raw_end];
    let boundary_ok = *run_len != 4
        && (*run_len == 0
            || raw[0] != *run_byte
            || (usize::from(*run_len) + leading_eq(raw, *run_byte)) < 4);
    if boundary_ok && find_next_run_autovec32(raw).is_none() {
        let (last, tr) = trailing_run_nonempty(raw);
        if usize::from(tr) == raw_len && *run_len > 0 && last == *run_byte {
            *run_len += tr;
        } else {
            *run_byte = last;
            *run_len = tr;
        }
        *crc_state = crc::update_with(*crc_state, raw, k);
        return true;
    }

    scratch.clear();
    scratch.extend_from_slice(raw);
    let mut dst = raw_start;
    let mut pos = 0usize;
    let slen = scratch.len();
    let max_dst = out.len().saturating_sub(INLINE);

    if *run_len == 4 {
        let extra = scratch[0] as usize;
        if dst + extra > max_dst {
            return false;
        }
        if extra > 0 {
            out[dst..dst + extra].fill(*run_byte);
            dst += extra;
        }
        *run_len = 0;
        pos = 1;
    }
    while pos < slen && *run_len > 0 {
        let b = scratch[pos];
        if b == *run_byte {
            if dst + 1 > max_dst {
                return false;
            }
            out[dst] = b;
            dst += 1;
            *run_len += 1;
            pos += 1;
            if *run_len == 4 {
                if pos < slen {
                    let extra = scratch[pos] as usize;
                    if dst + extra > max_dst {
                        return false;
                    }
                    if extra > 0 {
                        out[dst..dst + extra].fill(*run_byte);
                        dst += extra;
                    }
                    *run_len = 0;
                    pos += 1;
                }
                break;
            }
        } else {
            *run_len = 0;
            break;
        }
    }

    while pos < slen {
        match find_next_run_autovec32(&scratch[pos..]) {
            None => {
                let rem = slen - pos;
                if dst + rem > max_dst {
                    return false;
                }
                out[dst..dst + rem].copy_from_slice(&scratch[pos..]);
                dst += rem;
                let (last, tr) = trailing_run_nonempty(&scratch[pos..]);
                *run_byte = last;
                *run_len = tr;
                break;
            }
            Some(rel) => {
                if rel > 0 {
                    if dst + rel > max_dst {
                        return false;
                    }
                    out[dst..dst + rel].copy_from_slice(&scratch[pos..pos + rel]);
                    dst += rel;
                }
                let j = pos + rel;
                let c = scratch[j];
                if j + 4 < slen {
                    let extra = scratch[j + 4] as usize;
                    let total = 4 + extra;
                    if dst + total > max_dst {
                        return false;
                    }
                    out[dst..dst + total].fill(c);
                    dst += total;
                    pos = j + 5;
                } else {
                    if dst + 4 > max_dst {
                        return false;
                    }
                    out[dst..dst + 4].fill(c);
                    dst += 4;
                    *run_byte = c;
                    *run_len = 4;
                    break;
                }
            }
        }
    }

    *crc_state = crc::update_with(*crc_state, &out[raw_start..dst], k);
    *at = dst;
    true
}

// ---------------------------------------------------------------------------
// Arm 1: Single-lane forward emit + inline RLE1 FSM + pre-flagged WordMeta + tiled CRC32
// ---------------------------------------------------------------------------

#[inline(never)]
fn fused_1lane_fsm<const TILE: usize, const LITFAST: bool>(
    fd: &FusedDict,
    out: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    let rep = &fd.rep;
    let n = rep.n;
    out.resize(n + INLINE, 0);

    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mut j = rep.mark;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let word_meta = &fd.word_meta;
    let use_litfast = LITFAST && fd.open_literals;

    let mut pre_rem = n;
    let mut at = 0usize;
    let mut run_byte = 0u8;
    let mut run_len = 0u8;
    let mut crc_state = !0u32;
    let mut crc_at = 0usize;
    let mut next_crc_at = TILE;

    while pre_rem > 0 {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;

        if use_litfast && code < 256 {
            let b = code as u8;
            pre_rem -= 1;
            if run_len != 4 && at < out.len() {
                out[at] = b;
                at += 1;
                run_len = if b == run_byte { run_len + 1 } else { 1 };
                run_byte = b;
            } else {
                feed_rle_bytes_vec(
                    &[b],
                    out,
                    &mut at,
                    &mut run_byte,
                    &mut run_len,
                    &mut crc_state,
                    &mut crc_at,
                    TILE,
                    k,
                );
                next_crc_at = crc_at + TILE;
                continue;
            }
        } else {
            let len = lens[code];
            if len == 1 {
                let b = inline16[code][0];
                pre_rem -= 1;
                if run_len != 4 && at < out.len() {
                    out[at] = b;
                    at += 1;
                    run_len = if b == run_byte { run_len + 1 } else { 1 };
                    run_byte = b;
                } else {
                    feed_rle_bytes_vec(
                        &[b],
                        out,
                        &mut at,
                        &mut run_byte,
                        &mut run_len,
                        &mut crc_state,
                        &mut crc_at,
                        TILE,
                        k,
                    );
                    next_crc_at = crc_at + TILE;
                    continue;
                }
            } else {
                let meta = word_meta[code];
                let flags = meta.tail_and_flags;
                let fast_ok = (flags & WM_HAS_RUN) == 0
                    && run_len != 4
                    && (meta.head_byte != run_byte || run_len + meta.head_run < 4);
                if len != LONG {
                    let wlen = usize::from(len);
                    if fast_ok && wlen <= pre_rem && at + INLINE <= out.len() {
                        out[at..at + INLINE].copy_from_slice(&inline16[code]);
                        at += wlen;
                        pre_rem -= wlen;
                        if (flags & WM_ALL_SAME) == 0 {
                            run_byte = meta.tail_byte;
                            run_len = flags & WM_TAIL_MASK;
                        } else {
                            run_len = if meta.head_byte == run_byte {
                                run_len + meta.head_run
                            } else {
                                meta.head_run
                            };
                            run_byte = meta.head_byte;
                        }
                    } else {
                        let take = wlen.min(pre_rem);
                        pre_rem -= take;
                        feed_rle_bytes_vec(
                            &inline16[code][..take],
                            out,
                            &mut at,
                            &mut run_byte,
                            &mut run_len,
                            &mut crc_state,
                            &mut crc_at,
                            TILE,
                            k,
                        );
                        next_crc_at = crc_at + TILE;
                        continue;
                    }
                } else {
                    let word = long_word(&inline16[code], spelling);
                    let wlen = word.len();
                    if fast_ok && wlen <= pre_rem && at + wlen + INLINE <= out.len() {
                        out[at..at + wlen].copy_from_slice(word);
                        at += wlen;
                        pre_rem -= wlen;
                        // Long words (> 16 B) without internal runs are never all_same
                        run_byte = meta.tail_byte;
                        run_len = flags & WM_TAIL_MASK;
                    } else {
                        let take = wlen.min(pre_rem);
                        pre_rem -= take;
                        feed_rle_bytes_vec(
                            &word[..take],
                            out,
                            &mut at,
                            &mut run_byte,
                            &mut run_len,
                            &mut crc_state,
                            &mut crc_at,
                            TILE,
                            k,
                        );
                        next_crc_at = crc_at + TILE;
                        continue;
                    }
                }
            }
        }

        if at >= next_crc_at {
            let flush = ((at - crc_at) / TILE) * TILE;
            crc_state = crc::update_with(crc_state, &out[crc_at..crc_at + flush], k);
            crc_at += flush;
            next_crc_at = crc_at + TILE;
        }
    }

    if run_len == 4 {
        return Err(ErrorKind::RunUnfinished);
    }
    out.truncate(at);
    crc_state = crc::update_with(crc_state, &out[crc_at..at], k);
    Ok(!crc_state)
}

// ---------------------------------------------------------------------------
// Arm 2: Single-lane forward emit + L1d-hot in-place tile AVX2 scan/expand + CRC32
// ---------------------------------------------------------------------------

#[inline(never)]
fn fused_1lane_tile_scan<const TILE: usize>(
    rep: &DictReturnMap,
    out: &mut Vec<u8>,
    scratch: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ErrorKind> {
    const CRC_TILE: usize = 65536;
    let n = rep.n;
    out.resize(n + INLINE, 0);

    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mut j = rep.mark;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;

    let mut pre_rem = n;
    let mut at = 0usize;
    let mut run_byte = 0u8;
    let mut run_len = 0u8;
    let mut crc_state = !0u32;
    let mut crc_at = 0usize;

    while pre_rem > 0 {
        let raw_start = at;
        let target_raw = raw_start + TILE;
        if out.len() < target_raw + INLINE {
            out.resize(target_raw + INLINE, 0);
        }
        while pre_rem > 0 && at < target_raw {
            let edge = edges[j as usize];
            let code = if extra.is_empty() {
                edge >> bits
            } else {
                extra[j as usize]
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                let wlen = usize::from(len);
                if at + INLINE > out.len() {
                    out.resize(at + INLINE + TILE, 0);
                }
                if wlen <= pre_rem {
                    out[at..at + INLINE].copy_from_slice(&inline16[code]);
                    at += wlen;
                    pre_rem -= wlen;
                } else {
                    out[at..at + pre_rem].copy_from_slice(&inline16[code][..pre_rem]);
                    at += pre_rem;
                    pre_rem = 0;
                }
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(pre_rem);
                if at + take + INLINE > out.len() {
                    out.resize(at + take + INLINE, 0);
                }
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
                pre_rem -= take;
            }
        }
        process_l1_tile_vec(
            out,
            raw_start,
            &mut at,
            scratch,
            &mut run_byte,
            &mut run_len,
            &mut crc_state,
            &mut crc_at,
            CRC_TILE,
            k,
        );
    }

    if run_len == 4 {
        return Err(ErrorKind::RunUnfinished);
    }
    if at > crc_at {
        crc_state = crc::update_with(crc_state, &out[crc_at..at], k);
    }
    out.truncate(at);
    Ok(!crc_state)
}

// ---------------------------------------------------------------------------
// Arm 3: 4-lane forward emit + L1d-chunked AVX2 tile scan/expand + GF(2) crc32_combine
//        (both contiguous stitched and vectored zero-stitch modes)
// ---------------------------------------------------------------------------

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_raw_step_slice(
    edge: u32,
    idx: u32,
    bits: u32,
    extra: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    at: &mut usize,
    out: &mut [u8],
    overflow: &mut bool,
) {
    let code = if extra.is_empty() {
        edge >> bits
    } else {
        extra[idx as usize]
    } as usize;
    let len = lens[code];
    if len != LONG && *at + INLINE <= out.len() {
        out[*at..*at + INLINE].copy_from_slice(&inline16[code]);
        *at += usize::from(len);
    } else if len != LONG {
        *overflow = true;
    } else {
        let word = long_word(&inline16[code], spelling);
        let wlen = word.len();
        if *at + wlen + INLINE <= out.len() {
            out[*at..*at + wlen].copy_from_slice(word);
            *at += wlen;
        } else {
            *overflow = true;
        }
    }
}

/// Re-run a single lane from `start_j` for `steps` steps with a known incoming RLE1 state
/// `(init_run_byte, init_run_len)` (used ONLY when a 4-byte run straddles a lane boundary).
#[inline(never)]
#[cold]
#[allow(clippy::too_many_arguments)]
fn repair_lane_tile<const CHUNK_STEPS: usize>(
    rep: &DictReturnMap,
    start_j: u32,
    steps: usize,
    init_run_byte: u8,
    init_run_len: u8,
    slice: &mut [u8],
    scratch: &mut Vec<u8>,
    k: Kernels,
) -> Option<(usize, u32, u8, u8)> {
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let spelling = &rep.spelling;

    let mut j = start_j;
    let mut at = 0usize;
    let mut run_byte = init_run_byte;
    let mut run_len = init_run_len;
    let mut crc_state = 0u32;
    let mut overflow = false;

    let mut done = 0usize;
    while done < steps {
        let batch = (steps - done).min(CHUNK_STEPS);
        let raw_start = at;
        for _ in 0..batch {
            let cur = j;
            let ed = edges[cur as usize];
            j = ed & mask;
            emit_raw_step_slice(
                ed,
                cur,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                &mut at,
                slice,
                &mut overflow,
            );
            if overflow {
                return None;
            }
        }
        done += batch;
        if !process_l1_tile_slice(
            slice,
            raw_start,
            &mut at,
            scratch,
            &mut run_byte,
            &mut run_len,
            &mut crc_state,
            k,
        ) {
            return None;
        }
    }
    Some((at, crc_state, run_byte, run_len))
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_raw_step_vec(
    edge: u32,
    idx: u32,
    bits: u32,
    extra: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    at: &mut usize,
    out: &mut Vec<u8>,
) {
    let code = if extra.is_empty() {
        edge >> bits
    } else {
        extra[idx as usize]
    } as usize;
    let len = lens[code];
    if len != LONG {
        if *at + INLINE > out.len() {
            out.resize(*at + INLINE + 4096, 0);
        }
        out[*at..*at + INLINE].copy_from_slice(&inline16[code]);
        *at += usize::from(len);
    } else {
        let word = long_word(&inline16[code], spelling);
        let wlen = word.len();
        if *at + wlen + INLINE > out.len() {
            out.resize(*at + wlen + INLINE, 0);
        }
        out[*at..*at + wlen].copy_from_slice(word);
        *at += wlen;
    }
}

#[inline(never)]
#[cold]
#[allow(clippy::too_many_arguments)]
fn repair_lane_tile_vec<const CHUNK_STEPS: usize>(
    rep: &DictReturnMap,
    start_j: u32,
    steps: usize,
    init_run_byte: u8,
    init_run_len: u8,
    lane_buf: &mut Vec<u8>,
    scratch: &mut Vec<u8>,
    k: Kernels,
) -> (usize, u32, u8, u8) {
    const CRC_TILE: usize = 65536;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let spelling = &rep.spelling;

    let mut j = start_j;
    let mut at = 0usize;
    let mut run_byte = init_run_byte;
    let mut run_len = init_run_len;
    let mut crc_state = 0u32;
    let mut crc_at = 0usize;

    let mut done = 0usize;
    while done < steps {
        let batch = (steps - done).min(CHUNK_STEPS);
        let raw_start = at;
        for _ in 0..batch {
            let cur = j;
            let ed = edges[cur as usize];
            j = ed & mask;
            emit_raw_step_vec(
                ed, cur, bits, extra, inline16, lens, spelling, &mut at, lane_buf,
            );
        }
        done += batch;
        process_l1_tile_vec(
            lane_buf,
            raw_start,
            &mut at,
            scratch,
            &mut run_byte,
            &mut run_len,
            &mut crc_state,
            &mut crc_at,
            CRC_TILE,
            k,
        );
    }
    if at > crc_at {
        crc_state = crc::update_with(crc_state, &lane_buf[crc_at..at], k);
    }
    (at, crc_state, run_byte, run_len)
}

#[inline(never)]
fn fused_4lane_tile<const CHUNK_STEPS: usize, const CRC_TILE: usize, const STITCH: bool>(
    rep: &DictReturnMap,
    out: &mut Vec<u8>,
    lanes: &mut [Vec<u8>; 4],
    scratch: &mut Vec<u8>,
    crc_combine: &CrcCombine,
    k: Kernels,
    stats: Option<&mut FusionStats>,
) -> Result<(u32, LaneRanges), ErrorKind> {
    let Some(l4) = rep.lanes else {
        if let Some(st) = stats {
            st.no_lanes_fallbacks += 1;
        }
        let c = fused_1lane_tile_scan::<16384>(rep, out, scratch, k)?;
        return Ok((c, LaneRanges::single(out.len())));
    };

    let n = rep.n;
    let spans = [
        l4.byte_offs[1] - l4.byte_offs[0],
        l4.byte_offs[2] - l4.byte_offs[1],
        l4.byte_offs[3] - l4.byte_offs[2],
        n - l4.byte_offs[3],
    ];
    for l in 0..4 {
        let need = spans[l] + (spans[l] / 4).max(4096) + INLINE;
        if lanes[l].len() < need {
            lanes[l].resize(need, 0);
        }
    }

    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;

    let (mut j0, mut j1, mut j2, mut j3) = (l4.starts[0], l4.starts[1], l4.starts[2], l4.starts[3]);
    let (mut a0, mut a1, mut a2, mut a3) = (0usize, 0usize, 0usize, 0usize);
    let mut run_byte = [0u8; 4];
    let mut run_len = [0u8; 4];
    let mut crc_state = [!0u32, 0u32, 0u32, 0u32];
    let mut crc_at = [0usize; 4];
    let mut first4 = [[0u8; 4]; 4];
    let mut first4_len = [0usize; 4];

    let min_steps = l4.steps[0]
        .min(l4.steps[1])
        .min(l4.steps[2])
        .min(l4.steps[3]);

    let [s0, s1, s2, s3] = lanes;

    let mut overflow = false;
    let mut step = 0usize;
    while step < min_steps {
        let batch = (min_steps - step).min(CHUNK_STEPS);
        let rs0 = a0;
        let rs1 = a1;
        let rs2 = a2;
        let rs3 = a3;

        {
            let sl0 = s0.as_mut_slice();
            let sl1 = s1.as_mut_slice();
            let sl2 = s2.as_mut_slice();
            let sl3 = s3.as_mut_slice();
            for _ in 0..batch {
                let ed0 = edges[j0 as usize];
                let ed1 = edges[j1 as usize];
                let ed2 = edges[j2 as usize];
                let ed3 = edges[j3 as usize];
                let (oj0, oj1, oj2, oj3) = (j0, j1, j2, j3);
                j0 = ed0 & mask;
                j1 = ed1 & mask;
                j2 = ed2 & mask;
                j3 = ed3 & mask;

                emit_raw_step_slice(
                    ed0,
                    oj0,
                    bits,
                    extra,
                    inline16,
                    lens,
                    spelling,
                    &mut a0,
                    sl0,
                    &mut overflow,
                );
                emit_raw_step_slice(
                    ed1,
                    oj1,
                    bits,
                    extra,
                    inline16,
                    lens,
                    spelling,
                    &mut a1,
                    sl1,
                    &mut overflow,
                );
                emit_raw_step_slice(
                    ed2,
                    oj2,
                    bits,
                    extra,
                    inline16,
                    lens,
                    spelling,
                    &mut a2,
                    sl2,
                    &mut overflow,
                );
                emit_raw_step_slice(
                    ed3,
                    oj3,
                    bits,
                    extra,
                    inline16,
                    lens,
                    spelling,
                    &mut a3,
                    sl3,
                    &mut overflow,
                );
                if overflow {
                    break;
                }
            }
        }
        if overflow {
            if let Some(st) = stats {
                st.overflow_fallbacks += 1;
            }
            let c = fused_1lane_tile_scan::<16384>(rep, out, scratch, k)?;
            return Ok((c, LaneRanges::single(out.len())));
        }
        step += batch;

        if first4_len[1] < 4 {
            let avail = a1 - rs1;
            let need = (4 - first4_len[1]).min(avail);
            first4[1][first4_len[1]..first4_len[1] + need].copy_from_slice(&s1[rs1..rs1 + need]);
            first4_len[1] += need;
        }
        if first4_len[2] < 4 {
            let avail = a2 - rs2;
            let need = (4 - first4_len[2]).min(avail);
            first4[2][first4_len[2]..first4_len[2] + need].copy_from_slice(&s2[rs2..rs2 + need]);
            first4_len[2] += need;
        }
        if first4_len[3] < 4 {
            let avail = a3 - rs3;
            let need = (4 - first4_len[3]).min(avail);
            first4[3][first4_len[3]..first4_len[3] + need].copy_from_slice(&s3[rs3..rs3 + need]);
            first4_len[3] += need;
        }

        process_l1_tile_vec(
            s0,
            rs0,
            &mut a0,
            scratch,
            &mut run_byte[0],
            &mut run_len[0],
            &mut crc_state[0],
            &mut crc_at[0],
            CRC_TILE,
            k,
        );
        process_l1_tile_vec(
            s1,
            rs1,
            &mut a1,
            scratch,
            &mut run_byte[1],
            &mut run_len[1],
            &mut crc_state[1],
            &mut crc_at[1],
            CRC_TILE,
            k,
        );
        process_l1_tile_vec(
            s2,
            rs2,
            &mut a2,
            scratch,
            &mut run_byte[2],
            &mut run_len[2],
            &mut crc_state[2],
            &mut crc_at[2],
            CRC_TILE,
            k,
        );
        process_l1_tile_vec(
            s3,
            rs3,
            &mut a3,
            scratch,
            &mut run_byte[3],
            &mut run_len[3],
            &mut crc_state[3],
            &mut crc_at[3],
            CRC_TILE,
            k,
        );
    }

    // Tail steps (at most 3 steps total across lanes) + final CRC flush per lane
    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        let lb: &mut Vec<u8> = match lane {
            0 => s0,
            1 => s1,
            2 => s2,
            _ => s3,
        };
        if min_steps < l4.steps[lane] {
            let rs = rem_a[lane];
            for _ in min_steps..l4.steps[lane] {
                let cur = rem_j[lane];
                let ed = edges[cur as usize];
                rem_j[lane] = ed & mask;
                emit_raw_step_vec(
                    ed,
                    cur,
                    bits,
                    extra,
                    inline16,
                    lens,
                    spelling,
                    &mut rem_a[lane],
                    lb,
                );
            }
            if lane > 0 && first4_len[lane] < 4 {
                let avail = rem_a[lane] - rs;
                let need = (4 - first4_len[lane]).min(avail);
                first4[lane][first4_len[lane]..first4_len[lane] + need]
                    .copy_from_slice(&lb[rs..rs + need]);
                first4_len[lane] += need;
            }
            process_l1_tile_vec(
                lb,
                rs,
                &mut rem_a[lane],
                scratch,
                &mut run_byte[lane],
                &mut run_len[lane],
                &mut crc_state[lane],
                &mut crc_at[lane],
                CRC_TILE,
                k,
            );
        }
        if rem_a[lane] > crc_at[lane] {
            crc_state[lane] = crc::update_with(crc_state[lane], &lb[crc_at[lane]..rem_a[lane]], k);
        }
    }

    // Exact boundary-straddle check and sequential repair if needed
    let mut straddles = 0usize;
    for lane in 1..4 {
        if check_boundary_straddle(
            run_byte[lane - 1],
            run_len[lane - 1],
            &first4[lane],
            first4_len[lane],
        ) {
            straddles += 1;
            let lb: &mut Vec<u8> = match lane {
                1 => s1,
                2 => s2,
                _ => s3,
            };
            let (new_len, new_crc, new_rb, new_rl) = repair_lane_tile_vec::<CHUNK_STEPS>(
                rep,
                l4.starts[lane],
                l4.steps[lane],
                run_byte[lane - 1],
                run_len[lane - 1],
                lb,
                scratch,
                k,
            );
            rem_a[lane] = new_len;
            crc_state[lane] = new_crc;
            run_byte[lane] = new_rb;
            run_len[lane] = new_rl;
        }
    }

    let [la0, la1, la2, la3] = rem_a;
    if let Some(st) = stats {
        st.boundary_straddles += straddles;
        st.lanes_total += 4;
        for lane in 0..4 {
            if rem_a[lane] == spans[lane] {
                st.lanes_grown_zero += 1;
            }
        }
        st.out_bytes_written += (la0 + la1 + la2 + la3) as u64;
        if STITCH {
            st.stitch_bytes_moved += (la1 + la2 + la3) as u64;
        }
    }

    if run_len[3] == 4 {
        return Err(ErrorKind::RunUnfinished);
    }

    let mut c = crc_combine.combine(crc_state[0], crc_state[1], la1);
    c = crc_combine.combine(c, crc_state[2], la2);
    c = crc_combine.combine(c, crc_state[3], la3);
    let final_crc = !c;

    if STITCH {
        s0.truncate(la0);
        s0.extend_from_slice(&s1[..la1]);
        s0.extend_from_slice(&s2[..la2]);
        s0.extend_from_slice(&s3[..la3]);
        Ok((final_crc, LaneRanges::from_four_vecs([s0.len(), 0, 0, 0])))
    } else {
        Ok((final_crc, LaneRanges::from_four_vecs([la0, la1, la2, la3])))
    }
}

// ---------------------------------------------------------------------------
// Arm 4: 4-lane forward emit + inline RLE1 FSM + pre-flagged WordMeta + GF(2) combine
// ---------------------------------------------------------------------------

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_fsm_step_slice<const TILE: usize>(
    edge: u32,
    idx: u32,
    bits: u32,
    extra: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    word_meta: &[WordMeta],
    open_literals: bool,
    at: &mut usize,
    run_byte: &mut u8,
    run_len: &mut u8,
    crc_state: &mut u32,
    crc_at: &mut usize,
    out: &mut [u8],
    first4: Option<(&mut [u8; 4], &mut usize)>,
    overflow: &mut bool,
    k: Kernels,
) {
    let code = if extra.is_empty() {
        edge >> bits
    } else {
        extra[idx as usize]
    } as usize;

    if let Some((f4, f4_len)) = first4 {
        if *f4_len < 4 {
            let len = lens[code];
            let w: &[u8] = if len != LONG {
                &inline16[code][..usize::from(len)]
            } else {
                long_word(&inline16[code], spelling)
            };
            let take = (4 - *f4_len).min(w.len());
            f4[*f4_len..*f4_len + take].copy_from_slice(&w[..take]);
            *f4_len += take;
        }
    }

    if open_literals && code < 256 {
        let b = code as u8;
        if *run_len != 4 && *at + INLINE <= out.len() {
            out[*at] = b;
            *at += 1;
            *run_len = if b == *run_byte { *run_len + 1 } else { 1 };
            *run_byte = b;
        } else if !feed_rle_bytes_slice(
            &[b],
            out,
            at,
            run_byte,
            run_len,
            crc_state,
            crc_at,
            TILE,
            k,
        ) {
            *overflow = true;
            return;
        }
    } else {
        let len = lens[code];
        if len == 1 {
            let b = inline16[code][0];
            if *run_len != 4 && *at + INLINE <= out.len() {
                out[*at] = b;
                *at += 1;
                *run_len = if b == *run_byte { *run_len + 1 } else { 1 };
                *run_byte = b;
            } else if !feed_rle_bytes_slice(
                &[b],
                out,
                at,
                run_byte,
                run_len,
                crc_state,
                crc_at,
                TILE,
                k,
            ) {
                *overflow = true;
                return;
            }
        } else {
            let meta = word_meta[code];
            let flags = meta.tail_and_flags;
            let fast_ok = (flags & WM_HAS_RUN) == 0
                && *run_len != 4
                && (meta.head_byte != *run_byte || *run_len + meta.head_run < 4);
            if len != LONG {
                let wlen = usize::from(len);
                if fast_ok && *at + INLINE <= out.len() {
                    out[*at..*at + INLINE].copy_from_slice(&inline16[code]);
                    *at += wlen;
                    if (flags & WM_ALL_SAME) == 0 {
                        *run_byte = meta.tail_byte;
                        *run_len = flags & WM_TAIL_MASK;
                    } else {
                        *run_len = if meta.head_byte == *run_byte {
                            *run_len + meta.head_run
                        } else {
                            meta.head_run
                        };
                        *run_byte = meta.head_byte;
                    }
                } else if !feed_rle_bytes_slice(
                    &inline16[code][..wlen],
                    out,
                    at,
                    run_byte,
                    run_len,
                    crc_state,
                    crc_at,
                    TILE,
                    k,
                ) {
                    *overflow = true;
                    return;
                }
            } else {
                let word = long_word(&inline16[code], spelling);
                let wlen = word.len();
                if fast_ok && *at + wlen + INLINE <= out.len() {
                    out[*at..*at + wlen].copy_from_slice(word);
                    *at += wlen;
                    *run_byte = meta.tail_byte;
                    *run_len = flags & WM_TAIL_MASK;
                } else if !feed_rle_bytes_slice(
                    word, out, at, run_byte, run_len, crc_state, crc_at, TILE, k,
                ) {
                    *overflow = true;
                    return;
                }
            }
        }
    }

    if *at - *crc_at >= TILE {
        let flush = ((*at - *crc_at) / TILE) * TILE;
        *crc_state = crc::update_with(*crc_state, &out[*crc_at..*crc_at + flush], k);
        *crc_at += flush;
    }
}

#[inline(never)]
fn fused_4lane_fsm<const TILE: usize, const STITCH: bool>(
    fd: &FusedDict,
    out: &mut Vec<u8>,
    scratch: &mut Vec<u8>,
    crc_combine: &CrcCombine,
    k: Kernels,
) -> Result<(u32, LaneRanges), ErrorKind> {
    let rep = &fd.rep;
    let Some(l4) = rep.lanes else {
        let c = fused_1lane_fsm::<TILE, true>(fd, out, k)?;
        return Ok((c, LaneRanges::single(out.len())));
    };

    let n = rep.n;
    let spans = [
        l4.byte_offs[1] - l4.byte_offs[0],
        l4.byte_offs[2] - l4.byte_offs[1],
        l4.byte_offs[3] - l4.byte_offs[2],
        n - l4.byte_offs[3],
    ];
    let caps = [
        spans[0] + (spans[0] / 4).max(4096) + INLINE,
        spans[1] + (spans[1] / 4).max(4096) + INLINE,
        spans[2] + (spans[2] / 4).max(4096) + INLINE,
        spans[3] + (spans[3] / 4).max(4096) + INLINE,
    ];
    let bases = [
        0usize,
        caps[0],
        caps[0] + caps[1],
        caps[0] + caps[1] + caps[2],
    ];
    let total_cap = bases[3] + caps[3];
    if out.len() < total_cap {
        out.resize(total_cap, 0);
    }

    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let edges = &rep.edges;
    let extra = &rep.extra;
    let spelling = &rep.spelling;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let word_meta = &fd.word_meta;
    let open_lit = fd.open_literals;

    let (mut j0, mut j1, mut j2, mut j3) = (l4.starts[0], l4.starts[1], l4.starts[2], l4.starts[3]);
    let (mut a0, mut a1, mut a2, mut a3) = (0usize, 0usize, 0usize, 0usize);
    let mut run_byte = [0u8; 4];
    let mut run_len = [0u8; 4];
    let mut crc_state = [!0u32, 0u32, 0u32, 0u32];
    let mut crc_at = [0usize; 4];
    let mut first4 = [[0u8; 4]; 4];
    let mut first4_len = [0usize; 4];

    let min_steps = l4.steps[0]
        .min(l4.steps[1])
        .min(l4.steps[2])
        .min(l4.steps[3]);
    let mut overflow = false;

    {
        let (s0, rest) = out[..total_cap].split_at_mut(caps[0]);
        let (s1, rest) = rest.split_at_mut(caps[1]);
        let (s2, s3) = rest.split_at_mut(caps[2]);

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

            emit_fsm_step_slice::<TILE>(
                ed0,
                oj0,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                word_meta,
                open_lit,
                &mut a0,
                &mut run_byte[0],
                &mut run_len[0],
                &mut crc_state[0],
                &mut crc_at[0],
                s0,
                None,
                &mut overflow,
                k,
            );
            emit_fsm_step_slice::<TILE>(
                ed1,
                oj1,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                word_meta,
                open_lit,
                &mut a1,
                &mut run_byte[1],
                &mut run_len[1],
                &mut crc_state[1],
                &mut crc_at[1],
                s1,
                Some((&mut first4[1], &mut first4_len[1])),
                &mut overflow,
                k,
            );
            emit_fsm_step_slice::<TILE>(
                ed2,
                oj2,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                word_meta,
                open_lit,
                &mut a2,
                &mut run_byte[2],
                &mut run_len[2],
                &mut crc_state[2],
                &mut crc_at[2],
                s2,
                Some((&mut first4[2], &mut first4_len[2])),
                &mut overflow,
                k,
            );
            emit_fsm_step_slice::<TILE>(
                ed3,
                oj3,
                bits,
                extra,
                inline16,
                lens,
                spelling,
                word_meta,
                open_lit,
                &mut a3,
                &mut run_byte[3],
                &mut run_len[3],
                &mut crc_state[3],
                &mut crc_at[3],
                s3,
                Some((&mut first4[3], &mut first4_len[3])),
                &mut overflow,
                k,
            );
            if overflow {
                break;
            }
        }

        if !overflow {
            let mut rem_j = [j0, j1, j2, j3];
            let mut rem_a = [a0, a1, a2, a3];
            let slices = [s0, s1, s2, s3];
            for lane in 0..4 {
                for _ in min_steps..l4.steps[lane] {
                    let cur = rem_j[lane];
                    let ed = edges[cur as usize];
                    rem_j[lane] = ed & mask;
                    let f4_arg = if lane > 0 {
                        Some((&mut first4[lane], &mut first4_len[lane]))
                    } else {
                        None
                    };
                    emit_fsm_step_slice::<TILE>(
                        ed,
                        cur,
                        bits,
                        extra,
                        inline16,
                        lens,
                        spelling,
                        word_meta,
                        open_lit,
                        &mut rem_a[lane],
                        &mut run_byte[lane],
                        &mut run_len[lane],
                        &mut crc_state[lane],
                        &mut crc_at[lane],
                        slices[lane],
                        f4_arg,
                        &mut overflow,
                        k,
                    );
                    if overflow {
                        break;
                    }
                }
                if overflow {
                    break;
                }
                crc_state[lane] =
                    crc::update_with(crc_state[lane], &slices[lane][crc_at[lane]..rem_a[lane]], k);
            }

            if !overflow {
                for lane in 1..4 {
                    if check_boundary_straddle(
                        run_byte[lane - 1],
                        run_len[lane - 1],
                        &first4[lane],
                        first4_len[lane],
                    ) {
                        if let Some((new_len, new_crc, new_rb, new_rl)) = repair_lane_tile::<512>(
                            rep,
                            l4.starts[lane],
                            l4.steps[lane],
                            run_byte[lane - 1],
                            run_len[lane - 1],
                            slices[lane],
                            scratch,
                            k,
                        ) {
                            rem_a[lane] = new_len;
                            crc_state[lane] = new_crc;
                            run_byte[lane] = new_rb;
                            run_len[lane] = new_rl;
                        } else {
                            overflow = true;
                            break;
                        }
                    }
                }
                a0 = rem_a[0];
                a1 = rem_a[1];
                a2 = rem_a[2];
                a3 = rem_a[3];
            }
        }
    }

    if overflow {
        let c = fused_1lane_fsm::<TILE, true>(fd, out, k)?;
        return Ok((c, LaneRanges::single(out.len())));
    }

    if run_len[3] == 4 {
        return Err(ErrorKind::RunUnfinished);
    }

    let mut c = crc_combine.combine(crc_state[0], crc_state[1], a1);
    c = crc_combine.combine(c, crc_state[2], a2);
    c = crc_combine.combine(c, crc_state[3], a3);
    let final_crc = !c;

    if STITCH {
        let mut dst = a0;
        out.copy_within(bases[1]..bases[1] + a1, dst);
        dst += a1;
        out.copy_within(bases[2]..bases[2] + a2, dst);
        dst += a2;
        out.copy_within(bases[3]..bases[3] + a3, dst);
        dst += a3;
        out.truncate(dst);
        Ok((final_crc, LaneRanges::single(dst)))
    } else {
        Ok((
            final_crc,
            LaneRanges {
                ranges: [
                    (bases[0], bases[0] + a0),
                    (bases[1], bases[1] + a1),
                    (bases[2], bases[2] + a2),
                    (bases[3], bases[3] + a3),
                ],
                n_ranges: 4,
                four_vecs: false,
            },
        ))
    }
}

// ---------------------------------------------------------------------------
// Arm dispatcher for unified exactness & interleaved timing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArmKind {
    CtrlBidiUnfused,
    Ctrl1LaneUnfused,
    Ctrl4LaneUnfused,
    Floor1LanePlusCrc,
    Floor4LanePlusCrc,
    Fused1LaneFsmLit16k,
    Fused1LaneFsmLit32k,
    Fused1LaneFsmLit64k,
    Fused1LaneFsmUni32k,
    Fused1LaneTile16k,
    Fused1LaneTile32k,
    Fused1LaneTile64k,
    Fused4LaneTile512Stitch,
    Fused4LaneTile512Vectored,
    Fused4LaneTile1024Stitch,
    Fused4LaneTile1024Vectored,
    Fused4LaneTile2048Stitch,
    Fused4LaneTile2048Vectored,
    Fused4LaneTile4096Stitch,
    Fused4LaneTile4096Vectored,
    Fused4LaneTile1024DefCrcVec,
    Fused4LaneTile4096DefCrcVec,
    Fused4LaneFsm32kStitch,
    Fused4LaneFsm32kVectored,
}

impl ArmKind {
    fn name(self) -> &'static str {
        match self {
            Self::CtrlBidiUnfused => "ctrl_bidi_unfused",
            Self::Ctrl1LaneUnfused => "ctrl_1lane_unfused",
            Self::Ctrl4LaneUnfused => "ctrl_4lane_unfused",
            Self::Floor1LanePlusCrc => "floor_1lane_plus_crc",
            Self::Floor4LanePlusCrc => "floor_4lane_plus_crc",
            Self::Fused1LaneFsmLit16k => "fused_1l_fsm_lit_16k",
            Self::Fused1LaneFsmLit32k => "fused_1l_fsm_lit_32k",
            Self::Fused1LaneFsmLit64k => "fused_1l_fsm_lit_64k",
            Self::Fused1LaneFsmUni32k => "fused_1l_fsm_uni_32k",
            Self::Fused1LaneTile16k => "fused_1l_tile_16k",
            Self::Fused1LaneTile32k => "fused_1l_tile_32k",
            Self::Fused1LaneTile64k => "fused_1l_tile_64k",
            Self::Fused4LaneTile512Stitch => "fused_4l_tile512_stitch",
            Self::Fused4LaneTile512Vectored => "fused_4l_tile512_vec",
            Self::Fused4LaneTile1024Stitch => "fused_4l_tile1024_stitch",
            Self::Fused4LaneTile1024Vectored => "fused_4l_tile1024_vec",
            Self::Fused4LaneTile2048Stitch => "fused_4l_tile2048_stitch",
            Self::Fused4LaneTile2048Vectored => "fused_4l_tile2048_vec",
            Self::Fused4LaneTile4096Stitch => "fused_4l_tile4096_stitch",
            Self::Fused4LaneTile4096Vectored => "fused_4l_tile4096_vec",
            Self::Fused4LaneTile1024DefCrcVec => "fused_4l_t1024_defcrc_vec",
            Self::Fused4LaneTile4096DefCrcVec => "fused_4l_t4096_defcrc_vec",
            Self::Fused4LaneFsm32kStitch => "fused_4l_fsm32k_stitch",
            Self::Fused4LaneFsm32kVectored => "fused_4l_fsm32k_vec",
        }
    }
}

struct Scratch {
    out: Vec<u8>,
    lanes: [Vec<u8>; 4],
    runs: Vec<Run>,
    stage: Vec<u8>,
    tile: Vec<u8>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            out: Vec::with_capacity(1_200_000),
            lanes: [
                Vec::with_capacity(1_000_000),
                Vec::with_capacity(300_000),
                Vec::with_capacity(300_000),
                Vec::with_capacity(300_000),
            ],
            runs: Vec::with_capacity(2048),
            stage: Vec::with_capacity(1_000_000),
            tile: Vec::with_capacity(131_072),
        }
    }
}

struct BlockReps {
    bidi: DictReturnMap,
    fwd: FusedDict,
}

#[inline]
fn run_arm_on_block(
    arm: ArmKind,
    blk: &BlockReps,
    sc: &mut Scratch,
    crc_combine: &CrcCombine,
    k: Kernels,
) -> Result<(u32, LaneRanges), ErrorKind> {
    match arm {
        ArmKind::CtrlBidiUnfused => {
            consume_dict_bidi(&blk.bidi, &mut sc.out);
            let c = shipped_unrle_in_place(&mut sc.out, &mut sc.runs, &mut sc.stage, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Ctrl1LaneUnfused => {
            consume_dict_fwd(&blk.fwd.rep, &mut sc.out);
            let c = shipped_unrle_in_place(&mut sc.out, &mut sc.runs, &mut sc.stage, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Ctrl4LaneUnfused => {
            consume_dict_4lane(&blk.fwd.rep, &mut sc.out);
            let c = shipped_unrle_in_place(&mut sc.out, &mut sc.runs, &mut sc.stage, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Floor1LanePlusCrc => {
            consume_dict_fwd(&blk.fwd.rep, &mut sc.out);
            let c = crc::crc32(&sc.out, k);
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Floor4LanePlusCrc => {
            consume_dict_4lane(&blk.fwd.rep, &mut sc.out);
            let c = crc::crc32(&sc.out, k);
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneFsmLit16k => {
            let c = fused_1lane_fsm::<16384, true>(&blk.fwd, &mut sc.out, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneFsmLit32k => {
            let c = fused_1lane_fsm::<32768, true>(&blk.fwd, &mut sc.out, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneFsmLit64k => {
            let c = fused_1lane_fsm::<65536, true>(&blk.fwd, &mut sc.out, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneFsmUni32k => {
            let c = fused_1lane_fsm::<32768, false>(&blk.fwd, &mut sc.out, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneTile16k => {
            let c = fused_1lane_tile_scan::<16384>(&blk.fwd.rep, &mut sc.out, &mut sc.tile, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneTile32k => {
            let c = fused_1lane_tile_scan::<32768>(&blk.fwd.rep, &mut sc.out, &mut sc.tile, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused1LaneTile64k => {
            let c = fused_1lane_tile_scan::<65536>(&blk.fwd.rep, &mut sc.out, &mut sc.tile, k)?;
            Ok((c, LaneRanges::single(sc.out.len())))
        }
        ArmKind::Fused4LaneTile512Stitch => fused_4lane_tile::<512, 65536, true>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile512Vectored => fused_4lane_tile::<512, 65536, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile1024Stitch => fused_4lane_tile::<1024, 65536, true>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile1024Vectored => fused_4lane_tile::<1024, 65536, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile2048Stitch => fused_4lane_tile::<2048, 65536, true>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile2048Vectored => fused_4lane_tile::<2048, 65536, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile4096Stitch => fused_4lane_tile::<4096, 65536, true>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile4096Vectored => fused_4lane_tile::<4096, 65536, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile1024DefCrcVec => fused_4lane_tile::<1024, { usize::MAX }, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneTile4096DefCrcVec => fused_4lane_tile::<4096, { usize::MAX }, false>(
            &blk.fwd.rep,
            &mut sc.out,
            &mut sc.lanes,
            &mut sc.tile,
            crc_combine,
            k,
            None,
        ),
        ArmKind::Fused4LaneFsm32kStitch => {
            fused_4lane_fsm::<32768, true>(&blk.fwd, &mut sc.out, &mut sc.tile, crc_combine, k)
        }
        ArmKind::Fused4LaneFsm32kVectored => {
            fused_4lane_fsm::<32768, false>(&blk.fwd, &mut sc.out, &mut sc.tile, crc_combine, k)
        }
    }
}

fn build_block_reps(blocks: &[BlockInput]) -> Vec<BlockReps> {
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut out = Vec::with_capacity(blocks.len());
    for b in blocks {
        let (_, mb, markb, bitsb) = builder.construct_timed(
            &b.records,
            b.n as u32,
            b.orig_ptr,
            PROD_LIMIT,
            true,
            &mut scratch,
        );
        let bidi = builder.snapshot_dict_rep(b.n, mb, markb, bitsb);
        let (_, mf, markf, bitsf) = builder.construct_timed(
            &b.records,
            b.n as u32,
            b.orig_ptr,
            PROD_LIMIT,
            false,
            &mut scratch,
        );
        let fwd_raw = builder.snapshot_dict_rep(b.n, mf, markf, bitsf);
        let fwd = make_fused_dict(fwd_raw, true);
        out.push(BlockReps { bidi, fwd });
    }
    out
}

// ---------------------------------------------------------------------------
// Exactness verification suite
// ---------------------------------------------------------------------------

const ALL_EXACT_ARMS: &[ArmKind] = &[
    ArmKind::CtrlBidiUnfused,
    ArmKind::Ctrl1LaneUnfused,
    ArmKind::Ctrl4LaneUnfused,
    ArmKind::Fused1LaneFsmLit16k,
    ArmKind::Fused1LaneFsmLit32k,
    ArmKind::Fused1LaneFsmLit64k,
    ArmKind::Fused1LaneFsmUni32k,
    ArmKind::Fused1LaneTile16k,
    ArmKind::Fused1LaneTile32k,
    ArmKind::Fused1LaneTile64k,
    ArmKind::Fused4LaneTile512Stitch,
    ArmKind::Fused4LaneTile512Vectored,
    ArmKind::Fused4LaneTile1024Stitch,
    ArmKind::Fused4LaneTile1024Vectored,
    ArmKind::Fused4LaneTile2048Stitch,
    ArmKind::Fused4LaneTile2048Vectored,
    ArmKind::Fused4LaneTile4096Stitch,
    ArmKind::Fused4LaneTile4096Vectored,
    ArmKind::Fused4LaneTile1024DefCrcVec,
    ArmKind::Fused4LaneTile4096DefCrcVec,
    ArmKind::Fused4LaneFsm32kStitch,
    ArmKind::Fused4LaneFsm32kVectored,
];

fn verify_reps_exact(
    label: &str,
    blocks: &[BlockInput],
    reps: &[BlockReps],
    arms: &[ArmKind],
    crc_combine: &CrcCombine,
    k: Kernels,
) {
    let mut sc = Scratch::new();
    let mut want_buf = Vec::new();
    for (bi, (b, r)) in blocks.iter().zip(reps.iter()).enumerate() {
        want_buf.clear();
        let want_res = rle::unrle(&b.walked, &mut want_buf, k).map(|()| crc::crc32(&want_buf, k));
        for &arm in arms {
            let got = run_arm_on_block(arm, r, &mut sc, crc_combine, k);
            match (&want_res, got) {
                (Ok(wcrc), Ok((gcrc, ranges))) => {
                    assert_eq!(
                        gcrc,
                        *wcrc,
                        "CRC mismatch on {label} blk #{bi} (stream {} n={}) arm {}",
                        b.stream_idx,
                        b.n,
                        arm.name()
                    );
                    let got_bytes = ranges.materialize(&sc);
                    assert_eq!(
                        got_bytes.len(),
                        want_buf.len(),
                        "len mismatch on {label} blk #{bi} arm {}",
                        arm.name()
                    );
                    assert!(
                        got_bytes == want_buf,
                        "byte mismatch on {label} blk #{bi} arm {}",
                        arm.name()
                    );
                }
                (Err(we), Err(ge)) => {
                    assert_eq!(
                        ge,
                        *we,
                        "error mismatch on {label} blk #{bi} arm {}",
                        arm.name()
                    );
                }
                (w, g) => {
                    panic!(
                        "Ok/Err mismatch on {label} blk #{bi} arm {}: want {:?}, got {:?}",
                        arm.name(),
                        w.as_ref().map(|_| "Ok"),
                        g.as_ref().map(|_| "Ok")
                    );
                }
            }
        }
    }
    println!(
        "  [EXACT] {:24} {:4} blocks x {:2} arms = {:5} checks OK",
        label,
        blocks.len(),
        arms.len(),
        blocks.len() * arms.len()
    );
}

fn build_synthetic_exactness_blocks() -> Vec<BlockInput> {
    let mut texts: Vec<Vec<u8>> = vec![
        b"a".to_vec(),
        b"aaa".to_vec(),
        b"aaaa".to_vec(), // RunUnfinished
        b"aaaa\0".to_vec(),
        b"aaaa\x01".to_vec(),
        b"aaaa\xff".to_vec(),
        b"xaaaa".to_vec(), // RunUnfinished
        b"aaaa\0bbbb\0cccc\0".to_vec(),
        b"aaaa\xffbbbb\0cccc\x07z".to_vec(),
        b"aaaaa".to_vec(),
        b"aaaaaaaaa".to_vec(),
        b"aaaaaaaaaa".to_vec(),
        [b"q".repeat(31), b"aaaa\x05".to_vec(), b"q".repeat(40)].concat(),
        [b"q".repeat(30), b"rrrr\0".to_vec(), b"s".repeat(3)].concat(),
        [b"0123".repeat(100), b"zzzz".to_vec()].concat(), // RunUnfinished
        [b"0123".repeat(100), b"zzzz\0".to_vec()].concat(),
        // Periodic short cycles with runs
        b"ab".repeat(250),
        b"aaaa\x03".repeat(120),
        b"aaaa\x00".repeat(120),
    ];

    // Explicit lane-boundary straddle stress cases: non-repetitive base of length 1024
    // (guarantees 4 lanes!) with runs placed around 1/4, 1/2, 3/4 boundaries at every offset -4..=+4
    // and with extra = 0, 1, 17, 255, plus RunUnfinished at end of lane 3.
    let mut base = Vec::with_capacity(1024);
    let mut x = 0x1234_5678_9abc_def0u64;
    for i in 0..1024usize {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        // Avoid accidental 4-equal runs in base
        let b = (x as u8 & 0x3F) + b' ' + ((i & 1) as u8 * 64);
        base.push(b);
    }
    for &extra in &[0u8, 1, 17, 255] {
        for off in -4isize..=4 {
            let mut t = base.clone();
            for &bpos in &[256isize, 512, 768] {
                let p = (bpos + off) as usize;
                t[p] = b'!';
                t[p + 1] = b'!';
                t[p + 2] = b'!';
                t[p + 3] = b'!';
                t[p + 4] = extra;
            }
            texts.push(t);
        }
    }
    // Consecutive runs right across boundary + RunUnfinished at EOF on a 4-lane block
    {
        let mut t = base.clone();
        for p in [253usize, 258, 509, 514, 765, 770] {
            t[p..p + 4].fill(b'Z');
            t[p + 4] = 2;
        }
        t[1020..1024].fill(b'Q');
        texts.push(t);
    }

    let mut blocks = Vec::with_capacity(texts.len());
    for (i, t) in texts.iter().enumerate() {
        let (l, orig_ptr) = forward_bwt(t);
        let blk = block_from_column(&l, orig_ptr, i);
        assert_eq!(blk.walked, *t, "forward_bwt roundtrip failed on synth #{i}");
        blocks.push(blk);
    }
    blocks
}

// ---------------------------------------------------------------------------
// Interleaved benchmark helpers (warm & cold)
// ---------------------------------------------------------------------------

struct ThreadCpuClock {
    origin: Instant,
    use_schedstat: bool,
    buf: String,
}

impl ThreadCpuClock {
    fn new(force_wall: bool) -> Self {
        let use_schedstat = !force_wall
            && std::fs::read_to_string("/proc/thread-self/schedstat")
                .ok()
                .and_then(|s| s.split_whitespace().next()?.parse::<u64>().ok())
                .is_some();
        Self {
            origin: Instant::now(),
            use_schedstat,
            buf: String::with_capacity(64),
        }
    }

    fn label(&self) -> &'static str {
        if self.use_schedstat {
            "thread-cpu (/proc/thread-self/schedstat)"
        } else {
            "monotonic wall (Instant::now)"
        }
    }

    #[inline]
    fn now_ns(&mut self) -> u64 {
        if self.use_schedstat {
            self.buf.clear();
            if let Ok(mut f) = std::fs::File::open("/proc/thread-self/schedstat") {
                use std::io::Read;
                if f.read_to_string(&mut self.buf).is_ok() {
                    if let Some(tok) = self.buf.split_whitespace().next() {
                        if let Ok(v) = tok.parse::<u64>() {
                            return v;
                        }
                    }
                }
            }
        }
        self.origin.elapsed().as_nanos() as u64
    }
}

fn measure_warm_pass(
    arm: ArmKind,
    reps: &[BlockReps],
    k_warm: usize,
    sc: &mut Scratch,
    clock: &mut ThreadCpuClock,
    crc_combine: &CrcCombine,
    k: Kernels,
) -> (f64, u32) {
    if !clock.use_schedstat {
        let mut total_ns = 0u64;
        let mut xor_crc = 0u32;
        for r in reps {
            // Untimed in-situ per-block warm-up (identical to ideal_permutation_structures.rs)
            let _ = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
            black_box(&sc.out);
            black_box(&sc.lanes);
            let t0 = clock.now_ns();
            let (c, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
            let t1 = clock.now_ns();
            total_ns += t1.saturating_sub(t0);
            xor_crc ^= c;
            black_box(&sc.out);
            black_box(&sc.lanes);
        }
        return (total_ns as f64 / 1e6, xor_crc);
    }

    // Preemption-immune single-span differential measurement:
    // T_1 = 1 call per block (entry state = cold/L3-evicted for multi-block, warm for 1-block)
    // T_{1+k} = 1 entry call + k in-situ warm calls per block
    // Warm per-pass CPU time = (T_{1+k} - T_1) / k, using only 2 schedstat reads per span.
    let mut xor_crc = 0u32;
    let t0 = clock.now_ns();
    for r in reps {
        let (c, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
        xor_crc ^= c;
        black_box(&sc.out);
        black_box(&sc.lanes);
    }
    let t1 = clock.now_ns();
    let base_ns = t1.saturating_sub(t0);

    let mut xor_crc2 = 0u32;
    let t2 = clock.now_ns();
    for r in reps {
        let (c0, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
        black_box(&sc.out);
        black_box(&sc.lanes);
        for _ in 0..k_warm {
            let (c, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
            black_box(&sc.out);
            black_box(&sc.lanes);
            debug_assert_eq!(c, c0);
        }
        xor_crc2 ^= c0;
    }
    let t3 = clock.now_ns();
    assert_eq!(xor_crc, xor_crc2);
    let multi_ns = t3.saturating_sub(t2);
    let warm_ms = multi_ns.saturating_sub(base_ns) as f64 / (k_warm as f64 * 1e6);
    (warm_ms, xor_crc)
}

#[allow(clippy::too_many_arguments)]
fn measure_cold_pass(
    arm: ArmKind,
    reps: &[BlockReps],
    tiny_wall_fallback: bool,
    flush_buf: &mut [u8],
    sc: &mut Scratch,
    clock: &mut ThreadCpuClock,
    crc_combine: &CrcCombine,
    k: Kernels,
) -> (f64, u32) {
    flush_caches(flush_buf);
    let mut xor_crc = 0u32;
    if tiny_wall_fallback {
        let t0 = Instant::now();
        for r in reps {
            let (c, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
            xor_crc ^= c;
            black_box(&sc.out);
            black_box(&sc.lanes);
        }
        return (t0.elapsed().as_nanos() as f64 / 1e6, xor_crc);
    }
    let t0 = clock.now_ns();
    for r in reps {
        let (c, _) = run_arm_on_block(arm, r, sc, crc_combine, k).unwrap();
        xor_crc ^= c;
        black_box(&sc.out);
        black_box(&sc.lanes);
    }
    let t1 = clock.now_ns();
    (t1.saturating_sub(t0) as f64 / 1e6, xor_crc)
}

struct BenchResult {
    arm: ArmKind,
    warm: (f64, f64, f64), // (med, min, max)
    cold: (f64, f64, f64),
}

fn bench_arms(
    reps: &[BlockReps],
    arms: &[ArmKind],
    repeats: usize,
    force_wall: bool,
    flush_buf: &mut [u8],
    crc_combine: &CrcCombine,
    k: Kernels,
) -> Vec<BenchResult> {
    let mut sc = Scratch::new();
    let mut clock = ThreadCpuClock::new(force_wall);
    // 1 warmup sweep + estimate single-pass cost to size k_warm cleanly
    let w0 = Instant::now();
    for r in reps {
        let _ = run_arm_on_block(arms[0], r, &mut sc, crc_combine, k).unwrap();
        black_box(&sc.out);
        black_box(&sc.lanes);
    }
    let est_ms = w0.elapsed().as_secs_f64() * 1e3;
    let k_warm = if clock.use_schedstat {
        (350.0 / est_ms.max(1.0)).ceil().clamp(3.0, 40.0) as usize
    } else {
        1
    };
    let tiny_cold_wall = clock.use_schedstat && est_ms < 25.0;
    for &arm in &arms[1..] {
        for r in reps {
            let _ = run_arm_on_block(arm, r, &mut sc, crc_combine, k).unwrap();
            black_box(&sc.out);
            black_box(&sc.lanes);
        }
    }

    let mut warm_samples: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut cold_samples: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut crc_xors = vec![0u32; arms.len()];

    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..arms.len()).collect()
        } else {
            (0..arms.len()).rev().collect()
        };
        for &ai in &order {
            let (ms, cx) =
                measure_warm_pass(arms[ai], reps, k_warm, &mut sc, &mut clock, crc_combine, k);
            warm_samples[ai].push(ms);
            crc_xors[ai] = cx;
        }
        for &ai in &order {
            let (ms, cx) = measure_cold_pass(
                arms[ai],
                reps,
                tiny_cold_wall,
                flush_buf,
                &mut sc,
                &mut clock,
                crc_combine,
                k,
            );
            cold_samples[ai].push(ms);
            assert_eq!(cx, crc_xors[ai], "warm/cold CRC mismatch!");
        }
    }

    let ref_crc = crc_xors[0];
    for (i, &arm) in arms.iter().enumerate() {
        if !matches!(arm, ArmKind::Floor1LanePlusCrc | ArmKind::Floor4LanePlusCrc) {
            assert_eq!(
                crc_xors[i],
                ref_crc,
                "CRC XOR mismatch during bench for arm {}",
                arm.name()
            );
        }
    }

    arms.iter()
        .enumerate()
        .map(|(i, &arm)| BenchResult {
            arm,
            warm: median_min_max(&warm_samples[i]),
            cold: median_min_max(&cold_samples[i]),
        })
        .collect()
}

fn non_overlap_tag(a: (f64, f64, f64), b: (f64, f64, f64)) -> &'static str {
    if a.2 < b.1 {
        "FASTER"
    } else if a.1 > b.2 {
        "SLOWER"
    } else {
        "overlap"
    }
}

fn print_bench_table(title: &str, results: &[BenchResult], scale_to_i9_bidi_ms: Option<f64>) {
    println!("\n=== {title} ===");
    let bidi = results
        .iter()
        .find(|r| r.arm == ArmKind::CtrlBidiUnfused)
        .unwrap_or(&results[0]);
    let fwd1 = results
        .iter()
        .find(|r| r.arm == ArmKind::Ctrl1LaneUnfused)
        .unwrap_or(&results[0]);
    let fwd4 = results
        .iter()
        .find(|r| r.arm == ArmKind::Ctrl4LaneUnfused)
        .unwrap_or(fwd1);

    // Faster unfused forward control (per regime)
    let best_unfused_w = if fwd4.warm.0 <= fwd1.warm.0 {
        fwd4
    } else {
        fwd1
    };
    let best_unfused_c = if fwd4.cold.0 <= fwd1.cold.0 {
        fwd4
    } else {
        fwd1
    };

    println!(
        "{:<26} | {:>23} | {:>7} {:>7} {:>7} | {:>23} | {:>7} {:>7} {:>7}",
        "arm",
        "warm med [min..max] ms",
        "vs1l",
        "vs4l",
        "nov4l",
        "cold med [min..max] ms",
        "vs1l",
        "vs4l",
        "nov4l"
    );
    println!("{}", "-".repeat(128));
    for r in results {
        println!(
            "{:<26} | {:6.2} [{:6.2}..{:6.2}] | {:6.3}x {:6.3}x {:>7} | {:6.2} [{:6.2}..{:6.2}] | {:6.3}x {:6.3}x {:>7}",
            r.arm.name(),
            r.warm.0,
            r.warm.1,
            r.warm.2,
            r.warm.0 / fwd1.warm.0,
            r.warm.0 / fwd4.warm.0,
            non_overlap_tag(r.warm, best_unfused_w.warm),
            r.cold.0,
            r.cold.1,
            r.cold.2,
            r.cold.0 / fwd1.cold.0,
            r.cold.0 / fwd4.cold.0,
            non_overlap_tag(r.cold, best_unfused_c.cold),
        );
    }

    if let Some(i9_anchor_ms) = scale_to_i9_bidi_ms {
        println!(
            "\n--- VM-to-i9 1W projection (anchored at ctrl_bidi_unfused = {i9_anchor_ms:.1} ms on wikidata x4; 3% gate = {I9_GATE_3PCT_MS:.2} ms) ---"
        );
        println!(
            "{:<26} | {:>10} {:>10} {:>10} {:>8} | {:>10} {:>10} {:>10} {:>8}",
            "arm",
            "warm_i9",
            "d_vs_1l",
            "d_vs_best",
            "%wall_w",
            "cold_i9",
            "d_vs_1l",
            "d_vs_best",
            "%wall_c"
        );
        println!("{}", "-".repeat(110));
        for r in results {
            let w_i9 = i9_anchor_ms * (r.warm.0 / bidi.warm.0);
            let w_1l = i9_anchor_ms * (fwd1.warm.0 / bidi.warm.0);
            let w_best = i9_anchor_ms * (best_unfused_w.warm.0 / bidi.warm.0);
            let c_i9 = i9_anchor_ms * (r.cold.0 / bidi.cold.0);
            let c_1l = i9_anchor_ms * (fwd1.cold.0 / bidi.cold.0);
            let c_best = i9_anchor_ms * (best_unfused_c.cold.0 / bidi.cold.0);
            println!(
                "{:<26} | {:9.1}m {:+9.1}m {:+9.1}m {:+7.2}% | {:9.1}m {:+9.1}m {:+9.1}m {:+7.2}%",
                r.arm.name(),
                w_i9,
                w_1l - w_i9,
                w_best - w_i9,
                100.0 * (w_best - w_i9) / I9_WALL_1W_MS,
                c_i9,
                c_1l - c_i9,
                c_best - c_i9,
                100.0 * (c_best - c_i9) / I9_WALL_1W_MS,
            );
        }
    }
}

fn measure_word_meta_build(
    reps: &[BlockReps],
    repeats: usize,
    force_wall: bool,
) -> (f64, f64, f64) {
    let mut meta = Vec::new();
    let mut clock = ThreadCpuClock::new(force_wall);
    let inner = if clock.use_schedstat { 16usize } else { 1 };
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let t0 = clock.now_ns();
        let mut acc = 0usize;
        for _ in 0..inner {
            for r in reps {
                let (m, wr) = build_word_meta_into(&r.fwd.rep, r.fwd.open_literals, &mut meta);
                acc += m + wr;
                black_box(&meta);
            }
        }
        black_box(acc);
        let t1 = clock.now_ns();
        samples.push(t1.saturating_sub(t0) as f64 / (inner as f64 * 1e6));
    }
    median_min_max(&samples)
}

fn load_all_blocks(path: &str) -> Vec<BlockInput> {
    let (mut adm, mut non_adm) = load_wikidata(path);
    adm.append(&mut non_adm);
    adm.sort_by_key(|b| b.stream_idx);
    adm
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "Usage: forward_rle1_crc_fusion <wikidata.bz2> [--ab-dir DIR] [--repeats N] [--skip-f1b] [--wall]"
        );
        std::process::exit(1);
    }
    let wikidata_path = args[0].clone();
    let mut ab_dir: Option<String> = None;
    let mut repeats = 7usize;
    let mut skip_f1b = false;
    let mut force_wall = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ab-dir" => {
                ab_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--repeats" => {
                repeats = args[i + 1].parse().expect("repeats int");
                i += 2;
            }
            "--skip-f1b" => {
                skip_f1b = true;
                i += 1;
            }
            "--wall" => {
                force_wall = true;
                i += 1;
            }
            other => {
                eprintln!("Unknown arg: {other}");
                std::process::exit(1);
            }
        }
    }

    let k = Kernels::detect();
    let crc_combine = CrcCombine::new();
    let clock_probe = ThreadCpuClock::new(force_wall);
    println!("forward-rle1-crc-fusion smooth-zone probe (#![forbid(unsafe_code)])");
    println!(
        "Kernels: {k:?}, repeats: {repeats}, clock: {}",
        clock_probe.label()
    );

    // -----------------------------------------------------------------------
    // Step 1: Synthetic exactness (including lane-boundary straddles & RunUnfinished)
    // -----------------------------------------------------------------------
    println!("\n=== Step 1: Exactness verification ===");
    let synth_blocks = build_synthetic_exactness_blocks();
    let synth_reps = build_block_reps(&synth_blocks);
    verify_reps_exact(
        "synthetic+straddles",
        &synth_blocks,
        &synth_reps,
        ALL_EXACT_ARMS,
        &crc_combine,
        k,
    );

    // -----------------------------------------------------------------------
    // Step 2: Load wikidata.bz2 (all 431 blocks) and verify 100% exactness
    // -----------------------------------------------------------------------
    let (admitted, non_admitted) = load_wikidata(&wikidata_path);
    println!(
        "Loaded {wikidata_path}: {} admitted + {} non-admitted = {} total blocks",
        admitted.len(),
        non_admitted.len(),
        admitted.len() + non_admitted.len()
    );
    assert_eq!(admitted.len() + non_admitted.len(), 431);
    let adm_reps = build_block_reps(&admitted);
    let non_adm_reps = build_block_reps(&non_admitted);
    verify_reps_exact(
        "wikidata_admitted",
        &admitted,
        &adm_reps,
        ALL_EXACT_ARMS,
        &crc_combine,
        k,
    );
    verify_reps_exact(
        "wikidata_non_admitted",
        &non_admitted,
        &non_adm_reps,
        ALL_EXACT_ARMS,
        &crc_combine,
        k,
    );

    // -----------------------------------------------------------------------
    // Step 3: Load & verify all 6 ab gen corpora (if --ab-dir provided)
    // -----------------------------------------------------------------------
    let mut ab_corpora: Vec<(String, Vec<BlockInput>, Vec<BlockReps>)> = Vec::new();
    if let Some(dir) = &ab_dir {
        for name in ["text", "random", "onebyte", "pattern", "runs", "records"] {
            let p = Path::new(dir).join(format!("{name}.bz2"));
            let blks = load_all_blocks(p.to_str().unwrap());
            let reps = build_block_reps(&blks);
            verify_reps_exact(name, &blks, &reps, ALL_EXACT_ARMS, &crc_combine, k);
            ab_corpora.push((name.to_string(), blks, reps));
        }
    }

    // -----------------------------------------------------------------------
    // Step 4: Structural census (WordMeta, boundary straddles, write traffic)
    // -----------------------------------------------------------------------
    println!("\n=== Step 2: Held-out wikidata census & write-traffic accounting ===");
    let held_blocks = &admitted[TRAIN_SPLIT_BLOCKS..];
    let held_reps = &adm_reps[TRAIN_SPLIT_BLOCKS..];
    let held_pre_bytes: u64 = held_blocks.iter().map(|b| b.n as u64).sum();
    let multi_words: usize = held_reps.iter().map(|r| r.fwd.meta_multi_words).sum();
    let run_words: usize = held_reps.iter().map(|r| r.fwd.meta_words_with_run).sum();
    let mut st4 = FusionStats::default();
    let mut sc_c = Scratch::new();
    for r in held_reps {
        let _ = fused_4lane_tile::<512, 65536, true>(
            &r.fwd.rep,
            &mut sc_c.out,
            &mut sc_c.lanes,
            &mut sc_c.tile,
            &crc_combine,
            k,
            Some(&mut st4),
        )
        .unwrap();
    }
    let wm_ms = measure_word_meta_build(held_reps, repeats, force_wall);
    println!(
        "Held-out admitted: {} blocks, {} pre-RLE1 bytes ({:.2} MiB)",
        held_blocks.len(),
        held_pre_bytes,
        held_pre_bytes as f64 / (1024.0 * 1024.0)
    );
    println!(
        "WordMeta: {} multi-byte words, {} ({:.2}%) have internal 4B run; build time: {:.2} [{:.2}..{:.2}] ms",
        multi_words,
        run_words,
        100.0 * run_words as f64 / multi_words.max(1) as f64,
        wm_ms.0,
        wm_ms.1,
        wm_ms.2
    );
    println!(
        "4-lane census (held-out 116 blks): straddles={}, overflow_fallbacks={}, no_lanes={}, lanes_grown_zero={}/{} ({:.1}%), out_written={:.2} MiB, stitch_moved={:.2} MiB ({:.1}% of out)",
        st4.boundary_straddles,
        st4.overflow_fallbacks,
        st4.no_lanes_fallbacks,
        st4.lanes_grown_zero,
        st4.lanes_total,
        100.0 * st4.lanes_grown_zero as f64 / st4.lanes_total.max(1) as f64,
        st4.out_bytes_written as f64 / (1024.0 * 1024.0),
        st4.stitch_bytes_moved as f64 / (1024.0 * 1024.0),
        100.0 * st4.stitch_bytes_moved as f64 / st4.out_bytes_written.max(1) as f64,
    );

    // Exact write traffic accounting on full 431 wikidata blocks (scaled x4)
    let mut all_unfused_a9_writes = 0u64;
    let mut all_out_bytes = 0u64;
    let mut all_pre_bytes = 0u64;
    let mut all_straddles = 0usize;
    let mut all_stitch_bytes = 0u64;
    for (b, r) in admitted
        .iter()
        .zip(adm_reps.iter())
        .chain(non_admitted.iter().zip(non_adm_reps.iter()))
    {
        all_pre_bytes += b.n as u64;
        // Count shipped_unrle_in_place logical writes
        let mut runs = Vec::new();
        find_runs_fast(&b.walked, &mut runs).unwrap();
        if let Some(&(first, _, _)) = runs.first() {
            let (mut grown, mut shift) = (0isize, 0isize);
            for &(_, _, ex) in &runs {
                grown += ex as isize - 1;
                shift = shift.max(grown);
            }
            let len = (b.n as isize + grown) as u64;
            all_out_bytes += len;
            let first = first as u64;
            if shift as usize > b.n {
                all_unfused_a9_writes += len;
            } else {
                if shift > 0 {
                    all_unfused_a9_writes += b.n as u64 - first;
                }
                all_unfused_a9_writes += len - first;
            }
        } else {
            all_out_bytes += b.n as u64;
        }
        let mut st = FusionStats::default();
        let _ = fused_4lane_tile::<512, 65536, true>(
            &r.fwd.rep,
            &mut sc_c.out,
            &mut sc_c.lanes,
            &mut sc_c.tile,
            &crc_combine,
            k,
            Some(&mut st),
        )
        .unwrap();
        all_straddles += st.boundary_straddles;
        all_stitch_bytes += st.stitch_bytes_moved;
    }
    let mib4 = |bytes: u64| (bytes * 4) as f64 / (1024.0 * 1024.0);
    let unfused_total_wr = all_pre_bytes + all_unfused_a9_writes;
    let fused_1l_wr = all_out_bytes;
    let fused_4l_stitch_wr = all_out_bytes + all_stitch_bytes;
    let fused_4l_vec_wr = all_out_bytes;
    println!(
        "\nFull wikidata x4 logical write traffic (E + A9, 1,724 blocks, all_straddles={}):",
        all_straddles
    );
    println!(
        "  unfused E + A9:         E={:.1} + A9={:.1} = {:.1} MiB (baseline 0.0%)",
        mib4(all_pre_bytes),
        mib4(all_unfused_a9_writes),
        mib4(unfused_total_wr),
    );
    println!(
        "  fused 1-lane (in-out):  {:.1} MiB ({:+.1}%)",
        mib4(fused_1l_wr),
        100.0 * (fused_1l_wr as f64 - unfused_total_wr as f64) / unfused_total_wr as f64,
    );
    println!(
        "  fused 4-lane stitched:  {:.1} MiB ({:+.1}%)",
        mib4(fused_4l_stitch_wr),
        100.0 * (fused_4l_stitch_wr as f64 - unfused_total_wr as f64) / unfused_total_wr as f64,
    );
    println!(
        "  fused 4-lane vectored:  {:.1} MiB ({:+.1}%)",
        mib4(fused_4l_vec_wr),
        100.0 * (fused_4l_vec_wr as f64 - unfused_total_wr as f64) / unfused_total_wr as f64,
    );

    // -----------------------------------------------------------------------
    // Step 5: Main benchmark on held-out admitted wikidata blocks (116 blocks)
    // -----------------------------------------------------------------------
    let mut flush_buf = vec![0u8; COLD_FLUSH_BYTES];
    let main_arms = &[
        ArmKind::CtrlBidiUnfused,
        ArmKind::Ctrl1LaneUnfused,
        ArmKind::Ctrl4LaneUnfused,
        ArmKind::Floor1LanePlusCrc,
        ArmKind::Floor4LanePlusCrc,
        ArmKind::Fused1LaneFsmLit64k,
        ArmKind::Fused1LaneFsmUni32k,
        ArmKind::Fused1LaneTile16k,
        ArmKind::Fused1LaneTile32k,
        ArmKind::Fused4LaneTile1024Stitch,
        ArmKind::Fused4LaneTile1024Vectored,
        ArmKind::Fused4LaneTile2048Stitch,
        ArmKind::Fused4LaneTile2048Vectored,
        ArmKind::Fused4LaneTile4096Vectored,
        ArmKind::Fused4LaneTile1024DefCrcVec,
        ArmKind::Fused4LaneTile4096DefCrcVec,
        ArmKind::Fused4LaneFsm32kStitch,
        ArmKind::Fused4LaneFsm32kVectored,
    ];
    let main_res = bench_arms(
        held_reps,
        main_arms,
        repeats,
        force_wall,
        &mut flush_buf,
        &crc_combine,
        k,
    );
    print_bench_table(
        "Held-out admitted wikidata (116 blocks, uncontracted Open LIMIT=16): Full E + A9",
        &main_res,
        Some(I9_EA9_BIDI_1W_MS),
    );

    // Also print standalone E vs A9 delta from floor/ctrl
    let c_bidi = main_res
        .iter()
        .find(|r| r.arm == ArmKind::CtrlBidiUnfused)
        .unwrap();
    let c_1l = main_res
        .iter()
        .find(|r| r.arm == ArmKind::Ctrl1LaneUnfused)
        .unwrap();
    let c_4l = main_res
        .iter()
        .find(|r| r.arm == ArmKind::Ctrl4LaneUnfused)
        .unwrap();
    let f_1l = main_res
        .iter()
        .find(|r| r.arm == ArmKind::Floor1LanePlusCrc)
        .unwrap();
    let f_4l = main_res
        .iter()
        .find(|r| r.arm == ArmKind::Floor4LanePlusCrc)
        .unwrap();
    println!("\n  Floor breakdown (unfused - crc_only_floor = removable RLE1 scan+compaction):");
    println!(
        "    1-lane: warm {:.2} - {:.2} = {:.2} ms ({:.1}%); cold {:.2} - {:.2} = {:.2} ms ({:.1}%)",
        c_1l.warm.0,
        f_1l.warm.0,
        c_1l.warm.0 - f_1l.warm.0,
        100.0 * (c_1l.warm.0 - f_1l.warm.0) / c_1l.warm.0,
        c_1l.cold.0,
        f_1l.cold.0,
        c_1l.cold.0 - f_1l.cold.0,
        100.0 * (c_1l.cold.0 - f_1l.cold.0) / c_1l.cold.0,
    );
    println!(
        "    4-lane: warm {:.2} - {:.2} = {:.2} ms ({:.1}%); cold {:.2} - {:.2} = {:.2} ms ({:.1}%)",
        c_4l.warm.0,
        f_4l.warm.0,
        c_4l.warm.0 - f_4l.warm.0,
        100.0 * (c_4l.warm.0 - f_4l.warm.0) / c_4l.warm.0,
        c_4l.cold.0,
        f_4l.cold.0,
        c_4l.cold.0 - f_4l.cold.0,
        100.0 * (c_4l.cold.0 - f_4l.cold.0) / c_4l.cold.0,
    );

    // -----------------------------------------------------------------------
    // Step 6: Interaction with contracted return maps (F1b_repair_unb / F4a)
    // -----------------------------------------------------------------------
    if !skip_f1b {
        println!("\n=== Step 3: Building F1b_repair_unb (tau=2) on 116 held-out blocks ===");
        let t_f1 = Instant::now();
        let f1b_reps: Vec<BlockReps> = held_reps
            .iter()
            .map(|r| {
                let c_rep = build_f1_repair(&r.fwd.rep, 2, 0);
                let fd = make_fused_dict(c_rep, true);
                BlockReps {
                    bidi: r.bidi.clone(),
                    fwd: fd,
                }
            })
            .collect();
        println!(
            "Built 116 F1b_repair_unb blocks in {:.2} s",
            t_f1.elapsed().as_secs_f64()
        );
        let f1b_arms = &[
            ArmKind::Ctrl1LaneUnfused,
            ArmKind::Ctrl4LaneUnfused,
            ArmKind::Floor1LanePlusCrc,
            ArmKind::Floor4LanePlusCrc,
            ArmKind::Fused1LaneFsmUni32k,
            ArmKind::Fused1LaneTile16k,
            ArmKind::Fused1LaneTile32k,
            ArmKind::Fused4LaneTile512Stitch,
            ArmKind::Fused4LaneTile512Vectored,
            ArmKind::Fused4LaneTile1024Vectored,
            ArmKind::Fused4LaneFsm32kVectored,
        ];
        verify_reps_exact(
            "held_out_F1b_repair_unb",
            held_blocks,
            &f1b_reps,
            &[
                ArmKind::Ctrl1LaneUnfused,
                ArmKind::Ctrl4LaneUnfused,
                ArmKind::Fused1LaneFsmUni32k,
                ArmKind::Fused1LaneTile16k,
                ArmKind::Fused1LaneTile32k,
                ArmKind::Fused1LaneTile64k,
                ArmKind::Fused4LaneTile512Stitch,
                ArmKind::Fused4LaneTile512Vectored,
                ArmKind::Fused4LaneTile1024Stitch,
                ArmKind::Fused4LaneTile1024Vectored,
                ArmKind::Fused4LaneFsm32kStitch,
                ArmKind::Fused4LaneFsm32kVectored,
            ],
            &crc_combine,
            k,
        );
        let f1b_wm = measure_word_meta_build(&f1b_reps, repeats, force_wall);
        let f1b_mw: usize = f1b_reps.iter().map(|r| r.fwd.meta_multi_words).sum();
        let f1b_rw: usize = f1b_reps.iter().map(|r| r.fwd.meta_words_with_run).sum();
        println!(
            "F1b WordMeta: {} multi-byte words, {} ({:.2}%) have internal 4B run; build time: {:.2} [{:.2}..{:.2}] ms",
            f1b_mw,
            f1b_rw,
            100.0 * f1b_rw as f64 / f1b_mw.max(1) as f64,
            f1b_wm.0,
            f1b_wm.1,
            f1b_wm.2
        );
        let f1b_res = bench_arms(
            &f1b_reps,
            f1b_arms,
            repeats,
            force_wall,
            &mut flush_buf,
            &crc_combine,
            k,
        );
        print_bench_table(
            "Contracted return map F1b_repair_unb / F4a (116 held-out blocks): Full E + A9",
            &f1b_res,
            None,
        );
        // Also project F1b/F4a arms to i9 using uncontracted ctrl_bidi_unfused as the 560.8 ms anchor
        println!(
            "\n--- F1b/F4a VM-to-i9 1W projection (anchored at uncontracted ctrl_bidi_unfused = {:.2}/{:.2} ms -> {:.1} ms) ---",
            c_bidi.warm.0, c_bidi.cold.0, I9_EA9_BIDI_1W_MS
        );
        let f1_u1 = f1b_res
            .iter()
            .find(|r| r.arm == ArmKind::Ctrl1LaneUnfused)
            .unwrap();
        let f1_u4 = f1b_res
            .iter()
            .find(|r| r.arm == ArmKind::Ctrl4LaneUnfused)
            .unwrap();
        for r in &f1b_res {
            let w_i9 = I9_EA9_BIDI_1W_MS * (r.warm.0 / c_bidi.warm.0);
            let w_u4 = I9_EA9_BIDI_1W_MS * (f1_u4.warm.0 / c_bidi.warm.0);
            let w_u1 = I9_EA9_BIDI_1W_MS * (f1_u1.warm.0 / c_bidi.warm.0);
            let c_i9 = I9_EA9_BIDI_1W_MS * (r.cold.0 / c_bidi.cold.0);
            let c_u4 = I9_EA9_BIDI_1W_MS * (f1_u4.cold.0 / c_bidi.cold.0);
            let c_u1 = I9_EA9_BIDI_1W_MS * (f1_u1.cold.0 / c_bidi.cold.0);
            println!(
                "  {:<26} | warm {:6.1}m (vs F1b_u {:+6.1}m, vs F4a_u {:+6.1}m) | cold {:6.1}m (vs F1b_u {:+6.1}m, vs F4a_u {:+6.1}m)",
                r.arm.name(),
                w_i9,
                w_u1 - w_i9,
                w_u4 - w_i9,
                c_i9,
                c_u1 - c_i9,
                c_u4 - c_i9,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Step 7: Guard corpora (`runs`, `text`, `onebyte`, `random`, `pattern`, `records`)
    // -----------------------------------------------------------------------
    if !ab_corpora.is_empty() {
        println!("\n=== Step 4: Guard corpora (5 interleaved repeats) ===");
        let guard_arms = &[
            ArmKind::Ctrl1LaneUnfused,
            ArmKind::Ctrl4LaneUnfused,
            ArmKind::Fused1LaneFsmLit32k,
            ArmKind::Fused1LaneTile16k,
            ArmKind::Fused1LaneTile32k,
            ArmKind::Fused4LaneTile2048Stitch,
            ArmKind::Fused4LaneTile2048Vectored,
            ArmKind::Fused4LaneTile1024DefCrcVec,
            ArmKind::Fused4LaneFsm32kStitch,
            ArmKind::Fused4LaneFsm32kVectored,
        ];
        for (name, blks, reps) in &ab_corpora {
            let mut st = FusionStats::default();
            let mut sc_g = Scratch::new();
            for r in reps {
                let _ = fused_4lane_tile::<2048, 65536, true>(
                    &r.fwd.rep,
                    &mut sc_g.out,
                    &mut sc_g.lanes,
                    &mut sc_g.tile,
                    &crc_combine,
                    k,
                    Some(&mut st),
                )
                .unwrap();
            }
            let res = bench_arms(
                reps,
                guard_arms,
                5,
                force_wall,
                &mut flush_buf,
                &crc_combine,
                k,
            );
            let title = format!(
                "Guard corpus: {name}.bz2 ({} blks, straddles={}, overflow={}, no_lanes={})",
                blks.len(),
                st.boundary_straddles,
                st.overflow_fallbacks,
                st.no_lanes_fallbacks
            );
            print_bench_table(&title, &res, None);
        }
    }
}
