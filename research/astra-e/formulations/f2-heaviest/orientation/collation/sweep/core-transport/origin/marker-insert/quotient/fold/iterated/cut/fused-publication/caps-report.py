import csv, json, statistics, math, sys
from collections import Counter,defaultdict
from pathlib import Path
root=Path(sys.argv[1]);rows=list(csv.DictReader((root/'capped.csv').open()))
assert len(rows)==1440
coverage=Counter((q['round'],q['arm'],q['block']) for q in rows);assert max(coverage.values())==1
totals=defaultdict(Counter);slots=Counter()
for row in rows:
    r=int(row['round']);a=row['arm'];slots[row['block'],a,int(row['slot'])]+=1
    for k in ['thread_ns','first_paid_ns','cut_only_ns']:
        if row[k]:totals[r,a][k]+=int(row[k])
assert set(slots.values())=={4},slots
def ratio(a,p,b,k):
    x=[math.log(totals[r,a][p]/totals[r,b][k]) for r in range(1,25)]
    cl=[statistics.mean(x[i:i+6]) for i in range(0,24,6)]
    mu=statistics.mean(cl);half=3.1824463053*statistics.stdev(cl)/2
    return [round(math.exp(v),6) for v in (mu,mu-half,mu+half)]
arms=sorted({q['arm'] for q in rows});summary={}
research=[a for a in arms if a not in ('scalar','lazy','opend')]
results={}
for a in arms:
    summary[a]={p:round(statistics.median(totals[r,a][p] for r in range(1,25))/1e6,6) for p in totals[1,a]}
    for p in totals[1,a]:
        if a in research:
            for b in (['scalar','lazy','opend',*research] if p=='thread_ns' else ['scalar','opend']):
                if a!=b:results[f'{a}.{p}/{b}.thread_ns']=ratio(a,p,b,'thread_ns')
s={'rows':len(rows),'median_ms':summary,'ratios':results}
if (root/'fresh.csv').exists():
    rss=list(csv.DictReader((root/'fresh.csv').open()));assert len(rss)==180
    s['rss_kib']={}
    for a in arms:
        peak=[int(q['post_kernel_hwm_kib']) for q in rss if q['arm']==a];assert len(peak)==30
        s['rss_kib'][a]={'median':statistics.median(peak),'max':max(peak)}
print(json.dumps(s,indent=2))
