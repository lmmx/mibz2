"""Frequency and spelling overlap. Input is the independent Plain inversion, for census only."""

from bisect import bisect_left
from collections import Counter
import csv
import struct
import sys


def lcp(a, b):
    """Return the common prefix length of two byte strings."""
    return next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), min(len(a), len(b)))


def measure(i, model, known, c):
    """Count shared words, occurrences, spelling bytes and reversed-trie prefixes."""
    refs = sorted(t[::-1] for t in known)
    matched = total_suffix = 0
    previous = b""
    for v in sorted(t[::-1] for t in c):
        k = bisect_left(refs, v)
        best = max((lcp(v, refs[j]) for j in (k - 1, k) if 0 <= j < len(refs)), default=0)
        cut = lcp(v, previous)
        matched += max(0, best - cut)
        total_suffix += len(v) - cut
        previous = v
    common = c.keys() & known
    return [
        i,
        model,
        len(c),
        len(common),
        sum(c.values()),
        sum(c[t] for t in common),
        sum((len(t) + 1) * n for t, n in c.items()),
        sum((len(t) + 1) * c[t] for t in common),
        sum(1 + len(t) for t in c),
        sum(1 + len(t) for t in common),
        total_suffix + 1,
        matched + int(bool(known)),
    ]


def main():
    """Compare each large block with its predecessor and with a fixed training window."""
    out = csv.writer(sys.stdout)
    out.writerow(
        "block model types shared_types tokens shared_tokens bytes shared_bytes dict_bytes shared_dict_bytes suffixes shared_suffixes".split()
    )
    training = set()
    previous = None
    with open(sys.argv[1], "rb") as f:
        while header := f.read(8):
            i, n = struct.unpack("<II", header)
            s = f.read(n)
            parts = s.split(b'"')
            assert len(parts) > 1
            c = Counter(parts[1:-1])
            c[parts[-1] + parts[0]] += 1
            if previous is not None:
                out.writerow(measure(i, "previous", previous, c))
            # 220 blocks in the first six original Wikimedia byte windows.
            if i < 220:
                training.update(c)
            else:
                out.writerow(measure(i, "windows_0_5", training, c))
            previous = set(c)


if __name__ == "__main__":
    main()
