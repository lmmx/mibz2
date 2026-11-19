from pathlib import Path
from collections import Counter
import csv,sys
here=Path(__file__).resolve().parent
rows=list(csv.DictReader((here/'census.csv').open()));assert len(rows)==228*6
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['page','visits','mixed_visits','mixed_fraction','bands','band_touches','old','new','ratio','over_u8','max_exception_pages'])
for p in [4,8,16,32,64,128]:
    rr=[r for r in rows if int(r['page'])==p]
    a=Counter()
    for r in rr:a.update({k:int(v) for k,v in r.items()})
    out.writerow([p,a['cycle_visits'],a['mixed_visits'],a['mixed_visits']/a['cycle_visits'],a['bands'],a['band_page_touches'],a['old_bytes'],a['packed_bytes'],a['packed_bytes']/a['old_bytes'],a['over_u8'],max(int(r['mixed_pages']) for r in rr)])
