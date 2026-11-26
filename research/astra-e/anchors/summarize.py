"""Aggregate single-worker structural counters from the canonical Wikidata corpus."""
import csv
import sys

rows = []
with open(sys.argv[1]) as raw:
    for line in csv.reader(raw):
        assert line[0] == "anchors" and len(line) == 9
        rows.append(tuple(map(int, line[1:])))
def report(name, subset):
    n, primary, mark, retained, anchors, tail, marked, scratch = map(sum, zip(*subset))
    print(f"{name}: {len(subset)} blocks")
    print(f"  {retained} original marked residual hops, {marked} compact marked hops")
    print(f"  final/original {marked / retained:.6f}; new discovery reads {anchors + tail}")
    print(f"  {anchors} anchors, {tail} staged literal bytes")
    print(f"  maximum incremental owned capacity {max(r[-1] for r in subset)} bytes")
report("frozen", rows[:1])
report("all OpenGated blocks", rows)
