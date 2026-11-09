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

/// The F-order records, kept from block to block: `[a, p]` a run, then `[n, 0]`
#[derive(Default)]
pub struct Scratch {
    pub fo: Vec<[u32; 2]>,
}

/// The records' stable counting sort by byte (`recs` as `mtf::mtf_runs` leaves them, for
/// a block of `n` rows whose bytes `counts` counts) into `fo`. Each run's first F row is
/// `a = C(c) + k`, where k sums the lengths of its byte's runs before it. Returns false,
/// leaving `fo` unspecified, unless both orders tile `[0, n)`: the L runs (their starts
/// rise strictly from 0, and the closing record is n), and the F intervals (each byte's
/// runs' lengths sum to its count, and the counts to n). `fill` relies on exactly that.
pub fn sort(recs: &[u32], counts: &[u32; 256], n: usize, s: &mut Scratch) -> bool {
    s.fo.clear();
    if n == 0 {
        // the empty block: its closing record alone, and no counts
        s.fo.push([0, 0]);
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
    s.fo.reserve(r + 1);
    let fo = s.fo.as_mut_ptr();
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
        s.fo.set_len(r + 1);
    }
    true
}

/// The table from the L-order records and `sort`'s F-order ones (it must have returned
/// true for them, `n` rows): one sweep, a cell at a time
pub fn fill(recs: &[u32], s: &Scratch, n: usize, tt: &mut TVector, k: Kernels) {
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { fill_avx2(recs, &s.fo, n, &mut tt.0) };
        }
    }
    let _ = k;
    tt.0.clear();
    fill_rows(recs, &s.fo, 0, n as u32, &mut tt.0);
}

/// The sweep over rows `q0..q1`, appended to `out`: each iterator found by binary search
/// (the F order's over `a`, the L order's over p), so a share of the rows needs no count
/// checkpoints. The scalar fill, and the shares' (the tests split a table into shares)
pub fn fill_rows(recs: &[u32], fo: &[[u32; 2]], q0: u32, q1: u32, out: &mut Vec<u32>) {
    if q0 >= q1 {
        return;
    }
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
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_avx2(recs: &[u32], fo: &[[u32; 2]], n: usize, tt: &mut Vec<u32>) {
    use std::arch::x86_64::*;
    tt.clear();
    tt.reserve(n + 8);
    let t = tt.as_mut_ptr();
    let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
    let step = _mm256_set1_epi32(2048);
    let (fo, recs) = (fo.as_ptr(), recs.as_ptr());
    let n = n as u32;
    let (mut f, mut l, mut q) = (0usize, 0usize, 0u32);
    while q < n {
        // SAFETY (the loads): `sort` returned true, so both orders tile [0, n): while
        // q < n, run f's F interval [af, fe) and run l's L run [p, le) both hold q, and f
        // and l are below r, so f + 1 and l + 1 index the closing records at most
        let [af, pf] = *fo.add(f);
        let fe = (*fo.add(f + 1))[0];
        let (x, le) = (*recs.add(l), *recs.add(l + 1) >> 8);
        let end = fe.min(le);
        let base = (x & 0xff) | (pf + (q - af)) << 8;
        let mut v = _mm256_add_epi32(_mm256_set1_epi32(base as i32), lanes);
        let mut w = q;
        loop {
            // SAFETY: w < end <= n, so w + 8 <= n + 7, within the n + 8 reserved
            _mm256_storeu_si256(t.add(w as usize) as *mut __m256i, v);
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

/// The table's write alone, for the diagnostic's floor: n entries stored in order, 8 at a
/// time, into `tt`'s buffer (as `fill` stores them, without its cells), then the length
/// left 0, since they are no table. What a build of n entries can't go below, for its
/// stores; a build straight after finds the buffer's lines resident
pub fn floor(n: usize, tt: &mut TVector, k: Kernels) {
    tt.0.clear();
    tt.0.reserve(n);
    let spare = &mut tt.0.spare_capacity_mut()[..n];
    #[cfg(target_arch = "x86_64")]
    {
        if k.avx2 {
            // SAFETY: `k.avx2` is set only when AVX2 was detected
            return unsafe { floor_avx2(spare) };
        }
    }
    let _ = k;
    for (i, e) in spare.iter_mut().enumerate() {
        e.write(i as u32);
    }
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
        if sort(recs, counts, n, s) {
            fill(recs, s, n, tt, k);
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
        assert!(sort(&recs, &counts, n, &mut s), "n {n}: sort");
        let mut tt = TVector(vec![9; 3]);
        for k in kernels() {
            fill(&recs, &s, n, &mut tt, k);
            assert!(tt.0 == want, "n {n}, {k:?}: the table");
        }
        for shares in 1..=5u32 {
            let mut out = Vec::new();
            for j in 0..shares {
                let (q0, q1) = (n as u32 * j / shares, n as u32 * (j + 1) / shares);
                fill_rows(&recs, &s.fo, q0, q1, &mut out);
            }
            assert!(out == want, "n {n}, {shares} shares");
        }
        // a count off by one, a run's start moved, the closing record moved: refused
        if n > 0 {
            let mut c = counts;
            let b = ll[0] as usize;
            c[b] -= 1;
            c[(b + 1) % 256] += 1;
            assert!(!sort(&recs, &c, n, &mut s), "n {n}: a wrong count");
            let mut r = recs.clone();
            *r.last_mut().unwrap() += 256;
            assert!(!sort(&r, &counts, n, &mut s), "n {n}: a wrong end");
            if recs.len() > 2 {
                let mut r = recs.clone();
                r[1] += 256;
                assert!(!sort(&r, &counts, n, &mut s), "n {n}: a run moved");
            }
        }
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

    /// `sort`'s and `floor`'s unsafe writes under Miri (the scalar fill is safe code; the
    /// AVX2 fill and emitter are checked by the differential tests only), on columns small
    /// enough to interpret, runs of 16 and more included
    #[test]
    fn miri_sort_and_floor() {
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
