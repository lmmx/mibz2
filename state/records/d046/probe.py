"""D046: independent interval/spill arithmetic model, not a Rust or Miri run."""
from itertools import product
import json, random
stats=dict(columns=0,cells=0,wide_stores=0,spill_stores=0)
def check(ll):
    n=len(ll)
    counts=[ll.count(c) for c in range(3)]
    starts=[0]+[i for i in range(1,n) if ll[i]!=ll[i-1]]
    ends=starts[1:]+[n]
    seen=[0]*3
    cum=[0,counts[0],counts[0]+counts[1]]
    runs=[]
    for p,e in zip(starts,ends):
        c=ll[p]; a=cum[c]+seen[c]; seen[c]+=e-p
        runs.append((a,p,e-p))
    fo=sorted(runs)
    psi=[i for c in range(3) for i,x in enumerate(ll) if x==c]
    expect=[ll[q]|psi[psi[q]]<<8 for q in range(n)]
    dest=[None]*(n+8)
    cells=0; covered=0
    for a,p,length in fo:
        x=p;q=a;xe=p+length
        while x<xe:
            j=next(j for j,(fa,fp,fl) in enumerate(fo) if fa<=x<fa+fl)
            fa,fp,fl=fo[j];end=min(fa+fl,xe);length=end-x
            assert q==covered and length>0
            base=fp+x-fa
            if n-q>=8 and length<=n-q-8:
                for o in range(0,length,8):
                    assert q+o+8<=n
                    stats['wide_stores']+=1
                    stats['spill_stores']+=o+8>length
                    for m in range(8):
                        dest[q+o+m]=ll[q+o+m]|((base+o+m)<<8)
            else:
                for o in range(length): dest[q+o]=ll[q+o]|((base+o)<<8)
            q+=length;covered+=length;x=end;cells+=1
    assert covered==n and cells<=2*len(runs)-1
    assert dest[:n]==expect
    stats['columns']+=1;stats['cells']+=cells
for n in range(1,9):
    for ll in product(range(3),repeat=n):check(list(ll))
rng=random.Random(0xD046)
for n in [9,15,16,17,23,24,25,31,32,33,63,64,65,127,128,129,255,256,257]:
    for _ in range(25):
        ll=[];c=0
        while len(ll)<n:
            ll.extend([c]*rng.randrange(1,18));c=(c+rng.randrange(1,3))%3
        check(ll[:n])
# Explicit safe-API counterexample's index arithmetic.
ll=[1]; counts=[1]+[0]*255
f=[0]
for count in counts:f.append(f[-1]+count)
q=f[ll[0]]
assert sum(counts)==len(ll) and all(0<=v<=len(ll) for v in f)
assert q==len(ll)==1  # pair_steps reads ll.get_unchecked(q)
print(json.dumps(dict(model=stats,wrong_histogram=dict(ll=ll,counts_nonzero={"0":1},n=1,q=q,unchecked_read_in_bounds=False),limits="Python only; no Rust/Miri/SIMD execution or safety mutation claim"),indent=2))
