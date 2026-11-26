"""Extend the frozen scalar-plus-four worker through the existing expanded-output hook."""
from pathlib import Path
import subprocess
import sys
from generate import generate

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else here/'target'
subprocess.run([sys.executable,here.parent/'build_worker.py',out],check=True)
base=(out/'generated/scalar-four.rs').read_text()
marker='\nmod twoseed {\n'
assert base.count(marker)==1 and base.rstrip().endswith('}')
base=base.rstrip()[:-1]
module=generate().replace('use crate::{original,Table,writer};','use super::Table;')
module=module.replace('original::Kernel::stock(t,out);','super::super::emit(t.edges,t.extra,t.spelling,t.offsets,t.n,t.mark,t.bits,out);')
module=module.replace('writer::place(', 'super::place(').replace('writer::finish(', 'super::finish(')
module=module.replace('pub fn expand<', 'pub(super) fn expand<').replace('crate::slices::','super::slices::')
slices=(here/'slices.rs').read_text()
base+= '''
    mod slices {
''' + slices + '''
    }
    mod expanded {
''' + module + '''
    }
    pub(super) fn final_output<const SLICED:bool>(edges:&[u32],extra:&[u32],spelling:&[u8],
        offsets:&[u32],n:usize,mark:u32,bits:u32,raw:&mut Vec<u8>,dst:&mut Vec<u8>,k:Kernels
    )->Result<(),mibz2::ErrorKind> {
        let t=Table{edges,extra,spelling,offsets,n,mark,bits};
        expanded::expand::<4,SLICED,false>(&t,raw,dst,k,&mut slices::Audit::default())
    }
}
impl Kernel {
    pub(super) fn final_output<const SLICED:bool>(&mut self,r:&Runs,n:usize,p:u32,
        raw:&mut Vec<u8>,dst:&mut Vec<u8>,k:Kernels
    )->Result<(),mibz2::ErrorKind> {
        let (_,mark,bits)=self.construct::<true,true>(r.records(),n as u32,p,16);
        twoseed::final_output::<SLICED>(&self.edges,&self.extra,&self.spelling,&self.offsets,
                                      n,mark,bits,raw,dst,k)
    }
}
'''
base=base.replace('raw:&mut Vec<u8>,dst:&mut Vec<u8>,k:Kernels\n    )->Result<(),mibz2::ErrorKind> {\n        let t=Table', 'raw:&mut Vec<u8>,dst:&mut Vec<u8>,k:mibz2::Kernels\n    )->Result<(),mibz2::ErrorKind> {\n        let t=Table')
(out/'generated/expanded.rs').write_text(base)
adapter=(out/'generated/adapter.inc.rs').read_text()
needle='    lazy: LazyCells,'
assert adapter.count(needle)==1
adapter=adapter.replace(needle,needle+'\n    raw: Vec<u8>,')
begin=adapter.index('    fn invert_runs(')
adapter=adapter[:begin]+'''    fn invert_runs(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>,k:Kernels)->bool {
        // decode_block calls this method when the preceding expanded-output hook declined.
        self.lazy.invert_runs(r,n,p,out,k)
    }
    fn expand_runs(&mut self,r:&Runs,n:usize,p:u32,out:&mut Vec<u8>,k:Kernels)
        ->Option<Result<(),mibz2::ErrorKind>> {
        if r.records().len()*20>=n*3 && (n<65_536 || !fits(r.records(),p)) {return None;}
        Some(self.open.final_output::<SLICED>(r,n,p,&mut self.raw,out,k))
    }
}
'''
(out/'generated/expanded-adapter.rs').write_text(adapter)
