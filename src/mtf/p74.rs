//! The P74 list operations the frozen P74-C kernel (`p74_c::combined`) is built from,
//! from Gate 1 (e57400e) as frozen at 8b6caf1: the list's first 32 entries held in a ymm
//! register, the rest (the tail) in memory, so that an explicit symbol's update doesn't
//! go through the store the next symbol's list load can't be forwarded from. Gate 1's own
//! kernels, its `Arm`s and its selector are research and are not kept here; only the
//! masks and the macros the combined kernel expands are, their text unchanged.
//!
//! - A rank of 16-31 (and, as P74-C uses it, every rank below 32): the head's lanes
//!   swapped, each shuffled, and blended: v_q to byte 0, v_15 to byte 16, the lanes
//!   shifted between.
//! - A rank of 32 or more: the head shifted a byte (its byte 31 to the tail's front) and
//!   the tail's entry put at byte 0; the tail's first q - 31 bytes moved a byte on, in
//!   blended 32-byte chunks from the last down, so no `memmove` is called.
//!
//! A symbol's byte is the new head's byte 0. A run of up to 32 bytes is written as one
//! 32-byte store (`ll` is reserved 32 bytes past the block's limit), a longer one by
//! `write_bytes`.

/// The per-rank shuffle and blend masks (index: the rank; 0 unused)
#[repr(align(32))]
pub(super) struct Masks(pub(super) [[u8; 32]; 32]);

/// A rank below 16: the move to the front in the lower lane, the identity in the upper.
/// A rank of 16-31: the lower lane a byte on (byte 0 zeroed) and the upper lane's
/// 16..q a byte on (byte 16 zeroed), the head's own shuffle.
pub(super) static SHIFT: Masks = {
    let mut m = [[0u8; 32]; 32];
    let mut q = 1;
    while q < 32 {
        let mut i = 0;
        while i < 32 {
            m[q][i] = if q < 16 {
                if i >= 16 {
                    (i - 16) as u8
                } else if i == 0 {
                    q as u8
                } else if i <= q {
                    (i - 1) as u8
                } else {
                    i as u8
                }
            } else if i < 16 {
                if i == 0 {
                    0x80
                } else {
                    (i - 1) as u8
                }
            } else if i == 16 {
                0x80
            } else if i - 16 <= q - 16 {
                (i - 17) as u8
            } else {
                (i - 16) as u8
            };
            i += 1;
        }
        q += 1;
    }
    Masks(m)
};

/// A rank of 16-31: from the lane-swapped head, v_q to byte 0 and v_15 to byte 16
/// (the rest zeroed); a rank below 16 takes nothing from it
pub(super) static SWAPPED: Masks = {
    let mut m = [[0x80u8; 32]; 32];
    let mut q = 16;
    while q < 32 {
        m[q][0] = (q - 16) as u8;
        m[q][16] = 15;
        q += 1;
    }
    Masks(m)
};

/// `Arm::Free`'s blend: bytes 0 and 16 from the swapped shuffle for a rank of 16-31,
/// none below 16
pub(super) static BLEND: Masks = {
    let mut m = [[0u8; 32]; 32];
    let mut q = 16;
    while q < 32 {
        m[q][0] = 0x80;
        m[q][16] = 0x80;
        q += 1;
    }
    Masks(m)
};

#[cfg(target_arch = "x86_64")]
pub(super) mod avx2 {
    // (the macros name the masks and intrinsics where they are expanded:
    // `p74_c::combined` imports both)

    /// The head's byte 0: the list's front
    macro_rules! front {
        ($head:ident) => {
            (_mm256_cvtsi256_si32($head) as u8)
        };
    }

    /// Symbol $s's entry moved to the front; evaluates to it
    macro_rules! moved {
        ($s:expr, $head:ident, $base:ident, $idx:ident, $two:ident, $free:expr $(, $fallback:block)?) => {{
            // (s - 1 < 256: a symbol is below the alphabet, at most 257)
            let q = ($s - 1) as usize;
            if q < 32 {
                let shift = _mm256_load_si256(SHIFT.0[q].as_ptr() as *const __m256i);
                if !$free && q < 16 {
                    $head = _mm256_shuffle_epi8($head, shift);
                } else {
                    let swapped = _mm256_permute2x128_si256::<0x01>($head, $head);
                    let a = _mm256_shuffle_epi8($head, shift);
                    let m = _mm256_load_si256(SWAPPED.0[q].as_ptr() as *const __m256i);
                    let b = _mm256_shuffle_epi8(swapped, m);
                    let sel = if $free {
                        _mm256_load_si256(BLEND.0[q].as_ptr() as *const __m256i)
                    } else {
                        $two
                    };
                    $head = _mm256_blendv_epi8(a, b, sel);
                }
                front!($head)
            } else {
                $($fallback)?
            let j = q - 32;
                let v = *$base.add(j);
                let last = _mm256_extract_epi8::<31>($head) as u8;
                // the tail's 0..=j a byte on, chunk by chunk from the last down, each
                // blended by a mask made at run time (a chunk below the last all of it:
                // a plain copy there, LLVM's loop idioms make a `memmove` call)
                for c in (0..=j / 32).rev() {
                    let p = $base.add(32 * c);
                    let old = _mm256_load_si256(p as *const __m256i);
                    let on = _mm256_loadu_si256(p.sub(1) as *const __m256i);
                    let k = (j + 1 - 32 * c).min(32) as i8;
                    let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(k), $idx);
                    _mm256_store_si256(p as *mut __m256i, _mm256_blendv_epi8(old, on, m));
                }
                *$base = last;
                // the head a byte on (byte 0 zero), then v at byte 0
                let low = _mm256_permute2x128_si256::<0x08>($head, $head);
                let on = _mm256_alignr_epi8::<15>($head, low);
                $head =
                    _mm256_or_si256(on, _mm256_zextsi128_si256(_mm_cvtsi32_si128(v as i32)));
                v
            }
        }};
    }

    /// `len` bytes of `c` at `at`: one 32-byte store up to 32 (the buffer reserved 32
    /// bytes past its limit), else `write_bytes`
    macro_rules! fill {
        ($at:expr, $c:expr, $len:expr) => {{
            let (at, c, len) = ($at, $c, $len);
            if len <= 32 {
                _mm256_storeu_si256(at as *mut __m256i, _mm256_set1_epi8(c as i8));
            } else {
                std::ptr::write_bytes(at, c, len);
            }
        }};
    }

    pub(in crate::mtf) use {fill, front, moved};
}
