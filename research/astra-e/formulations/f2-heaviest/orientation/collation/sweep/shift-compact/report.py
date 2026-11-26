"""Use paired ten-block thread CPU sums, with no dropped measurements."""
import csv,math,sys
from collections import defaultdict
from pathlib import Path
from statistics import mean,stdev
from scipy.stats import t
rows=list(csv.DictReader(Path(sys.argv[1]).open()))
sums=defaultdict(lambda:defaultdict(int))
mem=defaultdict(list)
for r in rows:
    mem[r['arm']].append(int(r['reserved_bytes']))
    for f in ('setup_mark_ns','discover_ns','merge_ns','first_ns','total_ns'):
        sums[r['arm'],f][int(r['rep'])]+=int(r[f])
def pair(a,b,f):
    x=sums[a,f];y=sums[b,f]
    logs=[math.log(x[i]/y[i]) for i in x];m=mean(logs);e=t.ppf(.975,len(logs)-1)*stdev(logs)/math.sqrt(len(logs))
    return f'{math.exp(m):.4f} [{math.exp(m-e):.4f}, {math.exp(m+e):.4f}]'
for a in mem:
    for b in ['compact_quote','compact_frequent','projected_h0']:
        if a==b:continue
        print(a,'/',b,'total',pair(a,b,'total_ns'),'first',pair(a,b,'first_ns'),'setup',pair(a,b,'setup_mark_ns'),'discovery',pair(a,b,'discover_ns'))
for a,x in mem.items():print(a,'reserved bytes including output',min(x),max(x))

def adjusted(a,b,replace):
    samples=[]
    for i in sums[a,'total_ns']:
        v=sums[a,'total_ns'][i]-sums[a,'setup_mark_ns'][i]
        if replace:v+=sums[b,'setup_mark_ns'][i]
        samples.append(math.log(v/sums[b,'total_ns'][i]))
    m=mean(samples);e=t.ppf(.975,len(samples)-1)*stdev(samples)/math.sqrt(len(samples))
    return f'{math.exp(m):.4f} [{math.exp(m-e):.4f}, {math.exp(m+e):.4f}]'
for a in ('projected_h1','projected_h2','projected_h3'):
    for b in ('compact_quote','compact_frequent','projected_h0'):
        print(a,'/',b,'replace setup with baseline setup',adjusted(a,b,True),'free all setup',adjusted(a,b,False))
