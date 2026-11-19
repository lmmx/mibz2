#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};
use std::io::Write;
#[path="vendor/research/astra-e/narrow-short/constructor_ahead.rs"] mod writer;
include!("gate.rs");
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let raw=std::fs::read(&args[1]).unwrap();
    let mut file=std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let mut scan=Scanner::new(SliceSource(&raw));
    let mut worker=writer::Kernel::default();
    let mut base=Segmented::default();
    let (mut out,mut expected)=(Vec::new(),Vec::new());
    let k=Kernels::detect();
    let (mut count,mut total)=(0,0);
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;
        if r.records().len()*20>=3*n && (n<65536 || !fits(r.records(),p)){continue;}
        assert!(base.invert_runs(&r,n,p,&mut expected,k));
        worker.tape(&r,n,p,index as u32,stream as u32,&mut out,&mut file);
        assert_eq!(&out,&expected);
        count+=1;total+=out.len();
    }
    file.flush().unwrap();
    assert_eq!((count,total),(233,209697662));
    eprintln!("{count} marked real inverses, {total} emitted coded bytes, all band codes, queue monotonicities and output bytes checked");
}
