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

    /// Process a batch of packed `(c << 24) | total` runs (`1 <= total < 2^24`), returning the
    /// total rows processed. `injective` holds only when `seq_to_unseq` over all accessed MTF
    /// indices is injective (guaranteeing adjacent emitted runs have distinct bytes).
    #[inline]
    fn batch(&mut self, runs: &[u32], _injective: bool) -> u64 {
        let mut sum = 0u64;
        for &packed in runs {
            let w = packed & 0x00FF_FFFF;
            self.run((packed >> 24) as u8, w);
            sum += u64::from(w);
        }
        sum
    }
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
    let syms_len = {
        #[cfg(feature = "scan-oracle")]
        {
            crate::scan_oracle::with_syms(b, |syms| syms.len())
        }
        #[cfg(not(feature = "scan-oracle"))]
        {
            b.syms.len()
        }
    };
    enabled() && b.max_len <= MAX_LEN && syms_len >= MIN_SYMS
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
    #[cfg(feature = "scan-oracle")]
    {
        crate::scan_oracle::with_syms(b, |syms| front_with_syms(b, syms, f, k, sink, lits_refused))
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        front_with_syms(b, &b.syms, f, k, sink, lits_refused)
    }
}

fn front_with_syms<S: Sink>(
    b: &Symbols,
    syms: &[u16],
    f: &mut Runs,
    k: Kernels,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    #[cfg(target_arch = "x86_64")]
    if super::front_fast(k) {
        // SAFETY: `front_fast` holds only when AVX2, BMI1, BMI2, LZCNT and POPCNT were
        // detected (`Kernels::avx2` is set only when AVX2 and BMI1 were)
        return unsafe { front_avx2(b, syms, f, sink, lits_refused) };
    }
    let _ = (k, f);
    front_scalar_with_syms(b, syms, sink, lits_refused)
}

/// `runs::front_scalar`'s loop with each record group sent to the sink: `Bailed` wherever it
/// errs, `NotOpen` (before any run is sent) once the literals before the first end-of-block
/// symbol reach `lits_refused`
#[allow(dead_code)]
pub(crate) fn front_scalar<S: Sink>(
    b: &Symbols,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    #[cfg(feature = "scan-oracle")]
    {
        crate::scan_oracle::with_syms(b, |syms| {
            front_scalar_with_syms(b, syms, sink, lits_refused)
        })
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        front_scalar_with_syms(b, &b.syms, sink, lits_refused)
    }
}

fn front_scalar_with_syms<S: Sink>(
    b: &Symbols,
    syms: &[u16],
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    if lits_refused < syms.len() {
        let lits = syms
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
    for &s in syms {
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

/// L1d-resident packed maximal-run batch size (`512 * 4 = 2 KiB`, well inside 32 KiB L1d;
/// selected by `docs/journal/2026-11-27-explore/mtf-l1-batch-rescue-probe.md` as the sweet spot
/// on both baseline x86-64 and `-C target-cpu=native`).
#[cfg(target_arch = "x86_64")]
const BATCH: usize = 512;

/// Maximum run length representable in a packed `(c << 24) | total` batch entry. Any longer run
/// (> 16M rows, possible only in synthetic `Symbols` with lifted bounds) escapes via `BatchState::huge`.
#[cfg(target_arch = "x86_64")]
const MAX_PACKED_RUN: u64 = (1 << 24) - 1;

#[cfg(target_arch = "x86_64")]
#[repr(align(64))]
struct MtfRows([u8; 224]);

#[cfg(target_arch = "x86_64")]
#[repr(align(64))]
struct BatchState {
    front: [u8; 32],
    rows: MtfRows,
    wi: usize,
    w: u64,
    tz: usize,
    last: usize,
    end: usize,
    slack: usize,
    big: u64,
    extra_recs: usize,
    done: bool,
    huge: Option<(u8, u64)>,
}

/// Whether `s2u[..used]` contains all distinct bytes. When true and every literal accesses an
/// index `1 <= nn < used`, the MTF list's first `used` entries remain a permutation of distinct
/// bytes, so every literal moves a byte strictly distinct from the previous front (`c != prev_c`).
#[cfg(target_arch = "x86_64")]
fn s2u_injective(s2u: &[u8; 256], used: usize) -> bool {
    let mut seen = [0u64; 4];
    for &b in &s2u[..used] {
        let word = (b >> 6) as usize;
        let bit = 1u64 << (b & 63);
        if seen[word] & bit != 0 {
            return false;
        }
        seen[word] |= bit;
    }
    true
}

/// Cold out-of-line second-level MTF helper (`s >= 33`, ~1.5% of Wikidata literals). Keeping this
/// out-of-line prevents LLVM from hoisting five YMM constant loads into `fill_batch_avx2`'s
/// prologue and keeps `fill_batch_avx2` pure 128-bit XMM with zero `vzeroupper` instructions.
///
/// # Safety
///
/// AVX2 must be available; `tail` is 64-aligned and holds 224 bytes; `t < 224`.
#[cfg(target_arch = "x86_64")]
#[cold]
#[inline(never)]
#[target_feature(enable = "avx2")]
unsafe fn second_cold(tail: *mut u8, t: usize, f1: std::arch::x86_64::__m128i) -> u8 {
    use std::arch::x86_64::*;
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    super::second(tail, t, f1, idx)
}

/// Pass 1: build `lit` and `one` bitmaps from `b.syms`, count literals in-flight (free base
/// record count and early `lits_refused` check), and compute `max_sym` before `eob` so
/// `s2u_injective` is exact for arbitrary `Symbols`.
///
/// # Safety
///
/// AVX2, BMI1, BMI2, LZCNT and POPCNT must be available.
#[cfg(target_arch = "x86_64")]
#[inline(never)]
#[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
unsafe fn build_bitmaps_avx2(
    b: &Symbols,
    syms: &[u16],
    f: &mut Runs,
    lits_refused: usize,
) -> Result<(usize, usize, u16), Outcome> {
    use std::arch::x86_64::*;
    let m = syms.len();
    let sp = syms.as_ptr();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return Err(Outcome::Bailed);
    }

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
    let mut vmax = v0;
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                as u32
        };
    }
    let mut end = usize::MAX;
    let mut lits = 0usize;
    let mut max_sym = 0u16;
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
            for &s in &syms[ci * 32..end] {
                max_sym = max_sym.max(s);
            }
        } else {
            vmax = _mm256_max_epu16(vmax, _mm256_max_epu16(x, y));
        }
        lits += l.count_ones() as usize;
        // SAFETY: ci < 2·words (ci·32 < m, words > m / 64)
        *lit32.add(ci) = l;
        *one32.add(ci) = o;
        ci += 1;
    }
    if end == usize::MAX {
        let (mut l, mut o) = (0u32, 0u32);
        for (t, &s) in syms[ci * 32..].iter().enumerate() {
            o |= ((s != 0) as u32) << t;
            if s > 1 {
                if s == b.eob {
                    end = ci * 32 + t;
                    break;
                }
                l |= 1 << t;
            }
            max_sym = max_sym.max(s);
        }
        lits += l.count_ones() as usize;
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
    if lits_refused < m && lits >= lits_refused {
        return Err(Outcome::NotOpen);
    }
    // Horizontal max across the 16 u16 lanes of `vmax`
    let hi = _mm256_extracti128_si256::<1>(vmax);
    let lo = _mm256_castsi256_si128(vmax);
    let m8 = _mm_max_epu16(lo, hi);
    let m4 = _mm_max_epu16(m8, _mm_srli_si128::<8>(m8));
    let m2 = _mm_max_epu16(m4, _mm_srli_si128::<4>(m4));
    let m1 = _mm_max_epu16(m2, _mm_srli_si128::<2>(m2));
    max_sym = max_sym.max(_mm_cvtsi128_si32(m1) as u16);
    Ok((end, lits, max_sym))
}

/// Pass 2: pure 128-bit XMM hot loop emitting up to `BATCH` packed `(c << 24) | total` runs into
/// the L1d-resident stack batch. `f0` and `f1` live permanently in XMM registers on the 98.5%
/// `s < 33` fast path with zero `FoldSink` state, zero `cnt[256]`, zero `CAP=127` writes, and
/// zero `vzeroupper` instructions in the entire function.
///
/// # Safety
///
/// AVX2, BMI1, BMI2, LZCNT and POPCNT must be available. `sp`, `litp`, `onep`, and `st` must
/// come from a successful `build_bitmaps_avx2` call on the same block.
#[cfg(target_arch = "x86_64")]
#[inline(never)]
#[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
unsafe fn fill_batch_avx2(
    st: &mut BatchState,
    sp: *const u16,
    litp: *const u64,
    onep: *const u64,
    max_len: u64,
    batch: &mut [u32; BATCH],
) -> Result<usize, Outcome> {
    use super::STEPS;
    use std::arch::x86_64::*;

    // SAFETY: `BatchState` is 64-byte aligned, so `front` (offset 0) is 16-byte aligned
    let mut f0 = _mm_load_si128(st.front.as_ptr() as *const __m128i);
    let mut f1 = _mm_load_si128(st.front.as_ptr().add(16) as *const __m128i);
    let tail = st.rows.0.as_mut_ptr();
    let steps = STEPS.as_ptr();

    let mut wi = st.wi;
    let mut w = st.w;
    let mut tz = st.tz;
    let last = st.last;
    let end = st.end;
    let mut after = *onep.add(wi) >> 1;
    let mut spw = sp.add(wi * 64);

    let out0 = batch.as_mut_ptr();
    let mut out = out0;
    let out_end = out0.add(BATCH);

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
    macro_rules! emit_long {
        ($c:expr, $total:expr, $span:expr, $finished:expr) => {{
            if $span > 22 {
                return Err(Outcome::Bailed);
            }
            let r = $total.div_ceil(CAP as u64);
            let more = r.saturating_sub(1) as usize;
            st.big += $total;
            if more > st.slack || st.big > max_len {
                return Err(Outcome::Bailed);
            }
            st.slack -= more;
            st.extra_recs += more;
            if $total <= MAX_PACKED_RUN {
                // SAFETY: `out < out_end` holds at the start of every iteration
                *out = (($c << 24) | $total) as u32;
                out = out.add(1);
            } else {
                st.huge = Some(($c as u8, $total));
                st.done = $finished;
                break;
            }
        }};
    }

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
            let v = second_cold(tail, (s - 1).min(255) - 32, f1);
            f1 = _mm_alignr_epi8::<15>(f1, f0);
            f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
        }
        let c = _mm_cvtsi128_si32(f0) as u8 as u64;

        if w != 0 {
            let t2 = w.trailing_zeros() as usize;
            w &= w - 1;
            let total = _bzhi_u64(after, t2 as u32) >> tz;
            let span = t2 - tz;
            tz = t2;
            if total <= CAP as u64 {
                // SAFETY: `out < out_end` holds at the start of every iteration
                *out = ((c << 24) | total) as u32;
                out = out.add(1);
            } else {
                emit_long!(c, total, span, false);
            }
            if out == out_end {
                break;
            }
        } else {
            let q = wi * 64 + tz;
            let next = seek!();
            let q2 = next.map_or(end, |t| wi * 64 + t);
            let span = q2 - q;
            if span > 22 {
                return Err(Outcome::Bailed);
            }
            let total = count!(q, q2);
            let finished = match next {
                Some(t) => {
                    tz = t;
                    after = *onep.add(wi) >> 1;
                    spw = sp.add(wi * 64);
                    false
                }
                None => true,
            };
            if total <= CAP as u64 {
                // SAFETY: `out < out_end` holds at the start of every iteration
                *out = ((c << 24) | total) as u32;
                out = out.add(1);
            } else {
                emit_long!(c, total, span, finished);
            }
            if finished {
                st.done = true;
                break;
            }
            if out == out_end {
                break;
            }
        }
    }

    _mm_store_si128(st.front.as_mut_ptr() as *mut __m128i, f0);
    _mm_store_si128(st.front.as_mut_ptr().add(16) as *mut __m128i, f1);
    st.wi = wi;
    st.w = w;
    st.tz = tz;
    Ok(out.offset_from(out0) as usize)
}

/// Decoupled L1d-batched AVX2 front: Pass 1 (`build_bitmaps_avx2`) builds the bitmaps and checks
/// `lits_refused`; Pass 2 (`fill_batch_avx2`, `#[inline(never)]`, pure 128-bit XMM) fills a 2 KiB
/// `[u32; 512]` stack batch of packed maximal runs `(c << 24) | total`, flushing into `sink.batch`
/// (which runs outside `#[target_feature]` with all 15 GPRs dedicated to `Pass::take + OutQ16`).
/// `Bailed` wherever shipped `runs::front_avx2` returns `None`, and only there.
///
/// # Safety
///
/// AVX2, BMI1, BMI2, LZCNT and POPCNT must be available.
#[cfg(target_arch = "x86_64")]
unsafe fn front_avx2<S: Sink>(
    b: &Symbols,
    syms: &[u16],
    f: &mut Runs,
    sink: &mut S,
    lits_refused: usize,
) -> Result<(usize, usize), Outcome> {
    let (end, lits, max_sym) = build_bitmaps_avx2(b, syms, f, lits_refused)?;
    let used = usize::from(max_sym).clamp(1, 256);
    let injective = s2u_injective(&b.seq_to_unseq, used);

    let last = end / 64;
    let litp = f.lit.as_ptr();
    let onep = f.one.as_ptr();
    let mut wi = 0usize;
    let mut w = f.lit[0];
    let first = loop {
        if w != 0 {
            let t = w.trailing_zeros() as usize;
            w &= w - 1;
            break Some(t);
        }
        if wi == last {
            break None;
        }
        wi += 1;
        w = f.lit[wi];
    };

    let q = first.map_or(end, |t| wi * 64 + t);
    if q > 21 {
        return Err(Outcome::Bailed);
    }
    // Portable bit mask (q <= 21 => q + 1 <= 22 < 64): avoids BMI2 outside `#[target_feature]`
    let open_len = (f.one[0] & ((1u64 << (q + 1)) - 1)) - 1;
    let mut slack = b.max_len / CAP as usize + 1;
    let open_recs = open_len.div_ceil(CAP as u64);
    let open_more = open_recs.saturating_sub(1) as usize;
    if open_more > slack || open_len > b.max_len as u64 {
        return Err(Outcome::Bailed);
    }
    slack -= open_more;

    let mut n = open_len;
    if open_len != 0 {
        sink.run(b.seq_to_unseq[0], open_len as u32);
    }

    let mut extra_recs = 0usize;
    if let Some(tz) = first {
        let mut front = [0u8; 32];
        front.copy_from_slice(&b.seq_to_unseq[..32]);
        let mut rows = MtfRows([0u8; 224]);
        rows.0.copy_from_slice(&b.seq_to_unseq[32..]);
        let mut st = BatchState {
            front,
            rows,
            wi,
            w,
            tz,
            last,
            end,
            slack,
            big: open_len,
            extra_recs: 0,
            done: false,
            huge: None,
        };
        let mut batch = [0u32; BATCH];
        let sp = syms.as_ptr();
        let max_len = b.max_len as u64;
        while !st.done {
            let blen = fill_batch_avx2(&mut st, sp, litp, onep, max_len, &mut batch)?;
            if blen != 0 {
                n += sink.batch(&batch[..blen], injective);
            }
            if let Some((hc, htot)) = st.huge.take() {
                sink.run(hc, htot as u32);
                n += htot;
            }
        }
        extra_recs = st.extra_recs;
    }

    if n > b.max_len as u64 {
        return Err(Outcome::Bailed);
    }
    Ok((n as usize, open_recs as usize + lits + extra_recs))
}
