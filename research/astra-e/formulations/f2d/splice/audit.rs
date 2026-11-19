use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
#[path = "candidate.rs"]
mod candidate;
fn main() {
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&input));
    let mut runs = runs::Runs::default();
    let mut candidate = candidate::Candidate::default();
    let mut arena = candidate::Audit::default();
    let mut reference = Segmented::default();
    let (mut want, mut out) = (Vec::new(), Vec::new());
    let k = Kernels::detect();
    let mut checked = 0;
    println!("stream,rows,nodes,selectors,all_runs,root_runs,terminal_cells,split_cells,whole_splices,partial_splices,partial_reads,link_capacity_bytes");
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = runs::front(&symbols, &mut runs, k).unwrap();
        let caps = runs.records().len();
        if n < 65536 || caps * 20 >= 3 * n || caps * 64 < n {
            continue;
        }
        let stats = candidate.audit(&runs, n, symbols.orig_ptr, &mut out, &mut arena);
        assert!(reference.invert_runs(&runs, n, symbols.orig_ptr, &mut want, k));
        assert_eq!(out, want);
        checked += n;
        print!("{index},{n}");
        for count in stats {
            print!(",{count}");
        }
        println!();
    }
    assert_eq!(checked, 205197794);
    eprintln!("Every token-root RLE and each of {checked} output bytes agreed");
}
