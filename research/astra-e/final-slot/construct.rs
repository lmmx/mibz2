//! Count the shared inverse structure for every eligible block, outside timing.
#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
#[path = "../narrow-short/constructor_ahead.rs"]
mod kernel;
use mibz2::stream::{Event,Scanner,SliceSource};
use mibz2::bwt::InverseBwt;

fn main() {
    let data = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut records = runs::Runs::default();
    let mut inverse = kernel::Kernel::default();
    let mut lazy = bwt::LazyCells::default();
    let (mut out,mut reference,mut symbols_buf) = (Vec::new(),Vec::new(),Vec::new());
    let mut ordinal = 0;
    let k = Kernels::detect();
    println!("ordinal,n,records,primary,retained,nodes,steps,rules,bands,words,dictionary_spelling_bytes,edge_bytes");
    while let Event::Block {symbols,..} = sc.next(symbols_buf).unwrap() {
        if runs::takes_runs(&symbols) {
            let n = runs::front(&symbols,&mut records,k).unwrap();
            if records.records().len()*20<n*3 {
                let c = inverse.run::<true,true,false>(records.records(),n,symbols.orig_ptr,16,&mut out);
                assert!(lazy.invert_runs(&records,n,symbols.orig_ptr,&mut reference,k));
                assert_eq!(&out,&reference);
                println!("{ordinal},{n},{},{},{},{},{},{},{},{},{},{}",
                    records.records().len(),symbols.orig_ptr,c.retained,c.open_nodes,c.steps,
                    c.rules,c.bands,c.words,c.traced+c.words,c.table_bytes);
            }
        }
        ordinal += 1;
        symbols_buf = symbols.syms;
    }
}
