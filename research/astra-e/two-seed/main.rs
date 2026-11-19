pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Scanner,SliceSource,Event};
use std::path::Path;
pub struct Table {
    pub edges: Vec<u32>,
    pub extra: Vec<u32>,
    pub spelling: Vec<u8>,
    pub offsets: Vec<u32>,
    pub n: usize,
    pub mark: u32,
    pub bits: u32,
}
#[allow(dead_code)]
mod original {include!(env!("EDGE_TABLE_SOURCE"));}
include!("../edge-budget/gate.rs");
mod writer;
mod synthetic;
#[repr(C)] struct Timespec {sec:i64,nsec:i64}
unsafe extern "C" {fn clock_gettime(clock:i32,t:*mut Timespec)->i32;}
fn cpu()->u64 {
    let mut t=Timespec{sec:0,nsec:0};
    assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    (t.sec as u64)*1_000_000_000 + t.nsec as u64
}
const IDS: [u64;14]=[1,41,56,85,134,165,176,225,269,270,278,313,359,397];
const ARMS: [&str;8]=["original","original twin","split","split twin","single","single twin","freeze","freeze twin"];
fn run(arm:usize,t:&Table,out:&mut Vec<u8>) {
    match arm/2 {
        0=>original::Kernel::stock(t,out),
        1=>{writer::split::<false,false>(t,out);},
        2=>writer::single(t,out),
        3=>{writer::split::<false,true>(t,out);},
        _=>unreachable!(),
    }
}
fn main() {
    let args=std::env::args().collect::<Vec<_>>();
    if args[1]=="synthetic" {return synthetic::run();}
    let gate=args[1]=="gate";
    assert!(gate || args[1]=="price");
    let data=std::fs::read(Path::new(&args[2])).unwrap();
    let k=Kernels::detect();
    let mut scanner=Scanner::new(SliceSource(&data));
    let mut builder=original::Kernel::default();
    let mut base=Segmented::default();
    let mut runs=runs::Runs::default();
    let mut outs: [Vec<u8>;8]=std::array::from_fn(|_|Vec::new());
    let mut gold=Vec::new();
    let mut samples=Vec::new();
    let (mut admissions,mut bytes)=(0,0);
    if gate {println!("arm,block,n,m,mark,bits,paired,carried,moved,rejected,boundary,out_cap");}
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap() {
        if (!gate && !IDS.contains(&index)) || !runs::takes_runs(&symbols) {continue;}
        let n=runs::front(&symbols,&mut runs,k).unwrap();
        if runs.records().len()*20>=3*n && (n<65536 || !fits(runs.records(),symbols.orig_ptr)) {continue;}
        let t=builder.make(runs.records(),n,symbols.orig_ptr);
        assert!(base.invert_runs(&runs,n,symbols.orig_ptr,&mut gold,k));
        if gate {
            admissions+=1;
            bytes+=n;
            for (i,out) in outs.iter_mut().enumerate() {
                run(i,&t,out);
                assert_eq!(out,&gold);

            }
            assert!(outs.iter().all(|o|o.capacity()==outs[0].capacity()));
            let measures=[writer::split::<true,false>(&t,&mut outs[2]),writer::split::<true,true>(&t,&mut outs[2])];
            for (i,w) in measures.iter().enumerate() {
            assert_eq!(outs[2],gold);
            println!("{i},{index},{n},{},{},{},{},{},{},{},{},{}",t.edges.len(),t.mark,t.bits,w.paired,w.carried,w.moved,w.rejected,w.boundary,outs[0].capacity());
            }
        } else {samples.push((index,t,std::mem::take(&mut gold)));}
    }
    if gate {
        assert_eq!((admissions,bytes),(233,209697662));
        eprintln!("{admissions} real admissions, {bytes} marked bytes from independent Segmented, equal output and high-water capacities");
        return;
    }
    assert_eq!(samples.len(),14);
    println!("round,block,position,arm,cpu_ns");
    for lap in 0..=40 {
        for off in 0..samples.len() {
            let j=(lap*5+if lap%2==0{off}else{samples.len()-off-1})%samples.len();
            let (id,t,want)=&samples[j];
            for pos in 0..8 {
                let shift=(lap.saturating_sub(1)+j)%8;
                let ar=(shift+if (lap.saturating_sub(1)/8+j)%2==0{pos}else{8-pos})%8;
                let start=cpu();
                run(ar,t,&mut outs[ar]);
                let elapsed=cpu()-start;
                assert_eq!(&outs[ar],want);
                if lap>0 {println!("{lap},{id},{pos},{},{elapsed}",ARMS[ar]);}
            }
        }
    }
}
