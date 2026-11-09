"""Finite checks of mathematical formulas only; no decoder candidate or timing."""
import itertools,json,random,bisect
stats={'columns':0,'rows':0,'affine_pair_cells':0,'palette_rows':0,'palette_exception_rows':0,'failures':0}
def check(L):
    n=len(L); F=sorted(L)
    order=sorted(range(n),key=lambda i:L[i]); psi=order
    inv=[0]*n
    for q,p in enumerate(psi):inv[p]=q
    starts=[i for i in range(n) if i==0 or L[i]!=L[i-1]]
    S={inv[p] for p in starts}; A=S|set(starts); S2=S|{inv[s] for s in S}
    assert len(A)==len(S2)<=2*len(S)-1
    cuts=sorted(S2)+[n]
    for lo,hi in zip(cuts,cuts[1:]):
        base=psi[psi[lo]]; middle=F[psi[lo]]
        for q in range(lo,hi):
            assert psi[psi[q]]==base+q-lo
            assert F[psi[q]]==middle and F[q]==F[lo]
    stats['affine_pair_cells']+=len(S2)
    # Mathematical parallel weighted-histogram construction, 1..4 shards.
    rec=[(p,(starts[i+1] if i+1<len(starts) else n)-p,L[p]) for i,p in enumerate(starts)]
    C={c:bisect.bisect_left(F,c) for c in set(F)}
    for h in range(1,5):
        shards=[rec[len(rec)*t//h:len(rec)*(t+1)//h] for t in range(h)]
        prior={c:0 for c in C}; rebuilt=[None]*n
        for shard in shards:
            for p,ell,c in shard:
                a=C[c]+prior[c]; prior[c]+=ell
                rebuilt[a:a+ell]=range(p,p+ell)
        assert rebuilt==psi
    # Spare-bit dense-pair identity; 256-row directory contains high 5 symbol bits.
    B=256; exceptions=0; whole_exceptions=0; transitions=sum((F[i]>>3)!=(F[i-1]>>3) for i in range(1,n))
    for b in range(0,n,B):
        e=min(n,b+B); high=F[b]&248; exceptional=(F[e-1]&248)!=high
        if exceptional: whole_exceptions+=e-b
        # Conservative decoder below falls back for every row in a flagged block.
        for q in range(b,e):
            word=(psi[psi[q]]<<8)|F[psi[q]]|((F[q]&7)<<28)
            assert word<(1<<31)
            assert ((word>>8)&((1<<20)-1))==psi[psi[q]]
            if exceptional:
                first=F[q] # explicit slow-path oracle, not proposed lookup implementation
            else:first=high|((word>>28)&7)
            assert first==F[q] and (word&255)==F[psi[q]]
            packed=psi[q]|((F[q]&7)<<20)
            assert packed<(1<<23) and (packed&((1<<20)-1))==psi[q]
            restored=F[q] if exceptional else high|((packed>>20)&7)
            assert restored==F[q]
            if (F[q]&248)!=high:exceptions+=1
    assert exceptions<=transitions*(B-1)<=31*(B-1)
    assert whole_exceptions<=transitions*B<=31*B
    stats['palette_exception_rows']+=exceptions;stats['palette_rows']+=n
    stats['columns']+=1;stats['rows']+=n
for n in range(1,8):
    for L in itertools.product((0,7,255),repeat=n):check(L)
rng=random.Random(6729486092)
for n in (1,2,7,31,32,255,256,257,1024,9000):
    for k in range(8):check([rng.randrange(256) for _ in range(n)])
# Correct bound for a whole exceptional block fallback includes its leading rows too.
stats['whole_exception_block_row_bound']=31*256
stats['post_boundary_only_row_bound']=31*255
stats['max_n_pointer_bits']=900000 .bit_length()
stats['directory_bytes_at_900k']=(900000+255)//256
stats['whole_block_exception_fraction_bound_at_900k']=31*256/900000
stats['packed24_navigation_bytes_at_900k']=3*900000+(900000+255)//256+256*4
for p in (0,1,65535,65536,899999):
    for c in range(256):
        v=p|((c&7)<<20)
        assert v<(1<<23) and (v&((1<<20)-1))==p
        assert (c&248)|((v>>20)&7)==c
stats['notes']='Formula checks only. No marker/scheduler, concurrency, unsafe-code, corpus or machine-code validation.'
print(json.dumps(stats,indent=2))
