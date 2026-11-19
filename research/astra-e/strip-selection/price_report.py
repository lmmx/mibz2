import csv,sys
from pathlib import Path
from collections import defaultdict,Counter
from statistics import median
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[2]/'bench/moonshot'))
from moonshot import paired_ratio
rs=list(csv.DictReader((here/'price.csv').open()))
assert len(rs)==24*3*16 and max(Counter((r['arm'],r['position'],r['block']) for r in rs).values())==8
log=defaultdict(lambda:defaultdict(lambda:[0]*24))
for r in rs:
    arm=r['arm'];t=int(r['round'])-1
    total=int(r['complete_ns']);classify=int(r['classify_ns'])
    for k,v in [('complete',total),('classify',classify),('complete_less_classify',total-classify)]:
        log[arm][k][t]+=v
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['arm','metric','reference','ratio','lo','hi','median_ms'])
for a in ['0','1','2']:
    for k in ['complete','classify','complete_less_classify']:
        refs=['0','1'] if k!='classify' else []
        for b in refs:
            d=paired_ratio(log[a][k],log[b][k])
            out.writerow([a,k,b,*[f'{d[x]:.6f}' for x in ['geomean','lo','hi']],f'{median(log[a][k])/1e6:.6f}'])
        if k=='classify':out.writerow([a,k,'',*['']*3,f'{median(log[a][k])/1e6:.6f}'])
