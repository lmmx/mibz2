"""Regenerate the i9 winner into a disposable checkout and log nonliteral band owners."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
REV = 'badb2aec3624bd65606c429b2e6f79e9d9d0050f'
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
frozen = out/'srcroot'
frozen.mkdir(exist_ok=True)
pack = subprocess.Popen(['git','-C',ROOT,'archive',REV,'src','research/astra-e/narrow-short',
    'research/astra-e/narrow-endpoint','research/astra-e/short-words','research/astra-e/lookahead',
    'docs/journal/2026-11-18-open-return','research/astra-e/edge-budget'],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',frozen],stdin=pack.stdout,check=True)
assert pack.wait()==0
research=frozen/'research/astra-e'
subprocess.run([sys.executable,research/'edge-budget/prepare.py'],check=True)
source=research/'narrow-short/constructor_ahead.rs'
assert hashlib.sha256(source.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
def replace(path,old,new):
    s=path.read_text()
    assert s.count(old)==1
    path.write_text(s.replace(old,new))

replace(source,'    spelling: Vec<u8>,','    handles: Vec<(u32,u32)>,\n    spelling: Vec<u8>,')
replace(source,'        self.spelling.clear();','        self.handles.clear();\n        self.spelling.clear();')
for ep in [research/'narrow-short/narrow.rs',research/'short-words/endpoint.rs']:
    replace(ep,'e.append(node.word, &mut self.spelling);','self.handles.push((node.word,width));\n                    e.append(node.word, &mut self.spelling);')
with source.open('a') as f:
    f.write((HERE/'dump.inc.rs').read_text())
shutil.copy(research/'edge-budget/gate.rs',out/'gate.rs')
shutil.copy(HERE/'main.rs',out/'main.rs')
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',
                      frozen/'src/lib.rs','-o',lib],check=True)
subprocess.run(flags+[out/'main.rs','--extern',f'mibz2={lib}','-o',out/'dump'],check=True)
