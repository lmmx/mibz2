"""Expand the compact payload crossover to five independent complete-inverse states."""
from pathlib import Path
import re,subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here.parent/'harness.py'),str(out)],check=True)
price=(here.parents[2]/'f2c/price_impl.rs').read_text()
price=price.replace('impl Candidate {','impl<const MODE:u8> Dispatch<MODE> {').replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,file in [('coalesced','candidate.rs'),('eager','eager.rs')]:
    s=(here/file).read_text()
    def resolve(m):return m[0].replace(m[1],str((here/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    s+='\n'+price+'''
impl<const MODE:u8> Dispatch<MODE> {
    pub fn storage(&self)->(usize,usize) { (self.runs.len(),self.runs.capacity()*size_of::<Run>()) }
}
'''
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text().replace('mod ends;', 'mod ends;\n#[allow(dead_code)]\nmod coalesced;\n#[allow(dead_code)]\nmod eager;')
s=s.replace('    let mut tmp=','    let mut grouped=coalesced::Candidate::default();\n    let mut immediate=eager::Candidate::default();\n    let mut tmp=')
s=s.replace('            assert_eq!(c0,c1);','            assert_eq!(check!(grouped),(c2,b2));\n            assert_eq!(check!(immediate),(c2,b2));\n            assert_eq!(c0,c1);')
s=s.replace('let mut outputs: [Vec<u8>;3]', 'let mut outputs: [Vec<u8>;5]')
s=s.replace('    let perms=[[0,1,2],[1,2,0],[0,2,1],[2,0,1],[1,0,2],[2,1,0]];', '''    let perms: [[usize;5];10]=std::array::from_fn(|r| std::array::from_fn(|j| {
        let base=[0usize,1,4,2,3];
        (base[if r<5 {j} else {4-j}] + r%5)%5
    }));''')
s=s.replace('perms[(pass+bi)%6]', 'perms[(pass+bi)%10]')
s=s.replace('                    _=>z.measure', '''                    2=>z.measure(&b.r,b.n,b.mark,out,true),
                    3=>grouped.measure(&b.r,b.n,b.mark,out,true),
                    _=>immediate.measure''')
s=s.replace('["control12","starts8","ends8"]','["control12","starts8","delayed","coalesced","eager"]')
(out/'focus.rs').write_text(s)
