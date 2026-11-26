"""Generate the stock-like bitmap MTF and frozen combined constructor with a flat run log."""
from pathlib import Path
from tempfile import TemporaryDirectory
import subprocess,sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
with TemporaryDirectory() as directory:
    temp=Path(directory)
    subprocess.run([sys.executable,str(here.parent/"edge/prepare.py"),directory],check=True)
    base=(temp/"pre.rs").read_text()
    (out/"gate.rs").write_bytes((temp/"gate.rs").read_bytes())
base=base.replace('use crate::front::Bits;','use crate::Bits;').replace('include!("cap_count.rs");','')
a=base.index('pub(super) unsafe fn front_avx2')
b=base.index('    use std::arch::x86_64::*;',a)
base=base[:a]+'''pub(super) unsafe fn pack(sp:*mut u16,m:usize,dst:*mut u16,eob:u16,active:&[u8;256],max_len:usize,f:&mut Bits,written:&mut usize)->Option<(usize,usize)>{
'''+base[b:]
base=base.replace('    let m = b.syms.len();\n    let sp = b.syms.as_ptr();','')
base=base.replace('b.syms[ci * 32..]','std::slice::from_raw_parts(sp,m)[ci * 32..]').replace('b.eob','eob').replace('b.max_len','max_len')
base=base.replace('&b.seq_to_unseq[..','&active[..').replace('&b.seq_to_unseq;','active;')
a=base.index('    let caps=if LAZY');base=base[:a]+base[base.index('\n',a):]
a=base.index('    let mut row_count');b=base.index("    // the literals' bits are walked",a)
base=base[:a]+(here/"front.inc.rs").read_text()+base[b:]
base=base.replace('then_some((row_count,caps,end))','then_some((row_count,*written+cap_difference))')
assert all(name not in base for name in ['b.syms','b.max_len','b.eob','b.seq_to_unseq'])
(out/"front.rs").write_text(base)
with TemporaryDirectory() as directory:
    temp=Path(directory)
    subprocess.run([sys.executable,str(here.parent/"cap-batch/prepare.py"),directory],check=True)
    kernel=(temp/"control.rs").read_text()
    (out/"control.rs").write_text(kernel)
    (out/"adapter.inc.rs").write_bytes((temp/"adapter.inc.rs").read_bytes())
a=kernel.index('        let rec = records[i];')
b=kernel.index('        let end = pos + width;',a)
kernel=kernel[:a]+'        let (c,width)=crate::log::take(records,&mut i);\n'+kernel[b:]
(out/"packed.rs").write_text(kernel)
(out/"log.rs").write_bytes((here/"log.rs").read_bytes())
(out/"main.rs").write_bytes((here/"native.rs").read_bytes())

(out/"errors.rs").write_bytes((here/"errors.rs").read_bytes())
