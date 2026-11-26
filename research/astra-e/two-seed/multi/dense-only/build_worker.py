"""Compute density once at original admission, then pass the same boolean to emission."""
from pathlib import Path
import subprocess
import sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else here/'target'
subprocess.run([sys.executable,here.parent/'build_worker.py',out],check=True)
g=out/'generated'
s=(g/'scalar-four.rs').read_text()
s+='''
impl Kernel {
    pub(super) fn run_selected(&mut self,records:&[u16],n:usize,p:u32,dense:bool,out:&mut Vec<u8>) {
        assert!(n>0 && n<=900_000 && (p as usize)<n);
        let (_,mark,bits)=self.construct::<true,true>(records,n as u32,p,16);
        if dense {
            twoseed::emit_split(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        } else {
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,out);
        }
    }
}
'''
(g/'select.rs').write_text(s)
adapter=(g/'adapter.inc.rs').read_text()
cut='        if r.records().len() * 20 >= n * 3 && (n < 65_536 || !fits(r.records(), p)) {'
assert adapter.count(cut)==1
adapter=adapter.replace(cut,'        let dense=r.records().len()*20>=n*3;\n        if dense && (n<65_536 || !fits(r.records(),p)) {')
cut='self.open.run::<true, true, false>(r.records(), n, p, 16, out);'
assert adapter.count(cut)==1
adapter=adapter.replace(cut,'self.open.run_selected(r.records(),n,p,dense,out);')
(g/'select-adapter.rs').write_text(adapter)
