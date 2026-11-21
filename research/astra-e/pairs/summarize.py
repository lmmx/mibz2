"""Reduce the exact, single-worker structural receipt (no timings)."""
import csv
import sys

rows = []
with open(sys.argv[1]) as raw:
    for line in csv.reader(raw):
        assert line[0] == "pairs" and len(line) == 12
        rows.append(tuple(map(int, line[1:])))
fields = "n primary mark bits retained words fits lookups rewritten original_hops final_hops".split()
def report(name, subset):
    totals = dict(zip(fields, map(sum, zip(*subset))))
    print(f"{name}: {len(subset)} blocks, {totals['fits']} eligible")
    for key in ("n", "retained", "lookups", "rewritten", "original_hops", "final_hops"):
        print(f"  {key} {totals[key]}")
    print(f"  final/original hops {totals['final_hops'] / totals['original_hops']:.6f}")
report("frozen", rows[:1])
report("all OpenGated blocks", rows)
report("with bit capacity", [r for r in rows if r[6]])
