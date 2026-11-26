"""Apply the immutable six-arm Rust and cap-only RSS harness to a modular grammar plugin."""
import argparse
from collections import Counter
import csv
import importlib.util
import json
import os
from pathlib import Path
import subprocess as sp
import sys
import statistics as st
import math

D=Path(__file__).resolve().parent
PREV=D.parents[1]/'native/bench'
spec=importlib.util.spec_from_file_location('section_tools',PREV/'run.py')
tools=importlib.util.module_from_spec(spec);spec.loader.exec_module(tools)
sha,read,dump,write=tools.sha,tools.read,tools.dump,tools.write
OLD=tools.OLD;NEW=['q2','q4','q8',*OLD[3:]]
PHASE=['cap_ns','induce_ns','spell_ns']
SHAPE=['runs','q','layers','joins','terminal','lap','visits','copies','aliases','repeats','pieces','steps','retained_bytes','including_tree_bytes']

def execute(ref,out,action):
    out.mkdir(exist_ok=True,parents=True)
    orig=json.loads((tools.DONOR/'build.json').read_text());local=json.loads((ref/'build.json').read_text())
    inputs=json.loads((tools.DONOR/'receipt.json').read_text());cases=inputs['inputs']
    for k in ('donor','source_sha256','src_tree','rlib_sha256','rust_flags','rustc','exe_sha256'):assert orig[k]==local[k]
    assert sha(tools.CORPUS)==inputs['corpus_sha256'] and sha(ref/'panel')==orig['exe_sha256']
    so=out/'libgrammar.so';env=dict(os.environ,LD_LIBRARY_PATH=str(out));env.pop('MODULAR_STAMPS',None)
    if action=='build':
        flags=['-std=c++20','-O3','-g0','-Wall','-Wextra','-shared','-fPIC']
        sp.run(['g++',*flags,D/'bridge.cpp','-o',so],check=True)
        sp.run(['g++','-std=c++20','-O2',PREV/'dense.cpp','-o',out/'dense'],check=True)
        deps=sp.check_output(['g++','-std=c++20','-MM',D/'bridge.cpp'],text=True).replace('\\\n','').split()[1:]
        dump(out/'build.json',dict(so=sha(so),dense=sha(out/'dense'),sources={str(Path(x).resolve().relative_to(tools.ROOT)):sha(Path(x))for x in deps},
             corpus=sha(tools.CORPUS),controls=local,flags=flags,compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0]))
        return
    assert sha(so)==json.loads((out/'build.json').read_text())['so']
    if action=='rss-build':
        sp.run([sys.executable,PREV/'rss_build.py',ref,out],check=True);return
    if action!='gate':assert json.loads((out/'gate.json').read_text())['so']==sha(so)
    if action=='rss':
        rss=out/'cap-rss';assert sha(rss)==json.loads((out/'rss-build.json').read_text())['exe']
        assert f'libgrammar.so => {so}' in sp.check_output(['ldd',rss],text=True,env=env)
        data=[]
        for rnd in range(3):
            for v in cases:
                b=v['block']
                for j in range(6):
                    a=(rnd+b+j)%6
                    c=[rss,str(a),ref/f'block-{b}.caps',str(v['rows']),str(v['mark']),ref/f'block-{b}.raw']
                    before,peak=map(int,sp.check_output(c,text=True,env=env).strip().split(','))
                    data.append(dict(round=rnd,block=b,arm=NEW[a],pre_hwm_kib=before,post_hwm_kib=peak))
        write(out/'rss.csv',data);return
    assert f'libgrammar.so => {so}' in sp.check_output(['ldd',ref/'panel'],text=True,env=env)
    gate=action=='gate';stamp=out/('gate-stamps.csv' if gate else 'stamps.csv')
    assert not (out/'paired.csv').exists()
    sp.run([ref/'panel',tools.CORPUS,out,'0' if gate else '24'],env=dict(env,MODULAR_STAMPS=str(stamp)),check=True)
    tools.check_caps(out,cases)
    key={(v['rows'],v['mark'],v['records']):str(v['block']) for v in cases};assert len(key)==10
    st=read(stamp);assert len(st)==(30 if gate else 750)
    for s in st:
        s['block']=key[tuple(int(s[k])for k in ('n','mark','records'))]
        assert int(s['q'])==2<<int(s['arm']) and s['n']==s['lap']
    assert len({(s['block'],s['arm'])for s in st[:30]})==30
    if gate:
        oracle=[]
        for v in cases:
            b=v['block'];r,lap=map(int,sp.check_output([out/'dense',out/f'block-{b}.caps',out/f'block-{b}.raw',str(v['mark'])],text=True).strip().split(','))
            assert r==lap==v['rows'];oracle.append(dict(block=b,rows=r,lap=lap))
        dump(out/'gate.json',dict(so=sha(so),matched_rows=60,cases=cases,dense=oracle));return
    data=read(out/'times.csv');assert len(data)==1440
    it=iter(st[30:])
    for row in data:
        a=OLD.index(row['arm']);row['arm']=NEW[a]
        if a<3:
            s=next(it);assert int(s['arm'])==a and s['block']==row['block']
            assert sum(int(s[k])for k in PHASE)<=int(row['thread_ns'])
            row.update({k:s[k]for k in PHASE+SHAPE})
    assert not list(it)
    write(out/'paired.csv',data)
    dump(out/'runtime.json',dict(affinity=sorted(os.sched_getaffinity(0)),nice=os.getpriority(os.PRIO_PROCESS,0),
         quota=Path('/sys/fs/cgroup/cpu.max').read_text().strip(),cpu=next(x.split(':')[1].strip()for x in Path('/proc/cpuinfo').read_text().splitlines()if x.startswith('model name'))))

if __name__=='__main__':
    a=argparse.ArgumentParser();a.add_argument('donor',type=Path);a.add_argument('out',type=Path);a.add_argument('action',choices=['build','gate','price','rss-build','rss'])
    v=a.parse_args();execute(v.donor.resolve(),v.out.resolve(),v.action)
