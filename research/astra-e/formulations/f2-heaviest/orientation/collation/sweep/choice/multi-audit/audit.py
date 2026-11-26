"""Independent exact dictionaries for occurrence-specific, bounded movements of stable cyclic BWT cuts."""
from collections import Counter
from itertools import product
from json import dumps


def nonempty_proper(w):
    return {w[i:] for i in range(1,len(w))}


def states(words):
    return {b""}.union(*(nonempty_proper(w) for w in words))


def section(last,a,b):
    psi=sorted(range(len(last)),key=last.__getitem__)
    first=bytes(last[x] for x in psi)
    words=[];tau=[]
    for p in range(a,b):
        row=p;w=bytearray()
        while True:
            w.append(first[row]);row=psi[row]
            if a<=row<b:break
        words.append(bytes(w));tau.append(row-a)
    cycles=[];seen=set()
    for p in range(b-a):
        if p in seen:continue
        row=p;cycle=[]
        while row not in seen:
            seen.add(row);cycle.append(row);row=tau[row]
        assert row==p
        cycles.append(cycle)
    return psi,first,words,tau,cycles


def families(words,p,h):
    u=set().union(*(nonempty_proper(w[h:]) for w in words))
    forced={p[j:h] for j in range(h)}
    opts=[{s+p[:j] for j in range(h+1)} for s in u]
    disjoint=sum(map(len,opts))==len(set().union(*opts))
    avoids=all(not(f & forced) for f in opts)
    terminal=all(s[-1] not in p[:h] for s in u)
    assert not terminal or disjoint and avoids
    return u,forced,opts,disjoint,avoids,terminal


def rotations(words,p,h):
    return [w[h:]+p[:h] for w in words]


def inspect(last,a,b,count,witness):
    psi,first,words,tau,cycles=section(last,a,b)
    common=0
    while all(len(w)>common and w[common]==words[0][common] for w in words):
        common+=1
    for k in range(2,common+1):
        p=words[0][:k]
        full=len(states(rotations(words,p,k)))
        uniform=[len(states(rotations(words,p,h))) for h in range(k+1)]
        assert all(x>=y for x,y in zip(uniform,uniform[1:]))
        artifacts=[families(words,p,h) for h in range(k+1)]
        first_index={x:p.index(x)+1 for x in set(p)}
        one_pass=all(w[-1] not in first_index or len(w)<=first_index[w[-1]]+1 for w in words)
        assert one_pass == all(f[3] and f[4] for f in artifacts[1:])
        assert one_pass == all(f[5] for f in artifacts[1:])
        count["all_h_certificate"]+=one_pass
        if len(set(words))>1 and artifacts[k][3] and artifacts[k][4] and not artifacts[k][5]:
            if "last_byte_test_not_necessary" not in witness:
                witness["last_byte_test_not_necessary"]=dict(L=list(last),U=[a,b],P=list(p),
                    old_words=[list(w) for w in words],uniform=uniform,
                    tail_suffixes=[list(v) for v in sorted(artifacts[k][0])])
        count["sections_and_prefix_lengths"]+=1
        if len(cycles)>1:count["multicycle_section_lengths"]+=1
        outcomes=[]
        for raw in product(range(k+1),repeat=b-a):
            # Price actual positive arcs through the moved rank set, independent of the algebraic formula.
            where=[]
            for i,t in enumerate(raw):
                row=a+i
                for _ in range(t):row=psi[row]
                where.append(row)
            positions=set(where)
            if len(positions)!=len(where):continue
            z=[]
            for i,row in enumerate(where):
                spelling=bytearray()
                while True:
                    spelling.append(first[row]);row=psi[row]
                    if row in positions:break
                assert row==where[tau[i]]
                z.append(bytes(spelling))
            assert z==[w[t:]+p[:raw[tau[i]]] for i,(w,t) in enumerate(zip(words,raw))]
            adjusted=list(raw)
            removed=False
            for cyc in cycles:
                if all(raw[i]==k==len(words[i]) for i in cyc):
                    for i in cyc:adjusted[i]=0
                    removed=True
            assert z==[w[t:]+p[:adjusted[tau[i]]] for i,(w,t) in enumerate(zip(words,adjusted))]
            h=max(adjusted)
            s=states(z);q=len(s)
            if removed:count["removable_cycle_cases"]+=1
            if removed and 0<h<k:count["removal_drops_max_to_positive"]+=1
            if h:
                u,forced,opts,disjoint,avoids,_=artifacts[h]
                assert forced<=s and all(f&s for f in opts)
                assert any(len(w)>h for w in words)
                assert uniform[h]==1+h+len(u)
                if disjoint and avoids:assert q>=uniform[h]>=full
                # A strictly weaker sufficient condition: every family admits
                # its own mandatory contribution after charging the forced set.
                if disjoint:
                    assert q>=1+h+sum(not(f&forced) for f in opts)
            else:
                assert q==uniform[0]>=full
            count["positive_assignments"]+=1
            if h and not removed and raw!=tuple([h]*len(raw)):count["mixed_positive"]+=1
            outcomes.append(q)
            def record(label):
                if label not in witness:witness[label]=dict(L=list(last),U=[a,b],P=list(p),
                    old_words=[list(w) for w in words],tau=tau,old_cycles=cycles,t=raw,effective_h=h,
                    shifted_words=[list(v) for v in z],Q=q,uniform=uniform,
                    family_disjoint_by_h=[f[3] for f in artifacts[1:]],
                    avoids_forced_by_h=[f[4] for f in artifacts[1:]])
            if removed and 0<h<k: record("reset_special_cycle_then_use_lower_max")
            if q<full:
                count["better_than_uniform_k"]+=1;record("mixed_beats_uniform_k")
                if h and artifacts[h][3] and not artifacts[h][4]:record("avoidance_needed")
                if h and not artifacts[h][3] and artifacts[h][4]:record("family_overlap_needed")
                if h<k and artifacts[k][3] and artifacts[k][4]:record("checking_only_k_fails")
        if min(outcomes)==full and any(not(f[3] and f[4]) for f in artifacts[1:]):
            if len(set(words))>1 and "disjointness_not_necessary" not in witness:
                witness["disjointness_not_necessary"]=dict(L=list(last),U=[a,b],P=list(p),
                    old_words=[list(w) for w in words],uniform=uniform,optimum=min(outcomes),
                    family_disjoint_by_h=[f[3] for f in artifacts[1:]],avoids_forced_by_h=[f[4] for f in artifacts[1:]])


def main():
    count=Counter();witness={}
    for n in range(1,11):
        for alphabet in (2,3):
            if alphabet==3 and n>7:continue
            for digits in product(range(alphabet), repeat=n):
                if alphabet==3 and 2 not in digits:continue
                last=bytes(digits);count["columns"]+=1
                for a in range(n):
                    for b in range(a+1,n+1):
                        inspect(last,a,b,count,witness)
    print(dumps(dict(counts=count,witnesses=witness),indent=2))


if __name__=="__main__":
    main()
