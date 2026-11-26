#![forbid(unsafe_code)]
//! (research only, `multi-symbol-huffman-table`) Prototype and 7-repeat A/B probe of
//! 3-symbol (4 B/entry, 24 KiB for 6 tables) and 4-symbol (8 B/entry, 48 KiB for 6 tables)
//! 10-bit root Huffman tables against a faithful copy of the shipped 2-symbol decoder
//! (`src/huff.rs` + `src/block.rs` group loop).
//!
//!     cargo run --release --example huff_multi_symbol_probe -- corpora/wikidata.bz2 [REPEATS]
//!
//! Pre-parses every block's header, selectors, and code lengths once outside the clock, verifies
//! 100% symbol-for-symbol and ending-bit-position identity against a standalone canonical
//! bit-by-bit decoder on every block (plus synthetic edge cases for sym 256 and group tails),
//! then runs `REPEATS` (default 7) interleaved repeats timing A3 (table builds + group decode)
//! across all 431 blocks, plus per-stage (tables vs decode, full blocks vs runts), lookup
//! census, and per-lookup overhead diagnostics.

use std::hint::black_box;
use std::time::Instant;

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;
const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23;
const MAX_SELECTORS: usize = 18_002;
const MAX_SYMS: usize = 900_020;
const GROUP: usize = 50;
const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;
const STEPS: usize = 5;
const GATE_SEL: usize = 64;

// --- Bit reader (faithful mirror of src/bits.rs) ---

#[derive(Clone, Copy)]
struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    fn at(data: &[u8], bit: u64) -> Self {
        let mut r = Reg {
            pos: (bit / 8) as usize,
            buf: 0,
            live: 0,
        };
        let rem = (bit % 8) as u32;
        if rem != 0 {
            r.refill(data);
            r.skip(rem);
        }
        r
    }

    #[inline(always)]
    fn refill(&mut self, data: &[u8]) {
        if let Some(w) = data.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes(w.try_into().unwrap());
            self.buf |= v >> self.live;
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            *self = self.refill_slow(data);
        }
    }

    #[inline(always)]
    fn refill_win(&mut self, win: &[u8; 264]) {
        if let Some(w) = win.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes(w.try_into().unwrap());
            self.buf |= v >> self.live;
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            *self = self.refill_slow(win);
        }
    }

    #[cold]
    #[inline(never)]
    fn refill_slow(mut self, data: &[u8]) -> Reg {
        while self.live <= 56 {
            let b = data.get(self.pos).copied().unwrap_or(0);
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
        self
    }

    #[inline(always)]
    fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        if self.live < n {
            self.refill(data);
        }
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    fn skip_short(&mut self, n: u32) {
        self.buf <<= n & 63;
        self.live -= n;
    }

    #[inline(always)]
    fn skip_short_shl(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= e & 63;
    }

    #[inline(always)]
    fn skip(&mut self, n: u32) {
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
    }

    #[inline(always)]
    fn bits(&mut self, data: &[u8], n: u32) -> u32 {
        let v = self.peek(data, n);
        self.skip(n);
        v
    }

    #[inline(always)]
    fn bit(&mut self, data: &[u8]) -> bool {
        self.bits(data, 1) == 1
    }

    #[inline(always)]
    fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }

    #[inline(always)]
    fn overran(&self, data: &[u8]) -> bool {
        self.bit_pos() > data.len() as u64 * 8
    }
}

// --- Parsed block metadata (built once outside the clock) ---

struct ParsedBlock {
    alpha: usize,
    n_groups: usize,
    used_groups: u8,
    lens: [[u8; MAX_ALPHA]; 6],
    selectors: Vec<u8>,
    start_bit: u64,
    end_bit: u64,
    n_syms: usize,
    expected_syms: Vec<u16>,
}

fn parse_blocks(data: &[u8]) -> Vec<ParsedBlock> {
    let mut r = Reg::at(data, 0);
    let mut blocks = Vec::new();
    let total_bits = data.len() as u64 * 8;

    while r.bit_pos() + 32 <= total_bits {
        let hdr = r.bits(data, 24);
        assert_eq!(hdr, 0x425a68, "bad stream header");
        let _level = r.bits(data, 8);
        loop {
            let hi = r.bits(data, 24) as u64;
            let lo = r.bits(data, 24) as u64;
            let magic = (hi << 24) | lo;
            if magic == END_MAGIC {
                let _stream_crc = r.bits(data, 32);
                let rem = (8 - (r.bit_pos() % 8)) % 8;
                if rem != 0 {
                    r.skip(rem as u32);
                }
                break;
            }
            assert_eq!(magic, BLOCK_MAGIC, "bad block magic");
            let _crc = r.bits(data, 32);
            let _rand = r.bits(data, 1);
            let _orig_ptr = r.bits(data, 24);
            let used16 = r.bits(data, 16);
            let mut in_use = 0usize;
            for i in 0..16 {
                if (used16 >> (15 - i)) & 1 == 1 {
                    in_use += r.bits(data, 16).count_ones() as usize;
                }
            }
            let alpha = in_use + 2;
            let n_groups = r.bits(data, 3) as usize;
            let n_sel = r.bits(data, 15) as usize;
            let sel_cap = n_sel.min(MAX_SELECTORS);
            let mut mtf: Vec<u8> = (0..n_groups as u8).collect();
            let mut selectors = Vec::with_capacity(sel_cap);
            let mut used_groups = 0u8;
            for i in 0..n_sel {
                let mut j = 0usize;
                while r.bit(data) {
                    j += 1;
                }
                if i < MAX_SELECTORS {
                    let v = mtf.remove(j);
                    mtf.insert(0, v);
                    selectors.push(v);
                    used_groups |= 1u8 << v;
                }
            }
            let mut lens = [[0u8; MAX_ALPHA]; 6];
            for l in lens.iter_mut().take(n_groups) {
                let mut curr = r.bits(data, 5) as i32;
                for x in l.iter_mut().take(alpha) {
                    while r.bit(data) {
                        curr += if r.bit(data) { -1 } else { 1 };
                    }
                    *x = curr as u8;
                }
            }
            let start_bit = r.bit_pos();

            // Canonical bit-by-bit decode (independent reference oracle)
            let mut canon_tables = Vec::with_capacity(n_groups);
            for l in lens.iter().take(n_groups) {
                let sl = &l[..alpha];
                let mn = *sl.iter().min().unwrap() as usize;
                let mx = *sl.iter().max().unwrap() as usize;
                let mut perm = Vec::with_capacity(alpha);
                for len in mn..=mx {
                    for (s, &x) in sl.iter().enumerate() {
                        if x as usize == len {
                            perm.push(s as u16);
                        }
                    }
                }
                let mut base = [0i32; MAX_CODE_LEN];
                for &x in sl {
                    base[x as usize + 1] += 1;
                }
                for i in 1..MAX_CODE_LEN {
                    base[i] += base[i - 1];
                }
                let mut limit = [0i32; MAX_CODE_LEN];
                let mut vec = 0i32;
                for i in mn..=mx {
                    vec += base[i + 1] - base[i];
                    limit[i] = vec - 1;
                    vec <<= 1;
                }
                for i in mn + 1..=mx {
                    base[i] = ((limit[i - 1] + 1) << 1) - base[i];
                }
                canon_tables.push((limit, base, perm, mn as u32));
            }
            let eob = (alpha - 1) as u16;
            let mut expected_syms = Vec::new();
            'blk: for &g in &selectors {
                let (limit, base, perm, mn) = &canon_tables[g as usize];
                for _ in 0..GROUP {
                    let mut zn = *mn;
                    let mut v = r.bits(data, zn) as i32;
                    while v > limit[zn as usize] {
                        zn += 1;
                        v = (v << 1) | (r.bits(data, 1) as i32);
                    }
                    let s = perm[(v - base[zn as usize]) as usize];
                    expected_syms.push(s);
                    if s == eob {
                        break 'blk;
                    }
                }
            }
            let end_bit = r.bit_pos();
            let n_syms = expected_syms.len();
            blocks.push(ParsedBlock {
                alpha,
                n_groups,
                used_groups,
                lens,
                selectors,
                start_bit,
                end_bit,
                n_syms,
                expected_syms,
            });
        }
    }
    blocks
}

// --- Shared canonical base/limit/perm + SubLut builder ---

#[inline(always)]
fn build_canon(
    lens: &[u8],
    limit: &mut [i32; MAX_CODE_LEN],
    base: &mut [i32; MAX_CODE_LEN],
    perm: &mut [u16; MAX_ALPHA],
) -> (usize, usize) {
    let mut counts = [0i32; MAX_CODE_LEN];
    let mut min_l = 23usize;
    let mut max_l = 0usize;
    for &l in lens {
        let lu = l as usize;
        counts[lu] += 1;
        if lu < min_l {
            min_l = lu;
        }
        if lu > max_l {
            max_l = lu;
        }
    }
    let mut pos = [0u16; MAX_CODE_LEN];
    let mut acc = 0i32;
    base[0] = 0;
    for i in 1..MAX_CODE_LEN {
        base[i] = acc;
        pos[i] = acc as u16;
        acc += counts[i];
    }
    for (j, &l) in lens.iter().enumerate() {
        let lu = l as usize;
        let idx = pos[lu] as usize;
        perm[idx] = j as u16;
        pos[lu] = (idx + 1) as u16;
    }
    *limit = [0i32; MAX_CODE_LEN];
    let mut vec = 0i32;
    for i in min_l..=max_l {
        vec += counts[i];
        limit[i] = vec - 1;
        vec <<= 1;
    }
    for i in min_l + 1..=max_l {
        base[i] = ((limit[i - 1] + 1) << 1) - base[i];
    }
    (min_l, max_l)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn fill_sub_lut_u32(
    lut: &mut [u32; LUT_LEN],
    sub_lut: &mut [u16; SUB_CAP],
    limit: &[i32; MAX_CODE_LEN],
    base: &[i32; MAX_CODE_LEN],
    perm: &[u16; MAX_ALPHA],
    alpha: i32,
    min_len: u32,
    max_l: usize,
    filled_to: usize,
    a: i64,
    sub_ptr_flag: u32,
    bad_flag: u32,
) {
    if filled_to >= LUT_LEN {
        return;
    }
    if max_l <= LUT_BITS as usize {
        lut[filled_to..].fill(bad_flag);
        return;
    }
    lut[filled_to..].fill(0);
    let mut ranges = [(0u32, 0i64, -1i64); 10];
    let mut d_arr = [0u8; LUT_LEN];
    let mut max_root_w = filled_to;
    let start_zn = (LUT_BITS + 1).max(min_len);
    let mut a_hi = a;
    for zn in start_zn..=(max_l as u32) {
        let top = (1i64 << zn) - 1;
        let hi = (limit[zn as usize] as i64).min(top);
        ranges[(zn - 11) as usize] = (zn, a_hi, hi);
        if hi >= a_hi {
            let sh = zn - LUT_BITS;
            let w_lo = (a_hi >> sh) as usize;
            let w_hi = ((hi >> sh) as usize).min(LUT_LEN - 1);
            d_arr[w_lo..=w_hi].fill(sh as u8);
            if w_hi + 1 > max_root_w {
                max_root_w = w_hi + 1;
            }
        }
        a_hi = 2 * a_hi.max(hi + 1);
    }
    let mut sub_used = 0usize;
    for (w, &d_b) in d_arr[filled_to..max_root_w].iter().enumerate() {
        let d_w = d_b as u32;
        if d_w != 0 && sub_used + (1usize << d_w) <= SUB_CAP {
            lut[filled_to + w] = sub_ptr_flag | (d_w << 16) | (sub_used as u32);
            sub_used += 1usize << d_w;
        }
    }
    if sub_used > 0 {
        sub_lut[..sub_used].fill(SUB_BAD_U16);
        for &(zn, lo_c, hi_c) in &ranges {
            if zn == 0 || lo_c > hi_c {
                continue;
            }
            let sh = zn - LUT_BITS;
            for p in lo_c..=hi_c {
                let e_root = lut[(p >> sh) as usize];
                if (e_root & sub_ptr_flag) == 0 {
                    continue;
                }
                let d_w = (e_root >> 16) & 15;
                let sub_off = (e_root & 0xffff) as usize;
                let idx_sym = (p as i32) - base[zn as usize];
                let se = if (0..alpha).contains(&idx_sym) {
                    (perm[idx_sym as usize] << 6) | (zn as u16)
                } else {
                    SUB_BAD_U16
                };
                let pad = d_w - sh;
                let local = ((p as usize) & ((1usize << sh) - 1)) << pad;
                sub_lut[sub_off + local..sub_off + local + (1usize << pad)].fill(se);
            }
        }
    }
}

// ============================================================================
// 1. Shipped K = 2 decoder (faithful mirror of src/huff.rs + src/block.rs)
// ============================================================================

const K2_SUB_PTR: u32 = 1 << 27;
const K2_BAD: u32 = 1 << 28;
const K2_EOB_FAST: u32 = 1 << 29;
const K2_LEN1: u32 = 6;
const K2_SYM1: u32 = 11;
const K2_SYM2: u32 = 20;
const K2_CNT_SH: u32 = 30;

struct HuffK2 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffK2 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            sub_lut: [0; SUB_CAP],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl HuffK2 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], build_pairs: bool) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let (min_l, max_l) = build_canon(lens, &mut self.limit, &mut self.base, &mut self.perm);
        self.min_len = min_l as u32;
        self.alpha = alpha as i32;
        self.eob = eob;

        let min_len = min_l as u32;
        let max_pair_l1 = LUT_BITS.saturating_sub(min_len);
        let mut end_at_len = [0usize; 11];
        let mut zn_ranges = [(0i64, -1i64); 11];
        let mut a = 0i64;
        let mut filled_to = 0usize;

        for zn in min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            let sh = LUT_BITS - zn;
            for p in a..=hi {
                let e = match self.symbol(zn, p as i32) {
                    Some(s) if s == eob => K2_EOB_FAST | zn,
                    Some(s) => (1u32 << K2_CNT_SH) | ((s as u32) << K2_SYM1) | (zn << K2_LEN1) | zn,
                    None => K2_BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }

        fill_sub_lut_u32(
            &mut self.lut,
            &mut self.sub_lut,
            &self.limit,
            &self.base,
            &self.perm,
            self.alpha,
            min_len,
            max_l,
            filled_to,
            a,
            K2_SUB_PTR,
            K2_BAD,
        );

        if !build_pairs {
            return;
        }

        let mask = LUT_LEN - 1;
        for zn in min_len..=max_pair_l1 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = (LUT_BITS - zn) as usize;
            let u_lim = end_at_len[sh] >> zn;
            if u_lim == 0 {
                continue;
            }
            let mut tpl: Option<(usize, u32)> = None;
            for p in lo_p..=hi_p {
                let off_p = (p as usize) << sh;
                let e0 = self.lut[off_p];
                if (e0 >> K2_CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> K2_SYM1) & 0x1ff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << K2_SYM1;
                    for u in 0..u_lim {
                        self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let w = off_p + u;
                        let e = self.lut[w];
                        let e2 = self.lut[(u << zn) & mask];
                        let l2 = ((e2 >> K2_LEN1) & 31) as usize;
                        if (e2 >> K2_CNT_SH) == 0 || (zn as usize) + l2 > LUT_BITS as usize {
                            continue;
                        }
                        self.lut[w] = (2u32 << K2_CNT_SH)
                            | (((e2 >> K2_SYM1) & 0x1ff) << K2_SYM2)
                            | (e & (0x1ff << K2_SYM1))
                            | (zn << K2_LEN1)
                            | (zn + l2 as u32);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    #[inline(always)]
    fn decode_one_win(
        &self,
        r: &mut Reg,
        win: &[u8; 264],
        full_data: &[u8],
        base_pos: usize,
    ) -> u16 {
        if r.live < LUT_BITS {
            r.refill_win(win);
        }
        let e = self.lut[r.peek_live(LUT_BITS) as usize];
        if (e >> K2_CNT_SH) != 0 {
            r.skip_short((e >> K2_LEN1) & 31);
            return ((e >> K2_SYM1) & 0x1ff) as u16;
        }
        let (s, after) = self.decode_entry_slow_win(e, *r, win, full_data, base_pos);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_one_slice(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> K2_CNT_SH) != 0 {
            r.skip_short((e >> K2_LEN1) & 31);
            return ((e >> K2_SYM1) & 0x1ff) as u16;
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_group_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 66],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_win {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> K2_SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> K2_SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_win!();
            }
            if k > GROUP - 2 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        if k <= GROUP - 2 {
            r.refill_win(g_in);
            for _ in 0..4 {
                if k > GROUP - 2 {
                    break;
                }
                step_win!();
            }
            if k <= GROUP - 2 {
                r.refill_win(g_in);
                while k <= GROUP - 2 {
                    step_win!();
                }
            }
        }
        if k == GROUP - 1 {
            let s = self.decode_one_win(r, g_in, data, base_pos);
            g_out[GROUP - 1] = s;
            if s == eob {
                r.pos += base_pos;
                return (GROUP, true);
            }
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win_batched(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 66],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let mut used = 0u32;
        macro_rules! step_win_b {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                if c != 0 {
                    r.buf = r.buf.wrapping_shl(e);
                    used += e & 63;
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> K2_SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> K2_SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    r.live -= used;
                    used = 0;
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_win_b!();
            }
            r.live -= used;
            used = 0;
            if k > GROUP - 2 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        if k <= GROUP - 2 {
            r.refill_win(g_in);
            for _ in 0..4 {
                if k > GROUP - 2 {
                    break;
                }
                step_win_b!();
            }
            r.live -= used;
            used = 0;
            if k <= GROUP - 2 {
                r.refill_win(g_in);
                while k <= GROUP - 2 {
                    step_win_b!();
                }
                r.live -= used;
            }
        }
        if k == GROUP - 1 {
            let s = self.decode_one_win(r, g_in, data, base_pos);
            g_out[GROUP - 1] = s;
            if s == eob {
                r.pos += base_pos;
                return (GROUP, true);
            }
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_slice(&self, r: &mut Reg, data: &[u8], g_out: &mut [u16; 66]) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_slice {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> K2_SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> K2_SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        return (k, true);
                    }
                }
            }};
        }
        r.refill(data);
        loop {
            for _ in 0..STEPS {
                step_slice!();
            }
            if k > GROUP - 2 * STEPS {
                break;
            }
            r.refill(data);
        }
        if k <= GROUP - 2 {
            r.refill(data);
            for _ in 0..4 {
                if k > GROUP - 2 {
                    break;
                }
                step_slice!();
            }
            if k <= GROUP - 2 {
                r.refill(data);
                while k <= GROUP - 2 {
                    step_slice!();
                }
            }
        }
        if k == GROUP - 1 {
            let s = self.decode_one_slice(r, data);
            g_out[GROUP - 1] = s;
            if s == eob {
                return (GROUP, true);
            }
        }
        assert!(!(r.pos > data.len() && r.overran(data)));
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win_nostore(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        acc: &mut u64,
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let mut a = *acc;
        macro_rules! step_ns {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    a = a.wrapping_add(e as u64);
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    a = a.wrapping_add(s as u64);
                    k += 1;
                    if s == eob {
                        *acc = a;
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_ns!();
            }
            if k > GROUP - 2 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        if k <= GROUP - 2 {
            r.refill_win(g_in);
            for _ in 0..4 {
                if k > GROUP - 2 {
                    break;
                }
                step_ns!();
            }
            if k <= GROUP - 2 {
                r.refill_win(g_in);
                while k <= GROUP - 2 {
                    step_ns!();
                }
            }
        }
        if k == GROUP - 1 {
            let s = self.decode_one_win(r, g_in, data, base_pos);
            a = a.wrapping_add(s as u64);
            if s == eob {
                *acc = a;
                r.pos += base_pos;
                return (GROUP, true);
            }
        }
        *acc = a;
        r.pos += base_pos;
        (GROUP, false)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_win(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; 264],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & K2_SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_win(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_win(win);
                return (se >> 6, r);
            }
        } else if (e & K2_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_win(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & K2_SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill(data);
                return (se >> 6, r);
            }
        } else if (e & K2_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[cold]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid huff code")
    }
}

// ============================================================================
// 2. Packed 4-byte K = 3 decoder (4 KiB/table, 24 KiB for 6 tables)
// ============================================================================

const K3_SYM256: u32 = 1 << 26;
const K3_SUB_PTR: u32 = 1 << 27;
const K3_BAD: u32 = 1 << 28;
const K3_EOB_FAST: u32 = 1 << 29;
const K3_SYM1: u32 = 6;
const K3_SYM2: u32 = 14;
const K3_SYM3: u32 = 22;
const K3_CNT_SH: u32 = 30;

struct HuffK3 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    sym_len: [u8; MAX_ALPHA],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffK3 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            sub_lut: [0; SUB_CAP],
            sym_len: [0; MAX_ALPHA],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn build_packed_u32_levels(
    lens: &[u8],
    out_lut: &mut [u32; LUT_LEN],
    sub_lut: &mut [u16; SUB_CAP],
    sym_len: &mut [u8; MAX_ALPHA],
    limit: &mut [i32; MAX_CODE_LEN],
    base: &mut [i32; MAX_CODE_LEN],
    perm: &mut [u16; MAX_ALPHA],
    max_k: u32,
) -> (u32, [usize; 11], [(i64, i64); 11]) {
    let alpha = lens.len();
    let eob = (alpha - 1) as u16;
    sym_len[..alpha].copy_from_slice(lens);
    let (min_l, max_l) = build_canon(lens, limit, base, perm);
    let min_len = min_l as u32;
    let alpha_i32 = alpha as i32;

    let mut end_at_len = [0usize; 11];
    let mut zn_ranges = [(0i64, -1i64); 11];
    let mut a = 0i64;
    let mut filled_to = 0usize;

    for zn in min_len..=LUT_BITS {
        let top = (1i64 << zn) - 1;
        let hi = (limit[zn as usize] as i64).min(top);
        zn_ranges[zn as usize] = (a, hi);
        let sh = LUT_BITS - zn;
        for p in a..=hi {
            let idx = (p as i32) - base[zn as usize];
            let e = if (0..alpha_i32).contains(&idx) {
                let s = perm[idx as usize];
                if s == eob {
                    K3_EOB_FAST | zn
                } else if s >= 256 {
                    K3_SYM256 | zn
                } else {
                    (1u32 << K3_CNT_SH) | ((s as u32) << K3_SYM1) | zn
                }
            } else {
                K3_BAD
            };
            let end_idx = ((p + 1) << sh) as usize;
            out_lut[(p << sh) as usize..end_idx].fill(e);
            filled_to = end_idx;
        }
        end_at_len[zn as usize] = filled_to;
        a = 2 * a.max(hi + 1);
    }

    fill_sub_lut_u32(
        out_lut, sub_lut, limit, base, perm, alpha_i32, min_len, max_l, filled_to, a, K3_SUB_PTR,
        K3_BAD,
    );

    if max_k <= 1 {
        return (min_len, end_at_len, zn_ranges);
    }

    let lut1 = *out_lut;
    let mask = LUT_LEN - 1;
    let max_k2_l1 = LUT_BITS.saturating_sub(min_len);
    for zn in min_len..=max_k2_l1 {
        let (lo_p, hi_p) = zn_ranges[zn as usize];
        if lo_p > hi_p {
            continue;
        }
        let sh = (LUT_BITS - zn) as usize;
        let u_lim = end_at_len[sh] >> zn;
        if u_lim == 0 {
            continue;
        }
        let mut tpl: Option<(usize, u32)> = None;
        for p in lo_p..=hi_p {
            let off_p = (p as usize) << sh;
            let e0 = lut1[off_p];
            if (e0 >> K3_CNT_SH) == 0 {
                continue;
            }
            let s1 = (e0 >> K3_SYM1) & 0xff;
            if let Some((off0, s0)) = tpl {
                let delta = s1.wrapping_sub(s0) << K3_SYM1;
                for u in 0..u_lim {
                    out_lut[off_p + u] = out_lut[off0 + u].wrapping_add(delta);
                }
            } else {
                for u in 0..u_lim {
                    let e2 = lut1[(u << zn) & mask];
                    let l2 = e2 & 63;
                    if (e2 >> K3_CNT_SH) == 0 || zn + l2 > LUT_BITS {
                        continue;
                    }
                    out_lut[off_p + u] = (2u32 << K3_CNT_SH)
                        | (((e2 >> K3_SYM1) & 0xff) << K3_SYM2)
                        | (s1 << K3_SYM1)
                        | (zn + l2);
                }
                tpl = Some((off_p, s1));
            }
        }
    }

    if max_k <= 2 {
        return (min_len, end_at_len, zn_ranges);
    }

    let max_k3_l1 = LUT_BITS.saturating_sub(2 * min_len);
    if min_len <= max_k3_l1 {
        let lut2 = *out_lut;
        for zn in min_len..=max_k3_l1 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = (LUT_BITS - zn) as usize;
            let max_l2 = (LUT_BITS - zn - min_len) as usize;
            let u_lim = end_at_len[max_l2] >> zn;
            if u_lim == 0 {
                continue;
            }
            let mut tpl: Option<(usize, u32)> = None;
            for p in lo_p..=hi_p {
                let off_p = (p as usize) << sh;
                let e0 = lut1[off_p];
                if (e0 >> K3_CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> K3_SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << K3_SYM1;
                    for u in 0..u_lim {
                        out_lut[off_p + u] = out_lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let e23 = lut2[(u << zn) & mask];
                        if (e23 >> K3_CNT_SH) == 2 {
                            let l23 = e23 & 63;
                            if zn + l23 <= LUT_BITS {
                                out_lut[off_p + u] = (3u32 << K3_CNT_SH)
                                    | ((e23 & 0x003f_ffc0) << 8)
                                    | (s1 << K3_SYM1)
                                    | (zn + l23);
                            }
                        }
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    (min_len, end_at_len, zn_ranges)
}

impl HuffK3 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], max_k: u32) {
        self.alpha = lens.len() as i32;
        self.eob = (lens.len() - 1) as u16;
        let (min_len, _, _) = build_packed_u32_levels(
            lens,
            &mut self.lut,
            &mut self.sub_lut,
            &mut self.sym_len,
            &mut self.limit,
            &mut self.base,
            &mut self.perm,
            max_k,
        );
        self.min_len = min_len;
    }

    /// Branch-lean exact tail handler for `k == 48` or `k == 49`:
    /// - At `k == 48`: `c >= 2` (~85%) writes `s1, s2` and finishes immediately (no `c == 2` vs `3` split);
    ///   `c == 1` writes `s1` and falls through to `k == 49`.
    /// - At `k == 49`: any `c >= 1` (~99%) writes `s1` and skips `sym_len[s1]` directly (no `c <= rem` check).
    #[inline(always)]
    fn finish_tail_lean_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
        mut k: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        if k == GROUP - 2 {
            if r.live < LUT_BITS {
                r.refill_win(g_in);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> K3_CNT_SH) as usize;
            if c >= 2 {
                let s1 = ((e >> K3_SYM1) & 0xff) as u16;
                let s2 = ((e >> K3_SYM2) & 0xff) as u16;
                g_out[GROUP - 2] = s1;
                g_out[GROUP - 1] = s2;
                let bits = (self.sym_len[s1 as usize] + self.sym_len[s2 as usize]) as u32;
                r.skip_short(bits);
                r.pos += base_pos;
                return (GROUP, false);
            } else if c == 1 {
                r.skip_short_shl(e);
                g_out[GROUP - 2] = ((e >> K3_SYM1) & 0xff) as u16;
                k = GROUP - 1;
            } else {
                let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                *r = after;
                g_out[GROUP - 2] = s;
                if s == eob {
                    r.pos += base_pos;
                    return (GROUP - 1, true);
                }
                k = GROUP - 1;
            }
        }
        if k == GROUP - 1 {
            if r.live < LUT_BITS {
                r.refill_win(g_in);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            if (e >> K3_CNT_SH) != 0 {
                let s1 = ((e >> K3_SYM1) & 0xff) as u16;
                r.skip_short(self.sym_len[s1 as usize] as u32);
                g_out[GROUP - 1] = s1;
            } else {
                let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                *r = after;
                g_out[GROUP - 1] = s;
                if s == eob {
                    r.pos += base_pos;
                    return (GROUP, true);
                }
            }
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_win {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 3];
                    dst[0] = ((e >> K3_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K3_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K3_SYM3) & 0xff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_win!();
            }
            if k > GROUP - 3 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        while k <= GROUP - 3 {
            r.refill_win(g_in);
            for _ in 0..STEPS {
                if k > GROUP - 3 {
                    break;
                }
                step_win!();
            }
        }
        if k < GROUP {
            return self.finish_tail_lean_win(r, g_in, data, base_pos, g_out, k);
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win_batched<const S: usize>(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let mut used = 0u32;
        macro_rules! step_win_b {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                if c != 0 {
                    r.buf = r.buf.wrapping_shl(e);
                    used += e & 63;
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 3];
                    dst[0] = ((e >> K3_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K3_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K3_SYM3) & 0xff) as u16;
                    k += c;
                } else {
                    r.live -= used;
                    used = 0;
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..S {
                step_win_b!();
            }
            r.live -= used;
            used = 0;
            if k > GROUP - 3 * S {
                break;
            }
            r.refill_win(g_in);
        }
        while k <= GROUP - 3 {
            r.refill_win(g_in);
            for _ in 0..STEPS {
                if k > GROUP - 3 {
                    break;
                }
                step_win_b!();
            }
            r.live -= used;
            used = 0;
        }
        if k < GROUP {
            return self.finish_tail_lean_win(r, g_in, data, base_pos, g_out, k);
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win_nostore(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        acc: &mut u64,
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let mut a = *acc;
        macro_rules! step_ns {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    a = a.wrapping_add(e as u64);
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    a = a.wrapping_add(s as u64);
                    k += 1;
                    if s == eob {
                        *acc = a;
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_ns!();
            }
            if k > GROUP - 3 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        while k <= GROUP - 3 {
            r.refill_win(g_in);
            for _ in 0..STEPS {
                if k > GROUP - 3 {
                    break;
                }
                step_ns!();
            }
        }
        if k == GROUP - 2 {
            if r.live < LUT_BITS {
                r.refill_win(g_in);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> K3_CNT_SH) as usize;
            if c >= 2 {
                let s1 = ((e >> K3_SYM1) & 0xff) as u16;
                let s2 = ((e >> K3_SYM2) & 0xff) as u16;
                r.skip_short((self.sym_len[s1 as usize] + self.sym_len[s2 as usize]) as u32);
                a = a.wrapping_add(e as u64);
                *acc = a;
                r.pos += base_pos;
                return (GROUP, false);
            } else if c == 1 {
                r.skip_short_shl(e);
                a = a.wrapping_add(e as u64);
                k = GROUP - 1;
            } else {
                let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                *r = after;
                a = a.wrapping_add(s as u64);
                if s == eob {
                    *acc = a;
                    r.pos += base_pos;
                    return (GROUP - 1, true);
                }
                k = GROUP - 1;
            }
        }
        if k == GROUP - 1 {
            if r.live < LUT_BITS {
                r.refill_win(g_in);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            if (e >> K3_CNT_SH) != 0 {
                let s1 = ((e >> K3_SYM1) & 0xff) as u16;
                r.skip_short(self.sym_len[s1 as usize] as u32);
                a = a.wrapping_add(s1 as u64);
            } else {
                let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                *r = after;
                a = a.wrapping_add(s as u64);
                if s == eob {
                    *acc = a;
                    r.pos += base_pos;
                    return (GROUP, true);
                }
            }
        }
        *acc = a;
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_slice(&self, r: &mut Reg, data: &[u8], g_out: &mut [u16; 68]) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        while k < GROUP {
            if r.live < LUT_BITS {
                r.refill(data);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> K3_CNT_SH) as usize;
            if c != 0 {
                let rem = GROUP - k;
                if c <= rem {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 3];
                    dst[0] = ((e >> K3_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K3_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K3_SYM3) & 0xff) as u16;
                    k += c;
                } else {
                    let s1 = ((e >> K3_SYM1) & 0xff) as u16;
                    g_out[k] = s1;
                    let mut bits = self.sym_len[s1 as usize] as u32;
                    if rem == 2 {
                        let s2 = ((e >> K3_SYM2) & 0xff) as u16;
                        g_out[k + 1] = s2;
                        bits += self.sym_len[s2 as usize] as u32;
                    }
                    r.skip_short(bits);
                    k = GROUP;
                }
            } else {
                let (s, after) = self.decode_entry_slow(e, *r, data);
                *r = after;
                g_out[k] = s;
                k += 1;
                if s == eob {
                    return (k, true);
                }
            }
        }
        assert!(!(r.pos > data.len() && r.overran(data)));
        (GROUP, false)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_win(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; 264],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & K3_SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_win(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_win(win);
                return (se >> 6, r);
            }
        } else if (e & K3_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & K3_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_win(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & K3_SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill(data);
                return (se >> 6, r);
            }
        } else if (e & K3_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & K3_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[cold]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid huff code")
    }
}

// ============================================================================
// 3. Packed 8-byte K = 4 decoder (8 KiB/table, 48 KiB for 6 tables)
// ============================================================================

const K4_SYM1: u32 = 6;
const K4_SYM2: u32 = 14;
const K4_SYM3: u32 = 22;
const K4_SYM4: u32 = 30;
const K4_CNT_SH: u32 = 61;

struct HuffK4 {
    lut: [u64; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    sym_len: [u8; MAX_ALPHA],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffK4 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            sub_lut: [0; SUB_CAP],
            sym_len: [0; MAX_ALPHA],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl HuffK4 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], max_k: u32) {
        self.alpha = lens.len() as i32;
        self.eob = (lens.len() - 1) as u16;
        let mut lut3 = [0u32; LUT_LEN];
        let (min_len, end_at_len, zn_ranges) = build_packed_u32_levels(
            lens,
            &mut lut3,
            &mut self.sub_lut,
            &mut self.sym_len,
            &mut self.limit,
            &mut self.base,
            &mut self.perm,
            max_k.min(3),
        );
        self.min_len = min_len;

        for (dst, &e3) in self.lut.iter_mut().zip(lut3.iter()) {
            *dst = (((e3 >> K3_CNT_SH) as u64) << K4_CNT_SH) | ((e3 & 0x3fff_ffff) as u64);
        }

        if max_k <= 3 {
            return;
        }

        let max_k4_l1 = LUT_BITS.saturating_sub(3 * min_len);
        if min_len <= max_k4_l1 {
            let mask = LUT_LEN - 1;
            for zn in min_len..=max_k4_l1 {
                let (lo_p, hi_p) = zn_ranges[zn as usize];
                if lo_p > hi_p {
                    continue;
                }
                let sh = (LUT_BITS - zn) as usize;
                let max_l2 = (LUT_BITS - zn - 2 * min_len) as usize;
                let u_lim = end_at_len[max_l2] >> zn;
                if u_lim == 0 {
                    continue;
                }
                let mut tpl: Option<(usize, u64)> = None;
                for p in lo_p..=hi_p {
                    let off_p = (p as usize) << sh;
                    let e0 = lut3[off_p];
                    if (e0 >> K3_CNT_SH) == 0 {
                        continue;
                    }
                    let s1 = ((e0 >> K3_SYM1) & 0xff) as u64;
                    if let Some((off0, s0)) = tpl {
                        let delta = s1.wrapping_sub(s0) << K4_SYM1;
                        for u in 0..u_lim {
                            self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                        }
                    } else {
                        for u in 0..u_lim {
                            let e234 = lut3[(u << zn) & mask];
                            if (e234 >> K3_CNT_SH) == 3 {
                                let l234 = e234 & 63;
                                if zn + l234 <= LUT_BITS {
                                    self.lut[off_p + u] = (4u64 << K4_CNT_SH)
                                        | (((e234 & 0x3fff_ffc0) as u64) << 8)
                                        | (s1 << K4_SYM1)
                                        | ((zn + l234) as u64);
                                }
                            }
                        }
                        tpl = Some((off_p, s1));
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn finish_tail_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
        mut k: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        while k < GROUP {
            if r.live < LUT_BITS {
                r.refill_win(g_in);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> K4_CNT_SH) as usize;
            if c != 0 {
                let rem = GROUP - k;
                if c <= rem {
                    r.skip_short_shl(e as u32);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 3];
                    dst[0] = ((e >> K4_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K4_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K4_SYM3) & 0xff) as u16;
                    k += c;
                } else {
                    let s1 = ((e >> K4_SYM1) & 0xff) as u16;
                    g_out[k] = s1;
                    let mut bits = self.sym_len[s1 as usize] as u32;
                    if rem >= 2 {
                        let s2 = ((e >> K4_SYM2) & 0xff) as u16;
                        g_out[k + 1] = s2;
                        bits += self.sym_len[s2 as usize] as u32;
                    }
                    if rem == 3 {
                        let s3 = ((e >> K4_SYM3) & 0xff) as u16;
                        g_out[k + 2] = s3;
                        bits += self.sym_len[s3 as usize] as u32;
                    }
                    r.skip_short(bits);
                    k = GROUP;
                }
            } else {
                let (s, after) = self.decode_entry_slow_win(e as u32, *r, g_in, data, base_pos);
                *r = after;
                g_out[k] = s;
                k += 1;
                if s == eob {
                    r.pos += base_pos;
                    return (k, true);
                }
            }
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_win {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K4_CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e as u32);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 4];
                    dst[0] = ((e >> K4_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K4_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K4_SYM3) & 0xff) as u16;
                    dst[3] = ((e >> K4_SYM4) & 0xff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e as u32, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_win!();
            }
            if k > GROUP - 4 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        while k <= GROUP - 4 {
            r.refill_win(g_in);
            for _ in 0..STEPS {
                if k > GROUP - 4 {
                    break;
                }
                step_win!();
            }
        }
        if k < GROUP {
            return self.finish_tail_win(r, g_in, data, base_pos, g_out, k);
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_win_batched(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 68],
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let mut used = 0u32;
        macro_rules! step_win_b {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> K4_CNT_SH) as usize;
                if c != 0 {
                    r.buf = r.buf.wrapping_shl(e as u32);
                    used += (e as u32) & 63;
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 4];
                    dst[0] = ((e >> K4_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K4_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K4_SYM3) & 0xff) as u16;
                    dst[3] = ((e >> K4_SYM4) & 0xff) as u16;
                    k += c;
                } else {
                    r.live -= used;
                    used = 0;
                    let (s, after) = self.decode_entry_slow_win(e as u32, *r, g_in, data, base_pos);
                    *r = after;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_win(g_in);
        loop {
            for _ in 0..STEPS {
                step_win_b!();
            }
            r.live -= used;
            used = 0;
            if k > GROUP - 4 * STEPS {
                break;
            }
            r.refill_win(g_in);
        }
        while k <= GROUP - 4 {
            r.refill_win(g_in);
            for _ in 0..STEPS {
                if k > GROUP - 4 {
                    break;
                }
                step_win_b!();
            }
            r.live -= used;
            used = 0;
        }
        if k < GROUP {
            return self.finish_tail_win(r, g_in, data, base_pos, g_out, k);
        }
        r.pos += base_pos;
        (GROUP, false)
    }

    #[inline(always)]
    fn decode_group_slice(&self, r: &mut Reg, data: &[u8], g_out: &mut [u16; 68]) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        while k < GROUP {
            if r.live < LUT_BITS {
                r.refill(data);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> K4_CNT_SH) as usize;
            if c != 0 {
                let rem = GROUP - k;
                if c <= rem {
                    r.skip_short_shl(e as u32);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 4];
                    dst[0] = ((e >> K4_SYM1) & 0xff) as u16;
                    dst[1] = ((e >> K4_SYM2) & 0xff) as u16;
                    dst[2] = ((e >> K4_SYM3) & 0xff) as u16;
                    dst[3] = ((e >> K4_SYM4) & 0xff) as u16;
                    k += c;
                } else {
                    let s1 = ((e >> K4_SYM1) & 0xff) as u16;
                    g_out[k] = s1;
                    let mut bits = self.sym_len[s1 as usize] as u32;
                    if rem >= 2 {
                        let s2 = ((e >> K4_SYM2) & 0xff) as u16;
                        g_out[k + 1] = s2;
                        bits += self.sym_len[s2 as usize] as u32;
                    }
                    if rem == 3 {
                        let s3 = ((e >> K4_SYM3) & 0xff) as u16;
                        g_out[k + 2] = s3;
                        bits += self.sym_len[s3 as usize] as u32;
                    }
                    r.skip_short(bits);
                    k = GROUP;
                }
            } else {
                let (s, after) = self.decode_entry_slow(e as u32, *r, data);
                *r = after;
                g_out[k] = s;
                k += 1;
                if s == eob {
                    return (k, true);
                }
            }
        }
        assert!(!(r.pos > data.len() && r.overran(data)));
        (GROUP, false)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_win(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; 264],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & K3_SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_win(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_win(win);
                return (se >> 6, r);
            }
        } else if (e & K3_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & K3_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_win(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & K3_SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill(data);
                return (se >> 6, r);
            }
        } else if (e & K3_EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & K3_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[cold]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid huff code")
    }
}

// ============================================================================
// Dedicated #[inline(never)] per-arm runners (no shared switch bloat)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    K2Shipped,
    K2Gated,
    K2Batched,
    K2Slice,
    K3FourB,
    K3FourBGated,
    K3FourBBatched5,
    K3FourBBatched4,
    K4EightB,
    K4EightBGated,
    K4EightBBatched,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::K2Shipped => "k2_shipped",
            Arm::K2Gated => "k2_gated",
            Arm::K2Batched => "k2_batched",
            Arm::K2Slice => "k2_slice",
            Arm::K3FourB => "k3_4b",
            Arm::K3FourBGated => "k3_4b_gated",
            Arm::K3FourBBatched5 => "k3_4b_batch5",
            Arm::K3FourBBatched4 => "k3_4b_batch4",
            Arm::K4EightB => "k4_8b",
            Arm::K4EightBGated => "k4_8b_gated",
            Arm::K4EightBBatched => "k4_8b_batched",
        }
    }
}

struct Scratch {
    k2: [HuffK2; 6],
    k3: [HuffK3; 6],
    k4: [HuffK4; 6],
    syms: Vec<u16>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            k2: std::array::from_fn(|_| HuffK2::default()),
            k3: std::array::from_fn(|_| HuffK3::default()),
            k4: std::array::from_fn(|_| HuffK4::default()),
            syms: vec![0u16; MAX_SYMS + 68],
        }
    }
}

#[inline(never)]
fn build_block_tables(arm: Arm, blk: &ParsedBlock, sc: &mut Scratch) {
    let gated_big = blk.selectors.len() >= GATE_SEL;
    match arm {
        Arm::K2Shipped | Arm::K2Batched | Arm::K2Slice => {
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k2[g].build_into(&blk.lens[g][..blk.alpha], true);
                }
            }
        }
        Arm::K2Gated => {
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k2[g].build_into(&blk.lens[g][..blk.alpha], gated_big);
                }
            }
        }
        Arm::K3FourB => {
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k3[g].build_into(&blk.lens[g][..blk.alpha], 3);
                }
            }
        }
        Arm::K3FourBGated | Arm::K3FourBBatched5 | Arm::K3FourBBatched4 => {
            let mk = if gated_big { 3 } else { 1 };
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k3[g].build_into(&blk.lens[g][..blk.alpha], mk);
                }
            }
        }
        Arm::K4EightB => {
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k4[g].build_into(&blk.lens[g][..blk.alpha], 4);
                }
            }
        }
        Arm::K4EightBGated | Arm::K4EightBBatched => {
            let mk = if gated_big { 4 } else { 1 };
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    sc.k4[g].build_into(&blk.lens[g][..blk.alpha], mk);
                }
            }
        }
    }
}

#[inline(never)]
fn dec_k2_shipped(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 66).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 66];
    for &sel in &blk.selectors {
        let t = &sc.k2[sel as usize];
        let use_tail = n + 66 > max_out;
        let g_out: &mut [u16; 66] = if !use_tail {
            (&mut out[n..n + 66]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k2_batched(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 66).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 66];
    for &sel in &blk.selectors {
        let t = &sc.k2[sel as usize];
        let use_tail = n + 66 > max_out;
        let g_out: &mut [u16; 66] = if !use_tail {
            (&mut out[n..n + 66]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win_batched(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k2_slice(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let max_out = (blk.selectors.len() * GROUP + 66).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 66];
    for &sel in &blk.selectors {
        let t = &sc.k2[sel as usize];
        let use_tail = n + 66 > max_out;
        let g_out: &mut [u16; 66] = if !use_tail {
            (&mut out[n..n + 66]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = t.decode_group_slice(&mut r, data, g_out);
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k3_win(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 68).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 68];
    for &sel in &blk.selectors {
        let t = &sc.k3[sel as usize];
        let use_tail = n + 68 > max_out;
        let g_out: &mut [u16; 68] = if !use_tail {
            (&mut out[n..n + 68]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k3_batched<const S: usize>(
    blk: &ParsedBlock,
    data: &[u8],
    sc: &mut Scratch,
) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 68).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 68];
    for &sel in &blk.selectors {
        let t = &sc.k3[sel as usize];
        let use_tail = n + 68 > max_out;
        let g_out: &mut [u16; 68] = if !use_tail {
            (&mut out[n..n + 68]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win_batched::<S>(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k4_win(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 68).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 68];
    for &sel in &blk.selectors {
        let t = &sc.k4[sel as usize];
        let use_tail = n + 68 > max_out;
        let g_out: &mut [u16; 68] = if !use_tail {
            (&mut out[n..n + 68]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

#[inline(never)]
fn dec_k4_batched(blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    let data_len = data.len();
    let max_out = (blk.selectors.len() * GROUP + 68).min(MAX_SYMS);
    let out = &mut sc.syms[..max_out];
    let mut r = Reg::at(data, blk.start_bit);
    let mut n = 0usize;
    let mut tail = [0u16; 68];
    for &sel in &blk.selectors {
        let t = &sc.k4[sel as usize];
        let use_tail = n + 68 > max_out;
        let g_out: &mut [u16; 68] = if !use_tail {
            (&mut out[n..n + 68]).try_into().unwrap()
        } else {
            &mut tail
        };
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            t.decode_group_win_batched(&mut r, g_in, data, base_pos, g_out)
        } else {
            t.decode_group_slice(&mut r, data, g_out)
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

fn decode_block_syms(arm: Arm, blk: &ParsedBlock, data: &[u8], sc: &mut Scratch) -> (usize, u64) {
    match arm {
        Arm::K2Shipped | Arm::K2Gated => dec_k2_shipped(blk, data, sc),
        Arm::K2Batched => dec_k2_batched(blk, data, sc),
        Arm::K2Slice => dec_k2_slice(blk, data, sc),
        Arm::K3FourB | Arm::K3FourBGated => dec_k3_win(blk, data, sc),
        Arm::K3FourBBatched5 => dec_k3_batched::<5>(blk, data, sc),
        Arm::K3FourBBatched4 => dec_k3_batched::<4>(blk, data, sc),
        Arm::K4EightB | Arm::K4EightBGated => dec_k4_win(blk, data, sc),
        Arm::K4EightBBatched => dec_k4_batched(blk, data, sc),
    }
}

#[inline(never)]
fn run_nostore_k2(blocks: &[ParsedBlock], data: &[u8], sc: &mut Scratch) -> u64 {
    let mut acc = 0u64;
    let data_len = data.len();
    for blk in blocks {
        if blk.n_syms < 1000 {
            continue;
        }
        build_block_tables(Arm::K2Shipped, blk, sc);
        let mut r = Reg::at(data, blk.start_bit);
        let mut tail66 = [0u16; 66];
        for &sel in &blk.selectors {
            let t = &sc.k2[sel as usize];
            let (_, hit_eob) = if r.pos + 264 <= data_len {
                let base_pos = r.pos;
                let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                t.decode_group_win_nostore(&mut r, g_in, data, base_pos, &mut acc)
            } else {
                t.decode_group_slice(&mut r, data, &mut tail66)
            };
            if hit_eob {
                break;
            }
        }
    }
    acc
}

#[inline(never)]
fn run_nostore_k3(blocks: &[ParsedBlock], data: &[u8], sc: &mut Scratch) -> u64 {
    let mut acc = 0u64;
    let data_len = data.len();
    for blk in blocks {
        if blk.n_syms < 1000 {
            continue;
        }
        build_block_tables(Arm::K3FourB, blk, sc);
        let mut r = Reg::at(data, blk.start_bit);
        let mut tail68 = [0u16; 68];
        for &sel in &blk.selectors {
            let t = &sc.k3[sel as usize];
            let (_, hit_eob) = if r.pos + 264 <= data_len {
                let base_pos = r.pos;
                let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                t.decode_group_win_nostore(&mut r, g_in, data, base_pos, &mut acc)
            } else {
                t.decode_group_slice(&mut r, data, &mut tail68)
            };
            if hit_eob {
                break;
            }
        }
    }
    acc
}

/// Synthetic edge-case test: alpha=258 with short sym=256 (len=3), EOB (len=4), and len-12 SUB_PTR.
fn verify_synthetic_edge_cases() {
    let mut lens = [0u8; MAX_ALPHA];
    lens[0] = 2;
    lens[1] = 2;
    for l in lens.iter_mut().take(71).skip(2) {
        *l = 9;
    }
    for l in lens.iter_mut().take(252).skip(71) {
        *l = 10;
    }
    for l in lens.iter_mut().take(256).skip(252) {
        *l = 12;
    }
    lens[256] = 3;
    lens[257] = 4; // EOB

    let mut limit = [0i32; MAX_CODE_LEN];
    let mut base = [0i32; MAX_CODE_LEN];
    let mut perm = [0u16; MAX_ALPHA];
    let (mn, mx) = build_canon(&lens, &mut limit, &mut base, &mut perm);
    let mut sym_code = [(0u32, 0u8); MAX_ALPHA];
    let mut idx = 0usize;
    let mut code = 0u32;
    for len in mn..=mx {
        let mut cnt = 0usize;
        while idx + cnt < MAX_ALPHA && lens[perm[idx + cnt] as usize] as usize == len {
            sym_code[perm[idx + cnt] as usize] = (code, len as u8);
            code += 1;
            cnt += 1;
        }
        idx += cnt;
        code <<= 1;
    }

    let mut syms = Vec::new();
    for i in 0..149usize {
        let s = match i {
            46..=51 => 0,
            97..=101 => {
                if i == 99 {
                    256
                } else {
                    1
                }
            }
            10 => 256,
            20 => 252,
            30 => 255,
            148 => 256,
            _ => (i % 4) as u16,
        };
        syms.push(s);
    }
    syms.push(257);

    let mut bit_buf = Vec::new();
    let mut cur_byte = 0u8;
    let mut cur_bits = 0u32;
    for &s in &syms {
        let (c, l) = sym_code[s as usize];
        for b in (0..l).rev() {
            cur_byte = (cur_byte << 1) | (((c >> b) & 1) as u8);
            cur_bits += 1;
            if cur_bits == 8 {
                bit_buf.push(cur_byte);
                cur_byte = 0;
                cur_bits = 0;
            }
        }
    }
    let end_bit = bit_buf.len() as u64 * 8 + cur_bits as u64;
    if cur_bits > 0 {
        bit_buf.push(cur_byte << (8 - cur_bits));
    }
    bit_buf.resize(bit_buf.len() + 300, 0);

    let blk = ParsedBlock {
        alpha: MAX_ALPHA,
        n_groups: 1,
        used_groups: 1,
        lens: {
            let mut a = [[0u8; MAX_ALPHA]; 6];
            a[0] = lens;
            a
        },
        selectors: vec![0, 0, 0],
        start_bit: 0,
        end_bit,
        n_syms: syms.len(),
        expected_syms: syms,
    };

    let mut sc = Scratch::new();
    for arm in [
        Arm::K2Shipped,
        Arm::K2Batched,
        Arm::K2Slice,
        Arm::K3FourB,
        Arm::K3FourBBatched5,
        Arm::K3FourBBatched4,
        Arm::K4EightB,
        Arm::K4EightBBatched,
    ] {
        build_block_tables(arm, &blk, &mut sc);
        let (n, eb) = decode_block_syms(arm, &blk, &bit_buf, &mut sc);
        assert_eq!(n, blk.n_syms, "synthetic len mismatch on {}", arm.name());
        assert_eq!(
            eb,
            blk.end_bit,
            "synthetic end_bit mismatch on {}",
            arm.name()
        );
        assert_eq!(
            &sc.syms[..n],
            &blk.expected_syms[..],
            "synthetic syms mismatch on {}",
            arm.name()
        );
    }
}

fn stats(v: &[f64]) -> (f64, f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    (s[0], s[s.len() / 2], s[s.len() - 1])
}

/// Count exact LUT lookups and count distribution for K=2, K=3, K=4 on all full blocks (untimed).
fn count_lookups(blocks: &[ParsedBlock], data: &[u8], sc: &mut Scratch) {
    let mut c2 = [0u64; 3];
    let mut c3 = [0u64; 4];
    let mut c4 = [0u64; 5];
    let (mut tail_k2_49, mut tail_k3_48, mut tail_k3_49, mut groups_n) = (0u64, 0u64, 0u64, 0u64);
    for blk in blocks {
        if blk.n_syms < 1000 {
            continue;
        }
        build_block_tables(Arm::K2Shipped, blk, sc);
        build_block_tables(Arm::K3FourB, blk, sc);
        build_block_tables(Arm::K4EightB, blk, sc);

        // K=2
        let mut r = Reg::at(data, blk.start_bit);
        'b2: for &sel in &blk.selectors {
            groups_n += 1;
            let t = &sc.k2[sel as usize];
            let mut k = 0usize;
            while k <= GROUP - 2 {
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                c2[c] += 1;
                if c != 0 {
                    r.skip_short_shl(e);
                    k += c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    k += 1;
                    if s == t.eob {
                        break 'b2;
                    }
                }
            }
            if k == GROUP - 1 {
                tail_k2_49 += 1;
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K2_CNT_SH) as usize;
                c2[usize::from(c != 0)] += 1;
                let s = t.decode_one_slice(&mut r, data);
                if s == t.eob {
                    break 'b2;
                }
            }
        }

        // K=3
        let mut r = Reg::at(data, blk.start_bit);
        'b3: for &sel in &blk.selectors {
            let t = &sc.k3[sel as usize];
            let mut k = 0usize;
            while k <= GROUP - 3 {
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                c3[c] += 1;
                if c != 0 {
                    r.skip_short_shl(e);
                    k += c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    k += 1;
                    if s == t.eob {
                        break 'b3;
                    }
                }
            }
            if k == GROUP - 2 {
                tail_k3_48 += 1;
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                if c >= 2 {
                    c3[2] += 1;
                    let s1 = ((e >> K3_SYM1) & 0xff) as usize;
                    let s2 = ((e >> K3_SYM2) & 0xff) as usize;
                    r.skip_short((t.sym_len[s1] + t.sym_len[s2]) as u32);
                    continue;
                } else if c == 1 {
                    c3[1] += 1;
                    r.skip_short_shl(e);
                    k = GROUP - 1;
                } else {
                    c3[0] += 1;
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    if s == t.eob {
                        break 'b3;
                    }
                    k = GROUP - 1;
                }
            }
            if k == GROUP - 1 {
                tail_k3_49 += 1;
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K3_CNT_SH) as usize;
                if c != 0 {
                    c3[1] += 1;
                    let s1 = ((e >> K3_SYM1) & 0xff) as usize;
                    r.skip_short(t.sym_len[s1] as u32);
                } else {
                    c3[0] += 1;
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    if s == t.eob {
                        break 'b3;
                    }
                }
            }
        }

        // K=4
        let mut r = Reg::at(data, blk.start_bit);
        'b4: for &sel in &blk.selectors {
            let t = &sc.k4[sel as usize];
            let mut k = 0usize;
            while k < GROUP {
                let e = t.lut[r.peek(data, LUT_BITS) as usize];
                let c = (e >> K4_CNT_SH) as usize;
                if c != 0 {
                    let rem = GROUP - k;
                    let take = c.min(rem);
                    c4[take] += 1;
                    if c <= rem {
                        r.skip_short_shl(e as u32);
                    } else {
                        let s1 = ((e >> K4_SYM1) & 0xff) as usize;
                        let mut bits = t.sym_len[s1] as u32;
                        if rem >= 2 {
                            let s2 = ((e >> K4_SYM2) & 0xff) as usize;
                            bits += t.sym_len[s2] as u32;
                        }
                        if rem == 3 {
                            let s3 = ((e >> K4_SYM3) & 0xff) as usize;
                            bits += t.sym_len[s3] as u32;
                        }
                        r.skip_short(bits);
                    }
                    k += take;
                } else {
                    c4[0] += 1;
                    let (s, after) = t.decode_entry_slow(e as u32, r, data);
                    r = after;
                    k += 1;
                    if s == t.eob {
                        break 'b4;
                    }
                }
            }
        }
    }
    let full_syms: u64 = blocks
        .iter()
        .filter(|b| b.n_syms >= 1000)
        .map(|b| b.n_syms as u64)
        .sum();
    let tot2: u64 = c2.iter().sum();
    let tot3: u64 = c3.iter().sum();
    let tot4: u64 = c4.iter().sum();
    println!("\n=== LOOKUP CENSUS ON 233 FULL BLOCKS ({groups_n} groups, {full_syms} syms) ===");
    println!(
        "K=2: total={} ({:.4} root/sym; +{:.4} slow={:.4}) | c0={} c1={} c2={} | tail k=49: {:.1}% of groups",
        tot2,
        tot2 as f64 / full_syms as f64,
        c2[0] as f64 / full_syms as f64,
        (tot2 + c2[0]) as f64 / full_syms as f64,
        c2[0],
        c2[1],
        c2[2],
        100.0 * tail_k2_49 as f64 / groups_n as f64
    );
    println!(
        "K=3: total={} ({:.4} root/sym, {:+.2}% vs K2) | c0={} c1={} c2={} c3={} | tail k=48: {:.1}%, k=49: {:.1}%",
        tot3,
        tot3 as f64 / full_syms as f64,
        (tot3 as f64 - tot2 as f64) / tot2 as f64 * 100.0,
        c3[0],
        c3[1],
        c3[2],
        c3[3],
        100.0 * tail_k3_48 as f64 / groups_n as f64,
        100.0 * tail_k3_49 as f64 / groups_n as f64
    );
    println!(
        "K=4: total={} ({:.4} root/sym, {:+.2}% vs K2) | c0={} c1={} c2={} c3={} c4={}",
        tot4,
        tot4 as f64 / full_syms as f64,
        (tot4 as f64 - tot2 as f64) / tot2 as f64 * 100.0,
        c4[0],
        c4[1],
        c4[2],
        c4[3],
        c4[4]
    );
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: huff_multi_symbol_probe FILE.bz2 [REPEATS]");
    let repeats: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);

    verify_synthetic_edge_cases();

    let data = std::fs::read(&path).expect("read bz2");
    let blocks = parse_blocks(&data);

    let total_syms: usize = blocks.iter().map(|b| b.n_syms).sum();
    let full_blocks: usize = blocks.iter().filter(|b| b.n_syms >= 1000).count();
    let full_syms: usize = blocks
        .iter()
        .filter(|b| b.n_syms >= 1000)
        .map(|b| b.n_syms)
        .sum();
    let runt_blocks = blocks.len() - full_blocks;
    let runt_syms = total_syms - full_syms;
    let max_runt_sel = blocks
        .iter()
        .filter(|b| b.n_syms < 1000)
        .map(|b| b.selectors.len())
        .max()
        .unwrap_or(0);
    let min_full_sel = blocks
        .iter()
        .filter(|b| b.n_syms >= 1000)
        .map(|b| b.selectors.len())
        .min()
        .unwrap_or(0);
    let sym256_count: usize = blocks
        .iter()
        .flat_map(|b| b.expected_syms.iter())
        .filter(|&&s| s == 256)
        .count();
    let total_tables_built: usize = blocks
        .iter()
        .map(|b| (b.used_groups & ((1u8 << b.n_groups) - 1)).count_ones() as usize)
        .sum();
    let runt_tables_built: usize = blocks
        .iter()
        .filter(|b| b.n_syms < 1000)
        .map(|b| (b.used_groups & ((1u8 << b.n_groups) - 1)).count_ones() as usize)
        .sum();

    println!(
        "corpus: {} | blocks={} (full={} syms={}, runts={} syms={}) | total_syms={} | sym256={}",
        path,
        blocks.len(),
        full_blocks,
        full_syms,
        runt_blocks,
        runt_syms,
        total_syms,
        sym256_count
    );
    println!(
        "selectors: max_runt_sel={} min_full_sel={} gate={} | tables_built={} (full={}, runts={})",
        max_runt_sel,
        min_full_sel,
        GATE_SEL,
        total_tables_built,
        total_tables_built - runt_tables_built,
        runt_tables_built
    );
    println!(
        "table sizes: HuffK2.lut={}B HuffK3.lut={}B HuffK4.lut={}B | struct sizes: K2={}B K3={}B K4={}B",
        std::mem::size_of::<[u32; LUT_LEN]>(),
        std::mem::size_of::<[u32; LUT_LEN]>(),
        std::mem::size_of::<[u64; LUT_LEN]>(),
        std::mem::size_of::<HuffK2>(),
        std::mem::size_of::<HuffK3>(),
        std::mem::size_of::<HuffK4>(),
    );

    let arms = [
        Arm::K2Shipped,
        Arm::K2Gated,
        Arm::K2Batched,
        Arm::K2Slice,
        Arm::K3FourB,
        Arm::K3FourBGated,
        Arm::K3FourBBatched5,
        Arm::K3FourBBatched4,
        Arm::K4EightB,
        Arm::K4EightBGated,
        Arm::K4EightBBatched,
    ];

    let mut sc = Scratch::new();

    // 1. Exact verification on every single block of the corpus
    for &arm in &arms {
        for (bi, blk) in blocks.iter().enumerate() {
            build_block_tables(arm, blk, &mut sc);
            let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc);
            assert_eq!(
                n,
                blk.n_syms,
                "block {bi} sym count mismatch on {}",
                arm.name()
            );
            assert_eq!(
                eb,
                blk.end_bit,
                "block {bi} end_bit mismatch on {}",
                arm.name()
            );
            assert_eq!(
                &sc.syms[..n],
                &blk.expected_syms[..],
                "block {bi} symbol mismatch on {}",
                arm.name()
            );
        }
    }
    println!(
        "verify: PASS (all {} arms byte-identical on all {} blocks, {} symbols, + synthetic edge cases)",
        arms.len(),
        blocks.len(),
        total_syms
    );

    count_lookups(&blocks, &data, &mut sc);

    // Warmup passes across primary arms to stabilize CPU clock/caches
    for _ in 0..3 {
        for &arm in &[Arm::K2Shipped, Arm::K3FourBGated, Arm::K4EightBGated] {
            let mut chk = 0usize;
            for blk in &blocks {
                build_block_tables(arm, blk, &mut sc);
                let (n, _) = decode_block_syms(arm, blk, &data, &mut sc);
                chk ^= n;
            }
            black_box(chk);
        }
    }

    // 2. Primary A3 timing (single Instant::now() per full 431-block pass, interleaved repeats)
    let mut a3_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    for rep in 0..repeats {
        for off in 0..arms.len() {
            let ai = (rep + off) % arms.len();
            let arm = arms[ai];
            let t0 = Instant::now();
            let mut chk = 0u64;
            for blk in &blocks {
                build_block_tables(arm, blk, &mut sc);
                let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc);
                chk ^= (n as u64) ^ eb ^ (sc.syms[n - 1] as u64);
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            black_box(chk);
            a3_ms[ai].push(ms);
        }
    }

    println!("\n=== PRIMARY A3 (decode + tables, all 431 blocks, {repeats} interleaved repeats, single timer) ===");
    let (base_min, base_med, base_max) = stats(&a3_ms[0]);
    println!(
        "{:<16} {:>9} {:>9} {:>9} {:>9} {:>10} {:>10}",
        "arm", "min_ms", "med_ms", "max_ms", "vs_k2_%", "overlap?", "ns/sym"
    );
    for (ai, &arm) in arms.iter().enumerate() {
        let (mn, med, mx) = stats(&a3_ms[ai]);
        let delta_pct = (med - base_med) / base_med * 100.0;
        let overlap = if ai == 0 {
            "-"
        } else if mx < base_min || mn > base_max {
            "NO"
        } else {
            "YES"
        };
        let ns_sym = med * 1e6 / total_syms as f64;
        println!(
            "{:<16} {:>9.3} {:>9.3} {:>9.3} {:>+8.2}% {:>10} {:>10.3}",
            arm.name(),
            mn,
            med,
            mx,
            delta_pct,
            overlap,
            ns_sym
        );
    }

    println!("\nraw A3 ms by repeat:");
    for (ai, &arm) in arms.iter().enumerate() {
        print!("  {:<16}:", arm.name());
        for &v in &a3_ms[ai] {
            print!(" {:7.3}", v);
        }
        println!();
    }

    // 3. Per-stage breakdown (tables vs decode, full 233 blocks vs 198 runts, 7 interleaved repeats)
    let mut tbl_all = vec![Vec::with_capacity(repeats); arms.len()];
    let mut dec_all = vec![Vec::with_capacity(repeats); arms.len()];
    let mut tbl_full = vec![Vec::with_capacity(repeats); arms.len()];
    let mut dec_full = vec![Vec::with_capacity(repeats); arms.len()];
    let mut tbl_runt = vec![Vec::with_capacity(repeats); arms.len()];
    let mut dec_runt = vec![Vec::with_capacity(repeats); arms.len()];

    for rep in 0..repeats {
        for off in 0..arms.len() {
            let ai = (rep + off) % arms.len();
            let arm = arms[ai];
            let (mut tf, mut df, mut tr, mut dr) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let mut chk = 0u64;
            for blk in &blocks {
                let t0 = Instant::now();
                build_block_tables(arm, blk, &mut sc);
                let dt_tbl = t0.elapsed().as_secs_f64() * 1e3;
                let t1 = Instant::now();
                let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc);
                let dt_dec = t1.elapsed().as_secs_f64() * 1e3;
                chk ^= (n as u64) ^ eb;
                if blk.n_syms >= 1000 {
                    tf += dt_tbl;
                    df += dt_dec;
                } else {
                    tr += dt_tbl;
                    dr += dt_dec;
                }
            }
            black_box(chk);
            tbl_all[ai].push(tf + tr);
            dec_all[ai].push(df + dr);
            tbl_full[ai].push(tf);
            dec_full[ai].push(df);
            tbl_runt[ai].push(tr);
            dec_runt[ai].push(dr);
        }
    }

    println!("\n=== STAGE BREAKDOWN (medians over {repeats} interleaved repeats, ms) ===");
    println!(
        "{:<16} {:>8} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9} {:>9}",
        "arm",
        "tbl_all",
        "dec_all",
        "sum_all",
        "tbl_rnt",
        "dec_rnt",
        "full_ns/s",
        "tbl_us/b",
        "dec_vs_k2"
    );
    let (_, base_dec_med, _) = stats(&dec_all[0]);
    for (ai, &arm) in arms.iter().enumerate() {
        let (_, ta, _) = stats(&tbl_all[ai]);
        let (_, da, _) = stats(&dec_all[ai]);
        let (_, df, _) = stats(&dec_full[ai]);
        let (_, tr, _) = stats(&tbl_runt[ai]);
        let (_, dr, _) = stats(&dec_runt[ai]);
        let full_ns_sym = df * 1e6 / full_syms as f64;
        let tbl_us_blk = ta * 1e3 / blocks.len() as f64;
        let dec_delta = (da - base_dec_med) / base_dec_med * 100.0;
        println!(
            "{:<16} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>9.3} {:>9.2} {:>+8.2}%",
            arm.name(),
            ta,
            da,
            ta + da,
            tr,
            dr,
            full_ns_sym,
            tbl_us_blk,
            dec_delta
        );
    }

    // 4. Overhead diagnostic: pure LUT+shift+refill chain (no g_out store) on full blocks
    let mut ns_k2 = Vec::with_capacity(repeats);
    let mut ns_k3 = Vec::with_capacity(repeats);
    let mut full_k2_dec = Vec::with_capacity(repeats);
    let mut full_k3_dec = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let t0 = Instant::now();
        let a2 = run_nostore_k2(&blocks, &data, &mut sc);
        ns_k2.push(t0.elapsed().as_secs_f64() * 1e3);
        black_box(a2);

        let t1 = Instant::now();
        let a3 = run_nostore_k3(&blocks, &data, &mut sc);
        ns_k3.push(t1.elapsed().as_secs_f64() * 1e3);
        black_box(a3);

        let t2 = Instant::now();
        let mut c2 = 0u64;
        for blk in &blocks {
            if blk.n_syms < 1000 {
                continue;
            }
            build_block_tables(Arm::K2Shipped, blk, &mut sc);
            let (n, eb) = decode_block_syms(Arm::K2Shipped, blk, &data, &mut sc);
            c2 ^= (n as u64) ^ eb;
        }
        full_k2_dec.push(t2.elapsed().as_secs_f64() * 1e3);
        black_box(c2);

        let t3 = Instant::now();
        let mut c3 = 0u64;
        for blk in &blocks {
            if blk.n_syms < 1000 {
                continue;
            }
            build_block_tables(Arm::K3FourB, blk, &mut sc);
            let (n, eb) = decode_block_syms(Arm::K3FourB, blk, &data, &mut sc);
            c3 ^= (n as u64) ^ eb;
        }
        full_k3_dec.push(t3.elapsed().as_secs_f64() * 1e3);
        black_box(c3);
    }
    let (_, m_ns_k2, _) = stats(&ns_k2);
    let (_, m_ns_k3, _) = stats(&ns_k3);
    let (_, m_f_k2, _) = stats(&full_k2_dec);
    let (_, m_f_k3, _) = stats(&full_k3_dec);
    let (_, tf_k2, _) = stats(&tbl_full[0]);
    let (_, tf_k3, _) = stats(&tbl_full[4]);
    let k2_dec_ns = (m_f_k2 - tf_k2) * 1e6 / full_syms as f64;
    let k2_nostore_ns = (m_ns_k2 - tf_k2) * 1e6 / full_syms as f64;
    let k3_dec_ns = (m_f_k3 - tf_k3) * 1e6 / full_syms as f64;
    let k3_nostore_ns = (m_ns_k3 - tf_k3) * 1e6 / full_syms as f64;
    println!("\n=== FULL-BLOCK OVERHEAD DIAGNOSTIC (233 full blocks, {full_syms} syms) ===");
    println!(
        "k2 full decode: {:.3} ms ({:.3} ns/sym) | nostore: {:.3} ms ({:.3} ns/sym) | store+extract share: {:.3} ns/sym ({:.1}%)",
        m_f_k2 - tf_k2,
        k2_dec_ns,
        m_ns_k2 - tf_k2,
        k2_nostore_ns,
        k2_dec_ns - k2_nostore_ns,
        (k2_dec_ns - k2_nostore_ns) / k2_dec_ns * 100.0
    );
    println!(
        "k3 full decode: {:.3} ms ({:.3} ns/sym) | nostore: {:.3} ms ({:.3} ns/sym) | store+extract share: {:.3} ns/sym ({:.1}%)",
        m_f_k3 - tf_k3,
        k3_dec_ns,
        m_ns_k3 - tf_k3,
        k3_nostore_ns,
        k3_dec_ns - k3_nostore_ns,
        (k3_dec_ns - k3_nostore_ns) / k3_dec_ns * 100.0
    );
}
