"""Accounting of actual reusable allocations, process peaks, and an optimistic fixed-trace subtraction."""
from collections import defaultdict
from pathlib import Path
from statistics import mean
import csv
import sys
d=Path(sys.argv[1])
rss=list(csv.DictReader((d/'rss.csv').open()))
assert len(rss)==1824 and len(set((r['stream'],r['arm']) for r in rss))==len(rss)
total=defaultdict(lambda:defaultdict(int))
for r in csv.DictReader((d/'phase.csv').open()):
    cell=total[r['arm'],int(r['round'])]
    for k in ('whole_ns','post_first_ns','publish_ns','index_ns','discovery_ns','second_merge_ns','second_output_ns'):
        cell[k]+=int(r[k])
w=csv.writer(sys.stdout,lineterminator='\n')
w.writerow(('arm','retained_max_bytes','process_peak_kib','mean_post_first_fraction','mean_publication_fraction',
            'mean_index_fraction','remaining_if_publication_and_index_free'))
for a in sorted(set(r['arm'] for r in rss)):
    group=[r for r in rss if r['arm']==a];assert len(group)==228
    cells=[total[a,j] for j in range(64)]
    def share(k):return mean(c[k]/c['whole_ns'] for c in cells)
    w.writerow((a,max(int(r['retained_bytes_with_output']) for r in group),
                max(int(r['process_hwm_kib']) for r in group),share('post_first_ns'),share('publish_ns'),
                share('index_ns'),1-share('publish_ns')-share('index_ns')))
