"""Build all six complete worker instantiations, sharing source except for the frozen transformations."""
from pathlib import Path
from hashlib import sha256
import argparse,re,subprocess,sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
EDGE='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
BOTH='50b8ecb8946a498601e3fd894ed84a2687df6cc8'
R='research/astra-e'
J='docs/journal/2026-11-18-open-return'
out=Path(sys.argv[1]).resolve()
tree=out/'frozen';tree.mkdir(parents=True,exist_ok=True)
dirs=['src',J,*(f'{R}/{s}' for s in ['narrow-short','narrow-endpoint','short-words','lookahead','edge-budget'])]
p=subprocess.Popen(['git','-C',ROOT,'archive',EDGE,*dirs],stdout=subprocess.PIPE)
subprocess.run(['tar','-x','-C',tree],stdin=p.stdout,check=True);assert p.wait()==0
r=tree/R
subprocess.run([sys.executable,r/'edge-budget/prepare.py'],check=True)
control=r/'narrow-short/constructor_ahead.rs'
assert sha256(control.read_bytes()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
counter=r/'front-fusion/cap-counter'
counter.mkdir(parents=True,exist_ok=True)
for s in ('kernel.rs','narrow.rs','both.rs'):
    (counter/s).write_bytes(subprocess.check_output(['git','-C',ROOT,'show',f'{BOTH}:{R}/front-fusion/cap-counter/{s}']))
flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=line-tables-only']
subprocess.run(flags+['--crate-type','rlib','--crate-name','mibz2','--cfg','feature="moonshot"',tree/'src/lib.rs','-o',out/'libmibz2.rlib'],check=True)
arms=[]
for source in (control,counter/'kernel.rs'):
    both=source.parent==counter
    s=source.read_text()
    # Preserve all dependencies exactly. Only the copied site's filename changes.
    s=re.sub(r'(#\[path = ")([^"]+)("\])',lambda x:x[1]+str((source.parent/x[2]).resolve())+x[3],s)
    for name in ['range','single','four']:
        arm=('both_' if both else 'edge_')+name
        dest=out/arm;dest.mkdir(exist_ok=True)
        needle='self.queues[a].extend(m..m + w);'
        one='''let q = &mut self.queues[a];
                    if w == 1 {
                        q.push(m);
                    } else {
                        q.extend(m..m + w);
                    }'''
        four='''let q = &mut self.queues[a];
                    match w {
                        1 => q.push(m),
                        2 => q.extend_from_slice(&[m, m + 1]),
                        3 => q.extend_from_slice(&[m, m + 1, m + 2]),
                        4 => q.extend_from_slice(&[m, m + 1, m + 2, m + 3]),
                        _ => q.extend(m..m + w),
                    }'''
        assert s.count(needle)==1
        (dest/'kernel.rs').write_text(s.replace(needle,{'range':needle,'single':one,'four':four}[name]))
        constants='const SCALAR_CAP:bool=true; const RECOVER:bool=true;\n' if both else ''
        adapter=(r/'edge-budget/adapter.inc.rs').read_text()
        (dest/'mod.rs').write_text(constants+'mod kernel;\n'+(r/'edge-budget/gate.rs').read_text()+adapter)
        arms.append(arm)
body='''#![allow(dead_code)]
pub use mibz2::{bwt,runs,Kernels};
use std::io::Read;
fn decode<I:bwt::InverseBwt+'static>(bytes:Vec<u8>) -> u64 {
    let mut reader=mibz2::BzReader::parallel_workers_with::<I>(std::io::Cursor::new(bytes),1,4,Kernels::detect());
    std::io::copy(&mut reader,&mut std::io::sink()).unwrap()
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let data=std::fs::read(&args[2]).unwrap();
    let n=match args[1].as_str() {
'''
body+='\n'.join(f'        "{a}"=>decode::<{a}::Candidate>(data),' for a in arms)
body+='\n        _=>panic!(),\n    };println!("{n}");\n}\n'
body+=''.join(f'mod {a};\n' for a in arms)
(out/'main.rs').write_text(body)
subprocess.run(flags+['-A','unused_imports','-C','lto=fat','--crate-name','asm_worker',out/'main.rs',
                      '--extern',f'mibz2={out}/libmibz2.rlib','-o',out/'worker'],check=True)
