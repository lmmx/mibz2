//! Smooth-zone `explore` probe (zero `src/` edits):
//! Post-#1175 `src/runs.rs` (`runs::front_avx2` + `runs::fill_avx2_trusted`) census,
//! nanosecond phase profile, `objdump -d` audit, and microarchitectural redesign probe across 8
//! `Runs`-routed corpora (`wikidata`, `text`, `mdjournal`, `elf`, `json`, `runs`,
//! `pattern`, `onebyte`), warm & L3-cold, baseline x86-64 & JCC-32B-aligned.
//!
//! # Frozen Discovery / Stop Test (frozen BEFORE reading any timing data)
//! - **Exactness & memory:** 100% bit-exact (`runs`, `counts`, `per` where retained, `n`, `TT`,
//!   and error/bailout parity on synthetic edge cases including tile-boundary runs, `no-EOB`,
//!   `RunTooLong`, `BlockTooLong`, `s >= 256`, empty/single-byte blocks); `<= 1.00x`
//!   `Runs` heap scratch (`0 B` extra heap memory); verified via `objdump -d` (zero hot-loop
//!   `call`, zero hot-loop `vzeroupper`, zero hot-loop XMM/YMM spills).
//! - **PASS** (admit follow-on P2 `idea` claim `runs-front-and-fill-build` on
//!   `owned_paths = ["src/runs.rs"]`): at least one candidate clears BOTH:
//!   1. Cuts `runs::front` CPU by `>= 15.0%` (disjoint FASTER) on `wikidata` (233 full
//!      Open blocks) AND `>= 12.0%` (disjoint FASTER) on `text` across ALL 4 regimes
//!      (baseline warm, baseline cold, JCC-32B warm, JCC-32B cold), **OR** cuts
//!      `runs::front + runs::fill_avx2_trusted` combined by `>= 10.0%` (disjoint FASTER)
//!      on `text`, `mdjournal`, AND `elf` across all 4 regimes while also cutting
//!      `runs::front` on `wikidata` by `>= 10.0%` (disjoint FASTER) across all 4 regimes;
//!   2. Zero disjoint SLOWER cells and no median regression `> +2.0%` on ANY of the 8
//!      corpora (`wikidata`, `text`, `runs`, `pattern`, `onebyte`, `mdjournal`, `elf`,
//!      `json`) in `front`, `fill`, or `front + fill` across all 4 regimes.
//! - **STOP:** record scoped dead-end row in `frontier/dead-ends.tsv`, no new `idea` claim.

#[cfg(all(feature = "moonshot", target_arch = "x86_64", target_os = "linux"))]
mod x86 {
    use mibz2::block::Symbols;
    use mibz2::bwt::TVector;
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::arch::x86_64::*;
    use std::hint::black_box;
    use std::mem::MaybeUninit;
    use std::path::Path;

    const CAP: u32 = 127;

    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }

    extern "C" {
        fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
    }

    #[inline]
    fn thread_cpu_ns() -> u64 {
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            clock_gettime(3, &mut ts);
        }
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }

    struct L3Scrub {
        buf: Vec<u8>,
        acc: u64,
    }

    impl L3Scrub {
        fn new() -> Self {
            Self {
                buf: vec![1u8; 64 * 1024 * 1024],
                acc: 0,
            }
        }

        #[inline(never)]
        fn scrub(&mut self) {
            let mut s = self.acc;
            let len = self.buf.len();
            let mut i = 0usize;
            while i < len {
                let v = self.buf[i].wrapping_add(1);
                self.buf[i] = v;
                s = s.wrapping_add(v as u64);
                i += 64;
            }
            self.acc = black_box(s);
        }
    }

    /// Branchless 256-bit YMM table for `nn in 0..32` (`f01 = [f0 | f1]`):
    /// 2 `vpshufb` + 1 `vpermq` = 3 Port-5 uops (vs 4 in shipped), 2 loads (vs 4), 1 YMM reg (vs 2).
    #[repr(C, align(64))]
    #[derive(Clone, Copy)]
    struct YmmStep {
        same: [u8; 32],
        cross: [u8; 32],
    }

    const fn make_ymm_steps() -> [YmmStep; 32] {
        const Z: u8 = 0x80;
        let mut t = [YmmStep {
            same: [0; 32],
            cross: [Z; 32],
        }; 32];
        let mut nn = 0;
        while nn < 32 {
            let mut i = 0;
            while i < 16 {
                if nn < 16 {
                    t[nn].same[i] = if i == 0 {
                        nn as u8
                    } else if i <= nn {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                    t[nn].same[16 + i] = i as u8;
                } else {
                    t[nn].same[i] = if i == 0 { Z } else { (i - 1) as u8 };
                    t[nn].same[16 + i] = if i == 0 {
                        Z
                    } else if i <= nn - 16 {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                    if i == 0 {
                        t[nn].cross[0] = (nn - 16) as u8;
                        t[nn].cross[16] = 15;
                    }
                }
                i += 1;
            }
            nn += 1;
        }
        t
    }

    static YMM_STEPS: [YmmStep; 32] = make_ymm_steps();

    #[repr(align(64))]
    struct AlignedRows {
        _data: [u8; 224],
    }

    struct LocalRuns {
        runs: Vec<u16>,
        counts: [u32; 256],
        per: [u32; 256],
        lit: Vec<u64>,
        one: Vec<u64>,
        len: usize,
    }

    impl LocalRuns {
        fn new() -> Self {
            Self {
                runs: Vec::new(),
                counts: [0; 256],
                per: [0; 256],
                lit: Vec::new(),
                one: Vec::new(),
                len: 0,
            }
        }
    }

    #[inline(always)]
    unsafe fn second_shipped(tail: *mut u8, t: usize, f1: __m128i, idx: __m256i) -> u8 {
        let v = *tail.add(t);
        let row = |j: usize| _mm256_load_si256(tail.add(32 * j) as *const __m256i);
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let on = |r: __m256i, below: __m256i| {
            _mm256_alignr_epi8::<15>(r, _mm256_permute2x128_si256::<0x03>(r, below))
        };
        if t < 32 {
            let r0 = row(0);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t as i8 + 1), idx);
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, on(r0, below0), m),
            );
        } else {
            let tv = _mm256_set1_epi8(t as u8 as i8);
            let r: [__m256i; 7] = std::array::from_fn(row);
            for j in 0..7 {
                let p = _mm256_add_epi8(idx, _mm256_set1_epi8((32 * j) as u8 as i8));
                let m = _mm256_cmpeq_epi8(_mm256_min_epu8(p, tv), p);
                let below = if j == 0 { below0 } else { r[j - 1] };
                let new = _mm256_blendv_epi8(r[j], on(r[j], below), m);
                _mm256_store_si256(tail.add(32 * j) as *mut __m256i, new);
            }
        }
        v
    }

    #[inline(always)]
    unsafe fn zero_u32_256(dst: *mut u32) {
        let z = _mm256_setzero_si256();
        let p = dst as *mut __m256i;
        for i in 0..32 {
            _mm256_storeu_si256(p.add(i), z);
        }
    }

    #[inline(always)]
    unsafe fn scan_slice_u64(
        sp_base: *const u16,
        chunk_len: usize,
        eob_sym: u16,
        lit_base: *mut u64,
        one_base: *mut u64,
    ) -> (usize, bool) {
        let v0 = _mm256_setzero_si256();
        let ve = _mm256_set1_epi16(eob_sym as i16);
        macro_rules! bits {
            ($x:expr, $y:expr) => {
                _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                    as u32
            };
        }
        let mut tile_end = chunk_len;
        let mut found_eob = false;
        let mut wi = 0usize;
        let n64 = chunk_len >> 6;
        while wi < n64 {
            let p0 = sp_base.add(wi * 64);
            let x0 = _mm256_loadu_si256(p0 as *const __m256i);
            let y0 = _mm256_loadu_si256(p0.add(16) as *const __m256i);
            let x1 = _mm256_loadu_si256(p0.add(32) as *const __m256i);
            let y1 = _mm256_loadu_si256(p0.add(48) as *const __m256i);

            let r0 = bits!(
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(x0), v0),
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(y0), v0)
            );
            let r1 = bits!(
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(x1), v0),
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(y1), v0)
            );
            let a0 = bits!(_mm256_cmpeq_epi16(x0, v0), _mm256_cmpeq_epi16(y0, v0));
            let a1 = bits!(_mm256_cmpeq_epi16(x1, v0), _mm256_cmpeq_epi16(y1, v0));

            let ex0 = _mm256_cmpeq_epi16(x0, ve);
            let ey0 = _mm256_cmpeq_epi16(y0, ve);
            let ex1 = _mm256_cmpeq_epi16(x1, ve);
            let ey1 = _mm256_cmpeq_epi16(y1, ve);
            let eob_any = _mm256_or_si256(_mm256_or_si256(ex0, ey0), _mm256_or_si256(ex1, ey1));

            let run64 = (r0 as u64) | ((r1 as u64) << 32);
            let runa64 = (a0 as u64) | ((a1 as u64) << 32);
            let mut l64 = !run64;
            let mut o64 = !runa64;

            if _mm256_testz_si256(eob_any, eob_any) == 0 {
                let e0 = bits!(ex0, ey0) as u64;
                let e1 = bits!(ex1, ey1) as u64;
                let eob64 = (e0 | (e1 << 32)) & !run64;
                if eob64 != 0 {
                    let t = eob64.trailing_zeros();
                    tile_end = wi * 64 + t as usize;
                    found_eob = true;
                    let keep = if t == 63 { u64::MAX } else { (2u64 << t) - 1 };
                    l64 &= keep >> 1;
                    o64 &= keep;
                    *lit_base.add(wi) = l64;
                    *one_base.add(wi) = o64;
                    wi += 1;
                    break;
                }
            }
            *lit_base.add(wi) = l64;
            *one_base.add(wi) = o64;
            wi += 1;
        }
        if !found_eob && (wi * 64) < chunk_len {
            let (mut l64, mut o64) = (0u64, 0u64);
            let rem = chunk_len - wi * 64;
            let tail_p = sp_base.add(wi * 64);
            for t in 0..rem {
                let s = *tail_p.add(t);
                o64 |= ((s != 0) as u64) << t;
                if s > 1 {
                    if s == eob_sym {
                        tile_end = wi * 64 + t;
                        found_eob = true;
                        break;
                    }
                    l64 |= 1u64 << t;
                }
            }
            *lit_base.add(wi) = l64;
            *one_base.add(wi) = o64;
            wi += 1;
        }
        *lit_base.add(wi) = 0;
        *one_base.add(wi) = 0;
        *lit_base.add(wi + 1) = 0;
        *one_base.add(wi + 1) = 0;
        (tile_end, found_eob)
    }

    #[inline(always)]
    unsafe fn scan_slice_u32(
        sp_base: *const u16,
        chunk_len: usize,
        eob_sym: u16,
        lit_base: *mut u64,
        one_base: *mut u64,
    ) -> (usize, bool) {
        let lit32 = lit_base as *mut u32;
        let one32 = one_base as *mut u32;
        let v0 = _mm256_setzero_si256();
        let ve = _mm256_set1_epi16(eob_sym as i16);
        macro_rules! bits {
            ($x:expr, $y:expr) => {
                _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                    as u32
            };
        }
        let mut tile_end = chunk_len;
        let mut found_eob = false;
        let mut ci = 0usize;
        let n32 = chunk_len >> 5;
        while ci < n32 {
            let p0 = sp_base.add(ci * 32);
            let x = _mm256_loadu_si256(p0 as *const __m256i);
            let y = _mm256_loadu_si256(p0.add(16) as *const __m256i);
            let run = bits!(
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(x), v0),
                _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(y), v0)
            );
            let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
            let ex = _mm256_cmpeq_epi16(x, ve);
            let ey = _mm256_cmpeq_epi16(y, ve);
            let eob_any = _mm256_or_si256(ex, ey);
            let (mut l, mut o) = (!run, !runa);
            if _mm256_testz_si256(eob_any, eob_any) == 0 {
                let eob = bits!(ex, ey) & !run;
                if eob != 0 {
                    let t = eob.trailing_zeros();
                    tile_end = ci * 32 + t as usize;
                    found_eob = true;
                    let keep = ((2u64 << t) - 1) as u32;
                    l &= keep >> 1;
                    o &= keep;
                    *lit32.add(ci) = l;
                    *one32.add(ci) = o;
                    ci += 1;
                    break;
                }
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        if !found_eob && (ci * 32) < chunk_len {
            let (mut l, mut o) = (0u32, 0u32);
            let rem = chunk_len - ci * 32;
            let tail_p = sp_base.add(ci * 32);
            for t in 0..rem {
                let s = *tail_p.add(t);
                o |= ((s != 0) as u32) << t;
                if s > 1 {
                    if s == eob_sym {
                        tile_end = ci * 32 + t;
                        found_eob = true;
                        break;
                    }
                    l |= 1u32 << t;
                }
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        for i in 0..4 {
            *lit32.add(ci + i) = 0;
            *one32.add(ci + i) = 0;
        }
        (tile_end, found_eob)
    }

    /// Final clean drop-in `front_avx2` variants:
    /// - `MODE = 0` (`U0_per`):   64-sym `u64` `vptest` Pass 1 + `YMM_STEPS` + `[u64; 256]` (keeps `per`)
    /// - `MODE = 1` (`U1_u32`):   64-sym `u64` `vptest` Pass 1 + `YMM_STEPS` + `[u32; 256]` (drops unused `per`)
    /// - `MODE = 2` (`U2_izero`): `U1_u32` + inline AVX2 zeroing of stack `cnt32` (0 `memset`, 0 `vzeroupper` before Pass 2)
    /// - `MODE = 3` (`U3_p32`):   `U2_izero` with 32-sym `vptest` Pass 1 (avoids `one_base` spill in Pass 1)
    /// - `MODE = 4` (`U4_dir`):   `U2_izero` accumulating directly into `f.counts` (0 stack `cnt32`, 0 `memcpy`)
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn front_cand<const MODE: u8>(b: &Symbols, f: &mut LocalRuns) -> Option<usize> {
        let syms = &b.syms;
        let m = syms.len();
        let sp = syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        f.lit.clear();
        f.lit.reserve(words);
        f.one.clear();
        f.one.reserve(words);
        let (end, found_eob) = if MODE == 3 {
            scan_slice_u32(sp, m, b.eob, f.lit.as_mut_ptr(), f.one.as_mut_ptr())
        } else {
            scan_slice_u64(sp, m, b.eob, f.lit.as_mut_ptr(), f.one.as_mut_ptr())
        };
        if !found_eob {
            return None;
        }

        let s2u = &b.seq_to_unseq;
        let mut f01 = _mm256_loadu_si256(s2u.as_ptr() as *const __m256i);
        let mut rows = MaybeUninit::<AlignedRows>::uninit();
        let tail = rows.as_mut_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), tail, 224);
        let idx = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
        let ymm_steps = YMM_STEPS.as_ptr();

        let mut slack = b.max_len / CAP as usize + 1;
        f.runs.clear();
        f.runs.reserve(m + slack + 2);
        let out0 = f.runs.as_mut_ptr();
        let mut out = out0;

        let mut cnt64 = [0u64; 256];
        let mut cnt32_mu = MaybeUninit::<[u32; 256]>::uninit();
        let cnt32_ptr: *mut u32 = if MODE == 4 {
            zero_u32_256(f.counts.as_mut_ptr());
            f.counts.as_mut_ptr()
        } else if MODE == 2 || MODE == 3 {
            let p = cnt32_mu.as_mut_ptr() as *mut u32;
            zero_u32_256(p);
            p
        } else if MODE == 1 {
            cnt32_mu.write([0u32; 256]);
            cnt32_mu.as_mut_ptr() as *mut u32
        } else {
            cnt32_mu.as_mut_ptr() as *mut u32
        };

        let mut big = 0u64;
        let litp = f.lit.as_ptr();
        let onep = f.one.as_ptr();

        macro_rules! long {
            ($c:expr, $total:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                let recs = total.div_ceil(CAP as u64);
                let more = recs.saturating_sub(1) as usize;
                big += total;
                if more > slack || big > b.max_len as u64 {
                    return None;
                }
                slack -= more;
                if MODE == 0 {
                    cnt64[c as usize] += total | (recs << 32);
                } else {
                    *cnt32_ptr.add(c as usize) += total as u32;
                }
                let mut left = total;
                while left > 0 {
                    let len = left.min(CAP as u64);
                    *out = ((c << 8) | len) as u16;
                    out = out.add(1);
                    left -= len;
                }
            }};
        }
        macro_rules! emit {
            ($c:expr, $total:expr, $span:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                if total > CAP as u64 {
                    if $span > 22 {
                        return None;
                    }
                    long!(c, total);
                } else {
                    *(out as *mut u8) = total as u8;
                    *(out as *mut u8).add(1) = c as u8;
                    out = out.add(1);
                    if MODE == 0 {
                        cnt64[c as usize] += total | (1u64 << 32);
                    } else {
                        *cnt32_ptr.add(c as usize) += total as u32;
                    }
                }
            }};
        }

        let last = end / 64;
        let (mut wi, mut w) = (0usize, *litp);
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
                    w = *litp.add(wi);
                }
            }};
        }
        macro_rules! count {
            ($q:expr, $q2:expr) => {{
                let from = $q + 1;
                let bits = ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned()
                    >> (from & 7);
                _bzhi_u64(bits, ($q2 - $q) as u32)
            }};
        }

        let first = seek!();
        let q = first.map_or(end, |t| wi * 64 + t);
        if q > 21 {
            return None;
        }
        let open_len = _bzhi_u64(*onep, q as u32 + 1) - 1;
        if open_len > 0 {
            long!(*s2u.as_ptr() as u64, open_len);
        }
        if let Some(mut tz) = first {
            let mut after = *onep.add(wi) >> 1;
            let mut spw = sp.add(wi * 64);
            loop {
                let s = *spw.add(tz) as usize;
                if s < 33 {
                    let t = ymm_steps.add(s - 1);
                    let same = _mm256_load_si256((*t).same.as_ptr() as *const __m256i);
                    let cross = _mm256_load_si256((*t).cross.as_ptr() as *const __m256i);
                    let swapped = _mm256_permute4x64_epi64::<0x4e>(f01);
                    f01 = _mm256_or_si256(
                        _mm256_shuffle_epi8(f01, same),
                        _mm256_shuffle_epi8(swapped, cross),
                    );
                } else {
                    let f0_l = _mm256_castsi256_si128(f01);
                    let f1_l = _mm256_extracti128_si256::<1>(f01);
                    let v = second_shipped(tail, (s - 1).min(255) - 32, f1_l, idx);
                    let nf1 = _mm_alignr_epi8::<15>(f1_l, f0_l);
                    let nf0 = _mm_or_si128(_mm_slli_si128::<1>(f0_l), _mm_cvtsi32_si128(v as i32));
                    f01 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(nf0), nf1);
                }
                let c = _mm_cvtsi128_si32(_mm256_castsi256_si128(f01)) as u8 as u64;
                if w != 0 {
                    let t2 = w.trailing_zeros() as usize;
                    w &= w - 1;
                    emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                    tz = t2;
                } else {
                    let q = wi * 64 + tz;
                    let next = seek!();
                    let q2 = next.map_or(end, |t| wi * 64 + t);
                    if q2 - q > 22 {
                        return None;
                    }
                    emit!(c, count!(q, q2), q2 - q);
                    match next {
                        Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                        None => break,
                    }
                }
            }
        }

        let n: u64 = if MODE == 0 {
            let s: u64 = cnt64.iter().map(|&x| x & 0xffff_ffff).sum();
            if s > b.max_len as u64 {
                return None;
            }
            f.runs.set_len(out.offset_from(out0) as usize);
            for (c, &x) in cnt64.iter().enumerate() {
                f.counts[c] = x as u32;
                f.per[c] = (x >> 32) as u32;
            }
            s
        } else if MODE == 4 {
            let s: u64 = f.counts.iter().map(|&x| x as u64).sum();
            if s > b.max_len as u64 {
                return None;
            }
            f.runs.set_len(out.offset_from(out0) as usize);
            s
        } else {
            let cnt = cnt32_mu.assume_init();
            let s: u64 = cnt.iter().map(|&x| x as u64).sum();
            if s > b.max_len as u64 {
                return None;
            }
            f.runs.set_len(out.offset_from(out0) as usize);
            f.counts = cnt;
            s
        };
        f.len = n as usize;
        Some(n as usize)
    }

    /// Exact replica of shipped `src/runs.rs::fill_avx2_trusted` for paired `front + fill` timing
    /// and TT verification.
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_trusted_local(runs: &[u16], counts: &[u32; 256], n: usize, out: &mut Vec<u32>) {
        out.clear();
        out.reserve(n + 8);
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
        }
        let ramp = _mm256_setr_epi32(0, 1 << 8, 2 << 8, 3 << 8, 4 << 8, 5 << 8, 6 << 8, 7 << 8);
        let step = _mm256_set1_epi32(8 << 8);
        let mut dst = out.as_mut_ptr();
        let next_ptr = next.as_mut_ptr();
        for &rec in runs {
            let c = (rec >> 8) as usize;
            let len = (rec & 0xff) as usize;
            let slot = next_ptr.add(c);
            let k = *slot;
            *slot = k + len as u32;
            let mut v = _mm256_add_epi32(_mm256_set1_epi32((c as u32 | k << 8) as i32), ramp);
            _mm256_storeu_si256(dst as *mut __m256i, v);
            let mut j = 8usize;
            while j < len {
                v = _mm256_add_epi32(v, step);
                _mm256_storeu_si256(dst.add(j) as *mut __m256i, v);
                j += 8;
            }
            dst = dst.add(len);
        }
        out.set_len(n);
    }

    // ---- Helpers & Runner ----

    fn scan_blocks(data: &[u8]) -> Vec<Symbols> {
        let mut sc = Scanner::new(SliceSource(data));
        let mut blocks = Vec::new();
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).expect("scan valid bzip2") {
            blocks.push(symbols);
        }
        blocks
    }

    fn median(v: &mut [f64]) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        if n % 2 == 1 {
            v[n / 2]
        } else {
            0.5 * (v[n / 2 - 1] + v[n / 2])
        }
    }

    fn min_max(v: &[f64]) -> (f64, f64) {
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for &x in v {
            lo = lo.min(x);
            hi = hi.max(x);
        }
        (lo, hi)
    }

    fn verify_synthetic_edges(k: Kernels) {
        let mut cases: Vec<(&str, Symbols)> = Vec::new();
        let make_sym = |syms: Vec<u16>, eob: u16, max_len: usize| -> Symbols {
            let mut s2u = [0u8; 256];
            for (i, b) in s2u.iter_mut().enumerate() {
                *b = i as u8;
            }
            Symbols {
                syms,
                eob,
                seq_to_unseq: s2u,
                orig_ptr: 0,
                crc: 0,
                max_len,
            }
        };
        cases.push(("empty", make_sym(vec![2], 2, 1000)));
        cases.push(("no_eob", make_sym(vec![0, 1, 2, 3], 10, 1000)));
        let mut rtl = vec![0u16; 22];
        rtl.push(5);
        cases.push(("rtl_start", make_sym(rtl, 5, 900_000)));
        let mut rtl2 = vec![2u16];
        rtl2.extend(std::iter::repeat(1u16).take(22));
        rtl2.push(5);
        cases.push(("rtl_mid", make_sym(rtl2, 5, 900_000)));
        cases.push(("btl", make_sym(vec![1, 1, 1, 1, 5], 5, 10)));
        cases.push((
            "sym_gt256",
            make_sym(vec![2, 256, 257, 300, 0x8000, 258], 258, 1000),
        ));
        for run_syms in 0..=21usize {
            for offset in [31usize, 32, 63, 64, 65, 127, 128] {
                let mut syms = vec![2u16; offset];
                for j in 0..run_syms {
                    syms.push((j & 1) as u16);
                }
                syms.extend([3, 20, 50, 64, 65, 96, 150, 255, 256, 257]);
                cases.push(("word_boundary", make_sym(syms, 257, 900_000)));
            }
        }
        let mut ship_r = Runs::default();
        let mut lr = LocalRuns::new();
        for (label, b) in &cases {
            let ship_res = runs::front(b, &mut ship_r, k);
            let u0 = unsafe { front_cand::<0>(b, &mut lr) };
            for (nm, res) in [
                ("U1_u32", unsafe { front_cand::<1>(b, &mut lr) }),
                ("U2_izero", unsafe { front_cand::<2>(b, &mut lr) }),
                ("U3_p32", unsafe { front_cand::<3>(b, &mut lr) }),
                ("U4_dir", unsafe { front_cand::<4>(b, &mut lr) }),
            ] {
                assert_eq!(res, u0, "edge {label}: {nm}");
            }
            if let (Ok(n), Some(n1)) = (ship_res, u0) {
                assert_eq!(n, n1);
                assert_eq!(lr.runs.as_slice(), ship_r.records());
            }
        }
        println!("[EXACT-EDGES] {}/{} PASS", cases.len(), cases.len());
    }

    struct CorpusData {
        name: String,
        blocks: Vec<Symbols>,
    }

    fn verify_corpus(name: &str, all_blocks: Vec<Symbols>, k: Kernels) -> CorpusData {
        let mut admitted = Vec::new();
        for b in all_blocks {
            if runs::takes_runs(&b) {
                if name == "wikidata" && b.syms.len() < 4096 {
                    continue;
                }
                admitted.push(b);
            }
        }
        let mut ship_r = Runs::default();
        let mut lr_ref = LocalRuns::new();
        let mut lr_cand = LocalRuns::new();
        let mut tt_ship = TVector::default();
        let mut tt_loc = Vec::new();
        for (bi, b) in admitted.iter().enumerate() {
            let n_ship = runs::front(b, &mut ship_r, k).expect("shipped front");
            assert!(runs::fill(&ship_r, n_ship, &mut tt_ship, k));
            assert_eq!(tt_ship.len(), n_ship);
            let n_ref = unsafe { front_cand::<0>(b, &mut lr_ref) }.expect("U0");
            assert_eq!(n_ref, n_ship);
            assert_eq!(lr_ref.runs.as_slice(), ship_r.records());

            type FrontFn = unsafe fn(&Symbols, &mut LocalRuns) -> Option<usize>;
            let arms: [(&str, FrontFn); 4] = [
                ("U1_u32", front_cand::<1>),
                ("U2_izero", front_cand::<2>),
                ("U3_p32", front_cand::<3>),
                ("U4_dir", front_cand::<4>),
            ];
            for (aname, afn) in arms {
                let nc = unsafe { afn(b, &mut lr_cand) }.expect(aname);
                assert_eq!(nc, n_ref, "{name} blk {bi}: {aname} n");
                assert_eq!(lr_cand.runs, lr_ref.runs, "{name} blk {bi}: {aname} runs");
                assert_eq!(
                    lr_cand.counts, lr_ref.counts,
                    "{name} blk {bi}: {aname} counts"
                );
            }
            // Verify fill_trusted_local produces exact TT length & valid tiling
            unsafe { fill_trusted_local(&lr_cand.runs, &lr_cand.counts, n_ref, &mut tt_loc) };
            assert_eq!(tt_loc.len(), n_ship);
        }
        println!(
            "[EXACT] {:10}: {:3} blks x 5 arms + TT PASS",
            name,
            admitted.len()
        );
        CorpusData {
            name: name.to_string(),
            blocks: admitted,
        }
    }

    fn calibrate_inner(one_ms: f64, cold: bool) -> usize {
        if cold {
            if one_ms < 4.0 {
                (10.0 / one_ms.max(0.005)).ceil().clamp(1.0, 2000.0) as usize
            } else {
                1
            }
        } else {
            (25.0 / one_ms.max(0.005)).ceil().clamp(1.0, 4000.0) as usize
        }
    }

    fn bench_front(corpus: &CorpusData, reps: usize, cold: bool, k: Kernels) {
        let mut scrubber = if cold { Some(L3Scrub::new()) } else { None };
        let mut ship_r = Runs::default();
        let mut cal_n = 1usize;
        let one_ms = loop {
            let t0 = thread_cpu_ns();
            for _ in 0..cal_n {
                for b in &corpus.blocks {
                    black_box(runs::front(b, &mut ship_r, k).unwrap());
                }
            }
            let dt = (thread_cpu_ns().saturating_sub(t0)) as f64 / 1e6;
            if dt >= 5.0 || cal_n >= 4096 {
                break dt / cal_n as f64;
            }
            cal_n *= 4;
        };
        let inner = calibrate_inner(one_ms, cold);

        let names = [
            "F0_ship", "U0_per", "U1_u32", "U2_izero", "U3_p32", "U4_dir",
        ];
        let n_arms = names.len();
        let mut samples = vec![Vec::with_capacity(reps); n_arms];
        let mut lr = LocalRuns::new();

        for b in &corpus.blocks {
            black_box(runs::front(b, &mut ship_r, k).unwrap());
            black_box(unsafe { front_cand::<2>(b, &mut lr).unwrap() });
        }

        for rep in 0..reps {
            for step in 0..n_arms {
                let arm = (rep + step) % n_arms;
                if let Some(ref mut sc) = scrubber {
                    sc.scrub();
                }
                let start = thread_cpu_ns();
                for _ in 0..inner {
                    for b in &corpus.blocks {
                        let n = match arm {
                            0 => runs::front(b, &mut ship_r, k).unwrap(),
                            1 => unsafe { front_cand::<0>(b, &mut lr).unwrap() },
                            2 => unsafe { front_cand::<1>(b, &mut lr).unwrap() },
                            3 => unsafe { front_cand::<2>(b, &mut lr).unwrap() },
                            4 => unsafe { front_cand::<3>(b, &mut lr).unwrap() },
                            _ => unsafe { front_cand::<4>(b, &mut lr).unwrap() },
                        };
                        black_box(n);
                    }
                }
                let dt_ms = (thread_cpu_ns().saturating_sub(start)) as f64 / 1e6 / inner as f64;
                samples[arm].push(dt_ms);
            }
        }

        let mode = if cold { "COLD" } else { "WARM" };
        let base_med = median(&mut samples[0].clone());
        let (base_lo, base_hi) = min_max(&samples[0]);
        print!(
            "FRONT {:4} {:10} (in={:4}, ship={:6.3} [{:5.3},{:5.3}] ms):",
            mode, corpus.name, inner, base_med, base_lo, base_hi
        );
        for arm in 1..n_arms {
            let m = median(&mut samples[arm].clone());
            let (lo, hi) = min_max(&samples[arm]);
            let pct = 100.0 * (m - base_med) / base_med.max(1e-9);
            let tag = if hi < base_lo {
                "F"
            } else if lo > base_hi {
                "S"
            } else {
                "~"
            };
            print!(" {}={:+5.1}%{}", names[arm], pct, tag);
        }
        println!();
    }

    fn bench_combined(corpus: &CorpusData, reps: usize, cold: bool, k: Kernels) {
        let mut scrubber = if cold { Some(L3Scrub::new()) } else { None };
        let mut ship_r = Runs::default();
        let mut tt_ship = TVector::default();
        let mut cal_n = 1usize;
        let one_ms = loop {
            let t0 = thread_cpu_ns();
            for _ in 0..cal_n {
                for b in &corpus.blocks {
                    let n = runs::front(b, &mut ship_r, k).unwrap();
                    black_box(runs::fill(&ship_r, n, &mut tt_ship, k));
                }
            }
            let dt = (thread_cpu_ns().saturating_sub(t0)) as f64 / 1e6;
            if dt >= 5.0 || cal_n >= 4096 {
                break dt / cal_n as f64;
            }
            cal_n *= 4;
        };
        let inner = calibrate_inner(one_ms, cold);

        let names = ["C0_ship", "C1_U2_izero", "C2_U3_p32", "C3_U4_dir"];
        let n_arms = names.len();
        let mut samples = vec![Vec::with_capacity(reps); n_arms];
        let mut lr = LocalRuns::new();
        let mut tt_loc = Vec::new();

        for b in &corpus.blocks {
            let n = runs::front(b, &mut ship_r, k).unwrap();
            black_box(runs::fill(&ship_r, n, &mut tt_ship, k));
            let n2 = unsafe { front_cand::<2>(b, &mut lr).unwrap() };
            unsafe { fill_trusted_local(&lr.runs, &lr.counts, n2, &mut tt_loc) };
            black_box(tt_loc.len());
        }

        for rep in 0..reps {
            for step in 0..n_arms {
                let arm = (rep + step) % n_arms;
                if let Some(ref mut sc) = scrubber {
                    sc.scrub();
                }
                let start = thread_cpu_ns();
                for _ in 0..inner {
                    for b in &corpus.blocks {
                        match arm {
                            0 => {
                                let n = runs::front(b, &mut ship_r, k).unwrap();
                                black_box(runs::fill(&ship_r, n, &mut tt_ship, k));
                            }
                            1 => {
                                let n = unsafe { front_cand::<2>(b, &mut lr).unwrap() };
                                unsafe { fill_trusted_local(&lr.runs, &lr.counts, n, &mut tt_loc) };
                                black_box(tt_loc.len());
                            }
                            2 => {
                                let n = unsafe { front_cand::<3>(b, &mut lr).unwrap() };
                                unsafe { fill_trusted_local(&lr.runs, &lr.counts, n, &mut tt_loc) };
                                black_box(tt_loc.len());
                            }
                            _ => {
                                let n = unsafe { front_cand::<4>(b, &mut lr).unwrap() };
                                unsafe { fill_trusted_local(&lr.runs, &lr.counts, n, &mut tt_loc) };
                                black_box(tt_loc.len());
                            }
                        }
                    }
                }
                let dt_ms = (thread_cpu_ns().saturating_sub(start)) as f64 / 1e6 / inner as f64;
                samples[arm].push(dt_ms);
            }
        }

        let mode = if cold { "COLD" } else { "WARM" };
        let base_med = median(&mut samples[0].clone());
        let (base_lo, base_hi) = min_max(&samples[0]);
        print!(
            "F+FILL {:4} {:10} (in={:4}, ship={:6.3} [{:5.3},{:5.3}] ms):",
            mode, corpus.name, inner, base_med, base_lo, base_hi
        );
        for arm in 1..n_arms {
            let m = median(&mut samples[arm].clone());
            let (lo, hi) = min_max(&samples[arm]);
            let pct = 100.0 * (m - base_med) / base_med.max(1e-9);
            let tag = if hi < base_lo {
                "F"
            } else if lo > base_hi {
                "S"
            } else {
                "~"
            };
            print!(" {}={:+5.1}%{}", names[arm], pct, tag);
        }
        println!();
    }

    pub fn run() {
        let mut args = std::env::args().skip(1);
        let mut reps = 11usize;
        let mut files: Vec<String> = Vec::new();
        while let Some(a) = args.next() {
            match a.as_str() {
                "--reps" => reps = args.next().unwrap().parse().unwrap(),
                _ => files.push(a),
            }
        }
        let k = Kernels::detect();
        verify_synthetic_edges(k);

        let mut corpora = Vec::new();
        for path in &files {
            let p = Path::new(path);
            let name = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(path)
                .to_string();
            let data = std::fs::read(p).expect("read bz2 corpus");
            let blocks = scan_blocks(&data);
            corpora.push(verify_corpus(&name, blocks, k));
        }

        println!("\n=== FRONT R7 (WARM, reps={reps}) ===");
        for c in &corpora {
            bench_front(c, reps, false, k);
        }
        println!("\n=== FRONT R7 (L3-COLD 64 MiB scrub, reps={reps}) ===");
        for c in &corpora {
            bench_front(c, reps, true, k);
        }
        println!("\n=== FRONT+FILL R7 (WARM, reps={reps}) ===");
        for c in &corpora {
            bench_combined(c, reps, false, k);
        }
        println!("\n=== FRONT+FILL R7 (L3-COLD 64 MiB scrub, reps={reps}) ===");
        for c in &corpora {
            bench_combined(c, reps, true, k);
        }
    }
}

#[cfg(all(feature = "moonshot", target_arch = "x86_64", target_os = "linux"))]
fn main() {
    x86::run();
}

#[cfg(not(all(feature = "moonshot", target_arch = "x86_64", target_os = "linux")))]
fn main() {}
