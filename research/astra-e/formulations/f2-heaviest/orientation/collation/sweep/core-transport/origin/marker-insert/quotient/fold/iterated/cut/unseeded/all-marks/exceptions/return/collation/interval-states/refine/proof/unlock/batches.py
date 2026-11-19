"""Minimum largest simultaneous addition by exhaustive updates and by sink SCC girth."""
from collections import Counter, deque
from itertools import combinations, product
import json


def subsets(rows):
    rows = sorted(rows)
    for count in range(len(rows)+1):
        for selected in combinations(rows, count):
            yield frozenset(selected)


def reach(starts, graph):
    queue = deque((node, 0) for node in starts)
    distance = {}
    while queue:
        node, depth = queue.popleft()
        if node not in distance:
            distance[node] = depth
            queue.extend((other, depth+1) for other in graph[node])
    return distance


def graph_width(old, selected, supports):
    if not selected:
        return 0
    root = -1
    graph = {root:{root}}
    for seam in selected:
        graph[seam] = selected & supports[seam]
        if old & supports[seam]:
            graph[seam] |= {root}
    closure = {t:reach([t],graph) for t in graph}
    width = 1
    pending = set(selected)
    while pending:
        t = next(iter(pending))
        component = {v for v in pending if v in closure[t] and t in closure[v]}
        pending -= component
        if all(graph[v] <= component for v in component):
            # Each edge v->w in this SCC closes a cycle with a shortest path w->v.
            girth = min(1+closure[w][v] for v in component for w in graph[v])
            width = max(width,girth)
    return width


def census(alphabet, limit):
    counts = Counter()
    sizes = Counter()
    for n in range(1,limit+1):
        for last in product(range(alphabet),repeat=n):
            first = sorted(last)
            first_cuts = {t for t in range(1,n) if first[t-1]!=first[t]}
            interior = set(range(1,n))
            psi = sorted(range(n),key=last.__getitem__)
            supports = {t:set(range(psi[t-1]+1,psi[t]+1)) for t in interior-first_cuts}
            for extra in subsets(interior-first_cuts):
                old = first_cuts|extra
                counts['old_partitions']+=1
                optimum = {frozenset():0}
                for selected in subsets(interior-old):
                    if any(not supports[t] & (old|selected) for t in selected):
                        continue
                    if selected:
                        optimum[selected] = min(
                            max(price,len(selected)-len(prior))
                            for prior,price in optimum.items() if prior < selected)
                    assert graph_width(old,selected,supports) == optimum[selected]
                    sizes[optimum[selected]] += 1
                counts['neutral_sets'] += len(optimum)
            counts['columns'] += 1
    return {'alphabet':alphabet,'through_n':limit,**counts,'minimum_width_counts':sizes}


if __name__ == '__main__':
    print(json.dumps([census(2,7),census(3,5)],indent=2,sort_keys=True))
