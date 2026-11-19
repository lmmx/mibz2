use bwt::{InverseBwt, LazyCells, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code)]
#[path = "kernel.rs"]
mod kernel;
use std::time::Instant;

#[repr(C)]
struct Timespec {
    sec: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, out: *mut Timespec) -> i32;
}
fn thread_ns() -> u64 {
    let mut ts = Timespec { sec: 0, ns: 0 };
    // CLOCK_THREAD_CPUTIME_ID, on the Linux/x86-64 benchmark host.
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    ts.sec as u64 * 1_000_000_000 + ts.ns as u64
}
fn main() {
    let corpus = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&corpus));
    let mut records = runs::Runs::default();
    let k = Kernels::detect();
    let (mut endpoint, mut trace) = (kernel::Kernel::default(), kernel::Kernel::default());
    let (mut oracle, mut lazy) = (Segmented::default(), LazyCells::default());
    let (mut want, mut out) = (Vec::new(), Vec::new());
    let mut ordinal = 0;
    println!("stream,round,arm,rows,section_rows,plus_mark,complement_strips,endpoint_nodes,endpoint_steps,rules,bands,spelling_bytes,table_bytes,thread_ns,wall_ns");
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut records, k).unwrap();
        if n < 65536 {
            continue;
        }
        let chosen = ordinal % 24 == 0;
        ordinal += 1;
        let rec = records.records();
        let p = symbols.orig_ptr;
        if rec.len() * 20 >= 3 * n || rec.len() * 64 < n {
            continue;
        }
        assert!(oracle.invert_runs(&records, n, p, &mut want, k));
        // Record one warmup with each arm on all 228 admissible blocks.
        let rounds = if chosen { 13 } else { 1 };
        for round in 0..rounds {
            for offset in 0..3 {
                let arm = (round + offset) % 3;
                let start = Instant::now();
                let cpu = thread_ns();
                let (section, stats, bytes) = if arm != 2 {
                    let s = kernel::Kernel::section(rec, br#"":""#).unwrap();
                    let (counts, bytes) = if arm == 0 {
                        let counts = endpoint.run::<true, true, false>(rec, n, p, s, &mut out);
                        (counts, endpoint.sizes().0)
                    } else {
                        let counts = trace.run::<false, true, false>(rec, n, p, s, &mut out);
                        (counts, trace.sizes().0)
                    };
                    (s.1 - s.0, counts, bytes)
                } else {
                    assert!(lazy.invert_runs(&records, n, p, &mut out, k));
                    (0, kernel::Counts::default(), 0)
                };
                std::hint::black_box(&out);
                let cpu = thread_ns() - cpu;
                let elapsed = start.elapsed().as_nanos();
                assert_eq!(out, want);
                println!(
                    "{index},{round},{},{n},{section},{},{},{},{},{},{},{bytes},{},{cpu},{elapsed}",
                    ["endpoint", "trace", "lazy"][arm],
                    stats.retained,
                    stats.thick,
                    stats.open_nodes,
                    stats.steps,
                    stats.rules,
                    stats.bands,
                    stats.table_bytes
                );
            }
        }
    }
    assert_eq!(ordinal, 233);
}
