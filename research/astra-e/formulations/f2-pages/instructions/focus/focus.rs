use bwt::{InverseBwt,Segmented};
use mibz2::stream::{Event,Scanner,SliceSource};
pub use mibz2::{bwt,runs,Kernels};
use std::time::Instant;
#[allow(dead_code)]
mod counted;
#[allow(dead_code)]
mod dispatch;

#[repr(C)]
struct Timespec { s:i64,ns:i64 }
unsafe extern "C" { fn clock_gettime(id:i32, ts:*mut Timespec) -> i32; }
#[inline(never)]
fn thread_cpu_ns()->u64 {
    let mut ts=Timespec{s:0,ns:0};
    // SAFETY: 64-bit Linux accepts the aligned, writable stack timespec.
    assert_eq!(unsafe {clock_gettime(3,&mut ts)},0);
    ts.s as u64*1_000_000_000 + ts.ns as u64
}

struct Block { index:u64,n:usize,mark:u32,r:runs::Runs,want:Vec<u8> }

fn main() {
    assert!(std::is_x86_feature_detected!("popcnt"));
    let compressed=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut scanner=Scanner::new(SliceSource(&compressed));
    let (kn,mut reference)=(Kernels::detect(),Segmented::default());
    let mut blocks=Vec::new();
    let mut ordinal=0;
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap() {
        let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,kn).unwrap();
        if n<65536 {continue}
        let select=ordinal%24==0; ordinal+=1;
        if !select || r.records().len()*20>=3*n || r.records().len()*64<n {continue}
        let mut want=Vec::new();
        assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,kn));
        blocks.push(Block{index,n,mark:symbols.orig_ptr,r,want});
    }
    assert_eq!(blocks.len(),9);
    let mut frozen=counted::Candidate::default();
    let mut generic=dispatch::Candidate::default();
    let mut hardware=dispatch::Candidate::default();
    let mut outputs: [Vec<u8>;3]=Default::default();
    let perms=[[0,1,2],[1,2,0],[0,2,1],[2,0,1],[1,0,2],[2,1,0]];
    println!("block,round,position,arm,prep_mark_ns,discover_ns,merge_ns,residual_output_ns,inverse_cpu_ns,wall_ns");
    for pass in 0..=60 {
        for t in 0..9 {
            let bi=(pass*5 + if pass%2==0 {t} else {8-t})%9;
            let b=&blocks[bi];
            for (pos,&arm) in perms[(pass+bi)%6].iter().enumerate() {
                let out=&mut outputs[arm];
                let start=Instant::now();
                let r=match arm {
                    0=>frozen.measure(&b.r,b.n,b.mark,out,true),
                    1=>generic.matched(&b.r,b.n,b.mark,out,false),
                    _=>hardware.matched(&b.r,b.n,b.mark,out,true),
                };
                let wall=start.elapsed().as_nanos();
                assert_eq!(out,&b.want);
                if pass!=0 {
                    println!("{},{pass},{pos},{},{},{},{},{},{},{wall}",b.index,
                        ["counted","generic","hardware"][arm],r[0],r[1],r[2],r[3],r.iter().sum::<u64>());
                }
            }
        }
    }
}
