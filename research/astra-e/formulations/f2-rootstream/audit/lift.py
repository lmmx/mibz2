"""Build real byte columns whose first-return selector trees force frequent deep revisits."""
from hashlib import sha256
import csv
import sys
from tree import audit, encode


def lift(token_l):
    # Each token i spells 1^i 0. These words are prefix-free, with their
    # bytewise lexical order equal to the integer order 0,1,2,...
    psi=sorted(range(len(token_l)),key=token_l.__getitem__)
    used=set(); byte_nodes=[]; forward=[]; previous=[];cycles=0
    for start in range(len(psi)):
        if start in used: continue
        tokens=[];p=start
        while p not in used:
            used.add(p);p=psi[p];tokens.append(token_l[p])
        assert p==start
        b=[x for t in tokens for x in ([1]*t+[0])]
        first=len(byte_nodes)
        byte_nodes.extend(b)
        forward.extend(first+(i+1)%len(b) for i in range(len(b)))
        previous.extend(b[i-1] for i in range(len(b)))
        cycles+=1
    # Sort the infinite periodic F suffixes on all cycles by prefix doubling.
    rank=byte_nodes[:];step=forward[:];width=1
    while True:
        key=[(rank[i],rank[step[i]]) for i in range(len(rank))]
        groups={k:j for j,k in enumerate(sorted(set(key)))}
        new=[groups[k] for k in key]
        if new==rank:break
        rank=new;step=[step[j] for j in step];width*=2
        assert width<=4*len(rank)
    last=[previous[i] for i in sorted(range(len(rank)),key=rank.__getitem__)]
    return last,cycles


def recover(column):
    # Independent stable LF and one byte at a time to the previous zero.
    psi=sorted(range(len(column)),key=column.__getitem__)
    lf=[0]*len(column)
    for j,x in enumerate(psi): lf[x]=j
    token_l=[]
    for x in range(column.count(0)):
        k=0
        while column[x]:
            k+=1;x=lf[x]
        token_l.append(k)
    return token_l


def claimed(d,m):
    # At each level j>=1, v_j receives a single run of D followed by the
    # observed lower labels j..D-1. Its output therefore has D-j+1 runs.
    return (2*m+d-1,2*m+d-1+d*(d+1)//2,
            m*(d+1)+m+sum(i+1 for i in range(1,d)))


fields=["depth","repeats","raw_rows","original_runs","token_cycles","token_rows","root_runs",
        "nodes","selectors","materialized_runs","descent_visits","raw_column_sha256"]
w=csv.writer(sys.stdout,lineterminator="\n");w.writerow(fields)
for d,m in ((2,4),(4,16),(8,64),(16,256),(32,1024),(48,2304),(48,2344)):
    token_l=[d,0]*m+list(range(1,d))
    column,c=lift(token_l)
    got=recover(column)
    assert got==token_l
    row=audit(got,(-1,)+tuple(range(d)))
    t,local,visits=claimed(d,m)
    assert (row["root_runs"],row["materialized_runs"],row["visits"])==(t,local,visits)
    assert row["selectors"] == 2*m+3*d-2 and visits == len(column)
    w.writerow([d,m,len(column),len(encode(column)),c,len(got),t,row["nodes"],row["selectors"],
                local,visits,sha256(bytes(column)).hexdigest()])
