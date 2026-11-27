#![forbid(unsafe_code)]
//! Constructive rescue of contracted return-map constructors.
//!
//! Smooth-zone probe for claim `bwt-contraction-constructor-rescue`
//! (`docs/journal/2026-11-27-contraction-rescue.md` and
//! `docs/journal/2026-11-24-algorithm-level/bwt-contraction-rescue/`).
//! Zero edits to `src/`, zero `unsafe`.
//!
//! Usage:
//!   bwt_contraction_rescue exact  [FILE.bz2] [--runs PATH] [--text PATH]
//!   bwt_contraction_rescue train  [FILE.bz2] [--repeats N]
//!   bwt_contraction_rescue bench  [FILE.bz2] [--repeats N] [--runs PATH] [--text PATH]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::hint::black_box;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Production Open constants (mirroring src/open.rs, endpoint.rs, outq.rs)
// ---------------------------------------------------------------------------

const PROD_LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const TWO_PART_LONG: u8 = 254;
const TWO_PART_INL_BASE: u8 = 128; // 128..=140 => 0..=12 inline tail bytes after orig LONG c0
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

// Frozen train / held-out split matching #992, #998, #1013
const TRAIN_SPLIT_BLOCKS: usize = 117;

// Stage-1 i9-10900K reference anchors (wikidata x4, 1 worker)
const I9_WALL_1W_MS: f64 = 1306.0;
const I9_GATE_MS: f64 = I9_WALL_1W_MS * 0.03; // 39.18 ms
const I9_CE_BASE_MS: f64 = 1068.0; // 577.7 C + 490.3 E

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

// ---------------------------------------------------------------------------
// Exact production OutQ16 mirror
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Spill {
    next: Vec<u32>,
    ovf: Vec<[u8; CHUNK_BYTES]>,
    cuts: Vec<[u32; 256]>,
}

struct Heads {
    heads: [u32; 256],
    in_out: u32,
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
        } else if w <= 32 && f + 32 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * CHUNK_BYTES + 2 * f;
            let iota0: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            let iota1: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(16 + i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota0) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            for (d, v) in self.out[at + 32..at + 64].chunks_exact_mut(2).zip(iota1) {
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
    #[allow(clippy::too_many_arguments)]
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
// Exact production Endpoint IET mirror
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

#[inline(always)]
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

#[inline(always)]
fn inline_hash_bytes(entry: &[u8; INLINE], len: u8) -> usize {
    let low = u64::from_le_bytes([
        entry[0], entry[1], entry[2], entry[3], entry[4], entry[5], entry[6], entry[7],
    ]);
    let high = u64::from_le_bytes([
        entry[8], entry[9], entry[10], entry[11], entry[12], entry[13], entry[14], entry[15],
    ]);
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ (u64::from(len)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (h ^ (h >> 30)) as usize
}

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
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
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

fn compute_retained_f_buckets(
    strips_raw: &[u32],
    ns: usize,
    first: &[u32; 256],
    m: u32,
) -> [u32; 257] {
    let mut r_first = [0u32; 257];
    let mut si = 0usize;
    let mut elim_before = 0u32;
    for b in 0..256 {
        let x = first[b];
        while si < ns {
            let s_from = strips_raw[3 * si];
            let s_w = strips_raw[3 * si + 2] >> 8;
            if s_from + s_w <= x {
                elim_before += s_w;
                si += 1;
            } else {
                break;
            }
        }
        let partial = if si < ns {
            let s_from = strips_raw[3 * si];
            x.saturating_sub(s_from)
        } else {
            0
        };
        r_first[b] = x - (elim_before + partial);
    }
    r_first[256] = m;
    r_first
}

#[derive(Default, Clone, Copy)]
struct PhaseNs {
    a6a: u64,
    a6b: u64,
    a6c: u64,
    contract: u64,
    a7: u64,
}

impl PhaseNs {
    fn add(&mut self, o: PhaseNs) {
        self.a6a += o.a6a;
        self.a6b += o.a6b;
        self.a6c += o.a6c;
        self.contract += o.contract;
        self.a7 += o.a7;
    }
    fn total(&self) -> u64 {
        self.a6a + self.a6b + self.a6c + self.contract + self.a7
    }
}

#[derive(Clone, Copy)]
struct Lane4 {
    starts: [u32; 4],
    byte_offs: [usize; 4],
    ends: [usize; 4],
    steps: [usize; 4],
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
    // Reusable scratch for constructors and online 4-lane
    new_edges: Vec<u32>,
    new_extra: Vec<u32>,
    dm_cache: Vec<u32>,
    page_fb: Vec<u8>,
    kept_e0: Vec<u32>,
    lane_bufs: [Vec<u8>; 4],
}

#[inline(always)]
fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

// ---------------------------------------------------------------------------
// Consumers: fwd 1-lane, fwd 1-lane 2-part, bidi, online 4-lane, oracle 4-lane
// ---------------------------------------------------------------------------

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_fwd(
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
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;
    if extra.is_empty() {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
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
    } else {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = extra[j as usize] as usize;
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
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

/// Redesign 1 consumer: identical 1-branch fast path for `len < 128`, with zero-copy 2-part
/// prefix support in the cold `len >= 128` branch (`128..=140`: inline 12B tail, `254`: long tail,
/// `255`: standard `LONG`).
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_fwd_2part(
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
    let mut at = 0usize;
    let mask = (1u32 << bits) - 1;
    let mut j = mark;

    #[inline(always)]
    fn emit_cold(
        code: usize,
        len: u8,
        inline16: &[[u8; INLINE]],
        spelling: &[u8],
        out: &mut [u8],
        at: &mut usize,
        n: usize,
    ) {
        let slot = &inline16[code];
        if len == LONG {
            let word = long_word(slot, spelling);
            let take = word.len().min(n - *at);
            out[*at..*at + take].copy_from_slice(&word[..take]);
            *at += take;
        } else {
            let c0 = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
            let pref = long_word(&inline16[c0], spelling);
            let take = pref.len().min(n - *at);
            out[*at..*at + take].copy_from_slice(&pref[..take]);
            *at += take;
            if *at < n {
                if len != TWO_PART_LONG {
                    let tlen = usize::from(len - TWO_PART_INL_BASE);
                    let ttake = tlen.min(n - *at);
                    out[*at..*at + ttake].copy_from_slice(&slot[4..4 + ttake]);
                    *at += ttake;
                } else {
                    let s = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
                    let t = u32::from_le_bytes([slot[8], slot[9], slot[10], slot[11]]) as usize;
                    let tail = &spelling[s..t];
                    let ttake = tail.len().min(n - *at);
                    out[*at..*at + ttake].copy_from_slice(&tail[..ttake]);
                    *at += ttake;
                }
            }
        }
    }

    if extra.is_empty() {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = (edge >> bits) as usize;
            j = edge & mask;
            let len = lens[code];
            if len < TWO_PART_INL_BASE {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                emit_cold(code, len, inline16, spelling, out, &mut at, n);
            }
            if j == mark || at >= n {
                break;
            }
        }
    } else {
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = extra[j as usize] as usize;
            j = edge & mask;
            let len = lens[code];
            if len < TWO_PART_INL_BASE {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                emit_cold(code, len, inline16, spelling, out, &mut at, n);
            }
            if j == mark || at >= n {
                break;
            }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_bidi(
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
    let mask = (1u32 << bits) - 1;
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_fwd_4lane_online(
    edges: &mut [u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    lane_bufs: &mut [Vec<u8>; 4],
    out: &mut Vec<u8>,
) {
    let m = edges.len();
    let can_tag =
        m >= 256 && extra.is_empty() && (lens.len() - 1) <= ((1u32 << (31 - bits)) - 1) as usize;
    if !can_tag {
        emit_fwd(edges, extra, spelling, inline16, lens, n, mark, bits, out);
        return;
    }
    const K: usize = 16;
    const TAG: u32 = 1u32 << 31;
    let mask = (1u32 << bits) - 1;
    let code_mask = (1u32 << (31 - bits)) - 1;

    let mut anchors = [0u32; K];
    anchors[0] = mark;
    let step = (m / K).max(1);
    let mut count = 1usize;
    let mut cand = (mark as usize + step) % m;
    while count < K {
        if !anchors[..count].contains(&(cand as u32)) {
            anchors[count] = cand as u32;
            count += 1;
        }
        cand = (cand + step + 1) % m;
    }
    for &a in &anchors {
        edges[a as usize] |= TAG;
    }

    let find_anchor = |v: u32| -> usize {
        for (i, &a) in anchors.iter().enumerate() {
            if a == v {
                return i;
            }
        }
        unreachable!("tagged vertex not in anchors")
    };

    let init_cap = (n / 2).max(4096) + INLINE;
    for lb in lane_bufs.iter_mut() {
        if lb.len() < init_cap {
            lb.resize(init_cap, 0);
        }
    }

    let mut seg_meta = [(0usize, 0usize, 0usize, 0usize); K];
    let mut next_anchor = 0usize;
    let mut lane_active = [false; 4];
    let mut lane_anc = [0usize; 4];
    let mut lane_e = [0u32; 4];
    let mut lane_start = [0usize; 4];
    let mut lane_at = [0usize; 4];
    let mut active_cnt = 0usize;

    for l in 0..4 {
        lane_active[l] = true;
        lane_anc[l] = next_anchor;
        lane_e[l] = edges[anchors[next_anchor] as usize];
        lane_start[l] = 0;
        lane_at[l] = 0;
        next_anchor += 1;
        active_cnt += 1;
    }

    #[inline(always)]
    fn write_word(
        c: usize,
        buf: &mut Vec<u8>,
        at: &mut usize,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
    ) {
        let len = lens[c];
        if len != LONG {
            if *at + INLINE > buf.len() {
                buf.resize(*at + INLINE + 16384, 0);
            }
            buf[*at..*at + INLINE].copy_from_slice(&inline16[c]);
            *at += usize::from(len);
        } else {
            let w = long_word(&inline16[c], spelling);
            if *at + w.len() + INLINE > buf.len() {
                buf.resize(*at + w.len() + INLINE + 16384, 0);
            }
            buf[*at..*at + w.len()].copy_from_slice(w);
            *at += w.len();
        }
    }

    while active_cnt > 0 {
        if active_cnt == 4 {
            let (mut e0, mut e1, mut e2, mut e3) = (lane_e[0], lane_e[1], lane_e[2], lane_e[3]);
            let (mut a0, mut a1, mut a2, mut a3) = (lane_at[0], lane_at[1], lane_at[2], lane_at[3]);
            let (b0, rest) = lane_bufs.split_at_mut(1);
            let (b1, rest) = rest.split_at_mut(1);
            let (b2, b3) = rest.split_at_mut(1);
            let (buf0, buf1, buf2, buf3) = (&mut b0[0], &mut b1[0], &mut b2[0], &mut b3[0]);
            loop {
                let c0 = ((e0 >> bits) & code_mask) as usize;
                let c1 = ((e1 >> bits) & code_mask) as usize;
                let c2 = ((e2 >> bits) & code_mask) as usize;
                let c3 = ((e3 >> bits) & code_mask) as usize;
                let v0 = e0 & mask;
                let v1 = e1 & mask;
                let v2 = e2 & mask;
                let v3 = e3 & mask;

                e0 = edges[v0 as usize];
                e1 = edges[v1 as usize];
                e2 = edges[v2 as usize];
                e3 = edges[v3 as usize];

                write_word(c0, buf0, &mut a0, inline16, lens, spelling);
                write_word(c1, buf1, &mut a1, inline16, lens, spelling);
                write_word(c2, buf2, &mut a2, inline16, lens, spelling);
                write_word(c3, buf3, &mut a3, inline16, lens, spelling);

                if ((e0 | e1 | e2 | e3) as i32) < 0 {
                    lane_e = [e0, e1, e2, e3];
                    lane_at = [a0, a1, a2, a3];
                    let vs = [v0, v1, v2, v3];
                    for l in 0..4 {
                        if (lane_e[l] as i32) < 0 {
                            let ai = lane_anc[l];
                            let end_anc = find_anchor(vs[l]);
                            seg_meta[ai] = (l, lane_start[l], lane_at[l] - lane_start[l], end_anc);
                            if next_anchor < K {
                                lane_anc[l] = next_anchor;
                                lane_e[l] = edges[anchors[next_anchor] as usize];
                                lane_start[l] = lane_at[l];
                                next_anchor += 1;
                            } else {
                                lane_active[l] = false;
                                active_cnt -= 1;
                            }
                        }
                    }
                    break;
                }
            }
        } else {
            for l in 0..4 {
                if !lane_active[l] {
                    continue;
                }
                let mut e = lane_e[l];
                let mut a = lane_at[l];
                let buf = &mut lane_bufs[l];
                loop {
                    let c = ((e >> bits) & code_mask) as usize;
                    let v = e & mask;
                    e = edges[v as usize];
                    write_word(c, buf, &mut a, inline16, lens, spelling);
                    if (e as i32) < 0 {
                        let ai = lane_anc[l];
                        let end_anc = find_anchor(v);
                        seg_meta[ai] = (l, lane_start[l], a - lane_start[l], end_anc);
                        lane_at[l] = a;
                        lane_active[l] = false;
                        active_cnt -= 1;
                        break;
                    }
                }
            }
        }
    }

    for &a in &anchors {
        edges[a as usize] &= !TAG;
    }

    out.resize(n, 0);
    let mut pos = 0usize;
    let mut cur = 0usize;
    for _ in 0..K {
        let (l, start, len, nxt) = seg_meta[cur];
        let take = len.min(n - pos);
        out[pos..pos + take].copy_from_slice(&lane_bufs[l][start..start + take]);
        pos += take;
        cur = nxt;
        if cur == 0 || pos >= n {
            break;
        }
    }
    out.truncate(pos.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn compute_oracle_lanes(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
) -> Option<Lane4> {
    let m = edges.len();
    if m < 64 || !extra.is_empty() {
        return None;
    }
    let mask = (1u32 << bits) - 1;
    let mut verts = Vec::with_capacity(m);
    let mut offs = Vec::with_capacity(m);
    let mut j = mark;
    let mut at = 0usize;
    for _ in 0..m {
        verts.push(j);
        offs.push(at);
        let e = edges[j as usize];
        let c = (e >> bits) as usize;
        let len = lens[c];
        let wl = if len != LONG {
            usize::from(len)
        } else {
            long_word(&inline16[c], spelling).len()
        };
        at += wl;
        j = e & mask;
        if j == mark || at >= n {
            break;
        }
    }
    if j != mark || at != n || verts.len() < 64 {
        return None;
    }
    let tot = verts.len();
    let s1 = tot / 4;
    let s2 = tot / 2;
    let s3 = (3 * tot) / 4;
    let starts = [verts[0], verts[s1], verts[s2], verts[s3]];
    let byte_offs = [offs[0], offs[s1], offs[s2], offs[s3]];
    let ends = [byte_offs[1], byte_offs[2], byte_offs[3], n + INLINE];
    let steps = [s1, s2 - s1, s3 - s2, tot - s3];
    Some(Lane4 {
        starts,
        byte_offs,
        ends,
        steps,
    })
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_fwd_4lane_oracle(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    lanes: Option<Lane4>,
    out: &mut Vec<u8>,
) {
    let Some(l4) = lanes else {
        emit_fwd(edges, extra, spelling, inline16, lens, n, mark, bits, out);
        return;
    };
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
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
    fn step_lane(
        out: &mut [u8],
        at: &mut usize,
        end: usize,
        code: usize,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
    ) {
        let len = lens[code];
        if len != LONG && *at + INLINE <= end {
            out[*at..*at + INLINE].copy_from_slice(&inline16[code]);
            *at += usize::from(len);
        } else if len != LONG {
            let l = usize::from(len);
            out[*at..*at + l].copy_from_slice(&inline16[code][..l]);
            *at += l;
        } else {
            let w = long_word(&inline16[code], spelling);
            out[*at..*at + w.len()].copy_from_slice(w);
            *at += w.len();
        }
    }

    for _ in 0..min_steps {
        let x0 = edges[j0 as usize];
        let x1 = edges[j1 as usize];
        let x2 = edges[j2 as usize];
        let x3 = edges[j3 as usize];
        j0 = x0 & mask;
        j1 = x1 & mask;
        j2 = x2 & mask;
        j3 = x3 & mask;
        step_lane(
            out,
            &mut a0,
            e0,
            (x0 >> bits) as usize,
            inline16,
            lens,
            spelling,
        );
        step_lane(
            out,
            &mut a1,
            e1,
            (x1 >> bits) as usize,
            inline16,
            lens,
            spelling,
        );
        step_lane(
            out,
            &mut a2,
            e2,
            (x2 >> bits) as usize,
            inline16,
            lens,
            spelling,
        );
        step_lane(
            out,
            &mut a3,
            e3,
            (x3 >> bits) as usize,
            inline16,
            lens,
            spelling,
        );
    }
    let mut js = [j0, j1, j2, j3];
    let mut ats = [a0, a1, a2, a3];
    let ends = [e0, e1, e2, e3];
    for lane in 0..4 {
        for _ in min_steps..l4.steps[lane] {
            let x = edges[js[lane] as usize];
            js[lane] = x & mask;
            step_lane(
                out,
                &mut ats[lane],
                ends[lane],
                (x >> bits) as usize,
                inline16,
                lens,
                spelling,
            );
        }
    }
    out.truncate(n);
}

// ---------------------------------------------------------------------------
// OpenBuilder: A6a/A6b + Fused A6c Tagging + Redesign 1 & Redesign 2
// ---------------------------------------------------------------------------

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
        r_first_opt: Option<&[u32; 257]>,
    ) -> ([usize; 256], [u32; 256]) {
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

        // Optional fused literal F-bucket scoring during band emission (zero extra pass over bands!)
        let mut fb_lits = [0u32; 256];
        if let Some(r_first) = r_first_opt {
            let n_pages = ((m as usize) >> 8) + 2;
            if self.page_fb.len() < n_pages {
                self.page_fb.resize(n_pages, 0);
            }
            let mut cur_fb = 0usize;
            for pg in 0..n_pages {
                let x = (pg << 8) as u32;
                while cur_fb < 255 && r_first[cur_fb + 1] <= x {
                    cur_fb += 1;
                }
                self.page_fb[pg] = cur_fb as u8;
            }
        }
        let page_fb = &self.page_fb;

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
                if let Some(r_first) = r_first_opt {
                    if code < 256 {
                        let mut d = dst;
                        let dend = dst + width;
                        let mut fb = usize::from(page_fb[(d >> 8) as usize]);
                        while fb < 255 && r_first[fb + 1] <= d {
                            fb += 1;
                        }
                        while d < dend {
                            let take = dend.min(r_first[fb + 1]) - d;
                            fb_lits[fb] += take;
                            d += take;
                            fb += 1;
                        }
                    }
                }
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
        (cursors, fb_lits)
    }

    fn run_a6a_a6b(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        fused_score: bool,
        out: &mut Vec<u8>,
    ) -> (u64, u64, First, Heads, [usize; 256], [u32; 257], [u32; 256]) {
        let t0 = Instant::now();
        out.resize(n as usize + INLINE, 0);
        let mut pass = Pass {
            queues: Queues::new(&mut out[..], &mut self.spill, u32::MAX),
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

        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        let first = starts(all);
        let r_first = if fused_score {
            compute_retained_f_buckets(&self.edges[..3 * ns], ns, &first, f.m)
        } else {
            [0u32; 257]
        };
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
        let rf_opt = if fused_score { Some(&r_first) } else { None };
        let (cursors, fb_lits) = if wide_links {
            self.induced_with::<Wide>(ns, n, f.m, &f.thin, rf_opt)
        } else {
            self.induced_with::<Narrow>(ns, n, f.m, &f.thin, rf_opt)
        };
        let t2 = Instant::now();
        (
            t1.duration_since(t0).as_nanos() as u64,
            t2.duration_since(t1).as_nanos() as u64,
            f,
            heads,
            cursors,
            r_first,
            fb_lits,
        )
    }

    fn wire_forward(
        &mut self,
        m: usize,
        heads: &Heads,
        cursors: &[usize; 256],
        out: &[u8],
    ) -> (u32, bool) {
        let bits = (u32::BITS - (m.saturating_sub(1) as u32).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        let mut k = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = Reader::new(out, &self.spill, heads, a);
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
        (bits, wide)
    }

    /// Select eliminated F-buckets purely from pre-computed `r_first` and fused `fb_lits`
    /// (O(256), < 2 us!). Includes profitability guard (`min_m`, `min_elim_pct`).
    fn pick_elim_buckets(
        m: usize,
        r_first: &[u32; 257],
        fb_lits: &[u32; 256],
        target_elim_pct: f64,
        min_lit_frac: f64,
        min_elim_frac: f64,
    ) -> Option<[bool; 256]> {
        let mut cands = [(0u32, 0u8); 256];
        let mut nc = 0usize;
        for fb in 0..256 {
            let rows = r_first[fb + 1] - r_first[fb];
            if rows > 0 && (fb_lits[fb] as f64) >= (rows as f64) * min_lit_frac {
                cands[nc] = (fb_lits[fb], fb as u8);
                nc += 1;
            }
        }
        cands[..nc].sort_unstable_by_key(|a| std::cmp::Reverse(a.0));

        let mut elim_bucket = [false; 256];
        let target_elim = (m as f64 * target_elim_pct) as u32;
        let max_elim = ((m as f64 * (target_elim_pct + 0.06)).min(m as f64 * 0.92)) as u32;
        let mut elim_rows = 0u32;
        for &(_, fb_u8) in &cands[..nc] {
            let fb = usize::from(fb_u8);
            let rows = r_first[fb + 1] - r_first[fb];
            if elim_rows >= target_elim {
                break;
            }
            if elim_rows + rows <= max_elim || elim_rows == 0 {
                elim_bucket[fb] = true;
                elim_rows += rows;
            }
        }
        if (elim_rows as f64) < (m as f64) * min_elim_frac {
            return None;
        }
        Some(elim_bucket)
    }

    // -----------------------------------------------------------------------
    // Controls
    // -----------------------------------------------------------------------

    fn run_ctrl_bidi(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> PhaseNs {
        let (a6a, a6b, f, heads, cursors, _, _) =
            self.run_a6a_a6b(records, n as u32, p, PROD_LIMIT, false, out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        self.predecessor = None;
        if !wide && m >= 65_536 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = f.mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = Reader::new(out, &self.spill, &heads, a);
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
            self.wire_forward(m, &heads, &cursors, out);
        }
        let t3 = Instant::now();
        if let Some(before) = self.predecessor {
            emit_bidi(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                before,
                bits,
                out,
            );
        } else {
            emit_fwd(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
        }
        let t4 = Instant::now();
        PhaseNs {
            a6a,
            a6b,
            a6c: t3.duration_since(t2).as_nanos() as u64,
            contract: 0,
            a7: t4.duration_since(t3).as_nanos() as u64,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_ctrl_fwd(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        four_lane_online: bool,
        oracle_lanes: Option<Lane4>,
        out: &mut Vec<u8>,
    ) -> PhaseNs {
        let (a6a, a6b, f, heads, cursors, _, _) =
            self.run_a6a_a6b(records, n as u32, p, limit, false, out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let (bits, _wide) = self.wire_forward(m, &heads, &cursors, out);
        let t3 = Instant::now();
        if let Some(l4) = oracle_lanes {
            emit_fwd_4lane_oracle(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                Some(l4),
                out,
            );
        } else if four_lane_online {
            emit_fwd_4lane_online(
                &mut self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                &mut self.lane_bufs,
                out,
            );
        } else {
            emit_fwd(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
        }
        let t4 = Instant::now();
        PhaseNs {
            a6a,
            a6b,
            a6c: t3.duration_since(t2).as_nanos() as u64,
            contract: 0,
            a7: t4.duration_since(t3).as_nanos() as u64,
        }
    }

    fn precompute_oracle_lanes(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Option<Lane4> {
        let (_, _, f, heads, cursors, _, _) =
            self.run_a6a_a6b(records, n as u32, p, PROD_LIMIT, false, out);
        let m = f.m as usize;
        let (bits, _) = self.wire_forward(m, &heads, &cursors, out);
        compute_oracle_lanes(
            &self.edges[..m],
            &self.extra,
            &self.spelling,
            &self.inline16,
            &self.lens,
            n,
            f.mark,
            bits,
        )
    }

    // -----------------------------------------------------------------------
    // Reference #1013 baseline (`fb_elim_75_L16`, unchanged for direct paired comparison)
    // -----------------------------------------------------------------------

    fn run_ref1013_fb75(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> PhaseNs {
        let (a6a, a6b, f, heads, cursors, r_first, fb_lits) =
            self.run_a6a_a6b(records, n as u32, p, PROD_LIMIT, true, out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let (bits, wide) = self.wire_forward(m, &heads, &cursors, out);
        let t3 = Instant::now();
        let can_flag =
            !wide && m >= 2048 && (self.lens.len() - 1) <= ((1u32 << (31 - bits)) - 1) as usize;
        let elim_opt = if can_flag {
            Self::pick_elim_buckets(m, &r_first, &fb_lits, 0.75, 0.70, 1.0 / 6.0)
        } else {
            None
        };
        let Some(elim_bucket) = elim_opt else {
            let t3b = Instant::now();
            emit_fwd(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
            let t4 = Instant::now();
            return PhaseNs {
                a6a,
                a6b,
                a6c: t3.duration_since(t2).as_nanos() as u64,
                contract: t3b.duration_since(t3).as_nanos() as u64,
                a7: t4.duration_since(t3b).as_nanos() as u64,
            };
        };
        const ELIM_BIT: u32 = 1u32 << 31;
        self.kept_e0.clear();
        let edges = &mut self.edges[..m];
        let mark_idx = f.mark as usize;
        let mut new_mark = 0u32;
        let mut mark_in_elim = false;
        let mut next_id = 0u32;
        for fb in 0..256 {
            let lo = r_first[fb] as usize;
            let hi = r_first[fb + 1] as usize;
            if elim_bucket[fb] {
                if (lo..hi).contains(&mark_idx) {
                    mark_in_elim = true;
                }
                for e in &mut edges[lo..hi] {
                    *e |= ELIM_BIT;
                }
            } else {
                if (lo..hi).contains(&mark_idx) {
                    new_mark = next_id + (mark_idx - lo) as u32;
                }
                self.kept_e0.extend_from_slice(&edges[lo..hi]);
                for e in &mut edges[lo..hi] {
                    *e = next_id;
                    next_id += 1;
                }
            }
        }
        if mark_in_elim {
            new_mark = next_id;
            self.kept_e0.push(edges[mark_idx] & !ELIM_BIT);
            edges[mark_idx] = next_id;
            next_id += 1;
        }
        let new_m = next_id as usize;
        let mask = (1u32 << bits) - 1;
        let code_mask = (1u32 << (31 - bits)) - 1;
        let new_bits = (u32::BITS - (new_m.saturating_sub(1) as u32).leading_zeros()).max(1);
        let max_packed_code = (u32::MAX >> new_bits) as usize;
        if self.new_edges.len() < new_m {
            self.new_edges.resize(new_m, 0);
        }
        self.new_extra.clear();
        if self.dm_cache.len() < 1024 {
            self.dm_cache.resize(1024, 0);
        } else {
            self.dm_cache[..1024].fill(0);
        }
        let edges_ro = &self.edges[..m];
        let inline16 = &mut self.inline16;
        let lens = &mut self.lens;
        let spelling = &mut self.spelling;
        let dm_cache = &mut self.dm_cache[..1024];
        let new_edges = &mut self.new_edges[..new_m];
        let new_extra = &mut self.new_extra;
        let mut wide_new = false;
        for (dst_idx, &e0) in self.kept_e0.iter().enumerate() {
            let c0 = (e0 >> bits) & code_mask;
            let e1 = edges_ro[(e0 & mask) as usize];
            let (final_code, nv) = if (e1 as i32) >= 0 {
                (c0, e1)
            } else {
                let l0 = lens[c0 as usize];
                let mut inl_len: usize;
                let mut inl_val: u128 = 0;
                let mut sp_start: usize = 0;
                let mut is_long = false;
                if l0 != LONG {
                    inl_len = usize::from(l0);
                    inl_val = u128::from_le_bytes(inline16[c0 as usize]);
                } else {
                    is_long = true;
                    inl_len = 0;
                    sp_start = spelling.len();
                    let s = u32::from_le_bytes(inline16[c0 as usize][0..4].try_into().unwrap())
                        as usize;
                    let t = u32::from_le_bytes(inline16[c0 as usize][4..8].try_into().unwrap())
                        as usize;
                    spelling.extend_from_within(s..t);
                }
                let mut cur_e = e1;
                let final_nv = loop {
                    let cv = ((cur_e >> bits) & code_mask) as usize;
                    let nxt = cur_e & mask;
                    if cv < 256 {
                        if !is_long {
                            if inl_len < INLINE {
                                inl_val |= (cv as u128) << (8 * inl_len);
                                inl_len += 1;
                            } else {
                                is_long = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&inl_val.to_le_bytes());
                                spelling.push(cv as u8);
                            }
                        } else {
                            spelling.push(cv as u8);
                        }
                    } else {
                        let lv = lens[cv];
                        if lv != LONG {
                            let wl = usize::from(lv);
                            if !is_long && inl_len + wl <= INLINE {
                                inl_val |= u128::from_le_bytes(inline16[cv]) << (8 * inl_len);
                                inl_len += wl;
                            } else {
                                if !is_long {
                                    is_long = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&inl_val.to_le_bytes()[..inl_len]);
                                }
                                spelling.extend_from_slice(&inline16[cv][..wl]);
                            }
                        } else {
                            if !is_long {
                                is_long = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&inl_val.to_le_bytes()[..inl_len]);
                            }
                            let s =
                                u32::from_le_bytes(inline16[cv][0..4].try_into().unwrap()) as usize;
                            let t =
                                u32::from_le_bytes(inline16[cv][4..8].try_into().unwrap()) as usize;
                            spelling.extend_from_within(s..t);
                        }
                    }
                    let ne = edges_ro[nxt as usize];
                    if (ne as i32) >= 0 {
                        break ne;
                    }
                    cur_e = ne;
                };
                let fc = if !is_long {
                    let bytes = inl_val.to_le_bytes();
                    let len = inl_len as u8;
                    let slot = inline_hash_bytes(&bytes, len) & 1023;
                    let old = dm_cache[slot] as usize;
                    if old != 0 && lens[old] == len && inline16[old] == bytes {
                        old as u32
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(bytes);
                        lens.push(len);
                        dm_cache[slot] = code;
                        code
                    }
                } else {
                    let code = lens.len() as u32;
                    inline16.push(long_slot(sp_start as u32, spelling.len() as u32));
                    lens.push(LONG);
                    code
                };
                (fc, final_nv)
            };
            if !wide_new && (final_code as usize) > max_packed_code {
                wide_new = true;
                new_extra.resize(new_m, 0);
                let nm_mask = (1u32 << new_bits) - 1;
                for i in 0..dst_idx {
                    new_extra[i] = new_edges[i] >> new_bits;
                    new_edges[i] &= nm_mask;
                }
            }
            if wide_new {
                new_extra[dst_idx] = final_code;
                new_edges[dst_idx] = nv;
            } else {
                new_edges[dst_idx] = (final_code << new_bits) | nv;
            }
        }
        let ts1 = Instant::now();
        emit_fwd(
            &self.new_edges[..new_m],
            &self.new_extra,
            &self.spelling,
            &self.inline16,
            &self.lens,
            n,
            new_mark,
            new_bits,
            out,
        );
        let t4 = Instant::now();
        PhaseNs {
            a6a,
            a6b,
            a6c: t3.duration_since(t2).as_nanos() as u64,
            contract: ts1.duration_since(t3).as_nanos() as u64,
            a7: t4.duration_since(ts1).as_nanos() as u64,
        }
    }

    // -----------------------------------------------------------------------
    // Helper: fast sequential ELIM_BIT tagging & kept_e0 extraction
    // -----------------------------------------------------------------------
    fn tag_and_extract(
        &mut self,
        m: usize,
        mark: u32,
        r_first: &[u32; 257],
        elim_bucket: &[bool; 256],
    ) -> (usize, u32) {
        const ELIM_BIT: u32 = 1u32 << 31;
        self.kept_e0.clear();
        let edges = &mut self.edges[..m];
        let mark_idx = mark as usize;
        let mut new_mark = 0u32;
        let mut mark_in_elim = false;
        let mut next_id = 0u32;
        for fb in 0..256 {
            let lo = r_first[fb] as usize;
            let hi = r_first[fb + 1] as usize;
            if lo == hi {
                continue;
            }
            if elim_bucket[fb] {
                if (lo..hi).contains(&mark_idx) {
                    mark_in_elim = true;
                }
                for e in &mut edges[lo..hi] {
                    *e |= ELIM_BIT;
                }
            } else {
                if (lo..hi).contains(&mark_idx) {
                    new_mark = next_id + (mark_idx - lo) as u32;
                }
                self.kept_e0.extend_from_slice(&edges[lo..hi]);
                for e in &mut edges[lo..hi] {
                    *e = next_id;
                    next_id += 1;
                }
            }
        }
        if mark_in_elim {
            new_mark = next_id;
            self.kept_e0.push(edges[mark_idx] & !ELIM_BIT);
            edges[mark_idx] = next_id;
            next_id += 1;
        }
        (next_id as usize, new_mark)
    }

    // -----------------------------------------------------------------------
    // ABLATION / REDESIGN 1A: `clean1_dict_2part`
    // - Same clean 1-way loop as #1013 (zero state-machine / function-call overhead!)
    // - Replaces u128 variable shifts with stack [u8; 16] byte store
    // - Replaces 27.3 MiB c0 == LONG spelling recopy with zero-copy 2-part prefix encoding
    // - Expands L1d dm_cache 1024 -> 4096
    // - Pre-construction profitability guard (`min_m`, `min_elim_frac`)
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn run_clean1_dict_2part(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        min_m: usize,
        min_elim_frac: f64,
        out: &mut Vec<u8>,
    ) -> PhaseNs {
        let (a6a, a6b, f, heads, cursors, r_first, fb_lits) =
            self.run_a6a_a6b(records, n as u32, p, limit, true, out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let (bits, wide) = self.wire_forward(m, &heads, &cursors, out);
        let t3 = Instant::now();
        let can_flag =
            !wide && m >= min_m && (self.lens.len() - 1) <= ((1u32 << (31 - bits)) - 1) as usize;
        let elim_opt = if can_flag {
            Self::pick_elim_buckets(m, &r_first, &fb_lits, 0.75, 0.70, min_elim_frac)
        } else {
            None
        };
        let Some(elim_bucket) = elim_opt else {
            emit_fwd(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
            let t4 = Instant::now();
            return PhaseNs {
                a6a,
                a6b,
                a6c: t3.duration_since(t2).as_nanos() as u64,
                contract: 0,
                a7: t4.duration_since(t3).as_nanos() as u64,
            };
        };

        let (new_m, new_mark) = self.tag_and_extract(m, f.mark, &r_first, &elim_bucket);
        let mask = (1u32 << bits) - 1;
        let code_mask = (1u32 << (31 - bits)) - 1;
        let new_bits = (u32::BITS - (new_m.saturating_sub(1) as u32).leading_zeros()).max(1);
        let max_packed_code = (u32::MAX >> new_bits) as usize;

        if self.new_edges.len() < new_m {
            self.new_edges.resize(new_m, 0);
        }
        self.new_extra.clear();
        const DM_SIZE: usize = 4096;
        if self.dm_cache.len() < DM_SIZE {
            self.dm_cache.resize(DM_SIZE, 0);
        }
        self.dm_cache[..DM_SIZE].fill(0);

        let edges_ro = &self.edges[..m];
        let inline16 = &mut self.inline16;
        let lens = &mut self.lens;
        let spelling = &mut self.spelling;
        let dm_cache = &mut self.dm_cache[..DM_SIZE];
        let new_edges = &mut self.new_edges[..new_m];
        let new_extra = &mut self.new_extra;
        let mut wide_new = false;

        for (dst_idx, &e0) in self.kept_e0.iter().enumerate() {
            let c0 = (e0 >> bits) & code_mask;
            let e1 = edges_ro[(e0 & mask) as usize];
            let (final_code, nv) = if (e1 as i32) >= 0 {
                (c0, e1)
            } else {
                let c0_u = c0 as usize;
                let l0 = if c0_u < 256 { 1 } else { lens[c0_u] };
                if l0 != LONG {
                    let mut buf = if c0_u < 256 {
                        let mut b = [0u8; INLINE];
                        b[0] = c0 as u8;
                        b
                    } else {
                        inline16[c0_u]
                    };
                    let mut blen = usize::from(l0);
                    let mut is_long = false;
                    let mut sp_start = 0usize;
                    let mut cur_e = e1;
                    let final_nv = loop {
                        let cv = ((cur_e >> bits) & code_mask) as usize;
                        let nxt = cur_e & mask;
                        let ne = edges_ro[nxt as usize];
                        if cv < 256 {
                            if !is_long {
                                if blen < INLINE {
                                    buf[blen] = cv as u8;
                                    blen += 1;
                                } else {
                                    is_long = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&buf);
                                    spelling.push(cv as u8);
                                }
                            } else {
                                spelling.push(cv as u8);
                            }
                        } else {
                            let lv = lens[cv];
                            if lv != LONG {
                                let wl = usize::from(lv);
                                if !is_long && blen + wl <= INLINE {
                                    buf[blen..blen + wl].copy_from_slice(&inline16[cv][..wl]);
                                    blen += wl;
                                } else {
                                    if !is_long {
                                        is_long = true;
                                        sp_start = spelling.len();
                                        spelling.extend_from_slice(&buf[..blen]);
                                    }
                                    spelling.extend_from_slice(&inline16[cv][..wl]);
                                }
                            } else {
                                if !is_long {
                                    is_long = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&buf[..blen]);
                                }
                                let s = u32::from_le_bytes(inline16[cv][0..4].try_into().unwrap())
                                    as usize;
                                let t = u32::from_le_bytes(inline16[cv][4..8].try_into().unwrap())
                                    as usize;
                                spelling.extend_from_within(s..t);
                            }
                        }
                        if (ne as i32) >= 0 {
                            break ne;
                        }
                        cur_e = ne;
                    };
                    let fc = if !is_long {
                        // Trailing bytes `buf[blen..16]` are already zeroed from `[0; 16]` / `inline16[c0]`!
                        let len = blen as u8;
                        let slot = inline_hash_bytes(&buf, len) & (DM_SIZE - 1);
                        let old = dm_cache[slot] as usize;
                        if old != 0 && lens[old] == len && inline16[old] == buf {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(buf);
                            lens.push(len);
                            dm_cache[slot] = code;
                            code
                        }
                    } else {
                        let code = lens.len() as u32;
                        inline16.push(long_slot(sp_start as u32, spelling.len() as u32));
                        lens.push(LONG);
                        code
                    };
                    (fc, final_nv)
                } else {
                    // Zero-copy 2-part prefix mode! NEVER recopy word(c0)!
                    let mut entry = [0u8; INLINE];
                    entry[0..4].copy_from_slice(&c0.to_le_bytes());
                    let mut tlen = 0usize;
                    let mut is_long_tail = false;
                    let mut sp_start = 0usize;
                    let mut cur_e = e1;
                    let final_nv = loop {
                        let cv = ((cur_e >> bits) & code_mask) as usize;
                        let nxt = cur_e & mask;
                        let ne = edges_ro[nxt as usize];
                        if cv < 256 {
                            if !is_long_tail {
                                if tlen < 12 {
                                    entry[4 + tlen] = cv as u8;
                                    tlen += 1;
                                } else {
                                    is_long_tail = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&entry[4..16]);
                                    spelling.push(cv as u8);
                                }
                            } else {
                                spelling.push(cv as u8);
                            }
                        } else {
                            let lv = lens[cv];
                            if lv != LONG {
                                let wl = usize::from(lv);
                                if !is_long_tail && tlen + wl <= 12 {
                                    entry[4 + tlen..4 + tlen + wl]
                                        .copy_from_slice(&inline16[cv][..wl]);
                                    tlen += wl;
                                } else {
                                    if !is_long_tail {
                                        is_long_tail = true;
                                        sp_start = spelling.len();
                                        spelling.extend_from_slice(&entry[4..4 + tlen]);
                                    }
                                    spelling.extend_from_slice(&inline16[cv][..wl]);
                                }
                            } else {
                                if !is_long_tail {
                                    is_long_tail = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&entry[4..4 + tlen]);
                                }
                                let s = u32::from_le_bytes(inline16[cv][0..4].try_into().unwrap())
                                    as usize;
                                let t = u32::from_le_bytes(inline16[cv][4..8].try_into().unwrap())
                                    as usize;
                                spelling.extend_from_within(s..t);
                            }
                        }
                        if (ne as i32) >= 0 {
                            break ne;
                        }
                        cur_e = ne;
                    };
                    let fc = if !is_long_tail {
                        let len = TWO_PART_INL_BASE + (tlen as u8);
                        let slot = inline_hash_bytes(&entry, len) & (DM_SIZE - 1);
                        let old = dm_cache[slot] as usize;
                        if old != 0 && lens[old] == len && inline16[old] == entry {
                            old as u32
                        } else {
                            let code = lens.len() as u32;
                            inline16.push(entry);
                            lens.push(len);
                            dm_cache[slot] = code;
                            code
                        }
                    } else {
                        entry[4..8].copy_from_slice(&(sp_start as u32).to_le_bytes());
                        entry[8..12].copy_from_slice(&(spelling.len() as u32).to_le_bytes());
                        let code = lens.len() as u32;
                        inline16.push(entry);
                        lens.push(TWO_PART_LONG);
                        code
                    };
                    (fc, final_nv)
                }
            };
            if !wide_new && (final_code as usize) > max_packed_code {
                wide_new = true;
                new_extra.resize(new_m, 0);
                let nm_mask = (1u32 << new_bits) - 1;
                for i in 0..dst_idx {
                    new_extra[i] = new_edges[i] >> new_bits;
                    new_edges[i] &= nm_mask;
                }
            }
            if wide_new {
                new_extra[dst_idx] = final_code;
                new_edges[dst_idx] = nv;
            } else {
                new_edges[dst_idx] = (final_code << new_bits) | nv;
            }
        }

        let ts1 = Instant::now();
        emit_fwd_2part(
            &self.new_edges[..new_m],
            &self.new_extra,
            &self.spelling,
            &self.inline16,
            &self.lens,
            n,
            new_mark,
            new_bits,
            out,
        );
        let t4 = Instant::now();
        PhaseNs {
            a6a,
            a6b,
            a6c: t3.duration_since(t2).as_nanos() as u64,
            contract: ts1.duration_since(t3).as_nanos() as u64,
            a7: t4.duration_since(ts1).as_nanos() as u64,
        }
    }

    // -----------------------------------------------------------------------
    // REDESIGN 2: `mlp4_buf_bump_2part` (4-Lane Full-Chain L1d-Buf Chase + Zero-Hash Bump Dict)
    // - 4-lane MLP chase over ALL K->K and K->E chains (not evicted at 8B: 69% of E hops are in
    //   chains >8 hops!) into 4x256B stack buffers (1 KiB total, 100% L1d-resident).
    // - Out-of-line `#[inline(never)]` commit/slow-path helpers keep the 4-lane inner loop
    //   tiny and register-resident (ev0..3, len0..3) without register spilling.
    // - Eliminates ALL `dm_cache` hashing & 16-byte equality comparisons by direct bump-allocating
    //   into pre-reserved `inline16`/`lens` (still compact 4B `new_edges` = 107 KiB/b in L2).
    // - Preserves zero-copy 2-part prefix encoding for `c0 == LONG` (-72.4% spelling writes).
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn run_mlp4_buf_bump_2part(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        min_m: usize,
        min_elim_frac: f64,
        out: &mut Vec<u8>,
    ) -> PhaseNs {
        let (a6a, a6b, f, heads, cursors, r_first, fb_lits) =
            self.run_a6a_a6b(records, n as u32, p, limit, true, out);
        let t2 = Instant::now();
        let m = f.m as usize;
        let (bits, wide) = self.wire_forward(m, &heads, &cursors, out);
        let t3 = Instant::now();
        let can_flag =
            !wide && m >= min_m && (self.lens.len() - 1) <= ((1u32 << (31 - bits)) - 1) as usize;
        let elim_opt = if can_flag {
            Self::pick_elim_buckets(m, &r_first, &fb_lits, 0.75, 0.70, min_elim_frac)
        } else {
            None
        };
        let Some(elim_bucket) = elim_opt else {
            emit_fwd(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
            let t4 = Instant::now();
            return PhaseNs {
                a6a,
                a6b,
                a6c: t3.duration_since(t2).as_nanos() as u64,
                contract: 0,
                a7: t4.duration_since(t3).as_nanos() as u64,
            };
        };

        let (new_m, new_mark) = self.tag_and_extract(m, f.mark, &r_first, &elim_bucket);
        let mask = (1u32 << bits) - 1;
        let code_mask = (1u32 << (31 - bits)) - 1;
        let new_bits = (u32::BITS - (new_m.saturating_sub(1) as u32).leading_zeros()).max(1);
        let max_packed_code = (u32::MAX >> new_bits) as usize;

        if self.new_edges.len() < new_m {
            self.new_edges.resize(new_m, 0);
        }
        self.new_extra.clear();
        let reserve_add = new_m / 2 + 64;
        self.inline16.reserve(reserve_add);
        self.lens.reserve(reserve_add);

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn store_edge_bump(
            dst_idx: usize,
            final_code: u32,
            nv: u32,
            new_m: usize,
            new_bits: u32,
            max_packed_code: usize,
            wide_new: &mut bool,
            new_edges: &mut [u32],
            new_extra: &mut Vec<u32>,
        ) {
            if !*wide_new && (final_code as usize) > max_packed_code {
                *wide_new = true;
                new_extra.resize(new_m, 0);
                let nm_mask = (1u32 << new_bits) - 1;
                for i in 0..new_m {
                    new_extra[i] = new_edges[i] >> new_bits;
                    new_edges[i] &= nm_mask;
                }
            }
            if *wide_new {
                new_extra[dst_idx] = final_code;
                new_edges[dst_idx] = nv;
            } else {
                new_edges[dst_idx] = (final_code << new_bits) | nv;
            }
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn commit_fast_bump(
            dst_idx: usize,
            c0_u32: u32,
            nv: u32,
            tail: &[u8],
            inline16: &mut Vec<[u8; INLINE]>,
            lens: &mut Vec<u8>,
            spelling: &mut Vec<u8>,
            new_m: usize,
            new_bits: u32,
            max_packed_code: usize,
            wide_new: &mut bool,
            new_edges: &mut [u32],
            new_extra: &mut Vec<u32>,
        ) {
            let c0 = c0_u32 as usize;
            let tlen = tail.len();
            let code = lens.len() as u32;
            if c0 < 256 {
                if tlen < INLINE {
                    let mut entry = [0u8; INLINE];
                    entry[0] = c0 as u8;
                    entry[1..1 + tlen].copy_from_slice(tail);
                    inline16.push(entry);
                    lens.push((1 + tlen) as u8);
                } else {
                    let s = spelling.len() as u32;
                    spelling.push(c0 as u8);
                    spelling.extend_from_slice(tail);
                    inline16.push(long_slot(s, spelling.len() as u32));
                    lens.push(LONG);
                }
            } else {
                let l0 = lens[c0];
                if l0 != LONG {
                    let p0 = usize::from(l0);
                    if p0 + tlen <= INLINE {
                        let mut entry = inline16[c0];
                        entry[p0..p0 + tlen].copy_from_slice(tail);
                        inline16.push(entry);
                        lens.push((p0 + tlen) as u8);
                    } else {
                        let s = spelling.len() as u32;
                        spelling.extend_from_slice(&inline16[c0][..p0]);
                        spelling.extend_from_slice(tail);
                        inline16.push(long_slot(s, spelling.len() as u32));
                        lens.push(LONG);
                    }
                } else if tlen <= 12 {
                    let mut entry = [0u8; INLINE];
                    entry[0..4].copy_from_slice(&c0_u32.to_le_bytes());
                    entry[4..4 + tlen].copy_from_slice(tail);
                    inline16.push(entry);
                    lens.push(TWO_PART_INL_BASE + (tlen as u8));
                } else {
                    let s = spelling.len() as u32;
                    spelling.extend_from_slice(tail);
                    let mut entry = [0u8; INLINE];
                    entry[0..4].copy_from_slice(&c0_u32.to_le_bytes());
                    entry[4..8].copy_from_slice(&s.to_le_bytes());
                    entry[8..12].copy_from_slice(&(spelling.len() as u32).to_le_bytes());
                    inline16.push(entry);
                    lens.push(TWO_PART_LONG);
                }
            }
            store_edge_bump(
                dst_idx,
                code,
                nv,
                new_m,
                new_bits,
                max_packed_code,
                wide_new,
                new_edges,
                new_extra,
            );
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish_slow_bump(
            dst_idx: usize,
            c0_u32: u32,
            mut cur_e: u32,
            prefix_tail: &[u8],
            edges_ro: &[u32],
            bits: u32,
            mask: u32,
            code_mask: u32,
            inline16: &mut Vec<[u8; INLINE]>,
            lens: &mut Vec<u8>,
            spelling: &mut Vec<u8>,
            new_m: usize,
            new_bits: u32,
            max_packed_code: usize,
            wide_new: &mut bool,
            new_edges: &mut [u32],
            new_extra: &mut Vec<u32>,
        ) {
            let c0 = c0_u32 as usize;
            let l0 = lens[c0];
            let code = lens.len() as u32;
            let final_nv = if l0 != LONG {
                let p0 = usize::from(l0);
                let mut buf = inline16[c0];
                let mut blen = p0;
                let mut is_long = false;
                let mut sp_start = 0usize;
                if !prefix_tail.is_empty() {
                    if blen + prefix_tail.len() <= INLINE {
                        buf[blen..blen + prefix_tail.len()].copy_from_slice(prefix_tail);
                        blen += prefix_tail.len();
                    } else {
                        is_long = true;
                        sp_start = spelling.len();
                        spelling.extend_from_slice(&buf[..blen]);
                        spelling.extend_from_slice(prefix_tail);
                    }
                }
                let nv = loop {
                    let cv = ((cur_e >> bits) & code_mask) as usize;
                    let ne = edges_ro[(cur_e & mask) as usize];
                    if cv < 256 {
                        if !is_long {
                            if blen < INLINE {
                                buf[blen] = cv as u8;
                                blen += 1;
                            } else {
                                is_long = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&buf);
                                spelling.push(cv as u8);
                            }
                        } else {
                            spelling.push(cv as u8);
                        }
                    } else {
                        let lv = lens[cv];
                        if lv != LONG {
                            let wl = usize::from(lv);
                            if !is_long && blen + wl <= INLINE {
                                buf[blen..blen + wl].copy_from_slice(&inline16[cv][..wl]);
                                blen += wl;
                            } else {
                                if !is_long {
                                    is_long = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&buf[..blen]);
                                }
                                spelling.extend_from_slice(&inline16[cv][..wl]);
                            }
                        } else {
                            if !is_long {
                                is_long = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&buf[..blen]);
                            }
                            let s =
                                u32::from_le_bytes(inline16[cv][0..4].try_into().unwrap()) as usize;
                            let t =
                                u32::from_le_bytes(inline16[cv][4..8].try_into().unwrap()) as usize;
                            spelling.extend_from_within(s..t);
                        }
                    }
                    if (ne as i32) >= 0 {
                        break ne;
                    }
                    cur_e = ne;
                };
                if !is_long {
                    inline16.push(buf);
                    lens.push(blen as u8);
                } else {
                    inline16.push(long_slot(sp_start as u32, spelling.len() as u32));
                    lens.push(LONG);
                }
                nv
            } else {
                let mut entry = [0u8; INLINE];
                entry[0..4].copy_from_slice(&c0_u32.to_le_bytes());
                let mut tlen = 0usize;
                let mut is_long_tail = false;
                let mut sp_start = 0usize;
                if !prefix_tail.is_empty() {
                    if prefix_tail.len() <= 12 {
                        entry[4..4 + prefix_tail.len()].copy_from_slice(prefix_tail);
                        tlen = prefix_tail.len();
                    } else {
                        is_long_tail = true;
                        sp_start = spelling.len();
                        spelling.extend_from_slice(prefix_tail);
                    }
                }
                let nv = loop {
                    let cv = ((cur_e >> bits) & code_mask) as usize;
                    let ne = edges_ro[(cur_e & mask) as usize];
                    if cv < 256 {
                        if !is_long_tail {
                            if tlen < 12 {
                                entry[4 + tlen] = cv as u8;
                                tlen += 1;
                            } else {
                                is_long_tail = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&entry[4..16]);
                                spelling.push(cv as u8);
                            }
                        } else {
                            spelling.push(cv as u8);
                        }
                    } else {
                        let lv = lens[cv];
                        if lv != LONG {
                            let wl = usize::from(lv);
                            if !is_long_tail && tlen + wl <= 12 {
                                entry[4 + tlen..4 + tlen + wl].copy_from_slice(&inline16[cv][..wl]);
                                tlen += wl;
                            } else {
                                if !is_long_tail {
                                    is_long_tail = true;
                                    sp_start = spelling.len();
                                    spelling.extend_from_slice(&entry[4..4 + tlen]);
                                }
                                spelling.extend_from_slice(&inline16[cv][..wl]);
                            }
                        } else {
                            if !is_long_tail {
                                is_long_tail = true;
                                sp_start = spelling.len();
                                spelling.extend_from_slice(&entry[4..4 + tlen]);
                            }
                            let s =
                                u32::from_le_bytes(inline16[cv][0..4].try_into().unwrap()) as usize;
                            let t =
                                u32::from_le_bytes(inline16[cv][4..8].try_into().unwrap()) as usize;
                            spelling.extend_from_within(s..t);
                        }
                    }
                    if (ne as i32) >= 0 {
                        break ne;
                    }
                    cur_e = ne;
                };
                if !is_long_tail {
                    inline16.push(entry);
                    lens.push(TWO_PART_INL_BASE + (tlen as u8));
                } else {
                    entry[4..8].copy_from_slice(&(sp_start as u32).to_le_bytes());
                    entry[8..12].copy_from_slice(&(spelling.len() as u32).to_le_bytes());
                    inline16.push(entry);
                    lens.push(TWO_PART_LONG);
                }
                nv
            };
            store_edge_bump(
                dst_idx,
                code,
                final_nv,
                new_m,
                new_bits,
                max_packed_code,
                wide_new,
                new_edges,
                new_extra,
            );
        }

        let edges_ro = &self.edges[..m];
        let kept_e0 = &self.kept_e0[..new_m];
        let inline16 = &mut self.inline16;
        let lens = &mut self.lens;
        let spelling = &mut self.spelling;
        let new_edges = &mut self.new_edges[..new_m];
        let new_extra = &mut self.new_extra;
        let mut wide_new = false;

        let mut bufs = [[0u8; 256]; 4];
        let mut dsts = [0usize; 4];
        let mut c0s = [0u32; 4];

        if new_m >= 4 {
            let (mut ev0, mut ev1, mut ev2, mut ev3);
            for l in 0..4 {
                let ke = kept_e0[l];
                dsts[l] = l;
                c0s[l] = (ke >> bits) & code_mask;
            }
            ev0 = edges_ro[(kept_e0[0] & mask) as usize];
            ev1 = edges_ro[(kept_e0[1] & mask) as usize];
            ev2 = edges_ro[(kept_e0[2] & mask) as usize];
            ev3 = edges_ro[(kept_e0[3] & mask) as usize];
            let (mut len0, mut len1, mut len2, mut len3) = (0usize, 0usize, 0usize, 0usize);
            let mut active = [true; 4];
            let mut active_cnt = 4usize;
            let mut idx = 4usize;

            macro_rules! step_l4 {
                ($l:expr, $ev:ident, $len:ident) => {
                    if active[$l] {
                        if ($ev as i32) < 0 {
                            let cv = ($ev >> bits) & code_mask;
                            if cv < 256 && $len < 256 {
                                bufs[$l][$len] = cv as u8;
                                $len += 1;
                                $ev = edges_ro[($ev & mask) as usize];
                            } else {
                                finish_slow_bump(
                                    dsts[$l],
                                    c0s[$l],
                                    $ev,
                                    &bufs[$l][..$len],
                                    edges_ro,
                                    bits,
                                    mask,
                                    code_mask,
                                    inline16,
                                    lens,
                                    spelling,
                                    new_m,
                                    new_bits,
                                    max_packed_code,
                                    &mut wide_new,
                                    new_edges,
                                    new_extra,
                                );
                                $len = 0;
                                if idx < new_m {
                                    let ke = kept_e0[idx];
                                    dsts[$l] = idx;
                                    c0s[$l] = (ke >> bits) & code_mask;
                                    $ev = edges_ro[(ke & mask) as usize];
                                    idx += 1;
                                } else {
                                    active[$l] = false;
                                    active_cnt -= 1;
                                }
                            }
                        } else {
                            if $len == 0 {
                                if wide_new {
                                    new_extra[dsts[$l]] = c0s[$l];
                                    new_edges[dsts[$l]] = $ev;
                                } else {
                                    new_edges[dsts[$l]] = (c0s[$l] << new_bits) | $ev;
                                }
                            } else {
                                commit_fast_bump(
                                    dsts[$l],
                                    c0s[$l],
                                    $ev,
                                    &bufs[$l][..$len],
                                    inline16,
                                    lens,
                                    spelling,
                                    new_m,
                                    new_bits,
                                    max_packed_code,
                                    &mut wide_new,
                                    new_edges,
                                    new_extra,
                                );
                                $len = 0;
                            }
                            if idx < new_m {
                                let ke = kept_e0[idx];
                                dsts[$l] = idx;
                                c0s[$l] = (ke >> bits) & code_mask;
                                $ev = edges_ro[(ke & mask) as usize];
                                idx += 1;
                            } else {
                                active[$l] = false;
                                active_cnt -= 1;
                            }
                        }
                    }
                };
            }

            while active_cnt > 0 {
                step_l4!(0, ev0, len0);
                step_l4!(1, ev1, len1);
                step_l4!(2, ev2, len2);
                step_l4!(3, ev3, len3);
            }
        } else {
            for idx in 0..new_m {
                let ke = kept_e0[idx];
                let c0 = (ke >> bits) & code_mask;
                let ev = edges_ro[(ke & mask) as usize];
                if (ev as i32) >= 0 {
                    if wide_new {
                        new_extra[idx] = c0;
                        new_edges[idx] = ev;
                    } else {
                        new_edges[idx] = (c0 << new_bits) | ev;
                    }
                } else {
                    finish_slow_bump(
                        idx,
                        c0,
                        ev,
                        &[],
                        edges_ro,
                        bits,
                        mask,
                        code_mask,
                        inline16,
                        lens,
                        spelling,
                        new_m,
                        new_bits,
                        max_packed_code,
                        &mut wide_new,
                        new_edges,
                        new_extra,
                    );
                }
            }
        }

        let ts1 = Instant::now();
        emit_fwd_2part(
            &self.new_edges[..new_m],
            &self.new_extra,
            &self.spelling,
            &self.inline16,
            &self.lens,
            n,
            new_mark,
            new_bits,
            out,
        );
        let t4 = Instant::now();
        PhaseNs {
            a6a,
            a6b,
            a6c: t3.duration_since(t2).as_nanos() as u64,
            contract: ts1.duration_since(t3).as_nanos() as u64,
            a7: t4.duration_since(ts1).as_nanos() as u64,
        }
    }
}

// ---------------------------------------------------------------------------
// Corpus loading & synthetic exactness suite
// ---------------------------------------------------------------------------

struct BlockData {
    n: usize,
    p: u32,
    records: Vec<u16>,
    walked: Vec<u8>,
    admitted: bool,
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

fn load_bz2_blocks(path: &str, need_walked_all: bool) -> Vec<BlockData> {
    let data = std::fs::read(path).expect("read bz2");
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = TVector::default();
    let mut blocks = Vec::new();
    let mut probe_edges = Vec::new();
    let mut probe_spill = Spill::default();
    let mut probe_out = Vec::new();
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        let p = b.orig_ptr;
        let records = records_of(&ll);
        let admitted = if let Some(cap) = open_cap(records.len(), n, p, false) {
            if cap == Cap::NONE {
                true
            } else {
                probe_out.resize(n + INLINE, 0);
                let mut pass = Pass {
                    queues: Queues::new(&mut probe_out[..], &mut probe_spill, cap.m),
                    edges: &mut probe_edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                maximal(&records, |c, pos, width| {
                    pass.take::<false>(c, pos, width, p, PROD_LIMIT);
                });
                pass.f.m <= cap.m && pass.ns <= cap.strips
            }
        } else {
            false
        };
        let mut walked = Vec::new();
        if (need_walked_all || admitted) && n > 0 && (p as usize) < n {
            build(&ll, &counts, &mut tt);
            walk(&tt, p, &mut walked);
        }
        blocks.push(BlockData {
            n,
            p,
            records,
            walked,
            admitted,
        });
        syms = b.syms;
    }
    blocks
}

fn forward_bwt(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for i in 0..n {
            let ca = t[(a + i) % n];
            let cb = t[(b + i) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let p = rots.iter().position(|&r| r == 0).unwrap() as u32;
    let l: Vec<u8> = rots.iter().map(|&r| t[(r + n - 1) % n]).collect();
    (l, p)
}

fn block_from_column(l: &[u8], p: u32) -> BlockData {
    let mut counts = [0u32; 256];
    for &b in l {
        counts[usize::from(b)] += 1;
    }
    let mut tt = TVector::default();
    build(l, &counts, &mut tt);
    let mut walked = Vec::new();
    walk(&tt, p, &mut walked);
    BlockData {
        n: l.len(),
        p,
        records: records_of(l),
        walked,
        admitted: true,
    }
}

fn make_synthetic_suite() -> Vec<(&'static str, BlockData)> {
    let mut cases = Vec::new();
    let mut dense = Vec::with_capacity(4096);
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..4096 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        dense.push(b'a' + ((x >> 33) % 26) as u8);
    }
    let (l, p) = forward_bwt(&dense);
    cases.push(("dense_4k", block_from_column(&l, p)));

    let (l, p) = forward_bwt(&b"abac".repeat(256));
    cases.push(("periodic_abac256", block_from_column(&l, p)));
    let (l, p) = forward_bwt(&b"aaab".repeat(512));
    cases.push(("periodic_aaab512", block_from_column(&l, p)));
    let (l, p) = forward_bwt(&b"ab".repeat(500));
    cases.push(("periodic_ab500", block_from_column(&l, p)));

    let mut mc64 = Vec::with_capacity(64);
    for i in 0..64u8 {
        mc64.push(b'0' + (i.wrapping_mul(7).wrapping_add(3) % 10));
    }
    cases.push(("multicycle_64", block_from_column(&mc64, 13)));
    let mut mc4k = Vec::with_capacity(4096);
    for i in 0..4096usize {
        mc4k.push(b'0' + ((i.wrapping_mul(17) ^ (i >> 3)) % 16) as u8);
    }
    cases.push(("multicycle_4k", block_from_column(&mc4k, 1234)));

    let mut rh = Vec::with_capacity(4096);
    rh.extend_from_slice(&[b'a'; 400]);
    rh.extend_from_slice(&[b'b'; 300]);
    rh.extend_from_slice(&[b'c'; 250]);
    while rh.len() < 4096 {
        let rem = 4096 - rh.len();
        let s = b"\"Q1234567890-ABCDEF\"";
        rh.extend_from_slice(&s[..rem.min(s.len())]);
    }
    let (l, p) = forward_bwt(&rh);
    cases.push(("run_heavy_4k", block_from_column(&l, p)));

    cases.push(("edge_n1", block_from_column(b"x", 0)));
    cases.push(("edge_n2_p0", block_from_column(b"ab", 0)));
    cases.push(("edge_n2_p1", block_from_column(b"ab", 1)));
    cases.push(("edge_const64", block_from_column(&[b'z'; 64], 17)));
    cases
}

// ---------------------------------------------------------------------------
// Arm Registry
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ArmKind {
    CtrlBidi,
    CtrlFwd1L,
    CtrlFwd4LOnline,
    CtrlFwd4LOracle,
    Ref1013Fb75,
    Clean1Dict2Part {
        limit: u32,
        min_m: usize,
        min_ef: f64,
    },
    Mlp4BufBump2Part {
        limit: u32,
        min_m: usize,
        min_ef: f64,
    },
}

struct ArmSpec {
    name: &'static str,
    kind: ArmKind,
}

fn train_arms() -> Vec<ArmSpec> {
    vec![
        ArmSpec {
            name: "ctrl_shipped_bidi",
            kind: ArmKind::CtrlBidi,
        },
        ArmSpec {
            name: "ctrl_fwd_1lane",
            kind: ArmKind::CtrlFwd1L,
        },
        ArmSpec {
            name: "ctrl_fwd_4lane_onl",
            kind: ArmKind::CtrlFwd4LOnline,
        },
        ArmSpec {
            name: "ctrl_fwd_4lane_ora",
            kind: ArmKind::CtrlFwd4LOracle,
        },
        ArmSpec {
            name: "ref1013_fb75_L16",
            kind: ArmKind::Ref1013Fb75,
        },
        ArmSpec {
            name: "r1_clean1_2part",
            kind: ArmKind::Clean1Dict2Part {
                limit: 16,
                min_m: 32_768,
                min_ef: 0.55,
            },
        },
        ArmSpec {
            name: "r2_mlp4_buf_bump",
            kind: ArmKind::Mlp4BufBump2Part {
                limit: 16,
                min_m: 32_768,
                min_ef: 0.55,
            },
        },
    ]
}

fn eval_arms() -> Vec<ArmSpec> {
    vec![
        ArmSpec {
            name: "ctrl_shipped_bidi",
            kind: ArmKind::CtrlBidi,
        },
        ArmSpec {
            name: "ctrl_fwd_1lane",
            kind: ArmKind::CtrlFwd1L,
        },
        ArmSpec {
            name: "ctrl_fwd_4lane_onl",
            kind: ArmKind::CtrlFwd4LOnline,
        },
        ArmSpec {
            name: "ctrl_fwd_4lane_ora",
            kind: ArmKind::CtrlFwd4LOracle,
        },
        ArmSpec {
            name: "ref1013_fb75_L16",
            kind: ArmKind::Ref1013Fb75,
        },
        ArmSpec {
            name: "r1_clean1_2part",
            kind: ArmKind::Clean1Dict2Part {
                limit: 16,
                min_m: 32_768,
                min_ef: 0.55,
            },
        },
        ArmSpec {
            name: "r2_mlp4_buf_bump",
            kind: ArmKind::Mlp4BufBump2Part {
                limit: 16,
                min_m: 32_768,
                min_ef: 0.55,
            },
        },
    ]
}

fn run_arm(
    builder: &mut OpenBuilder,
    arm: &ArmSpec,
    b: &BlockData,
    l4_fwd: Option<Lane4>,
    out: &mut Vec<u8>,
) -> PhaseNs {
    match arm.kind {
        ArmKind::CtrlBidi => builder.run_ctrl_bidi(&b.records, b.n, b.p, out),
        ArmKind::CtrlFwd1L => {
            builder.run_ctrl_fwd(&b.records, b.n, b.p, PROD_LIMIT, false, None, out)
        }
        ArmKind::CtrlFwd4LOnline => {
            builder.run_ctrl_fwd(&b.records, b.n, b.p, PROD_LIMIT, true, None, out)
        }
        ArmKind::CtrlFwd4LOracle => {
            builder.run_ctrl_fwd(&b.records, b.n, b.p, PROD_LIMIT, false, l4_fwd, out)
        }
        ArmKind::Ref1013Fb75 => builder.run_ref1013_fb75(&b.records, b.n, b.p, out),
        ArmKind::Clean1Dict2Part {
            limit,
            min_m,
            min_ef,
        } => builder.run_clean1_dict_2part(&b.records, b.n, b.p, limit, min_m, min_ef, out),
        ArmKind::Mlp4BufBump2Part {
            limit,
            min_m,
            min_ef,
        } => builder.run_mlp4_buf_bump_2part(&b.records, b.n, b.p, limit, min_m, min_ef, out),
    }
}

fn median_min_max(v: &[f64]) -> (f64, f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (s[s.len() / 2], s[0], s[s.len() - 1])
}

fn verify_exact_set(
    label: &str,
    blocks: &[&BlockData],
    arms: &[ArmSpec],
    builder: &mut OpenBuilder,
) {
    let mut out = Vec::new();
    let mut checked = 0usize;
    for (bi, b) in blocks.iter().enumerate() {
        if b.n == 0 || (b.p as usize) >= b.n {
            continue;
        }
        let l4_fwd = builder.precompute_oracle_lanes(&b.records, b.n, b.p, &mut out);
        for arm in arms {
            run_arm(builder, arm, b, l4_fwd, &mut out);
            assert_eq!(
                out.as_slice(),
                b.walked.as_slice(),
                "EXACTNESS MISMATCH on {} blk={} (n={}, p={}) arm={}",
                label,
                bi,
                b.n,
                b.p,
                arm.name
            );
            checked += 1;
        }
    }
    println!(
        "  EXACTNESS PASS [{}]: {} blocks x {} arms = {} / {} byte-identical to bwt::walk",
        label,
        blocks.len(),
        arms.len(),
        checked,
        checked
    );
}

fn run_exact(path: &str, runs_path: Option<&str>, text_path: Option<&str>) {
    let mut arms = eval_arms();
    arms.push(ArmSpec {
        name: "r1_clean1_2part_small",
        kind: ArmKind::Clean1Dict2Part {
            limit: 16,
            min_m: 4,
            min_ef: 0.05,
        },
    });
    arms.push(ArmSpec {
        name: "r2_mlp4_buf_bump_small",
        kind: ArmKind::Mlp4BufBump2Part {
            limit: 16,
            min_m: 4,
            min_ef: 0.05,
        },
    });

    let mut builder = OpenBuilder::default();
    println!("=== Full Exactness Suite ===");

    let synth = make_synthetic_suite();
    let synth_refs: Vec<&BlockData> = synth.iter().map(|(_, b)| b).collect();
    verify_exact_set("synthetic_stress_11", &synth_refs, &arms, &mut builder);

    let wiki = load_bz2_blocks(path, true);
    let adm: Vec<&BlockData> = wiki.iter().filter(|b| b.admitted).collect();
    let non_adm: Vec<&BlockData> = wiki.iter().filter(|b| !b.admitted).collect();
    verify_exact_set("wikidata_admitted_233", &adm, &arms, &mut builder);
    verify_exact_set("wikidata_forced_nonadm_198", &non_adm, &arms, &mut builder);

    if let Some(rp) = runs_path {
        let rb = load_bz2_blocks(rp, true);
        let radm: Vec<&BlockData> = rb.iter().filter(|b| b.admitted).collect();
        verify_exact_set("guard_runs_admitted", &radm, &arms, &mut builder);
    }
    if let Some(tp) = text_path {
        let tb = load_bz2_blocks(tp, true);
        let tall: Vec<&BlockData> = tb.iter().collect();
        verify_exact_set("guard_text_forced", &tall, &arms, &mut builder);
    }
}

fn bench_corpus(label: &str, blocks: &[&BlockData], arms: &[ArmSpec], repeats: usize) {
    let mut builder = OpenBuilder::default();
    let mut out = Vec::new();
    let total_rows: usize = blocks.iter().map(|b| b.n).sum();
    let n_arms = arms.len();

    let mut l4_fwds = Vec::with_capacity(blocks.len());
    for b in blocks {
        l4_fwds.push(builder.precompute_oracle_lanes(&b.records, b.n, b.p, &mut out));
    }

    // Warmup + exactness check
    for (bi, b) in blocks.iter().enumerate() {
        for arm in arms {
            run_arm(&mut builder, arm, b, l4_fwds[bi], &mut out);
            assert_eq!(
                out.as_slice(),
                b.walked.as_slice(),
                "MISMATCH {} blk={} arm={}",
                label,
                bi,
                arm.name
            );
        }
    }

    let mut totals_ms = vec![Vec::with_capacity(repeats); n_arms];
    let mut phases = vec![Vec::with_capacity(repeats); n_arms];
    for r in 0..repeats {
        let mut rep_acc = vec![PhaseNs::default(); n_arms];
        for (bi, b) in blocks.iter().enumerate() {
            for k in 0..n_arms {
                let ai = (k + bi + r) % n_arms;
                let ph = run_arm(&mut builder, &arms[ai], b, l4_fwds[bi], &mut out);
                black_box(out.as_ptr());
                rep_acc[ai].add(ph);
            }
        }
        for ai in 0..n_arms {
            totals_ms[ai].push(rep_acc[ai].total() as f64 / 1e6);
            phases[ai].push(rep_acc[ai]);
        }
    }

    let f1_bidi_med = median_min_max(
        &(0..repeats)
            .map(|r| totals_ms[1][r] / totals_ms[0][r])
            .collect::<Vec<_>>(),
    )
    .0;
    let f4o_bidi_med = median_min_max(
        &(0..repeats)
            .map(|r| totals_ms[2][r] / totals_ms[0][r])
            .collect::<Vec<_>>(),
    )
    .0;
    let f4r_bidi_med = median_min_max(
        &(0..repeats)
            .map(|r| totals_ms[3][r] / totals_ms[0][r])
            .collect::<Vec<_>>(),
    )
    .0;
    let i9_f1 = I9_CE_BASE_MS * f1_bidi_med;
    let i9_f4o = I9_CE_BASE_MS * f4o_bidi_med;
    let i9_f4r = I9_CE_BASE_MS * f4r_bidi_med;
    let _ = I9_GATE_MS;

    println!(
        "\n=== {} ({} blocks, {} rows, {} block-interleaved repeats) ===",
        label,
        blocks.len(),
        total_rows,
        repeats
    );
    println!(
        "  {:19} {:>6} {:>6} {:>5} {:>6} {:>6} | {:>7} ({:>13}) | {:>17} {:>17} {:>17} | {:>6} {:>6} {:>6} {:>6}",
        "arm",
        "A6a",
        "A6b",
        "A6c",
        "contr",
        "A7",
        "C+E_ms",
        "min..max",
        "vsF1 (min..max)",
        "vsF4o (min..max)",
        "vsF4r (min..max)",
        "i9_CE",
        "dF1",
        "dF4o",
        "dF4r"
    );

    for (ai, arm) in arms.iter().enumerate() {
        let (med, mn, mx) = median_min_max(&totals_ms[ai]);
        let a6a = median_min_max(
            &phases[ai]
                .iter()
                .map(|p| p.a6a as f64 / 1e6)
                .collect::<Vec<_>>(),
        )
        .0;
        let a6b = median_min_max(
            &phases[ai]
                .iter()
                .map(|p| p.a6b as f64 / 1e6)
                .collect::<Vec<_>>(),
        )
        .0;
        let a6c = median_min_max(
            &phases[ai]
                .iter()
                .map(|p| p.a6c as f64 / 1e6)
                .collect::<Vec<_>>(),
        )
        .0;
        let contr = median_min_max(
            &phases[ai]
                .iter()
                .map(|p| p.contract as f64 / 1e6)
                .collect::<Vec<_>>(),
        )
        .0;
        let a7 = median_min_max(
            &phases[ai]
                .iter()
                .map(|p| p.a7 as f64 / 1e6)
                .collect::<Vec<_>>(),
        )
        .0;

        let pair_f1: Vec<f64> = (0..repeats)
            .map(|r| totals_ms[ai][r] / totals_ms[1][r])
            .collect();
        let (pf1_med, pf1_min, pf1_max) = median_min_max(&pair_f1);
        let pair_f4o: Vec<f64> = (0..repeats)
            .map(|r| totals_ms[ai][r] / totals_ms[2][r])
            .collect();
        let (pf4o_med, pf4o_min, pf4o_max) = median_min_max(&pair_f4o);
        let pair_f4r: Vec<f64> = (0..repeats)
            .map(|r| totals_ms[ai][r] / totals_ms[3][r])
            .collect();
        let (pf4r_med, pf4r_min, pf4r_max) = median_min_max(&pair_f4r);
        let pair_bidi: Vec<f64> = (0..repeats)
            .map(|r| totals_ms[ai][r] / totals_ms[0][r])
            .collect();
        let pbidi_med = median_min_max(&pair_bidi).0;

        let i9_ce = I9_CE_BASE_MS * pbidi_med;
        let d_f1 = i9_f1 - i9_ce;
        let d_f4o = i9_f4o - i9_ce;
        let d_f4r = i9_f4r - i9_ce;
        println!(
            "  {:19} {:>6.2} {:>6.2} {:>5.2} {:>6.2} {:>6.2} | {:>7.2} ({:>6.2}..{:>6.2}) | {:>5.3} ({:>5.3}..{:>5.3}) {:>5.3} ({:>5.3}..{:>5.3}) {:>5.3} ({:>5.3}..{:>5.3}) | {:>6.1} {:>+6.1} {:>+6.1} {:>+6.1}",
            arm.name,
            a6a, a6b, a6c, contr, a7,
            med, mn, mx,
            pf1_med, pf1_min, pf1_max,
            pf4o_med, pf4o_min, pf4o_max,
            pf4r_med, pf4r_min, pf4r_max,
            i9_ce, d_f1, d_f4o, d_f4r,
        );
    }
}

fn run_train(path: &str, repeats: usize) {
    let wiki = load_bz2_blocks(path, false);
    let adm: Vec<&BlockData> = wiki.iter().filter(|b| b.admitted).collect();
    let train = &adm[..TRAIN_SPLIT_BLOCKS.min(adm.len())];
    let arms = train_arms();
    bench_corpus("TRAIN WIKIDATA (117 blocks)", train, &arms, repeats);
}

fn run_bench(path: &str, repeats: usize, runs_path: Option<&str>, text_path: Option<&str>) {
    let wiki = load_bz2_blocks(path, false);
    let adm: Vec<&BlockData> = wiki.iter().filter(|b| b.admitted).collect();
    let held_out = &adm[TRAIN_SPLIT_BLOCKS.min(adm.len())..];
    let arms = eval_arms();
    bench_corpus("HELD-OUT WIKIDATA (116 blocks)", held_out, &arms, repeats);

    if let Some(rp) = runs_path {
        let rb = load_bz2_blocks(rp, true);
        let radm: Vec<&BlockData> = rb.iter().filter(|b| b.admitted).collect();
        bench_corpus("GUARD: runs.bz2 (admitted)", &radm, &arms, repeats);
    }
    if let Some(tp) = text_path {
        let tb = load_bz2_blocks(tp, true);
        let tall: Vec<&BlockData> = tb.iter().collect();
        bench_corpus("GUARD: text.bz2 (forced)", &tall, &arms, repeats);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let repeats = args
        .windows(2)
        .find(|w| w[0] == "--repeats")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(7);
    let runs_path = args
        .windows(2)
        .find(|w| w[0] == "--runs")
        .map(|w| w[1].as_str());
    let text_path = args
        .windows(2)
        .find(|w| w[0] == "--text")
        .map(|w| w[1].as_str());
    let path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--") && a.ends_with(".bz2"))
        .map(String::as_str)
        .unwrap_or("corpora/wikidata.bz2");
    match args.first().map(String::as_str) {
        Some("exact") => run_exact(path, runs_path, text_path),
        Some("train") => run_train(path, repeats),
        Some("bench") => run_bench(path, repeats, runs_path, text_path),
        _ => eprintln!("usage: bwt_contraction_rescue <exact|train|bench> [FILE.bz2] [--repeats N] [--runs PATH] [--text PATH]"),
    }
}
