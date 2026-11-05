//! Phase 2: RUNA/RUNB runs and move-to-front undone, into the BWT's last column (`ll`)
//! and its byte counts, with libbzip2's checks: a run's weight below 2^21, and the block
//! no longer than its stream's level allows. The scalar loop, or (AVX2) the list's first
//! 32 bytes held in two registers from one symbol to the next, so that the move of an
//! index below 32, nearly all of them, is four byte shuffles with no store between.

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

/// The `pshufb` controls that move list[i], for each i < 32, to the front of the list's
/// first 32 bytes, held as two registers (`lo` list[0..16], `hi` list[16..32]):
/// `lo' = lo[c0] | hi[c1]` and `hi' = hi[c2] | lo[c3]`, a control byte with its top bit
/// set giving 0. An index below 16 rotates `lo[0..=i]` by one and leaves `hi`; one in
/// 16..32 (k = i - 16) gives `lo' = [hi[k], lo[0..15]]` and
/// `hi' = [lo[15], hi[0..k], hi[k + 1..16]]`. Each index's four controls are one 64-byte
/// line.
#[cfg(target_arch = "x86_64")]
#[repr(C, align(64))]
struct Moves([[[u8; 16]; 4]; 32]);

#[cfg(target_arch = "x86_64")]
static MOVES: Moves = {
    const Z: u8 = 0x80;
    let mut m = [[[Z; 16]; 4]; 32];
    let mut i = 0;
    while i < 32 {
        let mut j = 0;
        while j < 16 {
            if i < 16 {
                m[i][0][j] = if j == 0 {
                    i as u8
                } else if j <= i {
                    (j - 1) as u8
                } else {
                    j as u8
                };
                m[i][2][j] = j as u8;
            } else {
                let k = i - 16;
                m[i][0][j] = if j == 0 { Z } else { (j - 1) as u8 };
                m[i][1][j] = if j == 0 { k as u8 } else { Z };
                m[i][2][j] = if j == 0 {
                    Z
                } else if j <= k {
                    (j - 1) as u8
                } else {
                    j as u8
                };
                m[i][3][j] = if j == 0 { 15 } else { Z };
            }
            j += 1;
        }
        i += 1;
    }
    Moves(m)
};

/// The list's places 32..256 as bytes, a 32-byte line for each chunk of the list that
/// `mtf_avx2` keeps in memory: a place moves when it is at most the index
#[cfg(target_arch = "x86_64")]
#[repr(C, align(64))]
struct Tail([[u8; 32]; 7]);

#[cfg(target_arch = "x86_64")]
static TAIL: Tail = {
    let mut t = [[0u8; 32]; 7];
    let mut i = 0;
    while i < 224 {
        t[i / 32][i % 32] = (32 + i) as u8;
        i += 1;
    }
    Tail(t)
};

/// The list's first 32 bytes live in two registers across the symbols, so the move of an
/// index below 32, 98% of them on Wikidata, is four `pshufb`s and two ORs on registers,
/// two instructions deep from one symbol's list to the next. The list in memory instead
/// put a store and a reload between them, which waited for the store to commit (the next
/// symbol's 32-byte load spanned the 32-byte store and the front's 1-byte one, so could
/// not be forwarded from either). An index of 32 or more moves the rest of the list in
/// memory, in aligned 32-byte chunks each loaded and stored whole (so that a later load
/// of one is forwarded from its store), and `lo` and `hi` a byte on.
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
    ll.reserve(max);
    *counts = [0; 256];
    // the list held as the bytes themselves (each symbol's seq_to_unseq), not their
    // indices: a move permutes both alike, so the front is the byte with no lookup. Its
    // bytes 32..256 are in memory here; 0..32 are in `lo` and `hi`, and `front` is its
    // first byte, for the runs
    #[repr(align(64))]
    struct L([u8; 256]);
    let mut list = L(b.seq_to_unseq);
    let t = list.0.as_mut_ptr();
    let mut lo = _mm_load_si128(t as *const __m128i);
    let mut hi = _mm_load_si128(t.add(16) as *const __m128i);
    let mut front = b.seq_to_unseq[0];
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
            let c = front;
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
        if s == b.eob {
            break;
        }
        if n >= max {
            result = Err(ErrorKind::BlockTooLong);
            break;
        }
        // (s - 1 < 256: a symbol is below the alphabet, at most 257)
        let nn = (s - 1) as usize;
        if nn < 32 {
            let m = MOVES.0.as_ptr().add(nn) as *const __m128i;
            let lo2 = _mm_or_si128(
                _mm_shuffle_epi8(lo, _mm_load_si128(m)),
                _mm_shuffle_epi8(hi, _mm_load_si128(m.add(1))),
            );
            hi = _mm_or_si128(
                _mm_shuffle_epi8(hi, _mm_load_si128(m.add(2))),
                _mm_shuffle_epi8(lo, _mm_load_si128(m.add(3))),
            );
            lo = lo2;
        } else {
            // list[32..=nn] a byte on, list[32] from `hi`'s last byte, and `lo` and `hi` a
            // byte on, list[nn] at the front
            let v = *t.add(nn);
            // the tail's aligned 32-byte chunks, each with the chunk a byte on made in
            // registers (its first byte the last of the chunk before, as it was) and
            // blended in up to nn: the first chunk alone for an index below 64, else all
            // seven, the same work for every index there with no branch on it
            let nv = _mm256_set1_epi8(nn as u8 as i8);
            let pos = TAIL.0.as_ptr() as *const __m256i;
            let p = t.add(32) as *mut __m256i;
            let y = _mm256_load_si256(p);
            let below = _mm256_permute2x128_si256(_mm256_castsi128_si256(hi), y, 0x20);
            let at = _mm256_load_si256(pos);
            let m = _mm256_cmpeq_epi8(_mm256_min_epu8(at, nv), at); // at <= nn
            let shifted = _mm256_alignr_epi8(y, below, 15);
            _mm256_store_si256(p, _mm256_blendv_epi8(y, shifted, m));
            if nn >= 64 {
                let mut prev = y;
                for k in 1..7 {
                    let p = t.add(32 + 32 * k) as *mut __m256i;
                    let y = _mm256_load_si256(p);
                    let below = _mm256_permute2x128_si256(prev, y, 0x21);
                    let at = _mm256_load_si256(pos.add(k));
                    let m = _mm256_cmpeq_epi8(_mm256_min_epu8(at, nv), at);
                    let shifted = _mm256_alignr_epi8(y, below, 15);
                    _mm256_store_si256(p, _mm256_blendv_epi8(y, shifted, m));
                    prev = y;
                }
            }
            hi = _mm_alignr_epi8(hi, lo, 15);
            lo = _mm_or_si128(_mm_slli_si128::<1>(lo), _mm_cvtsi32_si128(v as i32));
        }
        let c = _mm_cvtsi128_si32(lo) as u8;
        front = c;
        counts[c as usize] += 1;
        *out.add(n) = c;
        n += 1;
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

/// `mtf_avx2`'s loop as it was before the list's front moved into registers (and before
/// #172 made its list bytes: this one's holds indices), but for its limit, the nearer of
/// the block's end and the next checkpoint, fixed for the loop as `max` is there. Where it
/// stops at that limit, the outer loop, entered once every `CHECKPOINT` bytes, tells the
/// block's end (`mtf`'s error) from a checkpoint, copies the counts, splits a run that
/// crosses one, and takes the symbol it stopped at. Making the checkpoints in the loop itself instead put more
/// state live in it and cost about twice as much (the coop journal's §14.4, §15).
///
/// The compiler fence after the list's new front is stored emits no instruction. It keeps
/// that loop's order, the store before the count's increment: the next symbol's 32-byte
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
