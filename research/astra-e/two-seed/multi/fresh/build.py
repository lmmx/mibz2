from pathlib import Path
import subprocess
import sys

d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
lib=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,d.parent/'build_worker.py',out],check=True)
for source,slot,target in [('scalar.rs','scalar','emit'),('scalar-four.rs','four','twoseed::emit_split')]:
    source=(out/'generated'/source).read_text()
    source+='''
impl Kernel {
    pub fn build_phase(&mut self,r:&[u16],n:usize,p:u32)->(u32,u32) {
        let (_,m,b)=self.construct::<true,true>(r,n as u32,p,16);(m,b)
    }
    #[inline(never)]
    pub fn emit_phase(&self,n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        TARGET(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
    }
}
'''.replace('TARGET',target)
    (out/(slot+'.rs')).write_text(source)
main=(d/'main.rs').read_text().replace('include!("gate.rs");',f'include!("{out}/generated/gate.rs");')
(out/'main.rs').write_text(main)
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',out/'main.rs','--extern',f'mibz2={lib}','-o',out/'panel'],check=True)
