#![forbid(unsafe_code)]

//! Cycle 48 P3 — Deepening Analogue F:
//!   (a) `F-Inline4-ASCII`: 4-bit `0xF` tag + 28-bit (4x7-bit) ASCII quadgram handle packing
//!       (`SATURATED = 239`, `+0 B` WS, exact general fallback for non-ASCII `>= 128`).
//!   (b) `F-Prefix7-2Word` / `F-Prefix8-2Word`: 2-word (`8 B`) arena slot for `4 <= g <= 7`
//!       (and `4 <= g <= 8`) vs 4-word (`16 B`) `Prefix15`, plus `F-I4+Prefix8`.
//!   (c) `S3a-Band32`, `S3a-OutTail64+TrimWS`, `S3a-OutTail32+TrimWS`: variable-length 32-bit
//!       `Band` encoding, live-range `out`-tail band placement (`0 B` persistent `bands`, guarded
//!       by the exact `bands_count <= open_nodes + 256` side-0 conservation bound), and stepped
//!       `TrimWS` growth (`8192 B` spelling, `256` inline16) eliminating `48 -> 96 KiB` doubling.
//!
//! All variants compile cleanly with `#![forbid(unsafe_code)] -D warnings` and strictly match
//! forward-`FL` (`cand_r5_head` / `libbzip2`) on valid, adversarial, and malformed multi-cycle blocks.

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
const MAX_RULE_WORDS: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

// Inline handle encoding modes:
const MODE_BASE: u8 = 0;    // cand_r5_head exact (ID_BITS=22, inline g=1..2, RULE_BASE=65_793)
const MODE_INLINE3: u8 = 1; // F-Inline3 (ID_BITS=24, inline g=1..3, RULE_BASE=0, SAT=255)
const MODE_INLINE4: u8 = 2; // F-Inline4-ASCII (ID_BITS=24, 0xF=4x7-bit ASCII, SAT=239)

// Band / WS modes:
const BAND_U64_DEFAULT: u8 = 0; // Exact cand_r5_head Band(u64) + 64K/48K/29.75K pre-reserves
const BAND_U32_TRIM: u8 = 1;    // S3a-Band32 in Vec<u32> + TrimWS
const BAND_OUTTAIL64: u8 = 2;   // S3a-OutTail64 (0 B persistent bands via out tail) + TrimWS
const BAND_OUTTAIL32: u8 = 3;   // S3a-OutTail32 (Band32 in out tail) + TrimWS

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
struct RunStats {
    rules_words: usize,
    arena_peak_words: usize,
    bands_bytes: usize,
    used_out_tail: bool,
    ws_total_bytes: usize,
    ws_s3a_bytes: usize,
    ws_bands_bytes: usize,
    ws_spelling_bytes: usize,
    ws_inline_bytes: usize,
    ws_edges_bytes: usize,
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

#[inline(always)]
fn reserve_step_u8(v: &mut Vec<u8>, add: usize, step: usize) {
    let need = v.len() + add;
    if need > v.capacity() {
        let target = ((need + step - 1) / step) * step;
        v.reserve_exact(target - v.len());
    }
}

#[inline(always)]
fn reserve_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
    if lens.len() == lens.capacity() {
        let target = ((lens.len() + 1 + step - 1) / step) * step;
        inline16.reserve_exact(target - inline16.len());
        lens.reserve_exact(target - lens.len());
    }
}

#[inline(always)]
fn reserve_step_u32(v: &mut Vec<u32>, add: usize, step: usize) {
    let need = v.len() + add;
    if need > v.capacity() {
        let target = ((need + step - 1) / step) * step;
        v.reserve_exact(target - v.len());
    }
}

// ============================================================================
// Handle Encoding Helpers (Base / F-Inline3 / F-Inline4-ASCII / Prefix2W)
// ============================================================================

#[inline(always)]
const fn id_bits<const MODE: u8>() -> u32 {
    if MODE == MODE_BASE { 22 } else { 24 }
}

#[inline(always)]
const fn id_mask<const MODE: u8>() -> u32 {
    (1u32 << id_bits::<MODE>()) - 1
}

#[inline(always)]
const fn saturated<const MODE: u8>() -> u32 {
    match MODE {
        MODE_BASE => u32::MAX >> 22, // 1023
        MODE_INLINE3 => 255,         // 0xFF
        _ => 239,                    // 0xEF (0xF0..=0xFF reserved for 4x7-bit ASCII quadgrams)
    }
}

#[inline(always)]
const fn rule_base<const MODE: u8>() -> u32 {
    if MODE == MODE_BASE { 65_793 } else { 0 }
}

#[inline(always)]
const fn max_inline_grade<const MODE: u8>() -> u32 {
    if MODE == MODE_BASE { 2 } else { 3 }
}

#[inline(always)]
const fn literal<const MODE: u8>(byte: u8) -> u32 {
    (1u32 << id_bits::<MODE>()) | (byte as u32)
}

/// Exact bijection `{0..127}^4` -> `[0xF000_0000, 0xFFFF_FFFF]`.
/// Caller guarantees `(v & 0x8080_8080) == 0`.
#[inline(always)]
const fn pack_ascii4(v: u32) -> u32 {
    0xF000_0000
        | (v & 0x7F)
        | ((v & 0x0000_7F00) >> 1)
        | ((v & 0x007F_0000) >> 2)
        | ((v & 0x7F00_0000) >> 3)
}

/// Exact inverse bijection `[0xF000_0000, 0xFFFF_FFFF]` -> 4 little-endian ASCII bytes in `u32`.
#[inline(always)]
const fn unpack_ascii4(h: u32) -> u32 {
    (h & 0x7F)
        | ((h & 0x0000_3F80) << 1)
        | ((h & 0x001F_C000) << 2)
        | ((h & 0x0FE0_0000) << 3)
}

#[inline(always)]
const fn grade_of<const MODE: u8>(h: u32) -> u32 {
    if MODE == MODE_INLINE4 && h >= 0xF000_0000 {
        4
    } else {
        h >> id_bits::<MODE>()
    }
}

#[inline(always)]
const fn rule_unit<const P2W_MAX: u32>() -> usize {
    if P2W_MAX > 0 { 2 } else { 4 }
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

struct Elim<'a, L: Links, const MODE: u8, const P2W_MAX: u32> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const MODE: u8, const P2W_MAX: u32> Elim<'_, L, MODE, P2W_MAX> {
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
        self.base + (id - rule_base::<MODE>()) as usize * rule_unit::<P2W_MAX>()
    }

    #[inline(always)]
    fn alloc_id(&self) -> u32 {
        ((self.top - self.base) / rule_unit::<P2W_MAX>()) as u32 + rule_base::<MODE>()
    }

    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = grade_of::<MODE>(h);
        if g < saturated::<MODE>() {
            g
        } else {
            self.a[self.rule(h & id_mask::<MODE>()) + 2]
        }
    }

    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        if MODE == MODE_INLINE4 && h >= 0xF000_0000 {
            return u128::from(unpack_ascii4(h));
        }
        let g = h >> id_bits::<MODE>();
        let id = h & id_mask::<MODE>();
        if g <= max_inline_grade::<MODE>() {
            return u128::from(id);
        }
        let r = self.rule(id);
        if P2W_MAX > 0 && g <= P2W_MAX {
            let sl = &self.a[r..r + 2];
            return u128::from(sl[0]) | (u128::from(sl[1]) << 32);
        }
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
        let left_len = grade_of::<MODE>(left);
        let sum = left_len + grade_of::<MODE>(right);
        if sum < INLINE as u32 {
            if sum <= max_inline_grade::<MODE>() {
                let lp = left & id_mask::<MODE>();
                let rp = right & id_mask::<MODE>();
                return (sum << id_bits::<MODE>()) | lp | (rp << (8 * left_len));
            }
            if MODE == MODE_INLINE4 && sum == 4 {
                // Both `left` and `right` have grade in 1..=3, so both are already inline in 24 bits!
                let v = (left & 0x00FF_FFFF) | ((right & 0x00FF_FFFF) << (8 * left_len));
                if (v & 0x8080_8080) == 0 {
                    return pack_ascii4(v);
                }
                // Exact general fallback for non-ASCII quadgrams (>= 128)
                let id = self.alloc_id();
                if P2W_MAX >= 4 {
                    let out = &mut self.a[self.top..self.top + 2];
                    out[0] = v;
                    out[1] = 0;
                    self.top += 2;
                } else {
                    let out = &mut self.a[self.top..self.top + 4];
                    out[0] = v;
                    out[1] = 0;
                    out[2] = 0;
                    out[3] = 4 << 24;
                    self.top += 4;
                }
                return (4u32 << 24) | id;
            }
            let payload =
                self.short_payload_h(left) | (self.short_payload_h(right) << (8 * left_len));
            let id = self.alloc_id();
            if P2W_MAX > 0 && sum <= P2W_MAX {
                let out = &mut self.a[self.top..self.top + 2];
                out[0] = payload as u32;
                out[1] = (payload >> 32) as u32;
                self.top += 2;
            } else {
                let bytes = payload | (u128::from(sum) << 120);
                let out = &mut self.a[self.top..self.top + 4];
                out[0] = bytes as u32;
                out[1] = (bytes >> 32) as u32;
                out[2] = (bytes >> 64) as u32;
                out[3] = (bytes >> 96) as u32;
                self.top += 4;
            }
            (sum << id_bits::<MODE>()) | id
        } else {
            let len = if sum <= saturated::<MODE>() {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = self.alloc_id();
            let out = &mut self.a[self.top..self.top + 4];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += 4;
            (len.min(saturated::<MODE>()) << id_bits::<MODE>()) | id
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
            if self.top + MAX_RULE_WORDS > self.a.len() {
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
        let room = if SPIRALS {
            SPIRAL_RULES * MAX_RULE_WORDS
        } else {
            MAX_RULE_WORDS
        };
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

    fn inline_word<const TRIM: bool>(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = grade_of::<MODE>(word) as usize;
        if height < INLINE {
            let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        } else {
            let start = spelling.len() as u32;
            self.append::<TRIM>(word, stack, spelling);
            if TRIM {
                reserve_step_u8(spelling, 1, 8192);
            }
            spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }

    fn append<const TRIM: bool>(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = grade_of::<MODE>(h);
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                if TRIM {
                    reserve_step_u8(dest, g as usize, 8192);
                }
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & id_mask::<MODE>());
                let tag = self.a[r + 3];
                let rule = &mut self.a[r..r + 4];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = rule[2] as usize;
                    if TRIM {
                        reserve_step_u8(dest, rlen, 8192);
                    }
                    dest.extend_from_within(start..start + rlen);
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
    #[inline(always)]
    fn code<L: Links, const MODE: u8, const P2W_MAX: u32, const TRIM: bool>(
        &mut self,
        e: &mut Elim<L, MODE, P2W_MAX>,
        handle: u32,
        byte: u8,
    ) -> u32 {
        if MODE != MODE_BASE {
            // Injective 40-bit key `(handle, byte)`:
            // Every valid non-zero handle has `handle >= 0x0100_0000 != 0` (both `0x01..=0xEF` and `0xF0..=0xFF`!),
            // so zero-initialized `pairs[slot] == 0` can NEVER collide with any valid handle.
            let bucket = ((handle.wrapping_mul(0x9e37_79b9)
                ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
                >> 24) as usize;
            let slot = bucket * 2;
            let val = self.pairs[slot + 1];
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
                return val & 0x00FF_FFFF;
            }
            let code = self.code_miss::<L, MODE, P2W_MAX, TRIM>(
                e,
                handle,
                handle & id_mask::<MODE>(),
                byte,
            );
            self.pairs[slot] = handle;
            self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
            code
        } else {
            let g = handle >> id_bits::<MODE>();
            let id = handle & id_mask::<MODE>();
            let uid = id + 1 + (u32::from(g == 2) << 8);
            let key = (uid << 8) | u32::from(byte);
            let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
            let slot = bucket * 2;
            if self.pairs[slot] == key {
                return self.pairs[slot + 1];
            }
            let code = self.code_miss::<L, MODE, P2W_MAX, TRIM>(e, handle, id, byte);
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            code
        }
    }

    fn code_miss<L: Links, const MODE: u8, const P2W_MAX: u32, const TRIM: bool>(
        &mut self,
        e: &mut Elim<L, MODE, P2W_MAX>,
        handle: u32,
        id: u32,
        byte: u8,
    ) -> u32 {
        let height = grade_of::<MODE>(handle) as usize;
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
            let c = self.insert::<TRIM>(entry, len);
            self.contents[index] = tag | c;
            return c;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append::<TRIM>(handle, self.stack, self.spelling);
        if TRIM {
            reserve_step_u8(self.spelling, 1, 8192);
        }
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
        let c = self.insert::<TRIM>(entry, LONG);
        self.contents[index] = tag | c;
        c
    }

    #[inline(always)]
    fn insert<const TRIM: bool>(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        if TRIM {
            reserve_step_inline(self.inline16, self.lens, 256);
        }
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

// ============================================================================
// Band Sink Abstractions (`Band64`, `Band32`, `OutTail64`, `OutTail32`)
// ============================================================================

#[inline(always)]
fn push_band32_vec(v: &mut Vec<u32>, width: u32, dst: u32, code: u32, is_literal: bool) {
    reserve_step_u32(v, 3, 4096);
    if is_literal && width <= 2048 {
        v.push(((width - 1) << 20) | dst);
    } else if width <= 2047 {
        v.push(0x8000_0000 | ((width - 1) << 20) | dst);
        v.push(code);
    } else {
        v.push(0xFFF0_0000 | dst);
        v.push(code);
        v.push(width);
    }
}

#[inline(always)]
fn push_band32_tail(tail: &mut [u8], pos: &mut usize, width: u32, dst: u32, code: u32, is_literal: bool) {
    let p = *pos;
    if is_literal && width <= 2048 {
        let w0 = ((width - 1) << 20) | dst;
        tail[p..p + 4].copy_from_slice(&w0.to_le_bytes());
        *pos = p + 4;
    } else if width <= 2047 {
        let w0 = 0x8000_0000 | ((width - 1) << 20) | dst;
        tail[p..p + 4].copy_from_slice(&w0.to_le_bytes());
        tail[p + 4..p + 8].copy_from_slice(&code.to_le_bytes());
        *pos = p + 8;
    } else {
        let w0 = 0xFFF0_0000 | dst;
        tail[p..p + 4].copy_from_slice(&w0.to_le_bytes());
        tail[p + 4..p + 8].copy_from_slice(&code.to_le_bytes());
        tail[p + 8..p + 12].copy_from_slice(&width.to_le_bytes());
        *pos = p + 12;
    }
}

#[inline(always)]
fn push_band64_tail(tail: &mut [u8], pos: &mut usize, width: u32, dst: u32, code: u32) {
    let p = *pos;
    let b = Band::new(width, dst, code).0;
    tail[p..p + 8].copy_from_slice(&b.to_le_bytes());
    *pos = p + 8;
}

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
    bands32: Vec<u32>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    predecessor: Option<u32>,
    force_wide: bool,
    force_spirals: Option<bool>,
    force_no_outtail: bool,
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
        (4 * ns + (stride + MAX_RULE_WORDS) * Self::nodes_most(ns)).max(m)
    }

    fn measure_ws(&self) -> (usize, usize, usize, usize, usize, usize) {
        let ws_bands = self.bands.capacity() * 8 + self.bands32.capacity() * 4;
        let ws_spelling = self.spelling.capacity();
        let ws_inline = self.inline16.capacity() * INLINE + self.lens.capacity();
        let ws_edges = self.edges.capacity() * 4 + self.extra.capacity() * 4;
        let ws_other = self.spill.scratch_bytes() + self.stack.capacity() * 4;
        let ws_s3a = ws_bands + ws_spelling + ws_inline;
        let ws_total = ws_s3a + ws_edges + ws_other;
        (ws_total, ws_s3a, ws_bands, ws_spelling, ws_inline, ws_edges)
    }

    #[inline(never)]
    fn run<
        const MODE: u8,
        const P2W_MAX: u32,
        const BAND_MODE: u8,
        const TIMED: bool,
    >(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> (RunStats, StageTimes) {
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
        self.bands32.clear();
        self.spelling.clear();
        if BAND_MODE == BAND_U64_DEFAULT {
            if n >= 65_536 && self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192 - self.bands.len());
            }
            if n >= 65_536 && self.spelling.capacity() < 48 * 1024 {
                self.spelling.reserve_exact(48 * 1024 - self.spelling.len());
            }
        }

        // Exact byte offset in `out` after OutQ16's chunks (zero extra state in outq.rs!)
        let tail_offset = if BAND_MODE == BAND_OUTTAIL64 || BAND_MODE == BAND_OUTTAIL32 {
            let mut chunks_taken = 0usize;
            for a in 0..256 {
                let ta = f.thin[a] as usize;
                if ta > 0 {
                    chunks_taken += (ta + 511) >> 9;
                }
            }
            let in_out_chunks = out.len() / 1024;
            chunks_taken.min(in_out_chunks) * 1024
        } else {
            out.len()
        };

        let mut stats = RunStats::default();
        let most = Self::nodes_most(ns);
        let is_wide = self.is_wide(ns);
        let (cursors, used_tail, tail_end) = if is_wide {
            self.induced_with::<Wide, MODE, P2W_MAX, BAND_MODE, TIMED>(
                ns,
                n as u32,
                f.m,
                &f.thin,
                &first,
                &mut write,
                &mut stats,
                most,
                &mut out[tail_offset..],
                &mut tm,
            )
        } else {
            self.induced_with::<Narrow, MODE, P2W_MAX, BAND_MODE, TIMED>(
                ns,
                n as u32,
                f.m,
                &f.thin,
                &first,
                &mut write,
                &mut stats,
                most,
                &mut out[tail_offset..],
                &mut tm,
            )
        };
        stats.used_out_tail = used_tail;

        // S4 wiring
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
        let use_xor = !wide && m >= 65_536;
        if use_xor {
            self.edges[..m as usize].fill(0);
        }
        let mut predecessor = f.mark;
        let mb = usize::from(f.mark_byte);
        let (out_q, out_tail) = out.split_at(tail_offset);

        match BAND_MODE {
            BAND_U64_DEFAULT => {
                self.wire_from_band64(
                    &cursors,
                    &heads,
                    out_q,
                    m as usize,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            }
            BAND_U32_TRIM => {
                self.wire_from_band32_vec(
                    &cursors,
                    &heads,
                    out_q,
                    m as usize,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            }
            BAND_OUTTAIL64 => {
                if used_tail {
                    self.wire_from_outtail64(
                        &cursors,
                        tail_end,
                        out_tail,
                        &heads,
                        out_q,
                        m as usize,
                        bits,
                        wide,
                        use_xor,
                        f.mark,
                        mb,
                        &mut predecessor,
                    );
                } else {
                    self.wire_from_band64(
                        &cursors,
                        &heads,
                        out_q,
                        m as usize,
                        bits,
                        wide,
                        use_xor,
                        f.mark,
                        mb,
                        &mut predecessor,
                    );
                }
            }
            _ => {
                // BAND_OUTTAIL32
                if used_tail {
                    self.wire_from_outtail32(
                        &cursors,
                        tail_end,
                        out_tail,
                        &heads,
                        out_q,
                        m as usize,
                        bits,
                        wide,
                        use_xor,
                        f.mark,
                        mb,
                        &mut predecessor,
                    );
                } else {
                    self.wire_from_band32_vec(
                        &cursors,
                        &heads,
                        out_q,
                        m as usize,
                        bits,
                        wide,
                        use_xor,
                        f.mark,
                        mb,
                        &mut predecessor,
                    );
                }
            }
        }
        if use_xor {
            self.predecessor = Some(predecessor);
        }
        if let Some(t0) = t_s4 {
            tm.s4_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        // S5 emit
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
        let (ws_tot, ws_s3a, ws_b, ws_sp, ws_in, ws_ed) = self.measure_ws();
        stats.ws_total_bytes = ws_tot;
        stats.ws_s3a_bytes = ws_s3a;
        stats.ws_bands_bytes = ws_b;
        stats.ws_spelling_bytes = ws_sp;
        stats.ws_inline_bytes = ws_in;
        stats.ws_edges_bytes = ws_ed;
        (stats, tm)
    }

    #[allow(clippy::too_many_arguments)]
    fn wire_from_band64(
        &mut self,
        cursors: &[usize; 256],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
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
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn wire_from_band32_vec(
        &mut self,
        cursors: &[usize; 256],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let b32 = &self.bands32;
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(b32.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let lit_pack = (a as u32) << bits;
                let mut bi = cursors[a];
                if a != mb {
                    while bi < end {
                        let w0 = b32[bi];
                        let dst = (w0 & 0xF_FFFF) as usize;
                        if w0 < 0x8000_0000 {
                            bi += 1;
                            let span = ((w0 >> 20) + 1) as usize;
                            queue.wire_xor_nomark(edges, dst, span, lit_pack);
                        } else {
                            let rw = (w0 >> 20) & 0x7FF;
                            let code = b32[bi + 1];
                            let span = if rw < 2047 {
                                bi += 2;
                                (rw + 1) as usize
                            } else {
                                let sp = b32[bi + 2] as usize;
                                bi += 3;
                                sp
                            };
                            queue.wire_xor_nomark(edges, dst, span, code << bits);
                        }
                    }
                } else {
                    while bi < end {
                        let w0 = b32[bi];
                        let dst = (w0 & 0xF_FFFF) as usize;
                        if w0 < 0x8000_0000 {
                            bi += 1;
                            let span = ((w0 >> 20) + 1) as usize;
                            queue.wire_xor(edges, dst, span, lit_pack, mark, predecessor);
                        } else {
                            let rw = (w0 >> 20) & 0x7FF;
                            let code = b32[bi + 1];
                            let span = if rw < 2047 {
                                bi += 2;
                                (rw + 1) as usize
                            } else {
                                let sp = b32[bi + 2] as usize;
                                bi += 3;
                                sp
                            };
                            queue.wire_xor(edges, dst, span, code << bits, mark, predecessor);
                        }
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(b32.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let mut bi = cursors[a];
                while bi < end {
                    let w0 = b32[bi];
                    let dst = (w0 & 0xF_FFFF) as usize;
                    let (span, bcode) = if w0 < 0x8000_0000 {
                        bi += 1;
                        (((w0 >> 20) + 1) as usize, a as u32)
                    } else {
                        let rw = (w0 >> 20) & 0x7FF;
                        let c = b32[bi + 1];
                        if rw < 2047 {
                            bi += 2;
                            ((rw + 1) as usize, c)
                        } else {
                            let sp = b32[bi + 2] as usize;
                            bi += 3;
                            (sp, c)
                        }
                    };
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn wire_from_outtail64(
        &mut self,
        cursors: &[usize; 256],
        tail_end: usize,
        out_tail: &[u8],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let sl = &out_tail[cursors[a]..end];
                if a != mb {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for ch in sl.chunks_exact(8) {
                        let band = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            predecessor,
                        );
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                for ch in out_tail[cursors[a]..end].chunks_exact(8) {
                    let b = Band(u64::from_le_bytes(ch.try_into().unwrap()));
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn wire_from_outtail32(
        &mut self,
        cursors: &[usize; 256],
        tail_end: usize,
        out_tail: &[u8],
        heads: &outq::Heads,
        out_q: &[u8],
        m: usize,
        bits: u32,
        wide: bool,
        use_xor: bool,
        mark: u32,
        mb: usize,
        predecessor: &mut u32,
    ) {
        let edges = &mut self.edges[..m];
        let rd32 = |off: usize| u32::from_le_bytes(out_tail[off..off + 4].try_into().unwrap());
        if use_xor {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let lit_pack = (a as u32) << bits;
                let mut bi = cursors[a];
                if a != mb {
                    while bi < end {
                        let w0 = rd32(bi);
                        let dst = (w0 & 0xF_FFFF) as usize;
                        if w0 < 0x8000_0000 {
                            bi += 4;
                            let span = ((w0 >> 20) + 1) as usize;
                            queue.wire_xor_nomark(edges, dst, span, lit_pack);
                        } else {
                            let rw = (w0 >> 20) & 0x7FF;
                            let code = rd32(bi + 4);
                            let span = if rw < 2047 {
                                bi += 8;
                                (rw + 1) as usize
                            } else {
                                let sp = rd32(bi + 8) as usize;
                                bi += 12;
                                sp
                            };
                            queue.wire_xor_nomark(edges, dst, span, code << bits);
                        }
                    }
                } else {
                    while bi < end {
                        let w0 = rd32(bi);
                        let dst = (w0 & 0xF_FFFF) as usize;
                        if w0 < 0x8000_0000 {
                            bi += 4;
                            let span = ((w0 >> 20) + 1) as usize;
                            queue.wire_xor(edges, dst, span, lit_pack, mark, predecessor);
                        } else {
                            let rw = (w0 >> 20) & 0x7FF;
                            let code = rd32(bi + 4);
                            let span = if rw < 2047 {
                                bi += 8;
                                (rw + 1) as usize
                            } else {
                                let sp = rd32(bi + 8) as usize;
                                bi += 12;
                                sp
                            };
                            queue.wire_xor(edges, dst, span, code << bits, mark, predecessor);
                        }
                    }
                }
            }
        } else {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(tail_end);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                let mut bi = cursors[a];
                while bi < end {
                    let w0 = rd32(bi);
                    let dst = (w0 & 0xF_FFFF) as usize;
                    let (span, bcode) = if w0 < 0x8000_0000 {
                        bi += 4;
                        (((w0 >> 20) + 1) as usize, a as u32)
                    } else {
                        let rw = (w0 >> 20) & 0x7FF;
                        let c = rd32(bi + 4);
                        if rw < 2047 {
                            bi += 8;
                            ((rw + 1) as usize, c)
                        } else {
                            let sp = rd32(bi + 8) as usize;
                            bi += 12;
                            (sp, c)
                        }
                    };
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<
        L: Links,
        const MODE: u8,
        const P2W_MAX: u32,
        const BAND_MODE: u8,
        const TIMED: bool,
    >(
        &mut self,
        ns: usize,
        n: u32,
        m: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut RunStats,
        most: usize,
        out_tail: &mut [u8],
        tm: &mut StageTimes,
    ) -> ([usize; 256], bool, usize) {
        let t_s2a = if TIMED { Some(Instant::now()) } else { None };
        let nodes_words = L::STRIDE * most;
        assert!(self.edges.len() >= 4 * ns + nodes_words);

        let mut widest = 0u32;
        let (base, mut top, nodes, open_nodes) = {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L, MODE, P2W_MAX> {
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
                e.set_word(sid, literal::<MODE>(byte));
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
            let open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, base, e.nodes, open_nodes)
        };
        if let Some(t0) = t_s2a {
            tm.s2a_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s2b = if TIMED { Some(Instant::now()) } else { None };
        let spirals = self.force_spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        let mut remaining = n;
        loop {
            let mut e = Elim::<L, MODE, P2W_MAX> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            };
            top = e.top;
            if done {
                break;
            }
            grow(&mut self.edges, 4 * ns + top + SPIRAL_RULES * MAX_RULE_WORDS);
        }
        stats.rules_words = top - base;
        stats.arena_peak_words = 4 * ns + top;
        if let Some(t0) = t_s2b {
            tm.s2b_us = t0.elapsed().as_secs_f64() * 1e6;
        }

        let t_s3a = if TIMED { Some(Instant::now()) } else { None };
        {
            let mut e = Elim::<L, MODE, P2W_MAX> {
                a: &mut self.edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(m);
        }
        const PAIR_WORDS: usize = 512;
        const CACHE_WORDS: usize = PAIR_WORDS + 2048;
        let (header, arena) = self.edges.split_at_mut(4 * ns);
        let (live, tail_arena) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS {
            header
        } else {
            tail_arena
        };
        let mut e = Elim::<L, MODE, P2W_MAX> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        let init_words = if BAND_MODE == BAND_U64_DEFAULT && n >= 65_536 {
            1536
        } else {
            0
        };
        begin_inline(init_words, &mut self.inline16, &mut self.lens);

        // Exact mathematical upper bound on bands count: open_nodes + 256!
        let max_bands = open_nodes + 256;
        let use_tail = !self.force_no_outtail
            && match BAND_MODE {
                BAND_OUTTAIL64 => out_tail.len() >= max_bands * 8,
                BAND_OUTTAIL32 => out_tail.len() >= max_bands * 12,
                _ => false,
            };

        let mut cursors = [0usize; 256];
        let mut tail_pos = 0usize;
        let mut bands_count = 0usize;

        // Dispatch helper macro/closure-free loop for maximum LLVM codegen quality
        if cache.len() < CACHE_WORDS {
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = match BAND_MODE {
                    BAND_U64_DEFAULT => self.bands.len(),
                    BAND_U32_TRIM => self.bands32.len(),
                    BAND_OUTTAIL64 => {
                        if use_tail { tail_pos } else { self.bands.len() }
                    }
                    _ => {
                        if use_tail { tail_pos } else { self.bands32.len() }
                    }
                };
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let is_lit = node_word == 0;
                    let code = if is_lit {
                        a as u32
                    } else {
                        let (entry, len) = if BAND_MODE == BAND_U64_DEFAULT {
                            e.inline_word::<false>(
                                node_word,
                                a as u8,
                                &mut self.stack,
                                &mut self.spelling,
                            )
                        } else {
                            e.inline_word::<true>(
                                node_word,
                                a as u8,
                                &mut self.stack,
                                &mut self.spelling,
                            )
                        };
                        if BAND_MODE != BAND_U64_DEFAULT {
                            reserve_step_inline(&mut self.inline16, &mut self.lens, 256);
                        }
                        let c = self.lens.len() as u32;
                        self.inline16.push(entry);
                        self.lens.push(len);
                        c
                    };
                    bands_count += 1;
                    match BAND_MODE {
                        BAND_U64_DEFAULT => self.bands.push(Band::new(width, dst, code)),
                        BAND_U32_TRIM => {
                            push_band32_vec(&mut self.bands32, width, dst, code, is_lit)
                        }
                        BAND_OUTTAIL64 => {
                            if use_tail {
                                push_band64_tail(out_tail, &mut tail_pos, width, dst, code);
                            } else {
                                self.bands.push(Band::new(width, dst, code));
                            }
                        }
                        _ => {
                            if use_tail {
                                push_band32_tail(
                                    out_tail,
                                    &mut tail_pos,
                                    width,
                                    dst,
                                    code,
                                    is_lit,
                                );
                            } else {
                                push_band32_vec(&mut self.bands32, width, dst, code, is_lit);
                            }
                        }
                    }
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
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
            if BAND_MODE == BAND_OUTTAIL64 && use_tail {
                let mut chunks = out_tail[..max_bands * 8].chunks_exact_mut(8);
                for a in 0..256 {
                    cursors[a] = tail_pos;
                    let end = k + thin[a];
                    let mut last_w = 0u32;
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
                            let c = dictionary
                                .code::<L, MODE, P2W_MAX, true>(&mut e, node_word, a as u8);
                            last_w = node_word;
                            last_c = c;
                            c
                        };
                        bands_count += 1;
                        let slot = chunks.next().unwrap();
                        slot.copy_from_slice(&Band::new(width, dst, code).0.to_le_bytes());
                        tail_pos += 8;
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                            if id != 0 {
                                (next_id, node_bottom, node_width, node_word) =
                                    L::band_node(e.a, id);
                            }
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    cursors[a] = match BAND_MODE {
                        BAND_U64_DEFAULT | BAND_OUTTAIL64 => self.bands.len(),
                        BAND_U32_TRIM => self.bands32.len(),
                        _ => {
                            if use_tail { tail_pos } else { self.bands32.len() }
                        }
                    };
                    let end = k + thin[a];
                    // CRITICAL: sentinel MUST be 0 (not u32::MAX!), because in F-Inline4-ASCII
                    // `pack_ascii4([0x7F, 0x7F, 0x7F, 0x7F]) == 0xFFFF_FFFF == u32::MAX`!
                    // Since `node_word == 0` is handled in the first branch, `last_w = 0` can never match.
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        assert!(width > 0);
                        let dst = node_bottom + in_node;
                        let is_lit = node_word == 0;
                        let code = if is_lit {
                            a as u32
                        } else if node_word == last_w {
                            last_c
                        } else {
                            let c = if BAND_MODE == BAND_U64_DEFAULT {
                                dictionary
                                    .code::<L, MODE, P2W_MAX, false>(&mut e, node_word, a as u8)
                            } else {
                                dictionary
                                    .code::<L, MODE, P2W_MAX, true>(&mut e, node_word, a as u8)
                            };
                            last_w = node_word;
                            last_c = c;
                            c
                        };
                        bands_count += 1;
                        match BAND_MODE {
                            BAND_U64_DEFAULT | BAND_OUTTAIL64 => {
                                self.bands.push(Band::new(width, dst, code))
                            }
                            BAND_U32_TRIM => {
                                push_band32_vec(&mut self.bands32, width, dst, code, is_lit)
                            }
                            _ => {
                                if use_tail {
                                    push_band32_tail(
                                        out_tail,
                                        &mut tail_pos,
                                        width,
                                        dst,
                                        code,
                                        is_lit,
                                    );
                                } else {
                                    push_band32_vec(&mut self.bands32, width, dst, code, is_lit);
                                }
                            }
                        }
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                            if id != 0 {
                                (next_id, node_bottom, node_width, node_word) =
                                    L::band_node(e.a, id);
                            }
                        }
                    }
                }
            }
            assert_eq!(id, 0);
        }
        assert!(bands_count <= max_bands, "bands_count {bands_count} > max_bands {max_bands}");
        stats.bands_bytes = if use_tail {
            tail_pos
        } else if BAND_MODE == BAND_U32_TRIM || BAND_MODE == BAND_OUTTAIL32 {
            self.bands32.len() * 4
        } else {
            self.bands.len() * 8
        };
        if let Some(t0) = t_s3a {
            tm.s3a_us = t0.elapsed().as_secs_f64() * 1e6;
        }
        (cursors, use_tail, tail_pos)
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
// Corpora & Exactness Verification Helpers
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

fn load_bin_blocks(
    path: &str,
    names: Option<&[&str]>,
    only_full: bool,
    with_expected: bool,
) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::new();
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
        let expected = if with_expected {
            let l = expand_records(&records, n);
            forward_fl_invert(&l, p)
        } else {
            Vec::new()
        };
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
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
// Adversarial Exactness Suite
// ============================================================================

fn test_dictionary_injectivity() {
    // Exhaustive proof test: ALL 256 bytes x boundary handles (0x00, 0x7F, 0x80, 0xFF, rule_id=0,
    // ASCII quadgram 0xF000_0000 & 0xFFFF_FFFF, non-ASCII quadgram 0x0400_0000, Prefix7/8, Prefix15,
    // Binary g=16, 238, 239, 240, 255, 512) under BOTH P2W_MAX=0 and P2W_MAX=8!
    fn check_mode<const P2W_MAX: u32>() {
        let mut arena = vec![0u32; 4096];
        let mut e = Elim::<Narrow, MODE_INLINE4, P2W_MAX> {
            a: &mut arena,
            base: 0,
            top: 0,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        // Construct adversarial handles:
        let make_word = |e: &mut Elim<Narrow, MODE_INLINE4, P2W_MAX>, bs: &[u8]| -> u32 {
            let mut h = 0u32;
            for &b in bs {
                h = e.join(h, literal::<MODE_INLINE4>(b));
            }
            h
        };
        // First arena rule MUST have rule_id = 0 and be non-ASCII g=4!
        let h_nonascii4_id0 = make_word(&mut e, &[0x80, 0x00, 0x00, 0x00]);
        assert_eq!(h_nonascii4_id0, 0x0400_0000, "first non-ASCII g=4 rule must be 0x0400_0000");

        let h_q4_pow = make_word(&mut e, b"ab\x00\x7f");
        let h_q4_512 = e.power(h_q4_pow, 128);
        let h_p16 = e.power(literal::<MODE_INLINE4>(0x00), 16);
        let h_p238 = e.power(literal::<MODE_INLINE4>(0x7F), 238);
        let h_p239 = e.power(literal::<MODE_INLINE4>(0x80), 239);
        let h_p240 = e.power(literal::<MODE_INLINE4>(0xFF), 240);
        let cases: Vec<(u32, Vec<u8>)> = vec![
            (literal::<MODE_INLINE4>(0x00), vec![0x00]),
            (literal::<MODE_INLINE4>(0x7F), vec![0x7F]),
            (literal::<MODE_INLINE4>(0x80), vec![0x80]),
            (literal::<MODE_INLINE4>(0xFF), vec![0xFF]),
            (make_word(&mut e, &[0x00, 0x00]), vec![0x00, 0x00]),
            (make_word(&mut e, &[0x7F, 0x7F]), vec![0x7F, 0x7F]),
            (make_word(&mut e, &[0xFF, 0xFF]), vec![0xFF, 0xFF]),
            (make_word(&mut e, &[0x00, 0x00, 0x00]), vec![0x00, 0x00, 0x00]),
            (make_word(&mut e, &[0x7F, 0x7F, 0x7F]), vec![0x7F, 0x7F, 0x7F]),
            (make_word(&mut e, &[0xFF, 0xFF, 0xFF]), vec![0xFF, 0xFF, 0xFF]),
            // ASCII quadgrams: min (0xF000_0000) and max (0xFFFF_FFFF = u32::MAX)!
            (make_word(&mut e, &[0x00, 0x00, 0x00, 0x00]), vec![0x00, 0x00, 0x00, 0x00]),
            (make_word(&mut e, &[0x7F, 0x7F, 0x7F, 0x7F]), vec![0x7F, 0x7F, 0x7F, 0x7F]),
            (make_word(&mut e, b"\"id\""), b"\"id\"".to_vec()),
            // Non-ASCII quadgrams (every byte position >= 128!)
            (h_nonascii4_id0, vec![0x80, 0x00, 0x00, 0x00]),
            (make_word(&mut e, &[0x00, 0x80, 0x00, 0x00]), vec![0x00, 0x80, 0x00, 0x00]),
            (make_word(&mut e, &[0x00, 0x00, 0x80, 0x00]), vec![0x00, 0x00, 0x80, 0x00]),
            (make_word(&mut e, &[0x00, 0x00, 0x00, 0x80]), vec![0x00, 0x00, 0x00, 0x80]),
            (make_word(&mut e, &[0xFF, 0xFE, 0x80, 0xC3]), vec![0xFF, 0xFE, 0x80, 0xC3]),
            // Joins of ASCII quadgram + non-ASCII quadgram (g=5..=15)
            (make_word(&mut e, &[0x7F, 0x7F, 0x7F, 0x7F, 0x80]), vec![0x7F, 0x7F, 0x7F, 0x7F, 0x80]),
            (make_word(&mut e, &[0x00, 0x00, 0x00, 0x00, 0xFF, 0x80, 0x7F]), vec![0x00, 0x00, 0x00, 0x00, 0xFF, 0x80, 0x7F]),
            (make_word(&mut e, &[0x01, 0x02, 0x03, 0x04, 0x80, 0x81, 0x82, 0xFF]), vec![0x01, 0x02, 0x03, 0x04, 0x80, 0x81, 0x82, 0xFF]),
            (make_word(&mut e, &(241..=255u8).collect::<Vec<_>>()), (241..=255u8).collect()),
            // Binary rules crossing SATURATED = 239 boundary!
            (h_p16, vec![0x00; 16]),
            (h_p238, vec![0x7F; 238]),
            (h_p239, vec![0x80; 239]),
            (h_p240, vec![0xFF; 240]),
            (h_q4_512, b"ab\x00\x7f".repeat(128)),
        ];
        // Verify exact handle bit tags for ASCII quadgrams
        assert_eq!(cases[10].0, 0xF000_0000);
        assert_eq!(cases[11].0, 0xFFFF_FFFF);
        assert!(cases[12].0 >= 0xF000_0000);
        for &(h, _) in &cases {
            assert!(h >= 0x0100_0000, "handle {h:#x} violated >= 0x0100_0000 invariant");
        }

        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut inline16 = Vec::new();
        let mut lens = Vec::new();
        let mut spelling = Vec::new();
        let mut stack = Vec::new();
        begin_inline(0, &mut inline16, &mut lens);
        let mut dict = Dictionary {
            pairs: &mut pairs,
            contents: &mut contents,
            inline16: &mut inline16,
            lens: &mut lens,
            spelling: &mut spelling,
            stack: &mut stack,
        };
        for byte in 0..=255u8 {
            for (h, word) in &cases {
                let code = dict.code::<Narrow, MODE_INLINE4, P2W_MAX, true>(&mut e, *h, byte);
                // Re-query immediately (pairs cache hit!)
                let code2 = dict.code::<Narrow, MODE_INLINE4, P2W_MAX, true>(&mut e, *h, byte);
                assert_eq!(code, code2);
                let mut want = word.clone();
                want.push(byte);
                let got = if dict.lens[code as usize] != LONG {
                    dict.inline16[code as usize][..dict.lens[code as usize] as usize].to_vec()
                } else {
                    long_word(&dict.inline16[code as usize], dict.spelling).to_vec()
                };
                assert_eq!(got, want, "Dictionary::code mismatch for h={h:#x}, byte={byte:#x}");
            }
        }
    }
    check_mode::<0>();
    check_mode::<7>();
    check_mode::<8>();
    println!("  PASS: Dictionary::code 40-bit `(handle, byte)` injectivity across 256 bytes x 27 boundary handles (incl. 0xF000_0000, 0xFFFF_FFFF, 0x0400_0000, SAT=239) x P2W={{0,7,8}}.");
}

fn run_exactness_suite(wiki6: &[Block], nonwiki6: &[Block], wiki233: &[Block]) {
    println!("================================================================================");
    println!("EXACTNESS & ADVERSARIAL VERIFICATION SUITE (ALL 8 VARIANTS vs cand_r5_head & FL)");
    println!("================================================================================");
    test_dictionary_injectivity();

    let mut r5 = cand_r5_head::Kernel::default();
    let mut fk = FKernel::default();
    let mut out_r5 = Vec::new();
    let mut out_v = Vec::new();

    let mut check_all = |l: &[u8], p: u32, tag: &str| {
        let recs = records_of(l);
        let n = l.len();
        let fl_exp = forward_fl_invert(l, p);
        r5.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_r5)
            .unwrap();
        assert_eq!(out_r5, fl_exp, "cand_r5_head vs FL mismatch on {tag}");

        fk.run::<MODE_INLINE3, 0, BAND_U64_DEFAULT, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-Inline3 mismatch on {tag}");

        fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-Inline4-ASCII mismatch on {tag}");

        fk.run::<MODE_INLINE3, 7, BAND_U64_DEFAULT, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-Prefix7-2Word mismatch on {tag}");

        fk.run::<MODE_INLINE3, 8, BAND_U64_DEFAULT, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-Prefix8-2Word mismatch on {tag}");

        fk.run::<MODE_INLINE4, 8, BAND_U64_DEFAULT, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-I4+Prefix8 mismatch on {tag}");

        fk.run::<MODE_INLINE4, 0, BAND_U32_TRIM, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-I4+Band32 mismatch on {tag}");

        fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-I4+OutTail64 mismatch on {tag}");

        fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL32, false>(&recs, n, p, &mut out_v);
        assert_eq!(out_v, out_r5, "F-I4+OutTail32 mismatch on {tag}");
    };

    // 1. 5,590+ small binary + synthetic thick-strip cases (including 0x00, 0x7F, 0x80, 0xFF alphabets!)
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
    let mut rng = 0xC48F_0048_DEAD_BEEFu64;
    let next_u64 = |s: &mut u64| -> u64 {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *s
    };
    let alphabets: [&[u8]; 4] = [
        b"abcdef",
        &[0x00, 0x7F, 0x01, 0x7E, 0x41, 0x00],          // 0x00 & 0x7F heavy (0xF000_0000 & 0xFFFF_FFFF!)
        &[0x80, 0xC3, 0xE2, 0xF0, 0xFE, 0xFF],          // 100% non-ASCII forced fallback!
        &[0x00, 0x7F, 0x80, 0xFF, b'a', 0xC3],          // Mixed ASCII + non-ASCII!
    ];
    for trial in 0..1600usize {
        let alph = alphabets[trial % alphabets.len()];
        let sigma = 2 + (next_u64(&mut rng) as usize % (alph.len() - 1));
        let n_runs = 4 + (next_u64(&mut rng) as usize % 18);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = alph[next_u64(&mut rng) as usize % sigma];
            let w = if next_u64(&mut rng) % 3 == 0 {
                17 + (next_u64(&mut rng) as usize % 35)
            } else {
                1 + (next_u64(&mut rng) as usize % 16)
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("synth-thick-{trial}"));
        small_cnt += 1;
    }
    println!("  PASS: {small_cnt} small, 0x00/0x7F, pure-non-ASCII, & mixed thick-strip cases 100% BYTE-EXACT.");

    // 2. 300+ malformed multi-cycle / pure-thick-cycle columns across ASCII & non-ASCII
    let mut mal_cnt = 0usize;
    for &(b0, b1, b2) in &[
        (0x00u8, 0x01u8, 0x02u8),
        (0x7F, 0x7F, 0x00),
        (0x80, 0xFE, 0xFF),
        (0x00, 0x7F, 0xFF),
    ] {
        for &w in &[17usize, 20, 32, 64] {
            let mut l = Vec::new();
            l.push(b2);
            l.resize(l.len() + w, b1);
            l.resize(l.len() + w, b0);
            for p in [0u32, 1, w as u32, (2 * w) as u32] {
                check_all(&l, p, "malformed-thick-cycle");
                mal_cnt += 1;
            }
        }
    }
    for trial in 0..260usize {
        let alph = alphabets[trial % alphabets.len()];
        let sigma = 2 + (next_u64(&mut rng) as usize % (alph.len() - 1));
        let n_runs = 6 + (next_u64(&mut rng) as usize % 20);
        let mut l = Vec::new();
        for _ in 0..n_runs {
            let b = alph[next_u64(&mut rng) as usize % sigma];
            let w = match next_u64(&mut rng) % 4 {
                0 => 20,
                1 => 40,
                2 => 1 + (next_u64(&mut rng) as usize % 5),
                _ => 17 + (next_u64(&mut rng) as usize % 25),
            };
            l.resize(l.len() + w, b);
        }
        let p = (next_u64(&mut rng) as usize % l.len()) as u32;
        check_all(&l, p, &format!("malformed-mc-{trial}"));
        mal_cnt += 1;
    }
    println!("  PASS: {mal_cnt} malformed multi-cycle columns (incl. 0x00/0x7F/0x80/0xFF) 100% BYTE-EXACT vs cand_r5_head & forward-FL.");

    // 3. SATURATED = 239 exact boundary spirals (len 235..=260, 512, 200_000) + forced non-ASCII large block
    //    + Band32 width > 2048 3-word escape + forced OutTail fallback + force_wide + force_spirals!
    for &slen in &[235usize, 238, 239, 240, 241, 254, 255, 256, 512] {
        let mut sp = vec![0x7Fu8; slen * 20];
        sp[0] = 0x00;
        sp[slen * 20 - 1] = 0x80;
        check_all(&sp, 0, &format!("spiral-sat-{slen}"));
    }
    // 200k spiral + wide-band > 2048 escape (a run of 5000 thin alternating rows inside a single gap!)
    let mut spiral200 = vec![0x7Fu8; 200_000];
    spiral200[0] = 0x00;
    spiral200[199_999] = 0xFF;
    check_all(&spiral200, 0, "spiral-200k-0x7F-0xFF");

    // Large 120k UTF-8 / binary non-ASCII block with thick runs triggering g=4 non-ASCII fallback & wide bands
    let mut bin_l = Vec::with_capacity(120_000);
    for step in 0..4000usize {
        let b = match step % 8 {
            0 => 0x00,
            1 => 0x7F,
            2 => 0x80,
            3 => 0xC3,
            4 => 0xE2,
            5 => 0xFF,
            6 => b'a',
            _ => 0xFE,
        };
        let w = if step % 5 == 0 { 20 + (step % 40) } else { 1 + (step % 15) };
        bin_l.resize(bin_l.len() + w, b);
    }
    // Append a >3000 thin-row section of alternating 0x00/0x7F to exercise Band32 >2048 3-word escape
    for _ in 0..2500 {
        bin_l.push(0x00);
        bin_l.push(0x7F);
    }
    check_all(&bin_l, 1234, "forced-nonascii-binary-wideband-125k");

    // Forced fallbacks on wiki6[0]: force_wide, force_spirals, force_no_outtail
    for (label, fw, fs, fno) in [
        ("force_wide", true, None, false),
        ("force_spirals=true", false, Some(true), false),
        ("force_spirals=false", false, Some(false), false),
        ("force_no_outtail", false, None, true),
    ] {
        fk.force_wide = fw;
        fk.force_spirals = fs;
        fk.force_no_outtail = fno;
        fk.run::<MODE_INLINE4, 8, BAND_OUTTAIL32, false>(&wiki6[0].records, wiki6[0].n, wiki6[0].p, &mut out_v);
        assert_eq!(out_v, wiki6[0].expected, "fallback {label} mismatch (I4+P8+OutTail32)");
        fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, false>(&wiki6[0].records, wiki6[0].n, wiki6[0].p, &mut out_v);
        assert_eq!(out_v, wiki6[0].expected, "fallback {label} mismatch (I4+OutTail64)");
    }
    fk.force_wide = false;
    fk.force_spirals = None;
    fk.force_no_outtail = false;
    println!("  PASS: SATURATED=239 spirals (235..=512, 200k) + forced 125k non-ASCII/0x00/0x7F/wide-band + force_wide/force_spirals/force_no_outtail BYTE-EXACT.");

    // 3b. Explicit forced non-ASCII g==4 fallback firing test (ASCII twin vs non-ASCII twin)
    //     + OutTail chunk/epoch/overflow/0xAA-poison aliasing test!
    {
        // Construct twin columns that force g=4 thick-strip joins:
        // 5 distinct thick strips (w=20..30) cycling in order so S2b creates g=4 rules.
        let make_twin = |alph: [u8; 5]| -> Vec<u8> {
            let mut l = Vec::new();
            l.push(alph[4]);
            for (k, &w) in [21usize, 23, 25, 27, 29].iter().enumerate() {
                l.resize(l.len() + w, alph[k]);
                l.push(alph[(k + 1) % 5]);
            }
            l
        };
        let l_asc = make_twin([b'a', b'b', b'c', b'd', b'e']);
        let l_non = make_twin([0x80, 0x91, 0xC3, 0xE2, 0xFF]);
        let r_asc = records_of(&l_asc);
        let r_non = records_of(&l_non);
        let mut out_a = vec![0xAAu8; l_asc.len() + 64];
        let mut out_b = vec![0xAAu8; l_non.len() + 64];
        let (st_i3_a, _) = fk.run::<MODE_INLINE3, 0, BAND_U64_DEFAULT, false>(&r_asc, l_asc.len(), 0, &mut out_a);
        let (st_i4_a, _) = fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, false>(&r_asc, l_asc.len(), 0, &mut out_a);
        assert_eq!(out_a, forward_fl_invert(&l_asc, 0));
        let (st_i3_n, _) = fk.run::<MODE_INLINE3, 0, BAND_U64_DEFAULT, false>(&r_non, l_non.len(), 0, &mut out_b);
        let (st_i4_n, _) = fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, false>(&r_non, l_non.len(), 0, &mut out_b);
        assert_eq!(out_b, forward_fl_invert(&l_non, 0));
        assert!(
            st_i4_a.rules_words < st_i3_a.rules_words,
            "expected ASCII twin to eliminate g=4 rules: i4={} vs i3={}",
            st_i4_a.rules_words,
            st_i3_a.rules_words
        );
        assert_eq!(
            st_i4_n.rules_words, st_i3_n.rules_words,
            "expected non-ASCII twin to fall back 100% to arena: i4={} vs i3={}",
            st_i4_n.rules_words,
            st_i3_n.rules_words
        );

        // OutTail chunk boundaries (thin[a]=511, 512, 513), epoch boundary (m=65_535..65_537),
        // and OutQ overflow (high-m -> out_tail_free == 0 -> exact fallback to self.bands/bands32),
        // all with 0xAA pre-poisoned `out`!
        let mut boundary_cases: Vec<(String, Vec<u8>)> = Vec::new();
        for &target_thin in &[511usize, 512, 513] {
            let mut l = Vec::new();
            for _ in 0..target_thin {
                l.push(b'A');
                l.push(b'B');
            }
            l.resize(l.len() + 200, b'C');
            boundary_cases.push((format!("chunk-bd-{target_thin}"), l));
        }
        // Epoch crossing m ~ 65_536 + thick strips so bands > 0 and chunks span epoch boundary
        for &m_half in &[32_767usize, 32_768, 32_769] {
            let mut l = Vec::with_capacity(m_half * 2 + 4096);
            for i in 0..m_half {
                l.push(b'0' + (i % 10) as u8);
                l.push(b'a' + (i % 26) as u8);
            }
            for c in 0..40u8 {
                l.resize(l.len() + 40, b'A' + (c % 26));
            }
            boundary_cases.push((format!("epoch-bd-{}", m_half * 2), l));
        }
        // OutQ overflow case: 256 distinct bytes each with 3 thin rows (256 chunks = 256 KiB > n = 2000!)
        {
            let mut l = Vec::new();
            for c in 0..=255u8 {
                l.push(c);
                l.push(c);
                l.push(c);
            }
            l.resize(l.len() + 40, 0x80);
            l.resize(l.len() + 40, 0xFF);
            boundary_cases.push(("outq-overflow-fallback".to_string(), l));
        }
        let mut ovf_fallback_seen = false;
        for (tag, l) in &boundary_cases {
            let recs = records_of(l);
            let exp = forward_fl_invert(l, 7);
            let mut out_p = vec![0xAAu8; l.len() + 128];
            let (st64, _) = fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, false>(&recs, l.len(), 7, &mut out_p);
            assert_eq!(out_p, exp, "OutTail64 poison/boundary mismatch on {tag}");
            if tag == "outq-overflow-fallback" {
                assert!(!st64.used_out_tail, "expected OutTail64 fallback on overflow block");
                ovf_fallback_seen = true;
            }
            out_p.fill(0xAA);
            let (st32, _) = fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL32, false>(&recs, l.len(), 7, &mut out_p);
            assert_eq!(out_p, exp, "OutTail32 poison/boundary mismatch on {tag}");
            if tag == "outq-overflow-fallback" {
                assert!(!st32.used_out_tail, "expected OutTail32 fallback on overflow block");
            }
            out_p.fill(0xAA);
            fk.run::<MODE_INLINE3, 0, BAND_OUTTAIL64, false>(&recs, l.len(), 7, &mut out_p);
            assert_eq!(out_p, exp, "I3+OutTail64 poison/boundary mismatch on {tag}");
        }
        assert!(ovf_fallback_seen);
        println!("  PASS: Forced non-ASCII g==4 twin fallback (ASCII i4={}W < i3={}W; non-ASCII i4={}W == i3={}W) + OutTail chunk(511/512/513)/epoch(65536)/overflow/0xAA-poison tests BYTE-EXACT.",
            st_i4_a.rules_words, st_i3_a.rules_words, st_i4_n.rules_words, st_i3_n.rules_words);
    }

    // 4. All 6 Admitted + 6 Non-Wikidata + 4 Code/Docs (if present) + ALL 233 full wikidata.bz2 blocks!
    for blk in wiki6.iter().chain(nonwiki6.iter()) {
        fk.run::<MODE_INLINE3, 0, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I3 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I4 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE3, 7, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "P7 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE3, 8, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "P8 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 8, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I4+P8 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 0, BAND_U32_TRIM, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I4+Band32 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE3, 0, BAND_OUTTAIL64, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I3+OutTail64 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I4+OutTail64 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL32, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, blk.expected, "I4+OutTail32 mismatch on {}", blk.name);
    }
    let mut ot64_hits = 0usize;
    let mut ot32_hits = 0usize;
    for blk in wiki233 {
        r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out_r5)
            .unwrap();
        fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I4 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE3, 7, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "P7 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 8, BAND_U64_DEFAULT, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I4+P8 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE4, 0, BAND_U32_TRIM, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I4+Band32 mismatch on {}", blk.name);
        fk.run::<MODE_INLINE3, 0, BAND_OUTTAIL64, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I3+OutTail64 mismatch on {}", blk.name);
        let (st64, _) = fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I4+OutTail64 mismatch on {}", blk.name);
        if st64.used_out_tail { ot64_hits += 1; }
        let (st32, _) = fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL32, false>(&blk.records, blk.n, blk.p, &mut out_v);
        assert_eq!(out_v, out_r5, "I4+OutTail32 mismatch on {}", blk.name);
        if st32.used_out_tail { ot32_hits += 1; }
    }
    println!(
        "  PASS: 6/6 Admitted + {}/{} Non-Wikidata(+Code/Docs) + {}/{} full wikidata.bz2 BYTE-EXACT (OutTail64 hit {}/{}, OutTail32 hit {}/{}).",
        nonwiki6.len(), nonwiki6.len(), wiki233.len(), wiki233.len(), ot64_hits, wiki233.len(), ot32_hits, wiki233.len()
    );
}

// ============================================================================
// Persistent WS Audit Across Fresh Worker Streaming All Blocks
// ============================================================================

#[derive(Clone, Copy)]
struct ArmSpec {
    name: &'static str,
    id: usize,
}

const ARMS: [ArmSpec; 11] = [
    ArmSpec { name: "R5-Head", id: 0 },
    ArmSpec { name: "Base-AA", id: 1 },
    ArmSpec { name: "F-Inline3", id: 2 },
    ArmSpec { name: "F-Inline4", id: 3 },
    ArmSpec { name: "F-Pref7", id: 4 },
    ArmSpec { name: "F-Pref8", id: 5 },
    ArmSpec { name: "F-I4+P8", id: 6 },
    ArmSpec { name: "I4+Band32", id: 7 },
    ArmSpec { name: "I3+OT64", id: 8 },
    ArmSpec { name: "I4+OT64", id: 9 },
    ArmSpec { name: "I4+OT32", id: 10 },
];

fn run_arm<const TIMED: bool>(
    arm_id: usize,
    r5: &mut cand_r5_head::Kernel,
    fk: &mut FKernel,
    blk: &Block,
    out: &mut Vec<u8>,
) -> (RunStats, StageTimes) {
    match arm_id {
        0 => {
            let t0 = if TIMED { Some(Instant::now()) } else { None };
            r5.run_entry(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, out);
            let mut tm = StageTimes::default();
            if let Some(t) = t0 {
                tm.s5_us = t.elapsed().as_secs_f64() * 1e6;
            }
            (RunStats::default(), tm)
        }
        1 => fk.run::<MODE_BASE, 0, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        2 => fk.run::<MODE_INLINE3, 0, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        3 => fk.run::<MODE_INLINE4, 0, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        4 => fk.run::<MODE_INLINE3, 7, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        5 => fk.run::<MODE_INLINE3, 8, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        6 => fk.run::<MODE_INLINE4, 8, BAND_U64_DEFAULT, TIMED>(&blk.records, blk.n, blk.p, out),
        7 => fk.run::<MODE_INLINE4, 0, BAND_U32_TRIM, TIMED>(&blk.records, blk.n, blk.p, out),
        8 => fk.run::<MODE_INLINE3, 0, BAND_OUTTAIL64, TIMED>(&blk.records, blk.n, blk.p, out),
        9 => fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL64, TIMED>(&blk.records, blk.n, blk.p, out),
        _ => fk.run::<MODE_INLINE4, 0, BAND_OUTTAIL32, TIMED>(&blk.records, blk.n, blk.p, out),
    }
}

fn audit_working_set(blocks: &[Block], label: &str) {
    println!("\n================================================================================");
    println!("STEADY-STATE PERSISTENT WORKER WS AUDIT (after streaming all {} blocks) — {label}", blocks.len());
    println!("================================================================================");
    println!(
        "{:<11} | {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} | {:>10} {:>10}",
        "Variant", "Total_KiB", "S3a_KiB", "Bands_KiB", "Spell_KiB", "Inl16_KiB", "Edges_KiB", "RuleWords", "dWS_vs_I3"
    );
    let mut i3_ws = 0usize;
    for arm in &ARMS[1..] {
        let mut r5 = cand_r5_head::Kernel::default();
        let mut fk = FKernel::default();
        let mut out = Vec::new();
        let mut sum_rw = 0usize;
        let mut last_st = RunStats::default();
        for blk in blocks {
            let (st, _) = run_arm::<false>(arm.id, &mut r5, &mut fk, blk, &mut out);
            sum_rw += st.rules_words;
            last_st = st;
        }
        if arm.id == 2 {
            i3_ws = last_st.ws_total_bytes;
        }
        let d_kib = (last_st.ws_total_bytes as isize - i3_ws as isize) as f64 / 1024.0;
        println!(
            "{:<11} | {:>9.2} {:>9.2} {:>9.2} {:>9.2} {:>9.2} {:>9.2} | {:>10} {:>+9.2} KiB",
            arm.name,
            last_st.ws_total_bytes as f64 / 1024.0,
            last_st.ws_s3a_bytes as f64 / 1024.0,
            last_st.ws_bands_bytes as f64 / 1024.0,
            last_st.ws_spelling_bytes as f64 / 1024.0,
            last_st.ws_inline_bytes as f64 / 1024.0,
            last_st.ws_edges_bytes as f64 / 1024.0,
            sum_rw,
            d_kib
        );
    }
}

fn bench_stage_1t(blocks: &[Block], label: &str, rounds: usize) {
    println!("\n================================================================================");
    println!("1T STAGE BREAKDOWN (PAIRED {rounds} ROUNDS, p50 us, SUM over {} blocks) — {label}", blocks.len());
    println!("================================================================================");
    let n_arms = ARMS.len();
    let mut s1 = vec![0.0; n_arms];
    let mut s2a = vec![0.0; n_arms];
    let mut s2b = vec![0.0; n_arms];
    let mut s3a = vec![0.0; n_arms];
    let mut s4 = vec![0.0; n_arms];
    let mut s5 = vec![0.0; n_arms];
    let mut ind = vec![0.0; n_arms];
    let mut con = vec![0.0; n_arms];
    let mut tot = vec![0.0; n_arms];

    let mut r5 = cand_r5_head::Kernel::default();
    let mut fks: Vec<FKernel> = (0..n_arms).map(|_| FKernel::default()).collect();
    let mut out = Vec::new();

    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[v.len() / 2]
    };

    for blk in blocks {
        // Warmup
        for arm in &ARMS {
            run_arm::<false>(arm.id, &mut r5, &mut fks[arm.id], blk, &mut out);
        }
        let mut v_s1 = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_s2a = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_s2b = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_s3a = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_s4 = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_s5 = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_ind = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_con = vec![Vec::with_capacity(rounds); n_arms];
        let mut v_tot = vec![Vec::with_capacity(rounds); n_arms];

        for _ in 0..rounds {
            for arm in &ARMS {
                if arm.id == 0 {
                    let (c, e) = r5.run_stages(&blk.records, blk.n, blk.p, cand_r5_head::Cap::NONE, &mut out);
                    v_con[0].push(c);
                    v_s5[0].push(e);
                    v_tot[0].push(c + e);
                } else {
                    let (_, tm) = run_arm::<true>(arm.id, &mut r5, &mut fks[arm.id], blk, &mut out);
                    v_s1[arm.id].push(tm.s1_us);
                    v_s2a[arm.id].push(tm.s2a_us);
                    v_s2b[arm.id].push(tm.s2b_us);
                    v_s3a[arm.id].push(tm.s3a_us);
                    v_s4[arm.id].push(tm.s4_us);
                    v_s5[arm.id].push(tm.s5_us);
                    v_ind[arm.id].push(tm.induced_us());
                    v_con[arm.id].push(tm.construct_us());
                }
            }
            // Also untimed end-to-end total for clean comparison
            for arm in &ARMS[1..] {
                let t0 = Instant::now();
                run_arm::<false>(arm.id, &mut r5, &mut fks[arm.id], blk, &mut out);
                v_tot[arm.id].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        for arm in &ARMS {
            let i = arm.id;
            if i == 0 {
                con[0] += p50(&mut v_con[0]);
                s5[0] += p50(&mut v_s5[0]);
                tot[0] += p50(&mut v_tot[0]);
            } else {
                s1[i] += p50(&mut v_s1[i]);
                s2a[i] += p50(&mut v_s2a[i]);
                s2b[i] += p50(&mut v_s2b[i]);
                s3a[i] += p50(&mut v_s3a[i]);
                s4[i] += p50(&mut v_s4[i]);
                s5[i] += p50(&mut v_s5[i]);
                ind[i] += p50(&mut v_ind[i]);
                con[i] += p50(&mut v_con[i]);
                tot[i] += p50(&mut v_tot[i]);
            }
        }
    }

    println!(
        "{:<11} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>8} {:>7} {:>7} | {:>8} {:>7} {:>7}",
        "Variant", "S1", "S2a", "S2b", "S3a", "S4", "S5", "Ind(us)", "vsBase", "vsI3", "Tot(us)", "vsR5H", "vsI3"
    );
    for arm in &ARMS {
        let i = arm.id;
        if i == 0 {
            println!(
                "{:<11} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7.1} | {:>8} {:>7} {:>7} | {:>8.1}  1.000x  {:.3}x",
                arm.name, "-", "-", "-", "-", "-", s5[0], "-", "-", "-", tot[0], tot[0] / tot[2]
            );
        } else {
            println!(
                "{:<11} | {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} {:>7.1} | {:>8.1}  {:.3}x  {:.3}x | {:>8.1}  {:.3}x  {:.3}x",
                arm.name,
                s1[i], s2a[i], s2b[i], s3a[i], s4[i], s5[i],
                ind[i], ind[i] / ind[1], ind[i] / ind[2],
                tot[i], tot[i] / tot[0], tot[i] / tot[2]
            );
        }
    }
}

fn bench_multiworker(
    blocks: &[Block],
    label: &str,
    arm_ids: &[usize],
    rounds: usize,
    iters_1w: usize,
    iters_mw: usize,
) {
    println!(
        "\n--- Multi-Worker (1T / 2W / 4W, paired alternating {rounds} rounds, p50 wall ms [paired vsR5H/vsI3]) — {label} ---"
    );
    let warm_n = blocks.len().min(16);
    for &workers in &[1usize, 2, 4] {
        let mut res: Vec<Vec<f64>> = vec![Vec::with_capacity(rounds); arm_ids.len()];
        let iters = if workers == 1 { iters_1w } else { iters_mw };
        for _ in 0..rounds {
            for (idx, &arm_id) in arm_ids.iter().enumerate() {
                let bar = Barrier::new(workers);
                let t_wall = std::thread::scope(|s| {
                    let mut handles = Vec::new();
                    for _ in 0..workers {
                        handles.push(s.spawn(|| {
                            let mut r5 = cand_r5_head::Kernel::default();
                            let mut fk = FKernel::default();
                            let mut out = Vec::new();
                            // Steady-state warmup (capped at 16 blocks to avoid doubling 233-block runtime)
                            for blk in &blocks[..warm_n] {
                                run_arm::<false>(arm_id, &mut r5, &mut fk, blk, &mut out);
                            }
                            bar.wait();
                            let t0 = Instant::now();
                            for _ in 0..iters {
                                for blk in blocks {
                                    run_arm::<false>(arm_id, &mut r5, &mut fk, blk, &mut out);
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
                res[idx].push(t_wall);
            }
        }
        let p50_slice = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        };
        let meds: Vec<f64> = res.iter().map(|v| p50_slice(v)).collect();
        print!("  {workers}W: ");
        for (idx, &arm_id) in arm_ids.iter().enumerate() {
            let m = meds[idx];
            if idx == 0 {
                print!("{}={:.2}ms ", ARMS[arm_id].name, m);
            } else {
                let mut pr_r5: Vec<f64> = (0..rounds).map(|r| res[idx][r] / res[0][r]).collect();
                let mut pr_i3: Vec<f64> = (0..rounds).map(|r| res[idx][r] / res[2][r]).collect();
                pr_r5.sort_by(|a, b| a.total_cmp(b));
                pr_i3.sort_by(|a, b| a.total_cmp(b));
                print!(
                    "| {}={:.2}ms({:.3}x/{:.3}x) ",
                    ARMS[arm_id].name,
                    m,
                    pr_r5[rounds / 2],
                    pr_i3[rounds / 2]
                );
            }
        }
        println!();
    }
}

fn resolve_corpus_path(candidates: &[&str]) -> Option<String> {
    for &p in candidates {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    None
}

fn main() {
    let focused = std::env::args().any(|a| a == "--focused");
    let wiki6 = load_six_admitted();
    let nonwiki_names = [
        "jsonl_0", "jsonl_1", "jsonl_2", "jsonl_3", "jsonl_4", "syslog_0",
    ];
    let nonwiki_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/non_wikidata_blocks.bin",
    ])
    .expect("locate non_wikidata_blocks.bin");
    let wiki431_path = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/wikidata_431_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/wikidata_431_blocks.bin",
    ])
    .expect("locate wikidata_431_blocks.bin");
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false, true);
    let code4 = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ])
    .map(|p| {
        let names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
        load_bin_blocks(&p, Some(&names), false, true)
    })
    .unwrap_or_default();
    let wiki233 = load_bin_blocks(&wiki431_path, None, true, false);

    let nonwiki_exact: Vec<Block> = nonwiki6
        .iter()
        .chain(code4.iter())
        .map(|b| Block {
            name: b.name.clone(),
            n: b.n,
            p: b.p,
            records: b.records.clone(),
            expected: b.expected.clone(),
        })
        .collect();
    run_exactness_suite(&wiki6, &nonwiki_exact, &wiki233);

    audit_working_set(&wiki6, "6 Admitted Wikidata Blocks");
    audit_working_set(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)");
    if !code4.is_empty() {
        audit_working_set(&code4, "4 Real Code & Docs Blocks");
    }
    audit_working_set(&wiki233, "233 Full wikidata.bz2 Blocks");

    bench_stage_1t(&wiki6, "6 Admitted Wikidata Blocks", 15);
    bench_stage_1t(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)", 15);
    if !code4.is_empty() {
        bench_stage_1t(&code4, "4 Real Code & Docs Blocks", 9);
    }

    let arm_ids: Vec<usize> = if focused {
        vec![0, 1, 2, 3, 5, 8, 9, 10]
    } else {
        (0..ARMS.len()).collect()
    };
    bench_multiworker(&wiki6, "6 Admitted Wikidata Blocks", &arm_ids, 9, 4, 4);
    bench_multiworker(&nonwiki6, "6 Real Non-Wikidata Blocks (JSONL + Syslog)", &arm_ids, 9, 6, 6);
    if !code4.is_empty() {
        bench_multiworker(&code4, "4 Real Code & Docs Blocks", &arm_ids, 5, 2, 2);
    }
    bench_multiworker(&wiki233, "233 Full wikidata.bz2 Blocks", &arm_ids, 5, 1, 1);
}
