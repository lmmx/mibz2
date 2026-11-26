"""Verify gates, position-balance and paired cluster inference."""
from collections import Counter,defaultdict
from pathlib import Path
from math import exp, log, sqrt
from statistics import mean, stdev
import csv,sys
d=Path(sys.argv[1])
def rows(f):
    with (d/f).open() as src:return list(csv.DictReader(src))
ten=rows('ten.csv')
full=rows('admissions-dag.csv')
assert len(ten)==10 and len(full)==228
for tag,rr in [('systematic',ten),('production',full)]:
    good=[r for r in rr if r['terminal']=='true']
    print(f'{tag} terminal {len(good)}/{len(rr)}, total n={sum(int(r["n"]) for r in rr)}, fallbacks={Counter(r["cold"] for r in rr)}')
    for key in ['old_rows','old_classes','rounds','final_rows','dag_nodes','paid_runs','paid_classes','removed','moved','period','expanded_internal','leaf_copies','stack','grammar_and_fold_bytes','total_capacity']:
        v=[int(r[key]) for r in rr]
        print(f'  {key:25s} sum={sum(v)} min={min(v)} max={max(v)}')
for arm in ['dag','named','joint']:
    r=rows(f'admissions-{arm}.csv')
    assert [z['hash'] for z in r]==[z['hash'] for z in full]
    print(f'{arm} capacity B={max(int(z["total_capacity"]) for z in r) if arm!="joint" else "unmetered"} fresh-process peak KiB={max(int(z["hwm_kib"]) for z in r)}')
phase=rows('phase.csv')
assert len(phase)==1280 and all(int(r['total_ns'])>0 for r in phase)
totals=defaultdict(int);parts=defaultdict(Counter);balance=Counter()
for r in phase:
    c,rep,st,pos=[int(r[k]) for k in ['cluster','rep','stream','position']]
    assert rep//4==c
    balance[c,st,pos,r['arm']]+=1
    totals[rep,r['arm']]+=int(r['total_ns'])
    for k in ['first_setup_mark','first_discover','first_merge','dag_init','folds','emit','total_ns']:
        parts[r['arm']][k]+=int(r[k])
assert len(balance)==1280 and all(x==1 for x in balance.values())
for arm,metrics in parts.items():
    print(f'{arm}: '+' '.join(f'{k}={v/320/1000:.1f}us/block' for k,v in metrics.items()))
for a,b in [('named_b','named_a'),('dag','named_a'),('dag','named_b'),('dag','joint'),('named_a','joint')]:
    per=[log(totals[rep,a]/totals[rep,b]) for rep in range(32)]
    grouped=[mean(per[i:i+4]) for i in range(0,32,4)]
    center=mean(grouped);radius=2.364624251*stdev(grouped)/sqrt(8)
    print(f'{a}/{b}={exp(center):.4f} [{exp(center-radius):.4f},{exp(center+radius):.4f}]')
