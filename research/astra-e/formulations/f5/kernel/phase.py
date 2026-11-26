"""Prepare a diagnostic copy with phase clocks; the frozen candidate stays clock-free."""
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
journal = repo / "docs/journal/2026-11-18-open-return"
dest = Path(sys.argv[1])
dest.mkdir(parents=True, exist_ok=True)
s = (journal / "open_return.rs").read_text()
for path in ("segments.rs", "endpoint.rs", "switches.rs", "checks.rs"):
    s = s.replace(f'#[path = "{path}"]', f'#[path = "{journal / path}"]')
start = s.index("    pub fn run<const INDUCE:")
end = s.index("    pub fn price<const INDUCE:", start)
observed = s[start:end].replace("pub fn run<", "pub fn observed<").replace("-> Counts", "-> [u64; 2]")
observed = observed.replace("let (stats, mark, bits)", "let t0=crate::cpu();\n        let (_, mark, bits)")
observed = observed.replace("if SEGMENTS {", "let t1=crate::cpu();\n        if SEGMENTS {")
observed = observed.replace("        stats\n", "        [t1-t0,crate::cpu()-t1]\n")
(dest / "old.rs").write_text(s + "\nimpl Kernel {\n" + observed + "}\n")
s=(here / "open.rs").read_text().replace('../../../../../docs/journal/2026-11-18-open-return/endpoint.rs',str(journal / "endpoint.rs"))
writer = (here / "writer.rs").read_text()
actions = (here / "actions.rs").read_text()
s=s.replace('include!("writer.rs");',writer.replace('include!("actions.rs");',actions))
observed=writer[writer.index('impl Kernel'):].replace('fn expanded(', 'fn observed(')
observed=observed.replace('Result<(), mibz2::ErrorKind>', 'Result<[u64;5], mibz2::ErrorKind>')
observed=observed.replace('let (_, mark, bits)', 'let start=crate::cpu();\n        let (_, mark, bits)')
observed=observed.replace('book.compile(', 'let t1=crate::cpu();\n        book.compile(')
observed=observed.replace('let mask =', 'let t2=crate::cpu();\n        let mask =')
observed=observed.replace('Ok(())','Ok([t1-start,t2-t1,crate::cpu()-t2,0,0])')
s+=observed+'''
impl Book {
    pub(super) fn sizes(&self)->[usize;5] {
        [self.actions.len(),std::mem::size_of::<Action>(),self.expanded.len(),self.expanded.capacity(),
        self.actions.capacity()*std::mem::size_of::<Action>()]
    }
}
'''
(dest / "direct.rs").write_text(s)
(dest / "main.rs").write_bytes((here / "phase.rs").read_bytes())
