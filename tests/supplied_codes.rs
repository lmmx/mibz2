//! (research only, `supplied-codes`) bench-supplied-codes' arms through production's lazy
//! workers (`OpenCells`), as examples/supplied_codes.rs drives them: on the crate's test
//! corpora at levels 1 and 9, and on streams of a repeated passage whose blocks the open kernel
//! takes two-ended, the oracle pass records every open block's codes with no collision; arm A
//! and arm B (plain and conditioned) each decode every stream to the original bytes, with the
//! same blocks, every arm-B block supplied; and the check decode finds no block whose arm-B
//! bytes differ from arm A's.
#![cfg(feature = "supplied-codes")]
mod common;
use common::{compress, corpora};
use mibz2::supplied_codes::{self as sup, Mode, Rec};
use mibz2::{BzReader, Kernels};
use std::collections::BTreeMap;
use std::io::Read;

fn decode(z: &'static [u8], workers: usize, mode: Mode, cond: bool) -> (Vec<u8>, Vec<Rec>) {
    let _ = sup::take();
    sup::set_mode(mode);
    sup::set_condition(cond);
    let mut r = BzReader::parallel_workers(
        z,
        workers,
        mibz2::parallel::SLOTS_PER_THREAD * workers,
        Kernels::detect(),
        true,
    );
    let mut out = Vec::new();
    r.read_to_end(&mut out).expect("decode");
    drop(r);
    sup::set_mode(Mode::Open);
    sup::set_condition(false);
    (out, sup::take())
}

/// (one test: the mode is process-global)
#[test]
fn both_arms_decode_the_corpora_byte_exactly() {
    let (mut two_ended, mut forward, mut short) = (0, 0, 0);
    let mut sets = corpora(1);
    let text = sets
        .iter()
        .find(|(name, _)| *name == "text")
        .unwrap()
        .1
        .clone();
    // copies of one 60 kB passage with a byte in `rate` changed (tests/lazy_workers.rs'), whose
    // blocks the open kernel takes two-ended (retained rows past 65,536), and the passage
    // repeated exactly (a periodic block: its marked cycle shorter than its retained rows)
    let mut rng = common::Rng(7);
    for (name, rate) in [("copies 1/250", 250), ("copies 1/20", 20), ("periodic", 0)] {
        let mut d = Vec::new();
        while d.len() < 2_000_000 {
            for &b in &text[..60_000] {
                d.push(if rate > 0 && rng.below(rate) == 0 {
                    b'a' + rng.below(26) as u8
                } else {
                    b
                });
            }
        }
        sets.push((name, d));
    }
    for (name, data) in sets {
        for level in [1, 9] {
            let z: &'static [u8] = compress(&data, level).leak();
            sup::clear_store();
            let (out, recs) = decode(z, 3, Mode::Record, false);
            assert!(out == data, "{name} level {level}: the oracle pass's bytes");
            assert_eq!(sup::collisions(), 0, "{name} level {level}: collisions");
            for r in recs.iter().filter(|r| !r.refused) {
                assert!(
                    r.cycle >= 1 && r.cycle <= r.m,
                    "{name} level {level}: {r:?}"
                );
                short += usize::from(r.cycle < r.m);
            }
            for workers in [1, 3] {
                let mut shapes = Vec::new();
                for (mode, cond) in [
                    (Mode::Open, false),
                    (Mode::Supplied, false),
                    (Mode::Open, true),
                    (Mode::Supplied, true),
                ] {
                    let ctx = format!("{name} level {level} workers {workers} {mode:?} {cond}");
                    let (out, recs) = decode(z, workers, mode, cond);
                    assert!(out == data, "{ctx}: the bytes");
                    let mut shape = BTreeMap::new();
                    for r in &recs {
                        assert!(r.mode == mode && r.cond == cond, "{ctx}: {r:?}");
                        *shape.entry((r.n, r.m, r.refused, r.bidi)).or_insert(0) += 1;
                        if r.refused {
                            continue;
                        }
                        match mode {
                            Mode::Open => assert!(!r.supplied && r.cycle == 0, "{ctx}: {r:?}"),
                            _ => assert!(r.supplied && r.cycle >= 1, "{ctx}: {r:?}"),
                        }
                        assert!(r.scratch >= 4 * u64::from(r.m), "{ctx}: {r:?}");
                        if mode == Mode::Supplied && !cond {
                            two_ended += usize::from(r.bidi);
                            forward += usize::from(!r.bidi);
                        }
                    }
                    shapes.push(shape);
                }
                assert!(
                    shapes.windows(2).all(|w| w[0] == w[1]),
                    "{name} level {level} workers {workers}: the blocks differ between arms"
                );
            }
            let _ = sup::mismatches();
            let (out, recs) = decode(z, 3, Mode::Check, false);
            assert!(out == data, "{name} level {level}: the check's bytes");
            assert_eq!(sup::mismatches(), 0, "{name} level {level}: mismatches");
            assert!(recs.iter().all(|r| r.refused || r.supplied));
        }
    }
    eprintln!("{two_ended} two-ended, {forward} forward, {short} short cycles (oracle passes)");
    assert!(
        two_ended > 0 && forward > 0,
        "{two_ended} two-ended, {forward} forward"
    );
}
