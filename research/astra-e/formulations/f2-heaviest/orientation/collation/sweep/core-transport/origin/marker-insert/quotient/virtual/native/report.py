"""Paired ten-block CPU with chronological, order-balanced clusters."""
from collections import Counter,defaultdict
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,stdev
import csv
import sys
here=Path(sys.argv[1])
rows=list(csv.DictReader((here/'phase.csv').open()))
assert len(rows)==3600
arms=sorted(set(r['arm'] for r in rows))
assert len(arms)==6
pairs=[('named_stream','named'),('quot4_stream','quot4'),('quot8_stream','quot8'),
       ('quot4_stream','named'),('quot8_stream','named')] if 'direct4' not in arms else [
       ('direct4','quot4'),('direct4','quot4_stream'),('direct4','named'),('direct8','quot8'),
       ('direct8','direct4'),('quot4_stream','quot4')]
t=defaultdict(int); slots=Counter();cells=set()
for r in rows:
    c,j,p,k,stream=map(int,(r['cluster'],r['rep'],r['position'],r['block_position'],r['stream']))
    arm=r['arm']
    assert c==j//6 and j in range(60) and p in range(6) and k in range(10)
    cell=(j,stream,arm)
    assert cell not in cells;cells.add(cell)
    slots[c,stream,arm,p]+=1
    for col in ('total_ns','second_index_ns','second_discover_ns','second_merge_ns','second_emit_ns','publication_ns'):
        t[j,arm,col]+=int(r[col])
    t[j,arm,'after_first_ns']+=int(r['total_ns'])-sum(int(r[col]) for col in ('setup_ns','discover_ns','merge_ns'))
    t[j,arm,'index_discover_ns']+=int(r['second_index_ns'])+int(r['second_discover_ns'])
assert set(slots.values())=={1} and len(slots)==3600
with (here/'clusters.csv').open('w') as f:
    out=csv.writer(f,lineterminator='\n');out.writerow(('numerator','denominator','metric','cluster','log_ratio'))
    for a,b in pairs:
        for col in ('total_ns','after_first_ns','index_discover_ns','second_merge_ns','second_emit_ns'):
            groups=[mean(log(t[j,a,col]/t[j,b,col]) for j in range(i,i+6)) for i in range(0,60,6)]
            for c,v in enumerate(groups):out.writerow((a,b,col,c,v))
            center=mean(groups);half=stdev(groups)*2.2621571628540993/sqrt(10)
            print(a,b,col,*(f'{exp(x):.5f}' for x in (center,center-half,center+half)))
for arm in arms:
    print(arm,'mean ten-block ms',{k:round(mean(t[j,arm,k] for j in range(60))/1e6,4) for k in ('total_ns','after_first_ns','index_discover_ns','second_index_ns','second_discover_ns','second_merge_ns','second_emit_ns','publication_ns')})
