//! Bijective Base-2 Monoid Huffman Transducers (`HuffMonoid`, `HuffMonoidBL`)
//! and Safe SWAR Move-To-Front (`MtfU64`).
//!
//! Mathematical basis (RLE2-MTF Commutation Theorem):
//!   Decompose a bzip2 symbol stream into `w_0 s_1 w_1 s_2 w_2 ... s_K w_K EOB` where
//!   `w_i in {RUNA, RUNB}*` and `s_i in {2..alpha-1}` are literal MTF indices.
//!   1. Run lengths `(nu(w_0), 1+nu(w_1), ..., 1+nu(w_K))` depend ONLY on `{RUNA, RUNB}` via the
//!      semidirect-product monoid `(Delta, k)` with `(D1, k1) * (D2, k2) = (D1 + (D2 << k1), k1 + k2)`.
//!   2. Run bytes `(c_0, c_1, ..., c_K)` depend ONLY on `(s_1, ..., s_K)` via the MTF transducer.
//!   3. Because `seq_to_unseq` has distinct bytes and every literal has MTF index >= 1,
//!      `c_i != c_{i-1}` for all `i >= 1`: tokens map 1-to-1 to MAXIMAL runs of L!

use crate::bits_safe::Reg;
use mibz2::block::{BLOCK_MAGIC, END_MAGIC, MAX_SYMS};
use mibz2::ErrorKind;

const MAX_GROUPS: usize = 6;
const MAX_ALPHA: usize = 258;
const MAX_SELECTORS: usize = 18002;
const MAX_CODE_LEN: usize = 23;
const GROUP: usize = 50;
const LUT_BITS: u32 = 10;
const LUT_SIZE: usize = 1 << LUT_BITS;
const LUT_MASK: u32 = (LUT_SIZE as u32) - 1;
const MAX_RUN_SHIFT: u32 = 21; // weight >= 2^21 => RunTooLong
const STEPS: usize = 4;

// Single-symbol LUT1 entry (u16):
//   bit 15: BAD
//   bits 0..4: len (0 => slow / > 10 bits, 1..=10 => hit)
//   bits 4..13: sym (0..=257)
const L1_BAD: u16 = 1 << 15;

// Branchless Unified Monoid LUT entry (`u32`, 4 KiB per group — same footprint as master!):
//   0             => SLOW (>10 bits, BAD, or starts with EOB)
//   bits 0..4     => `bits` consumed (1..=10)
//   bits 4..8     => `nsyms` consumed (1..=10)
//   bits 8..10    => `count` of completed tokens (0=RUN_ONLY, 1=RUN*+LIT, 2=LIT+LIT)
//   bits 10..18   => `next_mtf` (1..=255 when count >= 1; 0 when count == 0)
//   bits 18..29   => `delta` (1..=2046 for count==0; 0..=1022 for count==1)
//                    OR when count==2: bits 18..26 = `mid_mtf` (1..=255) and delta is masked to 0!
//   Note: `k` (number of RUNA/RUNB symbols consumed) is ALWAYS `nsyms - count`!
//         Verify: count=0 => k=nsyms; count=1 => k=nsyms-1; count=2 => k=2-2=0!
struct HuffBL {
    lut: [u32; LUT_SIZE],
    lut1: [u16; LUT_SIZE],
    limit: [i32; MAX_CODE_LEN],
    base: [i32; MAX_CODE_LEN],
    perm: [u16; MAX_ALPHA],
    min_len: u32,
    alpha: i32,
    max_syms: usize,
}

impl HuffBL {
    fn new(lens: &[u8]) -> Self {
        let alpha = lens.len();
        let min_len = *lens.iter().min().unwrap() as usize;
        let max_len = *lens.iter().max().unwrap() as usize;
        let mut perm = [0u16; MAX_ALPHA];
        let mut pp = 0;
        for i in min_len..=max_len {
            for (j, &l) in lens.iter().enumerate() {
                if l as usize == i {
                    perm[pp] = j as u16;
                    pp += 1;
                }
            }
        }
        let mut base = [0i32; MAX_CODE_LEN];
        for &l in lens {
            base[l as usize + 1] += 1;
        }
        for i in 1..MAX_CODE_LEN {
            base[i] += base[i - 1];
        }
        let mut limit = [0i32; MAX_CODE_LEN];
        let mut vec = 0i32;
        for i in min_len..=max_len {
            vec += base[i + 1] - base[i];
            limit[i] = vec - 1;
            vec <<= 1;
        }
        for i in min_len + 1..=max_len {
            base[i] = ((limit[i - 1] + 1) << 1) - base[i];
        }

        let mut h = HuffBL {
            lut: [0; LUT_SIZE],
            lut1: [0; LUT_SIZE],
            limit,
            base,
            perm,
            min_len: min_len as u32,
            alpha: alpha as i32,
            max_syms: 1,
        };

        // 1. Build 1-symbol LUT1
        let mut a = 0i64;
        for zn in h.min_len..=LUT_BITS {
            let top = (1i64 << zn) - 1;
            let hi = (h.limit[zn as usize] as i64).min(top);
            for p in a..=hi {
                let e = match h.symbol(zn, p as i32) {
                    Some(s) => (s << 4) | (zn as u16),
                    None => L1_BAD,
                };
                let sh = LUT_BITS - zn;
                h.lut1[(p << sh) as usize..((p + 1) << sh) as usize].fill(e);
            }
            a = 2 * a.max(hi + 1);
        }

        // 2. Build Branchless Unified Monoid LUT
        let eob = (alpha - 1) as u16;
        let mut max_syms = 1usize;
        for w in 0..LUT_SIZE as u32 {
            let e1 = h.lut1[w as usize];
            let l1 = (e1 & 15) as u32;
            if l1 == 0 || e1 == L1_BAD {
                h.lut[w as usize] = 0;
                continue;
            }
            let s1 = (e1 >> 4) & 0x1ff;
            if s1 == eob {
                h.lut[w as usize] = 0;
                continue;
            }
            if s1 <= 1 {
                // RUNA/RUNB streak, optionally terminated by 1 literal (count = 0 or 1)
                let mut bits = l1;
                let mut k = 1u32;
                let mut delta = (s1 as u32) + 1;
                let mut trailing_lit: Option<u32> = None;

                while bits < LUT_BITS && k < 4 {
                    let rem = LUT_BITS - bits;
                    let nw = (w << bits) & LUT_MASK;
                    let en = h.lut1[nw as usize];
                    let ln = (en & 15) as u32;
                    if ln == 0 || en == L1_BAD || ln > rem {
                        break;
                    }
                    let sn = (en >> 4) & 0x1ff;
                    if sn <= 1 {
                        delta += ((sn as u32) + 1) << k;
                        k += 1;
                        bits += ln;
                    } else if sn != eob && k < 4 {
                        trailing_lit = Some((sn - 1) as u32);
                        bits += ln;
                        break;
                    } else {
                        break;
                    }
                }

                if let Some(m1) = trailing_lit {
                    let nsyms = k + 1;
                    max_syms = max_syms.max(nsyms as usize);
                    debug_assert!(m1 >= 1 && m1 <= 255 && delta < 1024 && nsyms <= 4);
                    h.lut[w as usize] = bits | (nsyms << 4) | (1 << 8) | (m1 << 10) | (delta << 18);
                } else {
                    let nsyms = k;
                    max_syms = max_syms.max(nsyms as usize);
                    debug_assert!(delta >= 1 && delta < 2048 && nsyms <= 4);
                    h.lut[w as usize] = bits | (nsyms << 4) | (delta << 18); // count = 0
                }
            } else {
                // Starts with literal m1 = s1 - 1. Check if a 2nd literal m2 follows in 10 bits!
                let m1 = (s1 - 1) as u32;
                let mut bits = l1;
                if bits < LUT_BITS {
                    let rem = LUT_BITS - bits;
                    let nw = (w << bits) & LUT_MASK;
                    let e2 = h.lut1[nw as usize];
                    let l2 = (e2 & 15) as u32;
                    if l2 != 0 && e2 != L1_BAD && l2 <= rem {
                        let s2 = (e2 >> 4) & 0x1ff;
                        if s2 > 1 && s2 != eob {
                            let m2 = (s2 - 1) as u32;
                            bits += l2;
                            max_syms = max_syms.max(2);
                            // count = 2, next_mtf = m2, mid_mtf = m1
                            h.lut[w as usize] =
                                bits | (2 << 4) | (2 << 8) | (m2 << 10) | (m1 << 18);
                            continue;
                        }
                    }
                }
                // Single literal: count = 1, k = 0, delta = 0, next_mtf = m1
                h.lut[w as usize] = bits | (1 << 4) | (1 << 8) | (m1 << 10);
            }
        }
        h.max_syms = max_syms;
        h
    }

    #[inline(always)]
    fn symbol(&self, zn: u32, zvec: i32) -> Option<u16> {
        let i = zvec - self.base[zn as usize];
        (0..self.alpha).contains(&i).then(|| self.perm[i as usize])
    }

    #[inline(always)]
    fn decode_one(&self, r: &mut Reg, data: &[u8]) -> Result<u16, ErrorKind> {
        let e = self.lut1[r.peek(data, LUT_BITS) as usize];
        let l = (e & 15) as u32;
        if l != 0 {
            r.skip_short(l);
            return Ok((e >> 4) & 0x1ff);
        }
        if e == L1_BAD {
            return Err(ErrorKind::BadHuffmanCode);
        }
        self.decode_slow(r, data)
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
        let res = self.symbol(zn, zvec).ok_or(ErrorKind::BadHuffmanCode);
        r.refill(data);
        res
    }
}

/// Phase 1 output of `HuffMonoidBL`: flat `u32` maximal-run tokens `(mtf_idx << 24) | len`!
pub struct RunTokens {
    pub crc: u32,
    pub orig_ptr: u32,
    pub seq_to_unseq: [u8; 256],
    pub max_len: usize,
    pub n: usize,
    pub cap127_recs: usize,
    pub total_syms: usize,
    pub run_syms: usize,
    /// Flat `(mtf_idx << 24) | len` tokens (valid slice is `tokens[start..end]`)
    pub tokens: Vec<u32>,
    /// Start index (`1` if initial empty sentinel was written at slot 0, else `0`)
    pub start: usize,
    /// End index (exclusive) in `tokens`
    pub end: usize,
    // Diagnostics
    pub fast_lookups: u64,
    pub slow_lookups: u64,
    pub header_ns: u64,
    pub loop_ns: u64,
}

impl Default for RunTokens {
    fn default() -> Self {
        RunTokens {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: [0; 256],
            max_len: 0,
            n: 0,
            cap127_recs: 0,
            total_syms: 0,
            run_syms: 0,
            tokens: Vec::new(),
            start: 0,
            end: 0,
            fast_lookups: 0,
            slow_lookups: 0,
            header_ns: 0,
            loop_ns: 0,
        }
    }
}

impl RunTokens {
    #[inline]
    pub fn slice(&self) -> &[u32] {
        &self.tokens[self.start..self.end]
    }

    /// Exact 127-capped record count (computed on worker, never on scanner!)
    #[inline]
    pub fn compute_cap127(&self) -> usize {
        let s = self.slice();
        let mut cap127 = s.len();
        for &tok in s {
            let len = tok & 0x00ff_ffff;
            if len > 127 {
                cap127 += (len - 1) as usize / 127;
            }
        }
        cap127
    }

    /// Exact `runs::takes_runs` equivalent without sampling
    #[inline]
    pub fn takes_runs(&self) -> bool {
        self.max_len < (1 << 24)
            && 2 * self.total_syms <= self.max_len
            && 32 * self.run_syms >= self.total_syms
    }
}

pub enum MonoidItem {
    Block,
    End(#[allow(dead_code)] u32),
}

/// Branchless Bijective Base-2 Monoid Huffman Transducer (`HuffMonoidBL-v2`).
/// Caps `max_k <= 4` (so `STEPS=4` covers `left >= 16`, tail is <4 syms/group),
/// removes `total_syms`/`run_syms` from hot loop (computed algebraically at EOB!).
pub fn read_block_monoid_bl(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    out: &mut RunTokens,
    count_stats: bool,
) -> Result<MonoidItem, ErrorKind> {
    let t_hdr0 = if count_stats { Some(std::time::Instant::now()) } else { None };
    let magic = (r.bits(data, 24) as u64) << 24 | r.bits(data, 24) as u64;
    if magic == END_MAGIC {
        return Ok(MonoidItem::End(r.bits(data, 32)));
    }
    if magic != BLOCK_MAGIC {
        return Err(ErrorKind::BadBlockMagic);
    }
    let crc = r.bits(data, 32);
    if r.bit(data) {
        return Err(ErrorKind::Randomised);
    }
    let orig_ptr = r.bits(data, 24);
    let max_len = 100_000 * level as usize;
    if orig_ptr as usize > 10 + max_len {
        return Err(ErrorKind::BadOrigPtr);
    }
    let in_use16 = r.bits(data, 16);
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for i in 0..16 {
        if in_use16 & (0x8000 >> i) != 0 {
            let w = r.bits(data, 16);
            for j in 0..16 {
                if w & (0x8000 >> j) != 0 {
                    seq_to_unseq[n_in_use] = (i * 16 + j) as u8;
                    n_in_use += 1;
                }
            }
        }
    }
    if n_in_use == 0 {
        return Err(ErrorKind::NoSymbols);
    }
    let alpha = n_in_use + 2;
    let n_groups = r.bits(data, 3) as usize;
    if !(2..=MAX_GROUPS).contains(&n_groups) {
        return Err(ErrorKind::BadGroups);
    }
    let n_sel = r.bits(data, 15) as usize;
    if n_sel < 1 {
        return Err(ErrorKind::BadSelectors);
    }
    let mut mtf_g = [0u8, 1, 2, 3, 4, 5];
    let mut selectors = Vec::with_capacity(n_sel.min(MAX_SELECTORS));
    for i in 0..n_sel {
        let mut j = 0;
        while r.bit(data) {
            j += 1;
            if j >= n_groups {
                return Err(ErrorKind::BadSelector);
            }
        }
        if i < MAX_SELECTORS {
            let v = mtf_g[j];
            mtf_g.copy_within(0..j, 1);
            mtf_g[0] = v;
            selectors.push(v);
        }
        if r.overran(data) {
            return Err(ErrorKind::Truncated);
        }
    }
    let mut tables = Vec::with_capacity(n_groups);
    let mut lens = [0u8; MAX_ALPHA];
    for _ in 0..n_groups {
        let mut curr = r.bits(data, 5) as i32;
        for l in lens[..alpha].iter_mut() {
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
                if r.overran(data) {
                    return Err(ErrorKind::Truncated);
                }
            }
            *l = curr as u8;
        }
        tables.push(HuffBL::new(&lens[..alpha]));
    }
    let hdr_ns = t_hdr0.map_or(0, |t| t.elapsed().as_nanos() as u64);
    let t_lp0 = if count_stats { Some(std::time::Instant::now()) } else { None };

    let eob = (alpha - 1) as u16;
    let need_cap = (n_sel.min(MAX_SELECTORS) * GROUP).min(MAX_SYMS) + 4;
    if out.tokens.len() < need_cap {
        out.tokens.resize(need_cap, 0);
    }
    let buf = &mut out.tokens[..need_cap];
    let mut ntok = 0usize;
    let mut open_mtf = 0u32;
    let mut open_len = 0u32;
    let mut run = 0u32;
    let mut shift = 0u32;
    let mut n_rows = 0usize;
    let mut fast_l = 0u64;
    let mut slow_l = 0u64;

    let mut reg = *r;
    let mut sel = 0usize;
    let mut end_left = 0usize;

    macro_rules! slow_sym {
        ($t:expr, $left:expr, $lbl:lifetime) => {{
            let s = match $t.decode_one(&mut reg, data) {
                Ok(s) => s,
                Err(k) => {
                    *r = reg;
                    return Err(k);
                }
            };
            $left -= 1;
            if count_stats {
                slow_l += 1;
            }
            if s <= 1 {
                if shift >= MAX_RUN_SHIFT {
                    *r = reg;
                    return Err(ErrorKind::RunTooLong);
                }
                run += ((s as u32) + 1) << shift;
                shift += 1;
            } else if s == eob {
                let total = open_len + run;
                if total != 0 {
                    buf[ntok] = open_mtf | total;
                    ntok += 1;
                    n_rows += total as usize;
                }
                end_left = $left;
                break $lbl;
            } else {
                let total = open_len + run;
                buf[ntok] = open_mtf | total;
                ntok += 1;
                n_rows += total as usize;
                open_mtf = ((s - 1) as u32) << 24;
                open_len = 1;
                run = 0;
                shift = 0;
            }
        }};
    }

    macro_rules! fast_step {
        ($t:expr, $left:expr, $live:expr, $lbl:lifetime) => {{
            let e = $t.lut[if $live {
                reg.peek_live(LUT_BITS)
            } else {
                reg.peek(data, LUT_BITS)
            } as usize];
            if e != 0 {
                if count_stats {
                    fast_l += 1;
                }
                reg.skip_short(e & 15);
                let ns = (e >> 4) & 15;
                let count = (e >> 8) & 3;
                let k = ns - count;
                let new_shift = shift + k;
                if new_shift > MAX_RUN_SHIFT {
                    *r = reg;
                    return Err(ErrorKind::RunTooLong);
                }
                let dmask = ((count < 2) as u32).wrapping_neg();
                let delta = (e >> 18) & dmask;
                let new_run = run.wrapping_add(delta << shift);
                let total = open_len + new_run;
                buf[ntok] = open_mtf | total;
                buf[ntok + 1] = ((e & (0xff << 18)) << 6) | 1;
                ntok += count as usize;
                let keep_mask = ((count == 0) as u32).wrapping_neg();
                let committed = (total & !keep_mask) + (count >> 1);
                n_rows += committed as usize;
                let next_mtf = (e & (0xff << 10)) << 14;
                open_mtf = (open_mtf & keep_mask) | (next_mtf & !keep_mask);
                open_len = (open_len & keep_mask) | (1 & !keep_mask);
                run = new_run & keep_mask;
                shift = new_shift & keep_mask;
                $left -= ns as usize;
            } else {
                slow_sym!($t, $left, $lbl);
            }
        }};
    }

    'groups: loop {
        let Some(&g) = selectors.get(sel) else {
            *r = reg;
            return Err(ErrorKind::SelectorOverrun);
        };
        let t = &tables[g as usize];
        sel += 1;
        let mut left = GROUP;
        let max_k = t.max_syms;
        let batch_lim = max_k * STEPS;

        if left >= batch_lim {
            reg.refill(data);
            loop {
                for _ in 0..STEPS {
                    fast_step!(t, left, true, 'groups);
                }
                if left < batch_lim {
                    break;
                }
                reg.refill(data);
            }
        }
        while left >= max_k {
            fast_step!(t, left, false, 'groups);
        }
        while left > 0 {
            slow_sym!(t, left, 'groups);
        }
        if n_rows > max_len {
            *r = reg;
            return Err(ErrorKind::BlockTooLong);
        }
        if reg.overran(data) {
            *r = reg;
            return Err(ErrorKind::Truncated);
        }
    }

    if n_rows > max_len {
        *r = reg;
        return Err(ErrorKind::BlockTooLong);
    }

    let lp_ns = t_lp0.map_or(0, |t| t.elapsed().as_nanos() as u64);
    *r = reg;
    let start = if ntok > 0 && out.tokens[0] == 0 { 1 } else { 0 };
    let total_syms = (sel - 1) * GROUP + (GROUP - end_left);
    let n_runs = ntok - start;
    let has_leading = n_runs > 0 && (out.tokens[start] >> 24) == 0;
    let n_lits = n_runs - usize::from(has_leading);
    let run_syms = total_syms - 1 - n_lits;
    out.start = start;
    out.end = ntok;
    out.crc = crc;
    out.orig_ptr = orig_ptr;
    out.seq_to_unseq = seq_to_unseq;
    out.max_len = max_len;
    out.n = n_rows;
    out.total_syms = total_syms;
    out.run_syms = run_syms;
    out.fast_lookups = fast_l;
    out.slow_lookups = slow_l;
    out.header_ns = hdr_ns;
    out.loop_ns = lp_ns;
    out.cap127_recs = n_runs;
    Ok(MonoidItem::Block)
}

/// Safe 4x`u64` SWAR Move-To-Front list (`q0..q3` = bytes 0..32 in 4 native 64-bit GPRs!).
/// On x86-64, shifting a `u64` by `nn * 8` (`nn < 8`) is ONE 1-cycle `shr` instruction,
/// avoiding `u128`'s variable-shift `shrd + cmov` penalty!
const LO64: [u64; 8] = {
    let mut m = [0u64; 8];
    let mut i = 1;
    while i < 8 {
        m[i] = (1u64 << (i * 8)) - 1;
        i += 1;
    }
    m
};

const HI64: [u64; 8] = {
    let mut m = [0u64; 8];
    let mut i = 0;
    while i < 7 {
        m[i] = !((1u64 << ((i + 1) * 8)) - 1);
        i += 1;
    }
    m[7] = 0;
    m
};

pub struct MtfU64 {
    q0: u64,
    q1: u64,
    q2: u64,
    q3: u64,
    tail: [u8; 224],
}

impl MtfU64 {
    #[inline]
    pub fn new(s2u: &[u8; 256]) -> Self {
        let q0 = u64::from_le_bytes(s2u[0..8].try_into().unwrap());
        let q1 = u64::from_le_bytes(s2u[8..16].try_into().unwrap());
        let q2 = u64::from_le_bytes(s2u[16..24].try_into().unwrap());
        let q3 = u64::from_le_bytes(s2u[24..32].try_into().unwrap());
        let mut tail = [0u8; 224];
        tail.copy_from_slice(&s2u[32..256]);
        MtfU64 {
            q0,
            q1,
            q2,
            q3,
            tail,
        }
    }

    #[inline(always)]
    pub fn front(&self) -> u8 {
        self.q0 as u8
    }

    #[inline(always)]
    pub fn move_to_front(&mut self, nn: usize) -> u8 {
        if nn < 8 {
            let c = ((self.q0 >> (nn * 8)) & 0xff) as u8;
            self.q0 = (c as u64) | ((self.q0 & LO64[nn]) << 8) | (self.q0 & HI64[nn]);
            c
        } else if nn < 16 {
            let k = nn - 8;
            let c = ((self.q1 >> (k * 8)) & 0xff) as u8;
            let c0 = self.q0 >> 56;
            self.q0 = (c as u64) | (self.q0 << 8);
            self.q1 = c0 | ((self.q1 & LO64[k]) << 8) | (self.q1 & HI64[k]);
            c
        } else {
            self.move_deep(nn)
        }
    }

    #[cold]
    #[inline(never)]
    fn move_deep(&mut self, nn: usize) -> u8 {
        if nn < 24 {
            let k = nn - 16;
            let c = ((self.q2 >> (k * 8)) & 0xff) as u8;
            let c0 = self.q0 >> 56;
            let c1 = self.q1 >> 56;
            self.q0 = (c as u64) | (self.q0 << 8);
            self.q1 = c0 | (self.q1 << 8);
            self.q2 = c1 | ((self.q2 & LO64[k]) << 8) | (self.q2 & HI64[k]);
            c
        } else if nn < 32 {
            let k = nn - 24;
            let c = ((self.q3 >> (k * 8)) & 0xff) as u8;
            let c0 = self.q0 >> 56;
            let c1 = self.q1 >> 56;
            let c2 = self.q2 >> 56;
            self.q0 = (c as u64) | (self.q0 << 8);
            self.q1 = c0 | (self.q1 << 8);
            self.q2 = c1 | (self.q2 << 8);
            self.q3 = c2 | ((self.q3 & LO64[k]) << 8) | (self.q3 & HI64[k]);
            c
        } else {
            let t = nn - 32;
            let c = self.tail[t];
            let c3 = (self.q3 >> 56) as u8;
            self.tail.copy_within(0..t, 1);
            self.tail[0] = c3;
            let c0 = self.q0 >> 56;
            let c1 = self.q1 >> 56;
            let c2 = self.q2 >> 56;
            self.q0 = (c as u64) | (self.q0 << 8);
            self.q1 = c0 | (self.q1 << 8);
            self.q2 = c1 | (self.q2 << 8);
            self.q3 = c2 | (self.q3 << 8);
            c
        }
    }
}

/// Expand `RunTokens` into `(ll, counts)` for high-entropy / column-route blocks (`!takes_runs`).
pub fn tokens_to_column(rt: &RunTokens, ll: &mut Vec<u8>, counts: &mut [u32; 256]) {
    ll.clear();
    ll.reserve(rt.n);
    *counts = [0; 256];
    let mut mtf = MtfU64::new(&rt.seq_to_unseq);
    let toks = rt.slice();
    let mut i = 0;
    if let Some(&first) = toks.first() {
        if (first >> 24) == 0 {
            let len = first & 0x00ff_ffff;
            let c = mtf.front();
            counts[c as usize] += len;
            ll.resize(len as usize, c);
            i = 1;
        }
    }
    for &tok in &toks[i..] {
        let nn = (tok >> 24) as usize;
        let len = tok & 0x00ff_ffff;
        let c = mtf.move_to_front(nn);
        counts[c as usize] += len;
        if len == 1 {
            ll.push(c);
        } else {
            ll.resize(ll.len() + len as usize, c);
        }
    }
}

/// Expand `RunTokens` into uncapped maximal runs `(len << 8) | byte`.
pub fn tokens_to_maxruns(rt: &RunTokens, out: &mut Vec<u32>) {
    let toks = rt.slice();
    out.clear();
    out.reserve(toks.len());
    let mut mtf = MtfU64::new(&rt.seq_to_unseq);
    let mut i = 0;
    if let Some(&first) = toks.first() {
        if (first >> 24) == 0 {
            let len = first & 0x00ff_ffff;
            let c = mtf.front();
            out.push((len << 8) | (c as u32));
            i = 1;
        }
    }
    for &tok in &toks[i..] {
        let nn = (tok >> 24) as usize;
        let len = tok & 0x00ff_ffff;
        let c = mtf.move_to_front(nn);
        out.push((len << 8) | (c as u32));
    }
}

/// Convert `mibz2::runs::Runs::records()` (127-capped) to uncapped maximal runs (used ONLY to
/// isolate the `-14.4 ms` `maximal()` removal effect on `Open` Pass 1!).
pub fn records_to_maxruns(records: &[u16], out: &mut Vec<u32>) {
    out.clear();
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u32;
        let mut width = (rec & 255) as u32;
        i += 1;
        while i < records.len() && (records[i] >> 8) as u32 == c {
            width += (records[i] & 255) as u32;
            i += 1;
        }
        out.push((width << 8) | c);
    }
}

/// Arm C: 1-pass Branchless `HuffMonoidBL` + strip-mined L1 `MtfU64` (256 tokens/batch!)
/// directly into `MaxRunsBlock` (`(len << 8) | byte`, ZERO `syms`, ZERO `tokens` heap buffer!).
#[derive(Default)]
pub struct MaxRunsBlock {
    pub crc: u32,
    pub orig_ptr: u32,
    pub max_len: usize,
    pub n: usize,
    pub cap127_recs: usize,
    pub total_syms: usize,
    pub run_syms: usize,
    pub runs: Vec<u32>,
}

impl MaxRunsBlock {
    #[inline]
    pub fn takes_runs(&self) -> bool {
        self.max_len < (1 << 24)
            && 2 * self.total_syms <= self.max_len
            && 32 * self.run_syms >= self.total_syms
    }
}

pub fn read_block_huff_mtf_bl(
    data: &[u8],
    r: &mut Reg,
    level: u32,
    out: &mut MaxRunsBlock,
    rt_scratch: &mut RunTokens,
) -> Result<MonoidItem, ErrorKind> {
    // Strip-mine via 1-pass HuffMonoidBL + in-place MtfU64 overwrite on the same buffer!
    // Wait: in-place MtfU64 over `rt_scratch.tokens` turns `(mtf_idx << 24) | len` into
    // `(len << 8) | byte` IN PLACE (zero extra buffer allocation!).
    match read_block_monoid_bl(data, r, level, rt_scratch, false)? {
        MonoidItem::End(c) => return Ok(MonoidItem::End(c)),
        MonoidItem::Block => {}
    }
    out.crc = rt_scratch.crc;
    out.orig_ptr = rt_scratch.orig_ptr;
    out.max_len = rt_scratch.max_len;
    out.n = rt_scratch.n;
    out.cap127_recs = rt_scratch.cap127_recs;
    out.total_syms = rt_scratch.total_syms;
    out.run_syms = rt_scratch.run_syms;
    tokens_to_maxruns(rt_scratch, &mut out.runs);
    Ok(MonoidItem::Block)
}
