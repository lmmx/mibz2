"""Lower bound allowing every byte occurrence to choose its own direction, without any seam constraints."""
from collections import deque
from pathlib import Path
import argparse
import csv
import sys
import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import maximum_bipartite_matching

from optimize import Column, Direct, HERE, contexts, events, heavy_set


def cover(words,inc):
    f,b=contexts(words,False),contexts(words,True)
    f_index,b_index={},{}
    edge=set()
    for i,w in enumerate(words):
        for j in range(len(w)):
            for r in inc[i][j]:
                left=f_index.setdefault((f[i][j],r),len(f_index))
                right=b_index.setdefault((b[i][j],r),len(b_index))
                edge.add((left,right))
    a,b=map(np.array,zip(*edge))
    graph=coo_matrix((np.ones(len(a),dtype=np.uint8),(a,b)),shape=(len(f_index),len(b_index))).tocsr()
    match=maximum_bipartite_matching(graph,perm_type="column")
    n=sum(x>=0 for x in match)
    assert len(match)==len(f_index) and len({int(x) for x in match if x>=0})==n
    assert all((i,int(x)) in edge for i,x in enumerate(match) if x>=0)
    inverse=np.full(len(b_index),-1,dtype=np.int32)
    for i,x in enumerate(match):
        if x>=0:inverse[x]=i
    # Construct the matching's alternating forest, certify an equally sized vertex cover.
    left_seen={i for i,x in enumerate(match) if x<0}
    q=deque(left_seen)
    right_seen=set()
    while q:
        i=q.popleft()
        for j in graph.indices[graph.indptr[i]:graph.indptr[i+1]]:
            if j==match[i] or j in right_seen:continue
            right_seen.add(j)
            k=int(inverse[j])
            assert k>=0
            if k not in left_seen:
                left_seen.add(k)
                q.append(k)
    left_paid=len(f_index)-len(left_seen)
    assert left_paid+len(right_seen)==n and all(i not in left_seen or j in right_seen for i,j in edge)
    return dict(independent_edges=len(edge),runwise_matching=n,
                cover_forward=left_paid,cover_backward=len(right_seen))


def main(tapes):
    sys.path.insert(0,str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    ref={(int(r["ordinal"]),r["section"]):r for r in csv.DictReader((HERE/"packets.csv").open())}
    out=None
    for tape in paths(tapes):
        idx=int(tape.name.split(".")[0])
        col=Column(read_input(tape)[0])
        p=col.interval(b'"')
        for s,section in [("quote",[p]),("frequent",heavy_set(col,p,1)[0])]:
            w,_,i,_=events(col,section)
            row=dict(ordinal=idx,section=s,**cover(w,i),
                     trace_seams=ref[idx,s]["independent_trace_optimum"],
                     word_seams=ref[idx,s]["joined_run_optimum"])
            assert row["runwise_matching"]<=int(row["trace_seams"])
            if out is None:
                out=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                out.writeheader()
            out.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    main(Path(sys.argv[1]))
