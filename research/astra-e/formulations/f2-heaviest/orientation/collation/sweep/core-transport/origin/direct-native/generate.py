"""Build the two paid fixed-literal sections with the hash-checked clipped Compact writer."""
import hashlib
from pathlib import Path
import subprocess
import sys

D=Path(__file__).resolve().parent
FOLLOWER=D.parents[1]/"follower/native"
def main(out):
    subprocess.run([sys.executable,"-B",FOLLOWER/"generate.py",out],check=True)
    for module in ("shared","control"):
        p=out/f"{module}.rs"
        s=p.read_text()
        s+="""
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn info(&self)->[usize;7]{
        let root=self.nodes[0].out as usize;
        [self.words.len(),self.dict.len(),self.merged.len()-root,
         self.edges.len(),self.selectors.len(),self.runs.len(),self.second.dict_size()]
    }
}
"""
        if module=="shared":
            s+=(D/"dispatch.rs").read_text()
        p.write_text(s)
    sec=out/"second.rs";s=sec.read_text()
    assert hashlib.sha256(s.encode()).hexdigest()=="1ab93a07c04f227506c729f3292e6c7920c02d9dff2f03c2b80e18ec9930a475"
    sec.write_text(s+"\nimpl Second{pub(super) fn dict_size(&self)->usize {self.dictionary.len()}}\n")
    m=(out/"main.rs").read_text()
    start=m.index("use std::collections::HashMap;")
    end=m.index("impl Truth {\n    // Derive the moved section")
    m="pub use mibz2::{bwt,runs,Kernels};\n#[allow(dead_code,unused_imports)] mod shared;\n#[allow(dead_code,unused_imports)] mod control;\n"+m[m.index("type Obj="):start]+m[start:end]
    (out/"main.rs").write_text(m+(D/"selector.rs").read_text()+"\n"+(D/"main.rs").read_text())
if __name__=="__main__":main(Path(sys.argv[1]))
