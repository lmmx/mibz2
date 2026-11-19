"""Search common-prefix cylinders with internal 01 forbidden, directly realizable by a one-cycle text."""
from functools import cache
from itertools import product
import json
from audit import section,states,families

P=b"\x00\x01"
CODES={b"":0}
def bits(w):
    return sum(1<<CODES.setdefault(w[j:],len(CODES)) for j in range(1,len(w)+1))

@cache
def shape(w,t,nxt):
    z=w[t:]+P[:nxt]
    return bits(z) if z else -1

def main():
    # Keep every cut a 01-cylinder visit. Strictly less than metered words
    # can be realized by a longer full-text search; this is a targeted witness.
    words=[P+bytes(y) for n in range(5) for y in product((0,2),repeat=n)]
    forced={P,P[1:]}
    suffixes={w:{w[i:] for i in range(3,len(w))} for w in words}
    options={s:{s+P[:j] for j in range(3)} for v in suffixes.values() for s in v}
    count=0
    # Include short bridge arcs, needed for endpoint moves different from 2.
    for m in (3,4):
        for whole in product(words,repeat=m):
            if whole[0]!=min(whole) or len(set(whole))<2:continue
            if m==4 and (whole[0]!=P or sum(map(len,whole))>17):continue
            us=set().union(*(suffixes[w] for w in whole))
            if any(forced & options[s] for s in us):continue
            if sum(len(options[s]) for s in us)==len(set().union(*(options[s] for s in us))):continue
            count+=1
            full=1+2+len(us)
            for t in product(range(3),repeat=m):
                if max(t)!=2 or len(set(t))<2:continue
                mask=0
                for i,w in enumerate(whole):
                    budget=shape(w,t[i],t[(i+1)%m])
                    if budget<0:break
                    mask|=budget
                else:
                    if mask.bit_count()<full:
                        text=b"".join(whole)
                        rank=sorted(range(len(text)),key=lambda j:text[j:]+text[:j])
                        offsets=[sum(map(len,whole[:i])) for i in range(m)]
                        pos=[rank.index(x) for x in offsets]
                        assert max(pos)-min(pos)+1==m
                        last=bytes(text[(j-1)%len(text)] for j in rank)
                        a,b=min(pos),max(pos)+1
                        psi,first,words,tau,_=section(last,a,b)
                        local=[t[pos.index(j)] for j in range(a,b)]
                        dest=[]
                        for i,v in enumerate(local):
                            row=a+i
                            for _ in range(v):row=psi[row]
                            dest.append(row)
                        actual=[]
                        for i,row in enumerate(dest):
                            out=bytearray()
                            while True:
                                out.append(first[row]);row=psi[row]
                                if row in dest:break
                            assert row==dest[tau[i]]
                            actual.append(bytes(out))
                        assert len(states(actual))==mask.bit_count()
                        u,mandatory,opts,disjoint,avoids,_=families(words,P,2)
                        assert not disjoint and avoids and mandatory<=states(actual)
                        print(json.dumps(dict(result="witness",actual_words=[list(w) for w in actual],
                            physical_source_moves=local,family_overlap=sorted([list(z) for z in set().union(*opts)
                            if sum(z in opt for opt in opts)>1]),checked=count,text=list(text),L=list(last),
                            U=[min(pos),max(pos)+1],orbit_words=[list(w) for w in whole],orbit_moves=t,
                            Q=mask.bit_count(),uniform=full),indent=2));return
    print(json.dumps(dict(result="no witness within targeted space",checked=count)))


if __name__=="__main__":
    main()
