//! The stop-and-walk arms (`bwt::ForcedStopLazyCells16`, `32`, `64`: the lazy induction
//! stopped at n/16, n/32 or n/64 and the rest of the block walked from ranked starts) give
//! the shipped lazy kernel's output, block for block, on every test corpus and sample at
//! levels 9 and 1; and libbzip2's, file for file, through the public decoder. Both paths a
//! bzip2 block takes are taken: blocks walked, and blocks whose induction ends before the
//! stop. (A block decoded again because the walk does not apply needs a stopped exchange of
//! several cycles, which these files do not reach: `cells_stop.rs`'s unit tests take that
//! path, on arbitrary columns.)

mod common;
use common::*;
use mibz2::bwt::{
    ForcedLazyCells, ForcedStopLazyCells16, ForcedStopLazyCells32, ForcedStopLazyCells64,
    InverseBwt,
};
use mibz2::runs::{front, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

/// Each block of `z` inverted by the shipped lazy kernel and by each arm, compared; the
/// blocks
fn compare(
    z: &[u8],
    arms: &mut (
        ForcedStopLazyCells16,
        ForcedStopLazyCells32,
        ForcedStopLazyCells64,
    ),
) -> usize {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(z));
    let mut syms = Vec::new();
    let mut lazy = ForcedLazyCells::default();
    let mut runs = Runs::default();
    let mut blocks = 0;
    while let Event::Block { symbols, index, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let n = front(&symbols, &mut runs, k).unwrap();
        let p = symbols.orig_ptr;
        let mut want = Vec::new();
        lazy.invert_runs(&runs, n, p, &mut want, k);
        want.truncate(n);
        let check = |stop: usize, got: Vec<u8>| {
            assert!(got == want, "block {index} (n {n}), stop n/{stop}");
        };
        check(16, invert(&mut arms.0, &runs, n, p, &want));
        check(32, invert(&mut arms.1, &runs, n, p, &want));
        check(64, invert(&mut arms.2, &runs, n, p, &want));
        blocks += 1;
        syms = symbols.syms;
    }
    blocks
}

/// One block by one arm, its output buffer holding the last block's bytes, as a worker's does
fn invert<I: InverseBwt>(arm: &mut I, runs: &Runs, n: usize, p: u32, last: &[u8]) -> Vec<u8> {
    let mut got = last.to_vec();
    arm.invert_runs(runs, n, p, &mut got, Kernels::detect());
    got.truncate(n);
    got
}

#[test]
fn every_block_as_the_lazy_kernel() {
    let mut rng = Rng(0x570b_3a1c);
    let mut inputs: Vec<(String, Vec<u8>)> = corpora(1)
        .into_iter()
        .map(|(n, d)| (n.to_string(), d))
        .collect();
    inputs.extend(samples(&mut rng).into_iter().filter(|(_, d)| !d.is_empty()));
    let mut arms = Default::default();
    let mut blocks = 0;
    for (_, data) in &inputs {
        for level in [9, 1] {
            blocks += compare(&compress(data, level), &mut arms);
        }
    }
    let (s16, s32, s64) = (arms.0.stops(), arms.1.stops(), arms.2.stops());
    let natural = |s: (usize, usize)| blocks - s.0 - s.1;
    // both paths: walked, and ended before the stop
    for (stop, s) in [(16, s16), (32, s32), (64, s64)] {
        assert!(s.0 > 20, "n/{stop}: {s:?} of {blocks} blocks walked");
        assert!(
            natural(s) > 0,
            "n/{stop}: no block ended its induction before the stop"
        );
    }
}

/// Through the public decoder, as a worker runs it: libbzip2's bytes
#[test]
fn files_as_libbzip2() {
    let mut rng = Rng(0x570b_3a1d);
    for (name, data) in samples(&mut rng)
        .into_iter()
        .chain(corpora(1).into_iter().map(|(n, d)| (n.to_string(), d)))
    {
        let z = compress(&data, 9);
        let k = Kernels::detect();
        for got in [
            mibz2::decode_with::<ForcedStopLazyCells16>(&z, k),
            mibz2::decode_with::<ForcedStopLazyCells32>(&z, k),
            mibz2::decode_with::<ForcedStopLazyCells64>(&z, k),
        ] {
            assert!(got.as_deref().ok() == Some(&data[..]), "{name}");
        }
    }
}
