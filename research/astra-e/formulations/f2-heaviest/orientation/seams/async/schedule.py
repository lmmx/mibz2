"""Asynchronous spelling-price oracle, with and without early endpoint pairing."""
from heapq import heappop, heappush
from itertools import combinations, product
from pathlib import Path
import csv
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from cut import Column, Direct, heavy_set, price, optimize


def grow(counts, paired, bucket_queue=None, seams_out=None):
    words = list(counts)
    masses = list(counts.values())
    ids = list(range(len(words)))
    # Each live occurrence has one forward and one backward suffix/prefix state.
    owner = [[None] * len(words) for _ in (0,1)]
    done = [False] * len(words)
    depths = [[0] * len(words) for _ in (0,1)]
    live = {}
    heap = []
    created = states = pairs = heap_pops = 0

    def push(side, depth, members):
        nonlocal created
        gid = created
        created += 1
        weight = sum(masses[i] for i in members)
        live[gid] = [side, depth, members, weight]
        for i in members:
            owner[side][i] = gid
        if bucket_queue is not None:
            bucket_queue.add(weight, side, gid)
        elif not paired:
            heappush(heap, (-weight, depth, side, gid))
        return gid

    push(0, 0, ids.copy())
    push(1, 0, ids.copy())

    def advance(gid):
        nonlocal states
        side, depth, members, mass = live.pop(gid)
        if not mass:
            return
        states += 1
        children = {}
        for i in members:
            if done[i]:
                continue
            assert owner[side][i] == gid and depths[side][i] == depth
            w = words[i]
            depths[side][i] += 1
            if depths[0][i] + depths[1][i] == len(w):
                done[i] = True
                # A concrete constructor detects this by intersecting ranks after the step.
                other = live[owner[1-side][i]]
                other[3] -= masses[i]
            else:
                byte = w[depth] if side == 0 else w[-depth-1]
                children.setdefault(byte, []).append(i)
        for byte in sorted(children):
            push(side, depth+1, children[byte])

    if paired:
        for i in ids:
            while not done[i]:
                f,b = owner[0][i],owner[1][i]
                pairs += 1
                advance(f)
                advance(b)
    elif bucket_queue is not None:
        while (gid := bucket_queue.next(live)) is not None:
            advance(gid)
        heap_pops = bucket_queue.pops
    else:
        while heap:
            neg, depth, side, gid = heappop(heap)
            heap_pops += 1
            mass = live[gid][3]
            assert -neg >= mass
            if mass == 0:
                del live[gid]
            elif -neg != mass:
                heappush(heap,(-mass,depth,side,gid))
            else:
                advance(gid)
    assert all(done) and states == price(words, depths[0])
    if seams_out is not None:
        seams_out.extend(depths[0])
    return states, pairs, heap_pops, sum(0<d<len(w) for d,w in zip(depths[0],words))


def tiny():
    ws=[bytes(w) for n in (1,2,3) for w in product(range(2),repeat=n)]
    cases=0
    for size in (1,2,3):
        for selected in combinations(ws,size):
            counts={w:1+(i%3) for i,w in enumerate(selected)}
            opt=optimize(selected)["queries"]
            protected, pairs,_,_ = grow(counts,True)
            greedy,_,_,_ = grow(counts,False)
            assert pairs<=opt<=protected<=2*pairs and opt<=greedy
            cases+=1
    print(f"{cases} dictionaries paid exactly their induced seam prices; every witnessed matching lower bound and 2-approximation held")


def real(tapes):
    sys.path.insert(0,str(HERE.parents[6]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths,read_input
    oracles = {(int(r["ordinal"]),r["section"]):r for r in csv.DictReader((HERE.parent/"clock.csv").open())}
    writer=None
    for tape in paths(tapes):
        ordinal=int(tape.name.split(".")[0])
        col=Column(read_input(tape)[0])
        p=col.interval(b'"')
        for name,selection in (("quote",[p]),("frequent",heavy_set(col,p,1)[0])):
            obj=Direct(col,selection)
            # Direct returns source-ordered bands. Sort words for a reproducible oracle witness choice.
            counts={w:obj.word_counts[w] for w in sorted(obj.word_counts)}
            matched, pairs,_,interior = grow(counts,True)
            greedy,_,pops,greedy_seams = grow(counts,False)
            r=oracles[ordinal,name]
            opt=int(r["unrestricted"])
            assert pairs<=opt<=matched<=2*pairs and opt<=greedy
            row=dict(ordinal=ordinal,section=name,words=len(counts),optimal=opt,
                     matching_witnesses=pairs,matching_price=matched,matching_interior=interior,
                     width_price=greedy,width_interior=greedy_seams,width_heap_pops=pops,
                     alternating=int(r["alternating"]),frozen_clock=int(r["frozen_clock"]))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=="__main__":
    real(Path(sys.argv[1])) if len(sys.argv)>1 else tiny()
