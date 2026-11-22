#![forbid(unsafe_code)]
//! Investigator Beta — Cycle 6 Candidate (v2: Contiguous F-order run table + Fast 1-load K=4/1-lane emit):
//! `256-Cursor Zero-Sort SelThinSq (L=16)` — Theorem B5.
//!
//! Eliminates:
//!   - 509k random reads of `run_len_c[u]` / `run_thin_base[u]` via contiguous `f_run_info`
//!   - per-block `inline_rev` allocation & 8-lane `3.5 MiB` `segs` L3 thrashing
//!   - `Vec::extend_from_slice` capacity checks in hot emit loop!

use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};

pub type Candidate = SelThinSq;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

const DENSE_NUM: usize = 7;
const DENSE_DEN: usize = 25;
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

const IS_THICK: u32 = 1 << 31;
const IS_ANCHOR_RUN: u32 = 1 << 30;
const STOP_MASK: u32 = IS_THICK | IS_ANCHOR_RUN;
const LEN_MASK: u32 = 0x3FFF_FF00;

const NEXT_MASK: u64 = 0xF_FFFF;
const IS_CODE_BIT: u64 = 1 << 20;
const EXTRA_BYTE_BIT: u64 = 1 << 21;
const ANCHOR_BIT: u64 = 1 << 22;

const MAX_LANES: usize = 4;
const PARALLEL_MIN_M: u32 = 32_768;

#[inline(always)]
fn pack_inline2(next: u32, b0: u8, b1: u8) -> u64 {
    u64::from(next) | (u64::from(b0) << 24) | (u64::from(b1) << 32)
}

#[inline(always)]
fn pack_code1(next: u32, code: u32) -> u64 {
    u64::from(next) | IS_CODE_BIT | (u64::from(code) << 32)
}

#[inline(always)]
fn pack_code2(next: u32, b0: u8, code: u32) -> u64 {
    u64::from(next) | IS_CODE_BIT | EXTRA_BYTE_BIT | (u64::from(b0) << 24) | (u64::from(code) << 32)
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

fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0u32;
    counts.map(|w| {
        let s = total;
        total += w;
        s
    })
}

fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
    let mut slot = [0u8; INLINE];
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
            let mut v = [0u8; INLINE];
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

// ============================================================================
// Thick-Strip Endpoint Elimination (identical to Open endpoint.rs)
// ============================================================================

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

// ============================================================================
// SelThinSq Kernel (v2: Contiguous F-order run table + Fast 1-lane / K=4 emit)
// ============================================================================

#[derive(Default)]
pub struct SelThinSq {
    lazy: LazyCells,
    arena: Vec<u32>,
    stack: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    inline_rev: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    // L-order runs (4B/run) + contiguous F-order (len_c, thin_base) (8B/run, 0 random reads!)
    run_len_c: Vec<u32>,
    run_thin_base: Vec<u32>,
    f_run_info: Vec<(u32, u32)>,
    // Packed 8B edges
    edges: Vec<u64>,
    // Reusable K=4 segment buffers
    segs: [Vec<u8>; MAX_LANES],
}

impl SelThinSq {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        most: usize,
    ) -> [usize; 256] {
        let Self {
            arena,
            stack,
            bands,
            spelling,
            inline16,
            lens,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(arena.len() >= 4 * ns + nodes_words);
        let (hdr, body) = arena.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> {
            a: body,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
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
            grow(arena, 4 * ns + top + RULE);
            e = Elim {
                a: &mut arena[4 * ns..],
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
        let (header, arena_tail) = arena.split_at_mut(4 * ns);
        let (live, tail) = arena_tail.split_at_mut(top);
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
        let mut cursors = [0usize; 256];
        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0u32;
        let mut k = 0u32;
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

    fn try_invert_runs(&mut self, records: &[u16], n: usize, p: u32, cap_m: u32, cap_strips: usize, out: &mut Vec<u8>) -> bool {
        let n32 = n as u32;
        self.run_len_c.clear();
        self.run_thin_base.clear();
        self.arena.clear();

        let mut thin = [0u32; 256];
        let mut thick_rows = [0u32; 256];
        let mut thick_cnt = [0u32; 256];
        let mut run_cnt = [0u32; 256];
        let mut m = 0u32;
        let mut ns = 0usize;
        let mut mark = 0u32;
        let mut mark_run = 0usize;

        let mut pos = 0u32;
        let mut i = 0usize;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            let a = usize::from(c);
            if width <= LIMIT {
                let r_idx = self.run_len_c.len();
                if p.wrapping_sub(pos) < width {
                    mark = m + (p - pos);
                    mark_run = r_idx;
                }
                self.run_len_c.push((width << 8) | u32::from(c));
                self.run_thin_base.push(m);
                thin[a] += width;
                run_cnt[a] += 1;
                m += width;
                if m > cap_m {
                    return false;
                }
            } else if p.wrapping_sub(pos) >= width {
                if ns >= cap_strips {
                    return false;
                }
                self.run_len_c.push(IS_THICK | (width << 8) | u32::from(c));
                self.run_thin_base.push(u32::MAX);
                self.arena.push(pos);
                self.arena.push(thin[a] + thick_rows[a]);
                self.arena.push((width << 8) | u32::from(c));
                ns += 1;
                thick_cnt[a] += 1;
                thick_rows[a] += width;
                run_cnt[a] += 1;
            } else {
                let end = pos + width;
                if pos < p {
                    let w = p - pos;
                    if ns >= cap_strips {
                        return false;
                    }
                    self.run_len_c.push(IS_THICK | (w << 8) | u32::from(c));
                    self.run_thin_base.push(u32::MAX);
                    self.arena.push(pos);
                    self.arena.push(thin[a] + thick_rows[a]);
                    self.arena.push((w << 8) | u32::from(c));
                    ns += 1;
                    thick_cnt[a] += 1;
                    thick_rows[a] += w;
                    run_cnt[a] += 1;
                }
                {
                    let r_idx = self.run_len_c.len();
                    mark = m;
                    mark_run = r_idx;
                    self.run_len_c.push((1u32 << 8) | u32::from(c));
                    self.run_thin_base.push(m);
                    thin[a] += 1;
                    run_cnt[a] += 1;
                    m += 1;
                    if m > cap_m {
                        return false;
                    }
                }
                if p + 1 < end {
                    let w = end - p - 1;
                    if ns >= cap_strips {
                        return false;
                    }
                    self.run_len_c.push(IS_THICK | (w << 8) | u32::from(c));
                    self.run_thin_base.push(u32::MAX);
                    self.arena.push(p + 1);
                    self.arena.push(thin[a] + thick_rows[a]);
                    self.arena.push((w << 8) | u32::from(c));
                    ns += 1;
                    thick_cnt[a] += 1;
                    thick_rows[a] += w;
                    run_cnt[a] += 1;
                }
            }
            pos += width;
        }
        if pos != n32 || m == 0 {
            return false;
        }

        let r0 = self.run_len_c.len();
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);

        let most = Self::nodes_most(ns);
        let wide = most > NARROW_NODES;
        let stride = if wide { Wide::STRIDE } else { Narrow::STRIDE };
        let need = 4 * ns + (stride + RULE) * most;
        if self.arena.len() < need {
            self.arena.resize(need, 0);
        }
        let mut write = starts(thick_cnt);
        for idx in 0..ns {
            let sw = &mut self.arena[3 * idx..3 * idx + 3];
            let a = (sw[2] & 0xFF) as usize;
            sw[1] += first[a];
            let slot = write[a] as usize;
            write[a] += 1;
            self.arena[3 * ns + slot] = idx as u32;
        }

        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 && self.spelling.capacity() < 48 * 1024 {
            self.spelling.reserve_exact(48 * 1024 - self.spelling.capacity());
        }
        let cursors = if wide {
            self.induced_with::<Wide>(ns, n32, m, &thin, most)
        } else {
            self.induced_with::<Narrow>(ns, n32, m, &thin, most)
        };

        // Mark mark_run (ALWAYS protected -> exact period & no overshoot on 1-lane emit!)
        self.run_len_c[mark_run] |= IS_ANCHOR_RUN;

        // Pack contiguous F-order (len_c, thin_base) -> ZERO random reads during S3_256cur!
        let mut f_write = starts(run_cnt);
        self.f_run_info.resize(r0, (0, 0));
        for u in 0..r0 {
            let lc = self.run_len_c[u];
            let a = (lc & 0xFF) as usize;
            let dst = f_write[a] as usize;
            f_write[a] += 1;
            self.f_run_info[dst] = (lc, self.run_thin_base[u]);
        }

        // Stage 3: Theorem B5 256-Cursor Zero-Sort SelThinSq Wiring
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        let edges = &mut self.edges[..m as usize];
        let bands = &self.bands;
        let f_info = &self.f_run_info[..r0];
        let l_info = &self.run_len_c[..r0];

        let mut b_idx_u = 0usize;
        let mut b_k_u = 0u32;
        let mut b_start_u = 0u32;

        let mut b_idx_v = cursors;
        let mut b_k_v = starts(thin);
        let mut b_start_v = b_k_v;

        let mut iu = 0usize;
        let mut v = 0usize;
        let mut ou = 0u32;
        let mut ov = 0u32;

        while iu < r0 && v < r0 {
            let (lc_u, base_u) = f_info[iu];
            let len_u = (lc_u & LEN_MASK) >> 8;
            let lc_v = l_info[v];
            let len_v = (lc_v & LEN_MASK) >> 8;

            let take = (len_u - ou).min(len_v - ov);
            let thick_u = (lc_u & IS_THICK) != 0;
            let thick_v = (lc_v & IS_THICK) != 0;

            if thick_u {
                if !thick_v {
                    let cv = (lc_v & 0xFF) as usize;
                    let target = b_k_v[cv] + take;
                    let mut idx = b_idx_v[cv];
                    let mut st = b_start_v[cv];
                    while idx < bands.len() && bands[idx].end() <= target {
                        st = bands[idx].end();
                        idx += 1;
                    }
                    b_idx_v[cv] = idx;
                    b_k_v[cv] = target;
                    b_start_v[cv] = st;
                }
            } else {
                let cu = (lc_u & 0xFF) as u8;
                let mut x_tid = (base_u + ou) as usize;

                if (lc_v & STOP_MASK) != 0 {
                    let mut rem = take;
                    while rem > 0 {
                        let bnd = bands[b_idx_u];
                        let bend = bnd.end();
                        let step = rem.min(bend - b_k_u);
                        let mut dst = bnd.dst() + (b_k_u - b_start_u);
                        let code = bnd.code();
                        for slot in &mut edges[x_tid..x_tid + step as usize] {
                            *slot = pack_code1(dst, code);
                            dst += 1;
                        }
                        x_tid += step as usize;
                        b_k_u += step;
                        rem -= step;
                        if b_k_u == bend {
                            b_start_u = bend;
                            b_idx_u += 1;
                        }
                    }
                    if !thick_v {
                        let cv = (lc_v & 0xFF) as usize;
                        let target = b_k_v[cv] + take;
                        let mut idx = b_idx_v[cv];
                        let mut st = b_start_v[cv];
                        while idx < bands.len() && bands[idx].end() <= target {
                            st = bands[idx].end();
                            idx += 1;
                        }
                        b_idx_v[cv] = idx;
                        b_k_v[cv] = target;
                        b_start_v[cv] = st;
                    }
                } else {
                    let target_u = b_k_u + take;
                    while b_idx_u < bands.len() && bands[b_idx_u].end() <= target_u {
                        b_start_u = bands[b_idx_u].end();
                        b_idx_u += 1;
                    }
                    b_k_u = target_u;

                    let cv = (lc_v & 0xFF) as usize;
                    let cv_u8 = cv as u8;
                    let mut idx = b_idx_v[cv];
                    let mut kv = b_k_v[cv];
                    let mut st = b_start_v[cv];
                    let mut rem = take;
                    while rem > 0 {
                        let bnd = bands[idx];
                        let bend = bnd.end();
                        let step = rem.min(bend - kv);
                        let mut dst2 = bnd.dst() + (kv - st);
                        let code2 = bnd.code();
                        if code2 < 256 {
                            for slot in &mut edges[x_tid..x_tid + step as usize] {
                                *slot = pack_inline2(dst2, cu, cv_u8);
                                dst2 += 1;
                            }
                        } else {
                            for slot in &mut edges[x_tid..x_tid + step as usize] {
                                *slot = pack_code2(dst2, cu, code2);
                                dst2 += 1;
                            }
                        }
                        x_tid += step as usize;
                        kv += step;
                        rem -= step;
                        if kv == bend {
                            st = bend;
                            idx += 1;
                        }
                    }
                    b_idx_v[cv] = idx;
                    b_k_v[cv] = kv;
                    b_start_v[cv] = st;
                }
            }

            ou += take;
            ov += take;
            if ou == len_u {
                iu += 1;
                ou = 0;
            }
            if ov == len_v {
                v += 1;
                ov = 0;
            }
        }
        assert!(iu == r0 && v == r0);

        // Stage 4: Direct 1-lane backward emit (ZERO segs scratch, ZERO reverse, ZERO stitch!)
        // Uses branchless 16-byte shifted u128 store for inline16 words (matching Open's emit_bidirectional!).
        self.emit_single_lane_fast(n, m as usize, mark, out);
        true
    }

    fn emit_single_lane_fast(&self, n: usize, m: usize, mark: u32, out: &mut Vec<u8>) {
        out.resize(n + INLINE, 0);
        let edges = &self.edges[..m];
        let inline16 = &self.inline16;
        let lens = &self.lens;
        let spelling = &self.spelling;

        let mut hi = n + INLINE;
        let floor = INLINE;
        let mut cur = mark as usize;

        for _ in 0..=m {
            let e = edges[cur];
            cur = (e & NEXT_MASK) as usize;
            if (e & IS_CODE_BIT) == 0 {
                // 73% HOTTEST PATH: single u64 load, ZERO dict load, 1 unaligned u16 store!
                let b0 = (e >> 24) as u8;
                let b1 = (e >> 32) as u8;
                if hi >= floor + 2 {
                    hi -= 2;
                    out[hi] = b1;
                    out[hi + 1] = b0;
                } else if hi > floor {
                    hi -= 1;
                    out[hi] = b0;
                }
            } else {
                if (e & EXTRA_BYTE_BIT) != 0 && hi > floor {
                    hi -= 1;
                    out[hi] = (e >> 24) as u8;
                }
                let code = (e >> 32) as usize;
                let len = lens[code];
                if len != LONG && hi >= floor + INLINE {
                    let l = usize::from(len);
                    let shifted = u128::from_le_bytes(inline16[code]) << (8 * (INLINE - l));
                    out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= l;
                } else {
                    let word = if len == LONG {
                        long_word(&inline16[code], spelling)
                    } else {
                        &inline16[code][..usize::from(len)]
                    };
                    let take = word.len().min(hi - floor);
                    hi -= take;
                    out[hi..hi + take].copy_from_slice(&word[word.len() - take..]);
                }
            }
            if cur == mark as usize || hi == floor {
                break;
            }
        }
        let produced = (n + INLINE) - hi;
        if hi != 0 {
            out.copy_within(hi..n + INLINE, 0);
        }
        out.truncate(produced.min(n));
        repeat(n, out);
    }

    pub fn scratch_kib(&self) -> usize {
        (self.arena.capacity() * 4
            + self.stack.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * 16
            + self.inline_rev.capacity() * 16
            + self.lens.capacity()
            + self.run_len_c.capacity() * 4
            + self.run_thin_base.capacity() * 4
            + self.f_run_info.capacity() * 8
            + self.edges.capacity() * 8)
            / 1024
    }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

impl InverseBwt for SelThinSq {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let recs = r.records();
        if n > 0 && n <= 900_000 && (p as usize) < n {
            let (admit, cap_m, cap_strips) = if recs.len() * 20 < n * 3 && recs.len() * 64 >= n {
                (true, u32::MAX, usize::MAX)
            } else if recs.len() * 64 < n {
                (true, u32::MAX, usize::MAX)
            } else if recs.len() * DENSE_DEN < n * DENSE_NUM {
                (true, DENSE_M.min((n as u64 * 3 / 10) as u32), DENSE_STRIPS)
            } else {
                (false, 0, 0)
            };
            if admit && self.try_invert_runs(recs, n, p, cap_m, cap_strips, out) {
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
