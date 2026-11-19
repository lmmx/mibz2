"""Compare every systematic native construction with independent physical-rank core transport."""
from pathlib import Path
import csv

D=Path(__file__).resolve().parent
reference={(r["ordinal"],r["h"]):r for r in csv.DictReader((D.parent/"wikidata.csv").open())}
local=list(csv.DictReader((D/"ten.csv").open()))
assert len(local)==30
for row in local:
    r=reference[row["stream"],row["h"]]
    assert int(row["hi"])-int(row["lo"])==int(r["mass"])
    for key, other in [("live_states","live_states"),("core_runs","core_runs"),("words","full_types"),
                       ("origin_bands","origin_bands"),("transport_segments","transport_segments"),("full_runs","full_runs")]:
        assert row[key]==r[other],(key,row,r)
    assert int(row["overlay_slices"])<=int(r["core_types"])+int(row["origin_bands"])-1
    assert row["transport_segments"]==row["full_runs"]
print(f"{len(local)} independent geometries agree in all seven outputs and both transport bounds")
