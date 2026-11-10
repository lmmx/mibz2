"""Finite mathematical checks of word/landing certificates; no decoder code."""
import itertools,json

def analyse(text):
    n=len(text)
    rotations=sorted((text[i:]+text[:i],i) for i in range(n))
    L=''.join(s[-1] for s,_ in rotations)
    F=''.join(sorted(L))
    psi=sorted(range(n),key=lambda q:(L[q],q))
    S=[q==0 or psi[q]!=psi[q-1]+1 or F[q]!=F[q-1] for q in range(n)]
    cells=[]
    for q in range(n):
        if S[q]: cells.append([])
        cells[-1].append(q)
    def path(q,h):
        word=''
        for _ in range(h): word+=F[q];q=psi[q]
        return word,q
    K=[]
    for q in range(n):
        y=q;k=0
        while not S[y] and k<=n:
            k+=1;y=psi[y]
        K.append(k)
    # Replay the source's reverse-trajectory recurrence on these single-cycle cases.
    origin=next(q for q,(_,i) in enumerate(rotations) if i==0)
    trajectory=[];y=origin
    for _ in range(n):trajectory.append(y);y=psi[y]
    assert len(set(trajectory))==n
    landings=[t for t,y in enumerate(trajectory) if S[y]]
    assert len(landings)==len(cells)
    kk=[0]*n;next_k=0;t0=landings[0]
    for i in reversed(range(n)):
        y=trajectory[(t0+i)%n]
        kk[y]=0 if S[y] else next_k+1
        next_k=kk[y]
    assert kk==K
    out=[]
    for cell in cells:
        if len(cell)<2:continue
        a=cell[0];h=min(K[q] for q in cell[1:])
        assert h>=1
        base,end=path(a,h)
        assert all(path(q,h)==(base,end+q-a) for q in cell)
        claimed=h+1;w,e=path(a,claimed)
        bad=[dict(row=q,word=path(q,claimed)[0],landing=path(q,claimed)[1],claimed_landing=e+q-a) for q in cell if path(q,claimed)!=(w,e+q-a)]
        if bad:out.append(dict(text=text,L=L,F=F,psi=psi,starts=[i for i,x in enumerate(S) if x],K=K,cell=cell,safe_length=h,claimed_length=claimed,base_word=w,bad=bad))
    # Boundary recurrence equals equal-word + affine endpoint cells, not map-only cells.
    for k in range(1,5):
        for q in range(1,n):
            inside=K[q]>=k
            w0,e0=path(q-1,k);w1,e1=path(q,k)
            assert inside==(w0==w1 and e1==e0+1)
    return out

counts={'texts':0,'counterexamples':0};first=None;word_bad=None
for n in range(1,9):
    for bits in itertools.product('ab',repeat=n):
        text=''.join(bits)+'$';bad=analyse(text);counts['texts']+=1
        counts['counterexamples']+=len(bad)
        if bad and first is None:first=bad[0]
        if word_bad is None:
            word_bad=next((w for w in bad if any(z['word']!=w['base_word'] for z in w['bad'])),None)
print(json.dumps({'checks':counts,'first_bad_landing':first,'first_bad_word':word_bad,'cyclic_aab':analyse('aab')},indent=2))
