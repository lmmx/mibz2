"""Schedule direct compound-word discovery in the verified eight-job breadth-first batches."""
from hashlib import sha256
from pathlib import Path
import subprocess

here=Path(__file__).resolve().parent
base=here.parent/"f2-word/candidate.rs"
src=base.read_text()
assert sha256(src.encode()).hexdigest()=="643235ceca780e12a4dedf37c69a9d2123c0dfc3c0cc850a331a1b467e193bcf"
walk=(here.parent/"f2-frontier/walk.rs").read_text()
walk=walk.replace("paths[lane].push(node.first);","if node.parent != NONE { paths[lane].push(node.first); }")
walk=walk.replace("|| r.byte==QUOTE", "|| (section..section+tokens).contains(&(r.lf+(lo[lane]-r.start)))")
src=src.replace("    fallback: Segmented,","    fallback: Segmented,\n    lane_paths: [Vec<u8>;8],")
before=src.index("        while i < self.nodes.len()")
after=src.index("            self.nodes[i].end",before)
src=src[:before]+walk+src[after:]
end="            i += 1;\n        }\n    }"
assert src.count(end)==1
src=src.replace(end,"                i += 1;\n            }\n        }\n        self.lane_paths=paths;\n    }")
src=src.replace('#[path = "checks.rs"]','#[path = "../f2-word/checks.rs"]')
candidate=here/"candidate.rs"
candidate.write_text(src)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",candidate],check=True)
