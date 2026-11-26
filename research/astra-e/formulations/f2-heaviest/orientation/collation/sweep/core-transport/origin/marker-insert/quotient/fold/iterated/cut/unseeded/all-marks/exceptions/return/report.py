"""Sum raw interval observations at each context order."""
from pathlib import Path
import csv,sys
rows=[{k:int(v) for k,v in r.items()} for r in csv.DictReader(Path(sys.argv[1]).open())]
assert len(rows)==60
keys=['runs','roots','anchors','atoms','hops','visited_width','singleton_hops','splits','leaves','return_runs','map_lookups','map_comparisons','clock_comparisons','exceptions']
out=csv.writer(sys.stdout,lineterminator='\n')
out.writerow(['k',*keys,'hops_per_row','singleton_share','minimum_hops_per_row','maximum_hops_per_row','max_heap_bytes','max_stack'])
for k in [4,8,16]:
    cell=[r for r in rows if r['k']==k and r['coalesced']==1]
    assert len(cell)==10
    assert all(r['n']==r['visited_width']==r['marked_lap'] for r in cell)
    by={v:sum(r[v]for r in cell)for v in keys}
    frac=[r['hops']/r['n']for r in cell]
    out.writerow([k,*by.values(),by['hops']/by['visited_width'],by['singleton_hops']/by['hops'],min(frac),max(frac),
        max(max(r['overlay_heap_bytes'],r['walk_heap_bytes'])for r in cell),max(r['max_stack']for r in cell)])
