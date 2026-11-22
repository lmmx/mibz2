#![forbid(unsafe_code)]

//! Cycle 49 P2 — Analogue K, Theorem K2:
//! Cayley (1857, 1859) Rooted-Tree Anatomy of `Open<16>`'s `S2b` `open_nodes` / `spelling`
//! Grammar Forest over `F-Inline3` & Exact Cayley Rooted-Tree Transformation Experiments.
//!
//! Primary Sources:
//!   - A. Cayley, "On the theory of the analytical forms called trees",
//!     Phil. Mag. 13 (1857), 172-176 & 18 (1859), 374-378
//!   - A. Cayley, "A theorem on trees", Quart. J. Pure Appl. Math. 23 (1889), 376-378
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Barrier;
use std::time::Instant;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

// F-Inline3 handle encoding (ID_BITS = 24, inline g=1..3, RULE_BASE = 0, SATURATED = 255)
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = 255;

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
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

#[derive(Clone, Copy)]
struct First {
    thick_rows: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
}

impl Default for First {
    fn default() -> Self {
        First {
            thick_rows: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            mark_byte: 0,
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
    if words > 0 && inline16.capacity() < 256 + words {
        inline16.reserve_exact(256 + words - inline16.len());
        lens.reserve_exact(256 + words - lens.len());
    }
}

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

fn spelling_hash_long(bytes: &[u8]) -> u64 {
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8)
            .rotate_left(29)
            .wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

// ============================================================================
// 1. Links Trait & Elim Engine with Const-Parameterized Cayley Transformations
// ============================================================================

trait Links {
    const STRIDE: usize;
    const IS_NARROW: bool;
    const BOTTOM: usize;
    fn prev(a: &[u32], id: u32, side: usize) -> u32;
    fn next(a: &[u32], id: u32, side: usize) -> u32;
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
}

struct Narrow;
struct Wide;

impl Links for Narrow {
    const STRIDE: usize = 4;
    const IS_NARROW: bool = true;
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
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const IS_NARROW: bool = false;
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
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

/// Const-parameterized Cayley rooted-tree modes:
///   CAT_COMPRESS: true => Cayley unary-caterpillar inline-3 path compression in S2b join()
///   TAIL_UNWIND:  true => Cayley low-Strahler register-spine tail-call unwind in S3a append()
struct Elim<'a, L: Links, const CAT_COMPRESS: bool, const TAIL_UNWIND: bool> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const CAT_COMPRESS: bool, const TAIL_UNWIND: bool>
    Elim<'_, L, CAT_COMPRESS, TAIL_UNWIND>
{
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
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> ID_BITS;
        let id = h & ID_MASK;
        if g <= 3 {
            return u128::from(id);
        }
        let r = self.rule(id);
        let sl = &self.a[r..r + 4];
        u128::from(sl[0])
            | (u128::from(sl[1]) << 32)
            | (u128::from(sl[2]) << 64)
            | (u128::from(sl[3] & 0x00ff_ffff) << 96)
    }

    #[inline(always)]
    fn join(&mut self, mut left: u32, mut right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let right_len = right >> ID_BITS;
        let sum = left_len + right_len;
        if sum < INLINE as u32 {
            if sum <= 3 {
                let lp = left & ID_MASK;
                let rp = right & ID_MASK;
                return (sum << ID_BITS) | lp | (rp << (8 * left_len));
            }
            let payload =
                self.short_payload_h(left) | (self.short_payload_h(right) << (8 * left_len));
            let id = ((self.top - self.base) / RULE) as u32;
            let bytes = payload | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            // Cayley rooted-tree caterpillar path-compression (zero extra arena allocation):
            // Eliminate unary degree-1 internal chain nodes when a short leaf (g <= 2) attaches
            // to a long internal node whose adjacent child is also short with combined len <= 3.
            if CAT_COMPRESS {
                if left_len >= INLINE as u32 && right_len <= 2 {
                    let rl = self.rule(left & ID_MASK);
                    let lr = self.a[rl + 1];
                    let lr_len = lr >> ID_BITS;
                    if lr_len + right_len <= 3 {
                        let ll = self.a[rl];
                        let merged = ((lr_len + right_len) << ID_BITS)
                            | (lr & ID_MASK)
                            | ((right & ID_MASK) << (8 * lr_len));
                        left = ll;
                        right = merged;
                    }
                } else if right_len >= INLINE as u32 && left_len <= 2 {
                    let rr = self.rule(right & ID_MASK);
                    let rl = self.a[rr];
                    let rl_len = rl >> ID_BITS;
                    if left_len + rl_len <= 3 {
                        let r_right = self.a[rr + 1];
                        let merged = ((left_len + rl_len) << ID_BITS)
                            | (left & ID_MASK)
                            | ((rl & ID_MASK) << (8 * left_len));
                        left = merged;
                        right = r_right;
                    }
                }
            }
            let id = ((self.top - self.base) / RULE) as u32;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
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

    #[inline(never)]
    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                continue;
            }
            let ny = &self.a[yi..yi + 4];
            let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            let word = self.join(wx, wy);
            if aw <= bw {
                let px = x0 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                if aw < bw {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y | (y_next << 16);
                    nxm[3] = word;
                    let nym = &mut self.a[yi..yi + 4];
                    nym[0] = (y0 & 0xFFFF) | (x << 16);
                    nym[2] = bw - aw;
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    let y_next = if y == px { 0 } else { y0 >> 16 };
                    let y_prev = y0 & 0xFFFF;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[3] = word;
                    let ypi = y_prev as usize * 4;
                    self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                }
            } else {
                let py = y1 & 0xFFFF;
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                let nym = &mut self.a[yi..yi + 4];
                nym[1] = x | (x_next << 16);
                nym[3] = word;
                let nxm = &mut self.a[xi..xi + 4];
                nxm[1] = (x1 & 0xFFFF) | (y << 16);
                nxm[2] = aw - bw;
                let xni = x_next as usize * 4 + 1;
                self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        if !SPIRALS && L::IS_NARROW {
            return self.eliminate_narrow_plain(remaining);
        }
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
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        if !TAIL_UNWIND {
            stack.clear();
            stack.push(handle);
            while let Some(h) = stack.pop() {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&bytes[..g as usize]);
                } else {
                    let r = self.rule(h & ID_MASK);
                    let tag = self.a[r + 3];
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
        } else {
            // Cayley low-Strahler register-spine tail-call unwind:
            // Avoids pushing/popping left children and emits short leaf-pairs & right-caterpillars
            // with ZERO stack traffic!
            stack.clear();
            let mut h = handle;
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&bytes[..g as usize]);
                    match stack.pop() {
                        Some(next) => {
                            h = next;
                            continue;
                        }
                        None => return,
                    }
                }
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + rlen);
                    match stack.pop() {
                        Some(next) => {
                            h = next;
                            continue;
                        }
                        None => return,
                    }
                }
                self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                let left = self.a[r];
                let right = self.a[r + 1];
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        dest.extend_from_slice(&rb[..rg as usize]);
                        match stack.pop() {
                            Some(next) => {
                                h = next;
                                continue;
                            }
                            None => return,
                        }
                    }
                    h = right;
                } else {
                    stack.push(right);
                    h = left;
                }
            }
        }
    }

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append(word, stack, spelling);
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }
}

struct Dictionary<'a> {
    pairs: &'a mut [u32; 512],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links, const CAT: bool, const TAIL: bool>(
        &mut self,
        e: &mut Elim<L, CAT, TAIL>,
        handle: u32,
        byte: u8,
    ) -> u32 {
        let bucket = ((handle.wrapping_mul(0x9e37_79b9)
            ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
            >> 24) as usize;
        let slot = bucket * 2;
        let val = self.pairs[slot + 1];
        if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
            return val & 0x00FF_FFFF;
        }
        let code = self.code_miss(e, handle, handle & ID_MASK, byte);
        self.pairs[slot] = handle;
        self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
        code
    }

    fn code_miss<L: Links, const CAT: bool, const TAIL: bool>(
        &mut self,
        e: &mut Elim<L, CAT, TAIL>,
        handle: u32,
        id: u32,
        byte: u8,
    ) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            return ent & 0x00FF_FFFF;
        }
        let c = self.insert(entry, LONG);
        self.contents[index] = tag | c;
        c
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

// ============================================================================
// 2. Full Open<16> Kernel Wrapper (Matching Cycle 48 F-Inline3 Verbatim)
// ============================================================================

#[derive(Default, Clone, Copy, Debug)]
struct StageTimes {
    s1_us: f64,
    s2a_us: f64,
    s2b_us: f64,
    s3a_us: f64,
    s4_us: f64,
    s5_us: f64,
}

impl StageTimes {
    fn induced_us(&self) -> f64 {
        self.s2a_us + self.s2b_us + self.s3a_us
    }
}

#[derive(Default)]
struct KKernel {
    spill: outq::Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    force_wide: bool,
    force_spirals: Option<bool>,
}

#[inline(always)]
fn split_run(
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

impl KKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }
    fn wide(&self, ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES || self.force_wide
    }
    fn arena_words(&self, ns: usize, m: usize) -> usize {
        let stride = if self.wide(ns) { Wide::STRIDE } else { Narrow::STRIDE };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }
    fn ws_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.stack.capacity() * 4
    }

    fn run<const CAT: bool, const TAIL: bool, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> StageTimes {
        let mut tm = StageTimes::default();
        out.resize(n + INLINE, 0);
        if n == 0 {
            out.truncate(0);
            return tm;
        }
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        let mut f = First::default();
        let mut ns = 0usize;
        let heads = {
            let mut queues = outq::Queues::new(out, &mut self.spill, u32::MAX);
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
                if width <= LIMIT {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        f.mark = f.m + (p - pos);
                        f.mark_byte = c;
                    }
                    queues.push_thin(a, f.m, width);
                    f.m += width;
                } else if p.wrapping_sub(pos) >= width {
                    let a = usize::from(c);
                    let at = 3 * ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3).max(2 * self.edges.len()).max(1024);
                        self.edges.reserve_exact(need - self.edges.len());
                        self.edges.resize(need, 0);
                    }
                    let thin_a = queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[
                        pos,
                        thin_a + f.thick_rows[a],
                        (width << 8) | u32::from(c),
                    ]);
                    ns += 1;
                    f.thick[a] += 1;
                    f.thick_rows[a] += width;
                } else {
                    split_run(c, pos, width, p, LIMIT, |ch, x, w, keep| {
                        let a = usize::from(ch);
                        if keep {
                            if p.wrapping_sub(x) < w {
                                f.mark = f.m + (p - x);
                                f.mark_byte = ch;
                            }
                            queues.push_thin(a, f.m, w);
                            f.m += w;
                        } else {
                            let at = 3 * ns;
                            if self.edges.len() < at + 3 {
                                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                                self.edges.reserve_exact(need - self.edges.len());
                                self.edges.resize(need, 0);
                            }
                            let thin_a = queues.thin_of(a);
                            self.edges[at..at + 3].copy_from_slice(&[
                                x,
                                thin_a + f.thick_rows[a],
                                (w << 8) | u32::from(ch),
                            ]);
                            ns += 1;
                            f.thick[a] += 1;
                            f.thick_rows[a] += w;
                        }
                    });
                }
                pos += width;
            }
            assert_eq!(pos as usize, n);
            f.thin = queues.extract_thin();
            queues.heads()
        };
        if let Some(t) = t_s1 {
            tm.s1_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let m = f.m as usize;
        let mut counts = [0u32; 256];
        for a in 0..256 {
            counts[a] = f.thin[a] + f.thick_rows[a];
        }
        let first = starts(counts);
        let mut write = starts(f.thick);

        let need = self.arena_words(ns, m);
        if self.edges.len() < need {
            self.edges.reserve_exact(need - self.edges.len());
            self.edges.resize(need, 0);
        }
        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 {
            if self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192 - self.bands.len());
            }
            if self.spelling.capacity() < 49_152 {
                self.spelling.reserve_exact(49_152 - self.spelling.len());
            }
        }

        let cursors = if self.wide(ns) {
            self.induced::<Wide, CAT, TAIL, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        } else {
            self.induced::<Narrow, CAT, TAIL, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        };

        // S4: Wiring matching cand_r5_head/mod.rs verbatim
        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
        if self.edges.len() < m {
            self.edges.resize(m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m, 0);
        }
        let mark = f.mark;
        let mb = usize::from(f.mark_byte);
        let use_xor = !wide && m >= 65_536;
        let mut predecessor = mark;
        if use_xor {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                }
            }
        }
        if let Some(t) = t_s4 {
            tm.s4_us = t.elapsed().as_secs_f64() * 1e6;
        }

        // S5: Emit matching cand_r5_head::walk verbatim (forward-FL exact on valid & malformed!)
        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        let edges = &self.edges[..m];
        let inline16 = &self.inline16;
        let lens = &self.lens;
        let spelling = &self.spelling;
        let mask = (1u32 << bits) - 1;
        if use_xor {
            let (mut prev, mut front) = (predecessor, mark);
            let (mut back, mut next) = (predecessor, mark);
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
                    out[lo..lo + INLINE].copy_from_slice(&inline16[cf]);
                    lo += usize::from(lf);
                } else {
                    let word = if lf == LONG {
                        long_word(&inline16[cf], spelling)
                    } else {
                        &inline16[cf][..usize::from(lf)]
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
                    let shifted =
                        u128::from_le_bytes(inline16[cb]) << (8 * (INLINE - usize::from(lb)));
                    out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                    hi -= usize::from(lb);
                } else {
                    let word = if lb == LONG {
                        long_word(&inline16[cb], spelling)
                    } else {
                        &inline16[cb][..usize::from(lb)]
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
                let period = lo + (n - hi);
                let mut filled = period;
                while filled < n {
                    let take = filled.min(n - filled);
                    out.copy_within(0..take, filled);
                    filled += take;
                }
            }
        } else {
            let mut cur = mark;
            let mut pos_out = 0usize;
            while pos_out < n {
                let w = edges[cur as usize];
                let code = if wide {
                    self.extra[cur as usize] as usize
                } else {
                    (w >> bits) as usize
                };
                cur = w & mask;
                let len_b = lens[code];
                if len_b != LONG {
                    let l_sz = usize::from(len_b);
                    let take = l_sz.min(n - pos_out);
                    out[pos_out..pos_out + take].copy_from_slice(&inline16[code][..take]);
                    pos_out += take;
                } else {
                    let sl = long_word(&inline16[code], spelling);
                    let take = sl.len().min(n - pos_out);
                    out[pos_out..pos_out + take].copy_from_slice(&sl[..take]);
                    pos_out += take;
                }
            }
        }
        out.truncate(n);
        if let Some(t) = t_s5 {
            tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
        }
        tm
    }

    #[allow(clippy::too_many_arguments)]
    fn induced<L: Links, const CAT: bool, const TAIL: bool, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        m: u32,
        tm: &mut StageTimes,
    ) -> [usize; 256] {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let (hdr, arena) = self.edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L, CAT, TAIL> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
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
            let to = strips_raw[3 * idx + 1];
            let width = strips_raw[3 * idx + 2] >> 8;
            if x < to {
                widest = widest.max(to - x);
                let id = e.node(to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = to + width;
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
        if let Some(t) = t_s2a {
            tm.s2a_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let mut remaining = n;
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
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
        if let Some(t) = t_s2b {
            tm.s2b_us = t.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        e.assign_bottom(m);
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS { header } else { tail };
        let mut e = Elim::<L, CAT, TAIL> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);
        let mut cursors = [0usize; 256];
        if cache.len() < CACHE_WORDS {
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(node_word, a as u8, &mut self.stack, &mut self.spelling);
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    self.bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
        } else {
            cache[..CACHE_WORDS].fill(0);
            let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
            let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
            let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
            let mut dictionary = Dictionary {
                pairs,
                contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut self.stack,
            };
            let mut id = L::next(e.a, 0, 0);
            let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                L::band_node(e.a, id)
            } else {
                (0, 0, 0, 0)
            };
            let mut in_node = 0u32;
            let mut k = 0u32;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                let mut last_w = u32::MAX;
                let mut last_c = 0u32;
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else if node_word == last_w {
                        last_c
                    } else {
                        let c = dictionary.code(&mut e, node_word, a as u8);
                        last_w = node_word;
                        last_c = c;
                        c
                    };
                    self.bands.push(Band::new(width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                        if id != 0 {
                            (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                        }
                    }
                }
            }
        }
        if let Some(t) = t_s3a {
            tm.s3a_us = t.elapsed().as_secs_f64() * 1e6;
        }
        cursors
    }
}

// ============================================================================
// 3. Part 1: Exact Cayley (1859) Rooted-Tree Anatomy Census over F-Inline3
// ============================================================================

#[derive(Default, Clone, Debug)]
struct CayleyAnatomy {
    open_nodes: usize,
    surv_nodes: usize,
    root_zero: usize,
    root_inline3: usize,
    root_short4_15: usize,
    root_long: usize,
    unique_long_roots: usize,
    // Rule counts (all vs live reachable from surviving side-0 roots)
    rules_total: usize,
    rules_short: usize,
    rules_long: usize,
    long_live: usize,
    long_dead: usize,
    // Arity / child-type breakdown of live g>=16 internal binary nodes
    node_ll: usize, // (short, short)
    node_il: usize, // (long, short) left-caterpillar
    node_li: usize, // (short, long) right-caterpillar
    node_ii: usize, // (long, long) true binary branching
    cat_i3_ops: usize, // IL/LI nodes collapsible by inline-3 path compression
    cat_p15_ops: usize, // IL/LI nodes collapsible into prefix-15 leaf
    // DAG in-degree distribution of live g>=16 nodes
    indeg_1: usize, // pure unshared tree node!
    indeg_2: usize,
    indeg_3p: usize,
    // Depth & Strahler-Horton branching number over all unique long roots
    sum_root_height: usize,
    max_root_height: usize,
    sum_root_strahler: usize,
    max_root_strahler: usize,
    strahler_hist: [usize; 6], // index = S clamped to 5 (S=0..=4, 5+)
    unrolled_tree_nodes: u64,
    // Append traversal simulation (base stack pushes vs TailUnwind stack pushes & memo hits)
    base_stack_pushes: usize,
    tail_stack_pushes: usize,
    append_memo_hits: usize,
    append_memo_misses: usize,
    // S3a spelling shared-prefix trie & shared node_word multiplicity
    long_dict_entries: usize,
    unique_long_nw_used: usize,
    spelling_bytes: usize,
    trie_edge_bytes: usize,
    nw_dup_bytes: usize,
}

fn analyze_cayley_forest<const CAT_COMPRESS: bool>(records: &[u16], n: usize, p: u32) -> CayleyAnatomy {
    let mut k = KKernel::default();
    let mut out = vec![0u8; n + INLINE];
    // Run S1..S2b cleanly to inspect arena BEFORE S3a overwrites memo tags!
    let mut f = First::default();
    let mut ns = 0usize;
    let min_strips_words = (n / (LIMIT as usize + 1) + 4) * 3;
    k.edges.resize(min_strips_words, 0);
    {
        let mut queues = outq::Queues::new(&mut out, &mut k.spill, u32::MAX);
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
            split_run(c, pos, width, p, LIMIT, |ch, x, w, keep| {
                let a = usize::from(ch);
                if keep {
                    if p.wrapping_sub(x) < w {
                        f.mark = f.m + (p - x);
                        f.mark_byte = ch;
                    }
                    queues.push_thin(a, f.m, w);
                    f.m += w;
                } else {
                    let at = 3 * ns;
                    if k.edges.len() < at + 3 {
                        let need = (at + 3).max(2 * k.edges.len()).max(1024);
                        k.edges.resize(need, 0);
                    }
                    let thin_a = queues.thin_of(a);
                    k.edges[at..at + 3].copy_from_slice(&[
                        x,
                        thin_a + f.thick_rows[a],
                        (w << 8) | u32::from(ch),
                    ]);
                    ns += 1;
                    f.thick[a] += 1;
                    f.thick_rows[a] += w;
                }
            });
            pos += width;
        }
        assert_eq!(pos as usize, n);
        f.thin = queues.extract_thin();
    }

    let m = f.m as usize;
    let mut counts = [0u32; 256];
    for a in 0..256 { counts[a] = f.thin[a] + f.thick_rows[a]; }
    let first = starts(counts);
    let mut write = starts(f.thick);
    let need = k.arena_words(ns, m);
    if k.edges.len() < need { k.edges.resize(need, 0); }

    fn inspect<L: Links, const CAT: bool>(
        edges: &mut Vec<u32>,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        m: u32,
    ) -> CayleyAnatomy {
        let mut ca = CayleyAnatomy::default();
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L, CAT, false> {
            a: arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
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
            let to = strips_raw[3 * idx + 1];
            let width = strips_raw[3 * idx + 2] >> 8;
            if x < to {
                widest = widest.max(to - x);
                let id = e.node(to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = to + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        ca.open_nodes = e.nodes as usize - 1;
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

        let num_rules = (e.top - e.base) / RULE;
        ca.rules_total = num_rules;
        // Identify which rule IDs are long (out[3] == u32::MAX) vs short (out[3] >> 24 in 4..=15)
        let mut is_long = vec![false; num_rules];
        for rid in 0..num_rules {
            let r = e.base + rid * RULE;
            if e.a[r + 3] == u32::MAX {
                is_long[rid] = true;
                ca.rules_long += 1;
            } else {
                ca.rules_short += 1;
            }
        }

        // Walk surviving side-0 list to collect all roots!
        let mut id = L::next(e.a, 0, 0);
        let mut long_roots_all: Vec<u32> = Vec::new();
        while id != 0 {
            ca.surv_nodes += 1;
            let w = e.word(id);
            if w == 0 {
                ca.root_zero += 1;
            } else {
                let g = w >> ID_BITS;
                if g <= 3 {
                    ca.root_inline3 += 1;
                } else if g < INLINE as u32 {
                    ca.root_short4_15 += 1;
                } else {
                    ca.root_long += 1;
                    long_roots_all.push(w & ID_MASK);
                }
            }
            id = L::next(e.a, id, 0);
        }

        let mut unique_roots: Vec<u32> = long_roots_all.clone();
        unique_roots.sort_unstable();
        unique_roots.dedup();
        ca.unique_long_roots = unique_roots.len();

        // Mark live g>=16 rules & compute exact DAG in-degree from surviving roots + live internal parents!
        let mut live = vec![false; num_rules];
        let mut in_deg = vec![0usize; num_rules];
        let mut dfs_stack: Vec<usize> = Vec::new();
        for &rid in &unique_roots {
            let u = rid as usize;
            in_deg[u] += 1;
            if !live[u] {
                live[u] = true;
                dfs_stack.push(u);
            }
        }
        while let Some(u) = dfs_stack.pop() {
            let r = e.base + u * RULE;
            let lc = e.a[r];
            let rc = e.a[r + 1];
            if (lc >> ID_BITS) >= INLINE as u32 {
                let v = (lc & ID_MASK) as usize;
                in_deg[v] += 1;
                if !live[v] {
                    live[v] = true;
                    dfs_stack.push(v);
                }
            }
            if (rc >> ID_BITS) >= INLINE as u32 {
                let v = (rc & ID_MASK) as usize;
                in_deg[v] += 1;
                if !live[v] {
                    live[v] = true;
                    dfs_stack.push(v);
                }
            }
        }

        // Dynamic programming bottom-up (since rid of children < rid of parent!) for exact
        // height, Strahler-Horton branching number S(T), and unrolled tree size!
        let mut height = vec![0usize; num_rules];
        let mut strahler = vec![1usize; num_rules];
        let mut tree_sz = vec![1u64; num_rules];

        for u in 0..num_rules {
            if !is_long[u] {
                continue;
            }
            if !live[u] {
                ca.long_dead += 1;
                continue;
            }
            ca.long_live += 1;
            match in_deg[u] {
                0 | 1 => ca.indeg_1 += 1,
                2 => ca.indeg_2 += 1,
                _ => ca.indeg_3p += 1,
            }
            let r = e.base + u * RULE;
            let lc = e.a[r];
            let rc = e.a[r + 1];
            let lg = lc >> ID_BITS;
            let rg = rc >> ID_BITS;
            let l_long = lg >= INLINE as u32;
            let r_long = rg >= INLINE as u32;

            let (lh, ls, lsz) = if l_long {
                let li = (lc & ID_MASK) as usize;
                (height[li], strahler[li], tree_sz[li])
            } else {
                (0, 1, 1)
            };
            let (rh, rs, rsz) = if r_long {
                let ri = (rc & ID_MASK) as usize;
                (height[ri], strahler[ri], tree_sz[ri])
            } else {
                (0, 1, 1)
            };
            height[u] = 1 + lh.max(rh);
            strahler[u] = if ls == rs { ls + 1 } else { ls.max(rs) };
            tree_sz[u] = 1u64.saturating_add(lsz).saturating_add(rsz);

            match (l_long, r_long) {
                (false, false) => ca.node_ll += 1,
                (true, false) => {
                    ca.node_il += 1;
                    let rl = e.base + (lc & ID_MASK) as usize * RULE;
                    let lr_g = e.a[rl + 1] >> ID_BITS;
                    if lr_g + rg <= 3 {
                        ca.cat_i3_ops += 1;
                    }
                    if lr_g < INLINE as u32 && lr_g + rg < INLINE as u32 {
                        ca.cat_p15_ops += 1;
                    }
                }
                (false, true) => {
                    ca.node_li += 1;
                    let rr = e.base + (rc & ID_MASK) as usize * RULE;
                    let rl_g = e.a[rr] >> ID_BITS;
                    if lg + rl_g <= 3 {
                        ca.cat_i3_ops += 1;
                    }
                    if rl_g < INLINE as u32 && lg + rl_g < INLINE as u32 {
                        ca.cat_p15_ops += 1;
                    }
                }
                (true, true) => ca.node_ii += 1,
            }
        }

        for &rid in &unique_roots {
            let u = rid as usize;
            let h = height[u];
            let s = strahler[u];
            ca.sum_root_height += h;
            ca.max_root_height = ca.max_root_height.max(h);
            ca.sum_root_strahler += s;
            ca.max_root_strahler = ca.max_root_strahler.max(s);
            ca.strahler_hist[s.min(5)] += 1;
            ca.unrolled_tree_nodes = ca.unrolled_tree_nodes.saturating_add(tree_sz[u]);
        }

        // Simulate S3a dictionary & exact append() traversal + Cayley shared-prefix trie of spelling!
        e.assign_bottom(m);
        let mut memo_sim = vec![false; num_rules];
        let mut memo_sim_tail = vec![false; num_rules];
        let mut stack = Vec::new();
        let mut spelling = Vec::new();
        let mut inline16 = Vec::new();
        let mut lens = Vec::new();
        begin_inline(0, &mut inline16, &mut lens);
        let mut seen_pair: HashSet<(u32, u8)> = HashSet::new();
        let mut unique_nw: HashMap<u32, usize> = HashMap::new(); // node_word -> len
        let mut long_words_bytes: Vec<Vec<u8>> = Vec::new();
        let mut seen_long_word: HashSet<Vec<u8>> = HashSet::new();

        let mut id = L::next(e.a, 0, 0);
        let (mut next_id, _, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k_pos = 0u32;
        for a in 0..256 {
            let end = k_pos + thin[a];
            while k_pos < end {
                let width = (end - k_pos).min(node_width - in_node);
                if node_word != 0 && (node_word >> ID_BITS) >= INLINE as u32
                    && seen_pair.insert((node_word, a as u8))
                {
                    // Base stack push count simulation
                    {
                        let mut st = vec![node_word];
                        ca.base_stack_pushes += 1;
                        while let Some(h) = st.pop() {
                            if (h >> ID_BITS) >= INLINE as u32 {
                                let rid = (h & ID_MASK) as usize;
                                if !memo_sim[rid] {
                                    memo_sim[rid] = true;
                                    ca.append_memo_misses += 1;
                                    let r = e.base + rid * RULE;
                                    st.push(e.a[r + 1]);
                                    st.push(e.a[r]);
                                    ca.base_stack_pushes += 2;
                                } else {
                                    ca.append_memo_hits += 1;
                                }
                            }
                        }
                    }
                    // TailUnwind stack push count simulation
                    {
                        let mut st = Vec::new();
                        let mut h = node_word;
                        loop {
                            if (h >> ID_BITS) < INLINE as u32 {
                                match st.pop() {
                                    Some(nx) => { h = nx; continue; }
                                    None => break,
                                }
                            }
                            let rid = (h & ID_MASK) as usize;
                            if memo_sim_tail[rid] {
                                match st.pop() {
                                    Some(nx) => { h = nx; continue; }
                                    None => break,
                                }
                            }
                            memo_sim_tail[rid] = true;
                            let r = e.base + rid * RULE;
                            let lc = e.a[r];
                            let rc = e.a[r + 1];
                            if (lc >> ID_BITS) < INLINE as u32 {
                                if (rc >> ID_BITS) < INLINE as u32 {
                                    match st.pop() {
                                        Some(nx) => { h = nx; continue; }
                                        None => break,
                                    }
                                }
                                h = rc;
                            } else {
                                st.push(rc);
                                ca.tail_stack_pushes += 1;
                                h = lc;
                            }
                        }
                    }

                    // Actually spell the word to measure exact spelling bytes & Cayley prefix trie!
                    let start = spelling.len();
                    e.append(node_word, &mut stack, &mut spelling);
                    let nw_len = spelling.len() - start;
                    spelling.push(a as u8);
                    let w_vec = spelling[start..].to_vec();
                    if seen_long_word.insert(w_vec.clone()) {
                        ca.long_dict_entries += 1;
                        ca.spelling_bytes += w_vec.len();
                        if let Some(&prev_len) = unique_nw.get(&node_word) {
                            ca.nw_dup_bytes += prev_len;
                        } else {
                            unique_nw.insert(node_word, nw_len);
                        }
                        long_words_bytes.push(w_vec);
                    }
                }
                in_node += width;
                k_pos += width;
                if in_node == node_width {
                    id = next_id;
                    in_node = 0;
                    if id != 0 {
                        (next_id, _, node_width, node_word) = L::band_node(e.a, id);
                    }
                }
            }
        }
        ca.unique_long_nw_used = unique_nw.len();

        // Exact Cayley rooted prefix-trie size (total unique byte-edges = sum of (len - LCP) after sorting!)
        long_words_bytes.sort_unstable();
        let mut trie_bytes = 0usize;
        for (i, w) in long_words_bytes.iter().enumerate() {
            let lcp = if i == 0 {
                0
            } else {
                let prev = &long_words_bytes[i - 1];
                w.iter().zip(prev.iter()).take_while(|(x, y)| x == y).count()
            };
            trie_bytes += w.len() - lcp;
        }
        ca.trie_edge_bytes = trie_bytes;
        ca
    }

    if k.wide(ns) {
        inspect::<Wide, CAT_COMPRESS>(&mut k.edges, ns, n as u32, &f.thin, &first, &mut write, f.m)
    } else {
        inspect::<Narrow, CAT_COMPRESS>(&mut k.edges, ns, n as u32, &f.thin, &first, &mut write, f.m)
    }
}

// ============================================================================
// 4. Corpora, Exactness Verification & Paired 1T/2W/4W Benchmarking
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
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

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    if n == 0 {
        return Vec::new();
    }
    let mut cnt = [0usize; 256];
    for &b in l {
        cnt[usize::from(b)] += 1;
    }
    let mut starts = [0usize; 256];
    let mut sum = 0usize;
    for a in 0..256 {
        starts[a] = sum;
        sum += cnt[a];
    }
    let mut fl = vec![0usize; n];
    for (i, &b) in l.iter().enumerate() {
        let a = usize::from(b);
        fl[starts[a]] = i;
        starts[a] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        x = fl[x];
        out.push(l[x]);
    }
    out
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &rot) in sa.iter().enumerate() {
        if rot == 0 {
            p = i as u32;
        }
        l.push(text[(rot + n - 1) % n]);
    }
    (l, p)
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    let mut r5 = cand_r5_head::Kernel::default();
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let records = records_of(&l);
            let mut expected = Vec::new();
            r5.run_entry(&records, l.len(), p, cand_r5_head::Cap::NONE, &mut expected);
            Block { name: f, n: l.len(), p, records, expected }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool, need_exp: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    let mut r5 = cand_r5_head::Kernel::default();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        if only_full && route_class > 1 {
            continue;
        }
        let name = if let Some(ns) = names {
            if b_idx < ns.len() { ns[b_idx].to_string() } else { format!("blk-{idx}") }
        } else {
            format!("w431-{idx}")
        };
        let mut expected = Vec::new();
        if need_exp {
            r5.run_entry(&records, n, p, cand_r5_head::Cap::NONE, &mut expected);
        }
        blocks.push(Block { name, n, p, records, expected });
    }
    blocks
}

fn resolve_corpus_path(candidates: &[&str]) -> String {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate corpus among {:?}", candidates);
}

fn pct(a: usize, b: usize) -> f64 {
    if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 }
}

fn summarize_cayley_corpus(label: &str, blocks: &[Block], per_block: bool) {
    println!("================================================================================");
    println!("PART 1 — CAYLEY (1859) ROOTED-TREE ANATOMY OVER F-INLINE3: {label} ({} blks)", blocks.len());
    println!("================================================================================");
    let base_stats: Vec<CayleyAnatomy> = blocks
        .iter()
        .map(|b| analyze_cayley_forest::<false>(&b.records, b.n, b.p))
        .collect();
    let cat_stats: Vec<CayleyAnatomy> = blocks
        .iter()
        .map(|b| analyze_cayley_forest::<true>(&b.records, b.n, b.p))
        .collect();

    if per_block {
        println!(
            "{:<13} {:>6} {:>6} {:>6} | {:>5} {:>5} {:>5} {:>5} | {:>5} {:>4} {:>4} | {:>6} {:>6} | {:>6} {:>6}",
            "Block", "openN", "rLong", "live", "LL%", "IL%", "LI%", "II%", "in1%", "avgH", "maxS", "baseSt", "tailSt", "spKiB", "trKiB"
        );
        for (b, s) in blocks.iter().zip(base_stats.iter()) {
            let avg_h = s.sum_root_height as f64 / s.unique_long_roots.max(1) as f64;
            println!(
                "{:<13} {:>6} {:>6} {:>6} | {:>4.1}% {:>4.1}% {:>4.1}% {:>4.1}% | {:>4.1}% {:>4.1} {:>4} | {:>6} {:>6} | {:>5.1}K {:>5.1}K",
                b.name,
                s.open_nodes,
                s.rules_long,
                s.long_live,
                pct(s.node_ll, s.long_live),
                pct(s.node_il, s.long_live),
                pct(s.node_li, s.long_live),
                pct(s.node_ii, s.long_live),
                pct(s.indeg_1, s.long_live),
                avg_h,
                s.max_root_strahler,
                s.base_stack_pushes,
                s.tail_stack_pushes,
                s.spelling_bytes as f64 / 1024.0,
                s.trie_edge_bytes as f64 / 1024.0
            );
        }
    }

    let sum = |v: &[CayleyAnatomy], f: fn(&CayleyAnatomy) -> usize| -> usize { v.iter().map(f).sum() };
    let r_tot = sum(&base_stats, |s| s.rules_total);
    let r_sh = sum(&base_stats, |s| s.rules_short);
    let r_lg = sum(&base_stats, |s| s.rules_long);
    let lg_live = sum(&base_stats, |s| s.long_live);
    let lg_dead = sum(&base_stats, |s| s.long_dead);
    let ll = sum(&base_stats, |s| s.node_ll);
    let il = sum(&base_stats, |s| s.node_il);
    let li = sum(&base_stats, |s| s.node_li);
    let ii = sum(&base_stats, |s| s.node_ii);
    let i1 = sum(&base_stats, |s| s.indeg_1);
    let i2 = sum(&base_stats, |s| s.indeg_2);
    let i3p = sum(&base_stats, |s| s.indeg_3p);
    let cat3 = sum(&base_stats, |s| s.cat_i3_ops);
    let cat15 = sum(&base_stats, |s| s.cat_p15_ops);
    let u_roots = sum(&base_stats, |s| s.unique_long_roots);
    let sum_h = sum(&base_stats, |s| s.sum_root_height);
    let max_h = base_stats.iter().map(|s| s.max_root_height).max().unwrap_or(0);
    let sum_s = sum(&base_stats, |s| s.sum_root_strahler);
    let max_s = base_stats.iter().map(|s| s.max_root_strahler).max().unwrap_or(0);
    let mut shist = [0usize; 6];
    for s in &base_stats {
        for j in 0..6 {
            shist[j] += s.strahler_hist[j];
        }
    }
    let unrolled: u64 = base_stats.iter().map(|s| s.unrolled_tree_nodes).sum();
    let b_push = sum(&base_stats, |s| s.base_stack_pushes);
    let t_push = sum(&base_stats, |s| s.tail_stack_pushes);
    let m_hit = sum(&base_stats, |s| s.append_memo_hits);
    let m_miss = sum(&base_stats, |s| s.append_memo_misses);
    let l_dict = sum(&base_stats, |s| s.long_dict_entries);
    let u_nw = sum(&base_stats, |s| s.unique_long_nw_used);
    let sp_b = sum(&base_stats, |s| s.spelling_bytes);
    let tr_b = sum(&base_stats, |s| s.trie_edge_bytes);
    let dup_b = sum(&base_stats, |s| s.nw_dup_bytes);

    // Compare K-CatI3 effect on live nodes, height, and stack pushes!
    let c_live = sum(&cat_stats, |s| s.long_live);
    let c_sum_h = sum(&cat_stats, |s| s.sum_root_height);
    let c_max_h = cat_stats.iter().map(|s| s.max_root_height).max().unwrap_or(0);
    let c_b_push = sum(&cat_stats, |s| s.base_stack_pushes);
    let c_t_push = sum(&cat_stats, |s| s.tail_stack_pushes);
    let c_miss = sum(&cat_stats, |s| s.append_memo_misses);

    println!("\n  [AGGREGATE CAYLEY ROOTED-TREE ANATOMY — {label}]:");
    println!(
        "    S2b Rules: total={r_tot} | short_leaf(g=4..15)={r_sh} ({:.1}%) | long_internal(g>=16)={r_lg} ({:.1}%: live={lg_live}, dead={lg_dead} [{:.1}%])",
        pct(r_sh, r_tot), pct(r_lg, r_tot), pct(lg_dead, r_lg)
    );
    println!(
        "    Surviving side-0 roots: zero={} ({:.1}%), inline3={} ({:.1}%), short4..15={} ({:.1}%), long={} ({:.1}%, unique={u_roots})",
        sum(&base_stats, |s| s.root_zero),
        pct(sum(&base_stats, |s| s.root_zero), sum(&base_stats, |s| s.surv_nodes)),
        sum(&base_stats, |s| s.root_inline3),
        pct(sum(&base_stats, |s| s.root_inline3), sum(&base_stats, |s| s.surv_nodes)),
        sum(&base_stats, |s| s.root_short4_15),
        pct(sum(&base_stats, |s| s.root_short4_15), sum(&base_stats, |s| s.surv_nodes)),
        sum(&base_stats, |s| s.root_long),
        pct(sum(&base_stats, |s| s.root_long), sum(&base_stats, |s| s.surv_nodes))
    );
    println!(
        "    Cayley Arity (live g>=16): LL(leaf,leaf)={:.2}% | IL(long,leaf)={:.2}% | LI(leaf,long)={:.2}% | II(long,long)={:.2}% | Unary caterpillar (IL+LI)={:.2}%",
        pct(ll, lg_live), pct(il, lg_live), pct(li, lg_live), pct(ii, lg_live), pct(il + li, lg_live)
    );
    println!(
        "    DAG In-Degree (live g>=16): indeg==1 (pure tree!)={:.2}% | indeg==2={:.2}% | indeg>=3={:.2}% | Unrolled/DAG={:.2}x",
        pct(i1, lg_live), pct(i2, lg_live), pct(i3p, lg_live), unrolled as f64 / lg_live.max(1) as f64
    );
    println!(
        "    Height & Strahler-Horton S(T) (unique long roots={u_roots}): mean_H={:.2}, max_H={max_h} | mean_S={:.2}, max_S={max_s} | S hist: [S2={:.1}%, S3={:.1}%, S4={:.1}%, S5+={:.1}%]",
        sum_h as f64 / u_roots.max(1) as f64,
        sum_s as f64 / u_roots.max(1) as f64,
        pct(shist[2], u_roots), pct(shist[3], u_roots), pct(shist[4], u_roots), pct(shist[5], u_roots)
    );
    println!(
        "    S3a append() Traversal: memo_hits={m_hit} ({:.2}% hit rate vs {m_miss} misses) | Base stack pushes={b_push} -> TailUnwind={t_push} ({:+.1}%!)",
        pct(m_hit, m_hit + m_miss),
        100.0 * (t_push as f64 - b_push as f64) / b_push.max(1) as f64
    );
    println!(
        "    Cayley Caterpillar Collapse (`K-CatI3`): eligible(I3={:.1}%, P15={:.1}% of live) | live_g16: {lg_live}->{c_live} ({:+.1}%), mean_H: {:.2}->{:.2} (max {max_h}->{c_max_h}), misses: {m_miss}->{c_miss} ({:+.1}%), pushes: {b_push}->{c_b_push}(base)/{c_t_push}(tail, {:+.1}%)",
        pct(cat3, lg_live), pct(cat15, lg_live),
        100.0 * (c_live as f64 - lg_live as f64) / lg_live.max(1) as f64,
        sum_h as f64 / u_roots.max(1) as f64, c_sum_h as f64 / u_roots.max(1) as f64,
        100.0 * (c_miss as f64 - m_miss as f64) / m_miss.max(1) as f64,
        100.0 * (c_t_push as f64 - b_push as f64) / b_push.max(1) as f64
    );
    println!(
        "    S3a `spelling` Shared-Prefix Anatomy: long_entries={l_dict}, unique_nw={u_nw} (mult={:.2}x) | flat={:.1} KiB | same-nw dup={:.1} KiB ({:.1}%) | Cayley prefix-trie={:.1} KiB ({:.1}% of flat, -{:.1}% prefix redundancy!)",
        l_dict as f64 / u_nw.max(1) as f64,
        sp_b as f64 / 1024.0,
        dup_b as f64 / 1024.0, pct(dup_b, sp_b),
        tr_b as f64 / 1024.0, pct(tr_b, sp_b), 100.0 - pct(tr_b, sp_b)
    );
    println!();
}

// ============================================================================
// 5. Exactness Gate + WS Audit + Paired 1T/2W/4W Benchmarking
// ============================================================================

const ARMS: [&str; 6] = [
    "R5-Head",
    "F-Inline3",
    "I3-AA",
    "K-TailUnw",
    "K-CatI3",
    "K-Cat+Tail",
];

fn run_arm<const TIMED: bool>(
    arm: usize,
    r5: &mut cand_r5_head::Kernel,
    kk: &mut KKernel,
    blk: &Block,
    out: &mut Vec<u8>,
) -> StageTimes {
    match arm {
        0 => {
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, out);
            let mut tm = StageTimes::default();
            if let Some(t) = t0 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            tm
        }
        1 | 2 => kk.run::<false, false, TIMED>(&blk.records, blk.n, blk.p, out),
        3 => kk.run::<false, true, TIMED>(&blk.records, blk.n, blk.p, out),
        4 => kk.run::<true, false, TIMED>(&blk.records, blk.n, blk.p, out),
        _ => kk.run::<true, true, TIMED>(&blk.records, blk.n, blk.p, out),
    }
}

fn run_exactness_suite(wiki6: &[Block], nonwiki6: &[Block], code4: &[Block], wiki233: &[Block]) {
    println!("================================================================================");
    println!("PART 2 — EXHAUSTIVE EXACTNESS GATE (5,590 small + 280 malformed + 249 real blks)");
    println!("================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut kk = KKernel::default();
    let mut out_r5 = Vec::new();
    let mut out_v = Vec::new();

    let mut check_all = |l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let exp = forward_fl_invert(l, p);
        r5.run_entry(&recs, l.len(), p, cand_r5_head::Cap::NONE, &mut out_r5);
        assert_eq!(out_r5, exp, "R5 mismatch on {tag}");
        kk.run::<false, false, false>(&recs, l.len(), p, &mut out_v);
        assert_eq!(out_v, exp, "F-Inline3 mismatch on {tag}");
        kk.run::<false, true, false>(&recs, l.len(), p, &mut out_v);
        assert_eq!(out_v, exp, "K-TailUnw mismatch on {tag}");
        kk.run::<true, false, false>(&recs, l.len(), p, &mut out_v);
        assert_eq!(out_v, exp, "K-CatI3 mismatch on {tag}");
        kk.run::<true, true, false>(&recs, l.len(), p, &mut out_v);
        assert_eq!(out_v, exp, "K-Cat+Tail mismatch on {tag}");
    };

    // 1. 5,590 small valid BWTs: binary n=1..=11 (4,094) + ternary n=1..=6 (1,092) + 404 random (n=20..=200)
    let mut small_cnt = 0usize;
    for n in 1..=11usize {
        for mask in 0..(1u32 << n) {
            let mut text = Vec::with_capacity(n);
            for bit in 0..n {
                text.push(if ((mask >> bit) & 1) == 0 { b'a' } else { b'b' });
            }
            let (l, p) = bwt_small(&text);
            check_all(&l, p, "small-bin");
            small_cnt += 1;
        }
    }
    for n in 1..=6usize {
        let total = 3usize.pow(n as u32);
        for mut code in 0..total {
            let mut text = Vec::with_capacity(n);
            for _ in 0..n {
                text.push(b'a' + (code % 3) as u8);
                code /= 3;
            }
            let (l, p) = bwt_small(&text);
            check_all(&l, p, "small-ter");
            small_cnt += 1;
        }
    }
    let mut rng = 0xC49_CA1E_7777u64;
    let next_u64 = |s: &mut u64| {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s
    };
    for _ in 0..404 {
        let n = 40 + (next_u64(&mut rng) as usize % 300);
        let alpha = 2 + (next_u64(&mut rng) as usize % 8);
        let mut text = Vec::with_capacity(n);
        for _ in 0..n {
            text.push(b'a' + (next_u64(&mut rng) as usize % alpha) as u8);
        }
        let (l, p) = bwt_small(&text);
        check_all(&l, p, "small-rand");
        small_cnt += 1;
    }

    // 2. 280 malformed multi-cycle columns (incl. thick cycles, 0x00/0x7F/0xFF, deep caterpillar chains!)
    let mut mal_cnt = 0usize;
    let alphabets: [&[u8]; 3] = [
        &[0x00, 0x01, 0x7F, 0x80, 0xFF],
        &[b'a', b'b', b'c', b'd', b'e', b'f'],
        &[0x00, b'A', b'z', 0xFE, 0xFF],
    ];
    for trial in 0..260usize {
        let alph = alphabets[trial % alphabets.len()];
        let sigma = 2 + (next_u64(&mut rng) as usize % (alph.len() - 1));
        let n_runs = 8 + (next_u64(&mut rng) as usize % 30);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = alph[next_u64(&mut rng) as usize % sigma];
            let w = match next_u64(&mut rng) % 4 {
                0 => 20,
                1 => 40,
                2 => 1 + (next_u64(&mut rng) as usize % 5),
                _ => 17 + (next_u64(&mut rng) as usize % 30),
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("mal-{trial}"));
        mal_cnt += 1;
    }
    // 20 deep-caterpillar + SATURATED=255 spiral stress tests
    for &slen in &[253usize, 254, 255, 256, 257, 512] {
        let mut sp = vec![0x7Fu8; slen * 20];
        sp[0] = 0x00;
        sp[slen * 20 - 1] = 0xFF;
        check_all(&sp, 0, &format!("spiral-{slen}"));
        mal_cnt += 1;
    }
    for k_c in 0..14usize {
        // Long chain of 25+ distinct thick strips inducing deep unary caterpillars in S2b
        let mut l = Vec::new();
        for c in 0..(20 + k_c) {
            let b = (c as u8).wrapping_mul(7).wrapping_add(1);
            l.resize(l.len() + 18 + (c % 7), b);
            l.push(b.wrapping_add(1));
        }
        check_all(&l, (k_c * 13 % l.len()) as u32, &format!("cat-stress-{k_c}"));
        mal_cnt += 1;
    }

    // 3. Forced fallbacks (force_wide, force_spirals) on wiki6[0]
    for (fw, fs) in [(true, None), (false, Some(true)), (false, Some(false))] {
        kk.force_wide = fw;
        kk.force_spirals = fs;
        kk.run::<true, true, false>(&wiki6[0].records, wiki6[0].n, wiki6[0].p, &mut out_v);
        assert_eq!(out_v, wiki6[0].expected, "fallback mismatch!");
    }
    kk.force_wide = false;
    kk.force_spirals = None;

    // 4. All 6 Admitted + 6 JSONL/Syslog + 4 Code/Docs + 233 Full wikidata.bz2
    for blk in wiki6.iter().chain(nonwiki6.iter()).chain(code4.iter()) {
        for arm in 1..ARMS.len() {
            run_arm::<false>(arm, &mut r5, &mut kk, blk, &mut out_v);
            assert_eq!(out_v, blk.expected, "{} mismatch on {}", ARMS[arm], blk.name);
        }
    }
    for blk in wiki233 {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_r5);
        for arm in [1usize, 3, 4, 5] {
            run_arm::<false>(arm, &mut r5, &mut kk, blk, &mut out_v);
            assert_eq!(out_v, out_r5, "{} mismatch on {}", ARMS[arm], blk.name);
        }
    }

    println!(
        "  [PASS] {small_cnt} small valid + {mal_cnt} malformed/spiral/caterpillar + force_wide/spirals + 6/6 Admitted + 6/6 Non-Wiki + 4/4 Code/Docs + {}/{} Full Wikidata 100% BYTE-EXACT!\n",
        wiki233.len(),
        wiki233.len()
    );
}

fn audit_ws(blocks: &[Block], label: &str) {
    println!("--- Steady-State Persistent WS Audit — {label} ({} blks) ---", blocks.len());
    let mut i3_ws = 0usize;
    for arm in 1..ARMS.len() {
        let mut r5 = cand_r5_head::Kernel::default();
        let mut kk = KKernel::default();
        let mut out = Vec::new();
        for blk in blocks {
            run_arm::<false>(arm, &mut r5, &mut kk, blk, &mut out);
        }
        let ws = kk.ws_bytes();
        if arm == 1 {
            i3_ws = ws;
        }
        println!(
            "  {:<11}: {:>8.2} KiB (dWS vs F-Inline3 = {:>+7.2} KiB)",
            ARMS[arm],
            ws as f64 / 1024.0,
            (ws as isize - i3_ws as isize) as f64 / 1024.0
        );
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!("\n================================================================================");
    println!("PART 3A — 1T STAGE BREAKDOWN (PAIRED {rounds} ROUNDS, p50 us, SUM) — {label}");
    println!("================================================================================");
    let na = ARMS.len();
    let mut s1 = vec![0.0; na];
    let mut s2a = vec![0.0; na];
    let mut s2b = vec![0.0; na];
    let mut s3a = vec![0.0; na];
    let mut s4 = vec![0.0; na];
    let mut s5 = vec![0.0; na];
    let mut ind = vec![0.0; na];
    let mut tot = vec![0.0; na];
    let mut r5 = cand_r5_head::Kernel::default();
    let mut kks: Vec<KKernel> = (0..na).map(|_| KKernel::default()).collect();
    let mut out = Vec::new();
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };

    for blk in blocks {
        for arm in 0..na {
            run_arm::<false>(arm, &mut r5, &mut kks[arm], blk, &mut out);
        }
        let mut vs1 = vec![Vec::with_capacity(rounds); na];
        let mut vs2a = vec![Vec::with_capacity(rounds); na];
        let mut vs2b = vec![Vec::with_capacity(rounds); na];
        let mut vs3a = vec![Vec::with_capacity(rounds); na];
        let mut vs4 = vec![Vec::with_capacity(rounds); na];
        let mut vs5 = vec![Vec::with_capacity(rounds); na];
        let mut vind = vec![Vec::with_capacity(rounds); na];
        let mut vtot = vec![Vec::with_capacity(rounds); na];

        for _ in 0..rounds {
            for arm in 0..na {
                if arm == 0 {
                    let t0 = Instant::now();
                    r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
                    vtot[0].push(t0.elapsed().as_secs_f64() * 1e6);
                } else {
                    let tm = run_arm::<true>(arm, &mut r5, &mut kks[arm], blk, &mut out);
                    vs1[arm].push(tm.s1_us);
                    vs2a[arm].push(tm.s2a_us);
                    vs2b[arm].push(tm.s2b_us);
                    vs3a[arm].push(tm.s3a_us);
                    vs4[arm].push(tm.s4_us);
                    vs5[arm].push(tm.s5_us);
                    vind[arm].push(tm.induced_us());
                }
            }
            for arm in 1..na {
                let t0 = Instant::now();
                run_arm::<false>(arm, &mut r5, &mut kks[arm], blk, &mut out);
                vtot[arm].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        tot[0] += p50(&mut vtot[0]);
        for arm in 1..na {
            s1[arm] += p50(&mut vs1[arm]);
            s2a[arm] += p50(&mut vs2a[arm]);
            s2b[arm] += p50(&mut vs2b[arm]);
            s3a[arm] += p50(&mut vs3a[arm]);
            s4[arm] += p50(&mut vs4[arm]);
            s5[arm] += p50(&mut vs5[arm]);
            ind[arm] += p50(&mut vind[arm]);
            tot[arm] += p50(&mut vtot[arm]);
        }
    }

    println!(
        "{:<11} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} {:>7} {:>7} | {:>8} {:>7} {:>7}",
        "Variant", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Ind(us)", "vsI3", "S2b/I3", "S3a/I3", "Tot(us)", "vsR5H", "vsI3"
    );
    println!(
        "{:<11} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} {:>7} {:>7} | {:>8.1}  1.000x  {:.3}x",
        ARMS[0], "-", "-", "-", "-", "-", "-", "-", "-", "-", "-", tot[0], tot[0] / tot[1]
    );
    for arm in 1..na {
        println!(
            "{:<11} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1}  {:.3}x  {:.3}x  {:.3}x | {:>8.1}  {:.3}x  {:.3}x",
            ARMS[arm],
            s1[arm], s2a[arm], s2b[arm], s3a[arm], s4[arm], s5[arm],
            ind[arm], ind[arm] / ind[1], s2b[arm] / s2b[1], s3a[arm] / s3a[1],
            tot[arm], tot[arm] / tot[0], tot[arm] / tot[1]
        );
    }
}

fn bench_multiworker(
    blocks: &[Block],
    label: &str,
    rounds: usize,
    iters_1w: usize,
    iters_mw: usize,
) {
    println!(
        "\n--- PART 3B: Multi-Worker (1T / 2W / 4W, paired {rounds} rounds, p50 ms [vsR5H/vsI3]) — {label} ---"
    );
    let warm_n = blocks.len().min(12);
    for &workers in &[1usize, 2, 4] {
        let mut res: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); ARMS.len()];
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for arm in 0..ARMS.len() {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut kk = KKernel::default();
                            let mut out = Vec::new();
                            for blk in &blocks[..warm_n] {
                                run_arm::<false>(arm, &mut r5, &mut kk, blk, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_arm::<false>(arm, &mut r5, &mut kk, blk, &mut out);
                                    std::hint::black_box(&out);
                                }
                            }
                            t0.elapsed().as_secs_f64() * 1e3 / iters as f64
                        }));
                    }
                    let mut max_ms = 0.0f64;
                    for h in handles {
                        max_ms = max_ms.max(h.join().unwrap());
                    }
                    max_ms
                });
                res[arm].push(t_wall);
            }
        }
        let p50_s = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        print!("  {workers}W: ");
        for arm in 0..ARMS.len() {
            let m = p50_s(&res[arm]);
            if arm == 0 {
                print!("{}={:.2}ms ", ARMS[arm], m);
            } else {
                let mut pr_r5: Vec<f64> = (0..rounds).map(|r| res[arm][r] / res[0][r]).collect();
                let mut pr_i3: Vec<f64> = (0..rounds).map(|r| res[arm][r] / res[1][r]).collect();
                pr_r5.sort_by(|a, b| a.total_cmp(b));
                pr_i3.sort_by(|a, b| a.total_cmp(b));
                print!(
                    "| {}={:.2}ms({:.3}x/{:.3}x) ",
                    ARMS[arm], m, pr_r5[rounds / 2], pr_i3[rounds / 2]
                );
            }
        }
        println!();
    }
}

fn main() {
    let t0 = Instant::now();
    let wiki6 = load_six_admitted();
    let nonwiki_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let nonwiki_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let code_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let wiki431_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false, true);
    let code_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
    let code4 = load_bin_blocks(&code_path, Some(&code_names), false, true);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true, false);

    // Part 1: Full Cayley (1859) Rooted-Tree Anatomy across all 4 corpora
    summarize_cayley_corpus("6 Admitted Wikidata", &wiki6, true);
    summarize_cayley_corpus("6 Real Non-Wikidata (JSONL + Syslog)", &nonwiki6, true);
    summarize_cayley_corpus("4 Real Code & Docs", &code4, true);
    summarize_cayley_corpus("233 Full wikidata.bz2", &wiki233, false);

    // Part 2: Exhaustive exactness gate
    run_exactness_suite(&wiki6, &nonwiki6, &code4, &wiki233);

    // Part 3: Working set + 1T stage breakdown + paired 1T/2W/4W multi-worker
    audit_ws(&wiki6, "6 Admitted Wikidata");
    audit_ws(&nonwiki6, "6 JSONL/Syslog");
    audit_ws(&code4, "4 Code/Docs");
    audit_ws(&wiki233, "233 Full Wikidata");

    bench_stage_1t(&wiki6, "6 Admitted Wikidata", 15);
    bench_stage_1t(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)", 15);
    bench_stage_1t(&code4, "4 Real Code & Docs", 9);

    bench_multiworker(&wiki6, "6 Admitted Wikidata", 9, 4, 4);
    bench_multiworker(&nonwiki6, "6 Real Non-Wikidata (JSONL + Syslog)", 9, 6, 6);
    bench_multiworker(&code4, "4 Real Code & Docs", 5, 2, 2);
    bench_multiworker(&wiki233, "233 Full wikidata.bz2", 5, 1, 1);

    println!("\nTotal runtime: {:.2}s", t0.elapsed().as_secs_f64());
}
