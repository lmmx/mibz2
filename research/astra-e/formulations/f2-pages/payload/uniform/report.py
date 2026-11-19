import csv
from collections import defaultdict
from pathlib import Path
here=Path(__file__).resolve().parent
rows=list(csv.DictReader((here/'census.csv').open()))
assert len(rows)==228*14
p=[r for r in rows if r['kind']=='page']
total=sum(int(r['lookups']) for r in p)
assert total==19750839
print("First-level dynamic predecessor queries",total)
for i in range(5):
    part=[r for r in p if int(r['cuts_or_wave_hits'])==i]
    count={k:sum(int(r[k]) for r in part) for k in ('pages_or_waves','longest_rows','lookups','unary','longest_hits','longest_unary')}
    print('cuts', i if i<4 else '4+',count, 'lookup share',f"{count['lookups']/total:.6f}",'longest hit share',f"{count['longest_hits']/total:.6f}")
w=[r for r in rows if r['kind']=='wave']
print('wave histogram', {i:sum(int(r['pages_or_waves']) for r in w if int(r['cuts_or_wave_hits'])==i) for i in range(9)})
print('largest-span queries on at most one cut',sum(int(r['longest_hits']) for r in p if int(r['cuts_or_wave_hits'])<=1)/total)
