"""Construct marked minority distances from runs, context intervals and stable destination ranks."""
from bisect import bisect_right
from collections import Counter,deque
from itertools import accumulate
from rope import Rope,add,join,size,leaves
from holes import pair_holes

def geometry(runs,start,modal,mark):
    K=len(modal);n=start[-1];freq=[0]*256
    for b,w in runs:freq[b]+=w
    assert sum(freq)==n and len(start)==K+1 and 0<=mark<n
    F=[0,*accumulate(freq)];occ=[0]*256
    stops=[[] for _ in modal];pieces=[[] for _ in modal];D=[None]*K
    births=[];x=state=fragments=0
    for b,width in runs:
        while width:
            w=min(width,start[state+1]-x);y=F[b]+occ[b];end=x+w
            if b!=modal[state]:
                for z in range(w):
                    r=x+z;target=y+z;j=bisect_right(start,target)-1
                    stops[state].append(r);births.append((j,target-start[j],r))
            else:
                j=bisect_right(start,y)-1
                assert y+w<=start[j+1] and D[state] in (None,j);D[state]=j
                def publish(l,r):
                    if r<=l:return
                    lo=y+l-x-start[j];length=r-l
                    if pieces[state] and pieces[state][-1][0]+pieces[state][-1][1]==lo:
                        p,num=pieces[state][-1];pieces[state][-1]=(p,num+length)
                    else:pieces[state].append((lo,length))
                if x<=mark<end:
                    publish(x,mark);publish(mark+1,end)
                    stops[state].append(mark);births.append((j,y+mark-x-start[j],mark))
                else:publish(x,end)
            occ[b]+=w;x=end;width-=w;fragments+=1
            if x==start[state+1]:state+=1
    assert all(v is not None for v in D)
    return D,pieces,stops,births,fragments

def anchors_of(D,width,step):
    status=[False]*len(D);anchors={}
    for j in range(len(D)):
        stack=[];seen={}
        while not status[j] and j not in seen:seen[j]=len(stack);stack.append(j);j=D[j]
        if j in seen:
            cycle=stack[seen[j]:];anchor=min(cycle,key=lambda z:(width[z],z))
            anchors[anchor]=sum(step[v] for v in cycle)
        for j in stack:status[j]=True
    return anchors

def contract(D,pieces,stops,births,width):
    active=[j for j in range(len(D)) if stops[j]];ids={j:i for i,j in enumerate(active)}
    route={j:(ids[j],0,0) for j in active}
    for j in range(len(D)):
        seen=set();path=[]
        while j not in route and j not in seen:
            seen.add(j);path.append(j);j=D[j]
        if j not in route:
            # Such cycles fill every destination, so no incoming tree or event can reach one.
            assert path[0]==j
            for z in path:route[z]=None
            continue
        known=route[j]
        for t in reversed(path):
            assert known is not None
            end,delta,hops=known
            (offset,w),=pieces[t];assert w==width[t]
            known=(end,delta+offset,hops+1);route[t]=known
    dest=[];packets=[];step=[]
    for j in active:
        out,off,hops=route[D[j]]
        dest.append(out);packets.append([(off+v,w) for v,w in pieces[j]]);step.append(1+hops)
    starts=[]
    for t,off,e in births:
        z,b,hops=route[t];starts.append((z,b+off,e,1+hops))
    return active,dest,packets,step,starts

def acquire(runs,start,modal,mark,seed=143,compact=False):
    width=[y-x for x,y in zip(start,start[1:])]
    D,pieces,stops,births,fragments=geometry(runs,start,modal,mark)
    original_states=len(D)
    if compact:
        active,D,pieces,step,births=contract(D,pieces,stops,births,width)
        width=[width[j] for j in active];start=[start[j] for j in active];stops=[stops[j] for j in active]
    else:
        step=[1]*len(D);births=[(j,off,event,1) for j,off,event in births]
    K=len(D);anchors=anchors_of(D,width,step);r=Rope(seed)
    incoming=[[] for _ in D];finish={};U={a:{} for a in anchors};V={a:{} for a in anchors}
    for j,off,event,age in births:incoming[j].append((off,r.node(-1,event,age=age)))
    def collect(j):
        res=None;pos=0
        for off,t in sorted(incoming[j],key=lambda packet:packet[0]):
            assert off==pos;pos+=size(t);res=join(res,t)
        incoming[j]=[]
        assert pos==width[j]
        return res
    def process(j,seq):
        good=None;pos=0
        for event in stops[j]:
            at=event-start[j];head,seq=r.split(seq,at-pos);good=join(good,head)
            one,seq=r.split(seq,1)
            (origin,source,w,age),=leaves(one);assert w==1
            if origin<0:finish[source]=(event,age)
            else:U[origin][source]=(event,age)
            pos=at+1
        good=add(join(good,seq),step[j])
        for off,w in pieces[j]:
            segment,good=r.split(good,w);incoming[D[j]].append((off,segment))
        assert good is None
    for a in anchors:process(a,r.node(a,0,width[a]))
    degree=[0]*K
    for j,tgt in enumerate(D):
        if j not in anchors and tgt not in anchors:degree[tgt]+=1
    queue=deque(j for j in range(K)if j not in anchors and degree[j]==0);done=len(anchors)
    while queue:
        j=queue.popleft();process(j,collect(j));done+=1;tgt=D[j]
        if tgt not in anchors:
            degree[tgt]-=1
            if degree[tgt]==0:queue.append(tgt)
    assert done==K
    anchor_survivors=regular=anchored=peak_queue=0
    for a,d in anchors.items():
        pos=0;prev=0;holes=sorted(U[a]);hi=0
        for origin,source,w,age in leaves(collect(a)):
            if origin<0:
                assert w==1;V[a][pos]=(source,age)
            else:
                while hi<len(holes) and holes[hi]==prev:hi+=1;prev+=1
                assert origin==a and source==prev and age==d
                assert hi==len(holes) or source+w<=holes[hi]
                prev+=w;anchor_survivors+=w
            pos+=w
        assert pos==width[a]
        pair,scanned=pair_holes(width[a],U[a],V[a],r);regular+=scanned;anchored+=len(pair)
        # Endpoint sweep also yields the largest FIFO width.
        bal=0
        for point in sorted(U[a].keys()|V[a].keys()):
            bal+=(point in V[a])-(point in U[a]);peak_queue=max(peak_queue,abs(bal))
        for v,(u,laps) in pair.items():
            source,first=V[a][v];target,last=U[a][u]
            finish[source]=(target,first+laps*d+last)
    assert len(finish)==len(births) and {t for t,dt in finish.values()}==set(finish)
    assert all(dt>0 for t,dt in finish.values())
    return finish,dict(states=K,original_states=original_states,events=len(births),overlay_fragments=fragments,
        default_packets=sum(map(len,pieces)),cycle_anchors=len(anchors),virtual_rows=sum(width[a]for a in anchors),
        virtual_full_returns=anchor_survivors,real_via_anchors=anchored,
        regular_rows_bulk=regular,maximum_fifo=peak_queue,rope_nodes_created=r.created)
