"""Pointwise independent one-step oracle with every bicoloring and variable byte weights."""
from collections import Counter
from itertools import groupby, product
import json

def lap(start,next_row,words):
    out=bytearray();i=start
    while True:
        out.extend(words[i]);i=next_row[i]
        if i==start:return bytes(out)

def check():
    counts=Counter()
    for n in range(1,8):
        for last in product(range(3),repeat=n):
            k=len(set(last))
            if set(last)!=set(range(k)):continue
            first=sorted(last)
            nxt=sorted(range(n),key=last.__getitem__)
            prev=[nxt.index(i) for i in range(n)]
            start=[first.index(c) for c in range(k)]
            words=[bytes([65+c])*(1+c) for c in first]
            original=[lap(x,nxt,words) for x in range(n)]
            for mask in range(1<<k):
                target=[bool(mask>>c&1) for c in range(k)]
                dead={nxt[i] for i in range(n) if not target[first[i]] and target[first[nxt[i]]]}
                retained=[i for i in range(n) if i not in dead]
                rank={i:j for j,i in enumerate(retained)}
                out_next=[rank[nxt[nxt[i]] if nxt[i] in dead else nxt[i]] for i in retained]
                out_words=[words[i]+(words[nxt[i]] if nxt[i] in dead else b'') for i in retained]
                freq=[0]*k
                entries=[]
                for x,(a,c) in enumerate(zip(last,first)):
                    p=start[a]+freq[a]
                    freq[a]+=1
                    assert p==prev[x]
                    entries.append((a,c,p))
                groups=[(a,c,len(list(g))) for (a,c),g in groupby(zip(last,first))]
                deleted=[0]*k
                saved=[]
                for a,c,width in groups:
                    moved=not target[a] and target[c]
                    saved.extend([deleted[c]]*width)
                    deleted[c]+=width*moved
                counts['cuts']+=1
                for x,(a,c,p) in enumerate(entries):
                    move=not target[a] and target[c]
                    z=p if move else x
                    owner=a if move else c
                    new=z-sum(deleted[:owner])-(0 if move else saved[x])
                    assert new==rank[z] and move==(x in dead)
                    word=lap(new,out_next,out_words)
                    count=len(words[z]) if move else 0
                    cycle=word[count:]+word[:count]
                    assert cycle==original[x]
                    for skip in range(len(words[x])):
                        assert count+skip<len(out_words[new])
                        offset=count+skip
                        assert word[offset:]+word[:offset]==original[x][skip:]+original[x][:skip]
                        counts['weighted_marks']+=1
                    counts['marked_rows']+=1
                    counts['moved']+=move
                    counts['local_prefix_nonzero']+=bool(saved[x] and not move)
                    counts['whole_bucket_prefix_nonzero']+=bool(sum(deleted[:owner]))
    return counts
if __name__=='__main__':print(json.dumps(check(),indent=2,sort_keys=True))
