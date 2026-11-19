#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
#[path="source/research/astra-e/narrow-short/constructor_ahead.rs"] mod writer;
include!("gate.rs");

fn main() {
    let args:Vec<_>=std::env::args().collect();
    let input=std::fs::read(&args[1]).unwrap();
    let oracle=std::fs::read(&args[2]).unwrap();
    let mut scan=Scanner::new(SliceSource(&input));
    let k=Kernels::detect();
    let mut stock=mibz2::Scratch::<bwt::Segmented>::default();
    let mut workers:[writer::Kernel;17]=std::array::from_fn(|_|writer::Kernel::default());
    let (mut coded,mut plain,mut expanded)=(Vec::new(),Vec::new(),Vec::new());
    let (mut consumed, mut checks, mut all, mut coded_trials, mut plain_trials)=(0,0,0,0usize,0usize);
    let mut streams=std::collections::BTreeSet::new();
    while let Event::Block{stream,symbols,..}=scan.next(Vec::new()).unwrap() {
        streams.insert(stream);
        plain.clear();
        mibz2::decode_block(&symbols,k,&mut stock,&mut plain).unwrap();
        let independent=&oracle[consumed..consumed+plain.len()];
        assert_eq!(plain,independent);
        consumed+=plain.len();
        all+=1;
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;
        if r.records().len()*20>=3*n && (n<65536 || !fits(r.records(),p)){continue;}
        for (t,w) in workers.iter_mut().enumerate() {
            w.run::<true,true,false>(r.records(),n,p,t as u32,&mut coded);
            assert_eq!(coded.len(),n);
            expanded.clear();
            mibz2::rle::unrle(&coded,&mut expanded,k).unwrap();
            assert_eq!(mibz2::crc::crc32(&expanded,k),symbols.crc);
            assert_eq!(expanded,independent);
            checks+=1;coded_trials+=n;plain_trials+=expanded.len();
        }
    }
    assert_eq!((all,streams.len(),consumed,checks),(431,12,oracle.len(),3961));
    println!("{all} real blocks, {} streams with validated combined CRCs, {consumed} independent libbzip2 bytes",streams.len());
    println!("{checks} complete width-specific inverses, {coded_trials} coded bytes and {plain_trials} RLE1 output bytes checked with block CRCs and libbzip2 across all 17 widths");
}
