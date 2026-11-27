//! Deep dissection & redesign probe of `src/crc.rs` (`CRC-32/BZIP2`):
//! PCLMULQDQ + AVX2 folding, Port-5 offload (`vpshufb` ring, `movbe`/`bswap` ring,
//! hybrid), zero-table pure-register Barrett reduction, and `crc32_combine`.
//!
//! Smooth-zone standalone probe (zero `src/` edits).

#[cfg(target_arch = "x86_64")]
mod x86 {
    use mibz2::crc::{self, POLY};
    use std::arch::x86_64::*;

    /// `x^n mod P` (non-reflected, `P(x) = x^32 + POLY`) as a 32-bit remainder in `u64`.
    pub const fn xpow_mod(n: u32) -> u64 {
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
    pub const fn barrett_mu() -> u64 {
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

    /// Polynomial multiplication in GF(2)[x] for compile-time / Miri verification.
    pub const fn gf2_mul_u128(a: u64, b: u64) -> u128 {
        let mut out: u128 = 0;
        let mut i = 0;
        while i < 64 {
            if (b >> i) & 1 != 0 {
                out ^= (a as u128) << i;
            }
            i += 1;
        }
        out
    }

    const K128_HI: u64 = xpow_mod(128 + 64);
    const K128_LO: u64 = xpow_mod(128);
    const K256_HI: u64 = xpow_mod(256 + 64);
    const K256_LO: u64 = xpow_mod(256);
    const K512_HI: u64 = xpow_mod(512 + 64);
    const K512_LO: u64 = xpow_mod(512);
    const K1024_HI: u64 = xpow_mod(1024 + 64);
    const K1024_LO: u64 = xpow_mod(1024);
    const K96: u64 = xpow_mod(96);
    const K64: u64 = xpow_mod(64);
    const MU: u64 = barrett_mu();

    /// Sliding shuffle/mask tables for 1..=15 byte tails (96 bytes total = 1.5 cache lines,
    /// vs 16,384 bytes for `src/crc.rs::TABLES`).
    #[repr(align(64))]
    struct TailTables {
        /// `[0..16, 0x80..0x80]`: slice `[16 - r .. 32 - r]` shifts `x` right by `16 - r` bytes
        /// (moving top `r` bytes to low `r` bytes, zero-filling the rest).
        shr_top: [u8; 32],
        /// `[0x80..0x80, 0..16]`: slice `[16 - r .. 32 - r]` shifts `x` left by `r` bytes
        /// (moving low `16 - r` bytes to top `16 - r` bytes, zero-filling low `r` bytes).
        shl_bot: [u8; 32],
        /// `[0xFF..0xFF, 0x00..0x00]`: slice `[16 - r .. 32 - r]` keeps low `r` bytes, zeros rest.
        mask_lo: [u8; 32],
    }

    static TAIL_TABLES: TailTables = TailTables {
        shr_top: [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
        ],
        shl_bot: [
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ],
        mask_lo: [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    };

    #[repr(align(64))]
    struct Ring64([[u64; 8]; 4]);

    #[repr(align(64))]
    struct Ring128([[u64; 16]; 4]);

    #[repr(align(64))]
    struct Ring32B([[__m256i; 2]; 4]);

    #[repr(align(64))]
    struct Ring32B4([[__m256i; 4]; 4]);

    /// Pure-register 128-bit -> 32-bit MSB-first Barrett reduction (`X(x) * x^32 mod P(x)`).
    /// 4 `pclmulqdq` instructions, 0 bytes of `TABLES` touched.
    #[inline]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    unsafe fn reduce_128_to_32_sse(x: __m128i) -> u32 {
        let k96_64 = _mm_set_epi64x(K64 as i64, K96 as i64);
        // Step 1: X_hi * x^96 + X_lo * x^32 -> 96-bit polynomial in bits 0..95.
        let t = _mm_clmulepi64_si128(x, k96_64, 0x01); // x.hi64 * K96
        let lo_shl32 = _mm_slli_si128::<4>(_mm_and_si128(x, _mm_set_epi64x(0, -1)));
        let r96 = _mm_xor_si128(t, lo_shl32);
        // Step 2: r96[64..95] * K64 ^ r96[0..63] -> 64-bit polynomial in bits 0..63.
        let t2 = _mm_clmulepi64_si128(r96, k96_64, 0x11); // r96.hi64 * K64
        let u64v = _mm_xor_si128(t2, r96);
        let u_bits = _mm_cvtsi128_si64(u64v) as u64;
        // Step 3: Barrett reduction of 64-bit U(x) mod P(x):
        // q = clmul(U >> 32, MU) >> 32; rem = (U ^ clmul(q, POLY)) as u32.
        let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
        let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
        let q_vec = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
        let p_vec = _mm_clmulepi64_si128(q_vec, mu_poly, 0x10);
        let p_lo = _mm_cvtsi128_si64(p_vec) as u32;
        (u_bits as u32) ^ p_lo
    }

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    unsafe fn reduce_128_to_32_vex(x: __m128i) -> u32 {
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

    /// Pure-register 64-bit -> 32-bit Barrett reduction (`U(x) mod P(x)`).
    #[inline]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    unsafe fn barrett_64_to_32_sse(u_bits: u64) -> u32 {
        let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
        let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
        let q_vec = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
        let p_vec = _mm_clmulepi64_si128(q_vec, mu_poly, 0x10);
        let p_lo = _mm_cvtsi128_si64(p_vec) as u32;
        (u_bits as u32) ^ p_lo
    }

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    unsafe fn barrett_64_to_32_vex(u_bits: u64) -> u32 {
        let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
        let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
        let q_vec = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
        let p_vec = _mm_clmulepi64_si128(q_vec, mu_poly, 0x10);
        let p_lo = _mm_cvtsi128_si64(p_vec) as u32;
        (u_bits as u32) ^ p_lo
    }

    /// Zero-table CRC update for short buffers `0 <= data.len() < 16` via pure `pclmulqdq` Barrett.
    /// Computes `(state * x^(8n) ^ M(x) * x^32) mod P(x)`.
    #[inline]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    unsafe fn short_barrett_sse(state: u32, data: &[u8]) -> u32 {
        let n = data.len();
        if n == 0 {
            return state;
        }
        let mut m: u128 = 0;
        let mut i = 0;
        while i < n {
            m = (m << 8) | (*data.get_unchecked(i) as u128);
            i += 1;
        }
        if n < 4 {
            let u = ((state as u64) << (n * 8)) ^ ((m as u64) << 32);
            return barrett_64_to_32_sse(u);
        }
        let lo128 = m ^ ((state as u128) << ((n - 4) * 8));
        let x = _mm_set_epi64x((lo128 >> 64) as i64, lo128 as i64);
        reduce_128_to_32_sse(x)
    }

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    unsafe fn short_barrett_vex(state: u32, data: &[u8]) -> u32 {
        let n = data.len();
        if n == 0 {
            return state;
        }
        let mut m: u128 = 0;
        let mut i = 0;
        while i < n {
            m = (m << 8) | (*data.get_unchecked(i) as u128);
            i += 1;
        }
        if n < 4 {
            let u = ((state as u64) << (n * 8)) ^ ((m as u64) << 32);
            return barrett_64_to_32_vex(u);
        }
        let lo128 = m ^ ((state as u128) << ((n - 4) * 8));
        let x = _mm_set_epi64x((lo128 >> 64) as i64, lo128 as i64);
        reduce_128_to_32_vex(x)
    }

    /// Fold 16-byte chunks + final `1..=15` tail bytes into `x` (128-bit MSB-first) and Barrett reduce.
    /// Precondition: `total_len >= 16` and `consumed <= total_len`.
    #[inline]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    unsafe fn finish_from_lane_sse(
        mut x: __m128i,
        p: *const u8,
        mut off: usize,
        len: usize,
        rev: __m128i,
    ) -> u32 {
        let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
        while off + 16 <= len {
            let y = _mm_shuffle_epi8(_mm_loadu_si128(p.add(off) as *const __m128i), rev);
            let h = _mm_clmulepi64_si128(x, k128, 0x11);
            let l = _mm_clmulepi64_si128(x, k128, 0x00);
            x = _mm_xor_si128(_mm_xor_si128(h, l), y);
            off += 16;
        }
        let r = len - off;
        if r > 0 {
            // Unaligned overlap read of last 16 bytes (safe: len >= 16).
            let raw_last =
                _mm_shuffle_epi8(_mm_loadu_si128(p.add(len - 16) as *const __m128i), rev);
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

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    unsafe fn finish_from_lane_vex(
        mut x: __m128i,
        p: *const u8,
        mut off: usize,
        len: usize,
        rev: __m128i,
    ) -> u32 {
        let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
        while off + 16 <= len {
            let y = _mm_shuffle_epi8(_mm_loadu_si128(p.add(off) as *const __m128i), rev);
            let h = _mm_clmulepi64_si128(x, k128, 0x11);
            let l = _mm_clmulepi64_si128(x, k128, 0x00);
            x = _mm_xor_si128(_mm_xor_si128(h, l), y);
            off += 16;
        }
        let r = len - off;
        if r > 0 {
            let raw_last =
                _mm_shuffle_epi8(_mm_loadu_si128(p.add(len - 16) as *const __m128i), rev);
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

    #[inline]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    unsafe fn fold4_to_1_sse(x0: __m128i, x1: __m128i, x2: __m128i, x3: __m128i) -> __m128i {
        let k256 = _mm_set_epi64x(K256_HI as i64, K256_LO as i64);
        let k128 = _mm_set_epi64x(K128_HI as i64, K128_LO as i64);
        // Parallel tree: (x0 -> x2) and (x1 -> x3) by 256 bits, then (x2 -> x3) by 128 bits.
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

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    unsafe fn fold4_to_1_vex(x0: __m128i, x1: __m128i, x2: __m128i, x3: __m128i) -> __m128i {
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

    #[inline]
    #[target_feature(enable = "avx,pclmulqdq")]
    #[allow(clippy::too_many_arguments)]
    unsafe fn fold8_to_1_vex(
        x0: __m128i,
        x1: __m128i,
        x2: __m128i,
        x3: __m128i,
        x4: __m128i,
        x5: __m128i,
        x6: __m128i,
        x7: __m128i,
    ) -> __m128i {
        let k512 = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let f = |a: __m128i, b: __m128i| {
            _mm_xor_si128(
                _mm_xor_si128(
                    _mm_clmulepi64_si128(a, k512, 0x11),
                    _mm_clmulepi64_si128(a, k512, 0x00),
                ),
                b,
            )
        };
        fold4_to_1_vex(f(x0, x4), f(x1, x5), f(x2, x6), f(x3, x7))
    }

    // -----------------------------------------------------------------------
    // A0: Shipped crate directly (`mibz2::crc::update_with`)
    // -----------------------------------------------------------------------
    #[inline(never)]
    pub fn a0_shipped_crate(state: u32, data: &[u8]) -> u32 {
        crc::update_with(state, data, mibz2::Kernels::detect())
    }

    // -----------------------------------------------------------------------
    // A1: Faithful mirror of shipped `src/crc.rs::clmul_update` (4x SSE + TABLES)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    pub unsafe fn a1_shipped_sse_4x_table(state: u32, data: &[u8]) -> u32 {
        if data.len() < 128 {
            return crc::update(state, data);
        }
        let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
        let blocks = data.len() / 64;
        let mut x0 = ld(p);
        let mut x1 = ld(p.add(16));
        let mut x2 = ld(p.add(32));
        let mut x3 = ld(p.add(48));
        x0 = _mm_xor_si128(x0, _mm_set_epi32(state as i32, 0, 0, 0));
        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold = |x: __m128i, y: __m128i| {
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
        let mut tail = [0u8; 64];
        for (i, x) in [x0, x1, x2, x3].into_iter().enumerate() {
            _mm_storeu_si128(
                tail.as_mut_ptr().add(i * 16) as *mut __m128i,
                _mm_shuffle_epi8(x, rev),
            );
        }
        let s = crc::update(0, &tail);
        crc::update(s, &data[blocks * 64..])
    }

    // -----------------------------------------------------------------------
    // A2: 4x128b SSE + zero-table Barrett reduction (0 B TABLES, 0 vzeroupper)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
    pub unsafe fn a2_sse_4x_barrett(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 16 {
            return short_barrett_sse(state, data);
        }
        let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
        if len < 64 {
            let x = _mm_xor_si128(ld(p), _mm_set_epi32(state as i32, 0, 0, 0));
            return finish_from_lane_sse(x, p, 16, len, rev);
        }
        let blocks = len / 64;
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
            let q = p.add(b * 64);
            x0 = fold(x0, ld(q));
            x1 = fold(x1, ld(q.add(16)));
            x2 = fold(x2, ld(q.add(32)));
            x3 = fold(x3, ld(q.add(48)));
        }
        let x = fold4_to_1_sse(x0, x1, x2, x3);
        finish_from_lane_sse(x, p, blocks * 64, len, rev)
    }

    // -----------------------------------------------------------------------
    // A3: 4x128b VEX (non-destructive 3-op, 0 vzeroupper, 0 B TABLES)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn a3_vex_4x_barrett(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 16 {
            return short_barrett_vex(state, data);
        }
        let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
        if len < 64 {
            let x = _mm_xor_si128(ld(p), _mm_set_epi32(state as i32, 0, 0, 0));
            return finish_from_lane_vex(x, p, 16, len, rev);
        }
        let blocks = len / 64;
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
            let q = p.add(b * 64);
            x0 = fold(x0, ld(q));
            x1 = fold(x1, ld(q.add(16)));
            x2 = fold(x2, ld(q.add(32)));
            x3 = fold(x3, ld(q.add(48)));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev)
    }

    // -----------------------------------------------------------------------
    // A4: 8x128b VEX (128 B/iter, 0 vzeroupper, 0 B TABLES)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn a4_vex_8x_barrett(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 128 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
        let blocks = len / 128;
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

    // -----------------------------------------------------------------------
    // A5: 256-bit `vpshufb` + `vextracti128` + 4x128b `vpclmulqdq` + Barrett
    // (Tests whether `vextracti128` Port-5 uop cancels `vpshufb` Port-5 saving.)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn a5_avx2_vpshufb_extract_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 64 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 64;
        let v0 = _mm256_shuffle_epi8(_mm256_loadu_si256(p as *const __m256i), rev256);
        let v1 = _mm256_shuffle_epi8(_mm256_loadu_si256(p.add(32) as *const __m256i), rev256);
        let mut x0 = _mm_xor_si128(
            _mm256_castsi256_si128(v0),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm256_extracti128_si256::<1>(v0);
        let mut x2 = _mm256_castsi256_si128(v1);
        let mut x3 = _mm256_extracti128_si256::<1>(v1);
        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold = |x: __m128i, y: __m128i| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        for b in 1..blocks {
            let q = p.add(b * 64);
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(q as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(q.add(32) as *const __m256i), rev256);
            x0 = fold(x0, _mm256_castsi256_si128(w0));
            x1 = fold(x1, _mm256_extracti128_si256::<1>(w0));
            x2 = fold(x2, _mm256_castsi256_si128(w1));
            x3 = fold(x3, _mm256_extracti128_si256::<1>(w1));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A6: 256-bit `vpshufb` -> aligned L1d 4-slot ring -> 128-bit `vpxor`
    // (10 Port-5 uops / 64 B vs 12 shipped = -16.7% Port-5 floor, 2 stores/64B!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn a6_avx2_vpshufb_store_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring32B([[_mm256_setzero_si256(); 2]; 4]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        // Pre-fill ring slots 0, 1, 2 (blocks 1, 2, 3) so reads trail writes by 3 iterations.
        let v0 = _mm256_shuffle_epi8(_mm256_loadu_si256(p as *const __m256i), rev256);
        let v1 = _mm256_shuffle_epi8(_mm256_loadu_si256(p.add(32) as *const __m256i), rev256);
        _mm256_store_si256(rptr as *mut __m256i, v0);
        _mm256_store_si256(rptr.add(32) as *mut __m256i, v1);

        for s in 1..4 {
            let q = p.add(s * 64);
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(q as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(q.add(32) as *const __m256i), rev256);
            let dst = rptr.add(s * 64);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
        }

        let mut x0 = _mm_xor_si128(
            _mm_load_si128(rptr as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(rptr.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(rptr.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(rptr.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        // Main steady-state loop: write block `b + 3` into slot `(b + 3) & 3`,
        // consume block `b` from slot `b & 3` (3 blocks = 192 B behind store!).
        for b in 1..(blocks - 3) {
            let q_wr = p.add((b + 3) * 64);
            let dst = rptr.add(((b + 3) & 3) * 64);
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(q_wr as *const __m256i), rev256);
            let w1 =
                _mm256_shuffle_epi8(_mm256_loadu_si256(q_wr.add(32) as *const __m256i), rev256);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);

            let src = rptr.add((b & 3) * 64);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        // Drain final 3 pre-swapped blocks from ring.
        for b in (blocks - 3)..blocks {
            let src = rptr.add((b & 3) * 64);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A7: 8-lane (128 B/iter), 4x 256-bit `vpshufb` -> aligned L1d ring -> 128-bit `vpxor`
    // (10 Port-5 uops / 64 B, 2 stores/64B, 8-lane latency hiding!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn a7_avx2_vpshufb_store_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 640 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring32B4([[_mm256_setzero_si256(); 4]; 4]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        let store_block = |src: *const u8, dst: *mut u8| {
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(src as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
            let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
            let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
            _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
            _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
        };

        for s in 0..4 {
            store_block(p.add(s * 128), rptr.add(s * 128));
        }

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
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            store_block(p.add((b + 3) * 128), rptr.add(((b + 3) & 3) * 128));
            let src = rptr.add((b & 3) * 128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
            x4 = fold_mem(x4, src.add(64));
            x5 = fold_mem(x5, src.add(80));
            x6 = fold_mem(x6, src.add(96));
            x7 = fold_mem(x7, src.add(112));
        }
        for b in (blocks - 3)..blocks {
            let src = rptr.add((b & 3) * 128);
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

    // -----------------------------------------------------------------------
    // A8: `movbe` scalar 64-bit byte-swap -> qword-swapped aligned L1d 4-slot ring
    //     -> 4x128b VEX `vpclmulqdq` (ZERO `pshufb` = 8 Port-5 uops / 64 B = -33.3%!)
    //     ZERO `vzeroupper`, ZERO `TABLES`!
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn a8_movbe_ring_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64([[0u64; 8]; 4]);

        #[inline(always)]
        unsafe fn swap_block64(src: *const u8, dst: &mut [u64; 8]) {
            let s = src as *const u64;
            // Each 16-byte lane: bytes 0..8 -> hi64 (dst[2*i+1]), bytes 8..16 -> lo64 (dst[2*i]).
            dst[1] = u64::from_be(s.read_unaligned());
            dst[0] = u64::from_be(s.add(1).read_unaligned());
            dst[3] = u64::from_be(s.add(2).read_unaligned());
            dst[2] = u64::from_be(s.add(3).read_unaligned());
            dst[5] = u64::from_be(s.add(4).read_unaligned());
            dst[4] = u64::from_be(s.add(5).read_unaligned());
            dst[7] = u64::from_be(s.add(6).read_unaligned());
            dst[6] = u64::from_be(s.add(7).read_unaligned());
        }

        for s in 0..4 {
            swap_block64(p.add(s * 64), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(r0.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(r0.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_block64(p.add((b + 3) * 64), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A9: 8-lane (128 B/iter) `movbe` -> qword-swapped L1d ring -> 8x128b VEX
    // (ZERO `pshufb` = 8 Port-5 uops / 64 B, ZERO `vzeroupper`, ZERO `TABLES`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn a9_movbe_ring_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 640 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring128([[0u64; 16]; 4]);

        #[inline(always)]
        unsafe fn swap_block128(src: *const u8, dst: &mut [u64; 16]) {
            let s = src as *const u64;
            for i in 0..8 {
                dst[2 * i + 1] = u64::from_be(s.add(2 * i).read_unaligned());
                dst[2 * i] = u64::from_be(s.add(2 * i + 1).read_unaligned());
            }
        }

        for s in 0..4 {
            swap_block128(p.add(s * 128), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(r0.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(r0.add(48) as *const __m128i);
        let mut x4 = _mm_load_si128(r0.add(64) as *const __m128i);
        let mut x5 = _mm_load_si128(r0.add(80) as *const __m128i);
        let mut x6 = _mm_load_si128(r0.add(96) as *const __m128i);
        let mut x7 = _mm_load_si128(r0.add(112) as *const __m128i);

        let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_block128(p.add((b + 3) * 128), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
            x4 = fold_mem(x4, src.add(64));
            x5 = fold_mem(x5, src.add(80));
            x6 = fold_mem(x6, src.add(96));
            x7 = fold_mem(x7, src.add(112));
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
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

    // -----------------------------------------------------------------------
    // A10: `bswap` (no `movbe` feature required) -> qword-swapped L1d ring -> 4x128b VEX
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn a10_bswap_ring_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64([[0u64; 8]; 4]);

        #[inline(always)]
        unsafe fn swap_block64(src: *const u8, dst: &mut [u64; 8]) {
            let s = src as *const u64;
            dst[1] = u64::from_be(s.read_unaligned());
            dst[0] = u64::from_be(s.add(1).read_unaligned());
            dst[3] = u64::from_be(s.add(2).read_unaligned());
            dst[2] = u64::from_be(s.add(3).read_unaligned());
            dst[5] = u64::from_be(s.add(4).read_unaligned());
            dst[4] = u64::from_be(s.add(5).read_unaligned());
            dst[7] = u64::from_be(s.add(6).read_unaligned());
            dst[6] = u64::from_be(s.add(7).read_unaligned());
        }

        for s in 0..4 {
            swap_block64(p.add(s * 64), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(r0.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(r0.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_block64(p.add((b + 3) * 64), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A11: Hybrid 3 lanes `movbe` ring + 1 lane `vpshufb xmm`
    //      (9 Port-5 uops / 64 B, 6 stores/64B, ZERO `vzeroupper`, ZERO `TABLES`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn a11_hybrid_3movbe_1pshufb_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64([[0u64; 8]; 4]);

        #[inline(always)]
        unsafe fn swap_3lanes(src: *const u8, dst: &mut [u64; 8]) {
            let s = src as *const u64;
            dst[1] = u64::from_be(s.read_unaligned());
            dst[0] = u64::from_be(s.add(1).read_unaligned());
            dst[3] = u64::from_be(s.add(2).read_unaligned());
            dst[2] = u64::from_be(s.add(3).read_unaligned());
            dst[5] = u64::from_be(s.add(4).read_unaligned());
            dst[4] = u64::from_be(s.add(5).read_unaligned());
        }

        for s in 0..4 {
            swap_3lanes(p.add(s * 64), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(r0.add(32) as *const __m128i);
        let mut x3 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(48) as *const __m128i), rev128);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        let fold_reg = |x: __m128i, y: __m128i| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_3lanes(p.add((b + 3) * 64), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            let y3 = _mm_shuffle_epi8(
                _mm_loadu_si128(p.add(b * 64 + 48) as *const __m128i),
                rev128,
            );
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_reg(x3, y3);
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
            let y3 = _mm_shuffle_epi8(
                _mm_loadu_si128(p.add(b * 64 + 48) as *const __m128i),
                rev128,
            );
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_reg(x3, y3);
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A12: Hybrid 2 lanes `movbe` + 1x 256-bit `vpshufb` -> aligned L1d ring
    //      (9 Port-5 uops / 64 B, ONLY 5 stores/64B, ZERO `TABLES`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq,movbe")]
    pub unsafe fn a12_hybrid_2movbe_1vpshufb256_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64([[0u64; 8]; 4]);

        let swap_hybrid = |src: *const u8, dst: &mut [u64; 8]| {
            let s = src as *const u64;
            dst[1] = u64::from_be(s.read_unaligned());
            dst[0] = u64::from_be(s.add(1).read_unaligned());
            dst[3] = u64::from_be(s.add(2).read_unaligned());
            dst[2] = u64::from_be(s.add(3).read_unaligned());
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
            _mm256_store_si256(dst.as_mut_ptr().add(4) as *mut __m256i, w1);
        };

        for s in 0..4 {
            swap_hybrid(p.add(s * 64), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(r0.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(r0.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_hybrid(p.add((b + 3) * 64), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A13: 256-bit `vpshufb` 1-slot immediate store -> 128-bit load (tests STL forwarding!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn a13_avx2_vpshufb_1slot_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 64 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 64;
        #[repr(align(64))]
        struct Buf([__m256i; 2]);
        let mut buf = Buf([_mm256_setzero_si256(); 2]);
        let bp = buf.0.as_mut_ptr() as *mut u8;

        _mm256_store_si256(
            bp as *mut __m256i,
            _mm256_shuffle_epi8(_mm256_loadu_si256(p as *const __m256i), rev256),
        );
        _mm256_store_si256(
            bp.add(32) as *mut __m256i,
            _mm256_shuffle_epi8(_mm256_loadu_si256(p.add(32) as *const __m256i), rev256),
        );
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(bp as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(bp.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(bp.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(bp.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        for b in 1..blocks {
            let q = p.add(b * 64);
            _mm256_store_si256(
                bp as *mut __m256i,
                _mm256_shuffle_epi8(_mm256_loadu_si256(q as *const __m256i), rev256),
            );
            _mm256_store_si256(
                bp.add(32) as *mut __m256i,
                _mm256_shuffle_epi8(_mm256_loadu_si256(q.add(32) as *const __m256i), rev256),
            );
            x0 = fold_mem(x0, bp);
            x1 = fold_mem(x1, bp.add(16));
            x2 = fold_mem(x2, bp.add(32));
            x3 = fold_mem(x3, bp.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A14: 2-lane `movbe` + 2-lane `vpshufb xmm` (10 Port-5 uops / 64 B,
    //      4 stores/64B, ZERO `vzeroupper`, ZERO `TABLES`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn a14_hybrid_2movbe_2pshufb_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 384 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        #[repr(align(64))]
        struct Ring32([[u64; 4]; 4]);
        let mut ring = Ring32([[0u64; 4]; 4]);

        #[inline(always)]
        unsafe fn swap_2lanes(src: *const u8, dst: &mut [u64; 4]) {
            let s = src as *const u64;
            dst[1] = u64::from_be(s.read_unaligned());
            dst[0] = u64::from_be(s.add(1).read_unaligned());
            dst[3] = u64::from_be(s.add(2).read_unaligned());
            dst[2] = u64::from_be(s.add(3).read_unaligned());
        }

        for s in 0..4 {
            swap_2lanes(p.add(s * 64), &mut ring.0[s]);
        }

        let r0 = ring.0[0].as_ptr() as *const u8;
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(r0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(r0.add(16) as *const __m128i);
        let mut x2 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(32) as *const __m128i), rev128);
        let mut x3 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(48) as *const __m128i), rev128);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        let fold_reg = |x: __m128i, y: __m128i| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 3) {
            swap_2lanes(p.add((b + 3) * 64), &mut ring.0[(b + 3) & 3]);
            let src = ring.0[b & 3].as_ptr() as *const u8;
            let q = p.add(b * 64);
            let y2 = _mm_shuffle_epi8(_mm_loadu_si128(q.add(32) as *const __m128i), rev128);
            let y3 = _mm_shuffle_epi8(_mm_loadu_si128(q.add(48) as *const __m128i), rev128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_reg(x2, y2);
            x3 = fold_reg(x3, y3);
        }
        for b in (blocks - 3)..blocks {
            let src = ring.0[b & 3].as_ptr() as *const u8;
            let q = p.add(b * 64);
            let y2 = _mm_shuffle_epi8(_mm_loadu_si128(q.add(32) as *const __m128i), rev128);
            let y3 = _mm_shuffle_epi8(_mm_loadu_si128(q.add(48) as *const __m128i), rev128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_reg(x2, y2);
            x3 = fold_reg(x3, y3);
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // A15: Aligned 16B peel + 8-lane VEX (tests cache-line split overhead)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn a15_aligned_vex_8x_barrett(mut state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 256 {
            return a3_vex_4x_barrett(state, data);
        }
        let mis = (data.as_ptr() as usize) & 15;
        let peel = if mis == 0 { 0 } else { 16 - mis };
        if peel > 0 {
            state = short_barrett_vex(state, &data[..peel]);
        }
        a4_vex_8x_barrett(state, &data[peel..])
    }

    // -----------------------------------------------------------------------
    // Helper: true scalar `movbe` + qword-swapped store via `asm!` so LLVM cannot
    // SLP-vectorize `u64::from_be` back onto Port 5 (`vpshufb`/`vinsertf128`/`vshufpd`).
    // Writes 16 bytes (one lane) from `src` to `dst` byte-reversed into 128-bit MSB order.
    // -----------------------------------------------------------------------
    #[inline(always)]
    unsafe fn movbe_lane16(src: *const u8, dst: *mut u8) {
        let lo: u64;
        let hi: u64;
        core::arch::asm!(
            "movbe {hi}, [{src}]",
            "movbe {lo}, [{src} + 8]",
            "mov [{dst} + 8], {hi}",
            "mov [{dst}], {lo}",
            src = in(reg) src,
            dst = in(reg) dst,
            hi = out(reg) hi,
            lo = out(reg) lo,
            options(nostack, preserves_flags),
        );
        let _ = (lo, hi);
    }

    #[repr(align(64))]
    struct Ring64x8([[u64; 8]; 8]);

    #[repr(align(64))]
    struct Ring128x8([[u64; 16]; 8]);

    #[repr(align(64))]
    struct Strip512([__m256i; 16]);

    #[repr(align(64))]
    struct Strip1024([u64; 128]);

    // -----------------------------------------------------------------------
    // B1: TRUE scalar `movbe` -> 8-slot (512 B) ring (6 blocks = 48 stores ahead > 42 SB)
    //     -> 4x128b VEX `vpclmulqdq` (8.0 Port-5 uops / 64 B, ZERO `vzeroupper`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn b1_true_movbe_ring8_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 640 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64x8([[0u64; 8]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        for s in 0..8 {
            let src = p.add(s * 64);
            let dst = rptr.add(s * 64);
            movbe_lane16(src, dst);
            movbe_lane16(src.add(16), dst.add(16));
            movbe_lane16(src.add(32), dst.add(32));
            movbe_lane16(src.add(48), dst.add(48));
        }

        let mut x0 = _mm_xor_si128(
            _mm_load_si128(rptr as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(rptr.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(rptr.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(rptr.add(48) as *const __m128i);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            let q = p.add((b + 7) * 64);
            let dst = rptr.add(((b + 7) & 7) * 64);
            movbe_lane16(q, dst);
            movbe_lane16(q.add(16), dst.add(16));
            movbe_lane16(q.add(32), dst.add(32));
            movbe_lane16(q.add(48), dst.add(48));

            let src = rptr.add((b & 7) * 64);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        for b in (blocks - 7)..blocks {
            let src = rptr.add((b & 7) * 64);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // B2: TRUE scalar `movbe` -> 8-slot (1024 B) ring (7 blocks = 112 stores ahead)
    //     -> 8x128b VEX `vpclmulqdq` (8.0 Port-5 uops / 64 B, ZERO `vzeroupper`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn b2_true_movbe_ring8_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 1280 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring128x8([[0u64; 16]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        for s in 0..8 {
            let src = p.add(s * 128);
            let dst = rptr.add(s * 128);
            for l in 0..8 {
                movbe_lane16(src.add(l * 16), dst.add(l * 16));
            }
        }

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
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            let q = p.add((b + 7) * 128);
            let dst = rptr.add(((b + 7) & 7) * 128);
            for l in 0..8 {
                movbe_lane16(q.add(l * 16), dst.add(l * 16));
            }
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

    // -----------------------------------------------------------------------
    // B3: TRUE Hybrid 4 lanes `movbe` + 2x 256-bit `vpshufb` (8 lanes = 128 B/iter)
    //     (9.0 Port-5 uops / 64 B = -25% Port-5 floor, ONLY 5 stores/64B!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq,movbe")]
    pub unsafe fn b3_true_hyb_4movbe_2avx2_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 1280 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring128x8([[0u64; 16]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        let prep = |src: *const u8, dst: *mut u8| {
            movbe_lane16(src, dst);
            movbe_lane16(src.add(16), dst.add(16));
            movbe_lane16(src.add(32), dst.add(32));
            movbe_lane16(src.add(48), dst.add(48));
            let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
            let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
            _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
            _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
        };

        for s in 0..8 {
            prep(p.add(s * 128), rptr.add(s * 128));
        }

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
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            prep(p.add((b + 7) * 128), rptr.add(((b + 7) & 7) * 128));
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

    // -----------------------------------------------------------------------
    // B4: TRUE Hybrid 3 lanes `movbe` + 1 lane 128-bit `vpshufb` (8-slot ring)
    //     (9.0 Port-5 uops / 64 B, 6 stores/64B, ZERO `vzeroupper`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn b4_true_hyb_3movbe_1shuf_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 640 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64x8([[0u64; 8]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        for s in 0..8 {
            let src = p.add(s * 64);
            let dst = rptr.add(s * 64);
            movbe_lane16(src, dst);
            movbe_lane16(src.add(16), dst.add(16));
            movbe_lane16(src.add(32), dst.add(32));
        }

        let mut x0 = _mm_xor_si128(
            _mm_load_si128(rptr as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(rptr.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(rptr.add(32) as *const __m128i);
        let mut x3 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(48) as *const __m128i), rev128);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        let fold_reg = |x: __m128i, y: __m128i| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            let q = p.add((b + 7) * 64);
            let dst = rptr.add(((b + 7) & 7) * 64);
            movbe_lane16(q, dst);
            movbe_lane16(q.add(16), dst.add(16));
            movbe_lane16(q.add(32), dst.add(32));

            let src = rptr.add((b & 7) * 64);
            let y3 = _mm_shuffle_epi8(
                _mm_loadu_si128(p.add(b * 64 + 48) as *const __m128i),
                rev128,
            );
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_reg(x3, y3);
        }
        for b in (blocks - 7)..blocks {
            let src = rptr.add((b & 7) * 64);
            let y3 = _mm_shuffle_epi8(
                _mm_loadu_si128(p.add(b * 64 + 48) as *const __m128i),
                rev128,
            );
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_reg(x3, y3);
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // B5: TRUE Hybrid 2 lanes `movbe` + 2 lanes 128-bit `vpshufb` (8-slot ring)
    //     (10.0 Port-5 uops / 64 B, ONLY 4 stores/64B, ZERO `vzeroupper`!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn b5_true_hyb_2movbe_2shuf_4x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 640 {
            return a3_vex_4x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let blocks = len / 64;
        let mut ring = Ring64x8([[0u64; 8]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        for s in 0..8 {
            let src = p.add(s * 64);
            let dst = rptr.add(s * 64);
            movbe_lane16(src, dst);
            movbe_lane16(src.add(16), dst.add(16));
        }

        let mut x0 = _mm_xor_si128(
            _mm_load_si128(rptr as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(rptr.add(16) as *const __m128i);
        let mut x2 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(32) as *const __m128i), rev128);
        let mut x3 = _mm_shuffle_epi8(_mm_loadu_si128(p.add(48) as *const __m128i), rev128);

        let k = _mm_set_epi64x(K512_HI as i64, K512_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };
        let fold_reg = |x: __m128i, y: __m128i| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            let q = p.add((b + 7) * 64);
            let dst = rptr.add(((b + 7) & 7) * 64);
            movbe_lane16(q, dst);
            movbe_lane16(q.add(16), dst.add(16));

            let src = rptr.add((b & 7) * 64);
            let cur = p.add(b * 64);
            let y2 = _mm_shuffle_epi8(_mm_loadu_si128(cur.add(32) as *const __m128i), rev128);
            let y3 = _mm_shuffle_epi8(_mm_loadu_si128(cur.add(48) as *const __m128i), rev128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_reg(x2, y2);
            x3 = fold_reg(x3, y3);
        }
        for b in (blocks - 7)..blocks {
            let src = rptr.add((b & 7) * 64);
            let cur = p.add(b * 64);
            let y2 = _mm_shuffle_epi8(_mm_loadu_si128(cur.add(32) as *const __m128i), rev128);
            let y3 = _mm_shuffle_epi8(_mm_loadu_si128(cur.add(48) as *const __m128i), rev128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_reg(x2, y2);
            x3 = fold_reg(x3, y3);
        }
        let x = fold4_to_1_vex(x0, x1, x2, x3);
        finish_from_lane_vex(x, p, blocks * 64, len, rev128)
    }

    // -----------------------------------------------------------------------
    // B6: 8-lane 256-bit `vpshufb` -> unrolled 2-slot (256 B) fixed-offset ping-pong
    //     (ZERO ring index arithmetic! 10.0 Port-5 uops / 64 B, 256 B stack)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn b6_avx2_pingpong_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 512 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 128;
        #[repr(align(64))]
        struct PingPong([[__m256i; 4]; 2]);
        let mut buf = PingPong([[_mm256_setzero_si256(); 4]; 2]);
        let b0 = buf.0[0].as_mut_ptr() as *mut u8;
        let b1 = buf.0[1].as_mut_ptr() as *mut u8;

        let store128 = |src: *const u8, dst: *mut u8| {
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(src as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
            let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
            let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
            _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
            _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
        };

        store128(p, b0);
        store128(p.add(128), b1);

        let mut x0 = _mm_xor_si128(
            _mm_load_si128(b0 as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(b0.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(b0.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(b0.add(48) as *const __m128i);
        let mut x4 = _mm_load_si128(b0.add(64) as *const __m128i);
        let mut x5 = _mm_load_si128(b0.add(80) as *const __m128i);
        let mut x6 = _mm_load_si128(b0.add(96) as *const __m128i);
        let mut x7 = _mm_load_si128(b0.add(112) as *const __m128i);

        let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        let mut b = 1usize;
        while b + 2 < blocks {
            // Write block b+1 to b0, fold block b from b1
            store128(p.add((b + 1) * 128), b0);
            x0 = fold_mem(x0, b1);
            x1 = fold_mem(x1, b1.add(16));
            x2 = fold_mem(x2, b1.add(32));
            x3 = fold_mem(x3, b1.add(48));
            x4 = fold_mem(x4, b1.add(64));
            x5 = fold_mem(x5, b1.add(80));
            x6 = fold_mem(x6, b1.add(96));
            x7 = fold_mem(x7, b1.add(112));

            // Write block b+2 to b1, fold block b+1 from b0
            store128(p.add((b + 2) * 128), b1);
            x0 = fold_mem(x0, b0);
            x1 = fold_mem(x1, b0.add(16));
            x2 = fold_mem(x2, b0.add(32));
            x3 = fold_mem(x3, b0.add(48));
            x4 = fold_mem(x4, b0.add(64));
            x5 = fold_mem(x5, b0.add(80));
            x6 = fold_mem(x6, b0.add(96));
            x7 = fold_mem(x7, b0.add(112));
            b += 2;
        }
        // Remaining 1 or 2 blocks: block `b` is already in `b1` (because b is odd!).
        if b + 1 < blocks {
            store128(p.add((b + 1) * 128), b0);
            x0 = fold_mem(x0, b1);
            x1 = fold_mem(x1, b1.add(16));
            x2 = fold_mem(x2, b1.add(32));
            x3 = fold_mem(x3, b1.add(48));
            x4 = fold_mem(x4, b1.add(64));
            x5 = fold_mem(x5, b1.add(80));
            x6 = fold_mem(x6, b1.add(96));
            x7 = fold_mem(x7, b1.add(112));

            x0 = fold_mem(x0, b0);
            x1 = fold_mem(x1, b0.add(16));
            x2 = fold_mem(x2, b0.add(32));
            x3 = fold_mem(x3, b0.add(48));
            x4 = fold_mem(x4, b0.add(64));
            x5 = fold_mem(x5, b0.add(80));
            x6 = fold_mem(x6, b0.add(96));
            x7 = fold_mem(x7, b0.add(112));
        } else {
            x0 = fold_mem(x0, b1);
            x1 = fold_mem(x1, b1.add(16));
            x2 = fold_mem(x2, b1.add(32));
            x3 = fold_mem(x3, b1.add(48));
            x4 = fold_mem(x4, b1.add(64));
            x5 = fold_mem(x5, b1.add(80));
            x6 = fold_mem(x6, b1.add(96));
            x7 = fold_mem(x7, b1.add(112));
        }
        let x = fold8_to_1_vex(x0, x1, x2, x3, x4, x5, x6, x7);
        finish_from_lane_vex(x, p, blocks * 128, len, rev128)
    }

    // -----------------------------------------------------------------------
    // B7: 8-lane 256-bit `vpshufb` 512 B strip-mined (zero ring math, 512 B L1d)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn b7_avx2_strip512_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 512 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let mut strip = Strip512([_mm256_setzero_si256(); 16]);
        let sp = strip.0.as_mut_ptr() as *mut u8;

        // First 512 B strip
        for i in 0..16 {
            let v =
                _mm256_shuffle_epi8(_mm256_loadu_si256(p.add(i * 32) as *const __m256i), rev256);
            _mm256_store_si256(sp.add(i * 32) as *mut __m256i, v);
        }
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(sp as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(sp.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(sp.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(sp.add(48) as *const __m128i);
        let mut x4 = _mm_load_si128(sp.add(64) as *const __m128i);
        let mut x5 = _mm_load_si128(sp.add(80) as *const __m128i);
        let mut x6 = _mm_load_si128(sp.add(96) as *const __m128i);
        let mut x7 = _mm_load_si128(sp.add(112) as *const __m128i);

        let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for s in 1..4 {
            let src = sp.add(s * 128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
            x4 = fold_mem(x4, src.add(64));
            x5 = fold_mem(x5, src.add(80));
            x6 = fold_mem(x6, src.add(96));
            x7 = fold_mem(x7, src.add(112));
        }

        let strips = len / 512;
        for st in 1..strips {
            let q = p.add(st * 512);
            for i in 0..16 {
                let v = _mm256_shuffle_epi8(
                    _mm256_loadu_si256(q.add(i * 32) as *const __m256i),
                    rev256,
                );
                _mm256_store_si256(sp.add(i * 32) as *mut __m256i, v);
            }
            for s in 0..4 {
                let src = sp.add(s * 128);
                x0 = fold_mem(x0, src);
                x1 = fold_mem(x1, src.add(16));
                x2 = fold_mem(x2, src.add(32));
                x3 = fold_mem(x3, src.add(48));
                x4 = fold_mem(x4, src.add(64));
                x5 = fold_mem(x5, src.add(80));
                x6 = fold_mem(x6, src.add(96));
                x7 = fold_mem(x7, src.add(112));
            }
        }
        let mut off = strips * 512;
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev128);
        while off + 128 <= len {
            let q = p.add(off);
            let fold = |x: __m128i, y: __m128i| {
                let h = _mm_clmulepi64_si128(x, k, 0x11);
                let l = _mm_clmulepi64_si128(x, k, 0x00);
                _mm_xor_si128(_mm_xor_si128(h, l), y)
            };
            x0 = fold(x0, ld(q));
            x1 = fold(x1, ld(q.add(16)));
            x2 = fold(x2, ld(q.add(32)));
            x3 = fold(x3, ld(q.add(48)));
            x4 = fold(x4, ld(q.add(64)));
            x5 = fold(x5, ld(q.add(80)));
            x6 = fold(x6, ld(q.add(96)));
            x7 = fold(x7, ld(q.add(112)));
            off += 128;
        }
        let x = fold8_to_1_vex(x0, x1, x2, x3, x4, x5, x6, x7);
        finish_from_lane_vex(x, p, off, len, rev128)
    }

    // -----------------------------------------------------------------------
    // B8: TRUE scalar `movbe` 1 KiB strip-mined (128 stores >> 56 SB) + 8-lane VEX
    //     (8.0 Port-5 uops / 64 B, ZERO `vzeroupper`, ZERO ring wraparound!)
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq,movbe")]
    pub unsafe fn b8_true_movbe_strip1k_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 1024 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev128 = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let p = data.as_ptr();
        let mut strip = Strip1024([0u64; 128]);
        let sp = strip.0.as_mut_ptr() as *mut u8;

        for i in 0..64 {
            movbe_lane16(p.add(i * 16), sp.add(i * 16));
        }
        let mut x0 = _mm_xor_si128(
            _mm_load_si128(sp as *const __m128i),
            _mm_set_epi32(state as i32, 0, 0, 0),
        );
        let mut x1 = _mm_load_si128(sp.add(16) as *const __m128i);
        let mut x2 = _mm_load_si128(sp.add(32) as *const __m128i);
        let mut x3 = _mm_load_si128(sp.add(48) as *const __m128i);
        let mut x4 = _mm_load_si128(sp.add(64) as *const __m128i);
        let mut x5 = _mm_load_si128(sp.add(80) as *const __m128i);
        let mut x6 = _mm_load_si128(sp.add(96) as *const __m128i);
        let mut x7 = _mm_load_si128(sp.add(112) as *const __m128i);

        let k = _mm_set_epi64x(K1024_HI as i64, K1024_LO as i64);
        let fold_mem = |x: __m128i, src: *const u8| {
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for s in 1..8 {
            let src = sp.add(s * 128);
            x0 = fold_mem(x0, src);
            x1 = fold_mem(x1, src.add(16));
            x2 = fold_mem(x2, src.add(32));
            x3 = fold_mem(x3, src.add(48));
            x4 = fold_mem(x4, src.add(64));
            x5 = fold_mem(x5, src.add(80));
            x6 = fold_mem(x6, src.add(96));
            x7 = fold_mem(x7, src.add(112));
        }

        let strips = len / 1024;
        for st in 1..strips {
            let q = p.add(st * 1024);
            for i in 0..64 {
                movbe_lane16(q.add(i * 16), sp.add(i * 16));
            }
            for s in 0..8 {
                let src = sp.add(s * 128);
                x0 = fold_mem(x0, src);
                x1 = fold_mem(x1, src.add(16));
                x2 = fold_mem(x2, src.add(32));
                x3 = fold_mem(x3, src.add(48));
                x4 = fold_mem(x4, src.add(64));
                x5 = fold_mem(x5, src.add(80));
                x6 = fold_mem(x6, src.add(96));
                x7 = fold_mem(x7, src.add(112));
            }
        }
        let mut off = strips * 1024;
        let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev128);
        while off + 128 <= len {
            let q = p.add(off);
            let fold = |x: __m128i, y: __m128i| {
                let h = _mm_clmulepi64_si128(x, k, 0x11);
                let l = _mm_clmulepi64_si128(x, k, 0x00);
                _mm_xor_si128(_mm_xor_si128(h, l), y)
            };
            x0 = fold(x0, ld(q));
            x1 = fold(x1, ld(q.add(16)));
            x2 = fold(x2, ld(q.add(32)));
            x3 = fold(x3, ld(q.add(48)));
            x4 = fold(x4, ld(q.add(64)));
            x5 = fold(x5, ld(q.add(80)));
            x6 = fold(x6, ld(q.add(96)));
            x7 = fold(x7, ld(q.add(112)));
            off += 128;
        }
        let x = fold8_to_1_vex(x0, x1, x2, x3, x4, x5, x6, x7);
        finish_from_lane_vex(x, p, off, len, rev128)
    }

    #[repr(align(64))]
    struct Ring32B8([[__m256i; 4]; 8]);

    // -----------------------------------------------------------------------
    // C2: 8-lane (128 B/iter), 4x 256-bit `vpshufb` -> 8-slot (1024 B) aligned L1d ring
    //     (7 blocks = 896 B = 308 uops ahead > 192 Haswell ROB & > 224 Comet Lake ROB!)
    //     10.0 Port-5 uops / 64 B, 0 B `TABLES`, zero `vzeroupper` on `len < 1280`!
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn c2_avx2_ring8_8x(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 1280 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring32B8([[_mm256_setzero_si256(); 4]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        let store_block = |src: *const u8, dst: *mut u8| {
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(src as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
            let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
            let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
            _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
            _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
        };

        for s in 0..8 {
            store_block(p.add(s * 128), rptr.add(s * 128));
        }

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
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            store_block(p.add((b + 7) * 128), rptr.add(((b + 7) & 7) * 128));
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

    // -----------------------------------------------------------------------
    // C3: Same as C2 (8-slot AVX2 ring, 8-lane) + software prefetch 512 B ahead
    // -----------------------------------------------------------------------
    #[inline(never)]
    #[target_feature(enable = "avx2,pclmulqdq")]
    pub unsafe fn c3_avx2_ring8_8x_pf(state: u32, data: &[u8]) -> u32 {
        let len = data.len();
        if len < 1280 {
            return a4_vex_8x_barrett(state, data);
        }
        let rev256 = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let rev128 = _mm256_castsi256_si128(rev256);
        let p = data.as_ptr();
        let blocks = len / 128;
        let mut ring = Ring32B8([[_mm256_setzero_si256(); 4]; 8]);
        let rptr = ring.0.as_mut_ptr() as *mut u8;

        let store_block = |src: *const u8, dst: *mut u8| {
            let w0 = _mm256_shuffle_epi8(_mm256_loadu_si256(src as *const __m256i), rev256);
            let w1 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(32) as *const __m256i), rev256);
            let w2 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(64) as *const __m256i), rev256);
            let w3 = _mm256_shuffle_epi8(_mm256_loadu_si256(src.add(96) as *const __m256i), rev256);
            _mm256_store_si256(dst as *mut __m256i, w0);
            _mm256_store_si256(dst.add(32) as *mut __m256i, w1);
            _mm256_store_si256(dst.add(64) as *mut __m256i, w2);
            _mm256_store_si256(dst.add(96) as *mut __m256i, w3);
        };

        for s in 0..8 {
            store_block(p.add(s * 128), rptr.add(s * 128));
        }

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
            let h = _mm_clmulepi64_si128(x, k, 0x11);
            let l = _mm_clmulepi64_si128(x, k, 0x00);
            let y = _mm_load_si128(src as *const __m128i);
            _mm_xor_si128(_mm_xor_si128(h, l), y)
        };

        for b in 1..(blocks - 7) {
            let q = p.add((b + 7) * 128);
            _mm_prefetch::<_MM_HINT_T0>(q.add(512) as *const i8);
            _mm_prefetch::<_MM_HINT_T0>(q.add(576) as *const i8);
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

    // -----------------------------------------------------------------------
    // CRC-32/BZIP2 zero-copy combine: given `s1 = update(init, a)` and `s2 = update(0, b)`
    // with `b.len() == len2`, returns `update(init, a ++ b) = (s1 * x^(8*len2) mod P) ^ s2`.
    // -----------------------------------------------------------------------
    const fn gf2_mul_mod_p(a: u32, b: u32) -> u32 {
        let prod = gf2_mul_u128(a as u64, b as u64) as u64;
        const P_FULL: u64 = (1u64 << 32) | POLY as u64;
        let mut r = prod;
        let mut bit = 63;
        while bit >= 32 {
            if (r >> bit) & 1 != 0 {
                r ^= P_FULL << (bit - 32);
            }
            bit -= 1;
        }
        r as u32
    }

    /// Precomputed `x^(8 * 2^k) mod P` for `k = 0..32` (128 bytes total, 0 `TABLES`!).
    const fn combine_pows() -> [u32; 32] {
        let mut out = [0u32; 32];
        // k = 0: x^8 mod P = 0x100
        let mut cur = xpow_mod(8) as u32;
        let mut k = 0;
        while k < 32 {
            out[k] = cur;
            cur = gf2_mul_mod_p(cur, cur);
            k += 1;
        }
        out
    }

    static COMBINE_POWS: [u32; 32] = combine_pows();
    const POW_32K: u32 = COMBINE_POWS[15]; // x^(8 * 32768) mod P

    /// Portable O(log len2) GF(2) CRC-32/BZIP2 register combine (no tables beyond 128 B).
    #[inline(never)]
    pub fn combine_scalar(mut s1: u32, s2: u32, mut len2: usize) -> u32 {
        let mut k = 0usize;
        while len2 != 0 {
            if (len2 & 1) != 0 {
                s1 = gf2_mul_mod_p(s1, COMBINE_POWS[k]);
            }
            len2 >>= 1;
            k += 1;
        }
        s1 ^ s2
    }

    /// PCLMULQDQ + Barrett O(popcnt(len2)) CRC-32/BZIP2 register combine (0 `TABLES`).
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn combine_clmul(mut s1: u32, s2: u32, mut len2: usize) -> u32 {
        let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
        while len2 != 0 {
            let k = len2.trailing_zeros() as usize;
            len2 &= len2 - 1;
            let prod = _mm_clmulepi64_si128(
                _mm_cvtsi64_si128(s1 as i64),
                _mm_cvtsi64_si128(COMBINE_POWS[k] as i64),
                0x00,
            );
            let u_bits = _mm_cvtsi128_si64(prod) as u64;
            let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
            let q = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
            let p_lo = _mm_cvtsi128_si64(_mm_clmulepi64_si128(q, mu_poly, 0x10)) as u32;
            s1 = (u_bits as u32) ^ p_lo;
        }
        s1 ^ s2
    }

    /// Single-step PCLMULQDQ combine for fixed 32 KiB slabs (3 `pclmulqdq`s = ~20 cycles!).
    #[inline(never)]
    #[target_feature(enable = "avx,pclmulqdq")]
    pub unsafe fn combine_32k_clmul(s1: u32, s2: u32) -> u32 {
        let mu_poly = _mm_set_epi64x(POLY as i64, MU as i64);
        let prod = _mm_clmulepi64_si128(
            _mm_cvtsi64_si128(s1 as i64),
            _mm_cvtsi64_si128(POW_32K as i64),
            0x00,
        );
        let u_bits = _mm_cvtsi128_si64(prod) as u64;
        let u_hi = _mm_cvtsi64_si128((u_bits >> 32) as i64);
        let q = _mm_srli_epi64::<32>(_mm_clmulepi64_si128(u_hi, mu_poly, 0x00));
        let p_lo = _mm_cvtsi128_si64(_mm_clmulepi64_si128(q, mu_poly, 0x10)) as u32;
        ((u_bits as u32) ^ p_lo) ^ s2
    }

    #[derive(Clone, Copy)]
    pub struct Arm {
        pub name: &'static str,
        pub f: fn(u32, &[u8]) -> u32,
        pub needs_avx2: bool,
        pub needs_movbe: bool,
    }

    macro_rules! wrap_arm {
        ($fn_name:ident, $inner:ident) => {
            fn $fn_name(s: u32, d: &[u8]) -> u32 {
                unsafe { $inner(s, d) }
            }
        };
    }

    wrap_arm!(w_a1, a1_shipped_sse_4x_table);
    wrap_arm!(w_a2, a2_sse_4x_barrett);
    wrap_arm!(w_a3, a3_vex_4x_barrett);
    wrap_arm!(w_a4, a4_vex_8x_barrett);
    wrap_arm!(w_a5, a5_avx2_vpshufb_extract_4x);
    wrap_arm!(w_a6, a6_avx2_vpshufb_store_4x);
    wrap_arm!(w_a7, a7_avx2_vpshufb_store_8x);
    wrap_arm!(w_a8, a8_movbe_ring_4x);
    wrap_arm!(w_a9, a9_movbe_ring_8x);
    wrap_arm!(w_a10, a10_bswap_ring_4x);
    wrap_arm!(w_a11, a11_hybrid_3movbe_1pshufb_4x);
    wrap_arm!(w_a12, a12_hybrid_2movbe_1vpshufb256_4x);
    wrap_arm!(w_a13, a13_avx2_vpshufb_1slot_4x);
    wrap_arm!(w_a14, a14_hybrid_2movbe_2pshufb_4x);
    wrap_arm!(w_a15, a15_aligned_vex_8x_barrett);
    wrap_arm!(w_b1, b1_true_movbe_ring8_4x);
    wrap_arm!(w_b2, b2_true_movbe_ring8_8x);
    wrap_arm!(w_b3, b3_true_hyb_4movbe_2avx2_8x);
    wrap_arm!(w_b4, b4_true_hyb_3movbe_1shuf_4x);
    wrap_arm!(w_b5, b5_true_hyb_2movbe_2shuf_4x);
    wrap_arm!(w_b6, b6_avx2_pingpong_8x);
    wrap_arm!(w_b7, b7_avx2_strip512_8x);
    wrap_arm!(w_b8, b8_true_movbe_strip1k_8x);
    wrap_arm!(w_c2, c2_avx2_ring8_8x);
    wrap_arm!(w_c3, c3_avx2_ring8_8x_pf);

    pub fn all_arms() -> Vec<Arm> {
        let has_avx2 = is_x86_feature_detected!("avx2");
        let has_movbe = is_x86_feature_detected!("movbe");
        let raw = [
            Arm {
                name: "A0_shipped_crate",
                f: a0_shipped_crate,
                needs_avx2: false,
                needs_movbe: false,
            },
            Arm {
                name: "A1_shipped_sse_4x_table",
                f: w_a1,
                needs_avx2: false,
                needs_movbe: false,
            },
            Arm {
                name: "A2_sse_4x_barrett",
                f: w_a2,
                needs_avx2: false,
                needs_movbe: false,
            },
            Arm {
                name: "A3_vex_4x_barrett",
                f: w_a3,
                needs_avx2: false,
                needs_movbe: false,
            },
            Arm {
                name: "A4_vex_8x_barrett",
                f: w_a4,
                needs_avx2: false,
                needs_movbe: false,
            },
            Arm {
                name: "A5_avx2_extract_4x",
                f: w_a5,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "A6_avx2_ring_4x",
                f: w_a6,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "A7_avx2_ring_8x",
                f: w_a7,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "A13_avx2_1slot_4x",
                f: w_a13,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "C2_avx2_ring8_8x",
                f: w_c2,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "C3_avx2_ring8_8x_pf",
                f: w_c3,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "B1_movbe_ring8_4x",
                f: w_b1,
                needs_avx2: false,
                needs_movbe: true,
            },
            Arm {
                name: "B2_movbe_ring8_8x",
                f: w_b2,
                needs_avx2: false,
                needs_movbe: true,
            },
            Arm {
                name: "B3_hyb_4movbe_2avx2_8x",
                f: w_b3,
                needs_avx2: true,
                needs_movbe: true,
            },
            Arm {
                name: "B6_avx2_pingpong_8x",
                f: w_b6,
                needs_avx2: true,
                needs_movbe: false,
            },
            Arm {
                name: "B7_avx2_strip512_8x",
                f: w_b7,
                needs_avx2: true,
                needs_movbe: false,
            },
        ];
        // Keep other exploratory variants referenced for objdump comparison if desired.
        let _ = (
            w_a8, w_a9, w_a10, w_a11, w_a12, w_a14, w_a15, w_b4, w_b5, w_b8,
        );
        raw.into_iter()
            .filter(|a| (!a.needs_avx2 || has_avx2) && (!a.needs_movbe || has_movbe))
            .collect()
    }

    pub fn check_const_identities() {
        const P_FULL: u64 = (1u64 << 32) | POLY as u64;
        let prod = gf2_mul_u128(MU, P_FULL);
        assert_eq!(prod >> 32, 1u128 << 32, "Barrett MU * P != x^64 + r(<32)");
        assert_eq!(MU, 0x1_04C1_1DB7_u64 ^ (0x1_04C1_1DB7_u64 ^ MU)); // sanity
                                                                      // Cross-check xpow_mod against update_bytewise on [1, 0, ..., 0]
        assert_eq!(xpow_mod(32), POLY as u64);
        let mut z = vec![0u8; 17];
        z[0] = 1;
        // [1, 0*16] represents x^128; CRC register is x^(128+32) mod P = x^160 mod P
        assert_eq!(crc::update_bytewise(0, &z) as u64, xpow_mod(128 + 32));
    }
}

#[cfg(target_arch = "x86_64")]
mod bench {
    use super::x86::*;
    use mibz2::crc;
    use mibz2::Kernels;
    use std::hint::black_box;
    use std::path::Path;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn bytes(&mut self, n: usize) -> Vec<u8> {
            (0..n).map(|_| self.next() as u8).collect()
        }
    }

    /// Thread CPU time in nanoseconds via `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`.
    #[cfg(target_os = "linux")]
    fn thread_cpu_ns() -> u64 {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
        }
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
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

    fn stats(vals: &[f64]) -> (f64, f64, f64) {
        let mut s = vals.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    }

    /// Decode all 431 real blocks of `wikidata.bz2` individually with their stored CRCs.
    fn load_wikidata_blocks(path: &Path) -> Vec<(Vec<u8>, u32)> {
        let raw = std::fs::read(path).expect("read wikidata.bz2");
        let k = Kernels::detect();
        let mut out = Vec::new();
        for span in mibz2::scan(&raw) {
            let span = span.expect("scan span");
            let mut blk = Vec::new();
            mibz2::decode_blocks(&raw, span.start_bit, span.end_bit, span.level, k, &mut blk)
                .expect("decode block");
            out.push((blk, span.crc));
        }
        out
    }

    pub fn run_verify(wikidata: &Path) {
        check_const_identities();
        let arms = all_arms();
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let rand_buf = rng.bytes(5100);
        let zeros = vec![0u8; 5100];
        let ones = vec![0xFFu8; 5100];
        let states = [0u32, !0, 0x8000_0000, 1, 0x1234_5678, 0xDEAD_BEEF];
        let mut checks: u64 = 0;

        // 1. Standard check value "123456789" -> 0xFC89_1918
        for arm in &arms {
            assert_eq!(!(arm.f)(!0, b"123456789"), 0xFC89_1918, "{}", arm.name);
            checks += 1;
        }

        // 2. All lengths 0..=5000 on pseudorandom with initial !0
        let mut ref_crc = vec![0u32; 5001];
        let mut cur = !0u32;
        ref_crc[0] = cur;
        for i in 0..5000 {
            cur = crc::update_bytewise(cur, &rand_buf[i..i + 1]);
            ref_crc[i + 1] = cur;
        }
        for arm in &arms {
            for n in 0..=5000 {
                let got = (arm.f)(!0, &rand_buf[..n]);
                assert_eq!(got, ref_crc[n], "{} n={n}", arm.name);
                checks += 1;
            }
        }

        // 3. All alignments 0..64 x lengths 0..=300 & 4096..=4170 x multiple states & patterns
        for buf in [&rand_buf, &zeros, &ones] {
            for &s in &states {
                for off in 0..64 {
                    for &n in &[
                        0usize, 1, 2, 3, 4, 7, 8, 12, 13, 14, 15, 16, 17, 31, 32, 33, 63, 64, 65,
                        127, 128, 129, 255, 256, 257, 383, 384, 385, 639, 640, 641, 1023, 1024,
                        4096, 4111,
                    ] {
                        let sl = &buf[off..off + n];
                        let want = crc::update_bytewise(s, sl);
                        for arm in &arms {
                            let got = (arm.f)(s, sl);
                            assert_eq!(got, want, "{} s={s:#x} off={off} n={n}", arm.name);
                            checks += 1;
                        }
                    }
                }
            }
        }

        // 4. Multi-piece streaming splits across tiles
        let big = rng.bytes((3 << 16) + 333);
        let want_big = crc::update_bytewise(!0, &big);
        for arm in &arms {
            for tile in [
                1usize, 7, 15, 16, 31, 64, 127, 128, 129, 384, 640, 4096, 32768, 65536,
            ] {
                let mut s = !0u32;
                for c in big.chunks(tile) {
                    s = (arm.f)(s, c);
                }
                assert_eq!(s, want_big, "{} tile={tile}", arm.name);
                checks += 1;
            }
        }

        // 5. Real wikidata decoded payload (209.7 MB) both whole-block and 32K/64K tiled
        if wikidata.exists() {
            let blocks = load_wikidata_blocks(wikidata);
            for (bi, (blk, want_crc)) in blocks.iter().enumerate() {
                for arm in &arms {
                    let got = !(arm.f)(!0, blk);
                    assert_eq!(got, *want_crc, "{} wikidata blk {bi}", arm.name);
                    let mut s32 = !0u32;
                    for c in blk.chunks(32768) {
                        s32 = (arm.f)(s32, c);
                    }
                    assert_eq!(!s32, *want_crc, "{} wikidata 32K blk {bi}", arm.name);
                    checks += 2;
                }
            }
            println!(
                "wikidata.bz2 verified: {} blocks ({} bytes) across {} arms",
                blocks.len(),
                blocks.iter().map(|(b, _)| b.len()).sum::<usize>(),
                arms.len()
            );
        }

        // 6. `crc32_combine` exactness across 500 random splits + exact 32 KiB slabs
        for _ in 0..500 {
            let split = (rng.next() as usize) % big.len();
            let s1 = crc::update_with(!0, &big[..split], Kernels::detect());
            let s2 = crc::update_with(0, &big[split..], Kernels::detect());
            let len2 = big.len() - split;
            assert_eq!(combine_scalar(s1, s2, len2), want_big);
            let cl = unsafe { combine_clmul(s1, s2, len2) };
            assert_eq!(cl, want_big);
            checks += 2;
        }
        // 32 KiB fixed combine check
        let s1 = crc::update_with(!0, &big[..32768], Kernels::detect());
        let s2 = crc::update_with(0, &big[32768..65536], Kernels::detect());
        let want64k = crc::update_with(!0, &big[..65536], Kernels::detect());
        assert_eq!(unsafe { combine_32k_clmul(s1, s2) }, want64k);
        checks += 1;

        println!(
            "VERIFY PASS: {} / {} bit-exact checks passed across {} arms + crc32_combine",
            checks,
            checks,
            arms.len()
        );
    }

    pub fn run_bench_sizes(wikidata: &Path, repeats: usize) {
        let arms = all_arms();
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let base_buf = rng.bytes(1 << 20); // 1 MiB
        let sizes: &[(usize, usize, &str)] = &[
            (64, 524_288, "64B"),
            (512, 131_072, "512B"),
            (4096, 32_768, "4KiB"),
            (32768, 8_192, "32KiB"),
            (65536, 4_096, "64KiB"),
            (900_000, 300, "900KiB"),
        ];

        println!(
            "\n=== PART 1: Isolated CRC-32/BZIP2 Size Sweep (L1d/L2/L3-warm, {} interleaved repeats, thread CPU) ===",
            repeats
        );
        for &(sz, iters, label) in sizes {
            let total_bytes = (sz * iters) as f64;
            let total_mb = total_bytes / 1e6;
            let slice = &base_buf[..sz];
            // Warmup
            for arm in &arms {
                let mut acc = !0u32;
                for _ in 0..(iters / 8).max(1) {
                    acc = (arm.f)(acc, black_box(slice));
                }
                black_box(acc);
            }
            let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
            for r in 0..repeats {
                let order: Vec<usize> = if r % 2 == 0 {
                    (0..arms.len()).collect()
                } else {
                    (0..arms.len()).rev().collect()
                };
                for idx in order {
                    let arm = &arms[idx];
                    let t0 = thread_cpu_ns();
                    let mut acc = !0u32;
                    for _ in 0..iters {
                        acc = (arm.f)(acc, black_box(slice));
                    }
                    let dt_ms = (thread_cpu_ns() - t0) as f64 / 1e6;
                    black_box(acc);
                    times[idx].push(dt_ms);
                }
            }
            let (b_med, b_min, b_max) = stats(&times[0]);
            println!("\n--- Size {} ({:.1} MB/repeat) ---", label, total_mb);
            println!(
                "{:<24} {:>8} {:>18} {:>9} {:>9} {:>10}",
                "arm", "med_ms", "[min..max]", "GB/s", "vs_A0", "disjoint"
            );
            for (i, arm) in arms.iter().enumerate() {
                let (med, mn, mx) = stats(&times[i]);
                let gbps = total_bytes / (med * 1e6);
                let pct = (med - b_med) / b_med * 100.0;
                let disj = if i == 0 {
                    "-"
                } else if mx < b_min {
                    "FASTER"
                } else if mn > b_max {
                    "SLOWER"
                } else {
                    "overlap"
                };
                println!(
                    "{:<24} {:>8.2} [{:>7.2}..{:>7.2}] {:>8.2} {:>+8.2}% {:>10}",
                    arm.name, med, mn, mx, gbps, pct, disj
                );
            }
        }

        // Cold-stream on real decoded wikidata (209.7 MB >> 45 MiB L3)
        if wikidata.exists() {
            let blocks = load_wikidata_blocks(wikidata);
            let total_bytes: usize = blocks.iter().map(|(b, _)| b.len()).sum();
            let total_mb = total_bytes as f64 / 1e6;
            for &(tile, label) in &[
                (900_000usize, "wikidata_cold_fullblk"),
                (65536, "wikidata_cold_64KiB"),
                (32768, "wikidata_cold_32KiB"),
            ] {
                println!(
                    "\n=== PART 2: L3-Cold {} ({:.2} MB, {} interleaved repeats, thread CPU) ===",
                    label, total_mb, repeats
                );
                let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
                // Warmup one pass
                for arm in &arms {
                    let mut acc = 0u32;
                    for (blk, _) in &blocks {
                        for c in blk.chunks(tile) {
                            acc ^= (arm.f)(!0, black_box(c));
                        }
                    }
                    black_box(acc);
                }
                for r in 0..repeats {
                    let order: Vec<usize> = if r % 2 == 0 {
                        (0..arms.len()).collect()
                    } else {
                        (0..arms.len()).rev().collect()
                    };
                    for idx in order {
                        let arm = &arms[idx];
                        let t0 = thread_cpu_ns();
                        let mut acc = 0u32;
                        for (blk, _) in &blocks {
                            let mut s = !0u32;
                            for c in blk.chunks(tile) {
                                s = (arm.f)(s, black_box(c));
                            }
                            acc ^= !s;
                        }
                        let dt_ms = (thread_cpu_ns() - t0) as f64 / 1e6;
                        black_box(acc);
                        times[idx].push(dt_ms);
                    }
                }
                let (b_med, b_min, b_max) = stats(&times[0]);
                println!(
                    "{:<24} {:>8} {:>18} {:>9} {:>9} {:>10}",
                    "arm", "med_ms", "[min..max]", "GB/s", "vs_A0", "disjoint"
                );
                for (i, arm) in arms.iter().enumerate() {
                    let (med, mn, mx) = stats(&times[i]);
                    let gbps = (total_bytes as f64) / (med * 1e6);
                    let pct = (med - b_med) / b_med * 100.0;
                    let disj = if i == 0 {
                        "-"
                    } else if mx < b_min {
                        "FASTER"
                    } else if mn > b_max {
                        "SLOWER"
                    } else {
                        "overlap"
                    };
                    println!(
                        "{:<24} {:>8.2} [{:>7.2}..{:>7.2}] {:>8.2} {:>+8.2}% {:>10}",
                        arm.name, med, mn, mx, gbps, pct, disj
                    );
                }
            }
        }
    }

    /// Coupled L1d-pollution benchmark: interleaves a 24 KiB L1d-resident random pointer-chase
    /// working set (384 x 64B lines, mimicking Open worker `OutQ16`/`inline16`/hot `edges`)
    /// with CRC calls of varying tile sizes (`4 KiB`, `16 KiB`, `32 KiB`, `64 KiB`).
    /// Also benchmarks `crc32_combine` (25,860 combines = 15/block * 1,724 blocks on wikidata x4).
    pub fn run_bench_l1_and_combine(repeats: usize) {
        let arms = all_arms();
        let mut rng = Rng(0x517c_c1b7_2722_0a95);
        // Build single-cycle random permutation over 384 cache lines (24 KiB = 75% of 32 KiB L1d).
        const LINES: usize = 384;
        let mut perm: Vec<usize> = (0..LINES).collect();
        for i in (1..LINES).rev() {
            let j = (rng.next() as usize) % (i + 1);
            perm.swap(i, j);
        }
        let mut ws = vec![0u32; LINES * 16]; // 384 * 64B = 24 KiB
        for i in 0..LINES {
            let cur = perm[i];
            let nxt = perm[(i + 1) % LINES];
            ws[cur * 16] = (nxt * 16) as u32;
        }
        let payload = rng.bytes(1 << 20);

        println!(
            "\n=== PART 3: Coupled 24 KiB L1d Working-Set Eviction + CRC Interleaving ({} repeats) ===",
            repeats
        );
        for &(tile, rounds, label) in &[
            (4096usize, 8_192usize, "4KiB_tile+24K_L1ws"),
            (16384, 4_096, "16KiB_tile+24K_L1ws"),
            (32768, 2_048, "32KiB_tile+24K_L1ws"),
            (65536, 1_024, "64KiB_tile+24K_L1ws"),
        ] {
            let mut tot_times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
            let mut ws_times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
            for r in 0..repeats {
                let order: Vec<usize> = if r % 2 == 0 {
                    (0..arms.len()).collect()
                } else {
                    (0..arms.len()).rev().collect()
                };
                for idx in order {
                    let arm = &arms[idx];
                    let mut pos = 0usize;
                    let mut crc_acc = !0u32;
                    let mut ws_ns: u64 = 0;
                    let t_all0 = thread_cpu_ns();
                    for k in 0..rounds {
                        let off = (k * tile) & ((1 << 20) - tile);
                        crc_acc = (arm.f)(crc_acc, black_box(&payload[off..off + tile]));
                        let tw0 = thread_cpu_ns();
                        // Touch all 384 cache lines of 24 KiB working set once (dependent chase).
                        let mut p = pos;
                        for _ in 0..LINES {
                            p = unsafe { *ws.get_unchecked(p) } as usize;
                        }
                        ws_ns += thread_cpu_ns() - tw0;
                        pos = p;
                    }
                    let all_ms = (thread_cpu_ns() - t_all0) as f64 / 1e6;
                    black_box((pos, crc_acc));
                    tot_times[idx].push(all_ms);
                    ws_times[idx].push(ws_ns as f64 / 1e6);
                }
            }
            let (b_med, b_min, b_max) = stats(&tot_times[0]);
            let (w_base, _, _) = stats(&ws_times[0]);
            println!("\n--- {} ({} rounds) ---", label, rounds);
            println!(
                "{:<24} {:>8} {:>18} {:>9} {:>10} {:>10} {:>9}",
                "arm", "tot_ms", "[min..max]", "vs_A0", "disjoint", "ws_ms", "ws_vs_A0"
            );
            for (i, arm) in arms.iter().enumerate() {
                let (med, mn, mx) = stats(&tot_times[i]);
                let (wm, _, _) = stats(&ws_times[i]);
                let pct = (med - b_med) / b_med * 100.0;
                let wpct = (wm - w_base) / w_base * 100.0;
                let disj = if i == 0 {
                    "-"
                } else if mx < b_min {
                    "FASTER"
                } else if mn > b_max {
                    "SLOWER"
                } else {
                    "overlap"
                };
                println!(
                    "{:<24} {:>8.2} [{:>7.2}..{:>7.2}] {:>+8.2}% {:>10} {:>9.2} {:>+8.2}%",
                    arm.name, med, mn, mx, pct, disj, wm, wpct
                );
            }
        }

        // Part 4: `crc32_combine` cost (25,860 combines = 15/blk * 1,724 blks on wikidata x4)
        println!(
            "\n=== PART 4: crc32_combine (25,860 combines = wikidata x4 @ 15/blk, {} repeats) ===",
            repeats
        );
        const N_COMB: usize = 25_860;
        let lens: Vec<usize> = (0..N_COMB)
            .map(|i| 16_384 + ((i * 7919) % 32_768))
            .collect();
        let mut t_sc = Vec::new();
        let mut t_cl = Vec::new();
        let mut t_32 = Vec::new();
        for _ in 0..repeats {
            let t0 = thread_cpu_ns();
            let mut acc = 0x1234_5678u32;
            for &l in &lens {
                acc = combine_scalar(acc, 0x9abc_def0, black_box(l));
            }
            t_sc.push((thread_cpu_ns() - t0) as f64 / 1e6);
            black_box(acc);

            let t0 = thread_cpu_ns();
            let mut acc = 0x1234_5678u32;
            for &l in &lens {
                acc = unsafe { combine_clmul(acc, 0x9abc_def0, black_box(l)) };
            }
            t_cl.push((thread_cpu_ns() - t0) as f64 / 1e6);
            black_box(acc);

            let t0 = thread_cpu_ns();
            let mut acc = 0x1234_5678u32;
            for _ in 0..N_COMB {
                acc = unsafe { combine_32k_clmul(acc, 0x9abc_def0) };
            }
            t_32.push((thread_cpu_ns() - t0) as f64 / 1e6);
            black_box(acc);
        }
        for (name, v) in [
            ("combine_scalar_varlen", &t_sc),
            ("combine_clmul_varlen", &t_cl),
            ("combine_32k_clmul_const", &t_32),
        ] {
            let (m, mn, mx) = stats(v);
            println!(
                "{:<26} {:>7.3} ms [{:.3}..{:.3}] ({:.1} ns/combine)",
                name,
                m,
                mn,
                mx,
                m * 1e6 / N_COMB as f64
            );
        }
    }

    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn find_runs_avx2_local(rle: &[u8], runs: &mut Vec<(u32, u8, u8)>) {
        use std::arch::x86_64::*;
        runs.clear();
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0usize;
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
            runs.push((j as u32, *p.add(j), *p.add(j + 4)));
            i = j + 5;
        }
        while i + 3 < n {
            let c = rle[i];
            if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
                runs.push((i as u32, c, rle[i + 4]));
                i += 5;
            } else {
                i += 1;
            }
        }
    }

    /// Faithful mirror of `src/rle.rs::unrle_in_place` parameterized by CRC function & tile size.
    fn unrle_in_place_with(
        buf: &mut Vec<u8>,
        runs: &mut Vec<(u32, u8, u8)>,
        tile: usize,
        crc_fn: fn(u32, &[u8]) -> u32,
    ) -> u32 {
        let n = buf.len();
        unsafe { find_runs_avx2_local(buf, runs) };
        let Some(&(first, _, _)) = runs.first() else {
            return !crc_fn(!0, buf);
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        let first = first as usize;
        if shift > 0 {
            buf.reserve_exact(shift);
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= tile {
                *s = crc_fn(*s, &buf[*at..*at + tile]);
                *at += tile;
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
        s = crc_fn(s, &buf[at..]);
        !s
    }

    /// Real `wikidata.bz2` A9 (`unrle_in_place` + CRC32) coupled benchmark across all 431
    /// pre-RLE1 blocks (both L3-warm x4 = 838.80 MB and L3-cold x1 = 209.70 MB).
    pub fn run_bench_a9(wikidata: &Path, repeats: usize) {
        if !wikidata.exists() {
            return;
        }
        let raw = std::fs::read(wikidata).expect("read wikidata.bz2");
        let k = Kernels::detect();
        let mut sc = mibz2::stream::Scanner::new(mibz2::stream::SliceSource(&raw));
        let mut syms = Vec::new();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let mut tt = mibz2::bwt::TVector::default();
        let mut pre_rle: Vec<(Vec<u8>, u32)> = Vec::new();
        while let mibz2::stream::Event::Block { symbols, .. } = sc.next(syms).expect("scan block") {
            let n = mibz2::mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf");
            mibz2::bwt::build(&ll[..n], &counts, &mut tt);
            let mut inv = Vec::with_capacity(n);
            mibz2::bwt::walk(&tt, symbols.orig_ptr, &mut inv);
            pre_rle.push((inv, symbols.crc));
            syms = symbols.syms;
        }

        let want_names = [
            "A0_shipped_crate",
            "A2_sse_4x_barrett",
            "A4_vex_8x_barrett",
            "A6_avx2_ring_4x",
            "A7_avx2_ring_8x",
            "C2_avx2_ring8_8x",
            "C3_avx2_ring8_8x_pf",
        ];
        let arms: Vec<Arm> = all_arms()
            .into_iter()
            .filter(|a| want_names.contains(&a.name))
            .collect();

        // Exactness check of `unrle_in_place_with` across all 431 blocks x selected arms
        let mut tmp = Vec::with_capacity(1 << 20);
        let mut runs = Vec::with_capacity(2048);
        for arm in &arms {
            for (bi, (blk, want_crc)) in pre_rle.iter().enumerate() {
                tmp.clear();
                tmp.extend_from_slice(blk);
                let got = unrle_in_place_with(&mut tmp, &mut runs, 65536, arm.f);
                assert_eq!(got, *want_crc, "A9 verify {} blk {bi}", arm.name);
            }
        }

        // L3-warm A9 (x4 = 838.80 MB, 1,724 blocks, matching `rle1_crc_census.rs` regime)
        for &(tile, tlabel) in &[(65536usize, "64KiB"), (32768usize, "32KiB")] {
            println!(
                "\n=== PART 5A: Real wikidata x4 A9 (unrle_in_place, {} tiles, L3-warm, {} repeats) ===",
                tlabel, repeats
            );
            let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
            for r in 0..repeats {
                let order: Vec<usize> = if r % 2 == 0 {
                    (0..arms.len()).collect()
                } else {
                    (0..arms.len()).rev().collect()
                };
                for idx in order {
                    let arm = &arms[idx];
                    let mut total_ns: u64 = 0;
                    let mut acc = 0u32;
                    for _ in 0..4 {
                        for (blk, _) in &pre_rle {
                            tmp.clear();
                            tmp.reserve(blk.len() + blk.len() / 16 + 64);
                            tmp.extend_from_slice(blk);
                            let t0 = thread_cpu_ns();
                            acc ^= unrle_in_place_with(&mut tmp, &mut runs, tile, arm.f);
                            total_ns += thread_cpu_ns() - t0;
                        }
                    }
                    black_box(acc);
                    times[idx].push(total_ns as f64 / 1e6);
                }
            }
            let (b_med, b_min, b_max) = stats(&times[0]);
            println!(
                "{:<24} {:>8} {:>18} {:>9} {:>10}",
                "arm", "med_ms", "[min..max]", "vs_A0", "disjoint"
            );
            for (i, arm) in arms.iter().enumerate() {
                let (med, mn, mx) = stats(&times[i]);
                let pct = (med - b_med) / b_med * 100.0;
                let disj = if i == 0 {
                    "-"
                } else if mx < b_min {
                    "FASTER"
                } else if mn > b_max {
                    "SLOWER"
                } else {
                    "overlap"
                };
                println!(
                    "{:<24} {:>8.2} [{:>7.2}..{:>7.2}] {:>+8.2}% {:>10}",
                    arm.name, med, mn, mx, pct, disj
                );
            }
        }

        // Cold-stream A9 (x1 = 209.70 MB across 431 distinct buffers >> 45 MiB L3, 64 KiB tiles)
        println!(
            "\n=== PART 5B: Real wikidata A9 (unrle_in_place, 64KiB tiles, L3-cold, {} repeats) ===",
            repeats
        );
        let mut bufs: Vec<Vec<u8>> = pre_rle
            .iter()
            .map(|(b, _)| {
                let mut v = Vec::with_capacity(b.len() + b.len() / 16 + 64);
                v.extend_from_slice(b);
                v
            })
            .collect();
        let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); arms.len()];
        for r in 0..repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for idx in order {
                let arm = &arms[idx];
                for (dst, (src, _)) in bufs.iter_mut().zip(pre_rle.iter()) {
                    dst.clear();
                    dst.extend_from_slice(src);
                }
                let t0 = thread_cpu_ns();
                let mut acc = 0u32;
                for buf in &mut bufs {
                    acc ^= unrle_in_place_with(buf, &mut runs, 65536, arm.f);
                }
                let dt_ms = (thread_cpu_ns() - t0) as f64 / 1e6;
                black_box(acc);
                times[idx].push(dt_ms);
            }
        }
        let (b_med, b_min, b_max) = stats(&times[0]);
        println!(
            "{:<24} {:>8} {:>18} {:>9} {:>10}",
            "arm", "med_ms", "[min..max]", "vs_A0", "disjoint"
        );
        for (i, arm) in arms.iter().enumerate() {
            let (med, mn, mx) = stats(&times[i]);
            let pct = (med - b_med) / b_med * 100.0;
            let disj = if i == 0 {
                "-"
            } else if mx < b_min {
                "FASTER"
            } else if mn > b_max {
                "SLOWER"
            } else {
                "overlap"
            };
            println!(
                "{:<24} {:>8.2} [{:>7.2}..{:>7.2}] {:>+8.2}% {:>10}",
                arm.name, med, mn, mx, pct, disj
            );
        }
    }
}

fn main() {
    #[cfg(not(target_arch = "x86_64"))]
    {
        eprintln!("This probe targets x86_64 PCLMULQDQ/AVX2.");
    }
    #[cfg(target_arch = "x86_64")]
    {
        let args: Vec<String> = std::env::args().collect();
        let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("verify");
        let default_wiki = "corpora/wikidata.bz2";
        let (repeats, wiki_str) = if cmd == "verify" {
            (
                7usize,
                args.get(2).map(|s| s.as_str()).unwrap_or(default_wiki),
            )
        } else {
            (
                args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7),
                args.get(3).map(|s| s.as_str()).unwrap_or(default_wiki),
            )
        };
        let wikidata = std::path::Path::new(wiki_str);
        match cmd {
            "verify" => bench::run_verify(wikidata),
            "bench" => {
                bench::run_bench_sizes(wikidata, repeats);
                bench::run_bench_l1_and_combine(repeats);
                bench::run_bench_a9(wikidata, repeats);
            }
            "sizes" => bench::run_bench_sizes(wikidata, repeats),
            "l1" => bench::run_bench_l1_and_combine(repeats),
            "a9" => bench::run_bench_a9(wikidata, repeats),
            _ => {
                eprintln!("Usage: crc32_bzip2_dissection [verify|bench|sizes|l1|a9] [repeats] [wikidata.bz2]");
                std::process::exit(2);
            }
        }
    }
}
