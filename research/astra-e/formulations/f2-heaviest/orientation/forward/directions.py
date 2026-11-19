"""Price indexed forward versus independent backward Direct with matched section addressing."""
import argparse
import csv
import json
from pathlib import Path
import subprocess
from summarize import interval
from price import HEADER
from census import Column,heavy_set,paths,read_input


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    p.add_argument("--recycled",action="store_true")
    a=p.parse_args()
    paired={}
    stem="recycled-directions" if a.recycled else "directions"
    raw=a.out/(stem+".csv")
    with raw.open("w") as f:
        sink=csv.writer(f,lineterminator="\n");sink.writerow(("ordinal","section",*HEADER))
        for path in paths(a.tapes):
            ordinal=int(path.name.split(".")[0]);runs,mark=read_input(path)
            col=Column(runs);parent=col.interval(b'"')
            for section,ints in (("quote",[parent]),("frequent",heavy_set(col,parent,1)[0])):
                head=f"{col.rows} {mark} {len(runs)} {len(ints)}\n"
                data=(head+"".join(f"{x} {y}\n" for x,y in [*runs,*ints])).encode()
                rows=subprocess.check_output([a.binary.resolve(),"16",*(["RECYCLE"] if a.recycled else [])],input=data).decode().splitlines()
                assert len(rows)==32
                for line in rows:
                    fields=line.split(",")
                    rnd,pos,arm=map(int,fields[:3]);assert arm==pos^(rnd%4 in (1,2))
                    sink.writerow((ordinal,section,*fields))
                f.flush()
    with raw.open() as f:
        rows=list(csv.DictReader(f))
    for s in ("quote","frequent"):
        cells=[r for r in rows if r["section"]==s]
        scores={}
        for k in ("cpu_ms","wall_ms","setup_ms","emit_ms"):
            ratios=[]
            for i in range(16):
                controls=[sum(float(r[k]) for r in cells if int(r["round"])==i and int(r["arm"])==arm) for arm in (0,1)]
                ratios.append(controls[1]/controls[0])
            scores[k]=interval(ratios)
        # Spelling is produced in the discovery phase by Direct and in a separate pass by forward.
        ratios=[]
        for i in range(16):
            vals=[sum(float(r["discover_ms"])+float(r["dictionary_ms"])+float(r["order_ms"]) for r in cells
                      if int(r["round"])==i and int(r["arm"])==arm) for arm in (0,1)]
            ratios.append(vals[1]/vals[0])
        scores["discovery_spelling_order_wall"]=interval(ratios)
        paired[s]=scores
    (a.out/(stem+".json")).write_text(json.dumps(paired,indent=2)+"\n")


if __name__=="__main__":
    main()
