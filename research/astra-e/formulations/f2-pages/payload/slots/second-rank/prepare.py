"""Generate each second-column variant against the immutable eight-slot preload."""
from pathlib import Path
from hashlib import sha256

here = Path(__file__).resolve().parent
first = (here.parent / "candidate.rs").read_text()
stock = (here.parents[3] / "f2c/second.rs").read_text()
assert sha256(first.encode()).hexdigest() == "84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06"
assert sha256(stock.encode()).hexdigest() == "877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992"
first = first.replace('../../../f2c/second.rs', 'second.rs').replace('pub type Candidate = Dispatch<2>;', 'pub type Candidate = Dispatch<2,2,2>;')
first = first.replace('const SCHEDULE:u8=1>', 'const SCHEDULE:u8=2,const SEC:u8=2>').replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const SEC:u8>').replace('Dispatch<MODE,SCHEDULE>', 'Dispatch<MODE,SCHEDULE,SEC>').replace('second: second::Second,', 'second: second::Second<SEC>,')
(here / 'family.rs').write_text(first)
s = stock.replace('struct Second {', 'struct Second<const SEC: u8> {').replace('impl Second {', 'impl<const SEC: u8> Second<SEC> {')
start = s.index('        self.runs.clear();')
s = s[:start]+s[start:].replace('        self.runs.clear();','''        self.runs.clear();
        let pages = m.div_ceil(64) as usize;
        if SEC == 2 {
            Self::reset(&mut self.masks, pages, 0);
        }''',1)
s=s.replace('            self.runs.push(TokenRun {','''            if SEC == 2 && row != 0 {
                self.masks[(row >> 6) as usize] |= 1u64 << (row & 63);
            }
            self.runs.push(TokenRun {''',1)
a=s.index('        let pages = m.div_ceil(64) as usize;',s.index('debug_assert_eq!(row, m);'))
b=s.index('        self.runs.push(TokenRun {',a)
s=s[:a]+'''        if SEC != 2 {
            Self::reset(&mut self.masks, pages, 0);
            for r in &self.runs {
                let offset = r.start & 63;
                if offset != 0 {
                    self.masks[(r.start >> 6) as usize] |= 1u64 << offset;
                }
            }
        }
'''+s[b:]
a=s.index('        let mut k = 0;');b=s.index('        // The most frequent token;',a)
s=s[:a]+'''        if SEC == 2 {
            self.page.clear();
            self.page.reserve(pages);
            let mut ordinal = 0;
            for mask in &mut self.masks {
                ordinal += (*mask & 1) as u32;
                self.page.push(ordinal);
                *mask &= !1;
                ordinal += mask.count_ones();
            }
        } else {
            Self::reset(&mut self.page, pages, 0);
            let mut k = 0;
            for (page, idx) in self.page.iter_mut().enumerate() {
                while self.runs[k + 1].start <= page as u32 * 64 {
                    k += 1;
                }
                *idx = k as u32;
            }
        }
'''+s[b:]
# Dispatch outside each block-wide hot loop. The ordinary method remains callable on every architecture.
for name, typed, args, result, minimum in [
    ('column', 'root: &[Piece], first_nodes: &[Node], alphabet: usize, m: u32', 'root, first_nodes, alphabet, m', ' -> u32', 2),
    ('discover', 'd: u32, first_words: &[Piece], bytes: &[u8]', 'd, first_words, bytes', '', 1),
]:
    signature = f'fn {name}(&mut self, {typed}){result} {{'
    assert s.count(signature)==1
    prefix=f'''fn {name}(&mut self, {typed}){result} {{
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if SEC >= {minimum} && std::is_x86_feature_detected!("popcnt") {{
            // SAFETY: runtime detection establishes POPCNT for this thread.
            return unsafe {{ self.{name}_pop({args}) }};
        }}
        self.{name}_body({args})
    }}

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "popcnt")]
    unsafe fn {name}_pop(&mut self, {typed}){result} {{
        self.{name}_body({args})
    }}

    #[inline(always)]
    fn {name}_body(&mut self, {typed}){result} {{'''
    s=s.replace(signature,prefix)
s=s.replace('#[inline]\n    fn find(', '#[inline(always)]\n    fn find(')
(here/'second.rs').write_text(s)
