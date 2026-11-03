#!/usr/bin/env python3
"""Independent mathematical/state-machine probes for mibz2 compact30 constructor.

NOT a run of ctor.rs and NOT a Rust memory-safety test. Builds an instrumented
state-machine from the frozen algorithm, compares against an independently
indexed oracle, and flags reads of abstractly uninitialised headers/fields.
No benchmarks, timing or repository changes.
"""
import json
import random
from collections import Counter

ROWS=30
MAX_OFF=65533
RAWF=65535
RAW_BIT=1<<28
M20=(1<<20)-1
MAX_N=ROWS*(M20//32)
MASK64=(1<<64)-1

def phi(x): return 32*(x//ROWS)+x%ROWS
def phi_inv(x): return 30*(x//32)+x%32
def mix(x):
    x &= MASK64
    x ^= x>>33; x=(x*0xff51afd7ed558ccd)&MASK64
    x ^= x>>33; x=(x*0xc4ceb9fe1a85ec53)&MASK64
    return x^(x>>33)
def force(g,n,pm): return pm>0 and (mix((g ^ (n<<32)) & MASK64)%1000)<pm
def counts_for(ll):
    z=Counter(ll)
    return [z.get(c,0) for c in range(256)]
def rec(h=None,fields=None): return {'h':h,'f':[None]*ROWS if fields is None else list(fields)}
def canon(recs,bitmap,raw):
    ranks={};rank=0
    for g,flag in enumerate(bitmap):
        if not flag: continue
        assert recs[g] is not None and recs[g]['h'] is not None
        old=recs[g]['h']&M20
        assert old < len(raw)
        assert old not in ranks, 'duplicate provisional RAW index'
        assert raw[old][31] == 0
        ranks[old]=rank
        raw[old][31]=rank
        recs[g]['h']=RAW_BIT|rank
        rank+=1
    assert rank==len(raw) and set(ranks)==set(range(len(raw)))
    swaps=0
    for s in range(len(raw)):
        if raw[s][31]==s:continue
        carry=raw[s].copy()
        steps=0
        while True:
            d=carry[31]
            assert 0<=d<len(raw)
            carry,raw[d]=raw[d],carry
            steps+=1
            assert steps<=len(raw), 'canonicalization cycle did not terminate'
            if d==s:break
        swaps+=steps
    for k,v in enumerate(raw):
        assert v[31]==k
        assert v[30]==0
        v[31]=0
    return swaps

def oracle(ll,counts,pm):
    n=len(ll)
    if n==0: return [rec(RAW_BIT|M20,[RAWF]*ROWS)],[]
    starts=[0]*256;cum=0
    for c in range(256): starts[c]=cum; cum+=counts[c]
    assert cum==n
    dest=starts.copy()
    psi=[None]*n; out=[None]*n
    for i,c in enumerate(ll):
        j=dest[c]
        if j>=n: return None
        psi[j]=phi(i);out[j]=c;dest[c]+=1
    if any(x is None for x in psi): return None
    ret=[];raw=[]
    for g in range((n+29)//30):
        a=30*g;b=min(a+30,n)
        crossed=any(x!=out[a] for x in out[a:b])
        wide=psi[b-1]-psi[a]>MAX_OFF
        israw=crossed or wide or force(g,n,pm)
        if israw:
            ret.append(rec(RAW_BIT|len(raw),[RAWF]*ROWS))
            gr=[0]*32
            for j in range(b-a):gr[j]=(psi[a+j]<<8)|out[a+j]
            raw.append(gr)
        else:
            h=(psi[a]<<8)|out[a]
            fs=[psi[i]-psi[a] for i in range(a,b)]+[RAWF]*(a+ROWS-b)
            assert all(0<=x<=MAX_OFF for x in fs[:b-a])
            ret.append(rec(h,fs))
    ret.append(rec(RAW_BIT|M20,[RAWF]*ROWS))
    return ret,raw

class InstrumentedCtor:
    def __init__(self):
        self.events=Counter()
    def build(self,ll,counts,pm):
        n=len(ll); G=(n+29)//30
        if n>=1<<24 or n>MAX_N or sum(counts)!=n:
            return None
        starts=[];ends=[];cum=0
        for c in range(256):
            starts.append(phi(cum));cum+=counts[c];ends.append(phi(cum))
        x=starts.copy();xl=ends
        R=[None]*(G+1);RAW=[];bits=[False]*G
        y=0;p=0
        def new_raw(g):
            assert 0<=g<G and not bits[g]
            k=len(RAW); RAW.append([0]*32)
            R[g]=rec(RAW_BIT|k,[RAWF]*ROWS);bits[g]=True
            self.events['new_raw']+=1
            return k
        def head(g):
            assert 0<=g<G and R[g] is not None and R[g]['h'] is not None,'unwritten header read'
            self.events['header_reads']+=1
            return R[g]['h']
        def fields(g):
            assert 0<=g<G
            if R[g] is None:R[g]=rec()
            return R[g]['f']
        def advance(k):
            nonlocal y,p
            assert 1<=k<=30
            p+=k
            if p>=30:
                p-=30;y+=k+2
            else: y+=k
        def promote(g,j,h):
            assert 1<=j<30 and not bits[g]
            f=fields(g)
            prev=[]
            for t in range(j):
                assert f[t] is not None, 'unwritten earlier field read in promote'
                prev.append(f[t])
            idx=new_raw(g)
            for t,off in enumerate(prev): RAW[idx][t]=(((h>>8)+off)<<8)|(h&255)
            self.events['promotions']+=1
            return idx
        def pattern(k):
            return [t + 2*(p+t>=30) for t in range(k)]
        def fragment(c,g,j,k):
            nonlocal y,p
            assert 0<=g<G and 0<=j<30 and 1<=k<=30-j
            pat=pattern(k)
            if j==0:
                if bits[g]: idx=head(g)&M20
                else:
                    if R[g] is None:R[g]=rec()
                    assert R[g]['h'] is None
                    R[g]['h']=(y<<8)|c
                    for t,o in enumerate(pat):
                        assert R[g]['f'][t] is None
                        R[g]['f'][t]=o
                    idx=None
            else:
                h=head(g)
                if h&RAW_BIT: idx=h&M20
                else:
                    assert (h&255)==c,'different byte in ordinary group'
                    assert y>=(h>>8), 'nonmonotone source within a bucket'
                    d=y-(h>>8)
                    if d+pat[-1]>MAX_OFF:idx=promote(g,j,h)
                    else:
                        f=fields(g)
                        for t,o in enumerate(pat):
                            assert f[j+t] is None
                            assert 0<=o+d<=MAX_OFF
                            f[j+t]=o+d
                        idx=None
            if idx is not None:
                assert idx<len(RAW)
                for t,o in enumerate(pat):
                    assert RAW[idx][j+t]==0  # RAW entries written once, includes zero-value entry
                    RAW[idx][j+t]=((y+o)<<8)|c
            advance(k)
        def complete(c,g,whole,checked):
            nonlocal y,p
            assert 0<=g<=G and g+whole<=G
            for t in range(whole):
                if checked and bits[g+t]:fragment(c,g+t,0,30)
                else:
                    assert R[g+t] is None,'complete wrote an already-touched group'
                    pat=pattern(30)
                    R[g+t]=rec((y<<8)|c,pat)
                    y+=32
                    self.events['complete']+=1
        def run(c,m):
            nonlocal y,p
            xx=x[c];j=xx%32;tot=j+m;q=tot//30
            xend=xx+m+2*q
            if xend>xl[c]:return False
            x[c]=xend
            g=xx//32;whole=q
            if j:
                k=m if q==0 else 30-j
                fragment(c,g,j,k)
                if q==0:return True
                g+=1;whole-=1
            if whole:
                complete(c,g,whole,pm>0)
                g+=whole
            r=tot-30*q
            if r:fragment(c,g,0,r)
            return True
        def one(c):
            nonlocal y,p
            xx=x[c]
            if xx>=xl[c]:return False
            x[c]=xx+1+2*(xx%32==29)
            g=xx//32;j=xx%32
            assert 0<=g<G and j<30
            e=(y<<8)|c
            if j==0:
                if bits[g]:idx=head(g)&M20
                else:
                    if R[g] is None:R[g]=rec()
                    assert R[g]['h'] is None and R[g]['f'][0] is None
                    R[g]['h']=e;R[g]['f'][0]=0
                    idx=None
            else:
                h=head(g)
                if h&RAW_BIT:idx=h&M20
                else:
                    assert (h&255)==c
                    assert y>=h>>8
                    d=y-(h>>8)
                    if d>MAX_OFF:idx=promote(g,j,h)
                    else:
                        f=fields(g);assert f[j] is None;f[j]=d;idx=None
            if idx is not None:
                assert RAW[idx][j]==0
                RAW[idx][j]=e
            advance(1)
            return True
        cum=0;last=-1
        for count in counts:
            cum+=count
            if 0<cum<n and cum%30:
                g=cum//30
                if g!=last:new_raw(g);last=g
        if pm:
            for g in range(G):
                if not bits[g] and force(g,n,pm):new_raw(g)
        i=0
        while i+16<=n:
            chunk=ll[i:i+16]
            if all(b==chunk[0] for b in chunk):
                e=i+16
                while e+16<=n and all(z==chunk[0] for z in ll[e:e+16]):e+=16
                if not run(chunk[0],e-i):return None
                i=e
            else:
                for c in chunk:
                    if not one(c):return None
                i+=16
        for c in ll[i:]:
            if not one(c):return None
        if x!=xl:return None
        if G:
            used=n-30*(G-1)
            if used<30:
                assert R[G-1] is not None
                for j in range(used,30):R[G-1]['f'][j]=RAWF
        R[G]=rec(RAW_BIT|M20,[RAWF]*ROWS)
        for g in range(G):
            assert R[g] is not None and R[g]['h'] is not None,'uninitialised header at publication'
            assert all(o is not None for o in R[g]['f']),'uninitialised field at publication'
        for rr in RAW:assert rr[30]==rr[31]==0
        canon(R,bits,RAW)
        return R,RAW

def representation(v):
    if v is None:return None
    r,raw=v
    return ([(x['h'],tuple(x['f'])) for x in r],[tuple(x) for x in raw])

def test_cursor_arithmetic():
    checked=0
    for start in range(0,95):
        for m in range(1,100):
            x=phi(start);j=x%32;q=(j+m)//30
            assert x+m+2*q==phi(start+m),(start,m)
            checked+=1
    return checked

def test_case(ctor,ll,pm=0,counts=None):
    if counts is None:counts=counts_for(ll)
    z=ctor.build(ll,counts,pm)
    if counts==counts_for(ll) and len(ll)<=MAX_N:
        q=oracle(ll,counts,pm)
        assert representation(z)==representation(q), ('candidate != oracle',len(ll),pm,ctor.events)
    else:
        assert z is None, ('malformed counts unexpectedly accepted',len(ll),pm)
    return z

def target_span(target,phase):
    src=phase
    end_label=phi(src)+target
    if end_label%32>=30:return None
    end=phi_inv(end_label)
    if end<=src+60:return None
    n=end+50
    ll=bytearray(2+(i&1) for i in range(n))
    marks=[src+2*t for t in range(29)]+[end]
    if len(set(marks))!=30:return None
    for x in marks:ll[x]=1
    return bytes(ll)

def main():
    randomizer=random.Random(0xC30A11D17)
    ctor=InstrumentedCtor()
    cursor=test_cursor_arithmetic()
    small=0
    for n in range(0,91):
        for trial in range(4):
            values=[0,1,2,3,55,255][:1+trial%6]
            ll=bytes(randomizer.choice(values) for _ in range(n))
            for pm in (0,50,1000):test_case(ctor,ll,pm);small+=1
    malformed=0
    for n in (1,29,30,31,59,60,61,90,300,2048,70000):
        ll=bytes(randomizer.randrange(7) for _ in range(n))
        own=counts_for(ll)
        for t in range(10):
            c=own.copy()
            old=randomizer.choice([i for i,v in enumerate(c) if v>0]);new=randomizer.choice([i for i in range(256) if i!=old]);c[old]-=1;c[new]+=1
            test_case(ctor,ll,(0,50,1000)[t%3],c);malformed+=1
    spans={}
    for target in (65530,65531,65532,65533,65534,65535,65536,65538):
        cases=0
        for phase in (0,3,7,14,21,29):
            ll=target_span(target,phase)
            if ll is None:continue
            before=ctor.events['promotions']
            z=test_case(ctor,ll,0)
            assert z is not None
            # Exactly one 30-row group of byte 1; its span is target.
            assert ctor.events['promotions']-before==int(target>MAX_OFF)
            cases+=1
        assert cases>0
        spans[str(target)]=cases
    # canonicalisation: independently constructed arbitrary permutations, including full cycles
    cycles=0
    for count in (0,1,2,3,5,30,64,101,257,1200):
        for kind in ('identity','reverse','rotate','random','cycle'):
            ranks=list(range(count))
            if kind=='reverse':ranks.reverse()
            if kind=='rotate' and count:ranks=ranks[1:]+ranks[:1]
            if kind=='random':randomizer.shuffle(ranks)
            if kind=='cycle' and count>1:ranks=[(i+1)%count for i in range(count)]
            recs=[rec(RAW_BIT|r,[RAWF]*30) for r in ranks]
            raw=[[0]*32 for _ in range(count)]
            for r,p in enumerate(ranks):raw[p][0]=r+1
            canon(recs,[True]*count,raw)
            assert [a[0] for a in raw]==list(range(1,count+1))
            assert [r['h'] for r in recs]==[RAW_BIT|i for i in range(count)]
            cycles+=1
    # MAX_N and reuse-shaped groups
    reuse=0
    for n in (MAX_N, 1, 31, 0, 65_537, 7):
        ll=bytes([7])*n if n in (MAX_N, MAX_N-10, 0,7) else bytes((i%17) for i in range(n))
        for pm in ((0,) if n>=65536 else (0,1000)):
            test_case(ctor,ll,pm);reuse+=1
    print(json.dumps({'frozen':'e003a32','tests_are':'Python independent instrumented model, not Rust or Miri',
       'cursor_equalities':cursor,'small_paired_oracle_cases':small,'malformed_equal_total_rejections':malformed,
       'span_boundary_cases':spans,'canonical_permutation_cases':cycles,'large_and_reuse_cases':reuse,
       'events':dict(ctor.events),'result':'all independent model assertions passed'},indent=2,sort_keys=True))

if __name__=='__main__':main()
