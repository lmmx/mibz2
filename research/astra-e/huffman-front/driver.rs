use mibz2::{bwt::{InverseBwt,Plain},ErrorKind,Kernels};
use std::hint::black_box;
mod bits;mod stock;mod fused;
#[derive(Clone,Copy)]struct Envelope{start:u64,end:u64,level:u32,stream:u64,index:u64}
#[derive(Debug,Clone,PartialEq)]struct Meta{crc:u32,mark:u32,n:usize,record:bool,end:u64}
struct Job{runs:mibz2::runs::Runs,front:fused::Output,syms:Vec<u16>,ll:Vec<u8>,counts:[u32;256],joint:[u64;256],accepted:bool,k:Kernels}
impl Default for Job{
    fn default()->Self{Self{runs:mibz2::runs::Runs::default(),front:fused::Output::default(),syms:Vec::new(),ll:Vec::new(),counts:[0;256],joint:[0;256],accepted:false,k:Kernels::detect()}}
}
impl Job{
    fn read(&mut self,id:usize,data:&[u8],e:Envelope)->Result<Meta,ErrorKind>{
        use mibz2::block::Item;
        self.accepted=false;
        if id>=2{
            let mut reader=bits::Bits::at(data,e.start);
            let done=unsafe{if id==2{fused::read_fused::<50>(&mut reader,e.level,&mut self.front,&mut self.joint)}
                else{fused::read_fused::<800>(&mut reader,e.level,&mut self.front,&mut self.joint)}};
            if let Some((crc,mark,n))=done{self.accepted=true;return Ok(Meta{crc,mark,n,record:true,end:reader.bit_pos()});}
        }
        self.ordinary(id==1,data,e)
    }
    fn ordinary(&mut self,copy:bool,data:&[u8],e:Envelope)->Result<Meta,ErrorKind>{
        use mibz2::block::Item;
        let storage=std::mem::take(&mut self.syms);
        let (item,end,overran)=if copy{
            let mut br=bits::Bits::at(data,e.start);let v=stock::read_item(&mut br,e.level,storage);(v,br.bit_pos(),br.overran())
        }else{
            let mut br=mibz2::bits::Bits::at(data,e.start);let v=mibz2::block::read_item(&mut br,e.level,storage);(v,br.bit_pos(),br.overran())
        };
        let b=match item{Ok(Item::Block(b))=>b,Ok(Item::End(crc))=>return if overran{Err(ErrorKind::Truncated)}else{Ok(Meta{crc,mark:0,n:0,record:false,end})},
            Err((err,storage))=>{self.syms=storage;return Err(if overran{ErrorKind::Truncated}else{err});}};
        let route=mibz2::runs::takes_runs(&b);
        let front=if overran {Err(ErrorKind::Truncated)} else if route{
            mibz2::runs::front(&b,&mut self.runs,self.k)
        }else{
            mibz2::mtf::mtf(&b,&mut self.ll,&mut self.counts,self.k)
        };
        self.syms=b.syms;
        let n=front?;if b.orig_ptr as usize>=n{return Err(ErrorKind::BadOrigPtr);}
        Ok(Meta{crc:b.crc,mark:b.orig_ptr,n,record:route,end})
    }
    fn records(&self)->&[u16]{if self.accepted{&self.front.records}else{self.runs.records()}}
    fn output(&self,m:&Meta)->Vec<u8>{if m.record{
        let mut l=Vec::with_capacity(m.n);for &x in self.records(){l.extend(std::iter::repeat_n((x>>8)as u8,usize::from(x&255)));}l
    }else{self.ll.clone()}}
    fn exact(&self,refjob:&Self,m:&Meta){
        if m.n==0{return;}
        if m.record{
            assert_eq!(self.records(),refjob.records());
            if self.accepted{let mut calc=[0u64;256];for &w in refjob.records(){calc[(w>>8)as usize]+=(1u64<<32)|u64::from(w&255);}assert_eq!(self.joint,calc);}
        }else{assert_eq!(self.ll,refjob.ll);assert_eq!(self.counts,refjob.counts);}
    }
}
fn envelopes(input:&[u8])->Vec<Envelope>{
    use mibz2::stream::{Event,Scanner,SliceSource};
    let mut scan=Scanner::new(SliceSource(input));let mut out=vec![];let mut scratch=Vec::new();
    while let Event::Block{bit,end,level,stream,index,symbols}=scan.next(scratch).unwrap(){out.push(Envelope{start:bit,end,level,stream,index});scratch=symbols.syms;}
    out
}
fn check(data:&[u8],env:&[Envelope],oracle:&[u8]){
    let mut jobs:[Job;4]=std::array::from_fn(|_|Job::default());
    let mut plain=Plain::default();let mut coded=Vec::new();let mut raw=Vec::new();let mut offset=0;let mut accepted=[0;4];
    println!("block,stream,begin,end,level,rows,record_route,fused,record_count,output,crc");
    for &e in env{
        let(base,rest)=jobs.split_at_mut(1);let m=base[0].read(0,data,e).unwrap();assert_eq!(m.end,e.end);
        for (id,job)in rest.iter_mut().enumerate(){assert_eq!(job.read(id+1,data,e).unwrap(),m);job.exact(&base[0],&m);accepted[id+1]+=usize::from(job.accepted);}
        let chosen=&rest[2];let l=chosen.output(&m);assert_eq!(l.len(),m.n);let mut counts=[0;256];
        for &c in &l{counts[c as usize]+=1;}
        plain.invert(&l,&counts,m.mark,&mut coded);raw.clear();
        mibz2::rle::unrle(&coded,&mut raw,chosen.k).unwrap();assert_eq!(mibz2::crc::crc32(&raw,chosen.k),m.crc);
        assert_eq!(&oracle[offset..offset+raw.len()],raw.as_slice());offset+=raw.len();
        println!("{},{},{},{},{},{},{},{},{},{},{:08x}",e.index,e.stream,e.start,e.end,e.level,m.n,m.record,chosen.accepted,if m.record{chosen.records().len()}else{0},raw.len(),m.crc);
    }
    assert_eq!(oracle.len(),offset);eprintln!("all {} blocks, {} libbzip2 bytes, fused accepts {:?}",env.len(),offset,accepted);
}
fn adverse(data:&[u8],env:&[Envelope]){
    let mut cases=0;let mut good=0;let mut fallback=0;
    let mut jobs:[Job;4]=std::array::from_fn(|_|Job::default());
    for &e in env.iter().step_by(7){
        let a=(e.start/8)as usize;let b=e.end.div_ceil(8)as usize;
        let original=&data[a..b];let local=Envelope{start:e.start%8,end:e.end-(a as u64)*8,..e};
        let mut check=|bytes:&[u8]|{
            let(head,others)=jobs.split_at_mut(1);let v=head[0].read(0,bytes,local);
            for(j,x)in others.iter_mut().enumerate(){let got=x.read(j+1,bytes,local);assert_eq!(got,v);if let Ok(ref m)=got{x.exact(&head[0],m);}}
            good+=usize::from(v.is_ok());fallback+=usize::from(!others[2].accepted);cases+=1;
        };
        for len in [0,1,4,6,10,14,original.len()/4,original.len()/2,original.len().saturating_sub(1),original.len()]{check(&original[..len.min(original.len())]);}
        let mut state=1234u32;
        for _ in 0..16{state=state.wrapping_mul(1664525).wrapping_add(1013904223);let ix=state as usize%original.len();let mut v=original.to_vec();v[ix]^=1<<((state>>16)&7);check(&v);}
    }
    eprintln!("adverse and clean items {cases}, valid {good}, real fused fallback {fallback}; all four result and accepted records/column checks match");
}
#[repr(C)]struct Time{i:i64,n:i64}
unsafe extern "C"{fn clock_gettime(c:i32,t:*mut Time)->i32;}
fn cpu()->i64{let mut t=Time{i:0,n:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.i*1_000_000_000+t.n}
fn lap(j:&mut Job,id:usize,data:&[u8],env:&[Envelope])->(usize,usize,usize,u64){
    let mut rows=0;let mut used=0;let mut accepted=0;let mut fold=0u64;
    for &e in env{
        let m=j.read(id,data,e).unwrap();assert_eq!(m.end,e.end);rows+=m.n;accepted+=usize::from(j.accepted);
        let n=if m.record {let r=j.records();black_box(r);r.len()}else{black_box(&j.ll);j.ll.len()};
        used+=n;fold=fold.rotate_left(7)^u64::from(m.crc)^m.end;black_box(&j.joint);
    }
    (rows,used,accepted,fold)
}
fn main(){
    for h in [is_x86_feature_detected!("avx2"),is_x86_feature_detected!("bmi1"),is_x86_feature_detected!("bmi2"),is_x86_feature_detected!("popcnt"),is_x86_feature_detected!("lzcnt")]{assert!(h);}
    let args=std::env::args().collect::<Vec<_>>();let input=std::fs::read(&args[2]).unwrap();let env=envelopes(&input);assert_eq!(env.len(),431);
    if args[1]=="exact"{return check(&input,&env,&std::fs::read(&args[3]).unwrap());}
    if args[1]=="adverse"{return adverse(&input,&env);}
    let rounds:usize=args[3].parse().unwrap();let mut jobs:[Job;4]=std::array::from_fn(|_|Job::default());
    let orders=[[0,1,3,2],[1,2,0,3],[2,3,1,0],[3,0,2,1]];
    println!("round,position,arm,thread_ns,rows,front_items,accept,metadata_fold");
    for r in 0usize..=rounds{for(pos,&id)in orders[r.saturating_sub(1)%4].iter().enumerate(){
        let t=cpu();let (n,total,accept,fold)=lap(&mut jobs[id],id,&input,&env);let elapsed=cpu()-t;
        if r>0{println!("{r},{pos},{},{elapsed},{n},{total},{accept},{fold:016x}",["library","copy","fifty","eight_hundred"][id]);}
    }}
}
