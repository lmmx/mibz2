//! CRC-32/BZIP2 (polynomial 0x04C11DB7, MSB first, initial value and final xor all ones),
//! the block CRC bzip2 stores: sliced by 16 (portable), or by carry-less multiplication
//! (PCLMULQDQ with SSE4.1/SSSE3, and 8-slot ROB-drained AVX2 `vpshufb` ring + 8-lane
//! `vpclmulqdq` + zero-table Barrett reduction on AVX2+PCLMULQDQ, dispatched at run time,
//! see `Kernels`).

use super::Kernels;

pub const POLY: u32 = 0x04C1_1DB7;

const fn tables() -> [[u32; 256]; 16] {
    let mut t = [[0u32; 256]; 16];
    let mut i = 0;
    while i < 256 {
        let mut c = (i as u32) << 24;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x8000_0000 != 0 {
                (c << 1) ^ POLY
            } else {
                c << 1
            };
            k += 1;
        }
        t[0][i] = c;
        i += 1;
    }
    let mut j = 1;
    while j < 16 {
        let mut i = 0;
        while i < 256 {
            let p = t[j - 1][i];
            t[j][i] = (p << 8) ^ t[0][(p >> 24) as usize];
            i += 1;
        }
        j += 1;
    }
    t
}

static TABLES: [[u32; 256]; 16] = tables();

/// The CRC of `data`, by the kernels given
pub fn crc32(data: &[u8], k: Kernels) -> u32 {
    !update_with(!0, data, k)
}

/// The CRC register after `data`, from `s` (no initial or final xor), by the kernels given:
/// `update`'s register, so a CRC taken in pieces, each from the register the one before
/// left, is the CRC of the whole (`!update_with(update_with(!0, a, k), b, k)` is the CRC of
/// `a` then `b`)
pub fn update_with(s: u32, data: &[u8], k: Kernels) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if k.clmul {
            if data.is_empty() {
                return s;
            }
            if k.avx2 && data.len() >= 1280 {
                // SAFETY: `k.avx2` guarantees AVX2 + BMI1 (implying AVX, SSE4.1, SSSE3, SSE2)
                // and `k.clmul` guarantees PCLMULQDQ + SSE4.1 + SSSE3 (a `Kernels` invariant);
                // `data.len() >= 1280` is checked immediately above.
                return unsafe { avx2_ring8_update(s, data) };
            }
            if k.avx2 && data.len() >= 128 {
                // SAFETY: `k.avx2 && k.clmul` guarantees AVX + PCLMULQDQ + SSE4.1 + SSSE3 + SSE2
                // (a `Kernels` invariant); `data.len() >= 128` is checked immediately above.
                return unsafe { vex_8x_barrett_update(s, data) };
            }
            // SAFETY: `k.clmul` guarantees PCLMULQDQ + SSE4.1 + SSSE3 + SSE2 (a `Kernels`
            // invariant).
            return unsafe { sse_barrett_update(s, data) };
        }
    }
    let _ = k;
    update(s, data)
}

/// The CRC register after `data`, from `s` (no initial or final xor): a byte at a time
/// as bzip2 takes it, 16 at a time where it can (then 8 once, then bytes, for the rest).
/// The 16 lookups of a slice are combined as a balanced tree of XORs, 4 deep, not as one
/// chain of 15
pub fn update(mut s: u32, data: &[u8]) -> u32 {
    let t = &TABLES;
    let mut chunks = data.chunks_exact(16);
    for c in &mut chunks {
        let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        let a = (t[15][(hi >> 24) as usize] ^ t[14][((hi >> 16) & 0xff) as usize])
            ^ (t[13][((hi >> 8) & 0xff) as usize] ^ t[12][(hi & 0xff) as usize]);
        let b = (t[11][c[4] as usize] ^ t[10][c[5] as usize])
            ^ (t[9][c[6] as usize] ^ t[8][c[7] as usize]);
        let d = (t[7][c[8] as usize] ^ t[6][c[9] as usize])
            ^ (t[5][c[10] as usize] ^ t[4][c[11] as usize]);
        let e = (t[3][c[12] as usize] ^ t[2][c[13] as usize])
            ^ (t[1][c[14] as usize] ^ t[0][c[15] as usize]);
        s = (a ^ b) ^ (d ^ e);
    }
    let rem = chunks.remainder();
    let tail = if rem.len() >= 8 {
        let c = &rem[..8];
        let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        s = (t[7][(hi >> 24) as usize] ^ t[6][((hi >> 16) & 0xff) as usize])
            ^ (t[5][((hi >> 8) & 0xff) as usize] ^ t[4][(hi & 0xff) as usize])
            ^ (t[3][c[4] as usize] ^ t[2][c[5] as usize])
            ^ (t[1][c[6] as usize] ^ t[0][c[7] as usize]);
        &rem[8..]
    } else {
        rem
    };
    for &b in tail {
        s = (s << 8) ^ t[0][((s >> 24) as u8 ^ b) as usize];
    }
    s
}

/// A byte at a time, by the one table: the reference the others are tested against
pub fn update_bytewise(mut s: u32, data: &[u8]) -> u32 {
    for &b in data {
        s = (s << 8) ^ TABLES[0][((s >> 24) as u8 ^ b) as usize];
    }
    s
}

/// `x^n mod P` (non-reflected, `P(x) = x^32 + POLY`), as a 32-bit remainder in `u64`.
#[cfg(any(target_arch = "x86_64", test))]
const fn xpow_mod(n: u32) -> u64 {
    let mut r: u64 = 1;
    let mut i = 0;
    while i < n {
        r <<= 1;
        if r & (1 << 32) != 0 {
            r ^= (1 << 32) | POLY as u64;
        }
        i += 1;
    }
    r
}

/// Barrett quotient constant `mu = floor(x^64 / P(x))` in GF(2)[x] (degree 32, 33 bits).
#[cfg(any(target_arch = "x86_64", test))]
const fn barrett_mu() -> u64 {
    const P_FULL: u64 = (1u64 << 32) | POLY as u64;
    let mut rem: u64 = 1u64 << 32;
    let mut q: u64 = 0;
    let mut k = 0;
    while k <= 32 {
        if rem & (1u64 << 32) != 0 {
            q |= 1u64 << (32 - k);
            rem ^= P_FULL;
        }
        rem <<= 1;
        k += 1;
    }
    q
}

#[cfg(target_arch = "x86_64")]
const K128_HI: u64 = xpow_mod(128 + 64);
#[cfg(target_arch = "x86_64")]
const K128_LO: u64 = xpow_mod(128);
#[cfg(target_arch = "x86_64")]
const K256_HI: u64 = xpow_mod(256 + 64);
#[cfg(target_arch = "x86_64")]
const K256_LO: u64 = xpow_mod(256);
#[cfg(target_arch = "x86_64")]
const K512_HI: u64 = xpow_mod(512 + 64);
#[cfg(target_arch = "x86_64")]
const K512_LO: u64 = xpow_mod(512);
#[cfg(target_arch = "x86_64")]
const K1024_HI: u64 = xpow_mod(1024 + 64);
#[cfg(target_arch = "x86_64")]
const K1024_LO: u64 = xpow_mod(1024);
#[cfg(target_arch = "x86_64")]
const K96: u64 = xpow_mod(96);
#[cfg(target_arch = "x86_64")]
const K64: u64 = xpow_mod(64);
#[cfg(target_arch = "x86_64")]
const MU: u64 = barrett_mu();

/// 96-byte sliding shuffle/mask tables (1.5 cache lines vs 16 KiB for `TABLES`) for folding
/// `1..=15` trailing bytes into a 128-bit MSB-first CRC accumulator without touching `TABLES`.
#[cfg(target_arch = "x86_64")]
#[repr(align(64))]
struct TailTables {
    /// `[0..16, 0x80..0x80]`: slice `[16 - r .. 32 - r]` moves the top `r` bytes to lanes `0..r`
    /// and zeros lanes `r..16`.
    shr_top: [u8; 32],
    /// `[0x80..0x80, 0..16]`: slice `[16 - r .. 32 - r]` shifts the low `16 - r` bytes up by `r`
    /// bytes and zeros lanes `0..r`.
    shl_bot: [u8; 32],
    /// `[0xFF..0xFF, 0x00..0x00]`: slice `[16 - r .. 32 - r]` keeps lanes `0..r` and zeros rest.
    mask_lo: [u8; 32],
}

#[cfg(target_arch = "x86_64")]
static TAIL_TABLES: TailTables = TailTables {
    shr_top: [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    ],
    shl_bot: [
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
        0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ],
    mask_lo: [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

/// 8-slot, 64-byte-aligned L1d ring buffer (`8 * 128 B = 1024 B`). Keeping 256-bit `vpshufb`
/// stores 7 blocks (896 B, ~308 uops > 192 Haswell ROB and > 224 Comet Lake ROB) ahead of
/// 128-bit `vpxor` memory loads guarantees store-buffer drainage before consumption, avoiding
/// Intel's 16B-from-32B store-to-load forwarding stall while halving `pshufb` Port-5 uops.
#[cfg(target_arch = "x86_64")]
#[repr(align(64))]
#[allow(dead_code)]
struct Ring32B8([[std::arch::x86_64::__m256i; 4]; 8]);

/// Pure-register 64-bit -> 32-bit Barrett reduction (`U(x) mod P(x)`), 2 `pclmulqdq`s, 0 `TABLES`.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn barrett_64_to_32_sse(u_bits: u64) -> u32 {
    use std::arch::x86_64::*;
    let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
    let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
    let q_vec = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
    let p_vec = _mm_clmulepi64_si128(q_vec, mu_poly, 0x10);
    let p_lo = _mm_cvtsi128_si64(p_vec) as u32;
    (u_bits as u32) ^ p_lo
}

/// Pure-register 128-bit -> 32-bit MSB-first Barrett reduction (`X(x) * x^32 mod P(x)`),
/// 4 `pclmulqdq`s, 0 bytes of `TABLES` touched.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn reduce_128_to_32_sse(x: std::arch::x86_64::__m128i) -> u32 {
    use std::arch::x86_64::*;
    let k96_64 = _mm_set_epi64x(K64 as i64, K96 as i64);
    // Step 1: X_hi * x^96 + X_lo * x^32 -> 96-bit polynomial in bits 0..95.
    let t = _mm_clmulepi64_si128(x, k96_64, 0x01); // x.hi64 * K96
    let lo_shl32 = _mm_slli_si128::<4>(_mm_and_si128(x, _mm_set_epi64x(0, -1)));
    let r96 = _mm_xor_si128(t, lo_shl32);
    // Step 2: r96[64..95] * K64 ^ r96[0..63] -> 64-bit polynomial in bits 0..63.
    let t2 = _mm_clmulepi64_si128(r96, k96_64, 0x11); // r96.hi64 * K64
    let u64v = _mm_xor_si128(t2, r96);
    let u_bits = _mm_cvtsi128_si64(u64v) as u64;
    barrett_64_to_32_sse(u_bits)
}

#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx,pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn reduce_128_to_32_vex(x: std::arch::x86_64::__m128i) -> u32 {
    use std::arch::x86_64::*;
    let k96_64 = _mm_set_epi64x(K64 as i64, K96 as i64);
    let t = _mm_clmulepi64_si128(x, k96_64, 0x01);
    let lo_shl32 = _mm_slli_si128::<4>(_mm_and_si128(x, _mm_set_epi64x(0, -1)));
    let r96 = _mm_xor_si128(t, lo_shl32);
    let t2 = _mm_clmulepi64_si128(r96, k96_64, 0x11);
    let u64v = _mm_xor_si128(t2, r96);
    let u_bits = _mm_cvtsi128_si64(u64v) as u64;
    let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
    let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
    let q_vec = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
    let p_vec = _mm_clmulepi64_si128(q_vec, mu_poly, 0x10);
    let p_lo = _mm_cvtsi128_si64(p_vec) as u32;
    (u_bits as u32) ^ p_lo
}

/// Fold remaining 16-byte chunks and final `1..=15` bytes into `x` and Barrett reduce.
///
/// # Safety
/// `p` must be valid for reads of `len` bytes, `len >= 16`, and `off <= len`.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn finish_from_lane_sse(
    mut x: std::arch::x86_64::__m128i,
    p: *const u8,
    mut off: usize,
    len: usize,
    rev: std::arch::x86_64::__m128i,
) -> u32 {
    use std::arch::x86_64::*;
    let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
    while off + 16 <= len {
        // SAFETY: `off + 16 <= len` and `p` is valid for `len` bytes.
        let y = _mm_shuffle_epi8(_mm_loadu_si128(p.add(off) as *const __m128i), rev);
        let h = _mm_clmulepi64_si128(x, k128, 0x11);
        let l = _mm_clmulepi64_si128(x, k128, 0x00);
        x = _mm_xor_si128(_mm_xor_si128(h, l), y);
        off += 16;
    }
    let r = len - off;
    if r > 0 {
        // SAFETY: `len >= 16`, so `p.add(len - 16)` is in-bounds for 16 bytes; `1 <= r <= 15`,
        // so `idx = 16 - r` is in `1..=15` and `idx + 16 <= 31 < 32` for each 32-byte table.
        let raw_last = _mm_shuffle_epi8(_mm_loadu_si128(p.add(len - 16) as *const __m128i), rev);
        let idx = 16 - r;
        let m_shr = _mm_loadu_si128(TAIL_TABLES.shr_top.as_ptr().add(idx) as *const __m128i);
        let m_shl = _mm_loadu_si128(TAIL_TABLES.shl_bot.as_ptr().add(idx) as *const __m128i);
        let m_lo = _mm_loadu_si128(TAIL_TABLES.mask_lo.as_ptr().add(idx) as *const __m128i);
        let x_hi = _mm_shuffle_epi8(x, m_shr);
        let x_lo = _mm_shuffle_epi8(x, m_shl);
        let tail_lo = _mm_and_si128(raw_last, m_lo);
        let y = _mm_or_si128(x_lo, tail_lo);
        let h = _mm_clmulepi64_si128(x_hi, k128, 0x11);
        let l = _mm_clmulepi64_si128(x_hi, k128, 0x00);
        x = _mm_xor_si128(_mm_xor_si128(h, l), y);
    }
    reduce_128_to_32_sse(x)
}

/// VEX-encoded version of `finish_from_lane_sse` (128-bit XMM only, zero `vzeroupper`).
///
/// # Safety
/// `p` must be valid for reads of `len` bytes, `len >= 16`, and `off <= len`.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx,pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn finish_from_lane_vex(
    mut x: std::arch::x86_64::__m128i,
    p: *const u8,
    mut off: usize,
    len: usize,
    rev: std::arch::x86_64::__m128i,
) -> u32 {
    use std::arch::x86_64::*;
    let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
    while off + 16 <= len {
        // SAFETY: `off + 16 <= len` and `p` is valid for `len` bytes.
        let y = _mm_shuffle_epi8(_mm_loadu_si128(p.add(off) as *const __m128i), rev);
        let h = _mm_clmulepi64_si128(x, k128, 0x11);
        let l = _mm_clmulepi64_si128(x, k128, 0x00);
        x = _mm_xor_si128(_mm_xor_si128(h, l), y);
        off += 16;
    }
    let r = len - off;
    if r > 0 {
        // SAFETY: `len >= 16` and `1 <= r <= 15` (`idx = 16 - r` in `1..=15`, `idx + 16 <= 31`).
        let raw_last = _mm_shuffle_epi8(_mm_loadu_si128(p.add(len - 16) as *const __m128i), rev);
        let idx = 16 - r;
        let m_shr = _mm_loadu_si128(TAIL_TABLES.shr_top.as_ptr().add(idx) as *const __m128i);
        let m_shl = _mm_loadu_si128(TAIL_TABLES.shl_bot.as_ptr().add(idx) as *const __m128i);
        let m_lo = _mm_loadu_si128(TAIL_TABLES.mask_lo.as_ptr().add(idx) as *const __m128i);
        let x_hi = _mm_shuffle_epi8(x, m_shr);
        let x_lo = _mm_shuffle_epi8(x, m_shl);
        let tail_lo = _mm_and_si128(raw_last, m_lo);
        let y = _mm_or_si128(x_lo, tail_lo);
        let h = _mm_clmulepi64_si128(x_hi, k128, 0x11);
        let l = _mm_clmulepi64_si128(x_hi, k128, 0x00);
        x = _mm_xor_si128(_mm_xor_si128(h, l), y);
    }
    reduce_128_to_32_vex(x)
}

/// Parallel tree fold of 4 x 128-bit lanes -> 1 lane (2 sequential `pclmulqdq` latencies).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn fold4_to_1_sse(
    x0: std::arch::x86_64::__m128i,
    x1: std::arch::x86_64::__m128i,
    x2: std::arch::x86_64::__m128i,
    x3: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;
    let k256 = _mm_set_epi64x(K256_HI as i64, K256_LO as i64);
    let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
    let y2 = _mm_xor_si128(
        _mm_xor_si128(
            _mm_clmulepi64_si128(x0, k256, 0x11),
            _mm_clmulepi64_si128(x0, k256, 0x00),
        ),
        x2,
    );
    let y3 = _mm_xor_si128(
        _mm_xor_si128(
            _mm_clmulepi64_si128(x1, k256, 0x11),
            _mm_clmulepi64_si128(x1, k256, 0x00),
        ),
        x3,
    );
    _mm_xor_si128(
        _mm_xor_si128(
            _mm_clmulepi64_si128(y2, k128, 0x11),
            _mm_clmulepi64_si128(y2, k128, 0x00),
        ),
        y3,
    )
}

/// Parallel tree fold of 8 x 128-bit lanes -> 1 lane (3 sequential `vpclmulqdq` latencies).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx,pclmulqdq,sse4.1,ssse3,sse2")]
#[allow(clippy::too_many_arguments)]
unsafe fn fold8_to_1_vex(
    x0: std::arch::x86_64::__m128i,
    x1: std::arch::x86_64::__m128i,
    x2: std::arch::x86_64::__m128i,
    x3: std::arch::x86_64::__m128i,
    x4: std::arch::x86_64::__m128i,
    x5: std::arch::x86_64::__m128i,
    x6: std::arch::x86_64::__m128i,
    x7: std::arch::x86_64::__m128i,
) -> std::arch::x86_64::__m128i {
    use std::arch::x86_64::*;
    let k512 = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
    let k256 = _mm_set_epi64x(K256_HI as i64, K256_LO as i64);
    let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
    let f = |a: __m128i, b: __m128i, k: __m128i| {
        _mm_xor_si128(
            _mm_xor_si128(
                _mm_clmulepi64_si128(a, k, 0x11),
                _mm_clmulepi64_si128(a, k, 0x00),
            ),
            b,
        )
    };
    let z0 = f(x0, x4, k512);
    let z1 = f(x1, x5, k512);
    let z2 = f(x2, x6, k512);
    let z3 = f(x3, x7, k512);
    let y2 = f(z0, z2, k256);
    let y3 = f(z1, z3, k256);
    f(y2, y3, k128)
}

/// 128-bit SSE4.1 + PCLMULQDQ zero-table Barrett CRC-32/BZIP2 for all lengths `>= 1`:
/// zero `ymm`, zero `vzeroupper`, zero `TABLES` touches. Used for `1..128` on AVX2 CPUs and
/// for all non-empty slices on non-AVX2 PCLMULQDQ CPUs.
///
/// # Safety
///
/// The CPU must support PCLMULQDQ, SSE4.1, SSSE3 and SSE2 (`Kernels::clmul`). Any length
/// is in bounds, including 0 (the `len < 16` path reads no byte at or past `data.len()`).
#[cfg(target_arch = "x86_64")]
#[inline(never)]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn sse_barrett_update(state: u32, data: &[u8]) -> u32 {
    use std::arch::x86_64::*;
    let len = data.len();
    if len < 16 {
        // Compute `(state * x^(8*len) ^ M(x) * x^32) mod P(x)` with 0 bytes of `TABLES`.
        let mut m: u128 = 0;
        let mut i = 0;
        while i < len {
            // SAFETY: `i < len == data.len()`.
            m = (m << 8) | (*data.get_unchecked(i) as u128);
            i += 1;
        }
        if len < 4 {
            let u = ((state as u64) << (len * 8)) ^ ((m as u64) << 32);
            return barrett_64_to_32_sse(u);
        }
        let lo128 = m ^ ((state as u128) << ((len - 4) * 8));
        let x = _mm_set_epi64x((lo128 >> 64) as i64, lo128 as i64);
        return reduce_128_to_32_sse(x);
    }
    // byte order: the first byte is the highest-degree coefficient
    let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let p = data.as_ptr();
    let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
    if len < 64 {
        // SAFETY: `len >= 16`, so `p` is valid for 16 bytes.
        let x = _mm_xor_si128(ld(p), _mm_set_epi32(state as i32, 0, 0, 0));
        return finish_from_lane_sse(x, p, 16, len, rev);
    }
    let blocks = len / 64;
    // SAFETY: `blocks >= 1`, so `0..64` is within `data.len()`.
    let mut x0 = _mm_xor_si128(ld(p), _mm_set_epi32(state as i32, 0, 0, 0));
    let mut x1 = ld(p.add(16));
    let mut x2 = ld(p.add(32));
    let mut x3 = ld(p.add(48));
    let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
    let fold = |x: __m128i, y: __m128i| {
        let h = _mm_clmulepi64_si128(x, k, 0x11);
        let l = _mm_clmulepi64_si128(x, k, 0x00);
        _mm_xor_si128(_mm_xor_si128(h, l), y)
    };
    for b in 1..blocks {
        // SAFETY: `b * 64 + 64 <= blocks * 64 <= len`.
        let q = p.add(b * 64);
        x0 = fold(x0, ld(q));
        x1 = fold(x1, ld(q.add(16)));
        x2 = fold(x2, ld(q.add(32)));
        x3 = fold(x3, ld(q.add(48)));
    }
    let x = fold4_to_1_sse(x0, x1, x2, x3);
    finish_from_lane_sse(x, p, blocks * 64, len, rev)
}

/// 8-lane (128 B/iter) 128-bit VEX `vpclmulqdq` + zero-table Barrett for `128..1280` bytes on
/// AVX2+PCLMULQDQ CPUs: zero `ymm`, zero `vzeroupper`, zero `TABLES` touches.
///
/// # Safety
///
/// The CPU must support AVX, PCLMULQDQ, SSE4.1, SSSE3 and SSE2 (`Kernels::avx2 &&
/// Kernels::clmul`), and `data.len() >= 128`: the first 128-byte block is loaded
/// unconditionally, so a shorter slice would be read out of bounds.
#[cfg(target_arch = "x86_64")]
#[inline(never)]
#[target_feature(enable = "avx,pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn vex_8x_barrett_update(state: u32, data: &[u8]) -> u32 {
    use std::arch::x86_64::*;
    let len = data.len();
    debug_assert!(len >= 128);
    let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let p = data.as_ptr();
    let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
    let blocks = len / 128;
    // SAFETY: `len >= 128`, so `blocks >= 1` and `0..128` is within `data.len()`.
    let mut x0 = _mm_xor_si128(ld(p), _mm_set_epi32(state as i32, 0, 0, 0));
    let mut x1 = ld(p.add(16));
    let mut x2 = ld(p.add(32));
    let mut x3 = ld(p.add(48));
    let mut x4 = ld(p.add(64));
    let mut x5 = ld(p.add(80));
    let mut x6 = ld(p.add(96));
    let mut x7 = ld(p.add(112));
    let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
    let fold = |x: __m128i, y: __m128i| {
        let h = _mm_clmulepi64_si128(x, k, 0x11);
        let l = _mm_clmulepi64_si128(x, k, 0x00);
        _mm_xor_si128(_mm_xor_si128(h, l), y)
    };
    for b in 1..blocks {
        // SAFETY: `b * 128 + 128 <= blocks * 128 <= len`.
        let q = p.add(b * 128);
        x0 = fold(x0, ld(q));
        x1 = fold(x1, ld(q.add(16)));
        x2 = fold(x2, ld(q.add(32)));
        x3 = fold(x3, ld(q.add(48)));
        x4 = fold(x4, ld(q.add(64)));
        x5 = fold(x5, ld(q.add(80)));
        x6 = fold(x6, ld(q.add(96)));
        x7 = fold(x7, ld(q.add(112)));
    }
    let x = fold8_to_1_vex(x0, x1, x2, x3, x4, x5, x6, x7);
    finish_from_lane_vex(x, p, blocks * 128, len, rev)
}

/// 8-slot (1024 B) ROB-drained AVX2 `vpshufb` ring + 8-lane (128 B/iter) `vpclmulqdq` +
/// zero-table Barrett reduction for `data.len() >= 1280` on AVX2+PCLMULQDQ CPUs.
///
/// Reduces Port-5 uops from 12.0 (`8 pclmulqdq + 4 pshufb`) to 10.0 (`8 vpclmulqdq + 2 vpshufb
/// ymm`) per 64 bytes while keeping 256-bit stores 7 blocks (896 B) ahead of 128-bit `vpxor`
/// loads so stores retire and drain to L1d before consumption. Touches 0 bytes of `TABLES`.
///
/// # Safety
///
/// The CPU must support AVX2, PCLMULQDQ, SSE4.1, SSSE3 and SSE2 (`Kernels::avx2 &&
/// Kernels::clmul`), and `data.len() >= 1280`: the ring's pre-fill loads 8 blocks of 128
/// bytes and the main loop indexes `blocks - 7` with `blocks = len / 128 >= 10`, so a
/// shorter slice would be read out of bounds or underflow the index.
#[cfg(target_arch = "x86_64")]
#[inline(never)]
#[target_feature(enable = "avx2,pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn avx2_ring8_update(state: u32, data: &[u8]) -> u32 {
    use std::arch::x86_64::*;
    use std::mem::MaybeUninit;

    let len = data.len();
    debug_assert!(len >= 1280);
    let rev256 = _mm256_setr_epi8(
        15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6,
        5, 4, 3, 2, 1, 0,
    );
    let rev128 = _mm256_castsi256_si128(rev256);
    let p = data.as_ptr();
    let blocks = len / 128; // >= 10 since len >= 1280

    // Uninitialized 64-byte-aligned 1024-byte stack ring: all 8 slots (0..1024) are written in
    // the pre-fill loop below before any slot is read, avoiding redundant stack zero-fill.
    let mut ring = MaybeUninit::<Ring32B8>::uninit();
    let rptr = ring.as_mut_ptr() as *mut u8;

    let store_block = |src: *const u8, dst: *mut u8| {
        // SAFETY: callers guarantee `src..src+128` is within `data` and `dst..dst+128` is a
        // 32-byte-aligned slot inside `ring` (`Ring32B8` is 64-byte aligned, slots are 128 B).
        let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(src as *const __m256i), rev256);
        let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
        let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
        let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
        _mm256_store_si256(dst as *mut __m256i, w0);
        _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
        _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
        _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
    };

    // Pre-fill all 8 slots (blocks 0..8, bytes 0..1024 <= 1280 <= len).
    for s in 0..8 {
        store_block(p.add(s * 128), rptr.add(s * 128));
    }

    // SAFETY: slot 0 (`rptr..rptr+128`) was initialized above and is 64-byte aligned.
    let mut x0 = _mm_xor_si128(
        _mm_load_si128(rptr as *const __m128i),
        _mm_set_epi32(state as i32, 0, 0, 0),
    );
    let mut x1 = _mm_load_si128(rptr.add(16) as *const __m128i);
    let mut x2 = _mm_load_si128(rptr.add(32) as *const __m128i);
    let mut x3 = _mm_load_si128(rptr.add(48) as *const __m128i);
    let mut x4 = _mm_load_si128(rptr.add(64) as *const __m128i);
    let mut x5 = _mm_load_si128(rptr.add(80) as *const __m128i);
    let mut x6 = _mm_load_si128(rptr.add(96) as *const __m128i);
    let mut x7 = _mm_load_si128(rptr.add(112) as *const __m128i);

    let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
    let fold_mem = |x: __m128i, src: *const u8| {
        // SAFETY: `src` points to an initialized, 16-byte-aligned lane inside `ring`.
        let h = _mm_clmulepi64_si128(x, k, 0x11);
        let l = _mm_clmulepi64_si128(x, k, 0x00);
        let y = _mm_load_si128(src as *const __m128i);
        _mm_xor_si128(_mm_xor_si128(h, l), y)
    };

    // Steady-state loop (C2_avx2_ring8_8x, no software prefetcht0): hardware stride prefetching
    // handles sequential 128 B reads without consuming extra load-port uops on Zen 5 / Comet Lake.
    // For `1 <= b < blocks - 7`, `(b + 7) * 128 + 128 <= blocks * 128 <= len`, and slot
    // `(b + 7) & 7` was consumed at `b - 1` (7 blocks ahead of its next read).
    for b in 1..(blocks - 7) {
        let q = p.add((b + 7) * 128);
        store_block(q, rptr.add(((b + 7) & 7) * 128));
        let src = rptr.add((b & 7) * 128);
        x0 = fold_mem(x0, src);
        x1 = fold_mem(x1, src.add(16));
        x2 = fold_mem(x2, src.add(32));
        x3 = fold_mem(x3, src.add(48));
        x4 = fold_mem(x4, src.add(64));
        x5 = fold_mem(x5, src.add(80));
        x6 = fold_mem(x6, src.add(96));
        x7 = fold_mem(x7, src.add(112));
    }
    // Drain the final 7 pre-swapped blocks from the ring.
    for b in (blocks - 7)..blocks {
        let src = rptr.add((b & 7) * 128);
        x0 = fold_mem(x0, src);
        x1 = fold_mem(x1, src.add(16));
        x2 = fold_mem(x2, src.add(32));
        x3 = fold_mem(x3, src.add(48));
        x4 = fold_mem(x4, src.add(64));
        x5 = fold_mem(x5, src.add(80));
        x6 = fold_mem(x6, src.add(96));
        x7 = fold_mem(x7, src.add(112));
    }
    let x = fold8_to_1_vex(x0, x1, x2, x3, x4, x5, x6, x7);
    finish_from_lane_vex(x, p, blocks * 128, len, rev128)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kernels() -> Vec<Kernels> {
        let det = Kernels::detect();
        let mut ks = vec![Kernels::scalar()];
        let sse_only = det.restrict(false, true);
        if !ks.contains(&sse_only) {
            ks.push(sse_only);
        }
        if !ks.contains(&det) {
            ks.push(det);
        }
        ks
    }

    #[test]
    fn barrett_and_xpow_const_identities() {
        const P_FULL: u64 = (1u64 << 32) | POLY as u64;
        let mu = barrett_mu();
        let mut prod: u128 = 0;
        for i in 0..64 {
            if (P_FULL >> i) & 1 != 0 {
                prod ^= (mu as u128) << i;
            }
        }
        assert_eq!(prod >> 32, 1u128 << 32, "MU * P_FULL != x^64 + r(<32)");
        assert_eq!(xpow_mod(32), POLY as u64);
        let mut z = [0u8; 17];
        z[0] = 1;
        assert_eq!(update_bytewise(0, &z) as u64, xpow_mod(128 + 32));
    }

    /// Exercised directly by CI's `cargo miri test --lib -- miri_` workflow: checks GF(2)
    /// Barrett/fold constants, `Kernels::detect()` under Miri (scalar fallback), `update_with`,
    /// and streaming splits across every dispatch threshold.
    #[test]
    fn miri_crc_dispatch_and_constants() {
        let k = Kernels::detect();
        assert_eq!(crc32(b"123456789", k), 0xFC89_1918);
        let mut x = 0x517c_c1b7_2722_0a95u64;
        let data: Vec<u8> = (0..1409)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect();
        for n in [0usize, 1, 15, 16, 64, 127, 128, 1279, 1280, 1409] {
            let want = !update_bytewise(!0, &data[..n]);
            assert_eq!(crc32(&data[..n], k), want, "n={n}");
        }
        let s = update_with(update_with(!0, &data[..128], k), &data[128..], k);
        assert_eq!(!s, !update_bytewise(!0, &data));
        barrett_and_xpow_const_identities();
    }

    #[test]
    fn kernels_agree() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let data: Vec<u8> = (0..5000)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect();
        let mut ref_reg = Vec::with_capacity(5001);
        let mut cur = !0u32;
        ref_reg.push(cur);
        for &b in &data {
            cur = update_bytewise(cur, &[b]);
            ref_reg.push(cur);
        }
        for k in test_kernels() {
            // Under Miri, test representative boundary lengths; natively, test every length 0..=5000.
            let lens: Vec<usize> = if cfg!(miri) {
                (0..40)
                    .chain([
                        63, 64, 65, 127, 128, 129, 255, 256, 511, 512, 1279, 1280, 1281, 1408,
                        2047, 4096, 4999, 5000,
                    ])
                    .collect()
            } else {
                (0..=5000).collect()
            };
            for n in lens {
                let want = !ref_reg[n];
                assert_eq!(crc32(&data[..n], k), want, "n {n} {k:?}");
            }
        }
        // the check value of CRC-32/BZIP2
        for k in test_kernels() {
            assert_eq!(crc32(b"123456789", k), 0xFC89_1918, "{k:?}");
        }
    }

    /// Exhaustive alignment (`0..64`), initial-register, and threshold boundary test across
    /// scalar, SSE4.1-only Barrett, and AVX2-ring + VEX-8x Barrett kernels.
    #[test]
    fn alignments_states_and_thresholds() {
        if cfg!(miri) {
            return;
        }
        let mut x = 0x517c_c1b7_2722_0a95u64;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let random: Vec<u8> = (0..4300).map(|_| next() as u8).collect();
        let zeros = vec![0u8; 4300];
        let ones = vec![0xffu8; 4300];
        let states = [0u32, !0, 0x8000_0000, 1, 0x1234_5678, 0xDEAD_BEEF];
        let lens = [
            0usize, 1, 2, 3, 4, 7, 8, 12, 13, 14, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128,
            129, 255, 256, 257, 1279, 1280, 1281, 1407, 1408, 1409, 1535, 1536, 1537, 1791, 1792,
            1793, 2047, 2048, 2049, 4095, 4096, 4111,
        ];
        let ks = test_kernels();
        for buf in [&random, &zeros, &ones] {
            for &s in &states {
                for off in 0..64 {
                    for &n in &lens {
                        let d = &buf[off..off + n];
                        let want = update_bytewise(s, d);
                        for &k in &ks {
                            assert_eq!(
                                update_with(s, d, k),
                                want,
                                "s={s:#x} off={off} n={n} {k:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// A CRC taken in pieces (`update_with` from the register the piece before left) is the
    /// whole's, at every split, by each kernel: pieces around 32 KiB / 64 KiB multiples (the
    /// in-place RLE1's tiles, `rle::unrle_in_place`), prime sizes, and across every dispatch
    /// threshold (`16`, `64`, `128`, `1280`)
    #[test]
    fn pieces_make_the_whole() {
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let len = if cfg!(miri) { 5000 } else { (3 << 16) + 300 };
        let data: Vec<u8> = (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect();
        let want = !update_bytewise(!0, &data);
        for k in test_kernels() {
            assert_eq!(crc32(&data, k), want, "{k:?}");
            for tile in [
                1usize,
                7,
                15,
                16,
                17,
                63,
                64,
                127,
                128,
                129,
                1279,
                1280,
                1281,
                1999,
                4096,
                (1 << 15) - 1,
                1 << 15,
                (1 << 16) - 1,
                1 << 16,
                (1 << 16) + 1,
            ] {
                if cfg!(miri) && tile < 64 {
                    continue;
                }
                let mut s = !0u32;
                for c in data.chunks(tile) {
                    s = update_with(s, c, k);
                }
                assert_eq!(!s, want, "tile {tile} {k:?}");
            }
            for cut in [
                0,
                1,
                15,
                16,
                127,
                128,
                1279,
                1280,
                (1 << 15) - 1,
                1 << 15,
                (1 << 16) - 1,
                1 << 16,
                (2 << 16) + 5,
                data.len(),
            ] {
                if cut > data.len() {
                    continue;
                }
                let s = update_with(update_with(!0, &data[..cut], k), &data[cut..], k);
                assert_eq!(!s, want, "cut {cut} {k:?}");
            }
        }
    }

    /// `update` (sliced by 16, its 8-byte tail and its bytes) against `update_bytewise`, from
    /// registers other than the initial one, at every length up to 300 (every remainder of
    /// 16 and 8 many times over), from every offset in a 16-byte window, and on bytes of all
    /// zeros and all ones
    #[test]
    fn sliced_by_16_against_bytewise() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let random: Vec<u8> = (0..400).map(|_| next() as u8).collect();
        let zeros = vec![0u8; 400];
        let ones = vec![0xffu8; 400];
        let states = [0u32, !0, 0x8000_0000, 1, next() as u32, next() as u32];
        for data in [&random, &zeros, &ones] {
            for &s in &states {
                for off in 0..16 {
                    for n in 0..=300 {
                        let d = &data[off..off + n];
                        assert_eq!(
                            update(s, d),
                            update_bytewise(s, d),
                            "s {s:#x} off {off} n {n}"
                        );
                    }
                }
            }
        }
    }
}
