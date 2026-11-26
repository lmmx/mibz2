import csv
from collections import defaultdict
from pathlib import Path
from statistics import median
import sys
d=Path(sys.argv[1])
root=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(root/'bench/moonshot'))
from moonshot import paired_ratio

rows=list(csv.DictReader((d/'stage.csv').open()))
assert len(rows)==2250
samples=defaultdict(dict)
for x in rows:
    if int(x['round'])!=0:
        samples[int(x['block']),x['arm']][int(x['round'])]=int(x['cpu_ns'])
ids=sorted({i for i,a in samples})
arms=sorted({a for i,a in samples})
assert len(ids)==10 and len(arms)==9
def ordered(v):
    return [v[r] for r in range(1,25)]
with (d/'block-ratios.csv').open('w') as f:
    wr=csv.writer(f,lineterminator='\n')
    wr.writerow(('block','arm','paired_median','paired_geomean','lo','hi','faster','arm_ns_median','stock_ns_median'))
    for i in ids:
        stock=ordered(samples[i,'stock'])
        for arm in arms:
            a=ordered(samples[i,arm]);r=paired_ratio(a,stock)
            wr.writerow((i,arm,median(x/y for x,y in zip(a,stock)),r['geomean'],r['lo'],r['hi'],sum(x<y for x,y in zip(a,stock)),median(a),median(stock)))
with (d/'summary.csv').open('w') as f:
    wr=csv.writer(f,lineterminator='\n')
    wr.writerow(('arm','sum_block_medians_over_stock','median_of_all_paired_ratios','blocks_with_median_below_one','faster_pairs','timed_pairs'))
    stock=sum(median(samples[i,'stock'].values()) for i in ids)
    for arm in arms:
        ratios=[[samples[i,arm][r]/samples[i,'stock'][r] for r in range(1,25)] for i in ids]
        flat=[v for vs in ratios for v in vs]
        wr.writerow((arm,sum(median(samples[i,arm].values()) for i in ids)/stock,median(flat),sum(median(v)<1 for v in ratios),sum(v<1 for v in flat),len(flat)))
ns=[int(r['cpu_ns']) for r in rows if r['round']!='0']
print(f"{len(ns)} timed stages; {sum(x>=20_000_000 for x in ns)} >=20ms, {sum(x>=100_000_000 for x in ns)} >=100ms")
