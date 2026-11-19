use std::{env,fs,io::Write};
use mibz2::{runs,Kernels};
use mibz2::stream::{Event,Scanner,SliceSource};
include!("../edge-budget/gate.rs");
fn main(){
    let args=env::args().collect::<Vec<_>>();
    let raw=fs::read(&args[1]).unwrap();let mut scan=Scanner::new(SliceSource(&raw));let k=Kernels::detect();
    let mut dest=std::io::BufWriter::new(fs::File::create(&args[2]).unwrap());let mut count=0;
    while let Event::Block{index,stream,symbols,..}=scan.next(Vec::new()).unwrap(){
        if !runs::takes_runs(&symbols){continue;}
        let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();let p=symbols.orig_ptr;
        let caps=r.records();if caps.len()*20>=3*n && (n<65536 || !fits(caps,p)){continue;}
        let mut thin=Vec::<(u8,u32)>::new();let (mut i,mut at,mut m,mut mark,mut maxruns)=(0,0,0,u32::MAX,0);
        while i<caps.len(){
            let c=(caps[i]>>8)as u8;let mut w=0;maxruns+=1;
            while i<caps.len() && (caps[i]>>8)as u8==c{w+=(caps[i]&255) as u32;i+=1;}
            let primary=(at..at+w).contains(&p);let v=if w<=16{w}else{u32::from(primary)};
            if primary{mark=m+if w<=16{p-at}else{0};}
            if v!=0{m+=v;if thin.last().is_some_and(|x|x.0==c){thin.last_mut().unwrap().1+=v;}else{thin.push((c,v));}}
            at+=w;
        }
        assert!(mark<m);assert_eq!(at,n as u32);
        for v in [index as u32,stream as u32,n as u32,m,mark,caps.len() as u32,maxruns,thin.len() as u32]{dest.write_all(&v.to_le_bytes()).unwrap();}
        for (c,w) in thin{dest.write_all(&((w<<8)|u32::from(c)).to_le_bytes()).unwrap();}
        count+=1;
    }
    assert_eq!(count,233);dest.flush().unwrap();
}
