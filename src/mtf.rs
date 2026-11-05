//! Phase 2: RUNA/RUNB runs and move-to-front undone, into the BWT's last column (`ll`)
//! and its byte counts, with libbzip2's checks: a run's weight below 2^21, and the block
//! no longer than its stream's level allows. The scalar loop, or (AVX2) the move of an
//! index below 32, nearly all of them, as one blend of the list and the list a byte on,
//! with the runs read 64 symbols at a time as bit masks: a step a byte symbol, its run
//! with it, and no branch on a symbol's kind.

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

/// The run symbols in a row that libbzip2 refuses: the 22nd's weight is 2^21
const RUN_SYMBOLS_REFUSED: u32 = 22;

/// A window of 64 symbols from `w`, as bit masks (bit j for `w[j]`): the byte symbols
/// (s >= 2), the symbols other than RUNA (s != 0), and whether the window holds the
/// end-of-block symbol or `RUN_SYMBOLS_REFUSED` run symbols in a row (the window is then
/// left to the symbol loop)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn window64(w: *const u16, eob: u16) -> (u64, u64, bool) {
    use std::arch::x86_64::*;
    let z = _mm256_setzero_si256();
    let e = _mm256_set1_epi16(eob as i16);
    let s0 = _mm256_loadu_si256(w as *const __m256i);
    let s1 = _mm256_loadu_si256(w.add(16) as *const __m256i);
    let s2 = _mm256_loadu_si256(w.add(32) as *const __m256i);
    let s3 = _mm256_loadu_si256(w.add(48) as *const __m256i);
    // run symbols (s >> 1 == 0) and RUNA (s == 0), each 16-bit lane 0 or -1
    macro_rules! run {
        ($x:expr) => {
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>($x), z)
        };
    }
    let ends = _mm256_or_si256(
        _mm256_or_si256(_mm256_cmpeq_epi16(s0, e), _mm256_cmpeq_epi16(s1, e)),
        _mm256_or_si256(_mm256_cmpeq_epi16(s2, e), _mm256_cmpeq_epi16(s3, e)),
    );
    // 32 lanes' masks in order: the packs interleave the 128-bit halves, the permute
    // puts them back
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0b11_01_10_00>(
                _mm256_packs_epi16($x, $y),
            )) as u32 as u64
        };
    }
    let runs = bits!(run!(s0), run!(s1)) | bits!(run!(s2), run!(s3)) << 32;
    let runa = bits!(_mm256_cmpeq_epi16(s0, z), _mm256_cmpeq_epi16(s1, z))
        | bits!(_mm256_cmpeq_epi16(s2, z), _mm256_cmpeq_epi16(s3, z)) << 32;
    // bit j of x set where symbols j.. j + 21 are all run symbols
    let mut x = runs;
    x &= x >> 1;
    x &= x >> 2;
    x &= x >> 4;
    x &= x >> 8;
    x &= x >> (RUN_SYMBOLS_REFUSED - 16);
    (!runs, !runa, x != 0 || _mm256_testz_si256(ends, ends) == 0)
}

/// The AVX2 kernel. Its symbol loop is `mtf_scalar`'s, the list held as bytes and moved
/// by a blend (an index below 32) or a copy. After the block's first byte symbol it runs
/// the window loop, one step a byte symbol with the run before it, until a window it
/// leaves to the symbol loop: the block's last symbols (the end-of-block symbol among
/// them), a run of 22 symbols (refused), or a step that would end within 64 bytes of
/// the block's limit. The symbol loop then finishes the block from that run's start.
///
/// The window loop: 64 symbols from a run's start as bit masks, their byte symbols taken
/// in order (tzcnt, blsr). A byte symbol's run is the bijective base-2 number of the run
/// symbols before it, sum (s + 1) 2^k, which is 2^m - 1 plus their RUNB bits. With the
/// byte symbol's own bit (bit m, set in the not-RUNA mask) that is the run plus one: the
/// bits from the last byte symbol to this one, read with one shift and a bzhi. No branch
/// on a symbol's kind, so none mispredicted from one symbol to the next: the loop's one
/// unpredictable exit is once a window.
///
/// Each step writes a segment: the byte the step before moved to the front, and the run
/// of it before this step's byte symbol, `seg` bytes as two 32-byte stores (or a fill
/// above 64), counted at once. Its byte is then moved, and written by the next step (or
/// at the loop's exit). Every segment ends at most at `max - 64`, so the stores past it
/// stay inside `ll`'s capacity, the checks that `mtf_scalar` makes on the run and the
/// byte both pass, and a step that would fail it is left to the symbol loop, which makes
/// them.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn mtf_avx2(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], with slack before it for the shifted load, held as the
    // bytes themselves (each symbol's seq_to_unseq), not their indices: a move permutes
    // both alike, so the front is the byte with no lookup
    for i in 0..256 {
        store.0[32 + i] = b.seq_to_unseq[i];
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    // symbol s's byte (s >= 2: index s - 1 < 256, a symbol below the alphabet, at most
    // 257) moved to the front
    macro_rules! move_front {
        ($s:expr) => {{
            let s = $s as usize;
            let v = *base.add(s - 1);
            if s <= 32 {
                let old = _mm256_load_si256(base as *const __m256i);
                let shifted = _mm256_loadu_si256(base.sub(1) as *const __m256i);
                let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(s as i8), idx); // i <= s - 1
                _mm256_store_si256(base as *mut __m256i, _mm256_blendv_epi8(old, shifted, m));
            } else {
                std::ptr::copy(base, base.add(1), s - 1);
            }
            *base = v;
            v
        }};
    }
    let syms = &b.syms[..];
    let (p, len, eob) = (syms.as_ptr(), syms.len(), b.eob);
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max`, which
    // `ll`'s capacity holds
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    // the next symbol; whether the window loop is still to run
    let mut i = 0usize;
    let mut windows = true;
    while i < len {
        let s = *p.add(i);
        i += 1;
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
            let c = *base;
            counts[c as usize] += run;
            if run <= 64 && n + 64 <= max {
                // a short run as two 32-byte stores of its byte: the bytes past the run are
                // still inside `ll`'s capacity (n + 64 <= max), and are written again by
                // what follows or left past the length set at the end
                let v = _mm256_set1_epi8(c as i8);
                _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
            } else {
                std::ptr::write_bytes(out.add(n), c, run as usize);
            }
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == eob {
            break;
        }
        if n >= max {
            result = Err(ErrorKind::BlockTooLong);
            break;
        }
        let v = move_front!(s);
        if !windows {
            counts[v as usize] += 1;
            *out.add(n) = v;
            n += 1;
            continue;
        }
        // the window loop, from symbol i (a run's start), with v's byte at n to write
        windows = false;
        let mut v = v;
        let mut q = i;
        let lim = max.saturating_sub(64);
        'windows: while q + 64 <= len {
            let w = p.add(q);
            let (mut bytes, notruna, leave) = window64(w, eob);
            if leave {
                break;
            }
            // (bytes != 0: 64 symbols with no byte symbol would be 22 run symbols in a row)
            let mut prev = 0u32;
            loop {
                let j = bytes.trailing_zeros();
                bytes &= bytes - 1;
                // v's byte and the run of it: the run symbols' RUNB bits, then a 1
                let seg = _bzhi_u64(notruna >> prev, j + 1 - prev) as usize;
                if n + seg > lim {
                    q += prev as usize;
                    break 'windows;
                }
                if seg <= 64 {
                    let x = _mm256_set1_epi8(v as i8);
                    _mm256_storeu_si256(out.add(n) as *mut __m256i, x);
                    _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, x);
                } else {
                    std::ptr::write_bytes(out.add(n), v, seg);
                }
                counts[v as usize] += seg as u32;
                n += seg;
                v = move_front!(*w.add(j as usize));
                prev = j + 1;
                if bytes == 0 {
                    break;
                }
            }
            q += prev as usize;
        }
        // v's byte written; the symbol loop goes on from the run after it (q)
        counts[v as usize] += 1;
        *out.add(n) = v;
        n += 1;
        i = q;
    }
    // SAFETY: the first n bytes were written above, n <= max <= capacity
    ll.set_len(n);
    result.map(|()| n)
}

/// The rows a count checkpoint spans: `mtf_checkpoints` records the byte counts at every
/// multiple of it, so a build that splits the rows among threads at those points knows
/// each range's first slot in every bucket without counting it (the cooperative
/// inversion's build, research for now: #78, #81)
pub const CHECKPOINT: usize = 16 << 10;

/// `mtf`, also recording in `checkpoints` (cleared) the counts of `ll[..i * CHECKPOINT]`
/// for each i with i * CHECKPOINT at most the block's length, the first all zero. Its
/// result, `ll` and `counts` are `mtf`'s on every input, its errors included; the
/// checkpoints are complete only where it succeeds.
pub fn mtf_checkpoints(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
    k: Kernels,
) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { mtf_checkpoints_avx2(b, ll, counts, checkpoints) };
        }
    }
    let _ = k;
    mtf_checkpoints_scalar(b, ll, counts, checkpoints)
}

/// The scalar twin: `mtf_scalar`, then the checkpoints counted from its output, a second
/// pass over `ll` that the AVX2 kernel doesn't make
pub fn mtf_checkpoints_scalar(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
) -> Result<usize, ErrorKind> {
    let r = mtf_scalar(b, ll, counts);
    checkpoints.clear();
    let mut cum = [0u32; 256];
    checkpoints.push(cum);
    if r.is_ok() {
        for chunk in ll.chunks_exact(CHECKPOINT) {
            for &c in chunk {
                cum[c as usize] += 1;
            }
            checkpoints.push(cum);
        }
    }
    r
}

/// `mtf_avx2`'s loop unchanged but for its limit, the nearer of the block's end and the
/// next checkpoint, fixed for the loop as `max` is there. Where it stops at that limit,
/// the outer loop, entered once every `CHECKPOINT` bytes, tells the block's end (`mtf`'s
/// error) from a checkpoint, copies the counts, splits a run that crosses one, and takes
/// the symbol it stopped at. Making the checkpoints in the loop itself instead put more
/// state live in it and cost about twice as much (the coop journal's §14.4, §15).
///
/// The compiler fence after the list's new front is stored emits no instruction. It keeps
/// `mtf_avx2`'s order, that store before the count's increment: the next symbol's 32-byte
/// list load overlaps it, can't be forwarded from it and waits for it to commit, so with
/// the store later that wait takes in the `s2u` load and the count's update (§15.1).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn mtf_checkpoints_avx2(
    b: &Symbols,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    checkpoints: &mut Vec<[u32; 256]>,
) -> Result<usize, ErrorKind> {
    use std::arch::x86_64::*;
    use std::sync::atomic::{compiler_fence, Ordering};
    let max = b.max_len;
    ll.clear();
    ll.reserve(max);
    *counts = [0; 256];
    checkpoints.clear();
    checkpoints.push([0; 256]);
    #[repr(align(64))]
    struct L([u8; 320]);
    let mut store = L([0u8; 320]);
    // the list at store[32..288], with slack before it for the shifted load
    for i in 0..256 {
        store.0[32 + i] = i as u8;
    }
    let base = store.0.as_mut_ptr().add(32);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let s2u = &b.seq_to_unseq;
    let (mut run, mut weight) = (0u32, 1u32);
    // `n` bytes written at `out`; every write below is checked against `max` (through the
    // limit, at most `max`), which `ll`'s capacity holds
    let out = ll.as_mut_ptr();
    let mut n = 0usize;
    let mut result = Ok(());
    // `mtf_avx2`'s byte step: symbol $s's byte moved to the front and written at n
    macro_rules! byte {
        ($s:expr) => {{
            // (s - 1 < 256: a symbol is below the alphabet, at most 257)
            let nn = ($s - 1) as usize;
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
            compiler_fence(Ordering::SeqCst);
            let c = s2u[v as usize];
            counts[c as usize] += 1;
            *out.add(n) = c;
            n += 1;
        }};
    }
    let mut next = CHECKPOINT;
    let mut syms = b.syms.iter();
    'block: loop {
        let limit = max.min(next);
        // `mtf_avx2`'s loop, `max` replaced by `limit`; `at` the symbol it stopped before
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
                let c = s2u[*base as usize];
                counts[c as usize] += run;
                std::ptr::write_bytes(out.add(n), c, run as usize);
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
        // the symbols ran out with no end-of-block symbol: `mtf_avx2`'s loop ends there too
        let Some(s) = at else { break 'block };
        if run != 0 {
            // a run past the limit: past the block's end, `mtf`'s error; else it crosses
            // one or more checkpoints, written up to each, then the copy
            if n + run as usize > max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
            let c = s2u[*base as usize];
            let mut left = run as usize;
            while n + left >= next {
                let part = next - n;
                counts[c as usize] += part as u32;
                std::ptr::write_bytes(out.add(n), c, part);
                (n, left) = (next, left - part);
                checkpoints.push(*counts);
                next += CHECKPOINT;
            }
            counts[c as usize] += left as u32;
            std::ptr::write_bytes(out.add(n), c, left);
            n += left;
            (run, weight) = (0, 1);
            // then the symbol after the run, as `mtf_avx2`'s loop takes it (n < next here)
            if s == b.eob {
                break 'block;
            }
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
        } else {
            // a byte at the limit: at the block's end, `mtf`'s error; else n == next, and
            // the bytes before it complete a checkpoint
            if n >= max {
                result = Err(ErrorKind::BlockTooLong);
                break 'block;
            }
            checkpoints.push(*counts);
            next += CHECKPOINT;
        }
        byte!(s);
    }
    // a block that ends on a checkpoint completes it
    if result.is_ok() && n == next {
        checkpoints.push(*counts);
    }
    // SAFETY: the first n bytes were written above, n <= max <= capacity
    ll.set_len(n);
    result.map(|()| n)
}
