//! The pipelined team (`StreamDecoder::pipelined`, research): its leader's RLE1, CRC and
//! next move-to-front overlapped with its helpers' build and walk. Against the default
//! decoder and libbzip2, at teams of 2, 3 and 4: valid streams byte for byte (the crate's
//! corpora, several streams, small blocks mixed with large, one block, no blocks), and
//! bad ones failing as the default fails, at the same block, with the same bytes before it
//! (a block CRC, a primary index, a truncation, and a move-to-front error on the block a
//! leader prefetched while the one before was in flight); and a reader dropped mid-stream.

mod common;
use common::*;
use mibz2::{Error, ErrorKind, Kernels, StreamDecoder};
use std::io::Read;

/// A decoder's output up to its end or its first error, fed 100,003 bytes at a time from
/// another thread, and its pipeline counts
fn run(dec: &StreamDecoder, z: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    let mut out = Vec::new();
    let res = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(100_003) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 16];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break Ok(()),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    dec.cancel();
                    break Err(e);
                }
            }
        }
    });
    (out, res)
}

fn pipe(
    z: &[u8],
    teams: usize,
    k: usize,
    slots: usize,
) -> (Vec<u8>, Result<(), Error>, (u64, u64)) {
    let dec = StreamDecoder::pipelined(teams, k, slots, Kernels::detect());
    assert_eq!(dec.threads(), teams * k);
    let (out, res) = run(&dec, z);
    let counts = dec.pipeline_counts();
    (out, res, counts)
}

fn default(z: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    run(
        &StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(2, 4, Kernels::detect()),
        z,
    )
}

/// The plans each test runs: teams of 2, 3 and 4; one team or two; a tight window and a wide
const PLANS: [(usize, usize, usize); 6] = [
    (1, 2, 1),
    (1, 3, 4),
    (1, 4, 2),
    (2, 2, 8),
    (2, 3, 2),
    (1, 4, 16),
];

/// Byte for byte the default decoder's output and error (kind, block and bit), at every plan
fn same_as_default(z: &[u8], what: &str) -> (u64, u64) {
    let (want, want_res) = default(z);
    let mut counts = (0, 0);
    for (teams, k, slots) in PLANS {
        let (got, res, c) = pipe(z, teams, k, slots);
        assert!(
            got == want,
            "{what}: {teams} teams of {k} in {slots}: the bytes"
        );
        assert_eq!(
            res, want_res,
            "{what}: {teams} teams of {k} in {slots}: the result"
        );
        counts = (counts.0 + c.0, counts.1 + c.1);
    }
    counts
}

#[test]
fn the_corpora() {
    let mut overlapped = 0;
    for (name, data) in corpora(1) {
        let z = compress(&data, 9);
        // the default decoder (the policy's plan) unchanged: libbzip2's bytes
        let (dflt, res) = run(&StreamDecoder::new(4, 16, Kernels::detect()), &z);
        res.unwrap();
        assert!(
            dflt == data && oracle(&z).unwrap() == data,
            "{name}: the default"
        );
        let (got, res, _) = pipe(&z, 1, 3, 4);
        res.unwrap();
        assert!(got == data, "{name}: against the input");
        overlapped += same_as_default(&z, name).0;
    }
    assert!(
        overlapped > 0,
        "no block was ever staged during another's inversion"
    );
}

#[test]
fn every_level_and_shape() {
    let mut rng = Rng(0x005e_ed0f_9195);
    for (name, data) in samples(&mut rng) {
        for level in [1, 3, 9] {
            let z = compress(&data, level);
            same_as_default(&z, &format!("{name} level {level}"));
            let (got, res, _) = pipe(&z, 2, 2, 4);
            res.unwrap();
            assert!(got == data, "{name} level {level}: against the input");
        }
    }
}

/// Several streams, their blocks below `MIN_COOP` (levels 1 and 2) between full ones, and
/// streams ending in a short block: the pipeline drains for each small block
#[test]
fn small_blocks_among_large_and_streams() {
    let corp = corpora(1);
    let text = &corp[0].1;
    let mut z = Vec::new();
    let mut want = Vec::new();
    for (i, level) in [9, 1, 9, 2, 1, 9, 9].into_iter().enumerate() {
        let part = &text[i * 300_001..i * 300_001 + 250_000 + i * 97_003];
        z.extend(compress(part, level));
        want.extend_from_slice(part);
    }
    assert!(oracle(&z).unwrap() == want);
    same_as_default(&z, "mixed");
    let (got, res, _) = pipe(&z, 1, 4, 4);
    res.unwrap();
    assert!(got == want);
}

#[test]
fn one_block_and_none() {
    let corp = corpora(1);
    let text = &corp[0].1;
    // one full block, one small, and a stream with no blocks at all
    for (what, data) in [
        ("one full block", text[..850_000].to_vec()),
        ("one small block", text[..1_000].to_vec()),
        ("empty stream", Vec::new()),
    ] {
        let z = compress(&data, 9);
        same_as_default(&z, what);
        let (got, res, _) = pipe(&z, 1, 2, 2);
        res.unwrap();
        assert!(got == data, "{what}");
    }
    // no input at all
    let (got, res, _) = pipe(&[], 1, 3, 3);
    assert!(got.is_empty());
    assert_eq!(res.unwrap_err().kind, ErrorKind::Empty);
}

/// A stored block CRC flipped in the middle of a file: the error at that block, every block
/// before it delivered
#[test]
fn a_block_crc_in_the_middle() {
    let corp = corpora(1);
    let text = &corp[0].1[..];
    let z = compress(text, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 4);
    for at in [1, 2, starts.len() - 1] {
        let mut t = z.clone();
        let bit = starts[at] + 48 + 5;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        same_as_default(&t, &format!("CRC of block {at}"));
        let (got, res, _) = pipe(&t, 1, 3, 4);
        let e = res.unwrap_err();
        assert!(matches!(e.kind, ErrorKind::BlockCrc { .. }), "{e}");
        assert_eq!(e.block, Some(at as u64));
        assert_eq!(e.bit, Some(starts[at]));
        assert!(!got.is_empty() && text.starts_with(&got));
        // (the blocks before it: 900,000 bytes each or so, less the RLE1 runs)
        assert!(
            got.len() as u64 >= at as u64 * 800_000,
            "{} bytes",
            got.len()
        );
    }
}

/// A random last column of n bytes, as a crafted block's
fn column(n: usize, seed: u64) -> Vec<u8> {
    let mut r = Rng(seed);
    (0..n).map(|_| b"abcdefgh"[r.below(8) as usize]).collect()
}

/// A primary index at the block's length, on a full block between good ones
#[test]
fn a_bad_orig_ptr() {
    let n = 300_000;
    let good = |seed| Spec::of(column(n, seed), 12_345);
    let mut bad = Spec::of(column(n, 3), n as u32);
    bad.crc = Some(0);
    let z = craft(9, &[good(1), good(2), bad, good(4)], Some(0));
    assert!(oracle(&z).is_err());
    same_as_default(&z, "bad orig_ptr");
    let (got, res, _) = pipe(&z, 1, 2, 4);
    let e = res.unwrap_err();
    assert_eq!((e.kind, e.block), (ErrorKind::BadOrigPtr, Some(2)));
    let mut want = reference(&column(n, 1), 12_345).unwrap();
    want.extend(reference(&column(n, 2), 12_345).unwrap());
    assert!(got == want);
}

#[test]
fn truncated() {
    let corp = corpora(1);
    let z = compress(&corp[0].1[..3_000_000], 9);
    for cut in [z.len() / 2, z.len() - 3, z.len() - 11, 10] {
        same_as_default(&z[..cut], &format!("cut at {cut}"));
        let (_, res, _) = pipe(&z[..cut], 2, 2, 4);
        assert_eq!(res.unwrap_err().kind, ErrorKind::Truncated, "cut at {cut}");
    }
}

/// A block whose move-to-front fails (a RUNA run too long) right after two full blocks:
/// with one team, the leader stages it while its helpers invert the block before, so its
/// error is published before that block's bytes; the reader still gets them first
#[test]
fn an_mtf_error_on_the_prefetched_block() {
    let n = 400_000;
    let good = |seed| Spec::of(column(n, seed), 777);
    let mut bad = Spec::of(vec![b'a'; 10], 0);
    bad.in_use = Some(vec![b'a']);
    bad.syms = Some([vec![0u16; 22], vec![2]].concat());
    bad.crc = Some(0);
    let z = craft(9, &[good(5), good(6), bad, good(7)], Some(0));
    let (want, want_res) = default(&z);
    let e = want_res.clone().unwrap_err();
    assert_eq!((e.kind.clone(), e.block), (ErrorKind::RunTooLong, Some(2)));
    let mut expect = reference(&column(n, 5), 777).unwrap();
    expect.extend(reference(&column(n, 6), 777).unwrap());
    assert!(want == expect);
    // until a run has staged it overlapped (the scanner is far ahead of the decode, so
    // nearly always the first time), and every time the same bytes and error
    let mut overlapped_errors = 0;
    for _ in 0..20 {
        for k in [2, 3, 4] {
            let (got, res, (_, errs)) = pipe(&z, 1, k, 8);
            assert!(got == want, "k {k}: the bytes before the error");
            assert_eq!(res, want_res, "k {k}");
            overlapped_errors += errs;
        }
        if overlapped_errors > 0 {
            break;
        }
    }
    assert!(
        overlapped_errors > 0,
        "the failing block was never staged overlapped"
    );
}

/// The reader dropped mid-stream, on every plan: nothing hangs (a watchdog fails the test
/// if a drop takes a minute)
#[test]
fn dropped_mid_stream() {
    let corp = corpora(1);
    let data = corp[0].1.clone();
    let z = compress(&data, 9);
    for (teams, k, slots) in PLANS {
        let z = z.clone();
        let data = data.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut r = mibz2::BzReader::parallel_pipelined(
                std::io::Cursor::new(z),
                teams,
                k,
                slots,
                Kernels::detect(),
            );
            let mut buf = vec![0u8; 1_500_000];
            r.read_exact(&mut buf).unwrap();
            assert!(buf == data[..buf.len()]);
            drop(r);
            tx.send(()).unwrap();
        });
        rx.recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or_else(|_| panic!("{teams} teams of {k}: the drop hung"));
    }
    // and the push form, cancelled by its reader while its writer writes
    let dec = StreamDecoder::pipelined(1, 3, 3, Kernels::detect());
    std::thread::scope(|s| {
        let w = s.spawn(|| {
            for c in z.chunks(1000) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 100_000];
        assert_eq!(dec.read(&mut buf).unwrap(), 100_000);
        dec.cancel();
        w.join().unwrap();
        assert_eq!(dec.read(&mut buf).unwrap_err().kind, ErrorKind::Cancelled);
    });
    dec.join();
}
