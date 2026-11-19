"""Bounded single-CPU direct price versus the exact scalar candidate."""
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys
def plan():
    for i in range(24):
        for a in ([0,1] if i%2==0 else [1,0]):yield i,a
def run():
    exes=[Path(x) for x in sys.argv[2:4]]
    corpus=Path(sys.argv[4]);out=Path(sys.argv[5])
    assert os.sched_getaffinity(0)=={0}
    with out.open('x',buffering=1) as f:
        for k,a in plan():
            o=json.loads(subprocess.check_output([exes[a],'run','--arm','candidate','--workers','1','--repeat','1','--file',corpus]))
            assert o['out_bytes']==209700196 and o['workers']==1
            print(json.dumps(dict(round=k,label=a,**o)),file=f)
def report():
    r=[json.loads(l) for l in Path(sys.argv[2]).read_text().splitlines()]
    assert [(v['round'],v['label'])for v in r]==list(plan())
    assert all(v['out_bytes']==209700196 for v in r)
    d={(v['round'],v['label']):v for v in r};out={}
    for m in ['wall_s','cpu_s','peak_rss_kb']:
        z=[math.log(d[k,1][m]/d[k,0][m]) for k in range(24)]
        mean=statistics.mean(z);h=2.06865761*statistics.stdev(z)/math.sqrt(24)
        c=[statistics.mean(z[i:i+4]) for i in range(0,24,4)]
        w=2.57058184*statistics.stdev(c)/math.sqrt(6)
        out[m]={'ratio':math.exp(mean),'paired_95':[math.exp(mean-h),math.exp(mean+h)],
                'four_round_95':[math.exp(mean-w),math.exp(mean+w)],
                'median_scalar':statistics.median(d[k,0][m] for k in range(24)),
                'median_selective':statistics.median(d[k,1][m]for k in range(24))}
    print(json.dumps(out,indent=2))
if sys.argv[1]=='run':run()
else:report()
