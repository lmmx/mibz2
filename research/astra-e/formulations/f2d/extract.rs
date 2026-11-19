//! Save ten systematic canonical Wikidata block run streams, including the frozen block.
use mibz2::stream::{Event, Scanner, SliceSource};
use std::{env, fs, path::Path};
fn main() {
    let args: Vec<_> = env::args().collect();
    let data = fs::read(&args[1]).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut runs = mibz2::runs::Runs::default();
    let mut syms = Vec::new();
    while let Event::Block { index, symbols, .. } = scanner.next(syms).unwrap() {
        if [1,41,85,134,176,225,269,313,359,397].contains(&index) {
            let n = mibz2::runs::front(&symbols, &mut runs, mibz2::Kernels::detect()).unwrap();
            let bytes: Vec<_> = runs.records().iter().flat_map(|r| r.to_le_bytes()).collect();
            let p = Path::new(&args[2]).join(format!("{index}.recs"));
            fs::write(p, bytes).unwrap();
            println!("{index},{n},{},{}", symbols.orig_ptr, runs.records().len());
        }
        syms = symbols.syms;
    }
}
