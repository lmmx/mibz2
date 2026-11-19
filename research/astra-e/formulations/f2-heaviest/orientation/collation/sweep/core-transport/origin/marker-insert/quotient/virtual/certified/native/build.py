from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
out=Path(sys.argv[1]);deps=sys.argv[2]
subprocess.run([sys.executable,'-B',str(HERE.parents[1]/'native'/'build.py'),str(out),deps],check=True)
def one(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
p=out/'second.rs';s=p.read_text()
s=one(s,'    pub(super) stream:bool,','    pub(super) stream:bool,\n    pub(super) use_forest:bool,\n    forest:Forest,\n    ready:usize,')
s=one(s,'let mut i = usize::from(ROOT_READY);','let mut i = if ROOT_READY {self.ready} else {0};')
s=one(s,'        let (d,mut row,mut begin)=if self.stream {','''        self.ready=1;
        let (d,mut row,mut begin)=if self.use_forest {
            let Some(triple)=self.forest_index(root,first_nodes,first_words,bytes,p,skip,m) else{return false;};
            triple
        } else if self.stream {''')
s=one(s,'bytes(&self.runs)+bytes(&self.count)','self.forest.capacity()+bytes(&self.runs)+bytes(&self.count)')
p.write_text(s+(HERE/'forest.rs').read_text())
p=out/'control.rs';s=p.read_text()
p.write_text(s+'''
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn set_forest(&mut self,on:bool){self.second.use_forest=on;}
}
''')
p=out/'main.rs';s=p.read_text()
s=one(s,'self.fast.set_root_stream(inspect!=0 || route>=6);','self.fast.set_root_stream(inspect!=0 || route>=6);self.fast.set_forest(inspect!=0 || route>=12);')
s=one(s,'let codes=[0,6,3,9,4,10];','let codes=[0,12,3,15,5,17];')
s=one(s,'let names=["named","named_stream","quot4","quot4_stream","quot8","quot8_stream"];','let names=["named","named_forest","quot4","quot4_forest","quot12","quot12_forest"];')
s=one(s,'let code=[0,6,3,9,4,10][arm];let name=["named","named_stream","quot4","quot4_stream","quot8","quot8_stream"][arm];','let code=[0,12,3,15,5,17][arm];let name=["named","named_forest","quot4","quot4_forest","quot12","quot12_forest"][arm];')
p.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(p),'--extern','mibz2='+deps+'/libmibz2.rlib','-L','dependency='+deps,'-o',str(out/'forest')],check=True)
