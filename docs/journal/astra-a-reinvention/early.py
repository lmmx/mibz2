"""Three frozen screens. Dense arrays are counting oracles, never free inputs to a decoder."""
import argparse
import csv
import hashlib
import sys
from itertools import groupby
from pathlib import Path

import numpy as np


def lf_data(column):
    counts = np.bincount(column, minlength=256)
    alphabet = np.flatnonzero(counts)
    first = np.repeat(alphabet.astype(np.uint8), counts[alphabet])
    ends = counts.cumsum()
    lf = np.empty(len(column), np.int32)
    for c in alphabet:
        rows = np.flatnonzero(column == c)
        lf[rows] = np.arange(ends[c] - counts[c], ends[c])
    return first, lf, counts


def census(column, primary):
    n = len(column)
    first, lf, counts = lf_data(column)
    alphabet = np.flatnonzero(counts)
    matrix = np.bincount(256 * column.astype(np.int32) + first,
                         minlength=65536).reshape(256, 256)
    singleton = (matrix > 0).sum(axis=0) == 1
    # Closest-to-half nontrivial prefix of the ordered byte alphabet. Lower cut wins ties.
    threshold = alphabet[:-1][np.abs(2 * counts[alphabet].cumsum()[:-1] - n).argmin()]
    low = column <= threshold
    keep = ~(low & (first > threshold))
    before = column[lf]
    keys = np.where(~low & (before <= threshold), (before.astype(int) + 1) * 256 + column,
                    column.astype(int))[keep]
    fragments = np.r_[True, column[1:] != column[:-1]]
    physical = np.diff(np.r_[np.flatnonzero(fragments), n])
    orbit = np.empty(n, np.int32)
    row = primary
    for j in range(n):
        orbit[j] = row
        row = lf[row]
    assert row == primary and np.unique(orbit).size == n  # real sample, one cycle
    spelling = column[orbit].tobytes()
    # Balanced chunks from the primary in LF output order. Distinct maximal affine
    # intervals at depth k can be counted by composing two depth-k/2 intervals.
    jump = lf
    levels = []
    k = 2
    while k <= n:
        fragments = fragments | fragments[jump]
        jump = jump[jump]
        group = np.cumsum(fragments)
        q = n // k
        spellings = len({spelling[j:j+k] for j in range(0, q * k, k)})
        occupied = np.unique(group[orbit[:q*k:k]]).size
        levels.append((k, q, int(fragments.sum()), spellings, occupied))
        k *= 2
    return dict(n=n, primary=primary, sha256=hashlib.sha256(column).hexdigest(),
                runs=len(physical), capped=int(((physical + 126) // 127).sum()),
                alphabet=len(alphabet), bigrams=int((matrix > 0).sum()),
                forced=int(matrix[:, singleton].sum()), modal_exceptions=int(n-matrix.max(axis=0).sum()),
                pair_cut=int(threshold), pair_removed=int(n-len(keys)),
                pair_runs=int(1+np.count_nonzero(keys[1:] != keys[:-1])),
                pair_alphabet=int(np.unique(keys).size),
                balanced_types=sum(r[3] for r in levels),
                balanced_certificates=sum(r[4] for r in levels)), levels


def counterexample():
    a, b = b"001011", b"001101"
    def signature(s):
        rots = sorted(s[i:]+s[:i] for i in range(len(s)))
        edges = sorted(zip(s, s[1:] + s[:1]))
        return rots.index(s), edges
    assert a != b and signature(a) == signature(b)
    assert b not in a + a


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("columns", type=Path)
    ap.add_argument("manifest", type=Path, help="CSV with block and primary")
    ap.add_argument("summary", type=Path)
    ap.add_argument("levels", type=Path)
    args = ap.parse_args()
    counterexample()
    with args.summary.open("w") as out, args.levels.open("w") as levels_out:
        levels_writer = csv.writer(levels_out, lineterminator="\n")
        levels_writer.writerow(("block", "width", "chunks", "domain_intervals", "byte_words", "occupied"))
        writer = None
        for record in csv.DictReader(args.manifest.open()):
            block = int(record["block"])
            column = np.fromfile(args.columns / f"{block:04}.L", dtype=np.uint8)
            result, levels = census(column, int(record["primary"]))
            result = dict(block=block, **result)
            if writer is None:
                writer = csv.DictWriter(out, result.keys(), lineterminator="\n")
                writer.writeheader()
            writer.writerow(result)
            levels_writer.writerows((block, *row) for row in levels)
