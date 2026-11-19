"""Exact dictionary-payments oracle with one unknown direction chosen per word."""
from itertools import groupby
from pathlib import Path
import argparse
import csv
import sys

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import breadth_first_order, maximum_flow

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "union"))
from direct import Column, Direct, components, heavy_set


def common_length(a, b, start):
    i = start
    while i < len(a) and i < len(b) and a[i] == b[i]:
        i += 1
    return i


def build(words):
    source, sink = 0, 1
    count = len(words) + 2
    edges = []
    infinity = 3 + sum(map(len, words))
    costs = []
    for forward in (True, False):
        keys = [w[:-1] if forward else w[:0:-1] for w in words]
        order = sorted(range(len(keys)), key=keys.__getitem__)
        groups = [(0, len(order), None, 0)]
        full_cost = 0
        while groups:
            lo, hi, parent, depth = groups.pop()
            first = keys[order[lo]]
            length = common_length(first, keys[order[hi - 1]], depth)
            node = count
            count += 1
            weight = length - depth + (parent is None)
            full_cost += weight
            edges.append((node, sink, weight) if forward else (source, node, weight))
            if parent is not None:
                edges.append((node, parent, infinity) if forward else (parent, node, infinity))
            i = lo
            while i < hi and len(keys[order[i]]) == length:
                word = 2 + order[i]
                edges.append((word, node, infinity) if forward else (node, word, infinity))
                i += 1
            while i < hi:
                j = i + 1
                letter = keys[order[i]][length]
                while j < hi and keys[order[j]][length] == letter:
                    j += 1
                groups.append((i, j, node, length))
                i = j
        costs.append(full_cost)
    left, right, cap = map(np.array, zip(*edges))
    graph = coo_matrix((cap, (left, right)), shape=(count, count)).tocsr()
    flow = maximum_flow(graph, source, sink)
    residual = graph - flow.flow
    residual.eliminate_zeros()
    assert (residual.data > 0).all()
    reach = set(breadth_first_order(residual, source, directed=True, return_predecessors=False))
    assign = [2+i in reach for i in range(len(words))]
    return dict(words=len(words), bytes=sum(map(len, words)), forward=costs[0], backward=costs[1],
                minimum=flow.flow_value, forward_words=sum(assign), flow_nodes=count,
                flow_edges=len(edges)), assign


def paid(words, assignment):
    prefix, suffix = set(), set()
    for word, forward in zip(words, assignment):
        if forward:
            prefix.update(word[:i] for i in range(len(word)))
        else:
            suffix.update(word[len(word)-i:] for i in range(len(word)))
    return len(prefix) + len(suffix)


def check():
    from itertools import combinations, product
    allwords = [bytes(w) for n in (1, 2, 3) for w in product(range(2), repeat=n)]
    cases = mixed = 0
    for size in (1, 2, 3, 4):
        for words in combinations(allwords, size):
            props, assignment = build(words)
            best = min(paid(words, a) for a in product([False, True], repeat=size))
            assert best == props["minimum"] == paid(words, assignment)
            assert props["forward"] == paid(words, [True] * size)
            assert props["backward"] == paid(words, [False] * size)
            mixed += best < min(props["forward"], props["backward"])
            cases += 1
    print(f"{cases} optimal cuts matched exhaustive direction assignments; {mixed} strictly beat both one-direction tries")


def census(tapes):
    sys.path.insert(0, str(HERE.parents[4]/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import paths, read_input
    output = None
    for tape in paths(tapes):
        ordinal = int(tape.name.split(".")[0])
        runs, _ = read_input(tape)
        col = Column(runs)
        parent = col.interval(b'"')
        for kind, sections in (("quote", [parent]), ("frequent", heavy_set(col, parent, 1)[0])):
            index = Direct(col, sections)
            for subset in ("all", "once"):
                words = [w for w,n in index.word_counts.items() if subset == "all" or n == 1]
                values, assign = build(words)
                assert paid(words, assign) == values["minimum"]
                row = dict(ordinal=ordinal, section=kind, subset=subset, **values)
                if output is None:
                    output = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                    output.writeheader()
                output.writerow(row)
                sys.stdout.flush()


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("tapes", nargs="?", type=Path)
    opt = p.parse_args()
    census(opt.tapes) if opt.tapes else check()
