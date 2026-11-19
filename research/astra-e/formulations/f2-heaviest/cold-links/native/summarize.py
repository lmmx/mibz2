"""Paired sum-of-block CPU comparisons and all live first-stage capacities."""
import csv
from collections import defaultdict
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[5]/"bench/moonshot"))
from moonshot import paired_ratio

def rows(p):
    with p.open() as f:return list(csv.DictReader(f))
def pair(a,b):
    r=paired_ratio(a,b)
    return " ".join(f"{r[k]:.4f}" for k in ("geomean","lo","hi"))

def panel(p):
    data=rows(p);idx={(int(r["round"]),int(r["stream"]),r["arm"]):r for r in data}
    blocks=sorted({b for _,b,_ in idx});rounds=range(1,37)
    assert len(data)==len(idx)==36*9*6
    arms=("refill","known","compact","cut1x8","cut8x8","cut8x1")
    fs=("prepare_mark_ns","discovery_ns","root_merge_ns","publish_ns","psi_build_ns","group_stage_ns","total_ns")
    sums={(a,f):[sum(int(idx[r,b,a][f]) for b in blocks) for r in rounds] for a in arms for f in fs}
    print(p.name,"means in ms per nine admitted blocks across 36 rotations")
    for a in arms:
        print(a,*(f"{sum(sums[a,f])/36e6:.3f}" for f in fs))
    for a,b in (("cut1x8","compact"),("cut8x8","compact"),("cut8x8","cut8x1"),("cut1x8","refill"),("cut8x8","known")):
        print("paid total",a,"/",b,pair(sums[a,"total_ns"],sums[b,"total_ns"]))
    for a in ("cut1x8","cut8x8"):
        free=[v-i for v,i in zip(sums[a,"total_ns"],sums[a,"psi_build_ns"])]
        print("free Psi index",a,"/ compact",pair(free,sums["compact","total_ns"]))
    print("walker cut8x8 / cut8x1",pair(sums["cut8x8","group_stage_ns"],sums["cut8x1","group_stage_ns"]))

DATA=Path(sys.argv[1]) if len(sys.argv)>1 else HERE
for p in ("phase.csv","repeat.csv"):panel(DATA/p)
geom=rows(DATA/"geometry.csv");assert len(geom)==228*6
struct={(int(r["stream"]),int(r["threshold"])):r for r in rows(HERE.parent/"widths/real.csv")}
for r in geom:
    seq=int(r["stream"])
    if r["arm"] in ("cut1x8","cut8x8","cut8x1"):
        t=1 if r["arm"]=="cut1x8" else 8
        if (seq,t) in struct:
            ref=struct[seq,t]
            for n,k in (("original_runs","source_runs"),("psi_cells","tail_table"),("seed_groups","cold_groups"),
                        ("source_gaps","tail_entrances"),("cold_rows","cold_rows"),("terminal_spans","tail_terminal_spans"),
                        ("staged_bytes","tail_spell_bytes_by_span")):
                assert int(r[n])==int(ref[k]),(seq,t,n,r[n],ref[k])
arms=sorted({r["arm"] for r in geom})
print("scratch capacity: frozen / maximum over 228, bytes")
for a in arms:
    group=[r for r in geom if r["arm"]==a]
    print(a,group[0]["total_capacity"],max(int(r["total_capacity"]) for r in group))
p=DATA/"rss.csv"
if p.exists():
    rs=rows(p);assert len(rs)==228*6
    print("isolated first-stage process high water KiB across 228 admissions")
    by={(r["stream"],r["arm"]):int(r["total_capacity"]) for r in geom}
    for a in arms:
        subset=[r for r in rs if r["arm"]==a]
        assert all(int(r["scratch_capacity_bytes"])==by[r["stream"],a] for r in subset)
        print(a,max(int(r["process_peak_kib"]) for r in subset))
