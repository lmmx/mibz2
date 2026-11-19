"""Audit all four direct arms, the matched materialization control and observed capacities."""
import csv,json,math,statistics as st
from pathlib import Path
base=Path(__file__).resolve().parent
def rows(name):return list(csv.DictReader((base/name).open()))
def intervals(name,arms,contrasts):
    raw=rows(name);n=len(raw)//len(arms);t={18:2.109816,24:2.068658}[n]
    z={(int(q["round"]),q["arm"]):q for q in raw}
    assert len(raw)==len(z)==n*len(arms)
    assert all(int(q['rows'])==209706557 and int(q['front_items'])==16327419 and q["metadata_fold"]=="1af44a82b1684732" for q in raw)
    for a in arms:
        key=[q for q in raw if q["arm"]==a]
        assert len(key)==n and all(sum(int(x["position"])==i for x in key)==n//len(arms) for i in range(len(arms)))
    def ns(i,a):return int(z[i,a]["thread_ns"])
    out={"median_ms":{a:st.median(ns(i,a)for i in range(1,n+1))/1e6 for a in arms}}
    for a,b in contrasts:
        l=[math.log(ns(i,a)/ns(i,b))for i in range(1,n+1)]
        m=st.mean(l);h=t*st.stdev(l)/math.sqrt(n)
        out[a+"/"+b]={"ratio":math.exp(m),"ci95":[math.exp(m-h),math.exp(m+h)],"median_paired_difference_ms":st.median((ns(i,a)-ns(i,b))/1e6 for i in range(1,n+1))}
    return out
p=intervals("panel.csv",["library","copy","fifty","eight_hundred"],[(x,y)for x in ["fifty","eight_hundred"]for y in ["library","copy"]]+[("copy","library")])
s=intervals("split.csv",["library","materialized","eight_hundred"],[("eight_hundred","materialized"),("materialized","library"),("eight_hundred","library")])
m={}
for code,label in [("0","library"),("2","fifty"),("3","eight_hundred"),("4","materialized")]:
    data=[q for q in csv.reader((base/"memory.csv").open()) if q[0]==code];assert len(data)==431
    cols=["symbol_vector","private_run_front","stock_run_front","column"]
    m[label]={c:max(int(q[i+3])for q in data)for i,c in enumerate(cols)}
    m[label]["max_sum_bytes"]=max(sum(int(v)for v in q[3:])for q in data)
print(json.dumps({"native":p,"split":s,"retained_vector_capacity":m},indent=2))
