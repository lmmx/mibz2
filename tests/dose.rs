//! Research only (`dose`): the Cells memory-dose build decodes byte-identically, through
//! the Workers plan the experiment forces, and its dose engages (Cells admits blocks there).
//! Run with `MIBZ2_DOSE_MB=1 cargo test --features dose --test dose` for a nonzero dose
//! (and `MIBZ2_DOSE_KIND=chase` for the chase).
#![cfg(feature = "dose")]
mod common;
use common::{cells, compress, corpora, craft, oracle, Spec};
use mibz2::parallel::{with_plan, Plan};

#[test]
fn the_dose_build_decodes_byte_identically_and_engages() {
    let corp = corpora(1);
    // a column Cells admits in production (tests/cells.rs's long writer: n = 440 with 8
    // records, inside both gates)
    let mut word = [vec![b'a'; 9], vec![b'b'; 9]].concat().repeat(3);
    word.push(b'c');
    let n = word.len();
    let mut rotations: Vec<_> = (0..n).collect();
    rotations.sort_by_key(|&s| (0..n).map(|i| word[(s + i) % n]).collect::<Vec<_>>());
    let column: Vec<u8> = rotations
        .iter()
        .flat_map(|&s| [word[(s + n - 1) % n]; 8])
        .collect();
    let admitted = craft(9, &[Spec::of(column, 0)], None);
    let mut inputs = vec![
        admitted.clone(),
        [
            admitted.clone(),
            compress(&corp[0].1[..300_000], 9),
            admitted,
        ]
        .concat(),
        compress(&corp[0].1[..2_000_000], 9),
        compress(&b"abacaba".repeat(200_000), 9),
        [
            compress(&b"ab".repeat(100_000), 9),
            compress(&corp[0].1[..300_000], 9),
        ]
        .concat(),
    ];
    inputs.extend(cells::witnesses());
    // (the blocks dosed in the Workers plan's decodes only)
    let mut dosed = 0;
    for z in &inputs {
        let want = oracle(z).unwrap();
        for t in [1, 2, 4] {
            let before = mibz2::dose::blocks();
            let got = with_plan(Plan::Workers(t), || mibz2::decode_parallel(z, t)).unwrap();
            dosed += mibz2::dose::blocks() - before;
            assert!(got == want, "Workers({t}): the bytes");
        }
        assert!(
            mibz2::decode(z).unwrap() == want,
            "calling thread: the bytes"
        );
    }
    assert!(
        dosed > 0,
        "no block was dosed in the Workers plan: Cells admitted nothing (dose {} MB)",
        mibz2::dose::DOSE_MB
    );
}
