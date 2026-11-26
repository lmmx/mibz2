"""Generate the physical-follower dispatch and the matched named/original observers."""
import hashlib
from pathlib import Path
import subprocess
import sys

D=Path(__file__).resolve().parent
PREVIOUS=D.parents[1]/"native"

def once(s,original,replacement):
    assert s.count(original)==1,(original,s.count(original))
    return s.replace(original,replacement)

def main(out):
    subprocess.run([sys.executable,"-B",PREVIOUS/"generate.py",out],check=True)
    original=(out/"projected.rs").read_text()
    assert hashlib.sha256(original.encode()).hexdigest()=="709375cac6a6fb7eec82fd9751972c166e07818783a15f12c67fff6143b6ff87"
    sec=(out/"second.rs").read_text()
    assert hashlib.sha256(sec.encode()).hexdigest()=="60c828228d27397676d822cede82d2e1033e0af6507184572782a0dc10edc80b"
    sec=once(sec,"pub(super) struct Second {","pub(super) struct Second {\n    pub(super) phases:[u64;4],")
    sec=once(sec,"        let d = self.column(","        self.phases=[0;4];\n        let t0=crate::now();\n        let d = self.column(")
    sec=once(sec,"        self.discover(d, first_words, bytes);\n        self.merge();",
             """        let t1=crate::now();self.phases[0]=t1-t0;
        self.discover(d, first_words, bytes);
        let t2=crate::now();self.phases[1]=t2-t1;
        self.merge();
        let t3=crate::now();self.phases[2]=t3-t2;""")
    sec=once(sec,"        true\n    }\n}", "        self.phases[3]=crate::now()-t3;\n        true\n    }\n}")
    (out/"second.rs").write_text(sec)
    getter="""
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn second_phases(&self)->[u64;4]{self.second.phases}
}
"""
    for name in ["shared","control","projected"]:
        p=out/f"{name}.rs"
        p.write_text(p.read_text()+getter)
    start=original.index("#[derive(Default,Clone,Copy)]\nstruct Slice {")
    end=original.index("    pub fn first(&mut self,caps",start)
    s=original[:start]+(D/"publish.rs").read_text()+"\nimpl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {\n"
    s+=original[end:]+"\n"+(D/"mark.rs").read_text()
    s=once(s,"self.original_mark(p,u)","self.physical_mark(p,u)")
    s=once(s,"self.compose(m)","self.publish(m)")
    (out/"follower.rs").write_text(s+getter)
    m=(out/"main.rs").read_text()
    assert hashlib.sha256(m.encode()).hexdigest()=="5ccd5d885e17c8277fa0ffc5e9204d1a0b99ec470b7cd38f0bd7f2c3f9530418"
    a=m.index("#[derive(Default)] struct Subject{")
    b=m.index("fn highest(",a)
    c=m.index("fn name(",b)
    rss=m[m.index("fn rss("):m.index("fn main(){")]
    # Original and physical depth use distinct arms so both are measured on the same instance of each input.
    (out/"main.rs").write_text("#[allow(dead_code,unused_imports)] mod follower;\n"+m[:a]+m[b:c]+
                               (D/"oracle.rs").read_text()+(D/"main.rs").read_text().replace("// RSS",rss))
if __name__=="__main__":main(Path(sys.argv[1]))
