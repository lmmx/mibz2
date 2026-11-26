//! Research only (`mtf-fold`): the MTF-into-construction fold
//! (`docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md`, frozen before this code).
//!
//! **What it removes.** On a run-route block for the open kernel, the shipped decoder runs two
//! passes back to back: `runs::front` writes the block's run records (`runs: Vec<u16>`, each run
//! cut at 127 rows, with `counts` and `per`), then the open kernel's first pass
//! (`open::Kernel::construct_capped`) reads them back, re-merges the cut runs into maximal ones
//! (`maximal`) and queues or strips them. Here the front's MTF/RLE2 loop hands each run straight
//! to a [`Sink`], the open kernel's first pass (`open::FoldSink`), which merges adjacent runs of
//! one byte exactly as `maximal` does. There are no records, no cut at 127, no re-merge and no
//! `counts`/`per`. The record count that the open kernel's admission reads
//! (`OpenCells::cap`) is counted as the records would have been: one a run of at most 127
//! rows, `total.div_ceil(127)` a longer one.
//!
//! **Exactness.** The fronts here are `runs::front_scalar` and `runs::front_avx2` with their
//! record writes replaced by the sink, and every check kept. Each of them gives up (`None`)
//! wherever its original errs or gives up, and wherever it could not finish the block. The
//! caller then takes the shipped path from the start (`runs::front` and `invert_runs` on the
//! same block), which gives the block's bytes or its error. So the fold's output is the
//! shipped output wherever it finishes, and the shipped path's otherwise.
//!
//! **Its run-time switch** ([`set_enabled`]) lets one binary run a bench's arms with and
//! without it. Its counters ([`take_counts`]) give the blocks folded, those given up before the
//! open kernel's admission was known ([`Counts::bailed`]: the shipped front then ran), and
//! those its admission refused ([`Counts::not_open`]: the shipped path then ran, as it runs on
//! every such block).

use super::{Runs, CAP, MAX_RUN_WEIGHT};
use crate::block::Symbols;
use crate::Kernels;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Where the fold sends a block's runs, in L order: each run's byte and its rows (at least
/// one). Adjacent runs may share a byte (the sink merges them, as `open::maximal` merges
/// records).
pub(crate) trait Sink {
    fn run(&mut self, c: u8, total: u32);
}

/// What a fold attempt came to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// the block inverted, its `n` rows' bytes in the output buffer
    Done(usize),
    /// given up before the open kernel's admission was known: the shipped front decides
    Bailed,
    /// the open kernel's admission refused the block (as the shipped path's refuses it)
    NotOpen,
}

/// Blocks of fewer symbols than this go the shipped way without an attempt: the fold takes
/// the block's output buffer before its rows are known (a pooled one, sized by the block's
/// bound), which a small block should not hold. (Wikidata's 198 runts have under 1,000
/// symbols; its 233 full blocks, at 0.05 to 0.22 symbols a row, at least 45,000.)
pub(crate) const MIN_SYMS: usize = 4096;

/// The open kernel's largest block (`OpenCells::cap`'s entry condition): the fold sizes its
/// buffer by the block's bound, so a larger bound goes the shipped way
pub(crate) const MAX_LEN: usize = 900_000;

static ENABLED: AtomicBool = AtomicBool::new(true);
static FOLDED: AtomicU64 = AtomicU64::new(0);
static BAILED: AtomicU64 = AtomicU64::new(0);
static NOT_OPEN: AtomicU64 = AtomicU64::new(0);

/// Turn the fold on or off for the blocks decoded from now on (on by default in a build with
/// the feature): one binary, the bench's arms with and without it
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::SeqCst);
}

/// Whether the fold is on
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// The fold's outcomes since the last call
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub folded: u64,
    pub bailed: u64,
    pub not_open: u64,
}

/// The outcomes counted since the last call, and zeroed
pub fn take_counts() -> Counts {
    Counts {
        folded: FOLDED.swap(0, Ordering::Relaxed),
        bailed: BAILED.swap(0, Ordering::Relaxed),
        not_open: NOT_OPEN.swap(0, Ordering::Relaxed),
    }
}

pub(crate) fn count(o: Outcome) {
    let c = match o {
        Outcome::Done(_) => &FOLDED,
        Outcome::Bailed => &BAILED,
        Outcome::NotOpen => &NOT_OPEN,
    };
    c.fetch_add(1, Ordering::Relaxed);
}

/// Whether to attempt the fold on `b` (the kernel and the route are the caller's to check):
/// on, a bound the open kernel takes, enough symbols, and no ideal-parts oracle replacing a
/// part the fold fuses
pub(crate) fn wants(b: &Symbols) -> bool {
    #[cfg(feature = "ideal-mtf")]
    if crate::ideal::mtf::mode() != crate::ideal::Mode::Real {
        return false;
    }
    #[cfg(feature = "ideal-construction")]
    if crate::ideal::construction::mode() != crate::ideal::Mode::Real {
        return false;
    }
    #[cfg(feature = "ideal-rle1")]
    if crate::ideal::rle1::mode() != crate::ideal::Mode::Real {
        return false;
    }
    enabled() && b.max_len <= MAX_LEN && b.syms.len() >= MIN_SYMS
}

/// The least literals with which a block of bound `max_len` is refused by the open kernel's
/// admission whatever its rows (`OpenCells::cap`, unforced). Every literal opens a record, so
/// a block has at least as many records as literals, and `cap` admits r records in n rows
/// only below its dense line, 25 r < 7 n (its lazy line, 20 r < 3 n, is inside it), with
/// n <= `max_len`. So `lits >= refused(max_len)` is refused.
pub(crate) fn refused(max_len: usize) -> usize {
    (7 * max_len).div_ceil(25)
}

/// The block's runs into `sink`: its rows and the records `runs::front` would have written.
/// Or `Bailed` where the shipped front must decide (its errors, and what its AVX2 kernel
/// leaves to its scalar one); or `NotOpen` once its literals reach `lits_refused`
/// ([`refused`]), before any run is sent (the shipped path then decides, as it decides every
/// block the admission refuses). `f` lends the AVX2 kernel its bitmaps.
pub(crate) fn front<S: Sink>(
    b: &Symbols,
    f: &mut Runs,
    k: Kernels,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    #[cfg(target_arch = "x86_64")]
    if super::front_fast(k) {
        // SAFETY: `front_fast` holds only when AVX2, BMI1, BMI2, LZCNT and POPCNT were
        // detected (`Kernels::avx2` is set only when AVX2 and BMI1 were)
        return unsafe { front_avx2(b, f, sink, lits_refused) };
    }
    let _ = (k, f);
    front_scalar(b, sink, lits_refused)
}

/// `runs::front_scalar`'s loop with each record group sent to the sink: `Bailed` wherever it
/// errs, `NotOpen` (before any run is sent) once the literals before the first end-of-block
/// symbol reach `lits_refused`
pub(crate) fn front_scalar<S: Sink>(
    b: &Symbols,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    if lits_refused < b.syms.len() {
        let lits = b
            .syms
            .iter()
            .take_while(|&&s| s != b.eob || s <= 1)
            .filter(|&&s| s > 1)
            .count();
        if lits >= lits_refused {
            return Err(Outcome::NotOpen);
        }
    }
    let mut recs = 0usize;
    let mut close = |open: (u8, u32), sink: &mut S| {
        if open.1 > 0 {
            sink.run(open.0, open.1);
            recs += open.1.div_ceil(CAP) as usize;
        }
    };
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    let mut open = (0u8, 0u32);
    for &s in &b.syms {
        if s <= 1 {
            if weight >= MAX_RUN_WEIGHT {
                return Err(Outcome::Bailed);
            }
            run += (s as u32 + 1) * weight;
            weight <<= 1;
            continue;
        }
        if run != 0 {
            if n + run as usize > b.max_len {
                return Err(Outcome::Bailed);
            }
            let c = b.seq_to_unseq[list[0] as usize];
            open = (c, open.1 + run);
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= b.max_len {
            return Err(Outcome::Bailed);
        }
        let nn = ((s - 1) as usize).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        close(open, sink);
        open = (c, 1);
        n += 1;
    }
    close(open, sink);
    Ok((n, recs))
}

/// `runs::front_avx2` with each record's write replaced by the sink (and no `counts`/`per`).
/// `Bailed` wherever `front_avx2` returns None, and only there (`NotOpen` as `front_scalar`'s,
/// from its bitmaps): the bitmaps, the list's two levels
/// and every check are its own, line for line.
///
/// What it writes: the bitmaps, as `front_avx2` writes them (`words` words each, reserved,
/// every one written before the length is set); and the second level's rows (an index is
/// clamped to 255). No records.
///
/// What it reads: as `front_avx2` reads (a symbol at a set bit of `lit`, all of which are below
/// `end`; whole words of `lit` and `one` up to the word holding `end`; and 8 bytes of `one`
/// from the byte holding the bit after a literal's, inside the two words of zeros after the
/// last symbol's).
///
/// # Safety
///
/// AVX2, BMI1, BMI2, LZCNT and POPCNT must be available.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
unsafe fn front_avx2<S: Sink>(
    b: &Symbols,
    f: &mut Runs,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    use super::{second, STEPS};
    use std::arch::x86_64::*;
    let m = b.syms.len();
    let sp = b.syms.as_ptr();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return Err(Outcome::Bailed);
    }

    // ---- the bitmaps, 32 symbols a step, to the first end-of-block symbol ----
    let words = m / 64 + 3;
    f.lit.clear();
    f.lit.reserve(words);
    f.one.clear();
    f.one.reserve(words);
    let lit32 = f.lit.as_mut_ptr() as *mut u32;
    let one32 = f.one.as_mut_ptr() as *mut u32;
    let v1 = _mm256_set1_epi16(1);
    let v0 = _mm256_setzero_si256();
    let ve = _mm256_set1_epi16(b.eob as i16);
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                as u32
        };
    }
    let mut end = usize::MAX;
    let mut ci = 0usize;
    while end == usize::MAX && ci * 32 + 32 <= m {
        // SAFETY: symbols ci·32 .. ci·32 + 32 are below m
        let x = _mm256_loadu_si256(sp.add(ci * 32) as *const __m256i);
        let y = _mm256_loadu_si256(sp.add(ci * 32 + 16) as *const __m256i);
        let run = bits!(
            _mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
            _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y)
        );
        let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
        let eob = bits!(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & !run;
        let (mut l, mut o) = (!run, !runa);
        if eob != 0 {
            let t = eob.trailing_zeros();
            end = ci * 32 + t as usize;
            let keep = ((2u64 << t) - 1) as u32;
            l &= keep >> 1;
            o &= keep;
        }
        // SAFETY: ci < 2·words (ci·32 < m, words > m / 64)
        *lit32.add(ci) = l;
        *one32.add(ci) = o;
        ci += 1;
    }
    if end == usize::MAX {
        let (mut l, mut o) = (0u32, 0u32);
        for (t, &s) in b.syms[ci * 32..].iter().enumerate() {
            o |= ((s != 0) as u32) << t;
            if s > 1 {
                if s == b.eob {
                    end = ci * 32 + t;
                    break;
                }
                l |= 1 << t;
            }
        }
        // SAFETY: ci·32 <= m, so ci < 2·words
        *lit32.add(ci) = l;
        *one32.add(ci) = o;
        ci += 1;
    }
    for i in ci..2 * words {
        // SAFETY: below the 2·words halves reserved
        *lit32.add(i) = 0;
        *one32.add(i) = 0;
    }
    // SAFETY: all `words` words of each were written above
    f.lit.set_len(words);
    f.one.set_len(words);
    if end == usize::MAX {
        return Err(Outcome::Bailed);
    }
    // (the admission's refusal by the literals alone, `refused`: `lit` holds exactly the
    // literals before `end`)
    if lits_refused < m {
        let lits: usize = f.lit.iter().map(|w| w.count_ones() as usize).sum();
        if lits >= lits_refused {
            return Err(Outcome::NotOpen);
        }
    }

    // ---- the literals ----
    let s2u = &b.seq_to_unseq;
    let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
    let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
    #[repr(align(64))]
    struct Rows([u8; 224]);
    let mut rows = Rows([0u8; 224]);
    rows.0.copy_from_slice(&s2u[32..]);
    let tail = rows.0.as_mut_ptr();
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let steps = STEPS.as_ptr();
    // `front_avx2`'s budget for the further records of runs over 127 rows, kept so that the
    // fold gives up exactly where it does
    let mut slack = b.max_len / CAP as usize + 1;
    // the rows, the records `front_avx2` would have written, and the rows of the runs over 127
    let (mut n, mut recs, mut big) = (0u64, 0usize, 0u64);
    let litp = f.lit.as_ptr();
    let onep = f.one.as_ptr();
    macro_rules! long {
        ($c:expr, $total:expr) => {{
            let (c, total): (u64, u64) = ($c, $total);
            let r = total.div_ceil(CAP as u64);
            let more = r.saturating_sub(1) as usize;
            big += total;
            if more > slack || big > b.max_len as u64 {
                return Err(Outcome::Bailed);
            }
            slack -= more;
            recs += r as usize;
            n += total;
            if total != 0 {
                // (total <= big <= max_len < 2^31)
                sink.run(c as u8, total as u32);
            }
        }};
    }
    macro_rules! emit {
        ($c:expr, $total:expr, $span:expr) => {{
            let (c, total): (u64, u64) = ($c, $total);
            if total > CAP as u64 {
                if $span > 22 {
                    return Err(Outcome::Bailed);
                }
                long!(c, total);
            } else {
                recs += 1;
                n += total;
                sink.run(c as u8, total as u32);
            }
        }};
    }
    let last = end / 64;
    let (mut wi, mut w) = (0usize, *litp);
    macro_rules! seek {
        () => {{
            loop {
                if w != 0 {
                    let t = w.trailing_zeros() as usize;
                    w &= w - 1;
                    break Some(t);
                }
                if wi == last {
                    break None;
                }
                wi += 1;
                // SAFETY: wi <= last = end / 64 < words
                w = *litp.add(wi);
            }
        }};
    }
    macro_rules! count {
        ($q:expr, $q2:expr) => {{
            let from = $q + 1;
            // SAFETY: from <= end + 1, so the 8 bytes from byte from / 8 are inside the
            // `words` words (end / 64 + 2 < words, the last two zero)
            let bits =
                ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned() >> (from & 7);
            _bzhi_u64(bits, ($q2 - $q) as u32)
        }};
    }
    let first = seek!();
    let q = first.map_or(end, |t| wi * 64 + t);
    if q > 21 {
        return Err(Outcome::Bailed);
    }
    long!(*s2u.as_ptr() as u64, _bzhi_u64(*onep, q as u32 + 1) - 1);
    if let Some(mut tz) = first {
        let mut after = *onep.add(wi) >> 1;
        let mut spw = sp.add(wi * 64);
        loop {
            // SAFETY: its bit is set in `lit`, so it is below `end` < m
            let s = *spw.add(tz) as usize;
            if s < 33 {
                // SAFETY: s - 1 < 32 rows of STEPS
                let t = steps.add(s - 1);
                let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                let cc = _mm_load_si128((*t).c.as_ptr() as *const __m128i);
                let dd = _mm_load_si128((*t).d.as_ptr() as *const __m128i);
                let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, cc), _mm_shuffle_epi8(f0, dd));
                f0 = g0;
            } else {
                // SAFETY: (s - 1).min(255) - 32 < 224; `tail` 64-aligned, 224 bytes
                let v = second(tail, (s - 1).min(255) - 32, f1, idx);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            let c = _mm_cvtsi128_si32(f0) as u8 as u64;
            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                tz = t2;
            } else {
                let q = wi * 64 + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                if q2 - q > 22 {
                    return Err(Outcome::Bailed);
                }
                emit!(c, count!(q, q2), q2 - q);
                match next {
                    Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                    None => break,
                }
            }
        }
    }
    if n > b.max_len as u64 {
        return Err(Outcome::Bailed);
    }
    Ok((n as usize, recs))
}
