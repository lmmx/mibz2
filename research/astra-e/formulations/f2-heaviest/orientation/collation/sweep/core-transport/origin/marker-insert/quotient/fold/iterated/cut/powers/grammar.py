"""Positive literal, concatenation and repetition nodes for marked output."""
from collections import Counter


class Node:
    __slots__ = ('literal','left','right','repeat','n')

    def __init__(self,literal=None,left=None,right=None,repeat=1):
        self.literal,self.left,self.right,self.repeat=literal,left,right,repeat
        if literal is not None:
            self.n=len(literal)
        elif right is None:
            self.n=left.n*repeat
        else:
            self.n=left.n+right.n
        assert self.n>0

    def __len__(self):
        return self.n

    def __add__(self,right):
        return self if right==b'' else Node(left=self,right=right)

    def power(self,d):
        assert d>=1
        return self if d==1 else Node(left=self,repeat=d)


def output(root,skip,horizon):
    assert 0<=skip<root.n and horizon>=0
    # A marked span can cross the end of its cycle any number of times.
    top=root.power(max(1,(skip+horizon+root.n-1)//root.n))
    buf=bytearray()
    cache={}
    stack=[('span',top,skip,skip+horizon)]
    stats=Counter(visits=0,copied_bytes=0,literal_bytes=0,power_copy_bytes=0,stack_peak=1)
    while stack:
        tag,x,a,b=stack.pop()
        if tag=='remember':
            cache[x]=a
        elif tag=='repeat':
            # Already emitted one full period in this rotation. Doubling its
            # initialized prefix also handles a final fractional repetition.
            remain=a
            while remain:
                n=min(remain,len(buf)-x)
                assert n>0
                buf.extend(buf[x:x+n])
                stats['power_copy_bytes']+=n
                remain-=n
        elif a<b:
            stats['visits']+=1
            if x.literal is not None:
                buf.extend(x.literal[a:b])
                stats['literal_bytes']+=b-a
            elif x in cache:
                start=cache[x]
                buf.extend(buf[start+a:start+b])
                stats['copied_bytes']+=b-a
            else:
                if a==0 and b==x.n:
                    stack.append(('remember',x,len(buf),0))
                if x.right is not None:
                    boundary=x.left.n
                    if b>boundary:
                        stack.append(('span',x.right,max(0,a-boundary),b-boundary))
                    if a<boundary:
                        stack.append(('span',x.left,a,min(b,boundary)))
                else:
                    period=x.left.n
                    amount=b-a
                    prefix=min(amount,period)
                    start=a%period
                    first=min(prefix,period-start)
                    if amount>prefix:
                        stack.append(('repeat',len(buf),amount-prefix,0))
                    if first<prefix:
                        stack.append(('span',x.left,0,prefix-first))
                    stack.append(('span',x.left,start,start+first))
        stats['stack_peak']=max(stats['stack_peak'],len(stack))
    assert len(buf)==horizon==sum(stats[k] for k in ('literal_bytes','copied_bytes','power_copy_bytes'))
    return bytes(buf),stats
