#!/usr/bin/env python3
"""Cross-check the fixed sample against Study 5 and print the declared cost sensitivity."""
import argparse
import csv
import sys
from collections import Counter
from pathlib import Path

from schedules import irec

HERE = Path(__file__).resolve().parent


def read(path):
    with path.open() as f:
        return list(csv.DictReader(f))


def ledger(row):
    r, k, h = row["rules"], row["copies"], row["stack_peak"]
    pushes, wraps = row["pushes"], row["last_wraps"]
    edges = r - 1 + k
    emits = row["literals"] + k
    assert emits == r + 1
    # A cold doubling descriptor stack initially has no slots. Each already full
    # capacity (1, 2, 4, ...) is read and written once when the next is allocated.
    cap = 1 << (h - 1).bit_length() if h else 0
    growth = 2 * max(cap - 1, 0)
    # Full formulas and what an addressed field, branch or move means are in README.
    last_control = 7 * r + edges + wraps + 3 * k
    visit_control = (2 * r + 1) + 2 * (r + k) + (2 * emits + 1) + (4 * pushes + 1) + r
    return dict(growth_fields=growth, allocations=cap.bit_length(), last_control=last_control,
                visit_control=visit_control)


def grammar_census(path):
    """One scan of the recorded kinds and operand identifiers. No ownership search."""
    out = None
    for name in irec.paths(path):
        rec = irec.load(name)
        assert rec.n == rec.period
        kinds = rec.a["w_kind"]
        is_rule = bytes((kind & 3) == irec.RULE for kind in kinds)
        r = sum(is_rule)
        left = sum(map(is_rule.__getitem__, rec.a["s_a"]))
        right = sum(map(is_rule.__getitem__, rec.a["s_b"]))
        k = left + right - r + 1
        assert r > 0
        row = dict(block=rec.block, rows=rec.n, rules=r, copies=k, left_long=left,
                   last_fields=4 * r + 5 * k, text_fields=3 * r + k + 2 * left)
        if out is None:
            out = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
            out.writeheader()
        out.writerow(row)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--grammar-census", metavar="IREC_DIR")
    args = parser.parse_args()
    if args.grammar_census:
        grammar_census(args.grammar_census)
        return
    old = {int(r["block"]): r for r in read(HERE.parent / "2026-11-18-cells-live-reuse/schedules.csv")
           if r["corpus"] == "wikidata.bz2"}
    policies = read(HERE / "wikidata.csv")
    base = {int(r["block"]): r for r in policies if r["policy"] == "last"}
    walks = [{k: int(x) for k, x in r.items()} for r in read(HERE / "first-visit.csv")]
    assert [r["block"] for r in walks] == list(range(0, 233, 24))
    census = {int(a["block"]): {k: int(v) for k, v in a.items()} for a in read(HERE / "counts.csv")}
    assert list(census) == list(range(233))
    for idx, c in census.items():
        assert c["copies"] == int(old[idx]["copies"])
        assert c["rules"] + 1 == c["copies"] + int(old[idx]["literal_edges"])
    lf, tf = (sum(c[k] for c in census.values()) for k in ("last_fields", "text_fields"))
    print("233 grammars", lf, "->", tf, "positive blocks",
          sum(c["last_fields"] > c["text_fields"] for c in census.values()))
    for a in walks:
        block = a["block"]
        assert (a["fields"], a["last_fields"], a["pushes"]) == tuple(
            census[block][k] for k in ("text_fields", "last_fields", "left_long"))
        assert (a["copies"], a["copy_bytes"], a["literals"]) == tuple(
            int(old[block][k]) for k in ("copies", "copy_bytes", "literal_edges"))
        assert a["last_output_misses"] == int(base[block]["output_misses"])
        assert a["last_wraps"] == int(old[block]["wraps"])
        a.update(ledger(a))
    for name, group in (("frozen", walks[:1]), ("ten blocks", walks)):
        z = Counter()
        for row in group:
            z.update(row)
        print(name)
        for k in ("rules", "copies", "copy_bytes", "literals", "pushes", "allocations", "growth_fields",
                  "last_later_in_word", "source_runs", "last_fields", "fields", "last_control", "visit_control",
                  "last_rule_misses", "rule_misses", "last_output_misses", "text_birth_output_misses",
                  "output_misses", "last_combined_misses", "combined_misses", "last_combined_rule", "combined_rule",
                  "last_combined_output", "combined_output", "last_combined_arena", "combined_arena",
                  "last_combined_scratch", "combined_scratch", "last_writebacks", "writebacks"):
            print(f"  {k} {z[k]:,}")
        print("  maximum pending descriptors", max(r["stack_peak"] for r in group))
        for field_price in (1, 2):
            old_price = field_price * z["last_fields"] + z["last_control"]
            new_price = field_price * (z["fields"] + z["growth_fields"]) + z["visit_control"] + z["allocations"]
            print(f"  field price {field_price}: {old_price:,} -> {new_price:,} "
                  f"({100 * (1 - new_price / old_price):.2f}% lower)")
        saved = z["last_combined_misses"] - z["combined_misses"]
        dirty = z["last_writebacks"] - z["writebacks"]
        extra = z["combined_rule"] - z["last_combined_rule"]
        all_extra = extra + z["combined_arena"] - z["last_combined_arena"]
        print(f"  rule fill premium break-even {saved / extra:.3f}, including writebacks {(saved + dirty) / extra:.3f}")
        print(f"  rule and arena premium {saved / all_extra:.3f}, including writebacks {(saved + dirty) / all_extra:.3f}")
    for k in ("rounds", "span_bytes", "distance", "same_line", "same_page", "line_bound", "page_bound",
              "aligned", "adjacent", "output_misses", "wave_misses", "wave_adjacent"):
        print(k, *(sum(int(r[k]) for r in policies if r["policy"] == name)
                   for name in ("last", "first", "rounds", "bytes", "median")))


if __name__ == "__main__":
    main()
