"""Sum actual per-block lookups without conflating static table entry counts."""
from collections import Counter
import csv
from pathlib import Path
import sys
root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent
modes={}
blocks={}
for r in csv.DictReader((root/"census.csv").open()):
    mode=r["mode"]
    c=modes.setdefault(mode,Counter())
    c.update({k:int(v) for k,v in r.items() if k not in ("mode","block")})
    blocks.setdefault(mode,[]).append(int(r["block"]))
ref=sum(modes["pair"][k] for k in ("body","tail","tail_single"))
out=csv.writer(sys.stdout,lineterminator="\n")
out.writerow(["mode","blocks","invocations","relative_to_pair",*modes["pair"].keys()])
for m,c in modes.items():
    assert blocks[m]==list(range(431)) and c["symbols"]==24979299 and c["tables"]==1796
    t=c["body"]+c["tail"]+c["tail_single"]
    assert sum(c[k] for k in ("one","two","three","four"))==t
    assert sum(c[k] for k in ("table_long","table_one","table_two","table_three","table_four"))==1024*c["tables"]
    out.writerow([m,len(blocks[m]),t,f"{t/ref:.9f}",*c.values()])
