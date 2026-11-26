"""Each pair comprises the summed observations on the same nine preselected blocks."""
import csv
import math
from pathlib import Path
from statistics import mean, stdev

here=Path(__file__).resolve().parent
rows=list(csv.DictReader((here / "phase.csv").open()))
assert len(rows)==9*3*21
for field in ("wall_ns","cpu_ns"):
    total={(arm,r):sum(int(row[field]) for row in rows if row["arm"]==arm and int(row["round"])==r)
           for arm in ("opend","lean","direct") for r in range(1,22)}
    for a,b in [("lean","opend"),("lean","direct"),("direct","opend")]:
        logs=[math.log(total[a,r]/total[b,r]) for r in range(1,22)]
        x=mean(logs)
        half=2.085963*stdev(logs)/math.sqrt(len(logs))
        print(f"{field} {a}/{b} {math.exp(x):.5f} [{math.exp(x-half):.5f}, {math.exp(x+half):.5f}]")

hints=list(csv.DictReader((here / "hints.csv").open()))
assert len(hints)==228
print("maximum retained lean hint bytes",max(int(x["lean_bytes"]) for x in hints))
