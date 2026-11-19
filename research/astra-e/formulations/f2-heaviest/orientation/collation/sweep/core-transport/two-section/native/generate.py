"""Materialize the alternating-section variant from the pinned packed native constructor."""
from pathlib import Path
import hashlib
import subprocess
import sys

D=Path(__file__).resolve().parent
PREVIOUS=D.parents[1]/"follower/native"
def once(s,a,b):
    assert s.count(a)==1,(a,s.count(a))
    return s.replace(a,b)
def main(out):
    subprocess.run([sys.executable,"-B",PREVIOUS/"generate.py",out],check=True)
    original=(out/"projected.rs").read_text()
    assert hashlib.sha256(original.encode()).hexdigest()=="39aac3736d0994243dbe1f3fe64b8fe6400ff0c0f2aaeeb6d8afe85e6d40d802"
    cut=original.index("#[derive(Default,Clone,Copy)]\nstruct Slice {")
    first=original.index("    pub fn first(&mut self,caps",cut)
    s=original[:cut]+(D/"publish.rs").read_text()+"\nimpl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {\n"
    s+=original[first:]+"\n"+(D/"mark.rs").read_text()
    s=once(s,"    pub compose_ns:u64,","    pub compose_ns:u64,\n    pub alias_count:u32,\n    pub core_type_count:u32,\n    pub prefix_bytes:u32,")
    s=once(s,"p:u32,u:(u32,u32),h:usize)","p:u32,u:(u32,u32),h:usize,force:bool)")
    s=once(s,"self.original_mark(p,u)","self.lift_mark(p,u)")
    s=once(s,"self.compose(m)","self.publish_lift(m,force)")
    s=once(s,"Some((mark,m,missing))","Some((mark,2*m,missing))")
    (out/"lifted.rs").write_text(s)
    m=(out/"main.rs").read_text()
    a=m.index("#[derive(Default)] struct Subject {")
    b=m.index("fn name(",a)
    subject=m[a:b]
    subject=once(subject,"    physical:follower::","    lifted:lifted::Dispatch<2,2>,\n    physical:follower::")
    # Existing control codes remain -1 (named), 8 (old publication), 24 (physical follower).
    arm=subject.index("        let (a,m,q)=self.physical.first")
    subject=subject[:arm]+"""        if arm>=32 {
            let cap=if arm>=64{arm-64}else{arm-32};
            let (a,m,q)=self.lifted.first(&x.caps,&x.freq,x.n,x.p,u,cap as usize,arm<64)?;
            return Some(finish!(self.lifted,a,m,q,self.lifted.measures,self.lifted.compose_ns));
        }
"""+subject[arm:]
    rss=m[m.index("fn rss("):m.index("fn main(){")]
    # The getters below observe only populated second-stage vectors, outside the timed calls.
    sec=out/"second.rs"
    s=sec.read_text()
    assert hashlib.sha256(s.encode()).hexdigest()=="1ab93a07c04f227506c729f3292e6c7920c02d9dff2f03c2b80e18ec9930a475"
    sec.write_text(s+"""
impl Second{
    pub(super) fn information(&self)->[usize;7]{
        [self.edges.len(),self.nodes.len(),self.words.len(),self.dictionary.len(),self.selectors.len(),
         self.merged.len(),self.links.len()]
    }
}
""")
    for name in ["control","shared","projected","follower","lifted"]:
        path=out/f"{name}.rs"
        path.write_text(path.read_text()+"""
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn information(&self)->[usize;7]{self.second.information()}
}
""")
    # Exclude the previous physical oracle, which is not an input to this construction.
    common=m[:m.index("impl Truth {\n    // Derive the moved section")]
    (out/"main.rs").write_text("#[allow(dead_code,unused_imports)] mod lifted;\n"+common+
                               (D/"oracle.rs").read_text()+subject+(D/"main.rs").read_text().replace("// RSS",rss))
if __name__=="__main__":main(Path(sys.argv[1]))
