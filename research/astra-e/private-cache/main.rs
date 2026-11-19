#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
#[path = "source/research/astra-e/narrow-short/constructor_ahead.rs"]
mod writer;
include!("gate.rs");

include!("geometry.rs");

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let raw = std::fs::read(&args[1]).unwrap();
    let mut scan = Scanner::new(SliceSource(&raw));
    let kernels=Kernels::detect();
    let (mut oracle, mut writer, mut out, mut want) = (Segmented::default(), writer::Kernel::default(), Vec::new(), Vec::new());
    println!("block,stream,n,records,primary,limit,m,h,code_bound,narrow_bits,edge_bound,edge,spell,offset,marked_visits,marked_spell,marked_offset,mutable_vec_capacity,initial_nodes,steps,rules,bands,words,recurrent_spell_aligned,recurrent_offset_aligned,recurrent_spell_shifted,recurrent_offset_shifted");
    while let Event::Block {index,stream,symbols,..} = scan.next(Vec::new()).unwrap() {
        if !runs::takes_runs(&symbols) {continue;}
        let mut r = runs::Runs::default();
        let n = runs::front(&symbols,&mut r,kernels).unwrap();
        let p = symbols.orig_ptr;
        if r.records().len()*20>=3*n && (n<65536 || !fits(r.records(),p)) {continue;}
        assert!(oracle.invert_runs(&r,n,p,&mut want,kernels));
        let (m,h)=run_geometry(r.records(),p as usize);
        for t in 0..=16 {
            let bits=(usize::BITS-(m[t]-1).leading_zeros()).max(1);
            let code_bound=511+3*h[t];
            let bound=m[t]*if code_bound<=((u32::MAX>>bits) as usize){4}else{8};
            let (c,s)=writer.inspect(r.records(),n,p,t as u32,&mut out);
            assert_eq!(&out,&want, "block {index} threshold {t}");
            assert_eq!((c.retained,c.thick),(m[t],h[t]));
            assert!(s[0]<=bound);
            println!("{index},{stream},{n},{},{p},{t},{},{},{code_bound},{bits},{bound},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                     r.records().len(),m[t],h[t],s[0],s[1],s[2],s[3],s[4],s[5],s[6],c.open_nodes,c.steps,c.rules,c.bands,c.words,s[7],s[8],s[9],s[10]);
        }
    }
}
