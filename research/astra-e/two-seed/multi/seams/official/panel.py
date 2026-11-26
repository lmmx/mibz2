"""Four independently compiled official workers, complete W1/W4 comparison."""
from collections import Counter
from pathlib import Path
import hashlib
import json
import math
import random
import statistics
import subprocess
import sys

labels=['scalar','four','joined','sliced']
williams=[(0,1,3,2),(1,2,0,3),(2,3,1,0),(3,0,2,1)]
def plan():
    rng=random.Random(781412)
    for group in range(5):
        arms=labels[:];rng.shuffle(arms)
        for phase in range(4):
            rnd=group*4+phase
            for w in ([1,4] if (rnd//2)%2==0 else [4,1]):
                for p,a in enumerate(williams[(phase+(w==4)*2)%4]):
                    yield rnd,w,p,arms[a]
def run(exes,corpus,out):
    out.mkdir(parents=True,exist_ok=True)
    salt={a:{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'info':json.loads(subprocess.check_output([p,'info']))} for a,p in exes.items()}
    (out/'binaries.json').write_text(json.dumps(salt,indent=2)+'\n')
    assert hashlib.sha256(corpus.read_bytes()).hexdigest()=='82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915'
    with (out/'runs.jsonl').open('x',buffering=1) as f:
        for r,w,p,a in plan():
            data=json.loads(subprocess.check_output([exes[a],'run','--arm','candidate','--workers',str(w),'--repeat','1','--file',corpus],text=True))
            assert data['workers']==w and data['out_bytes']==209700196
            print(json.dumps(dict(round=r,position=p,label=a,**data)),file=f)
def report(path):
    rows=[json.loads(x) for x in path.read_text().splitlines()]
    assert [(r['round'],r['workers'],r['position'],r['label']) for r in rows]==list(plan())
    idx={(r['round'],r['workers'],r['label']):r for r in rows};assert len(idx)==len(rows)==160
    assert set(Counter((r['workers'],r['position'],r['label']) for r in rows).values())=={5}
    adj=Counter((a['workers'],a['label'],b['label']) for a,b in zip(rows,rows[1:]) if a['round']==b['round'] and a['workers']==b['workers'])
    assert len(adj)==24 and set(adj.values())=={5}
    assert all(r['out_bytes']==209700196 for r in rows)
    summary={}
    for w in [1,4]:
        summary[w]={}
        for m in ['wall_s','cpu_s','peak_rss_kb']:
            comparisons={}
            for a,b in [('sliced','joined'),('sliced','four'),('sliced','scalar'),('joined','four'),('four','scalar')]:
                z=[math.log(idx[r,w,a][m]/idx[r,w,b][m]) for r in range(20)]
                mean=statistics.mean(z)
                delta=2.093024054*statistics.stdev(z)/math.sqrt(20)
                batches=[statistics.mean(z[i:i+4]) for i in range(0,20,4)]
                wide=2.776445105*statistics.stdev(batches)/math.sqrt(5)
                comparisons[a+'/'+b]=dict(ratio=math.exp(mean),paired_95=[math.exp(mean-delta),math.exp(mean+delta)],
                                           four_round_95=[math.exp(mean-wide),math.exp(mean+wide)])
            comparisons['medians']={a:statistics.median(idx[r,w,a][m] for r in range(20)) for a in labels}
            summary[w][m]=comparisons
    print(json.dumps(summary,indent=2))
if __name__=='__main__':
    if sys.argv[1]=='run':
        assert len(sys.argv)==8
        run(dict(zip(labels,[Path(x).resolve() for x in sys.argv[2:6]])),Path(sys.argv[6]).resolve(),Path(sys.argv[7]))
    else:report(Path(sys.argv[2]))
