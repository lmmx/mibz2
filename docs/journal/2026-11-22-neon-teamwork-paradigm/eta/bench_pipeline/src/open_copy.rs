//! Safe-Rust verbatim port of `mibz2`'s `src/open.rs` (`outq` + `endpoint` + `Kernel`)
//! with per-substage timing (`pass1`, `elim`, `wire`, `emit`) and fused entry points:
//!   - `run_from_records`: verbatim `master` (`Open` over 127-capped `Runs.records`)
//!   - `run_from_maxruns`: uncapped maximal runs `(len << 8) | byte` (no 127-cap, no `maximal()`)
//!   - `run_from_tokens_direct`: per-run `MtfSwar` fused directly into `Pass::take` (safe analogue of #666)
//!   - `run_from_tokens_striped`: L1-blocked strip-mined `MtfSwar` (256 runs/batch in L1 stack) -> `Pass::take`

use crate::huff_monoid::{MtfU64, RunTokens};
use std::time::{Duration, Instant};

pub const LIMIT: u32 = 16;
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

#[derive(Default, Clone, Copy, Debug)]
pub struct OpenTimings {
    pub pass1: Duration,
    pub elim: Duration,
    pub wire: Duration,
    pub emit: Duration,
}

impl OpenTimings {
    pub fn add(&mut self, other: OpenTimings) {
        self.pass1 += other.pass1;
        self.elim += other.elim;
        self.wire += other.wire;
        self.emit += other.emit;
    }
    pub fn total(&self) -> Duration {
        self.pass1 + self.elim + self.wire + self.emit
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cap {
    pub m: u32,
    pub strips: usize,
}

impl Cap {
    pub const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };
    pub fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
    /// Exact production `OpenCells::cap` admission logic
    pub fn for_block(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if records * 64 >= n && records * 20 < n * 3 {
            // Exact `lazy_counts_admit`: 1/64 <= r/n < 3/20 (0.15)
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Counts {
    pub open_nodes: usize,
    pub steps: usize,
    pub rules: usize,
    pub n: usize,
    pub runs: usize,
    pub retained: usize,
    pub thick: usize,
    pub bands: usize,
    pub words: usize,
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

// ---- OutQ16 (verbatim from src/open/outq.rs) ----
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

// ---- Endpoint elimination (verbatim from src/open/endpoint.rs) ----
mod endpoint {
    use super::{Band, Counts, OpenKernel};

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
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;

    #[derive(Default)]
    pub(super) struct Endpoint {
        stack: Vec<u32>,
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
                self.a[self.rule(w) + 2]
            }
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
            let len = self.len(left) + self.len(right);
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
            self.top += RULE;
            id
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
        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
            let (mut left, mut taken) = (*remaining, *steps);
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
                        taken += q as usize;
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
                        taken += q as usize;
                        continue;
                    }
                }
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                taken += 1;
                if x == y {
                    self.cut(x, 0);
                    self.cut(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.cut(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    if aw == bw {
                        self.cut(y, 0);
                        self.cut(y, 1);
                    } else {
                        self.set_width(y, bw - w);
                    }
                } else {
                    self.cut(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                }
            };
            (*remaining, *steps) = (left, taken);
            done
        }
        fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
            stack.clear();
            stack.push(word);
            while let Some(id) = stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let r = self.rule(id);
                    let rule = &mut self.a[r..r + RULE];
                    if rule[3] == u32::MAX {
                        rule[3] = dest.len() as u32;
                        stack.push(rule[1]);
                        stack.push(rule[0]);
                    } else {
                        let start = rule[3] as usize;
                        dest.extend_from_within(start..start + rule[2] as usize);
                    }
                }
            }
        }
    }

    impl OpenKernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }
        fn wide(&self, ns: usize) -> bool {
            Self::nodes_most(ns) > NARROW_NODES
        }
        pub(super) fn arena_words(&self, ns: usize, m: usize) -> usize {
            let stride = if self.wide(ns) {
                Wide::STRIDE
            } else {
                Narrow::STRIDE
            };
            (ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
        }
        pub(super) fn induced(
            &mut self,
            n: u32,
            thin: &[u32; 256],
            stats: &mut Counts,
        ) -> [usize; 256] {
            let most = Self::nodes_most(self.strips.len());
            if self.wide(self.strips.len()) {
                self.induced_with::<Wide>(n, thin, stats, most)
            } else {
                self.induced_with::<Narrow>(n, thin, stats, most)
            }
        }
        fn induced_with<L: Links>(
            &mut self,
            n: u32,
            thin: &[u32; 256],
            stats: &mut Counts,
            most: usize,
        ) -> [usize; 256] {
            let OpenKernel {
                endpoint,
                strips,
                bands,
                spelling,
                offsets,
                edges,
                ..
            } = self;
            let ns = strips.len();
            let nodes_words = L::STRIDE * most;
            assert!(edges.len() >= ns + nodes_words);
            let (image_order, arena) = edges.split_at_mut(ns);
            let mut e = Elim::<L> {
                a: arena,
                base: 0,
                top: 0,
                nodes: 0,
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            let mut widest = 0;
            for s in strips.iter() {
                e.node(s.width, u32::from(s.byte) + 1);
                widest = widest.max(s.width);
            }
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0;
            for (i, s) in strips.iter().enumerate() {
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
                let s = strips[i as usize];
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
            stats.open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            e.base = L::STRIDE * e.nodes as usize;
            e.top = e.base;
            let (mut remaining, mut steps) = (n, 0);
            let spirals = widest > CERTIFIED_WIDTH;
            while !if spirals {
                e.eliminate::<true>(&mut remaining, &mut steps)
            } else {
                e.eliminate::<false>(&mut remaining, &mut steps)
            } {
                let (base, top, nodes) = (e.base, e.top, e.nodes);
                grow(edges, ns + top + RULE);
                e = Elim {
                    a: &mut edges[ns..],
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
            }
            stats.steps = steps;
            stats.rules = (e.top - e.base) / RULE;

            let mut id = L::next(e.a, 0, 1);
            x = 0;
            while id != 0 {
                let next = L::next(e.a, id, 1);
                let w = e.width(id);
                e.set_bottom(id, x);
                x += w;
                id = next;
            }
            assert_eq!(x as usize, stats.retained);
            let mut cursors = [0; 256];
            id = L::next(e.a, 0, 0);
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
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        stats.words += 1;
                        e.append(node_word, &mut endpoint.stack, spelling);
                        spelling.push(a as u8);
                        offsets.push(spelling.len() as u32);
                        254 + offsets.len() as u32
                    };
                    bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
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
}

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
        }
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

struct Pass<'a> {
    queues: outq::Queues<'a>,
    strips: &'a mut Vec<Strip>,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
        self.f.runs += 1;
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
                f.mark = f.m + (p - x);
            }
            self.queues.push(a, f.m, w, &f.thin);
            f.thin[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip {
                from: x,
                to: f.all[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
        }
        f.all[a] += w;
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

#[derive(Default)]
pub struct OpenKernel {
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

impl OpenKernel {
    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32) -> Pass<'a> {
        self.strips.clear();
        Pass {
            queues: outq::Queues::new(out, &mut self.spill, limit),
            strips: &mut self.strips,
            f: First::default(),
        }
    }

    /// Baseline: verbatim `master` Open over 127-capped `records`
    pub fn run_from_records(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
        timings: Option<&mut OpenTimings>,
    ) -> Option<Counts> {
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m);
        maximal(records, |c, pos, width| pass.take(c, pos, width, p));
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        let d_pass1 = t0.elapsed();
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish_and_emit(&f, n, &heads, out, d_pass1, timings))
    }

    /// Arm B2: uncapped maximal runs `(len << 8) | byte` (no 127-cap, no `maximal()` re-merge)
    pub fn run_from_maxruns(
        &mut self,
        runs: &[u32],
        n: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
        timings: Option<&mut OpenTimings>,
    ) -> Option<Counts> {
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m);
        let mut pos = 0u32;
        for &r in runs {
            let c = (r & 0xff) as u8;
            let width = r >> 8;
            pass.take(c, pos, width, p);
            pos += width;
        }
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        let d_pass1 = t0.elapsed();
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish_and_emit(&f, n, &heads, out, d_pass1, timings))
    }

    /// Arm B0: per-run fused `MtfU64` -> `Pass::take` directly from flat `RunTokens` (no `records`!)
    pub fn run_from_tokens_direct(
        &mut self,
        rt: &RunTokens,
        cap: Cap,
        out: &mut Vec<u8>,
        timings: Option<&mut OpenTimings>,
    ) -> Option<Counts> {
        let t0 = Instant::now();
        let n = rt.n;
        let p = rt.orig_ptr;
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m);
        let mut mtf = MtfU64::new(&rt.seq_to_unseq);
        let toks = rt.slice();
        let mut pos = 0u32;
        let mut i = 0usize;
        if let Some(&first) = toks.first() {
            if (first >> 24) == 0 {
                let width = first & 0x00ff_ffff;
                pass.take(mtf.front(), 0, width, p);
                pos = width;
                i = 1;
            }
        }
        for &tok in &toks[i..] {
            let nn = (tok >> 24) as usize;
            let width = tok & 0x00ff_ffff;
            let c = mtf.move_to_front(nn);
            pass.take(c, pos, width, p);
            pos += width;
        }
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        let d_pass1 = t0.elapsed();
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish_and_emit(&f, n, &heads, out, d_pass1, timings))
    }

    /// Arm B1: L1-blocked strip-mined fusion (`MtfU64` -> 256-run L1 stack buffer -> `Pass::take`)
    pub fn run_from_tokens_striped(
        &mut self,
        rt: &RunTokens,
        cap: Cap,
        out: &mut Vec<u8>,
        timings: Option<&mut OpenTimings>,
    ) -> Option<Counts> {
        const BATCH: usize = 256;
        let t0 = Instant::now();
        let n = rt.n;
        let p = rt.orig_ptr;
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m);
        let mut mtf = MtfU64::new(&rt.seq_to_unseq);
        let toks = rt.slice();
        let mut buf = [0u32; BATCH];
        let mut pos = 0u32;
        let mut i = 0usize;
        if let Some(&first) = toks.first() {
            if (first >> 24) == 0 {
                let width = first & 0x00ff_ffff;
                pass.take(mtf.front(), 0, width, p);
                pos = width;
                i = 1;
            }
        }
        for chunk in toks[i..].chunks(BATCH) {
            for (dst, &tok) in buf.iter_mut().zip(chunk.iter()) {
                let nn = (tok >> 24) as usize;
                let width = tok & 0x00ff_ffff;
                let c = mtf.move_to_front(nn);
                *dst = (width << 8) | (c as u32);
            }
            for &r in &buf[..chunk.len()] {
                let c = (r & 0xff) as u8;
                let width = r >> 8;
                pass.take(c, pos, width, p);
                pos += width;
            }
        }
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        let d_pass1 = t0.elapsed();
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish_and_emit(&f, n, &heads, out, d_pass1, timings))
    }

    fn finish_and_emit(
        &mut self,
        f: &First,
        n: usize,
        heads: &outq::Heads,
        out: &mut Vec<u8>,
        d_pass1: Duration,
        timings: Option<&mut OpenTimings>,
    ) -> Counts {
        let t_elim0 = Instant::now();
        let First {
            all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;
        let mut stats = Counts {
            n,
            runs,
            retained: m as usize,
            thick: self.strips.len(),
            ..Counts::default()
        };
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.strips.len();
        let need = self.arena_words(ns, m as usize);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        let image_order = &mut self.edges[..ns];
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let cursors = self.induced(n as u32, &thin, &mut stats);
        stats.bands = self.bands.len();
        let d_elim = t_elim0.elapsed();

        let t_wire0 = Instant::now();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        let mut k = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(&out[..], &self.spill, heads, a);
            for b in &self.bands[cursors[a]..end] {
                let span = (b.end - k) as usize;
                let dst = k.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    self.extra[dst..dst + span].fill(b.code);
                }
                queue.wire(&mut self.edges[dst..dst + span], pack);
                k = b.end;
            }
        }
        let d_wire = t_wire0.elapsed();

        let t_emit0 = Instant::now();
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        emit(
            &self.edges[..m as usize],
            &self.extra,
            &self.spelling,
            &self.offsets,
            &self.inline16,
            &self.lens,
            n,
            mark,
            bits,
            out,
        );
        let d_emit = t_emit0.elapsed();

        if let Some(t) = timings {
            t.add(OpenTimings {
                pass1: d_pass1,
                elim: d_elim,
                wire: d_wire,
                emit: d_emit,
            });
        }
        stats
    }
}

fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
) {
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
    let words = offsets.len().saturating_sub(1);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
    for w in offsets.windows(2) {
        let word = &spelling[w[0] as usize..w[1] as usize];
        let mut v = [0; INLINE];
        let head = word.len().min(INLINE);
        v[..head].copy_from_slice(&word[..head]);
        inline16.push(v);
        lens.push(if word.len() <= INLINE {
            word.len() as u8
        } else {
            LONG
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
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
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if j == mark || at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}
