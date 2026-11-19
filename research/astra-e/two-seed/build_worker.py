"""Keep the historical constructor and adapter. Change only their marked emitter entry."""
from pathlib import Path
import subprocess
import sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else here/'target'
subprocess.run([sys.executable,here.parent/'queue-short-fill/prepare.py',out],check=True)
s=(out/'generated/edge0.rs').read_text()
old='''        } else {
            emit(
                &self.edges,
                &self.extra,
                &self.spelling,
                &self.offsets,
                n,
                mark,
                bits,
                out,
            );
        }
        stats'''
assert s.count(old)==1
s=s.replace(old,old.replace('            emit(', '            twoseed::emit_split('))
writer=(here/'writer.rs').read_text()
assert writer.count('use crate::{original, Table};')==1
writer=writer.replace('use crate::{original, Table};','')
assert writer.count('original::Kernel::stock(t,out)')==2
writer=writer.replace('original::Kernel::stock(t,out)', 'super::emit(t.edges,t.extra,t.spelling,t.offsets,t.n,t.mark,t.bits,out)')
s+='''
mod twoseed {
    struct Table<'a> {
        edges: &'a [u32], extra: &'a [u32], spelling: &'a [u8], offsets: &'a [u32],
        n: usize, mark: u32, bits: u32,
    }
    pub(super) fn emit_split(edges:&[u32],extra:&[u32],spelling:&[u8],offsets:&[u32],
        n:usize,mark:u32,bits:u32,out:&mut Vec<u8>) {
        let t=Table{edges,extra,spelling,offsets,n,mark,bits};
        split::<false,true>(&t,out);
    }
''' + writer.replace('pub fn ', 'fn ') + '\n}\n'
(out/'generated/two-seed.rs').write_text(s)
