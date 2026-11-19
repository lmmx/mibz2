from pathlib import Path
from statistics import median
import csv
d=Path(__file__).resolve().parent
r=list(csv.DictReader((d/"attribution.csv").open()));assert len(r)==432
for a in ("0","1"):
    z=[v for v in r if v["arm"]==a]
    print("arm",a,"medians of nine-block CPU sums in microseconds")
    for key in ("complete_cpu","nav","hazards","reorder","scatter","geometry","copy","rle"):
        xs=[sum(int(v[key])for v in z if int(v["round"])==i)/1000 for i in range(1,25)]
        print(key,round(median(xs),2))
