#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

use std::fs;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 0. Exact Stage-Timed & Analogue-F Construction Kernel over cand_r5_head
// ============================================================================
// Const-generic variants over exact cand_r5_head architecture:
//   INLINE3=false, PRECHAIN=false, TAILPIN=false -> Exact cand_r5_head mirror (ID_BITS=22)
//   INLINE3=true,  PRECHAIN=false, TAILPIN=false -> F-Inline3: Order-3 De Bruijn 24-bit
//                                                   inline trigram packing (ID_BITS=24, +0 B WS!)
//   INLINE3=true,  PRECHAIN=false, TAILPIN=true  -> F-Inline3+TP: F-Inline3 + TailPin S2b
//   INLINE3=true,  PRECHAIN=true,  TAILPIN=false -> F-HierFast: Branchless O(ns) Konig/Hierholzer
//                                                   exact 1-local pre-chaining + F-Inline3

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const NONE_STRIP: u16 = u16::MAX;

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
    fn construct_us(&self) -> f64 {
        self.s1_us + self.s2a_us + self.s2b_us + self.s3a_us + self.s4_us
    }
    fn induced_us(&self) -> f64 {
        self.s2a_us + self.s2b_us + self.s3a_us
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct FStats {
    ns_orig: usize,
    ns_live: usize,
    open_nodes: usize,
    chains_contracted: usize,
    cycles_excised: usize,
    rules: usize,
    bands: usize,
    retained: usize,
    spirals: bool,
    wide: bool,
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

#[cold]
#[inline(never)]
fn grow(a: &mut Vec<u32>, need: usize) {
    a.resize(need.max(2 * a.len()), 0);
}

/// Handle encoding policy:
///   INLINE3 = false -> cand_r5_head exact (ID_BITS = 22, inline grades 1..=2, RULE_BASE = 65_793)
///   INLINE3 = true  -> Order-3 De Bruijn inline (ID_BITS = 24, inline grades 1..=3, RULE_BASE = 0)
#[inline(always)]
const fn id_bits<const INLINE3: bool>() -> u32 {
    if INLINE3 { 24 } else { 22 }
}

#[inline(always)]
const fn id_mask<const INLINE3: bool>() -> u32 {
    (1u32 << id_bits::<INLINE3>()) - 1
}

#[inline(always)]
const fn saturated<const INLINE3: bool>() -> u32 {
    u32::MAX >> id_bits::<INLINE3>()
}

#[inline(always)]
const fn rule_base<const INLINE3: bool>() -> u32 {
    if INLINE3 { 0 } else { 65_793 }
}

#[inline(always)]
const fn max_inline_grade<const INLINE3: bool>() -> u32 {
    if INLINE3 { 3 } else { 2 }
}

#[inline(always)]
const fn literal<const INLINE3: bool>(byte: u8) -> u32 {
    (1u32 << id_bits::<INLINE3>()) | (byte as u32)
}

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

struct Elim<'a, L: Links, const INLINE3: bool> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const INLINE3: bool> Elim<'_, L, INLINE3> {
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

    #[inline(always)]
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
        self.base + (id - rule_base::<INLINE3>()) as usize * RULE
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> id_bits::<INLINE3>();
        if g < saturated::<INLINE3>() {
            g
        } else {
            self.a[self.rule(h & id_mask::<INLINE3>()) + 2]
        }
    }

    /// Order-3 De Bruijn inline payload extraction:
    /// When INLINE3=true, grades 1, 2, AND 3 decode in 0 memory loads!
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> id_bits::<INLINE3>();
        let id = h & id_mask::<INLINE3>();
        if g <= max_inline_grade::<INLINE3>() {
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
    fn push_short_bytes(&mut self, payload: u128, len: u32) -> u32 {
        if len <= max_inline_grade::<INLINE3>() {
            return (len << id_bits::<INLINE3>()) | (payload as u32 & id_mask::<INLINE3>());
        }
        let id = ((self.top - self.base) / RULE) as u32 + rule_base::<INLINE3>();
        let bytes = payload | (u128::from(len) << 120);
        let out = &mut self.a[self.top..self.top + RULE];
        out[0] = bytes as u32;
        out[1] = (bytes >> 32) as u32;
        out[2] = (bytes >> 64) as u32;
        out[3] = (bytes >> 96) as u32;
        self.top += RULE;
        (len << id_bits::<INLINE3>()) | id
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> id_bits::<INLINE3>();
        let sum = left_len + (right >> id_bits::<INLINE3>());
        if sum < INLINE as u32 {
            if sum <= max_inline_grade::<INLINE3>() {
                // Zero arena writes for bigrams (and trigrams when INLINE3=true)!
                let lp = left & id_mask::<INLINE3>();
                let rp = right & id_mask::<INLINE3>();
                return (sum << id_bits::<INLINE3>()) | lp | (rp << (8 * left_len));
            }
            let id = ((self.top - self.base) / RULE) as u32 + rule_base::<INLINE3>();
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << id_bits::<INLINE3>()) | id
        } else {
            let len = if sum <= saturated::<INLINE3>() {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + rule_base::<INLINE3>();
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(saturated::<INLINE3>()) << id_bits::<INLINE3>()) | id
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

    #[inline(never)]
    fn eliminate_narrow_tailpin(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        if left == 0 {
            return true;
        }
        let mut x = self.a[0] & 0xFFFF;
        let mut y = self.a[1] & 0xFFFF;
        let mut xi = x as usize * 4;
        let mut yi = y as usize * 4;
        let (mut x0, mut x1, mut aw, mut wx) = {
            let sl = &self.a[xi..xi + 4];
            (sl[0], sl[1], sl[2], sl[3])
        };
        let (mut y0, mut y1, mut bw, mut wy) = {
            let sl = &self.a[yi..yi + 4];
            (sl[0], sl[1], sl[2], sl[3])
        };

        let done = loop {
            if self.top + RULE > self.a.len() {
                self.a[xi + 2] = aw;
                self.a[yi + 2] = bw;
                self.a[0] = (self.a[0] & 0xFFFF_0000) | x;
                self.a[1] = (self.a[1] & 0xFFFF_0000) | y;
                break false;
            }
            if x == y {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px0 = x0 & 0xFFFF;
                let px1 = x1 & 0xFFFF;
                self.a[px0 as usize * 4] &= 0xFFFF;
                self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                x = px0;
                y = px1;
                if left == 0 {
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | x;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | y;
                    break true;
                }
                xi = x as usize * 4;
                yi = y as usize * 4;
                let slx = &self.a[xi..xi + 4];
                (x0, x1, aw, wx) = (slx[0], slx[1], slx[2], slx[3]);
                let sly = &self.a[yi..yi + 4];
                (y0, y1, bw, wy) = (sly[0], sly[1], sly[2], sly[3]);
                continue;
            }
            let word = if wx == 0 {
                wy
            } else if wy == 0 {
                wx
            } else {
                self.join(wx, wy)
            };
            if aw < bw {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px = x0 & 0xFFFF;
                let y_next = if y == px { 0 } else { y0 >> 16 };
                self.a[px as usize * 4] &= 0xFFFF;
                let new_x0 = y | (y_next << 16);
                {
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = new_x0;
                    nxm[2] = aw;
                    nxm[3] = word;
                }
                let new_y0 = (y0 & 0xFFFF) | (x << 16);
                self.a[yi] = new_y0;
                bw -= aw;
                y0 = new_y0;
                if y_next != 0 {
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    x = px;
                    xi = x as usize * 4;
                    let slx = &self.a[xi..xi + 4];
                    (x0, x1, aw, wx) = (slx[0], slx[1], slx[2], slx[3]);
                } else {
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | x;
                    x0 = new_x0;
                    wx = word;
                }
            } else if aw > bw {
                assert!(bw > 0 && bw <= left);
                left -= bw;
                let py = y1 & 0xFFFF;
                let x_next = if x == py { 0 } else { x1 >> 16 };
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                let new_y1 = x | (x_next << 16);
                {
                    let nym = &mut self.a[yi..yi + 4];
                    nym[1] = new_y1;
                    nym[2] = bw;
                    nym[3] = word;
                }
                let new_x1 = (x1 & 0xFFFF) | (y << 16);
                self.a[xi + 1] = new_x1;
                aw -= bw;
                x1 = new_x1;
                if x_next != 0 {
                    let xni = x_next as usize * 4 + 1;
                    self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
                    y = py;
                    yi = y as usize * 4;
                    let sly = &self.a[yi..yi + 4];
                    (y0, y1, bw, wy) = (sly[0], sly[1], sly[2], sly[3]);
                } else {
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | y;
                    y1 = new_y1;
                    wy = word;
                }
            } else {
                assert!(aw > 0 && aw <= left);
                left -= aw;
                let px = x0 & 0xFFFF;
                let py = y1 & 0xFFFF;
                self.a[px as usize * 4] &= 0xFFFF;
                let y_next = if y == px { 0 } else { y0 >> 16 };
                let y_prev = y0 & 0xFFFF;
                {
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[0] = y_prev | (y_next << 16);
                    nxm[2] = aw;
                    nxm[3] = word;
                }
                let ypi = y_prev as usize * 4;
                self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                if y_next != 0 {
                    let yni = y_next as usize * 4;
                    self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                } else {
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | x;
                }
                self.a[py as usize * 4 + 1] &= 0xFFFF;
                x = if px == y { x } else { px };
                y = py;
                if left == 0 {
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | x;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | y;
                    break true;
                }
                xi = x as usize * 4;
                yi = y as usize * 4;
                let slx = &self.a[xi..xi + 4];
                (x0, x1, aw, wx) = (slx[0], slx[1], slx[2], slx[3]);
                let sly = &self.a[yi..yi + 4];
                (y0, y1, bw, wy) = (sly[0], sly[1], sly[2], sly[3]);
            }
        };
        *remaining = left;
        done
    }

    fn eliminate<const SPIRALS: bool, const TAILPIN: bool>(&mut self, remaining: &mut u32) -> bool {
        if !SPIRALS && L::IS_NARROW {
            if TAILPIN {
                return self.eliminate_narrow_tailpin(remaining);
            } else {
                return self.eliminate_narrow_plain(remaining);
            }
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

    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> id_bits::<INLINE3>()) as usize;
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

    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> id_bits::<INLINE3>();
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & id_mask::<INLINE3>());
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
    /// Works identically for both ID_BITS=22 and ID_BITS=24 with exact same 512-word `pairs`!
    /// When INLINE3=true: stores `(handle, (byte << 24) | code)` — zero collisions, zero `uid` math!
    #[inline(always)]
    fn code<L: Links, const INLINE3: bool>(&mut self, e: &mut Elim<L, INLINE3>, handle: u32, byte: u8) -> u32 {
        if INLINE3 {
            let bucket = ((handle.wrapping_mul(0x9e37_79b9) ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b))) >> 24) as usize;
            let slot = bucket * 2;
            let val = self.pairs[slot + 1];
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
                return val & 0x00FF_FFFF;
            }
            let code = self.code_miss(e, handle, handle & id_mask::<INLINE3>(), byte);
            self.pairs[slot] = handle;
            self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
            code
        } else {
            let g = handle >> id_bits::<INLINE3>();
            let id = handle & id_mask::<INLINE3>();
            let uid = id + 1 + (u32::from(g == 2) << 8);
            let key = (uid << 8) | u32::from(byte);
            let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
            let slot = bucket * 2;
            if self.pairs[slot] == key {
                return self.pairs[slot + 1];
            }
            let code = self.code_miss(e, handle, id, byte);
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            code
        }
    }

    fn code_miss<L: Links, const INLINE3: bool>(
        &mut self,
        e: &mut Elim<L, INLINE3>,
        handle: u32,
        id: u32,
        byte: u8,
    ) -> u32 {
        let height = (handle >> id_bits::<INLINE3>()) as usize;
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

/// Exact cand_r5_head Pass helper with cold `take_marked` so S1 codegen matches 1.000x!
struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
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
        let end = pos + width;
        if pos < p {
            self.piece(c, pos, p - pos, false, p);
        }
        self.piece(c, p, 1, true, p);
        if p + 1 < end {
            self.piece(c, p + 1, end - p - 1, false, p);
        }
    }

    #[inline(always)]
    fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }
            self.queues.push_thin(a, f.m, w);
            f.m += w;
        } else {
            let at = 3 * self.ns;
            if self.edges.len() < at + 3 {
                let need = (at + 3).max(2 * self.edges.len()).max(1024);
                self.edges.reserve_exact(need - self.edges.len());
                self.edges.resize(need, 0);
            }
            let thin_a = self.queues.thin_of(a);
            self.edges[at..at + 3].copy_from_slice(&[
                x,
                thin_a + f.thick_rows[a],
                (w << 8) | u32::from(c),
            ]);
            self.ns += 1;
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

#[derive(Default)]
struct FKernel {
    spill: outq::Spill,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    predecessor: Option<u32>,
    img_packed: Vec<u64>,
    next_exact: Vec<u16>,
    tail_sid: Vec<u16>,
    has_pred: Vec<u8>,
    force_wide: bool,
    force_spirals: Option<bool>,
}

impl FKernel {
    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn is_wide(&self, ns: usize) -> bool {
        Self::nodes_most(ns) > NARROW_NODES || self.force_wide
    }

    fn arena_words(&self, ns: usize, m: usize) -> usize {
        let stride = if self.is_wide(ns) {
            Wide::STRIDE
        } else {
            Narrow::STRIDE
        };
        (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
    }

    #[inline(never)]
    fn run<const INLINE3: bool, const PRECHAIN: bool, const TAILPIN: bool, const TIMED: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (FStats, StageTimes) {
        let mut tm = StageTimes::default();
        let t_s1 = if TIMED { Some(Instant::now()) } else { None };
        out.resize(n + INLINE, 0);
        let (ns, f, heads) = {
            let mut pass = Pass {
                queues: outq::Queues::new(&mut out[..], &mut self.spill, u32::MAX),
                edges: &mut self.edges,
                ns: 0,
                f: First::default(),
            };
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= LIMIT {
                    pass.piece(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take(c, pos, width, p);
                    pos += width;
                }
            }
            let thin = pass.queues.extract_thin();
            let heads = pass.queues.heads();
            let mut f = pass.f;
            f.thin = thin;
            (pass.ns, f, heads)
        };
        if let Some(t0) = t_s1 {
            tm.s1_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        assert!(f.m > 0 && all.iter().sum::<u32>() == n as u32);
        let first = starts(all);
        let mut write = starts(f.thick);
        let need = self.arena_words(ns, f.m as usize);
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        self.bands.clear();
        if n >= 65_536 && self.bands.capacity() < 8192 {
            self.bands.reserve_exact(8192);
        }
        self.spelling.clear();
        if n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }

        let mut stats = FStats {
            ns_orig: ns,
            retained: f.m as usize,
            wide: self.is_wide(ns),
            ..Default::default()
        };
        let most = Self::nodes_most(ns);
        let cursors = if stats.wide {
            self.induced_with::<Wide, INLINE3, false, false, TIMED>(
                ns, n as u32, &f.thin, &first, &mut write, &mut stats, most, &mut tm,
            )
        } else {
            self.induced_with::<Narrow, INLINE3, PRECHAIN, TAILPIN, TIMED>(
                ns, n as u32, &f.thin, &first, &mut write, &mut stats, most, &mut tm,
            )
        };
        stats.bands = self.bands.len();

        let t_s4 = if TIMED { Some(Instant::now()) } else { None };
        let m = f.m;
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
            let mut predecessor = f.mark;
            let mb = usize::from(f.mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
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
                            f.mark,
                            &mut predecessor,
                        );
                    }
                }
            }
            self.predecessor = Some(predecessor);
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(&out[..], &self.spill, &heads, a);
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
        if let Some(t0) = t_s4 {
            tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s5 = if TIMED { Some(Instant::now()) } else { None };
        if let Some(before) = self.predecessor {
            emit_bidirectional(
                &self.edges[..m as usize],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                before,
                bits,
                out,
            );
        } else {
            emit(
                &self.edges[..m as usize],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                f.mark,
                bits,
                out,
            );
        }
        if let Some(t0) = t_s5 {
            tm.s5_us = t0.elapsed().as_secs_f64() * 1e6;
        }
        (stats, tm)
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links, const INLINE3: bool, const PRECHAIN: bool, const TAILPIN: bool, const TIMED: bool>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut FStats,
        most: usize,
        tm: &mut StageTimes,
    ) -> [usize; 256] {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= 4 * ns + nodes_words);

        let mut remaining = n;
        let mut widest = 0u32;

        let (base_after_s2a, top_after_s2a, nodes_after_s2a) = if !PRECHAIN || ns < 2 {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L, INLINE3> {
                a: arena,
                base: 0,
                top: 0,
                nodes: 0,
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            e.nodes = ns as u32 + 1;
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
                e.set_word(sid, literal::<INLINE3>(byte));
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
                let sw = &strips_raw[3 * idx..3 * idx + 3];
                let to = sw[1];
                let width = sw[2] >> 8;
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
            stats.ns_live = ns;
            stats.open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, base, e.nodes)
        } else {
            if self.img_packed.len() < ns {
                self.img_packed.resize(ns, 0);
                self.next_exact.resize(ns, NONE_STRIP);
                self.tail_sid.resize(ns, 0);
                self.has_pred.resize(ns, 0);
            }
            let img_packed = &mut self.img_packed[..ns];
            let next_exact = &mut self.next_exact[..ns];
            let tail_sid = &mut self.tail_sid[..ns];
            let has_pred = &mut self.has_pred[..ns];
            next_exact.fill(NONE_STRIP);
            tail_sid.fill(0);
            has_pred.fill(0);

            {
                let strips_raw = &mut self.edges[..3 * ns];
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let wbyte = sw[2];
                    let a = (wbyte & 0xFF) as usize;
                    let w = wbyte >> 8;
                    let to = sw[1] + first[a];
                    sw[1] = to;
                    let dst_idx = write[a] as usize;
                    write[a] = dst_idx as u32 + 1;
                    img_packed[dst_idx] =
                        (u64::from(to) << 44) | (u64::from(w) << 24) | ((i as u64) << 8) | (a as u64);
                }
            }

            let mut exact_cnt = 0usize;
            {
                let strips_raw = &self.edges[..3 * ns];
                let mut d = 0usize;
                let mut k = 0usize;
                while d < ns && k < ns {
                    let pk = img_packed[k];
                    let to_i = (pk >> 44) as u32;
                    let from_d = strips_raw[3 * d];
                    if to_i == from_d {
                        let wi = ((pk >> 24) & 0xF_FFFF) as u32;
                        let wd = strips_raw[3 * d + 2] >> 8;
                        if wi == wd {
                            let i = ((pk >> 8) & 0xFFFF) as usize;
                            next_exact[i] = d as u16;
                            has_pred[d] = 1;
                            exact_cnt += 1;
                        }
                        k += 1;
                        d += 1;
                    } else {
                        let lt = usize::from(to_i < from_d);
                        k += lt;
                        d += 1 - lt;
                    }
                }
            }

            let fixed_base = L::STRIDE * most;
            if 4 * ns + fixed_base + (exact_cnt + 16) * RULE > self.edges.len() {
                grow(&mut self.edges, 4 * ns + fixed_base + (exact_cnt + 16) * RULE);
            }
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let strips_raw = &hdr[..3 * ns];
            let mut e = Elim::<L, INLINE3> {
                a: arena,
                base: fixed_base,
                top: fixed_base,
                nodes: 1,
                _links: std::marker::PhantomData,
            };
            e.set_width(0, 0);
            e.set_word(0, 0);

            let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
            let mut x = 0u32;
            let mut pending_gap = 0u32;
            let mut live_ns = 0usize;
            let mut chains_contracted = 0usize;
            let mut contracted_rows = 0u32;

            for h in 0..ns {
                let sw = &strips_raw[3 * h..3 * h + 3];
                let from = sw[0];
                let wbyte = sw[2];
                let w = wbyte >> 8;
                if x < from {
                    pending_gap += from - x;
                }
                x = from + w;
                if (has_pred[h] & 1) != 0 {
                    continue;
                }
                has_pred[h] |= 2;
                if pending_gap > 0 {
                    widest = widest.max(pending_gap);
                    let gid = e.node(pending_gap, 0);
                    e.append_to(&mut order[0], gid, 0);
                    e.append_to(&mut terminals[1], gid, 1);
                    pending_gap = 0;
                }
                live_ns += 1;
                widest = widest.max(w);
                let b0 = (wbyte & 0xFF) as u8;
                let nxt0 = next_exact[h];
                let (word_h, tail) = if nxt0 == NONE_STRIP {
                    (literal::<INLINE3>(b0), h)
                } else {
                    let mut payload = u128::from(b0);
                    let mut chunk_len = 1u32;
                    let mut acc = 0u32;
                    let mut cur = nxt0 as usize;
                    let mut hops = 0u32;
                    loop {
                        has_pred[cur] |= 2;
                        hops += 1;
                        let b = (strips_raw[3 * cur + 2] & 0xFF) as u8;
                        if chunk_len == (INLINE - 1) as u32 {
                            let ch = e.push_short_bytes(payload, chunk_len);
                            acc = if acc == 0 { ch } else { e.join(ch, acc) };
                            payload = u128::from(b);
                            chunk_len = 1;
                        } else {
                            payload = (payload << 8) | u128::from(b);
                            chunk_len += 1;
                        }
                        let nxt = next_exact[cur];
                        if nxt == NONE_STRIP {
                            break;
                        }
                        cur = nxt as usize;
                    }
                    let ch = e.push_short_bytes(payload, chunk_len);
                    let final_w = if acc == 0 { ch } else { e.join(ch, acc) };
                    chains_contracted += hops as usize;
                    contracted_rows += hops * w;
                    (final_w, cur)
                };
                let sid = e.node(w, word_h);
                e.append_to(&mut order[0], sid, 0);
                tail_sid[tail] = sid as u16;
            }
            if x < n {
                pending_gap += n - x;
            }
            if pending_gap > 0 {
                widest = widest.max(pending_gap);
                let gid = e.node(pending_gap, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }

            let mut cycles_excised = 0usize;
            if chains_contracted < exact_cnt {
                for i in 0..ns {
                    if has_pred[i] == 1 {
                        let w = strips_raw[3 * i + 2] >> 8;
                        let mut cur = i;
                        let mut clen = 0u32;
                        while (has_pred[cur] & 2) == 0 {
                            has_pred[cur] |= 2;
                            clen += 1;
                            cur = next_exact[cur] as usize;
                        }
                        cycles_excised += clen as usize;
                        contracted_rows += clen * w;
                    }
                }
            }
            remaining = n - contracted_rows;

            x = 0;
            pending_gap = 0;
            for &pk in img_packed.iter() {
                let to = (pk >> 44) as u32;
                let w = ((pk >> 24) & 0xF_FFFF) as u32;
                let idx = ((pk >> 8) & 0xFFFF) as usize;
                if x < to {
                    pending_gap += to - x;
                }
                x = to + w;
                let sid = u32::from(tail_sid[idx]);
                if sid != 0 {
                    if pending_gap > 0 {
                        widest = widest.max(pending_gap);
                        let gid = e.node(pending_gap, 0);
                        e.append_to(&mut order[1], gid, 1);
                        e.append_to(&mut terminals[0], gid, 0);
                        pending_gap = 0;
                    }
                    e.append_to(&mut order[1], sid, 1);
                }
            }
            if x < n {
                pending_gap += n - x;
            }
            if pending_gap > 0 {
                widest = widest.max(pending_gap);
                let gid = e.node(pending_gap, 0);
                e.append_to(&mut order[1], gid, 1);
                e.append_to(&mut terminals[0], gid, 0);
            }

            stats.ns_live = live_ns;
            stats.open_nodes = e.nodes as usize - 1;
            stats.chains_contracted = chains_contracted;
            stats.cycles_excised = cycles_excised;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            (e.base, e.top, e.nodes)
        };
        if let Some(t0) = t_s2a {
            tm.s2a_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        stats.spirals = spirals;
        let (mut base, mut top, nodes) = (base_after_s2a, top_after_s2a, nodes_after_s2a);
        loop {
            let mut e = Elim::<L, INLINE3> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true, TAILPIN>(&mut remaining)
            } else {
                e.eliminate::<false, TAILPIN>(&mut remaining)
            };
            base = e.base;
            top = e.top;
            if done {
                break;
            }
            grow(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
        }
        stats.rules = (top - base) / RULE;
        if let Some(t0) = t_s2b {
            tm.s2b_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        {
            let mut e = Elim::<L, INLINE3> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(stats.retained as u32);
        }
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail
        };
        let mut e = Elim::<L, INLINE3> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, &mut self.inline16, &mut self.lens);
        let cursors = if cache.len() < CACHE_WORDS {
            let mut cursors = [0; 256];
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(
                            node_word,
                            a as u8,
                            &mut self.stack,
                            &mut self.spelling,
                        );
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
            assert_eq!(id, 0);
            cursors
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
            let mut cursors = [0; 256];
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
                    assert!(width > 0);
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
            assert_eq!(id, 0);
            cursors
        };
        if let Some(t0) = t_s3a {
            tm.s3a_us = t0.elapsed().as_secs_f64() * 1e6;
        }
        cursors
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
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

// ============================================================================
// 1. Exact 2-Unique Forward-FL Theorem (Unconditional on ALL 2-Unique Columns)
// ============================================================================

/// Exact 2-Unique Eulerian Decoder: matches forward-FL on EVERY 2-unique column
/// (valid BWT OR malformed multi-cycle column!), returns None cleanly in O(1) when
/// n > 65_536 or any bigram repeats.
fn try_invert_2unique_exact_fl(l: &[u8], p: u32) -> Option<Vec<u8>> {
    let n = l.len();
    if n == 0 || (p as usize) >= n || n > 65_536 {
        return None;
    }
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut out_edges: Vec<Vec<u8>> = (0..256).map(|_| Vec::new()).collect();
    let mut seen_in_bucket = [0u16; 256];
    for c in 0..256usize {
        let s = starts[c] as usize;
        let e = s + counts[c] as usize;
        let stamp = (c + 1) as u16;
        for &b in &l[s..e] {
            if seen_in_bucket[b as usize] == stamp {
                return None;
            }
            seen_in_bucket[b as usize] = stamp;
            out_edges[b as usize].push(c as u8);
        }
    }
    let mut next_after = vec![0u8; 256 * 256];
    let mut p_a = 0u8;
    for a in 0..256usize {
        let s = starts[a] as usize;
        let cnt = counts[a] as usize;
        if cnt == 0 {
            continue;
        }
        if (p as usize) >= s && (p as usize) < s + cnt {
            p_a = a as u8;
        }
        let outs = &out_edges[a];
        for j in 0..cnt {
            let b = l[s + j] as usize;
            next_after[(b << 8) | a] = outs[j];
        }
    }
    let mut out = Vec::with_capacity(n);
    let mut prev = l[p as usize];
    let mut cur = p_a;
    for _ in 0..n {
        out.push(cur);
        let nxt = next_after[((prev as usize) << 8) | (cur as usize)];
        prev = cur;
        cur = nxt;
    }
    Some(out)
}

// ============================================================================
// 2. Corpora & Exactness Verification Helpers
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
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

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        let b = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        l.resize(l.len() + len, b);
    }
    assert_eq!(l.len(), n);
    l
}

fn forward_fl_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    let mut cur = starts;
    for (i, &b) in l.iter().enumerate() {
        let pos = cur[b as usize] as usize;
        fl[pos] = ((i as u32) << 8) | u32::from(b);
        cur[b as usize] += 1;
    }
    let mut out = Vec::with_capacity(n);
    let mut x = p as usize;
    for _ in 0..n {
        let e = fl[x];
        out.push((e & 0xFF) as u8);
        x = (e >> 8) as usize;
    }
    out
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
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = forward_fl_invert(&l, p);
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
                expected,
            }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
    for b_idx in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let _crc = u32::from_le_bytes(raw[off + 12..off + 16].try_into().unwrap());
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
        let expected = if with_expected {
            let l = expand_records(&records, n);
            forward_fl_invert(&l, p)
        } else {
            Vec::new()
        };
        let name = if let Some(ns) = names {
            if b_idx < ns.len() { ns[b_idx].to_string() } else { format!("blk-{idx}") }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block {
            name,
            n,
            p,
            records,
            expected,
        });
    }
    blocks
}

// ============================================================================
// 3. Exactness Suite: 5,590 Small + 250 Malformed + 200 Malformed 2-Unique + Fallbacks
// ============================================================================

fn run_exactness_suite(wiki: &[Block], nonwiki: &[Block]) {
    println!("================================================================================");
    println!("EXACTNESS VERIFICATION SUITE (5,590 small + 450 malformed FL + forced fallbacks)");
    println!("================================================================================");
    let mut r5 = cand_r5_head::Kernel::default();
    let mut fk = FKernel::default();
    let mut out_r5 = Vec::new();
    let mut out_i3 = Vec::new();
    let mut out_tp = Vec::new();
    let mut out_hf = Vec::new();

    let mut check_all = |l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let n = l.len();
        let fl_exp = forward_fl_invert(l, p);
        r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_r5).unwrap();
        assert_eq!(out_r5, fl_exp, "cand_r5_head vs FL mismatch on {tag}");
        fk.run::<true, false, false, false>(&recs, n, p, &mut out_i3);
        assert_eq!(out_i3, out_r5, "F-Inline3 mismatch on {tag}");
        fk.run::<true, false, true, false>(&recs, n, p, &mut out_tp);
        assert_eq!(out_tp, out_r5, "F-Inline3+TP mismatch on {tag}");
        fk.run::<true, true, false, false>(&recs, n, p, &mut out_hf);
        assert_eq!(out_hf, out_r5, "F-HierFast mismatch on {tag}");
        if let Some(u2) = try_invert_2unique_exact_fl(l, p) {
            assert_eq!(u2, fl_exp, "Exact 2-Unique FL mismatch on {tag}");
        }
    };

    let mut small_cnt = 0usize;
    for n in 1..=10usize {
        for mask in 0..(1u32 << n) {
            let mut l = Vec::with_capacity(n);
            for bit in 0..n {
                l.push(if (mask >> bit) & 1 == 1 { b'b' } else { b'a' });
            }
            check_all(&l, 0, "bin-p0");
            small_cnt += 1;
            if n >= 2 {
                check_all(&l, (n - 1) as u32, "bin-plast");
                small_cnt += 1;
            }
        }
    }

    let mut rng = 0xDEAD_BEEF_CAFE_BABEu64;
    let next_u64 = |s: &mut u64| -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s
    };
    for trial in 0..1500usize {
        let sigma = 2 + (next_u64(&mut rng) as usize % 6);
        let n_runs = 4 + (next_u64(&mut rng) as usize % 16);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = b'a' + (next_u64(&mut rng) as usize % sigma) as u8;
            let w = if next_u64(&mut rng) % 3 == 0 {
                17 + (next_u64(&mut rng) as usize % 30)
            } else {
                1 + (next_u64(&mut rng) as usize % 16)
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("synth-thick-{trial}"));
        small_cnt += 1;
    }
    println!("  PASS: {small_cnt} small & thick-strip synthetic cases 100% BYTE-EXACT.");

    // 250 malformed multi-cycle / pure-thick-cycle columns
    let mut mal_cnt = 0usize;
    for &w in &[17usize, 20, 32, 64] {
        let mut l = Vec::new();
        l.push(b'z');
        l.resize(l.len() + w, b'b');
        l.resize(l.len() + w, b'a');
        for p in [0u32, 1, w as u32, (2 * w) as u32] {
            check_all(&l, p, "thick-cycle-2");
            mal_cnt += 1;
        }
        let mut l2 = Vec::new();
        l2.resize(w, b'a');
        l2.push(b'b');
        l2.resize(l2.len() + w, b'c');
        for p in [0u32, w as u32, (w + 1) as u32] {
            check_all(&l2, p, "thick-self-loop");
            mal_cnt += 1;
        }
    }
    for trial in 0..222usize {
        let mut l = Vec::new();
        let n_runs = 6 + (next_u64(&mut rng) as usize % 20);
        let sigma = 2 + (next_u64(&mut rng) as usize % 5);
        for _ in 0..n_runs {
            let b = b'A' + (next_u64(&mut rng) as usize % sigma) as u8;
            let w = match next_u64(&mut rng) % 4 {
                0 => 20,
                1 => 40,
                2 => 1 + (next_u64(&mut rng) as usize % 5),
                _ => 17 + (next_u64(&mut rng) as usize % 25),
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("malformed-{trial}"));
        mal_cnt += 1;
    }
    println!("  PASS: {mal_cnt} malformed multi-cycle / pure-thick-cycle columns 100% BYTE-EXACT vs cand_r5_head & forward-FL.");

    // 200 crafted random 2-unique columns (almost all malformed multi-cycle!) testing Exact 2-Unique FL Theorem
    let mut u2_accepted = 0usize;
    for trial in 0..200usize {
        let sigma = 4 + (next_u64(&mut rng) as usize % 12);
        let deg = 2 + (next_u64(&mut rng) as usize % (sigma - 1).min(6));
        // Build a column L where each F-bucket of size `deg` has `deg` distinct symbols (arbitrary permutation -> usually multi-cycle!)
        let mut l = Vec::with_capacity(sigma * deg);
        for _bucket in 0..sigma {
            let mut syms: Vec<u8> = (0..sigma as u8).map(|x| b'a' + x).collect();
            for i in (1..sigma).rev() {
                let j = (next_u64(&mut rng) as usize) % (i + 1);
                syms.swap(i, j);
            }
            // Wait: to keep total counts equal to `deg` per symbol, use a union of `deg` disjoint cyclic shifts!
            l.clear();
            break;
        }
        let mut buckets: Vec<Vec<u8>> = vec![Vec::with_capacity(deg); sigma];
        for shift in 0..deg {
            for b in 0..sigma {
                buckets[b].push(b'a' + ((b + shift) % sigma) as u8);
            }
        }
        for b in 0..sigma {
            for i in (1..deg).rev() {
                let j = (next_u64(&mut rng) as usize) % (i + 1);
                buckets[b].swap(i, j);
            }
            l.extend_from_slice(&buckets[b]);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        let fl_exp = forward_fl_invert(&l, p);
        let got = try_invert_2unique_exact_fl(&l, p).expect("valid 2-unique column must be accepted");
        assert_eq!(got, fl_exp, "2-Unique FL Theorem mismatch on malformed multi-cycle trial {trial}");
        u2_accepted += 1;
    }
    println!("  PASS: {u2_accepted}/200 random malformed multi-cycle 2-unique columns match forward-FL 100% BYTE-EXACT.");

    // Forced fallback tests: spiral-200k, force_wide, force_spirals, long words >255 bytes (SATURATED=255!)
    let mut spiral_l = vec![b'a'; 200_000];
    spiral_l[0] = b'c';
    spiral_l[199_999] = b'b';
    let spiral_recs = records_of(&spiral_l);
    let spiral_exp = forward_fl_invert(&spiral_l, 0);
    fk.run::<true, false, false, false>(&spiral_recs, spiral_l.len(), 0, &mut out_i3);
    assert_eq!(out_i3, spiral_exp, "F-Inline3 spiral-200k (>255B SATURATED!) mismatch");
    fk.run::<true, true, false, false>(&spiral_recs, spiral_l.len(), 0, &mut out_hf);
    assert_eq!(out_hf, spiral_exp, "F-HierFast spiral-200k mismatch");

    fk.force_wide = true;
    fk.run::<true, false, false, false>(&wiki[0].records, wiki[0].n, wiki[0].p, &mut out_i3);
    assert_eq!(out_i3, wiki[0].expected, "force_wide fallback mismatch");
    fk.force_wide = false;

    fk.force_spirals = Some(true);
    fk.run::<true, false, false, false>(&wiki[0].records, wiki[0].n, wiki[0].p, &mut out_i3);
    assert_eq!(out_i3, wiki[0].expected, "force_spirals fallback mismatch");
    fk.force_spirals = None;

    for blk in wiki.iter().chain(nonwiki.iter()) {
        // Verify Analogue-F 2-unique guard rejects real blocks cleanly in O(1)
        assert!(try_invert_2unique_exact_fl(&expand_records(&blk.records, blk.n), blk.p).is_none());
        fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut out_i3);
        assert_eq!(out_i3, blk.expected, "F-Inline3 mismatch on {}", blk.name);
        fk.run::<true, false, true, false>(&blk.records, blk.n, blk.p, &mut out_tp);
        assert_eq!(out_tp, blk.expected, "F-Inline3+TP mismatch on {}", blk.name);
        fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out_hf);
        assert_eq!(out_hf, blk.expected, "F-HierFast mismatch on {}", blk.name);
    }
    println!("  PASS: spiral-200k (SATURATED=255) + force_wide + force_spirals + 2-unique guard reject + 6/6 Wiki + 6/6 Non-Wiki BYTE-EXACT.");
}

// ============================================================================
// 4. Paired Stage Breakdown & 1T / 2W / 4W Multi-Worker Benchmarks
// ============================================================================

fn bench_detailed_1t(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("1T STAGE BREAKDOWN (PAIRED 15 ROUNDS, p50) — {label}");
    println!("================================================================================");

    let mut fk = FKernel::default();
    let mut r5 = cand_r5_head::Kernel::default();
    let mut out = Vec::new();

    println!("Rule & Node Counts (Base ID_BITS=22 vs F-Inline3 ID_BITS=24 vs F-HierFast+Inline3):");
    println!("{:<12} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "Block", "ns", "rules:B", "rules:I3", "dRule%", "rules:HF", "nodes:HF", "chains");
    let (mut sr_b, mut sr_i3, mut sr_hf) = (0usize, 0usize, 0usize);
    for blk in blocks {
        let (sb, _) = fk.run::<false, false, false, false>(&blk.records, blk.n, blk.p, &mut out);
        let (si, _) = fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut out);
        let (sh, _) = fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out);
        sr_b += sb.rules;
        sr_i3 += si.rules;
        sr_hf += sh.rules;
        println!("{:<12} {:>6} {:>7} {:>7} {:>+6.1}% {:>7} {:>7} {:>7}",
            blk.name, sb.ns_orig, sb.rules, si.rules,
            100.0 * (si.rules as f64 - sb.rules as f64) / sb.rules.max(1) as f64,
            sh.rules, sh.open_nodes, sh.chains_contracted + sh.cycles_excised);
    }
    println!("{:<12} {:>6} {:>7} {:>7} {:>+6.1}% {:>7}",
        "SUM", "", sr_b, sr_i3, 100.0 * (sr_i3 as f64 - sr_b as f64) / sr_b.max(1) as f64, sr_hf);

    println!("\n{:<12} | {:>7} {:>7} {:>7} {:>7} | {:>7} {:>7} {:>7} {:>7} | {:>8} {:>8} {:>8} {:>8}",
        "Block", "Ind:B", "Ind:I3", "Ind:TP", "Ind:HF", "Con:R5", "Con:B", "Con:I3", "Con:TP", "Tot:R5", "Tot:I3", "Tot:TP", "Tot:HF");
    let mut s_ind = [0.0f64; 4]; // [Base, Inline3, Inline3+TP, HierFast+Inline3]
    let mut s_2a = [0.0f64; 4];
    let mut s_2b = [0.0f64; 4];
    let mut s_3a = [0.0f64; 4];
    let mut s_con = [0.0f64; 4];
    let mut s_tot = [0.0f64; 4];
    let mut s_r5_con = 0.0;
    let mut s_r5_tot = 0.0;

    for blk in blocks {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
        fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out);

        let mut v_ind = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_2a = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_2b = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_3a = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_con = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_tot = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut v_r5_c = Vec::new();
        let mut v_r5_t = Vec::new();

        for _ in 0..15 {
            let (c, e) = r5.run_stages(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
            v_r5_c.push(c);
            v_r5_t.push(c + e);

            let (_, tm0) = fk.run::<false, false, false, true>(&blk.records, blk.n, blk.p, &mut out);
            v_ind[0].push(tm0.induced_us()); v_2a[0].push(tm0.s2a_us); v_2b[0].push(tm0.s2b_us); v_3a[0].push(tm0.s3a_us); v_con[0].push(tm0.construct_us());
            let (_, tm1) = fk.run::<true, false, false, true>(&blk.records, blk.n, blk.p, &mut out);
            v_ind[1].push(tm1.induced_us()); v_2a[1].push(tm1.s2a_us); v_2b[1].push(tm1.s2b_us); v_3a[1].push(tm1.s3a_us); v_con[1].push(tm1.construct_us());
            let (_, tm2) = fk.run::<true, false, true, true>(&blk.records, blk.n, blk.p, &mut out);
            v_ind[2].push(tm2.induced_us()); v_2a[2].push(tm2.s2a_us); v_2b[2].push(tm2.s2b_us); v_3a[2].push(tm2.s3a_us); v_con[2].push(tm2.construct_us());
            let (_, tm3) = fk.run::<true, true, false, true>(&blk.records, blk.n, blk.p, &mut out);
            v_ind[3].push(tm3.induced_us()); v_2a[3].push(tm3.s2a_us); v_2b[3].push(tm3.s2b_us); v_3a[3].push(tm3.s3a_us); v_con[3].push(tm3.construct_us());

            let t0 = Instant::now();
            fk.run::<false, false, false, false>(&blk.records, blk.n, blk.p, &mut out);
            v_tot[0].push(t0.elapsed().as_secs_f64() * 1e6);

            let t0 = Instant::now();
            fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut out);
            v_tot[1].push(t0.elapsed().as_secs_f64() * 1e6);

            let t0 = Instant::now();
            fk.run::<true, false, true, false>(&blk.records, blk.n, blk.p, &mut out);
            v_tot[2].push(t0.elapsed().as_secs_f64() * 1e6);

            let t0 = Instant::now();
            fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out);
            v_tot[3].push(t0.elapsed().as_secs_f64() * 1e6);
        }
        let p50 = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let r5c = p50(&mut v_r5_c);
        let r5t = p50(&mut v_r5_t);
        s_r5_con += r5c;
        s_r5_tot += r5t;
        let mi = [p50(&mut v_ind[0]), p50(&mut v_ind[1]), p50(&mut v_ind[2]), p50(&mut v_ind[3])];
        let mc = [p50(&mut v_con[0]), p50(&mut v_con[1]), p50(&mut v_con[2]), p50(&mut v_con[3])];
        let mt = [p50(&mut v_tot[0]), p50(&mut v_tot[1]), p50(&mut v_tot[2]), p50(&mut v_tot[3])];
        for j in 0..4 {
            s_ind[j] += mi[j];
            s_2a[j] += p50(&mut v_2a[j]);
            s_2b[j] += p50(&mut v_2b[j]);
            s_3a[j] += p50(&mut v_3a[j]);
            s_con[j] += mc[j];
            s_tot[j] += mt[j];
        }
        println!("{:<12} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
            blk.name, mi[0], mi[1], mi[2], mi[3], r5c, mc[0], mc[1], mc[2], r5t, mt[1], mt[2], mt[3]);
    }
    println!("{:<12} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
        "SUM", s_ind[0], s_ind[1], s_ind[2], s_ind[3], s_r5_con, s_con[0], s_con[1], s_con[2], s_r5_tot, s_tot[1], s_tot[2], s_tot[3]);
    println!("  S2a sums   | {:>7.1} {:>7.1} {:>7.1} {:>7.1}", s_2a[0], s_2a[1], s_2a[2], s_2a[3]);
    println!("  S2b sums   | {:>7.1} {:>7.1} {:>7.1} {:>7.1}", s_2b[0], s_2b[1], s_2b[2], s_2b[3]);
    println!("  S3a sums   | {:>7.1} {:>7.1} {:>7.1} {:>7.1}", s_3a[0], s_3a[1], s_3a[2], s_3a[3]);
    println!("  Ind/Base   |  1.000x  {:.3}x  {:.3}x  {:.3}x | Con/R5:  {:.3}x  {:.3}x  {:.3}x | Tot/R5:  {:.3}x  {:.3}x  {:.3}x",
        s_ind[1] / s_ind[0], s_ind[2] / s_ind[0], s_ind[3] / s_ind[0],
        s_con[0] / s_r5_con, s_con[1] / s_r5_con, s_con[2] / s_r5_con,
        s_tot[1] / s_r5_tot, s_tot[2] / s_r5_tot, s_tot[3] / s_r5_tot);
}

fn bench_multiworker(blocks: &[Block], label: &str, rounds: usize, iters_1w: usize, iters_mw: usize) {
    println!("\n--- Multi-Worker (1T / 2W / 4W, paired alternating {rounds} rounds, p50 wall ms) — {label} ---");
    for &workers in &[1usize, 2, 4] {
        let mut r5_ms = Vec::new();
        let mut b_ms = Vec::new();
        let mut i3_ms = Vec::new();
        let mut tp_ms = Vec::new();
        let mut hf_ms = Vec::new();
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for mode in 0..5 {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut fk = FKernel::default();
                            let mut out = Vec::new();
                            for blk in blocks {
                                match mode {
                                    0 => { r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out); }
                                    1 => { fk.run::<false, false, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                    2 => { fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                    3 => { fk.run::<true, false, true, false>(&blk.records, blk.n, blk.p, &mut out); }
                                    _ => { fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                }
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    match mode {
                                        0 => { r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out); }
                                        1 => { fk.run::<false, false, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                        2 => { fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                        3 => { fk.run::<true, false, true, false>(&blk.records, blk.n, blk.p, &mut out); }
                                        _ => { fk.run::<true, true, false, false>(&blk.records, blk.n, blk.p, &mut out); }
                                    }
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
                match mode {
                    0 => r5_ms.push(t_wall),
                    1 => b_ms.push(t_wall),
                    2 => i3_ms.push(t_wall),
                    3 => tp_ms.push(t_wall),
                    _ => hf_ms.push(t_wall),
                }
            }
        }
        let med = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let (mr5, mb, mi3, mtp, mhf) = (
            med(&mut r5_ms),
            med(&mut b_ms),
            med(&mut i3_ms),
            med(&mut tp_ms),
            med(&mut hf_ms),
        );
        println!("  {}W: R5={:7.2}ms | Base={:7.2}ms ({:.4}x) | F-Inline3={:7.2}ms ({:.4}x) | F-I3+TP={:7.2}ms ({:.4}x) | F-HierFast={:7.2}ms ({:.4}x)",
            workers, mr5, mb, mb / mr5, mi3, mi3 / mr5, mtp, mtp / mr5, mhf, mhf / mr5);
    }
}

fn main() {
    let wiki = load_six_admitted();
    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki = load_bin_blocks(
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        Some(&nonwiki_names),
        false,
        true,
    );

    run_exactness_suite(&wiki, &nonwiki);
    bench_detailed_1t(&wiki, "6 Admitted Wikidata Blocks");
    bench_detailed_1t(&nonwiki, "6 Non-Wikidata JSONL+Syslog Blocks");
    bench_multiworker(&wiki, "6 Admitted Wikidata Blocks", 11, 5, 4);
    bench_multiworker(&nonwiki, "6 Non-Wikidata JSONL+Syslog Blocks", 11, 5, 4);

    // Also verify & benchmark on all 233 full blocks of wikidata_431_blocks.bin!
    let w233 = load_bin_blocks(
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        None,
        true,
        false,
    );
    println!("\nLoaded {} full blocks from wikidata_431_blocks.bin; verifying 100% byte-exactness vs cand_r5_head...", w233.len());
    {
        let mut r5 = cand_r5_head::Kernel::default();
        let mut fk = FKernel::default();
        let mut o1 = Vec::new();
        let mut o2 = Vec::new();
        for blk in &w233 {
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut o1).unwrap();
            fk.run::<true, false, false, false>(&blk.records, blk.n, blk.p, &mut o2);
            assert_eq!(o1, o2, "F-Inline3 mismatch on {}", blk.name);
        }
        println!("  PASS: {}/{} full wikidata.bz2 blocks 100% BYTE-EXACT (F-Inline3 == cand_r5_head).", w233.len(), w233.len());
    }
    bench_multiworker(&w233, "233 Full Wikidata.bz2 Blocks", 5, 1, 1);
}
