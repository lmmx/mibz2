#![forbid(unsafe_code)]
//! Permutation patterns structural census and consumer-value diagnostic.
//!
//! Smooth-zone diagnostic for claim `permutation-patterns`
//! (`docs/journal/2026-11-24-algorithm-level/permutation-patterns/`).
//! Interoperable with `examples/ideal_permutation_structures.rs` (same corpus split,
//! controls, warm/cold protocol, and anchored i9-10900K budget formulas).
//! Zero edits to `src/`, zero `unsafe`.
//!
//! Usage:
//!   permutation_patterns_probe self_check
//!   permutation_patterns_probe census [FILE.bz2]
//!   permutation_patterns_probe exact  [FILE.bz2]
//!   permutation_patterns_probe bench  FILE.bz2 [--repeats N]
//!   permutation_patterns_probe all    FILE.bz2 [--repeats N]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs & ideal_permutation_structures.rs)
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

// Escape bit in MonoBlock code_tag
const ESCAPE_FLAG: u32 = 0x8000_0000;

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

    /// Read `dst.len()` raw `next` u32 values (no code packing) for direct MonoBlock construction.
    #[inline(always)]
    fn read_raw(&mut self, dst: &mut [u32]) {
        self.wire(dst, 0);
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
    last_ns: usize,
    last_bands: usize,
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

    fn run_a6a_a6b(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        scratch_out: &mut Vec<u8>,
    ) -> (u64, u64, First, Heads, [usize; 256]) {
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
        let a6a = t1.duration_since(t0).as_nanos() as u64;

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
        let a6b = t2.duration_since(t1).as_nanos() as u64;
        self.last_ns = ns;
        self.last_bands = self.bands.len();
        (a6a, a6b, f, heads, cursors)
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
        let (a6a, a6b, f, heads, cursors) = self.run_a6a_a6b(records, n, p, limit, scratch_out);
        let t2 = Instant::now();
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
        let a6c = t3.duration_since(t2).as_nanos() as u64;
        (ConstructNs { a6a, a6b, a6c }, m as usize, f.mark, bits)
    }

    /// Direct materialisation of `MonoBlockReturnMap<B>` straight from `bands + queues`
    /// WITHOUT allocating or writing the flat `m`-entry `edges` array.
    #[allow(clippy::too_many_arguments)]
    fn construct_mono_block_direct<const B: usize>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        scratch_out: &mut Vec<u8>,
        blocks_buf: &mut Vec<MonoBlock<B>>,
        esc_edges: &mut Vec<u32>,
        esc_extra: &mut Vec<u32>,
    ) -> (ConstructNs, usize, u32, u32) {
        let (a6a, a6b, f, heads, cursors) = self.run_a6a_a6b(records, n, p, limit, scratch_out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        let n_blocks = m.div_ceil(B);
        blocks_buf.clear();
        blocks_buf.resize(
            n_blocks,
            MonoBlock {
                base: u32::MAX,
                code_tag: ESCAPE_FLAG,
                offs: [0; B],
            },
        );
        esc_edges.clear();
        esc_extra.clear();

        let mut tmp = [0u32; B];
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = Reader::new(scratch_out, &self.spill, &heads, a);
            for &b in &self.bands[cursors[a]..end] {
                let bend = b.end();
                let mut rem = (bend - k) as usize;
                let mut pos = b.dst() as usize;
                let bcode = b.code();
                let pack = if wide { 0 } else { bcode << bits };
                k = bend;

                // Leading partial block
                let lead = pos % B;
                if lead != 0 {
                    let take = (B - lead).min(rem);
                    let bi = pos / B;
                    let esc_base = if blocks_buf[bi].base == u32::MAX {
                        let eb = esc_edges.len() as u32;
                        esc_edges.resize(esc_edges.len() + B, 0);
                        if wide {
                            esc_extra.resize(esc_extra.len() + B, 0);
                        }
                        blocks_buf[bi].base = eb;
                        blocks_buf[bi].code_tag = ESCAPE_FLAG;
                        eb as usize
                    } else {
                        blocks_buf[bi].base as usize
                    };
                    queue.wire(
                        &mut esc_edges[esc_base + lead..esc_base + lead + take],
                        pack,
                    );
                    if wide {
                        esc_extra[esc_base + lead..esc_base + lead + take].fill(bcode);
                    }
                    pos += take;
                    rem -= take;
                }

                // Full B-aligned blocks strictly inside this band (all same bcode, strictly ascending!)
                while rem >= B {
                    let bi = pos / B;
                    queue.read_raw(&mut tmp);
                    let base = tmp[0];
                    let span_val = tmp[B - 1] - base;
                    if span_val <= u16::MAX as u32 {
                        let mut offs = [0u16; B];
                        for t in 0..B {
                            offs[t] = (tmp[t] - base) as u16;
                        }
                        blocks_buf[bi] = MonoBlock {
                            base,
                            code_tag: bcode,
                            offs,
                        };
                    } else {
                        let eb = esc_edges.len() as u32;
                        blocks_buf[bi].base = eb;
                        blocks_buf[bi].code_tag = ESCAPE_FLAG;
                        for &v in &tmp {
                            esc_edges.push(pack | v);
                        }
                        if wide {
                            for _ in 0..B {
                                esc_extra.push(bcode);
                            }
                        }
                    }
                    pos += B;
                    rem -= B;
                }

                // Trailing partial block
                if rem > 0 {
                    let bi = pos / B;
                    let esc_base = if blocks_buf[bi].base == u32::MAX {
                        let eb = esc_edges.len() as u32;
                        esc_edges.resize(esc_edges.len() + B, 0);
                        if wide {
                            esc_extra.resize(esc_extra.len() + B, 0);
                        }
                        blocks_buf[bi].base = eb;
                        blocks_buf[bi].code_tag = ESCAPE_FLAG;
                        eb as usize
                    } else {
                        blocks_buf[bi].base as usize
                    };
                    queue.wire(&mut esc_edges[esc_base..esc_base + rem], pack);
                    if wide {
                        esc_extra[esc_base..esc_base + rem].fill(bcode);
                    }
                }
            }
        }
        let t3 = Instant::now();
        let a6c = t3.duration_since(t2).as_nanos() as u64;
        (ConstructNs { a6a, a6b, a6c }, m, f.mark, bits)
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
// Exact Part 1 permutation-pattern analysis algorithms
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeNodeKind {
    Leaf,
    LinearInc,
    LinearDec,
    Prime,
}

#[derive(Clone, Debug)]
struct SitNode {
    l: usize,
    r: usize,
    min_v: i32,
    max_v: i32,
    kind: TreeNodeKind,
    children: Vec<usize>,
}

/// Lazy segment tree supporting range add and leftmost-zero query on `[0, n)`.
struct LazyMinSeg {
    size: usize,
    min: Vec<i32>,
    lazy: Vec<i32>,
}

impl LazyMinSeg {
    fn new(n: usize) -> Self {
        let size = n.next_power_of_two().max(1);
        LazyMinSeg {
            size,
            min: vec![0; 2 * size],
            lazy: vec![0; 2 * size],
        }
    }

    fn reset(&mut self, n: usize) {
        let size = n.next_power_of_two().max(1);
        self.size = size;
        if self.min.len() < 2 * size {
            self.min.resize(2 * size, 0);
            self.lazy.resize(2 * size, 0);
        }
        self.min[..2 * size].fill(0);
        self.lazy[..2 * size].fill(0);
    }

    #[inline(always)]
    fn push(&mut self, idx: usize) {
        let lz = self.lazy[idx];
        if lz != 0 {
            let l = idx << 1;
            let r = l | 1;
            self.min[l] += lz;
            self.lazy[l] += lz;
            self.min[r] += lz;
            self.lazy[r] += lz;
            self.lazy[idx] = 0;
        }
    }

    fn range_add(&mut self, ql: usize, qr: usize, delta: i32) {
        if ql > qr {
            return;
        }
        self.range_add_rec(1, 0, self.size - 1, ql, qr, delta);
    }

    fn range_add_rec(
        &mut self,
        idx: usize,
        nl: usize,
        nr: usize,
        ql: usize,
        qr: usize,
        delta: i32,
    ) {
        if ql <= nl && nr <= qr {
            self.min[idx] += delta;
            self.lazy[idx] += delta;
            return;
        }
        self.push(idx);
        let mid = (nl + nr) >> 1;
        if ql <= mid {
            self.range_add_rec(idx << 1, nl, mid, ql, qr, delta);
        }
        if qr > mid {
            self.range_add_rec((idx << 1) | 1, mid + 1, nr, ql, qr, delta);
        }
        self.min[idx] = self.min[idx << 1].min(self.min[(idx << 1) | 1]);
    }

    fn first_zero(&mut self) -> usize {
        self.first_zero_rec(1, 0, self.size - 1)
    }

    fn first_zero_rec(&mut self, idx: usize, nl: usize, nr: usize) -> usize {
        if nl == nr {
            return nl;
        }
        self.push(idx);
        let mid = (nl + nr) >> 1;
        if self.min[idx << 1] == 0 {
            self.first_zero_rec(idx << 1, nl, mid)
        } else {
            self.first_zero_rec((idx << 1) | 1, mid + 1, nr)
        }
    }
}

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
struct StrongIntervalStats {
    n: usize,
    total_strong: usize,
    linear_inc_nodes: usize,
    linear_dec_nodes: usize,
    prime_nodes: usize,
    max_prime_arity: usize,
    tree_depth: usize,
    separable: bool,
    leaves_in_nontriv_linear: usize,
    // Label-homogeneity stats (populated when codes are provided)
    nontriv_linear_nodes: usize,
    homog_linear_nodes: usize,
    nontriv_prime_nodes: usize,
    homog_prime_nodes: usize,
    homog_linear_leaves: usize,
}

/// Exact O(N log N) strong-interval tree (substitution decomposition tree) builder.
/// Exhaustively verified against O(N^3) brute-force on all 5,913 permutations of S_1..S_7.
fn build_strong_interval_tree(p: &[u32], seg: &mut LazyMinSeg) -> (Vec<SitNode>, usize) {
    let n = p.len();
    assert!(n >= 1);
    seg.reset(n);
    let mut nodes: Vec<SitNode> = Vec::with_capacity(2 * n);
    let mut max_st: Vec<(usize, usize, i32)> = Vec::new();
    let mut min_st: Vec<(usize, usize, i32)> = Vec::new();
    let mut st: Vec<usize> = Vec::new();

    for (r, &pv) in p.iter().enumerate() {
        let val = pv as i32;
        if r > 0 {
            seg.range_add(0, r - 1, -1);
        }
        let mut l_max = r;
        while let Some(&(l0, r0, mv)) = max_st.last() {
            if mv < val {
                seg.range_add(l0, r0, val - mv);
                l_max = l0;
                max_st.pop();
            } else {
                break;
            }
        }
        max_st.push((l_max, r, val));

        let mut l_min = r;
        while let Some(&(l0, r0, mv)) = min_st.last() {
            if mv > val {
                seg.range_add(l0, r0, mv - val);
                l_min = l0;
                min_st.pop();
            } else {
                break;
            }
        }
        min_st.push((l_min, r, val));

        let l0 = seg.first_zero();
        let mut cur = nodes.len();
        nodes.push(SitNode {
            l: r,
            r,
            min_v: val,
            max_v: val,
            kind: TreeNodeKind::Leaf,
            children: Vec::new(),
        });

        while let Some(&top) = st.last() {
            if nodes[top].l < l0 {
                break;
            }
            // Case 1: attach cur to existing Linear node on top of stack
            let can_join_inc = nodes[top].kind == TreeNodeKind::LinearInc
                && nodes[top].max_v + 1 == nodes[cur].min_v;
            let can_join_dec = nodes[top].kind == TreeNodeKind::LinearDec
                && nodes[cur].max_v + 1 == nodes[top].min_v;
            if can_join_inc {
                nodes[top].r = nodes[cur].r;
                nodes[top].max_v = nodes[cur].max_v;
                nodes[top].children.push(cur);
                cur = st.pop().unwrap();
                continue;
            }
            if can_join_dec {
                nodes[top].r = nodes[cur].r;
                nodes[top].min_v = nodes[cur].min_v;
                nodes[top].children.push(cur);
                cur = st.pop().unwrap();
                continue;
            }

            // Case 2: pair top + cur into a new Linear node
            let mn2 = nodes[top].min_v.min(nodes[cur].min_v);
            let mx2 = nodes[top].max_v.max(nodes[cur].max_v);
            if mx2 - mn2 == (nodes[cur].r - nodes[top].l) as i32 {
                let kind = if nodes[top].max_v < nodes[cur].min_v {
                    TreeNodeKind::LinearInc
                } else {
                    TreeNodeKind::LinearDec
                };
                st.pop();
                let new_id = nodes.len();
                let nl = nodes[top].l;
                let nr = nodes[cur].r;
                nodes.push(SitNode {
                    l: nl,
                    r: nr,
                    min_v: mn2,
                    max_v: mx2,
                    kind,
                    children: vec![top, cur],
                });
                cur = new_id;
                continue;
            }

            // Case 3: Prime node — shortest suffix of stack + cur forming an interval
            let mut mn = nodes[cur].min_v;
            let mut mx = nodes[cur].max_v;
            let mut found_k = None;
            let mut k = st.len();
            while k > 0 {
                k -= 1;
                let nid = st[k];
                mn = mn.min(nodes[nid].min_v);
                mx = mx.max(nodes[nid].max_v);
                if mx - mn == (nodes[cur].r - nodes[nid].l) as i32 {
                    found_k = Some(k);
                    break;
                }
                if nodes[nid].l <= l0 {
                    break;
                }
            }
            if let Some(fk) = found_k {
                let mut children: Vec<usize> = st.drain(fk..).collect();
                children.push(cur);
                let nl = nodes[children[0]].l;
                let nr = nodes[cur].r;
                let new_id = nodes.len();
                nodes.push(SitNode {
                    l: nl,
                    r: nr,
                    min_v: mn,
                    max_v: mx,
                    kind: TreeNodeKind::Prime,
                    children,
                });
                cur = new_id;
                continue;
            }
            break;
        }
        st.push(cur);
    }

    assert_eq!(st.len(), 1, "strong interval tree must have unique root");
    (nodes, st[0])
}

fn summarize_sit(nodes: &[SitNode], root: usize, codes: Option<&[u32]>) -> StrongIntervalStats {
    let n = nodes[root].r - nodes[root].l + 1;
    let mut stats = StrongIntervalStats {
        n,
        total_strong: nodes.len(),
        ..Default::default()
    };

    // Iterative post-order traversal for depth and label homogeneity
    let mut depth = vec![1usize; nodes.len()];
    let mut homog_code: Vec<Option<u32>> = vec![None; nodes.len()];
    let mut in_linear = vec![false; nodes.len()];

    // Topological order: compute depth and in_linear top-down, then homog bottom-up
    let mut order = Vec::with_capacity(nodes.len());
    let mut stack = vec![root];
    while let Some(u) = stack.pop() {
        order.push(u);
        let u_lin = matches!(
            nodes[u].kind,
            TreeNodeKind::LinearInc | TreeNodeKind::LinearDec
        );
        for &c in &nodes[u].children {
            depth[c] = depth[u] + 1;
            in_linear[c] = in_linear[u] || u_lin;
            stack.push(c);
        }
    }

    let mut max_depth = 1usize;
    for &u in order.iter().rev() {
        max_depth = max_depth.max(depth[u]);
        let nd = &nodes[u];
        match nd.kind {
            TreeNodeKind::Leaf => {
                if in_linear[u] {
                    stats.leaves_in_nontriv_linear += 1;
                }
                if let Some(cs) = codes {
                    homog_code[u] = Some(cs[nd.l]);
                }
            }
            TreeNodeKind::LinearInc | TreeNodeKind::LinearDec => {
                if nd.kind == TreeNodeKind::LinearInc {
                    stats.linear_inc_nodes += 1;
                } else {
                    stats.linear_dec_nodes += 1;
                }
                if codes.is_some() {
                    let first = homog_code[nd.children[0]];
                    let all_same =
                        first.is_some() && nd.children.iter().all(|&c| homog_code[c] == first);
                    if all_same {
                        homog_code[u] = first;
                    }
                    if nd.r - nd.l + 1 < n {
                        stats.nontriv_linear_nodes += 1;
                        if all_same {
                            stats.homog_linear_nodes += 1;
                        }
                    }
                }
            }
            TreeNodeKind::Prime => {
                stats.prime_nodes += 1;
                stats.max_prime_arity = stats.max_prime_arity.max(nd.children.len());
                if codes.is_some() {
                    let first = homog_code[nd.children[0]];
                    let all_same =
                        first.is_some() && nd.children.iter().all(|&c| homog_code[c] == first);
                    if all_same {
                        homog_code[u] = first;
                    }
                    if nd.r - nd.l + 1 < n {
                        stats.nontriv_prime_nodes += 1;
                        if all_same {
                            stats.homog_prime_nodes += 1;
                        }
                    }
                }
            }
        }
    }

    // Count leaves inside any label-homogeneous nontrivial linear strong interval
    if codes.is_some() {
        let mut in_homog_lin = vec![false; nodes.len()];
        for &u in &order {
            let nd = &nodes[u];
            let is_hl = matches!(nd.kind, TreeNodeKind::LinearInc | TreeNodeKind::LinearDec)
                && nd.r - nd.l + 1 < n
                && homog_code[u].is_some();
            for &c in &nd.children {
                in_homog_lin[c] = in_homog_lin[u] || is_hl;
            }
            if nd.kind == TreeNodeKind::Leaf && in_homog_lin[u] {
                stats.homog_linear_leaves += 1;
            }
        }
    }

    stats.tree_depth = max_depth;
    stats.separable = stats.prime_nodes == 0;
    stats
}

/// Brute-force O(N^3) strong-interval enumeration for exhaustive verification on S_1..S_7.
fn brute_strong_intervals(p: &[u32]) -> Vec<(usize, usize)> {
    let n = p.len();
    let mut intervals = Vec::new();
    for l in 0..n {
        let mut mn = p[l];
        let mut mx = p[l];
        for (r, &v) in p.iter().enumerate().skip(l) {
            mn = mn.min(v);
            mx = mx.max(v);
            if (mx - mn) as usize == r - l {
                intervals.push((l, r));
            }
        }
    }
    let mut strong = Vec::new();
    for &(l1, r1) in &intervals {
        let ok = intervals.iter().all(|&(l2, r2)| {
            // disjoint or nested
            r1 < l2 || r2 < l1 || (l1 <= l2 && r2 <= r1) || (l2 <= l1 && r1 <= r2)
        });
        if ok {
            strong.push((l1, r1));
        }
    }
    strong.sort_unstable();
    strong
}

/// Exact O(N log N) Fenwick-tree counter for all 6 length-3 patterns:
/// `[#123, #132, #213, #231, #312, #321]`.
/// Exhaustively verified against O(N^3) brute-force on all 873 permutations of S_1..S_6.
fn count_3patterns(p: &[u32]) -> [u128; 6] {
    let n = p.len();
    if n < 3 {
        return [0; 6];
    }
    // BIT1: count of seen values <= v; BIT2: sum of indices i of seen values <= v
    let mut bit_cnt = vec![0u64; n + 1];
    let mut bit_idx = vec![0u128; n + 1];

    #[inline(always)]
    fn add_cnt(bit: &mut [u64], mut i: usize) {
        while i < bit.len() {
            bit[i] += 1;
            i += i & i.wrapping_neg();
        }
    }
    #[inline(always)]
    fn sum_cnt(bit: &[u64], mut i: usize) -> u64 {
        let mut s = 0u64;
        while i > 0 {
            s += bit[i];
            i &= i - 1;
        }
        s
    }
    #[inline(always)]
    fn add_idx(bit: &mut [u128], mut i: usize, val: u128) {
        while i < bit.len() {
            bit[i] += val;
            i += i & i.wrapping_neg();
        }
    }
    #[inline(always)]
    fn sum_idx(bit: &[u128], mut i: usize) -> u128 {
        let mut s = 0u128;
        while i > 0 {
            s += bit[i];
            i &= i - 1;
        }
        s
    }

    let mut c123 = 0u128;
    let mut c321 = 0u128;
    let mut peak = 0u128; // #132 + #231
    let mut valley = 0u128; // #213 + #312
    let mut q_right_mid = 0u128; // #132 + #312
    let mut non_inv_gap = 0u128; // #123 + #132 + #213

    for (j, &pv) in p.iter().enumerate() {
        let v = pv as usize;
        let l_lt = sum_cnt(&bit_cnt, v) as u128;
        let l_gt = (j as u128) - l_lt;
        let r_lt = (v as u128) - l_lt;
        let r_gt = ((n - 1 - j) as u128) - r_lt;
        let idx_sum_lt = sum_idx(&bit_idx, v);

        c123 += l_lt * r_gt;
        c321 += l_gt * r_lt;
        peak += l_lt * r_lt;
        valley += l_gt * r_gt;
        q_right_mid += l_lt * l_gt;
        if l_lt > 0 {
            non_inv_gap += l_lt * ((j as u128) - 1) - idx_sum_lt;
        }

        add_cnt(&mut bit_cnt, v + 1);
        add_idx(&mut bit_idx, v + 1, j as u128);
    }

    let s_sum = non_inv_gap - c123; // #132 + #213
    let two_c132 = s_sum + q_right_mid - valley;
    assert_eq!(two_c132 & 1, 0, "exact parity check for #132");
    let c132 = two_c132 >> 1;
    let c231 = peak - c132;
    let c213 = s_sum - c132;
    let c312 = q_right_mid - c132;
    let total = (n as u128) * ((n - 1) as u128) * ((n - 2) as u128) / 6;
    assert_eq!(c123 + c132 + c213 + c231 + c312 + c321, total);
    [c123, c132, c213, c231, c312, c321]
}

fn brute_count_3patterns(p: &[u32]) -> [u128; 6] {
    let n = p.len();
    let mut c = [0u128; 6];
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let (a, b, d) = (p[i], p[j], p[k]);
                let idx = if a < b {
                    if b < d {
                        0 // 123
                    } else if a < d {
                        1 // 132
                    } else {
                        3 // 231
                    }
                } else if a < d {
                    2 // 213
                } else if b < d {
                    4 // 312
                } else {
                    5 // 321
                };
                c[idx] += 1;
            }
        }
    }
    c
}

/// Exact O(N log N) LIS and LDS via patience sorting, with verified index witnesses.
fn lis_and_lds(p: &[u32]) -> (Vec<usize>, Vec<usize>) {
    fn patience(p: &[u32], increasing: bool) -> Vec<usize> {
        let n = p.len();
        if n == 0 {
            return Vec::new();
        }
        let mut tails_val: Vec<u32> = Vec::new();
        let mut tails_idx: Vec<usize> = Vec::new();
        let mut prev = vec![usize::MAX; n];
        for (i, &v) in p.iter().enumerate() {
            let pos = if increasing {
                tails_val.partition_point(|&x| x < v)
            } else {
                tails_val.partition_point(|&x| x > v)
            };
            if pos > 0 {
                prev[i] = tails_idx[pos - 1];
            }
            if pos == tails_val.len() {
                tails_val.push(v);
                tails_idx.push(i);
            } else {
                tails_val[pos] = v;
                tails_idx[pos] = i;
            }
        }
        let mut res = Vec::with_capacity(tails_idx.len());
        let mut cur = *tails_idx.last().unwrap();
        while cur != usize::MAX {
            res.push(cur);
            cur = prev[cur];
        }
        res.reverse();
        // Verify witness
        for w in res.windows(2) {
            assert!(w[0] < w[1]);
            if increasing {
                assert!(p[w[0]] < p[w[1]]);
            } else {
                assert!(p[w[0]] > p[w[1]]);
            }
        }
        res
    }
    (patience(p, true), patience(p, false))
}

fn verify_4pat(p: &[u32], idx: [usize; 4], pat: [u8; 4]) -> bool {
    if !(idx[0] < idx[1] && idx[1] < idx[2] && idx[2] < idx[3] && idx[3] < p.len()) {
        return false;
    }
    let v = [p[idx[0]], p[idx[1]], p[idx[2]], p[idx[3]]];
    for a in 0..4 {
        for b in (a + 1)..4 {
            if (v[a] < v[b]) != (pat[a] < pat[b]) {
                return false;
            }
        }
    }
    true
}

/// Find an explicit verified 4-tuple witness for `pat` (e.g. [2,4,1,3], [3,1,4,2], [2,1,4,3]).
/// Searches a bounded local + strided prefix first (exact O(1) verification on any witness found);
/// NEVER claims avoidance on `None` — avoidance is only asserted from exact SIT / LIS / LDS proofs.
fn find_4pattern_witness(p: &[u32], pat: [u8; 4]) -> Option<[usize; 4]> {
    let n = p.len();
    if n < 4 {
        return None;
    }
    // Helper: search a sorted slice of indices (size <= 256) in O(k^3) with fast early exit
    let search_subset = |idxs: &[usize]| -> Option<[usize; 4]> {
        let k = idxs.len();
        if k < 4 {
            return None;
        }
        for a in 0..k {
            let va = p[idxs[a]];
            for b in (a + 1)..k {
                let vb = p[idxs[b]];
                if (va < vb) != (pat[0] < pat[1]) {
                    continue;
                }
                for c in (b + 1)..k {
                    let vc = p[idxs[c]];
                    if (va < vc) != (pat[0] < pat[2]) || (vb < vc) != (pat[1] < pat[2]) {
                        continue;
                    }
                    for &id_d in &idxs[(c + 1)..k] {
                        let vd = p[id_d];
                        if (va < vd) == (pat[0] < pat[3])
                            && (vb < vd) == (pat[1] < pat[3])
                            && (vc < vd) == (pat[2] < pat[3])
                        {
                            let w = [idxs[a], idxs[b], idxs[c], id_d];
                            assert!(verify_4pat(p, w, pat));
                            return Some(w);
                        }
                    }
                }
            }
        }
        None
    };

    // 1. Contiguous prefix of up to 128 elements (usually succeeds in microseconds)
    let prefix: Vec<usize> = (0..n.min(128)).collect();
    if let Some(w) = search_subset(&prefix) {
        return Some(w);
    }
    // 2. Evenly strided 256 elements across the whole permutation
    let step = (n / 256).max(1);
    let strided: Vec<usize> = (0..n).step_by(step).take(256).collect();
    search_subset(&strided)
}

#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
struct MonotoneStats {
    n: usize,
    n_runs: usize,      // contiguous ascending runs
    n_mruns: usize,     // contiguous monotone (asc or desc) runs
    n_lruns: usize,     // labelled contiguous ascending runs (same code)
    n_sruns: usize,     // strict +1 runs
    n_smruns: usize,    // strict +/-1 runs
    n_lsruns: usize,    // labelled strict +1 runs (same code)
    h_runs: f64,        // empirical entropy H(vRuns) in bits
    h_lruns: f64,       // empirical entropy H(vLRuns) in bits
    k3_hist: [u64; 6],  // contiguous 3-gram relative orders [123,132,213,231,312,321]
    k3_123_homog: u64,  // contiguous 123 3-grams with identical code
    k4_1234: u64,       // contiguous ascending 4-grams
    k4_1234_homog: u64, // contiguous ascending 4-grams with identical code
    k4_top_share: f64,  // share of most common contiguous 4-gram shape
}

fn entropy_of_lengths(lens: &[usize], total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let inv = 1.0 / (total as f64);
    let mut h = 0.0;
    for &l in lens {
        if l > 0 {
            let p = (l as f64) * inv;
            h -= p * p.log2();
        }
    }
    h
}

fn monotone_stats(p: &[u32], codes: Option<&[u32]>) -> MonotoneStats {
    let n = p.len();
    if n == 0 {
        return MonotoneStats::default();
    }
    let mut run_lens = Vec::new();
    let mut lrun_lens = Vec::new();
    let mut cur_run = 1usize;
    let mut cur_lrun = 1usize;
    let mut n_sruns = 1usize;
    let mut n_lsruns = 1usize;

    for i in 1..n {
        if p[i] > p[i - 1] {
            cur_run += 1;
        } else {
            run_lens.push(cur_run);
            cur_run = 1;
        }
        let same_code = codes.map_or(true, |c| c[i] == c[i - 1]);
        if p[i] > p[i - 1] && same_code {
            cur_lrun += 1;
        } else {
            lrun_lens.push(cur_lrun);
            cur_lrun = 1;
        }
        if p[i] != p[i - 1] + 1 {
            n_sruns += 1;
        }
        if !(p[i] == p[i - 1] + 1 && same_code) {
            n_lsruns += 1;
        }
    }
    run_lens.push(cur_run);
    lrun_lens.push(cur_lrun);

    // Contiguous monotone (asc or desc) runs
    let mut n_mruns = 0usize;
    let mut i = 0;
    while i < n {
        n_mruns += 1;
        if i + 1 == n {
            break;
        }
        if p[i + 1] > p[i] {
            i += 1;
            while i + 1 < n && p[i + 1] > p[i] {
                i += 1;
            }
        } else {
            i += 1;
            while i + 1 < n && p[i + 1] < p[i] {
                i += 1;
            }
        }
        i += 1;
    }

    // Strict +/-1 runs
    let mut n_smruns = 0usize;
    let mut j = 0;
    while j < n {
        n_smruns += 1;
        if j + 1 == n {
            break;
        }
        if p[j + 1] == p[j] + 1 {
            j += 1;
            while j + 1 < n && p[j + 1] == p[j] + 1 {
                j += 1;
            }
        } else if p[j] == p[j + 1] + 1 {
            j += 1;
            while j + 1 < n && p[j] == p[j + 1] + 1 {
                j += 1;
            }
        }
        j += 1;
    }

    // Contiguous 3-gram and 4-gram local relative-order shapes
    let mut k3_hist = [0u64; 6];
    let mut k3_123_homog = 0u64;
    for t in 0..n.saturating_sub(2) {
        let (a, b, d) = (p[t], p[t + 1], p[t + 2]);
        let idx = if a < b {
            if b < d {
                0
            } else if a < d {
                1
            } else {
                3
            }
        } else if a < d {
            2
        } else if b < d {
            4
        } else {
            5
        };
        k3_hist[idx] += 1;
        if idx == 0 && codes.map_or(true, |c| c[t] == c[t + 1] && c[t + 1] == c[t + 2]) {
            k3_123_homog += 1;
        }
    }

    let mut k4_hist = [0u64; 24];
    let mut k4_1234 = 0u64;
    let mut k4_1234_homog = 0u64;
    for t in 0..n.saturating_sub(3) {
        let v = [p[t], p[t + 1], p[t + 2], p[t + 3]];
        // Lehmer code in 0..24
        let mut r0 = 0usize;
        let mut r1 = 0usize;
        let mut r2 = 0usize;
        if v[0] > v[1] {
            r0 += 1;
        }
        if v[0] > v[2] {
            r0 += 1;
        }
        if v[0] > v[3] {
            r0 += 1;
        }
        if v[1] > v[2] {
            r1 += 1;
        }
        if v[1] > v[3] {
            r1 += 1;
        }
        if v[2] > v[3] {
            r2 += 1;
        }
        let code = r0 * 6 + r1 * 2 + r2;
        k4_hist[code] += 1;
        if code == 0 {
            k4_1234 += 1;
            if codes.map_or(true, |c| {
                c[t] == c[t + 1] && c[t + 1] == c[t + 2] && c[t + 2] == c[t + 3]
            }) {
                k4_1234_homog += 1;
            }
        }
    }
    let k4_total = n.saturating_sub(3) as f64;
    let k4_top_share = if k4_total > 0.0 {
        (*k4_hist.iter().max().unwrap() as f64) / k4_total
    } else {
        0.0
    };

    MonotoneStats {
        n,
        n_runs: run_lens.len(),
        n_mruns,
        n_lruns: lrun_lens.len(),
        n_sruns,
        n_smruns,
        n_lsruns,
        h_runs: entropy_of_lengths(&run_lens, n),
        h_lruns: entropy_of_lengths(&lrun_lens, n),
        k3_hist,
        k3_123_homog,
        k4_1234,
        k4_1234_homog,
        k4_top_share,
    }
}

fn invert_perm(p: &[u32]) -> Vec<u32> {
    let mut inv = vec![0u32; p.len()];
    for (i, &v) in p.iter().enumerate() {
        inv[v as usize] = i as u32;
    }
    inv
}

fn raw_lf_from_records(records: &[u16], n: usize) -> (Vec<u32>, usize, usize, usize) {
    let mut counts = [0u32; 256];
    let mut max_runs = 0usize;
    let mut h1 = 0usize;
    maximal(records, |c, _pos, w| {
        counts[c as usize] += w;
        max_runs += 1;
        if w == 1 {
            h1 += 1;
        }
    });
    let sigma = counts.iter().filter(|&&c| c > 0).count();
    let mut cursor = starts(counts);
    let mut lf = vec![0u32; n];
    maximal(records, |c, pos, w| {
        let base = cursor[c as usize];
        for d in 0..w {
            lf[(pos + d) as usize] = base + d;
        }
        cursor[c as usize] = base + w;
    });
    (lf, sigma, max_runs, h1)
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

    fn nav_bytes(&self) -> usize {
        4 * (self.edges.len() + self.extra.len()) + if self.lanes.is_some() { 48 } else { 0 }
    }

    fn dict_bytes(&self) -> usize {
        self.inline16.len() * INLINE + self.lens.len() + self.spelling.len()
    }

    fn rep_bytes(&self) -> usize {
        self.nav_bytes() + self.dict_bytes()
    }
}

/// Family A: Labelled Strict +1 Run / linear-interval rank-indexed return map.
/// Each 64-bit `dir[w]` entry packs `(prefix: u32 << 32) | (bits: u32)` for 32 vertices
/// `[32*w .. 32*w+32)`. Bit 0 of every word is always 1 (capping run length at word
/// boundaries for branchless single-word `lzcnt` + `popcnt` lookup).
#[derive(Clone)]
struct StrictRunReturnMap {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    dir: Vec<u64>,
    head_edges: Vec<u32>,
    head_extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    lanes: Option<Lane4>,
}

impl StrictRunReturnMap {
    fn nav_bytes(&self) -> usize {
        8 * self.dir.len()
            + 4 * (self.head_edges.len() + self.head_extra.len())
            + if self.lanes.is_some() { 48 } else { 0 }
    }

    fn dict_bytes(&self) -> usize {
        self.inline16.len() * INLINE + self.lens.len() + self.spelling.len()
    }

    fn rep_bytes(&self) -> usize {
        self.nav_bytes() + self.dict_bytes()
    }
}

/// Family B: Labelled Ascending-Run Monotone-Block return map.
#[derive(Clone, Copy)]
#[repr(C)]
struct MonoBlock<const B: usize> {
    base: u32,
    code_tag: u32,
    offs: [u16; B],
}

#[derive(Clone)]
struct MonoBlockReturnMap<const B: usize> {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    blocks: Vec<MonoBlock<B>>,
    escape_edges: Vec<u32>,
    escape_extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    lanes: Option<Lane4>,
    pure_blocks: usize,
}

impl<const B: usize> MonoBlockReturnMap<B> {
    fn nav_bytes(&self) -> usize {
        self.blocks.len() * std::mem::size_of::<MonoBlock<B>>()
            + 4 * (self.escape_edges.len() + self.escape_extra.len())
            + if self.lanes.is_some() { 48 } else { 0 }
    }

    fn dict_bytes(&self) -> usize {
        self.inline16.len() * INLINE + self.lens.len() + self.spelling.len()
    }

    fn rep_bytes(&self) -> usize {
        self.nav_bytes() + self.dict_bytes()
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

fn unpack_fwd_graph(rep: &DictReturnMap) -> (Vec<u32>, Vec<u32>) {
    let m = rep.m;
    let mask = (1u32 << rep.bits) - 1;
    let mut next = Vec::with_capacity(m);
    let mut code = Vec::with_capacity(m);
    for j in 0..m {
        let e = rep.edges[j];
        next.push(e & mask);
        if rep.extra.is_empty() {
            code.push(e >> rep.bits);
        } else {
            code.push(rep.extra[j]);
        }
    }
    (next, code)
}

fn compute_lanes_dict(rep: &DictReturnMap) -> Option<Lane4> {
    if rep.m < 64 {
        return None;
    }
    let mask = (1u32 << rep.bits) - 1;
    let mut j = rep.mark;
    let mut steps = 0usize;
    let mut bytes = 0usize;
    let mut seq = Vec::with_capacity(rep.m);
    for _ in 0..rep.m {
        let e = rep.edges[j as usize];
        let c = if rep.extra.is_empty() {
            e >> rep.bits
        } else {
            rep.extra[j as usize]
        } as usize;
        let wlen = rep.word_bytes(c).len();
        seq.push((j, bytes));
        bytes += wlen;
        steps += 1;
        j = e & mask;
        if j == rep.mark || bytes >= rep.n {
            break;
        }
    }
    if bytes != rep.n || steps < 16 {
        return None;
    }
    let q = steps / 4;
    let idx = [0, q, 2 * q, 3 * q];
    let starts = [seq[idx[0]].0, seq[idx[1]].0, seq[idx[2]].0, seq[idx[3]].0];
    let byte_offs = [seq[idx[0]].1, seq[idx[1]].1, seq[idx[2]].1, seq[idx[3]].1];
    let ends = [byte_offs[1], byte_offs[2], byte_offs[3], rep.n];
    let lane_steps = [q, q, q, steps - 3 * q];
    Some(Lane4 {
        starts,
        byte_offs,
        ends,
        steps: lane_steps,
    })
}

fn build_strict_run_from_dict(base: &DictReturnMap) -> StrictRunReturnMap {
    let (next, code) = unpack_fwd_graph(base);
    let m = base.m;
    let bits = base.bits;
    let wide = !base.extra.is_empty();
    let n_words = m.div_ceil(32);
    let mut dir = Vec::with_capacity(n_words);
    let mut head_edges = Vec::with_capacity(m / 2 + 16);
    let mut head_extra = if wide {
        Vec::with_capacity(m / 2 + 16)
    } else {
        Vec::new()
    };

    for w in 0..n_words {
        let start = w * 32;
        let end = (start + 32).min(m);
        let prefix = head_edges.len() as u32;
        let mut word_bits = 0u32;
        for j in start..end {
            let b = j - start;
            let is_head =
                b == 0 || next[j] != next[j - 1].wrapping_add(1) || code[j] != code[j - 1];
            if is_head {
                word_bits |= 1u32 << b;
                if wide {
                    head_edges.push(next[j]);
                    head_extra.push(code[j]);
                } else {
                    head_edges.push((code[j] << bits) | next[j]);
                }
            }
        }
        dir.push((u64::from(prefix) << 32) | u64::from(word_bits));
    }

    StrictRunReturnMap {
        n: base.n,
        m,
        mark: base.mark,
        bits,
        dir,
        head_edges,
        head_extra,
        inline16: base.inline16.clone(),
        lens: base.lens.clone(),
        spelling: base.spelling.clone(),
        lanes: base.lanes,
    }
}

fn build_mono_block_from_dict<const B: usize>(base: &DictReturnMap) -> MonoBlockReturnMap<B> {
    let (next, code) = unpack_fwd_graph(base);
    let m = base.m;
    let bits = base.bits;
    let wide = !base.extra.is_empty();
    let n_blocks = m.div_ceil(B);
    let mut blocks = Vec::with_capacity(n_blocks);
    let mut escape_edges = Vec::new();
    let mut escape_extra = Vec::new();
    let mut pure_blocks = 0usize;

    for bi in 0..n_blocks {
        let start = bi * B;
        let end = (start + B).min(m);
        let len = end - start;
        let c0 = code[start];
        let mut is_pure = len == B;
        if is_pure {
            for t in 1..B {
                if code[start + t] != c0 || next[start + t] <= next[start + t - 1] {
                    is_pure = false;
                    break;
                }
            }
        }
        if is_pure && (next[start + B - 1] - next[start]) <= u16::MAX as u32 {
            let b0 = next[start];
            let mut offs = [0u16; B];
            for t in 0..B {
                offs[t] = (next[start + t] - b0) as u16;
            }
            blocks.push(MonoBlock {
                base: b0,
                code_tag: c0,
                offs,
            });
            pure_blocks += 1;
        } else {
            let eb = escape_edges.len() as u32;
            blocks.push(MonoBlock {
                base: eb,
                code_tag: ESCAPE_FLAG,
                offs: [0; B],
            });
            for j in start..end {
                if wide {
                    escape_edges.push(next[j]);
                    escape_extra.push(code[j]);
                } else {
                    escape_edges.push((code[j] << bits) | next[j]);
                }
            }
            if len < B {
                escape_edges.resize(escape_edges.len() + (B - len), 0);
                if wide {
                    escape_extra.resize(escape_extra.len() + (B - len), 0);
                }
            }
        }
    }

    MonoBlockReturnMap {
        n: base.n,
        m,
        mark: base.mark,
        bits,
        blocks,
        escape_edges,
        escape_extra,
        inline16: base.inline16.clone(),
        lens: base.lens.clone(),
        spelling: base.spelling.clone(),
        lanes: base.lanes,
        pure_blocks,
    }
}

// ---------------------------------------------------------------------------
// Real charged consumers
// ---------------------------------------------------------------------------

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

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn spell_code(
    code: usize,
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    at: &mut usize,
    end: usize,
    out: &mut [u8],
) {
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

    for _ in 0..min_steps {
        let ed0 = edges[j0 as usize];
        let ed1 = edges[j1 as usize];
        let ed2 = edges[j2 as usize];
        let ed3 = edges[j3 as usize];
        let c0 = if extra.is_empty() {
            ed0 >> bits
        } else {
            extra[j0 as usize]
        } as usize;
        let c1 = if extra.is_empty() {
            ed1 >> bits
        } else {
            extra[j1 as usize]
        } as usize;
        let c2 = if extra.is_empty() {
            ed2 >> bits
        } else {
            extra[j2 as usize]
        } as usize;
        let c3 = if extra.is_empty() {
            ed3 >> bits
        } else {
            extra[j3 as usize]
        } as usize;
        j0 = ed0 & mask;
        j1 = ed1 & mask;
        j2 = ed2 & mask;
        j3 = ed3 & mask;
        spell_code(c0, inline16, lens, spelling, &mut a0, e0, out);
        spell_code(c1, inline16, lens, spelling, &mut a1, e1, out);
        spell_code(c2, inline16, lens, spelling, &mut a2, e2, out);
        spell_code(c3, inline16, lens, spelling, &mut a3, e3, out);
    }

    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let cur = rem_j[lane];
            let ed = edges[cur as usize];
            let c = if extra.is_empty() {
                ed >> bits
            } else {
                extra[cur as usize]
            } as usize;
            rem_j[lane] = ed & mask;
            spell_code(
                c,
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

#[inline(always)]
fn lookup_strict_run(
    j: u32,
    dir: &[u64],
    head_edges: &[u32],
    head_extra: &[u32],
    bits: u32,
    mask: u32,
) -> (u32, usize) {
    let dw = dir[(j >> 5) as usize];
    let b = j & 31;
    let masked = (dw as u32) & (u32::MAX >> (31 - b));
    let head_bit = 31 - masked.leading_zeros();
    let offset = b - head_bit;
    let r = (dw >> 32) as usize + (masked.count_ones() as usize) - 1;
    let he = head_edges[r];
    let code = if head_extra.is_empty() {
        he >> bits
    } else {
        head_extra[r]
    } as usize;
    ((he & mask) + offset, code)
}

#[inline(never)]
fn consume_strict_run_fwd(rep: &StrictRunReturnMap, out: &mut Vec<u8>) {
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = rep.mark;
    let mut j = mark;
    let dir = &rep.dir;
    let head_edges = &rep.head_edges;
    let head_extra = &rep.head_extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let spelling = &rep.spelling;

    for _ in 0..rep.m {
        let (next, code) = lookup_strict_run(j, dir, head_edges, head_extra, bits, mask);
        j = next;
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

#[inline(never)]
fn consume_strict_run_4lane(rep: &StrictRunReturnMap, out: &mut Vec<u8>) {
    let Some(l4) = rep.lanes else {
        consume_strict_run_fwd(rep, out);
        return;
    };
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let dir = &rep.dir;
    let head_edges = &rep.head_edges;
    let head_extra = &rep.head_extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
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

    for _ in 0..min_steps {
        let (n0, c0) = lookup_strict_run(j0, dir, head_edges, head_extra, bits, mask);
        let (n1, c1) = lookup_strict_run(j1, dir, head_edges, head_extra, bits, mask);
        let (n2, c2) = lookup_strict_run(j2, dir, head_edges, head_extra, bits, mask);
        let (n3, c3) = lookup_strict_run(j3, dir, head_edges, head_extra, bits, mask);
        j0 = n0;
        j1 = n1;
        j2 = n2;
        j3 = n3;
        spell_code(c0, inline16, lens, spelling, &mut a0, e0, out);
        spell_code(c1, inline16, lens, spelling, &mut a1, e1, out);
        spell_code(c2, inline16, lens, spelling, &mut a2, e2, out);
        spell_code(c3, inline16, lens, spelling, &mut a3, e3, out);
    }
    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let (nx, c) = lookup_strict_run(rem_j[lane], dir, head_edges, head_extra, bits, mask);
            rem_j[lane] = nx;
            spell_code(
                c,
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

#[inline(always)]
fn lookup_mono_block<const B: usize>(
    j: u32,
    blocks: &[MonoBlock<B>],
    esc_edges: &[u32],
    esc_extra: &[u32],
    bits: u32,
    mask: u32,
) -> (u32, usize) {
    let ju = j as usize;
    let bi = ju / B;
    let rem = ju % B;
    let blk = &blocks[bi];
    let ct = blk.code_tag;
    if (ct & ESCAPE_FLAG) == 0 {
        (blk.base + u32::from(blk.offs[rem]), ct as usize)
    } else {
        let idx = blk.base as usize + rem;
        let ed = esc_edges[idx];
        let code = if esc_extra.is_empty() {
            ed >> bits
        } else {
            esc_extra[idx]
        } as usize;
        (ed & mask, code)
    }
}

#[inline(never)]
fn consume_mono_block_fwd<const B: usize>(rep: &MonoBlockReturnMap<B>, out: &mut Vec<u8>) {
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = rep.mark;
    let mut j = mark;
    let blocks = &rep.blocks;
    let esc_edges = &rep.escape_edges;
    let esc_extra = &rep.escape_extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
    let spelling = &rep.spelling;

    for _ in 0..rep.m {
        let (next, code) = lookup_mono_block(j, blocks, esc_edges, esc_extra, bits, mask);
        j = next;
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

#[inline(never)]
fn consume_mono_block_4lane<const B: usize>(rep: &MonoBlockReturnMap<B>, out: &mut Vec<u8>) {
    let Some(l4) = rep.lanes else {
        consume_mono_block_fwd(rep, out);
        return;
    };
    let n = rep.n;
    out.resize(n + INLINE, 0);
    let bits = rep.bits;
    let mask = (1u32 << bits) - 1;
    let blocks = &rep.blocks;
    let esc_edges = &rep.escape_edges;
    let esc_extra = &rep.escape_extra;
    let inline16 = &rep.inline16;
    let lens = &rep.lens;
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

    for _ in 0..min_steps {
        let (n0, c0) = lookup_mono_block(j0, blocks, esc_edges, esc_extra, bits, mask);
        let (n1, c1) = lookup_mono_block(j1, blocks, esc_edges, esc_extra, bits, mask);
        let (n2, c2) = lookup_mono_block(j2, blocks, esc_edges, esc_extra, bits, mask);
        let (n3, c3) = lookup_mono_block(j3, blocks, esc_edges, esc_extra, bits, mask);
        j0 = n0;
        j1 = n1;
        j2 = n2;
        j3 = n3;
        spell_code(c0, inline16, lens, spelling, &mut a0, e0, out);
        spell_code(c1, inline16, lens, spelling, &mut a1, e1, out);
        spell_code(c2, inline16, lens, spelling, &mut a2, e2, out);
        spell_code(c3, inline16, lens, spelling, &mut a3, e3, out);
    }
    let mut rem_j = [j0, j1, j2, j3];
    let mut rem_a = [a0, a1, a2, a3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let (nx, c) = lookup_mono_block(rem_j[lane], blocks, esc_edges, esc_extra, bits, mask);
            rem_j[lane] = nx;
            spell_code(
                c,
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

// ---------------------------------------------------------------------------
// Corpus loading & synthetic exactness suite
// ---------------------------------------------------------------------------

struct BlockInput {
    #[allow(dead_code)]
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

    // 5. Edge cases + exact bbaaa counterexample
    out.push(("synth_edge_n1", block_from_column(b"Z", 0, 7)));
    out.push(("synth_edge_n2_p0", block_from_column(b"ba", 0, 8)));
    out.push(("synth_edge_n2_p1", block_from_column(b"ba", 1, 9)));
    out.push(("synth_edge_const64", block_from_column(&[b'x'; 64], 31, 10)));
    out.push(("synth_bbaaa", block_from_column(b"bbaaa", 0, 11)));

    out
}

// ---------------------------------------------------------------------------
// Command 1: Algorithmic self-verification
// ---------------------------------------------------------------------------

fn next_permutation(a: &mut [u32]) -> bool {
    let n = a.len();
    if n < 2 {
        return false;
    }
    let mut i = n - 2;
    while a[i] >= a[i + 1] {
        if i == 0 {
            return false;
        }
        i -= 1;
    }
    let mut j = n - 1;
    while a[j] <= a[i] {
        j -= 1;
    }
    a.swap(i, j);
    a[i + 1..].reverse();
    true
}

fn cmd_self_check() {
    println!("=== SELF-CHECK: EXHAUSTIVE ALGORITHM VERIFICATION ===");
    let mut seg = LazyMinSeg::new(16);
    let mut sit_checked = 0usize;
    let mut pat3_checked = 0usize;
    let mut sep_counts = [0usize; 8];

    for (n, sep_slot) in sep_counts.iter_mut().enumerate().skip(1) {
        let mut p: Vec<u32> = (0..n as u32).collect();
        loop {
            let (nodes, root) = build_strong_interval_tree(&p, &mut seg);
            let mut got: Vec<(usize, usize)> = nodes.iter().map(|nd| (nd.l, nd.r)).collect();
            got.sort_unstable();
            let want = brute_strong_intervals(&p);
            assert_eq!(got, want, "SIT mismatch on {:?}", p);

            let st = summarize_sit(&nodes, root, None);
            if st.separable {
                *sep_slot += 1;
            }
            // Verify separability == Av(2413, 3142) exhaustively!
            let has_2413 = find_4pattern_witness(&p, [2, 4, 1, 3]).is_some();
            let has_3142 = find_4pattern_witness(&p, [3, 1, 4, 2]).is_some();
            assert_eq!(
                st.separable,
                !has_2413 && !has_3142,
                "Separability vs Av(2413,3142) mismatch on {:?}",
                p
            );
            sit_checked += 1;

            if n <= 6 {
                let fast = count_3patterns(&p);
                let slow = brute_count_3patterns(&p);
                assert_eq!(fast, slow, "3-pattern count mismatch on {:?}", p);
                pat3_checked += 1;
            }
            if !next_permutation(&mut p) {
                break;
            }
        }
    }
    // Large Schroeder numbers (separable permutation counts for n=1..7: 1, 2, 6, 22, 90, 394, 1806)
    assert_eq!(
        &sep_counts[1..=7],
        &[1, 2, 6, 22, 90, 394, 1806],
        "separable counts must equal large Schroeder numbers!"
    );
    println!(
        "  Strong-interval tree verified on {sit_checked}/5913 permutations (S_1..S_7); separable counts = {:?} (exact Schroeder numbers)",
        &sep_counts[1..=7]
    );
    println!("  3-pattern Fenwick counter verified on {pat3_checked}/873 permutations (S_1..S_6)");

    // Exact bbaaa counterexample verification
    let l = b"bbaaa";
    let (lf, sigma, _, _) = raw_lf_from_records(&records_of(l), l.len());
    assert_eq!(sigma, 2);
    assert_eq!(lf, vec![3, 4, 0, 1, 2]);
    let (_, lds_lf) = lis_and_lds(&lf);
    assert_eq!(
        lds_lf.len(),
        2,
        "raw LF of bbaaa avoids 321 (LDS=2 <= sigma=2)"
    );
    assert_eq!(count_3patterns(&lf)[5], 0, "#321(LF) == 0");

    // First return of LF to {0, 1, 2, 3}: 0->3, 1->4->2, 2->0, 3->1 => [3, 2, 0, 1]
    let lf_ret = vec![3u32, 2, 0, 1];
    let psi_ret = invert_perm(&lf_ret); // [2, 3, 1, 0]
    let (_, lds_lf_ret) = lis_and_lds(&lf_ret);
    let (_, lds_psi_ret) = lis_and_lds(&psi_ret);
    assert_eq!(lds_lf_ret.len(), 3, "LF_R = [3,2,0,1] contains 321!");
    assert_eq!(lds_psi_ret.len(), 3, "Psi_R = [2,3,1,0] contains 321!");
    println!(
        "  Exact bbaaa counterexample verified: raw LF={:?} (LDS={}, #321=0), LF_R={:?} (LDS={}, #321={}), Psi_R={:?} (LDS={}, #321={})",
        lf,
        lds_lf.len(),
        lf_ret,
        lds_lf_ret.len(),
        count_3patterns(&lf_ret)[5],
        psi_ret,
        lds_psi_ret.len(),
        count_3patterns(&psi_ret)[5],
    );
}

// ---------------------------------------------------------------------------
// Command 2: Part 1 Exact Structural Census (117 train blocks + controls)
// ---------------------------------------------------------------------------

fn quantiles(xs: &[f64]) -> (f64, f64, f64, f64, f64) {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    let at = |q: f64| -> f64 {
        let idx = ((n - 1) as f64 * q).round() as usize;
        v[idx.min(n - 1)]
    };
    (v[0], at(0.10), at(0.50), at(0.90), v[n - 1])
}

fn pearson_r(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let ax = x - mx;
        let ay = y - my;
        num += ax * ay;
        dx += ax * ax;
        dy += ay * ay;
    }
    if dx == 0.0 || dy == 0.0 {
        0.0
    } else {
        num / (dx * dy).sqrt()
    }
}

fn cmd_census(path: &str) {
    let (admitted, _non_admitted) = load_wikidata(path);
    assert!(admitted.len() >= TRAIN_SPLIT_BLOCKS);
    let train = &admitted[..TRAIN_SPLIT_BLOCKS];
    println!(
        "\n=== PART 1: EXACT STRUCTURAL CENSUS ON {} TRAIN BLOCKS (raw LF vs Open T=Psi_R & T^-1=LF_R) ===",
        train.len()
    );

    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut seg = LazyMinSeg::new(900_000);

    // Collected per-block vectors for quantile reporting
    let mut cov_n = Vec::new();
    let mut cov_m = Vec::new();
    let mut cov_mn = Vec::new();
    let mut cov_sigma = Vec::new();
    let mut cov_runs = Vec::new();
    let mut cov_h1m = Vec::new();
    let mut cov_stripsm = Vec::new();
    let mut cov_bands = Vec::new();

    // Raw LF stats
    let mut lf_nruns = Vec::new();
    let mut lf_nsruns = Vec::new();
    let mut lf_lds = Vec::new();
    let mut lf_lds_over_sigma = Vec::new();
    let mut lf_lis = Vec::new();
    let mut lf_max_prime_share = Vec::new();
    let mut lf_sit_depth = Vec::new();
    let mut lf_lin_leaf_share = Vec::new();
    let mut lf_sep_count = 0usize;
    let mut lf_w2413 = 0usize;
    let mut lf_w3142 = 0usize;
    let mut lf_w4321 = 0usize;
    let mut lf_w1234 = 0usize;
    let mut lf_w2143 = 0usize;

    // Open T = Psi_R stats
    let mut t_nruns = Vec::new();
    let mut t_nruns_ratio = Vec::new();
    let mut t_nmruns = Vec::new();
    let mut t_nlruns = Vec::new();
    let mut t_nlruns_ratio = Vec::new();
    let mut t_h_runs = Vec::new();
    let mut t_h_lruns = Vec::new();
    let mut t_nsruns = Vec::new();
    let mut t_nsruns_ratio = Vec::new();
    let mut t_nsmruns = Vec::new();
    let mut t_nlsruns = Vec::new();
    let mut t_nlsruns_ratio = Vec::new();
    let mut t_inv_nruns_ratio = Vec::new();
    let mut t_lis = Vec::new();
    let mut t_lds = Vec::new();
    let mut t_lds_over_sigma = Vec::new();
    let mut t_lds_gt_sigma = 0usize;
    let mut t_max_prime_share = Vec::new();
    let mut t_prime_nodes = Vec::new();
    let mut t_lin_inc_nodes = Vec::new();
    let mut t_lin_dec_nodes = Vec::new();
    let mut t_sit_depth = Vec::new();
    let mut t_lin_leaf_share = Vec::new();
    let mut t_homog_lin_node_share = Vec::new();
    let mut t_homog_lin_leaf_share = Vec::new();
    let mut t_homog_prime_node_share = Vec::new();
    let mut t_sep_count = 0usize;

    let mut t_pat3_shares: [Vec<f64>; 6] = Default::default();
    let mut t_w2413 = 0usize;
    let mut t_w3142 = 0usize;
    let mut t_w4321 = 0usize;
    let mut t_w1234 = 0usize;
    let mut t_w2143 = 0usize;
    let mut t_k3_123_share = Vec::new();
    let mut t_k3_123_homog_share = Vec::new();
    let mut t_k4_1234_share = Vec::new();
    let mut t_k4_1234_homog_share = Vec::new();
    let mut t_k4_top_share = Vec::new();

    // Example witnesses from block 0 for auditability
    let mut blk0_witnesses = String::new();

    let t_start = Instant::now();
    for (bi, blk) in train.iter().enumerate() {
        // 1. Raw LF
        let (lf, sigma, max_runs, h1) = raw_lf_from_records(&blk.records, blk.n);
        let lf_mono = monotone_stats(&lf, None);
        let (lf_lis_w, lf_lds_w) = lis_and_lds(&lf);
        assert!(
            lf_lds_w.len() <= sigma,
            "raw LF must satisfy LDS <= sigma ({} > {})",
            lf_lds_w.len(),
            sigma
        );
        let (lf_nodes, lf_root) = build_strong_interval_tree(&lf, &mut seg);
        let lf_sit = summarize_sit(&lf_nodes, lf_root, None);
        if lf_sit.separable {
            lf_sep_count += 1;
        }
        if find_4pattern_witness(&lf, [2, 4, 1, 3]).is_some() {
            lf_w2413 += 1;
        }
        if find_4pattern_witness(&lf, [3, 1, 4, 2]).is_some() {
            lf_w3142 += 1;
        }
        if lf_lds_w.len() >= 4 {
            lf_w4321 += 1;
        }
        if lf_lis_w.len() >= 4 {
            lf_w1234 += 1;
        }
        if find_4pattern_witness(&lf, [2, 1, 4, 3]).is_some() {
            lf_w2143 += 1;
        }

        lf_nruns.push(lf_mono.n_runs as f64 / blk.n as f64);
        lf_nsruns.push(lf_mono.n_sruns as f64 / blk.n as f64);
        lf_lds.push(lf_lds_w.len() as f64);
        lf_lds_over_sigma.push(lf_lds_w.len() as f64 / sigma as f64);
        lf_lis.push(lf_lis_w.len() as f64);
        lf_max_prime_share.push(lf_sit.max_prime_arity as f64 / blk.n as f64);
        lf_sit_depth.push(lf_sit.tree_depth as f64);
        lf_lin_leaf_share.push(lf_sit.leaves_in_nontriv_linear as f64 / blk.n as f64);

        // 2. Open retained map T = Psi_R and T^-1 = LF_R
        let (_, m, mark, bits) = builder.construct_timed(
            &blk.records,
            blk.n as u32,
            blk.orig_ptr,
            PROD_LIMIT,
            false,
            &mut scratch,
        );
        let rep = builder.snapshot_dict_rep(blk.n, m, mark, bits);
        let (t_next, t_code) = unpack_fwd_graph(&rep);
        let t_inv = invert_perm(&t_next);

        cov_n.push(blk.n as f64);
        cov_m.push(m as f64);
        cov_mn.push(m as f64 / blk.n as f64);
        cov_sigma.push(sigma as f64);
        cov_runs.push(max_runs as f64);
        cov_h1m.push(h1 as f64 / m as f64);
        cov_stripsm.push(builder.last_ns as f64 / m as f64);
        cov_bands.push(builder.last_bands as f64);

        let t_mono = monotone_stats(&t_next, Some(&t_code));
        let tinv_mono = monotone_stats(&t_inv, None);
        assert!(
            t_mono.n_lruns <= builder.last_bands,
            "nLRuns(T)={} must be <= bands={}",
            t_mono.n_lruns,
            builder.last_bands
        );
        let (t_lis_w, t_lds_w) = lis_and_lds(&t_next);
        if t_lds_w.len() > sigma {
            t_lds_gt_sigma += 1;
        }
        let (t_nodes, t_root) = build_strong_interval_tree(&t_next, &mut seg);
        let t_sit = summarize_sit(&t_nodes, t_root, Some(&t_code));
        if t_sit.separable {
            t_sep_count += 1;
        }

        let p3 = count_3patterns(&t_next);
        let p3_tot = p3.iter().sum::<u128>() as f64;
        for q in 0..6 {
            t_pat3_shares[q].push(p3[q] as f64 / p3_tot);
        }

        let w2413 = find_4pattern_witness(&t_next, [2, 4, 1, 3]);
        let w3142 = find_4pattern_witness(&t_next, [3, 1, 4, 2]);
        let w2143 = find_4pattern_witness(&t_next, [2, 1, 4, 3]);
        if w2413.is_some() {
            t_w2413 += 1;
        }
        if w3142.is_some() {
            t_w3142 += 1;
        }
        if t_lds_w.len() >= 4 {
            t_w4321 += 1;
        }
        if t_lis_w.len() >= 4 {
            t_w1234 += 1;
        }
        if w2143.is_some() {
            t_w2143 += 1;
        }

        if bi == 0 {
            let fmt_w = |w: Option<[usize; 4]>| -> String {
                match w {
                    Some(idx) => format!(
                        "idx={:?} vals=[{},{},{},{}]",
                        idx, t_next[idx[0]], t_next[idx[1]], t_next[idx[2]], t_next[idx[3]]
                    ),
                    None => "none".to_string(),
                }
            };
            blk0_witnesses = format!(
                "block[0] (n={}, m={}, sigma={}, bands={}): 2413: {} | 3142: {} | 2143: {} | LDS={} (4321 prefix idx={:?}, vals=[{},{},{},{}])",
                blk.n,
                m,
                sigma,
                builder.last_bands,
                fmt_w(w2413),
                fmt_w(w3142),
                fmt_w(w2143),
                t_lds_w.len(),
                &t_lds_w[..4.min(t_lds_w.len())],
                t_next[t_lds_w[0]],
                t_next[t_lds_w[1]],
                t_next[t_lds_w[2]],
                t_next[t_lds_w[3]],
            );
        }

        t_nruns.push(t_mono.n_runs as f64);
        t_nruns_ratio.push(t_mono.n_runs as f64 / m as f64);
        t_nmruns.push(t_mono.n_mruns as f64 / m as f64);
        t_nlruns.push(t_mono.n_lruns as f64);
        t_nlruns_ratio.push(t_mono.n_lruns as f64 / m as f64);
        t_h_runs.push(t_mono.h_runs);
        t_h_lruns.push(t_mono.h_lruns);
        t_nsruns.push(t_mono.n_sruns as f64);
        t_nsruns_ratio.push(t_mono.n_sruns as f64 / m as f64);
        t_nsmruns.push(t_mono.n_smruns as f64 / m as f64);
        t_nlsruns.push(t_mono.n_lsruns as f64);
        t_nlsruns_ratio.push(t_mono.n_lsruns as f64 / m as f64);
        t_inv_nruns_ratio.push(tinv_mono.n_runs as f64 / m as f64);
        t_lis.push(t_lis_w.len() as f64);
        t_lds.push(t_lds_w.len() as f64);
        t_lds_over_sigma.push(t_lds_w.len() as f64 / sigma as f64);
        t_max_prime_share.push(t_sit.max_prime_arity as f64 / m as f64);
        t_prime_nodes.push(t_sit.prime_nodes as f64);
        t_lin_inc_nodes.push(t_sit.linear_inc_nodes as f64);
        t_lin_dec_nodes.push(t_sit.linear_dec_nodes as f64);
        t_sit_depth.push(t_sit.tree_depth as f64);
        t_lin_leaf_share.push(t_sit.leaves_in_nontriv_linear as f64 / m as f64);
        t_homog_lin_node_share.push(if t_sit.nontriv_linear_nodes > 0 {
            t_sit.homog_linear_nodes as f64 / t_sit.nontriv_linear_nodes as f64
        } else {
            0.0
        });
        t_homog_lin_leaf_share.push(t_sit.homog_linear_leaves as f64 / m as f64);
        t_homog_prime_node_share.push(if t_sit.nontriv_prime_nodes > 0 {
            t_sit.homog_prime_nodes as f64 / t_sit.nontriv_prime_nodes as f64
        } else {
            0.0
        });

        let k3_tot = m.saturating_sub(2) as f64;
        t_k3_123_share.push(t_mono.k3_hist[0] as f64 / k3_tot);
        t_k3_123_homog_share.push(if t_mono.k3_hist[0] > 0 {
            t_mono.k3_123_homog as f64 / t_mono.k3_hist[0] as f64
        } else {
            0.0
        });
        let k4_tot = m.saturating_sub(3) as f64;
        t_k4_1234_share.push(t_mono.k4_1234 as f64 / k4_tot);
        t_k4_1234_homog_share.push(if t_mono.k4_1234 > 0 {
            t_mono.k4_1234_homog as f64 / t_mono.k4_1234 as f64
        } else {
            0.0
        });
        t_k4_top_share.push(t_mono.k4_top_share);
    }

    println!(
        "Census completed in {:.2}s across {} train blocks.",
        t_start.elapsed().as_secs_f64(),
        train.len()
    );
    println!("Sample exact witnesses: {blk0_witnesses}");

    let print_q = |label: &str, xs: &[f64], prec: usize| {
        let (mn, p10, p50, p90, mx) = quantiles(xs);
        println!(
            "  {:<32} min={:>9.*}  p10={:>9.*}  p50={:>9.*}  p90={:>9.*}  max={:>9.*}",
            label, prec, mn, prec, p10, prec, p50, prec, p90, prec, mx
        );
    };

    println!("\n--- Covariates (117 train blocks) ---");
    print_q("n", &cov_n, 0);
    print_q("m (retained rows)", &cov_m, 0);
    print_q("m / n", &cov_mn, 4);
    print_q("sigma (alphabet size)", &cov_sigma, 0);
    print_q("maximal runs r", &cov_runs, 0);
    print_q("h1 / m", &cov_h1m, 4);
    print_q("strips / m", &cov_stripsm, 4);
    print_q("Open bands", &cov_bands, 0);

    println!("\n--- Raw LF (pi in S_n, exact) ---");
    print_q("LF nRuns / n", &lf_nruns, 5);
    print_q("LF nSRuns / n", &lf_nsruns, 5);
    print_q("LF LIS", &lf_lis, 0);
    print_q("LF LDS (= min nSUS)", &lf_lds, 0);
    print_q("LF LDS / sigma (<= 1 proved)", &lf_lds_over_sigma, 4);
    print_q("LF max_prime_arity / n", &lf_max_prime_share, 5);
    print_q("LF SIT depth", &lf_sit_depth, 0);
    print_q("LF leaves in nontriv linear / n", &lf_lin_leaf_share, 4);
    println!(
        "  LF exact separability: {}/117 | verified witnesses: 2413={}/117, 3142={}/117, 4321={}/117, 1234={}/117, 2143={}/117",
        lf_sep_count, lf_w2413, lf_w3142, lf_w4321, lf_w1234, lf_w2143
    );

    println!("\n--- Open Forward Retained Map T = Psi_R (and T^-1 = LF_R, exact) ---");
    print_q("T nRuns (ascending)", &t_nruns, 0);
    print_q("T nRuns / m", &t_nruns_ratio, 5);
    print_q("T nMRuns / m", &t_nmruns, 5);
    print_q("T nLRuns (labelled asc)", &t_nlruns, 0);
    print_q("T nLRuns / m (<= bands/m)", &t_nlruns_ratio, 5);
    print_q("T H(vRuns) bits", &t_h_runs, 3);
    print_q("T H(vLRuns) bits", &t_h_lruns, 3);
    print_q("T^-1 (LF_R) nRuns / m", &t_inv_nruns_ratio, 5);
    print_q("T nSRuns (strict +1)", &t_nsruns, 0);
    print_q("T nSRuns / m", &t_nsruns_ratio, 4);
    print_q("T nSMRuns / m", &t_nsmruns, 4);
    print_q("T nLSRuns (labelled +1)", &t_nlsruns, 0);
    print_q("T nLSRuns / m", &t_nlsruns_ratio, 4);
    print_q("T LIS", &t_lis, 0);
    print_q("T LDS (= min nSUS)", &t_lds, 0);
    print_q("T LDS / sigma", &t_lds_over_sigma, 4);
    println!("  T blocks with LDS(T) > sigma: {}/117", t_lds_gt_sigma);
    print_q("T max_prime_arity / m", &t_max_prime_share, 4);
    print_q("T prime_nodes", &t_prime_nodes, 0);
    print_q("T linear_inc_nodes", &t_lin_inc_nodes, 0);
    print_q("T linear_dec_nodes", &t_lin_dec_nodes, 0);
    print_q("T SIT depth", &t_sit_depth, 0);
    print_q("T leaves in nontriv lin / m", &t_lin_leaf_share, 4);
    print_q(
        "T homog lin nodes / nontriv lin",
        &t_homog_lin_node_share,
        4,
    );
    print_q("T homog lin leaves / m", &t_homog_lin_leaf_share, 4);
    print_q(
        "T homog prime nodes / nontriv",
        &t_homog_prime_node_share,
        4,
    );
    println!(
        "  T exact separability: {}/117 | verified witnesses: 2413={}/117, 3142={}/117, 4321={}/117, 1234={}/117, 2143={}/117",
        t_sep_count, t_w2413, t_w3142, t_w4321, t_w1234, t_w2143
    );

    println!("\n--- Exact 3-pattern profile of T (share of C(m,3); uniform = 0.1667) ---");
    let p3_names = ["123", "132", "213", "231", "312", "321"];
    for q in 0..6 {
        print_q(&format!("T share({})", p3_names[q]), &t_pat3_shares[q], 5);
    }

    println!("\n--- Local contiguous k-gram shapes of T (uniform 123=0.1667, 1234=0.0417) ---");
    print_q("T local 3-gram 123 share", &t_k3_123_share, 4);
    print_q("T local 123 label-homog share", &t_k3_123_homog_share, 4);
    print_q("T local 4-gram 1234 share", &t_k4_1234_share, 4);
    print_q("T local 1234 label-homog share", &t_k4_1234_homog_share, 4);
    print_q("T local 4-gram top-shape share", &t_k4_top_share, 4);

    println!("\n--- Pearson correlations with block covariates (117 train blocks) ---");
    println!(
        "  corr(m/n, nLSRuns/m)={:+.3} | corr(h1/m, nLSRuns/m)={:+.3} | corr(m/n, nLRuns/m)={:+.3} | corr(strips/m, nLRuns/m)={:+.3}",
        pearson_r(&cov_mn, &t_nlsruns_ratio),
        pearson_r(&cov_h1m, &t_nlsruns_ratio),
        pearson_r(&cov_mn, &t_nlruns_ratio),
        pearson_r(&cov_stripsm, &t_nlruns_ratio),
    );
    println!(
        "  corr(m/n, max_prime/m)={:+.3} | corr(h1/m, max_prime/m)={:+.3} | corr(sigma, LDS(T))={:+.3} | corr(m, LDS(T))={:+.3}",
        pearson_r(&cov_mn, &t_max_prime_share),
        pearson_r(&cov_h1m, &t_max_prime_share),
        pearson_r(&cov_sigma, &t_lds),
        pearson_r(&cov_m, &t_lds),
    );

    // Pre-registered qualification checks
    let (_, _, med_nlsruns, _, _) = quantiles(&t_nlsruns_ratio);
    let (_, _, med_nlruns, _, _) = quantiles(&t_nlruns_ratio);
    let (_, _, med_max_prime, _, _) = quantiles(&t_max_prime_share);
    let (_, _, med_lds, _, _) = quantiles(&t_lds);
    println!("\n--- Pre-registered Train Qualification Gate (select at most 2 families) ---");
    println!(
        "  Family A (Strict +1 Runs):     median nLSRuns/m = {:.4} <= 0.75? {}",
        med_nlsruns,
        if med_nlsruns <= 0.75 {
            "QUALIFIES"
        } else {
            "EXCLUDED"
        }
    );
    println!(
        "  Family B (Ascending Runs):     median nLRuns/m  = {:.5} <= 0.15? {}",
        med_nlruns,
        if med_nlruns <= 0.15 {
            "QUALIFIES"
        } else {
            "EXCLUDED"
        }
    );
    println!(
        "  Family C (Subst Tree / Sep):   sep={}/117, median max_prime/m = {:.4} <= 0.50? {}",
        t_sep_count,
        med_max_prime,
        if t_sep_count == train.len() || med_max_prime <= 0.50 {
            "QUALIFIES"
        } else {
            "STRUCTURALLY EXCLUDED"
        }
    );
    println!(
        "  Family D (Pattern Avoidance):  all 4-pats witnessed (2413={}/117, 3142={}/117, 4321={}/117, 1234={}/117), median LDS = {:.0} <= 8? {}",
        t_w2413,
        t_w3142,
        t_w4321,
        t_w1234,
        med_lds,
        if med_lds <= 8.0 {
            "QUALIFIES"
        } else {
            "STRUCTURALLY EXCLUDED"
        }
    );

    // Also run Part 1 census on the 12 synthetic exactness controls (strictly separate!)
    println!("\n--- Part 1 Census on 12 Synthetic Controls (strictly separate from train) ---");
    for (name, blk) in &build_synthetic_stress_blocks() {
        let (lf, sigma, _, _) = raw_lf_from_records(&blk.records, blk.n);
        let (_, lf_lds_w) = lis_and_lds(&lf);
        let (_, m, mark, bits) = builder.construct_timed(
            &blk.records,
            blk.n as u32,
            blk.orig_ptr,
            PROD_LIMIT,
            false,
            &mut scratch,
        );
        let rep = builder.snapshot_dict_rep(blk.n, m, mark, bits);
        let (t_next, t_code) = unpack_fwd_graph(&rep);
        let t_mono = monotone_stats(&t_next, Some(&t_code));
        let (_, t_lds_w) = lis_and_lds(&t_next);
        let (t_nodes, t_root) = build_strong_interval_tree(&t_next, &mut seg);
        let t_sit = summarize_sit(&t_nodes, t_root, Some(&t_code));
        println!(
            "  {:<24} n={:>4} m={:>4} sig={:>2} | LF_LDS={:>2} | T: nLRuns/m={:.3} nLSRuns/m={:.3} LDS={:>3} maxPrime/m={:.3} sep={}",
            name,
            blk.n,
            m,
            sigma,
            lf_lds_w.len(),
            t_mono.n_lruns as f64 / m as f64,
            t_mono.n_lsruns as f64 / m as f64,
            t_lds_w.len(),
            t_sit.max_prime_arity as f64 / m as f64,
            t_sit.separable
        );
    }
}

// ---------------------------------------------------------------------------
// Command 3: Exactness verification across all arms & controls
// ---------------------------------------------------------------------------

fn verify_all_arms_on_block(
    name: &str,
    blk: &BlockInput,
    builder: &mut OpenBuilder,
    scratch: &mut Vec<u8>,
    out: &mut Vec<u8>,
) {
    let (_, m, mark, bits) = builder.construct_timed(
        &blk.records,
        blk.n as u32,
        blk.orig_ptr,
        PROD_LIMIT,
        true,
        scratch,
    );
    let rep_bidi = builder.snapshot_dict_rep(blk.n, m, mark, bits);

    let (_, m2, mark2, bits2) = builder.construct_timed(
        &blk.records,
        blk.n as u32,
        blk.orig_ptr,
        PROD_LIMIT,
        false,
        scratch,
    );
    let mut rep_fwd = builder.snapshot_dict_rep(blk.n, m2, mark2, bits2);
    rep_fwd.lanes = compute_lanes_dict(&rep_fwd);

    // Controls
    consume_dict_bidi(&rep_bidi, out);
    assert_eq!(out, &blk.walked, "ctrl_shipped_bidi mismatch on {name}");
    consume_dict_fwd(&rep_fwd, out);
    assert_eq!(out, &blk.walked, "ctrl_shipped_fwd mismatch on {name}");
    consume_dict_4lane(&rep_fwd, out);
    assert_eq!(
        out, &blk.walked,
        "ctrl_shipped_fwd_4lane mismatch on {name}"
    );

    // Family A: Strict +1 Run
    let rep_sr = build_strict_run_from_dict(&rep_fwd);
    consume_strict_run_fwd(&rep_sr, out);
    assert_eq!(out, &blk.walked, "P_strict_run_fwd mismatch on {name}");
    consume_strict_run_4lane(&rep_sr, out);
    assert_eq!(out, &blk.walked, "P_strict_run_4lane mismatch on {name}");

    // Family B: MonoBlock B=16, 28, 32 (both from_dict and direct materialisation!)
    macro_rules! check_mb {
        ($B:literal) => {{
            let mb = build_mono_block_from_dict::<$B>(&rep_fwd);
            consume_mono_block_fwd(&mb, out);
            assert_eq!(
                out, &blk.walked,
                "P_mono_block_{} fwd mismatch on {name}",
                $B
            );
            consume_mono_block_4lane(&mb, out);
            assert_eq!(
                out, &blk.walked,
                "P_mono_block_{} 4lane mismatch on {name}",
                $B
            );

            // Verify direct construction from bands+queues produces identical output!
            let mut blocks_buf = Vec::new();
            let mut esc_e = Vec::new();
            let mut esc_x = Vec::new();
            let (_, md, markd, bitsd) = builder.construct_mono_block_direct::<$B>(
                &blk.records,
                blk.n as u32,
                blk.orig_ptr,
                PROD_LIMIT,
                scratch,
                &mut blocks_buf,
                &mut esc_e,
                &mut esc_x,
            );
            let mb_direct = MonoBlockReturnMap::<$B> {
                n: blk.n,
                m: md,
                mark: markd,
                bits: bitsd,
                blocks: blocks_buf,
                escape_edges: esc_e,
                escape_extra: esc_x,
                inline16: builder.inline16.clone(),
                lens: builder.lens.clone(),
                spelling: builder.spelling.clone(),
                lanes: rep_fwd.lanes,
                pure_blocks: 0,
            };
            consume_mono_block_fwd(&mb_direct, out);
            assert_eq!(
                out, &blk.walked,
                "P_mono_block_{} DIRECT mismatch on {name}",
                $B
            );
        }};
    }
    check_mb!(16);
    check_mb!(28);
    check_mb!(32);
}

fn cmd_exactness(bz2_path: Option<&str>) {
    println!("\n=== EXACTNESS VERIFICATION (100% BYTE-IDENTICAL TO bwt::walk) ===");
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut out = Vec::new();

    let synth = build_synthetic_stress_blocks();
    for (name, blk) in &synth {
        verify_all_arms_on_block(name, blk, &mut builder, &mut scratch, &mut out);
    }
    println!(
        "  Synthetic stress suite: {}/{} cases x 14 arms = {} exact",
        synth.len(),
        synth.len(),
        synth.len() * 14
    );

    if let Some(path) = bz2_path {
        let (admitted, non_admitted) = load_wikidata(path);
        for (i, blk) in non_admitted.iter().enumerate() {
            verify_all_arms_on_block(
                &format!("non_admitted[{i}]"),
                blk,
                &mut builder,
                &mut scratch,
                &mut out,
            );
        }
        println!(
            "  Forced non-admitted wikidata controls: {}/{} blocks x 14 arms = {} exact",
            non_admitted.len(),
            non_admitted.len(),
            non_admitted.len() * 14
        );
        for (i, blk) in admitted.iter().enumerate() {
            verify_all_arms_on_block(
                &format!("admitted[{i}]"),
                blk,
                &mut builder,
                &mut scratch,
                &mut out,
            );
        }
        println!(
            "  Production-admitted wikidata blocks: {}/{} blocks x 14 arms = {} exact",
            admitted.len(),
            admitted.len(),
            admitted.len() * 14
        );
    }
}

// ---------------------------------------------------------------------------
// Command 4: Train selection + Held-out benchmark & Pareto / budget ledger
// ---------------------------------------------------------------------------

enum PrebuiltArm {
    DictBidi(Vec<DictReturnMap>),
    DictFwd(Vec<DictReturnMap>),
    DictFwd4Lane(Vec<DictReturnMap>),
    StrictRunFwd(Vec<StrictRunReturnMap>),
    StrictRun4Lane(Vec<StrictRunReturnMap>),
    MonoBlock16Fwd(Vec<MonoBlockReturnMap<16>>),
    MonoBlock16_4Lane(Vec<MonoBlockReturnMap<16>>),
    MonoBlock28Fwd(Vec<MonoBlockReturnMap<28>>),
    MonoBlock28_4Lane(Vec<MonoBlockReturnMap<28>>),
    MonoBlock32Fwd(Vec<MonoBlockReturnMap<32>>),
    MonoBlock32_4Lane(Vec<MonoBlockReturnMap<32>>),
}

struct ArmBundle {
    name: &'static str,
    arm: PrebuiltArm,
    offline_build_ms: f64,
    nav_bytes_total: usize,
    dict_bytes_total: usize,
    rep_bytes_total: usize,
    total_visits: usize,
    dep_loads_per_visit: f64,
    extra_note: String,
}

impl ArmBundle {
    fn len(&self) -> usize {
        match &self.arm {
            PrebuiltArm::DictBidi(v) | PrebuiltArm::DictFwd(v) | PrebuiltArm::DictFwd4Lane(v) => {
                v.len()
            }
            PrebuiltArm::StrictRunFwd(v) | PrebuiltArm::StrictRun4Lane(v) => v.len(),
            PrebuiltArm::MonoBlock16Fwd(v) | PrebuiltArm::MonoBlock16_4Lane(v) => v.len(),
            PrebuiltArm::MonoBlock28Fwd(v) | PrebuiltArm::MonoBlock28_4Lane(v) => v.len(),
            PrebuiltArm::MonoBlock32Fwd(v) | PrebuiltArm::MonoBlock32_4Lane(v) => v.len(),
        }
    }

    #[inline(always)]
    fn consume_one(&self, idx: usize, out: &mut Vec<u8>) {
        match &self.arm {
            PrebuiltArm::DictBidi(v) => consume_dict_bidi(&v[idx], out),
            PrebuiltArm::DictFwd(v) => consume_dict_fwd(&v[idx], out),
            PrebuiltArm::DictFwd4Lane(v) => consume_dict_4lane(&v[idx], out),
            PrebuiltArm::StrictRunFwd(v) => consume_strict_run_fwd(&v[idx], out),
            PrebuiltArm::StrictRun4Lane(v) => consume_strict_run_4lane(&v[idx], out),
            PrebuiltArm::MonoBlock16Fwd(v) => consume_mono_block_fwd(&v[idx], out),
            PrebuiltArm::MonoBlock16_4Lane(v) => consume_mono_block_4lane(&v[idx], out),
            PrebuiltArm::MonoBlock28Fwd(v) => consume_mono_block_fwd(&v[idx], out),
            PrebuiltArm::MonoBlock28_4Lane(v) => consume_mono_block_4lane(&v[idx], out),
            PrebuiltArm::MonoBlock32Fwd(v) => consume_mono_block_fwd(&v[idx], out),
            PrebuiltArm::MonoBlock32_4Lane(v) => consume_mono_block_4lane(&v[idx], out),
        }
    }
}

fn flush_caches(buf: &mut [u8]) {
    let mut acc = 0u8;
    for chunk in buf.chunks_exact_mut(64) {
        chunk[0] = chunk[0].wrapping_add(1);
        acc = acc.wrapping_add(chunk[0]);
    }
    black_box(acc);
}

fn median_min_max(xs: &[f64]) -> (f64, f64, f64) {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let med = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    (med, v[0], *v.last().unwrap())
}

fn measure_warm_pass(arm: &ArmBundle, out: &mut Vec<u8>) -> f64 {
    let mut total_ns = 0u64;
    for i in 0..arm.len() {
        arm.consume_one(i, out);
        black_box(out.as_slice());
        let t0 = Instant::now();
        arm.consume_one(i, out);
        total_ns += t0.elapsed().as_nanos() as u64;
        black_box(out.as_slice());
    }
    total_ns as f64 / 1e6
}

fn measure_cold_pass(arm: &ArmBundle, flush_buf: &mut [u8], out: &mut Vec<u8>) -> f64 {
    flush_caches(flush_buf);
    let t0 = Instant::now();
    for i in 0..arm.len() {
        arm.consume_one(i, out);
        black_box(out.as_slice());
    }
    t0.elapsed().as_nanos() as f64 / 1e6
}

fn build_open_controls(blocks: &[BlockInput]) -> (Vec<DictReturnMap>, Vec<DictReturnMap>) {
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut bidi = Vec::with_capacity(blocks.len());
    let mut fwd = Vec::with_capacity(blocks.len());
    for blk in blocks {
        let (_, m, mark, bits) = builder.construct_timed(
            &blk.records,
            blk.n as u32,
            blk.orig_ptr,
            PROD_LIMIT,
            true,
            &mut scratch,
        );
        bidi.push(builder.snapshot_dict_rep(blk.n, m, mark, bits));

        let (_, m2, mark2, bits2) = builder.construct_timed(
            &blk.records,
            blk.n as u32,
            blk.orig_ptr,
            PROD_LIMIT,
            false,
            &mut scratch,
        );
        let mut r = builder.snapshot_dict_rep(blk.n, m2, mark2, bits2);
        r.lanes = compute_lanes_dict(&r);
        fwd.push(r);
    }
    (bidi, fwd)
}

fn cmd_bench(path: &str, repeats: usize) {
    let (admitted, _non_admitted) = load_wikidata(path);
    let train = &admitted[..TRAIN_SPLIT_BLOCKS];
    let held = &admitted[TRAIN_SPLIT_BLOCKS..];
    let train_rows: usize = train.iter().map(|b| b.n).sum();
    let held_rows: usize = held.iter().map(|b| b.n).sum();
    println!(
        "\n=== PART 2: TRAIN PARAMETER SWEEP ({} blks, {} rows) & HELD-OUT BENCH ({} blks, {} rows, {} repeats) ===",
        train.len(),
        train_rows,
        held.len(),
        held_rows,
        repeats
    );

    let mut out = Vec::with_capacity(1_000_000);
    let mut flush_buf = vec![0u8; COLD_FLUSH_BYTES];

    // 1. Train sweep for Family B block size B in {16, 28, 32} (warm 3-repeat medians)
    let (_, train_fwd) = build_open_controls(train);
    let train_fwd_rep_bytes: usize = train_fwd.iter().map(|r| r.rep_bytes()).sum();
    let train_fwd_visits: usize = train_fwd.iter().map(|r| r.m).sum();
    let train_ctrl = ArmBundle {
        name: "train_ctrl_fwd",
        arm: PrebuiltArm::DictFwd(train_fwd.clone()),
        offline_build_ms: 0.0,
        nav_bytes_total: train_fwd.iter().map(|r| r.nav_bytes()).sum(),
        dict_bytes_total: train_fwd.iter().map(|r| r.dict_bytes()).sum(),
        rep_bytes_total: train_fwd_rep_bytes,
        total_visits: train_fwd_visits,
        dep_loads_per_visit: 1.0,
        extra_note: String::new(),
    };
    let train_sr_reps: Vec<StrictRunReturnMap> =
        train_fwd.iter().map(build_strict_run_from_dict).collect();
    let train_sr = ArmBundle {
        name: "train_P_strict_run",
        nav_bytes_total: train_sr_reps.iter().map(|r| r.nav_bytes()).sum(),
        dict_bytes_total: train_sr_reps.iter().map(|r| r.dict_bytes()).sum(),
        rep_bytes_total: train_sr_reps.iter().map(|r| r.rep_bytes()).sum(),
        total_visits: train_fwd_visits,
        dep_loads_per_visit: 2.0,
        offline_build_ms: 0.0,
        extra_note: String::new(),
        arm: PrebuiltArm::StrictRunFwd(train_sr_reps),
    };

    macro_rules! make_mb_train {
        ($B:literal, $name:literal, $var:ident) => {{
            let t0 = Instant::now();
            let reps: Vec<MonoBlockReturnMap<$B>> = train_fwd
                .iter()
                .map(build_mono_block_from_dict::<$B>)
                .collect();
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let pure: usize = reps.iter().map(|r| r.pure_blocks).sum();
            let tot_b: usize = reps.iter().map(|r| r.blocks.len()).sum();
            ArmBundle {
                name: $name,
                offline_build_ms: ms,
                nav_bytes_total: reps.iter().map(|r| r.nav_bytes()).sum(),
                dict_bytes_total: reps.iter().map(|r| r.dict_bytes()).sum(),
                rep_bytes_total: reps.iter().map(|r| r.rep_bytes()).sum(),
                total_visits: train_fwd_visits,
                dep_loads_per_visit: 1.0 + (1.0 - pure as f64 / tot_b as f64),
                extra_note: format!("pure={:.1}%", 100.0 * pure as f64 / tot_b as f64),
                arm: PrebuiltArm::$var(reps),
            }
        }};
    }
    let tr_mb16 = make_mb_train!(16, "train_P_mono_B16", MonoBlock16Fwd);
    let tr_mb28 = make_mb_train!(28, "train_P_mono_B28", MonoBlock28Fwd);
    let tr_mb32 = make_mb_train!(32, "train_P_mono_B32", MonoBlock32Fwd);

    let train_arms = [train_ctrl, train_sr, tr_mb16, tr_mb28, tr_mb32];
    let mut tr_times: Vec<Vec<f64>> = vec![Vec::new(); train_arms.len()];
    for _ in 0..3 {
        for (ai, arm) in train_arms.iter().enumerate() {
            tr_times[ai].push(measure_warm_pass(arm, &mut out));
        }
    }
    println!("\n--- Train Sweep (117 blocks, 3 interleaved warm repeats) ---");
    let tr_fwd_med = median_min_max(&tr_times[0]).0;
    let mut best_b = 16usize;
    let mut best_b_ms = f64::INFINITY;
    for (ai, arm) in train_arms.iter().enumerate() {
        let (med, mn, mx) = median_min_max(&tr_times[ai]);
        let rep_mib = arm.rep_bytes_total as f64 / (1024.0 * 1024.0);
        let nav_kib = arm.nav_bytes_total as f64 / (1024.0 * train.len() as f64);
        println!(
            "  {:<22} rep={:>6.2} MiB (nav={:>6.1} KiB/b) | warm={:>6.2} ms ({:.2}..{:.2}) ratio={:.3}x | {}",
            arm.name,
            rep_mib,
            nav_kib,
            med,
            mn,
            mx,
            med / tr_fwd_med,
            arm.extra_note
        );
        if ai >= 2 && arm.rep_bytes_total <= train_fwd_rep_bytes && med < best_b_ms {
            best_b_ms = med;
            best_b = match ai {
                2 => 16,
                3 => 28,
                _ => 32,
            };
        }
    }
    println!("  ==> Frozen Family B parameter on train: B = {best_b}");
    drop(train_arms);
    drop(train_fwd);

    // 2. Materialisation benchmark on held-out 116 blocks (5 repeats, separated from consumer)
    println!(
        "\n--- Held-out Materialisation Cost (116 blocks, 5 repeats, separate from consumer) ---"
    );
    let mut builder = OpenBuilder::default();
    let mut scratch = Vec::new();
    let mut mat_bidi_tot = Vec::new();
    let mut mat_bidi_a6c = Vec::new();
    let mut mat_fwd_tot = Vec::new();
    let mut mat_fwd_a6c = Vec::new();
    let mut mat_sr_pass = Vec::new();
    let mut mat_mb_direct_tot = Vec::new();
    let mut mat_mb_direct_a6c = Vec::new();
    let mut blocks16 = Vec::new();
    let mut blocks28 = Vec::new();
    let mut blocks32 = Vec::new();
    let mut esc_e = Vec::new();
    let mut esc_x = Vec::new();

    for _ in 0..5 {
        let mut cb = ConstructNs::default();
        for blk in held {
            let (ns, _, _, _) = builder.construct_timed(
                &blk.records,
                blk.n as u32,
                blk.orig_ptr,
                PROD_LIMIT,
                true,
                &mut scratch,
            );
            cb.add(ns);
        }
        mat_bidi_tot.push(cb.total() as f64 / 1e6);
        mat_bidi_a6c.push(cb.a6c as f64 / 1e6);

        let mut cf = ConstructNs::default();
        let mut sr_ns = 0u64;
        for blk in held {
            let (ns, m, mark, bits) = builder.construct_timed(
                &blk.records,
                blk.n as u32,
                blk.orig_ptr,
                PROD_LIMIT,
                false,
                &mut scratch,
            );
            cf.add(ns);
            // Measure linear O(m) conversion from fwd edges -> StrictRun
            let t0 = Instant::now();
            let mask = (1u32 << bits) - 1;
            let wide = !builder.extra.is_empty();
            let n_words = m.div_ceil(32);
            let mut dir = Vec::with_capacity(n_words);
            let mut he = Vec::with_capacity(m / 2 + 16);
            let mut prev_next = 0u32;
            let mut prev_code = 0u32;
            for w in 0..n_words {
                let start = w * 32;
                let end = (start + 32).min(m);
                let pref = he.len() as u32;
                let mut wb = 0u32;
                for j in start..end {
                    let b = j - start;
                    let e = builder.edges[j];
                    let nx = e & mask;
                    let c = if wide { builder.extra[j] } else { e >> bits };
                    if b == 0 || nx != prev_next.wrapping_add(1) || c != prev_code {
                        wb |= 1u32 << b;
                        he.push(e);
                    }
                    prev_next = nx;
                    prev_code = c;
                }
                dir.push((u64::from(pref) << 32) | u64::from(wb));
            }
            sr_ns += t0.elapsed().as_nanos() as u64;
            black_box((dir.len(), he.len(), mark));
        }
        mat_fwd_tot.push(cf.total() as f64 / 1e6);
        mat_fwd_a6c.push(cf.a6c as f64 / 1e6);
        mat_sr_pass.push(sr_ns as f64 / 1e6);

        let mut cm = ConstructNs::default();
        for blk in held {
            let ns = match best_b {
                16 => {
                    builder
                        .construct_mono_block_direct::<16>(
                            &blk.records,
                            blk.n as u32,
                            blk.orig_ptr,
                            PROD_LIMIT,
                            &mut scratch,
                            &mut blocks16,
                            &mut esc_e,
                            &mut esc_x,
                        )
                        .0
                }
                28 => {
                    builder
                        .construct_mono_block_direct::<28>(
                            &blk.records,
                            blk.n as u32,
                            blk.orig_ptr,
                            PROD_LIMIT,
                            &mut scratch,
                            &mut blocks28,
                            &mut esc_e,
                            &mut esc_x,
                        )
                        .0
                }
                _ => {
                    builder
                        .construct_mono_block_direct::<32>(
                            &blk.records,
                            blk.n as u32,
                            blk.orig_ptr,
                            PROD_LIMIT,
                            &mut scratch,
                            &mut blocks32,
                            &mut esc_e,
                            &mut esc_x,
                        )
                        .0
                }
            };
            cm.add(ns);
        }
        mat_mb_direct_tot.push(cm.total() as f64 / 1e6);
        mat_mb_direct_a6c.push(cm.a6c as f64 / 1e6);
    }

    let (mb_med, mb_mn, mb_mx) = median_min_max(&mat_bidi_tot);
    let mb_a6c = median_min_max(&mat_bidi_a6c).0;
    let (mf_med, mf_mn, mf_mx) = median_min_max(&mat_fwd_tot);
    let mf_a6c = median_min_max(&mat_fwd_a6c).0;
    let (msr_med, msr_mn, msr_mx) = median_min_max(&mat_sr_pass);
    let (mmb_med, mmb_mn, mmb_mx) = median_min_max(&mat_mb_direct_tot);
    let mmb_a6c = median_min_max(&mat_mb_direct_a6c).0;

    let i9_c_fwd_proj = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS * (mf_a6c / mb_a6c);
    let i9_c_mb_proj = I9_A6A_MS + I9_A6B_MS + I9_A6C_MS * (mmb_a6c / mb_a6c);
    let i9_c_sr_proj = i9_c_fwd_proj + I9_A6C_MS * (msr_med / mb_a6c);

    println!(
        "  mat_shipped_bidi (A6a+b+c_xor):   {:.2} ms ({:.2}..{:.2}) | A6c={:.2} ms | i9_C=577.7 ms (anchor)",
        mb_med, mb_mn, mb_mx, mb_a6c
    );
    println!(
        "  mat_shipped_fwd  (A6a+b+c_fwd):   {:.2} ms ({:.2}..{:.2}) | A6c={:.2} ms ({:.3}x) | i9_C_proj={:.1} ms",
        mf_med, mf_mn, mf_mx, mf_a6c, mf_a6c / mb_a6c, i9_c_fwd_proj
    );
    println!(
        "  mat_strict_run   (fwd + O(m)):    +{:.2} ms ({:.2}..{:.2}) over fwd (tot={:.2} ms) | i9_C_proj={:.1} ms",
        msr_med, msr_mn, msr_mx, mf_med + msr_med, i9_c_sr_proj
    );
    println!(
        "  mat_mono_direct  (B={} no edges): {:.2} ms ({:.2}..{:.2}) | A6c={:.2} ms ({:.3}x bidi, {:.3}x fwd) | i9_C_proj={:.1} ms",
        best_b,
        mmb_med,
        mmb_mn,
        mmb_mx,
        mmb_a6c,
        mmb_a6c / mb_a6c,
        mmb_a6c / mf_a6c,
        i9_c_mb_proj
    );

    // 3. Build held-out arms (3 matched controls + selected Family A + selected Family B, 1-lane & 4-lane)
    let t0 = Instant::now();
    let (held_bidi, held_fwd) = build_open_controls(held);
    let open_build_ms = t0.elapsed().as_secs_f64() * 1e3;
    let held_visits: usize = held_fwd.iter().map(|r| r.m).sum();

    // Theoretical Barbay-Navarro Huffman-merge-tree numbers on held-out for literature comparison
    let mut bn_bits_sum = 0.0f64;
    let mut bn_depth_sum = 0.0f64;
    for r in &held_fwd {
        let (nx, cd) = unpack_fwd_graph(r);
        let ms = monotone_stats(&nx, Some(&cd));
        bn_bits_sum +=
            (1.0 + ms.h_lruns) * (r.m as f64) + (ms.n_lruns as f64) * (r.m as f64).log2();
        bn_depth_sum += (1.0 + ms.h_lruns) * (r.m as f64);
    }
    println!(
        "\n  [Reference: classic Barbay-Navarro (2013) Huffman-merge wavelet tree on held-out T]:"
    );
    println!(
        "    Theoretical permutation-only space = {:.1} KiB/blk (excluding codes/dict), avg merge-tree depth = {:.2} dependent select ops/step!",
        bn_bits_sum / (8.0 * 1024.0 * held.len() as f64),
        bn_depth_sum / held_visits as f64
    );

    let t_sr = Instant::now();
    let held_sr: Vec<StrictRunReturnMap> =
        held_fwd.iter().map(build_strict_run_from_dict).collect();
    let sr_build_ms = t_sr.elapsed().as_secs_f64() * 1e3;
    let sr_heads_total: usize = held_sr.iter().map(|r| r.head_edges.len()).sum();

    let mut arms: Vec<ArmBundle> = vec![
        ArmBundle {
            name: "ctrl_shipped_bidi",
            nav_bytes_total: held_bidi.iter().map(|r| r.nav_bytes()).sum(),
            dict_bytes_total: held_bidi.iter().map(|r| r.dict_bytes()).sum(),
            rep_bytes_total: held_bidi.iter().map(|r| r.rep_bytes()).sum(),
            total_visits: held_visits,
            dep_loads_per_visit: 1.0,
            offline_build_ms: open_build_ms * 0.5,
            extra_note: "MLP=2".to_string(),
            arm: PrebuiltArm::DictBidi(held_bidi),
        },
        ArmBundle {
            name: "ctrl_shipped_fwd",
            nav_bytes_total: held_fwd.iter().map(|r| r.nav_bytes()).sum(),
            dict_bytes_total: held_fwd.iter().map(|r| r.dict_bytes()).sum(),
            rep_bytes_total: held_fwd.iter().map(|r| r.rep_bytes()).sum(),
            total_visits: held_visits,
            dep_loads_per_visit: 1.0,
            offline_build_ms: open_build_ms * 0.5,
            extra_note: "MLP=1".to_string(),
            arm: PrebuiltArm::DictFwd(held_fwd.clone()),
        },
        ArmBundle {
            name: "ctrl_shipped_fwd_4lane",
            nav_bytes_total: held_fwd.iter().map(|r| r.nav_bytes()).sum(),
            dict_bytes_total: held_fwd.iter().map(|r| r.dict_bytes()).sum(),
            rep_bytes_total: held_fwd.iter().map(|r| r.rep_bytes()).sum(),
            total_visits: held_visits,
            dep_loads_per_visit: 1.0,
            offline_build_ms: open_build_ms * 0.5,
            extra_note: "MLP=4".to_string(),
            arm: PrebuiltArm::DictFwd4Lane(held_fwd.clone()),
        },
        ArmBundle {
            name: "P1_strict_run_fwd",
            nav_bytes_total: held_sr.iter().map(|r| r.nav_bytes()).sum(),
            dict_bytes_total: held_sr.iter().map(|r| r.dict_bytes()).sum(),
            rep_bytes_total: held_sr.iter().map(|r| r.rep_bytes()).sum(),
            total_visits: held_visits,
            dep_loads_per_visit: 2.0,
            offline_build_ms: sr_build_ms,
            extra_note: format!(
                "heads/m={:.3} (dir+head_edges)",
                sr_heads_total as f64 / held_visits as f64
            ),
            arm: PrebuiltArm::StrictRunFwd(held_sr.clone()),
        },
        ArmBundle {
            name: "P1_strict_run_4lane",
            nav_bytes_total: held_sr.iter().map(|r| r.nav_bytes()).sum(),
            dict_bytes_total: held_sr.iter().map(|r| r.dict_bytes()).sum(),
            rep_bytes_total: held_sr.iter().map(|r| r.rep_bytes()).sum(),
            total_visits: held_visits,
            dep_loads_per_visit: 2.0,
            offline_build_ms: sr_build_ms,
            extra_note: "MLP=4".to_string(),
            arm: PrebuiltArm::StrictRun4Lane(held_sr),
        },
    ];

    macro_rules! push_mb_held {
        ($B:literal, $n1:literal, $n4:literal, $v1:ident, $v4:ident) => {{
            let t_mb = Instant::now();
            let reps: Vec<MonoBlockReturnMap<$B>> = held_fwd
                .iter()
                .map(build_mono_block_from_dict::<$B>)
                .collect();
            let ms = t_mb.elapsed().as_secs_f64() * 1e3;
            let pure: usize = reps.iter().map(|r| r.pure_blocks).sum();
            let tot_b: usize = reps.iter().map(|r| r.blocks.len()).sum();
            let pure_pct = 100.0 * pure as f64 / tot_b as f64;
            let esc_rate = 1.0 - pure as f64 / tot_b as f64;
            arms.push(ArmBundle {
                name: $n1,
                nav_bytes_total: reps.iter().map(|r| r.nav_bytes()).sum(),
                dict_bytes_total: reps.iter().map(|r| r.dict_bytes()).sum(),
                rep_bytes_total: reps.iter().map(|r| r.rep_bytes()).sum(),
                total_visits: held_visits,
                dep_loads_per_visit: 1.0 + esc_rate,
                offline_build_ms: ms,
                extra_note: format!(
                    "B={} pure={:.1}% esc={:.1}%",
                    $B,
                    pure_pct,
                    100.0 * esc_rate
                ),
                arm: PrebuiltArm::$v1(reps.clone()),
            });
            arms.push(ArmBundle {
                name: $n4,
                nav_bytes_total: reps.iter().map(|r| r.nav_bytes()).sum(),
                dict_bytes_total: reps.iter().map(|r| r.dict_bytes()).sum(),
                rep_bytes_total: reps.iter().map(|r| r.rep_bytes()).sum(),
                total_visits: held_visits,
                dep_loads_per_visit: 1.0 + esc_rate,
                offline_build_ms: ms,
                extra_note: format!("B={} MLP=4", $B),
                arm: PrebuiltArm::$v4(reps),
            });
        }};
    }

    match best_b {
        16 => push_mb_held!(
            16,
            "P2_mono_B16_fwd",
            "P2_mono_B16_4lane",
            MonoBlock16Fwd,
            MonoBlock16_4Lane
        ),
        28 => push_mb_held!(
            28,
            "P2_mono_B28_fwd",
            "P2_mono_B28_4lane",
            MonoBlock28Fwd,
            MonoBlock28_4Lane
        ),
        _ => push_mb_held!(
            32,
            "P2_mono_B32_fwd",
            "P2_mono_B32_4lane",
            MonoBlock32Fwd,
            MonoBlock32_4Lane
        ),
    }

    // Print held-out structural & working-set ledger
    println!(
        "\n--- Held-Out Representation & Working-Set Ledger (116 blocks, 104,398,845 bytes) ---"
    );
    let nb = held.len() as f64;
    for arm in &arms {
        println!(
            "  {:<24} nav={:>6.1} KiB/b  dict={:>5.1} KiB/b  rep={:>6.1} KiB/b ({:.2}x fwd)  dep/vis={:.2}  build={:>6.1} ms  [{}]",
            arm.name,
            arm.nav_bytes_total as f64 / (1024.0 * nb),
            arm.dict_bytes_total as f64 / (1024.0 * nb),
            arm.rep_bytes_total as f64 / (1024.0 * nb),
            arm.rep_bytes_total as f64 / arms[1].rep_bytes_total as f64,
            arm.dep_loads_per_visit,
            arm.offline_build_ms,
            arm.extra_note
        );
    }

    // 4. Interleaved repeated timings: Warm & Cold
    let mut warm_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    let mut cold_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];

    for _ in 0..repeats {
        for (ai, arm) in arms.iter().enumerate() {
            warm_ms[ai].push(measure_warm_pass(arm, &mut out));
        }
    }
    for _ in 0..repeats {
        for (ai, arm) in arms.iter().enumerate() {
            cold_ms[ai].push(measure_cold_pass(arm, &mut flush_buf, &mut out));
        }
    }

    let print_table = |title: &str, data: &[Vec<f64>]| {
        println!("\n--- {title} ({repeats} interleaved repeats, held-out 116 blocks) ---");
        println!(
            "  {:<24} {:>7} {:>15} {:>6} {:>6} {:>7} {:>7} {:>7} | {:>6} {:>7} {:>7} {:>7} {:>7} {:>7} {:>6}",
            "arm",
            "med ms",
            "min..max",
            "ns/row",
            "ns/vis",
            "vs bidi",
            "vs fwd",
            "vs 4ln",
            "E_i9",
            "dE_bidi",
            "dE_fwd",
            "Cmax_b",
            "Cmax_f",
            "dC_ext",
            "8w_dE"
        );
        let bidi_med = median_min_max(&data[0]).0;
        let fwd_med = median_min_max(&data[1]).0;
        let ln4_med = median_min_max(&data[2]).0;
        let e_fwd_i9 = I9_E_BASE_MS * (fwd_med / bidi_med);

        for (ai, arm) in arms.iter().enumerate() {
            let (med, mn, mx) = median_min_max(&data[ai]);
            let ns_row = med * 1e6 / held_rows as f64;
            let ns_vis = med * 1e6 / arm.total_visits as f64;
            // Paired per-repeat ratios
            let r_bidi: Vec<f64> = data[ai].iter().zip(&data[0]).map(|(a, b)| a / b).collect();
            let r_fwd: Vec<f64> = data[ai].iter().zip(&data[1]).map(|(a, b)| a / b).collect();
            let r_4ln: Vec<f64> = data[ai].iter().zip(&data[2]).map(|(a, b)| a / b).collect();
            let rb = median_min_max(&r_bidi).0;
            let rf = median_min_max(&r_fwd).0;
            let r4 = median_min_max(&r_4ln).0;

            let e_i9 = I9_E_BASE_MS * (med / bidi_med);
            let de_bidi = I9_E_BASE_MS - e_i9;
            let de_fwd = e_fwd_i9 - e_i9;
            let cmax_b = I9_CE_BASE_MS - e_i9 - I9_GATE_MS;
            let cmax_f = (i9_c_fwd_proj + e_fwd_i9) - e_i9 - I9_GATE_MS;
            let dc_ext = de_fwd - I9_GATE_MS;
            let w8_de = de_fwd * I9_8W_E_SCALE;

            println!(
                "  {:<24} {:>7.2} ({:>6.2}..{:>6.2}) {:>6.3} {:>6.2} {:>6.3}x {:>6.3}x {:>6.3}x | {:>6.1} {:>+7.1} {:>+7.1} {:>7.1} {:>7.1} {:>+7.1} {:>+6.1}",
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
                cmax_b,
                cmax_f,
                dc_ext,
                w8_de
            );
        }
        let _ = ln4_med;
    };

    print_table("WARM (in-situ per-block L1/L2/L3 warm)", &warm_ms);
    print_table("COLD (64 MiB flush before each arm pass)", &cold_ms);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <self_check|census|exact|bench|all> [FILE.bz2] [--repeats N]",
            args[0]
        );
        std::process::exit(1);
    }
    let cmd = args[1].as_str();
    let mut path: Option<String> = None;
    let mut repeats = 7usize;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("valid --repeats");
            i += 2;
        } else if path.is_none() {
            path = Some(args[i].clone());
            i += 1;
        } else {
            i += 1;
        }
    }

    match cmd {
        "self_check" => cmd_self_check(),
        "census" => cmd_census(path.as_deref().unwrap_or("corpora/wikidata.bz2")),
        "exact" => cmd_exactness(path.as_deref()),
        "bench" => cmd_bench(path.as_deref().unwrap_or("corpora/wikidata.bz2"), repeats),
        "all" => {
            let p = path.as_deref().unwrap_or("corpora/wikidata.bz2");
            cmd_self_check();
            cmd_census(p);
            cmd_exactness(Some(p));
            cmd_bench(p, repeats);
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
