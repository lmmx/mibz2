"""Compare compressed native publication and second output with the independent two-section lift."""
import csv
from pathlib import Path
import sys

D=Path(__file__).resolve().parent
path=Path(sys.argv[1]) if len(sys.argv)>1 else D/"ten.csv"
ref={(r["ordinal"],r["h"]):r for r in csv.DictReader((D.parent/"ten.csv").open()) if r["section"]=="frequent"}
checked=0
for r in csv.DictReader(path.open()):
    arm=r["arm"]
    if not arm.startswith("lift") or arm.endswith("h0"):continue
    other=ref[r["stream"],arm[-1:]]
    force=arm.startswith("lift_core")
    count=lambda key:int(other[key])
    assert int(r["hi"])-int(r["lo"])==count("m")
    assert int(r["aliases"])==(count("capped_extra_ids") if force else 0)
    assert int(r["prefix_runs"])==count("prefix_runs")+(count("capped_extra_runs") if force else 0)
    assert int(r["words"])==count("core_types")+count("prefix_ids")+int(r["aliases"])
    for a,b in [("origin_bands","atlas_bands"),("core_types","core_types"),("prefix_bins","prefix_ids"),
                ("core_runs","core_runs"),("mark","mark_lifted_row"),("skip","mark_original_lf_steps"),
                ("second_links","core_mass" if force else "stock_mass")]:
        assert r[a]==other[b],(arm,r["stream"],a,r[a],other[b])
    if force or other["stock_heaviest"]=="core":
        for a,b in [("second_words","capped_second_types" if force else "second_types"),
                    ("second_dict_bytes","capped_second_dictionary_bytes" if force else "second_naive_bytes")]:
            assert r[a]==other[b],(arm,r["stream"],a,r[a],other[b])
    checked+=1
assert checked==40
print("40 independent capped/uncapped ordinal geometries, marks and winning masses verified")
