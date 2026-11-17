//! Phase 5's RLE1 undone: 4 equal bytes, then a byte counting 0-255 more of them. Scalar,
//! or (AVX2) the test for 4 equal bytes made at 32 positions at once, the stretches
//! between runs copied whole.

use super::crc::Running;
use super::{ErrorKind, Kernels};

/// `rle` with its runs undone, appended to `out`. Err if it ends with a run's 4 bytes and
/// no count: libbzip2 then reads a count past the block's end and refuses the block
pub fn unrle(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<(), ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant)
            return unsafe { unrle_avx2(rle, out) };
        }
    }
    let _ = k;
    unrle_scalar(rle, out, 0)
}

/// `unrle` with the CRC of the bytes it appends folded in as it goes (`crc::Running`), every
/// `crc::FOLD` bytes while they are still in cache: RLE1 and the block's CRC in one pass over
/// the output, not a second one reading it back. Returns that CRC, `crc32(&out[start..])`
/// for `out` `start` bytes long before; on Err, `out` holds what was appended so far, as
/// `unrle` leaves it.
pub fn unrle_crc(rle: &[u8], out: &mut Vec<u8>, k: Kernels) -> Result<u32, ErrorKind> {
    let mut crc = Running::new(out.len());
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant)
            unsafe { unrle_avx2_folding(rle, out, &mut crc, k)? };
            return Ok(crc.finish(out, k));
        }
    }
    unrle_scalar(rle, out, 0)?;
    Ok(crc.finish(out, k))
}

/// From `rle[i..]` (i at a run's start or a literal, never inside a run)
pub fn unrle_scalar(rle: &[u8], out: &mut Vec<u8>, mut i: usize) -> Result<(), ErrorKind> {
    let n = rle.len();
    out.reserve(n - i.min(n));
    while i < n {
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            let Some(&extra) = rle.get(i + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
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
unsafe fn unrle_avx2(rle: &[u8], out: &mut Vec<u8>) -> Result<(), ErrorKind> {
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
        let Some(&extra) = rle.get(j + 4) else {
            return Err(ErrorKind::RunUnfinished);
        };
        out.extend_from_slice(&[c, c, c, c]);
        out.resize(out.len() + extra as usize, c);
        i = j + 5;
    }
    unrle_scalar(rle, out, i)
}

/// `unrle_avx2`, with `crc` folding in the output as it grows (`unrle_crc`); the scalar tail
/// (under 35 bytes) is left for `crc`'s finish
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_avx2_folding(
    rle: &[u8],
    out: &mut Vec<u8>,
    crc: &mut Running,
    k: Kernels,
) -> Result<(), ErrorKind> {
    use std::arch::x86_64::*;
    let n = rle.len();
    out.reserve(n + 64);
    let p = rle.as_ptr();
    let mut i = 0;
    // whole 32-position windows while 35 bytes remain (SAFETY: every load is below i + 35)
    while i + 35 <= n {
        crc.fold(out, k);
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
        let Some(&extra) = rle.get(j + 4) else {
            return Err(ErrorKind::RunUnfinished);
        };
        out.extend_from_slice(&[c, c, c, c]);
        out.resize(out.len() + extra as usize, c);
        i = j + 5;
    }
    unrle_scalar(rle, out, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An RLE1 stream of `len` symbols or so: literals, and runs of 4 with counts 0-255
    fn coded(x: &mut u64, len: usize) -> Vec<u8> {
        let mut v = Vec::new();
        let mut next = || {
            *x ^= *x << 13;
            *x ^= *x >> 7;
            *x ^= *x << 17;
            *x
        };
        while v.len() < len {
            let r = next();
            let c = (r >> 8) as u8 % 6;
            if r % 5 == 0 {
                v.extend_from_slice(&[c; 4]);
                v.push((r >> 16) as u8);
            } else if v.len() < 3 || v[v.len() - 3..].iter().any(|&b| b != c) {
                v.push(c);
            } else {
                v.push(c.wrapping_add(1) % 6);
            }
        }
        v
    }

    /// `unrle_crc` is `unrle` then `crc32` of what it appended: the same bytes, the same
    /// CRC, the same error, across folds and after bytes already in `out`
    #[test]
    fn unrle_crc_is_unrle_then_crc32() {
        let mut x = 0x5eed_u64;
        for k in [Kernels::scalar(), Kernels::detect()] {
            for len in [0usize, 1, 34, 35, 36, 100, 4096, 40_000, 100_000, 300_000] {
                let rle = coded(&mut x, len);
                let (mut a, mut b) = (vec![9u8; 7], vec![9u8; 7]);
                let want = unrle(&rle, &mut a, k).map(|()| crate::crc::crc32(&a[7..], k));
                let got = unrle_crc(&rle, &mut b, k);
                assert_eq!(got, want, "{len} {k:?}");
                assert!(a == b, "{len} {k:?}");
                // a run's four bytes with no count at the end (a byte that is not the
                // stream's last, so they start a run of their own)
                let mut cut = rle.clone();
                let c = cut.last().map_or(0, |&b| b.wrapping_add(1));
                cut.extend_from_slice(&[c; 4]);
                let (mut a, mut b) = (Vec::new(), Vec::new());
                assert_eq!(unrle(&cut, &mut a, k), Err(ErrorKind::RunUnfinished));
                assert_eq!(unrle_crc(&cut, &mut b, k), Err(ErrorKind::RunUnfinished));
            }
        }
    }
}
