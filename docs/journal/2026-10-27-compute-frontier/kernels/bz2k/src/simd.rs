//! SIMD kernels for the bzip2 decode's last phase, with runtime dispatch and a scalar
//! fallback:
//!
//! - `crc32_bzip2_clmul`: CRC-32/BZIP2 by carry-less multiplication, 4 x 128-bit lanes
//!   folded (PCLMULQDQ, SSE4.1; every x86-64 laptop since 2010; aarch64 has PMULL), the
//!   last 64 bytes and the remainder by the sliced table;
//! - `unrle_avx2`: RLE1 undone with the 4-equal-bytes test made 32 positions at a time
//!   (AVX2), the stretches between runs copied whole.
use crate::crc;

/// x^n mod P (non-reflected, P = x^32 + POLY), as a 32-bit remainder
fn xpow_mod(n: u32) -> u64 {
    // r = x^n mod P, bit by bit
    let mut r: u64 = 1;
    for _ in 0..n {
        r <<= 1;
        if r & (1 << 32) != 0 {
            r ^= (1 << 32) | crc::POLY as u64;
        }
    }
    r
}

pub fn crc32_bzip2(data: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("pclmulqdq") && std::is_x86_feature_detected!("sse4.1") && data.len() >= 128 {
            return unsafe { !clmul_update(!0, data) };
        }
    }
    crc::crc32_bzip2(data, crc::Kind::Slice8)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn clmul_update(state: u32, data: &[u8]) -> u32 {
    use std::arch::x86_64::*;
    use std::sync::OnceLock;
    // fold constants: a lane 512 bits on, x^(512+64) and x^512 (mod P); with x^32 shifts
    // because the lane's value is taken as a polynomial of 128 bits
    static K: OnceLock<(u64, u64)> = OnceLock::new();
    let &(k_hi, k_lo) = K.get_or_init(|| (xpow_mod(512 + 64), xpow_mod(512)));
    // byte order: the first byte is the highest-degree coefficient
    let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let ld = |p: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(p as *const __m128i), rev);
    let n = data.len();
    let blocks = n / 64; // >= 2
    let p = data.as_ptr();
    let mut x0 = ld(p);
    let mut x1 = ld(p.add(16));
    let mut x2 = ld(p.add(32));
    let mut x3 = ld(p.add(48));
    // the initial state goes into the top 32 bits of the first lane
    x0 = _mm_xor_si128(x0, _mm_set_epi32(state as i32, 0, 0, 0));
    let k = _mm_set_epi64x(k_hi as i64, k_lo as i64);
    let fold = |x: __m128i, y: __m128i| {
        // x * x^512 = x_hi * x^(512+64) + x_lo * x^512 (mod P), then + y
        let h = _mm_clmulepi64_si128(x, k, 0x11);
        let l = _mm_clmulepi64_si128(x, k, 0x00);
        _mm_xor_si128(_mm_xor_si128(h, l), y)
    };
    for b in 1..blocks {
        let q = p.add(b * 64);
        x0 = fold(x0, ld(q));
        x1 = fold(x1, ld(q.add(16)));
        x2 = fold(x2, ld(q.add(32)));
        x3 = fold(x3, ld(q.add(48)));
    }
    // the four lanes are the last 64 bytes' worth, congruent mod P: run them through the
    // table from state 0, then the remainder
    let mut tail = [0u8; 64];
    for (i, x) in [x0, x1, x2, x3].into_iter().enumerate() {
        let mut v = [0u8; 16];
        _mm_storeu_si128(v.as_mut_ptr() as *mut __m128i, _mm_shuffle_epi8(x, rev));
        tail[i * 16..i * 16 + 16].copy_from_slice(&v);
    }
    let s = crc::update(0, &tail, crc::Kind::Slice8);
    crc::update(s, &data[blocks * 64..], crc::Kind::Slice8)
}

/// RLE1 undone (as lib::unrle_crc's loop), appended to `out`
pub fn unrle(rle: &[u8], out: &mut Vec<u8>) {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            return unsafe { unrle_avx2(rle, out) };
        }
    }
    unrle_scalar(rle, out, 0)
}

pub fn unrle_scalar(rle: &[u8], out: &mut Vec<u8>, mut i: usize) {
    let n = rle.len();
    while i < n {
        let c = rle[i];
        if i + 3 < n && rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
            out.extend_from_slice(&[c, c, c, c]);
            let extra = if i + 4 < n { rle[i + 4] as usize } else { 0 };
            out.resize(out.len() + extra, c);
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1")]
unsafe fn unrle_avx2(rle: &[u8], out: &mut Vec<u8>) {
    use std::arch::x86_64::*;
    let n = rle.len();
    out.reserve(n + 64);
    let p = rle.as_ptr();
    let mut i = 0;
    // whole 32-position windows while 35 bytes remain
    while i + 35 <= n {
        let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
        let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
        let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
        let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
        let eq = _mm256_and_si256(_mm256_cmpeq_epi8(v0, v1), _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)));
        let m = _mm256_movemask_epi8(eq) as u32;
        if m == 0 {
            out.extend_from_slice(std::slice::from_raw_parts(p.add(i), 32));
            i += 32;
            continue;
        }
        let j = i + m.trailing_zeros() as usize; // the first run's start
        out.extend_from_slice(std::slice::from_raw_parts(p.add(i), j - i));
        let c = *p.add(j);
        out.extend_from_slice(&[c, c, c, c]);
        let extra = if j + 4 < n { *p.add(j + 4) as usize } else { 0 };
        out.resize(out.len() + extra, c);
        i = j + 5;
    }
    unrle_scalar(rle, out, i)
}

/// Phase 2 (lib::mtf) with the move-to-front of an index below 32 made as one AVX2
/// blend of the list and the list shifted by a byte (the list kept with a byte before
/// it, so the shifted list is one unaligned load); deeper indices as the scalar does
pub fn mtf(b: &crate::Block, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            return unsafe { mtf_avx2(b, ll, counts) };
        }
    }
    crate::mtf(b, ll, counts)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_avx2(b: &crate::Block, ll: &mut Vec<u8>, counts: &mut [u32; 256]) -> usize {
    use std::arch::x86_64::*;
    ll.clear();
    ll.reserve(900_000 + 64);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], a byte of slack before it
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                               23, 24, 25, 26, 27, 28, 29, 30, 31);
    let s2u = b.seq_to_unseq.as_ptr();
    let mut run: u32 = 0;
    let mut run_bit: u32 = 0;
    let mut out = ll.as_mut_ptr();
    let start = out;
    for &s in &b.syms {
        if s <= 1 {
            run += (s as u32 + 1) << run_bit;
            run_bit += 1;
            continue;
        }
        if run_bit != 0 {
            let c = *s2u.add(*base as usize);
            counts[c as usize] += run;
            std::ptr::write_bytes(out, c, run as usize);
            out = out.add(run as usize);
            run = 0;
            run_bit = 0;
        }
        if s == b.eob {
            break;
        }
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
        let c = *s2u.add(v as usize);
        counts[c as usize] += 1;
        *out = c;
        out = out.add(1);
        assert!(out.offset_from(start) as usize <= 900_000);
    }
    let n = out.offset_from(start) as usize;
    ll.set_len(n);
    n
}
