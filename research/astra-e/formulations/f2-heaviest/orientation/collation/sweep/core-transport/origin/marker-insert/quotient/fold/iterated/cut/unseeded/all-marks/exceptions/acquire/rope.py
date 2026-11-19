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
        push(t);s=size(t.left)
        if k<s:
            a,t.left=self.split(t.left,k);return a,update(t)
        if k>s+t.width:
            t.right,b=self.split(t.right,k-s-t.width);return update(t),b
        if k==s:
            a=t.left;t.left=None;return a,update(t)
        if k==s+t.width:
            b=t.right;t.right=None;return update(t),b
        left=k-s;right=t.width-left
        a=self.node(t.kind,t.start,left,t.age);b=self.node(t.kind,t.start+left,right,t.age)
        return join(t.left,a),join(b,t.right)
