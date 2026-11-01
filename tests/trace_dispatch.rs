//! The dispatch marks (feature `trace`; docs/journal/2026-11-01-dispatch-causality-protocol.md §4):
//! through the parallel reader, each worker's cycle is marked in its code order
//! (`JobLock`, `JobRecv`, `RecvDone`, `JobGot`, `BufLockGot`, `DecodeStart` … `DecodeEnd`,
//! `PubLockGot`, `Ready`, `Notified`); every thread carries its own OS thread id; the
//! condition variable's waits and wakes pair up; nothing is dropped; and a clock anchor
//! carries CLOCK_MONOTONIC on the epoch's clock.
//!
//! One test function: the trace's level is process-wide.
#![cfg(feature = "trace")]

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use mibz2::trace::{self, Kind, UNREAD};
use mibz2::{BzReader, Kernels};

#[test]
fn dispatch_marks() {
    let mut x = 7u64;
    let plain: Vec<u8> = (0..350_000)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            b'a' + (x % 26) as u8
        })
        .collect();
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
    e.write_all(&plain).unwrap();
    let data = e.finish().unwrap();
    let workers = 3;

    trace::set_level(1);
    let epoch = trace::epoch_monotonic_ns();
    trace::anchor();
    let mut r = BzReader::parallel_with(
        std::io::Cursor::new(data),
        workers,
        2 * workers,
        Kernels::detect(),
    );
    let mut out = Vec::new();
    r.read_to_end(&mut out).unwrap();
    drop(r);
    assert!(out == plain, "decoded bytes differ");
    trace::anchor();
    let deadline = Instant::now() + Duration::from_secs(5);
    while trace::finished() < workers + 2 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    let traces = trace::take();
    trace::set_level(0);

    assert!(epoch != UNREAD && epoch > 0, "no CLOCK_MONOTONIC epoch");
    let mut tids: Vec<u64> = traces.iter().map(|t| t.tid).collect();
    assert!(
        tids.iter().all(|&t| t != UNREAD && t > 0),
        "a thread without its OS id"
    );
    tids.sort_unstable();
    tids.dedup();
    assert_eq!(tids.len(), traces.len(), "thread ids not distinct");
    assert!(traces.iter().all(|t| t.dropped == 0), "marks dropped");

    // the cycle, in code order, on every worker
    let cycle = [
        Kind::JobLock,
        Kind::JobRecv,
        Kind::RecvDone,
        Kind::JobGot,
        Kind::BufLockGot,
        Kind::DecodeStart,
        Kind::DecodeEnd,
        Kind::PubLockGot,
        Kind::Ready,
        Kind::Notified,
    ];
    let mut blocks = 0;
    for t in traces
        .iter()
        .filter(|t| t.name.starts_with("mibz2-") && t.name[6..].parse::<usize>().is_ok())
    {
        let seq: Vec<Kind> = t
            .marks
            .iter()
            .map(|m| m.kind)
            .filter(|k| cycle.contains(k))
            .collect();
        // each full cycle, then the last JobLock, JobRecv, RecvDone (the channel closed)
        let full = (seq.len() - 3) / cycle.len();
        for (i, c) in seq.chunks(cycle.len()).take(full).enumerate() {
            assert_eq!(c, &cycle[..], "{}: cycle {i} out of order", t.name);
        }
        assert_eq!(
            &seq[full * cycle.len()..],
            &cycle[..3],
            "{}: its end",
            t.name
        );
        blocks += full;
    }
    assert_eq!(blocks, 4, "every block decoded once");

    // waits and wakes pair up on the reader and the scanner
    for t in &traces {
        let w: Vec<Kind> = t
            .marks
            .iter()
            .map(|m| m.kind)
            .filter(|k| matches!(k, Kind::CvWait | Kind::CvWoke))
            .collect();
        assert!(
            w.chunks(2).all(|p| p == [Kind::CvWait, Kind::CvWoke]),
            "{}: CvWait/CvWoke unpaired",
            t.name
        );
    }

    // the anchors: on the caller's thread, their CLOCK_MONOTONIC within the epoch's clock
    let me = traces
        .iter()
        .find(|t| t.marks.iter().any(|m| m.kind == Kind::ClockAnchor))
        .expect("anchors");
    let a: Vec<_> = me
        .marks
        .iter()
        .filter(|m| m.kind == Kind::ClockAnchor)
        .collect();
    assert_eq!(a.len(), 2);
    for m in a {
        // the mark's own time on the epoch's clock against the CLOCK_MONOTONIC it read
        let d = (epoch + m.ns) as i128 - m.len as i128;
        assert!(d.abs() < 1_000_000, "anchor {d} ns off the epoch's clock");
    }
}
