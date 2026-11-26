"""Generate the width-guarded admission adapter around the frozen combined constructor."""
import runpy
from pathlib import Path
here = Path(__file__).resolve().parent
source = here.parent / "narrow-short"
runpy.run_path(str(source / "prepare.py"))
adapter = (source / "adapter.inc.rs").read_text()
old = "if r.records().len() * 20 >= n * 3 {"
assert adapter.count(old) == 1
adapter = adapter.replace(old, "if r.records().len() * 20 >= n * 3 && (n < 65_536 || !fits(r.records(), p)) {")
(here / "adapter.inc.rs").write_text(adapter)
