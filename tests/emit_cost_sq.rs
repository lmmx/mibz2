//! (research only, `squared-trace`) MONODROMY record 93's T² arms through production's lazy
//! workers (`OpenCells`), as examples/emit_cost_sq.rs drives them: on the crate's test corpora
//! at levels 1 and 9, every arm decodes every stream to the original bytes (each block's CRC
//! and the stream's checked), with the same blocks and the same two-ended blocks, and the T²
//! walk runs on the two-ended ones.
#![cfg(feature = "squared-trace")]
mod common;
use common::{compress, corpora};
use mibz2::emit_cost::{self, Arm};
use mibz2::{BzReader, Kernels};
use std::collections::BTreeMap;
use std::io::Read;

fn decode(z: &'static [u8], workers: usize, arm: Arm) -> (Vec<u8>, Vec<emit_cost::Rec>) {
    let _ = emit_cost::take();
    emit_cost::set_arm(arm);
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
    emit_cost::set_arm(Arm::Base);
    (out, emit_cost::take())
}

/// (one test: the arm is process-global)
#[test]
fn every_arm_decodes_the_corpora() {
    let mut two_ended = 0;
    // blocks whose T² walk ran, in sq and in sq-shadow
    let mut sq_ran = [0usize; 3];
    // copies of one 60 kB passage with a byte in `rate` changed (tests/lazy_workers.rs'),
    // whose blocks the open kernel takes two-ended (retained rows past 65,536)
    let mut sets = corpora(1);
    let text = sets
        .iter()
        .find(|(name, _)| *name == "text")
        .unwrap()
        .1
        .clone();
    let mut rng = common::Rng(7);
    for (name, rate) in [
        ("copies 1/250", 250),
        ("copies 1/60", 60),
        ("copies 1/20", 20),
    ] {
        let mut d = Vec::new();
        while d.len() < 2_000_000 {
            for &b in &text[..60_000] {
                d.push(if rng.below(rate) == 0 {
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
            for workers in [1, 3] {
                let mut shapes = Vec::new();
                for arm in [Arm::Base, Arm::Sq, Arm::SqShadow, Arm::Base] {
                    let ctx = format!("{name} level {level} workers {workers} {arm:?}");
                    let (out, recs) = decode(z, workers, arm);
                    assert!(out == data, "{ctx}: the bytes");
                    assert!(recs.iter().all(|r| r.arm == arm), "{ctx}: the arm");
                    let mut shape = BTreeMap::new();
                    for r in &recs {
                        *shape.entry((r.n, r.m, r.refused, r.bidi)).or_insert(0) += 1;
                        if arm == Arm::Base {
                            assert!(!r.sq_rounds && r.scratch == 0 && r.square_ns < 1_000_000);
                        }
                        if r.bidi && arm != Arm::Base {
                            assert!(r.scratch >= 4 * u64::from(r.m), "{ctx}: {r:?}");
                        }
                        two_ended += usize::from(r.bidi);
                        sq_ran[arm as usize] += usize::from(r.sq_rounds);
                    }
                    if arm != Arm::Base {
                        eprintln!(
                            "{ctx}: {} blocks, {} two-ended, {} with T² rounds",
                            recs.len(),
                            recs.iter().filter(|r| r.bidi).count(),
                            recs.iter().filter(|r| r.sq_rounds).count()
                        );
                    }
                    shapes.push(shape);
                }
                assert!(
                    shapes.windows(2).all(|w| w[0] == w[1]),
                    "{name} level {level} workers {workers}: the blocks differ between arms"
                );
            }
        }
    }
    assert!(
        two_ended > 0 && sq_ran[1] > 0 && sq_ran[2] > 0,
        "{two_ended} {sq_ran:?}"
    );
}
