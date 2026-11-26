"""Cache an affine return-to-delimiter and a spelling handle for an interval inside each L run."""
from bisect import bisect_right
from collections import Counter

from gaps import compile_runs, construct, expand


def label(runs, delimiter, starts, policy):
    """Given true token-run starts, assign each a word and its destination rank."""
    n, base, counts, top, _ = compile_runs(runs)
    q, tokens = base[delimiter], counts[delimiter]
    ends = [end for _, end, _, _ in top]
    # A certificate (begin, end, delta, word_id) maps rows to terminal ranks.
    cache = [[] for _ in runs]
    nodes = [(-1, None)]
    children = {}
    terminal_ids = {}
    dictionary = []
    bands = []
    stats = Counter()
    for start, end in zip(starts, starts[1:]):
        row = q + start
        path = []
        while True:
            k = bisect_right(ends, row)
            stats["run_lookups"] += 1
            memo = cache[k]
            got = None
            for cert in reversed(memo):
                stats["certificate_tests"] += 1
                if cert[0] <= row < cert[1]:
                    got = cert
                    break
            if got is not None:
                stats["hits"] += 1
                break
            a, b, c, e = top[k]
            delta = e - a
            if c == delimiter:
                got = (a, b, delta - q, 0)
                break
            path.append((k, a, b, c, delta))
            row += delta

        lo, hi, dest_delta, node = got
        for k, a, b, c, delta in reversed(path):
            lo = max(a, lo - delta)
            hi = min(b, hi - delta)
            assert lo < hi
            dest_delta += delta
            key = (node, c)
            next_id = children.get(key)
            if next_id is None:
                next_id = len(nodes)
                nodes.append(key)
                children[key] = next_id
            node = next_id
            cert = (lo, hi, dest_delta, node)
            stats["writes"] += 1
            if policy == "last":
                cache[k] = [cert]
            elif policy == "all":
                cache[k].append(cert)
            elif policy == "wide":
                if not cache[k] or hi-lo >= cache[k][0][1] - cache[k][0][0]:
                    cache[k] = [cert]
            else:
                raise ValueError(policy)
        # The returned certificate is maximal or a subset shared at a cache hit.
        # Section entries in this true BWT run are affine translates, independent of certificate width.
        assert lo <= q + start < hi
        code = terminal_ids.get(node)
        if code is None:
            code = len(dictionary)
            terminal_ids[node] = code
            source = node
            text = bytearray()
            while source:
                source, c = nodes[source]
                text.append(c)
            text.reverse()
            text.append(delimiter)
            dictionary.append(bytes(text))
        bands.append((start, end - start, q + start + dest_delta, code))
    stats.update(stored=sum(map(len, cache)), trie_nodes=len(nodes),
                 dictionary_bytes=sum(map(len,dictionary)))
    assert sum(b[1] for b in bands) == tokens <= n
    return bands, dictionary, stats


def compare(runs,p,d,policies):
    base, c = construct(runs,p,d)
    if base is None:
        return None
    arcs, words, mark, skip = base
    starts=[a[0] for a in arcs]+[c["tokens"]]
    metrics={}
    for policy in policies:
        ab, aw, stats=label(runs,d,starts,policy)
        assert all((s,w,dst,words[i])==(x,h,y,aw[j]) for (s,w,dst,i),(x,h,y,j) in zip(arcs,ab))
        assert len(ab)==len(arcs)
        assert sum(map(len,aw))==sum(map(len,words))
        metrics[policy]=stats
    return c,metrics
