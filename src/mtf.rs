//! Phase 2: RUNA/RUNB runs and move-to-front undone, into the BWT's last column (`ll`)
//! and its byte counts, with libbzip2's checks: a run's weight below 2^21, and the block
//! no longer than its stream's level allows. The scalar loop, or (AVX2) the move of an
//! index below 32, nearly all of them, as one blend of the list and the list a byte on.

use super::block::Symbols;
use super::{Kernels, ErrorKind};

/// Undo the runs and the move-to-front of `b`'s symbols into `ll` (cleared), counting
/// each byte in `counts`. Returns the block's length.
pub fn mtf(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256], k: Kernels) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { mtf_avx2(b, ll, counts) };
        }
    }
    let _ = k;
    mtf_scalar(b, ll, counts)
}

/// A run's weight (N in libbzip2) refused at this, so its length cannot overflow
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

pub fn mtf_scalar(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    ll.clear();
    ll.reserve(b.max_len);
    *counts = [0; 256];
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
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
            if ll.len() + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            let c = b.seq_to_unseq[list[0] as usize];
            counts[c as usize] += run;
            ll.resize(ll.len() + run as usize, c);
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if ll.len() >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = (s - 1) as usize;
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        counts[c as usize] += 1;
        ll.push(c);
    }
    Ok(ll.len())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_avx2(b: &Symbols, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], with slack before it for the shifted load
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31,
    );
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max`, which
    // `ll`'s capacity holds
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                result = Err(ErrorKind::RunTooLong);
                break;
            }
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            if n + run as usize > max {
                result = Err(ErrorKind::BlockTooLong);
                break;
            }
            let c = s2u[*base as usize];
            counts[c as usize] += run;
            std::ptr::write_bytes(out.add(n), c, run as usize);
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= max {
            result = Err(ErrorKind::BlockTooLong);
            break;
        }
        // (s - 1 < 256: a symbol is below the alphabet, at most 257)
        let nn = (s - 1) as usize;
        let v = *base.add(nn);
        if nn < 32 {
            let old = _mm256_load_si256(base as *const __m256i);
            let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx); // i <= nn
            _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
        } else {
            std::ptr::copy(base, base.add(1), nn);
        }
        *base = v;
        let c = s2u[v as usize];
        counts[c as usize] += 1;
        *out.add(n) = c;
        n += 1;
    }
    // SAFETY: the first n bytes were written above, n <= max <= capacity
    ll.set_len(n);
    result.map(|()| n)
}
