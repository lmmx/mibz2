import math,bisect,random
def bwt(t):
    n=len(t); r=sorted(range(n),key=lambda i:t[i:]+t[:i]); return ''.join(t[i-1] for i in r)
def runs(s): return 1+sum(1 for a,b in zip(s,s[1:]) if a!=b)
a=(math.sqrt(5)-1)/2; n=5000
for p in (2,8,64,256,1024):
    cuts=[(j+0.37)/p for j in range(p)]
    rng=random.Random(7); lab=[rng.randint(0,1) for _ in range(p)]
    t=''.join('ab'[lab[bisect.bisect_right(cuts,(0.1+k*a)%1)%p]] for k in range(n))
    s=bwt(t); print(p, runs(s), round(n/runs(s),1))
