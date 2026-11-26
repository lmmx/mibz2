"""Matched complete-lap CPU; lower is faster. Controls share machine code."""
import csv,json,math,statistics
from collections import Counter,defaultdict
from pathlib import Path
HERE=Path(__file__).resolve().parent
labels=['edge','edge twin','edge1','edge4','both','both twin','both1','both4']
def paired(z):
    n=len(z);mu=statistics.mean(z);t={24:2.06865761,48:2.01174051}[n]
    h=t*statistics.stdev(z)/math.sqrt(n)
    return [math.exp(v) for v in (mu,mu-h,mu+h)]
def summarize(name,n,fields):
    total=defaultdict(int);order=Counter();blocks=defaultdict(set)
    with (HERE/name).open() as f:
        for raw in csv.DictReader(f):
            r={k:int(v) for k,v in raw.items()};a=r['arm'];i=r['round'];b=r['block']
            group='dense' if r['dense'] else 'ordinary'
            blocks[group].add(b);order[b,r['position'],a]+=1
            for g in ['all',group]:
                for key in fields:
                    v=r[key];assert v>0
                    total[g,i,a,key]+=v
    assert len(order)==17*64 and set(order.values())=={n//8}
    assert len(blocks['dense'])==5 and len(blocks['ordinary'])==12
    out={'blocks':{g:sorted(v) for g,v in blocks.items()}}
    for g in ['all','ordinary','dense']:
        out[g]={}
        for key in fields:
            effects={}
            for a,b in [(1,0),(2,0),(3,0),(5,4),(6,4),(7,4),(2,1),(3,1),(6,5),(7,5),(3,2),(7,6)]:
                logs=[math.log(total[g,i,a,key]/total[g,i,b,key]) for i in range(1,n+1)]
                effects[f'{labels[a]} / {labels[b]}']=paired(logs)
            for a in [2,3]:
                logs=[math.log((total[g,i,a+4,key]/total[g,i,4,key])/(total[g,i,a,key]/total[g,i,0,key])) for i in range(1,n+1)]
                effects[f'fill interaction {labels[a]}']=paired(logs)
            out[g][key]=effects
    return out
print(json.dumps({'complete':summarize('paid.csv',48,['total_ns']),
                  'metered':summarize('meter.csv',24,['scan_ns','total_ns'])},indent=2))
