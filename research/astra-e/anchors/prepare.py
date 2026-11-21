"""Prepare a private Open-Return constructor with reusable anchor scratch."""
from pathlib import Path
import runpy

here = Path(__file__).resolve().parent
# Identical pinned constructor and endpoint code to the two-literal experiment.
base = here.parent / "pairs"
runpy.run_path(str(base / "prepare.py"))
source = (base / "constructor.rs").read_text()
source = source.replace("pub struct Kernel {", "pub struct Kernel {\n    anchors: AnchorScratch,", 1)
(here / "constructor.rs").write_text(source)
