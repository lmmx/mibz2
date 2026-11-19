"""Apply the same dynamic-refill schedule to the compound-word first-return construction."""
from hashlib import sha256
from pathlib import Path
import subprocess
here=Path(__file__).resolve().parent
src=(here.parents[1]/"f2-frontier-direct/candidate.rs").read_text()
assert sha256(src.encode()).hexdigest()=="9ab806ac57b727f6a54a3ca0d9da549bfc8f0155cf3a14ac27db61a0b16a6d91"
walk=(here.parent/"refill.rs").read_text()
walk=walk.replace("paths[lane].push(n.first);","if n.parent!=NONE {paths[lane].push(n.first);}")
walk=walk.replace("r.byte == QUOTE","(section..section+tokens).contains(&(r.lf+(lo[lane]-r.start)))")
a=src.index("        let mut i = 0;"); b=src.index("                self.nodes[i].end =",a)
src=src[:a]+walk+src[b:]
src=src.replace("                i += 1;\n","")
src=src.replace('../f2-word/checks.rs','../../f2-word/checks.rs')
(here/"candidate.rs").write_text(src)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",str(here/"candidate.rs")],check=True)
