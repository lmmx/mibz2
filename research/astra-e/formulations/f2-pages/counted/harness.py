"""Build a matched five-arm test from the frozen page and counts sources."""
from pathlib import Path
import re
import sys
import subprocess
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'sequential/harness.py'),str(out)],check=True)
price=(here.parents[1]/'f2c/price_impl.rs').read_text()
price=price.replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for module,directory in [('counted',here.parents[1]/'f2-front-counts'),('joint',here)]:
    source=(directory/'candidate.rs').read_text()
    def resolve(m):return m[0].replace(m[1],str((directory/m[1]).resolve()))
    source=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,source)
    (out/(module+'.rs')).write_text(source+'\n'+price)

s=(out/'price.rs').read_text()
s=s.replace('mod layout;','mod counted;\n#[allow(dead_code)]\nmod joint;')
a=s.index('    let mut p32 =')
b=s.index('    let mut ordinal',a)
s=s[:a]+'''    let mut seq = sequential::PageCandidate::<64,true>::default();
    let mut packed = sequential::PageCandidate::<44>::default();
    let mut old = counted::Candidate::default();
    let mut combo = joint::Candidate::default();
'''+s[b:]
a=s.index('        println!("block,rows,')
b=s.index('    } else {\n        assert_eq!(mode, "price")',a)
s=s[:a]+'        println!("block,rows,capped_records");\n'+s[b:]
a=s.index('            macro_rules! check')
b=s.index('            continue;',a)
s=s[:a]+'''            macro_rules! check { ($arm:expr) => {{
                assert!($arm.invert_runs(&r,n,p,&mut out,kernels));
                assert_eq!(out,want);
            }}; }
            check!(base); check!(seq); check!(packed); check!(old); check!(combo);
            println!("{index},{n},{}",r.records().len());
'''+s[b:]
s=s.replace('0..=32usize','0..=40usize').replace('for pos in 0..8','for pos in 0..5').replace('rot/8%2 == 0 {pos} else {7-pos}) % 8','rot/5%2 == 0 {pos} else {4-pos}) % 5')
a=s.index('                    1 => p32.measure')
b=s.index('\n                };',a)
s=s[:a]+'''                    1 => seq.measure(&r,n,p,&mut out,true),
                    2 => packed.measure(&r,n,p,&mut out,true),
                    3 => old.measure(&r,n,p,&mut out,true),
                    _ => combo.measure(&r,n,p,&mut out,true),'''+s[b:]
s=s.replace('["split64","seq32","seq40","seq44","seq_split64","packed32","packed44","seq_wide64"]','["split64","seq_split64","seq44","counted","joint"]')
(out/'price.rs').write_text(s)
