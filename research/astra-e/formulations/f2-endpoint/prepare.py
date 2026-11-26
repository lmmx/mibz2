"""Reuse the measured Open-Return eliminator, changing only which rows it retains."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
source = here.parents[3] / "docs/journal/2026-11-18-open-return/open_return.rs"
s = source.read_text()
assert sha256(s.encode()).hexdigest() == "b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
lo = s.index("// Production records")
hi = s.index("fn starts(", lo)
s = s[:lo] + 'include!("section.rs");\n\n' + s[hi:]
s = s.replace("limit: u32,", "section: (u32,u32),")
s = s.replace("each_run(records, p, limit,", "each_section_run(records, p, section,")
s = s.replace("p, limit)", "p, section)")
# Keep only the two observed entry points, the pinned emitter and source modules.
s = s[:s.index("    pub fn price")] + "}\n\n" + s[s.index("fn emit("):s.index("fn records_of(")]
s = s.replace("    #[allow(dead_code)]\n}", "}")
s = s.replace("use crate::bwt::{InverseBwt, Segmented};\nuse crate::{runs::Runs, Kernels};", "")
s += '''impl Kernel {
    pub fn section(caps:&[u16], pattern:&[u8])->Option<(u32,u32)> {cylinder(caps,pattern)}
    pub fn sizes(&self)->(usize,usize,usize) {(self.spelling.len(), self.offsets.len(), self.extra.len())}
}
'''
for module in ("endpoint", "segments"):
    s += f'#[path="../../../../docs/journal/2026-11-18-open-return/{module}.rs"] mod {module};\n'
(here / "kernel.rs").write_text(s)
