"""A fixed-alphabet feasible cyclic return dictionary with a large mixed-direction gain."""
from itertools import groupby

from cut import Column, Direct, build, paid

for m in (2, 4, 8, 16, 32):
    width = (m - 1).bit_length()
    code = [bytes(2 + int(c) for c in f"{i:0{width}b}") for i in range(m)]
    length = 16 * m
    words = [b"\x00" + b"\x01" * length + s for s in code]
    words += [b"\x00" + s + b"\x04" * length for s in code]
    text = b"".join(words)
    # Cutting a single cyclic source at the 2m delimiter occurrences recovers precisely these types.
    starts = [i for i, c in enumerate(text) if c == 0]
    assert [text[a:b] for a,b in zip(starts, starts[1:]+[len(text)])] == words
    if m <= 4:
        # Also build its actual cyclic BWT and independently recover the marked dictionary.
        rotations = sorted((text[i:] + text[:i], i) for i in range(len(text)))
        col = Column([(c, sum(1 for _ in group)) for c,group in groupby(r[-1] for r,_ in rotations)])
        decoded = Direct(col, [col.interval(b"\x00")])
        assert set(decoded.word_counts) == set(words)
        assert decoded.expand(next(j for j,(_, i) in enumerate(rotations) if i == 0)) == text
    r, assignment = build(words)
    assert paid(words, assignment) == r["minimum"] < min(r["forward"], r["backward"])
    print(f"m={m} L={length} forward={r['forward']} backward={r['backward']} minimum={r['minimum']}")
