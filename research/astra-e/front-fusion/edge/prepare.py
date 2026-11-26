"""Compare frozen edge admission against eager and conditional bitmap cap counting."""
from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d.parent/"precount/prepare.py"),str(p)],check=True)
gate=subprocess.check_output(["git","show","badb2aec3624bd65606c429b2e6f79e9d9d0050f:research/astra-e/edge-budget/gate.rs"],text=True)
(p/"gate.rs").write_text(gate)
s=(p/"pre.rs").read_text()
s=s.replace("fn front_avx2(b:", "fn front_avx2<const LAZY:bool>(b:").replace("impl FnMut(u8,u32)->bool","impl FnMut(u8,u32,bool)->bool")
s=s.replace("-> Option<(usize,usize)>", "-> Option<(usize,usize,usize)>")
s=s.replace("let (caps,_)=capped_count(&f.lit,&f.one,end)?;\n    if caps*20>=3*b.max_len{return None;}", "let caps=if LAZY {usize::MAX} else {capped_count(&f.lit,&f.one,end)?.0};")
s=s.replace("($c:expr, $total:expr) =>", "($c:expr, $total:expr, $boundary:expr) =>")
s=s.replace("($c:expr, $total:expr, $span:expr) =>","($c:expr, $total:expr, $span:expr, $boundary:expr) =>")
s=s.replace("visit(c as u8,total as u32)", "visit(c as u8,total as u32,$boundary)")
s=s.replace("long!(c, total);", "long!(c, total,$boundary);")
s=s.replace("_bzhi_u64(*onep, q as u32 + 1) - 1);", "_bzhi_u64(*onep, q as u32 + 1) - 1,true);")
s=s.replace(">> tz, t2 - tz);", ">> tz, t2 - tz,false);").replace("count!(q, q2), q2 - q);","count!(q, q2), q2 - q,true);")
s=s.replace("then_some((row_count,caps))","then_some((row_count,caps,end))")
(p/"pre.rs").write_text(s)
s=(p/"kernel.rs").read_text()
a=s.index("impl Kernel {\n    pub(super) fn fuse_pre")
before=s[:a];s=s[a:]
s=s.replace("fn fuse_pre<const TILE:usize>","fn fuse_edge<const LAZY:bool>")
a=s.index("        let mut tile=");b=s.index("        let cap_count=p.caps;",a)
s=s[:a]+'''        let (n,mut caps,end)=unsafe{crate::pre::front_avx2::<LAZY>(b,f,|c,w,boundary|{
            p.run(self,c,w);
            !boundary || p.m<=262_144
        })}?;
        if n==0 || b.orig_ptr as usize>=n {return None;}
        let bits=(32-(p.m-1).leading_zeros()).max(1);
        let code_bound=511+3*self.strips.len() as u32;
        let width=if code_bound<=u32::MAX>>bits{4}else{8};
        let edge_ok=n>=65_536 && p.m*width<=1<<20;
        if !edge_ok{
            if LAZY{caps=crate::pre::capped_count(&f.lit,&f.one,end)?.0;}
            if caps*20>=3*n{return None;}
        }
        p.caps=if caps==usize::MAX{u32::MAX}else{caps as u32};
'''+s[b:]
(p/"kernel.rs").write_text(before+s)
main=(p/"main.rs").read_text().replace("mod pre;","mod pre;\ninclude!(\"gate.rs\");")
a=main.index("            fused=if TILE==64");b=main.index(";\n",a)
main=main[:a]+'''            fused=if TILE==64{self.kernel.fuse_edge::<true>(b,&mut self.bits,&mut self.raw)}else{self.kernel.fuse_edge::<false>(b,&mut self.bits,&mut self.raw)}'''+main[b:]
main=main.replace("if caps.len()*20<3*n {", "if caps.len()*20<3*n || (n>=65_536 && fits(caps,b.orig_ptr)) {")
main=main.replace("assert_eq!((n,caps),(n0,c0));", "assert_eq!(n,n0);if caps!=u32::MAX as usize&&c0!=u32::MAX as usize{assert_eq!(caps,c0);}")
(p/"main.rs").write_text(main)
# Keep independent error and full-output checks even where a real cap count was elided.
adv=(p/"adversarial.rs").read_text().replace('a.map(|(n,c,_)|(n,c))','a.map(|(n,_,_)|n)').replace('z.map(|(n,c,_)|(n,c))','z.map(|(n,_,_)|n)')
(p/"adversarial.rs").write_text(adv)
# Time all five newly edge-admitted large blocks in addition to the systematic sample.
s=(p/"main.rs").read_text().replace("let selected=!large || ordinal%24==0;", "let selected=!large || ordinal%24==0 || [41,56,165,270,278].contains(&index);")
(p/"main.rs").write_text(s)
