pub use mibz2::{bwt, runs, Kernels};
use bwt::InverseBwt;
use mibz2::stream::{Event, Scanner, SliceSource};
use std::{ffi::{c_char,c_void,CString}, io::{BufWriter,Write}, path::{Path,PathBuf}, time::Instant};
use std::os::unix::ffi::OsStrExt;
const IDS:[usize;10]=[1,41,85,134,176,225,269,313,359,397];
#[repr(C)] struct Timespec {sec:i64,nsec:i64}
extern "C" {
    fn clock_gettime(id:i32,t:*mut Timespec)->i32;
    fn range_input(raw:*const c_char,ctx:*const c_char)->*mut c_void;
    fn range_input_free(input:*mut c_void);
    fn range_state()->*mut c_void;
    fn range_state_free(state:*mut c_void);
    fn range_construct(state:*mut c_void,input:*mut c_void,n:*mut u32)->*const u32;
}
fn cpu()->u64{
    let mut t=Timespec{sec:0,nsec:0};
    assert_eq!(unsafe {clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.nsec as u64
}
fn peak()->usize{
    let s=std::fs::read_to_string("/proc/self/status").unwrap();
    s.lines().find(|line|line.starts_with("VmHWM:")).unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
}
struct Ready(*mut c_void);
impl Ready{
    fn new(dir:&Path,id:usize)->Self{
        let cpath=|p:PathBuf|CString::new(p.as_os_str().as_bytes()).unwrap();
        let raw=cpath(dir.join("raw").join(format!("{id}.txt")));
        let ctx=cpath(dir.join("data").join(id.to_string()).join("8.ctx"));
        let h=unsafe{range_input(raw.as_ptr(),ctx.as_ptr())};assert!(!h.is_null());Self(h)
    }
}
impl Drop for Ready{fn drop(&mut self){unsafe{range_input_free(self.0)}}}
struct State(*mut c_void);
impl State{
    fn new()->Self{let h=unsafe{range_state()};assert!(!h.is_null());Self(h)}
    fn build<'a>(&'a mut self,input:&Ready)->&'a [u32]{
        let mut n=0;let p=unsafe{range_construct(self.0,input.0,&mut n)};assert!(n>0&&!p.is_null());
        unsafe{std::slice::from_raw_parts(p,3*n as usize)}
    }
}
impl Drop for State{fn drop(&mut self){unsafe{range_state_free(self.0)}}}
fn words(p:PathBuf)->Vec<u32>{
    let b=std::fs::read(p).unwrap();assert_eq!(b.len()%12,0);
    b.chunks_exact(4).map(|v|u32::from_le_bytes(v.try_into().unwrap())).collect()
}
struct Block {id:usize,n:usize,p:u32,cap:runs::Runs,ready:Ready,inverse:Vec<u8>,bands:Vec<u32>}
fn caps(symbols:&mibz2::block::Symbols,k:Kernels)->(runs::Runs,usize){
    assert!(runs::takes_runs(symbols));let mut r=runs::Runs::default();
    let n=runs::front(symbols,&mut r,k).unwrap();(r,n)
}
fn panel(corpus:&Path,models:&Path,refdir:&Path,out:&Path,reps:usize){
    let kernels=Kernels::detect();let data=std::fs::read(corpus).unwrap();
    let mut scanner=Scanner::new(SliceSource(&data));let mut blocks=Vec::new();
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap(){
        let id=index as usize;if !IDS.contains(&id){continue;}
        let (cap,n)=caps(&symbols,kernels);let p=symbols.orig_ptr;
        let file=std::fs::read(refdir.join(format!("block-{id}.caps"))).unwrap();
        assert_eq!(cap.records().len()*2,file.len());
        assert!(cap.records().iter().zip(file.chunks_exact(2)).all(|(v,b)|*v==u16::from_le_bytes(b.try_into().unwrap())));
        let inverse=std::fs::read(refdir.join(format!("block-{id}.raw"))).unwrap();
        assert_eq!(inverse.len(),n);
        blocks.push(Block{id,n,p,cap,ready:Ready::new(models,id),inverse,bands:words(out.join(format!("{id}.bands")))});
    }
    assert_eq!(blocks.iter().map(|x|x.id).collect::<Vec<_>>(),IDS);
    drop(scanner);drop(data);
    let mut rust=candidate::Candidate::default();let mut inverse=Vec::new();let mut state=State::new();
    let target=if reps==0 {"gate.csv"}else{"pairs.csv"};
    let mut f=BufWriter::new(std::fs::File::create_new(out.join(target)).unwrap());
    writeln!(f,"round,block,slot,arm,rows,mark,cap_records,band_records,thread_ns,wall_ns").unwrap();
    for round in 0..=reps{
        for h in 0..blocks.len(){
            let i=(h+round)%blocks.len();let b=&blocks[i];
            for slot in 0..2{
                let scalar=(round+i+slot)%2==0;
                let (ns,wall);
                if scalar{
                    let w=Instant::now();let c=cpu();
                    let ok=rust.invert_runs(&b.cap,b.n,b.p,&mut inverse,kernels);
                    ns=cpu()-c;wall=w.elapsed().as_nanos();
                    assert!(ok&&inverse.as_slice()==b.inverse);
                }else{
                    let w=Instant::now();let c=cpu();
                    let result=state.build(&b.ready);ns=cpu()-c;wall=w.elapsed().as_nanos();
                    assert!(result==b.bands);
                }
                if round!=0||reps==0{
                    writeln!(f,"{round},{},{slot},{},{},{},{},{},{ns},{wall}",b.id,
                        if scalar{"scalar"}else{"ranges"},b.n,b.p,b.cap.records().len(),b.bands.len()/3).unwrap();
                }
            }
        }
    }
}
fn rss(a:&[String]){
    let arm=&a[2];let id:usize=a[3].parse().unwrap();let inputs=Path::new(&a[4]);
    let refdir=Path::new(&a[5]);let out=Path::new(&a[6]);let single=&a[7];
    let (before,after);
    if arm=="ranges"{
        let input=Ready::new(inputs,id);let mut state=State::new();before=peak();
        let got=state.build(&input);after=peak();
        assert!(got==words(out.join(format!("{id}.bands"))));
    }else{
        assert_eq!(arm,"scalar");let k=Kernels::detect();let data=std::fs::read(single).unwrap();
        let mut scanner=Scanner::new(SliceSource(&data));
        let Event::Block{symbols,..}=scanner.next(Vec::new()).unwrap() else{panic!()};
        let (cap,n)=caps(&symbols,k);let p=symbols.orig_ptr;drop(symbols);drop(scanner);drop(data);
        let mut rust=candidate::Candidate::default();let mut raw=Vec::new();before=peak();
        assert!(rust.invert_runs(&cap,n,p,&mut raw,k));after=peak();
        assert!(raw==std::fs::read(refdir.join(format!("block-{id}.raw"))).unwrap());
    }
    println!("{before},{after}");
}
fn main(){
    let a:Vec<_>=std::env::args().collect();
    match a[1].as_str(){
        "panel"=>panel(Path::new(&a[2]),Path::new(&a[3]),Path::new(&a[4]),Path::new(&a[5]),a[6].parse().unwrap()),
        "rss"=>rss(&a),_=>panic!()
    }
}
