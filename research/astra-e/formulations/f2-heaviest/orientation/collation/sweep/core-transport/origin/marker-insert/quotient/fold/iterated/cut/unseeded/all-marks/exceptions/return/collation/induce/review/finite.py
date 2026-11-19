"""Independent dense simultaneous deletion oracle for conjugated LF sections."""
from collections import Counter
from itertools import product
from pathlib import Path
import csv
import json
import sys

def stable_lf(last):
    psi=sorted(range(len(last)),key=last.__getitem__)
    lf=[None]*len(last)
    for i,x in enumerate(psi):lf[x]=i
    return lf

def chunks(seq):
    spans=[]
    for x in seq:
        if spans and spans[-1][-1]+1==x:spans[-1].append(x)
        else:spans.append([x])
    return spans

def layout(last,kept,policy):
    front=[i for i in range(len(last)) if i in kept]
    complement=chunks(i for i in range(len(last)) if i not in kept)
    if policy=="reverse_spans":complement.reverse()
    elif policy=="long_first":complement.sort(key=lambda a:(-len(a),a[0]))
    elif policy=="short_first":complement.sort(key=lambda a:(len(a),a[0]))
    elif policy=="reflected":
        front.reverse()
        return front+[x for a in reversed(complement) for x in reversed(a)],-1
    elif policy!="stable":raise ValueError(policy)
    return front+[x for a in complement for x in a],1

def stab(edges):
    # The cut between suffix coordinates k and k+1 has integer index k.
    intervals=sorted((max(i,j)-1,min(i,j)) for i,j in edges if i!=j)
    used=[];last=-1
    for hi,lo in intervals:
        if lo>last:used.append(hi);last=hi
    return used

def induce(last,kept,order,sign,merge=False,check_cylinders=False,join_check=None):
    n=len(last);m=len(kept);raw_lf=stable_lf(last)
    assert len(set(order))==n and set(order[:m])==kept
    go={x:i for i,x in enumerate(order)}
    t=[go[raw_lf[x]] for x in order]
    cuts=sum(order[i+1]-order[i]!=sign for i in range(n-1))
    if check_cylinders and m<n and order[m]-order[m-1]==sign:cuts+=1
    runs=1+sum(x!=y for x,y in zip(last,last[1:]))
    words=[bytes([last[x]]) for x in order]
    ids=[];node=0
    for i in range(n):
        if i and (words[i]!=words[i-1] or t[i]!=t[i-1]+1 or
                  (not merge and order[i]-order[i-1]!=sign) or
                  (check_cylinders and (i==m or t[i]==m))):
            node+=1
        ids.append(node)
    initial=node+1
    assert initial<=runs+2*cuts
    direct=[(i,j) for i,j in enumerate(t) if i>=m and j>=m and i!=j]
    bound=1+len(stab(direct)) if m<n else 0
    adjacency=sum(abs(i-j)==1 for i,j in set((min(i,j),max(i,j)) for i,j in direct))
    original_ids=ids.copy();original_t=t.copy()
    steps=partial=joins=0
    while len(t)>m:
        cur=len(t);inv=[None]*cur
        for i,j in enumerate(t):inv[j]=i
        a,b=ids[-1],ids[inv[-1]]
        wa,wb=ids.count(a),ids.count(b)
        assert ids[-wa:]==[a]*wa and [ids[inv[j]] for j in range(cur-wb,cur)]==[b]*wb
        w=min(wa,wb,cur-m);h=cur-w;steps+=1
        partial+=w<min(wa,wb)
        if a!=b:
            joins+=1
            assert all(t[x]<h for x in range(h,cur))
        else:
            assert all(t[x]==x for x in range(h,cur))
        ids2=[];words2=[];t2=[];fresh={}
        for i in range(h):
            j=t[i]
            if j<h:
                ids2.append(ids[i]);words2.append(words[i]);t2.append(j)
            else:
                z=(ids[i],ids[j])
                if z not in fresh:node+=1;fresh[z]=node
                ids2.append(fresh[z]);words2.append(words[i]+words[j]);t2.append(t[j])
        assert len(fresh)<=1
        ids,words,t=ids2,words2,t2
    assert steps>=bound and (not direct or bound>=adjacency+1)
    for i,x in enumerate(order[:m]):
        stop=x;out=[]
        while True:
            out.append(last[stop]);stop=raw_lf[stop]
            if stop in kept:break
        assert t[i]==go[stop] and words[i]==bytes(out)
    # Label the source bands of backward LF by their order on the target axis.
    inv=[None]*m
    for i,j in enumerate(t):inv[j]=i
    code={};incoming=[None]*m
    for j,x in enumerate(inv):
        if ids[x] not in code:code[ids[x]]=len(code)
        elif j:assert ids[inv[j-1]]==ids[x]
        incoming[x]=code[ids[x]]
    assert stable_lf(incoming)==t
    raw_psi=sorted(range(n),key=last.__getitem__)
    for i,x in enumerate(order[:m]):
        out=[];j=i
        while len(out)<n:
            j=inv[j];out.extend(reversed(words[j]))
        oracle=[]
        for _ in range(n):x=raw_psi[x];oracle.append(last[x])
        assert out[:n]==oracle
    if check_cylinders:
        assert partial==0
        itineraries=[]
        for i in range(m):
            j=i;w=[]
            while True:
                w.append(original_ids[j]);j=original_t[j]
                if j<m:break
            itineraries.append(tuple(w))
        pair=[ids[i]!=ids[i+1] for i in range(m-1)]
        assert pair==[itineraries[i]!=itineraries[i+1] for i in range(m-1)]
        if join_check is not None:
            inv0=[None]*n
            for x,y in enumerate(original_t):inv0[y]=x
            live=set(range(m));stack=list(live)
            while stack:
                y=original_t[stack.pop()]
                if y not in live:stack.append(y);live.add(y)
            all_middle={(original_ids[inv0[y]],original_ids[y]) for y in range(m,n)}
            live_middle={(original_ids[inv0[y]],original_ids[y]) for y in range(m,n) if y in live}
            for q in (2,3,4):
                distinct={w[i:i+q] for w in itineraries for i in range(len(w)-q+1)}
                if q==2:
                    assert distinct==live_middle
                    if len(all_middle)>joins and not join_check.get("omitted_cycle_witness"):
                        join_check["omitted_cycle_witness"]=dict(last=last,V=sorted(kept),order=order,
                            all_pairs=len(all_middle),live_pairs=len(live_middle),joins=joins)
                assert len(distinct)<=joins*(q-1)
                join_check["distinct_totals"][q]+=len(distinct)
            join_check["layouts"]+=1;join_check["joins"]+=joins
        return tuple(pair[::sign]), steps
    return (steps,bound,adjacency,initial,len(code),partial)

def exhaustive():
    policies=["stable","reverse_spans","long_first","short_first","reflected"]
    totals=Counter();wins={p:Counter() for p in policies};witness={}
    partial_witness=None
    for alphabet,upper in [(2,6),(3,4)]:
        for n in range(1,upper+1):
            for last in product(range(alphabet),repeat=n):
                for bits in range(1,1<<n):
                    kept={x for x in range(n) if bits>>x&1}
                    res={}
                    for p in policies:
                        order,sign=layout(last,kept,p)
                        ans=induce(last,kept,order,sign);res[p]=ans
                        totals["sections_layouts"]+=1;totals["physical_marked_outputs"]+=len(kept)
                        totals["tail_steps"]+=ans[0];totals["partial_steps"]+=ans[-1]
                        totals["hitting_lower_bound"]+=ans[1]
                        if ans[-1] and partial_witness is None:
                            partial_witness=dict(last=list(last),V=sorted(kept),order=order,metrics=ans)
                    for p in policies:
                        d=res[p][0]-res["stable"][0];direction="more" if d>0 else "fewer" if d<0 else "equal"
                        wins[p][direction]+=1
                        if d and p+direction not in witness:
                            witness[p+direction]=dict(last=list(last),V=sorted(kept),stable=res["stable"],alternate=res[p])
                    totals["sections"]+=1
    return dict(counts=dict(totals),versus_stable=wins,witnesses=witness,partial=partial_witness)

def longlaps():
    result=[]
    for k in range(1,10):
        n=1<<k;last=[1]+[0]*(n-1);kept={0}
        bitrev=lambda x:int(f"{x:0{k}b}"[::-1],2)
        # LF on these physical ranks decrements the old time. Reverse bits of orbit time -x.
        order=[(-bitrev(q))%n for q in range(n)]
        for policy,q,sign,merged in [("stable",list(range(n)),1,False),
                    ("bit-reversed orbit, seam-refined",order,1,False),
                    ("bit-reversed orbit, merged",order,1,True)]:
            s,lb,adj,initial,final,partial=induce(last,kept,q,sign,merge=merged)
            result.append(dict(n=n,order=policy,initial=initial,steps=s,stabbing_lower_bound=lb,adjacent_links=adj,final=final))
    return result

if __name__=="__main__":
    out=Path(sys.argv[1]);out.mkdir(exist_ok=True)
    (out/"small.json").write_text(json.dumps(exhaustive(),indent=2)+"\n")
    x=longlaps()
    with (out/"long-laps.csv").open("w") as f:
        writer=csv.DictWriter(f,list(x[0]),lineterminator="\n")
        writer.writeheader();writer.writerows(x)
