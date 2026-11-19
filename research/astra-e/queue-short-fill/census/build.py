"""Build a temporary copy of the i9-certified worker and observe its unchanged row queues."""
from hashlib import sha256
from pathlib import Path
import shutil, subprocess, sys
d=Path(__file__).resolve().parent
repo=d.parents[3]
out=Path(sys.argv[1]).resolve()
src=out/'frozen'
pin='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
src.mkdir(parents=True,exist_ok=True)
paths=['src','docs/journal/2026-11-18-open-return','research/astra-e/edge-budget',
       'research/astra-e/narrow-short','research/astra-e/narrow-endpoint','research/astra-e/short-words',
       'research/astra-e/lookahead']
p=subprocess.Popen(['git','-C',repo,'archive',pin,*paths],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',src],stdin=p.stdout,check=True)
assert p.wait()==0
r=src/'research/astra-e'
subprocess.run([sys.executable,r/'edge-budget/prepare.py'],check=True)
k=r/'narrow-short/constructor_ahead.rs'
assert sha256(k.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
with k.open('a') as f:
    f.write("""
pub fn kept(records:&[u16], mark:u32, mut f:impl FnMut(u8,u32,u32)) {
    each_run(records,mark,16,|c,x,w,keep| {if keep {f(c,x,w)}});
}
impl Kernel {
    pub fn queue(&self,a:usize)->&Vec<u32>{&self.queues[a]}
}
""")
shutil.copy(d/'main.rs',out/'main.rs')
shutil.copy(r/'edge-budget/gate.rs',out/'gate.rs')
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1']
lib=out/'libmibz2.rlib'
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',src/'src/lib.rs','-o',lib],check=True)
subprocess.run(flags+[out/'main.rs','--extern',f'mibz2={lib}','-o',out/'observe'],check=True)
