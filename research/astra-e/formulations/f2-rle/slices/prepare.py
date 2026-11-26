"""Build the isolated narrow predecessor with O(number of substitutions) extra word storage."""
from pathlib import Path
import subprocess, sys

d = Path(__file__).resolve().parent
base = d.parent / "count-only"
subprocess.run([sys.executable, str(base / "prepare.py")], check=True)
core = (base / "open.rs").read_text().replace('include!("nav.inc.rs");', 'include!("../count-only/nav.inc.rs");')
core = core.replace("edits: Vec<Edit>,", "edit_start:Vec<u32>, edits: Vec<Edit>,")
(d / "open.rs").write_text(core)
old = (base / "patches.rs").read_text()
a = old.index("    fn extend_raw_part(")
b = old.index("    fn edits(", a)
c = old.index("    fn patch(", b)
e = old.index("    pub fn expanded(", c)
s = old[:a] + old[b:c] + (d / "writer.rs").read_text() + old[e:]
(d / "patches.rs").write_text(s)
(d / "adapter.inc.rs").write_bytes((base / "adapter.inc.rs").read_bytes())
