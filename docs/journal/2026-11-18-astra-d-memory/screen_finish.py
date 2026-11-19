"""Count RLE1 edits and the displacement (output minus consumed encoded prefix)."""

import csv
import re
import sys
from pathlib import Path

# A count byte is consumed with the four equal bytes and cannot start another token.
quad = re.compile(rb"(.)\1{3}", re.DOTALL)


def edits(data):
    """Yield the RLE1 edits found in the data."""
    i = 0
    while match := quad.search(data, i):
        j = match.start()
        if j + 4 >= len(data):
            raise ValueError("truncated RLE1")
        yield j, data[j], data[j + 4]
        i = j + 5


if __name__ == "__main__":
    w = csv.writer(sys.stdout, lineterminator="\n")
    w.writerow(
        (
            "ordinal",
            "rows",
            "output",
            "edits",
            "contract",
            "equal",
            "expand",
            "min_delta",
            "max_delta",
            "literal_bytes",
            "stationary_literals",
            "screen_dir_bytes",
            "packed_dir_bytes",
        )
    )
    for path in sorted(Path(sys.argv[1]).glob("*/inverse")):
        s = path.read_bytes()
        prev = d = lo = hi = literal = fixed = 0
        counts = [0, 0, 0]
        exact = bytearray()
        for j, c, extra in edits(s):
            n = j - prev
            literal += n
            if d == 0:
                fixed += n
            exact += s[prev:j] + bytes([c]) * (4 + extra)
            counts[(extra > 0) + (extra > 1)] += 1
            d += extra - 1
            lo, hi = min(lo, d), max(hi, d)
            prev = j + 5
        n = len(s) - prev
        exact += s[prev:]
        assert exact == (path.parent / "output").read_bytes()
        w.writerow(
            (
                int(path.parent.name),
                len(s),
                len(exact),
                sum(counts),
                *counts,
                lo,
                hi,
                literal + n,
                fixed + n * (d == 0),
                12 * sum(counts),
                8 * sum(counts),
            )
        )
