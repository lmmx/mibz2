pub use mibz2::{bwt,runs,Kernels};
use mibz2::{block::Symbols,bwt::InverseBwt,stream::{Scanner,SliceSource,Event}};
include!("generated/gate.rs");
thread_local! {static SCAN:std::cell::Cell<u64>=const {std::cell::Cell::new(0)};}
#[repr(C)] struct Ts{sec:i64,ns:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Ts)->i32;}
fn cpu()->u64{
    let mut t=Ts{sec:0,ns:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.ns as u64
}
trait Decode {fn call(&mut self,b:&Symbols,k:Kernels)->&[u8];}
#[derive(Default)] struct Worker<I:InverseBwt> {scratch:mibz2::Scratch<I>,out:Vec<u8>}
impl<I:InverseBwt> Decode for Worker<I>{
    fn call(&mut self,b:&Symbols,k:Kernels)->&[u8]{
        self.out.clear();
        mibz2::decode_block(b,k,&mut self.scratch,&mut self.out).unwrap();
        &self.out
    }
}
struct Block {idx:u64,stream:u64,b:Symbols,expected:std::ops::Range<usize>,dense:bool}
fn adversarial(){
    // The independent stable-sort construction works for any last column and mark,
    // including columns for which the marked cycle is shorter than the output.
    let (mut a,mut b,mut c,mut d,mut e,mut f)=(edge0::kernel::Kernel::default(),edge1::kernel::Kernel::default(),edge4::kernel::Kernel::default(),both0::kernel::Kernel::default(),both1::kernel::Kernel::default(),both4::kernel::Kernel::default());
    let mut out=Vec::new();let mut tests=0;
    let mut verify=|col:&[u8],mark:usize,limit:u32|{
        let mut ids:Vec<_>=(0..col.len()).collect();ids.sort_by_key(|&x|col[x]);
        let mut x=mark;let want:Vec<_>=(0..col.len()).map(|_|{x=ids[x];col[x]}).collect();
        let mut recs=Vec::<u16>::new();
        for &v in col{
            if let Some(last)=recs.last_mut(){if *last>>8==v as u16 && (*last&255)<127 {*last+=1;continue;}}
            recs.push((v as u16)<<8|1);
        }
        macro_rules! check {($worker:expr)=>{{
            $worker.run::<true,true,false>(&recs,col.len(),mark as u32,limit,&mut out);
            assert_eq!(out,want);
        }};}
        check!(a);check!(b);check!(c);check!(d);check!(e);check!(f);tests+=1;
    };
    for n in 1..=9{for mask in 0..1<<n{
        let v:Vec<_>=(0..n).map(|i|((mask>>i)&1)as u8).collect();
        for p in 0..n{verify(&v,p,1);verify(&v,p,16);}
    }}
    for z in 0..96u32{
        let mut seed=z+1;let mut v=Vec::new();
        for j in 0..120{seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let width=[1,2,3,4,5,15,16,17,126,127,128,255][j%12];
            v.extend(std::iter::repeat_n(((seed>>24)%13) as u8,width));}
        for p in [0,1,4,16,127,v.len()/2,v.len()-1]{verify(&v,p,16);}
    }
    for h in [21844,21845,100,21846]{let mut v=vec![2];for i in 0..h{v.extend(std::iter::repeat_n((i&1)as u8,17));}verify(&v,0,16);}
    println!("{tests} arbitrary stable marked-LF comparisons per kernel across all six implementations, including capped boundary, short width, periodic and wide endpoint cases");
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="adversarial"{adversarial();return;}
    let warm_only=args[1]=="gate";
    let rounds:usize=if warm_only{0}else{args[4].parse().unwrap()};
    let raw=std::fs::read(&args[2]).unwrap();
    let oracle=std::fs::read(&args[3]).unwrap();
    let mut scan=Scanner::new(SliceSource(&raw));
    let k=Kernels::detect();
    macro_rules! worker {($t:ty)=>{Box::new(Worker::<$t>::default()) as Box<dyn Decode>};}
    let mut arms=[worker!(edge0::Candidate),worker!(edge0::Candidate),worker!(edge1::Candidate),worker!(edge4::Candidate),
                  worker!(both0::Candidate),worker!(both0::Candidate),worker!(both1::Candidate),worker!(both4::Candidate)];
    let mut first=[false;12];let mut selected=Vec::new();
    let (mut pos,mut all,mut admissions)=(0usize,0usize,0usize);
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap() {
        let expected;
        {
            let base=arms[0].call(&symbols,k);expected=pos..pos+base.len();assert_eq!(base,&oracle[expected.clone()]);
            pos=expected.end;
        }
        if warm_only{
            for arm in &mut arms[1..] {assert_eq!(arm.call(&symbols,k),&oracle[expected.clone()]);}
        }
        all+=1;
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();
        let dense=r.records().len()*20>=3*n;
        if dense && (n<65536 || !fits(r.records(),symbols.orig_ptr)){continue;}
        admissions+=1;
        if n>=65536 && (!first[stream as usize] || dense) {
            first[stream as usize]=true;
            selected.push(Block{idx:index,stream,b:symbols,expected,dense});
        }
    }
    assert_eq!((all,pos,admissions,selected.len()),(431,209700196,233,17));
    if warm_only {println!("431 blocks, 12 streams, 233 true edge admissions; all six implementations and both twin labels matched every independently generated libbzip2 byte and block/stream CRC over {pos} bytes");return;}
    println!("round,block,stream,position,arm,dense,scan_ns,total_ns");
    for round in 0..=rounds {
        let lap=round.saturating_sub(1);
        for off in 0..selected.len() {
            let i=(if round%2==0{off}else{selected.len()-off-1}+round*7)%selected.len();
            let b=&selected[i];
            for position in 0..8{
                let arm=(lap+i+if (lap/8+i)%2==0{position}else{8-position})%8;
                SCAN.with(|v|v.set(0));
                let t=cpu();let out=arms[arm].call(&b.b,k);let stop=cpu();
                assert_eq!(out,&oracle[b.expected.clone()]);
                if round>0 {println!("{round},{},{},{position},{arm},{},{},{}",b.idx,b.stream,usize::from(b.dense),SCAN.with(|v|v.get()),stop-t);}
            }
        }
    }
}
