from pathlib import Path
from hashlib import sha256
import argparse, subprocess, sys
here=Path(__file__).resolve().parent
p=argparse.ArgumentParser()
p.add_argument('out',type=Path);p.add_argument('deps',type=Path);p.add_argument('--sources-only',action='store_true')
a=p.parse_args();out=a.out.resolve();deps=a.deps.resolve()
out.mkdir(parents=True,exist_ok=True)
# The frozen repeated and event generators differ only by these module additions.
# Verify the upstream generated module before adding the three studied algorithms.
native=here.parents[3]/'native'
subprocess.run([sys.executable,'-B',native/'generate.py',out],check=True)
ctrl=out/'control.rs'
assert sha256(ctrl.read_bytes()).hexdigest()=='01663fb4fa08e977bf775b925565e7f71a1faf3f822f02d9122d1278fc7529ab'
root=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
pins=[('a34c81ba357b3186915113ebe5445a6e97cd7fcb',here.parents[1]/'grammar.rs'),
      ('df54544fedb5cacd522cc99bde6d7430ad0eb32b',here.parent/'event.rs')]
pieces=[subprocess.check_output(['git','-C',root,'show',f'{pin}:{p.relative_to(root)}'],text=True)
        for pin,p in pins]
ctrl.write_text(ctrl.read_text()+'\n'+'\n'.join(pieces+[(here/'ribbons.rs').read_text()]))
q=out/'main.rs';q.write_text(q.read_text().split('struct Origins {')[0]+(here/'main.rs').read_text())
if not a.sources_only:
    subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',q,'--extern',f'mibz2={deps}/libmibz2.rlib','-L',f'dependency={deps}','-o',out/'ribbons'],check=True)
