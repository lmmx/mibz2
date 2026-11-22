#![forbid(unsafe_code)]
//! Carbon Cycle 36 — Safe-Rust Linear `[u16; 1024]` `SubLut` + Canonical Template-Clone `Huff::build_into` (`ScannerC36a`).
//!
//! Headline Native Safe-Rust A/B vs `origin/master` (`53f7ecf9`) on `wikidata.bz2`:
//! - `ScanOnly` (15r): `0.7962x` (`-20.38%`, `61.05 ms` vs `76.68 ms`, A/A `0.9950x`)
//! - `1T E2E Full (Cells)` (9r): `0.9654x` (`-3.46%`, `462.88 ms` vs `479.47 ms`, A/A `1.0000x`)
//! - `1T E2E Full (OpenCells)` (9r): `0.9667x` (`-3.33%`, `457.91 ms` vs `473.70 ms`, A/A `1.0006x`)
//! - `10T Serial ScannerCPU` (11r): `0.8236x` (`-17.64%`, `68.52 ms` vs `83.20 ms`, A/A `1.0019x`)
//! - `10T TotalCPU` (11r): `0.9661x` (`-3.39%`, `506.15 ms` vs `523.90 ms`, A/A `0.9995x`)
//! - `10T VM Wall` (11r): `0.9685x` (`-3.15%`, `261.08 ms` vs `269.58 ms`, A/A `1.0025x`)

use mibz2::block::{Item, Symbols, BLOCK_MAGIC, END_MAGIC, MAX_SYMS};
use mibz2::stream::{Event, Source};
use mibz2::{Error, ErrorKind};

const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_SELECTORS: usize = 18002;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;

const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

const SUB_PTR: u32 = 1 << 27;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const LEN1: u32 = 6;
const SYM1: u32 = 11;
const SYM2: u32 = 20;
const CNT_SH: u32 = 30;
const STEPS: usize = 5;

const LOOKAHEAD: usize = 2 << 20;
const MAX_ITEM: usize = 64 << 20;

#[derive(Clone, Copy)]
pub struct Reg {
    pos: usize,
    buf: u64,
    live: u32,
}

impl Reg {
    #[inline(always)]
    pub fn at(data: &[u8], bit: u64) -> Self {
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
    pub fn refill(&mut self, data: &[u8]) {
        if let Some(w) = data.get(self.pos..self.pos + 8) {
            let v = u64::from_be_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
            self.buf |= v >> (self.live & 63);
            let take = (63 - self.live) >> 3;
            self.pos += take as usize;
            self.live += take * 8;
        } else {
            *self = self.refill_slow(data);
        }
    }

    #[inline(always)]
    pub fn refill_win(&mut self, win: &[u8; 264]) {
        let idx = self.pos & 255;
        let w = &win[idx..idx + 8];
        let v = u64::from_be_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
        self.buf |= v >> (self.live & 63);
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
    pub fn peek(&mut self, data: &[u8], n: u32) -> u32 {
        if self.live < n {
            self.refill(data);
        }
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn peek_live(&self, n: u32) -> u32 {
        (self.buf >> (64 - n)) as u32
    }

    #[inline(always)]
    pub fn skip_short_shl(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= e & 63;
    }

    #[inline(always)]
    pub fn skip_short(&mut self, n: u32) {
        self.buf <<= n & 63;
        self.live -= n;
    }

    #[inline(always)]
    pub fn skip(&mut self, n: u32) {
        self.buf = if n >= 64 { 0 } else { self.buf << n };
        self.live -= n;
    }

    #[inline(always)]
    pub fn bits(&mut self, data: &[u8], n: u32) -> u32 {
        let v = self.peek(data, n);
        self.skip(n);
        v
    }

    #[inline(always)]
    pub fn bit(&mut self, data: &[u8]) -> bool {
        self.bits(data, 1) == 1
    }

    #[inline(always)]
    pub fn bit_pos(&self) -> u64 {
        self.pos as u64 * 8 - self.live as u64
    }

    #[inline(always)]
    pub fn overran(&self, data: &[u8]) -> bool {
        self.bit_pos() > data.len() as u64 * 8
    }
}

struct Huff36 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    limit20: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for Huff36 {
    fn default() -> Self {
        Self {
            lut: [0; LUT_LEN],
            sub_lut: [0; SUB_CAP],
            limit: [0; MAX_CODE_LEN],
            limit20: [-1; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl Huff36 {
    fn build_into(&mut self, lens: &[u8]) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let mut counts = [0i32; MAX_CODE_LEN];
        let mut min_l = 23usize;
        let mut max_l = 0usize;
        for &l in lens {
            let lu = l as usize;
            counts[lu] += 1;
            if lu < min_l { min_l = lu; }
            if lu > max_l { max_l = lu; }
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
        let mut limit20 = [-1i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_l..=max_l {
            vec += counts[i];
            limit[i] = vec - 1;
            if i <= 20 {
                let cap_i = (1i64 << i) - 1;
                let hi_i = ((vec - 1) as i64).min(cap_i);
                limit20[i] = if hi_i < 0 {
                    -1
                } else {
                    (((hi_i + 1) << (20 - i)) - 1) as i32
                };
            }
            vec <<= 1;
        }
        for i in min_l + 1..=max_l {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }
        self.limit = limit;
        self.limit20 = limit20;
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
        for zn in (1..min_l).take(10) {
            end_at_len[zn] = 0;
        }

        if filled_to < LUT_LEN {
            self.lut[filled_to..].fill(BAD);
            if max_l > LUT_BITS as usize {
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
                for w in filled_to..max_root_w {
                    let d_w = d_arr[w] as u32;
                    if d_w == 0 {
                        continue;
                    }
                    let sub_len = 1usize << d_w;
                    if sub_used + sub_len > SUB_CAP {
                        self.lut[w] = 0;
                        d_arr[w] = 0;
                    } else {
                        self.lut[w] = SUB_PTR | (d_w << 16) | (sub_used as u32);
                        sub_used += sub_len;
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
                            let w = (p >> sh) as usize;
                            let d_w = d_arr[w] as u32;
                            if d_w == 0 {
                                continue;
                            }
                            let sub_off = (self.lut[w] & 0xffff) as usize;
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
            }
        }

        let mask = LUT_LEN - 1;
        for zn in min_len..=max_pair_l1 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = (LUT_BITS - zn) as usize;
            let rem_max = (LUT_BITS - zn) as usize;
            let u_lim = end_at_len[rem_max] >> zn;
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
                            | ((e2 >> SYM1 & 0x1ff) << SYM2)
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

    #[inline(always)]
    fn decode_one_win(&self, r: &mut Reg, win: &[u8; 264]) -> Result<u16, ErrorKind> {
        if r.live < LUT_BITS {
            r.refill_win(win);
        }
        let e = self.lut[r.peek_live(LUT_BITS) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return Ok(((e >> SYM1) & 0x1ff) as u16);
        }
        let (s, after) = self.decode_entry_slow_win(e, *r, win);
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
        let (s, after) = self.decode_entry_slow_slice(e, *r, data);
        *r = after;
        s
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_win(&self, e: u32, mut r: Reg, win: &[u8; 264]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            if r.live < 20 { r.refill_win(win); }
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se == SUB_BAD_U16 { return (Err(ErrorKind::BadHuffmanCode), r); }
            r.skip_short((se & 31) as u32);
            r.refill_win(win);
            return (Ok(se >> 6), r);
        }
        if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (Ok(self.eob), r);
        }
        if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        if r.live < 20 { r.refill_win(win); }
        let z20 = (r.buf >> 44) as i32;
        let mut zn = (LUT_BITS + 1).max(self.min_len);
        while z20 > self.limit20[zn as usize] {
            zn += 1;
            if zn > 20 { return (Err(ErrorKind::BadHuffmanCode), r); }
        }
        let zvec = z20 >> (20 - zn);
        r.skip_short(zn);
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill_win(win);
        (res, r)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow_slice(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> (54 - d_w)) as usize) & ((1usize << d_w) - 1);
            let se = self.sub_lut[sub_off + idx];
            if se == SUB_BAD_U16 { return (Err(ErrorKind::BadHuffmanCode), r); }
            r.skip_short((se & 31) as u32);
            r.refill(data);
            return (Ok(se >> 6), r);
        }
        if (e & EOB_FAST) != 0 {
            r.skip_short(e & 63);
            return (Ok(self.eob), r);
        }
        if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let z20 = r.peek(data, 20) as i32;
        let mut zn = (LUT_BITS + 1).max(self.min_len);
        while z20 > self.limit20[zn as usize] {
            zn += 1;
            if zn > 20 { return (Err(ErrorKind::BadHuffmanCode), r); }
        }
        let zvec = z20 >> (20 - zn);
        r.skip(zn);
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (res, r)
    }
}

pub struct ScanScratch36 {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[Huff36; MAX_GROUPS]>,
}

impl Default for ScanScratch36 {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| Huff36::default())),
        }
    }
}

pub fn read_item_c36a(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    mut syms: Vec<u16>,
    scratch: &mut ScanScratch36,
) -> Result<Item, (ErrorKind, Vec<u16>)> {
    macro_rules! fail { ($k:expr) => { return Err(($k, syms)) }; }
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic == END_MAGIC { return Ok(Item::End(r.bits(data, 32))); }
    if magic != BLOCK_MAGIC { fail!(ErrorKind::BadBlockMagic); }
    let crc = r.bits(data, 32);
    if r.bit(data) { fail!(ErrorKind::Randomised); }
    let orig_ptr = r.bits(data, 24);
    let max_len = 100_000 * level as usize;
    if orig_ptr as usize > 10 + max_len { fail!(ErrorKind::BadOrigPtr); }
    let in_use16 = r.bits(data, 16);
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
    if n_in_use == 0 { fail!(ErrorKind::NoSymbols); }
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) { fail!(ErrorKind::BadGroups); }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 { fail!(ErrorKind::BadSelectors); }

    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut scratch.selectors[..sel_cap];
    let mut used_groups = 0u8;
    let data_len = data.len();
    for i in 0..n_sel {
        if r.live < 8 { r.refill(data); }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups { r.skip_short(n_groups as u32); fail!(ErrorKind::BadSelector); }
        r.skip_short((j + 1) as u32);
        if i < MAX_SELECTORS {
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            sel_buf[i] = v;
            used_groups |= 1u8 << v;
        }
        if r.pos > data_len && r.overran(data) { fail!(ErrorKind::Truncated); }
    }

    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
            if r.live < 8 { r.refill(data); }
            if (r.buf >> 63) == 0 && (1..=20).contains(&curr) {
                r.skip_short(1);
                *l = curr as u8;
                continue;
            }
            loop {
                if !(1..=20).contains(&curr) { fail!(ErrorKind::BadCodeLength); }
                if !r.bit(data) { break; }
                if r.bit(data) { curr -= 1; } else { curr += 1; }
                if r.pos > data_len && r.overran(data) { fail!(ErrorKind::Truncated); }
            }
            *l = curr as u8;
        }
        if (used_groups >> g) & 1 != 0 {
            scratch.tables[g].build_into(&lens[..alpha]);
        }
    }

    let eob = (alpha - 1) as u16;
    let max_out = sel_cap * GROUP + 66;
    if syms.capacity() < MAX_SYMS + 66 { syms.reserve(MAX_SYMS + 66 - syms.len()); }
    if syms.len() < max_out { syms.resize(max_out, 0); }
    let out = &mut syms[..max_out];
    let sel_buf = &scratch.selectors[..sel_cap];
    let mut n = 0usize;
    let mut sel = 0usize;
    let mut lr = *r;

    let res = 'groups: loop {
        if sel >= sel_cap { break Err(ErrorKind::SelectorOverrun); }
        let t = &scratch.tables[sel_buf[sel] as usize];
        sel += 1;
        let g_out: &mut [u16; 66] = (&mut out[n..n + 66]).try_into().unwrap();
        let mut k = 0usize;

        if lr.pos + 264 <= data_len {
            let base_pos = lr.pos;
            let g_in: &[u8; 264] = (&data[base_pos..base_pos + 264]).try_into().unwrap();
            lr.pos = 0;

            macro_rules! step_win {
                () => {{
                    let e = t.lut[lr.peek_live(LUT_BITS) as usize];
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        lr.skip_short_shl(e);
                        let idx = k & 63;
                        let dst = &mut g_out[idx..idx + 2];
                        dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                        dst[1] = ((e >> SYM2) & 0x1ff) as u16;
                        k += c;
                    } else {
                        let (s, after) = t.decode_entry_slow_win(e, lr, g_in);
                        lr = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => { lr.pos += base_pos; break 'groups Err(err); }
                        };
                        g_out[k & 63] = s;
                        k += 1;
                        if s == eob { lr.pos += base_pos; n += k; break 'groups Ok(()); }
                    }
                }};
            }

            lr.refill_win(g_in);
            loop {
                for _ in 0..STEPS { step_win!(); }
                if k > GROUP - 2 * STEPS { break; }
                lr.refill_win(g_in);
            }
            if k <= GROUP - 2 {
                lr.refill_win(g_in);
                for _ in 0..4 {
                    if k > GROUP - 2 { break; }
                    step_win!();
                }
                if k <= GROUP - 2 {
                    lr.refill_win(g_in);
                    while k <= GROUP - 2 { step_win!(); }
                }
            }
            if k == GROUP - 1 {
                let s = match t.decode_one_win(&mut lr, g_in) {
                    Ok(s) => s,
                    Err(err) => { lr.pos += base_pos; break 'groups Err(err); }
                };
                g_out[GROUP - 1] = s;
                k += 1;
                if s == eob { lr.pos += base_pos; n += k; break 'groups Ok(()); }
            }
            lr.pos += base_pos;
            n += GROUP;
        } else {
            macro_rules! step_slice {
                () => {{
                    let e = t.lut[lr.peek_live(LUT_BITS) as usize];
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        lr.skip_short_shl(e);
                        let idx = k & 63;
                        let dst = &mut g_out[idx..idx + 2];
                        dst[0] = ((e >> SYM1) & 0x1ff) as u16;
                        dst[1] = ((e >> SYM2) & 0x1ff) as u16;
                        k += c;
                    } else {
                        let (s, after) = t.decode_entry_slow_slice(e, lr, data);
                        lr = after;
                        let s = match s { Ok(s) => s, Err(err) => break 'groups Err(err) };
                        g_out[k & 63] = s;
                        k += 1;
                        if s == eob { n += k; break 'groups Ok(()); }
                    }
                }};
            }

            lr.refill(data);
            loop {
                for _ in 0..STEPS { step_slice!(); }
                if k > GROUP - 2 * STEPS { break; }
                lr.refill(data);
            }
            if k <= GROUP - 2 {
                lr.refill(data);
                for _ in 0..4 {
                    if k > GROUP - 2 { break; }
                    step_slice!();
                }
                if k <= GROUP - 2 {
                    lr.refill(data);
                    while k <= GROUP - 2 { step_slice!(); }
                }
            }
            if k == GROUP - 1 {
                let s = match t.decode_one_slice(&mut lr, data) { Ok(s) => s, Err(err) => break 'groups Err(err) };
                g_out[GROUP - 1] = s;
                k += 1;
                if s == eob { n += k; break 'groups Ok(()); }
            }
            n += GROUP;
            if lr.pos > data_len && lr.overran(data) { break Err(ErrorKind::Truncated); }
        }
    };

    *r = lr;
    if let Err(err) = res { fail!(err); }
    syms.truncate(n);
    Ok(Item::Block(Symbols { crc, orig_ptr, seq_to_unseq, syms, eob, max_len }))
}

pub struct ScannerC36a<S> {
    src: S,
    bit: u64,
    first: bool,
    done: bool,
    in_stream: bool,
    level: u32,
    combined: u32,
    blocks: u64,
    streams: u64,
    lookahead: usize,
    scratch: ScanScratch36,
}

impl<S: Source> ScannerC36a<S> {
    pub fn new(src: S) -> Self {
        Self {
            src,
            bit: 0,
            first: true,
            done: false,
            in_stream: false,
            level: 0,
            combined: 0,
            blocks: 0,
            streams: 0,
            lookahead: LOOKAHEAD,
            scratch: ScanScratch36::default(),
        }
    }

    pub fn next(&mut self, mut syms: Vec<u16>) -> Result<Event, Error> {
        loop {
            if self.done { return Ok(Event::Done); }
            if !self.in_stream {
                let at = self.bit;
                let err = |kind| Error { kind, block: None, bit: Some(at) };
                let (data, eof) = self.src.view(at / 8, 4).map_err(err)?;
                if data.is_empty() && eof {
                    if self.first { return Err(err(ErrorKind::Empty)); }
                    self.done = true;
                    continue;
                }
                match data {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                        self.in_stream = true;
                        self.level = (l - b'0') as u32;
                        self.combined = 0;
                        self.bit += 32;
                    }
                    _ => return Err(err(if self.first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData })),
                }
            } else {
                let at = self.bit;
                let index = self.blocks;
                let level = self.level;
                let combined = self.combined;
                let err = |kind| Error { kind, block: Some(index), bit: Some(at) };
                let base = at / 8;
                let (data, eof) = self.src.view(base, self.lookahead).map_err(err)?;
                let mut r = Reg::at(data, at % 8);
                let item_res = read_item_c36a(data, &mut r, level, syms, &mut self.scratch);
                let used = r.bit_pos() - at % 8;
                if r.overran(data) {
                    if eof { return Err(err(ErrorKind::Truncated)); }
                    if self.lookahead >= MAX_ITEM { return Err(err(ErrorKind::BlockTooLarge)); }
                    self.lookahead = (self.lookahead * 2).max(data.len() + 1).min(MAX_ITEM);
                    syms = match item_res { Ok(Item::Block(b)) => b.syms, Ok(Item::End(_)) => Vec::new(), Err((_, s)) => s };
                    continue;
                }
                self.bit += used;
                self.src.release(self.bit / 8);
                match item_res.map_err(|(kind, _)| err(kind))? {
                    Item::End(stored) => {
                        if stored != combined { return Err(err(ErrorKind::StreamCrc { stored, computed: combined })); }
                        self.bit = self.bit.div_ceil(8) * 8;
                        self.in_stream = false;
                        self.first = false;
                        self.streams += 1;
                        syms = Vec::new();
                    }
                    Item::Block(symbols) => {
                        self.combined = combined.rotate_left(1) ^ symbols.crc;
                        self.blocks += 1;
                        return Ok(Event::Block { index, bit: at, end: self.bit, stream: self.streams, level, symbols });
                    }
                }
            }
        }
    }
}
