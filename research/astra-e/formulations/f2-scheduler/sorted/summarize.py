import csv
from pathlib import Path
from statistics import median
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[4]/"bench/moonshot"));from moonshot import paired_ratio
r=list(csv.DictReader((here/"phase.csv").open()));assert len(r)==9*3*30
blocks={x["stream"] for x in r}
for arm in ("sorted","refill","lazy"):
 print(arm,{k:round(sum(median(int(x[k]) for x in r if x["arm"]==arm and x["stream"]==b) for b in blocks)/1e6,4) for k in r[0] if k.endswith("_ns")})
for ref,key in (("refill","discover_ns"),("refill","total_ns"),("lazy","total_ns")):
 vals=lambda a:[sum(int(x[key]) for x in r if x["arm"]==a and int(x["round"])==j) for j in range(1,31)]
 z=paired_ratio(vals("sorted"),vals(ref))
 print(f"sorted/{ref} {key} {z['geomean']:.4f} [{z['lo']:.4f},{z['hi']:.4f}]")
