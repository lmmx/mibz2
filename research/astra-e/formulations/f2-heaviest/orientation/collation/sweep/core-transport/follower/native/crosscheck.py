"""Local projected discovery and physical ordinal publication against independent original-rank geometry."""
from pathlib import Path
import csv
import sys

D=Path(__file__).resolve().parent
local=Path(sys.argv[1]) if len(sys.argv)>1 else D/"ten.csv"
ref={(r["ordinal"],r["h"]):r for r in csv.DictReader((D.parent/"frequent.csv").open())}
rows=list(csv.DictReader(local.open()))
assert len(rows)==30
for row in rows:
    r=ref[row["stream"],row["h"]]
    assert int(row["hi"])-int(row["lo"])==int(r["mass"])
    for a,b in [("words","refined_K"),("dict_bytes","refined_D"),("max_mass","max_label_mass"),
                ("origin_bands","prefix_bands"),("overlay_fragments","overlay_fragments"),
                ("live_states","lookup_states"),("core_runs","core_R"),("physical_runs","refined_R")]:
        assert row[a]==r[b],(row["stream"],row["h"],a,row[a],r[b])
print("30 physical RLE objects agree with the independent constructor on mass and eight exact geometry/payload quantities")
