//! (research only, `explore/rle1-2load-filter`) Deep dissection of `src/rle.rs`
//! (`find_runs_avx2`, `unrle_in_place`, `unrle_avx2`) comparing shipped 4-load
//! 3-`vpcmpeqb` scanning against 2-load 1-`vpcmpeqb` `m1` filters, 1-load
//! `vpalignr`, inline small-copy/run-fill, and raw-pointer out-of-place AVX2.
//!
//!     taskset -c 0 cargo run --release --example rle1_dissection -- /path/to/wikidata.bz2
//!
//! Zero `src/` edits. Every candidate arm is verified byte- and CRC-exact
//! (including `Err(ErrorKind::RunUnfinished)`) across exhaustive boundary cases
//! and every real block of Wikidata + all 7 guard corpora before timing.

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("rle1_dissection requires x86_64 with AVX2+BMI1");
}

#[cfg(target_arch = "x86_64")]
#[path = "../tests/common/mod.rs"]
mod common;

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::common;
    use std::arch::x86_64::*;
    use std::hint::black_box;
    use std::io::Read;
    use std::path::Path;

    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{bwt, crc, mtf, rle, BzReader, ErrorKind, Kernels};

    const TILE: usize = 64 << 10;
    const POOLED_FROM: usize = 64 << 10;

    pub type Run = (u32, u8, u8);

    #[inline]
    fn prod_pool_cap(n: usize) -> usize {
        if n >= POOLED_FROM {
            n + n / 16 + 64
        } else {
            n
        }
    }

    #[cfg(target_os = "linux")]
    fn thread_cpu_ns() -> u64 {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
        unsafe extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
        }
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut ts);
        }
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }

    #[cfg(not(target_os = "linux"))]
    fn thread_cpu_ns() -> u64 {
        0
    }

    #[inline(never)]
    fn flush_l3(buf: &mut [u8]) {
        let mut acc = 0u8;
        let mut i = 0usize;
        while i < buf.len() {
            acc = acc.wrapping_add(buf[i]);
            buf[i] = acc.wrapping_add(1);
            i += 64;
        }
        black_box(acc);
    }

    // -----------------------------------------------------------------------
    // Scalar helpers (matching src/rle.rs verbatim)
    // -----------------------------------------------------------------------

    #[inline]
    fn find_runs_scalar(rle: &[u8], runs: &mut Vec<Run>, mut i: usize) -> Result<(), ErrorKind> {
        let n = rle.len();
        while i + 3 < n {
            let c = rle[i];
            if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
                let Some(&extra) = rle.get(i + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((i as u32, c, extra));
                i += 5;
            } else {
                i += 1;
            }
        }
        Ok(())
    }

    #[inline]
    fn find_runs_scalar_stats(
        rle: &[u8],
        runs: &mut Vec<Run>,
        mut i: usize,
        grown: &mut isize,
        shift: &mut isize,
    ) -> Result<(), ErrorKind> {
        let n = rle.len();
        while i + 3 < n {
            let c = rle[i];
            if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
                let Some(&extra) = rle.get(i + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((i as u32, c, extra));
                *grown += extra as isize - 1;
                if *grown > *shift {
                    *shift = *grown;
                }
                i += 5;
            } else {
                i += 1;
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Family A: `find_runs` AVX2 kernels
    // -----------------------------------------------------------------------

    /// F0: Exact verbatim copy of shipped `src/rle.rs::find_runs_avx2` (4-load, 3-vpcmpeqb).
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F0_shipped_4load(
        rle: &[u8],
        runs: &mut Vec<Run>,
    ) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0;
        while i + 35 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
            let eq = _mm256_and_si256(
                _mm256_cmpeq_epi8(v0, v1),
                _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
            );
            let m = _mm256_movemask_epi8(eq) as u32;
            if m == 0 {
                i += 32;
                continue;
            }
            let j = i + m.trailing_zeros() as usize;
            let Some(&extra) = rle.get(j + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            runs.push((j as u32, rle[j], extra));
            i = j + 5;
        }
        find_runs_scalar(rle, runs, i)
    }

    /// F1: 2-load, 1-vpcmpeqb `m1` filter, 32-byte stride, 2-bit carry, multi-hit drain.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F1_m1_32b(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u32; // bit 0: b[i-2]==b[i-1], bit 1: b[i-1]==b[i]
        while i + 33 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32;
            let w0 = (m << 2) | prev;
            let w1 = (m << 1) | (prev >> 1);
            let tri = w0 & w1 & m;
            if tri == 0 {
                prev = m >> 30;
                i += 32;
                continue;
            }
            let base = i.wrapping_sub(2);
            let mut mask = tri;
            let mut next_i = i + 32;
            let mut next_prev = m >> 30;
            while mask != 0 {
                let tz = mask.trailing_zeros() as usize;
                let j = base.wrapping_add(tz);
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, *p.add(j), extra));
                let skip = tz + 5;
                if skip >= 32 {
                    next_i = j + 5;
                    next_prev = 0;
                    break;
                }
                mask &= !((1u32 << skip) - 1);
            }
            i = next_i;
            prev = next_prev;
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar(rle, runs, tail)
    }

    /// F2: 2-load, 1-vpcmpeqb `m1` filter, 64-byte `u64` unrolled, 2-bit carry, multi-hit drain.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F2_m1_64b(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u64;
        while i + 65 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32 as u64;
            let v2 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 33) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v2, v3)) as u32 as u64;
            let m = m0 | (m1 << 32);
            let w0 = (m << 2) | prev;
            let w1 = (m << 1) | (prev >> 1);
            let tri = w0 & w1 & m;
            if tri == 0 {
                prev = m >> 62;
                i += 64;
                continue;
            }
            let base = i.wrapping_sub(2);
            let mut mask = tri;
            let mut next_i = i + 64;
            let mut next_prev = m >> 62;
            while mask != 0 {
                let tz = mask.trailing_zeros() as usize;
                let j = base.wrapping_add(tz);
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, *p.add(j), extra));
                let skip = tz + 5;
                if skip >= 64 {
                    next_i = j + 5;
                    next_prev = 0;
                    break;
                }
                mask &= !((1u64 << skip) - 1);
            }
            i = next_i;
            prev = next_prev;
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar(rle, runs, tail)
    }

    /// F3: 2-load, 1-vpcmpeqb `m1` filter, 128-byte (2×`u64`) unrolled, 1 branch per 128B.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F3_m1_128b(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u64;
        while i + 129 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32 as u64;
            let v2 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 33) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v2, v3)) as u32 as u64;
            let ma = m0 | (m1 << 32);
            let tri_a = ((ma << 2) | prev) & ((ma << 1) | (prev >> 1)) & ma;
            let mid = ma >> 62;

            let v4 = _mm256_loadu_si256(p.add(i + 64) as *const __m256i);
            let v5 = _mm256_loadu_si256(p.add(i + 65) as *const __m256i);
            let m2 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v4, v5)) as u32 as u64;
            let v6 = _mm256_loadu_si256(p.add(i + 96) as *const __m256i);
            let v7 = _mm256_loadu_si256(p.add(i + 97) as *const __m256i);
            let m3 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v6, v7)) as u32 as u64;
            let mb = m2 | (m3 << 32);
            let tri_b = ((mb << 2) | mid) & ((mb << 1) | (mid >> 1)) & mb;

            if (tri_a | tri_b) == 0 {
                prev = mb >> 62;
                i += 128;
                continue;
            }

            if tri_a != 0 {
                let base = i.wrapping_sub(2);
                let mut mask = tri_a;
                let mut broke = false;
                while mask != 0 {
                    let tz = mask.trailing_zeros() as usize;
                    let j = base.wrapping_add(tz);
                    let Some(&extra) = rle.get(j + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((j as u32, *p.add(j), extra));
                    let skip = tz + 5;
                    if skip >= 64 {
                        i = j + 5;
                        prev = 0;
                        broke = true;
                        break;
                    }
                    mask &= !((1u64 << skip) - 1);
                }
                if broke {
                    continue;
                }
            }
            if tri_b != 0 {
                let base = (i + 64).wrapping_sub(2);
                let mut mask = tri_b;
                let mut next_i = i + 128;
                let mut next_prev = mb >> 62;
                while mask != 0 {
                    let tz = mask.trailing_zeros() as usize;
                    let j = base.wrapping_add(tz);
                    let Some(&extra) = rle.get(j + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((j as u32, *p.add(j), extra));
                    let skip = tz + 5;
                    if skip >= 64 {
                        next_i = j + 5;
                        next_prev = 0;
                        break;
                    }
                    mask &= !((1u64 << skip) - 1);
                }
                i = next_i;
                prev = next_prev;
            } else {
                prev = mb >> 62;
                i += 128;
            }
        }
        // Finish with 32B/scalar tail cleanly.
        while i + 33 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32;
            let p32 = prev as u32;
            let tri = ((m << 2) | p32) & ((m << 1) | (p32 >> 1)) & m;
            if tri == 0 {
                prev = (m >> 30) as u64;
                i += 32;
                continue;
            }
            let base = i.wrapping_sub(2);
            let mut mask = tri;
            let mut next_i = i + 32;
            let mut next_prev = (m >> 30) as u64;
            while mask != 0 {
                let tz = mask.trailing_zeros() as usize;
                let j = base.wrapping_add(tz);
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, *p.add(j), extra));
                let skip = tz + 5;
                if skip >= 32 {
                    next_i = j + 5;
                    next_prev = 0;
                    break;
                }
                mask &= !((1u32 << skip) - 1);
            }
            i = next_i;
            prev = next_prev;
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar(rle, runs, tail)
    }

    /// F4: 32B-aligned prologue + F2_m1_64b (eliminates cache-line splits on `v0`/`v2`).
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F4_m1_64b_aligned(
        rle: &[u8],
        runs: &mut Vec<Run>,
    ) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u64;
        // Align `p.add(i)` to 32 bytes when enough bytes remain.
        let mis = (p as usize) & 31;
        if mis != 0 && n >= 128 {
            let align_to = 32 - mis;
            while i < align_to {
                let c = *p.add(i);
                if *p.add(i + 1) == c && *p.add(i + 2) == c && *p.add(i + 3) == c {
                    let Some(&extra) = rle.get(i + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((i as u32, c, extra));
                    i += 5;
                } else {
                    i += 1;
                }
            }
            // If a run crossed `align_to`, advance to the next 32B boundary again if still misaligned.
            while ((p.add(i) as usize) & 31) != 0 && i + 4 < n {
                let c = *p.add(i);
                if *p.add(i + 1) == c && *p.add(i + 2) == c && *p.add(i + 3) == c {
                    let Some(&extra) = rle.get(i + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((i as u32, c, extra));
                    i += 5;
                } else {
                    i += 1;
                }
            }
        }
        while i + 65 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32 as u64;
            let v2 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 33) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v2, v3)) as u32 as u64;
            let m = m0 | (m1 << 32);
            let tri = ((m << 2) | prev) & ((m << 1) | (prev >> 1)) & m;
            if tri == 0 {
                prev = m >> 62;
                i += 64;
                continue;
            }
            let base = i.wrapping_sub(2);
            let mut mask = tri;
            let mut next_i = i + 64;
            let mut next_prev = m >> 62;
            while mask != 0 {
                let tz = mask.trailing_zeros() as usize;
                let j = base.wrapping_add(tz);
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, *p.add(j), extra));
                let skip = tz + 5;
                if skip >= 64 {
                    next_i = j + 5;
                    next_prev = 0;
                    break;
                }
                mask &= !((1u64 << skip) - 1);
            }
            i = next_i;
            prev = next_prev;
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar(rle, runs, tail)
    }

    /// F5: True 1-load per 32B via `vperm2i128` + `vpalignr::<1>`, 64B unrolled.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_F5_1load_alignr_64b(
        rle: &[u8],
        runs: &mut Vec<Run>,
    ) -> Result<(), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u64;
        if i + 96 <= n {
            let mut v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            while i + 96 <= n {
                let v1 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
                let v2 = _mm256_loadu_si256(p.add(i + 64) as *const __m256i);
                let c0 = _mm256_permute2x128_si256::<0x21>(v0, v1);
                let s0 = _mm256_alignr_epi8::<1>(c0, v0);
                let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, s0)) as u32 as u64;
                let c1 = _mm256_permute2x128_si256::<0x21>(v1, v2);
                let s1 = _mm256_alignr_epi8::<1>(c1, v1);
                let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v1, s1)) as u32 as u64;
                let m = m0 | (m1 << 32);
                let tri = ((m << 2) | prev) & ((m << 1) | (prev >> 1)) & m;
                if tri == 0 {
                    prev = m >> 62;
                    i += 64;
                    v0 = v2;
                    continue;
                }
                let base = i.wrapping_sub(2);
                let mut mask = tri;
                let mut next_i = i + 64;
                let mut next_prev = m >> 62;
                let mut broke = false;
                while mask != 0 {
                    let tz = mask.trailing_zeros() as usize;
                    let j = base.wrapping_add(tz);
                    let Some(&extra) = rle.get(j + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((j as u32, *p.add(j), extra));
                    let skip = tz + 5;
                    if skip >= 64 {
                        next_i = j + 5;
                        next_prev = 0;
                        broke = true;
                        break;
                    }
                    mask &= !((1u64 << skip) - 1);
                }
                i = next_i;
                prev = next_prev;
                if i + 96 <= n {
                    v0 = if broke {
                        _mm256_loadu_si256(p.add(i) as *const __m256i)
                    } else {
                        v2
                    };
                }
            }
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar(rle, runs, tail)
    }

    // -----------------------------------------------------------------------
    // Family B: Whole `unrle_in_place` (A9) arms
    // -----------------------------------------------------------------------

    /// Exact verbatim copy of shipped `src/rle.rs::unrle_in_place` using a pluggable `find_runs`
    /// and optional `U2_m1_64b_raw` on the `shift > n` stage-swap path.
    #[inline(always)]
    unsafe fn unrle_in_place_core(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
        find_fn: unsafe fn(&[u8], &mut Vec<Run>) -> Result<(), ErrorKind>,
        use_u2_stage: bool,
    ) -> Result<u32, ErrorKind> {
        let n = buf.len();
        if n > u32::MAX as usize {
            std::mem::swap(buf, stage);
            buf.clear();
            if use_u2_stage {
                rle1_probe_U2_m1_64b_raw(stage, buf)?;
            } else {
                rle::unrle(stage, buf, k)?;
            }
            return Ok(crc::crc32(buf, k));
        }
        find_fn(buf, runs)?;
        let Some(&(first, _, _)) = runs.first() else {
            return Ok(crc::crc32(buf, k));
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        if shift > n {
            std::mem::swap(buf, stage);
            buf.clear();
            if use_u2_stage {
                rle1_probe_U2_m1_64b_raw(stage, buf)?;
            } else {
                rle::unrle(stage, buf, k)?;
            }
            return Ok(crc::crc32(buf, k));
        }
        let first = first as usize;
        if shift > 0 {
            buf.reserve_exact(shift);
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= TILE {
                *s = crc::update_with(*s, &buf[*at..*at + TILE], k);
                *at += TILE;
            }
        };
        tiles(&mut s, &mut at, buf, first);
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                if dst != src {
                    buf.copy_within(src..from, dst);
                }
                dst += from - src;
            }
            let run = 4 + extra as usize;
            buf[dst..dst + run].fill(c);
            dst += run;
            src = from + 5;
            tiles(&mut s, &mut at, buf, dst);
        }
        if n + shift > src && dst != src {
            buf.copy_within(src..n + shift, dst);
        }
        buf.truncate(len);
        s = crc::update_with(s, &buf[at..], k);
        Ok(!s)
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P0_shipped(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        unrle_in_place_core(buf, runs, stage, k, rle1_probe_F0_shipped_4load, false)
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P1_m1_64b(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        unrle_in_place_core(buf, runs, stage, k, rle1_probe_F2_m1_64b, false)
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P2_m1_128b(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        unrle_in_place_core(buf, runs, stage, k, rle1_probe_F3_m1_128b, false)
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P5_m1_128b_u2stage(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        unrle_in_place_core(buf, runs, stage, k, rle1_probe_F3_m1_128b, true)
    }

    /// On-the-fly `find_runs` returning `(grown, shift)` with 64B `m1` filter.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn find_runs_m1_64b_stats(
        rle: &[u8],
        runs: &mut Vec<Run>,
    ) -> Result<(isize, isize), ErrorKind> {
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
        let mut prev = 0u64;
        let (mut grown, mut shift) = (0isize, 0isize);
        while i + 65 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32 as u64;
            let v2 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 33) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v2, v3)) as u32 as u64;
            let m = m0 | (m1 << 32);
            let tri = ((m << 2) | prev) & ((m << 1) | (prev >> 1)) & m;
            if tri == 0 {
                prev = m >> 62;
                i += 64;
                continue;
            }
            let base = i.wrapping_sub(2);
            let mut mask = tri;
            let mut next_i = i + 64;
            let mut next_prev = m >> 62;
            while mask != 0 {
                let tz = mask.trailing_zeros() as usize;
                let j = base.wrapping_add(tz);
                let Some(&extra) = rle.get(j + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((j as u32, *p.add(j), extra));
                grown += extra as isize - 1;
                if grown > shift {
                    shift = grown;
                }
                let skip = tz + 5;
                if skip >= 64 {
                    next_i = j + 5;
                    next_prev = 0;
                    break;
                }
                mask &= !((1u64 << skip) - 1);
            }
            i = next_i;
            prev = next_prev;
        }
        let tail = if prev != 0 { i - 2 } else { i };
        find_runs_scalar_stats(rle, runs, tail, &mut grown, &mut shift)?;
        Ok((grown, shift))
    }

    /// Inline small-copy + inline short-run fill (zero `call memset` on `run <= 32`,
    /// zero `call memmove` on `copy_len <= 64`), plus `U2` on `shift > n` stage-swap.
    #[inline(always)]
    unsafe fn copy_small_or_memmove(ptr: *mut u8, src: usize, dst: usize, len: usize) {
        if len <= 64 {
            let mut s = ptr.add(src);
            let mut d = ptr.add(dst);
            let mut rem = len;
            // Forward copy is safe because `dst < src` always holds here (`dst != src` and
            // `src - dst == shift - prefix_growth >= 0` in `unrle_in_place`).
            while rem >= 16 {
                let v = std::ptr::read_unaligned(s as *const u128);
                std::ptr::write_unaligned(d as *mut u128, v);
                s = s.add(16);
                d = d.add(16);
                rem -= 16;
            }
            if rem >= 8 {
                let v = std::ptr::read_unaligned(s as *const u64);
                std::ptr::write_unaligned(d as *mut u64, v);
                s = s.add(8);
                d = d.add(8);
                rem -= 8;
            }
            if rem >= 4 {
                let v = std::ptr::read_unaligned(s as *const u32);
                std::ptr::write_unaligned(d as *mut u32, v);
                s = s.add(4);
                d = d.add(4);
                rem -= 4;
            }
            while rem > 0 {
                *d = *s;
                s = s.add(1);
                d = d.add(1);
                rem -= 1;
            }
        } else {
            std::ptr::copy(ptr.add(src), ptr.add(dst), len);
        }
    }

    #[inline(always)]
    unsafe fn fill_run_fast(dst: *mut u8, c: u8, run: usize) {
        // `run >= 4` always (4 + extra).
        if run <= 32 {
            let c4 = u32::from_ne_bytes([c; 4]);
            if run == 4 {
                std::ptr::write_unaligned(dst as *mut u32, c4);
                return;
            }
            let c8 = (c4 as u64) | ((c4 as u64) << 32);
            if run <= 8 {
                std::ptr::write_unaligned(dst as *mut u32, c4);
                std::ptr::write_unaligned(dst.add(run - 4) as *mut u32, c4);
                return;
            }
            if run <= 16 {
                std::ptr::write_unaligned(dst as *mut u64, c8);
                std::ptr::write_unaligned(dst.add(run - 8) as *mut u64, c8);
                return;
            }
            let c16 = (c8 as u128) | ((c8 as u128) << 64);
            std::ptr::write_unaligned(dst as *mut u128, c16);
            std::ptr::write_unaligned(dst.add(run - 16) as *mut u128, c16);
        } else {
            std::ptr::write_bytes(dst, c, run);
        }
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P3_m1_64b_inline(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = buf.len();
        if n > u32::MAX as usize {
            std::mem::swap(buf, stage);
            buf.clear();
            rle1_probe_U2_m1_64b_raw(stage, buf)?;
            return Ok(crc::crc32(buf, k));
        }
        let (grown, shift) = find_runs_m1_64b_stats(buf, runs)?;
        let Some(&(first, _, _)) = runs.first() else {
            return Ok(crc::crc32(buf, k));
        };
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        if shift > n {
            std::mem::swap(buf, stage);
            buf.clear();
            rle1_probe_U2_m1_64b_raw(stage, buf)?;
            return Ok(crc::crc32(buf, k));
        }
        let first = first as usize;
        if shift > 0 {
            buf.reserve_exact(shift);
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let mut next_tile = TILE;
        while first >= next_tile {
            s = crc::update_with(s, &buf[at..next_tile], k);
            at = next_tile;
            next_tile += TILE;
        }
        let base_ptr = buf.as_mut_ptr();
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                let clen = from - src;
                if dst != src {
                    copy_small_or_memmove(base_ptr, src, dst, clen);
                }
                dst += clen;
            }
            let run = 4 + extra as usize;
            fill_run_fast(base_ptr.add(dst), c, run);
            dst += run;
            src = from + 5;
            if dst >= next_tile {
                while dst >= next_tile {
                    let tile = std::slice::from_raw_parts(base_ptr.add(at), TILE);
                    s = crc::update_with(s, tile, k);
                    at = next_tile;
                    next_tile += TILE;
                }
            }
        }
        if n + shift > src && dst != src {
            copy_small_or_memmove(base_ptr, src, dst, n + shift - src);
        }
        buf.truncate(len);
        s = crc::update_with(s, &buf[at..], k);
        Ok(!s)
    }

    #[inline(never)]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_P4_m1_128b_inline(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = buf.len();
        if n > u32::MAX as usize {
            std::mem::swap(buf, stage);
            buf.clear();
            rle1_probe_U2_m1_64b_raw(stage, buf)?;
            return Ok(crc::crc32(buf, k));
        }
        rle1_probe_F3_m1_128b(buf, runs)?;
        let Some(&(first, _, _)) = runs.first() else {
            return Ok(crc::crc32(buf, k));
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        if shift > n {
            std::mem::swap(buf, stage);
            buf.clear();
            rle1_probe_U2_m1_64b_raw(stage, buf)?;
            return Ok(crc::crc32(buf, k));
        }
        let first = first as usize;
        if shift > 0 {
            buf.reserve_exact(shift);
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let mut next_tile = TILE;
        while first >= next_tile {
            s = crc::update_with(s, &buf[at..next_tile], k);
            at = next_tile;
            next_tile += TILE;
        }
        let base_ptr = buf.as_mut_ptr();
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                let clen = from - src;
                if dst != src {
                    copy_small_or_memmove(base_ptr, src, dst, clen);
                }
                dst += clen;
            }
            let run = 4 + extra as usize;
            fill_run_fast(base_ptr.add(dst), c, run);
            dst += run;
            src = from + 5;
            if dst >= next_tile {
                while dst >= next_tile {
                    let tile = std::slice::from_raw_parts(base_ptr.add(at), TILE);
                    s = crc::update_with(s, tile, k);
                    at = next_tile;
                    next_tile += TILE;
                }
            }
        }
        if n + shift > src && dst != src {
            copy_small_or_memmove(base_ptr, src, dst, n + shift - src);
        }
        buf.truncate(len);
        s = crc::update_with(s, &buf[at..], k);
        Ok(!s)
    }

    // -----------------------------------------------------------------------
    // Family C: Out-of-place `unrle` AVX2 kernels (zero memcpy/memset/vzeroupper on hot loop)
    // -----------------------------------------------------------------------

    #[cold]
    #[inline(never)]
    fn grow_out(out: &mut Vec<u8>, cur_len: usize, need: usize) -> (*mut u8, usize) {
        // SAFETY: caller has initialized `out[..cur_len]`.
        unsafe { out.set_len(cur_len) };
        out.reserve(need);
        (out.as_mut_ptr(), out.capacity())
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn avx2_fill_run(dst: *mut u8, c: u8, run: usize) {
        // `run >= 4`, `run <= 259`, caller guarantees `run + 32` writable headroom in `out`!
        let vc = _mm256_set1_epi8(c as i8);
        if run <= 32 {
            _mm256_storeu_si256(dst as *mut __m256i, vc);
            return;
        }
        let mut k = 0usize;
        while k + 32 <= run {
            _mm256_storeu_si256(dst.add(k) as *mut __m256i, vc);
            k += 32;
        }
        if k < run {
            _mm256_storeu_si256(dst.add(run - 32) as *mut __m256i, vc);
        }
    }

    /// U1: 2-load 1-`vpcmpeqb` `m1` filter, 32B stride, direct AVX2 stores + inline AVX2 fill.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_U1_m1_32b_raw(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
        let n = rle.len();
        out.reserve(n + 320);
        let p = rle.as_ptr();
        let mut out_ptr = out.as_mut_ptr();
        let mut out_cap = out.capacity();
        let mut out_len = out.len();
        let mut i = 0usize;
        // 35 bytes needed to test 32 start positions [i..i+32] with zero carry complexity:
        // v0 = p[i..i+32], v1 = p[i+1..i+33], tail 2 bits from p[i+32]==p[i+33], p[i+33]==p[i+34]
        // OR simply advance by 30 bytes per 2-load window (still 2 loads / 30B vs 4 loads / 32B)!
        // Better: carry 2 bits (`prev`) cleanly by delaying emitting the last 2 bytes of each
        // chunk? Even cleaner: 2-load 30-byte stride has zero carry and directly emits 30B via
        // 32B unaligned store (`out_cap` always has >= 320 B headroom!). Let's use 30B stride here
        // and 62B stride in U2 so we never have to back-patch `out_len`!
        while i + 33 <= n {
            if out_cap - out_len < 320 {
                let (np, nc) = grow_out(out, out_len, (n - i) + 320);
                out_ptr = np;
                out_cap = nc;
            }
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32;
            // Bit k (0..30) of `tri` is (b[i+k]==b[i+k+1]==b[i+k+2]==b[i+k+3]).
            let tri = m & (m >> 1) & (m >> 2);
            if tri == 0 {
                _mm256_storeu_si256(out_ptr.add(out_len) as *mut __m256i, v0);
                out_len += 30;
                i += 30;
                continue;
            }
            let off = tri.trailing_zeros() as usize;
            let j = i + off;
            if off > 0 {
                _mm256_storeu_si256(out_ptr.add(out_len) as *mut __m256i, v0);
                out_len += off;
            }
            let Some(&extra) = rle.get(j + 4) else {
                out.set_len(out_len);
                return Err(ErrorKind::RunUnfinished);
            };
            let run = 4 + extra as usize;
            avx2_fill_run(out_ptr.add(out_len), *p.add(j), run);
            out_len += run;
            i = j + 5;
        }
        out.set_len(out_len);
        rle::unrle_scalar(rle, out, i)
    }

    /// U2: 2-load 1-`vpcmpeqb` `m1` filter, 62-byte stride (`u64` mask), 2× AVX2 stores,
    /// inline AVX2 run fill, and multi-hit drain inside the same 64B chunk!
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1")]
    #[allow(non_snake_case)]
    unsafe fn rle1_probe_U2_m1_64b_raw(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
        let n = rle.len();
        out.reserve(n + 512);
        let p = rle.as_ptr();
        let mut out_ptr = out.as_mut_ptr();
        let mut out_cap = out.capacity();
        let mut out_len = out.len();
        let mut i = 0usize;
        while i + 65 <= n {
            if out_cap - out_len < 384 {
                let (np, nc) = grow_out(out, out_len, (n - i) + 512);
                out_ptr = np;
                out_cap = nc;
            }
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let m0 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v0, v1)) as u32 as u64;
            let v2 = _mm256_loadu_si256(p.add(i + 32) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 33) as *const __m256i);
            let m1 = _mm256_movemask_epi8(_mm256_cmpeq_epi8(v2, v3)) as u32 as u64;
            let m = m0 | (m1 << 32);
            // Bits 0..62 of `tri` are exact 4-equal-byte starts in `[i .. i+62)`.
            let tri = m & (m >> 1) & (m >> 2);
            if tri == 0 {
                _mm256_storeu_si256(out_ptr.add(out_len) as *mut __m256i, v0);
                _mm256_storeu_si256(out_ptr.add(out_len + 32) as *mut __m256i, v2);
                out_len += 62;
                i += 62;
                continue;
            }
            // Drain all runs whose start lies in `[i .. i+62)`!
            let mut mask = tri;
            let mut cur = i;
            let mut next_i = i + 62;
            while mask != 0 {
                if out_cap - out_len < 384 {
                    let (np, nc) = grow_out(out, out_len, (n - cur) + 512);
                    out_ptr = np;
                    out_cap = nc;
                }
                let tz = mask.trailing_zeros() as usize;
                let j = i + tz;
                let lit = j - cur;
                if lit > 0 {
                    // Copy `lit` bytes (`lit <= 61`): 1 or 2 unaligned 32B stores (headroom >= 384).
                    let a = _mm256_loadu_si256(p.add(cur) as *const __m256i);
                    _mm256_storeu_si256(out_ptr.add(out_len) as *mut __m256i, a);
                    if lit > 32 {
                        let b = _mm256_loadu_si256(p.add(cur + 32) as *const __m256i);
                        _mm256_storeu_si256(out_ptr.add(out_len + 32) as *mut __m256i, b);
                    }
                    out_len += lit;
                }
                let Some(&extra) = rle.get(j + 4) else {
                    out.set_len(out_len);
                    return Err(ErrorKind::RunUnfinished);
                };
                let run = 4 + extra as usize;
                avx2_fill_run(out_ptr.add(out_len), *p.add(j), run);
                out_len += run;
                cur = j + 5;
                let skip = tz + 5;
                if skip >= 62 {
                    next_i = cur;
                    break;
                }
                mask &= !((1u64 << skip) - 1);
            }
            if cur < next_i {
                let lit = next_i - cur;
                if out_cap - out_len < 96 {
                    let (np, nc) = grow_out(out, out_len, (n - cur) + 512);
                    out_ptr = np;
                    out_cap = nc;
                }
                let a = _mm256_loadu_si256(p.add(cur) as *const __m256i);
                _mm256_storeu_si256(out_ptr.add(out_len) as *mut __m256i, a);
                if lit > 32 {
                    let b = _mm256_loadu_si256(p.add(cur + 32) as *const __m256i);
                    _mm256_storeu_si256(out_ptr.add(out_len + 32) as *mut __m256i, b);
                }
                out_len += lit;
            }
            i = next_i;
        }
        out.set_len(out_len);
        rle::unrle_scalar(rle, out, i)
    }

    // -----------------------------------------------------------------------
    // Corpus extraction (pre-RLE1 inverted BWT blocks + end-to-end 1W timing)
    // -----------------------------------------------------------------------

    struct Block {
        pre_rle: Vec<u8>,
        want_out: Vec<u8>,
        want_crc: u32,
    }

    struct Corpus {
        name: String,
        bz2_bytes: Vec<u8>,
        blocks: Vec<Block>,
        mult: usize,
        total_pre_bytes: usize,
        total_out_bytes: usize,
        total_runs: usize,
        shift_gt_0_blks: usize,
        stage_swap_blks: usize,
    }

    fn extract_corpus(name: &str, bz2_bytes: Vec<u8>, mult: usize, k: Kernels) -> Corpus {
        let mut sc = Scanner::new(SliceSource(&bz2_bytes));
        let mut blocks = Vec::new();
        let mut syms = Vec::new();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let mut tt = bwt::TVector::default();
        let mut runs_scratch = Vec::new();
        let mut total_pre_bytes = 0usize;
        let mut total_out_bytes = 0usize;
        let mut total_runs = 0usize;
        let mut shift_gt_0_blks = 0usize;
        let mut stage_swap_blks = 0usize;

        while let Event::Block { symbols, .. } = sc.next(syms).expect("scan error") {
            let n = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf failed");
            bwt::build(&ll[..n], &counts, &mut tt);
            let mut pre_rle = Vec::with_capacity(n);
            bwt::walk(&tt, symbols.orig_ptr, &mut pre_rle);
            let mut want_out = Vec::new();
            rle::unrle(&pre_rle, &mut want_out, k).expect("unrle failed");
            let want_crc = crc::crc32(&want_out, k);
            assert_eq!(want_crc, symbols.crc, "CRC mismatch extracting {name}");

            unsafe { rle1_probe_F0_shipped_4load(&pre_rle, &mut runs_scratch).unwrap() };
            let (mut grown, mut shift) = (0isize, 0isize);
            for &(_, _, extra) in &runs_scratch {
                grown += extra as isize - 1;
                shift = shift.max(grown);
            }
            total_runs += runs_scratch.len();
            if shift > 0 {
                shift_gt_0_blks += 1;
            }
            if shift as usize > pre_rle.len() {
                stage_swap_blks += 1;
            }
            total_pre_bytes += pre_rle.len();
            total_out_bytes += want_out.len();
            blocks.push(Block {
                pre_rle,
                want_out,
                want_crc,
            });
            syms = symbols.syms;
        }
        Corpus {
            name: name.to_string(),
            bz2_bytes,
            blocks,
            mult,
            total_pre_bytes,
            total_out_bytes,
            total_runs,
            shift_gt_0_blks,
            stage_swap_blks,
        }
    }

    // -----------------------------------------------------------------------
    // Exhaustive exactness verification
    // -----------------------------------------------------------------------

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    fn verify_one_input(rle_in: &[u8], k: Kernels, checks: &mut usize) {
        let mut ref_out = Vec::new();
        let ref_res = rle::unrle(rle_in, &mut ref_out, Kernels::scalar());
        let ref_crc = ref_res.clone().map(|()| crc::crc32(&ref_out, k));
        let mut ref_runs = Vec::new();
        let ref_runs_res = find_runs_scalar(rle_in, &mut ref_runs, 0);
        assert_eq!(ref_res.is_ok(), ref_runs_res.is_ok());

        // Family A: find_runs
        type FindRunsFn = unsafe fn(&[u8], &mut Vec<Run>) -> Result<(), ErrorKind>;
        let f_arms: [(&str, FindRunsFn); 6] = [
            ("F0_shipped_4load", rle1_probe_F0_shipped_4load),
            ("F1_m1_32b", rle1_probe_F1_m1_32b),
            ("F2_m1_64b", rle1_probe_F2_m1_64b),
            ("F3_m1_128b", rle1_probe_F3_m1_128b),
            ("F4_m1_64b_aligned", rle1_probe_F4_m1_64b_aligned),
            ("F5_1load_alignr_64b", rle1_probe_F5_1load_alignr_64b),
        ];
        let mut runs = Vec::new();
        for (name, f) in f_arms {
            let got = unsafe { f(rle_in, &mut runs) };
            assert_eq!(
                got,
                ref_runs_res,
                "{name} error mismatch on len {}",
                rle_in.len()
            );
            if got.is_ok() {
                assert_eq!(
                    runs,
                    ref_runs,
                    "{name} runs mismatch on len {}",
                    rle_in.len()
                );
            }
            *checks += 1;
        }

        // Family B: unrle_in_place
        type InPlaceFn =
            unsafe fn(&mut Vec<u8>, &mut Vec<Run>, &mut Vec<u8>, Kernels) -> Result<u32, ErrorKind>;
        let p_arms: [(&str, InPlaceFn); 6] = [
            ("P0_shipped", rle1_probe_P0_shipped),
            ("P1_m1_64b", rle1_probe_P1_m1_64b),
            ("P2_m1_128b", rle1_probe_P2_m1_128b),
            ("P3_m1_64b_inline", rle1_probe_P3_m1_64b_inline),
            ("P4_m1_128b_inline", rle1_probe_P4_m1_128b_inline),
            ("P5_m1_128b_u2stage", rle1_probe_P5_m1_128b_u2stage),
        ];
        let mut stage = Vec::new();
        for (name, f) in p_arms {
            let mut buf = rle_in.to_vec();
            runs.clear();
            stage.clear();
            let got = unsafe { f(&mut buf, &mut runs, &mut stage, k) };
            assert_eq!(
                got,
                ref_crc,
                "{name} crc/err mismatch on len {}",
                rle_in.len()
            );
            if got.is_ok() {
                assert_eq!(
                    buf,
                    ref_out,
                    "{name} bytes mismatch on len {}",
                    rle_in.len()
                );
            }
            *checks += 1;
        }

        // Family C: out-of-place unrle
        type UnrleFn = unsafe fn(&[u8], &mut Vec<u8>) -> Result<(), ErrorKind>;
        let u_arms: [(&str, UnrleFn); 2] = [
            ("U1_m1_32b_raw", rle1_probe_U1_m1_32b_raw),
            ("U2_m1_64b_raw", rle1_probe_U2_m1_64b_raw),
        ];
        for (name, f) in u_arms {
            let mut out = Vec::new();
            let got = unsafe { f(rle_in, &mut out) };
            assert_eq!(got, ref_res, "{name} err mismatch on len {}", rle_in.len());
            if got.is_ok() {
                assert_eq!(
                    out,
                    ref_out,
                    "{name} bytes mismatch on len {}",
                    rle_in.len()
                );
            }
            *checks += 1;
        }
    }

    fn run_exactness_suite(corpora: &[Corpus], k: Kernels) -> usize {
        let mut checks = 0usize;

        // 1. miri_edge_runs + seam cases
        let edge_cases: Vec<Vec<u8>> = vec![
            vec![],
            b"a".to_vec(),
            b"aa".to_vec(),
            b"aaa".to_vec(),
            b"aaaa".to_vec(),
            b"aaaa\0".to_vec(),
            b"aaaa\x01".to_vec(),
            b"aaaa\xff".to_vec(),
            b"xaaaa".to_vec(),
            b"aaaa\0bbbb\0cccc\0".to_vec(),
            b"aaaa\xffbbbb\0cccc\x07z".to_vec(),
            b"aaaaa".to_vec(),
            b"aaaaaaaaa".to_vec(),
            b"aaaaaaaaaa".to_vec(),
            [b"q".repeat(31), b"aaaa\x05".to_vec(), b"q".repeat(40)].concat(),
            [b"q".repeat(30), b"rrrr\0".to_vec(), b"s".repeat(3)].concat(),
            [b"0123".repeat(100), b"zzzz".to_vec()].concat(),
            [b"0123".repeat(100), b"zzzz\0".to_vec()].concat(),
            // 64B and 128B seam unfinished runs
            [b"0123".repeat(15), b"xyzzzz".to_vec()].concat(),
            [b"0123".repeat(31), b"xyzzzz".to_vec()].concat(),
            [b"0123".repeat(32), b"zzzz".to_vec()].concat(),
        ];
        for c in &edge_cases {
            verify_one_input(c, k, &mut checks);
        }

        // 2. Runs at every offset 0..=130 across 32B/64B/128B boundaries, lengths 4..=40, short & long tails
        for tail_len in [0usize, 1, 4, 31, 70] {
            for len in [4usize, 5, 6, 8, 9, 10, 32, 37] {
                for at in 0..=130 {
                    let mut v: Vec<u8> = (0..at).map(|i| (i % 7) as u8 + b'a').collect();
                    v.extend(std::iter::repeat(b'#').take(len));
                    v.extend((0..tail_len).map(|i| (i % 5) as u8 + b'A'));
                    verify_one_input(&v, k, &mut checks);
                }
            }
        }

        // 3. Consecutive 2-run interference matrix (extra1 == c1 or extra1 == c2) at offsets 0..=68
        for at in 0..=68 {
            for extra1 in [0u8, 1, b'a', b'b', 255] {
                for extra2 in [0u8, 1, 255] {
                    let mut v: Vec<u8> = (0..at).map(|i| (i % 7) as u8 + b'0').collect();
                    v.extend_from_slice(&[b'a', b'a', b'a', b'a', extra1]);
                    v.extend_from_slice(&[b'b', b'b', b'b', b'b', extra2]);
                    v.extend((0..40).map(|i| (i % 5) as u8 + b'A'));
                    verify_one_input(&v, k, &mut checks);
                }
            }
        }

        // 4. Tile boundaries 64 KiB * 1..=3 +/- 2 B
        for tiles in 1..=3usize {
            for d in [-2isize, -1, 0, 1, 2] {
                let want = (tiles * TILE) as isize + d;
                for extra in [0u8, 1, 3] {
                    let mut v: Vec<u8> = (0..10_240).map(|i| (i % 251) as u8).collect();
                    let mut out_len = v.len() as isize;
                    while out_len + 4 + (extra as isize) < want {
                        v.extend([b'r'; 4]);
                        v.push(extra);
                        out_len += 4 + extra as isize;
                        v.push(b'.');
                        out_len += 1;
                    }
                    while out_len < want {
                        v.push(b'.');
                        out_len += 1;
                    }
                    verify_one_input(&v, k, &mut checks);
                }
            }
        }

        // 5. 400 seeded random blocks (small, medium, tile-edge, shrinking/growing/stage-swap)
        let mut r = Rng(0x9e37_79b9_7f4a_7c15);
        for round in 0..400 {
            let target = match round % 4 {
                0 => r.below(300) as usize,
                1 => r.below(70_000) as usize,
                2 => (1 << 16) * (1 + r.below(2) as usize) + r.below(64) as usize - 32,
                _ => r.below(2 * TILE as u64) as usize,
            };
            let density = 1 + r.below(200);
            let max_extra = [0u64, 1, 2, 8, 255][r.below(5) as usize];
            let mut v = Vec::with_capacity(target + 8);
            while v.len() < target {
                if r.below(density) == 0 {
                    let c = r.below(4) as u8;
                    v.extend([c; 4]);
                    v.push(r.below(max_extra + 1) as u8);
                } else {
                    v.push(r.below(256) as u8);
                }
            }
            verify_one_input(&v, k, &mut checks);
        }

        // 6. Every real extracted block across all corpora (Wikidata + 7 guard corpora)
        for corpus in corpora {
            for blk in &corpus.blocks {
                verify_one_input(&blk.pre_rle, k, &mut checks);
            }
        }

        checks
    }

    // -----------------------------------------------------------------------
    // Stats & formatting helpers
    // -----------------------------------------------------------------------

    fn stats(samples: &[f64]) -> (f64, f64, f64) {
        let mut s = samples.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        let med = s[s.len() / 2];
        (med, s[0], s[s.len() - 1])
    }

    fn tag(base: (f64, f64, f64), cand: (f64, f64, f64)) -> &'static str {
        if cand.2 < base.1 {
            "DISJ_FASTER"
        } else if cand.1 > base.2 {
            "DISJ_SLOWER"
        } else {
            "overlap"
        }
    }

    fn pct(base_med: f64, cand_med: f64) -> f64 {
        (cand_med / base_med - 1.0) * 100.0
    }

    // -----------------------------------------------------------------------
    // Benchmark runners
    // -----------------------------------------------------------------------

    fn bench_find_runs(corpus: &Corpus, repeats: usize, cold: bool, scrubber: &mut [u8]) {
        type FindRunsFn = unsafe fn(&[u8], &mut Vec<Run>) -> Result<(), ErrorKind>;
        let arms: [(&str, FindRunsFn); 6] = [
            ("F0_shipped_4load", rle1_probe_F0_shipped_4load),
            ("F1_m1_32b", rle1_probe_F1_m1_32b),
            ("F2_m1_64b", rle1_probe_F2_m1_64b),
            ("F3_m1_128b", rle1_probe_F3_m1_128b),
            ("F4_m1_64b_aligned", rle1_probe_F4_m1_64b_aligned),
            ("F5_1load_alignr_64b", rle1_probe_F5_1load_alignr_64b),
        ];
        let mut times = vec![Vec::with_capacity(repeats); arms.len()];
        let mut runs = Vec::with_capacity(32_768);
        let max_blk = corpus
            .blocks
            .iter()
            .map(|b| b.pre_rle.len())
            .max()
            .unwrap_or(0);
        let mut hot = Vec::with_capacity(max_blk + 64);

        for rep in 0..=repeats {
            let order: Vec<usize> = if rep % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for idx in order {
                let (_, f) = arms[idx];
                if cold {
                    flush_l3(scrubber);
                }
                let mut total_ns = 0u64;
                if cold {
                    let t0 = thread_cpu_ns();
                    for _ in 0..corpus.mult {
                        for blk in &corpus.blocks {
                            unsafe { f(&blk.pre_rle, &mut runs).unwrap() };
                            black_box(runs.len());
                        }
                    }
                    total_ns = thread_cpu_ns() - t0;
                } else {
                    for _ in 0..corpus.mult {
                        for blk in &corpus.blocks {
                            hot.clear();
                            hot.extend_from_slice(&blk.pre_rle);
                            let t0 = thread_cpu_ns();
                            unsafe { f(&hot, &mut runs).unwrap() };
                            total_ns += thread_cpu_ns() - t0;
                            black_box(runs.len());
                        }
                    }
                }
                if rep > 0 {
                    times[idx].push(total_ns as f64 / 1e6);
                }
            }
        }
        let mb = (corpus.total_pre_bytes * corpus.mult) as f64 / 1e6;
        let base = stats(&times[0]);
        let regime = if cold { "L3-cold" } else { "L3-warm" };
        println!(
            "\n--- Family A `find_runs` | {} ({:.1} MB, {}) ---",
            corpus.name, mb, regime
        );
        for (idx, (name, _)) in arms.iter().enumerate() {
            let st = stats(&times[idx]);
            let gbps = mb / st.0;
            let p = pct(base.0, st.0);
            let t = if idx == 0 { "base" } else { tag(base, st) };
            println!(
                "  {:<22} med {:7.2} ms [{:7.2}..{:7.2}]  {:6.2} GB/s  {:+6.2}%  {}",
                name, st.0, st.1, st.2, gbps, p, t
            );
        }
    }

    fn bench_in_place(
        corpus: &Corpus,
        repeats: usize,
        cold: bool,
        scrubber: &mut [u8],
        k: Kernels,
    ) -> (f64, f64, f64) {
        type InPlaceFn =
            unsafe fn(&mut Vec<u8>, &mut Vec<Run>, &mut Vec<u8>, Kernels) -> Result<u32, ErrorKind>;
        let arms: [(&str, InPlaceFn); 6] = [
            ("P0_shipped", rle1_probe_P0_shipped),
            ("P1_m1_64b", rle1_probe_P1_m1_64b),
            ("P2_m1_128b", rle1_probe_P2_m1_128b),
            ("P3_m1_64b_inline", rle1_probe_P3_m1_64b_inline),
            ("P4_m1_128b_inline", rle1_probe_P4_m1_128b_inline),
            ("P5_m1_128b_u2stage", rle1_probe_P5_m1_128b_u2stage),
        ];
        let mut times = vec![Vec::with_capacity(repeats); arms.len()];
        let mut runs = Vec::with_capacity(32_768);
        let mut stage = Vec::with_capacity(1 << 20);
        let max_cap = corpus
            .blocks
            .iter()
            .map(|b| prod_pool_cap(b.pre_rle.len()).max(b.want_out.len()))
            .max()
            .unwrap_or(0);
        let mut hot = Vec::with_capacity(max_cap);
        let mut cold_bufs: Vec<Vec<u8>> = if cold {
            corpus
                .blocks
                .iter()
                .map(|b| Vec::with_capacity(prod_pool_cap(b.pre_rle.len()).max(b.want_out.len())))
                .collect()
        } else {
            Vec::new()
        };

        for rep in 0..=repeats {
            let order: Vec<usize> = if rep % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for idx in order {
                let (_, f) = arms[idx];
                let mut total_ns = 0u64;
                if cold {
                    for _ in 0..corpus.mult {
                        for (dst, blk) in cold_bufs.iter_mut().zip(&corpus.blocks) {
                            dst.clear();
                            dst.extend_from_slice(&blk.pre_rle);
                        }
                        flush_l3(scrubber);
                        let t0 = thread_cpu_ns();
                        for (dst, blk) in cold_bufs.iter_mut().zip(&corpus.blocks) {
                            let c = unsafe { f(dst, &mut runs, &mut stage, k).unwrap() };
                            debug_assert_eq!(c, blk.want_crc);
                            black_box(c);
                        }
                        total_ns += thread_cpu_ns() - t0;
                    }
                } else {
                    for _ in 0..corpus.mult {
                        for blk in &corpus.blocks {
                            hot.clear();
                            hot.extend_from_slice(&blk.pre_rle);
                            let t0 = thread_cpu_ns();
                            let c = unsafe { f(&mut hot, &mut runs, &mut stage, k).unwrap() };
                            total_ns += thread_cpu_ns() - t0;
                            debug_assert_eq!(c, blk.want_crc);
                            black_box(c);
                        }
                    }
                }
                if rep > 0 {
                    times[idx].push(total_ns as f64 / 1e6);
                }
            }
        }
        let mb = (corpus.total_out_bytes * corpus.mult) as f64 / 1e6;
        let base = stats(&times[0]);
        let regime = if cold { "L3-cold" } else { "L3-warm" };
        println!(
            "\n--- Family B `unrle_in_place` (A9) | {} ({:.1} MB, {}) ---",
            corpus.name, mb, regime
        );
        let mut best_med = base.0;
        for (idx, (name, _)) in arms.iter().enumerate() {
            let st = stats(&times[idx]);
            if idx > 0 && st.0 < best_med {
                best_med = st.0;
            }
            let gbps = mb / st.0;
            let p = pct(base.0, st.0);
            let t = if idx == 0 { "base" } else { tag(base, st) };
            println!(
                "  {:<22} med {:7.2} ms [{:7.2}..{:7.2}]  {:6.2} GB/s  {:+6.2}%  {}",
                name, st.0, st.1, st.2, gbps, p, t
            );
        }
        (base.0, stats(&times[5]).0, best_med)
    }

    fn bench_unrle_oop(
        corpus: &Corpus,
        repeats: usize,
        cold: bool,
        scrubber: &mut [u8],
        k: Kernels,
    ) {
        let arms: [&str; 3] = ["U0_shipped", "U1_m1_32b_raw", "U2_m1_64b_raw"];
        let mut times = vec![Vec::with_capacity(repeats); arms.len()];
        let max_out = corpus
            .blocks
            .iter()
            .map(|b| b.want_out.len())
            .max()
            .unwrap_or(0);
        let mut out = Vec::with_capacity(max_out + 512);
        let max_pre = corpus
            .blocks
            .iter()
            .map(|b| b.pre_rle.len())
            .max()
            .unwrap_or(0);
        let mut hot = Vec::with_capacity(max_pre + 64);

        for rep in 0..=repeats {
            let order: Vec<usize> = if rep % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for idx in order {
                if cold {
                    flush_l3(scrubber);
                }
                let mut total_ns = 0u64;
                if cold {
                    let t0 = thread_cpu_ns();
                    for _ in 0..corpus.mult {
                        for blk in &corpus.blocks {
                            out.clear();
                            match idx {
                                0 => rle::unrle(&blk.pre_rle, &mut out, k).unwrap(),
                                1 => unsafe {
                                    rle1_probe_U1_m1_32b_raw(&blk.pre_rle, &mut out).unwrap()
                                },
                                _ => unsafe {
                                    rle1_probe_U2_m1_64b_raw(&blk.pre_rle, &mut out).unwrap()
                                },
                            }
                            black_box(out.len());
                        }
                    }
                    total_ns = thread_cpu_ns() - t0;
                } else {
                    for _ in 0..corpus.mult {
                        for blk in &corpus.blocks {
                            hot.clear();
                            hot.extend_from_slice(&blk.pre_rle);
                            out.clear();
                            let t0 = thread_cpu_ns();
                            match idx {
                                0 => rle::unrle(&hot, &mut out, k).unwrap(),
                                1 => unsafe { rle1_probe_U1_m1_32b_raw(&hot, &mut out).unwrap() },
                                _ => unsafe { rle1_probe_U2_m1_64b_raw(&hot, &mut out).unwrap() },
                            }
                            total_ns += thread_cpu_ns() - t0;
                            black_box(out.len());
                        }
                    }
                }
                if rep > 0 {
                    times[idx].push(total_ns as f64 / 1e6);
                }
            }
        }
        let mb = (corpus.total_out_bytes * corpus.mult) as f64 / 1e6;
        let base = stats(&times[0]);
        let regime = if cold { "L3-cold" } else { "L3-warm" };
        println!(
            "\n--- Family C `unrle` (out-of-place) | {} ({:.1} MB, {}) ---",
            corpus.name, mb, regime
        );
        for (idx, name) in arms.iter().enumerate() {
            let st = stats(&times[idx]);
            let gbps = mb / st.0;
            let p = pct(base.0, st.0);
            let t = if idx == 0 { "base" } else { tag(base, st) };
            println!(
                "  {:<22} med {:7.2} ms [{:7.2}..{:7.2}]  {:6.2} GB/s  {:+6.2}%  {}",
                name, st.0, st.1, st.2, gbps, p, t
            );
        }
    }

    fn bench_full_1w_decode(corpus: &Corpus, repeats: usize) -> f64 {
        let mut times = Vec::with_capacity(repeats);
        let mut sink = vec![0u8; 1 << 20];
        for rep in 0..=repeats {
            let t0 = thread_cpu_ns();
            let mut total = 0usize;
            for _ in 0..corpus.mult {
                let mut rdr = BzReader::new(corpus.bz2_bytes.as_slice());
                loop {
                    let n = rdr.read(&mut sink).unwrap();
                    if n == 0 {
                        break;
                    }
                    total += n;
                }
            }
            let dt = (thread_cpu_ns() - t0) as f64 / 1e6;
            black_box(total);
            if rep > 0 {
                times.push(dt);
            }
        }
        stats(&times).0
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut wiki_path: Option<String> = None;
        let mut repeats = 7usize;
        let mut skip_bench = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--repeats" => {
                    i += 1;
                    repeats = args[i].parse().expect("invalid --repeats");
                }
                "--skip-bench" => skip_bench = true,
                p => wiki_path = Some(p.to_string()),
            }
            i += 1;
        }
        let k = Kernels::detect();
        assert!(k.avx2(), "AVX2 required for rle1_dissection");

        println!("=== RLE1 Dissection Harness (Kernels={k:?}, repeats={repeats}) ===");
        let mut corpora = Vec::new();
        if let Some(p) = wiki_path.as_deref() {
            let path = Path::new(p);
            let bytes = std::fs::read(path).expect("failed to read wikidata.bz2");
            corpora.push(extract_corpus("wikidata_x4", bytes, 4, k));
        }
        // Standard `ab gen` corpora at scale 4, scaled so each sweep is >= 40 MB
        for (raw_name, raw_data) in common::corpora(4) {
            let name = raw_name.replace(' ', "");
            let bz2 = common::compress(&raw_data, 9);
            let mult = match name.as_str() {
                "onebyte" => 4,
                "runs" => 8,
                _ => 4,
            };
            corpora.push(extract_corpus(&name, bz2, mult, k));
        }
        // Synthetic `zeros` (20 MiB of 0x00)
        {
            let z = vec![0u8; 20 << 20];
            let bz2 = common::compress(&z, 9);
            corpora.push(extract_corpus("zeros", bz2, 4, k));
        }

        println!("\n=== Corpus RLE1 Census ===");
        println!(
            "  {:<12} {:>5} {:>6} {:>9} {:>9} {:>9} {:>8} {:>7}",
            "corpus", "blks", "mult", "pre_MB", "out_MB", "runs/pass", "shift>0", "stage"
        );
        for c in &corpora {
            println!(
                "  {:<12} {:>5} {:>6} {:>9.2} {:>9.2} {:>9} {:>8} {:>7}",
                c.name,
                c.blocks.len(),
                c.mult,
                c.total_pre_bytes as f64 / 1e6,
                c.total_out_bytes as f64 / 1e6,
                c.total_runs,
                c.shift_gt_0_blks,
                c.stage_swap_blks
            );
        }

        let checks = run_exactness_suite(&corpora, k);
        println!("\nEXACTNESS: {checks}/{checks} bit- & CRC- & error-exact across all 13 arms!");
        if skip_bench {
            return;
        }

        let mut scrubber = vec![0u8; 64 << 20];
        println!("\n=== End-to-End 1W Decode Share & Component Benchmarks ===");
        for c in &corpora {
            let full_1w_ms = bench_full_1w_decode(c, 5);
            // Warm + Cold for all 3 families on wikidata_x4; Warm + Cold on guards for B & C, Warm for A
            bench_find_runs(c, repeats, false, &mut scrubber);
            if c.name == "wikidata_x4" || c.name == "pattern" || c.name == "text" {
                bench_find_runs(c, repeats, true, &mut scrubber);
            }
            let (a9_warm_base, a9_warm_p3, a9_warm_best) =
                bench_in_place(c, repeats, false, &mut scrubber, k);
            let (a9_cold_base, a9_cold_p3, a9_cold_best) =
                bench_in_place(c, repeats, true, &mut scrubber, k);
            bench_unrle_oop(c, repeats, false, &mut scrubber, k);
            if c.name == "wikidata_x4"
                || c.name == "onebyte"
                || c.name == "runs"
                || c.name == "zeros"
                || c.name == "pattern"
                || c.name == "text"
            {
                bench_unrle_oop(c, repeats, true, &mut scrubber, k);
            }
            let share_pct = 100.0 * a9_warm_base / full_1w_ms;
            let e2e_p3_pct = 100.0 * (a9_warm_p3 - a9_warm_base) / full_1w_ms;
            let e2e_best_pct = 100.0 * (a9_warm_best - a9_warm_base) / full_1w_ms;
            println!(
                "\n>>> E2E SHARE [{}]: full_1W={:.2} ms | A9_warm={:.2} ms ({:.2}% of 1W) | P5_warm={:+.2}% A9 ({:+.2}% 1W) | best_warm={:+.2}% A9 ({:+.2}% 1W) | P5_cold={:+.2}% A9 | best_cold={:+.2}% A9",
                c.name,
                full_1w_ms,
                a9_warm_base,
                share_pct,
                pct(a9_warm_base, a9_warm_p3),
                e2e_p3_pct,
                pct(a9_warm_base, a9_warm_best),
                e2e_best_pct,
                pct(a9_cold_base, a9_cold_p3),
                pct(a9_cold_base, a9_cold_best),
            );
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    x86::run();
}
