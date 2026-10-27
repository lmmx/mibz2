//! Phase 2: RUNA/RUNB runs and move-to-front undone, into the BWT's last column (`ll`)
//! and its byte counts, with libbzip2's checks: a run's weight below 2^21, and the block
//! no longer than its stream's level allows. The scalar loop, or (AVX2) the list's first
//! 32 entries kept in two registers, so the move of an index below 32, nearly all of
//! them, is a shuffle or two with no memory touched.

use super::block::Symbols;
use super::{ErrorKind, Kernels};

/// Undo the runs and the move-to-front of `b`'s symbols into `ll` (cleared), counting
/// each byte in `counts`. Returns the block's length.
pub fn mtf(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    k: Kernels,
) -> Result<usize, ErrorKind> {
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

pub fn mtf_scalar(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
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

/// The shuffles that move entry `k` (below 16) of a 16-entry head to its front: entry 0
/// takes entry k, entries 1..=k the one before, the rest stay
#[cfg(target_arch = "x86_64")]
#[repr(align(16))]
struct Moves([[u8; 16]; 16]);

#[cfg(target_arch = "x86_64")]
static MOVES: Moves = {
    let mut m = [[0u8; 16]; 16];
    let mut k = 0;
    while k < 16 {
        let mut i = 0;
        while i < 16 {
            m[k][i] = if i == 0 {
                k as u8
            } else if i <= k {
                (i - 1) as u8
            } else {
                i as u8
            };
            i += 1;
        }
        k += 1;
    }
    Moves(m)
};

/// The list's first 32 entries in two registers (`h0`, entries 0-15; `h1`, 16-31), the
/// rest in memory. An index below 16 is one shuffle of `h0`; below 32, a shuffle and two
/// byte-aligns across both; past that, the memory's entries move (one blend of 32 bytes
/// below 64, nearly all of them, else a copy) and each register takes one byte across.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_avx2(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max + 32);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list's entries 32-255 at store[32..256], with slack before them for the
    // shifted load and after them for a 32-byte store
    for i in 32..256 {
        store.0[i] = i as u8;
    }
    let tail = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let idx16 = _mm256_castsi256_si128(idx);
    let mut h0 = idx16;
    let mut h1 = _mm_add_epi8(idx16, _mm_set1_epi8(16));
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max`, which
    // `ll`'s capacity holds with 32 bytes over
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
            let c = s2u[_mm_cvtsi128_si32(h0) as u8 as usize];
            counts[c as usize] += run;
            // 32 bytes a store, the last store's excess in the capacity's slack (a call
            // to memset would cost more than nearly every run's bytes, and spill h0, h1)
            let fill = _mm256_set1_epi8(c as i8);
            let mut i = 0;
            while i < run as usize {
                _mm256_storeu_si256(out.add(n + i) as *mut __m256i, fill);
                i += 32;
            }
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
        if nn < 16 {
            h0 = _mm_shuffle_epi8(h0, _mm_load_si128(MOVES.0[nn].as_ptr() as *const __m128i));
        } else if nn < 32 {
            let k = (nn - 16) as i8;
            // entry k of h1 in every byte; h1's entries 0..=k a byte on, h0's last first
            let v = _mm_shuffle_epi8(h1, _mm_set1_epi8(k));
            let on = _mm_alignr_epi8(h1, h0, 15);
            let m = _mm_cmpgt_epi8(_mm_set1_epi8(k + 1), idx16); // i <= k
            h1 = _mm_blendv_epi8(h1, on, m);
            h0 = _mm_alignr_epi8(h0, v, 15);
        } else {
            let k = nn - 32;
            let v = *tail.add(k);
            if k < 32 {
                let old = _mm256_load_si256(tail as *const __m256i);
                let shifted = _mm256_loadu_si256(tail.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(k as i8 + 1), idx); // i <= k
                _mm256_store_si256(tail as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            } else {
                std::ptr::copy(tail, tail.add(1), k);
            }
            *tail = _mm_extract_epi8(h1, 15) as u8;
            h1 = _mm_alignr_epi8(h1, h0, 15);
            h0 = _mm_alignr_epi8(h0, _mm_set1_epi8(v as i8), 15);
        }
        let c = s2u[_mm_cvtsi128_si32(h0) as u8 as usize];
        counts[c as usize] += 1;
        *out.add(n) = c;
        n += 1;
    }
    // SAFETY: the first n bytes were written above, n <= max < capacity
    ll.set_len(n);
    result.map(|()| n)
}
