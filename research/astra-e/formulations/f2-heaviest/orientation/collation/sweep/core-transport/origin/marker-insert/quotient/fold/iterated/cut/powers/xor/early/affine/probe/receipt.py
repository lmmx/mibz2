"""Summarize one independently regenerated index manifest and both original-input series."""
import csv,hashlib,json,math,statistics as st,sys
from pathlib import Path

p=Path(sys.argv[1]);files={}
for f in sorted(p.glob("*.idx")):
    blob=f.read_bytes();files[f.stem]={"bytes":len(blob),"sha256":hashlib.sha256(blob).hexdigest()}
assert len(files)==10
def read(name):
    samples=list(csv.DictReader((p/name).open()))
    t={(int(x["round"]),int(x["block"]),x["arm"]):int(x["thread_ns"]) for x in samples}
    assert len(t)==len(samples)==480
    ids=sorted(map(int,files));report={}
    for key,blocks in [("all",ids)]+[(str(i),[i]) for i in ids]:
        def series(arm):return [sum(t[r,b,arm]for b in blocks) for r in range(1,25)]
        a,b=series("implicit"),series("scalar")
        z=[math.log(i/j)for i,j in zip(a,b)];m=st.mean(z);d=2.068657610419*st.stdev(z)/math.sqrt(24)
        report[key]={"median_ms":[round(st.median(x)/1e6,6) for x in (a,b)],
                     "paired_ratio":[round(math.exp(v),6)for v in (m,m-d,m+d)]}
    return report
out={"fixtures":files,"rotating":read("times.csv")}
if (p/"warm.csv").is_file() and sum(1 for _ in (p/"warm.csv").open())==481:out["warm"]=read("warm.csv")
print(json.dumps(out,indent=2))
