//! Spelling census: where emission's spelling of codes into bytes (`E_consume`) spends its time
//! on real blocks (wikidata, wikidata x4, ab gen corpora).
//!
//! Characterisation only; zero changes to `src/`, zero `unsafe`.
//!
//! Usage:
//!   spelling_census stats FILE.bz2 [--forced]
//!   spelling_census bench FILE.bz2 [--forced] [--repeats N]

use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::collections::HashMap;
use std::hint::black_box;
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

#[inline(always)]
fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    &spelling[start..end]
}

#[inline(always)]
fn long_bounds(slot: &[u8; INLINE]) -> (usize, usize) {
    let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
    let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
    (start, end)
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
            for (e, s) in dst[..take]
                .iter_mut()
                .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }
            self.at += take;
            self.read += take as u32;
            dst = &mut dst[take..];
        }
    }
}

// ---------------------------------------------------------------------------
// First pass + endpoint elimination (mirroring src/open.rs & src/open/endpoint.rs)
// ---------------------------------------------------------------------------

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
}

struct BuiltBlock {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    predecessor: Option<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    codes: Vec<u32>,
    walked: Vec<u8>,
    // Pre-built alternative representations (built outside the clock for representation probes)
    inline_back: Vec<[u8; INLINE]>,
    packed15: Vec<[u8; 16]>,
    inline32: Vec<[u8; 32]>,
    lens32: Vec<u8>,
    direct_meta: Vec<(u32, u32)>,
    exact_cycle: bool,
    // Frequency-sorted dictionary & remapped codes
    freq_codes: Vec<u32>,
    freq_inline16: Vec<[u8; INLINE]>,
    freq_lens: Vec<u8>,
    // Branch-ablation chunked pairs: 256-code chunks (real mixed order vs inline-before-long sorted)
    chunk_all: Vec<Vec<(u32, u32)>>,
    chunk_sorted: Vec<Vec<(u32, u32)>>,
    // Greedy fused <=16B super-words upper bound (simulating fewer, longer words)
    fused_codes: Vec<u32>,
    fused_inline16: Vec<[u8; INLINE]>,
    fused_lens: Vec<u8>,
}

impl OpenMirror {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn arena_words(ns: usize, m: usize) -> usize {
        let most = Self::nodes_most(ns);
        let stride = if most > NARROW_NODES {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * most).max(m)
    }

    fn construct_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(usize, u32, u32)> {
        let n32 = n as u32;
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
        if over || f.m > cap.m || ns > cap.strips {
            return None;
        }
        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark,
        } = f;
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        assert!(m > 0 && all.iter().sum::<u32>() == n32);
        let first = starts(all);
        let mut write = starts(thick);
        let need = Self::arena_words(ns, m as usize);
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
        let most = Self::nodes_most(ns);
        let cursors = if most > NARROW_NODES {
            self.induced_with::<Wide>(ns, n32, &thin, m, most)
        } else {
            self.induced_with::<Narrow>(ns, n32, &thin, m, most)
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
        Some((m as usize, mark, bits))
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        m: u32,
        most: usize,
    ) -> [usize; 256] {
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
        e.node(0, 0);
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
            e.append_to(&mut ord[0], i as u32 + 1, 0);
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
            e.append_to(&mut ord[1], i + 1, 1);
            x = s.to + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut ord[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        for side in 0..2 {
            e.close(side, terminals[side], ord[side]);
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
            grow(edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut edges[4 * ns..],
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
        assert_eq!(id, 0);
        cursors
    }
}

fn cycle_codes_slices(
    predecessor: Option<u32>,
    edges: &[u32],
    extra: &[u32],
    mark: u32,
    bits: u32,
) -> Vec<u32> {
    let m = edges.len();
    let mask = (1u32 << bits) - 1;
    let mut codes = Vec::with_capacity(m);
    if let Some(before) = predecessor {
        let (mut prev, mut j) = (before, mark);
        for _ in 0..m {
            let e = edges[j as usize];
            codes.push(e >> bits);
            (prev, j) = (j, (e & mask) ^ prev);
            if j == mark {
                break;
            }
        }
    } else {
        let mut j = mark;
        for _ in 0..m {
            let e = edges[j as usize];
            codes.push(if extra.is_empty() {
                e >> bits
            } else {
                extra[j as usize]
            });
            j = e & mask;
            if j == mark {
                break;
            }
        }
    }
    codes
}

fn word_len(code: usize, inline16: &[[u8; INLINE]], lens: &[u8]) -> usize {
    let l = lens[code];
    if l != LONG {
        usize::from(l)
    } else {
        let (s, e) = long_bounds(&inline16[code]);
        e - s
    }
}

fn word_bytes<'a>(
    code: usize,
    inline16: &'a [[u8; INLINE]],
    lens: &[u8],
    spelling: &'a [u8],
) -> &'a [u8] {
    let l = lens[code];
    if l != LONG {
        &inline16[code][..usize::from(l)]
    } else {
        long_word(&inline16[code], spelling)
    }
}

#[allow(clippy::too_many_arguments)]
fn build_block_variants(
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    predecessor: Option<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    walked: Vec<u8>,
) -> BuiltBlock {
    let codes = cycle_codes_slices(predecessor, &edges, &extra, mark, bits);
    let dict_len = lens.len();

    let mut total_spelled = 0usize;
    let mut counts = vec![0u32; dict_len];
    for &c in &codes {
        let cu = c as usize;
        counts[cu] += 1;
        total_spelled += word_len(cu, &inline16, &lens);
    }
    let exact_cycle = total_spelled == n;

    // Pre-right-aligned inline words for bidirectional back-side emission (avoids runtime u128 shift)
    let mut inline_back = Vec::with_capacity(dict_len);
    for c in 0..dict_len {
        let l = lens[c];
        if l != LONG {
            let shifted = u128::from_le_bytes(inline16[c]) << (8 * (INLINE - usize::from(l)));
            inline_back.push(shifted.to_le_bytes());
        } else {
            inline_back.push(inline16[c]);
        }
    }

    // Single-array 16B slot: bytes 0..15 hold payload and byte 15 holds len (1..=15),
    // 254 for len == 16 (falls back to inline16), or LONG (255) with (start, end) in 0..8.
    let mut packed15 = Vec::with_capacity(dict_len);
    for c in 0..dict_len {
        let l = lens[c];
        let mut slot = inline16[c];
        if l != LONG && l <= 15 {
            slot[15] = l;
        } else if l == 16 {
            slot[15] = 254;
        } else {
            slot[15] = LONG;
        }
        packed15.push(slot);
    }

    // inline32: words <= 32 bytes stored inline in [u8; 32], > 32 bytes use long_slot in 0..8
    let mut inline32 = Vec::with_capacity(dict_len);
    let mut lens32 = Vec::with_capacity(dict_len);
    for c in 0..dict_len {
        let wb = word_bytes(c, &inline16, &lens, &spelling);
        let mut slot = [0u8; 32];
        if wb.len() <= 32 {
            slot[..wb.len()].copy_from_slice(wb);
            lens32.push(wb.len() as u8);
        } else {
            let (s, e) = long_bounds(&inline16[c]);
            slot[..4].copy_from_slice(&(s as u32).to_le_bytes());
            slot[4..8].copy_from_slice(&(e as u32).to_le_bytes());
            lens32.push(LONG);
        }
        inline32.push(slot);
    }

    // direct_meta: pre-unpacked (start, len) for every code (avoids byte-unpack + sub on long words)
    let mut direct_meta = Vec::with_capacity(dict_len);
    for c in 0..dict_len {
        if lens[c] == LONG {
            let (s, e) = long_bounds(&inline16[c]);
            direct_meta.push((s as u32, (e - s) as u32));
        } else {
            direct_meta.push((0, u32::from(lens[c])));
        }
    }

    // Frequency-sorted dictionary (using exact frequency from the cycle, which A6b already knows
    // from band widths for full-cycle blocks!)
    let mut order: Vec<usize> = (0..dict_len).collect();
    order.sort_unstable_by(|&a, &b| counts[b].cmp(&counts[a]).then_with(|| a.cmp(&b)));
    let mut remap = vec![0u32; dict_len];
    let mut freq_inline16 = Vec::with_capacity(dict_len);
    let mut freq_lens = Vec::with_capacity(dict_len);
    for (new_id, &old_id) in order.iter().enumerate() {
        remap[old_id] = new_id as u32;
        freq_inline16.push(inline16[old_id]);
        freq_lens.push(lens[old_id]);
    }
    let freq_codes: Vec<u32> = codes.iter().map(|&c| remap[c as usize]).collect();

    // Chunked 256-code pairs for L1d-matched, same-loop branch-misprediction ablation
    const BR_CHUNK: usize = 256;
    let n_chunks = codes.len().div_ceil(BR_CHUNK);
    let mut chunk_all = Vec::with_capacity(n_chunks);
    let mut chunk_sorted = Vec::with_capacity(n_chunks);
    let mut at = 0usize;
    for ch in codes.chunks(BR_CHUNK) {
        let mut all_p = Vec::with_capacity(ch.len());
        let mut inl_p = Vec::with_capacity(ch.len());
        let mut lng_p = Vec::new();
        for &c in ch {
            if at >= n {
                break;
            }
            let cu = c as usize;
            let l = lens[cu];
            let pair = (at as u32, c);
            all_p.push(pair);
            if l != LONG {
                inl_p.push(pair);
                at += usize::from(l);
            } else {
                lng_p.push(pair);
                let (s, e) = long_bounds(&inline16[cu]);
                at += (e - s).min(n - at);
            }
        }
        inl_p.extend_from_slice(&lng_p);
        chunk_all.push(all_p);
        chunk_sorted.push(inl_p);
    }

    // Greedy fused <=16B super-words upper bound (only if exact_cycle; else keep original codes)
    let (fused_codes, fused_inline16, fused_lens) = if exact_cycle {
        let mut f_inline: Vec<[u8; INLINE]> = inline16.clone();
        let mut f_lens: Vec<u8> = lens.clone();
        let mut f_map: HashMap<([u8; INLINE], u8), u32> = HashMap::new();
        for c in 0..dict_len {
            if f_lens[c] != LONG {
                f_map.insert((f_inline[c], f_lens[c]), c as u32);
            }
        }
        let mut f_codes = Vec::with_capacity(codes.len());
        let mut i = 0;
        while i < codes.len() {
            let c0 = codes[i] as usize;
            let l0 = lens[c0];
            if l0 == LONG {
                f_codes.push(codes[i]);
                i += 1;
                continue;
            }
            let mut buf = inline16[c0];
            let mut cur_len = usize::from(l0);
            let mut j = i + 1;
            while j < codes.len() {
                let cj = codes[j] as usize;
                let lj = lens[cj];
                if lj == LONG || cur_len + usize::from(lj) > INLINE {
                    break;
                }
                let lju = usize::from(lj);
                buf[cur_len..cur_len + lju].copy_from_slice(&inline16[cj][..lju]);
                cur_len += lju;
                j += 1;
            }
            if j == i + 1 {
                f_codes.push(codes[i]);
            } else {
                let key = (buf, cur_len as u8);
                let id = *f_map.entry(key).or_insert_with(|| {
                    let id = f_lens.len() as u32;
                    f_inline.push(buf);
                    f_lens.push(cur_len as u8);
                    id
                });
                f_codes.push(id);
            }
            i = j;
        }
        (f_codes, f_inline, f_lens)
    } else {
        (codes.clone(), inline16.clone(), lens.clone())
    };

    BuiltBlock {
        n,
        m,
        mark,
        bits,
        predecessor,
        edges,
        extra,
        spelling,
        inline16,
        lens,
        codes,
        walked,
        inline_back,
        packed15,
        inline32,
        lens32,
        direct_meta,
        exact_cycle,
        freq_codes,
        freq_inline16,
        freq_lens,
        chunk_all,
        chunk_sorted,
        fused_codes,
        fused_inline16,
        fused_lens,
    }
}

// ---------------------------------------------------------------------------
// Production emitters & spelling mirrors (verbatim src/open.rs & #921)
// ---------------------------------------------------------------------------

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_prod_uni(
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
fn emit_prod_bidi(
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

fn emit_codes_uni(
    codes: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
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
        if at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

fn emit_codes_bidi(
    codes: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let (mut lo, mut hi) = (0, n);
    let (mut f, mut b) = (0usize, codes.len() - 1);
    for _ in 0..=codes.len() / 2 {
        if lo == hi || f > b {
            break;
        }
        let cf = codes[f] as usize;
        let single = f == b;
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
        let cb = codes[b] as usize;
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
        f += 1;
        b -= 1;
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

// ---------------------------------------------------------------------------
// Ablation & representation probe kernels
// ---------------------------------------------------------------------------

fn emit_codes_bidi_pre_shifted(
    codes: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    inline_back: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let (mut lo, mut hi) = (0, n);
    let (mut f, mut b) = (0usize, codes.len() - 1);
    for _ in 0..=codes.len() / 2 {
        if lo == hi || f > b {
            break;
        }
        let cf = codes[f] as usize;
        let single = f == b;
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
        let cb = codes[b] as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            out[hi - INLINE..hi].copy_from_slice(&inline_back[cb]);
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
        f += 1;
        b -= 1;
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

fn emit_codes_bidi_no_shift_ablat(
    codes: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    out.resize(n + INLINE, 0);
    let (mut lo, mut hi) = (0, n);
    let (mut f, mut b) = (0usize, codes.len() - 1);
    for _ in 0..=codes.len() / 2 {
        if lo == hi || f > b {
            break;
        }
        let cf = codes[f] as usize;
        let single = f == b;
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
        let cb = codes[b] as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            out[hi - INLINE..hi].copy_from_slice(&inline[cb]);
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
        f += 1;
        b -= 1;
    }
    lo + (n - hi)
}

fn emit_inline_only_skip_long(
    codes: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let (s, e) = long_bounds(&inline16[code]);
            at += (e - s).min(n - at);
        }
        if at >= n {
            break;
        }
    }
    at
}

fn emit_long_only_skip_inline(
    codes: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if at >= n {
            break;
        }
    }
    at
}

fn emit_no_copy_length_only(
    codes: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
) -> usize {
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            at += usize::from(len);
        } else {
            let (s, e) = long_bounds(&inline16[code]);
            at += (e - s).min(n - at);
        }
        if at >= n {
            break;
        }
    }
    at
}

fn emit_codes_scan_only(codes: &[u32]) -> u64 {
    let mut acc = 0u64;
    for &c in codes {
        acc = acc.wrapping_add(u64::from(c));
    }
    acc
}

fn emit_dict_l1_pinned(
    codes: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    out.resize(n + INLINE, 0);
    let hot_inline = inline16[0];
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&hot_inline);
            at += usize::from(len);
        } else {
            let (s, e) = long_bounds(&inline16[code]);
            let take = (e - s).min(n - at).min(spelling.len());
            out[at..at + take].copy_from_slice(&spelling[..take]);
            at += take;
        }
        if at >= n {
            break;
        }
    }
    at
}

fn emit_out_l1_pinned(
    codes: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    let cap_needed = 4096 + n.min(65_536) + INLINE;
    if out.len() < cap_needed {
        out.resize(cap_needed, 0);
    }
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            let dst = at & 4095;
            out[dst..dst + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            let dst = if take <= 4096 { at & 4095 } else { 0 };
            if dst + take <= out.len() {
                out[dst..dst + take].copy_from_slice(&word[..take]);
            }
            at += take;
        }
        if at >= n {
            break;
        }
    }
    at
}

#[inline(never)]
fn emit_chunk_pairs(
    chunks: &[Vec<(u32, u32)>],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) -> usize {
    out.resize(n + INLINE, 0);
    let mut written = 0usize;
    for ch in chunks {
        for &(at_u32, c) in ch {
            let at = at_u32 as usize;
            let code = c as usize;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
            }
            written += 1;
        }
    }
    written
}

fn emit_chunk_mixed_pairs(b: &BuiltBlock, out: &mut Vec<u8>) {
    emit_chunk_pairs(&b.chunk_all, &b.spelling, &b.inline16, &b.lens, b.n, out);
    out.truncate(b.n);
    if !b.exact_cycle {
        let period = b
            .chunk_all
            .last()
            .and_then(|c| c.last())
            .map(|&(at, code)| {
                let l = word_len(code as usize, &b.inline16, &b.lens);
                (at as usize + l).min(b.n)
            })
            .unwrap_or(b.n);
        out.truncate(period);
        repeat(b.n, out);
    }
}

fn emit_chunk_sorted_pairs(b: &BuiltBlock, out: &mut Vec<u8>) -> usize {
    emit_chunk_pairs(&b.chunk_sorted, &b.spelling, &b.inline16, &b.lens, b.n, out)
}

fn emit_packed15(b: &BuiltBlock, out: &mut Vec<u8>) {
    out.resize(b.n + INLINE, 0);
    let mut at = 0;
    for &c in &b.codes {
        let code = c as usize;
        let slot = &b.packed15[code];
        let len = slot[15];
        if len < 254 {
            out[at..at + INLINE].copy_from_slice(slot);
            at += usize::from(len);
        } else if len == 254 {
            out[at..at + INLINE].copy_from_slice(&b.inline16[code]);
            at += INLINE;
        } else {
            let word = long_word(slot, &b.spelling);
            let take = word.len().min(b.n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if at >= b.n {
            break;
        }
    }
    out.truncate(at.min(b.n));
    repeat(b.n, out);
}

fn emit_inline32(b: &BuiltBlock, out: &mut Vec<u8>) {
    out.resize(b.n + 32, 0);
    let mut at = 0;
    for &c in &b.codes {
        let code = c as usize;
        let len = b.lens32[code];
        if len != LONG {
            out[at..at + 32].copy_from_slice(&b.inline32[code]);
            at += usize::from(len);
        } else {
            let slot = &b.inline32[code];
            let s = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
            let e = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
            let word = &b.spelling[s..e];
            let take = word.len().min(b.n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if at >= b.n {
            break;
        }
    }
    out.truncate(at.min(b.n));
    repeat(b.n, out);
}

fn emit_direct_no_clamp(b: &BuiltBlock, out: &mut Vec<u8>) {
    if !b.exact_cycle {
        emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
        return;
    }
    out.resize(b.n + INLINE, 0);
    let mut at = 0;
    for &c in &b.codes {
        let code = c as usize;
        let len = b.lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&b.inline16[code]);
            at += usize::from(len);
        } else {
            let (s, l) = b.direct_meta[code];
            let su = s as usize;
            let lu = l as usize;
            out[at..at + lu].copy_from_slice(&b.spelling[su..su + lu]);
            at += lu;
        }
    }
    out.truncate(b.n);
}

fn emit_unrolled_4x(b: &BuiltBlock, out: &mut Vec<u8>) {
    if !b.exact_cycle {
        emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
        return;
    }
    out.resize(b.n + 4 * INLINE, 0);
    let mut at = 0;
    let chunks = b.codes.chunks_exact(4);
    let rem = chunks.remainder();
    for ch in chunks {
        let (c0, c1, c2, c3) = (
            ch[0] as usize,
            ch[1] as usize,
            ch[2] as usize,
            ch[3] as usize,
        );
        let (l0, l1, l2, l3) = (b.lens[c0], b.lens[c1], b.lens[c2], b.lens[c3]);
        if ((l0 | l1 | l2 | l3) & 0x80) == 0 {
            let a0 = at;
            let a1 = a0 + usize::from(l0);
            let a2 = a1 + usize::from(l1);
            let a3 = a2 + usize::from(l2);
            out[a0..a0 + INLINE].copy_from_slice(&b.inline16[c0]);
            out[a1..a1 + INLINE].copy_from_slice(&b.inline16[c1]);
            out[a2..a2 + INLINE].copy_from_slice(&b.inline16[c2]);
            out[a3..a3 + INLINE].copy_from_slice(&b.inline16[c3]);
            at = a3 + usize::from(l3);
        } else {
            for (code, len) in [(c0, l0), (c1, l1), (c2, l2), (c3, l3)] {
                if len != LONG {
                    out[at..at + INLINE].copy_from_slice(&b.inline16[code]);
                    at += usize::from(len);
                } else {
                    let word = long_word(&b.inline16[code], &b.spelling);
                    let lu = word.len();
                    out[at..at + lu].copy_from_slice(word);
                    at += lu;
                }
            }
        }
    }
    for &c in rem {
        let code = c as usize;
        let len = b.lens[code];
        if len != LONG {
            out[at..at + INLINE].copy_from_slice(&b.inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&b.inline16[code], &b.spelling);
            let lu = word.len();
            out[at..at + lu].copy_from_slice(word);
            at += lu;
        }
    }
    out.truncate(b.n);
}

// ---------------------------------------------------------------------------
// Corpus loading & helpers
// ---------------------------------------------------------------------------

struct BlockInput {
    n: usize,
    orig_ptr: u32,
    records: Vec<u16>,
    cap: Cap,
    walked: Vec<u8>,
}

fn load_blocks(path: &str, forced: bool, keep_walked: bool) -> (Vec<BlockInput>, usize, usize) {
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

    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        let n = ll.len();
        total_blocks += 1;
        total_rows += n;
        let recs = records_of(&ll);
        if let Some(cap) = open_cap(recs.len(), n, b.orig_ptr, forced) {
            let mut walked = Vec::new();
            if keep_walked {
                build(&ll, &counts, &mut tt);
                walk(&tt, b.orig_ptr, &mut walked);
            }
            blocks.push(BlockInput {
                n,
                orig_ptr: b.orig_ptr,
                records: recs,
                cap,
                walked,
            });
        }
        syms = b.syms;
    }
    (blocks, total_blocks, total_rows)
}

fn build_corpus(path: &str, forced: bool) -> (Vec<BuiltBlock>, usize, usize) {
    let (inputs, total_blocks, total_rows) = load_blocks(path, forced, true);
    let mut kernel = OpenMirror::default();
    let mut out = Vec::new();
    let mut built = Vec::with_capacity(inputs.len());
    for blk in inputs {
        let Some((m, mark, bits)) =
            kernel.construct_block(&blk.records, blk.n, blk.orig_ptr, blk.cap, &mut out)
        else {
            continue;
        };
        let edges = kernel.edges[..m].to_vec();
        let extra = kernel.extra.clone();
        let spelling = kernel.spelling.clone();
        let inline16 = kernel.inline16.clone();
        let lens = kernel.lens.clone();
        let predecessor = kernel.predecessor;

        let bb = build_block_variants(
            blk.n,
            m,
            mark,
            bits,
            predecessor,
            edges,
            extra,
            spelling,
            inline16,
            lens,
            blk.walked,
        );
        built.push(bb);
    }
    (built, total_blocks, total_rows)
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

fn pct_f64(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn pct_usize(sorted: &[usize], p: f64) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn weighted_percentile(hist: &[(usize, u64)], total: u64, p: f64) -> usize {
    if hist.is_empty() || total == 0 {
        return 0;
    }
    let target = ((total as f64) * p).ceil() as u64;
    let mut acc = 0u64;
    for &(len, cnt) in hist {
        acc += cnt;
        if acc >= target {
            return len;
        }
    }
    hist.last().unwrap().0
}

fn len_bucket(len: usize) -> usize {
    match len {
        1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=8 => 3,
        9..=16 => 4,
        17..=32 => 5,
        33..=64 => 6,
        65..=128 => 7,
        129..=256 => 8,
        257..=1024 => 9,
        _ => 10,
    }
}

const BUCKET_NAMES: [&str; 11] = [
    "1",
    "2",
    "3..4",
    "5..8",
    "9..16",
    "17..32",
    "33..64",
    "65..128",
    "129..256",
    "257..1024",
    ">1024",
];

fn reuse_bucket(cnt: u32) -> usize {
    match cnt {
        1 => 0,
        2..=4 => 1,
        5..=16 => 2,
        17..=64 => 3,
        65..=256 => 4,
        _ => 5,
    }
}

const REUSE_NAMES: [&str; 6] = ["1 (one-shot)", "2..4", "5..16", "17..64", "65..256", ">256"];

// ---------------------------------------------------------------------------
// Command: stats (structural census)
// ---------------------------------------------------------------------------

fn cmd_stats(path: &str, forced: bool) {
    let (blocks, total_blocks, total_rows) = build_corpus(path, forced);
    println!(
        "# spelling stats: {path} (forced={forced}, total_blocks={total_blocks}, total_rows={total_rows}, open_blocks={})",
        blocks.len()
    );
    if blocks.is_empty() {
        println!("NO_ADMITTED_BLOCKS");
        return;
    }

    // Verify every block byte-identical against bwt::walk via production mirror
    let mut out = Vec::new();
    for (idx, b) in blocks.iter().enumerate() {
        if let Some(before) = b.predecessor {
            emit_prod_bidi(
                &b.edges,
                &b.spelling,
                &b.inline16,
                &b.lens,
                b.n,
                b.mark,
                before,
                b.bits,
                &mut out,
            );
        } else {
            emit_prod_uni(
                &b.edges,
                &b.extra,
                &b.spelling,
                &b.inline16,
                &b.lens,
                b.n,
                b.mark,
                b.bits,
                &mut out,
            );
        }
        assert_eq!(out, b.walked, "prod emit mismatch on block {idx}");
    }

    let mut bidi_blocks = 0usize;
    let mut exact_cycle_blocks = 0usize;
    let mut total_n = 0u64;
    let mut total_m = 0u64;
    let mut total_codes = 0u64;
    let mut total_fused_codes = 0u64;

    let mut total_dict_entries = 0u64;
    let mut total_used_entries = 0u64;
    let mut total_spelling_bytes = 0u64;
    let mut total_long_oneshot_spelling_bytes = 0u64;
    let mut total_long_reused_spelling_bytes = 0u64;
    let mut total_long_unused_spelling_bytes = 0u64;
    let mut total_long_memo_slack_bytes = 0u64;

    let mut dict_bytes_per_block = Vec::with_capacity(blocks.len());
    let mut inline_bytes_per_block = Vec::with_capacity(blocks.len());
    let mut spelling_bytes_per_block = Vec::with_capacity(blocks.len());
    let mut dict_len_per_block = Vec::with_capacity(blocks.len());

    // Class aggregates: [literal (len=1, c<256), inline2..16, long (>16)]
    let mut cls_dict = [0u64; 3];
    let mut cls_used = [0u64; 3];
    let mut cls_codes = [0u64; 3];
    let mut cls_bytes = [0u64; 3];

    // Fine buckets
    let mut bkt_dict = [0u64; 11];
    let mut bkt_codes = [0u64; 11];
    let mut bkt_bytes = [0u64; 11];

    // Exact length histograms (len -> count)
    let mut hist_codes_map: HashMap<usize, u64> = HashMap::new();
    let mut hist_bytes_map: HashMap<usize, u64> = HashMap::new();
    let mut hist_dict_map: HashMap<usize, u64> = HashMap::new();

    // Reuse buckets: all words (c >= 0), inline (2..=16), long (>16)
    let mut reuse_all = [(0u64, 0u64, 0u64); 6]; // (distinct, codes, bytes)
    let mut reuse_inl = [(0u64, 0u64, 0u64); 6];
    let mut reuse_lng = [(0u64, 0u64, 0u64); 6];

    // Top-k hotness (per block)
    const TOP_KS: [usize; 6] = [16, 64, 256, 512, 1024, 2048];
    let mut topk_code_shares: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::new());
    let mut topk_byte_shares: [Vec<f64>; 6] = std::array::from_fn(|_| Vec::new());
    let mut topk_code_sum = [0u64; 6];
    let mut topk_byte_sum = [0u64; 6];

    // Hotness locality: median code-id of top-256 hot codes / dict_len
    let mut hot256_id_fracs = Vec::with_capacity(blocks.len());

    // Branch predictability & consecutive streaks
    let mut br_transitions = 0u64;
    let mut br_1bit_miss = 0u64;
    let mut br_2bit_miss = 0u64;
    let mut br_total_steps = 0u64;
    let mut br_long_steps = 0u64;

    let mut lit_following_lit = 0u64;
    let mut total_lit_streaks = 0u64;
    let mut codes_in_lit_streak_ge2 = 0u64;
    let mut adj_pair_fits_16 = 0u64;
    let mut adj_pairs_total = 0u64;
    let mut len16_codes = 0u64;
    let mut len16_bytes = 0u64;

    for b in &blocks {
        if b.predecessor.is_some() {
            bidi_blocks += 1;
        }
        if b.exact_cycle {
            exact_cycle_blocks += 1;
        }
        total_n += b.n as u64;
        total_m += b.m as u64;
        total_codes += b.codes.len() as u64;
        total_fused_codes += b.fused_codes.len() as u64;

        let dlen = b.lens.len();
        total_dict_entries += dlen as u64;
        total_spelling_bytes += b.spelling.len() as u64;

        let inl_b = dlen * (INLINE + 1);
        let sp_b = b.spelling.len();
        dict_len_per_block.push(dlen);
        inline_bytes_per_block.push(inl_b);
        spelling_bytes_per_block.push(sp_b);
        dict_bytes_per_block.push(inl_b + sp_b);

        let mut counts = vec![0u32; dlen];
        let mut prev_long = false;
        let mut pred_2bit: u8 = 0; // 0,1 = inline; 2,3 = long
        let mut cur_lit_streak = 0u64;
        let mut prev_len: Option<usize> = None;

        for (step_idx, &c) in b.codes.iter().enumerate() {
            let cu = c as usize;
            counts[cu] += 1;
            let is_long = b.lens[cu] == LONG;
            let wlen = word_len(cu, &b.inline16, &b.lens);
            if wlen == 16 {
                len16_codes += 1;
                len16_bytes += 16;
            }

            // Branch stats
            br_total_steps += 1;
            if is_long {
                br_long_steps += 1;
            }
            if step_idx > 0 {
                if is_long != prev_long {
                    br_transitions += 1;
                    br_1bit_miss += 1;
                }
                let pred_long = pred_2bit >= 2;
                if is_long != pred_long {
                    br_2bit_miss += 1;
                }
            }
            if is_long {
                pred_2bit = (pred_2bit + 1).min(3);
            } else {
                pred_2bit = pred_2bit.saturating_sub(1);
            }
            prev_long = is_long;

            // Literal streak & adjacent-pair <= 16B stats
            if cu < 256 {
                if cur_lit_streak > 0 {
                    lit_following_lit += 1;
                }
                cur_lit_streak += 1;
            } else {
                if cur_lit_streak > 0 {
                    total_lit_streaks += 1;
                    if cur_lit_streak >= 2 {
                        codes_in_lit_streak_ge2 += cur_lit_streak;
                    }
                    cur_lit_streak = 0;
                }
            }
            if let Some(pl) = prev_len {
                adj_pairs_total += 1;
                if pl + wlen <= INLINE {
                    adj_pair_fits_16 += 1;
                }
            }
            prev_len = Some(wlen);
        }
        if cur_lit_streak > 0 {
            total_lit_streaks += 1;
            if cur_lit_streak >= 2 {
                codes_in_lit_streak_ge2 += cur_lit_streak;
            }
        }

        // Dictionary & reuse statistics
        let mut long_ref_bytes = 0u64;
        let mut by_freq: Vec<(usize, u32, usize)> = Vec::with_capacity(dlen);
        for (c, &cnt) in counts.iter().enumerate() {
            let wlen = word_len(c, &b.inline16, &b.lens);
            let cls = if c < 256 {
                0
            } else if b.lens[c] != LONG {
                1
            } else {
                2
            };
            cls_dict[cls] += 1;
            bkt_dict[len_bucket(wlen)] += 1;
            *hist_dict_map.entry(wlen).or_insert(0) += 1;

            if cls == 2 {
                long_ref_bytes += wlen as u64;
            }

            if cnt > 0 {
                total_used_entries += 1;
                cls_used[cls] += 1;
                let bytes = u64::from(cnt) * wlen as u64;
                cls_codes[cls] += u64::from(cnt);
                cls_bytes[cls] += bytes;
                let bk = len_bucket(wlen);
                bkt_codes[bk] += u64::from(cnt);
                bkt_bytes[bk] += bytes;
                *hist_codes_map.entry(wlen).or_insert(0) += u64::from(cnt);
                *hist_bytes_map.entry(wlen).or_insert(0) += bytes;

                let rb = reuse_bucket(cnt);
                reuse_all[rb].0 += 1;
                reuse_all[rb].1 += u64::from(cnt);
                reuse_all[rb].2 += bytes;
                if cls == 1 {
                    reuse_inl[rb].0 += 1;
                    reuse_inl[rb].1 += u64::from(cnt);
                    reuse_inl[rb].2 += bytes;
                } else if cls == 2 {
                    reuse_lng[rb].0 += 1;
                    reuse_lng[rb].1 += u64::from(cnt);
                    reuse_lng[rb].2 += bytes;
                    if cnt == 1 {
                        total_long_oneshot_spelling_bytes += wlen as u64;
                    } else {
                        total_long_reused_spelling_bytes += wlen as u64;
                    }
                }
                by_freq.push((c, cnt, wlen));
            } else if cls == 2 {
                total_long_unused_spelling_bytes += wlen as u64;
            }
        }
        // Bytes in `spelling` not referenced by any long_slot (e.g. memoized child slices kept on dedup)
        total_long_memo_slack_bytes += (b.spelling.len() as u64).saturating_sub(long_ref_bytes);

        by_freq.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        let blk_codes = b.codes.len() as f64;
        let blk_bytes: u64 = by_freq
            .iter()
            .map(|&(_, cnt, wl)| u64::from(cnt) * wl as u64)
            .sum();
        for (ki, &k) in TOP_KS.iter().enumerate() {
            let mut kc = 0u64;
            let mut kb = 0u64;
            for &(_, cnt, wl) in by_freq.iter().take(k) {
                kc += u64::from(cnt);
                kb += u64::from(cnt) * wl as u64;
            }
            topk_code_shares[ki].push(kc as f64 / blk_codes.max(1.0));
            topk_byte_shares[ki].push(kb as f64 / (blk_bytes as f64).max(1.0));
            topk_code_sum[ki] += kc;
            topk_byte_sum[ki] += kb;
        }
        if dlen > 0 && !by_freq.is_empty() {
            let mut hot_ids: Vec<usize> = by_freq.iter().take(256).map(|&(c, _, _)| c).collect();
            hot_ids.sort_unstable();
            let med_id = pct_usize(&hot_ids, 0.5);
            hot256_id_fracs.push(med_id as f64 / dlen as f64);
        }
    }

    dict_bytes_per_block.sort_unstable();
    inline_bytes_per_block.sort_unstable();
    spelling_bytes_per_block.sort_unstable();
    dict_len_per_block.sort_unstable();
    hot256_id_fracs.sort_by(|a, b| a.total_cmp(b));

    let over_l1 = dict_bytes_per_block
        .iter()
        .filter(|&&x| x > 32 * 1024)
        .count();
    let over_l2 = dict_bytes_per_block
        .iter()
        .filter(|&&x| x > 256 * 1024)
        .count();
    let inl_over_l1 = inline_bytes_per_block
        .iter()
        .filter(|&&x| x > 32 * 1024)
        .count();

    println!("\n## Overview");
    println!(
        "open_blocks={} bidi={} ({:.1}%) uni={} exact_cycle={} ({:.1}%)",
        blocks.len(),
        bidi_blocks,
        100.0 * bidi_blocks as f64 / blocks.len() as f64,
        blocks.len() - bidi_blocks,
        exact_cycle_blocks,
        100.0 * exact_cycle_blocks as f64 / blocks.len() as f64
    );
    println!(
        "total_bytes={} total_retained_m={} total_emitted_codes={} mean_bytes_per_code={:.3} fused16_codes={} ({:.1}% reduction)",
        total_n,
        total_m,
        total_codes,
        total_n as f64 / total_codes.max(1) as f64,
        total_fused_codes,
        100.0 * (1.0 - total_fused_codes as f64 / total_codes.max(1) as f64)
    );
    println!(
        "dict_entries_total={} used={} ({:.2}%) dict_len_per_block(p50/p90/max)={}/{}/{}",
        total_dict_entries,
        total_used_entries,
        100.0 * total_used_entries as f64 / total_dict_entries.max(1) as f64,
        pct_usize(&dict_len_per_block, 0.5),
        pct_usize(&dict_len_per_block, 0.9),
        dict_len_per_block.last().copied().unwrap_or(0)
    );
    println!(
        "footprint_per_block_KiB: inline+lens(p50/p90/max)={:.1}/{:.1}/{:.1}  spelling(p50/p90/max)={:.1}/{:.1}/{:.1}  total(p50/p90/max)={:.1}/{:.1}/{:.1}",
        pct_usize(&inline_bytes_per_block, 0.5) as f64 / 1024.0,
        pct_usize(&inline_bytes_per_block, 0.9) as f64 / 1024.0,
        *inline_bytes_per_block.last().unwrap_or(&0) as f64 / 1024.0,
        pct_usize(&spelling_bytes_per_block, 0.5) as f64 / 1024.0,
        pct_usize(&spelling_bytes_per_block, 0.9) as f64 / 1024.0,
        *spelling_bytes_per_block.last().unwrap_or(&0) as f64 / 1024.0,
        pct_usize(&dict_bytes_per_block, 0.5) as f64 / 1024.0,
        pct_usize(&dict_bytes_per_block, 0.9) as f64 / 1024.0,
        *dict_bytes_per_block.last().unwrap_or(&0) as f64 / 1024.0,
    );
    println!(
        "blocks_exceeding_cache: total_dict>32KiB(L1d)={}/{} ({:.1}%)  inline+lens>32KiB={}/{} ({:.1}%)  total_dict>256KiB(L2)={}/{} ({:.1}%)",
        over_l1,
        blocks.len(),
        100.0 * over_l1 as f64 / blocks.len() as f64,
        inl_over_l1,
        blocks.len(),
        100.0 * inl_over_l1 as f64 / blocks.len() as f64,
        over_l2,
        blocks.len(),
        100.0 * over_l2 as f64 / blocks.len() as f64,
    );

    println!("\n## By Class (Literal vs Inline 2..16 vs Long >16)");
    println!("class\tdict_entries\tused_entries\temitted_codes\tcode_pct\temitted_bytes\tbyte_pct\tmean_len");
    let cls_names = ["literal (1 B)", "inline (2..16 B)", "long (>16 B)"];
    let emitted_total_bytes: u64 = cls_bytes.iter().sum();
    for i in 0..3 {
        println!(
            "{}\t{}\t{}\t{}\t{:.2}%\t{}\t{:.2}%\t{:.2}",
            cls_names[i],
            cls_dict[i],
            cls_used[i],
            cls_codes[i],
            100.0 * cls_codes[i] as f64 / total_codes.max(1) as f64,
            cls_bytes[i],
            100.0 * cls_bytes[i] as f64 / emitted_total_bytes.max(1) as f64,
            cls_bytes[i] as f64 / cls_codes[i].max(1) as f64,
        );
    }
    let all_inl_codes = cls_codes[0] + cls_codes[1];
    let all_inl_bytes = cls_bytes[0] + cls_bytes[1];
    println!(
        "ALL inline (<=16 B)\t{}\t{}\t{}\t{:.2}%\t{}\t{:.2}%\t{:.2}",
        cls_dict[0] + cls_dict[1],
        cls_used[0] + cls_used[1],
        all_inl_codes,
        100.0 * all_inl_codes as f64 / total_codes.max(1) as f64,
        all_inl_bytes,
        100.0 * all_inl_bytes as f64 / emitted_total_bytes.max(1) as f64,
        all_inl_bytes as f64 / all_inl_codes.max(1) as f64,
    );
    println!(
        "len==16 specifically: codes={} ({:.2}%), bytes={} ({:.2}%)",
        len16_codes,
        100.0 * len16_codes as f64 / total_codes.max(1) as f64,
        len16_bytes,
        100.0 * len16_bytes as f64 / emitted_total_bytes.max(1) as f64,
    );

    println!("\n## Length Histogram");
    println!("bucket\tdict_entries\temitted_codes\tcode_pct\temitted_bytes\tbyte_pct");
    for i in 0..11 {
        println!(
            "{}\t{}\t{}\t{:.2}%\t{}\t{:.2}%",
            BUCKET_NAMES[i],
            bkt_dict[i],
            bkt_codes[i],
            100.0 * bkt_codes[i] as f64 / total_codes.max(1) as f64,
            bkt_bytes[i],
            100.0 * bkt_bytes[i] as f64 / emitted_total_bytes.max(1) as f64,
        );
    }

    let mut hist_codes: Vec<(usize, u64)> = hist_codes_map.into_iter().collect();
    hist_codes.sort_unstable_by_key(|&(l, _)| l);
    let mut hist_bytes: Vec<(usize, u64)> = hist_bytes_map.into_iter().collect();
    hist_bytes.sort_unstable_by_key(|&(l, _)| l);
    let mut hist_dict: Vec<(usize, u64)> = hist_dict_map.into_iter().collect();
    hist_dict.sort_unstable_by_key(|&(l, _)| l);

    println!("\n## Length Percentiles (min / p10 / p25 / p50 / p75 / p90 / p95 / p99 / max)");
    for (label, h, tot) in [
        ("by_emitted_codes", &hist_codes, total_codes),
        ("by_emitted_bytes", &hist_bytes, emitted_total_bytes),
        ("by_dict_entries ", &hist_dict, total_dict_entries),
    ] {
        let min_l = h.first().map(|x| x.0).unwrap_or(0);
        let max_l = h.last().map(|x| x.0).unwrap_or(0);
        println!(
            "{}: {} / {} / {} / {} / {} / {} / {} / {} / {}",
            label,
            min_l,
            weighted_percentile(h, tot, 0.10),
            weighted_percentile(h, tot, 0.25),
            weighted_percentile(h, tot, 0.50),
            weighted_percentile(h, tot, 0.75),
            weighted_percentile(h, tot, 0.90),
            weighted_percentile(h, tot, 0.95),
            weighted_percentile(h, tot, 0.99),
            max_l,
        );
    }

    println!("\n## Reuse Distribution (per block)");
    for (title, arr, tot_c, tot_b) in [
        (
            "ALL used codes",
            reuse_all,
            total_codes,
            emitted_total_bytes,
        ),
        (
            "INLINE words (2..16 B)",
            reuse_inl,
            cls_codes[1],
            cls_bytes[1],
        ),
        ("LONG words (>16 B)", reuse_lng, cls_codes[2], cls_bytes[2]),
    ] {
        let tot_d: u64 = arr.iter().map(|x| x.0).sum();
        println!("--- {title} ---");
        println!("reuse\tdistinct\tdistinct_pct\tcodes\tcode_pct\tbytes\tbyte_pct");
        for i in 0..6 {
            println!(
                "{}\t{}\t{:.2}%\t{}\t{:.2}%\t{}\t{:.2}%",
                REUSE_NAMES[i],
                arr[i].0,
                100.0 * arr[i].0 as f64 / tot_d.max(1) as f64,
                arr[i].1,
                100.0 * arr[i].1 as f64 / tot_c.max(1) as f64,
                arr[i].2,
                100.0 * arr[i].2 as f64 / tot_b.max(1) as f64,
            );
        }
    }

    println!("\n## Spelling Buffer Breakdown (bytes built in A6b)");
    println!(
        "total_spelling_bytes={}  oneshot_long_bytes={} ({:.2}%)  reused_long_bytes={} ({:.2}%)  unused_long_bytes={} ({:.2}%)  memo_dedup_slack={} ({:.2}%)",
        total_spelling_bytes,
        total_long_oneshot_spelling_bytes,
        100.0 * total_long_oneshot_spelling_bytes as f64 / total_spelling_bytes.max(1) as f64,
        total_long_reused_spelling_bytes,
        100.0 * total_long_reused_spelling_bytes as f64 / total_spelling_bytes.max(1) as f64,
        total_long_unused_spelling_bytes,
        100.0 * total_long_unused_spelling_bytes as f64 / total_spelling_bytes.max(1) as f64,
        total_long_memo_slack_bytes,
        100.0 * total_long_memo_slack_bytes as f64 / total_spelling_bytes.max(1) as f64,
    );

    println!("\n## Top-K Hotness Concentration (per-block median & corpus-weighted mean)");
    println!("top_k\tcode_share_med\tcode_share_mean\tbyte_share_med\tbyte_share_mean");
    for (ki, &k) in TOP_KS.iter().enumerate() {
        topk_code_shares[ki].sort_by(|a, b| a.total_cmp(b));
        topk_byte_shares[ki].sort_by(|a, b| a.total_cmp(b));
        println!(
            "{}\t{:.2}%\t{:.2}%\t{:.2}%\t{:.2}%",
            k,
            100.0 * pct_f64(&topk_code_shares[ki], 0.5),
            100.0 * topk_code_sum[ki] as f64 / total_codes.max(1) as f64,
            100.0 * pct_f64(&topk_byte_shares[ki], 0.5),
            100.0 * topk_byte_sum[ki] as f64 / emitted_total_bytes.max(1) as f64,
        );
    }
    println!(
        "hot256_median_code_id / dict_len: p50={:.3} p90={:.3}",
        pct_f64(&hot256_id_fracs, 0.5),
        pct_f64(&hot256_id_fracs, 0.9),
    );

    println!("\n## Branch Predictability (`len != LONG`) & Consecutive Streaks");
    println!(
        "p_long={:.2}%  transitions={:.2}%  1bit_miss={:.2}%  2bit_miss={:.2}%",
        100.0 * br_long_steps as f64 / br_total_steps.max(1) as f64,
        100.0 * br_transitions as f64 / br_total_steps.max(1) as f64,
        100.0 * br_1bit_miss as f64 / br_total_steps.max(1) as f64,
        100.0 * br_2bit_miss as f64 / br_total_steps.max(1) as f64,
    );
    println!(
        "literal_streaks={}  mean_lit_streak={:.2}  lit_following_lit={:.2}%_of_lits  codes_in_lit_streak>=2={:.2}%_of_all_codes",
        total_lit_streaks,
        cls_codes[0] as f64 / total_lit_streaks.max(1) as f64,
        100.0 * lit_following_lit as f64 / cls_codes[0].max(1) as f64,
        100.0 * codes_in_lit_streak_ge2 as f64 / total_codes.max(1) as f64,
    );
    println!(
        "adjacent_code_pairs_fitting_in_16B={}/{} ({:.2}%)",
        adj_pair_fits_16,
        adj_pairs_total,
        100.0 * adj_pair_fits_16 as f64 / adj_pairs_total.max(1) as f64,
    );
}

// ---------------------------------------------------------------------------
// Command: bench (timing census against production mirror)
// ---------------------------------------------------------------------------

struct ArmSpec {
    name: &'static str,
    exact: bool,
    run_one: fn(&BuiltBlock, &mut Vec<u8>) -> u64,
    ms: Vec<f64>,
}

fn cmd_bench(path: &str, forced: bool, repeats: usize) {
    let (blocks, total_blocks, total_rows) = build_corpus(path, forced);
    let total_codes: usize = blocks.iter().map(|b| b.codes.len()).sum();
    let total_bytes: usize = blocks.iter().map(|b| b.n).sum();
    let fused_codes_total: usize = blocks.iter().map(|b| b.fused_codes.len()).sum();
    println!(
        "# spelling bench: {path} (forced={forced}, repeats={repeats}, interleaved=true, total_blocks={total_blocks}, total_rows={total_rows}, open_blocks={}, open_codes={total_codes}, open_bytes={total_bytes})",
        blocks.len()
    );
    if blocks.is_empty() {
        println!("NO_ADMITTED_BLOCKS");
        return;
    }

    let mut arms: Vec<ArmSpec> = vec![
        ArmSpec {
            name: "prod_walk_and_spell",
            exact: true,
            run_one: |b, out| {
                if let Some(before) = b.predecessor {
                    emit_prod_bidi(
                        &b.edges,
                        &b.spelling,
                        &b.inline16,
                        &b.lens,
                        b.n,
                        b.mark,
                        before,
                        b.bits,
                        out,
                    );
                } else {
                    emit_prod_uni(
                        &b.edges,
                        &b.extra,
                        &b.spelling,
                        &b.inline16,
                        &b.lens,
                        b.n,
                        b.mark,
                        b.bits,
                        out,
                    );
                }
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "mirror_e_consume",
            exact: true,
            run_one: |b, out| {
                if b.predecessor.is_some() {
                    emit_codes_bidi(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
                } else {
                    emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
                }
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "bidi_pre_shifted",
            exact: true,
            run_one: |b, out| {
                if b.predecessor.is_some() {
                    emit_codes_bidi_pre_shifted(
                        &b.codes,
                        &b.spelling,
                        &b.inline16,
                        &b.inline_back,
                        &b.lens,
                        b.n,
                        out,
                    );
                } else {
                    emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
                }
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "bidi_no_shift_ablat",
            exact: false,
            run_one: |b, out| {
                if b.predecessor.is_some() {
                    emit_codes_bidi_no_shift_ablat(
                        &b.codes,
                        &b.spelling,
                        &b.inline16,
                        &b.lens,
                        b.n,
                        out,
                    ) as u64
                } else {
                    emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
                    out.len() as u64
                }
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "forward_only",
            exact: true,
            run_one: |b, out| {
                emit_codes_uni(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "inline_only_skip_long",
            exact: false,
            run_one: |b, out| {
                emit_inline_only_skip_long(&b.codes, &b.inline16, &b.lens, b.n, out) as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "long_only_skip_inline",
            exact: false,
            run_one: |b, out| {
                emit_long_only_skip_inline(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out)
                    as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "no_copy_length_only",
            exact: false,
            run_one: |b, _out| emit_no_copy_length_only(&b.codes, &b.inline16, &b.lens, b.n) as u64,
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "codes_scan_only",
            exact: false,
            run_one: |b, _out| emit_codes_scan_only(&b.codes),
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "memcpy_out_floor",
            exact: true,
            run_one: |b, out| {
                out.resize(b.n, 0);
                out.copy_from_slice(&b.walked);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "chunk_mixed_pairs",
            exact: true,
            run_one: |b, out| {
                emit_chunk_mixed_pairs(b, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "chunk_sorted_pairs",
            exact: false,
            run_one: |b, out| emit_chunk_sorted_pairs(b, out) as u64,
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "dict_l1_pinned",
            exact: false,
            run_one: |b, out| {
                emit_dict_l1_pinned(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out) as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "out_l1_pinned",
            exact: false,
            run_one: |b, out| {
                emit_out_l1_pinned(&b.codes, &b.spelling, &b.inline16, &b.lens, b.n, out) as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "packed_15b_single_arr",
            exact: true,
            run_one: |b, out| {
                emit_packed15(b, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "inline_32b",
            exact: true,
            run_one: |b, out| {
                emit_inline32(b, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "direct_meta_no_clamp",
            exact: true,
            run_one: |b, out| {
                emit_direct_no_clamp(b, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "freq_sorted_dict",
            exact: true,
            run_one: |b, out| {
                emit_codes_uni(
                    &b.freq_codes,
                    &b.spelling,
                    &b.freq_inline16,
                    &b.freq_lens,
                    b.n,
                    out,
                );
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "unrolled_4x_forward",
            exact: true,
            run_one: |b, out| {
                emit_unrolled_4x(b, out);
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
        ArmSpec {
            name: "fused_16b_upper_bound",
            exact: true,
            run_one: |b, out| {
                emit_codes_uni(
                    &b.fused_codes,
                    &b.spelling,
                    &b.fused_inline16,
                    &b.fused_lens,
                    b.n,
                    out,
                );
                out.len() as u64
            },
            ms: Vec::with_capacity(repeats),
        },
    ];

    let mut out = Vec::new();
    // Pass 0: warm-up + byte-exact verification against bwt::walk for every exact arm
    for arm in &arms {
        for (idx, b) in blocks.iter().enumerate() {
            let sink = (arm.run_one)(b, &mut out);
            black_box(sink);
            if arm.exact {
                assert_eq!(
                    out, b.walked,
                    "arm `{}` byte mismatch on block {idx}",
                    arm.name
                );
            }
        }
    }

    // Interleaved timed repeats across all arms (cancels temporal host jitter)
    for _ in 0..repeats {
        for arm in &mut arms {
            let t0 = Instant::now();
            let mut acc = 0u64;
            for b in &blocks {
                acc = acc.wrapping_add((arm.run_one)(b, &mut out));
            }
            black_box(acc);
            black_box(&out);
            arm.ms.push(t0.elapsed().as_secs_f64() * 1_000.0);
        }
    }

    // Print summary table with paired per-repeat ratios against mirror_e_consume
    assert_eq!(arms[1].name, "mirror_e_consume");
    let mirror_ms = arms[1].ms.clone();

    println!(
        "\narm\texact\tms_med (min..max)\tns/code_med (min..max)\tns/byte_med (min..max)\tratio_vs_mirror (min..max)"
    );
    for r in &arms {
        let (ms_med, ms_min, ms_max) = median_min_max(&r.ms);
        let ns_code: Vec<f64> = r.ms.iter().map(|&m| m * 1e6 / total_codes as f64).collect();
        let ns_byte: Vec<f64> = r.ms.iter().map(|&m| m * 1e6 / total_bytes as f64).collect();
        let ratios: Vec<f64> =
            r.ms.iter()
                .zip(mirror_ms.iter())
                .map(|(&m, &mm)| m / mm)
                .collect();
        let (nc_med, nc_min, nc_max) = median_min_max(&ns_code);
        let (nb_med, nb_min, nb_max) = median_min_max(&ns_byte);
        let (ra_med, ra_min, ra_max) = median_min_max(&ratios);
        println!(
            "{:<22}\t{}\t{:.2} ({:.2}..{:.2})\t{:.3} ({:.3}..{:.3})\t{:.4} ({:.4}..{:.4})\t{:.3}x ({:.3}..{:.3}x)",
            r.name,
            if r.exact { "exact" } else { "ablat" },
            ms_med,
            ms_min,
            ms_max,
            nc_med,
            nc_min,
            nc_max,
            nb_med,
            nb_min,
            nb_max,
            ra_med,
            ra_min,
            ra_max,
        );
    }

    // Derived deltas (ns/code & % of mirror_e_consume)
    let med_ns_code = |name: &str| -> f64 {
        let r = arms.iter().find(|r| r.name == name).unwrap();
        let (ms_med, _, _) = median_min_max(&r.ms);
        ms_med * 1e6 / total_codes as f64
    };
    let mir = med_ns_code("mirror_e_consume");
    let fwd = med_ns_code("forward_only");
    let inl_only = med_ns_code("inline_only_skip_long");
    let lng_only = med_ns_code("long_only_skip_inline");
    let no_copy = med_ns_code("no_copy_length_only");
    let scan = med_ns_code("codes_scan_only");
    let ch_mix = med_ns_code("chunk_mixed_pairs");
    let ch_srt = med_ns_code("chunk_sorted_pairs");
    let dict_l1 = med_ns_code("dict_l1_pinned");
    let out_l1 = med_ns_code("out_l1_pinned");
    let walk_all = med_ns_code("prod_walk_and_spell");

    println!("\n## Derived Component Deltas (median ns/code and share of mirror_e_consume)");
    let row = |label: &str, val: f64| {
        println!(
            "{:<38}\t{:+.3} ns/code\t{:+.4} ns/byte\t{:+.1}% of mirror",
            label,
            val,
            val * total_codes as f64 / total_bytes as f64,
            100.0 * val / mir
        );
    };
    row("E_consume share of full emission", mir);
    println!(
        "  (mirror_e_consume / prod_walk_and_spell = {:.1}%)",
        100.0 * mir / walk_all
    );
    let bidi_pre = med_ns_code("bidi_pre_shifted");
    let bidi_noshift = med_ns_code("bidi_no_shift_ablat");
    row("1. Bidirectional overhead (mir - fwd)", mir - fwd);
    row("   - of which u128 shift (mir - nosh)", mir - bidi_noshift);
    row("   - of which dual-cursor (nosh - fwd)", bidi_noshift - fwd);
    row("   - pre-shifted saving (mir - pre)", mir - bidi_pre);
    row("2. Per-code lookup+branch (no_copy)", no_copy);
    row("   - of which codes[] scan", scan);
    row("3. Long-word copy cost (fwd - inl_only)", fwd - inl_only);
    row("4. Inline 16B copy cost (fwd - lng_only)", fwd - lng_only);
    row(
        "   - additive check: no_copy+inl+lng - fwd",
        no_copy + (fwd - lng_only) + (fwd - inl_only) - fwd,
    );
    row("5. Branch mispred delta (mix - sort)", ch_mix - ch_srt);
    row("6. Dict payload cache (fwd - dict_l1)", fwd - dict_l1);
    row("7. Output write stream (fwd - out_l1)", fwd - out_l1);
    println!(
        "fused_16b_upper_bound code count: {} -> {} ({:.1}% fewer codes)",
        total_codes,
        fused_codes_total,
        100.0 * (1.0 - fused_codes_total as f64 / total_codes as f64)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage:\n  {} stats FILE.bz2 [--forced]\n  {} bench FILE.bz2 [--forced] [--repeats N]",
            args[0], args[0]
        );
        std::process::exit(2);
    }
    let cmd = args[1].as_str();
    let path = args[2].as_str();
    let mut forced = false;
    let mut repeats = 7usize;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--forced" => {
                forced = true;
                i += 1;
            }
            "--repeats" => {
                repeats = args[i + 1].parse().expect("--repeats N");
                i += 2;
            }
            other => panic!("unknown arg: {other}"),
        }
    }
    match cmd {
        "stats" => cmd_stats(path, forced),
        "bench" => cmd_bench(path, forced, repeats),
        _ => panic!("unknown command: {cmd}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
        let n = text.len();
        let mut rots: Vec<usize> = (0..n).collect();
        rots.sort_by(|&a, &b| {
            let mut i = 0;
            while i < n {
                let ca = text[(a + i) % n];
                let cb = text[(b + i) % n];
                if ca != cb {
                    return ca.cmp(&cb);
                }
                i += 1;
            }
            a.cmp(&b)
        });
        let l: Vec<u8> = rots.iter().map(|&r| text[(r + n - 1) % n]).collect();
        let p = rots.iter().position(|&r| r == 0).unwrap() as u32;
        (l, p)
    }

    #[test]
    fn mirror_and_variants_match_bwt_walk() {
        let mut text = Vec::new();
        for i in 0..400usize {
            text.extend_from_slice(b"{\"id\":\"Q");
            text.extend_from_slice(i.to_string().as_bytes());
            text.extend_from_slice(b"\",\"type\":\"item\",\"claims\":{\"P31\":[1,2,3]}}\n");
        }
        let (l, p) = bwt_of(&text);
        let recs = records_of(&l);
        let mut kernel = OpenMirror::default();
        let mut out = Vec::new();
        let (m, mark, bits) = kernel
            .construct_block(&recs, text.len(), p, Cap::NONE, &mut out)
            .expect("open admits synthetic json");
        let bb = build_block_variants(
            text.len(),
            m,
            mark,
            bits,
            kernel.predecessor,
            kernel.edges[..m].to_vec(),
            kernel.extra.clone(),
            kernel.spelling.clone(),
            kernel.inline16.clone(),
            kernel.lens.clone(),
            text.clone(),
        );

        emit_codes_uni(
            &bb.codes,
            &bb.spelling,
            &bb.inline16,
            &bb.lens,
            bb.n,
            &mut out,
        );
        assert_eq!(out, text, "forward_only");

        emit_codes_bidi(
            &bb.codes,
            &bb.spelling,
            &bb.inline16,
            &bb.lens,
            bb.n,
            &mut out,
        );
        assert_eq!(out, text, "bidi");

        emit_codes_bidi_pre_shifted(
            &bb.codes,
            &bb.spelling,
            &bb.inline16,
            &bb.inline_back,
            &bb.lens,
            bb.n,
            &mut out,
        );
        assert_eq!(out, text, "bidi_pre_shifted");

        emit_packed15(&bb, &mut out);
        assert_eq!(out, text, "packed15");

        emit_inline32(&bb, &mut out);
        assert_eq!(out, text, "inline32");

        emit_direct_no_clamp(&bb, &mut out);
        assert_eq!(out, text, "direct_no_clamp");

        emit_unrolled_4x(&bb, &mut out);
        assert_eq!(out, text, "unrolled_4x");

        emit_codes_uni(
            &bb.freq_codes,
            &bb.spelling,
            &bb.freq_inline16,
            &bb.freq_lens,
            bb.n,
            &mut out,
        );
        assert_eq!(out, text, "freq_sorted");

        emit_codes_uni(
            &bb.fused_codes,
            &bb.spelling,
            &bb.fused_inline16,
            &bb.fused_lens,
            bb.n,
            &mut out,
        );
        assert_eq!(out, text, "fused_16b");

        emit_chunk_mixed_pairs(&bb, &mut out);
        assert_eq!(out, text, "chunk_mixed_pairs");
    }
}
