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
//! The tests check the two builders word for word (and their LF rows and start row),
//! exhaustively on small columns and on random ones, and the walk against `bwt::walk`.

use crate::bwt::{
    prefetch, random_rows, read_u64, InverseBwt, SegScratch, Segmented, FLAGGED_LANES,
    FLAGGED_SEGMENTS, MIN_SEGMENTED, PF_L1, PF_OFF, START,
};

/// The pair table: row r's entry is its last-column byte L[r] | ψ²(r) << 8, and the byte
/// after L[r] in the text, L[ψ(r)], is F(r), row r's first byte, found from `first` and
/// the buckets' starts `f`. So a step from r gives two bytes and the row two on. Built
/// only by `build_pairs` or `build_pairs_runs`, so every link is below its length; bits 30
/// and 31 are the walk's flags (links are below 2^20).
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
    /// The table's words, as a builder left them (the walk adds its flags)
    pub fn words(&self) -> &[u32] {
        &self.p
    }

    /// The first-byte lookup's rows a byte (`1 << fshift`) and its length
    pub fn first_lookup(&self) -> (u32, usize) {
        (self.fshift, self.first.len())
    }

    /// Row r's first byte: its F bucket, from the slice of rows r is in
    #[inline(always)]
    fn first_byte(&self, r: u32) -> u8 {
        // SAFETY: r < n, so r >> fshift < first.len(), and the bucket ends rise to f[256] = n
        unsafe {
            let mut c = *self.first.get_unchecked((r >> self.fshift) as usize) as usize;
            while *self.f.get_unchecked(c + 1) <= r {
                c += 1;
            }
            c as u8
        }
    }

    /// `first_byte` and the correction loop's iterations (the census's)
    fn first_byte_counted(&self, r: u32) -> (u8, u32) {
        let mut c = self.first[(r >> self.fshift) as usize] as usize;
        let mut k = 0;
        while self.f[c + 1] <= r {
            c += 1;
            k += 1;
        }
        (c as u8, k)
    }
}

/// ψ² of the last column `ll` (whose bytes `counts` counts), as libsais builds it (#34's
/// `build_pairs`, unchanged).
/// - A step i of the counting sort has p = LF(i) (its byte c's next slot), and i's row
///   two on from the row LF(p) = LF²(i), the next slot of the bigram (L[p], c): the rows
///   sorted by their first two bytes, in i's order within each.
/// - The bigram counts come from a byte histogram of each F bucket's slice of `ll` (row p
///   in bucket c is bigram (L[p], c)): sequential, not a pass of LF lookups.
/// - 16 equal bytes c at i whose LF rows' 16 bytes are equal too (the run lemma, twice)
///   go in with one 16-wide store, libsais's RUN_LANES, as in `build`.
///
/// `rows` (sorted) are the rows whose LF is wanted: each one's is pushed to `lf`, in order.
#[allow(clippy::needless_range_loop)]
pub fn build_pairs(
    ll: &[u8],
    counts: &[u32; 256],
    t: &mut PairTable,
    rows: &[u32],
    lf: &mut Vec<u32>,
) {
    let n = ll.len();
    t.p.clear();
    t.p.extend(ll.iter().map(|&c| c as u32));
    let mut cum = 0u32;
    for c in 0..256 {
        t.f[c] = cum;
        cum += counts[c];
    }
    t.f[256] = cum;
    t.fshift = 0;
    while (n >> t.fshift) >= 1 << 15 {
        t.fshift += 1;
    }
    t.first.clear();
    let mut c = 0;
    for j in 0..((n - 1) >> t.fshift) + 1 {
        let r = (j << t.fshift) as u32;
        while t.f[c + 1] <= r {
            c += 1;
        }
        t.first.push(c as u8);
    }
    // The bigram counts, then each bigram's first slot (first byte major)
    t.b2.clear();
    t.b2.resize(1 << 16, 0);
    let mut h = [[0u32; 256]; 4];
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
            t.b2[d << 8 | c] = h[0][d] + h[1][d] + h[2][d] + h[3][d];
            (h[0][d], h[1][d], h[2][d], h[3][d]) = (0, 0, 0, 0);
        }
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
    lf.clear();
    let mut next = rows.first().map_or(usize::MAX, |&r| r as usize);
    let mut ri = 0;
    let mut i = 0;
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
            pair_steps(ll, p, b2, &mut b1, i..e);
        } else {
            for i in i..e {
                if i == next {
                    lf.push(b1[ll[i] as usize]);
                    ri += 1;
                    next = rows.get(ri).map_or(usize::MAX, |&r| r as usize);
                }
                pair_steps(ll, p, b2, &mut b1, i..i + 1);
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
        pair_steps(ll, p, b2, &mut b1, i..i + 1);
    }
}

/// `build_pairs`'s one-row steps, over the rows `rows`
#[inline(always)]
fn pair_steps(
    ll: &[u8],
    p: &mut [u32],
    b2: &mut [u32],
    b1: &mut [u32; 256],
    rows: std::ops::Range<usize>,
) {
    for i in rows {
        // SAFETY: the counts are ll's own, so q < n, and the bigram slots x < n
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

/// The row ψ(orig_ptr) the text starts at: the row whose LF is `orig_ptr`, the
/// (orig_ptr − f[c])-th c in `ll`, for c = F(orig_ptr) (#34's, unchanged)
pub fn pair_start(ll: &[u8], t: &PairTable, orig_ptr: u32) -> u32 {
    let c = t.first_byte(orig_ptr);
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

/// `build_pairs_runs`'s buffers, kept from block to block
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
    /// the last build's runs and cells
    pub runs: usize,
    pub cells: usize,
}

/// The pair table `build_pairs` builds, word for word, and the same LF rows for `rows`
/// (sorted) in `lf`, from the runs of `ll`:
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
/// Returns the text's first row ψ(orig_ptr) (`pair_start`'s), or None, with `t` unusable,
/// if `counts` are not `ll`'s own or the block is too long for the walk's flags (n > 2^22):
/// the caller then builds with `build_pairs`. `orig_ptr` is below n, which is at least 1.
#[allow(clippy::needless_range_loop)]
pub fn build_pairs_runs(
    ll: &[u8],
    counts: &[u32; 256],
    t: &mut PairTable,
    s: &mut RunScratch,
    rows: &[u32],
    lf: &mut Vec<u32>,
    orig_ptr: u32,
) -> Option<u32> {
    let n = ll.len();
    t.p.clear();
    if n == 0 || n > 1 << 22 || orig_ptr as usize >= n {
        return None;
    }
    // the buckets' starts, and the first-byte lookup (as `build_pairs` makes them)
    let mut cum = 0u32;
    for c in 0..256 {
        t.f[c] = cum;
        cum += counts[c];
    }
    t.f[256] = cum;
    if cum as usize != n {
        return None;
    }
    t.fshift = 0;
    while (n >> t.fshift) >= 1 << 15 {
        t.fshift += 1;
    }
    t.first.clear();
    let mut c = 0;
    for j in 0..((n - 1) >> t.fshift) + 1 {
        let r = (j << t.fshift) as u32;
        while t.f[c + 1] <= r {
            c += 1;
        }
        t.first.push(c as u8);
    }

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
    // a run starts at `pos` (the one before it, if any, closes): its record written
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
    if seen != *counts {
        return None;
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

    // The fill, in F order, so in rising q: F interval k is run i = fi[k]'s I_i, rows
    // [fa[k], fa[k + 1]); its image J_i starts at x = fp[k], in F interval j; on each cell
    // [x, end) of J_i ∩ I_j, ψ(q + t) = x + t and ψ(x + t) = fp[j] + x − fa[j] + t. The
    // words go in 8 at a time, past the cell's end into rows the next cells (in rising q)
    // overwrite, or exactly near the block's end.
    t.p.reserve(n + 8);
    let d = &mut t.p.spare_capacity_mut()[..n + 8];
    #[cfg(target_arch = "x86_64")]
    let cells = if std::is_x86_feature_detected!("avx2") {
        // SAFETY: AVX2 is present
        unsafe { fill_avx2(ll, d, fo, &s.fi, &s.js) }
    } else {
        // SAFETY: without AVX2, no precondition
        unsafe { fill::<false>(ll, d, fo, &s.fi, &s.js) }
    };
    #[cfg(not(target_arch = "x86_64"))]
    // SAFETY: without AVX2, no precondition
    let cells = unsafe { fill::<false>(ll, d, fo, &s.fi, &s.js) };
    // SAFETY: every slot below n was last written by its own cell. The F intervals tile
    // [0, n) in F order (the runs of each byte c, in L order, have a = C(c) + the lengths
    // of the runs of c before them; their lengths sum to counts[c], checked, and the counts
    // sum to n, checked); each interval's cells, as x sweeps its image, tile it in rising
    // q; so the cells tile [0, n) in the order they are written. A wide write goes past its
    // cell only upward, into rows of later cells, each of which then writes its own rows.
    unsafe { t.p.set_len(n) };
    s.runs = r;
    s.cells = cells;
    // The LF rows of `rows`: row v in L run i has LF(v) = a_i + v − p_i
    lf.clear();
    let mut i = 0;
    for &v in rows {
        while s.p[i + 1] <= v {
            i += 1;
        }
        lf.push(s.a[i] + (v - s.p[i]));
    }
    // ψ(orig_ptr): orig_ptr in F interval j
    let j = s.fo.partition_point(|e| e[0] <= orig_ptr) - 1;
    Some(s.fo[j][1] + (orig_ptr - s.fo[j][0]))
}

/// Whether a builder's words are ψ² by definition: row r's word is `L[r] | ψ(ψ(r)) << 8`,
/// ψ read from production's one-hop table (`bwt::build`'s, built here); the slow check
pub fn is_psi2(t: &PairTable, ll: &[u8], counts: &[u32; 256]) -> bool {
    let mut tt = crate::bwt::TVector::new();
    crate::bwt::build(ll, counts, &mut tt);
    let psi = |r: usize| (tt.0[r] >> 8) as usize;
    t.p.len() == ll.len()
        && t.p
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
) -> usize {
    // SAFETY: AVX2 is present (the caller's)
    unsafe { fill::<true>(ll, d, fo, fi, js) }
}

/// `build_pairs_runs`'s fill, in F order, so in rising q: F interval k is run i = fi[k]'s
/// I_i, rows [fo[k][0], fo[k + 1][0]); its image J_i starts at x = fo[k][1], in F interval
/// js[i]; on each cell [x, end) of J_i ∩ I_j, ψ(q + t) = x + t and ψ(x + t) = fo[j][1] + x
/// − fo[j][0] + t. The words go in 8 at a time, past the cell's end into rows the next
/// cells (in rising q) overwrite, or exactly near the block's end. `d` holds n + 8 words
/// (n = ll.len()); `fo` holds the r F intervals and [n, 0]; `js` and `fi` are the merge's
/// and the sort's. Returns the cells. Its memory safety doesn't rest on those being right
/// (each index is checked, or bounded by a check); a wrong input panics or writes wrong
/// words in bounds.
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
) -> usize {
    let n = ll.len();
    assert!(d.len() >= n + 8 && !fo.is_empty() && fo[fo.len() - 1][0] as usize == n);
    let r = fo.len() - 1;
    let dp = d.as_mut_ptr() as *mut u32;
    let lp = ll.as_ptr();
    let mut cells = 0;
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
            } else {
                for (o, (e, &l)) in d[q..q + len].iter_mut().zip(&ll[q..q + len]).enumerate() {
                    e.write(l as u32 | (base + ((o as u32) << 8)));
                }
            }
            cells += 1;
            q += len;
            if end >= xe {
                break;
            }
            j += 1;
            assert!(j < r);
            x = end;
        }
    }
    cells
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
    /// #34's `build_pairs` and `pair_start`
    #[default]
    Rows,
    /// `build_pairs_runs` (`build_pairs` should it decline)
    Runs,
}

/// The pair walk's buffers, kept from block to block
#[derive(Default)]
pub struct PairScratch {
    pub t: PairTable,
    pub seg: SegScratch,
    lfs: Vec<(u32, u32)>,
    lf_rows: Vec<u32>,
    pub runs: RunScratch,
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
/// flags. The last column's last read. Returns the text's first row.
pub fn setup(
    build: Build,
    x: &mut PairScratch,
    ll: &[u8],
    counts: &[u32; 256],
    orig_ptr: u32,
    segments: usize,
) -> u32 {
    let n = ll.len();
    let PairScratch {
        t,
        seg: s,
        lfs,
        lf_rows,
        runs,
    } = x;
    random_rows(n, segments - 1, &mut s.starts);
    s.starts.sort_unstable();
    s.starts.dedup();
    let built = match build {
        Build::Runs => build_pairs_runs(ll, counts, t, runs, &s.starts, lf_rows, orig_ptr),
        Build::Rows => None,
    };
    let r0 = match built {
        Some(r0) => r0,
        None => {
            build_pairs(ll, counts, t, &s.starts, lf_rows);
            pair_start(ll, t, orig_ptr)
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
    lfs.clear();
    lfs.extend(st.iter().enumerate().map(|(k, e)| (e.1, k as u32)));
    lfs.sort_unstable();
    for &(r, l) in &st {
        if l != r {
            t.p[l as usize] |= START | PAIR_LF;
        }
        t.p[r as usize] |= START;
    }
    r0
}

/// The pair walk's second half, #34's loop unchanged: `L` lanes over the flagged table
/// `setup` left, two bytes a step, then the segments joined from the text's first row
/// `r0` into `out` (cleared). Reads only `x`, never the last column.
pub fn walk<const L: usize, const PF: bool>(x: &mut PairScratch, r0: u32, out: &mut Vec<u8>) {
    let PairScratch { t, seg: s, lfs, .. } = x;
    let n = t.p.len();
    out.clear();
    let ns = s.starts.len();
    let cap = 2 * n / ns + 64;
    s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
    for b in &mut s.bufs[..ns] {
        b.clear();
        b.reserve(cap);
    }
    s.next.clear();
    s.next.resize(ns, usize::MAX);
    let starts = &s.starts;
    let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
    let lfs = &*lfs;
    let seg_after = |row: u32| match lfs.binary_search_by_key(&row, |e| e.0) {
        Ok(k) => lfs[k].1 as usize,
        Err(_) => usize::MAX,
    };
    let tp = t.p.as_ptr();
    let tt = &*t;
    let bufs = &mut s.bufs;
    let next = &mut s.next;
    let pf = |row: u32| {
        if PF {
            prefetch::<PF_L1>(tp, row)
        } else {
            prefetch::<PF_OFF>(tp, row)
        }
    };

    // A lane takes segment `seg`: its first step (its start's own, flagged, entry) here
    let take = |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
        let b = &mut bufs[seg];
        let base = b.as_mut_ptr();
        let r = starts[seg];
        // SAFETY: the starts are rows below n; the buffer has `cap` > 2 spare bytes
        let q = unsafe { *tp.add(r as usize) } & PAIR_LINK;
        unsafe {
            *base = q as u8;
            *base.add(1) = tt.first_byte(r);
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
    let finish = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
        if q & PAIR_LF != 0 {
            // SAFETY: a step leaves at least 2 spare bytes
            unsafe {
                *l.p = q as u8;
                l.p = l.p.add(1);
            }
            next[l.seg] = seg_after(l.row);
        } else {
            next[l.seg] = seg_of(l.row);
        }
        let b = &mut bufs[l.seg];
        // SAFETY: p is within b's allocation, every byte below it written
        unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
    };
    // Fewer than 2 spare bytes: the buffer grows (its written bytes kept)
    let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
        let b = &mut bufs[l.seg];
        let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
        unsafe { b.set_len(len) };
        b.reserve(b.capacity());
        l.p = unsafe { b.as_mut_ptr().add(len) };
        l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
    };
    // One step: two bytes, and the row two on
    let step = |l: &mut Lane, q: u32, bufs: &mut Vec<Vec<u8>>| {
        let r = l.row;
        l.row = q >> 8;
        pf(l.row);
        // SAFETY: at least 2 spare bytes, within the buffer's allocation
        unsafe {
            (l.p as *mut u16).write_unaligned(u16::from_le_bytes([q as u8, tt.first_byte(r)]));
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
                // SAFETY: rows are below n (PairTable)
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
}

/// The largest sampled run density (runs a row, `run_density`) at which `Pairs` on the
/// run builder takes a block; above it the block goes to production's `Segmented`. Fixed
/// before any timing (docs/journal/2026-11-09-pair-from-runs.md): Wikidata's blocks are
/// at about 0.07, the synthetic text at 0.24, records and random near 1.
pub const ADMIT_DENSITY: f64 = 0.15;

/// An estimate of a column's runs a row: the run starts in one 8-byte word of every 16
/// (a 1/16 sample, spread over the column), counted 8 bytes at a time
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
/// `Segmented` instead (counted in `declined`).
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
        let r0 = setup(
            self.build,
            &mut self.x,
            ll,
            counts,
            orig_ptr,
            FLAGGED_SEGMENTS,
        );
        walk::<FLAGGED_LANES, true>(&mut self.x, r0, out);
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
        let r0 = setup(
            self.build,
            &mut self.x,
            ll,
            counts,
            orig_ptr,
            FLAGGED_SEGMENTS,
        );
        walk::<FLAGGED_LANES, true>(&mut self.x, r0, ll);
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

/// The census on a table a builder left (unflagged) and its text's first row
pub fn spare_bits(t: &PairTable, r0: u32) -> SpareBits {
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
    let mut r = r0;
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
mod tests {
    use super::*;
    use crate::bwt::{build, walk as plain_walk, TVector};

    fn counts_of(ll: &[u8]) -> [u32; 256] {
        let mut counts = [0u32; 256];
        for &c in ll {
            counts[c as usize] += 1;
        }
        counts
    }

    /// The two builders agree on one column: the words, the LF rows of `rows`, and the
    /// text's first row at every primary index in `ptrs`; and the words are ψ² by
    /// definition (from `bwt::build`'s table)
    fn check(ll: &[u8], rows: &[u32], ptrs: &[u32], s: &mut RunScratch) {
        let counts = counts_of(ll);
        let (mut a, mut b) = (PairTable::default(), PairTable::default());
        let (mut lfa, mut lfb) = (Vec::new(), Vec::new());
        build_pairs(ll, &counts, &mut a, rows, &mut lfa);
        for &o in ptrs {
            let r0 = build_pairs_runs(ll, &counts, &mut b, s, rows, &mut lfb, o)
                .expect("the counts are ll's own");
            assert!(a.p == b.p, "{ll:?}: the words");
            assert_eq!(lfa, lfb, "{ll:?}: the LF rows");
            assert_eq!((a.f, a.fshift, &a.first), (b.f, b.fshift, &b.first));
            assert_eq!(r0, pair_start(ll, &a, o), "{ll:?}, orig_ptr {o}");
            assert!(s.cells < 2 * s.runs, "{} cells, {} runs", s.cells, s.runs);
        }
        let mut tt = TVector::new();
        build(ll, &counts, &mut tt);
        let psi = |r: u32| tt.0[r as usize] >> 8;
        for (r, &w) in b.p.iter().enumerate() {
            assert_eq!(w, ll[r] as u32 | psi(psi(r as u32)) << 8, "row {r}");
        }
    }

    /// Every column over {0, 1, 2} of length 1 to 8 and over {0, 1} of length 9 to 12, at
    /// every primary index, with every row's LF asked for
    #[test]
    fn exhaustive_small() {
        let mut s = RunScratch::default();
        let mut cases = 0;
        for (k, lens) in [(3u32, 1..=8u32), (2, 9..=12)] {
            for n in lens {
                let rows: Vec<u32> = (0..n).collect();
                for code in 0..k.pow(n) {
                    let mut v = code;
                    let ll: Vec<u8> = (0..n)
                        .map(|_| {
                            let d = v % k;
                            v /= k;
                            // (bytes spread over the alphabet, past the 3-bit groups)
                            [0u8, 9, 200][d as usize]
                        })
                        .collect();
                    check(&ll, &rows, &rows, &mut s);
                    cases += 1;
                }
            }
        }
        assert_eq!(
            cases,
            3 + 9 + 27 + 81 + 243 + 729 + 2187 + 6561 + 512 + 1024 + 2048 + 4096
        );
    }

    fn rng(seed: u64) -> impl FnMut() -> u64 {
        let mut x = seed;
        move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
    }

    /// A column in runs of the given lengths over `k` bytes
    fn runs_col(n: usize, k: u64, lens: &[usize], rnd: &mut impl FnMut() -> u64) -> Vec<u8> {
        let mut v = Vec::new();
        while v.len() < n {
            let c = (rnd() % k) as u8;
            v.extend(std::iter::repeat_n(
                c,
                lens[(rnd() % lens.len() as u64) as usize],
            ));
        }
        v.truncate(n);
        v
    }

    /// Random columns: uniform over small and full alphabets, periodic, and in runs short
    /// and long (the 16-wide stores on both sides of `build_pairs`)
    #[test]
    fn random_columns() {
        let mut rnd = rng(0x9e37_79b9_7f4a_7c15);
        let mut s = RunScratch::default();
        let sizes: &[usize] = if cfg!(miri) {
            &[1, 2, 17, 40, 300]
        } else {
            &[1, 2, 17, 255, 256, 257, 1000, 4097, 70_000, 200_000]
        };
        for &n in sizes {
            let mut cols = vec![
                (0..n).map(|_| (rnd() % 7) as u8).collect::<Vec<u8>>(),
                (0..n).map(|_| rnd() as u8).collect(),
                (0..n).map(|i| b"abc"[i % 3]).collect(),
                vec![b'z'; n],
            ];
            for k in [2, 4, 40, 256] {
                cols.push(runs_col(n, k, &[1, 2, 5, 16, 17, 40, 300], &mut rnd));
            }
            for ll in &cols {
                let mut rows: Vec<u32> = (0..64).map(|_| (rnd() % n as u64) as u32).collect();
                rows.sort_unstable();
                rows.dedup();
                let ptrs = [0, (n / 3) as u32, n as u32 - 1];
                check(ll, &rows, &ptrs, &mut s);
            }
        }
    }

    /// The pair walk, on either builder, gives the plain walk's output (#34's test, on the
    /// run-built table too, in place as well)
    #[test]
    fn pair_walk_equals_plain() {
        let n = if cfg!(miri) {
            3000
        } else {
            MIN_SEGMENTED + 1001
        };
        let mut rnd = rng(0x2545_f491_4f6c_dd1d);
        let inputs = vec![
            (0..n).map(|_| (rnd() % 7) as u8).collect::<Vec<u8>>(),
            (0..n).map(|i| b"abc"[i % 3]).collect(),
            (0..n).map(|i| b"ab"[i % 2]).collect(),
            runs_col(n, 4, &[1, 5, 16, 17, 40, 300], &mut rnd),
        ];
        let segs: &[usize] = if cfg!(miri) {
            &[2, 16]
        } else {
            &[2, 64, 512, 4096]
        };
        for ll in &inputs {
            let counts = counts_of(ll);
            let mut tt = TVector::new();
            build(ll, &counts, &mut tt);
            let (mut plain, mut pairs) = (Vec::new(), Vec::new());
            let mut x = PairScratch::default();
            for orig_ptr in [0, ll.len() as u32 / 3, ll.len() as u32 - 1] {
                plain_walk(&tt, orig_ptr, &mut plain);
                for build in [Build::Rows, Build::Runs] {
                    for &segments in segs {
                        let r0 = setup(build, &mut x, ll, &counts, orig_ptr, segments);
                        walk::<7, true>(&mut x, r0, &mut pairs);
                        assert!(plain == pairs, "{build:?}, orig_ptr {orig_ptr}, {segments}");
                    }
                    let mut k = Pairs::new(build);
                    k.invert(ll, &counts, orig_ptr, &mut pairs);
                    assert!(plain == pairs, "Pairs {build:?}, orig_ptr {orig_ptr}");
                    let mut io = ll.clone();
                    assert!(k.invert_in_place(&mut io, &counts, orig_ptr));
                    assert!(plain == io, "Pairs {build:?} in place, orig_ptr {orig_ptr}");
                }
            }
        }
    }

    /// Under Miri (miri.yml runs `miri_*`): both builders' unchecked writes and `set_len`,
    /// and #34's walk's raw-pointer lanes (on the run-built table), on columns small
    /// enough to interpret
    #[test]
    fn miri_pair_runs() {
        let mut rnd = rng(0x5eed);
        let mut s = RunScratch::default();
        for n in [1usize, 17, 300] {
            for ll in [
                (0..n).map(|_| (rnd() % 5) as u8).collect::<Vec<u8>>(),
                runs_col(n, 3, &[1, 2, 16, 17, 40], &mut rnd),
            ] {
                let rows: Vec<u32> = (0..n as u32).step_by(3).collect();
                check(&ll, &rows, &[0, n as u32 - 1], &mut s);
            }
        }
        let n = 1200;
        for ll in [
            (0..n).map(|_| (rnd() % 7) as u8).collect::<Vec<u8>>(),
            runs_col(n, 4, &[1, 5, 16, 17, 40, 300], &mut rnd),
        ] {
            let counts = counts_of(&ll);
            let mut tt = TVector::new();
            build(&ll, &counts, &mut tt);
            let (mut plain, mut pairs) = (Vec::new(), Vec::new());
            plain_walk(&tt, 7, &mut plain);
            // (the walk's code is the same whichever builder; #34's builder is checked above)
            let mut x = PairScratch::default();
            for segments in [2, 12] {
                let r0 = setup(Build::Runs, &mut x, &ll, &counts, 7, segments);
                walk::<7, true>(&mut x, r0, &mut pairs);
                assert!(plain == pairs, "{segments} segments");
            }
        }
    }

    /// The census's bounds: at most 31 flagged blocks, and the directory one byte a block
    #[test]
    fn spare_bits_bounds() {
        let mut rnd = rng(7);
        let n = 900_000;
        let ll: Vec<u8> = (0..n).map(|_| rnd() as u8).collect();
        let counts = counts_of(&ll);
        let mut t = PairTable::default();
        let mut lf = Vec::new();
        let r0 = build_pairs_runs(
            &ll,
            &counts,
            &mut t,
            &mut RunScratch::default(),
            &[],
            &mut lf,
            5,
        )
        .unwrap();
        let c = spare_bits(&t, r0);
        assert_eq!((c.dir_bytes, c.old_bytes), (3516, 28125));
        assert!(
            c.flagged_blocks <= 31 && c.flagged_rows <= 31 * 256,
            "{c:?}"
        );
    }
}
