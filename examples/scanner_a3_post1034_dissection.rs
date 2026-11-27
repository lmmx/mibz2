#![deny(unsafe_op_in_unsafe_fn)]
//! (research only, `explore`) Post-`#1034` deep dissection & redesign probe for `A3`
//! (`decode_runs_bmi2` / `decode_runs_scalar` + `Huff::decode_run` + `Huff::build_into`).
//!
//!     cargo run --release --example scanner_a3_post1034_dissection -- /path/to/wikidata.bz2 [REPEATS]
//!
//! Runs:
//! 1. Exact 6-corpus census (`wikidata` + `ab gen` `text`, `records`, `random`, `runs`, `onebyte`).
//! 2. 100% exactness verification (symbol-for-symbol & bit_pos parity vs shipped
//!    `mibz2::block::read_item_with_scratch`) across all 431 wikidata blocks, all 5 guard
//!    corpora, and synthetic `alpha = 258` edge cases.
//! 3. 7-repeat interleaved rotated timings (sub-microsecond `CLOCK_THREAD_CPUTIME_ID` + wall)
//!    with ZERO `Reg` stack spills across every arm.

#[path = "../tests/common/mod.rs"]
mod common;

use mibz2::bits::Bits;
use mibz2::block::{read_item_with_scratch, Item, ScanScratch};
use mibz2::ErrorKind;
use std::hint::black_box;
use std::time::Instant;

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23;
const MAX_SELECTORS: usize = 18_002;
const GROUP: usize = 50;
const PAIR_GATE_SEL: usize = 64;
const K3_GATE_SEL: usize = 1024;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;

const CHUNK_LEN: usize = 1042;
const CHUNK_MASK: usize = 1023;
const CHUNK_STOP: usize = 1000;
const PATCH_CAP: usize = 32;

// Shipped #1034 B=10 K=2 LUT layout
const SYM256: u32 = 1 << 26;
const SUB_PTR: u32 = 1 << 27;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const SYM1: u32 = 8;
const SYM2: u32 = 16;
const LEN1: u32 = 24;
const CNT_SH: u32 = 30;

// High-2-bit cnt K=3 single-u32-store LUT layout (B=10 or B=9):
// - bits 30..31: cnt (1, 2, 3 for fast; 0 for slow tags) -> single `shr $30` + `je slow`!
// - bits 22..29: sym3 (0..=255)
// - bits 14..21: sym2 (0..=255)
// - bits  6..13: sym1 (0..=255)
//   => `(e >> 6)` places [sym1, sym2, sym3, cnt] in bytes 0..3 -> ONE `shr $6` + ONE `movl`!
// - bits  4..5 : ALWAYS 0 for fast entries -> `e & 63 == total_bits` -> ZERO `and` on `shlx`!
// - bits  0..3 : total_bits (1..=10)
// When cnt == 0 (bits 30..31 == 0):
// - bit 29: EOB_FAST (low 6 bits = zn)
// - bit 28: BAD
// - bit 27: SUB_PTR (d_w in 16..19, sub_off in 0..15)
// - bit 26: SYM256 (low 6 bits = zn)
const K3H_SYM1: u32 = 6;
const K3H_SYM2: u32 = 14;
const K3H_SYM3: u32 = 22;

// ---- High-resolution thread CPU timer (CLOCK_THREAD_CPUTIME_ID via vDSO) ----

#[repr(C)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

const CLOCK_THREAD_CPUTIME_ID: i32 = 3;

extern "C" {
    fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
}

#[inline(always)]
fn thread_cpu_ns() -> u64 {
    let mut ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a valid stack pointer to a `Timespec`.
    let rc = unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut ts) };
    if rc == 0 {
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    } else {
        0
    }
}

// ---- Bit reader (exact mirror of src/bits.rs) ----

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
    fn skip_short_shl_u8(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= (e as u8) as u32;
    }

    #[inline(always)]
    fn skip_short_shl_low4(&mut self, e: u32) {
        self.buf = self.buf.wrapping_shl(e);
        self.live -= ((e as u8) & 15) as u32;
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

// ---- Pre-parsed block metadata ----

struct ParsedBlock {
    start_bit: u64,
    end_bit: u64,
    alpha: usize,
    n_groups: usize,
    used_groups: u8,
    selectors: Vec<u8>,
    lens: [[u8; MAX_ALPHA]; MAX_GROUPS],
    expected_syms: Vec<u16>,
}

struct CorpusData {
    name: &'static str,
    raw_bz2: Vec<u8>,
    blocks: Vec<ParsedBlock>,
    total_syms: usize,
}

fn parse_corpus(name: &'static str, raw_bz2: Vec<u8>) -> CorpusData {
    let mut blocks = Vec::new();
    let mut scratch = ScanScratch::default();
    let mut syms_buf = Vec::new();
    let mut bit = 0u64;
    let mut total_syms = 0usize;

    while (bit / 8) < raw_bz2.len() as u64 {
        let byte_pos = (bit / 8) as usize;
        if byte_pos + 4 > raw_bz2.len() {
            break;
        }
        assert_eq!(&raw_bz2[byte_pos..byte_pos + 3], b"BZh");
        let level = (raw_bz2[byte_pos + 3] - b'0') as u32;
        bit += 32;

        loop {
            let mut br = Bits::at(&raw_bz2, bit);
            let item =
                read_item_with_scratch(&mut br, level, std::mem::take(&mut syms_buf), &mut scratch)
                    .expect("shipped read_item_with_scratch");
            let end_bit = br.bit_pos();
            match item {
                Item::End(_) => {
                    bit = end_bit.div_ceil(8) * 8;
                    break;
                }
                Item::Block(s) => {
                    let mut r = Reg::at(&raw_bz2, bit);
                    let magic = (r.bits(&raw_bz2, 24) as u64) << 24 | r.bits(&raw_bz2, 24) as u64;
                    assert_eq!(magic, BLOCK_MAGIC);
                    let _crc = r.bits(&raw_bz2, 32);
                    let _rand = r.bit(&raw_bz2);
                    let _orig = r.bits(&raw_bz2, 24);
                    let in_use16 = r.bits(&raw_bz2, 16) as u16;
                    let mut n_in_use = 0usize;
                    for i in 0..16 {
                        if in_use16 & (0x8000 >> i) != 0 {
                            let w = r.bits(&raw_bz2, 16) as u16;
                            n_in_use += w.count_ones() as usize;
                        }
                    }
                    let alpha = n_in_use + 2;
                    let n_groups = r.bits(&raw_bz2, 3) as usize;
                    let n_sel = r.bits(&raw_bz2, 15) as usize;
                    let sel_cap = n_sel.min(MAX_SELECTORS);
                    let mut selectors = vec![0u8; sel_cap];
                    let mut mtf: u64 = 0x0000_0504_0302_0100;
                    let mut used_groups = 0u8;
                    for i in 0..n_sel {
                        if r.live < 8 {
                            r.refill(&raw_bz2);
                        }
                        let j = (!r.buf).leading_zeros() as usize;
                        r.skip_short((j + 1) as u32);
                        if let Some(slot) = selectors.get_mut(i) {
                            let sh = (j * 8) as u32;
                            let v = ((mtf >> sh) & 0xff) as u8;
                            let low_mask = (1u64 << sh) - 1;
                            mtf = (mtf & !((low_mask << 8) | 0xff))
                                | ((mtf & low_mask) << 8)
                                | (v as u64);
                            *slot = v;
                            used_groups |= 1u8 << v;
                        }
                    }
                    let mut lens = [[0u8; MAX_ALPHA]; MAX_GROUPS];
                    for group_lens in lens.iter_mut().take(n_groups) {
                        let mut curr = r.bits(&raw_bz2, 5) as i32;
                        for l in group_lens[..alpha].iter_mut() {
                            loop {
                                if !r.bit(&raw_bz2) {
                                    break;
                                }
                                if r.bit(&raw_bz2) {
                                    curr -= 1;
                                } else {
                                    curr += 1;
                                }
                            }
                            *l = curr as u8;
                        }
                    }
                    let sym_start_bit = r.bit_pos();
                    total_syms += s.syms.len();
                    blocks.push(ParsedBlock {
                        start_bit: sym_start_bit,
                        end_bit,
                        alpha,
                        n_groups,
                        used_groups,
                        selectors,
                        lens,
                        expected_syms: s.syms.clone(),
                    });
                    syms_buf = s.syms;
                    bit = end_bit;
                }
            }
        }
    }

    CorpusData {
        name,
        raw_bz2,
        blocks,
        total_syms,
    }
}

// ---- Generic K=2 Huff table (B=10) ----

struct HuffK2<const B: usize, const L: usize> {
    lut: [u32; L],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl<const B: usize, const L: usize> Default for HuffK2<B, L> {
    fn default() -> Self {
        Self {
            lut: [0; L],
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

impl<const B: usize, const L: usize> HuffK2<B, L> {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    fn build_into(&mut self, lens: &[u8], build_pairs: bool) {
        let lut_bits = B as u32;
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
        let max_pair_l1 = lut_bits.saturating_sub(min_len);
        let mut end_at_len = [0usize; 16];
        let mut zn_ranges = [(0i64, -1i64); 16];
        let mut a = 0i64;
        let mut filled_to = 0usize;

        for zn in min_len..=lut_bits {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            let sh = lut_bits - zn;
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

        if filled_to < L {
            if max_l <= B {
                self.lut[filled_to..].fill(BAD);
            } else {
                self.lut[filled_to..].fill(0);
                let mut ranges = [(0u32, 0i64, -1i64); 12];
                let mut d_arr = [0u8; L];
                let mut max_root_w = filled_to;
                let start_zn = (lut_bits + 1).max(min_len);
                let mut a_hi = a;
                for zn in start_zn..=(max_l as u32) {
                    let top = (1i64 << zn) - 1;
                    let hi = (self.limit[zn as usize] as i64).min(top);
                    ranges[(zn - (lut_bits + 1)) as usize] = (zn, a_hi, hi);
                    if hi >= a_hi {
                        let sh = zn - lut_bits;
                        let w_lo = (a_hi >> sh) as usize;
                        let w_hi = ((hi >> sh) as usize).min(L - 1);
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
                        let sh = zn - lut_bits;
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
        let mask = L - 1;
        for zn in min_len..=max_pair_l1 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = B - zn as usize;
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
                        if (e2 >> CNT_SH) == 0 || (zn as usize) + l2 > B {
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
    fn decode_one_val(&self, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        let e = self.lut[r.peek(data, B as u32) as usize];
        if (e >> CNT_SH) != 0 {
            r.skip_short((e >> LEN1) & 31);
            return (Ok(((e >> SYM1) as u8) as u16), r);
        }
        self.decode_entry_slow(e, r, data)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> ((64 - B as u32) - d_w)) as usize) & ((1usize << d_w) - 1);
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
            r.refill(data);
            return (Ok(256), r);
        } else if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let mut zn = self.min_len;
        let mut zvec = r.bits(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return (Err(ErrorKind::BadHuffmanCode), r);
            }
            zvec = (zvec << 1) | r.bit(data) as i32;
        }
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (res, r)
    }
}

// ---- High-2-bit cnt K=3 Huff table (B=10 or B=9), 100% in-place zero-copy build ----

struct HuffK3Hi<const B: usize, const L: usize> {
    lut: [u32; L],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    sym_len: [u8; 256],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl<const B: usize, const L: usize> Default for HuffK3Hi<B, L> {
    fn default() -> Self {
        Self {
            lut: [0; L],
            sub_lut: [0; SUB_CAP],
            limit: [0; MAX_CODE_LEN],
            base: [0; MAX_CODE_LEN],
            perm: [0; MAX_ALPHA],
            sym_len: [0; 256],
            min_len: 1,
            alpha: 0,
            eob: 0,
        }
    }
}

impl<const B: usize, const L: usize> HuffK3Hi<B, L> {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// 100% in-place zero-copy K=1/K=2/K=3 builder with step-jumping template fill and
    /// non-aliasing `split_at_mut` `wrapping_add(delta)` cloning.
    fn build_into_fast(&mut self, lens: &[u8], build_k2: bool, build_k3: bool) {
        let lut_bits = B as u32;
        let mask = L - 1;
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let copy_n = alpha.min(256);
        self.sym_len[..copy_n].copy_from_slice(&lens[..copy_n]);
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
        let mut end_at_len = [0usize; 12];
        let mut zn_ranges = [(0i64, -1i64); 12];
        let mut a = 0i64;
        let mut filled_to = 0usize;

        for zn in min_len..=lut_bits {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            let sh = lut_bits - zn;
            for p in a..=hi {
                let e = match self.symbol(zn, p as i32) {
                    Some(s) if s == eob => EOB_FAST | zn,
                    Some(256) => SYM256 | zn,
                    Some(s) => (1u32 << CNT_SH) | ((s as u32) << K3H_SYM1) | zn,
                    None => BAD,
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }
        self.fill_tail_and_sublut(filled_to, max_l, min_len, a);

        if !build_k2 || min_len * 2 > lut_bits {
            return;
        }

        // Pass 2 (100% in-place): K=1 -> K=2 via aligned step-jumping fill + `split_at_mut` clone.
        let max_l1_k2 = lut_bits - min_len;
        for zn in min_len..=max_l1_k2 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = B - zn as usize;
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
                let s1 = (e0 >> K3H_SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << K3H_SYM1;
                    let (left, right) = self.lut.split_at_mut(off_p);
                    let src = &left[off0..off0 + u_lim];
                    let dst = &mut right[..u_lim];
                    for (d, &s) in dst.iter_mut().zip(src) {
                        *d = s.wrapping_add(delta);
                    }
                } else {
                    let mut u = 0usize;
                    while u < u_lim {
                        let e2 = self.lut[(u << zn) & mask];
                        if (e2 >> CNT_SH) == 0 {
                            u += 1;
                            continue;
                        }
                        let s2 = (e2 >> K3H_SYM1) & 0xff;
                        let l2 = self.sym_len[s2 as usize] as u32;
                        let tot = zn + l2;
                        if tot <= lut_bits {
                            let step = 1usize << (lut_bits - tot);
                            let end_u = (u + step).min(u_lim);
                            let entry =
                                (2u32 << CNT_SH) | (s2 << K3H_SYM2) | (s1 << K3H_SYM1) | tot;
                            self.lut[off_p + u..off_p + end_u].fill(entry);
                            u = end_u;
                        } else {
                            u += 1;
                        }
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }

        // Pass 3 (100% in-place, gated on block size): K=2 -> K=3 via step-jumping fill + clone.
        if !build_k3 || min_len * 3 > lut_bits {
            return;
        }
        let max_l1_k3 = lut_bits - 2 * min_len;
        for zn in min_len..=max_l1_k3 {
            let (lo_p, hi_p) = zn_ranges[zn as usize];
            if lo_p > hi_p {
                continue;
            }
            let sh = B - zn as usize;
            let max_l2 = (lut_bits - zn - min_len) as usize;
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
                let s1 = (e0 >> K3H_SYM1) & 0xff;
                if let Some((off0, s0)) = tpl {
                    let delta = s1.wrapping_sub(s0) << K3H_SYM1;
                    let (left, right) = self.lut.split_at_mut(off_p);
                    let src = &left[off0..off0 + u_lim];
                    let dst = &mut right[..u_lim];
                    for (d, &s) in dst.iter_mut().zip(src) {
                        *d = s.wrapping_add(delta);
                    }
                } else {
                    let mut u = 0usize;
                    while u < u_lim {
                        let e23 = self.lut[(u << zn) & mask];
                        let sc = e23 >> CNT_SH;
                        if sc < 2 {
                            u += 1;
                            continue;
                        }
                        let s2 = (e23 >> K3H_SYM1) & 0xff;
                        let s3 = (e23 >> K3H_SYM2) & 0xff;
                        let l23 = if sc == 2 {
                            e23 & 15
                        } else {
                            (self.sym_len[s2 as usize] + self.sym_len[s3 as usize]) as u32
                        };
                        let tot = zn + l23;
                        if tot <= lut_bits {
                            let step = 1usize << (lut_bits - tot);
                            let end_u = (u + step).min(u_lim);
                            let entry = (3u32 << CNT_SH)
                                | (s3 << K3H_SYM3)
                                | (s2 << K3H_SYM2)
                                | (s1 << K3H_SYM1)
                                | tot;
                            self.lut[off_p + u..off_p + end_u].fill(entry);
                            u = end_u;
                        } else {
                            u += 1;
                        }
                    }
                    tpl = Some((off_p, s1));
                }
            }
        }
    }

    fn fill_tail_and_sublut(&mut self, filled_to: usize, max_l: usize, min_len: u32, a: i64) {
        let lut_bits = B as u32;
        if filled_to >= L {
            return;
        }
        if max_l <= B {
            self.lut[filled_to..].fill(BAD);
            return;
        }
        self.lut[filled_to..].fill(0);
        let mut ranges = [(0u32, 0i64, -1i64); 12];
        let mut d_arr = [0u8; L];
        let mut max_root_w = filled_to;
        let start_zn = (lut_bits + 1).max(min_len);
        let mut a_hi = a;
        for zn in start_zn..=(max_l as u32) {
            let top = (1i64 << zn) - 1;
            let hi = (self.limit[zn as usize] as i64).min(top);
            ranges[(zn - (lut_bits + 1)) as usize] = (zn, a_hi, hi);
            if hi >= a_hi {
                let sh = zn - lut_bits;
                let w_lo = (a_hi >> sh) as usize;
                let w_hi = ((hi >> sh) as usize).min(L - 1);
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
                let sh = zn - lut_bits;
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
    }

    #[inline(always)]
    fn decode_one_val(&self, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        let e = self.lut[r.peek(data, B as u32) as usize];
        if (e >> CNT_SH) != 0 {
            let s1 = ((e >> K3H_SYM1) & 0xff) as usize;
            r.skip_short(self.sym_len[s1] as u32);
            return (Ok(s1 as u16), r);
        }
        self.decode_entry_slow(e, r, data)
    }

    #[cold]
    #[inline(never)]
    fn decode_entry_slow(&self, e: u32, mut r: Reg, data: &[u8]) -> (Result<u16, ErrorKind>, Reg) {
        if (e & SUB_PTR) != 0 {
            let _ = r.peek(data, 20);
            let sub_off = (e & 0xffff) as usize;
            let d_w = (e >> 16) & 15;
            let idx = ((r.buf >> ((64 - B as u32) - d_w)) as usize) & ((1usize << d_w) - 1);
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
            r.refill(data);
            return (Ok(256), r);
        } else if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let mut zn = self.min_len;
        let mut zvec = r.bits(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return (Err(ErrorKind::BadHuffmanCode), r);
            }
            zvec = (zvec << 1) | r.bit(data) as i32;
        }
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (res, r)
    }
}

// ---- Split hot/cold K=3 tables (6x4KiB LUTs contiguous 64B-aligned) ----

#[repr(align(64))]
struct SplitHotLuts {
    luts: [[u32; 1024]; MAX_GROUPS],
}

impl Default for SplitHotLuts {
    fn default() -> Self {
        Self {
            luts: [[0; 1024]; MAX_GROUPS],
        }
    }
}

// ---- Bulk u8 -> u16 flush with zero-cost sym256 patch list ----

struct PatchBuf {
    pos: [u16; PATCH_CAP],
    len: usize,
}

impl PatchBuf {
    #[inline(always)]
    fn new() -> Self {
        Self {
            pos: [0; PATCH_CAP],
            len: 0,
        }
    }

    #[cold]
    #[inline(never)]
    fn push_if_room(&mut self, cur_len: usize) -> bool {
        if self.len < PATCH_CAP {
            self.pos[self.len] = cur_len as u16;
            self.len += 1;
            true
        } else {
            false
        }
    }

    #[inline(never)]
    fn flush(&mut self, chunk: &[u8], syms: &mut Vec<u16>) {
        flush_u8_patched(chunk, &self.pos[..self.len], syms);
        self.len = 0;
    }
}

#[inline(never)]
fn flush_u8_patched(chunk: &[u8], patches: &[u16], syms: &mut Vec<u16>) {
    let base = syms.len();
    syms.reserve(chunk.len());
    syms.extend(chunk.iter().map(|&b| b as u16));
    for &p in patches {
        syms[base + p as usize] = 256;
    }
}

// ============================================================================
// BMI2 runtime dispatch helper (passes pos/buf/live as 3 scalars => ZERO stack spills!)
// ============================================================================

macro_rules! dispatch_bmi2_reg {
    ($fn_name:ident, $bmi2_name:ident, $inner:ident, ( $($arg:ident : $ty:ty),* ) -> $ret:ty) => {
        #[cfg(target_arch = "x86_64")]
        #[target_feature(enable = "bmi2")]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        unsafe fn $bmi2_name($($arg: $ty,)* pos: usize, buf: u64, live: u32) -> ($ret, Reg) {
            $inner($($arg,)* Reg { pos, buf, live })
        }

        #[inline(never)]
        fn $fn_name($($arg: $ty,)* r: Reg) -> ($ret, Reg) {
            #[cfg(target_arch = "x86_64")]
            if std::is_x86_feature_detected!("bmi2") {
                return unsafe { $bmi2_name($($arg,)* r.pos, r.buf, r.live) };
            }
            $inner($($arg,)* r)
        }
    };
}

// ---- R0_shipped: exact mirror of shipped #1034 ----

#[inline(always)]
fn run_shipped_inner(
    tables: &[HuffK2<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
    mut r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut ck = 0usize;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
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
            let mut rem_syms = take * GROUP;
            let eob = t.eob;
            r.refill(data);
            while rem_syms >= 10 {
                let stop_ck = (ck + (rem_syms - 10)).min(CHUNK_STOP);
                let before_ck = ck;
                while ck <= stop_ck {
                    let batch: &mut [u16; 18] = (&mut chunk[ck..ck + 18]).try_into().unwrap();
                    let mut bk = 0usize;
                    for _ in 0..5 {
                        let e = t.lut[r.peek_live(10) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = bk & 15;
                            let dst = &mut batch[idx..idx + 2];
                            dst[0] = ((e >> SYM1) as u8) as u16;
                            dst[1] = ((e >> SYM2) as u8) as u16;
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            batch[bk & 15] = s;
                            bk += 1;
                            if s == eob {
                                syms.extend_from_slice(&chunk[..ck + bk]);
                                return (Ok(()), r);
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
                if rem_syms >= 2 {
                    let tail_stop = target_ck - 2;
                    while ck <= tail_stop {
                        if r.live < 10 {
                            r.refill(data);
                        }
                        let e = t.lut[r.peek_live(10) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = ck & CHUNK_MASK;
                            let dst = &mut chunk[idx..idx + 2];
                            dst[0] = ((e >> SYM1) as u8) as u16;
                            dst[1] = ((e >> SYM2) as u8) as u16;
                            ck += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            chunk[ck & CHUNK_MASK] = s;
                            ck += 1;
                            if s == eob {
                                syms.extend_from_slice(&chunk[..ck]);
                                return (Ok(()), r);
                            }
                        }
                    }
                }
                let last = target_ck - 1;
                if ck == last {
                    let (s, after) = t.decode_one_val(r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => return (Err(err), r),
                    };
                    chunk[last & CHUNK_MASK] = s;
                    ck += 1;
                    if s == eob {
                        syms.extend_from_slice(&chunk[..ck]);
                        return (Ok(()), r);
                    }
                }
                if ck > CHUNK_STOP {
                    syms.extend_from_slice(&chunk[..ck]);
                    ck = 0;
                }
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

dispatch_bmi2_reg!(
    run_shipped,
    run_shipped_bmi2,
    run_shipped_inner,
    (tables: &[HuffK2<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u16; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

// ---- Diagnostics: D1_k2_nostore & D2_k3_nostore (BOTH 100% spill-free!) ----

#[inline(always)]
fn run_d1_k2_nostore_inner(
    tables: &[HuffK2<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    mut r: Reg,
) -> (u64, Reg) {
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut acc = 0u64;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
        let eob = t.eob;
        let mut rem_syms = rem * GROUP;
        r.refill(data);
        while rem_syms >= 10 {
            let mut got = 0usize;
            for _ in 0..5 {
                let e = t.lut[r.peek_live(10) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_u8(e);
                    acc ^= e as u64;
                    got += c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    let Ok(s) = s else { return (acc, r) };
                    acc ^= s as u64;
                    got += 1;
                    if s == eob {
                        return (acc, r);
                    }
                }
            }
            rem_syms -= got;
            r.refill(data);
        }
        while rem_syms >= 2 {
            if r.live < 10 {
                r.refill(data);
            }
            let e = t.lut[r.peek_live(10) as usize];
            let c = (e >> CNT_SH) as usize;
            if c != 0 {
                r.skip_short_shl_u8(e);
                acc ^= e as u64;
                rem_syms -= c;
            } else {
                let (s, after) = t.decode_entry_slow(e, r, data);
                r = after;
                let Ok(s) = s else { return (acc, r) };
                acc ^= s as u64;
                rem_syms -= 1;
                if s == eob {
                    return (acc, r);
                }
            }
        }
        if rem_syms == 1 {
            let (s, after) = t.decode_one_val(r, data);
            r = after;
            let Ok(s) = s else { return (acc, r) };
            acc ^= s as u64;
            if s == eob {
                return (acc, r);
            }
        }
    }
    (acc, r)
}

#[inline(always)]
fn run_d2_k3_nostore_inner(
    tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    mut r: Reg,
) -> (u64, Reg) {
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut acc = 0u64;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
        let eob = t.eob;
        let mut rem_syms = rem * GROUP;
        r.refill(data);
        while rem_syms >= 12 {
            let mut got = 0usize;
            for _ in 0..3 {
                let e = t.lut[r.peek_live(10) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_low4(e);
                    acc ^= e as u64;
                    got += c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    let Ok(s) = s else { return (acc, r) };
                    acc ^= s as u64;
                    got += 1;
                    if s == eob {
                        return (acc, r);
                    }
                }
            }
            let e = t.lut[r.peek_live(10) as usize];
            r.refill(data);
            let c = (e >> CNT_SH) as usize;
            if c != 0 {
                r.skip_short_shl_low4(e);
                acc ^= e as u64;
                got += c;
            } else {
                let (s, after) = t.decode_entry_slow(e, r, data);
                r = after;
                let Ok(s) = s else { return (acc, r) };
                acc ^= s as u64;
                got += 1;
                if s == eob {
                    return (acc, r);
                }
            }
            rem_syms -= got;
        }
        while rem_syms >= 3 {
            if r.live < 10 {
                r.refill(data);
            }
            let e = t.lut[r.peek_live(10) as usize];
            let c = (e >> CNT_SH) as usize;
            if c != 0 {
                r.skip_short_shl_low4(e);
                acc ^= e as u64;
                rem_syms -= c;
            } else {
                let (s, after) = t.decode_entry_slow(e, r, data);
                r = after;
                let Ok(s) = s else { return (acc, r) };
                acc ^= s as u64;
                rem_syms -= 1;
                if s == eob {
                    return (acc, r);
                }
            }
        }
        while rem_syms > 0 {
            let (s, after) = t.decode_one_val(r, data);
            r = after;
            let Ok(s) = s else { return (acc, r) };
            acc ^= s as u64;
            rem_syms -= 1;
            if s == eob {
                return (acc, r);
            }
        }
    }
    (acc, r)
}

dispatch_bmi2_reg!(
    run_d1_k2_nostore,
    run_d1_k2_nostore_bmi2,
    run_d1_k2_nostore_inner,
    (tables: &[HuffK2<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8]) -> u64
);

dispatch_bmi2_reg!(
    run_d2_k3_nostore,
    run_d2_k3_nostore_bmi2,
    run_d2_k3_nostore_inner,
    (tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8]) -> u64
);

// ---- C1_pipe4_u16: K=2 pipelined 4-step overlapped refill, u16 chunk, ZERO spills ----

#[inline(always)]
fn run_c1_pipe4_u16_inner(
    tables: &[HuffK2<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u16; CHUNK_LEN],
    syms: &mut Vec<u16>,
    mut r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut ck = 0usize;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
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
            let mut rem_syms = take * GROUP;
            let eob = t.eob;
            r.refill(data);
            while rem_syms >= 8 {
                let stop_ck = (ck + (rem_syms - 8)).min(CHUNK_STOP);
                let before_ck = ck;
                while ck <= stop_ck {
                    let batch: &mut [u16; 18] = (&mut chunk[ck..ck + 18]).try_into().unwrap();
                    let mut bk = 0usize;
                    for _ in 0..3 {
                        let e = t.lut[r.peek_live(10) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = bk & 15;
                            let dst = &mut batch[idx..idx + 2];
                            dst[0] = ((e >> SYM1) as u8) as u16;
                            dst[1] = ((e >> SYM2) as u8) as u16;
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            batch[bk & 15] = s;
                            bk += 1;
                            if s == eob {
                                syms.extend_from_slice(&chunk[..ck + bk]);
                                return (Ok(()), r);
                            }
                        }
                    }
                    let e = t.lut[r.peek_live(10) as usize];
                    r.refill(data);
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        r.skip_short_shl_u8(e);
                        let idx = bk & 15;
                        let dst = &mut batch[idx..idx + 2];
                        dst[0] = ((e >> SYM1) as u8) as u16;
                        dst[1] = ((e >> SYM2) as u8) as u16;
                        bk += c;
                    } else {
                        let (s, after) = t.decode_entry_slow(e, r, data);
                        r = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => return (Err(err), r),
                        };
                        batch[bk & 15] = s;
                        bk += 1;
                        if s == eob {
                            syms.extend_from_slice(&chunk[..ck + bk]);
                            return (Ok(()), r);
                        }
                    }
                    ck += bk;
                }
                rem_syms -= ck - before_ck;
                if ck > CHUNK_STOP {
                    syms.extend_from_slice(&chunk[..ck]);
                    ck = 0;
                }
            }
            if rem_syms > 0 {
                let target_ck = ck + rem_syms;
                if rem_syms >= 2 {
                    let tail_stop = target_ck - 2;
                    while ck <= tail_stop {
                        if r.live < 10 {
                            r.refill(data);
                        }
                        let e = t.lut[r.peek_live(10) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = ck & CHUNK_MASK;
                            let dst = &mut chunk[idx..idx + 2];
                            dst[0] = ((e >> SYM1) as u8) as u16;
                            dst[1] = ((e >> SYM2) as u8) as u16;
                            ck += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            chunk[ck & CHUNK_MASK] = s;
                            ck += 1;
                            if s == eob {
                                syms.extend_from_slice(&chunk[..ck]);
                                return (Ok(()), r);
                            }
                        }
                    }
                }
                let last = target_ck - 1;
                if ck == last {
                    let (s, after) = t.decode_one_val(r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => return (Err(err), r),
                    };
                    chunk[last & CHUNK_MASK] = s;
                    ck += 1;
                    if s == eob {
                        syms.extend_from_slice(&chunk[..ck]);
                        return (Ok(()), r);
                    }
                }
                if ck > CHUNK_STOP {
                    syms.extend_from_slice(&chunk[..ck]);
                    ck = 0;
                }
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

dispatch_bmi2_reg!(
    run_c1_pipe4_u16,
    run_c1_pipe4_u16_bmi2,
    run_c1_pipe4_u16_inner,
    (tables: &[HuffK2<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u16; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

// ---- C2/C3: K=2 u8-chunk single-u16-store + sym256 patch list (s5 vs pipe4), ZERO spills ----

#[inline(always)]
fn run_k2_u8_inner<const B: usize, const L: usize, const PIPE: bool>(
    tables: &[HuffK2<B, L>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    mut r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut ck = 0usize;
    let mut patches = [0u16; PATCH_CAP];
    let mut np = 0usize;
    let max_batch_syms = if PIPE { 8 } else { 10 };
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
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
            let mut rem_syms = take * GROUP;
            let eob = t.eob;
            r.refill(data);
            while rem_syms >= max_batch_syms {
                let stop_ck = (ck + (rem_syms - max_batch_syms)).min(CHUNK_STOP);
                let mut done = 0usize;
                'batch: while ck <= stop_ck {
                    let batch: &mut [u8; 18] = (&mut chunk[ck..ck + 18]).try_into().unwrap();
                    let mut bk = 0usize;
                    let pre = if PIPE { 3 } else { 5 };
                    for _ in 0..pre {
                        let e = t.lut[r.peek_live(B as u32) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = bk & 15;
                            let pair = ((e >> 8) as u16).to_le_bytes();
                            batch[idx..idx + 2].copy_from_slice(&pair);
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            if s == eob {
                                flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                syms.push(s);
                                return (Ok(()), r);
                            }
                            batch[bk & 15] = s as u8;
                            if s >= 256 {
                                if np < PATCH_CAP {
                                    patches[np] = (ck + bk) as u16;
                                    np += 1;
                                } else {
                                    flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                    syms.push(s);
                                    np = 0;
                                    done += bk + 1;
                                    ck = 0;
                                    break 'batch;
                                }
                            }
                            bk += 1;
                        }
                    }
                    if PIPE {
                        let e = t.lut[r.peek_live(B as u32) as usize];
                        r.refill(data);
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_u8(e);
                            let idx = bk & 15;
                            let pair = ((e >> 8) as u16).to_le_bytes();
                            batch[idx..idx + 2].copy_from_slice(&pair);
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            if s == eob {
                                flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                syms.push(s);
                                return (Ok(()), r);
                            }
                            batch[bk & 15] = s as u8;
                            if s >= 256 {
                                if np < PATCH_CAP {
                                    patches[np] = (ck + bk) as u16;
                                    np += 1;
                                } else {
                                    flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                    syms.push(s);
                                    np = 0;
                                    done += bk + 1;
                                    ck = 0;
                                    break 'batch;
                                }
                            }
                            bk += 1;
                        }
                    } else {
                        r.refill(data);
                    }
                    ck += bk;
                    done += bk;
                }
                rem_syms -= done;
                if ck > CHUNK_STOP {
                    flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                    ck = 0;
                    np = 0;
                }
            }
            if rem_syms > 0 {
                while rem_syms >= 2 {
                    if r.live < B as u32 {
                        r.refill(data);
                    }
                    let e = t.lut[r.peek_live(B as u32) as usize];
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        r.skip_short_shl_u8(e);
                        let idx = ck & CHUNK_MASK;
                        let pair = ((e >> 8) as u16).to_le_bytes();
                        chunk[idx..idx + 2].copy_from_slice(&pair);
                        ck += c;
                        rem_syms -= c;
                    } else {
                        let (s, after) = t.decode_entry_slow(e, r, data);
                        r = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => return (Err(err), r),
                        };
                        if s == eob {
                            flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                            syms.push(s);
                            return (Ok(()), r);
                        }
                        chunk[ck & CHUNK_MASK] = s as u8;
                        if s >= 256 {
                            if np < PATCH_CAP {
                                patches[np] = ck as u16;
                                np += 1;
                                ck += 1;
                            } else {
                                flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                                syms.push(s);
                                ck = 0;
                                np = 0;
                            }
                        } else {
                            ck += 1;
                        }
                        rem_syms -= 1;
                    }
                    if ck > CHUNK_STOP {
                        flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                        ck = 0;
                        np = 0;
                    }
                }
                if rem_syms == 1 {
                    let (s, after) = t.decode_one_val(r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => return (Err(err), r),
                    };
                    if s == eob {
                        flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                        syms.push(s);
                        return (Ok(()), r);
                    }
                    chunk[ck & CHUNK_MASK] = s as u8;
                    if s >= 256 {
                        if np < PATCH_CAP {
                            patches[np] = ck as u16;
                            np += 1;
                            ck += 1;
                        } else {
                            flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                            syms.push(s);
                            ck = 0;
                            np = 0;
                        }
                    } else {
                        ck += 1;
                    }
                }
                if ck > CHUNK_STOP {
                    flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                    ck = 0;
                    np = 0;
                }
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

#[inline(always)]
fn run_c2_inner(
    tables: &[HuffK2<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k2_u8_inner::<10, 1024, false>(tables, sel_buf, data, chunk, syms, r)
}

#[inline(always)]
fn run_c3_inner(
    tables: &[HuffK2<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k2_u8_inner::<10, 1024, true>(tables, sel_buf, data, chunk, syms, r)
}

#[inline(always)]
fn run_c8_b11_inner(
    tables: &[HuffK2<11, 2048>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k2_u8_inner::<11, 2048, true>(tables, sel_buf, data, chunk, syms, r)
}

dispatch_bmi2_reg!(
    run_c2_u8chunk_s5,
    run_c2_u8chunk_s5_bmi2,
    run_c2_inner,
    (tables: &[HuffK2<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

dispatch_bmi2_reg!(
    run_c3_pipe4_u8,
    run_c3_pipe4_u8_bmi2,
    run_c3_inner,
    (tables: &[HuffK2<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

dispatch_bmi2_reg!(
    run_c8_b11_pipe4,
    run_c8_b11_pipe4_bmi2,
    run_c8_b11_inner,
    (tables: &[HuffK2<11, 2048>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

// ---- C6/C7/C9/C10/C12: High-2-bit cnt K=3 single-u32-store + sym256 patch list, ZERO spills! ----

#[inline(always)]
fn run_k3hi_u32_inner<const B: usize, const L: usize, const PRE_STEPS: usize, const PIPE: bool>(
    tables: &[HuffK3Hi<B, L>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    mut r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut ck = 0usize;
    let mut pbuf = PatchBuf::new();
    let max_batch_syms = 3 * (PRE_STEPS + usize::from(PIPE));
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let t = &tables[g as usize];
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
            let mut rem_syms = take * GROUP;
            let eob = t.eob;
            r.refill(data);
            while rem_syms >= max_batch_syms {
                let stop_ck = (ck + (rem_syms - max_batch_syms)).min(CHUNK_STOP);
                let before_ck = ck;
                let mut flushed_early = 0usize;
                'batch: while ck <= stop_ck {
                    let batch: &mut [u8; 20] = (&mut chunk[ck..ck + 20]).try_into().unwrap();
                    let mut bk = 0usize;
                    for _ in 0..PRE_STEPS {
                        let e = t.lut[r.peek_live(B as u32) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_low4(e);
                            let idx = bk & 15;
                            let word = (e >> K3H_SYM1).to_le_bytes();
                            batch[idx..idx + 4].copy_from_slice(&word);
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            if s == eob {
                                pbuf.flush(&chunk[..ck + bk], syms);
                                syms.push(s);
                                return (Ok(()), r);
                            }
                            batch[bk & 15] = s as u8;
                            if s >= 256 && !pbuf.push_if_room(ck + bk) {
                                pbuf.flush(&chunk[..ck + bk], syms);
                                syms.push(256);
                                flushed_early = (ck + bk + 1) - before_ck;
                                ck = 0;
                                break 'batch;
                            }
                            bk += 1;
                        }
                    }
                    if PIPE {
                        let e = t.lut[r.peek_live(B as u32) as usize];
                        r.refill(data);
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_low4(e);
                            let idx = bk & 15;
                            let word = (e >> K3H_SYM1).to_le_bytes();
                            batch[idx..idx + 4].copy_from_slice(&word);
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            if s == eob {
                                pbuf.flush(&chunk[..ck + bk], syms);
                                syms.push(s);
                                return (Ok(()), r);
                            }
                            batch[bk & 15] = s as u8;
                            if s >= 256 && !pbuf.push_if_room(ck + bk) {
                                pbuf.flush(&chunk[..ck + bk], syms);
                                syms.push(256);
                                flushed_early = (ck + bk + 1) - before_ck;
                                ck = 0;
                                break 'batch;
                            }
                            bk += 1;
                        }
                    } else {
                        r.refill(data);
                    }
                    ck += bk;
                }
                rem_syms -= if flushed_early != 0 {
                    flushed_early
                } else {
                    ck - before_ck
                };
                if ck > CHUNK_STOP {
                    pbuf.flush(&chunk[..ck], syms);
                    ck = 0;
                }
            }
            while rem_syms >= 3 {
                if r.live < B as u32 {
                    r.refill(data);
                }
                let e = t.lut[r.peek_live(B as u32) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_low4(e);
                    let idx = ck & CHUNK_MASK;
                    let word = (e >> K3H_SYM1).to_le_bytes();
                    chunk[idx..idx + 4].copy_from_slice(&word);
                    ck += c;
                    rem_syms -= c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => return (Err(err), r),
                    };
                    if s == eob {
                        pbuf.flush(&chunk[..ck], syms);
                        syms.push(s);
                        return (Ok(()), r);
                    }
                    chunk[ck & CHUNK_MASK] = s as u8;
                    if s >= 256 && !pbuf.push_if_room(ck) {
                        pbuf.flush(&chunk[..ck], syms);
                        syms.push(256);
                        ck = 0;
                    } else {
                        ck += 1;
                    }
                    rem_syms -= 1;
                }
                if ck > CHUNK_STOP {
                    pbuf.flush(&chunk[..ck], syms);
                    ck = 0;
                }
            }
            while rem_syms > 0 {
                let (s, after) = t.decode_one_val(r, data);
                r = after;
                let s = match s {
                    Ok(s) => s,
                    Err(err) => return (Err(err), r),
                };
                if s == eob {
                    pbuf.flush(&chunk[..ck], syms);
                    syms.push(s);
                    return (Ok(()), r);
                }
                chunk[ck & CHUNK_MASK] = s as u8;
                if s >= 256 && !pbuf.push_if_room(ck) {
                    pbuf.flush(&chunk[..ck], syms);
                    syms.push(256);
                    ck = 0;
                } else {
                    ck += 1;
                }
                rem_syms -= 1;
            }
            if ck > CHUNK_STOP {
                pbuf.flush(&chunk[..ck], syms);
                ck = 0;
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

#[inline(always)]
fn run_c6_inner(
    tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k3hi_u32_inner::<10, 1024, 3, true>(tables, sel_buf, data, chunk, syms, r)
}

#[inline(always)]
fn run_c7_inner(
    tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k3hi_u32_inner::<10, 1024, 5, false>(tables, sel_buf, data, chunk, syms, r)
}

#[inline(always)]
fn run_c9_b9k3_inner(
    tables: &[HuffK3Hi<9, 512>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    run_k3hi_u32_inner::<9, 512, 4, true>(tables, sel_buf, data, chunk, syms, r)
}

dispatch_bmi2_reg!(
    run_c6_k3hi_pipe4,
    run_c6_k3hi_pipe4_bmi2,
    run_c6_inner,
    (tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

dispatch_bmi2_reg!(
    run_c7_k3hi_s5,
    run_c7_k3hi_s5_bmi2,
    run_c7_inner,
    (tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

dispatch_bmi2_reg!(
    run_c9_b9k3_pipe5,
    run_c9_b9k3_pipe5_bmi2,
    run_c9_b9k3_inner,
    (tables: &[HuffK3Hi<9, 512>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

// ---- C10_k3hi_split_pipe4: Split 24 KiB contiguous 64B-aligned hot LUTs + K3Hi pipe4 ----

#[inline(always)]
fn run_c10_split_k3_inner(
    hot: &SplitHotLuts,
    tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS],
    sel_buf: &[u8],
    data: &[u8],
    chunk: &mut [u8; CHUNK_LEN],
    syms: &mut Vec<u16>,
    mut r: Reg,
) -> (Result<(), ErrorKind>, Reg) {
    let data_len = data.len();
    let sel_cap = sel_buf.len();
    let mut sel = 0usize;
    let mut ck = 0usize;
    let mut patches = [0u16; PATCH_CAP];
    let mut np = 0usize;
    while sel < sel_cap {
        let g = sel_buf[sel];
        let mut rem = 1usize;
        while sel + rem < sel_cap && sel_buf[sel + rem] == g {
            rem += 1;
        }
        sel += rem;
        let lut = &hot.luts[g as usize];
        let t = &tables[g as usize];
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
            let mut rem_syms = take * GROUP;
            let eob = t.eob;
            r.refill(data);
            while rem_syms >= 12 {
                let stop_ck = (ck + (rem_syms - 12)).min(CHUNK_STOP);
                let mut done = 0usize;
                'batch: while ck <= stop_ck {
                    let batch: &mut [u8; 20] = (&mut chunk[ck..ck + 20]).try_into().unwrap();
                    let mut bk = 0usize;
                    for _ in 0..3 {
                        let e = lut[r.peek_live(10) as usize];
                        let c = (e >> CNT_SH) as usize;
                        if c != 0 {
                            r.skip_short_shl_low4(e);
                            let idx = bk & 15;
                            let word = (e >> K3H_SYM1).to_le_bytes();
                            batch[idx..idx + 4].copy_from_slice(&word);
                            bk += c;
                        } else {
                            let (s, after) = t.decode_entry_slow(e, r, data);
                            r = after;
                            let s = match s {
                                Ok(s) => s,
                                Err(err) => return (Err(err), r),
                            };
                            if s == eob {
                                flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                syms.push(s);
                                return (Ok(()), r);
                            }
                            batch[bk & 15] = s as u8;
                            if s >= 256 {
                                if np < PATCH_CAP {
                                    patches[np] = (ck + bk) as u16;
                                    np += 1;
                                } else {
                                    flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                    syms.push(s);
                                    np = 0;
                                    done += bk + 1;
                                    ck = 0;
                                    break 'batch;
                                }
                            }
                            bk += 1;
                        }
                    }
                    let e = lut[r.peek_live(10) as usize];
                    r.refill(data);
                    let c = (e >> CNT_SH) as usize;
                    if c != 0 {
                        r.skip_short_shl_low4(e);
                        let idx = bk & 15;
                        let word = (e >> K3H_SYM1).to_le_bytes();
                        batch[idx..idx + 4].copy_from_slice(&word);
                        bk += c;
                    } else {
                        let (s, after) = t.decode_entry_slow(e, r, data);
                        r = after;
                        let s = match s {
                            Ok(s) => s,
                            Err(err) => return (Err(err), r),
                        };
                        if s == eob {
                            flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                            syms.push(s);
                            return (Ok(()), r);
                        }
                        batch[bk & 15] = s as u8;
                        if s >= 256 {
                            if np < PATCH_CAP {
                                patches[np] = (ck + bk) as u16;
                                np += 1;
                            } else {
                                flush_u8_patched(&chunk[..ck + bk], &patches[..np], syms);
                                syms.push(s);
                                np = 0;
                                done += bk + 1;
                                ck = 0;
                                break 'batch;
                            }
                        }
                        bk += 1;
                    }
                    ck += bk;
                    done += bk;
                }
                rem_syms -= done;
                if ck > CHUNK_STOP {
                    flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                    ck = 0;
                    np = 0;
                }
            }
            while rem_syms >= 3 {
                if r.live < 10 {
                    r.refill(data);
                }
                let e = lut[r.peek_live(10) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_low4(e);
                    let idx = ck & CHUNK_MASK;
                    let word = (e >> K3H_SYM1).to_le_bytes();
                    chunk[idx..idx + 4].copy_from_slice(&word);
                    ck += c;
                    rem_syms -= c;
                } else {
                    let (s, after) = t.decode_entry_slow(e, r, data);
                    r = after;
                    let s = match s {
                        Ok(s) => s,
                        Err(err) => return (Err(err), r),
                    };
                    if s == eob {
                        flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                        syms.push(s);
                        return (Ok(()), r);
                    }
                    chunk[ck & CHUNK_MASK] = s as u8;
                    if s >= 256 {
                        if np < PATCH_CAP {
                            patches[np] = ck as u16;
                            np += 1;
                            ck += 1;
                        } else {
                            flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                            syms.push(s);
                            ck = 0;
                            np = 0;
                        }
                    } else {
                        ck += 1;
                    }
                    rem_syms -= 1;
                }
                if ck > CHUNK_STOP {
                    flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                    ck = 0;
                    np = 0;
                }
            }
            while rem_syms > 0 {
                let (s, after) = t.decode_one_val(r, data);
                r = after;
                let s = match s {
                    Ok(s) => s,
                    Err(err) => return (Err(err), r),
                };
                if s == eob {
                    flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                    syms.push(s);
                    return (Ok(()), r);
                }
                chunk[ck & CHUNK_MASK] = s as u8;
                if s >= 256 {
                    if np < PATCH_CAP {
                        patches[np] = ck as u16;
                        np += 1;
                        ck += 1;
                    } else {
                        flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                        syms.push(s);
                        ck = 0;
                        np = 0;
                    }
                } else {
                    ck += 1;
                }
                rem_syms -= 1;
            }
            if ck > CHUNK_STOP {
                flush_u8_patched(&chunk[..ck], &patches[..np], syms);
                ck = 0;
                np = 0;
            }
            if check_overran && r.pos > data_len && r.overran(data) {
                return (Err(ErrorKind::Truncated), r);
            }
            rem -= take;
        }
    }
    (Err(ErrorKind::SelectorOverrun), r)
}

dispatch_bmi2_reg!(
    run_c10_split_k3_pipe4,
    run_c10_split_k3_pipe4_bmi2,
    run_c10_split_k3_inner,
    (hot: &SplitHotLuts, tables: &[HuffK3Hi<10, 1024>; MAX_GROUPS], sel_buf: &[u8], data: &[u8], chunk: &mut [u8; CHUNK_LEN], syms: &mut Vec<u16>) -> Result<(), ErrorKind>
);

// ---- Timing runner ----

#[derive(Clone, Copy)]
enum Arm {
    R0Shipped,
    D1K2Nostore,
    D2K3Nostore,
    C1Pipe4U16,
    C2U8ChunkS5,
    C3Pipe4U8,
    C6K3HiPipe4,
    C7K3HiS5,
    C8B11Pipe4,
    C9B9K3Pipe5,
    C10SplitK3Pipe4,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::R0Shipped => "R0_shipped",
            Arm::D1K2Nostore => "D1_k2_nostore",
            Arm::D2K3Nostore => "D2_k3_nostore",
            Arm::C1Pipe4U16 => "C1_pipe4_u16",
            Arm::C2U8ChunkS5 => "C2_u8chunk_s5",
            Arm::C3Pipe4U8 => "C3_pipe4_u8",
            Arm::C6K3HiPipe4 => "C6_k3hi_pipe4",
            Arm::C7K3HiS5 => "C7_k3hi_s5",
            Arm::C8B11Pipe4 => "C8_b11_pipe4",
            Arm::C9B9K3Pipe5 => "C9_b9k3_pipe5",
            Arm::C10SplitK3Pipe4 => "C10_split_k3_p4",
        }
    }

    fn scratch_bytes(self) -> usize {
        let sel = MAX_SELECTORS;
        match self {
            Arm::R0Shipped | Arm::D1K2Nostore | Arm::C1Pipe4U16 => {
                sel + std::mem::size_of::<[HuffK2<10, 1024>; MAX_GROUPS]>() + CHUNK_LEN * 2
            }
            Arm::C2U8ChunkS5 | Arm::C3Pipe4U8 => {
                sel + std::mem::size_of::<[HuffK2<10, 1024>; MAX_GROUPS]>() + CHUNK_LEN
            }
            Arm::C8B11Pipe4 => {
                sel + std::mem::size_of::<[HuffK2<11, 2048>; MAX_GROUPS]>() + CHUNK_LEN
            }
            Arm::D2K3Nostore | Arm::C6K3HiPipe4 | Arm::C7K3HiS5 | Arm::C10SplitK3Pipe4 => {
                // Note: in production SplitHotLuts replaces the inline `lut` field (same total size!).
                sel + std::mem::size_of::<[HuffK3Hi<10, 1024>; MAX_GROUPS]>() + CHUNK_LEN
            }
            Arm::C9B9K3Pipe5 => {
                sel + std::mem::size_of::<[HuffK3Hi<9, 512>; MAX_GROUPS]>() + CHUNK_LEN
            }
        }
    }
}

struct ScratchAll {
    t10: Box<[HuffK2<10, 1024>; MAX_GROUPS]>,
    t11: Box<[HuffK2<11, 2048>; MAX_GROUPS]>,
    tk3_10: Box<[HuffK3Hi<10, 1024>; MAX_GROUPS]>,
    tk3_9: Box<[HuffK3Hi<9, 512>; MAX_GROUPS]>,
    hot: Box<SplitHotLuts>,
    chunk16: Box<[u16; CHUNK_LEN]>,
    chunk8: Box<[u8; CHUNK_LEN]>,
    syms: Vec<u16>,
}

impl ScratchAll {
    fn new() -> Self {
        Self {
            t10: Box::new(std::array::from_fn(|_| HuffK2::default())),
            t11: Box::new(std::array::from_fn(|_| HuffK2::default())),
            tk3_10: Box::new(std::array::from_fn(|_| HuffK3Hi::default())),
            tk3_9: Box::new(std::array::from_fn(|_| HuffK3Hi::default())),
            hot: Box::new(SplitHotLuts::default()),
            chunk16: Box::new([0u16; CHUNK_LEN]),
            chunk8: Box::new([0u8; CHUNK_LEN]),
            syms: Vec::with_capacity(MAX_SELECTORS * GROUP),
        }
    }
}

fn run_arm_corpus(
    arm: Arm,
    corpus: &CorpusData,
    sc: &mut ScratchAll,
    verify: bool,
) -> (u64, u64, u64, u64) {
    let mut build_cpu = 0u64;
    let mut dec_cpu = 0u64;
    let w0 = Instant::now();
    let c0 = thread_cpu_ns();
    let mut chk = 0u64;

    for (bi, blk) in corpus.blocks.iter().enumerate() {
        let bp = blk.selectors.len() >= PAIR_GATE_SEL;
        let bp_k3 = blk.selectors.len() >= K3_GATE_SEL;
        let b_start = thread_cpu_ns();
        match arm {
            Arm::R0Shipped
            | Arm::D1K2Nostore
            | Arm::C1Pipe4U16
            | Arm::C2U8ChunkS5
            | Arm::C3Pipe4U8 => {
                for g in 0..blk.n_groups {
                    if (blk.used_groups >> g) & 1 != 0 {
                        sc.t10[g].build_into(&blk.lens[g][..blk.alpha], bp);
                    }
                }
            }
            Arm::C8B11Pipe4 => {
                for g in 0..blk.n_groups {
                    if (blk.used_groups >> g) & 1 != 0 {
                        sc.t11[g].build_into(&blk.lens[g][..blk.alpha], bp);
                    }
                }
            }
            Arm::D2K3Nostore | Arm::C6K3HiPipe4 | Arm::C7K3HiS5 => {
                for g in 0..blk.n_groups {
                    if (blk.used_groups >> g) & 1 != 0 {
                        sc.tk3_10[g].build_into_fast(&blk.lens[g][..blk.alpha], bp, bp_k3);
                    }
                }
            }
            Arm::C10SplitK3Pipe4 => {
                for g in 0..blk.n_groups {
                    if (blk.used_groups >> g) & 1 != 0 {
                        sc.tk3_10[g].build_into_fast(&blk.lens[g][..blk.alpha], bp, bp_k3);
                        sc.hot.luts[g] = sc.tk3_10[g].lut;
                    }
                }
            }
            Arm::C9B9K3Pipe5 => {
                for g in 0..blk.n_groups {
                    if (blk.used_groups >> g) & 1 != 0 {
                        sc.tk3_9[g].build_into_fast(&blk.lens[g][..blk.alpha], bp, bp_k3);
                    }
                }
            }
        }
        let d_start = thread_cpu_ns();
        build_cpu += d_start.saturating_sub(b_start);

        sc.syms.clear();
        let r0 = Reg::at(&corpus.raw_bz2, blk.start_bit);
        let (res, r_end) = match arm {
            Arm::R0Shipped => run_shipped(
                &sc.t10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk16,
                &mut sc.syms,
                r0,
            ),
            Arm::D1K2Nostore => {
                let (a, re) = run_d1_k2_nostore(&sc.t10, &blk.selectors, &corpus.raw_bz2, r0);
                chk ^= a;
                (Ok(()), re)
            }
            Arm::D2K3Nostore => {
                let (a, re) = run_d2_k3_nostore(&sc.tk3_10, &blk.selectors, &corpus.raw_bz2, r0);
                chk ^= a;
                (Ok(()), re)
            }
            Arm::C1Pipe4U16 => run_c1_pipe4_u16(
                &sc.t10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk16,
                &mut sc.syms,
                r0,
            ),
            Arm::C2U8ChunkS5 => run_c2_u8chunk_s5(
                &sc.t10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C3Pipe4U8 => run_c3_pipe4_u8(
                &sc.t10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C6K3HiPipe4 => run_c6_k3hi_pipe4(
                &sc.tk3_10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C7K3HiS5 => run_c7_k3hi_s5(
                &sc.tk3_10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C8B11Pipe4 => run_c8_b11_pipe4(
                &sc.t11,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C9B9K3Pipe5 => run_c9_b9k3_pipe5(
                &sc.tk3_9,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
            Arm::C10SplitK3Pipe4 => run_c10_split_k3_pipe4(
                &sc.hot,
                &sc.tk3_10,
                &blk.selectors,
                &corpus.raw_bz2,
                &mut sc.chunk8,
                &mut sc.syms,
                r0,
            ),
        };
        let d_end = thread_cpu_ns();
        dec_cpu += d_end.saturating_sub(d_start);

        if verify && !matches!(arm, Arm::D1K2Nostore | Arm::D2K3Nostore) {
            assert!(
                res.is_ok(),
                "{} failed on {} blk {bi}: {res:?}",
                arm.name(),
                corpus.name
            );
            assert_eq!(
                r_end.bit_pos(),
                blk.end_bit,
                "{} bit_pos mismatch on {} blk {bi}: got {} expected {}",
                arm.name(),
                corpus.name,
                r_end.bit_pos(),
                blk.end_bit
            );
            assert_eq!(
                sc.syms.len(),
                blk.expected_syms.len(),
                "{} len mismatch on {} blk {bi}",
                arm.name(),
                corpus.name
            );
            assert_eq!(
                sc.syms,
                blk.expected_syms,
                "{} syms mismatch on {} blk {bi}",
                arm.name(),
                corpus.name
            );
        }
        chk = chk.wrapping_add(sc.syms.len() as u64 ^ r_end.bit_pos());
    }

    let c_tot = thread_cpu_ns().saturating_sub(c0);
    let w_tot = w0.elapsed().as_nanos() as u64;
    black_box(chk);
    (c_tot, w_tot, build_cpu, dec_cpu)
}

fn verify_synthetic_alpha258(sc: &mut ScratchAll) {
    let mut raw = Vec::with_capacity(40_000);
    for _ in 0..150 {
        for b in 0..=255u8 {
            raw.push(b);
        }
    }
    let bz2 = common::compress(&raw, 9);
    let corpus = parse_corpus("synth258", bz2);
    assert!(
        corpus.blocks.iter().any(|b| b.alpha == 258),
        "expected alpha == 258 block"
    );
    let verify_arms = [
        Arm::R0Shipped,
        Arm::C1Pipe4U16,
        Arm::C2U8ChunkS5,
        Arm::C3Pipe4U8,
        Arm::C6K3HiPipe4,
        Arm::C7K3HiS5,
        Arm::C8B11Pipe4,
        Arm::C9B9K3Pipe5,
        Arm::C10SplitK3Pipe4,
    ];
    for arm in verify_arms {
        run_arm_corpus(arm, &corpus, sc, true);
    }
    println!(
        "EXACTNESS synth_alpha258 ({} blocks, {} syms): ALL {} ARMS PASS",
        corpus.blocks.len(),
        corpus.total_syms,
        verify_arms.len()
    );
}

fn bench_corpus(corpus: &CorpusData, arms: &[Arm], repeats: usize, sc: &mut ScratchAll) {
    // Repeat tiny corpora inside each sample so each measurement spans >= ~2 ms (zero jitter).
    let inner_iters = (500_000 / corpus.total_syms.max(1)).clamp(1, 1000);
    println!(
        "\n=== BENCH {} ({} blocks, {} syms, {} repeats x{} inner) ===",
        corpus.name,
        corpus.blocks.len(),
        corpus.total_syms,
        repeats,
        inner_iters
    );
    for &arm in arms {
        run_arm_corpus(arm, corpus, sc, false);
    }

    let n_arms = arms.len();
    let mut cpu_ms = vec![Vec::with_capacity(repeats); n_arms];
    let mut wall_ms = vec![Vec::with_capacity(repeats); n_arms];
    let mut bld_ms = vec![Vec::with_capacity(repeats); n_arms];
    let mut dec_ms = vec![Vec::with_capacity(repeats); n_arms];

    for r in 0..repeats {
        for off in 0..n_arms {
            let idx = (r + off) % n_arms;
            let mut c_sum = 0u64;
            let mut w_sum = 0u64;
            let mut b_sum = 0u64;
            let mut d_sum = 0u64;
            for _ in 0..inner_iters {
                let (c, w, b, d) = run_arm_corpus(arms[idx], corpus, sc, false);
                c_sum += c;
                w_sum += w;
                b_sum += b;
                d_sum += d;
            }
            let scale = inner_iters as f64 * 1e6;
            cpu_ms[idx].push(c_sum as f64 / scale);
            wall_ms[idx].push(w_sum as f64 / scale);
            bld_ms[idx].push(b_sum as f64 / scale);
            dec_ms[idx].push(d_sum as f64 / scale);
        }
    }

    let stats = |v: &[f64]| -> (f64, f64, f64) {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[0], s[s.len() / 2], s[s.len() - 1])
    };

    let (ref_min, ref_med, ref_max) = stats(&cpu_ms[0]);
    let (_, ref_dec_med, _) = stats(&dec_ms[0]);
    println!(
        "{:<18} {:>24} {:>8} {:>5} {:>7} {:>8} {:>8} {:>8} {:>7}",
        "arm",
        "CPU min/med/max (ms)",
        "vs ref",
        "ovlp",
        "ns/sym",
        "bld_med",
        "dec_med",
        "dec_vs",
        "mem_KiB"
    );
    for (i, &arm) in arms.iter().enumerate() {
        let (mn, md, mx) = stats(&cpu_ms[i]);
        let (_, bm, _) = stats(&bld_ms[i]);
        let (_, dm, _) = stats(&dec_ms[i]);
        let (_, wm, _) = stats(&wall_ms[i]);
        let pct = 100.0 * (md / ref_med - 1.0);
        let dpct = 100.0 * (dm / ref_dec_med - 1.0);
        let ovlp = if i == 0 {
            "-"
        } else if mx < ref_min || mn > ref_max {
            "NO"
        } else {
            "YES"
        };
        println!(
            "{:<18} {:>7.3} / {:>6.3} / {:>6.3} {:>+7.2}% {:>5} {:>7.3} {:>8.3} {:>8.3} {:>+7.2}% {:>6.1} (wall {:.3})",
            arm.name(),
            mn,
            md,
            mx,
            pct,
            ovlp,
            md * 1e6 / corpus.total_syms as f64,
            bm,
            dm,
            dpct,
            arm.scratch_bytes() as f64 / 1024.0,
            wm
        );
    }
}

fn print_census(corpora: &[&CorpusData]) {
    println!("=== SECTION 0: 6-CORPUS EXACT CENSUS ===");
    println!(
        "{:<10} {:>4} {:>10} {:>7} {:>8} {:>9} {:>9} {:>9} {:>7}",
        "corpus", "blks", "syms", "sel<1k%", "sym<1k%", "K2_lkp/s", "K3_lkp/s", "K3_vs_K2", "c3%"
    );
    let mut t10: Box<[HuffK2<10, 1024>; MAX_GROUPS]> =
        Box::new(std::array::from_fn(|_| HuffK2::default()));
    let mut tk3: Box<[HuffK3Hi<10, 1024>; MAX_GROUPS]> =
        Box::new(std::array::from_fn(|_| HuffK3Hi::default()));
    for &c in corpora {
        let mut blks_lt = 0usize;
        let mut syms_lt = 0usize;
        let mut k2_lkps = 0u64;
        let mut k3_lkps = 0u64;
        let mut k3_c3 = 0u64;
        for blk in &c.blocks {
            if blk.selectors.len() < K3_GATE_SEL {
                blks_lt += 1;
                syms_lt += blk.expected_syms.len();
            }
            let bp = blk.selectors.len() >= PAIR_GATE_SEL;
            let bp_k3 = blk.selectors.len() >= K3_GATE_SEL;
            for g in 0..blk.n_groups {
                if (blk.used_groups >> g) & 1 != 0 {
                    t10[g].build_into(&blk.lens[g][..blk.alpha], bp);
                    tk3[g].build_into_fast(&blk.lens[g][..blk.alpha], bp, bp_k3);
                }
            }
            // Count exact lookups on K2 and K3 (with K3_GATE_SEL applied)
            let mut r2 = Reg::at(&c.raw_bz2, blk.start_bit);
            let mut r3 = Reg::at(&c.raw_bz2, blk.start_bit);
            let mut done2 = false;
            let mut done3 = false;
            let mut sel = 0usize;
            while sel < blk.selectors.len() && (!done2 || !done3) {
                let g = blk.selectors[sel] as usize;
                let mut rem = 1usize;
                while sel + rem < blk.selectors.len() && blk.selectors[sel + rem] == g as u8 {
                    rem += 1;
                }
                sel += rem;
                if !done2 {
                    let t = &t10[g];
                    let mut left = rem * GROUP;
                    while left > 0 {
                        let e = t.lut[r2.peek(&c.raw_bz2, 10) as usize];
                        let cnt = (e >> CNT_SH) as usize;
                        if cnt != 0 && cnt <= left {
                            r2.skip_short_shl_u8(e);
                            k2_lkps += 1;
                            left -= cnt;
                        } else if cnt != 0 {
                            let (s, nr) = t.decode_one_val(r2, &c.raw_bz2);
                            r2 = nr;
                            k2_lkps += 1;
                            left -= 1;
                            if s.unwrap() == t.eob {
                                done2 = true;
                                break;
                            }
                        } else {
                            let (s, nr) = t.decode_entry_slow(e, r2, &c.raw_bz2);
                            r2 = nr;
                            k2_lkps += 1;
                            left -= 1;
                            if s.unwrap() == t.eob {
                                done2 = true;
                                break;
                            }
                        }
                    }
                }
                if !done3 {
                    let t = &tk3[g];
                    let mut left = rem * GROUP;
                    while left > 0 {
                        let e = t.lut[r3.peek(&c.raw_bz2, 10) as usize];
                        let cnt = (e >> CNT_SH) as usize;
                        if cnt != 0 && cnt <= left {
                            r3.skip_short_shl_low4(e);
                            k3_lkps += 1;
                            if cnt == 3 {
                                k3_c3 += 1;
                            }
                            left -= cnt;
                        } else if cnt != 0 {
                            let (s, nr) = t.decode_one_val(r3, &c.raw_bz2);
                            r3 = nr;
                            k3_lkps += 1;
                            left -= 1;
                            if s.unwrap() == t.eob {
                                done3 = true;
                                break;
                            }
                        } else {
                            let (s, nr) = t.decode_entry_slow(e, r3, &c.raw_bz2);
                            r3 = nr;
                            k3_lkps += 1;
                            left -= 1;
                            if s.unwrap() == t.eob {
                                done3 = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        let k2_r = k2_lkps as f64 / c.total_syms as f64;
        let k3_r = k3_lkps as f64 / c.total_syms as f64;
        println!(
            "{:<10} {:>4} {:>10} {:>6.1}% {:>7.1}% {:>9.4} {:>9.4} {:>+8.2}% {:>6.1}%",
            c.name,
            c.blocks.len(),
            c.total_syms,
            100.0 * blks_lt as f64 / c.blocks.len() as f64,
            100.0 * syms_lt as f64 / c.total_syms as f64,
            k2_r,
            k3_r,
            100.0 * (k3_r / k2_r - 1.0),
            100.0 * k3_c3 as f64 / k3_lkps.max(1) as f64
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wiki_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("/home/antigravity/wt-a6b-probe/corpora/wikidata.bz2");
    let repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);

    let wiki_bytes = std::fs::read(wiki_path).expect("read wikidata.bz2");
    let wikidata = parse_corpus("wikidata", wiki_bytes);

    let raw_guards = common::corpora(2);
    let mut guards = Vec::new();
    for (name, raw) in raw_guards {
        let short: &'static str = match name {
            "text" => "text",
            "records" => "records",
            "random" => "random",
            "runs" => "runs",
            "one byte" => "onebyte",
            _ => continue,
        };
        let bz2 = common::compress(&raw, 9);
        guards.push(parse_corpus(short, bz2));
    }

    let mut all_corpora: Vec<&CorpusData> = vec![&wikidata];
    for g in &guards {
        all_corpora.push(g);
    }
    print_census(&all_corpora);

    let mut sc = ScratchAll::new();
    println!("\n=== SECTION 1: 100% EXACTNESS VERIFICATION ===");
    verify_synthetic_alpha258(&mut sc);
    let verify_arms = [
        Arm::R0Shipped,
        Arm::C1Pipe4U16,
        Arm::C2U8ChunkS5,
        Arm::C3Pipe4U8,
        Arm::C6K3HiPipe4,
        Arm::C7K3HiS5,
        Arm::C8B11Pipe4,
        Arm::C9B9K3Pipe5,
        Arm::C10SplitK3Pipe4,
    ];
    for &arm in &verify_arms {
        run_arm_corpus(arm, &wikidata, &mut sc, true);
        for g in &guards {
            run_arm_corpus(arm, g, &mut sc, true);
        }
    }
    println!(
        "ALL {} ARMS 100% BIT- AND SYMBOL-EXACT ON wikidata ({} blks) + {} GUARD CORPORA!",
        verify_arms.len(),
        wikidata.blocks.len(),
        guards.len()
    );

    let bench_arms = [
        Arm::R0Shipped,
        Arm::D1K2Nostore,
        Arm::D2K3Nostore,
        Arm::C1Pipe4U16,
        Arm::C2U8ChunkS5,
        Arm::C3Pipe4U8,
        Arm::C6K3HiPipe4,
        Arm::C7K3HiS5,
        Arm::C8B11Pipe4,
        Arm::C9B9K3Pipe5,
        Arm::C10SplitK3Pipe4,
    ];
    bench_corpus(&wikidata, &bench_arms, repeats, &mut sc);
    for g in &guards {
        bench_corpus(g, &bench_arms, 7, &mut sc);
    }
}
