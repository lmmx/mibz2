"""Use the frozen F5 constructor and writer with one-byte identity hints."""
from pathlib import Path
import runpy

here = Path(__file__).resolve().parent
base = here.parent
runpy.run_path(str(base / "prepare.py"))
state = (base / "actions.rs").read_text().split("// After the first input,", 1)[0]
open_src = (base / "open.rs").read_text().replace(
    "../../../../../docs", "../../../../../../docs"
)
writer = (base / "writer.rs").read_text().replace(
    'include!("actions.rs");',state + '\ninclude!("book.rs");'
)
(here / "open.rs").write_text(open_src.replace('include!("writer.rs");', writer))
entry = (base / "candidate.rs").read_text().split("mod kernel;\n", 1)[1]
(here / "adapter.inc.rs").write_text(entry)
