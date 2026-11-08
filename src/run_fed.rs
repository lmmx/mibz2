//! Research only, compiled for the tests and nothing else: two leads from #145 comment
//! 6726489772 (docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md).
//!
//! - **Line A, the run-fed T-vector.** The production table, `T[q] = L[q] | Ψ(q) << 8`,
//!   built from the block's run records (c, p, ℓ, k) by one monotone sweep, with no last
//!   column: two iterators, one over the runs in F order (Ψ's translated intervals) and one
//!   over the runs in L order (where L is constant), cut the rows into at most 2r − 1
//!   cells, and on each cell `T[q + i] = T[q] + 256·i`. The tests check it word for word
//!   against `bwt::build`.
//! - **Line B, a run-compressed Ψ².** The cells on which Ψ² is one translation and both of
//!   a two-hop step's output bytes are constant: `S₂ = S ∪ Ψ⁻¹(S \ {0})`, at most 2r − 1 of
//!   them. The census counts them on real blocks, untimed.
//!
//! Nothing in the decoder calls this module.

use crate::block::Symbols;
use crate::bwt::{self, TVector};
use crate::ErrorKind;

/// A run of the last column: its byte, its first position in L, its length, and k, the
/// copies of its byte in L before it
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Run {
    pub c: u8,
    pub p: u32,
    pub len: u32,
    pub k: u32,
}

/// The runs of `ll` in L order: maximal, or (with `split` > 0) also cut at every multiple
/// of `split`, as the checkpointed MTF loop cuts a run that crosses a checkpoint
pub(crate) fn runs_of(ll: &[u8], split: usize) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut seen = [0u32; 256];
    let mut i = 0;
    while i < ll.len() {
        let c = ll[i];
        let lim = i
            .checked_div(split)
            .map_or(usize::MAX, |w| (w + 1) * split)
            .min(ll.len());
        let mut j = i + 1;
        while j < lim && ll[j] == c {
            j += 1;
        }
        runs.push(Run {
            c,
            p: i as u32,
            len: (j - i) as u32,
            k: seen[c as usize],
        });
        seen[c as usize] += (j - i) as u32;
        i = j;
    }
    runs
}

/// `mtf::mtf_scalar` with run records out instead of the last column: the same loop, the
/// same checks and errors, but a literal pushes a record and a RUNA/RUNB count extends the
/// open one (or opens the block's first run). Returns the block's length. No n-byte buffer
/// is written. It asserts the post's claim that a literal always starts a run (its byte
/// differs from the open run's).
pub(crate) fn runs_from_symbols(
    b: &Symbols,
    runs: &mut Vec<Run>,
    counts: &mut [u32; 256],
) -> Result<usize, ErrorKind> {
    const MAX_RUN_WEIGHT: u32 = 2 * 1024 * 1024;
    runs.clear();
    *counts = [0; 256];
    let mut n = 0usize;
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
            if n + run as usize > b.max_len {
                return Err(ErrorKind::BlockTooLong);
            }
            let c = b.seq_to_unseq[list[0] as usize];
            match runs.last_mut() {
                // the count repeats the open run's byte (list[0] is the last byte written)
                Some(open) => {
                    assert_eq!(open.c, c, "a count repeats the open run's byte");
                    open.len += run;
                }
                // the block's first run, which begins with a count
                None => runs.push(Run {
                    c,
                    p: 0,
                    len: run,
                    k: 0,
                }),
            }
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
        let nn = (s - 1) as usize;
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let c = b.seq_to_unseq[v as usize];
        if let Some(open) = runs.last() {
            assert_ne!(open.c, c, "a literal starts a run");
        }
        runs.push(Run {
            c,
            p: n as u32,
            len: 1,
            k: counts[c as usize],
        });
        counts[c as usize] += 1;
        n += 1;
    }
    Ok(n)
}

/// The runs in F order: run indices, counting-sorted by byte, stable (so within a byte in
/// L order, k rising), and each one's first row a = C(c) + k
pub(crate) fn f_order(runs: &[Run], counts: &[u32; 256]) -> (Vec<u32>, Vec<u32>) {
    let mut cstart = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        cstart[c] = cum;
        cum += counts[c];
    }
    let mut slot = [0u32; 257];
    for r in runs {
        slot[r.c as usize + 1] += 1;
    }
    for c in 0..256 {
        slot[c + 1] += slot[c];
    }
    let mut order = vec![0u32; runs.len()];
    for (i, r) in runs.iter().enumerate() {
        order[slot[r.c as usize] as usize] = i as u32;
        slot[r.c as usize] += 1;
    }
    let a = order
        .iter()
        .map(|&i| {
            let r = &runs[i as usize];
            cstart[r.c as usize] + r.k
        })
        .collect();
    (order, a)
}

/// Line A's sweep over the rows `q0..q1` (a share, or the whole block), appending their
/// entries to `out`: the F-order iterator found by its interval's first row `a`, the
/// L-order one by its run's start `p` (binary searches: a share needs no count
/// checkpoints). Returns the cells it wrote.
pub(crate) fn sweep(
    runs: &[Run],
    order: &[u32],
    a: &[u32],
    q0: u32,
    q1: u32,
    out: &mut Vec<u32>,
) -> usize {
    if q0 >= q1 {
        return 0;
    }
    // the last interval (run) starting at or before q0, in each order
    let mut f = a.partition_point(|&x| x <= q0) - 1;
    let mut l = runs.partition_point(|r| r.p <= q0) - 1;
    let mut q = q0;
    let mut cells = 0;
    while q < q1 {
        let rf = &runs[order[f] as usize];
        let (af, fe) = (a[f], a[f] + rf.len);
        let rl = &runs[l];
        let le = rl.p + rl.len;
        assert!(
            af <= q && q < fe && rl.p <= q && q < le,
            "the intervals tile the rows"
        );
        let end = fe.min(le).min(q1);
        let base = rl.c as u32 | (rf.p + (q - af)) << 8;
        out.extend((0..end - q).map(|i| base + (i << 8)));
        cells += 1;
        if end == fe {
            f += 1;
        }
        if end == le {
            l += 1;
        }
        q = end;
    }
    cells
}

/// The whole table from the runs: the F-order sort, then one sweep. Returns the cells.
pub(crate) fn build_from_runs(runs: &[Run], counts: &[u32; 256], tt: &mut Vec<u32>) -> usize {
    let n: u32 = counts.iter().sum();
    let (order, a) = f_order(runs, counts);
    tt.clear();
    tt.reserve(n as usize);
    sweep(runs, &order, &a, 0, n, tt)
}

/// `build_from_runs` with every cell's entries written 8 at a time, unconditionally, past
/// the cell's end into slack that the next cell overwrites (`n + 8` slots, truncated after):
/// the branch-free fill the cost model assumes. Exactly the same table.
pub(crate) fn build_from_runs_wide(runs: &[Run], counts: &[u32; 256], tt: &mut Vec<u32>) {
    let n: u32 = counts.iter().sum();
    let (order, a) = f_order(runs, counts);
    tt.clear();
    tt.resize(n as usize + 8, 0);
    let (mut f, mut l, mut q) = (0usize, 0usize, 0u32);
    while q < n {
        let rf = &runs[order[f] as usize];
        let fe = a[f] + rf.len;
        let rl = &runs[l];
        let le = rl.p + rl.len;
        let end = fe.min(le);
        let base = rl.c as u32 | (rf.p + (q - a[f])) << 8;
        let mut i = 0;
        loop {
            let w = &mut tt[(q + i) as usize..(q + i) as usize + 8];
            for (j, e) in w.iter_mut().enumerate() {
                *e = base + ((i + j as u32) << 8);
            }
            i += 8;
            if q + i >= end {
                break;
            }
        }
        f += (end == fe) as usize;
        l += (end == le) as usize;
        q = end;
    }
    tt.truncate(n as usize);
}

/// The production table, `bwt::build`'s, word for word
pub(crate) fn production(ll: &[u8], counts: &[u32; 256]) -> Vec<u32> {
    let mut tt = TVector::new();
    bwt::build(ll, counts, &mut tt);
    tt.0
}

pub(crate) fn counts_of(ll: &[u8]) -> [u32; 256] {
    let mut counts = [0u32; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    counts
}

/// Line A's checks on one last column: the run-fed tables (plain, wide, in 1–5 shares,
/// from maximal runs and from runs cut every `split` rows) are the production table word
/// for word, and the cells number at most 2r − 1. Returns (r, cells) for the maximal runs.
pub(crate) fn check_line_a(ll: &[u8], split: usize) -> (usize, usize) {
    let counts = counts_of(ll);
    let want = production(ll, &counts);
    let n = ll.len() as u32;
    let mut got = Vec::new();
    let mut out = (0, 0);
    for (which, s) in [(0, 0), (1, split)] {
        let runs = runs_of(ll, s);
        let cells = build_from_runs(&runs, &counts, &mut got);
        assert!(got == want, "n {n}, split {s}: the run-fed table");
        assert!(
            n == 0 || cells < 2 * runs.len(),
            "n {n}: {cells} cells, {} runs",
            runs.len()
        );
        build_from_runs_wide(&runs, &counts, &mut got);
        assert!(got == want, "n {n}, split {s}: the wide fill");
        let (order, a) = f_order(&runs, &counts);
        for k in 1..=5u32 {
            got.clear();
            for j in 0..k {
                sweep(&runs, &order, &a, n * j / k, n * (j + 1) / k, &mut got);
            }
            assert!(got == want, "n {n}, split {s}, {k} shares");
        }
        if which == 0 {
            out = (runs.len(), cells);
        }
    }
    out
}

// ---- Line B: the census of a run-compressed Ψ² ----

/// One block's Ψ² census (the F formulation: a step at row x emits F[x], F[Ψ(x)] and moves
/// to Ψ²(x); the production walk from `orig_ptr` emits exactly these bytes)
#[derive(Clone, Debug, Default)]
pub(crate) struct Psi2 {
    pub n: usize,
    /// Ψ's intervals (one a run) and the refinement S ∪ Ψ⁻¹(S \ {0})
    pub r: usize,
    pub r2: usize,
    /// the fewest cells on which Ψ² translates and both bytes are constant (S₂'s cells
    /// that a minimal partition could merge are counted once)
    pub r2_min: usize,
    /// records after capping at 2n/r (and at 2n/r₂), no markers
    pub cap_r: usize,
    pub rec_cap_r: usize,
    pub cap_r2: usize,
    pub rec_cap_r2: usize,
    /// records at cap 2n/r with the segment markers of 512 and of 1,024 starts
    pub rec_m512: usize,
    pub rec_m1024: usize,
    /// bits a record needs at cap 2n/r with 1,024 starts' markers
    pub bits: u32,
    /// fast-forwards a two-hop step, in a one-chain walk of those records from `orig_ptr`
    pub ff_mean: f64,
    pub ff_max: u32,
    /// two-hop steps the chain took
    pub steps: usize,
}

/// A two-hop record: rows [start, start + len), the two bytes, and its image's interval
/// and offset there
#[derive(Clone, Copy)]
struct Rec {
    start: u32,
    len: u32,
    b1: u8,
    b2: u8,
    idx: u32,
    d: u32,
}

fn bits_for(x: usize) -> u32 {
    usize::BITS - x.leading_zeros()
}

/// The census and the checks on one table (`bwt::build`'s, unflagged) and its counts: the
/// refinement's claims checked against dense Ψ, Ψ² and F, then a one-chain walk of the
/// capped, marked records from `orig_ptr`, its first n bytes checked against `want` (the
/// production walk's output) when given
pub(crate) fn psi2_census(
    tt: &[u32],
    counts: &[u32; 256],
    orig_ptr: u32,
    want: Option<&[u8]>,
) -> Psi2 {
    let n = tt.len();
    let mut out = Psi2 {
        n,
        ..Psi2::default()
    };
    if n == 0 {
        return out;
    }
    let psi: Vec<u32> = tt.iter().map(|&e| e >> 8).collect();
    let mut inv = vec![0u32; n];
    for (x, &y) in psi.iter().enumerate() {
        inv[y as usize] = x as u32;
    }
    let mut f = vec![0u8; n];
    let mut q = 0;
    for c in 0..256 {
        f[q..q + counts[c] as usize].fill(c as u8);
        q += counts[c] as usize;
    }
    // S: a row starts an interval if its byte differs from the row before, or Ψ breaks
    let in_s = |x: usize| x == 0 || f[x] != f[x - 1] || psi[x] != psi[x - 1] + 1;
    let mut s2 = vec![false; n + 1];
    s2[n] = true;
    for x in 0..n {
        if in_s(x) {
            out.r += 1;
            s2[x] = true;
            if x != 0 {
                s2[inv[x] as usize] = true;
            }
        }
    }
    out.r2 = s2[..n].iter().filter(|&&b| b).count();
    assert!(
        out.r2 < 2 * out.r,
        "|S₂| {} ≥ 2r − 1 + 1, r {}",
        out.r2,
        out.r
    );
    // the claims: off S₂, Ψ² steps by one and both bytes hold; r2_min counts the rows
    // where one of them actually breaks
    let psi2 = |x: usize| psi[psi[x] as usize] as usize;
    for x in 0..n {
        let breaks = x == 0
            || psi2(x) != psi2(x - 1) + 1
            || f[x] != f[x - 1]
            || f[psi[x] as usize] != f[psi[x - 1] as usize];
        if breaks {
            out.r2_min += 1;
            assert!(s2[x], "row {x}: Ψ² or a byte breaks off S₂");
        }
    }
    // cells of S₂, in order
    let starts: Vec<u32> = (0..n).filter(|&x| s2[x]).map(|x| x as u32).collect();
    let cells = |extra: &[u32]| -> Vec<(u32, u32)> {
        let mut b: Vec<u32> = starts
            .iter()
            .copied()
            .chain(extra.iter().copied())
            .collect();
        b.sort_unstable();
        b.dedup();
        b.retain(|&x| (x as usize) < n);
        let mut v: Vec<(u32, u32)> = Vec::with_capacity(b.len());
        for (i, &s) in b.iter().enumerate() {
            let e = b.get(i + 1).copied().unwrap_or(n as u32);
            v.push((s, e - s));
        }
        v
    };
    let capped = |v: &[(u32, u32)], cap: u32| -> Vec<(u32, u32)> {
        let mut w = Vec::with_capacity(v.len());
        for &(s, l) in v {
            let mut o = 0;
            while o < l {
                let m = cap.min(l - o);
                w.push((s + o, m));
                o += m;
            }
        }
        w
    };
    let plain = cells(&[]);
    let cap_r = (2 * n / out.r).max(1) as u32;
    let cap_r2 = (2 * n / out.r2).max(1) as u32;
    out.cap_r = cap_r as usize;
    out.cap_r2 = cap_r2 as usize;
    out.rec_cap_r = capped(&plain, cap_r).len();
    out.rec_cap_r2 = capped(&plain, cap_r2).len();
    // segment markers: each start s its own one-row record, and so is the row whose
    // intermediate row is s (Ψ⁻¹(s)), so a lane can stop there after one byte; anchored:
    // the cap counts from each cell's own start
    let marks = |segments: usize| -> Vec<u32> {
        let mut rows = Vec::new();
        bwt::random_rows(n, segments - 1, &mut rows);
        rows.push(psi[orig_ptr as usize]);
        let mut m = Vec::with_capacity(4 * rows.len());
        // (a production start s is the row whose entry the walk reads; in the F
        // formulation that step is at row Ψ⁻¹(s), and r0's at `orig_ptr`)
        for &s in &rows {
            let x = inv[s as usize];
            let p = inv[x as usize];
            m.extend([x, x + 1, p, p + 1]);
        }
        m
    };
    out.rec_m512 = capped(&cells(&marks(512)), cap_r).len();
    let recs_cells = capped(&cells(&marks(1024)), cap_r);
    out.rec_m1024 = recs_cells.len();
    // the records, their images' intervals found by a merge (images sorted)
    let mut recs: Vec<Rec> = recs_cells
        .iter()
        .map(|&(s, l)| Rec {
            start: s,
            len: l,
            b1: f[s as usize],
            b2: f[psi[s as usize] as usize],
            idx: 0,
            d: 0,
        })
        .collect();
    let rec_starts: Vec<u32> = recs.iter().map(|r| r.start).collect();
    for r in recs.iter_mut() {
        let img = psi2(r.start as usize) as u32;
        let i = rec_starts.partition_point(|&x| x <= img) - 1;
        r.idx = i as u32;
        r.d = img - rec_starts[i];
    }
    let max_d = recs.iter().map(|r| r.d).max().unwrap_or(0) as usize;
    let max_len = recs.iter().map(|r| r.len).max().unwrap_or(0) as usize;
    out.bits =
        bits_for(recs.len().saturating_sub(1)) + bits_for(max_d) + bits_for(max_len) + 16 + 2;
    // the one-chain walk from orig_ptr, F formulation, lazy fast-forward
    let x0 = orig_ptr;
    let mut idx = rec_starts.partition_point(|&x| x <= x0) - 1;
    let mut off = x0 - rec_starts[idx];
    let mut bytes = Vec::with_capacity(n + 1);
    let (mut ff_sum, mut ff_max) = (0u64, 0u32);
    while bytes.len() < n {
        let mut ff = 0;
        while off >= recs[idx].len {
            off -= recs[idx].len;
            idx += 1;
            ff += 1;
        }
        ff_sum += ff as u64;
        ff_max = ff_max.max(ff);
        out.steps += 1;
        let r = recs[idx];
        bytes.push(r.b1);
        bytes.push(r.b2);
        idx = r.idx as usize;
        off += r.d;
    }
    bytes.truncate(n);
    out.ff_mean = ff_sum as f64 / out.steps as f64;
    out.ff_max = ff_max;
    if let Some(w) = want {
        assert!(bytes == w, "n {n}: the two-hop records' bytes");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rng(seed: u64) -> impl FnMut() -> u64 {
        let mut x = seed;
        move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        }
    }

    /// Every string over `sigma` letters of each length up to `max`
    fn all_strings(sigma: u8, max: usize, mut each: impl FnMut(&[u8])) {
        let mut s = Vec::new();
        for len in 1..=max {
            s.clear();
            s.resize(len, 0u8);
            loop {
                each(&s);
                let mut i = 0;
                while i < len && s[i] == sigma - 1 {
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

    /// Random last columns: run-heavy (runs of 1–40, some of 16 and more, which take the
    /// production build's 16-row stores), uniform, and periodic
    fn random_columns() -> Vec<Vec<u8>> {
        let mut rnd = rng(0x9e37_79b9_7f4a_7c15);
        let mut v = Vec::new();
        for &len in &[16usize, 17, 31, 64, 900, 1000, 4096, 9000, 70_000] {
            for sigma in [1u64, 2, 3, 5, 40, 256] {
                let mut ll = Vec::new();
                while ll.len() < len {
                    let c = (rnd() % sigma) as u8;
                    let run = [1, 1, 2, 3, 15, 16, 17, 33, 40][(rnd() % 9) as usize];
                    ll.extend(std::iter::repeat_n(c, run));
                }
                ll.truncate(len);
                v.push(ll);
                v.push((0..len).map(|_| (rnd() % sigma) as u8).collect());
            }
            v.push((0..len).map(|i| (i % 3) as u8).collect());
            v.push((0..len).map(|i| b"abcab"[i % 5]).collect());
        }
        v
    }

    #[test]
    fn line_a_exhaustive_small() {
        let mut cases = 0;
        all_strings(3, 9, |s| {
            check_line_a(s, 4);
            cases += 1;
        });
        all_strings(4, 7, |s| {
            check_line_a(s, 3);
            cases += 1;
        });
        all_strings(2, 14, |s| {
            check_line_a(s, 5);
            cases += 1;
        });
        assert_eq!(cases, 29_523 + 21_844 + 32_766);
    }

    #[test]
    fn line_a_random() {
        for ll in random_columns() {
            check_line_a(&ll, 1 << 10);
        }
    }

    /// The real BWT of a text (rotations sorted), its primary index
    fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
        let n = text.len();
        let mut rot: Vec<usize> = (0..n).collect();
        rot.sort_by(|&a, &b| {
            (0..n)
                .map(|i| text[(a + i) % n])
                .cmp((0..n).map(|i| text[(b + i) % n]))
        });
        let ll = rot.iter().map(|&i| text[(i + n - 1) % n]).collect();
        let p = rot.iter().position(|&i| i == 0).unwrap() as u32;
        (ll, p)
    }

    /// The production walk's output for a column and primary index
    fn walked(tt: &[u32], orig_ptr: u32) -> Vec<u8> {
        let t = TVector(tt.to_vec());
        let mut out = Vec::new();
        bwt::walk(&t, orig_ptr, &mut out);
        out
    }

    #[test]
    fn line_b_exhaustive_permutations() {
        // every permutation of 1..=8 elements: S₂ = S ∪ π⁻¹(S \ {0}) holds every breakpoint
        // of π², and has at most 2|S| − 1 rows
        fn perms(n: usize, each: &mut impl FnMut(&[usize])) {
            let mut p: Vec<usize> = (0..n).collect();
            let mut c = vec![0usize; n];
            each(&p);
            let mut i = 0;
            while i < n {
                if c[i] < i {
                    if i % 2 == 0 {
                        p.swap(0, i)
                    } else {
                        p.swap(c[i], i)
                    }
                    each(&p);
                    c[i] += 1;
                    i = 0;
                } else {
                    c[i] = 0;
                    i += 1;
                }
            }
        }
        let mut total = 0;
        for n in 1..=8 {
            perms(n, &mut |p| {
                let mut inv = vec![0; n];
                for (x, &y) in p.iter().enumerate() {
                    inv[y] = x;
                }
                let s: Vec<bool> = (0..n).map(|x| x == 0 || p[x] != p[x - 1] + 1).collect();
                let mut s2 = s.clone();
                for (x, &b) in s.iter().enumerate() {
                    if b && x != 0 {
                        s2[inv[x]] = true;
                    }
                }
                let r = s.iter().filter(|&&b| b).count();
                let r2 = s2.iter().filter(|&&b| b).count();
                assert!(r2 < 2 * r);
                for x in 1..n {
                    if p[p[x]] != p[p[x - 1]] + 1 {
                        assert!(s2[x]);
                    }
                }
                total += 1;
            });
        }
        assert_eq!(total, 46_233);
    }

    #[test]
    fn line_b_real_bwts_and_columns() {
        // real BWTs of every ternary text of length 1–9 (cyclic, periodic ones included),
        // the two-hop records' walk against the production walk
        let mut cases = 0;
        all_strings(3, 9, |text| {
            let (ll, p) = bwt_of(text);
            let counts = counts_of(&ll);
            let tt = production(&ll, &counts);
            let want = walked(&tt, p);
            psi2_census(&tt, &counts, p, Some(&want));
            cases += 1;
        });
        assert_eq!(cases, 29_523);
        // any last column is some Ψ (several cycles where it isn't a text's): random,
        // run-heavy and periodic columns at three primary indices
        for ll in random_columns() {
            let counts = counts_of(&ll);
            let tt = production(&ll, &counts);
            let n = ll.len() as u32;
            for p in [0, n / 2, n - 1] {
                let want = walked(&tt, p);
                psi2_census(&tt, &counts, p, Some(&want));
            }
        }
    }

    /// The census on real blocks: every block of each file in `MIBZ2_RUNFED_CORPORA`
    /// (colon-separated .bz2 paths), one line a block to `MIBZ2_RUNFED_OUT` (TSV). Line A's
    /// table from the blocks' MTF symbols (no last column), word for word against
    /// `bwt::build`, maximal and checkpoint-cut; Line B's census and two-hop walk against the
    /// production room walk's output; each block's CRC. Ignored without the variable:
    /// `MIBZ2_RUNFED_CORPORA=a.bz2:b.bz2 cargo test --release --lib run_fed -- --ignored`
    #[test]
    #[ignore]
    fn census_real_blocks() {
        use crate::stream::{Event, Scanner, SliceSource};
        use std::io::Write;
        let Ok(paths) = std::env::var("MIBZ2_RUNFED_CORPORA") else {
            eprintln!("MIBZ2_RUNFED_CORPORA unset: skipped");
            return;
        };
        let mut tsv: Box<dyn Write> = match std::env::var("MIBZ2_RUNFED_OUT") {
            Ok(p) => Box::new(std::fs::File::create(p).unwrap()),
            Err(_) => Box::new(std::io::sink()),
        };
        writeln!(
            tsv,
            "file\tblock\tn\tr\tr_split\tcells\tcells_split\tlen_le8\tr2\tr2_min\tcap_r\trec_cap_r\tcap_r2\trec_cap_r2\trec_m512\trec_m1024\tbits\tff_mean\tff_max\tsteps\tsyms\twin16"
        )
        .unwrap();
        let k = crate::Kernels::detect();
        for path in paths.split(':') {
            let data = std::fs::read(path).unwrap();
            let name = std::path::Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let mut sc = Scanner::new(SliceSource(&data));
            let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
            let (mut runs, mut rcounts) = (Vec::new(), [0u32; 256]);
            let (mut tt, mut seg, mut walk_out, mut rle_out) = (
                TVector::new(),
                bwt::SegScratch::default(),
                Vec::new(),
                Vec::new(),
            );
            let mut got = Vec::new();
            let mut block = 0;
            while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap()
            {
                let n = crate::mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
                // line A: runs straight from the symbols, no column
                let rn = runs_from_symbols(&b, &mut runs, &mut rcounts).unwrap();
                assert_eq!((rn, rcounts), (n, counts));
                assert!(runs == runs_of(&ll, 0), "block {block}: the symbols' runs");
                bwt::build(&ll, &counts, &mut tt);
                let cells = build_from_runs(&runs, &counts, &mut got);
                assert!(got == tt.0, "block {block}: the run-fed table");
                build_from_runs_wide(&runs, &counts, &mut got);
                assert!(got == tt.0, "block {block}: the wide fill");
                let split = runs_of(&ll, crate::mtf::CHECKPOINT);
                let cells_split = build_from_runs(&split, &counts, &mut got);
                assert!(got == tt.0, "block {block}: the checkpoint-cut runs' table");
                let (order, a) = f_order(&runs, &counts);
                got.clear();
                for j in 0..3u32 {
                    let nn = n as u32;
                    sweep(&runs, &order, &a, nn * j / 3, nn * (j + 1) / 3, &mut got);
                }
                assert!(got == tt.0, "block {block}: three shares");
                // short cells, where the fill's loop ends after one 8-wide store
                let mut le8 = 0;
                {
                    let (mut f, mut l, mut q) = (0usize, 0usize, 0u32);
                    while (q as usize) < n {
                        let rf = &runs[order[f] as usize];
                        let fe = a[f] + rf.len;
                        let le = runs[l].p + runs[l].len;
                        let end = fe.min(le);
                        le8 += (end - q <= 8) as usize;
                        f += (end == fe) as usize;
                        l += (end == le) as usize;
                        q = end;
                    }
                }
                // the 16-row windows the production build stores whole (its run test)
                let win16 = ll[..n / 16 * 16]
                    .chunks_exact(16)
                    .filter(|w| w.iter().all(|&x| x == w[0]))
                    .count();
                // the production walk (the room walk, as the decoder runs it) and the CRC
                let unflagged = tt.0.clone();
                bwt::walk_segments_room(&mut tt, b.orig_ptr, &mut walk_out, &mut seg);
                assert!(tt.0 == unflagged, "the walk leaves the table as built");
                rle_out.clear();
                crate::rle::unrle(&walk_out, &mut rle_out, k).unwrap();
                assert_eq!(crate::crc::crc32(&rle_out, k), b.crc, "block {block}: CRC");
                // line B
                let c = psi2_census(&tt.0, &counts, b.orig_ptr, Some(&walk_out));
                writeln!(
                    tsv,
                    "{name}\t{block}\t{n}\t{}\t{}\t{cells}\t{cells_split}\t{le8}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}\t{}\t{}\t{win16}",
                    runs.len(),
                    split.len(),
                    c.r2,
                    c.r2_min,
                    c.cap_r,
                    c.rec_cap_r,
                    c.cap_r2,
                    c.rec_cap_r2,
                    c.rec_m512,
                    c.rec_m1024,
                    c.bits,
                    c.ff_mean,
                    c.ff_max,
                    c.steps,
                    b.syms.len()
                )
                .unwrap();
                assert_eq!(
                    (c.n, c.r),
                    (n, runs.len()),
                    "block {block}: Ψ's intervals are the runs"
                );
                block += 1;
                syms = b.syms;
            }
            eprintln!("{name}: {block} blocks checked");
        }
    }
}
