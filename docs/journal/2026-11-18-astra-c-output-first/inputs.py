"""Only initial top runs, n and p leave the archived input adapter."""
import array
import hashlib
import itertools
import json
import lzma
from pathlib import Path
import sys


def read_input(path):
    with lzma.open(path, "rb") as stream:
        h = json.loads(stream.readline())
        assert h["format"] == "irec/1"
        # Both input arrays precede every induction array. Stop reading there.
        offset = 0
        arrays = []
        for name in ("l_byte", "l_lam"):
            start, count = h["arrays"][name]
            assert start == offset
            a = array.array("I")
            assert a.itemsize == 4
            a.frombytes(stream.read(4 * count))
            if sys.byteorder != "little":
                a.byteswap()
            assert len(a) == count
            arrays.append(a)
            offset += 4 * count
    runs = []
    for c, length in zip(*arrays, strict=True):
        assert 0 <= c < 256 and length > 0
        if runs and runs[-1][0] == c:
            runs[-1] = c, runs[-1][1] + length
        else:
            runs.append((c, length))
    assert sum(w for _, w in runs) == h["n"] and 0 <= h["p"] < h["n"]
    return runs, h["p"]


def paths(arg):
    p = Path(arg)
    return sorted(p.glob("*.irec.xz")) if p.is_dir() else [p]


def pieces(runs):
    """F-ordered array of (start, exclusive end, S displacement, emitted byte)."""
    freq, buckets = [0] * 256, [0] * 256
    for c, w in runs:
        freq[c] += w
        buckets[c] += 1
    base = list(itertools.accumulate([0] + freq[:-1]))
    slot = list(itertools.accumulate([0] + buckets[:-1]))
    entries = [None] * len(runs)
    top = 0
    for c, w in runs:
        left = base[c]
        entries[slot[c]] = left, left + w, top - left, c
        slot[c] += 1
        base[c] += w
        top += w
    return entries


def dense_audit(runs, p):
    """Independent stable inversion using row sort. For screens/verification only."""
    column = b"".join(bytes([c]) * w for c, w in runs)
    s = array.array("I", sorted(range(len(column)), key=column.__getitem__))
    row = p
    orbit = array.array("I", [row])
    output = bytearray()
    for _ in range(len(column)):
        row = s[row]
        output.append(column[row])
        orbit.append(row)
    return column, bytes(output), orbit


def digest(data):
    return hashlib.sha256(data).hexdigest()


def check_frozen(block, column, output, p):
    if block == 0:
        assert (len(column), p) == (899931, 264045)
        assert digest(column) == "db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67"
        assert digest(output) == "f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6"
