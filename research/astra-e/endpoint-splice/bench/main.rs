#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use bwt::InverseBwt;
use mibz2::{block::Symbols,stream::{Scanner,SliceSource,Event}};
use std::{cell::RefCell,collections::BTreeMap,fs::File,io::{BufWriter,Write},path::Path};
mod control;
#[path="control.rs"] mod replica;
mod batched;
#[cfg(census)] mod count;
include!("scalar-cap/gate.rs");
include!("finite.inc.rs");

#[derive(Default)]
struct Group {events:u64,total_width:u64,min:u32,max:u32}
thread_local! {static GROUPS:RefCell<BTreeMap<(usize,usize,bool),Group>>=const{RefCell::new(BTreeMap::new())};}
fn observe_batch(side:usize,count:usize,full:bool,width:u32) {
    assert!(side<4 && count>0 && width>0);
    GROUPS.with(|m|{let mut m=m.borrow_mut();let g=m.entry((side,count,full)).or_default();
        if g.events==0 {g.min=width;}
        g.events+=1;g.total_width+=u64::from(width);g.min=g.min.min(width);g.max=g.max.max(width);});
}
#[repr(C)] struct Time{sec:i64,nsec:i64}
unsafe extern "C"{fn clock_gettime(which:i32,t:*mut Time)->i32;}
fn cpu()->u64{let mut t=Time{sec:0,nsec:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);
    t.sec as u64*1_000_000_000+t.nsec as u64}

type Original=control::kernel::Kernel;
type Replica=replica::kernel::Kernel;
type Batched=batched::kernel::Kernel;
struct Arms {original:Original,replica:Replica,batched:Batched,out:[Vec<u8>;3]}
impl Arms {
    fn new()->Self {Self{original:Default::default(),replica:Default::default(),batched:Default::default(),out:Default::default()}}
    fn run(&mut self,a:usize,r:&[u16],n:usize,p:u32)->&[u8]{
        let out=&mut self.out[a];
        match a {
            0=>{self.original.run::<true,true,false>(r,n,p,16,out);}
            1=>{self.replica.run::<true,true,false>(r,n,p,16,out);}
            2=>{self.batched.run::<true,true,false>(r,n,p,16,out);}
            _=>unreachable!(),
        }out
    }
}
fn checked_worker<A:InverseBwt>(s:&Symbols,k:Kernels,scratch:&mut mibz2::Scratch<A>,out:&mut Vec<u8>){
    out.clear();mibz2::decode_block(s,k,scratch,out).unwrap();
}
struct Block{index:u64,stream:u64,records:Vec<u16>,n:usize,mark:u32,dense:bool,gold:Vec<u8>}
fn read_inputs(corpus:&Path,plain:&Path,compare_workers:bool)->Vec<Block>{
    let raw=std::fs::read(corpus).unwrap();let oracle=std::fs::read(plain).unwrap();
    let mut scan=Scanner::new(SliceSource(&raw));let mut k0=mibz2::Scratch::<control::Candidate>::default();
    let mut k1=mibz2::Scratch::<batched::Candidate>::default();
    let mut seg=bwt::Segmented::default();let k=Kernels::detect();
    let (mut pos,mut count)=(0usize,0usize);
    let mut out=Vec::new();let mut blocks=Vec::new();let mut caps=runs::Runs::default();
    while let Event::Block{index,stream,symbols:s,..}=scan.next(Vec::new()).unwrap(){
        checked_worker(&s,k,&mut k0,&mut out);
        let range=pos..pos+out.len();assert_eq!(out,&oracle[range.clone()]);
        if compare_workers{checked_worker(&s,k,&mut k1,&mut out);assert_eq!(out,&oracle[range.clone()]);}
        count+=1;pos=range.end;
        if !runs::takes_runs(&s){continue;}
        let n=runs::front(&s,&mut caps,k).unwrap();let r=caps.records();let dense=r.len()*20>=3*n;
        if dense&&(n<65_536||!fits(r,s.orig_ptr)){continue;}
        let mut gold=Vec::new();assert!(seg.invert_runs(&caps,n,s.orig_ptr,&mut gold,k));
        out.clear();mibz2::rle::unrle(&gold,&mut out,k).unwrap();
        assert_eq!(out,&oracle[range]);assert_eq!(mibz2::crc::crc32(&out,k),s.crc);
        blocks.push(Block{index,stream,records:r.to_vec(),n,mark:s.orig_ptr,dense,gold});
    }
    assert_eq!((count,pos,blocks.len(),blocks.iter().filter(|b|b.dense).count()),(431,209700196,233,5));
    blocks
}

#[cfg(census)]
fn census(blocks:&[Block],dest:&Path){
    let mut g=BufWriter::new(File::create(dest.join("groups.csv")).unwrap());
    writeln!(g,"block,side,length,full,events,width_sum,width_min,width_max").unwrap();
    let mut out=BufWriter::new(File::create(dest.join("blocks.csv")).unwrap());
    writeln!(out,"block,stream,rows,mark,cap_records,dense,retained_mark,link_bits,open_nodes,steps,rules,n,runs,retained,thick,bands,words,copies,traced,thick_area,unit_work,longest,table_bytes,strict_steps,long_strict_steps,strict_groups,full_groups,equal_steps,self_steps,old_links,new_links").unwrap();
    let mut arms=Arms::new();let mut metered=count::kernel::Kernel::default();
    let mut measured=Vec::new();
    for b in blocks{
        for a in 0..3 {assert!(arms.run(a,&b.records,b.n,b.mark)==b.gold);}
        metered.run::<true,true,false>(&b.records,b.n,b.mark,16,&mut measured);
        assert!(measured==b.gold);
        let original=arms.original.bench_state(&b.records,b.n,b.mark,16);
        let changed=arms.batched.bench_state(&b.records,b.n,b.mark,16);
        assert!(original==changed,"state of {}",b.index);
        let groups=GROUPS.with(|g|std::mem::take(&mut *g.borrow_mut()));
        let (mut strict,mut long,mut strict_groups,mut full_groups,mut equal,mut same)=(0u64,0,0,0,0,0);
        for ((side,k,full),data) in groups {
            let v=data.events;
            if side<2{
                strict+=v*k as u64;strict_groups+=v;
                if k>=2{long+=v*k as u64;}
                if full{full_groups+=v;}
            }else if side==2{assert_eq!((k,full),(1,false));equal+=v;}
            else{assert_eq!((k,full),(1,false));same+=v;}
            writeln!(g,"{},{side},{k},{},{v},{},{},{}",b.index,u8::from(full),data.total_width,data.min,data.max).unwrap();
        }
        assert_eq!(strict+equal+same,original.0[1]as u64,"group steps of {}",b.index);
        let old=6*strict+10*equal+4*same;
        let new=6*(strict_groups-full_groups)+10*equal+4*same;
        assert!(new<=old);
        write!(out,"{},{},{},{},{},{},{},{}",b.index,b.stream,b.n,b.mark,b.records.len(),u8::from(b.dense),original.1,original.2).unwrap();
        for x in original.0{write!(out,",{x}").unwrap();}
        writeln!(out,",{strict},{long},{strict_groups},{full_groups},{equal},{same},{old},{new}").unwrap();
    }
}

fn price(blocks:&[Block],dest:&Path) {
    let mut arms=Arms::new();let mut output=BufWriter::new(File::create(dest.join("times.csv")).unwrap());
    writeln!(output,"round,block,stream,slot,arm,dense,rows,inverse_ns").unwrap();
    let labels=["control","replica","batched"];let count=blocks.len();
    for round in 0..=24{
        let lap=usize::saturating_sub(round,1);
        for offset in 0..count{
            let i=(17*lap+if lap%2==0{offset}else{count-1-offset})%count;
            let b=&blocks[i];
            for slot in 0..3{
                let a=(i+lap+if (lap/3+i)%2==0{slot}else{2-slot})%3;
                let t=cpu();let got=arms.run(a,&b.records,b.n,b.mark);let dt=cpu()-t;
                assert!(got==b.gold);
                if round>0{writeln!(output,"{round},{},{},{slot},{},{},{},{dt}",b.index,b.stream,labels[a],u8::from(b.dense),b.n).unwrap();}
            }
        }
    }
}
fn hwm()->usize{
    std::fs::read_to_string("/proc/self/status").unwrap().lines().find(|s|s.starts_with("VmHWM:"))
        .unwrap().split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn one<A:Default>(caps:&[u16],n:usize,mark:u32,gold:&Path,run:impl Fn(&mut A,&[u16],usize,u32,&mut Vec<u8>)){
    let mut kernel=A::default();let mut output=Vec::new();let pre=hwm();
    run(&mut kernel,caps,n,mark,&mut output);let post=hwm();
    assert!(output==std::fs::read(gold).unwrap());println!("{pre},{post}");
}
fn main(){
    let z:Vec<_>=std::env::args().collect();
    if z[1]=="rss"{
        let bytes=std::fs::read(&z[2]).unwrap();
        let caps:Vec<_>=bytes.chunks_exact(2).map(|p|u16::from_le_bytes([p[0],p[1]])).collect();drop(bytes);
        let n=z[3].parse().unwrap();let p=z[4].parse().unwrap();let gold=Path::new(&z[5]);
        macro_rules! mem {($t:ty)=>{one::<$t>(&caps,n,p,gold,|k,r,n,p,o|{k.run::<true,true,false>(r,n,p,16,o);})};}
        match z[6].as_str(){"control"=>mem!(Original),"replica"=>mem!(Replica),"batched"=>mem!(Batched),_=>unreachable!()};return;
    }
    #[cfg(census)] if z[1]=="finite"{finite();return;}
    let blocks=read_inputs(Path::new(&z[2]),Path::new(&z[3]),z[1]=="census");
    let dest=Path::new(&z[4]);
    #[cfg(census)] if z[1]=="census"{census(&blocks,dest);return;}
    assert_eq!(z[1],"price");price(&blocks,dest);
}
