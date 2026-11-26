"""Independently check direct word sections against F2c's run-stream constructor."""
import hashlib
import itertools
import json
from collections import Counter
from pathlib import Path
import sys

F2C = Path(__file__).resolve().parents[2] / "f2c"
sys.path.insert(0, str(F2C))
from second import Layer  # The implementation under comparison; all byte-row oracles below are independent.


def forward(psi, first, row, length):
    text = bytearray()
    for _ in range(length):
        text.append(first[row])
        row = psi[row]
    return row, bytes(text)


def to_next(psi, first, row, domain):
    text = bytearray()
    while True:
        text.append(first[row])
        row = psi[row]
        if row in domain:
            return row, bytes(text)


def previous_mark(lf, p, domain):
    row, offset = p, 0
    while row not in domain:
        row = lf[row]
        offset += 1
        if row == p:
            return None
    return row, offset


def prefix_rows(column, pattern):
    # FM backward search defines the byte prefix cylinder independent of token construction.
    lo, hi = 0, len(column)
    for c in reversed(pattern):
        start = sum(x < c for x in column)
        lo, hi = start + column[:lo].count(c), start + column[:hi].count(c)
    return list(range(lo, hi))


def emit(edges, p, offset, count):
    out = bytearray()
    while len(out) < count:
        p, a = edges[p]
        assert offset < len(a)
        out.extend(a[offset:offset + count - len(out)])
        offset = 0
    return bytes(out)


def verify(column, delimiters, marks, stats, only=None):
    n = len(column)
    psi = sorted(range(n), key=column.__getitem__)
    first = bytes(column[x] for x in psi)
    lf = [0] * n
    for x, y in enumerate(psi):
        lf[y] = x
    cycle_length = {}
    cycle_count = 0
    for x in range(n):
        if x in cycle_length:
            continue
        cycle_count += 1
        cycle, y = [x], psi[x]
        while y != x:
            cycle.append(y)
            y = psi[y]
        cycle_length.update(dict.fromkeys(cycle, len(cycle)))
    references = {p: forward(psi, first, p, n)[1] for p in marks}
    stats["columns"] += 1
    stats["multicycle_columns"] += int(cycle_count > 1)
    for d in delimiters:
        cuts = [x for x, c in enumerate(column) if c == d]
        cut_rank = {x: rank for rank, x in enumerate(cuts)}
        first_edges = [to_next(psi, first, x, cut_rank) for x in cuts]
        # Arbitrary dictionary names deliberately run opposite byte collation.
        words = sorted({word for _, word in first_edges}, reverse=True)
        names = {word: i for i, word in enumerate(words)}
        bases = [None] * len(words)
        token_l = [None] * len(cuts)
        for rank, (dst, word) in enumerate(first_edges):
            t = names[word]
            if bases[t] is None:
                bases[t] = rank
            token_l[cut_rank[dst]] = t
        token_runs = [(t, sum(1 for _ in run)) for t, run in itertools.groupby(token_l)]
        for t,w in enumerate(words):
            if only is not None and w != only:
                continue
            pattern = bytes([d]) + w
            length = len(pattern)
            starts = prefix_rows(column, pattern)
            assert starts == [x for x in range(n) if forward(psi, first, x, length)[1] == pattern]
            start_rank = {x: i for i, x in enumerate(starts)}
            after = [cuts[i] for i, token in enumerate(token_l) if token == t]
            translated = [forward(psi, first, x, length)[0] for x in starts]
            assert translated == after
            layer = Layer(token_runs, bases, words, t)
            assert len(layer.t) == len(starts)
            edges = {}
            for i,x in enumerate(starts):
                y, a = to_next(psi, first, x, start_rank)
                edges[x] = y, a
                dst, code = layer.t[i]
                q = layer.spell[code]
                assert dst == start_rank[y]
                assert (after[dst], q) == forward(psi, first, after[i], len(a))
                assert a + pattern == pattern + q
                shift = length % len(a)
                assert q == a[shift:] + a[:shift]
                if len(a) < length:
                    assert len(a) == length - 1
                    stats["overlap_arrows"] += 1
                stats["nonidentical_conjugates"] += int(a != q)
                stats["arrows"] += 1
            stats["sections"] += 1
            for p in marks:
                mark = previous_mark(lf, p, start_rank)
                token_mark = previous_mark(lf, p, cut_rank)
                contracted = None
                if token_mark is not None:
                    x, skip = token_mark
                    contracted = layer.expand(cut_rank[x], skip, n, words)
                stats["marked_cases"] += 1
                if mark is None:
                    assert contracted is None
                    stats["fallback_marks"] += 1
                    continue
                x, skip = mark
                assert emit(edges, x, skip, n) == contracted == references[p]
                stats["admitted_marks"] += 1
                stats["nondividing_admitted_marks"] += int(n % cycle_length[p] != 0)
                stats["interior_admitted_marks"] += int(skip > 0)


def column_for_text(text):
    rows = sorted(range(len(text)), key=lambda i: text[i:] + text[:i])
    return bytes(text[i - 1] for i in rows)


def main():
    result = {}
    for alphabet, max_length in ((2, 9), (3, 6)):
        stats = Counter()
        for n in range(1, max_length + 1):
            for col in itertools.product(range(alphabet), repeat=n):
                verify(bytes(col), range(alphabet), range(n), stats)
        result[f"all_{alphabet}-ary_columns_1_to_{max_length}"] = stats
    # The d + w marker recurs after every w: 65 adjacent hits share an endpoint.
    # A separator and a different d-ended word give nonconstant transition labels.
    # Tripling each L row lifts the orbit to three equal disjoint cycles.
    # Five high singleton rows force nondividing output lengths and true fallback marks.
    w = bytes([1]) * 17 + bytes([0])
    text = w * 65 + bytes([2, 1, 0, 2, 0])
    column = bytes(c for v in column_for_text(text) for c in (v, v, v)) + bytes([3]) * 5
    stats = Counter()
    verify(column, [0], list(range(0,len(column),43)) + [len(column)-1], stats, only=w)
    assert stats["overlap_arrows"] >= 3 * 64
    assert stats["nondividing_admitted_marks"] and stats["fallback_marks"]
    result["65_adjacent_18_byte_tokens_three_copies_plus_five_singletons"] = stats
    one = Counter()
    verify(bytes([0,1,1]), [0], [0,1,2], one, only=bytes([0]))
    assert one["overlap_arrows"] == one["admitted_marks"] == 1 and one["fallback_marks"] == 2
    result["dd_marker_longer_than_one_row_cycle"] = one
    result["f2c_constructor_sha256"] = hashlib.sha256((F2C/"second.py").read_bytes()).hexdigest()
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
