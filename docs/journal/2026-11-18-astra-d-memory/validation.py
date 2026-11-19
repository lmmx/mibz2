"""Add verification entry points to a disposable stock, direct or tile tree after capture."""

from pathlib import Path
import sys

tree = Path(sys.argv[1])
mode = sys.argv[2]
here = Path(__file__).resolve().parent
p = tree / "src/lib.rs"
s = p.read_text() + (here / "corpus.rs").read_text()
if mode != "stock":
    s += (here / ("tests_" + mode + ".rs")).read_text()
p.write_text(s)
p = tree / "src/runs.rs"
p.write_text(
    p.read_text()
    + """
impl Runs { pub(crate) fn astra_n(&self) -> usize { self.counts.iter().map(|&x|x as usize).sum() } }
"""
)
(tree / "examples/astra_corpus.rs").write_text("""fn main(){
    let a:Vec<_>=std::env::args().collect();
    mibz2::astra_corpus(&a[1],&a[2],&a[3],a.get(4).is_some());
}
""")
