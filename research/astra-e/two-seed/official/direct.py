"""Position-balanced direct two-worker comparison, using the official compiled run command."""
from collections import Counter
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys

def plan():
    for k in range(20):
        workers=[1,4] if (k//2)%2==0 else [4,1]
        for w in workers:
            order=[0,1] if (k+(w==4))%2==0 else [1,0]
            for p,arm in enumerate(order):
                yield k,w,p,arm

def run(binaries,corpus,out):
    with out.open('x',buffering=1) as f:
        for k,w,pos,arm in plan():
            cmd=[str(binaries[arm]),'run','--arm','candidate','--workers',str(w),
                 '--repeat','2','--file',str(corpus)]
            r=json.loads(subprocess.check_output(cmd,text=True))
            assert r['workers']==w and r['out_bytes']==419400392
            print(json.dumps(dict(round=k,position=pos,label=['edge','two-seed'][arm],**r)),file=f)

def report(path):
    rows=[json.loads(x) for x in path.read_text().splitlines()]
    assert len(rows)==80
    assert [(r['round'],r['workers'],r['position'],['edge','two-seed'].index(r['label'])) for r in rows]==list(plan())
    totals={}
    for w in [1,4]:
        t={}
        for metric in ['wall_s','cpu_s','peak_rss_kb']:
            z=[]
            for k in range(20):
                pair={r['label']:r[metric] for r in rows if r['workers']==w and r['round']==k}
                z.append(math.log(pair['two-seed']/pair['edge']))
            m=statistics.mean(z)
            delta=2.093024054*statistics.stdev(z)/math.sqrt(20)
            group4=[statistics.mean(z[i:i+4]) for i in range(0,20,4)]
            h=2.776445105*statistics.stdev(group4)/math.sqrt(5)
            t[metric]={'ratio': math.exp(m),'paired_95':[math.exp(m-delta),math.exp(m+delta)],
                       'four_round_95':[math.exp(m-h),math.exp(m+h)]}
            t[metric]['median']={arm:statistics.median(r[metric] for r in rows if r['workers']==w and r['label']==arm)
                                 for arm in ['edge','two-seed']}
        totals[w]=t
    print(json.dumps(totals,indent=2))

if __name__=='__main__':
    if sys.argv[1]=='run':run([Path(x).resolve() for x in sys.argv[2:4]],Path(sys.argv[4]).resolve(),Path(sys.argv[5]))
    else:report(Path(sys.argv[2]))
