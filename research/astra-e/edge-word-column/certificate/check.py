"""Verify every certificate against the fully published ordinary column, plus the ambiguity."""
from collections import Counter
import csv
from pathlib import Path

here=Path(__file__).resolve().parent
with (here/'choose.csv').open() as f:
    got={r['block']:r for r in csv.DictReader(f)}
with (here.parent/'cuts.csv').open() as f:
    reference={r['block']:r for r in csv.DictReader(f) if r['model']=='physical' and r['depth']=='0'}
assert len(got)==233 and got.keys()==reference.keys()
for k,r in got.items():
    v=reference[k]
    for left,right in [('self_mass','self_mass'),('ingress','initial_ingress'),('seeded_classes','old_classes')]:
        assert r[left]==v[right]
F=(0,1,2,2)
def object_(column):
    target=sorted(range(4),key=column.__getitem__)
    emitted=[]
    x=0
    for _ in range(4):
        emitted.append(F[x])
        x=target[x]
    return Counter(zip(F,column)),emitted
a=object_([1,2,0,2]);b=object_([1,2,2,0])
assert a[0]==b[0] and a[1]==[0,2,1,0] and b[1]==[0,2,2,1]
print('233 seeded edge certificates give the original exact self mass, chosen ingress and class count.')
print('Four-row marked ambiguity: F=0122, L=1202 or 1220, source-pair masses identical; marked words 0210 vs 0221.')
