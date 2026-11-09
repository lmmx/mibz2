//! Research, behind the `run-fed` feature: the T-vector built from MTF run records, with no
//! last column (line A of `docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md`,
//! made production-shaped for one diagnostic: `docs/journal/2026-11-09-run-fed-stage.md`).
//! Nothing in the decoder calls it, with or without the feature.
//!
//! The stage, against today's `mtf::mtf` then `bwt::build`:
//! 1. `mtf::mtf_runs`: the MTF loop (the AVX2 kernel, or the scalar one) storing a 4-byte
//!    record a literal, `p << 8 | c`, and nothing a run;
//! 2. `sort`: the records' stable counting sort by byte, into F order: each run's first F
//!    row `a = C(c) + k` and its first L row p, as `[a, p]`, the row ranges both orders
//!    tile checked on the way (the replacement for `build_one_pass`'s end check);
//! 3. `fill`: one sequential sweep of the rows, cutting them where either order's run ends
//!    (at most 2r − 1 cells); on a cell `T[q + i] = T[q] + 256·i`, written 8 entries at a
//!    time with AVX2, unconditionally, into slack that the next cell overwrites.
//!
//! `takes_runs` sends a block with many symbols for its size to the column's path instead
//! (the fallback), decided from the symbols before the MTF loop runs. `build` is the whole
//! stage with its fallback, and its table is `bwt::build`'s word for word (the tests).

use crate::block::Symbols;
use crate::bwt::{self, TVector};
use crate::{mtf, ErrorKind, Kernels};

/// Whether `build` takes the run records' path for `b` (else the column's): the block is
/// below 2^24 rows, and its symbols number at most an eighth of its bound, so that its
/// records (at most one a symbol) are few against its rows. The bound, not the length, as
/// the length is known only after the MTF loop; for a full block they are about equal
pub fn takes_runs(b: &Symbols) -> bool {
    b.max_len < 1 << 24 && 8 * b.syms.len() <= b.max_len
}

/// The F-order records, kept from block to block: `[a, p]` a run, then `[n, 0]`. Private:
/// only `sort` writes them, and only a `Sorted` it returns reads them
#[derive(Default)]
pub struct Scratch {
    fo: Vec<[u32; 2]>,
}

/// A block's records that `sort` has checked: the L-order records, the F-order ones it
/// made from them in its scratch, and the block's n rows, both orders tiling `[0, n)`.
///
/// Only `sort` makes one (its fields are private), and it holds the records and the
/// scratch borrowed immutably for its life, so neither can change, nor the scratch be
/// sorted again, while it lives. `fill` and `fill_rows` take one, so the premise of their
/// reads, the tilings, is a fact of the type and not the caller's promise.
///
/// None can be made by hand:
/// ```compile_fail,E0451
/// let t = mibz2::run_stage::Sorted { recs: &[], fo: &[], n: 1 };
/// ```
/// nor the scratch forged:
/// ```compile_fail,E0616
/// let mut s = mibz2::run_stage::Scratch::default();
/// s.fo.push([0, 0]);
/// ```
/// nor the records changed under one:
/// ```compile_fail,E0502
/// use mibz2::run_stage::{sort, Scratch};
/// let (mut recs, mut s) = (vec![0u32 << 8 | 7, 1 << 8], Scratch::default());
/// let mut counts = [0u32; 256];
/// counts[7] = 1;
/// let t = sort(&recs, &counts, 1, &mut s).unwrap();
/// recs[1] = 9 << 8;
/// let _ = t.n();
/// ```
/// nor the scratch sorted again under one (the wrong scratch for its records):
/// ```compile_fail,E0499
/// use mibz2::run_stage::{sort, Scratch};
/// let (a, b, mut s) = (vec![7u32, 1 << 8], vec![8u32, 1 << 8], Scratch::default());
/// let (mut ca, mut cb) = ([0u32; 256], [0u32; 256]);
/// (ca[7], cb[8]) = (1, 1);
/// let t = sort(&a, &ca, 1, &mut s).unwrap();
/// let u = sort(&b, &cb, 1, &mut s).unwrap();
/// let _ = (t.n(), u.n());
/// ```
/// while a checked one fills:
/// ```
/// use mibz2::{bwt::TVector, run_stage::{fill, sort, Scratch}, Kernels};
/// let (recs, mut s, mut tt) = (vec![7u32, 1 << 8], Scratch::default(), TVector::new());
/// let mut counts = [0u32; 256];
/// counts[7] = 1;
/// let t = sort(&recs, &counts, 1, &mut s).unwrap();
/// fill(&t, &mut tt, Kernels::detect());
/// assert_eq!(tt.as_slice(), [7]);
/// ```
#[derive(Clone, Copy)]
pub struct Sorted<'a> {
    recs: &'a [u32],
    fo: &'a [[u32; 2]],
    n: u32,
}

impl<'a> Sorted<'a> {
    /// The block's rows
    pub fn n(&self) -> usize {
        self.n as usize
    }
    /// The L-order records, the closing one included
    pub fn recs(&self) -> &'a [u32] {
        self.recs
    }
    /// The F-order records, `[a, p]` a run, the closing `[n, 0]` included
    pub fn fo(&self) -> &'a [[u32; 2]] {
        self.fo
    }
}

/// The records' stable counting sort by byte (`recs` as `mtf::mtf_runs` leaves them, for
/// a block of `n` rows whose bytes `counts` counts) into `s`. Each run's first F row is
/// `a = C(c) + k`, where k sums the lengths of its byte's runs before it. Returns `None`
/// unless both orders tile `[0, n)`: the L runs (their starts rise strictly from 0, and
/// the closing record is n), and the F intervals (each byte's runs' lengths sum to its
/// count, and the counts to n). The `Sorted` it returns holds exactly that.
pub fn sort<'a>(
    recs: &'a [u32],
    counts: &[u32; 256],
    n: usize,
    s: &'a mut Scratch,
) -> Option<Sorted<'a>> {
    if checked_sort(recs, counts, n, &mut s.fo) {
        Some(Sorted {
            recs,
            fo: &s.fo,
            n: n as u32,
        })
    } else {
        // nothing a later reader could mistake for a sorted block's
        s.fo.clear();
        None
    }
}

/// `sort`'s work: true, `fo` the F order, only if both orders tile `[0, n)` (with
/// `n < 2^24`, `recs.len() == fo.len()`, and `fo`'s starts rising strictly from 0)
fn checked_sort(recs: &[u32], counts: &[u32; 256], n: usize, fo_v: &mut Vec<[u32; 2]>) -> bool {
    fo_v.clear();
    if n == 0 {
        // the empty block: its closing record alone, and no counts
        fo_v.push([0, 0]);
        return recs == [0] && counts.iter().all(|&c| c == 0);
    }
    if n >= 1 << 24 || recs.len() < 2 || recs[0] >> 8 != 0 || recs[recs.len() - 1] >> 8 != n as u32
    {
        return false;
    }
    let r = recs.len() - 1;
    let mut slot = [0u32; 256];
    for &x in &recs[..r] {
        slot[(x & 0xff) as usize] += 1;
    }
    let (mut a, mut end) = ([0u32; 256], [0u32; 256]);
    let (mut runs, mut rows) = (0u32, 0u64);
    for c in 0..256 {
        let k = slot[c];
        slot[c] = runs;
        runs += k;
        a[c] = rows as u32;
        rows += counts[c] as u64;
        end[c] = rows as u32;
    }
    if rows != n as u64 {
        return false;
    }
    fo_v.reserve(r + 1);
    let fo = fo_v.as_mut_ptr();
    for w in recs.windows(2) {
        let (x, next) = (w[0], w[1] >> 8);
        let (c, p) = ((x & 0xff) as usize, x >> 8);
        if next <= p {
            return false;
        }
        let j = slot[c] as usize;
        // SAFETY: j < r: byte c's slots are slot[c]'s first value and the next ones, one a
        // run of c, as many as the first loop counted, and the bytes' ranges tile [0, r)
        unsafe { *fo.add(j) = [a[c], p] };
        slot[c] += 1;
        a[c] += next - p;
    }
    if a != end {
        return false;
    }
    // SAFETY: the slots [0, r) were each written once (above), and r + 1 were reserved
    unsafe {
        *fo.add(r) = [n as u32, 0];
        fo_v.set_len(r + 1);
    }
    true
}

/// The table of a checked block (`sort`'s `Sorted`): one sweep, a cell at a time
pub fn fill(t: &Sorted<'_>, tt: &mut TVector, k: Kernels) {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected (`Kernels`' contract, as
            // `mtf::mtf`'s), and `t` is `sort`'s, as `fill_avx2` requires
            return unsafe { fill_avx2(t, &mut tt.0) };
        }
    }
    let _ = k;
    tt.0.clear();
    fill_rows(t, 0, t.n, &mut tt.0);
}

/// The sweep over rows `q0..q1` of a checked block, appended to `out`: each iterator
/// found by binary search (the F order's over `a`, the L order's over p), so a share of
/// the rows needs no count checkpoints. The scalar fill, and the shares' (the tests split
/// a table into shares). Safe code throughout; panics unless `q1 <= t.n()`
pub fn fill_rows(t: &Sorted<'_>, q0: u32, q1: u32, out: &mut Vec<u32>) {
    assert!(q1 <= t.n, "rows {q0}..{q1} of a block of {}", t.n);
    if q0 >= q1 {
        return;
    }
    let (recs, fo) = (t.recs, t.fo);
    let mut f = fo.partition_point(|e| e[0] <= q0) - 1;
    let mut l = recs.partition_point(|&x| x >> 8 <= q0) - 1;
    let mut q = q0;
    while q < q1 {
        let [af, pf] = fo[f];
        let fe = fo[f + 1][0];
        let (x, le) = (recs[l], recs[l + 1] >> 8);
        let end = fe.min(le);
        let base = (x & 0xff) | (pf + (q - af)) << 8;
        let stop = end.min(q1);
        out.extend((0..stop - q).map(|i| base + (i << 8)));
        f += (end == fe) as usize;
        l += (end == le) as usize;
        q = stop;
    }
}

/// The whole table's sweep with each cell written 8 entries at a time: a cell's entries
/// rise by 256, so a vector of them is the base plus 0, 256, .., 1792, and the next 8 add
/// 2048. The last store of a cell runs past its end into the next cell's rows, which that
/// cell writes again (it starts there and stores at least 8); past n, into 8 slots of
/// slack, cut off by the length.
///
/// # Safety
///
/// AVX2 must be available. `t` is a `Sorted`, so `sort` checked its tilings: that is the
/// premise of every raw read below, and no caller can supply a `t` without it. The one
/// caller is `fill`, under `Kernels`' detection.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_avx2(t: &Sorted<'_>, tt: &mut Vec<u32>) {
    use std::arch::x86_64::*;
    let n = t.n;
    tt.clear();
    tt.reserve(n as usize + 8);
    let out = tt.as_mut_ptr();
    let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    let step = _mm256_set1_epi32(2048);
    let (fo, recs) = (t.fo.as_ptr(), t.recs.as_ptr());
    let (mut f, mut l, mut q) = (0usize, 0usize, 0u32);
    while q < n {
        // SAFETY (the loads): `t` is `sort`'s, so both orders tile [0, n): while q < n,
        // run f's F interval [af, fe) and run l's L run [p, le) both hold q, and f and l
        // are below r, so f + 1 and l + 1 index the closing records at most
        let [af, pf] = *fo.add(f);
        let fe = (*fo.add(f + 1))[0];
        let (x, le) = (*recs.add(l), *recs.add(l + 1) >> 8);
        let end = fe.min(le);
        let base = (x & 0xff) | (pf + (q - af)) << 8;
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(base as i32), lanes);
        let mut w = q;
        loop {
            // SAFETY: w < end <= n, so w + 8 <= n + 7, within the n + 8 reserved
            _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
            w += 8;
            if w >= end {
                break;
            }
            v = _mm256_add_epi32(v, step);
        }
        f += (end == fe) as usize;
        l += (end == le) as usize;
        q = end;
    }
    // SAFETY: the cells tile [0, n) (the tilings, above), and each cell's rows were
    // written, the slack past a cell's end written again by the next
    tt.set_len(n as usize);
}

/// The table's write alone, the diagnostic's store-only reference: n entries stored in
/// order, 8 at a time, into `tt`'s buffer (as `fill` stores them, without its cells), then
/// the length left 0, since they are no table. A reference for the stores, not a floor
/// every build obeys: `fill` overstores at its cells' ends (up to `8·Σ⌈lᵢ/8⌉` slots for
/// cells of lengths lᵢ, against these n), and a build overlaps its stores with other work.
/// The buffer's address is passed through `black_box` after the stores, so the compiler
/// must keep them (an optimiser that saw no reader could drop them; no assembly has been
/// inspected to show it doesn't). A build straight after finds the buffer's lines resident
pub fn floor(n: usize, tt: &mut TVector, k: Kernels) {
    tt.0.clear();
    tt.0.reserve(n);
    let spare = &mut tt.0.spare_capacity_mut()[..n];
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            unsafe { floor_avx2(spare) };
            std::hint::black_box(spare.as_ptr());
            return;
        }
    }
    let _ = k;
    for (i, e) in spare.iter_mut().enumerate() {
        e.write(i as u32);
    }
    std::hint::black_box(spare.as_ptr());
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn floor_avx2(spare: &mut [std::mem::MaybeUninit<u32>]) {
    for (i, e) in spare.iter_mut().enumerate() {
        e.write(i as u32);
    }
}

/// The whole stage with its fallback: the block's table from its symbols, into `tt`, as
/// `mtf::mtf` then `bwt::build` make it, word for word; returns the block's length and
/// whether the records' path was taken. `recs`, `s` and `ll` are kept from block to block;
/// `ll` (the column's path) holds the column only when that path was taken.
pub fn build(
    b: &Symbols,
    recs: &mut Vec<u32>,
    s: &mut Scratch,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    tt: &mut TVector,
    k: Kernels,
) -> Result<(usize, bool), ErrorKind> {
    if takes_runs(b) {
        let n = mtf::mtf_runs(b, recs, counts, k)?;
        if let Some(t) = sort(recs, counts, n, s) {
            fill(&t, tt, k);
            return Ok((n, true));
        }
        // a broken tiling is a bug in `mtf_runs` (a debug build stops here): the column's
        // path still gives the right table
        debug_assert!(false, "the run records don't tile the block");
    }
    let n = mtf::mtf(b, ll, counts, k)?;
    bwt::build(ll, counts, tt);
    Ok((n, false))
}

// C31, the direct placement (docs/journal/2026-11-09-run-fed-direct.md): the same records,
// production's same table, and no F order. Each run (c, p, ℓ) of L, taken in L order, owns
// the next ℓ rows of its byte's bucket, `[a, a + ℓ)` with a = C(c) + k; on them Ψ is the
// translation q ↦ p + (q − a). What each row also needs is L[q], the byte of the L run
// that holds row q, and for that each bucket keeps one L cursor, resumed run after run.
// The rows a bucket's runs own rise, so its cursor only moves forward, inside the bucket:
// at most r − 1 advances in all, and O(256) state.

/// Production's table, `T[q] = L[q] | Ψ(q) << 8`, placed straight from the L-order records
/// (`recs` as `mtf::mtf_runs` leaves them, for a block of `n` rows whose bytes `counts`
/// counts), run by run, with no sort and no F-order records. True, and `tt` the table,
/// only if both orders tile `[0, n)`, as `sort` requires; otherwise false, `tt` empty.
///
/// The checks are `sort`'s, made on the way: the L starts rise strictly from 0 to the
/// closing n, the counts sum to n, every write stays below its bucket's end, and at the
/// end every bucket's runs have reached its end exactly, which proves each row below n
/// written by its own run. The vector stores write 8 slots, past a cell's end only
/// where the bucket has those slots and its later cells or runs write them again; within
/// 8 of a bucket's end the rest is stored one slot at a time, so no store reaches a row
/// of another bucket (which may be written already) or past n.
pub fn place(recs: &[u32], counts: &[u32; 256], n: usize, tt: &mut TVector, k: Kernels) -> bool {
    tt.0.clear();
    if n >= 1 << 24 {
        return false;
    }
    tt.0.reserve(n);
    let out = tt.0.as_mut_ptr();
    #[cfg(target_arch = "x86_64")]
    let ok = if k.avx2 {
        // SAFETY: `k.avx2` is set only when AVX2 was detected (`Kernels`' contract, as
        // `fill`'s); `out` has room for n slots, as `place_rows` requires
        unsafe { place_avx2(recs, counts, n, out) }
    } else {
        // SAFETY: as above, for the scalar stores
        unsafe { place_rows::<false>(recs, counts, n, out) }
    };
    #[cfg(not(target_arch = "x86_64"))]
    let ok = {
        let _ = k;
        // SAFETY: `out` has room for n slots
        unsafe { place_rows::<false>(recs, counts, n, out) }
    };
    if ok {
        // SAFETY: true only once both orders tile [0, n), so every slot below n was
        // written by its own run (any overstore into it written again after)
        unsafe { tt.0.set_len(n) };
    }
    ok
}

/// `place_rows` with the 8-wide stores
///
/// # Safety
///
/// AVX2 must be available, and `out` valid for writes of n slots
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn place_avx2(recs: &[u32], counts: &[u32; 256], n: usize, out: *mut u32) -> bool {
    place_rows::<true>(recs, counts, n, out)
}

/// The placement, `V` the 8-wide stores (else one slot a store): `place`'s checks, and
/// its writes, every one to a slot below n.
///
/// The reads are safe code, and need no premise: the records are checked as the loop
/// meets them, while the cursors read ahead of that, so on records that fail a check the
/// cursors may find wrong bytes, and the table they make is discarded. Every write is
/// below its bucket's end (checked before the run's first), hence below n; and each cell
/// ends past its start, so the loop ends: the cursor stops at a record past q, and the
/// closing record is n, past every q.
///
/// # Safety
///
/// `out` valid for writes of n slots; with `V`, AVX2 available (only `place_avx2`
/// instantiates it)
#[inline(always)]
unsafe fn place_rows<const V: bool>(
    recs: &[u32],
    counts: &[u32; 256],
    n: usize,
    out: *mut u32,
) -> bool {
    if n == 0 {
        // the empty block: its closing record alone, and no counts
        return recs == [0] && counts.iter().all(|&c| c == 0);
    }
    if n >= 1 << 24 || recs.len() < 2 || recs[0] >> 8 != 0 || recs[recs.len() - 1] >> 8 != n as u32
    {
        return false;
    }
    let r = recs.len() - 1;
    // the prefixes: each bucket's next row a[c] and its end
    let (mut a, mut end) = ([0u32; 256], [0u32; 256]);
    let mut rows = 0u64;
    for c in 0..256 {
        a[c] = rows as u32;
        rows += counts[c] as u64;
        end[c] = rows as u32;
    }
    if rows != n as u64 {
        return false;
    }
    // the cursors, seeded once a bucket: the L run holding its first row (a search
    // narrowed from the last bucket's, since the buckets' starts rise). Kept below r, so
    // `cur + 1` is a record; on unchecked records any index, as above
    let mut cur = [0u32; 256];
    let mut lo = 0usize;
    for c in 0..256 {
        if a[c] < end[c] {
            let q = a[c];
            lo += recs[lo..].partition_point(|&x| x >> 8 <= q);
            lo = lo.saturating_sub(1).min(r - 1);
            cur[c] = lo as u32;
        }
    }
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    for j in 0..r {
        let (x, next) = (recs[j], recs[j + 1] >> 8);
        let (c, p) = ((x & 0xff) as usize, x >> 8);
        if next <= p {
            return false;
        }
        // the run's rows in its bucket: [q0, e), below the bucket's end
        let q0 = a[c];
        let e = q0 + (next - p);
        let bend = end[c];
        if e > bend {
            return false;
        }
        let mut l = cur[c] as usize;
        let mut q = q0;
        loop {
            // the cursor resumed: the L run holding row q (it stops at the closing record
            // at the latest, which is n > q)
            while recs[l + 1] >> 8 <= q {
                l += 1;
            }
            let ce = e.min(recs[l + 1] >> 8);
            // Ψ(q) = p + (q − q0) < next < 2^24, so the entry is exact
            let base = (recs[l] & 0xff) | (p + (q - q0)) << 8;
            let mut w = q;
            #[cfg(target_arch = "x86_64")]
            if V {
                // (only `place_avx2`'s instance reaches these: AVX2 is there)
                let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
                let step = _mm256_set1_epi32(2048);
                let mut v = _mm256_add_epi32(_mm256_set1_epi32(base as i32), lanes);
                while w < ce && w + 8 <= bend {
                    // SAFETY: w + 8 <= bend <= n: the slots are the bucket's, written
                    // again by its later cells and runs past ce
                    _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
                    w += 8;
                    v = _mm256_add_epi32(v, step);
                }
            }
            // the rest one slot at a time (with `V`, only within 8 of the bucket's end)
            let mut val = base.wrapping_add(w.wrapping_sub(q) << 8);
            while w < ce {
                // SAFETY: w < ce <= e <= bend <= n
                *out.add(w as usize) = val;
                val = val.wrapping_add(256);
                w += 1;
            }
            q = ce;
            if q == e {
                break;
            }
        }
        cur[c] = l as u32;
        a[c] = e;
    }
    // implied by the checks above (the runs' lengths sum to n, each bucket's stay within
    // it, and the buckets' sizes sum to n), and kept: it is the tiling, stated once
    a == end
}

/// The stage as `build` makes it, with the direct placement in place of the sort and the
/// fill (arm D of the diagnostic): `takes_runs`, then `mtf::mtf_runs` and `place`, or the
/// column's path, the fallback, exactly `build`'s. The same table, word for word, and the
/// same result, error kind included
pub fn build_direct(
    b: &Symbols,
    recs: &mut Vec<u32>,
    ll: &mut Vec<u8>,
    counts: &mut [u32; 256],
    tt: &mut TVector,
    k: Kernels,
) -> Result<(usize, bool), ErrorKind> {
    if takes_runs(b) {
        let n = mtf::mtf_runs(b, recs, counts, k)?;
        if place(recs, counts, n, tt, k) {
            return Ok((n, true));
        }
        // as `build`: a broken tiling is a bug in `mtf_runs`, and the column still serves
        debug_assert!(false, "the run records don't tile the block");
    }
    let n = mtf::mtf(b, ll, counts, k)?;
    bwt::build(ll, counts, tt);
    Ok((n, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_fed;

    /// A column's maximal runs as `mtf_runs` records them
    fn recs_of(ll: &[u8]) -> Vec<u32> {
        let mut v: Vec<u32> = run_fed::runs_of(ll, 0)
            .iter()
            .map(|r| r.p << 8 | r.c as u32)
            .collect();
        v.push((ll.len() as u32) << 8);
        v
    }

    fn kernels() -> Vec<Kernels> {
        let mut v = vec![Kernels::scalar()];
        if Kernels::detect() != Kernels::scalar() {
            v.push(Kernels::detect());
        }
        v
    }

    /// The table from a column's records, every way (each kernel's fill, 1-5 shares), the
    /// production table word for word; and every tiling broken by one row refused
    fn check(ll: &[u8]) {
        let counts = run_fed::counts_of(ll);
        let want = run_fed::production(ll, &counts);
        let recs = recs_of(ll);
        let n = ll.len();
        let mut s = Scratch::default();
        let t = sort(&recs, &counts, n, &mut s).expect("sort");
        assert_eq!(
            (t.n(), t.recs().len(), t.fo().len()),
            (n, recs.len(), recs.len())
        );
        let mut tt = TVector(vec![9; 3]);
        for k in kernels() {
            fill(&t, &mut tt, k);
            assert!(tt.0 == want, "n {n}, {k:?}: the table");
        }
        direct(ll, &recs, &counts, &want);
        for shares in 1..=5u32 {
            let mut out = Vec::new();
            for j in 0..shares {
                let (q0, q1) = (n as u32 * j / shares, n as u32 * (j + 1) / shares);
                fill_rows(&t, q0, q1, &mut out);
            }
            assert!(out == want, "n {n}, {shares} shares");
        }
        // a count off by one, a run's start moved, the closing record moved, the wrong n
        // either way: refused, and the scratch left holding nothing
        if n > 0 {
            let mut c = counts;
            let b = ll[0] as usize;
            c[b] -= 1;
            c[(b + 1) % 256] += 1;
            refused(&recs, &c, n, &mut s, "a wrong count");
            let mut r = recs.clone();
            *r.last_mut().unwrap() += 256;
            refused(&r, &counts, n, &mut s, "a wrong end");
            if recs.len() > 2 {
                let mut r = recs.clone();
                r[1] += 256;
                refused(&r, &counts, n, &mut s, "a run moved");
            }
        }
        refused(&recs, &counts, n + 1, &mut s, "n one more");
        if n > 0 {
            refused(&recs, &counts, n - 1, &mut s, "n one less");
        }
    }

    /// The direct placement (C31) of a column's records, every kernel, the production
    /// table word for word, into a dirty table; and each broken tiling refused, the table
    /// left empty: a count off by one, a count moved to another byte, n off by one either
    /// way, the closing record moved, a run's start moved, two records swapped
    fn direct(ll: &[u8], recs: &[u32], counts: &[u32; 256], want: &[u32]) {
        let n = ll.len();
        for k in kernels() {
            let mut tt = TVector(vec![9; 3]);
            assert!(place(recs, counts, n, &mut tt, k), "n {n}, {k:?}: placed");
            assert!(tt.0 == want, "n {n}, {k:?}: the direct table");
        }
        let no = |recs: &[u32], counts: &[u32; 256], n: usize, what: &str| {
            for k in kernels() {
                let mut tt = TVector(vec![9; 3]);
                assert!(!place(recs, counts, n, &mut tt, k), "n {n}, {k:?}: {what}");
                assert!(tt.0.is_empty(), "n {n}, {k:?}: {what}: the table");
            }
        };
        no(recs, counts, n + 1, "n one more");
        if n > 0 {
            no(recs, counts, n - 1, "n one less");
            let b = ll[0] as usize;
            let mut c = *counts;
            c[b] -= 1;
            no(recs, &c, n, "a count one less");
            c[b] += 2;
            no(recs, &c, n, "a count one more");
            let mut c = *counts;
            c[b] -= 1;
            c[(b + 1) % 256] += 1;
            no(recs, &c, n, "a count moved to another byte");
            let mut r = recs.to_vec();
            *r.last_mut().unwrap() += 256;
            no(&r, counts, n, "the closing record moved");
            if recs.len() > 2 {
                let mut r = recs.to_vec();
                r[1] += 256;
                no(&r, counts, n, "a run's start moved");
                let mut r = recs.to_vec();
                r.swap(0, 1);
                no(&r, counts, n, "two records swapped");
            }
        }
    }

    /// The direct table of `ll` against production's, with its refusals
    fn check_direct(ll: &[u8]) {
        let counts = run_fed::counts_of(ll);
        direct(ll, &recs_of(ll), &counts, &run_fed::production(ll, &counts));
    }

    /// C31's cases by name: the empty and tiny blocks; one symbol; every one of the 256;
    /// r = n (no run longer than a row); runs that cross bucket boundaries; and a bucket's
    /// last cell within 8 of its end where the next bucket is already written (the 8-wide
    /// store must not reach it: L = c^10 b^10 writes c's rows first, then b's)
    #[test]
    fn direct_cases() {
        check_direct(&[]);
        for ll in [&b"a"[..], b"ab", b"ba", b"aa", b"abba", b"bba", b"aab"] {
            check_direct(ll);
        }
        // one symbol: one run, or a byte of each length to 40
        for len in 1..=40 {
            check_direct(&vec![7u8; len]);
        }
        check_direct(&vec![255u8; 70_000]);
        // all 256: each once (r = n), in order and reversed; each as a run of 1..=20
        let all: Vec<u8> = (0..=255).collect();
        check_direct(&all);
        check_direct(&all.iter().rev().copied().collect::<Vec<_>>());
        let mut runs = Vec::new();
        for c in 0..=255u8 {
            runs.extend(std::iter::repeat_n(c, 1 + c as usize % 20));
        }
        check_direct(&runs);
        // r = n: alternating bytes, and a cycle of three
        check_direct(&b"ab".repeat(500));
        check_direct(&b"cab".repeat(333));
        // runs crossing bucket boundaries: a = rows 0..3, b = 3..8, L's b run is 1..6
        check_direct(b"abbbbbaa");
        check_direct(&[vec![1u8; 9], vec![0; 30], vec![1; 31], vec![2; 3]].concat());
        // the protected tail: b's bucket [0, 10) placed after c's [10, 20)
        for len in 1..=24 {
            check_direct(&[vec![2u8; len], vec![1; len]].concat());
            check_direct(&[vec![2u8; len], vec![1; len], vec![0; 3], vec![1; 5]].concat());
        }
    }

    /// The direct placement's states it must never accept, each refused with no table,
    /// natively and under Miri (scalar and AVX2): no records, for n 0 and 1; the closing
    /// record alone; the empty block's records for one row; another block's n; records out
    /// of order, two runs at one row, a first run past row 0, an uncounted byte; counts
    /// that sum past n, or wrap; and n at 2^24
    #[test]
    fn miri_direct_unchecked_states_refused() {
        expect_avx2();
        let zero = [0u32; 256];
        let no = |recs: &[u32], counts: &[u32; 256], n: usize, what: &str| {
            for k in kernels() {
                let mut tt = TVector(vec![9; 3]);
                assert!(!place(recs, counts, n, &mut tt, k), "{k:?}: {what}");
                assert!(tt.0.is_empty(), "{k:?}: {what}: the table");
            }
        };
        no(&[], &zero, 0, "no records, the empty block");
        no(&[], &zero, 1, "no records, one row");
        let mut one = zero;
        one[7] = 1;
        no(&[], &one, 1, "no records, one row counted");
        no(&[1 << 8], &one, 1, "only the closing record");
        no(&[0], &zero, 1, "the empty block's records, one row");
        no(&[7, 1 << 8], &one, 1 << 24, "n at 2^24");
        for k in kernels() {
            let mut tt = TVector(vec![1; 4]);
            assert!(place(&[0], &zero, 0, &mut tt, k), "{k:?}: the empty block");
            assert!(tt.0.is_empty(), "{k:?}: the empty table");
        }
        let ll = b"abba";
        let (counts, recs) = (run_fed::counts_of(ll), recs_of(ll));
        for n in [0, 1, 3, 5] {
            no(&recs, &counts, n, "another block's n");
        }
        let mut swapped = recs.clone();
        swapped.swap(1, 2);
        no(&swapped, &counts, 4, "records out of order");
        let mut late = recs.clone();
        late[0] += 1 << 8;
        no(&late, &counts, 4, "the first run past row 0");
        let mut dup = recs.clone();
        dup[1] = dup[0];
        no(&dup, &counts, 4, "two runs at one row");
        let mut byte = recs.clone();
        byte[1] = byte[1] & !0xff | b'c' as u32;
        no(&byte, &counts, 4, "a run's byte not counted");
        let mut wrap = counts;
        wrap[0] = u32::MAX;
        wrap[1] = 5;
        no(&recs, &wrap, 4, "counts that wrap a u32");
        // a run longer than its byte's bucket, its byte's count short by its overflow
        let mut long = zero;
        long[b'a' as usize] = 1;
        long[b'b' as usize] = 3;
        no(&recs, &long, 4, "a run past its bucket's end");
        // and the tiny tables it does accept, by both kernels, under Miri
        for ll in [&b"a"[..], b"ab", b"abba", b"bba"] {
            check_direct(ll);
        }
        check_direct(&[vec![2u8; 10], vec![1; 10]].concat());
        check_direct(&[vec![0u8; 17], vec![2; 3], vec![0; 16]].concat());
    }

    /// With `MIBZ2_EXPECT_AVX2` set (`miri.yml`'s AVX2 step), the AVX2 fill must be among
    /// the kernels tested, so a pass there means its reads and stores were checked
    fn expect_avx2() {
        if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
            assert!(Kernels::detect().avx2, "AVX2 expected, and not detected");
        }
    }

    /// `sort` refuses the records, and leaves its scratch empty
    fn refused(recs: &[u32], counts: &[u32; 256], n: usize, s: &mut Scratch, what: &str) {
        assert!(sort(recs, counts, n, s).is_none(), "n {n}: {what}");
        assert!(s.fo.is_empty(), "n {n}: {what}: the scratch");
    }

    /// The states `fill` must never read unchecked, each refused by `sort`, so no
    /// `Sorted` exists for them (the doc tests show none can be made by hand): no records
    /// at all, for an empty block and a one-row one (the fill that read empty F-order
    /// storage); the records of one block with another's n; records out of order; records
    /// that start past row 0; a scratch sorted for one block, then refused another
    /// (nothing of the first left to read). And `fill_rows` past the block's rows panics
    #[test]
    fn miri_unchecked_states_refused() {
        expect_avx2();
        let zero = [0u32; 256];
        let mut s = Scratch::default();
        refused(&[], &zero, 0, &mut s, "no records, the empty block");
        refused(&[], &zero, 1, &mut s, "no records, one row");
        let mut one = zero;
        one[7] = 1;
        refused(&[], &one, 1, &mut s, "no records, one row counted");
        refused(&[1 << 8], &one, 1, &mut s, "only the closing record");
        refused(&[0], &zero, 1, &mut s, "the empty block's records, one row");
        // the empty block itself is a block: no rows, no table
        let t = sort(&[0], &zero, 0, &mut s).expect("the empty block");
        let mut tt = TVector(vec![1; 4]);
        for k in kernels() {
            fill(&t, &mut tt, k);
            assert!(tt.0.is_empty(), "{k:?}: the empty table");
        }
        // abba: runs a@0, b@1, a@3, closed by 4
        let ll = b"abba";
        let (counts, recs) = (run_fed::counts_of(ll), recs_of(ll));
        assert!(sort(&recs, &counts, 4, &mut s).is_some());
        for n in [0, 1, 3, 5, 1 << 24] {
            refused(&recs, &counts, n, &mut s, "another block's n");
        }
        let mut swapped = recs.clone();
        swapped.swap(1, 2);
        refused(&swapped, &counts, 4, &mut s, "records out of order");
        let mut late = recs.clone();
        late[0] += 1 << 8;
        refused(&late, &counts, 4, &mut s, "the first run past row 0");
        let mut dup = recs.clone();
        dup[1] = dup[0];
        refused(&dup, &counts, 4, &mut s, "two runs at one row");
        let mut byte = recs.clone();
        byte[1] = byte[1] & !0xff | b'c' as u32;
        refused(&byte, &counts, 4, &mut s, "a run's byte not counted");
        // a scratch sorted for one block: refused for the next, it holds nothing of either
        assert!(sort(&recs, &counts, 4, &mut s).is_some());
        refused(&recs, &one, 4, &mut s, "the next block's counts");
        let t = sort(&recs, &counts, 4, &mut s).unwrap();
        let past = std::panic::catch_unwind(|| fill_rows(&t, 2, 5, &mut Vec::new()));
        assert!(past.is_err(), "fill_rows past the block's rows");
    }

    /// Every ternary column of 1 to `max` rows (and the empty one)
    fn ternary(max: usize) {
        check(&[]);
        for len in 1..=max {
            let mut s = vec![0u8; len];
            loop {
                check(&s);
                let mut i = 0;
                while i < len && s[i] == 2 {
                    s[i] = 0;
                    i += 1;
                }
                if i == len {
                    break;
                }
                s[i] += 1;
            }
        }
    }

    #[test]
    fn tables_from_records_small() {
        ternary(8);
    }

    /// `sort`'s and `floor`'s unsafe writes under Miri, and the AVX2 fill's reads and
    /// stores where Miri runs with AVX2 (`miri.yml`'s second step; the scalar fill is safe
    /// code; the AVX2 emitter is checked by the differential tests only), on columns small
    /// enough to interpret, runs of 16 and more included
    #[test]
    fn miri_sort_and_floor() {
        expect_avx2();
        ternary(4);
        for ll in [
            vec![1u8; 20],
            [vec![0u8; 17], vec![2; 3], vec![0; 16]].concat(),
        ] {
            check(&ll);
        }
        let mut tt = TVector(vec![5; 2]);
        floor(37, &mut tt, Kernels::scalar());
        assert!(tt.0.is_empty() && tt.0.capacity() >= 37);
    }

    #[test]
    fn tables_from_records_random() {
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut rnd = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for &len in &[7usize, 8, 9, 15, 16, 17, 31, 900, 4096, 70_000] {
            for sigma in [1u64, 2, 5, 40, 256] {
                let mut ll = Vec::new();
                while ll.len() < len {
                    let c = (rnd() % sigma) as u8;
                    let run = [1, 1, 2, 3, 7, 8, 9, 16, 17, 40][(rnd() % 10) as usize];
                    ll.extend(std::iter::repeat_n(c, run));
                }
                ll.truncate(len);
                check(&ll);
                check(&(0..len).map(|_| (rnd() % sigma) as u8).collect::<Vec<_>>());
            }
        }
    }
}
