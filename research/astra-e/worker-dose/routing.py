"""Reassess the same directly paired inverse stages used by edge on its 228 original admissions."""
from collections import defaultdict
from pathlib import Path
import csv
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"bench/moonshot"))
from moonshot import paired_ratio

data=defaultdict(dict)
for r in csv.DictReader((HERE.parent/"narrow-short/stage/laps.csv").open()):
    if r["arm"] in ("eager","ahead"):
        data[int(r["ordinal"]),int(r["round"])][r["arm"]]=int(r["thread_cpu_ns"])
assert len(data)==231
out=csv.DictWriter(sys.stdout,fieldnames=["ordinal","rounds","eager_over_edge_inverse_cpu","lo","hi"],lineterminator="\n")
out.writeheader()
for block in sorted(set(b for b,_ in data)):
    e,v=([data[block,i][arm] for i in range(21)] for arm in ("eager","ahead"))
    cell=paired_ratio(e,v)
    out.writerow(dict(ordinal=block,rounds=21,eager_over_edge_inverse_cpu=cell["geomean"],lo=cell["lo"],hi=cell["hi"]))
