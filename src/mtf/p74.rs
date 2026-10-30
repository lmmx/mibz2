//! Gate 1 research kernels imported from e57400e; never used by production dispatch.
//! `mtf` and `mtf_checkpoints` with the list's first 32 entries held in a ymm register,
//! the rest (the tail) in memory, so that an explicit symbol's update doesn't go through
//! the store the next symbol's list load can't be forwarded from. The decoder never calls
//! these; `examples/p74_c.rs` retains Free for the assembly comparison, and
//! tests/mtf_p74.rs checks them against the scalar ones.
//!
//! - A rank below 16 (`Arm::Branch`): one `vpshufb` of the head, its lower lane the move to
//!   the front, its upper lane the identity.
//! - A rank of 16-31 (and, in `Arm::Free`, every rank below 32): the head's lanes swapped,
//!   each shuffled, and blended: v_q to byte 0, v_15 to byte 16, the lanes shifted between.
//! - A rank of 32 or more: the head shifted a byte (its byte 31 to the tail's front) and
//!   the tail's entry put at byte 0; the tail's first q - 31 bytes moved a byte on, in
//!   blended 32-byte chunks from the last down, so no `memmove` is called (each chunk's
//!   mask made at run time: a plain copy of the chunks below the last became one).
//!
//! A symbol's byte is the new head's byte 0. A run of up to 32 bytes is written as one
//! 32-byte store (`ll` is reserved 32 bytes past the block's limit), a longer one by
//! `write_bytes`. Results, errors, `ll`, counts and checkpoints are the shipped kernels'.
use super::Symbols;
use crate::ErrorKind;

/// How a rank below 32 is moved: by rank (one shuffle below 16), or every one alike
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Branch,
    Free,
}

/// `super::mtf` with the head in a register (`super::mtf_scalar` without AVX2)
pub fn mtf(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    arm: Arm,
) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected
            return unsafe {
                match arm {
                    Arm::Branch => avx2::mtf::<false>(b, ll, counts),
                    Arm::Free => avx2::mtf::<true>(b, ll, counts),
                }
            };
        }
    }
    let _ = arm;
    super::mtf_scalar(b, ll, counts)
}

/// `super::mtf_checkpoints` with the head in a register (its scalar twin without AVX2)
pub fn mtf_checkpoints(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
    arm: Arm,
) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 was detected
            return unsafe {
                match arm {
                    Arm::Branch => avx2::mtf_checkpoints::<false>(b, ll, counts, checkpoints),
                    Arm::Free => avx2::mtf_checkpoints::<true>(b, ll, counts, checkpoints),
                }
            };
        }
    }
    let _ = arm;
    super::mtf_checkpoints_scalar(b, ll, counts, checkpoints)
}

/// Gate 1's selector (frozen before its code: the pre-registration's "The selector
/// arm"): whether a block takes `Arm::Free`. Yes if its used alphabet (`eob - 1`) has
/// at most 32 symbols, which no rank of 32 or more can reach, without a count; else if
/// it has no explicit symbol, or a tenth or fewer of them (2 <= s < eob) are of rank 32
/// or more (s >= 33), by exact integers
pub fn select(b: &Symbols) -> bool {
    if b.eob.saturating_sub(1) <= 32 {
        return true;
    }
    let (mut explicit, mut high) = (0u64, 0u64);
    for &s in &b.syms {
        explicit += (s >= 2 && s < b.eob) as u64;
        high += (s >= 33 && s < b.eob) as u64;
    }
    explicit == 0 || 10 * high <= explicit
}

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
    use super::super::{Symbols, CHECKPOINT, MAX_RUN_WEIGHT};
    use super::{BLEND, SHIFT, SWAPPED};
    use crate::ErrorKind;
    use std::arch::x86_64::*;

    /// The list past its first 32 entries, with a chunk of slack before it for the
    /// shifted load
    #[repr(align(32))]
    struct Tail([u8; 32 + 224]);

    /// The state the two loops share: the head, the tail and the masks' constants
    macro_rules! state {
        ($head:ident, $tail:ident, $base:ident, $idx:ident, $two:ident) => {
            let mut $head = _mm256_setr_epi8(
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31,
            );
            let mut $tail = Tail([0u8; 32 + 224]);
            for i in 0..224 {
                $tail.0[32 + i] = (32 + i) as u8;
            }
            let $base = $tail.0.as_mut_ptr().add(32);
            let $idx = _mm256_setr_epi8(
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31,
            );
            // bytes 0 and 16: the swapped shuffle's in `Arm::Branch`'s blend
            let $two = _mm256_setr_epi8(
                -128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -128, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            );
        };
    }

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

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn mtf<const FREE: bool>(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max + 32);
        *counts = [0; 256];
        state!(head, tail, base, idx, two);
        let s2u = &b.seq_to_unseq;
        let (mut run, mut weight) = (0u32, 1u32);
        // `n` bytes written at `out`; every write below is checked against `max`, and
        // a run's 32-byte store stays inside `ll`'s capacity, `max` + 32
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
                let c = s2u[front!(head) as usize];
                counts[c as usize] += run;
                fill!(out.add(n), c, run as usize);
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
            let v = moved!(s, head, base, idx, two, FREE);
            let c = s2u[v as usize];
            counts[c as usize] += 1;
            *out.add(n) = c;
            n += 1;
        }
        // SAFETY: the first n bytes were written above, n <= max <= capacity
        ll.set_len(n);
        result.map(|()| n)
    }

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn mtf_checkpoints<const FREE: bool>(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> Result<usize, ErrorKind> {
        let max = b.max_len;
        ll.clear();
        ll.reserve(max + 32);
        *counts = [0; 256];
        checkpoints.clear();
        checkpoints.push([0; 256]);
        state!(head, tail, base, idx, two);
        let s2u = &b.seq_to_unseq;
        let (mut run, mut weight) = (0u32, 1u32);
        // as in `mtf`; the limit is at most `max`
        let out = ll.as_mut_ptr();
        let mut n = 0usize;
        let mut result = Ok(());
        macro_rules! byte {
            ($s:expr) => {{
                let v = moved!($s, head, base, idx, two, FREE);
                let c = s2u[v as usize];
                counts[c as usize] += 1;
                *out.add(n) = c;
                n += 1;
            }};
        }
        let mut next = CHECKPOINT;
        let mut syms = b.syms.iter();
        // `super::super::mtf_checkpoints_avx2`'s loops, its list ops replaced
        'block: loop {
            let limit = max.min(next);
            let mut at = None;
            for &s in syms.by_ref() {
                if s <= 1 {
                    if weight >= MAX_RUN_WEIGHT {
                        result = Err(ErrorKind::RunTooLong);
                        break 'block;
                    }
                    run += (s as u32 + 1) * weight;
                    weight <<= 1;
                    continue;
                }
                if run != 0 {
                    if n + run as usize > limit {
                        at = Some(s);
                        break;
                    }
                    let c = s2u[front!(head) as usize];
                    counts[c as usize] += run;
                    fill!(out.add(n), c, run as usize);
                    n += run as usize;
                    (run, weight) = (0, 1);
                }
                if s == b.eob {
                    break 'block;
                }
                if n >= limit {
                    at = Some(s);
                    break;
                }
                byte!(s);
            }
            let Some(s) = at else { break 'block };
            if run != 0 {
                if n + run as usize > max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
                let c = s2u[front!(head) as usize];
                let mut left = run as usize;
                while n + left >= next {
                    let part = next - n;
                    counts[c as usize] += part as u32;
                    fill!(out.add(n), c, part);
                    (n, left) = (next, left - part);
                    checkpoints.push(*counts);
                    next += CHECKPOINT;
                }
                counts[c as usize] += left as u32;
                fill!(out.add(n), c, left);
                n += left;
                (run, weight) = (0, 1);
                if s == b.eob {
                    break 'block;
                }
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
            } else {
                if n >= max {
                    result = Err(ErrorKind::BlockTooLong);
                    break 'block;
                }
                checkpoints.push(*counts);
                next += CHECKPOINT;
            }
            byte!(s);
        }
        if result.is_ok() && n == next {
            checkpoints.push(*counts);
        }
        // SAFETY: the first n bytes were written above, n <= max <= capacity
        ll.set_len(n);
        result.map(|()| n)
    }
}
