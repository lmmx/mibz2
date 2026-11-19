"""Frozen six-arm original-input donor plus independent stable-Psi oracle and cap-only HWM."""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import subprocess as sp
import sys

HERE=Path(__file__).resolve().parent
ROOT=Path(sp.check_output(['git','-C',HERE,'rev-parse','--show-toplevel'],text=True).strip())
DONOR=next(p/'powers/xor/capped' for p in HERE.parents if (p/'powers/xor/capped/receipt.json').is_file())
CORPUS=ROOT/'corpora/wikidata.bz2'
OLD=['stream','streamgaps','dense','scalar','lazy','opend']
NEW=['prefix','thin4','raref16','scalar','lazy','opend']
PHASE=['caps_ns','selector_ns','induce_ns','map_ns','emit_ns']
SHAPE=['kept','records','joins','lap','selected_spans','vector_bytes','live_with_result']

def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def read(f):return list(csv.DictReader(f.open()))
def dump(f,j):f.write_text(json.dumps(j,indent=2)+'\n')
def write(f,rows):
    with f.open('w') as out:
        w=csv.DictWriter(out,fieldnames=list(dict.fromkeys(k for r in rows for k in r)),lineterminator='\n')
        w.writeheader();w.writerows(rows)
def check_caps(d,inputs):
    for v in inputs:
        b=v['block'];assert sha(d/f'block-{b}.raw')==v['raw_sha256'] and sha(d/f'block-{b}.caps')==v['caps_le_sha256']
def main():
    arg=argparse.ArgumentParser(description=__doc__)
    arg.add_argument('donor',type=Path);arg.add_argument('work',type=Path)
    arg.add_argument('action',choices=['build','gate','price','rss'])
    opt=arg.parse_args()
    ref,d=opt.donor.resolve(),opt.work.resolve();d.mkdir(parents=True,exist_ok=True)
    receipt=json.loads((DONOR/'receipt.json').read_text());inputs=receipt['inputs']
    original=json.loads((DONOR/'build.json').read_text());actual=json.loads((ref/'build.json').read_text())
    for k in ('donor','source_sha256','src_tree','rlib_sha256','rust_flags','rustc','exe_sha256'):assert actual[k]==original[k]
    assert sha(ref/'panel')==original['exe_sha256'] and sha(CORPUS)==receipt['corpus_sha256']
    so=d/'libgrammar.so';env=dict(os.environ,LD_LIBRARY_PATH=str(d))
    env.pop('SECTION_STAMPS',None)
    if opt.action=='build':
        flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-fPIC','-shared']
        sp.run(['g++',*flags,HERE/'bridge.cpp','-o',so],check=True)
        sp.run(['g++','-std=c++20','-O2','-Wall','-Wextra',HERE/'dense.cpp','-o',d/'dense'],check=True)
        deps=sp.check_output(['g++','-std=c++20','-MM',HERE/'bridge.cpp'],text=True).replace('\\\n','').split()[1:]
        dump(d/'build.json',dict(so=sha(so),dense=sha(d/'dense'),bridge_deps={str(Path(x).resolve().relative_to(ROOT)):sha(Path(x)) for x in deps},
             controls=actual,corpus_sha256=sha(CORPUS),flags=flags,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0]))
        return
    build=json.loads((d/'build.json').read_text());assert sha(so)==build['so'] and sha(d/'dense')==build['dense']
    assert f'libgrammar.so => {so}' in sp.check_output(['ldd',ref/'panel'],text=True,env=env)
    if opt.action in ('gate','price'):
        if opt.action=='price':
            assert json.loads((d/'gate.json').read_text())['so']==sha(so)
            assert not (d/'paired.csv').exists()
        stamp=d/('gate-stamps.csv' if opt.action=='gate' else 'stamps.csv')
        rounds=0 if opt.action=='gate' else 24
        sp.run([ref/'panel',CORPUS,d,str(rounds)],env=dict(env,SECTION_STAMPS=str(stamp)),check=True)
        check_caps(d,inputs)
        bykey={(v['rows'],v['mark'],v['records']):v['block'] for v in inputs};assert len(bykey)==10
        stamps=read(stamp);assert len(stamps)==30*(rounds+1)
        for r in stamps:
            r['block']=str(bykey[tuple(int(r[k])for k in ('n','primary','input'))])
            assert int(r['lap'])==int(r['n']) and int(r['kept'])>0
        assert len({(r['arm'],r['block'])for r in stamps[:30]})==30
        if opt.action=='gate':
            dense=[]
            for v in inputs:
                b=v['block'];rows,lap=map(int,sp.check_output([d/'dense',d/f'block-{b}.caps',d/f'block-{b}.raw',str(v['mark'])],text=True).strip().split(','))
                assert rows==lap==v['rows'];dense.append(dict(block=b,rows=rows,lap=lap))
            dump(d/'gate.json',dict(so=sha(so),donor_checks=60,dense=dense,inputs=inputs))
        else:
            times=read(d/'times.csv');assert len(times)==1440
            src=iter(stamps[30:])
            for r in times:
                a=OLD.index(r['arm']);r['arm']=NEW[a]
                if a>=3:continue
                s=next(src);assert int(s['arm'])==a and s['block']==r['block']
                assert sum(int(s[k]) for k in PHASE)<=int(r['thread_ns'])
                r.update({k:s[k] for k in PHASE+SHAPE})
            assert not list(src)
            write(d/'paired.csv',times)
            dump(d/'runtime.json',dict(cpu=sorted(os.sched_getaffinity(0)),nice=os.getpriority(os.PRIO_PROCESS,0),
                 quota=Path('/sys/fs/cgroup/cpu.max').read_text().strip(),cpu_model=next(x.split(':')[1].strip()for x in Path('/proc/cpuinfo').read_text().splitlines()if x.startswith('model name'))))
        return
    assert json.loads((d/'gate.json').read_text())['so']==sha(so)
    rss=d/'cap-rss';assert sha(rss)==json.loads((d/'rss-build.json').read_text())['exe']
    assert f'libgrammar.so => {so}' in sp.check_output(['ldd',rss],text=True,env=env)
    result=[]
    for rotation in range(3):
        for v in inputs:
            b=v['block']
            for j in range(6):
                a=(b+rotation+j)%6
                before,after=map(int,sp.check_output([rss,str(a),ref/f'block-{b}.caps',str(v['rows']),str(v['mark']),ref/f'block-{b}.raw'],text=True,env=env).strip().split(','))
                result.append(dict(round=rotation,block=b,arm=NEW[a],pre_hwm_kib=before,post_hwm_kib=after))
    write(d/'rss.csv',result)

if __name__=='__main__':main()
