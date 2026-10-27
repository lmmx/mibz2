//! Phase 5's RLE1 undone: 4 equal bytes, then a byte counting 0-255 more of them. Scalar,
//! or (AVX2) the test for 4 equal bytes made at 32 positions at once, the stretches
//! between runs copied whole.

use super::{Kernels, Kind};

/// `rle` with its runs undone, appended to `out`. Err if it ends with a run's 4 bytes and
/// no count: libbzip2 then reads a count past the block's end and refuses the block
pub fn unrle(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<(), Kind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { unrle_avx2(rle, out) };
        }
    }
    let _ = k;
    unrle_scalar(rle, out, 0)
}

/// From `rle[i..]` (i at a run's start or a literal, never inside a run)
pub fn unrle_scalar(rle: &[u8], out: &mut Vec<u8>, mut i: usize) -> Result<(), Kind> {
    let n = rle.len();
    out.reserve(n - i.min(n));
    while i < n {
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else { return Err(Kind::RunUnfinished) };
            out.extend_from_slice(&[c, c, c, c]);
            out.resize(out.len() + extra as usize, c);
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_avx2(rle: &[u8], out: &mut Vec<u8>) -> Result<(), Kind> {
    use std::arch::x86_64::*;
    let n = rle.len();
    out.reserve(n + 64);
    let p = rle.as_ptr();
    let mut i = 0;
    // whole 32-position windows while 35 bytes remain (SAFETY: every load is below i + 35)
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
            out.extend_from_slice(&rle[i..i + 32]);
            i += 32;
            continue;
        }
        let j = i + m.trailing_zeros() as usize; // the first run's start
        out.extend_from_slice(&rle[i..j]);
        let c = rle[j];
        let Some(&extra) = rle.get(j + 4) else { return Err(Kind::RunUnfinished) };
        out.extend_from_slice(&[c, c, c, c]);
        out.resize(out.len() + extra as usize, c);
        i = j + 5;
    }
    unrle_scalar(rle, out, i)
}
