"""Minimize distinct original-run translation events for forward/backward return spelling."""
from bisect import bisect_right
from pathlib import Path
import argparse
import csv
import sys
import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import breadth_first_order, maximum_flow

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"union"))
sys.path.insert(0,str(HERE.parent/"async"))
from direct import Column, Direct, heavy_set
from schedule import grow


def events(col, selected):
    index = Direct(col, selected)
    words = sorted(index.word_counts)
    word_id = {w:i for i,w in enumerate(words)}
    # LF sends each original run bijectively to a whole forward translation interval.
    ordered = sorted((a+col.maps[r], col.starts[r+1]+col.maps[r], r)
                     for r,a in enumerate(col.starts[:-1]))
    starts = [a for a,_,_ in ordered]
    inc = [[set() for _ in w] for w in words]
    visits = 0
    for a,b,width,w,_ in index.bands:
        row = a
        ids = inc[word_id[w]]
        for i,c in enumerate(w):
            lo,hi,r = ordered[bisect_right(starts,row)-1]
            assert lo <= row and row+width <= hi and col.symbols[r] == c
            ids[i].add(r)
            row -= col.maps[r]
            visits += 1
        assert row == b
    return words, index.word_counts, inc, visits


def contexts(words, reverse):
    child = {}
    keys = []
    for w in words:
        node = 0
        path = []
        for byte in (w[::-1] if reverse else w):
            path.append(node)
            key = node,byte
            if key not in child:
                child[key] = len(child)+1
            node = child[key]
        keys.append(path[::-1] if reverse else path)
    return keys


def optimize(words, inc, weighted=True):
    keysets = [contexts(words,False),contexts(words,True)]
    pools = [{},{}]
    variables = []
    size = 2
    a,b,cap = [],[],[]
    infinity = 1+sum(len(ids) for row in inc for ids in row)
    def attach(x,y,c):
        a.append(x)
        b.append(y)
        cap.append(c)
    recipes=[]
    for wi,word in enumerate(words):
        cells = list(range(size,size+len(word)))
        size += len(word)
        variables.append(cells)
        instructions=[]
        for i,v in enumerate(cells):
            if i:attach(v,cells[i-1],infinity)
            actions=[[],[]]
            for side,pool in enumerate(pools):
                for run in (inc[wi][i] if weighted else (0,)):
                    key = keysets[side][wi][i],run
                    e = pool.get(key)
                    if e is None:
                        e = size
                        size += 1
                        pool[key] = e
                        attach(0,e,1) if side else attach(e,1,1)
                    attach(e,v,infinity) if side else attach(v,e,infinity)
                    actions[side].append(e)
            instructions.append(actions)
        recipes.append(instructions)
    graph=coo_matrix((np.asarray(cap,dtype=np.int64),(a,b)),shape=(size,size)).tocsr()
    flow=maximum_flow(graph,0,1)
    residual=graph-flow.flow
    residual.eliminate_zeros()
    reachable=set(breadth_first_order(residual,0,directed=True,return_predecessors=False))
    seams=[]
    for cells in variables:
        t=sum(v in reachable for v in cells)
        assert all((v in reachable)==(i<t) for i,v in enumerate(cells))
        seams.append(t)

    def price(t):
        charged=set()
        for seam,row in zip(t,recipes):
            for i,actions in enumerate(row):
                charged.update(actions[i>=seam])
        return len(charged)

    result = price(seams)
    assert result==flow.flow_value
    return result,seams,price,len(pools[0]),len(pools[1])


def check(col, selected, brute=False):
    words,counts,inc,visits=events(col,selected)
    best,seams,price,f,b=optimize(words,inc)
    simple,plain_seams,plain_price,_,_=optimize(words,inc,weighted=False)
    assert price(list(map(len,words)))==f and price([0]*len(words))==b
    if brute:
        from itertools import product
        assert best == min(price(t) for t in product(*(range(len(w)+1) for w in words)))
    greedy=[]
    states,_,_,_=grow({w:counts[w] for w in words}, False, seams_out=greedy)
    assert plain_price(greedy)==states
    return dict(words=len(words), backward_band_spelling_steps=visits,
                incidence=sum(len(s) for w in inc for s in w),
                all_forward=f,all_backward=b,minimum_run_events=best,
                run_events_at_state_optimum=price(plain_seams),run_events_max_mass=price(greedy),
                run_events_alternating=price([(len(w)+1)//2 for w in words]),
                states_at_run_optimum=plain_price(seams),minimum_states=simple)


def small():
    from itertools import groupby,product
    n=0
    for size in range(1,8):
        for last in product(range(2),repeat=size):
            col=Column([(k,sum(1 for _ in v)) for k,v in groupby(last)])
            for token in (b"\x00",b"\x01",b"\x00\x01"):
                bounds=col.interval(token)
                if bounds[0]!=bounds[1]:
                    check(col,[bounds],True)
                    n+=1
    print(f"{n} cyclic rank sections matched every feasible seam assignment for distinct translation-run events")


def real(tapes,blocks):
    sys.path.insert(0,str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    writer=None
    for path in paths(tapes):
        ordinal=int(path.name.split(".")[0])
        if blocks and ordinal not in blocks:continue
        col=Column(read_input(path)[0])
        p=col.interval(b'"')
        for name,section in (("quote",[p]),("frequent",heavy_set(col,p,1)[0])):
            row=dict(ordinal=ordinal,section=name,**check(col,section))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("tapes",type=Path,nargs="?")
    parser.add_argument("--blocks",type=int,nargs="+")
    args=parser.parse_args()
    real(args.tapes,args.blocks) if args.tapes else small()
