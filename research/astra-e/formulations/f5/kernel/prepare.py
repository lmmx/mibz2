"""Regenerate the isolated constructor from the pinned queued Open-Return source."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
journal = here.parents[4] / "docs/journal/2026-11-18-open-return"
source = (journal / "open_return.rs").read_bytes()
if sha256(source).hexdigest() != "b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1":
    raise ValueError("unexpected Open-Return source")
constructor, _ = source.decode().split("    pub fn run<const INDUCE:", 1)
for unused in (
    "    segments: segments::Segments,\n",
    "use crate::bwt::{InverseBwt, Segmented};\n",
    "use crate::{runs::Runs, Kernels};\n",
):
    if unused not in constructor:
        raise ValueError(f"missing {unused!r}")
    constructor = constructor.replace(unused, "")
constructor += '''}
#[path="../../../../../docs/journal/2026-11-18-open-return/endpoint.rs"]
mod endpoint;
include!("writer.rs");
'''
(here / "open.rs").write_text(constructor)
