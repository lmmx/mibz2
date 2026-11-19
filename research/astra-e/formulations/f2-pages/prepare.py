"""Build the page-layout experiment from the pinned F2 dynamic scheduler."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
source = (here.parent / "f2-scheduler/candidate.rs").read_text()
assert sha256(source.encode()).hexdigest() == "1a659b1c65663db8901146d58623261c238c87dcd1a5ce6cb234c372cd59d4bf"


def edit(old, new):
    global source
    assert source.count(old) == 1, old
    source = source.replace(old, new)


edit("#[derive(Default)]\npub struct Candidate", """include!("rank.inc.rs");

pub type Candidate = PageCandidate<44>;

#[derive(Default)]
pub struct PageCandidate<const WIDTH: u32>""")
edit("    page_first: Vec<u32>,\n    starts: Vec<u64>,", "    pages: RankPages,")
edit("impl Candidate {", "impl<const WIDTH: u32> PageCandidate<WIDTH> {")
edit("impl InverseBwt for Candidate {", "impl<const WIDTH: u32> InverseBwt for PageCandidate<WIDTH> {")
edit("if row != n || counts", "if row != n || (WIDTH != 64 && n > 1 << 20) || counts")
start = source.index("        let pages = n.div_ceil(64)")
end = source.index("        self.runs.push(Run {\n            start: n", start)
source = source[:start] + """        self.pages.clear::<WIDTH>(n);
        for r in &mut self.runs {
            r.lf += counts[r.byte as usize];
            self.pages.cut::<WIDTH>(r.start);
        }
""" + source[end:]
start = source.index("        let mut k = 0;")
end = source.index("        Some((counts", start)
source = source[:start] + "        self.pages.bases::<WIDTH>(n, &self.runs);\n" + source[end:]
start = source.index("        let page = (x >> 6)")
end = source.index("\n    }", start)
source = source[:start] + "        self.pages.locate::<WIDTH>(x)" + source[end:]
edit('#[path = "../f2c/checks.rs"]', '#[path = "checks.rs"]')
edit('include!("checks.inc.rs")', 'include!("../f2-scheduler/checks.inc.rs")')
(here / "candidate.rs").write_text(source)
