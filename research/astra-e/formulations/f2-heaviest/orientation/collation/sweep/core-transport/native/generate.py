"""Create isolated Rust core transport from the pinned packed projected Compact."""
import hashlib
from pathlib import Path
import subprocess
import sys

D=Path(__file__).resolve().parent
SHIFT=D.parents[1]/"shift-compact"

def once(s,a,b):
    assert s.count(a)==1,(a,s.count(a))
    return s.replace(a,b)

def main(out):
    subprocess.run([sys.executable,"-B",SHIFT/"generate.py",out],check=True)
    path=out/"projected.rs"
    s=path.read_text()
    assert hashlib.sha256(s.encode()).hexdigest()=="650ea5a8fc985b00468d3c7f2f84376df977108e4b48dc6547bca8922af2eac0"
    a=s.index("#[derive(Clone,Copy)]\nstruct Early");b=s.index("#[repr(C, align(8))]",a)
    s=s[:a]+(D/"atlas.rs").read_text()+"\n"+s[b:]
    s=once(s,"    plan:Plan,", "    plan:Atlas,\n    transport:Transport,\n    pub compose_ns:u64,\n    pub setup_scratch:usize,")
    s=once(s,"let target=self.plan.target();","let target=&self.plan.target;")
    a=s.index("    fn mark_shifted(");b=s.index("    fn prepare<const ONLY:bool>(",a)
    s=s[:a]+s[b:]
    a=s.index("        for &(from,base) in &self.plan.types{");b=s.index("        let mut root",a)
    s=s[:a]+s[b:]
    s=once(s,"                else{paths[lane].extend(self.plan.prefix.iter().rev().copied());}","")
    a=s.index("    pub fn first(&mut self,caps:");b=s.index("    pub fn object(",a)
    s=s[:a]+s[b:]
    a=s.index("+bytes(&self.plan.levels)");b=s.index("\n    }",a)
    s=s[:a]+"+self.plan.bytes()+self.transport.bytes()"+s[b:]
    s+="\n"+(D/"compose.rs").read_text()
    path.write_text(s)
    generated=(out/"main.rs").read_text()
    helpers=generated[generated.index("use std::collections::HashMap;"):generated.index("impl Truth{\n    fn shifted")]
    subject=generated[generated.index("#[derive(Default)] struct Subject{"):generated.index("fn name(")]
    # Exclude the untimed copy of the token object in timed calls.
    subject=subject.replace("cap:usize,first:u64,total:u64","cap:usize,first:u64,total:u64,compose_ns:u64,setup_scratch:usize")
    subject=subject.replace("Receipt{obj,mark:", "Receipt{obj,compose_ns:if arm<0{0}else{self.projected.compose_ns},setup_scratch:if arm<0{0}else{self.projected.setup_scratch},mark:")
    common=generated[generated.index("fn highest("):generated.index("// The full sorted")]
    (out/"main.rs").write_text(generated[:generated.index("use std::collections::HashMap;")]+helpers+subject+common+
                               (D/"main.rs").read_text().replace("// RSS",(SHIFT/"rss.inc.rs").read_text()))

if __name__=="__main__":
    main(Path(sys.argv[1]))
