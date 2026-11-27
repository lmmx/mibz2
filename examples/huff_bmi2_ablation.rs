#![deny(unsafe_op_in_unsafe_fn)]
//! (research only, `huff-bmi2-dispatch`) Runtime ISA-dispatch ablation for the shipped
//! `K = 2` 10-bit root Huffman decoder (`src/huff.rs` + `src/block.rs` group loop).
//!
//!     cargo run --release --example huff_bmi2_ablation -- corpora/wikidata.bz2 [REPEATS]
//!
//! Isolates pure `bmi2` (`shlx`/`shrx`) from `bmi1`, `avx2`, `avx2+bmi1` (what `Kernels::detect`
//! currently probes), `bmi1+bmi2`, and `avx2+bmi1+bmi2`, plus block-level vs per-group `bmi2`
//! dispatch, via targeted `#[target_feature]` arms inside a single baseline x86-64 binary (zero
//! `src/` edits; the only `unsafe` is calling runtime-checked `#[target_feature]` wrappers).
//! Every arm is verified 100% symbol-for-symbol and end-bit-identical against an independent
//! canonical bit-by-bit oracle on all 431 blocks of `corpora/wikidata.bz2` (24,979,299 symbols)
//! plus synthetic edge cases (sym 256, EOB, `SUB_PTR`, `decode_group_slice`, `decode_slow`,
//! and `BadHuffmanCode`).

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

const SUB_PTR: u32 = 1 << 27;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const LEN1: u32 = 6;
const SYM1: u32 = 11;
const SYM2: u32 = 20;
const CNT_SH: u32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorKind {
    BadHuffmanCode,
    SelectorOverrun,
    Truncated,
}

// ============================================================================
// Faithful mirror of src/bits.rs (Reg)
// ============================================================================

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

// ============================================================================
// Faithful mirror of src/huff.rs (Huff)
// ============================================================================

struct Huff {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for Huff {
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

impl Huff {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    #[inline(always)]
    fn build_into_inline(&mut self, lens: &[u8]) {
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

    #[inline(always)]
    fn decode_one_win(
        &self,
        r: &mut Reg,
        win: &[u8; 264],
        full_data: &[u8],
        base_pos: usize,
    ) -> Result<u16, ErrorKind> {
        if r.live < LUT_BITS {
            r.refill_win(win);
        }
        let e = self.lut[r.peek_live(LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return Ok(((e >> SYM1) & 0x1ff) as u16);
        }
        let (s, after) = self.decode_entry_slow_win(e, *r, win, full_data, base_pos);
        *r = after;
        s
    }

    #[inline(always)]
    fn decode_one_slice(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return Ok(((e >> SYM1) & 0x1ff) as u16);
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
    ) -> Result<(usize, bool), ErrorKind> {
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
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => {
                            r.pos += base_pos;
                            return Err(err);
                        }
                    };
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return Ok((k, true));
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
            let s = match self.decode_one_win(r, g_in, data, base_pos) {
                Ok(s) => s,
                Err(err) => {
                    r.pos += base_pos;
                    return Err(err);
                }
            };
            g_out[GROUP - 1] = s;
            if s == eob {
                r.pos += base_pos;
                return Ok((GROUP, true));
            }
        }
        r.pos += base_pos;
        Ok((GROUP, false))
    }

    #[inline(always)]
    fn decode_group_slice(
        &self,
        r: &mut Reg,
        data: &[u8],
        g_out: &mut [u16; 66],
    ) -> Result<(usize, bool), ErrorKind> {
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
                    let s = s?;
                    g_out[k & 63] = s;
                    k += 1;
                    if s == eob {
                        return Ok((k, true));
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
            let s = self.decode_one_slice(r, data)?;
            g_out[GROUP - 1] = s;
            if s == eob {
                return Ok((GROUP, true));
            }
        }
        if r.pos > data.len() && r.overran(data) {
            Err(ErrorKind::Truncated)
        } else {
            Ok((GROUP, false))
        }
    }

    #[inline(always)]
    fn decode_group_win_nostore(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
    ) -> Result<(usize, bool, u64), ErrorKind> {
        let eob = self.eob;
        let mut k = 0usize;
        let mut acc = 0u64;
        macro_rules! step_ns {
            () => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl(e);
                    acc = acc.wrapping_add(e as u64);
                    k += c;
                } else {
                    let (s, after) = self.decode_entry_slow_win(e, *r, g_in, data, base_pos);
                    *r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => {
                            r.pos += base_pos;
                            return Err(err);
                        }
                    };
                    acc = acc.wrapping_add(s as u64);
                    k += 1;
                    if s == eob {
                        r.pos += base_pos;
                        return Ok((k, true, acc));
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
            let s = match self.decode_one_win(r, g_in, data, base_pos) {
                Ok(s) => s,
                Err(err) => {
                    r.pos += base_pos;
                    return Err(err);
                }
            };
            acc = acc.wrapping_add(s as u64);
            if s == eob {
                r.pos += base_pos;
                return Ok((GROUP, true, acc));
            }
        }
        r.pos += base_pos;
        Ok((GROUP, false, acc))
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
    ) -> (Result<u16, ErrorKind>, Reg) {
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
                return (Ok(se >> 6), r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (Ok(self.eob), r);
        } else if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
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
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill(data);
                return (Ok(se >> 6), r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (Ok(self.eob), r);
        } else if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let s = self.decode_slow(&mut r, data);
        r.refill(data);
        (s, r)
    }

    #[cold]
    #[inline(never)]
    fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return Err(ErrorKind::BadHuffmanCode);
            }
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode)
    }
}

// ============================================================================
// Block-level symbol decode loop (faithful mirror of src/block.rs:220-262)
// ============================================================================

#[inline(always)]
fn decode_block_groups_inline(
    tables: &[Huff; 6],
    selectors: &[u8],
    data: &[u8],
    start_bit: u64,
    syms: &mut [u16],
) -> Result<(usize, u64), ErrorKind> {
    let sel_cap = selectors.len();
    let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
    let out = &mut syms[..max_out];
    let data_len = data.len();
    let mut r = Reg::at(data, start_bit);
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut tail_buf = [0u16; 66];

    loop {
        if sel >= sel_cap {
            return Err(ErrorKind::SelectorOverrun);
        }
        let t = &tables[selectors[sel] as usize];
        sel += 1;
        let use_tail = n + 66 > max_out;
        let g_res = {
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
        let (k, hit_eob) = g_res?;
        if use_tail {
            out[n..n + k].copy_from_slice(&tail_buf[..k]);
        }
        n += k;
        if hit_eob {
            return Ok((n, r.bit_pos()));
        }
    }
}

#[inline(always)]
fn decode_block_nostore_inline(
    tables: &[Huff; 6],
    selectors: &[u8],
    data: &[u8],
    start_bit: u64,
) -> Result<(usize, u64, u64), ErrorKind> {
    let sel_cap = selectors.len();
    let data_len = data.len();
    let mut r = Reg::at(data, start_bit);
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut acc = 0u64;
    let mut tail_buf = [0u16; 66];

    loop {
        if sel >= sel_cap {
            return Err(ErrorKind::SelectorOverrun);
        }
        let t = &tables[selectors[sel] as usize];
        sel += 1;
        let (k, hit_eob) = if r.pos + 264 <= data_len {
            let base_pos = r.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            let (k, hit_eob, g_acc) = t.decode_group_win_nostore(&mut r, g_in, data, base_pos)?;
            acc = acc.wrapping_add(g_acc);
            (k, hit_eob)
        } else {
            let res = t.decode_group_slice(&mut r, data, &mut tail_buf)?;
            acc = acc.wrapping_add(tail_buf[0] as u64);
            res
        };
        n += k;
        if hit_eob {
            return Ok((n, r.bit_pos(), acc));
        }
    }
}

// ============================================================================
// Non-inlined ablation entry points (clear symbols for objdump -d inspection)
// ============================================================================

#[inline(never)]
fn huff_build_baseline(h: &mut Huff, lens: &[u8]) {
    h.build_into_inline(lens);
}

#[inline(never)]
fn huff_dec_baseline(
    tables: &[Huff; 6],
    selectors: &[u8],
    data: &[u8],
    start_bit: u64,
    syms: &mut [u16],
) -> Result<(usize, u64), ErrorKind> {
    decode_block_groups_inline(tables, selectors, data, start_bit, syms)
}

#[inline(never)]
fn huff_nostore_baseline(
    tables: &[Huff; 6],
    selectors: &[u8],
    data: &[u8],
    start_bit: u64,
) -> Result<(usize, u64, u64), ErrorKind> {
    decode_block_nostore_inline(tables, selectors, data, start_bit)
}

#[cfg(target_arch = "x86_64")]
mod x86_arms {
    use super::*;

    #[target_feature(enable = "bmi2")]
    #[inline(never)]
    pub unsafe fn huff_build_bmi2(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "bmi1")]
    #[inline(never)]
    pub unsafe fn huff_build_bmi1(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "avx2")]
    #[inline(never)]
    pub unsafe fn huff_build_avx2(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "avx2,bmi1")]
    #[inline(never)]
    pub unsafe fn huff_build_avx2_bmi1(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "bmi1,bmi2")]
    #[inline(never)]
    pub unsafe fn huff_build_bmi1_bmi2(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "avx2,bmi1,bmi2")]
    #[inline(never)]
    pub unsafe fn huff_build_avx2_bmi1_bmi2(h: &mut Huff, lens: &[u8]) {
        h.build_into_inline(lens);
    }

    #[target_feature(enable = "bmi2")]
    #[inline(never)]
    pub unsafe fn huff_dec_bmi2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "bmi2")]
    #[inline(never)]
    unsafe fn huff_group_win_bmi2(
        t: &Huff,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; 66],
    ) -> Result<(usize, bool), ErrorKind> {
        t.decode_group_win(r, g_in, data, base_pos, g_out)
    }

    #[target_feature(enable = "bmi2")]
    #[inline(never)]
    unsafe fn huff_group_slice_bmi2(
        t: &Huff,
        r: &mut Reg,
        data: &[u8],
        g_out: &mut [u16; 66],
    ) -> Result<(usize, bool), ErrorKind> {
        t.decode_group_slice(r, data, g_out)
    }

    /// Per-group BMI2 dispatch (baseline outer block loop calling non-inlined BMI2 group fn).
    #[inline(never)]
    pub unsafe fn huff_dec_bmi2_grp(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        let sel_cap = selectors.len();
        let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
        let out = &mut syms[..max_out];
        let data_len = data.len();
        let mut r = Reg::at(data, start_bit);
        let mut n = 0usize;
        let mut sel = 0usize;
        let mut tail_buf = [0u16; 66];

        loop {
            if sel >= sel_cap {
                return Err(ErrorKind::SelectorOverrun);
            }
            let t = &tables[selectors[sel] as usize];
            sel += 1;
            let use_tail = n + 66 > max_out;
            let g_res = {
                let g_out: &mut [u16; 66] = if !use_tail {
                    (&mut out[n..n + 66]).try_into().unwrap()
                } else {
                    &mut tail_buf
                };
                if r.pos + 264 <= data_len {
                    let base_pos = r.pos;
                    let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
                    // SAFETY: caller verified BMI2 via `is_x86_feature_detected!("bmi2")`.
                    unsafe { huff_group_win_bmi2(t, &mut r, g_in, data, base_pos, g_out) }
                } else {
                    // SAFETY: caller verified BMI2 via `is_x86_feature_detected!("bmi2")`.
                    unsafe { huff_group_slice_bmi2(t, &mut r, data, g_out) }
                }
            };
            let (k, hit_eob) = g_res?;
            if use_tail {
                out[n..n + k].copy_from_slice(&tail_buf[..k]);
            }
            n += k;
            if hit_eob {
                return Ok((n, r.bit_pos()));
            }
        }
    }

    #[target_feature(enable = "bmi1")]
    #[inline(never)]
    pub unsafe fn huff_dec_bmi1(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "avx2")]
    #[inline(never)]
    pub unsafe fn huff_dec_avx2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "avx2,bmi1")]
    #[inline(never)]
    pub unsafe fn huff_dec_avx2_bmi1(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "bmi1,bmi2")]
    #[inline(never)]
    pub unsafe fn huff_dec_bmi1_bmi2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "avx2,bmi1,bmi2")]
    #[inline(never)]
    pub unsafe fn huff_dec_avx2_bmi1_bmi2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
        syms: &mut [u16],
    ) -> Result<(usize, u64), ErrorKind> {
        decode_block_groups_inline(tables, selectors, data, start_bit, syms)
    }

    #[target_feature(enable = "bmi2")]
    #[inline(never)]
    pub unsafe fn huff_nostore_bmi2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
    ) -> Result<(usize, u64, u64), ErrorKind> {
        decode_block_nostore_inline(tables, selectors, data, start_bit)
    }

    #[target_feature(enable = "bmi1")]
    #[inline(never)]
    pub unsafe fn huff_nostore_bmi1(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
    ) -> Result<(usize, u64, u64), ErrorKind> {
        decode_block_nostore_inline(tables, selectors, data, start_bit)
    }

    #[target_feature(enable = "avx2")]
    #[inline(never)]
    pub unsafe fn huff_nostore_avx2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
    ) -> Result<(usize, u64, u64), ErrorKind> {
        decode_block_nostore_inline(tables, selectors, data, start_bit)
    }

    #[target_feature(enable = "avx2,bmi1,bmi2")]
    #[inline(never)]
    pub unsafe fn huff_nostore_avx2_bmi1_bmi2(
        tables: &[Huff; 6],
        selectors: &[u8],
        data: &[u8],
        start_bit: u64,
    ) -> Result<(usize, u64, u64), ErrorKind> {
        decode_block_nostore_inline(tables, selectors, data, start_bit)
    }
}

// ============================================================================
// Arm dispatch table & scratch
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Baseline,
    #[cfg(target_arch = "x86_64")]
    Bmi2Dec,
    #[cfg(target_arch = "x86_64")]
    Bmi2All,
    #[cfg(target_arch = "x86_64")]
    Bmi2Grp,
    #[cfg(target_arch = "x86_64")]
    Bmi1All,
    #[cfg(target_arch = "x86_64")]
    Avx2All,
    #[cfg(target_arch = "x86_64")]
    Avx2Bmi1,
    #[cfg(target_arch = "x86_64")]
    Bmi1Bmi2,
    #[cfg(target_arch = "x86_64")]
    Avx2Bmi1Bmi2,
    #[cfg(target_arch = "x86_64")]
    Avx2TblBmi2Dec,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Baseline => "baseline",
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi2Dec => "bmi2_dec",
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi2All => "bmi2_all",
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi2Grp => "bmi2_grp",
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi1All => "bmi1_all",
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2All => "avx2_all",
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2Bmi1 => "avx2_bmi1",
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi1Bmi2 => "bmi1_bmi2",
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2Bmi1Bmi2 => "avx2_bmi1_bmi2",
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2TblBmi2Dec => "avx2_t_bmi2_d",
        }
    }

    fn available_arms() -> Vec<Arm> {
        let mut v = vec![Arm::Baseline];
        #[cfg(target_arch = "x86_64")]
        {
            let has_bmi1 = std::is_x86_feature_detected!("bmi1");
            let has_bmi2 = std::is_x86_feature_detected!("bmi2");
            let has_avx2 = std::is_x86_feature_detected!("avx2");
            if has_bmi2 {
                v.push(Arm::Bmi2Dec);
                v.push(Arm::Bmi2All);
                v.push(Arm::Bmi2Grp);
            }
            if has_bmi1 {
                v.push(Arm::Bmi1All);
            }
            if has_avx2 {
                v.push(Arm::Avx2All);
            }
            if has_avx2 && has_bmi1 {
                v.push(Arm::Avx2Bmi1);
            }
            if has_bmi1 && has_bmi2 {
                v.push(Arm::Bmi1Bmi2);
            }
            if has_avx2 && has_bmi1 && has_bmi2 {
                v.push(Arm::Avx2Bmi1Bmi2);
                v.push(Arm::Avx2TblBmi2Dec);
            }
        }
        v
    }
}

struct Scratch {
    tables: Box<[Huff; 6]>,
    syms: Vec<u16>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            tables: Box::new(std::array::from_fn(|_| Huff::default())),
            syms: vec![0u16; MAX_SYMS],
        }
    }
}

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

fn build_block_tables(arm: Arm, blk: &ParsedBlock, sc: &mut Scratch) {
    for g in 0..blk.n_groups {
        if (blk.used_groups >> g) & 1 == 0 {
            continue;
        }
        let lens = &blk.lens[g][..blk.alpha];
        match arm {
            Arm::Baseline => huff_build_baseline(&mut sc.tables[g], lens),
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi2Dec | Arm::Bmi2Grp => huff_build_baseline(&mut sc.tables[g], lens),
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi2All => {
                // SAFETY: `Arm::available_arms` checked `is_x86_feature_detected!("bmi2")`.
                unsafe { x86_arms::huff_build_bmi2(&mut sc.tables[g], lens) }
            }
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi1All => {
                // SAFETY: `Arm::available_arms` checked `is_x86_feature_detected!("bmi1")`.
                unsafe { x86_arms::huff_build_bmi1(&mut sc.tables[g], lens) }
            }
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2All => {
                // SAFETY: `Arm::available_arms` checked `is_x86_feature_detected!("avx2")`.
                unsafe { x86_arms::huff_build_avx2(&mut sc.tables[g], lens) }
            }
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2Bmi1 => {
                // SAFETY: `Arm::available_arms` checked `avx2 && bmi1`.
                unsafe { x86_arms::huff_build_avx2_bmi1(&mut sc.tables[g], lens) }
            }
            #[cfg(target_arch = "x86_64")]
            Arm::Bmi1Bmi2 => {
                // SAFETY: `Arm::available_arms` checked `bmi1 && bmi2`.
                unsafe { x86_arms::huff_build_bmi1_bmi2(&mut sc.tables[g], lens) }
            }
            #[cfg(target_arch = "x86_64")]
            Arm::Avx2Bmi1Bmi2 | Arm::Avx2TblBmi2Dec => {
                // SAFETY: `Arm::available_arms` checked `avx2 && bmi1 && bmi2`.
                unsafe { x86_arms::huff_build_avx2_bmi1_bmi2(&mut sc.tables[g], lens) }
            }
        }
    }
}

fn decode_block_syms(
    arm: Arm,
    blk: &ParsedBlock,
    data: &[u8],
    sc: &mut Scratch,
) -> Result<(usize, u64), ErrorKind> {
    match arm {
        Arm::Baseline => huff_dec_baseline(
            &sc.tables,
            &blk.selectors,
            data,
            blk.start_bit,
            &mut sc.syms,
        ),
        #[cfg(target_arch = "x86_64")]
        Arm::Bmi2Dec | Arm::Bmi2All | Arm::Avx2TblBmi2Dec => {
            // SAFETY: `Arm::available_arms` checked `bmi2`.
            unsafe {
                x86_arms::huff_dec_bmi2(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Bmi2Grp => {
            // SAFETY: `Arm::available_arms` checked `bmi2`.
            unsafe {
                x86_arms::huff_dec_bmi2_grp(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Bmi1All => {
            // SAFETY: `Arm::available_arms` checked `bmi1`.
            unsafe {
                x86_arms::huff_dec_bmi1(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Avx2All => {
            // SAFETY: `Arm::available_arms` checked `avx2`.
            unsafe {
                x86_arms::huff_dec_avx2(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Avx2Bmi1 => {
            // SAFETY: `Arm::available_arms` checked `avx2 && bmi1`.
            unsafe {
                x86_arms::huff_dec_avx2_bmi1(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Bmi1Bmi2 => {
            // SAFETY: `Arm::available_arms` checked `bmi1 && bmi2`.
            unsafe {
                x86_arms::huff_dec_bmi1_bmi2(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
        #[cfg(target_arch = "x86_64")]
        Arm::Avx2Bmi1Bmi2 => {
            // SAFETY: `Arm::available_arms` checked `avx2 && bmi1 && bmi2`.
            unsafe {
                x86_arms::huff_dec_avx2_bmi1_bmi2(
                    &sc.tables,
                    &blk.selectors,
                    data,
                    blk.start_bit,
                    &mut sc.syms,
                )
            }
        }
    }
}

// ============================================================================
// Independent canonical bit-by-bit parser & oracle
// ============================================================================

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

// ============================================================================
// Synthetic edge-case verification (sym=256, EOB, SUB_PTR, slice, decode_slow, BAD)
// ============================================================================

fn encode_canonical(lens: &[u8]) -> Vec<(u32, u8)> {
    let alpha = lens.len();
    let mn = *lens.iter().min().unwrap() as usize;
    let mx = *lens.iter().max().unwrap() as usize;
    let mut perm = Vec::with_capacity(alpha);
    for len in mn..=mx {
        for (s, &x) in lens.iter().enumerate() {
            if x as usize == len {
                perm.push(s);
            }
        }
    }
    let mut sym_code = vec![(0u32, 0u8); alpha];
    let mut idx = 0usize;
    let mut code = 0u32;
    for len in mn..=mx {
        while idx < alpha && lens[perm[idx]] as usize == len {
            sym_code[perm[idx]] = (code, len as u8);
            code += 1;
            idx += 1;
        }
        code <<= 1;
    }
    sym_code
}

fn pack_bits(start_bit: u64, codes: &[(u32, u8)], pad_bytes: usize) -> (Vec<u8>, u64) {
    let mut buf = Vec::new();
    let mut cur_byte = 0u8;
    let mut cur_bits = 0u32;
    for _ in 0..start_bit {
        cur_byte <<= 1;
        cur_bits += 1;
        if cur_bits == 8 {
            buf.push(cur_byte);
            cur_byte = 0;
            cur_bits = 0;
        }
    }
    for &(c, l) in codes {
        for b in (0..l).rev() {
            cur_byte = (cur_byte << 1) | (((c >> b) & 1) as u8);
            cur_bits += 1;
            if cur_bits == 8 {
                buf.push(cur_byte);
                cur_byte = 0;
                cur_bits = 0;
            }
        }
    }
    let end_bit = buf.len() as u64 * 8 + cur_bits as u64;
    if cur_bits > 0 {
        buf.push(cur_byte << (8 - cur_bits));
    }
    buf.resize(buf.len() + pad_bytes, 0);
    (buf, end_bit)
}

fn verify_synthetic_edge_cases(arms: &[Arm]) {
    let mut sc = Scratch::new();

    // Case 1 & 2: alpha=258, sym=256 (len=3), EOB=257 (len=4), len=12 SUB_PTR, k=48/49/50 edges.
    // Tested with both 300B padding (decode_group_win) and 0B padding (decode_group_slice).
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
    lens[257] = 4;

    let sym_code = encode_canonical(&lens);
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
    let codes: Vec<(u32, u8)> = syms.iter().map(|&s| sym_code[s as usize]).collect();

    for &(label, start_bit, pad_bytes) in &[("win_pad300", 0u64, 300usize), ("slice_pad0", 5u64, 0)]
    {
        let (buf, end_bit) = pack_bits(start_bit, &codes, pad_bytes);
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
            start_bit,
            end_bit,
            n_syms: syms.len(),
            expected_syms: syms.clone(),
        };
        for &arm in arms {
            build_block_tables(arm, &blk, &mut sc);
            let (n, eb) = decode_block_syms(arm, &blk, &buf, &mut sc)
                .unwrap_or_else(|e| panic!("synthetic {label} error {e:?} on {}", arm.name()));
            assert_eq!(n, blk.n_syms, "synthetic {label} len on {}", arm.name());
            assert_eq!(
                eb,
                blk.end_bit,
                "synthetic {label} end_bit on {}",
                arm.name()
            );
            assert_eq!(
                &sc.syms[..n],
                &blk.expected_syms[..],
                "synthetic {label} syms on {}",
                arm.name()
            );
        }
    }

    // Case 3: Kraft-exact 31-symbol code with two 10-bit prefixes reaching len=20:
    // lengths 1..=9 (sum 1 - 2^-9), then two subtrees each with 11..=20 + 20 (each 2^-10).
    // First prefix fills SUB_CAP=1024 (SUB_PTR); second prefix overflows SUB_CAP (decode_slow).
    let mut long_lens = [0u8; 31];
    for (i, l) in long_lens.iter_mut().enumerate().take(9) {
        *l = (i + 1) as u8;
    }
    for j in 0..10 {
        long_lens[9 + 2 * j] = (11 + j) as u8;
        long_lens[9 + 2 * j + 1] = (11 + j) as u8;
    }
    long_lens[29] = 20;
    long_lens[30] = 20; // EOB at len 20 (hits decode_slow)
    let long_codes = encode_canonical(&long_lens);
    let long_syms: Vec<u16> = (0..31u16).collect();
    let long_seq: Vec<(u32, u8)> = long_syms.iter().map(|&s| long_codes[s as usize]).collect();
    let (long_buf, long_end) = pack_bits(3, &long_seq, 300);
    let long_blk = ParsedBlock {
        alpha: 31,
        n_groups: 1,
        used_groups: 1,
        lens: {
            let mut a = [[0u8; MAX_ALPHA]; 6];
            a[0][..31].copy_from_slice(&long_lens);
            a
        },
        selectors: vec![0],
        start_bit: 3,
        end_bit: long_end,
        n_syms: long_syms.len(),
        expected_syms: long_syms,
    };
    for &arm in arms {
        build_block_tables(arm, &long_blk, &mut sc);
        let (n, eb) = decode_block_syms(arm, &long_blk, &long_buf, &mut sc)
            .unwrap_or_else(|e| panic!("synthetic len20 error {e:?} on {}", arm.name()));
        assert_eq!(n, long_blk.n_syms, "synthetic len20 len on {}", arm.name());
        assert_eq!(
            eb,
            long_blk.end_bit,
            "synthetic len20 end_bit on {}",
            arm.name()
        );
        assert_eq!(
            &sc.syms[..n],
            &long_blk.expected_syms[..],
            "synthetic len20 syms on {}",
            arm.name()
        );
    }

    // Case 4: incomplete / invalid code triggering BadHuffmanCode (all-1s bits past limit)
    let bad_lens = [2u8, 2, 2]; // 3 codes of length 2 -> prefix 0b11 is BAD
    let bad_buf = vec![0xffu8; 300];
    let bad_blk = ParsedBlock {
        alpha: 3,
        n_groups: 1,
        used_groups: 1,
        lens: {
            let mut a = [[0u8; MAX_ALPHA]; 6];
            a[0][..3].copy_from_slice(&bad_lens);
            a
        },
        selectors: vec![0],
        start_bit: 0,
        end_bit: 0,
        n_syms: 0,
        expected_syms: Vec::new(),
    };
    for &arm in arms {
        build_block_tables(arm, &bad_blk, &mut sc);
        let err = decode_block_syms(arm, &bad_blk, &bad_buf, &mut sc);
        assert_eq!(
            err,
            Err(ErrorKind::BadHuffmanCode),
            "synthetic BAD mismatch on {}",
            arm.name()
        );
    }
}

fn stats(v: &[f64]) -> (f64, f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    (s[0], s[s.len() / 2], s[s.len() - 1])
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .unwrap_or_else(|| "corpora/wikidata.bz2".to_string());
    let repeats: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(7);

    println!(
        "compile-time cfg(target_feature): bmi1={} bmi2={} avx2={}",
        cfg!(target_feature = "bmi1"),
        cfg!(target_feature = "bmi2"),
        cfg!(target_feature = "avx2"),
    );
    #[cfg(target_arch = "x86_64")]
    println!(
        "runtime CPU detection:            bmi1={} bmi2={} avx2={}",
        std::is_x86_feature_detected!("bmi1"),
        std::is_x86_feature_detected!("bmi2"),
        std::is_x86_feature_detected!("avx2"),
    );

    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let blocks = parse_blocks(&data);
    let total_syms: usize = blocks.iter().map(|b| b.n_syms).sum();
    let full_blocks = blocks.iter().filter(|b| b.n_syms >= 1000).count();
    let full_syms: usize = blocks
        .iter()
        .filter(|b| b.n_syms >= 1000)
        .map(|b| b.n_syms)
        .sum();
    let runt_blocks = blocks.len() - full_blocks;
    println!(
        "corpus: {path} ({} bytes, {} blocks: {} full / {} runts, {} symbols)",
        data.len(),
        blocks.len(),
        full_blocks,
        runt_blocks,
        total_syms
    );

    let arms = Arm::available_arms();
    verify_synthetic_edge_cases(&arms);

    let mut sc = Scratch::new();
    for &arm in &arms {
        for (bi, blk) in blocks.iter().enumerate() {
            build_block_tables(arm, blk, &mut sc);
            let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc)
                .unwrap_or_else(|e| panic!("blk {bi} error {e:?} on {}", arm.name()));
            assert_eq!(n, blk.n_syms, "blk {bi} len mismatch on {}", arm.name());
            assert_eq!(
                eb,
                blk.end_bit,
                "blk {bi} end_bit mismatch on {}",
                arm.name()
            );
            assert_eq!(
                &sc.syms[..n],
                &blk.expected_syms[..],
                "blk {bi} sym mismatch on {}",
                arm.name()
            );
        }
    }
    println!(
        "verify: PASS (all {} arms 100% symbol- and end-bit-identical on {} blocks, {} symbols, + 4 synthetic edge cases)",
        arms.len(),
        blocks.len(),
        total_syms
    );

    // 3 warmup passes
    for _ in 0..3 {
        for &arm in &arms {
            let mut chk = 0u64;
            for blk in &blocks {
                build_block_tables(arm, blk, &mut sc);
                let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc).unwrap();
                chk ^= (n as u64) ^ eb ^ (sc.syms[n - 1] as u64);
            }
            black_box(chk);
        }
    }

    // 1. Primary A3 timing (decode + tables, all 431 blocks, single timer, interleaved repeats)
    let mut a3_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
    for rep in 0..repeats {
        for off in 0..arms.len() {
            let ai = (rep + off) % arms.len();
            let arm = arms[ai];
            let t0 = Instant::now();
            let mut chk = 0u64;
            for blk in &blocks {
                build_block_tables(arm, blk, &mut sc);
                let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc).unwrap();
                chk ^= (n as u64) ^ eb ^ (sc.syms[n - 1] as u64);
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            black_box(chk);
            a3_ms[ai].push(ms);
        }
    }

    let (base_min, base_med, base_max) = stats(&a3_ms[0]);
    println!(
        "\n=== PRIMARY A3 (decode + tables, all {} blocks, {repeats} interleaved repeats, single timer) ===",
        blocks.len()
    );
    println!(
        "{:<16} {:>9} {:>9} {:>9} {:>9} {:>10} {:>10}",
        "arm", "min_ms", "med_ms", "max_ms", "vs_base%", "overlap?", "ns/sym"
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

    // 2. Per-stage breakdown (tables vs decode, full vs runts, interleaved repeats)
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
                let (n, eb) = decode_block_syms(arm, blk, &data, &mut sc).unwrap();
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

    let (_, base_tbl_med, _) = stats(&tbl_all[0]);
    let (_, base_dec_med, _) = stats(&dec_all[0]);
    println!("\n=== STAGE BREAKDOWN (medians over {repeats} interleaved repeats, ms) ===");
    println!(
        "{:<16} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9} {:>9}",
        "arm", "tbl_all", "tbl_vs%", "dec_all", "dec_vs%", "full_ns/s", "tbl_rnt", "dec_rnt"
    );
    for (ai, &arm) in arms.iter().enumerate() {
        let (_, ta, _) = stats(&tbl_all[ai]);
        let (_, da, _) = stats(&dec_all[ai]);
        let (_, df, _) = stats(&dec_full[ai]);
        let (_, tr, _) = stats(&tbl_runt[ai]);
        let (_, dr, _) = stats(&dec_runt[ai]);
        let full_ns_sym = df * 1e6 / full_syms as f64;
        let tbl_delta = (ta - base_tbl_med) / base_tbl_med * 100.0;
        let dec_delta = (da - base_dec_med) / base_dec_med * 100.0;
        println!(
            "{:<16} {:>8.3} {:>+7.2}% {:>8.3} {:>+7.2}% {:>9.3} {:>9.3} {:>9.3}",
            arm.name(),
            ta,
            tbl_delta,
            da,
            dec_delta,
            full_ns_sym,
            tr,
            dr
        );
    }

    // 3. Nostore chain diagnostic on 233 full blocks (isolates pure shift + LUT + refill chain)
    #[allow(clippy::type_complexity)]
    let mut ns_arms: Vec<(
        &'static str,
        fn(&[Huff; 6], &[u8], &[u8], u64) -> Result<(usize, u64, u64), ErrorKind>,
    )> = vec![("baseline", huff_nostore_baseline)];
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("bmi2") {
            ns_arms.push(("bmi2", |t, s, d, b| unsafe {
                x86_arms::huff_nostore_bmi2(t, s, d, b)
            }));
        }
        if std::is_x86_feature_detected!("bmi1") {
            ns_arms.push(("bmi1", |t, s, d, b| unsafe {
                x86_arms::huff_nostore_bmi1(t, s, d, b)
            }));
        }
        if std::is_x86_feature_detected!("avx2") {
            ns_arms.push(("avx2", |t, s, d, b| unsafe {
                x86_arms::huff_nostore_avx2(t, s, d, b)
            }));
        }
        if std::is_x86_feature_detected!("avx2")
            && std::is_x86_feature_detected!("bmi1")
            && std::is_x86_feature_detected!("bmi2")
        {
            ns_arms.push(("avx2_bmi1_bmi2", |t, s, d, b| unsafe {
                x86_arms::huff_nostore_avx2_bmi1_bmi2(t, s, d, b)
            }));
        }
    }

    let mut ns_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); ns_arms.len()];
    for rep in 0..repeats {
        for off in 0..ns_arms.len() {
            let ai = (rep + off) % ns_arms.len();
            let (_, f) = ns_arms[ai];
            let mut dec_only_ms = 0.0f64;
            let mut chk = 0u64;
            for blk in &blocks {
                if blk.n_syms < 1000 {
                    continue;
                }
                build_block_tables(Arm::Baseline, blk, &mut sc);
                let t0 = Instant::now();
                let (n, eb, acc) = f(&sc.tables, &blk.selectors, &data, blk.start_bit).unwrap();
                dec_only_ms += t0.elapsed().as_secs_f64() * 1e3;
                chk ^= (n as u64) ^ eb ^ acc;
            }
            black_box(chk);
            ns_ms[ai].push(dec_only_ms);
        }
    }
    let (ns_base_min, ns_base_med, ns_base_max) = stats(&ns_ms[0]);
    println!(
        "\n=== NOSTORE DIAGNOSTIC (decode-only, {full_blocks} full blocks, {full_syms} syms, {repeats} interleaved repeats) ==="
    );
    println!(
        "{:<16} {:>9} {:>9} {:>9} {:>9} {:>10} {:>10}",
        "arm", "min_ms", "med_ms", "max_ms", "vs_base%", "overlap?", "ns/sym"
    );
    for (ai, &(name, _)) in ns_arms.iter().enumerate() {
        let (mn, med, mx) = stats(&ns_ms[ai]);
        let delta = (med - ns_base_med) / ns_base_med * 100.0;
        let overlap = if ai == 0 {
            "-"
        } else if mx < ns_base_min || mn > ns_base_max {
            "NO"
        } else {
            "YES"
        };
        println!(
            "{:<16} {:>9.3} {:>9.3} {:>9.3} {:>+8.2}% {:>10} {:>10.3}",
            name,
            mn,
            med,
            mx,
            delta,
            overlap,
            med * 1e6 / full_syms as f64
        );
    }
}
