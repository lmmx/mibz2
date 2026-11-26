"""Realize connected simple MAX-CUT instances as primitive circular BWTs with a unique anchor."""
from itertools import combinations
from certify import graph, ingress


def witness(k, edges):
    # Attach one leaf to vertex zero. In a maximum cut the leaf can always be
    # separated from zero, then a global flip puts the leaf outside.
    adj = [[] for _ in range(k+1)]
    for i,j in [*edges,(k,0)]:
        adj[i].append(j)
        adj[j].append(i)
    stack, circuit = [k],[]
    while stack:
        if adj[stack[-1]]:
            stack.append(adj[stack[-1]].pop())
        else:
            circuit.append(stack.pop())
    if any(adj) or len(set(circuit))!=k+1:
        return False
    word = tuple(reversed(circuit))[:-1]
    assert word.count(k)==1 and word[0]==k
    transitions = set(zip(word,word[1:]+word[:1]))
    assert len(transitions)==len(word)==2*(1+len(edges))
    actual = graph(word)
    best_plain = max(sum(((s>>i)^(s>>j))&1 for i,j in edges) for s in range(1<<k))
    best_marked = max(ingress(actual,{i for i in range(k) if s>>i&1}) for s in range(1<<k))
    assert best_marked==best_plain+1
    return True


if __name__ == "__main__":
    for k in range(2,6):
        pairs = list(combinations(range(k),2))
        count = sum(witness(k,[e for j,e in enumerate(pairs) if mask>>j&1]) for mask in range(1<<len(pairs)))
        print(k,count)
