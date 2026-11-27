"""A small independent reference for the marked cyclic BWT quotient and RLE1 cut.

python3 verify_phrase_bwt.py
python3 verify_phrase_bwt.py COLUMN PRIMARY MARKER EXPECTED_PRE_RLE
The explicit row arrays make the proofs testable. The Rust constructors use runs.
"""

from collections import Counter, defaultdict
from itertools import groupby, product
from pathlib import Path
import sys


def bwt(text):
    """Reference BWT on a nonempty tuple, with the original rotation marked."""
    sa = sorted(range(len(text)), key=lambda i: text[i:] + text[:i])
    return tuple(text[(i - 1) % len(text)] for i in sa), sa.index(0)


def tables(last):
    """Build first-column offsets, symbol counts and a dense stable LF permutation."""
    counts = Counter(last)
    base, total = {}, 0
    for a in sorted(counts):
        base[a] = total
        total += counts[a]
    seen = base.copy()
    lf = []
    for a in last:
        lf.append(seen[a])
        seen[a] += 1
    return base, counts, lf


def cycle(last, primary):
    """Spell one marked LF cycle in forward order."""
    _, _, lf = tables(last)
    p, result = primary, []
    while True:
        result.append(last[p])
        p = lf[p]
        if p == primary:
            return tuple(reversed(result))


def reference(last, primary):
    """Repeat the marked cycle and truncate it to the length of the last column."""
    w = cycle(last, primary)
    return (w * ((len(last) + len(w) - 1) // len(w)))[: len(last)]


def factor(last, primary, marker):
    """Enumerate and check maximal equal-phrase runs starting at the marker bucket."""
    base, counts, lf = tables(last)
    if marker not in counts:
        return None
    b, m = base[marker], counts[marker]
    row, skip = primary, 0
    while last[row] != marker:
        row = lf[row]
        skip += 1
        if row == primary:
            return None
    marked_phrase = lf[row] - b
    room = [1] * len(last)
    for i in reversed(range(len(last) - 1)):
        if last[i] == last[i + 1]:
            room[i] = room[i + 1] + 1

    pieces, source = [], 0
    while source < m:
        row, width, backwards = b + source, m - source, []
        while True:
            width = min(width, room[row])
            a = last[row]
            row = lf[row]
            if a == marker:
                break
            backwards.append(a)
        word = tuple(reversed(backwards)) + (marker,)
        pieces.append((word, width, row - b))
        source += width
    dictionary = sorted({w for w, _, _ in pieces})
    ids = {w: i for i, w in enumerate(dictionary)}
    reduced = tuple(ids[w] for w, width, _ in pieces for _ in range(width))
    # A completely separate stable sort of the token column must give these images.
    expected = [v for _, width, dest in pieces for v in range(dest, dest + width)]
    assert tables(reduced)[2] == expected
    assert len(pieces) == sum(1 for _ in groupby(reduced))
    assert len(pieces) <= sum(1 for _ in groupby(last))
    return reduced, marked_phrase, skip, dictionary


def rebuild_byte_bwt(phrase_column, dictionary, marker):
    """Inverse stable routing. The cycles represented by this parse all contain the marker."""
    buckets = defaultdict(bytearray)
    for token, group in groupby(phrase_column):
        width = sum(1 for _ in group)
        w = dictionary[token]
        for j in reversed(range(len(w))):
            preceding = w[j - 1] if j else marker
            buckets[w[j:]].extend(bytes([preceding]) * width)
    return b"".join(buckets[s] for s in sorted(buckets))


def read_only_branches(last, dictionary, frequencies, marker):
    """Decode a supplied phrase multiset's order. Requires coverage of all input cycles."""
    words = [bytes(w) for w in dictionary]
    ids = {w: i for i, w in enumerate(words)}
    sizes = Counter()
    choices = defaultdict(set)
    for i, w in enumerate(words):
        for j in range(len(w)):
            suffix = w[j:]
            sizes[suffix] += frequencies[i]
            choices[suffix].add(w[j - 1] if j else marker)
    starts, end = {}, 0
    for suffix in sorted(sizes):
        starts[suffix] = end
        end += sizes[suffix]
    assert end == len(last)
    root = bytes([marker])
    result = [None] * sizes[root]
    stack = [(root, list(range(len(result))))]
    mixed_rows = mixed_runs = mixed_nodes = 0
    while stack:
        s, slots = stack.pop()
        while len(choices[s]) == 1:
            a = next(iter(choices[s]))
            if a == marker:
                for place in slots:
                    result[place] = ids[s]
                break
            s = bytes([a]) + s
        else:
            start = starts[s]
            # This is the only read of the original column.
            routing = last[start : start + len(slots)]
            mixed_nodes += 1
            mixed_rows += len(routing)
            mixed_runs += sum(1 for _ in groupby(routing))
            groups = defaultdict(list)
            for a, place in zip(routing, slots):
                if a == marker:
                    result[place] = ids[s]
                else:
                    groups[a].append(place)
            for a, subset in groups.items():
                stack.append((bytes([a]) + s, subset))
    return tuple(result), (mixed_nodes, mixed_runs, mixed_rows)


def rle1(word):
    """Decode one independent raw phrase, raising for a missing final count."""
    out, i = bytearray(), 0
    while i < len(word):
        a = word[i]
        if i + 3 < len(word) and all(v == a for v in word[i + 1 : i + 4]):
            if i + 4 == len(word):
                raise ValueError("RunUnfinished")
            out.extend([a] * (4 + word[i + 4]))
            i += 5
        else:
            out.append(a)
            i += 1
    return bytes(out)


def outcome(chunks):
    """Decode a sequence of independent chunks or report its terminal error."""
    try:
        return b"".join(rle1(chunk) for chunk in chunks)
    except ValueError:
        return "RunUnfinished"


def check(last, primary, marker):
    """Compare marked expansion and independently decoded chunks with the byte reference."""
    raw = reference(last, primary)
    result = factor(last, primary, marker)
    assert (result is not None) == (marker in raw)
    if result is None:
        return raw
    column, q, skip, dictionary = result
    ids = cycle(column, q)
    chunks, left, i = [], len(raw), 0
    while left:
        w = dictionary[ids[i % len(ids)]][skip:][:left]
        chunks.append(w)
        left, skip, i = left - len(w), 0, i + 1
    assert tuple(x for w in chunks for x in w) == raw
    # A local oracle for the sufficient condition. Rust tests c^4 using FM intervals.
    period = cycle(last, primary)
    if bytes([marker]) * 4 not in bytes(period) * (1 + (3 + len(period) - 1) // len(period)):
        assert outcome(chunks) == outcome([raw])
    return raw


def main():
    """Check small columns exhaustively, or validate the published frozen block."""
    if len(sys.argv) == 5:
        last = Path(sys.argv[1]).read_bytes()
        raw = check(last, int(sys.argv[2]), int(sys.argv[3]))
        assert bytes(raw) == Path(sys.argv[4]).read_bytes()
        c = int(sys.argv[3])
        col, _, _, dictionary = factor(last, int(sys.argv[2]), c)
        if len(cycle(last, int(sys.argv[2]))) == len(last):
            assert rebuild_byte_bwt(col, dictionary, c) == last
            recovered, counts = read_only_branches(last, dictionary, Counter(col), c)
            assert recovered == col
            print("mixed nodes, run slices and rows:", *counts)
        print("Frozen inverse and forward bucket reconstruction verified")
        return
    last, p = bwt(tuple(b"b#z#ab#z#y#a"))
    col, primary, skip, words = factor(last, p, ord("#"))
    assert (bytes(last), p, col, primary, skip) == (b"yzzbb##aa###", 7, (1, 2, 2, 0, 0), 0, 1)
    print("example", bytes(last), p, col, primary, skip, [bytes(w) for w in words])
    total = 0
    for alphabet, max_n in [(2, 7), (3, 5)]:
        for n in range(1, max_n + 1):
            for last in product(range(alphabet), repeat=n):
                for p in range(n):
                    for c in range(alphabet):
                        check(last, p, c)
                        total += 1
    # Check the phrase-BWT theorem by sorting rotations of an explicitly parsed text.
    direct = 0
    for n in range(1, 8):
        for t in product(range(3), repeat=n):
            last, p = bwt(t)
            for c in set(t):
                col, _, _, words = factor(last, p, c)
                i = t.index(c) + 1
                rotated = t[i:] + t[:i]
                start, tokens = 0, []
                for j, a in enumerate(rotated):
                    if a == c:
                        tokens.append(words.index(rotated[start : j + 1]))
                        start = j + 1
                assert bwt(tuple(tokens))[0] == col
                assert rebuild_byte_bwt(col, words, c) == bytes(last)
                assert read_only_branches(last, words, Counter(col), c)[0] == col
                direct += 1
    print(total, "arbitrary column/primary/marker checks")
    print(direct, "reference token BWT comparisons")


if __name__ == "__main__":
    main()
