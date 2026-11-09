"""Check store census identities using row ownership and literal store-loop counting.
No Rust execution or timing. Exhaustive ternary columns and longer deterministic cases.
"""
import itertools, json, random
from pathlib import Path

def check(col):
    n=len(col)
    starts=[i for i in range(n) if i==0 or col[i]!=col[i-1]]+[n]
    runs=list(zip(starts[:-1],starts[1:]))
    symbols=sorted(set(col)); counts={c:col.count(c) for c in symbols}
    end={}; nextrow={}; p=0
    for c in symbols:
        nextrow[c]=p; p+=counts[c]; end[c]=p
    owner=[None]*n
    for j,(lo,hi) in enumerate(runs):
        c=col[lo]; q=nextrow[c]
        owner[q:q+hi-lo]=[j]*(hi-lo); nextrow[c]+=hi-lo
    # Cut the F row grid on an owner or L-run change, independent of cursor seeding.
    cuts=[0]+[q for q in range(1,n) if owner[q]!=owner[q-1] or col[q]!=col[q-1]]+[n]
    vector=scalar=bslots=0
    for q,ce in zip(cuts,cuts[1:]):
        b=end[col[runs[owner[q]][0]]]; ell=ce-q
        v=min((ell+7)//8,(b-q)//8); s=max(0,ell-8*v)
        w=q; vv=ss=0
        while w<ce and w+8<=b: w+=8;vv+=1
        while w<ce: w+=1;ss+=1
        assert (v,s)==(vv,ss)
        vector+=v;scalar+=s;bslots+=8*((ell+7)//8)
    cells=len(cuts)-1; r=len(runs); streams=len(symbols)
    assert r<=cells<=2*r-1
    assert streams<=r and scalar<=7*streams
    assert 8*cells<=bslots<=n+7*cells
    assert cells<=vector+scalar and n<=8*vector+scalar<=bslots
    tail_cells=bslots//8-vector
    assert (scalar+6)//7 <= tail_cells <= min(scalar,cells)
    assert 0 <= bslots-(8*vector+scalar) <= 7*scalar

cases=0
for n in range(1,9):
    for col in itertools.product(range(3), repeat=n): check(list(col));cases+=1
rng=random.Random(6732258318)
for _ in range(1000):
    col=[]
    for j in range(rng.randrange(1,100)):
        col += [rng.randrange(8)]*rng.randrange(1,60)
    check(col);cases+=1
out={'cases':cases,'exhaustive_ternary':9840,'longer_run_columns':1000,'result':'PASS',
     'scope':'mathematical census model, not compiled instruction counts or native safety'}
Path(__file__).with_suffix('.json').write_text(json.dumps(out,indent=2)+'\n');print(out)
