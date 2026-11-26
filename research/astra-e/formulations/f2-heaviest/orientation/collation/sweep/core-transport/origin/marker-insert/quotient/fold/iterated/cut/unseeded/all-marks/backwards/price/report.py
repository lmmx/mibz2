"""Same-round byte-equal scalar denominator with 4 independent six-round time clusters."""
from pathlib import Path
from collections import defaultdict,Counter
import csv,json,math,statistics as st,sys
base=Path(sys.argv[1]);here=Path(__file__).resolve().parent
rows=list(csv.DictReader((base/'paired.csv').open()))
assert len(rows)==1440
obs=Counter((q['round'],q['block'],q['arm']) for q in rows);assert max(obs.values())==1
slots=Counter((q['block'],q['arm'],q['slot']) for q in rows);assert set(slots.values())=={4}
fields=['thread_ns','parse_ns','provision_ns','reverse_ns','scatter_ns','terminal_ns','rehearsal_ns']
def ci(a,b):
    per=[math.log(x/y) for x,y in zip(a,b)]
    groups=[st.mean(per[i:i+6]) for i in range(0,24,6)]
    mu=st.mean(groups);radius=3.1824463053*st.stdev(groups)/2
    return [round(math.exp(v),6) for v in (mu,mu-radius,mu+radius)]
result={}
for case in ['all',*sorted({q['block'] for q in rows},key=int)]:
    T=defaultdict(Counter)
    for q in rows:
        if case!='all' and case!=q['block']:continue
        d=T[int(q['round']),q['arm']]
        for k in fields:
            if k=='terminal_ns':
                if q['arm'] in ('free','warm','parse'):d[k]+=int(q['reverse_ns'])+int(q['scatter_ns'])
            elif q.get(k):d[k]+=int(q[k])
    def col(a,k):return [T[r,a][k] for r in range(1,25)]
    scalar=col('scalar','thread_ns');arms=['free','warm','parse','scalar','lazy','opend']
    median={};ratios={}
    for a in arms:
        median[a]={}
        for k in fields:
            v=col(a,k)
            if min(v)<=0:continue
            median[a][k]=round(st.median(v)/1e6,6)
            if a not in ('scalar','lazy','opend'):ratios[a+'.'+k+'/scalar']=ci(v,scalar)
    ratios['warm.terminal_ns/free.terminal_ns']=ci(col('warm','terminal_ns'),col('free','terminal_ns'))
    ratios['free.thread_ns/lazy']=ci(col('free','thread_ns'),col('lazy','thread_ns'))
    ratios['free.thread_ns/opend']=ci(col('free','thread_ns'),col('opend','thread_ns'))
    result[case]=dict(median_ms=median,ratios=ratios)
if (base/'rss.csv').exists():
    z=list(csv.DictReader((base/'rss.csv').open()));assert len(z)==180
    mem={}
    for a in sorted({q['arm'] for q in z}):
        s=[int(q['post_hwm_kib']) for q in z if q['arm']==a]
        assert len(s)==30;mem[a]=dict(median=st.median(s),max=max(s))
    result['rss_kib']=mem
out={k:max(int(q[k]) for q in rows if q['arm'] not in ('scalar','lazy','opend')) for k in ('tree_capacity','offset_capacity','output_capacity','resident_wire_bytes')}
result['largest_per_array_capacity']=out
print(json.dumps(result,indent=2))
