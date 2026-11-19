//! Standalone boundary and append/rollback checks using the scorer's compiled libraries.
#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
#[path = "../../../bench/moonshot/src/arms.rs"]
mod arms;
mod candidate;
#[path = "../../../tests/common/mod.rs"]
mod common;
#[path = "../../../docs/journal/2026-11-18-open-return/open_return.rs"]
mod open_return;

mod boundary {
    include!("chunks.rs");

    pub fn check() {
        let mut cuts = 0;
        let mut boundaries = 0;
        for k in [Kernels::scalar(), Kernels::detect()] {
            // Counts may have the same value as their run's byte. Exercise every
            // such ambiguity, the zero count and a high count, at every short cut.
            for len in 8..=11 {
                for mut code in 0..3usize.pow(len as u32) {
                    let bytes: Vec<_> = (0..len)
                        .map(|_| {
                            let c = [0, 65, 255][code % 3];
                            code /= 3;
                            c
                        })
                        .collect();
                    let cut = safe_prefix(&bytes);
                    let mut whole = Vec::new();
                    let mut part = Vec::new();
                    let expected = rle::unrle(&bytes, &mut whole, k);
                    rle::unrle(&bytes[..cut], &mut part, k).unwrap();
                    let actual = rle::unrle(&bytes[cut..], &mut part, k);
                    assert_eq!(expected, actual, "{bytes:?}, cut={cut}");
                    if expected.is_ok() {
                        assert_eq!(whole, part)
                    }
                    cuts += 1;
                }
            }

            // Long dictionary words and token starts at every offset around
            // a chunk boundary. Compare the complete monolithic expansion.
            for offset in CHUNK - 8..CHUNK + 8 {
                for prefix in [
                    vec![3u8; 5],
                    vec![3, 3, 3, 3, 255],
                    vec![3, 3, 3, 3, 0],
                    vec![2, 7],
                ] {
                    let mut input: Vec<_> = (0..offset).map(|i| 128 + (i % 73) as u8).collect();
                    input.extend(prefix.repeat(2 * CHUNK / prefix.len()));
                    for suffix in [vec![], vec![9; 3], vec![9; 4], vec![9; 5], vec![9; 6]] {
                        let mut test = input.clone();
                        test.extend(suffix);
                        for step in [257, CHUNK - 1, CHUNK, CHUNK + 1, test.len()] {
                            let mut chunk = Vec::new();
                            let mut actual = Vec::new();
                            let mut expected = Vec::new();
                            let result = rle::unrle(&test, &mut expected, k);
                            for part in test.chunks(step) {
                                word(&mut chunk, part, &mut actual, k).unwrap();
                                assert!(chunk.len() < CHUNK);
                            }
                            let split = rle::unrle(&chunk, &mut actual, k);
                            assert_eq!(result, split);
                            if result.is_ok() {
                                assert_eq!(actual, expected)
                            }
                            boundaries += 1;
                        }
                    }
                }
            }
        }
        println!("{cuts} exhaustive count-byte cut/parsing comparisons; {boundaries} long-word stream boundaries");
    }
}

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
    boundary::check();
    // With a one-byte last column, each row is a separate one-row LF cycle.
    // The mark at the last row expands to the repeated RLE1-coded byte.
    let invalid = common::craft(9, &[common::Spec::of(vec![b'A'; 16_384], 16_383)], None);
    check_block(&invalid, Some(mibz2::ErrorKind::RunUnfinished));
    let periodic = common::craft(9, &[common::Spec::of(vec![b'Z'; 16_385], 16_384)], None);
    check_block(&periodic, None);
    let frozen = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    check_block(&frozen, None);
    println!("Scalar and detected kernels passed nonempty append, CRC rollback, retry, and unfinished-run rollback on the streamed route.");
}
