//! The run-fed team's run that panics outside its stages' own catches (Guardian D049 §5,
//! D047's M12 on this module's copied worker): the panic unwinds through the pool's run
//! into `worker`'s outer `Err`, which must retire every block the run still holds, each
//! exactly once, with its own place, bit and an `Internal` error, and reset the team, so
//! that the reader gets the ordered prefix, then the error, and never waits on a place
//! nobody will fill.
//!
//! The twin of the shipped worker's tests (`parallel::unwind_tests`, #275), on the same
//! hook (`parallel::Fault`, `parallel::Faults` in `Shared::faults`, `parallel::inject` in the
//! leader's part of the run while the helpers build and walk the block, `parallel::publish`
//! recording every publication): the same three states, the same checks, on
//! `worker::<MODE>` for every `MODE` (the two controls, the leader's sort, the
//! helpers' sort and the direct fill). The tests drive the real `worker` on jobs queued in
//! full before it starts (so `try_job` finds the next one at every run: the team's state at
//! each run is fixed), except the last, which runs `StreamDecoder::pipelined_run_fed` end to
//! end from the scanner. Every wait is under a watchdog.

use super::super::{Error, Event, Faults, Reader, Scanner, State};
use super::*;
use crate::bwt::MIN_COOP;
use std::collections::BTreeMap;
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar};
use std::time::Duration;

/// How long a read, the retirements or the join may take before the test fails, and how
/// often the retirements are polled (`parallel::unwind_tests`' own)
const WATCHDOG: Duration = Duration::from_secs(if cfg!(miri) { 600 } else { 60 });
const POLL: Duration = Duration::from_millis(if cfg!(miri) { 50 } else { 1 });

fn kernels() -> Kernels {
    if cfg!(miri) {
        Kernels::scalar()
    } else {
        Kernels::detect()
    }
}

/// Each sub-arm's worker, with its name
type Worker = fn(&Shared, &Mutex<Receiver<Job>>, Kernels, usize);

const WORKERS: [(&str, Worker); 5] = [
    ("Control", worker::<CTL>),
    ("RegControl", worker::<CTLR>),
    ("Leader", worker::<LS>),
    ("Helpers", worker::<HS>),
    ("Direct", worker::<DIR>),
];

/// Text of phrases each repeated many times, `n` bytes: its BWT is runs, so a full block
/// of it is one the records' rule takes (`takes_runs`), and RLE1 leaves it at its length
#[cfg(not(miri))]
fn runny(seed: u64, n: usize) -> Vec<u8> {
    const WORDS: [&[u8]; 8] = [
        b"alpha ",
        b"bravo ",
        b"charlie ",
        b"delta ",
        b"echo ",
        b"foxtrot ",
        b"golf ",
        b"hotel ",
    ];
    let mut x = seed | 1;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let mut out = Vec::with_capacity(n + 512);
    while out.len() < n {
        let words = 2 + next() % 5;
        let phrase: Vec<u8> = (0..words)
            .flat_map(|_| WORDS[(next() % 8) as usize].iter().copied())
            .collect();
        for _ in 0..8 + next() % 40 {
            out.extend_from_slice(&phrase);
        }
    }
    out.truncate(n);
    out
}

/// Each chunk its own bz2 stream (so its own block), concatenated
#[cfg(not(miri))]
fn streams(chunks: &[Vec<u8>]) -> Vec<u8> {
    use std::io::Write;
    let mut all = Vec::new();
    for c in chunks {
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(9));
        e.write_all(c).unwrap();
        all.extend(e.finish().unwrap());
    }
    all
}

use super::super::unwind_tests::{tiny_chunk, TINY};

/// A block's job, as the scanner would give it: (place, first bit, symbols)
fn jobs_of(bz: &[u8]) -> Vec<Job> {
    let mut sc = Scanner::new(crate::stream::SliceSource(bz));
    let mut jobs = Vec::new();
    loop {
        match sc.next(Vec::new()).unwrap() {
            Event::Block {
                index,
                bit,
                symbols,
                ..
            } => jobs.push((index, bit, symbols)),
            Event::Done => return jobs,
        }
    }
}

/// `f` on the decode's faults (`Shared::faults`)
fn with<R>(dec: &StreamDecoder, f: impl FnOnce(&mut Faults) -> R) -> R {
    f(&mut dec.shared.faults.lock().unwrap())
}

/// One run-fed team (`work`, one sub-arm's worker) on `jobs`, all queued before it starts,
/// the scanner's part done (every place issued, the end known), with `fault` to inject
fn team(
    work: Worker,
    jobs: Vec<Job>,
    team_k: usize,
    fault: (u64, Fault),
    min_coop: Option<usize>,
) -> Arc<StreamDecoder> {
    let n = jobs.len() as u64;
    let shared = Arc::new(Shared {
        state: Mutex::new(State {
            ready: BTreeMap::new(),
            issued: n,
            taken: 0,
            end: Some(n),
            cancelled: false,
            free_out: Vec::new(),
            free_syms: Vec::new(),
        }),
        cv: Condvar::new(),
        slots: n.max(1),
        overlapped: AtomicU64::new(0),
        overlap_errors: AtomicU64::new(0),
        faults: Mutex::new(Faults {
            at: Some(fault),
            min_coop,
            ..Faults::default()
        }),
    });
    let (tx, rx) = channel::<Job>();
    for j in jobs {
        tx.send(j).unwrap();
    }
    drop(tx);
    let (sh, k) = (shared.clone(), kernels());
    let h = std::thread::Builder::new()
        .name("mibz2-0".into())
        .spawn(move || work(&sh, &Mutex::new(rx), k, team_k))
        .unwrap();
    Arc::new(StreamDecoder {
        input: Mutex::new(None),
        shared,
        handles: Mutex::new(vec![h]),
        reader: Mutex::new(Reader::default()),
        threads: team_k,
        pipelined: true,
    })
}

/// The whole read, under the watchdog: the bytes, then the error if any. On the watchdog,
/// the decoder is cancelled (so the reader thread ends) and the test fails.
fn read_all(dec: &Arc<StreamDecoder>, what: &str) -> (Vec<u8>, Option<Error>) {
    let (tx, rx) = channel();
    let d = dec.clone();
    let h = std::thread::spawn(move || {
        let (mut out, mut buf) = (Vec::new(), vec![0u8; 1 << 16]);
        let err = loop {
            match d.read(&mut buf) {
                Ok(0) => break None,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => break Some(e),
            }
        };
        let _ = tx.send((out, err));
    });
    match rx.recv_timeout(WATCHDOG) {
        Ok(r) => {
            h.join().unwrap();
            r
        }
        Err(_) => {
            let st = dec.shared.lock();
            let (taken, ready) = (st.taken, st.ready.keys().copied().collect::<Vec<_>>());
            drop(st);
            dec.cancel();
            panic!("{what}: watchdog: the reader is stuck at place {taken}, with places {ready:?} ready");
        }
    }
}

type Published = Vec<(u64, Option<Error>)>;

/// Waits, under the watchdog, until `n` blocks have been published
fn all_published(dec: &StreamDecoder, n: usize, what: &str) -> Published {
    let until = std::time::Instant::now() + WATCHDOG;
    loop {
        let got = with(dec, |f| f.published.clone());
        if got.len() >= n {
            return got;
        }
        if std::time::Instant::now() > until {
            let places: Vec<u64> = got.iter().map(|p| p.0).collect();
            panic!(
                "{what}: watchdog: {} of {n} blocks published, places {places:?}",
                places.len()
            );
        }
        std::thread::sleep(POLL);
    }
}

/// `dec.join()` under the watchdog
fn join(dec: &Arc<StreamDecoder>, what: &str) {
    let (tx, rx) = channel();
    let d = dec.clone();
    let h = std::thread::spawn(move || {
        d.join();
        let _ = tx.send(());
    });
    rx.recv_timeout(WATCHDOG)
        .unwrap_or_else(|_| panic!("{what}: watchdog: the join didn't end"));
    h.join().unwrap();
}

/// The decoder's threads end of themselves, under the watchdog, before any cancel
fn worker_ends(dec: &StreamDecoder, what: &str) {
    let handles = std::mem::take(&mut *dec.handles.lock().unwrap());
    let (tx, rx) = channel();
    let h = std::thread::spawn(move || {
        for h in handles {
            h.join().unwrap();
        }
        let _ = tx.send(());
    });
    rx.recv_timeout(WATCHDOG)
        .unwrap_or_else(|_| panic!("{what}: watchdog: the worker didn't end"));
    h.join().unwrap();
}

/// A run's state when its fault fires: the prior inverted block held, the next staged
type Held = (Option<u64>, Option<u64>);

/// The checks of one faulted run, `fault` in a decode of `chunks` (each a block of its
/// own) whose state then must be `held`
fn check(
    dec: &Arc<StreamDecoder>,
    chunks: &[Vec<u8>],
    bits: &[u64],
    fault: (u64, Fault),
    held: Held,
    what: &str,
) {
    let n = chunks.len();
    let (place, _) = fault;
    // the blocks the run held: its own, and the prior and the next if held
    let mut owned: Vec<u64> = [Some(place), held.0, held.1]
        .into_iter()
        .flatten()
        .collect();
    owned.sort_unstable();
    let first = owned[0] as usize;
    let injected = |m: &str, at: u64| {
        assert!(
            m.starts_with(&format!("injected at block {place}, {:?}", fault.1)),
            "{what}: block {at}: {m}"
        );
    };

    // the reader: the ordered prefix, then the first held block's error
    let (got, err) = read_all(dec, what);
    assert!(
        got == chunks[..first].concat(),
        "{what}: not the prefix of {first} blocks ({} bytes)",
        got.len()
    );
    let e = err.unwrap_or_else(|| panic!("{what}: no error"));
    let ErrorKind::Internal(m) = &e.kind else {
        panic!("{what}: the reader's error is not Internal: {e:?}");
    };
    injected(m, first as u64);
    assert_eq!(
        (e.block, e.bit),
        (Some(first as u64), Some(bits[first])),
        "{what}: the reader's error's place and bit"
    );

    // the fault fired in the state asked for, and the outer branch recovered from it
    let published = all_published(dec, n, what);
    with(dec, |f| {
        assert_eq!(
            f.held,
            Some(held),
            "{what}: the team's state when the fault fired"
        );
        assert_eq!(f.recovered, 1, "{what}: the outer unwinds recovered");
        assert!(f.at.is_none(), "{what}: the fault never fired");
    });
    // every block published exactly once; the owned ones as Internal at their own place
    // and bit, the others not
    let mut places: Vec<u64> = published.iter().map(|p| p.0).collect();
    places.sort_unstable();
    assert_eq!(
        places,
        (0..n as u64).collect::<Vec<_>>(),
        "{what}: each block published once"
    );
    for (at, e) in &published {
        match (owned.contains(at), e) {
            (true, Some(e)) => {
                let ErrorKind::Internal(m) = &e.kind else {
                    panic!("{what}: block {at}: not Internal: {e:?}");
                };
                injected(m, *at);
                assert_eq!(
                    (e.block, e.bit),
                    (Some(*at), Some(bits[*at as usize])),
                    "{what}: block {at}'s place and bit"
                );
            }
            (true, None) => panic!("{what}: block {at}, held by the run, published as decoded"),
            (false, Some(e)) => panic!("{what}: block {at}, not held by the run: {e:?}"),
            (false, None) => {}
        }
    }
    // the worker ends of itself, its jobs done (the decoder not cancelled)
    worker_ends(dec, what);
    // the blocks after the reader's error, decoded on the reset team, are the right bytes
    let st = dec.shared.lock();
    for (at, r) in &st.ready {
        if let Ok(v) = r {
            assert!(
                *v == chunks[*at as usize],
                "{what}: block {at}'s bytes after the reset"
            );
        }
    }
    let after = st.ready.iter().filter(|(_, r)| r.is_ok()).count();
    drop(st);
    assert_eq!(
        after,
        n - 1 - owned[owned.len() - 1] as usize,
        "{what}: the blocks after the run, decoded"
    );
    join(dec, what);
}

/// A run's three states, each with its fault: the last block's run with nothing else held
/// (the prefix is the three before it)
const CURRENT: (&str, (u64, Fault), Held) = ("current", (3, Fault::LeadEnd), (None, None));
/// Block 1's run, block 0 already published by its leader, block 2 staged
const NEXT: (&str, (u64, Fault), Held) = ("current + next", (1, Fault::LeadEnd), (None, Some(2)));
/// Block 2's run, its leader yet to finish block 1: block 0 is the prefix
const PRIOR: (&str, (u64, Fault), Held) = (
    "current + held prior",
    (2, Fault::LeadStart),
    (Some(1), None),
);

/// A state on four full run-heavy blocks (above `MIN_COOP`, the team's own threshold, and
/// taken by the records' rule), every sub-arm, on teams of 2 and 3
#[cfg(not(miri))]
fn full((name, fault, held): (&str, (u64, Fault), Held)) {
    let chunks: Vec<Vec<u8>> = (0..4).map(|i| runny(41 + i, MIN_COOP + 10_000)).collect();
    let bz = streams(&chunks);
    let jobs = jobs_of(&bz);
    assert_eq!(jobs.len(), 4);
    for (_, _, s) in &jobs {
        assert!(api::takes_runs(s), "a block the records' rule takes");
    }
    let bits: Vec<u64> = jobs.iter().map(|j| j.1).collect();
    for (arm, work) in WORKERS {
        for team_k in [2, 3] {
            let what = format!("{arm}, {name}, k {team_k}");
            let dec = team(work, jobs_of(&bz), team_k, fault, None);
            check(&dec, &chunks, &bits, fault, held, &what);
        }
    }
}

#[cfg(not(miri))]
#[test]
fn a_run_panic_retires_the_current_block() {
    full(CURRENT);
}

#[cfg(not(miri))]
#[test]
fn a_run_panic_retires_the_current_and_next_blocks() {
    full(NEXT);
}

#[cfg(not(miri))]
#[test]
fn a_run_panic_retires_the_current_and_held_prior_blocks() {
    full(PRIOR);
}

/// A state on four tiny blocks (taken by the records' rule), on a team of two (one
/// helper, so the run's unwind waits on a live helper), its threshold lowered so that
/// they run the team: under Miri too, which checks the unwind's hand-offs between the
/// leader, the helper, the reset and the reader. Under Miri, the arms with a helpers'
/// sort or fill from records (the leader's sort's fill, the helpers' sort, the direct
/// fill); natively, every arm.
fn tiny((name, fault, held): (&str, (u64, Fault), Held)) {
    let chunks: Vec<Vec<u8>> = (0..4).map(tiny_chunk).collect();
    let jobs = jobs_of(&TINY);
    for (_, _, s) in &jobs {
        assert!(api::takes_runs(s));
    }
    let bits: Vec<u64> = jobs.iter().map(|j| j.1).collect();
    let arms = if cfg!(miri) {
        &WORKERS[2..]
    } else {
        &WORKERS[..]
    };
    for (arm, work) in arms {
        let what = format!("tiny, {arm}, {name}");
        let dec = team(*work, jobs_of(&TINY), 2, fault, Some(1));
        check(&dec, &chunks, &bits, fault, held, &what);
    }
}

#[test]
fn miri_run_fed_panic_current() {
    tiny(CURRENT);
}

#[test]
fn miri_run_fed_panic_current_and_next() {
    tiny(NEXT);
}

#[test]
fn miri_run_fed_panic_current_and_held_prior() {
    tiny(PRIOR);
}

/// `StreamDecoder::pipelined_run_fed` end to end, from the scanner, at the registered
/// plan's shape (three teams of three) and each sub-arm, a run panicking: which team runs
/// the block, and what it holds then, is up to the threads, so the error expected is at
/// the first block the run held
#[cfg(not(miri))]
#[test]
fn the_run_fed_decoder_recovers_from_a_run_panic() {
    let chunks: Vec<Vec<u8>> = (0..6).map(|i| runny(71 + i, MIN_COOP + 5_000)).collect();
    let bz = streams(&chunks);
    let bits: Vec<u64> = jobs_of(&bz).iter().map(|j| j.1).collect();
    for sort in [
        RunFedSort::Control,
        RunFedSort::RegControl,
        RunFedSort::Leader,
        RunFedSort::Helpers,
        RunFedSort::Direct,
    ] {
        for fault in [
            (2, Fault::LeadStart),
            (2, Fault::LeadEnd),
            (4, Fault::LeadStart),
        ] {
            let what = format!("{sort:?} 3x3, {fault:?}");
            let dec = Arc::new(StreamDecoder::pipelined_run_fed(3, 3, 16, kernels(), sort));
            assert!(dec.is_pipelined() && dec.threads() == 9);
            with(&dec, |f| f.at = Some(fault));
            let d = dec.clone();
            let bz2 = bz.clone();
            let writer = std::thread::spawn(move || {
                for c in bz2.chunks(1 << 20) {
                    if d.write(c.to_vec()).is_err() {
                        break;
                    }
                }
                d.close();
            });
            let (got, err) = read_all(&dec, &what);
            writer.join().unwrap();
            let held = with(&dec, |f| f.held);
            let held = held.unwrap_or_else(|| panic!("{what}: the fault never fired"));
            let first = [Some(fault.0), held.0, held.1]
                .into_iter()
                .flatten()
                .min()
                .unwrap() as usize;
            assert!(
                got == chunks[..first].concat(),
                "{what}: not the prefix of {first} blocks"
            );
            let e = err.unwrap_or_else(|| panic!("{what}: no error"));
            assert!(matches!(e.kind, ErrorKind::Internal(_)), "{what}: {e:?}");
            assert_eq!(
                (e.block, e.bit),
                (Some(first as u64), Some(bits[first])),
                "{what}"
            );
            let published = all_published(&dec, chunks.len(), &what);
            let mut places: Vec<u64> = published.iter().map(|p| p.0).collect();
            places.sort_unstable();
            assert_eq!(
                places,
                (0..chunks.len() as u64).collect::<Vec<_>>(),
                "{what}: each block once"
            );
            assert_eq!(with(&dec, |f| f.recovered), 1, "{what}");
            join(&dec, &what);
        }
    }
}

/// The safe constructor at team sizes about `MAX_SHARES` (Guardian D049, B6: k = 66 made
/// 65 helpers for 64 share entries): k = 64, 65, 66 and 67, every sub-arm, each kernel,
/// on two full run-heavy blocks that take the team (`team_records`): byte for byte the
/// input, the thread count advertised the threads spawned
#[cfg(not(miri))]
#[test]
fn the_constructor_past_the_shares() {
    let chunks: Vec<Vec<u8>> = (0..2).map(|i| runny(91 + i, MIN_COOP + 3_000)).collect();
    let bz = streams(&chunks);
    let all = chunks.concat();
    let mut kerns = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        kerns.push(Kernels::detect());
    }
    for k in [64, 65, 66, 67] {
        for sort in [
            RunFedSort::Control,
            RunFedSort::RegControl,
            RunFedSort::Leader,
            RunFedSort::Helpers,
            RunFedSort::Direct,
        ] {
            for &kern in &kerns {
                let what = format!("k {k}, {sort:?}, {kern:?}");
                let before = PH.team_records.load(Ordering::Relaxed);
                let dec = StreamDecoder::pipelined_run_fed(1, k, 2, kern, sort);
                assert!(dec.is_pipelined() && dec.threads() == k, "{what}");
                let mut got = Vec::new();
                std::thread::scope(|s| {
                    s.spawn(|| {
                        dec.write(bz.clone()).unwrap();
                        dec.close();
                    });
                    let mut buf = vec![0u8; 1 << 16];
                    loop {
                        match dec.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => got.extend_from_slice(&buf[..n]),
                            Err(e) => panic!("{what}: {e:?}"),
                        }
                    }
                });
                assert!(got == all, "{what}: the bytes");
                if !matches!(sort, RunFedSort::Control | RunFedSort::RegControl) {
                    // (a lower bound: other tests' runs add to the same totals)
                    assert!(
                        PH.team_records.load(Ordering::Relaxed) >= before + 2,
                        "{what}: the team by records"
                    );
                }
            }
        }
    }
}
