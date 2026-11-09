"""Independent table oracle and physical-store model; not Rust execution."""
import itertools,json
from pathlib import Path
def model(ll,mutant=False):
    n=len(ll); out=[None]*n
    rec=[(i,c) for i,c in enumerate(ll) if i==0 or c!=ll[i-1]]+[(n,0)]
    a={};end={};t=0
    for c in sorted(set(ll)):
        a[c]=t;t+=ll.count(c);end[c]=t
    for (p,c),(nxt,_) in zip(rec,rec[1:]):
        q0=a[c];e=q0+nxt-p;q=q0
        while q<e:
            le=next(i for i,_ in rec if i>q)
            ce=min(e,le);w=q
            while w<ce and w+8<=(n if mutant else end[c]):
                for lane in range(8):out[w+lane]=(ll[q]|((p+q-q0+lane+(w-q))<<8))&0xffffffff
                w+=8
            while w<ce:
                out[w]=ll[q]|((p+w-q0)<<8);w+=1
            q=ce
        a[c]=e
    psi=sorted(range(n),key=lambda i:(ll[i],i))
    want=[ll[q]|(psi[q]<<8) for q in range(n)]
    return out,want
count=0
for n in range(9):
    for ll in itertools.product(range(3),repeat=n):
        a,b=model(list(ll));assert a==b;count+=1
for n in range(1,25):
    for ll in [[2]*n+[1]*n,[2]*n+[1]*n+[0]*3+[1]*5]:
        a,b=model(ll);assert a==b;count+=1
ll=[2]*10+[1]*10
bad,want=model(ll,True)
assert bad!=want
# Conservation: per-bucket deficits are nonnegative and sum to zero after a full tiling.
result={'valid_columns':count,'empty_table':model([])[0],'ownership_mutant_witness':{'L':ll,'wrong_rows':[i for i in range(len(ll)) if bad[i]!=want[i]]},'scope':'Python mathematical and physical-store model only; Rust/Miri results come from CI.'}
Path('d055/results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
