//! Frozen Option C (§12a of the 2026-10-30 worst-case note), research only.
//! T0 calls the standalone shipped kernel, T1 enters the combined kernel in shipped
//! mode, T2 uses the frozen dispatch and one-way guard. No production caller uses this.
//!
//! As with the shipped MTF stage, `Symbols` must have scanner-valid symbol values
//! (ranks at most 255). Malformed RLE2, truncated symbol streams and block limits have
//! the shipped stage's behavior; invalid Huffman alphabets are the scanner's concern.

use super::{Symbols, CHECKPOINT};
use crate::{ErrorKind, Kernels};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    T0,
    T1,
    T2,
}

/// The frozen zero-scan dispatch. Widening preserves `2*S >= N_max` without overflow.
#[inline]
fn bypass(s: usize, max: usize) -> bool {
    s < 16_384 || 2 * s as u128 >= max as u128
}

pub fn mtf(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    k: Kernels,
    arm: Arm,
) -> Result<usize, ErrorKind> {
    if arm == Arm::T0 || (arm == Arm::T2 && bypass(b.syms.len(), b.max_len)) {
        return super::mtf(b, ll, counts, k);
    }
    #[cfg(target_arch = "x86_64")]
    if k.avx2 {
        // SAFETY: Kernels only enables AVX2 when detected.
        return unsafe {
            combined::<0>(
                b,
                ll,
                counts,
                &mut Vec::new(),
                arm == Arm::T1,
                #[cfg(test)]
                &mut TestControl::default(),
            )
        };
    }
    super::mtf(b, ll, counts, k)
}

pub fn mtf_checkpoints(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
    k: Kernels,
    arm: Arm,
) -> Result<usize, ErrorKind> {
    if arm == Arm::T0 || (arm == Arm::T2 && bypass(b.syms.len(), b.max_len)) {
        return super::mtf_checkpoints(b, ll, counts, checkpoints, k);
    }
    #[cfg(target_arch = "x86_64")]
    if k.avx2 {
        // SAFETY: Kernels only enables AVX2 when detected.
        return unsafe {
            combined::<CHECKPOINT>(
                b,
                ll,
                counts,
                checkpoints,
                arm == Arm::T1,
                #[cfg(test)]
                &mut TestControl::default(),
            )
        };
    }
    super::mtf_checkpoints(b, ll, counts, checkpoints, k)
}

/// One frame, two separately expanded loops. CP=0 removes checkpoint handling entirely.
/// Keeping the entry non-inlined makes T1 and T2 enter the *same* combined machine code.
/// Only test builds can override/observe the guard or change the checkpoint interval.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline(never)]
unsafe fn combined<const CP: usize>(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
    start_shipped: bool,
    #[cfg(test)] control: &mut TestControl,
) -> Result<usize, ErrorKind> {
    use super::p74::avx2::{fill, front, moved};
    use super::p74::{BLEND, SHIFT, SWAPPED};
    use super::MAX_RUN_WEIGHT;
    use std::arch::x86_64::*;
    use std::sync::atomic::{compiler_fence, Ordering};

    let max = b.max_len;
    ll.clear();
    ll.reserve(max + 32); // P74's short-run store may write 31 bytes past n.
    *counts = [0; 256];
    if CP != 0 {
        checkpoints.clear();
        checkpoints.push([0; 256]);
    }
    #[repr(align(64))]
    struct List([u8; 320]);
    let mut store = List([0; 320]);
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let tail = base.add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let mut head = idx;
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    // Only these two errors can arise in the loops. Construct the full ErrorKind
    // after decoding so its heap-capable enum representation is not live there.
    let mut error = 0u8;
    let mut next = CP;
    let mut syms = b.syms.iter();
    // Keep monitor-only state out of the low-rank loop's register allocation.
    // Volatile accesses to these ordinary local slots are intentional: LLVM must not
    // hoist the origin or F into registers live across every token. This is not shared
    // state, synchronization, a census, or an additional per-token counter.
    let origin = b.syms.as_ptr();
    let mut fallbacks = 0usize;

    macro_rules! shipped_fill {
        ($at:expr, $c:expr, $len:expr) => {
            std::ptr::write_bytes($at, $c, $len)
        };
    }
    macro_rules! shipped_byte {
        ($s:expr) => {{
            let nn = ($s - 1) as usize;
            let v = *base.add(nn);
            if nn < 32 {
                let old = _mm256_load_si256(base as *const __m256i);
                let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(nn as i8 + 1), idx);
                _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            } else {
                std::ptr::copy(base, base.add(1), nn);
            }
            *base = v;
            if CP != 0 {
                compiler_fence(Ordering::SeqCst);
            }
            let c = s2u[v as usize];
            counts[c as usize] += 1;
            *out.add(n) = c;
            n += 1;
        }};
    }

    'done: {
        // The shipped checkpoint control flow, expanded once for each list mode.
        // No mode branch, counter or monitor in the shipped suffix. The compile-time
        // CP=0 arm is the plain loop (its limit is max and cannot be crossed legally).
        macro_rules! decode_loop {
            ($byte:ident, $front:expr, $fill:ident) => {
                loop {
                    let limit = if CP == 0 { max } else { max.min(next) };
                    let mut at = None;
                    // `while let` permits consulting the iterator at a fallback event
                    // without a second per-symbol counter or an outstanding by_ref borrow.
                    while let Some(&s) = syms.next() {
                        if s <= 1 {
                            if weight >= MAX_RUN_WEIGHT {
                                error = 1;
                                break 'done;
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
                            let c = s2u[$front as usize];
                            counts[c as usize] += run;
                            $fill!(out.add(n), c, run as usize);
                            n += run as usize;
                            (run, weight) = (0, 1);
                        }
                        if s == b.eob {
                            break 'done;
                        }
                        if n >= limit {
                            at = Some(s);
                            break;
                        }
                        $byte!(s);
                    }
                    let Some(s) = at else { break 'done };
                    if run != 0 {
                        if n + run as usize > max {
                            error = 2;
                            break 'done;
                        }
                        let c = s2u[$front as usize];
                        let mut left = run as usize;
                        if CP != 0 {
                            while n + left >= next {
                                let part = next - n;
                                counts[c as usize] += part as u32;
                                $fill!(out.add(n), c, part);
                                (n, left) = (next, left - part);
                                checkpoints.push(*counts);
                                next += CP;
                            }
                        }
                        counts[c as usize] += left as u32;
                        $fill!(out.add(n), c, left);
                        n += left;
                        (run, weight) = (0, 1);
                        if s == b.eob {
                            break 'done;
                        }
                        if n >= max {
                            error = 2;
                            break 'done;
                        }
                    } else {
                        if n >= max {
                            error = 2;
                            break 'done;
                        }
                        if CP != 0 {
                            checkpoints.push(*counts);
                            next += CP;
                        }
                    }
                    $byte!(s);
                }
            };
        }
        let pending = if start_shipped {
            None
        } else {
            'p74: {
                macro_rules! p74_byte {
                    ($s:expr) => {{
                        let v = moved!($s, head, tail, idx, idx, true, {
                            // Only here, at rank >=32. `next()` has consumed the trigger;
                            // pointer distance counts ALL symbols, including run digits.
                            let consumed = syms
                                .as_slice()
                                .as_ptr()
                                .offset_from(std::ptr::read_volatile(&origin))
                                as usize;
                            let previous = std::ptr::read_volatile(&fallbacks);
                            let switch = previous + 1 > (consumed >> 5) + 123;
                            #[cfg(test)]
                            let switch = control.force.map_or(switch, |at| consumed == at);
                            if switch {
                                #[cfg(test)]
                                {
                                    control.at = Some(consumed);
                                    control.fallbacks = previous;
                                    control.state = Some((n, run, weight, next));
                                }
                                break 'p74 Some($s);
                            }
                            std::ptr::write_volatile(&mut fallbacks, previous + 1);
                        });
                        let c = s2u[v as usize];
                        counts[c as usize] += 1;
                        *out.add(n) = c;
                        n += 1;
                    }};
                }
                decode_loop!(p74_byte, front!(head), fill);
            }
        };
        // Tail is already at buf[64..288]; this sole store materializes all 256 entries.
        // The trigger is consumed but unapplied; run=0, weight=1 and n<limit. T1 uses
        // the same entry with initial state and no pending token.
        _mm256_store_si256(base as *mut __m256i, head);
        #[cfg(test)]
        {
            control.list = Some(std::ptr::read(base as *const [u8; 256]));
        }
        if let Some(s) = pending {
            shipped_byte!(s);
        }
        decode_loop!(shipped_byte, *base, shipped_fill);
    }
    if CP != 0 && error == 0 && n == next {
        checkpoints.push(*counts);
    }
    // All published bytes are initialized and n <= max <= capacity, also on errors.
    ll.set_len(n);
    match error {
        0 => Ok(n),
        1 => Err(ErrorKind::RunTooLong),
        _ => Err(ErrorKind::BlockTooLong),
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
#[derive(Default)]
struct TestControl {
    // Some(usize::MAX) disables the live guard; otherwise force that consumed position.
    force: Option<usize>,
    at: Option<usize>,
    fallbacks: usize,
    state: Option<(usize, u32, u32, usize)>,
    list: Option<[u8; 256]>,
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests;
