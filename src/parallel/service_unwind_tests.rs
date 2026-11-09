//! The service core's outer unwind (Guardian D067): a service-core thread that panics
//! outside every stage's catch, at each point where it can (`OuterAt`: holding nothing, a
//! job just taken, a block before and after each hand-off, a team before its pool is
//! chosen, a walked block just taken, a result published but not retired, and the hub's
//! feeder). The guard (`Outer`) fails the decode as a whole (`Hub::fail`), and each test
//! checks, under a watchdog:
//! 1. **shutdown:** the reader's read ends, the writer's writes end, and `join` (or the
//!    decoder's drop) ends; every one of the hub's threads has ended;
//! 2. **the reader:** its bytes are a prefix of the stream's (only what it already had:
//!    the ready blocks are dropped, as a cancel drops them), then the terminal error
//!    (`Internal`, naming the thread), or, where the fault came after the last block was
//!    read, the whole stream and no error; with a cancel racing the fault, `Cancelled`
//!    may come instead;
//! 3. **ownership:** no live block lost or retired twice (`live` back to 0, as many
//!    retirements as blocks taken), and no block published twice;
//! 4. **the fault fired**, so the path was reached.
//!
//! The publication history (each `publish`, cfg(test)) is read after the decoder is joined
//! or dropped, so the no-duplicate check covers every publication before its threads ended.
//! The full-block suite asserts a team's pool was reached across all its runs together, not
//! in each injection's run.
//!
//! Shapes: one team and one service worker (so the faulting thread is the last of its
//! kind), and svc2's two and two; tiny blocks (each team's first thread alone) and full
//! ones (a team's pool). And: a parked-state handshake (every other hub thread parked in
//! its wait when the fault fires, so only `Hub::fail`'s wake-up can end them), a writer
//! blocked on a full window, a cancel racing the fault, and the decoder dropped rather than
//! joined. Miri runs the tiny-block ones it can interpret (`miri_outer_*`).

use super::*;
use std::time::Duration;

const WATCHDOG: Duration = Duration::from_secs(if cfg!(miri) { 600 } else { 60 });

fn kernels() -> Kernels {
    if cfg!(miri) {
        Kernels::scalar()
    } else {
        Kernels::detect()
    }
}

/// Every outer point, each at block 1 (or, holding nothing, the first time)
const ALL: [OuterAt; 9] = [
    OuterAt::SvcIdle,
    OuterAt::AfterJob,
    OuterAt::BeforePrepared,
    OuterAt::AfterPrepared,
    OuterAt::AfterTake,
    OuterAt::AfterWalked,
    OuterAt::AfterFinishTake,
    OuterAt::AfterPublish,
    OuterAt::Feeder,
];

fn fault(at: OuterAt, place: u64, parked: usize) -> OuterFault {
    OuterFault {
        at,
        place: (at != OuterAt::SvcIdle).then_some(place),
        parked,
    }
}

/// Full blocks, each its own stream: text at level 3 (four blocks of 300 kB rows)
#[cfg(not(miri))]
fn full() -> (Vec<u8>, Vec<u8>) {
    use std::io::Write;
    let mut x = 0x5e41_u64 | 1;
    let data: Vec<u8> = (0..1_150_000)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            if i % 61 == 0 {
                b' '
            } else {
                b'a' + (x % 26) as u8
            }
        })
        .collect();
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(3));
    e.write_all(&data).unwrap();
    (e.finish().unwrap(), data)
}

fn tiny() -> (Vec<u8>, Vec<u8>) {
    (service_tests::Z.to_vec(), service_tests::want())
}

#[derive(Clone, Copy, Default)]
struct How {
    /// the reader reads only once the fault has fired (a writer blocked meanwhile)
    read_after: bool,
    /// a thread cancels the decode as soon as the fault has fired, racing `Hub::fail`
    cancel_race: bool,
    /// drop the decoder instead of joining it
    drop_only: bool,
    /// the write size
    chunk: usize,
}

/// What a run left, for `check`
struct Ran {
    out: Vec<u8>,
    res: Result<(), Error>,
    wrote: Result<(), Stopped>,
    fired: bool,
    failed: bool,
    live: usize,
    tokens: (u64, u64),
    exited: (usize, usize),
    published: Vec<u64>,
    /// blocks inverted on a team's pool
    pooled: u64,
}

/// The service core with `f` at shape `(teams, k, services, slots)`, decoding `z`, driven
/// as `how` says, all under the watchdog
fn run(f: OuterFault, shape: (usize, usize, usize, usize), z: &[u8], how: How) -> Ran {
    let (teams, k, services, slots) = shape;
    let z = z.to_vec();
    let what = format!("{f:?} at {shape:?}");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let dec = Arc::new(StreamDecoder::service_built(
            teams,
            k,
            services,
            slots,
            kernels(),
            None,
            Some(f),
        ));
        let hub = dec.hub.clone().unwrap();
        let fired = |h: &Hub| {
            let deadline = std::time::Instant::now() + WATCHDOG;
            while !h.fired.load(Ordering::Acquire) {
                if std::time::Instant::now() > deadline {
                    return false;
                }
                std::thread::yield_now();
            }
            true
        };
        let r = std::thread::scope(|s| {
            let writer = s.spawn(|| {
                for c in z.chunks(how.chunk.max(1)) {
                    dec.write(c.to_vec())?;
                }
                dec.close();
                Ok(())
            });
            let canceller = how.cancel_race.then(|| {
                s.spawn(|| {
                    if fired(&hub) {
                        dec.cancel();
                    }
                })
            });
            if how.read_after {
                fired(&hub);
            }
            let mut out = Vec::new();
            let mut buf = vec![0u8; 1 << 16];
            let res = loop {
                match dec.read(&mut buf) {
                    Ok(0) => break Ok(()),
                    Ok(n) => out.extend_from_slice(&buf[..n]),
                    Err(e) => break Err(e),
                }
            };
            // (a writer still blocked once the reader is done: the decode's stop must end
            // it, as a reader gone ends a pipe's writer)
            if res.is_err() {
                dec.cancel();
            }
            let wrote = writer.join().unwrap();
            if let Some(c) = canceller {
                c.join().unwrap();
            }
            (out, res, wrote)
        });
        let (out, res, wrote) = r;
        // (the publication history is read after the join or the drop below, so it holds
        // every publication any of the decoder's threads made before it ended)
        let shared = dec.shared.clone();
        if how.drop_only {
            // (the last reference: the drop joins every thread)
            let dec = Arc::try_unwrap(dec).ok().expect("the only reference");
            drop(dec);
        } else {
            dec.join();
        }
        let published = shared
            .faults
            .lock()
            .unwrap()
            .published
            .iter()
            .map(|p| p.0)
            .collect();
        let st = hub.lock();
        let _ = tx.send(Ran {
            out,
            res,
            wrote,
            fired: hub.fired.load(Ordering::Acquire),
            failed: st.failed,
            live: st.live,
            tokens: st.tokens,
            exited: (st.exited, hub.threads),
            published,
            pooled: st.counts.pooled,
        });
    });
    rx.recv_timeout(WATCHDOG)
        .unwrap_or_else(|_| panic!("{what}: hung (the read, the writer or the join)"))
}

fn check(r: &Ran, want: &[u8], what: &str, cancel_race: bool) {
    assert!(
        r.fired,
        "{what}: the fault never fired (the path not reached)"
    );
    assert!(r.failed, "{what}: the hub never failed");
    assert_eq!(
        r.exited.0, r.exited.1,
        "{what}: the hub's threads all ended"
    );
    assert_eq!(r.live, 0, "{what}: a live block lost");
    assert_eq!(r.tokens.0, r.tokens.1, "{what}: blocks taken, retired");
    let mut p = r.published.clone();
    p.sort_unstable();
    let n = p.len();
    p.dedup();
    assert_eq!(
        p.len(),
        n,
        "{what}: a block published twice: {:?}",
        r.published
    );
    assert!(
        r.out.len() <= want.len() && r.out[..] == want[..r.out.len()],
        "{what}: not a prefix of the stream"
    );
    match &r.res {
        Ok(()) => assert!(r.out == want, "{what}: ended early without an error"),
        Err(e) => match &e.kind {
            ErrorKind::Internal(m) => {
                assert!(m.contains("outside its stages"), "{what}: {m}")
            }
            ErrorKind::Cancelled if cancel_race => {}
            k => panic!("{what}: {k:?}"),
        },
    }
}

/// Every outer point, at the last-of-its-kind shape and at svc2's, on `input`
fn every_point(input: (Vec<u8>, Vec<u8>)) -> u64 {
    let (z, want) = input;
    let mut pooled = 0;
    for shape in [(1, 3, 1, 4), (2, 3, 2, 8)] {
        for at in ALL {
            let what = format!("{at:?} at {shape:?}");
            let how = How {
                chunk: 100_003,
                ..How::default()
            };
            let r = run(fault(at, 1, 0), shape, &z, how);
            check(&r, &want, &what, false);
            pooled += r.pooled;
        }
    }
    pooled
}

#[cfg(not(miri))]
#[test]
fn every_outer_point_fails_the_decode_full_blocks() {
    // (full blocks reach a team's pool: asserted across the whole suite, not for each
    // injection, so this shows pooled execution in these runs, not that every fault fired
    // while a team's helpers were active)
    assert!(every_point(full()) > 0, "no block on a team's pool");
}

#[cfg(not(miri))]
#[test]
fn every_outer_point_fails_the_decode_tiny_blocks() {
    assert_eq!(every_point(tiny()), 0);
}

/// The handshake: every other hub thread parked in its wait, the jobs closed, when the
/// fault fires. One service worker (the last), two teams parked for a prepared block:
/// only `Hub::fail`'s own wake-up ends them. The same at a team's take (the last team, a
/// service worker parked).
fn parked(z: &[u8], want: &[u8], last: u64) {
    for (at, shape, others) in [
        (OuterAt::AfterJob, (2, 3, 1, 8), 2),
        (OuterAt::AfterTake, (1, 3, 1, 8), 1),
    ] {
        let what = format!("{at:?} parked {others}");
        let how = How {
            chunk: 1 << 20,
            ..How::default()
        };
        let r = run(fault(at, last, others), shape, z, how);
        check(&r, want, &what, false);
    }
}

#[cfg(not(miri))]
#[test]
fn a_parked_hub_wakes_on_the_failure() {
    // (the full input's last block: 1.15 MB at level 3's 300 kB blocks)
    let (z, want) = full();
    parked(&z, &want, 3);
    let (z, want) = tiny();
    parked(&z, &want, 3);
}

/// A writer blocked (a window of one block, the reader not yet reading), a cancel racing
/// the failure, and the decoder dropped, not joined
#[cfg(not(miri))]
#[test]
fn a_blocked_writer_a_racing_cancel_and_a_drop() {
    let (z, want) = full();
    for (what, at, how) in [
        (
            "a blocked writer",
            OuterAt::AfterPrepared,
            How {
                read_after: true,
                chunk: 4093,
                ..How::default()
            },
        ),
        (
            "a racing cancel",
            OuterAt::AfterTake,
            How {
                cancel_race: true,
                chunk: 100_003,
                ..How::default()
            },
        ),
        (
            "dropped",
            OuterAt::AfterPublish,
            How {
                drop_only: true,
                chunk: 100_003,
                ..How::default()
            },
        ),
    ] {
        for shape in [(1, 3, 1, 1), (2, 3, 2, 8)] {
            let what = format!("{what} at {shape:?}");
            let r = run(fault(at, 0, 0), shape, &z, how);
            check(&r, &want, &what, how.cancel_race);
            if how.read_after {
                assert!(r.res.is_err(), "{what}: the reader read after the failure");
            }
            let _ = r.wrote;
        }
    }
}

// (Miri: the tiny blocks, scalar kernels, one service worker and one team, so the faulting
// thread is the last of its kind; and the parked handshake)
#[test]
fn miri_outer_after_job() {
    let (z, want) = tiny();
    let how = How {
        chunk: 64,
        ..How::default()
    };
    let r = run(fault(OuterAt::AfterJob, 1, 0), (1, 2, 1, 2), &z, how);
    check(&r, &want, "after a job (Miri)", false);
}

#[test]
fn miri_outer_after_publish() {
    let (z, want) = tiny();
    let how = How {
        chunk: 64,
        ..How::default()
    };
    let r = run(fault(OuterAt::AfterPublish, 1, 0), (1, 2, 1, 2), &z, how);
    check(&r, &want, "after a publish (Miri)", false);
}

#[test]
fn miri_outer_parked() {
    let (z, want) = tiny();
    let how = How {
        chunk: 1 << 20,
        ..How::default()
    };
    // (a window of more blocks than the stream's four, so the scanner reaches its end and
    // the jobs close before the handshake)
    let r = run(fault(OuterAt::AfterJob, 3, 2), (2, 2, 1, 8), &z, how);
    check(&r, &want, "parked (Miri)", false);
}
