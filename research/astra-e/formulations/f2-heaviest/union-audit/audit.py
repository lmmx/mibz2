"""Exact finite oracle for unions of complete return types in arbitrary stable-LF columns."""
from collections import Counter, defaultdict
from itertools import groupby, product
import json
from pathlib import Path
import sys

def runs(seq):
    return sum(1 for _ in groupby(seq))

def inverse(perm):
    inv = [0]*len(perm)
    for i,j in enumerate(perm):
        inv[j]=i
    return inv

def prefix(row,step,first,n):
    out=[]
    for _ in range(n):
        out.append(first[row])
        row=step[row]
    return tuple(out)

def outgoing(rows,step,first):
    section=set(rows)
    words, targets={},{}
    for x in rows:
        y=x
        w=[]
        while True:
            w.append(first[y])
            y=step[y]
            if y in section: break
        words[x]=tuple(w)
        targets[x]=y
    return words,targets

def contiguous(indices):
    return len(indices)==max(indices)-min(indices)+1

def display(q):
    return ''.join(map(str,q))

def column(n,alphabet,l,stats,examples):
    first=tuple(sorted(l))
    psi=sorted(range(n),key=l.__getitem__)
    lf=inverse(psi)
    patterns={prefix(i,psi,first,m) for i in range(n) for m in range(1,5)}
    for p in sorted(patterns,key=lambda z:(len(z),z)):
        s=[i for i in range(n) if prefix(i,psi,first,len(p))==p]
        assert contiguous(s)
        sw,st=outgoing(s,psi,first)
        parent_types=sorted(set(sw.values()),key=lambda w: min(x for x in s if sw[x]==w))
        types={w:j for j,w in enumerate(parent_types)}
        sr={x:j for j,x in enumerate(s)}
        so=[types[sw[x]] for x in s]
        sp=[sr[st[x]] for x in s]
        slf=inverse(sp)
        parent_last=[so[slf[j]] for j in range(len(s))]
        assert sorted(range(len(s)),key=parent_last.__getitem__)==sp
        rp=runs(parent_last)
        for mask in range(1,1<<len(parent_types)):
            selected={j for j in range(len(parent_types)) if mask>>j&1}
            ur=[j for j in range(len(s)) if so[j] in selected]
            u=[s[j] for j in ur]
            comp={}
            h=0
            for pos,j in enumerate(ur):
                if pos==0 or j!=ur[pos-1]+1: h+=1
                comp[s[j]]=h-1
            pass_u(l,p,psi,lf,first,sw,so,sp,slf,parent_last,parent_types,rp,ur,u,comp,h,selected,stats,examples)

def pass_u(l,p,psi,lf,first,sw,so,sp,slf,parent_last,parent_types,rp,ur,u,comp,h,selected,stats,examples):
    n=len(l)
    uw,ut=outgoing(u,psi,first)
    # Independently induce the dense stable token column on its chosen first letters.
    tw,tt=outgoing(ur,sp,so)
    incoming=[None]*len(u)
    colored=[None]*len(u)
    token_for={}; bucket=defaultdict(list)
    u2r={x:i for i,x in enumerate(u)}
    ur2r={x:i for i,x in enumerate(ur)}
    for i,x in enumerate(u):
        y=ut[x]
        assert ur2r[tt[ur[i]]]==u2r[y]
        w=uw[x]
        token=tw[ur[i]]
        assert tuple(b for t in token for b in parent_types[t])==w
        # Determine all parent visits by the bytes of wP alone, permitting overlaps.
        trailer=w+p
        cuts=[a for a in range(len(w)+1) if trailer[a:a+len(p)]==p]
        assert cuts[0]==0 and cuts[-1]==len(w)
        assert tuple(trailer[a:b] for a,b in zip(cuts,cuts[1:]))==tuple(parent_types[t] for t in token)
        assert w not in token_for or token_for[w]==token
        token_for[w]=token
        key=(w,comp[y])
        incoming[u2r[y]]=w
        colored[u2r[y]]=key
        bucket[key].append(i)
    plain=runs(incoming); tc=runs(colored)
    assert all(contiguous(v) for v in bucket.values())
    # Give each colored bucket its actual low source rank and pair equal labels stably.
    heads={key:min(v) for key,v in bucket.items()}
    seen=Counter()
    packed_prev=[]
    for dest,key in enumerate(colored):
        source=heads[key]+seen[key]
        assert ut[u[source]]==u[dest]
        seen[key]+=1
        packed_prev.append(source)
    packed_next=inverse(packed_prev)
    # Distinct bucket collation from token sequences and a representative destination letter.
    rep={comp[u[i]]:so[ur[i]] for i in reversed(range(len(u)))}
    intrinsic=sorted(bucket,key=lambda k:token_for[k[0]]+(rep[k[1]],))
    assert intrinsic==sorted(bucket,key=heads.get)
    seams=set()
    for a,b in zip(ur,ur[1:]):
        if b!=a+1: continue
        plain_change=incoming[ur2r[a]]!=incoming[ur2r[b]]
        x,y=a,b
        while parent_last[x]==parent_last[y]:
            x,y=slf[x],slf[y]
            assert y==x+1
            assert (so[x] in selected)==(so[y] in selected)
            if so[x] in selected: break
        else:
            assert plain_change and x not in seams
            seams.add(x)
            continue
        assert not plain_change
    assert tc==h+len(seams) and tc<=rp+h-1 and 0<=tc-plain<=h-1
    # Verify the direct original-LF discovery using only h component roots.
    roots=[[x for x in u if comp[x]==j] for j in range(h)]
    nodes=[(q,(),c) for c,q in enumerate(roots)]
    expanded=set()
    overlays=0; q=0
    reached={}
    while nodes:
        interval,word,c=nodes.pop()
        assert contiguous(interval)
        assert expanded.isdisjoint(interval)
        expanded.update(interval);q+=1
        overlays+=runs(l[i] for i in interval)
        for byte in {l[i] for i in interval}:
            child=[lf[i] for i in interval if l[i]==byte]
            assert contiguous(child)
            hit=[row in u2r for row in child]
            assert all(hit) or not any(hit)
            spelled=(byte,)+word
            if hit[0]:
                reached[spelled,c]=[u2r[x] for x in child]
            else:
                nodes.append((child,spelled,c))
    assert overlays<=q+runs(l)-1 and reached==bucket
    # Look for minimal witnesses under this enumerator's n,L,|P|,P,mask order.
    plain_buckets=defaultdict(list)
    for i,x in enumerate(u): plain_buckets[uw[x]].append(i)
    separated=[w for w,v in plain_buckets.items() if not contiguous(v)]
    overlap=any(len(sw[x])<len(p) for x in u)
    unequal=len({len(parent_types[t]) for t in selected})>1
    flags={"bound_tight_h2":h>=2 and tc==plain==rp+h-1,"bound_tight_h3":h>=3 and tc==plain==rp+h-1,
           "color_extra_max":h>=2 and tc-plain==h-1,"color_needed":bool(separated),
           "separated_unequal_overlap":bool(separated) and unequal and overlap,
           "raw_tuple_order_wrong":sorted(bucket)!=intrinsic,
           "plain_extra":plain>rp}
    stats['cases']+=1; stats['h_ge_2']+=h>=2;stats['separated']+=bool(separated)
    stats['overlapping_parent']+=overlap;stats['unequal_selected_lengths']+=unequal
    stats['weak_bound_failures']+=tc>rp+h-1;stats['marked_links']+=len(u)
    for mark in range(n):
        x=mark
        skip=0
        while x not in u2r:
            x=lf[x]
            skip+=1
            if x==mark: break
        if x not in u2r:
            assert mark not in expanded
            stats['marks_on_disjoint_cycles']+=1
            continue
        assert mark in expanded
        current=u2r[x]; emitted=()
        while len(emitted)<n:
            dest=packed_next[current]
            w=colored[dest][0]
            assert skip<len(w)
            emitted+=w[skip:]
            skip=0;current=dest
        assert emitted[:n]==prefix(mark,psi,first,n)
        stats['marked_outputs']+=1
    for label,matches in flags.items():
        if not matches or label in examples: continue
        examples[label]=dict(last=display(l),P=display(p),parent_words=[display(w) for w in parent_types],selected=sorted(selected),
                             parent_last=parent_last,section_rows=u,components=[comp[x] for x in u],r_parent=rp,h=h,
                             incoming=[display(w) for w in incoming],colored=[(display(w),c) for w,c in colored],
                             source_buckets=[(display(w),c,bucket[w,c]) for w,c in intrinsic],plain=plain,colored_runs=tc,
                             flags=[k for k,v in flags.items() if v])
if __name__=="__main__":
    stats=Counter(); examples={}
    limits=[(2,10),(3,7)]
    for n in range(1,max(b for a,b in limits)+1):
        for alphabet,limit in limits:
            if n>limit: continue
            for l in product(range(alphabet),repeat=n):
                if alphabet==3 and 2 not in l: continue
                column(n,alphabet,l,stats,examples)
    print(json.dumps(dict(limits=limits,counts=stats,witnesses=examples),indent=2))
