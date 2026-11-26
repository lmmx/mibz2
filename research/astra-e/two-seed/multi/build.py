from pathlib import Path
import subprocess
import sys

d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
lib=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,d.parent/'prepare.py',out],check=True)
p=(d.parent/'writer.rs').read_text()
for fn in ['fn place(', 'fn finish(']:
    assert p.count(fn)==1
    p=p.replace(fn,'pub(crate) '+fn)
(out/'writer.rs').write_text(p)
(out/'multi.rs').write_bytes((d/'writer.rs').read_bytes())
base=(d.parent/'main.rs').read_text()
base=base[:base.index('mod synthetic;')] + 'mod multi;\n' + base[base.index('#[repr(C)]'):base.index('const ARMS:')]
(out/'main.rs').write_text(base.replace('include!("../edge-budget/gate.rs");',f'include!("{out}/generated/gate.rs");')+(d/'driver.rs').read_text())
import os
env=dict(os.environ,EDGE_TABLE_SOURCE=str(out/'kernel.rs'))
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',out/'main.rs','--extern',f'mibz2={lib}','-o',out/'panel'],env=env,check=True)
