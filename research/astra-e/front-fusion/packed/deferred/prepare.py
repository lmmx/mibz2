"""Move n, marked edge eligibility and density accounting out of the MTF dependency chain."""
from pathlib import Path
import subprocess,sys

d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d.parent/"prepare.py"),str(out)],check=True)
p=out/"front.rs";s=p.read_text().replace("unsafe fn pack(","unsafe fn pack<const DEFER:bool>(")
s=s.replace("if total+row_count as u64>max_len", "if !DEFER && total+row_count as u64>max_len").replace("if row_count>max_len","if !DEFER && row_count>max_len")
s=s.replace("cap_difference+=total.div_ceil(127)as usize-words;","if !DEFER{cap_difference+=total.div_ceil(127)as usize-words;}")
s=s.replace("row_count+=total as usize;","if !DEFER{row_count+=total as usize;}")
p.write_text(s)
p=out/"packed.rs";s=p.read_text()
a=s.index("fn each_run(");b=s.index("fn starts(",a)
checked=s[a:b].strip().replace("fn each_run(records: &[u16],","fn checked_runs(records:&[u16], max:u32,").replace("u32, bool)) {","u32, bool))->Option<u32>{")
checked=checked.replace("let end = pos + width;", "let end=pos+width; if end>max{return None;}")
assert checked.endswith("}");checked=checked[:-1]+"Some(pos)\n}"
a=s.index("    fn construct<const");b=s.index("    pub fn run<const",a)
core=s[a:b]
start=core.index("        let mut all");core=core[start:]
core=core.replace("        each_run(records, p, limit, |c, x, w, keep| {", "        let n=checked_runs(records,max,p,limit,|c,x,w,keep|{")
old="        });\n        assert!(m > 0 && all.iter().sum::<u32>() == n);"
assert old in core
core=core.replace(old,'''        })?;
        if p>=n{return None;}
        let edge_bits=(32-(m-1).leading_zeros()).max(1);
        let field=if 511+3*self.strips.len() as u32<=u32::MAX>>edge_bits{4}else{8};
        if n<65536 || m*field>1<<20 {
            let(mut i,mut caps)=(0,0u32);
            while i<records.len(){let(_,w)=crate::log::take(records,&mut i);caps+=w.div_ceil(127);}
            if 20*caps>=3*n{return None;}
        }
''')
core=core.replace("        (stats, mark, bits)","        emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n as usize,mark,bits,out);\n        Some(())")
core=core.replace("return (stats, mark, bits);","unreachable!();")
s += "\n"+checked+"\nimpl Kernel {\n pub fn deferred(&mut self,records:&[u16],max:u32,p:u32,out:&mut Vec<u8>)->Option<()> {\nconst QUEUE:bool=true;const INDUCE:bool=true;let limit=16;let mut stats=Counts::default();\n"+core+"}\n"
p.write_text(s)
p=out/"main.rs";s=p.read_text()
a=s.index("        let dst=if kind==2");b=s.index("                let finish=",a)
s=s[:a]+'''        let mut words=0;
        let result=if kind==3{
            unsafe{front::pack::<true>(src,m,src,b.eob,&b.seq_to_unseq,b.max_len,&mut self.bits,&mut words)}
        }else{
            unsafe{front::pack::<false>(src,m,src,b.eob,&b.seq_to_unseq,b.max_len,&mut self.bits,&mut words)}
        };
        if let Some((n,caps))=result{
            let records=&b.syms[..words];
            let admitted=if kind==3{self.kernel.deferred(records,b.max_len as u32,b.orig_ptr,&mut self.raw).is_some()}
            else if n>0&&(b.orig_ptr as usize)<n&&(20*caps<3*n || (n>=65536 && packed_fits(records,b.orig_ptr))){
                self.kernel.run::<true,true,false>(records,n,b.orig_ptr,16,&mut self.raw);true
            }else{false};
            if admitted{
'''+s[b:]
s=s.replace("if kind==2 && words!=0","if words!=0")
p.write_text(s)
p=out/"errors.rs";s=p.read_text()
s=s.replace("let(mut checks,mut rollback,mut direct,mut bytes)=(0,0,0,0);","let mut checks=0;let(mut rollback,mut direct,mut bytes)=([0;2],[0;2],[0;2]);")
s=s.replace("if kind==2{rollback+=usize::from(!done&&words>0);direct+=usize::from(done);if !done{bytes+=words*2;}}","let idx=kind-2;rollback[idx]+=usize::from(!done&&words>0);direct[idx]+=usize::from(done);if !done{bytes[idx]+=words*2;}")
s=s.replace("assert!(rollback>=17 && direct>=13);","assert!(rollback.iter().all(|&v|v>=17) && direct.iter().all(|&v|v>=13));")
s=s.replace("{rollback} mutated", "{rollback:?} mutated").replace("({bytes} packed", "({bytes:?} packed").replace("{direct} directly","{direct:?} directly")
p.write_text(s)
