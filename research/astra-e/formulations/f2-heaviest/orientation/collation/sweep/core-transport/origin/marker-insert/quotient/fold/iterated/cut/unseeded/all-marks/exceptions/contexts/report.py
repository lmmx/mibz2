"""Account each chosen k as a fresh construction, keeping structural and measured costs separate."""
import csv
from pathlib import Path
from statistics import median
import sys
d=Path(sys.argv[1])
def load(name):return list(csv.DictReader((d/(name+'.csv')).open()))
blocks=load('blocks');layers=load('layers');times=load('times');rss=load('rss')
assert len(times)==960 and len(rss)==80
cols=('k','models','states','extension_edges','exception_rows','physical_ranges','wire_bytes',
      'original_width_scans','partition_intersections','touched_histogram_entries',
      'final_revisit_intersections','fixed_alphabet_levels','max_observed_dynamic_bytes',
      'median_construct_ms_per_block','fastest_ms','slowest_ms','max_peak_kib')
w=csv.writer(sys.stdout,lineterminator='\n');w.writerow(cols)
for k in sorted({int(r['k']) for r in blocks}):
    group=lambda ds:[r for r in ds if int(r['k'])==k]
    b=group(blocks);l=group(layers);t=group(times)
    assert len(b)==10 and len(t)==120
    tot=lambda rows,key:sum(int(x[key]) for x in rows)
    # First take the median for each physical tape, then its ten-block mean.
    m=[median(int(x['thread_ns']) for x in t if int(x['tape'])==i)/1e6 for i in range(10)]
    w.writerow([k,len(b),*(tot(b,key) for key in ('states','edges','exception_rows','descriptors','wire_bytes','runs')),
                *(tot(l,key) for key in ('overlap','edges','revisit')),len(l),
                max(int(x['peak_dynamic_bytes']) for x in b),round(sum(m)/len(m),6),
                round(min(m),6),round(max(m),6),max(int(x['peak_kib']) for x in group(rss))])
