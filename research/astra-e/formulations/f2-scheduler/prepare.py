"""Replace the scheduler only, retaining the immutable per-node publication and second level."""
from hashlib import sha256
from pathlib import Path
import subprocess
here=Path(__file__).resolve().parent
s=(here.parent/"f2-frontier/candidate.rs").read_text()
assert sha256(s.encode()).hexdigest()=="b5ae8e9b6df171b790b53cc0cd346445537054294e936ca4d29f4389cbdfe919"
a=s.index("        let mut i = 0;")
b=s.index("                self.nodes[i].end =",a)
s=s[:a]+(here/"refill.rs").read_text()+s[b:]
s=s.replace("                i += 1;\n","")
s=s.replace("    lane_paths: [Vec<u8>; 8],", "    lane_paths: [Vec<u8>; 8],\n    #[cfg(test)] histogram:[u64;9],")
s=s.replace("        let mut next=0;","        #[cfg(test)] {self.histogram=[0;9];}\n        let mut next=0;")
s=s.replace("            if live==ALL {","            #[cfg(test)] {self.histogram[live.count_ones() as usize]+=1;}\n            if live==ALL {")
s+='\n#[cfg(test)] mod observations {use super::*;include!("checks.inc.rs");}\n'
(here/"candidate.rs").write_text(s)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",str(here/"candidate.rs")],check=True)
