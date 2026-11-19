pub use mibz2::{bwt,runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
#[path="base.rs"] mod base;
fn main() {
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));
    let (mut kernel,mut r)=(base::Candidate::default(),runs::Runs::default());
    let mut out=std::io::BufWriter::new(std::io::stdout().lock());
    use std::io::Write;
    writeln!(out,"stream,node,parent,ranks,selectors,dict_bytes").unwrap();
    let mut ordinal=0;
    while let Ok(Event::Block{index,symbols,..})=scan.next(Vec::new()) {
        let n=runs::front(&symbols,&mut r,Kernels::detect()).unwrap();
        if n<65536 {continue}
        let selected=ordinal%24==0;
        ordinal+=1;
        if selected && r.records().len()*20<3*n && r.records().len()*64>=n {
            kernel.export(r.records(),n as u32,symbols.orig_ptr,&mut out,index as usize);
        }
    }
}
