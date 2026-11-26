"""Check all ordered nonzero ranks and the complementary single-selector theorem."""
from collections import Counter
from itertools import product
import json

def previous(k,rank):
    if k==0:return rank
    if k<=rank:return k-1
    return k

def run(limit):
    count=Counter()
    seen=set()
    for i,j in product(range(1,limit),repeat=2):
        state=list(range(limit))
        first=state.pop(i);state.insert(0,first)
        second=state.pop(j);state.insert(0,second)
        assert state[:2]==[second,first]
        assert set(state)==set(range(limit))
        assert (first,second) not in seen
        seen.add((first,second))
        for dst,want in enumerate(state):
            old=previous(previous(dst,j),i)
            assert old==want
            if limit==32:
                encoded=old%16 | (128 if old>=16 else 0)
                from_low=(0 if encoded&128 else encoded&15)
                from_high=(0 if (encoded^128)&128 else 16+(encoded&15))
                assert from_low|from_high == want
            count["positions"]+=1
        count["pairs"]+=1
    return dict(count)
print(json.dumps({k:run(k) for k in [16,32]},indent=2))
