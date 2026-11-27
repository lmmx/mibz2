//! Cold dissection of `src/open/lanes.rs` (4-lane forward emission, spelling, stitch, RLE1/CRC32)
//! and `src/open.rs` setup/classify/clear outside A6a/A6b/A6c/A7/A9.
//!
//! Smooth-zone probe — zero edits to `src/`.
//! Usage:
//!   open_lanes_b5_dissection [CORPORA_DIR] [--repeats N]

#![allow(
    dead_code,
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::manual_memcpy
)]

use mibz2::bwt::Segmented;
use mibz2::mtf;
use mibz2::runs;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, rle, Kernels, Scratch};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs, endpoint.rs, outq.rs)
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

const ANCHOR_TAG: u32 = 1u32 << 31;
const MAX_ANCHORS: usize = 32;
const SLAB_SIZE: usize = 32 * 1024;
const COLD_FLUSH_BYTES: usize = 64 * 1024 * 1024;

// Stage-1 i9-10900K reference anchors (wikidata x4, 1 worker)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_A6C_MS: f64 = 161.4;
const I9_A7_MS: f64 = 490.3;
const I9_A6C_A7_MS: f64 = I9_A6C_MS + I9_A7_MS; // 651.7 ms

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

fn within_cap(records: &[u16], p: u32, limit: u32, cap: Cap) -> bool {
    let (mut pos, mut m, mut ns) = (0u32, 0u32, 0usize);
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = rec >> 8;
        let mut width = u32::from(rec & 0xFF);
        i += 1;
        while i < records.len() && (records[i] >> 8) == c {
            width += u32::from(records[i] & 0xFF);
            i += 1;
        }
        if width <= limit {
            m += width;
        } else if p.wrapping_sub(pos) >= width {
            ns += 1;
        } else {
            m += 1;
            ns += usize::from(pos < p) + usize::from(p + 1 < pos + width);
        }
        if m > cap.m || ns > cap.strips {
            return false;
        }
        pos += width;
    }
    true
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

#[derive(Default, Clone)]
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
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
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

#[derive(Default)]
struct OpenBuilder {
    stack: Vec<u32>,
    spill: Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
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

    /// Run A6a + A6b and capture the exact post-A6b state handed into A6c.
    fn prepare_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        scratch_out: &mut Vec<u8>,
    ) -> (PreparedBlock, u64, u64) {
        let t0 = Instant::now();
        scratch_out.resize(n + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut scratch_out[..], &mut self.spill, u32::MAX),
            edges: &mut self.edges,
            ns: 0,
            cap_strips: usize::MAX,
            f: First::default(),
        };
        maximal(records, |c, pos, width| {
            pass.take(c, pos, width, p, limit);
        });
        let ns = pass.ns;
        let chunks_taken = pass.queues.chunks.min(pass.queues.in_out) as usize;
        let (f, heads) = (pass.f, pass.queues.heads());
        let t1 = Instant::now();

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
            self.induced_with::<Wide>(ns, n as u32, f.m, &f.thin)
        } else {
            self.induced_with::<Narrow>(ns, n as u32, f.m, &f.thin)
        };
        let t2 = Instant::now();

        let m = f.m as usize;
        let bits = (u32::BITS - (m.max(1) as u32 - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        let can_tag =
            !wide && m >= 256 && (self.lens.len() - 1) <= ((u32::MAX >> 1) >> bits) as usize;

        let outq_bytes = scratch_out[..chunks_taken * CHUNK_BYTES].to_vec();
        let prep = PreparedBlock {
            n,
            m,
            mark: f.mark,
            bits,
            wide,
            can_tag,
            outq_bytes,
            spill: self.spill.clone(),
            heads,
            bands: self.bands.clone(),
            cursors,
            inline16: self.inline16.clone(),
            lens: self.lens.clone(),
            spelling: self.spelling.clone(),
            edges_fwd: Vec::new(),
            extra_fwd: Vec::new(),
            expected_bwt: Vec::new(),
            expected_out: Vec::new(),
            expected_crc: 0,
            records: Vec::new(),
            orig_ptr: p,
        };

        (
            prep,
            t1.duration_since(t0).as_nanos() as u64,
            t2.duration_since(t1).as_nanos() as u64,
        )
    }
}

struct PreparedBlock {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    wide: bool,
    can_tag: bool,
    outq_bytes: Vec<u8>,
    spill: Spill,
    heads: Heads,
    bands: Vec<Band>,
    cursors: [usize; 256],
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    edges_fwd: Vec<u32>,
    extra_fwd: Vec<u32>,
    expected_bwt: Vec<u8>,
    expected_out: Vec<u8>,
    expected_crc: u32,
    records: Vec<u16>,
    orig_ptr: u32,
}

#[inline(never)]
fn wire_forward(prep: &PreparedBlock, outq_buf: &[u8], edges: &mut [u32], extra: &mut [u32]) {
    let mut k = 0u32;
    for a in 0..256 {
        let end = prep.cursors.get(a + 1).copied().unwrap_or(prep.bands.len());
        let mut queue = Reader::new(outq_buf, &prep.spill, &prep.heads, a);
        for &b in &prep.bands[prep.cursors[a]..end] {
            let bend = b.end();
            let span = (bend - k) as usize;
            let dst = b.dst() as usize;
            let bcode = b.code();
            let pack = if prep.wide { 0 } else { bcode << prep.bits };
            if prep.wide {
                extra[dst..dst + span].fill(bcode);
            }
            queue.wire(&mut edges[dst..dst + span], pack);
            k = bend;
        }
    }
}

// ===========================================================================
// Steal-free Linux thread CPU timing (/proc/thread-self/schedstat) + L3 flush
// ===========================================================================

fn thread_cpu_ns() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/thread-self/schedstat") {
        if let Some(first) = s.split_whitespace().next() {
            if let Ok(ns) = first.parse::<u64>() {
                return ns;
            }
        }
    }
    0
}

fn flush_l3(buf: &mut [u8]) {
    let mut acc = 0u64;
    let mut i = 0usize;
    while i < buf.len() {
        buf[i] = buf[i].wrapping_add(1);
        acc = acc.wrapping_add(buf[i] as u64);
        i += 64;
    }
    black_box(acc);
}

fn median_f64(vals: &[f64]) -> (f64, f64, f64) {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    let med = if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    };
    (med, v[0], v[n - 1])
}

// ===========================================================================
// Exact mirror of production A9 (unrle_in_place + CRC32 from src/rle.rs)
// ===========================================================================

const RLE_TILE: usize = 64 * 1024;
type RleRun = (u32, u8, u8);

fn find_runs_scalar_mirror(rle: &[u8], runs: &mut Vec<RleRun>, mut i: usize) -> Result<(), ()> {
    let n = rle.len();
    while i + 3 < n {
        let c = rle[i];
        if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else {
                return Err(());
            };
            runs.push((i as u32, c, extra));
            i += 5;
        } else {
            i += 1;
        }
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn find_runs_avx2_mirror(rle: &[u8], runs: &mut Vec<RleRun>) -> Result<(), ()> {
    use std::arch::x86_64::*;
    let n = rle.len();
    let p = rle.as_ptr();
    let mut i = 0usize;
    while i + 35 <= n {
        // SAFETY: `i + 35 <= n` keeps all four 32-byte loads inside `rle`.
        let (v0, v1, v2, v3) = unsafe {
            (
                _mm256_loadu_si256(p.add(i) as *const __m256i),
                _mm256_loadu_si256(p.add(i + 1) as *const __m256i),
                _mm256_loadu_si256(p.add(i + 2) as *const __m256i),
                _mm256_loadu_si256(p.add(i + 3) as *const __m256i),
            )
        };
        let eq = _mm256_and_si256(
            _mm256_cmpeq_epi8(v0, v1),
            _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
        );
        let m = _mm256_movemask_epi8(eq) as u32;
        if m == 0 {
            i += 32;
            continue;
        }
        let j = i + m.trailing_zeros() as usize;
        let Some(&extra) = rle.get(j + 4) else {
            return Err(());
        };
        runs.push((j as u32, rle[j], extra));
        i = j + 5;
    }
    find_runs_scalar_mirror(rle, runs, i)
}

#[inline(never)]
fn unrle_in_place_mirror(
    buf: &mut Vec<u8>,
    runs: &mut Vec<RleRun>,
    stage: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ()> {
    let n = buf.len();
    runs.clear();
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2() {
            // SAFETY: `k.avx2` guarantees AVX2+BMI1.
            unsafe { find_runs_avx2_mirror(buf, runs)? };
        } else {
            find_runs_scalar_mirror(buf, runs, 0)?;
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    find_runs_scalar_mirror(buf, runs, 0)?;

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
        rle::unrle(stage, buf, k).map_err(|_| ())?;
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
        while end - *at >= RLE_TILE {
            *s = crc::update_with(*s, &buf[*at..*at + RLE_TILE], k);
            *at += RLE_TILE;
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

// ===========================================================================
// Track A: Production src/open/lanes.rs mirror + progressive redesign arms
// ===========================================================================

const LANES_ANCHORS: usize = 16;
const LANES_TAG: u32 = 1 << 31;
const LANES_SLAB: usize = 32 * 1024;
const LANES_K: usize = 4;

fn repeat_cycle(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[inline(never)]
fn emit_fallback(
    edges: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = (edge >> bits) as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat_cycle(n, out);
}

// ---- L0: 1:1 faithful port of production src/open/lanes.rs ----

#[derive(Default)]
struct L0Arena {
    bytes: Vec<u8>,
    slices: Vec<(u8, u32, u32)>,
}

impl L0Arena {
    fn scratch_bytes(&self) -> usize {
        self.bytes.capacity() + self.slices.capacity() * std::mem::size_of::<(u8, u32, u32)>()
    }
}

#[derive(Clone, Copy, Default)]
struct L0Lane {
    j: u32,
    seg: usize,
    start: usize,
    at: usize,
    end: usize,
}

struct L0Walk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    bytes: &'a mut [u8],
    slices: &'a mut Vec<(u8, u32, u32)>,
    top: usize,
    steps: usize,
    m: usize,
    anchors: [u32; LANES_ANCHORS],
    ends: [u32; LANES_ANCHORS],
    next: usize,
    bad: bool,
}

impl L0Walk<'_> {
    #[inline(always)]
    fn word(&mut self, lane: &mut L0Lane, edge: u32) {
        let code = (edge >> self.bits) as usize;
        lane.j = edge & self.mask;
        let len = self.lens[code];
        if len != LONG && lane.at + INLINE <= lane.end {
            self.bytes[lane.at..lane.at + INLINE].copy_from_slice(&self.inline[code]);
            lane.at += usize::from(len);
        } else {
            self.word_slow(lane, code, len);
        }
    }

    #[cold]
    #[inline(never)]
    fn word_slow(&mut self, lane: &mut L0Lane, code: usize, len: u8) {
        let inline = self.inline;
        let spelling = self.spelling;
        let mut word = if len == LONG {
            long_word(&inline[code], spelling)
        } else {
            &inline[code][..usize::from(len)]
        };
        loop {
            let take = word.len().min(lane.end - lane.at);
            self.bytes[lane.at..lane.at + take].copy_from_slice(&word[..take]);
            lane.at += take;
            word = &word[take..];
            if word.is_empty() {
                return;
            }
            self.close(lane);
            if !self.slab(lane) {
                return;
            }
        }
    }

    fn close(&mut self, lane: &L0Lane) {
        if lane.at > lane.start {
            self.slices
                .push((lane.seg as u8, lane.start as u32, lane.at as u32));
        }
    }

    fn slab(&mut self, lane: &mut L0Lane) -> bool {
        if self.top + LANES_SLAB > self.bytes.len() {
            self.bad = true;
            lane.at = lane.end;
            return false;
        }
        (lane.start, lane.at, lane.end) = (self.top, self.top, self.top + LANES_SLAB);
        self.top += LANES_SLAB;
        true
    }

    fn start(&mut self, lane: &mut L0Lane) -> bool {
        loop {
            if self.next == LANES_ANCHORS || self.bad || self.steps >= self.m {
                self.bad |= self.next < LANES_ANCHORS;
                return false;
            }
            lane.seg = self.next;
            lane.start = lane.at;
            self.next += 1;
            let edge = self.edges[self.anchors[lane.seg] as usize] & !LANES_TAG;
            self.word(lane, edge);
            self.steps += 1;
            if !self.hit(lane) {
                return !self.bad;
            }
        }
    }

    #[inline(always)]
    fn hit(&mut self, lane: &L0Lane) -> bool {
        if self.edges[lane.j as usize] & LANES_TAG == 0 {
            return false;
        }
        self.ends[lane.seg] = lane.j;
        self.close(lane);
        true
    }

    /// Exact production `src/open/lanes.rs` `run<K>` (`#[inline(always)]` into `walk`).
    #[inline(always)]
    fn run_inlined<const K: usize>(&mut self, lanes: &mut [L0Lane; LANES_K]) -> bool {
        let (edges, inline, lens) = (self.edges, self.inline, self.lens);
        let (bits, mask) = (self.bits, self.mask);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut at: [usize; K] = std::array::from_fn(|i| lanes[i].at);
        let mut end: [usize; K] = std::array::from_fn(|i| lanes[i].end);
        let iters = (self.m - self.steps) / K;
        let mut done = 0;
        let mut hit = false;
        let mut bytes = std::mem::take(&mut self.bytes);
        'walk: while done < iters {
            let e: [u32; K] = std::array::from_fn(|i| edges[j[i] as usize]);
            if (e.iter().fold(0, |a, &b| a | b) as i32) < 0 {
                hit = true;
                break;
            }
            done += 1;
            for i in 0..K {
                let code = (e[i] >> bits) as usize;
                j[i] = e[i] & mask;
                let len = lens[code];
                if len != LONG && at[i] + INLINE <= end[i] {
                    bytes[at[i]..at[i] + INLINE].copy_from_slice(&inline[code]);
                    at[i] += usize::from(len);
                } else {
                    self.bytes = bytes;
                    let mut lane = lanes[i];
                    (lane.j, lane.at, lane.end) = (j[i], at[i], end[i]);
                    self.word_slow(&mut lane, code, len);
                    lanes[i] = lane;
                    (at[i], end[i]) = (lane.at, lane.end);
                    bytes = std::mem::take(&mut self.bytes);
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.bytes = bytes;
        self.steps += K * done;
        for i in 0..K {
            (lanes[i].j, lanes[i].at, lanes[i].end) = (j[i], at[i], end[i]);
        }
        if self.bad {
            return false;
        }
        hit || (0..K).any(|i| edges[j[i] as usize] & LANES_TAG != 0)
    }

    /// L1: identical code to `run_inlined` but `#[inline(never)]` to prevent mega-fn spills.
    #[inline(never)]
    fn run_noinline<const K: usize>(&mut self, lanes: &mut [L0Lane; LANES_K]) -> bool {
        self.run_inlined::<K>(lanes)
    }

    /// L2: `#[inline(never)]` + pre-sliced `lens`/`inline` + inline `LONG`-fits-in-slab fast-path.
    #[inline(never)]
    fn run_inline_long<const K: usize>(&mut self, lanes: &mut [L0Lane; LANES_K]) -> bool {
        let edges = self.edges;
        let nw = self.lens.len().min(self.inline.len());
        let lens = &self.lens[..nw];
        let inline = &self.inline[..nw];
        let spelling = self.spelling;
        let (bits, mask) = (self.bits, self.mask);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut at: [usize; K] = std::array::from_fn(|i| lanes[i].at);
        let mut end: [usize; K] = std::array::from_fn(|i| lanes[i].end);
        let iters = (self.m - self.steps) / K;
        let mut done = 0;
        let mut hit = false;
        let mut bytes = std::mem::take(&mut self.bytes);
        'walk: while done < iters {
            let e: [u32; K] = std::array::from_fn(|i| edges[j[i] as usize]);
            if (e.iter().fold(0, |a, &b| a | b) as i32) < 0 {
                hit = true;
                break;
            }
            done += 1;
            for i in 0..K {
                let code = (e[i] >> bits) as usize;
                j[i] = e[i] & mask;
                let len = lens[code];
                if len != LONG && at[i] + INLINE <= end[i] {
                    bytes[at[i]..at[i] + INLINE].copy_from_slice(&inline[code]);
                    at[i] += usize::from(len);
                } else if len == LONG {
                    let word = long_word(&inline[code], spelling);
                    if word.len() <= end[i] - at[i] {
                        bytes[at[i]..at[i] + word.len()].copy_from_slice(word);
                        at[i] += word.len();
                    } else {
                        self.bytes = bytes;
                        let mut lane = lanes[i];
                        (lane.j, lane.at, lane.end) = (j[i], at[i], end[i]);
                        self.word_slow(&mut lane, code, len);
                        lanes[i] = lane;
                        (at[i], end[i]) = (lane.at, lane.end);
                        bytes = std::mem::take(&mut self.bytes);
                        if self.bad {
                            break 'walk;
                        }
                    }
                } else {
                    self.bytes = bytes;
                    let mut lane = lanes[i];
                    (lane.j, lane.at, lane.end) = (j[i], at[i], end[i]);
                    self.word_slow(&mut lane, code, len);
                    lanes[i] = lane;
                    (at[i], end[i]) = (lane.at, lane.end);
                    bytes = std::mem::take(&mut self.bytes);
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.bytes = bytes;
        self.steps += K * done;
        for i in 0..K {
            (lanes[i].j, lanes[i].at, lanes[i].end) = (j[i], at[i], end[i]);
        }
        if self.bad {
            return false;
        }
        hit || (0..K).any(|i| edges[j[i] as usize] & LANES_TAG != 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum L0Mode {
    ShippedInlined,
    NoinlineRun,
    InlineLong,
}

#[inline(never)]
fn walk_l0_family(
    mode: L0Mode,
    edges: &mut [u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    arena: &mut L0Arena,
    out: &mut Vec<u8>,
) {
    let m = edges.len();
    let anchors: [u32; LANES_ANCHORS] = std::array::from_fn(|k| {
        let a = mark as u64 + (k as u64 * m as u64) / LANES_ANCHORS as u64;
        (a % m as u64) as u32
    });
    for &a in &anchors {
        edges[a as usize] |= LANES_TAG;
    }
    let slabs = n.div_ceil(LANES_SLAB) + LANES_K + 1;
    if arena.bytes.len() < slabs * LANES_SLAB {
        arena.bytes.resize(slabs * LANES_SLAB, 0);
    }
    arena.slices.clear();
    let mut w = L0Walk {
        edges,
        spelling,
        inline,
        lens,
        bits,
        mask: (1 << bits) - 1,
        bytes: &mut arena.bytes[..slabs * LANES_SLAB],
        slices: &mut arena.slices,
        top: 0,
        steps: 0,
        m,
        anchors,
        ends: [u32::MAX; LANES_ANCHORS],
        next: 0,
        bad: false,
    };
    let mut lanes = [L0Lane::default(); LANES_K];
    let mut live = 0usize;
    for _ in 0..LANES_K {
        let mut lane = L0Lane::default();
        if w.slab(&mut lane) && w.start(&mut lane) {
            lanes[live] = lane;
            live += 1;
        }
    }
    while live > 0 && !w.bad {
        let ok = match mode {
            L0Mode::ShippedInlined => match live {
                4 => w.run_inlined::<4>(&mut lanes),
                3 => w.run_inlined::<3>(&mut lanes),
                2 => w.run_inlined::<2>(&mut lanes),
                _ => w.run_inlined::<1>(&mut lanes),
            },
            L0Mode::NoinlineRun => match live {
                4 => w.run_noinline::<4>(&mut lanes),
                3 => w.run_noinline::<3>(&mut lanes),
                2 => w.run_noinline::<2>(&mut lanes),
                _ => w.run_noinline::<1>(&mut lanes),
            },
            L0Mode::InlineLong => match live {
                4 => w.run_inline_long::<4>(&mut lanes),
                3 => w.run_inline_long::<3>(&mut lanes),
                2 => w.run_inline_long::<2>(&mut lanes),
                _ => w.run_inline_long::<1>(&mut lanes),
            },
        };
        if !ok {
            w.bad = true;
            break;
        }
        let mut i = 0;
        while i < live {
            let mut lane = lanes[i];
            if w.hit(&lane) && !w.start(&mut lane) {
                live -= 1;
                lanes[i] = lanes[live];
                continue;
            }
            lanes[i] = lane;
            i += 1;
        }
    }
    let (bad, ends) = (w.bad, w.ends);
    for &a in &anchors {
        edges[a as usize] &= !LANES_TAG;
    }
    if bad {
        emit_fallback(edges, spelling, inline, lens, n, mark, bits, out);
        return;
    }
    out.clear();
    let mut seg = 0usize;
    for _ in 0..LANES_ANCHORS {
        for &(s, a, b) in arena.slices.iter() {
            if usize::from(s) == seg {
                let take = ((b - a) as usize).min(n - out.len());
                out.extend_from_slice(&arena.bytes[a as usize..a as usize + take]);
            }
        }
        if out.len() >= n || ends[seg] == mark {
            break;
        }
        match anchors.iter().position(|&a| a == ends[seg]) {
            Some(next) => seg = next,
            None => break,
        }
    }
    repeat_cycle(n, out);
}

// ---- L3/L4/L5/L6: Per-lane disjoint Box<[u8; SLAB]> slabs + lazy pool ----

#[derive(Default)]
struct DisjointArena {
    /// Reusable fixed-size 32 KiB slabs (lazily allocated up to actual block demand, <= 34).
    pool: Vec<Box<[u8; LANES_SLAB]>>,
    /// Slabs filled during the current block walk (indexed by `slab_id: u16`).
    used: Vec<Box<[u8; LANES_SLAB]>>,
    /// `(segment, slab_id, start_off, end_off)` of each closed slice.
    slices: Vec<(u8, u16, u16, u16)>,
}

impl DisjointArena {
    fn reset(&mut self) {
        self.pool.append(&mut self.used);
        self.slices.clear();
    }

    fn alloc_slab(&mut self, max_slabs: usize) -> Option<Box<[u8; LANES_SLAB]>> {
        if self.used.len() >= max_slabs {
            return None;
        }
        match self.pool.pop() {
            Some(b) => Some(b),
            None => Some(vec![0u8; LANES_SLAB].into_boxed_slice().try_into().unwrap()),
        }
    }

    fn scratch_bytes(&self) -> usize {
        (self.pool.len() + self.used.len()) * LANES_SLAB
            + (self.pool.capacity() + self.used.capacity()) * std::mem::size_of::<usize>()
            + self.slices.capacity() * std::mem::size_of::<(u8, u16, u16, u16)>()
    }
}

struct DisjointLane {
    j: u32,
    seg: u8,
    start: usize,
    off: usize,
    slab: Box<[u8; LANES_SLAB]>,
}

impl DisjointArena {
    fn commit_slice(&mut self, seg: u8, slab_id: u16, start: usize, end: usize) {
        if end > start {
            self.slices.push((seg, slab_id, start as u16, end as u16));
        }
    }
}

struct DisjointWalk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    arena: &'a mut DisjointArena,
    max_slabs: usize,
    steps: usize,
    m: usize,
    anchors: [u32; LANES_ANCHORS],
    ends: [u32; LANES_ANCHORS],
    next: usize,
    bad: bool,
}

impl DisjointWalk<'_> {
    #[cold]
    #[inline(never)]
    fn spill_word(&mut self, lane: &mut DisjointLane, code: usize, len: u8) {
        let mut word = if len == LONG {
            long_word(&self.inline[code], self.spelling)
        } else {
            &self.inline[code][..usize::from(len)]
        };
        loop {
            let take = word.len().min(LANES_SLAB - lane.off);
            lane.slab[lane.off..lane.off + take].copy_from_slice(&word[..take]);
            lane.off += take;
            word = &word[take..];
            if word.is_empty() {
                return;
            }
            let Some(fresh) = self.arena.alloc_slab(self.max_slabs) else {
                self.bad = true;
                lane.off = LANES_SLAB;
                return;
            };
            let old = std::mem::replace(&mut lane.slab, fresh);
            let slab_id = self.arena.used.len() as u16;
            self.arena.used.push(old);
            self.arena
                .commit_slice(lane.seg, slab_id, lane.start, lane.off);
            lane.start = 0;
            lane.off = 0;
        }
    }

    fn close_in_place(&mut self, lane: &mut DisjointLane) {
        if lane.off > lane.start {
            // Keep writing in the same slab: only swap slab when full or at walk end!
            // Record pending slice on slab_id = u16::MAX? Even simpler: push slice with a
            // slab_id assigned lazily when the slab is finalized!
            // Wait: if we store active slabs in `used` at walk end, every slice on the same
            // slab shares that slab_id! Let's assign each lane a permanent slab_id when it
            // takes a slab? No, `Box<[u8; SLAB]>` is owned by `lane` during the walk, so we
            // only need `lane.pending: SmallVec` OR swap slab on segment close?
            // Wait: across 233 blocks, there are ONLY 16 segments per block!
            // Let's keep a tiny per-lane `seg_slices` (max 16 entries) flushed when slab swaps!
        }
    }
}

/// Clean disjoint-slab lane: keeps its owned 32 KiB slab + tiny list of `(seg, start, end)`
/// slices on that slab, flushed to `arena.used` + `arena.slices` ONLY when the slab fills
/// (~1.8 times per lane per block) or when the lane retires!
struct SlabLane {
    j: u32,
    seg: u8,
    start: usize,
    off: usize,
    slab: Box<[u8; LANES_SLAB]>,
    pending: Vec<(u8, u16, u16)>,
}

impl SlabLane {
    fn flush_slab(&mut self, arena: &mut DisjointArena, replacement: Box<[u8; LANES_SLAB]>) {
        if self.off > self.start {
            self.pending
                .push((self.seg, self.start as u16, self.off as u16));
        }
        let old = std::mem::replace(&mut self.slab, replacement);
        let slab_id = arena.used.len() as u16;
        arena.used.push(old);
        for (seg, a, b) in self.pending.drain(..) {
            arena.slices.push((seg, slab_id, a, b));
        }
        self.start = 0;
        self.off = 0;
    }

    fn retire(mut self, arena: &mut DisjointArena) {
        if self.off > self.start {
            self.pending
                .push((self.seg, self.start as u16, self.off as u16));
        }
        let slab_id = arena.used.len() as u16;
        arena.used.push(self.slab);
        for (seg, a, b) in self.pending.drain(..) {
            arena.slices.push((seg, slab_id, a, b));
        }
    }
}

struct SlabWalk<'a> {
    edges: &'a [u32],
    spelling: &'a [u8],
    inline: &'a [[u8; INLINE]],
    lens: &'a [u8],
    bits: u32,
    mask: u32,
    arena: &'a mut DisjointArena,
    max_slabs: usize,
    steps: usize,
    m: usize,
    anchors: [u32; LANES_ANCHORS],
    ends: [u32; LANES_ANCHORS],
    next: usize,
    bad: bool,
}

impl SlabWalk<'_> {
    #[cold]
    #[inline(never)]
    fn spill_word(&mut self, lane: &mut SlabLane, code: usize, len: u8) {
        let mut word = if len == LONG {
            long_word(&self.inline[code], self.spelling)
        } else {
            &self.inline[code][..usize::from(len)]
        };
        loop {
            let take = word.len().min(LANES_SLAB - lane.off);
            lane.slab[lane.off..lane.off + take].copy_from_slice(&word[..take]);
            lane.off += take;
            word = &word[take..];
            if word.is_empty() {
                return;
            }
            let Some(fresh) = self.arena.alloc_slab(self.max_slabs) else {
                self.bad = true;
                lane.off = LANES_SLAB;
                return;
            };
            lane.flush_slab(self.arena, fresh);
        }
    }

    #[inline(always)]
    fn word(&mut self, lane: &mut SlabLane, edge: u32) {
        let code = (edge >> self.bits) as usize;
        lane.j = edge & self.mask;
        let len = self.lens[code];
        if len != LONG && lane.off <= LANES_SLAB - INLINE {
            lane.slab[lane.off..lane.off + INLINE].copy_from_slice(&self.inline[code]);
            lane.off += usize::from(len);
        } else {
            self.spill_word(lane, code, len);
        }
    }

    #[inline(always)]
    fn hit(&mut self, lane: &mut SlabLane) -> bool {
        if self.edges[lane.j as usize] & LANES_TAG == 0 {
            return false;
        }
        self.ends[lane.seg as usize] = lane.j;
        if lane.off > lane.start {
            lane.pending
                .push((lane.seg, lane.start as u16, lane.off as u16));
        }
        lane.start = lane.off;
        true
    }

    fn start(&mut self, lane: &mut SlabLane) -> bool {
        loop {
            if self.next == LANES_ANCHORS || self.bad || self.steps >= self.m {
                self.bad |= self.next < LANES_ANCHORS;
                return false;
            }
            lane.seg = self.next as u8;
            lane.start = lane.off;
            self.next += 1;
            let edge = self.edges[self.anchors[lane.seg as usize] as usize] & !LANES_TAG;
            self.word(lane, edge);
            self.steps += 1;
            if !self.hit(lane) {
                return !self.bad;
            }
        }
    }

    /// L3 (100% safe Rust): `#[inline(never)]`, per-lane disjoint `[u8; 32768]` slabs
    /// (`off <= SLAB - 16` eliminates ALL slab bounds & overflow checks + `end[0..4]`!),
    /// `lens.len() == inline.len()` pre-slice, and inline `LONG`-fits-in-slab fast-path!
    #[inline(never)]
    fn run_safe<const K: usize>(&mut self, lanes: &mut [SlabLane]) -> bool {
        let edges = self.edges;
        let nw = self.lens.len().min(self.inline.len());
        let lens = &self.lens[..nw];
        let inline = &self.inline[..nw];
        let spelling = self.spelling;
        let (bits, mask) = (self.bits, self.mask);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut off: [usize; K] = std::array::from_fn(|i| lanes[i].off);
        let iters = (self.m - self.steps) / K;
        let mut done = 0usize;
        let mut hit = false;
        'walk: while done < iters {
            let e: [u32; K] = std::array::from_fn(|i| edges[j[i] as usize]);
            if (e.iter().fold(0, |a, &b| a | b) as i32) < 0 {
                hit = true;
                break;
            }
            done += 1;
            for i in 0..K {
                let code = (e[i] >> bits) as usize;
                j[i] = e[i] & mask;
                let len = lens[code];
                let o = off[i];
                if len != LONG && o <= LANES_SLAB - INLINE {
                    lanes[i].slab[o..o + INLINE].copy_from_slice(&inline[code]);
                    off[i] = o + usize::from(len);
                } else if len == LONG {
                    let word = long_word(&inline[code], spelling);
                    if word.len() <= LANES_SLAB - o {
                        lanes[i].slab[o..o + word.len()].copy_from_slice(word);
                        off[i] = o + word.len();
                    } else {
                        lanes[i].j = j[i];
                        lanes[i].off = o;
                        self.spill_word(&mut lanes[i], code, len);
                        off[i] = lanes[i].off;
                        if self.bad {
                            break 'walk;
                        }
                    }
                } else {
                    lanes[i].j = j[i];
                    lanes[i].off = o;
                    self.spill_word(&mut lanes[i], code, len);
                    off[i] = lanes[i].off;
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.steps += K * done;
        for i in 0..K {
            lanes[i].j = j[i];
            lanes[i].off = off[i];
        }
        if self.bad {
            return false;
        }
        hit || (0..K).any(|i| edges[j[i] as usize] & LANES_TAG != 0)
    }

    /// L4 (bounded `unsafe` ceiling check): L3 + `get_unchecked` on `edges`/`lens`/`inline`
    /// and direct `copy_nonoverlapping` on 16B inline + inline-fitting `LONG`.
    #[inline(never)]
    fn run_nobc<const K: usize>(&mut self, lanes: &mut [SlabLane]) -> bool {
        let edges = self.edges;
        let nw = self.lens.len().min(self.inline.len());
        let lens = &self.lens[..nw];
        let inline = &self.inline[..nw];
        let spelling = self.spelling;
        let (bits, mask) = (self.bits, self.mask);
        // Validate every edge target `< edges.len()` and code `< nw`?
        // In production Open construction, `mask` can exceed `m - 1`, so we check `j < m` via
        // construction invariant (already verified in `construct` & `b5` tests: every wired edge
        // has `edge & mask < m` and `(edge >> bits) < lens.len()`).
        assert!(lanes.len() >= K);
        let mut j: [u32; K] = std::array::from_fn(|i| lanes[i].j);
        let mut off: [usize; K] = std::array::from_fn(|i| lanes[i].off);
        let iters = (self.m - self.steps) / K;
        let mut done = 0usize;
        let mut hit = false;
        'walk: while done < iters {
            // SAFETY: every valid Open edge points to `j < edges.len()`.
            let e: [u32; K] =
                std::array::from_fn(|i| unsafe { *edges.get_unchecked(j[i] as usize) });
            if (e.iter().fold(0, |a, &b| a | b) as i32) < 0 {
                hit = true;
                break;
            }
            done += 1;
            for i in 0..K {
                let code = (e[i] >> bits) as usize;
                j[i] = e[i] & mask;
                // SAFETY: every constructed code is `< nw == lens.len() == inline.len()`.
                let len = unsafe { *lens.get_unchecked(code) };
                let o = off[i];
                if len != LONG && o <= LANES_SLAB - INLINE {
                    // SAFETY: `o + INLINE <= LANES_SLAB` and `code < inline.len()`.
                    unsafe {
                        let dst = lanes.get_unchecked_mut(i).slab.as_mut_ptr().add(o);
                        let src = inline.get_unchecked(code).as_ptr();
                        std::ptr::copy_nonoverlapping(src, dst, INLINE);
                    }
                    off[i] = o + usize::from(len);
                } else if len == LONG {
                    // SAFETY: `code < inline.len()`; `long_word` bounds-checks `spelling`.
                    let slot = unsafe { inline.get_unchecked(code) };
                    let word = long_word(slot, spelling);
                    let wlen = word.len();
                    if wlen <= LANES_SLAB - o {
                        // SAFETY: `o + wlen <= LANES_SLAB`; `word` and `slab` are disjoint.
                        unsafe {
                            let dst = lanes.get_unchecked_mut(i).slab.as_mut_ptr().add(o);
                            std::ptr::copy_nonoverlapping(word.as_ptr(), dst, wlen);
                        }
                        off[i] = o + wlen;
                    } else {
                        lanes[i].j = j[i];
                        lanes[i].off = o;
                        self.spill_word(&mut lanes[i], code, len);
                        off[i] = lanes[i].off;
                        if self.bad {
                            break 'walk;
                        }
                    }
                } else {
                    lanes[i].j = j[i];
                    lanes[i].off = o;
                    self.spill_word(&mut lanes[i], code, len);
                    off[i] = lanes[i].off;
                    if self.bad {
                        break 'walk;
                    }
                }
            }
        }
        self.steps += K * done;
        for i in 0..K {
            lanes[i].j = j[i];
            lanes[i].off = off[i];
        }
        if self.bad {
            return false;
        }
        hit || (0..K).any(|i| edges[j[i] as usize] & LANES_TAG != 0)
    }
}

/// Ordered slice reference produced by `walk_disjoint_core` (ready for either stitch or fused RLE1+CRC).
fn collect_ordered_slices(
    arena: &DisjointArena,
    anchors: &[u32; LANES_ANCHORS],
    ends: &[u32; LANES_ANCHORS],
    mark: u32,
    n: usize,
    out_slices: &mut Vec<(u16, u16, u16)>,
) -> usize {
    out_slices.clear();
    let mut total = 0usize;
    let mut seg = 0usize;
    for _ in 0..LANES_ANCHORS {
        for &(s, slab_id, a, b) in &arena.slices {
            if usize::from(s) == seg {
                let take = ((b - a) as usize).min(n - total);
                if take > 0 {
                    out_slices.push((slab_id, a, a + take as u16));
                    total += take;
                }
            }
        }
        if total >= n || ends[seg] == mark {
            break;
        }
        match anchors.iter().position(|&a| a == ends[seg]) {
            Some(next) => seg = next,
            None => break,
        }
    }
    total
}

#[inline(never)]
fn walk_disjoint_core(
    nobc: bool,
    edges: &mut [u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    arena: &mut DisjointArena,
    ordered: &mut Vec<(u16, u16, u16)>,
) -> Result<usize, ()> {
    let m = edges.len();
    let anchors: [u32; LANES_ANCHORS] = std::array::from_fn(|k| {
        let a = mark as u64 + (k as u64 * m as u64) / LANES_ANCHORS as u64;
        (a % m as u64) as u32
    });
    for &a in &anchors {
        edges[a as usize] |= LANES_TAG;
    }
    arena.reset();
    let max_slabs = n.div_ceil(LANES_SLAB) + LANES_K + 1;
    let mut w = SlabWalk {
        edges,
        spelling,
        inline,
        lens,
        bits,
        mask: (1 << bits) - 1,
        arena,
        max_slabs,
        steps: 0,
        m,
        anchors,
        ends: [u32::MAX; LANES_ANCHORS],
        next: 0,
        bad: false,
    };
    let mut lanes: Vec<SlabLane> = Vec::with_capacity(LANES_K);
    for _ in 0..LANES_K {
        let Some(slab) = w.arena.alloc_slab(max_slabs) else {
            w.bad = true;
            break;
        };
        let mut lane = SlabLane {
            j: 0,
            seg: 0,
            start: 0,
            off: 0,
            slab,
            pending: Vec::with_capacity(8),
        };
        if w.start(&mut lane) {
            lanes.push(lane);
        } else {
            lane.retire(w.arena);
        }
    }
    while !lanes.is_empty() && !w.bad {
        let ok = if nobc {
            match lanes.len() {
                4 => w.run_nobc::<4>(&mut lanes),
                3 => w.run_nobc::<3>(&mut lanes),
                2 => w.run_nobc::<2>(&mut lanes),
                _ => w.run_nobc::<1>(&mut lanes),
            }
        } else {
            match lanes.len() {
                4 => w.run_safe::<4>(&mut lanes),
                3 => w.run_safe::<3>(&mut lanes),
                2 => w.run_safe::<2>(&mut lanes),
                _ => w.run_safe::<1>(&mut lanes),
            }
        };
        if !ok {
            w.bad = true;
            break;
        }
        let mut i = 0;
        while i < lanes.len() {
            if w.hit(&mut lanes[i]) && !w.start(&mut lanes[i]) {
                let retired = lanes.swap_remove(i);
                retired.retire(w.arena);
                continue;
            }
            i += 1;
        }
    }
    for lane in lanes {
        lane.retire(w.arena);
    }
    let (bad, ends) = (w.bad, w.ends);
    for &a in &anchors {
        edges[a as usize] &= !LANES_TAG;
    }
    if bad {
        return Err(());
    }
    let total = collect_ordered_slices(arena, &anchors, &ends, mark, n, ordered);
    Ok(total)
}

#[inline(never)]
fn walk_disjoint_stitch(
    nobc: bool,
    edges: &mut [u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    arena: &mut DisjointArena,
    ordered: &mut Vec<(u16, u16, u16)>,
    out: &mut Vec<u8>,
) {
    let Ok(total) = walk_disjoint_core(
        nobc, edges, spelling, inline, lens, n, mark, bits, arena, ordered,
    ) else {
        emit_fallback(edges, spelling, inline, lens, n, mark, bits, out);
        return;
    };
    out.clear();
    out.reserve(n);
    for &(slab_id, a, b) in ordered.iter() {
        out.extend_from_slice(&arena.used[slab_id as usize][a as usize..b as usize]);
    }
    debug_assert_eq!(out.len(), total);
    repeat_cycle(n, out);
}

// ---- L5 (100% safe) & L6 (AVX2): Fused Stitch + RLE1 + 64 KiB L2-hot CRC32 ----

/// Decode one forward RLE1 chunk into `out`, returning trailing unfinished bytes (`0..=4`).
#[inline(never)]
fn decode_rle_chunk_safe(src: &[u8], is_eof: bool, out: &mut Vec<u8>) -> Result<usize, ()> {
    let avail = src.len();
    let limit = if is_eof {
        avail
    } else {
        avail.saturating_sub(4)
    };
    let mut i = 0usize;
    while i < limit {
        let c = src[i];
        if i + 3 < avail && src[i + 1] == c && src[i + 2] == c && src[i + 3] == c {
            if i + 4 >= avail {
                return Err(());
            }
            let run = 4 + src[i + 4] as usize;
            out.resize(out.len() + run, c);
            i += 5;
        } else {
            let start = i;
            i += 1;
            while i < limit {
                let b = src[i];
                if i + 3 < avail && src[i + 1] == b && src[i + 2] == b && src[i + 3] == b {
                    break;
                }
                i += 1;
            }
            out.extend_from_slice(&src[start..i]);
        }
    }
    Ok(avail - i)
}

#[inline(never)]
fn fused_slices_rle_crc_safe(
    arena: &DisjointArena,
    ordered: &[(u16, u16, u16)],
    n: usize,
    out: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ()> {
    out.clear();
    out.reserve(n + 64);
    let mut carry = [0u8; 40];
    let mut carry_len = 0usize;
    let mut crc_state = !0u32;
    let mut crc_at = 0usize;

    let flush_crc = |crc_state: &mut u32, crc_at: &mut usize, out: &[u8]| {
        let avail = out.len() - *crc_at;
        if avail >= RLE_TILE {
            let take = avail & !63;
            *crc_state = crc::update_with(*crc_state, &out[*crc_at..*crc_at + take], k);
            *crc_at += take;
        }
    };

    for (idx, &(slab_id, a, b)) in ordered.iter().enumerate() {
        let slice = &arena.used[slab_id as usize][a as usize..b as usize];
        let is_last = idx + 1 == ordered.len();
        let mut s_off = 0usize;
        if carry_len > 0 {
            let need = (carry.len() - carry_len).min(slice.len());
            carry[carry_len..carry_len + need].copy_from_slice(&slice[..need]);
            let tot = carry_len + need;
            let eof_here = is_last && need == slice.len();
            let rem = decode_rle_chunk_safe(&carry[..tot], eof_here, out)?;
            if need < slice.len() && rem <= need {
                s_off = need - rem;
                carry_len = 0;
            } else {
                s_off = need;
                if rem > 0 {
                    carry.copy_within(tot - rem..tot, 0);
                }
                carry_len = rem;
            }
        }
        if s_off < slice.len() {
            let tail = &slice[s_off..];
            let rem = decode_rle_chunk_safe(tail, is_last, out)?;
            if rem > 0 {
                carry[..rem].copy_from_slice(&tail[tail.len() - rem..]);
                carry_len = rem;
            }
        }
        flush_crc(&mut crc_state, &mut crc_at, out);
    }
    if carry_len > 0 {
        let rem = decode_rle_chunk_safe(&carry[..carry_len], true, out)?;
        debug_assert_eq!(rem, 0);
    }
    if crc_at < out.len() {
        crc_state = crc::update_with(crc_state, &out[crc_at..], k);
    }
    Ok(!crc_state)
}

#[cfg(target_arch = "x86_64")]
#[cold]
#[inline(never)]
fn grow_out_fused(
    out: &mut Vec<u8>,
    cur_len: usize,
    extra_need: usize,
    consumed: usize,
    rem_in: usize,
) -> (*mut u8, usize) {
    // SAFETY: `out[..cur_len]` is initialized and `cur_len <= out.capacity()`.
    unsafe { out.set_len(cur_len) };
    let est_rem = if consumed >= 1024 && cur_len > consumed {
        let raw = ((cur_len as u128 * rem_in as u128) / (consumed as u128)) as usize;
        raw.saturating_add(raw / 100).saturating_add(16 << 10)
    } else {
        rem_in.saturating_add(rem_in / 16).saturating_add(32 << 10)
    };
    let max_rem = rem_in.saturating_mul(52).saturating_add(320);
    let step = extra_need.max(est_rem.min(max_rem));
    out.reserve_exact(step);
    (out.as_mut_ptr(), out.capacity())
}

/// Single-`#[target_feature(enable = "avx2,bmi1")]` 2-load AVX2 RLE1 + 64 KiB L2-hot CRC32
/// operating DIRECTLY on already-forward ordered 32 KiB slab slices (ZERO 4 KiB reversal,
/// ZERO stitch copy, ZERO `find_runs_avx2`, ZERO `copy_within`!).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn fused_slices_rle_crc_avx2(
    arena: &DisjointArena,
    ordered: &[(u16, u16, u16)],
    n: usize,
    out: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ()> {
    use core::arch::x86_64::*;

    out.clear();
    out.reserve(n + 64);
    let mut out_len = 0usize;
    let mut out_cap = out.capacity();
    let mut out_p = out.as_mut_ptr();
    let mut crc_state = !0u32;
    let mut crc_at = 0usize;
    let mut consumed_before = 0usize;

    let mut seam = [0u8; 64];
    let mut seam_len = 0usize;

    macro_rules! decode_slice_avx2 {
        ($slice_ptr:expr, $avail:expr, $is_eof:expr, $consumed_base:expr) => {{
            let p: *const u8 = $slice_ptr;
            let avail: usize = $avail;
            let is_eof: bool = $is_eof;
            let mut i = 0usize;
            if out_len + avail + 32 > out_cap {
                let consumed = $consumed_base;
                let rem_in = n.saturating_sub(consumed);
                let (np, nc) = grow_out_fused(out, out_len, avail + 32, consumed, rem_in);
                out_p = np;
                out_cap = nc;
            }
            while i + 36 <= avail {
                let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
                let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
                let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32;
                _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, v0);
                let tri = m1 & (m1 >> 1) & (m1 >> 2);
                if (tri | (m1 & (1 << 31))) == 0 {
                    out_len += 32;
                    i += 32;
                    continue;
                }
                let tz = if tri != 0 {
                    tri.trailing_zeros() as usize
                } else {
                    let b32 = *p.add(i + 32);
                    let b33 = *p.add(i + 33);
                    if (m1 & (1 << 30)) != 0 && b32 == b33 {
                        30
                    } else if b32 == b33 && b33 == *p.add(i + 34) {
                        31
                    } else {
                        out_len += 32;
                        i += 32;
                        continue;
                    }
                };
                let j = i + tz;
                let c = *p.add(j);
                let run = 4 + (*p.add(j + 4) as usize);
                out_len += tz;
                let need = run + (avail - (j + 5)) + 32;
                if out_len + need > out_cap {
                    let consumed = $consumed_base + j;
                    let rem_in = n.saturating_sub(consumed);
                    let (np, nc) = grow_out_fused(out, out_len, need, consumed, rem_in);
                    out_p = np;
                    out_cap = nc;
                }
                if run <= 32 {
                    let fv = _mm256_set1_epi8(c as i8);
                    _mm256_storeu_si256(out_p.add(out_len) as *mut __m256i, fv);
                } else {
                    std::ptr::write_bytes(out_p.add(out_len), c, run);
                }
                out_len += run;
                i = j + 5;
            }
            let limit = if is_eof {
                avail
            } else {
                avail.saturating_sub(4)
            };
            while i < limit {
                let c = *p.add(i);
                if i + 3 < avail && *p.add(i + 1) == c && *p.add(i + 2) == c && *p.add(i + 3) == c {
                    if i + 4 >= avail {
                        out.set_len(0);
                        return Err(());
                    }
                    let run = 4 + (*p.add(i + 4) as usize);
                    let need = run + (avail - (i + 5)) + 32;
                    if out_len + need > out_cap {
                        let consumed = $consumed_base + i;
                        let rem_in = n.saturating_sub(consumed);
                        let (np, nc) = grow_out_fused(out, out_len, need, consumed, rem_in);
                        out_p = np;
                        out_cap = nc;
                    }
                    std::ptr::write_bytes(out_p.add(out_len), c, run);
                    out_len += run;
                    i += 5;
                } else {
                    *out_p.add(out_len) = c;
                    out_len += 1;
                    i += 1;
                }
            }
            avail - i
        }};
    }

    for (idx, &(slab_id, a, b)) in ordered.iter().enumerate() {
        let slice = &arena.used[slab_id as usize][a as usize..b as usize];
        let is_last = idx + 1 == ordered.len();
        let mut s_off = 0usize;
        if seam_len > 0 {
            let take = (seam.len() - seam_len).min(slice.len());
            seam[seam_len..seam_len + take].copy_from_slice(&slice[..take]);
            let tot = seam_len + take;
            let eof_here = is_last && take == slice.len();
            let rem = decode_slice_avx2!(seam.as_ptr(), tot, eof_here, consumed_before);
            consumed_before += tot - rem;
            if take < slice.len() && rem <= take {
                s_off = take - rem;
                seam_len = 0;
            } else {
                s_off = take;
                if rem > 0 {
                    seam.copy_within(tot - rem..tot, 0);
                }
                seam_len = rem;
            }
        }
        if s_off < slice.len() {
            let tail = &slice[s_off..];
            let rem = decode_slice_avx2!(tail.as_ptr(), tail.len(), is_last, consumed_before);
            consumed_before += tail.len() - rem;
            if rem > 0 {
                seam[..rem].copy_from_slice(&tail[tail.len() - rem..]);
                seam_len = rem;
            }
        }
        let avail_out = out_len - crc_at;
        if avail_out >= RLE_TILE {
            let take = avail_out & !63;
            out.set_len(out_len);
            crc_state = crc::update_with(crc_state, &out[crc_at..crc_at + take], k);
            crc_at += take;
        }
    }
    if seam_len > 0 {
        let rem = decode_slice_avx2!(seam.as_ptr(), seam_len, true, consumed_before);
        debug_assert_eq!(rem, 0);
    }
    out.set_len(out_len);
    if crc_at < out_len {
        crc_state = crc::update_with(crc_state, &out[crc_at..out_len], k);
    }
    Ok(!crc_state)
}

#[inline(never)]
fn walk_disjoint_fused_rle_crc(
    nobc: bool,
    use_avx2: bool,
    edges: &mut [u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    arena: &mut DisjointArena,
    ordered: &mut Vec<(u16, u16, u16)>,
    out: &mut Vec<u8>,
    runs_scratch: &mut Vec<RleRun>,
    stage_scratch: &mut Vec<u8>,
    k: Kernels,
) -> Result<u32, ()> {
    let Ok(total) = walk_disjoint_core(
        nobc, edges, spelling, inline, lens, n, mark, bits, arena, ordered,
    ) else {
        emit_fallback(edges, spelling, inline, lens, n, mark, bits, out);
        return unrle_in_place_mirror(out, runs_scratch, stage_scratch, k);
    };
    if total < n {
        // Short cycle (`total < n`, rare periodic block): stitch + repeat + standard A9.
        out.clear();
        out.reserve(n);
        for &(slab_id, a, b) in ordered.iter() {
            out.extend_from_slice(&arena.used[slab_id as usize][a as usize..b as usize]);
        }
        repeat_cycle(n, out);
        return unrle_in_place_mirror(out, runs_scratch, stage_scratch, k);
    }
    #[cfg(target_arch = "x86_64")]
    if use_avx2 && k.avx2() {
        // SAFETY: `k.avx2` guarantees AVX2 + BMI1.
        return unsafe { fused_slices_rle_crc_avx2(arena, ordered, n, out, k) };
    }
    let _ = use_avx2;
    fused_slices_rle_crc_safe(arena, ordered, n, out, k)
}

// ===========================================================================
// Corpus loader & Open block preparation
// ===========================================================================

fn load_open_blocks(path: &str, k: Kernels) -> (usize, Vec<PreparedBlock>) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let mut sc = Scanner::new(SliceSource(&data));
    let mut scratch = Scratch::<Segmented>::default();
    let mut builder = OpenBuilder::default();
    let mut outq_scratch = Vec::new();
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut total_blocks = 0usize;
    let mut preps = Vec::new();

    while let Event::Block { symbols: sym, .. } = sc
        .next(std::mem::take(&mut syms))
        .unwrap_or_else(|e| panic!("scan error in {path}: {e:?}"))
    {
        total_blocks += 1;
        let mut ref_out = Vec::new();
        let ref_crc = mibz2::decode_block(&sym, k, &mut scratch, &mut ref_out).expect("ref decode");
        if runs::takes_runs(&sym) {
            mtf::mtf(&sym, &mut ll, &mut counts, k).expect("mtf");
            let n = ll.len();
            let recs = records_of(&ll);
            let admitted = match open_cap(recs.len(), n, sym.orig_ptr, false) {
                Some(Cap::NONE) => true,
                Some(cap) => within_cap(&recs, sym.orig_ptr, PROD_LIMIT, cap),
                None => false,
            };
            if admitted {
                let (mut prep, _, _) =
                    builder.prepare_block(&recs, n, sym.orig_ptr, PROD_LIMIT, &mut outq_scratch);
                let mut edges_fwd = vec![0u32; prep.m];
                let mut extra_fwd = if prep.wide {
                    vec![0u32; prep.m]
                } else {
                    Vec::new()
                };
                wire_forward(&prep, &prep.outq_bytes, &mut edges_fwd, &mut extra_fwd);
                prep.edges_fwd = edges_fwd;
                prep.extra_fwd = extra_fwd;
                let mut bwt = Vec::new();
                emit_fallback(
                    &prep.edges_fwd,
                    &prep.spelling,
                    &prep.inline16,
                    &prep.lens,
                    prep.n,
                    prep.mark,
                    prep.bits,
                    &mut bwt,
                );
                prep.expected_bwt = bwt;
                prep.expected_out = ref_out;
                prep.expected_crc = ref_crc;
                prep.records = recs;
                preps.push(prep);
            }
        }
        syms = sym.syms;
    }
    (total_blocks, preps)
}

// ===========================================================================
// Census of lanes.rs micro-behavior on 233 Wikidata Open blocks
// ===========================================================================

fn run_census(preps: &[PreparedBlock]) {
    let mut total_m = 0u64;
    let mut total_n = 0u64;
    let mut fits_count = 0usize;
    let mut long_steps = 0u64;
    let mut lit1_steps = 0u64;
    let mut inline2_16_steps = 0u64;
    let mut long_fit_in_slab = 0u64;
    let mut long_cross_slab = 0u64;
    let mut inline_cross_slab = 0u64;
    let mut long_le_32 = 0u64;
    let mut long_33_64 = 0u64;
    let mut long_65_128 = 0u64;
    let mut long_gt_128 = 0u64;
    let mut total_slices_l0 = 0usize;
    let mut total_slabs_l0 = 0usize;
    let mut total_slabs_l3 = 0usize;
    let mut max_slabs_l0 = 0usize;
    let mut max_slabs_l3 = 0usize;
    let mut shift_gt0_blocks = 0usize;
    let mut rle_runs_total = 0usize;

    let k = Kernels::detect();
    let mut runs_tmp = Vec::new();
    let mut arena_l3 = DisjointArena::default();
    let mut ordered = Vec::new();

    for p in preps {
        total_m += p.m as u64;
        total_n += p.n as u64;
        let fits =
            !p.wide && p.m >= 256 && (p.lens.len() - 1) <= ((u32::MAX >> 1) >> p.bits) as usize;
        if fits {
            fits_count += 1;
        }
        // RLE1 stats on pre-RLE1 BWT bytes
        runs_tmp.clear();
        find_runs_scalar_mirror(&p.expected_bwt, &mut runs_tmp, 0).unwrap();
        rle_runs_total += runs_tmp.len();
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in &runs_tmp {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        if shift > 0 {
            shift_gt0_blocks += 1;
        }

        // Exact step & slab-crossing census
        let mut edges = p.edges_fwd.clone();
        let slabs_l0 = p.n.div_ceil(LANES_SLAB) + LANES_K + 1;
        total_slabs_l0 += slabs_l0;
        max_slabs_l0 = max_slabs_l0.max(slabs_l0);
        if fits {
            let _ = walk_disjoint_core(
                false,
                &mut edges,
                &p.spelling,
                &p.inline16,
                &p.lens,
                p.n,
                p.mark,
                p.bits,
                &mut arena_l3,
                &mut ordered,
            );
            total_slabs_l3 += arena_l3.used.len();
            max_slabs_l3 = max_slabs_l3.max(arena_l3.used.len());
            total_slices_l0 += arena_l3.slices.len();

            // Simulate exact slab offsets along the 16 segments
            let mask = (1u32 << p.bits) - 1;
            let mut j = p.mark;
            let mut off = 0usize;
            for _ in 0..p.m {
                let e = p.edges_fwd[j as usize];
                let code = (e >> p.bits) as usize;
                j = e & mask;
                let len = p.lens[code];
                if len == 1 {
                    lit1_steps += 1;
                    if off + INLINE <= LANES_SLAB {
                        off += 1;
                    } else {
                        inline_cross_slab += 1;
                        off = (off + 1) % LANES_SLAB;
                    }
                } else if len != LONG {
                    inline2_16_steps += 1;
                    let wl = len as usize;
                    if off + INLINE <= LANES_SLAB {
                        off += wl;
                    } else {
                        inline_cross_slab += 1;
                        off = (off + wl) % LANES_SLAB;
                    }
                } else {
                    long_steps += 1;
                    let wl = long_word(&p.inline16[code], &p.spelling).len();
                    if wl <= 32 {
                        long_le_32 += 1;
                    } else if wl <= 64 {
                        long_33_64 += 1;
                    } else if wl <= 128 {
                        long_65_128 += 1;
                    } else {
                        long_gt_128 += 1;
                    }
                    if wl <= LANES_SLAB - off {
                        long_fit_in_slab += 1;
                        off += wl;
                    } else {
                        long_cross_slab += 1;
                        off = (off + wl) % LANES_SLAB;
                    }
                }
                if j == p.mark {
                    break;
                }
            }
        }
    }
    let _ = k;
    let total_steps = lit1_steps + inline2_16_steps + long_steps;
    let word_slow_l0 = long_steps + inline_cross_slab;
    let word_slow_l2 = long_cross_slab + inline_cross_slab;
    println!(
        "=== TRACK A CENSUS ({} Wikidata Open blocks, {} fit 4-lane) ===",
        preps.len(),
        fits_count
    );
    println!(
        "  total_n = {} B ({:.2} MiB), total_m = {} steps",
        total_n,
        total_n as f64 / (1024.0 * 1024.0),
        total_m
    );
    println!(
        "  step mix: 1B lit = {} ({:.2}%), 2..16B inline = {} ({:.2}%), LONG = {} ({:.2}%)",
        lit1_steps,
        100.0 * lit1_steps as f64 / total_steps as f64,
        inline2_16_steps,
        100.0 * inline2_16_steps as f64 / total_steps as f64,
        long_steps,
        100.0 * long_steps as f64 / total_steps as f64
    );
    println!("  LONG len hist: 17..32B = {} ({:.1}%), 33..64B = {} ({:.1}%), 65..128B = {} ({:.1}%), >128B = {} ({:.1}%)",
        long_le_32, 100.0 * long_le_32 as f64 / long_steps.max(1) as f64,
        long_33_64, 100.0 * long_33_64 as f64 / long_steps.max(1) as f64,
        long_65_128, 100.0 * long_65_128 as f64 / long_steps.max(1) as f64,
        long_gt_128, 100.0 * long_gt_128 as f64 / long_steps.max(1) as f64);
    println!("  word_slow calls: L0 shipped = {} -> L2 inline_long = {} ({:.1}x reduction! long_fit_in_slab = {} / {:.2}%, long_cross = {}, inline_cross = {})",
        word_slow_l0, word_slow_l2, word_slow_l0 as f64 / word_slow_l2.max(1) as f64,
        long_fit_in_slab, 100.0 * long_fit_in_slab as f64 / long_steps.max(1) as f64,
        long_cross_slab, inline_cross_slab);
    println!("  arena slabs: L0 avg {:.2} / max {} ({} KiB) vs L3 lazy avg {:.2} / max {} ({} KiB); slices/blk = {:.1}",
        total_slabs_l0 as f64 / preps.len() as f64, max_slabs_l0, max_slabs_l0 * 32,
        total_slabs_l3 as f64 / preps.len() as f64, max_slabs_l3, max_slabs_l3 * 32,
        total_slices_l0 as f64 / preps.len() as f64);
    println!(
        "  RLE1 on Open blocks: total_runs = {}, shift>0 blocks = {}/{} ({:.1}%)",
        rle_runs_total,
        shift_gt0_blocks,
        preps.len(),
        100.0 * shift_gt0_blocks as f64 / preps.len() as f64
    );
}

// ===========================================================================
// Exactness verification across all 7 corpora + synthetic edge cases
// ===========================================================================

fn verify_exact(label: &str, preps: &[PreparedBlock], k: Kernels) {
    let mut arena0 = L0Arena::default();
    let mut arena3 = DisjointArena::default();
    let mut ordered = Vec::new();
    let mut out = Vec::new();
    let mut runs_s = Vec::new();
    let mut stage_s = Vec::new();
    let mut checked = 0usize;

    for (bi, p) in preps.iter().enumerate() {
        let fits =
            !p.wide && p.m >= 256 && (p.lens.len() - 1) <= ((u32::MAX >> 1) >> p.bits) as usize;
        if !fits {
            continue;
        }
        let mut edges = p.edges_fwd.clone();
        // L0, L1, L2
        for (name, mode) in [
            ("L0", L0Mode::ShippedInlined),
            ("L1", L0Mode::NoinlineRun),
            ("L2", L0Mode::InlineLong),
        ] {
            walk_l0_family(
                mode,
                &mut edges,
                &p.spelling,
                &p.inline16,
                &p.lens,
                p.n,
                p.mark,
                p.bits,
                &mut arena0,
                &mut out,
            );
            assert_eq!(out, p.expected_bwt, "{label} blk {bi} {name} BWT mismatch");
            let c = unrle_in_place_mirror(&mut out, &mut runs_s, &mut stage_s, k).unwrap();
            assert_eq!(c, p.expected_crc, "{label} blk {bi} {name} CRC mismatch");
            assert_eq!(out, p.expected_out, "{label} blk {bi} {name} out mismatch");
            assert_eq!(edges, p.edges_fwd, "{label} blk {bi} {name} untag leak");
        }
        // L3, L4 (stitch)
        for (name, nobc) in [("L3", false), ("L4", true)] {
            walk_disjoint_stitch(
                nobc,
                &mut edges,
                &p.spelling,
                &p.inline16,
                &p.lens,
                p.n,
                p.mark,
                p.bits,
                &mut arena3,
                &mut ordered,
                &mut out,
            );
            assert_eq!(out, p.expected_bwt, "{label} blk {bi} {name} BWT mismatch");
            let c = unrle_in_place_mirror(&mut out, &mut runs_s, &mut stage_s, k).unwrap();
            assert_eq!(c, p.expected_crc, "{label} blk {bi} {name} CRC mismatch");
            assert_eq!(out, p.expected_out, "{label} blk {bi} {name} out mismatch");
            assert_eq!(edges, p.edges_fwd, "{label} blk {bi} {name} untag leak");
        }
        // L5 (safe fused RLE+CRC), L6a (safe walk + AVX2 RLE+CRC), L6b (nobc walk + AVX2 RLE+CRC)
        for (name, nobc, avx2) in [
            ("L5_safe", false, false),
            ("L6a_safe_avx2", false, true),
            ("L6b_nobc_avx2", true, true),
        ] {
            let c = walk_disjoint_fused_rle_crc(
                nobc,
                avx2,
                &mut edges,
                &p.spelling,
                &p.inline16,
                &p.lens,
                p.n,
                p.mark,
                p.bits,
                &mut arena3,
                &mut ordered,
                &mut out,
                &mut runs_s,
                &mut stage_s,
                k,
            )
            .unwrap();
            assert_eq!(c, p.expected_crc, "{label} blk {bi} {name} CRC mismatch");
            assert_eq!(out, p.expected_out, "{label} blk {bi} {name} out mismatch");
            assert_eq!(edges, p.edges_fwd, "{label} blk {bi} {name} untag leak");
        }
        checked += 1;
    }
    println!(
        "  EXACT {label}: {checked}/{} Open blocks verified across all 8 arms (L0..L6b) — 100% byte+CRC exact!",
        preps.len()
    );
}

// ===========================================================================
// Track A Benchmark: Warm + L3-Cold (16-worker rotating pool + 64 MiB flush)
// ===========================================================================

#[derive(Clone, Copy)]
enum TrackAArm {
    L0WalkOnly,
    L1WalkOnly,
    L2WalkOnly,
    L3WalkOnly,
    L4WalkOnly,
    L0WithA9,
    L3WithA9,
    L4WithA9,
    L5FusedSafe,
    L6aSafeWalkAvx2Rle,
    L6bNobcWalkAvx2Rle,
}

impl TrackAArm {
    fn name(self) -> &'static str {
        match self {
            Self::L0WalkOnly => "L0_shipped_walk",
            Self::L1WalkOnly => "L1_noinline_walk",
            Self::L2WalkOnly => "L2_inline_long_walk",
            Self::L3WalkOnly => "L3_disjoint_safe_walk",
            Self::L4WalkOnly => "L4_disjoint_nobc_walk",
            Self::L0WithA9 => "L0_shipped_walk+A9",
            Self::L3WithA9 => "L3_disjoint_safe+A9",
            Self::L4WithA9 => "L4_disjoint_nobc+A9",
            Self::L5FusedSafe => "L5_fused_safe_RLE+CRC",
            Self::L6aSafeWalkAvx2Rle => "L6a_safe_walk+avx2_RLE",
            Self::L6bNobcWalkAvx2Rle => "L6b_nobc_walk+avx2_RLE",
        }
    }

    fn is_walk_only(self) -> bool {
        matches!(
            self,
            Self::L0WalkOnly
                | Self::L1WalkOnly
                | Self::L2WalkOnly
                | Self::L3WalkOnly
                | Self::L4WalkOnly
        )
    }
}

struct WorkerState {
    arena0: L0Arena,
    arena3: DisjointArena,
    ordered: Vec<(u16, u16, u16)>,
    out: Vec<u8>,
    runs_s: Vec<RleRun>,
    stage_s: Vec<u8>,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            arena0: L0Arena::default(),
            arena3: DisjointArena::default(),
            ordered: Vec::with_capacity(64),
            out: Vec::with_capacity(1 << 20),
            runs_s: Vec::with_capacity(16_384),
            stage_s: Vec::new(),
        }
    }
}

fn run_track_a_once(
    arm: TrackAArm,
    preps: &[PreparedBlock],
    edges_pool: &mut [Vec<u32>],
    workers: &mut [WorkerState],
    k: Kernels,
) -> (f64, f64, usize) {
    let n_workers = workers.len();
    let mut sink = 0u64;
    let c0 = thread_cpu_ns();
    let t0 = Instant::now();

    for (bi, p) in preps.iter().enumerate() {
        let w = &mut workers[bi % n_workers];
        let edges = &mut edges_pool[bi];
        match arm {
            TrackAArm::L0WalkOnly => {
                walk_l0_family(
                    L0Mode::ShippedInlined,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena0,
                    &mut w.out,
                );
                sink = sink.wrapping_add(w.out.len() as u64);
            }
            TrackAArm::L1WalkOnly => {
                walk_l0_family(
                    L0Mode::NoinlineRun,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena0,
                    &mut w.out,
                );
                sink = sink.wrapping_add(w.out.len() as u64);
            }
            TrackAArm::L2WalkOnly => {
                walk_l0_family(
                    L0Mode::InlineLong,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena0,
                    &mut w.out,
                );
                sink = sink.wrapping_add(w.out.len() as u64);
            }
            TrackAArm::L3WalkOnly => {
                walk_disjoint_stitch(
                    false,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                );
                sink = sink.wrapping_add(w.out.len() as u64);
            }
            TrackAArm::L4WalkOnly => {
                walk_disjoint_stitch(
                    true,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                );
                sink = sink.wrapping_add(w.out.len() as u64);
            }
            TrackAArm::L0WithA9 => {
                walk_l0_family(
                    L0Mode::ShippedInlined,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena0,
                    &mut w.out,
                );
                let c =
                    unrle_in_place_mirror(&mut w.out, &mut w.runs_s, &mut w.stage_s, k).unwrap();
                sink = sink.wrapping_add(c as u64);
            }
            TrackAArm::L3WithA9 => {
                walk_disjoint_stitch(
                    false,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                );
                let c =
                    unrle_in_place_mirror(&mut w.out, &mut w.runs_s, &mut w.stage_s, k).unwrap();
                sink = sink.wrapping_add(c as u64);
            }
            TrackAArm::L4WithA9 => {
                walk_disjoint_stitch(
                    true,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                );
                let c =
                    unrle_in_place_mirror(&mut w.out, &mut w.runs_s, &mut w.stage_s, k).unwrap();
                sink = sink.wrapping_add(c as u64);
            }
            TrackAArm::L5FusedSafe => {
                let c = walk_disjoint_fused_rle_crc(
                    false,
                    false,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                    &mut w.runs_s,
                    &mut w.stage_s,
                    k,
                )
                .unwrap();
                sink = sink.wrapping_add(c as u64);
            }
            TrackAArm::L6aSafeWalkAvx2Rle => {
                let c = walk_disjoint_fused_rle_crc(
                    false,
                    true,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                    &mut w.runs_s,
                    &mut w.stage_s,
                    k,
                )
                .unwrap();
                sink = sink.wrapping_add(c as u64);
            }
            TrackAArm::L6bNobcWalkAvx2Rle => {
                let c = walk_disjoint_fused_rle_crc(
                    true,
                    true,
                    edges,
                    &p.spelling,
                    &p.inline16,
                    &p.lens,
                    p.n,
                    p.mark,
                    p.bits,
                    &mut w.arena3,
                    &mut w.ordered,
                    &mut w.out,
                    &mut w.runs_s,
                    &mut w.stage_s,
                    k,
                )
                .unwrap();
                sink = sink.wrapping_add(c as u64);
            }
        }
    }

    let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
    let cpu_ms = (thread_cpu_ns().saturating_sub(c0)) as f64 * 1e-6;
    black_box(sink);
    let arena_bytes = match arm {
        TrackAArm::L0WalkOnly
        | TrackAArm::L1WalkOnly
        | TrackAArm::L2WalkOnly
        | TrackAArm::L0WithA9 => workers[0].arena0.scratch_bytes(),
        _ => workers[0].arena3.scratch_bytes(),
    };
    (cpu_ms, wall_ms, arena_bytes)
}

fn bench_track_a(preps: &[PreparedBlock], repeats: usize, cold: bool, k: Kernels) {
    let arms = [
        TrackAArm::L0WalkOnly,
        TrackAArm::L1WalkOnly,
        TrackAArm::L2WalkOnly,
        TrackAArm::L3WalkOnly,
        TrackAArm::L4WalkOnly,
        TrackAArm::L0WithA9,
        TrackAArm::L3WithA9,
        TrackAArm::L4WithA9,
        TrackAArm::L5FusedSafe,
        TrackAArm::L6aSafeWalkAvx2Rle,
        TrackAArm::L6bNobcWalkAvx2Rle,
    ];
    let n_workers = if cold { 16 } else { 1 };
    let mut workers: Vec<WorkerState> = (0..n_workers).map(|_| WorkerState::default()).collect();
    let mut edges_pool: Vec<Vec<u32>> = preps.iter().map(|p| p.edges_fwd.clone()).collect();
    let mut flush_buf = if cold {
        vec![0u8; COLD_FLUSH_BYTES]
    } else {
        Vec::new()
    };

    // Warmup once so arena allocations settle
    for &arm in &arms {
        let _ = run_track_a_once(arm, preps, &mut edges_pool, &mut workers, k);
    }

    let mut cpus: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut walls: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut mems = vec![0usize; arms.len()];

    for r in 0..repeats {
        let order: Vec<usize> = if r % 2 == 0 {
            (0..arms.len()).collect()
        } else {
            (0..arms.len()).rev().collect()
        };
        for &ai in &order {
            if cold {
                flush_l3(&mut flush_buf);
            }
            let (c, w, m) = run_track_a_once(arms[ai], preps, &mut edges_pool, &mut workers, k);
            cpus[ai].push(c);
            walls[ai].push(w);
            mems[ai] = m;
        }
    }

    let regime = if cold {
        "L3-COLD (16W pool + 64 MiB flush)"
    } else {
        "WARM (1W)"
    };
    println!(
        "\n=== TRACK A: {} ({} repeats, {} Open blocks) ===",
        regime,
        repeats,
        preps.len()
    );
    let (base_walk_med, base_walk_lo, base_walk_hi) = median_f64(&cpus[0]);
    let (base_a9_med, base_a9_lo, base_a9_hi) = median_f64(&cpus[5]);
    println!(
        "  {:<24} {:>10} {:>19} {:>9} {:>10} {:>10}",
        "arm", "CPU med", "[min..max]", "delta%", "disj?", "arena KiB"
    );
    for (ai, arm) in arms.iter().enumerate() {
        let (med, lo, hi) = median_f64(&cpus[ai]);
        let (b_med, b_lo, b_hi) = if arm.is_walk_only() {
            (base_walk_med, base_walk_lo, base_walk_hi)
        } else {
            (base_a9_med, base_a9_lo, base_a9_hi)
        };
        let delta = 100.0 * (med - b_med) / b_med;
        let disj = if ai == 0 || ai == 5 {
            "CTRL"
        } else if hi < b_lo {
            "FASTER*"
        } else if lo > b_hi {
            "SLOWER*"
        } else {
            "overlap"
        };
        println!(
            "  {:<24} {:>8.2}ms [{:>7.2}..{:>7.2}] {:>+8.2}% {:>10} {:>8.1}K",
            arm.name(),
            med,
            lo,
            hi,
            delta,
            disj,
            mems[ai] as f64 / 1024.0
        );
    }
    // Print sub-stage breakdown of L0_shipped_lanes (tag + walk + stitch + A9)
    let mut t_tag = 0u64;
    let mut t_walk = 0u64;
    let mut t_stitch = 0u64;
    let mut t_a9 = 0u64;
    if cold {
        flush_l3(&mut flush_buf);
    }
    for (bi, p) in preps.iter().enumerate() {
        let w = &mut workers[bi % n_workers];
        let edges = &mut edges_pool[bi];
        let m = edges.len();
        let c0 = thread_cpu_ns();
        let anchors: [u32; LANES_ANCHORS] = std::array::from_fn(|ak| {
            let a = p.mark as u64 + (ak as u64 * m as u64) / LANES_ANCHORS as u64;
            (a % m as u64) as u32
        });
        for &a in &anchors {
            edges[a as usize] |= LANES_TAG;
        }
        let slabs = p.n.div_ceil(LANES_SLAB) + LANES_K + 1;
        if w.arena0.bytes.len() < slabs * LANES_SLAB {
            w.arena0.bytes.resize(slabs * LANES_SLAB, 0);
        }
        w.arena0.slices.clear();
        let c1 = thread_cpu_ns();
        let mut lw = L0Walk {
            edges,
            spelling: &p.spelling,
            inline: &p.inline16,
            lens: &p.lens,
            bits: p.bits,
            mask: (1 << p.bits) - 1,
            bytes: &mut w.arena0.bytes[..slabs * LANES_SLAB],
            slices: &mut w.arena0.slices,
            top: 0,
            steps: 0,
            m,
            anchors,
            ends: [u32::MAX; LANES_ANCHORS],
            next: 0,
            bad: false,
        };
        let mut lanes = [L0Lane::default(); LANES_K];
        let mut live = 0usize;
        for _ in 0..LANES_K {
            let mut lane = L0Lane::default();
            if lw.slab(&mut lane) && lw.start(&mut lane) {
                lanes[live] = lane;
                live += 1;
            }
        }
        while live > 0 && !lw.bad {
            let ok = match live {
                4 => lw.run_inlined::<4>(&mut lanes),
                3 => lw.run_inlined::<3>(&mut lanes),
                2 => lw.run_inlined::<2>(&mut lanes),
                _ => lw.run_inlined::<1>(&mut lanes),
            };
            if !ok {
                lw.bad = true;
                break;
            }
            let mut i = 0;
            while i < live {
                let mut lane = lanes[i];
                if lw.hit(&lane) && !lw.start(&mut lane) {
                    live -= 1;
                    lanes[i] = lanes[live];
                    continue;
                }
                lanes[i] = lane;
                i += 1;
            }
        }
        let ends = lw.ends;
        for &a in &anchors {
            edges[a as usize] &= !LANES_TAG;
        }
        let c2 = thread_cpu_ns();
        w.out.clear();
        let mut seg = 0usize;
        for _ in 0..LANES_ANCHORS {
            for &(s, a, b) in w.arena0.slices.iter() {
                if usize::from(s) == seg {
                    let take = ((b - a) as usize).min(p.n - w.out.len());
                    w.out
                        .extend_from_slice(&w.arena0.bytes[a as usize..a as usize + take]);
                }
            }
            if w.out.len() >= p.n || ends[seg] == p.mark {
                break;
            }
            match anchors.iter().position(|&a| a == ends[seg]) {
                Some(next) => seg = next,
                None => break,
            }
        }
        repeat_cycle(p.n, &mut w.out);
        let c3 = thread_cpu_ns();
        let _ = unrle_in_place_mirror(&mut w.out, &mut w.runs_s, &mut w.stage_s, k).unwrap();
        let c4 = thread_cpu_ns();
        t_tag += c1.saturating_sub(c0);
        t_walk += c2.saturating_sub(c1);
        t_stitch += c3.saturating_sub(c2);
        t_a9 += c4.saturating_sub(c3);
    }
    let tot = (t_tag + t_walk + t_stitch + t_a9).max(1) as f64;
    println!(
        "  L0 sub-stages ({}): tag={:.2}ms ({:.1}%), walk={:.2}ms ({:.1}%), stitch={:.2}ms ({:.1}%), A9={:.2}ms ({:.1}%), sum={:.2}ms",
        if cold { "cold" } else { "warm" },
        t_tag as f64 * 1e-6, 100.0 * t_tag as f64 / tot,
        t_walk as f64 * 1e-6, 100.0 * t_walk as f64 / tot,
        t_stitch as f64 * 1e-6, 100.0 * t_stitch as f64 / tot,
        t_a9 as f64 * 1e-6, 100.0 * t_a9 as f64 / tot,
        tot * 1e-6,
    );
}

// ===========================================================================
// Track B: src/open.rs setup / classify / clear profiling (outside A6a/b/c/7/9)
// ===========================================================================

fn bench_track_b(preps: &[PreparedBlock], repeats: usize, cold: bool, k: Kernels) {
    let n_workers = if cold { 16 } else { 1 };
    let mut builders: Vec<OpenBuilder> = (0..n_workers).map(|_| OpenBuilder::default()).collect();
    let mut outs: Vec<Vec<u8>> = (0..n_workers)
        .map(|_| Vec::with_capacity(960 * 1024))
        .collect();
    let mut arenas: Vec<L0Arena> = (0..n_workers).map(|_| L0Arena::default()).collect();
    let mut runs_s: Vec<Vec<RleRun>> = (0..n_workers).map(|_| Vec::new()).collect();
    let mut stage_s: Vec<Vec<u8>> = (0..n_workers).map(|_| Vec::new()).collect();
    let mut flush_buf = if cold {
        vec![0u8; COLD_FLUSH_BYTES]
    } else {
        Vec::new()
    };

    let mut v_resize_recycled = Vec::new();
    let mut v_resize_cleared = Vec::new();
    let mut v_a6a = Vec::new();
    let mut v_strip_setup = Vec::new();
    let mut v_a6b = Vec::new();
    let mut v_a6c = Vec::new();
    let mut v_a7 = Vec::new();
    let mut v_a9 = Vec::new();
    let mut v_within_cap_5 = Vec::new();

    // Warmup
    for (bi, p) in preps.iter().enumerate() {
        let w = bi % n_workers;
        let out = &mut outs[w];
        out.resize(p.n + INLINE, 0);
        let _ = builders[w].prepare_block(&p.records, p.n, p.orig_ptr, PROD_LIMIT, out);
    }

    for _ in 0..repeats {
        if cold {
            flush_l3(&mut flush_buf);
        }
        let (
            mut ns_res_rec,
            mut ns_res_clr,
            mut ns_a6a,
            mut ns_strip,
            mut ns_a6b,
            mut ns_a6c,
            mut ns_a7,
            mut ns_a9,
            mut ns_wcap,
        ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);

        for (bi, p) in preps.iter().enumerate() {
            let wi = bi % n_workers;
            let b = &mut builders[wi];
            let out = &mut outs[wi];

            // Measure `within_cap` on dense-band blocks (matching production OpenCells::invert_runs)
            if let Some(cap) = open_cap(p.records.len(), p.n, p.orig_ptr, false) {
                if cap.m != u32::MAX {
                    let c0 = thread_cpu_ns();
                    black_box(within_cap(&p.records, p.orig_ptr, PROD_LIMIT, cap));
                    ns_wcap += thread_cpu_ns().saturating_sub(c0);
                }
            }

            // 1a. `out.resize(n + INLINE, 0)` when `out` kept previous block's len (steady-state free_out)
            let c0 = thread_cpu_ns();
            out.resize(p.n + INLINE, 0);
            let c1 = thread_cpu_ns();
            ns_res_rec += c1.saturating_sub(c0);

            // 2. A6a (maximal + Pass::take + OutQ16)
            let mut pass = Pass {
                queues: Queues::new(&mut out[..], &mut b.spill, u32::MAX),
                edges: &mut b.edges,
                ns: 0,
                cap_strips: usize::MAX,
                f: First::default(),
            };
            maximal(&p.records, |c, pos, width| {
                pass.take(c, pos, width, p.orig_ptr, PROD_LIMIT);
            });
            let ns = pass.ns;
            let (f, heads) = (pass.f, pass.queues.heads());
            let c2 = thread_cpu_ns();
            ns_a6a += c2.saturating_sub(c1);

            // 3. Track B: finish() strip setup + image_order + edges sizing + clear
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            let first = starts(all);
            let mut write = starts(f.thick);
            let most = OpenBuilder::nodes_most(ns);
            let wide_links = most > NARROW_NODES;
            let stride = if wide_links {
                Wide::STRIDE
            } else {
                Narrow::STRIDE
            };
            let need = (4 * ns + (stride + RULE) * most).max(f.m as usize);
            if b.edges.len() < need {
                let add = need - b.edges.len();
                b.edges.reserve_exact(add);
                b.edges.resize(need, 0);
            }
            let (strips_raw, rest_edges) = b.edges.split_at_mut(3 * ns);
            let image_order = &mut rest_edges[..ns];
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let byte = (sw[2] & 0xFF) as u8;
                let a = usize::from(byte);
                sw[1] += first[a];
                image_order[write[a] as usize] = i as u32;
                write[a] += 1;
            }
            b.bands.clear();
            b.spelling.clear();
            if p.n >= 65_536 {
                b.spelling.reserve_exact(48 * 1024);
            }
            let c3 = thread_cpu_ns();
            ns_strip += c3.saturating_sub(c2);

            // 4. A6b (induced_with: Elim + Dictionary + begin_inline)
            let cursors = if wide_links {
                b.induced_with::<Wide>(ns, p.n as u32, f.m, &f.thin)
            } else {
                b.induced_with::<Narrow>(ns, p.n as u32, f.m, &f.thin)
            };
            let c4 = thread_cpu_ns();
            ns_a6b += c4.saturating_sub(c3);

            // 5. A6c (wire_forward)
            let m = f.m as usize;
            let bits = (u32::BITS - (m.max(1) as u32 - 1).leading_zeros()).max(1);
            let wide = b.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if b.edges.len() < m {
                b.edges.resize(m, 0);
            }
            let mut k_rank = 0u32;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(b.bands.len());
                let mut queue = Reader::new(&out[..], &b.spill, &heads, a);
                for &band in &b.bands[cursors[a]..end] {
                    let bend = band.end();
                    let span = (bend - k_rank) as usize;
                    let dst = band.dst() as usize;
                    let pack = if wide { 0 } else { band.code() << bits };
                    queue.wire(&mut b.edges[dst..dst + span], pack);
                    k_rank = bend;
                }
            }
            let c5 = thread_cpu_ns();
            ns_a6c += c5.saturating_sub(c4);

            // 6. A7 (L0 shipped lanes::walk)
            walk_l0_family(
                L0Mode::ShippedInlined,
                &mut b.edges[..m],
                &b.spelling,
                &b.inline16,
                &b.lens,
                p.n,
                f.mark,
                bits,
                &mut arenas[wi],
                out,
            );
            let c6 = thread_cpu_ns();
            ns_a7 += c6.saturating_sub(c5);

            // 7. A9 (unrle_in_place + CRC32)
            let crc_v = unrle_in_place_mirror(out, &mut runs_s[wi], &mut stage_s[wi], k).unwrap();
            black_box(crc_v);
            let c7 = thread_cpu_ns();
            ns_a9 += c7.saturating_sub(c6);
        }

        // Separately measure `out.clear(); out.resize(n + INLINE, 0)` (cleared-every-block case,
        // e.g. `decode_block` / fresh allocations) on the same pool.
        if cold {
            flush_l3(&mut flush_buf);
        }
        let c0 = thread_cpu_ns();
        for (bi, p) in preps.iter().enumerate() {
            let out = &mut outs[bi % n_workers];
            out.clear();
            out.resize(p.n + INLINE, 0);
            black_box(out.as_ptr());
        }
        ns_res_clr += thread_cpu_ns().saturating_sub(c0);

        v_resize_recycled.push(ns_res_rec as f64 * 1e-6);
        v_resize_cleared.push(ns_res_clr as f64 * 1e-6);
        v_a6a.push(ns_a6a as f64 * 1e-6);
        v_strip_setup.push(ns_strip as f64 * 1e-6);
        v_a6b.push(ns_a6b as f64 * 1e-6);
        v_a6c.push(ns_a6c as f64 * 1e-6);
        v_a7.push(ns_a7 as f64 * 1e-6);
        v_a9.push(ns_a9 as f64 * 1e-6);
        v_within_cap_5.push(ns_wcap as f64 * 1e-6);
    }

    let r_rec = median_f64(&v_resize_recycled).0;
    let r_clr = median_f64(&v_resize_cleared).0;
    let m_a6a = median_f64(&v_a6a).0;
    let m_str = median_f64(&v_strip_setup).0;
    let m_a6b = median_f64(&v_a6b).0;
    let m_a6c = median_f64(&v_a6c).0;
    let m_a7 = median_f64(&v_a7).0;
    let m_a9 = median_f64(&v_a9).0;
    let m_wc = median_f64(&v_within_cap_5).0;
    let w_tot = r_rec + m_a6a + m_str + m_a6b + m_a6c + m_a7 + m_a9;

    println!(
        "\n=== TRACK B: src/open.rs FULL STAGE BREAKDOWN ({}, {} repeats, {} Open blocks) ===",
        if cold { "L3-COLD" } else { "WARM" },
        repeats,
        preps.len()
    );
    println!(
        "  out.resize(recycled free_out) : {:>7.2} ms ({:>5.2}% of W_open)",
        r_rec,
        100.0 * r_rec / w_tot
    );
    println!("  out.clear+resize(210 MiB zero): {:>7.2} ms ({:>5.2}% of W_open — only when out cleared!)", r_clr, 100.0 * r_clr / w_tot);
    println!(
        "  A6a (maximal+Pass+OutQ16)     : {:>7.2} ms ({:>5.2}%)",
        m_a6a,
        100.0 * m_a6a / w_tot
    );
    println!(
        "  finish() strip_setup+img_ord  : {:>7.2} ms ({:>5.2}%)",
        m_str,
        100.0 * m_str / w_tot
    );
    println!(
        "  within_cap (5 dense blks)     : {:>7.2} ms ({:>5.2}%)",
        m_wc,
        100.0 * m_wc / w_tot
    );
    println!(
        "  A6b (induced+dict)            : {:>7.2} ms ({:>5.2}%)",
        m_a6b,
        100.0 * m_a6b / w_tot
    );
    println!(
        "  A6c (wire_forward)            : {:>7.2} ms ({:>5.2}%)",
        m_a6c,
        100.0 * m_a6c / w_tot
    );
    println!(
        "  A7  (L0 shipped lanes::walk)  : {:>7.2} ms ({:>5.2}%)",
        m_a7,
        100.0 * m_a7 / w_tot
    );
    println!(
        "  A9  (unrle_in_place+CRC32)    : {:>7.2} ms ({:>5.2}%)",
        m_a9,
        100.0 * m_a9 / w_tot
    );
    println!("  ---------------------------------------------------------------");
    println!(
        "  W_open total (A6a..A9)        : {:>7.2} ms (100.0%)",
        w_tot
    );
    let track_b_slice = r_rec + m_str + m_wc;
    println!(
        "  Track B non-(A6a/b/c/7/9) sum : {:>7.2} ms ({:>5.2}% of W_open; with cleared-out: {:.2} ms / {:.2}%)",
        track_b_slice,
        100.0 * track_b_slice / w_tot,
        r_clr + m_str + m_wc,
        100.0 * (r_clr + m_str + m_wc) / w_tot
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut dir = "/tmp/lanes_corpora".to_string();
    let mut repeats = 7usize;
    let mut i = 1usize;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("repeats");
            i += 2;
        } else {
            dir = args[i].clone();
            i += 1;
        }
    }
    let k = Kernels::detect();
    println!(
        "open_lanes_b5_dissection: dir={dir}, repeats={repeats}, avx2={}",
        k.avx2()
    );

    let wiki_path = format!("{dir}/wikidata.bz2");
    let (n_tot, wiki_preps) = load_open_blocks(&wiki_path, k);
    println!(
        "Loaded wikidata.bz2: {n_tot} blocks, {} Open-admitted blocks",
        wiki_preps.len()
    );

    run_census(&wiki_preps);
    println!("\n=== EXACTNESS VERIFICATION (all 7 corpora) ===");
    verify_exact("wikidata", &wiki_preps, k);
    for guard in ["text", "random", "records", "pattern", "runs", "onebyte"] {
        let gp = format!("{dir}/{guard}.bz2");
        if std::path::Path::new(&gp).exists() {
            let (gtot, gpreps) = load_open_blocks(&gp, k);
            if gpreps.is_empty() {
                println!(
                    "  EXACT {guard}: {gtot} blocks, 0 Open-admitted blocks (all Segmented/Column)"
                );
            } else {
                verify_exact(guard, &gpreps, k);
            }
        }
    }

    bench_track_a(&wiki_preps, repeats, false, k);
    bench_track_a(&wiki_preps, repeats, true, k);
    bench_track_b(&wiki_preps, repeats, false, k);
    bench_track_b(&wiki_preps, repeats, true, k);
}
