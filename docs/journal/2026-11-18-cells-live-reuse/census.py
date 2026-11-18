#!/usr/bin/env python3
"""Retrospective classifications; the ownership schedule does not have these dictionaries."""
import argparse
import csv
import itertools
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "bench"))
import irec


def census(rec, examples=None):
    v, r = rec.a, rec.r
    parent_a, parent_b, types = v["s_a"], v["s_b"], v["s_type"]
    kind, length, byte = v["w_kind"], v["w_len"], v["l_byte"]
    words = [bytes([x]) for x in byte] + [None] * rec.steps
    trees = list(byte) + [None] * rec.steps
    tree_ids, seen_words, splits = {}, set(), set()
    handle_pairs, left_tenure, right_tenure = set(), set(), set()
    c = Counter()
    previous_handles = None
    previous_a = previous_b = previous_type = None
    breaks = set(v["c_step"])
    # Marked originals cannot be children of an unmarked short word.
    def handle(x):
        return byte[x] if x < r else 256 + x

    for i, (a, b, t) in enumerate(zip(parent_a, parent_b, types)):
        x = r + i
        if i in breaks:
            previous_handles = previous_a = previous_b = previous_type = None
        if t == previous_type and t in (irec.TOP_WINS, irec.BOTTOM_WINS):
            c["TT" if t == irec.TOP_WINS else "BB"] += 1
        if t == irec.TOP_WINS:
            c["T"] += 1
            c["old_literal_T"] += b < r and b != rec.row_letter
        if t == irec.BOTTOM_WINS:
            c["B"] += 1
        fresh_bottom = previous_type == irec.TOP_WINS and v["s_fresh"][i] & irec.FRESH_B
        if fresh_bottom:
            c["lookahead_F"] += 1
            c["lookahead_TT"] += t == irec.TOP_WINS
            c["lookahead_TB"] += t == irec.BOTTOM_WINS
        if kind[x] == irec.ARENA:
            ha, hb = handle(a), handle(b)
            pair = ha, hb
            shape = trees[a], trees[b]
            data = words[a] + words[b]
            split = data, len(words[a])
            if data in seen_words:
                if pair == previous_handles and (a == previous_a or b == previous_b):
                    label = "live_adjacent"
                    if fresh_bottom and t != irec.TOP_WINS:
                        c["study575"] += 1
                    if fresh_bottom and t == irec.TOP_WINS:
                        c["live_fresh_TT"] += 1
                elif (a, hb) in left_tenure or (ha, b) in right_tenure:
                    label = "older_same_front"
                elif pair in handle_pairs:
                    assert a < r and b < r, "a surviving nonliteral has only one front tenure"
                    label = "other_literal_pair"
                elif shape in tree_ids:
                    label = "same_tree_new_handles"
                elif split in splits:
                    label = "different_tree_same_split"
                else:
                    label = "different_root_split"
                c[label] += 1
                if examples is not None:
                    examples.setdefault(label, (i, a, b, data.hex()))
            c["short"] += 1
            c["digram"] += len(data) == 2
            seen_words.add(data)
            splits.add(split)
            trees[x] = tree_ids.setdefault(shape, len(tree_ids) + 256)
            words[x] = data
            handle_pairs.add(pair)
            left_tenure.add((a, hb))
            right_tenure.add((ha, b))
            previous_handles = pair
        else:
            previous_handles = None
            c["rules"] += 1
            c["marked_rules"] += bool(kind[x] & irec.MARKED)
            for child, survives in ((a, t == irec.TOP_WINS), (b, t == irec.BOTTOM_WINS)):
                if kind[child] == irec.RULE:
                    c["long_edges"] += 1
                    if survives:
                        c["copies"] += 1
                        c["copy_bytes"] += length[child]
                if kind[child] & 3 != irec.RULE:
                    c["literal_edges"] += 1
                    c["literal_bytes"] += length[child]
        previous_a, previous_b, previous_type = a, b, t
    fields = ("short digram live_adjacent older_same_front other_literal_pair same_tree_new_handles "
              "different_tree_same_split different_root_split study575 T B old_literal_T TT BB lookahead_F lookahead_TT "
              "lookahead_TB live_fresh_TT rules marked_rules long_edges copies copy_bytes literal_edges literal_bytes").split()
    # Undo the primary split before counting the capped run records in src/runs.rs.
    groups = itertools.groupby(zip(byte, v["l_lam"]), key=lambda item: item[0])
    records = sum((sum(w for _, w in group) + 126) // 127 for _, group in groups)
    eager = 10 * records < rec.n <= 64 * records and rec.steps <= 4 * rec.n // 25
    return dict(corpus=rec.corpus, block=rec.block, n=rec.n, period=rec.period, r=r,
                steps=rec.steps, records=records, eager=int(eager), lazy=int(rec.admitted), **{f: c[f] for f in fields})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("records", nargs="+")
    parser.add_argument("--examples", action="store_true", help="first witness of each class on stderr")
    args = parser.parse_args()
    writer = None
    for path in (p for group in args.records for p in irec.paths(group)):
        examples = {} if args.examples else None
        row = census(irec.load(path), examples)
        if examples is not None:
            print(row["corpus"], row["block"], examples, file=sys.stderr)
        if writer is None:
            writer = csv.DictWriter(sys.stdout, fieldnames=row.keys(), lineterminator="\n")
            writer.writeheader()
        writer.writerow(row)


if __name__ == "__main__":
    main()
