"""Generate the paid root-streaming variant with matched explicit-column arms."""
from hashlib import sha256
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
parent=HERE.parents[1]/'native'
out=Path(sys.argv[1])
out.mkdir(exist_ok=True,parents=True)
subprocess.run([sys.executable,'-B',parent/'generate.py',str(out)],check=True)

def one(s,orig,repl):
    assert s.count(orig)==1,orig
    return s.replace(orig,repl)

p=out/'second.rs'
s=p.read_text()
assert sha256(s.encode()).hexdigest()==(HERE/'second.sha').read_text().strip()
s=one(s,'    pub(super) phases:[u64;4],','    pub(super) phases:[u64;4],\n    pub(super) stream:bool,')
s=one(s,'    fn discover(&mut self,','    fn discover<const ROOT_READY:bool>(&mut self,')
s=one(s,'        self.nodes.clear();','        if !ROOT_READY {\n        self.nodes.clear();')
s=one(s,'        let mut i = 0;','        }\n        let z=d as usize;\n        let mut i = usize::from(ROOT_READY);')
s=one(s,'mapped - self.base[z];','if ROOT_READY {mapped} else {mapped - self.base[z]};')
old=s[s.index('        let d = self.column('):s.index('        let t1=crate::now()')]
s=one(s,old,'''        let (d,mut row,mut begin)=if self.stream {
            let Some(triple)=self.root_index(root,first_nodes,first_words,bytes,p,skip,m) else{return false;};
            triple
        } else {
'''+old+'''            (d,row,begin)
        };
        let token_count=self.count[d as usize];
''')
s=one(s,'        self.discover(d, first_words, bytes);','        if self.stream {self.discover::<true>(d,first_words,bytes);} else {self.discover::<false>(d,first_words,bytes);}')
p.write_text(s+(HERE/'stream.rs').read_text())
p=out/'control.rs'
s=p.read_text()
p.write_text(s+'''
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn set_root_stream(&mut self,on:bool){self.second.stream=on;}
}
''')
p=out/'main.rs'
s=p.read_text()
s=one(s,'let t=now();\n        let (mut mark','self.fast.set_root_stream(inspect!=0 || route>=6);\n        let route=route%6;\n        let t=now();\n        let (mut mark')
s=one(s,'let codes=[0,0,2,3,4,5];','let codes=[0,6,3,9,4,10];\n    let names=["named","named_stream","quot4","quot4_stream","quot8","quot8_stream"];')
s=one(s,'x.index,LABELS[arm],r.first[0]','x.index,names[arm],r.first[0]')
p.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(p),'--extern','mibz2='+sys.argv[2]+'/libmibz2.rlib','-L','dependency='+sys.argv[2],'-o',str(out/'rootstream')],check=True)
