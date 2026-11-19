from pathlib import Path
from collections import defaultdict,Counter
from statistics import median
import csv,sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[2]/'bench/moonshot'))
from moonshot import paired_ratio
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['process','metric','page','ratio','ci_lo','ci_hi','median_total_ms'])
for path in map(Path,sys.argv[1:]):
    rows=list(csv.DictReader(path.open()));assert len(rows)==11*4*64
    assert all(v==16 for v in Counter((r['block'],r['arm'],r['position']) for r in rows).values())
    for metric in ['prepare','emit','complete']:
        laps=defaultdict(lambda:[0]*64)
        for r in rows:laps[r['arm']][int(r['round'])-1]+=int(r[metric])
        for arm in ['0','8','16','32']:
            v=laps[arm];ci=paired_ratio(v,laps['0'])
            out.writerow([path.name,metric,arm,*[f'{ci[k]:.5f}' for k in ['geomean','lo','hi']],f'{median(v)/1e6:.5f}'])
