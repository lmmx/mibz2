"""Aggregate physical per-block durations before calculating each corpus share or ratio."""
from collections import defaultdict
import csv
import lzma
from pathlib import Path
from statistics import median
import sys

here = Path(__file__).resolve().parent
source = Path(sys.argv[1]) if len(sys.argv) > 1 else here
dest = Path(sys.argv[2]) if len(sys.argv) > 2 else here

def read(name):
    path = source / (name + ".csv")
    with (open(path, "rt") if path.exists() else lzma.open(str(path)+".xz", "rt")) as stream:
        return list(csv.DictReader(stream))

stage = defaultdict(lambda: defaultdict(int))
geometry = defaultdict(lambda: defaultdict(int))
arms = ("lazy","opend","ahead","refill","bundle")
parts = ("front","inverse","rle","crc","total")
keyfile = here.parent / "formulations/f2-scheduler/publication/phase.csv"
oracle9 = {int(r["stream"]) for r in csv.DictReader(keyfile.open())}
native = {int(r["stream"]) for r in csv.DictReader((here.parent/"formulations/f2-scheduler/native-waves.csv").open())}
counts = defaultdict(int)
for row in read("stages"):
    arm = row["arm"]
    rnd, index, n, out, caps, fused = [int(row[k]) for k in ("round","block","rows","output","caps","fused")]
    group = "admitted" if index in native else ("large-other" if n >= 65536 else "tail-run" if caps else "tail-column")
    if n >= 65536:
        assert (group == "admitted") == (caps > 0 and caps * 20 < 3*n and caps * 64 >= n)
    assert (arm == "bundle" and group == "admitted") == bool(fused)
    assert sum(int(row[p+"_ns"]) for p in parts[:-1]) == int(row["total_ns"])
    assert fused == 0 or row["rle_ns"] == "0"
    counts[arm,rnd] += 1
    for g in ("all",group) + (("oracle9",) if index in oracle9 else ()):
        s = stage[arm,rnd,g]
        if arm=="lazy" and rnd==1:
            t=geometry[g]
            t["blocks"]+=1
            t["rows"]+=n
            t["output"]+=out
        for p in parts:
            s[p] += int(row[p+"_ns"])
rounds = sorted({r for _,r in counts})
assert len(native)==228 and len(oracle9)==9 and oracle9 <= native
assert len(counts)==len(arms)*len(rounds) and set(counts.values())=={431}
single = defaultdict(lambda: defaultdict(int))
for key in ("scanner","copy"):
    seen = set()
    for row in read(key):
        rnd,index=int(row["round"]),int(row["block"])
        assert (rnd,index) not in seen
        seen.add((rnd,index))
        single[rnd][key] += int(row["ns"])
    assert len(seen)==len(rounds)*(431+(key=="scanner"))

def writer(filename, header):
    f = (dest / filename).open("w")
    w = csv.writer(f,lineterminator="\n")
    w.writerow(header)
    return f,w

f,w = writer("classes.csv",["group","blocks","rows","output","lazy_worker_ms","lazy_inverse_ms","lazy_inverse_share_all_worker","lazy_worker_share_all_worker"])
for g,geom in geometry.items():
    group_per_round = [stage["lazy",r,g] for r in rounds]
    denom = [stage["lazy",r,"all"]["total"] for r in rounds]
    w.writerow([g,geom["blocks"],geom["rows"],geom["output"],
                *(round(median(s[k] for s in group_per_round)/1e6,4) for k in ("total","inverse")),
                *(round(median(s[k]/d for s,d in zip(group_per_round,denom)),6) for k in ("inverse","total"))])
f.close()
f,w = writer("scope-summary.csv",["arm",*(p+"_ms" for p in parts),"inverse_worker_share","paired_worker_ratio_over_lazy","min_worker_ratio","max_worker_ratio"])
for arm in arms:
    values = [stage[arm,r,"all"] for r in rounds]
    rel = [s["total"]/stage["lazy",r,"all"]["total"] for s,r in zip(values,rounds)]
    w.writerow([arm,*(round(median(s[p] for s in values)/1e6,4) for p in parts),
                round(median(s["inverse"]/s["total"] for s in values),6),round(median(rel),6),round(min(rel),6),round(max(rel),6)])
f.close()
f,w = writer("model.csv",["denominator","affected","fraction","min_fraction","max_fraction","ratio_at_0.675","ratio_if_free","largest_stage_ratio_for_0.80"])
for scope in ("worker","worker+scan","worker+scan+copy"):
    for group in ("oracle9","admitted","all"):
        values=[]
        for r in rounds:
            divisor=stage["lazy",r,"all"]["total"]
            if "scan" in scope: divisor+=single[r]["scanner"]
            if "copy" in scope: divisor+=single[r]["copy"]
            values.append(stage["lazy",r,group]["inverse"]/divisor)
        share=median(values)
        w.writerow([scope,group,*(round(x,6) for x in (share,min(values),max(values),1-share*.325,1-share,1-.2/share))])
f.close()
f,w=writer("solo.csv",["scope","median_ms","min_ms","max_ms"])
for key in ("scanner","copy"):
    values=[single[r][key]/1e6 for r in rounds]
    w.writerow([key,*(round(x,4) for x in (median(values),min(values),max(values)))])
f.close()
