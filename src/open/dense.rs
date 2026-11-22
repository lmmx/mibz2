// The dense band (`DENSE`, `Cap::dense`): what `OpenCells` admits above `LazyCells`' density
// line, the first pass's cap and its refusal, exactly at its edges.
use super::b5::{plain, Lcg};
use super::*;

/// A column of `n` rows of Wikidata's dense kind: short runs (1 or 2 rows) and, a run in
/// `long` (more short runs with a larger `long`), long ones (17 to 400 rows), over 40 bytes.
/// Near `long` 40 it lies in the dense band within the cap (r/n about 0.155, 0.22 n retained);
/// from about 60 it is past the cap, and from about 100 past the band.
pub(super) fn dense_column(g: &mut Lcg, n: usize, long: u32) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    let mut c = 0u8;
    while l.len() < n {
        c = (c + 1 + g.below(39) as u8) % 40;
        let w = if g.below(long) == 0 {
            17 + g.below(384)
        } else {
            1 + g.below(2)
        } as usize;
        l.extend(std::iter::repeat_n(c, w.min(n - l.len())));
    }
    l
}

/// The record-count gates' edges: the old line, the dense band's 7 in 25, the sparse gate
/// below both, and the kernel's entry condition; a cap only in the dense band
#[test]
fn dense_band_gate_edges() {
    type O = OpenCells;
    assert_eq!(O::cap(149, 1000, 0), Some(Cap::NONE));
    assert_eq!(O::cap(150, 1000, 0), Some(Cap::dense(1000)));
    assert_eq!(O::cap(279, 1000, 999), Some(Cap::dense(1000)));
    assert_eq!(O::cap(280, 1000, 0), None, "r/n 0.28 is refused");
    assert_eq!(O::cap(15, 1000, 0), None, "the sparse gate holds");
    assert_eq!(O::cap(200, 1000, 1000), None, "the row outside the block");
    assert_eq!(O::cap(0, 0, 0), None);
    assert_eq!(O::cap(200_000, 900_001, 0), None);
    assert_eq!(
        O::cap(200_000, 900_000, 0),
        Some(Cap {
            m: 256_000,
            strips: 16_384
        })
    );
    assert_eq!(
        Cap::dense(800_000).m,
        240_000,
        "3 in 10 of the rows below 853,334"
    );
    assert_eq!(Cap::dense(853_336).m, 256_000);
    assert_eq!(ForcedOpenCells::cap(500, 1000, 0), Some(Cap::NONE));
}

/// `try_run`'s cap, exactly: a block is taken at its own retained rows and thick strips and
/// refused one below either, whichever of the queues' cold path (`over`) or the pass's end
/// sees it; a refused block leaves the kernel to decode the next exactly, and takes no chunk
/// past the cap
#[test]
fn cap_is_exact_and_refusal_leaves_no_state() {
    let mut g = Lcg(0x6361_7073);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for (n, long) in [(3_000usize, 6u32), (70_000, 4), (300_000, 8), (900_000, 5)] {
        let l = dense_column(&mut g, n, long);
        let r = Runs::of_column(&l);
        for p in [0, n as u32 / 2, n as u32 - 1] {
            let want = plain(&l, p);
            let s = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
            assert!(out == want);
            let (m, ns) = (s.retained as u32, s.thick);
            for cap in [
                Cap { m, strips: ns },
                Cap::NONE,
                Cap {
                    m: u32::MAX,
                    strips: ns,
                },
            ] {
                out.clear();
                let t = k.try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out);
                assert!(t.is_some() && out == want, "n {n}, at the cap {cap:?}");
            }
            for cap in [
                Cap {
                    m: m - 1,
                    strips: ns,
                },
                Cap {
                    m,
                    strips: ns.wrapping_sub(1),
                },
                Cap { m: 0, strips: 0 },
                Cap {
                    m: m / 2,
                    strips: ns,
                },
            ] {
                if cap.strips == usize::MAX {
                    continue;
                }
                let previous = k.edges.capacity();
                let t = k.try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out);
                assert!(t.is_none(), "n {n}: past the cap {cap:?} ({m}, {ns})");
                assert!(k.edges.capacity() <= previous.max(cap.strips.saturating_mul(3)));
                // every chunk the pass took, a whole chunk of ranks within the cap, or the
                // last of a byte's, partly filled (at most one a byte)
                assert!(
                    k.spill.taken as usize <= cap.m as usize / 512 + 1 + 256,
                    "{} chunks under a cap of {}",
                    k.spill.taken as usize,
                    cap.m
                );
                // the kernel decodes the next block exactly after a refusal
                let s2 = k.run::<true, true>(r.records(), n, p, LIMIT, &mut out);
                assert!(out == want && s2.retained == s.retained);
                assert!(k
                    .try_run::<true, true>(r.records(), n, p, LIMIT, cap, &mut out)
                    .is_none());
                let other = dense_column(&mut g, 5_000, 6);
                let ro = Runs::of_column(&other);
                k.run::<true, true>(ro.records(), other.len(), 7, LIMIT, &mut out);
                assert!(out == plain(&other, 7));
            }
        }
    }
}

/// The sticky refusal in the queues' cold path: once past the cap the pass takes no chunk,
/// whatever the block's length, so a refused full block's queues stay within the cap's (no
/// overflow chunk, and every chunk within `out`)
#[test]
fn refusal_is_sticky_in_the_queues() {
    // every row retained (runs of one): m = n, far past any cap
    let n = 900_000;
    let l: Vec<u8> = (0..n).map(|i| (i % 7) as u8).collect();
    let r = Runs::of_column(&l);
    let mut k = Kernel::default();
    let mut out = Vec::new();
    for m in [0u32, 1, 511, 512, 513, 65_535, 65_536, 200_000, 256_000] {
        let cap = Cap { m, strips: 16_384 };
        assert!(k
            .try_run::<true, true>(r.records(), n, 3, LIMIT, cap, &mut out)
            .is_none());
        assert_eq!(k.spill.overflow(), 0);
        assert!(
            k.spill.taken as usize <= m as usize / 512 + 7,
            "{} chunks under a cap of {m}",
            k.spill.taken as usize
        );
    }
}

/// `OpenCells` on dense columns across the band and either side of it: opened where the
/// record count admits outright, or in the band within `Cap::dense`; refused by the cap
/// exactly where the uncapped construction's counts are past it; the plain inverse's output
/// whatever the route, through one decoder reused
#[test]
fn dense_band_routes_and_decodes_exactly() {
    let mut g = Lcg(0x6465_6e73);
    let mut cells = OpenCells::default();
    let (mut outright, mut banded, mut capped, mut refused) = (0, 0, 0, 0);
    for i in 0..160 {
        let n = if i % 8 == 0 {
            900_000
        } else {
            2_000 + g.below(120_000) as usize
        };
        let long = 20 + g.below(100);
        let l = dense_column(&mut g, n, long);
        let p = g.below(n as u32);
        let r = Runs::of_column(&l);
        let recs = r.records().len();
        let (was, was_capped) = (cells.opened, cells.capped);
        let mut out = vec![0x5A; g.below(2 * n as u32) as usize];
        assert!(cells.invert_runs(&r, n, p, &mut out, Kernels::detect()));
        assert!(out == plain(&l, p), "n {n}, records {recs}");
        let opened = cells.opened > was;
        match OpenCells::cap(recs, n, p) {
            Some(Cap::NONE) => {
                assert!(opened);
                outright += 1;
            }
            Some(cap) => {
                let s = Kernel::default().run::<true, true>(r.records(), n, p, LIMIT, &mut out);
                let within = s.retained as u32 <= cap.m && s.thick <= cap.strips;
                assert_eq!(
                    opened, within,
                    "n {n}: {} retained, {} strips",
                    s.retained, s.thick
                );
                assert_eq!(cells.capped > was_capped, !within);
                if within {
                    banded += 1;
                } else {
                    capped += 1;
                }
            }
            None => {
                assert!(!opened);
                refused += 1;
            }
        }
    }
    assert!(
        outright > 5 && banded > 5 && capped > 5 && refused > 5,
        "outright {outright}, banded {banded}, capped {capped}, refused {refused}"
    );
}
