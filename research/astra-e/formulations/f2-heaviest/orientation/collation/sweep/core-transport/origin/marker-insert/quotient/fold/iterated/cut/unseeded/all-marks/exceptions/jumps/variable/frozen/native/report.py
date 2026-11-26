import csv,hashlib,json,math,statistics as st
from collections import Counter,defaultdict
from pathlib import Path
import sys
d=Path(sys.argv[1])
raw=[dict(r,**{k:int(v) for k,v in r.items() if k!='arm'}) for r in csv.DictReader((d/'times.csv').open())]
arms=['stream','streamgaps','dense','scalar','lazy','opend']
thresholds=(32,128,512)
stats=[list(map(int,r)) for r in csv.reader((d/'stamps.csv').open())]
inputs=json.loads((d/'manifest.json').read_text())['blocks']
ids=[v['block'] for v in inputs]
marks={v['mark']:v['block'] for v in inputs}
assert len(raw)==1440 and len(stats)==750 and len(set(marks))==10
assert set(Counter((r['round'],r['block'],r['arm']) for r in raw).values())=={1}
assert set(Counter((r['block'],r['arm'],r['slot']) for r in raw).values())=={4}
for j,code in enumerate(arms[:3]):
    lines=[v for v in stats if v[2]==j]
    assert [marks[v[1]] for v in lines[:10]]==ids
    matching=[r for r in raw if r['arm']==code]
    assert len(lines)==len(matching)+10
    for v,r in zip(lines[10:],matching):
        n,mark,_,caps,project,index,emit,states,exceptions,lap=v
        assert n==r['rows'] and mark==r['mark'] and not lap
        assert sum(v[3:7])<=r['thread_ns']
        r.update(caps=caps,project=project,index=index,emit=emit)
def grouped_ci(vals):
    # Four complete six-slot placement rotations per arm/block.
    logs=[math.log(v) for v in vals]
    groups=[st.mean(logs[i:i+6]) for i in range(0,24,6)]
    center=st.mean(groups);width=3.182446305*st.stdev(groups)/2
    return [round(math.exp(v),6) for v in (center,center-width,center+width)]
def result(scope):
    rows=[r for r in raw if r['block'] in scope]
    sums=defaultdict(Counter)
    for r in rows:
        for k in ('thread_ns','caps','project','index','emit'):
            if k in r:sums[r['round'],r['arm']][k]+=r[k]
    rates={}
    for a,name in zip(arms[:3],thresholds):
        out=rates[name]={}
        for k in ('thread_ns','caps','project','index','emit'):
            vals=[sums[i,a][k]/sums[i,'scalar']['thread_ns'] for i in range(1,25)]
            out[k+'_over_scalar']=grouped_ci(vals)
    return rates
rss=list(csv.DictReader((d/'rss.csv').open()))
assert len(rss)==180
out=dict(all=result(ids),held_out=result(ids[1:]),by_index={i:result([i]) for i in ids},
         peak_kib={a:max(int(r['peak_kib']) for r in rss if r['arm']==a) for a in arms},
         selector_names=dict(zip(arms[:3],thresholds)))
print(json.dumps(out,indent=2))
