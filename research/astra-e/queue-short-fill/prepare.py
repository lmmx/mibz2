"""Rebuild each isolated entry using the two immutable predecessor source trees."""
from pathlib import Path
import hashlib
import re
import shutil
import subprocess
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
EDGE='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
BOTH='50b8ecb8946a498601e3fd894ed84a2687df6cc8'
def archive(dst,rev,paths):
    dst.mkdir(parents=True,exist_ok=True)
    p=subprocess.Popen(['git','-C',ROOT,'archive',rev,*paths],stdout=subprocess.PIPE)
    subprocess.run(['tar','-x','-C',dst],stdin=p.stdout,check=True)
    assert p.wait()==0

def transform(source,mode):
    old='                    self.queues[a].extend(m..m + w);'
    assert source.count(old)==1
    if mode==0:return source
    if mode==1:
        return source.replace(old,'''                    let queue=&mut self.queues[a];
                    if w == 1 {
                        queue.push(m);
                    } else {
                        queue.extend(m..m + w);
                    }''')
    assert mode==4
    return source.replace(old,'''                    let queue=&mut self.queues[a];
                    match w {
                        1=>queue.push(m),
                        2=>queue.extend_from_slice(&[m,m+1]),
                        3=>queue.extend_from_slice(&[m,m+1,m+2]),
                        4=>queue.extend_from_slice(&[m,m+1,m+2,m+3]),
                        _=>queue.extend(m..m+w),
                    }''')

def prepare(dest, meter=False):
    dest=Path(dest).resolve()
    frozen=dest/'frozen'
    base=Path('research/astra-e')
    archive(frozen,EDGE,['docs/journal/2026-11-18-open-return',*(base/x for x in
        ['edge-budget','narrow-short','narrow-endpoint','short-words','lookahead'])])
    archive(frozen,BOTH,[base/'front-fusion/cap-counter'])
    subprocess.run([sys.executable,'-B',frozen/base/'edge-budget/prepare.py'],check=True)
    (dest/'generated').mkdir(exist_ok=True)
    for pred,entry in [('edge',frozen/base/'narrow-short/constructor_ahead.rs'),
                       ('both',frozen/base/'front-fusion/cap-counter/kernel.rs')]:
        original=entry.read_text()
        if pred=='edge':assert hashlib.sha256(original.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
        # Generated source includes remain pinned to the isolated extracted checkout.
        raw=re.sub(r'#\[path = "([^"]+)"\]',lambda m:f'#[path = "{(entry.parent/m[1]).resolve()}"]',original)
        for mode in [0,1,4]:
            s=transform(raw,mode)
            if meter:
                first='        let mut last = None;'
                last='        assert!(m > 0 && all.iter()'
                assert s.count(first)==s.count(last)==1
                s=s.replace(first,'        let started = crate::cpu();\n'+first)
                s=s.replace(last,'        crate::SCAN.with(|p|p.set(crate::cpu()-started));\n'+last)
            (dest/'generated'/f'{pred}{mode}.rs').write_text(s)
    shutil.copy(frozen/base/'edge-budget/adapter.inc.rs',dest/'generated/adapter.inc.rs')
    shutil.copy(frozen/base/'edge-budget/gate.rs',dest/'generated/gate.rs')
    return dest
if __name__=='__main__':prepare(Path(sys.argv[1]) if len(sys.argv)>1 else HERE/'target')
