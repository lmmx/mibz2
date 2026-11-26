import csv
from pathlib import Path
import statistics
import sys

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio

with (here/"phase.csv").open() as source:
    rows=list(csv.DictReader(source))
blocks={r["stream"] for r in rows}
assert len(blocks)==9 and len(rows)==30*3*9
for arm in ("counted","refill","lazy"):
    for phase in ("prep_mark_ns","total_ns"):
        values=[statistics.median(int(r[phase]) for r in rows if r["stream"]==b and r["arm"]==arm) for b in blocks]
        print(arm,phase,"sum of block medians",sum(values))
for arm,phase in (("refill","prep_mark_ns"), ("refill","total_ns"),("lazy","total_ns")):
    v=lambda a:[sum(int(r[phase]) for r in rows if r["arm"]==a and int(r["round"])==i) for i in range(1,31)]
    print("counted /",arm,phase,paired_ratio(v("counted"),v(arm)))
