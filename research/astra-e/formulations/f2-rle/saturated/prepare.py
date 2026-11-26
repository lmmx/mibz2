"""Retain the original Open section; color only whole returns that own a hazardous byte."""
from pathlib import Path
from hashlib import sha256
here=Path(__file__).resolve().parent
journal=here.parents[4]/"docs/journal/2026-11-18-open-return"
raw=(journal/"open_return.rs").read_text()
assert sha256(raw.encode()).hexdigest()=="b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
s=raw[:raw.index("    pub fn run<const INDUCE:")]+"}\n"+raw[raw.index("fn emit("):raw.index("fn records_of(")]
s=s.replace("use crate::bwt::{InverseBwt, Segmented};","").replace("use crate::{runs::Runs, Kernels};","")
s=s.replace("    segments: segments::Segments,","    owners: Vec<u32>,labels:Vec<u32>,steps:usize,worked:bool,raw:Vec<u8>,")
s=s.replace("        limit: u32,\n    ) -> (Counts", "        limit: u32,\n        max_tags:u32,\n    ) -> (Counts",1)
s=s.replace("(255 + stats.words as u32)", "(255 + stats.words as u32 + max_tags)")
s+='''#[path="../../../../../docs/journal/2026-11-18-open-return/endpoint.rs"] mod endpoint;
include!("writer.rs");
'''
(here/"open.rs").write_text(s)
s=(here.parent/"candidate.rs").read_text().split("mod section;",1)[1]
s=s.replace("out: &mut Vec<u8>,\n        _: Kernels,","out: &mut Vec<u8>,\n        k: Kernels,")
s=s.replace("&self.hazards, out)", "&self.hazards, out, k)")
(here/"adapter.inc.rs").write_text(s)
