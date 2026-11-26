"""Sum per-block median inverse-stage measurements on matched systematic inputs."""
import csv
from pathlib import Path
from statistics import median

d = Path(__file__).resolve().parent
rows = list(csv.DictReader((d / "blocks.csv").open()))
warm = [r for r in rows if r["arm"] == "endpoint" and r["round"] == "0"]
assert len(warm) == 228
print(f"{len(warm)} admitted blocks, {sum(int(r['rows']) for r in warm):,} inverse bytes")
for scope in (["1"],sorted({r["stream"] for r in rows if r["round"] != "0"}, key=int)):
    print("Streams", ",".join(scope))
    times = {}
    for arm in ("endpoint","trace","lazy"):
        selected = [r for r in rows if r["round"] != "0" and r["arm"] == arm]
        times[arm] = [sum(median(int(r[k]) for r in selected if r["stream"] == i) for i in scope) for k in ("thread_ns","wall_ns")]
        print(arm, "thread/wall ms", *(round(t/1e6,4) for t in times[arm]))
    for arm in ("endpoint","trace"):
        print(arm, "thread/wall ratio to Lazy", *(round(t/b,4) for t,b in zip(times[arm],times["lazy"])))
for arm in ("endpoint","trace"):
    r=next(r for r in rows if r["stream"]=="1" and r["arm"]==arm)
    print(arm,"frozen", {k:int(r[k]) for k in ("section_rows","plus_mark","complement_strips","endpoint_nodes","endpoint_steps","rules","bands","spelling_bytes","table_bytes")})
