#![forbid(unsafe_code)]
//! Cycle 13 Composition Candidate: `BandCoal + NoMarkXor` (Arm X, `wire_xor` preserved) &
//! `BandCoal + CleanPlainUni` (Arm Y, 1-write `wire` + `emit_uni`).
//!
//! Zero dead fields, single-kernel (`lazy: LazyCells` fallback, `reserve_exact`), fast `Pass` S1
//! with zero-register-cost packed `mark_and_byte`, peeled-first-segment register-held adjacent
//! band coalescing (`-27.7%` bands on 431 blocks, `-22.1%` on 6 blocks), and hoisted-mark
//! `wire_xor::<CHECK_MARK>` (`NoMarkXor`, skipping `v == mark` on 255/256 bytes).
//!
//! Conforms to mibz2's `bench/moonshot` `crate::candidate` contract (`v0.5.12` / `ee7fbf0e`).

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;
use std::time::Instant;

/// Locked candidate for `bench/moonshot`: strongest exact arm (`BandCoal + NoMarkXor`, `wire_xor` preserved)
pub type Candidate = BandCoalNoMark;

pub type LeanEmb = LeanOpenArm<false, false, false>;
pub type NoMarkOnly = LeanOpenArm<false, true, false>;
pub type BandCoalOnly = LeanOpenArm<true, false, false>;
/// Arm X: BandCoal + NoMarkXor (`wire_xor` + `emit_bidirectional` preserved, `maxCh <= 120`, `lazy: LazyCells`)
pub type BandCoalNoMark = LeanOpenArm<true, true, false>;
pub type CleanPlainUni = LeanOpenArm<false, false, true>;
/// Arm Y: BandCoal + CleanPlainUni (1-write `wire` + `emit_uni`, `lazy: LazyCells`)
pub type BandCoalPlainUni = LeanOpenArm<true, false, true>;

/// Exact per-stage timing and structural metrics for the Cycle 13 benchmark harness
#[derive(Default, Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct StageStats {
    pub n: usize,
    pub retained: usize,
    pub thick: usize,
    pub bands: usize,
    pub dict_words: usize,
    pub spelling_bytes: usize,
    pub peak_scratch_bytes: usize,
    pub t_s1_ns: u64,
    pub t_s2_ns: u64,
    pub t_s3a_ns: u64,
    pub t_s4_ns: u64,
    pub t_emit_ns: u64,
}

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

#[inline(always)]
fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cap {
    m: u32,
    strips: usize,
}

impl Cap {
    const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };
    #[inline(always)]
    fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
}

#[inline(always)]
fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
    if n == 0 || n > 900_000 || p as usize >= n {
        None
    } else if lazy_counts_admit(records, n) {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
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
        debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
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

// ============================================================================
// OutQ16 queues in `out` with hoisted-mark `wire_xor::<CHECK_MARK>` & 1-write `wire`
// ============================================================================
mod outq {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub(super) struct Spill {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub(super) fn scratch_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub(super) struct Queues<'a> {
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

    pub(super) struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    impl<'a> Queues<'a> {
        pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            Queues {
                in_out: (out.len() / BYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE; 256],
                tails: [NONE; 256],
                fill: [CHUNK as u32; 256],
                limit,
                over: false,
            }
        }

        pub(super) fn over(&self) -> bool {
            self.over
        }

        pub(super) fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }

        #[inline(always)]
        pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = self.tails[a];
            if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
                let at = tail as usize * BYTES + 2 * f;
                let iota: [u16; 16] =
                    std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
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
                self.spill.ovf.push([0; BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE);
            }
            self.spill.next[id as usize] = NONE;
            if self.heads[a] == NONE {
                self.heads[a] = id;
            } else {
                self.spill.next[self.tails[a] as usize] = id;
            }
            self.tails[a] = id;
            self.fill[a] = 0;
        }

        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    pub(super) struct Reader<'a> {
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
        pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
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
        pub(super) fn wire_xor<const CHECK_MARK: bool>(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if CHECK_MARK && v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                if CHECK_MARK {
                    *predecessor = before;
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow::<CHECK_MARK>(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow<const CHECK_MARK: bool>(
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
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
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
                    if CHECK_MARK && v == mark {
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
        pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * BYTES + 2 * self.at;
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
                    let at = self.id as usize * BYTES;
                    &self.out[at..at + BYTES]
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
}

// ============================================================================
// RV Endpoint (`v0.5.12` + peeled-first-segment single-branch `BandCoal`)
// ============================================================================
mod endpoint {
    use super::*;

    const RULE: usize = 4;
    const ID_BITS: u32 = 22;
    const ID_MASK: u32 = (1 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const NARROW_NODES: usize = 1 << 16;

    #[inline(always)]
    fn literal(byte: u8) -> u32 {
        (1 << ID_BITS) | (u32::from(byte) + 1)
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

        #[inline(always)]
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
        #[inline(always)]
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
            let memoized =
                handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
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

        #[inline(always)]
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

    #[inline(always)]
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    pub(super) fn arena_words(ns: usize, m: usize) -> usize {
        let stride = if nodes_most(ns) > NARROW_NODES {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * nodes_most(ns)).max(m)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn induced<const BAND_COAL: bool, const TIMED: bool>(
        edges: &mut Vec<u32>,
        stack: &mut Vec<u32>,
        bands: &mut Vec<Band>,
        spelling: &mut Vec<u8>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        stats: &mut StageStats,
    ) -> [usize; 256] {
        let most = nodes_most(ns);
        if most > NARROW_NODES {
            induced_with::<Wide, BAND_COAL, TIMED>(
                edges, stack, bands, spelling, inline16, lens, ns, n, m, thin, stats, most,
            )
        } else {
            induced_with::<Narrow, BAND_COAL, TIMED>(
                edges, stack, bands, spelling, inline16, lens, ns, n, m, thin, stats, most,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links, const BAND_COAL: bool, const TIMED: bool>(
        edges: &mut Vec<u32>,
        stack: &mut Vec<u32>,
        bands: &mut Vec<Band>,
        spelling: &mut Vec<u8>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        stats: &mut StageStats,
        most: usize,
    ) -> [usize; 256] {
        let t_s2 = if TIMED { Some(Instant::now()) } else { None };
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
        if let Some(t0) = t_s2 {
            stats.t_s2_ns = t0.elapsed().as_nanos() as u64;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
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
            if !BAND_COAL {
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
            } else {
                if k == end {
                    continue;
                }
                let (mut cur_dst, mut cur_dst_end, mut cur_end, mut cur_word, mut cur_code) = {
                    let node_width = e.width(id);
                    let node_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = e.bottom(id) + in_node;
                    let code = dictionary.code(&mut e, node_word, a as u8);
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                    (dst, dst + width, k, node_word, code)
                };
                while k < end {
                    let node_width = e.width(id);
                    let node_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = e.bottom(id) + in_node;
                    if dst != cur_dst_end {
                        bands.push(Band::new(cur_end, cur_dst, cur_code));
                        cur_code = dictionary.code(&mut e, node_word, a as u8);
                        cur_dst = dst;
                        cur_dst_end = dst + width;
                        cur_end = k + width;
                        cur_word = node_word;
                    } else if node_word == cur_word {
                        cur_dst_end += width;
                        cur_end = k + width;
                    } else {
                        let code = dictionary.code(&mut e, node_word, a as u8);
                        if code == cur_code {
                            cur_dst_end += width;
                            cur_end = k + width;
                            cur_word = node_word;
                        } else {
                            bands.push(Band::new(cur_end, cur_dst, cur_code));
                            cur_dst = dst;
                            cur_dst_end = dst + width;
                            cur_end = k + width;
                            cur_word = node_word;
                            cur_code = code;
                        }
                    }
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
                bands.push(Band::new(cur_end, cur_dst, cur_code));
            }
        }
        assert_eq!(id, 0);
        if let Some(t0) = t_s3a {
            stats.t_s3a_ns = t0.elapsed().as_nanos() as u64;
        }
        cursors
    }
}

// ============================================================================
// Inline Word Table, Emitters & Fast First Pass
// ============================================================================
#[inline(always)]
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

#[allow(clippy::too_many_arguments)]
fn emit_uni(
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

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark_and_byte: u64,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark_and_byte: 0,
        }
    }
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) -> u32 {
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
    pos
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

struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        if width <= LIMIT {
            self.piece(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece(c, pos, width, false, p);
        } else {
            self.take_marked(c, pos, width, p);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        split(c, pos, width, p, LIMIT, |c, x, w, keep| {
            self.piece(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark_and_byte = (u64::from(c) << 32) | u64::from(f.m + (p - x));
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

// ============================================================================
// Single-Kernel, Zero-Dead-Field `LeanOpenArm` (`lazy: LazyCells`)
// ============================================================================
#[derive(Default)]
pub struct LeanOpenArm<const BAND_COAL: bool, const NO_MARK: bool, const PLAIN_UNI: bool> {
    lazy: LazyCells,
    stack: Vec<u32>,
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

impl<const BAND_COAL: bool, const NO_MARK: bool, const PLAIN_UNI: bool>
    LeanOpenArm<BAND_COAL, NO_MARK, PLAIN_UNI>
{
    #[allow(dead_code)]
    pub fn scratch_bytes(&self) -> usize {
        self.spill.scratch_bytes()
            + 8 * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * (self.edges.capacity() + self.extra.capacity() + self.stack.capacity())
            + 16 * self.inline16.capacity()
            + self.lens.capacity()
    }

    #[allow(dead_code)]
    pub fn invert_runs_timed(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> (bool, Option<StageStats>) {
        if let Some(cap) = open_cap(r.records().len(), n, p) {
            if let Some(st) = self.try_run::<true>(r.records(), n, p, cap, out) {
                return (true, Some(st));
            }
        }
        (self.lazy.invert_runs(r, n, p, out, k), None)
    }

    fn try_run<const TIMED: bool>(
        &mut self,
        records: &[u16],
        n_usize: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<StageStats> {
        if n_usize == 0 || n_usize > 900_000 || p as usize >= n_usize {
            return None;
        }
        let n = n_usize as u32;
        let mut stats = StageStats {
            n: n_usize,
            ..StageStats::default()
        };

        // ---- S1: First pass (OutQ16 + thick strips + packed mark_byte + image_order) ----
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n_usize + INLINE, 0);
        let (f, ns, heads, over, total_pos) = {
            let mut pass = Pass {
                queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.m),
                edges: &mut self.edges,
                ns: 0,
                cap_strips: cap.strips,
                f: First::default(),
            };
            let pos = maximal(records, |c, pos, width| pass.take(c, pos, width, p));
            (pass.f, pass.ns, pass.queues.heads(), pass.queues.over(), pos)
        };
        if total_pos != n || over || f.m == 0 || f.m > cap.m || ns > cap.strips {
            return None;
        }
        let First {
            thick_rows,
            thin,
            thick,
            m,
            mark_and_byte,
        } = f;
        let mark = mark_and_byte as u32;
        let mark_byte = (mark_and_byte >> 32) as u8;

        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);
        let need = endpoint::arena_words(ns, m as usize);
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
        if n_usize >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        if let Some(t0) = t_s1 {
            stats.t_s1_ns = t0.elapsed().as_nanos() as u64;
        }

        // ---- S2 + S3a: RV Elimination + Dictionary & Coalesced Bands ----
        let cursors = endpoint::induced::<BAND_COAL, TIMED>(
            &mut self.edges,
            &mut self.stack,
            &mut self.bands,
            &mut self.spelling,
            &mut self.inline16,
            &mut self.lens,
            ns,
            n,
            m,
            &thin,
            &mut stats,
        );

        // ---- S4: Wiring ----
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m as usize {
            let add = m as usize - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }

        let predecessor = if !PLAIN_UNI && !wide && m >= 65_536 {
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut pred = mark;
            let mut k = 0;
            let ma = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
                if !NO_MARK || a == ma {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor::<true>(
                            edges,
                            band.dst() as usize,
                            (band.end() - k) as usize,
                            band.code() << bits,
                            mark,
                            &mut pred,
                        );
                        k = band.end();
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor::<false>(
                            edges,
                            band.dst() as usize,
                            (band.end() - k) as usize,
                            band.code() << bits,
                            mark,
                            &mut pred,
                        );
                        k = band.end();
                    }
                }
            }
            Some(pred)
        } else {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
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
            None
        };
        if let Some(t0) = t_s4 {
            stats.t_s4_ns = t0.elapsed().as_nanos() as u64;
        }

        // ---- Emit ----
        let t_emit = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = predecessor {
            emit_bidirectional(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n_usize,
                mark,
                before,
                bits,
                out,
            );
        } else {
            emit_uni(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n_usize,
                mark,
                bits,
                out,
            );
        }
        if let Some(t0) = t_emit {
            stats.t_emit_ns = t0.elapsed().as_nanos() as u64;
        }

        if TIMED {
            stats.retained = m as usize;
            stats.thick = ns;
            stats.bands = self.bands.len();
            stats.dict_words = self.lens.len() - 256;
            stats.spelling_bytes = self.spelling.len();
            stats.peak_scratch_bytes = self.scratch_bytes();
        }
        Some(stats)
    }
}

impl<const BAND_COAL: bool, const NO_MARK: bool, const PLAIN_UNI: bool> InverseBwt
    for LeanOpenArm<BAND_COAL, NO_MARK, PLAIN_UNI>
{
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = open_cap(r.records().len(), n, p) {
            if self.try_run::<false>(r.records(), n, p, cap, out).is_some() {
                return true;
            }
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }

    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }

    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }

    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }

    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}
