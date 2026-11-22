#![forbid(unsafe_code)]
//! Carbon Cycle 38 — Three Targeted Front-End Candidates (`#![forbid(unsafe_code)]`):
//! - `C38a`: Word-Accelerated Block Header, `8x` Batched Selector MTF, & `lzcnt` Zero-Delta Code-Length Run-Skipping (`read_item_c38a` / `ScannerC38a` / `decode_parscan_c38a`).
//! - `C38b`: Bounded-Window (`4 MiB`) Streaming Parallel Front End over `R: std::io::Read` (`decode_stream_parscan_c38b`) with persistent worker pool.
//! - `C38c`: Safe-Rust Two-Level `(lo, hi)` + Seward `SlidingTail` Column `mtf_safe_c38c` vs `mtf_scalar` & `mtf_avx2`.

use mibz2::block::{Item, Symbols, BLOCK_MAGIC, END_MAGIC, MAX_SYMS};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::stream::{Event, Source};
use mibz2::{crc, mtf, rle, runs, Error, ErrorKind, Kernels};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex};

const MASK48: u64 = (1u64 << 48) - 1;
const WINDOW: usize = 64;
const WINDOWS: usize = 64;
const RUN_SHARE: usize = 32;

/// Cycle 37 kept winner: LLVM-autovectorized 64-symbol window counts + monotonic threshold short-circuit.
#[inline]
pub fn takes_runs_fast(b: &Symbols) -> bool {
    let syms = &b.syms;
    let len = syms.len();
    if b.max_len >= (1 << 24) || 2 * len > b.max_len {
        return false;
    }
    if len <= WINDOW * WINDOWS {
        let need = len.div_ceil(RUN_SHARE);
        let mut runs_cnt = 0usize;
        let full = len / WINDOW;
        for i in 0..full {
            let w = &syms[i * WINDOW..(i + 1) * WINDOW];
            runs_cnt += w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
            if runs_cnt >= need {
                return true;
            }
            if runs_cnt + (len - (i + 1) * WINDOW) < need {
                return false;
            }
        }
        runs_cnt += syms[full * WINDOW..].iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
        RUN_SHARE * runs_cnt >= len
    } else {
        const NEED: usize = (WINDOW * WINDOWS) / RUN_SHARE;
        let step = len / WINDOWS;
        let mut runs_cnt = 0usize;
        for i in 0..WINDOWS {
            let w = &syms[i * step..i * step + WINDOW];
            runs_cnt += w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
            if runs_cnt >= NEED {
                return true;
            }
            if runs_cnt + (WINDOWS - 1 - i) * WINDOW < NEED {
                return false;
            }
        }
        runs_cnt >= NEED
    }
}

pub struct WorkerScratch<I: InverseBwt = OpenCells> {
    pub runs: runs::Runs,
    pub ll: Vec<u8>,
    pub rle_buf: Vec<u8>,
    pub counts: [u32; 256],
    pub inv: I,
}

impl<I: InverseBwt + Default> Default for WorkerScratch<I> {
    fn default() -> Self {
        Self {
            runs: runs::Runs::default(),
            ll: Vec::new(),
            rle_buf: Vec::new(),
            counts: [0u32; 256],
            inv: I::default(),
        }
    }
}

#[inline]
pub fn decode_block_fast<I: InverseBwt>(
    b: &Symbols,
    k: Kernels,
    s: &mut WorkerScratch<I>,
    out: &mut Vec<u8>,
) -> Result<u32, ErrorKind> {
    if s.inv.reads_runs() && takes_runs_fast(b) {
        let n = runs::front(b, &mut s.runs, k)?;
        if b.orig_ptr as usize >= n {
            return Err(ErrorKind::BadOrigPtr);
        }
        if s.inv.invert_runs(&s.runs, n, b.orig_ptr, &mut s.ll, k) {
            return finish_block(&s.ll, b.crc, k, out);
        }
    }
    let n = mtf::mtf(b, &mut s.ll, &mut s.counts, k)?;
    if b.orig_ptr as usize >= n {
        return Err(ErrorKind::BadOrigPtr);
    }
    let in_place = s.inv.invert_in_place(&mut s.ll, &s.counts, b.orig_ptr);
    if !in_place {
        s.inv.invert(&s.ll, &s.counts, b.orig_ptr, &mut s.rle_buf);
    }
    let inv_slice: &[u8] = if in_place { &s.ll } else { &s.rle_buf };
    finish_block(inv_slice, b.crc, k, out)
}

#[inline]
fn finish_block(inverted: &[u8], stored: u32, k: Kernels, out: &mut Vec<u8>) -> Result<u32, ErrorKind> {
    let start = out.len();
    if let Err(kind) = rle::unrle(inverted, out, k) {
        out.truncate(start);
        return Err(kind);
    }
    let computed = crc::crc32(&out[start..], k);
    if computed != stored {
        out.truncate(start);
        return Err(ErrorKind::BlockCrc { stored, computed });
    }
    Ok(computed)
}

// ============================================================================
// Candidate C38c: `#![forbid(unsafe_code)]` `(lo, hi)` + Seward `SlidingTail` `mtf_safe_c38c`
// ============================================================================

const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;
const MTFL_SIZE: usize = 4096;

struct SlidingTail240 {
    mtfa: [u8; MTFL_SIZE],
    base: [usize; 16], // buckets 1..16 (15 buckets of 16 bytes = 240 tail entries for nn in 16..256)
}

impl SlidingTail240 {
    #[inline]
    fn new(s2u: &[u8; 256]) -> Self {
        let mut s = Self {
            mtfa: [0u8; MTFL_SIZE],
            base: [0usize; 16],
        };
        s.recompact_init(s2u);
        s
    }

    #[inline]
    fn recompact_init(&mut self, s2u: &[u8; 256]) {
        let mut kk = MTFL_SIZE;
        for lno in (1..16).rev() {
            kk -= 16;
            self.base[lno] = kk;
            self.mtfa[kk..kk + 16].copy_from_slice(&s2u[lno * 16..lno * 16 + 16]);
        }
    }

    #[cold]
    #[inline(never)]
    fn recompact(&mut self) {
        let mut flat = [0u8; 240];
        for lno in 1..16 {
            let b = self.base[lno];
            flat[(lno - 1) * 16..lno * 16].copy_from_slice(&self.mtfa[b..b + 16]);
        }
        let mut kk = MTFL_SIZE;
        for lno in (1..16).rev() {
            kk -= 16;
            self.base[lno] = kk;
            self.mtfa[kk..kk + 16].copy_from_slice(&flat[(lno - 1) * 16..lno * 16]);
        }
    }

    /// Moves index `nn >= 16` (`16..256`), pushing `spill15` (former lane 15) into slot 16,
    /// and returning the symbol byte `v` that was at `nn`.
    #[inline(always)]
    fn move_ge16(&mut self, nn: usize, spill15: u8) -> u8 {
        if self.base[1] == 0 {
            self.recompact();
        }
        let lno = nn >> 4; // 1..=15
        let off = nn & 15; // 0..=15
        let b_lno = self.base[lno];
        let v = self.mtfa[b_lno + off];
        self.mtfa.copy_within(b_lno..b_lno + off, b_lno + 1);
        if lno == 1 {
            self.mtfa[b_lno] = spill15;
        } else {
            self.mtfa[b_lno] = self.mtfa[self.base[lno - 1] + 15];
            let mut u = lno - 1;
            while u > 1 {
                let prev_last = self.mtfa[self.base[u - 1] + 15];
                self.base[u] -= 1;
                self.mtfa[self.base[u]] = prev_last;
                u -= 1;
            }
            self.base[1] -= 1;
            self.mtfa[self.base[1]] = spill15;
        }
        v
    }
}

/// `#![forbid(unsafe_code)]` column MTF + RLE2 decoder with `(lo, hi)` GP-register top-16 cache
/// storing `seq_to_unseq` bytes directly + Seward `SlidingTail240` for `nn >= 16`.
pub fn mtf_safe_c38c(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    if ll.len() < max {
        ll.resize(max, 0);
    }
    *counts = [0u32; 256];

    let s2u = &b.seq_to_unseq;
    let mut lo = u64::from_le_bytes([
        s2u[0], s2u[1], s2u[2], s2u[3], s2u[4], s2u[5], s2u[6], s2u[7],
    ]);
    let mut hi = u64::from_le_bytes([
        s2u[8], s2u[9], s2u[10], s2u[11], s2u[12], s2u[13], s2u[14], s2u[15],
    ]);
    let mut tail = SlidingTail240::new(s2u);

    let out: &mut [u8] = &mut ll[..max];
    let mut n = 0usize;
    let (mut run, mut weight) = (0u32, 1u32);

    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                ll.truncate(0);
                return Err(ErrorKind::RunTooLong);
            }
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            let rlen = run as usize;
            if n + rlen > max {
                ll.truncate(0);
                return Err(ErrorKind::BlockTooLong);
            }
            let c = (lo & 0xff) as u8;
            counts[c as usize] += run;
            out[n..n + rlen].fill(c);
            n += rlen;
            run = 0;
            weight = 1;
        }
        if s == b.eob {
            break;
        }
        if n >= max {
            ll.truncate(0);
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = (s - 1) as usize;
        let c = if nn < 8 {
            let sh = (nn * 8) as u32;
            let v = ((lo >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            lo = (lo & !((low_mask << 8) | 0xff)) | ((lo & low_mask) << 8) | (v as u64);
            v
        } else if nn < 16 {
            let sh = ((nn - 8) * 8) as u32;
            let v = ((hi >> sh) & 0xff) as u8;
            let spill7 = lo >> 56;
            let low_mask = (1u64 << sh) - 1;
            hi = (hi & !((low_mask << 8) | 0xff)) | ((hi & low_mask) << 8) | spill7;
            lo = (lo << 8) | (v as u64);
            v
        } else {
            let spill15 = (hi >> 56) as u8;
            let spill7 = lo >> 56;
            let v = tail.move_ge16(nn, spill15);
            hi = (hi << 8) | spill7;
            lo = (lo << 8) | (v as u64);
            v
        };
        counts[c as usize] += 1;
        out[n] = c;
        n += 1;
    }

    ll.truncate(n);
    Ok(n)
}

// ============================================================================
// Candidate C38a: Word-Accelerated Header, `8x` Batched Selector MTF, & `lzcnt` Delta Code-Length
// ============================================================================

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
        let mut r = Reg { pos: (bit / 8) as usize, buf: 0, live: 0 };
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
        if self.live < n { self.refill(data); }
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

struct Huff38 {
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

impl Default for Huff38 {
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

impl Huff38 {
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
                limit20[i] = if hi_i < 0 { -1 } else { (((hi_i + 1) << (20 - i)) - 1) as i32 };
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
                        if w_hi + 1 > max_root_w { max_root_w = w_hi + 1; }
                    }
                    a_hi = 2 * a_hi.max(hi + 1);
                }

                let mut sub_used = 0usize;
                for w in filled_to..max_root_w {
                    let d_w = d_arr[w] as u32;
                    if d_w == 0 { continue; }
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
                        if zn == 0 || lo_c > hi_c { continue; }
                        let sh = zn - LUT_BITS;
                        for p in lo_c..=hi_c {
                            let w = (p >> sh) as usize;
                            let d_w = d_arr[w] as u32;
                            if d_w == 0 { continue; }
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
            if lo_p > hi_p { continue; }
            let sh = (LUT_BITS - zn) as usize;
            let u_lim = end_at_len[sh] >> zn;
            if u_lim == 0 { continue; }
            let mut tpl: Option<(usize, u32)> = None;
            for p in lo_p..=hi_p {
                let off_p = (p as usize) << sh;
                let e0 = self.lut[off_p];
                if (e0 >> CNT_SH) == 0 { continue; }
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
                        if (e2 >> CNT_SH) == 0 || (zn as usize) + l2 > LUT_BITS as usize { continue; }
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
        if r.live < LUT_BITS { r.refill_win(win); }
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
        if (e & BAD) != 0 { return (Err(ErrorKind::BadHuffmanCode), r); }
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
        if (e & BAD) != 0 { return (Err(ErrorKind::BadHuffmanCode), r); }
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

pub struct ScanScratch38 {
    selectors: Box<[u8; MAX_SELECTORS]>,
    tables: Box<[Huff38; MAX_GROUPS]>,
}

impl Default for ScanScratch38 {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            tables: Box::new(std::array::from_fn(|_| Huff38::default())),
        }
    }
}

pub fn read_item_c38a(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    mut syms: Vec<u16>,
    scratch: &mut ScanScratch38,
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

    // 1. `trailing_zeros` active-group `in_use` unpack (skips empty 16-byte groups cleanly)
    let in_use16 = r.bits(data, 16) as u16;
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    let mut rem16 = in_use16.reverse_bits();
    while rem16 != 0 {
        let i = rem16.trailing_zeros() as usize;
        rem16 &= rem16 - 1;
        let mut w = (r.bits(data, 16) as u16).reverse_bits();
        let base_b = (i * 16) as u8;
        while w != 0 {
            let tz = w.trailing_zeros() as u8;
            w &= w - 1;
            seq_to_unseq[n_in_use] = base_b + tz;
            n_in_use += 1;
        }
    }
    if n_in_use == 0 { fail!(ErrorKind::NoSymbols); }
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) { fail!(ErrorKind::BadGroups); }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 { fail!(ErrorKind::BadSelectors); }

    // 2. `8x` Batched Unary Selector MTF (one refill covers 8 * 6 = 48 <= 56 bits with zero per-selector bounds/overrun checks!)
    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut scratch.selectors[..sel_cap];
    let mut used_groups = 0u8;
    let data_len = data.len();
    let mut i = 0usize;

    while i + 8 <= sel_cap && r.pos + 16 <= data_len {
        r.refill(data);
        for k in 0..8 {
            let j = (!r.buf).leading_zeros() as usize;
            if j >= n_groups {
                r.skip_short(n_groups as u32);
                fail!(ErrorKind::BadSelector);
            }
            r.skip_short((j + 1) as u32);
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let low_mask = (1u64 << sh) - 1;
            mtf = (mtf & !((low_mask << 8) | 0xff)) | ((mtf & low_mask) << 8) | (v as u64);
            sel_buf[i + k] = v;
            used_groups |= 1u8 << v;
        }
        i += 8;
    }

    while i < n_sel {
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
        i += 1;
    }

    // 3. `lzcnt` Zero-Delta Run-Skipping + In-Register `10`/`11` Delta Walk for Huffman Code Lengths
    let mut lens = [0u8; MAX_ALPHA];
    for g in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        let mut sym_idx = 0usize;
        while sym_idx < alpha {
            if r.pos + 16 <= data_len && (1..=20).contains(&curr) {
                if r.live < 32 {
                    r.refill(data);
                }
                // Leading zeros in `r.buf` are consecutive `0` termination bits (0 delta -> repeat `curr`!)
                let z = r.buf.leading_zeros() as usize;
                let run0 = z.min(alpha - sym_idx).min(32);
                if run0 > 0 {
                    lens[sym_idx..sym_idx + run0].fill(curr as u8);
                    r.skip_short(run0 as u32);
                    sym_idx += run0;
                    if sym_idx == alpha {
                        break;
                    }
                }
                // Walk in-register `1` + sign pairs while `r.live >= 2` and `curr in 1..=20`
                while r.live >= 2 && (1..=20).contains(&curr) && (r.buf >> 63) != 0 {
                    let sign = ((r.buf >> 62) & 1) as i32; // 1 -> -1, 0 -> +1
                    curr += 1 - 2 * sign;
                    r.skip_short(2);
                }
                if r.live >= 1 && (1..=20).contains(&curr) && (r.buf >> 63) == 0 {
                    r.skip_short(1);
                    lens[sym_idx] = curr as u8;
                    sym_idx += 1;
                    continue;
                }
            }
            loop {
                if !(1..=20).contains(&curr) { fail!(ErrorKind::BadCodeLength); }
                if !r.bit(data) { break; }
                if r.bit(data) { curr -= 1; } else { curr += 1; }
                if r.pos > data_len && r.overran(data) { fail!(ErrorKind::Truncated); }
            }
            lens[sym_idx] = curr as u8;
            sym_idx += 1;
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

pub struct ScannerC38a<S> {
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
    scratch: ScanScratch38,
}

impl<S: Source> ScannerC38a<S> {
    pub fn new(src: S) -> Self {
        Self {
            src, bit: 0, first: true, done: false, in_stream: false, level: 0, combined: 0,
            blocks: 0, streams: 0, lookahead: LOOKAHEAD, scratch: ScanScratch38::default(),
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
                let item_res = read_item_c38a(data, &mut r, level, syms, &mut self.scratch);
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

// ============================================================================
// Parallel Disjoint-Anchor Finder (`C38a` Whole-Slice + `C38b` Bounded-Window Streaming `io::Read`)
// ============================================================================

const fn build_anchor_bitmap() -> [u64; 1024] {
    let mut bm = [0u64; 1024];
    let magics = [BLOCK_MAGIC, END_MAGIC];
    let mut mi = 0;
    while mi < 2 {
        let m = magics[mi];
        let mut s = 0u32;
        while s < 8 {
            let w = (m << (16 - s)).to_be_bytes();
            let mut r = 0usize;
            while r < 4 {
                let idx = ((w[1 + r] as usize) << 8) | (w[2 + r] as usize);
                bm[idx >> 6] |= 1u64 << (idx & 63);
                r += 1;
            }
            s += 1;
        }
        mi += 1;
    }
    bm
}

static ANCHOR_BITMAP: [u64; 1024] = build_anchor_bitmap();

#[derive(Clone, Copy, Debug)]
struct RawMagic {
    bit: u64,
    is_end: bool,
    hdr_level: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct MagicCand {
    pub bit: u64,
    pub guessed_level: u32,
}

#[cold]
#[inline(never)]
fn verify_quad_raw(data: &[u8], k_base: usize, out: &mut Vec<RawMagic>) {
    let total_bits = data.len() as u64 * 8;
    for k in k_base..k_base + 4 {
        if (k as u64) * 8 + 48 > total_bits {
            return;
        }
        let w64 = if let Some(s8) = data.get(k..k + 8) {
            u64::from_be_bytes([s8[0], s8[1], s8[2], s8[3], s8[4], s8[5], s8[6], s8[7]])
        } else {
            let mut arr = [0u8; 8];
            let avail = data.len() - k;
            arr[..avail].copy_from_slice(&data[k..]);
            u64::from_be_bytes(arr)
        };
        for s in 0..8u32 {
            let bit = (k as u64) * 8 + (s as u64);
            if bit + 48 > total_bits {
                break;
            }
            let v = (w64 >> (16 - s)) & MASK48;
            if v == BLOCK_MAGIC || v == END_MAGIC {
                let hdr_level = if s == 0 && k >= 4 {
                    match data[k - 4..k] {
                        [b'B', b'Z', b'h', l @ b'1'..=b'9'] => l - b'0',
                        _ => 0,
                    }
                } else {
                    0
                };
                out.push(RawMagic {
                    bit,
                    is_end: v == END_MAGIC,
                    hdr_level,
                });
            }
        }
    }
}

fn find_magics_anchor_range(data: &[u8], m_start: usize, m_end: usize, out: &mut Vec<RawMagic>) {
    out.clear();
    if m_start >= m_end || data.len() < 10 {
        return;
    }
    let bm = &ANCHOR_BITMAP;
    let mut p = m_start * 4;
    let p_end = m_end * 4;
    let len = data.len();

    while p + 12 < p_end && p + 14 <= len {
        let s = &data[p..p + 14];
        let i0 = ((s[0] as usize) << 8) | (s[1] as usize);
        let i1 = ((s[4] as usize) << 8) | (s[5] as usize);
        let i2 = ((s[8] as usize) << 8) | (s[9] as usize);
        let i3 = ((s[12] as usize) << 8) | (s[13] as usize);

        let h0 = (bm[i0 >> 6] >> (i0 & 63)) & 1;
        let h1 = (bm[i1 >> 6] >> (i1 & 63)) & 1;
        let h2 = (bm[i2 >> 6] >> (i2 & 63)) & 1;
        let h3 = (bm[i3 >> 6] >> (i3 & 63)) & 1;

        if (h0 | h1 | h2 | h3) != 0 {
            if h0 != 0 { verify_quad_raw(data, p - 4, out); }
            if h1 != 0 { verify_quad_raw(data, p, out); }
            if h2 != 0 { verify_quad_raw(data, p + 4, out); }
            if h3 != 0 { verify_quad_raw(data, p + 8, out); }
        }
        p += 16;
    }

    while p < p_end && p + 2 <= len {
        let idx = ((data[p] as usize) << 8) | (data[p + 1] as usize);
        if ((bm[idx >> 6] >> (idx & 63)) & 1) != 0 {
            verify_quad_raw(data, p - 4, out);
        }
        p += 4;
    }
}

enum WorkerDone {
    Block {
        end_bit: u64,
        crc: u32,
        out: Vec<u8>,
    },
    Failed(ErrorKind),
    Truncated,
}

fn decode_one_cand_c38a(
    data: &[u8],
    bit: u64,
    level: u32,
    k: Kernels,
    scan_scratch: &mut ScanScratch38,
    bwt_scratch: &mut WorkerScratch<OpenCells>,
    syms: &mut Vec<u16>,
    mut out: Vec<u8>,
) -> (WorkerDone, Option<Vec<u8>>) {
    let base = (bit / 8) as usize;
    let sub = data.get(base..).unwrap_or(&[]);
    let mut r = Reg::at(sub, bit % 8);
    let syms_in = std::mem::take(syms);
    let item = read_item_c38a(sub, &mut r, level, syms_in, scan_scratch);
    let overran = r.overran(sub);
    let end_bit = (base as u64) * 8 + r.bit_pos();
    if overran {
        *syms = match item {
            Ok(Item::Block(b)) => b.syms,
            Ok(Item::End(_)) => Vec::new(),
            Err((_, s)) => s,
        };
        return (WorkerDone::Truncated, Some(out));
    }
    match item {
        Ok(Item::Block(b)) => {
            out.clear();
            let res = decode_block_fast(&b, k, bwt_scratch, &mut out);
            *syms = b.syms;
            match res {
                Ok(crc) => (WorkerDone::Block { end_bit, crc, out }, None),
                Err(kind) => (WorkerDone::Failed(kind), Some(out)),
            }
        }
        Ok(Item::End(_)) => (WorkerDone::Failed(ErrorKind::BadBlockMagic), Some(out)),
        Err((kind, s)) => {
            *syms = s;
            (WorkerDone::Failed(kind), Some(out))
        }
    }
}

/// Whole-slice Zero-Serial-Thread Parallel Decoder with `C38a` header/selector/delta acceleration.
pub fn decode_parscan_c38a<F, C>(
    data: &[u8],
    workers: usize,
    k: Kernels,
    thread_clock: C,
    mut on_chunk: F,
) -> Result<(u64, Vec<u64>), Error>
where
    F: FnMut(&[u8]),
    C: Fn() -> u64 + Copy + Send + Sync,
{
    let workers = workers.max(1);
    let slots = 4 * workers;

    struct Shared {
        done: BTreeMap<u64, (u32, WorkerDone)>,
        workers_left: usize,
        in_flight_ready: usize,
        cancelled: bool,
        chain_bit: u64,
        free_out: Vec<Vec<u8>>,
    }

    let shared = Arc::new((
        Mutex::new(Shared {
            done: BTreeMap::new(),
            workers_left: workers,
            in_flight_ready: 0,
            cancelled: false,
            chain_bit: 0,
            free_out: Vec::new(),
        }),
        Condvar::new(),
    ));

    let raw_parts: Arc<Vec<Mutex<Vec<RawMagic>>>> =
        Arc::new((0..workers).map(|_| Mutex::new(Vec::with_capacity(64))).collect());
    let published_cands: Arc<Mutex<Vec<MagicCand>>> = Arc::new(Mutex::new(Vec::new()));
    let cursor = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(workers));

    let total_anchors = if data.len() >= 6 { (data.len() - 2) / 4 } else { 0 };
    let mut serial_merge_ns = 0u64;
    let mut worker_cpus = vec![0u64; workers];

    std::thread::scope(|s| {
        let merge_ns_slot = Arc::new(AtomicUsize::new(0));
        let mut whs = Vec::with_capacity(workers);

        for tid in 0..workers {
            let sh_w = Arc::clone(&shared);
            let rp_w = Arc::clone(&raw_parts);
            let pub_w = Arc::clone(&published_cands);
            let cur_w = Arc::clone(&cursor);
            let bar_w = Arc::clone(&barrier);
            let mns_w = Arc::clone(&merge_ns_slot);

            whs.push(s.spawn(move || {
                let c0 = thread_clock();
                if total_anchors >= 1 {
                    let m_start = 1 + (total_anchors * tid) / workers;
                    let m_end = 1 + (total_anchors * (tid + 1)) / workers;
                    let mut local = rp_w[tid].lock().unwrap();
                    find_magics_anchor_range(data, m_start, m_end, &mut local);
                }
                bar_w.wait();
                if tid == 0 {
                    let m0 = thread_clock();
                    let mut cur_lvl = match data.get(0..4) {
                        Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) => (l - b'0') as u32,
                        _ => 0u32,
                    };
                    let mut out_cands = pub_w.lock().unwrap();
                    out_cands.clear();
                    for part_mu in rp_w.iter() {
                        let part = part_mu.lock().unwrap();
                        for &rm in part.iter() {
                            if rm.hdr_level != 0 { cur_lvl = rm.hdr_level as u32; }
                            if !rm.is_end && cur_lvl != 0 {
                                out_cands.push(MagicCand { bit: rm.bit, guessed_level: cur_lvl });
                            }
                        }
                    }
                    mns_w.store((thread_clock() - m0) as usize, Ordering::Relaxed);
                }
                bar_w.wait();

                let cands_snapshot: Vec<MagicCand> = pub_w.lock().unwrap().clone();
                let n_cands = cands_snapshot.len();
                let mut scan_sc = ScanScratch38::default();
                let mut bwt_sc = WorkerScratch::<OpenCells>::default();
                let mut syms = Vec::new();

                loop {
                    let ci = cur_w.fetch_add(1, Ordering::Relaxed);
                    if ci >= n_cands { break; }
                    let c = cands_snapshot[ci];
                    let out = {
                        let (lk, cv) = &*sh_w;
                        let mut st = lk.lock().unwrap();
                        while !st.cancelled && st.in_flight_ready >= slots {
                            if c.bit <= st.chain_bit { break; }
                            st = cv.wait(st).unwrap();
                        }
                        if st.cancelled { break; }
                        if c.bit < st.chain_bit { continue; }
                        st.free_out.pop().unwrap_or_default()
                    };
                    let (res, ret_out) = decode_one_cand_c38a(
                        data, c.bit, c.guessed_level, k, &mut scan_sc, &mut bwt_sc, &mut syms, out,
                    );
                    let mut st = sh_w.0.lock().unwrap();
                    if let Some(buf) = ret_out { st.free_out.push(buf); }
                    if st.cancelled || c.bit < st.chain_bit {
                        if let WorkerDone::Block { out: buf, .. } = res { st.free_out.push(buf); }
                    } else {
                        if matches!(res, WorkerDone::Block { .. }) { st.in_flight_ready += 1; }
                        st.done.insert(c.bit, (c.guessed_level, res));
                        sh_w.1.notify_all();
                    }
                }
                let mut st = sh_w.0.lock().unwrap();
                st.workers_left -= 1;
                sh_w.1.notify_all();
                thread_clock() - c0
            }));
        }

        let mut bit = 0u64;
        let mut first = true;
        let mut in_stream = false;
        let mut level = 0u32;
        let mut combined = 0u32;
        let mut blocks = 0u64;
        let mut fallback_scan: Option<ScanScratch38> = None;
        let mut fallback_bwt: Option<WorkerScratch<OpenCells>> = None;
        let mut fallback_syms = Vec::new();
        let mut fallback_out = Vec::new();

        let join_res = loop {
            if !in_stream {
                let at = bit;
                let base = (at / 8) as usize;
                let rem = data.get(base..).unwrap_or(&[]);
                if rem.is_empty() {
                    if first { break Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(at) }); }
                    break Ok(());
                }
                match rem {
                    [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                        in_stream = true;
                        level = (l - b'0') as u32;
                        combined = 0;
                        bit += 32;
                    }
                    _ => break Err(Error {
                        kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData },
                        block: None, bit: Some(at),
                    }),
                }
            } else {
                let at = bit;
                let idx = blocks;
                let err_at = |kind| Error { kind, block: Some(idx), bit: Some(at) };
                let base = (at / 8) as usize;
                let sub = data.get(base..).unwrap_or(&[]);
                let mut r_peek = Reg::at(sub, at % 8);
                let magic = (r_peek.bits(sub, 24) as u64) << 24 | r_peek.bits(sub, 24) as u64;
                if magic == END_MAGIC {
                    let stored = r_peek.bits(sub, 32);
                    if r_peek.overran(sub) { break Err(err_at(ErrorKind::Truncated)); }
                    if stored != combined { break Err(err_at(ErrorKind::StreamCrc { stored, computed: combined })); }
                    bit = ((base as u64) * 8 + r_peek.bit_pos()).div_ceil(8) * 8;
                    in_stream = false;
                    first = false;
                    continue;
                }
                if magic != BLOCK_MAGIC {
                    if r_peek.overran(sub) { break Err(err_at(ErrorKind::Truncated)); }
                    break Err(err_at(ErrorKind::BadBlockMagic));
                }

                let taken_worker = {
                    let (lk, cv) = &*shared;
                    let mut st = lk.lock().unwrap();
                    while let Some((&k_bit, _)) = st.done.first_key_value() {
                        if k_bit < at {
                            if let Some((_, (_, WorkerDone::Block { out: buf, .. }))) = st.done.pop_first() {
                                st.in_flight_ready -= 1;
                                st.free_out.push(buf);
                            }
                        } else { break; }
                    }
                    st.chain_bit = at;
                    cv.notify_all();
                    loop {
                        if let Some((w_lvl, w_res)) = st.done.remove(&at) {
                            if matches!(w_res, WorkerDone::Block { .. }) {
                                st.in_flight_ready -= 1;
                                cv.notify_all();
                            }
                            break Some((w_lvl, w_res));
                        }
                        if st.workers_left == 0 { break None; }
                        st = cv.wait(st).unwrap();
                    }
                };

                match taken_worker {
                    Some((w_lvl, w_res)) if w_lvl == level => match w_res {
                        WorkerDone::Block { end_bit, crc, out } => {
                            on_chunk(&out);
                            {
                                let mut st = shared.0.lock().unwrap();
                                st.chain_bit = end_bit;
                                st.free_out.push(out);
                                shared.1.notify_all();
                            }
                            combined = combined.rotate_left(1) ^ crc;
                            blocks += 1;
                            bit = end_bit;
                        }
                        WorkerDone::Truncated => break Err(err_at(ErrorKind::Truncated)),
                        WorkerDone::Failed(kind) => break Err(err_at(kind)),
                    },
                    other => {
                        if let Some((_, WorkerDone::Block { out, .. })) = other {
                            shared.0.lock().unwrap().free_out.push(out);
                        }
                        let sc = fallback_scan.get_or_insert_with(ScanScratch38::default);
                        let bw = fallback_bwt.get_or_insert_with(WorkerScratch::default);
                        let (w_res, _) = decode_one_cand_c38a(
                            data, at, level, k, sc, bw, &mut fallback_syms, std::mem::take(&mut fallback_out),
                        );
                        match w_res {
                            WorkerDone::Block { end_bit, crc, out } => {
                                on_chunk(&out);
                                fallback_out = out;
                                shared.0.lock().unwrap().chain_bit = end_bit;
                                shared.1.notify_all();
                                combined = combined.rotate_left(1) ^ crc;
                                blocks += 1;
                                bit = end_bit;
                            }
                            WorkerDone::Truncated => break Err(err_at(ErrorKind::Truncated)),
                            WorkerDone::Failed(kind) => break Err(err_at(kind)),
                        }
                    }
                }
            }
        };

        {
            let mut st = shared.0.lock().unwrap();
            st.cancelled = true;
            shared.1.notify_all();
        }
        for (i, h) in whs.into_iter().enumerate() {
            worker_cpus[i] = h.join().unwrap();
        }
        serial_merge_ns = merge_ns_slot.load(Ordering::Relaxed) as u64;
        join_res
    })
    .map(|()| (serial_merge_ns, worker_cpus))
}

// ============================================================================
// Candidate C38b: Bounded-Window (`4 MiB`) Streaming Parallel Front End over `R: std::io::Read`
// ============================================================================

/// Decodes any `R: std::io::Read` using a bounded compressed window (`chunk_cap`, e.g. `4 MiB`)
/// and a persistent `W`-worker pool that parallel-scans disjoint 4-byte anchors and steals blocks
/// via `AtomicUsize` on each window, retaining only the unconsumed tail across windows.
pub fn decode_stream_parscan_c38b<R, F, C>(
    mut reader: R,
    chunk_cap: usize,
    workers: usize,
    k: Kernels,
    thread_clock: C,
    mut on_chunk: F,
) -> Result<(u64, Vec<u64>), Error>
where
    R: Read,
    F: FnMut(&[u8]),
    C: Fn() -> u64 + Copy + Send + Sync,
{
    let workers = workers.max(1);
    let slots = 4 * workers;
    let mut win_cap = chunk_cap.max(64 << 10);

    let empty_arc: Arc<Vec<u8>> = Arc::new(Vec::new());

    struct BatchShared {
        // Zero-copy shared window published by main thread before each epoch
        win_arc: Arc<Vec<u8>>,
        rel_start_bit: u64,
        init_level: u32,
        epoch: usize,
        shutting_down: bool,

        // Per-epoch candidate table & completion map
        cands: Vec<MagicCand>,
        done: BTreeMap<u64, (u32, WorkerDone)>,
        workers_active: usize,
        in_flight_ready: usize,
        batch_stop: bool,
        chain_bit: u64,
        free_out: Vec<Vec<u8>>,
    }

    let shared = Arc::new((
        Mutex::new(BatchShared {
            win_arc: Arc::clone(&empty_arc),
            rel_start_bit: 0,
            init_level: 0,
            epoch: 0,
            shutting_down: false,
            cands: Vec::new(),
            done: BTreeMap::new(),
            workers_active: 0,
            in_flight_ready: 0,
            batch_stop: false,
            chain_bit: 0,
            free_out: Vec::new(),
        }),
        Condvar::new(),
    ));

    let raw_parts: Arc<Vec<Mutex<Vec<RawMagic>>>> =
        Arc::new((0..workers).map(|_| Mutex::new(Vec::with_capacity(64))).collect());
    let cursor = Arc::new(AtomicUsize::new(0));
    let bar_find = Arc::new(Barrier::new(workers));
    let bar_ready = Arc::new(Barrier::new(workers));

    let mut serial_io_merge_ns = 0u64;
    let mut worker_cpus = vec![0u64; workers];

    std::thread::scope(|s| {
        let merge_ns_acc = Arc::new(AtomicUsize::new(0));
        let mut whs = Vec::with_capacity(workers);

        for tid in 0..workers {
            let sh_w = Arc::clone(&shared);
            let rp_w = Arc::clone(&raw_parts);
            let cur_w = Arc::clone(&cursor);
            let bf_w = Arc::clone(&bar_find);
            let br_w = Arc::clone(&bar_ready);
            let mns_w = Arc::clone(&merge_ns_acc);

            whs.push(s.spawn(move || {
                let c0 = thread_clock();
                let mut seen_epoch = 0usize;
                let mut scan_sc = ScanScratch38::default();
                let mut bwt_sc = WorkerScratch::<OpenCells>::default();
                let mut syms = Vec::new();
                let mut cands_snapshot: Vec<MagicCand> = Vec::with_capacity(128);

                loop {
                    let (win_local, rel_start, init_lvl) = {
                        let (lk, cv) = &*sh_w;
                        let mut st = lk.lock().unwrap();
                        while !st.shutting_down && st.epoch == seen_epoch {
                            st = cv.wait(st).unwrap();
                        }
                        if st.shutting_down {
                            break;
                        }
                        seen_epoch = st.epoch;
                        (Arc::clone(&st.win_arc), st.rel_start_bit, st.init_level)
                    };

                    {
                        let data = &win_local[..];
                        let total_anchors = if data.len() >= 6 { (data.len() - 2) / 4 } else { 0 };
                        if total_anchors >= 1 {
                            let m_start = 1 + (total_anchors * tid) / workers;
                            let m_end = 1 + (total_anchors * (tid + 1)) / workers;
                            let mut local = rp_w[tid].lock().unwrap();
                            find_magics_anchor_range(data, m_start, m_end, &mut local);
                        } else {
                            rp_w[tid].lock().unwrap().clear();
                        }

                        bf_w.wait();

                        if tid == 0 {
                            let m0 = thread_clock();
                            let mut cur_lvl = init_lvl;
                            if cur_lvl == 0 {
                                let sb = (rel_start / 8) as usize;
                                if let Some([b'B', b'Z', b'h', l @ b'1'..=b'9']) = data.get(sb..sb + 4) {
                                    cur_lvl = (l - b'0') as u32;
                                }
                            }
                            let mut st = sh_w.0.lock().unwrap();
                            st.cands.clear();
                            for part_mu in rp_w.iter() {
                                let part = part_mu.lock().unwrap();
                                for &rm in part.iter() {
                                    if rm.bit + 32 < rel_start {
                                        continue;
                                    }
                                    if rm.hdr_level != 0 && rm.bit >= rel_start {
                                        cur_lvl = rm.hdr_level as u32;
                                    }
                                    if !rm.is_end && cur_lvl != 0 && rm.bit >= rel_start {
                                        st.cands.push(MagicCand { bit: rm.bit, guessed_level: cur_lvl });
                                    }
                                }
                            }
                            cur_w.store(0, Ordering::Relaxed);
                            mns_w.fetch_add((thread_clock() - m0) as usize, Ordering::Relaxed);
                        }

                        br_w.wait();

                        cands_snapshot.clear();
                        cands_snapshot.extend_from_slice(&sh_w.0.lock().unwrap().cands);
                        let n_cands = cands_snapshot.len();

                        loop {
                            let ci = cur_w.fetch_add(1, Ordering::Relaxed);
                            if ci >= n_cands {
                                break;
                            }
                            let c = cands_snapshot[ci];
                            let out = {
                                let (lk, cv) = &*sh_w;
                                let mut st = lk.lock().unwrap();
                                while !st.batch_stop && st.in_flight_ready >= slots {
                                    if c.bit <= st.chain_bit { break; }
                                    st = cv.wait(st).unwrap();
                                }
                                if st.batch_stop { break; }
                                if c.bit < st.chain_bit { continue; }
                                st.free_out.pop().unwrap_or_default()
                            };

                            let (res, ret_out) = decode_one_cand_c38a(
                                data, c.bit, c.guessed_level, k, &mut scan_sc, &mut bwt_sc, &mut syms, out,
                            );

                            let mut st = sh_w.0.lock().unwrap();
                            if let Some(buf) = ret_out { st.free_out.push(buf); }
                            if st.batch_stop || c.bit < st.chain_bit {
                                if let WorkerDone::Block { out: buf, .. } = res { st.free_out.push(buf); }
                            } else {
                                if matches!(res, WorkerDone::Block { .. }) { st.in_flight_ready += 1; }
                                st.done.insert(c.bit, (c.guessed_level, res));
                                sh_w.1.notify_all();
                            }
                        }
                    }
                    // Drop `win_local` BEFORE decrementing `workers_active` so main thread can `Arc::try_unwrap`!
                    drop(win_local);

                    let mut st = sh_w.0.lock().unwrap();
                    st.workers_active -= 1;
                    sh_w.1.notify_all();
                }

                thread_clock() - c0
            }));
        }

        // Main reader + chain joiner over bounded `win_cap` windows
        let mut raw_buf = Vec::with_capacity(win_cap);
        let mut eof = false;
        let mut abs_base_bytes = 0u64;
        let mut rel_bit = 0u64;
        let mut first = true;
        let mut in_stream = false;
        let mut level = 0u32;
        let mut combined = 0u32;
        let mut blocks = 0u64;
        let mut fallback_scan: Option<ScanScratch38> = None;
        let mut fallback_bwt: Option<WorkerScratch<OpenCells>> = None;
        let mut fallback_syms = Vec::new();
        let mut fallback_out = Vec::new();

        let outer_res: Result<(), Error> = 'windows: loop {
            let io_t0 = thread_clock();
            let consumed_bytes = (rel_bit / 8) as usize;
            if consumed_bytes > 0 {
                let rem = raw_buf.len() - consumed_bytes;
                raw_buf.copy_within(consumed_bytes.., 0);
                raw_buf.truncate(rem);
                abs_base_bytes += consumed_bytes as u64;
                rel_bit %= 8;
            }
            while !eof && raw_buf.len() < win_cap {
                let cur = raw_buf.len();
                raw_buf.resize(win_cap, 0);
                match reader.read(&mut raw_buf[cur..]) {
                    Ok(0) => {
                        raw_buf.truncate(cur);
                        eof = true;
                    }
                    Ok(n) => {
                        raw_buf.truncate(cur + n);
                    }
                    Err(e) => {
                        raw_buf.truncate(cur);
                        serial_io_merge_ns += thread_clock() - io_t0;
                        break 'windows Err(Error {
                            kind: ErrorKind::Io(e.kind(), e.to_string()),
                            block: in_stream.then_some(blocks),
                            bit: Some(abs_base_bytes * 8 + rel_bit),
                        });
                    }
                }
            }
            let win_len = raw_buf.len();
            serial_io_merge_ns += thread_clock() - io_t0;

            if win_len == 0 && eof {
                if first {
                    break 'windows Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(0) });
                }
                break 'windows Ok(());
            }

            // Publish zero-copy `Arc<Vec<u8>>` to workers
            let local_view = Arc::new(raw_buf);
            {
                let mut st = shared.0.lock().unwrap();
                for (_, (_, wd)) in std::mem::take(&mut st.done) {
                    if let WorkerDone::Block { out: b, .. } = wd { st.free_out.push(b); }
                }
                st.win_arc = Arc::clone(&local_view);
                st.rel_start_bit = rel_bit;
                st.init_level = if in_stream { level } else { 0 };
                st.workers_active = workers;
                st.in_flight_ready = 0;
                st.batch_stop = false;
                st.chain_bit = rel_bit;
                st.epoch += 1;
                shared.1.notify_all();
            }
            let start_rel_bit = rel_bit;

            let need_more_input = {
                let data = &local_view[..];
                loop {
                    if !in_stream {
                        let at = rel_bit;
                        let abs_at = abs_base_bytes * 8 + at;
                        let base = (at / 8) as usize;
                        let rem = data.get(base..).unwrap_or(&[]);
                        if rem.len() < 4 && !eof {
                            break true;
                        }
                        if rem.is_empty() && eof {
                            if first {
                                break 'windows Err(Error { kind: ErrorKind::Empty, block: None, bit: Some(abs_at) });
                            }
                            break false;
                        }
                        match rem {
                            [b'B', b'Z', b'h', l @ b'1'..=b'9', ..] => {
                                in_stream = true;
                                level = (l - b'0') as u32;
                                combined = 0;
                                rel_bit += 32;
                            }
                            _ => {
                                break 'windows Err(Error {
                                    kind: if first { ErrorKind::NotBz2 } else { ErrorKind::TrailingData },
                                    block: None,
                                    bit: Some(abs_at),
                                });
                            }
                        }
                    } else {
                        let at = rel_bit;
                        let abs_at = abs_base_bytes * 8 + at;
                        let idx = blocks;
                        let err_at = |kind| Error { kind, block: Some(idx), bit: Some(abs_at) };
                        let base = (at / 8) as usize;
                        let sub = data.get(base..).unwrap_or(&[]);
                        if sub.len() < 12 && !eof {
                            break true;
                        }
                        let mut r_peek = Reg::at(sub, at % 8);
                        let magic = (r_peek.bits(sub, 24) as u64) << 24 | r_peek.bits(sub, 24) as u64;
                        if magic == END_MAGIC {
                            let stored = r_peek.bits(sub, 32);
                            if r_peek.overran(sub) {
                                if !eof { break true; }
                                break 'windows Err(err_at(ErrorKind::Truncated));
                            }
                            if stored != combined {
                                break 'windows Err(err_at(ErrorKind::StreamCrc { stored, computed: combined }));
                            }
                            rel_bit = ((base as u64) * 8 + r_peek.bit_pos()).div_ceil(8) * 8;
                            in_stream = false;
                            first = false;
                            continue;
                        }
                        if magic != BLOCK_MAGIC {
                            if r_peek.overran(sub) {
                                if !eof { break true; }
                                break 'windows Err(err_at(ErrorKind::Truncated));
                            }
                            break 'windows Err(err_at(ErrorKind::BadBlockMagic));
                        }

                        let taken_worker = {
                            let (lk, cv) = &*shared;
                            let mut st = lk.lock().unwrap();
                            while let Some((&k_bit, _)) = st.done.first_key_value() {
                                if k_bit < at {
                                    if let Some((_, (_, WorkerDone::Block { out: buf, .. }))) = st.done.pop_first() {
                                        st.in_flight_ready -= 1;
                                        st.free_out.push(buf);
                                    }
                                } else { break; }
                            }
                            st.chain_bit = at;
                            cv.notify_all();
                            loop {
                                if let Some((w_lvl, w_res)) = st.done.remove(&at) {
                                    if matches!(w_res, WorkerDone::Block { .. }) {
                                        st.in_flight_ready -= 1;
                                        cv.notify_all();
                                    }
                                    break Some((w_lvl, w_res));
                                }
                                if st.workers_active == 0 { break None; }
                                st = cv.wait(st).unwrap();
                            }
                        };

                        match taken_worker {
                            Some((w_lvl, w_res)) if w_lvl == level => match w_res {
                                WorkerDone::Block { end_bit, crc, out } => {
                                    on_chunk(&out);
                                    {
                                        let mut st = shared.0.lock().unwrap();
                                        st.chain_bit = end_bit;
                                        st.free_out.push(out);
                                        shared.1.notify_all();
                                    }
                                    combined = combined.rotate_left(1) ^ crc;
                                    blocks += 1;
                                    rel_bit = end_bit;
                                }
                                WorkerDone::Truncated => {
                                    if !eof { break true; }
                                    break 'windows Err(err_at(ErrorKind::Truncated));
                                }
                                WorkerDone::Failed(kind) => break 'windows Err(err_at(kind)),
                            },
                            other => {
                                if let Some((_, WorkerDone::Block { out, .. })) = other {
                                    shared.0.lock().unwrap().free_out.push(out);
                                }
                                let sc = fallback_scan.get_or_insert_with(ScanScratch38::default);
                                let bw = fallback_bwt.get_or_insert_with(WorkerScratch::default);
                                let (w_res, _) = decode_one_cand_c38a(
                                    data, at, level, k, sc, bw, &mut fallback_syms, std::mem::take(&mut fallback_out),
                                );
                                match w_res {
                                    WorkerDone::Block { end_bit, crc, out } => {
                                        on_chunk(&out);
                                        fallback_out = out;
                                        shared.0.lock().unwrap().chain_bit = end_bit;
                                        shared.1.notify_all();
                                        combined = combined.rotate_left(1) ^ crc;
                                        blocks += 1;
                                        rel_bit = end_bit;
                                    }
                                    WorkerDone::Truncated => {
                                        if !eof { break true; }
                                        break 'windows Err(err_at(ErrorKind::Truncated));
                                    }
                                    WorkerDone::Failed(kind) => break 'windows Err(err_at(kind)),
                                }
                            }
                        }
                    }
                }
            };

            drop(local_view);

            // Drain/stop workers for this window epoch and reclaim `raw_buf` with zero allocation via `Arc::try_unwrap`
            raw_buf = {
                let (lk, cv) = &*shared;
                let mut st = lk.lock().unwrap();
                st.batch_stop = true;
                cv.notify_all();
                while st.workers_active > 0 {
                    st = cv.wait(st).unwrap();
                }
                let old = std::mem::replace(&mut st.win_arc, Arc::clone(&empty_arc));
                drop(st);
                Arc::try_unwrap(old).unwrap_or_else(|a| (*a).clone())
            };

            if !need_more_input {
                break 'windows Ok(());
            }
            if (rel_bit / 8) == (start_rel_bit / 8) && win_len >= win_cap {
                if win_cap >= MAX_ITEM {
                    break 'windows Err(Error {
                        kind: ErrorKind::BlockTooLarge,
                        block: Some(blocks),
                        bit: Some(abs_base_bytes * 8 + rel_bit),
                    });
                }
                win_cap = (win_cap * 2).min(MAX_ITEM);
            }
        };

        {
            let mut st = shared.0.lock().unwrap();
            st.batch_stop = true;
            st.shutting_down = true;
            shared.1.notify_all();
        }
        for (i, h) in whs.into_iter().enumerate() {
            worker_cpus[i] = h.join().unwrap();
        }
        serial_io_merge_ns += merge_ns_acc.load(Ordering::Relaxed) as u64;
        outer_res
    })
    .map(|()| (serial_io_merge_ns, worker_cpus))
}
