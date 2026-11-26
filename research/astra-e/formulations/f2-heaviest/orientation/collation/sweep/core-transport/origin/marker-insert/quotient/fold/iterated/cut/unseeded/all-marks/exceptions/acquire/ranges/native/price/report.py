"""Reproduce the preregistered k8 construction screen and each physical-block paired estimate."""
import csv,json,sys
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,median,stdev
p=Path(sys.argv[1])
blocks=[r['block'] for r in json.loads((p/'inputs.json').read_text())['blocks']]
samples=list(csv.DictReader((p/'pairs.csv').open()));assert len(samples)==480
values={};first=Counter()
for j,r in enumerate(samples):
    rep=int(r['round']);i=int(r['block']);which=r['arm']
    assert (j//20)+1==rep and i==blocks[((j//2)%10+rep)%10]
    assert int(r['slot'])==j%2
    assert (which=='scalar')==((rep+blocks.index(i)+int(r['slot']))%2==0)
    first[i,which]+=int(r['slot'])==0
    key=rep,i,which
    assert key not in values
    values[key]={'cpu':int(r['thread_ns']),'wall':int(r['wall_ns'])}
assert all(first[i,a]==12 for i in blocks for a in ('ranges','scalar'))
def interval(logs):
    assert len(logs)==24
    clusters=[mean(logs[j:j+6]) for j in range(0,24,6)]
    m=mean(clusters);h=3.182446305284263*stdev(clusters)/sqrt(4)
    return [round(exp(x),6) for x in (m,m-h,m+h)]
def price(ids):
    out={}
    for clock in ('cpu','wall'):
        sums={a:[sum(values[r,i,a][clock] for i in ids) for r in range(1,25)] for a in ('ranges','scalar')}
        out[clock]=dict(ratio_ci95=interval([log(x/y) for x,y in zip(sums['ranges'],sums['scalar'])]),
                        range_median_ms=round(median(sums['ranges'])/1e6,6),scalar_median_ms=round(median(sums['scalar'])/1e6,6))
    return out
rss=list(csv.DictReader((p/'rss.csv').open()));assert len(rss)==60
mem=defaultdict(list)
for r in rss:
    b,a=int(r['block']),r['arm'];mem[b,a].append([int(r['before_kib']),int(r['post_kib'])])
rblocks={}
for i in blocks:
    rblocks[i]=price([i])
    for arm in ('ranges','scalar'):
        rows=mem[i,arm];assert len(rows)==3
        rblocks[i][arm+'_rss_kib']=dict(median_pre=median(x[0] for x in rows),median_post=median(x[1] for x in rows),
                                        max_post=max(x[1] for x in rows))
result=dict(full=price(blocks),blocks=rblocks)
for arm in ('ranges','scalar'):
    allrows=[r for i in blocks for r in mem[i,arm]]
    result[arm+'_rss_kib']=dict(median_pre=median(r[0] for r in allrows),
        median_post=median(r[1] for r in allrows),max_pre=max(r[0] for r in allrows),max_post=max(r[1] for r in allrows))
lo,hi=result['full']['cpu']['ratio_ci95'][1:]
result['screen']='stop_at_k8' if lo>=1 else ('go_to_fully_charged_comparison' if hi<1 else 'unresolved')
print(json.dumps(result,indent=2))
