"""Compute per-depth paid floors and paired intervals, controlling for block identity."""
import csv,json,math
from pathlib import Path
import statistics as st

p=Path(__file__).resolve().parent
rows=list(csv.DictReader((p/"prefix.csv").open()))
geo=list(csv.DictReader((p/"geometry.csv").open()))
blocks=sorted({int(x["block"]) for x in rows})
v={(int(x["round"]),int(x["block"]),x["arm"],int(x["depth"])):int(x["thread_ns"]) for x in rows}
assert len(v)==len(rows)==24*(len(geo)+len(blocks))
def ci(aa,bb):
    z=[math.log(a/b) for a,b in zip(aa,bb)]
    m=st.mean(z);h=2.068657610419*st.stdev(z)/math.sqrt(len(z))
    return [round(math.exp(a),6) for a in (m,m-h,m+h)]
out={}
for key,ids in [("all",blocks)]+[(str(i),[i]) for i in blocks]:
    n=min(max(int(x["depth"]) for x in geo if int(x["block"])==i) for i in ids)+1
    scalar=[sum(v[r,i,"scalar",0] for i in ids) for r in range(1,25)]
    prefixes=[[sum(v[r,i,"prefix",d] for i in ids) for r in range(1,25)] for d in range(n)]
    paid=[ci(a,scalar) for a in prefixes]
    out[key]={"first_cut_after_free_setup":ci([b-a for a,b in zip(prefixes[0],prefixes[1])],scalar),
              "paired_ratios":paid,"first_depth_lower_bound_exceeds_scalar":next(d for d,z in enumerate(paid) if z[1]>1),
              "scalar_median_ms":round(st.median(scalar)/1e6,6),"prefix_median_ms":[round(st.median(a)/1e6,6) for a in prefixes]}
assert {a["first_depth_lower_bound_exceeds_scalar"] for a in out.values()}=={1}
print(json.dumps(out,indent=2))
