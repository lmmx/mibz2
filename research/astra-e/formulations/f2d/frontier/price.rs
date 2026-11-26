use bwt::{InverseBwt, LazyCells, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[path = "batched/candidate.rs"]
mod batched;
#[path = "combined/candidate.rs"]
mod combined;
#[path = "old/candidate.rs"]
mod original;
#[path = "new/candidate.rs"]
mod owned;
#[repr(C)]
struct Timespec {
    s: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, ts: *mut Timespec) -> i32;
}
#[inline(never)]
fn thread_cpu_ns() -> u64 {
    let mut ts = Timespec { s: 0, ns: 0 };
    // Linux/x86-64 CLOCK_THREAD_CPUTIME_ID; an exclusive stack timespec is writable.
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    ts.s as u64 * 1_000_000_000 + ts.ns as u64
}
fn main() {
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&input));
    let mut r = runs::Runs::default();
    let (mut one, mut two, mut lazy) = (
        original::Candidate::default(),
        owned::Candidate::default(),
        LazyCells::default(),
    );
    let mut frontier = batched::Candidate::default();
    let mut combo = combined::Candidate::default();
    let (mut reference, mut want, mut out) = (Segmented::default(), Vec::new(), Vec::new());
    let k = Kernels::detect();
    let mut ordinal = 0;
    println!("stream,round,arm,prep_mark_ns,discover_ns,merge_ns,residual_output_ns,total_ns");
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        // Compare the same admitted input, excluding the density-refused control.
        if !selected || r.records().len() * 20 >= 3 * n || r.records().len() * 64 < n {
            continue;
        }
        let p = symbols.orig_ptr;
        assert!(reference.invert_runs(&r, n, p, &mut want, k));
        for round in 0..26 {
            for offset in 0..5 {
                let arm = (round + offset) % 5;
                let phases = match arm {
                    0 => one.measure(&r, n, p, &mut out, true),
                    1 => two.measure(&r, n, p, &mut out, true),
                    2 => frontier.measure(&r, n, p, &mut out, true),
                    3 => combo.measure(&r, n, p, &mut out, true),
                    _ => {
                        let t = thread_cpu_ns();
                        assert!(lazy.invert_runs(&r, n, p, &mut out, k));
                        std::hint::black_box(&out);
                        [0, 0, 0, thread_cpu_ns() - t]
                    }
                };
                assert_eq!(out, want);
                if round != 0 {
                    println!(
                        "{index},{round},{},{},{},{},{},{}",
                        ["f2c", "learn", "batched", "certfront", "lazy"][arm],
                        phases[0],
                        phases[1],
                        phases[2],
                        phases[3],
                        phases.iter().sum::<u64>()
                    );
                }
            }
        }
    }
}
