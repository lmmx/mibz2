from hashlib import sha256
from pathlib import Path
import subprocess
here=Path(__file__).resolve().parent
s=(here.parents[1]/"f2c/candidate.rs").read_text()
assert sha256(s.encode()).hexdigest()=="42cc096c2ae3031530875a6ea413785e7d93c8bc2bdda762430d71a7d6f9eb25"
s=s.replace('#[derive(Default)]\npub struct Candidate {','''#[derive(Clone,Copy)]
struct Job {lo:u32,hi:u32,id:u32,label:u32}
#[derive(Default)]
pub struct Candidate {
    current:Vec<Vec<Job>>,
    next:Vec<Vec<Job>>,
    labels:Vec<u32>,''')
a=s.index("    fn build(");b=s.index("    #[inline]\n    fn emit(",a)
s=s[:a]+(here/"build.rs").read_text()+s[b:]
s=s.replace('#[path = "second.rs"]','#[path = "../../f2c/second.rs"]').replace('#[path = "checks.rs"]','#[path = "../../f2c/checks.rs"]')
(here/"candidate.rs").write_text(s)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",str(here/"candidate.rs")],check=True)
