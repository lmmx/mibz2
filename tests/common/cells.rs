//! Shared production/forced cells differential checks and mandatory reach witnesses.
use super::*;
use mibz2::bwt::{Cells, CellsCounts, ForcedCells, InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Error, Kernels, Scratch};

pub fn decode<I: InverseBwt>(data: &[u8], k: Kernels) -> (Vec<u8>, Result<(), Error>, Scratch<I>) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut scratch = Scratch::<I>::default();
    let mut out = Vec::new();
    let mut syms = Vec::new();
    let result = (|| loop {
        match sc.next(std::mem::take(&mut syms))? {
            Event::Block {
                index,
                bit,
                symbols,
                ..
            } => {
                decode_block(&symbols, k, &mut scratch, &mut out).map_err(|kind| Error {
                    kind,
                    block: Some(index),
                    bit: Some(bit),
                })?;
                syms = symbols.syms;
            }
            Event::Done => return Ok(()),
        }
    })();
    (out, result, scratch)
}

pub fn add(a: &mut CellsCounts, b: CellsCounts) {
    a.admitted += b.admitted;
    a.density += b.density;
    a.sparse += b.sparse;
    a.floor += b.floor;
    a.budget += b.budget;
    a.invalid += b.invalid;
    a.short_cycles += b.short_cycles;
    a.cells += b.cells;
}

/// Compare partial output and exact error (including computed CRC, block and bit) to the
/// shipped kernel, and complete output/error parity to bundled libbzip2 1.0.8.
pub fn check(data: &[u8], k: Kernels) -> (CellsCounts, CellsCounts) {
    let (want, err, _) = decode::<Segmented>(data, k);
    let (got, got_err, prod) = decode::<Cells>(data, k);
    assert_eq!((&got, &got_err), (&want, &err));
    let (got, got_err, forced) = decode::<ForcedCells>(data, k);
    assert_eq!((&got, &got_err), (&want, &err));
    // Cells with lazy letters (the workers' kernel on many threads), under Cells' admission
    // and under the forced route: the same partial output and exact error, and the routes
    // Cells and ForcedCells take
    let same = |a: CellsCounts, b: CellsCounts| {
        assert_eq!(
            (a.admitted, a.density, a.sparse, a.floor, a.budget, a.invalid, a.cells),
            (b.admitted, b.density, b.sparse, b.floor, b.budget, b.invalid, b.cells)
        )
    };
    {
        use mibz2::bwt::{ForcedLazyCells, LazyCells};
        let (g, e, s) = decode::<LazyCells>(data, k);
        assert_eq!((&g, &e), (&want, &err), "lazy letters");
        same(s.cells_counts(), prod.cells_counts());
        let (g, e, s) = decode::<ForcedLazyCells>(data, k);
        assert_eq!((&g, &e), (&want, &err), "lazy letters, forced");
        same(s.cells_counts(), forced.cells_counts());
    }
    // (research only, `memory-lever`: the experiment's staged kernel, the same)
    #[cfg(feature = "memory-lever")]
    {
        use mibz2::bwt::{ForcedStagedCells, StagedCells};
        let (g, e, s) = decode::<StagedCells>(data, k);
        assert_eq!((&g, &e), (&want, &err), "staged rules");
        same(s.cells_counts(), prod.cells_counts());
        let (g, e, s) = decode::<ForcedStagedCells>(data, k);
        assert_eq!((&g, &e), (&want, &err), "staged rules, forced");
        same(s.cells_counts(), forced.cells_counts());
    }
    match (oracle(data), &err) {
        (Ok(expected), Ok(())) => assert_eq!(got, expected),
        (Err(_), Err(_)) => {}
        // The shipped decoder intentionally refuses these historical bzip2 cases.
        (Ok(_), Err(e)) if matches!(e.kind, mibz2::ErrorKind::Randomised) => {}
        (a, b) => panic!("libbzip2/decoder mismatch: {a:?}, {b:?}"),
    }
    (prod.cells_counts(), forced.cells_counts())
}

pub fn witnesses() -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    for (ll, p) in [
        (vec![0; 10], 0),
        (vec![0; 11], 1),
        ([vec![1; 51], vec![0; 50]].concat(), 0),
        (vec![0; 25], 1),
    ] {
        out.push(craft(9, &[Spec::of(ll, p)], None));
    }
    out
}

pub fn assert_reach(prod: CellsCounts, forced: CellsCounts) {
    assert!(
        prod.admitted > 0 && prod.density > 0 && prod.floor > 0 && prod.budget > 0,
        "production reach: {prod:?}"
    );
    assert!(
        forced.admitted > 0 && forced.cells > 0 && forced.short_cycles > 0,
        "forced reach: {forced:?}"
    );
    assert_eq!(
        forced.invalid + forced.floor + forced.budget + forced.density,
        0
    );
}
