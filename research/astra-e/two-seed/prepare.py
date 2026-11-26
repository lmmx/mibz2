"""Make an isolated historical edge table constructor, retaining its original emitter."""
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
subprocess.run([sys.executable, here.parent/'queue-short-fill/prepare.py', out], check=True)
s = (out/'generated/edge0.rs').read_text().replace('//! First return','// First return',1)
s += """
impl Kernel {
    pub fn make(&mut self, r: &[u16], n: usize, p: u32) -> crate::Table {
        let (_,mark,bits)=self.construct::<true,true>(r,n as u32,p,16);
        crate::Table {edges:self.edges.clone(),extra:self.extra.clone(),spelling:self.spelling.clone(),
            offsets:self.offsets.clone(),n,mark,bits}
    }
    pub fn stock(t: &crate::Table, out: &mut Vec<u8>) {
        emit(&t.edges,&t.extra,&t.spelling,&t.offsets,t.n,t.mark,t.bits,out);
    }
}
"""
(out/'kernel.rs').write_text(s)
