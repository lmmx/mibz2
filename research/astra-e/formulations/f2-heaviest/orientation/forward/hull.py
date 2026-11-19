"""Price the exact opportunity for sharing one displacement across a fragmented prefix."""
import argparse
import csv
import hashlib
from itertools import groupby
import json
from pathlib import Path
import subprocess
from census import Column,heavy_set,paths,read_input
from family import bwt,debruijn

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("output",type=Path)
    a=p.parse_args();rows=[]
    def measure(name,sect,runs,n,mark,ints,expected_digest):
        head=f"{n} {mark} {len(runs)} {len(ints)}\n"
        data=(head+"".join(f"{x} {y}\n" for x,y in [*runs,*ints])).encode()
        j=json.loads(subprocess.check_output([a.binary.resolve()],input=data))
        assert hashlib.sha256(bytes.fromhex(j["expanded_hex"])).hexdigest()==expected_digest
        rows.append(dict(input=name,section=sect,**{k:j[k] for k in ("groups","packet_inputs","select_translations",
            "multi_packet_groups","multi_packet_inputs","hull_translates","hull_input_savings")}))
    with (Path(__file__).parent/"real.csv").open() as f:
        baseline={(r["ordinal"],r["section"]):r["output_sha256"] for r in csv.DictReader(f)}
    for path in paths(a.tapes):
        ordinal=str(int(path.name.split(".")[0]));runs,mark=read_input(path);col=Column(runs)
        parent=col.interval(b'"')
        for sect,ints in (("quote",[parent]),("frequent",heavy_set(col,parent,1)[0])):
            measure(ordinal,sect,runs,col.rows,mark,ints,baseline[ordinal,sect])
    for q in (3,5,7):
        for k in (4,16,32):
            toks=debruijn(q)
            word=[c for t in toks for c in ([0]+[2]*t+[1]*k)]
            last,mark=bwt(word);runs=[(c,len(list(g))) for c,g in groupby(last)]
            measure(f"debruijn-{q}-{k}","d",runs,len(last),mark,[(0,len(toks))],hashlib.sha256(bytes(word)).hexdigest())
    with a.output.open("w") as f:
        w=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n");w.writeheader();w.writerows(rows)

if __name__=="__main__":
    main()
