"""Check every persistent worker and actual per-worker run-route traffic."""
from collections import Counter
import csv
import json
from pathlib import Path
import sys

root=Path(sys.argv[1]);writer=None
for w in (1,2,4,7,8,9,10):
    assert json.loads((root/f"w{w}.json").read_text())["out_bytes"]==209700196
    assigned={};logs=Counter(); mass=Counter()
    for line in (root/f"w{w}.log").read_text().splitlines():
        if line.startswith("dose-init "):
            _,k,arm=line.split()
            assert k.startswith("Some(") and k.endswith(")")
            i=int(k[5:-1]);assert i not in assigned
            assigned[i]=arm
        if line.startswith("dose-block "):
            _,worker,size=line.split();i=int(worker.removeprefix("mibz2-"))
            logs[i]+=1;mass[i]+=int(size)
    assert assigned=={i:("eager" if i%4==0 else "edge") for i in range(w)}
    assert set(logs)==set(assigned)
    for role in ("edge","eager"):
        ids=[i for i in assigned if assigned[i]==role]
        row=dict(workers=w,role=role,indices=" ".join(map(str,sorted(ids))),run_blocks=sum(logs[i] for i in ids),run_rows=sum(mass[i] for i in ids))
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=row,lineterminator="\n");writer.writeheader()
        writer.writerow(row)
