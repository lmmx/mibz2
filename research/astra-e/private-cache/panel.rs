#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
use bwt::{InverseBwt, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use std::hint::black_box;
#[path = "source/research/astra-e/narrow-short/constructor_ahead.rs"] mod writer;
include!("gate.rs");
include!("geometry.rs");

#[repr(C)]
struct Timespec {sec:i64, ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Timespec)->i32;}
fn cpu()->u64 {
    let mut t=Timespec{sec:0,ns:0};
    assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.ns as u64
}

#[inline(never)]
fn widths(records:&[u16],p:u32)->[u32;2] {
    let (m,h)=run_geometry(records,p as usize);
    let mut result=[0;2];
    for t in 0..=16 {
        let bits=(usize::BITS-(m[t]-1).leading_zeros()).max(1);
        let bytes=m[t]*if 511+3*h[t]<=(u32::MAX>>bits) as usize {4}else{8};
        for (j,budget) in [192*1024,256*1024].into_iter().enumerate() {
            if bytes<=budget {result[j]=t as u32;}
        }
    }
    result
}

struct Block {idx:u64,stream:u64,n:usize,p:u32,r:runs::Runs,want:Vec<u8>,cross:[u32;2]}
#[derive(Default)]
struct Arm {writer:writer::Kernel,out:Vec<u8>}

fn main(){
    let args:Vec<_>=std::env::args().collect();
    let raw=std::fs::read(&args[1]).unwrap();
    let mut scan=Scanner::new(SliceSource(&raw));
    let k=Kernels::detect();
    let mut blocks=Vec::new();
    let mut seen=[false;12];
    let mut reference=Segmented::default();
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap() {
        if seen[stream as usize] || !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;
        if n<65536 || (r.records().len()*20>=3*n && !fits(r.records(),p)){continue;}
        let cross=widths(r.records(),p);
        if cross[1]==0 || cross[1]>=16 {continue;}
        let mut want=Vec::new();
        assert!(reference.invert_runs(&r,n,p,&mut want,k));
        blocks.push(Block{idx:index,stream,n,p,r,want,cross});
        seen[stream as usize]=true;
    }
    assert!(seen.iter().all(|&x|x));
    let mut workers:[Arm;8]=std::array::from_fn(|_|Arm::default());
    let rounds:usize=args[2].parse().unwrap();
    println!("round,block,stream,position,arm,selected,limit,cpu_ns");
    for round in 0..=rounds {
        for off in 0..blocks.len() {
            let i=(if round%2==0 {off}else{blocks.len()-1-off}+round*5)%blocks.len();
            let b=&blocks[i];
            for pos in 0..8 {
                let arm=(round+i+if (round+i)/8%2==0{pos}else{8-pos})%8;
                let work=&mut workers[arm];
                let t0=cpu();
                assert!(b.r.records().len()*20<3*b.n || (b.n>=65536 && fits(b.r.records(),b.p)));
                let selected=if arm<2 {16} else {
                    let planned=black_box(widths(b.r.records(),b.p));
                    if arm==7 {planned[0]}else{planned[1]}
                };
                let limit=match arm {0..=2=>16,3|7=>selected,4=>selected+1,5=>0,_=>selected-1};
                if arm<2 {work.writer.run::<true,true,false>(b.r.records(),b.n,b.p,16,&mut work.out);}
                else {work.writer.run::<true,true,false>(b.r.records(),b.n,b.p,limit,&mut work.out);}
                let elapsed=cpu()-t0;
                assert_eq!(work.out,b.want);
                if arm>=2 {assert_eq!(selected,b.cross[usize::from(arm!=7)]);}
                if round>0 {println!("{round},{},{},{pos},{arm},{selected},{limit},{elapsed}",b.idx,b.stream);}
            }
        }
    }
}
