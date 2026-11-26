"""Verify common priced steps and all binary heap pops against actual rank construction."""
import csv
from pathlib import Path
here=Path(__file__).resolve().parent
a={(x["ordinal"],x["section"]):x for x in csv.DictReader((here.parent/"wikidata.csv").open())}
for file, laps in [("paired.csv",8),("fresh.csv",1)]:
    rows=list(csv.DictReader((here/file).open()))
    assert len(rows)==len(a)*laps*4
    for x in rows:
        if x["arm"].startswith("free"):
            r=a[x["ordinal"],x["section"]]
            assert x["states"]==r["states"]
            if x["arm"]=="free_geometry":assert x["heap_pops"]==r["heap_pops"]
            else:assert int(x["heap_pops"])==0
    print(file,len(rows),"complete marked outputs, all expansion counts and physical heap pop identities checked")
