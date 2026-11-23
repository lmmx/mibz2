//! The block reader's symbols filled in tiles (`block::read_item_with_scratch`): truncated
//! and corrupt input fails as it did, on every path the serial scanner runs (the
//! one-thread decoder, the reader, and the parallel decoder's scanner at 1 to 6 threads).
//! Each fails with the same `Error { kind, block, bit }` and the same bytes before it, and
//! where libbzip2 fails. The truncation and wrong-level cases are #770's error-preservation
//! tests, on the serial paths.

mod common;
use common::*;
use mibz2::{BzReader, Error, ErrorKind, Kernels, StreamDecoder};
use std::io::Read;

fn read_all<R: Read>(mut r: R) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res = match r.read_to_end(&mut out) {
        Ok(_) => Ok(()),
        Err(e) => Err(e
            .get_ref()
            .and_then(|x| x.downcast_ref::<Error>())
            .cloned()
            .expect("the decoder's own error")),
    };
    (out, res)
}

/// The one-thread decoder's bytes and result
fn sequential(data: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    read_all(BzReader::new(data))
}

/// Every serial-scanner path on `data` against the one-thread decoder (`want`) and
/// libbzip2: the same bytes before the end or the first error, and the same error
fn check(data: &[u8], what: &str) -> (Vec<u8>, Result<(), Error>) {
    let (want, want_res) = sequential(data);
    assert_eq!(
        oracle(data).is_ok(),
        want_res.is_ok(),
        "{what}: libbzip2 against {want_res:?}"
    );
    if let Ok(v) = oracle(data) {
        assert!(v == want, "{what}: libbzip2's bytes differ");
    }
    let whole = mibz2::decode(data);
    assert_eq!(
        whole.as_ref().err(),
        want_res.as_ref().err(),
        "{what}: decode"
    );
    let k = Kernels::detect();
    for t in 1..=6 {
        assert_eq!(
            mibz2::decode_parallel(data, t).as_ref().err(),
            want_res.as_ref().err(),
            "{what}: decode_parallel at {t}"
        );
        // (a byte at a time for a small input only)
        let chunks: &[usize] = if data.len() < 4096 {
            &[1 << 20, 7, 1]
        } else {
            &[1 << 20, 4093]
        };
        for &chunk in chunks {
            let dec = StreamDecoder::new(t, 4 * t, k);
            let (got, res) = drive(&dec, data, chunk, 1 << 16);
            assert_eq!(res, want_res, "{what}: {t} threads, chunks of {chunk}");
            assert!(
                got == want,
                "{what}: {t} threads, chunks of {chunk}: differs"
            );
        }
    }
    (want, want_res)
}

/// Two crafted streams (levels 9 and 1) of small blocks each
fn small_streams() -> Vec<u8> {
    let mut z = craft(
        9,
        &[
            Spec::of(b"banana_bandana".to_vec(), 3),
            Spec::of(b"mississippi river".to_vec(), 5),
        ],
        None,
    );
    z.extend(craft(
        1,
        &[
            Spec::of(b"abracadabra".to_vec(), 2),
            Spec::of(b"zzzzzzzzzzzzzzzzzzzzzzzzzzzzyx".to_vec(), 7),
            Spec::of(b"tail".to_vec(), 1),
        ],
        None,
    ));
    z
}

#[test]
fn every_truncation_of_a_small_multi_stream_file() {
    let z = small_streams();
    assert_eq!(sequential(&z).1, Ok(()));
    let mut truncated = 0;
    for len in 0..z.len() {
        let (_, res) = check(&z[..len], &format!("{len} of {} bytes", z.len()));
        if matches!(&res, Err(e) if e.kind == ErrorKind::Truncated) {
            truncated += 1;
        }
    }
    assert!(truncated > z.len() / 2, "{truncated} truncated prefixes");
}

/// A level-1 block whose original pointer is past level 1's bound fails at once
/// (`BadOrigPtr`), before the truncation behind it
#[test]
fn a_bad_original_pointer_before_a_truncation() {
    let mut z = craft(9, &[Spec::of(b"banana_bandana".to_vec(), 3)], None);
    let first = z.len();
    z.extend(craft(
        1,
        &[Spec::of(b"abracadabra".to_vec(), 150_000)],
        None,
    ));
    // (the second block's header: 4 + 6 + 4 + 1 + 3 bytes, then a little of its map)
    for len in first + 18..z.len() {
        let (_, res) = check(&z[..len], &format!("{len} bytes"));
        assert_eq!(
            res.map_err(|e| e.kind),
            Err(ErrorKind::BadOrigPtr),
            "{len} bytes"
        );
    }
}

/// Blocks of many tiles' symbols (random bytes: about a symbol a byte), cut and with a
/// bit flipped across their symbols, so the error falls in every tile, its first and its
/// last groups among them
#[test]
fn large_blocks_cut_and_corrupted_in_every_tile() {
    let mut rng = Rng(0x5eed_711e);
    let mut z = Vec::new();
    for (n, level) in [(90_000, 1u32), (40_000, 9)] {
        z.extend(compress(&rng.bytes(n), level));
    }
    let starts = block_starts(&z);
    assert_eq!(starts.len(), 2, "{starts:?}");
    let (want, res) = check(&z, "whole");
    assert_eq!(res, Ok(()));
    assert_eq!(want.len(), 130_000);
    let mut kinds = std::collections::BTreeMap::<String, usize>::new();
    let mut truncated = 0;
    for i in 0..120 {
        let at = (starts[0] / 8) as usize + 40 + rng.below((z.len() - 60) as u64) as usize;
        let at = at.min(z.len() - 1);
        let (_, res) = check(&z[..at], &format!("cut at {at}"));
        assert!(res.is_err(), "cut at {at}");
        truncated += usize::from(matches!(&res, Err(e) if e.kind == ErrorKind::Truncated));
        let mut c = z.clone();
        c[at] ^= 1 << (i % 8);
        let (_, res) = check(&c, &format!("bit flipped at {at}"));
        let k = res.err().map_or("ok".into(), |e| kind_name(&e));
        *kinds.entry(k).or_default() += 1;
    }
    assert!(truncated > 100, "{truncated} of 120 cuts truncated");
    assert!(kinds.len() >= 3, "{kinds:?}");
}
