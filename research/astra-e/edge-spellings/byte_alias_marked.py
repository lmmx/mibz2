"""Compile an independent marked periodic test against the exact frozen edge source."""
import hashlib
from pathlib import Path
import subprocess
import sys

from byte_alias import apply

here = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=True)
frozen = out/'source'
frozen.mkdir(exist_ok=True)
pipe = subprocess.Popen(['git','-C',here.parents[2],'archive','badb2aec3624bd65606c429b2e6f79e9d9d0050f',
                         'src','research/astra-e','docs/journal/2026-11-18-open-return'],
                        stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',frozen],stdin=pipe.stdout,check=True)
assert pipe.wait()==0
d=frozen/'research/astra-e'
subprocess.run([sys.executable,d/'edge-budget/prepare.py'],check=True)
old=d/'narrow-short/constructor_ahead.rs'
assert hashlib.sha256(old.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
alias=old.parent/'constructor_alias.rs'
alias.write_text(apply(old.read_text()))
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',
                      frozen/'src/lib.rs','-o',lib],check=True)
# Reuse the frozen independent stable-LF oracle and the exact production cap encoding.
checker=(d/'narrow-endpoint/check.rs').read_text()
preamble=checker[checker.index('fn reference('):checker.index('fn main()')]
main=out/'main.rs'
main.write_text('#![allow(dead_code)]\npub use mibz2::{bwt,runs,Kernels};\n'
                +f'#[path="{old}"] mod old;\n#[path="{alias}"] mod alias;\n'
                +preamble+(here/'byte_alias_marked.inc.rs').read_text())
subprocess.run(flags+[main,'--extern',f'mibz2={lib}','-o',out/'check'],check=True)
subprocess.run([out/'check'],check=True)
