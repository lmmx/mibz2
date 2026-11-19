"""Compare phase CPU work using a median per block and the sum over equal-weight blocks."""
import csv, statistics, sys
from collections import defaultdict
from pathlib import Path

p=Path(sys.argv[1])
rows=list(csv.DictReader(p.open()))
phases=("prep_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns")
by=defaultdict(list)
for row in rows:
    by[row["stream"],row["arm"]].append(row)
med={tag:[statistics.median(int(x[k]) for x in group) for k in phases] for tag,group in by.items()}
out=csv.writer(sys.stdout)
out.writerow(("sample","arm",*phases,"common_over_lazy","total_over_lazy"))
for label in ("1","all-nine"):
    totals={}
    for arm in ("one","two","lazy"):
        matches=[v for (stream,a),v in med.items() if a==arm and (stream==label or label=="all-nine")]
        totals[arm]=[sum(x) for x in zip(*matches)]
    lazy=totals["lazy"][-1]
    for arm,values in totals.items():
        out.writerow((label,arm,*values,round(sum(values[:3])/lazy,6),round(values[-1]/lazy,6)))
