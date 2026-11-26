"""Build the frozen width-16 control and both charged successor layouts, with matching clocks."""
from pathlib import Path
import importlib.util
import hashlib
import shutil
import subprocess
import sys
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
for name in ['bucket','local']:
    folder=out/name
    shutil.copytree(stock,folder,dirs_exist_ok=True)
    p=importlib.util.spec_from_file_location(name,HERE/name/'native.py')
    module=importlib.util.module_from_spec(p)
    p.loader.exec_module(module)
    file=folder/k
    src=module.apply(file.read_text())
    assert src.count('// DISPLACEMENT_EMIT_BOUNDARY')==1
    src=src.replace('// DISPLACEMENT_EMIT_BOUNDARY', 'crate::boundary();')
    if name=='bucket':
        before='        if self.bucket.used {\n            crate::boundary();'
        assert src.count(before)==1
        src=src.replace(before,'        if !self.bucket.used {crate::boundary();}\n'+before)
    file.write_text(src)
original=(stock/k).read_text()
anchor='        if SEGMENTS {\n            self.segments.emit('
assert original.count(anchor)==1
original=original.replace(anchor,'        crate::boundary();\n'+anchor)
original+='''
impl Kernel {
    pub fn displacement_entry(&self,row:usize,bits:u32)->(u32,u32) {
        let edge=self.edges[row];
        (edge&((1u32<<bits)-1),if self.extra.is_empty(){edge>>bits}else{self.extra[row]})
    }
    pub fn displacement_storage(&self)->[usize;4] {
        [4*(self.edges.len()+self.extra.len()),4*(self.edges.capacity()+self.extra.capacity()),
         self.spelling.len()+4*self.offsets.len(),self.spelling.capacity()+4*self.offsets.capacity()]
    }
}
'''
(stock/k).write_text(original)
code=''
for name in ['stock','local','bucket']:
    code+=f'#[path="{name}/{k}"] mod {name};\n'
orig=(stock/base/'narrow-endpoint/check.rs').read_text()
reference=orig[orig.index('fn reference'):orig.index('fn main()')]
code+=reference+(stock/base/'edge-budget/gate.rs').read_text()
shutil.copy(stock/base/'edge-budget/adapter.inc.rs',out/'adapter.inc.rs')
driver=out/'price.rs'
driver.write_text('#![allow(dead_code)]\n'+code+(HERE/'price.rs').read_text())
subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1',driver,
                '--extern',f'mibz2={out/"libmibz2.rlib"}','-o',out/'price'],check=True)
