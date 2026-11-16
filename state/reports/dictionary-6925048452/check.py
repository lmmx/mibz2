"""Independent finite checks of the assignment's identities; no decoder timing."""
from itertools import product
import json

counts=dict(inputs=0,states=0,steps=0,tower_points=0,certificates=0)
def lf(L):
    return [sum(v<c for v in L)+sum(v==c for v in L[:i]) for i,c in enumerate(L)]

def check(L,mark):
    n=len(L); pi=lf(L); inv=[pi.index(i) for i in range(n)]
    tok=[(c,i==mark) for i,c in enumerate(L)]
    cuts=[i for i in range(n) if i==0 or L[i]!=L[i-1] or i==mark or i==mark+1]+[n]
    cuts=sorted(set(cuts)); top=list(range(len(cuts)-1)); r0=len(top)
    width={i:cuts[i+1]-cuts[i] for i in top}
    word={i:(tok[cuts[i]],) for i in top}
    bot=sorted(top,key=lambda a:pi[cuts[a]])
    S=eq=0; selected=[];x=mark
    while x not in selected:selected.append(x);x=pi[x]
    single=len(selected)==n
    while top:
        counts['states']+=1
        m=sum(width.values());start={};dest={};t=0
        for a in top:start[a]=t;t+=width[a]
        t=0
        for a in bot:dest[a]=t;t+=width[a]
        T={};rowword={};levels=[];rowlabel={}
        for a in top:
            for x in range(start[a],start[a]+width[a]):
                y=x;path=[]
                while True:
                    path.append(y);y=pi[y]
                    if y<m:break
                assert y==dest[a]+x-start[a]
                assert tuple(tok[z] for z in path)==word[a][::-1]
                T[x]=y;rowword[x]=word[a];rowlabel[x]=a
                levels+=path
                for j,z in enumerate(path):
                    assert pi[z]==(path[j+1] if j+1<len(path) else y)
        sat=set()
        for x in range(m):
            y=x
            while y not in sat:sat.add(y);y=pi[y]
        assert len(levels)==len(set(levels))==len(sat)
        assert set(levels)==sat
        assert sum(width[a]*len(word[a]) for a in top)==len(sat)
        counts['tower_points']+=len(levels)
        # Exact itinerary cylinder intersection versus explicit execution.
        for x in range(m):
            lo=0;hi=m;y=x;shift=0;it=[]
            for k in range(4):
                a=rowlabel[y];it.append(a)
                lo=max(lo,start[a]-shift);hi=min(hi,start[a]+width[a]-shift)
                shift+=T[y]-y;y=T[y]
                brute=[]
                for z in range(m):
                    zz=z;seen=[]
                    for _ in it:seen.append(rowlabel[zz]);zz=T[zz]
                    if seen==it:brute.append(z);assert zz==z+shift
                assert brute==list(range(lo,hi));counts['certificates']+=1
        A=top[-1];B=bot[-1]
        if A==B:
            E=word[A]
            if any(v[1] for v in E):
                assert width[A]==1
                assert len(E)==len(selected)
                j=next(i for i,v in enumerate(E) if v[1])
                x=mark;out=[]
                for _ in range(n):x=inv[x];out.append(L[x])
                assert out==[E[(j+1+k)%len(E)][0] for k in range(n)]
                if single:assert len(top)==1 and r0-1<=S<=n-1 and eq==r0-1
                break
            top.remove(A);bot.remove(A);del width[A];del word[A];continue
        a,b=width[A],width[B];hA,hB=len(word[A]),len(word[B]);N=word[A]+word[B]
        if a>b:
            width[A]-=b;word[B]=N;bot.remove(B);bot.insert(bot.index(A)+1,B)
            assert a*hA+b*hB==width[A]*hA+b*len(N)
        elif a<b:
            width[B]-=a;word[A]=N;top.remove(A);top.insert(top.index(B)+1,A)
            assert a*hA+b*hB==a*len(N)+width[B]*hB
        else:
            word[B]=N;top.remove(A);bot.remove(B);bot[bot.index(A)]=B;del width[A];del word[A];eq+=1
            assert a*hA+b*hB==b*len(N)
        counts['steps']+=1;S+=1
    counts['inputs']+=1

for n in range(1,7):
    for L in product(range(3),repeat=n):
        for mark in range(n):check(L,mark)

# Batching identities, including noncommutative words.
for q in range(1,30):
    A='ab';B='cba';w=B
    for _ in range(q):w=A+w
    assert w==A*q+B
    w=A
    for _ in range(q):w=w+B
    assert w==A+B*q

# Modulus obstruction and corrected incidence relation, loop included.
for h in [(1,), (3,), (1,1), (2,1,3)]:
    for modulus in range(1,8):
        solutions=[v for v in product(range(modulus),repeat=len(h))
                   if all((v[(i+1)%len(h)]-v[i]+h[i])%modulus==0 for i in range(len(h)))]
        assert len(solutions)==(modulus if sum(h)%modulus==0 else 0)

# Grammar relation with two DIFFERENT operand incidences to the same symbol.
alphabet=['','a','b','aa','ab','ba','bb']
sol=[(u,v,w) for u in alphabet for v in alphabet for w in alphabet if w==u+v and u=='a' and v=='a']
assert sol==[('a','a','aa')]
counts['result']='all assertions passed'
def certificate_share(m,threshold,cap):
    found=0
    for start in range(m):
        x=start;lo=-m;hi=m
        for j in range(cap):
            a,b=(0,m-1) if x<m-1 else (m-1,m)
            lo=max(lo,a-x);hi=min(hi,b-x)
            if j+1>=threshold:
                found+=hi-lo>=2;break
            x=(x+1)%m
    return found/m
counts['certificate_cap_counterexample']={
    'map':'F(x)=x+1 mod 1000; intervals [0,999),[999,1000); unit word lengths',
    'uncapped_256_byte_share':certificate_share(1000,256,256),
    'cap64_share':certificate_share(1000,256,64)}
assert counts['certificate_cap_counterexample']['uncapped_256_byte_share']==0.744
assert counts['certificate_cap_counterexample']['cap64_share']==0
print(json.dumps(counts,indent=2))
