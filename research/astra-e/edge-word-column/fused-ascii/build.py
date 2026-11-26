"""Four-arm native corpus/price audit using the exact edge-displacement harness."""
import pathlib
import shutil
import subprocess
import sys

import generate as src

out=pathlib.Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
base=out/'stock'
src.frozen(base)
original=(base/src.KERNEL).read_text()
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',base/'src/lib.rs','-o',lib],check=True)
code='#![allow(dead_code)]\n'
for name in ['stock','local','bucket']:
    tree=out/name
    if tree!=base:
        shutil.copytree(base,tree,dirs_exist_ok=True)
    text=original if name=='stock' else src.modify(original,name=='bucket')
    # The same one wall-clock boundary in every arm.
    anchor='        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records, n as u32, p, limit);'
    assert text.count(anchor)==1
    text=text.replace(anchor,anchor+'\n        crate::boundary();')
    geometries='(0,0)' if name!='bucket' else '''(
       8*self.pair_dir.starts.len()+4*self.pair_dir.prefix.len()+12*self.pair_dir.partner.len(),
       8*self.pair_dir.starts.capacity()+4*self.pair_dir.prefix.capacity()+12*self.pair_dir.partner.capacity())'''
    text+='''
impl Kernel {
    pub fn displacement_entry(&self,row:usize,bits:u32)->(u32,u32){
        let v=self.edges[row];
        (v&((1u32<<bits)-1),if self.extra.is_empty(){v>>bits}else{self.extra[row]})
    }
    pub fn displacement_storage(&self)->[usize;4]{
        let (dl,dc)='''+geometries+''';
        [4*(self.edges.len()+self.extra.len())+dl,4*(self.edges.capacity()+self.extra.capacity())+dc,
        self.spelling.len()+4*self.offsets.len(),self.spelling.capacity()+4*self.offsets.capacity()]
    }
    pub fn first_pair_for_check(&self)->u32{'''+('0' if name=='stock' else 'self.pair_first')+'''}
}
'''
    (tree/src.KERNEL).write_text(text)
    code+=f'#[path="{name}/{src.KERNEL}"] mod {name};\n'
check=(base/src.ASTRA/'narrow-endpoint/check.rs').read_text()
code+=check[check.index('fn reference'):check.index('fn main()')]
code+=(base/src.ASTRA/'edge-budget/gate.rs').read_text()
shutil.copy(base/src.ASTRA/'edge-budget/adapter.inc.rs',out/'adapter.inc.rs')
harness=(src.ROOT/src.ASTRA/'edge-displacements/price.rs').read_text()
a=harness.index('                if a!=0 {')
z=harness.index('                let [s,o,e,c]',a)
harness=harness[:a]+'''                if a==3 {
                    let m=workers[0].storage(0)[0]/4;
                    let bits=(usize::BITS-(m-1).leading_zeros()).max(1);
                    let first=workers[a].bucket.first_pair_for_check();
                    let mut prim=0;
                    let mut at=0usize;
                    let mut retained=0usize;
                    let mut i=0usize;
                    while i<r.records().len(){
                        let byte=r.records()[i]>>8;
                        let mut w=0usize;
                        while i<r.records().len()&&r.records()[i]>>8==byte{
                            w+=usize::from(r.records()[i]&255);i+=1;
                        }
                        let hit=(at..at+w).contains(&(p as usize));
                        if hit{prim=retained+if w<=16{p as usize-at}else{0};}
                        retained+=if w<=16{w}else{usize::from(hit)};
                        at+=w;
                    }
                    assert_eq!(m,retained);
                    for row in 0..m {
                        let (y,c)=workers[0].entry(0,row,bits);
                        let (t,z)=workers[0].entry(0,y as usize,bits);
                        let expected=if first!=0&&c<128&&z<128&&y as usize!=prim{(t,first+128*c+z)}else{(y,c)};
                        assert_eq!(workers[a].entry(a,row,bits),expected);
                    }
                }
'''+harness[z:]
harness=harness.replace('local and bucket kernels','legacy and directly fused pair kernels')
driver=out/'price.rs';driver.write_text(code+harness)
subprocess.run(flags+[driver,'--extern',f'mibz2={lib}','-o',out/'price'],check=True)
