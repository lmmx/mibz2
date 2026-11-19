"""Reproduce scoped CPU shares, local conditional budgets and paid kernel ratios."""
import csv,lzma,math,statistics,sys
from collections import defaultdict
from pathlib import Path
d=Path(__file__).resolve().parent
src=Path(sys.argv[1]) if len(sys.argv)>1 else d
def rows(p):
    with p.open() as f: return list(csv.DictReader(f))
def cm(vals):
    logs=list(map(math.log,vals)); m=statistics.mean(logs)
    z=(2.069 if len(logs)==24 else 2.040)*statistics.stdev(logs)/math.sqrt(len(logs))
    return [round(math.exp(x),6) for x in [m,m-z,m+z]]
matrix=rows(src/'phases.csv')
phases=list(matrix[0])[3:-2]
groups=defaultdict(lambda:defaultdict(int))
for row in matrix:
    assert sum(int(row[v]) for v in phases)==int(row['meter_total'])
    for k in phases+['meter_total','original_total']:
        groups[int(row['round'])][k]+=int(row[k])
assert len(groups)==24 and len(matrix)==24*11
g=list(groups.values())
profile={}
for name in phases:
    share=[r[name]/r['meter_total'] for r in g]
    profile[name]=statistics.median(share)
    print(name,round(statistics.median(r[name] for r in g)/1e6,4),"ms",*(round(x,5) for x in (profile[name],min(share),max(share))))
print('instrument/original full',cm(r['meter_total']/r['original_total'] for r in g))
paid=defaultdict(lambda:[0,0])
with lzma.open(d.parents[1]/'budget/stages.csv.xz','rt') as f:
    for row in csv.DictReader(f):
        if row['arm']!='ahead':continue
        n,c=int(row['rows']),int(row['caps']); data=paid[int(row['round'])]
        data[0]+=int(row['total_ns'])
        if n>=65536 and n<=c*64 and c*20<3*n:data[1]+=int(row['inverse_ns'])
admit=statistics.median(a/b for b,a in paid.values())
print("admitted inverse fraction of all paid worker CPU",round(admit,5))
for w,actual in [(7,.8982),(8,.9117),(9,.9031),(10,.8871)]:
    saving=1-.8/actual
    report=[saving/(admit*x) for x in (profile['scan_queues'],profile['residual_emit'],profile['scan_queues']+profile['residual_emit'])]
    print("W",w,"required fraction of current wall",round(saving,5),"conditional reductions (scan, emit, both)",*(round(x,5) for x in report))
trial_file = src/'paired.csv' if (src/'paired.csv').exists() else src/'rebill/paired.csv'
if trial_file.exists():
    trial=defaultdict(lambda:defaultdict(int))
    for row in rows(trial_file):
        m=trial[int(row['round'])];arm=row['arm']
        for col in ('scan_ns','total_ns'):m[arm,col]+=int(row[col])
    assert len(trial)==32
    for new,old,col in [('meter_queues','meter_original','scan_ns'),('meter_queues','meter_original','total_ns'),('queues','original','total_ns')]:
        print('counter elision',new,old,col,cm(r[new,col]/r[old,col] for r in trial.values()))
