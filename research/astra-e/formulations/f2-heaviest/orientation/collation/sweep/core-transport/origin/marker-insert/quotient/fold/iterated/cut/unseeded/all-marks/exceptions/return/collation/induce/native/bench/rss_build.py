"""Add one original cap-input constructor to an isolated RSS-only copy of the frozen lib."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import os
HERE=Path(__file__).resolve().parent
REF,OUT=map(lambda a:Path(a).resolve(),sys.argv[1:])
manifest=json.loads((REF/'build.json').read_text())
rust=Path(os.environ.get('RUSTC','/workspace/.mibz2-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc'))
assert subprocess.check_output([rust,'-Vv'],text=True).strip()==manifest['rustc']
first=(REF/'main.rs').read_text().split('pub use mibz2')[0]
source=Path(first.split('"')[1]).parents[3]
assert subprocess.check_output(['git','-C',source,'rev-parse','HEAD'],text=True).strip()==manifest['donor']
root=OUT/'rss-source';root.mkdir(exist_ok=True)
subprocess.run(['git','-C',source,'archive','-o',root/'src.tar',manifest['donor'],'src'],check=True)
subprocess.run(['tar','xf',root/'src.tar','-C',root],check=True)
(root/'src.tar').unlink()
p=root/'src/runs.rs'
with p.open('ab') as f:f.write((HERE/'cap_input.rs').read_bytes())
sha=lambda v:hashlib.sha256(v.read_bytes()).hexdigest()
rlib=root/'libmibz2.rlib'
flags=manifest['rust_flags']
subprocess.run([rust,*flags,'--cfg','feature="moonshot"','--crate-name','mibz2','--crate-type','rlib',root/'src/lib.rs','-o',rlib],check=True)
p=root/'main.rs';p.write_text((REF/'main.rs').read_text().split('fn main() {')[0]+(HERE/'cap_rss.rs').read_text())
subprocess.run([rust,'-A','unused',*flags,p,'--extern',f'mibz2={rlib}','-L',f'native={REF}','-l','dylib=grammar','-o',OUT/'cap-rss'],check=True)
(OUT/'rss-build.json').write_text(json.dumps(dict(original_src_tree=manifest['src_tree'],cap_input=sha(HERE/'cap_input.rs'),
    generated_runs=sha(root/'src/runs.rs'),cap_main=sha(HERE/'cap_rss.rs'),generated_main=sha(p),rlib=sha(rlib),
    exe=sha(OUT/'cap-rss'),rust_flags=flags,rustc=manifest['rustc']),indent=2)+'\n')
