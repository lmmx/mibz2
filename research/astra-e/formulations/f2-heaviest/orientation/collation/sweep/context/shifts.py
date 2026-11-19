"""Exact context and order constraints on type-dependent one-byte shifts in a monochrome section."""
from collections import Counter, defaultdict
from itertools import groupby, product
from pathlib import Path
import csv
import sys

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parents[1]))
from order import Column, Oracle, heavy_set, truth


def objects(oracle, section):
    words, incoming = truth(oracle, section)
    types = list(words)
    named = {w: i for i, w in enumerate(types)}
    a, b = section
    source = [None] * (b - a)
    returns = [None] * (b - a)
    for word, cells in words.items():
        for x, y in cells:
            source[x-a] = named[word]
            returns[x-a] = y-a
    assert len(set(w[0] for w in words)) == 1
    return types, source, returns


def groups(types, source, returns, f, a):
    """Closure graph. Selected implies selected on every outgoing edge."""
    k = len(types)
    graph = [set() for _ in types]
    followers = [set() for _ in types]
    for s, t in zip(source, returns):
        followers[s].add(source[t])
    def edge(x, y):
        if x != y:
            graph[x].add(y)
    for followers_of_type in followers:
        pivot = min(followers_of_type)
        for s in followers_of_type:
            edge(s, pivot)
            edge(pivot, s)
    for j in range(1, len(source)):
        x = a + j - 1
        if f[x] >= x+1:
            edge(source[j-1], source[j])
        if f[x+1] <= x:
            edge(source[j], source[j-1])
    # Iterative Kosaraju; the class graph can be large on dense text.
    seen = set()
    order = []
    for s in range(k):
        if s in seen:
            continue
        seen.add(s)
        stack = [(s, iter(graph[s]))]
        while stack:
            u, children = stack[-1]
            v = next(children, None)
            if v is None:
                order.append(u)
                stack.pop()
            elif v not in seen:
                seen.add(v)
                stack.append((v, iter(graph[v])))
    reverse = [set() for _ in types]
    for u in range(k):
        for v in graph[u]:
            reverse[v].add(u)
    cid = [-1] * k
    members = []
    for s in reversed(order):
        if cid[s] >= 0:
            continue
        component = len(members)
        members.append([])
        cid[s] = component
        work = [s]
        while work:
            u = work.pop()
            members[component].append(u)
            for v in reverse[u]:
                if cid[v] < 0:
                    cid[v] = component
                    work.append(v)
    dag = {i: set() for i in range(len(members))}
    for u in range(k):
        dag[cid[u]].update(cid[v] for v in graph[u] if cid[v] != cid[u])
    return cid, members, dag, followers


def assignments(dag):
    # The Kosaraju order above is a topological order on the SCC graph.
    states = [0]
    for s in reversed(range(len(dag))):
        assert all(t>s for t in dag[s])
        need = sum(1<<t for t in dag[s])
        states += [mask | (1<<s) for mask in states if (mask & need) == need]
    return states


def shifted(oracle,a,types,source,returns,bit):
    positions = [oracle.forward[a+i] if bit[c] else a+i for i,c in enumerate(source)]
    if any(x>=y for x,y in zip(positions,positions[1:])):
        return None
    c = types[0][:1]
    spell = {}
    for tid,dest in zip(source,returns):
        w = types[tid][bit[tid]:] + (c if bit[source[dest]] else b'')
        if not w or (tid in spell and w != spell[tid]):
            return None
        spell[tid] = w
    if len(set(spell.values())) != len(spell):
        return None
    return [spell[k] for k in range(len(types))],positions


def suffixes(words):
    # Adjacent LCP of sorted distinct reversed proper-prefix keys.
    strings = sorted({w[:0:-1] for w in words})
    previous = b''
    total = 1
    for w in strings:
        i = 0
        while i < min(len(previous), len(w)) and previous[i] == w[i]:
            i += 1
        total += len(w)-i
        previous=w
    return total


def analyze(oracle, section, exhaustive=False, mark=None):
    a,b = section
    types,source,returns = objects(oracle,section)
    cid,members,dag,followers = groups(types,source,returns,oracle.forward,a)
    masks=assignments(dag)
    if exhaustive:
        expected=set()
        for raw in product(range(2),repeat=len(types)):
            valid=shifted(oracle,a,types,source,returns,raw)
            if valid:
                expected.add(raw)
    valid_bits=set()
    if mark is not None:
        row=mark
        mark_bytes=bytearray()
        for _ in range(len(oracle.forward)):
            mark_bytes.append(oracle.first[row])
            row=oracle.forward[row]
    outcomes = []
    sections = {}
    for mask in masks:
        bit=tuple((mask>>cl)&1 for cl in cid)
        candidate=shifted(oracle,a,types,source,returns,bit)
        if candidate is None:
            continue
        new,positions=candidate
        if exhaustive:
            selected=set(positions)
            for i,x in enumerate(positions):
                word=bytearray()
                while True:
                    word.append(oracle.first[x])
                    x=oracle.forward[x]
                    if x in selected:
                        break
                assert x==positions[returns[i]] and word==new[source[i]]
        valid_bits.add(bit)
        pieces=1 + sum(x+1 != y for x,y in zip(positions,positions[1:]))
        sections[mask]=pieces
        outcomes.append((suffixes(new),sum(map(len,new)),sum(bit),mask))
        if mark is not None:
            # Compare actual output from a mark that need not start in the section.
            n=len(oracle.forward)
            row=mark
            out=bytearray()
            while not a<=row<b and len(out)<n:
                out.append(oracle.first[row])
                row=oracle.forward[row]
            if len(out)<n:
                i=row-a
                if bit[source[i]]:
                    out.append(oracle.first[row])
                while len(out)<n:
                    w=new[source[i]]
                    out.extend(w[:n-len(out)])
                    i=returns[i]
            assert out==mark_bytes
    if exhaustive:
        assert valid_bits == expected
    uniform0=next(at for at in outcomes if at[2]==0)
    uniform1=next(at for at in outcomes if at[2]==len(types))
    mixed=[x for x in outcomes if 0<x[2]<len(types)]
    best=min(mixed or [(0,0,0,0)])
    return dict(types=len(types),mass=b-a,components=len(dag),largest=max(map(len,members)),
                closed_masks=len(masks),injective=len(outcomes),mixed=len(mixed),
                Q0=uniform0[0],Q1=uniform1[0],Qmixed=best[0],D0=uniform0[1],Dmixed=best[1],
                fragments_uniform=sections[uniform1[3]],fragments_best_mixed=sections[best[3]] if mixed else 0,
                useful_mixed=sum(x[0]<uniform0[0] and sections[x[3]]<sections[uniform1[3]] for x in mixed))


def small():
    counts=Counter()
    witness=None
    beats=None
    for n in range(1,9):
        for raw in product(range(2),repeat=n):
            obj=Oracle(bytes(raw))
            for a in range(n):
                for b in range(a+1,n+1):
                    if obj.first[a]!=obj.first[b-1]:
                        continue
                    row=analyze(obj,(a,b),True)
                    counts['sections']+=1
                    counts['valid_assignments']+=row['injective']
                    counts['nonuniform']+=row['mixed']
                    if row['mixed'] and row['Qmixed'] < row['Q1']:
                        counts['sections_beating_uniform']+=1
                        if beats is None:
                            beats=dict(column=bytes(raw).hex(),section=[a,b],**row)
                    if row['mixed'] and not witness:
                        witness=dict(column=bytes(raw).hex(),section=[a,b],**row)
    import json
    print(json.dumps(dict(counters=counts,first_mixed_witness=witness,mixed_beats_uniform=beats),indent=2))


def real(tapes):
    sys.path.insert(0,str(here.parents[7]/'docs/journal/2026-11-18-astra-c-output-first'))
    from inputs import paths,read_input
    writer=None
    for tape in paths(tapes):
        idx=int(tape.name.split('.')[0])
        runs,mark=read_input(tape)
        col=Column(runs)
        oracle=Oracle(b''.join(bytes([c])*width for c,width in runs))
        quote=col.interval(b'"')
        deep=heavy_set(col,quote,1)[0][0]
        for label,sec in [('quote',quote),('frequent',deep)]:
            row=dict(ordinal=idx,section=label,**analyze(oracle,sec,mark=mark))
            if writer is None:
                writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator='\n')
                writer.writeheader()
            writer.writerow(row)
            sys.stdout.flush()


if __name__=='__main__':
    real(Path(sys.argv[1])) if len(sys.argv)>1 else small()
