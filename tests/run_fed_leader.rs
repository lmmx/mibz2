//! The pipelined team with a run-fed leader (research, the `run-fed` feature:
//! `StreamDecoder::pipelined_run_fed`; docs/journal/2026-11-09-run-fed-leader.md), in each of
//! its sub-arms (the instrumented control, the register-list control, the sort on the
//! leader, on the helpers, and no F-order array), against the default decoder and
//! libbzip2: valid streams byte for byte (the crate's corpora, run-heavy full blocks that
//! take the records, small blocks among large, several streams), bad ones failing as the
//! default fails, at the same block, with the same bytes before it, and a reader dropped or
//! cancelled mid-stream. Every test also checks that no block needed the column's path
//! after a refused check (the phases' `fallbacks`, a bug's count).

mod common;
use common::*;
use mibz2::parallel::{run_fed_phases, RunFedSort};
use mibz2::{Error, ErrorKind, Kernels, StreamDecoder};
use std::io::Read;

const SORTS: [RunFedSort; 5] = [
    RunFedSort::Control,
    RunFedSort::RegControl,
    RunFedSort::Leader,
    RunFedSort::Helpers,
    RunFedSort::Direct,
];

/// Teams, team size, slots
const PLANS: [(usize, usize, usize); 5] = [(1, 2, 1), (1, 3, 4), (2, 3, 6), (3, 3, 12), (1, 4, 4)];

fn phase(name: &str) -> u64 {
    run_fed_phases()
        .into_iter()
        .find(|p| p.0 == name)
        .map(|p| p.2)
        .unwrap()
}

/// A decoder's output up to its end or its first error, fed from another thread
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

fn default(z: &[u8]) -> (Vec<u8>, Result<(), Error>) {
    run(
        &StreamDecoder::with_inverse::<mibz2::bwt::Segmented>(2, 4, Kernels::detect()),
        z,
    )
}

fn run_fed(
    z: &[u8],
    teams: usize,
    k: usize,
    slots: usize,
    sort: RunFedSort,
) -> (Vec<u8>, Result<(), Error>) {
    let dec = StreamDecoder::pipelined_run_fed(teams, k, slots, Kernels::detect(), sort);
    assert!(dec.is_pipelined() && dec.threads() == teams * k);
    run(&dec, z)
}

/// Byte for byte the default decoder's output and error (kind, block and bit), at every
/// plan and sub-arm; no fallback
fn same_as_default(z: &[u8], what: &str) {
    let (want, want_res) = default(z);
    for sort in SORTS {
        for (teams, k, slots) in PLANS {
            let (got, res) = run_fed(z, teams, k, slots, sort);
            assert!(
                got == want,
                "{what}: {teams} × {k} in {slots}, {sort:?}: the bytes"
            );
            assert_eq!(
                res, want_res,
                "{what}: {teams} × {k} in {slots}, {sort:?}: the result"
            );
        }
    }
    assert_eq!(phase("fallbacks"), 0, "{what}: a refused check");
}

/// A last column of runs over `sigma` bytes, run lengths from `lens`: a block the records'
/// rule takes (8 symbols or fewer a row's eighth)
fn runs_column(n: usize, sigma: u64, lens: &[usize], seed: u64) -> Vec<u8> {
    let mut r = Rng(seed);
    let mut ll = Vec::with_capacity(n);
    while ll.len() < n {
        let c = b"abcdefghijklmnop"[r.below(sigma) as usize];
        let run = lens[r.below(lens.len() as u64) as usize];
        ll.extend(std::iter::repeat(c).take(run));
    }
    ll.truncate(n);
    ll
}

/// A block of the column whose primary index, from `orig` on, gives a valid block (one
/// not ending inside an RLE1 run, which libbzip2 refuses)
fn valid(ll: Vec<u8>, orig: u32) -> Spec {
    let o = (orig..)
        .map(|o| o % ll.len() as u32)
        .find(|&o| reference(&ll, o).is_some())
        .unwrap();
    Spec::of(ll, o)
}

/// Run lengths the records' rule takes at a full block: about 6 symbols a run of about 49
const RUNS: &[usize] = &[16, 24, 40, 64, 100];

fn column(n: usize, seed: u64) -> Vec<u8> {
    let mut r = Rng(seed);
    (0..n).map(|_| b"abcdefgh"[r.below(8) as usize]).collect()
}

#[test]
fn the_corpora() {
    for (name, data) in corpora(1) {
        let z = compress(&data, 9);
        assert!(oracle(&z).unwrap() == data, "{name}");
        same_as_default(&z, name);
    }
}

/// Full blocks of runs: the team takes them by records (the phases count them), and every
/// sub-arm decodes them as the default does
#[test]
fn run_heavy_blocks_take_the_records() {
    let n = 600_000;
    let specs: Vec<Spec> = (0..5)
        .map(|i| valid(runs_column(n, 3 + 3 * i, RUNS, 70 + i), 4_321 + i as u32))
        .collect();
    let z = craft(9, &specs, None);
    let mut want = Vec::new();
    for s in &specs {
        want.extend(reference(&s.ll, s.orig_ptr).unwrap());
    }
    assert!(oracle(&z).unwrap() == want);
    let before = phase("team_records");
    same_as_default(&z, "run-heavy");
    // (3 record sub-arms × 5 plans × at least the 4 blocks after the first)
    assert!(phase("team_records") >= before + 3 * 5 * 4);
    assert!(phase("stores") >= phase("rows"));
}

#[test]
fn small_blocks_among_large_and_streams() {
    let mut z = Vec::new();
    let mut want = Vec::new();
    for (i, level) in [9u8, 1, 9, 2, 9, 3, 9].into_iter().enumerate() {
        let ll = if i % 2 == 0 {
            runs_column(280_000 + 9_973 * i, 4, RUNS, i as u64 + 1)
        } else {
            runs_column(1_500 + 12_000 * i, 5, &[1, 9, 20], i as u64 + 1)
        };
        let s = valid(ll, 77);
        want.extend(reference(&s.ll, s.orig_ptr).unwrap());
        z.extend(craft(level, &[s], None));
    }
    assert!(oracle(&z).unwrap() == want);
    let before = phase("small_records");
    same_as_default(&z, "mixed");
    // (small blocks by records: the one-core path, for the leader's sort and the others')
    assert!(phase("small_records") > before);
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
        same_as_default(&compress(&data, 9), what);
    }
    for sort in SORTS {
        let (got, res) = run_fed(&[], 1, 3, 3, sort);
        assert!(got.is_empty());
        assert_eq!(res.unwrap_err().kind, ErrorKind::Empty);
    }
}

#[test]
fn a_block_crc_in_the_middle() {
    let specs: Vec<Spec> = (0..5)
        .map(|i| valid(runs_column(400_000, 6, RUNS, 90 + i), 1_000))
        .collect();
    let z = craft(9, &specs, None);
    let starts = block_starts(&z);
    for at in [1, 2, starts.len() - 1] {
        let mut t = z.clone();
        let bit = starts[at] + 48 + 5;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        let (want, want_res) = default(&t);
        let e = want_res.clone().unwrap_err();
        assert!(matches!(e.kind, ErrorKind::BlockCrc { .. }) && e.block == Some(at as u64));
        assert!(!want.is_empty());
        same_as_default(&t, &format!("CRC of block {at}"));
    }
}

#[test]
fn a_bad_orig_ptr() {
    let n = 300_000;
    let good = |seed| valid(runs_column(n, 5, RUNS, seed), 12_345);
    let mut bad = Spec::of(runs_column(n, 5, RUNS, 3), n as u32);
    bad.crc = Some(0);
    let z = craft(9, &[good(1), good(2), bad, good(4)], Some(0));
    let (_, want_res) = default(&z);
    let e = want_res.unwrap_err();
    assert_eq!((e.kind, e.block), (ErrorKind::BadOrigPtr, Some(2)));
    same_as_default(&z, "bad orig_ptr");
}

#[test]
fn truncated() {
    let corp = corpora(1);
    let z = compress(&corp[0].1[..3_000_000], 9);
    for cut in [z.len() / 2, z.len() - 3, 10] {
        same_as_default(&z[..cut], &format!("cut at {cut}"));
        assert_eq!(default(&z[..cut]).1.unwrap_err().kind, ErrorKind::Truncated);
    }
}

/// A move-to-front error (a RUNA run too long, and a block too long) on the block a
/// leader stages while its helpers invert the one before, after full blocks by records
/// and after a small one
#[test]
fn mtf_errors_on_the_prefetched_block() {
    let n = 400_000;
    let good = |seed| valid(runs_column(n, 4, RUNS, seed), 777);
    let long_run = || {
        let mut s = Spec::of(vec![b'a'; 10], 0);
        s.in_use = Some(vec![b'a']);
        s.syms = Some([vec![0u16; 22], vec![2]].concat());
        s.crc = Some(0);
        s
    };
    let small = valid(column(1_500, 9), 42);
    for (what, blocks, kind) in [
        (
            "a run too long",
            vec![good(5), good(6), long_run(), good(7)],
            ErrorKind::RunTooLong,
        ),
        (
            "after a small block",
            vec![good(5), small, long_run(), good(7)],
            ErrorKind::RunTooLong,
        ),
    ] {
        let z = craft(9, &blocks, Some(0));
        let (_, want_res) = default(&z);
        let e = want_res.unwrap_err();
        assert_eq!((e.kind, e.block), (kind, Some(2)), "{what}");
        same_as_default(&z, what);
    }
    // a block longer than its level allows: a run past the bound (level 1: 100,000 rows)
    let mut too_long = Spec::of(vec![b'a'; 10], 0);
    too_long.in_use = Some(vec![b'a', b'b']);
    too_long.syms = Some([vec![2u16], vec![1u16; 17], vec![3]].concat());
    too_long.crc = Some(0);
    let z = craft(
        1,
        &[valid(runs_column(90_000, 3, &[8, 9], 1), 5), too_long],
        Some(0),
    );
    let (_, want_res) = default(&z);
    assert_eq!(want_res.unwrap_err().kind, ErrorKind::BlockTooLong);
    same_as_default(&z, "a block too long");
}

/// The reader dropped mid-stream, and the push form cancelled while its writer writes:
/// nothing hangs (a watchdog fails the test if a drop takes a minute)
#[test]
fn dropped_and_cancelled_mid_stream() {
    let specs: Vec<Spec> = (0..6)
        .map(|i| valid(runs_column(500_000, 5, RUNS, 40 + i), 99))
        .collect();
    let z = craft(9, &specs, None);
    let data = oracle(&z).unwrap();
    for sort in SORTS {
        for (teams, k, slots) in PLANS {
            let (z, data) = (z.clone(), data.clone());
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let r = std::io::Cursor::new(z);
                let mut r = mibz2::BzReader::parallel_pipelined_run_fed(
                    r,
                    teams,
                    k,
                    slots,
                    Kernels::detect(),
                    sort,
                );
                let mut buf = vec![0u8; data.len() / 3];
                r.read_exact(&mut buf).unwrap();
                assert!(buf == data[..buf.len()]);
                drop(r);
                tx.send(()).unwrap();
            });
            rx.recv_timeout(std::time::Duration::from_secs(60))
                .unwrap_or_else(|_| panic!("{teams} × {k}, {sort:?}: the drop hung"));
        }
        let dec = StreamDecoder::pipelined_run_fed(1, 3, 3, Kernels::detect(), sort);
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
            dec.read_exact_for_test(&mut buf);
            dec.cancel();
            w.join().unwrap();
            assert_eq!(dec.read(&mut buf).unwrap_err().kind, ErrorKind::Cancelled);
        });
        dec.join();
        let dec = StreamDecoder::pipelined_run_fed(2, 2, 4, Kernels::detect(), sort);
        dec.write(z[..z.len() / 2].to_vec()).unwrap();
        drop(dec);
    }
}

trait ReadExact {
    fn read_exact_for_test(&self, buf: &mut [u8]);
}

impl ReadExact for StreamDecoder {
    fn read_exact_for_test(&self, buf: &mut [u8]) {
        let mut at = 0;
        while at < buf.len() {
            let n = self.read(&mut buf[at..]).unwrap();
            assert!(n > 0);
            at += n;
        }
    }
}
