//! Smooth-zone probe for claim `scanner-per-block-fixed-cost`:
//! tests whether `#1034` (`5ed20c1d`) introduced a per-block fixed scanner cost or first-block
//! pipeline-fill latency penalty relative to its first parent (`5ed20c1d^`).
//!
//! 100% safe Rust (`#![forbid(unsafe_code)]`), zero `src/` edits.
//!
//! Frozen Stop Test (`frontier/claims/scanner-per-block-fixed-cost.toml`):
//! - CONFIRMED if on `pattern` the post-`#1034` per-block fixed scanner cost (or first-block
//!   latency) exceeds the parent's by an amount >= 50% of the CI's 4T median wall loss scaled to
//!   this machine's 4T `pattern` wall, with non-overlapping ranges;
//! - REFUTED if the difference is within the parent's range (or faster);
//! - INCONCLUSIVE otherwise.
//!
//! Usage:
//!   cargo run --release --example scanner_per_block_fixed_cost -- [--repeats 9] FILE.bz2...
#![forbid(unsafe_code)]

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::ErrorKind;
use std::fs::File;
use std::hint::black_box;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;
const MAX_GROUPS: usize = 6;
const MAX_SELECTORS: usize = 18002;
const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;
const MAX_SYMS: usize = MAX_SELECTORS * GROUP;
const PAIR_GATE_SEL: usize = 64;
const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;
const STEPS: usize = 5;

const CHUNK_LEN: usize = 1042;
const CHUNK_MASK: usize = 1023;
const CHUNK_STOP: usize = 1000;

// ---------------------------------------------------------------------------
// Exact bit register matching `src/bits.rs` (both `5ed20c1d^` and `5ed20c1d`)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Reg {
    buf: u64,
    live: u32,
    pos: usize,
}

impl Reg {
    fn at_bit(data: &[u8], bit_pos: u64) -> Self {
        let byte_pos = (bit_pos / 8) as usize;
        let rem = (bit_pos % 8) as u32;
        let mut r = Self {
            buf: 0,
            live: 0,
            pos: byte_pos,
        };
        r.refill(data);
        if rem > 0 {
            r.skip_short(rem);
        }
        r
    }

    fn bit_pos(&self) -> u64 {
        (self.pos as u64) * 8 - (self.live as u64)
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
    fn refill_slow(mut self, data: &[u8]) -> Self {
        while self.live <= 56 {
            let b = data.get(self.pos).copied().unwrap_or(0);
            self.pos += 1;
            self.buf |= (b as u64) << (56 - self.live);
            self.live += 8;
        }
        self
    }

    #[inline(always)]
    fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        if self.live < n {
            self.refill(data);
        }
        self.peek_live(n)
    }

    #[inline(always)]
    fn skip_short(&mut self, n: u32) {
        self.buf <<= n;
        self.live -= n;
    }

    #[inline(always)]
    fn skip_short_shl(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= e & 63;
    }

    #[inline(always)]
    fn skip_short_shl_u8(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= (e as u8) as u32;
    }

    #[inline(always)]
    fn bits(&mut self, data: &[u8], n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let v = self.peek(data, n);
        self.skip_short(n);
        v
    }

    #[inline(always)]
    fn bit(&mut self, data: &[u8]) -> bool {
        self.bits(data, 1) != 0
    }

    #[inline(always)]
    fn overran(&self, data: &[u8]) -> bool {
        self.pos > data.len() && (self.pos - data.len()) * 8 > self.live as usize
    }
}

// ---------------------------------------------------------------------------
// Parent (`5ed20c1d^`) Huffman table & decoder (9-bit SYM1/SYM2, always pairs)
// ---------------------------------------------------------------------------

mod parent_huff {
    use super::*;

    const SUB_PTR: u32 = 1 << 27;
    const BAD: u32 = 1 << 28;
    const EOB_FAST: u32 = 1 << 29;
    const LEN1: u32 = 6;
    const SYM1: u32 = 11;
    const SYM2: u32 = 20;
    const CNT_SH: u32 = 30;

    pub struct Huff {
        lut: [u32; LUT_LEN],
        sub_lut: [u16; SUB_CAP],
        base: [i32; MAX_CODE_LEN],
        limit: [i32; MAX_CODE_LEN],
        perm: [u16; MAX_ALPHA],
        min_len: u32,
        max_len: u32,
        alpha: i32,
        eob: u16,
    }

    impl Default for Huff {
        fn default() -> Self {
            Self {
                lut: [0; LUT_LEN],
                sub_lut: [SUB_BAD_U16; SUB_CAP],
                base: [0; MAX_CODE_LEN],
                limit: [0; MAX_CODE_LEN],
                perm: [0; MAX_ALPHA],
                min_len: 1,
                max_len: 1,
                alpha: 0,
                eob: 0,
            }
        }
    }

    impl Huff {
        pub fn build_into(&mut self, lens: &[u8]) {
            let alpha = lens.len();
            let eob = (alpha - 1) as u16;
            let mut counts = [0i32; MAX_CODE_LEN];
            let (mut min_l, mut max_l) = (MAX_CODE_LEN, 0usize);
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
            self.max_len = max_l as u32;
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
        fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
            let i = zvec - self.base[zn as usize];
            (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
        }

        #[cold]
        #[inline(never)]
        fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
            let mut zn = self.min_len;
            let mut zvec = r.bits(data, zn) as i32;
            while zvec > self.limit[zn as usize] {
                zn += 1;
                if zn > self.max_len {
                    return Err(ErrorKind::BadHuffmanCode);
                }
                zvec = (zvec << 1) | i32::from(r.bit(data));
            }
            self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode)
        }

        #[cold]
        #[inline(never)]
        fn decode_entry_slow(
            &self,
            e: u32,
            mut r: Reg,
            data: &[u8],
        ) -> (Result<u16, ErrorKind>, Reg) {
            if (e & SUB_PTR) != 0 {
                if r.live < 20 {
                    r.refill(data);
                }
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

        #[inline(always)]
        fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
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
        pub fn decode_group_win(
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
        pub fn decode_group_slice(
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
    }
}

// ---------------------------------------------------------------------------
// Post-`#1034` (`5ed20c1d`) Huffman table & coalesced chunked decoder
// ---------------------------------------------------------------------------

mod post1034_huff {
    use super::*;

    const SYM256: u32 = 1 << 26;
    const SUB_PTR: u32 = 1 << 27;
    const BAD: u32 = 1 << 28;
    const EOB_FAST: u32 = 1 << 29;
    const SYM1: u32 = 8;
    const SYM2: u32 = 16;
    const LEN1: u32 = 24;
    const CNT_SH: u32 = 30;

    pub struct Huff {
        lut: [u32; LUT_LEN],
        sub_lut: [u16; SUB_CAP],
        base: [i32; MAX_CODE_LEN],
        limit: [i32; MAX_CODE_LEN],
        perm: [u16; MAX_ALPHA],
        min_len: u32,
        max_len: u32,
        alpha: i32,
        eob: u16,
    }

    impl Default for Huff {
        fn default() -> Self {
            Self {
                lut: [0; LUT_LEN],
                sub_lut: [SUB_BAD_U16; SUB_CAP],
                base: [0; MAX_CODE_LEN],
                limit: [0; MAX_CODE_LEN],
                perm: [0; MAX_ALPHA],
                min_len: 1,
                max_len: 1,
                alpha: 0,
                eob: 0,
            }
        }
    }

    impl Huff {
        pub fn build_into(&mut self, lens: &[u8], build_pairs: bool) {
            let alpha = lens.len();
            let eob = (alpha - 1) as u16;
            let mut counts = [0i32; MAX_CODE_LEN];
            let (mut min_l, mut max_l) = (MAX_CODE_LEN, 0usize);
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
            self.max_len = max_l as u32;
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

        #[inline(always)]
        fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
            let i = zvec - self.base[zn as usize];
            (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
        }

        #[cold]
        #[inline(never)]
        fn decode_slow(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
            let mut zn = self.min_len;
            let mut zvec = r.bits(data, zn) as i32;
            while zvec > self.limit[zn as usize] {
                zn += 1;
                if zn > self.max_len {
                    return Err(ErrorKind::BadHuffmanCode);
                }
                zvec = (zvec << 1) | i32::from(r.bit(data));
            }
            self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode)
        }

        #[cold]
        #[inline(never)]
        fn decode_entry_slow(
            &self,
            e: u32,
            mut r: Reg,
            data: &[u8],
        ) -> (Result<u16, ErrorKind>, Reg) {
            if (e & SUB_PTR) != 0 {
                if r.live < 20 {
                    r.refill(data);
                }
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
            let s = self.decode_slow(&mut r, data);
            r.refill(data);
            (s, r)
        }

        #[inline(always)]
        fn decode(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
            let e = self.lut[r.peek(data, LUT_BITS) as usize];
            if (e >> CNT_SH) != 0 {
                r.skip_short((e >> LEN1) & 31);
                return Ok(((e >> SYM1) as u8) as u16);
            }
            let (s, after) = self.decode_entry_slow(e, *r, data);
            *r = after;
            s
        }

        #[inline(always)]
        pub fn decode_run(
            &self,
            r_out: &mut Reg,
            data: &[u8],
            chunk: &mut [u16; CHUNK_LEN],
            ck_out: &mut usize,
            syms: &mut Vec<u16>,
            target: usize,
        ) -> Result<bool, ErrorKind> {
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
    }
}

// ---------------------------------------------------------------------------
// Per-block A2 (fixed setup) and A3 (symbol decode) for Parent vs Post-#1034
// ---------------------------------------------------------------------------

struct ParentScratch {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[parent_huff::Huff; MAX_GROUPS]>,
}

impl Default for ParentScratch {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| parent_huff::Huff::default())),
        }
    }
}

struct Post1034Scratch {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[post1034_huff::Huff; MAX_GROUPS]>,
    chunk: Box<[u16; CHUNK_LEN]>,
}

impl Default for Post1034Scratch {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| post1034_huff::Huff::default())),
            chunk: Box::new([0u16; CHUNK_LEN]),
        }
    }
}

struct A2Out {
    r: Reg,
    sel_cap: usize,
    alpha: usize,
    used_groups: u8,
    n_sel: usize,
    max_out: usize,
}

fn a2_parent(
    data: &[u8],
    mut r: Reg,
    syms: &mut Vec<u16>,
    sc: &mut ParentScratch,
) -> Result<A2Out, ErrorKind> {
    let data_len = data.len();
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic != BLOCK_MAGIC {
        return Err(ErrorKind::BadBlockMagic);
    }
    let _crc = r.bits(data, 32);
    if r.bit(data) {
        return Err(ErrorKind::Randomised);
    }
    let _orig = r.bits(data, 24);
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
    let sel_buf = &mut sc.selectors[..sel_cap];
    let mut used_groups = 0u8;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            return Err(ErrorKind::BadSelector);
        }
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
            return Err(ErrorKind::Truncated);
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
                if !(1..=20).contains(&curr) {
                    return Err(ErrorKind::BadCodeLength);
                }
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
                if r.pos > data_len && r.overran(data) {
                    return Err(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            sc.tables[g].build_into(&lens[..alpha]);
        }
    }
    if syms.capacity() < MAX_SYMS {
        syms.clear();
        syms.reserve(MAX_SYMS);
    }
    let max_out = (sel_cap * GROUP + 66).min(MAX_SYMS);
    if syms.len() < max_out {
        syms.resize(max_out, 0);
    }
    Ok(A2Out {
        r,
        sel_cap,
        alpha,
        used_groups,
        n_sel,
        max_out,
    })
}

fn a3_parent(
    data: &[u8],
    mut r: Reg,
    sel_cap: usize,
    max_out: usize,
    syms: &mut Vec<u16>,
    sc: &ParentScratch,
) -> Result<Reg, ErrorKind> {
    let data_len = data.len();
    let out = &mut syms[..max_out];
    let sel_buf = &sc.selectors[..sel_cap];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut tail_buf = [0u16; 66];
    let res = loop {
        if sel >= sel_cap {
            break Err(ErrorKind::SelectorOverrun);
        }
        let t = &sc.tables[sel_buf[sel] as usize];
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
        let (k, hit_eob) = match g_res {
            Ok(v) => v,
            Err(err) => break Err(err),
        };
        if use_tail {
            out[n..n + k].copy_from_slice(&tail_buf[..k]);
        }
        n += k;
        if hit_eob {
            break Ok(());
        }
    };
    res?;
    syms.truncate(n);
    Ok(r)
}

fn a2_post1034(
    data: &[u8],
    mut r: Reg,
    syms: &mut Vec<u16>,
    sc: &mut Post1034Scratch,
) -> Result<A2Out, ErrorKind> {
    let data_len = data.len();
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic != BLOCK_MAGIC {
        return Err(ErrorKind::BadBlockMagic);
    }
    let _crc = r.bits(data, 32);
    if r.bit(data) {
        return Err(ErrorKind::Randomised);
    }
    let _orig = r.bits(data, 24);
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
    let sel_buf = &mut sc.selectors[..sel_cap];
    let mut used_groups = 0u8;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            return Err(ErrorKind::BadSelector);
        }
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
            return Err(ErrorKind::Truncated);
        }
    }
    let build_pairs = n_sel >= PAIR_GATE_SEL;
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
                if !(1..=20).contains(&curr) {
                    return Err(ErrorKind::BadCodeLength);
                }
                if !r.bit(data) {
                    break;
                }
                if r.bit(data) {
                    curr -= 1;
                } else {
                    curr += 1;
                }
                if r.pos > data_len && r.overran(data) {
                    return Err(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            sc.tables[g].build_into(&lens[..alpha], build_pairs);
        }
    }
    if syms.capacity() < MAX_SYMS {
        syms.clear();
        syms.reserve(MAX_SYMS);
    } else {
        syms.clear();
    }
    Ok(A2Out {
        r,
        sel_cap,
        alpha,
        used_groups,
        n_sel,
        max_out: 0,
    })
}

fn a3_post1034(
    data: &[u8],
    mut r: Reg,
    sel_cap: usize,
    syms: &mut Vec<u16>,
    sc: &mut Post1034Scratch,
) -> Result<Reg, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    black_box(std::is_x86_feature_detected!("bmi2"));
    let data_len = data.len();
    let sel_buf = &sc.selectors[..sel_cap];
    let mut sel = 0usize;
    let mut ck = 0usize;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &sc.tables[g as usize];
        while rem > 0 {
            let need = rem * 125 + 16;
            let (take, check_overran) = if r.pos.saturating_add(need) <= data_len {
                (rem, false)
            } else {
                let safe_m = data_len.saturating_sub(r.pos.saturating_add(16)) / 125;
                if safe_m >= 1 {
                    (safe_m, false)
                } else {
                    (1, true)
                }
            };
            match t.decode_run(&mut r, data, &mut sc.chunk, &mut ck, syms, take * GROUP) {
                Ok(true) => return Ok(r),
                Ok(false) => {}
                Err(err) => return Err(err),
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return Err(ErrorKind::Truncated);
            }
            rem -= take;
        }
    }
    Err(ErrorKind::SelectorOverrun)
}

// ---------------------------------------------------------------------------
// Corpus loader & shipped parity verification
// ---------------------------------------------------------------------------

struct BlockMeta {
    start_bit: u64,
    end_bit: u64,
    syms_len: usize,
    n_sel: usize,
    used_groups: usize,
    sel_runs: usize,
    alpha: usize,
}

fn verify_and_index(path: &Path, data: &[u8]) -> Vec<BlockMeta> {
    let mut scanner = Scanner::new(SliceSource(data));
    let mut p_sc = ParentScratch::default();
    let mut n_sc = Post1034Scratch::default();
    let mut p_syms = Vec::new();
    let mut n_syms = Vec::new();
    let mut s_syms = Vec::new();
    let mut blocks = Vec::new();

    while let Event::Block { bit, symbols, .. } =
        scanner.next(s_syms).expect("shipped scanner failed")
    {
        let r0 = Reg::at_bit(data, bit);
        let pa2 = a2_parent(data, r0, &mut p_syms, &mut p_sc).expect("a2_parent");
        let pr = a3_parent(data, pa2.r, pa2.sel_cap, pa2.max_out, &mut p_syms, &p_sc)
            .expect("a3_parent");
        let na2 = a2_post1034(data, r0, &mut n_syms, &mut n_sc).expect("a2_post1034");
        let nr =
            a3_post1034(data, na2.r, na2.sel_cap, &mut n_syms, &mut n_sc).expect("a3_post1034");
        assert_eq!(
            p_syms,
            symbols.syms,
            "{} block {}: parent syms mismatch",
            path.display(),
            blocks.len()
        );
        assert_eq!(
            n_syms,
            symbols.syms,
            "{} block {}: post1034 syms mismatch",
            path.display(),
            blocks.len()
        );
        assert_eq!(
            pr.bit_pos(),
            nr.bit_pos(),
            "{} block {}: end bit mismatch",
            path.display(),
            blocks.len()
        );
        let mut sel_runs = 1usize;
        for i in 1..na2.sel_cap {
            if n_sc.selectors[i] != n_sc.selectors[i - 1] {
                sel_runs += 1;
            }
        }
        blocks.push(BlockMeta {
            start_bit: bit,
            end_bit: nr.bit_pos(),
            syms_len: symbols.syms.len(),
            n_sel: na2.n_sel,
            used_groups: na2.used_groups.count_ones() as usize,
            sel_runs,
            alpha: na2.alpha,
        });
        s_syms = symbols.syms;
    }
    let last_end = blocks.last().expect("at least one block").end_bit;
    let mut er = Reg::at_bit(data, last_end);
    let m = (er.bits(data, 24) as u64) << 24 | er.bits(data, 24) as u64;
    assert_eq!(m, END_MAGIC, "{}: final END_MAGIC check", path.display());
    blocks
}

fn stats(xs: &[f64]) -> (f64, f64, f64, f64) {
    let mut v = xs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let min = v[0];
    let max = v[v.len() - 1];
    let med = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    (min, med, max, mean)
}

fn measure_first_block_latency(path: &Path, arm: u8, cold_syms: bool) -> f64 {
    // Persistent waiting worker thread: measures File::open -> read first chunk -> stream header
    // -> scan block 0 -> channel send -> worker receives block 0.
    let (tx_job, rx_job) = mpsc::sync_channel::<Vec<u16>>(1);
    let (tx_done, rx_done) = mpsc::sync_channel::<Instant>(1);
    let worker = thread::spawn(move || {
        if let Ok(syms) = rx_job.recv() {
            let arrived = Instant::now();
            black_box(syms.len());
            let _ = tx_done.send(arrived);
        }
    });

    let mut p_sc = ParentScratch::default();
    let mut n_sc = Post1034Scratch::default();
    let mut syms = if cold_syms {
        Vec::new()
    } else {
        Vec::with_capacity(MAX_SYMS)
    };
    let mut chunk = vec![0u8; 4 << 20];

    let t0 = Instant::now();
    let mut f = File::open(path).expect("open");
    let n = f.read(&mut chunk).expect("read");
    let data = &chunk[..n];
    assert!(data.len() >= 4 && &data[..3] == b"BZh");
    let r0 = Reg::at_bit(data, 32);
    match arm {
        0 => {
            let a2 = a2_parent(data, r0, &mut syms, &mut p_sc).unwrap();
            let _ = a3_parent(data, a2.r, a2.sel_cap, a2.max_out, &mut syms, &p_sc).unwrap();
        }
        1 => {
            let a2 = a2_post1034(data, r0, &mut syms, &mut n_sc).unwrap();
            let _ = a3_post1034(data, a2.r, a2.sel_cap, &mut syms, &mut n_sc).unwrap();
        }
        _ => {
            let mut sc = Scanner::new(SliceSource(data));
            match sc.next(syms).unwrap() {
                Event::Block { symbols, .. } => syms = symbols.syms,
                Event::Done => panic!("empty stream"),
            }
        }
    }
    tx_job.send(syms).unwrap();
    let t1 = rx_done.recv().unwrap();
    worker.join().unwrap();
    t1.saturating_duration_since(t0).as_secs_f64() * 1e6
}

fn bench_corpus(path: &Path, repeats: usize) {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let data = std::fs::read(path).expect("read bz2");
    let blocks = verify_and_index(path, &data);
    let n_blk = blocks.len();
    let tot_syms: usize = blocks.iter().map(|b| b.syms_len).sum();
    let tot_sel: usize = blocks.iter().map(|b| b.n_sel).sum();
    let tot_runs: usize = blocks.iter().map(|b| b.sel_runs).sum();
    let tot_used_tbl: usize = blocks.iter().map(|b| b.used_groups).sum();
    let pair_gated: usize = blocks.iter().filter(|b| b.n_sel < PAIR_GATE_SEL).count();
    let mean_alpha = blocks.iter().map(|b| b.alpha as f64).sum::<f64>() / n_blk as f64;
    let first_bits = blocks[0].end_bit - blocks[0].start_bit;

    println!("\n======================================================================");
    println!(
        "CORPUS: {name} | size={} B | blocks={n_blk} | pair_gated(<64 sel)={pair_gated}/{n_blk}",
        data.len()
    );
    println!(
        "  mean alpha={mean_alpha:.1} | used_tables={tot_used_tbl} ({:.2}/blk) | n_sel={tot_sel} ({:.1}/blk) | sel_runs={tot_runs} ({:.1}/blk) | syms={tot_syms} ({:.1}/blk) | blk0={first_bits} bits",
        tot_used_tbl as f64 / n_blk as f64,
        tot_sel as f64 / n_blk as f64,
        tot_runs as f64 / n_blk as f64,
        tot_syms as f64 / n_blk as f64,
    );

    // Choose inner iterations per repeat so even tiny corpora take >= 2 ms per repeat.
    let inner = if tot_syms < 10_000 {
        200usize
    } else if tot_syms < 100_000 {
        30usize
    } else {
        1usize
    };

    let mut p_sc = ParentScratch::default();
    let mut n_sc = Post1034Scratch::default();
    let mut p_syms = Vec::with_capacity(MAX_SYMS);
    let mut n_syms = Vec::with_capacity(MAX_SYMS);

    let mut p_a2_us = Vec::with_capacity(repeats);
    let mut n_a2_us = Vec::with_capacity(repeats);
    let mut p_a3_us = Vec::with_capacity(repeats);
    let mut n_a3_us = Vec::with_capacity(repeats);
    let mut p_tot_us = Vec::with_capacity(repeats);
    let mut n_tot_us = Vec::with_capacity(repeats);

    // Warmup both arms
    for b in &blocks {
        let r0 = Reg::at_bit(&data, b.start_bit);
        let a2 = a2_parent(&data, r0, &mut p_syms, &mut p_sc).unwrap();
        black_box(a3_parent(&data, a2.r, a2.sel_cap, a2.max_out, &mut p_syms, &p_sc).unwrap());
        let a2n = a2_post1034(&data, r0, &mut n_syms, &mut n_sc).unwrap();
        black_box(a3_post1034(&data, a2n.r, a2n.sel_cap, &mut n_syms, &mut n_sc).unwrap());
    }

    // Interleaved 1T best-of-R measurement of A2 and A3 separately (per block & total)
    for rep in 0..repeats {
        let order = if rep % 2 == 0 { [0u8, 1u8] } else { [1u8, 0u8] };
        for arm in order {
            let mut a2_ns = 0u128;
            let mut a3_ns = 0u128;
            let t_pass0 = Instant::now();
            for _ in 0..inner {
                for b in &blocks {
                    let r0 = Reg::at_bit(&data, b.start_bit);
                    if arm == 0 {
                        let t0 = Instant::now();
                        let a2 = a2_parent(&data, r0, &mut p_syms, &mut p_sc).unwrap();
                        let t1 = Instant::now();
                        let r1 = a3_parent(&data, a2.r, a2.sel_cap, a2.max_out, &mut p_syms, &p_sc)
                            .unwrap();
                        let t2 = Instant::now();
                        black_box((r1.pos, p_syms.len()));
                        a2_ns += t1.duration_since(t0).as_nanos();
                        a3_ns += t2.duration_since(t1).as_nanos();
                    } else {
                        let t0 = Instant::now();
                        let a2 = a2_post1034(&data, r0, &mut n_syms, &mut n_sc).unwrap();
                        let t1 = Instant::now();
                        let r1 =
                            a3_post1034(&data, a2.r, a2.sel_cap, &mut n_syms, &mut n_sc).unwrap();
                        let t2 = Instant::now();
                        black_box((r1.pos, n_syms.len()));
                        a2_ns += t1.duration_since(t0).as_nanos();
                        a3_ns += t2.duration_since(t1).as_nanos();
                    }
                }
            }
            let pass_us = t_pass0.elapsed().as_secs_f64() * 1e6 / inner as f64;
            let a2_us = (a2_ns as f64) / 1e3 / inner as f64;
            let a3_us = (a3_ns as f64) / 1e3 / inner as f64;
            if arm == 0 {
                p_a2_us.push(a2_us);
                p_a3_us.push(a3_us);
                p_tot_us.push(pass_us);
            } else {
                n_a2_us.push(a2_us);
                n_a3_us.push(a3_us);
                n_tot_us.push(pass_us);
            }
        }
    }

    let show = |label: &str, p: &[f64], n: &[f64]| {
        let (pmin, pmed, pmax, _) = stats(p);
        let (nmin, nmed, nmax, _) = stats(n);
        let d_best = nmin - pmin;
        let d_med = nmed - pmed;
        let pct_best = (nmin / pmin - 1.0) * 100.0;
        let pct_med = (nmed / pmed - 1.0) * 100.0;
        let per_blk_p = pmin / n_blk as f64;
        let per_blk_n = nmin / n_blk as f64;
        let overlap = if nmax < pmin {
            "FASTER (disjoint)"
        } else if nmin > pmax {
            "SLOWER (disjoint)"
        } else {
            "overlap"
        };
        println!(
            "  {label:10} | parent best={pmin:9.2}us med={pmed:9.2}us [{pmin:8.2}..{pmax:8.2}] ({per_blk_p:7.3} us/blk)"
        );
        println!(
            "             | #1034  best={nmin:9.2}us med={nmed:9.2}us [{nmin:8.2}..{nmax:8.2}] ({per_blk_n:7.3} us/blk)"
        );
        println!(
            "             | delta  best={d_best:+9.2}us ({pct_best:+6.2}%) med={d_med:+9.2}us ({pct_med:+6.2}%) | {overlap}"
        );
    };
    println!(
        "  --- Per-file & per-block A2 vs A3 (1T, interleaved, best of {repeats}, inner={inner}) ---"
    );
    show("A2 (setup)", &p_a2_us, &n_a2_us);
    show("A3 (syms) ", &p_a3_us, &n_a3_us);
    show("S (total) ", &p_tot_us, &n_tot_us);

    // First-block scan latency (File::open -> read -> scan blk0 -> channel handoff to worker)
    for (tag, cold) in [
        ("cold_syms (Vec::new)", true),
        ("warm_syms (reserved)", false),
    ] {
        let mut p_lat = Vec::with_capacity(repeats);
        let mut n_lat = Vec::with_capacity(repeats);
        let mut s_lat = Vec::with_capacity(repeats);
        for rep in 0..repeats {
            let order = match rep % 3 {
                0 => [0u8, 1, 2],
                1 => [1, 2, 0],
                _ => [2, 0, 1],
            };
            for arm in order {
                let us = measure_first_block_latency(path, arm, cold);
                match arm {
                    0 => p_lat.push(us),
                    1 => n_lat.push(us),
                    _ => s_lat.push(us),
                }
            }
        }
        let (pmin, pmed, pmax, _) = stats(&p_lat);
        let (nmin, nmed, nmax, _) = stats(&n_lat);
        let (smin, smed, smax, _) = stats(&s_lat);
        println!("  --- First-block latency (open->read->blk0->worker recv, {tag}, best of {repeats}) ---");
        println!("    parent  : best={pmin:8.2}us med={pmed:8.2}us [{pmin:7.2}..{pmax:7.2}]");
        println!(
            "    #1034   : best={nmin:8.2}us med={nmed:8.2}us [{nmin:7.2}..{nmax:7.2}]  delta best={:+.2}us ({:+.2}%)",
            nmin - pmin,
            (nmin / pmin - 1.0) * 100.0
        );
        println!("    shipped : best={smin:8.2}us med={smed:8.2}us [{smin:7.2}..{smax:7.2}]");
    }
}

fn main() {
    let mut repeats = 9usize;
    let mut files = Vec::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("repeats");
            i += 2;
        } else {
            files.push(args[i].clone());
            i += 1;
        }
    }
    if files.is_empty() {
        eprintln!("usage: scanner_per_block_fixed_cost [--repeats 9] FILE.bz2 [FILE.bz2 ...]");
        std::process::exit(2);
    }
    println!(
        "=== Scanner Per-Block Fixed Cost & First-Block Latency Probe (parent 5ed20c1d^ vs #1034 5ed20c1d) ==="
    );
    println!(
        "repeats={repeats} | cfg(target_feature=bmi2)={} | runtime_bmi2={}",
        cfg!(target_feature = "bmi2"),
        {
            #[cfg(target_arch = "x86_64")]
            {
                std::is_x86_feature_detected!("bmi2")
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                false
            }
        }
    );
    for f in &files {
        bench_corpus(Path::new(f), repeats);
    }
}
