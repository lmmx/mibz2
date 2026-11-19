"""Paired round sums, using the design frozen before native pricing."""
from collections import Counter, defaultdict
import csv
import json
import math
from pathlib import Path
import statistics
import sys

here=Path(__file__).resolve().parent
base=Path(sys.argv[1]) if len(sys.argv)>1 else here

def load(name):
    with (base/name).open() as f:
        return [{k:int(v) for k,v in r.items()} for r in csv.DictReader(f)]

geometry={r['block']:r for r in csv.DictReader((here/'fused-pairs.csv').open())}
phases=('construct_ns','emit_ns','cpu_ns')
times=defaultdict(Counter)
balance=Counter()
for r in load('paid.csv'):
    assert r['cpu_ns']==r['construct_ns']+r['emit_ns']
    assert r['round'] in range(1,25)
    balance[r['block'],r['arm'],r['position']]+=1
    for grp in ('all','dense' if r['dense'] else 'ordinary',
                'packable' if int(geometry[str(r['block'])]['fits']) else 'fallback'):
        for phase in phases:
            times[grp,r['round'],r['arm']][phase]+=r[phase]
assert len(balance)==17*16 and set(balance.values())=={6}
def estimate(x):
    l=list(map(math.log,x));m=statistics.mean(l);half=2.06865761*statistics.stdev(l)/math.sqrt(24)
    return [math.exp(m),math.exp(m-half),math.exp(m+half)]
ratios={}
for grp in sorted({k[0] for k in times}):
    ratios[grp]={}
    for arm,ctrl in ((1,0),(2,0),(3,0),(3,1),(3,2)):
        ratios[grp][f'{arm}/{ctrl}']={
            phase:estimate(times[grp,r,arm][phase]/times[grp,r,ctrl][phase] for r in range(1,25)) for phase in phases}
rows=load('checked.csv')
storage={}
for arm in (0,2,3):
    a=[r for r in rows if r['arm']==arm]
    assert len(a)==233
    storage[arm]={'sum_initialized_table_and_directory':sum(r['representation_live'] for r in a),
                  'max_retained_table_and_directory':max(r['representation_capacity'] for r in a),
                  'max_retained_including_word_storage':max(r['representation_capacity']+r['dictionary_capacity'] for r in a)}
rss={}
if (base/'memory.csv').is_file():
    vals=load('memory.csv')
    for arm in (0,2,3):
        r=[v['max_rss_kib'] for v in vals if v['arm']==arm]
        assert len(r)==12
        rss[arm]={'n':len(r),'min':min(r),'median':statistics.median(r),'max':max(r)}
print(json.dumps({'storage':storage,'ratios':ratios,'fresh_single_worker_peak_kib':rss},indent=2))
