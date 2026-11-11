"""First-return construction on delimiter rows directly from run-level LF.

This tests the class-induced exchange already flagged in mibz2's research, now
with an explicit phrase dictionary / token-BWT interpretation. Counts, not speed.
"""
from batch_induction import *
from collections import deque

def induce_on_byte(L: bytes, p0: int, delimiter: int, *, step_limit: int|None=None):
    P,G=from_column(L); starts=[a.s for a in P]
    anchors=[a for a in P if a.w==delimiter]
    if not anchors: return None
    ast=[a.s for a in anchors]; acc=[0]
    for a in anchors: acc.append(acc[-1]+a.n)
    def rank(x):
        k=bisect_right(ast,x)-1
        assert k>=0 and x<anchors[k].s+anchors[k].n
        return acc[k]+x-anchors[k].s
    # The selected cycle must encounter an anchor; reject if it does not.
    p=p0; shift=0
    while True:
        a=P[bisect_right(starts,p)-1]
        if a.w==delimiter: break
        p=a.t+p-a.s; shift+=1
        if p==p0: return None
    q0=rank(p)
    # (source rank within anchor set, current coordinate, width, forward word ID)
    queue=deque((acc[k],a.s,a.n,None) for k,a in enumerate(anchors))
    completed=[]; steps=0; weighted=0; peak=len(queue)
    while queue:
        s,x,width,w=queue.popleft()
        a=P[bisect_right(starts,x)-1]
        assert x+width<=a.s+a.n
        steps+=1; weighted+=width
        if step_limit is not None and steps>step_limit: return {'aborted':True,'steps':steps}
        w=a.w if w is None else G.cat(w,a.w)
        y=a.t+x-a.s; end=y+width; j=bisect_right(starts,y)-1
        while y<end:
            b=P[j]; v=min(end,b.s+b.n)-y
            ss=s+(y-(a.t+x-a.s))
            if b.w==delimiter:
                completed.append(Piece(ss,rank(y),v,w))
            else:
                queue.append((ss,y,v,w))
            y+=v; j+=1
        peak=max(peak,len(queue))
    completed.sort(key=lambda a:a.s)
    merged=[]
    for a in completed:
        if merged and merged[-1].s+merged[-1].n==a.s and merged[-1].t+merged[-1].n==a.t and merged[-1].w==a.w:
            b=merged[-1]; merged[-1]=Piece(b.s,b.t,b.n+a.n,b.w)
        else: merged.append(a)
    validate(merged)
    # Check that it really is LF of the preceding phrase IDs, with tokens ordered
    # lexicographically by the forward-text phrase (reverse of LF's word).
    words={a.w:G.expand(a.w)[::-1] for a in merged}
    tokens=sorted(words,key=words.get); counts=Counter()
    for a in merged: counts[a.w]+=a.n
    curs={}; z=0
    for w in tokens: curs[w]=z; z+=counts[w]
    is_token_lf=True
    for a in merged:
        if a.t!=curs[a.w]: is_token_lf=False
        curs[a.w]+=a.n
    # Recover the selected cycle with correct origin, including a short cycle.
    st=[a.s for a in merged]; q=q0; E=bytearray()
    while True:
        a=merged[bisect_right(st,q)-1]; E.extend(G.expand(a.w)); q=a.t+q-a.s
        if q==q0: break
    R=bytes(reversed(E)); shift%=len(R); R=R[shift:]+R[:shift]
    got=(R*((len(L)+len(R)-1)//len(R)))[:len(L)]
    assert got==row_oracle(L,p0)
    return dict(steps=steps,weighted_rows=weighted,anchor_rows=acc[-1],anchor_intervals=len(anchors),return_pieces=len(merged),dictionary_words=len(words),dictionary_bytes=sum(map(len,words.values())),grammar_nodes=len(G.nodes)-256,peak_frontier=peak,token_lf=is_token_lf,origin_prefix_steps=shift)

def tests():
    count=0; declined=0
    for n in range(1,9):
        for tup in itertools.product((65,66),repeat=n):
            L=bytes(tup)
            for p in range(n):
                for delimiter in (65,66):
                    a=induce_on_byte(L,p,delimiter)
                    if a is None: declined+=1
                    else: count+=1; assert a['token_lf']
    return count,declined

def run():
    accepted,rejected=tests(); print('phrase tests',accepted,'fallback',rejected,flush=True)
    rows=[]
    for name,s in fixtures().items():
        L,p=bwt(s); r=len(from_column(L)[0]); S=reference_rauzy(L)
        choices=list(dict.fromkeys([c for c in (10,32,34,44,58,123,125,65,66) if c in L]))
        for d in choices:
            a=induce_on_byte(L,p,d)
            assert a is not None and a['token_lf']
            row=dict(name=name,n=len(L),r=r,S=S,delimiter=d,**a); rows.append(row)
        relevant=[x for x in rows if x['name']==name]
        if relevant:
            best=min(relevant,key=lambda x:x['steps']+x['anchor_rows'])
            print(name,'d',repr(chr(best['delimiter'])),'steps',best['steps'],'tokens',best['anchor_rows'],'pieces',best['return_pieces'],'dict bytes',best['dictionary_bytes'],'S',S,flush=True)
    pathlib.Path('phrase_results.json').write_text(json.dumps(dict(correctness_cases=accepted,fallback_cases=rejected,census=rows),indent=2))

if __name__=='__main__': run()
