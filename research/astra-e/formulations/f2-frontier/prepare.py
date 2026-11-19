"""Construct the frontier scheduler from the exact F2c first-level discovery loop."""
from hashlib import sha256
from pathlib import Path

here=Path(__file__).resolve().parent
base=here.parent / "f2c/candidate.rs"
src=base.read_text()
assert sha256(src.encode()).hexdigest() == "42cc096c2ae3031530875a6ea413785e7d93c8bc2bdda762430d71a7d6f9eb25"
src=src.replace('    second: second::Second,','    second: second::Second,\n    lane_paths: [Vec<u8>; 8],')
before=src.index('        while i < self.nodes.len()')
after=src.index('            self.nodes[i].end',before)
src=src[:before]+(here/'walk.rs').read_text()+src[after:]
e='            i += 1;\n        }\n    }'
assert src.count(e)==1
src=src.replace(e, '                i += 1;\n            }\n        }\n        self.lane_paths = paths;\n    }')
src=src.replace('#[path = "second.rs"]','#[path = "../f2c/second.rs"]').replace('#[path = "checks.rs"]','#[path = "../f2c/checks.rs"]')
(here/'candidate.rs').write_text(src)
import subprocess
subprocess.run(["rustfmt", "--edition", "2021", "--config", "skip_children=true", str(here/"candidate.rs")], check=True)
