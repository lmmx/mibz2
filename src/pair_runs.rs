//! Research only, behind the non-default `pair-runs` feature (and compiled for the tests):
//! the dense pair (ψ², two bytes a step) table of #34 built from the last column's run
//! intersections instead of by a per-row bigram counting sort
//! (docs/journal/2026-11-09-pair-from-runs.md). Nothing in the decoder calls this module.
//!
//! - **The old pair path, recovered from `perf/pair-walk` (#34, head `a3687d8f`):** the
//!   table `PairTable` (row r's word `L[r] | ψ²(r) << 8`, bit 31 a segment start, bit 30
//!   the mark on a start's LF row), its per-row builder `build_pairs` and its consumer, the
//!   flagged 32-lane pair walk. The builder, `pair_start` and the walk's loop are #34's
//!   code; the walk is cut in two (`setup`, `walk`) only so that its output can go over the
//!   last column, as production's in-place path does: the last column is read only before
//!   the cut.
//! - **The run-intersection builder, `build_pairs_runs`:** the same words. Ψ maps the F
//!   interval I_i = [a_i, a_i + ℓ_i) of L run i onto J_i = [p_i, p_i + ℓ_i). One monotone
//!   sweep over x cuts [0, n) at the L runs' starts and the F intervals' starts (line A's
//!   cells, at most 2r − 1: #266). On the cell [x, y) of J_i ∩ I_j, with q = a_i + x − p_i,
//!   ψ²(q + t) = p_j + x − a_j + t, so `D[q + t] = L[q + t] | (p_j + x − a_j + t) << 8`:
//!   an arithmetic fill of the cell, OR'd with the last column's bytes at q (which are
//!   contiguous). The cells are filled in F order, so in rising q: the table is written and
//!   the column read in order, and the destinations' non-monotone order is paid instead as
//!   one gather a run into the r-entry arrays (the F interval its image starts in, from a
//!   merge in L order). The F ordering is a counting sort of the r runs by byte.
//!
//!   The low byte is **L[q]**, #34's, not the Researcher's F(Ψ(q)), so that #34's consumer
//!   runs unchanged; L[q] is not constant on a cell, so it is read from the column.
//!
//! **The safe boundary (Guardian D046).** Every unchecked access rests on a premise that a
//! safe entry establishes first, and a type then carries:
//! - the builders check the block (`1 <= n <= MAX_N`, `orig_ptr < n`, the counts summing to
//!   n without overflow and equal to each byte's count in `ll`, the LF rows asked for
//!   strictly rising below n) before any unchecked access, and on success alone return a
//!   [`Built`], which borrows the table and the column immutably. A failed check is an
//!   error, never a reason to try the other builder;
//! - `pair_start` takes a `Built`, and checks its row;
//! - `setup` checks its segment count, builds, flags the table, and on success alone
//!   returns a [`Prepared`], which borrows the scratch's table, segment starts and LF marks
//!   immutably (the scratch's fields are private), so `walk`, which takes it, reads only a
//!   state `setup` made;
//! - the unchecked loops are private `unsafe fn`s with `# Safety` contracts.
//!
//! **Never parallelise the fill across adjacent cells.** The run builder's 8-word stores
//! run past their cell's end into the next cells' rows, which those cells then write again
//! (the sequential spill repair). Logically disjoint cells are not physically disjoint
//! stores: filling adjacent cells (or F intervals) on two threads is a data race, which no
//! later overwrite repairs. A parallel fill would need stores truncated at each cell's
//! owned boundary, or physically disjoint windows with their own completion, and its own
//! proof; nothing here runs the fill in parallel, and nothing may.
//!
//! The tests check the two builders word for word (and their LF rows and start row),
//! exhaustively on small columns and on random ones, and the walk against `bwt::walk`; a
//! differential fuzz checks both against a stable-sort ψ² oracle, cell by cell.

use crate::bwt::{
    prefetch, random_rows, read_u64, InverseBwt, SegScratch, Segmented, FLAGGED_LANES,
    FLAGGED_SEGMENTS, MIN_SEGMENTED, PF_L1, PF_OFF, START,
};

/// The longest block the pair table takes: its links are below 2^22, so a word
/// (`L[r] | link << 8`) is below 2^30 and bits 30 and 31 are free for the walk's flags.
/// (bzip2's blocks are at most 900,000 rows.)
pub const MAX_N: usize = 1 << 22;
/// The most segments `setup` cuts a block into (more than the block's rows is allowed: the
/// starts are deduplicated)
pub const MAX_SEGMENTS: usize = 1 << 16;

/// Why a safe entry refused its input, before any unchecked access
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairError {
    /// no rows
    Empty,
    /// more than `MAX_N` rows
    TooLong(usize),
    /// the primary index is not below n
    OrigPtr { orig_ptr: u32, n: usize },
    /// the counts are not the column's own (their sum, or a byte's count)
    Counts,
    /// the rows whose LF is asked for are not strictly rising below n
    Rows,
    /// a segment count of 0 or above `MAX_SEGMENTS`
    Segments(usize),
}

impl std::fmt::Display for PairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PairError::Empty => write!(f, "an empty block"),
            PairError::TooLong(n) => write!(f, "{n} rows, above the pair table's {MAX_N}"),
            PairError::OrigPtr { orig_ptr, n } => {
                write!(f, "primary index {orig_ptr} not below {n} rows")
            }
            PairError::Counts => write!(f, "the counts are not the column's own"),
            PairError::Rows => write!(f, "the LF rows asked for are not strictly rising below n"),
            PairError::Segments(s) => write!(f, "{s} segments (1 to {MAX_SEGMENTS})"),
        }
    }
}

impl std::error::Error for PairError {}

/// The pair table: row r's entry is its last-column byte L[r] | ψ²(r) << 8, and the byte
/// after L[r] in the text, L[ψ(r)], is F(r), row r's first byte, found from `first` and
/// the buckets' starts `f`. So a step from r gives two bytes and the row two on. Its
/// fields are private: only `build_pairs` and `build_pairs_runs` write it, and only through
/// the [`Built`] they return (or `setup`'s [`Prepared`]) is it read unchecked. A built
/// table's links are below its length n <= `MAX_N` = 2^22, so its words are below 2^30;
/// bits 30 and 31 are the walk's flags.
pub struct PairTable {
    p: Vec<u32>,
    b2: Vec<u32>,
    first: Vec<u8>,
    f: [u32; 257],
    fshift: u32,
}

impl Default for PairTable {
    fn default() -> Self {
        PairTable {
            p: Vec::new(),
            b2: Vec::new(),
            first: Vec::new(),
            f: [0; 257],
            fshift: 0,
        }
    }
}

/// A segment's start (in its own entry), and with it, in the entry of the row one before
/// it in the text (LF of the start), the mark that a lane reaching that row ends there
/// after one byte: a step of two rows passes over every other row, so a start is seen
/// from whichever of the two the lane lands on (libsais's marks and their LF rows)
const PAIR_LF: u32 = 1 << 30;
const PAIR_LINK: u32 = !(START | PAIR_LF);

impl PairTable {
    /// The table's words, as a builder left them (the walk adds its flags); empty, or a
    /// failed build's partial words, unless a build succeeded
    pub fn words(&self) -> &[u32] {
        &self.p
    }

    /// The first-byte lookup's rows a byte (`1 << fshift`) and its length
    pub fn first_lookup(&self) -> (u32, usize) {
        (self.fshift, self.first.len())
    }

    /// Row r's first byte: its F bucket, from the slice of rows r is in
    ///
    /// # Safety
    /// The table was built (by `build_pairs` or `build_pairs_runs`, which returned Ok) and
    /// not changed since but by `setup`'s flags, and r is below its n: then
    /// `r >> fshift < first.len()`, and the bucket ends rise to `f[256] = n > r`, so the
    /// loop stops at some c + 1 <= 256
    #[inline(always)]
    unsafe fn first_byte_unchecked(&self, r: u32) -> u8 {
        // SAFETY: the caller's (above)
        unsafe {
            let mut c = *self.first.get_unchecked((r >> self.fshift) as usize) as usize;
            while *self.f.get_unchecked(c + 1) <= r {
                c += 1;
            }
            c as u8
        }
    }

    /// `first_byte` and the correction loop's iterations (the census's), checked
    fn first_byte_counted(&self, r: u32) -> (u8, u32) {
        let mut c = self.first[(r >> self.fshift) as usize] as usize;
        let mut k = 0;
        while self.f[c + 1] <= r {
            c += 1;
            k += 1;
        }
        (c as u8, k)
    }

    /// The buckets' starts from counts that sum to n (checked by `check_block`), and the
    /// first-byte lookup over them
    fn buckets(&mut self, counts: &[u32; 256], n: usize) {
        let mut cum = 0u32;
        for (f, &k) in self.f.iter_mut().zip(counts) {
            *f = cum;
            cum += k;
        }
        self.f[256] = cum;
        self.fshift = 0;
        while (n >> self.fshift) >= 1 << 15 {
            self.fshift += 1;
        }
        self.first.clear();
        let mut c = 0;
        for j in 0..((n - 1) >> self.fshift) + 1 {
            let r = (j << self.fshift) as u32;
            while self.f[c + 1] <= r {
                c += 1;
            }
            self.first.push(c as u8);
        }
    }
}

/// A table a builder has built from a column whose own counts it checked: the proof that
/// the table's premises hold. Only `build_pairs` and `build_pairs_runs` make one (its
/// fields are private), and it borrows the table and the column immutably for its life, so
/// neither can change under it.
///
/// None can be made by hand:
/// ```compile_fail,E0451
/// let t = mibz2::pair_runs::PairTable::default();
/// let b = mibz2::pair_runs::Built { t: &t, ll: &[], r0: 0 };
/// ```
/// nor can #34's `pair_start` be given a table no builder made (D046's first witness):
/// ```compile_fail,E0308
/// use mibz2::pair_runs::{pair_start, PairTable};
/// let _ = pair_start(&PairTable::default(), 0);
/// ```
/// nor the table changed under one:
/// ```compile_fail,E0499
/// use mibz2::pair_runs::{build_pairs, PairTable};
/// let (mut t, mut lf) = (PairTable::default(), Vec::new());
/// let mut counts = [0u32; 256];
/// counts[7] = 1;
/// let b = build_pairs(&[7], &counts, 0, &[], &mut t, &mut lf).unwrap();
/// let _ = build_pairs(&[7], &counts, 0, &[], &mut t, &mut lf);
/// let _ = b.r0();
/// ```
/// while a checked one is read:
/// ```
/// use mibz2::pair_runs::{build_pairs, pair_start, PairTable};
/// let (mut t, mut lf) = (PairTable::default(), Vec::new());
/// let mut counts = [0u32; 256];
/// counts[7] = 1;
/// let b = build_pairs(&[7], &counts, 0, &[], &mut t, &mut lf).unwrap();
/// assert_eq!((b.r0(), pair_start(&b, 0), b.words()), (0, Ok(0), &[7u32][..]));
/// ```
#[derive(Clone, Copy)]
pub struct Built<'a> {
    t: &'a PairTable,
    ll: &'a [u8],
    r0: u32,
}

impl<'a> Built<'a> {
    /// The block's rows
    pub fn n(&self) -> usize {
        self.ll.len()
    }
    /// The words (`L[r] | ψ²(r) << 8`)
    pub fn words(&self) -> &'a [u32] {
        &self.t.p
    }
    /// The table
    pub fn table(&self) -> &'a PairTable {
        self.t
    }
    /// The text's first row ψ(orig_ptr), for the primary index the build was given
    pub fn r0(&self) -> u32 {
        self.r0
    }
}

/// The checks every safe entry makes before anything unchecked: `1 <= n <= MAX_N`,
/// `orig_ptr < n`, the counts' sum is n (summed in u64, so a wrapping sum is no sum), and
/// `rows` rise strictly below n. Each byte's own count against `ll` is checked by each
/// builder, from the histogram it makes anyway, before its unchecked steps.
fn check_block(
    ll: &[u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    rows: &[u32],
) -> Result<(), PairError> {
    let n = ll.len();
    if n == 0 {
        return Err(PairError::Empty);
    }
    if n > MAX_N {
        return Err(PairError::TooLong(n));
    }
    if orig_ptr as usize >= n {
        return Err(PairError::OrigPtr { orig_ptr, n });
    }
    if counts.iter().map(|&c| c as u64).sum::<u64>() != n as u64 {
        return Err(PairError::Counts);
    }
    if rows.windows(2).any(|w| w[0] >= w[1]) || rows.last().is_some_and(|&r| r as usize >= n) {
        return Err(PairError::Rows);
    }
    Ok(())
}

/// ψ² of the last column `ll`, as libsais builds it (#34's `build_pairs`; the checks
/// added).
/// - A step i of the counting sort has p = LF(i) (its byte c's next slot), and i's row
///   two on from the row LF(p) = LF²(i), the next slot of the bigram (L[p], c): the rows
///   sorted by their first two bytes, in i's order within each.
/// - The bigram counts come from a byte histogram of each F bucket's slice of `ll` (row p
///   in bucket c is bigram (L[p], c)): sequential, not a pass of LF lookups. Summed over
///   the buckets they are `ll`'s own byte counts, checked against `counts` here, before
///   the scatter's unchecked steps.
/// - 16 equal bytes c at i whose LF rows' 16 bytes are equal too (the run lemma, twice)
///   go in with one 16-wide store, libsais's RUN_LANES, as in `build`.
///
/// `rows` (strictly rising, below n) are the rows whose LF is wanted: each one's is pushed
/// to `lf`, in order. Returns the table, with the text's first row ψ(orig_ptr), or the
/// check that failed (`t`'s words then partial, and no `Built` to read them unchecked).
#[allow(clippy::needless_range_loop)]
pub fn build_pairs<'a>(
    ll: &'a [u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    rows: &[u32],
    t: &'a mut PairTable,
    lf: &mut Vec<u32>,
) -> Result<Built<'a>, PairError> {
    t.p.clear();
    lf.clear();
    check_block(ll, counts, orig_ptr, rows)?;
    let n = ll.len();
    t.p.extend(ll.iter().map(|&c| c as u32));
    t.buckets(counts, n);
    // The bigram counts, then each bigram's first slot (first byte major); and each byte's
    // count over the whole column, the buckets' slices tiling it
    t.b2.clear();
    t.b2.resize(1 << 16, 0);
    let mut h = [[0u32; 256]; 4];
    let mut own = [0u32; 256];
    for c in 0..256 {
        let slice = &ll[t.f[c] as usize..t.f[c + 1] as usize];
        if slice.is_empty() {
            continue;
        }
        let mut q = slice.chunks_exact(4);
        for x in &mut q {
            h[0][x[0] as usize] += 1;
            h[1][x[1] as usize] += 1;
            h[2][x[2] as usize] += 1;
            h[3][x[3] as usize] += 1;
        }
        for &d in q.remainder() {
            h[0][d as usize] += 1;
        }
        for d in 0..256 {
            let k = h[0][d] + h[1][d] + h[2][d] + h[3][d];
            t.b2[d << 8 | c] = k;
            own[d] += k;
            (h[0][d], h[1][d], h[2][d], h[3][d]) = (0, 0, 0, 0);
        }
    }
    if own != *counts {
        t.p.clear();
        return Err(PairError::Counts);
    }
    let mut sum = 0u32;
    for e in t.b2.iter_mut() {
        let v = *e;
        *e = sum;
        sum += v;
    }
    // The scatter
    let (p, b2) = (&mut t.p, &mut t.b2);
    let mut b1 = [0u32; 256];
    b1.copy_from_slice(&t.f[..256]);
    let mut next = rows.first().map_or(usize::MAX, |&r| r as usize);
    let mut ri = 0;
    let mut i = 0;
    // SAFETY (each `pair_steps` below): `counts` is `ll`'s own histogram (checked above),
    // `b1` starts at the buckets' starts and `b2` at the bigrams' first slots, and both
    // advance exactly as `pair_steps` and the 16-wide store do, over the rows in rising
    // order, each once: so each holds the next slot of its byte (bigram) when its row comes
    while i + 16 <= n {
        let c = ll[i] as usize;
        let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
        if a == b && a == a.rotate_left(8) {
            let q = b1[c] as usize;
            if q + 16 <= n {
                let (a, b) = (read_u64(ll, q), read_u64(ll, q + 8));
                if a == b && a == a.rotate_left(8) {
                    let w = (ll[q] as usize) << 8 | c;
                    let x = b2[w] as usize;
                    let base = (i as u32) << 8;
                    for (j, e) in p[x..x + 16].iter_mut().enumerate() {
                        *e = (*e & 0xff) | (base + ((j as u32) << 8));
                    }
                    while next < i + 16 {
                        lf.push((q + next - i) as u32);
                        ri += 1;
                        next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
                    }
                    b1[c] += 16;
                    b2[w] += 16;
                    i += 16;
                    continue;
                }
            }
        }
        let e = i + 16;
        if next >= e {
            // SAFETY: above; i..e is below n
            unsafe { pair_steps(ll, p, b2, &mut b1, i..e) };
        } else {
            for i in i..e {
                if i == next {
                    lf.push(b1[ll[i] as usize]);
                    ri += 1;
                    next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
                }
                // SAFETY: above; i is below n
                unsafe { pair_steps(ll, p, b2, &mut b1, i..i + 1) };
            }
        }
        i = e;
    }
    for i in i..n {
        if i == next {
            lf.push(b1[ll[i] as usize]);
            ri += 1;
            next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
        }
        // SAFETY: above; i is below n
        unsafe { pair_steps(ll, p, b2, &mut b1, i..i + 1) };
    }
    // SAFETY: the table was just built from `ll`, whose counts were checked, and orig_ptr
    // is below n (`check_block`)
    let r0 = unsafe { pair_start_unchecked(ll, t, orig_ptr) };
    Ok(Built { t, ll, r0 })
}

/// `build_pairs`'s one-row steps, over the rows `rows`
///
/// # Safety
/// `rows` are below `ll.len()` = n = `p.len()`; `ll`'s histogram is exactly the counts the
/// buckets were made from; `b1[c]` is byte c's next unfilled row (its bucket's start plus
/// the copies of c already stepped), and `b2[w]` bigram w's next unfilled slot, as
/// `build_pairs` keeps them, the rows stepped in rising order, each once. Then
/// `q = b1[c] < f[c + 1] <= n` (c has exactly `counts[c]` copies) and `x = b2[w]` is below
/// its bigram's next first slot, so below n: every unchecked index is in bounds.
#[inline(always)]
unsafe fn pair_steps(
    ll: &[u8],
    p: &mut [u32],
    b2: &mut [u32],
    b1: &mut [u32; 256],
    rows: std::ops::Range<usize>,
) {
    for i in rows {
        // SAFETY: the caller's (above)
        unsafe {
            let c = *ll.get_unchecked(i) as usize;
            let q = *b1.get_unchecked(c) as usize;
            *b1.get_unchecked_mut(c) += 1;
            let w = (*ll.get_unchecked(q) as usize) << 8 | c;
            let x = *b2.get_unchecked(w) as usize;
            *b2.get_unchecked_mut(w) += 1;
            let e = p.get_unchecked_mut(x);
            *e = (*e & 0xff) | (i as u32) << 8;
        }
    }
}

/// The row ψ(orig_ptr) on a built table (#34's): the row whose LF is `orig_ptr`, or the
/// primary index refused
pub fn pair_start(b: &Built<'_>, orig_ptr: u32) -> Result<u32, PairError> {
    if orig_ptr as usize >= b.n() {
        return Err(PairError::OrigPtr { orig_ptr, n: b.n() });
    }
    // SAFETY: `b` is a builder's, of `b.ll`, and orig_ptr is below n (checked)
    Ok(unsafe { pair_start_unchecked(b.ll, b.t, orig_ptr) })
}

/// The row ψ(orig_ptr): the (orig_ptr − f[c])-th c in `ll`, for c = F(orig_ptr) (#34's)
///
/// # Safety
/// `t` was built from `ll` (whose counts the builder checked), and `orig_ptr < ll.len()`:
/// `first_byte_unchecked`'s premise. (The rest is checked: the count is found.)
unsafe fn pair_start_unchecked(ll: &[u8], t: &PairTable, orig_ptr: u32) -> u32 {
    // SAFETY: the caller's
    let c = unsafe { t.first_byte_unchecked(orig_ptr) };
    let mut k = (orig_ptr - t.f[c as usize]) as usize;
    let mut base = 0;
    for chunk in ll.chunks(64) {
        let m = chunk.iter().filter(|&&b| b == c).count();
        if k < m {
            let j = chunk
                .iter()
                .enumerate()
                .filter(|&(_, &b)| b == c)
                .nth(k)
                .unwrap()
                .0;
            return (base + j) as u32;
        }
        k -= m;
        base += chunk.len();
    }
    unreachable!("the counts are ll's own")
}

/// What the run builder's fill did, cell by cell (the tests' and the fuzz's assertions)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FillStats {
    /// the cells (F intervals cut at the L runs' starts' images)
    pub cells: usize,
    /// the cells filled 8 words a store, and those of them whose last store ran past the
    /// cell's end (into the rows a later cell writes again: the spill repair)
    pub vector_cells: usize,
    pub spills: usize,
    /// the cells filled a word at a time, near the block's end (the scalar tail)
    pub scalar_cells: usize,
    /// whether the fill was the AVX2 one (else SSE2, or portable off x86_64)
    pub avx2: bool,
}

/// `build_pairs_runs`'s buffers, kept from block to block. Its arrays are private.
#[derive(Default)]
pub struct RunScratch {
    /// L order: each run's first position (and n after the last), and its F interval's
    /// first row a = C(c) + (the copies of c in L before it)
    p: Vec<u32>,
    a: Vec<u32>,
    /// F order (the runs stable-sorted by byte): each interval's first row and its run's
    /// first position in L (and [n, 0] after the last), and its run's index in L order
    fo: Vec<[u32; 2]>,
    fi: Vec<u32>,
    c: Vec<u8>,
    /// L order: the F interval each run's image starts in
    js: Vec<u32>,
    /// the last build's runs, and its fill
    pub runs: usize,
    pub fill: FillStats,
}

/// The pair table `build_pairs` builds, word for word, and the same LF rows for `rows`
/// (strictly rising, below n) in `lf`, from the runs of `ll`:
/// - the runs, in one pass over `ll` (a run start wherever a byte differs from the one
///   before, 8 bytes at a time), each with its F interval's first row a, from `counts` and
///   the copies of its byte before it;
/// - the F order, a stable counting sort of the r runs by byte (`fa` rises: within a byte
///   the runs' a rise in L order);
/// - a merge of the L runs' starts with the F intervals' starts, giving the F interval
///   each run's image J_i begins in;
/// - the fill, in F order, so in rising q: each run's I_i cut where J_i crosses an F
///   interval's start, each cell [x, y) of J_i ∩ I_j filling `D[q..q + y − x]` at
///   q = a_i + x − p_i, 8 words a store. The table and the column are read and written
///   in order; the random accesses are to the r-entry arrays, one gather a run.
///
/// Returns the table, with the text's first row ψ(orig_ptr), or the check that failed
/// (`check_block`'s, or the runs' byte counts against `counts`, checked before the fill).
/// A failed check is an error: it is never a reason to build with `build_pairs` instead.
#[allow(clippy::needless_range_loop)]
pub fn build_pairs_runs<'a>(
    ll: &'a [u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    rows: &[u32],
    t: &'a mut PairTable,
    s: &mut RunScratch,
    lf: &mut Vec<u32>,
) -> Result<Built<'a>, PairError> {
    t.p.clear();
    lf.clear();
    (s.runs, s.fill) = (0, FillStats::default());
    check_block(ll, counts, orig_ptr, rows)?;
    let n = ll.len();
    // the buckets' starts, and the first-byte lookup (as `build_pairs` makes them)
    t.buckets(counts, n);

    // The runs in L order, each with its a, and the runs of each byte: a run starts where
    // a byte differs from the one before, found 8 bytes at a time
    s.p.clear();
    s.a.clear();
    s.c.clear();
    s.p.reserve(n + 1);
    s.a.reserve(n);
    s.c.reserve(n);
    let (pp, ap, cp) = (s.p.as_mut_ptr(), s.a.as_mut_ptr(), s.c.as_mut_ptr());
    let lp = ll.as_ptr();
    let mut seen = [0u32; 256];
    let mut per = [0u32; 257];
    let (mut prev, mut pc, mut r) = (0usize, 0u8, 0usize);
    // a run starts at `pos` (the one before it, if any, closes): its record written. (Its
    // a, f[c] + seen[c], is at most 2n < 2^23: f[c] <= n, the counts summing to n.)
    macro_rules! start {
        ($pos:expr) => {{
            let pos = $pos;
            seen[pc as usize] += (pos - prev) as u32;
            let c = ll[pos];
            debug_assert!(r < n);
            // SAFETY: the starts rise strictly from 0 and are below n, so there are at
            // most n of them and r < n, within the capacities reserved above
            unsafe {
                pp.add(r).write(pos as u32);
                ap.add(r).write(t.f[c as usize] + seen[c as usize]);
                cp.add(r).write(c);
            }
            r += 1;
            per[c as usize + 1] += 1;
            (prev, pc) = (pos, c);
        }};
    }
    start!(0);
    const LO7: u64 = 0x7f7f_7f7f_7f7f_7f7f;
    let mut w = 1;
    while w + 8 <= n {
        // SAFETY: w ≥ 1 and w + 8 <= n, so both 8-byte reads are within ll
        let x = unsafe {
            u64::from_le((lp.add(w) as *const u64).read_unaligned())
                ^ u64::from_le((lp.add(w - 1) as *const u64).read_unaligned())
        };
        if x != 0 {
            // bit 7 of each byte that differs from the byte before it
            let mut m = ((x & LO7).wrapping_add(LO7) | x) & !LO7;
            while m != 0 {
                start!(w + (m.trailing_zeros() >> 3) as usize);
                m &= m - 1;
            }
        }
        w += 8;
    }
    for pos in w..n {
        if ll[pos] != ll[pos - 1] {
            start!(pos);
        }
    }
    seen[pc as usize] += (n - prev) as u32;
    // SAFETY: records 0..r were each written once, above
    unsafe {
        s.p.set_len(r);
        s.a.set_len(r);
        s.c.set_len(r);
    }
    // each byte's runs' lengths are its count: the F intervals then tile [0, n), the fill's
    // `set_len` premise
    if seen != *counts {
        return Err(PairError::Counts);
    }
    s.p.push(n as u32);

    // The F order: the runs counting-sorted by byte, stable (each one's F interval's first
    // row, its run's first position in L, and its index in L order)
    for c in 0..256 {
        per[c + 1] += per[c];
    }
    s.fo.clear();
    s.fo.resize(r + 1, [n as u32, 0]);
    s.fi.clear();
    s.fi.resize(r, 0);
    for k in 0..r {
        let c = s.c[k] as usize;
        let slot = per[c] as usize;
        per[c] += 1;
        s.fo[slot] = [s.a[k], s.p[k]];
        s.fi[slot] = k as u32;
    }
    let fo = &s.fo[..];
    // In L order, the F interval each run's image J_i = [p_i, p_{i+1}) starts in: one
    // merge of the L runs' starts with the F intervals' (both rise)
    s.js.clear();
    let mut j = 0;
    for &p in &s.p[..r] {
        // SAFETY: p < n = fo[r][0], so the loop stops with j + 1 <= r, within fo
        while unsafe { fo.get_unchecked(j + 1)[0] } <= p {
            j += 1;
        }
        s.js.push(j as u32);
    }

    // The fill, in F order, so in rising q (serially: see `fill`'s contract)
    t.p.reserve(n + 8);
    let d = &mut t.p.spare_capacity_mut()[..n + 8];
    #[cfg(target_arch = "x86_64")]
    let fill_stats = if std::is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 is present
        let mut f = unsafe { fill_avx2(ll, d, fo, &s.fi, &s.js) };
        f.avx2 = true;
        f
    } else {
        // SAFETY: without AVX2, no precondition
        unsafe { fill::<false>(ll, d, fo, &s.fi, &s.js) }
    };
    #[cfg(not(target_arch = "x86_64"))]
    // SAFETY: without AVX2, no precondition
    let fill_stats = unsafe { fill::<false>(ll, d, fo, &s.fi, &s.js) };
    // SAFETY: every slot below n was last written by its own cell. The F intervals tile
    // [0, n) in F order (the runs of each byte c, in L order, have a = C(c) + the lengths
    // of the runs of c before them; their lengths sum to counts[c], checked, and the counts
    // sum to n, checked); each interval's cells, as x sweeps its image, tile it in rising
    // q; so the cells tile [0, n) in the order they are written. A wide write goes past its
    // cell only upward, into rows of later cells, each of which then writes its own rows.
    unsafe { t.p.set_len(n) };
    s.runs = r;
    s.fill = fill_stats;
    // The LF rows of `rows` (rising, below n: checked): row v in L run i has
    // LF(v) = a_i + v − p_i
    let mut i = 0;
    for &v in rows {
        while s.p[i + 1] <= v {
            i += 1;
        }
        lf.push(s.a[i] + (v - s.p[i]));
    }
    // ψ(orig_ptr): orig_ptr in F interval j
    let j = s.fo.partition_point(|e| e[0] <= orig_ptr) - 1;
    let r0 = s.fo[j][1] + (orig_ptr - s.fo[j][0]);
    Ok(Built { t, ll, r0 })
}

/// Whether a built table's words are ψ² by definition: row r's word is
/// `L[r] | ψ(ψ(r)) << 8`, ψ read from production's one-hop table (`bwt::build`'s, built
/// here from the column's own counts); the slow check
pub fn is_psi2(b: &Built<'_>) -> bool {
    let ll = b.ll;
    let mut counts = [0u32; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut tt = crate::bwt::TVector::new();
    crate::bwt::build(ll, &counts, &mut tt);
    let psi = |r: usize| (tt.0[r] >> 8) as usize;
    b.t.p.len() == ll.len()
        && b.t
            .p
            .iter()
            .enumerate()
            .all(|(r, &w)| w == ll[r] as u32 | (psi(psi(r)) as u32) << 8)
}

/// `build_pairs_runs`'s fill with AVX2's 8-lane words
///
/// # Safety
/// AVX2 is present
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_avx2(
    ll: &[u8],
    d: &mut [std::mem::MaybeUninit<u32>],
    fo: &[[u32; 2]],
    fi: &[u32],
    js: &[u32],
) -> FillStats {
    // SAFETY: AVX2 is present (the caller's)
    unsafe { fill::<true>(ll, d, fo, fi, js) }
}

/// `build_pairs_runs`'s fill, in F order, so in rising q: F interval k is run i = fi[k]'s
/// I_i, rows [fo[k][0], fo[k + 1][0]); its image J_i starts at x = fo[k][1], in F interval
/// js[i]; on each cell [x, end) of J_i ∩ I_j, ψ(q + t) = x + t and ψ(x + t) = fo[j][1] + x
/// − fo[j][0] + t. The words go in 8 at a time, past the cell's end into rows the next
/// cells (in rising q) overwrite, or exactly near the block's end. `d` holds n + 8 words
/// (n = ll.len()); `fo` holds the r F intervals and [n, 0]; `js` and `fi` are the merge's
/// and the sort's. Returns what it did. Its memory safety doesn't rest on those being
/// right (each index is checked, or bounded by a check); a wrong input panics or writes
/// wrong words in bounds. That every slot below n is written (`build_pairs_runs`'s
/// `set_len`) does rest on them: on the F intervals tiling [0, n).
///
/// **Serial only (Guardian D046).** A vector store writes up to 7 words past its cell, into
/// the next cells' rows, which those cells then write again: the cells must be filled one
/// after another in rising q, on one thread. Adjacent cells or F intervals filled
/// concurrently would race on those words (a data race, which a later store cannot undo),
/// however disjoint their logical cells. Never call this, or split its loop, in parallel.
///
/// # Safety
/// With `AVX2`, AVX2 is present.
#[inline(always)]
unsafe fn fill<const AVX2: bool>(
    ll: &[u8],
    d: &mut [std::mem::MaybeUninit<u32>],
    fo: &[[u32; 2]],
    fi: &[u32],
    js: &[u32],
) -> FillStats {
    let n = ll.len();
    assert!(d.len() >= n + 8 && !fo.is_empty() && fo[fo.len() - 1][0] as usize == n);
    let r = fo.len() - 1;
    let dp = d.as_mut_ptr() as *mut u32;
    let lp = ll.as_ptr();
    let mut st = FillStats::default();
    for k in 0..r {
        let [a, mut x] = fo[k];
        let mut q = a as usize;
        let xe = x + (fo[k + 1][0] - a);
        let mut j = js[fi[k] as usize] as usize;
        assert!(j < r);
        loop {
            // SAFETY: j + 1 <= r: j starts below r, and steps only past an F interval that
            // ends (fo[j + 1][0] = end) before xe <= n = fo[r][0], so not at fo[r]
            let ([fa, fp], fe) = unsafe { (*fo.get_unchecked(j), fo.get_unchecked(j + 1)[0]) };
            debug_assert!(fa <= x && x < fe && x < xe);
            let end = fe.min(xe);
            let len = (end - x) as usize;
            let base = (fp + (x - fa)) << 8;
            if n.checked_sub(q)
                .is_some_and(|room| room >= 8 && len <= room - 8)
            {
                let mut o = 0;
                loop {
                    // SAFETY: o < len, so q + o + 8 < q + len + 8 <= n (checked, without
                    // overflow): the 8 bytes read and the 8 words written are in bounds
                    unsafe {
                        fill8::<AVX2>(lp.add(q + o), dp.add(q + o), base + ((o as u32) << 8))
                    };
                    o += 8;
                    if o >= len {
                        break;
                    }
                }
                st.vector_cells += 1;
                st.spills += (o > len) as usize;
            } else {
                for (o, (e, &l)) in d[q..q + len].iter_mut().zip(&ll[q..q + len]).enumerate() {
                    e.write(l as u32 | (base + ((o as u32) << 8)));
                }
                st.scalar_cells += 1;
            }
            st.cells += 1;
            q += len;
            if end >= xe {
                break;
            }
            j += 1;
            assert!(j < r);
            x = end;
        }
    }
    st
}

/// Eight words `l[m] | (base + 256·m)` at `d`, from the eight bytes at `l`
///
/// # Safety
/// `l` is readable for 8 bytes and `d` writable for 8 words (neither aligned); with `AVX2`,
/// AVX2 is present
#[inline(always)]
unsafe fn fill8<const AVX2: bool>(l: *const u8, d: *mut u32, base: u32) {
    #[cfg(target_arch = "x86_64")]
    if AVX2 {
        // SAFETY: AVX2 is present (the caller's); the caller's bounds
        unsafe {
            use core::arch::x86_64::*;
            let b = _mm256_cvtepu8_epi32(_mm_loadl_epi64(l as *const __m128i));
            let v = _mm256_add_epi32(
                _mm256_set1_epi32(base as i32),
                _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792),
            );
            _mm256_storeu_si256(d as *mut __m256i, _mm256_or_si256(b, v));
        }
        return;
    }
    #[cfg(target_arch = "x86_64")]
    // SAFETY: SSE2 is in x86_64's baseline; the caller's bounds
    unsafe {
        use core::arch::x86_64::*;
        let z = _mm_setzero_si128();
        let b = _mm_unpacklo_epi8(_mm_loadl_epi64(l as *const __m128i), z);
        let v = _mm_add_epi32(
            _mm_set1_epi32(base as i32),
            _mm_setr_epi32(0, 256, 512, 768),
        );
        let w = _mm_add_epi32(v, _mm_set1_epi32(1024));
        _mm_storeu_si128(d as *mut __m128i, _mm_or_si128(_mm_unpacklo_epi16(b, z), v));
        _mm_storeu_si128(
            d.add(4) as *mut __m128i,
            _mm_or_si128(_mm_unpackhi_epi16(b, z), w),
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    // SAFETY: the caller's bounds
    unsafe {
        let b = (l as *const [u8; 8]).read_unaligned();
        let w: [u32; 8] = std::array::from_fn(|m| b[m] as u32 | (base + ((m as u32) << 8)));
        (d as *mut [u32; 8]).write_unaligned(w);
    }
}

/// Which builder the pair walk uses
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Build {
    /// #34's `build_pairs`
    #[default]
    Rows,
    /// `build_pairs_runs` (no fallback: its checks are `build_pairs`'s, and a block either
    /// fails both or neither)
    Runs,
}

/// The pair walk's buffers, kept from block to block. Its fields are private: the table,
/// the segment starts and the LF marks are written only by `setup`, and read by `walk`
/// only through the [`Prepared`] `setup` returns.
#[derive(Default)]
pub struct PairScratch {
    t: PairTable,
    seg: SegScratch,
    lfs: Vec<(u32, u32)>,
    lf_rows: Vec<u32>,
    runs: RunScratch,
}

impl PairScratch {
    /// The run builder's last build (its runs and fill), for the diagnostic's census
    pub fn runs(&self) -> &RunScratch {
        &self.runs
    }
}

/// A block `setup` has prepared: its flagged table, its segment starts (sorted, distinct,
/// the text's first row among them, no start another's LF row), its LF marks, and the
/// text's first row. Only `setup` makes one (its fields are private), and it borrows the
/// scratch for its life, the table, starts and marks immutably, so none of them can be
/// replaced or rebuilt under it (D046's fourth witness). `walk` takes it by value.
///
/// None can be made by hand:
/// ```compile_fail,E0451
/// let t = mibz2::pair_runs::PairTable::default();
/// let mut v = (Vec::new(), Vec::new());
/// let p = mibz2::pair_runs::Prepared {
///     t: &t, starts: &[0], lfs: &[], bufs: &mut v.0, next: &mut v.1, r0: 0,
/// };
/// ```
/// nor the scratch's table replaced (the witness itself):
/// ```compile_fail,E0616
/// let mut x = mibz2::pair_runs::PairScratch::default();
/// x.t = mibz2::pair_runs::PairTable::default();
/// ```
/// nor the scratch prepared again under one:
/// ```compile_fail,E0499
/// use mibz2::pair_runs::{setup, walk, Build, PairScratch};
/// let (mut x, mut out) = (PairScratch::default(), Vec::new());
/// let mut counts = [0u32; 256];
/// counts[0] = 17;
/// let p = setup(Build::Runs, &mut x, &[0; 17], &counts, 0, 2).unwrap();
/// let q = setup(Build::Runs, &mut x, &[0; 17], &counts, 0, 2).unwrap();
/// walk::<1, false>(p, &mut out);
/// ```
/// while a prepared one walks:
/// ```
/// use mibz2::pair_runs::{setup, walk, Build, PairScratch};
/// let (mut x, mut out) = (PairScratch::default(), Vec::new());
/// let mut counts = [0u32; 256];
/// counts[0] = 17;
/// let p = setup(Build::Runs, &mut x, &[0; 17], &counts, 0, 2).unwrap();
/// walk::<1, false>(p, &mut out);
/// assert_eq!(out, [0; 17]);
/// ```
pub struct Prepared<'a> {
    t: &'a PairTable,
    starts: &'a [u32],
    lfs: &'a [(u32, u32)],
    bufs: &'a mut Vec<Vec<u8>>,
    next: &'a mut Vec<usize>,
    r0: u32,
}

impl Prepared<'_> {
    /// The block's rows
    pub fn n(&self) -> usize {
        self.t.p.len()
    }
    /// The text's first row
    pub fn r0(&self) -> u32 {
        self.r0
    }
    /// The segment starts
    pub fn starts(&self) -> &[u32] {
        self.starts
    }
}

/// One lane's state in the pair walk (`bwt`'s `Lane`)
#[derive(Clone, Copy)]
struct Lane {
    row: u32,
    seg: usize,
    p: *mut u8,
    end: *mut u8,
}

fn next_unset(l: &Lane, next: &[usize]) -> bool {
    l.seg != usize::MAX && next[l.seg] == usize::MAX
}

/// The pair walk's first half, #34's `walk_pairs_flagged` up to its lanes: the segment
/// starts, the table (by `build`), the text's first row, the dropped starts and the
/// flags. The last column's last read. Returns the prepared block, or the check that
/// failed (the builder's, or `segments` outside 1 to `MAX_SEGMENTS`).
pub fn setup<'a>(
    build: Build,
    x: &'a mut PairScratch,
    ll: &[u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    segments: usize,
) -> Result<Prepared<'a>, PairError> {
    if !(1..=MAX_SEGMENTS).contains(&segments) {
        return Err(PairError::Segments(segments));
    }
    let n = ll.len();
    let PairScratch {
        t,
        seg: s,
        lfs,
        lf_rows,
        runs,
    } = x;
    lfs.clear();
    s.starts.clear();
    if n == 0 {
        // (random_rows's modulus: the builders refuse it below)
        t.p.clear();
        return Err(PairError::Empty);
    }
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.sort_unstable();
    s.starts.dedup();
    let built = match build {
        Build::Runs => build_pairs_runs(ll, counts, orig_ptr, &s.starts, t, runs, lf_rows),
        Build::Rows => build_pairs(ll, counts, orig_ptr, &s.starts, t, lf_rows),
    };
    let r0 = match built {
        Ok(b) => b.r0(),
        Err(e) => {
            s.starts.clear();
            return Err(e);
        }
    };
    // (row, its LF) for every start, the text's first row among them, sorted by row
    let mut st: Vec<(u32, u32)> = s
        .starts
        .iter()
        .copied()
        .zip(lf_rows.iter().copied())
        .collect();
    match st.binary_search_by_key(&r0, |e| e.0) {
        Ok(_) => {}
        Err(k) => st.insert(k, (r0, orig_ptr)),
    }
    // A start whose LF row is another start: one of the two dropped (never the first row)
    let mut drop = vec![false; st.len()];
    for k in 0..st.len() {
        // (a row that is its own LF, a cycle of one, needs no LF mark: its lane comes back
        // to it. ψ has several cycles when a block's text is periodic, as `pattern` is)
        if let Ok(j) = st.binary_search_by_key(&st[k].1, |e| e.0) {
            if j == k {
                continue;
            }
            if st[j].0 == r0 {
                drop[k] = true;
            } else {
                drop[j] = true;
            }
        }
    }
    let mut kept = drop.iter();
    st.retain(|_| !*kept.next().unwrap());
    s.starts.clear();
    s.starts.extend(st.iter().map(|e| e.0));
    lfs.extend(st.iter().enumerate().map(|(k, e)| (e.1, k as u32)));
    lfs.sort_unstable();
    for &(r, l) in &st {
        if l != r {
            t.p[l as usize] |= START | PAIR_LF;
        }
        t.p[r as usize] |= START;
    }
    let SegScratch {
        starts, bufs, next, ..
    } = s;
    Ok(Prepared {
        t,
        starts,
        lfs,
        bufs,
        next,
        r0,
    })
}

/// What the walk did (the tests' and the fuzz's assertions)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkStats {
    /// segments ended at a start's own row, and at a start's LF row (one byte more)
    pub start_ends: usize,
    pub lf_ends: usize,
    /// a lane's buffer grown (its pointers refreshed)
    pub grows: usize,
}

/// The pair walk's second half, #34's loop unchanged: `L` lanes over the flagged table a
/// [`Prepared`] block holds, two bytes a step, then the segments joined from the text's
/// first row into `out` (cleared). Reads only the prepared state, never the last column.
pub fn walk<const L: usize, const PF: bool>(p: Prepared<'_>, out: &mut Vec<u8>) -> WalkStats {
    const { assert!(L > 0, "a walk of no lanes never ends") };
    // SAFETY: `p` is `setup`'s (only it makes one), and borrows what it prepared
    // immutably: `walk_lanes`'s premise
    unsafe { walk_lanes::<L, PF>(p, out) }
}

/// `walk`'s lanes
///
/// # Safety
/// `p` is as `setup` returned it: its table built from a column whose counts were checked
/// (so every link is below n and the first-byte lookup is the column's), then flagged by
/// `setup` (START on each start, START | PAIR_LF on each start's LF row unless it is its
/// own, no start another's LF row), its starts sorted, distinct and below n with the text's
/// first row among them, and its LF marks (LF row, start index) sorted. Then every row a
/// lane holds is a link or a start, below n, so `tp.add(row)` and `first_byte_unchecked`
/// are in bounds, and a lane reaching a flagged row ends at a known start.
unsafe fn walk_lanes<const L: usize, const PF: bool>(
    p: Prepared<'_>,
    out: &mut Vec<u8>,
) -> WalkStats {
    let Prepared {
        t,
        starts,
        lfs,
        bufs,
        next,
        r0,
    } = p;
    let n = t.p.len();
    out.clear();
    let ns = starts.len();
    let cap = 2 * n / ns + 64;
    bufs.resize_with(ns.max(bufs.len()), Vec::new);
    for b in &mut bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    next.clear();
    next.resize(ns, usize::MAX);
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let seg_after = |row: u32| match lfs.binary_search_by_key(&row, |e| e.0) {
        Ok(k) => lfs[k].1 as usize,
        Err(_) => usize::MAX,
    };
    let tp = t.p.as_ptr();
    let tt = t;
    let pf = |row: u32| {
        if PF {
            prefetch::<PF_L1>(tp, row)
        } else {
            prefetch::<PF_OFF>(tp, row)
        }
    };
    let mut ws = WalkStats::default();

    // A lane takes segment `seg`: its first step (its start's own, flagged, entry) here
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        let r = starts[seg];
        // SAFETY: the starts are rows below n (the caller's); the buffer has `cap` > 2
        // spare bytes
        let q = unsafe { *tp.add(r as usize) } & PAIR_LINK;
        debug_assert!(((q >> 8) as usize) < n);
        unsafe {
            *base = q as u8;
            *base.add(1) = tt.first_byte_unchecked(r);
        }
        Lane {
            row: q >> 8,
            seg,
            p: unsafe { base.add(2) },
            end: unsafe { base.add(b.capacity()) },
        }
    };
    // A lane reads a flagged entry at its row: its segment ends there (after the row's
    // byte, at an LF row), its length and successor recorded
    let mut finish = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        if q & PAIR_LF != 0 {
            // SAFETY: a step leaves at least 2 spare bytes
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            next[l.seg] = seg_after(l.row);
            ws.lf_ends += 1;
        } else {
            next[l.seg] = seg_of(l.row);
            ws.start_ends += 1;
        }
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
    };
    // Fewer than 2 spare bytes: the buffer grows (its written bytes kept, its pointers
    // refreshed: the old allocation is gone)
    let mut grows = 0;
    let mut grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        grows += 1;
    };
    // One step: two bytes, and the row two on
    let mut step = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>| {
        let r = l.row;
        l.row = q >> 8;
        debug_assert!((l.row as usize) < n);
        pf(l.row);
        // SAFETY: at least 2 spare bytes, within the buffer's allocation; r < n (the
        // caller's)
        unsafe {
            (l.p as *mut u16)
                .write_unaligned(u16::from_le_bytes([q as u8, tt.first_byte_unchecked(r)]));
            l.p = l.p.add(2);
        }
        if unsafe { l.end.offset_from(l.p) } < 2 {
            grow(l, bufs);
        }
    };

    let mut queued = 0;
    let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
        row: 0,
        seg: usize::MAX,
        p: std::ptr::null_mut(),
        end: std::ptr::null_mut(),
    });
    let mut active = 0;
    while active < L && queued < ns {
        lanes[active] = take(queued, bufs);
        pf(lanes[active].row);
        queued += 1;
        active += 1;
    }
    if active == L {
        'full: loop {
            for l in lanes.iter_mut() {
                // SAFETY: rows are below n (the caller's)
                let q = unsafe { *tp.add(l.row as usize) };
                if q & START != 0 {
                    finish(l, q, bufs, next);
                    if queued == ns {
                        break 'full;
                    }
                    *l = take(queued, bufs);
                    pf(l.row);
                    queued += 1;
                    continue;
                }
                step(l, q, bufs);
            }
        }
    }
    let mut live: Vec<Lane> = lanes[..active]
        .iter()
        .copied()
        .filter(|l| next_unset(l, next))
        .collect();
    while !live.is_empty() {
        let mut i = 0;
        while i < live.len() {
            let l = &mut live[i];
            // SAFETY: rows are below n (the caller's)
            let q = unsafe { *tp.add(l.row as usize) };
            if q & START != 0 {
                finish(l, q, bufs, next);
                live.swap_remove(i);
                continue;
            }
            step(l, q, bufs);
            i += 1;
        }
    }
    out.reserve(n);
    let mut seg = seg_of(r0);
    while out.len() < n {
        let b = &bufs[seg];
        out.extend_from_slice(&b[..b.len().min(n - out.len())]);
        seg = next[seg];
    }
    ws.grows = grows;
    ws
}

/// The largest sampled run density (runs a row, `run_density`) at which `Pairs` on the
/// run builder takes a block; above it the block goes to production's `Segmented`.
///
/// **How it was chosen (disclosed, docs/journal/2026-11-09-pair-from-runs.md §8):** the
/// need for an admission rule followed the local pilot's losses on the records and random
/// corpora, and 0.15 was picked from the corpora's observed run densities (Wikidata about
/// 0.07, the synthetic text 0.24, records and random near 1) to separate those families,
/// before the registered run but not before any timing. It is frozen as it is, with the
/// sampler, as part of the registered R candidate.
pub const ADMIT_DENSITY: f64 = 0.15;

/// A heuristic estimate of a column's runs a row, **not a bound on it**: the run starts in
/// one 8-byte word of every 16 (a fixed 1/16 sample, the comparisons at offsets 1 to 8 of
/// each 128 bytes), counted 8 bytes at a time; 1.0 for a column of fewer than 9 bytes.
/// A column can alias the sample: the Researcher's witness (6730270019, kept as
/// `tests::admission_heuristic_aliases`) samples to 0 at a true density of 0.9375, and is
/// admitted. Frozen literally for the registration; robustness is later work.
pub fn run_density(ll: &[u8]) -> f64 {
    const LO7: u64 = 0x7f7f_7f7f_7f7f_7f7f;
    let (mut starts, mut bytes) = (0u32, 0usize);
    let mut w = 1;
    while w + 8 <= ll.len() {
        let x = u64::from_le_bytes(ll[w..w + 8].try_into().unwrap())
            ^ u64::from_le_bytes(ll[w - 1..w + 7].try_into().unwrap());
        starts += (((x & LO7).wrapping_add(LO7) | x) & !LO7).count_ones();
        bytes += 8;
        w += 128;
    }
    if bytes == 0 {
        return 1.0;
    }
    starts as f64 / bytes as f64
}

/// The pair walk as an inverse-BWT kernel (#34's `Pairs`, its builder chosen): the
/// segmented walk for a short block. In place as production's `Segmented` is: the table
/// is the last column's last read, and the output then goes over it. On the run builder,
/// a block whose sampled run density is above `ADMIT_DENSITY` goes to production's
/// `Segmented` instead (counted in `declined`). A block the builders refuse (its counts
/// not its own, its primary index out of range) panics, as production's kernels do on an
/// out-of-range primary index: it is never walked.
#[derive(Default)]
pub struct Pairs {
    pub build: Build,
    pub x: PairScratch,
    short: Segmented,
    /// blocks the run builder declined (`ADMIT_DENSITY`), and blocks taken
    pub declined: usize,
    pub taken: usize,
}

impl Pairs {
    pub fn new(build: Build) -> Self {
        Pairs {
            build,
            ..Pairs::default()
        }
    }
}

impl InverseBwt for Pairs {
    fn invert(&mut self, ll: &[u8], counts: &[u32; 256], orig_ptr: u32, out: &mut Vec<u8>) {
        if ll.len() < MIN_SEGMENTED {
            return self.short.invert(ll, counts, orig_ptr, out);
        }
        if self.build == Build::Runs && run_density(ll) > ADMIT_DENSITY {
            self.declined += 1;
            return self.short.invert(ll, counts, orig_ptr, out);
        }
        self.taken += 1;
        let p = setup(
            self.build,
            &mut self.x,
            ll,
            counts,
            orig_ptr,
            FLAGGED_SEGMENTS,
        )
        .unwrap_or_else(|e| panic!("the pair walk: {e}"));
        walk::<FLAGGED_LANES, true>(p, out);
    }

    fn invert_in_place(&mut self, ll: &mut Vec<u8>, counts: &[u32; 256], orig_ptr: u32) -> bool {
        if ll.len() < MIN_SEGMENTED {
            return self.short.invert_in_place(ll, counts, orig_ptr);
        }
        if self.build == Build::Runs && run_density(ll) > ADMIT_DENSITY {
            self.declined += 1;
            return self.short.invert_in_place(ll, counts, orig_ptr);
        }
        self.taken += 1;
        let p = setup(
            self.build,
            &mut self.x,
            ll,
            counts,
            orig_ptr,
            FLAGGED_SEGMENTS,
        )
        .unwrap_or_else(|e| panic!("the pair walk: {e}"));
        walk::<FLAGGED_LANES, true>(p, ll);
        true
    }
}

/// The spare-bit census (the Researcher's second idea, not built): F rows in blocks of 256,
/// a block flagged where its first and last rows' bytes differ in their high five bits
#[derive(Clone, Copy, Debug, Default)]
pub struct SpareBits {
    pub n: usize,
    /// the directory's bytes (one a block of 256 rows) and the old lookup's (one a
    /// `1 << fshift` rows)
    pub dir_bytes: usize,
    pub old_bytes: usize,
    pub flagged_blocks: usize,
    /// rows in flagged blocks (the distinct rows a whole-block fallback covers)
    pub flagged_rows: usize,
    /// a one-chain pair walk of the text from ψ(orig_ptr), n / 2 steps: the steps whose
    /// row (whose first byte the step looks up) is in a flagged block, and the old
    /// lookup's correction-loop iterations over those steps
    pub steps: usize,
    pub flagged_steps: usize,
    pub ends_iters: u64,
}

/// The census on a built (unflagged) table, from its text's first row (checked indexing)
pub fn spare_bits(b: &Built<'_>) -> SpareBits {
    let t = b.t;
    let n = t.p.len();
    let blocks = n.div_ceil(256);
    let byte_at = |r: usize| t.first_byte_counted(r as u32).0;
    let flagged: Vec<bool> = (0..blocks)
        .map(|b| {
            let (lo, hi) = (b * 256, ((b + 1) * 256).min(n) - 1);
            byte_at(lo) >> 3 != byte_at(hi) >> 3
        })
        .collect();
    let mut c = SpareBits {
        n,
        dir_bytes: blocks,
        old_bytes: t.first.len(),
        ..SpareBits::default()
    };
    for (b, &f) in flagged.iter().enumerate() {
        if f {
            c.flagged_blocks += 1;
            c.flagged_rows += ((b + 1) * 256).min(n) - b * 256;
        }
    }
    let mut r = b.r0;
    for _ in 0..n / 2 {
        c.steps += 1;
        if flagged[r as usize >> 8] {
            c.flagged_steps += 1;
        }
        c.ends_iters += t.first_byte_counted(r).1 as u64;
        r = (t.p[r as usize] & PAIR_LINK) >> 8;
    }
    c
}

#[cfg(test)]
mod tests;
