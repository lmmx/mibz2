use std::{env,fs,io::Write};
use mibz2::{runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
include!("../edge-budget/gate.rs");
fn extend(spans:&mut Vec<(u32,u32)>,lo:u32,hi:u32){
    if let Some(last)=spans.last_mut(){if last.1==lo{last.1=hi;return;}}
    spans.push((lo,hi));
}
fn union_mask(left:&[(u32,u32)],right:&[(u32,u32)])->Vec<u32>{
    let (mut a,mut b,mut at)=(0,0,left[0].0.min(right[0].0));let mut mask:Vec<u32>=Vec::new();
    loop{
        while a<left.len()&&left[a].1<=at{a+=1;}
        while b<right.len()&&right[b].1<=at{b+=1;}
        if a==left.len()&&b==right.len(){break;}
        let l=left.get(a).copied().unwrap_or((u32::MAX,u32::MAX));
        let r=right.get(b).copied().unwrap_or((u32::MAX,u32::MAX));
        let flags=u32::from(l.0<=at)+2*u32::from(r.0<=at);
        let end=(if l.0<=at{l.1}else{l.0}).min(if r.0<=at{r.1}else{r.0});
        if flags!=0{
            let add=(end-at)<<2;
            if mask.last().is_some_and(|p|*p&3==flags){*mask.last_mut().unwrap()+=add;}else{mask.push(add|flags);}
        }
        assert!(end>at);at=end;
    }
    mask
}
fn main(){
    let args=env::args().collect::<Vec<_>>();
    let raw=fs::read(&args[1]).unwrap();let mut scan=Scanner::new(SliceSource(&raw));let k=Kernels::detect();
    let mut merge=args.get(3).map(|f|std::io::BufWriter::new(fs::File::create(f).unwrap()));
    let mut dest=std::io::BufWriter::new(fs::File::create(&args[2]).unwrap());let mut count=0;
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();let p=symbols.orig_ptr;
        let caps=r.records();if caps.len()*20>=3*n && (n<65536 || !fits(caps,p)){continue;}
        let mut thin=Vec::<(u8,u32)>::new();
        let mut source=Vec::new();let mut queue:[Vec<(u32,u32)>;256]=std::array::from_fn(|_|Vec::new());let mut occ=[0u32;256];let (mut i,mut at,mut m,mut mark,mut maxruns)=(0,0,0,u32::MAX,0);
        while i<caps.len(){
            let c=(caps[i]>>8)as u8;let mut w=0;maxruns+=1;
            while i<caps.len() && (caps[i]>>8)as u8==c{w+=(caps[i]&255) as u32;i+=1;}
            let primary=(at..at+w).contains(&p);let v=if w<=16{w}else{u32::from(primary)};
            if primary{mark=m+if w<=16{p-at}else{0};}
            if v!=0&&merge.is_some(){
                let start=if w<=16{at}else{p};extend(&mut source,start,start+v);
                let low=occ[c as usize]+start-at;extend(&mut queue[c as usize],low,low+v);
            }
            occ[c as usize]+=w;
            if v!=0{m+=v;if thin.last().is_some_and(|x|x.0==c){thin.last_mut().unwrap().1+=v;}else{thin.push((c,v));}}
            at+=w;
        }
        assert!(mark<m);assert_eq!(at,n as u32);
        for v in [index as u32,stream as u32,n as u32,m,mark,caps.len() as u32,maxruns,thin.len() as u32]{dest.write_all(&v.to_le_bytes()).unwrap();}
        for (c,w) in thin{dest.write_all(&((w<<8)|u32::from(c)).to_le_bytes()).unwrap();}
        if let Some(f)=merge.as_mut(){
            let mut dest=Vec::new();let mut base=0;
            for (byte,rows) in queue.iter().enumerate(){
                for &(l,h) in rows{extend(&mut dest,base+l,base+h);}
                base+=occ[byte];
            }
            let mask=union_mask(&source,&dest);let nmask=mask.len();
            for v in [index as u32,m,source.len() as u32,dest.len() as u32,nmask as u32]{f.write_all(&v.to_le_bytes()).unwrap();}
            for v in mask{f.write_all(&v.to_le_bytes()).unwrap();}
        }
        count+=1;
    }
    assert_eq!(count,233);dest.flush().unwrap();
}
