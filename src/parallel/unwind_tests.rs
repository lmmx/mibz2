//! A pipelined team's run that panics outside its stages' own catches (Guardian D047, M12):
//! the panic unwinds through the pool's run into `pipelined_worker`'s outer `Err`, which
//! must retire every block the run still holds, each exactly once, with its own place,
//! bit and an `Internal` error, and reset the team, so that the reader gets the ordered
//! prefix, then the error, and never waits on a place nobody will fill.
//!
//! The faults are injected by `inject` (cfg(test) only) in the leader's part of the run,
//! with the team's helpers inverting the block. The tests drive the real
//! `pipelined_worker` on jobs queued in full before it starts (so `try_job` finds the
//! next one at every run: the team's state at each run is fixed), except the last, which
//! runs the default decoder end to end. Every wait is under a watchdog.

use super::*;
use std::time::Duration;

/// How long a read, the retirements or the join may take before the test fails. Miri's
/// clock is its own: it runs at about the wall's pace under Miri here (a passing tiny test
/// takes about 25 s of it), so its watchdog is longer, but not so long that a failing
/// test's polling runs for ever.
const WATCHDOG: Duration = Duration::from_secs(if cfg!(miri) { 600 } else { 60 });

/// The wait between polls of the retirements
const POLL: Duration = Duration::from_millis(if cfg!(miri) { 50 } else { 1 });

fn kernels() -> Kernels {
    if cfg!(miri) {
        Kernels::scalar()
    } else {
        Kernels::detect()
    }
}

/// Lowercase text that RLE1 leaves at its length: `n` rows, a block of its own
#[cfg(not(miri))]
fn text(seed: u64, n: usize) -> Vec<u8> {
    let mut x = seed | 1;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            b'a' + (x % 26) as u8
        })
        .collect()
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

/// The tiny blocks for Miri: chunk i of `TINY`, 240 bytes
fn tiny_chunk(i: usize) -> Vec<u8> {
    (0..240)
        .map(|j| b"abcdefghij"[(j * 7 + i * 3 + j / 11 + (j * j) / 97) % 10])
        .collect()
}

/// `tiny_chunk(0..4)`, each compressed alone at level 1 (Python's bz2, libbzip2 1.0.8),
/// concatenated: libbzip2 can't run under Miri
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
    let mut sc = Scanner::new(super::super::stream::SliceSource(bz));
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

/// One pipelined team with the default's levers (park, small), on `jobs`, all queued
/// before it starts, the scanner's part done (every place issued, the end known), with
/// `fault` to inject
fn team(
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
        #[cfg(feature = "c29-reach")]
        c29_stages: AtomicU64::new(0),
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
        .spawn(move || {
            pipelined_worker::<true, true, { crate::coop::BARRIER_SPINS }>(
                &sh,
                &Mutex::new(rx),
                k,
                team_k,
                false,
            )
        })
        .unwrap();
    Arc::new(StreamDecoder {
        input: Mutex::new(None),
        shared,
        handles: Mutex::new(vec![h]),
        reader: Mutex::default(),
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

/// Waits, under the watchdog, until `n` blocks have been published
fn all_published(dec: &StreamDecoder, n: usize, what: &str) -> Vec<(u64, Option<Error>)> {
    let until = std::time::Instant::now() + WATCHDOG;
    loop {
        let f = dec.shared.faults.lock().unwrap();
        if f.published.len() >= n {
            return f.published.clone();
        }
        if std::time::Instant::now() > until {
            let got: Vec<u64> = f.published.iter().map(|p| p.0).collect();
            panic!(
                "{what}: watchdog: {} of {n} blocks published, places {got:?}",
                got.len()
            );
        }
        drop(f);
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
    let internal = |e: &Error, at: u64| {
        let ErrorKind::Internal(m) = &e.kind else {
            panic!("{what}: block {at}: not Internal: {e:?}");
        };
        assert!(
            m.starts_with(&format!("injected at block {place}, {:?}", fault.1)),
            "{what}: block {at}: {m}"
        );
        assert_eq!(
            (e.block, e.bit),
            (Some(at), Some(bits[at as usize])),
            "{what}: block {at}'s place and bit"
        );
    };

    // the reader: the ordered prefix, then the first held block's error
    let (got, err) = read_all(dec, what);
    assert!(
        got == chunks[..first].concat(),
        "{what}: not the prefix of {first} blocks ({} bytes)",
        got.len()
    );
    internal(
        &err.unwrap_or_else(|| panic!("{what}: no error")),
        first as u64,
    );

    // the fault fired in the state asked for, and the outer branch recovered from it
    let published = all_published(dec, n, what);
    {
        let f = dec.shared.faults.lock().unwrap();
        assert_eq!(
            f.held,
            Some(held),
            "{what}: the team's state when the fault fired"
        );
        assert_eq!(f.recovered, 1, "{what}: the outer unwinds recovered");
        assert!(f.at.is_none(), "{what}: the fault never fired");
    }
    // every block published exactly once; the owned ones as Internal, the others not
    let mut places: Vec<u64> = published.iter().map(|p| p.0).collect();
    places.sort_unstable();
    assert_eq!(
        places,
        (0..n as u64).collect::<Vec<_>>(),
        "{what}: each block published once"
    );
    for (at, e) in &published {
        match (owned.contains(at), e) {
            (true, Some(e)) => internal(e, *at),
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

/// A state on four full blocks (above `MIN_COOP`, the team's own threshold), on teams of
/// 2, 3 and 4
#[cfg(not(miri))]
fn full((name, fault, held): (&str, (u64, Fault), Held)) {
    let chunks: Vec<Vec<u8>> = (0..4)
        .map(|i| text(41 + i, crate::bwt::MIN_COOP + 10_000))
        .collect();
    let bz = streams(&chunks);
    let bits: Vec<u64> = jobs_of(&bz).iter().map(|j| j.1).collect();
    for team_k in [2, 3, 4] {
        let what = format!("{name}, k {team_k}");
        let dec = team(jobs_of(&bz), team_k, fault, None);
        check(&dec, &chunks, &bits, fault, held, &what);
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

/// A state on four tiny blocks, on a team of two (one helper, so the run's unwind waits on
/// a live helper), its threshold lowered so that they run the team: under Miri too, which
/// checks the unwind's hand-offs between the leader, the helper, the reset and the reader
fn tiny((name, fault, held): (&str, (u64, Fault), Held)) {
    let chunks: Vec<Vec<u8>> = (0..4).map(tiny_chunk).collect();
    let bits: Vec<u64> = jobs_of(&TINY).iter().map(|j| j.1).collect();
    let what = format!("tiny, {name}");
    let dec = team(jobs_of(&TINY), 2, fault, Some(1));
    check(&dec, &chunks, &bits, fault, held, &what);
}

#[test]
fn miri_run_panic_current() {
    tiny(CURRENT);
}

#[test]
fn miri_run_panic_current_and_next() {
    tiny(NEXT);
}

#[test]
fn miri_run_panic_current_and_held_prior() {
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

/// The default decoder (three pipelined teams of three, forced), end to end, from the
/// scanner, a run panicking: which team runs the block, and what it holds then, is up to
/// the threads, so the error expected is at the first block the run held
#[cfg(not(miri))]
#[test]
fn the_default_decoder_recovers_from_a_run_panic() {
    let chunks: Vec<Vec<u8>> = (0..6)
        .map(|i| text(71 + i, crate::bwt::MIN_COOP + 5_000))
        .collect();
    let bz = streams(&chunks);
    let bits: Vec<u64> = jobs_of(&bz).iter().map(|j| j.1).collect();
    for fault in [
        (2, Fault::LeadStart),
        (2, Fault::LeadEnd),
        (4, Fault::LeadStart),
    ] {
        let what = format!("default 3x3, {fault:?}");
        let dec = Arc::new(with_plan(PIPELINED_PLAN, || {
            StreamDecoder::new(4, 16, kernels())
        }));
        assert!(dec.is_pipelined() && dec.threads() == 9);
        dec.shared.faults.lock().unwrap().at = Some(fault);
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
        let held = dec.shared.faults.lock().unwrap().held;
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
        assert_eq!(dec.shared.faults.lock().unwrap().recovered, 1, "{what}");
        join(&dec, &what);
    }
}
