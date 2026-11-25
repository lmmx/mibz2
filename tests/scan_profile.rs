//! (research only, `scan-profile`) bench-scan-profile's timers through production's lazy workers,
//! as examples/scan_profile.rs drives them: on the crate's test corpora at levels 1 and 9 and on
//! several streams end to end, at 1 and 3 workers, the decode's bytes with the timers on are the
//! original's; every block is recorded once (indices 0..N-1, N the blocks the stream holds), with
//! its symbols, selectors and bits nonzero and its read's phases within the read; every worker
//! records its end, its jobs summing to the blocks; and with the timers off nothing is recorded.
#![cfg(feature = "scan-profile")]
mod common;
use common::{compress, corpora};
use mibz2::scan_profile as sp;
use mibz2::{BzReader, Kernels};
use std::io::Read;

fn decode(z: &'static [u8], workers: usize, on: bool) -> Vec<u8> {
    let _ = sp::take();
    sp::set_on(on);
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
    sp::set_on(false);
    out
}

/// (one test: the switch is process-global)
#[test]
fn the_timers_record_every_block_and_change_nothing() {
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
        for workers in [1, 3] {
            let ctx = format!("{name} workers {workers}");
            let out = decode(z, workers, true);
            assert!(out == data, "{ctx}: the bytes");
            let (blocks, ws) = sp::take();
            assert!(!blocks.is_empty(), "{ctx}: no blocks");
            let mut idx: Vec<u64> = blocks.iter().map(|b| b.index).collect();
            idx.sort();
            assert_eq!(
                idx,
                (0..blocks.len() as u64).collect::<Vec<_>>(),
                "{ctx}: the indices"
            );
            for b in &blocks {
                assert!(
                    b.syms > 0 && b.selectors > 0 && b.bits > 0 && b.tables > 0,
                    "{ctx}: {b:?}"
                );
                assert!(b.switches < b.selectors, "{ctx}: {b:?}");
                let phases: u64 = b.phase_ns.iter().sum();
                assert!(
                    phases <= b.read_ns,
                    "{ctx}: the phases past the read: {b:?}"
                );
                assert!(b.input_wait_ns <= b.read_ns, "{ctx}: {b:?}");
            }
            assert_eq!(ws.len(), workers, "{ctx}: the workers' records");
            assert_eq!(
                ws.iter().map(|w| w.jobs).sum::<u64>(),
                blocks.len() as u64,
                "{ctx}: jobs"
            );
            assert!(ws.iter().all(|w| w.idle_ns <= w.life_ns), "{ctx}: {ws:?}");
            let out = decode(z, workers, false);
            assert!(out == data, "{ctx}: the bytes, the timers off");
            let (blocks, ws) = sp::take();
            assert!(
                blocks.is_empty() && ws.is_empty(),
                "{ctx}: records with the timers off"
            );
        }
    }
}
