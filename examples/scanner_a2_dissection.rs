//! Smooth-zone cold dissection & redesign probe of Serial Scanner `A2` (`A2a..d`:
//! header/bitmap, selector MTF, delta code-lengths, and `Huff::build_into`) in 100% safe Rust
//! (`#![forbid(unsafe_code)]`, zero `src/` edits).
//!
//! Frozen Stop Test (frozen before any benchmark data):
//! - PASS if a 100% symbol-for-symbol and bit-position-exact safe-Rust `A2a..d` redesign:
//!   1. Cuts standalone `A2a..d` vs post-`#1034` baseline (`A2a+b+c + A2d_u8_gated64`) by `>= 25.0%`
//!      (non-overlapping 7-repeat thread-CPU spread), AND
//!   2. Cuts full post-`#1034`-style `A2a..d + A3` serial scan `S'` (coalesced `K=2` + `u8` LUT +
//!      no-zero `syms`) by `>= 3.0%` non-overlapping across BOTH baseline x86-64 and
//!      `-C target-cpu=native`, confirmed under Intel JCC-erratum 32B branch alignment control
//!      (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`),
//!      with zero `A3` regression and no `> 2%` regression on off-wikidata guard corpora.
//! - STOP otherwise (record scoped dead-end row in `frontier/dead-ends.tsv`).
//!
//! Usage:
//!   cargo run --release --example scanner_a2_dissection -- /tmp/wikidata.bz2 [--repeats 7] [EXTRA_BZ2...]
#![forbid(unsafe_code)]

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::ErrorKind;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;
const MAX_GROUPS: usize = 6;
const MAX_SELECTORS: usize = 18002;
const MAX_ALPHA: usize = 258;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;
const LUT_BITS: u32 = 10;
const LUT_LEN: usize = 1 << LUT_BITS;
const SUB_CAP: usize = 1024;
const SUB_BAD_U16: u16 = 0xffff;
const PAIR_GATE_SEL: usize = 64;

// Post-#1034 byte-aligned u8 LUT layout:
const SYM256: u32 = 1 << 26;
const SUB_PTR: u32 = 1 << 27;
const BAD: u32 = 1 << 28;
const EOB_FAST: u32 = 1 << 29;
const SYM1: u32 = 8;
const SYM2: u32 = 16;
const LEN1: u32 = 24;
const CNT_SH: u32 = 30;
const STEPS: usize = 5;
const CHUNK_LEN: usize = 1042;
const CHUNK_STOP: usize = 1000;

/// Linux steal-free thread on-CPU nanoseconds from `/proc/thread-self/schedstat` field 0.
fn thread_cpu_ns() -> u64 {
    if let Ok(s) = fs::read_to_string("/proc/thread-self/schedstat") {
        if let Some(tok) = s.split_whitespace().next() {
            if let Ok(v) = tok.parse::<u64>() {
                return v;
            }
        }
    }
    0
}

fn median_range(vals: &[f64]) -> (f64, f64, f64) {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

// ---- Exact register bit reader (faithful mirror of src/bits.rs + #1034 skip_short_shl_u8) ----

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

// ---- Precomputed A2c Delta Code-Length LUTs (100% const, safe Rust) ----

/// Single-symbol 8-bit delta code-length LUT (`1 KiB`):
/// Covers all `<= 7`-bit transitions (`0`, `1x0`, `1x1x0`, `1x1x1x0` -> up to 3 delta steps).
/// Layout of `u32`:
/// - bits `0..7`:   `bits` consumed (`1..=7`, or `0` if `>= 4` delta steps -> slow fallback)
/// - bits `8..15`:  `net_delta` (`i8` as `u8`)
/// - bits `16..23`: `min_off` (`i8` as `u8`, `<= 0`, minimum intermediate offset from `curr`)
/// - bits `24..31`: `max_off` (`i8` as `u8`, `>= 0`, maximum intermediate offset from `curr`)
const fn build_delta_lut1() -> [u32; 256] {
    let mut lut = [0u32; 256];
    let mut p = 0usize;
    while p < 256 {
        let mut bit = 7i32;
        let mut off = 0i8;
        let mut min_off = 0i8;
        let mut max_off = 0i8;
        let consumed: u8;
        loop {
            if bit < 0 {
                consumed = 0; // need >= 4 delta steps -> slow fallback
                break;
            }
            let b0 = (p >> bit) & 1;
            bit -= 1;
            if b0 == 0 {
                consumed = (7 - bit) as u8;
                break;
            }
            if bit < 0 {
                consumed = 0;
                break;
            }
            let b1 = (p >> bit) & 1;
            bit -= 1;
            if b1 != 0 {
                off -= 1;
            } else {
                off += 1;
            }
            if off < min_off {
                min_off = off;
            }
            if off > max_off {
                max_off = off;
            }
        }
        lut[p] = (consumed as u32)
            | (((off as u8) as u32) << 8)
            | (((min_off as u8) as u32) << 16)
            | (((max_off as u8) as u32) << 24);
        p += 1;
    }
    lut
}

static DELTA_LUT1: [u32; 256] = build_delta_lut1();

/// Multi-symbol 8-bit delta code-length LUT (`256 × 16 B = 4 KiB` L1d):
/// Greedily decodes ALL complete symbols fitting in an 8-bit prefix (1..=8 symbols; 0 if slow).
#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct MultiDelta {
    bits: u8,
    count: u8,
    min_off: i8,
    max_off: i8,
    net: i8,
    _pad: [u8; 3],
    offs: [i8; 8],
}

const fn build_delta_lut_multi() -> [MultiDelta; 256] {
    let mut lut = [MultiDelta {
        bits: 0,
        count: 0,
        min_off: 0,
        max_off: 0,
        net: 0,
        _pad: [0; 3],
        offs: [0; 8],
    }; 256];
    let mut p = 0usize;
    while p < 256 {
        let mut bit = 7i32;
        let mut off = 0i8;
        let mut min_off = 0i8;
        let mut max_off = 0i8;
        let mut count = 0usize;
        let mut consumed = 0u8;
        let mut offs = [0i8; 8];
        while bit >= 0 && count < 8 {
            let mut cur_bit = bit;
            let mut cur_off = off;
            let mut cur_min = min_off;
            let mut cur_max = max_off;
            let mut ok = false;
            loop {
                if cur_bit < 0 {
                    break;
                }
                let b0 = (p >> cur_bit) & 1;
                cur_bit -= 1;
                if b0 == 0 {
                    ok = true;
                    break;
                }
                if cur_bit < 0 {
                    break;
                }
                let b1 = (p >> cur_bit) & 1;
                cur_bit -= 1;
                if b1 != 0 {
                    cur_off -= 1;
                } else {
                    cur_off += 1;
                }
                if cur_off < cur_min {
                    cur_min = cur_off;
                }
                if cur_off > cur_max {
                    cur_max = cur_off;
                }
            }
            if !ok {
                break;
            }
            offs[count] = cur_off;
            count += 1;
            bit = cur_bit;
            off = cur_off;
            min_off = cur_min;
            max_off = cur_max;
            consumed = (7 - bit) as u8;
        }
        lut[p] = MultiDelta {
            bits: consumed,
            count: count as u8,
            min_off,
            max_off,
            net: off,
            _pad: [0; 3],
            offs,
        };
        p += 1;
    }
    lut
}

static DELTA_LUT_MULTI: [MultiDelta; 256] = build_delta_lut_multi();

/// Precomputed MTF low-byte masks for `j in 0..6`: `LOW_MASK[j] = (1u64 << (j*8)) - 1`.
const LOW_MASK: [u64; 6] = [
    0x0000_0000_0000_0000,
    0x0000_0000_0000_00ff,
    0x0000_0000_0000_ffff,
    0x0000_0000_00ff_ffff,
    0x0000_0000_ffff_ffff,
    0x0000_00ff_ffff_ffff,
];

// ---- Post-#1034 Huff table (u8 LUT + SubLut + Canonical slow) ----

struct HuffU8 {
    lut: [u32; LUT_LEN],
    sub_lut: [u16; SUB_CAP],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    eob: u16,
}

impl Default for HuffU8 {
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

impl HuffU8 {
    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    /// Faithful mirror of PR `#1034`'s `Huff::build_into(lens, build_pairs)`.
    fn build_post1034(&mut self, lens: &[u8], build_pairs: bool) {
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

    /// Candidate `A2d_split_vec`:
    /// 1. Compact `counts: [u16; 21]` + sequential `perm` stepping (zero `self.symbol` calls).
    /// 2. Zeroes ONLY `d_arr[..LUT_LEN - filled_to]` in scratch (no 1 KiB stack memset!).
    /// 3. Non-aliasing `split_at_mut` template clone (`vpaddd` SIMD auto-vectorization in safe Rust!).
    fn build_split_vec(&mut self, lens: &[u8], build_pairs: bool, d_arr: &mut [u8; LUT_LEN]) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let mut counts = [0u16; MAX_CODE_LEN];
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
        let mut acc = 0u16;
        for i in min_l..=max_l {
            base[i] = acc as i32;
            pos[i] = acc;
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
            vec += counts[i] as i32;
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
            let hi = (limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            let sh = LUT_BITS - zn;
            let base_zn = base[zn as usize];
            for p in a..=hi {
                let idx = (p as i32) - base_zn;
                let e = if (0..alpha as i32).contains(&idx) {
                    let s = self.perm[idx as usize];
                    if s == eob {
                        EOB_FAST | zn
                    } else if s == 256 {
                        SYM256 | zn
                    } else {
                        (1u32 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn
                    }
                } else {
                    BAD
                };
                let end_idx = ((p + 1) << sh) as usize;
                self.lut[(p << sh) as usize..end_idx].fill(e);
                filled_to = end_idx;
            }
            end_at_len[zn as usize] = filled_to;
            a = 2 * a.max(hi + 1);
        }

        self.build_tail_sublut(filled_to, max_l, min_len, a, d_arr);

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
                    let (left, right) = self.lut.split_at_mut(off_p);
                    for (dst, &src) in right[..u_lim].iter_mut().zip(&left[off0..off0 + u_lim]) {
                        *dst = src.wrapping_add(delta);
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

    /// Candidate `A2d_direct_single_write`:
    /// Writes EVERY `self.lut[0..1024]` entry **EXACTLY ONCE**!
    /// 1. Computes canonical `counts`, `perm`, `limit`, `base` + records compact short-code slices
    ///    (`zn <= 10`) in `short_codes` (max 258 entries, typically ~15-60 short codes!).
    /// 2. For `zn in min_len..=max_pair_l1` (when `build_pairs` is true):
    ///    - Builds the first valid symbol `s0`'s `span = 1 << (10 - zn)` template DIRECTLY from
    ///      `short_codes` (each `l2 <= 10 - zn` fills its contiguous `1 << (10 - zn - l2)` run via
    ///      slice `.fill()` — zero `lut[(u << zn)]` random reads, zero K=1 pre-fill!), filling
    ///      the remaining suffix `u_lim..span` once with `k1_e0`.
    ///    - Clones every subsequent valid symbol `s1` of length `zn` in ONE non-aliasing
    ///      `split_at_mut` SIMD `vpaddd` loop across the entire `0..span` block (no K=1 pre-fill!).
    ///    - Special symbols (`eob`, `256`, `BAD`) do a single `.fill()`.
    /// 3. For `zn > max_pair_l1` (or `!build_pairs`), does a single `.fill()` per symbol.
    /// 4. Tail `SubLut` zeroes ONLY `d_arr[filled_to..LUT_LEN]` and supports `build_sublut` gating.
    fn build_direct_single_write(
        &mut self,
        lens: &[u8],
        build_pairs: bool,
        build_sublut: bool,
        d_arr: &mut [u8; LUT_LEN],
    ) {
        let alpha = lens.len();
        let eob = (alpha - 1) as u16;
        let mut counts = [0u16; MAX_CODE_LEN];
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
        let mut acc = 0u16;
        for i in min_l..=max_l {
            base[i] = acc as i32;
            pos[i] = acc;
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
            vec += counts[i] as i32;
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
        let max_pair_l1 = if build_pairs {
            LUT_BITS.saturating_sub(min_len)
        } else {
            0
        };

        // Precompute canonical ranges `[lo_p, hi_p]` for zn in min_len..=10 (<= 10 iterations!).
        let mut zn_ranges = [(0i64, -1i64); 11];
        let mut a = 0i64;
        let mut filled_to = 0usize;
        for zn in min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (limit[zn as usize] as i64).min(top);
            zn_ranges[zn as usize] = (a, hi);
            if hi >= a {
                filled_to = ((hi + 1) << (LUT_BITS - zn)) as usize;
            }
            a = 2 * a.max(hi + 1);
        }

        // Fast K=1-only path (when !build_pairs or min_len * 2 > LUT_BITS)
        if !build_pairs || max_pair_l1 < min_len {
            for zn in min_len..=LUT_BITS {
                let (lo_p, hi_p) = zn_ranges[zn as usize];
                if lo_p > hi_p {
                    continue;
                }
                let sh = LUT_BITS - zn;
                let base_zn = base[zn as usize];
                for p in lo_p..=hi_p {
                    let idx = (p as i32) - base_zn;
                    let e = if (0..alpha as i32).contains(&idx) {
                        let s = self.perm[idx as usize];
                        if s == eob {
                            EOB_FAST | zn
                        } else if s == 256 {
                            SYM256 | zn
                        } else {
                            (1u32 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn
                        }
                    } else {
                        BAD
                    };
                    self.lut[(p << sh) as usize..((p + 1) << sh) as usize].fill(e);
                }
            }
        } else {
            // Single-write fused K=2 + K=1 builder!
            // Pairable zn: min_len..=max_pair_l1
            for zn in min_len..=max_pair_l1 {
                let (lo_p, hi_p) = zn_ranges[zn as usize];
                if lo_p > hi_p {
                    continue;
                }
                let sh = (LUT_BITS - zn) as usize;
                let span = 1usize << sh;
                let base_zn = base[zn as usize];
                let max_l2 = LUT_BITS - zn;
                let mut tpl: Option<(usize, u32)> = None;

                for p in lo_p..=hi_p {
                    let off_p = (p as usize) << sh;
                    let idx = (p as i32) - base_zn;
                    if !(0..alpha as i32).contains(&idx) {
                        self.lut[off_p..off_p + span].fill(BAD);
                        continue;
                    }
                    let s1 = self.perm[idx as usize];
                    if s1 == eob {
                        self.lut[off_p..off_p + span].fill(EOB_FAST | zn);
                        continue;
                    }
                    if s1 == 256 {
                        self.lut[off_p..off_p + span].fill(SYM256 | zn);
                        continue;
                    }
                    let s1_u32 = s1 as u32;
                    if let Some((off0, s0)) = tpl {
                        // Invariant: within fixed `zn`, `perm` is sorted by ascending symbol ID,
                        // so `s0 < s1 <= 255` -> `(s1 - s0) << 8` never borrows or carries into bit 16!
                        let delta = s1_u32.wrapping_sub(s0) << SYM1;
                        let (left, right) = self.lut.split_at_mut(off_p);
                        for (dst, &src) in right[..span].iter_mut().zip(&left[off0..off0 + span]) {
                            *dst = src.wrapping_add(delta);
                        }
                    } else {
                        // First valid symbol of length `zn`: fill `off_p..off_p + span` DIRECTLY!
                        let k1_e = (1u32 << CNT_SH) | (zn << LEN1) | (s1_u32 << SYM1) | zn;
                        let pair_base = (2u32 << CNT_SH) | (zn << LEN1) | (s1_u32 << SYM1);
                        let mut sub_pos = 0usize;
                        for l2 in min_len..=max_l2 {
                            let (lo2, hi2) = zn_ranges[l2 as usize];
                            if lo2 > hi2 {
                                continue;
                            }
                            let sh2 = (max_l2 - l2) as usize;
                            let step2 = 1usize << sh2;
                            let base2 = base[l2 as usize];
                            let tot_bits = zn + l2;
                            for p2 in lo2..=hi2 {
                                let idx2 = (p2 as i32) - base2;
                                let e_pair = if (0..alpha as i32).contains(&idx2) {
                                    let s2 = self.perm[idx2 as usize];
                                    if s2 == eob || s2 == 256 {
                                        k1_e
                                    } else {
                                        pair_base | ((s2 as u32) << SYM2) | tot_bits
                                    }
                                } else {
                                    k1_e
                                };
                                let start = off_p + ((p2 as usize) << sh2);
                                self.lut[start..start + step2].fill(e_pair);
                                sub_pos = ((p2 as usize) + 1) << sh2;
                            }
                        }
                        if sub_pos < span {
                            self.lut[off_p + sub_pos..off_p + span].fill(k1_e);
                        }
                        tpl = Some((off_p, s1_u32));
                    }
                }
            }
            // Non-pairable short codes: (max_pair_l1 + 1)..=LUT_BITS (single K=1 fill!)
            for zn in (max_pair_l1 + 1)..=LUT_BITS {
                let (lo_p, hi_p) = zn_ranges[zn as usize];
                if lo_p > hi_p {
                    continue;
                }
                let sh = LUT_BITS - zn;
                let base_zn = base[zn as usize];
                for p in lo_p..=hi_p {
                    let idx = (p as i32) - base_zn;
                    let e = if (0..alpha as i32).contains(&idx) {
                        let s = self.perm[idx as usize];
                        if s == eob {
                            EOB_FAST | zn
                        } else if s == 256 {
                            SYM256 | zn
                        } else {
                            (1u32 << CNT_SH) | (zn << LEN1) | ((s as u32) << SYM1) | zn
                        }
                    } else {
                        BAD
                    };
                    self.lut[(p << sh) as usize..((p + 1) << sh) as usize].fill(e);
                }
            }
        }

        if build_sublut {
            self.build_tail_sublut(filled_to, max_l, min_len, a, d_arr);
        } else if filled_to < LUT_LEN {
            if max_l <= LUT_BITS as usize {
                self.lut[filled_to..].fill(BAD);
            } else {
                self.lut[filled_to..].fill(0);
            }
        }
    }

    #[inline(always)]
    fn build_tail_sublut(
        &mut self,
        filled_to: usize,
        max_l: usize,
        min_len: u32,
        a: i64,
        d_arr: &mut [u8; LUT_LEN],
    ) {
        if filled_to >= LUT_LEN {
            return;
        }
        if max_l <= LUT_BITS as usize {
            self.lut[filled_to..].fill(BAD);
            return;
        }
        self.lut[filled_to..].fill(0);
        // Zero ONLY the active tail slice `filled_to..LUT_LEN` (typically 10-60 bytes vs 1024 B!)
        d_arr[filled_to..].fill(0);
        let mut ranges = [(0u32, 0i64, -1i64); 10];
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
                let base_zn = self.base[zn as usize];
                for p in lo_c..=hi_c {
                    let e_root = self.lut[(p >> sh) as usize];
                    if (e_root & SUB_PTR) == 0 {
                        continue;
                    }
                    let d_w = (e_root >> 16) & 15;
                    let sub_off = (e_root & 0xffff) as usize;
                    let idx_sym = (p as i32) - base_zn;
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

    // ---- Post-#1034 coalesced decode_run (A3) ----

    #[inline(always)]
    fn decode_one(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
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
    fn decode_run(
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

        if rem_syms >= 2 {
            r.refill(data);
            let mut steps = 0usize;
            while rem_syms >= 2 {
                if steps == 4 {
                    r.refill(data);
                    steps = 0;
                }
                steps += 1;
                let e = self.lut[r.peek_live(LUT_BITS) as usize];
                let c = (e >> CNT_SH) as usize;
                if c != 0 {
                    r.skip_short_shl_u8(e);
                    let dst = &mut chunk[ck..ck + 2];
                    dst[0] = ((e >> SYM1) as u8) as u16;
                    dst[1] = ((e >> SYM2) as u8) as u16;
                    ck += c;
                    rem_syms -= c;
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
                    chunk[ck] = s;
                    ck += 1;
                    rem_syms -= 1;
                    if s == eob {
                        syms.extend_from_slice(&chunk[..ck]);
                        *ck_out = 0;
                        *r_out = r;
                        return Ok(true);
                    }
                }
            }
        }
        if rem_syms == 1 {
            let s = match self.decode_one(&mut r, data) {
                Ok(s) => s,
                Err(err) => {
                    *ck_out = ck;
                    *r_out = r;
                    return Err(err);
                }
            };
            chunk[ck] = s;
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
            r.refill(data);
            return (Ok(256), r);
        } else if (e & BAD) != 0 {
            return (Err(ErrorKind::BadHuffmanCode), r);
        }
        let mut zn = self.min_len;
        let mut zvec = r.peek(data, zn) as i32;
        while zvec > self.limit[zn as usize] {
            zn += 1;
            if zn > 20 {
                return (Err(ErrorKind::BadHuffmanCode), r);
            }
            zvec = r.peek(data, zn) as i32;
        }
        r.skip(zn);
        let s = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        (s, r)
    }
}

// ---- Reusable scratch ----

struct ProbeScratch {
    selectors: Box<[u8; MAX_SELECTORS]>,
    runs: Vec<(u8, u16)>,
    tables: Box<[HuffU8; MAX_GROUPS]>,
    chunk: Box<[u16; CHUNK_LEN]>,
    d_arr: Box<[u8; LUT_LEN]>,
}

impl Default for ProbeScratch {
    fn default() -> Self {
        Self {
            selectors: Box::new([0u8; MAX_SELECTORS]),
            runs: Vec::with_capacity(MAX_SELECTORS),
            tables: Box::new(std::array::from_fn(|_| HuffU8::default())),
            chunk: Box::new([0u16; CHUNK_LEN]),
            d_arr: Box::new([0u8; LUT_LEN]),
        }
    }
}

// ---- A2a: Header + in_use16 ----

struct HeaderInfo {
    alpha: usize,
    n_groups: usize,
    n_sel: usize,
    seq_to_unseq: [u8; 256],
}

#[inline(always)]
fn parse_a2a(data: &[u8], r: &mut Reg) -> Result<HeaderInfo, ErrorKind> {
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic == END_MAGIC {
        let _ = r.bits(data, 32);
        return Err(ErrorKind::BadBlockMagic);
    }
    if magic != BLOCK_MAGIC {
        return Err(ErrorKind::BadBlockMagic);
    }
    let _crc = r.bits(data, 32);
    if r.bit(data) {
        return Err(ErrorKind::Randomised);
    }
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
    if n_in_use == 0 {
        return Err(ErrorKind::NoSymbols);
    }
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) {
        return Err(ErrorKind::BadGroups);
    }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 {
        return Err(ErrorKind::BadSelectors);
    }
    Ok(HeaderInfo {
        alpha: n_in_use + 2,
        n_groups,
        n_sel,
        seq_to_unseq,
    })
}

// ---- A2b: Selector MTF decode variants ----

#[inline(never)]
fn parse_a2b_shipped(
    data: &[u8],
    r: &mut Reg,
    n_groups: usize,
    n_sel: usize,
    selectors: &mut [u8; MAX_SELECTORS],
) -> Result<(usize, u8), ErrorKind> {
    let data_len = data.len();
    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let sel_buf = &mut selectors[..sel_cap];
    let mut used_groups = 0u8;
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_sel {
        if r.live < 8 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            r.skip_short(n_groups as u32);
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
    Ok((sel_cap, used_groups))
}

/// Candidate `A2b_fast_mtf`: hoists `overran` & `MAX_SELECTORS`, batches refills (8 sel/refill),
/// zero-work `j == 0` fast path + `LOW_MASK[j]` table.
#[inline(never)]
fn parse_a2b_fast_mtf(
    data: &[u8],
    r: &mut Reg,
    n_groups: usize,
    n_sel: usize,
    selectors: &mut [u8; MAX_SELECTORS],
) -> Result<(usize, [u16; MAX_GROUPS]), ErrorKind> {
    let data_len = data.len();
    let sel_cap = n_sel.min(MAX_SELECTORS);
    let mut group_uses = [0u16; MAX_GROUPS];
    // Safe margin: each selector is at most 7 bits (< 1 B), so `pos + n_sel + 8 <= data_len`
    // guarantees zero overrun across all `n_sel` selectors!
    if r.pos.saturating_add(n_sel + 8) > data_len || n_sel > MAX_SELECTORS {
        let (cap, _u) = parse_a2b_shipped(data, r, n_groups, n_sel, selectors)?;
        for &g in &selectors[..cap] {
            group_uses[g as usize] += 1;
        }
        return Ok((cap, group_uses));
    }

    let mut mtf: u64 = 0x0000_0504_0302_0100;
    let sel_buf = &mut selectors[..sel_cap];
    let mut i = 0usize;
    r.refill(data);
    while i + 8 <= sel_cap {
        let batch: &mut [u8; 8] = (&mut sel_buf[i..i + 8]).try_into().unwrap();
        for slot in batch.iter_mut() {
            let v = if (r.buf as i64) >= 0 {
                r.buf <<= 1;
                r.live -= 1;
                mtf as u8
            } else {
                let j = (!r.buf).leading_zeros() as usize;
                if j >= n_groups {
                    r.skip_short(n_groups as u32);
                    return Err(ErrorKind::BadSelector);
                }
                r.skip_short((j + 1) as u32);
                let sh = (j * 8) as u32;
                let v = ((mtf >> sh) & 0xff) as u8;
                let lm = LOW_MASK[j];
                mtf = (mtf & !((lm << 8) | 0xff)) | ((mtf & lm) << 8) | (v as u64);
                v
            };
            *slot = v;
            group_uses[v as usize] += 1;
        }
        i += 8;
        r.refill(data);
    }
    while i < sel_cap {
        if r.live < 8 {
            r.refill(data);
        }
        let v = if (r.buf as i64) >= 0 {
            r.buf <<= 1;
            r.live -= 1;
            mtf as u8
        } else {
            let j = (!r.buf).leading_zeros() as usize;
            if j >= n_groups {
                r.skip_short(n_groups as u32);
                return Err(ErrorKind::BadSelector);
            }
            r.skip_short((j + 1) as u32);
            let sh = (j * 8) as u32;
            let v = ((mtf >> sh) & 0xff) as u8;
            let lm = LOW_MASK[j];
            mtf = (mtf & !((lm << 8) | 0xff)) | ((mtf & lm) << 8) | (v as u64);
            v
        };
        sel_buf[i] = v;
        group_uses[v as usize] += 1;
        i += 1;
    }
    Ok((sel_cap, group_uses))
}

/// Candidate `A2b_direct_runs`: emits coalesced `(group, run_len)` pairs DIRECTLY during `A2b`,
/// using `r.buf.leading_zeros()` to consume entire `j == 0` streaks in ONE `lzcnt` instruction!
/// Eliminates BOTH the 501k `sel_buf` writes AND `#1034`'s `while sel_buf[sel + rem] == g` re-scan!
#[inline(never)]
fn parse_a2b_direct_runs(
    data: &[u8],
    r: &mut Reg,
    n_groups: usize,
    n_sel: usize,
    selectors_fallback: &mut [u8; MAX_SELECTORS],
    runs: &mut Vec<(u8, u16)>,
) -> Result<[u16; MAX_GROUPS], ErrorKind> {
    runs.clear();
    let data_len = data.len();
    let mut group_uses = [0u16; MAX_GROUPS];
    if r.pos.saturating_add(n_sel + 8) > data_len || n_sel > MAX_SELECTORS {
        let (cap, _) = parse_a2b_shipped(data, r, n_groups, n_sel, selectors_fallback)?;
        let mut idx = 0usize;
        while idx < cap {
            let g = selectors_fallback[idx];
            let mut m = 1usize;
            while idx + m < cap && selectors_fallback[idx + m] == g {
                m += 1;
            }
            runs.push((g, m as u16));
            group_uses[g as usize] += m as u16;
            idx += m;
        }
        return Ok(group_uses);
    }

    let mut mtf: u64 = 0x0000_0504_0302_0100;
    // Decode first selector to seed `(cur_g, cur_run)`
    if r.live < 8 {
        r.refill(data);
    }
    let j0 = (!r.buf).leading_zeros() as usize;
    if j0 >= n_groups {
        r.skip_short(n_groups as u32);
        return Err(ErrorKind::BadSelector);
    }
    r.skip_short((j0 + 1) as u32);
    let mut cur_g = if j0 == 0 {
        mtf as u8
    } else {
        let sh = (j0 * 8) as u32;
        let v = ((mtf >> sh) & 0xff) as u8;
        let lm = LOW_MASK[j0];
        mtf = (mtf & !((lm << 8) | 0xff)) | ((mtf & lm) << 8) | (v as u64);
        v
    };
    let mut cur_run = 1u16;
    let mut rem = n_sel - 1;

    // Drain any initial `j == 0` streak after selector 0 so loop invariant holds: top bit is 1.
    while rem > 0 {
        if r.live < 8 {
            r.refill(data);
        }
        if (r.buf as i64) < 0 {
            break;
        }
        let k = r.buf.leading_zeros().min(r.live).min(rem as u32);
        r.skip_short(k);
        cur_run += k as u16;
        rem -= k as usize;
    }

    // Loop invariant: `rem > 0` => next unread bit is `1` (`j >= 1`).
    // One iteration per coalesced run (189k vs 501k), ZERO `j==0` vs `j>=1` branch!
    while rem > 0 {
        if r.live < 24 {
            r.refill(data);
        }
        let j = (!r.buf).leading_zeros() as usize;
        if j >= n_groups {
            r.skip_short(n_groups as u32);
            return Err(ErrorKind::BadSelector);
        }
        r.skip_short((j + 1) as u32);
        rem -= 1;
        runs.push((cur_g, cur_run));
        group_uses[cur_g as usize] += cur_run;
        let sh = (j * 8) as u32;
        let v = ((mtf >> sh) & 0xff) as u8;
        let lm = LOW_MASK[j];
        mtf = (mtf & !((lm << 8) | 0xff)) | ((mtf & lm) << 8) | (v as u64);
        cur_g = v;
        // Consume ALL trailing `j == 0` bits branchlessly (k <= r.live <= 63).
        let k = r.buf.leading_zeros().min(r.live).min(rem as u32);
        r.skip_short(k);
        cur_run = 1 + k as u16;
        rem -= k as usize;
        if rem > 0 && r.live == 0 {
            r.refill(data);
            while rem > 0 && (r.buf as i64) >= 0 {
                let k2 = r.buf.leading_zeros().min(r.live).min(rem as u32);
                r.skip_short(k2);
                cur_run += k2 as u16;
                rem -= k2 as usize;
                if rem > 0 && r.live == 0 {
                    r.refill(data);
                }
            }
        }
    }
    runs.push((cur_g, cur_run));
    group_uses[cur_g as usize] += cur_run;
    Ok(group_uses)
}

// ---- A2c: Delta code-length decode variants ----

#[inline(never)]
fn parse_a2c_shipped(
    data: &[u8],
    r: &mut Reg,
    alpha: usize,
    lens: &mut [u8; MAX_ALPHA],
) -> Result<(), ErrorKind> {
    let data_len = data.len();
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
    Ok(())
}

#[cold]
#[inline(never)]
fn parse_a2c_one_slow(data: &[u8], r: &mut Reg, curr: &mut i32) -> Result<u8, ErrorKind> {
    let data_len = data.len();
    loop {
        if !(1..=20).contains(curr) {
            return Err(ErrorKind::BadCodeLength);
        }
        if !r.bit(data) {
            break;
        }
        if r.bit(data) {
            *curr -= 1;
        } else {
            *curr += 1;
        }
        if r.pos > data_len && r.overran(data) {
            return Err(ErrorKind::Truncated);
        }
    }
    Ok(*curr as u8)
}

/// Candidate `A2c_lut1`: 8-bit single-symbol LUT (`1 KiB`), zero unpredictable `0`/`10`/`11` branches.
#[inline(never)]
fn parse_a2c_lut1(
    data: &[u8],
    r: &mut Reg,
    alpha: usize,
    lens: &mut [u8; MAX_ALPHA],
) -> Result<(), ErrorKind> {
    let data_len = data.len();
    let mut curr = r.bits(data, 5) as i32;
    for l in lens[..alpha].iter_mut() {
        if r.live < 8 {
            r.refill(data);
        }
        let e = DELTA_LUT1[(r.buf >> 56) as usize];
        let bits = e & 0xff;
        let min_off = ((e >> 16) as u8 as i8) as i32;
        let max_off = ((e >> 24) as u8 as i8) as i32;
        if bits != 0 && curr + min_off >= 1 && curr + max_off <= 20 {
            curr += ((e >> 8) as u8 as i8) as i32;
            r.skip_short(bits);
            *l = curr as u8;
        } else {
            *l = parse_a2c_one_slow(data, r, &mut curr)?;
        }
    }
    if r.pos > data_len && r.overran(data) {
        return Err(ErrorKind::Truncated);
    }
    Ok(())
}

/// Candidate `A2c_lut_multi`: 8-bit multi-symbol LUT (`4 KiB`), decoding 1..=8 code lengths per lookup!
#[inline(never)]
fn parse_a2c_lut_multi(
    data: &[u8],
    r: &mut Reg,
    alpha: usize,
    lens: &mut [u8; MAX_ALPHA],
) -> Result<(), ErrorKind> {
    let data_len = data.len();
    let mut curr = r.bits(data, 5) as i32;
    let out = &mut lens[..alpha];
    let mut i = 0usize;
    while i + 8 <= alpha {
        if r.live < 8 {
            r.refill(data);
        }
        let e = &DELTA_LUT_MULTI[(r.buf >> 56) as usize];
        if e.count != 0 && curr + (e.min_off as i32) >= 1 && curr + (e.max_off as i32) <= 20 {
            let cnt = e.count as usize;
            let dst = &mut out[i..i + 8];
            let base = curr as u8;
            // Branchless 8-byte write (always in bounds since `i + 8 <= alpha`); advance by `cnt`.
            for (d, &off) in dst.iter_mut().zip(&e.offs) {
                *d = base.wrapping_add(off as u8);
            }
            curr += e.net as i32;
            r.skip_short(e.bits as u32);
            i += cnt;
        } else {
            out[i] = parse_a2c_one_slow(data, r, &mut curr)?;
            i += 1;
        }
    }
    while i < alpha {
        if r.live < 8 {
            r.refill(data);
        }
        let e = DELTA_LUT1[(r.buf >> 56) as usize];
        let bits = e & 0xff;
        let min_off = ((e >> 16) as u8 as i8) as i32;
        let max_off = ((e >> 24) as u8 as i8) as i32;
        if bits != 0 && curr + min_off >= 1 && curr + max_off <= 20 {
            curr += ((e >> 8) as u8 as i8) as i32;
            r.skip_short(bits);
            out[i] = curr as u8;
        } else {
            out[i] = parse_a2c_one_slow(data, r, &mut curr)?;
        }
        i += 1;
    }
    if r.pos > data_len && r.overran(data) {
        return Err(ErrorKind::Truncated);
    }
    Ok(())
}

// ---- Full block A2+A3 runners (post-#1034 baseline vs candidate combinations) ----

#[derive(Clone, Copy)]
struct FullCfg {
    name: &'static str,
    a2b_mode: u8,           // 0=shipped, 1=fast_mtf, 2=direct_runs
    a2c_mode: u8,           // 0=shipped, 1=lut1, 2=lut_multi
    a2d_mode: u8,           // 0=post1034, 1=split_vec, 2=direct_single_write
    pair_gate_table: u16,   // 0=block-gate(n_sel>=64), >0=per-table(group_uses[g]>=T)
    sublut_gate_table: u16, // 0=always build sublut, >0=only if group_uses[g]>=T
}

fn scan_block_cfg(
    data: &[u8],
    r_in: Reg,
    scratch: &mut ProbeScratch,
    syms: &mut Vec<u16>,
    cfg: FullCfg,
) -> Result<Reg, ErrorKind> {
    let mut r = r_in;
    let hdr = parse_a2a(data, &mut r)?;
    let _ = hdr.seq_to_unseq[0];

    let mut group_uses = [0u16; MAX_GROUPS];
    let mut sel_cap = 0usize;
    match cfg.a2b_mode {
        0 => {
            let (cap, used) = parse_a2b_shipped(
                data,
                &mut r,
                hdr.n_groups,
                hdr.n_sel,
                &mut scratch.selectors,
            )?;
            sel_cap = cap;
            for (g, u) in group_uses.iter_mut().enumerate().take(hdr.n_groups) {
                if (used >> g) & 1 != 0 {
                    *u = 1;
                }
            }
        }
        1 => {
            let (cap, uses) = parse_a2b_fast_mtf(
                data,
                &mut r,
                hdr.n_groups,
                hdr.n_sel,
                &mut scratch.selectors,
            )?;
            sel_cap = cap;
            group_uses = uses;
        }
        _ => {
            group_uses = parse_a2b_direct_runs(
                data,
                &mut r,
                hdr.n_groups,
                hdr.n_sel,
                &mut scratch.selectors,
                &mut scratch.runs,
            )?;
        }
    }

    let mut lens = [0u8; MAX_ALPHA];
    for (g, &uses_g) in group_uses.iter().enumerate().take(hdr.n_groups) {
        match cfg.a2c_mode {
            0 => parse_a2c_shipped(data, &mut r, hdr.alpha, &mut lens)?,
            1 => parse_a2c_lut1(data, &mut r, hdr.alpha, &mut lens)?,
            _ => parse_a2c_lut_multi(data, &mut r, hdr.alpha, &mut lens)?,
        }
        if uses_g > 0 {
            let build_pairs = if cfg.pair_gate_table == 0 {
                hdr.n_sel >= PAIR_GATE_SEL
            } else {
                uses_g >= cfg.pair_gate_table
            };
            let build_sublut = cfg.sublut_gate_table == 0 || uses_g >= cfg.sublut_gate_table;
            match cfg.a2d_mode {
                0 => scratch.tables[g].build_post1034(&lens[..hdr.alpha], build_pairs),
                1 => scratch.tables[g].build_split_vec(
                    &lens[..hdr.alpha],
                    build_pairs,
                    &mut scratch.d_arr,
                ),
                _ => scratch.tables[g].build_direct_single_write(
                    &lens[..hdr.alpha],
                    build_pairs,
                    build_sublut,
                    &mut scratch.d_arr,
                ),
            }
        }
    }

    // A3: post-#1034 coalesced decode_run (zero syms.resize memset)
    syms.clear();
    let data_len = data.len();
    let mut ck = 0usize;
    if cfg.a2b_mode == 2 {
        for &(g, run_m) in &scratch.runs {
            let t = &scratch.tables[g as usize];
            let mut rem = run_m as usize;
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
                if t.decode_run(
                    &mut r,
                    data,
                    &mut scratch.chunk,
                    &mut ck,
                    syms,
                    take * GROUP,
                )? {
                    return Ok(r);
                }
                if check_overran && r.pos > data_len && r.overran(data) {
                    return Err(ErrorKind::Truncated);
                }
                rem -= take;
            }
        }
    } else {
        let sel_buf = &scratch.selectors[..sel_cap];
        let mut sel = 0usize;
        while sel < sel_cap {
            let g = sel_buf[sel];
            let mut rem = 1usize;
            while sel + rem < sel_cap && sel_buf[sel + rem] == g {
                rem += 1;
            }
            sel += rem;
            let t = &scratch.tables[g as usize];
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
                if t.decode_run(
                    &mut r,
                    data,
                    &mut scratch.chunk,
                    &mut ck,
                    syms,
                    take * GROUP,
                )? {
                    return Ok(r);
                }
                if check_overran && r.pos > data_len && r.overran(data) {
                    return Err(ErrorKind::Truncated);
                }
                rem -= take;
            }
        }
    }
    Err(ErrorKind::SelectorOverrun)
}

// ---- Pre-extracted block offsets & table census ----

struct BlockLoc {
    bit_start: u64,
    bit_after_a2a: u64,
    bit_after_a2b: u64,
    bit_end: u64,
    alpha: usize,
    n_groups: usize,
    n_sel: usize,
    ref_syms: Vec<u16>,
    tables_lens: Vec<Vec<u8>>,
    group_uses: [u16; MAX_GROUPS],
}

fn load_corpus_blocks(data: &[u8]) -> Vec<BlockLoc> {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms_buf = Vec::new();
    let mut scratch = ProbeScratch::default();
    let mut locs = Vec::new();

    loop {
        match sc.next(syms_buf).expect("Scanner::next failed") {
            Event::Done => break,
            Event::Block {
                bit, end, symbols, ..
            } => {
                let ref_syms = symbols.syms.clone();
                syms_buf = symbols.syms;
                let mut r = Reg::at(data, bit);
                let hdr = parse_a2a(data, &mut r).unwrap();
                let bit_after_a2a = r.bit_pos();
                let group_uses = parse_a2b_direct_runs(
                    data,
                    &mut r,
                    hdr.n_groups,
                    hdr.n_sel,
                    &mut scratch.selectors,
                    &mut scratch.runs,
                )
                .unwrap();
                let bit_after_a2b = r.bit_pos();
                let mut tables_lens = Vec::with_capacity(hdr.n_groups);
                let mut lens = [0u8; MAX_ALPHA];
                for _ in 0..hdr.n_groups {
                    parse_a2c_shipped(data, &mut r, hdr.alpha, &mut lens).unwrap();
                    tables_lens.push(lens[..hdr.alpha].to_vec());
                }
                locs.push(BlockLoc {
                    bit_start: bit,
                    bit_after_a2a,
                    bit_after_a2b,
                    bit_end: end,
                    alpha: hdr.alpha,
                    n_groups: hdr.n_groups,
                    n_sel: hdr.n_sel,
                    ref_syms,
                    tables_lens,
                    group_uses,
                });
            }
        }
    }
    locs
}

fn print_census(name: &str, data: &[u8], locs: &[BlockLoc]) {
    let mut full_blks = 0usize;
    let mut runt_blks = 0usize;
    let mut tot_sel = 0usize;
    let mut tot_tables = 0usize;
    let mut used_tables = 0usize;
    let mut unused_tables = 0usize;
    let mut use_buckets = [0usize; 5]; // 0, 1..4, 5..15, 16..63, >=64
    let mut max_l_le10 = 0usize;
    let mut max_l_gt10 = 0usize;
    let mut sum_short_le9 = 0usize;
    let mut sum_tail = 0usize;
    let mut sum_lens = 0usize;
    let mut a2c_bits_hist = [0usize; 5]; // 1b, 3b, 5b, 7b, slow
    let mut multi_lookups = 0usize;

    for b in locs {
        if b.n_sel >= PAIR_GATE_SEL {
            full_blks += 1;
        } else {
            runt_blks += 1;
        }
        tot_sel += b.n_sel;
        tot_tables += b.n_groups;
        let mut r = Reg::at(data, b.bit_after_a2b);
        for g in 0..b.n_groups {
            let u = b.group_uses[g];
            if u == 0 {
                unused_tables += 1;
                use_buckets[0] += 1;
            } else {
                used_tables += 1;
                if u <= 4 {
                    use_buckets[1] += 1;
                } else if u <= 15 {
                    use_buckets[2] += 1;
                } else if u <= 63 {
                    use_buckets[3] += 1;
                } else {
                    use_buckets[4] += 1;
                }
            }
            let lens = &b.tables_lens[g];
            sum_lens += lens.len();
            let max_l = lens.iter().copied().max().unwrap_or(0);
            if u > 0 {
                if max_l <= 10 {
                    max_l_le10 += 1;
                } else {
                    max_l_gt10 += 1;
                }
                sum_short_le9 += lens.iter().filter(|&&l| l <= 9).count();
                // Compute filled_to
                let mut counts = [0u16; MAX_CODE_LEN];
                for &l in lens {
                    counts[l as usize] += 1;
                }
                let mut filled = 0usize;
                for (zn, &cnt) in counts.iter().enumerate().take(11).skip(1) {
                    filled += (cnt as usize) << (10 - zn);
                }
                sum_tail += LUT_LEN.saturating_sub(filled);
            }
            // Histogram A2c transitions
            let mut curr = r.bits(data, 5) as i32;
            let mut i = 0usize;
            while i < lens.len() {
                if r.live < 8 {
                    r.refill(data);
                }
                let e = DELTA_LUT1[(r.buf >> 56) as usize];
                let bits = e & 0xff;
                let min_off = ((e >> 16) as u8 as i8) as i32;
                let max_off = ((e >> 24) as u8 as i8) as i32;
                if bits != 0 && curr + min_off >= 1 && curr + max_off <= 20 {
                    match bits {
                        1 => a2c_bits_hist[0] += 1,
                        3 => a2c_bits_hist[1] += 1,
                        5 => a2c_bits_hist[2] += 1,
                        _ => a2c_bits_hist[3] += 1,
                    }
                    curr += ((e >> 8) as u8 as i8) as i32;
                    r.skip_short(bits);
                } else {
                    a2c_bits_hist[4] += 1;
                    let _ = parse_a2c_one_slow(data, &mut r, &mut curr).unwrap();
                }
                i += 1;
            }
        }
        // Count multi-symbol LUT lookups for A2c
        let mut rm = Reg::at(data, b.bit_after_a2b);
        for g in 0..b.n_groups {
            let mut curr = rm.bits(data, 5) as i32;
            let alpha = b.tables_lens[g].len();
            let mut i = 0usize;
            while i + 8 <= alpha {
                if rm.live < 8 {
                    rm.refill(data);
                }
                multi_lookups += 1;
                let e = &DELTA_LUT_MULTI[(rm.buf >> 56) as usize];
                if e.count != 0 && curr + (e.min_off as i32) >= 1 && curr + (e.max_off as i32) <= 20
                {
                    curr += e.net as i32;
                    rm.skip_short(e.bits as u32);
                    i += e.count as usize;
                } else {
                    let _ = parse_a2c_one_slow(data, &mut rm, &mut curr).unwrap();
                    i += 1;
                }
            }
            while i < alpha {
                if rm.live < 8 {
                    rm.refill(data);
                }
                multi_lookups += 1;
                let e = DELTA_LUT1[(rm.buf >> 56) as usize];
                let bits = e & 0xff;
                let min_off = ((e >> 16) as u8 as i8) as i32;
                let max_off = ((e >> 24) as u8 as i8) as i32;
                if bits != 0 && curr + min_off >= 1 && curr + max_off <= 20 {
                    curr += ((e >> 8) as u8 as i8) as i32;
                    rm.skip_short(bits);
                } else {
                    let _ = parse_a2c_one_slow(data, &mut rm, &mut curr).unwrap();
                }
                i += 1;
            }
        }
    }

    println!(
        "\n=== Census: {name} ({} blks: {full_blks} full, {runt_blks} runts) ===",
        locs.len()
    );
    println!(
        "  selectors={tot_sel}, tables={tot_tables} (used={used_tables}, unused={unused_tables})"
    );
    println!(
        "  per-table uses: 0={}, 1..4={}, 5..15={}, 16..63={}, >=64={}",
        use_buckets[0], use_buckets[1], use_buckets[2], use_buckets[3], use_buckets[4]
    );
    println!(
        "  used tables: max_l<=10: {max_l_le10}, max_l>10: {max_l_gt10} | mean short(<=9)={:.1}, mean tail(1024-filled)={:.1} entries",
        sum_short_le9 as f64 / used_tables.max(1) as f64,
        sum_tail as f64 / used_tables.max(1) as f64
    );
    println!(
        "  A2c lens={sum_lens}: 1b={} ({:.1}%), 3b={} ({:.1}%), 5b={} ({:.1}%), 7b={} ({:.1}%), slow={} ({:.2}%) | multi-LUT lookups={multi_lookups} ({:.2} lens/lookup)",
        a2c_bits_hist[0], 100.0 * a2c_bits_hist[0] as f64 / sum_lens as f64,
        a2c_bits_hist[1], 100.0 * a2c_bits_hist[1] as f64 / sum_lens as f64,
        a2c_bits_hist[2], 100.0 * a2c_bits_hist[2] as f64 / sum_lens as f64,
        a2c_bits_hist[3], 100.0 * a2c_bits_hist[3] as f64 / sum_lens as f64,
        a2c_bits_hist[4], 100.0 * a2c_bits_hist[4] as f64 / sum_lens as f64,
        sum_lens as f64 / multi_lookups.max(1) as f64
    );
}

/// Synthetic edge-case exactness test (`alpha = 258`, `sym = 256`, `eob = 257`, `len = 1..20`,
/// `max_l <= 10`, oversubscribed/incomplete tables, long delta runs > 4 steps).
fn verify_synthetic_edge_cases() {
    let mut scratch = ProbeScratch::default();
    let mut h_ref = HuffU8::default();
    let mut h_sv = HuffU8::default();
    let mut h_dir = HuffU8::default();

    // Case 1: alpha = 258 with sym 256 at len 4, eob 257 at len 5, rest lengths 5..14
    let mut lens258 = vec![10u8; 258];
    lens258[0] = 2;
    lens258[1] = 3;
    lens258[2] = 4;
    lens258[256] = 4;
    lens258[257] = 5;
    for l in lens258[3..60].iter_mut() {
        *l = 9;
    }
    for l in lens258[60..256].iter_mut() {
        *l = 12;
    }
    for &bp in &[false, true] {
        h_ref.build_post1034(&lens258, bp);
        h_sv.build_split_vec(&lens258, bp, &mut scratch.d_arr);
        h_dir.build_direct_single_write(&lens258, bp, true, &mut scratch.d_arr);
        assert_eq!(h_ref.lut, h_sv.lut, "split_vec LUT mismatch (bp={bp})");
        assert_eq!(h_ref.lut, h_dir.lut, "direct LUT mismatch (bp={bp})");
        assert_eq!(h_ref.sub_lut, h_dir.sub_lut, "direct sub_lut mismatch");
        assert_eq!(h_ref.limit, h_dir.limit);
        assert_eq!(h_ref.base[..=12], h_dir.base[..=12]);
        assert_eq!(h_ref.perm, h_dir.perm);
    }

    // Case 2: 500 random valid + incomplete + oversubscribed tables across alpha 3..=258
    let mut rng = 0x9e37_79b9_7f4a_7c15u64;
    let mut next_u64 = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for case in 0..500 {
        let alpha = 3 + (next_u64() % 256) as usize;
        let lo = 1 + (next_u64() % 12) as u8;
        let hi = lo + (next_u64() % (21 - lo as u64)) as u8;
        let lens: Vec<u8> = (0..alpha)
            .map(|_| lo + (next_u64() % ((hi - lo + 1) as u64)) as u8)
            .collect();
        for &bp in &[false, true] {
            h_ref.build_post1034(&lens, bp);
            h_sv.build_split_vec(&lens, bp, &mut scratch.d_arr);
            h_dir.build_direct_single_write(&lens, bp, true, &mut scratch.d_arr);
            assert_eq!(h_ref.lut, h_sv.lut, "rand case {case} split_vec bp={bp}");
            assert_eq!(h_ref.lut, h_dir.lut, "rand case {case} direct bp={bp}");
        }
    }
    println!("Synthetic edge-case LUT parity: 1,002/1,002 tables bit-identical!");
}

fn bench_corpus(name: &str, data: &[u8], locs: &[BlockLoc], repeats: usize) {
    let mut scratch = ProbeScratch::default();
    let mut syms = Vec::with_capacity(MAX_SELECTORS * GROUP);

    // Verify 100% LUT bit-exactness across EVERY real table in corpus
    let mut h_ref = HuffU8::default();
    let mut h_sv = HuffU8::default();
    let mut h_dir = HuffU8::default();
    let mut tables_checked = 0usize;
    for b in locs {
        for (g, lens) in b.tables_lens.iter().enumerate() {
            if b.group_uses[g] == 0 {
                continue;
            }
            for &bp in &[false, true] {
                h_ref.build_post1034(lens, bp);
                h_sv.build_split_vec(lens, bp, &mut scratch.d_arr);
                h_dir.build_direct_single_write(lens, bp, true, &mut scratch.d_arr);
                assert_eq!(h_ref.lut, h_sv.lut);
                assert_eq!(h_ref.lut, h_dir.lut);
            }
            tables_checked += 1;
        }
    }
    println!(
        "\n=== Benchmarks: {name} ({} blks, {tables_checked} used tables bit-exact) ===",
        locs.len()
    );

    // ---- Part A: Standalone A2 sub-phase & candidate benchmarks (7 interleaved repeats) ----
    // Each arm runs `iters` passes so thread-CPU timing is well above schedstat granularity.
    let iters = if locs.len() >= 100 { 25 } else { 100 };
    let scale = 1.0 / (iters as f64);

    let sub_names = [
        "A2a_header",
        "A2b_shipped",
        "A2b_fast_mtf",
        "A2b_direct_runs",
        "A2c_shipped",
        "A2c_lut1",
        "A2c_lut_multi",
        "A2d_shipped_all",
        "A2d_post1034_g64",
        "A2d_split_vec_g64",
        "A2d_direct_g64",
        "A2d_direct_t16",
        "A2d_direct_t16_sub4",
        "A2_post1034_tot",
        "A2_cand_tot_g64",
        "A2_cand_tot_t16",
    ];
    let n_sub = sub_names.len();
    let mut sub_cpu: Vec<Vec<f64>> = vec![Vec::new(); n_sub];
    let mut sub_wall: Vec<Vec<f64>> = vec![Vec::new(); n_sub];
    let mut lens_tmp = [0u8; MAX_ALPHA];

    for rep in 0..repeats {
        for step in 0..n_sub {
            let ai = (step + rep) % n_sub;
            let c0 = thread_cpu_ns();
            let t0 = Instant::now();
            let mut acc = 0usize;
            for _ in 0..iters {
                match ai {
                    0 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_start);
                            let h = parse_a2a(data, &mut r).unwrap();
                            acc += h.alpha + h.n_sel;
                        }
                    }
                    1 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2a);
                            let (cap, u) = parse_a2b_shipped(
                                data,
                                &mut r,
                                b.n_groups,
                                b.n_sel,
                                &mut scratch.selectors,
                            )
                            .unwrap();
                            acc += cap + u as usize;
                        }
                    }
                    2 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2a);
                            let (cap, uses) = parse_a2b_fast_mtf(
                                data,
                                &mut r,
                                b.n_groups,
                                b.n_sel,
                                &mut scratch.selectors,
                            )
                            .unwrap();
                            acc += cap + uses[0] as usize;
                        }
                    }
                    3 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2a);
                            let uses = parse_a2b_direct_runs(
                                data,
                                &mut r,
                                b.n_groups,
                                b.n_sel,
                                &mut scratch.selectors,
                                &mut scratch.runs,
                            )
                            .unwrap();
                            acc += scratch.runs.len() + uses[0] as usize;
                        }
                    }
                    4 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2b);
                            for _ in 0..b.n_groups {
                                parse_a2c_shipped(data, &mut r, b.alpha, &mut lens_tmp).unwrap();
                                acc += lens_tmp[0] as usize;
                            }
                        }
                    }
                    5 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2b);
                            for _ in 0..b.n_groups {
                                parse_a2c_lut1(data, &mut r, b.alpha, &mut lens_tmp).unwrap();
                                acc += lens_tmp[0] as usize;
                            }
                        }
                    }
                    6 => {
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_after_a2b);
                            for _ in 0..b.n_groups {
                                parse_a2c_lut_multi(data, &mut r, b.alpha, &mut lens_tmp).unwrap();
                                acc += lens_tmp[0] as usize;
                            }
                        }
                    }
                    7 => {
                        for b in locs {
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_post1034(lens, true);
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    8 => {
                        for b in locs {
                            let bp = b.n_sel >= PAIR_GATE_SEL;
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_post1034(lens, bp);
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    9 => {
                        for b in locs {
                            let bp = b.n_sel >= PAIR_GATE_SEL;
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_split_vec(lens, bp, &mut scratch.d_arr);
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    10 => {
                        for b in locs {
                            let bp = b.n_sel >= PAIR_GATE_SEL;
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_direct_single_write(
                                        lens,
                                        bp,
                                        true,
                                        &mut scratch.d_arr,
                                    );
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    11 => {
                        for b in locs {
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_direct_single_write(
                                        lens,
                                        b.group_uses[g] >= 16,
                                        true,
                                        &mut scratch.d_arr,
                                    );
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    12 => {
                        for b in locs {
                            for (g, lens) in b.tables_lens.iter().enumerate() {
                                if b.group_uses[g] > 0 {
                                    scratch.tables[g].build_direct_single_write(
                                        lens,
                                        b.group_uses[g] >= 16,
                                        b.group_uses[g] >= 4,
                                        &mut scratch.d_arr,
                                    );
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    13 => {
                        // Full A2a..d post-#1034 baseline (no A3)
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_start);
                            let h = parse_a2a(data, &mut r).unwrap();
                            let (_cap, used) = parse_a2b_shipped(
                                data,
                                &mut r,
                                h.n_groups,
                                h.n_sel,
                                &mut scratch.selectors,
                            )
                            .unwrap();
                            let bp = h.n_sel >= PAIR_GATE_SEL;
                            for g in 0..h.n_groups {
                                parse_a2c_shipped(data, &mut r, h.alpha, &mut lens_tmp).unwrap();
                                if (used >> g) & 1 != 0 {
                                    scratch.tables[g].build_post1034(&lens_tmp[..h.alpha], bp);
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    14 => {
                        // Full A2a..d candidate (direct_runs + lut_multi + direct_single_write g64)
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_start);
                            let h = parse_a2a(data, &mut r).unwrap();
                            let uses = parse_a2b_direct_runs(
                                data,
                                &mut r,
                                h.n_groups,
                                h.n_sel,
                                &mut scratch.selectors,
                                &mut scratch.runs,
                            )
                            .unwrap();
                            let bp = h.n_sel >= PAIR_GATE_SEL;
                            for (g, &ug) in uses.iter().enumerate().take(h.n_groups) {
                                parse_a2c_lut_multi(data, &mut r, h.alpha, &mut lens_tmp).unwrap();
                                if ug > 0 {
                                    scratch.tables[g].build_direct_single_write(
                                        &lens_tmp[..h.alpha],
                                        bp,
                                        true,
                                        &mut scratch.d_arr,
                                    );
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                    _ => {
                        // Full A2a..d candidate (direct_runs + lut_multi + direct_single_write t16)
                        for b in locs {
                            let mut r = Reg::at(data, b.bit_start);
                            let h = parse_a2a(data, &mut r).unwrap();
                            let uses = parse_a2b_direct_runs(
                                data,
                                &mut r,
                                h.n_groups,
                                h.n_sel,
                                &mut scratch.selectors,
                                &mut scratch.runs,
                            )
                            .unwrap();
                            for (g, &ug) in uses.iter().enumerate().take(h.n_groups) {
                                parse_a2c_lut_multi(data, &mut r, h.alpha, &mut lens_tmp).unwrap();
                                if ug > 0 {
                                    scratch.tables[g].build_direct_single_write(
                                        &lens_tmp[..h.alpha],
                                        ug >= 16,
                                        true,
                                        &mut scratch.d_arr,
                                    );
                                    acc += scratch.tables[g].lut[0] as usize;
                                }
                            }
                        }
                    }
                }
            }
            black_box(acc);
            let cpu_ms = (thread_cpu_ns().saturating_sub(c0)) as f64 / 1e6 * scale;
            let wall_ms = t0.elapsed().as_secs_f64() * 1e3 * scale;
            sub_cpu[ai].push(cpu_ms);
            sub_wall[ai].push(wall_ms);
        }
    }

    println!("  --- Standalone A2 sub-phase & combined A2a..d (ms/pass, {repeats} repeats x {iters} iters) ---");
    let a2_ref = median_range(&sub_cpu[13]);
    for (ai, &aname) in sub_names.iter().enumerate() {
        let (cm, c0, c1) = median_range(&sub_cpu[ai]);
        let (wm, w0, w1) = median_range(&sub_wall[ai]);
        let extra = match ai {
            2 | 3 => {
                let r = median_range(&sub_cpu[1]);
                let disj = if c1 < r.1 { "DISJOINT" } else { "overlap" };
                format!("vs A2b_ship: {:+.1}% ({disj})", 100.0 * (cm - r.0) / r.0)
            }
            5 | 6 => {
                let r = median_range(&sub_cpu[4]);
                let disj = if c1 < r.1 { "DISJOINT" } else { "overlap" };
                format!("vs A2c_ship: {:+.1}% ({disj})", 100.0 * (cm - r.0) / r.0)
            }
            8..=12 => {
                let r = median_range(&sub_cpu[8]);
                let disj = if c1 < r.1 { "DISJOINT" } else { "overlap" };
                format!("vs A2d_p1034: {:+.1}% ({disj})", 100.0 * (cm - r.0) / r.0)
            }
            14 | 15 => {
                let disj = if c1 < a2_ref.1 { "DISJOINT" } else { "overlap" };
                format!(
                    "vs A2_p1034: {:+.1}% ({disj})",
                    100.0 * (cm - a2_ref.0) / a2_ref.0
                )
            }
            _ => String::new(),
        };
        println!(
            "    {aname:<20}: cpu={cm:6.3}ms [{c0:6.3}..{c1:6.3}] | wall={wm:6.3}ms [{w0:6.3}..{w1:6.3}]  {extra}"
        );
    }

    // ---- Part B: Joint End-to-End A2a..d + A3 (full post-#1034 serial scan S') ----
    let cfgs = [
        FullCfg {
            name: "S_post1034_ref",
            a2b_mode: 0,
            a2c_mode: 0,
            a2d_mode: 0,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_fast_a2b_only",
            a2b_mode: 2,
            a2c_mode: 0,
            a2d_mode: 0,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_fast_a2c_only",
            a2b_mode: 0,
            a2c_mode: 2,
            a2d_mode: 0,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_fast_a2d_only",
            a2b_mode: 0,
            a2c_mode: 0,
            a2d_mode: 2,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_all_g64",
            a2b_mode: 2,
            a2c_mode: 2,
            a2d_mode: 2,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_all_t16",
            a2b_mode: 2,
            a2c_mode: 2,
            a2d_mode: 2,
            pair_gate_table: 16,
            sublut_gate_table: 0,
        },
        FullCfg {
            name: "S_all_t16_sub4",
            a2b_mode: 2,
            a2c_mode: 2,
            a2d_mode: 2,
            pair_gate_table: 16,
            sublut_gate_table: 4,
        },
        FullCfg {
            name: "S_all_fastmtf_g64",
            a2b_mode: 1,
            a2c_mode: 2,
            a2d_mode: 2,
            pair_gate_table: 0,
            sublut_gate_table: 0,
        },
    ];

    // Exact symbol-for-symbol + bit_pos parity check across ALL configs & blocks!
    for &cfg in &cfgs {
        for b in locs {
            let r_end = scan_block_cfg(
                data,
                Reg::at(data, b.bit_start),
                &mut scratch,
                &mut syms,
                cfg,
            )
            .unwrap();
            assert_eq!(
                r_end.bit_pos(),
                b.bit_end,
                "bit_pos mismatch on {name} cfg={}",
                cfg.name
            );
            assert_eq!(
                syms, b.ref_syms,
                "symbols mismatch on {name} cfg={}",
                cfg.name
            );
        }
    }

    let full_iters = if locs.len() >= 100 {
        5
    } else if locs.len() >= 10 {
        10
    } else {
        40
    };
    let fscale = 1.0 / (full_iters as f64);
    let mut full_cpu: Vec<Vec<f64>> = vec![Vec::new(); cfgs.len()];
    let mut full_wall: Vec<Vec<f64>> = vec![Vec::new(); cfgs.len()];
    for rep in 0..repeats {
        for step in 0..cfgs.len() {
            let ci = (step + rep) % cfgs.len();
            let cfg = cfgs[ci];
            let c0 = thread_cpu_ns();
            let t0 = Instant::now();
            let mut acc = 0usize;
            for _ in 0..full_iters {
                for b in locs {
                    let r_end = scan_block_cfg(
                        data,
                        Reg::at(data, b.bit_start),
                        &mut scratch,
                        &mut syms,
                        cfg,
                    )
                    .unwrap();
                    acc += syms.len() + (r_end.bit_pos() as usize & 7);
                }
            }
            black_box(acc);
            full_cpu[ci].push((thread_cpu_ns().saturating_sub(c0)) as f64 / 1e6 * fscale);
            full_wall[ci].push(t0.elapsed().as_secs_f64() * 1e3 * fscale);
        }
    }

    println!("  --- Joint End-to-End Post-#1034 Serial Scan S' (A2a..d + A3, ms/pass, {repeats} repeats x {full_iters} iters) ---");
    let (ref_m, ref_0, ref_1) = median_range(&full_cpu[0]);
    let (rw_m, rw_0, rw_1) = median_range(&full_wall[0]);
    println!(
        "    {:<20}: cpu={ref_m:6.2}ms [{ref_0:6.2}..{ref_1:6.2}] | wall={rw_m:6.2}ms [{rw_0:6.2}..{rw_1:6.2}]  (ref)",
        cfgs[0].name
    );
    for (ci, cfg) in cfgs.iter().enumerate().skip(1) {
        let (cm, c0, c1) = median_range(&full_cpu[ci]);
        let (wm, w0, w1) = median_range(&full_wall[ci]);
        let pct = 100.0 * (cm - ref_m) / ref_m;
        let wpct = 100.0 * (wm - rw_m) / rw_m;
        let disj = if c1 < ref_0 {
            "DISJOINT-FAST"
        } else if c0 > ref_1 {
            "DISJOINT-SLOW"
        } else {
            "overlap"
        };
        println!(
            "    {:<20}: cpu={cm:6.2}ms [{c0:6.2}..{c1:6.2}] ({pct:+5.2}%, {disj}) | wall={wm:6.2}ms [{w0:6.2}..{w1:6.2}] ({wpct:+5.2}%)",
            cfg.name
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repeats = 7usize;
    let mut paths = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().unwrap();
            i += 2;
        } else {
            paths.push(args[i].clone());
            i += 1;
        }
    }
    if paths.is_empty() {
        eprintln!("Usage: scanner_a2_dissection <wikidata.bz2> [--repeats 7] [extra.bz2...]");
        std::process::exit(1);
    }

    println!("=== Serial Scanner A2 (A2a..d) Cold Dissection & Redesign Probe ===");
    println!(
        "cfg(target_feature=avx2)={} | cfg(target_feature=bmi2)={} | repeats={repeats}",
        cfg!(target_feature = "avx2"),
        cfg!(target_feature = "bmi2")
    );
    verify_synthetic_edge_cases();

    for p in &paths {
        let name = Path::new(p)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("corpus");
        let data = fs::read(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
        let locs = load_corpus_blocks(&data);
        print_census(name, &data, &locs);
        bench_corpus(name, &data, &locs, repeats);
    }
}
