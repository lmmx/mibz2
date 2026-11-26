"""Use the exact same cut, tagged edge writer and fallback with the diagonal cylinder index."""
from pathlib import Path
import runpy
d=Path(__file__).resolve().parent
runpy.run_path(str(d.parent/"prepare.py"))
s=(d.parent/"open.rs").read_text().replace("../../../../docs","../../../../../docs").replace('include!("split.rs");','include!("../split.rs");').replace('include!("writer.rs");','include!("../writer.rs");')
(d/"open.rs").write_text(s)
entry=(d.parent/"candidate.rs").read_text().split("mod section;",1)[1]
(d/"adapter.inc.rs").write_text(entry)
