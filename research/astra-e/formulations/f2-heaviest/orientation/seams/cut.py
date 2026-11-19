"""Minimum closed prefix/suffix cover when each return can meet at an interior seam."""
from itertools import product
from pathlib import Path
import argparse
import csv
import sys
import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import breadth_first_order, maximum_flow

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "union"))
from direct import Column, Direct, heavy_set


def price(words, seams):
    prefix, suffix = set(), set()
    for word, t in zip(words, seams):
        prefix.update(word[:i] for i in range(t))
        suffix.update(word[i+1:] for i in range(t, len(word)))
    return len(prefix) + len(suffix)


def optimize(words):
    states = [{}, {}]
    size = 2
    # In a maximum-flow representation zero is the source, one is the sink.
    edges = set()
    infinity = 1 + sum(map(len, words))
    for word in words:
        forward, backward = [], []
        for reverse, out in [(False, forward), (True, backward)]:
            trie = states[reverse]
            path = word[::-1] if reverse else word
            parent = None
            for i in range(len(word)):
                key = path[:i]
                node = trie.get(key)
                if node is None:
                    node = size
                    size += 1
                    trie[key] = node
                    edges.add((0, node, 1) if reverse else (node, 1, 1))
                    if parent is not None:
                        edges.add((parent, node, infinity) if reverse else (node, parent, infinity))
                out.append(node)
                parent = node
        for f, b in zip(forward, reversed(backward)):
            edges.add((b, f, infinity))
    a, b, cap = np.array(list(edges), dtype=np.int64).T
    graph = coo_matrix((cap, (a, b)), shape=(size, size)).tocsr()
    flow = maximum_flow(graph, 0, 1)
    residual = graph - flow.flow
    residual.eliminate_zeros()
    assert (residual.data > 0).all()
    reach = set(breadth_first_order(residual, 0, directed=True, return_predecessors=False))
    seams = []
    for w in words:
        # Prefer fully one-direction spells if an optimal cover permits it.
        f = next((i for i in range(len(w)) if states[0][w[:i]] not in reach), len(w))
        b = next((j for j in range(len(w)) if states[1][w[::-1][:j]] in reach), len(w))
        assert f+b >= len(w)
        seams.append(0 if b == len(w) and f < len(w) else f)
    total = price(words, seams)
    assert total == flow.flow_value
    return dict(queries=total, vertices=size, edges=len(edges),
                pure_forward=sum(t==len(w) for w,t in zip(words,seams)), pure_backward=seams.count(0),
                interior=sum(0<t<len(w) for w,t in zip(words,seams)),
                forward_bytes=sum(seams), spelled_bytes=sum(map(len,words)))


def check():
    from itertools import combinations
    allwords = [bytes(x) for n in (1,2,3) for x in product(range(2),repeat=n)]
    checked = 0
    for k in (1, 2, 3):
        for words in combinations(allwords, k):
            expected = min(price(words, seams) for seams in product(*(range(len(w)+1) for w in words)))
            assert expected == optimize(words)["queries"]
            checked += 1
    print(f"{checked} dictionaries matched exhaustive choices of all interior seams")


def real(tapes, blocks):
    sys.path.insert(0, str(HERE.parents[5] / "docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, read_input
    output = None
    for tape in paths(tapes):
        ordinal = int(tape.name.split(".")[0])
        if blocks and ordinal not in blocks:
            continue
        col = Column(read_input(tape)[0])
        parent = col.interval(b'"')
        for name, section in [("quote", [parent]), ("frequent", heavy_set(col,parent,1)[0])]:
            direct = Direct(col, section)
            row = dict(ordinal=ordinal, section=name, words=len(direct.word_counts),
                       **optimize(list(direct.word_counts)))
            if output is None:
                output = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                output.writeheader()
            output.writerow(row)
            sys.stdout.flush()


if __name__ == "__main__":
    p=argparse.ArgumentParser()
    p.add_argument("tapes", type=Path, nargs="?")
    p.add_argument("--blocks", type=int, nargs="+")
    args=p.parse_args()
    real(args.tapes,args.blocks) if args.tapes else check()
