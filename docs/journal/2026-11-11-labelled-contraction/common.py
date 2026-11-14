"""Shared count units, unchanged source imports and fixed synthetic inputs."""

import sys
from collections import Counter
from itertools import product
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "vendor"))
import batch_induction as original
import phrase_induction as phrase

Piece = original.Piece


def bisect_right(a, x, c, category="search"):
    """Count actual binary-search comparisons, not log-size estimates."""
    c[category + "_queries"] += 1
    lo, hi = 0, len(a)
    while lo < hi:
        c[category + "_comparisons"] += 1
        mid = (lo + hi) // 2
        if x < a[mid]:
            hi = mid
        else:
            lo = mid + 1
    return lo


def ordered(values, key, c):
    """Count key records and TimSort comparisons."""

    class Key:
        def __init__(self, value):
            self.value = value

        def __lt__(self, other):
            c["sort_comparisons"] += 1
            return self.value < other.value

    c["sorts"] += 1
    c["sort_items"] += len(values)
    return sorted(values, key=lambda value: Key(key(value)))


class Grammar(original.Grammar):
    """The original shared word DAG, instrumented without changing identities."""

    def __init__(self, c):
        super().__init__()
        self.c = c
        c["grammar_literal_nodes"] += 256

    def cat(self, a, b):
        """Intern a word pair and count the actual dictionary operation."""
        self.c["word_pair_queries"] += 1
        before = len(self.nodes)
        result = super().cat(a, b)
        self.c["grammar_nodes_created"] += len(self.nodes) - before
        return result

    def expand(self, word):
        """Count DAG visits and emitted bytes, including repeated expansions."""
        out, stack = bytearray(), [word]
        while stack:
            self.c["word_node_visits"] += 1
            value = self.nodes[stack.pop()]
            if isinstance(value, int):
                out.append(value)
                self.c["expanded_bytes"] += 1
            else:
                stack.extend((value[1], value[0]))
        return bytes(out)


def oracle(column):
    """Build LF independently by stable sorting individual row indices."""
    inverse = sorted(range(len(column)), key=column.__getitem__)
    forward = [0] * len(column)
    for y, x in enumerate(inverse):
        forward[x] = y
    return forward, inverse


def decode_oracle(column, origin):
    """Emit n inverse-LF bytes, even when the cycle length does not divide n."""
    _, inverse = oracle(column)
    result = bytearray()
    for _ in column:
        origin = inverse[origin]
        result.append(column[origin])
    return bytes(result)


def inputs():
    """Yield exactly the pre-data column families and delimiter alphabets."""
    for alphabet, bound in ((b"ab", 6), (b"abc", 4)):
        for n in range(1, bound + 1):
            for letters in product(alphabet, repeat=n):
                yield "exhaustive_" + alphabet.decode(), bytes(letters), alphabet
    for size in (1, 2, 4, 8, 16, 32, 64):
        for name, col in (
            ("aligned", b"b" * size + b"a" * size),
            ("excursion", b"b" + b"a" * size),
            ("alternating", b"ab" * size),
        ):
            yield name, col, b"ab"
        for base in (b"ab", b"aba", b"abc", b"aab"):
            col, _ = original.bwt(base * size)
            yield "periodic_" + base.decode(), col, bytes(sorted(set(base)))


def ranges(rows):
    """Losslessly encode a sorted set of rows as half-open intervals."""
    out = []
    for x in sorted(rows):
        if out and out[-1][1] == x:
            out[-1][1] += 1
        else:
            out.append([x, x + 1])
    return out


def unranges(intervals):
    """Decode a fixture's retained/removed original-row identities."""
    return [x for lo, hi in intervals for x in range(lo, hi)]


def chosen_color(word, seed):
    """Use original seeds, plus the one frozen PR369 witness coloring."""
    return int(word == ord("b")) if seed == -1 else original.color(word, seed)


def initial(column, special=False, c=None):
    """Build the input map; special is PR369's already-induced regression."""
    c = Counter() if c is None else c
    pieces, _ = original.from_column(column)
    grammar = Grammar(c)
    if not special:
        return pieces, grammar, list(range(len(column)))
    assert column == b"babaaa"
    keep = [0, 1, 3, 4, 5]
    rank = {x: i for i, x in enumerate(keep)}
    f, _ = oracle(column)
    pieces = []
    for x in keep:
        y, word = f[x], column[x]
        while y not in rank:
            word = grammar.cat(word, column[y])
            y = f[y]
        a = Piece(rank[x], rank[y], 1, word)
        if pieces and (
            pieces[-1].s + pieces[-1].n == a.s
            and pieces[-1].t + pieces[-1].n == a.t
            and pieces[-1].w == a.w
        ):
            b = pieces.pop()
            a = Piece(b.s, b.t, b.n + 1, b.w)
        pieces.append(a)
    return pieces, grammar, keep
