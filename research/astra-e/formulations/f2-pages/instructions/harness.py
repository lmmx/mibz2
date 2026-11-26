"""Reuse the matched F2 inputs and clocks with one counts control and five ISA routes."""
from pathlib import Path
import re, subprocess, sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'counted/harness.py'),str(out)],check=True)
s=(here/'candidate.rs').read_text()
def resolve(m):return m[0].replace(m[1],str((here/m[1]).resolve()))
s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
price=(here.parents[1]/'f2c/price_impl.rs').read_text()
price=price.replace('impl Candidate {','impl<const MODE:u8> Dispatch<MODE> {').replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
(out/'dispatch.rs').write_text(s+'\n'+price)
s=(out/'price.rs').read_text()
a=s.index('#[allow(dead_code)]\nmod baseline;')
b=s.index('#[repr(C)]',a)
s=s[:a]+'''#[allow(dead_code)]
mod counted;
#[allow(dead_code)]
mod dispatch;
'''+s[b:]
a=s.index('    let mut base =')
b=s.index('    let mut ordinal',a)
s=s[:a]+'''    let mut old = counted::Candidate::default();
    let mut generic = dispatch::Dispatch::<0>::default();
    let mut discover = dispatch::Dispatch::<1>::default();
    let mut both = dispatch::Dispatch::<2>::default();
    let mut bmi = dispatch::Dispatch::<3>::default();
    let mut prep = dispatch::Dispatch::<4>::default();
'''+s[b:]
s=s.replace('check!(base); check!(seq); check!(packed); check!(old); check!(combo);','check!(old); check!(generic); check!(discover); check!(both); check!(bmi); check!(prep);')
s=s.replace('0..=40usize','0..=36usize').replace('for pos in 0..5','for pos in 0..6').replace('rot/5%2 == 0 {pos} else {4-pos}) % 5','rot/6%2 == 0 {pos} else {5-pos}) % 6')
a=s.index('                    0 => base.measure')
b=s.index('\n                };',a)
s=s[:a]+'''                    0 => old.measure(&r,n,p,&mut out,true),
                    1 => generic.measure(&r,n,p,&mut out,true),
                    2 => discover.measure(&r,n,p,&mut out,true),
                    3 => both.measure(&r,n,p,&mut out,true),
                    4 => bmi.measure(&r,n,p,&mut out,true),
                    _ => prep.measure(&r,n,p,&mut out,true),'''+s[b:]
s=s.replace('["split64","seq_split64","seq44","counted","joint"]','["counted","generic","discover","both","bmi2","prepare"]')
(out/'price.rs').write_text(s)
