#![forbid(unsafe_code)]
//! L1d-resident canonical Huffman decode table (`4 KiB` 10-bit root LUT + `2 KiB` `O(1)`
//! `SubLut` for `11..=20`-bit codes + Canonical Template-Clone 2-symbol pair builder +
//! `264 B` group-window decoder).

use super::bits::{Bits, Reg};
use super::ErrorKind;

pub(crate) const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23; // (the arrays' size; codes are at most 20 bits)
pub(crate) const GROUP: usize = 50;

pub(crate) const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

/// Root LUT entry layout (`C36a`/`C37`):
/// - bits `30..31` (`CNT_SH`): symbol count (`1` or `2` for non-EOB fast entries; `0` for `EOB_FAST` / `SUB_PTR` / `BAD` / slow)
/// - bit `29` (`EOB_FAST`): `<= 10`-bit EOB sentinel (low 6 bits = `zn`)
/// - bit `28` (`BAD`): invalid/oversubscribed prefix
/// - bit `27` (`SUB_PTR`): 2nd-level `SubLut` pointer (`d_w` in `16..19`, `sub_off` in `0..15`)
/// - bits `20..28` (`SYM2`): 2nd symbol (`0..258`)
/// - bits `11..19` (`SYM1`): 1st symbol (`0..258`)
/// - bits `6..10` (`LEN1`): 1st symbol bit length (`1..=10`)
/// - bits `0..5`: total bits consumed (`1..=10`, bits `4..5 == 0` so `wrapping_shl(e)` needs no `& 31` mask)
pub(crate) const SUB_PTR: u32 = 1 << 27;
pub(crate) const BAD: u32 = 1 << 28;
pub(crate) const EOB_FAST: u32 = 1 << 29;
pub(crate) const LEN1: u32 = 6;
pub(crate) const SYM1: u32 = 11;
pub(crate) const SYM2: u32 = 20;
pub(crate) const CNT_SH: u32 = 30;
const STEPS: usize = 5;

pub(crate) struct Huff {
    pub(crate) lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    pub(crate) limit: [i32; MAX_CODE_LEN],
    pub(crate) base: [i32; MAX_CODE_LEN],
    pub(crate) perm: [u16; MAX_ALPHA],
    pub(crate) min_len: u32,
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
    #[cfg(test)]
    pub(crate) fn new(lens: &[u8]) -> Huff {
        let mut h = Huff::default();
        h.build_into(lens);
        h
    }

    pub(crate) fn build_into(&mut self, lens: &[u8]) {
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
    pub(crate) fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
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
    pub(crate) fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
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
    pub(crate) fn decode_group_win(
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
    pub(crate) fn decode_group_slice(
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
            let s = self.decode(r, data)?;
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
        let mut br = Bits::of(full_data, abs_r);
        let s = self.decode_slow(&mut br);
        let mut out_r = br.parts().1;
        out_r.pos -= base_pos;
        out_r.refill_win(win);
        (s, out_r)
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
        let mut br = Bits::of(data, r);
        let s = self.decode_slow(&mut br);
        br.refill();
        (s, br.parts().1)
    }

    #[cold]
    pub(crate) fn decode_slow(&self, br: &mut Bits) -> Result<u16, ErrorKind> {
        let mut zn = self.min_len;
        let mut zvec = br.peek(zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return Err(ErrorKind::BadHuffmanCode);
            }
            zvec = br.peek(zn) as i32;
        }
        br.skip(zn);
        self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode)
    }
}
