"""Independent checked-index model of cells induction and writer reach; not Rust/Miri.

Models slot transitions and the writer's rule-position passes at PR365 57f51bbc.
Oracle is the stable-sort Psi walk, independent of the cells representation.
Run: python replay.py
"""
from collections import Counter
from itertools import product
import json, random

def oracle(a, p):
    psi = sorted(range(len(a)), key=lambda i: a[i])
    q = p
    out = []
    while True:
        q = psi[q]
        out.append(a[q])
        if q == p:
            return bytes(out)

def model(a, primary, omit_recopy=False):
    n = len(a)
    # Words: (actual length, marked, bytes-or-rule-index). Separate identity keeps
    # the model independent of the Rust packed pointer/marker representation.
    words = [(1, i == primary, bytes([c])) for i,c in enumerate(a)]
    letters = []
    i = 0
    while i < n:
        j = i+1
        while j < n and a[j] == a[i]: j += 1
        for lo,hi in ([(i,primary),(primary,primary+1),(primary+1,j)] if i<=primary<j else [(i,j)]):
            if lo < hi: letters.append((hi-lo, words[lo], a[i]))
        i = j
    rr = len(letters)
    order = sorted(range(rr), key=lambda i: letters[i][2])
    # Slots are [predecessor, interval length, word, counterpart].
    slots = {}
    for i,(lam,w,_) in enumerate(letters):
        k = rr+order.index(i)
        slots[i] = [max(i-1,0),lam,w,k]
        slots[k] = [max(k-1,rr),lam,w,i]
    ft,fb = 2*rr, 2*rr+n
    sa,sb = rr-1,2*rr-1
    rules=[]; cells=0; dropped=0
    while True:
        na,ra,aw,bs = slots[sa]
        nb,rb,bw,ts = slots[sb]
        if bs == sb:
            if aw[1]: root=aw; break
            sa,sb=na,nb; dropped+=1
            assert dropped<=rr+n
            continue
        while True:
            assert cells < n
            cells+=1
            assert not (aw[1] and bw[1]), 'duplicated mark'
            ln=aw[0]+bw[0]; marked=aw[1] or bw[1]
            assert ln<=n
            if ln<=16 and not marked:
                assert isinstance(aw[2],bytes) and isinstance(bw[2],bytes)
                cw=(ln,False,aw[2]+bw[2])
            else:
                cw=(ln,marked,len(rules));rules.append((aw,bw))
            if ra==rb:
                slots[ts][1:]=[ra,cw,bs];slots[bs][1:]=[ra,cw,ts]
                sa,sb=na,nb;break
            if ra>rb:
                slots[ts][1:]=[rb,cw,bs]
                slots[fb]=[slots[bs][0],None,None,None]
                slots[bs]=[fb,rb,cw,ts]
                bs=fb;fb+=1;ra-=rb
                nb,rb,bw,ts=slots[nb]
            else:
                slots[bs][1:]=[ra,cw,ts]
                slots[ft]=[slots[ts][0],None,None,None]
                slots[ts]=[ft,ra,cw,bs]
                ts=ft;ft+=1;rb-=ra
                na,ra,aw,bs=slots[na]
    period=root[0]
    trace=Counter(cells=cells,period=period,rules=len(rules),dropped=dropped)
    # Independently flatten root to verify the modeled output and marker rotation.
    def expand(w):
        if isinstance(w[2],bytes):return w[2], (0 if w[1] else None)
        x,y=rules[w[2]];u,mu=expand(x);v,mv=expand(y)
        return u+v,mu if mu is not None else (None if mv is None else len(u)+mv)
    whole,j=expand(root);start=(j+1)%period
    rotated=whole[start:]+whole[:start]
    assert rotated==oracle(a,primary)
    # Literal structural replay of the source's output algorithm. Sentinels detect
    # stale/unwritten bytes; every source/destination range is checked before use.
    pos=[None]*len(rules);out=[None]*period
    def seed(w,q):
        assert not w[1] and w[0]>16
        idx=w[2];assert isinstance(idx,int) and 0<=idx<len(rules)
        pos[idx]=q if pos[idx] is None else min(pos[idx],q)
    def put(q,word):
        assert 0<=q<=period-len(word)
        assert all(v is not None for v in word)
        out[q:q+len(word)]=word
    def spine():
        w,q=root,0
        while isinstance(w[2],int):
            x,y=rules[w[2]]
            if x[1]:plain,at,nxt=y,q+x[0],x
            else:plain,at,nxt=x,q,y
            yield plain,(at+period-start)%period
            if not x[1]:q+=x[0]
            w=nxt
        return
    for w,d in spine():
        if w[0]<=16:put(d,list(w[2]))
        else:seed(w,d);trace['long_spine_seed']+=1
    put((j+period-start)%period,[whole[j]])
    for i in reversed(range(len(rules))):
        q=pos[i]
        if q is None:continue
        trace['position_propagation']+=1
        x,y=rules[i]
        for w,d in [(x,q),(y,q+x[0])]:
            if w[0]>16:
                assert not w[1] and w[2]<i
                seed(w,d)
    for i,(x,y) in enumerate(rules):
        q=pos[i]
        if q is None:continue
        trace['unmarked_rule_expansion']+=1
        for w,d in [(x,q),(y,q+x[0])]:
            if w[0]<=16:put(d,list(w[2]))
            else:
                f=pos[w[2]];assert f is not None and f<=d
                if f!=d:
                    trace['long_factor_copy']+=1
                    trace['overlapping_long_copy']+=int(f+w[0]>d)
                    put(d,out[f:f+w[0]])
    for w,d in spine():
        if w[0]>16:
            f=pos[w[2]];assert f is not None and f<=d
            if f!=d:
                trace['spine_recopy']+=1
                if not omit_recopy:put(d,out[f:f+w[0]])
    assert all(v is not None for v in out), 'unwritten output'
    assert bytes(out)==rotated
    return dict(trace)

def main():
    fixtures=[bytes([1,0]),b'bbaaa',b'abababa',bytes(17),bytes(i%3 for i in range(65))]
    original=[]
    for a in fixtures:
        for p in [0,len(a)//2,len(a)-1]:
            original.append(dict(n=len(a),primary=p,**model(a,p)))
    counts=Counter();witnesses={};cases=0
    def check(a,p):
        nonlocal cases
        t=model(a,p);cases+=1
        for key in ['long_spine_seed','position_propagation','unmarked_rule_expansion','long_factor_copy','spine_recopy','overlapping_long_copy']:
            counts[key]+=t.get(key,0)
            if t.get(key) and key not in witnesses:
                witnesses[key]={'column_hex':a.hex(),'primary':p,**t}
    for n in range(1,8):
        for a in product(range(3),repeat=n):
            for p in range(n):check(bytes(a),p)
    for k in [8,9,16,17,31,32,33,63,64,65,127,128]:
        a=bytes([1])*k+bytes([0])*(k-1)
        for p in [0,k,len(a)-1]:check(a,p)
    rng=random.Random(6782111848)
    for n in [17,32,33,64,65,127,128,255,256]:
        for _ in range(30):
            a=bytes(rng.randrange(4) for _ in range(n))
            for p in [0,n//2,n-1]:check(a,p)
    # A repeated long factor is needed: random columns mostly miss the copy paths.
    s=(b'a'*9+b'b'*9)*3+b'c'
    order=sorted(range(len(s)),key=lambda i:s[i:]+s[:i])
    a=bytes(s[(i-1)%len(s)] for i in order)
    for p in [0,3]:check(a,p)
    try:model(a,0,omit_recopy=True)
    except AssertionError as e:mutant=str(e)
    else:raise AssertionError('model recopy mutation escaped')
    expanded=bytes(c for c in a for _ in range(10))
    # Coalesce and cap at 127 exactly as records, to check admission separately.
    runs=[]
    for c in expanded:
        if runs and runs[-1][0]==c and runs[-1][1]<127:runs[-1][1]+=1
        else:runs.append([c,1])
    admitted=model(expanded,0)
    admitted.update(n=len(expanded),records=len(runs),budget=4*len(expanded)//25)
    assert admitted['records']*10 < len(expanded)
    assert admitted['cells']<=admitted['budget']
    print(json.dumps({'original_miri_writer_fixtures':original,'checked_cases':cases,'writer_reach':counts,'first_witnesses':witnesses,'production_admission_witness':admitted,'model_omit_recopy_mutation':mutant},indent=2))

if __name__=='__main__':main()
