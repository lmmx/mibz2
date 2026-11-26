"""Summarize the native per-label census, conditional queue growth, and split marks."""
import csv
import json
import lzma
from collections import Counter,defaultdict
from pathlib import Path

here=Path(__file__).resolve().parent
def records(name):
    with lzma.open(here/(name+'.csv.xz'),'rt') as src:
        for row in csv.DictReader(src):
            yield {k:int(v) for k,v in row.items()}
perblock=defaultdict(Counter)
for r in records('widths'):
    h={i:r['width'+str(i)] for i in range(1,17)}
    assert sum(i*v for i,v in h.items())==r['rows']
    perblock[r['block']].update(h)
total=sum(perblock.values(),Counter())
cumulative={}
for cut in (1,2,3,4,8,16):
    calls=sum(total[w] for w in range(1,cut+1))
    entries=sum(w*total[w] for w in range(1,cut+1))
    part=[sum(v[w] for w in range(1,cut+1))/sum(v.values()) for v in perblock.values()]
    cumulative[cut]=dict(calls=calls,call_fraction=calls/sum(total.values()),rows=entries,
        row_fraction=entries/sum(k*v for k,v in total.items()),min_block_call_fraction=min(part),
        max_block_call_fraction=max(part))
ledger=defaultdict(Counter)
for r in records('growth'):
    ledger[r['slots'],r['cut']].update({k:r[k] for k in ('growing_appends','first_allocations','reallocations','existing_u32_at_reallocation')})
peaks=defaultdict(int)
for r in records('blocks'):
    k=r['slots'],r['cut']
    peaks[k]=max(peaks[k],4*r['all_slot_capacity_rows'])
changes=defaultdict(Counter)
for r in records('capacities'):
    for key in ('upto8','allpush'):
        old,new=r['range'],r[key]
        changes[r['slots'],key]['equal' if old==new else 'smaller' if new<old else 'larger']+=1
split=defaultdict(list);split_grows=Counter()
for r in records('marks'):
    k=r['slots'],r['cut']
    split[k].append(r['block'])
    split_grows[k]+=r['capacity_after']!=r['capacity_before']
modes={}
for slots in (0,1,7,8,9,10):
    modes[slots]={}
    for cut in (0,1,2,3,4,8,16):
        k=slots,cut
        v={**ledger[k],'peak_queue_bytes':peaks[k],'split_singleton_growths':split_grows[k]}
        if cut in (8,16):
            v['active_block_label_capacity_vs_range']=changes[slots,'upto8' if cut==8 else 'allpush']
        modes[slots][cut]=v
        assert len(split[k])==7 and (cut>4 or peaks[k]==peaks[slots,0])
old={int(r['block']):r for r in csv.DictReader((here.parents[1]/'private-cache/census.csv').open()) if r['limit']=='16'}
assert set(old)==set(perblock) and all(int(old[b]['m'])==sum(w*v for w,v in h.items()) for b,h in perblock.items())
print(json.dumps(dict(blocks=len(perblock),width_histogram=total,cumulative=cumulative,
                      forced_split_blocks=split[1,0],modes=modes),indent=2))
