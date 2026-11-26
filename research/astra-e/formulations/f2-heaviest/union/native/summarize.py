"""Pair full inverse stages across identical system blocks and verify tagged bands."""
import csv
from pathlib import Path
import sys
from statistics import median
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[5]/"bench/moonshot"))
from moonshot import paired_ratio


def rows(path):
    with path.open() as f:
        return list(csv.DictReader(f))

data=rows(here/"phase.csv")
by={(int(r["round"]),int(r["stream"]),r["arm"]):r for r in data}
blocks=sorted({s for _,s,_ in by})
rounds=sorted({r for r,_,_ in by})
assert len(data)==len(by)==9*32*4
fields=("prepare_ns","select_mark_ns","discover_ns","merge_ns","residual_output_ns","total_ns")
for arm in ("eight","four","one","lazy"):
    print(arm, {key:round(sum(median(int(by[r,s,arm][key]) for r in rounds) for s in blocks)/1e6,4) for key in fields})
for a,b in (("eight","one"),("four","one"),("eight","four"),("eight","lazy")):
    for field in (("discover_ns","residual_output_ns","total_ns") if b=="one" else ("total_ns",)):
        totals=[[sum(int(by[r,s,t][field]) for s in blocks) for r in rounds] for t in (a,b)]
        print(f"{a}/{b} {field}",paired_ratio(*totals))
if (here/"geometry.csv").exists():
    geom=rows(here/"geometry.csv")
    assert len(geom)==3*228
    # The independent band constructor operates outside the native stable-merge implementation.
    systematic=rows(here.parent/"wikidata.csv")
    corpus=rows(here.parents[2]/"f2b/geometry.csv")
    ordered=[int(r["stream"]) for r in corpus]
    for row in systematic:
        k=int(row["k"]); stream=ordered[int(row["ordinal"])]
        if k in (1,4,8) and stream in blocks:
            native=next(r for r in geom if int(r["stream"])==stream and int(r["k"])==k)
            assert native["pieces"]==row["colored_runs"] and native["token_rows"]==row["section_rows"]
    print("Native root runs and section sizes match the independent point-band constructor across all systematic admissions.")
    for k in (1,4,8):
        print(k,{f:sum(int(r[f]) for r in geom if int(r["k"])==k) for f in ("token_rows","pieces","spelling_bytes","lookup_steps")})
