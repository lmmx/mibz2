from pathlib import Path
import os
import subprocess
import sys
from generate import generate

d=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
lib=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,d.parents[1]/'prepare.py',out],check=True)
p=(d.parents[1]/'writer.rs').read_text().replace('fn place(', 'pub(crate) fn place(').replace('fn finish(', 'pub(crate) fn finish(')
(out/'writer.rs').write_text(p)
(out/'multi.rs').write_bytes((d.parent/'writer.rs').read_bytes())
(out/'slices.rs').write_bytes((d/'slices.rs').read_bytes())
(out/'expanded.rs').write_text(generate())
base=(d.parents[1]/'main.rs').read_text()
base=base[:base.index('mod synthetic;')] + 'mod multi;mod slices;mod expanded;\n' + base[base.index('#[repr(C)]'):base.index('const ARMS:')]
(out/'main.rs').write_text(base.replace('include!("../edge-budget/gate.rs");',f'include!("{out}/generated/gate.rs");')+(d/'driver.rs').read_text())
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',out/'main.rs','--extern',f'mibz2={lib}','-o',out/'panel'],env=dict(os.environ,EDGE_TABLE_SOURCE=str(out/'kernel.rs')),check=True)
