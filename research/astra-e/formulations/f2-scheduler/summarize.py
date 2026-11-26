"""Compare rotating paired stage observations and validate predicted against observed wave counts."""
import csv
from collections import defaultdict
from pathlib import Path
from statistics import median
import sys
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio

def read(name):
    with (here/name).open() as f: return list(csv.DictReader(f))
phases=read("phase.csv")
index={(int(r["round"]),r["stream"],r["arm"]):r for r in phases}
blocks=sorted({r["stream"] for r in phases})
rounds=sorted({key[0] for key in index})
assert len(phases)==len(index)==9*30*3
for arm in ("refill","batch","lazy"):
    med={}
    for key in ("prep_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns"):
        med[key]=sum(median(int(index[r,b,arm][key]) for r in rounds) for b in blocks)/1e6
    print(arm," ".join(f"{k.removesuffix('_ns')}={v:.4f}ms" for k,v in med.items()))
for key,ref in [("discover_ns","batch"),("total_ns","batch"),("total_ns","lazy")]:
    samples={a:[sum(int(index[r,b,a][key]) for b in blocks) for r in rounds] for a in ("refill",ref)}
    ratio=paired_ratio(samples["refill"],samples[ref])
    print(f"refill/{ref} {key}: {ratio}")
w=[r for r in read("waves.csv") if r["lanes"]=="8"]
print("nine graph blocks", {k:sum(int(r[k]) for r in w) for k in ("lookups","batch_waves","refill_waves","lower_waves","selectors","dict_bytes")})
if (here/"native-waves.csv").exists():
    actual=read("native-waves.csv")
    if len(actual)==228:
        for row in w:
            got=next(r for r in actual if r["stream"]==row["stream"])
            assert got["waves"]==row["refill_waves"] and got["lookups"]==row["lookups"]
        print("native simulation agreement for every systematic admission")
        waves=sum(int(r["waves"]) for r in actual)
        lookups=sum(int(r["lookups"]) for r in actual)
        print(f"all admissions observed: {lookups} rank lookups in {waves} waves, occupancy {lookups/(8*waves):.6f}")
