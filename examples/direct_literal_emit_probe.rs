#![forbid(unsafe_code)]
//! Direct-literal 4-lane forward emission & compact dictionary probe (`A6b-dict + A6c + A7`).
//!
//! Smooth-zone diagnostic (`Claim: none`, zero `src/` edits, `#![forbid(unsafe_code)]`).
//!
//! Hypothesis: In `Open`, ~82% of all `A7` steps are 1-byte literals (`code < 256`), yet
//! shipped 4-lane forward emit performs a 2-level dictionary indirection (`lens[code]` 1B load +
//! `inline16[code]` 16B load + `len == LONG` branch + 16B unaligned overwrite) on every literal.
//! Bypassing the dictionary on `code < 256` with direct 1-byte stores (per-lane, all-4 fastpath,
//! or tight inner literal-streak loop) plus a compact `inline16`/`lens` table (indexed by
//! `code - 256`, saving 4.25 KiB L1d) may cut `A7` and `A6b-dict + A7`.
//!
//! FROZEN STOP BAR (recorded before timing):
//!   Worthwhile for follow-on ONLY IF the winning direct-literal arm cuts `A7` (or `A6b-dict + A7`)
//!   by >= 5.0% of `A7`, with disjoint `[min..max]` spreads on baseline x86-64 AND JCC-aligned
//!   `-C target-cpu=native`, in BOTH warm AND L3-cold regimes, and 233/233 byte-exact vs `bwt::walk`.
//!   Otherwise STOP cleanly and record a scoped row in `frontier/dead-ends.tsv`.
//!
//! Usage:
//!   direct_literal_emit_probe [FILE.bz2] [--repeats N]

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
const S_ANCHORS: usize = 16;
const MAX_ANCHORS: usize = 32;
const SLAB_SIZE: usize = 32 * 1024;
const COLD_FLUSH_BYTES: usize = 64 * 1024 * 1024;
const COLD_POOL_WORKERS: usize = 16;
const WARM_A7_INNER_REPS: usize = 4;

// Stage-1 i9-10900K reference anchors (wikidata x4, 1 worker)
const I9_A7_FWD4_EST_MS: f64 = 295.0; // modeled 4-lane forward A7 anchor on 1W i9-10900K
const I9_A7_SHIPPED_MS: f64 = 490.3;
const STOP_BAR_PCT: f64 = 5.0;

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

// ---------------------------------------------------------------------------
// Compact Dictionary (Arm 5: no 256-entry literal table, -4.25 KiB L1d, direct literal bypass)
// ---------------------------------------------------------------------------

fn begin_inline_compact(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
    inline16.clear();
    lens.clear();
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
}

struct DictionaryCompact<'a> {
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl DictionaryCompact<'_> {
    /// Returns code `>= 256` for non-zero handles (1-byte literals `handle == 0` are bypassed
    /// directly by the caller as `u32::from(byte)` without touching the dictionary!).
    #[inline(always)]
    fn code_nonlit<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
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
        let same = old >= 256 && {
            let wi = old - 256;
            self.lens[wi] == len
                && if len == LONG {
                    long_word(&self.inline16[wi], self.spelling) == bytes
                } else {
                    self.inline16[wi] == entry
                }
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

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = 256 + self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

// ---------------------------------------------------------------------------
// Post-elimination snapshot & PreparedBlock
// ---------------------------------------------------------------------------

struct PreparedBlock {
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    can_tag: bool,
    wide_links: bool,
    thin: [u32; 256],
    // Post-elimination Elim<L> live slice (before A6b-dict); only long_rule_tag_slots are mutated
    // during `Elim::append` and reset outside timing between repeats.
    elim_live: Vec<u32>,
    elim_base: usize,
    elim_top: usize,
    elim_nodes: u32,
    long_rule_tag_slots: Vec<usize>,
    // Post-A6a queues for A6c wiring
    outq_bytes: Vec<u8>,
    spill: Spill,
    heads: Heads,
    // Pre-built post-A6b / post-A6c tables for isolated A7 suite & census
    inline16_full: Vec<[u8; INLINE]>,
    lens_full: Vec<u8>,
    inline16_compact: Vec<[u8; INLINE]>,
    lens_compact: Vec<u8>,
    spelling: Vec<u8>,
    tagged_edges: Vec<u32>,
    anchors: [u32; MAX_ANCHORS],
    anchor_cnt: usize,
    expected: Vec<u8>,
}

impl PreparedBlock {
    fn reset_elim_tags(&mut self) {
        for &slot in &self.long_rule_tag_slots {
            self.elim_live[slot] = u32::MAX;
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

    fn run_elim_snapshot<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
    ) -> (Vec<u32>, usize, usize, u32, Vec<usize>) {
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
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let live = self.edges[4 * ns..4 * ns + top].to_vec();
        let mut long_slots = Vec::new();
        let mut r = base;
        while r + RULE <= top {
            if live[r + 3] == u32::MAX {
                long_slots.push(r + 3);
            }
            r += RULE;
        }
        (live, base, top, nodes, long_slots)
    }

    fn prepare_block(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        scratch_out: &mut Vec<u8>,
        expected: Vec<u8>,
    ) -> PreparedBlock {
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

        let (elim_live, elim_base, elim_top, elim_nodes, long_rule_tag_slots) = if wide_links {
            self.run_elim_snapshot::<Wide>(ns, n as u32, f.m)
        } else {
            self.run_elim_snapshot::<Narrow>(ns, n as u32, f.m)
        };

        let outq_bytes = scratch_out[..chunks_taken * CHUNK_BYTES].to_vec();
        let m = f.m as usize;
        let bits = (u32::BITS - (m.max(1) as u32 - 1).leading_zeros()).max(1);

        let mut pb = PreparedBlock {
            n,
            m,
            mark: f.mark,
            bits,
            can_tag: false,
            wide_links,
            thin: f.thin,
            elim_live,
            elim_base,
            elim_top,
            elim_nodes,
            long_rule_tag_slots,
            outq_bytes,
            spill: self.spill.clone(),
            heads,
            inline16_full: Vec::new(),
            lens_full: Vec::new(),
            inline16_compact: Vec::new(),
            lens_compact: Vec::new(),
            spelling: Vec::new(),
            tagged_edges: Vec::new(),
            anchors: [0u32; MAX_ANCHORS],
            anchor_cnt: 0,
            expected,
        };

        // Run shipped A6b-dict + A6c once to populate pre-wired isolated A7 tables
        let mut dict_cache = vec![0u32; 2 * 1024 + 4096];
        self.bands.clear();
        self.spelling.clear();
        let cursors = run_a6b_dict_shipped(
            &mut pb,
            &mut dict_cache,
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.stack,
            &mut self.bands,
        );
        pb.reset_elim_tags();

        let dict_len = self.lens.len();
        let wide = dict_len - 1 > (u32::MAX >> bits) as usize;
        pb.can_tag =
            !wide && m >= S_ANCHORS * 8 && (dict_len - 1) <= ((u32::MAX >> 1) >> bits) as usize;
        assert!(
            pb.can_tag,
            "expected all admitted Open wikidata blocks to fit bit-31 tag"
        );

        pb.inline16_full = self.inline16.clone();
        pb.lens_full = self.lens.clone();
        pb.inline16_compact = self.inline16[256..].to_vec();
        pb.lens_compact = self.lens[256..].to_vec();
        pb.spelling = self.spelling.clone();

        pb.tagged_edges.resize(m, 0);
        wire_forward_bands(
            &pb.outq_bytes,
            &pb.spill,
            &pb.heads,
            &self.bands,
            &cursors,
            bits,
            &mut pb.tagged_edges,
        );
        pb.anchor_cnt = tag_uniform_anchors(&mut pb.tagged_edges, m, f.mark, &mut pb.anchors);
        pb
    }
}

// ---------------------------------------------------------------------------
// A6b-dict (shipped vs compact) and A6c forward wiring
// ---------------------------------------------------------------------------

const PAIR_WORDS: usize = 2 * 1024;
const CACHE_WORDS: usize = PAIR_WORDS + 4096;

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn run_a6b_dict_shipped(
    pb: &mut PreparedBlock,
    cache: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    bands: &mut Vec<Band>,
) -> [usize; 256] {
    bands.clear();
    spelling.clear();
    if pb.n >= 65_536 {
        spelling.reserve_exact(48 * 1024);
    }
    cache[..CACHE_WORDS].fill(0);
    let (pairs, contents) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
    begin_inline(if pb.n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mut dict = Dictionary {
        pairs,
        contents,
        inline16,
        lens,
        spelling,
        stack,
    };
    if pb.wide_links {
        run_a6b_dict_shipped_with::<Wide>(pb, &mut dict, bands)
    } else {
        run_a6b_dict_shipped_with::<Narrow>(pb, &mut dict, bands)
    }
}

fn run_a6b_dict_shipped_with<L: Links>(
    pb: &mut PreparedBlock,
    dict: &mut Dictionary<'_>,
    bands: &mut Vec<Band>,
) -> [usize; 256] {
    let mut e = Elim::<L> {
        a: &mut pb.elim_live[..pb.elim_top],
        base: pb.elim_base,
        top: pb.elim_top,
        nodes: pb.elim_nodes,
        _links: std::marker::PhantomData,
    };
    let mut cursors = [0usize; 256];
    let mut id = L::next(e.a, 0, 0);
    let mut in_node = 0u32;
    let mut k = 0u32;
    for (a, cursor) in cursors.iter_mut().enumerate() {
        *cursor = bands.len();
        let end = k + pb.thin[a];
        while k < end {
            let node_width = e.width(id);
            let node_word = e.word(id);
            let width = (end - k).min(node_width - in_node);
            let dst = e.bottom(id) + in_node;
            let code = dict.code(&mut e, node_word, a as u8);
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn run_a6b_dict_compact(
    pb: &mut PreparedBlock,
    cache: &mut [u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    spelling: &mut Vec<u8>,
    stack: &mut Vec<u32>,
    bands: &mut Vec<Band>,
) -> [usize; 256] {
    bands.clear();
    spelling.clear();
    if pb.n >= 65_536 {
        spelling.reserve_exact(48 * 1024);
    }
    cache[..CACHE_WORDS].fill(0);
    let (pairs, contents) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
    begin_inline_compact(if pb.n >= 65_536 { 1536 } else { 0 }, inline16, lens);
    let mut dict = DictionaryCompact {
        pairs,
        contents,
        inline16,
        lens,
        spelling,
        stack,
    };
    if pb.wide_links {
        run_a6b_dict_compact_with::<Wide>(pb, &mut dict, bands)
    } else {
        run_a6b_dict_compact_with::<Narrow>(pb, &mut dict, bands)
    }
}

fn run_a6b_dict_compact_with<L: Links>(
    pb: &mut PreparedBlock,
    dict: &mut DictionaryCompact<'_>,
    bands: &mut Vec<Band>,
) -> [usize; 256] {
    let mut e = Elim::<L> {
        a: &mut pb.elim_live[..pb.elim_top],
        base: pb.elim_base,
        top: pb.elim_top,
        nodes: pb.elim_nodes,
        _links: std::marker::PhantomData,
    };
    let mut cursors = [0usize; 256];
    let mut id = L::next(e.a, 0, 0);
    let mut in_node = 0u32;
    let mut k = 0u32;
    for (a, cursor) in cursors.iter_mut().enumerate() {
        *cursor = bands.len();
        let end = k + pb.thin[a];
        while k < end {
            let node_width = e.width(id);
            let node_word = e.word(id);
            let width = (end - k).min(node_width - in_node);
            let dst = e.bottom(id) + in_node;
            // Fast inline literal bypass: skips DictionaryCompact call, hashing, and 256-entry table!
            let code = if node_word == 0 {
                a as u32
            } else {
                dict.code_nonlit(&mut e, node_word, a as u8)
            };
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn wire_forward_bands(
    outq_buf: &[u8],
    spill: &Spill,
    heads: &Heads,
    bands: &[Band],
    cursors: &[usize; 256],
    bits: u32,
    edges: &mut [u32],
) {
    let mut k = 0u32;
    for a in 0..256 {
        let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
        let mut queue = Reader::new(outq_buf, spill, heads, a);
        for &band in &bands[cursors[a]..end] {
            let bend = band.end();
            let span = (bend - k) as usize;
            let dst = band.dst() as usize;
            queue.wire(&mut edges[dst..dst + span], band.code() << bits);
            k = bend;
        }
    }
}

#[inline(always)]
fn tag_uniform_anchors(
    edges: &mut [u32],
    m: usize,
    mark: u32,
    anchors: &mut [u32; MAX_ANCHORS],
) -> usize {
    let m32 = m as u32;
    anchors[0] = mark;
    for (k, slot) in anchors.iter_mut().enumerate().take(S_ANCHORS).skip(1) {
        let off = ((k as u64 * m as u64) / S_ANCHORS as u64) as u32;
        let mut a = mark + off;
        if a >= m32 {
            a -= m32;
        }
        *slot = a;
    }
    for &a in &anchors[..S_ANCHORS] {
        edges[a as usize] |= ANCHOR_TAG;
    }
    S_ANCHORS
}

// ---------------------------------------------------------------------------
// 4-Lane Forward Slab Allocator & 5 Emission Arms
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct SliceRef {
    off: u32,
    len: u32,
    next: u16,
}

#[derive(Default)]
struct EmitScratch {
    slices: Vec<SliceRef>,
    ordered: Vec<(usize, usize)>,
}

struct SlabAlloc<'a> {
    arena: &'a mut Vec<u8>,
    top: usize,
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
            let want = self.top + 4 * SLAB_SIZE;
            if want > self.arena.capacity() {
                self.arena.reserve_exact(want - self.arena.len());
            }
            self.arena.resize(want, 0);
        }
        (base, base + size)
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_slow_full(
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
        let word: &[u8] = if len != LONG {
            &inline16[code][..usize::from(len)]
        } else {
            long_word(&inline16[code], spelling)
        };
        let wlen = word.len();
        if *at + wlen + INLINE > *end {
            self.push_slice(seg, *slice_start, *at);
            let (new_base, new_end) = self.take_slab(wlen);
            *slice_start = new_base;
            *at = new_base;
            *end = new_end;
        }
        self.arena[*at..*at + wlen].copy_from_slice(word);
        *at += wlen;
    }

    #[cold]
    #[inline(never)]
    fn emit_lit_slab_switch(
        &mut self,
        seg: usize,
        byte: u8,
        slice_start: &mut usize,
        at: &mut usize,
        end: &mut usize,
    ) {
        self.push_slice(seg, *slice_start, *at);
        let (new_base, new_end) = self.take_slab(1);
        *slice_start = new_base;
        self.arena[new_base] = byte;
        *at = new_base + 1;
        *end = new_end;
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_slow_compact(
        &mut self,
        seg: usize,
        wi: usize,
        len: u8,
        inline16_c: &[[u8; INLINE]],
        spelling: &[u8],
        slice_start: &mut usize,
        at: &mut usize,
        end: &mut usize,
    ) {
        let word: &[u8] = if len != LONG {
            &inline16_c[wi][..usize::from(len)]
        } else {
            long_word(&inline16_c[wi], spelling)
        };
        let wlen = word.len();
        if *at + wlen + INLINE > *end {
            self.push_slice(seg, *slice_start, *at);
            let (new_base, new_end) = self.take_slab(wlen);
            *slice_start = new_base;
            *at = new_base;
            *end = new_end;
        }
        self.arena[*at..*at + wlen].copy_from_slice(word);
        *at += wlen;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Ctrl4LaneDict,
    LitBranchPerLane,
    LitAll4Fastpath,
    LitStreakInner,
    LitPlusCompactDict,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Ctrl4LaneDict => "ctrl_4lane_dict",
            Arm::LitBranchPerLane => "lit_branch_per_lane",
            Arm::LitAll4Fastpath => "lit_all4_fastpath",
            Arm::LitStreakInner => "lit_streak_inner",
            Arm::LitPlusCompactDict => "lit_plus_compact_dict",
        }
    }

    fn uses_compact_dict_build(self) -> bool {
        matches!(self, Arm::LitPlusCompactDict)
    }

    fn emit_mode(self) -> u8 {
        match self {
            Arm::Ctrl4LaneDict => 0,
            Arm::LitBranchPerLane | Arm::LitPlusCompactDict => 1,
            Arm::LitAll4Fastpath => 2,
            Arm::LitStreakInner => 3,
        }
    }
}

const ARMS: &[Arm] = &[
    Arm::Ctrl4LaneDict,
    Arm::LitBranchPerLane,
    Arm::LitAll4Fastpath,
    Arm::LitStreakInner,
    Arm::LitPlusCompactDict,
];

/// Generic 4-lane forward emission with const `MODE`:
/// - `0`: `ctrl_4lane_dict` (shipped `inline16[code]` / `lens[code]`)
/// - `1`: `lit_branch_per_lane` (`code < 256` -> direct 1B store, else `inline16_c[code - 256]`)
/// - `2`: `lit_all4_fastpath` (`((ed0|ed1|ed2|ed3) & non_lit_mask) == 0` -> 4x 1B store, else per-lane)
/// - `3`: `lit_streak_inner` (hoisted slab-budget inner `while k < lit_cap` all-4-literal loop + cold slow path)
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_4lane_mode<const MODE: u8>(
    n: usize,
    m: usize,
    bits: u32,
    edges: &[u32],
    anchors: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    arena: &mut Vec<u8>,
    scratch: &mut EmitScratch,
) {
    let s = anchors.len();
    let mask = (1u32 << bits) - 1;
    // `non_lit_mask` has 1s in bits `bits+8 .. 31`: covers high bits of `code >= 256` AND bit 31 ANCHOR_TAG.
    debug_assert!(bits <= 23);
    let non_lit_mask: u32 = !((1u32 << (bits + 8)) - 1);

    let want_arena = n + 8 * SLAB_SIZE + 4 * INLINE;
    if arena.len() < want_arena {
        if want_arena > arena.capacity() {
            arena.reserve_exact(want_arena - arena.len());
        }
        arena.resize(want_arena, 0);
    }
    scratch.slices.clear();
    scratch.ordered.clear();

    let mut alloc = SlabAlloc {
        arena,
        top: 0,
        slices: &mut scratch.slices,
        seg_head: [u16::MAX; MAX_ANCHORS],
        seg_tail: [u16::MAX; MAX_ANCHORS],
    };

    let mut seg_end_row = [u32::MAX; MAX_ANCHORS];
    let mut lj = [0u32; 4];
    let mut lseg = [0usize; 4];
    let mut lstart = [0usize; 4];
    let mut lat = [0usize; 4];
    let mut lend = [0usize; 4];
    let mut lactive = [false; 4];
    let mut next_seg = 0usize;
    let mut steps_done = 0usize;

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn emit_one<const MODE: u8>(
        seg: usize,
        code: usize,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
        alloc: &mut SlabAlloc<'_>,
        slice_start: &mut usize,
        at: &mut usize,
        end: &mut usize,
    ) {
        if MODE == 0 {
            let len = lens[code];
            if len != LONG && *at + INLINE <= *end {
                alloc.arena[*at..*at + INLINE].copy_from_slice(&inline16[code]);
                *at += usize::from(len);
            } else {
                alloc.emit_slow_full(seg, code, len, inline16, spelling, slice_start, at, end);
            }
        } else if code < 256 {
            if *at < *end {
                alloc.arena[*at] = code as u8;
                *at += 1;
            } else {
                alloc.emit_lit_slab_switch(seg, code as u8, slice_start, at, end);
            }
        } else {
            let wi = code - 256;
            let len = lens[wi];
            if len != LONG && *at + INLINE <= *end {
                alloc.arena[*at..*at + INLINE].copy_from_slice(&inline16[wi]);
                *at += usize::from(len);
            } else {
                alloc.emit_slow_compact(seg, wi, len, inline16, spelling, slice_start, at, end);
            }
        }
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn start_segment<const MODE: u8>(
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
        steps_done: &mut usize,
    ) {
        lseg[lane] = seg;
        lstart[lane] = lat[lane];
        let edge = edges[anchor_row as usize] & !ANCHOR_TAG;
        lj[lane] = edge & mask;
        let code = (edge >> bits) as usize;
        emit_one::<MODE>(
            seg,
            code,
            inline16,
            lens,
            spelling,
            alloc,
            &mut lstart[lane],
            &mut lat[lane],
            &mut lend[lane],
        );
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
        start_segment::<MODE>(
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
            &mut steps_done,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_hit<const MODE: u8>(
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
        next_seg: &mut usize,
        steps_done: &mut usize,
        m: usize,
    ) -> bool {
        while (edges[lj[lane] as usize] as i32) < 0 {
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
                start_segment::<MODE>(
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
            handle_hit::<MODE>(
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
                &mut next_seg,
                &mut steps_done,
                m,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Phase 1: 4-lane counted hot loop
    // -----------------------------------------------------------------------
    if lactive[0] && lactive[1] && lactive[2] && lactive[3] {
        let (mut j0, mut j1, mut j2, mut j3) = (lj[0], lj[1], lj[2], lj[3]);
        let (mut a0, mut a1, mut a2, mut a3) = (lat[0], lat[1], lat[2], lat[3]);
        let (mut e0, mut e1, mut e2, mut e3) = (lend[0], lend[1], lend[2], lend[3]);

        loop {
            let max_iters = (m - steps_done) / 4;
            let mut done = 0usize;
            let mut hit = false;

            if MODE == 0 {
                // Arm 1: shipped inline16[code] / lens[code] on every step
                for _ in 0..max_iters {
                    let ed0 = edges[j0 as usize];
                    let ed1 = edges[j1 as usize];
                    let ed2 = edges[j2 as usize];
                    let ed3 = edges[j3 as usize];
                    if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                        hit = true;
                        break;
                    }
                    j0 = ed0 & mask;
                    j1 = ed1 & mask;
                    j2 = ed2 & mask;
                    j3 = ed3 & mask;

                    emit_one::<0>(
                        lseg[0],
                        (ed0 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                    emit_one::<0>(
                        lseg[1],
                        (ed1 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                    emit_one::<0>(
                        lseg[2],
                        (ed2 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                    emit_one::<0>(
                        lseg[3],
                        (ed3 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                    done += 1;
                }
            } else if MODE == 1 {
                // Arm 2: per-lane direct literal branch (`c < 256` -> 1B store, else compact `inline16[c - 256]`)
                for _ in 0..max_iters {
                    let ed0 = edges[j0 as usize];
                    let ed1 = edges[j1 as usize];
                    let ed2 = edges[j2 as usize];
                    let ed3 = edges[j3 as usize];
                    if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                        hit = true;
                        break;
                    }
                    j0 = ed0 & mask;
                    j1 = ed1 & mask;
                    j2 = ed2 & mask;
                    j3 = ed3 & mask;

                    emit_one::<1>(
                        lseg[0],
                        (ed0 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                    emit_one::<1>(
                        lseg[1],
                        (ed1 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                    emit_one::<1>(
                        lseg[2],
                        (ed2 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                    emit_one::<1>(
                        lseg[3],
                        (ed3 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                    done += 1;
                }
            } else if MODE == 2 {
                // Arm 3: all-4-literal fastpath (`(ed_or & non_lit_mask) == 0` checks all 4 anchors AND all 4 literals in one test!)
                for _ in 0..max_iters {
                    let ed0 = edges[j0 as usize];
                    let ed1 = edges[j1 as usize];
                    let ed2 = edges[j2 as usize];
                    let ed3 = edges[j3 as usize];
                    let ed_or = ed0 | ed1 | ed2 | ed3;
                    if (ed_or & non_lit_mask) == 0 && a0 < e0 && a1 < e1 && a2 < e2 && a3 < e3 {
                        j0 = ed0 & mask;
                        j1 = ed1 & mask;
                        j2 = ed2 & mask;
                        j3 = ed3 & mask;
                        alloc.arena[a0] = (ed0 >> bits) as u8;
                        alloc.arena[a1] = (ed1 >> bits) as u8;
                        alloc.arena[a2] = (ed2 >> bits) as u8;
                        alloc.arena[a3] = (ed3 >> bits) as u8;
                        a0 += 1;
                        a1 += 1;
                        a2 += 1;
                        a3 += 1;
                        done += 1;
                        continue;
                    }
                    if (ed_or as i32) < 0 {
                        hit = true;
                        break;
                    }
                    j0 = ed0 & mask;
                    j1 = ed1 & mask;
                    j2 = ed2 & mask;
                    j3 = ed3 & mask;
                    emit_one::<2>(
                        lseg[0],
                        (ed0 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                    emit_one::<2>(
                        lseg[1],
                        (ed1 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                    emit_one::<2>(
                        lseg[2],
                        (ed2 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                    emit_one::<2>(
                        lseg[3],
                        (ed3 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                    done += 1;
                }
            } else {
                // Arm 4 / Arm 5 (`lit_streak_inner`): hoisted slab-budget tight inner literal-streak loop
                while done < max_iters {
                    let rem = max_iters - done;
                    let lit_cap = rem.min(e0 - a0).min(e1 - a1).min(e2 - a2).min(e3 - a3);
                    let mut stop_edges = (0u32, 0u32, 0u32, 0u32);
                    let mut stopped = false;
                    if lit_cap > 0 {
                        let arena = &mut alloc.arena[..];
                        let alen = arena.len();
                        assert!(e0 <= alen && e1 <= alen && e2 <= alen && e3 <= alen);
                        let mut k = 0usize;
                        while k < lit_cap {
                            let ed0 = edges[j0 as usize];
                            let ed1 = edges[j1 as usize];
                            let ed2 = edges[j2 as usize];
                            let ed3 = edges[j3 as usize];
                            if ((ed0 | ed1 | ed2 | ed3) & non_lit_mask) != 0 {
                                stop_edges = (ed0, ed1, ed2, ed3);
                                stopped = true;
                                break;
                            }
                            j0 = ed0 & mask;
                            j1 = ed1 & mask;
                            j2 = ed2 & mask;
                            j3 = ed3 & mask;
                            arena[a0 + k] = (ed0 >> bits) as u8;
                            arena[a1 + k] = (ed1 >> bits) as u8;
                            arena[a2 + k] = (ed2 >> bits) as u8;
                            arena[a3 + k] = (ed3 >> bits) as u8;
                            k += 1;
                        }
                        a0 += k;
                        a1 += k;
                        a2 += k;
                        a3 += k;
                        done += k;
                        if done >= max_iters {
                            break;
                        }
                    }
                    let (ed0, ed1, ed2, ed3) = if stopped {
                        stop_edges
                    } else {
                        (
                            edges[j0 as usize],
                            edges[j1 as usize],
                            edges[j2 as usize],
                            edges[j3 as usize],
                        )
                    };
                    if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                        hit = true;
                        break;
                    }
                    j0 = ed0 & mask;
                    j1 = ed1 & mask;
                    j2 = ed2 & mask;
                    j3 = ed3 & mask;
                    emit_one::<3>(
                        lseg[0],
                        (ed0 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[0],
                        &mut a0,
                        &mut e0,
                    );
                    emit_one::<3>(
                        lseg[1],
                        (ed1 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[1],
                        &mut a1,
                        &mut e1,
                    );
                    emit_one::<3>(
                        lseg[2],
                        (ed2 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[2],
                        &mut a2,
                        &mut e2,
                    );
                    emit_one::<3>(
                        lseg[3],
                        (ed3 >> bits) as usize,
                        inline16,
                        lens,
                        spelling,
                        &mut alloc,
                        &mut lstart[3],
                        &mut a3,
                        &mut e3,
                    );
                    done += 1;
                }
            }

            steps_done += done * 4;
            lj = [j0, j1, j2, j3];
            lat = [a0, a1, a2, a3];
            lend = [e0, e1, e2, e3];

            if !hit {
                break;
            }
            let mut all_still = true;
            for lane in 0..4 {
                if (edges[lj[lane] as usize] as i32) < 0
                    && !handle_hit::<MODE>(
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
                        &mut next_seg,
                        &mut steps_done,
                        m,
                    )
                {
                    all_still = false;
                }
            }
            if !all_still {
                break;
            }
            (j0, j1, j2, j3) = (lj[0], lj[1], lj[2], lj[3]);
            (a0, a1, a2, a3) = (lat[0], lat[1], lat[2], lat[3]);
            (e0, e1, e2, e3) = (lend[0], lend[1], lend[2], lend[3]);
        }
    }

    // -----------------------------------------------------------------------
    // Phase 2: Tail drain (< 4 active lanes)
    // -----------------------------------------------------------------------
    while lactive[0] || lactive[1] || lactive[2] || lactive[3] {
        for lane in 0..4 {
            if !lactive[lane] {
                continue;
            }
            if steps_done >= m {
                lactive[lane] = false;
                continue;
            }
            let edge = edges[lj[lane] as usize];
            if (edge as i32) < 0 {
                handle_hit::<MODE>(
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
                    &mut next_seg,
                    &mut steps_done,
                    m,
                );
                continue;
            }
            lj[lane] = edge & mask;
            let code = (edge >> bits) as usize;
            emit_one::<MODE>(
                lseg[lane],
                code,
                inline16,
                lens,
                spelling,
                &mut alloc,
                &mut lstart[lane],
                &mut lat[lane],
                &mut lend[lane],
            );
            steps_done += 1;
            if (edges[lj[lane] as usize] as i32) < 0 {
                handle_hit::<MODE>(
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
                    &mut next_seg,
                    &mut steps_done,
                    m,
                );
            }
        }
    }

    // Chain S segments in chronological order starting at seg 0 (= mark)
    let mut cur_seg = 0usize;
    for _ in 0..s {
        let mut sid = alloc.seg_head[cur_seg];
        while sid != u16::MAX {
            let sr = alloc.slices[sid as usize];
            scratch.ordered.push((sr.off as usize, sr.len as usize));
            sid = sr.next;
        }
        let end_r = seg_end_row[cur_seg];
        if end_r == anchors[0] {
            break;
        }
        let mut found = None;
        for (k, &a) in anchors.iter().enumerate().take(s) {
            if a == end_r {
                found = Some(k);
                break;
            }
        }
        cur_seg = found.expect("anchor chain broken");
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dispatch_emit(
    arm: Arm,
    n: usize,
    m: usize,
    bits: u32,
    edges: &[u32],
    anchors: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    spelling: &[u8],
    arena: &mut Vec<u8>,
    scratch: &mut EmitScratch,
) {
    match arm.emit_mode() {
        0 => emit_4lane_mode::<0>(
            n, m, bits, edges, anchors, inline16, lens, spelling, arena, scratch,
        ),
        1 => emit_4lane_mode::<1>(
            n, m, bits, edges, anchors, inline16, lens, spelling, arena, scratch,
        ),
        2 => emit_4lane_mode::<2>(
            n, m, bits, edges, anchors, inline16, lens, spelling, arena, scratch,
        ),
        _ => emit_4lane_mode::<3>(
            n, m, bits, edges, anchors, inline16, lens, spelling, arena, scratch,
        ),
    }
}

// ---------------------------------------------------------------------------
// Worker State for Combined `A6b-dict + A6c + A7` Suite
// ---------------------------------------------------------------------------

struct WorkerState {
    dict_cache: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    inline16_c: Vec<[u8; INLINE]>,
    lens_c: Vec<u8>,
    spelling: Vec<u8>,
    stack: Vec<u32>,
    bands: Vec<Band>,
    edges: Vec<u32>,
    arena: Vec<u8>,
    emit: EmitScratch,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            dict_cache: vec![0u32; CACHE_WORDS],
            inline16: Vec::new(),
            lens: Vec::new(),
            inline16_c: Vec::new(),
            lens_c: Vec::new(),
            spelling: Vec::new(),
            stack: Vec::new(),
            bands: Vec::new(),
            edges: Vec::new(),
            arena: Vec::new(),
            emit: EmitScratch::default(),
        }
    }
}

#[derive(Default, Clone, Copy)]
struct PhaseNs {
    dict: u64,
    wire: u64,
    emit: u64,
}

impl WorkerState {
    #[inline(never)]
    fn run_dict_wire_emit(&mut self, pb: &mut PreparedBlock, arm: Arm, ph: &mut PhaseNs) {
        let t0 = Instant::now();
        let cursors = if arm.uses_compact_dict_build() {
            run_a6b_dict_compact(
                pb,
                &mut self.dict_cache,
                &mut self.inline16_c,
                &mut self.lens_c,
                &mut self.spelling,
                &mut self.stack,
                &mut self.bands,
            )
        } else {
            run_a6b_dict_shipped(
                pb,
                &mut self.dict_cache,
                &mut self.inline16,
                &mut self.lens,
                &mut self.spelling,
                &mut self.stack,
                &mut self.bands,
            )
        };
        let t1 = Instant::now();

        if self.edges.len() < pb.m {
            self.edges.resize(pb.m, 0);
        }
        wire_forward_bands(
            &pb.outq_bytes,
            &pb.spill,
            &pb.heads,
            &self.bands,
            &cursors,
            pb.bits,
            &mut self.edges[..pb.m],
        );
        let mut anchors = [0u32; MAX_ANCHORS];
        let anchor_cnt = tag_uniform_anchors(&mut self.edges[..pb.m], pb.m, pb.mark, &mut anchors);
        let t2 = Instant::now();

        let (inl_slice, len_slice): (&[[u8; INLINE]], &[u8]) = if arm == Arm::Ctrl4LaneDict {
            (&self.inline16, &self.lens)
        } else if arm.uses_compact_dict_build() {
            (&self.inline16_c, &self.lens_c)
        } else {
            (&self.inline16[256..], &self.lens[256..])
        };
        dispatch_emit(
            arm,
            pb.n,
            pb.m,
            pb.bits,
            &self.edges[..pb.m],
            &anchors[..anchor_cnt],
            inl_slice,
            len_slice,
            &self.spelling,
            &mut self.arena,
            &mut self.emit,
        );
        let t3 = Instant::now();

        ph.dict += t1.duration_since(t0).as_nanos() as u64;
        ph.wire += t2.duration_since(t1).as_nanos() as u64;
        ph.emit += t3.duration_since(t2).as_nanos() as u64;
    }
}

// ---------------------------------------------------------------------------
// Census, Timers, Stats & Verification
// ---------------------------------------------------------------------------

fn thread_cpu_ns() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/thread-self/schedstat") {
        if let Some(first) = s.split_whitespace().next() {
            if let Ok(v) = first.parse::<u64>() {
                return v;
            }
        }
    }
    0
}

fn flush_llc(buf: &mut [u8]) {
    let mut acc = 0u8;
    let mut i = 0usize;
    while i < buf.len() {
        buf[i] = buf[i].wrapping_add(1);
        acc ^= buf[i];
        i += 64;
    }
    black_box(acc);
}

fn median_min_max(vals: &[f64]) -> (f64, f64, f64) {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let mn = v[0];
    let mx = v[v.len() - 1];
    let med = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    (med, mn, mx)
}

fn run_census(blocks: &[PreparedBlock]) {
    let mut total_steps = 0u64;
    let mut total_bytes = 0u64;
    let mut lit_steps = 0u64;
    let mut short_word_steps = 0u64;
    let mut long_word_steps = 0u64;
    let mut lit_streaks = 0u64;
    let mut max_bits = 0u32;
    let mut max_code = 0usize;
    let mut total_full_dict_bytes = 0u64;
    let mut total_compact_dict_bytes = 0u64;

    // 4-lane all-literal iteration census (replaying the exact S=16 4-lane phase-1 iterations!)
    let mut iters_4lane = 0u64;
    let mut iters_all4_lit = 0u64;
    let mut all4_streaks = 0u64;

    for pb in blocks {
        total_steps += pb.m as u64;
        total_bytes += pb.n as u64;
        max_bits = max_bits.max(pb.bits);
        max_code = max_code.max(pb.lens_full.len() - 1);
        total_full_dict_bytes += (pb.inline16_full.len() * INLINE + pb.lens_full.len()) as u64;
        total_compact_dict_bytes +=
            (pb.inline16_compact.len() * INLINE + pb.lens_compact.len()) as u64;

        let mask = (1u32 << pb.bits) - 1;
        let mut j = pb.mark;
        let mut in_lit = false;
        for _ in 0..pb.m {
            let edge = pb.tagged_edges[j as usize] & !ANCHOR_TAG;
            let code = (edge >> pb.bits) as usize;
            if code < 256 {
                lit_steps += 1;
                if !in_lit {
                    lit_streaks += 1;
                    in_lit = true;
                }
            } else {
                in_lit = false;
                if pb.lens_full[code] == LONG {
                    long_word_steps += 1;
                } else {
                    short_word_steps += 1;
                }
            }
            j = edge & mask;
        }

        // Exact 4-lane all-literal iteration count across S=16 anchors
        let s = pb.anchor_cnt;
        let mut lj = [0u32; 4];
        let mut lactive = [true; 4];
        let mut next_seg = 4usize;
        for (lane, jslot) in lj.iter_mut().enumerate() {
            let e = pb.tagged_edges[pb.anchors[lane] as usize] & !ANCHOR_TAG;
            *jslot = e & mask;
        }
        let mut steps_done = 4usize;
        let non_lit_mask: u32 = !((1u32 << (pb.bits + 8)) - 1);
        let mut in_all4 = false;
        loop {
            for lane in 0..4 {
                while lactive[lane] && (pb.tagged_edges[lj[lane] as usize] as i32) < 0 {
                    if next_seg < s && steps_done < pb.m {
                        let e = pb.tagged_edges[pb.anchors[next_seg] as usize] & !ANCHOR_TAG;
                        lj[lane] = e & mask;
                        next_seg += 1;
                        steps_done += 1;
                    } else {
                        lactive[lane] = false;
                    }
                }
            }
            if !(lactive[0] && lactive[1] && lactive[2] && lactive[3]) || steps_done + 4 > pb.m {
                break;
            }
            let ed0 = pb.tagged_edges[lj[0] as usize];
            let ed1 = pb.tagged_edges[lj[1] as usize];
            let ed2 = pb.tagged_edges[lj[2] as usize];
            let ed3 = pb.tagged_edges[lj[3] as usize];
            let ed_or = ed0 | ed1 | ed2 | ed3;
            if (ed_or as i32) < 0 {
                in_all4 = false;
                continue;
            }
            iters_4lane += 1;
            if (ed_or & non_lit_mask) == 0 {
                iters_all4_lit += 1;
                if !in_all4 {
                    all4_streaks += 1;
                    in_all4 = true;
                }
            } else {
                in_all4 = false;
            }
            lj[0] = ed0 & mask;
            lj[1] = ed1 & mask;
            lj[2] = ed2 & mask;
            lj[3] = ed3 & mask;
            steps_done += 4;
        }
    }

    println!("\n=== STRUCTURAL LITERAL-STREAK & DICTIONARY CENSUS (233 admitted Open blocks) ===");
    println!(
        "  total A7 steps = {} ({:.2}M) | decoded bytes = {} ({:.2} MiB)",
        total_steps,
        total_steps as f64 / 1e6,
        total_bytes,
        total_bytes as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  1-byte literals (code < 256): {} ({:.2}% of steps), avg sequential lit streak = {:.2}",
        lit_steps,
        100.0 * lit_steps as f64 / total_steps as f64,
        lit_steps as f64 / lit_streaks.max(1) as f64
    );
    println!(
        "  multi-byte inline (2..16B):   {} ({:.2}% of steps)",
        short_word_steps,
        100.0 * short_word_steps as f64 / total_steps as f64
    );
    println!(
        "  long words (len == LONG):     {} ({:.2}% of steps)",
        long_word_steps,
        100.0 * long_word_steps as f64 / total_steps as f64
    );
    println!(
        "  4-lane Phase-1 iters:         {} total | all-4-literal = {} ({:.2}%), avg all-4 streak = {:.2}",
        iters_4lane,
        iters_all4_lit,
        100.0 * iters_all4_lit as f64 / iters_4lane.max(1) as f64,
        iters_all4_lit as f64 / all4_streaks.max(1) as f64
    );
    println!(
        "  bit budget: max_bits={} max_code={} (fits 31-bit tag: 233/233)",
        max_bits, max_code
    );
    println!(
        "  avg dict table/blk (inline16+lens): full {:.2} KiB -> compact {:.2} KiB (delta -{:.2} KiB/blk)",
        total_full_dict_bytes as f64 / (blocks.len() as f64 * 1024.0),
        total_compact_dict_bytes as f64 / (blocks.len() as f64 * 1024.0),
        (total_full_dict_bytes - total_compact_dict_bytes) as f64 / (blocks.len() as f64 * 1024.0),
    );
}

fn verify_exact(blocks: &mut [PreparedBlock]) {
    println!("\n=== BYTE-EXACTNESS AUDIT (233 admitted Open blocks vs bwt::walk) ===");
    let mut worker = WorkerState::default();
    let mut scratch = EmitScratch::default();
    let mut arena = Vec::new();
    let mut stitched = Vec::new();

    for &arm in ARMS {
        let mut ok_a7 = 0usize;
        let mut ok_full = 0usize;
        for (bi, pb) in blocks.iter_mut().enumerate() {
            // 1. Isolated A7 check
            let (inl, len): (&[[u8; INLINE]], &[u8]) = if arm == Arm::Ctrl4LaneDict {
                (&pb.inline16_full, &pb.lens_full)
            } else {
                (&pb.inline16_compact, &pb.lens_compact)
            };
            dispatch_emit(
                arm,
                pb.n,
                pb.m,
                pb.bits,
                &pb.tagged_edges,
                &pb.anchors[..pb.anchor_cnt],
                inl,
                len,
                &pb.spelling,
                &mut arena,
                &mut scratch,
            );
            stitched.clear();
            for &(off, slen) in &scratch.ordered {
                stitched.extend_from_slice(&arena[off..off + slen]);
            }
            assert_eq!(
                stitched,
                pb.expected,
                "A7-only MISMATCH on {} blk {bi}",
                arm.name()
            );
            ok_a7 += 1;

            // 2. Combined A6b-dict + A6c + A7 check
            let mut ph = PhaseNs::default();
            worker.run_dict_wire_emit(pb, arm, &mut ph);
            pb.reset_elim_tags();
            stitched.clear();
            for &(off, slen) in &worker.emit.ordered {
                stitched.extend_from_slice(&worker.arena[off..off + slen]);
            }
            assert_eq!(
                stitched,
                pb.expected,
                "A6b-dict+A6c+A7 MISMATCH on {} blk {bi}",
                arm.name()
            );
            ok_full += 1;
        }
        println!(
            "  EXACT {:<22} A7-only {}/{} | A6b-dict+A6c+A7 {}/{}",
            arm.name(),
            ok_a7,
            blocks.len(),
            ok_full,
            blocks.len()
        );
    }
    // Cross-arm reuse guard: verify Ctrl4LaneDict again on the same WorkerState after LitPlusCompactDict
    for (bi, pb) in blocks.iter_mut().enumerate() {
        let mut ph = PhaseNs::default();
        worker.run_dict_wire_emit(pb, Arm::Ctrl4LaneDict, &mut ph);
        pb.reset_elim_tags();
        stitched.clear();
        for &(off, slen) in &worker.emit.ordered {
            stitched.extend_from_slice(&worker.arena[off..off + slen]);
        }
        assert_eq!(
            stitched, pb.expected,
            "post-compact reuse MISMATCH blk {bi}"
        );
    }
    println!(
        "  EXACT post-compact reuse check: ctrl_4lane_dict {}/{} byte-exact after lit_plus_compact_dict",
        blocks.len(),
        blocks.len()
    );

    // Free oracle expected buffers (~210 MiB) before benchmark sweeps
    for pb in blocks.iter_mut() {
        pb.expected = Vec::new();
    }
}

#[derive(Default, Clone, Copy)]
struct RunSample {
    cpu_ns: u64,
    wall_ns: u64,
    ph: PhaseNs,
}

fn print_a7_table(
    title: &str,
    samples: &[Vec<RunSample>],
    use_cpu: bool,
    div: f64,
) -> (String, f64, bool) {
    println!("\n=== {title} ===");
    println!(
        "{:<22} {:>10}  {:<21} {:>8}  {:>7}  {:>11}",
        "arm", "med(ms)", "[min..max]", "vs_ctrl", "disj", "i9_A7_delta"
    );
    let series: Vec<Vec<f64>> = samples
        .iter()
        .map(|v| {
            v.iter()
                .map(|r| (if use_cpu { r.cpu_ns } else { r.wall_ns }) as f64 / (1e6 * div))
                .collect()
        })
        .collect();
    let (c_med, c_min, c_max) = median_min_max(&series[0]);
    let mut best_name = String::new();
    let mut best_pct = 0.0f64;
    let mut best_disj = false;

    for (i, &arm) in ARMS.iter().enumerate() {
        let (med, mn, mx) = median_min_max(&series[i]);
        let pct = 100.0 * (med - c_med) / c_med;
        let disj = if i == 0 {
            "base"
        } else if mx < c_min {
            "FASTER"
        } else if mn > c_max {
            "SLOWER"
        } else {
            "overlap"
        };
        let i9_delta_1w = I9_A7_FWD4_EST_MS * (c_med - med) / c_med;
        println!(
            "{:<22} {:>10.2}  [{:>7.2}..{:>7.2}] {:>+7.2}%  {:>7}  {:>+8.1} ms",
            arm.name(),
            med,
            mn,
            mx,
            pct,
            disj,
            i9_delta_1w
        );
        if i > 0 {
            let cut_pct = -pct;
            if cut_pct > best_pct {
                best_pct = cut_pct;
                best_name = arm.name().to_string();
                best_disj = disj == "FASTER";
            }
        }
    }
    let gate_pass = best_pct >= STOP_BAR_PCT && best_disj;
    println!(
        "  -> Best vs ctrl: {} ({:+.2}% cut, disj={}) | Frozen >={:.1}% gate: {}",
        if best_name.is_empty() {
            "none"
        } else {
            &best_name
        },
        best_pct,
        best_disj,
        STOP_BAR_PCT,
        if gate_pass { "PASS" } else { "STOP" }
    );
    (best_name, best_pct, gate_pass)
}

fn print_combined_table(title: &str, samples: &[Vec<RunSample>], use_cpu: bool) {
    println!("\n=== {title} ===");
    println!(
        "{:<22} {:>8} {:>7} {:>8} {:>10}  {:<21} {:>8}  {:>7} {:>9}",
        "arm", "A6b-dct", "A6c", "A7", "tot(ms)", "[min..max]", "vs_ctrl", "disj", "d+A7_vs_A7"
    );
    let series: Vec<Vec<f64>> = samples
        .iter()
        .map(|v| {
            v.iter()
                .map(|r| (if use_cpu { r.cpu_ns } else { r.wall_ns }) as f64 / 1e6)
                .collect()
        })
        .collect();
    let (c_med, c_min, c_max) = median_min_max(&series[0]);

    let phase_med = |idx: usize, sel: fn(&PhaseNs) -> u64| -> f64 {
        let tot = median_min_max(&series[idx]).0;
        let v: Vec<f64> = samples[idx]
            .iter()
            .map(|r| {
                let s = (r.ph.dict + r.ph.wire + r.ph.emit).max(1) as f64;
                tot * (sel(&r.ph) as f64 / s)
            })
            .collect();
        median_min_max(&v).0
    };

    let c_dict = phase_med(0, |p| p.dict);
    let c_emit = phase_med(0, |p| p.emit);

    for (i, &arm) in ARMS.iter().enumerate() {
        let (med, mn, mx) = median_min_max(&series[i]);
        let p_dict = phase_med(i, |p| p.dict);
        let p_wire = phase_med(i, |p| p.wire);
        let p_emit = phase_med(i, |p| p.emit);
        let pct = 100.0 * (med - c_med) / c_med;
        let disj = if i == 0 {
            "base"
        } else if mx < c_min {
            "FASTER"
        } else if mn > c_max {
            "SLOWER"
        } else {
            "overlap"
        };
        let da7_pct_of_a7 = 100.0 * ((p_dict + p_emit) - (c_dict + c_emit)) / c_emit.max(1e-9);
        println!(
            "{:<22} {:>8.2} {:>7.2} {:>8.2} {:>10.2}  [{:>7.2}..{:>7.2}] {:>+7.2}%  {:>7} {:>+8.2}%",
            arm.name(),
            p_dict,
            p_wire,
            p_emit,
            med,
            mn,
            mx,
            pct,
            disj,
            da7_pct_of_a7
        );
    }
}

fn load_admitted(path: &str) -> Vec<PreparedBlock> {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut l_col = Vec::new();
    let mut counts = [0u32; 256];
    let mut builder = OpenBuilder::default();
    let mut scratch_out = Vec::new();
    let mut tvec = TVector::default();
    let k = Kernels::detect();
    let mut admitted = Vec::new();
    let mut total = 0usize;

    while let Event::Block { symbols: b, .. } = sc
        .next(std::mem::take(&mut syms))
        .unwrap_or_else(|e| panic!("scan error: {e:?}"))
    {
        total += 1;
        mtf::mtf(&b, &mut l_col, &mut counts, k).expect("mtf");
        let n = l_col.len();
        let p = b.orig_ptr;
        let recs = records_of(&l_col);
        let is_admitted = match open_cap(recs.len(), n, p, false) {
            Some(Cap::NONE) => true,
            Some(cap) => within_cap(&recs, p, PROD_LIMIT, cap),
            None => false,
        };
        if is_admitted {
            build(&l_col, &counts, &mut tvec);
            let mut expected = Vec::with_capacity(n);
            walk(&tvec, p, &mut expected);
            admitted.push(builder.prepare_block(
                &recs,
                n,
                p,
                PROD_LIMIT,
                &mut scratch_out,
                expected,
            ));
        }
        syms = b.syms;
    }
    println!(
        "Loaded {} total blocks from {}; {} production-admitted Open blocks (I9_A7_SHIPPED={:.1}ms, I9_A7_FWD4_EST={:.1}ms).",
        total,
        path,
        admitted.len(),
        I9_A7_SHIPPED_MS,
        I9_A7_FWD4_EST_MS
    );
    admitted
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut path = "/tmp/wikidata.bz2".to_string();
    let mut repeats = 7usize;
    let mut i = 1usize;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("valid --repeats");
            i += 2;
        } else {
            path = args[i].clone();
            i += 1;
        }
    }

    println!(
        "direct_literal_emit_probe corpus={} repeats={} target_avx2={} SLAB=32KiB S={} frozen_bar=>={:.1}%_A7",
        path,
        repeats,
        cfg!(target_feature = "avx2"),
        S_ANCHORS,
        STOP_BAR_PCT
    );

    let mut blocks = load_admitted(&path);
    run_census(&blocks);
    verify_exact(&mut blocks);

    let mut flush_buf = vec![0u8; COLD_FLUSH_BYTES];

    // -----------------------------------------------------------------------
    // Suite 1: A7-only WARM (single reused arena, WARM_A7_INNER_REPS per block)
    // -----------------------------------------------------------------------
    let mut warm_arena = Vec::new();
    let mut warm_scratch = EmitScratch::default();
    let mut res_a7_warm: Vec<Vec<RunSample>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    // Warmup
    for pb in &blocks {
        dispatch_emit(
            Arm::Ctrl4LaneDict,
            pb.n,
            pb.m,
            pb.bits,
            &pb.tagged_edges,
            &pb.anchors[..pb.anchor_cnt],
            &pb.inline16_full,
            &pb.lens_full,
            &pb.spelling,
            &mut warm_arena,
            &mut warm_scratch,
        );
    }
    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..ARMS.len()).collect()
        } else {
            (0..ARMS.len()).rev().collect()
        };
        for idx in order {
            let arm = ARMS[idx];
            let c0 = thread_cpu_ns();
            let w0 = Instant::now();
            for pb in &blocks {
                let (inl, len): (&[[u8; INLINE]], &[u8]) = if arm == Arm::Ctrl4LaneDict {
                    (&pb.inline16_full, &pb.lens_full)
                } else {
                    (&pb.inline16_compact, &pb.lens_compact)
                };
                for _ in 0..WARM_A7_INNER_REPS {
                    dispatch_emit(
                        arm,
                        pb.n,
                        pb.m,
                        pb.bits,
                        &pb.tagged_edges,
                        &pb.anchors[..pb.anchor_cnt],
                        inl,
                        len,
                        &pb.spelling,
                        &mut warm_arena,
                        &mut warm_scratch,
                    );
                }
                black_box(warm_scratch.ordered.len());
            }
            let wall_ns = w0.elapsed().as_nanos() as u64;
            let cpu_ns = thread_cpu_ns().saturating_sub(c0);
            res_a7_warm[idx].push(RunSample {
                cpu_ns,
                wall_ns,
                ph: PhaseNs::default(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Suite 2: A7-only L3-COLD (16 rotating arenas + 64 MiB flush + 106 MiB edges)
    // -----------------------------------------------------------------------
    let mut cold_arenas: Vec<Vec<u8>> = (0..COLD_POOL_WORKERS).map(|_| Vec::new()).collect();
    let mut cold_scratches: Vec<EmitScratch> = (0..COLD_POOL_WORKERS)
        .map(|_| EmitScratch::default())
        .collect();
    let mut res_a7_cold: Vec<Vec<RunSample>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    // Pre-size pool arenas once so allocation doesn't perturb repeat 0
    for w in 0..COLD_POOL_WORKERS {
        let pb = &blocks[w % blocks.len()];
        dispatch_emit(
            Arm::Ctrl4LaneDict,
            pb.n,
            pb.m,
            pb.bits,
            &pb.tagged_edges,
            &pb.anchors[..pb.anchor_cnt],
            &pb.inline16_full,
            &pb.lens_full,
            &pb.spelling,
            &mut cold_arenas[w],
            &mut cold_scratches[w],
        );
    }
    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..ARMS.len()).collect()
        } else {
            (0..ARMS.len()).rev().collect()
        };
        for idx in order {
            let arm = ARMS[idx];
            flush_llc(&mut flush_buf);
            let c0 = thread_cpu_ns();
            let w0 = Instant::now();
            for (bi, pb) in blocks.iter().enumerate() {
                let w = bi % COLD_POOL_WORKERS;
                let (inl, len): (&[[u8; INLINE]], &[u8]) = if arm == Arm::Ctrl4LaneDict {
                    (&pb.inline16_full, &pb.lens_full)
                } else {
                    (&pb.inline16_compact, &pb.lens_compact)
                };
                dispatch_emit(
                    arm,
                    pb.n,
                    pb.m,
                    pb.bits,
                    &pb.tagged_edges,
                    &pb.anchors[..pb.anchor_cnt],
                    inl,
                    len,
                    &pb.spelling,
                    &mut cold_arenas[w],
                    &mut cold_scratches[w],
                );
                black_box(cold_scratches[w].ordered.len());
            }
            let wall_ns = w0.elapsed().as_nanos() as u64;
            let cpu_ns = thread_cpu_ns().saturating_sub(c0);
            res_a7_cold[idx].push(RunSample {
                cpu_ns,
                wall_ns,
                ph: PhaseNs::default(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Suite 3: Combined A6b-dict + A6c + A7 WARM (single reused WorkerState)
    // -----------------------------------------------------------------------
    let mut warm_worker = WorkerState::default();
    let mut res_comb_warm: Vec<Vec<RunSample>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    // Warmup
    for pb in blocks.iter_mut() {
        let mut ph = PhaseNs::default();
        warm_worker.run_dict_wire_emit(pb, Arm::Ctrl4LaneDict, &mut ph);
    }
    for pb in blocks.iter_mut() {
        pb.reset_elim_tags();
    }
    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..ARMS.len()).collect()
        } else {
            (0..ARMS.len()).rev().collect()
        };
        for idx in order {
            let arm = ARMS[idx];
            let mut ph = PhaseNs::default();
            let c0 = thread_cpu_ns();
            let w0 = Instant::now();
            for pb in blocks.iter_mut() {
                warm_worker.run_dict_wire_emit(pb, arm, &mut ph);
                black_box(warm_worker.emit.ordered.len());
            }
            let wall_ns = w0.elapsed().as_nanos() as u64;
            let cpu_ns = thread_cpu_ns().saturating_sub(c0);
            for pb in blocks.iter_mut() {
                pb.reset_elim_tags();
            }
            res_comb_warm[idx].push(RunSample {
                cpu_ns,
                wall_ns,
                ph,
            });
        }
    }

    // -----------------------------------------------------------------------
    // Suite 4: Combined A6b-dict + A6c + A7 L3-COLD (16 WorkerStates >50MiB + 64MiB flush)
    // -----------------------------------------------------------------------
    let mut cold_workers: Vec<WorkerState> = (0..COLD_POOL_WORKERS)
        .map(|_| WorkerState::default())
        .collect();
    let mut res_comb_cold: Vec<Vec<RunSample>> = vec![Vec::with_capacity(repeats); ARMS.len()];
    // Pre-size worker pool buffers once
    for (bi, pb) in blocks.iter_mut().enumerate().take(COLD_POOL_WORKERS * 2) {
        let mut ph = PhaseNs::default();
        cold_workers[bi % COLD_POOL_WORKERS].run_dict_wire_emit(pb, Arm::Ctrl4LaneDict, &mut ph);
    }
    for pb in blocks.iter_mut() {
        pb.reset_elim_tags();
    }
    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..ARMS.len()).collect()
        } else {
            (0..ARMS.len()).rev().collect()
        };
        for idx in order {
            let arm = ARMS[idx];
            flush_llc(&mut flush_buf);
            let mut ph = PhaseNs::default();
            let c0 = thread_cpu_ns();
            let w0 = Instant::now();
            for (bi, pb) in blocks.iter_mut().enumerate() {
                let w = &mut cold_workers[bi % COLD_POOL_WORKERS];
                w.run_dict_wire_emit(pb, arm, &mut ph);
                black_box(w.emit.ordered.len());
            }
            let wall_ns = w0.elapsed().as_nanos() as u64;
            let cpu_ns = thread_cpu_ns().saturating_sub(c0);
            for pb in blocks.iter_mut() {
                pb.reset_elim_tags();
            }
            res_comb_cold[idx].push(RunSample {
                cpu_ns,
                wall_ns,
                ph,
            });
        }
    }

    // -----------------------------------------------------------------------
    // Reports
    // -----------------------------------------------------------------------
    let (_, _, pass_w) = print_a7_table(
        &format!(
            "A7-ONLY WARM CPU (schedstat, {} inner reps/blk, {} interleaved)",
            WARM_A7_INNER_REPS, repeats
        ),
        &res_a7_warm,
        true,
        WARM_A7_INNER_REPS as f64,
    );
    print_a7_table(
        &format!(
            "A7-ONLY WARM WALL (Instant, {} inner reps/blk, {} interleaved)",
            WARM_A7_INNER_REPS, repeats
        ),
        &res_a7_warm,
        false,
        WARM_A7_INNER_REPS as f64,
    );
    let (_, _, pass_c) = print_a7_table(
        &format!(
            "A7-ONLY L3-COLD CPU (schedstat, 16W arenas + 64MiB flush + 106MiB edges, {} interleaved)",
            repeats
        ),
        &res_a7_cold,
        true,
        1.0,
    );
    print_a7_table(
        &format!(
            "A7-ONLY L3-COLD WALL (Instant, 16W arenas + 64MiB flush + 106MiB edges, {} interleaved)",
            repeats
        ),
        &res_a7_cold,
        false,
        1.0,
    );

    print_combined_table(
        &format!(
            "COMBINED A6b-dict + A6c + A7 WARM CPU (schedstat, reused worker, {} interleaved)",
            repeats
        ),
        &res_comb_warm,
        true,
    );
    print_combined_table(
        &format!(
            "COMBINED A6b-dict + A6c + A7 WARM WALL (Instant, reused worker, {} interleaved)",
            repeats
        ),
        &res_comb_warm,
        false,
    );
    print_combined_table(
        &format!(
            "COMBINED A6b-dict + A6c + A7 L3-COLD CPU (schedstat, 16W pool + 64MiB flush, {} interleaved)",
            repeats
        ),
        &res_comb_cold,
        true,
    );
    print_combined_table(
        &format!(
            "COMBINED A6b-dict + A6c + A7 L3-COLD WALL (Instant, 16W pool + 64MiB flush, {} interleaved)",
            repeats
        ),
        &res_comb_cold,
        false,
    );

    println!(
        "\n=== BUILD VERDICT (frozen >={:.1}% A7 cut, disjoint warm & cold CPU) ===",
        STOP_BAR_PCT
    );
    println!(
        "  A7-only WARM pass={} | A7-only L3-COLD pass={} => {}",
        pass_w,
        pass_c,
        if pass_w && pass_c {
            "BUILD PASS"
        } else {
            "BUILD STOP"
        }
    );
}
