"""Dense independent numerical transport oracle. It stores no expansions or grammar nodes."""
from collections import Counter,defaultdict
from itertools import accumulate,groupby
from math import gcd

def index(last):
    first=sorted(last);cursor={}
    for x,v in enumerate(first):cursor.setdefault(v,x)
    psi=[None]*len(last)
    for x,v in enumerate(last):psi[cursor[v]]=x;cursor[v]+=1
    return first,psi

def walk(first,psi,x,n):
    out=[]
    for _ in range(n):out.append(first[x]);x=psi[x]
    return tuple(out)

def components(first,psi):
    seen=set();out=[]
    for r in range(len(first)):
        if r in seen:continue
        a=[];x=r
        while x not in seen:seen.add(x);a.append(first[x]);x=psi[x]
        assert x==r
        seq=tuple(a);out.append(min(seq[t:]+seq[:t] for t in range(len(seq))))
    return out

def width_quotient(last):
    runs=[(v,sum(1 for _ in h)) for v,h in groupby(last)]
    g=gcd(*(x for v,x in runs))
    return g,tuple(v for v,n in runs for _ in range(n//g))

def ids(keys):
    table={};return [table.setdefault(k,len(table)) for k in keys]

def colors(types,psi):
    # Greedy directed cut on a balanced graph of token types, including all cycles.
    edges=defaultdict(Counter)
    for x,a in enumerate(types):
        b=types[psi[x]]
        if a!=b:edges[a][b]+=1;edges[b][a]+=1
    ch={}
    for k in sorted(set(types)):
        ch[k]=sum(w*(-1 if ch[j] else 1) for j,w in edges[k].items() if j in ch)>0
    return ch

def reverse(last):
    F,original=index(last)
    psi=original;types=F;w=[1]*len(last);frames=[];term=[];stats=Counter()
    while psi:
        n=len(psi);order=defaultdict(list)
        for x,k in enumerate(types):order[k].append(psi[x])
        assert all(v==sorted(v) for v in order.values())
        action=[None]*n
        live=[x for x in range(n) if psi[x]!=x]
        if len(live)<n:
            ordinal={x:i for i,x in enumerate(live)}
            for x in range(n):
                if x in ordinal:action[x]=(ordinal[x],0)
                else:action[x]=(-1-len(term),0);term.append(w[x])
            frames.append(action);psi=[ordinal[psi[x]] for x in live];w=[w[x] for x in live];types=[types[x] for x in live]
            stats["fixed_terminal_events"]+=n-len(live)
            continue
        prev=[0]*n
        for x,y in enumerate(psi):prev[y]=x
        T=colors(types,psi)
        deleted={y for y in range(n) if not T[types[prev[y]]] and T[types[y]]}
        W=[];keys=[];targets=[]
        if 4*len(deleted)>=n:
            starts=[x for x in range(n) if x not in deleted]
            for t,x in enumerate(starts):
                y=psi[x];action[x]=(t,0)
                if y in deleted:
                    action[y]=(t,w[x]);W.append(w[x]+w[y]);keys.append((types[x],types[y]));targets.append(psi[y])
                else:W.append(w[x]);keys.append((types[x],));targets.append(y)
            stats["cuts"]+=1
        else:
            starts=[x for x in range(n) if types[prev[x]]!=types[x]]
            assert starts
            for t,x in enumerate(starts):
                y=x;add=length=0
                while types[y]==types[x]:
                    action[y]=(t,add);add+=w[y];length+=1;y=psi[y]
                W.append(add);keys.append((types[x],length));targets.append(y)
            stats["power_levels"]+=1
        assert all(v is not None for v in action)
        ordinal={x:i for i,x in enumerate(starts)};frames.append(action)
        psi=[ordinal[x] for x in targets];w=W;types=ids(keys)
    rows=[]
    for actions in reversed(frames):
        old=[]
        for target,add in actions:
            t,start=(-1-target,0) if target<0 else rows[target]
            assert 0<=start+add<term[t]
            old.append((t,start+add))
        rows=old
    assert len(set(rows))==len(last)==sum(term)
    tapes=[[None]*v for v in term]
    for r,(t,i) in enumerate(rows):tapes[t][i]=F[r]
    for r,(t,i) in enumerate(rows):
        assert rows[original[r]]==(t,(i+1)%term[t])
    return rows,tapes,stats
