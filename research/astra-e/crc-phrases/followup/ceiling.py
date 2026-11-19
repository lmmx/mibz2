"""Remove every paid CRC-and-return cost of every block with an eligible recurrence."""
import csv,lzma
from collections import defaultdict
from pathlib import Path
from statistics import median
here=Path(__file__).resolve().parent
geom=list(csv.DictReader((here.parent/'geometry.csv').open()))
with lzma.open(here.parents[1]/'budget/stages.csv.xz','rt') as f:
    rows=[r for r in csv.DictReader(f) if r['arm']=='ahead']
assert len(rows)==4310
print('cut blocks min_fraction median_fraction max_fraction')
for cut in [32,128,512,1024,2048]:
    ids={r['block'] for r in geom if int(r['cut'])==cut and int(r['visits'])>0}
    total=defaultdict(int); affected=defaultdict(int)
    for row in rows:
        lap=row['round']
        total[lap]+=int(row['total_ns'])
        if row['block'] in ids:affected[lap]+=int(row['crc_ns'])
    ratios=[affected[k]/v for k,v in total.items()]
    print(cut,len(ids),*[f'{x:.8f}' for x in (min(ratios),median(ratios),max(ratios))])
