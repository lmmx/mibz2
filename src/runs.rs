//! Phases 2 and 3 for a block of few symbols: the inverse BWT's table built from the
//! block's run records in the LF direction, with no last column
//! (docs/journal/2026-11-10-run-front-production.md; the Pathfinder's design A,
//! docs/journal/2026-11-10-backward-table.md on #313).
//!
//! The column's path (`mtf::mtf`, then `bwt::build`) writes the n-byte last column, then
//! reads it back a row at a time, scattering each row into its byte's bucket. On a block whose
//! rows are mostly in long runs (Wikidata's: about 14 rows a run), this path writes the table
//! straight from the runs:
//! 1. `front`: the MTF symbols to the runs of L in L order, each `byte << 8 | length` and cut
//!    at 127 rows, with each byte's rows and records counted, and nothing written a row. The
//!    AVX2 kernel (the Pathfinder's, `examples/run_front.rs` on #313) walks two bitmaps of the
//!    symbols, so a literal and the count after it are one step of the MTF list's chain; the
//!    scalar kernel is `mtf::mtf_scalar`'s loop with a record for a column.
//! 2. `fill`: the LF table `P[i] = L[i] | LF(i) << 8`. Indexed by L rows, a run of L is a run
//!    of consecutive entries rising by 256, so the records are written in the order they
//!    arrive, one counter a byte, 8 entries a store with AVX2: no sort, no scatter.
//! 3. The walk is the room walk's own machine code on that table, from `orig_ptr` rather than
//!    from Ψ's first row: it follows LF, so it gives the block's bytes last to first, and its
//!    segments are joined in reverse (`bwt::walk_back_room`).
//!
//! `takes_runs` decides from the symbols, before any of it, which path a block takes. The
//! bytes are the column's path's (the tests).

use crate::block::Symbols;
use crate::bwt::TVector;
use crate::{ErrorKind, Kernels};

/// A record's longest run: a longer run is cut into records of at most this many rows
const CAP: u32 = 127;

/// A run's weight (N in libbzip2) refused at this, as `mtf::mtf` refuses it
const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;

/// Symbols in a window of the admission's sample, and the most windows it reads
const WINDOW: usize = 64;
const WINDOWS: usize = 64;

/// The least share of run symbols (RUNA, RUNB) among a block's symbols for the records'
/// path: one in this many
const RUN_SHARE: usize = 32;

/// Whether the decoder takes the run records' path for `b` (else the column's): the block is
/// below 2^24 rows, its symbols number at most three-quarters of its bound (`4 * len <= 3 * max_len`),
/// and at least one in RUN_SHARE of them, in a sample, is a run symbol.
///
/// **The bound,** not the length, as the length is known only after the front; for a full
/// block they are about equal. The cut was widened from `0.50` to `0.75` after #1080
/// (`build_lf` and `fill_avx2_trusted`, docs/journal/2026-11-27-explore/runs-takes-runs-post-1080.md):
/// compressible blocks in `0.50 < syms/max_len <= 0.75` (markdown, ELF, JSON) still have
/// 23-35% run symbols and decode faster through `runs::front` + `Segmented::invert_runs` than
/// through `mtf::mtf`'s RLE2 branches, while high-entropy blocks (`random`, `records`) still
/// short-circuit in O(1) past `0.75` or fail `RUN_SHARE` below it.
///
/// **The run symbols.** A short block of high entropy (a small incompressible file, a
/// stream's last block) also has few symbols against its bound, and by the bound alone it
/// would take the records' path. Such a block has almost no run symbols: under 1% of
/// `random`'s and `records`' symbols, against 37% of text's and 25% or more of all but one
/// of Wikidata's full blocks (that one: 10%). So a block of fewer than one run symbol in
/// RUN_SHARE keeps the column. The share is read from up to WINDOWS windows of WINDOW
/// symbols, evenly spaced (a block of at most WINDOW x WINDOWS symbols is counted whole): a
/// proportion, which a sample of 4,096 gives within a point or two. (The rows themselves are
/// not sampled: a few long runs hold most of a block's rows, and a sample misses them.)
///
/// Every block this admits, `4 * syms <= 3 * max_len` admits too.
pub fn takes_runs(b: &Symbols) -> bool {
    #[cfg(feature = "scan-oracle")]
    {
        crate::scan_oracle::with_syms(b, |syms| takes_runs_slice(b.max_len, syms))
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        takes_runs_slice(b.max_len, &b.syms)
    }
}

/// Minimum run-symbol share (`1/8 = 12.5%`) required in the widened `0.50 < len/max_len <= 0.75`
/// band so short small-alphabet random blocks (such as uniform 26-letter noise, ~3.8% runs)
/// stay on the column path while compressible blocks (`mdjournal`, `elf`, `json`, `prose`,
/// `pysrc`: 17–49% run symbols) take the branchless `Runs` front.
const WIDENED_RUN_SHARE: usize = 8;

fn takes_runs_slice(max_len: usize, syms: &[u16]) -> bool {
    let len = syms.len();
    if max_len >= (1 << 24) || 4 * len > 3 * max_len {
        return false;
    }
    let share = if 2 * len <= max_len {
        RUN_SHARE
    } else {
        WIDENED_RUN_SHARE
    };
    if len <= WINDOW * WINDOWS {
        let need = len.div_ceil(share);
        let mut runs = 0usize;
        let full = len / WINDOW;
        for i in 0..full {
            let w = &syms[i * WINDOW..(i + 1) * WINDOW];
            runs += w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
            if runs >= need {
                return true;
            }
            if runs + (len - (i + 1) * WINDOW) < need {
                return false;
            }
        }
        runs += syms[full * WINDOW..]
            .iter()
            .map(|&s| usize::from(s <= 1))
            .sum::<usize>();
        share * runs >= len
    } else {
        let need = (WINDOW * WINDOWS) / share;
        let step = len / WINDOWS;
        let mut runs = 0usize;
        for i in 0..WINDOWS {
            let w = &syms[i * step..i * step + WINDOW];
            runs += w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
            if runs >= need {
                return true;
            }
            if runs + (WINDOWS - 1 - i) * WINDOW < need {
                return false;
            }
        }
        runs >= need
    }
}

/// A block's run records and the path's scratch, kept from block to block
pub struct Runs {
    /// the runs of L in L order, cut at CAP: byte << 8 | length
    runs: Vec<u16>,
    /// (research only, `ideal-mtf`) zero-copy shared run records replayed from the frozen store
    #[cfg(feature = "ideal-mtf")]
    shared_runs: Option<std::sync::Arc<[u16]>>,
    /// each byte's rows
    counts: [u32; 256],
    /// each byte's records (unused in production; kept for internal tests and `ideal-mtf`)
    #[cfg(any(test, feature = "ideal-mtf"))]
    per: [u32; 256],
    /// the symbols' bitmaps (the AVX2 kernel's): not a run symbol; not RUNA
    lit: Vec<u64>,
    one: Vec<u64>,
    /// Whether `runs` and `counts` were produced by a completed `front_scalar` or `front_avx2`
    /// call for `len < 2^24` rows and have not been modified since: guarantees every record has
    /// `1 <= len <= CAP`, each byte's record lengths sum to `counts[c]`, and the total is `len`.
    trusted: bool,
    /// The block row count validated when `trusted` is true.
    len: usize,
}

impl Default for Runs {
    fn default() -> Self {
        Runs {
            runs: Vec::new(),
            #[cfg(feature = "ideal-mtf")]
            shared_runs: None,
            counts: [0; 256],
            #[cfg(any(test, feature = "ideal-mtf"))]
            per: [0; 256],
            lit: Vec::new(),
            one: Vec::new(),
            trusted: false,
            len: 0,
        }
    }
}

impl Runs {
    #[inline]
    fn records_slice(&self) -> &[u16] {
        #[cfg(feature = "ideal-mtf")]
        if let Some(ref s) = self.shared_runs {
            return s;
        }
        &self.runs
    }

    /// Read-only records for the experimental cells kernel; format unchanged.
    #[cfg(not(feature = "moonshot"))]
    pub(crate) fn records(&self) -> &[u16] {
        self.records_slice()
    }

    /// Research only (`moonshot`): the same records, public, so that a candidate kernel built
    /// outside the crate (bench/moonshot/) reads them as an in-crate kernel does, by the same
    /// name. Without the feature the decoder compiles exactly as shipped.
    #[cfg(feature = "moonshot")]
    #[doc(hidden)]
    pub fn records(&self) -> &[u16] {
        self.records_slice()
    }

    /// Its buffers' sizes, for the footprint ledger (src/trace.rs)
    #[cfg(any(test, feature = "trace"))]
    pub fn buffers(&self) -> Vec<crate::trace::Buffer> {
        use crate::trace::Buffer;
        vec![
            Buffer::of("runs.runs", &self.runs),
            Buffer::of("runs.lit", &self.lit),
            Buffer::of("runs.one", &self.one),
        ]
    }

    /// (research only, `ideal-mtf`) The run-records representation of L produced by `front`
    #[cfg(feature = "ideal-mtf")]
    pub(crate) fn snapshot(&self) -> (&[u16], &[u32; 256], &[u32; 256]) {
        (self.records_slice(), &self.counts, &self.per)
    }

    /// (research only, `ideal-mtf`) Zero-copy handover of recorded run records via `Arc<[u16]>`
    #[cfg(feature = "ideal-mtf")]
    pub(crate) fn load_shared(
        &mut self,
        runs: std::sync::Arc<[u16]>,
        counts: &[u32; 256],
        per: &[u32; 256],
    ) {
        self.trusted = false;
        self.len = 0;
        self.shared_runs = Some(runs);
        self.counts = *counts;
        self.per = *per;
    }
}

#[cfg(test)]
impl Runs {
    /// A column's records, as `front` leaves them: its runs cut at CAP, each byte's rows
    /// and records counted (the tests', where no symbols are at hand)
    pub(crate) fn of_column(ll: &[u8]) -> Runs {
        let mut r = Runs::default();
        let mut i = 0;
        while i < ll.len() {
            let c = ll[i];
            let mut j = i;
            while j < ll.len() && ll[j] == c {
                j += 1;
            }
            r.counts[c as usize] += (j - i) as u32;
            let mut left = j - i;
            while left > 0 {
                let len = left.min(CAP as usize);
                r.runs.push((c as u16) << 8 | len as u16);
                r.per[c as usize] += 1;
                left -= len;
            }
            i = j;
        }
        r
    }

    /// The records, to break them
    pub(crate) fn records_mut(&mut self) -> &mut Vec<u16> {
        self.trusted = false;
        self.len = 0;
        &mut self.runs
    }
}

#[cfg(test)]
impl Cut {
    /// The records before the cut
    pub(crate) fn rec_for_tests(&self) -> usize {
        self.rec
    }
}

/// Whether this CPU has what the AVX2 front needs beyond `Kernels::avx2` (AVX2 and BMI1)
#[cfg(target_arch = "x86_64")]
fn front_fast(k: Kernels) -> bool {
    k.avx2
        && std::is_x86_feature_detected!("bmi2")
        && std::is_x86_feature_detected!("lzcnt")
        && std::is_x86_feature_detected!("popcnt")
}

/// The block's run records into `r`, with each byte's rows and records counted: returns
/// `mtf::mtf`'s length, or its error, with no column written. (A symbol over 256, which no
/// decoded block holds, is taken as 256, as `mtf_avx2` takes it.)
///
/// A literal's byte is never the open run's (MTF index 0 is coded only by RUNA/RUNB), so a
/// literal always starts a run and the records are the column's runs cut at 127 rows. That
/// relies on `seq_to_unseq`'s bytes in use being distinct, as a decoded block's are; with a
/// table that repeats a byte, two adjacent records could share it, which is still the column
/// (`sort` and `fill` don't need the runs maximal).
pub fn front(b: &Symbols, r: &mut Runs, k: Kernels) -> Result<usize, ErrorKind> {
    #[cfg(feature = "ideal-mtf")]
    {
        r.shared_runs = None;
    }
    #[cfg(feature = "scan-oracle")]
    {
        crate::scan_oracle::with_syms(b, |syms| front_with_syms(b, syms, r, k))
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        front_with_syms(b, &b.syms, r, k)
    }
}

fn front_with_syms(
    b: &Symbols,
    syms: &[u16],
    r: &mut Runs,
    k: Kernels,
) -> Result<usize, ErrorKind> {
    #[cfg(target_arch = "x86_64")]
    if front_fast(k) {
        // SAFETY: `front_fast` holds only when AVX2, BMI1, BMI2, LZCNT and POPCNT were
        // detected (`Kernels::avx2` is set only when AVX2 and BMI1 were)
        if let Some(n) = unsafe { front_avx2(b, syms, r) } {
            return Ok(n);
        }
    }
    let _ = k;
    front_scalar_with_syms(b, syms, r)
}

/// The scalar front: `mtf::mtf_scalar`'s loop, checks and errors, a literal opening a record
/// and a count extending the open one (or opening the block's first)
pub fn front_scalar(b: &Symbols, r: &mut Runs) -> Result<usize, ErrorKind> {
    #[cfg(feature = "ideal-mtf")]
    {
        r.shared_runs = None;
    }
    #[cfg(feature = "scan-oracle")]
    {
        crate::scan_oracle::with_syms(b, |syms| front_scalar_with_syms(b, syms, r))
    }
    #[cfg(not(feature = "scan-oracle"))]
    {
        front_scalar_with_syms(b, &b.syms, r)
    }
}

fn front_scalar_with_syms(b: &Symbols, syms: &[u16], r: &mut Runs) -> Result<usize, ErrorKind> {
    #[cfg(any(test, feature = "ideal-mtf"))]
    fn close(open: (u8, u32), runs: &mut Vec<u16>, per: &mut [u32; 256]) {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((c as u16) << 8 | len as u16);
            per[c as usize] += 1;
            left -= len;
        }
    }
    #[cfg(not(any(test, feature = "ideal-mtf")))]
    fn close(open: (u8, u32), runs: &mut Vec<u16>) {
        let (c, mut left) = open;
        while left > 0 {
            let len = left.min(CAP);
            runs.push((c as u16) << 8 | len as u16);
            left -= len;
        }
    }
    r.trusted = false;
    r.len = 0;
    let (runs, counts) = (&mut r.runs, &mut r.counts);
    runs.clear();
    *counts = [0; 256];
    #[cfg(any(test, feature = "ideal-mtf"))]
    let per = {
        r.per = [0; 256];
        &mut r.per
    };
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let (mut run, mut weight) = (0u32, 1u32);
    let mut n = 0usize;
    // the open run's byte and length: empty only before the block's first row
    let mut open = (0u8, 0u32);
    for &s in syms {
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
        #[cfg(any(test, feature = "ideal-mtf"))]
        close(open, runs, per);
        #[cfg(not(any(test, feature = "ideal-mtf")))]
        close(open, runs);
        open = (c, 1);
        counts[c as usize] += 1;
        n += 1;
    }
    #[cfg(any(test, feature = "ideal-mtf"))]
    close(open, runs, per);
    #[cfg(not(any(test, feature = "ideal-mtf")))]
    close(open, runs);
    r.trusted = n < (1 << 24);
    r.len = n;
    Ok(n)
}

/// `src/mtf.rs`'s first level, the list's bytes 0-31 as two 16-byte sub-lists in registers
/// (`f0`, `f1`), with its move of an index nn < 32 as four byte shuffles and two ors: row nn
/// holds `f0`'s new bytes' places in `f0` (`a`) and in `f1` (`b`), and `f1`'s in `f1` (`c`) and
/// in `f0` (`d`). Kept for `src/runs/fold.rs` (`mtf-fold`).
#[cfg(all(target_arch = "x86_64", feature = "mtf-fold"))]
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct Step {
    a: [u8; 16],
    b: [u8; 16],
    c: [u8; 16],
    d: [u8; 16],
}

#[cfg(all(target_arch = "x86_64", feature = "mtf-fold"))]
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

#[cfg(all(target_arch = "x86_64", feature = "mtf-fold"))]
static STEPS: [Step; 32] = steps();

/// Branchless 256-bit first-level MTF table for `nn < 32`, holding `[f0 | f1]` in a single YMM
/// register (`f01`): each move executes `vpermq $0x4e` (lane swap) + 2×`vpshufb` (`same` on
/// `f01`, `cross` on swapped `f01`) + `vpor` — 3 Port-5 uops and 2 aligned 32-byte loads instead
/// of 4 Port-5 uops and 4 loads across separate `(f0, f1)` XMM registers
/// (`docs/journal/2026-11-27-explore/runs-front-and-fill-dissection.md`).
#[cfg(target_arch = "x86_64")]
#[repr(C, align(64))]
#[derive(Clone, Copy)]
struct YmmStep {
    same: [u8; 32],
    cross: [u8; 32],
}

#[cfg(target_arch = "x86_64")]
const fn ymm_steps() -> [YmmStep; 32] {
    const Z: u8 = 0x80; // vpshufb zero lane
    let mut t = [YmmStep {
        same: [0; 32],
        cross: [Z; 32],
    }; 32];
    let mut nn = 0;
    while nn < 32 {
        let mut i = 0;
        while i < 16 {
            if nn < 16 {
                // index nn to the front of f0; f1 unchanged; no cross-lane bytes
                t[nn].same[i] = if i == 0 {
                    nn as u8
                } else if i <= nn {
                    (i - 1) as u8
                } else {
                    i as u8
                };
                t[nn].same[16 + i] = i as u8;
            } else {
                // f1[nn - 16] crosses into f0[0]; f0[15] crosses into f1[0]; rest shift in-lane
                t[nn].same[i] = if i == 0 { Z } else { (i - 1) as u8 };
                t[nn].same[16 + i] = if i == 0 {
                    Z
                } else if i <= nn - 16 {
                    (i - 1) as u8
                } else {
                    i as u8
                };
                if i == 0 {
                    t[nn].cross[0] = (nn - 16) as u8;
                    t[nn].cross[16] = 15;
                }
            }
            i += 1;
        }
        nn += 1;
    }
    t
}

#[cfg(target_arch = "x86_64")]
static YMM_STEPS: [YmmStep; 32] = ymm_steps();

/// `src/mtf.rs`'s second level: the move of its byte t (the list's t + 32), returned; the
/// bytes before it a byte on, `f1`'s last byte in at 0. Seven aligned rows of 32 at `tail`.
///
/// # Safety
///
/// AVX2; `tail` is 64-aligned and holds 224 bytes; t < 224.
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

/// The AVX2 front. None where it leaves the block to `front_scalar`: a block with no
/// end-of-block symbol, with 22 or more run symbols together, longer than `max_len`, or of
/// 2^24 symbols or more. Those are `mtf::mtf`'s errors, a count it drops (the first) or no
/// decoded block (the last), and the scalar loop decides which. On None, `r`'s records and
/// counts are left for `front_scalar` to overwrite.
///
/// What it writes, and why each write is in bounds:
/// - the bitmaps: `words = m / 64 + 3` `u64` words reserved each; Pass 1 writes `wi` words up
///   to and including the word holding `end` (`wi <= m / 64 + 1`) plus 2 trailing zero words
///   (`wi + 2 <= words`), then sets `len = wi + 2` so every word in `0..len` is initialized;
/// - the records: one for each literal, of which there are fewer than `m`, and the further
///   records of a run over 127 rows, which are taken from `slack` and refused past it;
/// - the byte counts: `[u32; 256]` on the stack cannot overflow `u32::MAX` — at most `m < 2^24`
///   short runs (`<= 127` rows each -> `< 2^31`) plus long runs guarded by `big <= max_len < 2^31`
///   sum to `< 2^32 - 1` across all 256 bytes combined;
/// - the second level's rows: `AlignedRows` is 64-byte aligned with 224 bytes initialized from
///   `seq_to_unseq[32..256]`, and an index is clamped to 255 before subtracting 32 (`< 224`).
///
/// What it reads: a symbol at a set bit of `lit`, all of which are strictly below `end < m`;
/// whole words of `lit` and `one` up to `last = end / 64 < wi`; and 8 unaligned bytes of `one`
/// from byte `from >> 3` where `from = q + 1 <= end`, accessing at most bytes
/// `(end >> 3)..(end >> 3) + 8 <= 8 * last + 15 < 8 * (last + 2) <= 8 * (wi + 1) < 8 * (wi + 2)`.
///
/// # Safety
///
/// AVX2, BMI1, BMI2, LZCNT and POPCNT must be available.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
unsafe fn front_avx2(b: &Symbols, syms: &[u16], f: &mut Runs) -> Option<usize> {
    use std::arch::x86_64::*;
    use std::mem::MaybeUninit;
    f.trusted = false;
    f.len = 0;
    let m = syms.len();
    let sp = syms.as_ptr();
    if b.max_len >= 1 << 31 || m >= 1 << 24 {
        return None;
    }

    // ---- Pass 1: 64-symbol `u64` bitmaps with `vptest` EOB fast path ----
    let words = m / 64 + 3;
    f.lit.clear();
    f.lit.reserve(words);
    f.one.clear();
    f.one.reserve(words);
    let lit_base = f.lit.as_mut_ptr();
    let one_base = f.one.as_mut_ptr();
    let v0 = _mm256_setzero_si256();
    let ve = _mm256_set1_epi16(b.eob as i16);
    // 32 words' compares as 32 bits, in the symbols' order
    macro_rules! bits {
        ($x:expr, $y:expr) => {
            _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                as u32
        };
    }
    let mut end = m;
    let mut found_eob = false;
    let mut wi = 0usize;
    let n64 = m >> 6;
    while wi < n64 {
        // SAFETY: `wi < m >> 6`, so `wi * 64 + 64 <= m`; all four 32-byte unaligned loads
        // (`0..16`, `16..32`, `32..48`, `48..64` `u16`s) are within `syms[..m]`.
        let p0 = sp.add(wi * 64);
        let x0 = _mm256_loadu_si256(p0 as *const __m256i);
        let y0 = _mm256_loadu_si256(p0.add(16) as *const __m256i);
        let x1 = _mm256_loadu_si256(p0.add(32) as *const __m256i);
        let y1 = _mm256_loadu_si256(p0.add(48) as *const __m256i);

        // Unsigned `s <= 1` iff `(s >> 1) == 0` (exact across all `u16`, including `>= 0x8000`)
        let r0 = bits!(
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(x0), v0),
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(y0), v0)
        );
        let r1 = bits!(
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(x1), v0),
            _mm256_cmpeq_epi16(_mm256_srli_epi16::<1>(y1), v0)
        );
        let a0 = bits!(_mm256_cmpeq_epi16(x0, v0), _mm256_cmpeq_epi16(y0, v0));
        let a1 = bits!(_mm256_cmpeq_epi16(x1, v0), _mm256_cmpeq_epi16(y1, v0));

        let ex0 = _mm256_cmpeq_epi16(x0, ve);
        let ey0 = _mm256_cmpeq_epi16(y0, ve);
        let ex1 = _mm256_cmpeq_epi16(x1, ve);
        let ey1 = _mm256_cmpeq_epi16(y1, ve);
        let eob_any = _mm256_or_si256(_mm256_or_si256(ex0, ey0), _mm256_or_si256(ex1, ey1));

        let run64 = (r0 as u64) | ((r1 as u64) << 32);
        let runa64 = (a0 as u64) | ((a1 as u64) << 32);
        let mut l64 = !run64;
        let mut o64 = !runa64;

        if _mm256_testz_si256(eob_any, eob_any) == 0 {
            // (an end-of-block symbol of 0 or 1 would be a run symbol, as it is to `mtf`)
            let e0 = bits!(ex0, ey0) as u64;
            let e1 = bits!(ex1, ey1) as u64;
            let eob64 = (e0 | (e1 << 32)) & !run64;
            if eob64 != 0 {
                let t = eob64.trailing_zeros();
                end = wi * 64 + t as usize;
                found_eob = true;
                // EOB's own bit stays in `one` alone; symbols after it are never read
                let keep = if t == 63 { u64::MAX } else { (2u64 << t) - 1 };
                l64 &= keep >> 1;
                o64 &= keep;
                // SAFETY: `wi < n64 <= m / 64 < words`
                *lit_base.add(wi) = l64;
                *one_base.add(wi) = o64;
                wi += 1;
                break;
            }
        }
        // SAFETY: `wi < n64 <= m / 64 < words`
        *lit_base.add(wi) = l64;
        *one_base.add(wi) = o64;
        wi += 1;
    }
    if !found_eob && (wi * 64) < m {
        // the last symbols, fewer than 64
        let (mut l64, mut o64) = (0u64, 0u64);
        let rem = m - wi * 64;
        // SAFETY: `wi * 64 + rem == m`, so `tail_p.add(t)` is within `syms[..m]` for `t < rem`
        let tail_p = sp.add(wi * 64);
        for t in 0..rem {
            let s = *tail_p.add(t);
            o64 |= ((s != 0) as u64) << t;
            if s > 1 {
                if s == b.eob {
                    end = wi * 64 + t;
                    found_eob = true;
                    break;
                }
                l64 |= 1u64 << t;
            }
        }
        // SAFETY: here `wi == n64 == m / 64 < words`
        *lit_base.add(wi) = l64;
        *one_base.add(wi) = o64;
        wi += 1;
    }
    // Two trailing zero words for unaligned 8-byte `count!` reads past `end` (no `memset` tail).
    // SAFETY: `wi <= m / 64 + 1` (exact multiple of 64 skips tail so `wi <= m / 64`), hence
    // `wi + 1 <= m / 64 + 2 < words = m / 64 + 3`, and `wi + 2 <= words` reserved capacity.
    *lit_base.add(wi) = 0;
    *one_base.add(wi) = 0;
    *lit_base.add(wi + 1) = 0;
    *one_base.add(wi + 1) = 0;
    f.lit.set_len(wi + 2);
    f.one.set_len(wi + 2);
    if !found_eob {
        return None;
    }

    // ---- Pass 2: literals via 256-bit `YMM_STEPS` and stack `[u32; 256]` counts ----
    // Hold the first 32 bytes `[f0 | f1]` in one YMM register (`f01`), with the remaining 224
    // bytes in 64-byte-aligned stack storage for `second()`.
    let s2u = &b.seq_to_unseq;
    // SAFETY: `s2u` has 256 bytes, so loading bytes `0..32` and copying `32..256` are in bounds.
    let mut f01 = _mm256_loadu_si256(s2u.as_ptr() as *const __m256i);
    #[repr(align(64))]
    struct AlignedRows {
        _data: [u8; 224],
    }
    let mut rows = MaybeUninit::<AlignedRows>::uninit();
    let tail = rows.as_mut_ptr() as *mut u8;
    std::ptr::copy_nonoverlapping(s2u.as_ptr().add(32), tail, 224);
    let idx = _mm256_setr_epi8(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    );
    let ymm_steps = YMM_STEPS.as_ptr();
    // A record for each literal and one for the block's first run, plus `slack` for runs > 127
    // rows: a block within `max_len` needs at most `max_len / 127` extra records.
    let mut slack = b.max_len / CAP as usize + 1;
    f.runs.clear();
    f.runs.reserve(m + slack + 2);
    let out0 = f.runs.as_mut_ptr();
    let mut out = out0;
    // Stack `[u32; 256]` (1 KiB, half of `[u64; 256]`): frees a GPR for hoisting `YMM_STEPS` and
    // eliminates `movabs $0x100000000; or` on every literal. Cannot overflow `u32::MAX` (see doc).
    let mut cnt = [0u32; 256];
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
            cnt[c as usize] += total as u32;
            let mut left = total;
            while left > 0 {
                let len = left.min(CAP as u64);
                // SAFETY: within the `m + slack + 2` reserved slots: this run's first record is
                // its literal's (or the block's opening run), and the `more` extra records were
                // deducted from `slack` above.
                *out = ((c << 8) | len) as u16;
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
                // SAFETY: the literal's own record, within the `m` reserved for literals. Writing
                // the two bytes separately avoids lengthening `c`'s register chain before store.
                *(out as *mut u8) = total as u8;
                *(out as *mut u8).add(1) = c as u8;
                out = out.add(1);
                cnt[c as usize] += total as u32;
            }
        }};
    }
    // Walk literal bits a word at a time: `w` holds remaining set bits of `lit[wi]`.
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
                // SAFETY: `wi <= last = end / 64 < f.lit.len()`
                w = *litp.add(wi);
            }
        }};
    }
    // 1 + the count after the symbol at q, up to the non-run symbol at q2:
    // the `q2 - q - 1` run symbols' bits of `one`, with q2's own bit as their leading one
    macro_rules! count {
        ($q:expr, $q2:expr) => {{
            let from = $q + 1;
            // SAFETY: `from <= end`, so `from >> 3 <= end >> 3 <= 8 * last + 7`, and reading 8
            // unaligned bytes stays inside `0..(8 * (last + 2)) < 8 * f.one.len()` (two zero
            // slack words written past `last`).
            let bits =
                ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned() >> (from & 7);
            _bzhi_u64(bits, ($q2 - $q) as u32)
        }};
    }
    // Opening run of the list's first byte (at virtual index -1, less the one row a literal adds)
    let first = seek!();
    let q = first.map_or(end, |t| wi * 64 + t);
    if q > 21 {
        return None;
    }
    let open_len = _bzhi_u64(*onep, q as u32 + 1) - 1;
    if open_len > 0 {
        long!(*s2u.as_ptr() as u64, open_len);
    }
    if let Some(mut tz) = first {
        // Word `wi` of `one`, shifted down 1 bit so run bits after a literal start at its own bit.
        let mut after = *onep.add(wi) >> 1;
        let mut spw = sp.add(wi * 64);
        loop {
            // SAFETY: bit `tz` was set in `lit[wi]`, so `wi * 64 + tz < end < m` and `s >= 2`.
            let s = *spw.add(tz) as usize;
            if s < 33 {
                // SAFETY: `2 <= s <= 32`, so `s - 1 < 32` is within `YMM_STEPS: [YmmStep; 32]`;
                // `YmmStep` is `#[repr(C, align(64))]`, so `same` (offset 0) and `cross`
                // (offset 32) are 32-byte aligned for `_mm256_load_si256`.
                let t = ymm_steps.add(s - 1);
                let same = _mm256_load_si256((*t).same.as_ptr() as *const __m256i);
                let cross = _mm256_load_si256((*t).cross.as_ptr() as *const __m256i);
                let swapped = _mm256_permute4x64_epi64::<0x4e>(f01);
                f01 = _mm256_or_si256(
                    _mm256_shuffle_epi8(f01, same),
                    _mm256_shuffle_epi8(swapped, cross),
                );
            } else {
                let f0_l = _mm256_castsi256_si128(f01);
                let f1_l = _mm256_extracti128_si256::<1>(f01);
                // SAFETY: `s >= 33`, so `(s - 1).min(255) - 32 < 224`; `tail` is 64-byte aligned
                // with 224 initialized bytes.
                let v = second(tail, (s - 1).min(255) - 32, f1_l, idx);
                let nf1 = _mm_alignr_epi8::<15>(f1_l, f0_l);
                let nf0 = _mm_or_si128(_mm_slli_si128::<1>(f0_l), _mm_cvtsi32_si128(v as i32));
                f01 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(nf0), nf1);
            }
            let c = _mm_cvtsi128_si32(_mm256_castsi256_si128(f01)) as u8 as u64;
            if w != 0 {
                // Next literal is in this word: run bits lie between `tz` and `t2`.
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                tz = t2;
            } else {
                // In a later word, or none left and the count runs to `end`.
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
    let n: u64 = cnt.iter().map(|&x| x as u64).sum();
    if n > b.max_len as u64 {
        return None;
    }
    // SAFETY: `out0..out` were all written within `f.runs`'s `m + slack + 2` reserved capacity.
    f.runs.set_len(out.offset_from(out0) as usize);
    f.counts = cnt;
    #[cfg(any(test, feature = "ideal-mtf"))]
    {
        f.per = [0; 256];
        for &rec in &f.runs {
            f.per[(rec >> 8) as usize] += 1;
        }
    }
    let n = n as usize;
    f.trusted = n < (1 << 24);
    f.len = n;
    Some(n)
}

/// The block's LF table from its records (as `front` left them in `r`, for a block of `n`
/// rows) into `tt`: `P[i] = L[i] | LF(i) << 8`, with LF(i) = C(L[i]) + the copies of L[i]
/// before row i. Returns false, `tt` left empty, unless the records tile the block.
///
/// **Why it needs no sort.** A run of L at rows p..p + len, of byte c with k copies of c
/// before p, has LF(p + j) = C(c) + k + j: its entries are `c | (C(c) + k) << 8` rising by
/// 256, at the next len slots. So the table is written in the order the records arrive, with
/// one counter a byte (`next[c]`, from C(c)).
///
/// **The checks (the tiling, in place of `bwt::build`'s end check):** the counts sum to n,
/// so the buckets [C(c), C(c + 1)) tile [0, n); every record is at least a row, and is
/// refused if it would pass the block's end or its byte's bucket's end. So every value is
/// below n, and once the records' lengths reach exactly n, every slot of [0, n) has been
/// written, in order, once. That makes `tt` a T-vector in the walks' sense (every link below
/// its length), walked backward: from `orig_ptr`, the block's bytes last to first
/// (`bwt::walk_back_room`). When `r` was produced by `front` for `n` rows and has not been
/// modified since (`r.trusted && n == r.len`), `front` has already established those exact
/// tiling invariants, so the AVX2 kernel runs without per-record bounds checks.
pub fn fill(r: &Runs, n: usize, tt: &mut TVector, k: Kernels) -> bool {
    tt.0.clear();
    if n >= 1 << 24 {
        return false;
    }
    let (mut next, mut lim) = ([0u32; 256], [0u32; 256]);
    let mut cum = 0u64;
    for c in 0..256 {
        next[c] = cum as u32;
        cum += r.counts[c] as u64;
        lim[c] = cum as u32;
    }
    if cum != n as u64 {
        return false;
    }
    let runs = r.records_slice();
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            if r.trusted && n == r.len {
                // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant);
                // `r.trusted && n == r.len` is set only on a completed `front_scalar` or
                // `front_avx2` call for `n < 2^24` rows (which clears `shared_runs`, and is
                // cleared on any mutation of `r` including `load_shared` and `records_mut`),
                // establishing that `runs` tiles `[0, n)` with `1 <= len <= CAP`, each byte's
                // records sum to `r.counts[c]`, and `next` holds the exclusive prefix sums
                // summing to `n`.
                unsafe { fill_avx2_trusted(runs, n as u32, next, &mut tt.0) };
                return true;
            }
            // SAFETY: `k.avx2` is set only when AVX2 was detected (a `Kernels` invariant)
            return unsafe { fill_avx2(runs, n as u32, next, &lim, &mut tt.0) };
        }
    }
    let _ = k;
    fill_scalar(runs, n as u32, next, &lim, &mut tt.0)
}

/// The fill in safe code: each entry pushed
fn fill_scalar(
    runs: &[u16],
    n: u32,
    mut next: [u32; 256],
    lim: &[u32; 256],
    tt: &mut Vec<u32>,
) -> bool {
    tt.reserve(n as usize);
    let mut q = 0u32;
    for &x in runs {
        let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
        if len == 0 || len > n - q || len > lim[c] - next[c] {
            tt.clear();
            return false;
        }
        let e = c as u32 | next[c] << 8;
        tt.extend((0..len).map(|j| e + (j << 8)));
        next[c] += len;
        q += len;
    }
    if q != n {
        tt.clear();
        return false;
    }
    true
}

/// The fill with each record's entries written 8 at a time: they rise by 256, so a vector of
/// them is the first plus 0, 256, .., 1792, and the next 8 add 2048. A record's last store
/// runs up to 7 slots past its end, into the next records' rows, which they write again (each
/// starts where the last ended); past n, into 8 slots of slack, cut off by the length.
///
/// # Safety
///
/// AVX2 must be available. `next` and `lim` are the buckets of counts that sum to n (as
/// `fill` makes them), so that `next[c] <= lim[c] <= n`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_avx2(
    runs: &[u16],
    n: u32,
    mut next: [u32; 256],
    lim: &[u32; 256],
    tt: &mut Vec<u32>,
) -> bool {
    use std::arch::x86_64::*;
    tt.reserve(n as usize + 8);
    let out = tt.as_mut_ptr();
    let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    let step = _mm256_set1_epi32(2048);
    let mut q = 0u32;
    for &x in runs {
        let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
        // (q <= n and next[c] <= lim[c]: neither subtraction wraps)
        if len == 0 || len > n - q || len > lim[c] - next[c] {
            return false;
        }
        let e = c as u32 | next[c] << 8;
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
        let (mut w, end) = (q, q + len);
        loop {
            // SAFETY: w < end <= n, so w + 8 <= n + 7, within the n + 8 reserved
            _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
            w += 8;
            if w >= end {
                break;
            }
            v = _mm256_add_epi32(v, step);
        }
        next[c] += len;
        q = end;
    }
    if q != n {
        return false;
    }
    // SAFETY: the records' rows tile [0, n) in order, each record's rows written by its own
    // stores (the slack a record wrote past its end written again by the records after it)
    tt.set_len(n as usize);
    true
}

/// `fill_avx2` for a `Runs` produced by `front` and not modified since: `front` has already
/// established that every record has `1 <= len <= CAP`, each byte's record lengths sum to
/// `r.counts[c]`, and the total length is `n < 2^24`. That eliminates the per-record bounds
/// checks against `n - q` and `lim[c] - next[c]` and the `lim` array on the stack.
///
/// # Safety
///
/// AVX2 must be available. `runs` must tile `[0, n)` with `n < (1 << 24)`: every record has
/// `1 <= (x & 0xff) <= 127`, the records for each byte `c` sum to `r.counts[c]`, and `next`
/// holds the exclusive prefix sums of `r.counts` (whose total sum is `n`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_avx2_trusted(runs: &[u16], n: u32, mut next: [u32; 256], tt: &mut Vec<u32>) {
    use std::arch::x86_64::*;
    tt.reserve(n as usize + 8);
    let out = tt.as_mut_ptr();
    let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    let step = _mm256_set1_epi32(2048);
    let mut q = 0usize;
    for &x in runs {
        let c = (x >> 8) as usize;
        let len = (x & 0xff) as usize;
        let cur = next[c];
        next[c] = cur + len as u32;
        let e = (c as u32) | (cur << 8);
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
        let end = q + len;
        let mut w = q;
        loop {
            // SAFETY: by the tiling precondition `1 <= len` and `end <= n`, so `w < end <= n`
            // and `w + 8 <= n + 7`, within the `n + 8` elements reserved in `tt`.
            unsafe { _mm256_storeu_si256(out.add(w) as *mut __m256i, v) };
            w += 8;
            if w >= end {
                break;
            }
            v = _mm256_add_epi32(v, step);
        }
        q = end;
    }
    debug_assert_eq!(q, n as usize);
    // SAFETY: the records tile `[0, n)` in order (each with `len >= 1`), so every slot in
    // `[0, n)` was written with its final LF entry (slack past a record's end is overwritten
    // by the following records, and slack past `n` is cut off by `set_len(n)`).
    unsafe { tt.set_len(n as usize) };
}

// ---- The fill on a team's threads (`coop::RunsBuild`) ----
//
// The records are cut into pieces at `cuts`, each cut holding where the fill stands there (the
// row, and every byte's next LF value), so that a thread can fill any run of whole pieces with
// no other thread's state: `fill_share`. The cuts are made by one pass over the records, which
// is also the tiling's whole check, so a block whose cuts are made fills without a refusal.

/// The pieces a block's records are cut into (`cuts`): a team of 1, 2, 3, 4, 6 or 12 threads
/// takes whole pieces, equal shares of them
pub(crate) const PIECES: usize = 12;

/// The fill's state at a cut in a block's records
#[derive(Clone)]
pub(crate) struct Cut {
    /// the records before the cut, and their rows
    rec: usize,
    row: u32,
    /// each byte's next LF value: its bucket's start plus its rows before the cut
    next: [u32; 256],
}

/// The block's records (as `front` left them in `r`, for a block of `n` rows) cut into
/// `PIECES` pieces of about equal work in the fill, into `out`: `PIECES + 1` cuts, the first
/// at the block's start and the last at its end. Returns false, `out` left empty, unless the
/// records tile the block (`fill`'s checks: the counts sum to n, every record is at least a
/// row, and the records' rows are n in all and each byte's are its count).
///
/// A piece's work is taken as a store for each 8 rows and one more for each record.
pub(crate) fn cuts(r: &Runs, n: usize, out: &mut Vec<Cut>) -> bool {
    out.clear();
    let runs = r.records_slice();
    // (below 2^24 records of at most 255 rows: the sums below fit 32 bits)
    if n >= 1 << 24 || runs.len() >= 1 << 24 {
        return false;
    }
    let (mut next, mut lim) = ([0u32; 256], [0u32; 256]);
    let mut cum = 0u64;
    for c in 0..256 {
        next[c] = cum as u32;
        cum += r.counts[c] as u64;
        lim[c] = cum as u32;
    }
    if cum != n as u64 {
        return false;
    }
    let total = n as u64 + 8 * runs.len() as u64;
    // the work at which the next cut falls: cut j at j / PIECES of it
    let at = |j: usize| (total * j as u64).div_ceil(PIECES as u64);
    out.push(Cut {
        rec: 0,
        row: 0,
        next,
    });
    let (mut q, mut work) = (0u32, 0u64);
    let mut due = at(1);
    let mut empty = false;
    for (i, &x) in runs.iter().enumerate() {
        if work >= due {
            while out.len() < PIECES && work >= at(out.len()) {
                out.push(Cut {
                    rec: i,
                    row: q,
                    next,
                });
            }
            due = if out.len() < PIECES {
                at(out.len())
            } else {
                u64::MAX
            };
        }
        let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
        empty |= len == 0;
        next[c] += len;
        q += len;
        work += len as u64 + 8;
    }
    // (each byte's rows are its count, so the rows in all are the counts' sum, n)
    if empty || next != lim {
        out.clear();
        return false;
    }
    // (a block of fewer records than pieces: the pieces left are empty, at the end)
    let end = Cut {
        rec: runs.len(),
        row: q,
        next,
    };
    out.resize(PIECES + 1, end);
    true
}

/// Whether `cuts` can be filled by shares for a block of `n` rows with `r`'s records: what
/// `fill_share`'s callers need of them for the shares to write disjoint rows that tile the
/// block, whatever made them. The first cut is the block's start (no record, row 0, each
/// byte's bucket's start by `r`'s counts, which sum to n), the last its end (every record,
/// row n), and from each cut to the next nothing falls: not the records, the row, or any
/// byte's value, which stays at or below n.
pub(crate) fn cuts_tile(r: &Runs, n: usize, cuts: &[Cut]) -> bool {
    let (Some(first), Some(last)) = (cuts.first(), cuts.last()) else {
        return false;
    };
    if n >= 1 << 24 {
        return false;
    }
    let mut cum = 0u64;
    for c in 0..256 {
        if first.next[c] as u64 != cum {
            return false;
        }
        cum += r.counts[c] as u64;
    }
    if cum != n as u64 || first.rec != 0 || first.row != 0 {
        return false;
    }
    if last.rec != r.records_slice().len() || last.row as usize != n {
        return false;
    }
    if last.next.iter().any(|&v| v as usize > n) {
        return false;
    }
    cuts.windows(2).all(|w| {
        let (a, b) = (&w[0], &w[1]);
        a.rec <= b.rec && a.row <= b.row && a.next.iter().zip(&b.next).all(|(x, y)| x <= y)
    })
}

/// The rows from cut `from` to cut `to` of the block's LF table, written through `out` (the
/// table's first slot): `fill`'s entries for the records between the cuts, `from`'s state
/// carried through them. Returns whether they filled the rows between the cuts exactly, and
/// each byte's values between the cuts exactly (the share's end check).
///
/// It writes only slots `from.row .. to.row`, whatever the records and the cuts hold: a
/// record is refused if it is empty or would pass `to.row` or its byte's value at `to`, and
/// no store runs past `to.row` (a record whose 8-slot stores would is written a slot at a
/// time). A share that passes has `fill`'s entry in each of its rows, each byte's values
/// below `to`'s for it; one that fails may leave anything in its rows, the values of an
/// 8-slot store's slots past its record's end among them, and the caller must not read
/// them.
///
/// # Safety
///
/// `out` must be valid for writes of `to.row` slots, and no other thread may read or write
/// slots `from.row .. to.row` during the call.
pub(crate) unsafe fn fill_share(r: &Runs, from: &Cut, to: &Cut, out: *mut u32, k: Kernels) -> bool {
    let Some(runs) = r.records_slice().get(from.rec..to.rec) else {
        return false;
    };
    if from.row > to.row || from.next.iter().zip(&to.next).any(|(a, b)| a > b) {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected; the rest as this
            // function's contract, with the cuts' order checked above
            return unsafe { fill_share_avx2(runs, from.row, to.row, from.next, &to.next, out) };
        }
    }
    let _ = k;
    // SAFETY: as this function's contract, with the cuts' order checked above
    unsafe { fill_share_scalar(runs, from.row, to.row, from.next, &to.next, out) }
}

/// `fill_share` in scalar code: each entry stored alone
///
/// # Safety
///
/// As `fill_share`, for rows `row .. row_end`; `row <= row_end`, and `next[c] <= lim[c]`
/// for every byte.
unsafe fn fill_share_scalar(
    runs: &[u16],
    row: u32,
    row_end: u32,
    mut next: [u32; 256],
    lim: &[u32; 256],
    out: *mut u32,
) -> bool {
    let mut q = row;
    for &x in runs {
        let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
        // (q <= row_end and next[c] <= lim[c]: neither subtraction wraps)
        if len == 0 || len > row_end - q || len > lim[c] - next[c] {
            return false;
        }
        let e = c as u32 | next[c] << 8;
        for j in 0..len {
            // SAFETY: q + j < q + len <= row_end, a slot of this share's rows
            unsafe { *out.add((q + j) as usize) = e + (j << 8) };
        }
        next[c] += len;
        q += len;
    }
    q == row_end && next == *lim
}

/// `fill_share` with `fill_avx2`'s stores, 8 entries each, for every record whose stores end
/// inside the share's rows: a record's last store runs up to 7 slots past its end, so a
/// record ending within 7 rows of the share's end (at most 7 records) is stored a slot at a
/// time. The slots a store wrote past its record's end are the share's own next records',
/// which write them again.
///
/// # Safety
///
/// AVX2 must be available. Otherwise as `fill_share_scalar`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_share_avx2(
    runs: &[u16],
    row: u32,
    row_end: u32,
    mut next: [u32; 256],
    lim: &[u32; 256],
    out: *mut u32,
) -> bool {
    use std::arch::x86_64::*;
    let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    let step = _mm256_set1_epi32(2048);
    let mut q = row;
    for &x in runs {
        let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
        // (q <= row_end and next[c] <= lim[c]: neither subtraction wraps)
        if len == 0 || len > row_end - q || len > lim[c] - next[c] {
            return false;
        }
        let e = c as u32 | next[c] << 8;
        let end = q + len;
        if row_end - end >= 7 {
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let mut w = q;
            loop {
                // SAFETY: w < end, so w + 8 <= end + 7 <= row_end: inside this share's rows
                unsafe { _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v) };
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
        } else {
            for j in 0..len {
                // SAFETY: q + j < end <= row_end, a slot of this share's rows
                unsafe { *out.add((q + j) as usize) = e + (j << 8) };
            }
        }
        next[c] += len;
        q = end;
    }
    q == row_end && next == *lim
}

// (research only, `mtf-fold`: the MTF/RLE2 front emitting straight into the open kernel's first
// pass, src/runs/fold.rs)
#[cfg(feature = "mtf-fold")]
#[doc(hidden)]
pub mod fold;

#[cfg(test)]
mod tests;
