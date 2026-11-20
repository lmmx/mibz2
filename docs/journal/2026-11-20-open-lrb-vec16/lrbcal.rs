//! (research/open-lrb-vec16) Best-of-N per block: the front, the open kernel on its records,
//! the fused route (`OpenCells::invert_symbols`), and Cells on the records.
//! lrbcal FILE.bz2 REPS BLOCK...
use mibz2::bwt::{Cells, InverseBwt, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::time::Instant;
fn best<F: FnMut()>(reps: usize, mut f: F) -> f64 {
    (0..reps)
        .map(|_| {
            let s = Instant::now();
            f();
            s.elapsed().as_secs_f64() * 1e6
        })
        .fold(f64::INFINITY, f64::min)
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let reps: usize = a[2].parse().unwrap();
    let want: Vec<usize> = a[3..].iter().map(|s| s.parse().unwrap()).collect();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut i = 0;
    println!("block\tfront_us\topen_us\tfront+open\tfused_us\tfused/(f+o)\tcells_us\tf+cells");
    while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).unwrap() {
        if want.contains(&i) {
            let (mut o, mut of, mut c) =
                (OpenCells::default(), OpenCells::default(), Cells::default());
            let (mut r, mut rf) = (Runs::default(), Runs::default());
            let (mut out, mut out2) = (Vec::new(), Vec::new());
            let mut n = 0;
            let p = b.orig_ptr;
            let front = best(reps, || n = runs::front(&b, &mut r, k).unwrap());
            let open = best(reps, || assert!(o.invert_runs(&r, n, p, &mut out, k)));
            let fused = best(reps, || {
                assert!(of.invert_symbols(&b, &mut rf, k, &mut out2).is_some())
            });
            assert_eq!(out, out2);
            let cells = best(reps, || assert!(c.invert_runs(&r, n, p, &mut out2, k)));
            assert_eq!(out, out2);
            println!(
                "{i}\t{front:.1}\t{open:.1}\t{:.1}\t{fused:.1}\t{:.3}\t{cells:.1}\t{:.1}",
                front + open,
                fused / (front + open),
                front + cells
            );
        }
        i += 1;
    }
}
