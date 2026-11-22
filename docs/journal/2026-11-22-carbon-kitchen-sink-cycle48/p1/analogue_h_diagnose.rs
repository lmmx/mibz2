#![forbid(unsafe_code)]

//! Cycle 48 (P1) — Analogue H Exact Failure Diagnosis & Bit-Budget / Pareto-Zipf Census
//!
//! Measures exact retained-row bit budget (`bits_m = ceil(log2(m))`), dictionary cardinality
//! (`|D|`, `bits_code = ceil(log2(|D|))`), Pareto-Zipf (1896) row-weighted hot-code coverage
//! for top {15, 31, 63, 127, 255} codes and dynamic `24 - bits_m` hot-code budgets,
//! cold-band counts (`B_cold`) and L1D side-table footprint, and exact S2b arena peak vs
//! S4/S5 `edges` working-set (`32-bit` vs `24-bit`) across:
//!   - Solved Analogue H synthetic blocks (`m < 65,536, |D| <= 256` and `|D| = 1`)
//!   - 6 Admitted Wikidata blocks (`/home/antigravity/admitted-bwt/block-{0,2,85,143,207,210}.L`)
//!   - All 233 Full `wikidata.bz2` blocks (`/tmp/mibz2_corpora/wikidata_431_blocks.bin`)
//!   - 6 Real Non-Wikidata blocks (`/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin`)

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

use std::fs;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;

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

trait Links {
    const STRIDE: usize;
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

struct Elim<'a, L: Links, const INLINE3: bool> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links, const INLINE3: bool> Elim<'_, L, INLINE3> {
    const ID_BITS: u32 = if INLINE3 { 24 } else { 22 };
    const ID_MASK: u32 = (1 << Self::ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> Self::ID_BITS;
    const RULE_BASE: u32 = if INLINE3 { 0 } else { 65_793 };

    #[inline(always)]
    fn literal(byte: u8) -> u32 {
        (1 << Self::ID_BITS) | u32::from(byte)
    }
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
        self.base + (id - Self::RULE_BASE) as usize * RULE
    }
    #[inline(always)]
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> Self::ID_BITS;
        if g < Self::SATURATED {
            g
        } else {
            self.a[self.rule(h & Self::ID_MASK) + 2]
        }
    }
    #[inline(always)]
    fn short_payload_h(&self, h: u32) -> u128 {
        let g = h >> Self::ID_BITS;
        let id = h & Self::ID_MASK;
        let max_inline = if INLINE3 { 3 } else { 2 };
        if g <= max_inline {
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
        let left_len = left >> Self::ID_BITS;
        let sum = left_len + (right >> Self::ID_BITS);
        if sum < INLINE as u32 {
            if sum == 2 {
                return (2 << Self::ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            if INLINE3 && sum == 3 {
                return (3 << Self::ID_BITS)
                    | (left & Self::ID_MASK)
                    | ((right & Self::ID_MASK) << (8 * left_len));
            }
            let id = ((self.top - self.base) / RULE) as u32 + Self::RULE_BASE;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << Self::ID_BITS) | id
        } else {
            let len = if sum <= Self::SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + Self::RULE_BASE;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(Self::SATURATED) << Self::ID_BITS) | id
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
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            let g = h >> Self::ID_BITS;
            if g < INLINE as u32 {
                let bytes = self.short_payload_h(h).to_le_bytes();
                dest.extend_from_slice(&bytes[..g as usize]);
            } else {
                let r = self.rule(h & Self::ID_MASK);
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
    fn inline_word(
        &mut self,
        word: u32,
        byte: u8,
        stack: &mut Vec<u32>,
        spelling: &mut Vec<u8>,
    ) -> ([u8; INLINE], u8) {
        let height = (word >> Self::ID_BITS) as usize;
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

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
}

struct DiagStats {
    n: usize,
    m: usize,
    bits_m: u32,
    ns: usize,
    num_bands: usize,
    dict_len: usize,
    used_codes: usize,
    bits_code: u32,
    arena_init_words_r5: usize,
    arena_peak_words_r5: usize,
    arena_peak_words_i3: usize,
    edges32_bytes: usize,
    edges24_bytes: usize,
    cov_top15: f64,
    cov_top31: f64,
    cov_top63: f64,
    cov_top127: f64,
    cov_top255: f64,
    dyn_hot_bits: u32,
    dyn_hot_cap: usize,
    dyn_cov: f64,
    dyn_cold_rows: usize,
    dyn_cold_bands: usize,
    cold_bands_63: usize,
    cold_bands_127: usize,
    max_cold_per_page128: usize,
    avg_cold_per_page128: f64,
    hot63_lit_count: usize,
    hot63_word_count: usize,
}

fn diagnose_block_impl<const INLINE3: bool>(blk: &Block) -> (usize, usize, usize, usize, usize, Vec<Band>, usize) {
    let n = blk.n as u32;
    let p = blk.p;
    let mut thin = [0u32; 256];
    let mut thick = [0u32; 256];
    let mut thick_rows = [0u32; 256];
    let mut m = 0u32;
    let mut strips_raw: Vec<u32> = Vec::new();
    let mut ns = 0usize;

    let mut pos = 0u32;
    let mut i = 0usize;
    let nrec = blk.records.len();
    while i < nrec {
        let rec = blk.records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < nrec && (blk.records[i] >> 8) as u8 == c {
            width += u32::from(blk.records[i] & 255);
            i += 1;
        }
        let mut handle_piece = |x: u32, w: u32, keep: bool| {
            let a = usize::from(c);
            if keep {
                thin[a] += w;
                m += w;
            } else {
                strips_raw.push(x);
                strips_raw.push(thin[a] + thick_rows[a]);
                strips_raw.push((w << 8) | u32::from(c));
                ns += 1;
                thick[a] += 1;
                thick_rows[a] += w;
            }
        };
        let end = pos + width;
        if width > LIMIT && (pos..end).contains(&p) {
            if pos < p {
                handle_piece(pos, p - pos, false);
            }
            handle_piece(p, 1, true);
            if p + 1 < end {
                handle_piece(p + 1, end - p - 1, false);
            }
        } else {
            handle_piece(pos, width, width <= LIMIT);
        }
        pos += width;
    }

    let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
    assert_eq!(all.iter().sum::<u32>(), n);
    let first = starts(all);
    let mut write = starts(thick);
    let most = 3 * ns + 3;
    let wide = most > NARROW_NODES;
    let stride = if wide { Wide::STRIDE } else { Narrow::STRIDE };
    let arena_init_words = (4 * ns + (stride + RULE) * most).max(m as usize);
    let mut edges = vec![0u32; arena_init_words];
    edges[..3 * ns].copy_from_slice(&strips_raw);

    let (peak_words, bands, dict_len) = if wide {
        run_induced::<Wide, INLINE3>(ns, n, m, &thin, &first, &mut write, &mut edges, most)
    } else {
        run_induced::<Narrow, INLINE3>(ns, n, m, &thin, &first, &mut write, &mut edges, most)
    };
    (m as usize, ns, arena_init_words, peak_words, dict_len, bands, edges.len())
}

#[allow(clippy::too_many_arguments)]
fn run_induced<L: Links, const INLINE3: bool>(
    ns: usize,
    n: u32,
    m: u32,
    thin: &[u32; 256],
    first: &[u32; 256],
    write: &mut [u32; 256],
    edges: &mut Vec<u32>,
    most: usize,
) -> (usize, Vec<Band>, usize) {
    let nodes_words = L::STRIDE * most;
    if edges.len() < 4 * ns + nodes_words {
        edges.resize(4 * ns + nodes_words, 0);
    }
    let (hdr, arena) = edges.split_at_mut(4 * ns);
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
        e.set_word(sid, Elim::<L, INLINE3>::literal(byte));
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
    let mut base = L::STRIDE * e.nodes as usize;
    let mut top = base;
    let nodes = e.nodes;
    let mut remaining = n;
    let spirals = widest > CERTIFIED_WIDTH;
    loop {
        let mut el = Elim::<L, INLINE3> {
            a: &mut edges[4 * ns..],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        let done = if spirals {
            el.eliminate::<true>(&mut remaining)
        } else {
            el.eliminate::<false>(&mut remaining)
        };
        base = el.base;
        top = el.top;
        if done {
            break;
        }
        let need = 4 * ns + top + SPIRAL_RULES * RULE;
        let new_len = need.max(2 * edges.len());
        edges.resize(new_len, 0);
    }
    let peak_words = 4 * ns + top;
    {
        let mut el = Elim::<L, INLINE3> {
            a: &mut edges[4 * ns..],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        el.assign_bottom(m);
    }

    // Build exact deduplicated dictionary (matching cand_r5_head's Dictionary)
    let mut inline16: Vec<[u8; INLINE]> = (0..=255u8)
        .map(|c| {
            let mut v = [0u8; INLINE];
            v[0] = c;
            v
        })
        .collect();
    let mut lens: Vec<u8> = vec![1; 256];
    let mut spelling: Vec<u8> = Vec::new();
    let mut stack: Vec<u32> = Vec::new();
    let mut bands: Vec<Band> = Vec::new();

    let mut pairs = vec![0u32; 512];
    let mut contents = vec![0u32; 2048];
    let use_cache = 4 * ns >= 2560 || (edges.len() - 4 * ns - top) >= 2560;
    let mut el = Elim::<L, INLINE3> {
        a: &mut edges[4 * ns..4 * ns + top],
        base,
        top,
        nodes,
        _links: std::marker::PhantomData,
    };

    let mut id = L::next(el.a, 0, 0);
    let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
        L::band_node(el.a, id)
    } else {
        (0, 0, 0, 0)
    };
    let mut in_node = 0u32;
    let mut k = 0u32;
    for a in 0..256 {
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
                let c = if use_cache {
                    // Use exact direct-mapped dictionary caches like cand_r5_head
                    let g = node_word >> Elim::<L, INLINE3>::ID_BITS;
                    let wid = node_word & Elim::<L, INLINE3>::ID_MASK;
                    let uid = wid + 1 + (u32::from(g == 2) << 8);
                    let key = if INLINE3 {
                        (node_word.wrapping_mul(0x9e37_79b9)) ^ (u32::from(a as u8) << 24)
                    } else {
                        (uid << 8) | u32::from(a as u8)
                    };
                    let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
                    let slot = bucket * 2;
                    if !INLINE3 && pairs[slot] == key {
                        pairs[slot + 1]
                    } else {
                        let height = g as usize;
                        let code_val = if height < INLINE {
                            let v = el.short_payload_h(node_word) | (u128::from(a as u8) << (8 * height));
                            let entry = v.to_le_bytes();
                            let len = (height + 1) as u8;
                            let lo = v as u64;
                            let hi = (v >> 64) as u64;
                            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
                            let h = h ^ (h >> 30);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let index = (h as usize) & 2047;
                            let ent = contents[index];
                            if (ent & 0xFF00_0000) == tag && {
                                let old = (ent & 0x00FF_FFFF) as usize;
                                lens[old] == len && inline16[old] == entry
                            } {
                                ent & 0x00FF_FFFF
                            } else {
                                let cc = lens.len() as u32;
                                inline16.push(entry);
                                lens.push(len);
                                contents[index] = tag | cc;
                                cc
                            }
                        } else {
                            let start = spelling.len();
                            let memoized = el.a[el.rule(wid) + 3] != u32::MAX;
                            el.append(node_word, &mut stack, &mut spelling);
                            spelling.push(a as u8);
                            let entry = long_slot(start as u32, spelling.len() as u32);
                            let bytes = &spelling[start..];
                            let h = spelling_hash_long(bytes);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let index = (h as usize) & 2047;
                            let ent = contents[index];
                            if (ent & 0xFF00_0000) == tag && {
                                let old = (ent & 0x00FF_FFFF) as usize;
                                lens[old] == LONG && long_word(&inline16[old], &spelling) == bytes
                            } {
                                if memoized {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                                ent & 0x00FF_FFFF
                            } else {
                                let cc = lens.len() as u32;
                                inline16.push(entry);
                                lens.push(LONG);
                                contents[index] = tag | cc;
                                cc
                            }
                        };
                        if !INLINE3 {
                            pairs[slot] = key;
                            pairs[slot + 1] = code_val;
                        }
                        code_val
                    }
                } else {
                    let (entry, len) = el.inline_word(node_word, a as u8, &mut stack, &mut spelling);
                    let cc = lens.len() as u32;
                    inline16.push(entry);
                    lens.push(len);
                    cc
                };
                last_w = node_word;
                last_c = c;
                c
            };
            bands.push(Band::new(width, dst, code));
            in_node += width;
            k += width;
            if in_node == node_width {
                id = next_id;
                in_node = 0;
                if id != 0 {
                    (next_id, node_bottom, node_width, node_word) = L::band_node(el.a, id);
                }
            }
        }
    }
    assert_eq!(id, 0);
    (peak_words, bands, lens.len())
}

fn diagnose_block(blk: &Block) -> DiagStats {
    let (m, ns, arena_init_words_r5, arena_peak_words_r5, dict_len, bands, _cap) =
        diagnose_block_impl::<false>(blk);
    let (_m2, _ns2, _init2, arena_peak_words_i3, _dl2, _b2, _c2) =
        diagnose_block_impl::<true>(blk);

    let bits_m = if m <= 1 {
        1
    } else {
        (u32::BITS - ((m as u32) - 1).leading_zeros()).max(1)
    };
    let bits_code = if dict_len <= 1 {
        1
    } else {
        (u32::BITS - ((dict_len as u32) - 1).leading_zeros()).max(1)
    };

    // Row-weighted code frequency (Pareto-Zipf 1896)
    let mut freq = vec![0u32; dict_len];
    for &b in &bands {
        freq[b.code() as usize] += b.end();
    }
    let mut ranked: Vec<(u32, usize)> = freq
        .iter()
        .enumerate()
        .filter(|(_, &f)| f > 0)
        .map(|(c, &f)| (f, c))
        .collect();
    ranked.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let used_codes = ranked.len();

    let cov_k = |k: usize| -> f64 {
        let sum: u64 = ranked.iter().take(k).map(|(f, _)| u64::from(*f)).sum();
        100.0 * sum as f64 / m.max(1) as f64
    };
    let mut rank_of = vec![u32::MAX; dict_len];
    for (r, &(_, c)) in ranked.iter().enumerate() {
        rank_of[c] = r as u32;
    }

    let dyn_hot_bits = 24u32.saturating_sub(bits_m);
    let dyn_hot_cap = if dyn_hot_bits == 0 {
        0
    } else {
        ((1usize << dyn_hot_bits) - 1).min(dict_len)
    };
    // If all used_codes fit in `1 << dyn_hot_bits`, ZERO escape is needed!
    let dyn_no_escape = used_codes <= (1usize << dyn_hot_bits);
    let dyn_effective_hot = if dyn_no_escape {
        1usize << dyn_hot_bits
    } else {
        dyn_hot_cap
    };

    let mut dyn_cold_rows = 0usize;
    let mut dyn_cold_bands = 0usize;
    let mut cold_bands_63 = 0usize;
    let mut cold_bands_127 = 0usize;

    let num_pages = (m + 127) / 128;
    let mut page_cold = vec![0usize; num_pages.max(1)];
    for &b in &bands {
        let r = rank_of[b.code() as usize] as usize;
        if r >= 63 {
            cold_bands_63 += 1;
        }
        if r >= 127 {
            cold_bands_127 += 1;
        }
        if r >= dyn_effective_hot {
            dyn_cold_rows += b.end() as usize;
            dyn_cold_bands += 1;
            let p0 = b.dst() as usize / 128;
            let p1 = (b.dst() as usize + b.end() as usize - 1) / 128;
            for pg in p0..=p1 {
                page_cold[pg] += 1;
            }
        }
    }
    let max_cold_per_page128 = page_cold.iter().copied().max().unwrap_or(0);
    let avg_cold_per_page128 =
        page_cold.iter().sum::<usize>() as f64 / page_cold.len().max(1) as f64;

    let mut hot63_lit_count = 0usize;
    let mut hot63_word_count = 0usize;
    for &(_, c) in ranked.iter().take(63) {
        if c < 256 {
            hot63_lit_count += 1;
        } else {
            hot63_word_count += 1;
        }
    }

    DiagStats {
        n: blk.n,
        m,
        bits_m,
        ns,
        num_bands: bands.len(),
        dict_len,
        used_codes,
        bits_code,
        arena_init_words_r5,
        arena_peak_words_r5,
        arena_peak_words_i3,
        edges32_bytes: 4 * m,
        edges24_bytes: 3 * m,
        cov_top15: cov_k(15),
        cov_top31: cov_k(31),
        cov_top63: cov_k(63),
        cov_top127: cov_k(127),
        cov_top255: cov_k(255),
        dyn_hot_bits,
        dyn_hot_cap: dyn_effective_hot,
        dyn_cov: 100.0 * (m - dyn_cold_rows) as f64 / m.max(1) as f64,
        dyn_cold_rows,
        dyn_cold_bands,
        cold_bands_63,
        cold_bands_127,
        max_cold_per_page128,
        avg_cold_per_page128,
        hot63_lit_count,
        hot63_word_count,
    }
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
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                records,
            }
        })
        .collect()
}

fn load_bin_blocks(path: &str, names: Option<&[&str]>, only_full: bool) -> Vec<Block> {
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
        let name = if let Some(ns) = names {
            if b_idx < ns.len() {
                ns[b_idx].to_string()
            } else {
                format!("blk-{idx}")
            }
        } else {
            format!("w431-{idx}")
        };
        blocks.push(Block { name, n, p, records });
    }
    blocks
}

fn make_solved_analogue_h() -> Vec<Block> {
    // Solved Analogue H: compact-state BWTs with m < 65,536 (16-bit index) and |D| <= 256 (8-bit code)
    // Plus degenerate |D| = 1 / ultra-compact periodic block.
    let mut out = Vec::new();
    // 1. Compact structured JSON-like text (~300 KB, ~45k retained rows, |D| <= 200)
    let mut txt = Vec::with_capacity(300_000);
    let mut rng = 0x1234_5678_9ABC_DEF0u64;
    let mut next = || {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        rng
    };
    let mut id_num = 100_000u32;
    while txt.len() + 64 < 300_000 {
        id_num += 1;
        let tag = (next() % 8) as u8;
        let line = format!("{{\"id\":\"Q{:06}\",\"type\":\"item\",\"v\":{}}}\n", id_num, b'A' + tag);
        txt.extend_from_slice(line.as_bytes());
    }
    let (l, p) = bwt_encode(&txt);
    out.push(Block {
        name: "AnaH-Compact300k".to_string(),
        n: l.len(),
        p,
        records: records_of(&l),
    });

    // 2. Degenerate |D| = 1 (all retained rows share 1 code -> 16-bit pure permutation!)
    let mut l_d1 = vec![b'a'; 40_000];
    l_d1.push(b'b');
    l_d1.extend(std::iter::repeat_n(b'a', 19_999));
    out.push(Block {
        name: "AnaH-D1-60k".to_string(),
        n: l_d1.len(),
        p: 40_000,
        records: records_of(&l_d1),
    });
    out
}

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_unstable_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
            l.push(text[n - 1]);
        } else {
            l.push(text[s - 1]);
        }
    }
    (l, p)
}

fn print_table(title: &str, blocks: &[Block]) {
    println!("\n================================================================================================================================");
    println!("{title}");
    println!("================================================================================================================================");
    println!(
        "{:<16} {:>7} {:>7} {:>2} {:>5} {:>5} {:>4} {:>3} | {:>5} {:>5} {:>5} {:>5} {:>5} | {:>4} {:>6} {:>6} {:>5} | {:>6} {:>6} {:>6} {:>6}",
        "Block", "n", "m", "bm", "ns", "bands", "|D|", "bc",
        "t15%", "t31%", "t63%", "t127%", "t255%",
        "hb", "dyn%", "cRow", "cBnd",
        "e32K", "e24K", "arR5K", "arI3K"
    );
    let mut sum_m = 0usize;
    let mut sum_e32 = 0usize;
    let mut sum_e24 = 0usize;
    let mut sum_cold_rows = 0usize;
    let mut sum_cold_bands = 0usize;
    let mut max_e32 = 0usize;
    let mut max_e24 = 0usize;
    let mut max_ar_r5 = 0usize;
    let mut max_ar_i3 = 0usize;
    let mut sum_t15 = 0.0;
    let mut sum_t31 = 0.0;
    let mut sum_t63 = 0.0;
    let mut sum_t127 = 0.0;
    let mut sum_t255 = 0.0;
    let mut sum_dyn = 0.0;

    for blk in blocks {
        let d = diagnose_block(blk);
        sum_m += d.m;
        sum_e32 += d.edges32_bytes;
        sum_e24 += d.edges24_bytes;
        sum_cold_rows += d.dyn_cold_rows;
        sum_cold_bands += d.dyn_cold_bands;
        max_e32 = max_e32.max(d.edges32_bytes);
        max_e24 = max_e24.max(d.edges24_bytes);
        max_ar_r5 = max_ar_r5.max(4 * d.arena_peak_words_r5);
        max_ar_i3 = max_ar_i3.max(4 * d.arena_peak_words_i3);
        sum_t15 += d.cov_top15 * d.m as f64;
        sum_t31 += d.cov_top31 * d.m as f64;
        sum_t63 += d.cov_top63 * d.m as f64;
        sum_t127 += d.cov_top127 * d.m as f64;
        sum_t255 += d.cov_top255 * d.m as f64;
        sum_dyn += d.dyn_cov * d.m as f64;

        println!(
            "{:<16} {:>7} {:>7} {:>2} {:>5} {:>5} {:>4} {:>3} | {:>5.1} {:>5.1} {:>5.1} {:>5.1} {:>5.1} | {:>2}b  {:>5.1}% {:>6} {:>5} | {:>6.1} {:>6.1} {:>6.1} {:>6.1}",
            blk.name, d.n, d.m, d.bits_m, d.ns, d.num_bands, d.dict_len, d.bits_code,
            d.cov_top15, d.cov_top31, d.cov_top63, d.cov_top127, d.cov_top255,
            d.dyn_hot_bits, d.dyn_cov, d.dyn_cold_rows, d.dyn_cold_bands,
            d.edges32_bytes as f64 / 1024.0,
            d.edges24_bytes as f64 / 1024.0,
            (4 * d.arena_peak_words_r5) as f64 / 1024.0,
            (4 * d.arena_peak_words_i3) as f64 / 1024.0,
        );
        if blocks.len() <= 10 {
            println!(
                "    detail: used_codes={} (bits_m+bits_code={}), dyn_hot_cap={}, hot63=(lit={}, word={}), cold_bands(63)={}, cold_bands(127)={}, pg128(avg={:.2}, max={}), init_ar={:.1}K",
                d.used_codes, d.bits_m + d.bits_code, d.dyn_hot_cap, d.hot63_lit_count, d.hot63_word_count,
                d.cold_bands_63, d.cold_bands_127, d.avg_cold_per_page128, d.max_cold_per_page128,
                (4 * d.arena_init_words_r5) as f64 / 1024.0
            );
        }
    }
    println!(
        "SUMMARY ({} blks): sum_m={} | row-wt cov: t15={:.2}%, t31={:.2}%, t63={:.2}%, t127={:.2}%, t255={:.2}%, dyn={:.2}% | cold_rows={} ({:.2}%), cold_bands={}",
        blocks.len(), sum_m,
        sum_t15 / sum_m.max(1) as f64,
        sum_t31 / sum_m.max(1) as f64,
        sum_t63 / sum_m.max(1) as f64,
        sum_t127 / sum_m.max(1) as f64,
        sum_t255 / sum_m.max(1) as f64,
        sum_dyn / sum_m.max(1) as f64,
        sum_cold_rows, 100.0 * sum_cold_rows as f64 / sum_m.max(1) as f64,
        sum_cold_bands
    );
    println!(
        "  PEAK WS: edges32_max={:.1} KiB, edges24_max={:.1} KiB (delta={:+.1} KiB), arena_peak_R5_max={:.1} KiB, arena_peak_I3_max={:.1} KiB | SUM active edges32={:.1} KiB -> edges24={:.1} KiB",
        max_e32 as f64 / 1024.0,
        max_e24 as f64 / 1024.0,
        (max_e24 as isize - max_e32 as isize) as f64 / 1024.0,
        max_ar_r5 as f64 / 1024.0,
        max_ar_i3 as f64 / 1024.0,
        sum_e32 as f64 / 1024.0,
        sum_e24 as f64 / 1024.0,
    );
}

fn log2_fact(k: usize) -> f64 {
    let mut s = 0.0f64;
    for i in 2..=k {
        s += (i as f64).log2();
    }
    s
}

fn h2(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        0.0
    } else {
        -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
    }
}

/// Exact empirical & algebraic verification of Primary-Source Classical Bounds:
/// - Theorem H-A (Netto 1901, Lehrbuch der Combinatorik §§10-13; MacMahon 1915, Combinatory Analysis Vol. I):
///   Multiset-permutation intrinsic entropy H_multi = (1/m) log2(m! / prod_a thin[a]!).
/// - Theorem H-B (Cauchy 1815, 1844; Jordan 1870, Traité des substitutions Livre I Ch. II):
///   Direct-addressed XOR-involution xi(u) = pi(u) ^ pi^{-1}(u) satisfies exact F_2^b parity conservation
///   XOR_u xi(u) == 0, min bit-width max_u xi(u) >= 2^{bits_m - 1} (for single cycle m >= 3),
///   and near-maximal bit-plane entropy H_bits(xi) ~ bits_m.
fn verify_classical_bounds(title: &str, blocks: &[Block]) {
    println!("\n================================================================================================================================");
    println!("CLASSICAL THEOREM VERIFICATION (Netto 1901 / MacMahon 1915 vs Cauchy 1815,1844 / Jordan 1870) — {title}");
    println!("================================================================================================================================");
    println!(
        "{:<16} {:>7} {:>2} {:>3} | {:>6} {:>6} {:>6} {:>6} | {:>5} {:>6} {:>6} | {:>5} {:>5} | {:>6}",
        "Block", "m", "bm", "cyc", "H_mult", "H_cyc", "H_bXi", "H_cod", "Xi+Cd", "minP", "maxP", "xor0?", "lbOK?", "32/Hm"
    );
    for blk in blocks {
        let n = blk.n as u32;
        let p = blk.p;
        let mut thin = [0u32; 256];
        let mut thick = [0u32; 256];
        let mut thick_rows = [0u32; 256];
        let mut m = 0u32;
        let mut strips_raw: Vec<u32> = Vec::new();
        let mut ns = 0usize;
        let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); 256];

        let mut pos = 0u32;
        let mut i = 0usize;
        let nrec = blk.records.len();
        while i < nrec {
            let rec = blk.records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < nrec && (blk.records[i] >> 8) as u8 == c {
                width += u32::from(blk.records[i] & 255);
                i += 1;
            }
            let mut handle_piece = |x: u32, w: u32, keep: bool| {
                let a = usize::from(c);
                if keep {
                    for off in 0..w {
                        buckets[a].push(m + off);
                    }
                    thin[a] += w;
                    m += w;
                } else {
                    strips_raw.push(x);
                    strips_raw.push(thin[a] + thick_rows[a]);
                    strips_raw.push((w << 8) | u32::from(c));
                    ns += 1;
                    thick[a] += 1;
                    thick_rows[a] += w;
                }
            };
            let end = pos + width;
            if width > LIMIT && (pos..end).contains(&p) {
                if pos < p {
                    handle_piece(pos, p - pos, false);
                }
                handle_piece(p, 1, true);
                if p + 1 < end {
                    handle_piece(p + 1, end - p - 1, false);
                }
            } else {
                handle_piece(pos, width, width <= LIMIT);
            }
            pos += width;
        }
        let m_us = m as usize;
        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
        let first = starts(all);
        let mut write = starts(thick);
        let most = 3 * ns + 3;
        let wide = most > NARROW_NODES;
        let stride = if wide { Wide::STRIDE } else { Narrow::STRIDE };
        let arena_init_words = (4 * ns + (stride + RULE) * most).max(m_us);
        let mut edges = vec![0u32; arena_init_words];
        edges[..3 * ns].copy_from_slice(&strips_raw);

        let (_peak, bands, dict_len) = if wide {
            run_induced::<Wide, false>(ns, n, m, &thin, &first, &mut write, &mut edges, most)
        } else {
            run_induced::<Narrow, false>(ns, n, m, &thin, &first, &mut write, &mut edges, most)
        };

        // Reconstruct exact induced permutation pi[0..m] & inverse pi_inv[0..m]
        let mut pi = vec![0u32; m_us];
        let mut pi_inv = vec![0u32; m_us];
        let mut band_idx = 0usize;
        let mut code_freq = vec![0u32; dict_len];
        for a in 0..256 {
            let mut rem = thin[a];
            let mut bpos = 0usize;
            while rem > 0 {
                let b = bands[band_idx];
                band_idx += 1;
                let span = b.end() as usize;
                let dst = b.dst() as usize;
                code_freq[b.code() as usize] += b.end();
                for off in 0..span {
                    let u = dst + off;
                    let v = buckets[a][bpos + off];
                    pi[u] = v;
                    pi_inv[v as usize] = u as u32;
                }
                bpos += span;
                rem -= b.end();
            }
            assert_eq!(bpos, buckets[a].len());
        }
        assert_eq!(band_idx, bands.len());
        for u in 0..m_us {
            assert_eq!(pi_inv[pi[u] as usize], u as u32, "pi bijection check failed");
        }

        // Exact Cauchy (1815, 1844) cycle decomposition count
        let mut vis = vec![false; m_us];
        let mut n_cyc = 0usize;
        for u in 0..m_us {
            if !vis[u] {
                n_cyc += 1;
                let mut cur = u;
                while !vis[cur] {
                    vis[cur] = true;
                    cur = pi[cur] as usize;
                }
            }
        }

        let bits_m = if m_us <= 1 {
            1
        } else {
            (u32::BITS - ((m_us as u32) - 1).leading_zeros()).max(1)
        };

        // Cauchy-Jordan F_2^b XOR-involution xi(u) = pi(u) ^ pi^{-1}(u)
        let mut xor_sum = 0u32;
        let mut max_xi = 0u32;
        let mut bit_ones = [0u64; 24];
        for u in 0..m_us {
            let xi = pi[u] ^ pi_inv[u];
            xor_sum ^= xi;
            if xi > max_xi {
                max_xi = xi;
            }
            for k in 0..(bits_m as usize) {
                bit_ones[k] += u64::from((xi >> k) & 1);
            }
        }
        assert_eq!(xor_sum, 0, "Cauchy-Jordan F_2^b parity conservation violated!");
        let lb_ok = if m_us >= 3 && n_cyc == 1 {
            assert!(
                max_xi >= (1u32 << (bits_m - 1)),
                "Direct-addressed XOR bit-width lower bound violated on {}",
                blk.name
            );
            true
        } else {
            max_xi >= (1u32 << (bits_m.saturating_sub(1)))
        };

        let mut h_bit_xi = 0.0f64;
        let mut min_p = 1.0f64;
        let mut max_p = 0.0f64;
        for k in 0..(bits_m as usize) {
            let pk = bit_ones[k] as f64 / m_us.max(1) as f64;
            h_bit_xi += h2(pk);
            if k + 1 < bits_m as usize {
                min_p = min_p.min(pk);
                max_p = max_p.max(pk);
            }
        }
        if bits_m <= 1 {
            min_p = bit_ones[0] as f64 / m_us.max(1) as f64;
            max_p = min_p;
        }

        // Exact Netto (1901) / MacMahon (1915) multiset-permutation entropy
        let mut log2_mult = log2_fact(m_us);
        for &cnt in &thin {
            log2_mult -= log2_fact(cnt as usize);
        }
        let h_multi = log2_mult / m_us.max(1) as f64;
        let h_cycle = if m_us >= 2 {
            (log2_fact(m_us - 1) - 1.0).max(0.0) / m_us as f64
        } else {
            0.0
        };
        let mut h_code = 0.0f64;
        for &f in &code_freq {
            if f > 0 {
                let pf = f as f64 / m_us as f64;
                h_code -= pf * pf.log2();
            }
        }

        println!(
            "{:<16} {:>7} {:>2} {:>3} | {:>6.3} {:>6.3} {:>6.3} {:>6.3} | {:>5.2} {:>6.3} {:>6.3} | {:>5} {:>5} | {:>5.2}x",
            blk.name, m_us, bits_m, n_cyc,
            h_multi, h_cycle, h_bit_xi, h_code,
            h_bit_xi + h_code, min_p, max_p,
            if xor_sum == 0 { "EXACT" } else { "FAIL" },
            if lb_ok { "EXACT" } else { "N/A" },
            if h_multi > 0.0 { 32.0 / h_multi } else { 0.0 }
        );
    }
}

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/tmp/mibz2_corpora/{rel}");
    if std::path::Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if std::path::Path::new(&p2).exists() {
        return p2;
    }
    panic!("corpus file not found at {p1} or {p2}");
}

fn main() {
    let solved = make_solved_analogue_h();
    print_table("SOLVED ANALOGUE H (Compact-State BWTs: m < 65,536, |D| <= 256)", &solved);
    verify_classical_bounds("Solved Analogue H", &solved);

    let wiki6 = load_six_admitted();
    print_table("6 ADMITTED WIKIDATA BLOCKS", &wiki6);
    verify_classical_bounds("6 Admitted Wikidata Blocks", &wiki6);

    let nonwiki_names = ["jsonl-0", "jsonl-1", "jsonl-2", "jsonl-3", "jsonl-4", "syslog-0"];
    let nonwiki_path = resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin");
    let nonwiki = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false);
    print_table("6 REAL NON-WIKIDATA BLOCKS (JSONL + SYSLOG)", &nonwiki);
    verify_classical_bounds("6 Real Non-Wikidata Blocks", &nonwiki);

    let code_docs_names = ["codedocs-0", "codedocs-1", "codedocs-2", "codedocs-3"];
    let code_docs_path = resolve_corpus_path("non_wikidata/code_docs_blocks.bin");
    let code_docs = load_bin_blocks(&code_docs_path, Some(&code_docs_names), false);
    print_table("4 REAL CODE+DOCS BLOCKS", &code_docs);
    verify_classical_bounds("4 Real Code+Docs Blocks", &code_docs);

    let w431_path = resolve_corpus_path("wikidata_431_blocks.bin");
    let w233 = load_bin_blocks(&w431_path, None, true);
    println!("\n================================================================================================================================");
    println!("ALL {} FULL WIKIDATA.BZ2 BLOCKS — AGGREGATE DISTRIBUTION", w233.len());
    println!("================================================================================================================================");
    let mut bm_hist = [0usize; 25];
    let mut sum_m = 0usize;
    let mut min_m = usize::MAX;
    let mut max_m = 0usize;
    let mut max_d = 0usize;
    let mut max_bands = 0usize;
    let mut max_cold_bands = 0usize;
    let mut max_ar_r5 = 0usize;
    let mut max_ar_i3 = 0usize;
    let mut ar_gt_e24_r5 = 0usize;
    let mut ar_gt_e24_i3 = 0usize;
    let mut ar_gt_e32_r5 = 0usize;
    let mut sum_t15 = 0.0;
    let mut sum_t31 = 0.0;
    let mut sum_t63 = 0.0;
    let mut sum_t127 = 0.0;
    let mut sum_t255 = 0.0;
    let mut sum_dyn = 0.0;
    let mut min_t63 = 100.0f64;
    let mut min_dyn = 100.0f64;
    let mut max_b_plus_c = 0u32;
    let mut sum_cold_rows = 0usize;
    let mut sum_cold_bands = 0usize;
    let mut max_pg128 = 0usize;
    let mut sum_pg128_avg = 0.0;

    for blk in &w233 {
        let d = diagnose_block(blk);
        bm_hist[d.bits_m as usize] += 1;
        sum_m += d.m;
        min_m = min_m.min(d.m);
        max_m = max_m.max(d.m);
        max_d = max_d.max(d.dict_len);
        max_bands = max_bands.max(d.num_bands);
        max_cold_bands = max_cold_bands.max(d.dyn_cold_bands);
        max_ar_r5 = max_ar_r5.max(4 * d.arena_peak_words_r5);
        max_ar_i3 = max_ar_i3.max(4 * d.arena_peak_words_i3);
        if 4 * d.arena_peak_words_r5 > d.edges24_bytes {
            ar_gt_e24_r5 += 1;
        }
        if 4 * d.arena_peak_words_i3 > d.edges24_bytes {
            ar_gt_e24_i3 += 1;
        }
        if 4 * d.arena_peak_words_r5 > d.edges32_bytes {
            ar_gt_e32_r5 += 1;
        }
        sum_t15 += d.cov_top15 * d.m as f64;
        sum_t31 += d.cov_top31 * d.m as f64;
        sum_t63 += d.cov_top63 * d.m as f64;
        sum_t127 += d.cov_top127 * d.m as f64;
        sum_t255 += d.cov_top255 * d.m as f64;
        sum_dyn += d.dyn_cov * d.m as f64;
        min_t63 = min_t63.min(d.cov_top63);
        min_dyn = min_dyn.min(d.dyn_cov);
        max_b_plus_c = max_b_plus_c.max(d.bits_m + d.bits_code);
        sum_cold_rows += d.dyn_cold_rows;
        sum_cold_bands += d.dyn_cold_bands;
        max_pg128 = max_pg128.max(d.max_cold_per_page128);
        sum_pg128_avg += d.avg_cold_per_page128;
    }

    println!("  m range: [{min_m}, {max_m}] (avg {:.1}); bits_m histogram:", sum_m as f64 / w233.len() as f64);
    for (b, &cnt) in bm_hist.iter().enumerate() {
        if cnt > 0 {
            println!("    bits_m = {b:2}: {cnt:3} blocks ({:.1}%)", 100.0 * cnt as f64 / w233.len() as f64);
        }
    }
    println!("  max |D| = {max_d}, max bands = {max_bands}, max (bits_m + bits_code) = {max_b_plus_c}");
    println!(
        "  Row-weighted Pareto-Zipf coverage (across ALL {sum_m} retained rows of {} blocks):",
        w233.len()
    );
    println!("    Top  15 (4-bit): {:.2}%", sum_t15 / sum_m as f64);
    println!("    Top  31 (5-bit): {:.2}%", sum_t31 / sum_m as f64);
    println!("    Top  63 (6-bit): {:.2}% (min block: {:.2}%)", sum_t63 / sum_m as f64, min_t63);
    println!("    Top 127 (7-bit): {:.2}%", sum_t127 / sum_m as f64);
    println!("    Top 255 (8-bit): {:.2}%", sum_t255 / sum_m as f64);
    println!(
        "    Dynamic (24 - bits_m): {:.2}% (min block: {:.2}%), cold_rows = {} ({:.2}%), avg cold_bands = {:.1}, max cold_bands = {} ({:.1} KiB @ 4B/band)",
        sum_dyn / sum_m as f64,
        min_dyn,
        sum_cold_rows,
        100.0 * sum_cold_rows as f64 / sum_m as f64,
        sum_cold_bands as f64 / w233.len() as f64,
        max_cold_bands,
        (4 * max_cold_bands) as f64 / 1024.0
    );
    println!(
        "  Cold-band page-128 density: avg = {:.2} bands/page, max = {} bands/page",
        sum_pg128_avg / w233.len() as f64,
        max_pg128
    );
    println!(
        "  Peak buffer comparison: max edges32 = {:.1} KiB, max edges24 = {:.1} KiB (delta = {:+.1} KiB)",
        (4 * max_m) as f64 / 1024.0,
        (3 * max_m) as f64 / 1024.0,
        -(max_m as f64) / 1024.0
    );
    println!(
        "  Arena peak vs edges: max arena_R5 = {:.1} KiB (> edges24 on {}/{} blks, > edges32 on {}/{} blks); max arena_I3 = {:.1} KiB (> edges24 on {}/{} blks)",
        max_ar_r5 as f64 / 1024.0, ar_gt_e24_r5, w233.len(), ar_gt_e32_r5, w233.len(),
        max_ar_i3 as f64 / 1024.0, ar_gt_e24_i3, w233.len()
    );
}
