"""Structural panel with authenticated supplied last-column and k8 event geometry."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess as sp
import sys

d=Path(__file__).resolve().parent
exe,wires,out=map(Path,sys.argv[1:])
inputs=[]
with (out/"orders.csv").open("wb") as f:
    f.write(b"block,section,policy,merged,n,kept,coordinate_seams,initial_atoms,adjacent_edges,stabbing_step_floor,steps,clipped,final_bands\n")
    f.flush()
    for w in sorted(wires.glob("*.mask8"),key=lambda a:int(a.stem)):
        b=int(w.stem)
        sp.run([exe,str(b),w],stdout=f,check=True)
        inputs.append({"block":b,"sha256":hashlib.sha256(w.read_bytes()).hexdigest()})
assert len(inputs)==10
rows=list(csv.DictReader((out/"orders.csv").open()))
summary=[]
for section in ["seed8","heads","prefix"]:
    for layout in ["stable","reverse_spans","long_first","short_first","reflected"]:
        for m in ["0","1"]:
            chosen=[r for r in rows if r["section"]==section and r["policy"]==layout and r["merged"]==m]
            assert len(chosen)==10
            sums={k:sum(int(x[k]) for x in chosen) for k in ("n","kept","coordinate_seams","initial_atoms",
                    "adjacent_edges","stabbing_step_floor","steps","clipped","final_bands")}
            summary.append(dict(section=section,policy=layout,merged=m,**sums))
with (out/"totals.csv").open("w") as f:
    wr=csv.DictWriter(f,list(summary[0]),lineterminator="\n");wr.writeheader();wr.writerows(summary)
(out/"wires.json").write_text(json.dumps(inputs,indent=2)+"\n")
