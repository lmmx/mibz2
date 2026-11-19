"""Exact cardinality-constrained search-index size over optional source-bucket retirements."""
from collections import Counter
from itertools import combinations,groupby,product
from pathlib import Path
import csv
import json
import sys
from closure import Atlas,Column,Quotient,incoming,paths,read_input
from bridges import check,profile

def optimum(records,mates,root,optional):
    # Mandatory root removal is already authorized. All other bucket order remains.
    keep=[i for i in range(len(records)) if i!=root]
    labels=[None]+keep+[None]
    indices={w:i for i,w in enumerate(labels) if w is not None}
    pred={}
    for a,b in enumerate(mates):
        if a in indices and b in indices:pred[indices[b]]=indices[a]
    budget=len(optional);inf=10**15;best={0:(0,0)}
    values=[[0]+[inf]*budget];parent=[[-1]*(budget+1)]
    last_mandatory=0
    for j,w in enumerate(labels[1:],1):
        cost=records[w] if w is not None else 0
        val=[];back=[]
        for a in range(budget+1):
            diag=a-j+1
            z,prior=best.get(diag,(inf,-1))
            i=pred.get(j,-1);left=i+diag
            if i>=last_mandatory and 0<=left<=budget and values[i][left]<inf and values[i][left]-1<z:
                z,prior=values[i][left]-1,i
            val.append(z+cost if z<inf else inf);back.append(prior)
        values.append(val);parent.append(back)
        if w not in optional:
            last_mandatory=j;best.clear()
        for a,value in enumerate(val):
            diag=a-j
            if value<best.get(diag,(inf,-1))[0]:best[diag]=(value,j)
    ans=[]
    for a,score in enumerate(values[-1]):
        cut={root};j=len(labels)-1;left=a
        while j:
            i=parent[j][left];assert i>=0
            cut.update(labels[i+1:j]);left-=j-i-1;j=i
        assert left==0 and len(cut)==a+1
        ans.append((score,cut))
    return ans

def small():
    totals=Counter()
    for n in range(1,7):
        for seq in product(range(3),repeat=n):
            freq=Counter(seq);order=sorted(freq)
            raw=[(c,sum(1 for _ in v)) for c,v in groupby(seq)]
            for root in order:
                q=Quotient(raw,{c:bytes([c]) for c in order},order,freq,{root},root);idx=q.ids[root]
                prof=profile(q);possible=set(range(len(order)))-{idx}
                for nopt in range(len(possible)+1):
                    for opt in combinations(possible,nopt):
                        out=optimum(prof[2],prof[3],idx,set(opt))
                        for j,(score,choice) in enumerate(out):
                            assert score==check(q,choice,prof)['remaining']==min(check(q,{idx,*c},prof)['remaining'] for c in combinations(opt,j))
                            totals['budgets']+=1
                        totals['instances']+=1
    print(json.dumps(totals,sort_keys=True,indent=2))

def real(tapes):
    writer=None
    for tape in paths(tapes):
        rs,_=read_input(tape);col=Column(rs);u=col.interval(b'"')
        raw,lead,mins,count,_=incoming(col,u,Atlas(col,u,0));order=sorted(lead,key=mins.get)
        orig={k:w[1:]+b'"' for k,w in lead.items()};z=next(k for k,w in orig.items() if w==b',"')
        for h in (0,4,8,12):
            if h:
                selected={k for k,w in lead.items() if w[min(h,len(w)-1):]==b','};d=z
            else:
                d=max(order,key=lambda k:(count[k],-mins[k]));selected={d}
            q=Quotient(raw,orig,order,count,selected,d);freq,_,pieces,mates,first,last=profile(q)
            c=q.ids[d];b=q.bases[c]
            opts={i for i,f in enumerate(freq) if i!=c and pieces[i]>=2 and b<=first[i] and last[i]<b+freq[c]}
            out=optimum(pieces,mates,c,opts)
            for width,(score,cut) in enumerate(out):
                chosen=sorted(orig[order[i]].hex() for i in cut if i!=c)
                row=dict(tape=int(tape.name.split('.')[0]),h=h,budget=width,rank_records=score,
                         root_only=out[0][0],all_branch=out[-1][0],saved=out[0][0]-score,original_spellings=' '.join(chosen))
                if writer is None:writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n');writer.writeheader()
                writer.writerow(row);sys.stdout.flush()
if __name__=='__main__':real(sys.argv[1]) if len(sys.argv)>1 else small()
