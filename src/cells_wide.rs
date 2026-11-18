// Wider admission for the lazy kernel (`WideLazyCells`, a bench arm: #145 QUEUED 6973253342 and
// the Pathfinder's REVIEW 6973651863, `just bench-admit`). The lazy kernel unchanged, its
// induction and writer master's own code, with two of Cells' admission figures moved:
// - **the density line:** a block is refused for density from 3 records in 20 rows (r/n ≥ 0.15),
//   not 1 in 10. On the Wikidata sample the blocks between the two lines are 32 full blocks,
//   13.7% of the bytes, at r/n 0.097 to 0.148: the T-vector walk decodes them today, and its
//   DRAM traffic is about a third of the decoder's. Text starts at r/n 0.28, and random columns
//   take the column path: both stay refused.
// - **the budget:** 8 cells in 25 rows (0.32 n), not 4 in 25. Those blocks need 0.161 to 0.241
//   cells a row; the admitted ones need at most 0.156.
// The sparse gate (fewer than one record per 64 rows) is Cells' own. Included in `cells.rs`.

/// The kernel `CellsVariant<V>` runs for V = `LAZY_WIDE`: lazy letters, wider admission
pub(crate) const LAZY_WIDE: u8 = 4;

/// Lazy letters with wider admission (a bench arm; the same output as `LazyCells`, block for
/// block, with fewer blocks refused)
#[doc(hidden)]
pub type WideLazyCells = CellsVariant<LAZY_WIDE>;

/// The density line: refused from `WIDE_DENSITY.0` records in `WIDE_DENSITY.1` rows
const WIDE_DENSITY: (usize, usize) = (3, 20);
/// The cells allowed: `WIDE_BUDGET.0` in `WIDE_BUDGET.1` rows
const WIDE_BUDGET: (usize, usize) = (8, 25);

/// The wide arm's two record-count gates, before the census (whose own are then skipped):
/// density at r/n ≥ 0.15, and Cells' sparse gate (fewer than one record per 64 rows)
fn wide_gates(n: usize, records: usize) -> Result<(), Refusal> {
    if records * WIDE_DENSITY.1 >= n * WIDE_DENSITY.0 {
        return Err(Refusal::Density);
    }
    if records * 64 < n {
        return Err(Refusal::Sparse);
    }
    Ok(())
}

/// The wide arm's budget for a block of `n` rows: 0.32 n, rounded down as Cells' own is
fn wide_budget(n: usize) -> usize {
    n / WIDE_BUDGET.1 * WIDE_BUDGET.0 + n % WIDE_BUDGET.1 * WIDE_BUDGET.0 / WIDE_BUDGET.1
}

#[cfg(test)]
mod wide_tests {
    use super::*;
    use crate::bwt::LazyCells;

    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, m: u64) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) % m
        }
    }

    fn records(col: &[u8]) -> Vec<u16> {
        let mut recs: Vec<u16> = Vec::new();
        for &c in col {
            match recs.last_mut() {
                Some(last) if *last >> 8 == c as u16 && *last & 0xFF < 127 => *last += 1,
                _ => recs.push((c as u16) << 8 | 1),
            }
        }
        recs
    }

    /// The gates and the budget at their lines
    #[test]
    fn the_lines() {
        assert_eq!(wide_gates(2000, 299), Ok(()));
        assert_eq!(wide_gates(2000, 300), Err(Refusal::Density));
        assert_eq!(wide_gates(2000, 32), Ok(()));
        assert_eq!(wide_gates(2000, 31), Err(Refusal::Sparse));
        assert_eq!(wide_budget(900_000), 288_000);
        assert_eq!(wide_budget(24), 7);
        for n in [1usize, 24, 25, 26, 899_999, 900_000] {
            assert_eq!(wide_budget(n), n * 8 / 25, "{n}");
        }
    }

    /// A column of runs of random letters, their mean length `mean` (so r/n about 1/mean)
    fn column(rng: &mut Lcg, n: usize, sigma: u64, mean: u64) -> Vec<u8> {
        let mut col = Vec::with_capacity(n);
        while col.len() < n {
            let c = b'a' + rng.below(sigma) as u8;
            let len = 1 + rng.below(2 * mean - 1) as usize;
            col.extend(std::iter::repeat_n(c, len));
        }
        col.truncate(n);
        col
    }

    /// Arbitrary columns, most of them no BWT, at run densities across both lines: wherever
    /// the lazy kernel admits, the wide arm admits with the same result; where the wide arm
    /// alone admits, forced Cells' bytes
    #[test]
    fn arbitrary_columns_as_the_lazy_kernel() {
        let mut rng = Lcg(0x31de);
        let (mut lazy, mut wide) = (
            Kernel {
                variant: LAZY,
                ..Kernel::default()
            },
            Kernel {
                variant: LAZY_WIDE,
                ..Kernel::default()
            },
        );
        let (mut both, mut wider) = (0, 0);
        for i in 0..4000 {
            let n = 64 + rng.below(1500) as usize;
            let mean = [3, 5, 7, 8, 9, 12, 20][i % 7];
            let sigma = 2 + rng.below(4);
            let col = column(&mut rng, n, sigma, mean);
            let p = rng.below(n as u64) as u32;
            let recs = records(&col);
            // (the gates as `attempt` takes them: the lazy kernel's in its census, the wide
            // arm's before it)
            let mut want = Vec::new();
            let w = lazy.try_invert(&recs, n, p, n / 25 * 4 + n % 25 * 4 / 25, false, &mut want);
            let mut got = Vec::new();
            let g = wide.try_invert(&recs, n, p, wide_budget(n), false, &mut got);
            if let Ok(a) = w {
                // the lazy kernel's admission is inside the wide arm's
                let b = g.expect("the wide arm admits what the lazy kernel admits");
                assert_eq!(a.1, b.1, "n {n} p {p}");
                assert!(got[..n] == want[..n], "n {n} p {p}");
                both += 1;
            } else if g.is_ok() {
                // admitted by the wide arm alone: the same bytes as forced Cells
                let mut forced = Kernel {
                    variant: LAZY,
                    ..Kernel::default()
                };
                let mut f = Vec::new();
                forced
                    .try_invert(&recs, n, p, n, true, &mut f)
                    .expect("forced Cells admits");
                assert!(got[..n] == f[..n], "n {n} p {p}");
                wider += 1;
            }
        }
        assert!(both > 50 && wider > 50, "{both} {wider}");
    }

    /// Through Cells' own admission (`attempt`, its budget by variant): between the lines (r/n
    /// in [0.10, 0.15)) the lazy kernel refuses for density and the wide arm doesn't; past
    /// 0.15 both do; below both, the wide arm admits whatever the lazy kernel admits; and where
    /// both admit, the bytes are the same
    #[test]
    fn through_attempt() {
        let mut rng = Lcg(0xad);
        let (mut between, mut past) = (0, 0);
        for i in 0..600 {
            let n = 2000 + rng.below(2000) as usize;
            let mean = [3, 4, 5, 6, 7, 8, 9, 10][i % 8];
            let col = column(&mut rng, n, 3, mean);
            let recs = records(&col);
            let r = recs.len() as f64 / n as f64;
            let p = rng.below(n as u64) as u32;
            let (mut lazy, mut wide) = (LazyCells::default(), WideLazyCells::default());
            let (mut a, mut b) = (Vec::new(), Vec::new());
            let la = lazy.0.attempt(&recs, n, p, &mut a);
            let wa = wide.0.attempt(&recs, n, p, &mut b);
            if la && wa {
                assert!(a[..n] == b[..n], "n {n} r/n {r:.3}");
            }
            let (lc, wc) = (lazy.counts(), wide.counts());
            if (0.10..0.15).contains(&r) {
                assert_eq!(lc.density, 1, "r/n {r:.3}: {lc:?}");
                assert_eq!(wc.density, 0, "r/n {r:.3}: {wc:?}");
                between += 1;
            } else if r >= 0.15 {
                assert_eq!((lc.density, wc.density), (1, 1), "r/n {r:.3}");
                past += 1;
            } else {
                // below both lines: what the lazy kernel admits, the wide arm admits (its budget
                // is larger, so it may admit more)
                assert!(!la || wa, "r/n {r:.3}: admitted by the lazy kernel alone");
            }
        }
        assert!(between > 30 && past > 30, "{between} {past}");
    }
}
