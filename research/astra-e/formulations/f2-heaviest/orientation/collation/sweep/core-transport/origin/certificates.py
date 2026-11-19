"""Exact upper/lower bounds on shortened-core frequency from heavy original word certificates."""
from collections import Counter, defaultdict
from heapq import heappop, heappush
from itertools import groupby, product
import csv
import json
import sys
from origin import Column, RunIndex, arcs, clip, heavy_set, paths, read_input


def known_words(index,u,k,work):
    ranking=[]
    queue=[(u[0]-u[1],b'',*u)]
    threshold=1
    while queue and -queue[0][0]>=threshold:
        _,q,lo,hi=heappop(queue)
        if q:
            a,b=max(lo,u[0]),min(hi,u[1])
            if b-a>=threshold:
                ranking.append((a-b,q,a,b))
                ranking.sort()
                if len(ranking)>=k:
                    ranking=ranking[:k]
                    threshold=-ranking[-1][0]
            lo,hi=clip((lo,hi),u)
        if hi-lo<threshold:
            continue
        kids,pieces=index.children((lo,hi))
        work['old_nodes']+=1
        work['old_pieces']+=pieces
        for sym,(l,r) in kids.items():
            if r-l>=threshold:
                heappush(queue,(l-r,bytes([sym])+q,l,r))
    return [(q,a,b) for _,q,a,b in ranking]


class Certificates:
    def __init__(self,index,u,h,k):
        self.index,self.u,self.h=index,u,h
        self.work=Counter()
        self.known=known_words(index,u,k,self.work)
        self.ending=defaultdict(list)
        self.mass=Counter()
        self.selected=Counter()
        self.possible=Counter()
        for w,a,b in self.known:
            c=w[min(h,len(w)-1):]
            self.selected[c]+=b-a
            for n in range(1,len(w)+1):
                q=w[-n:]
                self.ending[q].append((w,a,b,c))
                self.mass[q]+=b-a
                if n<=len(c):
                    self.possible[q]+=b-a
                self.work['suffix_entries']+=1
                self.work['suffix_key_bytes']+=n

    def probe(self,c,l,r):
        stack=[(c,l,r,0)]
        out=[]
        while stack:
            q,lo,hi,d=stack.pop()
            self.work['probe_nodes']+=1
            known=self.mass[q]
            assert known<=hi-lo
            if known and known==hi-lo:
                self.work['covered_nodes']+=1
                for w,a,b,target in self.ending[q]:
                    if target==c:
                        out.append((a,b,w[:len(w)-len(c)]))
                continue
            a,b=max(lo,self.u[0]),min(hi,self.u[1])
            if a<b and (len(c)==1 or d==self.h):
                out.append((a,b,q[:d]))
            if d==self.h:
                continue
            rem=clip((lo,hi),self.u)
            if rem[0]<rem[1]:
                kids,pieces=self.index.children(rem)
                self.work['probe_pieces']+=pieces
                for sym,(a,b) in kids.items():
                    stack.append((bytes([sym])+q,a,b,d+1))
        return sorted(out)

    def select(self):
        best=max(self.selected.values())
        winners=[]
        queue=[]
        def extend(q,l,r):
            kids,pieces=self.index.children((l,r))
            self.work['search_nodes']+=1
            self.work['search_pieces']+=pieces
            for sym,(a,b) in kids.items():
                word=bytes([sym])+q
                unknown=b-a-self.mass[word]
                assert unknown>=0
                upper=unknown+self.possible[word]
                if upper>=best:
                    heappush(queue,(-upper,word,a,b))
        extend(b'',*self.u)
        while queue and -queue[0][0]>=best:
            _,word,l,r=heappop(queue)
            self.work['popped']+=1
            unknown=r-l-self.mass[word]
            eligible=self.selected[word]
            if unknown+eligible>=best:
                self.work['candidates']+=1
                own=self.probe(word,l,r)
                m=sum(b-a for a,b,p in own)
                assert eligible<=m<=eligible+unknown
                if m>best:
                    best=m
                    winners=[]
                if m==best:
                    winners.append((word,own))
            # All known occurrences with target exactly word cease to be eligible
            # after even one further extension. Unknowns still bound every descendant.
            if unknown+self.possible[word]-eligible>=best:
                lo,hi=clip((l,r),self.u)
                if hi-lo>=best:
                    extend(word,lo,hi)
        return best,sorted(winners)


def verify(runs,u,words,h,k):
    solver=Certificates(RunIndex(runs),u,h,k)
    known=Counter(words)
    assert [(w,b-a) for w,a,b in solver.known]==sorted(known.items(),key=lambda kv:(-kv[1],kv[0]))[:k]
    assert all(all(x==w for x in words[a-u[0]:b-u[0]]) for w,a,b in solver.known)
    best,wins=solver.select()
    tails=Counter(w[min(h,len(w)-1):] for w in words)
    assert best==max(tails.values())
    assert [c for c,_ in wins]==sorted(c for c,n in tails.items() if n==best)
    for c,own in wins:
        hits={x:p for a,b,p in own for x in range(a,b)}
        assert len(hits)==sum(b-a for a,b,_ in own)==best
        assert hits=={x:w[:len(w)-len(c)] for x,w in enumerate(words,u[0]) if w[min(h,len(w)-1):]==c}
    metrics=dict(best=best,top_hex=wins[0][0].hex(),ties=len(wins),num_known=len(solver.known))
    for name in ('old_nodes','old_pieces','search_nodes','search_pieces','candidates','popped','probe_nodes',
                 'probe_pieces','covered_nodes','suffix_entries','suffix_key_bytes'):
        metrics[name]=solver.work[name]
    metrics['total_pieces']=sum(metrics[k] for k in ('old_pieces','search_pieces','probe_pieces'))
    return metrics


def small():
    counts=Counter()
    for n in range(1,8):
        for xs in product(range(2),repeat=n):
            runs=[(b,sum(1 for _ in it)) for b,it in groupby(xs)]
            last=bytes(xs)
            f=sorted(range(n),key=last.__getitem__)
            for a in range(n):
                for b in range(a+1,n+1):
                    words=arcs(last,f,a,b)
                    for h in (0,1,3,n):
                        for k in (1,4,16):
                            m=verify(runs,(a,b),words,h,k)
                            counts.update(cases=1,ties=m['ties']>1,covered=m['covered_nodes'],
                                          selected_sources=m['best'])
    print(json.dumps(counts,indent=2))


def real(tapes):
    writer=None
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs)
        last=b''.join(bytes([c])*w for c,w in runs)
        f=sorted(range(len(last)),key=last.__getitem__)
        q=col.interval(b'"')
        for name,sec in [('frequent',heavy_set(col,q,1)[0][0]),('quote',q)]:
            words=arcs(last,f,*sec)
            for h in (4,8,12):
                for k in (1,4,16):
                    row=dict(ordinal=int(tape.name.split('.')[0]),section=name,h=h,k=k,
                             **verify(runs,sec,words,h,k))
                    if writer is None:
                        writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                        writer.writeheader()
                    writer.writerow(row)
                    sys.stdout.flush()


if __name__=='__main__':
    real(sys.argv[1]) if len(sys.argv)>1 else small()
