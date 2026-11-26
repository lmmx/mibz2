"""Raw lap aggregation; the 24 round sums are the paired sampling units."""
from collections import Counter,defaultdict
import csv,json,math,statistics
from pathlib import Path
here=Path(__file__).resolve().parent

def data(name):
    with (here/name).open() as f:
        for x in csv.DictReader(f):
            yield {k:int(v) for k,v in x.items()}
shapes=defaultdict(list)
for r in data('checked.csv'):shapes[r['arm']].append(r)
geometry={}
for a,vals in shapes.items():
    assert len(vals)==233
    geometry[a]={k:sum(x[k] for x in vals) for k in ['representation_live','dictionary_live']}
    geometry[a]['max_representation_retained']=max(x['representation_capacity'] for x in vals)
    geometry[a]['max_representation_and_dictionary_retained']=max(x['representation_capacity']+x['dictionary_capacity'] for x in vals)
    geometry[a]['minimum_complete_walk']=min(x['representation_live']+x['dictionary_live'] for x in vals)
    for cap in [192,256]:
        geometry[a][f'fitting_{cap}KiB']=sum(x['representation_live']+x['dictionary_live']<=cap*1024 for x in vals)
times=defaultdict(Counter);balance=Counter();ordinals=defaultdict(set)
for r in data('paid.csv'):
    a=r['arm'];i=r['round'];g='dense' if r['dense'] else 'ordinary'
    assert r['construct_ns']+r['emit_ns']==r['cpu_ns']
    balance[r['block'],a,r['position']]+=1
    ordinals[g].add(r['block'])
    for z in ['all',g]:
        for k in ['construct_ns','emit_ns','cpu_ns']:times[z,i,a][k]+=r[k]
assert len(balance)==17*16 and set(balance.values())=={6} and len(times)==3*4*24

def interval(z):
    mid=statistics.mean(z);r=2.06865761*statistics.stdev(z)/math.sqrt(24)
    return [math.exp(x) for x in [mid,mid-r,mid+r]]

paid={}
for g in ['all','dense','ordinary']:
    paid[g]={}
    for a in [1,2,3]:
        paid[g][a]={k:interval([math.log(times[g,i,a][k]/times[g,i,0][k]) for i in range(1,25)])
                    for k in ['construct_ns','emit_ns','cpu_ns']}
memory=defaultdict(list)
if (here/'memory.csv').is_file():
    for r in data('memory.csv'):memory[r['arm']].append(r['max_rss_kib'])
print(json.dumps({'structural':geometry,'paid_cpu_ratio':paid,'ordinals':{k:sorted(v) for k,v in ordinals.items()},
                  'single_worker_rss':{a:{'n':len(v),'min':min(v),'median':statistics.median(v),'max':max(v)} for a,v in memory.items()}},indent=2))
