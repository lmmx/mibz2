"""Selective pair contraction. Bands are (source, image, width, word) in source order."""
from itertools import accumulate

def initial(runs):
    counts=[0]*256
    for w,n in runs:counts[w]+=n
    cursor=[0,*accumulate(counts)]
    buckets=[[] for _ in range(256)];src=0;bands=[]
    for w,n in runs:
        buckets[w].append(len(bands));bands.append((src,cursor[w],n,w))
        src+=n;cursor[w]+=n
    return bands,[i for bucket in buckets for i in bucket]

def step(bands,image,mark,colors,rules):
    zero=[e for e in image if not colors[bands[e][3]]]
    one=[e for e,b in enumerate(bands) if colors[b[3]]]
    count=len(bands)
    assert len(zero)+len(one)==count
    first=[-1]*count;end=[0]*count;hits=[];words=[];width=[]
    i=j=comparisons=removed=below=0
    while i<len(zero) and j<len(one):
        a,b=zero[i],one[j]
        _,y,wa,ca=bands[a];s,_,wb,cb=bands[b]
        lo,hi=max(y,s),min(y+wa,s+wb)
        comparisons+=1
        if lo<hi:
            live=[(lo,mark),(mark+1,hi)] if lo<=mark<hi else [(lo,hi)]
            for l,h in live:
                if h<=l:continue
                t=len(hits);hits.append((l,h))
                words.append(rules.setdefault((cb,ca),256+len(rules)));width.append(h-l)
                removed+=h-l;below+=(h-l) if h<=mark else 0
                for e in (a,b):
                    if first[e]<0:first[e]=t
                    else:assert end[e]==t
                    end[e]=t+1
        if y+wa<=s+wb:i+=1
        if s+wb<=y+wa:j+=1
    assert not zero or not one or comparisons<=count-1
    J=len(hits);items=[];base=[0]
    def gap(lo,hi,w):
        if lo<hi:
            items.append(len(width));width.append(hi-lo);words.append(w)
    for e,(x,y,n,w) in enumerate(bands):
        lo=x if colors[w] else y;hi=lo+n
        if first[e]>=0:
            for h in range(first[e],end[e]):
                l,r=hits[h];gap(lo,l,w);items.append(h);lo=r
        gap(lo,hi,w);base.append(len(items))
    size=len(width)
    def order(owners,joined_color):
        order=[]
        for e in owners:
            keep=colors[bands[e][3]]==joined_color
            for off in range(base[e],base[e+1]):
                t=items[off]
                if keep or t>=J:order.append(t)
        assert len(order)==size
        return order
    source_order=order(range(count),0);image_order=order(image,1)
    target=[0]*size;at=0
    for t in image_order:target[t]=at;at+=width[t]
    nxt=[];rename=[0]*size;at=0
    for t in source_order:
        y,n,w=target[t],width[t],words[t]
        if nxt and nxt[-1][3]==w and nxt[-1][1]+nxt[-1][2]==y:
            a,b,left,w=nxt[-1];nxt[-1]=(a,b,left+n,w)
        else:nxt.append((at,y,n,w))
        rename[t]=len(nxt)-1;at+=n
    next_image=[]
    for t in image_order:
        h=rename[t]
        if not next_image or next_image[-1]!=h:next_image.append(h)
    assert len(next_image)==len(nxt)
    return nxt,next_image,mark-below,dict(comparisons=comparisons,hits=J,gaps=size-J,raw=size,removed=removed,rows=at,bands=len(nxt))
