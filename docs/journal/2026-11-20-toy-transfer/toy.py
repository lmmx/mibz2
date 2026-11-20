import math
def bwt(t):
    n=len(t); rots=sorted(range(n),key=lambda i:t[i:]+t[:i]); return ''.join(t[i-1] for i in rots)
def runs(s): return 1+sum(1 for a,b in zip(s,s[1:]) if a!=b)
# Christoffel 3/8
w=''.join('b' if (k+1)*3//8-k*3//8 else 'a' for k in range(8)); print('christoffel 3/8',w,bwt(w),runs(bwt(w)))
a=(math.sqrt(5)-1)/2
for n in (100,1000,5000):
    stu=''.join('b' if math.floor((k+1)*a)-math.floor(k*a) else 'a' for k in range(n))
    # rotation coded by r-piece partition of circle, cut points evenly-ish
    out=[n, runs(bwt(stu))]
    for r in (3,4,8):
        cuts=[(j+0.37)/r for j in range(r)]  # pieces labelled cyclically
        lab=lambda x: sum(1 for c in cuts if x>=c)%r
        t=''.join(chr(97+lab((0.1+k*a)%1)) for k in range(n))
        out.append((r,runs(bwt(t))))
    print(out)
print('many pieces, n=5000')
n=5000
for r in (16,64,256,1024):
    cuts=[(j+0.37)/r for j in range(r)]
    import bisect
    t=[ (bisect.bisect_right(cuts,(0.1+k*a)%1))%r for k in range(n)]
    s=''.join(chr(0x100+x) for x in t)
    print(r, runs(bwt(s)), round(n/runs(bwt(s)),1))
