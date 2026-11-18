#!/usr/bin/env python3
"""Offline census of the shipped steps. verify.py chooses its steps without this oracle."""
import argparse
import csv
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "bench"))
import irec

PAIR_NAMES = [a + b for a in "ETB" for b in "ETB"]


def census(rec):
    arrays = rec.a
    left, right = arrays["s_a"], arrays["s_b"]
    kind, fresh = arrays["s_type"], arrays["s_fresh"]
    words, lengths = arrays["w_kind"], arrays["w_len"]
    byte = arrays["l_byte"]
    counts = Counter()
    # A closure breaks an adjacent pair, even when both steps exist in the log.
    breaks = set(arrays["c_step"])
    for i in range(rec.steps - 1):
        if i + 1 in breaks:
            continue
        p, q = kind[i], kind[i + 1]
        counts[PAIR_NAMES[3 * p + q]] += 1
        x = rec.r + i
        if p != q and x in (left[i + 1], right[i + 1]):
            counts["mixed_child"] += 1
            counts["single_child"] += arrays["w_uses"][x] == 1
        if p != irec.TOP_WINS or not fresh[i + 1] & irec.FRESH_B:
            continue
        counts["fresh"] += 1
        if q == irec.TOP_WINS:
            continue
        counts["eligible"] += 1
        counts["fresh_TB"] += q == irec.BOTTOM_WINS
        b, d = right[i], right[i + 1]
        # D is a new byte. A common operand handle must be an equal literal
        # with the same mark. Distinct original letters cannot both be marked.
        assert d < rec.r and b != d
        if b >= rec.r or byte[b] != byte[d] or words[b] != words[d]:
            counts["different"] += 1
        elif words[x] == irec.ARENA:
            counts["short_TE" if q == irec.EQUAL else "short_TB"] += 1
            counts["bytes_saved"] += lengths[x]
        else:
            counts["equal_rule"] += 1
    base = dict(corpus=rec.corpus, block=rec.block, index=rec.index, n=rec.n, r=rec.r,
                steps=rec.steps, admitted=int(rec.admitted))
    fields = PAIR_NAMES + ["mixed_child", "single_child", "fresh", "eligible", "fresh_TB",
                           "different", "equal_rule", "short_TE", "short_TB", "bytes_saved"]
    return {**base, **{k: counts[k] for k in fields}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="+", help="irec file or directory")
    args = parser.parse_args()
    writer = None
    for group in args.records:
        for path in irec.paths(group):
            row = census(irec.load(path))
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=list(row), lineterminator="\n")
                writer.writeheader()
            writer.writerow(row)


if __name__ == "__main__":
    main()
