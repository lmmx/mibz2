pub use mibz2::{bwt,runs,Kernels};
use mibz2::{block::Symbols,bwt::InverseBwt,stream::{Scanner,SliceSource,Event}};
include!("arms.inc.rs");
thread_local! { static PHASE: std::cell::Cell<u64> = const {std::cell::Cell::new(0)}; }

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
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let data=std::fs::read(&args[1]).unwrap();
    let rounds:usize=args[2].parse().unwrap();
    assert_eq!(rounds%8,0);
    let mode=&args[3];
    let k=Kernels::detect();
    let mut workers:[Box<dyn Decoder>;4]=[
        Box::new(Worker::<edge::Candidate>::default()),
        Box::new(Worker::<counter_edge::Candidate>::default()),
        Box::new(Worker::<scalar_edge::Candidate>::default()),
        Box::new(Worker::<both::Candidate>::default())
    ];
    let mut oracle=Worker::<bwt::LazyCells>::default();
    let mut scan=Scanner::new(SliceSource(&data));
    let(mut count,mut large_ordinal,mut bytes)=(0,0,0);
    let mut sample=Vec::new();
    while let Ok(Event::Block{index,symbols:s,..})=scan.next(Vec::new()){
        let truth=oracle.decode(&s,k);
        let mut phases=[0u64;4];
        for (arm,worker) in workers.iter_mut().enumerate(){
            PHASE.with(|p|p.set(0));
            assert_eq!(truth,worker.decode(&s,k));
            phases[arm]=PHASE.with(std::cell::Cell::get);
        }
        assert!(phases.iter().all(|&x|(x>0)==(phases[0]>0)));
        let large=s.syms.len()>4096;
        let extra=[41,56,165,270,278].contains(&index);
        if large && large_ordinal%24==0 || extra {
            sample.push((index,if extra{"extra"}else{"ordinary"},s,truth.to_vec()));
        }
        count+=1;
        bytes+=truth.len();
        if large {large_ordinal+=1;}
    }
    assert_eq!((count,bytes),(431,209700196));
    eprintln!("oracle=LazyCells all_blocks={} all_bytes={} large={} sample={} rounds={} mode={}",count,bytes,large_ordinal,sample.len(),rounds,mode);
    println!("mode,round,stream,class,position,arm,worker_thread_cpu_ns,scan_thread_cpu_ns");
    for round in 0..rounds{
        for (j,(index,class,s,truth)) in sample.iter().enumerate(){
            for position in 0..4{
                let arm=(round%4+j+if round/4%2==0{position}else{4-position})%4;
                PHASE.with(|p|p.set(0));
                let start=cpu();
                let out=workers[arm].decode(s,k);
                let elapsed=cpu()-start;
                let phase=PHASE.with(std::cell::Cell::get);
                assert_eq!(truth,out);
                println!("{mode},{round},{index},{class},{position},{arm},{elapsed},{phase}");
            }
        }
    }
}
