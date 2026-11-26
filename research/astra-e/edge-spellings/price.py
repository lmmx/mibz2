"""Build baseline and both exact reuse candidates with frozen endpoint dependencies."""
from pathlib import Path
import hashlib
import shutil
import subprocess
import sys
import byte_alias
import handle_alias

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
REV='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
out=Path(sys.argv[1]).resolve()
out.mkdir(parents=True,exist_ok=True)
stock=out/'stock'
stock.mkdir(exist_ok=True)
paths=['docs/journal/2026-11-18-open-return','research/astra-e/narrow-short','research/astra-e/narrow-endpoint',
       'research/astra-e/short-words','research/astra-e/lookahead','research/astra-e/edge-budget']
pack=subprocess.Popen(['git','-C',ROOT,'archive',REV,*paths],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',stock],stdin=pack.stdout,check=True)
assert pack.wait()==0
base=Path('research/astra-e')
k=base/'narrow-short/constructor_ahead.rs'
subprocess.run([sys.executable,stock/base/'edge-budget/prepare.py'],check=True)
assert hashlib.sha256((stock/k).read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
for v in ['handle','byte']:
    shutil.copytree(stock,out/v,dirs_exist_ok=True)
for ep in [out/'handle'/base/'narrow-short/narrow.rs',out/'handle'/base/'short-words/endpoint.rs']:
    ep.write_text(handle_alias.apply(ep.read_text()))
file=out/'byte'/k
file.write_text(byte_alias.apply(file.read_text()))
code=''
for name in ['stock','handle','byte']:
    file=out/name/k
    with file.open('a') as f:
        f.write('''
impl Kernel {
    pub fn measured_storage(&self)->[usize;4] {
        [self.spelling.len(), self.offsets.len()*4, (self.edges.len()+self.extra.len())*4,
         self.spelling.capacity()+4*(self.offsets.capacity()+self.edges.capacity()+self.extra.capacity())]
    }
}
''')
    code+=f'#[path="{name}/{k}"] mod {name};\n'
orig=(stock/base/'narrow-endpoint/check.rs').read_text()
reference=orig[orig.index('fn reference'):orig.index('fn main()')]
code+=reference+(stock/base/'edge-budget/gate.rs').read_text()
driver=out/'price.rs'
driver.write_text('#![allow(dead_code)]\n'+code+(HERE/'price.rs').read_text())
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1',driver,
                '--extern',f'mibz2={out/"libmibz2.rlib"}','-o',out/'price'],check=True)
