"""Exact modal marked clocks, including precompiled posts and default-cycle chunking."""
from collections import Counter,defaultdict
from itertools import accumulate

def index(L):
    F=tuple(sorted(L));cursor={}
    for i,c in enumerate(F):cursor.setdefault(c,i)
    psi=[0]*len(L);lf=[0]*len(L)
    for i,c in enumerate(L):
        j=cursor[c];cursor[c]+=1;psi[j]=i;lf[i]=j
    return F,psi,lf

def seq(F,psi,x,n):
    out=bytearray()
    for _ in range(n):out.append(F[x]);x=psi[x]
    return bytes(out)

def intervals(L,k):
    """Enumerate exact context intervals by stable backward extension; small oracle."""
    base={};counts=Counter(L);at=0
    for c in sorted(counts):base[c]=at;at+=counts[c]
    parts={b'':(0,len(L))}
    for _ in range(k):
        rank=Counter();new={}
        for context,(l,h) in sorted(parts.items()):
            inc=Counter(L[l:h])
            for c,w in inc.items():
                a=base[c]+rank[c];new[bytes([c])+context]=(a,a+w);rank[c]+=w
        parts=new
    return parts

class Clock:
    def __init__(self,L,k):
        self.L=bytes(L);self.k=k;self.F,self.psi,self.lf=index(L)
        contexts=[seq(self.F,self.psi,x,k) for x in range(len(L))]
        self.words=sorted(set(contexts));ids={v:i for i,v in enumerate(self.words)}
        self.state=[ids[w] for w in contexts]
        counts=[Counter() for _ in self.words];next_state={}
        for i,r in enumerate(self.state):
            c=L[i];counts[r][c]+=1;y=self.state[self.lf[i]]
            assert next_state.setdefault((r,c),y)==y
        self.modal=[min(z,key=lambda c:(-z[c],c)) for z in counts]
        self.default=[next_state[r,c] for r,c in enumerate(self.modal)]
        refs=intervals(L,k);assert set(refs)==set(contexts)
        for w,(a,b) in refs.items():assert contexts[a:b]==[w]*(b-a)
    def encode_from_end(self,end,n):
        r=end;events=[]
        for t in range(n):
            s=self.state[r];y=self.lf[r];c=self.L[r]
            if c!=self.modal[s]:
                events.append((t,self.state[y]) if self.k else (t,c))
            r=y
        return self.state[end],events
    def finite(self,mark,n):
        end=mark
        for _ in range(n):end=self.psi[end]
        return self.encode_from_end(end,n)
    def lap(self,mark):
        r=self.lf[mark];n=1
        while r!=mark:r=self.lf[r];n+=1
        return n,self.encode_from_end(mark,n)
    def stretch(self,state,n):
        # Generate one tail/cycle and use its period for the outstanding part.
        seen={};data=bytearray();visits=[]
        while len(data)<n and state not in seen:
            seen[state]=len(data);visits.append(state)
            data.append(self.modal[state]);state=self.default[state]
        if len(data)==n:return data,state
        lo=seen[state];word=data[lo:];rest=n-len(data);lap,rem=divmod(rest,len(word))
        data+=word*lap+word[:rem]
        return data,visits[lo+rem]
    def emit(self,code,n):
        state,events=code;t=0;out=bytearray()
        for clock,v in events:
            assert t<=clock<n
            s,state=self.stretch(state,clock-t);out+=s
            if self.k:state=v;out.append(self.words[v][0])
            else:out.append(v)
            t=clock+1
        s,_=self.stretch(state,n-t);out+=s
        return bytes(reversed(out))
