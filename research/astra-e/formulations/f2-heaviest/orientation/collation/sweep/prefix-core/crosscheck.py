"""Compare two independently built first-level objects where the two cut rules coincide."""
import csv
import io
import subprocess
from pathlib import Path

HERE=Path(__file__).resolve().parent
PIN="11cc7ee"
TABLE="research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/short-core/wikidata.csv"
oracle=subprocess.check_output(["git","show",f"{PIN}:{TABLE}"]).decode()
expected={(r["ordinal"],r["cap"]):r for r in csv.DictReader(io.StringIO(oracle)) if r["section"]=="frequent"}
pairs=[("mass","mass"),("prefix_types","prefix_types"),("final_tags","terminal_bands"),
       ("forward_pieces","forward_intersections"),("full_words","types"),("core_words","core_types"),
       ("core_bytes","core_D"),("prefix_bytes","prefix_dictionary_bytes"),("live_states","core_Q")]
cases=0
with (HERE/"real.csv").open() as f:
    for row in csv.DictReader(f):
        key=row["ordinal"],row["h"]
        if key not in expected:continue
        for native,ref in pairs:
            assert int(row[native])==int(expected[key][ref]),(key,native,ref)
        cases+=1
assert cases==20
print(f"{cases} real h=3,8 cells matched all {len(pairs)} independently computed forward, ownership, dictionary and live-state metrics at {PIN}")
