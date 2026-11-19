"""Induce hot return paths using only known first links, plus a rank-set of missing sources."""
from collections import Counter
from itertools import groupby

def descriptors(hot,cut):
    m=len(cut);cold={x for x,v in enumerate(cut) if v is not None}
    E=set(range(m))-{target for target,_ in hot.values()}
    assert len(E)==len(cold)
    if not hot:return None
    hot_freq=Counter(w for _,w in hot.values())
    minbase={}
    for target,w in hot.values():
        minbase[w]=min(target,minbase.get(w,target))
    d=min(hot_freq,key=lambda w:(-hot_freq[w],minbase[w]))
    D=set(range(minbase[d],minbase[d]+hot_freq[d]))
    assert not E&D
    B=D|E
    starts=sorted(B)
    components={}
    comp=-1;previous=-2
    for x in starts:
        if x!=previous+1:comp+=1
        components[x]=comp;previous=x
    macros={}
    suffix_sets=[set() for _ in range(comp+1)]
    hot_types=set(); cold_indices=set(); known_bytes=0; total_visits=0
    for b in starts:
        x=b;prefix=[];color=components[b]
        while x in hot:
            x,w=hot[x];prefix.append(w);total_visits+=1
            if x in B:break
        reached_hot=bool(prefix and x in B and x in D)
        if reached_hot:
            assert prefix[-1]==d
            macros[b]=(prefix,None,x)
            # Exclude the final d from proper backward prefixes.
            proper=tuple(prefix[:-1]);hot_types.add((color,tuple(prefix)))
        else:
            assert x in cold and x not in cold_indices
            cold_indices.add(x);macros[b]=(prefix,x,None);total_visits+=1
            proper=tuple(prefix);known_bytes+=sum(map(len,prefix))
        suffix_sets[color].add(proper)
    assert cold_indices==cold
    Q=0
    for ss in suffix_sets:
        Q+=1;prev=()
        for w in sorted(ss):
            n=min(len(prev),len(w));i=0
            while i<n and w[i]==prev[i]:i+=1
            Q+=len(w)-i;prev=w
    metrics=dict(second_rows=len(D),barrier_rows=len(B),components=comp+1,
                 regularity_queries=Q,hot_types=len(hot_types),dangling_types=len(cold),
                 hot_dict=sum(sum(map(len,w)) for _,w in hot_types),dangling_known_bytes=known_bytes,
                 dense_first_token_visits=total_visits)
    return B,macros,metrics

def decode_barriers(last,p,q,lf,hot,cut,B,macros):
    x=p;reverse=bytearray();offset=queries=0
    while not q<=x<q+len(cut):
        reverse.append(last[x]);x=lf[x];offset+=1
        if x==p:return None
    x-=q;start=x
    def resolve(rank):
        nonlocal queries
        pos,prefix=cut[rank];rev=bytearray(prefix)
        while True:
            c=last[pos];rev.append(c);pos=lf[pos];queries+=1
            if c==34: return pos-q,rev
    # A marked cycle with no B stays hot. Use its first section as anchor.
    initial=bytearray()
    while x not in B:
        if x in hot:
            x,w=hot[x];initial.extend(reversed(w))
        else:
            x,rev=resolve(x);initial.extend(rev)
        if x==start:
            lap=bytes(reversed(initial));off=offset%len(lap)
            lap=lap[off:]+lap[:off]
            return lap*(len(last)//len(lap))+lap[:len(last)%len(lap)],queries,0
    offset+=len(initial);anchor=x;rev=bytearray();visits=0
    while True:
        prefix,cold,next_x=macros[x]
        for w in prefix: rev.extend(reversed(w))
        if cold is None: x=next_x
        else:
            x,tail=resolve(cold);rev.extend(tail)
        visits+=1
        if x==anchor:break
    lap=bytes(reversed(rev));off=offset%len(lap)
    lap=lap[off:]+lap[:off]
    return lap*(len(last)//len(lap))+lap[:len(last)%len(lap)],queries,visits

def verify_partial_intervals(B,hot,cut,macros):
    # Independent FM interval recurrence, with every cold edge stopped before
    # its unknown source rank is needed. Keep exact original token-rank widths.
    slices=[]
    for x in sorted(B):
        if slices and slices[-1][1]==x: slices[-1][1]+=1
        else:slices.append([x,x+1])
    roots=len(slices)
    nodes=[(a,b,()) for a,b in slices]
    children=[];intersections=0
    for a,b,back in nodes:
        bins={};prev=None
        for x in range(a,b):
            if x not in hot:
                tag=(False,x)
            else:
                dest,w=hot[x]
                tag=(True,w)
                if dest not in B:
                    if w not in bins:
                        bins[w]=[dest,dest]
                    assert bins[w][1]==dest;bins[w][1]+=1
            intersections+=int(tag!=prev);prev=tag
        for w,(lo,hi) in list(bins.items()):
            bins[w]=len(nodes);nodes.append((lo,hi,back+(w,)))
        children.append(bins)
    words=[None]*len(nodes)
    for j in reversed(range(len(nodes))):
        a,b,back=nodes[j];out=[];counter={}
        for x in range(a,b):
            if x not in hot:
                assert cut[x] is not None;out.append((back,x,None))
            else:
                target,w=hot[x]
                if target in B: out.append((back+(w,),None,target))
                else:
                    i=counter.get(w,0);out.append(words[children[j][w]][i]);counter[w]=i+1
        words[j]=out
        for k in children[j].values(): words[k]=None
    for j in range(roots):
        a,b,_=nodes[j]
        for x in range(a,b):
            seq,c,t=macros[x]
            assert words[j][x-a]==(tuple(seq),c,t)
    return len(nodes),intersections
