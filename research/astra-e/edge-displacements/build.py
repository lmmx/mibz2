"""Build a disposable decoder and the frozen edge constructor with a tape writer."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys
here = Path(__file__).resolve().parent
root = here.parents[2]
rev = 'badb2aec3624bd65606c429b2e6f79e9d9d0050f'
out=Path(sys.argv[1]).resolve()
frozen=out/'vendor'
frozen.mkdir(parents=True,exist_ok=True)
paths=['src','docs/journal/2026-11-18-open-return','research/astra-e/edge-budget',
       'research/astra-e/narrow-short','research/astra-e/narrow-endpoint',
       'research/astra-e/short-words','research/astra-e/lookahead']
p=subprocess.Popen(['git','-C',root,'archive',rev,*paths],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',frozen],stdin=p.stdout,check=True)
assert p.wait()==0
r=frozen/'research/astra-e'
subprocess.run([sys.executable,r/'edge-budget/prepare.py'],check=True)
kernel=r/'narrow-short/constructor_ahead.rs'
assert hashlib.sha256(kernel.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
with kernel.open('a') as f:
    f.write((here/'dump.inc.rs').read_text())
shutil.copy(r/'edge-budget/gate.rs',out/'gate.rs')
shutil.copy(here/'dump.rs',out/'dump.rs')
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',frozen/'src/lib.rs','-o',lib],check=True)
subprocess.run(flags+[out/'dump.rs','--extern',f'mibz2={lib}','-o',out/'dump'],check=True)
