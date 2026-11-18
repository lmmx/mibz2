//! The workers' two Cells kernels through the parallel decoder: a slot for every letter
//! (`Cells`), and lazy letters (`LazyCells`, the plan's choice from `LAZY_FROM_THREADS`
//! workers). Whatever the kernel and the thread count, one output and one error.
mod common;
use common::{cells, compress, craft, drive, plain_parallel, Spec};
use mibz2::bwt::LazyCells;
use mibz2::parallel::{Plan, LAZY_FROM_THREADS};
use mibz2::{Kernels, StreamDecoder};

/// A text of copies of one 60 kB passage, each byte changed with probability 1 in `rate`:
/// `len` bytes
fn copies(len: usize, rate: u64) -> Vec<u8> {
    let corpora = common::corpora(1);
    let text = &corpora.iter().find(|(name, _)| *name == "text").unwrap().1;
    let mut rng = common::Rng(7);
    let mut d = Vec::new();
    while d.len() < len {
        for &b in &text[..60_000] {
            d.push(if rng.below(rate) == 0 {
                b'a' + rng.below(26) as u8
            } else {
                b
            });
        }
    }
    d
}

/// Streams of several blocks. Cells admits every block of the first (copies of a passage
/// with few changes), and gives up on its budget inside the induction on every block of
/// the second (more changes). Then the admission witnesses joined, a short pattern, the
/// witnesses with a block whose CRC is wrong among them, and the first cut short.
fn streams() -> Vec<Vec<u8>> {
    let admitted = compress(&copies(5_000_000, 250), 9);
    // (more changes than the lazy kernel's budget of 0.32 n cells allows: at 1 in 100 its
    // blocks now fit it, `tests/lazy_admission.rs`)
    let over_budget = compress(&copies(1_500_000, 60), 9);
    let witnesses = cells::witnesses().concat();
    let mut bad = Spec::of([vec![1; 51], vec![0; 50]].concat(), 0);
    bad.crc = Some(123);
    let spoiled = [
        witnesses.clone(),
        craft(9, &[Spec::of(vec![0; 25], 1), bad], None),
        witnesses.clone(),
    ]
    .concat();
    let cut = admitted[..admitted.len() * 2 / 3].to_vec();
    vec![
        admitted,
        over_budget,
        witnesses,
        compress(&b"abacaba".repeat(200_000), 9),
        spoiled,
        cut,
    ]
}

#[test]
fn both_kernels_one_output_at_every_thread_count() {
    let k = Kernels::detect();
    let (mut admitted, mut budget) = (0, 0);
    for z in streams() {
        // the reference: workers on the Segmented kernel
        let (want, err) = plain_parallel(&z);
        // (the lazy kernel takes these blocks itself: they are not all fallbacks)
        let counts = cells::decode::<LazyCells>(&z, k).2.cells_counts();
        admitted += counts.admitted;
        budget += counts.budget;
        for t in [1, 4, 8, 12] {
            for lazy in [false, true] {
                let dec = StreamDecoder::workers(t, 2 * t, k, lazy);
                assert_eq!(dec.is_lazy(), lazy);
                let (out, res) = drive(&dec, &z, 1 << 16, 1 << 14);
                assert!(
                    out == want && res == err,
                    "{t} workers, lazy {lazy}: {res:?}"
                );
            }
            // the plan's own choice of kernel
            let dec = StreamDecoder::planned(Plan::Workers(t), 2 * t, k);
            assert_eq!(dec.is_lazy(), t >= LAZY_FROM_THREADS);
            let (out, res) = drive(&dec, &z, 1 << 20, 1 << 16);
            assert!(out == want && res == err, "{t} workers, planned: {res:?}");
        }
    }
    assert!(
        admitted >= 8 && budget >= 2,
        "the lazy kernel inverted {admitted} blocks itself and gave up on {budget}"
    );
}
