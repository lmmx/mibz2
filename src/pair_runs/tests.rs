//! `pair_runs`'s tests: the builders word for word, the walk against `bwt::walk`, the
//! states D046 named refused, the AVX2/SSE2 fills reached (`MIBZ2_EXPECT_AVX2`,
//! `MIBZ2_EXPECT_SSE2`), and a differential fuzz of builders and walk against a stable-sort
//! ψ² oracle, cell by cell (`fuzz_smoke`; `fuzz_long`, ignored, about 5 minutes in release).

use super::*;
use crate::bwt::{build, walk as plain_walk, TVector};

fn counts_of(ll: &[u8]) -> [u32; 256] {
    let mut counts = [0u32; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    counts
}

/// With `MIBZ2_EXPECT_AVX2` set (`miri.yml`'s AVX2 step), the fill a build used must be the
/// AVX2 one; with `MIBZ2_EXPECT_SSE2`, the SSE2 one (its default step): so a pass there means
/// that fill's reads and stores were checked. Either set on the wrong build fails (the
/// negative control).
fn expect_fill(f: &FillStats) {
    if std::env::var_os("MIBZ2_EXPECT_AVX2").is_some() {
        assert!(f.avx2, "AVX2 fill expected, and the SSE2 one ran");
    }
    if std::env::var_os("MIBZ2_EXPECT_SSE2").is_some() {
        assert!(!f.avx2, "SSE2 fill expected, and the AVX2 one ran");
    }
}

/// The independent reference: ψ from a stable sort of the rows by byte (ψ(q) is the q-th
/// row in that order), LF its inverse, and the cells as the maximal intervals of x between
/// the L runs' starts and the F intervals' starts, each at q = LF(x), in rising q (the
/// fill's order), with the fill's vector/scalar classification
struct Oracle {
    psi: Vec<u32>,
    lf: Vec<u32>,
    words: Vec<u32>,
    runs: usize,
    cells: Vec<(usize, usize)>,
}

impl Oracle {
    fn new(ll: &[u8]) -> Self {
        let n = ll.len();
        let mut psi: Vec<u32> = (0..n as u32).collect();
        psi.sort_by_key(|&p| ll[p as usize]);
        let mut lf = vec![0u32; n];
        for (q, &p) in psi.iter().enumerate() {
            lf[p as usize] = q as u32;
        }
        let words = (0..n)
            .map(|q| ll[q] as u32 | psi[psi[q] as usize] << 8)
            .collect();
        let mut cut = vec![false; n + 1];
        let mut runs = 0;
        for p in 0..n {
            if p == 0 || ll[p] != ll[p - 1] {
                runs += 1;
                cut[p] = true;
                // the F interval of the run starting at p starts at LF(p)
                cut[lf[p] as usize] = true;
            }
        }
        cut[n] = true;
        let mut cells = Vec::new();
        let mut x = 0;
        for (y, &c) in cut.iter().enumerate().skip(1) {
            if c {
                cells.push((lf[x] as usize, y - x));
                x = y;
            }
        }
        cells.sort_unstable();
        Oracle {
            psi,
            lf,
            words,
            runs,
            cells,
        }
    }

    /// The fill's counts, as its guard classifies each cell
    fn fill(&self, n: usize) -> (usize, usize, usize, usize) {
        let (mut vector, mut spills, mut scalar) = (0, 0, 0);
        for &(q, len) in &self.cells {
            if n >= q + 8 && len <= n - q - 8 {
                vector += 1;
                spills += (len % 8 != 0) as usize;
            } else {
                scalar += 1;
            }
        }
        (self.cells.len(), vector, spills, scalar)
    }

    /// The text from the primary index: L[ψ(o)], L[ψ²(o)], ... (`bwt::walk`'s output)
    fn text(&self, ll: &[u8], orig_ptr: u32) -> Vec<u8> {
        let mut r = self.psi[orig_ptr as usize];
        (0..ll.len())
            .map(|_| {
                let c = ll[r as usize];
                r = self.psi[r as usize];
                c
            })
            .collect()
    }
}

/// The two builders agree on one column: the words, the LF rows of `rows`, and the
/// text's first row at every primary index in `ptrs`; and the words are ψ² by
/// definition (from `bwt::build`'s table)
fn check(ll: &[u8], rows: &[u32], ptrs: &[u32], s: &mut RunScratch) {
    let counts = counts_of(ll);
    let (mut a, mut b) = (PairTable::default(), PairTable::default());
    let (mut lfa, mut lfb) = (Vec::new(), Vec::new());
    let ba = build_pairs(ll, &counts, 0, rows, &mut a, &mut lfa).unwrap();
    for &o in ptrs {
        let bb = build_pairs_runs(ll, &counts, o, rows, &mut b, s, &mut lfb)
            .expect("the counts are ll's own");
        assert!(ba.words() == bb.words(), "{ll:?}: the words");
        assert_eq!(lfa, lfb, "{ll:?}: the LF rows");
        let (ta, tb) = (ba.table(), bb.table());
        assert_eq!((ta.f, ta.fshift, &ta.first), (tb.f, tb.fshift, &tb.first));
        assert_eq!(Ok(bb.r0()), pair_start(&ba, o), "{ll:?}, orig_ptr {o}");
        assert!(
            s.fill.cells < 2 * s.runs,
            "{} cells, {} runs",
            s.fill.cells,
            s.runs
        );
        assert!(is_psi2(&bb));
        expect_fill(&s.fill);
    }
    let mut tt = TVector::new();
    build(ll, &counts, &mut tt);
    let psi = |r: u32| tt.0[r as usize] >> 8;
    for (r, &w) in ba.words().iter().enumerate() {
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
    assert_eq!(cases, 17_520);
    assert_eq!(
        cases,
        3 + 9 + 27 + 81 + 243 + 729 + 2187 + 6561 + 512 + 1024 + 2048 + 4096
    );
}

fn rng(seed: u64) -> impl FnMut() -> u64 {
    let mut x = seed | 1;
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
                    let p = setup(build, &mut x, ll, &counts, orig_ptr, segments).unwrap();
                    walk::<7, true>(p, &mut pairs);
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
/// enough to interpret: short and long cells, spills and scalar tails, buffer reuse, more
/// segments than rows, the flagged walk in place and out of place
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
    let mut seen = WalkStats::default();
    for ll in [
        (0..n).map(|_| (rnd() % 7) as u8).collect::<Vec<u8>>(),
        runs_col(n, 4, &[1, 5, 16, 17, 40, 300], &mut rnd),
    ] {
        let counts = counts_of(&ll);
        let mut tt = TVector::new();
        build(&ll, &counts, &mut tt);
        let (mut plain, mut pairs) = (Vec::new(), Vec::new());
        plain_walk(&tt, 7, &mut plain);
        let mut x = PairScratch::default();
        for (build, segments) in [(Build::Runs, 2), (Build::Runs, 12), (Build::Rows, 40)] {
            let p = setup(build, &mut x, &ll, &counts, 7, segments).unwrap();
            let w = walk::<7, true>(p, &mut pairs);
            assert!(plain == pairs, "{segments} segments");
            (seen.start_ends, seen.lf_ends, seen.grows) = (
                seen.start_ends + w.start_ends,
                seen.lf_ends + w.lf_ends,
                seen.grows + w.grows,
            );
        }
        expect_fill(&x.runs().fill);
        // in place, over the column itself
        let mut io = ll.clone();
        let p = setup(Build::Runs, &mut x, &io, &counts, 7, 30).unwrap();
        walk::<3, false>(p, &mut io);
        assert!(plain == io, "in place");
    }
    // more segments than rows
    let ll = runs_col(40, 3, &[1, 2, 9], &mut rnd);
    let counts = counts_of(&ll);
    let mut x = PairScratch::default();
    let p = setup(Build::Runs, &mut x, &ll, &counts, 3, 100).unwrap();
    let mut out = Vec::new();
    walk::<32, true>(p, &mut out);
    assert_eq!(out, Oracle::new(&ll).text(&ll, 3));
    assert!(
        seen.start_ends > 0 && seen.lf_ends > 0 && seen.grows > 0,
        "{seen:?}"
    );
}

/// The run builder's fill on a column with vector cells, spills past a cell and scalar
/// tails, under the fill `MIBZ2_EXPECT_AVX2` or `MIBZ2_EXPECT_SSE2` names (miri.yml runs
/// it with each, and with the wrong one as the negative control, which must fail)
#[test]
fn miri_fill_reached() {
    let (mut t, mut s, mut lf) = (PairTable::default(), RunScratch::default(), Vec::new());
    // one cell of 13 rows ending at the block's end, in an allocation of exactly 13 bytes: a
    // vector store from row 8 would read the 8 bytes from 8, past it (out of bounds under
    // Miri), so the guard sends it to the scalar tail
    let one = vec![7u8; 13].into_boxed_slice();
    let b = build_pairs_runs(&one, &counts_of(&one), 0, &[], &mut t, &mut s, &mut lf).unwrap();
    assert!(b.words() == Oracle::new(&one).words);
    assert_eq!(
        (s.fill.cells, s.fill.vector_cells, s.fill.scalar_cells),
        (1, 0, 1)
    );
    let mut rnd = rng(0xf111);
    // (an allocation of exactly the column, so a read past it is out of bounds under Miri)
    let ll = runs_col(500, 3, &[1, 3, 8, 9, 17, 40], &mut rnd).into_boxed_slice();
    let counts = counts_of(&ll);
    let b = build_pairs_runs(&ll, &counts, 0, &[], &mut t, &mut s, &mut lf).unwrap();
    let o = Oracle::new(&ll);
    assert!(b.words() == o.words);
    let f = s.fill;
    assert_eq!(
        (f.cells, f.vector_cells, f.spills, f.scalar_cells),
        o.fill(ll.len())
    );
    assert!(
        f.vector_cells > 0 && f.spills > 0 && f.scalar_cells > 0,
        "{f:?}"
    );
    expect_fill(&f);
    #[cfg(target_arch = "x86_64")]
    assert_eq!(f.avx2, std::is_x86_feature_detected!("avx2"));
}

/// D046's first witness, and the bad original row: no table can reach the first-byte
/// lookup but a builder's (the doc tests show none can be made by hand), the empty column
/// is refused by every entry, and an out-of-range primary index by every entry, on an
/// otherwise good block
#[test]
fn miri_witness_default_table_and_bad_row() {
    let zero = [0u32; 256];
    let (mut t, mut s, mut lf) = (PairTable::default(), RunScratch::default(), Vec::new());
    assert_eq!(
        build_pairs(&[], &zero, 0, &[], &mut t, &mut lf).err(),
        Some(PairError::Empty)
    );
    assert_eq!(
        build_pairs_runs(&[], &zero, 0, &[], &mut t, &mut s, &mut lf).err(),
        Some(PairError::Empty)
    );
    let mut x = PairScratch::default();
    for build in [Build::Rows, Build::Runs] {
        assert_eq!(
            setup(build, &mut x, &[], &zero, 0, 2).err(),
            Some(PairError::Empty)
        );
    }
    assert!(t.words().is_empty());
    let ll = [3u8, 1, 3, 3, 2];
    let counts = counts_of(&ll);
    let bad = PairError::OrigPtr { orig_ptr: 5, n: 5 };
    assert_eq!(
        build_pairs(&ll, &counts, 5, &[], &mut t, &mut lf).err(),
        Some(bad)
    );
    assert_eq!(
        build_pairs_runs(&ll, &counts, 5, &[], &mut t, &mut s, &mut lf).err(),
        Some(bad)
    );
    for build in [Build::Rows, Build::Runs] {
        assert_eq!(setup(build, &mut x, &ll, &counts, 5, 2).err(), Some(bad));
        assert_eq!(
            setup(build, &mut x, &ll, &counts, u32::MAX, 2).err(),
            Some(PairError::OrigPtr {
                orig_ptr: u32::MAX,
                n: 5
            })
        );
    }
    let b = build_pairs(&ll, &counts, 0, &[], &mut t, &mut lf).unwrap();
    assert_eq!(pair_start(&b, 5), Err(bad));
    assert_eq!(pair_start(&b, 4), Ok(Oracle::new(&ll).psi[4]));
}

/// D046's second and third witnesses: a histogram whose sum is n but whose bytes are not
/// the column's (`ll = [1]`, `counts[0] = 1`) is refused by both builders and by `setup`
/// on either, before any unchecked step; `setup(Build::Runs)` returns the error and builds
/// nothing else. Likewise wrong counts of equal sum on longer columns, a sum that wraps in
/// u32 to n, and a sum off by one
#[test]
fn miri_witness_wrong_histogram_and_no_fallback() {
    let (mut t, mut s, mut lf) = (PairTable::default(), RunScratch::default(), Vec::new());
    let mut x = PairScratch::default();
    let mut refused = |ll: &[u8], counts: &[u32; 256], what: &str| {
        assert_eq!(
            build_pairs(ll, counts, 0, &[], &mut t, &mut lf).err(),
            Some(PairError::Counts),
            "build_pairs: {what}"
        );
        assert!(t.words().is_empty(), "{what}: no words left");
        assert_eq!(
            build_pairs_runs(ll, counts, 0, &[], &mut t, &mut s, &mut lf).err(),
            Some(PairError::Counts),
            "build_pairs_runs: {what}"
        );
        for build in [Build::Rows, Build::Runs] {
            assert_eq!(
                setup(build, &mut x, ll, counts, 0, 3).err(),
                Some(PairError::Counts),
                "setup {build:?}: {what}"
            );
        }
    };
    let mut c = [0u32; 256];
    c[0] = 1;
    refused(&[1], &c, "the witness: ll [1], counts[0] = 1");
    let mut c = [0u32; 256];
    (c[0], c[1]) = (u32::MAX, 2);
    refused(&[1], &c, "counts summing to 1 only in wrapping u32");
    let mut rnd = rng(0xbad);
    let sizes: &[usize] = if cfg!(miri) {
        &[2, 17]
    } else {
        &[2, 9, 17, 64, 300]
    };
    for &n in sizes {
        let ll = runs_col(n, 4, &[1, 3, 16, 17], &mut rnd);
        let counts = counts_of(&ll);
        let a = ll[0] as usize;
        let mut c = counts;
        c[a] -= 1;
        c[(a + 1) % 256] += 1;
        refused(&ll, &c, "one copy moved to the next byte, equal sum");
        let mut c = counts;
        c[a] -= 1;
        c[255 - a] += 1;
        if 255 - a != a {
            refused(&ll, &c, "one copy moved far, equal sum");
        }
        let mut c = counts;
        c[a] += 1;
        refused(&ll, &c, "a sum one more");
        let mut c = counts;
        c[a] = c[a].wrapping_add(u32::MAX).wrapping_add(1);
        c[(a + 7) % 256] = c[(a + 7) % 256].wrapping_add(u32::MAX);
        c[(a + 9) % 256] = c[(a + 9) % 256].wrapping_add(1);
        refused(&ll, &c, "a sum that wraps in u32 to n");
    }
    // after the refusals, the same buffers build and walk a good block
    let ll = runs_col(300, 3, &[1, 5, 17], &mut rnd);
    let counts = counts_of(&ll);
    let p = setup(Build::Runs, &mut x, &ll, &counts, 11, 9).unwrap();
    let mut out = Vec::new();
    walk::<4, true>(p, &mut out);
    assert_eq!(out, Oracle::new(&ll).text(&ll, 11));
}

/// D046's fourth witness, at run time (the doc tests show the table cannot be replaced
/// nor a `Prepared` forged or kept across another `setup`): a scratch prepared for
/// seventeen zero bytes in two segments, then refused another block, holds nothing a walk
/// could read (no `Prepared` exists), and prepares and walks a good block again; and the
/// segment-count limits
#[test]
fn miri_witness_stale_state() {
    let mut counts = [0u32; 256];
    counts[0] = 17;
    let mut x = PairScratch::default();
    let mut out = Vec::new();
    let p = setup(Build::Runs, &mut x, &[0; 17], &counts, 0, 2).unwrap();
    assert!(p.n() == 17 && !p.starts().is_empty());
    walk::<1, false>(p, &mut out);
    assert_eq!(out, [0; 17]);
    // refused: its state is cleared, and no Prepared exists to walk it
    assert_eq!(
        setup(Build::Runs, &mut x, &[1], &counts, 0, 2).err(),
        Some(PairError::Counts)
    );
    assert!(x.seg.starts.is_empty() && x.lfs.is_empty() && x.t.p.is_empty());
    for bad in [0, MAX_SEGMENTS + 1, usize::MAX] {
        assert_eq!(
            setup(Build::Runs, &mut x, &[0; 17], &counts, 0, bad).err(),
            Some(PairError::Segments(bad))
        );
    }
    let p = setup(Build::Rows, &mut x, &[0; 17], &counts, 16, MAX_SEGMENTS).unwrap();
    walk::<32, true>(p, &mut out);
    assert_eq!(out, [0; 17]);
}

/// The LF rows asked for must rise strictly below n
#[test]
fn miri_rows_refused() {
    let ll = [5u8, 5, 6, 5, 7];
    let counts = counts_of(&ll);
    let (mut t, mut s, mut lf) = (PairTable::default(), RunScratch::default(), Vec::new());
    for rows in [&[0u32, 0][..], &[3, 1], &[5], &[0, 2, u32::MAX]] {
        assert_eq!(
            build_pairs(&ll, &counts, 0, rows, &mut t, &mut lf).err(),
            Some(PairError::Rows),
            "{rows:?}"
        );
        assert_eq!(
            build_pairs_runs(&ll, &counts, 0, rows, &mut t, &mut s, &mut lf).err(),
            Some(PairError::Rows),
            "{rows:?}"
        );
    }
    let b = build_pairs_runs(&ll, &counts, 0, &[0, 4], &mut t, &mut s, &mut lf).unwrap();
    assert_eq!(b.n(), 5);
    let o = Oracle::new(&ll);
    assert_eq!(lf, [o.lf[0], o.lf[4]]);
}

/// The size limit: above `MAX_N` (so links that could reach the flag bits) is refused
#[test]
#[cfg_attr(miri, ignore)]
fn too_long_refused() {
    let ll = vec![0u8; MAX_N + 1];
    let mut counts = [0u32; 256];
    counts[0] = ll.len() as u32;
    let (mut t, mut s, mut lf) = (PairTable::default(), RunScratch::default(), Vec::new());
    let e = Some(PairError::TooLong(MAX_N + 1));
    assert_eq!(build_pairs(&ll, &counts, 0, &[], &mut t, &mut lf).err(), e);
    assert_eq!(
        build_pairs_runs(&ll, &counts, 0, &[], &mut t, &mut s, &mut lf).err(),
        e
    );
}

/// The Researcher's witness (6730270019): the admission's sampler is a heuristic, not a
/// density bound. An alternating column of 16,384 bytes with nine zero bytes at each
/// sampled window samples to density 0 at a true density of 0.9375, and is admitted; at
/// 65,536 bytes (segmented) `Pairs` takes it, and decodes it right
#[test]
fn admission_heuristic_aliases() {
    let alias = |n: usize| {
        let mut ll: Vec<u8> = (0..n).map(|i| (i % 2) as u8).collect();
        let mut w = 1;
        while w + 8 <= n {
            ll[w - 1..w + 8].fill(0);
            w += 128;
        }
        ll
    };
    let ll = alias(16_384);
    let runs = 1 + ll.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(run_density(&ll), 0.0);
    assert_eq!(runs as f64 / ll.len() as f64, 0.9375);
    assert!(run_density(&ll) <= ADMIT_DENSITY, "admitted");
    if cfg!(miri) {
        return;
    }
    let ll = alias(MIN_SEGMENTED);
    let counts = counts_of(&ll);
    assert_eq!(run_density(&ll), 0.0);
    let mut k = Pairs::new(Build::Runs);
    let mut out = Vec::new();
    k.invert(&ll, &counts, 12_345, &mut out);
    assert_eq!((k.taken, k.declined), (1, 0), "taken, not declined");
    assert_eq!(out, Oracle::new(&ll).text(&ll, 12_345));
}

/// The census's bounds: at most 31 flagged blocks, and the directory one byte a block
#[test]
#[cfg_attr(miri, ignore)]
fn spare_bits_bounds() {
    let mut rnd = rng(7);
    let n = 900_000;
    let ll: Vec<u8> = (0..n).map(|_| rnd() as u8).collect();
    let counts = counts_of(&ll);
    let (mut t, mut lf) = (PairTable::default(), Vec::new());
    let mut s = RunScratch::default();
    let b = build_pairs_runs(&ll, &counts, 5, &[], &mut t, &mut s, &mut lf).unwrap();
    let c = spare_bits(&b);
    assert_eq!((c.dir_bytes, c.old_bytes), (3516, 28125));
    assert!(
        c.flagged_blocks <= 31 && c.flagged_rows <= 31 * 256,
        "{c:?}"
    );
}

// ---- The differential fuzz ----

/// What the fuzz reached
#[derive(Debug, Default)]
struct Reach {
    cases: usize,
    kinds: [usize; KINDS],
    rows: usize,
    builds: usize,
    avx2_builds: usize,
    cells: usize,
    vector_cells: usize,
    spills: usize,
    scalar_cells: usize,
    walks: usize,
    presized: usize,
    start_ends: usize,
    lf_ends: usize,
    grows: usize,
    more_segments_than_rows: usize,
    refused: usize,
    pairs_taken: usize,
    pairs_declined: usize,
    pairs_short: usize,
}

const KINDS: usize = 11;

/// A column of `n` bytes of kind `kind`: constant; alternating; periodic; runs of lengths
/// around the 8-word stores; runs of random length; random over 2-4 bytes; random over all
/// 256; the sampler-aliasing shape; a ramp of runs; one byte with sparse others; the two
/// extreme bytes in runs
fn column(kind: usize, n: usize, rnd: &mut impl FnMut() -> u64) -> Vec<u8> {
    let b = |rnd: &mut dyn FnMut() -> u64| rnd() as u8;
    match kind {
        0 => vec![b(rnd); n],
        1 => {
            let (x, y) = (b(rnd), b(rnd));
            (0..n).map(|i| if i % 2 == 0 { x } else { y }).collect()
        }
        2 => {
            let per = 1 + (rnd() % 9) as usize;
            let pat: Vec<u8> = (0..per).map(|_| (rnd() % 3) as u8 * 100).collect();
            (0..n).map(|i| pat[i % per]).collect()
        }
        3 => runs_col(
            n,
            2 + rnd() % 3,
            &[1, 2, 7, 8, 9, 15, 16, 17, 63, 64, 65],
            rnd,
        ),
        4 => {
            let k = 2 + rnd() % 255;
            let mut v = Vec::new();
            while v.len() < n {
                let c = (rnd() % k) as u8;
                v.extend(std::iter::repeat_n(c, 1 + (rnd() % 300) as usize));
            }
            v.truncate(n);
            v
        }
        5 => {
            let k = 2 + rnd() % 3;
            (0..n).map(|_| (rnd() % k) as u8).collect()
        }
        6 => (0..n).map(|_| rnd() as u8).collect(),
        7 => {
            let mut v: Vec<u8> = (0..n).map(|i| (i % 2) as u8).collect();
            let mut w = 1;
            while w + 8 <= n {
                v[w - 1..w + 8].fill(0);
                w += 128;
            }
            v
        }
        8 => {
            let len = 1 + (rnd() % 20) as usize;
            (0..n).map(|i| (i / len) as u8).collect()
        }
        9 => {
            let base = b(rnd);
            (0..n)
                .map(|_| if rnd() % 23 == 0 { b(rnd) } else { base })
                .collect()
        }
        _ => runs_col(n, 2, &[1, 4, 8, 13, 31], rnd)
            .into_iter()
            .map(|c| if c == 0 { 0 } else { 255 })
            .collect(),
    }
}

/// A column length: the 8/16/64 boundaries and others, or random up to `max`
fn length(rnd: &mut impl FnMut() -> u64, max: usize) -> usize {
    const EDGES: &[usize] = &[
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 23, 24, 25, 31, 32,
        33, 47, 48, 49, 63, 64, 65, 66, 71, 72, 73, 127, 128, 129, 255, 256, 257, 511, 512, 513,
        1023, 1024, 1025,
    ];
    if rnd() % 2 == 0 {
        EDGES[(rnd() % EDGES.len() as u64) as usize]
    } else {
        1 + (rnd() % max as u64) as usize
    }
}

/// One fuzz case: both builders (the run one on a poisoned, reused table) against the
/// oracle's words, LF rows, first rows and cells; refused wrong counts and rows; the walk
/// on both builders, into an empty or a pre-filled output buffer (never the column's own buffer: setup reads `ll` while the walk writes `io`), at several lane and segment counts
fn fuzz_case(
    rnd: &mut impl FnMut() -> u64,
    max: usize,
    reach: &mut Reach,
    tb: &mut PairTable,
    s: &mut RunScratch,
    x: &mut PairScratch,
) {
    let n = length(rnd, max);
    let kind = (rnd() % KINDS as u64) as usize;
    let ll = column(kind, n, rnd);
    let counts = counts_of(&ll);
    let o = Oracle::new(&ll);
    reach.cases += 1;
    reach.kinds[kind] += 1;
    reach.rows += n;
    let mut rows: Vec<u32> = (0..1 + rnd() % 40)
        .map(|_| (rnd() % n as u64) as u32)
        .collect();
    rows.sort_unstable();
    rows.dedup();
    let want_lf: Vec<u32> = rows.iter().map(|&r| o.lf[r as usize]).collect();
    let ptrs = [0, n as u32 - 1, (rnd() % n as u64) as u32];
    let (mut ta, mut lf) = (PairTable::default(), Vec::new());
    for &orig in &ptrs {
        let a = build_pairs(&ll, &counts, orig, &rows, &mut ta, &mut lf).unwrap();
        assert!(a.words() == o.words, "row builder, kind {kind}, n {n}");
        assert_eq!(lf, want_lf, "row builder's LF rows");
        assert_eq!(a.r0(), o.psi[orig as usize], "row builder's first row");
        // the run builder over a table whose spare words hold another build's: a cell not
        // written would leave them
        let poison: Vec<u8> = vec![0xa5; n + 9];
        let pc = counts_of(&poison);
        build_pairs_runs(&poison, &pc, 0, &[], tb, s, &mut lf).unwrap();
        let b = build_pairs_runs(&ll, &counts, orig, &rows, tb, s, &mut lf).unwrap();
        assert!(b.words() == o.words, "run builder, kind {kind}, n {n}");
        assert_eq!(lf, want_lf, "run builder's LF rows");
        assert_eq!(b.r0(), o.psi[orig as usize], "run builder's first row");
        let f = s.fill;
        assert_eq!(
            (f.cells, f.vector_cells, f.spills, f.scalar_cells),
            o.fill(n),
            "the fill's cells, kind {kind}, n {n}"
        );
        assert_eq!(s.runs, o.runs);
        assert!(f.cells < 2 * s.runs);
        reach.builds += 2;
        reach.avx2_builds += f.avx2 as usize;
        reach.cells += f.cells;
        reach.vector_cells += f.vector_cells;
        reach.spills += f.spills;
        reach.scalar_cells += f.scalar_cells;
    }
    // invalid inputs: wrong counts of equal sum, a primary index past the end
    let a = ll[(rnd() % n as u64) as usize] as usize;
    let mut bad = counts;
    bad[a] -= 1;
    bad[(a + 1 + (rnd() % 255) as usize) % 256] += 1;
    for build in [Build::Rows, Build::Runs] {
        assert_eq!(
            setup(build, x, &ll, &bad, 0, 3).err(),
            Some(PairError::Counts)
        );
        assert_eq!(
            setup(build, x, &ll, &counts, n as u32, 3).err(),
            Some(PairError::OrigPtr {
                orig_ptr: n as u32,
                n
            })
        );
        reach.refused += 2;
    }
    // the walk
    let text: Vec<Vec<u8>> = ptrs.iter().map(|&p| o.text(&ll, p)).collect();
    for (i, &orig) in ptrs.iter().enumerate() {
        let build = if rnd() % 2 == 0 {
            Build::Runs
        } else {
            Build::Rows
        };
        let segments = match rnd() % 4 {
            0 => 1 + (rnd() % 3) as usize,
            1 => 1 + (rnd() % (n as u64 + 8)) as usize,
            2 => 1 + (rnd() % 64) as usize,
            _ => 1 + (rnd() % 512) as usize,
        };
        reach.more_segments_than_rows += (segments > n) as usize;
        // a pre-filled output buffer (a copy of the column), not the same buffer: true
        // in-place coverage is in miri_pair_runs and fuzz_pairs (`invert_in_place`)
        let presized = rnd() % 2 == 0;
        let mut io = if presized {
            ll.clone()
        } else {
            vec![0x5a; rnd() as usize % 64]
        };
        let p = setup(build, x, &ll, &counts, orig, segments).unwrap();
        let w = match rnd() % 3 {
            0 => walk::<1, false>(p, &mut io),
            1 => walk::<7, true>(p, &mut io),
            _ => walk::<32, true>(p, &mut io),
        };
        assert!(
            io == text[i],
            "walk {build:?}, kind {kind}, n {n}, orig {orig}, {segments} segments"
        );
        reach.walks += 1;
        reach.presized += presized as usize;
        reach.start_ends += w.start_ends;
        reach.lf_ends += w.lf_ends;
        reach.grows += w.grows;
    }
}

/// `Pairs` on a segmented block (its admission taken or declined) against the oracle
fn fuzz_pairs(rnd: &mut impl FnMut() -> u64, reach: &mut Reach, dense: bool) {
    let n = MIN_SEGMENTED + (rnd() % 5000) as usize;
    let ll = if dense {
        column(6, n, rnd)
    } else {
        runs_col(n, 4, &[3, 9, 16, 40, 300], rnd)
    };
    let counts = counts_of(&ll);
    let orig = (rnd() % n as u64) as u32;
    let want = Oracle::new(&ll).text(&ll, orig);
    let mut k = Pairs::new(Build::Runs);
    let mut out = Vec::new();
    k.invert(&ll, &counts, orig, &mut out);
    assert!(out == want, "Pairs, dense {dense}");
    let mut io = ll.clone();
    assert!(k.invert_in_place(&mut io, &counts, orig));
    assert!(io == want, "Pairs in place, dense {dense}");
    reach.pairs_taken += k.taken;
    reach.pairs_declined += k.declined;
    // and a short block, to the short path
    let ll = column(5, 1 + (rnd() % 2000) as usize, rnd);
    let mut k = Pairs::new(Build::Runs);
    k.invert(&ll, &counts_of(&ll), 0, &mut out);
    assert!(out == Oracle::new(&ll).text(&ll, 0));
    assert_eq!((k.taken, k.declined), (0, 0));
    reach.pairs_short += 1;
}

/// Every obligation the fuzz exists for was reached
fn assert_reached(r: &Reach) {
    assert!(r.kinds.iter().all(|&k| k > 0), "{r:?}");
    assert!(
        r.vector_cells > 0 && r.spills > 0 && r.scalar_cells > 0,
        "spills and tails: {r:?}"
    );
    assert!(
        r.start_ends > 0 && r.lf_ends > 0 && r.grows > 0 && r.presized > 0,
        "the walk's flags and growth: {r:?}"
    );
    assert!(r.more_segments_than_rows > 0 && r.refused > 0, "{r:?}");
    assert!(
        r.pairs_taken > 0 && r.pairs_declined > 0 && r.pairs_short > 0,
        "admission taken, declined and short: {r:?}"
    );
}

/// A short, fixed-seed run of the fuzz, in every `cargo test`
#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_smoke() {
    let mut rnd = rng(0x00f0_220d);
    let mut reach = Reach::default();
    let (mut tb, mut s, mut x) = (
        PairTable::default(),
        RunScratch::default(),
        PairScratch::default(),
    );
    for _ in 0..400 {
        fuzz_case(&mut rnd, 1500, &mut reach, &mut tb, &mut s, &mut x);
    }
    fuzz_pairs(&mut rnd, &mut reach, true);
    fuzz_pairs(&mut rnd, &mut reach, false);
    assert_reached(&reach);
}

/// The bounded fuzz (B5): `MIBZ2_PAIR_FUZZ_SECS` seconds (300 by default) from seed
/// `MIBZ2_PAIR_FUZZ_SEED`, in release:
/// `cargo test --release --lib pair_runs::tests::fuzz_long -- --ignored --nocapture`
#[test]
#[ignore]
fn fuzz_long() {
    let env = |k: &str, d: u64| {
        std::env::var(k)
            .ok()
            .map_or(d, |v| v.parse().expect("a number"))
    };
    let seed = env("MIBZ2_PAIR_FUZZ_SEED", 0x00b5_d046);
    let secs = env("MIBZ2_PAIR_FUZZ_SECS", 300);
    let mut rnd = rng(seed);
    let mut reach = Reach::default();
    let (mut tb, mut s, mut x) = (
        PairTable::default(),
        RunScratch::default(),
        PairScratch::default(),
    );
    let start = std::time::Instant::now();
    let mut i = 0u64;
    while start.elapsed().as_secs() < secs {
        if i % 64 == 63 {
            fuzz_pairs(&mut rnd, &mut reach, i % 128 == 63);
        } else {
            fuzz_case(&mut rnd, 6000, &mut reach, &mut tb, &mut s, &mut x);
        }
        i += 1;
    }
    println!("fuzz_long: seed {seed:#x}, {secs} s: {reach:?}");
    assert_reached(&reach);
}
