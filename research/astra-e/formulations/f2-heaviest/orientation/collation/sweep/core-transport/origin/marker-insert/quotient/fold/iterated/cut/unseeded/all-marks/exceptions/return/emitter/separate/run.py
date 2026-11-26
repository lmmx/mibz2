"""Run the preregistered split panels on the frozen physical-return donor."""
from pathlib import Path
import csv
import hashlib
import json
import math
import os
import statistics
import subprocess as sp
import sys

here=Path(__file__).resolve().parent
repo=Path(sp.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
ref,wires,work=[Path(v).resolve()for v in sys.argv[1:4]]
stage=sys.argv[4]
work.mkdir(exist_ok=True,parents=True)
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
prior=json.loads((here.parent/'build.json').read_text())
small_gate=here.parent/'small.csv'
assert sha(ref/'panel')==prior['rust']['exe_sha256']
corpus=repo/'corpora/wikidata.bz2'
assert sha(corpus)==prior['corpus_sha256'] and sha(here.parent/'terminal.cpp')==prior['source_sha256']
rows=json.loads((here.parent/'inputs.json').read_text())
arms=['fused','chronological','split','scalar','lazy','opend']
labels=dict(zip(['stream','streamgaps','dense','scalar','lazy','opend'],arms,strict=True))
if stage=='build':
    price=here.parents[2]/'price'
    header=(price/'clock.cpp').read_text().split('struct Stats{',1)[0]
    (work/'clock-model.hpp').write_text(header)
    points=list(csv.DictReader((here.parent/'times.csv').open()))
    seen={int(r['block']):r for r in points}
    ids='unsigned lookup(unsigned n,unsigned p,unsigned c){\n'
    for q in rows:
        b=q['block'];v=seen[b]
        ids+=f"if(n=={q['n']}u&&p=={q['primary']}u&&c=={v['cap_records']}u)return {b}u;\n"
    (work/'ids.inc').write_text(ids+'std::abort();}\n')
    for file,key in [('clock-model.hpp','header_sha256'),('ids.inc','ids_sha256')]:
        assert sha(work/file)==prior[key]
    small=(here.parent/'terminal.cpp').read_text().split('#else\nint main() {\n',1)[1].split('#endif',1)[0]
    needle='emit_ring(m,table,ring,got.data());assert(got==want);'
    assert small.count(needle)==1
    addition="""
                        m.start=p;std::vector<Event> known,found;
                        U row=p,time=0;
                        do {if(exc[row]) known.push_back({time,payload[row]});row=lf[row];++time;} while(row!=p);
                        U length=acquire(ring,exc[p]!=0,found);
                        assert(length==time&&same_events(known,found));
                        check_clock_lap(m,table,length,found,got.data());assert(got==want);
                        assert(m.n==n&&m.events.empty()&&same_events(known,found));
    """
    (work/'small.inc').write_text('int main() {\n'+small.replace(needle,needle+addition))
    for opt,name in [(['-fPIC','-shared'],'libgrammar.so'),(['-DFINITE'],'small')]:
        sp.run(['g++',*prior['flags'],*opt,'-I',work,here/'split.cpp','-o',work/name],check=True)
    manifest=dict(source=sha(here/'split.cpp'),driver=sha(here/'run.py'),parent=prior['source_sha256'],flags=prior['flags'],
                  rust=prior['rust']['exe_sha256'],small=sha(work/'small'),small_inc=sha(work/'small.inc'),
                  dso=sha(work/'libgrammar.so'),gxx=sp.check_output(['g++','--version'],text=True).splitlines()[0])
    (work/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
elif stage in ('gate','price'):
    mode=sys.argv[5];assert mode in ('reuse','fresh')
    manifest=json.loads((work/'build.json').read_text())
    for path,key in [(here/'split.cpp','source'),(here/'run.py','driver'),(work/'small','small'),(work/'libgrammar.so','dso')]:
        assert sha(path)==manifest[key]
    out=work/(mode+'-'+stage);out.mkdir(exist_ok=True)
    if stage=='gate':
        got=sp.check_output([work/'small']);assert got==small_gate.read_bytes();(work/'small.csv').write_bytes(got)
    for q in rows:
        b=q['block']
        for kind in ('mask','pred'):assert sha(wires/f'{b}.{kind}8')==q[kind+'_sha256']
    env=dict(os.environ,RING_MODE=mode,RING_STAMPS=str(out/'stamps.csv'),CLOCK_WIRES=str(wires),LD_LIBRARY_PATH=str(work))
    assert f'libgrammar.so => {work}/libgrammar.so' in sp.check_output(['ldd',ref/'panel'],env=env,text=True)
    sp.run([ref/'panel',corpus,out,'0' if stage=='gate' else '36'],env=env,check=True)
    stamps=list(csv.DictReader((out/'stamps.csv').open()))
    assert len(stamps)==(10 if stage=='gate' else 370)
    checks={q['block']:q for q in rows}
    for z in stamps:
        q=checks[int(z['block'])];assert int(z['lap'])==q['n'] and int(z['events'])==q['events']
        target=int(z['allocation_bytes']);expected=8*q['return_records']
        assert target>=expected and (target==expected or mode=='reuse')
    for q in rows:
        for ext,key in [('caps','caps_le_sha256'),('raw','raw_sha256')]:
            assert sha(out/f"block-{q['block']}.{ext}")==q[key]
elif stage=='report':
    mode=sys.argv[5];out=work/(mode+'-price')
    zs=list(csv.DictReader((out/'stamps.csv').open()));assert len(zs)==370
    stamps=iter(zs[10:])
    times=list(csv.DictReader((out/'times.csv').open()));assert len(times)==2160
    per={}
    for v in times:
        r,a,b=int(v['round']),labels[v['arm']],int(v['block'])
        slot=int(v['slot']);assert arms[((r+b)+(slot if (r//6)%2==0 else 5-slot))%6]==a
        def put(key,t):
            triple=r,key,b;assert triple not in per;per[triple]=t
        dur=int(v['thread_ns']);put(a,dur)
        if a=='split':
            z=next(stamps);assert int(z['block'])==b
            stages=[int(z[k+'_ns'])for k in ['acquire','emit','release']]
            assert sum(stages)<=dur
            for key,t in zip(['acquire','emit','release'],stages,strict=True):
                put(key,t)
            put('acquire_release',stages[0]+stages[2])
    assert next(stamps,None) is None
    phases=['acquire','emit','release','acquire_release']
    totals={a:[sum(per[r,a,q['block']] for q in rows)for r in range(1,37)]for a in arms+phases}
    def compare(a,b):
        logs=[math.log(x/y)for x,y in zip(totals[a],totals[b],strict=True)]
        g=[statistics.mean(logs[i:i+6])for i in range(0,36,6)]
        m=statistics.mean(g);h=2.570581835636305*statistics.stdev(g)/math.sqrt(6)
        return dict(ratio=math.exp(m),lo=math.exp(m-h),hi=math.exp(m+h),groups=[math.exp(x)for x in g])
    contrasts=[('split','scalar'),('split','fused'),('split','chronological'),('acquire','scalar'),
               ('emit','scalar'),('acquire_release','scalar'),('emit','chronological'),
               ('chronological','scalar'),('fused','scalar')]
    print(json.dumps(dict(mode=mode,rounds=36,arms=labels,
          median_ten_block_ms={a:statistics.median(x)/1e6 for a,x in totals.items()},
          comparisons={f'{a}/{b}':compare(a,b)for a,b in contrasts},
          scratch_bytes=sorted({int(z['allocation_bytes'])for z in zs[10:]})),indent=2))
else: raise ValueError(stage)
