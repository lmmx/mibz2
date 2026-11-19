import csv,lzma,sys
from collections import defaultdict,Counter
from pathlib import Path
from statistics import median
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[3]/'bench/moonshot'))
from moonshot import paired_ratio
with lzma.open(here/'times.csv.xz','rt') as f: rows=list(csv.DictReader(f))
assert len(rows)==228*9*36 and all(v==4 for v in Counter((r['block'],r['arm'],r['position']) for r in rows).values())
geom=list(csv.DictReader((here.parent/'geometry.csv').open()))
cohorts={'all':{r['block'] for r in geom}}
for n in [512,1024,2048]:cohorts[f'has{n}']={r['block'] for r in geom if r['cut']==str(n) and int(r['visits'])>0}
out=csv.writer(sys.stdout,lineterminator='\n');out.writerow(['cohort','blocks','arm','reference','ratio','ci_lo','ci_hi','median_ms'])
for group,ids in cohorts.items():
    arms=defaultdict(lambda:[0]*36)
    for r in rows:
        if r['block'] in ids:arms[r['arm']][int(r['round'])-1]+=int(r['cpu_ns'])
    for arm,values in arms.items():
        for ref in ['stock','paid_empty' if arm.startswith('paid') else 'empty']:
            ci=paired_ratio(values,arms[ref])
            out.writerow([group,len(ids),arm,ref,*[f'{ci[k]:.5f}' for k in ['geomean','lo','hi']],f'{median(values)/1e6:.6f}'])
