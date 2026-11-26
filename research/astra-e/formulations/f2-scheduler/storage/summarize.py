from pathlib import Path
import csv
here=Path(__file__).resolve().parent
with (here/"observed.csv").open() as data: rows=list(csv.DictReader(data))
assert len(rows)==4*233
for name in ("scalar","fixed","refill","double"):
    selected=[r for r in rows if r["arm"]==name]
    assert sum(int(r["admitted"]) for r in selected)==228
    for tag,sl in [("frozen",selected[:1]),("full stream high water",selected)]:
        print(name,tag,{key:max(int(r[key]) for r in sl) for key in
                ("first_vectors","first_lane_paths","second_vectors","second_lane_paths","total_capacity_bytes")})
