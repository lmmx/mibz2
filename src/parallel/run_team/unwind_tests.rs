//! The run-fed team's run that panics outside its stages' own catches (Guardian D049 §5,
//! D047's M12 on this module's copied worker): the panic unwinds through the pool's run
//! into `worker`'s outer `Err`, which must retire every block the run still holds, each
//! exactly once, with its own place, bit and an `Internal` error, and reset the team, so
//! that the reader gets the ordered prefix, then the error, and never waits on a place
//! nobody will fill.
//!
//! The twin of the shipped worker's tests (`parallel::unwind_tests`, branch
//! safety/pipeline-m12): the same faults (`inject`, cfg(test) only, in the leader's part
//! of the run while the helpers build and walk the block), the same three states, the same
//! checks, on `worker::<MODE>` for every `MODE` (the two controls, the leader's sort, the
//! helpers' sort and the direct fill). The tests drive the real `worker` on jobs queued in
//! full before it starts (so `try_job` finds the next one at every run: the team's state at
//! each run is fixed), except the last, which runs `StreamDecoder::pipelined_run_fed` end to
//! end from the scanner. Every wait is under a watchdog.

use super::super::{Event, Reader, Scanner, State};
use super::*;
use std::collections::BTreeMap;
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar};
use std::time::Duration;

/// How long a read, the retirements or the join may take before the test fails (Miri's
/// clock is its own, and it moves on at once once every thread is blocked)
const WATCHDOG: Duration = Duration::from_secs(if cfg!(miri) { 86_400 } else { 60 });

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

/// The tiny blocks for Miri: chunk i of `TINY`, 240 bytes (`parallel::unwind_tests`')
fn tiny_chunk(i: usize) -> Vec<u8> {
    (0..240)
        .map(|j| b"abcdefghij"[(j * 7 + i * 3 + j / 11 + (j * j) / 97) % 10])
        .collect()
}

/// `tiny_chunk(0..4)`, each compressed alone at level 1 (Python's bz2, libbzip2 1.0.8),
/// concatenated: libbzip2 can't run under Miri (`parallel::unwind_tests`' `TINY`, checked
/// by `the_tiny_blocks` here too)
const TINY: [u8; 539] = [
    0x42, 0x5a, 0x68, 0x31, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x37, 0x7d, 0x1c, 0x4e, 0x00, 0x00,
    0x07, 0xc1, 0x00, 0x54, 0x00, 0x3f, 0xf0, 0x30, 0x00, 0xb8, 0x08, 0x06, 0x80, 0x08, 0x06, 0x80,
    0x04, 0xa7, 0xa8, 0x98, 0xa4, 0xda, 0x4e, 0xf7, 0xf1, 0xf7, 0x8f, 0x8c, 0x5a, 0x9f, 0xa3, 0xd4,
    0xb0, 0xb6, 0xa8, 0x2a, 0x84, 0x2d, 0xe6, 0x21, 0x6b, 0x5d, 0x8a, 0xe3, 0x06, 0x5a, 0x5a, 0xd9,
    0x7f, 0x04, 0xc4, 0xe3, 0x23, 0xdd, 0x38, 0x2b, 0xa8, 0x9c, 0x3d, 0xcd, 0x69, 0x60, 0x4c, 0x58,
    0x3d, 0xbb, 0x92, 0x42, 0x56, 0x38, 0x17, 0x3f, 0x55, 0x38, 0x4c, 0xc9, 0xdd, 0x50, 0xd3, 0x44,
    0xb9, 0x1c, 0x15, 0xae, 0x98, 0x12, 0xe1, 0x22, 0x9f, 0x6e, 0x34, 0x93, 0xc9, 0xa7, 0xd3, 0x63,
    0x66, 0x64, 0x6c, 0x9d, 0x9c, 0x5c, 0xda, 0x0a, 0x1d, 0xea, 0x1e, 0x2c, 0x7f, 0x17, 0x72, 0x45,
    0x38, 0x50, 0x90, 0x37, 0x7d, 0x1c, 0x4e, 0x42, 0x5a, 0x68, 0x31, 0x31, 0x41, 0x59, 0x26, 0x53,
    0x59, 0x22, 0x35, 0x8f, 0xe4, 0x00, 0x00, 0x23, 0xc1, 0x00, 0x54, 0x00, 0x3f, 0xf0, 0x30, 0x00,
    0xb8, 0x08, 0x06, 0x80, 0x08, 0x06, 0x80, 0x04, 0xa7, 0xa9, 0x31, 0x29, 0xe4, 0xd4, 0xaa, 0xe8,
    0xc7, 0xab, 0xd7, 0x45, 0x46, 0xb8, 0x70, 0x29, 0xb2, 0x5d, 0x68, 0x91, 0xc9, 0xa5, 0xcc, 0x65,
    0xe9, 0xb8, 0xdc, 0xca, 0x08, 0x38, 0x16, 0x33, 0xf4, 0x2f, 0x92, 0x62, 0xdc, 0x25, 0x8b, 0x53,
    0xb8, 0x18, 0xa8, 0xc6, 0x39, 0x22, 0x18, 0xf1, 0x62, 0xe3, 0x18, 0xbc, 0x4b, 0x5b, 0xe4, 0x23,
    0x93, 0x0e, 0x27, 0xdb, 0x34, 0x2b, 0xb8, 0x4a, 0x3b, 0x9b, 0x74, 0xb0, 0x26, 0x0a, 0x0a, 0x74,
    0xbb, 0x99, 0x09, 0x58, 0xa2, 0xd1, 0x9f, 0x6b, 0x29, 0x9a, 0xd4, 0xa8, 0xfd, 0xb1, 0x63, 0x65,
    0xc2, 0x5f, 0xf1, 0x77, 0x24, 0x53, 0x85, 0x09, 0x02, 0x23, 0x58, 0xfe, 0x40, 0x42, 0x5a, 0x68,
    0x31, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x7b, 0x64, 0xf1, 0x42, 0x00, 0x00, 0x48, 0xc1, 0x00,
    0x54, 0x00, 0x3f, 0xf0, 0x30, 0x00, 0xb8, 0x08, 0x06, 0x80, 0x08, 0x06, 0x80, 0x04, 0xa9, 0xfa,
    0x4a, 0x78, 0xa1, 0x34, 0xdd, 0xb0, 0x33, 0xe6, 0x33, 0x85, 0x4e, 0xde, 0x92, 0x35, 0x08, 0x54,
    0xee, 0xa9, 0xb8, 0xc4, 0xc3, 0x32, 0x40, 0x91, 0x83, 0x08, 0x75, 0x45, 0xc5, 0x1f, 0x2d, 0x6f,
    0xd5, 0xae, 0x14, 0x50, 0xa7, 0x9b, 0xb5, 0x39, 0x3a, 0xfc, 0x6e, 0xc9, 0xc8, 0x34, 0x4d, 0x9a,
    0xbb, 0xac, 0x77, 0xe4, 0x3b, 0x5e, 0x60, 0x58, 0xf4, 0x1e, 0xb6, 0x16, 0xf8, 0x11, 0xb5, 0x3d,
    0xce, 0x18, 0xab, 0xcc, 0x72, 0x08, 0x63, 0xe2, 0x96, 0xa6, 0x3c, 0xb3, 0xa5, 0xc9, 0x44, 0x58,
    0x89, 0x7e, 0x02, 0x63, 0xc6, 0xdf, 0xb0, 0x9f, 0xc5, 0xdc, 0x91, 0x4e, 0x14, 0x24, 0x1e, 0xd9,
    0x3c, 0x50, 0x80, 0x42, 0x5a, 0x68, 0x31, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x17, 0x10, 0x7c,
    0xee, 0x00, 0x00, 0x70, 0xc1, 0x00, 0x54, 0x00, 0x3f, 0xf0, 0x30, 0x00, 0xb8, 0x08, 0x06, 0x80,
    0x08, 0x06, 0x80, 0x04, 0xa7, 0xa5, 0x1a, 0x6a, 0x4f, 0x44, 0xef, 0x7c, 0x3e, 0xcf, 0x18, 0x1f,
    0x3b, 0xe9, 0x56, 0x2d, 0x5c, 0xad, 0x24, 0xd2, 0x6d, 0x7d, 0x5a, 0xdf, 0xc1, 0x0e, 0x08, 0xea,
    0x4f, 0x70, 0xd8, 0xae, 0x22, 0x5d, 0x9f, 0xa6, 0x6a, 0x0d, 0x09, 0xc5, 0xa3, 0xa5, 0xdb, 0x4b,
    0x19, 0x5b, 0x0b, 0x9f, 0x6b, 0x4f, 0x23, 0x13, 0xdc, 0x50, 0xc3, 0x04, 0xb8, 0x72, 0x28, 0xeb,
    0xd3, 0x81, 0x1e, 0x10, 0x53, 0xec, 0xc7, 0x49, 0x37, 0xd3, 0x2b, 0x24, 0x83, 0xa2, 0x64, 0xb7,
    0x31, 0x56, 0x67, 0xa8, 0x78, 0xb5, 0xaa, 0xb1, 0xf3, 0x4d, 0x4a, 0x6e, 0xba, 0x9c, 0x0b, 0xdc,
    0xfe, 0x2e, 0xe4, 0x8a, 0x70, 0xa1, 0x20, 0x2e, 0x20, 0xf9, 0xdc,
];

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

/// A decode's faults registered for its life (`FAULTS`), removed on drop
struct Registered(usize);

impl Registered {
    fn new(shared: &Shared, f: Faults) -> Registered {
        let key = shared as *const Shared as usize;
        let mut all = lock(&FAULTS);
        assert!(all.iter().all(|e| e.0 != key), "a decode registered twice");
        all.push((key, f));
        Registered(key)
    }
}

impl Drop for Registered {
    fn drop(&mut self) {
        lock(&FAULTS).retain(|e| e.0 != self.0);
    }
}

/// `f` on the decode's faults (registered)
fn with<R>(dec: &StreamDecoder, f: impl FnOnce(&mut Faults) -> R) -> R {
    faults(&dec.shared, f).expect("the decode's faults are registered")
}

/// One run-fed team (`work`, one sub-arm's worker) on `jobs`, all queued before it starts,
/// the scanner's part done (every place issued, the end known), with `fault` to inject
fn team(
    work: Worker,
    jobs: Vec<Job>,
    team_k: usize,
    fault: (u64, Fault),
    min_coop: Option<usize>,
) -> (Arc<StreamDecoder>, Registered) {
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
    });
    let reg = Registered::new(
        &shared,
        Faults {
            at: Some(fault),
            min_coop,
            ..Faults::default()
        },
    );
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
    let dec = Arc::new(StreamDecoder {
        input: Mutex::new(None),
        shared,
        handles: Mutex::new(vec![h]),
        reader: Mutex::new(Reader::default()),
        threads: team_k,
        pipelined: true,
    });
    (dec, reg)
}

/// The whole read, under the watchdog: the bytes, then the error if any. On the watchdog,
/// the decoder is cancelled (so the reader thread ends) and the test fails.
fn read_all(dec: &Arc<StreamDecoder>, what: &str) -> (Vec<u8>, Option<crate::Error>) {
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

type Published = Vec<(u64, u64, Option<ErrorKind>)>;

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
        std::thread::sleep(Duration::from_millis(1));
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
    // every block published exactly once, at its own bit; the owned ones as Internal, the
    // others not
    let mut places: Vec<u64> = published.iter().map(|p| p.0).collect();
    places.sort_unstable();
    assert_eq!(
        places,
        (0..n as u64).collect::<Vec<_>>(),
        "{what}: each block published once"
    );
    for (at, bit, e) in &published {
        assert_eq!(*bit, bits[*at as usize], "{what}: block {at}'s bit");
        match (owned.contains(at), e) {
            (true, Some(ErrorKind::Internal(m))) => injected(m, *at),
            (true, Some(e)) => panic!("{what}: block {at}: not Internal: {e:?}"),
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
            let (dec, _reg) = team(work, jobs_of(&bz), team_k, fault, None);
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
        let (dec, _reg) = team(*work, jobs_of(&TINY), 2, fault, Some(1));
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

/// The tiny blocks are what `TINY` says
#[cfg(not(miri))]
#[test]
fn the_tiny_blocks() {
    use std::io::Read;
    let mut got = Vec::new();
    bzip2::read::MultiBzDecoder::new(&TINY[..])
        .read_to_end(&mut got)
        .unwrap();
    assert!(got == (0..4).map(tiny_chunk).collect::<Vec<_>>().concat());
    assert_eq!(jobs_of(&TINY).len(), 4);
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
            let _reg = Registered::new(
                &dec.shared,
                Faults {
                    at: Some(fault),
                    ..Faults::default()
                },
            );
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
