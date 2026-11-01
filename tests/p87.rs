//! P87's diagnostic path (feature `p87`; docs/journal/2026-11-01-p87-four-core-protocol.md):
//! both arms decode exactly what the shipped path and libbzip2 decode, on every shape the
//! shared samples hold, at block sizes from short (the plain-walk fallback) to full, with a
//! scratch reused across blocks and across streams of different levels; the two buffers
//! start at one capacity, and arm B never writes `rle` after its prefault; the error paths
//! end as the shipped path's do, with the same output kept.
//!
//! These call `p87::decode_block` with the arm given, never the process-wide arm (that is
//! tests/p87_parallel.rs, a separate process).
#![cfg(feature = "p87")]

mod common;
use common::{compress, craft, oracle, Rng, Spec};
use mibz2::bwt::Segmented;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{p87, ErrorKind, Kernels, Scratch};

/// What one arm's sequential decode gave: its output (or the error and the output kept),
/// and the scratch's buffers after the first block and at the end
struct Run {
    out: Vec<u8>,
    err: Option<ErrorKind>,
    first: Option<(usize, usize, usize)>,
    last: (usize, usize, usize),
}

/// `decode_with`'s loop, each block through `p87::decode_block` (`None`: the shipped
/// `decode_block`)
fn run(data: &[u8], arm: Option<(bool, bool)>) -> Run {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut s = Scratch::<Segmented>::default();
    let mut out = Vec::new();
    let mut syms = Vec::new();
    let mut first = None;
    loop {
        match sc.next(syms) {
            Ok(Event::Block { symbols, .. }) => {
                let r = match arm {
                    Some((into_ll, rle_first)) => {
                        p87::decode_block(&symbols, k, &mut s, &mut out, into_ll, rle_first)
                    }
                    None => mibz2::decode_block(&symbols, k, &mut s, &mut out),
                };
                if first.is_none() {
                    first = Some(p87::buffers(&s));
                }
                if let Err(e) = r {
                    return Run {
                        out,
                        err: Some(e),
                        first,
                        last: p87::buffers(&s),
                    };
                }
                syms = symbols.syms;
            }
            Ok(Event::Done) => break,
            Err(e) => {
                return Run {
                    out,
                    err: Some(e.kind),
                    first,
                    last: p87::buffers(&s),
                }
            }
        }
    }
    Run {
        out,
        err: None,
        first,
        last: p87::buffers(&s),
    }
}

const ARMS: [(bool, bool); 4] = [(false, false), (false, true), (true, false), (true, true)];

#[test]
fn both_arms_decode_every_sample_exactly() {
    let mut rng = Rng(0x87);
    for (name, data) in common::samples(&mut rng) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let want = oracle(&z).unwrap();
            assert_eq!(want, data, "{name} -{level}: the oracle");
            let shipped = run(&z, None);
            assert!(shipped.err.is_none(), "{name} -{level}: shipped");
            assert_eq!(shipped.out, want, "{name} -{level}: shipped");
            for arm in ARMS {
                let r = run(&z, Some(arm));
                assert!(r.err.is_none(), "{name} -{level} {arm:?}: {:?}", r.err);
                assert_eq!(r.out, want, "{name} -{level} {arm:?}: output");
            }
        }
    }
}

#[test]
fn a_scratch_reused_across_levels_and_short_blocks() {
    // streams one after another at levels 1, 9, 2 and 9 again (block capacities 100k,
    // 900k, 200k): each ends in a short block (the plain walk); the scratch carries over
    let mut rng = Rng(0x8701);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let runs = &samples.iter().find(|(n, _)| n == "runs").unwrap().1;
    let mut z = Vec::new();
    let mut want = Vec::new();
    for (level, part) in [
        (1, &text[..350_000]),
        (9, &text[..]),
        (2, &runs[..]),
        (9, &text[..70_000]),
    ] {
        z.extend(compress(part, level));
        want.extend_from_slice(part);
    }
    assert_eq!(oracle(&z).unwrap(), want);
    for arm in ARMS {
        let r = run(&z, Some(arm));
        assert!(r.err.is_none(), "{arm:?}: {:?}", r.err);
        assert_eq!(r.out, want, "{arm:?}");
    }
}

#[test]
fn the_buffers_start_equal_and_arm_b_never_writes_rle() {
    let mut rng = Rng(0x8702);
    let samples = common::samples(&mut rng);
    let text = &samples.iter().find(|(n, _)| n == "text").unwrap().1;
    let z = compress(text, 9);
    for arm in ARMS {
        let r = run(&z, Some(arm));
        assert!(r.err.is_none());
        // the first block's: both prefaulted to the level's capacity, 900,000
        let (ll, rle, _) = r.first.unwrap();
        assert_eq!((ll, rle), (900_000, 900_000), "{arm:?}: first block");
        // at the end: neither reallocated
        let (ll, rle, rle_len) = r.last;
        assert_eq!((ll, rle), (900_000, 900_000), "{arm:?}: at the end");
        let (into_ll, _) = arm;
        if into_ll {
            assert_eq!(rle_len, 0, "{arm:?}: arm B wrote rle");
        } else {
            assert!(rle_len > 0, "{arm:?}: arm A's assembly is in rle");
        }
    }
}

#[test]
fn errors_end_as_the_shipped_path_does() {
    // a good block, then: a wrong CRC; a primary index past the block (each its own stream)
    let mut rng = Rng(0x8703);
    let good: Vec<u8> = rng.bytes(200_000);
    let good_z = compress(&good, 9);
    let ll: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
    let mut bad_crc = Spec::of(ll.clone(), 17);
    bad_crc.crc = Some(0x1234_5678);
    let mut cases = Vec::new();
    for spec in [bad_crc, Spec::of(ll.clone(), ll.len() as u32)] {
        let mut z = good_z.clone();
        z.extend(craft(9, &[spec], None));
        cases.push(z);
    }
    for (i, z) in cases.iter().enumerate() {
        let shipped = run(z, None);
        let e = shipped
            .err
            .clone()
            .unwrap_or_else(|| panic!("case {i}: shipped decoded"));
        assert_eq!(&shipped.out, &good, "case {i}: shipped kept the good block");
        for arm in ARMS {
            let r = run(z, Some(arm));
            assert_eq!(r.err.as_ref(), Some(&e), "case {i} {arm:?}: the error");
            assert_eq!(r.out, shipped.out, "case {i} {arm:?}: the output kept");
        }
    }
}
