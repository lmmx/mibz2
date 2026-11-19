"""Summarize stable-source counts, library paths, cold capacities and paired prices."""
from pathlib import Path
from collections import Counter,defaultdict
import csv,json,statistics,math
here=Path(__file__).resolve().parent
def geometry(p):
    sums=defaultdict(Counter)
    with p.open() as f:
        for r in csv.DictReader(f):
            c={k:int(v) for k,v in r.items() if k!='case'}
            key='-'.join(r['case'].split('-')[1:])
            for group in [key,'positive' if c['holes'] else 'identity','all']:
                sums[group].update(c)
                sums[group]['cases']+=1
    return {k:{v:z[v] for v in ['cases','m_j','m_h','holes','q','k_j','k_h','r_j','r_h','compound_bytes','original_dictionary','old_capacity','new_capacity','arena_moved','j_used_second','h_used_second','old_j_fallback','run_fragments','source_segments']} for k,z in sums.items()}
def price(p):
    affected=set()
    with (here/'checked.csv').open()as f:
        for r in csv.DictReader(f):
            if int(r['holes']):affected.add(r['case'])
    obs=defaultdict(Counter);order=Counter()
    with p.open()as f:
        for r in csv.DictReader(f):
            i,a=(int(r[k]) for k in ['round','arm']);key=r['case']
            for g in ['all','positive' if key in affected else 'identity',key.split('-')[1]]:
                obs[g,i,a].update({k:int(r[k]) for k in ['splice_ns','consume_ns','complete_ns']})
            order[key,a,int(r['position'])]+=1
    assert len(order)==20*4*4 and set(order.values())=={6}
    def interval(v):
        h=2.06865761*statistics.stdev(v)/math.sqrt(len(v));m=statistics.mean(v)
        return [math.exp(z) for z in [m,m-h,m+h]]
    out={}
    for g in ['all','positive','identity','frequent','quote']:
        out[g]={}
        for a,ref in [(1,0),(2,0),(3,0),(2,3)]:
            out[g][f'complete_{a}/{ref}']=interval([math.log(obs[g,i,a]['complete_ns']/obs[g,i,ref]['complete_ns']) for i in range(1,25)])
        out[g]['median_splice_ns_arm2_per_lap']=statistics.median(obs[g,i,2]['splice_ns'] for i in range(1,25))
    return out
result={'primary':geometry(here/'checked.csv'),'extended':geometry(here/'extended.csv'),'paid':price(here/'paid.csv')}
for name in ['reserved-primary','reserved-extended']:
    p=here/(name+'.csv')
    if p.exists():result[name]=geometry(p)
print(json.dumps(result,indent=2))
