"""Expose the unmodified Open-Return constructor in a private copy for this experiment."""
from pathlib import Path
from hashlib import sha256

here = Path(__file__).resolve().parent
journal = here.parents[2] / "docs/journal/2026-11-18-open-return"
original = (journal / "open_return.rs").read_bytes()
assert sha256(original).hexdigest() == "b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
source = original.decode()
# Keep only the shared data, trace helper and constructor. The pair writer supplies
# its own kernel entry point. Endpoint induction is the journal's original module.
source = source[:source.index("    pub fn run<const INDUCE:")]
source = source.replace("    segments: segments::Segments,\n", "")
source = source.replace("use crate::bwt::{InverseBwt, Segmented};\n", "")
source = source.replace("use crate::{runs::Runs, Kernels};\n", "")
source += '}\n#[path = "../../../docs/journal/2026-11-18-open-return/endpoint.rs"]\nmod endpoint;\n'
source += 'include!("writer.rs");\n'
(here / "constructor.rs").write_text(source)
