"""Produce the scalar, fixed-eight or refill-eight second-level arm, fixing the same first scheduler."""
from hashlib import sha256
from pathlib import Path
import subprocess
import sys
here=Path(__file__).resolve().parent
form=here.parents[1]
mode=sys.argv[1] if len(sys.argv)>1 else 'refill'
dest=Path(sys.argv[2]).resolve() if len(sys.argv)>2 else here
dest.mkdir(parents=True,exist_ok=True)
first=(here.parent/"candidate.rs").read_text().replace('../f2c/second.rs','second.rs')
s=(form/"f2c/second.rs").read_text()
assert sha256(s.encode()).hexdigest()=="877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992"
assert sha256((here.parent/"candidate.rs").read_bytes()).hexdigest()=="1a659b1c65663db8901146d58623261c238c87dcd1a5ce6cb234c372cd59d4bf"
if mode!='scalar':
    s=s.replace("    runs: Vec<TokenRun>,", "    runs: Vec<TokenRun>,\n    paths:[Vec<u32>;8],")
    template=here.parent/"refill.rs" if mode=="refill" else form/"f2-frontier/walk.rs"
    walk=template.read_text()
    walk=walk.replace("Vec<u8>","Vec<u32>").replace("self.lane_paths","self.paths")
    walk=walk.replace("node.first","node.edge").replace("paths[lane].push(n.first)","paths[lane].push(n.edge)")
    walk=walk.replace("self.locate(","self.find(").replace("r.byte","r.word").replace("QUOTE","d")
    if mode=="refill":
        walk=walk.replace("            if live==ALL {","            #[cfg(test)]{self.events[0]+=live.count_ones() as u64;}\n            if live==ALL {")
    elif mode=="batch":
        walk=walk.replace("            while live!=0 {","            while live!=0 {\n                #[cfg(test)]{self.events[0]+=live.count_ones() as u64;}")
        walk="        let mut i = 0;\n"+walk
    else: raise ValueError(mode)
    a=s.index("        let mut i = 0;",s.index("    fn discover("))
    b=s.index("            self.nodes[i].end =",a)
    s=s[:a]+walk+"\n                let a=lo; let b=hi;\n"+s[b:]
    c=s.index('            i += 1;',a);e=s.index('    fn merge(',c)
    s=s[:c]+('                i+=1;\n' if mode=='batch' else '')+'            }\n        }\n        self.paths=paths;\n    }\n\n'+s[e:]
first=first.replace('#[path = "../f2c/checks.rs"]','#[path = "../../f2c/checks.rs"]').replace('include!("checks.inc.rs")','include!("../checks.inc.rs")')
(dest/"candidate.rs").write_text(first)
(dest/"second.rs").write_text(s)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",str(dest/"second.rs"),str(dest/"candidate.rs")],check=True)
