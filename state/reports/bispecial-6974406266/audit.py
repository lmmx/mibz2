"""Independent SUP1, multiplicity, and exact cyclic spectrum audit; uses NumPy."""
from collections import Counter, deque
from csv import DictReader
import hashlib
from itertools import permutations, product
import json
from math import comb, gcd
from pathlib import Path
import struct
import numpy as np

ROOT = Path(__file__).resolve().parent
E = ROOT / "evidence"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def decode_tape(tape):
    """Decode only SUP1, using separate node arrays and a flat ordered output."""
    magic, n, mark, mask, length = struct.unpack_from("<4sII32sQ", tape)
    assert magic == b"SUP1" and len(tape) == 52 + (length + 7) // 8
    alphabet = [c for c in range(256) if mask[c >> 3] & (1 << (c & 7))]
    children = [None]
    left = [None]
    queue = deque()
    cursor = processed = mixed = 0

    def create_split(node, width):
        assert children[node] is None and width >= 2
        start = len(children)
        children[node] = range(start, start + width)
        children.extend([None] * width)
        left.extend([None] * width)
        queue.append(node)
        return children[node]

    def connect(node, entry):
        if left[node] is None:
            left[node] = []
        left[node].append(entry)

    bits = np.unpackbits(np.frombuffer(tape, dtype=np.uint8, offset=52), bitorder="little")
    if len(alphabet) == 1:
        left[0] = [(alphabet[0], 0)]
    else:
        left[0] = list(zip(alphabet, create_split(0, len(alphabet))))
    while queue:
        v = queue.popleft()
        sub = children[v]
        incoming = left[v]
        left[v] = None
        processed += 1
        branching = len(incoming) > 1
        mixed += branching
        for a, target in incoming:
            if branching:
                present = [u for u, bit in zip(sub, bits[cursor:cursor+len(sub)]) if bit]
                cursor += len(sub)
            else:
                present = sub
            assert present
            if len(present) == 1:
                connect(present[0], (a, target))
            else:
                for u, image in zip(present, create_split(target, len(present))):
                    connect(u, (a, image))
    assert cursor == length
    leaves = []
    stack = [0]
    while stack:
        v = stack.pop()
        if children[v] is None:
            assert len(left[v]) == 1
            leaves.append(v)
        else:
            stack.extend(reversed(children[v]))
    rank = {v: i for i, v in enumerate(leaves)}
    predecessors = bytes(left[v][0][0] for v in leaves)
    lf = [rank[left[v][0][1]] for v in leaves]
    assert len(set(lf)) == len(leaves)
    word = marked_bytes(predecessors, lf, mark)
    raw = (word * ((n + len(word) - 1) // len(word)))[:n]
    column = None
    if len(word) == len(leaves) and n % len(word) == 0:
        column = b"".join(bytes([x]) * (n // len(word)) for x in predecessors)
    return raw, column, dict(n=n, mark=mark, alphabet=len(alphabet), bits=length,
                             contexts=processed, mixed=mixed, classes=len(leaves),
                             period=len(word), total_nodes=len(children))


def lf_of(last):
    # Rank a row by a stable argsort, then use inverse permutation as LF.
    order = np.argsort(np.frombuffer(last, dtype=np.uint8), kind="stable")
    lf = np.empty(len(last), dtype=np.int32)
    lf[order] = np.arange(len(last), dtype=np.int32)
    return lf.tolist()


def marked_bytes(last, lf, primary):
    row = primary
    backward = bytearray()
    while True:
        backward.append(last[row])
        row = lf[row]
        if row == primary:
            return bytes(reversed(backward))


def cycle_sizes(lf):
    seen = bytearray(len(lf))
    sizes = []
    for i in range(len(lf)):
        if seen[i]:
            continue
        row, size = i, 0
        while not seen[row]:
            seen[row], size = 1, size + 1
            row = lf[row]
        assert row == i
        sizes.append(size)
    return sorted(sizes)


def primitive(word):
    return (word + word).find(word, 1) == len(word)


def direct_spectra(a, b):
    """Joint exact doubling ranks with integer sort, followed by cyclic count balances."""
    n = len(a)
    assert n == len(b)
    ids = np.frombuffer(a+b, dtype=np.uint8).reshape(2, n).astype(np.uint64)
    span = 1
    while span < 4096:
        key = (ids << 32) | np.roll(ids, -span, axis=1)
        _, ids = np.unique(key.ravel(), return_inverse=True)
        ids = ids.reshape(2, n).astype(np.uint64)
        span *= 2
    results = []
    for length in (4744, 4745):
        assert span <= length <= 2*span
        key = (ids << 32) | np.roll(ids, span-length, axis=1)
        keys, groups = np.unique(key.ravel(), return_inverse=True)
        a_count = np.bincount(groups[:n], minlength=len(keys))
        b_count = np.bincount(groups[n:], minlength=len(keys))
        delta = a_count-b_count
        ids_changed = np.flatnonzero(delta)
        examples = []
        for q in ids_changed:
            where = int(np.flatnonzero(groups == q)[0])
            s = a if where < n else b
            at = where % n
            data = (s+s)[at:at+length]
            examples.append([int(a_count[q]), int(b_count[q]), digest(data)])
        results.append(dict(length=length, types=len(keys), differences=len(ids_changed),
                            l1=int(np.abs(delta).sum()), changed_counts=examples))
    return results


def witnesses(last, original, mark):
    off = 138927
    assert last[off:off+3] == b"mne"
    perms = {}
    primitive_text = None
    for candidate in permutations(b"mne"):
        key = bytes(candidate)
        column = last[:off] + key + last[off+3:]
        lf = lf_of(column)
        sizes = cycle_sizes(lf)
        perms[key.hex()] = sizes
        if key == b"emn":
            assert sizes == [len(last)]
            primitive_text = marked_bytes(column, lf, mark)
    assert primitive(original) and primitive(primitive_text)
    assert primitive_text not in original + original
    assert perms[b"men".hex()] == [442958, 456973]
    end = direct_spectra(original, primitive_text)
    assert [(r["types"],r["differences"],r["l1"]) for r in end] == [(899853,0,0),(899856,4,4)]
    return dict(cycles=perms, alternative_sha256=digest(primitive_text), spectra=end)


def rle1(raw):
    out = bytearray()
    prev = None
    count = 0
    for a in raw:
        if count == 4:
            out.extend(bytes([prev]) * a)
            prev, count = None, 0
        else:
            count = count + 1 if a == prev else 1
            out.append(a)
            prev = a
    assert count != 4
    return bytes(out)


def rle_cut_safe(t, c):
    return c*4 not in t+t[:3]


def section_family():
    texts = [b"#|" + b"|".join(bits) + b"|" for bits in product((b"0",b"1"), repeat=8)
             if bits.count(b"1")==4]
    assert len(texts)==comb(8,4)
    histograms = {tuple(sorted(Counter(t).items())) for t in texts}
    orders = set()
    for t in texts:
        cut = t[2:] + t[:2]
        words = cut.split(b"|")[:-1]
        assert Counter(words) == {b"#":1, b"0":4, b"1":4}
        assert rle_cut_safe(t,b"#") and rle_cut_safe(t,b"|")
        orders.add(tuple(words))
    assert len(histograms)==1 and len(orders)==len(texts)
    return len(texts)


def csv_summary(name):
    with (E/name).open() as source:
        rows = list(DictReader(source))
    if name.endswith("levels.csv"):
        return {k:sum(int(r[k]) for r in rows) for k in ("right_special","bispecial","support_tests")}
    assert [int(r["block"]) for r in rows] == list(range(len(rows)))
    assert all(int(r["support_tape_bytes"])==52+(int(r["support_bits"])+7)//8 for r in rows)
    assert all(r["n"]==r["marked_period"]==r["terminal_classes"] for r in rows)
    return dict(blocks=len(rows), **{k:sum(int(r[k]) for r in rows) for k in
                    ("n","output_bytes","right_special","bispecial","support_tape_bytes")})


def certify_threshold(last, raw, mark):
    """Read mixed predecessor intervals from exact lexical ranks on reference bytes."""
    count = len(raw)
    ids = np.frombuffer(raw, dtype=np.uint8).astype(np.uint64)
    span = 1
    while span < 4096:
        keys = (ids << 32) | np.roll(ids, -span)
        _, inverse = np.unique(keys, return_inverse=True)
        ids = inverse.astype(np.uint64)
        span *= 2

    def prefix_key(k):
        return (ids << 32) | np.roll(ids, span-k)

    # 4823-byte prefixes distinguish all rotations here, independently recovering row order.
    order = np.argsort(prefix_key(4823), kind="stable")
    assert len(np.unique(prefix_key(4823))) == count
    assert int(order[mark]) == 0
    prior = np.frombuffer(raw, dtype=np.uint8)[(order-1) % count]
    assert prior.tobytes() == last
    intervals = {}
    for k in (4743, 4744, 4822):
        keys = prefix_key(k)[order]
        # A group has a mixed predecessor exactly when an interior boundary changes byte.
        changed = np.flatnonzero((keys[:-1] == keys[1:]) & (prior[:-1] != prior[1:]))
        hits = {}
        for i in changed:
            lo = int(np.searchsorted(keys, keys[i], side="left"))
            hi = int(np.searchsorted(keys, keys[i], side="right"))
            hits[lo] = prior[lo:hi].tobytes().hex()
        intervals[str(k)] = hits
    assert intervals == {"4743":{138927:"6d6e65"},"4744":{138928:"6e65"},"4822":{}}
    assert gcd(*Counter(raw).values()) == 1
    return intervals


def bzip_crc(data):
    table = []
    for a in range(256):
        reg = a << 24
        for _ in range(8):
            reg = ((reg << 1) ^ (0x04C11DB7 if reg >> 31 else 0)) & 0xffffffff
        table.append(reg)
    reg = 0xffffffff
    for a in data:
        reg = ((reg << 8) & 0xffffffff) ^ table[(reg >> 24) ^ a]
    return reg ^ 0xffffffff


if __name__ == "__main__":
    tape = (E/"frozen.support").read_bytes()
    raw, column, stats = decode_tape(tape)
    assert raw == (ROOT/"baseline-rle1.bin").read_bytes()
    assert column == (ROOT/"column.bin").read_bytes()
    output = rle1(raw)
    assert output == (ROOT/"baseline-output.bin").read_bytes()
    assert bzip_crc(output) == 3567409756
    print(json.dumps(dict(frozen=stats, tape_hash=digest(tape), raw_hash=digest(raw),
                          column_hash=digest(column), output_hash=digest(output),
                          witness=witnesses(column, raw, stats["mark"]),
                          mixed_intervals=certify_threshold(column, raw, stats["mark"]),
                          separator_cases=section_family(), wikidata=csv_summary("wikidata.csv"),
                          prose=csv_summary("prose.csv"),
                          levels=csv_summary("frozen-levels.csv")), indent=2))
