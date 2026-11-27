#![forbid(unsafe_code)]
//! Paid sampled-anchor multi-lane forward wiring (`A6c`) and 4-lane emission (`A7`) probe.
//!
//! Smooth-zone diagnostic for claim `forward-4lane-anchor-probe`
//! (`frontier/claims/forward-4lane-anchor-probe.toml`). Zero edits to `src/`, zero `unsafe`.
//!
//! Usage:
//!   forward_4lane_probe exact  [FILE.bz2]
//!   forward_4lane_probe census [FILE.bz2]
//!   forward_4lane_probe bench  [FILE.bz2] [--repeats N]
//!   forward_4lane_probe all    [FILE.bz2] [--repeats N]

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
        let mut prep = PreparedBlock {
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
            oracle_lanes: None,
            expected: Vec::new(),
        };

        // Precompute oracle 4-lane split (offline, never charged to oracle A6c)
        let mut tmp_edges = vec![0u32; m];
        let mut tmp_extra = if wide { vec![0u32; m] } else { Vec::new() };
        wire_forward(&prep, &prep.outq_bytes, &mut tmp_edges, &mut tmp_extra);
        prep.oracle_lanes = build_oracle_4lanes(&prep, &tmp_edges, &tmp_extra);

        (
            prep,
            t1.duration_since(t0).as_nanos() as u64,
            t2.duration_since(t1).as_nanos() as u64,
        )
    }
}

#[derive(Clone, Copy)]
struct Lane4 {
    starts: [u32; 4],
    byte_offs: [usize; 4],
    ends: [usize; 4],
    steps: [usize; 4],
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
    oracle_lanes: Option<Lane4>,
    expected: Vec<u8>,
}

fn build_oracle_4lanes(prep: &PreparedBlock, edges: &[u32], extra: &[u32]) -> Option<Lane4> {
    if prep.m < 64 {
        return None;
    }
    let mask = (1u32 << prep.bits) - 1;
    let mut cycle_rows = Vec::with_capacity(prep.m);
    let mut cycle_offs = Vec::with_capacity(prep.m + 1);
    let mut j = prep.mark;
    let mut at = 0usize;
    for _ in 0..prep.m {
        cycle_rows.push(j);
        cycle_offs.push(at);
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> prep.bits
        } else {
            extra[j as usize]
        } as usize;
        let len = prep.lens[code];
        let wlen = if len != LONG {
            usize::from(len)
        } else {
            long_word(&prep.inline16[code], &prep.spelling).len()
        };
        at += wlen;
        j = edge & mask;
        if j == prep.mark || at >= prep.n {
            break;
        }
    }
    if at != prep.n || j != prep.mark || cycle_rows.len() < 4 {
        return None;
    }
    cycle_offs.push(at);
    let total_steps = cycle_rows.len();
    let mut starts = [0u32; 4];
    let mut byte_offs = [0usize; 4];
    let mut ends = [0usize; 4];
    let mut steps = [0usize; 4];
    let mut s_prev = 0usize;
    for lane in 0..4 {
        let s_next = if lane == 3 {
            total_steps
        } else {
            total_steps * (lane + 1) / 4
        };
        starts[lane] = cycle_rows[s_prev];
        byte_offs[lane] = cycle_offs[s_prev];
        ends[lane] = cycle_offs[s_next];
        steps[lane] = s_next - s_prev;
        s_prev = s_next;
    }
    Some(Lane4 {
        starts,
        byte_offs,
        ends,
        steps,
    })
}

// ---------------------------------------------------------------------------
// A6c wiring routines (shipped XOR bidi, 1-lane forward, paid sampled-anchor forward)
// ---------------------------------------------------------------------------

#[inline(never)]
fn wire_bidi(
    prep: &PreparedBlock,
    outq_buf: &[u8],
    edges: &mut [u32],
    extra: &mut [u32],
) -> Option<u32> {
    if !prep.wide && prep.m >= 65_536 {
        let edges = &mut edges[..prep.m];
        edges.fill(0);
        let mut predecessor = prep.mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = prep.cursors.get(a + 1).copied().unwrap_or(prep.bands.len());
            let mut queue = Reader::new(outq_buf, &prep.spill, &prep.heads, a);
            for &band in &prep.bands[prep.cursors[a]..end] {
                queue.wire_xor(
                    edges,
                    band.dst() as usize,
                    (band.end() - k) as usize,
                    band.code() << prep.bits,
                    prep.mark,
                    &mut predecessor,
                );
                k = band.end();
            }
        }
        Some(predecessor)
    } else {
        wire_forward(prep, outq_buf, edges, extra);
        None
    }
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnchorScheme {
    Uniform,
    Golden,
}

/// Paid sampled-anchor forward wiring: identical SIMD `Reader::wire` plus O(S)
/// anchor selection and bit-31 tagging directly on `edges[anchors[k]]`.
#[inline(never)]
fn wire_forward_sampled(
    prep: &PreparedBlock,
    outq_buf: &[u8],
    edges: &mut [u32],
    extra: &mut [u32],
    s_anchors: usize,
    scheme: AnchorScheme,
    anchors_out: &mut [u32; MAX_ANCHORS],
) -> usize {
    wire_forward(prep, outq_buf, edges, extra);
    if !prep.can_tag || !(4..=MAX_ANCHORS).contains(&s_anchors) || prep.m < s_anchors * 8 {
        return 0;
    }
    let m = prep.m as u32;
    let mark = prep.mark;
    anchors_out[0] = mark;
    let mut count = 1usize;
    match scheme {
        AnchorScheme::Uniform => {
            // Evenly spaced in BWT retained-row index space (0..m). Since m >= 8*S,
            // every offset k*m/S is strictly distinct modulo m.
            for k in 1..s_anchors {
                let off = ((k as u64 * m as u64) / s_anchors as u64) as u32;
                let mut a = mark + off;
                if a >= m {
                    a -= m;
                }
                anchors_out[count] = a;
                count += 1;
            }
        }
        AnchorScheme::Golden => {
            // Golden-ratio Weyl sequence modulo m, with tiny O(S) dedup guard.
            const PHI: u64 = 0x9E37_79B9_7F4A_7C15;
            for k in 1..s_anchors {
                let h = (k as u64).wrapping_mul(PHI) >> 32;
                let mut a = ((mark as u64 + ((h * m as u64) >> 32)) % m as u64) as u32;
                while anchors_out[..count].contains(&a) {
                    a += 1;
                    if a == m {
                        a = 0;
                    }
                }
                anchors_out[count] = a;
                count += 1;
            }
        }
    }
    for &a in &anchors_out[..count] {
        edges[a as usize] |= ANCHOR_TAG;
    }
    count
}

// ---------------------------------------------------------------------------
// A7 emitters: shipped bidi, 1-lane forward, 4-lane oracle, paid sampled-anchor 4-lane
// ---------------------------------------------------------------------------

#[inline(always)]
fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[inline(never)]
fn emit_fwd_1lane(prep: &PreparedBlock, edges: &[u32], extra: &[u32], out: &mut Vec<u8>) {
    let n = prep.n;
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let bits = prep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = prep.mark;
    let mut j = mark;
    let spelling = &prep.spelling;
    let inline16 = &prep.inline16;
    let lens = &prep.lens;
    for _ in 0..prep.m {
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
fn emit_bidi(
    prep: &PreparedBlock,
    edges: &[u32],
    extra: &[u32],
    predecessor: Option<u32>,
    out: &mut Vec<u8>,
) {
    let Some(before) = predecessor else {
        emit_fwd_1lane(prep, edges, extra, out);
        return;
    };
    let n = prep.n;
    out.resize(n + INLINE, 0);
    let bits = prep.bits;
    let mask = (1u32 << bits) - 1;
    let mark = prep.mark;
    let spelling = &prep.spelling;
    let inline = &prep.inline16;
    let lens = &prep.lens;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=prep.m / 2 {
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
fn emit_4lane_oracle(prep: &PreparedBlock, edges: &[u32], extra: &[u32], out: &mut Vec<u8>) {
    let Some(l4) = prep.oracle_lanes else {
        emit_fwd_1lane(prep, edges, extra, out);
        return;
    };
    let n = prep.n;
    out.resize(n + INLINE, 0);
    let bits = prep.bits;
    let mask = (1u32 << bits) - 1;
    let spelling = &prep.spelling;
    let inline16 = &prep.inline16;
    let lens = &prep.lens;

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

// ---------------------------------------------------------------------------
// Paid sampled-anchor 4-lane forward emitter (slabbed vectored + optional stitch)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct SliceRef {
    off: u32,
    len: u32,
    next: u16, // linked list of slices per segment (u16::MAX = none)
}

#[derive(Default)]
struct SampledScratch {
    slices: Vec<SliceRef>,
    ordered: Vec<(usize, usize)>,
}

#[derive(Default, Clone, Copy)]
struct SampledStats {
    iters_4: u64,
    iters_3: u64,
    iters_2: u64,
    iters_1: u64,
    anchor_hits: u64,
    slab_switches: u64,
    ordered_slices: u64,
    max_arena_used: usize,
    seg_max_steps: usize,
    seg_total_steps: usize,
    fallbacks: u64,
}

struct SlabAlloc<'a> {
    arena: &'a mut Vec<u8>,
    top: usize,
    slab_switches: u64,
    slices: &'a mut Vec<SliceRef>,
    seg_head: [u16; MAX_ANCHORS],
    seg_tail: [u16; MAX_ANCHORS],
}

impl<'a> SlabAlloc<'a> {
    #[inline(always)]
    fn push_slice(&mut self, seg: usize, start: usize, end: usize) {
        if end > start {
            let id = self.slices.len() as u16;
            self.slices.push(SliceRef {
                off: start as u32,
                len: (end - start) as u32,
                next: u16::MAX,
            });
            if self.seg_head[seg] == u16::MAX {
                self.seg_head[seg] = id;
            } else {
                self.slices[self.seg_tail[seg] as usize].next = id;
            }
            self.seg_tail[seg] = id;
        }
    }

    #[inline(always)]
    fn take_slab(&mut self, min_bytes: usize) -> (usize, usize) {
        let size = SLAB_SIZE.max(min_bytes + INLINE);
        let base = self.top;
        self.top = base + size;
        if self.top > self.arena.len() {
            self.arena.resize(self.top.max(self.arena.len() * 2), 0);
        }
        (base, base + size)
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_slow(
        &mut self,
        seg: usize,
        code: usize,
        len: u8,
        inline16: &[[u8; INLINE]],
        spelling: &[u8],
        slice_start: &mut usize,
        at: &mut usize,
        end: &mut usize,
    ) {
        let word = if len == LONG {
            long_word(&inline16[code], spelling)
        } else {
            &inline16[code][..usize::from(len)]
        };
        let wlen = word.len();
        if *at + wlen + INLINE > *end {
            self.push_slice(seg, *slice_start, *at);
            self.slab_switches += 1;
            let (new_base, new_end) = self.take_slab(wlen);
            *slice_start = new_base;
            *at = new_base;
            *end = new_end;
        }
        self.arena[*at..*at + wlen].copy_from_slice(word);
        *at += wlen;
    }
}

/// Emit `prep` using `s_anchors` pre-tagged anchors (`edges[a_k]` has bit 31 set)
/// dynamically scheduled across `K = 4` concurrent lanes into flat slab `arena`.
///
/// On return, `scratch.ordered` holds the exact ordered `(offset, len)` slices in
/// `arena` whose concatenation equals `prep`'s exact `n` decoded bytes (for periodic
/// blocks, the period's slices are repeated to `n` bytes).
///
/// Clears bit-31 anchor tags from `edges` before returning.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_4lane_sampled<const STATS: bool>(
    prep: &PreparedBlock,
    edges: &mut [u32],
    extra: &[u32],
    anchors: &[u32],
    arena: &mut Vec<u8>,
    scratch: &mut SampledScratch,
) -> SampledStats {
    let mut st = SampledStats::default();
    let s = anchors.len();
    if s < 4 || !prep.can_tag {
        for &a in anchors {
            edges[a as usize] &= !ANCHOR_TAG;
        }
        emit_fwd_1lane(prep, edges, extra, arena);
        scratch.ordered.clear();
        scratch.ordered.push((0, prep.n));
        st.fallbacks = 1;
        return st;
    }

    let n = prep.n;
    let m = prep.m;
    let bits = prep.bits;
    let mask = (1u32 << bits) - 1;
    let inline16 = &prep.inline16;
    let lens = &prep.lens;
    let spelling = &prep.spelling;

    let want_arena = n + 4 * SLAB_SIZE + 4 * INLINE;
    if arena.len() < want_arena {
        arena.resize(want_arena, 0);
    }
    scratch.slices.clear();
    scratch.ordered.clear();

    let mut alloc = SlabAlloc {
        arena,
        top: 0,
        slab_switches: 0,
        slices: &mut scratch.slices,
        seg_head: [u16::MAX; MAX_ANCHORS],
        seg_tail: [u16::MAX; MAX_ANCHORS],
    };

    let mut seg_end_row = [u32::MAX; MAX_ANCHORS];
    let mut seg_steps = [0usize; MAX_ANCHORS];

    let mut lj = [0u32; 4];
    let mut lseg = [0usize; 4];
    let mut lstart = [0usize; 4];
    let mut lat = [0usize; 4];
    let mut lend = [0usize; 4];
    let mut lactive = [false; 4];

    let mut next_seg = 0usize;
    let mut steps_done = 0usize;
    let mut budget_exceeded = false;

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn start_segment(
        lane: usize,
        seg: usize,
        anchor_row: u32,
        edges: &[u32],
        bits: u32,
        mask: u32,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
        alloc: &mut SlabAlloc<'_>,
        lj: &mut [u32; 4],
        lseg: &mut [usize; 4],
        lstart: &mut [usize; 4],
        lat: &mut [usize; 4],
        lend: &mut [usize; 4],
        seg_steps: &mut [usize; MAX_ANCHORS],
        steps_done: &mut usize,
    ) {
        lseg[lane] = seg;
        lstart[lane] = lat[lane];
        let edge = edges[anchor_row as usize] & !ANCHOR_TAG;
        lj[lane] = edge & mask;
        let code = (edge >> bits) as usize;
        let len = lens[code];
        if len != LONG && lat[lane] + INLINE <= lend[lane] {
            let at = lat[lane];
            alloc.arena[at..at + INLINE].copy_from_slice(&inline16[code]);
            lat[lane] = at + usize::from(len);
        } else {
            alloc.emit_slow(
                seg,
                code,
                len,
                inline16,
                spelling,
                &mut lstart[lane],
                &mut lat[lane],
                &mut lend[lane],
            );
        }
        seg_steps[seg] = 1;
        *steps_done += 1;
    }

    for lane in 0..4 {
        let (base, end) = alloc.take_slab(0);
        lstart[lane] = base;
        lat[lane] = base;
        lend[lane] = end;
        lactive[lane] = true;
        let seg = next_seg;
        next_seg += 1;
        start_segment(
            lane,
            seg,
            anchors[seg],
            edges,
            bits,
            mask,
            inline16,
            lens,
            spelling,
            &mut alloc,
            &mut lj,
            &mut lseg,
            &mut lstart,
            &mut lat,
            &mut lend,
            &mut seg_steps,
            &mut steps_done,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_hit(
        lane: usize,
        s: usize,
        anchors: &[u32],
        edges: &[u32],
        bits: u32,
        mask: u32,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
        alloc: &mut SlabAlloc<'_>,
        lj: &mut [u32; 4],
        lseg: &mut [usize; 4],
        lstart: &mut [usize; 4],
        lat: &mut [usize; 4],
        lend: &mut [usize; 4],
        lactive: &mut [bool; 4],
        seg_end_row: &mut [u32; MAX_ANCHORS],
        seg_steps: &mut [usize; MAX_ANCHORS],
        next_seg: &mut usize,
        steps_done: &mut usize,
        m: usize,
        anchor_hits: &mut u64,
    ) -> bool {
        while (edges[lj[lane] as usize] as i32) < 0 {
            *anchor_hits += 1;
            let finished = lseg[lane];
            seg_end_row[finished] = lj[lane];
            alloc.push_slice(finished, lstart[lane], lat[lane]);
            if *next_seg < s {
                if *steps_done >= m {
                    lactive[lane] = false;
                    return false;
                }
                let seg = *next_seg;
                *next_seg += 1;
                start_segment(
                    lane,
                    seg,
                    anchors[seg],
                    edges,
                    bits,
                    mask,
                    inline16,
                    lens,
                    spelling,
                    alloc,
                    lj,
                    lseg,
                    lstart,
                    lat,
                    lend,
                    seg_steps,
                    steps_done,
                );
            } else {
                lactive[lane] = false;
                return false;
            }
        }
        true
    }

    for lane in 0..4 {
        if (edges[lj[lane] as usize] as i32) < 0 {
            handle_hit(
                lane,
                s,
                anchors,
                edges,
                bits,
                mask,
                inline16,
                lens,
                spelling,
                &mut alloc,
                &mut lj,
                &mut lseg,
                &mut lstart,
                &mut lat,
                &mut lend,
                &mut lactive,
                &mut seg_end_row,
                &mut seg_steps,
                &mut next_seg,
                &mut steps_done,
                m,
                &mut st.anchor_hits,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Phase 1: 4-lane counted hot loop (all 4 lanes active)
    // -----------------------------------------------------------------------
    if lactive[0] && lactive[1] && lactive[2] && lactive[3] {
        let (mut j0, mut j1, mut j2, mut j3) = (lj[0], lj[1], lj[2], lj[3]);
        let (mut a0, mut a1, mut a2, mut a3) = (lat[0], lat[1], lat[2], lat[3]);
        let (mut e0, mut e1, mut e2, mut e3) = (lend[0], lend[1], lend[2], lend[3]);

        loop {
            let max_iters = (m - steps_done) / 4;
            let mut done = 0usize;
            let mut hit = false;
            for _ in 0..max_iters {
                let ed0 = edges[j0 as usize];
                let ed1 = edges[j1 as usize];
                let ed2 = edges[j2 as usize];
                let ed3 = edges[j3 as usize];

                // Single combined sign-bit test across all 4 lanes!
                if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                    hit = true;
                    break;
                }

                j0 = ed0 & mask;
                j1 = ed1 & mask;
                j2 = ed2 & mask;
                j3 = ed3 & mask;

                let c0 = (ed0 >> bits) as usize;
                let l0 = lens[c0];
                if l0 != LONG && a0 + INLINE <= e0 {
                    alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                    a0 += usize::from(l0);
                } else {
                    alloc.emit_slow(
                        lseg[0],
                        c0,
                        l0,
                        inline16,
                        spelling,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                }

                let c1 = (ed1 >> bits) as usize;
                let l1 = lens[c1];
                if l1 != LONG && a1 + INLINE <= e1 {
                    alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                    a1 += usize::from(l1);
                } else {
                    alloc.emit_slow(
                        lseg[1],
                        c1,
                        l1,
                        inline16,
                        spelling,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                }

                let c2 = (ed2 >> bits) as usize;
                let l2 = lens[c2];
                if l2 != LONG && a2 + INLINE <= e2 {
                    alloc.arena[a2..a2 + INLINE].copy_from_slice(&inline16[c2]);
                    a2 += usize::from(l2);
                } else {
                    alloc.emit_slow(
                        lseg[2],
                        c2,
                        l2,
                        inline16,
                        spelling,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                }

                let c3 = (ed3 >> bits) as usize;
                let l3 = lens[c3];
                if l3 != LONG && a3 + INLINE <= e3 {
                    alloc.arena[a3..a3 + INLINE].copy_from_slice(&inline16[c3]);
                    a3 += usize::from(l3);
                } else {
                    alloc.emit_slow(
                        lseg[3],
                        c3,
                        l3,
                        inline16,
                        spelling,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                }

                done += 1;
            }
            steps_done += done * 4;
            if STATS {
                st.iters_4 += done as u64;
                seg_steps[lseg[0]] += done;
                seg_steps[lseg[1]] += done;
                seg_steps[lseg[2]] += done;
                seg_steps[lseg[3]] += done;
            }
            lj = [j0, j1, j2, j3];
            lat = [a0, a1, a2, a3];
            lend = [e0, e1, e2, e3];
            if !hit
                && ((edges[j0 as usize]
                    | edges[j1 as usize]
                    | edges[j2 as usize]
                    | edges[j3 as usize]) as i32)
                    >= 0
            {
                budget_exceeded = true;
                break;
            }
            let mut all_ok = true;
            for lane in 0..4 {
                if (edges[lj[lane] as usize] as i32) < 0
                    && !handle_hit(
                        lane,
                        s,
                        anchors,
                        edges,
                        bits,
                        mask,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lj,
                        &mut lseg,
                        &mut lstart,
                        &mut lat,
                        &mut lend,
                        &mut lactive,
                        &mut seg_end_row,
                        &mut seg_steps,
                        &mut next_seg,
                        &mut steps_done,
                        m,
                        &mut st.anchor_hits,
                    )
                {
                    all_ok = false;
                }
            }
            if !all_ok {
                break;
            }
            (j0, j1, j2, j3) = (lj[0], lj[1], lj[2], lj[3]);
            (a0, a1, a2, a3) = (lat[0], lat[1], lat[2], lat[3]);
            (e0, e1, e2, e3) = (lend[0], lend[1], lend[2], lend[3]);
        }
    }

    // -----------------------------------------------------------------------
    // Phase 2: Dynamic 3-lane -> 2-lane -> 1-lane tail drain
    // -----------------------------------------------------------------------
    let mut act = [0usize; 4];
    let mut n_act = 0usize;
    for (lane, &active) in lactive.iter().enumerate() {
        if active {
            act[n_act] = lane;
            n_act += 1;
        }
    }

    // 3-lane drain
    while !budget_exceeded && n_act == 3 {
        let (p0, p1, p2) = (act[0], act[1], act[2]);
        let (mut j0, mut j1, mut j2) = (lj[p0], lj[p1], lj[p2]);
        let (mut a0, mut a1, mut a2) = (lat[p0], lat[p1], lat[p2]);
        let (mut e0, mut e1, mut e2) = (lend[p0], lend[p1], lend[p2]);
        loop {
            let ed0 = edges[j0 as usize];
            let ed1 = edges[j1 as usize];
            let ed2 = edges[j2 as usize];
            if ((ed0 | ed1 | ed2) as i32) < 0 {
                (lj[p0], lj[p1], lj[p2]) = (j0, j1, j2);
                (lat[p0], lat[p1], lat[p2]) = (a0, a1, a2);
                (lend[p0], lend[p1], lend[p2]) = (e0, e1, e2);
                break;
            }
            if steps_done + 3 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            j1 = ed1 & mask;
            j2 = ed2 & mask;

            let c0 = (ed0 >> bits) as usize;
            let l0 = lens[c0];
            if l0 != LONG && a0 + INLINE <= e0 {
                alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                a0 += usize::from(l0);
            } else {
                alloc.emit_slow(
                    lseg[p0],
                    c0,
                    l0,
                    inline16,
                    spelling,
                    &mut lstart[p0],
                    &mut a0,
                    &mut e0,
                );
            }
            let c1 = (ed1 >> bits) as usize;
            let l1 = lens[c1];
            if l1 != LONG && a1 + INLINE <= e1 {
                alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                a1 += usize::from(l1);
            } else {
                alloc.emit_slow(
                    lseg[p1],
                    c1,
                    l1,
                    inline16,
                    spelling,
                    &mut lstart[p1],
                    &mut a1,
                    &mut e1,
                );
            }
            let c2 = (ed2 >> bits) as usize;
            let l2 = lens[c2];
            if l2 != LONG && a2 + INLINE <= e2 {
                alloc.arena[a2..a2 + INLINE].copy_from_slice(&inline16[c2]);
                a2 += usize::from(l2);
            } else {
                alloc.emit_slow(
                    lseg[p2],
                    c2,
                    l2,
                    inline16,
                    spelling,
                    &mut lstart[p2],
                    &mut a2,
                    &mut e2,
                );
            }
            steps_done += 3;
            if STATS {
                st.iters_3 += 1;
                seg_steps[lseg[p0]] += 1;
                seg_steps[lseg[p1]] += 1;
                seg_steps[lseg[p2]] += 1;
            }
        }
        if budget_exceeded {
            break;
        }
        let mut next_act = [0usize; 4];
        let mut new_n = 0;
        for &lane in &act[..n_act] {
            if (edges[lj[lane] as usize] as i32) < 0 {
                if handle_hit(
                    lane,
                    s,
                    anchors,
                    edges,
                    bits,
                    mask,
                    inline16,
                    lens,
                    spelling,
                    &mut alloc,
                    &mut lj,
                    &mut lseg,
                    &mut lstart,
                    &mut lat,
                    &mut lend,
                    &mut lactive,
                    &mut seg_end_row,
                    &mut seg_steps,
                    &mut next_seg,
                    &mut steps_done,
                    m,
                    &mut st.anchor_hits,
                ) {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            } else {
                next_act[new_n] = lane;
                new_n += 1;
            }
        }
        act = next_act;
        n_act = new_n;
    }

    // 2-lane drain
    while !budget_exceeded && n_act == 2 {
        let (p0, p1) = (act[0], act[1]);
        let (mut j0, mut j1) = (lj[p0], lj[p1]);
        let (mut a0, mut a1) = (lat[p0], lat[p1]);
        let (mut e0, mut e1) = (lend[p0], lend[p1]);
        loop {
            let ed0 = edges[j0 as usize];
            let ed1 = edges[j1 as usize];
            if ((ed0 | ed1) as i32) < 0 {
                (lj[p0], lj[p1]) = (j0, j1);
                (lat[p0], lat[p1]) = (a0, a1);
                (lend[p0], lend[p1]) = (e0, e1);
                break;
            }
            if steps_done + 2 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            j1 = ed1 & mask;

            let c0 = (ed0 >> bits) as usize;
            let l0 = lens[c0];
            if l0 != LONG && a0 + INLINE <= e0 {
                alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                a0 += usize::from(l0);
            } else {
                alloc.emit_slow(
                    lseg[p0],
                    c0,
                    l0,
                    inline16,
                    spelling,
                    &mut lstart[p0],
                    &mut a0,
                    &mut e0,
                );
            }
            let c1 = (ed1 >> bits) as usize;
            let l1 = lens[c1];
            if l1 != LONG && a1 + INLINE <= e1 {
                alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                a1 += usize::from(l1);
            } else {
                alloc.emit_slow(
                    lseg[p1],
                    c1,
                    l1,
                    inline16,
                    spelling,
                    &mut lstart[p1],
                    &mut a1,
                    &mut e1,
                );
            }
            steps_done += 2;
            if STATS {
                st.iters_2 += 1;
                seg_steps[lseg[p0]] += 1;
                seg_steps[lseg[p1]] += 1;
            }
        }
        if budget_exceeded {
            break;
        }
        let mut next_act = [0usize; 4];
        let mut new_n = 0;
        for &lane in &act[..n_act] {
            if (edges[lj[lane] as usize] as i32) < 0 {
                if handle_hit(
                    lane,
                    s,
                    anchors,
                    edges,
                    bits,
                    mask,
                    inline16,
                    lens,
                    spelling,
                    &mut alloc,
                    &mut lj,
                    &mut lseg,
                    &mut lstart,
                    &mut lat,
                    &mut lend,
                    &mut lactive,
                    &mut seg_end_row,
                    &mut seg_steps,
                    &mut next_seg,
                    &mut steps_done,
                    m,
                    &mut st.anchor_hits,
                ) {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            } else {
                next_act[new_n] = lane;
                new_n += 1;
            }
        }
        act = next_act;
        n_act = new_n;
    }

    // 1-lane drain
    while !budget_exceeded && n_act == 1 {
        let p0 = act[0];
        let mut j0 = lj[p0];
        let mut a0 = lat[p0];
        let mut e0 = lend[p0];
        loop {
            let ed0 = edges[j0 as usize];
            if (ed0 as i32) < 0 {
                lj[p0] = j0;
                lat[p0] = a0;
                lend[p0] = e0;
                break;
            }
            if steps_done + 1 > m {
                budget_exceeded = true;
                break;
            }
            j0 = ed0 & mask;
            let c0 = (ed0 >> bits) as usize;
            let l0 = lens[c0];
            if l0 != LONG && a0 + INLINE <= e0 {
                alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                a0 += usize::from(l0);
            } else {
                alloc.emit_slow(
                    lseg[p0],
                    c0,
                    l0,
                    inline16,
                    spelling,
                    &mut lstart[p0],
                    &mut a0,
                    &mut e0,
                );
            }
            steps_done += 1;
            if STATS {
                st.iters_1 += 1;
                seg_steps[lseg[p0]] += 1;
            }
        }
        if budget_exceeded {
            break;
        }
        if !handle_hit(
            p0,
            s,
            anchors,
            edges,
            bits,
            mask,
            inline16,
            lens,
            spelling,
            &mut alloc,
            &mut lj,
            &mut lseg,
            &mut lstart,
            &mut lat,
            &mut lend,
            &mut lactive,
            &mut seg_end_row,
            &mut seg_steps,
            &mut next_seg,
            &mut steps_done,
            m,
            &mut st.anchor_hits,
        ) {
            n_act = 0;
        }
    }

    // Untag anchor bits in O(S)
    for &a in anchors {
        edges[a as usize] &= !ANCHOR_TAG;
    }

    // Chain the cycle starting from anchor 0 (= mark) in O(S)
    let mut cycle_closed = false;
    let mut cycle_bytes = 0usize;
    if !budget_exceeded {
        let mut cur = 0usize;
        for _ in 0..s {
            let mut sid = alloc.seg_head[cur];
            while sid != u16::MAX {
                let sl = alloc.slices[sid as usize];
                scratch.ordered.push((sl.off as usize, sl.len as usize));
                cycle_bytes += sl.len as usize;
                sid = sl.next;
            }
            let end_row = seg_end_row[cur];
            let Some(nxt) = anchors.iter().position(|&a| a == end_row) else {
                break;
            };
            cur = nxt;
            if cur == 0 {
                cycle_closed = true;
                break;
            }
        }
    }

    if !cycle_closed || cycle_bytes == 0 || cycle_bytes > n {
        // Safe fallback (corrupt/non-permutation input)
        emit_fwd_1lane(prep, edges, extra, arena);
        scratch.ordered.clear();
        scratch.ordered.push((0, n));
        st.fallbacks = 1;
        return st;
    }

    // Periodic block: repeat the period's ordered slices to exact `n` bytes
    if cycle_bytes < n {
        let base_slices = scratch.ordered.len();
        let mut total = cycle_bytes;
        'rep: while total < n {
            for idx in 0..base_slices {
                let (off, len) = scratch.ordered[idx];
                let take = len.min(n - total);
                scratch.ordered.push((off, take));
                total += take;
                if total == n {
                    break 'rep;
                }
            }
        }
    }

    st.slab_switches = alloc.slab_switches;
    st.ordered_slices = scratch.ordered.len() as u64;
    st.max_arena_used = alloc.top;
    if STATS {
        st.seg_max_steps = seg_steps[..s].iter().copied().max().unwrap_or(0);
        st.seg_total_steps = seg_steps[..s].iter().sum();
    }
    st
}

#[inline(never)]
fn stitch_slices(arena: &[u8], ordered: &[(usize, usize)], n: usize, out: &mut Vec<u8>) {
    out.resize(n, 0);
    let mut at = 0usize;
    for &(off, len) in ordered {
        out[at..at + len].copy_from_slice(&arena[off..off + len]);
        at += len;
    }
    debug_assert_eq!(at, n);
}

// ---------------------------------------------------------------------------
// Corpus loading & synthetic edge-case generator
// ---------------------------------------------------------------------------

struct RawBlock {
    admitted: bool,
    records: Vec<u16>,
    n: usize,
    p: u32,
    walked: Vec<u8>,
}

fn load_bz2_blocks(path: &str) -> Vec<RawBlock> {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut l = Vec::new();
    let mut counts = [0u32; 256];
    let mut t = TVector::default();
    let mut out = Vec::new();
    while let Event::Block { symbols: b, .. } = sc
        .next(std::mem::take(&mut syms))
        .unwrap_or_else(|e| panic!("scan error in {path}: {e:?}"))
    {
        mtf::mtf(&b, &mut l, &mut counts, k).expect("mtf");
        let n = l.len();
        let p = b.orig_ptr;
        let records = records_of(&l);
        let admitted = match open_cap(records.len(), n, p, false) {
            Some(Cap::NONE) => true,
            Some(cap) => within_cap(&records, p, PROD_LIMIT, cap),
            None => false,
        };
        build(&l, &counts, &mut t);
        let mut walked = Vec::with_capacity(n);
        walk(&t, p, &mut walked);
        out.push(RawBlock {
            admitted,
            records,
            n,
            p,
            walked,
        });
        syms = b.syms;
    }
    out
}

fn make_bwt_block(name: &str, text: &[u8]) -> RawBlock {
    let n = text.len();
    let mut rots: Vec<usize> = (0..n).collect();
    rots.sort_by(|&a, &b| {
        for off in 0..n {
            let ca = text[(a + off) % n];
            let cb = text[(b + off) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut p = 0u32;
    let mut l = Vec::with_capacity(n);
    for (idx, &start) in rots.iter().enumerate() {
        if start == 0 {
            p = idx as u32;
        }
        l.push(text[(start + n - 1) % n]);
    }
    let mut counts = [0u32; 256];
    for &b in &l {
        counts[usize::from(b)] += 1;
    }
    let mut t = TVector::default();
    build(&l, &counts, &mut t);
    let mut walked = Vec::with_capacity(n);
    walk(&t, p, &mut walked);
    assert_eq!(walked, text, "BWT roundtrip failed for synthetic {name}");
    RawBlock {
        admitted: true,
        records: records_of(&l),
        n,
        p,
        walked,
    }
}

fn synthetic_edge_blocks() -> Vec<RawBlock> {
    let mut blocks = vec![
        make_bwt_block("syn_n1", b"a"),
        make_bwt_block("syn_n2_ab", b"ab"),
        make_bwt_block("syn_n2_ba", b"ba"),
        make_bwt_block("syn_short_10", b"abracadabr"),
        make_bwt_block(
            "syn_short_64",
            b"the quick brown fox jumps over the lazy dog 0123456789!@#$%^&*()",
        ),
    ];
    // Short block around m ~ 256 boundary (tests S > m and tight-anchor edge cases)
    let mut s300 = Vec::with_capacity(300);
    let mut x = 0x1234_5678u32;
    for _ in 0..300 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        s300.push(b'a' + ((x >> 24) as u8 % 26));
    }
    blocks.push(make_bwt_block("syn_dense_300", &s300));
    // Periodic multi-cycle blocks
    blocks.push(make_bwt_block("syn_periodic_abac256", &b"abac".repeat(256)));
    blocks.push(make_bwt_block("syn_periodic_aaab512", &b"aaab".repeat(512)));
    blocks.push(make_bwt_block("syn_periodic_ab500", &b"ab".repeat(500)));
    blocks.push(make_bwt_block("syn_const_1000", &vec![b'z'; 1000]));
    // Dense 4 KiB pseudo-random (all thin runs, m = 4096)
    let mut d4k = Vec::with_capacity(4096);
    for _ in 0..4096 {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        d4k.push(b'!' + ((x >> 24) as u8 % 90));
    }
    blocks.push(make_bwt_block("syn_dense_4k", &d4k));
    // Run-heavy 4 KiB
    let mut rh = Vec::with_capacity(4096);
    for i in 0..64 {
        let ch = b'a' + (i % 8) as u8;
        let rep = if i % 3 == 0 { 40 } else { 5 };
        for _ in 0..rep {
            rh.push(ch);
        }
    }
    blocks.push(make_bwt_block("syn_run_heavy", &rh));
    blocks
}

// ---------------------------------------------------------------------------
// Exactness verification across ALL blocks and ALL arms
// ---------------------------------------------------------------------------

fn verify_exact_all(preps: &[PreparedBlock], label: &str) {
    let mut edges = Vec::new();
    let mut extra = Vec::new();
    let mut out = Vec::new();
    let mut arena = Vec::new();
    let mut scratch = SampledScratch::default();
    let mut anchors = [0u32; MAX_ANCHORS];

    let s_list = [4usize, 8, 12, 16, 24, 32];
    let mut total_checks = 0usize;

    for (bi, prep) in preps.iter().enumerate() {
        edges.resize(prep.m, 0);
        extra.resize(if prep.wide { prep.m } else { 0 }, 0);

        // 1. shipped_bidi
        let pred = wire_bidi(prep, &prep.outq_bytes, &mut edges, &mut extra);
        emit_bidi(prep, &edges, &extra, pred, &mut out);
        assert_eq!(
            out, prep.expected,
            "{label} blk {bi}: shipped_bidi mismatch"
        );
        total_checks += 1;

        // 2. fwd_1lane
        wire_forward(prep, &prep.outq_bytes, &mut edges, &mut extra);
        emit_fwd_1lane(prep, &edges, &extra, &mut out);
        assert_eq!(out, prep.expected, "{label} blk {bi}: fwd_1lane mismatch");
        total_checks += 1;

        // 3. fwd_4lane_oracle
        emit_4lane_oracle(prep, &edges, &extra, &mut out);
        assert_eq!(
            out, prep.expected,
            "{label} blk {bi}: fwd_4lane_oracle mismatch"
        );
        total_checks += 1;

        // 4. All sampled arms (uniform + golden, vectored + stitched)
        for &s in &s_list {
            for scheme in [AnchorScheme::Uniform, AnchorScheme::Golden] {
                let cnt = wire_forward_sampled(
                    prep,
                    &prep.outq_bytes,
                    &mut edges,
                    &mut extra,
                    s,
                    scheme,
                    &mut anchors,
                );
                emit_4lane_sampled::<false>(
                    prep,
                    &mut edges,
                    &extra,
                    &anchors[..cnt],
                    &mut arena,
                    &mut scratch,
                );
                // Verify vectored concatenation matches expected byte-for-byte
                let mut concat = Vec::with_capacity(prep.n);
                for &(off, len) in &scratch.ordered {
                    concat.extend_from_slice(&arena[off..off + len]);
                }
                assert_eq!(
                    concat, prep.expected,
                    "{label} blk {bi}: sampled S={s} vec mismatch"
                );
                // Verify stitched output matches expected byte-for-byte
                stitch_slices(&arena, &scratch.ordered, prep.n, &mut out);
                assert_eq!(
                    out, prep.expected,
                    "{label} blk {bi}: sampled S={s} stitch mismatch"
                );
                // Verify anchor tags were completely cleared
                assert!(
                    edges[..prep.m].iter().all(|&e| (e & ANCHOR_TAG) == 0),
                    "{label} blk {bi}: stale ANCHOR_TAG left in edges"
                );
                total_checks += 2;
            }
        }
    }
    println!(
        "EXACT {label}: {}/{} blocks x {} arms = {} checks 100% byte-exact!",
        preps.len(),
        preps.len(),
        3 + s_list.len() * 4,
        total_checks
    );
}

// ---------------------------------------------------------------------------
// Structural & load-balance census
// ---------------------------------------------------------------------------

fn run_census(admitted: &[PreparedBlock]) {
    let total_m: usize = admitted.iter().map(|b| b.m).sum();
    let total_n: usize = admitted.iter().map(|b| b.n).sum();
    let can_tag_cnt = admitted.iter().filter(|b| b.can_tag).count();
    let wide_cnt = admitted.iter().filter(|b| b.wide).count();
    let oracle_cnt = admitted.iter().filter(|b| b.oracle_lanes.is_some()).count();

    // Verify single-cycle vs multi-cycle count across all admitted blocks
    let mut single_cycle = 0usize;
    let mut max_bits = 0u32;
    let mut max_dict = 0usize;
    let mut edges = Vec::new();
    let mut extra = Vec::new();
    for prep in admitted {
        max_bits = max_bits.max(prep.bits);
        max_dict = max_dict.max(prep.lens.len());
        edges.resize(prep.m, 0);
        extra.resize(if prep.wide { prep.m } else { 0 }, 0);
        wire_forward(prep, &prep.outq_bytes, &mut edges, &mut extra);
        let mask = (1u32 << prep.bits) - 1;
        let mut j = prep.mark;
        let mut steps = 0usize;
        for _ in 0..prep.m {
            steps += 1;
            j = edges[j as usize] & mask;
            if j == prep.mark {
                break;
            }
        }
        if steps == prep.m && j == prep.mark {
            single_cycle += 1;
        }
    }

    println!(
        "\n=== STRUCTURAL CENSUS ({} admitted Open blocks) ===",
        admitted.len()
    );
    println!(
        "total_n={} total_m={} (m/n={:.2}%) single_cycle={}/{} can_tag={}/{} wide={} oracle_ok={}/{} max_bits={} max_dict={}",
        total_n,
        total_m,
        100.0 * total_m as f64 / total_n as f64,
        single_cycle,
        admitted.len(),
        can_tag_cnt,
        admitted.len(),
        wide_cnt,
        oracle_cnt,
        admitted.len(),
        max_bits,
        max_dict
    );

    println!(
        "\n=== LOAD-BALANCE & LANE UTILIZATION CENSUS (K=4 lanes, SLAB={} KiB) ===",
        SLAB_SIZE / 1024
    );
    println!(
        "{:<16} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8} {:>9} {:>9} {:>9}",
        "scheme_S",
        "eff_K",
        "4L%",
        "3L%",
        "2L%",
        "1L%",
        "Lmax/(m/4)",
        "slices/b",
        "sw/blk",
        "max_arena"
    );

    let mut arena = Vec::new();
    let mut scratch = SampledScratch::default();
    let mut anchors = [0u32; MAX_ANCHORS];

    for (scheme_name, scheme) in [
        ("unif", AnchorScheme::Uniform),
        ("gold", AnchorScheme::Golden),
    ] {
        for s in [4usize, 8, 12, 16, 24, 32] {
            let mut sum_i4 = 0u64;
            let mut sum_i3 = 0u64;
            let mut sum_i2 = 0u64;
            let mut sum_i1 = 0u64;
            let mut sum_sw = 0u64;
            let mut sum_sl = 0u64;
            let mut max_ar = 0usize;
            let mut sum_lmax_ratio = 0.0f64;

            for prep in admitted {
                edges.resize(prep.m, 0);
                extra.resize(if prep.wide { prep.m } else { 0 }, 0);
                let cnt = wire_forward_sampled(
                    prep,
                    &prep.outq_bytes,
                    &mut edges,
                    &mut extra,
                    s,
                    scheme,
                    &mut anchors,
                );
                let st = emit_4lane_sampled::<true>(
                    prep,
                    &mut edges,
                    &extra,
                    &anchors[..cnt],
                    &mut arena,
                    &mut scratch,
                );
                sum_i4 += st.iters_4;
                sum_i3 += st.iters_3;
                sum_i2 += st.iters_2;
                sum_i1 += st.iters_1;
                sum_sw += st.slab_switches;
                sum_sl += st.ordered_slices;
                max_ar = max_ar.max(st.max_arena_used);
                sum_lmax_ratio += st.seg_max_steps as f64 / (prep.m as f64 / 4.0);
            }

            let tot_iters = (sum_i4 + sum_i3 + sum_i2 + sum_i1) as f64;
            let tot_steps = (4 * sum_i4 + 3 * sum_i3 + 2 * sum_i2 + sum_i1) as f64;
            let eff_k = tot_steps / tot_iters;
            println!(
                "{:<16} {:>7.3} {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}% {:>8.3}x {:>9.1} {:>9.1} {:>7.1}KiB",
                format!("{scheme_name}_S{s}"),
                eff_k,
                100.0 * sum_i4 as f64 / tot_iters,
                100.0 * sum_i3 as f64 / tot_iters,
                100.0 * sum_i2 as f64 / tot_iters,
                100.0 * sum_i1 as f64 / tot_iters,
                sum_lmax_ratio / admitted.len() as f64,
                sum_sl as f64 / admitted.len() as f64,
                sum_sw as f64 / admitted.len() as f64,
                max_ar as f64 / 1024.0
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Interleaved benchmark harness (warm + L3-cold/rotating-blocks)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ArmKind {
    ShippedBidi,
    Fwd1Lane,
    Fwd4Oracle,
    SampledVec(usize),
    SampledStitch(usize),
}

struct ArmSpec {
    name: &'static str,
    kind: ArmKind,
}

const ARMS: &[ArmSpec] = &[
    ArmSpec {
        name: "shipped_bidi",
        kind: ArmKind::ShippedBidi,
    },
    ArmSpec {
        name: "fwd_1lane",
        kind: ArmKind::Fwd1Lane,
    },
    ArmSpec {
        name: "fwd_4lane_oracle",
        kind: ArmKind::Fwd4Oracle,
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S4_vec",
        kind: ArmKind::SampledVec(4),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S8_vec",
        kind: ArmKind::SampledVec(8),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S12_vec",
        kind: ArmKind::SampledVec(12),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S16_vec",
        kind: ArmKind::SampledVec(16),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S24_vec",
        kind: ArmKind::SampledVec(24),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S8_stitch",
        kind: ArmKind::SampledStitch(8),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S12_stitch",
        kind: ArmKind::SampledStitch(12),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S16_stitch",
        kind: ArmKind::SampledStitch(16),
    },
    ArmSpec {
        name: "fwd_4lane_sampled_S24_stitch",
        kind: ArmKind::SampledStitch(24),
    },
];

#[derive(Default, Clone, Copy)]
struct Timing {
    a6c_ns: u64,
    a7_ns: u64,
    stitch_ns: u64,
}

impl Timing {
    fn total_ns(self) -> u64 {
        self.a6c_ns + self.a7_ns + self.stitch_ns
    }
}

#[derive(Default, Clone, Copy)]
struct DualTiming {
    cpu: Timing,
    wall: Timing,
}

/// Pure-safe steal-free thread CPU nanosecond timer via `/proc/thread-self/schedstat`
/// (field 0 = `se.sum_exec_runtime` in ns, excluding OS runnable-queue wait time).
/// Falls back to wall `Instant` if procfs is unavailable (e.g. macOS/Windows).
struct CpuClock {
    file: Option<std::fs::File>,
    origin: Instant,
}

impl CpuClock {
    fn new() -> Self {
        let file = std::fs::File::open("/proc/thread-self/schedstat")
            .or_else(|_| std::fs::File::open("/proc/self/schedstat"))
            .ok();
        CpuClock {
            file,
            origin: Instant::now(),
        }
    }

    #[inline(always)]
    fn now_ns(&mut self) -> u64 {
        if let Some(f) = self.file.as_mut() {
            use std::io::{Read, Seek, SeekFrom};
            if f.seek(SeekFrom::Start(0)).is_ok() {
                let mut buf = [0u8; 64];
                if let Ok(n) = f.read(&mut buf) {
                    let mut val = 0u64;
                    for &b in &buf[..n] {
                        if b.is_ascii_digit() {
                            val = val * 10 + u64::from(b - b'0');
                        } else {
                            break;
                        }
                    }
                    return val;
                }
            }
        }
        self.origin.elapsed().as_nanos() as u64
    }
}

fn flush_l3(buf: &mut [u8]) {
    let mut acc = 0u8;
    for b in buf.iter_mut().step_by(64) {
        *b = b.wrapping_add(1);
        acc ^= *b;
    }
    black_box(acc);
}

#[allow(clippy::too_many_arguments)]
fn run_arm_warm(
    arm: ArmKind,
    admitted: &[PreparedBlock],
    edges: &mut Vec<u32>,
    extra: &mut Vec<u32>,
    out: &mut Vec<u8>,
    arena: &mut Vec<u8>,
    scratch: &mut SampledScratch,
    clock: &mut CpuClock,
) -> DualTiming {
    let mut tm = DualTiming::default();
    let mut anchors = [0u32; MAX_ANCHORS];
    let mut acc = 0usize;

    for prep in admitted {
        edges.resize(prep.m, 0);
        extra.resize(if prep.wide { prep.m } else { 0 }, 0);
        let qlen = prep.outq_bytes.len();
        if out.len() < qlen.max(prep.n + INLINE) {
            out.resize(qlen.max(prep.n + INLINE), 0);
        }
        // Restore OutQ16 snapshot into `out` (untimed, warms `out` just like A6a)
        out[..qlen].copy_from_slice(&prep.outq_bytes);

        match arm {
            ArmKind::ShippedBidi => {
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let pred = wire_bidi(prep, out, edges, extra);
                let t1 = Instant::now();
                let c1 = clock.now_ns();
                emit_bidi(prep, edges, extra, pred, out);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                tm.wall.a6c_ns += t1.duration_since(t0).as_nanos() as u64;
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.cpu.a6c_ns += c1.saturating_sub(c0);
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::Fwd1Lane => {
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                wire_forward(prep, out, edges, extra);
                let t1 = Instant::now();
                let c1 = clock.now_ns();
                emit_fwd_1lane(prep, edges, extra, out);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                tm.wall.a6c_ns += t1.duration_since(t0).as_nanos() as u64;
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.cpu.a6c_ns += c1.saturating_sub(c0);
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::Fwd4Oracle => {
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                wire_forward(prep, out, edges, extra);
                let t1 = Instant::now();
                let c1 = clock.now_ns();
                emit_4lane_oracle(prep, edges, extra, out);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                tm.wall.a6c_ns += t1.duration_since(t0).as_nanos() as u64;
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.cpu.a6c_ns += c1.saturating_sub(c0);
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::SampledVec(s) => {
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let cnt = wire_forward_sampled(
                    prep,
                    out,
                    edges,
                    extra,
                    s,
                    AnchorScheme::Uniform,
                    &mut anchors,
                );
                let t1 = Instant::now();
                let c1 = clock.now_ns();
                emit_4lane_sampled::<false>(prep, edges, extra, &anchors[..cnt], out, scratch);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                tm.wall.a6c_ns += t1.duration_since(t0).as_nanos() as u64;
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.cpu.a6c_ns += c1.saturating_sub(c0);
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                let (off, _) = scratch.ordered[0];
                acc ^= usize::from(out[off]) ^ scratch.ordered.len();
            }
            ArmKind::SampledStitch(s) => {
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let cnt = wire_forward_sampled(
                    prep,
                    out,
                    edges,
                    extra,
                    s,
                    AnchorScheme::Uniform,
                    &mut anchors,
                );
                let t1 = Instant::now();
                let c1 = clock.now_ns();
                emit_4lane_sampled::<false>(prep, edges, extra, &anchors[..cnt], arena, scratch);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                stitch_slices(arena, &scratch.ordered, prep.n, out);
                let t3 = Instant::now();
                let c3 = clock.now_ns();
                tm.wall.a6c_ns += t1.duration_since(t0).as_nanos() as u64;
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.wall.stitch_ns += t3.duration_since(t2).as_nanos() as u64;
                tm.cpu.a6c_ns += c1.saturating_sub(c0);
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                tm.cpu.stitch_ns += c3.saturating_sub(c2);
                acc ^= usize::from(out[prep.n >> 1]);
            }
        }
    }
    black_box(acc);
    tm
}

#[allow(clippy::too_many_arguments)]
fn run_arm_cold(
    arm: ArmKind,
    admitted: &[PreparedBlock],
    cold_edges: &mut [Vec<u32>],
    cold_extra: &mut [Vec<u32>],
    cold_preds: &mut [Option<u32>],
    cold_anchors: &mut [([u32; MAX_ANCHORS], usize)],
    out: &mut Vec<u8>,
    arena: &mut Vec<u8>,
    scratch: &mut SampledScratch,
    l3_flush_buf: &mut [u8],
    clock: &mut CpuClock,
) -> DualTiming {
    let mut tm = DualTiming::default();
    let mut acc = 0usize;

    // Pass 1: L3-cold A6c across all 233 separate per-block buffers (>160 MiB >> 45 MiB L3)
    flush_l3(l3_flush_buf);
    let c0 = clock.now_ns();
    let t0 = Instant::now();
    for (i, prep) in admitted.iter().enumerate() {
        let edges = &mut cold_edges[i];
        let extra = &mut cold_extra[i];
        match arm {
            ArmKind::ShippedBidi => {
                cold_preds[i] = wire_bidi(prep, &prep.outq_bytes, edges, extra);
            }
            ArmKind::Fwd1Lane | ArmKind::Fwd4Oracle => {
                wire_forward(prep, &prep.outq_bytes, edges, extra);
            }
            ArmKind::SampledVec(s) | ArmKind::SampledStitch(s) => {
                let (anc, cnt) = &mut cold_anchors[i];
                *cnt = wire_forward_sampled(
                    prep,
                    &prep.outq_bytes,
                    edges,
                    extra,
                    s,
                    AnchorScheme::Uniform,
                    anc,
                );
            }
        }
    }
    tm.wall.a6c_ns = t0.elapsed().as_nanos() as u64;
    tm.cpu.a6c_ns = clock.now_ns().saturating_sub(c0);

    // Pass 2: L3-cold A7 (+ stitch if stitched) across all 233 L3-evicted per-block edge tables
    flush_l3(l3_flush_buf);
    for (i, prep) in admitted.iter().enumerate() {
        let edges = &mut cold_edges[i];
        let extra = &cold_extra[i];
        match arm {
            ArmKind::ShippedBidi => {
                let c1 = clock.now_ns();
                let t1 = Instant::now();
                emit_bidi(prep, edges, extra, cold_preds[i], out);
                tm.wall.a7_ns += t1.elapsed().as_nanos() as u64;
                tm.cpu.a7_ns += clock.now_ns().saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::Fwd1Lane => {
                let c1 = clock.now_ns();
                let t1 = Instant::now();
                emit_fwd_1lane(prep, edges, extra, out);
                tm.wall.a7_ns += t1.elapsed().as_nanos() as u64;
                tm.cpu.a7_ns += clock.now_ns().saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::Fwd4Oracle => {
                let c1 = clock.now_ns();
                let t1 = Instant::now();
                emit_4lane_oracle(prep, edges, extra, out);
                tm.wall.a7_ns += t1.elapsed().as_nanos() as u64;
                tm.cpu.a7_ns += clock.now_ns().saturating_sub(c1);
                acc ^= usize::from(out[prep.n >> 1]);
            }
            ArmKind::SampledVec(_) => {
                let (anc, cnt) = &cold_anchors[i];
                let c1 = clock.now_ns();
                let t1 = Instant::now();
                emit_4lane_sampled::<false>(prep, edges, extra, &anc[..*cnt], out, scratch);
                tm.wall.a7_ns += t1.elapsed().as_nanos() as u64;
                tm.cpu.a7_ns += clock.now_ns().saturating_sub(c1);
                let (off, _) = scratch.ordered[0];
                acc ^= usize::from(out[off]) ^ scratch.ordered.len();
            }
            ArmKind::SampledStitch(_) => {
                let (anc, cnt) = &cold_anchors[i];
                let c1 = clock.now_ns();
                let t1 = Instant::now();
                emit_4lane_sampled::<false>(prep, edges, extra, &anc[..*cnt], arena, scratch);
                let t2 = Instant::now();
                let c2 = clock.now_ns();
                stitch_slices(arena, &scratch.ordered, prep.n, out);
                let t3 = Instant::now();
                let c3 = clock.now_ns();
                tm.wall.a7_ns += t2.duration_since(t1).as_nanos() as u64;
                tm.wall.stitch_ns += t3.duration_since(t2).as_nanos() as u64;
                tm.cpu.a7_ns += c2.saturating_sub(c1);
                tm.cpu.stitch_ns += c3.saturating_sub(c2);
                acc ^= usize::from(out[prep.n >> 1]);
            }
        }
    }
    black_box(acc);
    tm
}

fn median_min_max(vals: &[f64]) -> (f64, f64, f64) {
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let med = s[s.len() / 2];
    (med, s[0], s[s.len() - 1])
}

fn print_bench_table(label: &str, results: &[Vec<Timing>], show_i9: bool) {
    println!("\n=== {label} (ms across 233 admitted blocks; median [min..max]) ===");
    println!(
        "{:<30} {:>7} {:>7} {:>7} {:>24} {:>8} {:>8} {:>9} {:>9}",
        "arm",
        "A6c",
        "A7",
        "stitch",
        "A6c+A7 med [min..max]",
        "vs_bidi",
        "vs_fwd1",
        "disj_bi",
        "disj_f1"
    );

    let totals: Vec<Vec<f64>> = results
        .iter()
        .map(|v| v.iter().map(|t| t.total_ns() as f64 / 1e6).collect())
        .collect();
    let (bidi_med, bidi_min, bidi_max) = median_min_max(&totals[0]);
    let (fwd1_med, fwd1_min, fwd1_max) = median_min_max(&totals[1]);

    for (idx, spec) in ARMS.iter().enumerate() {
        let a6cs: Vec<f64> = results[idx].iter().map(|t| t.a6c_ns as f64 / 1e6).collect();
        let a7s: Vec<f64> = results[idx].iter().map(|t| t.a7_ns as f64 / 1e6).collect();
        let sts: Vec<f64> = results[idx]
            .iter()
            .map(|t| t.stitch_ns as f64 / 1e6)
            .collect();
        let (a6c_med, _, _) = median_min_max(&a6cs);
        let (a7_med, _, _) = median_min_max(&a7s);
        let (st_med, _, _) = median_min_max(&sts);
        let (tot_med, tot_min, tot_max) = median_min_max(&totals[idx]);

        let d_bidi = (tot_med / bidi_med - 1.0) * 100.0;
        let d_fwd1 = (tot_med / fwd1_med - 1.0) * 100.0;
        let disj_bidi = if idx == 0 {
            "base"
        } else if tot_max < bidi_min {
            "FASTER"
        } else if tot_min > bidi_max {
            "SLOWER"
        } else {
            "overlap"
        };
        let disj_fwd1 = if idx == 1 {
            "base"
        } else if tot_max < fwd1_min {
            "FASTER"
        } else if tot_min > fwd1_max {
            "SLOWER"
        } else {
            "overlap"
        };
        println!(
            "{:<30} {:>7.2} {:>7.2} {:>7.2} {:>7.2} [{:>6.2}..{:>6.2}] {:>+7.2}% {:>+7.2}% {:>9} {:>9}",
            spec.name,
            a6c_med,
            a7_med,
            st_med,
            tot_med,
            tot_min,
            tot_max,
            d_bidi,
            d_fwd1,
            disj_bidi,
            disj_fwd1
        );
    }

    if show_i9 {
        println!("\n--- Modeled i9-10900K A6c+A7 projections (anchored to 161.4 + 490.3 = 651.7 ms of 1,306 ms 1W wall; VM diagnostic) ---");
        let (bidi_a6c, _, _) = median_min_max(
            &results[0]
                .iter()
                .map(|t| t.a6c_ns as f64 / 1e6)
                .collect::<Vec<_>>(),
        );
        let (bidi_a7, _, _) = median_min_max(
            &results[0]
                .iter()
                .map(|t| t.a7_ns as f64 / 1e6)
                .collect::<Vec<_>>(),
        );
        for (idx, spec) in ARMS.iter().enumerate() {
            let (a6c_med, _, _) = median_min_max(
                &results[idx]
                    .iter()
                    .map(|t| t.a6c_ns as f64 / 1e6)
                    .collect::<Vec<_>>(),
            );
            let (a7s_med, _, _) = median_min_max(
                &results[idx]
                    .iter()
                    .map(|t| (t.a7_ns + t.stitch_ns) as f64 / 1e6)
                    .collect::<Vec<_>>(),
            );
            let (tot_med, _, _) = median_min_max(&totals[idx]);
            let i9_anc = I9_A6C_MS * (a6c_med / bidi_a6c) + I9_A7_MS * (a7s_med / bidi_a7);
            let i9_anc_save = I9_A6C_A7_MS - i9_anc;
            let i9_prp_save = I9_A6C_A7_MS * (1.0 - tot_med / bidi_med);
            println!(
                "  {:<30} i9_anc={:>6.1} ms (save {:>+6.1} ms = {:>+5.2}% 1W)  i9_prp_save={:>+6.1} ms ({:>+5.2}% 1W)",
                spec.name,
                i9_anc,
                i9_anc_save,
                100.0 * i9_anc_save / I9_WALL_1W_MS,
                i9_prp_save,
                100.0 * i9_prp_save / I9_WALL_1W_MS
            );
        }
    }
}

fn run_bench(admitted: &[PreparedBlock], repeats: usize) {
    let mut edges = Vec::new();
    let mut extra = Vec::new();
    let mut out = Vec::with_capacity(900_000 + 4 * SLAB_SIZE + 4 * INLINE);
    let mut arena = Vec::with_capacity(900_000 + 4 * SLAB_SIZE + 4 * INLINE);
    let mut scratch = SampledScratch::default();
    let mut clock = CpuClock::new();

    // Warm-up pass
    for spec in ARMS {
        run_arm_warm(
            spec.kind,
            admitted,
            &mut edges,
            &mut extra,
            &mut out,
            &mut arena,
            &mut scratch,
            &mut clock,
        );
    }

    // 1. Warm interleaved benchmark (L2-cold / L3-hot per-block, reused worker buffers)
    let mut warm_cpu: Vec<Vec<Timing>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    let mut warm_wall: Vec<Vec<Timing>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    for rep in 0..repeats {
        let rev = rep % 2 == 1;
        for step in 0..ARMS.len() {
            let idx = if rev { ARMS.len() - 1 - step } else { step };
            let tm = run_arm_warm(
                ARMS[idx].kind,
                admitted,
                &mut edges,
                &mut extra,
                &mut out,
                &mut arena,
                &mut scratch,
                &mut clock,
            );
            warm_cpu[idx].push(tm.cpu);
            warm_wall[idx].push(tm.wall);
        }
    }
    print_bench_table(
        "WARM CPU (steal-free schedstat, L2-cold/L3-hot reused buffers)",
        &warm_cpu,
        true,
    );
    print_bench_table(
        "WARM WALL (Instant, L2-cold/L3-hot reused buffers)",
        &warm_wall,
        false,
    );

    // 2. Cold interleaved benchmark (233 separate per-block buffers > 160 MiB >> 45 MiB L3 + 64 MiB flush)
    let mut cold_edges: Vec<Vec<u32>> = admitted.iter().map(|b| vec![0u32; b.m]).collect();
    let mut cold_extra: Vec<Vec<u32>> = admitted
        .iter()
        .map(|b| if b.wide { vec![0u32; b.m] } else { Vec::new() })
        .collect();
    let mut cold_preds: Vec<Option<u32>> = vec![None; admitted.len()];
    let mut cold_anchors: Vec<([u32; MAX_ANCHORS], usize)> =
        vec![([0u32; MAX_ANCHORS], 0); admitted.len()];
    let mut l3_flush_buf = vec![0u8; COLD_FLUSH_BYTES];

    let mut cold_cpu: Vec<Vec<Timing>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    let mut cold_wall: Vec<Vec<Timing>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    for rep in 0..repeats {
        let rev = rep % 2 == 1;
        for step in 0..ARMS.len() {
            let idx = if rev { ARMS.len() - 1 - step } else { step };
            let tm = run_arm_cold(
                ARMS[idx].kind,
                admitted,
                &mut cold_edges,
                &mut cold_extra,
                &mut cold_preds,
                &mut cold_anchors,
                &mut out,
                &mut arena,
                &mut scratch,
                &mut l3_flush_buf,
                &mut clock,
            );
            cold_cpu[idx].push(tm.cpu);
            cold_wall[idx].push(tm.wall);
        }
    }
    print_bench_table(
        "L3-COLD CPU (steal-free schedstat, rotating 233 buffers >160MiB + 64MiB flush)",
        &cold_cpu,
        true,
    );
    print_bench_table(
        "L3-COLD WALL (Instant, rotating 233 buffers >160MiB + 64MiB flush)",
        &cold_wall,
        false,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let path = args
        .get(2)
        .filter(|s| !s.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("/tmp/wikidata.bz2");
    let mut repeats = 7usize;
    for i in 1..args.len() {
        if args[i] == "--repeats" {
            if let Some(v) = args.get(i + 1) {
                repeats = v.parse().expect("invalid --repeats");
            }
        }
    }

    let target_avx2 = cfg!(target_feature = "avx2");
    println!(
        "forward_4lane_probe mode={mode} corpus={path} repeats={repeats} target_avx2={target_avx2} SLAB={}KiB",
        SLAB_SIZE / 1024
    );

    // 1. Always verify synthetic edge cases first
    let syn_raw = synthetic_edge_blocks();
    let mut builder = OpenBuilder::default();
    let mut scratch_out = Vec::new();
    let syn_preps: Vec<PreparedBlock> = syn_raw
        .into_iter()
        .map(|b| {
            let (mut prep, _, _) =
                builder.prepare_block(&b.records, b.n, b.p, PROD_LIMIT, &mut scratch_out);
            prep.expected = b.walked;
            prep
        })
        .collect();
    verify_exact_all(&syn_preps, "synthetic_edge_cases");

    // 2. Load ALL 431 blocks of wikidata.bz2
    let raw_blocks = load_bz2_blocks(path);
    let mut admitted_preps = Vec::new();
    let mut non_admitted_preps = Vec::new();
    let (mut sum_a6a_ns, mut sum_a6b_ns) = (0u64, 0u64);
    for b in raw_blocks {
        let (mut prep, a6a, a6b) =
            builder.prepare_block(&b.records, b.n, b.p, PROD_LIMIT, &mut scratch_out);
        prep.expected = b.walked;
        if b.admitted {
            sum_a6a_ns += a6a;
            sum_a6b_ns += a6b;
            admitted_preps.push(prep);
        } else {
            non_admitted_preps.push(prep);
        }
    }
    println!(
        "Loaded {} total blocks: {} production-admitted Open + {} non-admitted (forced exact check). One-pass A6a={:.2}ms A6b={:.2}ms.",
        admitted_preps.len() + non_admitted_preps.len(),
        admitted_preps.len(),
        non_admitted_preps.len(),
        sum_a6a_ns as f64 / 1e6,
        sum_a6b_ns as f64 / 1e6
    );

    verify_exact_all(&admitted_preps, "wikidata_admitted_233");
    verify_exact_all(&non_admitted_preps, "wikidata_non_admitted_198");

    if mode == "exact" {
        return;
    }
    if mode == "census" || mode == "all" {
        run_census(&admitted_preps);
    }
    if mode == "bench" || mode == "all" {
        run_bench(&admitted_preps, repeats);
    }
}
