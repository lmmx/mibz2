"""Generate quotient and matched controls on precisely the earlier sparse/named Compact code."""
from hashlib import sha256
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
PREV=HERE.parents[1]/'native'

def main(out):
    subprocess.run([sys.executable,'-B',PREV/'generate.py',out],check=True)
    p=out/'control.rs'
    s=p.read_text()
    expected=(HERE/'control.sha').read_text().strip()
    assert sha256(s.encode()).hexdigest()==expected
    assert s.count('    links: Vec<u32>,')==1
    s=s.replace('    links: Vec<u32>,','    links: Vec<u32>,\n    quot_events: Vec<QEvent>,')
    s=s.replace('+self.second.capacity()+0','+self.second.capacity()+bytes(&self.quot_events)')
    sec=out/'second.rs'
    sec.write_text(sec.read_text()+'''
impl Second {
    pub(super) fn chosen_base(&self)->u32 {
        let i=(0..self.count.len()).max_by_key(|&j|(self.count[j],u32::MAX-self.base[j])).unwrap();
        self.base[i]
    }
}
''')
    p.write_text(s+'\n'+(HERE/'quotient.rs').read_text()+'''
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn chosen_base(&self)->u32{self.second.chosen_base()}
}
''')
    m=(out/'main.rs').read_text()
    m=m[:m.index('#[allow(dead_code,unused_imports)] mod lifted12;')]
    m=m.replace('#[allow(dead_code,unused_imports)] mod lifted;\n','')
    (out/'main.rs').write_text(m+(HERE/'oracle.rs').read_text()+(HERE/'main.rs').read_text())

if __name__=='__main__':main(Path(sys.argv[1]))
