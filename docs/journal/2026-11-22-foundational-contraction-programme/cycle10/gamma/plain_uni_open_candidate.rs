#![forbid(unsafe_code)]
//! Tube-Open (V6): Zero-RV Post-S3a Inclusion-Forest Thin-Tube Contraction
//! built directly inside v0.5.12 Open's flat-array algebra.
//!
//! Conforms to mibz2's `bench/moonshot` `crate::candidate` contract (`ee7fbf0`).

use crate::bwt::{InverseBwt, OpenCells};
use crate::runs::Runs;
use crate::Kernels;

/// The scorer reads this name.
pub type Candidate = TubeOpen;

/// Whether a block of `records` capped run records in `n` rows is under LazyCells' density line.
#[inline]
pub fn under_density(records: usize, n: usize) -> bool {
    records * 20 < n * 3
}

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TubeMode {
    /// Exact v0.5.12 Open (two-ended XOR) with zero tube work (1.00x control!)
    BaselineOpen,
    /// Pure 1-write store wire + emit_uni, zero tubes, zero extra scratch (ablation control!)
    PlainUniOpen,
    /// Plain unidirectional wire + Tube-Open V6 (Incl+ThickTail <= 16B) + unidirectional emit
    TubeUniV6,
    /// Plain unidirectional wire + Tube-Open V6 + active-only XOR conversion + two-ended XOR emit
    TubeBiXorV6,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TubeStats {
    pub m_open: usize,
    pub m_tube_steps: usize,
    pub r2_runs: usize,
    pub tube_hops: usize,
    pub tubes_applied: usize,
    pub codes_total: usize,
    pub peak_scratch_bytes: usize,
    pub ns_s1: u64,
    pub ns_s23: u64,
    pub ns_s4: u64,
    pub ns_tube: u64,
    pub ns_emit: u64,
}

// ============================================================================
// Embedded exact v0.5.12 Open::outq (OutQ16 queues in `out`)
// ============================================================================

mod outq {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
    const NONE: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;

    #[derive(Default)]
    pub struct Spill {
        next: Vec<u32>,
        ovf: Vec<[u8; BYTES]>,
        cuts: Vec<[u32; 256]>,
    }

    impl Spill {
        pub fn capacity_bytes(&self) -> usize {
            4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
        }
    }

    pub struct Queues<'a> {
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

    pub struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    impl<'a> Queues<'a> {
        pub fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
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
        pub fn over(&self) -> bool {
            self.over
        }
        pub fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }
        #[inline(always)]
        pub fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = self.tails[a];
            if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
                let at = tail as usize * BYTES + 2 * f;
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

    pub struct Reader<'a> {
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
        pub fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
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
        pub fn wire_xor(
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
        pub fn wire(&mut self, dst: &mut [u32], pack: u32) {
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
// Embedded exact v0.5.12 Open::endpoint (Rauzy-Veech + GradedHandle/Prefix15)
// ============================================================================

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

mod endpoint {
    use super::{begin_inline, long_slot, long_word, Band, Strip, INLINE, LONG};

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

    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    const ID_BITS: u32 = 22;
    const ID_MASK: u32 = (1 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;

    #[inline(always)]
    fn literal(byte: u8) -> u32 {
        (1 << ID_BITS) | (u32::from(byte) + 1)
    }

    #[derive(Default)]
    pub struct Endpoint {
        pub stack: Vec<u32>,
    }

    impl Endpoint {
        pub fn capacity_bytes(&self) -> usize {
            4 * self.stack.capacity()
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
                    return;
                }
                first = next;
                last = prev;
            }
            if first != 0 {
                self.set_bottom(first, lo);
            }
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

    pub fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    pub fn arena_words(ns: usize, m: usize) -> usize {
        let wide = nodes_most(ns) > NARROW_NODES;
        let stride = if wide { Wide::STRIDE } else { Narrow::STRIDE };
        (4 * ns + (stride + RULE) * nodes_most(ns)).max(m)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn induced(
        endpoint: &mut Endpoint,
        bands: &mut Vec<Band>,
        spelling: &mut Vec<u8>,
        edges: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
    ) -> [usize; 256] {
        let most = nodes_most(ns);
        if most > NARROW_NODES {
            induced_with::<Wide>(endpoint, bands, spelling, edges, inline16, lens, ns, n, m, thin, most)
        } else {
            induced_with::<Narrow>(endpoint, bands, spelling, edges, inline16, lens, ns, n, m, thin, most)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links>(
        endpoint: &mut Endpoint,
        bands: &mut Vec<Band>,
        spelling: &mut Vec<u8>,
        edges: &mut Vec<u32>,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        _most: usize,
    ) -> [usize; 256] {
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

        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS { header } else { tail };
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
            stack: &mut endpoint.stack,
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
        cursors
    }
}

// ============================================================================
// Tube-Open (V6) Compact R_2 Thin-Tube Record & Inclusion-Forest Contractor
// ============================================================================

#[derive(Clone, Copy, Default)]
struct R2Run {
    pos_w_b: u32, // low 20: orig BWT pos; bits 20..24: w - 1; high 8: byte
    ret_m: u32,   // retained start rank in 0..m
    dst: u32,     // dst_in_byte during S1, then absolute dst = first[b] + dst_in_byte!
}

impl R2Run {
    #[inline(always)]
    fn new(pos: u32, w: u32, b: u8, ret_m: u32, dst_in_byte: u32) -> Self {
        Self {
            pos_w_b: pos | ((w - 1) << 20) | ((b as u32) << 24),
            ret_m,
            dst: dst_in_byte,
        }
    }
    #[inline(always)]
    fn pos(self) -> u32 {
        self.pos_w_b & 0xF_FFFF
    }
    #[inline(always)]
    fn w(self) -> u32 {
        ((self.pos_w_b >> 20) & 0xF) + 1
    }
    #[inline(always)]
    fn byte(self) -> u8 {
        (self.pos_w_b >> 24) as u8
    }
}

const NO_SUCC: u32 = u32::MAX;

pub struct TubeOpen {
    shipped: OpenCells,
    pub mode: TubeMode,
    /// Minimum saved rows (`saved = hops * w`) required to materialize a tube shortcut.
    pub tau: usize,

    // Exact v0.5.12 Open scratch
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,

    // Compact Tube-Open scratch (reused across blocks!)
    r2_runs: Vec<R2Run>,
    r2_f_order: Vec<u32>,
    succ: Vec<u32>,       // packed (off_in_j << 24) | j, or NO_SUCC
    covered_in: Vec<u8>,
    vis_stamp: Vec<u32>,
    cur_stamp: u32,
    worklist: Vec<u32>,
    /// Used only in TubeBiXorV6 to hold XOR edges / active bitmap
    edges_xor: Vec<u32>,
    active_bits: Vec<u64>,

    pub last_stats: TubeStats,
}

impl Default for TubeOpen {
    fn default() -> Self {
        Self::with_config(TubeMode::PlainUniOpen, 2)
    }
}

impl TubeOpen {
    pub fn with_config(mode: TubeMode, tau: usize) -> Self {
        Self {
            shipped: OpenCells::default(),
            mode,
            tau,
            endpoint: endpoint::Endpoint::default(),
            spill: outq::Spill::default(),
            bands: Vec::new(),
            spelling: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            r2_runs: Vec::new(),
            r2_f_order: Vec::new(),
            succ: Vec::new(),
            covered_in: Vec::new(),
            vis_stamp: Vec::new(),
            cur_stamp: 0,
            worklist: Vec::new(),
            edges_xor: Vec::new(),
            active_bits: Vec::new(),
            last_stats: TubeStats::default(),
        }
    }

    /// Exact sum of all allocated scratch Vec capacities (bytes).
    pub fn allocated_scratch_bytes(&self) -> usize {
        let mut tot = 0usize;
        tot += self.endpoint.capacity_bytes();
        tot += self.spill.capacity_bytes();
        tot += self.bands.capacity() * std::mem::size_of::<Band>();
        tot += self.spelling.capacity();
        tot += self.edges.capacity() * 4;
        tot += self.extra.capacity() * 4;
        tot += self.inline16.capacity() * INLINE;
        tot += self.lens.capacity();
        tot += self.r2_runs.capacity() * std::mem::size_of::<R2Run>();
        tot += self.r2_f_order.capacity() * 4;
        tot += self.succ.capacity() * 4;
        tot += self.covered_in.capacity();
        tot += self.vis_stamp.capacity() * 4;
        tot += self.worklist.capacity() * 4;
        tot += self.edges_xor.capacity() * 4;
        tot += self.active_bits.capacity() * 8;
        tot
    }

    pub fn run_raw(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        self.run_inner::<false>(records, n, p, out)
    }

    pub fn run_profiled(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        self.run_inner::<true>(records, n, p, out)
    }

    fn run_inner<const PROFILE: bool>(
        &mut self,
        records: &[u16],
        n_usize: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        use std::time::Instant;
        if n_usize == 0 || n_usize > 900_000 || p as usize >= n_usize {
            return false;
        }
        let n = n_usize as u32;
        let t0 = if PROFILE { Some(Instant::now()) } else { None };
        let do_tubes = matches!(self.mode, TubeMode::TubeUniV6 | TubeMode::TubeBiXorV6);

        // Stage S1: Pass 1 over capped records -> queues in `out`, thick strips in `edges`, R_2 runs in `r2_runs`!
        out.resize(n_usize + INLINE, 0);
        self.r2_runs.clear();
        let mut r2_cnt = [0u32; 256];
        let mut thick_rows = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m = 0u32;
        let mut mark = 0u32;
        let mut ns = 0usize;
        let mut p_r2: u32 = u32::MAX;

        {
            let mut queues = outq::Queues::new(&mut out[..], &mut self.spill, u32::MAX);
            let mut pos = 0u32;
            let mut i = 0usize;
            let rec_len = records.len();
            while i < rec_len {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                if width == 0 {
                    return false;
                }
                i += 1;
                while i < rec_len && (records[i] >> 8) as u8 == c {
                    let wl = u32::from(records[i] & 255);
                    if wl == 0 {
                        return false;
                    }
                    width += wl;
                    i += 1;
                }
                let a = usize::from(c);
                if width <= LIMIT {
                    if p.wrapping_sub(pos) < width {
                        mark = m + (p - pos);
                        if do_tubes && width >= 2 {
                            p_r2 = self.r2_runs.len() as u32;
                        }
                    }
                    if do_tubes && width >= 2 {
                        self.r2_runs.push(R2Run::new(pos, width, c, m, thin[a] + thick_rows[a]));
                        r2_cnt[a] += 1;
                    }
                    queues.push(a, m, width, &thin);
                    thin[a] += width;
                    m += width;
                } else if p.wrapping_sub(pos) >= width {
                    let at = 3 * ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).max(2 * self.edges.len()).max(1024);
                        self.edges.resize(need, 0);
                    }
                    self.edges[at..at + 3].copy_from_slice(&[
                        pos,
                        thin[a] + thick_rows[a],
                        (width << 8) | u32::from(c),
                    ]);
                    ns += 1;
                    thick[a] += 1;
                    thick_rows[a] += width;
                } else {
                    // Thick run containing primary row p: split into (before, singleton p, after)
                    if pos < p {
                        let w1 = p - pos;
                        let at = 3 * ns;
                        if self.edges.len() < at + 3 {
                            let need = (at + 3).max(2 * self.edges.len()).max(1024);
                            self.edges.resize(need, 0);
                        }
                        self.edges[at..at + 3].copy_from_slice(&[
                            pos,
                            thin[a] + thick_rows[a],
                            (w1 << 8) | u32::from(c),
                        ]);
                        ns += 1;
                        thick[a] += 1;
                        thick_rows[a] += w1;
                    }
                    mark = m;
                    queues.push(a, m, 1, &thin);
                    thin[a] += 1;
                    m += 1;
                    let end = pos + width;
                    if p + 1 < end {
                        let w2 = end - p - 1;
                        let at = 3 * ns;
                        if self.edges.len() < at + 3 {
                            let need = (at + 3).max(2 * self.edges.len()).max(1024);
                            self.edges.resize(need, 0);
                        }
                        self.edges[at..at + 3].copy_from_slice(&[
                            p + 1,
                            thin[a] + thick_rows[a],
                            (w2 << 8) | u32::from(c),
                        ]);
                        ns += 1;
                        thick[a] += 1;
                        thick_rows[a] += w2;
                    }
                }
                pos += width;
            }
            if pos != n || queues.over() || m == 0 {
                return false;
            }
            let heads = queues.heads();
            let t_s1_end = if PROFILE { Some(Instant::now()) } else { None };

            // Stage S2 + S3a: EXACT v0.5.12 Open endpoint induction (100% UNCHANGED!)
            let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
            let first = starts(all);
            let mut write = starts(thick);
            let need = endpoint::arena_words(ns, m as usize);
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
            let cursors = endpoint::induced(
                &mut self.endpoint,
                &mut self.bands,
                &mut self.spelling,
                &mut self.edges,
                &mut self.inline16,
                &mut self.lens,
                ns,
                n,
                m,
                &thin,
            );
            let t_s23_end = if PROFILE { Some(Instant::now()) } else { None };

            // Stage S4: Wire edges!
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let max_code = (u32::MAX >> bits) as usize;
            let wide = self.lens.len() - 1 > max_code;
            if self.edges.len() < m as usize {
                self.edges.resize(m as usize, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }

            if !do_tubes || wide || self.r2_runs.len() < 4 {
                // Exact v0.5.12 Open S4 + S5!
                let mut predecessor = None;
                if self.mode != TubeMode::PlainUniOpen && !wide && m >= 65_536 {
                    let edges = &mut self.edges[..m as usize];
                    edges.fill(0);
                    let mut pred = mark;
                    let mut k = 0;
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            queue.wire_xor(
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
                    predecessor = Some(pred);
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
                }
                let t_s4_end = if PROFILE { Some(Instant::now()) } else { None };
                let steps = if let Some(before) = predecessor {
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
                    )
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
                    )
                };
                if PROFILE {
                    let t_end = Instant::now();
                    self.last_stats = TubeStats {
                        m_open: m as usize,
                        m_tube_steps: steps,
                        r2_runs: self.r2_runs.len(),
                        tube_hops: 0,
                        tubes_applied: 0,
                        codes_total: self.lens.len(),
                        peak_scratch_bytes: self.allocated_scratch_bytes(),
                        ns_s1: (t_s1_end.unwrap() - t0.unwrap()).as_nanos() as u64,
                        ns_s23: (t_s23_end.unwrap() - t_s1_end.unwrap()).as_nanos() as u64,
                        ns_s4: (t_s4_end.unwrap() - t_s23_end.unwrap()).as_nanos() as u64,
                        ns_tube: 0,
                        ns_emit: (t_end - t_s4_end.unwrap()).as_nanos() as u64,
                    };
                }
                return true;
            }

            // Tube-Open Path (`wide == false`):
            // 1. Plain unidirectional wire (`~360 us` faster than wire_xor!)
            {
                let mut k = 0;
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let bend = b.end();
                        let span = (bend - k) as usize;
                        let dst = b.dst() as usize;
                        let pack = b.code() << bits;
                        queue.wire(&mut self.edges[dst..dst + span], pack);
                        k = bend;
                    }
                }
            }
            let t_s4_end = if PROFILE { Some(Instant::now()) } else { None };

            // 2. Stage T: Zero-Sort 2-Pointer R_2 Inclusion-Forest Merge + TubePatch (V6)!
            let (tube_hops, tubes_applied) = self.apply_tube_v6(&first, &r2_cnt, p_r2, mark, bits, max_code);
            let t_tube_end = if PROFILE { Some(Instant::now()) } else { None };

            // 3. Emit (Unidirectional OR Active-Only Two-Ended XOR!)
            let steps = if self.mode == TubeMode::TubeBiXorV6 && m >= 65_536 {
                // Convert ONLY active edges along the patched graph? Wait: we don't know which edges are active
                // without walking! Converting all non-bypassed rows: we already know which R_2 rows were bypassed!
                // Even simpler: compute `edges_xor` by scattering over `0..m` skipping bypassed R_2 rows!
                let before = self.build_active_xor_edges(m as usize, mark, bits);
                emit_bidirectional(
                    &self.edges_xor[..m as usize],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n_usize,
                    mark,
                    before,
                    bits,
                    out,
                )
            } else {
                emit_uni(
                    &self.edges[..m as usize],
                    &[],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n_usize,
                    mark,
                    bits,
                    out,
                )
            };

            if PROFILE {
                let t_end = Instant::now();
                self.last_stats = TubeStats {
                    m_open: m as usize,
                    m_tube_steps: steps,
                    r2_runs: self.r2_runs.len(),
                    tube_hops,
                    tubes_applied,
                    codes_total: self.lens.len(),
                    peak_scratch_bytes: self.allocated_scratch_bytes(),
                    ns_s1: (t_s1_end.unwrap() - t0.unwrap()).as_nanos() as u64,
                    ns_s23: (t_s23_end.unwrap() - t_s1_end.unwrap()).as_nanos() as u64,
                    ns_s4: (t_s4_end.unwrap() - t_s23_end.unwrap()).as_nanos() as u64,
                    ns_tube: (t_tube_end.unwrap() - t_s4_end.unwrap()).as_nanos() as u64,
                    ns_emit: (t_end - t_tube_end.unwrap()).as_nanos() as u64,
                };
            }
        }
        true
    }

    /// Zero-Sort 2-Pointer R_2 Merge + V6 Inclusion-Forest Tube Patching directly onto `edges[0..m]`!
    fn apply_tube_v6(
        &mut self,
        first: &[u32; 256],
        r2_cnt: &[u32; 256],
        p_r2: u32,
        _mark: u32,
        bits: u32,
        max_code: usize,
    ) -> (usize, usize) {
        let num_r2 = self.r2_runs.len();
        if num_r2 == 0 {
            return (0, 0);
        }

        // Step T1: Absolute `dst` + Counting-Scatter into F-order (`r2_f_order`, ALREADY sorted by `dst`!)
        let mut r2_off = starts(*r2_cnt);
        self.r2_f_order.resize(num_r2, 0);
        for i in 0..num_r2 {
            let r = &mut self.r2_runs[i];
            let a = usize::from(r.byte());
            r.dst += first[a];
            let pos = r2_off[a] as usize;
            self.r2_f_order[pos] = i as u32;
            r2_off[a] = (pos + 1) as u32;
        }

        // Step T2: ONE Sequential 2-Pointer Merge (`r2_runs` sorted by `pos` vs `r2_f_order` sorted by `dst`)!
        self.succ.clear();
        self.succ.resize(num_r2, NO_SUCC);
        self.covered_in.clear();
        self.covered_in.resize(num_r2, 0);

        let r2 = &self.r2_runs;
        let mut j = 0usize;
        for &fi in &self.r2_f_order {
            let i = fi as usize;
            let ri = r2[i];
            let dst = ri.dst;
            let wi = ri.w();
            while j < num_r2 && r2[j].pos() + r2[j].w() <= dst {
                j += 1;
            }
            if j < num_r2 && (j as u32) != p_r2 {
                let rj = r2[j];
                let jp = rj.pos();
                let jw = rj.w();
                if dst >= jp && dst + wi <= jp + jw {
                    // Unsplit thin inclusion: R_2[i] maps inside R_2[j] at offset (dst - jp)!
                    // Guard against self-loop i == j!
                    if i != j {
                        let off = dst - jp;
                        self.succ[i] = (off << 24) | (j as u32);
                        self.covered_in[j] = self.covered_in[j].saturating_add(wi as u8);
                    }
                }
            }
        }

        // Step T3: Trace Inclusion-Forest Chains & Patch `edges`!
        // In TubeBiXorV6, compute Kahn's topological order so children always patch BEFORE parents
        // (guaranteeing active_bits and XOR edges stay 100% exact on every merging inclusion tree!).
        if self.vis_stamp.len() < num_r2 {
            self.vis_stamp.resize(num_r2, 0);
        }
        if self.mode == TubeMode::TubeBiXorV6 {
            let m_words = (self.edges.len() + 63) / 64;
            self.active_bits.clear();
            self.active_bits.resize(m_words, !0u64); // 1 = active, 0 = bypassed!
        }

        self.worklist.clear();
        if self.mode == TubeMode::TubeBiXorV6 {
            // Simple in-degree count for Kahn topological order (only on nodes in R_2)
            let mut indeg = vec![0u32; num_r2];
            for &sc in &self.succ {
                if sc != NO_SUCC {
                    indeg[(sc & 0x00FF_FFFF) as usize] += 1;
                }
            }
            let mut q: Vec<u32> = Vec::with_capacity(num_r2);
            for i in 0..num_r2 {
                if indeg[i] == 0 {
                    q.push(i as u32);
                }
            }
            let mut qi = 0;
            while qi < q.len() {
                let u = q[qi] as usize;
                qi += 1;
                if self.succ[u] != NO_SUCC && (u32::from(self.covered_in[u]) < r2[u].w() || (u as u32) == p_r2) {
                    self.worklist.push(u as u32);
                }
                let sc = self.succ[u];
                if sc != NO_SUCC {
                    let v = (sc & 0x00FF_FFFF) as usize;
                    indeg[v] -= 1;
                    if indeg[v] == 0 {
                        q.push(v as u32);
                    }
                }
            }
        } else {
            for i in 0..num_r2 {
                if self.succ[i] != NO_SUCC && (u32::from(self.covered_in[i]) < r2[i].w() || (i as u32) == p_r2) {
                    self.worklist.push(i as u32);
                }
            }
        }

        let mask = (1u32 << bits) - 1;
        let tau = self.tau;
        let mut total_hops = 0usize;
        let mut tubes_applied = 0usize;
        let track_bypass = self.mode == TubeMode::TubeBiXorV6;

        let mut wi_ptr = 0usize;
        while wi_ptr < self.worklist.len() {
            let start_i = self.worklist[wi_ptr] as usize;
            wi_ptr += 1;
            if self.succ[start_i] == NO_SUCC {
                continue;
            }
            // Check code budget headroom before tracing
            if self.lens.len() >= max_code {
                break;
            }

            // Bump stamp (rare wrap guard)
            self.cur_stamp = self.cur_stamp.wrapping_add(1);
            if self.cur_stamp == 0 {
                self.vis_stamp.fill(0);
                self.cur_stamp = 1;
            }
            let stamp = self.cur_stamp;

            let r_start = r2[start_i];
            let w_i = r_start.w() as usize;
            let r0_ret = r_start.ret_m as usize;
            let r0_has_mark = (start_i as u32) == p_r2;

            // Trace LF chain: start_i -> s1 -> s2 -> ... -> top (at most 15 hops!)
            // Collect bytes in REVERSE (forward text order: b_{h-1}, ..., b_1, b_0)!
            let mut cur = start_i;
            let mut cur_off = 0usize;
            let mut hops = 0usize;
            let mut buf = [0u8; INLINE]; // buf[INLINE - hops .. INLINE] holds [b_{h-1}, ..., b_0]!
            let mut chain_nodes = [0usize; 16]; // for marking bypassed rows in TubeBiXorV6
            let mut chain_offs = [0usize; 16];
            self.vis_stamp[cur] = stamp;

            while hops < 15 {
                let sc = self.succ[cur];
                if sc == NO_SUCC {
                    break;
                }
                let nxt = (sc & 0x00FF_FFFF) as usize;
                let off_in_nxt = (sc >> 24) as usize;
                if self.vis_stamp[nxt] == stamp {
                    // Closed thin cycle! Break cleanly before repeating `nxt`.
                    break;
                }
                self.vis_stamp[nxt] = stamp;
                // Record byte of `cur` (which is the FL word of edge `nxt -> cur`!)
                buf[INLINE - 1 - hops] = r2[cur].byte();
                chain_nodes[hops] = cur;
                chain_offs[hops] = cur_off;
                cur_off += off_in_nxt;
                cur = nxt;
                hops += 1;
            }
            total_hops += hops;
            if hops == 0 {
                continue;
            }
            // If stopped early at `cur` due to hops == 15 and `cur` still has a successor, schedule `cur`!
            if hops == 15 && self.succ[cur] != NO_SUCC {
                self.worklist.push(cur as u32);
            }

            let r_top = r2[cur].ret_m as usize + cur_off;

            // Try ThickTail / Exit Absorption on `r0_ret` (skips ALL `hops` steps, including `r0`!)
            let mut absorbed_exit = false;
            let r_top_active = !track_bypass
                || (0..w_i).all(|k| {
                    let idx = r_top + k;
                    (self.active_bits[idx >> 6] & (1u64 << (idx & 63))) != 0
                });
            let all_chain_active = !track_bypass
                || (0..hops).all(|t| {
                    let base = r2[chain_nodes[t]].ret_m as usize + chain_offs[t];
                    (0..w_i).all(|k| {
                        let idx = base + k;
                        (self.active_bits[idx >> 6] & (1u64 << (idx & 63))) != 0
                    })
                });
            if !r_top_active || !all_chain_active {
                continue;
            }
            if !r0_has_mark && hops * w_i >= tau {
                let c0 = (self.edges[r0_ret] >> bits) as usize;
                let l0 = self.lens[c0];
                if l0 != LONG && (l0 as usize) + hops <= INLINE {
                    let mut all_same = true;
                    for k in 1..w_i {
                        if (self.edges[r0_ret + k] >> bits) as usize != c0 {
                            all_same = false;
                            break;
                        }
                    }
                    if all_same {
                        let l0_u = l0 as usize;
                        let tot_len = hops + l0_u;
                        let mut entry = [0u8; INLINE];
                        entry[..hops].copy_from_slice(&buf[INLINE - hops..INLINE]);
                        entry[hops..tot_len].copy_from_slice(&self.inline16[c0][..l0_u]);
                        let code = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(tot_len as u8);
                        let pack = code << bits;
                        for k in 0..w_i {
                            let v_exit = self.edges[r0_ret + k] & mask;
                            self.edges[r_top + k] = pack | v_exit;
                        }
                        if track_bypass {
                            for t in 0..hops {
                                let base = r2[chain_nodes[t]].ret_m as usize + chain_offs[t];
                                for k in 0..w_i {
                                    let idx = base + k;
                                    self.active_bits[idx >> 6] &= !(1u64 << (idx & 63));
                                }
                            }
                        }
                        tubes_applied += 1;
                        absorbed_exit = true;
                    }
                }
            }

            // Otherwise, if hops >= 2 and (hops - 1) * w_i >= tau, jump `r_top -> r0_ret` (skips `hops - 1` steps!)
            if !absorbed_exit && hops >= 2 && (hops - 1) * w_i >= tau {
                let mut entry = [0u8; INLINE];
                entry[..hops].copy_from_slice(&buf[INLINE - hops..INLINE]);
                let code = self.lens.len() as u32;
                self.inline16.push(entry);
                self.lens.push(hops as u8);
                let pack = code << bits;
                for k in 0..w_i {
                    self.edges[r_top + k] = pack | ((r0_ret + k) as u32);
                }
                if track_bypass {
                    // Only intermediate nodes t = 1..hops are bypassed (`r0` at t=0 is landed on!)
                    for t in 1..hops {
                        let base = r2[chain_nodes[t]].ret_m as usize + chain_offs[t];
                        for k in 0..w_i {
                            let idx = base + k;
                            self.active_bits[idx >> 6] &= !(1u64 << (idx & 63));
                        }
                    }
                }
                tubes_applied += 1;
            }
        }

        (total_hops, tubes_applied)
    }

    /// Build XOR-encoded edges over active (non-bypassed) rows for two-ended emit!
    fn build_active_xor_edges(&mut self, m: usize, mark: u32, bits: u32) -> u32 {
        self.edges_xor.clear();
        self.edges_xor.resize(m, 0);
        let mask = (1u32 << bits) - 1;
        let mut before = mark;
        let edges = &self.edges[..m];
        let xor_e = &mut self.edges_xor[..m];
        let active = &self.active_bits;

        for (wi, &word) in active.iter().enumerate() {
            let mut bits_w = word;
            let base = wi << 6;
            while bits_w != 0 {
                let tz = bits_w.trailing_zeros() as usize;
                bits_w &= bits_w - 1;
                let u = base + tz;
                if u >= m {
                    break;
                }
                let eu = edges[u];
                let v = eu & mask;
                if v == mark {
                    before = u as u32;
                }
                xor_e[u] ^= eu;
                xor_e[v as usize] ^= u as u32;
            }
        }
        before
    }
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
) -> usize {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    let mask = (1 << bits) - 1;
    let mut j = mark;
    let mut steps = 0usize;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        } as usize;
        j = edge & mask;
        steps += 1;
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
    steps
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
) -> usize {
    out.resize(n + INLINE, 0);
    let mask = (1 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0, n);
    let mut steps = 0usize;
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) {
            break;
        }
        let ef = edges[front as usize];
        let cf = (ef >> bits) as usize;
        let single = front == back;
        let lf = lens[cf];
        steps += 1;
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
        steps += 1;
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
    steps
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

impl InverseBwt for TubeOpen {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        self.shipped.invert(ll, counts, orig_ptr, out)
    }
    fn wants_checkpoints(&self) -> bool {
        self.shipped.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        orig_ptr: u32,
        out: &mut Vec<u8>,
    ) {
        self.shipped
            .invert_checkpointed(ll, counts, checkpoints, orig_ptr, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        self.shipped.invert_in_place(ll, counts, orig_ptr)
    }
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(
        &mut self,
        r: &Runs,
        n: usize,
        orig_ptr: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        let recs = r.records();
        if n == 0 || n > 900_000 || !under_density(recs.len(), n) {
            return self.shipped.invert_runs(r, n, orig_ptr, out, k);
        }
        self.run_raw(recs, n, orig_ptr, out)
    }
}
