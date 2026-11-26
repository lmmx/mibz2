"""Checkable linear-time parity cut and exact best Walsh character."""
def signed(v,t):
    return -1 if (v&t).bit_count()%2 else 1


def select(edges,protected,exact=False):
    size=1<<(len(edges)-1).bit_length()
    d=[0]*size
    for i,g in enumerate(edges):
        for j,n in g.items():
            d[i^j]+=n
    if exact:
        f=d.copy()
        width=1
        while width<size:
            for i in range(0,size,width*2):
                for j in range(i,i+width):
                    a,b=f[j],f[j+width]
                    f[j]=a+b
                    f[j+width]=a-b
            width*=2
        mask=min(range(size),key=f.__getitem__)
        bound=f[mask]
    else:
        mask=0
        width=1
        bound=d[0]
        while width<size:
            value=sum(signed(mask,t)*d[t] for t in range(width,2*width))
            if value>0:
                mask+=width
            bound-=abs(value)
            width*=2
    inside={k for k in range(len(edges)) if signed(mask,k^protected)<0}
    entering=sum(n for i,g in enumerate(edges) if i not in inside for j,n in g.items() if j in inside)
    assert 4*entering==sum(d)-bound and bound<=d[0]
    return inside,mask,entering


def tiny():
    from itertools import product
    from collections import Counter
    c=Counter()
    for n in range(1,8):
        for k in range(1,min(4,n)+1):
            for incoming in product(range(k),repeat=n):
                if len(set(incoming))<k:
                    continue
                g=[{} for _ in range(k)]
                for i,j in zip(incoming,sorted(incoming)):
                    g[i][j]=g[i].get(j,0)+1
                for a in range(k):
                    _,m,greedy=select(g,a)
                    _,v,exact=select(g,a,True)
                    brute=max(sum(n for i,targets in enumerate(g) for j,n in targets.items()
                                  if signed(z,i^a)>0 and signed(z,j^a)<0)
                              for z in range(1<<(k-1).bit_length()))
                    assert exact==brute and greedy<=exact
                    c.update(anchored_graphs=1,strict_spectral_improvement=int(exact>greedy))
    return c


if __name__=='__main__':
    import json
    print(json.dumps(tiny(),indent=2))
