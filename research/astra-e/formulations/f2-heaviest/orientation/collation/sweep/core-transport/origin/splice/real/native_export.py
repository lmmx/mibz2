"""Capture the true fixed-literal first object and the complete native marked output."""
from pathlib import Path
import struct
import subprocess
import sys
from sources import read

here=Path(__file__).resolve().parent
root=next(p for p in here.parents if (p/'Cargo.toml').exists())
native=here.parents[1]/'direct-native'
rel=native.relative_to(root)
pin='8ec1f2efb6e84b5a1626d92aa42039af79934e8a'
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
lib=Path(sys.argv[2]).resolve()
# The generator performs its own pinned checks on all dynamically generated modules.
subprocess.run(['git','-C',root,'diff','--quiet',pin,'--',*(rel/p for p in ('generate.py','main.rs','selector.rs','dispatch.rs'))],check=True)
subprocess.run([sys.executable,'-B',native/'generate.py',out],check=True)
s=(out/'main.rs').read_text()
assert s.count('fn main()')==1
(out/'snapshot.rs').write_text(s.replace('fn main()','fn other_modes()')+(here/'native_export.inc.rs').read_text())
subprocess.run(['rustc','--edition=2021','-A','unexpected_cfgs','-A','unused_imports','-A','dead_code',
    '-C','opt-level=3','-C','codegen-units=1','-C','debug-assertions=yes','--cfg','clipped_profile',
    '--cfg','shift_counts',out/'snapshot.rs','--extern',f'mibz2={lib}','-L',f'dependency={lib.parent}',
    '-o',out/'take'],check=True)
with (out/'ten.bin').open('wb') as f:
    for i,runs,p,_ in read(here/'sources.xz'):
        caps=[]
        for b,w in runs:
            while w:
                q=min(w,127);w-=q;caps.append(b<<8|q)
        f.write(struct.pack('<4I',i,sum(q for _,q in runs),p,len(caps)))
        f.write(struct.pack('<'+'H'*len(caps),*caps))
subprocess.run([out/'take',out/'ten.bin',out],check=True)
