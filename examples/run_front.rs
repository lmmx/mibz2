//! The run-native front for the phrase walk, a research prototype
//! (docs/journal/2026-11-09-phrase-structure.md, §7 item 3): from a block's MTF symbols to the
//! phrase walk's table with no last column and no pass over the block's n rows. Nothing in the
//! decoder calls it.
//!
//! ```text
//! run_front check FILE.bz2          every block: records, counts, table, origin and walk
//! run_front time FILE.bz2 [ROUNDS [LANES]]
//!                                   the three fronts in decode order, one thread, the arms'
//!                                   order changing by round: medians, ns a row (7 rounds and
//!                                   4 merge lanes unless given)
//! ```
//!
//! **Research only.** The `unsafe` here (the AVX2 record kernel, the laned merge, and the
//! phrase walk copied from `examples/phrase_walk.rs`) has had no safety review, Miri, fuzzing
//! or mutation testing (RULES B5). `check` compares its results with the library's on the
//! corpus it is given and on synthetic symbol streams, and that is all.
//!
//! **Stage 1, `runs_from_symbols`.** The block's runs of L in L order, each `byte << 8 | length`
//! and cut at 127 rows, with every byte's rows and records counted. Two passes, neither over
//! the rows:
//! - *The symbols' bitmaps* (no dependent chain): one bit a symbol for "not RUNA or RUNB" and
//!   one for "not RUNA". A count of k RUNA/RUNB symbols is the bijective base-2 number whose
//!   digits are those k bits, so with the bit of the symbol that ends it as a leading one,
//!   `1 + run` is one mask and one shift of the second bitmap.
//! - *The literals* (the block's one dependent chain, the MTF list): the production kernel's
//!   two-level list, its first 32 bytes in two registers. A literal and the count after it are
//!   one record, one iteration and no branch that depends on the data, but for a count over
//!   126 or an index past 31.
//!
//! **Stage 2, `Phrase::table_from_runs`.** `Phrase::build`'s table from the records: a counting
//! sort by byte into F order, giving each cell's first F row, then the merge of the runs' L
//! starts against those F starts. The merge is cut into lanes, each a range of the runs with
//! its own position in the F starts, so that its chain (an index, the load it addresses, a
//! compare, a count) overlaps with the other lanes'.

use std::time::Instant;

use mibz2::block::Symbols;
use mibz2::bwt::{build, walk, TVector};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, ErrorKind, Kernels};

const CAP: u32 = 127;

/// A cell's record: idx (20 bits), d (7), len (7), byte (8)
#[inline(always)]
fn rec(idx: u32, d: u32, len: u32, c: u8) -> u64 {
    idx as u64 | (d as u64) << 20 | (len as u64) << 27 | (c as u64) << 34
}

#[inline(always)]
fn len_of(r: u64) -> u32 {
    (r >> 27) as u32 & 127
}

// ---- stage 1: the records from the symbols ----

/// A run's weight (N in libbzip2) refused at this, as `mtf::mtf` refuses it
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

/// A block's run records and their counts, and the kernel's scratch, kept between blocks
pub struct Front {
    /// the runs of L in L order, cut at CAP: byte << 8 | length
    pub runs: Vec<u16>,
    /// each byte's rows, and its records
    pub counts: [u32; 256],
    pub per: [u32; 256],
    /// the symbols' bitmaps: not a run symbol; not RUNA
    lit: Vec<u64>,
    one: Vec<u64>,
    /// the AVX2 kernel, where the CPU has it
    fast: bool,
    /// ns in the bitmap pass and in the literal loop, summed over the blocks so far
    pub ns_bitmap: u128,
    pub ns_literals: u128,
    /// blocks the AVX2 kernel handed to the scalar one
    pub fell_back: usize,
}

impl Front {
    pub fn new(fast: bool) -> Self {
        #[cfg(target_arch = "x86_64")]
        let has = is_x86_feature_detected!("avx2")
            && is_x86_feature_detected!("bmi1")
            && is_x86_feature_detected!("bmi2")
            && is_x86_feature_detected!("lzcnt")
            && is_x86_feature_detected!("popcnt");
        #[cfg(not(target_arch = "x86_64"))]
        let has = false;
        Front {
            runs: Vec::new(),
            counts: [0; 256],
            per: [0; 256],
            lit: Vec::new(),
            one: Vec::new(),
            fast: fast && has,
            ns_bitmap: 0,
            ns_literals: 0,
            fell_back: 0,
        }
    }
}

/// The block's run records into `f.runs`, its counts into `f.counts` and `f.per`, with no
/// column written: `mtf::mtf`'s length, and its errors. (A symbol over 256, which no decoded
/// block holds, is taken as 256, as `mtf_avx2` takes it; `mtf_scalar` panics there.)
///
/// It relies on the fact `run_fed::runs_from_symbols` asserts: a literal's byte is never the
/// byte before it, so a literal always starts a run. That holds whenever `seq_to_unseq`'s
/// bytes in use are distinct, as a decoded block's are. With a hand-built table that repeats a
/// byte, two adjacent records could share it: still the column, but not its maximal runs.
pub fn runs_from_symbols(b: &Symbols, f: &mut Front) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    if f.fast {
        // SAFETY: `fast` is set only when AVX2, BMI1, BMI2, LZCNT and POPCNT were detected
        if let Some(n) = unsafe { runs_avx2(b, f) } {
            return Ok(n);
        }
        f.fell_back += 1;
    }
    runs_scalar(b, &mut f.runs, &mut f.counts, &mut f.per)
}

/// The scalar reference: `mtf::mtf_scalar`'s loop, checks and errors, a literal opening a
/// record and a count extending the open one (or opening the block's first)
pub fn runs_scalar(
    b: &Symbols,
    runs: &mut Vec<u16>,
    counts: &mut [u32; 256],
    per: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    fn close(open: (u8, u32), runs: &mut Vec<u16>, per: &mut [u32; 256]) {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((c as u16) << 8 | len as u16);
            per[c as usize] += 1;
            left -= len;
        }
    }
    runs.clear();
    *counts = [0; 256];
    *per = [0; 256];
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    // the open run's byte and length: empty only before the block's first row
    let mut open = (0u8, 0u32);
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
            if n + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            // the count repeats the list's front, which is the open run's byte
            let c = b.seq_to_unseq[list[0] as usize];
            open = (c, open.1 + run);
            counts[c as usize] += run;
            n += run as usize;
            (run, weight) = (0, 1);
        }
        if s == b.eob {
            break;
        }
        if n >= b.max_len {
            return Err(ErrorKind::BlockTooLong);
        }
        let nn = ((s - 1) as usize).min(255);
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        close(open, runs, per);
        open = (c, 1);
        counts[c as usize] += 1;
        n += 1;
    }
    close(open, runs, per);
    Ok(n)
}

/// `src/mtf.rs`'s first level, the list's bytes 0-31 as two 16-byte sub-lists in registers
/// (`f0`, `f1`), with its move of an index nn < 32 as four byte shuffles and two ors: row nn
/// holds `f0`'s new bytes' places in `f0` (`a`) and in `f1` (`b`), and `f1`'s in `f1` (`c`) and
/// in `f0` (`d`). (`mtf.rs` makes `f1` by a blend of itself and itself a byte on. Its table is
/// private; this one is a shuffle more and a blend less, and two operations from each
/// sub-list to its next state.)
#[cfg(target_arch = "x86_64")]
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct Step {
    a: [u8; 16],
    b: [u8; 16],
    c: [u8; 16],
    d: [u8; 16],
}

#[cfg(target_arch = "x86_64")]
const fn steps() -> [Step; 32] {
    const Z: u8 = 0x80; // a shuffle's zero
    let mut t = [Step {
        a: [0; 16],
        b: [Z; 16],
        c: [0; 16],
        d: [Z; 16],
    }; 32];
    let mut nn = 0;
    while nn < 32 {
        let mut i = 0;
        while i < 16 {
            if nn < 16 {
                // index nn to the front, the bytes before it a byte on; `f1` as it is
                t[nn].a[i] = if i == 0 {
                    nn as u8
                } else if i <= nn {
                    (i - 1) as u8
                } else {
                    i as u8
                };
                t[nn].c[i] = i as u8;
            } else {
                // `f1`'s byte nn - 16 to the front, all of `f0` a byte on, its last byte into
                // `f1`, whose bytes up to nn - 16 go a byte on
                t[nn].a[i] = if i == 0 { Z } else { (i - 1) as u8 };
                t[nn].c[i] = if i == 0 {
                    Z
                } else if i <= nn - 16 {
                    (i - 1) as u8
                } else {
                    i as u8
                };
                if i == 0 {
                    t[nn].b[i] = (nn - 16) as u8;
                    t[nn].d[i] = 15;
                }
            }
            i += 1;
        }
        nn += 1;
    }
    t
}

#[cfg(target_arch = "x86_64")]
static STEPS: [Step; 32] = steps();

/// `src/mtf.rs`'s second level, copied: the move of its byte t (the list's t + 32), returned;
/// the bytes before it a byte on, `f1`'s last byte in at 0. Seven aligned rows of 32 at `tail`.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn second(
    tail: *mut u8,
    t: usize,
    f1: std::arch::x86_64::__m128i,
    idx: std::arch::x86_64::__m256i,
) -> u8 {
    use std::arch::x86_64::*;
    let v = *tail.add(t);
    let row = |j: usize| _mm256_load_si256(tail.add(32 * j) as *const __m256i);
    // row -1 as the shift reads it: `f1` in the high lane
    let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
    // [below's last byte, r's bytes 0..31]
    let on = |r: __m256i, below: __m256i| {
        _mm256_alignr_epi8::<15>(r, _mm256_permute2x128_si256::<0x03>(r, below))
    };
    if t < 32 {
        let r0 = row(0);
        let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t as i8 + 1), idx); // i <= t
        _mm256_store_si256(
            tail as *mut __m256i,
            _mm256_blendv_epi8(r0, on(r0, below0), m),
        );
    } else {
        // (t < 224: an index is below 256)
        let tv = _mm256_set1_epi8(t as u8 as i8);
        let r: [__m256i; 7] = std::array::from_fn(row);
        for j in 0..7 {
            let p = _mm256_add_epi8(idx, _mm256_set1_epi8((32 * j) as u8 as i8));
            let m = _mm256_cmpeq_epi8(_mm256_min_epu8(p, tv), p); // 32j + i <= t
            let below = if j == 0 { below0 } else { r[j - 1] };
            let new = _mm256_blendv_epi8(r[j], on(r[j], below), m);
            _mm256_store_si256(tail.add(32 * j) as *mut __m256i, new);
        }
    }
    v
}

/// The AVX2 kernel. None where it leaves the block to `runs_scalar`: a block with no
/// end-of-block symbol, with 22 or more run symbols together, longer than `max_len`, or of
/// 2^24 symbols or more. Those are `mtf::mtf`'s errors, a count it drops (the first) or no
/// decoded block (the last), and the scalar loop decides which.
///
/// What it writes, and why each write is in bounds:
/// - the bitmaps: `words` words each, reserved, every one written before the length is set;
/// - the records: one for each literal, of which there are fewer than the symbols, and the
///   further records of a run over 127 rows, which are taken from `slack` and refused past it;
/// - the second level's rows: an index is clamped to 255, as `mtf_avx2` clamps it.
///
/// What it reads: a symbol at a set bit of `lit`, all of which are below `end`; whole words of
/// `lit` and `one` up to the word holding `end`; and 8 bytes of `one` from the byte holding
/// the bit after a literal's, inside the two words of zeros after the last symbol's.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
unsafe fn runs_avx2(b: &Symbols, f: &mut Front) -> Option<usize> {
    use std::arch::x86_64::*;
    let t0 = Instant::now();
    let m = b.syms.len();
    let sp = b.syms.as_ptr();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return None;
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
    // 32 words' compares as 32 bits, in the symbols' order
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                as u32
        };
    }
    let mut end = usize::MAX;
    let mut ci = 0usize;
    while end == usize::MAX && ci * 32 + 32 <= m {
        let x = _mm256_loadu_si256(sp.add(ci * 32) as *const __m256i);
        let y = _mm256_loadu_si256(sp.add(ci * 32 + 16) as *const __m256i);
        let run = bits!(
            _mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
            _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y)
        );
        let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
        // (an end-of-block symbol of 0 or 1 would be a run symbol, as it is to `mtf`)
        let eob = bits!(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & !run;
        let (mut l, mut o) = (!run, !runa);
        if eob != 0 {
            let t = eob.trailing_zeros();
            end = ci * 32 + t as usize;
            // its own bit stays in `one` alone, and the symbols after it are never read
            let keep = ((2u64 << t) - 1) as u32;
            l &= keep >> 1;
            o &= keep;
        }
        *lit32.add(ci) = l;
        *one32.add(ci) = o;
        ci += 1;
    }
    if end == usize::MAX {
        // the last symbols, fewer than 32
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
        *lit32.add(ci) = l;
        *one32.add(ci) = o;
        ci += 1;
    }
    for i in ci..2 * words {
        *lit32.add(i) = 0;
        *one32.add(i) = 0;
    }
    // SAFETY: all `words` words of each were written above
    f.lit.set_len(words);
    f.one.set_len(words);
    let t1 = Instant::now();
    f.ns_bitmap += (t1 - t0).as_nanos();
    if end == usize::MAX {
        return None;
    }

    // ---- the literals ----
    // the list as bytes, as `mtf_avx2` holds it: the front is the byte with no lookup
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
    // a record for each literal and one for the block's first run, and `slack` more for the
    // runs over 127 rows: a block within `max_len` needs at most max_len / 127 of those
    let mut slack = b.max_len / CAP as usize + 1;
    f.runs.clear();
    f.runs.reserve(m + slack + 2);
    let out0 = f.runs.as_mut_ptr();
    let mut out = out0;
    // each byte's rows (low half) and records (high half). The low halves cannot carry: the
    // runs of at most 127 rows are fewer than 2^24 (m), and `big`, the rows of the longer
    // ones, is refused past `max_len`, itself below 2^31
    let mut cnt = [0u64; 256];
    let mut big = 0u64;
    let litp = f.lit.as_ptr();
    let onep = f.one.as_ptr();
    // a run of `total` rows of byte c, any length: its records
    macro_rules! long {
        ($c:expr, $total:expr) => {{
            let (c, total): (u64, u64) = ($c, $total);
            let recs = total.div_ceil(CAP as u64);
            let more = recs.saturating_sub(1) as usize;
            big += total;
            if more > slack || big > b.max_len as u64 {
                return None;
            }
            slack -= more;
            cnt[c as usize] += total | recs << 32;
            let mut left = total;
            while left > 0 {
                let len = left.min(CAP as u64);
                *out = (c << 8 | len) as u16;
                out = out.add(1);
                left -= len;
            }
        }};
    }
    // a run of `total` rows of byte c after `span` - 1 run symbols: one record, nearly always
    macro_rules! emit {
        ($c:expr, $total:expr, $span:expr) => {{
            let (c, total): (u64, u64) = ($c, $total);
            if total > CAP as u64 {
                // (22 run symbols or more: `mtf`'s RunTooLong, unless the length fails first)
                if $span > 22 {
                    return None;
                }
                long!(c, total);
            } else {
                // (the record's two bytes apart: low the length, high the byte)
                *(out as *mut u8) = total as u8;
                *(out as *mut u8).add(1) = c as u8;
                out = out.add(1);
                cnt[c as usize] += total | 1 << 32;
            }
        }};
    }
    // the literals' bits are walked a word at a time: `w`, word `wi`'s bits not yet taken.
    // The next literal's bit in its word, or None when there is none before `end`
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
                w = *litp.add(wi);
            }
        }};
    }
    // 1 + the count after the symbol at q, up to the symbol at q2 that is not a run symbol:
    // the q2 - q - 1 run symbols' bits of `one`, with q2's own bit as their leading one
    macro_rules! count {
        ($q:expr, $q2:expr) => {{
            let from = $q + 1;
            let bits =
                ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned() >> (from & 7);
            _bzhi_u64(bits, ($q2 - $q) as u32)
        }};
    }
    // the count that opens the block, of the list's first byte: as a literal's at -1, less the
    // one row the literal would have been
    let first = seek!();
    let q = first.map_or(end, |t| wi * 64 + t);
    if q > 21 {
        return None;
    }
    long!(*s2u.as_ptr() as u64, _bzhi_u64(*onep, q as u32 + 1) - 1);
    if let Some(mut tz) = first {
        // word `wi` of `one`, a bit down, so that the bits after a literal's start at its own
        // place; and the word's symbols
        let mut after = *onep.add(wi) >> 1;
        let mut spw = sp.add(wi * 64);
        loop {
            // a literal (a symbol of 2 or more): its byte to the front
            let s = *spw.add(tz) as usize;
            if s < 33 {
                let t = steps.add(s - 1);
                let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                let cc = _mm_load_si128((*t).c.as_ptr() as *const __m128i);
                let dd = _mm_load_si128((*t).d.as_ptr() as *const __m128i);
                let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, cc), _mm_shuffle_epi8(f0, dd));
                f0 = g0;
            } else {
                let v = second(tail, (s - 1).min(255) - 32, f1, idx);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            let c = _mm_cvtsi128_si32(f0) as u8 as u64;
            if w != 0 {
                // the next literal is in this word, as nearly all are: the count between is
                // `after`'s bits from this literal's place to below the next one's
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                tz = t2;
            } else {
                // in a later word, or there is none and the count runs to `end`
                let q = wi * 64 + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                if q2 - q > 22 {
                    return None;
                }
                emit!(c, count!(q, q2), q2 - q);
                match next {
                    Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                    None => break,
                }
            }
        }
    }
    // (every `mtf` check on the length is this one: the rows only grow, a literal adds one)
    let n: u64 = cnt.iter().map(|&x| x & 0xffff_ffff).sum();
    if n > b.max_len as u64 {
        return None;
    }
    // SAFETY: the records up to `out` were written, inside the capacity (see above)
    f.runs.set_len(out.offset_from(out0) as usize);
    for (c, &x) in cnt.iter().enumerate() {
        f.counts[c] = x as u32;
        f.per[c] = (x >> 32) as u32;
    }
    f.ns_literals += t1.elapsed().as_nanos();
    Some(n as usize)
}

// ---- stage 2: the table from the records; and the phrase walk ----

/// `examples/phrase_walk.rs`'s `Phrase`, copied: `build` and `walk` as they are there, with
/// `table_from_runs` and its timers added
#[derive(Default)]
pub struct Phrase {
    /// the cells, in F order, and two sentinels
    cells: Vec<u64>,
    /// each cell's two remembered visits: time (20 bits) and offset (7), twice
    memo: Vec<u64>,
    /// the runs in L order, cut at CAP: byte << 8 | length
    runs: Vec<u16>,
    /// the runs in L order: F index of each
    order: Vec<u32>,
    start: Vec<u32>,
    /// each text position's state: cell << 14 | rows above << 7 | rows below
    st: Vec<u32>,
    /// counters of the last walk: real steps, copies
    pub real: usize,
    pub copies: usize,
    /// `table_from_runs`'s lanes (0: the plain merge), and whether the CPU has what they need
    pub lanes: usize,
    simd: bool,
    /// ns in `table_from_runs`'s sort, merge and memo, summed over the blocks so far
    pub ns_sort: u128,
    pub ns_merge: u128,
    pub ns_memo: u128,
}

/// The F starts after the last cell's that `table_from_runs` keeps: the block's length, then
/// a row no run starts at, for the merge's 8-wide compare to stop on
const PAD: usize = 10;

impl Phrase {
    /// With `table_from_runs`'s merge in `lanes` lanes (2, 4, 8 or 16; 0 for the plain merge)
    pub fn new(lanes: usize) -> Self {
        #[cfg(target_arch = "x86_64")]
        let simd = is_x86_feature_detected!("avx2")
            && is_x86_feature_detected!("bmi1")
            && is_x86_feature_detected!("bmi2");
        #[cfg(not(target_arch = "x86_64"))]
        let simd = false;
        Phrase {
            lanes,
            simd,
            ..Phrase::default()
        }
    }

    /// The table, from the last column: a run scan, a counting sort into F order, and a merge
    /// of the runs' L starts against the cells' F starts. Returns the cell holding `orig_ptr`
    /// and its offset there, or None when the cells pass 2^18 (a block for the T-vector).
    pub fn build(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32) -> Option<(u32, u32)> {
        let n = ll.len();
        // pass 1: the runs, cut at CAP, and each byte's count of them
        let mut per = [0u32; 256];
        self.runs.clear();
        let mut i = 0;
        while i < n {
            let c = ll[i];
            let mut j = i + 1;
            // eight equal bytes at a time
            let w = u64::from_ne_bytes([c; 8]);
            while j + 8 <= n && u64::from_ne_bytes(ll[j..j + 8].try_into().unwrap()) == w {
                j += 8;
            }
            while j < n && ll[j] == c {
                j += 1;
            }
            let mut left = (j - i) as u32;
            while left > 0 {
                let len = left.min(CAP);
                self.runs.push((c as u16) << 8 | len as u16);
                per[c as usize] += 1;
                left -= len;
            }
            i = j;
        }
        let r = self.runs.len();
        if r >= 1 << 18 {
            return None;
        }
        // each byte's first F index and first F row
        let (mut nextj, mut nexta) = ([0u32; 256], [0u32; 256]);
        let (mut j, mut a) = (0, 0);
        for c in 0..256 {
            nextj[c] = j;
            nexta[c] = a;
            j += per[c];
            a += counts[c];
        }
        // pass 2: each run's F index, and the cells' F starts
        self.order.clear();
        self.order.reserve(r);
        self.start.clear();
        self.start.resize(r + 1, 0);
        self.cells.clear();
        self.cells.resize(r + 2, 0);
        for &run in &self.runs {
            let (c, len) = ((run >> 8) as usize, run as u32 & 255);
            let j = nextj[c];
            nextj[c] += 1;
            self.start[j as usize] = nexta[c];
            nexta[c] += len;
            self.cells[j as usize] = rec(0, 0, len, c as u8);
            self.order.push(j);
        }
        self.start[r] = n as u32;
        // pass 3: the image of run i is L rows p.., which lie in the cell jj with start ≤ p
        let (mut p, mut jj) = (0u32, 0usize);
        for &j in &self.order {
            while self.start[jj + 1] <= p {
                jj += 1;
            }
            let cell = &mut self.cells[j as usize];
            let len = len_of(*cell);
            *cell |= jj as u64 | ((p - self.start[jj]) as u64) << 20;
            p += len;
        }
        // the sentinels: a cell no offset is past
        self.cells[r] = rec(0, 0, 127, 0);
        self.cells[r + 1] = rec(0, 0, 127, 0);
        self.memo.clear();
        self.memo.resize(r, u64::MAX);
        // the origin's cell
        let k = self.start[..r].partition_point(|&s| s <= orig_ptr) - 1;
        Some((k as u32, orig_ptr - self.start[k]))
    }

    /// `build`'s table and origin from the run records alone (`runs` in L order, each byte's
    /// rows in `counts` and records in `per`): no column, and no pass over the rows.
    ///
    /// The records are trusted to be a column's: `per` and `counts` their own, no length over
    /// 127. If they are not, an index here panics or the table is wrong; nothing is written
    /// out of bounds, since every index into a table is checked (the lanes' too).
    pub fn table_from_runs(
        &mut self,
        runs: &[u16],
        counts: &[u32; 256],
        per: &[u32; 256],
        orig_ptr: u32,
    ) -> Option<(u32, u32)> {
        let t0 = Instant::now();
        let r = runs.len();
        if r >= 1 << 18 {
            return None;
        }
        // each byte's first F index (low half) and first F row (high half)
        let mut next = [0u64; 256];
        let (mut j, mut a) = (0u64, 0u64);
        for c in 0..256 {
            next[c] = j | a << 32;
            j += per[c] as u64;
            a += counts[c] as u64;
        }
        let n = a as u32;
        assert!(j as usize == r, "the records' counts are their own");
        // the sort: each run's F index, and each cell's F start; the tables keep what they
        // held (every slot below r is written, the counts being the records' own)
        if self.start.len() < r + PAD {
            self.start.resize(r + PAD, 0);
        }
        self.start[r] = n;
        self.start[r + 1..r + PAD].fill(i32::MAX as u32);
        self.order.clear();
        self.order.reserve(r);
        // the lanes: equal ranges of the runs, and each one's first L row
        let lanes = if r < 64 { 0 } else { self.lanes };
        let each = r.checked_div(lanes).unwrap_or(0);
        let mut first = [0u32; MAX_LANES + 1];
        let mut p = 0u32;
        for (l, lane_first) in first.iter_mut().enumerate().take(lanes + 1) {
            *lane_first = p;
            let range = if l < lanes {
                l * each..(l + 1) * each
            } else {
                lanes * each..r
            };
            let start = &mut self.start[..r];
            self.order.extend(runs[range].iter().map(|&run| {
                let (c, len) = ((run >> 8) as usize, run as u64 & 255);
                let v = next[c];
                next[c] = v + (1 | len << 32);
                start[v as u32 as usize] = (v >> 32) as u32;
                p += len as u32;
                v as u32
            }));
        }
        let t1 = Instant::now();
        // the merge: the image of run i is L rows p.., which lie in the cell jj with start ≤ p
        if self.cells.len() != r + 2 {
            self.cells.resize(r + 2, 0);
        }
        let (mut p, mut jj, mut done) = (0u32, 0usize, 0usize);
        #[cfg(target_arch = "x86_64")]
        if lanes != 0 && self.simd {
            let cells = &mut self.cells[..r];
            let (order, start) = (&self.order[..], &self.start[..]);
            // SAFETY: `simd` is set only when AVX2, BMI1 and BMI2 were detected; the lanes'
            // own conditions are asserted there
            (p, jj) = unsafe {
                match lanes {
                    2 => merge_lanes::<2>(runs, order, start, cells, &first),
                    4 => merge_lanes::<4>(runs, order, start, cells, &first),
                    8 => merge_lanes::<8>(runs, order, start, cells, &first),
                    16 => merge_lanes::<16>(runs, order, start, cells, &first),
                    _ => panic!("2, 4, 8 or 16 lanes"),
                }
            };
            done = lanes * each;
        }
        // the plain merge, and after the lanes the runs they left
        for (&run, &j) in runs[done..].iter().zip(&self.order[done..]) {
            while self.start[jj + 1] <= p {
                jj += 1;
            }
            let (c, len) = ((run >> 8) as u8, run as u32 & 255);
            self.cells[j as usize] = rec(jj as u32, p - self.start[jj], len, c);
            p += len;
        }
        // the sentinels: a cell no offset is past
        self.cells[r] = rec(0, 0, 127, 0);
        self.cells[r + 1] = rec(0, 0, 127, 0);
        let t2 = Instant::now();
        self.memo.clear();
        self.memo.resize(r, u64::MAX);
        // the origin's cell
        let k = self.start[..r].partition_point(|&s| s <= orig_ptr) - 1;
        self.ns_sort += (t1 - t0).as_nanos();
        self.ns_merge += (t2 - t1).as_nanos();
        self.ns_memo += t2.elapsed().as_nanos();
        Some((k as u32, orig_ptr - self.start[k]))
    }

    /// The walk from (cell, offset): n bytes into `out`
    pub fn walk(&mut self, n: usize, origin: (u32, u32), out: &mut Vec<u8>) {
        #[cfg(target_arch = "x86_64")]
        assert!(
            is_x86_feature_detected!("avx2"),
            "the prototype's copy loop is AVX2"
        );
        unsafe { self.walk_impl(n, origin, out) }
    }

    #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2"))]
    unsafe fn walk_impl(&mut self, n: usize, origin: (u32, u32), out: &mut Vec<u8>) {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        // the streams are written before they are read; their capacity is kept between blocks
        out.clear();
        out.reserve(n + 64);
        self.st.clear();
        self.st.reserve(n + 64);
        let (mut real, mut copies) = (0usize, 0usize);
        let cells = self.cells.as_ptr();
        let memo = self.memo.as_mut_ptr();
        let st = self.st.as_mut_ptr();
        let o_ = out.as_mut_ptr();
        let (mut k, mut o) = origin;
        let mut t = 0usize;
        #[cfg(target_arch = "x86_64")]
        let v127 = _mm256_set1_epi32(127);
        while t < n {
            let r = *cells.add(k as usize);
            let m = *memo.add(k as usize);
            let me = (t as u64) | (o as u64) << 20;
            if m == u64::MAX {
                // the cell's first visit: a real step
                *memo.add(k as usize) = me | me << 27;
                *st.add(t) = k << 14 | o << 7 | (len_of(r) - 1 - o);
                *o_.add(t) = (r >> 34) as u8;
                t += 1;
                real += 1;
                k = r as u32 & 0xfffff;
                o += (r >> 20) as u32 & 127;
                loop {
                    let l = len_of(*cells.add(k as usize));
                    if o < l {
                        break;
                    }
                    o -= l;
                    k += 1;
                }
                continue;
            }
            // the nearer in rows of the cell's first and latest visits
            let (q1, o1) = ((m & 0xfffff) as usize, (m >> 20) as u32 & 127);
            let (q2, o2) = ((m >> 27 & 0xfffff) as usize, (m >> 47) as u32 & 127);
            let (d1, d2) = (o as i32 - o1 as i32, o as i32 - o2 as i32);
            let (q, d) = if d1.abs() <= d2.abs() {
                (q1, d1)
            } else {
                (q2, d2)
            };
            *memo.add(k as usize) = (m & 0x7ff_ffff) | me << 27;
            copies += 1;
            // copy while the source's trajectory has d rows of room on the walk's side
            let kd = (d << 7).wrapping_sub(d) as u32;
            let (shift, need) = if d > 0 {
                (0, d as u32)
            } else {
                (7, (-d) as u32)
            };
            let mut i = 0usize;
            let s;
            #[cfg(target_arch = "x86_64")]
            if t - q >= 8 {
                let vneed = _mm256_set1_epi32(need as i32);
                let vk = _mm256_set1_epi32(kd as i32);
                let sh = _mm_cvtsi32_si128(shift);
                while t + i + 8 <= n {
                    let v = _mm256_loadu_si256(st.add(q + i) as *const __m256i);
                    let room = _mm256_and_si256(_mm256_srl_epi32(v, sh), v127);
                    let bad =
                        _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(vneed, room)));
                    if bad != 0 {
                        break;
                    }
                    _mm256_storeu_si256(st.add(t + i) as *mut __m256i, _mm256_add_epi32(v, vk));
                    (o_.add(t + i) as *mut u64)
                        .write_unaligned((o_.add(q + i) as *const u64).read_unaligned());
                    i += 8;
                }
            }
            loop {
                if t + i == n {
                    s = 0;
                    break;
                }
                let v = *st.add(q + i);
                if (v >> shift & 127) < need {
                    s = v;
                    break;
                }
                *st.add(t + i) = v.wrapping_add(kd);
                *o_.add(t + i) = *o_.add(q + i);
                i += 1;
            }
            t += i;
            if t == n {
                break;
            }
            // the source is at (cell, above, below): the walk is d rows from it
            k = s >> 14;
            if d > 0 {
                o = d as u32 - (s & 127) - 1;
                k += 1;
                loop {
                    let l = len_of(*cells.add(k as usize));
                    if o < l {
                        break;
                    }
                    o -= l;
                    k += 1;
                }
            } else {
                let mut u = (-d) as u32 - (s >> 7 & 127) - 1;
                loop {
                    k -= 1;
                    let l = len_of(*cells.add(k as usize));
                    if u < l {
                        o = l - 1 - u;
                        break;
                    }
                    u -= l;
                }
            }
        }
        out.set_len(n);
        self.real = real;
        self.copies = copies;
    }
}

const MAX_LANES: usize = 16;

/// The merge in K lanes: lane l takes runs l·each.. (each = runs / K), from the L row
/// `first[l]`. Each lane's position in the F starts is found by binary search, then moved on
/// by a compare of the next 8 starts with the run's row: the starts at or below it are a prefix
/// of them, counted by the first one above. Returns the last lane's row and position after its
/// runs, where the runs the lanes left go on.
///
/// The stores into `cells` are checked. The reads are not, and are in bounds because:
/// - `runs` and `order` are read at l·each + t < K·each <= r, their length (asserted);
/// - `start` is read at a lane's position k and the 8 places after it. It holds r + PAD
///   entries, the last PAD - 1 of them `i32::MAX` (asserted), which is above any run's row (a
///   row is below 255·2^18). A position moves only past starts at or below its run's row, so
///   it stops at or before r, and k + 8 <= r + 8 < r + PAD.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2")]
unsafe fn merge_lanes<const K: usize>(
    runs: &[u16],
    order: &[u32],
    start: &[u32],
    cells: &mut [u64],
    first: &[u32; MAX_LANES + 1],
) -> (u32, usize) {
    use std::arch::x86_64::*;
    let r = runs.len();
    assert!(r < 1 << 18 && order.len() == r && start.len() >= r + PAD);
    assert!(start[r + 1..r + PAD].iter().all(|&s| s == i32::MAX as u32));
    let each = r / K;
    let mut p: [u32; K] = std::array::from_fn(|l| first[l]);
    let mut jj: [usize; K] =
        std::array::from_fn(|l| start[..r].partition_point(|&s| s <= p[l]) - 1);
    let (rp, op, sp) = (runs.as_ptr(), order.as_ptr(), start.as_ptr());
    for t in 0..each {
        for l in 0..K {
            let i = l * each + t;
            let (run, j) = (*rp.add(i) as u64, *op.add(i) as usize);
            let mut k = jj[l];
            let vp = _mm256_set1_epi32(p[l] as i32);
            loop {
                let v = _mm256_loadu_si256(sp.add(k + 1) as *const __m256i);
                let above = _mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(v, vp)));
                let step = (above as u32 | 0x100).trailing_zeros() as usize;
                k += step;
                if step < 8 {
                    break;
                }
            }
            // idx, d, and the record's length and byte closed up to 15 bits: `rec`'s fields
            let d = p[l].wrapping_sub(*sp.add(k)) as u64;
            cells[j] = k as u64 | d << 20 | _pext_u64(run, 0xff7f) << 27;
            p[l] += run as u32 & 255;
            jj[l] = k;
        }
    }
    (p[K - 1], jj[K - 1])
}

// ---- the harness ----

fn symbols(data: &[u8]) -> Vec<Symbols> {
    let mut sc = Scanner::new(SliceSource(data));
    let mut v = Vec::new();
    while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).unwrap() {
        v.push(b);
    }
    v
}

/// A column's runs cut at CAP, and each byte's count of them: the plainest scan
fn runs_of_column(ll: &[u8]) -> (Vec<u16>, [u32; 256]) {
    let (mut runs, mut per) = (Vec::new(), [0u32; 256]);
    let mut i = 0;
    while i < ll.len() {
        let c = ll[i];
        let mut j = i;
        while j < ll.len() && ll[j] == c {
            j += 1;
        }
        let mut left = j - i;
        while left > 0 {
            let len = left.min(CAP as usize);
            runs.push((c as u16) << 8 | len as u16);
            per[c as usize] += 1;
            left -= len;
        }
        i = j;
    }
    (runs, per)
}

/// The check's scratch, and what it has counted
struct Check {
    k: Kernels,
    ll: Vec<u8>,
    counts: [u32; 256],
    fast: Front,
    slow: Front,
    want: Phrase,
    got: Phrase,
    tt: TVector,
    bytes: (Vec<u8>, Vec<u8>),
    /// blocks whose table was compared, and whose walk was
    tables: usize,
    walks: usize,
    /// rows, records, records' maximal runs, symbols, literals past the first level
    rows: usize,
    records: usize,
    maximal: usize,
    syms: usize,
    deep: usize,
}

impl Check {
    fn new() -> Self {
        Check {
            k: Kernels::detect(),
            ll: Vec::new(),
            counts: [0; 256],
            fast: Front::new(true),
            slow: Front::new(false),
            want: Phrase::default(),
            got: Phrase::new(0),
            tt: TVector::new(),
            bytes: (Vec::new(), Vec::new()),
            tables: 0,
            walks: 0,
            rows: 0,
            records: 0,
            maximal: 0,
            syms: 0,
            deep: 0,
        }
    }

    /// One block's symbols through both fronts, everything compared; `what` names it in a
    /// failure. Returns `mtf`'s result.
    fn block(&mut self, b: &Symbols, what: &str) -> Result<usize, ErrorKind> {
        // the column, by the scalar kernel and by the one the decoder uses
        let want = mtf::mtf_scalar(b, &mut self.ll, &mut self.counts);
        let (mut ll2, mut counts2) = (Vec::new(), [0u32; 256]);
        assert!(
            mtf::mtf(b, &mut ll2, &mut counts2, self.k) == want,
            "{what}: mtf's kernels differ"
        );
        // stage 1, both kernels
        let got = runs_from_symbols(b, &mut self.fast);
        let slow = runs_from_symbols(b, &mut self.slow);
        assert!(got == want, "{what}: {got:?}, mtf {want:?}");
        assert!(slow == want, "{what}: scalar {slow:?}, mtf {want:?}");
        let Ok(n) = want else { return want };
        assert!(ll2 == self.ll && counts2 == self.counts);
        let (runs, per) = runs_of_column(&self.ll);
        for (name, f) in [("avx2", &self.fast), ("scalar", &self.slow)] {
            assert!(f.runs == runs, "{what}: {name} records");
            assert!(f.counts == self.counts, "{what}: {name} row counts");
            assert!(f.per == per, "{what}: {name} record counts");
        }
        self.rows += n;
        self.records += runs.len();
        self.maximal +=
            runs.windows(2).filter(|w| w[0] >> 8 != w[1] >> 8).count() + !runs.is_empty() as usize;
        self.syms += b.syms.len();
        self.deep += b.syms.iter().filter(|&&s| s > 32 && s != b.eob).count();
        if n == 0 {
            return want;
        }
        // stage 2, plain and in each lane count, against the column's table
        let orig = b.orig_ptr.min(n as u32 - 1);
        let origin = self.want.build(&self.ll, &self.counts, orig);
        for lanes in [0, 2, 4, 8, 16] {
            self.got.lanes = lanes;
            let f = &self.fast;
            let mine = self.got.table_from_runs(&f.runs, &f.counts, &f.per, orig);
            assert!(mine == origin, "{what}, {lanes} lanes: origin");
            if origin.is_some() {
                assert!(
                    self.got.cells == self.want.cells,
                    "{what}, {lanes} lanes: table"
                );
                assert!(
                    self.got.memo == self.want.memo,
                    "{what}, {lanes} lanes: memo"
                );
            }
        }
        self.tables += 1;
        // the phrase walk from that table, against the T-vector's (times are 20 bits)
        if let (Some(origin), true) = (origin, n < 1 << 20) {
            build(&self.ll, &self.counts, &mut self.tt);
            walk(&self.tt, orig, &mut self.bytes.0);
            self.got.walk(n, origin, &mut self.bytes.1);
            assert!(self.bytes.0 == self.bytes.1, "{what}: the walk's bytes");
            self.walks += 1;
        }
        want
    }
}

/// xorshift64*
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 11) % n
    }
}

/// A made-up block: `in_use` bytes, symbols that are run symbols with probability
/// `run_pct`%, literals near the front or anywhere, and an end-of-block symbol at the end, in
/// the middle as well, or nowhere
fn synthetic(rng: &mut Rng) -> Symbols {
    let in_use = match rng.below(4) {
        0 => 1 + rng.below(4) as usize,
        1 => 256,
        _ => 1 + rng.below(256) as usize,
    };
    let mut bytes: [u8; 256] = std::array::from_fn(|i| i as u8);
    for i in (1..256).rev() {
        bytes.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..in_use].copy_from_slice(&bytes[..in_use]);
    let eob = in_use as u16 + 1;
    let run_pct = [5, 30, 60, 90, 97][rng.below(5) as usize];
    let wide = rng.below(3) == 0;
    let len = match rng.below(4) {
        0 => rng.below(40) as usize,
        _ => rng.below(3000) as usize,
    };
    let mut syms = Vec::with_capacity(len + 1);
    for _ in 0..len {
        if in_use == 1 || rng.below(100) < run_pct {
            syms.push(rng.below(2) as u16);
        } else {
            // an index in 1..in_use: any, or mostly the list's first few
            let nn = if wide || rng.below(16) == 0 {
                1 + rng.below(in_use as u64 - 1)
            } else {
                1 + rng.below((in_use as u64 - 1).min(6))
            };
            syms.push(nn as u16 + 1);
        }
    }
    match rng.below(8) {
        0 => {}
        1 if len > 0 => {
            let at = rng.below(len as u64) as usize;
            syms[at] = eob;
            syms.push(eob);
        }
        _ => syms.push(eob),
    }
    Symbols {
        crc: 0,
        orig_ptr: rng.next() as u32 & 0xff_ffff,
        seq_to_unseq,
        syms,
        eob,
        max_len: 1 << 20,
    }
}

fn check(blocks: &[Symbols]) {
    let mut ck = Check::new();
    assert!(
        ck.fast.fast,
        "no AVX2 and BMI2 here: only the scalar kernel would be checked"
    );
    for (i, b) in blocks.iter().enumerate() {
        ck.block(b, &format!("block {i}"))
            .unwrap_or_else(|e| panic!("block {i}: {e:?}"));
    }
    assert!(
        ck.fast.fell_back == 0,
        "a corpus block left to the scalar kernel"
    );
    println!(
        "corpus: {} blocks, {} rows: records, row counts and record counts equal mtf's column's \
         (AVX2 kernel and scalar); table, memo and origin equal Phrase::build's on {} blocks, at \
         0, 2, 4, 8 and 16 lanes; the phrase walk from it equals bwt::walk on {} blocks",
        blocks.len(),
        ck.rows,
        ck.tables,
        ck.walks
    );
    println!(
        "corpus: {:.4} records a row ({:.4} maximal runs), {:.3} symbols a record, {:.4} of \
         records' literals past the list's first 32",
        ck.records as f64 / ck.rows as f64,
        ck.maximal as f64 / ck.rows as f64,
        ck.syms as f64 / ck.records as f64,
        ck.deep as f64 / ck.records as f64
    );

    // made-up blocks, at a length nothing reaches and at lengths around their own
    let mut ck = Check::new();
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut cases, mut ok, mut too_long, mut run_too_long) = (0, 0, 0, 0);
    for i in 0..30_000 {
        let mut b = synthetic(&mut rng);
        let free = ck.block(&b, &format!("synthetic {i}"));
        let mut lens = vec![rng.below(1 << 12) as usize];
        if let Ok(n) = free {
            lens.extend([n, n + 1]);
            lens.extend(n.checked_sub(1));
        }
        cases += 1 + lens.len();
        for res in std::iter::once(free).chain(lens.into_iter().map(|max| {
            b.max_len = max;
            ck.block(&b, &format!("synthetic {i} at length {max}"))
        })) {
            match res {
                Ok(_) => ok += 1,
                Err(ErrorKind::BlockTooLong) => too_long += 1,
                Err(ErrorKind::RunTooLong) => run_too_long += 1,
                Err(e) => panic!("synthetic {i}: {e:?}"),
            }
        }
    }
    println!(
        "synthetic: {cases} cases, results equal mtf_scalar's: {ok} blocks ({} rows; {} tables, \
         {} walks), {too_long} BlockTooLong, {run_too_long} RunTooLong; {} left to the scalar \
         kernel by the AVX2 one",
        ck.rows, ck.tables, ck.walks, ck.fast.fell_back
    );
}

/// The arms' scratch, kept between blocks and rounds
struct Scratch {
    k: Kernels,
    ll: Vec<u8>,
    counts: [u32; 256],
    tt: TVector,
    phrase: Phrase,
    front: Front,
}

/// One pass over the blocks in decode order with one arm: ns in its two stages
fn pass(blocks: &[Symbols], arm: usize, s: &mut Scratch) -> [u128; 2] {
    let mut ns = [0u128; 2];
    for b in blocks {
        let t0 = Instant::now();
        let t1;
        match arm {
            0 => {
                mtf::mtf(b, &mut s.ll, &mut s.counts, s.k).unwrap();
                t1 = Instant::now();
                build(&s.ll, &s.counts, &mut s.tt);
                std::hint::black_box(&s.tt);
            }
            1 => {
                mtf::mtf(b, &mut s.ll, &mut s.counts, s.k).unwrap();
                t1 = Instant::now();
                std::hint::black_box(s.phrase.build(&s.ll, &s.counts, b.orig_ptr));
            }
            _ => {
                runs_from_symbols(b, &mut s.front).unwrap();
                t1 = Instant::now();
                let f = &s.front;
                std::hint::black_box(
                    s.phrase
                        .table_from_runs(&f.runs, &f.counts, &f.per, b.orig_ptr),
                );
            }
        }
        ns[0] += (t1 - t0).as_nanos();
        ns[1] += t1.elapsed().as_nanos();
    }
    ns
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let h = v.len() / 2;
    if v.len() % 2 == 1 {
        v[h]
    } else {
        (v[h - 1] + v[h]) / 2.0
    }
}

/// The TSC's rate in GHz, estimated against the clock over 200 ms
fn tsc_ghz() -> f64 {
    #[cfg(target_arch = "x86_64")]
    {
        let (t0, c0) = (Instant::now(), unsafe { std::arch::x86_64::_rdtsc() });
        std::thread::sleep(std::time::Duration::from_millis(200));
        let c1 = unsafe { std::arch::x86_64::_rdtsc() };
        (c1 - c0) as f64 / t0.elapsed().as_nanos() as f64
    }
    #[cfg(not(target_arch = "x86_64"))]
    f64::NAN
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn time(blocks: &[Symbols], rounds: usize, lanes: usize) {
    const ARMS: [&str; 3] = [
        "A  mtf + bwt::build",
        "B  mtf + Phrase::build",
        "C  runs_from_symbols + table_from_runs",
    ];
    // the arms' six orders, a round each
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [2, 1, 0],
        [1, 2, 0],
        [0, 2, 1],
        [2, 0, 1],
        [1, 0, 2],
    ];
    let mut s = Scratch {
        k: Kernels::detect(),
        ll: Vec::new(),
        counts: [0; 256],
        tt: TVector::new(),
        phrase: Phrase::new(lanes),
        front: Front::new(true),
    };
    // the rows and the records, and every buffer grown, before the clock starts
    let (mut rows, mut records) = (0usize, 0usize);
    for b in blocks {
        rows += runs_from_symbols(b, &mut s.front).unwrap();
        records += s.front.runs.len();
    }
    for arm in 0..3 {
        pass(blocks, arm, &mut s);
    }
    let ghz = tsc_ghz();
    println!("load before: {}", loadavg());
    println!(
        "{} blocks, {rows} rows, {records} records ({:.4} a row); {rounds} rounds; merge lanes \
         {lanes}; ns a row",
        blocks.len(),
        records as f64 / rows as f64
    );
    let mut t: [[Vec<f64>; 3]; 3] = Default::default();
    // stage 1's and stage 2's parts in C, from the kernels' own clocks
    let mut parts: [Vec<f64>; 5] = Default::default();
    for round in 0..rounds {
        let mut line = format!("round {round:2}");
        for &arm in &ORDERS[round % 6] {
            let f = &mut s.front;
            (f.ns_bitmap, f.ns_literals) = (0, 0);
            let p = &mut s.phrase;
            (p.ns_sort, p.ns_merge, p.ns_memo) = (0, 0, 0);
            let ns = pass(blocks, arm, &mut s);
            let (a, b) = (ns[0] as f64 / rows as f64, ns[1] as f64 / rows as f64);
            t[arm][0].push(a);
            t[arm][1].push(b);
            t[arm][2].push(a + b);
            line += &format!("  {}: {a:.3} + {b:.3} = {:.3}", &ARMS[arm][..1], a + b);
            if arm == 2 {
                let (f, p) = (&s.front, &s.phrase);
                for (v, ns) in parts.iter_mut().zip([
                    f.ns_bitmap,
                    f.ns_literals,
                    p.ns_sort,
                    p.ns_merge,
                    p.ns_memo,
                ]) {
                    v.push(ns as f64 / rows as f64);
                }
            }
        }
        println!("{line}");
    }
    println!("load after: {}", loadavg());
    println!("\nmedians of {rounds} rounds, ns a row (a row is a byte of the block before RLE1 is undone):");
    println!(
        "{:42} {:>8} {:>8} {:>8}",
        "arm", "stage 1", "stage 2", "summed"
    );
    let mut med = [[0f64; 3]; 3];
    for arm in 0..3 {
        for st in 0..3 {
            med[arm][st] = median(&mut t[arm][st]);
        }
        println!(
            "{:42} {:8.3} {:8.3} {:8.3}",
            ARMS[arm], med[arm][0], med[arm][1], med[arm][2]
        );
    }
    // cycles a run: an estimate, the TSC's rate standing for the core's clock
    let per_run = |ns: f64, runs_a_row: f64| ns * ghz / runs_a_row;
    let mine = records as f64 / rows as f64;
    println!(
        "\nC in cycles a run, ESTIMATED (ns x the TSC's {ghz:.3} GHz, not counted cycles), at \
         0.0722 runs a row [and at this file's {mine:.4} records a row]:"
    );
    let names = [
        "stage 1, runs_from_symbols",
        "  the symbols' bitmaps",
        "  the literal loop",
        "stage 2, table_from_runs",
        "  the sort (F indices, F starts)",
        "  the merge (idx, d)",
        "  the memo's fill and the origin",
        "summed",
    ];
    let p: Vec<f64> = parts.iter_mut().map(|v| median(v)).collect();
    let vals = [
        med[2][0], p[0], p[1], med[2][1], p[2], p[3], p[4], med[2][2],
    ];
    for (name, v) in names.iter().zip(vals) {
        println!(
            "{name:34} {v:7.3} ns a row  {:6.1} c/run  [{:6.1}]",
            per_run(v, 0.0722),
            per_run(v, mine)
        );
    }
    println!(
        "\nC / A {:.3}, C / B {:.3} (summed medians); A's stages in cycles a run at 0.0722: \
         {:.1} + {:.1}; B's: {:.1} + {:.1}",
        med[2][2] / med[0][2],
        med[2][2] / med[1][2],
        per_run(med[0][0], 0.0722),
        per_run(med[0][1], 0.0722),
        per_run(med[1][0], 0.0722),
        per_run(med[1][1], 0.0722)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "run_front check FILE.bz2 | run_front time FILE.bz2 [ROUNDS [LANES]]";
    let (mode, file) = (args.get(1).expect(usage), args.get(2).expect(usage));
    let data = std::fs::read(file).expect("read");
    let blocks = symbols(&data);
    match mode.as_str() {
        "check" => check(&blocks),
        "time" => {
            let rounds = args.get(3).map_or(7, |r| r.parse().expect(usage));
            let lanes = args.get(4).map_or(4, |l| l.parse().expect(usage));
            assert!(rounds >= 1 && [0, 2, 4, 8, 16].contains(&lanes), "{usage}");
            time(&blocks, rounds, lanes);
        }
        _ => panic!("{usage}"),
    }
}
