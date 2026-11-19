use bwt::{InverseBwt, LazyCells, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::io::Write;
#[path = "f2d/candidate.rs"]
mod certified;
#[path = "f2c/candidate.rs"]
mod original;
#[repr(C)]
struct Timespec {
    seconds: i64,
    nanoseconds: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, ts: *mut Timespec) -> i32;
}
#[inline(never)]
fn thread_cpu_ns() -> u64 {
    let mut ts = Timespec {
        seconds: 0,
        nanoseconds: 0,
    };
    // CLOCK_THREAD_CPUTIME_ID; the stack owns ts through the call.
    assert_eq!(unsafe { clock_gettime(3, &mut ts) }, 0);
    (ts.seconds as u64) * 1_000_000_000 + (ts.nanoseconds as u64)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let input = std::fs::read(&args[1]).unwrap();
    let mut census = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    writeln!(census, "stream,rows,primary,caps,gate_admitted,exact_arms").unwrap();
    let mut scan = Scanner::new(SliceSource(&input));
    let mut runs = runs::Runs::default();
    let mut original = original::Candidate::default();
    let mut single = certified::Candidate::default();
    let mut eight = certified::Solver::<true>::default();
    let mut lazy = LazyCells::default();
    let mut reference = Segmented::default();
    let (mut want, mut out) = (Vec::new(), Vec::new());
    let k = Kernels::detect();
    let (mut large, mut admitted, mut checked_bytes) = (0, 0, 0);
    println!("stream,round,arm,prep_mark_ns,discover_ns,merge_ns,residual_output_ns,total_ns");
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut runs, k).unwrap();
        if n < 65536 {
            continue;
        }
        let sampled = large % 24 == 0;
        large += 1;
        let caps = runs.records().len();
        let active = caps * 20 < 3 * n && caps * 64 >= n;
        let p = symbols.orig_ptr;
        assert!(reference.invert_runs(&runs, n, p, &mut want, k));
        assert!(original.invert_runs(&runs, n, p, &mut out, k));
        assert_eq!(out, want);
        assert!(single.invert_runs(&runs, n, p, &mut out, k));
        assert_eq!(out, want);
        assert!(eight.invert_runs(&runs, n, p, &mut out, k));
        assert_eq!(out, want);
        writeln!(census, "{index},{n},{p},{caps},{active},3").unwrap();
        if !active {
            continue;
        }
        admitted += 1;
        checked_bytes += n;
        if !sampled {
            continue;
        }
        for round in 0..=20 {
            for offset in 0..4 {
                let arm = (round + offset) % 4;
                let phases = match arm {
                    0 => original.measure(&runs, n, p, &mut out, true),
                    1 => single.measure(&runs, n, p, &mut out, true),
                    2 => eight.measure(&runs, n, p, &mut out, true),
                    _ => {
                        let t = thread_cpu_ns();
                        assert!(lazy.invert_runs(&runs, n, p, &mut out, k));
                        std::hint::black_box(&out);
                        [0, 0, 0, thread_cpu_ns() - t]
                    }
                };
                assert_eq!(out, want);
                if round > 0 {
                    println!(
                        "{index},{round},{},{},{},{},{},{}",
                        ["f2c", "learn", "eight", "lazy"][arm],
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
    eprintln!("{large} large blocks, {admitted} density admissions, {checked_bytes} admitted bytes, all three candidate outputs agree with Segmented");
    assert_eq!((large, admitted, checked_bytes), (233, 228, 205197794));
}
