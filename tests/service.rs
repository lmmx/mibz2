//! The service core (`StreamDecoder::service`, research): teams that only build and walk,
//! and service workers doing every block's move-to-front, assembly, RLE1, CRC and publish.
//! Against the default decoder and libbzip2, at teams of 1 to 4, one to three teams, one or
//! two service workers, tight windows and wide: valid streams byte for byte (the crate's
//! corpora, every level, several streams with small blocks among large, one block, no
//! blocks), and bad ones failing as the default fails, at the same block and bit, with the
//! same bytes before it (a block CRC, a primary index, truncations, a move-to-front error);
//! a panic injected in a team's helper, a team's first thread, and a service worker's
//! preparing and finishing (the tests' fault hook); and cancellation and a dropped reader,
//! all under a watchdog.

mod common;
use common::*;
use mibz2::parallel::ServiceFault;
use mibz2::{Error, ErrorKind, Kernels, StreamDecoder};
use std::io::Read;
use std::time::Duration;

/// `f` on another thread, failing the test if it takes a minute (a hang)
fn watchdog<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(60))
        .unwrap_or_else(|_| panic!("{what}: hung (a minute)"))
}

/// A decoder's output up to its end or its first error, fed 100,003 bytes at a time from
/// another thread
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

/// (teams, k, services, slots)
type Shape = (usize, usize, usize, usize);

fn svc_fault(z: &[u8], s: Shape, fault: Option<ServiceFault>) -> (Vec<u8>, Result<(), Error>) {
    let (teams, k, services, slots) = s;
    let z = z.to_vec();
    watchdog(&format!("{s:?} {fault:?}"), move || {
        let dec =
            StreamDecoder::service_with_fault(teams, k, services, slots, Kernels::detect(), fault);
        assert_eq!(dec.threads(), teams * k + services);
        let r = run(&dec, &z);
        drop(dec);
        r
    })
}

fn svc(z: &[u8], s: Shape) -> (Vec<u8>, Result<(), Error>) {
    svc_fault(z, s, None)
}

fn default(z: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    run(
        &StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(2, 4, Kernels::detect()),
        z,
    )
}

/// The shapes each test runs: teams of 1 to 4; one team to three; one or two service
/// workers; a window of one block a team and a wide one
const SHAPES: [Shape; 8] = [
    // (svc2: the default's shape at 10 threads on its target, `SERVICE_PLAN` in 8)
    (2, 3, 2, 8),
    (1, 2, 1, 1),
    (1, 3, 1, 4),
    (2, 3, 1, 2),
    (2, 2, 2, 8),
    (3, 3, 1, 12),
    (1, 4, 2, 16),
    (2, 1, 1, 4),
];

/// Byte for byte the default decoder's output and error (kind, block and bit), at every shape
fn same_as_default(z: &[u8], what: &str) {
    let (want, want_res) = default(z);
    for s in SHAPES {
        let (got, res) = svc(z, s);
        assert!(got == want, "{what}: {s:?}: the bytes");
        assert_eq!(res, want_res, "{what}: {s:?}: the result");
    }
}

#[test]
fn the_corpora() {
    for (name, data) in corpora(1) {
        let z = compress(&data, 9);
        assert!(oracle(&z).unwrap() == data, "{name}: libbzip2");
        let (got, res) = svc(&z, (3, 3, 1, 12));
        res.unwrap();
        assert!(got == data, "{name}: against the input");
        same_as_default(&z, name);
    }
}

#[test]
fn every_level_and_shape() {
    let mut rng = Rng(0x5e41_ce00_c04e);
    for (name, data) in samples(&mut rng) {
        for level in [1, 3, 9] {
            let z = compress(&data, level);
            same_as_default(&z, &format!("{name} level {level}"));
            let (got, res) = svc(&z, (2, 2, 1, 4));
            res.unwrap();
            assert!(got == data, "{name} level {level}: against the input");
        }
    }
}

/// Several streams, their blocks below `MIN_COOP` (levels 1 and 2) between full ones, and
/// streams ending in a short block
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
    let (got, res) = svc(&z, (3, 3, 2, 12));
    res.unwrap();
    assert!(got == want);
}

#[test]
fn one_block_and_none() {
    let corp = corpora(1);
    let text = &corp[0].1;
    for (what, data) in [
        ("one full block", text[..850_000].to_vec()),
        ("one small block", text[..1_000].to_vec()),
        ("empty stream", Vec::new()),
    ] {
        let z = compress(&data, 9);
        same_as_default(&z, what);
        let (got, res) = svc(&z, (1, 2, 1, 2));
        res.unwrap();
        assert!(got == data, "{what}");
    }
    let (got, res) = svc(&[], (3, 3, 1, 12));
    assert!(got.is_empty());
    assert_eq!(res.unwrap_err().kind, ErrorKind::Empty);
}

/// A stored block CRC flipped: the error at that block and bit, every block before it
/// delivered
#[test]
fn a_block_crc_in_the_middle() {
    // Source text can cross the generator's repeat threshold as code grows.
    // Leave enough room after RLE1 for the four blocks this fixture requires.
    let corp = corpora(2);
    let text = &corp[0].1[..];
    let z = compress(text, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 4);
    for at in [0, 1, 2, starts.len() - 1] {
        let mut t = z.clone();
        let bit = starts[at] + 48 + 5;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        same_as_default(&t, &format!("CRC of block {at}"));
        let (got, res) = svc(&t, (3, 3, 1, 12));
        let e = res.unwrap_err();
        assert!(matches!(e.kind, ErrorKind::BlockCrc { .. }), "{e}");
        assert_eq!((e.block, e.bit), (Some(at as u64), Some(starts[at])));
        assert!(text.starts_with(&got));
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
    let (got, res) = svc(&z, (2, 3, 1, 6));
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
        let (_, res) = svc(&z[..cut], (2, 2, 1, 4));
        assert_eq!(res.unwrap_err().kind, ErrorKind::Truncated, "cut at {cut}");
    }
}

/// A block whose move-to-front fails (a RUNA run too long) after two full blocks: the
/// service publishes its error while the blocks before it are still with the teams; the
/// reader still gets their bytes first
#[test]
fn an_mtf_error() {
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
    same_as_default(&z, "MTF error");
    for _ in 0..10 {
        let (got, res) = svc(&z, (1, 3, 2, 8));
        assert!(got == want);
        assert_eq!(res, want_res);
    }
}

/// A stream of several full blocks, and the default's bytes of its first `at` blocks
fn full_blocks() -> (Vec<u8>, Vec<u64>) {
    // Keep the four-block reach assertion independent of the source-copy count.
    let corp = corpora(2);
    let z = compress(&corp[0].1, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 4);
    (z, starts)
}

/// Each injected panic: the block's `Internal` error at its place and bit, the blocks
/// before it delivered as the default delivers them, the decoder ending (no hang), and the
/// same decoder shape decoding the whole stream right afterwards
#[test]
fn injected_panics() {
    let (z, starts) = full_blocks();
    let (all, ok) = default(&z);
    ok.unwrap();
    let corp = corpora(1);
    // (a small block, decoded whole by a team's first thread, at place 1 of this stream)
    let mut mixed = compress(&corp[0].1[..850_000], 9);
    mixed.extend(compress(&corp[0].1[..20_000], 9));
    mixed.extend(compress(&corp[0].1[..850_000], 9));
    let mixed_starts = block_starts(&mixed);
    let cases: Vec<(&str, &[u8], &[u64], ServiceFault)> = vec![
        (
            "a helper's walk",
            &z,
            &starts,
            ServiceFault::Team {
                place: 1,
                thread: 1,
            },
        ),
        (
            "a team's first thread's walk",
            &z,
            &starts,
            ServiceFault::Team {
                place: 2,
                thread: 0,
            },
        ),
        (
            "a small block's inversion",
            &mixed,
            &mixed_starts,
            ServiceFault::Team {
                place: 1,
                thread: 0,
            },
        ),
        (
            "the service's preparing",
            &z,
            &starts,
            ServiceFault::Prepare { place: 1 },
        ),
        (
            "the service's finishing",
            &z,
            &starts,
            ServiceFault::Finish { place: 2 },
        ),
        (
            "the first block's finishing",
            &z,
            &starts,
            ServiceFault::Finish { place: 0 },
        ),
    ];
    for (what, z, starts, fault) in cases {
        let (want, _) = default(z);
        let place = match fault {
            ServiceFault::Prepare { place }
            | ServiceFault::Finish { place }
            | ServiceFault::Team { place, .. } => place,
        };
        for s in [(1, 3, 1, 4), (3, 3, 1, 12), (2, 2, 2, 8)] {
            let (got, res) = svc_fault(z, s, Some(fault));
            let e = res.unwrap_err();
            assert!(
                matches!(&e.kind, ErrorKind::Internal(m) if m.contains("injected")),
                "{what} {s:?}: {e}"
            );
            assert_eq!(
                (e.block, e.bit),
                (Some(place), Some(starts[place as usize])),
                "{what}"
            );
            // the bytes before it: the default's blocks before `place`
            assert!(want.starts_with(&got), "{what} {s:?}");
            if place == 0 {
                assert!(got.is_empty(), "{what}");
            } else {
                assert!(
                    got.len() >= 700_000 * place as usize,
                    "{what}: {}",
                    got.len()
                );
            }
        }
    }
    // and without the fault, the whole stream
    let (got, res) = svc(&z, (3, 3, 1, 12));
    res.unwrap();
    assert!(got == all);
}

/// The reader dropped mid-stream, at every shape, and the push form cancelled while its
/// writer writes: nothing hangs (the watchdog)
#[test]
fn dropped_and_cancelled() {
    let corp = corpora(1);
    let data = corp[0].1.clone();
    let z = compress(&data, 9);
    for (teams, k, services, slots) in SHAPES {
        let (z, data) = (z.clone(), data.clone());
        watchdog(&format!("drop at {teams} x {k} + {services}"), move || {
            let mut r = mibz2::BzReader::parallel_service(
                std::io::Cursor::new(z),
                teams,
                k,
                services,
                slots,
                Kernels::detect(),
            );
            let mut buf = vec![0u8; 1_500_000];
            r.read_exact(&mut buf).unwrap();
            assert!(buf == data[..buf.len()]);
            drop(r);
        });
    }
    for services in [1, 2] {
        let z = z.clone();
        watchdog(&format!("cancel with {services} services"), move || {
            let dec = StreamDecoder::service(2, 3, services, 6, Kernels::detect());
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
        });
    }
    // a decoder dropped before any input, and one dropped with its input never closed
    watchdog("dropped idle", || {
        drop(StreamDecoder::service(3, 3, 2, 12, Kernels::detect()))
    });
    // and one dropped while its scanner waits for input never closed
    let z2 = z.clone();
    watchdog("dropped mid-input", move || {
        let dec = StreamDecoder::service(3, 3, 1, 12, Kernels::detect());
        dec.write(z2[..z2.len() / 2].to_vec()).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        drop(dec);
    });
}
