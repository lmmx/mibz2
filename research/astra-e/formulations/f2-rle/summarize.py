"""Each paired observation sums the same fixed nine systematic admission blocks."""
import csv,math,statistics,sys
from pathlib import Path
p=Path(sys.argv[1])
with (p/"phases.csv").open() as f:r=list(csv.DictReader(f))
assert len(r)==9*20*2
for k in ["wall_ns","cpu_ns","hazards_ns","construction_walk_ns","unrle_ns","crc_ns"]:
    s={a:[sum(int(z[k]) for z in r if z["arm"]==a and int(z["round"])==i) for i in range(1,21)] for a in ["sparse","opend"]}
    print(k,*(f"{a} {statistics.median(v)/1e6:.4f} ms" for a,v in s.items()))
    if k in ("wall_ns","cpu_ns"):
        logs=[math.log(i/j) for i,j in zip(s["sparse"],s["opend"])]
        mid=statistics.mean(logs);rad=2.093024*statistics.stdev(logs)/math.sqrt(20)
        print(f"paired {math.exp(mid):.5f} [{math.exp(mid-rad):.5f},{math.exp(mid+rad):.5f}]")
g=list(csv.DictReader((p/"native.csv").open()))
done=[row for row in g if row["took"]=="true"]
print("eligible",len(g),"actual",len(done),"verified coded bytes",sum(int(z["rows"]) for z in done))
print("index capacity frozen",done[0]["index_capacity"],"high water",max(int(z["index_capacity"]) for z in done))
