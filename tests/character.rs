//! (research only, `character`) bench-character's ledgers through production's lazy workers, as
//! examples/character.rs drives them: on the crate's test corpora at levels 1 and 9 and on
//! several streams end to end, at 1, 3 and 8 workers, the decode's bytes with the ledgers on
//! are the original's, and:
//! - every thread's ledger starts and ends, its times never go back, and its activities nest
//!   (every end closes the activity on top of its stack, and the stack is empty at the end);
//! - every block is issued, enqueued, taken by a worker, routed, published ready and taken by
//!   the reader exactly once (its places 0..N−1);
//! - every buffer is empty at the end, its arrivals the blocks, and its live occupancy
//!   integral exactly its blocks' residence (Little's law, an identity of integers here);
//! - with the ledgers off, nothing is recorded.
#![cfg(feature = "character")]
mod common;
use common::{compress, corpora};
use mibz2::character::{self as ch, Tag, BUFFERS, OCCS};
use mibz2::{BzReader, Kernels};
use std::io::Read;
use std::time::{Duration, Instant};

fn decode(z: &'static [u8], workers: usize, on: bool) -> Vec<u8> {
    ch::set_on(on);
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
    out
}

/// (one test: the switch is process-global)
#[test]
fn the_ledgers_account_for_every_block_and_change_nothing() {
    let sets = corpora(1);
    let mut inputs: Vec<(String, &'static [u8], Vec<u8>)> = Vec::new();
    for (name, data) in sets.iter().take(4) {
        for level in [1, 9] {
            inputs.push((
                format!("{name} level {level}"),
                compress(data, level).leak(),
                data.clone(),
            ));
        }
    }
    let one = compress(&sets[0].1, 1);
    let three: Vec<u8> = (0..3).flat_map(|_| one.clone()).collect();
    let data3: Vec<u8> = (0..3).flat_map(|_| sets[0].1.clone()).collect();
    inputs.push(("three streams".into(), three.leak(), data3));
    for (name, z, data) in inputs {
        for workers in [1, 3, 8] {
            let ctx = format!("{name} workers {workers}");
            let out = decode(z, workers, true);
            let end = ch::now_ns();
            assert!(out == data, "{ctx}: the bytes");
            let deadline = Instant::now() + Duration::from_secs(10);
            while ch::finished() < workers + 2 && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(1));
            }
            let occ = ch::take_occ(end);
            ch::set_on(false);
            let ledgers = ch::take();
            assert_eq!(ledgers.len(), workers + 3, "{ctx}: the threads' ledgers");
            // each thread's timeline
            for l in &ledgers {
                let ev = &l.events;
                assert_eq!(
                    ev.first().map(|e| e.tag),
                    Some(Tag::Start as u8),
                    "{ctx}: {}",
                    l.name
                );
                assert_eq!(
                    ev.last().map(|e| e.tag),
                    Some(Tag::End as u8),
                    "{ctx}: {}",
                    l.name
                );
                let mut stack = Vec::new();
                for w in ev.windows(2) {
                    assert!(w[0].ns <= w[1].ns, "{ctx}: {} goes back in time", l.name);
                }
                for e in ev {
                    match e.edge {
                        1 => stack.push(e.tag),
                        2 => assert_eq!(stack.pop(), Some(e.tag), "{ctx}: {} nests", l.name),
                        _ => {}
                    }
                }
                assert!(stack.is_empty(), "{ctx}: {} ends open", l.name);
            }
            // each block's points, once each
            let points = [
                Tag::Issued,
                Tag::Enqueued,
                Tag::Got,
                Tag::Route,
                Tag::Ready,
                Tag::Taken,
            ];
            let issued = ledgers
                .iter()
                .flat_map(|l| &l.events)
                .filter(|e| e.edge == 0 && e.tag == Tag::Issued as u8)
                .count();
            assert!(issued > 0, "{ctx}: no blocks");
            let mut residence = [0u128; OCCS];
            let mut at = vec![[0u64; 6]; issued];
            for (j, t) in points.iter().enumerate() {
                let mut places: Vec<u32> = ledgers
                    .iter()
                    .flat_map(|l| &l.events)
                    .filter(|e| e.edge == 0 && e.tag == *t as u8)
                    .map(|e| {
                        at[e.place as usize][j] = e.ns;
                        e.place
                    })
                    .collect();
                places.sort();
                assert_eq!(
                    places,
                    (0..issued as u32).collect::<Vec<_>>(),
                    "{ctx}: {t:?} once a block"
                );
            }
            let bytes: Vec<u64> = {
                let mut b = vec![0u64; issued];
                for e in ledgers.iter().flat_map(|l| &l.events) {
                    if e.edge == 0 && e.tag == Tag::Ready as u8 {
                        b[e.place as usize] = e.x;
                    }
                }
                b
            };
            for (p, t) in at.iter().enumerate() {
                // (issued, enqueued, got, route, ready, taken)
                residence[0] += u128::from(t[5] - t[0]);
                residence[1] += u128::from(t[2] - t[1]);
                residence[2] += u128::from(t[4] - t[2]);
                residence[3] += u128::from(t[5] - t[4]);
                residence[4] += u128::from(t[5] - t[4]) * u128::from(bytes[p]);
            }
            for i in 0..OCCS {
                assert_eq!(occ[i].n, 0, "{ctx}: {} not empty at the end", BUFFERS[i]);
                if i < 4 {
                    assert_eq!(occ[i].arrivals, issued as u64, "{ctx}: {}", BUFFERS[i]);
                }
                assert_eq!(
                    occ[i].integral, residence[i],
                    "{ctx}: Little's law for {}",
                    BUFFERS[i]
                );
            }
            // the ledgers off
            let out = decode(z, workers, false);
            assert!(out == data, "{ctx}: the bytes, the ledgers off");
            std::thread::sleep(Duration::from_millis(20));
            assert!(ch::take().is_empty(), "{ctx}: ledgers with the ledgers off");
        }
    }
}
