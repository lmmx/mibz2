//! Two independent normal decoder pools overlap. Every byte is compared to the corpus supplied by Python.
use std::io::{Cursor, Read};
use std::sync::{Arc, Barrier};
pub use mibz2::{bwt, runs, Kernels};
#[path = "observe.rs"]
mod observe;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let compressed = std::fs::read(&args[1]).unwrap();
    let expected = std::fs::read(&args[2]).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    std::thread::scope(|scope| {
        for w in [7, 10] {
            let (compressed, expected) = (compressed.clone(), &expected);
            let barrier = Arc::clone(&barrier);
            scope.spawn(move || {
                let mut reader = mibz2::BzReader::parallel_workers_with::<observe::Candidate>(
                    Cursor::new(compressed), w, 4*w, Kernels::detect(),
                );
                // Both pools exist before either reader can drop its scratch.
                barrier.wait();
                let mut decoded = Vec::new();
                reader.read_to_end(&mut decoded).unwrap();
                assert_eq!(&decoded, expected);
                println!("W{w} exact {} bytes", decoded.len());
            });
        }
    });
}
