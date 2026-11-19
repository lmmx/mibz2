"""Targeted stable-column witnesses for sparse lane retirement and root fusion."""
import json
from itertools import product


def consecutive(rows):
    intervals = []
    for rank in sorted(rows):
        if not intervals or rank != intervals[-1][1]:
            intervals.append([rank, rank + 1])
        else:
            intervals[-1][1] += 1
    return intervals


def truth(last, low, high, depth=3):
    n = len(last)
    psi = sorted(range(n), key=last.__getitem__)
    lf = [0] * n
    for x, z in enumerate(psi):
        lf[z] = x
    f = [last[z] for z in psi]
    levels, prefix = [list(range(low, high))], []
    for _ in range(depth):
        symbols = {f[x] for x in levels[-1]}
        if len(symbols) != 1:
            return None
        prefix += list(symbols)
        levels.append([psi[x] for x in levels[-1]])
        assert levels[-1] == sorted(levels[-1])
    V = levels[-1]
    index = {z: i for i, z in enumerate(V)}
    incoming, source, early, late = [], [], [], []
    active = {}
    for i, y in enumerate(V):
        x = y
        labels, key = [], ()
        while True:
            if len(labels) >= depth:
                active.setdefault(key, []).append(x)
                key += (last[x],)
            elif x != levels[depth-len(labels)][i]:
                raise AssertionError("root levels lost original destination order")
            labels.append(last[x])
            x = lf[x]
            if x in index:
                break
        w = tuple(reversed(labels))
        s = index[x]
        source.append(s)
        incoming.append(w)
        if len(w) <= depth:
            assert w == tuple(prefix[depth - len(w):])
            early.append([i, len(w), s])
        else:
            assert w[-depth:] == tuple(prefix)
            late.append(i)
    # Check the real hole content and the minimum deferred storage exactly once per child.
    holes, root = [], None
    for key, members in active.items():
        spans = consecutive(members)
        rebuilt = list(range(spans[0][0], spans[-1][1]))
        gaps = [[a[1], b[0]] for a,b in zip(spans,spans[1:])]
        for start, stop in reversed(gaps):
            del rebuilt[start-spans[0][0]:stop-spans[0][0]]
        assert rebuilt == sorted(members)
        if not key:root = spans
        if gaps:
            holes.append(dict(backward_prefix=''.join(map(str,key)),packets=spans,gaps=gaps))
        children = {}
        for x in sorted(members):
            c, y = last[x], lf[x]
            if y not in index:
                children.setdefault(c, []).append(y)
        for c, values in children.items():
            assert values == sorted(values) == sorted(active[key+(c,)])
    if not active:
        assert len(early) == len(V)
    else:
        assert sorted(active[()]) == [levels[0][i] for i in late]
    # Composed stable token transport and independently marked expansion.
    bases = {}
    for i in sorted(range(len(source)),key=source.__getitem__):
        w = incoming[i]
        if w not in bases:bases[w] = source[i]
    cursor = dict(bases)
    links, words = {}, {}
    for i,w in enumerate(incoming):
        s = cursor[w]
        assert s == source[i]
        cursor[w] += 1
        links[s],words[s] = i,w
    for p in range(n):
        forward, x = [], p
        for _ in range(n):forward.append(f[x]);x=psi[x]
        x=p;skip=0
        while x not in index and skip<n:
            x=lf[x];skip+=1
        if x in index:
            data=[];j=index[x]
            while len(data)<n:
                data.extend(words[j][skip:n-len(data)+skip]);skip=0;j=links[j]
            assert data == forward
    return dict(last=''.join(map(str,last)), section=[low,high], prefix=''.join(map(str,prefix)),
                levels=levels, target_F=sorted({f[x] for x in V}),
                early=early, late=late, root_packets=root, sparse=holes,
                incoming=[''.join(map(str,w)) for w in incoming], source=source)


def search():
    out={}
    for n in range(1,12):
        for last in product((0,1),repeat=n):
            for low in range(n):
                for hi in range(low+1,n+1):
                    obj=truth(last,low,hi)
                    if obj is None:continue
                    early,late=obj['early'],obj['late']
                    target=obj['levels'][-1]
                    early_ord={i for i,_,_ in early}
                    tests={
                        "all_early_nonconstant_prefix": bool(early) and not late and len(set(obj['prefix']))>1,
                        "fourth_symbol_not_uniform":len(obj['target_F'])>1,
                        "holed_target_with_nonuniform_fourth_symbol":len(consecutive(target))>1 and len(obj['target_F'])>1,
                        "holed_fused_root":len(obj['root_packets'] or [])>1,
                        "early_between_late":any(min(late)<i<max(late) for i in early_ord) if late else False,
                        "two_early_lengths_and_late":len({l for _,l,_ in early})>1 and bool(late),
                        "interleaved_early_with_moved_source":any(i!=s and min(late)<i<max(late) for i,_,s in early) if late else False,
                    }
                    for name,ok in tests.items():
                        if ok and name not in out:out[name]=obj
                    if len(out)==len(tests):
                        return out
    return out


if __name__=="__main__":
    print(json.dumps(search(),sort_keys=True,indent=2))
