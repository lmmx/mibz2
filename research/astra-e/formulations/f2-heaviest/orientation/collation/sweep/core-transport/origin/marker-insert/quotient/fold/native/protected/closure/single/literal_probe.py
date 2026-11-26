"""Turn per-block query support into static old-word labels and check exact one-class bounds."""
import csv,gzip,sys
from collections import Counter,defaultdict
from fractions import Fraction
from pathlib import Path
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P.parents[3]))
from fold import Atlas,Column,incoming,paths,read_input
def run(data,out):
    by_id={};states=defaultdict(Counter);successor=defaultdict(dict)
    with gzip.open(P.parent/'features.csv.gz','rt') as f:
        r=csv.reader(f);next(r)
        for kind,stream,*v in r:
            s=int(stream);x=list(map(int,v))
            if kind=='S':states[s][tuple(x)]+=1
            elif x[2]!=4294967295:successor[s][x[0]]=x[2]
    wordgain=defaultdict(Counter);total=Counter();budgets=defaultdict(list)
    for tape in paths(data):
        runs,_=read_input(tape);col=Column(runs);u=col.interval(b'"')
        _,words,minima,count,_=incoming(col,u,Atlas(col,u,0))
        s=int(tape.name.split('.')[0]);base=0;by_id[s]={}
        for k in sorted(words,key=minima.get):
            by_id[s][base]=words[k][1:]+b'"';base+=count[k]
        for e,w in states[s].items():
            if len(e)==1:
                label=by_id[s][e[0]];wordgain[s][label]+=w;total[label]+=w
        for b in [1,4,16,64,256]:
            credit=Counter()
            for e,w in states[s].items():
                if len(e)<=b:
                    for k in e:credit[k]+=Fraction(w,len(e))
            top=[k for k,_ in sorted(credit.items(),key=lambda v:(-v[1],v[0]))[:b]]
            upper=sum(credit[k] for k in top).__floor__()
            # An explicit compatible subset of the credit leaders.
            keep=set()
            for k in top:
                if successor[s][k] not in keep and all(successor[s][j]!=k for j in keep):keep.add(k)
            got=sum(w for e,w in states[s].items() if all(i in keep for i in e))
            budgets[b].append((s,upper,got))
    with (out/'budgets.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('stream','cap','upper','feasible'))
        for b,rr in budgets.items():
            for s,hi,lo in rr:w.writerow((s,b,hi,lo))
    with (out/'literals.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('source_word_hex','sum_singleton_saving',*wordgain))
        for label,score in sorted(total.items(),key=lambda x:(-x[1],x[0])):
            w.writerow((label.hex(),score,*(v[label] for v in wordgain.values())))
    with (out/'leaders.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(('stream','one_query_optimum','source_word_hex','successor_word_hex'))
        for s,c in wordgain.items():
            label,score=max(c.items(),key=lambda x:x[1]);k=next(k for k,l in by_id[s].items() if l==label)
            w.writerow((s,score,label.hex(),by_id[s][successor[s][k]].hex()))
if __name__=='__main__':run(Path(sys.argv[1]),Path(sys.argv[2]))
