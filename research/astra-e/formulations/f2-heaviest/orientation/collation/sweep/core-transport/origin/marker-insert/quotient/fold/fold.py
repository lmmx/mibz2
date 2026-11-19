"""Contract compatible whole source classes with a unique successor type."""
from bisect import bisect_right
from collections import Counter, defaultdict, deque
from itertools import groupby, product
from pathlib import Path
import csv
import json
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from quotient import Atlas,Column,Oracle,Quotient,append_run,forward,incoming,library,paths,read_input


def independent(successor,freq):
    """Maximum weight set with no pair k and successor[k] both selected."""
    pending=Counter(t for t in successor.values() if t in successor)
    take={k:freq[k] for k in successor}
    skip=dict.fromkeys(successor,0)
    queue=deque(k for k in successor if not pending[k])
    peeled=[]
    while queue:
        k=queue.popleft()
        peeled.append(k)
        t=successor[k]
        if t in successor:
            take[t]+=skip[k]
            skip[t]+=max(take[k],skip[k])
            pending[t]-=1
            if not pending[t]:
                queue.append(t)
    cycles={k for k in successor if pending[k]}
    chosen=set()
    while cycles:
        start=min(cycles)
        cycle=[]
        k=start
        while k in cycles:
            cycle.append(k)
            cycles.remove(k)
            k=successor[k]
        assert k==start and len(cycle)>1
        def boundary(first):
            scores={first:take[start] if first else skip[start]}
            trace=[]
            for k in cycle[1:]:
                prev=max(scores,key=scores.get)
                nxt={False:scores[prev]+skip[k]}
                step={False:prev}
                if False in scores:
                    nxt[True]=scores[False]+take[k]
                    step[True]=False
                scores=nxt
                trace.append(step)
            last=False if first else max(scores,key=scores.get)
            value=scores[last]
            picks=set()
            for i in range(len(cycle)-1,0,-1):
                if last:
                    picks.add(cycle[i])
                last=trace[i-1][last]
            if last:
                picks.add(start)
            return value,picks
        chosen.update(max((boundary(False),boundary(True)),key=lambda v:v[0])[1])
    for k in reversed(peeled):
        if successor[k] not in chosen and take[k]>skip[k]:
            chosen.add(k)
    assert chosen.isdisjoint(successor[k] for k in chosen)
    return chosen


class Fold:
    abi=Quotient.abi

    def __init__(self,raw,words,order,freq):
        self.old=raw
        self.words=words
        self.order=order
        self.freq=freq
        first={}
        last={}
        self.starts=[]
        self.base={}
        rows=0
        for k in order:
            self.starts.append(rows)
            self.base[k]=rows
            rows+=freq[k]
        pos=0
        for k,w in raw:
            first.setdefault(k,pos)
            last[k]=pos+w-1
            pos+=w
        assert pos==rows
        self.owner=lambda x: order[bisect_right(self.starts,x)-1]
        self.succ={k:self.owner(first[k]) for k in order if self.owner(first[k])==self.owner(last[k])!=k}
        self.chosen=independent(self.succ,freq)
        targets={self.succ[k] for k in self.chosen}
        maps={t:[] for t in targets}
        self.deleted=[]
        seen=0
        j=0
        for k,w in raw:
            if k in self.chosen:
                self.deleted.append((seen,seen+w,k))
            left=seen
            seen+=w
            while left<seen:
                t=order[j]
                end=min(seen,self.base[t]+freq[t])
                if t in targets:
                    append_run(maps[t],k if k in self.chosen else t,end-left)
                left=end
                if end==self.base[t]+freq[t]:
                    j+=1
        self.removed=sum(freq[k] for k in self.chosen)
        self.m=rows-self.removed
        self.extra=0
        after=dict(freq)
        for k in self.chosen:
            after[self.succ[k]]-=freq[k]
        keep=[k for k in order if after[k]]
        self.weights=[words[k]+(words[self.succ[k]] if k in self.chosen else b'') for k in keep]
        self.ids={k:i for i,k in enumerate(keep)}
        self.bases=[]
        prefix=0
        for k in keep:
            self.bases.append(prefix)
            prefix+=after[k]
        assert prefix==self.m
        self.root=[]
        self.slices=sum(map(len,maps.values()))
        self.drop_ids=len(order)-len(keep)
        cursor={t:(0,0) for t in targets}
        for k,w in raw:
            if k in self.chosen:
                continue
            if k not in targets:
                append_run(self.root,self.ids[k],w)
                continue
            j,used=cursor[k]
            stream=maps[k]
            while w:
                t,width=stream[j]
                n=min(w,width-used)
                append_run(self.root,self.ids[t],n)
                w-=n
                used+=n
                if used==width:
                    j+=1
                    used=0
            cursor[k]=j,used
        assert all(cursor[t]==(len(stream),0) for t,stream in maps.items())
        assert sum(w for _,w in self.root)==self.m

    def mark(self,x,skip):
        removed=0
        for a,b,k in self.deleted:
            if x<a:break
            if x<b:
                # Rank of the marked deleted vertex within incoming k.
                prior=sum(v-u for u,v,t in self.deleted if t==k and u<a)
                x=self.base[k]+prior+x-a
                y,offset,_=self.mark(x,skip+len(self.words[k]))
                return y,offset,True
            removed+=b-a
        return x-removed,skip,False


def differential(f):
    ids={k:i for i,k in enumerate(f.order)}
    old=forward([(ids[k],w) for k,w in f.old],f.starts)
    new=forward(f.root,f.bases)
    dead={d for d,i in old if f.order[i] in f.chosen}
    embedding={x:i for i,x in enumerate(x for x in range(len(old)) if x not in dead)}
    for x,v in embedding.items():
        y,i=old[x]
        k=f.order[i]
        literal=f.words[k]
        if y in dead:
            y,j=old[y]
            literal+=f.words[f.order[j]]
        dest,label=new[v]
        assert dest==embedding[y] and f.weights[label]==literal
    return old,new


def small(lib):
    counts=Counter()
    for n in range(1,7):
        for last in product(range(3),repeat=n):
            order=sorted(set(last),reverse=True)
            raw=[(k,sum(1 for _ in v)) for k,v in groupby(last)]
            freq=Counter(last)
            words={k:bytes(range(k+1)) for k in order}
            f=Fold(raw,words,order,freq)
            brute=max(sum(freq[k] for k in choice) for mask in product((0,1),repeat=len(order))
                      for choice in [{k for k,selected in zip(order,mask) if selected}]
                      if choice<=set(f.succ) and choice.isdisjoint(f.succ[k] for k in choice))
            assert brute==f.removed
            old,new=differential(f)
            size=sum(freq[k]*len(words[k]) for k in order)
            counts.update(objects=1,old_rows=n,after_rows=f.m,active=bool(f.chosen),empty_buckets=f.drop_ids)
            for x in range(n):
                k=f.order[old[x][1]]
                for skip in range(len(words[k])):
                    y=x
                    ref=bytearray()
                    while len(ref)<size+skip:
                        y,i=old[y]
                        ref.extend(words[f.order[i]])
                    v,byte,transferred=f.mark(x,skip)
                    got,_=f.abi(lib,v,byte,size)
                    assert got==ref[skip:skip+size]
                    counts.update(marks=1,transferred=transferred)
    print(json.dumps(counts,indent=2))


def real(lib,tapes):
    writer=None
    for tape in paths(tapes):
        runs,mark=read_input(tape)
        col=Column(runs)
        o=Oracle(b''.join(bytes([b])*n for b,n in runs))
        u=col.interval(b'"')
        raw,words,minima,count,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(words,key=minima.get)
        words={k:w[1:]+b'"' for k,w in words.items()}
        f=Fold(raw,words,order,count)
        differential(f)
        pos=mark
        ref=bytearray()
        for _ in range(col.rows):
            ref.append(o.first[pos])
            pos=o.forward[pos]
        shifted={o.forward[x]:x-u[0] for x in range(*u)}
        back=[0]*col.rows
        for i,x in enumerate(o.forward):
            back[x]=i
        x=mark
        skip=0
        while x not in shifted:
            x=back[x]
            skip+=1
            assert skip<=col.rows
        v,byte,transferred=f.mark(shifted[x],skip)
        got,taken=f.abi(lib,v,byte,col.rows)
        assert got==ref
        data=dict(ordinal=tape.name.split('.')[0],old_rows=sum(count.values()),new_rows=f.m,
                  eligible=len(f.succ),eligible_mass=sum(count[k] for k in f.succ),selected=len(f.chosen),
                  removed_rows=f.removed,empty_buckets=f.drop_ids,old_runs=len(raw),new_runs=len(f.root),
                  old_types=len(words),target_stream_runs=f.slices,old_dictionary=sum(map(len,words.values())),
                  new_dictionary=sum(map(len,f.weights)),mark_moved=int(transferred),second=taken,output=len(got))
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=data,lineterminator='\n')
            writer.writeheader()
        writer.writerow(data)
        sys.stdout.flush()


if __name__=='__main__':
    lib=library(sys.argv[1])
    if len(sys.argv)>2:
        real(lib,sys.argv[2])
    else:
        small(lib)
