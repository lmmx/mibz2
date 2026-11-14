"""Independent tiny oracle for a labelled first-return origin annotation.

Retain delimiter rows only. During the existing interval traversal, record the
source-anchor rank and emitted-prefix length when the current interval contains
the original marked row. No separate search from that row is used by the
candidate. Expanded tuples are suitable only for these tiny correctness tests.
This is not a production implementation, cost census, or speed measurement.
"""
from bisect import bisect_right
from collections import Counter, deque
from itertools import product
import json


def lf_pieces(column):
    counts = Counter(column)
    cursor, total = {}, 0
    for char in sorted(counts):
        cursor[char] = total
        total += counts[char]
    pieces = []
    start = 0
    while start < len(column):
        stop = start + 1
        while stop < len(column) and column[stop] == column[start]:
            stop += 1
        char = column[start]
        pieces.append((start, stop, cursor[char], char))
        cursor[char] += stop - start
        start = stop
    return pieces


def candidate(column, origin, delimiter):
    pieces = lf_pieces(column)
    starts = [row[0] for row in pieces]
    anchors = [x for x, char in enumerate(column) if char == delimiter]
    if not anchors:
        return None
    rank = {x: i for i, x in enumerate(anchors)}
    queue = deque((rank[start], start, stop - start, ())
                  for start, stop, _, char in pieces if char == delimiter)
    returns, words, marker = {}, {}, None
    while queue:
        source_rank, x, width, prefix = queue.popleft()
        if x <= origin < x + width:
            assert marker is None, 'Selected row must appear in just one return excursion'
            marker = (source_rank + origin - x, len(prefix))
        start, stop, target, char = pieces[bisect_right(starts, x) - 1]
        assert x + width <= stop
        word = prefix + (char,)
        y = target + x - start
        end, j = y + width, bisect_right(starts, y) - 1
        offset = 0
        while y < end:
            b_start, b_stop, _, b_char = pieces[j]
            chunk = min(end, b_stop) - y
            if b_char == delimiter:
                for delta in range(chunk):
                    q = source_rank + offset + delta
                    assert q not in returns
                    returns[q] = rank[y + delta]
                    words[q] = word
            else:
                queue.append((source_rank + offset, y, chunk, word))
            y += chunk
            offset += chunk
            j += 1
    if marker is None:
        return None
    root, phase = marker
    q, forward = root, ()
    while True:
        forward += words[q]
        q = returns[q]
        if q == root:
            break
    reverse = bytes(forward[::-1])
    shift = (-phase) % len(reverse)
    cycle = reverse[shift:] + reverse[:shift]
    decoded = (cycle * ((len(column) + len(cycle) - 1) // len(cycle)))[:len(column)]
    return decoded, anchors[root], phase


def oracle(column, origin, delimiter):
    pieces = lf_pieces(column)
    f = {}
    for start, stop, target, _ in pieces:
        for x in range(start, stop):
            f[x] = target + x - start
    inverse = {y: x for x, y in f.items()}
    x, reaches_anchor = origin, False
    while True:
        reaches_anchor |= column[x] == delimiter
        x = f[x]
        if x == origin:
            break
    if not reaches_anchor:
        return None, f
    x, out = origin, bytearray()
    for _ in column:
        x = inverse[x]
        out.append(column[x])
    return bytes(out), f


def main():
    cases, successes, fallbacks = 0, 0, 0
    for n in range(1, 9):
        for values in product(b'ab', repeat=n):
            column = bytes(values)
            for origin in range(n):
                for delimiter in b'ab':
                    got = candidate(column, origin, delimiter)
                    expected, f = oracle(column, origin, delimiter)
                    cases += 1
                    if expected is None:
                        assert got is None
                        fallbacks += 1
                    else:
                        assert got is not None
                        decoded, anchor, phase = got
                        assert decoded == expected, (column, origin, delimiter, got, expected)
                        x = anchor
                        for _ in range(phase):
                            x = f[x]
                        assert x == origin
                        successes += 1
    # A genuine cyclic BWT: text bab -> column bba, original row 1.
    # Adding row 1 to delimiter-a section {2} produces tokens b,ba.
    # Their sorted-token LF is identity, but induced LF exchanges the two rows.
    column, origin, delimiter = b'bba', 1, ord('a')
    _, f = oracle(column, origin, delimiter)
    keep = sorted({i for i, c in enumerate(column) if c == delimiter} | {origin})
    rank = {x: i for i, x in enumerate(keep)}
    induced, tokens = [], []
    for start in keep:
        x, emitted = start, bytearray()
        while True:
            emitted.append(column[x])
            x = f[x]
            if x in rank:
                break
        induced.append(rank[x])
        tokens.append(bytes(emitted[::-1]))
    cursors = {token: sum(other < token for other in tokens) for token in set(tokens)}
    token_lf = []
    for token in tokens:
        token_lf.append(cursors[token])
        cursors[token] += 1
    assert induced == [1, 0] and token_lf == [0, 1]
    witness = dict(text='bab', bwt=column.decode(), original_row=origin, delimiter='a',
                   augmented_section=keep, forward_text_tokens=[s.decode() for s in tokens],
                   induced_map=induced, token_lf=token_lf)
    print(json.dumps(dict(cases=cases, successful_origin_annotations=successes,
                         correct_no_anchor_fallbacks=fallbacks, mismatches=0,
                         anchor_augmentation_counterexample=witness,
                         scope='All binary columns lengths 1..8, every origin and both '
                               'delimiters. Correctness only; no constructor optimisation, '
                               'real corpus, timing or decoder integration.'), indent=2))

if __name__ == '__main__':
    main()
