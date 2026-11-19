"""Build return bands from run/interval inputs without expanding a physical event range."""
from bisect import bisect_right
from collections import Counter,deque
from itertools import accumulate
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from rope import Rope,add,join,leaves,size
from transport import contract,anchors_of
from sweep import route_holes

def span(arr,lo,w):
    if w<=0:return
    if arr and arr[-1][0]+arr[-1][1]==lo:
        prev,old=arr[-1];arr[-1]=(prev,old+w)
    else:arr.append((lo,w))

def geometry(runs,start,modal,mark):
    K=len(modal);n=start[-1];freq=[0]*256
    for b,w in runs:freq[b]+=w
    assert sum(freq)==n and len(start)==K+1 and 0<=mark<n
    F=[0,*accumulate(freq)];occ=[0]*256
    stops=[[] for _ in modal];pieces=[[] for _ in modal];D=[None]*K
    births=[];x=c=fragments=0
    def birth(x,y,w):
        span(stops[c],x,w)
        while w:
            j=bisect_right(start,y)-1;take=min(w,start[j+1]-y);off=y-start[j]
            if births:
                old,base,(src,count)=births[-1]
                if old==j and base+count==off and src+count==x:
                    births[-1]=(j,base,(src,count+take))
                else:births.append((j,off,(x,take)))
            else:births.append((j,off,(x,take)))
            x+=take;y+=take;w-=take
    for b,width in runs:
        while width:
            w=min(width,start[c+1]-x);y=F[b]+occ[b];end=x+w
            if b!=modal[c]:birth(x,y,w)
            else:
                j=bisect_right(start,y)-1
                assert y+w<=start[j+1] and D[c] in (None,j);D[c]=j
                lo=y-start[j]
                if x<=mark<end:
                    before=mark-x;span(pieces[c],lo,before);birth(mark,y+before,1)
                    span(pieces[c],lo+before+1,w-before-1)
                else:span(pieces[c],lo,w)
            occ[b]+=w;x=end;width-=w;fragments+=1
            if x==start[c+1]:c+=1
    assert all(v is not None for v in D)
    return D,pieces,stops,births,fragments

class Retained:
    def __init__(self,stops):
        ranges=[]
        for ss in stops:
            for lo,w in ss:span(ranges,lo,w)
        self.lo=[lo for lo,w in ranges];self.w=[w for lo,w in ranges]
        self.base=[0,*accumulate(self.w)]
    def rank(self,x):
        j=bisect_right(self.lo,x)-1;assert j>=0 and x<self.lo[j]+self.w[j]
        return self.base[j]+x-self.lo[j]
    def select(self,j):
        i=bisect_right(self.base,j)-1;assert 0<=i<len(self.w)
        return self.lo[i]+j-self.base[i]

def create(runs,start,modal,mark,seed=143):
    width=[y-x for x,y in zip(start,start[1:])]
    D,pieces,stops,births,H=geometry(runs,start,modal,mark)
    union=Retained(stops);original=len(D);P=len(births)
    active,D,pieces,step,births=contract(D,pieces,stops,births,width)
    width=[width[j] for j in active];start=[start[j] for j in active];stops=[stops[j] for j in active]
    K=len(D);anchors=anchors_of(D,width,step);r=Rope(seed)
    incoming=[[] for _ in D];out=[];U={a:[] for a in anchors};V={a:[] for a in anchors}
    for j,off,(event,w),age in births:incoming[j].append((off,r.node(-1,event,w,age)))
    def collect(j):
        res=None;pos=0
        for off,t in sorted(incoming[j],key=lambda packet:packet[0]):
            assert off==pos;pos+=size(t);res=join(res,t)
        incoming[j]=[];assert pos==width[j]
        return res
    def process(j,seq):
        good=None;pos=0
        for event,w in stops[j]:
            at=event-start[j];head,seq=r.split(seq,at-pos);good=join(good,head)
            end,seq=r.split(seq,w);cursor=event
            for origin,source,z,age in leaves(end):
                if origin<0:out.append((source,cursor,z,age))
                else:U[origin].append((source,cursor,z,age))
                cursor+=z
            assert cursor==event+w;pos=at+w
        good=add(join(good,seq),step[j])
        for off,w in pieces[j]:
            packet,good=r.split(good,w);incoming[D[j]].append((off,packet))
        assert good is None
    for a in anchors:process(a,r.node(a,0,width[a]))
    indeg=[0]*K
    for j,tgt in enumerate(D):
        if j not in anchors and tgt not in anchors:indeg[tgt]+=1
    queue=deque(j for j in range(K)if j not in anchors and not indeg[j]);done=len(anchors)
    while queue:
        j=queue.popleft();process(j,collect(j));done+=1;tgt=D[j]
        if tgt not in anchors:
            indeg[tgt]-=1
            if indeg[tgt]==0:queue.append(tgt)
    assert done==K
    totals=Counter()
    for a,d in anchors.items():
        covered=pos=prev=hi=0;U[a].sort()
        for origin,source,w,age in leaves(collect(a)):
            if origin<0:V[a].append((pos,source,w,age))
            else:
                while hi<len(U[a]) and U[a][hi][0]==prev:
                    prev+=U[a][hi][2];hi+=1
                assert origin==a and source==prev and age==d
                assert hi==len(U[a]) or source+w<=U[a][hi][0]
                prev+=w;covered+=w
            pos+=w
        assert pos==width[a] and covered+sum(u[2]for u in U[a])==pos
        extra,stats=route_holes(width[a],U[a],V[a],d,r);out.extend(extra)
        totals['virtual_return_mass']+=covered
        totals['hole_intervals']+=len(U[a])+len(V[a]);totals['hole_sweep_regions']+=stats['regions']
        totals['sign_splits']+=stats['sign_splits'];totals['bulk_fifo_rows']+=stats['rotated']
        totals['maximum_fifo']=max(totals['maximum_fifo'],stats['maximum_fifo'])
    # Compress both retained orders and coalesce exactly like independent first-return packets.
    canon=[];pos=0
    for x,y,w,age in sorted(out):
        s,t=union.rank(x),union.rank(y)
        assert s==pos and union.rank(x+w-1)==s+w-1 and union.rank(y+w-1)==t+w-1
        if canon and canon[-1][1]+canon[-1][2]==t and canon[-1][3]==age:
            l,old,num,h=canon[-1];canon[-1]=(l,old,num+w,h)
        else:canon.append((s,t,w,age))
        pos+=w
    assert pos==union.base[-1]
    cur=0
    for s,t,w,h in sorted(canon,key=lambda v:v[1]):assert cur==t and h>0;cur+=w
    assert cur==pos
    return canon,union,dict(original_states=original,active_states=K,events=pos,
        overlay_fragments=H,birth_intervals=P,stop_intervals=sum(map(len,stops)),physical_event_ranges=len(union.lo),
        anchor_count=len(anchors),raw_return_pieces=len(out),canonical_return_pieces=len(canon),
        rope_nodes_created=r.created,**totals)
