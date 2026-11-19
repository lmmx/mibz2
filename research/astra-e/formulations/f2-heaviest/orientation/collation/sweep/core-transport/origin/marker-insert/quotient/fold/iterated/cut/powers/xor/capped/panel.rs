pub use mibz2::{bwt, runs, Kernels};
use bwt::InverseBwt;
use mibz2::stream::{Scanner,SliceSource,Event};
use std::{ffi::c_void,io::Write,path::Path};

const IDS:[usize;10]=[1,41,85,134,176,225,269,313,359,397];
const ARMS:[&str;6]=["stream","streamgaps","dense","scalar","lazy","opend"];
#[repr(C)] struct Timespec {sec:i64,nsec:i64}
extern "C" {
    fn clock_gettime(id:i32,p:*mut Timespec)->i32;
    fn grammar_new()->*mut c_void;
    fn grammar_delete(p:*mut c_void);
    fn grammar_run(p:*mut c_void,caps:*const u16,count:u32,n:u32,mark:u32,arm:u32)->*const u8;
}
#[inline(never)]
fn cpu()->u64 {
    let mut t=Timespec{sec:0,nsec:0};
    assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.nsec as u64
}
struct Block {id:usize,n:usize,p:u32,r:runs::Runs,want:Vec<u8>}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let (corpus,dir,reps)=(&args[1],Path::new(&args[2]),args[3].parse::<usize>().unwrap());
    let k=Kernels::detect();let mut scalar=candidate::Candidate::default();
    let mut lazy=bwt::LazyCells::default();let mut opend=arms::OpenGated::default();
    let data=std::fs::read(corpus).unwrap();
    let mut sc=Scanner::new(SliceSource(&data));let mut blocks=Vec::new();
    while let Event::Block{index,symbols,..}=sc.next(Vec::new()).unwrap() {
        let id=index as usize;if !IDS.contains(&id){continue}
        assert!(runs::takes_runs(&symbols));
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;let mut want=Vec::new();
        assert!(lazy.invert_runs(&r,n,p,&mut want,k));assert_eq!(n,want.len());
        let path=dir.join(format!("block-{id}"));
        std::fs::write(path.with_extension("raw"),&want).unwrap();
        let caps:Vec<u8>=r.records().iter().flat_map(|c|c.to_le_bytes()).collect();
        std::fs::write(path.with_extension("caps"),caps).unwrap();
        blocks.push(Block{id,n,p,r,want});
    }
    assert_eq!(blocks.len(),IDS.len());
    let mut file=std::io::BufWriter::new(std::fs::File::create(dir.join("times.csv")).unwrap());
    writeln!(file,"round,slot,block,rows,mark,cap_records,maximal_runs,arm,thread_ns").unwrap();
    let states=[unsafe{grammar_new()},unsafe{grammar_new()},unsafe{grammar_new()}];
    let mut outputs=[Vec::new(),Vec::new(),Vec::new()];
    for round in 0..=reps {
        for j in 0..blocks.len() {
            let b=&blocks[(j+round)%blocks.len()];let caps=b.r.records();
            let maximal=1+caps.windows(2).filter(|w|w[0]>>8!=w[1]>>8).count();
            let base=(round+b.id)%ARMS.len();
            for slot in 0..ARMS.len() {
                let arm=(base+if (round/ARMS.len())%2==0 {slot} else {ARMS.len()-1-slot})%ARMS.len();
                let t=cpu();
                let got=if arm<3 {
                    let ptr=unsafe{grammar_run(states[arm],caps.as_ptr(),caps.len() as u32,b.n as u32,b.p,arm as u32)};
                    unsafe{std::slice::from_raw_parts(ptr,b.n)}
                } else {
                    let out=&mut outputs[arm-3];
                    let ok=match arm {3=>scalar.invert_runs(&b.r,b.n,b.p,out,k),4=>lazy.invert_runs(&b.r,b.n,b.p,out,k),_=>opend.invert_runs(&b.r,b.n,b.p,out,k)};
                    assert!(ok);out.as_slice()
                };
                let elapsed=cpu()-t;assert!(got==b.want);
                if round>0 {writeln!(file,"{round},{slot},{},{},{},{},{},{},{}",b.id,b.n,b.p,caps.len(),maximal,ARMS[arm],elapsed).unwrap();}
            }
        }
    }
    for s in states{unsafe{grammar_delete(s)};}
}
