"""BWT of maximal strict ascent or descent phrases, built from the original runs.

No expanded column, decoded text, full row permutation or suffix sorting is
used by transform(). The oracle routines follow transform() below.
"""
from bisect import bisect_right
from collections import Counter
from itertools import accumulate


def compress(items):
    result = []
    for value in items:
        if result and result[-1][0] == value:
            result[-1] = value, result[-1][1] + 1
        else:
            result.append((value, 1))
    return result


def transform(runs, primary, ascending):
    freq = [0] * (1 + max(c for c, _ in runs))
    for c, w in runs:
        freq[c] += w
    bounds = list(accumulate(freq))
    n = bounds[-1]
    base = [0] + bounds[:-1]
    # On an L-run/F-bucket intersection the predicate is constant and LF
    # translates by a constant. Store these intervals in original rank order.
    source = [[] for _ in freq]
    row_ends, row_info = [], []
    lfpos = base.copy()
    x = c = 0
    for a, width in runs:
        while width:
            while x >= bounds[c]:
                c += 1
            w = min(width, bounds[c] - x)
            y = lfpos[a]
            boundary = a >= c if ascending else a <= c
            source[c].append((x, x + w, a, y, boundary))
            row_ends.append(x + w)
            row_info.append((boundary, y - x))
            x += w
            width -= w
            lfpos[a] += w
    # Left extension resolves a swallowed primary in at most alphabet - 1 steps.
    anchor = primary
    skip = 0
    while True:
        boundary, delta = row_info[bisect_right(row_ends, anchor)]
        if boundary:
            break
        anchor += delta
        skip += 1
    stats = Counter(input_runs=len(runs), overlays=len(row_ends), mark_steps=skip)
    # A trie node is the exact prefix word followed by one byte. Queries by
    # (parent, byte) intern a node; no string comparison is needed.
    parents = [(-1, -1)]
    ids = {}
    children = [[]]

    def append_byte(parent, byte):
        key = (parent, byte)
        name = ids.get(key)
        if name is None:
            name = len(parents)
            ids[key] = name
            parents.append(key)
            children.append([])
            children[parent].append((byte, name))
        return name

    # P[r] is the current phrase prefix ending at F[r]. Equal consecutive P
    # values are stored as one interval. Every lookup goes to an earlier
    # stage of this acyclic computation. This implementation binary-searches
    # the initial interval of each translated query.
    pre_ends = [[] for _ in freq]
    pre_ids = [[] for _ in freq]

    def lookup(a, y, w):
        stops, names = pre_ends[a], pre_ids[a]
        j = bisect_right(stops, y)
        stop = y + w
        while y < stop:
            end = min(stop, stops[j])
            yield names[j], end - y
            y = end
            j += 1

    order = range(len(freq)) if ascending else reversed(range(len(freq)))
    for c in order:
        ends, names = pre_ends[c], pre_ids[c]
        for x, hi, a, y, boundary in source[c]:
            if boundary:
                pieces = [(0, hi - x)]
            else:
                pieces = lookup(a, y, hi - x)
                stats["prefix_searches"] += 1
            for parent, w in pieces:
                stats["prefix_pieces"] += 1
                name = append_byte(parent, c)
                x += w
                if names and names[-1] == name:
                    ends[-1] = x
                else:
                    ends.append(x)
                    names.append(name)
            assert x == hi

    token_runs = []
    token_rows = 0
    rank = None
    for spans in source:
        for x, hi, a, y, boundary in spans:
            if not boundary:
                continue
            if x <= anchor < hi:
                rank = token_rows + anchor - x
            stats["token_searches"] += 1
            for name, w in lookup(a, y, hi - x):
                stats["token_pieces"] += 1
                token_rows += w
                if token_runs and token_runs[-1][0] == name:
                    token_runs[-1] = name, token_runs[-1][1] + w
                else:
                    token_runs.append((name, w))
    assert rank is not None
    # Lexicographic names from the trie. At a shared prefix the completed
    # strict ascent precedes its extensions. A descent follows its extensions.
    present = {name for name, _ in token_runs}
    code = {}
    dictionary = []

    def publish(node):
        if node in present:
            code[node] = len(dictionary)
            dictionary.append(node)

    def visit(node):
        if ascending:
            publish(node)
        for _, child in sorted(children[node]):
            visit(child)
        if not ascending:
            publish(node)
    visit(0)
    stats.update(prefix_intervals=sum(map(len, pre_ids)), trie_nodes=len(parents) - 1,
                 token_rows=token_rows, token_runs=len(token_runs), alphabet=len(dictionary))
    return [(code[name], w) for name, w in token_runs], rank, skip, parents, dictionary, stats


def spelling(node, parents):
    word = []
    while node:
        node, c = parents[node]
        word.append(c)
    return word[::-1]


def inverse(column, primary, n=None):
    # Independent reference. Stable sorting the original row indices constructs psi.
    psi = sorted(range(len(column)), key=column.__getitem__)
    output = []
    row = primary
    for _ in range(len(column) if n is None else n):
        row = psi[row]
        output.append(column[row])
    return output


def expanded_marked(result, n):
    runs, primary, offset, parents, nodes, _ = result
    col = [c for c, w in runs for _ in range(w)]
    # Close the marked token cycle once. n need not be divisible by its expanded length.
    psi = sorted(range(len(col)), key=col.__getitem__)
    words = [spelling(node, parents) for node in nodes]
    period = []
    row = primary
    while True:
        row = psi[row]
        period.extend(words[col[row]])
        if row == primary:
            break
    length = n + offset
    return (period * ((length + len(period) - 1) // len(period)))[offset:length]
