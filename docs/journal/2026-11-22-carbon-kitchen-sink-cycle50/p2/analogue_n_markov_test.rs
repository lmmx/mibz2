#![forbid(unsafe_code)]

//! Cycle 50 P2 — Analogue N, Experimental Test N2:
//! Testing High-Partial-Quotient (`a_i >= 2` / `a_i >= 3` Markov-Threshold) 2-Band
//! Periodic/Christoffel Cycle Contraction inside `Open<16>` `S2b` (`F-Inline3 + K-TailUnw`).
//!
//! Primary Sources:
//!   - A. A. Markov, "Sur les formes quadratiques binaires indéfinies", Math. Ann. 15 (1879) & 17 (1880)
//!   - E. B. Christoffel (1875), Euclid (c. 300 BCE), É. Lucas (1891)
//!
//! Evaluates 5 arms on:
//!   - Exactness Gate: 5,590 small valid + 280 malformed multi-cycle/spiral forward-FL + 5 Analogue N stress + 249 real blocks
//!   - Paired 1T / 2W / 4W Benchmarks + Stage Breakdown (S1..S5) + Persistent Working-Set Audit:
//!     * 5 Synthetic Solved Analogue N Stress Blocks (Lucas/Markov-zone vs High-PQ `widest <= 32,768`)
//!     * 6 Admitted Wikidata blocks
//!     * 10 Non-Wikidata blocks (6 JSONL/Syslog + 4 Code/Docs)
//!     * 233 Full Wikidata blocks
//!
//! Compiles cleanly with:
//! `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

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

// F-Inline3 handle encoding (ID_BITS = 24, inline g=1..3, SATURATED = 255)
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
// 1. Links Trait & Elim Engine (F-Inline3 + K-TailUnw + N2 2-Band Contraction Modes)
// ============================================================================

/// N2 S2b 2-Band Cycle Contraction Modes:
///   MODE = 0: Baseline F-Inline3 + K-TailUnw (`eliminate_narrow_plain` if widest <= 32,768)
///   MODE = 1: N2a-NarSpiral2 (fused Narrow 2-band spiral contraction for any q >= 2)
///   MODE = 2: N2b-MarkovHiQ3 (Markov-thresholded fused Narrow 2-band spiral for q >= 3, div-free fast guard)
///   MODE = 3: N2c-2BandLoop  (Markov-thresholded q >= 3 + inner 2-band Euclid register loop)
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

struct Elim<'a, L: Links, const MODE: u8> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const MODE: u8> Elim<'_, L, MODE> {
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
    fn join(&mut self, left: u32, right: u32) -> u32 {
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

    /// Fused Narrow 2-band high-PQ Christoffel/Markov spiral elimination:
    ///   - MIN_Q = 2 (N2a-NarSpiral2): triggers whenever q = (big-1)/small >= 2
    ///   - MIN_Q = 3 (N2b-MarkovHiQ3 / N2c-2BandLoop): triggers ONLY above Markov's {1,2} threshold (q >= 3),
    ///     using a division-free fast guard (`aw > 3*bw` or `bw > 3*aw`) on the hot path!
    ///   - INNER_LOOP = true (N2c-2BandLoop): stays inside an in-register 2-band Euclid loop while
    ///     `(x, y)` remain mutually adjacent (avoids sentinel reloads & linked-list writes across CF steps!).
    #[inline(never)]
    fn eliminate_narrow_markov<const MIN_Q: u32, const INNER_LOOP: bool>(
        &mut self,
        remaining: &mut u32,
    ) -> bool {
        let mut left = *remaining;
        let room = SPIRAL_RULES * RULE;
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let sent = &self.a[..2];
            let x = sent[0] & 0xFFFF;
            let y = sent[1] & 0xFFFF;
            let xi = x as usize * 4;
            let yi = y as usize * 4;
            let nx = &self.a[xi..xi + 4];
            let (x0, x1, mut aw, mut wx) = (nx[0], nx[1], nx[2], nx[3]);
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
            let (y0, y1, mut bw, mut wy) = (ny[0], ny[1], ny[2], ny[3]);

            // Fast division-free Markov high-PQ check:
            // q = (aw - 1) / bw >= MIN_Q  <=>  aw > MIN_Q * bw
            let thresh_x = if MIN_Q == 3 {
                bw.saturating_mul(3)
            } else {
                bw.saturating_mul(2)
            };
            let thresh_y = if MIN_Q == 3 {
                aw.saturating_mul(3)
            } else {
                aw.saturating_mul(2)
            };

            if aw > thresh_x && (y1 & 0xFFFF) == x {
                let q = (aw - 1) / bw;
                let head = self.power(wx, q);
                wy = self.join(head, wy);
                aw -= q * bw;
                left -= q * bw;
                if INNER_LOOP && (x0 & 0xFFFF) == y {
                    // Mutual 2-band adjacency! Stay in register Euclid loop while room allows.
                    while self.top + room <= self.a.len() {
                        if aw > bw {
                            let q2 = (aw - 1) / bw;
                            if q2 == 0 {
                                break;
                            }
                            let h2 = if q2 == 1 { wx } else { self.power(wx, q2) };
                            wy = self.join(h2, wy);
                            aw -= q2 * bw;
                            left -= q2 * bw;
                        } else if bw > aw {
                            let q2 = (bw - 1) / aw;
                            if q2 == 0 {
                                break;
                            }
                            let t2 = if q2 == 1 { wy } else { self.power(wy, q2) };
                            wx = self.join(wx, t2);
                            bw -= q2 * aw;
                            left -= q2 * aw;
                        } else {
                            break;
                        }
                    }
                    self.a[xi + 2] = aw;
                    self.a[xi + 3] = wx;
                    self.a[yi + 2] = bw;
                    self.a[yi + 3] = wy;
                } else {
                    self.a[xi + 2] = aw;
                    self.a[yi + 3] = wy;
                }
                continue;
            }
            if bw > thresh_y && (x0 & 0xFFFF) == y {
                let q = (bw - 1) / aw;
                let tail = self.power(wy, q);
                wx = self.join(wx, tail);
                bw -= q * aw;
                left -= q * aw;
                if INNER_LOOP && (y1 & 0xFFFF) == x {
                    while self.top + room <= self.a.len() {
                        if bw > aw {
                            let q2 = (bw - 1) / aw;
                            if q2 == 0 {
                                break;
                            }
                            let t2 = if q2 == 1 { wy } else { self.power(wy, q2) };
                            wx = self.join(wx, t2);
                            bw -= q2 * aw;
                            left -= q2 * aw;
                        } else if aw > bw {
                            let q2 = (aw - 1) / bw;
                            if q2 == 0 {
                                break;
                            }
                            let h2 = if q2 == 1 { wx } else { self.power(wx, q2) };
                            wy = self.join(h2, wy);
                            aw -= q2 * bw;
                            left -= q2 * bw;
                        } else {
                            break;
                        }
                    }
                    self.a[xi + 2] = aw;
                    self.a[xi + 3] = wx;
                    self.a[yi + 2] = bw;
                    self.a[yi + 3] = wy;
                } else {
                    self.a[yi + 2] = bw;
                    self.a[xi + 3] = wx;
                }
                continue;
            }

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
        if L::IS_NARROW {
            if MODE == 1 {
                return self.eliminate_narrow_markov::<2, false>(remaining);
            } else if MODE == 2 {
                return self.eliminate_narrow_markov::<3, false>(remaining);
            } else if MODE == 3 {
                return self.eliminate_narrow_markov::<3, true>(remaining);
            } else if !SPIRALS {
                return self.eliminate_narrow_plain(remaining);
            }
        }
        let mut left = *remaining;
        let room = if SPIRALS || MODE > 0 { SPIRAL_RULES * RULE } else { RULE };
        let use_sp = SPIRALS || MODE > 0;
        let min_q = if MODE >= 2 { 3u32 } else { 2u32 };
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + room > self.a.len() {
                break false;
            }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if use_sp && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= min_q {
                    let q = (aw - 1) / bw;
                    assert!(q * bw < aw && q * bw <= left);
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= min_q {
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

    /// K-TailUnw (CS-16) register-spine tail-call unwind in S3a::append.
    #[inline(always)]
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
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
    fn code<L: Links, const MODE: u8>(
        &mut self,
        e: &mut Elim<L, MODE>,
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

    fn code_miss<L: Links, const MODE: u8>(
        &mut self,
        e: &mut Elim<L, MODE>,
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
// 2. Full Open<16> Kernel Wrapper (F-Inline3 + K-TailUnw + N2 Modes)
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
    fn total_us(&self) -> f64 {
        self.s1_us + self.s2a_us + self.s2b_us + self.s3a_us + self.s4_us + self.s5_us
    }
}

#[derive(Default)]
struct N2Kernel {
    spill: outq::Spill,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    force_wide: bool,
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

impl N2Kernel {
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

    fn run<const MODE: u8, const TIMED: bool>(
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
                if width == 0 {
                    continue;
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
            self.induced::<Wide, MODE, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        } else {
            self.induced::<Narrow, MODE, TIMED>(ns, n as u32, &f.thin, &first, &mut write, m as u32, &mut tm)
        };

        // S4: Wiring
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

        // S5: Emit (forward-FL exact on valid & malformed!)
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
    fn induced<L: Links, const MODE: u8, const TIMED: bool>(
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
        let mut e = Elim::<L, MODE> {
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
        let spirals = widest > CERTIFIED_WIDTH;
        let grow_extra = if spirals || MODE > 0 { SPIRAL_RULES * RULE } else { RULE };
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(&mut self.edges, 4 * ns + top + grow_extra);
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
        let mut e = Elim::<L, MODE> {
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
                        let (entry, len) =
                            e.inline_word(node_word, a as u8, &mut self.stack, &mut self.spelling);
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
// 3. BWT Helpers, Corpora Loaders & Synthetic Solved Analogue N Stress Blocks
// ============================================================================

#[derive(Clone)]
struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let b = l[i];
        let mut w = 1usize;
        while i + w < l.len() && l[i + w] == b && w < 127 {
            w += 1;
        }
        recs.push((u16::from(b) << 8) | (w as u16));
        i += w;
    }
    recs
}

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[usize::from(b)] += 1;
    }
    let mut st = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        st[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = st[usize::from(b)] as usize;
        st[usize::from(b)] += 1;
        fl[s] = ((i as u32) << 8) | u32::from(b);
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for slot in &mut out {
        let v = fl[x];
        *slot = v as u8;
        x = (v >> 8) as usize;
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

fn make_christoffel_block(name: &str, p_cnt: usize, q_cnt: usize) -> Block {
    let mut l = vec![b'b'; q_cnt];
    l.extend(std::iter::repeat_n(b'a', p_cnt));
    let n = l.len();
    let records = records_of(&l);
    let expected = forward_fl_invert(&l, 0);
    Block { name: name.to_string(), n, p: 0, records, expected }
}

/// Build a multi-block synthetic high-PQ stress block whose BWT packs 30 independent
/// high-PQ 2-band periodic/Christoffel components (all with widest <= 32,768 so Base uses plain S2b!).
fn make_multi_2band_hipq_block(name: &str) -> Block {
    let mut l = Vec::new();
    // 15 disjoint byte pairs (b0, b1) each forming a high-PQ 2-run Christoffel component (15,007 / 15,000 -> q=2142!)
    for pair in 0..15u8 {
        let a = b'A' + 2 * pair;
        let b = b'A' + 2 * pair + 1;
        l.extend(std::iter::repeat_n(b, 15_000));
        l.extend(std::iter::repeat_n(a, 15_007));
    }
    let n = l.len();
    let records = records_of(&l);
    let expected = forward_fl_invert(&l, 0);
    Block { name: name.to_string(), n, p: 0, records, expected }
}

fn resolve_corpus_path(candidates: &[&str]) -> String {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    panic!("Could not locate corpus among {:?}", candidates);
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

// ============================================================================
// 4. Exhaustive Exactness Gate + Benchmarks (1T / 2W / 4W)
// ============================================================================

const ARMS: [&str; 5] = [
    "Base-I3+Tail",
    "Base-AA",
    "N2a-NarSpiral2",
    "N2b-MarkovHiQ3",
    "N2c-2BandLoop",
];

fn run_arm<const TIMED: bool>(arm: usize, kk: &mut N2Kernel, blk: &Block, out: &mut Vec<u8>) -> StageTimes {
    match arm {
        0 | 1 => kk.run::<0, TIMED>(&blk.records, blk.n, blk.p, out),
        2 => kk.run::<1, TIMED>(&blk.records, blk.n, blk.p, out),
        3 => kk.run::<2, TIMED>(&blk.records, blk.n, blk.p, out),
        _ => kk.run::<3, TIMED>(&blk.records, blk.n, blk.p, out),
    }
}

fn run_exactness_suite(
    stress: &[Block],
    adm6: &[Block],
    nw6: &[Block],
    cd4: &[Block],
    wiki233: &[Block],
) {
    println!("================================================================================");
    println!("PART 1 — EXHAUSTIVE EXACTNESS GATE (5,590 small + 280 malformed + 5 stress + 249 real)");
    println!("================================================================================");
    let mut kk = N2Kernel::default();
    let mut out = Vec::new();

    let mut check_all = |l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let exp = forward_fl_invert(l, p);
        for &(arm, name) in &[(0usize, "Base"), (2, "N2a"), (3, "N2b"), (4, "N2c")] {
            run_arm::<false>(
                arm,
                &mut kk,
                &Block { name: String::new(), n: l.len(), p, records: recs.clone(), expected: Vec::new() },
                &mut out,
            );
            assert_eq!(out, exp, "{name} mismatch on {tag}");
        }
    };

    // 1. 5,590 small valid BWTs
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
    let mut rng = 0xC50_A1A2_BEEFu64;
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

    // 2. 280 malformed multi-cycle & high-PQ spiral forward-FL columns (incl. Wide & 0x00/0xFF)
    let mut mal_cnt = 0usize;
    let alphabets: [&[u8]; 3] = [
        &[0x00, 0x01, 0x7F, 0x80, 0xFF],
        &[b'a', b'b', b'c', b'd', b'e', b'f'],
        &[0x00, b'A', b'z', 0xFE, 0xFF],
    ];
    for case in 0..240usize {
        let alpha = alphabets[case % 3];
        let n = 35 + (next_u64(&mut rng) as usize % 800);
        let mut l = Vec::with_capacity(n);
        while l.len() < n {
            let b = alpha[next_u64(&mut rng) as usize % alpha.len()];
            let run = if (next_u64(&mut rng) & 3) == 0 {
                18 + (next_u64(&mut rng) as usize % 60)
            } else {
                1 + (next_u64(&mut rng) as usize % 8)
            };
            let take = run.min(n - l.len());
            l.extend(std::iter::repeat_n(b, take));
        }
        let p = (next_u64(&mut rng) as usize % n) as u32;
        check_all(&l, p, "malformed-rand");
        mal_cnt += 1;
    }
    // 40 targeted high-PQ 2-band malformed & periodic Christoffel columns (gcd > 1, multi-cycle, large q)
    for k in 0..40usize {
        let q_mult = 2 + (k % 7);
        let a_cnt = 20 * q_mult + (k % 5) * 3;
        let b_cnt = 20 * q_mult;
        let mut l = vec![b'b'; b_cnt];
        l.extend(std::iter::repeat_n(b'a', a_cnt));
        // Interleave a second detached cycle on 0x00 / 0xFF for half of cases
        if k % 2 == 1 {
            l.extend(std::iter::repeat_n(0xFFu8, 40));
            l.extend(std::iter::repeat_n(0x00u8, 120));
        }
        let p = (k * 17) as u32 % (l.len() as u32);
        check_all(&l, p, "malformed-2band-spiral");
        mal_cnt += 1;
    }

    // Also test force_wide = true on high-PQ stress block to exercise Wide path
    kk.force_wide = true;
    for blk in stress {
        for &arm in &[0usize, 2, 3, 4] {
            run_arm::<false>(arm, &mut kk, blk, &mut out);
            assert_eq!(out, blk.expected, "Wide mismatch on {}", blk.name);
        }
    }
    kk.force_wide = false;

    // 3. All 5 synthetic stress + 249 real blocks
    for (tag, list) in [
        ("Stress-5", stress),
        ("Adm-6", adm6),
        ("NonWiki-6", nw6),
        ("CodeDocs-4", cd4),
        ("Wiki-233", wiki233),
    ] {
        for blk in list {
            for &arm in &[0usize, 2, 3, 4] {
                run_arm::<false>(arm, &mut kk, blk, &mut out);
                assert_eq!(out, blk.expected, "{tag} arm {arm} mismatch on {}", blk.name);
            }
        }
    }

    println!(
        "  [PASS] 100% BYTE-EXACT across ALL 4 implementations (Narrow & Wide, strict forward-FL):"
    );
    println!(
        "         {small_cnt} small valid + {mal_cnt} malformed/spiral + {} stress + 249 real blocks (0 mismatches)!\n",
        stress.len()
    );
}

fn bench_1t_stages(label: &str, blocks: &[Block], rounds: usize) {
    let mut totals = [0.0f64; 5];
    let mut s2b_tot = [0.0f64; 5];
    let mut s3a_tot = [0.0f64; 5];
    let mut ws_max = [0usize; 5];
    let mut kernels: Vec<N2Kernel> = (0..5).map(|_| N2Kernel::default()).collect();
    let mut out = Vec::new();

    // Warmup
    for arm in 0..5 {
        for blk in blocks {
            run_arm::<false>(arm, &mut kernels[arm], blk, &mut out);
            ws_max[arm] = ws_max[arm].max(kernels[arm].ws_bytes());
        }
    }

    // Interleaved paired rounds (forward + reverse arm order)
    for r in 0..rounds {
        let order: [usize; 5] = if r % 2 == 0 { [0, 1, 2, 3, 4] } else { [4, 3, 2, 1, 0] };
        for &arm in &order {
            let mut sum_tm = StageTimes::default();
            for blk in blocks {
                let tm = run_arm::<true>(arm, &mut kernels[arm], blk, &mut out);
                sum_tm.s1_us += tm.s1_us;
                sum_tm.s2a_us += tm.s2a_us;
                sum_tm.s2b_us += tm.s2b_us;
                sum_tm.s3a_us += tm.s3a_us;
                sum_tm.s4_us += tm.s4_us;
                sum_tm.s5_us += tm.s5_us;
            }
            totals[arm] += sum_tm.total_us();
            s2b_tot[arm] += sum_tm.s2b_us;
            s3a_tot[arm] += sum_tm.s3a_us;
        }
    }

    let base = totals[0] / rounds as f64;
    let base_s2b = s2b_tot[0] / rounds as f64;
    println!("  [1T STAGE BENCHMARK — {label} ({} blks, {rounds} interleaved rounds)]:", blocks.len());
    println!(
        "    {:<16} {:>10} {:>8} | {:>10} {:>8} | {:>10} | {:>9}",
        "Arm", "Total(ms)", "vsBase", "S2b(ms)", "S2b_x", "S3a(ms)", "PeakWS"
    );
    for arm in 0..5 {
        let t_ms = (totals[arm] / rounds as f64) / 1000.0;
        let s2b_ms = (s2b_tot[arm] / rounds as f64) / 1000.0;
        let s3a_ms = (s3a_tot[arm] / rounds as f64) / 1000.0;
        println!(
            "    {:<16} {:>10.3} {:>7.4}x | {:>10.3} {:>7.4}x | {:>10.3} | {:>7.1}K",
            ARMS[arm],
            t_ms,
            (totals[arm] / rounds as f64) / base,
            s2b_ms,
            (s2b_tot[arm] / rounds as f64) / base_s2b.max(1e-9),
            s3a_ms,
            ws_max[arm] as f64 / 1024.0
        );
    }
    println!();
}

fn bench_parallel(label: &str, blocks: &[Block], workers: usize, rounds: usize) {
    let mut totals = [0.0f64; 5];
    for r in 0..rounds {
        let order: [usize; 5] = if r % 2 == 0 { [0, 1, 2, 3, 4] } else { [4, 3, 2, 1, 0] };
        for &arm in &order {
            let barrier = Barrier::new(workers);
            let elapsed_ms = std::thread::scope(|s| {
                let mut handles = Vec::with_capacity(workers);
                for _ in 0..workers {
                    let bar = &barrier;
                    handles.push(s.spawn(move || {
                        let mut kk = N2Kernel::default();
                        let mut out = Vec::new();
                        // Warmup 1 block
                        run_arm::<false>(arm, &mut kk, &blocks[0], &mut out);
                        bar.wait();
                        let t0 = Instant::now();
                        for blk in blocks {
                            run_arm::<false>(arm, &mut kk, blk, &mut out);
                        }
                        t0.elapsed().as_secs_f64() * 1000.0
                    }));
                }
                let mut max_ms = 0.0f64;
                for h in handles {
                    let v = h.join().unwrap();
                    if v > max_ms {
                        max_ms = v;
                    }
                }
                max_ms
            });
            totals[arm] += elapsed_ms;
        }
    }
    let base = totals[0] / rounds as f64;
    print!("  [{workers}W Wall — {label} ({rounds} rnds)]: ");
    for arm in 0..5 {
        let ms = totals[arm] / rounds as f64;
        print!("{}={ms:.2}ms ({:.4}x)  ", ARMS[arm], ms / base);
    }
    println!();
}

fn main() {
    let t0 = Instant::now();
    println!("Cycle 50 P2 — Analogue N, Experimental Test N2: 2-Band High-PQ Cycle Contraction in S2b\n");

    let stress = vec![
        make_christoffel_block("Chr-Fib-F23 (M=1)", 17_711, 10_946),
        make_christoffel_block("Chr-Pell (M=2)", 19_601, 8_119),
        make_christoffel_block("Chr-HiPQ-200", 10_051, 10_001),
        make_christoffel_block("Chr-HiPQ-1578", 30_019, 30_000),
        make_multi_2band_hipq_block("Multi15-2Band-HiPQ"),
    ];

    let adm6 = load_six_admitted();
    let nw_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ]);
    let cd_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]);
    let w_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ]);
    let nw_names = ["jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0"];
    let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];

    let nw6 = load_bin_blocks(&nw_path, Some(&nw_names), false, true);
    let cd4 = load_bin_blocks(&cd_path, Some(&cd_names), false, true);
    let wiki233 = load_bin_blocks(&w_path, None, true, true);
    assert_eq!(adm6.len(), 6);
    assert_eq!(nw6.len(), 6);
    assert_eq!(cd4.len(), 4);
    assert_eq!(wiki233.len(), 233);

    let mut nonwiki10 = nw6.clone();
    nonwiki10.extend(cd4.iter().cloned());

    run_exactness_suite(&stress, &adm6, &nw6, &cd4, &wiki233);

    println!("================================================================================");
    println!("PART 2 — SOLVED ANALOGUE N STRESS BENCHMARK (High-PQ 2-Band vs Markov-Zone)");
    println!("================================================================================");
    for blk in &stress {
        bench_1t_stages(&blk.name, std::slice::from_ref(blk), 15);
    }

    println!("================================================================================");
    println!("PART 3 — REAL CORPORA BENCHMARK (6 Adm + 10 Non-Wiki + 233 Full Wikidata, 1T/2W/4W)");
    println!("================================================================================");
    bench_1t_stages("6 Admitted Wikidata", &adm6, 10);
    bench_1t_stages("6 Non-Wiki (JSONL+Syslog)", &nw6, 10);
    bench_1t_stages("4 Code/Docs", &cd4, 10);
    bench_1t_stages("10 Non-Wiki Combined", &nonwiki10, 8);
    bench_1t_stages("233 Full Wikidata", &wiki233, 3);

    println!("  --- Multi-Worker Contention Scaling (2W & 4W) ---");
    bench_parallel("6 Admitted", &adm6, 2, 8);
    bench_parallel("6 Admitted", &adm6, 4, 8);
    bench_parallel("10 Non-Wiki", &nonwiki10, 2, 6);
    bench_parallel("10 Non-Wiki", &nonwiki10, 4, 6);
    bench_parallel("233 Full Wiki", &wiki233, 2, 2);
    bench_parallel("233 Full Wiki", &wiki233, 4, 2);

    println!("\nTotal elapsed: {:.2}s", t0.elapsed().as_secs_f64());
}
