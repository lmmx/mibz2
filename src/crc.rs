//! CRC-32/BZIP2 (polynomial 0x04C11DB7, MSB first, initial value and final xor all ones),
//! the block CRC bzip2 stores: sliced by 8 (portable), or by carry-less multiplication
//! (PCLMULQDQ with SSE4.1, dispatched at run time, see `Kernels`).

use super::Kernels;

pub const POLY: u32 = 0x04C1_1DB7;

const fn tables() -> [[u32; 256]; 8] {
    let mut t = [[0u32; 256]; 8];
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
    while j < 8 {
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

static TABLES: [[u32; 256]; 8] = tables();

/// The CRC of `data`, by the kernels given
pub fn crc32(data: &[u8], k: Kernels) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if k.clmul && data.len() >= 128 {
            // SAFETY: `k.clmul` is set only when PCLMULQDQ, SSE4.1 and SSSE3 were detected
            return !unsafe { clmul_update(!0, data) };
        }
    }
    let _ = k;
    !update(!0, data)
}

/// The CRC register after `data`, from `s` (no initial or final xor): a byte at a time
/// as bzip2 takes it, 8 at a time where it can
pub fn update(mut s: u32, data: &[u8]) -> u32 {
    let t = &TABLES;
    let mut chunks = data.chunks_exact(8);
    for c in &mut chunks {
        let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        s = t[7][(hi >> 24) as usize]
            ^ t[6][((hi >> 16) & 0xff) as usize]
            ^ t[5][((hi >> 8) & 0xff) as usize]
            ^ t[4][(hi & 0xff) as usize]
            ^ t[3][c[4] as usize]
            ^ t[2][c[5] as usize]
            ^ t[1][c[6] as usize]
            ^ t[0][c[7] as usize];
    }
    for &b in chunks.remainder() {
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

/// x^n mod P (non-reflected, P = x^32 + POLY), as a 32-bit remainder
#[cfg(target_arch = "x86_64")]
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

/// 4 x 128-bit lanes folded 512 bits at a time (x^(512+64) and x^512 mod P), the 64 bytes
/// they come to and the remainder by the table. Needs `data.len() >= 128`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "pclmulqdq,sse4.1,ssse3,sse2")]
unsafe fn clmul_update(state: u32, data: &[u8]) -> u32 {
    use std::arch::x86_64::*;
    const K_HI: u64 = xpow_mod(512 + 64);
    const K_LO: u64 = xpow_mod(512);
    debug_assert!(data.len() >= 128);
    // byte order: the first byte is the highest-degree coefficient
    let rev = _mm_setr_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let p = data.as_ptr();
    // SAFETY (every load): offsets below `blocks * 64 <= data.len()`
    let ld = |q: *const u8| _mm_shuffle_epi8(_mm_loadu_si128(q as *const __m128i), rev);
    let blocks = data.len() / 64;
    let mut x0 = ld(p);
    let mut x1 = ld(p.add(16));
    let mut x2 = ld(p.add(32));
    let mut x3 = ld(p.add(48));
    // the initial state goes into the top 32 bits of the first lane
    x0 = _mm_xor_si128(x0, _mm_set_epi32(state as i32, 0, 0, 0));
    let k = _mm_set_epi64x(K_HI as i64, K_LO as i64);
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
    // the four lanes are congruent (mod P) to the last 64 bytes' worth: through the table
    // from state 0, then the remainder
    let mut tail = [0u8; 64];
    for (i, x) in [x0, x1, x2, x3].into_iter().enumerate() {
        _mm_storeu_si128(
            tail.as_mut_ptr().add(i * 16) as *mut __m128i,
            _mm_shuffle_epi8(x, rev),
        );
    }
    let s = update(0, &tail);
    update(s, &data[blocks * 64..])
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for k in [Kernels::scalar(), Kernels::detect()] {
            for n in (0..300).chain([511, 512, 513, 1023, 4096, 4999, 5000]) {
                let want = !update_bytewise(!0, &data[..n]);
                assert_eq!(crc32(&data[..n], k), want, "n {n} {k:?}");
            }
        }
        // the check value of CRC-32/BZIP2
        assert_eq!(crc32(b"123456789", Kernels::scalar()), 0xFC89_1918);
    }
}
