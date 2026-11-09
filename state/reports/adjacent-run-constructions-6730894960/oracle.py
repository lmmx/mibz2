"""Untimed mathematical witnesses; not a decoder implementation or benchmark."""
from itertools import product
from collections import Counter
import json

def runs(s):
    starts = [i for i in range(len(s)) if i == 0 or s[i] != s[i-1]]
    return [(p, e-p, s[p]) for p,e in zip(starts, starts[1:]+[len(s)])]

def data(s):
    n=len(s); rs=runs(s); counts=Counter(s)
    starts={c:sum(v for d,v in counts.items() if d<c) for c in counts}
    nxt=starts.copy(); psi=[None]*n; lf=[None]*n
    for p,c in enumerate(s):
        q=nxt[c]; nxt[c]+=1; psi[q]=p; lf[p]=q
    nxt=starts.copy(); direct=[None]*n; intervals=[]
    for p,k,c in rs:
        a=nxt[c]; nxt[c]+=k; intervals.append((a,k,p))
        for t in range(k): direct[a+t]=(s[a+t],p+t)
    assert direct==list(zip(s,psi))
    # One L predecessor per nonempty F bucket, seeded in monotone bucket order.
    cursor={}; j=0
    for c in sorted(starts):
        while j+1<len(rs) and rs[j+1][0]<=starts[c]: j+=1
        cursor[c]=j
    resumed=[None]*n; advances=0
    for (p,k,c),(a,_,_) in zip(rs,intervals):
        q=a
        while q<a+k:
            j=cursor[c]
            while j+1<len(rs) and rs[j+1][0]<=q:
                j+=1; advances+=1
            cursor[c]=j
            end=min(a+k,rs[j][0]+rs[j][1])
            for t in range(end-q): resumed[q+t]=(rs[j][2],p+q-a+t)
            q=end
    assert resumed==direct
    assert advances<=len(rs)-1
    # Union partitions captures exactly the sorted fill's cells.
    bounds=sorted({0,n}|{p for p,_,_ in rs}|{a for a,_,_ in intervals})
    cells=[b-a for a,b in zip(bounds,bounds[1:])]
    return dict(runs=rs, intervals=intervals, cells=cells,
                W=sum(8*((x+7)//8) for x in cells),
                aligned16=sum(len(set(s[i:i+16]))==1 for i in range(0,n-15,16)),
                lf=lf, table=direct)

def signature(s,d):
    return (len(s),tuple(sorted(Counter(s).items())),
            tuple(sorted(k for _,k,_ in d['runs'])))

seen={}; collision=None; checked=0
valid_seen={}; valid_collision=None
for n in range(1,9):
    for s in product(range(3),repeat=n):
        d=data(s); checked+=1
        k=signature(s,d)
        if k in seen and seen[k][1]['W'] != d['W'] and collision is None:
            old,od=seen[k]
            collision={'a':old,'b':s,'a_cells':od['cells'],'b_cells':d['cells'],
                       'a_W':od['W'],'b_W':d['W'],'signature':k}
        else: seen[k]=(s,d)
        rots=sorted(s[i:]+s[:i] for i in range(n))
        col=tuple(x[-1] for x in rots)
        cd=data(col); ck=signature(col,cd)
        # The alternative orientation writes LF at each L row, then reconstructs backwards.
        primary=rots.index(s); row=primary; restored=[None]*n
        for pos in range(n-1,-1,-1):
            restored[pos]=col[row]; row=cd['lf'][row]
        assert tuple(restored)==s
        if ck in valid_seen and valid_seen[ck][2]['W']!=cd['W'] and valid_collision is None:
            ot,oc,od=valid_seen[ck]
            valid_collision={'text_a':ot,'text_b':s,'L_a':oc,'L_b':col,
                             'a_cells':od['cells'],'b_cells':cd['cells'],
                             'a_W':od['W'],'b_W':cd['W'],'signature':ck}
        else: valid_seen[ck]=(s,col,cd)

a=[0]*16+[1]+[0]+[1]*16
b=[0]+[1]*16+[0]*16+[1]
da,db=data(a),data(b)
assert signature(a,da)==signature(b,db)
assert (da['aligned16'],db['aligned16'])==(1,0)

# Invalid proposed transfer: blindly round each cell to eight writes in L-run order.
s=[1,1,0]; d=data(s); bad=[None]*(len(s)+8)
for a,k,p in d['intervals']:
    q=a
    while q<a+k:
        e=q+1
        while e<a+k and s[e]==s[q]: e+=1
        for t in range(8*((e-q+7)//8)):
            bad[q+t]=(s[q],p+q-a+t)
        q=e
assert bad[:len(s)]!=d['table']

# A valid periodic cyclic BWT need not have a single LF cycle.
t='abab'; rotations=sorted(t[i:]+t[:i] for i in range(len(t)))
column=''.join(x[-1] for x in rotations)
lf=data(list(map(ord,column)))['lf']; walk=[0]
for _ in range(len(t)): walk.append(lf[walk[-1]])
assert len(set(walk[:-1]))==2

print(json.dumps({'columns_checked':checked,'reverse_LF_texts_checked':checked,'same_histogram_cell_witness':collision,
 'valid_cyclic_BWT_cell_witness':valid_collision,
 'same_histogram_alignment_witness':{'a':'a^16 b a b^16','b':'a b^16 a^16 b',
 'a_aligned16':da['aligned16'],'b_aligned16':db['aligned16']},
 'blind_overstore_witness':{'L':s,'expected':d['table'],'observed':bad[:len(s)]},
 'periodic_bwt':{'text':t,'L':column,'LF':lf,'walk':walk}},indent=2))
