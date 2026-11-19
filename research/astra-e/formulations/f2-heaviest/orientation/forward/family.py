"""Force long forward packet lists with two return words in a genuine primitive cyclic BWT."""
import argparse
import csv
from itertools import groupby
import json
from pathlib import Path
import subprocess
import sys


def debruijn(order):
    """FKM concatenation of binary Lyndon words of length dividing order."""
    a=[0]*(2*order+1);out=[]
    def visit(t,p):
        if t>order:
            if order%p==0: out.extend(a[1:p+1])
            return
        a[t]=a[t-p];visit(t+1,p)
        for j in range(a[t-p]+1,2):
            a[t]=j;visit(t+1,t)
    visit(1,1)
    double=out+out
    assert len({tuple(double[i:i+order]) for i in range(len(out))})==len(out)==1<<order
    return out


def bwt(word):
    n=len(word);r=list(word);size=1
    while size<n:
        pairs=[(r[i],r[(i+size)%n]) for i in range(n)]
        order=sorted(range(n),key=pairs.__getitem__)
        new=[0]*n
        for a,b in zip(order,order[1:]): new[b]=new[a]+(pairs[a]!=pairs[b])
        r=new;size*=2
    assert min(r)==0 and max(r)==n-1, "family should be primitive"
    return [word[i-1] for i in order],r[0]


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);a=p.parse_args()
    writer=None
    for q in (2,3,4,5,6,7):
        tokens=debruijn(q);m=len(tokens)
        last,_=bwt(tokens)
        token_runs=sum(1 for _ in groupby(last))
        assert token_runs>=m//2+1
        for k in (1,4,16,32):
            original=[c for token in tokens for c in ([0]+[2]*token+[1]*k)]
            last,mark=bwt(original)
            runs=[(c,len(list(g))) for c,g in groupby(last)]
            inp=f"{len(last)} {mark} {len(runs)} 1\n"+"".join(f"{c} {w}\n" for c,w in runs)+f"0 {m}\n"
            j=json.loads(subprocess.check_output([a.binary.resolve()],input=inp.encode()))
            assert bytes.fromhex(j['expanded_hex'])==bytes(original)
            assert j['word_count']==2 and j['groups']==2*k+1
            assert j['select_translations']>=k*token_runs and j['live_merges']==0
            r=dict(order=q,tokens=m,token_bwt_runs=token_runs,tail=k,n=len(original),
                   bwt_runs=len(runs),proper_prefix_nodes=j['groups'],
                   packet_inputs=j['packet_inputs'],translation_lower_bound=k*token_runs,
                   translations=j['select_translations'],live_merges=j['live_merges'])
            if writer is None: writer=csv.DictWriter(sys.stdout,fieldnames=r,lineterminator="\n");writer.writeheader()
            writer.writerow(r)

if __name__=="__main__":
    main()
