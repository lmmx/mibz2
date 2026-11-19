use std::{fs,io::Write};
use mibz2::{runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
include!("../../../../edge-budget/gate.rs");
fn main(){
    let a=std::env::args().collect::<Vec<_>>();let input=fs::read(&a[1]).unwrap();
    let mut s=Scanner::new(SliceSource(&input));let mut out=std::io::BufWriter::new(fs::File::create(&a[2]).unwrap());
    let k=Kernels::detect();let mut cnt=0;
    while let Event::Block{index,symbols,..}=s.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();let p=symbols.orig_ptr;
        let rec=r.records();if rec.len()*20>=3*n && (n<65536 || !fits(rec,p)){continue;}
        for v in [index as u32,n as u32,p,rec.len() as u32]{out.write_all(&v.to_le_bytes()).unwrap();}
        for x in rec{out.write_all(&x.to_le_bytes()).unwrap();}cnt+=1;
    }
    assert_eq!(cnt,233);out.flush().unwrap();
}
