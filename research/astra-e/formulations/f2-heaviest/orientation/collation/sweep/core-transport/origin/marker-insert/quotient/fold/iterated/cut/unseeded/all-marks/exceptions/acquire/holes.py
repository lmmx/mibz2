"""Pair exceptional endpoints of an increasing partial permutation; count full applications."""
from rope import Rope,add,join,size,leaves

def pair_holes(n,U,V,rope=None):
    assert len(U)==len(V)
    rope=rope or Rope()
    points=sorted(set(U)|set(V));assert all(0<=x<n for x in points)
    found={};begin=balance=regular=0
    for i,x in enumerate(points):
        balance+=(x in V)-(x in U)
        if balance:continue
        block=points[begin:i+1];begin=i+1
        if block[0] not in V:block.reverse()
        queue=None;last=None
        for p in block:
            if last is not None:
                gap=abs(last-p)-1;regular+=gap
                if gap:
                    whole,extra=divmod(gap,size(queue));add(queue,whole)
                    head,tail=rope.split(queue,extra);queue=join(tail,add(head,1))
            if p in V:label=rope.node(-1,p)
            else:
                label,queue=rope.split(queue,1);add(label,1)
            if p in U:
                (_,birth,w,hops),=leaves(label);assert w==1
                found[birth]=(p,hops)
            else:queue=join(queue,label)
            last=p
        assert queue is None
    assert balance==0 and set(found)==set(V)
    return found,regular
