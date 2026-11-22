#![forbid(unsafe_code)]
//! Bijective R1-Open (F-Order OutQ16 Edition):
//! Exact-Match Bijective R1 Contraction (`dst(u) == pos(v) && len(u) == len(v)`)
//! fused natively into v0.5.12 Open's F-order OutQ16 `wire_xor` + `emit_bidirectional`!
//!
//! Why F-order OutQ16 matters (proven by Cycle 7 silicon autopsy):
//!   - L-order S4 caused 256-stream random reads into `bands` and `edges` (6.48 ms vs Open's 1.52 ms).
//!   - 95-stream concurrent `f_cur[a]` caused L2/L3 prefetcher thrashing (7.60-13.06 ms).
//!   - F-order OutQ16 + 2-stream sequential merge on `R_2` (22k runs/blk, 352 KiB) keeps
//!     S4 100% sequential on `bands` and `edges[dst..]`, skips 23% of wire_xor writes via
//!     `queue.skip(w)`, and preserves native two-ended XOR emission (`emit_bidirectional`)
//!     with 0/6 `wide` overflow and zero extra `endpoint.rs` nodes/steps/bands!
//!
//! Conforms to mibz2's `bench/moonshot` `crate::candidate` contract (`ee7fbf0`).

use crate::bwt::{InverseBwt, OpenCells};
use crate::runs::Runs;
use crate::Kernels;

/// The scorer reads this name.
pub type Candidate = BijectiveR1Open;

#[inline]
pub fn under_density(records: usize, n: usize) -> bool {
    records * 20 < n * 3
}

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum R1Mode {
    /// F-order OutQ16 Bijective R1 on 2 <= w <= 16 (22k runs/blk, 2-stream merge, native wire_xor)
    FOrderW2,
    /// F-order OutQ16 Bijective R1 on 1 <= w <= 16 with w>=2 (h>=1) or w==1 (h>=3)
    FOrderW2AndW1H3,
    /// Exact embedded v0.5.12 Open control (zero R1 work, 1.00x reference)
    BaselineOpen,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct R1Stats {
    pub m_open: usize,
    pub m_walk_steps: usize,
    pub r_tracked: usize,
    pub r1_chains: usize,
    pub r1_skipped_rows: usize,
    pub codes_total: usize,
    pub peak_scratch_bytes: usize,
    pub ns_s1: u64,
    pub ns_s23: u64,
    pub ns_r1: u64,
    pub ns_s4: u64,
    pub ns_emit: u64,
}

// ============================================================================
// Embedded exact v0.5.12 Open::outq + O(1) `skip(span)` for R1 dead rows!
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

        /// Advance the queue cursor by `span` ranks WITHOUT touching `edges`!
        #[inline(always)]
        pub fn skip(&mut self, mut span: usize) {
            if self.at + span <= CHUNK && self.read + span as u32 <= self.cut {
                self.at += span;
                self.read += span as u32;
                return;
            }
            while span > 0 {
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
                let take = span
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                self.at += take;
                self.read += take as u32;
                span -= take;
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
// Embedded exact v0.5.12 Open::endpoint
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
        if nodes_most(ns) > NARROW_NODES {
            induced_with::<Wide>(endpoint, bands, spelling, edges, inline16, lens, ns, n, m, thin)
        } else {
            induced_with::<Narrow>(endpoint, bands, spelling, edges, inline16, lens, ns, n, m, thin)
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
// Compact Thin-Run Record for 2-Stream Sequential R1 Merge + F-Order S4
// ============================================================================

/// 16-byte aligned tracked thin run (22k/blk for FOrderW2 = 352 KiB L2-resident!)
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct ThinRec {
    pos_w_b: u32,    // low 20: orig BWT pos; bits 20..24: w-1; high 8: byte
    ret_flags: u32,  // low 20: ret_m; bit 31: HAS_PREV
    dst: u32,        // dst_in_byte during S1 -> absolute dst = first[a] + dst_in_byte
    thin_z: u32,     // F-rank within byte `a`'s thin rows (0..thin[a])
}

const HAS_PREV_BIT: u32 = 1 << 31;

impl ThinRec {
    #[inline(always)]
    fn new(pos: u32, w: u32, byte: u8, ret_m: u32, dst_in_byte: u32, thin_z: u32) -> Self {
        Self {
            pos_w_b: pos | ((w - 1) << 20) | ((byte as u32) << 24),
            ret_flags: ret_m,
            dst: dst_in_byte,
            thin_z,
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
    #[inline(always)]
    fn ret_m(self) -> u32 {
        self.ret_flags & 0xF_FFFF
    }
    #[inline(always)]
    fn has_prev(self) -> bool {
        (self.ret_flags & HAS_PREV_BIT) != 0
    }
    #[inline(always)]
    fn set_has_prev(&mut self) {
        self.ret_flags |= HAS_PREV_BIT;
    }
}

const TAG_NONE: u32 = u32::MAX;
const TAG_SKIP: u32 = u32::MAX - 1;
const TAG_TOP_BIT: u32 = 1 << 31;

#[derive(Clone, Copy, Default)]
struct R1Chunk {
    ret_u0: u32,
    h: u8,
    bytes: [u8; 15],
}

/// Compact 12-byte sequential F-order S4 event (eliminates ALL random truns/r1_tag lookups in S4!)
#[derive(Clone, Copy, Default)]
struct R1Event {
    z_global: u32, // global thin F-rank where this event starts
    tag: u32,      // TAG_SKIP or TAG_TOP_BIT | chunk_id
    ret_uh_w: u32, // low 20: ret_m(u_h); high 8: w
}

pub struct BijectiveR1Open {
    shipped: OpenCells,
    pub mode: R1Mode,

    // Exact v0.5.12 Open scratch
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,

    // Compact F-order R1 scratch (for FOrderW2: ~500 KiB total!)
    truns: Vec<ThinRec>,
    f_order: Vec<u32>,
    r1_tag: Vec<u32>,
    chunks: Vec<R1Chunk>,
    r1_events: Vec<R1Event>,

    pub last_stats: R1Stats,
}

impl Default for BijectiveR1Open {
    fn default() -> Self {
        Self::with_mode(R1Mode::FOrderW2)
    }
}

impl BijectiveR1Open {
    pub fn with_mode(mode: R1Mode) -> Self {
        Self {
            shipped: OpenCells::default(),
            mode,
            endpoint: endpoint::Endpoint::default(),
            spill: outq::Spill::default(),
            bands: Vec::new(),
            spelling: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            truns: Vec::new(),
            f_order: Vec::new(),
            r1_tag: Vec::new(),
            chunks: Vec::new(),
            r1_events: Vec::new(),
            last_stats: R1Stats::default(),
        }
    }

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
        tot += self.truns.capacity() * std::mem::size_of::<ThinRec>();
        tot += self.f_order.capacity() * 4;
        tot += self.r1_tag.capacity() * 4;
        tot += self.chunks.capacity() * std::mem::size_of::<R1Chunk>();
        tot += self.r1_events.capacity() * std::mem::size_of::<R1Event>();
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
        let min_w: u32 = match self.mode {
            R1Mode::BaselineOpen => 99,
            R1Mode::FOrderW2 => 2,
            R1Mode::FOrderW2AndW1H3 => 1,
        };
        let min_h_w1: usize = if self.mode == R1Mode::FOrderW2AndW1H3 { 3 } else { 999 };

        // ====================================================================
        // Stage S1: Exact Open OutQ16 + record tracked thin runs (`w >= min_w`)
        // ====================================================================
        out.resize(n_usize + INLINE, 0);
        self.truns.clear();
        let mut t_cnt = [0u32; 256];
        let mut thick_rows = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m = 0u32;
        let mut mark = 0u32;
        let mut ns = 0usize;
        let mut p_trun: u32 = u32::MAX;

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
                        if width >= min_w {
                            p_trun = self.truns.len() as u32;
                        }
                    }
                    if width >= min_w {
                        self.truns.push(ThinRec::new(
                            pos,
                            width,
                            c,
                            m,
                            thin[a] + thick_rows[a],
                            thin[a],
                        ));
                        t_cnt[a] += 1;
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
                    if min_w <= 1 {
                        p_trun = self.truns.len() as u32;
                        self.truns.push(ThinRec::new(
                            p,
                            1,
                            c,
                            m,
                            thin[a] + thick_rows[a],
                            thin[a],
                        ));
                        t_cnt[a] += 1;
                    }
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

            // ================================================================
            // Stage S2 + S3a: EXACT v0.5.12 Open endpoint.rs (100% UNCHANGED!)
            // ================================================================
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
            for idx in 0..ns {
                let sw = &mut strips_raw[3 * idx..3 * idx + 3];
                let byte = (sw[2] & 0xFF) as u8;
                let a = usize::from(byte);
                sw[1] += first[a];
                image_order[write[a] as usize] = idx as u32;
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

            // ================================================================
            // Stage R1: 2-Stream Sequential Merge on `truns` + Path Slicing!
            // ================================================================
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let max_code = (u32::MAX >> bits) as usize;
            let base_wide = self.lens.len() - 1 > max_code;
            let num_t = self.truns.len();

            self.chunks.clear();
            let mut r1_skipped_rows = 0usize;
            let f_starts = starts(t_cnt);
            let do_r1 = self.mode != R1Mode::BaselineOpen && !base_wide && num_t >= 4;

            if do_r1 {
                // Counting-scatter `0..num_t` into `f_order` (sorted by `(byte, dst)`!)
                let mut f_off = f_starts;
                self.f_order.resize(num_t, 0);
                for idx in 0..num_t {
                    let tr = &mut self.truns[idx];
                    let a = usize::from(tr.byte());
                    tr.dst += first[a];
                    let p_out = f_off[a] as usize;
                    self.f_order[p_out] = idx as u32;
                    f_off[a] = (p_out + 1) as u32;
                }

                // 2-Stream Sequential Merge: `truns` (sorted by `pos`) vs `f_order` (sorted by `dst`)!
                self.r1_tag.clear();
                self.r1_tag.resize(num_t, TAG_NONE);
                let mut j = 0usize;
                for fi_idx in 0..num_t {
                    let u = self.f_order[fi_idx] as usize;
                    let dst = self.truns[u].dst;
                    let w = self.truns[u].w();
                    while j < num_t && self.truns[j].pos() + self.truns[j].w() <= dst {
                        j += 1;
                    }
                    if j < num_t
                        && self.truns[j].pos() == dst
                        && self.truns[j].w() == w
                        && j != u
                        && (j as u32) != p_trun
                    {
                        self.r1_tag[u] = j as u32;
                        self.truns[j].set_has_prev();
                    }
                }

                // Extract disjoint chunks (h <= 15) from chain heads (`!has_prev`)!
                let code_budget = max_code.saturating_sub(self.lens.len() + 32);
                let mut budget_used = 0usize;
                let max_pass = if min_w == 1 { 2 } else { 1 };
                for pass in 0..max_pass {
                    for u in 0..num_t {
                        let tu = self.truns[u];
                        if tu.has_prev() {
                            continue;
                        }
                        let first_nx = self.r1_tag[u];
                        if first_nx == TAG_NONE || first_nx == TAG_SKIP || (first_nx & TAG_TOP_BIT) != 0 {
                            continue;
                        }
                        let w = tu.w();
                        if pass == 0 && w == 1 {
                            continue;
                        }
                        if pass == 1 && w != 1 {
                            continue;
                        }
                        let req_h = if w == 1 { min_h_w1 } else { 1 };

                        let mut head = u;
                        while head < num_t {
                            let nx0 = self.r1_tag[head];
                            if nx0 == TAG_NONE || nx0 == TAG_SKIP || (nx0 & TAG_TOP_BIT) != 0 {
                                break;
                            }
                            let cost_codes = if w == 1 { 1 } else { 2 };
                            if budget_used + cost_codes > code_budget {
                                let mut clr = head;
                                while clr < num_t {
                                    let nx = self.r1_tag[clr];
                                    self.r1_tag[clr] = TAG_NONE;
                                    if nx == TAG_NONE || nx == TAG_SKIP || (nx & TAG_TOP_BIT) != 0 {
                                        break;
                                    }
                                    clr = nx as usize;
                                }
                                break;
                            }
                            let mut nodes = [0usize; 16];
                            nodes[0] = head;
                            let mut h = 0usize;
                            let mut cur = head;
                            while h < 15 {
                                let nx = self.r1_tag[cur];
                                if nx == TAG_NONE || nx == TAG_SKIP || (nx & TAG_TOP_BIT) != 0 {
                                    break;
                                }
                                cur = nx as usize;
                                h += 1;
                                nodes[h] = cur;
                            }
                            let after_top = self.r1_tag[cur];
                            if h < req_h {
                                for &nd in &nodes[..=h] {
                                    self.r1_tag[nd] = TAG_NONE;
                                }
                                if after_top == TAG_NONE || after_top == TAG_SKIP || (after_top & TAG_TOP_BIT) != 0 {
                                    break;
                                }
                                head = after_top as usize;
                                continue;
                            }
                            let mut bytes = [0u8; 15];
                            for t in 0..h {
                                bytes[h - 1 - t] = self.truns[nodes[t]].byte();
                            }
                            let cid = self.chunks.len() as u32;
                            self.chunks.push(R1Chunk {
                                ret_u0: self.truns[head].ret_m(),
                                h: h as u8,
                                bytes,
                            });
                            for &nd in &nodes[..h] {
                                self.r1_tag[nd] = TAG_SKIP;
                            }
                            self.r1_tag[nodes[h]] = TAG_TOP_BIT | cid;
                            r1_skipped_rows += h * (w as usize);
                            budget_used += cost_codes;
                            if after_top == TAG_NONE || after_top == TAG_SKIP || (after_top & TAG_TOP_BIT) != 0 {
                                break;
                            }
                            head = after_top as usize;
                        }
                    }
                }
                // Compact ONLY active SKIP/TOP events into contiguous F-order `r1_events`!
                self.r1_events.clear();
                let mut thin_base = [0u32; 256];
                let mut tacc = 0u32;
                for a in 0..256 {
                    thin_base[a] = tacc;
                    tacc += thin[a];
                }
                for &fi_u in &self.f_order {
                    let u = fi_u as usize;
                    let t = self.r1_tag[u];
                    if t != TAG_NONE && (t == TAG_SKIP || (t & TAG_TOP_BIT) != 0) {
                        let tu = self.truns[u];
                        let a = usize::from(tu.byte());
                        self.r1_events.push(R1Event {
                            z_global: thin_base[a] + tu.thin_z,
                            tag: t,
                            ret_uh_w: tu.ret_m() | (tu.w() << 24),
                        });
                    }
                }
            }
            let t_r1_end = if PROFILE { Some(Instant::now()) } else { None };

            // ================================================================
            // Stage S4: Native F-Order OutQ16 `wire_xor` (100% Sequential!)
            // ================================================================
            if self.edges.len() < m as usize {
                self.edges.resize(m as usize, 0);
            }
            self.extra.clear();
            if base_wide {
                self.extra.resize(m as usize, 0);
            }
            let use_bi_xor = !base_wide && m >= 65_536;
            let mut predecessor = mark;

            if !do_r1 || self.chunks.is_empty() {
                if use_bi_xor {
                    let edges = &mut self.edges[..m as usize];
                    edges.fill(0);
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
                                &mut predecessor,
                            );
                            k = band.end();
                        }
                    }
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
                            let pack = if base_wide { 0 } else { bcode << bits };
                            if base_wide {
                                self.extra[dst..dst + span].fill(bcode);
                            }
                            queue.wire(&mut self.edges[dst..dst + span], pack);
                            k = bend;
                        }
                    }
                }
            } else if use_bi_xor {
                // F-Order OutQ16 `wire_xor` with contiguous sequential `r1_events` stream!
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut k_global = 0u32;
                let events = &self.r1_events;
                let ev_len = events.len();
                let mut ei = 0usize;

                for a in 0..256 {
                    let b_lo = cursors[a];
                    let b_hi = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);

                    let mut cur_ev_rem = 0u32;
                    let mut cur_ev_tag = TAG_NONE;
                    let mut cur_ev_off = 0u32;
                    let mut cur_ev_ret_uh = 0u32;
                    let mut cur_own_code = u32::MAX;

                    for &band in &self.bands[b_lo..b_hi] {
                        let bend = band.end();
                        let bcode = band.code() as usize;
                        let mut bdst = band.dst();
                        let mut k = k_global;

                        // Fast path: NO R1 event overlaps this entire band!
                        if cur_ev_rem == 0 && (ei >= ev_len || events[ei].z_global >= bend) {
                            queue.wire_xor(
                                edges,
                                bdst as usize,
                                (bend - k) as usize,
                                (bcode as u32) << bits,
                                mark,
                                &mut predecessor,
                            );
                            k_global = bend;
                            continue;
                        }

                        while k < bend {
                            if cur_ev_rem == 0 {
                                let ev_z = if ei < ev_len { events[ei].z_global } else { u32::MAX };
                                if ev_z > k {
                                    let stop = bend.min(ev_z);
                                    let span = (stop - k) as usize;
                                    queue.wire_xor(
                                        edges,
                                        bdst as usize,
                                        span,
                                        (bcode as u32) << bits,
                                        mark,
                                        &mut predecessor,
                                    );
                                    bdst += span as u32;
                                    k = stop;
                                    continue;
                                }
                                let ev = events[ei];
                                ei += 1;
                                cur_ev_tag = ev.tag;
                                cur_ev_rem = ev.ret_uh_w >> 24;
                                cur_ev_ret_uh = ev.ret_uh_w & 0xF_FFFF;
                                cur_ev_off = 0;
                                cur_own_code = u32::MAX;
                            }

                            let take = (bend - k).min(cur_ev_rem);
                            queue.skip(take as usize);
                            if cur_ev_tag != TAG_SKIP {
                                let ch = self.chunks[(cur_ev_tag & !TAG_TOP_BIT) as usize];
                                let ch_h = ch.h as usize;
                                let blen = self.lens[bcode];
                                let v_u0 = ch.ret_u0 + cur_ev_off;
                                if blen != LONG
                                    && (blen as usize) + ch_h <= INLINE
                                    && self.lens.len() <= max_code
                                {
                                    let bl = blen as usize;
                                    let mut entry = self.inline16[bcode];
                                    entry[bl..bl + ch_h].copy_from_slice(&ch.bytes[..ch_h]);
                                    let code = self.lens.len() as u32;
                                    self.inline16.push(entry);
                                    self.lens.push((bl + ch_h) as u8);
                                    let pack = code << bits;
                                    for off in 0..take {
                                        let uu = bdst + off;
                                        let vv = v_u0 + off;
                                        if vv == mark {
                                            predecessor = uu;
                                        }
                                        edges[uu as usize] ^= pack | vv;
                                        edges[vv as usize] ^= uu;
                                    }
                                } else {
                                    if cur_own_code == u32::MAX {
                                        let mut entry = [0u8; INLINE];
                                        entry[..ch_h].copy_from_slice(&ch.bytes[..ch_h]);
                                        cur_own_code = self.lens.len() as u32;
                                        self.inline16.push(entry);
                                        self.lens.push(ch.h);
                                    }
                                    let v_uh = cur_ev_ret_uh + cur_ev_off;
                                    let pack_b = (bcode as u32) << bits;
                                    let pack_c = cur_own_code << bits;
                                    for off in 0..take {
                                        let uu = bdst + off;
                                        let mid = v_uh + off;
                                        let vv = v_u0 + off;
                                        if mid == mark {
                                            predecessor = uu;
                                        }
                                        edges[uu as usize] ^= pack_b | mid;
                                        edges[mid as usize] ^= uu;
                                        if vv == mark {
                                            predecessor = mid;
                                        }
                                        edges[mid as usize] ^= pack_c | vv;
                                        edges[vv as usize] ^= mid;
                                    }
                                }
                            }
                            bdst += take;
                            k += take;
                            cur_ev_rem -= take;
                            cur_ev_off += take;
                        }
                        k_global = bend;
                    }
                }
            } else {
                // Small block (< 65,536 retained rows): F-order unidirectional OutQ16 with `r1_events`!
                let edges = &mut self.edges[..m as usize];
                let mut k_global = 0u32;
                let events = &self.r1_events;
                let ev_len = events.len();
                let mut ei = 0usize;
                for a in 0..256 {
                    let b_lo = cursors[a];
                    let b_hi = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
                    let mut cur_ev_rem = 0u32;
                    let mut cur_ev_tag = TAG_NONE;
                    let mut cur_ev_off = 0u32;
                    let mut cur_ev_ret_uh = 0u32;
                    let mut cur_own_code = u32::MAX;
                    for &band in &self.bands[b_lo..b_hi] {
                        let bend = band.end();
                        let bcode = band.code() as usize;
                        let mut bdst = band.dst();
                        let mut k = k_global;
                        if cur_ev_rem == 0 && (ei >= ev_len || events[ei].z_global >= bend) {
                            queue.wire(
                                &mut edges[bdst as usize..bdst as usize + (bend - k) as usize],
                                (bcode as u32) << bits,
                            );
                            k_global = bend;
                            continue;
                        }
                        while k < bend {
                            if cur_ev_rem == 0 {
                                let ev_z = if ei < ev_len { events[ei].z_global } else { u32::MAX };
                                if ev_z > k {
                                    let stop = bend.min(ev_z);
                                    let span = (stop - k) as usize;
                                    queue.wire(
                                        &mut edges[bdst as usize..bdst as usize + span],
                                        (bcode as u32) << bits,
                                    );
                                    bdst += span as u32;
                                    k = stop;
                                    continue;
                                }
                                let ev = events[ei];
                                ei += 1;
                                cur_ev_tag = ev.tag;
                                cur_ev_rem = ev.ret_uh_w >> 24;
                                cur_ev_ret_uh = ev.ret_uh_w & 0xF_FFFF;
                                cur_ev_off = 0;
                                cur_own_code = u32::MAX;
                            }
                            let take = (bend - k).min(cur_ev_rem);
                            queue.skip(take as usize);
                            if cur_ev_tag != TAG_SKIP {
                                let ch = self.chunks[(cur_ev_tag & !TAG_TOP_BIT) as usize];
                                let ch_h = ch.h as usize;
                                let blen = self.lens[bcode];
                                let v_u0 = ch.ret_u0 + cur_ev_off;
                                if blen != LONG
                                    && (blen as usize) + ch_h <= INLINE
                                    && self.lens.len() <= max_code
                                {
                                    let bl = blen as usize;
                                    let mut entry = self.inline16[bcode];
                                    entry[bl..bl + ch_h].copy_from_slice(&ch.bytes[..ch_h]);
                                    let code = self.lens.len() as u32;
                                    self.inline16.push(entry);
                                    self.lens.push((bl + ch_h) as u8);
                                    let pack = code << bits;
                                    for off in 0..take {
                                        edges[(bdst + off) as usize] = pack | (v_u0 + off);
                                    }
                                } else {
                                    if cur_own_code == u32::MAX {
                                        let mut entry = [0u8; INLINE];
                                        entry[..ch_h].copy_from_slice(&ch.bytes[..ch_h]);
                                        cur_own_code = self.lens.len() as u32;
                                        self.inline16.push(entry);
                                        self.lens.push(ch.h);
                                    }
                                    let v_uh = cur_ev_ret_uh + cur_ev_off;
                                    let pack_b = (bcode as u32) << bits;
                                    let pack_c = cur_own_code << bits;
                                    for off in 0..take {
                                        let mid = v_uh + off;
                                        edges[(bdst + off) as usize] = pack_b | mid;
                                        edges[mid as usize] = pack_c | (v_u0 + off);
                                    }
                                }
                            }
                            bdst += take;
                            k += take;
                            cur_ev_rem -= take;
                            cur_ev_off += take;
                        }
                        k_global = bend;
                    }
                }
            }
            let t_s4_end = if PROFILE { Some(Instant::now()) } else { None };

            // ================================================================
            // Stage S5: Native Two-Ended XOR Emit (`emit_bidirectional`)!
            // ================================================================
            let walk_steps = if use_bi_xor {
                emit_bidirectional(
                    &self.edges[..m as usize],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n_usize,
                    mark,
                    predecessor,
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
                self.last_stats = R1Stats {
                    m_open: m as usize,
                    m_walk_steps: walk_steps,
                    r_tracked: num_t,
                    r1_chains: self.chunks.len(),
                    r1_skipped_rows,
                    codes_total: self.lens.len(),
                    peak_scratch_bytes: self.allocated_scratch_bytes(),
                    ns_s1: (t_s1_end.unwrap() - t0.unwrap()).as_nanos() as u64,
                    ns_s23: (t_s23_end.unwrap() - t_s1_end.unwrap()).as_nanos() as u64,
                    ns_r1: (t_r1_end.unwrap() - t_s23_end.unwrap()).as_nanos() as u64,
                    ns_s4: (t_s4_end.unwrap() - t_r1_end.unwrap()).as_nanos() as u64,
                    ns_emit: (t_end - t_s4_end.unwrap()).as_nanos() as u64,
                };
            }
        }
        true
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

impl InverseBwt for BijectiveR1Open {
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
