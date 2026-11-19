"""Reuse the pinned endpoint eliminator under the sparse-window and primary cuts."""
from pathlib import Path
from hashlib import sha256

here=Path(__file__).resolve().parent
journal=here.parents[3]/"docs/journal/2026-11-18-open-return"
code=(journal/"open_return.rs").read_text()
assert sha256(code.encode()).hexdigest()=="b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
code=code[:code.index("    pub fn run<const INDUCE:")]+"}\n"
code=code[:code.index("// Production records")]+ 'include!("split.rs");\n'+code[code.index("fn starts("):]
code=code.replace("use crate::bwt::{InverseBwt, Segmented};","").replace("use crate::{runs::Runs, Kernels};","")
code=code.replace("    segments: segments::Segments,","    protected:Vec<u32>,")
code=code.replace("        limit: u32,\n    ) -> (Counts", "        limit: u32,\n        h:&super::section::Hazards,\n    ) -> (Counts",1)
code=code.replace("        let mut last = None;","        let mut last = None;\n        self.protected.clear();\n        let mut d=0;")
code=code.replace("each_run(records, p, limit, |", "each_run(records,p,limit,&h.seal, |")
needle="            if keep {\n                if (x..x + w).contains(&p)"
code=code.replace(needle,'''            if keep {
                while d<h.danger.len() && h.danger[d]<x+w {
                    assert!(h.danger[d]>=x);
                    self.protected.push(m+(h.danger[d]-x)); d+=1;
                }
                if (x..x + w).contains(&p)''')
code=code.replace("assert!(m > 0", "assert_eq!(d,h.danger.len());\n        assert!(m > 0")
code=code.replace("(255 + stats.words as u32)", "(511 + stats.words as u32)")
code+='''#[path="../../../../docs/journal/2026-11-18-open-return/endpoint.rs"]
mod endpoint;
include!("writer.rs");
'''
(here/"open.rs").write_text(code)
