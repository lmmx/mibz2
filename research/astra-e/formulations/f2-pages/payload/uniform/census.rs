use mibz2::stream::{Scanner,SliceSource,Event};
pub use mibz2::{bwt,runs,Kernels};
use bwt::{Segmented,InverseBwt};
#[allow(dead_code)]
mod candidate;
fn main() {
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));
    let (mut r, k)=(runs::Runs::default(),Kernels::detect());
    let (mut kernel,mut reference)=(candidate::Candidate::default(),Segmented::default());
    let (mut out,mut want)=(Vec::new(),Vec::new());
    let mut accepted=0;
    println!("kind,block,rows,cuts_or_wave_hits,pages_or_waves,longest_rows,lookups,unary,longest_hits,longest_unary");
    while let Event::Block{index,symbols,..}=scan.next(Vec::new()).unwrap() {
        let n=runs::front(&symbols,&mut r,k).unwrap();
        if n<65536 || r.records().len()*20>=3*n || r.records().len()*64<n {continue}
        assert!(kernel.invert_runs(&r,n,symbols.orig_ptr,&mut out,k));
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
        assert_eq!(out,want);
        kernel.report(index,n);
        accepted+=1;
    }
    assert_eq!(accepted,228);
}
