//! The pipelined team (`StreamDecoder::pipelined`, research): its leader's RLE1, CRC and
//! next move-to-front overlapped with its helpers' build and walk. Against the default
//! decoder and libbzip2, at teams of 2, 3 and 4: valid streams byte for byte (the crate's
//! corpora, several streams, small blocks mixed with large, one block, no blocks), and
//! bad ones failing as the default fails, at the same block, with the same bytes before it
//! (a block CRC, a primary index, a truncation, and a move-to-front error on the block a
//! leader prefetched while the one before was in flight); and a reader dropped mid-stream.
//! Every test runs the registered prototype and each of its levers
//! (`StreamDecoder::pipelined_levers`): park, the spin-budget control (spin2k), small, park
//! and small together, and park after the short spin; and the default's levers (park and
//! small) with each pool-wait arm that is not the default's (`Levers::pool`: the pool's own
//! two waits parking at once, both0, or only the helpers' idle wait doing so,
//! helper0/join2000; docs/journal/2026-11-10-pool-wait-split.md), so all three arms (the
//! default's levers being both2000) are held to the same bytes, errors and cancellation.
//!
//! **The default at the measured plan:** `StreamDecoder::new` (so `BzReader::parallel` and
//! `decode_parallel`) runs pipelined teams with park and small at three teams of three
//! (`PIPELINED_PLAN`) alone. Every test here also runs the default constructor with that
//! plan forced (`parallel::with_plan`, on any machine), against the default decoder, the
//! plain three teams of three (`planned`) and libbzip2; and every other plan forced through
//! the default constructor takes the plain path.

mod common;
use common::*;
use mibz2::parallel::{with_plan, Levers, Plan, PoolWait, PIPELINED_PLAN};
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

/// The registered prototype (`StreamDecoder::pipelined`, no lever), then each lever alone,
/// the spin-budget control, park and small together, and park at the short budget; then
/// park and small with the pool-wait arms both0 and helper0/join2000 (park and small
/// alone being both2000)
const LEVERS: [Levers; 8] = [
    lv(false, false, false),
    lv(true, false, false),
    lv(false, false, true),
    lv(false, true, false),
    lv(true, true, false),
    lv(true, false, true),
    pool(PoolWait::Spin0),
    pool(PoolWait::Helper0Join2000),
];

const fn lv(park: bool, small: bool, spin2k: bool) -> Levers {
    Levers {
        park,
        small,
        spin2k,
        pool: PoolWait::Spin2000,
    }
}

/// The default's levers (park and small) with the pool-wait arm `pool`
const fn pool(pool: PoolWait) -> Levers {
    Levers {
        pool,
        ..lv(true, true, false)
    }
}

fn pipe(
    z: &[u8],
    teams: usize,
    k: usize,
    slots: usize,
) -> (Vec<u8>, Result<(), Error>, (u64, u64)) {
    pipe_with(z, teams, k, slots, Levers::default())
}

/// A pipelined decoder with its levers (none: `StreamDecoder::pipelined` itself)
fn pipe_with(
    z: &[u8],
    teams: usize,
    k: usize,
    slots: usize,
    levers: Levers,
) -> (Vec<u8>, Result<(), Error>, (u64, u64)) {
    let dec = if levers == Levers::default() {
        StreamDecoder::pipelined(teams, k, slots, Kernels::detect())
    } else {
        StreamDecoder::pipelined_levers(teams, k, slots, Kernels::detect(), levers)
    };
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

/// The default constructor (`StreamDecoder::new`) with `plan` forced, on 4 threads asked
/// for in 16 slots (the plan's own threads, and its window scaled as the policy's)
fn forced_new(plan: Plan) -> StreamDecoder {
    with_plan(plan, || StreamDecoder::new(4, 16, Kernels::detect()))
}

/// The default constructor at the measured plan: pipelined, its threads the plan's
fn default_33() -> StreamDecoder {
    let dec = forced_new(PIPELINED_PLAN);
    assert!(dec.is_pipelined() && dec.threads() == 9);
    dec
}

/// The default constructor at 3 × 3 against the plain three teams of three
/// (`StreamDecoder::planned`, which never pipelines) and `want`: the same bytes and result.
/// Its pipeline counts.
fn default_33_same(z: &[u8], want: &[u8], want_res: &Result<(), Error>, what: &str) -> (u64, u64) {
    let dec = default_33();
    let (got, res) = run(&dec, z);
    assert!(got == want, "{what}: the default at 3 × 3: the bytes");
    assert_eq!(&res, want_res, "{what}: the default at 3 × 3: the result");
    let plain = StreamDecoder::planned(PIPELINED_PLAN, 12, Kernels::detect());
    assert!(!plain.is_pipelined());
    let (pgot, pres) = run(&plain, z);
    assert!(pgot == want, "{what}: the plain 3 × 3: the bytes");
    assert_eq!(&pres, want_res, "{what}: the plain 3 × 3: the result");
    assert_eq!(plain.pipeline_counts(), (0, 0));
    dec.pipeline_counts()
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
/// and with every lever setting
fn same_as_default(z: &[u8], what: &str) -> (u64, u64) {
    let (want, want_res) = default(z);
    let mut counts = (0, 0);
    for levers in LEVERS {
        for (teams, k, slots) in PLANS {
            let (got, res, c) = pipe_with(z, teams, k, slots, levers);
            assert!(
                got == want,
                "{what}: {teams} teams of {k} in {slots}, {levers:?}: the bytes"
            );
            assert_eq!(
                res, want_res,
                "{what}: {teams} teams of {k} in {slots}, {levers:?}: the result"
            );
            counts = (counts.0 + c.0, counts.1 + c.1);
        }
    }
    let c = default_33_same(z, &want, &want_res, what);
    (counts.0 + c.0, counts.1 + c.1)
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
        // the default at 3 × 3 through each default entry point: libbzip2's bytes
        let (got, res) = run(&default_33(), &z);
        res.unwrap();
        assert!(got == data, "{name}: StreamDecoder::new at 3 × 3");
        let got = with_plan(PIPELINED_PLAN, || mibz2::decode_parallel(&z, 10)).unwrap();
        assert!(got == data, "{name}: decode_parallel at 3 × 3");
        let mut got = Vec::new();
        let mut r = with_plan(PIPELINED_PLAN, || {
            mibz2::BzReader::parallel(std::io::Cursor::new(z.clone()), 10)
        });
        r.read_to_end(&mut got).unwrap();
        assert!(got == data, "{name}: BzReader::parallel at 3 × 3");
        let mut got = Vec::new();
        let mut r = with_plan(PIPELINED_PLAN, || {
            mibz2::BzReader::parallel_with(
                std::io::Cursor::new(z.clone()),
                4,
                16,
                Kernels::detect(),
            )
        });
        r.read_to_end(&mut got).unwrap();
        assert!(got == data, "{name}: BzReader::parallel_with at 3 × 3");
    }
    assert!(
        overlapped > 0,
        "no block was ever staged during another's inversion"
    );
}

/// Every plan but three teams of three, forced through the default constructor, takes the
/// plain path: not pipelined, no job ever taken overlapped, the plan's threads, and the
/// default's bytes and errors
#[test]
fn other_plans_take_the_plain_path() {
    let corp = corpora(1);
    let text = &corp[0].1[..3_000_000];
    let z = compress(text, 9);
    let mut bad = z.clone();
    let starts = block_starts(&z);
    let bit = starts[1] + 48 + 5;
    bad[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
    let (want_bad, want_bad_res) = default(&bad);
    assert!(matches!(
        want_bad_res.as_ref().unwrap_err().kind,
        ErrorKind::BlockCrc { .. }
    ));
    let plans = [
        Plan::Workers(1),
        Plan::Workers(4),
        Plan::Workers(9),
        Plan::Teams { teams: 1, k: 3 },
        Plan::Teams { teams: 2, k: 3 },
        Plan::Teams { teams: 4, k: 3 },
        Plan::Teams { teams: 3, k: 2 },
        Plan::Teams { teams: 4, k: 2 },
        Plan::Teams { teams: 3, k: 4 },
        Plan::Teams { teams: 3, k: 1 },
        Plan::Teams { teams: 3, k: 5 },
    ];
    for plan in plans {
        let dec = forced_new(plan);
        assert!(!dec.is_pipelined(), "{plan:?}");
        assert_eq!(dec.threads(), plan.threads(), "{plan:?}");
        let (got, res) = run(&dec, &z);
        res.unwrap();
        assert!(got == text, "{plan:?}: the bytes");
        assert_eq!(dec.pipeline_counts(), (0, 0), "{plan:?}");
        let (got, res) = run(&forced_new(plan), &bad);
        assert!(got == want_bad, "{plan:?}: the bytes before the error");
        assert_eq!(res, want_bad_res, "{plan:?}: the error");
        let got = with_plan(plan, || mibz2::decode_parallel(&z, 10)).unwrap();
        assert!(got == text, "{plan:?}: decode_parallel");
    }
    // and the plan this machine's policy gives, unforced: pipelined at 3 × 3 only
    for t in [1, 2, 4, 8, 10, 20] {
        let dec = StreamDecoder::new(t, 4 * t, Kernels::detect());
        let plan = Plan::for_threads(t);
        assert_eq!(dec.is_pipelined(), plan == PIPELINED_PLAN, "{t}: {plan:?}");
        assert_eq!(dec.threads(), plan.threads(), "{t}");
        dec.close();
        assert_eq!(dec.read(&mut [0; 8]).unwrap_err().kind, ErrorKind::Empty);
    }
    // the forced plan is this thread's alone, and restored after
    let p = with_plan(PIPELINED_PLAN, || {
        std::thread::spawn(|| Plan::for_threads(10)).join().unwrap()
    });
    let here = Plan::for_threads(10);
    assert_eq!(p, here);
    let unwound = std::panic::catch_unwind(|| with_plan(PIPELINED_PLAN, || panic!("inside")));
    assert!(unwound.is_err());
    assert_eq!(Plan::for_threads(10), here);
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

/// Lever B's code path: full blocks and small ones alternating, one team. As registered, a
/// small block taken in a run leaves the next run nothing staged, so the job after it is
/// taken with the pipeline drained; with lever B the leader takes it in the run too. So the
/// jobs taken during runs (`pipeline_counts`) rise, whenever the scanner is ahead (nearly
/// always: it is tried until it is), and the bytes are the same either way. Jobs taken are
/// not overlap: this shows the path runs, not that it helps (the registration's evidence is
/// the traces' overlap and join delay, and the wall and CPU).
#[test]
fn small_blocks_keep_the_pipeline_primed() {
    let n = 300_000;
    let mut specs = Vec::new();
    for i in 0..6u64 {
        specs.push(Spec::of(column(n, 40 + i), 1_234));
        specs.push(Spec::of(column(2_000, 50 + i), 99));
    }
    specs.push(Spec::of(column(n, 60), 1_234));
    let z = craft(9, &specs, None);
    let (want, want_res) = default(&z);
    want_res.unwrap();
    assert!(oracle(&z).unwrap() == want);
    same_as_default(&z, "alternating");
    let mut raised = false;
    for _ in 0..20 {
        for k in [2, 3] {
            let (got, res, (plain, _)) = pipe(&z, 1, k, 16);
            res.unwrap();
            assert!(got == want);
            for park in [false, true] {
                let levers = lv(park, true, false);
                let (got, res, (primed, _)) = pipe_with(&z, 1, k, 16, levers);
                res.unwrap();
                assert!(got == want, "k {k}, {levers:?}");
                raised |= primed > plain;
            }
        }
        if raised {
            break;
        }
    }
    assert!(
        raised,
        "lever B never took more jobs overlapped than the prototype"
    );
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
    // (scale 2: the text corpus is the repository's own files, about 3 MB at scale 1, which
    // can compress to only 3 blocks at level 9 as those files change; this needs at least 4)
    let corp = corpora(2);
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
    default_33_same(&z, &want, &want_res, "an MTF error on the prefetched block");
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

/// A block whose move-to-front fails right after a small block, after a full one: with one
/// team and lever B, the leader stages it in the run that decoded the small block; every
/// lever setting gives the default's bytes and error
#[test]
fn an_mtf_error_after_a_small_block() {
    let n = 400_000;
    let good = |seed| Spec::of(column(n, seed), 777);
    let small = Spec::of(column(1_500, 9), 42);
    let mut bad = Spec::of(vec![b'a'; 10], 0);
    bad.in_use = Some(vec![b'a']);
    bad.syms = Some([vec![0u16; 22], vec![2]].concat());
    bad.crc = Some(0);
    let z = craft(9, &[good(5), small, bad, good(7)], Some(0));
    let (want, want_res) = default(&z);
    let e = want_res.clone().unwrap_err();
    assert_eq!((e.kind.clone(), e.block), (ErrorKind::RunTooLong, Some(2)));
    let mut expect = reference(&column(n, 5), 777).unwrap();
    expect.extend(reference(&column(1_500, 9), 42).unwrap());
    assert!(want == expect);
    same_as_default(&z, "an MTF error after a small block");
    let mut overlapped_errors = 0;
    for _ in 0..20 {
        for k in [2, 3, 4] {
            for park in [false, true] {
                let levers = lv(park, true, false);
                let (got, res, (_, errs)) = pipe_with(&z, 1, k, 8, levers);
                assert!(got == want, "k {k}, {levers:?}: the bytes before the error");
                assert_eq!(res, want_res, "k {k}, {levers:?}");
                overlapped_errors += errs;
            }
        }
        if overlapped_errors > 0 {
            break;
        }
    }
    assert!(
        overlapped_errors > 0,
        "the failing block was never staged overlapped after the small one"
    );
}

/// The reader dropped mid-stream, on every plan: nothing hangs (a watchdog fails the test
/// if a drop takes a minute)
#[test]
fn dropped_mid_stream() {
    let corp = corpora(1);
    let data = corp[0].1.clone();
    let z = compress(&data, 9);
    for levers in LEVERS {
        for (teams, k, slots) in PLANS {
            let z = z.clone();
            let data = data.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let r = std::io::Cursor::new(z);
                let mut r = if levers == Levers::default() {
                    mibz2::BzReader::parallel_pipelined(r, teams, k, slots, Kernels::detect())
                } else {
                    mibz2::BzReader::parallel_pipelined_levers(
                        r,
                        teams,
                        k,
                        slots,
                        Kernels::detect(),
                        levers,
                    )
                };
                let mut buf = vec![0u8; 1_500_000];
                r.read_exact(&mut buf).unwrap();
                assert!(buf == data[..buf.len()]);
                drop(r);
                tx.send(()).unwrap();
            });
            rx.recv_timeout(std::time::Duration::from_secs(60))
                .unwrap_or_else(|_| panic!("{teams} teams of {k}, {levers:?}: the drop hung"));
        }
    }
    // the default constructor at 3 × 3 (`BzReader::parallel`), dropped mid-stream, and
    // dropped before any read
    for read in [1_500_000, 0] {
        let z = z.clone();
        let data = data.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut r = with_plan(PIPELINED_PLAN, || {
                mibz2::BzReader::parallel(std::io::Cursor::new(z), 10)
            });
            let mut buf = vec![0u8; read];
            r.read_exact(&mut buf).unwrap();
            assert!(buf == data[..read]);
            drop(r);
            tx.send(()).unwrap();
        });
        rx.recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or_else(|_| panic!("the default at 3 × 3, read {read}: the drop hung"));
    }
    // and the push form, cancelled by its reader while its writer writes
    for levers in LEVERS {
        cancelled_while_writing(&z, || {
            StreamDecoder::pipelined_levers(1, 3, 3, Kernels::detect(), levers)
        });
    }
    cancelled_while_writing(&z, default_33);
    // the default at 3 × 3 cancelled before anything is written, and dropped unread
    let dec = default_33();
    dec.cancel();
    assert!(dec.write(z[..1000].to_vec()).is_err());
    assert_eq!(
        dec.read(&mut [0; 8]).unwrap_err().kind,
        ErrorKind::Cancelled
    );
    dec.join();
    let dec = default_33();
    dec.write(z[..200_000].to_vec()).unwrap();
    drop(dec);
}

fn cancelled_while_writing(z: &[u8], make: impl FnOnce() -> StreamDecoder) {
    let dec = make();
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
