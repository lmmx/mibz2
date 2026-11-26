"""Exhaust the de Bruijn feedback tables and every undetectable pair change of a one-cycle table."""
import json


def partners(k, code):
    size=1<<k
    middle=size//2
    # bit u exchanges the incoming bytes of adjacent rows u0 and u1.
    last=[((code>>(x//2))&1)^(x&1) for x in range(size)]
    lf=[middle*b+(x//2) for x,b in enumerate(last)]
    before=[0,0]
    for x,b in enumerate(last):
        assert lf[x]==middle*b+before[b]
        before[b]+=1
    assert before==[middle,middle]
    psi=[0]*size
    for x,y in enumerate(lf):psi[y]=x
    for source in range(size):
        word=0
        x=source
        for _ in range(k):
            word=2*word+(x//middle)
            x=psi[x]
        assert word==source
    return lf


def cycle_lengths(lf):
    seen=[False]*len(lf)
    lengths=[]
    for start in range(len(lf)):
        if seen[start]:continue
        p=start
        length=0
        while not seen[p]:
            seen[p]=True
            p=lf[p]
            length+=1
        assert p==start
        lengths.append(length)
    return lengths


def verify(k):
    size=1<<k
    pairs=size//2
    total=1<<pairs
    count=swaps=0
    min_lap=size
    max_lap=0
    for table in range(total):
        lf=partners(k,table)
        cycles=cycle_lengths(lf)
        if len(cycles)!=1:continue
        assert cycles==[size]
        count+=1
        for pair in range(pairs):
            a=2*pair
            lf[a],lf[a+1]=lf[a+1],lf[a]
            changed=cycle_lengths(lf)
            assert len(changed)==2 and sum(changed)==size and all(n<size for n in changed)
            # Here the chosen primary is 0. The first cycle enumerated contains 0.
            min_lap=min(min_lap,changed[0])
            max_lap=max(max_lap,changed[0])
            swaps+=1
            lf[a],lf[a+1]=lf[a+1],lf[a]
    assert count==1<<(pairs-k)
    return dict(k=k,states=size,pairs=pairs,feedback_tables=total,single_cycle_tables=count,
                switched_tables_checked=swaps,minimum_changed_primary_lap=min_lap,
                maximum_changed_primary_lap=max_lap)


if __name__=='__main__':
    print(json.dumps([verify(k) for k in range(1,6)],indent=2))
