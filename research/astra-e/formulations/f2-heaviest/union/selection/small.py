"""All short multi-cycle byte columns, checking actual FM visits versus colored suffix counts."""
from collections import Counter,defaultdict
from itertools import combinations,product
from workload import query_count

def check(last):
    n=len(last)
    psi=sorted(range(n),key=last.__getitem__)
    first=[last[x] for x in psi]
    lf=[0]*n
    for i,p in enumerate(psi): lf[p]=i
    quote=[i for i,c in enumerate(first) if c==1]
    by_word=defaultdict(list)
    for start in quote:
        p=start;w=[]
        while True:
            w.append(first[p])
            p=psi[p]
            if first[p]==1: break
        by_word[bytes(w)].append(start)
    result=Counter()
    for count in (1,2):
        for ids in combinations(by_word,count):
            chosen=sorted(x for w in ids for x in by_word[w])
            union=set(chosen)
            comps=[]
            for x in chosen:
                if comps and x==comps[-1][-1]+1: comps[-1].append(x)
                else: comps.append([x])
            dictionaries=[]
            for group in comps:
                ds=set()
                for start in group:
                    p=start;w=[]
                    while True:
                        w.append(last[p]);p=lf[p]
                        if p in union: break
                    ds.add(bytes(reversed(w)))
                dictionaries.append(ds)
            # Independent explicit FM expansion. This performs exactly one lookup
            # per nonterminal query interval, including unary steps.
            active=comps.copy();actual=0
            while active:
                rows=active.pop();actual+=1
                nxt=defaultdict(list)
                for x in rows: nxt[last[x]].append(lf[x])
                for group in nxt.values():
                    assert group==list(range(group[0],group[-1]+1))
                    membership={p in union for p in group}
                    assert len(membership)==1
                    if not membership.pop(): active.append(group)
            from_labels=sum(query_count(ds) for ds in dictionaries)
            literal=sum(len({w[i:] for w in ds for i in range(1,len(w)+1)}) for ds in dictionaries)
            assert actual==from_labels==literal
            result["unions"]+=1;result["queries"]+=actual;result["components"]+=len(comps)
    return result

total=Counter()
for n in range(1,8):
    for l in product(range(3),repeat=n):
        total["columns"]+=1
        total.update(check(l))
print(dict(total))
