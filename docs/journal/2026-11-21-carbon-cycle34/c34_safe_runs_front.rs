#![forbid(unsafe_code)]
//! Carbon Cycle 34 — Safe-Rust (`#![forbid(unsafe_code)]`) SWAR-Bitmap + GP `(lo, hi)` 2-Level MTF `runs::front`
//! confronted directly against `origin/master` (`53f7ecf9`)'s `unsafe` AVX2+BMI2 `front_avx2` and `front_scalar`.
//!
//! Result on `wikidata.bz2` (`383` run-path blocks, 15 rotated rounds, `431/431` record-exact):
//! - `MasterA` (`front_avx2`): `42.99 ms` (`A/A = 0.9967x`)
//! - `Master` (`front_scalar`): `271.93 ms`
//! - `C34 SafeRuns` (`#![forbid(unsafe_code)]`): `198.25 ms` (`0.7290x` vs `front_scalar`, `4.6113x` vs `front_avx2`)
//! Verdict: KILLED immediately as a `runs::front` replacement (safe Rust lacks AVX2 `pshufb` 32-byte register shuffles).

use mibz2::block::Symbols;
use mibz2::ErrorKind;

const CAP: u32 = 127;
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

struct SlidingTail {
    mtfa: [u8; 4096],
    base: [usize; 16],
}

impl Default for SlidingTail {
    fn default() -> Self {
        Self {
            mtfa: [0u8; 4096],
            base: [0usize; 16],
        }
    }
}

impl SlidingTail {
    #[inline(always)]
    fn reset_from_s2u(&mut self, s2u: &[u8; 256]) {
        self.reset_from_slice(&s2u[16..256]);
    }

    fn reset_from_slice(&mut self, tail240: &[u8]) {
        let mut kk = 4096usize;
        for b in (1..16).rev() {
            kk -= 16;
            let src_off = (b - 1) * 16;
            self.mtfa[kk..kk + 16].copy_from_slice(&tail240[src_off..src_off + 16]);
            self.base[b] = kk;
        }
    }

    #[cold]
    #[inline(never)]
    fn move_ge16(&mut self, nn: usize, carry_from_f0: u8) -> u8 {
        let lno = nn >> 4;
        let off = nn & 15;
        let b_lno = self.base[lno];
        let v = self.mtfa[b_lno + off];
        self.mtfa.copy_within(b_lno..b_lno + off, b_lno + 1);
        if lno == 1 {
            self.mtfa[b_lno] = carry_from_f0;
        } else {
            self.mtfa[b_lno] = self.mtfa[self.base[lno - 1] + 15];
            let mut k = lno - 1;
            while k > 1 {
                self.base[k] -= 1;
                self.mtfa[self.base[k]] = self.mtfa[self.base[k - 1] + 15];
                k -= 1;
            }
            self.base[1] -= 1;
            self.mtfa[self.base[1]] = carry_from_f0;
            if self.base[1] == 0 {
                let mut flat = [0u8; 240];
                for b in 1..16 {
                    let st = self.base[b];
                    flat[(b - 1) * 16..b * 16].copy_from_slice(&self.mtfa[st..st + 16]);
                }
                self.reset_from_slice(&flat);
            }
        }
        v
    }
}

pub struct SafeRuns {
    pub runs: Vec<u16>,
    pub counts: [u32; 256],
    lit: Vec<u64>,
    one: Vec<u64>,
    tail: Box<SlidingTail>,
}

impl Default for SafeRuns {
    fn default() -> Self {
        Self {
            runs: Vec::new(),
            counts: [0; 256],
            lit: Vec::new(),
            one: Vec::new(),
            tail: Box::new(SlidingTail::default()),
        }
    }
}

#[inline(always)]
fn pack4_u16_gt1_and_ne0(s0: u16, s1: u16, s2: u16, s3: u16) -> (u64, u64) {
    let w = (s0 as u64)
        | ((s1 as u64) << 16)
        | ((s2 as u64) << 32)
        | ((s3 as u64) << 48);
    let u_gt1 = (w >> 1) & 0x7fff_7fff_7fff_7fff;
    let nz_gt1 = (u_gt1 + 0x7fff_7fff_7fff_7fff) & 0x8000_8000_8000_8000;
    let m_gt1 = ((nz_gt1 >> 15).wrapping_mul(0x0001_0002_0004_0008)) >> 48;

    let u_ne0 = (w | (w >> 1)) & 0x7fff_7fff_7fff_7fff;
    let nz_ne0 = (u_ne0 + 0x7fff_7fff_7fff_7fff) & 0x8000_8000_8000_8000;
    let m_ne0 = ((nz_ne0 >> 15).wrapping_mul(0x0001_0002_0004_0008)) >> 48;

    (m_gt1 & 0xf, m_ne0 & 0xf)
}

pub fn front_c34(b: &Symbols, r: &mut SafeRuns) -> Result<usize, ErrorKind> {
    if let Some(n) = front_c34_fast(b, r) {
        return Ok(n);
    }
    front_c34_scalar(b, r)
}

fn front_c34_fast(b: &Symbols, f: &mut SafeRuns) -> Option<usize> {
    let syms = &b.syms;
    let m = syms.len();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return None;
    }

    let words = m / 64 + 3;
    if f.lit.len() < words {
        f.lit.resize(words, 0);
    }
    if f.one.len() < words {
        f.one.resize(words, 0);
    }
    let lit = &mut f.lit[..words];
    let one = &mut f.one[..words];

    let eob = b.eob;
    let ve = (eob as u64) * 0x0001_0001_0001_0001;
    let mut end = usize::MAX;
    let full_words = m / 64;

    for wi in 0..full_words {
        let ch = &syms[wi * 64..wi * 64 + 64];
        let mut lw = 0u64;
        let mut ow = 0u64;
        let mut eob_hit = false;
        for q in 0..16 {
            let base = q * 4;
            let (s0, s1, s2, s3) = (ch[base], ch[base + 1], ch[base + 2], ch[base + 3]);
            let (ml, mo) = pack4_u16_gt1_and_ne0(s0, s1, s2, s3);
            lw |= ml << (q * 4);
            ow |= mo << (q * 4);
            let w = (s0 as u64) | ((s1 as u64) << 16) | ((s2 as u64) << 32) | ((s3 as u64) << 48);
            let x = w ^ ve;
            if (!(((x & 0x7fff_7fff_7fff_7fff) + 0x7fff_7fff_7fff_7fff) | x)
                & 0x8000_8000_8000_8000)
                != 0
            {
                eob_hit = true;
            }
        }
        if eob_hit && eob > 1 {
            for t in 0..64 {
                if ch[t] == eob {
                    end = wi * 64 + t;
                    let keep = if t == 63 { !0u64 } else { (2u64 << t) - 1 };
                    lw &= keep >> 1;
                    ow &= keep;
                    break;
                }
            }
        }
        lit[wi] = lw;
        one[wi] = ow;
        if end != usize::MAX {
            for z in wi + 1..words {
                lit[z] = 0;
                one[z] = 0;
            }
            break;
        }
    }

    if end == usize::MAX {
        let wi = full_words;
        let (mut lw, mut ow) = (0u64, 0u64);
        for (t, &s) in syms[wi * 64..].iter().enumerate() {
            ow |= ((s != 0) as u64) << t;
            if s > 1 {
                if s == eob {
                    end = wi * 64 + t;
                    break;
                }
                lw |= 1u64 << t;
            }
        }
        lit[wi] = lw;
        one[wi] = ow;
        for z in wi + 1..words {
            lit[z] = 0;
            one[z] = 0;
        }
    }
    if end == usize::MAX {
        return None;
    }

    let s2u = &b.seq_to_unseq;
    let mut lo = u64::from_le_bytes([
        s2u[0], s2u[1], s2u[2], s2u[3], s2u[4], s2u[5], s2u[6], s2u[7],
    ]);
    let mut hi = u64::from_le_bytes([
        s2u[8], s2u[9], s2u[10], s2u[11], s2u[12], s2u[13], s2u[14], s2u[15],
    ]);
    f.tail.reset_from_s2u(s2u);
    let tail = &mut *f.tail;

    let mut slack = b.max_len / (CAP as usize) + 1;
    let need_cap = m + slack + 4;
    if f.runs.len() < need_cap {
        f.runs.resize(need_cap, 0);
    }
    let out = &mut f.runs[..need_cap];
    let mut out_pos = 0usize;
    f.counts = [0u32; 256];
    let counts = &mut f.counts;
    let mut total_rows = 0u64;

    macro_rules! emit_long {
        ($c:expr, $tot:expr) => {{
            let (c, tot): (u8, u64) = ($c, $tot);
            let recs = tot.div_ceil(CAP as u64);
            let more = recs.saturating_sub(1) as usize;
            if more > slack || total_rows + tot > b.max_len as u64 {
                return None;
            }
            slack -= more;
            total_rows += tot;
            counts[c as usize] = counts[c as usize].wrapping_add(tot as u32);
            let hi_c = (c as u16) << 8;
            let mut left = tot;
            while left > 0 {
                let l = left.min(CAP as u64);
                out[out_pos] = hi_c | (l as u16);
                out_pos += 1;
                left -= l;
            }
        }};
    }

    macro_rules! emit_fast {
        ($c:expr, $tot:expr, $span:expr) => {{
            let (c, tot): (u8, u64) = ($c, $tot);
            if tot > CAP as u64 {
                if $span > 22 {
                    return None;
                }
                emit_long!(c, tot);
            } else {
                out[out_pos] = ((c as u16) << 8) | (tot as u16);
                out_pos += 1;
                total_rows += tot;
                counts[c as usize] = counts[c as usize].wrapping_add(tot as u32);
            }
        }};
    }

    #[inline(always)]
    fn extract_one_bits(one: &[u64], q: usize, q2: usize) -> u64 {
        let from = q + 1;
        let wi = from >> 6;
        let bi = from & 63;
        let w0 = one[wi];
        let w1 = one[wi + 1];
        let bits = if bi == 0 {
            w0
        } else {
            (w0 >> bi) | (w1 << (64 - bi))
        };
        let len = q2 - q;
        bits & ((1u64 << len) - 1)
    }

    let last = end / 64;
    let mut wi = 0usize;
    let mut w = lit[0];

    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                w = lit[wi];
            }
        }};
    }

    let first = seek!();
    let q0 = first.map_or(end, |t| wi * 64 + t);
    if q0 > 21 {
        return None;
    }
    let init_run = (one[0] & ((1u64 << (q0 + 1)) - 1)) - 1;
    emit_long!(s2u[0], init_run);

    if let Some(mut tz) = first {
        let mut after = one[wi] >> 1;
        let mut base_sym = wi * 64;
        loop {
            let s = syms[base_sym + tz] as usize;
            let nn = (s - 1).min(255);
            let c = if nn < 8 {
                let sh = (nn * 8) as u32;
                let v = (lo.wrapping_shr(sh) & 0xff) as u8;
                let keep = (!0xffu64).wrapping_shl(sh);
                lo = (lo & keep) | ((lo << 8) & !keep) | (v as u64);
                v
            } else if nn < 16 {
                let sh = ((nn - 8) * 8) as u32;
                let v = (hi.wrapping_shr(sh) & 0xff) as u8;
                let keep = (!0xffu64).wrapping_shl(sh);
                let carry = lo >> 56;
                hi = (hi & keep) | ((hi << 8) & !keep) | carry;
                lo = (lo << 8) | (v as u64);
                v
            } else {
                let v = tail.move_ge16(nn, (hi >> 56) as u8);
                hi = (hi << 8) | (lo >> 56);
                lo = (lo << 8) | (v as u64);
                v
            };

            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let tot = (after & ((1u64 << t2) - 1)) >> tz;
                emit_fast!(c, tot, t2 - tz);
                tz = t2;
            } else {
                let q = wi * 64 + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                if q2 - q > 22 {
                    return None;
                }
                let tot = extract_one_bits(one, q, q2);
                emit_fast!(c, tot, q2 - q);
                match next {
                    Some(t) => {
                        tz = t;
                        after = one[wi] >> 1;
                        base_sym = wi * 64;
                    }
                    None => break,
                }
            }
        }
    }

    if total_rows > b.max_len as u64 {
        return None;
    }
    f.runs.truncate(out_pos);
    Some(total_rows as usize)
}

fn front_c34_scalar(b: &Symbols, r: &mut SafeRuns) -> Result<usize, ErrorKind> {
    fn close(open: (u8, u32), runs: &mut Vec<u16>) {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((c as u16) << 8 | len as u16);
            left -= len;
        }
    }
    r.runs.clear();
    r.counts = [0; 256];
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    let mut open = (0u8, 0u32);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                return Err(ErrorKind::RunTooLong);
            }
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            if n + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            let c = b.seq_to_unseq[list[0] as usize];
            open = (c, open.1 + run);
            r.counts[c as usize] += run;
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = ((s - 1) as usize).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        close(open, &mut r.runs);
        open = (c, 1);
        r.counts[c as usize] += 1;
        n += 1;
    }
    close(open, &mut r.runs);
    Ok(n)
}
