#![forbid(unsafe_code)]
//! L1d-resident canonical Huffman decode table (`4 KiB` 10-bit byte-aligned root LUT + `2 KiB`
//! `O(1)` `SubLut` for `11..=20`-bit codes + runt-gated Canonical Template-Clone 2-symbol pair
//! builder + chunked coalesced-run decoder).

use super::bits::{Bits, Reg};
use super::ErrorKind;

pub(crate) const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23; // (the arrays' size; codes are at most 20 bits)
pub(crate) const GROUP: usize = 50;

pub(crate) const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

/// Root LUT entry layout (byte-aligned 8-bit symbols):
/// - bits `30..31` (`CNT_SH`): symbol count (`1` or `2` for non-EOB, `<= 255` fast entries;
///   `0` for `EOB_FAST` / `BAD` / `SUB_PTR` / `SYM256` / `0` (`> 10`-bit slow))
/// - when `cnt != 0`:
///   - bits `24..28` (`LEN1`): 1st symbol bit length (`1..=10`)
///   - bits `16..23` (`SYM2`): 2nd symbol (`0..=255`)
///   - bits `8..15`  (`SYM1`): 1st symbol (`0..=255`)
///   - bits `0..7`: total bits consumed (`1..=10`, bits `4..7 == 0` so `skip_short_shl_u8`
///     avoids `and $0x3f` on the shift chain)
/// - when `cnt == 0`:
///   - bit `29` (`EOB_FAST`): `<= 10`-bit EOB sentinel (low 6 bits = `zn`)
///   - bit `28` (`BAD`): invalid/oversubscribed prefix
///   - bit `27` (`SUB_PTR`): 2nd-level `SubLut` pointer (`d_w` in `16..19`, `sub_off` in `0..15`)
///   - bit `26` (`SYM256`): `<= 10`-bit literal symbol `256` (`alpha == 258` only; low 6 bits = `zn`)
pub(crate) const SYM256: u32 = 1 << 26;
pub(crate) const SUB_PTR: u32 = 1 << 27;
pub(crate) const BAD: u32 = 1 << 28;
pub(crate) const EOB_FAST: u32 = 1 << 29;
pub(crate) const SYM1: u32 = 8;
pub(crate) const SYM2: u32 = 16;
pub(crate) const LEN1: u32 = 24;
pub(crate) const CNT_SH: u32 = 30;
const STEPS: usize = 5;

/// Fixed L1d-hot chunk buffer size (`2 KiB`) for [`Huff::decode_run`]: `ck` persists across
/// coalesced runs and flushes to `syms` only once `ck > CHUNK_STOP` or on `EOB`, cutting
/// `extend_from_slice` calls by ~8x while keeping `ck & CHUNK_MASK` wrap-free and
/// `idx + 2 < CHUNK_LEN` (zero per-step bounds checks, zero `syms.resize` memset, safe Rust).
pub(crate) const CHUNK_LEN: usize = 1042;
const CHUNK_MASK: usize = 1023;
const CHUNK_STOP: usize = 1000;

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
        h.build_into(lens, true);
        h
    }

    /// The builder `build_into` replaced (two-pass: `K=1` fill, then `K=2` overwrite, with a
    /// full `d_arr`), kept to check the single-write builder bit for bit.
    #[cfg(test)]
    pub(crate) fn build_into_reference(&mut self, lens: &[u8], build_pairs: bool) {
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
                    Some(256) => SYM256 | zn,
                    Some(s) => (1u32 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn,
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
                let s1 = (e0 >> SYM1) & 0xff;
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
                            | (zn << LEN1)
                            | (((e2 >> SYM1) & 0xff) << SYM2)
                            | (e & (0xff << SYM1))
                            | (zn + l2 as u32);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    pub(crate) fn build_into(&mut self, lens: &[u8], build_pairs: bool) {
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

        self.fill_lut(min_l as u32, max_l, build_pairs);
    }

    /// The root LUT and `SubLut` from `limit`/`base`/`perm`, every root entry written once:
    /// a pairable first symbol's span gets its `K=2` pairs and `K=1` remainder directly
    /// (the first literal of each length builds it, the rest clone it with a symbol delta),
    /// and only the `1024 - filled_to` tail past the `<= 10`-bit codes is touched by the
    /// long-code `SubLut` pass. Bit-identical to [`Huff::build_into_reference`].
    fn fill_lut(&mut self, min_len: u32, max_l: usize, build_pairs: bool) {
        let alpha = self.alpha;
        let eob = self.eob;
        let max_pair_l1 = if build_pairs {
            LUT_BITS.saturating_sub(min_len)
        } else {
            0
        };
        // canonical code ranges `[lo, hi]` of each length `<= 10`, and where they end
        let mut zn_ranges = [(0i64, -1i64); 11];
        let mut a = 0i64;
        let mut filled_to = 0usize;
        for zn in min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            if hi >= a {
                filled_to = ((hi + 1) << (LUT_BITS - zn)) as usize;
            }
            a = 2 * a.max(hi + 1);
        }
        let k1 = |s: Option<u16>, zn: u32| -> u32 {
            match s {
                Some(s) if s == eob => EOB_FAST | zn,
                Some(256) => SYM256 | zn,
                Some(s) => (1u32 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn,
                None => BAD,
            }
        };
        for zn in min_len..=LUT_BITS {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = (LUT_BITS - zn) as usize;
            let span = 1usize << sh;
            let base_zn = self.base[zn as usize];
            let pairable = zn <= max_pair_l1;
            let max_l2 = LUT_BITS - zn;
            let mut tpl: Option<(usize, u32)> = None;
            for p in lo_p..=hi_p {
                let off_p = (p as usize) << sh;
                let idx = (p as i32) - base_zn;
                let s = (0..alpha).contains(&idx).then(|| self.perm[idx as usize]);
                let e = k1(s, zn);
                if !pairable || (e >> CNT_SH) == 0 {
                    self.lut[off_p..off_p + span].fill(e);
                    continue;
                }
                let s1 = (e >> SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    // `perm` is ascending within a length, so `s0 < s1 <= 255`: the delta
                    // never carries out of the `SYM1` byte
                    let delta = s1.wrapping_sub(s0) << SYM1;
                    let (left, right) = self.lut.split_at_mut(off_p);
                    for (dst, &src) in right[..span].iter_mut().zip(&left[off0..off0 + span]) {
                        *dst = src.wrapping_add(delta);
                    }
                    continue;
                }
                let pair_base = (2u32 << CNT_SH) | (zn << LEN1) | (s1 << SYM1);
                let mut sub_pos = 0usize;
                for l2 in min_len..=max_l2 {
                    let (lo2, hi2) = zn_ranges[l2 as usize];
                    if lo2 > hi2 {
                        continue;
                    }
                    let sh2 = (max_l2 - l2) as usize;
                    let base2 = self.base[l2 as usize];
                    for p2 in lo2..=hi2 {
                        let idx2 = (p2 as i32) - base2;
                        let e2 = k1(
                            (0..alpha).contains(&idx2).then(|| self.perm[idx2 as usize]),
                            l2,
                        );
                        let ep = if (e2 >> CNT_SH) != 0 {
                            pair_base | (((e2 >> SYM1) & 0xff) << SYM2) | (zn + l2)
                        } else {
                            e
                        };
                        let start = off_p + ((p2 as usize) << sh2);
                        self.lut[start..start + (1usize << sh2)].fill(ep);
                        sub_pos = ((p2 as usize) + 1) << sh2;
                    }
                }
                self.lut[off_p + sub_pos..off_p + span].fill(e);
                tpl = Some((off_p, s1));
            }
        }

        if filled_to >= LUT_LEN {
            return;
        }
        if max_l <= LUT_BITS as usize {
            self.lut[filled_to..].fill(BAD);
            return;
        }
        self.lut[filled_to..].fill(0);
        // Each root index `w` past `filled_to` gets a `SubLut` of `2^d` entries, `d` the
        // widest overhang of the long codes under it. The roots each length covers ascend
        // and meet only at one shared boundary root (where the longer length wins), so the
        // pointers are assigned in one ascending pass with a one-root lookahead, no `d_arr`.
        let mut ranges = [(0u32, 0i64, -1i64); 10];
        let mut sub_used = 0usize;
        let mut pending: Option<(usize, u32)> = None;
        let mut assign = |lut: &mut [u32; LUT_LEN], w: usize, d_w: u32| {
            if w >= filled_to && sub_used + (1usize << d_w) <= SUB_CAP {
                lut[w] = SUB_PTR | (d_w << 16) | (sub_used as u32);
                sub_used += 1usize << d_w;
            }
        };
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
                for w in w_lo..=w_hi {
                    match pending {
                        Some((pw, _)) if pw == w => {}
                        Some((pw, pd)) => assign(&mut self.lut, pw, pd),
                        None => {}
                    }
                    pending = Some((w, sh));
                }
            }
            a_hi = 2 * a_hi.max(hi + 1);
        }
        if let Some((pw, pd)) = pending {
            assign(&mut self.lut, pw, pd);
        }
        if sub_used == 0 {
            return;
        }
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
                self.sub_lut[sub_off + local..sub_off + local + (1usize << pad)].fill(se);
            }
        }
    }

    /// Every field equal, bit for bit (the `SubLut` whole, stale entries included)
    #[cfg(test)]
    pub(crate) fn identical(&self, o: &Huff) -> bool {
        self.lut == o.lut
            && self.sub_lut == o.sub_lut
            && self.limit == o.limit
            && self.base == o.base
            && self.perm == o.perm
            && self.min_len == o.min_len
            && self.alpha == o.alpha
            && self.eob == o.eob
    }

    #[inline(always)]
    pub(crate) fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    #[inline(always)]
    pub(crate) fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return Ok(((e >> SYM1) as u8) as u16);
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// Decode up to `target` symbols (`target = m * GROUP >= GROUP`) using this table,
    /// staging into the L1d-hot fixed `chunk` buffer starting at `*ck_out <= CHUNK_STOP` and
    /// flushing to `syms` only when `ck > CHUNK_STOP` or on `EOB`. Returns `Ok(true)` if
    /// `eob` was decoded (with all symbols flushed to `syms`), `Ok(false)` once all `target`
    /// symbols were staged/appended (maintaining `*ck_out <= CHUNK_STOP`).
    #[inline(always)]
    pub(crate) fn decode_run(
        &self,
        r_out: &mut Reg,
        data: &[u8],
        chunk: &mut [u16; CHUNK_LEN],
        ck_out: &mut usize,
        syms: &mut Vec<u16>,
        target: usize,
    ) -> Result<bool, ErrorKind> {
        debug_assert!(target >= GROUP && *ck_out <= CHUNK_STOP);
        let mut r = *r_out;
        let mut ck = *ck_out;
        let eob = self.eob;
        let mut rem_syms = target;

        r.refill(data);
        while rem_syms >= 2 * STEPS {
            let stop_ck = (ck + (rem_syms - 2 * STEPS)).min(CHUNK_STOP);
            let before_ck = ck;
            while ck <= stop_ck {
                let batch: &mut [u16; 18] = (&mut chunk[ck..ck + 18]).try_into().unwrap();
                let mut bk = 0usize;
                for _ in 0..STEPS {
                    let e = self.lut[r.peek_live(LUT_BITS) as usize];
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        r.skip_short_shl_u8(e);
                        let idx = bk & 15;
                        let dst = &mut batch[idx..idx + 2];
                        dst[0] = ((e >> SYM1) as u8) as u16;
                        dst[1] = ((e >> SYM2) as u8) as u16;
                        bk += c;
                    } else {
                        let (s, after) = self.decode_entry_slow(e, r, data);
                        r = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => {
                                *ck_out = ck + bk;
                                *r_out = r;
                                return Err(err);
                            }
                        };
                        batch[bk & 15] = s;
                        bk += 1;
                        if s == eob {
                            syms.extend_from_slice(&chunk[..ck + bk]);
                            *ck_out = 0;
                            *r_out = r;
                            return Ok(true);
                        }
                    }
                }
                ck += bk;
                r.refill(data);
            }
            rem_syms -= ck - before_ck;
            if ck > CHUNK_STOP {
                syms.extend_from_slice(&chunk[..ck]);
                ck = 0;
            }
        }

        if rem_syms > 0 {
            let target_ck = ck + rem_syms;
            macro_rules! step_tail {
                () => {{
                    let e = self.lut[r.peek_live(LUT_BITS) as usize];
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        r.skip_short_shl_u8(e);
                        let idx = ck & CHUNK_MASK;
                        let dst = &mut chunk[idx..idx + 2];
                        dst[0] = ((e >> SYM1) as u8) as u16;
                        dst[1] = ((e >> SYM2) as u8) as u16;
                        ck += c;
                    } else {
                        let (s, after) = self.decode_entry_slow(e, r, data);
                        r = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => {
                                *ck_out = ck;
                                *r_out = r;
                                return Err(err);
                            }
                        };
                        chunk[ck & CHUNK_MASK] = s;
                        ck += 1;
                        if s == eob {
                            syms.extend_from_slice(&chunk[..ck]);
                            *ck_out = 0;
                            *r_out = r;
                            return Ok(true);
                        }
                    }
                }};
            }

            if rem_syms >= 2 {
                let tail_stop = target_ck - 2;
                for _ in 0..4 {
                    if ck > tail_stop {
                        break;
                    }
                    step_tail!();
                }
                if ck <= tail_stop {
                    r.refill(data);
                    while ck <= tail_stop {
                        step_tail!();
                    }
                }
            }
            let last = target_ck - 1;
            if ck == last {
                let s = match self.decode(&mut r, data) {
                    Ok(s) => s,
                    Err(err) => {
                        *ck_out = ck;
                        *r_out = r;
                        return Err(err);
                    }
                };
                chunk[last & CHUNK_MASK] = s;
                ck += 1;
                if s == eob {
                    syms.extend_from_slice(&chunk[..ck]);
                    *ck_out = 0;
                    *r_out = r;
                    return Ok(true);
                }
            }
            if ck > CHUNK_STOP {
                syms.extend_from_slice(&chunk[..ck]);
                ck = 0;
            }
        }
        *ck_out = ck;
        *r_out = r;
        Ok(false)
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
        } else if (e & SYM256) != 0 {
            r.skip_short(e & 63);
            return (Ok(256), r);
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
