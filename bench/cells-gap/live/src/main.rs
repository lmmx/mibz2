//! Frozen live readers and drain/hash, separate binaries per arm and trace setting.
#![forbid(unsafe_code)]
#![allow(dead_code)] // frozen test helpers and original optional SHA support
use serde_json::json;
use std::io::{self, Read};
include!("../.generated/hash.rs");
#[path = "../.source/examples/ab_cells/mod.rs"]
mod ab_cells;
#[cfg(test)]
#[allow(clippy::manual_repeat_n)] // unchanged frozen helper, not diagnostic code
#[path = "../.source/tests/common/mod.rs"]
mod common;
#[cfg(test)]
#[path = "trace.rs"]
mod trace_contract;
#[cfg(test)]
fn parallel(name: &str) -> bool {
    name == "mibz2"
}

fn drain(mut r: impl Read, h: &mut Hash, mut output: Option<std::fs::File>) -> io::Result<()> {
    use std::io::Write;
    let mut buf = vec![0; 4 << 20];
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                h.update(&buf[..n]);
                if let Some(out) = &mut output {
                    out.write_all(&buf[..n])?;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 || !matches!(args[1].as_str(), "check" | "measure") {
        return Err("usage: cells-live check|measure INPUT [CHECK_OUTPUT]".into());
    }
    let timed = args[1] == "measure";
    if timed && (args.len() != 3 || std::env::var_os("AB_SHA256").is_some()) {
        return Err("measurement forbids output file and AB_SHA256".into());
    }
    let output = args
        .get(3)
        .map(|p| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(p)
        })
        .transpose()?;
    let mut h = Hash::new();
    #[cfg(feature = "live-trace")]
    mibz2::live_diagnostic::reset(timed);
    #[cfg(feature = "live-trace")]
    let whole = mibz2::live_diagnostic::phase("whole_reader_drain");
    let input = std::fs::File::open(&args[2])?;
    #[cfg(feature = "cells-reader")]
    let result = drain(ab_cells::CellsReader::new(input), &mut h, output);
    #[cfg(not(feature = "cells-reader"))]
    let result = drain(mibz2::BzReader::new(input), &mut h, output);
    #[cfg(feature = "live-trace")]
    drop(whole);
    let mut row = json!({"schema": 1, "arm": if cfg!(feature="cells-reader") {"cells"} else {"baseline"},
        "trace": cfg!(feature="live-trace"), "timed": timed, "bytes": h.n, "hash": h.finish(),
        "sha256": h.sha.as_ref().map(Sha256::hex), "error": result.as_ref().err().map(|e| format!("{e:?}"))});
    #[cfg(feature = "live-trace")]
    {
        let r = mibz2::live_diagnostic::take();
        let phases: serde_json::Map<_, _> = r.phases.iter().map(|(name, p)| (name.to_string(), json!({"calls":p.calls,"inclusive_ns":p.inclusive_ns,"exclusive_ns":p.exclusive_ns}))).collect();
        row["phases"] = phases.into();
        row["clock_calls"] = r.clock_calls.into();
        row["counters"] = json!(r.counters);
        row["event_growths"] = r.event_growths.into();
        row["event_capacity"] = r.event_capacity.into();
        row["blocks"] = r.blocks.iter().map(|b| json!({"bit":b.bit,"primary":b.primary,"stored_crc":b.stored_crc,"counters":b.counters,"outcome":b.outcome})).collect();
    }
    // Serialization and log writes occur after the internal whole span. Parent process
    // wall/CPU includes them; process minus internal whole is reported separately.
    row["missing"] = json!([
        "allocator physical traffic",
        "all decoder buffer allocations",
        "log allocation bytes",
        "clock cost isolated from other instrumentation"
    ]);
    println!("{row}");
    result?;
    Ok(())
}

#[cfg(all(test, feature = "live-trace"))]
mod live_tests {
    use super::*;
    use mibz2::live_diagnostic as trace;
    #[test]
    fn untimed_actual_readers_routes_and_crc() {
        let mut routes = std::collections::BTreeSet::new();
        for compressed in common::cells::witnesses() {
            let expected = common::oracle(&compressed).unwrap();
            for cells in [false, true] {
                trace::reset(false);
                let mut h = Hash::new();
                {
                    let _whole = trace::phase("whole_reader_drain");
                    let r: Box<dyn Read> = if cells {
                        Box::new(ab_cells::CellsReader::new(&compressed[..]))
                    } else {
                        Box::new(mibz2::BzReader::new(&compressed[..]))
                    };
                    drain(r, &mut h, None).unwrap();
                }
                let report = trace::take();
                let mut reference = Hash::new();
                reference.update(&expected);
                assert_eq!((h.n, h.finish()), (reference.n, reference.finish()));
                assert_eq!(report.clock_calls, 0);
                for b in &report.blocks {
                    assert_eq!(b.counters["computed_crc"], b.stored_crc as u64);
                    if cells && b.counters.contains_key("outer_runs") {
                        routes.insert(b.outcome.clone());
                        if b.outcome.starts_with("Err(") {
                            assert_eq!(b.counters["fallback_calls"], 1);
                        }
                        assert!(
                            b.counters.get("attempted_cells").copied().unwrap_or(0)
                                <= b.counters["budget"]
                        );
                    }
                }
            }
        }
        assert!(
            routes.iter().any(|x| x.starts_with("Ok(")),
            "admitted reach"
        );
        assert!(routes.contains("Err(Density)"));
        assert!(routes.contains("Err(Floor)"));
        assert!(routes.contains("Err(Budget)"));
    }
    #[test]
    fn long_factor_trace_keeps_overlap_recopy_and_period() {
        let mut word = [vec![b'a'; 9], vec![b'b'; 9]].concat().repeat(3);
        word.push(b'c');
        let n = word.len();
        let mut rotations: Vec<_> = (0..n).collect();
        rotations.sort_by_key(|&s| (0..n).map(|i| word[(s + i) % n]).collect::<Vec<_>>());
        let column: Vec<_> = rotations.iter().map(|&s| word[(s + n - 1) % n]).collect();
        let large: Vec<_> = column.iter().flat_map(|&b| [b; 10]).collect();
        let z = common::craft(9, &[common::Spec::of(large, 0)], None);
        trace::reset(false);
        let mut h = Hash::new();
        {
            let _whole = trace::phase("whole_reader_drain");
            drain(ab_cells::CellsReader::new(&z[..]), &mut h, None).unwrap();
        }
        let r = trace::take();
        assert_eq!(r.clock_calls, 0);
        let mut expected = Hash::new();
        expected.update(&common::oracle(&z).unwrap());
        assert_eq!(h.finish(), expected.finish());
        let c = &r.blocks[0].counters;
        assert_eq!(c["attempted_cells"], 57);
        assert_eq!(c["induction_copy_bytes"], 57 * 32);
        assert!(c["writer_long_copy_bytes"] > 0);
        assert!(c["writer_repeat_copy_bytes"] > 0);
        assert!(c["period"] < c["rows"]);
    }
    #[test]
    fn untimed_errors_unwind_all_phases() {
        for data in [
            b"BZh9broken".to_vec(),
            common::craft(
                9,
                &[common::Spec {
                    crc: Some(123),
                    ..common::Spec::of(vec![0; 25], 1)
                }],
                None,
            ),
        ] {
            trace::reset(false);
            {
                let _whole = trace::phase("whole_reader_drain");
                assert!(drain(
                    ab_cells::CellsReader::new(&data[..]),
                    &mut Hash::new(),
                    None
                )
                .is_err());
            }
            let report = trace::take();
            assert_eq!(report.clock_calls, 0);
        }
    }
}
