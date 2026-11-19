pub use mibz2::{bwt,runs,Kernels,ErrorKind};
use mibz2::{block::Symbols,stream::{Scanner,SliceSource,Event}};
mod log;mod front;mod packed;mod control;
include!("gate.rs");
mod stock{use crate::{control as kernel,fits};include!("adapter.inc.rs");}
#[derive(Default)]
struct Bits {lit:Vec<u64>,one:Vec<u64>}
#[derive(Default)]
struct Arm {old:mibz2::Scratch<stock::Candidate>,kernel:packed::Kernel,bits:Bits,sep:Vec<u16>,raw:Vec<u8>,out:Vec<u8>}
fn packed_fits(log:&[u16],primary:u32)->bool{
    let(mut i,mut pos,mut m,mut h)=(0,0,0u32,0u32);
    while i<log.len(){
        let(_,w)=log::take(log,&mut i);
        if w<=16{m+=w;}else if (pos..pos+w).contains(&primary){
            m+=1;h+=u32::from(primary>pos)+u32::from(primary+1<pos+w);
        }else{h+=1;}pos+=w;
    }
    let bits=(32-(m-1).leading_zeros()).max(1);
    m*if 511+3*h<=u32::MAX>>bits{4}else{8}<=1<<20
}
fn avx(k:Kernels)->bool{k!=Kernels::scalar()&&is_x86_feature_detected!("avx2")&&is_x86_feature_detected!("bmi1")&&is_x86_feature_detected!("bmi2")&&is_x86_feature_detected!("lzcnt")&&is_x86_feature_detected!("popcnt")}
impl Arm{
    fn decode(&mut self,b:&mut Symbols,k:Kernels,kind:usize)->(Result<u32,ErrorKind>,usize,bool){
        self.out.clear();
        if kind<2||!runs::takes_runs(b)||b.syms.len()<256||b.max_len>=1<<24||!avx(k){
            return (mibz2::decode_block(b,k,&mut self.old,&mut self.out),0,false);
        }
        let src=b.syms.as_mut_ptr();let m=b.syms.len();
        let dst=if kind==2{src}else{self.sep.clear();self.sep.reserve(m);self.sep.as_mut_ptr()};
        let mut words=0;
        let result=unsafe{front::pack(src,m,dst,b.eob,&b.seq_to_unseq,b.max_len,&mut self.bits,&mut words)};
        if let Some((n,caps))=result{
            let records=if kind==2{&b.syms[..words]}else{unsafe{self.sep.set_len(words);}&self.sep[..]};
            if n>0&&(b.orig_ptr as usize)<n&&(20*caps<3*n||(n>=65536&&packed_fits(records,b.orig_ptr))){
                self.kernel.run::<true,true,false>(records,n,b.orig_ptr,16,&mut self.raw);
                let finish=mibz2::rle::unrle(&self.raw,&mut self.out,k).and_then(|_|{
                    let computed=mibz2::crc::crc32(&self.out,k);
                    if computed==b.crc{Ok(computed)}else{Err(ErrorKind::BlockCrc{stored:b.crc,computed})}
                });
                if finish.is_err(){self.out.clear();}
                return (finish,words,true);
            }
        }
        if kind==2 && words!=0{log::restore(&mut b.syms,words,&b.seq_to_unseq[..usize::from(b.eob-1)]);}
        (mibz2::decode_block(b,k,&mut self.old,&mut self.out),words,false)
    }
}
#[repr(C)]struct Time{sec:i64,nsec:i64}
unsafe extern "C"{fn clock_gettime(id:i32,out:*mut Time)->i32;}
fn cpu()->u64{let mut t=Time{sec:0,nsec:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.nsec as u64}
include!("errors.rs");
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[2]=="errors"{disputes(&args[1]);return;}
    let data=std::fs::read(&args[1]).unwrap();let rounds:usize=args[2].parse().unwrap();
    let k=Kernels::detect();let mut arms:[Arm;4]=std::array::from_fn(|_|Arm::default());let mut scan=Scanner::new(SliceSource(&data));
    let(mut ordinal,mut count,mut bytes,mut direct)=(0,0,0,0);
    println!("stream,round,position,arm,class,cpu_ns,words,direct");
    while let Ok(Event::Block{index,symbols:mut b,..})=scan.next(Vec::new()){
        let gold=b.syms.clone();let mut expected=Vec::new();
        let large=b.syms.len()>4096;let extra=[41,56,165,270,278].contains(&index);
        let class=if extra{"extra"}else if large{"ordinary"}else if runs::takes_runs(&b){"short"}else{"column"};
        if large{ordinal+=1;}
        for r in 0..=rounds{
            if r>0&&!extra&&(!large||(r+ordinal)%24>=3){continue;}
            for pos in 0..4{
                let a=if r==0{pos}else{let j=r-1;(j%4+if j/4%2==0{pos}else{4-pos})%4};
                b.syms.copy_from_slice(&gold);
                let now=cpu();let(result,words,done)=arms[a].decode(&mut b,k,a);let elapsed=cpu()-now;
                assert_eq!(result.unwrap(),b.crc);
                if r==0&&a==0{expected.extend_from_slice(&arms[a].out);bytes+=expected.len();}else{assert_eq!(expected,arms[a].out);}
                if !done{assert_eq!(b.syms,gold);}
                if r==0&&a==2&&done{direct+=1;}
                println!("{index},{r},{pos},{a},{class},{elapsed},{words},{done}");
            }
        }
        count+=1;
    }
    assert_eq!((count,bytes,direct),(431,209700196,233));
    eprintln!("{count} exact real blocks, {bytes} output bytes checked, {direct} in-place completions; each fallback recovered identical symbols");
}
