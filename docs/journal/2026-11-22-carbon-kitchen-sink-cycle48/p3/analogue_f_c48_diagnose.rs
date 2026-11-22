#![forbid(unsafe_code)]

//! Cycle 48 P3 — Exact Post-`F-Inline3` Rule-Length, 7-bit ASCII Alphabet, & `S3a` Working-Set Diagnosis
//! Measures exact rule-length histograms, 7-bit ASCII quadgram shares, arena byte savings,
//! `short_payload_h` read frequencies, `Band` field distributions, and `S3a` persistent scratch
//! utilization across 6 Admitted Wikidata, 6 Non-Wikidata, and all 233 full `wikidata.bz2` blocks.

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/outq.rs"]
mod outq;

use std::fs;

const LIMIT: u32 = 16;
const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const NARROW_NODES: usize = 1 << 16;
const RULE: usize = 4;
const CERTIFIED_WIDTH: u32 = 32_768;
const SPIRAL_RULES: usize = 2 * 20 + 1;
const ID_BITS: u32 = 24;
const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
const SATURATED: u32 = u32::MAX >> ID_BITS;

#[inline(always)]
const fn literal(byte: u8) -> u32 {
    (1u32 << ID_BITS) | (byte as u32)
}

#[derive(Default, Clone, Debug)]
struct DiagStats {
    m: usize,
    ns: usize,
    open_nodes: usize,
    // Join histograms (every non-trivial join(left!=0, right!=0) in S2b)
    join_total: usize,
    join_g2: usize,
    join_g3: usize,
    join_g3_ascii: usize,
    join_g4_total: usize,
    join_g4_ascii: usize,
    join_g4_nonascii: usize,
    join_g5: usize,
    join_g5_ascii: usize,
    join_g6: usize,
    join_g7: usize,
    join_g8: usize,
    join_g9_15: usize,
    join_g16_238: usize,
    join_g239_254: usize,
    join_g255p: usize,
    // short_payload_h read frequencies inside S2b (for sum < 16 joins)
    sp_read_g1_3: usize,
    sp_read_g4_ascii: usize,
    sp_read_g4_nonascii: usize,
    sp_read_g5_7: usize,
    sp_read_g8: usize,
    sp_read_g9_15: usize,
    // Arena peak words
    s2b_rules_base: usize,
    s2b_rules_i3: usize,
    s2b_rules_i4: usize,
    s2b_words_i3: usize,
    s2b_words_i4: usize,
    s2b_words_i3_p7: usize,
    s2b_words_i3_p8: usize,
    s2b_words_i4_p7: usize,
    s2b_words_i4_p8: usize,
    arena_peak_words_i3: usize,
    // S3a & Band anatomy
    bands_len: usize,
    bands_literal: usize,
    bands_word: usize,
    bands_w1: usize,
    bands_w_le_2048: usize,
    bands_w_gt_2048: usize,
    bands_lit_w_le_2048: usize,
    bands_word_w1_c4096: usize,
    bands_word_w1_c8192: usize,
    bands_word_w_le_2047: usize,
    bands_max_w: u32,
    bands_max_dst: u32,
    bands_max_code: u32,
    band32_words_lit_only: usize, // 1 u32 for literal(w<=2048), 2 u32 for word(w<=2047), 3 u32 escape
    band32_words_plus_w1: usize,  // also 1 u32 for word(w==1, code<2048, dst<2^18) when m<=262144
    spelling_bytes: usize,
    inline16_entries: usize, // total lens.len() including 256 literals
    long_entries: usize,
    stack_peak: usize,
    outq_chunks: usize,
    outq_ovf: usize,
    out_tail_free_bytes: usize,
    // S3a cache stats
    s3a_dict_calls: usize,
    s3a_pairs_hits: usize,
    s3a_miss_g1_3: usize,
    s3a_miss_g4_ascii: usize,
    s3a_miss_g4_nonascii: usize,
    s3a_miss_g5_15: usize,
    s3a_miss_long: usize,
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
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] & 0xFFFF
    }
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 4 + side] >> 16
    }
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF_0000) | to;
    }
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }
}

impl Links for Wide {
    const STRIDE: usize = 6;
    const BOTTOM: usize = 2;
    fn prev(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side]
    }
    fn next(a: &[u32], id: u32, side: usize) -> u32 {
        a[id as usize * 6 + 2 * side + 1]
    }
    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side] = to;
    }
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }
}

struct Elim<'a, L: Links> {
    a: &'a mut [u32],
    base: usize,
    top: usize,
    nodes: u32,
    _links: std::marker::PhantomData<L>,
}

impl<L: Links> Elim<'_, L> {
    fn width(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2]
    }
    fn set_width(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
    }
    fn word(&self, id: u32) -> u32 {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1]
    }
    fn set_word(&mut self, id: u32, w: u32) {
        self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
    }
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
    fn cut(&mut self, id: u32, side: usize) {
        let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
        L::set_next(self.a, p, side, n);
        L::set_prev(self.a, n, side, p);
    }
    fn pop_tail(&mut self, id: u32, side: usize) {
        let prev = L::prev(self.a, id, side);
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }
    fn len_handle(&self, h: u32) -> u32 {
        let g = h >> ID_BITS;
        if g < SATURATED {
            g
        } else {
            self.a[self.rule(h & ID_MASK) + 2]
        }
    }
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
    fn record_sp_read(&self, h: u32, ds: &mut DiagStats) {
        let g = h >> ID_BITS;
        if g <= 3 {
            ds.sp_read_g1_3 += 1;
        } else if g == 4 {
            let v = self.short_payload_h(h) as u32;
            if (v & 0x8080_8080) == 0 {
                ds.sp_read_g4_ascii += 1;
            } else {
                ds.sp_read_g4_nonascii += 1;
            }
        } else if g <= 7 {
            ds.sp_read_g5_7 += 1;
        } else if g == 8 {
            ds.sp_read_g8 += 1;
        } else {
            ds.sp_read_g9_15 += 1;
        }
    }
    fn join(&mut self, left: u32, right: u32, ds: &mut DiagStats) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        ds.join_total += 1;
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum <= 3 {
                let lp = left & ID_MASK;
                let rp = right & ID_MASK;
                let v = lp | (rp << (8 * left_len));
                if sum == 2 {
                    ds.join_g2 += 1;
                } else {
                    ds.join_g3 += 1;
                    if (v & 0x0080_8080) == 0 {
                        ds.join_g3_ascii += 1;
                    }
                }
                return (sum << ID_BITS) | v;
            }
            self.record_sp_read(left, ds);
            self.record_sp_read(right, ds);
            let payload = self.short_payload_h(left) | (self.short_payload_h(right) << (8 * left_len));
            match sum {
                4 => {
                    ds.join_g4_total += 1;
                    if ((payload as u32) & 0x8080_8080) == 0 {
                        ds.join_g4_ascii += 1;
                    } else {
                        ds.join_g4_nonascii += 1;
                    }
                }
                5 => {
                    ds.join_g5 += 1;
                    if ((payload as u64) & 0x0000_0080_8080_8080) == 0 {
                        ds.join_g5_ascii += 1;
                    }
                }
                6 => ds.join_g6 += 1,
                7 => ds.join_g7 += 1,
                8 => ds.join_g8 += 1,
                _ => ds.join_g9_15 += 1,
            }
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
            if len < 239 {
                ds.join_g16_238 += 1;
            } else if len < 255 {
                ds.join_g239_254 += 1;
            } else {
                ds.join_g255p += 1;
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
    fn power(&mut self, word: u32, q: u32, ds: &mut DiagStats) -> u32 {
        let (mut acc, mut base, mut e) = (0, word, q);
        loop {
            if e & 1 == 1 {
                acc = self.join(acc, base, ds);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = self.join(base, base, ds);
        }
    }
    fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, ds: &mut DiagStats) -> bool {
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
                    let head = self.power(self.word(x), q, ds);
                    let word = self.join(head, self.word(y), ds);
                    self.set_word(y, word);
                    self.set_width(x, aw - q * bw);
                    left -= q * bw;
                    continue;
                }
                if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = self.power(self.word(y), q, ds);
                    let word = self.join(self.word(x), tail, ds);
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
            let word = self.join(self.word(x), self.word(y), ds);
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
    fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>, ds: &mut DiagStats) {
        stack.clear();
        stack.push(handle);
        while let Some(h) = stack.pop() {
            ds.stack_peak = ds.stack_peak.max(stack.len() + 1);
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

fn diagnose_block(records: &[u16], n: usize, p: u32) -> DiagStats {
    let mut ds = DiagStats::default();
    let mut out = vec![0u8; n + INLINE];
    let mut spill = outq::Spill::default();
    let mut edges: Vec<u32> = Vec::new();
    let mut ns = 0usize;
    let mut f = First::default();
    let outq_chunks;
    {
        let mut queues = outq::Queues::new(&mut out[..], &mut spill, u32::MAX);
        let mut piece = |c: u8, x: u32, w: u32, keep: bool| {
            let a = usize::from(c);
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                    f.mark_byte = c;
                }
                queues.push_thin(a, f.m, w);
                f.m += w;
            } else {
                let at = 3 * ns;
                if edges.len() < at + 3 {
                    let need = (at + 3).max(2 * edges.len()).max(1024);
                    edges.reserve_exact(need - edges.len());
                    edges.resize(need, 0);
                }
                let thin_a = queues.thin_of(a);
                edges[at..at + 3].copy_from_slice(&[
                    x,
                    thin_a + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
                ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        };
        let mut pos = 0u32;
        let mut i = 0usize;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            if width <= LIMIT {
                piece(c, pos, width, true);
                pos += width;
            } else {
                if width == 127 {
                    while i < records.len() && (records[i] >> 8) as u8 == c {
                        width += u32::from(records[i] & 255);
                        i += 1;
                    }
                }
                if p.wrapping_sub(pos) >= width {
                    piece(c, pos, width, false);
                } else {
                    let end = pos + width;
                    if pos < p {
                        piece(c, pos, p - pos, false);
                    }
                    piece(c, p, 1, true);
                    if p + 1 < end {
                        piece(c, p + 1, end - p - 1, false);
                    }
                }
                pos += width;
            }
        }
        f.thin = queues.extract_thin();
        // Estimate chunks used from thin counts: each non-empty byte takes ceil(thin[a] / 512) chunks
        let mut ch = 0usize;
        for a in 0..256 {
            if f.thin[a] > 0 {
                ch += (f.thin[a] as usize + 511) / 512;
            }
        }
        outq_chunks = ch;
    }
    ds.m = f.m as usize;
    ds.ns = ns;
    ds.outq_chunks = outq_chunks;
    let in_out_chunks = (n + INLINE) / 1024;
    ds.outq_ovf = outq_chunks.saturating_sub(in_out_chunks);
    ds.out_tail_free_bytes = (n + INLINE).saturating_sub(outq_chunks * 1024);

    let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
    let first = starts(all);
    let mut write = starts(f.thick);
    let most = 3 * ns + 3;
    let is_wide = most > NARROW_NODES;
    let stride = if is_wide { Wide::STRIDE } else { Narrow::STRIDE };
    let need = (4 * ns + (stride + RULE) * most).max(f.m as usize);
    if edges.len() < need {
        edges.resize(need, 0);
    }

    fn run_diag<L: Links>(
        edges: &mut Vec<u32>,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        ds: &mut DiagStats,
    ) {
        let mut widest = 0u32;
        let (base, mut top, nodes) = {
            let (hdr, arena) = edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L> {
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
            ds.open_nodes = e.nodes as usize - 1;
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            let base = L::STRIDE * e.nodes as usize;
            (base, base, e.nodes)
        };
        let mut remaining = n;
        let spirals = widest > CERTIFIED_WIDTH;
        loop {
            let mut e = Elim::<L> {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            let done = if spirals {
                e.eliminate::<true>(&mut remaining, ds)
            } else {
                e.eliminate::<false>(&mut remaining, ds)
            };
            top = e.top;
            if done {
                break;
            }
            grow(edges, 4 * ns + top + SPIRAL_RULES * RULE);
        }
        ds.s2b_rules_i3 = (top - base) / RULE;
        ds.s2b_rules_base = ds.s2b_rules_i3 + ds.join_g3;
        ds.s2b_rules_i4 = ds.s2b_rules_i3 - ds.join_g4_ascii;
        let g5_7 = ds.join_g5 + ds.join_g6 + ds.join_g7;
        let g4_7 = ds.join_g4_total + g5_7;
        let g4_8 = g4_7 + ds.join_g8;
        let g4na_7 = ds.join_g4_nonascii + g5_7;
        let g4na_8 = g4na_7 + ds.join_g8;
        ds.s2b_words_i3 = ds.s2b_rules_i3 * 4;
        ds.s2b_words_i4 = ds.s2b_rules_i4 * 4;
        ds.s2b_words_i3_p7 = g4_7 * 2 + (ds.s2b_rules_i3 - g4_7) * 4;
        ds.s2b_words_i3_p8 = g4_8 * 2 + (ds.s2b_rules_i3 - g4_8) * 4;
        ds.s2b_words_i4_p7 = g4na_7 * 2 + (ds.s2b_rules_i4 - g4na_7) * 4;
        ds.s2b_words_i4_p8 = g4na_8 * 2 + (ds.s2b_rules_i4 - g4na_8) * 4;
        ds.arena_peak_words_i3 = 4 * ns + top;

        {
            let mut e = Elim::<L> {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(ds.m as u32);
        }

        // Exact S3a simulation to measure bands, spelling, inline16, lens, cache hits
        let mut inline16 = Vec::new();
        let mut lens = Vec::new();
        let mut spelling = Vec::new();
        let mut stack = Vec::new();
        begin_inline(0, &mut inline16, &mut lens);
        let mut pairs = [0u32; 512];
        let mut contents = [0u32; 2048];
        let mut e = Elim::<L> {
            a: &mut edges[4 * ns..4 * ns + top],
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        let mut id = L::next(e.a, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        let m_fits_18 = ds.m <= (1 << 18);
        for a in 0..256 {
            let end = k + thin[a];
            let mut last_w = 0u32;
            let mut last_c = 0u32;
            while k < end {
                let width = (end - k).min(node_width - in_node);
                let dst = node_bottom + in_node;
                let code = if node_word == 0 {
                    ds.bands_literal += 1;
                    a as u32
                } else {
                    ds.bands_word += 1;
                    if node_word == last_w {
                        last_c
                    } else {
                        ds.s3a_dict_calls += 1;
                        let bucket = ((node_word.wrapping_mul(0x9e37_79b9)
                            ^ (u32::from(a as u8).wrapping_mul(0x85eb_ca6b)))
                            >> 24) as usize;
                        let slot = bucket * 2;
                        let val = pairs[slot + 1];
                        let c = if pairs[slot] == node_word && (val >> 24) == a as u32 {
                            ds.s3a_pairs_hits += 1;
                            val & 0x00FF_FFFF
                        } else {
                            let height = (node_word >> ID_BITS) as usize;
                            if height <= 3 {
                                ds.s3a_miss_g1_3 += 1;
                            } else if height == 4 {
                                let v = e.short_payload_h(node_word) as u32;
                                if (v & 0x8080_8080) == 0 {
                                    ds.s3a_miss_g4_ascii += 1;
                                } else {
                                    ds.s3a_miss_g4_nonascii += 1;
                                }
                            } else if height < INLINE {
                                ds.s3a_miss_g5_15 += 1;
                            } else {
                                ds.s3a_miss_long += 1;
                            }
                            let c_new = if height < INLINE {
                                let v = e.short_payload_h(node_word)
                                    | (u128::from(a as u8) << (8 * height));
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
                                    let c = lens.len() as u32;
                                    inline16.push(entry);
                                    lens.push(len);
                                    contents[index] = tag | c;
                                    c
                                }
                            } else {
                                let start = spelling.len();
                                let memoized = e.a[e.rule(node_word & ID_MASK) + 3] != u32::MAX;
                                e.append(node_word, &mut stack, &mut spelling, ds);
                                spelling.push(a as u8);
                                let entry = long_slot(start as u32, spelling.len() as u32);
                                let bytes = &spelling[start..];
                                let h = spelling_hash_long(bytes);
                                let tag = (((h >> 56) as u32) | 0x80) << 24;
                                let index = (h as usize) & 2047;
                                let ent = contents[index];
                                if (ent & 0xFF00_0000) == tag && {
                                    let old = (ent & 0x00FF_FFFF) as usize;
                                    lens[old] == LONG
                                        && long_word(&inline16[old], &spelling) == bytes
                                } {
                                    if memoized {
                                        spelling.truncate(start);
                                    } else {
                                        spelling.pop();
                                    }
                                    ent & 0x00FF_FFFF
                                } else {
                                    let c = lens.len() as u32;
                                    inline16.push(entry);
                                    lens.push(LONG);
                                    contents[index] = tag | c;
                                    c
                                }
                            };
                            pairs[slot] = node_word;
                            pairs[slot + 1] = ((a as u32) << 24) | c_new;
                            c_new
                        };
                        last_w = node_word;
                        last_c = c;
                        c
                    }
                };
                ds.bands_len += 1;
                ds.bands_max_w = ds.bands_max_w.max(width);
                ds.bands_max_dst = ds.bands_max_dst.max(dst);
                ds.bands_max_code = ds.bands_max_code.max(code);
                if width == 1 {
                    ds.bands_w1 += 1;
                }
                if width <= 2048 {
                    ds.bands_w_le_2048 += 1;
                } else {
                    ds.bands_w_gt_2048 += 1;
                }
                if node_word == 0 {
                    if width <= 2048 {
                        ds.bands_lit_w_le_2048 += 1;
                        ds.band32_words_lit_only += 1;
                        ds.band32_words_plus_w1 += 1;
                    } else {
                        ds.band32_words_lit_only += 3;
                        ds.band32_words_plus_w1 += 3;
                    }
                } else {
                    if width == 1 && code < 4096 {
                        ds.bands_word_w1_c4096 += 1;
                    }
                    if width == 1 && code < 8192 {
                        ds.bands_word_w1_c8192 += 1;
                    }
                    if width <= 2047 {
                        ds.bands_word_w_le_2047 += 1;
                        ds.band32_words_lit_only += 2;
                    } else {
                        ds.band32_words_lit_only += 3;
                    }
                    if m_fits_18 && width == 1 && code < 4096 {
                        ds.band32_words_plus_w1 += 1;
                    } else if width <= 2047 {
                        ds.band32_words_plus_w1 += 2;
                    } else {
                        ds.band32_words_plus_w1 += 3;
                    }
                }
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
        ds.spelling_bytes = spelling.len();
        ds.inline16_entries = lens.len();
        ds.long_entries = lens.iter().filter(|&&l| l == LONG).count();
    }

    if is_wide {
        run_diag::<Wide>(&mut edges, ns, n as u32, &f.thin, &first, &mut write, &mut ds);
    } else {
        run_diag::<Narrow>(&mut edges, ns, n as u32, &f.thin, &first, &mut write, &mut ds);
    }
    ds
}

struct Block {
    name: String,
    n: usize,
    p: u32,
    records: Vec<u16>,
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
            Block {
                name: f,
                n: l.len(),
                p,
                records: records_of(&l),
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

fn pct(a: usize, b: usize) -> f64 {
    if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 }
}

fn summarize_corpus(label: &str, blocks: &[Block], per_block: bool) {
    println!("\n================================================================================");
    println!("CORPUS: {label} ({} blocks)", blocks.len());
    println!("================================================================================");
    let stats: Vec<DiagStats> = blocks
        .iter()
        .map(|b| diagnose_block(&b.records, b.n, b.p))
        .collect();

    if per_block {
        println!(
            "{:<13} {:>6} {:>7} {:>7} {:>6} {:>6} {:>5} {:>5} {:>5} {:>5} {:>6} {:>6} | {:>5} {:>7} {:>6}",
            "Block", "ns", "r:Base", "r:I3", "g=4", "4:ASC", "g=5", "g=6", "g=7", "g=8", "9..15", ">=16", "bands", "spellB", "inl16"
        );
        for (b, s) in blocks.iter().zip(stats.iter()) {
            println!(
                "{:<13} {:>6} {:>7} {:>7} {:>6} {:>5.1}% {:>5} {:>5} {:>5} {:>5} {:>6} {:>6} | {:>5} {:>7} {:>6}",
                b.name,
                s.ns,
                s.s2b_rules_base,
                s.s2b_rules_i3,
                s.join_g4_total,
                pct(s.join_g4_ascii, s.join_g4_total),
                s.join_g5,
                s.join_g6,
                s.join_g7,
                s.join_g8,
                s.join_g9_15,
                s.join_g16_238 + s.join_g239_254 + s.join_g255p,
                s.bands_len,
                s.spelling_bytes,
                s.inline16_entries
            );
        }
    }

    // Aggregate totals
    let sum = |f: fn(&DiagStats) -> usize| -> usize { stats.iter().map(f).sum() };
    let r_base = sum(|s| s.s2b_rules_base);
    let r_i3 = sum(|s| s.s2b_rules_i3);
    let r_i4 = sum(|s| s.s2b_rules_i4);
    let g2 = sum(|s| s.join_g2);
    let g3 = sum(|s| s.join_g3);
    let g3a = sum(|s| s.join_g3_ascii);
    let g4 = sum(|s| s.join_g4_total);
    let g4a = sum(|s| s.join_g4_ascii);
    let g4na = sum(|s| s.join_g4_nonascii);
    let g5 = sum(|s| s.join_g5);
    let g5a = sum(|s| s.join_g5_ascii);
    let g6 = sum(|s| s.join_g6);
    let g7 = sum(|s| s.join_g7);
    let g8 = sum(|s| s.join_g8);
    let g9_15 = sum(|s| s.join_g9_15);
    let g16_238 = sum(|s| s.join_g16_238);
    let g239_254 = sum(|s| s.join_g239_254);
    let g255p = sum(|s| s.join_g255p);
    let g16p = g16_238 + g239_254 + g255p;

    println!("\n1) S2b Rule-Length & 7-bit ASCII Distribution (Post-F-Inline3):");
    println!("   Base rules (g>=3):          {:>9} (100.0%)", r_base);
    println!("   F-Inline3 rules (g>=4):     {:>9} ({:+.2}% vs Base; g=3 eliminated: {}, {:.2}% ASCII)",
        r_i3, 100.0 * (r_i3 as f64 - r_base as f64) / r_base.max(1) as f64, g3, pct(g3a, g3));
    println!("   F-Inline4-ASCII rules:      {:>9} ({:+.2}% vs F-Inline3, {:+.2}% vs Base)",
        r_i4,
        100.0 * (r_i4 as f64 - r_i3 as f64) / r_i3.max(1) as f64,
        100.0 * (r_i4 as f64 - r_base as f64) / r_base.max(1) as f64);
    println!("   Joins g=2 (inline Base/I3): {:>9}", g2);
    println!("   Post-I3 breakdown (denom = r_i3 = {}):", r_i3);
    println!("     g = 4 TOTAL:   {:>9} ({:>5.2}% of I3) | ASCII: {:>9} ({:>6.2}%) | Non-ASCII: {:>7} ({:>5.2}%)",
        g4, pct(g4, r_i3), g4a, pct(g4a, g4), g4na, pct(g4na, g4));
    println!("     g = 5:         {:>9} ({:>5.2}% of I3) | ASCII: {:>9} ({:>6.2}%)",
        g5, pct(g5, r_i3), g5a, pct(g5a, g5));
    println!("     g = 6:         {:>9} ({:>5.2}% of I3)", g6, pct(g6, r_i3));
    println!("     g = 7:         {:>9} ({:>5.2}% of I3) | sum g=4..7: {:>9} ({:>5.2}% of I3)",
        g7, pct(g7, r_i3), g4 + g5 + g6 + g7, pct(g4 + g5 + g6 + g7, r_i3));
    println!("     g = 8:         {:>9} ({:>5.2}% of I3) | sum g=4..8: {:>9} ({:>5.2}% of I3)",
        g8, pct(g8, r_i3), g4 + g5 + g6 + g7 + g8, pct(g4 + g5 + g6 + g7 + g8, r_i3));
    println!("     g = 9..=15:    {:>9} ({:>5.2}% of I3)", g9_15, pct(g9_15, r_i3));
    println!("     g >= 16:       {:>9} ({:>5.2}% of I3) | [16..238: {}, 239..254: {}, >=255: {}]",
        g16p, pct(g16p, r_i3), g16_238, g239_254, g255p);

    let w_i3 = sum(|s| s.s2b_words_i3);
    let w_i4 = sum(|s| s.s2b_words_i4);
    let w_i3p7 = sum(|s| s.s2b_words_i3_p7);
    let w_i3p8 = sum(|s| s.s2b_words_i3_p8);
    let w_i4p7 = sum(|s| s.s2b_words_i4_p7);
    let w_i4p8 = sum(|s| s.s2b_words_i4_p8);
    println!("\n2) S2b Rule-Arena Footprint (u32 words written across corpus):");
    println!("   F-Inline3 (4W all):         {:>10} words ({:.1} KiB) [1.000x]", w_i3, w_i3 as f64 * 4.0 / 1024.0);
    println!("   F-Inline4-ASCII (4W):       {:>10} words ({:.1} KiB) [{:.3}x, {:+.2}%]",
        w_i4, w_i4 as f64 * 4.0 / 1024.0, w_i4 as f64 / w_i3 as f64, 100.0 * (w_i4 as f64 - w_i3 as f64) / w_i3 as f64);
    println!("   F-Prefix7-2Word (on I3):    {:>10} words ({:.1} KiB) [{:.3}x, {:+.2}%]",
        w_i3p7, w_i3p7 as f64 * 4.0 / 1024.0, w_i3p7 as f64 / w_i3 as f64, 100.0 * (w_i3p7 as f64 - w_i3 as f64) / w_i3 as f64);
    println!("   F-Prefix8-2Word (on I3):    {:>10} words ({:.1} KiB) [{:.3}x, {:+.2}%]",
        w_i3p8, w_i3p8 as f64 * 4.0 / 1024.0, w_i3p8 as f64 / w_i3 as f64, 100.0 * (w_i3p8 as f64 - w_i3 as f64) / w_i3 as f64);
    println!("   F-Inline4 + Prefix7-2Word:  {:>10} words ({:.1} KiB) [{:.3}x, {:+.2}%]",
        w_i4p7, w_i4p7 as f64 * 4.0 / 1024.0, w_i4p7 as f64 / w_i3 as f64, 100.0 * (w_i4p7 as f64 - w_i3 as f64) / w_i3 as f64);
    println!("   F-Inline4 + Prefix8-2Word:  {:>10} words ({:.1} KiB) [{:.3}x, {:+.2}%]",
        w_i4p8, w_i4p8 as f64 * 4.0 / 1024.0, w_i4p8 as f64 / w_i3 as f64, 100.0 * (w_i4p8 as f64 - w_i3 as f64) / w_i3 as f64);

    let sp13 = sum(|s| s.sp_read_g1_3);
    let sp4a = sum(|s| s.sp_read_g4_ascii);
    let sp4na = sum(|s| s.sp_read_g4_nonascii);
    let sp57 = sum(|s| s.sp_read_g5_7);
    let sp8 = sum(|s| s.sp_read_g8);
    let sp915 = sum(|s| s.sp_read_g9_15);
    let sp_tot = sp13 + sp4a + sp4na + sp57 + sp8 + sp915;
    println!("\n3) S2b `short_payload_h` Operand Reads (total = {}):", sp_tot);
    println!("   g=1..3 (0-load I3): {:>8} ({:>5.2}%) | g=4 ASCII (0-load I4!): {:>8} ({:>5.2}%) | g=4 non-ASCII: {:>6} ({:>4.2}%)",
        sp13, pct(sp13, sp_tot), sp4a, pct(sp4a, sp_tot), sp4na, pct(sp4na, sp_tot));
    println!("   g=5..7 (2W load):   {:>8} ({:>5.2}%) | g=8 (2W load):          {:>8} ({:>5.2}%) | g=9..15 (4W):  {:>6} ({:>4.2}%)",
        sp57, pct(sp57, sp_tot), sp8, pct(sp8, sp_tot), sp915, pct(sp915, sp_tot));

    let d_calls = sum(|s| s.s3a_dict_calls);
    let d_hits = sum(|s| s.s3a_pairs_hits);
    let m13 = sum(|s| s.s3a_miss_g1_3);
    let m4a = sum(|s| s.s3a_miss_g4_ascii);
    let m4na = sum(|s| s.s3a_miss_g4_nonascii);
    let m515 = sum(|s| s.s3a_miss_g5_15);
    let mlong = sum(|s| s.s3a_miss_long);
    let m_tot = m13 + m4a + m4na + m515 + mlong;
    println!("   S3a Dict: calls={}, pairs_hit={:.2}%, misses={} [g1..3: {:.1}%, g4-ASC: {:.1}%, g4-nonASC: {:.2}%, g5..15: {:.1}%, long: {:.1}%]",
        d_calls, pct(d_hits, d_calls), m_tot, pct(m13, m_tot), pct(m4a, m_tot), pct(m4na, m_tot), pct(m515, m_tot), pct(mlong, m_tot));

    // Quantiles helper
    let qstats = |f: fn(&DiagStats) -> usize| -> (usize, usize, usize, usize) {
        let mut v: Vec<usize> = stats.iter().map(f).collect();
        v.sort_unstable();
        let l = v.len();
        (v[0], v[l / 2], v[(l * 90) / 100], v[l - 1])
    };
    let (b_min, b_p50, b_p90, b_max) = qstats(|s| s.bands_len);
    let (sp_min, sp_p50, sp_p90, sp_max) = qstats(|s| s.spelling_bytes);
    let (in_min, in_p50, in_p90, in_max) = qstats(|s| s.inline16_entries);
    let (ot_min, ot_p50, ot_p90, ot_max) = qstats(|s| s.out_tail_free_bytes);
    let (m_min, m_p50, m_p90, m_max) = qstats(|s| s.m);
    let (ar_min, ar_p50, ar_p90, ar_max) = qstats(|s| s.arena_peak_words_i3);

    let tot_bands = sum(|s| s.bands_len);
    let tot_lit = sum(|s| s.bands_literal);
    let tot_word = sum(|s| s.bands_word);
    let tot_w1 = sum(|s| s.bands_w1);
    let tot_w_gt = sum(|s| s.bands_w_gt_2048);
    let tot_b32_lit = sum(|s| s.band32_words_lit_only);
    let tot_b32_w1 = sum(|s| s.band32_words_plus_w1);
    let max_bw = stats.iter().map(|s| s.bands_max_w).max().unwrap_or(0);
    let max_bdst = stats.iter().map(|s| s.bands_max_dst).max().unwrap_or(0);
    let max_bcode = stats.iter().map(|s| s.bands_max_code).max().unwrap_or(0);
    let max_ovf = stats.iter().map(|s| s.outq_ovf).max().unwrap_or(0);

    println!("\n4) S3a Persistent Scratch Footprint vs Actual Utilization (min / p50 / p90 / max):");
    println!("   Pre-reserved in cand_r5_head (n>=65536): bands=65,536 B (8,192x8B) + spelling=49,152 B + inline16=28,672 B (1,792x16B) + lens=1,792 B = 145,152 B (141.75 KiB)");
    println!("   bands.len():       {:>6} / {:>6} / {:>6} / {:>6}   (x8B = {:.1} / {:.1} / {:.1} / {:.1} KiB)",
        b_min, b_p50, b_p90, b_max,
        b_min as f64 * 8.0 / 1024.0, b_p50 as f64 * 8.0 / 1024.0, b_p90 as f64 * 8.0 / 1024.0, b_max as f64 * 8.0 / 1024.0);
    println!("   spelling.len():    {:>6} / {:>6} / {:>6} / {:>6} B ({:.1} / {:.1} / {:.1} / {:.1} KiB)",
        sp_min, sp_p50, sp_p90, sp_max,
        sp_min as f64 / 1024.0, sp_p50 as f64 / 1024.0, sp_p90 as f64 / 1024.0, sp_max as f64 / 1024.0);
    println!("   inline16+lens len: {:>6} / {:>6} / {:>6} / {:>6}   (x17B = {:.1} / {:.1} / {:.1} / {:.1} KiB)",
        in_min, in_p50, in_p90, in_max,
        in_min as f64 * 17.0 / 1024.0, in_p50 as f64 * 17.0 / 1024.0, in_p90 as f64 * 17.0 / 1024.0, in_max as f64 * 17.0 / 1024.0);
    let actual_tot = |s: &DiagStats| s.bands_len * 8 + s.spelling_bytes + s.inline16_entries * 17;
    let (at_min, at_p50, at_p90, at_max) = qstats(actual_tot);
    println!("   ACTUAL S3a total:  {:.1} / {:.1} / {:.1} / {:.1} KiB (vs 141.75 KiB pre-reserved!)",
        at_min as f64 / 1024.0, at_p50 as f64 / 1024.0, at_p90 as f64 / 1024.0, at_max as f64 / 1024.0);
    println!("   edges m:           {:>6} / {:>6} / {:>6} / {:>6}   | arena_peak_words_i3: {} / {} / {} / {}",
        m_min, m_p50, m_p90, m_max, ar_min, ar_p50, ar_p90, ar_max);
    println!("   out_tail_free_B:   {:>6} / {:>6} / {:>6} / {:>6} B ({:.1} KiB min) | max outq_ovf={}",
        ot_min, ot_p50, ot_p90, ot_max, ot_min as f64 / 1024.0, max_ovf);

    println!("\n5) Band Anatomy & S3a-Band32 Compression Potential:");
    println!("   Total bands: {} | Literal (word==0): {} ({:.2}%) | Word: {} ({:.2}%) | width==1: {} ({:.2}%) | width>2048: {}",
        tot_bands, tot_lit, pct(tot_lit, tot_bands), tot_word, pct(tot_word, tot_bands), tot_w1, pct(tot_w1, tot_bands), tot_w_gt);
    println!("   Max band fields: max_width={}, max_dst={}, max_code={} (bits needed: dst={}, code={})",
        max_bw, max_bdst, max_bcode,
        32 - max_bdst.leading_zeros(), 32 - max_bcode.leading_zeros());
    println!("   Band(u64):             {:>9} bytes ({:.2} B/band)", tot_bands * 8, 8.0);
    println!("   Band32 (lit=1W,w=2W):  {:>9} bytes ({:.2} B/band, {:.1}% of u64)",
        tot_b32_lit * 4, tot_b32_lit as f64 * 4.0 / tot_bands.max(1) as f64, pct(tot_b32_lit * 4, tot_bands * 8));
    println!("   Band32 (+w1/c4K=1W):   {:>9} bytes ({:.2} B/band, {:.1}% of u64)",
        tot_b32_w1 * 4, tot_b32_w1 as f64 * 4.0 / tot_bands.max(1) as f64, pct(tot_b32_w1 * 4, tot_bands * 8));
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
    let nonwiki6 = load_bin_blocks(&nonwiki_path, Some(&nonwiki_names), false);
    let wiki233 = load_bin_blocks(&wiki431_path, None, true);

    summarize_corpus("6 Admitted Wikidata Blocks", &wiki6, true);
    summarize_corpus("6 Real Non-Wikidata Blocks (JSONL + Syslog)", &nonwiki6, true);
    if let Some(cd_path) = resolve_corpus_path(&[
        "/tmp/mibz2_corpora/non_wikidata/code_docs_blocks.bin",
        "/home/antigravity/mibz2_corpora_cache/non_wikidata/code_docs_blocks.bin",
    ]) {
        let cd_names = ["code_docs_0", "code_docs_1", "code_docs_2", "code_docs_3"];
        let code4 = load_bin_blocks(&cd_path, Some(&cd_names), false);
        summarize_corpus("4 Real Code & Docs Blocks", &code4, true);
    }
    summarize_corpus("233 Full wikidata.bz2 Blocks", &wiki233, false);
}
