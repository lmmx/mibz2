#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Scanner,SliceSource,Event};
include!("gate.rs");
mod scalar;mod four;
#[repr(C)] struct Timespec {sec:i64,nsec:i64}
unsafe extern "C" {fn clock_gettime(clock:i32,t:*mut Timespec)->i32;}
fn cpu()->u64 {let mut t=Timespec{sec:0,nsec:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.nsec as u64}
const IDS:[u64;14]=[1,41,56,85,134,165,176,225,269,270,278,313,359,397];
const LABELS:[&str;8]=["retained scalar","retained scalar twin","retained four","retained four twin",
                        "fresh scalar","fresh scalar twin","fresh four","fresh four twin"];
struct Block {id:u64,n:usize,p:u32,records:Vec<u16>,expected:Vec<u8>,a:scalar::Kernel,b:four::Kernel,mark:u32,bits:u32}
fn main(){
    let input=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner=Scanner::new(SliceSource(&input));
    let k=Kernels::detect();let mut reference=Segmented::default();let mut samples=Vec::new();
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap(){
        if !IDS.contains(&index){continue;}
        assert!(runs::takes_runs(&symbols));
        let mut rr=runs::Runs::default();let n=runs::front(&symbols,&mut rr,k).unwrap();let p=symbols.orig_ptr;
        assert!(rr.records().len()*20<3*n || (n>=65536 && fits(rr.records(),p)));
        let mut expected=Vec::new();assert!(reference.invert_runs(&rr,n,p,&mut expected,k));
        let mut a=scalar::Kernel::default();let mut b=four::Kernel::default();
        let (mark,bits)=a.build_phase(rr.records(),n,p);assert_eq!((mark,bits),b.build_phase(rr.records(),n,p));
        samples.push(Block{id:index,n,p,records:rr.records().to_vec(),expected,a,b,mark,bits});
    }
    assert_eq!(samples.len(),14);
    let mut af:[scalar::Kernel;2]=Default::default();let mut bf:[four::Kernel;2]=Default::default();
    let mut out:[Vec<u8>;8]=Default::default();
    println!("round,block,position,arm,construct_ns,emit_ns,total_ns");
    for lap in 0usize..=40{
        for off in 0..samples.len(){
            let j=(lap*5+if lap%2==0{off}else{samples.len()-1-off})%samples.len();let block=&samples[j];
            for pos in 0..8 {
                let phase=(lap.saturating_sub(1)+j)%8;
                let arm=(phase+if (lap.saturating_sub(1)/8+j)%2==0{pos}else{8-pos})%8;
                let start=cpu();
                let (m,b)=match arm/2 {
                    0|1=>(block.mark,block.bits),
                    2=>af[arm%2].build_phase(&block.records,block.n,block.p),
                    3=>bf[arm%2].build_phase(&block.records,block.n,block.p),
                    _=>unreachable!(),
                };
                let built=cpu();
                match arm/2 {
                    0=>block.a.emit_phase(block.n,m,b,&mut out[arm]),
                    1=>block.b.emit_phase(block.n,m,b,&mut out[arm]),
                    2=>af[arm%2].emit_phase(block.n,m,b,&mut out[arm]),
                    3=>bf[arm%2].emit_phase(block.n,m,b,&mut out[arm]),
                    _=>unreachable!(),
                }
                let end=cpu();
                assert_eq!((m,b),(block.mark,block.bits));assert_eq!(out[arm],block.expected);
                if lap>0 {println!("{lap},{},{pos},{},{},{},{}",block.id,LABELS[arm],built-start,end-built,end-start);}
            }
        }
    }
}
