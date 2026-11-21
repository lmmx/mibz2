"""Expose the pinned queued endpoint constructor beside its streamed writer."""
from pathlib import Path
import runpy

here = Path(__file__).resolve().parent
base = here.parent / "pairs"
runpy.run_path(str(base / "prepare.py"))
(here / "constructor.rs").write_bytes((base / "constructor.rs").read_bytes())
