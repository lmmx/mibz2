//! The service core as the default at its one target (`parallel::service_target`): the
//! public default constructors (`StreamDecoder::new`, `BzReader::parallel`,
//! `BzReader::parallel_with`, `decode_parallel`) reaching it by their own dispatch, with only
//! the machine's facts given (`parallel::with_machine`: the measured i9-10900K's), never the
//! plan forced; and reaching the existing plan at every other budget, on the scalar kernels
//! and off the target. Each build is observed (`parallel::observe_builds`): which decoder
//! the dispatch chose, its threads and window, and, for the service core, its counts once
//! the decode is done (blocks prepared, inverted on a team's pool, finished), so a test
//! sees the service core's own threads did the work, on full blocks. The output is
//! libbzip2's byte for byte, and a bad stream fails as the plain decoder fails it (kind,
//! block, bit, and the bytes before). All under a watchdog.

mod common;
use common::*;
use mibz2::parallel::{observe_builds, with_machine, with_plan, Built, Machine, PIPELINED_PLAN};
use mibz2::{BzReader, Error, Kernels, StreamDecoder};
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

/// Where the predicate can hold at all (it asks for x86-64 Linux)
fn supported() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
        && Kernels::detect().avx2
        && Kernels::detect().clmul
}

/// The default entry points, each driven to its end: its output, or the error as the
/// crate's `Error` (a reader's `io::Error` unwrapped)
#[derive(Clone, Copy, Debug)]
enum Entry {
    /// `StreamDecoder::new(threads, slots, detect)`, fed from another thread
    New { slots: usize },
    /// `BzReader::parallel(.., threads)`
    Parallel,
    /// `BzReader::parallel_with(.., threads, slots, detect)`
    ParallelWith { slots: usize },
    /// `decode_parallel(.., threads)`
    Whole,
}

fn unwrap_io(e: std::io::Error) -> Error {
    e.get_ref()
        .and_then(|x| x.downcast_ref::<Error>())
        .cloned()
        .unwrap_or_else(|| Error::new(mibz2::ErrorKind::Io(e.kind(), e.to_string())))
}

fn entry(e: Entry, z: &[u8], threads: usize) -> (Vec<u8>, Result<(), Error>) {
    let k = Kernels::detect();
    match e {
        Entry::New { slots } => {
            let dec = StreamDecoder::new(threads, slots, k);
            let r = drive(&dec, z, 1 << 20, 1 << 16);
            dec.join(); // (the completion fence)
            r
        }
        Entry::Parallel | Entry::ParallelWith { .. } => {
            let src = std::io::Cursor::new(z.to_vec());
            let mut r = match e {
                Entry::ParallelWith { slots } => BzReader::parallel_with(src, threads, slots, k),
                _ => BzReader::parallel(src, threads),
            };
            assert!(r.parallel_decoder().is_some());
            let mut out = Vec::new();
            let res = r.read_to_end(&mut out).map(|_| ()).map_err(unwrap_io);
            // (the completion fence before the counts are read: EOF is not the workers'
            // end, and dropping the reader may leave its feeder holding the decoder)
            r.parallel_decoder().unwrap().join();
            (out, res)
        }
        // (decode_parallel's decoder is dropped, so joined, before it returns)
        Entry::Whole => match mibz2::decode_parallel(z, threads) {
            Ok(v) => (v, Ok(())),
            // (decode_parallel returns no bytes with its error: the test compares only the
            // error then)
            Err(e) => (Vec::new(), Err(e)),
        },
    }
}

/// `e` at `threads` on machine `m`, under the watchdog: its output and result, and what the
/// dispatch built
fn on(m: Machine, e: Entry, z: &[u8], threads: usize) -> ((Vec<u8>, Result<(), Error>), Built) {
    let z = z.to_vec();
    watchdog(&format!("{e:?} at {threads}"), move || {
        let (r, b) = observe_builds(|| with_machine(m, || entry(e, &z, threads)));
        assert_eq!(b.len(), 1, "{e:?}: one decoder built");
        (r, b[0])
    })
}

const ENTRIES: [Entry; 4] = [
    Entry::New { slots: 40 },
    Entry::Parallel,
    Entry::ParallelWith { slots: 40 },
    Entry::Whole,
];

/// Every default entry point at 10 threads on the target: the service core, at 8 decoding
/// threads and a window of 8, its threads doing every block (each prepared and finished
/// by the service, the full ones inverted on a team's pool), libbzip2's bytes
#[test]
fn the_entry_points_reach_the_service_core() {
    if !supported() {
        return;
    }
    let corp = corpora(1);
    let text = &corp[0].1;
    let inputs: Vec<(String, Vec<u8>, Vec<u8>)> = vec![
        ("text level 9".into(), text[..2_700_000].to_vec()),
        ("text level 3".into(), text[..3_000_000].to_vec()),
        ("records level 6".into(), corp[5].1[..1_700_000].to_vec()),
    ]
    .into_iter()
    .zip([9, 3, 6])
    .map(|((n, d), l)| (n, compress(&d, l), d))
    .collect();
    for (name, z, data) in &inputs {
        assert!(oracle(z).unwrap() == *data, "{name}: libbzip2");
        let blocks = block_starts(z).len() as u64;
        assert!(blocks >= 3, "{name}: {blocks} blocks");
        for e in ENTRIES {
            let ((got, res), b) = on(Machine::service_target(), e, z, 10);
            res.unwrap_or_else(|err| panic!("{name}: {e:?}: {err}"));
            assert!(got == *data, "{name}: {e:?}: the bytes");
            assert!(b.service && !b.pipelined, "{name}: {e:?}: {b:?}");
            assert_eq!((b.threads, b.window), (8, 8), "{name}: {e:?}");
            let c = b.counts.expect("the service core's counts");
            assert!(c.ended, "{name}: {e:?}: the completion fence");
            assert_eq!((c.prepared, c.finished), (blocks, blocks), "{name}: {e:?}");
            assert!(c.pooled >= blocks - 1, "{name}: {e:?}: {c:?}");
        }
    }
}

/// A caller's own window, scaled to the service core's two teams as a plan's is (at least
/// a block a team): `StreamDecoder::new` and `BzReader::parallel_with`
#[test]
fn custom_windows_scale_to_the_teams() {
    if !supported() {
        return;
    }
    let corp = corpora(1);
    let data = corp[0].1[..2_000_000].to_vec();
    let z = compress(&data, 5);
    for (slots, want) in [(1, 2), (10, 2), (20, 4), (40, 8), (80, 16), (200, 40)] {
        for e in [Entry::New { slots }, Entry::ParallelWith { slots }] {
            let ((got, res), b) = on(Machine::service_target(), e, &z, 10);
            res.unwrap();
            assert!(got == data, "{e:?}");
            assert!(b.service, "{e:?}");
            assert_eq!(b.window, want, "{e:?}");
        }
    }
}

/// The existing plan everywhere else, by the same dispatch: 9 and 11 threads on the target
/// (the plan policy's teams of two, and the pipelined three teams of three), 10 threads off
/// it (a CPU of the same shape but another model: the existing 10-core rule's pipelined
/// teams; a smaller machine: workers), and a plan forced on the target
#[test]
fn the_existing_plan_otherwise() {
    if !supported() {
        return;
    }
    let corp = corpora(1);
    let data = corp[0].1[..2_000_000].to_vec();
    let z = compress(&data, 9);
    let i9 = Machine::service_target();
    let mut same_shape = i9.clone();
    same_shape.target.as_mut().unwrap().cpu = ("GenuineIntel".into(), 6, 151);
    let small = Machine {
        phys: Some(4),
        target: None,
        ..i9.clone()
    };
    let unknown_quota = Machine {
        quota: mibz2::parallel::Quota::Unknown,
        ..i9.clone()
    };
    // (machine, threads, pipelined, decoding threads)
    let cases = [
        (i9.clone(), 9, false, 8),
        (i9.clone(), 11, true, 9),
        (i9.clone(), 20, true, 9),
        (same_shape, 10, true, 9),
        (unknown_quota, 10, true, 9),
        (small, 10, false, 10),
    ];
    for (m, t, pipelined, threads) in cases {
        for e in ENTRIES {
            let ((got, res), b) = on(m.clone(), e, &z, t);
            res.unwrap();
            assert!(got == data, "{m:?} {t} {e:?}");
            assert!(!b.service && b.counts.is_none(), "{m:?} {t} {e:?}: {b:?}");
            assert_eq!((b.pipelined, b.threads), (pipelined, threads), "{t} {e:?}");
        }
    }
    // a forced plan wins on the target
    for e in ENTRIES {
        let z = z.clone();
        let (_, b) = watchdog("forced", move || {
            observe_builds(|| {
                with_machine(Machine::service_target(), || {
                    with_plan(PIPELINED_PLAN, || entry(e, &z, 10))
                })
            })
        });
        assert!(!b[0].service && b[0].pipelined, "{e:?}");
    }
}

/// Bad streams through the default entry points on the target fail as the plain decoder
/// fails them: the same error (kind, block, bit) and the same bytes before it
#[test]
fn errors_through_the_entry_points() {
    if !supported() {
        return;
    }
    let corp = corpora(1);
    let text = &corp[0].1;
    let z = compress(&text[..3_000_000], 3);
    let starts = block_starts(&z);
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();
    // a block's stored CRC flipped (the block's 48 header bits, then its 32-bit CRC)
    for b in [0, 2, starts.len() - 1] {
        let mut bad = z.clone();
        let at = (starts[b] + 48 + 5) as usize;
        bad[at / 8] ^= 1 << (7 - at % 8);
        cases.push((format!("block {b}'s CRC"), bad));
    }
    // truncated mid-block, and mid-way
    cases.push(("truncated".into(), z[..z.len() * 2 / 3].to_vec()));
    cases.push(("truncated at the end".into(), z[..z.len() - 3].to_vec()));
    for (what, bad) in &cases {
        let (want, want_res) = plain_parallel(bad);
        assert!(want_res.is_err(), "{what}");
        for e in ENTRIES {
            let ((got, res), b) = on(Machine::service_target(), e, bad, 10);
            assert!(b.service, "{what}: {e:?}");
            assert_eq!(res, want_res, "{what}: {e:?}: the error");
            if !matches!(e, Entry::Whole) {
                assert!(got == want, "{what}: {e:?}: the bytes before");
            }
        }
    }
}
