"""Experimental simultaneous first-return contraction of a discrete interval exchange.

Reference research code, NOT a fast decoder. Its construction stores interval
records and a word DAG, rather than a permutation entry per row. Sorting and
binary searches are counted honestly; Python timings are not performance evidence.
No modifications to mibz2 are made.
"""
from __future__ import annotations
from dataclasses import dataclass
from bisect import bisect_right
from collections import Counter
import random, json, itertools, pathlib, hashlib, argparse, time

@dataclass(frozen=True)
class Piece:
    s: int             # source interval start
    t: int             # image interval start
    n: int             # common length
    w: int             # forward-orbit emission word, grammar ID

class Grammar:
    def __init__(self):
        self.nodes: list[int|tuple[int,int]] = list(range(256))
        self.length = [1]*256
        self.ids = {}
    def cat(self, a: int, b: int) -> int:
        key=(a,b)
        if key not in self.ids:
            self.ids[key]=len(self.nodes)
            self.nodes.append(key)
            self.length.append(self.length[a]+self.length[b])
        return self.ids[key]
    def expand(self, w: int) -> bytes:
        out=bytearray(); st=[w]
        while st:
            x=st.pop(); v=self.nodes[x]
            if isinstance(v,int): out.append(v)
            else: st.extend((v[1],v[0]))
        return bytes(out)

def bwt(s: bytes) -> tuple[bytes,int]:
    """Cyclic doubling, used ONLY to make test fixtures from known original text."""
    n=len(s)
    if not n: raise ValueError('nonempty input required')
    order=list(range(n)); rank=list(s); k=1
    while k<n:
        order.sort(key=lambda i:(rank[i],rank[(i+k)%n]))
        r=[0]*n
        for j in range(1,n):
            a,b=order[j-1],order[j]
            r[b]=r[a]+((rank[a],rank[(a+k)%n])!=(rank[b],rank[(b+k)%n]))
        rank=r; k*=2
        if rank[order[-1]]==n-1: break
    # Stable sorting of tied rotations is immaterial to original bytes.
    return bytes(s[(i-1)%n] for i in order),order.index(0)

def from_column(L: bytes) -> tuple[list[Piece],Grammar]:
    counts=Counter(L); cursor={}; pos=0
    for c,v in sorted(counts.items()): cursor[c]=pos; pos+=v
    pieces=[]; s=0
    while s<len(L):
        e=s+1
        while e<len(L) and L[e]==L[s]: e+=1
        c=L[s]; pieces.append(Piece(s,cursor[c],e-s,c)); cursor[c]+=e-s; s=e
    return pieces,Grammar()

def row_oracle(L: bytes, p: int) -> bytes:
    """Existing inverse-LF semantics, including arbitrary columns and short cycles."""
    pieces,_=from_column(L); inv=[0]*len(L)
    for a in pieces:
        for j in range(a.n): inv[a.t+j]=a.s+j
    out=bytearray()
    for _ in L:
        p=inv[p]; out.append(L[p])
    return bytes(out)

def validate(P: list[Piece]):
    z=0
    for a in P:
        assert a.s==z and a.n>0; z+=a.n
    v=0
    for a in sorted(P,key=lambda x:x.t):
        assert a.t==v; v+=a.n
    assert v==z

def decode(P: list[Piece], G: Grammar, p: int, n: int) -> bytes:
    """Slow exact expansion of the selected cycle, then inversion of orientation.

    The marked starting row is never removed by contraction. Forward LF emissions
    from it form E; reversed E is exactly the inverse-LF output cycle. Repeat/truncate
    to n, even for arbitrary columns whose selected cycle length does not divide n.
    """
    starts=[a.s for a in P]; root=p; E=bytearray()
    while True:
        a=P[bisect_right(starts,p)-1]
        E.extend(G.expand(a.w)); p=a.t+p-a.s
        if p==root: break
        if len(E)>n: raise AssertionError('cycle longer than original domain')
    R=bytes(reversed(E))
    return (R*((n+len(R)-1)//len(R)))[:n]

def color(w: int, seed: int) -> int:
    # SplitMix64; deterministic across processes. Equal word IDs get equal colors.
    x=(w+seed*0x9e3779b97f4a7c15)&((1<<64)-1)
    x=((x^(x>>30))*0xbf58476d1ce4e5b9)&((1<<64)-1)
    x=((x^(x>>27))*0x94d049bb133111eb)&((1<<64)-1)
    return (x^(x>>31))&1

def contract(P: list[Piece], G: Grammar, p: int, seed: int):
    """Contract all edges whose word colors are 0 -> 1, except into marked p.

    D is the set of destination vertices of these edges. D and f(D) are disjoint.
    The induced map is f on other surviving edges, and f^2 on contracted edges.
    Pair words are concatenated in f-orbit order. Interval geometry is exact.
    """
    # Overlay images (sorted by target) and domains (sorted by source).
    edges=[]; i=j=0; images=sorted(P,key=lambda a:a.t)
    overlaps=0
    while i<len(images) and j<len(P):
        a,b=images[i],P[j]
        lo=max(a.t,b.s); hi=min(a.t+a.n,b.s+b.n)
        if lo<hi:
            overlaps+=1
            cuts=[lo,hi]
            yes=color(a.w,seed)==0 and color(b.w,seed)==1
            if yes and lo<=p<hi: cuts=sorted(set([lo,p,p+1,hi]))
            for yl,yh in zip(cuts,cuts[1:]):
                if yl==yh: continue
                selected=yes and yl!=p
                xs=a.s+yl-a.t
                edges.append((xs,yl,yh-yl,a,b,selected))
        ae,be=a.t+a.n,b.s+b.n
        if ae<=be: i+=1
        if be<=ae: j+=1
    # D is already in ascending coordinate order from the overlay.
    removed=[]
    for _,y,l,_,_,take in edges:
        if take:
            if removed and removed[-1][1]==y: removed[-1]=(removed[-1][0],y+l)
            else: removed.append((y,y+l))
    amount=sum(v-u for u,v in removed)
    if not amount: return P,p,dict(overlaps=overlaps,edges=len(edges),removed=0,output_pieces=len(P),raw_pieces=0)
    ds=[a for a,b in removed]; de=[b for a,b in removed]; pref=[0]
    for a,b in removed: pref.append(pref[-1]+b-a)
    def rank_keep(x):
        i=bisect_right(de,x)
        r=pref[i]
        if i<len(ds) and ds[i]<x: r+=x-ds[i]
        return x-r
    def keep_parts(lo,hi):
        i=bisect_right(de,lo); x=lo
        while i<len(ds) and ds[i]<hi:
            if x<ds[i]: yield x,min(hi,ds[i])
            x=max(x,de[i]); i+=1
        if x<hi: yield x,hi
    raw=[]
    for x,y,l,a,b,take in edges:
        target=b.t+y-b.s if take else y
        w=G.cat(a.w,b.w) if take else a.w
        for lo,hi in keep_parts(x,x+l):
            dest=target+lo-x
            # Image of a retained piece lies entirely in retained rows.
            assert rank_keep(dest+hi-lo)-rank_keep(dest)==hi-lo
            raw.append(Piece(rank_keep(lo),rank_keep(dest),hi-lo,w))
    raw.sort(key=lambda a:a.s)
    out=[]
    for a in raw:
        if out and out[-1].s+out[-1].n==a.s and out[-1].t+out[-1].n==a.t and out[-1].w==a.w:
            b=out[-1]; out[-1]=Piece(b.s,b.t,b.n+a.n,b.w)
        else: out.append(a)
    validate(out)
    return out,rank_keep(p),dict(overlaps=overlaps,edges=len(edges),removed=amount,output_pieces=len(out),raw_pieces=len(raw))

def reference_rauzy(L: bytes) -> int:
    """Unmarked right-end induction cell count; arbitrary columns may drop cycles."""
    P,_=from_column(L); top=list(range(len(P))); bottom=sorted(top,key=lambda i:P[i].t)
    lens=[a.n for a in P]; steps=0
    while top:
        a,b=top[-1],bottom[-1]
        if a==b: top.pop(); bottom.pop(); continue
        steps+=1
        if lens[a]>lens[b]:
            lens[a]-=lens[b]; bottom.pop(); bottom.insert(bottom.index(a)+1,b)
        elif lens[a]<lens[b]:
            lens[b]-=lens[a]; top.pop(); top.insert(top.index(b)+1,a)
        else:
            top.pop(); bottom.pop(); bottom[bottom.index(a)]=b
    return steps

def exhaustive(max_binary=9,rounds=4):
    cases=0
    for n in range(1,max_binary+1):
        for tup in itertools.product((0,1),repeat=n):
            L=bytes(tup)
            for p0 in range(n):
                P,G=from_column(L); p=p0; expect=row_oracle(L,p0)
                assert decode(P,G,p,n)==expect
                for k in range(1,rounds+1):
                    P,p,_=contract(P,G,p,k)
                    assert decode(P,G,p,n)==expect,(L,p0,k)
                cases+=1
    rng=random.Random(72913)
    for i in range(1000):
        n=rng.randrange(1,120); alph=rng.randrange(1,15)
        L=bytes(rng.randrange(alph) for _ in range(n)); p=rng.randrange(n)
        P,G=from_column(L); expected=row_oracle(L,p)
        for k in range(1,rounds+1):
            P,p,_=contract(P,G,p,k+i*rounds)
            assert decode(P,G,p,n)==expected
        cases+=1
    return cases

def fixtures():
    n=32768; rng=random.Random(20261111)
    tm=bytes(i.bit_count()%2+65 for i in range(n))
    f,b=b'A',b'AB'
    while len(b)<n: f,b=b,b+f
    data={
        'thue_morse':tm,
        'fibonacci_prefix':b[:n],
        'near_periodic':(b'ab'*(n//2))[:-1]+b'x',
        'synthetic_json':b''.join(json.dumps({'id':'Q'+str(i),'type':'item','labels':{'en':{'language':'en','value':'Entity '+str(i%127)}},'claims':{'P31':[{'value':'Q'+str(i%31)}] }},separators=(',',':')).encode()+b'\n' for i in range(300))[:n],
        'uniform_4':bytes(rng.randrange(4)+65 for _ in range(n)),
        'uniform_bytes':bytes(rng.randrange(256) for _ in range(n)),
    }
    p=pathlib.Path(__file__).resolve().parent/'fixtures'/'GPL-3.txt'
    if not p.exists():
        p=pathlib.Path('/usr/share/common-licenses/GPL-3')
    if p.exists(): data['gpl3_prose']=p.read_bytes()[:n]
    return data

def census(rounds=12):
    out=[]
    for name,s in fixtures().items():
        L,p=bwt(s); assert row_oracle(L,p)==s
        P,G=from_column(L); r=len(P); S=reference_rauzy(L)
        rec=dict(name=name,n=len(L),r=r,S=S,sha256=hashlib.sha256(s).hexdigest(),rounds=[])
        for k in range(1,rounds+1):
            before=len(P); m=sum(a.n for a in P)
            P,p,stats=contract(P,G,p,k)
            assert decode(P,G,p,len(L))==s
            rec['rounds'].append(dict(round=k,m_before=m,pieces_before=before,m_after=sum(a.n for a in P),grammar_nodes=len(G.nodes)-256,**stats))
        out.append(rec)
        print(name,'n',len(s),'r',r,'S',S,'final m',sum(a.n for a in P),'pieces',len(P),'total overlays',sum(q['overlaps'] for q in rec['rounds']),flush=True)
    return out

if __name__=='__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('--out',default='batch_results.json'); args=parser.parse_args()
    t=time.time(); count=exhaustive(); print('correctness cases',count,'seconds',round(time.time()-t,2),flush=True)
    result=dict(description='Structural counts, not runtime benchmark. No Wikidata corpus available.',correctness_cases=count,census=census())
    pathlib.Path(args.out).write_text(json.dumps(result,indent=2))
