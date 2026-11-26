from pathlib import Path
from hashlib import sha256
import sys, subprocess
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
deps=Path(sys.argv[2]).resolve()
subprocess.run([sys.executable,'-B',here.parent/'build.py',out,deps],check=True)
control=out/'control.rs'
control.write_text(control.read_text()+'\n'+(here/'event.rs').read_text())
main=out/'main.rs'
s=main.read_text().split('mod joint {')[0]
main.write_text(s+(here/'main.rs').read_text())
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports',
    '-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',main,
    '--extern',f'mibz2={deps}/libmibz2.rlib','-L',f'dependency={deps}','-o',out/'event'],check=True)
