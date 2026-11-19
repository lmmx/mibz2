"""Count direct construction of each original domain's longest common atom itinerary.

Dense arrays supply a favourable counting oracle. Production inputs do not supply them.
Each output span used for the greedy parse is checked against pinned Plain.
"""
import argparse
import csv
import sys
from pathlib import Path

import numpy as np

from early import lf_data


def reverse_gaps(column, lf, starts, words):
    """Compute all first boundary times together, then verify each wide root's minimum."""
    n = len(column)
    distance = np.full(n, -1, dtype=np.int32)
    distance[starts[1:-1]] = 0
    advances = 0
    for boundary in starts[1:-1]:
        gap, depth = int(boundary), 0
        while column[gap-1] == column[gap]:
            gap = int(lf[gap])
            depth += 1
            assert gap > 0 and distance[gap] == -1
            distance[gap] = depth
            advances += 1
    # A component with no boundary is a cycle. After n letters the block is complete.
    distance[distance < 0] = n
    for low, high, word in zip(starts[:-1], starts[1:], words):
        if high-low > 1:
            assert int(distance[low+1:high].min()) == len(word)
    return advances


def census(column, primary, expected):
    n = len(column)
    first, lf, _ = lf_data(column)
    psi = np.empty(n, dtype=np.int32)
    psi[lf] = np.arange(n)
    starts = np.r_[0, np.flatnonzero((np.diff(psi) != 1) | (first[1:] != first[:-1])) + 1, n]
    widths = np.diff(starts)
    atom = np.repeat(np.arange(len(widths)), widths)
    words, shifts = [], []
    eager_steps = 0
    for low, width in zip(starts[:-1], widths):
        low, width = int(low), int(width)
        row = low
        if width == 1:
            word = bytes([first[row]])
            row = int(psi[row])
        else:
            letters = bytearray()
            while len(letters) < n and atom[row] == atom[row + width - 1]:
                letters.append(first[row])
                row = int(psi[row])
            assert letters
            word = bytes(letters)
            eager_steps += len(word)
        words.append(word)
        shifts.append(row-low)

    used = set()
    row, pos, visits, one_byte, first_extra, wide_used = primary, 0, 0, 0, 0, 0
    while pos < n:
        root = int(atom[row])
        word = words[root]
        take = min(len(word), n-pos)
        assert expected[pos:pos+take] == word[:take]
        visits += 1
        one_byte += take == 1
        if widths[root] > 1 and root not in used:
            used.add(root)
            wide_used += 1
            first_extra += take-1
        pos += take
        row += shifts[root]

    gap_steps = reverse_gaps(column, lf, starts, words)
    return dict(wide=int((widths > 1).sum()), max_word=max(map(len, words)),
                eager_steps=eager_steps, reverse_gap_steps=gap_steps,
                greedy_visits=visits, one_byte_visits=one_byte,
                wide_first_uses=wide_used, first_extra_steps=first_extra,
                direct_on_demand_visits=visits+first_extra)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("columns", type=Path)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("plain", type=Path)
    args = parser.parse_args()
    writer = None
    with args.manifest.open() as manifest:
        for record in csv.DictReader(manifest):
            block = int(record["block"])
            column = np.fromfile(args.columns / f"{block:04}.L", dtype=np.uint8)
            expected = (args.plain / f"{block:04}.T").read_bytes()
            row = dict(block=block, **census(column, int(record["primary"]), expected))
            if writer is None:
                writer = csv.DictWriter(sys.stdout, row.keys(), lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)
