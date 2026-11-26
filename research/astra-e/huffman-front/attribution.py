"""Keep undisturbed paid binary; build a separate fully materialized parser plus identical tile front."""
from pathlib import Path
import subprocess
import sys
d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
subprocess.run([sys.executable,d/"generate.py",out],check=True)
p=out/"fused.rs"
s=p.read_text().replace("words:&[u16;864]","words:&[u16]")
s+='''
#[target_feature(enable="avx2,bmi1,bmi2,lzcnt,popcnt")]
pub unsafe fn known<const T:usize>(b:&mut Symbols,out:&mut Output,counts:&mut [u64;256])->Option<usize>{
    let m=b.syms.len();
    if m==0 || b.syms[m-1]!=b.eob{return None;}
    // The input is already materialized. Every wide load must read initialized owned words.
    b.syms.resize(m+64,0);
    let mut f=Front::new(&b.seq_to_unseq,b.max_len);
    out.reset(m,b.max_len);
    let result=(||{
        let mut start=0;
        while start<m-1{
            let n=(m-1-start).min(T);
            if !f.tile(&b.syms[start..],n,out){return None;}start+=n;
        }
        out.mask(0,1);
        if !out.admitted(b.max_len)||!f.close(out){return None;}
        let len=f.counts.iter().map(|q|(q&0xffff_ffff)as usize).sum();
        if len>b.max_len||b.orig_ptr as usize>=len{return None;}
        *counts=f.counts;Some(len)
    })();
    b.syms.truncate(m);result
}
'''
p.write_text(s)
s=(d/"driver.rs").read_text()
s=s.replace("if id>=2{","if id==2 || id==3{")
s=s.replace("self.ordinary(id==1,data,e)","self.ordinary(id,data,e)").replace("fn ordinary(&mut self,copy:bool,","fn ordinary(&mut self,id:usize,").replace("if copy{","if id==1{").replace("let b=match item","let mut b=match item")
a=s.index("        let route=mibz2::runs::takes_runs(&b);")
s=s[:a]+'''        if id==4 && !overran{
            if let Some(n)=unsafe{fused::known::<800>(&mut b,&mut self.front,&mut self.joint)}{
                self.accepted=true;self.syms=b.syms;return Ok(Meta{crc:b.crc,mark:b.orig_ptr,n,record:true,end});
            }
        }
'''+s[a:]
s=s.replace('let mut jobs:[Job;4]', 'let mut jobs:[Job;5]')
s=s.replace('let mut accepted=[0;4];','let mut accepted=[0;5];')
s=s.replace('    let rounds:usize=', '''    if args[1]=="memory"{
        for id in [0,2,3,4]{
            let mut j=Job::default();let mut scratch=(Vec::<u16>::new(),Vec::<u64>::new(),Vec::<u64>::new());
            for &e in &env{
                let m=j.read(id,&input,e).unwrap();
                if m.record&&!j.accepted{
                    let count=j.syms.len();let words=count/64+3;let cap=count+(e.level as usize*100000)/127+3;
                    scratch.0.clear();scratch.0.reserve(cap);scratch.1.clear();scratch.1.reserve(words);scratch.2.clear();scratch.2.reserve(words);
                }
                let retained_stock=scratch.0.capacity()*2+(scratch.1.capacity()+scratch.2.capacity())*8;
                println!("{id},{},{},{},{},{},{}",e.index,j.accepted,j.syms.capacity()*2,j.front.scratch_bytes(),retained_stock,j.ll.capacity());
            }
        }
        return;
    }
    let rounds:usize=''')
s=s.replace('let orders=[[0,1,3,2],[1,2,0,3],[2,3,1,0],[3,0,2,1]];', 'let orders=[[0,3,4],[3,4,0],[4,0,3],[4,3,0],[3,0,4],[0,4,3]];')
s=s.replace('%4].iter().enumerate()', '%6].iter().enumerate()')
s=s.replace('["library","copy","fifty","eight_hundred"][id]', '["library","copy","fifty","eight_hundred","materialized"][id]')
(out/"main.rs").write_text(s)
