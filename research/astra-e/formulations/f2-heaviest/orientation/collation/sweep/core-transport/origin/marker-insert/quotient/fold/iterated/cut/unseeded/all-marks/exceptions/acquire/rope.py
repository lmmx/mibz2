"""Exclusive-ownership treap of affine label segments and a common lazy age increment."""
import random

class Node:
    __slots__=("kind","start","width","age","left","right","size","lazy","priority")
    def __init__(self,kind,start,width,age,priority):
        self.kind=kind;self.start=start;self.width=width;self.age=age
        self.left=self.right=None;self.size=width;self.lazy=0;self.priority=priority

def size(t):return t.size if t is not None else 0
def add(t,v):
    if t is not None:t.age+=v;t.lazy+=v
    return t
def push(t):
    if t.lazy:add(t.left,t.lazy);add(t.right,t.lazy);t.lazy=0
def update(t):t.size=size(t.left)+t.width+size(t.right);return t
def join(a,b):
    if a is None:return b
    if b is None:return a
    if a.priority>b.priority:push(a);a.right=join(a.right,b);return update(a)
    push(b);b.left=join(a,b.left);return update(b)
def leaves(t):
    if t is None:return
    push(t);yield from leaves(t.left)
    yield t.kind,t.start,t.width,t.age
    yield from leaves(t.right)

class Rope:
    def __init__(self,seed=143):
        self.random=random.Random(seed);self.created=0
    def node(self,kind,start,width=1,age=0):
        assert width>0;self.created+=1
        return Node(kind,start,width,age,self.random.getrandbits(64))
    def split(self,t,k):
        assert 0<=k<=size(t)
        if k==0:return None,t
        if k==size(t):return t,None
        node=t;pos=0
        while node:
            push(node);begin=pos+size(node.left);end=begin+node.width
            if k==begin or k==end:return boundary(t,k)
            if k<begin:node=node.left
            elif k>end:node=node.right;pos=end
            else:
                before,rest=boundary(t,begin);old,after=boundary(rest,node.width)
                assert old is node and not old.left and not old.right
                left=self.node(old.kind,old.start,k-begin,old.age)
                right=self.node(old.kind,old.start+k-begin,end-k,old.age)
                return join(before,left),join(right,after)
        raise AssertionError("missing boundary")

def boundary(t,k):
    if k==0:return None,t
    if k==size(t):return t,None
    push(t);s=size(t.left)
    if k<=s:
        a,t.left=boundary(t.left,k);return a,update(t)
    assert k>=s+t.width
    t.right,b=boundary(t.right,k-s-t.width);return update(t),b
