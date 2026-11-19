"""Extend the original five-arm driver to a balanced eight-arm page-builder crossover."""
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
s=(here/'candidate.rs').read_text()
def resolve(m): return m[0].replace(m[1],str((here/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
price=(here.parents[1]/'f2c/price_impl.rs').read_text().replace('impl Candidate {','impl<const WIDTH:u32,const SPLIT:bool> PageCandidate<WIDTH,SPLIT> {')
geom=(here.parent/'geometry.inc.rs').read_text().replace('impl<const WIDTH: u32> PageCandidate<WIDTH>', 'impl<const WIDTH:u32,const SPLIT:bool> PageCandidate<WIDTH,SPLIT>')
geom=geom.replace('self.pages.packed.capacity() * 8','self.pages.split.capacity() * 4 + self.pages.packed.capacity() * 8')
(out/'sequential.rs').write_text(s+'\n'+price+'\n'+geom)
s=(out/'price.rs').read_text().replace('mod layout;', 'mod layout;\n#[allow(dead_code)]\nmod sequential;')
s=s.replace('layout::PageCandidate', 'sequential::PageCandidate')
s=s.replace('    let mut ordinal = 0;', '''    let mut split = sequential::PageCandidate::<64,true>::default();
    let mut old32 = layout::PageCandidate::<32>::default();
    let mut old44 = layout::PageCandidate::<44>::default();
    let mut ordinal = 0;''')
s=s.replace('                p32.check_pages(&caps, n as u32);','                split.check_pages(&caps, n as u32);\n                p32.check_pages(&caps, n as u32);')
s=s.replace('        p32 = Default::default();', '        split = Default::default();\n        p32 = Default::default();')
s=s.replace('check!(p32,32);', 'check!(split,"split64"); check!(p32,32);')
s=s.replace('0..=30usize', '0..=32usize').replace('for pos in 0..5', 'for pos in 0..8')
s=s.replace('rot/5%2 == 0 {pos} else {4-pos}) % 5','rot/8%2 == 0 {pos} else {7-pos}) % 8')
s=s.replace('_ => p64.measure', '''4 => split.measure(&r,n,p,&mut out,true),
                    5 => old32.measure(&r,n,p,&mut out,true),
                    6 => old44.measure(&r,n,p,&mut out,true),
                    _ => p64.measure''')
s=s.replace('["split64","packed32","packed40","packed44","wide64"]','["split64","seq32","seq40","seq44","seq_split64","packed32","packed44","seq_wide64"]')
(out/'price.rs').write_text(s)
