#![forbid(unsafe_code)]
//! Smooth-zone per-block and per-stage worker L3 footprint census for claim `worker-l3-footprint`.
//!
//! Measures, on every block of a real `.bz2` corpus (and its `x4` concatenation), the exact
//! logical, actively-touched, and resident-in-L3 bytes held live simultaneously by one worker
//! across stages A5 (MTF/RLE2), A6a (Open first pass + OutQ16), A6b (Endpoint induction +
//! dictionary), A6c (Open wiring), A7 (Open emission), A9 (RLE1 + CRC32), and fallback
//! Segmented BWT routes.
//!
//! Strictly mirrors shipped `src/open.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`,
//! `src/runs.rs`, `src/rle.rs`, `src/bwt.rs`, and `src/lib.rs` with zero `src/` edits and
//! zero `unsafe`. Every block's decoded output is verified byte-for-byte against `mibz2::decode`.
//!
//! Usage:
//!   cargo run --release --example worker_l3_footprint_census -- corpora/wikidata.bz2 [--x4]

use mibz2::block;
use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, runs, Kernels};
use std::env;
use std::fs;
use std::mem::size_of;

// ---------------------------------------------------------------------------
// Production constants mirroring src/open.rs, endpoint.rs, outq.rs, cells_lazy.rs
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

const POOLED_FROM: usize = 64 << 10;
const MIN_SEGMENTED: usize = 1 << 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    Open,
    OpenDense,
    Skipped,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn open_cap(records: usize, n: usize, p: u32) -> Option<(Cap, Route)> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if lazy_counts_admit(records, n) {
        Some((Cap::NONE, Route::Open))
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some((Cap::dense(n), Route::OpenDense))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Exact mirror of OutQ16 (src/open/outq.rs)
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

    fn alloc(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE_CHUNK);
        }
        self.spill.next[id as usize] = NONE_CHUNK;
        if id >= self.in_out {
            self.spill.ovf.push([0u8; CHUNK_BYTES]);
        }
        let tail = self.tails[a];
        if tail == NONE_CHUNK {
            self.heads[a] = id;
        } else {
            self.spill.next[tail as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = 0;
    }

    fn push(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            if self.fill[a] as usize == CHUNK {
                self.alloc(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let id = self.tails[a];
            let chunk: &mut [u8] = if id < self.in_out {
                let off = id as usize * CHUNK_BYTES;
                &mut self.out[off..off + CHUNK_BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            };
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

    fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }
}

struct OutQReader<'a> {
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

impl<'a> OutQReader<'a> {
    fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
        let cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
        OutQReader {
            out,
            spill,
            in_out: heads.in_out,
            a,
            id: heads.heads[a],
            at: 0,
            read: 0,
            epoch: 0,
            cut,
        }
    }

    fn wire_xor(
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
                let off = self.id as usize * CHUNK_BYTES;
                &self.out[off..off + CHUNK_BYTES]
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

    fn wire(&mut self, mut dst: &mut [u32], pack: u32) {
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
                let off = self.id as usize * CHUNK_BYTES;
                &self.out[off..off + CHUNK_BYTES]
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
// Exact mirror of Open Kernel + Endpoint (src/open.rs & src/open/endpoint.rs)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
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
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    fn end(self) -> u32 {
        (self.0 & 0xF_FFFF) as u32
    }
    fn dst(self) -> u32 {
        ((self.0 >> 20) & 0xF_FFFF) as u32
    }
    fn code(self) -> u32 {
        (self.0 >> 40) as u32
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
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const BOTTOM: usize = 2;
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
}

fn literal(byte: u8) -> u32 {
    (1 << ID_BITS) | (u32::from(byte) + 1)
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

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

impl<L: Links> Elim<'_, L> {
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
    fn bottom(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::BOTTOM]
    }
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

    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }

    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }

    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }

    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

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
            if e & 1 != 0 {
                acc = self.join(acc, base);
            }
            e >>= 1;
            if e == 0 {
                return acc;
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
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
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
                    continue;
                }
            }
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
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

struct Dictionary<'a> {
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<'_, L>, handle: u32, byte: u8) -> u32 {
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

#[derive(Default)]
struct OpenKernel {
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

struct OpenMetrics {
    outq_in_out_bytes: usize,
    outq_spill_bytes: usize,
    a6a_strips_bytes: usize,
    a6b_arena_active_bytes: usize,
    a6b_bands_bytes: usize,
    a6b_stack_peak_bytes: usize,
    built_edges_bytes: usize,
    built_extra_bytes: usize,
    built_inline16_bytes: usize,
    built_lens_bytes: usize,
    built_spelling_bytes: usize,
    built_total_bytes: usize,
    built_oracle_bytes: usize,
    wide_edges: bool,
    bidi: bool,
    cap_edges_bytes: usize,
    cap_inline16_bytes: usize,
    cap_spelling_bytes: usize,
    cap_bands_bytes: usize,
    cap_spill_bytes: usize,
}

impl OpenKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn is_wide(ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }

    fn arena_words(ns: usize, m: usize) -> usize {
        let stride = if Self::is_wide(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    fn run_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<OpenMetrics> {
        out.resize(n + INLINE, 0);
        let mut thick_rows = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m = 0u32;
        let mut mark = 0u32;
        let mut ns = 0usize;

        let mut queues = Queues::new(&mut out[..], &mut self.spill, cap.m);
        let mut piece = |c: u8, x: u32, w: u32, keep: bool| {
            let a = usize::from(c);
            if keep {
                if p.wrapping_sub(x) < w {
                    mark = m + (p - x);
                }
                queues.push(a, m, w, &thin);
                thin[a] += w;
                m += w;
            } else {
                if ns < cap.strips {
                    let at = 3 * ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3)
                            .max(2 * self.edges.len())
                            .max(1024)
                            .min(cap.strips.saturating_mul(3));
                        self.edges.reserve_exact(need - self.edges.len());
                        self.edges.resize(need, 0);
                    }
                    self.edges[at..at + 3].copy_from_slice(&[
                        x,
                        thin[a] + thick_rows[a],
                        (w << 8) | u32::from(c),
                    ]);
                }
                ns += 1;
                thick[a] += 1;
                thick_rows[a] += w;
            }
        };

        maximal(records, |c, pos, width| {
            if width <= PROD_LIMIT {
                piece(c, pos, width, true);
            } else if p.wrapping_sub(pos) >= width {
                piece(c, pos, width, false);
            } else {
                split(c, pos, width, p, PROD_LIMIT, |cc, xx, ww, kk| {
                    piece(cc, xx, ww, kk)
                });
            }
        });

        let heads = queues.heads();
        let chunks_taken = queues.chunks as usize;
        let in_out_chunks = queues.in_out as usize;
        let over = queues.over;

        if over || m > cap.m || ns > cap.strips {
            return None;
        }

        let outq_in_out_bytes = chunks_taken.min(in_out_chunks) * CHUNK_BYTES;
        let outq_spill_bytes = chunks_taken * size_of::<u32>()
            + self.spill.ovf.len() * CHUNK_BYTES
            + self.spill.cuts.len() * 1024;
        let a6a_strips_bytes = 3 * ns * size_of::<u32>();

        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(thick);
        let need = Self::arena_words(ns, m as usize);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        {
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
        }

        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        let (cursors, top_words) = if Self::is_wide(ns) {
            self.induced::<Wide>(ns, n as u32, m, &thin)
        } else {
            self.induced::<Narrow>(ns, n as u32, m, &thin)
        };
        let a6b_arena_active_bytes = (4 * ns + top_words) * size_of::<u32>();
        let a6b_bands_bytes = self.bands.len() * size_of::<Band>();
        let a6b_stack_peak_bytes = self.stack.capacity() * size_of::<u32>();

        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide_edges = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide_edges {
            self.extra.resize(m as usize, 0);
        }
        self.predecessor = None;
        let bidi = !wide_edges && m >= 65_536;
        if bidi {
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut predecessor = mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut reader = OutQReader::new(out, &self.spill, &heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    reader.wire_xor(
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
                let mut reader = OutQReader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide_edges { 0 } else { bcode << bits };
                    if wide_edges {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    reader.wire(&mut self.edges[dst..dst + span], pack);
                    k = bend;
                }
            }
        }

        // Emit into `out` (exact verification)
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
            emit_forward(
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

        let built_edges_bytes = m as usize * size_of::<u32>();
        let built_extra_bytes = self.extra.len() * size_of::<u32>();
        let built_inline16_bytes = self.inline16.len() * INLINE;
        let built_lens_bytes = self.lens.len();
        let built_spelling_bytes = self.spelling.len();
        let built_total_bytes = built_edges_bytes
            + built_extra_bytes
            + built_inline16_bytes
            + built_lens_bytes
            + built_spelling_bytes;
        // Oracle store stores inline16[256..], lens[256..], edges[..m], extra, spelling + struct Entry
        let words = self.inline16.len() - 256;
        let built_oracle_bytes =
            built_edges_bytes + built_extra_bytes + built_spelling_bytes + words * (INLINE + 1);

        Some(OpenMetrics {
            outq_in_out_bytes,
            outq_spill_bytes,
            a6a_strips_bytes,
            a6b_arena_active_bytes,
            a6b_bands_bytes,
            a6b_stack_peak_bytes,
            built_edges_bytes,
            built_extra_bytes,
            built_inline16_bytes,
            built_lens_bytes,
            built_spelling_bytes,
            built_total_bytes,
            built_oracle_bytes,
            wide_edges,
            bidi,
            cap_edges_bytes: self.edges.capacity() * size_of::<u32>(),
            cap_inline16_bytes: self.inline16.capacity() * INLINE + self.lens.capacity(),
            cap_spelling_bytes: self.spelling.capacity(),
            cap_bands_bytes: self.bands.capacity() * size_of::<Band>(),
            cap_spill_bytes: self.spill.next.capacity() * size_of::<u32>()
                + self.spill.ovf.capacity() * CHUNK_BYTES
                + self.spill.cuts.capacity() * 1024,
        })
    }

    fn induced<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
    ) -> ([usize; 256], usize) {
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
                assert!(width > 0);
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
        (cursors, top)
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_forward(
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
// Exact mirror of RLE1 run scan (src/rle.rs) for rle_runs / shift / stage census
// ---------------------------------------------------------------------------

struct RleStats {
    rle_runs_bytes: usize,
    shift: usize,
    out_len: usize,
    staged: bool,
}

fn measure_rle_and_crc(
    bwt_out: &[u8],
    expected_crc: u32,
    k: Kernels,
    decoded_out: &mut Vec<u8>,
) -> RleStats {
    let n = bwt_out.len();
    let mut runs_count = 0usize;
    let mut grown = 0isize;
    let mut shift = 0isize;
    let mut i = 0usize;
    while i + 3 < n {
        let c = bwt_out[i];
        if bwt_out[i + 1] == c && bwt_out[i + 2] == c && bwt_out[i + 3] == c {
            let extra = bwt_out[i + 4] as isize;
            runs_count += 1;
            grown += extra - 1;
            if grown > shift {
                shift = grown;
            }
            i += 5;
        } else {
            i += 1;
        }
    }
    decoded_out.clear();
    rle::unrle(bwt_out, decoded_out, k).expect("rle::unrle");
    let computed = crc::crc32(decoded_out, k);
    assert_eq!(computed, expected_crc, "block CRC mismatch");
    let shift_u = shift as usize;
    let len = (n as isize + grown) as usize;
    assert_eq!(decoded_out.len(), len);
    RleStats {
        // src/rle.rs: pub type Run = (u32, u8, u8) -> 8 bytes in Rust layout (4 + 1 + 1 + 2 pad)
        rle_runs_bytes: runs_count * size_of::<(u32, u8, u8)>(),
        shift: shift_u,
        out_len: len,
        staged: shift_u > n,
    }
}

/// Exact mirror of `runs::front` run records (cap 127) from `ll`
fn records_of_ll(ll: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    for &b in ll {
        if let Some(last) = recs.last_mut() {
            if (*last >> 8) as u8 == b && (*last & 255) < 127 {
                *last += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

// ---------------------------------------------------------------------------
// Per-block footprint record
// ---------------------------------------------------------------------------

struct BlockRecord {
    route: Route,
    n: usize,
    out_len: usize,
    pooled_out_cap: usize,
    syms_bytes: usize,
    mtf_out_bytes: usize,
    mtf_bitmaps_bytes: usize,
    outq_in_out_bytes: usize,
    outq_spill_bytes: usize,
    built_bytes: usize,
    built_oracle_bytes: usize,
    a6b_arena_active_bytes: usize,
    a6b_bands_bytes: usize,
    rle_runs_bytes: usize,
    rle_staged: bool,
    stage_bytes: usize,
    tt_bytes: usize,
    // Simultaneously live bytes per stage (strictly respecting buffer aliasing & lifetimes):
    //   A5:  symbols + MTF output (runs + bitmaps + counts/per, or ll + counts)
    //   A6a: runs.runs + OutQ16 (in out) + Spill + strips (in edges)
    //   A6b: edges arena + bands + inline16 + lens + spelling + stack
    //        (active only, and active + dormant OutQ16 in out waiting for A6c)
    //   A6c: edges[..m] + extra + bands + OutQ16 (in out) + Spill
    //        (active only, and active + dormant inline16+lens+spelling waiting for A7)
    //   A7:  Built (edges[..m] + extra + inline16 + lens + spelling) + out[..n+16]
    //        (or Segmented fallback: tt + out/ll + SegScratch)
    //   A9:  out[..n+shift] + rle_runs (+ stage if staged)
    live_a5: usize,
    live_a6a: usize,
    live_a6b_active: usize,
    live_a6b_with_outq: usize,
    live_a6c_active: usize,
    live_a6c_with_dict: usize,
    live_a7: usize,
    live_a9: usize,
    peak_live_active: usize,
    peak_live_resident: usize,
}

fn stats_line(label: &str, vals: &[usize]) {
    if vals.is_empty() {
        return;
    }
    let mut s = vals.to_vec();
    s.sort_unstable();
    let n = s.len();
    let sum: u128 = s.iter().map(|&v| v as u128).sum();
    let mean = sum as f64 / n as f64;
    let p50 = s[n / 2] as f64;
    let p90 = s[(n * 90) / 100] as f64;
    let max = s[n - 1] as f64;
    let kib = 1024.0;
    println!(
        "  {label:<32} mean={:7.1} KiB  p50={:7.1} KiB  p90={:7.1} KiB  max={:7.1} KiB  (sum={:6.2} MiB)",
        mean / kib,
        p50 / kib,
        p90 / kib,
        max / kib,
        sum as f64 / (1024.0 * 1024.0)
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("corpora/wikidata.bz2");
    let x4 = args.iter().any(|a| a == "--x4");

    let raw = fs::read(path).expect("read corpus");
    let data = if x4 {
        let mut v = Vec::with_capacity(raw.len() * 4);
        for _ in 0..4 {
            v.extend_from_slice(&raw);
        }
        v
    } else {
        raw
    };

    let k = Kernels::detect();
    let expected = mibz2::decode(&data).expect("reference mibz2::decode");

    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms_buf = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::default();
    let mut open = OpenKernel::default();
    let mut work_out = Vec::new();
    let mut decoded_block = Vec::new();
    let mut reconstructed = Vec::with_capacity(expected.len());

    let mut records: Vec<BlockRecord> = Vec::new();

    // Persistent worker scratch capacity HWMs (single worker processing stream sequentially)
    let mut hwm_syms_cap = 0usize;
    let mut hwm_runs_cap = 0usize;
    let mut hwm_bitmaps_cap = 0usize;
    let mut hwm_ll_cap = 0usize;
    let mut hwm_tt_cap = 0usize;
    let mut hwm_rle_runs_cap = 0usize;
    let mut hwm_open_edges_cap = 0usize;
    let mut hwm_open_dict_cap = 0usize;
    let mut hwm_open_spell_cap = 0usize;
    let mut hwm_open_bands_cap = 0usize;
    let mut hwm_open_spill_cap = 0usize;
    let mut hwm_out_cap = 0usize;
    let mut open_wide_count = 0usize;
    let mut open_bidi_count = 0usize;
    let mut rle_staged_count = 0usize;

    let mut total_blocks = 0usize;
    loop {
        match scanner.next(syms_buf) {
            Ok(Event::Block { symbols: mut b, .. }) => {
                total_blocks += 1;
                let syms_len = b.syms.len();
                let syms_bytes = syms_len * size_of::<u16>();
                hwm_syms_cap = hwm_syms_cap.max(b.syms.capacity() * size_of::<u16>());

                let takes_runs = runs::takes_runs(&b);
                let n = mtf::mtf(&b, &mut ll, &mut counts, k).expect("mtf");
                let pooled_out_cap = if n >= POOLED_FROM { n + n / 16 + 64 } else { 0 };
                hwm_out_cap = hwm_out_cap.max(pooled_out_cap.max(n + INLINE));

                let route: Route;
                let mut mtf_out_bytes = n; // ll on column route
                let mut mtf_bitmaps_bytes = 0usize;
                let mut outq_in_out_bytes = 0usize;
                let mut outq_spill_bytes = 0usize;
                let mut built_bytes = 0usize;
                let mut built_oracle_bytes = 0usize;
                let mut a6b_arena_active_bytes = 0usize;
                let mut a6b_bands_bytes = 0usize;
                let mut stage_bytes = n; // ll holds column on column route
                let mut tt_bytes = 0usize;

                let live_a5: usize;
                let live_a6a: usize;
                let live_a6b_active: usize;
                let live_a6b_with_outq: usize;
                let live_a6c_active: usize;
                let live_a6c_with_dict: usize;
                let live_a7: usize;

                if takes_runs {
                    let recs = records_of_ll(&ll);
                    let recs_bytes = recs.len() * size_of::<u16>();
                    // AVX2 runs::front bitmaps: lit + one = 2 * ceil(syms_len/64) * 8 bytes
                    let words = syms_len.div_ceil(64);
                    let bitmaps_bytes = 2 * words * size_of::<u64>();
                    hwm_runs_cap = hwm_runs_cap.max(recs_bytes);
                    hwm_bitmaps_cap = hwm_bitmaps_cap.max(bitmaps_bytes);

                    mtf_out_bytes = recs_bytes + 2048; // runs + counts[256] + per[256]
                    mtf_bitmaps_bytes = bitmaps_bytes;
                    stage_bytes = 0; // ll is NOT touched on run route (unless RLE1 shift > n)
                    live_a5 = syms_bytes + mtf_out_bytes + bitmaps_bytes;

                    if let Some((cap, r)) = open_cap(recs.len(), n, b.orig_ptr) {
                        work_out.clear();
                        if let Some(om) = open.run_block(&recs, n, b.orig_ptr, cap, &mut work_out) {
                            route = r;
                            if om.wide_edges {
                                open_wide_count += 1;
                            }
                            if om.bidi {
                                open_bidi_count += 1;
                            }
                            outq_in_out_bytes = om.outq_in_out_bytes;
                            outq_spill_bytes = om.outq_spill_bytes;
                            built_bytes = om.built_total_bytes;
                            built_oracle_bytes = om.built_oracle_bytes;
                            a6b_arena_active_bytes = om.a6b_arena_active_bytes;
                            a6b_bands_bytes = om.a6b_bands_bytes;

                            hwm_open_edges_cap = hwm_open_edges_cap.max(om.cap_edges_bytes);
                            hwm_open_dict_cap = hwm_open_dict_cap.max(om.cap_inline16_bytes);
                            hwm_open_spell_cap = hwm_open_spell_cap.max(om.cap_spelling_bytes);
                            hwm_open_bands_cap = hwm_open_bands_cap.max(om.cap_bands_bytes);
                            hwm_open_spill_cap = hwm_open_spill_cap.max(om.cap_spill_bytes);

                            // A6a: reads runs.runs, writes OutQ16 (inside out) + Spill + strips (inside edges) + ~6 KiB stack
                            live_a6a = recs_bytes
                                + outq_in_out_bytes
                                + outq_spill_bytes
                                + om.a6a_strips_bytes
                                + 6144;
                            // A6b: active arena (inside edges, includes 24 KiB dedup cache) + bands + dict + stack
                            let dict_bytes = om.built_inline16_bytes
                                + om.built_lens_bytes
                                + om.built_spelling_bytes;
                            live_a6b_active = om.a6b_arena_active_bytes
                                + om.a6b_bands_bytes
                                + dict_bytes
                                + om.a6b_stack_peak_bytes;
                            live_a6b_with_outq =
                                live_a6b_active + outq_in_out_bytes + outq_spill_bytes;
                            // A6c: reads bands + OutQ16 (in out) + Spill, writes edges[..m] + extra
                            live_a6c_active = om.built_edges_bytes
                                + om.built_extra_bytes
                                + om.a6b_bands_bytes
                                + outq_in_out_bytes
                                + outq_spill_bytes;
                            live_a6c_with_dict = live_a6c_active + dict_bytes;
                            // A7: reads Built (edges[..m] + extra + inline16 + lens + spelling), writes out[..n+16]
                            live_a7 = om.built_total_bytes + n + INLINE;
                        } else {
                            route = Route::Skipped;
                            build(&ll, &counts, &mut tt);
                            work_out.clear();
                            walk(&tt, b.orig_ptr, &mut work_out);
                            tt_bytes = n * size_of::<u32>();
                            hwm_tt_cap = hwm_tt_cap.max(tt_bytes);
                            let seg_bytes = if n >= MIN_SEGMENTED {
                                // SegScratch: bitmap (ceil(n/64)*8) + 32-lane staging (~8 KiB/lane = 256 KiB)
                                n.div_ceil(64) * 8 + 256 * 1024
                            } else {
                                0
                            };
                            live_a6a = recs_bytes + tt_bytes;
                            live_a6b_active = 0;
                            live_a6b_with_outq = 0;
                            live_a6c_active = 0;
                            live_a6c_with_dict = 0;
                            live_a7 = tt_bytes + n + seg_bytes;
                        }
                    } else {
                        route = Route::Skipped;
                        build(&ll, &counts, &mut tt);
                        work_out.clear();
                        walk(&tt, b.orig_ptr, &mut work_out);
                        tt_bytes = n * size_of::<u32>();
                        hwm_tt_cap = hwm_tt_cap.max(tt_bytes);
                        let seg_bytes = if n >= MIN_SEGMENTED {
                            n.div_ceil(64) * 8 + 256 * 1024
                        } else {
                            0
                        };
                        live_a6a = recs_bytes + tt_bytes;
                        live_a6b_active = 0;
                        live_a6b_with_outq = 0;
                        live_a6c_active = 0;
                        live_a6c_with_dict = 0;
                        live_a7 = tt_bytes + n + seg_bytes;
                    }
                } else {
                    // Column route: mtf::mtf -> s.ll, then Segmented::invert_in_place(&mut s.ll), then rle::unrle(&s.ll, &mut out)
                    route = Route::Column;
                    hwm_ll_cap = hwm_ll_cap.max(n);
                    build(&ll, &counts, &mut tt);
                    work_out.clear();
                    walk(&tt, b.orig_ptr, &mut work_out);
                    tt_bytes = n * size_of::<u32>();
                    hwm_tt_cap = hwm_tt_cap.max(tt_bytes);
                    let seg_bytes = if n >= MIN_SEGMENTED {
                        n.div_ceil(64) * 8 + 256 * 1024
                    } else {
                        0
                    };
                    live_a5 = syms_bytes + n + 1024; // syms + ll + counts
                    live_a6a = n + tt_bytes; // build reads ll, writes tt
                    live_a6b_active = 0;
                    live_a6b_with_outq = 0;
                    live_a6c_active = 0;
                    live_a6c_with_dict = 0;
                    live_a7 = tt_bytes + n + seg_bytes; // walk overwrites ll in place
                }

                let rle_st = measure_rle_and_crc(&work_out, b.crc, k, &mut decoded_block);
                if rle_st.staged {
                    rle_staged_count += 1;
                    hwm_ll_cap = hwm_ll_cap.max(n);
                }
                hwm_rle_runs_cap = hwm_rle_runs_cap.max(rle_st.rle_runs_bytes);

                let live_a9 = if route == Route::Column {
                    // Column route calls rle::unrle(&s.ll, &mut out) + crc32(&out)
                    n + rle_st.out_len
                } else if rle_st.staged {
                    n + rle_st.out_len + rle_st.rle_runs_bytes
                } else {
                    // In-place RLE1+CRC: buf[..n+shift] + rle_runs
                    (n + rle_st.shift).max(rle_st.out_len) + rle_st.rle_runs_bytes
                };

                let peak_live_active = live_a5
                    .max(live_a6a)
                    .max(live_a6b_active)
                    .max(live_a6c_active)
                    .max(live_a7)
                    .max(live_a9);
                let peak_live_resident = live_a5
                    .max(live_a6a)
                    .max(live_a6b_with_outq)
                    .max(live_a6c_with_dict)
                    .max(live_a7)
                    .max(live_a9);

                reconstructed.extend_from_slice(&decoded_block);

                records.push(BlockRecord {
                    route,
                    n,
                    out_len: rle_st.out_len,
                    pooled_out_cap,
                    syms_bytes,
                    mtf_out_bytes,
                    mtf_bitmaps_bytes,
                    outq_in_out_bytes,
                    outq_spill_bytes,
                    built_bytes,
                    built_oracle_bytes,
                    a6b_arena_active_bytes,
                    a6b_bands_bytes,
                    rle_runs_bytes: rle_st.rle_runs_bytes,
                    rle_staged: rle_st.staged,
                    stage_bytes: if rle_st.staged { n } else { stage_bytes },
                    tt_bytes,
                    live_a5,
                    live_a6a,
                    live_a6b_active,
                    live_a6b_with_outq,
                    live_a6c_active,
                    live_a6c_with_dict,
                    live_a7,
                    live_a9,
                    peak_live_active,
                    peak_live_resident,
                });

                b.syms.clear();
                syms_buf = b.syms;
            }
            Ok(Event::Done) => break,
            Err(e) => panic!("scan error: {e:?}"),
        }
    }

    assert_eq!(
        reconstructed, expected,
        "EXACTNESS FAILED: mirror output != mibz2::decode"
    );

    println!("=== Worker L3 Footprint Census ===");
    println!(
        "Corpus: {path}{} | blocks={total_blocks} | decoded={} B ({:.2} MiB) | EXACT=OK",
        if x4 { " (x4)" } else { "" },
        reconstructed.len(),
        reconstructed.len() as f64 / (1024.0 * 1024.0),
    );

    let count_route = |r: Route| records.iter().filter(|b| b.route == r).count();
    let bytes_route = |r: Route| -> usize {
        records
            .iter()
            .filter(|b| b.route == r)
            .map(|b| b.out_len)
            .sum()
    };
    let tot_bytes: usize = records.iter().map(|b| b.out_len).sum();
    for (name, r) in [
        ("open", Route::Open),
        ("open_dense", Route::OpenDense),
        ("skipped", Route::Skipped),
        ("column", Route::Column),
    ] {
        let c = count_route(r);
        let b = bytes_route(r);
        println!(
            "  route {name:<10}: {:4} blocks ({:5.1}%), {:10} decoded B ({:5.2}%)",
            c,
            100.0 * c as f64 / total_blocks as f64,
            b,
            100.0 * b as f64 / tot_bytes as f64
        );
    }
    println!(
        "  open_bidi={open_bidi_count}, open_wide={open_wide_count}, rle_staged={rle_staged_count}"
    );

    // Cross-check sums vs c71c4a56 oracle_store (distinct 431 blocks)
    let sum_syms: usize = records.iter().map(|b| b.syms_bytes).sum();
    let sum_built_oracle: usize = records.iter().map(|b| b.built_oracle_bytes).sum();
    let sum_built_full: usize = records.iter().map(|b| b.built_bytes).sum();
    let sum_mtf_out: usize = records.iter().map(|b| b.mtf_out_bytes).sum();
    let sum_out: usize = records.iter().map(|b| b.out_len).sum();
    println!("\n--- Oracle-store cross-check (sum over all blocks) ---");
    println!(
        "  symbols bytes     : {sum_syms:>12} B ({:.2} MiB)",
        sum_syms as f64 / 1048576.0
    );
    println!(
        "  MTF out bytes     : {sum_mtf_out:>12} B ({:.2} MiB)",
        sum_mtf_out as f64 / 1048576.0
    );
    println!(
        "  Open Built (ora)  : {sum_built_oracle:>12} B ({:.2} MiB)",
        sum_built_oracle as f64 / 1048576.0
    );
    println!(
        "  Open Built (full) : {sum_built_full:>12} B ({:.2} MiB)",
        sum_built_full as f64 / 1048576.0
    );
    println!(
        "  Decoded out bytes : {sum_out:>12} B ({:.2} MiB)",
        sum_out as f64 / 1048576.0
    );

    type BlockPred = fn(&BlockRecord) -> bool;
    let subsets: &[(&str, BlockPred)] = &[
        ("ALL BLOCKS", |_| true),
        ("OPEN BLOCKS (open + open_dense)", |b| {
            matches!(b.route, Route::Open | Route::OpenDense)
        }),
        ("SKIPPED BLOCKS (run-route -> Segmented)", |b| {
            b.route == Route::Skipped
        }),
        ("COLUMN BLOCKS (!takes_runs -> Segmented)", |b| {
            b.route == Route::Column
        }),
    ];

    for (title, pred) in subsets {
        let sub: Vec<&BlockRecord> = records.iter().filter(|b| pred(b)).collect();
        println!("\n--- {title} (n={}) ---", sub.len());
        stats_line(
            "out (decoded len)",
            &sub.iter().map(|b| b.out_len).collect::<Vec<_>>(),
        );
        stats_line(
            "out (pooled cap)",
            &sub.iter().map(|b| b.pooled_out_cap).collect::<Vec<_>>(),
        );
        stats_line(
            "stage (s.ll)",
            &sub.iter().map(|b| b.stage_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "symbols (b.syms)",
            &sub.iter().map(|b| b.syms_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "MTF output (runs/ll+cnts)",
            &sub.iter().map(|b| b.mtf_out_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "MTF AVX2 bitmaps (lit+one)",
            &sub.iter().map(|b| b.mtf_bitmaps_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "OutQ16 (aliased in out)",
            &sub.iter().map(|b| b.outq_in_out_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "OutQ16 Spill (separate)",
            &sub.iter().map(|b| b.outq_spill_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "Open A6b arena (in edges)",
            &sub.iter()
                .map(|b| b.a6b_arena_active_bytes)
                .collect::<Vec<_>>(),
        );
        stats_line(
            "Open A6b bands",
            &sub.iter().map(|b| b.a6b_bands_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "Open Built (edges+dict)",
            &sub.iter().map(|b| b.built_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "RLE1 rle_runs",
            &sub.iter().map(|b| b.rle_runs_bytes).collect::<Vec<_>>(),
        );
        stats_line(
            "Segmented tt (4*n)",
            &sub.iter().map(|b| b.tt_bytes).collect::<Vec<_>>(),
        );
        println!("  [Simultaneously live per stage — respecting aliasing & lifetimes]");
        stats_line(
            "stage A5  (MTF/RLE2)",
            &sub.iter().map(|b| b.live_a5).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A6a (Open pass1)",
            &sub.iter().map(|b| b.live_a6a).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A6b (active only)",
            &sub.iter().map(|b| b.live_a6b_active).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A6b (+dormant OutQ16)",
            &sub.iter().map(|b| b.live_a6b_with_outq).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A6c (active only)",
            &sub.iter().map(|b| b.live_a6c_active).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A6c (+dormant dict)",
            &sub.iter().map(|b| b.live_a6c_with_dict).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A7  (Built + out)",
            &sub.iter().map(|b| b.live_a7).collect::<Vec<_>>(),
        );
        stats_line(
            "stage A9  (RLE1 + CRC)",
            &sub.iter().map(|b| b.live_a9).collect::<Vec<_>>(),
        );
        stats_line(
            "PEAK live (active)",
            &sub.iter().map(|b| b.peak_live_active).collect::<Vec<_>>(),
        );
        stats_line(
            "PEAK live (w/ dormant)",
            &sub.iter().map(|b| b.peak_live_resident).collect::<Vec<_>>(),
        );
    }

    // Byte-weighted means across ALL blocks (equivalent to time-average if work ~ out_len)
    let bw = |f: fn(&BlockRecord) -> usize| -> f64 {
        let num: f64 = records.iter().map(|b| b.out_len as f64 * f(b) as f64).sum();
        num / tot_bytes as f64
    };
    println!("\n--- Byte-weighted per-worker live set (MiB, weighted by decoded bytes) ---");
    let mib = 1024.0 * 1024.0;
    println!(
        "  A5  (MTF/RLE2)            : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a5) / mib,
        bw(|b| b.live_a5) / 1024.0
    );
    println!(
        "  A6a (Open first pass)     : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a6a) / mib,
        bw(|b| b.live_a6a) / 1024.0
    );
    println!(
        "  A6b (active only)         : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a6b_active) / mib,
        bw(|b| b.live_a6b_active) / 1024.0
    );
    println!(
        "  A6b (+dormant OutQ16)     : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a6b_with_outq) / mib,
        bw(|b| b.live_a6b_with_outq) / 1024.0
    );
    println!(
        "  A6c (active only)         : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a6c_active) / mib,
        bw(|b| b.live_a6c_active) / 1024.0
    );
    println!(
        "  A6c (+dormant dict)       : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a6c_with_dict) / mib,
        bw(|b| b.live_a6c_with_dict) / 1024.0
    );
    println!(
        "  A7  (Built + out)         : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a7) / mib,
        bw(|b| b.live_a7) / 1024.0
    );
    println!(
        "  A9  (RLE1 + CRC)          : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.live_a9) / mib,
        bw(|b| b.live_a9) / 1024.0
    );
    println!(
        "  PEAK per block (active)   : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.peak_live_active) / mib,
        bw(|b| b.peak_live_active) / 1024.0
    );
    println!(
        "  PEAK per block (resident) : {:.3} MiB ({:.1} KiB)",
        bw(|b| b.peak_live_resident) / mib,
        bw(|b| b.peak_live_resident) / 1024.0
    );

    // Steady-state worker Scratch HWM allocated capacity (virtual/RSS, NOT all L3-hot at once)
    let scratch_hwm = hwm_runs_cap
        + hwm_bitmaps_cap
        + hwm_ll_cap
        + hwm_tt_cap
        + hwm_rle_runs_cap
        + hwm_open_edges_cap
        + hwm_open_dict_cap
        + hwm_open_spell_cap
        + hwm_open_bands_cap
        + hwm_open_spill_cap;
    println!("\n--- Persistent Scratch HWM per worker (allocated capacity across stream) ---");
    println!("  runs.runs   : {:7.1} KiB", hwm_runs_cap as f64 / 1024.0);
    println!(
        "  runs bitmaps: {:7.1} KiB",
        hwm_bitmaps_cap as f64 / 1024.0
    );
    println!("  ll (stage)  : {:7.1} KiB", hwm_ll_cap as f64 / 1024.0);
    println!("  tt (fallback):{:7.1} KiB", hwm_tt_cap as f64 / 1024.0);
    println!(
        "  rle_runs    : {:7.1} KiB",
        hwm_rle_runs_cap as f64 / 1024.0
    );
    println!(
        "  open.edges  : {:7.1} KiB",
        hwm_open_edges_cap as f64 / 1024.0
    );
    println!(
        "  open.dict   : {:7.1} KiB",
        hwm_open_dict_cap as f64 / 1024.0
    );
    println!(
        "  open.spell  : {:7.1} KiB",
        hwm_open_spell_cap as f64 / 1024.0
    );
    println!(
        "  open.bands  : {:7.1} KiB",
        hwm_open_bands_cap as f64 / 1024.0
    );
    println!(
        "  open.spill  : {:7.1} KiB",
        hwm_open_spill_cap as f64 / 1024.0
    );
    println!(
        "  TOTAL Scratch HWM (excl. pooled out/syms): {:.3} MiB ({:.1} KiB)",
        scratch_hwm as f64 / mib,
        scratch_hwm as f64 / 1024.0
    );
    println!(
        "  + 1 pooled out ({:.1} KiB) + 1 pooled syms ({:.1} KiB): {:.3} MiB",
        hwm_out_cap as f64 / 1024.0,
        hwm_syms_cap as f64 / 1024.0,
        (scratch_hwm + hwm_out_cap + hwm_syms_cap) as f64 / mib
    );

    // Check unused fields so clippy stays clean
    let _ = records.iter().filter(|b| b.rle_staged).count();
    let _ = block::MAX_SYMS;
    let _ = records.iter().map(|b| b.n).sum::<usize>();
    let _ = records.iter().map(|b| b.tt_bytes).sum::<usize>();
}
