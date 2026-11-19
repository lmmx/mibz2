"""Check whole marked outputs/objects, charge local lookups, and reproduce the frozen native panel."""
import csv
import hashlib
import json
from pathlib import Path
from math import exp,log,sqrt
from statistics import mean,median,stdev
import subprocess
import sys
here=Path(__file__).resolve().parent
base=here.parent.parent
arms=('delayed','bucket','local','dense')
def pairs(vals):
    clusters=[mean(vals[i:i+8]) for i in range(0,40,8)]
    m=mean(clusters);h=2.7764451051977987*stdev(clusters)/sqrt(5)
    return [round(exp(x),6) for x in (m,m-h,m+h)]
def gate(build,inputs,dest):
    dest.mkdir()
    truth=json.loads((base/'real.json').read_text())['blocks']
    expected={}
    for row in csv.DictReader((base/'trace.csv').open()):
        if row['arm']=='unseeded':expected.setdefault(row['tag'],[]).append(row)
    counts=[]
    for exe in ('run','audit'):
        small=json.loads(subprocess.check_output([build/exe,'small']))
        assert small==json.loads((base/'tiny.json').read_text())
        for b in truth:
            tag=b['tag']
            inp=inputs/tag/'input.txt'
            assert hashlib.sha256(inp.read_bytes()).hexdigest()==b['raw_input_sha256']
            out=dest/(tag+'-'+exe)
            out.mkdir()
            rows=subprocess.check_output([build/exe,inp,'check',out],text=True)
            for s in rows.splitlines():
                located,local,fixed,hist,prefix,global_runs,name,skip,lap,mult,moves,q,hops,powers=s.split(',')
                out_hash=hashlib.sha256((out/(name+'.bin')).read_bytes()).hexdigest()
                assert out_hash==b['pre_rle_sha256']
                for x,y in zip(csv.DictReader((out/(name+'.csv')).open()),expected[tag],strict=True):
                    assert all(x[k]==y[k] for k in x)
                old=b['arms']['unseeded']
                assert list(map(int,[skip,lap,mult,moves,q,hops,powers]))==[old[k] for k in
                    ('skip','lap','terminal_multiplicity','ordinary_moves','power_queries','power_hops','power_runs')]
                if exe=='audit':
                    counts.append(dict(tag=tag,arm=name,bootstrap_histogram=hist,watched_emissions=prefix,
                        global_rank_prefix=global_runs,possible_fixed=fixed,local_rank_slices=local,
                        local_representative_slices=located,moves=moves,sha256=out_hash))
    with (dest/'counts.csv').open('w',newline='') as f:
        w=csv.DictWriter(f,fieldnames=counts[0],lineterminator='\n');w.writeheader();w.writerows(counts)
    print(json.dumps(dict(builds={s:hashlib.sha256((build/s).read_bytes()).hexdigest() for s in ('run','audit')},
        small=small,real_hashes=60,complete_real_steps=250,tapes=len(truth)),indent=2))
def measure(exe,inputs,results):
    tapes=sorted(inputs.glob('[0-9][0-9][0-9][0-9]/input.txt'))
    assert len(tapes)==10
    with (results/'pairs.csv').open('x') as f:
        subprocess.run([exe,tapes[0],'panel','40',*tapes],stdout=f,check=True)
    with (results/'rss.csv').open('x',newline='') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(['tag','arm','peak_kib'])
        for src in tapes:
            for arm in arms:
                [r]=csv.DictReader(subprocess.check_output([exe,src,arm,'1'],text=True).splitlines())
                w.writerow([r['tag'],r['arm'],r['peak_kib']])
def report(results):
    t={}
    for r in csv.DictReader((results/'pairs.csv').open()):
        key=int(r['round']),r['tag'],r['arm']
        assert key not in t
        t[key]=(int(r['cpu_ns']),int(r['wall_ns']))
    tags=sorted({key[1] for key in t});assert len(t)==1600 and len(tags)==10
    data={}
    for a,b in [('bucket','delayed'),('local','bucket'),('local','delayed'),
                ('delayed','dense'),('bucket','dense'),('local','dense')]:
        entry={}
        for j,name in enumerate(['cpu','wall']):
            entry[name]=pairs([log(sum(t[r,i,a][j] for i in tags)/sum(t[r,i,b][j] for i in tags)) for r in range(40)])
        entry['per_tape_cpu']={i:pairs([log(t[r,i,a][0]/t[r,i,b][0]) for r in range(40)]) for i in tags}
        data[a+'/'+b]=entry
    rss=list(csv.DictReader((results/'rss.csv').open()));assert len(rss)==40
    for a in arms:
        v=[int(r['peak_kib']) for r in rss if r['arm']==a]
        data[a]=dict(max_peak_kib=max(v),median_peak_kib=median(v))
    print(json.dumps(data,indent=2))
cmd=sys.argv[1]
{'gate':gate,'measure':measure,'report':report}[cmd](*[Path(s).resolve() for s in sys.argv[2:]])
