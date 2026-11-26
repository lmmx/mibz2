"""Compare the immutable scalar and eight-path schedules on the same verified blocks."""
import csv
import statistics
from pathlib import Path

here=Path(__file__).resolve().parent
with (here/"phase-cpu.csv").open() as f:
    records=list(csv.DictReader(f))
blocks=sorted({r["stream"] for r in records})
assert len(records)==9*21*3
for arm in ("batched","two","lazy"):
    print(arm,end="")
    for key in ("prep_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns"):
        summed=sum(statistics.median(int(r[key]) for r in records if r["stream"]==b and r["arm"]==arm) for b in blocks)
        print(f" {key}={summed/1e6:.4f} ms",end="")
    print()
