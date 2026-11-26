"""Decode the original ordered return permutation from two masks and unlabelled physical arc spellings."""
from collections import defaultdict, deque
from itertools import groupby


def shuffle(source_mask, physical_mask):
    occurrences=[iter(i for i,v in enumerate(physical_mask) if not v),
                 iter(i for i,v in enumerate(physical_mask) if v)]
    return [next(occurrences[b]) for b in source_mask]


def geometry(psi, section, mask):
    a,b=section
    # Two independently sorted lists. A merge yields physical V rows; no old
    # successor or old source→destination return relation is an argument.
    from heapq import merge
    retained=((x,0) for x in range(a,b) if not mask[x-a])
    moved=((psi[x],1) for x in range(a,b) if mask[x-a])
    records=list(merge(retained,moved))
    if any(x[0]==y[0] for x,y in zip(records,records[1:])):
        return None
    return [x for x,_ in records],[v for _,v in records]


def reconstruct(words, counts, source_mask, physical_mask, physical_incoming, c):
    # words/counts use ORIGINAL source bucket order. No original or physical
    # rank-coordinate permutation or positive return is supplied.
    source=[w for w,n in zip(words,counts,strict=True) for _ in range(n)]
    sigma=shuffle(source_mask,physical_mask)
    inverse=[None]*len(source)
    for old,physical in enumerate(sigma):
        inverse[physical]=old
    origins=defaultdict(deque)
    for old in inverse:       # physical V source order
        origins[source[old][source_mask[old]:]].append(old)
    successor=[None]*len(source)
    for dest,physical in enumerate(sigma):  # ORIGINAL destination order
        z=physical_incoming[physical]
        if physical_mask[physical]:
            assert z[-1:]==c
            h=z[:-1]
        else:
            h=z
        old=origins[h].popleft()
        successor[old]=dest
    assert not any(origins.values()) and None not in successor
    return successor


def incoming_by_lf(ranks, last, lf):
    bits=bytearray(len(last))
    for x in ranks:bits[x]=1
    names=[]
    steps=0
    for dest in ranks:
        row=dest
        backward=bytearray()
        while True:
            backward.append(last[row]);row=lf[row]
            if bits[row]:break
        steps+=len(backward)
        names.append(bytes(reversed(backward)))
    assert steps<=len(last)
    return names,steps


def emit(oracle, section, mark, source, words, succ):
    a,b=section;n=len(oracle.first)
    row=mark;out=bytearray()
    while len(out)<n and not a<=row<b:
        out.append(oracle.first[row]);row=oracle.forward[row]
    j=row-a
    while len(out)<n:
        w=words[source[j]];out.extend(w[:n-len(out)]);j=succ[j]
    return bytes(out)


def mask_runs(s):
    return sum(1 for _ in groupby(s))
