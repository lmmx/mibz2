"""Range FIFO for the increasing map [n) minus U -> [n) minus V."""
from rope import Rope,add,join,leaves,size

def annotate(row,x):
    if row is None:return None
    p,label,w,age=row
    return label+x-p,age

def route_holes(n,U,V,period,rope=None):
    # (anchor coordinate, physical real label, width, real-path age)
    r=rope or Rope();U.sort();V.sort();out=[];max_q=balance=0
    endpoints=sorted({p for arr in (U,V) for x,_,w,_ in arr for p in (x,x+w)})
    assert len(U)==0 or (endpoints[0]>=0 and endpoints[-1]<=n)
    i=j=0;group=[];positive=0;stats=dict(regions=0,sign_splits=0,rotated=0,maximum_fifo=0)
    def walk(records,sign):
        queue=None
        for a,b,u,v in (records if sign>0 else reversed(records)):
            w=b-a
            def first(item):
                z,age=item
                return (z if sign>0 else -(z+w-1)),age
            if u and v:
                out.append((v[0],u[0],w,v[1]+u[1]))
            elif v:
                z,age=first(v);queue=join(queue,r.node(-1,z,w,age))
            elif u:
                part,queue=r.split(queue,w);target,age=first(u)
                for _,src,span,oldage in leaves(part):
                    if sign>0:lo,hi=src,target
                    else:lo,hi=-(src+span-1),-(target+span-1)
                    out.append((lo,hi,span,oldage+period+age));target+=span
            else:
                rounds,left=divmod(w,size(queue));add(queue,period*rounds)
                head,tail=r.split(queue,left);queue=join(tail,add(head,period));stats['rotated']+=w
            stats['maximum_fifo']=max(stats['maximum_fifo'],size(queue))
        assert queue is None
    for left,right in zip(endpoints,endpoints[1:]):
        while i<len(U) and U[i][0]+U[i][2]<=left:i+=1
        while j<len(V) and V[j][0]+V[j][2]<=left:j+=1
        u=U[i] if i<len(U) and U[i][0]<=left else None
        v=V[j] if j<len(V) and V[j][0]<=left else None
        diff=int(v is not None)-int(u is not None);w=right-left
        crosses=balance*(balance+diff*w)<0
        cuts=[left,left+abs(balance),right] if crosses else [left,right]
        stats['sign_splits']+=crosses
        for a,b in zip(cuts,cuts[1:]):
            old=balance;balance+=diff*(b-a);stats['regions']+=1
            if old==0 and balance==0:
                if u and v:
                    src,age0=annotate(v,a);dst,age1=annotate(u,a)
                    out.append((src,dst,b-a,age0+age1))
                continue
            if not group:positive=1 if balance>0 else -1
            group.append((a,b,annotate(u,a),annotate(v,a)))
            if balance==0:walk(group,positive);group=[]
    assert balance==0 and not group
    return out,stats
