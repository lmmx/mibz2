//! (research only, `scan-oracle`) bench-four-arm's arms through production's lazy workers
//! (`OpenCells`), as examples/four_arm.rs drives them: on the crate's test corpora at levels 1
//! and 9, and on several streams end to end, the oracle pass (both sides recording) records
//! every block once with no conflict and nothing missing; each of the four arms decodes every
//! input to the original bytes at 1 and 3 workers, the replaying arms delivering every recorded
//! block and the scanning arms none; a replay from an emptied store delivers nothing (the
//! harness's check then fails on the size); and a cancelled replay (the reader dropped early)
//! ends without hanging.
#![cfg(feature = "scan-oracle")]
mod common;
use common::{compress, corpora};
use mibz2::scan_oracle as scan;
use mibz2::supplied_codes::{self as sup, Mode};
use mibz2::{BzReader, Kernels};
use std::io::Read;

fn decode(z: &'static [u8], workers: usize, scanner: scan::Mode, emission: Mode) -> Vec<u8> {
    let _ = sup::take();
    let _ = scan::take_replayed();
    sup::set_mode(emission);
    scan::set_mode(scanner);
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
    scan::set_mode(scan::Mode::Scan);
    let _ = sup::take();
    out
}

/// (one test: the modes are process-global)
#[test]
fn the_four_arms_decode_byte_exactly() {
    let mut sets = corpora(1);
    // several streams end to end (the corpus's repeated copies): the block index counts across
    // streams, and the copies' blocks share one entry
    let one = compress(&sets[0].1, 9);
    let mut multi = Vec::new();
    for _ in 0..3 {
        multi.extend_from_slice(&one);
    }
    let multi_data: Vec<u8> = (0..3).flat_map(|_| sets[0].1.clone()).collect();
    sets.truncate(4);
    let mut inputs: Vec<(String, &'static [u8], Vec<u8>)> = Vec::new();
    for (name, data) in &sets {
        for level in [1, 9] {
            inputs.push((
                format!("{name} level {level}"),
                compress(data, level).leak(),
                data.clone(),
            ));
        }
    }
    inputs.push(("three streams".into(), multi.leak(), multi_data));
    for (name, z, data) in inputs {
        sup::clear_store();
        scan::clear_store();
        let out = decode(z, 3, scan::Mode::Record, Mode::Record);
        assert!(out == data, "{name}: the oracle pass's bytes");
        let (blocks, distinct, bytes, conflicts, missing) = scan::store_size();
        assert!(
            blocks >= 1 && distinct >= 1 && distinct <= blocks && bytes > 0,
            "{name}"
        );
        assert_eq!((conflicts, missing), (0, 0), "{name}");
        assert_eq!(sup::collisions(), 0, "{name}");
        if name == "three streams" {
            assert_eq!(
                blocks,
                3 * distinct,
                "{name}: the copies share their entries"
            );
        }
        for workers in [1, 3] {
            for (arm, scanner, emission) in [
                ("A", scan::Mode::Scan, Mode::Open),
                ("B", scan::Mode::Scan, Mode::Supplied),
                ("C", scan::Mode::Replay, Mode::Open),
                ("D", scan::Mode::Replay, Mode::Supplied),
            ] {
                let out = decode(z, workers, scanner, emission);
                assert!(out == data, "{name} workers {workers} arm {arm}: the bytes");
                let replayed = scan::take_replayed();
                let want = if scanner == scan::Mode::Replay {
                    blocks
                } else {
                    0
                };
                assert_eq!(replayed, want, "{name} workers {workers} arm {arm}");
            }
        }
        // the check decode: the replay with the emission compared block by block
        let _ = sup::mismatches();
        let out = decode(z, 3, scan::Mode::Replay, Mode::Check);
        assert!(out == data, "{name}: the check's bytes");
        assert_eq!(sup::mismatches(), 0, "{name}: mismatches");
    }
    // an emptied store: the replay delivers nothing, and the decode ends (short)
    scan::clear_store();
    let z: &'static [u8] = compress(&sets[0].1, 9).leak();
    let out = decode(z, 2, scan::Mode::Replay, Mode::Open);
    assert!(out.is_empty());
    assert_eq!(scan::take_replayed(), 0);
    // a replay cancelled part way (the reader dropped after one read): no hang
    let big: Vec<u8> = (0..8).flat_map(|_| sets[0].1.clone()).collect();
    let z: &'static [u8] = compress(&big, 1).leak();
    scan::clear_store();
    assert!(decode(z, 2, scan::Mode::Record, Mode::Open) == big);
    scan::set_mode(scan::Mode::Replay);
    let mut r = BzReader::parallel_workers(z, 2, 2, Kernels::detect(), true);
    let mut buf = [0u8; 1000];
    let _ = r.read(&mut buf).expect("a first read");
    drop(r);
    scan::set_mode(scan::Mode::Scan);
}
