"""Check every individual word, successor and true run boundary at every mark."""
import itertools
import json
from gaps import construct, expand


def rle(values):
    return [(c,len(list(it)))for c,it in itertools.groupby(values)]


def audit(col,p,d):
    n=len(col)
    psi=sorted(range(n), key=col.__getitem__)
    lf=[None]*n
    for j,i in enumerate(psi):lf[i]=j
    reference=[]
    x=p
    for _ in col:
        x=psi[x]
        reference.append(col[x])
    f,stats=construct(rle(col),p,d)
    if f is None:
        assert d not in reference
        return 0,1
    assert expand(f,n,stats["tokens"])==bytes(reference)
    section=[i for i,c in enumerate(col)if c==d]
    rank={i:j for j,i in enumerate(section)}
    prior=None
    b=0
    for i,x in enumerate(section):
        j=lf[x]
        w=[d]
        while col[j]!=d:
            w.append(col[j]);j=lf[j]
        word=bytes(w[::-1])
        if word!=prior:
            assert f[0][b][0]==i
            b+=1;prior=word
        s,h,dst,code=f[0][b-1]
        assert i<s+h and dst+i-s==rank[j] and f[1][code]==word
    assert b==len(f[0])
    return len(section),0

if __name__=="__main__":
    marks=arrows=miss=0
    for sigma,limit in [(2,8),(3,6)]:
        for n in range(1,limit+1):
            for col in itertools.product(range(sigma),repeat=n):
                for d in range(sigma):
                    for p in range(n):
                        a,m=audit(col,p,d)
                        marks+=1;arrows+=a;miss+=m
    print(json.dumps({"mark_delimiter_cases":marks,"return_arrows":arrows,"marked_cycle_without_delimiter":miss}))
