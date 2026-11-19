"""Reprice the registered wide-strip family at fixed radix in the actual raw tower."""
import csv
import sys
from trajectory import project,induce,spell


def check(q,j):
    M=q**j;n=M*M+3*M+1
    bands=[(0,M+1,n-M-1,1),(n-M-1,0,M+1,0)]
    grammar=[]
    total=0
    for _ in range(j):
        stage,m,_=project(bands,q,0)
        bands,_=induce(stage,m,grammar)
        total+=len(stage)
    assert sum(w for _,_,w,_ in bands)==M+4
    distinct=set()
    for s,t,w,rule in bands:
        word=bytes(spell(grammar,rule))
        distinct.add(word)
        for i in range(s,s+w):
            if i<=2:
                target,expected=i+M+1,bytes([1])*M
            elif i<=M+1:
                target,expected=i-1,bytes([1])*(M+2-i)+b'\0'+bytes([1])*(i-2)
            else:
                target,expected=i-(M+2),b'\0'
            assert target==t+i-s and expected==word
    assert len(distinct)==M+1 and len(bands)>=M+1
    return dict(q=q,j=j,M=M,n=n,R=2,distinct_words=len(distinct),raw_final_bands=len(bands),
                raw_progressions_so_far=total,raw_created_rules=len(grammar))


if __name__=='__main__':
    rows=[check(q,j) for q in (2,3,4,8) for j in range(1,10) if q**j<=729]
    out=csv.DictWriter(sys.stdout,rows[0],lineterminator='\n');out.writeheader();out.writerows(rows)
