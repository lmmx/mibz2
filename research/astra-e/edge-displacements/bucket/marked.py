"""Regenerate the complete kernel and independently exercise marked periodic examples."""
from pathlib import Path
import subprocess
import sys

from native import apply

here=Path(__file__).resolve().parent
root=here.parents[3]
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
frozen=out/'source'
frozen.mkdir(exist_ok=True)
files=['src','research/astra-e/narrow-short','research/astra-e/narrow-endpoint','research/astra-e/short-words',
       'research/astra-e/edge-budget','research/astra-e/lookahead','docs/journal/2026-11-18-open-return']
pipe=subprocess.Popen(['git','-C',root,'archive','badb2aec3624bd65606c429b2e6f79e9d9d0050f',*files],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',frozen],stdin=pipe.stdout,check=True)
assert pipe.wait()==0
d=frozen/'research/astra-e'
subprocess.run([sys.executable,d/'edge-budget/prepare.py'],check=True)
base=d/'narrow-short/constructor_ahead.rs'
alternative=base.parent/'constructor_bucket.rs'
alternative.write_text(apply(base.read_text()))
helper=(d/'narrow-endpoint/check.rs').read_text()
oracle=helper[helper.index('fn reference('):helper.index('fn main()')]
# Same immutable corpus generator/controls as the upstream spelling test. Activate
# the cycle check on reduced residual footprints instead of spelling aliases.
sample=subprocess.check_output(['git','-C',root,'show',
    'd2702e2:research/astra-e/edge-spellings/byte_alias_marked.inc.rs'],text=True)
assert sample.count('if c.words<b.words {')==1
sample=sample.replace('if c.words<b.words {','if c.table_bytes<b.table_bytes {').replace('reduced word codes','used fewer residual bytes')
main=out/'main.rs'
main.write_text('#![allow(dead_code)]\npub use mibz2::{bwt,runs,Kernels};\n'+
                f'#[path="{base}"]mod old;\n#[path="{alternative}"]mod alias;\n'+oracle+sample)
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',
                      frozen/'src/lib.rs','-o',lib],check=True)
subprocess.run(flags+[main,'--extern',f'mibz2={lib}','-o',out/'check'],check=True)
subprocess.run([out/'check'],check=True)
