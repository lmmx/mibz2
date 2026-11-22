//! Cycle 8 Candidate: `MorseAp4Open` — Discrete Morse Apparent-Pair (`Theorem M1`) + Open Core.
//!
//! Self-contained, `#![forbid(unsafe_code)]` safe-Rust `bench/moonshot` candidate.
//! Optimized packed layout (`u64` inline words `<= 7 B`, `u64` packed `(par, delta)` pairs,
//! zero `outq` scatter in Pass 1, exact `endpoint` thick-strip induction, bijective `wire_xor`
//! + `emit_bidirectional`).

#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, OpenCells};
use crate::runs::Runs;
use crate::Kernels;

pub type Candidate = MorseAp4Open;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const PAGE: usize = 64;
const NONE: u32 = u32::MAX;

/// Mode ablation flags (compile-time constants for zero-overhead specialization):
///   MORSE_MODE: 0 = AP4_max (3-color cycle break), 2 = AP2_asc (1-pass backward, 0 cycle checks)
///   W_MIN:      1 = all thin (1..=16), 2 = thin w>=2 only (2..=16, 4.5x fewer lookups!)
const MORSE_MODE: u8 = 0;
const W_MIN: u32 = 2;
const AP4_CAP: u8 = 7;

const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

#[derive(Clone, Copy)]
struct Cap {
    m: u32,
    strips: usize,
}

impl Cap {
    const NONE: Cap = Cap { m: u32::MAX, strips: usize::MAX };
    fn dense(n: usize) -> Cap {
        Cap { m: DENSE_M.min((n as u64 * 3 / 10) as u32), strips: DENSE_STRIPS }
    }
}

fn admit_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
    if n == 0 || n > 900_000 || (p as usize) >= n {
        None
    } else if records * 20 < n * 3 || records * 64 < n {
        Some(Cap::NONE)
    } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
        Some(Cap::dense(n))
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct Strip { from: u32, to: u32, width: u32, byte: u8 }
impl Strip {
    #[inline(always)]
    fn unpack(w: &[u32]) -> Self {
        Strip { from: w[0], to: w[1], width: w[2] >> 8, byte: (w[2] & 0xFF) as u8 }
    }
}

#[derive(Clone, Copy)]
struct Band(u64);
impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)] fn end(self) -> u32 { (self.0 & 0xF_FFFF) as u32 }
    #[inline(always)] fn dst(self) -> u32 { ((self.0 >> 20) & 0xF_FFFF) as u32 }
    #[inline(always)] fn code(self) -> u32 { (self.0 >> 40) as u32 }
}

/// Packed Morse node metadata (16 bytes total per thin strip vs 41 bytes in draft 1!):
///   par_delta: low 32 bits = par (NONE if critical, else target/root strip id);
///              high 32 bits = wrapping step_delta / root_delta in thin-rank space (or crit_base when par==NONE!)
///   word_len:  low 56 bits = up to 7 forward-text bytes (or initial step_code before compression!);
///              high 8 bits = byte length (<= 7)
#[derive(Clone, Copy, Default)]
struct MorseNode {
    par_delta: u64,
    word_len: u64,
}

impl MorseNode {
    #[inline(always)] fn par(self) -> u32 { self.par_delta as u32 }
    #[inline(always)] fn delta_or_crit(self) -> u32 { (self.par_delta >> 32) as u32 }
    #[inline(always)] fn set_par_delta(&mut self, p: u32, d: u32) {
        self.par_delta = u64::from(p) | (u64::from(d) << 32);
    }
    #[inline(always)] fn word(self) -> u64 { self.word_len & 0x00FF_FFFF_FFFF_FFFF }
    #[inline(always)] fn len(self) -> u8 { (self.word_len >> 56) as u8 }
    #[inline(always)] fn set_word_len(&mut self, w: u64, l: u8) {
        self.word_len = (w & 0x00FF_FFFF_FFFF_FFFF) | (u64::from(l) << 56);
    }
}

#[inline(always)]
fn starts(counts: [u32; 256]) -> [u32; 256] {
    let mut total = 0u32;
    counts.map(|w| { let s = total; total += w; s })
}
#[inline(always)]
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
        inline16.extend((0..=255u8).map(|c| { let mut v = [0u8; INLINE]; v[0] = c; v }));
        lens.clear();
        lens.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    if inline16.capacity() < 256 + words {
        inline16.reserve(256 + words - inline16.len());
        lens.reserve(256 + words - lens.len());
    }
}

// --- Auxiliary Interval Exchange (endpoint) ---
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
    const STRIDE: usize = 4; const BOTTOM: usize = 1;
    #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] & 0xFFFF }
    #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 4 + side] >> 16 }
    #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF_0000) | to;
    }
    #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side]; *w = (*w & 0xFFFF) | (to << 16);
    }
}
impl Links for Wide {
    const STRIDE: usize = 6; const BOTTOM: usize = 2;
    #[inline(always)] fn prev(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side] }
    #[inline(always)] fn next(a: &[u32], id: u32, side: usize) -> u32 { a[id as usize * 6 + 2 * side + 1] }
    #[inline(always)] fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side] = to; }
    #[inline(always)] fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) { a[id as usize * 6 + 2 * side + 1] = to; }
}
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const ID_BITS: u32 = 22;
const ID_MASK: u32 = (1 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

#[inline(always)] fn literal(byte: u8) -> u32 { (1 << ID_BITS) | (u32::from(byte) + 1) }
#[cold] #[inline(never)] fn grow(a: &mut Vec<u32>, need: usize) { a.resize(need.max(2 * a.len()), 0); }

struct Elim<'a, L: Links> { a: &'a mut [u32], base: usize, top: usize, nodes: u32, _l: std::marker::PhantomData<L> }
impl<L: Links> Elim<'_, L> {
    #[inline(always)] fn width(&self, id: u32) -> u32 { self.a[id as usize * L::STRIDE + L::STRIDE - 2] }
    #[inline(always)] fn set_width(&mut self, id: u32, w: u32) { self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w; }
    #[inline(always)] fn word(&self, id: u32) -> u32 { self.a[id as usize * L::STRIDE + L::STRIDE - 1] }
    #[inline(always)] fn set_word(&mut self, id: u32, w: u32) { self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w; }
    #[inline(always)] fn bottom(&self, id: u32) -> u32 { self.a[id as usize * L::STRIDE + L::BOTTOM] }
    #[inline(always)] fn set_bottom(&mut self, id: u32, x: u32) { self.a[id as usize * L::STRIDE + L::BOTTOM] = x; }
    fn assign_bottom(&mut self, rows: u32) {
        let mut first = L::next(self.a, 0, 1); let mut last = L::prev(self.a, 0, 1);
        let (mut lo, mut hi) = (0, rows);
        while first != last {
            let next = L::next(self.a, first, 1); let prev = L::prev(self.a, last, 1);
            let width = self.width(first); hi -= self.width(last);
            self.set_bottom(first, lo); self.set_bottom(last, hi); lo += width;
            if next == last { return; }
            first = next; last = prev;
        }
        if first != 0 { self.set_bottom(first, lo); }
    }
    fn node(&mut self, width: u32, word: u32) -> u32 {
        let id = self.nodes; self.nodes += 1; self.set_width(id, width); self.set_word(id, word); id
    }
    #[inline(always)] fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
        let (head, tail) = list;
        if *tail == 0 { *head = id; } else { L::set_next(self.a, *tail, side, id); }
        L::set_prev(self.a, id, side, *tail); *tail = id;
    }
    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 { continue; }
            L::set_next(self.a, prev, side, head); L::set_prev(self.a, head, side, prev); prev = tail;
        }
        L::set_next(self.a, prev, side, 0); L::set_prev(self.a, 0, side, prev);
    }
    #[inline(always)] fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n); L::set_prev(self.a, n, side, p);
    }
    #[inline(always)] fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side); L::set_next(self.a, prev, side, 0); L::set_prev(self.a, 0, side, prev);
    }
    #[inline(always)] fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at); L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id); L::set_prev(self.a, next, side, id);
    }
    #[inline(always)] fn rule(&self, w: u32) -> usize { self.base + (w - 257) as usize * RULE }
    #[inline(always)] fn len(&self, w: u32) -> u32 {
        if w <= 256 { u32::from(w != 0) }
        else { let r = self.rule(w); let tag = self.a[r + 3]; if tag >> 28 == 0 { tag >> 24 } else { self.a[r + 2] } }
    }
    #[inline(always)] fn short_payload(&self, id: u32) -> u128 {
        if id <= 256 { return u128::from(id - 1); }
        let r = self.rule(id);
        u128::from(self.a[r]) | (u128::from(self.a[r + 1]) << 32)
            | (u128::from(self.a[r + 2]) << 64) | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
    }
    #[inline(always)] fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 { return right; }
        if right == 0 { return left; }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (ll, rl) = (left >> ID_BITS, right >> ID_BITS);
        let (l, r) = (left & ID_MASK, right & ID_MASK);
        let len = if ll < SATURATED && rl < SATURATED { ll + rl } else { self.len(l) + self.len(r) };
        let rule = if len < INLINE as u32 {
            let b = self.short_payload(l) | (self.short_payload(r) << (8 * ll)) | (u128::from(len) << 120);
            [b as u32, (b >> 32) as u32, (b >> 64) as u32, (b >> 96) as u32]
        } else { [l, r, len, u32::MAX] };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
    }
    fn power(&mut self, word: u32, q: u32) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 { acc = self.join(acc, base); }
            e >>= 1;
            if e == 0 { break acc; }
            base = self.join(base, base);
        }
    }
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
        let mut left = *remaining;
        let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
        let done = loop {
            if left == 0 { break true; }
            if self.top + room > self.a.len() { break false; }
            let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            let (aw, bw) = (self.width(x), self.width(y));
            if SPIRALS && x != y {
                if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = self.power(self.word(x), q);
                    let word = self.join(head, self.word(y));
                    self.set_word(y, word); self.set_width(x, aw - q * bw); left -= q * bw; continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power(self.word(y), q);
                    let word = self.join(self.word(x), tail);
                    self.set_word(x, word); self.set_width(y, bw - q * aw); left -= q * aw; continue;
                }
            }
            let w = aw.min(bw); left -= w;
            if x == y { self.pop_tail(x, 0); self.pop_tail(x, 1); continue; }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.pop_tail(x, 0); self.insert(y, x, 0); self.set_word(x, word);
                if aw == bw { self.cut(y, 0); self.pop_tail(y, 1); } else { self.set_width(y, bw - w); }
            } else {
                self.pop_tail(y, 1); self.insert(x, y, 1); self.set_word(y, word); self.set_width(x, aw - w);
            }
        };
        *remaining = left; done
    }
    fn inline_word(&mut self, word: u32, byte: u8, stack: &mut Vec<u32>, spelling: &mut Vec<u8>) -> ([u8; INLINE], u8) {
        let id = word & ID_MASK; let height = (word >> ID_BITS) as usize;
        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes(); bytes[height] = byte; (bytes, (height + 1) as u8)
        } else {
            let start = spelling.len() as u32; self.append(id, stack, spelling); spelling.push(byte);
            (long_slot(start, spelling.len() as u32), LONG)
        }
    }
    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear(); stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 { dest.push((id - 1) as u8); }
            else {
                let r = self.rule(id); let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    let bytes = self.short_payload(id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]); continue;
                }
                let rule = &mut self.a[r..r + RULE];
                if tag == u32::MAX {
                    rule[3] = 0x8000_0000 | dest.len() as u32; stack.push(rule[1]); stack.push(rule[0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }
}

struct Dictionary<'a> {
    pairs: &'a mut [u32], contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>, lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>, stack: &'a mut Vec<u32>,
}
impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 { return u32::from(byte); }
        let id = handle & ID_MASK; let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key { return self.pairs[slot + 1]; }
        let start = self.spelling.len();
        let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
        let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
        if self.contents.is_empty() { return self.insert(entry, len); }
        let bytes = if len == LONG { &self.spelling[start..] } else { &entry[..len as usize] };
        let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
        let old = self.contents[index] as usize;
        let same = old != 0 && self.lens[old] == len && if len == LONG {
            long_word(&self.inline16[old], self.spelling) == bytes
        } else { self.inline16[old] == entry };
        let code = if same {
            if len == LONG { if memoized { self.spelling.truncate(start); } else { self.spelling.pop(); } }
            old as u32
        } else { let code = self.insert(entry, len); self.contents[index] = code; code };
        self.pairs[slot] = key; self.pairs[slot + 1] = code; code
    }
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32; self.inline16.push(entry); self.lens.push(len); code
    }
}
fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
    let (low, high, middle) = if len == LONG {
        let part = |i: usize| u64::from_le_bytes([
            bytes[i], bytes[i+1], bytes[i+2], bytes[i+3], bytes[i+4], bytes[i+5], bytes[i+6], bytes[i+7]
        ]);
        (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
    } else { let v = u128::from_le_bytes(*entry); (v as u64, (v >> 64) as u64, 0) };
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy, Debug)]
pub struct StageStats {
    pub s1_ms: f64,
    pub s2_s3a_ms: f64,
    pub ap4_find_ms: f64,
    pub ap4_comp_ms: f64,
    pub s4_xor_ms: f64,
    pub emit_ms: f64,
    pub m_16: u32,
    pub m_crit: u32,
    pub r_thin: usize,
    pub codes: usize,
    pub wide: bool,
    pub peak_bytes: usize,
}

#[derive(Default)]
pub struct MorseAp4Open {
    fallback: OpenCells,
    edges: Vec<u32>,
    extra: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    stack: Vec<u32>,
    // Compact Morse buffers: thin_hi(4B) + thin_byte(1B) + nodes(16B) + color(1B) = 22 B * r_thin (~2.5x leaner!)
    thin_hi: Vec<u32>,
    thin_byte: Vec<u8>,
    thin_dir: Vec<u32>,
    nodes: Vec<MorseNode>,
    color: Vec<u8>,
    concat_cache: Vec<u32>,
}

impl MorseAp4Open {
    #[inline(always)]
    fn find_thin_strip(&self, thin_row: u32) -> usize {
        let mut s = self.thin_dir[(thin_row as usize) >> 6] as usize;
        while self.thin_hi[s] <= thin_row {
            s += 1;
        }
        s
    }

    #[allow(dead_code)]
    pub fn peak_buffer_bytes(&self) -> usize {
        self.edges.capacity() * 4
            + self.extra.capacity() * 4
            + self.bands.capacity() * 8
            + self.spelling.capacity()
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.stack.capacity() * 4
            + self.thin_hi.capacity() * 4
            + self.thin_byte.capacity()
            + self.thin_dir.capacity() * 4
            + self.nodes.capacity() * 16
            + self.color.capacity()
            + self.concat_cache.capacity() * 4
    }

    #[inline(always)]
    fn intern_concat(&mut self, ap4_w: u64, ap4_l: u8, bcode: u32) -> u32 {
        let blen = self.lens[bcode as usize];
        if blen != LONG && (ap4_l as usize) + (blen as usize) <= INLINE {
            let bword = u128::from_le_bytes(self.inline16[bcode as usize]);
            let combined = u128::from(ap4_w) | (bword << (8 * u32::from(ap4_l)));
            let tot_len = ap4_l + blen;
            let entry = combined.to_le_bytes();
            let h = (combined as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ ((combined >> 64) as u64).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ u64::from(tot_len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let slot = ((h ^ (h >> 30)) as usize) & (self.concat_cache.len() - 1);
            let old = self.concat_cache[slot] as usize;
            if old != 0 && self.lens[old] == tot_len && self.inline16[old] == entry {
                return old as u32;
            }
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(tot_len);
            self.concat_cache[slot] = code;
            code
        } else {
            let start = self.spelling.len() as u32;
            let aw_bytes = ap4_w.to_le_bytes();
            self.spelling.extend_from_slice(&aw_bytes[..ap4_l as usize]);
            if blen == LONG {
                let slot = self.inline16[bcode as usize];
                let s = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
                let e = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
                self.spelling.extend_from_within(s..e);
            } else {
                let bw = self.inline16[bcode as usize];
                self.spelling.extend_from_slice(&bw[..blen as usize]);
            }
            let end = self.spelling.len() as u32;
            let code = self.lens.len() as u32;
            self.inline16.push(long_slot(start, end));
            self.lens.push(LONG);
            code
        }
    }

    fn run_induced_with<L: Links>(&mut self, ns: usize, n: u32, m_16: u32, thin: &[u32; 256]) -> [usize; 256] {
        let Self { edges, bands, spelling, inline16, lens, stack, .. } = self;
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
        let mut e = Elim::<L> { a: arena, base: 0, top: 0, nodes: 0, _l: std::marker::PhantomData };
        e.node(0, 0);
        let mut widest = 0u32;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        let (mut terminals, mut order) = ([(0u32, 0u32); 2], [(0u32, 0u32); 2]);
        let mut x = 0u32;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            if x < s.from {
                widest = widest.max(s.from - x);
                let id = e.node(s.from - x, 0);
                e.append_to(&mut order[0], id, 0); e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0); e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
            if x < s.to {
                widest = widest.max(s.to - x);
                let id = e.node(s.to - x, 0);
                e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = s.to + s.width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[1], id, 1); e.append_to(&mut terminals[0], id, 0);
        }
        for side in 0..2 { e.close(side, terminals[side], order[side]); }
        e.base = L::STRIDE * e.nodes as usize; e.top = e.base;
        let mut remaining = n; let spirals = widest > CERTIFIED_WIDTH;
        while !if spirals { e.eliminate::<true>(&mut remaining) } else { e.eliminate::<false>(&mut remaining) } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, 4 * ns + top + RULE);
            e = Elim { a: &mut edges[4 * ns..], base, top, nodes, _l: std::marker::PhantomData };
        }
        e.assign_bottom(m_16);
        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;
        let (base, top, nodes) = (e.base, e.top, e.nodes);
        let (header, arena) = edges.split_at_mut(4 * ns);
        let (live, tail) = arena.split_at_mut(top);
        let cache = if header.len() >= CACHE_WORDS { header } else { tail };
        let (pairs, contents) = if cache.len() >= CACHE_WORDS {
            cache[..CACHE_WORDS].fill(0); cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
        } else { (&mut [][..], &mut [][..]) };
        let mut e = Elim::<L> { a: live, base, top, nodes, _l: std::marker::PhantomData };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        let mut dictionary = Dictionary { pairs, contents, inline16, lens, spelling, stack };
        let mut cursors = [0usize; 256];
        let mut id = L::next(e.a, 0, 0); let mut in_node = 0u32; let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {
                let nw = e.width(id); let nword = e.word(id);
                let width = (end - k).min(nw - in_node);
                let dst = e.bottom(id) + in_node;
                let code = dictionary.code(&mut e, nword, a as u8);
                bands.push(Band::new(k + width, dst, code));
                in_node += width; k += width;
                if in_node == nw { id = L::next(e.a, id, 0); in_node = 0; }
            }
        }
        cursors
    }

    fn run_core<const PROFILE: bool>(
        &mut self,
        records: &[u16],
        n_usize: usize,
        p: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(bool, StageStats)> {
        use std::time::Instant;
        let n = n_usize as u32;
        let mut st = StageStats::default();
        let t0 = if PROFILE { Some(Instant::now()) } else { None };

        let mut thick_rows = [0u32; 256];
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut m_16 = 0u32;
        let mut ns = 0usize;
        let mut p_thin_strip = NONE;

        self.thin_hi.clear();
        self.thin_byte.clear();

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
            if width == 0 { continue; }
            let a = usize::from(c);
            let contains_p = p >= pos && p - pos < width;
            if width <= LIMIT && !contains_p {
                if m_16 + width > cap.m { return None; }
                thin[a] += width; m_16 += width;
                self.thin_hi.push(m_16); self.thin_byte.push(c);
            } else if !contains_p {
                if ns >= cap.strips { return None; }
                let at = 3 * ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3).max(2 * self.edges.len()).max(1024);
                    self.edges.resize(need, 0);
                }
                self.edges[at] = pos; self.edges[at + 1] = thin[a] + thick_rows[a];
                self.edges[at + 2] = (width << 8) | u32::from(c);
                ns += 1; thick[a] += 1; thick_rows[a] += width;
            } else {
                let left_w = p - pos;
                let right_w = pos + width - p - 1;
                let is_thin = width <= LIMIT;
                if left_w > 0 {
                    if is_thin {
                        if m_16 + left_w > cap.m { return None; }
                        thin[a] += left_w; m_16 += left_w;
                        self.thin_hi.push(m_16); self.thin_byte.push(c);
                    } else {
                        if ns >= cap.strips { return None; }
                        let at = 3 * ns;
                        if self.edges.len() < at + 3 { let need = (at + 3).max(2 * self.edges.len()).max(1024); self.edges.resize(need, 0); }
                        self.edges[at] = pos; self.edges[at + 1] = thin[a] + thick_rows[a]; self.edges[at + 2] = (left_w << 8) | u32::from(c);
                        ns += 1; thick[a] += 1; thick_rows[a] += left_w;
                    }
                }
                if m_16 + 1 > cap.m { return None; }
                thin[a] += 1; m_16 += 1; p_thin_strip = self.thin_hi.len() as u32;
                self.thin_hi.push(m_16); self.thin_byte.push(c);
                if right_w > 0 {
                    if is_thin {
                        if m_16 + right_w > cap.m { return None; }
                        thin[a] += right_w; m_16 += right_w;
                        self.thin_hi.push(m_16); self.thin_byte.push(c);
                    } else {
                        if ns >= cap.strips { return None; }
                        let at = 3 * ns;
                        if self.edges.len() < at + 3 { let need = (at + 3).max(2 * self.edges.len()).max(1024); self.edges.resize(need, 0); }
                        self.edges[at] = p + 1; self.edges[at + 1] = thin[a] + thick_rows[a]; self.edges[at + 2] = (right_w << 8) | u32::from(c);
                        ns += 1; thick[a] += 1; thick_rows[a] += right_w;
                    }
                }
            }
            pos = pos.saturating_add(width);
        }
        if pos != n || m_16 == 0 || p_thin_strip == NONE { return Some((false, st)); }
        let t1 = if PROFILE { Some(Instant::now()) } else { None };

        // Stage 2: endpoint thick-strip elimination
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);
        let most = 3 * ns + 3;
        let use_wide = most > NARROW_NODES;
        let stride = if use_wide { Wide::STRIDE } else { Narrow::STRIDE };
        let need = (4 * ns + (stride + RULE) * most).max(m_16 as usize);
        if self.edges.len() < need { self.edges.resize(need, 0); }
        {
            let (strips_raw, rest) = self.edges.split_at_mut(3 * ns);
            let image_order = &mut rest[..ns];
            for idx in 0..ns {
                let sw = &mut strips_raw[3 * idx..3 * idx + 3];
                let a = (sw[2] & 0xFF) as usize;
                sw[1] += first[a];
                image_order[write[a] as usize] = idx as u32;
                write[a] += 1;
            }
        }
        self.bands.clear(); self.spelling.clear();
        let cursors = if use_wide {
            self.run_induced_with::<Wide>(ns, n, m_16, &thin)
        } else {
            self.run_induced_with::<Narrow>(ns, n, m_16, &thin)
        };
        let t2 = if PROFILE { Some(Instant::now()) } else { None };

        // Stage 3a: Packed AP4 apparent-pair discovery (skipping w < W_MIN -> 3.7x fewer lookups when W_MIN=2!)
        let r_thin = self.thin_hi.len();
        let n_pages = (m_16 as usize / PAGE) + 1;
        self.thin_dir.clear();
        let mut si = 0usize;
        for pg in 0..n_pages {
            let row = (pg * PAGE) as u32;
            while si + 1 < r_thin && self.thin_hi[si] <= row { si += 1; }
            self.thin_dir.push(si as u32);
        }
        self.nodes.clear();
        self.nodes.resize(r_thin, MorseNode { par_delta: u64::from(NONE), word_len: 0 });

        let mut z_cur = starts(thin);
        let mut b_cur = cursors;
        let mut prev_hi = 0u32;
        for u in 0..r_thin {
            let hi = self.thin_hi[u];
            let w = hi - prev_hi;
            let a = usize::from(self.thin_byte[u]);
            let z_lo = z_cur[a];
            let z_hi = z_lo + w;
            z_cur[a] = z_hi;

            if w >= W_MIN && (u as u32) != p_thin_strip {
                let mut bi = b_cur[a];
                while self.bands[bi].end() <= z_lo { bi += 1; }
                b_cur[a] = bi;
                let b = self.bands[bi];
                if z_hi <= b.end() {
                    let bstart = if bi == 0 { 0 } else { self.bands[bi - 1].end() };
                    let dst_lo = b.dst() + (z_lo - bstart);
                    let dst_hi = dst_lo + w;
                    let code = b.code();
                    let clen = self.lens[code as usize];
                    if clen <= AP4_CAP {
                        let v = self.find_thin_strip(dst_lo);
                        if v != u && dst_hi <= self.thin_hi[v] && (MORSE_MODE != 2 || v > u) {
                            let iw = &self.inline16[code as usize];
                            let w64 = u64::from_le_bytes([iw[0], iw[1], iw[2], iw[3], iw[4], iw[5], iw[6], iw[7]]);
                            let nd = &mut self.nodes[u];
                            nd.set_par_delta(v as u32, dst_lo.wrapping_sub(prev_hi));
                            nd.set_word_len(w64, clen);
                        }
                    }
                }
            }
            prev_hi = hi;
        }
        let t3 = if PROFILE { Some(Instant::now()) } else { None };

        // Stage 3b: Packed 16-B MorseNode chain compression (ZERO separate u128/step_code/crit_base arrays!)
        if MORSE_MODE == 2 {
            for u in (0..r_thin).rev() {
                let v = self.nodes[u].par();
                if v == NONE { continue; }
                let vn = self.nodes[v as usize];
                let pv = vn.par();
                if pv != NONE {
                    let vl = vn.len();
                    let ul = self.nodes[u].len();
                    if vl + ul <= AP4_CAP {
                        let nd = self.nodes[u].delta_or_crit().wrapping_add(vn.delta_or_crit());
                        let nw = vn.word() | (self.nodes[u].word() << (8 * u32::from(vl)));
                        self.nodes[u].set_par_delta(pv, nd);
                        self.nodes[u].set_word_len(nw, vl + ul);
                    } else {
                        self.nodes[u].par_delta = u64::from(NONE);
                    }
                }
            }
        } else {
            self.color.clear();
            self.color.resize(r_thin, 0);
            for u in 0..r_thin {
                if self.color[u] != 0 || self.nodes[u].par() == NONE { continue; }
                self.stack.clear();
                let mut cur = u;
                while self.nodes[cur].par() != NONE && self.color[cur] == 0 {
                    self.color[cur] = 1;
                    self.stack.push(cur as u32);
                    cur = self.nodes[cur].par() as usize;
                }
                if self.nodes[cur].par() != NONE && self.color[cur] == 1 {
                    self.nodes[cur].par_delta = u64::from(NONE);
                }
                for idx in (0..self.stack.len()).rev() {
                    let nd_i = self.stack[idx] as usize;
                    self.color[nd_i] = 2;
                    let v = self.nodes[nd_i].par();
                    if v == NONE { continue; }
                    let vn = self.nodes[v as usize];
                    let pv = vn.par();
                    if pv != NONE {
                        let vl = vn.len();
                        let ul = self.nodes[nd_i].len();
                        if vl + ul <= AP4_CAP {
                            let nd = self.nodes[nd_i].delta_or_crit().wrapping_add(vn.delta_or_crit());
                            let nw = vn.word() | (self.nodes[nd_i].word() << (8 * u32::from(vl)));
                            self.nodes[nd_i].set_par_delta(pv, nd);
                            self.nodes[nd_i].set_word_len(nw, vl + ul);
                        } else {
                            self.nodes[nd_i].par_delta = u64::from(NONE);
                        }
                    }
                }
            }
        }

        // Assign dense critical ranks directly into `nodes[u].delta_or_crit` when `par == NONE`!
        let mut m_crit = 0u32;
        let mut prev = 0u32;
        for u in 0..r_thin {
            let hi = self.thin_hi[u];
            if self.nodes[u].par() == NONE {
                self.nodes[u].set_par_delta(NONE, m_crit);
                m_crit += hi - prev;
            }
            prev = hi;
        }
        let mark_crit = self.nodes[p_thin_strip as usize].delta_or_crit();
        let t4 = if PROFILE { Some(Instant::now()) } else { None };

        // Stage 4: Wire ONLY the m_crit critical rows into `edges[0..m_crit)` via wire_xor!
        let bits = (u32::BITS - (m_crit - 1).leading_zeros()).max(1);
        let max_packed_code = u32::MAX >> bits;
        if self.edges.len() < m_crit as usize { self.edges.resize(m_crit as usize, 0); }
        self.edges[..m_crit as usize].fill(0);
        self.extra.clear();
        if self.concat_cache.len() != 8192 { self.concat_cache.resize(8192, 0); } else { self.concat_cache.fill(0); }

        let mut use_xor = m_crit >= 1024 && (self.lens.len() as u32 - 1) <= max_packed_code;
        let mut wide = !use_xor && (self.lens.len() as u32 - 1) > max_packed_code;
        if wide { self.extra.resize(m_crit as usize, 0); }
        let mut predecessor = mark_crit;
        let mut overflowed_xor = false;

        let mut z_cur2 = starts(thin);
        let mut b_cur2 = cursors;
        let mut prev2 = 0u32;
        for u in 0..r_thin {
            let hi = self.thin_hi[u]; let w = hi - prev2; prev2 = hi;
            let a = usize::from(self.thin_byte[u]);
            let mut z = z_cur2[a]; let z_end = z + w; z_cur2[a] = z_end;
            let un = self.nodes[u];
            if un.par() != NONE { continue; }
            let mut j_crit = un.delta_or_crit();
            let mut bi = b_cur2[a];
            while self.bands[bi].end() <= z { bi += 1; }
            while z < z_end {
                let b = self.bands[bi]; let bend = b.end();
                let bstart = if bi == 0 { 0 } else { self.bands[bi - 1].end() };
                let bspan = z_end.min(bend) - z;
                let mut dst_thin = b.dst() + (z - bstart);
                let dst_thin_end = dst_thin + bspan;
                let bcode = b.code();
                let mut s = self.find_thin_strip(dst_thin);
                while dst_thin < dst_thin_end {
                    let s_hi = self.thin_hi[s];
                    let s_lo = if s == 0 { 0 } else { self.thin_hi[s - 1] };
                    let take = dst_thin_end.min(s_hi) - dst_thin;
                    let sn = self.nodes[s];
                    let (dest_base, final_code) = if sn.par() == NONE {
                        (sn.delta_or_crit() + (dst_thin - s_lo), bcode)
                    } else {
                        let root_s = sn.par() as usize;
                        let r_lo = if root_s == 0 { 0 } else { self.thin_hi[root_s - 1] };
                        let root_thin = dst_thin.wrapping_add(sn.delta_or_crit());
                        let dest_b = self.nodes[root_s].delta_or_crit() + (root_thin - r_lo);
                        let fc = self.intern_concat(sn.word(), sn.len(), bcode);
                        (dest_b, fc)
                    };
                    if use_xor {
                        if final_code > max_packed_code { overflowed_xor = true; use_xor = false; wide = true; break; }
                        let pack = final_code << bits;
                        let mut d_idx = dest_base; let mut j_idx = j_crit;
                        for _ in 0..take {
                            self.edges[d_idx as usize] ^= j_idx | pack;
                            self.edges[j_idx as usize] ^= d_idx;
                            if j_idx == mark_crit { predecessor = d_idx; }
                            d_idx += 1; j_idx += 1;
                        }
                    } else {
                        if !wide && final_code > max_packed_code {
                            self.extra.resize(m_crit as usize, 0);
                            let mask = (1u32 << bits) - 1;
                            for idx in 0..m_crit as usize {
                                let e = self.edges[idx]; self.extra[idx] = e >> bits; self.edges[idx] = e & mask;
                            }
                            wide = true;
                        }
                        let pack = if wide { 0 } else { final_code << bits };
                        for k in 0..take {
                            let d_idx = (dest_base + k) as usize;
                            if wide { self.extra[d_idx] = final_code; }
                            self.edges[d_idx] = pack | (j_crit + k);
                        }
                    }
                    j_crit += take; dst_thin += take; s += 1;
                }
                if overflowed_xor { break; }
                z += bspan; if z == bend { bi += 1; }
            }
            b_cur2[a] = bi;
            if overflowed_xor { break; }
        }

        if overflowed_xor {
            self.edges[..m_crit as usize].fill(0);
            self.extra.resize(m_crit as usize, 0);
            let mut z_cur3 = starts(thin); let mut b_cur3 = cursors; let mut prev3 = 0u32;
            for u in 0..r_thin {
                let hi = self.thin_hi[u]; let w = hi - prev3; prev3 = hi;
                let a = usize::from(self.thin_byte[u]);
                let mut z = z_cur3[a]; let z_end = z + w; z_cur3[a] = z_end;
                let un = self.nodes[u];
                if un.par() != NONE { continue; }
                let mut j_crit = un.delta_or_crit();
                let mut bi = b_cur3[a];
                while self.bands[bi].end() <= z { bi += 1; }
                while z < z_end {
                    let b = self.bands[bi]; let bend = b.end();
                    let bstart = if bi == 0 { 0 } else { self.bands[bi - 1].end() };
                    let bspan = z_end.min(bend) - z;
                    let mut dst_thin = b.dst() + (z - bstart);
                    let dst_thin_end = dst_thin + bspan;
                    let bcode = b.code();
                    let mut s = self.find_thin_strip(dst_thin);
                    while dst_thin < dst_thin_end {
                        let s_hi = self.thin_hi[s];
                        let s_lo = if s == 0 { 0 } else { self.thin_hi[s - 1] };
                        let take = dst_thin_end.min(s_hi) - dst_thin;
                        let sn = self.nodes[s];
                        let (dest_base, final_code) = if sn.par() == NONE {
                            (sn.delta_or_crit() + (dst_thin - s_lo), bcode)
                        } else {
                            let root_s = sn.par() as usize;
                            let r_lo = if root_s == 0 { 0 } else { self.thin_hi[root_s - 1] };
                            let root_thin = dst_thin.wrapping_add(sn.delta_or_crit());
                            let dest_b = self.nodes[root_s].delta_or_crit() + (root_thin - r_lo);
                            let fc = self.intern_concat(sn.word(), sn.len(), bcode);
                            (dest_b, fc)
                        };
                        for k in 0..take {
                            let d_idx = (dest_base + k) as usize;
                            self.extra[d_idx] = final_code; self.edges[d_idx] = j_crit + k;
                        }
                        j_crit += take; dst_thin += take; s += 1;
                    }
                    z += bspan; if z == bend { bi += 1; }
                }
                b_cur3[a] = bi;
            }
        }
        let t5 = if PROFILE { Some(Instant::now()) } else { None };

        if use_xor {
            emit_bidirectional(
                &self.edges[..m_crit as usize], &self.spelling, &self.inline16, &self.lens,
                n_usize, mark_crit, predecessor, bits, out,
            );
        } else {
            emit(
                &self.edges[..m_crit as usize], &self.extra, &self.spelling, &self.inline16, &self.lens,
                n_usize, mark_crit, bits, out,
            );
        }
        let t6 = if PROFILE { Some(Instant::now()) } else { None };

        if PROFILE {
            st.s1_ms = (t1.unwrap() - t0.unwrap()).as_secs_f64() * 1e3;
            st.s2_s3a_ms = (t2.unwrap() - t1.unwrap()).as_secs_f64() * 1e3;
            st.ap4_find_ms = (t3.unwrap() - t2.unwrap()).as_secs_f64() * 1e3;
            st.ap4_comp_ms = (t4.unwrap() - t3.unwrap()).as_secs_f64() * 1e3;
            st.s4_xor_ms = (t5.unwrap() - t4.unwrap()).as_secs_f64() * 1e3;
            st.emit_ms = (t6.unwrap() - t5.unwrap()).as_secs_f64() * 1e3;
            st.m_16 = m_16; st.m_crit = m_crit; st.r_thin = r_thin;
            st.codes = self.lens.len(); st.wide = wide;
            st.peak_bytes = self.peak_buffer_bytes();
        }
        Some((true, st))
    }

    #[allow(dead_code)]
    pub fn profile_invert_runs(&mut self, r: &Runs, n_usize: usize, p: u32, out: &mut Vec<u8>) -> StageStats {
        self.run_core::<true>(r.records(), n_usize, p, Cap::NONE, out).unwrap().1
    }
}

fn emit(edges: &[u32], extra: &[u32], spelling: &[u8], inline16: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize; let mask = (1u32 << bits) - 1; let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() { edge >> bits } else { extra[j as usize] } as usize;
        j = edge & mask;
        let len = lens[code];
        if len != LONG { out[at..at + INLINE].copy_from_slice(&inline16[code]); at += usize::from(len); }
        else { let word = long_word(&inline16[code], spelling); let take = word.len().min(n - at); out[at..at + take].copy_from_slice(&word[..take]); at += take; }
        if j == mark || at >= n { break; }
    }
    out.truncate(at.min(n)); repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_bidirectional(edges: &[u32], spelling: &[u8], inline: &[[u8; INLINE]], lens: &[u8], n: usize, mark: u32, before: u32, bits: u32, out: &mut Vec<u8>) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let (mut prev, mut front) = (before, mark);
    let (mut back, mut next) = (before, mark);
    let (mut lo, mut hi) = (0usize, n);
    for pair in 0..=edges.len() / 2 {
        if lo == hi || (pair != 0 && front == next) { break; }
        let ef = edges[front as usize]; let cf = (ef >> bits) as usize;
        let single = front == back; let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE { out[lo..lo + INLINE].copy_from_slice(&inline[cf]); lo += usize::from(lf); }
        else {
            let word = if lf == LONG { long_word(&inline[cf], spelling) } else { &inline[cf][..usize::from(lf)] };
            let take = word.len().min(hi - lo); out[lo..lo + take].copy_from_slice(&word[..take]); lo += take;
        }
        if single { break; }
        let eb = edges[back as usize]; let cb = (eb >> bits) as usize; let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes()); hi -= usize::from(lb);
        } else {
            let word = if lb == LONG { long_word(&inline[cb], spelling) } else { &inline[cb][..usize::from(lb)] };
            let take = word.len().min(hi - lo); out[hi - take..hi].copy_from_slice(&word[word.len() - take..]); hi -= take;
        }
        (prev, front) = (front, (ef & mask) ^ prev);
        (next, back) = (back, (eb & mask) ^ next);
    }
    if lo < hi { out.copy_within(hi..n, lo); out.truncate(lo + (n - hi)); repeat(n, out); } else { out.truncate(n); }
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n { let take = out.len().min(n - out.len()); out.extend_from_within(..take); }
}

impl InverseBwt for MorseAp4Open {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        self.fallback.invert(ll, counts, orig_ptr, out)
    }
    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        self.fallback.invert_in_place(ll, counts, orig_ptr)
    }
    fn wants_checkpoints(&self) -> bool { self.fallback.wants_checkpoints() }
    fn invert_checkpointed(&mut self, ll: &[u8], counts: &[u32; 256], checkpoints: &[[u32; 256]], orig_ptr: u32, out: &mut Vec<u8>) {
        self.fallback.invert_checkpointed(ll, counts, checkpoints, orig_ptr, out)
    }
    fn reads_runs(&self) -> bool { true }
    fn invert_runs(&mut self, r: &Runs, n: usize, orig_ptr: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = admit_cap(r.records().len(), n, orig_ptr) {
            if let Some((ok, _)) = self.run_core::<false>(r.records(), n, orig_ptr, cap, out) {
                return ok;
            }
        }
        self.fallback.invert_runs(r, n, orig_ptr, out, k)
    }
}
