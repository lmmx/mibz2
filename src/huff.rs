#![forbid(unsafe_code)]
//! L1d-resident canonical Huffman decode table (`4 KiB` 10-bit root LUT of up to three
//! symbols an entry + `2 KiB` `O(1)` `SubLut` for `11..=20`-bit codes + runt-gated
//! template-clone pair and triple builders + chunked coalesced-run decoder staging symbols
//! as bytes).

use super::bits::{Bits, Reg};
use super::ErrorKind;

pub(crate) const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23; // (the arrays' size; codes are at most 20 bits)
pub(crate) const GROUP: usize = 50;

pub(crate) const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

/// Root LUT entry layout:
/// - bits `30..31` (`CNT_SH`): symbol count (`1`, `2` or `3` symbols, each `<= 255` and none
///   the block's end; `0` for `EOB_FAST` / `BAD` / `SUB_PTR` / `SYM256` / `0` (`> 10`-bit slow))
/// - when `cnt != 0`:
///   - bits `22..29` (`SYM3`): 3rd symbol, bits `14..21` (`SYM2`): 2nd, bits `6..13`
///     (`SYM1`): 1st (`0..=255`), so `e >> SYM1` holds the symbols in its low bytes (one
///     32-bit store writes all of them; bytes past `cnt` are overwritten by the next step)
///   - bits `4..5`: zero, so `e & 63` is the total bits a shift by `e` consumes (no `and`
///     on the shift chain; [`Reg::skip_short_shl_low4`])
///   - bits `0..3`: total bits consumed (`1..=10`); the 1st symbol's own length is
///     [`Huff::sym_len`]
/// - when `cnt == 0`:
///   - bit `29` (`EOB_FAST`): `<= 10`-bit EOB sentinel (low 6 bits = `zn`)
///   - bit `28` (`BAD`): invalid/oversubscribed prefix
///   - bit `27` (`SUB_PTR`): 2nd-level `SubLut` pointer (`d_w` in `16..19`, `sub_off` in `0..15`)
///   - bit `26` (`SYM256`): `<= 10`-bit literal symbol `256` (`alpha == 258` only; low 6 bits = `zn`)
pub(crate) const SYM256: u32 = 1 << 26;
pub(crate) const SUB_PTR: u32 = 1 << 27;
pub(crate) const BAD: u32 = 1 << 28;
pub(crate) const EOB_FAST: u32 = 1 << 29;
pub(crate) const SYM1: u32 = 6;
pub(crate) const SYM2: u32 = 14;
pub(crate) const SYM3: u32 = 22;
pub(crate) const CNT_SH: u32 = 30;
/// Steps of a batch before its last, whose lookup the refill follows (see `decode_run`)
const PRE_STEPS: usize = 3;
/// The most symbols one batch stages: `PRE_STEPS + 1` lookups of up to 3
const BATCH_SYMS: usize = 3 * (PRE_STEPS + 1);
/// Lookups of a batch on a table of single-symbol entries, refilled after its last
const SINGLE_STEPS: usize = 5;
// A refill leaves at least 56 bits live and a lookup takes at most 10: the pipelined batch
// starts with at least 46 and must keep 10 for its last lookup's peek (so at most 4 lookups
// a refill), and the single-symbol batch must keep 10 for its last.
// (the batches below write their `PRE_STEPS + 1` and `SINGLE_STEPS` steps out by hand)
const _: () = assert!(56 - LUT_BITS as usize >= LUT_BITS as usize * (PRE_STEPS + 1));
const _: () = assert!(PRE_STEPS == 3 && SINGLE_STEPS == 5);
const _: () = assert!(56 >= LUT_BITS as usize * SINGLE_STEPS);

/// Fixed L1d-hot byte chunk (`1 KiB`) for [`Huff::decode_run`]: `ck` persists across
/// coalesced runs and flushes to `syms` (widened to `u16`) only once `ck > CHUNK_STOP` or on
/// `EOB`, keeping `ck & CHUNK_MASK` wrap-free and `idx + 4 <= CHUNK_LEN` (zero per-step bounds
/// checks, zero `syms.resize` memset, safe Rust). Symbol `256` (the only one past a byte that
/// is not the end) is staged as `0` and its position kept in [`Patches`].
pub(crate) const CHUNK_LEN: usize = 1042;
const CHUNK_MASK: usize = 1023;
const CHUNK_STOP: usize = 1000;

/// The chunk positions staged as `0` that are symbol `256` (`alpha == 258` blocks only).
/// One slot a chunk position, so it can never fill: a block holding many `256`s costs one
/// cold store each, never an early flush.
pub(crate) struct Patches {
    pos: [u16; CHUNK_LEN],
    len: usize,
}

impl Default for Patches {
    fn default() -> Self {
        Self {
            pos: [0; CHUNK_LEN],
            len: 0,
        }
    }
}

impl Patches {
    /// Forget positions a block left behind (one that ended in an error flushes nothing)
    #[inline(always)]
    pub(crate) fn clear(&mut self) {
        self.len = 0;
    }

    /// (in bounds: one position a chunk slot, cleared at each flush and block)
    #[inline(always)]
    fn push(&mut self, at: usize) {
        if let Some(slot) = self.pos.get_mut(self.len) {
            *slot = at as u16;
            self.len += 1;
        }
    }

    /// Append the staged `chunk` to `syms` as `u16`, restoring each `256`
    #[inline(never)]
    fn flush(&mut self, chunk: &[u8], syms: &mut Vec<u16>) {
        let base = syms.len();
        syms.extend(chunk.iter().map(|&b| u16::from(b)));
        for &p in &self.pos[..self.len] {
            syms[base + p as usize] = 256;
        }
        self.len = 0;
    }
}

pub(crate) struct Huff {
    pub(crate) lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    pub(crate) limit: [i32; MAX_CODE_LEN],
    pub(crate) base: [i32; MAX_CODE_LEN],
    pub(crate) perm: [u16; MAX_ALPHA],
    /// each symbol's code length (`< 256` only: a fast entry's 1st symbol's length)
    pub(crate) sym_len: [u8; 256],
    pub(crate) min_len: u32,
    /// whether an entry may hold more than one symbol (pairs built and `2 * min_len <= 10`);
    /// if not, every fast entry holds exactly one
    multi: bool,
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
            sym_len: [0; 256],
            min_len: 1,
            multi: false,
            alpha: 0,
            eob: 0,
        }
    }
}

/// Marks the branch that calls it as rarely taken, so the fast path is laid out straight
/// (`std::hint::cold_path` postdates the crate's `rust-version`)
#[cold]
#[inline(always)]
fn cold() {}

impl Huff {
    #[cfg(test)]
    pub(crate) fn new(lens: &[u8]) -> Huff {
        let mut h = Huff::default();
        h.build_into(lens, true, true);
        h
    }

    /// The builder `build_into` replaced (two-pass: `K=1` fill, then `K=2` overwrite, with a
    /// full `d_arr`), kept to check the single-write builder bit for bit.
    #[cfg(test)]
    pub(crate) fn build_into_reference(&mut self, lens: &[u8], build_pairs: bool) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let n = alpha.min(256);
        self.sym_len[..n].copy_from_slice(&lens[..n]);
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
                    Some(s) => (1u32 << CNT_SH) | ((s as u32) << SYM1) | zn,
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
                        if (e2 >> CNT_SH) == 0 {
                            continue;
                        }
                        let l2 = self.sym_len[((e2 >> SYM1) & 0xff) as usize] as usize;
                        if (zn as usize) + l2 > LUT_BITS as usize {
                            continue;
                        }
                        self.lut[w] = (2u32 << CNT_SH)
                            | (((e2 >> SYM1) & 0xff) << SYM2)
                            | (e & (0xff << SYM1))
                            | (zn + l2 as u32);
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    /// Build the table for `lens`: `K=2` pairs if `build_pairs`, then `K=3` triples too if
    /// `build_triples` (which needs `build_pairs`: triples are built from pair entries).
    pub(crate) fn build_into(&mut self, lens: &[u8], build_pairs: bool, build_triples: bool) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let n = alpha.min(256);
        self.sym_len[..n].copy_from_slice(&lens[..n]);
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

        self.multi = build_pairs && 2 * min_l <= LUT_BITS as usize;
        self.fill_lut(min_l as u32, max_l, build_pairs);
        if build_pairs && build_triples {
            self.fill_triples(min_l as u32);
        }
    }

    /// Upgrade pair entries in place to `K=3` where a third whole code fits in the root's
    /// 10 bits. For a first symbol of length `zn`, the rest of an entry's bits (`u`, the
    /// next `10 - zn`) are the root index `u << zn`, whose (pair or already upgraded) entry
    /// gives the 2nd and 3rd symbols; the indices `u` sharing those two codes are one
    /// aligned run, filled at once. The first code of each length builds its span and the
    /// rest clone it with a `SYM1` delta, as in `fill_lut`. Fast entries are never the end
    /// or `256`, so neither is ever part of a triple.
    fn fill_triples(&mut self, min_len: u32) {
        if min_len * 3 > LUT_BITS {
            return;
        }
        let mut zn_ranges = [(0i64, -1i64); 11];
        let mut end_at_len = [0usize; 11];
        let mut a = 0i64;
        let mut filled_to = 0usize;
        for zn in min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            if hi >= a {
                filled_to = ((hi + 1) << (LUT_BITS - zn)) as usize;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }
        let mask = LUT_LEN - 1;
        for zn in min_len..=LUT_BITS - 2 * min_len {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = (LUT_BITS - zn) as usize;
            // the suffixes whose first code is short enough to leave room for another
            let max_l2 = (LUT_BITS - zn - min_len) as usize;
            let u_lim = end_at_len[max_l2] >> zn;
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
                    // `perm` ascends within a length: `s0 < s1 <= 255`, no carry out of `SYM1`
                    let delta = s1.wrapping_sub(s0) << SYM1;
                    let (left, right) = self.lut.split_at_mut(off_p);
                    for (d, &s) in right[..u_lim].iter_mut().zip(&left[off0..off0 + u_lim]) {
                        *d = s.wrapping_add(delta);
                    }
                    continue;
                }
                let mut u = 0usize;
                while u < u_lim {
                    let e23 = self.lut[(u << zn) & mask];
                    let c23 = e23 >> CNT_SH;
                    if c23 < 2 {
                        u += 1;
                        continue;
                    }
                    let s2 = (e23 >> SYM1) & 0xff;
                    let s3 = (e23 >> SYM2) & 0xff;
                    let l23 = if c23 == 2 {
                        e23 & 15
                    } else {
                        (self.sym_len[s2 as usize] + self.sym_len[s3 as usize]) as u32
                    };
                    let tot = zn + l23;
                    if tot > LUT_BITS {
                        u += 1;
                        continue;
                    }
                    // `u` starts its run: the previous `u` visited was outside it, or the
                    // previous run ended here
                    let end_u = (u + (1usize << (LUT_BITS - tot))).min(u_lim);
                    let entry = (3u32 << CNT_SH) | (s3 << SYM3) | (s2 << SYM2) | (s1 << SYM1) | tot;
                    self.lut[off_p + u..off_p + end_u].fill(entry);
                    u = end_u;
                }
                tpl = Some((off_p, s1));
            }
        }
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
                Some(s) => (1u32 << CNT_SH) | ((s as u32) << SYM1) | zn,
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
                let pair_base = (2u32 << CNT_SH) | (s1 << SYM1);
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
            && self.sym_len == o.sym_len
            && self.min_len == o.min_len
            && self.alpha == o.alpha
            && self.eob == o.eob
    }

    #[inline(always)]
    pub(crate) fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// One symbol (the tests' single-symbol view of the table)
    #[cfg(test)]
    pub(crate) fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut[r.peek(data, LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            let s = ((e >> SYM1) & 0xff) as usize;
            r.skip_short(self.sym_len[s] as u32);
            return Ok(s as u16);
        }
        let (s, after) = self.decode_entry_slow(e, *r, data);
        *r = after;
        s
    }

    /// Decode up to `target` symbols (`target = m * GROUP >= GROUP`) using this table,
    /// staging bytes into the L1d-hot fixed `chunk` starting at `*ck_out <= CHUNK_STOP` and
    /// flushing to `syms` only when `ck > CHUNK_STOP` or on `EOB`. Returns `Ok(true)` if
    /// `eob` was decoded (with all symbols flushed to `syms`), `Ok(false)` once all `target`
    /// symbols were staged/appended (maintaining `*ck_out <= CHUNK_STOP`).
    ///
    /// A batch is `PRE_STEPS + 1` lookups of up to 3 symbols each, stopping short of the
    /// run's end so no lookup takes a symbol past it. The refill comes between the last
    /// lookup's load and its shift, off the shift chain: at a batch's start at least 46 bits
    /// are live (56 after a refill, less one step's 10), so at least 16 are before its last
    /// step (each earlier step takes at most 10 bits, or refills itself on the slow path),
    /// and a refill never changes the live bits. A table whose entries are all single
    /// symbols (no pairs, as on near-uniform byte distributions) gains nothing from the
    /// short pipelined batch and takes `SINGLE_STEPS` lookups a refill instead.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(crate) fn decode_run(
        &self,
        r_out: &mut Reg,
        data: &[u8],
        chunk: &mut [u8; CHUNK_LEN],
        patches: &mut Patches,
        ck_out: &mut usize,
        syms: &mut Vec<u16>,
        target: usize,
    ) -> Result<bool, ErrorKind> {
        debug_assert!(target >= GROUP && *ck_out <= CHUNK_STOP);
        let mut r = *r_out;
        let mut ck = *ck_out;
        let eob = self.eob;
        let mut rem_syms = target;

        // One slow-path symbol `s` at chunk position `at`: the end returns, `256` is patched.
        // A `<= 10`-bit `256` (frequent on `alpha == 258` blocks) is taken here, no call: its
        // bits skipped as `decode_entry_slow` would (at most 10, all live), `0` staged.
        macro_rules! slow {
            ($e:expr, $at:expr, $place:expr) => {{
                if ($e & SYM256) != 0 {
                    r.skip_short($e & 63);
                    patches.push($at);
                    $place = 0;
                } else {
                    slow_call!($e, $at, $place);
                }
            }};
        }
        macro_rules! slow_call {
            ($e:expr, $at:expr, $place:expr) => {{
                let (s, after) = self.decode_entry_slow($e, r, data);
                r = after;
                let s = match s {
                    Ok(s) => s,
                    Err(err) => {
                        *r_out = r;
                        return Err(err);
                    }
                };
                let at = $at;
                if s == eob {
                    patches.flush(&chunk[..at], syms);
                    syms.push(s);
                    *ck_out = 0;
                    *r_out = r;
                    return Ok(true);
                }
                if s > 255 {
                    patches.push(at);
                }
                $place = s as u8;
            }};
        }

        // One lookup of a batch at `bk`, the refill under its load if `$refill`; `$wide`: the
        // entry may hold up to 3 symbols (one 32-bit store), else exactly one (a byte store,
        // which never splits a cache line).
        macro_rules! step {
            ($batch:ident, $bk:ident, $refill:expr, $wide:expr) => {{
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                if $refill {
                    r.refill(data);
                }
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_low4(e);
                    let idx = $bk & 15;
                    if $wide {
                        $batch[idx..idx + 4].copy_from_slice(&(e >> SYM1).to_le_bytes());
                    } else {
                        $batch[idx] = (e >> SYM1) as u8;
                    }
                    $bk += c;
                } else {
                    cold();
                    slow!(e, ck + $bk, $batch[$bk & 15]);
                    $bk += 1;
                }
            }};
        }
        // Whole batches while `$most` symbols remain (the most a batch takes), each batch's
        // steps written out (`$body`), so the loop is unrolled whatever the slow paths' size.
        macro_rules! batches {
            ($most:expr, |$batch:ident, $bk:ident| $body:block) => {
                while rem_syms >= $most {
                    let stop_ck = (ck + (rem_syms - $most)).min(CHUNK_STOP);
                    let before_ck = ck;
                    while ck <= stop_ck {
                        let $batch: &mut [u8; 20] = (&mut chunk[ck..ck + 20]).try_into().unwrap();
                        let mut $bk = 0usize;
                        $body
                        ck += $bk;
                    }
                    rem_syms -= ck - before_ck;
                    if ck > CHUNK_STOP {
                        patches.flush(&chunk[..ck], syms);
                        ck = 0;
                    }
                }
            };
        }

        r.refill(data);
        if self.multi {
            // `PRE_STEPS` lookups, then the last with the refill under it
            batches!(BATCH_SYMS, |batch, bk| {
                step!(batch, bk, false, true);
                step!(batch, bk, false, true);
                step!(batch, bk, false, true);
                step!(batch, bk, true, true);
            });
        } else {
            // `SINGLE_STEPS` lookups, then the refill
            batches!(SINGLE_STEPS, |batch, bk| {
                step!(batch, bk, false, false);
                step!(batch, bk, false, false);
                step!(batch, bk, false, false);
                step!(batch, bk, false, false);
                step!(batch, bk, false, false);
                r.refill(data);
            });
        }

        // the run's last few symbols: whole entries while 3 or more remain, then
        // one symbol a lookup
        while rem_syms >= 3 {
            if r.live < LUT_BITS {
                r.refill(data);
            }
            let e = self.lut[r.peek_live(LUT_BITS) as usize];
            let c = (e >> CNT_SH) as usize;
            if c != 0 {
                r.skip_short_shl_low4(e);
                let idx = ck & CHUNK_MASK;
                chunk[idx..idx + 4].copy_from_slice(&(e >> SYM1).to_le_bytes());
                ck += c;
                rem_syms -= c;
            } else {
                slow!(e, ck, chunk[ck & CHUNK_MASK]);
                ck += 1;
                rem_syms -= 1;
            }
        }
        while rem_syms > 0 {
            let e = self.lut[r.peek(data, LUT_BITS) as usize];
            if (e >> CNT_SH) != 0 {
                let s = (e >> SYM1) as u8;
                r.skip_short(self.sym_len[s as usize] as u32);
                chunk[ck & CHUNK_MASK] = s;
            } else {
                slow!(e, ck, chunk[ck & CHUNK_MASK]);
            }
            ck += 1;
            rem_syms -= 1;
        }
        if ck > CHUNK_STOP {
            patches.flush(&chunk[..ck], syms);
            ck = 0;
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
