"""Recheck every paired rotation and its per-block/aggregate terminal prices."""
from collections import defaultdict,Counter
from pathlib import Path
import csv,json,statistics as st,math,sys
d=Path(sys.argv[1]);x=list(csv.DictReader((d/'paired.csv').open()));assert len(x)==1440
assert max(Counter((q['round'],q['block'],q['arm']) for q in x).values())==1
assert set(Counter((q['block'],q['arm'],q['slot']) for q in x).values())=={4}
def ci(a,b):
    x=[math.log(u/v) for u,v in zip(a,b)]
    g=[st.mean(x[j:j+6]) for j in range(0,24,6)];mu=st.mean(g);hw=3.1824463053*st.stdev(g)/2
    return [round(math.exp(v),6) for v in (mu,mu-hw,mu+hw)]
result={}
arms=sorted(set(q['arm'] for q in x))
mode='prepared' if 'jump_repeat' in arms else 'provision'
for block in ['all',*sorted({q['block'] for q in x},key=int)]:
    t=defaultdict(Counter)
    for q in x:
        if block!='all' and q['block']!=block:continue
        h=t[int(q['round']),q['arm']]
        for k in ['thread_ns','parse_ns','jump_ns','provision_ns','first_ns','verify_ns','second_ns']:
            if q.get(k):h[k]+=int(q[k])
    get=lambda a,k:[t[r,a][k] for r in range(1,25)]
    s=get('scalar','thread_ns');ratios={};median={}
    for a in arms:
        median[a]={}
        for k in t[1,a]:
            v=get(a,k)
            if min(v)<=0:continue
            median[a][k]=round(st.median(v)/1e6,6)
            if a not in ('scalar','lazy','opend'):ratios[f'{a}.{k}/scalar']=ci(v,s)
    if mode=='prepared':
        ratios['jump.first/unit.first']=ci(get('jump','first_ns'),get('unit_repeat','first_ns'))
        ratios['jump.first/lazy']=ci(get('jump','first_ns'),get('lazy','thread_ns'))
        ratios['jump.first/opend']=ci(get('jump','first_ns'),get('opend','thread_ns'))
        ratios['jump.warm/unit.warm']=ci(get('jump_repeat','second_ns'),get('unit_repeat','second_ns'))
        for a in ['unit_repeat','jump_repeat']:ratios[a+'.second/first']=ci(get(a,'second_ns'),get(a,'first_ns'))
        ratios['jump_replay_first/jump_first']=ci(get('jump_repeat','first_ns'),get('jump','first_ns'))
    else:
        ratios['paid_jump/paid_unit']=ci(get('parse_jump','thread_ns'),get('parse_unit','thread_ns'))
        ratios['prepared_model_jump/paid_unit']=ci(get('build_jump','thread_ns'),get('parse_unit','thread_ns'))
    result[block]=dict(median_ms=median,ratios=ratios)
if (d/'rss.csv').exists():
    q=list(csv.DictReader((d/'rss.csv').open()));assert len(q)==180;mem={}
    for a in arms:
        v=[int(row['post_hwm_kib']) for row in q if row['arm']==a];mem[a]=dict(median=st.median(v),max=max(v))
    result['rss_kib']=mem
g=json.loads((d/'gate.json').read_text())['files']
result['geometry']= {s:sum(q[s] for q in g) for s in ['n','states','events','wire_bytes','jump8','unit_tail']}
result['max_capacity']= {s:max(int(q[s]) for q in x if q.get(s)) for s in ['state_bytes','event_bytes','jump_bytes','output_bytes','resident_wire_bytes','reference_bytes']}
print(json.dumps(result,indent=2))
