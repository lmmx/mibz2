#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels,ErrorKind};
use mibz2::{block::Symbols,bwt::InverseBwt,stream::{Scanner,SliceSource,Event}};
mod front;
mod kernel;
include!("adversarial.rs");
#[repr(C)]struct Time {sec:i64,nsec:i64}
unsafe extern "C"{fn clock_gettime(id:i32,value:*mut Time)->i32;}
fn cpu()->u64{let mut t=Time{sec:0,nsec:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.nsec as u64}
#[derive(Default)]
struct Arm {column:Vec<u8>,checkpoints:Vec<[u32;256]>, kernel:kernel::Kernel, bits:front::Bits, records:runs::Runs, other:bwt::LazyCells, raw:Vec<u8>, decoded:Vec<u8> }
impl Arm {
    fn block<const TILE:usize>(&mut self,b:&Symbols,k:Kernels,enable:bool)->Result<(usize,usize,bool),ErrorKind> {
        if !runs::takes_runs(b){
            let mut counts=[0u32;256];
            let checkpoints=self.other.wants_checkpoints();
            let n=if checkpoints{mibz2::mtf::mtf_checkpoints(b,&mut self.column,&mut counts,&mut self.checkpoints,k)?}
                else{mibz2::mtf::mtf(b,&mut self.column,&mut counts,k)?};
            if b.orig_ptr as usize>=n{return Err(ErrorKind::BadOrigPtr);}
            if checkpoints {self.other.invert_checkpointed(&self.column,&counts,&self.checkpoints,b.orig_ptr,&mut self.raw);}
            else if self.other.invert_in_place(&mut self.column,&counts,b.orig_ptr){std::mem::swap(&mut self.column,&mut self.raw);}
            else {self.other.invert(&self.column,&counts,b.orig_ptr,&mut self.raw);}
            self.decoded.clear();mibz2::rle::unrle(&self.raw,&mut self.decoded,k)?;
            return Ok((n,0,false));
        }
        let mut fused=None;
        if enable && b.syms.len()>=256 && k!=Kernels::scalar() && is_x86_feature_detected!("avx2") && is_x86_feature_detected!("bmi1") && is_x86_feature_detected!("bmi2") && is_x86_feature_detected!("lzcnt") && is_x86_feature_detected!("popcnt") && b.max_len<1<<24 {
            fused=self.kernel.fuse::<TILE>(b,&mut self.bits,&mut self.raw);
        }
        let result=if let Some((n,caps))=fused {(n,caps as usize,true)}else{
            let n=runs::front(b,&mut self.records,k)?;
            if b.orig_ptr as usize>=n {return Err(ErrorKind::BadOrigPtr);}
            let caps=self.records.records();
            if caps.len()*20<3*n {self.kernel.run::<true,true,false>(caps,n,b.orig_ptr,16,&mut self.raw);}
            else {assert!(self.other.invert_runs(&self.records,n,b.orig_ptr,&mut self.raw,k));}
            (n,caps.len(),false)
        };
        self.decoded.clear();mibz2::rle::unrle(&self.raw,&mut self.decoded,k)?;
        Ok(result)
    }
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="adversarial" {adversarial();return;}
    let data=std::fs::read(&args[1]).unwrap();let rounds:usize=args[2].parse().unwrap();let k=Kernels::detect();
    let mut sc=Scanner::new(SliceSource(&data));let mut arms:[Arm;3]=std::array::from_fn(|_|Arm::default());let mut ordinal=0;let mut accepted=0;let mut delegated=0;
    println!("stream,round,position,arm,n,caps,fused,cpu_ns");
    while let Ok(Event::Block{index,symbols,..})=sc.next(Vec::new()){
        let large=symbols.syms.len()>4096;let selected=!large || ordinal%24==0;if large{ordinal+=1;}
        for round in 0..=if selected{rounds}else{0}{
            let mut expected=None;
            for pos in 0..3{
                let which=if round==0{pos}else{((round-1)%3+if (round-1)/3%2==0{pos}else{3-pos})%3};
                let a=&mut arms[which];let now=cpu();let result=match which{0=>a.block::<0>(&symbols,k,false),1=>a.block::<0>(&symbols,k,true),_=>a.block::<64>(&symbols,k,true)};let ns=cpu()-now;
                let (n,caps,direct)=result.unwrap();assert_eq!(mibz2::crc::crc32(&a.decoded,k),symbols.crc);
                if let Some((n0,c0))=expected{assert_eq!((n,caps),(n0,c0));}else{expected=Some((n,caps));}
                if round==0 || selected {println!("{index},{round},{pos},{which},{n},{caps},{direct},{ns}");}
                if round==0&&which==1{if direct{accepted+=1}else{delegated+=1}}
            }
            assert_eq!(arms[0].raw,arms[1].raw);assert_eq!(arms[0].raw,arms[2].raw);
            assert_eq!(arms[0].decoded,arms[1].decoded);assert_eq!(arms[0].decoded,arms[2].decoded);
        }
    }
    eprintln!("all scanner blocks exact: {accepted} fused, {delegated} stock finishes, {ordinal} large-symbol blocks");
}
