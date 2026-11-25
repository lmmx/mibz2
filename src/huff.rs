#![forbid(unsafe_code)]
//! L1d-resident canonical Huffman decode table (`8 KiB` 10-bit root LUT + `2 KiB` `O(1)`
//! `SubLut` for `11..=20`-bit codes + Canonical Template-Clone builder of entries of up to
//! three symbols + `264 B` group-window decoder).

use super::bits::{Bits, Reg};
use super::ErrorKind;

pub(crate) const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23; // (the arrays' size; codes are at most 20 bits)
pub(crate) const GROUP: usize = 50;

pub(crate) const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

/// Root LUT entry layout (`u64`). A fast entry (count `1..=3`) holds the symbols of that
/// many whole codes that the entry's prefix begins with, all within its `LUT_BITS` bits:
/// - bits `62..63` (`CNT_SH`): symbol count (`1..=3` for fast entries; `0` for `EOB_FAST` / `SUB_PTR` / `BAD` / slow)
/// - bits `8..23` (`SYM1`), `24..39` (`SYM2`), `40..55` (`SYM3`): the symbols, a 16-bit lane
///   each, so that `e >> SYM1` stores them into the output as they are, in one store
/// - bits `0..5`: total bits consumed (`1..=10`, bits `4..5 == 0` so `wrapping_shl(e)` needs no `& 31` mask)
///
/// A slow entry (count `0`):
/// - bit `29` (`EOB_FAST`): `<= 10`-bit EOB sentinel (low 6 bits = `zn`)
/// - bit `28` (`BAD`): invalid/oversubscribed prefix
/// - bit `27` (`SUB_PTR`): 2nd-level `SubLut` pointer (`d_w` in `16..19`, `sub_off` in `0..15`)
/// - none of these: decoded bit by bit (`decode_slow`)
///
/// The symbols' own lengths are `Huff::len_of`'s, for a group's last step, which may take
/// fewer of an entry's symbols than it holds.
pub(crate) const SUB_PTR: u64 = 1 << 27;
pub(crate) const BAD: u64 = 1 << 28;
pub(crate) const EOB_FAST: u64 = 1 << 29;
pub(crate) const SYM1: u32 = 8;
pub(crate) const SYM2: u32 = 24;
pub(crate) const SYM3: u32 = 40;
pub(crate) const SYM_MASK: u64 = 0xffff;
pub(crate) const CNT_SH: u32 = 62;
/// A group's output buffer: a group's 50 symbols, and room for an entry's store (four
/// lanes) at any `k & 63`
pub(crate) const G_OUT: usize = 68;
const STEPS: usize = 5;

pub(crate) struct Huff {
    pub(crate) lut: [u64; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    pub(crate) limit: [i32; MAX_CODE_LEN],
    pub(crate) base: [i32; MAX_CODE_LEN],
    pub(crate) perm: [u16; MAX_ALPHA],
    pub(crate) min_len: u32,
    /// each symbol's code length (room for any 9-bit index, so that a slow entry's bits
    /// read as a symbol index the builder then rejects)
    pub(crate) len_of: [u8; 512],
    alpha: i32,
    eob: u16,
    /// the main loop's bound: it runs a batch of `STEPS` whole entries only while at least
    /// this many of the group's symbols are left (`STEPS` times the most an entry holds)
    batch_left: usize,
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
            len_of: [0; 512],
            alpha: 0,
            eob: 0,
            batch_left: STEPS,
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
        self.len_of[..alpha].copy_from_slice(lens);
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
                    Some(s) if s == eob => EOB_FAST | zn as u64,
                    Some(s) => (1u64 << CNT_SH) | ((s as u64) << SYM1) | zn as u64,
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
                        self.lut[filled_to + w] =
                            SUB_PTR | ((d_w as u64) << 16) | (sub_used as u64);
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
                            let d_w = ((e_root >> 16) & 15) as u32;
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

        // Entries of two and three symbols. Every entry whose first code is `zn` bits long
        // has the same continuation, the codes the remaining `LUT_BITS - zn` bits begin
        // with, so it is worked out for the first such entry (the template) and copied to
        // the others with their own first symbol. An entry's first symbol is the same
        // whatever it has become, so the continuations read it from any entry.
        let mask = LUT_LEN - 1;
        let len_of = &self.len_of;
        let mut most = 1u64;
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
            let mut tpl: Option<(usize, u64)> = None;
            for p in lo_p..=hi_p {
                let off_p = (p as usize) << sh;
                let e0 = self.lut[off_p];
                if (e0 >> CNT_SH) == 0 {
                    continue;
                }
                let s1 = (e0 >> SYM1) & SYM_MASK;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << SYM1;
                    for u in 0..u_lim {
                        self.lut[off_p + u] = self.lut[off0 + u].wrapping_add(delta);
                    }
                } else {
                    for u in 0..u_lim {
                        let x = (u << zn) & mask;
                        let e2 = self.lut[x];
                        if (e2 >> CNT_SH) == 0 {
                            continue;
                        }
                        let s2 = (e2 >> SYM1) & SYM_MASK;
                        let t2 = zn as u64 + len_of[s2 as usize] as u64;
                        if t2 > LUT_BITS as u64 {
                            continue;
                        }
                        let pair = (s1 << SYM1) | (s2 << SYM2);
                        let e3 = self.lut[(u << t2) & mask];
                        let s3 = (e3 >> SYM1) & SYM_MASK;
                        self.lut[off_p + u] = match t2 + len_of[s3 as usize & 0x1ff] as u64 {
                            t3 if (e3 >> CNT_SH) != 0 && t3 <= LUT_BITS as u64 => {
                                (3u64 << CNT_SH) | pair | (s3 << SYM3) | t3
                            }
                            _ => (2u64 << CNT_SH) | pair | t2,
                        };
                    }
                    tpl = Some((off_p, s1));
                    // (the entries copied from the template hold as many symbols as it)
                    for &e in &self.lut[off_p..off_p + u_lim] {
                        most = most.max(e >> CNT_SH);
                    }
                }
            }
        }
        self.batch_left = most as usize * STEPS;
    }

    #[inline(always)]
    pub(crate) fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// One symbol (the tests' reference for a table's single steps)
    #[cfg(test)]
    pub(crate) fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            let s = ((e >> SYM1) & SYM_MASK) as u16;
            r.skip_short(self.len_of[s as usize] as u32);
            return Ok(s);
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// A group's 50 symbols (fewer if it holds the block's end) into `g_out`, through a
    /// 264-byte window of the input at `base_pos`. The steps take a whole entry (up to
    /// three symbols) in batches of five while a batch cannot pass the group's end
    /// (`batch_left`), then one at a time, as many of an entry's symbols as the group
    /// still wants, so no code past the group's end is ever read with this group's table.
    #[inline(always)]
    pub(crate) fn decode_group_win(
        &self,
        r: &mut Reg,
        g_in: &[u8; 264],
        data: &[u8],
        base_pos: usize,
        g_out: &mut [u16; G_OUT],
    ) -> Result<(usize, bool), ErrorKind> {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_win {
            ($clamp:expr, $acc:ident) => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    let idx = k & 63;
                    let v = e >> SYM1;
                    g_out[idx..idx + 4].copy_from_slice(&[
                        v as u16,
                        (v >> 16) as u16,
                        (v >> 32) as u16,
                        (v >> 48) as u16,
                    ]);
                    if $clamp && k + c > GROUP {
                        r.live -= $acc & 63;
                        $acc = 0;
                        k += self.take_some(r, e, GROUP - k);
                    } else {
                        // (the bits consumed are summed in `$acc` and taken from `live`
                        // once a batch, keeping the mask off the shift's chain)
                        r.buf = r.buf.wrapping_shl(e as u32);
                        $acc = $acc.wrapping_add(e as u32);
                        k += c;
                    }
                } else {
                    r.live -= $acc & 63;
                    $acc = 0;
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
            let mut acc = 0u32;
            for _ in 0..STEPS {
                step_win!(false, acc);
            }
            r.live -= acc & 63;
            if k + self.batch_left > GROUP {
                break;
            }
            r.refill_win(g_in);
        }
        while k < GROUP {
            r.refill_win(g_in);
            let mut acc = 0u32;
            for _ in 0..STEPS {
                if k >= GROUP {
                    break;
                }
                step_win!(true, acc);
            }
            r.live -= acc & 63;
        }
        r.pos += base_pos;
        Ok((GROUP, false))
    }

    /// The first `n` (fewer than its count) of fast entry `e`'s symbols taken: their bits
    /// consumed. Returns `n`.
    #[inline(always)]
    fn take_some(&self, r: &mut Reg, e: u64, n: usize) -> usize {
        let mut bits = self.len_of[((e >> SYM1) & SYM_MASK) as usize] as u32;
        if n == 2 {
            bits += self.len_of[((e >> SYM2) & SYM_MASK) as usize] as u32;
        }
        r.skip_short(bits);
        n
    }

    /// `decode_group_win` near the input's end, on the input itself
    #[inline(always)]
    pub(crate) fn decode_group_slice(
        &self,
        r: &mut Reg,
        data: &[u8],
        g_out: &mut [u16; G_OUT],
    ) -> Result<(usize, bool), ErrorKind> {
        let eob = self.eob;
        let mut k = 0usize;
        macro_rules! step_slice {
            ($clamp:expr, $acc:ident) => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    let idx = k & 63;
                    let v = e >> SYM1;
                    g_out[idx..idx + 4].copy_from_slice(&[
                        v as u16,
                        (v >> 16) as u16,
                        (v >> 32) as u16,
                        (v >> 48) as u16,
                    ]);
                    if $clamp && k + c > GROUP {
                        r.live -= $acc & 63;
                        $acc = 0;
                        k += self.take_some(r, e, GROUP - k);
                    } else {
                        // (the bits consumed are summed in `$acc` and taken from `live`
                        // once a batch, keeping the mask off the shift's chain)
                        r.buf = r.buf.wrapping_shl(e as u32);
                        $acc = $acc.wrapping_add(e as u32);
                        k += c;
                    }
                } else {
                    r.live -= $acc & 63;
                    $acc = 0;
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
            let mut acc = 0u32;
            for _ in 0..STEPS {
                step_slice!(false, acc);
            }
            r.live -= acc & 63;
            if k + self.batch_left > GROUP {
                break;
            }
            r.refill(data);
        }
        while k < GROUP {
            r.refill(data);
            let mut acc = 0u32;
            for _ in 0..STEPS {
                if k >= GROUP {
                    break;
                }
                step_slice!(true, acc);
            }
            r.live -= acc & 63;
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
        e: u64,
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
            let d_w = ((e >> 16) & 15) as u32;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill_win(win);
                return (Ok(se >> 6), r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short((e & 63) as u32);
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
    fn decode_entry_slow(&self, e: u64, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = ((e >> 16) & 15) as u32;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se != SUB_BAD_U16 {
                r.skip_short((se & 31) as u32);
                r.refill(data);
                return (Ok(se >> 6), r);
            }
        } else if (e & EOB_FAST) != 0 {
            r.skip_short((e & 63) as u32);
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
