// Complete scratch vector capacities in stream order, for one actual worker's two fronts.
use mibz2::stream::{Event,Scanner,SliceSource};
use mibz2::runs;
#[path="stock.rs"] mod original;
#[path="wide.rs"] mod paired;
include!("gate.rs");
struct Runs {
    runs:Vec<u16>, counts:[u32;256], per:[u32;256], lit:Vec<u64>,one:Vec<u64>,
}
impl Default for Runs {
    fn default()->Self{Self{runs:Vec::new(),counts:[0;256],per:[0;256],lit:Vec::new(),one:Vec::new()}}
}
impl Runs {
    fn bytes(&self)->usize{2*self.runs.capacity()+8*(self.lit.capacity()+self.one.capacity())}
}
fn main(){
    let data=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scan=Scanner::new(SliceSource(&data));
    let(mut base,mut new,mut fallback)=(Runs::default(),Runs::default(),Runs::default());
    println!("block,record_route,attempt,accept,syms,baseline_vec_bytes,private_vec_bytes,fallback_vec_bytes");
    while let Event::Block{index,symbols:s,..}=scan.next(Vec::new()).unwrap(){
        let route=runs::takes_runs(&s);
        let attempt=route&&s.syms.len()>=64;
        let mut accept=false;
        if route {unsafe{original::front_avx2(&s,&mut base).unwrap();}}
        if attempt {
            let n=unsafe{paired::front_avx2(&s,&mut new).unwrap()};
            assert_eq!(new.runs,base.runs);
            accept=new.runs.len()*20<n*3 || n>=65_536 && fits(&new.runs,s.orig_ptr);
        }
        if route&&!accept{unsafe{original::front_avx2(&s,&mut fallback).unwrap();}}
        println!("{index},{},{},{},{},{},{},{}",u8::from(route),u8::from(attempt),u8::from(accept),s.syms.len(),base.bytes(),new.bytes(),fallback.bytes());
    }
}
