"""Build the unchanged combined-ahead entry used by the final-slot candidate."""
from pathlib import Path
import runpy

runpy.run_path(str(Path(__file__).resolve().parent.parent / "narrow-short/prepare.py"))
