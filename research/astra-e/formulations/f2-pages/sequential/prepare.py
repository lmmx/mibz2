"""Generate a page-major F2 constructor from the measured two-pass candidate."""
from hashlib import sha256
from pathlib import Path

here = Path(__file__).resolve().parent
base = here.parent
s = (base/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest() == '020c2d38b631da23f09d03f697cb847021566fa10dbee9f47760a55b4b5760dd'
s = s.replace('../f2c/second.rs','../../f2c/second.rs')
s = s.replace('../f2-scheduler/checks.inc.rs','../../f2-scheduler/checks.inc.rs')
s = s.replace('path = "checks.rs"', 'path = "../checks.rs"')
pages = (base/'rank.inc.rs').read_text()
a = pages.index('    fn clear<')
b = pages.index('    #[inline]\n    fn locate',a)
pages = pages[:a] + pages[b:]
pages = pages.replace('    packed: Vec<u64>,','    packed: Vec<u64>,\n    split: Vec<u32>,')
s = s.replace('include!("rank.inc.rs");',pages+'\ninclude!("packing.inc.rs");')
s = s.replace('struct PageCandidate<const WIDTH: u32>', 'struct PageCandidate<const WIDTH: u32, const SPLIT: bool = false>')
s = s.replace('impl<const WIDTH: u32>', 'impl<const WIDTH: u32, const SPLIT: bool>')
s = s.replace('PageCandidate<WIDTH>', 'PageCandidate<WIDTH,SPLIT>')
start = s.index('        self.pages.clear::<WIDTH>(n);')
end = s.index('        self.runs.push(Run {\n            start: n,',start)
s = s[:start] + s[end:]
s = s.replace('self.pages.bases::<WIDTH>(n, &self.runs);', 'self.pages.build::<WIDTH,SPLIT>(n, &mut self.runs, &counts);')
s = s.replace('self.pages.locate::<WIDTH>(x)', 'self.pages.rank::<WIDTH,SPLIT>(x)')
(here/'candidate.rs').write_text(s)
