"""Check every entrance in every published frozen word by an independent dense cycle."""
import argparse
import csv
import hashlib
from bisect import bisect_left, bisect_right
from pathlib import Path

from shortcut import read_records
from whole import reconstruct


def audit(columns, receipt):
    column = (columns / "0000.L").read_bytes()
    n, primary = len(column), int(receipt["primary"])
    psi = sorted(range(n), key=column.__getitem__)
    first = bytes(column[x] for x in psi)
    boundaries = [x for x in range(1, n) if first[x] != first[x-1] or psi[x] != psi[x-1]+1]
    row_at, place, text = [0]*n, [-1]*n, bytearray(n)
    row = primary
    for pos in range(n):
        assert place[row] == -1
        row_at[pos], place[row], text[pos] = row, pos, first[row]
        row = psi[row]
    assert row == primary and hashlib.sha256(text).hexdigest() == receipt["pre_sha256"]
    cyclic = bytes(text+text)
    entrances = spelled_bytes = 0

    def check(node, pos, install):
        nonlocal entrances, spelled_bytes
        low, high, shift, length, source = node
        word = cyclic[pos:pos+length]
        if length == 1:
            assert source == word[0]
        else:
            assert cyclic.startswith(word, source)
        for x in range(low, high):
            start = place[x]
            assert cyclic.startswith(word, start)
            assert row_at[(start+length) % n] == x+shift
        if not install:
            assert bisect_right(boundaries, low+shift) < bisect_left(boundaries, high+shift)
        entrances += high-low
        spelled_bytes += length*(high-low)

    out, stats = reconstruct(read_records(columns / "0000.runs"), n, primary, publish=check)
    assert out == text
    assert all(stats[k] == int(receipt[k]) for k in ("phrases", "installs", "retired"))
    return entrances, spelled_bytes


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("columns", type=Path)
    args = parser.parse_args()
    with (Path(__file__).parent / "evidence/frozen-whole.csv").open() as f:
        print(*audit(args.columns, next(csv.DictReader(f))))
