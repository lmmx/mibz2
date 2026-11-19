//! Exact counts on the original retained run and band schedule.
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code)]
#[path = "candidate.rs"]
mod spans;

fn record<const MODE: u8>(
    kernel: &mut spans::kernel::Kernel, index: u64, r: &runs::Runs,
    n: usize, p: u32, out: &mut Vec<u8>
) {
    let s = kernel.run::<true, MODE, false>(r.records(), n, p, 16, out);
    println!("{index},{MODE},{n},{},{},{},{},{},{},{},{},{}",
        s.retained,s.bands,s.queue_input,s.queue_stored,s.queue_joined_rows,
        s.queue_cap,s.intersections,s.singletons,s.queue_capacity);
}

fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut kernels: [spans::kernel::Kernel;3] = Default::default();
    let mut r = runs::Runs::default();
    let mut answers: [Vec<u8>;3] = Default::default();
    println!("ordinal,mode,rows,retained,bands,input_spans,stored,joined_rows,cap_boundaries,intersections,singletons,capacity_bytes");
    while let Event::Block { index, symbols, .. } = scanner.next(Vec::new()).unwrap() {
        if !runs::takes_runs(&symbols) { continue; }
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        if r.records().len() * 20 >= n * 3 { continue; }
        record::<1>(&mut kernels[0], index, &r, n, symbols.orig_ptr, &mut answers[0]);
        record::<2>(&mut kernels[1], index, &r, n, symbols.orig_ptr, &mut answers[1]);
        record::<3>(&mut kernels[2], index, &r, n, symbols.orig_ptr, &mut answers[2]);
        assert_eq!(answers[0], answers[1]);
        assert_eq!(answers[1], answers[2]);
    }
}
