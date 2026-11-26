use mibz2::{runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
use std::io::{BufWriter,Write};
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));
    let mut r=runs::Runs::default();
    let mut out=BufWriter::new(std::io::stdout().lock());let mut checked=0;
    while let Ok(Event::Block{index,symbols,..})=scan.next(Vec::new()){
        let n=runs::front(&symbols,&mut r,Kernels::detect()).unwrap();
        if n<65536 || r.records().len()*20>=3*n || r.records().len()*64<n {continue;}
        for x in [index as u32,n as u32,symbols.orig_ptr,r.records().len() as u32]{
            out.write_all(&x.to_le_bytes()).unwrap();
        }
        for &v in r.records(){out.write_all(&v.to_le_bytes()).unwrap();}
        checked+=1;
    }
    assert_eq!(checked,228);
}
