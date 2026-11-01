//! The trace's phase and feeder marks (feature `trace`; docs/journal/2026-11-01-decoder-attribution-protocol.md
//! §7 C1, C2): through the parallel reader, every inverse BWT has one `WalkStart` between
//! its start and end, each with the thread's clocks at level 2 and none at level 1; and the
//! feeder's hand-offs are traced, alternating, from its start to its end.
//!
//! One test function: the trace's level is process-wide.
#![cfg(feature = "trace")]

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use mibz2::trace::{self, Kind, ThreadTrace, UNREAD};
use mibz2::{BzReader, Kernels};

fn letters(n: usize, seed: u64) -> Vec<u8> {
    let mut x = seed | 1;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            b'a' + (x % 26) as u8
        })
        .collect()
}

fn traced(data: &[u8], plain: &[u8], workers: usize, level: u8) -> Vec<ThreadTrace> {
    trace::set_level(level);
    let mut r = BzReader::parallel_with(
        std::io::Cursor::new(data.to_vec()),
        workers,
        2 * workers,
        Kernels::detect(),
    );
    let mut out = Vec::new();
    r.read_to_end(&mut out).unwrap();
    drop(r);
    assert!(out == plain, "decoded bytes differ");
    // the workers, the scanner and the feeder
    let deadline = Instant::now() + Duration::from_secs(5);
    while trace::finished() < workers + 2 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    let t = trace::take();
    trace::set_level(0);
    t
}

#[test]
fn phase_and_feeder_marks() {
    // three level-1 blocks: two of 100,000 rows (segmented) and one below MIN_SEGMENTED
    // (the plain walk, inside the same kernel)
    let plain = letters(250_000, 5);
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
    e.write_all(&plain).unwrap();
    let data = e.finish().unwrap();
    for level in [1u8, 2] {
        let traces = traced(&data, &plain, 2, level);
        let (mut inverts, mut feeders) = (0, 0);
        for t in &traces {
            let kinds: Vec<Kind> = t.marks.iter().map(|m| m.kind).collect();
            assert_eq!(kinds.first(), Some(&Kind::ThreadStart), "{}", t.name);
            assert_eq!(kinds.last(), Some(&Kind::ThreadEnd), "{}", t.name);
            if t.name == "mibz2-read" {
                feeders += 1;
                let feed: Vec<Kind> = kinds
                    .iter()
                    .copied()
                    .filter(|k| matches!(k, Kind::FeedWrite | Kind::FeedWritten))
                    .collect();
                assert!(!feed.is_empty(), "the feeder handed nothing over");
                assert!(
                    feed.chunks(2)
                        .all(|p| p == [Kind::FeedWrite, Kind::FeedWritten]),
                    "the feeder's marks don't alternate: {feed:?}"
                );
                // its CPU time at its start and end, at level 2
                let ends = [&t.marks[0], t.marks.last().unwrap()];
                assert!(ends.iter().all(|m| (m.cpu_ns != UNREAD) == (level == 2)));
                continue;
            }
            // a worker: DecodeStart, InvertStart, WalkStart, InvertEnd, DecodeEnd, in order
            let mut state = 0;
            let mut cpu = Vec::new();
            for m in &t.marks {
                match (state, m.kind) {
                    (0, Kind::DecodeStart) => state = 1,
                    (1, Kind::InvertStart) => state = 2,
                    (2, Kind::WalkStart) => state = 3,
                    (3, Kind::InvertEnd) => state = 4,
                    (4, Kind::DecodeEnd) => {
                        state = 0;
                        inverts += 1;
                    }
                    (
                        _,
                        Kind::InvertStart
                        | Kind::WalkStart
                        | Kind::InvertEnd
                        | Kind::DecodeStart
                        | Kind::DecodeEnd,
                    ) => {
                        panic!("{}: {:?} out of order (state {state})", t.name, m.kind)
                    }
                    _ => {}
                }
                if matches!(
                    m.kind,
                    Kind::InvertStart | Kind::WalkStart | Kind::InvertEnd
                ) {
                    assert_eq!(
                        m.cpu_ns != UNREAD,
                        level == 2,
                        "{:?} at level {level}",
                        m.kind
                    );
                    if level == 2 {
                        cpu.push(m.cpu_ns);
                    }
                }
            }
            assert_eq!(state, 0, "{}: a block left open", t.name);
            assert!(
                cpu.windows(2).all(|w| w[0] <= w[1]),
                "{}: CPU time not monotone",
                t.name
            );
        }
        assert_eq!(
            inverts, 3,
            "level {level}: every block's inverse BWT, split"
        );
        assert_eq!(feeders, 1, "level {level}: the feeder traced once");
    }
}
