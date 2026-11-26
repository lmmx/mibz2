"""Discover delimiter-token-BWT runs by reversing the graph of ordinary adjacent LF gaps."""
from bisect import bisect_right
from collections import Counter
from itertools import accumulate


def compile_runs(runs):
    count = [0]*256
    for c,w in runs:
        count[c]+=w
    base = list(accumulate([0]+count[:-1]))
    cursor = base.copy()
    top = []
    buckets = [[] for _ in count]
    x = 0
    for c,w in runs:
        y = cursor[c]
        top.append((x,x+w,c,y))
        buckets[c].append((y,y+w,c,x))
        cursor[c] += w
        x += w
    bottom = [t for group in buckets for t in group]
    return x,base,count,top,bottom


def construct(runs, primary, delimiter=34):
    n,base,count,top,bottom = compile_runs(runs)
    q,m=base[delimiter],count[delimiter]
    counts=Counter(n=n,runs=len(runs),tokens=m)
    te=[b for _,b,_,_ in top]
    be=[b for _,b,_,_ in bottom]
    # Mark in the section after a delimiter. Refuse its delimiter-free cycle.
    x = primary
    skip=0
    while True:
        lo,_,c,y = top[bisect_right(te,x)]
        counts["mark_lookups"]+=1
        if c==delimiter:
            mark=y+x-lo-q
            break
        x += y-lo
        skip += 1
        if x==primary:
            return None,counts

    # Each successful reverse path identifies one true token-run boundary and
    # the number of non-delimiter symbols shared by its adjacent reversed words.
    splits=[(0,0)]
    for g0,_,_,_ in top[1:]:
        g=g0
        depth=0
        while not q<g<q+m:
            lo,_,c,y=bottom[bisect_right(be,g)]
            counts["reverse_probes"]+=1
            if g==lo or c==delimiter:
                break
            g += y-lo
            depth += 1
        else:
            splits.append((g-q,depth))
            counts["success_steps"]+=depth
            continue
        counts["discard_steps"]+=depth
    splits.sort()
    splits.append((m,0))
    assert len({g for g,_ in splits}) == len(splits)
    assert counts["success_steps"]+counts["discard_steps"] <= n-1

    # A representative path keeps the displacement from its current source,
    # plus the trie node spelling each prefix in reverse traversal order.
    trie=[(-1,delimiter)]
    children={}
    ids=[0]
    disp=[0]
    terminals={}
    words=[]
    bands=[]
    plain = 0
    for (s,h),(end,_) in zip(splits,splits[1:]):
        del ids[h+1:]
        del disp[h+1:]
        assert len(ids)==h+1
        x=q+s+disp[h]
        while True:
            lo,_,c,y=top[bisect_right(te,x)]
            counts["label_probes"]+=1
            if c==delimiter:
                dest=y+x-lo-q
                break
            x+=y-lo
            key=ids[-1],c
            node=children.get(key)
            if node is None:
                node=len(trie)
                trie.append(key)
                children[key]=node
            ids.append(node)
            disp.append(x-(q+s))
        plain += len(ids)
        node=ids[-1]
        word=terminals.get(node)
        if word is None:
            word=len(words)
            terminals[node]=word
            spelling=bytearray()
            while node!=-1:
                par,c=trie[node]
                spelling.append(c)
                node=par
            words.append(bytes(spelling))
        bands.append((s,end-s,dest,word))
    counts.update(token_runs=len(bands),word_count=len(words),dict_bytes=sum(map(len,words)),
                  trie_nodes=len(trie),no_reuse_label_probes=plain,shared_depth=plain-counts["label_probes"])
    # Direct affine first-return bands. The terminal query gives the destination
    # rank, so spelling words never requires sorting them into a token alphabet.
    return (bands,words,mark,skip),counts


def expand(f,n,rows):
    # Materialize the actual token permutation and emit one marked period.
    bands,words,mark,skip=f
    edges=[None]*rows
    for s,w,d,code in bands:
        for k in range(w):
            assert edges[d+k] is None
            edges[d+k]=(s+k,code)
    assert None not in edges
    x=mark
    period=bytearray()
    while True:
        x,c=edges[x]
        period.extend(words[c])
        if x==mark: break
    assert 0<=skip<len(period)
    rotated=period[skip:]+period[:skip]
    return bytes((rotated*((n+len(rotated)-1)//len(rotated)))[:n])
