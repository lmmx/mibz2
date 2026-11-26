"""Independent ordered-interval modular induction, with actual syntax and dead-rule accounting."""
from itertools import groupby


def runs(column):
    atoms=[]
    src=0
    counts={a:column.count(a) for a in set(column)}
    base={}
    cur=0
    for a,count in sorted(counts.items()):
        base[a]=cur
        cur+=count
    for a,items in groupby(column):
        w=sum(1 for _ in items)
        atoms.append((src,base[a],w,a))
        base[a]+=w
        src+=w
    return atoms


def project(arcs,q,p):
    n=sum(a[2] for a in arcs)
    selected=p%q
    residues=[(selected+i)%q for i in range(q)]
    # Complete groups need not have equal sizes.
    base={}
    end=0
    for r in residues:
        base[r]=end
        end+=len(range(r,n,q))
    pieces=[]
    for src,dst,w,rule in arcs:
        for i in range(min(w,q)):
            x,y=src+i,dst+i
            width=1+(w-1-i)//q
            pieces.append((base[x%q]+x//q,base[y%q]+y//q,width,rule))
    k=len(range(selected,n,q))
    return sorted(pieces), k, p//q


def induce(arcs,keep,grammar):
    n=sum(r[2] for r in arcs)
    top=sorted(range(len(arcs)),key=lambda i:arcs[i][0])
    bottom=sorted(top,key=lambda i:arcs[i][1])
    widths=[r[2] for r in arcs]
    rule=[r[3] for r in arcs]
    strict=equal=self_count=partial=0
    while n>keep:
        t,b=top[-1],bottom[-1]
        tw,bw=widths[t],widths[b]
        w=min(tw,bw,n-keep)
        n-=w
        if t==b:
            self_count+=1
            widths[t]-=w
            if not widths[t]:
                top.pop();bottom.pop()
            continue
        new=256+len(grammar)
        grammar.append((rule[b],rule[t]))
        if w<min(tw,bw):
            partial+=1
            assert n==keep
            a=len(widths);widths.append(w);rule.append(new)
            widths[t]-=w;widths[b]-=w
            top.insert(top.index(b)+1,a)
            bottom.insert(bottom.index(t)+1,a)
        elif tw==bw:
            equal+=1
            top.pop();bottom.pop()
            bottom[bottom.index(t)]=b
            rule[b]=new
        elif w==bw:
            strict+=1
            widths[t]-=w;rule[b]=new
            bottom.pop();bottom.insert(bottom.index(t)+1,b)
        else:
            strict+=1
            widths[b]-=w;rule[t]=new
            top.pop();top.insert(top.index(b)+1,t)
    targets={}
    y=0
    for a in bottom:
        targets[a]=y;y+=widths[a]
    x=0;result=[]
    for a in top:
        result.append((x,targets[a],widths[a],rule[a]))
        x+=widths[a]
    assert x==y==keep
    return result,(strict,equal,self_count,partial)


def cascade(arcs,p,q):
    gr=[];st=[]
    while (n:=sum(x[2] for x in arcs))>1:
        proj,keep,mark=project(arcs,q,p)
        before=len(gr)
        nxt,bill=induce(proj,keep,gr)
        assert len(proj)<=min(q*len(arcs),n)
        assert len(nxt)<=min(keep,len(proj)+1)
        st.append(dict(n=n,marked=p,bands=len(arcs),progressions=len(proj),kept=keep,
                       survivors=len(nxt),strict=bill[0],equal=bill[1],fixed=bill[2],
                       partial=bill[3],joins=len(gr)-before))
        arcs,p=nxt,mark
    reachable=set()
    length={x:1 for x in range(256)}
    for j,(l,r) in enumerate(gr,256):
        length[j]=length[l]+length[r]
    stack=[arcs[0][3]]
    while stack:
        x=stack.pop()
        if x>=256 and x not in reachable:
            reachable.add(x);stack.extend(gr[x-256])
    return st,gr,reachable,length[arcs[0][3]],arcs[0][3]


def spell(gr,root):
    stack=[root];out=bytearray()
    while stack:
        x=stack.pop()
        if x<256:out.append(x)
        else:
            l,r=gr[x-256];stack.extend((r,l))
    return out
