//! Standalone boundary and append/rollback checks using the scorer's compiled libraries.
#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
#[path = "../../../../../bench/moonshot/src/arms.rs"]
mod arms;
mod candidate;
#[path = "../../../../../tests/common/mod.rs"]
mod common;
#[path = "../../../../../docs/journal/2026-11-18-open-return/open_return.rs"]
mod open_return;

fn check_block(data: &[u8], wanted: Option<mibz2::ErrorKind>) {
    use mibz2::stream::{Event, Scanner, SliceSource};
    let mut scanner = Scanner::new(SliceSource(data));
    let Event::Block { mut symbols, .. } = scanner.next(Vec::new()).unwrap() else {
        panic!()
    };
    assert!(runs::takes_runs(&symbols));
    let mut records = runs::Runs::default();
    let n = runs::front(&symbols, &mut records, Kernels::scalar()).unwrap();
    assert!(arms::under_density(records.records().len(), n));
    for k in [Kernels::scalar(), Kernels::detect()] {
        let mut scratch = mibz2::Scratch::<candidate::Candidate>::default();
        let mut output = b"existing caller bytes".to_vec();
        let first = output.len();
        let result = mibz2::decode_block(&symbols, k, &mut scratch, &mut output);
        match &wanted {
            Some(kind) => {
                assert_eq!(result, Err(kind.clone()));
                assert_eq!(output.as_slice(), b"existing caller bytes");
            }
            None => {
                assert_eq!(result, Ok(symbols.crc));
                let oracle = common::oracle(data).unwrap();
                assert_eq!(&output[..first], b"existing caller bytes");
                assert_eq!(&output[first..], &oracle);
                symbols.crc ^= 1;
                let before = output.clone();
                let failed = mibz2::decode_block(&symbols, k, &mut scratch, &mut output);
                assert_eq!(
                    failed,
                    Err(mibz2::ErrorKind::BlockCrc {
                        stored: symbols.crc,
                        computed: symbols.crc ^ 1
                    })
                );
                assert_eq!(before, output);
                symbols.crc ^= 1;
                assert_eq!(
                    mibz2::decode_block(&symbols, k, &mut scratch, &mut output),
                    Ok(symbols.crc)
                );
                assert_eq!(&output[before.len()..], &oracle);
            }
        }
    }
}

fn main() {
    // With a one-byte last column, each row is a separate one-row LF cycle.
    // The mark at the last row expands to the repeated RLE1-coded byte.
    let invalid = common::craft(9, &[common::Spec::of(vec![b'A'; 16_384], 16_383)], None);
    check_block(&invalid, Some(mibz2::ErrorKind::RunUnfinished));
    let periodic = common::craft(9, &[common::Spec::of(vec![b'Z'; 16_385], 16_384)], None);
    check_block(&periodic, None);
    // A long mirrored block with several constant intervals in L. These
    // arbitrary marked permutations exercise the open endpoint and its word actions.
    for mark in [0, 5, 256, 2048, 4098] {
        let column: Vec<_> = (0..4099)
            .map(|i| [17, 19, 1, 0, 255, 3][(i / 67 + (i / 1001) % 3) % 6])
            .collect();
        let input = common::craft(9, &[common::Spec::of(column, mark)], None);
        if common::oracle(&input).is_ok() {
            check_block(&input, None);
        } else {
            check_block(&input, Some(mibz2::ErrorKind::RunUnfinished));
        }
    }
    let frozen = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    check_block(&frozen, None);
    println!("Scalar and detected kernels passed nonempty append, CRC rollback, retry, and unfinished-run rollback on the F5 action route.");
}
