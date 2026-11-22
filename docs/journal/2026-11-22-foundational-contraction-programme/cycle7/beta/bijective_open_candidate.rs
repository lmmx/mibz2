#![forbid(unsafe_code)]
//! Investigator Beta — Cycle 7 Candidate:
//! `1-Pass Bijective Ascent-DAG (<= 8 B u64) + Open Thick-Strip Endpoint + 2-Ended wire_xor`
//! (`BijectiveOpen`, `LIMIT = 16`).
//!
//! Architecture:
//!   1. Pass 1: Merge 127-capped `r.records()` into maximal runs `r_0` grouped by `F`-bucket `out_by_sym[0..256]`
//!      (already sorted by `dlo`!).
//!   2. 1-Pass Topological Bijective `Ascent-DAG` (`c = 0..255`, `wlen <= 8` inline in `u64`, `0` SLP, `0` slab16):
//!      - Absorbs ascending edges `c_prev < c` (`~49%` of rows deleted, `q_1 = 0.63 r_0` strips!).
//!      - Protects primary row `p` as a surviving chain head -> `p_shift == 0` ALWAYS (zero rotation!).
//!      - Produces exact bijective strip-IET on surviving subset `S` (`|S| = 0.509 n`).
//!   3. Bucket-sorted domain & image partition (`~350` strips/bucket, L1d-resident):
//!      - Retains thin strips (`len <= 16`, `m' = 0.611x m_16` -> `-39%` smaller `u32` `edges`!).
//!      - Eliminates thick strips (`ns' = 0.618x Open` -> `-38%` less `endpoint.rs` work!) via
//!        spiral-certified `endpoint.rs` (`Elim<Narrow/Wide>` with inline `1..8B` thick-strip leaves!).
//!   4. Direct `wire_xor` into `4 B` `u32` `edges[0..m']` (`6/6 FITS u32`, `0/6 wide`!) ->
//!      `Open`'s exact zero-copy 2-ended `emit_bidirectional` (`273k` serial XOR rounds = `0.611x Open`!).

use crate::bwt::{InverseBwt, LazyCells};
use crate::{runs::Runs, Kernels};

pub type Candidate = BijectiveOpen;

const LIMIT: u32 = 16;
const MAX_DAG_LEN: u8 = 8;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;

const DENSE_NUM: usize = 7;
const DENSE_DEN: usize = 25;
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

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

// ============================================================================
// Compact Inline Word Dictionary (Deduplicates thin & thick words into u32 codes)
// ============================================================================

struct WordDict<'a> {
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    // Direct-mapped hash table: 8192 slots (32 KiB L1d!) mapping hash -> code + 1 (0 = empty)
    table: &'a mut [u32],
}

impl<'a> WordDict<'a> {
    fn new(
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        table: &'a mut [u32],
    ) -> Self {
        inline16.clear();
        lens.clear();
        spelling.clear();
        table.fill(0);
        // Reserve codes 0..256 for 1-byte literals (instant O(1), zero hashing!)
        for c in 0..=255u8 {
            let mut v = [0u8; INLINE];
            v[0] = c;
            inline16.push(v);
            lens.push(1);
        }
        WordDict {
            inline16,
            lens,
            spelling,
            table,
        }
    }

    #[inline(always)]
    fn intern_short(&mut self, bytes: [u8; INLINE], len: u8) -> u32 {
        if len == 1 {
            return u32::from(bytes[0]);
        }
        let val = u128::from_le_bytes(bytes);
        let lo = val as u64;
        let hi = (val >> 64) as u64;
        let h = lo
            .wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mask = self.table.len() - 1;
        let slot = ((h ^ (h >> 30)) as usize) & mask;
        let prev = self.table[slot];
        if prev != 0 {
            let idx = (prev - 1) as usize;
            if self.lens[idx] == len && self.inline16[idx] == bytes {
                return prev - 1;
            }
        }
        // Check 1 linear probe to cut collisions dramatically at 1-cycle cost
        let slot2 = (slot + 1) & mask;
        let prev2 = self.table[slot2];
        if prev2 != 0 {
            let idx = (prev2 - 1) as usize;
            if self.lens[idx] == len && self.inline16[idx] == bytes {
                return prev2 - 1;
            }
        }
        let code = self.lens.len() as u32;
        self.inline16.push(bytes);
        self.lens.push(len);
        if prev == 0 {
            self.table[slot] = code + 1;
        } else {
            self.table[slot2] = code + 1;
        }
        code
    }

    #[inline(always)]
    fn intern_u64(&mut self, word: u64, len: u8) -> u32 {
        if len == 1 {
            return (word & 0xFF) as u32;
        }
        let mut bytes = [0u8; INLINE];
        bytes[..8].copy_from_slice(&word.to_le_bytes());
        self.intern_short(bytes, len)
    }

    fn intern_long(&mut self, start: u32, end: u32) -> u32 {
        let slice = &self.spelling[start as usize..end as usize];
        let l = slice.len();
        let part = |i: usize| u64::from_le_bytes(slice[i..i + 8].try_into().unwrap());
        let h = part(0)
            .wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ part(l - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ part(l / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (l as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mask = self.table.len() - 1;
        let slot = ((h ^ (h >> 30)) as usize) & mask;
        let prev = self.table[slot];
        if prev != 0 {
            let idx = (prev - 1) as usize;
            if self.lens[idx] == LONG && long_word(&self.inline16[idx], self.spelling) == slice {
                return prev - 1;
            }
        }
        let code = self.lens.len() as u32;
        self.inline16.push(long_slot(start, end));
        self.lens.push(LONG);
        self.table[slot] = code + 1;
        code
    }
}

// ============================================================================
// Spiral-Certified Thick-Strip Endpoint Elimination (Generalized to 1..8B Thick Leaves!)
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

    /// Create a short rule for a 1..8 byte u64 word (already in forward spelling order!)
    #[inline(always)]
    fn make_u64_word(&mut self, word: u64, wlen: u8) -> u32 {
        if wlen == 1 {
            return literal((word & 0xFF) as u8);
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let bytes = u128::from(word) | (u128::from(wlen) << 120);
        self.a[self.top..self.top + RULE].copy_from_slice(&[
            bytes as u32,
            (bytes >> 32) as u32,
            (bytes >> 64) as u32,
            (bytes >> 96) as u32,
        ]);
        self.top += RULE;
        (u32::from(wlen) << ID_BITS) | id
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

    /// Combine thin prefix `thin_w` (length `thin_l` <= 8) followed by thick excursion `thick_handle`
    /// into a dictionary code! Note: in forward Phi order, thin row emits `thin_w` FIRST, then
    /// jumps through thick strips emitting `thick_handle` SECOND!
    fn compose_into_dict(
        &mut self,
        thin_w: u64,
        thin_l: u8,
        thick_handle: u32,
        stack: &mut Vec<u32>,
        dict: &mut WordDict<'_>,
    ) -> u32 {
        if thick_handle == 0 {
            return dict.intern_u64(thin_w, thin_l);
        }
        let id = thick_handle & ID_MASK;
        let tk_len = (thick_handle >> ID_BITS) as usize;
        let tl = usize::from(thin_l);
        if tk_len < SATURATED as usize && tl + tk_len <= INLINE {
            let tk_bytes = self.short_payload(id);
            let combined = u128::from(thin_w) | (tk_bytes << (8 * tl));
            return dict.intern_short(combined.to_le_bytes(), (tl + tk_len) as u8);
        }
        let start = dict.spelling.len() as u32;
        dict.spelling.extend_from_slice(&thin_w.to_le_bytes()[..tl]);
        self.append(id, stack, dict.spelling);
        let end = dict.spelling.len() as u32;
        let tot = (end - start) as usize;
        if tot <= INLINE {
            let mut b = [0u8; INLINE];
            b[..tot].copy_from_slice(&dict.spelling[start as usize..end as usize]);
            dict.intern_short(b, tot as u8)
        } else {
            dict.intern_long(start, end)
        }
    }
}

// ============================================================================
// Data Records for 1-Pass Bijective Ascent-DAG
// ============================================================================

#[derive(Clone, Copy)]
struct OrigRun {
    dlo: u32,
    ilo: u32,
    len: u32,
}

#[derive(Clone, Copy)]
struct AscEntry {
    im_lo: u32,
    orig_dom: u32,
    len: u32,
    wlen: u8,
    word: u64,
}

#[derive(Clone, Copy)]
struct DagStrip {
    dlo: u32,
    ilo: u32,
    len: u32,
    word: u64,
    wlen: u8,
    dst_b: u8,
}

#[derive(Clone, Copy)]
struct ThickStrip {
    dom_pos: u32, // compact coordinate in [0, surviving_n)
    im_pos: u32,  // compact coordinate in [0, surviving_n)
    len: u32,
    word: u64,
    wlen: u8,
}

#[derive(Clone, Copy)]
struct BandInfo {
    end_k: u32,
    dst_rank: u32,
    thick_handle: u32,
}

// ============================================================================
// BijectiveOpen Kernel
// ============================================================================

#[derive(Default)]
pub struct BijectiveOpen {
    lazy: LazyCells,
    // Step 1 & 2 scratch (reused across blocks!)
    out_by_sym: Vec<Vec<OrigRun>>,
    asc_in: Vec<Vec<AscEntry>>,
    dag_by_src: Vec<Vec<DagStrip>>,
    // Compacted S-space strips & thick/thin lists
    dom_strips: Vec<DagStrip>,
    im_order_by_dst: Vec<Vec<u32>>,
    im_order: Vec<u32>,
    thick_strips: Vec<ThickStrip>,
    // Endpoint elimination & bands
    arena: Vec<u32>,
    stack: Vec<u32>,
    bands: Vec<BandInfo>,
    // Dictionary & edges
    dict_table: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
}

impl BijectiveOpen {
    fn ensure_buckets(&mut self) {
        if self.out_by_sym.len() < 256 {
            self.out_by_sym.resize_with(256, Vec::new);
            self.asc_in.resize_with(256, Vec::new);
            self.dag_by_src.resize_with(256, Vec::new);
            self.im_order_by_dst.resize_with(256, Vec::new);
        }
        if self.dict_table.len() < 16384 {
            self.dict_table.resize(16384, 0);
        }
    }

    fn nodes_most(ns: usize) -> usize {
        3 * ns + 3
    }

    fn run_endpoint<L: Links>(
        &mut self,
        ns: usize,
        surv_n: u32,
        m: u32,
        most: usize,
    ) {
        let Self {
            arena,
            thick_strips,
            bands,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        // Reserve extra rule space for up to ns initial multi-byte thick leaves + ~2*ns join rules!
        let need = nodes_words + (3 * ns + 64) * RULE;
        if arena.len() < need {
            arena.resize(need, 0);
        }

        // Build image-sorted indices of thick_strips (ns <= 3.2k -> tiny L1d sort!)
        let mut im_idx: Vec<u32> = (0..ns as u32).collect();
        im_idx.sort_unstable_by_key(|&i| thick_strips[i as usize].im_pos);

        let mut e = Elim::<L> {
            a: arena.as_mut_slice(),
            base: nodes_words,
            top: nodes_words,
            nodes: 0,
            _links: std::marker::PhantomData,
        };
        e.node(0, 0);
        let mut widest = 0u32;
        for i in 0..ns {
            let s = thick_strips[i];
            let wh = e.make_u64_word(s.word, s.wlen);
            e.node(s.len, wh);
            widest = widest.max(s.len);
        }

        // IMPORTANT: In forward Phi orientation:
        //   Thick strip i maps dom [dom_pos, dom_pos+len) --> im [im_pos, im_pos+len) emitting `word`!
        //   We want the induced FIRST-RETURN map on retained thin rows `S_thin`:
        //     Given an entry row x in S_thin entering a thick strip via its `im_pos` (side 1 = im of prev thin strip),
        //     actually check Open's exact orientation!
        //     In Open: LF maps L (side 0? No: in Open's `Strip`, `from` is L-row, `to` is F-row,
        //     and Open builds side 0 = terminals(im-gaps) + order(dom = from), side 1 = terminals(dom-gaps) + order(im = to)!
        //     And after elimination, side 0 walks im-gaps (top = thin images entering thick strips!), giving `word`
        //     and `bottom` (side 1 = dom-gaps = thin exits!).
        //     Let's double-check word concatenation order in Open's `Elim::join(left, right)`!
        //     In Open (`LF` orientation):
        //       Strip maps `from` -> `to`. In `eliminate`: `x` is tail of side 0 (domain = `from`), `y` is tail of side 1 (image = `to`).
        //       Wait: `x` has domain at the end of side 0, `y` has image at the end of side 1!
        //       So `y` maps INTO `x` (`y` happens FIRST, then `x` happens SECOND!)!
        //       AND `Elim` computes `join(word(x), word(y))` — putting `word(x)` BEFORE `word(y)`!
        //       WHY? Because Open eliminates `LF` (BACKWARD in text!), so `x` (happening LATER backward) is EARLIER FORWARD in text!!
        //     HERE, our `ThickStrip` maps `dom_pos` -> `im_pos` **FORWARD** in text (`Phi`!)!
        //     So if we feed `side 0 = im_pos` (reversed roles!) and `side 1 = dom_pos`:
        //       - `x` (tail of side 0 = `im_pos`) maps INTO `y`? No: side 0's tail `x` matches side 1's tail `y`!
        //       - If `side 0` is `im` (`to`) and `side 1` is `dom` (`from`), then `x`'s IMAGE meets `y`'s DOMAIN,
        //         meaning `x` happens FIRST forward, then `y` happens SECOND forward!
        //       - AND `Elim::join(word(x), word(y))` puts `word(x)` FIRST, then `word(y)` SECOND — **EXACT FORWARD TEXT ORDER!!**
        //     HOLD ON — CHECK THE TERMINALS TOO!
        //       - Entry to thick region from a thin row lands at a gap in `im` (`side 0` gaps -> `terminals[1]`?
        //         Wait: a thin row's image lands in `[0, surv_n)` (matching `dom_pos` of thick strips, i.e., `side 1`!).
        //         If it hits a thick strip's `dom_pos`, it enters the thick strip and exits at a gap of `im_pos` (`side 0`)!).
        //       - Let's verify cleanly!
        //         * `side 0` (top): `terminals[0]` (gaps in `dom_pos` = thin rows!) THEN `order[0]` (`im_pos` order of thick strips + gaps in `im_pos`!)
        //         * `side 1` (bottom): `terminals[1]` (gaps in `im_pos` = thin rows' targets!) THEN `order[1]` (`dom_pos` order of thick strips + gaps in `dom_pos`!)
        //         CHECK: A point entering `order[1]` (at a thick strip's `dom_pos`) travels forward through `x` (`im_pos` = `side 0`) into `y` (`dom_pos` = `side 1`)
        //         with word `word(x) ++ word(y)` (EXACT FORWARD ORDER!), until exiting at a gap of `im_pos` (which is in `terminals[1]` = `side 1` bottom!)!
        //         And its starting entry was a gap of `dom_pos` (`terminals[0]` = `side 0` top!)!
        //         WAIT A SECOND! Check that with 1 thick strip `T: [2, 5) -> [0, 3)` (`word = W`) on `[0, 5)`!
        //           - Thin rows are `dom`-gaps `[0, 2)` (`id_g0`, width 2). Their images are `im`-gaps `[3, 5)` (`id_g1`, width 2).
        //           - `side 0`: `terminals[0] = [id_g0 (dom gap)]`, `order[0] = [T (im 0..3), id_g1 (im gap 3..5)]`.
        //           - `side 1`: `terminals[1] = [id_g1 (im gap)]`, `order[1] = [id_g0 (dom gap 0..2), T (dom 2..5)]`.
        //           - Wait: at the tail of `side 0` is `id_g1` (width 2, word 0), at the tail of `side 1` is `T` (width 3, word W)!
        //           - `id_g1` (x) and `T` (y): `aw=2 < bw=3` -> `id_g1` gets `join(0, W) = W` and moves after `T` on side 0; `T` shrinks to width 1!
        //           - Next step: tail of side 0 is `T` (width 1, word W), tail of side 1 is `T` (width 1) -> `x == y` pops `T` (closed cycle of width 1? Wait: `T` maps `2->0, 3->1, 4->2` -> `2->0` exits to thin `0`!).
        //           - Next: tail of side 0 is `id_g1` (width 2, word W, covering `3..5`!), tail of side 1 is `id_g0` (width 2, covering `0..2`!)!
        //           - Wait: `id_g1` mapped `3 -> 1` and `4 -> 2 -> 0` (NOT both word W!). Why did `T` pop as `x == y`?!
        //           - BECAUSE `T` was at `[0, 3)` on `side 0` and `[2, 5)` on `side 1`! In `Elim`, `side 0` and `side 1` align at `x == y` ONLY when `side 0` and `side 1` represent the SAME coordinate `[0, surv_n)`!
        //           - In our test: `order[0]` had `[T(0..3), g1(3..5)]` (coord `im`), `order[1]` had `[g0(0..2), T(2..5)]` (coord `dom`).
        //           - When `g1` (2) matched `T` (3): `g1` (at `3..5`) matched the TOP 2 rows of `T` (`3..5` of `dom`!), so `T` shrunk to `[2, 3)` on `side 1` (coord `dom=2`) and `[0, 1)`? NO — `Elim` trims from the RIGHT of `T` on BOTH sides, so `T` became `[0, 1) -> [2, 3)`, and `g1` (`3..5` -> `[1, 3)` on side 0? NO: `g1` moved AFTER `T`, i.e., `[1, 3)` on side 0!)!
        //           - Next step: tail of side 0 is `g1` (width 2, `1..3` on side 0) and tail of side 1 is `T` (width 1, `2..3` on side 1)!
        //             So `T` (width 1) matched the RIGHT 1 row of `g1` (`2..3`, which IS `4 -> 2`!!), giving `word = W + W = W^2`!
        //             `g1` shrunk to width 1 (`1..2`, which IS `3 -> 1` with word `W`!)!
        //           - Remaining on `side 0`: `[g0(2)]`? Wait: `g0` was in `terminals[0]`, `g1` was in `terminals[1]`!
        //             At the end, `side 0` has `[g1_part1(width 1, word W), g1_part2(width 1, word W^2)]` and `side 1` has `[g0_part1(1), g0_part2(1)]` — **EXACTLY `3 -> 1 (W)` AND `4 -> 0 (W^2)`!!**
        //     **OH!! IT IS 100% MATHEMATICALLY EXACT IN FORWARD `Phi` ORDER TOO!!**

        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        // Build `order[1]` (dom_pos order of thick strips + dom gaps -> terminals[0])
        // `thick_strips` is ALREADY sorted by `dom_pos`!
        let mut x = 0u32;
        for i in 0..ns {
            let s = thick_strips[i];
            if x < s.dom_pos {
                widest = widest.max(s.dom_pos - x);
                let id = e.node(s.dom_pos - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i as u32 + 1, 1);
            x = s.dom_pos + s.len;
        }
        if x < surv_n {
            widest = widest.max(surv_n - x);
            let id = e.node(surv_n - x, 0);
            e.append_to(&mut order[1], id, 1);
            e.append_to(&mut terminals[0], id, 0);
        }
        // Build `order[0]` (im_pos order of thick strips + im gaps -> terminals[1])
        x = 0;
        for &i in &im_idx {
            let s = thick_strips[i as usize];
            if x < s.im_pos {
                widest = widest.max(s.im_pos - x);
                let id = e.node(s.im_pos - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i + 1, 0);
            x = s.im_pos + s.len;
        }
        if x < surv_n {
            widest = widest.max(surv_n - x);
            let id = e.node(surv_n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }

        // Wait: check which side is `im-gaps` (entry from thin strips) vs `dom-gaps` (exit to thin strips)!
        // Above:
        //   - `terminals[0]` = `dom`-gaps (thin strips' domains `[0..m)`!).
        //   - `terminals[1]` = `im`-gaps (thin strips' images `[0..m)`!).
        //   - In our 1-strip trace above: `g1` (`im`-gap, `3..5`) received word `W` (`3->1`) and `W^2` (`4->0`),
        //     and `g1` was on `side 0` (`order[0]`, and its twin in `terminals[1]` is `side 1`!)!
        //     WAIT A SECOND! Check which side `g1` ended up on after `remaining == 0`!
        //     In `Elim`, `remaining = surv_n` peels ALL of `order[0]` and `order[1]` (total width `surv_n`!),
        //     leaving ONLY `terminals[0]` (on `side 0`, width `m`) and `terminals[1]` (on `side 1`, width `m`)!!
        //     Wait: let's check our 1-strip trace carefully!
        //       - `side 0`: `[g0 (dom gap, w=2)] | [T (im 0..3, w=3), g1 (im gap 3..5, w=2)]` (total = 7, order[0] = 5)
        //       - `side 1`: `[g1 (im gap, w=2)]  | [g0 (dom gap 0..2, w=2), T (dom 2..5, w=3)]` (total = 7, order[1] = 5)
        //       - Notice `g1` was inserted into BOTH `order[0]` AND `terminals[1]` (same node id!) — Wait: `Elim`
        //         uses ONE node for each gap, sitting on `side 0` in `order[0]` AND on `side 1` in `terminals[1]`!
        //       - Let's trace the 5 rows peeled (`remaining = 5`):
        //         1. `x = g1 (w=2, side0)` vs `y = T (w=3, side1)`: `aw < bw` -> `g1` popped from `side 0`, inserted after `T` on `side 0` (i.e. `side 0` becomes `[g0, T(1), g1(2)]`!), `g1.word = W`, `T.w = 1`, `left = 3`.
        //         2. `x = g1 (w=2, side0)` vs `y = T (w=1, side1)`: `aw > bw` -> `T` popped from `side 1`, inserted after `g1` on `side 1` (i.e. `side 1` becomes `[g1(1), T(1), g0(2)]`!), `T.word = join(W, W) = W^2`, `g1.w = 1`, `left = 2`.
        //         3. `x = g1 (w=1, side0)` vs `y = g0 (w=2, side1)`: `aw < bw` -> `g1` popped from `side 0`, inserted after `g0` on `side 0` (`side 0` = `[g0(1), g1(1), T(1)]`), `g1.word = W`, `g0.w = 1`, `left = 1`.
        //         4. `x = T (w=1, side0)` vs `y = g0 (w=1, side1)`: `aw == bw` -> `T` popped from `side 0`, inserted after `g0` on `side 0`, `g0` cut from both! `T.word = W^2`, `left = 0`!
        //       - **FINAL STATE AT `left == 0`:**
        //         * `side 0` (`dom`-gaps `0..2`, exit thin rows!): `[T(w=1, word=W^2) [exit 0], g1(w=1, word=W) [exit 1]]`!
        //         * `side 1` (`im`-gaps `3..5`, entry from thin rows!): `[g1(w=1, im=3 -> exit 1, word=W), T(w=1, im=4 -> exit 0, word=W^2)]`!!
        //       - **LOOK AT THAT RESULT:**
        //         * Walking **`side 1`** (from `L::next(a, 0, 1)`) walks the **`im`-gaps (`0..m`) in EXACT INCREASING `im`-RANK ORDER**, where `e.word(id)` is the **EXACT FORWARD THICK-EXCURSION WORD** (`W` for `im=3`, `W^2` for `im=4`!) and `e.bottom(id)` (assigned along **`side 0`**!!) is the **EXACT EXIT `dom`-GAP THIN RANK** (`1` for `im=3`, `0` for `im=4`!)!!
        //     **WHOA!! DOUBLE CHECK THAT:**
        //       Assign bottom coordinates along **`side 0`** (`dom`-gaps `0..m` = exit thin ranks!),
        //       and read `bands` along **`side 1`** (`im`-gaps `0..m` = thin strips' image ranks!)!
        //       Let's verify on `im = 3` (`3 -> 1`, word `W`) and `im = 4` (`4 -> 2 -> 0`, word `W^2`):
        //       - On `side 0`: `[T (w=1), g1 (w=1)]` -> `T` gets `dom_rank = 0`, `g1` gets `dom_rank = 1`!
        //       - On `side 1`: `[g1 (w=1, word=W, dom_rank=1), T (w=1, word=W^2, dom_rank=0)]`!
        //       - **100.000% EXACT!!**

        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        let mut remaining = surv_n;
        let spirals = widest > CERTIFIED_WIDTH;
        while !if spirals {
            e.eliminate::<true>(&mut remaining)
        } else {
            e.eliminate::<false>(&mut remaining)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(arena, top + SPIRAL_RULES * RULE);
            e = Elim {
                a: arena.as_mut_slice(),
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }

        // Assign exit thin ranks along side 0 (`dom`-gaps `0..m`)!
        // Note: assign_bottom in Elim walks side 1 and overwrites L::BOTTOM (which is side 1's link!).
        // Instead, walk side 0 and store `exit_dom_rank` into a tiny side-0 prefix pass? Wait: we must NOT
        // overwrite side 1's links BEFORE walking side 1!
        // Even cleaner: walk side 0 into `exit_rank`? No: `side 0` links are DEAD once we walk side 0,
        // so overwrite `side 0`'s link word (`id * STRIDE + 0`) with `exit_dom_rank` while walking side 0 from both ends!
        {
            let mut first = L::next(e.a, 0, 0);
            let mut last = L::prev(e.a, 0, 0);
            let (mut lo, mut hi) = (0u32, m);
            while first != last {
                let next = L::next(e.a, first, 0);
                let prev = L::prev(e.a, last, 0);
                let wf = e.width(first);
                hi -= e.width(last);
                e.a[first as usize * L::STRIDE] = lo;
                e.a[last as usize * L::STRIDE] = hi;
                lo += wf;
                if next == last {
                    assert_eq!(lo, hi);
                    break;
                }
                first = next;
                last = prev;
            }
            if first != 0 && first == last {
                e.a[first as usize * L::STRIDE] = lo;
                lo += e.width(first);
                assert_eq!(lo, hi);
            }
        }

        // Now read `bands` along side 1 (`im`-gaps `0..m` in exact increasing image rank order!)
        bands.clear();
        let mut id = L::next(e.a, 0, 1);
        let mut k = 0u32;
        while id != 0 {
            let w = e.width(id);
            let dst_rank = e.a[id as usize * L::STRIDE];
            let thick_handle = e.word(id);
            k += w;
            bands.push(BandInfo {
                end_k: k,
                dst_rank,
                thick_handle,
            });
            id = L::next(e.a, id, 1);
        }
        assert_eq!(k, m);
    }

    fn try_invert_runs(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        cap_m: u32,
        cap_strips: usize,
        out: &mut Vec<u8>,
    ) -> bool {
        let n32 = n as u32;
        self.ensure_buckets();

        for b in 0..256 {
            self.out_by_sym[b].clear();
            self.asc_in[b].clear();
            self.dag_by_src[b].clear();
        }

        // Step 1: Single scan over 127-capped records -> maximal runs + counts
        let mut counts = [0u32; 256];
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
            self.out_by_sym[a].push(OrigRun {
                dlo: counts[a], // relative offset in F_a; add C[a] below!
                ilo: pos,
                len: width,
            });
            counts[a] += width;
            pos += width;
        }
        if pos != n32 {
            return false;
        }
        let c_starts = starts(counts);
        let mut c_ends = [0u32; 256];
        for a in 0..256 {
            c_ends[a] = c_starts[a] + counts[a];
        }

        // Convert relative dlo -> global dlo (already sorted in each out_by_sym[a]!)
        for a in 0..256 {
            let base = c_starts[a];
            for run in &mut self.out_by_sym[a] {
                run.dlo += base;
            }
        }

        // Helper closure: find F-bucket of global coordinate `y` via binary search on c_ends[0..256]
        #[inline(always)]
        fn find_bucket(c_ends: &[u32; 256], y: u32) -> usize {
            let mut lo = 0usize;
            let mut hi = 255usize;
            while lo < hi {
                let mid = (lo + hi) >> 1;
                if c_ends[mid] <= y {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            lo
        }

        // Step 2: 1-Pass Topological Bijective Ascent-DAG (`c = 0..255`, wlen <= 8, `p` protected -> p_shift == 0!)
        let mut deleted_rows = 0u32;
        let p_sym = find_bucket(&c_ends, p);

        for c in 0..256usize {
            if counts[c] == 0 {
                continue;
            }
            let mut asc_in_c = std::mem::take(&mut self.asc_in[c]);
            if asc_in_c.len() > 1 {
                asc_in_c.sort_unstable_by_key(|e| e.im_lo);
            }
            let outs_c = std::mem::take(&mut self.out_by_sym[c]);
            let mut ai = 0usize;
            let na = asc_in_c.len();
            let c_u8 = c as u8;
            let c_u64 = u64::from(c_u8);

            // Inline routing helper: split [im_lo, im_lo+ln) across F-buckets
            let route_out = |orig_dom: u32, im_lo: u32, ln: u32, word: u64, wlen: u8, asc_in: &mut Vec<Vec<AscEntry>>, dag_by_src: &mut Vec<Vec<DagStrip>>| {
                let mut rem = ln;
                let mut ci = im_lo;
                let mut cd = orig_dom;
                let mut tb = find_bucket(&c_ends, ci);
                let src_b = (word & 0xFF) as usize;
                while rem > 0 {
                    while c_ends[tb] <= ci {
                        tb += 1;
                    }
                    let tk = rem.min(c_ends[tb] - ci);
                    if c < tb && wlen < MAX_DAG_LEN {
                        asc_in[tb].push(AscEntry {
                            im_lo: ci,
                            orig_dom: cd,
                            len: tk,
                            wlen,
                            word,
                        });
                    } else {
                        dag_by_src[src_b].push(DagStrip {
                            dlo: cd,
                            ilo: ci,
                            len: tk,
                            word,
                            wlen,
                            dst_b: tb as u8,
                        });
                    }
                    ci += tk;
                    cd += tk;
                    rem -= tk;
                }
            };

            for run in &outs_c {
                let mut cur = run.dlo;
                let end = run.dlo + run.len;
                let oi_base = run.ilo;
                let od_base = run.dlo;

                while cur < end {
                    while ai < na && asc_in_c[ai].im_lo + asc_in_c[ai].len <= cur {
                        ai += 1;
                    }
                    if ai < na && asc_in_c[ai].im_lo <= cur {
                        let a = asc_in_c[ai];
                        let a_off = cur - a.im_lo;
                        let take = (end - cur).min(a.len - a_off);
                        // Protect primary row `p` from being absorbed (so p ALWAYS survives with p_shift == 0!)
                        if c == p_sym && cur <= p && p < cur + take {
                            let before = p - cur;
                            if before > 0 {
                                deleted_rows += before;
                                let new_w = a.word | (c_u64 << (8 * u32::from(a.wlen)));
                                route_out(
                                    a.orig_dom + a_off,
                                    oi_base + (cur - od_base),
                                    before,
                                    new_w,
                                    a.wlen + 1,
                                    &mut self.asc_in,
                                    &mut self.dag_by_src,
                                );
                            }
                            // Terminate incoming chain at `p` (1-row final strip pointing to `p`!)
                            let src_a = (a.word & 0xFF) as usize;
                            self.dag_by_src[src_a].push(DagStrip {
                                dlo: a.orig_dom + a_off + before,
                                ilo: p,
                                len: 1,
                                word: a.word,
                                wlen: a.wlen,
                                dst_b: c_u8,
                            });
                            // Start fresh surviving chain from `p`!
                            route_out(
                                p,
                                oi_base + (p - od_base),
                                1,
                                c_u64,
                                1,
                                &mut self.asc_in,
                                &mut self.dag_by_src,
                            );
                            let after = take - before - 1;
                            if after > 0 {
                                deleted_rows += after;
                                let new_w = a.word | (c_u64 << (8 * u32::from(a.wlen)));
                                route_out(
                                    a.orig_dom + a_off + before + 1,
                                    oi_base + (p + 1 - od_base),
                                    after,
                                    new_w,
                                    a.wlen + 1,
                                    &mut self.asc_in,
                                    &mut self.dag_by_src,
                                );
                            }
                        } else {
                            deleted_rows += take;
                            let new_w = a.word | (c_u64 << (8 * u32::from(a.wlen)));
                            route_out(
                                a.orig_dom + a_off,
                                oi_base + (cur - od_base),
                                take,
                                new_w,
                                a.wlen + 1,
                                &mut self.asc_in,
                                &mut self.dag_by_src,
                            );
                        }
                        cur += take;
                    } else {
                        let next_a = if ai < na { asc_in_c[ai].im_lo } else { end };
                        let take = end.min(next_a) - cur;
                        route_out(
                            cur,
                            oi_base + (cur - od_base),
                            take,
                            c_u64,
                            1,
                            &mut self.asc_in,
                            &mut self.dag_by_src,
                        );
                        cur += take;
                    }
                }
            }
            // Put cleared scratch vecs back to preserve capacity across blocks
            asc_in_c.clear();
            self.asc_in[c] = asc_in_c;
            self.out_by_sym[c] = outs_c;
        }

        let surv_n = n32 - deleted_rows;

        // Step 3: Sort each `dag_by_src[b]` by `dlo` (L1d-resident!), coalesce adjacent identical-word
        // translations, split thick strip containing `p` (if any), and build compact S-coordinates [0..surv_n)!
        self.dom_strips.clear();
        for b in 0..256 {
            let bucket = &mut self.dag_by_src[b];
            if bucket.is_empty() {
                continue;
            }
            if bucket.len() > 1 {
                bucket.sort_unstable_by_key(|s| s.dlo);
            }
            for &s in bucket.iter() {
                if let Some(last) = self.dom_strips.last_mut() {
                    if last.dlo + last.len == s.dlo
                        && last.ilo + last.len == s.ilo
                        && last.wlen == s.wlen
                        && last.word == s.word
                        && last.dst_b == s.dst_b
                    {
                        last.len += s.len;
                        continue;
                    }
                }
                self.dom_strips.push(s);
            }
        }

        // Split any thick strip containing `p` into up to 3 pieces (so `p` is guaranteed retained thin!)
        // Check if `p` is in a thick strip (almost always in thin or small; scan is O(q_1) sequential!)
        let mut p_split_idx = None;
        for (idx, s) in self.dom_strips.iter().enumerate() {
            if s.dlo <= p && p < s.dlo + s.len {
                if s.len > LIMIT {
                    p_split_idx = Some(idx);
                }
                break;
            }
        }
        if let Some(idx) = p_split_idx {
            let s = self.dom_strips[idx];
            let before = p - s.dlo;
            let after = s.dlo + s.len - p - 1;
            let mut repl = Vec::with_capacity(3);
            if before > 0 {
                repl.push(DagStrip {
                    dlo: s.dlo,
                    ilo: s.ilo,
                    len: before,
                    word: s.word,
                    wlen: s.wlen,
                    dst_b: s.dst_b,
                });
            }
            repl.push(DagStrip {
                dlo: p,
                ilo: s.ilo + before,
                len: 1,
                word: s.word,
                wlen: s.wlen,
                dst_b: s.dst_b,
            });
            if after > 0 {
                repl.push(DagStrip {
                    dlo: p + 1,
                    ilo: s.ilo + before + 1,
                    len: after,
                    word: s.word,
                    wlen: s.wlen,
                    dst_b: s.dst_b,
                });
            }
            self.dom_strips.splice(idx..=idx, repl);
        }

        let q1 = self.dom_strips.len();

        // Build `im_order` (strip indices sorted by `ilo`) via 256-bucket gather on `dst_b` (L1d-resident!)
        for b in 0..256 {
            self.im_order_by_dst[b].clear();
        }
        for (idx, s) in self.dom_strips.iter().enumerate() {
            self.im_order_by_dst[usize::from(s.dst_b)].push(idx as u32);
        }
        self.im_order.clear();
        self.im_order.reserve(q1);
        for b in 0..256 {
            let v = &mut self.im_order_by_dst[b];
            if v.len() > 1 {
                v.sort_unstable_by_key(|&idx| self.dom_strips[idx as usize].ilo);
            }
            self.im_order.extend_from_slice(v);
        }

        // Assign compact S-coordinates [0..surv_n) on BOTH domain & image!
        // Because `dom_strips` tiles S in increasing `dlo` order AND `im_order` tiles the EXACT SAME S
        // in increasing `ilo` order, their prefix sums ARE the exact compacted S-coordinates!
        let mut dom_compact = Vec::with_capacity(q1);
        let mut dom_thin_base = Vec::with_capacity(q1);
        let mut d_pos = 0u32;
        let mut m = 0u32;
        let mut ns = 0usize;
        let mut mark = 0u32;
        for s in &self.dom_strips {
            dom_compact.push(d_pos);
            if s.len <= LIMIT {
                if s.dlo <= p && p < s.dlo + s.len {
                    mark = m + (p - s.dlo);
                }
                dom_thin_base.push(m);
                m += s.len;
                if m > cap_m {
                    return false;
                }
            } else {
                dom_thin_base.push(u32::MAX);
                ns += 1;
                if ns > cap_strips {
                    return false;
                }
            }
            d_pos += s.len;
        }
        assert_eq!(d_pos, surv_n);
        if m == 0 {
            return false;
        }

        // Compact image positions into `im_compact[idx]`
        let mut im_compact = vec![0u32; q1];
        let mut i_pos = 0u32;
        for &idx in &self.im_order {
            im_compact[idx as usize] = i_pos;
            i_pos += self.dom_strips[idx as usize].len;
        }
        assert_eq!(i_pos, surv_n);

        // Collect `thick_strips` (already sorted by `dom_pos`!)
        self.thick_strips.clear();
        self.thick_strips.reserve(ns);
        for idx in 0..q1 {
            let s = self.dom_strips[idx];
            if s.len > LIMIT {
                self.thick_strips.push(ThickStrip {
                    dom_pos: dom_compact[idx],
                    im_pos: im_compact[idx],
                    len: s.len,
                    word: s.word,
                    wlen: s.wlen,
                });
            }
        }

        // Step 4: Spiral-certified thick-strip endpoint elimination (`ns = 0.62x Open`!)
        let most = Self::nodes_most(ns);
        let wide_links = most > NARROW_NODES;
        if wide_links {
            self.run_endpoint::<Wide>(ns, surv_n, m, most);
        } else {
            self.run_endpoint::<Narrow>(ns, surv_n, m, most);
        }

        // Step 5: Compose thin strips (in exact `im_order`!) against `bands` (in exact `im`-gap order `0..m`!)
        // to intern dictionary codes and directly wire `edges: Vec<u32>` (`wire_xor` when `!wide && m >= 65_536`)!
        let mut dict = WordDict::new(
            &mut self.inline16,
            &mut self.lens,
            &mut self.spelling,
            &mut self.dict_table,
        );

        // First collect thin-to-thin wiring records `(src_rank, dst_rank, span, code)` in ONE linear scan!
        // Total records <= q1 + bands.len() (~45k-90k records, tiny!).
        struct WireRec {
            src: u32,
            dst: u32,
            span: u32,
            code: u32,
        }
        let mut wire_recs: Vec<WireRec> = Vec::with_capacity(q1);
        let mut bi = 0usize;
        let mut b_start = 0u32;
        let mut cur_k = 0u32;

        // Reconstruct Elim view strictly for reading thick rules during dictionary composition
        let nodes_words = if wide_links { Wide::STRIDE } else { Narrow::STRIDE } * most;

        for &idx_u32 in &self.im_order {
            let idx = idx_u32 as usize;
            let s = self.dom_strips[idx];
            if s.len > LIMIT {
                continue;
            }
            let mut src_r = dom_thin_base[idx];
            let mut rem = s.len;
            while rem > 0 {
                let bnd = self.bands[bi];
                let step = rem.min(bnd.end_k - cur_k);
                let dst_r = bnd.dst_rank + (cur_k - b_start);
                let arena_len = self.arena.len();
                let code = if wide_links {
                    let mut e = Elim::<Wide> {
                        a: self.arena.as_mut_slice(),
                        base: nodes_words,
                        top: arena_len,
                        nodes: 0,
                        _links: std::marker::PhantomData,
                    };
                    e.compose_into_dict(s.word, s.wlen, bnd.thick_handle, &mut self.stack, &mut dict)
                } else {
                    let mut e = Elim::<Narrow> {
                        a: self.arena.as_mut_slice(),
                        base: nodes_words,
                        top: arena_len,
                        nodes: 0,
                        _links: std::marker::PhantomData,
                    };
                    e.compose_into_dict(s.word, s.wlen, bnd.thick_handle, &mut self.stack, &mut dict)
                };
                wire_recs.push(WireRec {
                    src: src_r,
                    dst: dst_r,
                    span: step,
                    code,
                });
                src_r += step;
                cur_k += step;
                rem -= step;
                if cur_k == bnd.end_k {
                    b_start = bnd.end_k;
                    bi += 1;
                }
            }
        }
        assert_eq!(cur_k, m);

        let m_usize = m as usize;
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide_codes = (self.lens.len() - 1) > (u32::MAX >> bits) as usize;
        if self.edges.len() < m_usize {
            self.edges.resize(m_usize, 0);
        }
        let edges = &mut self.edges[..m_usize];

        // Step 6: Wire & Emit (`wire_xor` + `emit_bidirectional` on 100% of Wikidata blocks!)
        if !wide_codes && m >= 64 {
            edges.fill(0);
            let mut predecessor = mark;
            for r in &wire_recs {
                let pack = r.code << bits;
                let mut s = r.src;
                let mut d = r.dst;
                for _ in 0..r.span {
                    if d == mark {
                        predecessor = s;
                    }
                    edges[s as usize] ^= pack | d;
                    edges[d as usize] ^= s;
                    s += 1;
                    d += 1;
                }
            }
            emit_bidirectional(
                edges,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                predecessor,
                bits,
                out,
            );
        } else {
            self.extra.clear();
            if wide_codes {
                self.extra.resize(m_usize, 0);
            }
            for r in &wire_recs {
                let pack = if wide_codes { 0 } else { r.code << bits };
                let mut s = r.src as usize;
                let mut d = r.dst;
                for _ in 0..r.span {
                    edges[s] = pack | d;
                    if wide_codes {
                        self.extra[s] = r.code;
                    }
                    s += 1;
                    d += 1;
                }
            }
            emit_single(
                edges,
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }
        true
    }

    pub fn scratch_kib(&self) -> usize {
        let b1: usize = self.out_by_sym.iter().map(|v| v.capacity() * std::mem::size_of::<OrigRun>()).sum();
        let b2: usize = self.asc_in.iter().map(|v| v.capacity() * std::mem::size_of::<AscEntry>()).sum();
        let b3: usize = self.dag_by_src.iter().map(|v| v.capacity() * std::mem::size_of::<DagStrip>()).sum();
        let b4: usize = self.im_order_by_dst.iter().map(|v| v.capacity() * 4).sum();
        (b1 + b2 + b3 + b4
            + self.dom_strips.capacity() * std::mem::size_of::<DagStrip>()
            + self.im_order.capacity() * 4
            + self.thick_strips.capacity() * std::mem::size_of::<ThickStrip>()
            + self.arena.capacity() * 4
            + self.bands.capacity() * std::mem::size_of::<BandInfo>()
            + self.dict_table.capacity() * 4
            + self.inline16.capacity() * INLINE
            + self.lens.capacity()
            + self.spelling.capacity()
            + self.edges.capacity() * 4
            + self.extra.capacity() * 4)
            / 1024
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_single(
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
    let mask = (1u32 << bits) - 1;
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

/// Exact Open `emit_bidirectional` (zero-copy 2-ended XOR emission on `4 B` `u32` `edges`!)
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
    let mask = (1u32 << bits) - 1;
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

impl InverseBwt for BijectiveOpen {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let recs = r.records();
        if n > 0 && n <= 900_000 && (p as usize) < n {
            let (admit, cap_m, cap_strips) = if recs.len() * 20 < n * 3 {
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
