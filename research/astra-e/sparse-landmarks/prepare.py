from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys
d=Path(__file__).resolve().parent
base=d.parent/'narrow-short'
subprocess.run([sys.executable,str(d.parent/'edge-budget/prepare.py')],check=True)
p=base/'constructor_ahead.rs'
s=p.read_text()
assert sha256(p.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
import re
s=s.replace('//! First return','// First return',1)
s=re.sub(r'#\[path = "([^"]+)"\]',lambda m:'#[path = '+json.dumps(str((base/m[1]).resolve()))+']',s)
s+='''
impl Kernel {
    pub fn table(&mut self,records:&[u16],n:usize,p:u32)->crate::Table {
        let (_,mark,bits)=self.construct::<true,true>(records,n as u32,p,16);
        crate::Table{edges:self.edges.clone(),extra:self.extra.clone(),spelling:self.spelling.clone(),
            offsets:self.offsets.clone(),n,mark,bits}
    }
    pub fn stock(t:&crate::Table,out:&mut Vec<u8>){
        emit(&t.edges,&t.extra,&t.spelling,&t.offsets,t.n,t.mark,t.bits,out);
    }
}
'''
Path(sys.argv[1]).write_text(s)
