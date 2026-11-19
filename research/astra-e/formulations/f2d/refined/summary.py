"""Validate original dynamic geometry and report complete constructor charges."""
import csv
from pathlib import Path
HERE=Path(__file__).resolve().parent
def records(p):
    with p.open() as f: return list(csv.DictReader(f))
rows=records(HERE/"real.csv")
wave={int(r["stream"]):r for r in records(HERE.parents[1]/"f2-scheduler/native-waves.csv")}
learned={int(r["stream"]):int(r["lookups"]) for r in records(HERE.parent/"frontier/geometry.csv") if r["lookups"]!=""}
assert len(rows)==20
for row in rows:
    s=int(row["stream"])
    if s==41:continue
    assert int(row["base_queries"])==int(wave[s]["lookups"])
    assert int(row["scalar_waves"])==int(wave[s]["waves"])
for label,streams in (("frozen",{1}),("nine_admitted",{int(r["stream"]) for r in rows if r["stream"]!="41"})):
    print(label)
    print("learned original ranks",sum(learned[s] for s in streams))
    for k in (2,4):
        rs=[r for r in rows if int(r["stream"]) in streams and int(r["k"])==k]
        print("k",k)
        for name in ("runs","base_queries","queries","saved_queries","scalar_waves","waves","scalar_span","span",
                     "full_cells","stop_cells","distinct_cells_used","original_run_demand",
                     "doubling_setup_emissions","doubling_start_seeks","lazy_doubling_emissions",
                     "lazy_extra_ranks","lazy_original_ranks","jumps","scalar_advances","quote_terminations","branch_terminations"):
            print(name,sum(int(r[name]) for r in rs))
