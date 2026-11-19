"""Price nested selection, the direct ranked construction, and its output separately."""
import csv
from pathlib import Path
from statistics import median
import sys

d = Path(__file__).resolve().parent
sys.path.insert(0, str(d.parents[3] / "bench/moonshot"))
from moonshot import paired_ratio


def rows(p):
    with p.open() as handle:
        return list(csv.DictReader(handle))


sample = rows(d / "phase.csv")
index = {(r["round"], r["stream"], r["arm"]): r for r in sample}
rounds = sorted({key[0] for key in index})
blocks = sorted({key[1] for key in index})
assert len(sample) == len(index) == 9 * 30 * 3
phases = ("prepare_ns", "select_mark_ns", "discover_ns", "merge_ns", "residual_output_ns", "total_ns")
for arm in ("chosen", "fixed", "lazy"):
    sums = [sum(median(int(index[r,b,arm][k]) for r in rounds) for b in blocks) for k in phases]
    print(arm, dict(zip(phases, (round(v/1e6,4) for v in sums))))
for field, control in [(s,"fixed") for s in phases[2:]] + [("total_ns","lazy")]:
    totals = {a: [sum(int(index[r,b,a][field]) for b in blocks) for r in rounds]
              for a in ("chosen",control)}
    print(f"chosen/{control} {field}", paired_ratio(totals["chosen"], totals[control]))
deep = [r for r in rows(d / "native-geometry.csv") if r["took"] == "1"]
fixed = [r for r in rows(d.parent / "f2-word" / "geometry.csv") if r["took"] == "1"]
assert [r["stream"] for r in fixed] == [r["stream"] for r in deep] and len(deep) == 228
for field in ["token_rows","words","dict_bytes","lookups","root_runs","token_visits"]:
    print(field, "chosen", sum(int(r[field]) for r in deep), "fixed", sum(int(r[field]) for r in fixed))
