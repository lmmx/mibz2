//! Standalone safe-Rust port of production OpenCells (v0.5.11) with compile-time mode selector:
//!   MODE 0: `AA`         — exact byte-faithful copy of production OpenCells (A/A control)
//!   MODE 1: `LocalFused` — exact OpenCells with inline16 fused into band loop (no cross-block state)
//!   MODE 2: `XbSlp`      — exact OpenCells + Cross-Block Structural SLP Hash-Consing (HC_CAP slots)
//!
//! Zero `unsafe`. Exact for all columns (BWT or non-BWT, periodic, multi-cycle, all marks).
#![allow(dead_code)]

mod outq;

use mibz2::bwt::{InverseBwt, LazyCells};
use mibz2::runs::Runs;
use mibz2::Kernels;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;
const LAZY_DENSITY: (usize, usize) = (3, 20);

fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
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

#[derive(Clone, Copy, PartialEq)]
pub struct Cap {
    m: u32,
    strips: usize,
}

impl Cap {
    pub const NONE: Cap = Cap {
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

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
}

impl Default for First {
    fn default() -> Self {
        Self {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
        }
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

struct Pass<'a> {
    queues: outq::Queues<'a>,
    strips: &'a mut Vec<Strip>,
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

/// Persistent per-worker cross-block structural SLP hash-consing table.
/// Rule IDs:
///   0                = empty
///   1..=256          = single byte (id - 1)
///   257..LOCAL_BASE  = persistent cached canonical rules (index = id - 257)
///   >= LOCAL_BASE    = ephemeral block-local rules in Open's `edges` arena
struct SlpCache<const CAP: usize> {
    keys: Vec<u64>,
    vals: Vec<u32>,
    rules: Vec<[u32; 4]>, // left, right, len, first_in_pool (u32::MAX if unexpanded)
    pool: Vec<u8>,
}

impl<const CAP: usize> SlpCache<CAP> {
    const MAX_RULES: usize = (CAP * 3) / 4;
    const LOCAL_BASE: u32 = 257 + Self::MAX_RULES as u32;
    const SHIFT: u32 = if CAP == 0 { 0 } else { 64 - CAP.trailing_zeros() };
    const MASK: usize = CAP.saturating_sub(1);

    fn new() -> Self {
        if CAP == 0 {
            Self {
                keys: Vec::new(),
                vals: Vec::new(),
                rules: Vec::new(),
                pool: Vec::new(),
            }
        } else {
            Self {
                keys: vec![0u64; CAP],
                vals: vec![0u32; CAP],
                rules: Vec::with_capacity(Self::MAX_RULES),
                pool: Vec::with_capacity(256 * 1024),
            }
        }
    }

    fn maybe_reset_before_block(&mut self) {
        // Reset only between blocks when nearly full (> 70% full or pool > 512 KiB)
        if CAP > 0 && (self.rules.len() >= (CAP * 7) / 10 || self.pool.len() >= 512 * 1024) {
            self.keys.fill(0);
            self.rules.clear();
            self.pool.clear();
        }
    }

    #[inline(always)]
    fn try_join(&mut self, l: u32, r: u32) -> Option<u32> {
        if l >= Self::LOCAL_BASE || r >= Self::LOCAL_BASE {
            return None;
        }
        let key = ((l as u64) << 32) | (r as u64);
        let mut h = (key.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> Self::SHIFT) as usize & Self::MASK;
        loop {
            let k = self.keys[h];
            if k == key {
                return Some(self.vals[h]);
            }
            if k == 0 {
                if self.rules.len() >= Self::MAX_RULES {
                    return None;
                }
                let cid = self.rules.len() as u32 + 257;
                let ll = if l <= 256 {
                    1
                } else {
                    self.rules[(l - 257) as usize][2]
                };
                let rl = if r <= 256 {
                    1
                } else {
                    self.rules[(r - 257) as usize][2]
                };
                self.rules.push([l, r, ll + rl, u32::MAX]);
                self.keys[h] = key;
                self.vals[h] = cid;
                return Some(cid);
            }
            h = (h + 1) & Self::MASK;
        }
    }

    #[inline(always)]
    fn len_cached(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.rules[(w - 257) as usize][2]
        }
    }

    #[inline(always)]
    fn ensure_expanded(&mut self, cid: u32, stack: &mut Vec<u32>) -> (usize, usize) {
        let ri = (cid - 257) as usize;
        let first = self.rules[ri][3];
        let len = self.rules[ri][2] as usize;
        if first != u32::MAX {
            return (first as usize, len);
        }
        let start = self.pool.len();
        stack.clear();
        stack.push(cid);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                self.pool.push((id - 1) as u8);
            } else {
                let idx = (id - 257) as usize;
                if self.rules[idx][3] == u32::MAX {
                    self.rules[idx][3] = self.pool.len() as u32;
                    stack.push(self.rules[idx][1]);
                    stack.push(self.rules[idx][0]);
                } else {
                    let st = self.rules[idx][3] as usize;
                    let ln = self.rules[idx][2] as usize;
                    self.pool.extend_from_within(st..st + ln);
                }
            }
        }
        (start, len)
    }
}

struct Elim<'a, L: Links, const CAP: usize> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    cache: &'a mut SlpCache<CAP>,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const CAP: usize> Elim<'_, L, CAP> {
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
    fn local_rule(&self, w: u32) -> usize {
        let base_id = if CAP == 0 {
            257
        } else {
            SlpCache::<CAP>::LOCAL_BASE
        };
        self.base + (w - base_id) as usize * RULE
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else if CAP > 0 && w < SlpCache::<CAP>::LOCAL_BASE {
            self.cache.len_cached(w)
        } else {
            self.a[self.local_rule(w) + 2]
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
        if CAP > 0 {
            if let Some(cid) = self.cache.try_join(left, right) {
                return cid;
            }
        }
        let base_id = if CAP == 0 {
            257
        } else {
            SlpCache::<CAP>::LOCAL_BASE
        };
        let id = ((self.top - self.base) / RULE) as u32 + base_id;
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
        *remaining = left;
        done
    }

    fn append_local(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>, aux_stack: &mut Vec<u32>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else if CAP > 0 && id < SlpCache::<CAP>::LOCAL_BASE {
                let (st, ln) = self.cache.ensure_expanded(id, aux_stack);
                dest.extend_from_slice(&self.cache.pool[st..st + ln]);
            } else {
                let r = self.local_rule(id);
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

/// MODE:
///   0 = AA (exact production OpenCells)
///   1 = LocalFused (no cross-block cache, fused inline16 during band loop)
///   2 = XbSlp (cross-block hash-consing with CAP slots + fused inline16)
pub struct XbOpenKernel<const MODE: u8, const CAP: usize> {
    stack: Vec<u32>,
    aux_stack: Vec<u32>,
    spill: outq::Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    cache: SlpCache<CAP>,
}

impl<const MODE: u8, const CAP: usize> Default for XbOpenKernel<MODE, CAP> {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            aux_stack: Vec::new(),
            spill: outq::Spill::default(),
            strips: Vec::new(),
            bands: Vec::new(),
            spelling: Vec::new(),
            offsets: Vec::new(),
            edges: Vec::new(),
            extra: Vec::new(),
            inline16: Vec::new(),
            lens: Vec::new(),
            cache: SlpCache::new(),
        }
    }
}

impl<const MODE: u8, const CAP: usize> XbOpenKernel<MODE, CAP> {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }
    fn wide(ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES
    }
    fn arena_words(ns: usize, m: usize) -> usize {
        let stride = if Self::wide(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    fn init_inline_literals(&mut self) {
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
    }

    fn induced_with<L: Links>(&mut self, n: u32, thin: &[u32; 256], m: usize) -> ([usize; 256], usize) {
        let ns = self.strips.len();
        let most = Self::nodes_most(ns);
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= ns + nodes_words);
        let (image_order, arena) = self.edges.split_at_mut(ns);
        if MODE == 2 {
            self.cache.maybe_reset_before_block();
        }
        let mut e = Elim::<L, CAP> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            cache: &mut self.cache,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0;
        for s in self.strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for (i, s) in self.strips.iter().enumerate() {
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
            let s = self.strips[i as usize];
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
            grow(&mut self.edges, ns + top + RULE);
            e = Elim {
                a: &mut self.edges[ns..],
                base,
                top,
                nodes,
                cache: &mut self.cache,
                _links: std::marker::PhantomData,
            };
        }

        let mut id = L::next(e.a, 0, 1);
        x = 0;
        while id != 0 {
            let next = L::next(e.a, id, 1);
            let w = e.width(id);
            e.set_bottom(id, x);
            x += w;
            id = next;
        }
        assert_eq!(x as usize, m);

        let mut cursors = [0; 256];
        let mut words = 0usize;
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;

        if MODE == 0 {
            // Exact AA path (identical to src/open/endpoint.rs)
            for a in 0..256 {
                cursors[a] = self.bands.len();
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
                        words += 1;
                        e.append_local(node_word, &mut self.stack, &mut self.spelling, &mut self.aux_stack);
                        self.spelling.push(a as u8);
                        self.offsets.push(self.spelling.len() as u32);
                        254 + self.offsets.len() as u32
                    };
                    self.bands.push(Band {
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
        } else {
            // MODE 1 (LocalFused) or MODE 2 (XbSlp): build inline16 & lens directly during band loop!
            for a in 0..256 {
                cursors[a] = self.bands.len();
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
                        words += 1;
                        let code_id = 255 + words as u32;
                        if MODE == 2 && node_word <= 256 {
                            let mut v = [0u8; INLINE];
                            v[0] = (node_word - 1) as u8;
                            v[1] = a as u8;
                            self.inline16.push(v);
                            self.lens.push(2);
                            self.offsets.push(self.spelling.len() as u32);
                        } else if MODE == 2 && CAP > 0 && node_word < SlpCache::<CAP>::LOCAL_BASE {
                            let (st, ln) = e.cache.ensure_expanded(node_word, &mut self.aux_stack);
                            let tot_len = ln + 1;
                            let mut v = [0u8; INLINE];
                            if tot_len <= INLINE {
                                v[..ln].copy_from_slice(&e.cache.pool[st..st + ln]);
                                v[ln] = a as u8;
                                self.inline16.push(v);
                                self.lens.push(tot_len as u8);
                                self.offsets.push(self.spelling.len() as u32);
                            } else {
                                v.copy_from_slice(&e.cache.pool[st..st + INLINE]);
                                self.inline16.push(v);
                                self.lens.push(LONG);
                                self.spelling.extend_from_slice(&e.cache.pool[st..st + ln]);
                                self.spelling.push(a as u8);
                                self.offsets.push(self.spelling.len() as u32);
                            }
                        } else {
                            let start = self.spelling.len();
                            e.append_local(node_word, &mut self.stack, &mut self.spelling, &mut self.aux_stack);
                            self.spelling.push(a as u8);
                            let word = &self.spelling[start..];
                            let mut v = [0u8; INLINE];
                            let head = word.len().min(INLINE);
                            v[..head].copy_from_slice(&word[..head]);
                            self.inline16.push(v);
                            self.lens.push(if word.len() <= INLINE {
                                word.len() as u8
                            } else {
                                LONG
                            });
                            self.offsets.push(self.spelling.len() as u32);
                        }
                        code_id
                    };
                    self.bands.push(Band {
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
        }
        assert_eq!(id, 0);
        (cursors, words)
    }

    pub fn try_run(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> bool {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        out.resize(n + INLINE, 0);
        self.strips.clear();
        let (f, heads, over) = {
            let mut pass = Pass {
                queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.m),
                strips: &mut self.strips,
                f: First::default(),
            };
            maximal(records, |c, pos, width| {
                pass.take(c, pos, width, p, LIMIT)
            });
            (pass.f, pass.queues.heads(), pass.queues.over())
        };
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return false;
        }
        let First {
            all,
            thin,
            thick,
            m,
            mark,
        } = f;
        assert!(m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.strips.len();
        let need = Self::arena_words(ns, m as usize);
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
        if MODE != 0 {
            self.init_inline_literals();
        }
        let (cursors, words) = if Self::wide(ns) {
            self.induced_with::<Wide>(n as u32, &thin, m as usize)
        } else {
            self.induced_with::<Narrow>(n as u32, &thin, m as usize)
        };

        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + words as u32) > (u32::MAX >> bits);
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
            let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
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
        if MODE == 0 {
            self.init_inline_literals();
            let wcount = self.offsets.len().saturating_sub(1);
            self.inline16.reserve_exact(wcount);
            self.lens.reserve_exact(wcount);
            for w in self.offsets.windows(2) {
                let word = &self.spelling[w[0] as usize..w[1] as usize];
                let mut v = [0; INLINE];
                let head = word.len().min(INLINE);
                v[..head].copy_from_slice(&word[..head]);
                self.inline16.push(v);
                self.lens.push(if word.len() <= INLINE {
                    word.len() as u8
                } else {
                    LONG
                });
            }
        }
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
        true
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
    if at < n {
        let period = at;
        while at < n {
            let take = (n - at).min(period);
            out.copy_within(0..take, at);
            at += take;
        }
    }
    out.truncate(n);
}

#[derive(Default)]
pub struct XbOpenCells<const MODE: u8, const CAP: usize> {
    open: XbOpenKernel<MODE, CAP>,
    lazy: LazyCells,
}

impl<const MODE: u8, const CAP: usize> XbOpenCells<MODE, CAP> {
    fn cap(records: usize, n: usize, p: u32) -> Option<Cap> {
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
}

impl<const MODE: u8, const CAP: usize> InverseBwt for XbOpenCells<MODE, CAP> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Self::cap(r.records().len(), n, p) {
            if self.open.try_run(r.records(), n, p, cap, out) {
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

pub type AaOpenCells = XbOpenCells<0, 0>;
pub type LocalFusedCells = XbOpenCells<1, 0>;
pub type XbSlp8kCells = XbOpenCells<2, 8192>;
pub type XbSlp16kCells = XbOpenCells<2, 16384>;
