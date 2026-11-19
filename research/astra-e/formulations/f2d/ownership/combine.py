"""Apply the verified eight-lane frontier walk to the dictionary-ownership constructor."""
import hashlib
from pathlib import Path
import subprocess

here = Path(__file__).resolve().parent
walk = (here.parents[1] / "f2-frontier/walk.rs").read_text()
assert hashlib.sha256(walk.encode()).hexdigest() == "31661d61dbc85d1773fc85be1045c7b32f1b72c2def661efa5b16908214394cc"
src = (here / "candidate.rs").read_text()
src = src.replace("    second: second::Second,", "    second: second::Second,\n    lane_paths: [Vec<u8>;8],")
before = src.index("        while i < self.nodes.len()")
after = src.index("            self.nodes[i].end",before)
src = src[:before] + walk + "                let node = self.nodes[i];\n" + src[after:]
end = "            i += 1;\n        }\n    }"
assert src.count(end) == 1
src = src.replace(end,"                i += 1;\n            }\n        }\n        self.lane_paths = paths;\n    }")
output = here / "batched.rs"
output.write_text(src)
subprocess.run(["rustfmt","--edition","2021","--config","skip_children=true",str(output)], check=True)
