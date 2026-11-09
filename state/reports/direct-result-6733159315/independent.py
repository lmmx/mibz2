"""Untimed arithmetic and representation model; no decoder implementation."""
import json, math, statistics, itertools, hashlib
from pathlib import Path

def ci(xs):
    logs=list(map(math.log,xs)); mu=statistics.mean(logs)
    h=2.145*statistics.stdev(logs)/math.sqrt(len(logs))
    return [math.exp(mu),math.exp(mu-h),math.exp(mu+h)]
r=json.loads(Path('r24/artifact/run-fed-1.json').read_text())
out={'independent_ratios':{},'model_columns':0,'model_rows':0}
for f in r['files']:
    e=f['entry']; p=f['phases']
    out['independent_ratios'][f['name']]={'DA':ci([sum(x[2])/sum(x[0]) for x in e]),'BA':ci([sum(x[1])/sum(x[0]) for x in e]),'DB':ci([sum(x[2])/sum(x[1]) for x in e])}
w=r['files'][0]; ps=w['phases'];scale=r['ghz']*1e9/w['output']
out['place_over_sortfill']=ci([x[2][2]/(x[1][1]+x[1][2]) for x in ps])
out['delta_break_even']=statistics.median([(sum(x[0])-x[1][0]-x[1][2]-x[1][3]+(x[1][0]-x[2][0])+(x[1][3]-x[2][3]))*scale for x in ps])
out['phase_DA_gap']=statistics.median([(sum(x[2])-sum(x[0]))*scale for x in ps])
out['metadata_saved_bytes_per_record_row']=4*w['records']/w['rows_runs']
out['extra_table_stores']=(w['d_vector']+w['d_scalar'])-w['slots']//8
out['extra_table_stores_percent']=100*out['extra_table_stores']/(w['slots']//8)
out['table_slots_reduction_percent']=100*(1-(8*w['d_vector']+w['d_scalar'])/w['slots'])
rat=out['independent_ratios'][w['name']]
out['D_reduction_to_095']=1-.95/rat['DA'][0]
out['B_reduction_to_095']=1-.95/rat['BA'][0]

# Stable per-byte streams concatenated by byte; an index needs the ORIGINAL next start.
# No assumption of valid single-cycle BWT needed; this is a table constructor identity.
for n in range(10):
 for L in itertools.product(range(3),repeat=n):
    starts=[i for i in range(n) if i==0 or L[i]!=L[i-1]]
    ends=starts[1:]+[n]
    streams=[[j for j,p in enumerate(starts) if L[p]==c] for c in range(3)]
    psi=sorted(range(n),key=lambda p:L[p])
    table=[];q=0;cells=0;lcur=0
    for ids in streams:
     for j in ids:
      p=starts[j];length=ends[j]-p
      for t in range(length):
       table.append((L[q],p+t));q+=1
      # F interval intersects one monotonically consumed L partition.
      a=q-length
      while a<q:
       while lcur+1<len(starts) and starts[lcur+1]<=a:lcur+=1
       a=min(q,ends[lcur]);cells+=1
    assert table==list(zip(L,psi))
    assert cells<=max(0,2*len(starts)-1)
    out['model_columns']+=1;out['model_rows']+=n

# Failure witness: using adjacent F-sorted starts as run lengths is wrong.
L=[1,0,1];out['wrong_length_witness']={'L':L,'L_starts':[0,1,2],'F_sorted_original_starts':[1,0,2],'wrong_first_length':-1,'correct_first_length':1}
Path('r24/independent.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
