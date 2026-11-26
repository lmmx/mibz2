"""Matched whole-stream worker comparisons with balanced orders for the exact historical pair."""
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys

labels=['edge','scalar','two','edge-four','scalar-four']
def order():
    first=[0,1,4,2,3]
    for r in range(20):
        counts=[1,4] if (r//2)%2==0 else [4,1]
        for w in counts:
            i=(r+int(w==4)*3)%10
            permutation=[(x+i%5)%5 for x in first]
            if i>=5:permutation.reverse()
            for p,k in enumerate(permutation):yield r,w,p,labels[k]

def run(binaries,corpus,out):
    out.mkdir(parents=True,exist_ok=True)
    info={'binaries':{},'corpus_sha256':hashlib.sha256(corpus.read_bytes()).hexdigest()}
    for name,binary in binaries.items():
        info['binaries'][name]={'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
                                'info':json.loads(subprocess.check_output([binary,'info']))}
    assert info['corpus_sha256']=='82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915'
    (out/'binaries.json').write_text(json.dumps(info,indent=2)+'\n')
    with (out/'panel.jsonl').open('x',buffering=1) as f:
        for r,w,pos,arm in order():
            row=json.loads(subprocess.check_output([binaries[arm],'run','--arm','candidate',
                              '--workers',str(w),'--file',corpus,'--repeat','2'],text=True))
            assert row['workers']==w and row['out_bytes']==419400392
            print(json.dumps(dict(round=r,position=pos,label=arm,**row)),file=f)

def report(path):
    rows=[json.loads(x) for x in path.read_text().splitlines()]
    assert [(r['round'],r['workers'],r['position'],r['label']) for r in rows]==list(order())
    index={(r['round'],r['workers'],r['label']):r for r in rows}
    assert len(index)==len(rows)==200
    assert set(Counter((r['workers'],r['position'],r['label']) for r in rows).values())=={4}
    adj=Counter((a['workers'],a['label'],b['label']) for a,b in zip(rows,rows[1:])
                if a['round']==b['round'] and a['workers']==b['workers'])
    assert len(adj)==40 and set(adj.values())=={4}
    assert all(r['out_bytes']==419400392 for r in rows)
    result={}
    for w in [1,4]:
        result[w]={}
        for metric in ['wall_s','cpu_s','peak_rss_kb']:
            contrasts={}
            for name,num,den in [
               ('edge-four/edge',['edge-four'],['edge']),
               ('scalar-four/scalar',['scalar-four'],['scalar']),
               ('edge-four/two',['edge-four'],['two']),
               ('scalar-four/edge-four',['scalar-four'],['edge-four']),
               ('two/edge',['two'],['edge']),
               ('scalar/edge',['scalar'],['edge']),
               ('four-by-scalar interaction',['scalar-four','edge'],['scalar','edge-four']),
            ]:
                logs=[sum(math.log(index[r,w,a][metric]) for a in num)-sum(math.log(index[r,w,a][metric]) for a in den) for r in range(20)]
                mean=statistics.mean(logs)
                scale=2.093024054*statistics.stdev(logs)/math.sqrt(len(logs))
                batch=[statistics.mean(logs[i:i+4]) for i in range(0,20,4)]
                wide=2.776445105*statistics.stdev(batch)/math.sqrt(5)
                contrasts[name]={'ratio':math.exp(mean),'paired_95':[math.exp(mean-scale),math.exp(mean+scale)],
                                 'four_round_95':[math.exp(mean-wide),math.exp(mean+wide)]}
            result[w][metric]=contrasts
            result[w][metric]['medians']={a:statistics.median(index[r,w,a][metric] for r in range(20)) for a in labels}
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    if sys.argv[1]=='run':
        assert len(sys.argv)==9
        run(dict(zip(labels,[Path(p).resolve() for p in sys.argv[2:7]])),Path(sys.argv[7]).resolve(),Path(sys.argv[8]))
    else:report(Path(sys.argv[2]))
