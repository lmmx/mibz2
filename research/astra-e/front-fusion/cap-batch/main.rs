pub use mibz2::{bwt,runs,Kernels};
use mibz2::{block::Symbols,bwt::InverseBwt,stream::{Scanner,SliceSource,Event}};
mod control;
mod scalar;
mod grouped;
include!("gate.rs");

mod arm0 { use crate::{control as kernel, fits}; include!("adapter.inc.rs"); }
mod arm2 { use crate::{scalar as kernel, fits}; include!("adapter.inc.rs"); }
mod arm3 { use crate::{grouped as kernel, fits}; include!("adapter.inc.rs"); }

#[repr(C)]
struct Time { sec:i64, nsec:i64 }
unsafe extern "C" { fn clock_gettime(id:i32,value:*mut Time)->i32; }
fn cpu()->u64 {
    let mut t=Time{sec:0,nsec:0};
    assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.nsec as u64
}

trait Decoder { fn decode(&mut self, b:&Symbols,k:Kernels) -> &[u8]; }
#[derive(Default)]
struct Worker<I:InverseBwt> { scratch:mibz2::Scratch<I>, out:Vec<u8> }
impl<I:InverseBwt> Decoder for Worker<I> {
    fn decode(&mut self,b:&Symbols,k:Kernels)->&[u8] {
        self.out.clear();
        mibz2::decode_block(b,k,&mut self.scratch,&mut self.out).unwrap();
        &self.out
    }
}

fn boundaries() {
    let mut count=0;
    for prefix in 0..32 {
        for width in [1u32,15,16,17,126,127,128,143,253,254,255,5000] {
            for tail in [0usize,1,15,16,17,31] {
                let mut r=Vec::new();
                for i in 0..prefix {r.push(((i%2)<<8) as u16 | 1);}
                let mut left=width;
                while left>0 {let v=left.min(127);r.push(42<<8|v as u16);left-=v;}
                for i in 0..tail {r.push((((3+i%2)<<8) as u16) | 3);}
                let n=prefix + width + 3*tail as u32;
                for p in [0,prefix,(prefix+width/2).min(n-1),prefix+width-1,n-1] {
                    let a=control::observe(&r,p);
                    assert_eq!(a,scalar::observe(&r,p));
                    assert_eq!(a,grouped::observe(&r,p));
                    count+=1;
                }
            }
        }
    }
    println!("{count} marked geometry comparisons across 32 alignments, 6 tails, 12 widths and 5 marked rows");
}

fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="boundaries" {boundaries();return;}
    let data=std::fs::read(&args[1]).unwrap();
    let rounds:usize=args[2].parse().unwrap();
    let k=Kernels::detect();
    let mut workers:[Box<dyn Decoder>;4]=[
        Box::new(Worker::<arm0::Candidate>::default()),Box::new(Worker::<arm0::Candidate>::default()),
        Box::new(Worker::<arm2::Candidate>::default()),Box::new(Worker::<arm3::Candidate>::default())
    ];
    let mut scan=Scanner::new(SliceSource(&data));
    let (mut ordinal,mut count,mut bytes)=(0,0,0);
    println!("stream,round,position,arm,class,cpu_ns");
    while let Ok(Event::Block{index,symbols:s,..})=scan.next(Vec::new()){
        let large=s.syms.len()>4096;
        let edge=[41,56,165,270,278].contains(&index);
        let selected=!large || ordinal%24==0 || edge;
        let class=if edge{"extra"}else if large{"ordinary"}else if runs::takes_runs(&s){"short"}else{"column"};
        if large {ordinal+=1;}
        let mut expected=Vec::new();
        for round in 0..=if selected{rounds}else{0} {
            for step in 0..4 {
                let arm=if round==0{step}else{let r=round-1;(r%4 + if r/4%2==0{step}else{4-step})%4};
                let start=cpu();let output=workers[arm].decode(&s,k);let elapsed=cpu()-start;
                if round==0 && arm==0 {expected.extend_from_slice(output);bytes+=output.len();}
                else {assert_eq!(output,expected);}
                println!("{index},{round},{step},{arm},{class},{elapsed}");
            }
        }
        count+=1;
    }
    assert_eq!((count,bytes),(431,209700196));
    eprintln!("{count} stream blocks, {bytes} decoded bytes, {ordinal} large blocks; every returned block CRC and output byte matched");
}
