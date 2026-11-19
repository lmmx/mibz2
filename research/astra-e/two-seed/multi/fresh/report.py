from collections import Counter,defaultdict
from pathlib import Path
import csv,json,math,statistics
d=Path(__file__).resolve().parent
rows=list(csv.DictReader((d/'paid.csv').open()))
assert len(rows)==4480
balance=Counter((int(r['block']),int(r['position']),r['arm']) for r in rows)
assert len(balance)==14*64 and set(balance.values())=={5}
acc=defaultdict(int)
for r in rows:
    k=int(r['round']);b=int(r['block']);a=r['arm']
    for g in ('all','dense' if b in (41,56,165,270,278) else 'ordinary'):
        for v in ('construct_ns','emit_ns','total_ns'):
            acc[g,k,a,v]+=int(r[v])
def interval(z):
    m=statistics.mean(z);h=2.02269092*statistics.stdev(z)/math.sqrt(len(z))
    return [round(math.exp(x),6) for x in (m,m-h,m+h)]
res={}
for g in ('all','ordinary','dense'):
  res[g]={}
  for metric in ('emit_ns','construct_ns','total_ns'):
    def avg(k,arm):
        return (math.log(acc[g,k,arm,metric])+math.log(acc[g,k,arm+' twin',metric]))/2
    effects={}
    for prefix in ('retained','fresh'):
      for a in ('scalar','four'):
        arm=prefix+' '+a
        effects[arm+' twin/'+arm]=interval([math.log(acc[g,k,arm+' twin',metric]/acc[g,k,arm,metric]) for k in range(1,41)])
      effects[prefix+' four/scalar']=interval([avg(k,prefix+' four')-avg(k,prefix+' scalar') for k in range(1,41)])
    for a in ('scalar','four'):
        effects[a+' fresh/retained']=interval([avg(k,'fresh '+a)-avg(k,'retained '+a) for k in range(1,41)])
    effects['fresh-by-four interaction']=interval([avg(k,'fresh four')-avg(k,'fresh scalar')-avg(k,'retained four')+avg(k,'retained scalar')for k in range(1,41)])
    effects['per_complete_lap_geomean_ms']={a:round(math.exp(statistics.mean(avg(k,a) for k in range(1,41)))/1e6,5)for a in ('retained scalar','retained four','fresh scalar','fresh four')}
    res[g][metric]=effects
print(json.dumps(res,indent=2))
