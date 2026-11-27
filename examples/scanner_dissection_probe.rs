#![forbid(unsafe_code)]
//! (research only, `explore`) Cold end-to-end dissection of mibz2's serial scanner (`S`:
//! `src/block.rs` + `src/huff.rs` + `src/bits.rs`) on `corpora/wikidata.bz2`.
//!
//!     cargo run --release --example scanner_dissection_probe -- corpora/wikidata.bz2 [REPEATS]
//!
//! Measures:
//! 1. Exact data census (all 431 blocks, split into 233 full vs 198 runts):
//!    - selector MTF distribution (`j = 0..5`) and consecutive identical-selector run lengths `M`;
//!    - code-length delta distribution and built vs skipped tables;
//!    - `syms.resize(max_out, 0)` zero-fill volume under sequential reuse and 8-worker LIFO pool;
//!    - A3 lookup and tail statistics under shipped `GROUP = 50` vs coalesced `50 * M` runs.
//! 2. Full end-to-end `read_item_with_scratch` stage breakdown (`A2a+b`, `A2c`, `A2d` build,
//!    `A2e` `syms.resize(max_out, 0)`, `A3` symbol decode) vs shipped `mibz2::stream::Scanner`.
//! 3. Interleaved multi-repeat micro-mechanism ablations (all verified symbol-for-symbol and
//!    bit-position exact against `mibz2::stream::Scanner` on all 431 blocks + synthetic `alpha=258`):
//!    - eliminating `syms.resize(max_out, 0)` after runt truncations;
//!    - fast `A2` (batched `j == 0` selector MTF + runt-gated table build);
//!    - branchless masked `refill_win` (`pos & 255` in `[u8; 264]`);
//!    - consecutive same-selector run coalescing (`M <= 8` in `[u8; 1032]` and uncapped direct slice);
//!    - byte-aligned 8-bit `SYM1`/`SYM2` LUT layout (`SYM256` slow tag);
//!    - inlined `SUB_PTR` 2nd-level lookup;
//!    - coalesced 4-byte packed `K = 3` (testing whether coalescing rescues `K = 3` from group tails).

use mibz2::stream::{Event, Scanner, SliceSource};
use std::hint::black_box;
use std::time::Instant;

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23;
const MAX_SELECTORS: usize = 18_002;
const MAX_SYMS: usize = MAX_SELECTORS * GROUP;
const GROUP: usize = 50;
const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;
const STEPS: usize = 5;
const GATE_SEL: usize = 64;
const MAX_COAL_GROUPS: usize = 8; // 8 * 50 = 400 syms <= 1000 bytes + 24 slack <= 1032
const COAL_WIN: usize = 1032; // pos & 1023 => pos + 8 <= 1031 < 1032

// Shipped K=2 LUT bit layout (mirror of src/huff.rs)
const SUB_PTR: u32 = 1 << 27;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const LEN1: u32 = 6;
const SYM1: u32 = 11;
const SYM2: u32 = 20;
const CNT_SH: u32 = 30;

// Byte-aligned 8-bit K=2 LUT layout:
// - bits 30..31: cnt (1 or 2 for fast; 0 for slow tags)
// - bits 24..28: len1 (1..=10)
// - bits 16..23: sym2 (0..=255)
// - bits  8..15: sym1 (0..=255)
// - bits  0..5 : total_bits (1..=10, bits 4..5 == 0 for wrapping_shl)
// When cnt == 0:
// - bit 29: EOB_FAST (low 6 bits = zn)
// - bit 28: BAD
// - bit 27: SUB_PTR (d_w in 16..19, sub_off in 0..15)
// - bit 26: SYM256 (low 6 bits = zn)
const U8_SYM256: u32 = 1 << 26;
const U8_SYM1: u32 = 8;
const U8_SYM2: u32 = 16;
const U8_LEN1: u32 = 24;

// 4-byte packed K=3 LUT layout (same as huff_multi_symbol_probe):
// - bits 30..31: cnt (1, 2, or 3 for fast; 0 for slow tags)
// - bits 22..29: sym3 (0..=255)
// - bits 14..21: sym2 (0..=255)
// - bits  6..13: sym1 (0..=255)
// - bits  0..5 : total_bits (1..=10, bits 4..5 == 0)
const K3_SYM1: u32 = 6;
const K3_SYM2: u32 = 14;
const K3_SYM3: u32 = 22;

#[derive(Clone, Copy)]
struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    #[inline(always)]
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

    /// Branchless refill inside `[u8; 264]`: `pos & 255 <= 255` => `p + 8 <= 263 < 264`.
    #[inline(always)]
    fn refill_win_masked(&mut self, win: &[u8; 264]) {
        let p = self.pos & 255;
        let w: [u8; 8] = win[p..p + 8].try_into().unwrap();
        let v = u64::from_be_bytes(w);
        self.buf |= v >> self.live;
        let take = (63 - self.live) >> 3;
        self.pos += take as usize;
        self.live += take * 8;
    }

    /// Branchless refill inside `[u8; 1032]`: `pos & 1023 <= 1023` => `p + 8 <= 1031 < 1032`.
    #[inline(always)]
    fn refill_coal_masked(&mut self, win: &[u8; COAL_WIN]) {
        let p = self.pos & 1023;
        let w: [u8; 8] = win[p..p + 8].try_into().unwrap();
        let v = u64::from_be_bytes(w);
        self.buf |= v >> self.live;
        let take = (63 - self.live) >> 3;
        self.pos += take as usize;
        self.live += take * 8;
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

    /// Low-byte shift helper for byte-aligned LUT entries (bits 0..7 = total_bits in 1..=10,
    /// bits 4..7 == 0): keeps `(e as u8)` independent of `wrapping_shl(e)` so LLVM does not
    /// put `and $0x3f` on the `buf` critical path.
    #[inline(always)]
    fn skip_short_shl_u8(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= (e as u8) as u32;
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

// --- Shipped Huff (faithful mirror of src/huff.rs) ---

#[derive(Clone)]
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
        let mut base = [0i32; MAX_CODE_LEN];
        let mut pos = [0u16; MAX_CODE_LEN];
        let mut acc = 0i32;
        for i in 1..MAX_CODE_LEN {
            base[i] = acc;
            pos[i] = acc as u16;
            acc += counts[i];
        }
        for (j, &l) in lens.iter().enumerate() {
            let lu = l as usize;
            let idx = pos[lu] as usize;
            self.perm[idx] = j as u16;
            pos[lu] = (idx + 1) as u16;
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_l..=max_l {
            vec += counts[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_l + 1..=max_l {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        self.limit = limit;
        self.base = base;
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
                    Some(s) if s == eob => EOB_FAST | zn,
                    Some(s) => (1u32 << CNT_SH) | ((s as u32) << SYM1) | (zn << LEN1) | zn,
                    None => BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }

        if filled_to < LUT_LEN {
            if max_l <= LUT_BITS as usize {
                self.lut[filled_to..].fill(BAD);
            } else {
                self.lut[filled_to..].fill(0);
                let mut ranges = [(0u32, 0i64, -1i64); 10];
                let mut d_arr = [0u8; LUT_LEN];
                let mut max_root_w = filled_to;
                let start_zn = (LUT_BITS + 1).max(min_len);
                let mut a_hi = a;
                for zn in start_zn..=(max_l as u32) {
                    let top = (1i64 << zn) - 1;
                    let hi = (self.limit[zn as usize] as i64).min(top);
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
                        self.lut[filled_to + w] = SUB_PTR | (d_w << 16) | (sub_used as u32);
                        sub_used += 1usize << d_w;
                    }
                }
                if sub_used > 0 {
                    self.sub_lut[..sub_used].fill(SUB_BAD_U16);
                    for &(zn, lo_c, hi_c) in &ranges {
                        if zn == 0 || lo_c > hi_c {
                            continue;
                        }
                        let sh = zn - LUT_BITS;
                        for p in lo_c..=hi_c {
                            let e_root = self.lut[(p >> sh) as usize];
                            if (e_root & SUB_PTR) == 0 {
                                continue;
                            }
                            let d_w = (e_root >> 16) & 15;
                            let sub_off = (e_root & 0xffff) as usize;
                            let idx_sym = (p as i32) - self.base[zn as usize];
                            let se = if (0..self.alpha).contains(&idx_sym) {
                                (self.perm[idx_sym as usize] << 6) | (zn as u16)
                            } else {
                                SUB_BAD_U16
                            };
                            let pad = d_w - sh;
                            let local = ((p as usize) & ((1usize << sh) - 1)) << pad;
                            self.sub_lut[sub_off + local..sub_off + local + (1usize << pad)]
                                .fill(se);
                        }
                    }
                }
            }
        }

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
                if (e0 >> CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> SYM1) & 0x1ff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << SYM1;
                    for u in 0..u_lim {
                        self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let w = off_p + u;
                        let e = self.lut[w];
                        let e2 = self.lut[(u << zn) & mask];
                        let l2 = ((e2 >> LEN1) & 31) as usize;
                        if (e2 >> CNT_SH) == 0 || (zn as usize) + l2 > LUT_BITS as usize {
                            continue;
                        }
                        self.lut[w] = (2u32 << CNT_SH)
                            | (((e2 >> SYM1) & 0x1ff) << SYM2)
                            | (e & (0x1ff << SYM1))
                            | (zn << LEN1)
                            | (zn + l2 as u32);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid canonical code")
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
        if (e & SUB_PTR) != 0 {
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
        } else if (e & EOB_FAST) != 0 {
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
    fn decode_entry_slow_coal(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_coal_masked(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_coal_masked(win);
                return (se >> 6, r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_coal_masked(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
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
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
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
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return ((e >> SYM1) & 0x1ff) as u16;
        }
        let (s, after) = self.decode_entry_slow_win(e, *r, win, full_data, base_pos);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_one_coal(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> u16 {
        if r.live < LUT_BITS {
            r.refill_coal_masked(win);
        }
        let e = self.lut[r.peek_live(LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return ((e >> SYM1) & 0x1ff) as u16;
        }
        let (s, after) = self.decode_entry_slow_coal(e, *r, win, full_data, base_pos);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_one_slice(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return ((e >> SYM1) & 0x1ff) as u16;
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// Faithful mirror of `src/huff.rs::decode_group_win`.
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
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
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

    /// Branchless masked refill version of `decode_group_win` (same 50-sym groups).
    #[inline(always)]
    fn decode_group_win_masked(
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
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
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
        r.refill_win_masked(g_in);
        loop {
            for _ in 0..STEPS {
                step_win!();
            }
            if k > GROUP - 2 * STEPS {
                break;
            }
            r.refill_win_masked(g_in);
        }
        if k <= GROUP - 2 {
            r.refill_win_masked(g_in);
            for _ in 0..4 {
                if k > GROUP - 2 {
                    break;
                }
                step_win!();
            }
            if k <= GROUP - 2 {
                r.refill_win_masked(g_in);
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
    fn decode_group_slice(&self, r: &mut Reg, data: &[u8], g_out: &mut [u16; 66]) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_slice {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let idx = k & 63;
                    let dst = &mut g_out[idx..idx + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
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
        (GROUP, false)
    }

    /// Coalesced-run decoder (`target = 50 * m`, `1 <= m <= 8`, `out.len() >= target + 16`).
    /// Uses `[u8; 1032]` branchless masked window (`pos & 1023`).
    #[inline(always)]
    fn decode_run_coal(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        data: &[u8],
        base_pos: usize,
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        macro_rules! step_coal {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_coal(e, *r, win, data, base_pos);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_coal_masked(win);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_coal!();
            }
            r.refill_coal_masked(win);
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_coal!();
            }
            if k <= tail_stop {
                r.refill_coal_masked(win);
                while k <= tail_stop {
                    step_coal!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_coal(r, win, data, base_pos);
            out_slice[target - 1] = s;
            if s == eob {
                r.pos += base_pos;
                return (target, true);
            }
        }
        r.pos += base_pos;
        (target, false)
    }

    /// Uncapped coalesced-run decoder directly over `data` (no window copy/reslice, skips
    /// entry refill if `r.live >= 50`).
    #[inline(always)]
    fn decode_run_direct(
        &self,
        r: &mut Reg,
        data: &[u8],
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        macro_rules! step_dir {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 2];
                    dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                    dst[1] = ((e >> SYM2) & 0x1ff) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        return (k, true);
                    }
                }
            }};
        }
        r.refill(data);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_dir!();
            }
            r.refill(data);
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_dir!();
            }
            if k <= tail_stop {
                r.refill(data);
                while k <= tail_stop {
                    step_dir!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_slice(r, data);
            out_slice[target - 1] = s;
            if s == eob {
                return (target, true);
            }
        }
        (target, false)
    }
}

// --- Byte-aligned 8-bit K=2 LUT (`SYM1` at bits 8..15, `SYM2` at bits 16..23) ---

#[derive(Clone)]
struct HuffK2U8 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffK2U8 {
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

impl HuffK2U8 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], build_pairs: bool) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
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
        let mut base = [0i32; MAX_CODE_LEN];
        let mut pos = [0u16; MAX_CODE_LEN];
        let mut acc = 0i32;
        for i in 1..MAX_CODE_LEN {
            base[i] = acc;
            pos[i] = acc as u16;
            acc += counts[i];
        }
        for (j, &l) in lens.iter().enumerate() {
            let lu = l as usize;
            let idx = pos[lu] as usize;
            self.perm[idx] = j as u16;
            pos[lu] = (idx + 1) as u16;
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_l..=max_l {
            vec += counts[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_l + 1..=max_l {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        self.limit = limit;
        self.base = base;
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
                    Some(s) if s == eob => EOB_FAST | zn,
                    Some(256) => U8_SYM256 | zn,
                    Some(s) => (1u32 << CNT_SH) | (zn << U8_LEN1) | ((s as u32) << U8_SYM1) | zn,
                    None => BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }

        if filled_to < LUT_LEN {
            if max_l <= LUT_BITS as usize {
                self.lut[filled_to..].fill(BAD);
            } else {
                self.lut[filled_to..].fill(0);
                let mut ranges = [(0u32, 0i64, -1i64); 10];
                let mut d_arr = [0u8; LUT_LEN];
                let mut max_root_w = filled_to;
                let start_zn = (LUT_BITS + 1).max(min_len);
                let mut a_hi = a;
                for zn in start_zn..=(max_l as u32) {
                    let top = (1i64 << zn) - 1;
                    let hi = (self.limit[zn as usize] as i64).min(top);
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
                        self.lut[filled_to + w] = SUB_PTR | (d_w << 16) | (sub_used as u32);
                        sub_used += 1usize << d_w;
                    }
                }
                if sub_used > 0 {
                    self.sub_lut[..sub_used].fill(SUB_BAD_U16);
                    for &(zn, lo_c, hi_c) in &ranges {
                        if zn == 0 || lo_c > hi_c {
                            continue;
                        }
                        let sh = zn - LUT_BITS;
                        for p in lo_c..=hi_c {
                            let e_root = self.lut[(p >> sh) as usize];
                            if (e_root & SUB_PTR) == 0 {
                                continue;
                            }
                            let d_w = (e_root >> 16) & 15;
                            let sub_off = (e_root & 0xffff) as usize;
                            let idx_sym = (p as i32) - self.base[zn as usize];
                            let se = if (0..self.alpha).contains(&idx_sym) {
                                (self.perm[idx_sym as usize] << 6) | (zn as u16)
                            } else {
                                SUB_BAD_U16
                            };
                            let pad = d_w - sh;
                            let local = ((p as usize) & ((1usize << sh) - 1)) << pad;
                            self.sub_lut[sub_off + local..sub_off + local + (1usize << pad)]
                                .fill(se);
                        }
                    }
                }
            }
        }

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
                if (e0 >> CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> U8_SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << U8_SYM1;
                    for u in 0..u_lim {
                        self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let w = off_p + u;
                        let e = self.lut[w];
                        let e2 = self.lut[(u << zn) & mask];
                        let l2 = ((e2 >> U8_LEN1) & 31) as usize;
                        if (e2 >> CNT_SH) == 0 || (zn as usize) + l2 > LUT_BITS as usize {
                            continue;
                        }
                        self.lut[w] = (2u32 << CNT_SH)
                            | (zn << U8_LEN1)
                            | (((e2 >> U8_SYM1) & 0xff) << U8_SYM2)
                            | (e & (0xff << U8_SYM1))
                            | (zn + l2 as u32);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid canonical code")
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_coal(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_coal_masked(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_coal_masked(win);
                return (se >> 6, r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & U8_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_coal_masked(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_nosub(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & U8_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_coal_masked(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
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
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & U8_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[inline(always)]
    fn decode_one_coal(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> u16 {
        if r.live < LUT_BITS {
            r.refill_coal_masked(win);
        }
        let e = self.lut[r.peek_live(LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> U8_LEN1) & 31);
            return ((e >> U8_SYM1) as u8) as u16;
        }
        let (s, after) = self.decode_entry_slow_coal(e, *r, win, full_data, base_pos);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_one_slice(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> U8_LEN1) & 31);
            return ((e >> U8_SYM1) as u8) as u16;
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// Coalesced K=2 with byte-aligned 8-bit SYM1/SYM2 (`((e >> 8) as u8) as u16`, no `& 0x1ff`)
    /// and zero per-step bounds checks via fixed `out_win: &mut [u16; 514]` (`k & 511`).
    #[inline(always)]
    fn decode_run_coal_u8(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        data: &[u8],
        base_pos: usize,
        out_win: &mut [u16; 514],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        macro_rules! step_u8 {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_u8(e);
                    let idx = k & 511;
                    let dst = &mut out_win[idx..idx + 2];
                    dst[0] = ((e >> U8_SYM1) as u8) as u16;
                    dst[1] = ((e >> U8_SYM2) as u8) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_coal(e, *r, win, data, base_pos);
                    *r = after;
                    out_win[k & 511] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_coal_masked(win);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_u8!();
            }
            r.refill_coal_masked(win);
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_u8!();
            }
            if k <= tail_stop {
                r.refill_coal_masked(win);
                while k <= tail_stop {
                    step_u8!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_coal(r, win, data, base_pos);
            out_win[(target - 1) & 511] = s;
            if s == eob {
                r.pos += base_pos;
                return (target, true);
            }
        }
        r.pos += base_pos;
        (target, false)
    }

    /// Uncapped coalesced K=2 directly over `data` with byte-aligned 8-bit LUT and zero
    /// per-step bounds checks (`batch: &mut [u16; 18]` sliced once per 5-step batch, `bk & 15`).
    #[inline(always)]
    fn decode_run_direct_u8(
        &self,
        r: &mut Reg,
        data: &[u8],
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 18];
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        r.refill(data);
        while k <= main_stop {
            let batch: &mut [u16; 18] = (&mut out_slice[k..k + 18]).try_into().unwrap();
            let mut bk = 0usize;
            for _ in 0..STEPS {
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_u8(e);
                    let idx = bk & 15;
                    let dst = &mut batch[idx..idx + 2];
                    dst[0] = ((e >> U8_SYM1) as u8) as u16;
                    dst[1] = ((e >> U8_SYM2) as u8) as u16;
                    bk += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    batch[bk & 15] = s;
                    bk += 1;
                    if s == eob {
                        return (k + bk, true);
                    }
                }
            }
            k += bk;
            r.refill(data);
        }
        macro_rules! step_tail {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_u8(e);
                    let dst = &mut out_slice[k..k + 2];
                    dst[0] = ((e >> U8_SYM1) as u8) as u16;
                    dst[1] = ((e >> U8_SYM2) as u8) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        return (k, true);
                    }
                }
            }};
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_tail!();
            }
            if k <= tail_stop {
                r.refill(data);
                while k <= tail_stop {
                    step_tail!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_slice(r, data);
            out_slice[target - 1] = s;
            if s == eob {
                return (target, true);
            }
        }
        (target, false)
    }

    /// Coalesced K=2 + byte-aligned 8-bit LUT + inlined `SUB_PTR` lookup (only EOB/SYM256/slow
    /// out-of-line).
    #[inline(always)]
    fn decode_run_coal_inlinesub(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        data: &[u8],
        base_pos: usize,
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        macro_rules! step_is {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 2];
                    dst[0] = ((e >> U8_SYM1) as u8) as u16;
                    dst[1] = ((e >> U8_SYM2) as u8) as u16;
                    k += c;
                } else if (e & SUB_PTR) != 0 {
                    if r.live < 20 {
                        r.refill_coal_masked(win);
                    }
                    let sub_off = (e & 0xffff) as usize;
                    let d_w = (e >> 16) & 15;
                    let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
                    let se = self.sub_lut[sub_off + idx];
                    let s = if se != SUB_BAD_U16 {
                        r.skip_short((se & 31) as u32);
                        r.refill_coal_masked(win);
                        se >> 6
                    } else {
                        let (s, after) = self.decode_entry_slow_nosub(e, *r, win, data, base_pos);
                        *r = after;
                        s
                    };
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                } else {
                    let (s, after) = self.decode_entry_slow_nosub(e, *r, win, data, base_pos);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_coal_masked(win);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_is!();
            }
            r.refill_coal_masked(win);
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_is!();
            }
            if k <= tail_stop {
                r.refill_coal_masked(win);
                while k <= tail_stop {
                    step_is!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_coal(r, win, data, base_pos);
            out_slice[target - 1] = s;
            if s == eob {
                r.pos += base_pos;
                return (target, true);
            }
        }
        r.pos += base_pos;
        (target, false)
    }

    #[inline(always)]
    fn decode_run_slice_u8(
        &self,
        r: &mut Reg,
        data: &[u8],
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 2 * STEPS;
        let tail_stop = target - 2;
        macro_rules! step_sl {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 2];
                    dst[0] = ((e >> U8_SYM1) as u8) as u16;
                    dst[1] = ((e >> U8_SYM2) as u8) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        return (k, true);
                    }
                }
            }};
        }
        r.refill(data);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_sl!();
            }
            r.refill(data);
        }
        if k <= tail_stop {
            for _ in 0..4 {
                if k > tail_stop {
                    break;
                }
                step_sl!();
            }
            if k <= tail_stop {
                r.refill(data);
                while k <= tail_stop {
                    step_sl!();
                }
            }
        }
        if k == target - 1 {
            let s = self.decode_one_slice(r, data);
            out_slice[target - 1] = s;
            if s == eob {
                return (target, true);
            }
        }
        (target, false)
    }
}

// --- 4-byte packed K=3 LUT for coalesced rescue test ---

#[derive(Clone)]
struct HuffK3 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    sym_len: [u8; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffK3 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            sub_lut: [0; SUB_CAP],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            sym_len: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl HuffK3 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], build_multi: bool) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        self.sym_len[..alpha].copy_from_slice(lens);
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
        let mut base = [0i32; MAX_CODE_LEN];
        let mut pos = [0u16; MAX_CODE_LEN];
        let mut acc = 0i32;
        for i in 1..MAX_CODE_LEN {
            base[i] = acc;
            pos[i] = acc as u16;
            acc += counts[i];
        }
        for (j, &l) in lens.iter().enumerate() {
            let lu = l as usize;
            let idx = pos[lu] as usize;
            self.perm[idx] = j as u16;
            pos[lu] = (idx + 1) as u16;
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_l..=max_l {
            vec += counts[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_l + 1..=max_l {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        self.limit = limit;
        self.base = base;
        self.min_len = min_l as u32;
        self.alpha = alpha as i32;
        self.eob = eob;

        let min_len = min_l as u32;
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
                    Some(s) if s == eob => EOB_FAST | zn,
                    Some(256) => U8_SYM256 | zn,
                    Some(s) => (1u32 << CNT_SH) | ((s as u32) << K3_SYM1) | zn,
                    None => BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }

        if filled_to < LUT_LEN {
            if max_l <= LUT_BITS as usize {
                self.lut[filled_to..].fill(BAD);
            } else {
                self.lut[filled_to..].fill(0);
                let mut ranges = [(0u32, 0i64, -1i64); 10];
                let mut d_arr = [0u8; LUT_LEN];
                let mut max_root_w = filled_to;
                let start_zn = (LUT_BITS + 1).max(min_len);
                let mut a_hi = a;
                for zn in start_zn..=(max_l as u32) {
                    let top = (1i64 << zn) - 1;
                    let hi = (self.limit[zn as usize] as i64).min(top);
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
                        self.lut[filled_to + w] = SUB_PTR | (d_w << 16) | (sub_used as u32);
                        sub_used += 1usize << d_w;
                    }
                }
                if sub_used > 0 {
                    self.sub_lut[..sub_used].fill(SUB_BAD_U16);
                    for &(zn, lo_c, hi_c) in &ranges {
                        if zn == 0 || lo_c > hi_c {
                            continue;
                        }
                        let sh = zn - LUT_BITS;
                        for p in lo_c..=hi_c {
                            let e_root = self.lut[(p >> sh) as usize];
                            if (e_root & SUB_PTR) == 0 {
                                continue;
                            }
                            let d_w = (e_root >> 16) & 15;
                            let sub_off = (e_root & 0xffff) as usize;
                            let idx_sym = (p as i32) - self.base[zn as usize];
                            let se = if (0..self.alpha).contains(&idx_sym) {
                                (self.perm[idx_sym as usize] << 6) | (zn as u16)
                            } else {
                                SUB_BAD_U16
                            };
                            let pad = d_w - sh;
                            let local = ((p as usize) & ((1usize << sh) - 1)) << pad;
                            self.sub_lut[sub_off + local..sub_off + local + (1usize << pad)]
                                .fill(se);
                        }
                    }
                }
            }
        }

        if !build_multi {
            return;
        }
        // Snapshot 1-sym lut for K=2 and K=3 template-clone passes
        let base_lut = self.lut;
        let max_l1 = LUT_BITS.saturating_sub(min_len);
        let mask = LUT_LEN - 1;
        // Pass 2: K=2
        for zn in min_len..=max_l1 {
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
                let e0 = base_lut[off_p];
                if (e0 >> CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> K3_SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << K3_SYM1;
                    for u in 0..u_lim {
                        self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let e2 = base_lut[(u << zn) & mask];
                        let l2 = e2 & 63;
                        if (e2 >> CNT_SH) == 0 || zn + l2 > LUT_BITS {
                            continue;
                        }
                        let s2 = (e2 >> K3_SYM1) & 0xff;
                        self.lut[off_p + u] =
                            (2u32 << CNT_SH) | (s2 << K3_SYM2) | (s1 << K3_SYM1) | (zn + l2);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
        // Pass 3: K=3
        if 3 * min_len <= LUT_BITS {
            let max_l1_k3 = LUT_BITS - 2 * min_len;
            for zn in min_len..=max_l1_k3 {
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
                    let e0 = base_lut[off_p];
                    if (e0 >> CNT_SH) == 0 {
                        continue;
                    }
                    let s1 = (e0 >> K3_SYM1) & 0xff;
                    if let Some((off0, s0)) = tpl {
                        let delta = s1.wrapping_sub(s0) << K3_SYM1;
                        for u in 0..u_lim {
                            self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                        }
                    } else {
                        for u in 0..u_lim {
                            let w = off_p + u;
                            let cur = self.lut[w];
                            if (cur >> CNT_SH) != 2 {
                                continue;
                            }
                            let len12 = cur & 63;
                            let rem = LUT_BITS - len12;
                            if rem < min_len {
                                continue;
                            }
                            let e3 = base_lut[(w << len12) & mask];
                            let l3 = e3 & 63;
                            if (e3 >> CNT_SH) == 0 || len12 + l3 > LUT_BITS {
                                continue;
                            }
                            let s3 = (e3 >> K3_SYM1) & 0xff;
                            self.lut[w] = (3u32 << CNT_SH)
                                | (s3 << K3_SYM3)
                                | (cur & (0xffff << K3_SYM1))
                                | (len12 + l3);
                        }
                        tpl = Some((off_p, s1));
                    }
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> u16 {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).expect("valid canonical code")
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_coal(
        &self,
        e: u32,
        mut r: Reg,
        win: &[u8; COAL_WIN],
        full_data: &[u8],
        base_pos: usize,
    ) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
            if r.live < 20 {
                r.refill_coal_masked(win);
            }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_coal_masked(win);
                return (se >> 6, r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & U8_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let mut abs_r = r;
        abs_r.pos += base_pos;
        let s = self.decode_slow(&mut abs_r, full_data);
        abs_r.pos -= base_pos;
        abs_r.refill_coal_masked(win);
        (s, abs_r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (u16, Reg) {
        if (e & SUB_PTR) != 0 {
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
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (self.eob, r);
        } else if (e & U8_SYM256) != 0 {
            r.skip_short(e & 63);
            return (256, r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[inline(always)]
    fn decode_run_coal_k3(
        &self,
        r: &mut Reg,
        win: &[u8; COAL_WIN],
        data: &[u8],
        base_pos: usize,
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 3 * STEPS;
        let tail_stop = target - 3;
        macro_rules! step_k3 {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 3];
                    dst[0] = ((e >> K3_SYM1) as u8) as u16;
                    dst[1] = ((e >> K3_SYM2) as u8) as u16;
                    dst[2] = ((e >> K3_SYM3) as u8) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_coal(e, *r, win, data, base_pos);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return (k, true);
                    }
                }
            }};
        }
        r.pos = 0;
        r.refill_coal_masked(win);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_k3!();
            }
            r.refill_coal_masked(win);
        }
        while k <= tail_stop {
            if r.live < LUT_BITS {
                r.refill_coal_masked(win);
            }
            step_k3!();
        }
        // Finish remaining 1 or 2 symbols cleanly
        while k < target {
            if r.live < LUT_BITS {
                r.refill_coal_masked(win);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> CNT_SH) as usize;
            if c == 0 {
                let (s, after) = self.decode_entry_slow_coal(e, *r, win, data, base_pos);
                *r = after;
                out_slice[k] = s;
                k += 1;
                if s == eob {
                    r.pos += base_pos;
                    return (k, true);
                }
            } else if k + c <= target {
                r.skip_short_shl(e);
                let dst = &mut out_slice[k..k + 3];
                dst[0] = ((e >> K3_SYM1) as u8) as u16;
                dst[1] = ((e >> K3_SYM2) as u8) as u16;
                dst[2] = ((e >> K3_SYM3) as u8) as u16;
                k += c;
            } else {
                // Take only 1 (or 2 if target - k == 2)
                let s1 = ((e >> K3_SYM1) as u8) as u16;
                let l1 = self.sym_len[s1 as usize] as u32;
                if target - k == 2 && c >= 2 {
                    let s2 = ((e >> K3_SYM2) as u8) as u16;
                    let l2 = self.sym_len[s2 as usize] as u32;
                    r.skip_short(l1 + l2);
                    out_slice[k] = s1;
                    out_slice[k + 1] = s2;
                    k += 2;
                } else {
                    r.skip_short(l1);
                    out_slice[k] = s1;
                    k += 1;
                }
            }
        }
        r.pos += base_pos;
        (target, false)
    }

    #[inline(always)]
    fn decode_run_slice_k3(
        &self,
        r: &mut Reg,
        data: &[u8],
        out: &mut [u16],
        target: usize,
    ) -> (usize, bool) {
        let eob = self.eob;
        let out_slice = &mut out[..target + 16];
        let mut k = 0usize;
        let main_stop = target - 3 * STEPS;
        let tail_stop = target - 3;
        macro_rules! step_k3 {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    let dst = &mut out_slice[k..k + 3];
                    dst[0] = ((e >> K3_SYM1) as u8) as u16;
                    dst[1] = ((e >> K3_SYM2) as u8) as u16;
                    dst[2] = ((e >> K3_SYM3) as u8) as u16;
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow(e, *r, data);
                    *r = after;
                    out_slice[k] = s;
                    k += 1;
                    if s == eob {
                        return (k, true);
                    }
                }
            }};
        }
        r.refill(data);
        while k <= main_stop {
            for _ in 0..STEPS {
                step_k3!();
            }
            r.refill(data);
        }
        while k <= tail_stop {
            if r.live < LUT_BITS {
                r.refill(data);
            }
            step_k3!();
        }
        while k < target {
            let e = self.lut[r.peek(data, LUT_BITS) as usize];
            let c = (e >> CNT_SH) as usize;
            if c == 0 {
                let (s, after) = self.decode_entry_slow(e, *r, data);
                *r = after;
                out_slice[k] = s;
                k += 1;
                if s == eob {
                    return (k, true);
                }
            } else if k + c <= target {
                r.skip_short_shl(e);
                let dst = &mut out_slice[k..k + 3];
                dst[0] = ((e >> K3_SYM1) as u8) as u16;
                dst[1] = ((e >> K3_SYM2) as u8) as u16;
                dst[2] = ((e >> K3_SYM3) as u8) as u16;
                k += c;
            } else {
                let s1 = ((e >> K3_SYM1) as u8) as u16;
                let l1 = self.sym_len[s1 as usize] as u32;
                if target - k == 2 && c >= 2 {
                    let s2 = ((e >> K3_SYM2) as u8) as u16;
                    let l2 = self.sym_len[s2 as usize] as u32;
                    r.skip_short(l1 + l2);
                    out_slice[k] = s1;
                    out_slice[k + 1] = s2;
                    k += 2;
                } else {
                    r.skip_short(l1);
                    out_slice[k] = s1;
                    k += 1;
                }
            }
        }
        (target, false)
    }
}

// --- Pre-parsed block metadata for A3-only and stage-breakdown benchmarks ---

struct ParsedBlock {
    start_bit: u64,
    sym_start_bit: u64,
    end_bit: u64,
    alpha: usize,
    n_groups: usize,
    n_sel: usize,
    used_groups: u8,
    selectors: Vec<u8>,
    lens: Vec<[u8; MAX_ALPHA]>,
    expected_syms: Vec<u16>,
    /// Coalesced selector runs `(group_id, n_groups_in_run)` capped at `MAX_COAL_GROUPS`
    coal_runs_8: Vec<(u8, u16)>,
    /// Uncapped coalesced selector runs `(group_id, n_groups_in_run)`
    coal_runs_inf: Vec<(u8, u16)>,
}

fn build_coal_runs(sels: &[u8], cap: usize) -> Vec<(u8, u16)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < sels.len() {
        let g = sels[i];
        let mut m = 1usize;
        while i + m < sels.len() && sels[i + m] == g && m < cap {
            m += 1;
        }
        out.push((g, m as u16));
        i += m;
    }
    out
}

/// Parse all blocks using `mibz2::stream::Scanner` as ground truth, plus exact header/selector
/// census.
fn parse_corpus(data: &[u8]) -> (Vec<ParsedBlock>, CensusStats) {
    let mut stats = CensusStats::default();
    let mut scanner = Scanner::new(SliceSource(data));
    let mut blocks = Vec::new();
    let mut syms_buf = Vec::new();
    loop {
        match scanner.next(syms_buf).expect("valid bz2 corpus") {
            Event::Done => break,
            Event::Block {
                bit, end, symbols, ..
            } => {
                // Re-parse header/selectors/lengths at `bit` to record exact metadata
                let mut r = Reg::at(data, bit);
                let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
                assert_eq!(magic, BLOCK_MAGIC);
                let _crc = r.bits(data, 32);
                assert!(!r.bit(data));
                let _orig_ptr = r.bits(data, 24);
                let in_use16 = r.bits(data, 16) as u16;
                let mut n_in_use = 0usize;
                for i in 0..16 {
                    if in_use16 & (0x8000 >> i) != 0 {
                        let w = r.bits(data, 16) as u16;
                        n_in_use += w.count_ones() as usize;
                    }
                }
                let alpha = n_in_use + 2;
                let n_groups = r.bits(data, 3) as usize;
                let n_sel = r.bits(data, 15) as usize;
                let is_full = symbols.syms.len() >= 1000;
                if is_full {
                    stats.full_blocks += 1;
                    stats.full_sels += n_sel as u64;
                    stats.full_syms += symbols.syms.len() as u64;
                } else {
                    stats.runt_blocks += 1;
                    stats.runt_sels += n_sel as u64;
                    stats.runt_syms += symbols.syms.len() as u64;
                }

                let mut mtf: u64 = 0x0000_0504_0302_0100;
                let sel_cap = n_sel.min(MAX_SELECTORS);
                let mut selectors = vec![0u8; sel_cap];
                let mut used_groups = 0u8;
                #[allow(clippy::needless_range_loop)]
                for i in 0..n_sel {
                    if r.live < 8 {
                        r.refill(data);
                    }
                    let j = (!r.buf).leading_zeros() as usize;
                    r.skip_short((j + 1) as u32);
                    if j < 6 {
                        stats.sel_mtf_hist[j] += 1;
                    }
                    if i < MAX_SELECTORS {
                        let sh = (j * 8) as u32;
                        let v = ((mtf >> sh) & 0xff) as u8;
                        let low_mask = (1u64 << sh) - 1;
                        mtf = (mtf & !((low_mask << 8) | 0xff))
                            | ((mtf & low_mask) << 8)
                            | (v as u64);
                        selectors[i] = v;
                        used_groups |= 1u8 << v;
                    }
                }

                let mut lens = vec![[0u8; MAX_ALPHA]; n_groups];
                for (g, lg) in lens.iter_mut().enumerate().take(n_groups) {
                    let mut curr = r.bits(data, 5) as i32;
                    for l in lg[..alpha].iter_mut() {
                        stats.total_code_lens += 1;
                        if r.live < 8 {
                            r.refill(data);
                        }
                        if (r.buf >> 63) == 0 {
                            r.skip_short(1);
                            *l = curr as u8;
                            stats.unchanged_code_lens += 1;
                            continue;
                        }
                        loop {
                            if !r.bit(data) {
                                break;
                            }
                            stats.delta_steps += 1;
                            if r.bit(data) {
                                curr -= 1;
                            } else {
                                curr += 1;
                            }
                        }
                        *l = curr as u8;
                    }
                    stats.total_tables += 1;
                    if (used_groups >> g) & 1 != 0 {
                        if is_full {
                            stats.full_built_tables += 1;
                        } else {
                            stats.runt_built_tables += 1;
                        }
                    }
                }

                let sym_start_bit = r.bit_pos();
                let coal_runs_8 = build_coal_runs(&selectors, MAX_COAL_GROUPS);
                let coal_runs_inf = build_coal_runs(&selectors, usize::MAX);
                // Count actual groups used (up to EOB)
                let actual_groups = symbols.syms.len().div_ceil(GROUP);
                let actual_sels = &selectors[..actual_groups.min(selectors.len())];
                let actual_runs_inf = build_coal_runs(actual_sels, usize::MAX);
                let actual_runs_8 = build_coal_runs(actual_sels, MAX_COAL_GROUPS);
                if is_full {
                    stats.full_actual_groups += actual_sels.len() as u64;
                    stats.full_runs_inf += actual_runs_inf.len() as u64;
                    stats.full_runs_8 += actual_runs_8.len() as u64;
                    for &(_, m) in &actual_runs_inf {
                        let b = (m as usize).min(16);
                        stats.run_len_hist[b] += 1;
                    }
                }

                let expected_syms = symbols.syms;
                blocks.push(ParsedBlock {
                    start_bit: bit,
                    sym_start_bit,
                    end_bit: end,
                    alpha,
                    n_groups,
                    n_sel,
                    used_groups,
                    selectors,
                    lens,
                    expected_syms,
                    coal_runs_8,
                    coal_runs_inf,
                });
                syms_buf = Vec::new();
            }
        }
    }
    (blocks, stats)
}

#[derive(Default)]
struct CensusStats {
    full_blocks: usize,
    runt_blocks: usize,
    full_sels: u64,
    runt_sels: u64,
    full_syms: u64,
    runt_syms: u64,
    full_built_tables: usize,
    runt_built_tables: usize,
    total_tables: usize,
    sel_mtf_hist: [u64; 6],
    total_code_lens: u64,
    unchanged_code_lens: u64,
    delta_steps: u64,
    full_actual_groups: u64,
    full_runs_inf: u64,
    full_runs_8: u64,
    run_len_hist: [u64; 17], // 1..=15 exact, 16 = >=16
}

// --- Full read_item_with_scratch mirrors ---

struct ScratchK2 {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[HuffK2; MAX_GROUPS]>,
}

impl Default for ScratchK2 {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| HuffK2::default())),
        }
    }
}

struct ScratchK2U8 {
    runs: Vec<(u8, u16)>,
    tables: Box<[HuffK2U8; MAX_GROUPS]>,
}

impl Default for ScratchK2U8 {
    fn default() -> Self {
        Self {
            runs: Vec::with_capacity(MAX_SELECTORS),
            tables: Box::new(std::array::from_fn(|_| HuffK2U8::default())),
        }
    }
}

struct ScratchK3 {
    runs: Vec<(u8, u16)>,
    tables: Box<[HuffK3; MAX_GROUPS]>,
}

impl Default for ScratchK3 {
    fn default() -> Self {
        Self {
            runs: Vec::with_capacity(MAX_SELECTORS),
            tables: Box::new(std::array::from_fn(|_| HuffK3::default())),
        }
    }
}

/// Faithful mirror of `src/block.rs::read_item_with_scratch` (A2 + build + syms.resize + A3).
fn read_block_shipped(
    data: &[u8],
    bit: u64,
    syms: &mut Vec<u16>,
    scratch: &mut ScratchK2,
    zero_and_truncate: bool,
) -> (usize, u64) {
    let mut r = Reg::at(data, bit);
    let data_len = data.len();
    let _magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    let _crc = r.bits(data, 32);
    let _rand = r.bit(data);
    let _orig_ptr = r.bits(data, 24);
    let in_use16 = r.bits(data, 16) as u16;
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let mut w = (r.bits(data, 16) as u16).reverse_bits();
            let base_b = (i * 16) as u8;
            while w != 0 {
                let tz = w.trailing_zeros() as u8;
                w &= w - 1;
                seq_to_unseq[n_in_use] = base_b + tz;
                n_in_use += 1;
            }
        }
    }
    black_box(seq_to_unseq[0]);
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    let n_sel = r.bits(data, 15) as usize;

    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut scratch.selectors[..sel_cap];
    let mut used_groups = 0u8;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        r.skip_short((j + 1) as u32);
        if i < MAX_SELECTORS {
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            sel_buf[i] = v;
            used_groups |= 1u8 << v;
        }
        if r.pos > data_len && r.overran(data) {
            unreachable!();
        }
    }

    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 {
                r.refill(data);
            }
            if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
                if r.pos > data_len && r.overran(data) {
                    unreachable!();
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha], true);
        }
    }

    let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
    if zero_and_truncate {
        if syms.capacity() < MAX_SYMS {
            syms.clear();
            syms.reserve(MAX_SYMS);
        }
        if syms.len() < max_out {
            syms.resize(max_out, 0);
        }
    }
    let out = &mut syms[..max_out];
    let sel_buf = &scratch.selectors[..sel_cap];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut tail_buf = [0u16; 66];
    loop {
        let t = &scratch.tables[sel_buf[sel] as usize];
        sel += 1;
        let use_tail = n + 66 > max_out;
        let (k, hit_eob) = {
            let g_out: &mut [u16; 66] = if !use_tail {
                (&mut out[n..n + 66]).try_into().unwrap()
            } else {
                &mut tail_buf
            };
            if r.pos + 264 <= data_len {
                let base_pos = r.pos;
                let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
            } else {
                t.decode_group_slice(&mut r, data, g_out)
            }
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail_buf[..k]);
        }
        n += k;
        if hit_eob {
            break;
        }
    }
    if zero_and_truncate {
        syms.truncate(n);
    }
    (n, r.bit_pos())
}

/// Fast A2 selector MTF directly emitting coalesced runs `(group, count)` capped at `cap`.
/// Batches consecutive `j == 0` (`0` bits) using `r.buf.leading_zeros()`!
#[inline(always)]
fn decode_selectors_to_runs(
    r: &mut Reg,
    data: &[u8],
    n_sel: usize,
    n_groups: usize,
    runs: &mut Vec<(u8, u16)>,
    cap: usize,
) -> u8 {
    runs.clear();
    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let mut used_groups = 0u8;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let mut cur_g = 0u8;
    let mut cur_m = 0usize;
    let mut i = 0usize;
    while i < sel_cap {
        if r.live < 8 {
            r.refill(data);
        }
        if (r.buf >> 63) == 0 {
            // Fast path: batch consecutive `j == 0` (leading zero bits)
            let z = (r.buf.leading_zeros()).min(r.live) as usize;
            let take = z.min(sel_cap - i);
            r.skip_short(take as u32);
            let v = (mtf & 0xff) as u8;
            used_groups |= 1u8 << v;
            if cur_m > 0 && cur_g == v {
                cur_m += take;
            } else {
                if cur_m > 0 {
                    while cur_m > cap {
                        runs.push((cur_g, cap as u16));
                        cur_m -= cap;
                    }
                    runs.push((cur_g, cur_m as u16));
                }
                cur_g = v;
                cur_m = take;
            }
            i += take;
        } else {
            let j = (!r.buf).leading_zeros() as usize;
            debug_assert!(j < n_groups);
            r.skip_short((j + 1) as u32);
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            used_groups |= 1u8 << v;
            if cur_m > 0 {
                while cur_m > cap {
                    runs.push((cur_g, cap as u16));
                    cur_m -= cap;
                }
                runs.push((cur_g, cur_m as u16));
            }
            cur_g = v;
            cur_m = 1;
            i += 1;
        }
    }
    if cur_m > 0 {
        while cur_m > cap {
            runs.push((cur_g, cap as u16));
            cur_m -= cap;
        }
        runs.push((cur_g, cur_m as u16));
    }
    // Consume any selectors past MAX_SELECTORS
    for _ in sel_cap..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        r.skip_short((j + 1) as u32);
    }
    used_groups
}

/// Combined fast scanner (`fast A2` + no `syms.resize` + runt-gated `HuffK2U8` + coalesced `M<=8`).
fn read_block_combined_k2(
    data: &[u8],
    bit: u64,
    syms: &mut [u16],
    scratch: &mut ScratchK2U8,
) -> (usize, u64) {
    let mut r = Reg::at(data, bit);
    let data_len = data.len();
    let _magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    let _crc = r.bits(data, 32);
    let _rand = r.bit(data);
    let _orig_ptr = r.bits(data, 24);
    let in_use16 = r.bits(data, 16) as u16;
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let mut w = (r.bits(data, 16) as u16).reverse_bits();
            let base_b = (i * 16) as u8;
            while w != 0 {
                let tz = w.trailing_zeros() as u8;
                w &= w - 1;
                seq_to_unseq[n_in_use] = base_b + tz;
                n_in_use += 1;
            }
        }
    }
    black_box(seq_to_unseq[0]);
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    let n_sel = r.bits(data, 15) as usize;

    let used_groups =
        decode_selectors_to_runs(&mut r, data, n_sel, n_groups, &mut scratch.runs, usize::MAX);
    let build_pairs = n_sel >= GATE_SEL;
    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 {
                r.refill(data);
            }
            if (r.buf >> 63) == 0 {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha], build_pairs);
        }
    }

    let _ = data_len;
    let mut n = 0usize;
    for &(g, m) in &scratch.runs {
        let target = (m as usize) * GROUP;
        let t = &scratch.tables[g as usize];
        let (k, hit_eob) = t.decode_run_direct_u8(&mut r, data, &mut syms[n..], target);
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

/// Combined fast scanner with 4-byte packed K=3 + coalesced runs `M<=8`.
fn read_block_combined_k3(
    data: &[u8],
    bit: u64,
    syms: &mut [u16],
    scratch: &mut ScratchK3,
) -> (usize, u64) {
    let mut r = Reg::at(data, bit);
    let data_len = data.len();
    let _magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    let _crc = r.bits(data, 32);
    let _rand = r.bit(data);
    let _orig_ptr = r.bits(data, 24);
    let in_use16 = r.bits(data, 16) as u16;
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let mut w = (r.bits(data, 16) as u16).reverse_bits();
            let base_b = (i * 16) as u8;
            while w != 0 {
                let tz = w.trailing_zeros() as u8;
                w &= w - 1;
                seq_to_unseq[n_in_use] = base_b + tz;
                n_in_use += 1;
            }
        }
    }
    black_box(seq_to_unseq[0]);
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    let n_sel = r.bits(data, 15) as usize;

    let used_groups = decode_selectors_to_runs(
        &mut r,
        data,
        n_sel,
        n_groups,
        &mut scratch.runs,
        MAX_COAL_GROUPS,
    );
    let build_multi = n_sel >= GATE_SEL;
    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 {
                r.refill(data);
            }
            if (r.buf >> 63) == 0 {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha], build_multi);
        }
    }

    let mut n = 0usize;
    for &(g, m) in &scratch.runs {
        let target = (m as usize) * GROUP;
        let t = &scratch.tables[g as usize];
        let (k, hit_eob) = if r.pos + COAL_WIN <= data_len && n + target + 16 <= syms.len() {
            let base_pos = r.pos;
            let win: &[u8; COAL_WIN] = (&data[base_pos..base_pos + COAL_WIN]).try_into().unwrap();
            t.decode_run_coal_k3(&mut r, win, data, base_pos, &mut syms[n..], target)
        } else {
            t.decode_run_slice_k3(&mut r, data, &mut syms[n..], target)
        };
        n += k;
        if hit_eob {
            break;
        }
    }
    (n, r.bit_pos())
}

// --- A3-only decode runners (tables pre-built per block inside/outside clock) ---

fn run_a3_shipped(
    data: &[u8],
    blocks: &[ParsedBlock],
    tables: &mut [HuffK2; MAX_GROUPS],
    out: &mut [u16],
    masked: bool,
) -> u64 {
    let mut checksum = 0u64;
    let data_len = data.len();
    let mut tail_buf = [0u16; 66];
    for b in blocks {
        for (g, tbl) in tables.iter_mut().enumerate().take(b.n_groups) {
            if (b.used_groups >> g) & 1 != 0 {
                tbl.build_into(&b.lens[g][..b.alpha], true);
            }
        }
        let mut r = Reg::at(data, b.sym_start_bit);
        let max_out = (b.selectors.len() * GROUP + 66).min(MAX_SYMS);
        let mut n = 0usize;
        for &g in &b.selectors {
            let t = &tables[g as usize];
            let use_tail = n + 66 > max_out;
            let (k, hit_eob) = {
                let g_out: &mut [u16; 66] = if !use_tail {
                    (&mut out[n..n + 66]).try_into().unwrap()
                } else {
                    &mut tail_buf
                };
                if r.pos + 264 <= data_len {
                    let base_pos = r.pos;
                    let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                    if masked {
                        t.decode_group_win_masked(&mut r, g_in, data, base_pos, g_out)
                    } else {
                        t.decode_group_win(&mut r, g_in, data, base_pos, g_out)
                    }
                } else {
                    t.decode_group_slice(&mut r, data, g_out)
                }
            };
            if use_tail {
                out[n..n + k].copy_from_slice(&tail_buf[..k]);
            }
            n += k;
            if hit_eob {
                break;
            }
        }
        checksum ^= (n as u64).wrapping_add(out[n - 1] as u64);
    }
    checksum
}

fn run_a3_coalesce_k2(
    data: &[u8],
    blocks: &[ParsedBlock],
    tables: &mut [HuffK2; MAX_GROUPS],
    out: &mut [u16],
    direct: bool,
) -> u64 {
    let mut checksum = 0u64;
    let data_len = data.len();
    for b in blocks {
        let build_pairs = b.n_sel >= GATE_SEL;
        for (g, tbl) in tables.iter_mut().enumerate().take(b.n_groups) {
            if (b.used_groups >> g) & 1 != 0 {
                tbl.build_into(&b.lens[g][..b.alpha], build_pairs);
            }
        }
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0usize;
        if direct {
            for &(g, m) in &b.coal_runs_inf {
                let target = (m as usize) * GROUP;
                let (k, hit_eob) =
                    tables[g as usize].decode_run_direct(&mut r, data, &mut out[n..], target);
                n += k;
                if hit_eob {
                    break;
                }
            }
        } else {
            for &(g, m) in &b.coal_runs_8 {
                let target = (m as usize) * GROUP;
                let t = &tables[g as usize];
                let (k, hit_eob) = if r.pos + COAL_WIN <= data_len {
                    let base_pos = r.pos;
                    let win: &[u8; COAL_WIN] =
                        (&data[base_pos..base_pos + COAL_WIN]).try_into().unwrap();
                    t.decode_run_coal(&mut r, win, data, base_pos, &mut out[n..], target)
                } else {
                    t.decode_run_direct(&mut r, data, &mut out[n..], target)
                };
                n += k;
                if hit_eob {
                    break;
                }
            }
        }
        checksum ^= (n as u64).wrapping_add(out[n - 1] as u64);
    }
    checksum
}

/// `mode`: 0 = win8 (`[u16; 514]`, zero bounds checks), 1 = direct uncapped (`[u16; 18]` batch), 2 = win8 inlinesub.
fn run_a3_coalesce_u8(
    data: &[u8],
    blocks: &[ParsedBlock],
    tables: &mut [HuffK2U8; MAX_GROUPS],
    out: &mut [u16],
    mode: u8,
) -> u64 {
    let mut checksum = 0u64;
    let data_len = data.len();
    for b in blocks {
        let build_pairs = b.n_sel >= GATE_SEL;
        for (g, tbl) in tables.iter_mut().enumerate().take(b.n_groups) {
            if (b.used_groups >> g) & 1 != 0 {
                tbl.build_into(&b.lens[g][..b.alpha], build_pairs);
            }
        }
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0usize;
        if mode == 1 {
            for &(g, m) in &b.coal_runs_inf {
                let target = (m as usize) * GROUP;
                let (k, hit_eob) =
                    tables[g as usize].decode_run_direct_u8(&mut r, data, &mut out[n..], target);
                n += k;
                if hit_eob {
                    break;
                }
            }
        } else {
            for &(g, m) in &b.coal_runs_8 {
                let target = (m as usize) * GROUP;
                let t = &tables[g as usize];
                let (k, hit_eob) = if r.pos + COAL_WIN <= data_len && n + 514 <= out.len() {
                    let base_pos = r.pos;
                    let win: &[u8; COAL_WIN] =
                        (&data[base_pos..base_pos + COAL_WIN]).try_into().unwrap();
                    if mode == 2 {
                        t.decode_run_coal_inlinesub(
                            &mut r,
                            win,
                            data,
                            base_pos,
                            &mut out[n..],
                            target,
                        )
                    } else {
                        let out_win: &mut [u16; 514] = (&mut out[n..n + 514]).try_into().unwrap();
                        t.decode_run_coal_u8(&mut r, win, data, base_pos, out_win, target)
                    }
                } else {
                    t.decode_run_slice_u8(&mut r, data, &mut out[n..], target)
                };
                n += k;
                if hit_eob {
                    break;
                }
            }
        }
        checksum ^= (n as u64).wrapping_add(out[n - 1] as u64);
    }
    checksum
}

fn run_a3_coalesce_k3(
    data: &[u8],
    blocks: &[ParsedBlock],
    tables: &mut [HuffK3; MAX_GROUPS],
    out: &mut [u16],
) -> u64 {
    let mut checksum = 0u64;
    let data_len = data.len();
    for b in blocks {
        let build_multi = b.n_sel >= GATE_SEL;
        for (g, tbl) in tables.iter_mut().enumerate().take(b.n_groups) {
            if (b.used_groups >> g) & 1 != 0 {
                tbl.build_into(&b.lens[g][..b.alpha], build_multi);
            }
        }
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0usize;
        for &(g, m) in &b.coal_runs_8 {
            let target = (m as usize) * GROUP;
            let t = &tables[g as usize];
            let (k, hit_eob) = if r.pos + COAL_WIN <= data_len {
                let base_pos = r.pos;
                let win: &[u8; COAL_WIN] =
                    (&data[base_pos..base_pos + COAL_WIN]).try_into().unwrap();
                t.decode_run_coal_k3(&mut r, win, data, base_pos, &mut out[n..], target)
            } else {
                t.decode_run_slice_k3(&mut r, data, &mut out[n..], target)
            };
            n += k;
            if hit_eob {
                break;
            }
        }
        checksum ^= (n as u64).wrapping_add(out[n - 1] as u64);
    }
    checksum
}

fn simulate_syms_zeroing(blocks: &[ParsedBlock]) {
    // 1. Sequential reuse (single Vec<u16>, truncate(n) per block as in src/block.rs:268)
    let mut len = 0usize;
    let mut seq_triggers = 0usize;
    let mut seq_zero_u16s = 0u64;
    for b in blocks {
        let sel_cap = b.n_sel.min(MAX_SELECTORS);
        let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
        if len < max_out {
            seq_triggers += 1;
            seq_zero_u16s += (max_out - len) as u64;
        }
        len = b.expected_syms.len();
    }

    // 2. Parallel 8-worker LIFO free_syms stack (src/parallel.rs:1131,1205):
    // A runt finishes in ~0.002 ms (returned before next block is scanned), whereas a full block
    // finishes in ~5 ms (~15 blocks later). Simulate realistic return ordering!
    let slots = 32usize;
    let mut free_stack: Vec<usize> = Vec::new(); // stores syms.len() of recycled Vecs
    let mut in_flight: Vec<(usize, usize)> = Vec::new(); // (finish_step, syms_len)
    let mut par_triggers = 0usize;
    let mut par_zero_u16s = 0u64;
    let mut par_allocs = 0usize;
    for (step, b) in blocks.iter().enumerate() {
        // Retire any finished blocks in order
        let mut i = 0;
        while i < in_flight.len() {
            if in_flight[i].0 <= step {
                let (_, slen) = in_flight.remove(i);
                if free_stack.len() < slots {
                    free_stack.push(slen);
                }
            } else {
                i += 1;
            }
        }
        let cur_len = if let Some(l) = free_stack.pop() {
            l
        } else {
            par_allocs += 1;
            0
        };
        let sel_cap = b.n_sel.min(MAX_SELECTORS);
        let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
        if cur_len < max_out {
            par_triggers += 1;
            par_zero_u16s += (max_out - cur_len) as u64;
        }
        let n = b.expected_syms.len();
        let delay = if n < 1000 { 1 } else { 14 };
        in_flight.push((step + delay, n));
    }

    println!("\n=== 1c. syms.resize(max_out, 0) zero-fill census (src/block.rs:221-223,268) ===");
    println!(
        "  sequential reuse (1 Vec):  triggers = {:3}/{} blocks, zeroed = {:8} u16s ({:.2} MiB)",
        seq_triggers,
        blocks.len(),
        seq_zero_u16s,
        (seq_zero_u16s * 2) as f64 / (1024.0 * 1024.0),
    );
    println!(
        "  parallel 8W LIFO (sim):    triggers = {:3}/{} blocks (fresh allocs = {}), zeroed = {:8} u16s ({:.2} MiB)",
        par_triggers,
        blocks.len(),
        par_allocs,
        par_zero_u16s,
        (par_zero_u16s * 2) as f64 / (1024.0 * 1024.0),
    );
    let total_written_bytes = blocks
        .iter()
        .map(|b| b.expected_syms.len() as u64 * 2)
        .sum::<u64>();
    println!(
        "  actual symbols written:    {:.2} MiB (so seq zero-fill adds +{:.1}% extra writes!)",
        total_written_bytes as f64 / (1024.0 * 1024.0),
        100.0 * (seq_zero_u16s * 2) as f64 / total_written_bytes as f64,
    );
}

fn verify_all(data: &[u8], blocks: &[ParsedBlock]) {
    let mut sc_k2 = ScratchK2::default();
    let mut sc_u8 = ScratchK2U8::default();
    let mut sc_k3 = ScratchK3::default();
    let mut syms_seq = Vec::new();
    let mut syms_fixed = vec![0u16; MAX_SYMS + 520];
    let mut t_k2: [HuffK2; MAX_GROUPS] = std::array::from_fn(|_| HuffK2::default());
    let mut t_u8: [HuffK2U8; MAX_GROUPS] = std::array::from_fn(|_| HuffK2U8::default());
    let mut t_k3: [HuffK3; MAX_GROUPS] = std::array::from_fn(|_| HuffK3::default());
    let data_len = data.len();

    for (bi, b) in blocks.iter().enumerate() {
        let (n0, end0) = read_block_shipped(data, b.start_bit, &mut syms_seq, &mut sc_k2, true);
        assert_eq!(end0, b.end_bit, "shipped end bit block {bi}");
        assert_eq!(
            syms_seq[..n0],
            b.expected_syms[..],
            "shipped syms block {bi}"
        );

        let (n1, end1) = read_block_combined_k2(data, b.start_bit, &mut syms_fixed, &mut sc_u8);
        assert_eq!(end1, b.end_bit, "combined_k2 end bit block {bi}");
        assert_eq!(
            syms_fixed[..n1],
            b.expected_syms[..],
            "combined_k2 syms block {bi}"
        );

        let (n2, end2) = read_block_combined_k3(data, b.start_bit, &mut syms_fixed, &mut sc_k3);
        assert_eq!(end2, b.end_bit, "combined_k3 end bit block {bi}");
        assert_eq!(
            syms_fixed[..n2],
            b.expected_syms[..],
            "combined_k3 syms block {bi}"
        );

        // Also check A3 inline_sub and direct_k2 and masked_win
        for g in 0..b.n_groups {
            if (b.used_groups >> g) & 1 != 0 {
                t_k2[g].build_into(&b.lens[g][..b.alpha], b.n_sel >= GATE_SEL);
                t_u8[g].build_into(&b.lens[g][..b.alpha], b.n_sel >= GATE_SEL);
                t_k3[g].build_into(&b.lens[g][..b.alpha], b.n_sel >= GATE_SEL);
            }
        }
        // inline_sub
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0;
        for &(g, m) in &b.coal_runs_8 {
            let target = (m as usize) * GROUP;
            let (k, hit) = if r.pos + COAL_WIN <= data_len {
                let bp = r.pos;
                let win: &[u8; COAL_WIN] = (&data[bp..bp + COAL_WIN]).try_into().unwrap();
                t_u8[g as usize].decode_run_coal_inlinesub(
                    &mut r,
                    win,
                    data,
                    bp,
                    &mut syms_fixed[n..],
                    target,
                )
            } else {
                t_u8[g as usize].decode_run_slice_u8(&mut r, data, &mut syms_fixed[n..], target)
            };
            n += k;
            if hit {
                break;
            }
        }
        assert_eq!(r.bit_pos(), b.end_bit, "inline_sub end bit block {bi}");
        assert_eq!(
            syms_fixed[..n],
            b.expected_syms[..],
            "inline_sub block {bi}"
        );

        // direct_k2
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0;
        for &(g, m) in &b.coal_runs_inf {
            let target = (m as usize) * GROUP;
            let (k, hit) =
                t_k2[g as usize].decode_run_direct(&mut r, data, &mut syms_fixed[n..], target);
            n += k;
            if hit {
                break;
            }
        }
        assert_eq!(r.bit_pos(), b.end_bit, "direct_k2 end bit block {bi}");
        assert_eq!(syms_fixed[..n], b.expected_syms[..], "direct_k2 block {bi}");

        // coal_u8_win8 ([u16; 514])
        let mut r = Reg::at(data, b.sym_start_bit);
        let mut n = 0;
        for &(g, m) in &b.coal_runs_8 {
            let target = (m as usize) * GROUP;
            let (k, hit) = if r.pos + COAL_WIN <= data_len && n + 514 <= syms_fixed.len() {
                let bp = r.pos;
                let win: &[u8; COAL_WIN] = (&data[bp..bp + COAL_WIN]).try_into().unwrap();
                let out_win: &mut [u16; 514] = (&mut syms_fixed[n..n + 514]).try_into().unwrap();
                t_u8[g as usize].decode_run_coal_u8(&mut r, win, data, bp, out_win, target)
            } else {
                t_u8[g as usize].decode_run_slice_u8(&mut r, data, &mut syms_fixed[n..], target)
            };
            n += k;
            if hit {
                break;
            }
        }
        assert_eq!(r.bit_pos(), b.end_bit, "coal_u8_win8 end bit block {bi}");
        assert_eq!(
            syms_fixed[..n],
            b.expected_syms[..],
            "coal_u8_win8 block {bi}"
        );
    }
    // Synthetic alpha=258 edge case exercising sym=256 (short + SUB_PTR) and EOB at target-1
    {
        let mut lens = [9u8; MAX_ALPHA];
        lens[0] = 2; // 00
        lens[1] = 3; // 010
        lens[2] = 3; // 011
        lens[256] = 4; // 1000 (<=10-bit SYM256 slow tag!)
        lens[257] = 4; // 1001 (EOB_FAST)
                       // Remaining 253 symbols at len 12 (SUB_PTR) so Kraft sum <= 1
        for l in &mut lens[3..256] {
            *l = 12;
        }
        let mut tk2 = HuffK2::default();
        let mut tu8 = HuffK2U8::default();
        let mut tk3 = HuffK3::default();
        tk2.build_into(&lens, true);
        tu8.build_into(&lens, true);
        tk3.build_into(&lens, true);
        // Encode a 100-symbol run (M=2 groups) containing sym=0,1,2,256,SUB_PTR(100), ending with EOB=257 at pos 99
        let mut seq = Vec::new();
        for i in 0..99 {
            seq.push(match i % 7 {
                0 => 0u16,
                1 => 1,
                2 => 2,
                3 => 256,
                4 => 100,
                5 => 256,
                _ => 0,
            });
        }
        seq.push(257);
        let mut bits_out: Vec<u8> = Vec::new();
        let mut bit_n = 0usize;
        let mut put = |v: u32, nb: u32| {
            for k in (0..nb).rev() {
                if bit_n % 8 == 0 {
                    bits_out.push(0);
                }
                if (v >> k) & 1 != 0 {
                    *bits_out.last_mut().unwrap() |= 0x80 >> (bit_n % 8);
                }
                bit_n += 1;
            }
        };
        for &s in &seq {
            let zn = lens[s as usize] as u32;
            let at = tk2.perm[..MAX_ALPHA].iter().position(|&p| p == s).unwrap() as i32;
            let code = (at + tk2.base[zn as usize]) as u32;
            put(code, zn);
        }
        bits_out.resize(bits_out.len() + 1040, 0);
        let mut out_a = vec![0u16; 600];
        let mut r = Reg::at(&bits_out, 0);
        let (n, hit) = tu8.decode_run_direct_u8(&mut r, &bits_out, &mut out_a, 100);
        assert!(hit && n == 100 && out_a[..n] == seq[..] && r.bit_pos() == bit_n as u64);
        let mut r = Reg::at(&bits_out, 0);
        let win: &[u8; COAL_WIN] = (&bits_out[0..COAL_WIN]).try_into().unwrap();
        let out_win: &mut [u16; 514] = (&mut out_a[0..514]).try_into().unwrap();
        let (n, hit) = tu8.decode_run_coal_u8(&mut r, win, &bits_out, 0, out_win, 100);
        assert!(hit && n == 100 && out_a[..n] == seq[..] && r.bit_pos() == bit_n as u64);
        let mut r = Reg::at(&bits_out, 0);
        let (n, hit) = tk3.decode_run_coal_k3(&mut r, win, &bits_out, 0, &mut out_a, 100);
        assert!(hit && n == 100 && out_a[..n] == seq[..] && r.bit_pos() == bit_n as u64);
    }
    println!(
        "EXACTNESS PASS: all arms match mibz2::stream::Scanner symbol-for-symbol and end-bit-for-end-bit on all {} blocks ({} symbols) + synthetic alpha=258 (sym=256/SUB_PTR/EOB).",
        blocks.len(),
        blocks.iter().map(|b| b.expected_syms.len()).sum::<usize>()
    );
}

fn thread_cpu_ns() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    s.split_whitespace().next()?.parse::<u64>().ok()
}

fn median(vals: &mut [f64]) -> (f64, f64, f64) {
    vals.sort_by(|a, b| a.total_cmp(b));
    let n = vals.len();
    (vals[0], vals[n / 2], vals[n - 1])
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("corpora/wikidata.bz2");
    let repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);

    let data = std::fs::read(path).expect("read bz2 corpus");
    println!(
        "Corpus: {} ({} bytes, {:.2} MiB), repeats = {}",
        path,
        data.len(),
        data.len() as f64 / (1024.0 * 1024.0),
        repeats
    );

    let (blocks, stats) = parse_corpus(&data);
    let total_syms = stats.full_syms + stats.runt_syms;
    let total_sels = stats.full_sels + stats.runt_sels;
    println!("\n=== 1a. Corpus & selector-run census ===");
    println!(
        "  blocks: {} total ({} full, {} runts); symbols: {} ({} full, {} runts)",
        blocks.len(),
        stats.full_blocks,
        stats.runt_blocks,
        total_syms,
        stats.full_syms,
        stats.runt_syms
    );
    println!(
        "  selectors: {} total ({} full, {} runts); tables: {} read, {} built ({} full, {} runts)",
        total_sels,
        stats.full_sels,
        stats.runt_sels,
        stats.total_tables,
        stats.full_built_tables + stats.runt_built_tables,
        stats.full_built_tables,
        stats.runt_built_tables,
    );
    println!(
        "  selector MTF j=0..5: j0={} ({:.1}%), j1={} ({:.1}%), j2={} ({:.1}%), j3={} ({:.1}%), j4={} ({:.1}%), j5={} ({:.1}%)",
        stats.sel_mtf_hist[0],
        100.0 * stats.sel_mtf_hist[0] as f64 / total_sels as f64,
        stats.sel_mtf_hist[1],
        100.0 * stats.sel_mtf_hist[1] as f64 / total_sels as f64,
        stats.sel_mtf_hist[2],
        100.0 * stats.sel_mtf_hist[2] as f64 / total_sels as f64,
        stats.sel_mtf_hist[3],
        100.0 * stats.sel_mtf_hist[3] as f64 / total_sels as f64,
        stats.sel_mtf_hist[4],
        100.0 * stats.sel_mtf_hist[4] as f64 / total_sels as f64,
        stats.sel_mtf_hist[5],
        100.0 * stats.sel_mtf_hist[5] as f64 / total_sels as f64,
    );
    println!(
        "  233 full blocks actual groups: {} -> uncapped runs: {} (mean M={:.2}, -{:.1}% boundaries), M<=8 runs: {} (mean M={:.2}, -{:.1}% boundaries)",
        stats.full_actual_groups,
        stats.full_runs_inf,
        stats.full_actual_groups as f64 / stats.full_runs_inf as f64,
        100.0 * (1.0 - stats.full_runs_inf as f64 / stats.full_actual_groups as f64),
        stats.full_runs_8,
        stats.full_actual_groups as f64 / stats.full_runs_8 as f64,
        100.0 * (1.0 - stats.full_runs_8 as f64 / stats.full_actual_groups as f64),
    );
    print!("  run-length hist (M=1..15, >=16):");
    for m in 1..=16 {
        if m == 16 {
            print!(" M16+={}", stats.run_len_hist[m]);
        } else {
            print!(" M{m}={}", stats.run_len_hist[m]);
        }
    }
    println!();
    println!(
        "  code lengths: {} total, {} unchanged 0-bit ({:.1}%), {} delta steps (mean {:.2} steps/len)",
        stats.total_code_lens,
        stats.unchanged_code_lens,
        100.0 * stats.unchanged_code_lens as f64 / stats.total_code_lens as f64,
        stats.delta_steps,
        stats.delta_steps as f64 / stats.total_code_lens as f64,
    );

    simulate_syms_zeroing(&blocks);
    verify_all(&data, &blocks);

    // === 2. Interleaved stage breakdown of shipped read_item_with_scratch ===
    println!("\n=== 2. Full scanner stage breakdown ({repeats} interleaved repeats, ms) ===");
    let stage_names = [
        "A2a+b_hdr_sel_shipped",
        "A2b_sel_batched_runs",
        "A2c_codelens",
        "A2d_build_k2_all",
        "A2d_build_k2_gated",
        "A2d_build_k3_gated",
        "A2e_syms_zero_seq",
        "A3_dec_shipped_only",
    ];
    let mut stage_ms = vec![vec![0.0f64; repeats]; stage_names.len()];
    let mut t_k2: [HuffK2; MAX_GROUPS] = std::array::from_fn(|_| HuffK2::default());
    let mut t_k3: [HuffK3; MAX_GROUPS] = std::array::from_fn(|_| HuffK3::default());
    let mut sel_scratch = Box::new([0u8; MAX_SELECTORS]);
    let mut runs_scratch: Vec<(u8, u16)> = Vec::with_capacity(MAX_SELECTORS);
    let mut out_fixed = vec![0u16; MAX_SYMS + 520];
    // Pre-build tables once for A3_dec_shipped_only (per block tables stored)
    let prebuilt_k2: Vec<[HuffK2; MAX_GROUPS]> = blocks
        .iter()
        .map(|b| {
            let mut arr: [HuffK2; MAX_GROUPS] = std::array::from_fn(|_| HuffK2::default());
            for (g, tbl) in arr.iter_mut().enumerate().take(b.n_groups) {
                if (b.used_groups >> g) & 1 != 0 {
                    tbl.build_into(&b.lens[g][..b.alpha], true);
                }
            }
            arr
        })
        .collect();

    for rep in 0..repeats + 2 {
        for off in 0..stage_names.len() {
            let idx = (off + rep) % stage_names.len();
            let t0 = Instant::now();
            let mut acc = 0u64;
            match idx {
                0 => {
                    // A2a+b: header + in_use16 + shipped selector MTF
                    let data_len = data.len();
                    for b in &blocks {
                        let mut r = Reg::at(&data, b.start_bit);
                        let _m = (r.bits(&data, 24) as u64) << 24 | r.bits(&data, 24) as u64;
                        let _c = r.bits(&data, 32);
                        let _ra = r.bit(&data);
                        let _op = r.bits(&data, 24);
                        let in_use16 = r.bits(&data, 16) as u16;
                        let mut seq = [0u8; 256];
                        let mut niu = 0usize;
                        for i in 0..16 {
                            if in_use16 & (0x8000 >> i) != 0 {
                                let mut w = (r.bits(&data, 16) as u16).reverse_bits();
                                let base_b = (i * 16) as u8;
                                while w != 0 {
                                    let tz = w.trailing_zeros() as u8;
                                    w &= w - 1;
                                    seq[niu] = base_b + tz;
                                    niu += 1;
                                }
                            }
                        }
                        let _ng = r.bits(&data, 3) as usize;
                        let n_sel = r.bits(&data, 15) as usize;
                        let mut mtf: u64 = 0x0000_0504_0302_0100;
                        let sel_cap = n_sel.min(MAX_SELECTORS);
                        let mut ug = 0u8;
                        for i in 0..n_sel {
                            if r.live < 8 {
                                r.refill(&data);
                            }
                            let j = (!r.buf).leading_zeros() as usize;
                            r.skip_short((j + 1) as u32);
                            if i < MAX_SELECTORS {
                                let sh = (j * 8) as u32;
                                let v = ((mtf >> sh) & 0xff) as u8;
                                let lm = (1u64 << sh) - 1;
                                mtf = (mtf & !((lm << 8) | 0xff)) | ((mtf & lm) << 8) | (v as u64);
                                sel_scratch[i] = v;
                                ug |= 1u8 << v;
                            }
                            if r.pos > data_len && r.overran(&data) {
                                unreachable!();
                            }
                        }
                        acc ^= (ug as u64) + (seq[0] as u64) + (sel_scratch[sel_cap - 1] as u64);
                    }
                }
                1 => {
                    // A2a+b with batched j==0 selector runs
                    for b in &blocks {
                        let mut r = Reg::at(&data, b.start_bit);
                        let _m = (r.bits(&data, 24) as u64) << 24 | r.bits(&data, 24) as u64;
                        let _c = r.bits(&data, 32);
                        let _ra = r.bit(&data);
                        let _op = r.bits(&data, 24);
                        let in_use16 = r.bits(&data, 16) as u16;
                        let mut seq = [0u8; 256];
                        let mut niu = 0usize;
                        for i in 0..16 {
                            if in_use16 & (0x8000 >> i) != 0 {
                                let mut w = (r.bits(&data, 16) as u16).reverse_bits();
                                let base_b = (i * 16) as u8;
                                while w != 0 {
                                    let tz = w.trailing_zeros() as u8;
                                    w &= w - 1;
                                    seq[niu] = base_b + tz;
                                    niu += 1;
                                }
                            }
                        }
                        let ng = r.bits(&data, 3) as usize;
                        let n_sel = r.bits(&data, 15) as usize;
                        let ug = decode_selectors_to_runs(
                            &mut r,
                            &data,
                            n_sel,
                            ng,
                            &mut runs_scratch,
                            MAX_COAL_GROUPS,
                        );
                        acc ^= (ug as u64) + (seq[0] as u64) + (runs_scratch.len() as u64);
                    }
                }
                2 => {
                    // A2c: code-length delta decode (starting right after selectors)
                    let data_len = data.len();
                    let mut lens = [0u8; MAX_ALPHA];
                    for b in &blocks {
                        // Advance quickly to after selectors: re-read from header? Measure delta loop via bit offset!
                        // Wait: let's compute exact codelens start bit by scanning once? Better: parse from start_bit?
                        // No, let's measure A2c cleanly by advancing past header+selectors? Wait: we know start_bit and
                        // we can compute codelens_start_bit once before the loop! Let's use a simple inline check:
                        let mut r = Reg::at(&data, b.start_bit);
                        let _ = r.bits(&data, 24);
                        let _ = r.bits(&data, 24);
                        let _ = r.bits(&data, 32);
                        let _ = r.bit(&data);
                        let _ = r.bits(&data, 24);
                        let in_use16 = r.bits(&data, 16) as u16;
                        for i in 0..16 {
                            if in_use16 & (0x8000 >> i) != 0 {
                                let _ = r.bits(&data, 16);
                            }
                        }
                        let ng = r.bits(&data, 3) as usize;
                        let n_sel = r.bits(&data, 15) as usize;
                        for _ in 0..n_sel {
                            if r.live < 8 {
                                r.refill(&data);
                            }
                            let j = (!r.buf).leading_zeros() as usize;
                            r.skip_short((j + 1) as u32);
                        }
                        for _ in 0..ng {
                            let mut curr = r.bits(&data, 5) as i32;
                            for l in lens[..b.alpha].iter_mut() {
                                if r.live < 8 {
                                    r.refill(&data);
                                }
                                if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
                                    r.skip_short(1);
                                    *l = curr as u8;
                                    continue;
                                }
                                loop {
                                    if !r.bit(&data) {
                                        break;
                                    }
                                    if r.bit(&data) {
                                        curr -= 1;
                                    } else {
                                        curr += 1;
                                    }
                                    if r.pos > data_len && r.overran(&data) {
                                        unreachable!();
                                    }
                                }
                                *l = curr as u8;
                            }
                        }
                        acc ^= lens[b.alpha - 1] as u64;
                    }
                }
                3 => {
                    for b in &blocks {
                        for (g, tbl) in t_k2.iter_mut().enumerate().take(b.n_groups) {
                            if (b.used_groups >> g) & 1 != 0 {
                                tbl.build_into(&b.lens[g][..b.alpha], true);
                                acc ^= tbl.lut[0] as u64;
                            }
                        }
                    }
                }
                4 => {
                    for b in &blocks {
                        let bp = b.n_sel >= GATE_SEL;
                        for (g, tbl) in t_k2.iter_mut().enumerate().take(b.n_groups) {
                            if (b.used_groups >> g) & 1 != 0 {
                                tbl.build_into(&b.lens[g][..b.alpha], bp);
                                acc ^= tbl.lut[0] as u64;
                            }
                        }
                    }
                }
                5 => {
                    for b in &blocks {
                        let bm = b.n_sel >= GATE_SEL;
                        for (g, tbl) in t_k3.iter_mut().enumerate().take(b.n_groups) {
                            if (b.used_groups >> g) & 1 != 0 {
                                tbl.build_into(&b.lens[g][..b.alpha], bm);
                                acc ^= tbl.lut[0] as u64;
                            }
                        }
                    }
                }
                6 => {
                    // Pure syms.resize(max_out, 0) + truncate(n) sequential reuse cost!
                    let mut syms = Vec::with_capacity(MAX_SYMS);
                    for b in &blocks {
                        let sel_cap = b.n_sel.min(MAX_SELECTORS);
                        let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
                        if syms.len() < max_out {
                            syms.resize(max_out, 0);
                        }
                        acc ^= syms[max_out - 1] as u64;
                        syms.truncate(b.expected_syms.len());
                    }
                }
                7 => {
                    let data_len = data.len();
                    let mut tail_buf = [0u16; 66];
                    for (bi, b) in blocks.iter().enumerate() {
                        let tbls = &prebuilt_k2[bi];
                        let mut r = Reg::at(&data, b.sym_start_bit);
                        let max_out = (b.selectors.len() * GROUP + 66).min(MAX_SYMS);
                        let mut n = 0usize;
                        for &g in &b.selectors {
                            let t = &tbls[g as usize];
                            let use_tail = n + 66 > max_out;
                            let (k, hit) = {
                                let g_out: &mut [u16; 66] = if !use_tail {
                                    (&mut out_fixed[n..n + 66]).try_into().unwrap()
                                } else {
                                    &mut tail_buf
                                };
                                if r.pos + 264 <= data_len {
                                    let bp = r.pos;
                                    let g_in: &[u8; 264] =
                                        (&data[bp..bp + 264]).try_into().unwrap();
                                    t.decode_group_win(&mut r, g_in, &data, bp, g_out)
                                } else {
                                    t.decode_group_slice(&mut r, &data, g_out)
                                }
                            };
                            if use_tail {
                                out_fixed[n..n + k].copy_from_slice(&tail_buf[..k]);
                            }
                            n += k;
                            if hit {
                                break;
                            }
                        }
                        acc ^= (n as u64) + (out_fixed[n - 1] as u64);
                    }
                }
                _ => unreachable!(),
            }
            black_box(acc);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            if rep >= 2 {
                stage_ms[idx][rep - 2] = dt;
            }
        }
    }
    println!(
        "  {:<24} {:>8} {:>8} {:>8}",
        "stage", "min ms", "med ms", "max ms"
    );
    for (i, name) in stage_names.iter().enumerate() {
        let (mn, md, mx) = median(&mut stage_ms[i].clone());
        println!("  {:<24} {:>8.3} {:>8.3} {:>8.3}", name, mn, md, mx);
    }

    // === 3. Interleaved full-scanner & A3 ablation ===
    println!("\n=== 3. Full-scanner & A3 micro-mechanism ablation ({repeats} interleaved repeats, ms) ===");
    let arm_names = [
        "lib_scanner_seq",
        "shipped_full_zero",
        "full_no_zero",
        "pool16_shipped_zero",
        "pool16_no_zero",
        "full_combined_k2",
        "full_combined_k3",
        "a3_shipped",
        "a3_mask_refill",
        "a3_coal_k2_win8",
        "a3_coal_k2_direct",
        "a3_coal_k2_u8_win8",
        "a3_coal_k2_u8_dir",
        "a3_coal_k2_inlinesub",
        "a3_coal_k3_win8",
    ];
    let mut arm_ms = vec![vec![0.0f64; repeats]; arm_names.len()];
    let mut arm_cpu_ms = vec![vec![0.0f64; repeats]; arm_names.len()];
    let mut sc_k2 = ScratchK2::default();
    let mut sc_u8 = ScratchK2U8::default();
    let mut sc_k3 = ScratchK3::default();
    let mut t_u8: [HuffK2U8; MAX_GROUPS] = std::array::from_fn(|_| HuffK2U8::default());
    let mut syms_reused = Vec::with_capacity(MAX_SYMS);
    let mut lib_sv = Vec::with_capacity(MAX_SYMS);
    // 16-buffer rotating pool (~28.8 MiB capacity, ~4-5 MiB active) mirroring 8-worker free_syms
    let mut pool_zero: Vec<Vec<u16>> = (0..16).map(|_| vec![0u16; MAX_SYMS]).collect();
    let mut pool_nozero: Vec<Vec<u16>> = (0..16).map(|_| vec![0u16; MAX_SYMS]).collect();

    for rep in 0..repeats + 2 {
        for off in 0..arm_names.len() {
            let idx = (off + rep) % arm_names.len();
            let c0 = thread_cpu_ns();
            let t0 = Instant::now();
            let mut acc = 0u64;
            match idx {
                0 => {
                    // Shipped library Scanner::next with pre-allocated sequential Vec reuse
                    let mut sc = Scanner::new(SliceSource(&data));
                    let mut sv = std::mem::take(&mut lib_sv);
                    sv.clear();
                    loop {
                        match sc.next(sv).unwrap() {
                            Event::Done => {
                                lib_sv = Vec::with_capacity(MAX_SYMS);
                                break;
                            }
                            Event::Block { symbols, .. } => {
                                acc ^= (symbols.syms.len() as u64)
                                    .wrapping_add(*symbols.syms.last().unwrap() as u64);
                                sv = symbols.syms;
                            }
                        }
                    }
                }
                1 => {
                    syms_reused.clear();
                    for b in &blocks {
                        let (n, end) = read_block_shipped(
                            &data,
                            b.start_bit,
                            &mut syms_reused,
                            &mut sc_k2,
                            true,
                        );
                        acc ^= (n as u64) ^ end;
                    }
                }
                2 => {
                    if syms_reused.len() < MAX_SYMS {
                        syms_reused.resize(MAX_SYMS, 0);
                    }
                    for b in &blocks {
                        let (n, end) = read_block_shipped(
                            &data,
                            b.start_bit,
                            &mut syms_reused,
                            &mut sc_k2,
                            false,
                        );
                        acc ^= (n as u64) ^ end;
                    }
                }
                3 => {
                    // 16-buffer LIFO-like pool with runt truncation + resize(max_out, 0)
                    let mut pi = 0usize;
                    for b in &blocks {
                        let buf = &mut pool_zero[pi];
                        let (n, end) =
                            read_block_shipped(&data, b.start_bit, buf, &mut sc_k2, true);
                        acc ^= (n as u64) ^ end;
                        // A runt returns immediately to the top of the LIFO stack (reuse same slot!)
                        if n >= 1000 {
                            pi = (pi + 1) & 15;
                        }
                    }
                }
                4 => {
                    // 16-buffer LIFO-like pool WITHOUT truncate / resize(max_out, 0)
                    let mut pi = 0usize;
                    for b in &blocks {
                        let buf = &mut pool_nozero[pi];
                        let (n, end) =
                            read_block_shipped(&data, b.start_bit, buf, &mut sc_k2, false);
                        acc ^= (n as u64) ^ end;
                        if n >= 1000 {
                            pi = (pi + 1) & 15;
                        }
                    }
                }
                5 => {
                    for b in &blocks {
                        let (n, end) =
                            read_block_combined_k2(&data, b.start_bit, &mut out_fixed, &mut sc_u8);
                        acc ^= (n as u64) ^ end;
                    }
                }
                6 => {
                    for b in &blocks {
                        let (n, end) =
                            read_block_combined_k3(&data, b.start_bit, &mut out_fixed, &mut sc_k3);
                        acc ^= (n as u64) ^ end;
                    }
                }
                7 => {
                    acc = run_a3_shipped(&data, &blocks, &mut t_k2, &mut out_fixed, false);
                }
                8 => {
                    acc = run_a3_shipped(&data, &blocks, &mut t_k2, &mut out_fixed, true);
                }
                9 => {
                    acc = run_a3_coalesce_k2(&data, &blocks, &mut t_k2, &mut out_fixed, false);
                }
                10 => {
                    acc = run_a3_coalesce_k2(&data, &blocks, &mut t_k2, &mut out_fixed, true);
                }
                11 => {
                    acc = run_a3_coalesce_u8(&data, &blocks, &mut t_u8, &mut out_fixed, 0);
                }
                12 => {
                    acc = run_a3_coalesce_u8(&data, &blocks, &mut t_u8, &mut out_fixed, 1);
                }
                13 => {
                    acc = run_a3_coalesce_u8(&data, &blocks, &mut t_u8, &mut out_fixed, 2);
                }
                14 => {
                    acc = run_a3_coalesce_k3(&data, &blocks, &mut t_k3, &mut out_fixed);
                }
                _ => unreachable!(),
            }
            black_box(acc);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            let dc = match (c0, thread_cpu_ns()) {
                (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 / 1e6,
                _ => dt,
            };
            if idx == 0 {
                // Pre-touch outside clock for next repeat (Scanner::next drops sv on Done)
                lib_sv.resize(MAX_SYMS, 0);
                lib_sv.clear();
            }
            if rep >= 2 {
                arm_ms[idx][rep - 2] = dt;
                arm_cpu_ms[idx][rep - 2] = dc;
            }
        }
    }

    let print_table = |label: &str, data_ms: &[Vec<f64>]| {
        println!("\n--- {label} ---");
        let (_, lib_med, _) = median(&mut data_ms[0].clone());
        let (sh_min, sh_med, sh_max) = median(&mut data_ms[1].clone());
        let (a3_min, a3_med, a3_max) = median(&mut data_ms[7].clone());
        println!(
            "  {:<22} {:>8} {:>8} {:>8} {:>9} {:>9} {:>8}",
            "arm", "min ms", "med ms", "max ms", "vs ref", "overlap?", "ns/sym"
        );
        for (i, name) in arm_names.iter().enumerate() {
            let (mn, md, mx) = median(&mut data_ms[i].clone());
            let (ref_mn, ref_md, ref_mx) = if i < 7 {
                (sh_min, sh_med, sh_max)
            } else {
                (a3_min, a3_med, a3_max)
            };
            let pct = 100.0 * (md - ref_md) / ref_md;
            let ov = if i == 1 || i == 7 {
                "—"
            } else if mx < ref_mn || mn > ref_mx {
                "NO"
            } else {
                "YES"
            };
            let extra = if i == 0 {
                format!(
                    "(lib vs mirror: {:+.2}%)",
                    100.0 * (lib_med - sh_med) / sh_med
                )
            } else {
                String::new()
            };
            println!(
                "  {:<22} {:>8.3} {:>8.3} {:>8.3} {:>+8.2}% {:>9} {:>8.3} {}",
                name,
                mn,
                md,
                mx,
                pct,
                ov,
                md * 1e6 / total_syms as f64,
                extra
            );
        }
    };
    print_table(
        "On-CPU exec time (/proc/thread-self/schedstat)",
        &arm_cpu_ms,
    );
    print_table("Wall-clock time (Instant::now)", &arm_ms);
}
