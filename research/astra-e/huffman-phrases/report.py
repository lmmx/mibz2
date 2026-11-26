"""Report each scanner treatment against the frozen stock and its compiled same-source twin."""
from collections import Counter
import csv
import json
import os
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"bench/moonshot"))
from moonshot import paired_ratio

def main(path):
    rows=list(csv.DictReader(path.open()))
    third="scatter" if any(r["arm"]=="scatter" for r in rows) else "triple"
    fourth=next((a for a in ("interval","tail") if any(r["arm"]==a for r in rows)), "quad")
    arms=["stock","twin",third,fourth]
    rounds=len(rows)//len(arms)
    assert len(rows)==rounds*4 and rounds>=24 and rounds%8==0
    table={}
    for i,r in enumerate(rows):
        j=i%4
        lap=i//4
        expected=arms[(lap+(j if (lap//4)%2==0 else 3-j))%4]
        assert (int(r["round"]),int(r["position"]),r["arm"])==(lap,j,expected)
        assert (r["blocks"],r["symbols"])==("431","24979299")
        table[lap,r["arm"]]=r
    assert len({r["metadata_fold"]for r in rows})==1
    positions=Counter((r["arm"],r["position"])for r in rows)
    assert set(positions.values())=={rounds//4}
    out=csv.writer(sys.stdout,lineterminator="\n")
    out.writerow(["metric","candidate","reference","paired_rounds","geo","lo95","hi95"])
    metrics=["thread_ns","wall_ns"]
    if rows and "table_ns" in rows[0]:
        for r in rows:
            r["outside_table_ns"]=int(r["thread_ns"])-int(r["table_ns"])
            assert 0 < r["outside_table_ns"] < int(r["thread_ns"])
        metrics.extend(["table_ns","outside_table_ns"])
    for metric in metrics:
        for arm,ref in [("twin","stock"),(third,"stock"),(fourth,"stock"),(third,"twin"),(fourth,"twin"),(fourth,third)]:
            c=[int(table[r,arm][metric])for r in range(rounds)]
            b=[int(table[r,ref][metric])for r in range(rounds)]
            result=paired_ratio(c,b)
            out.writerow([metric,arm,ref,rounds,*[f"{result[k]:.8f}" for k in ("geomean","lo","hi")]])
if __name__=="__main__":main(Path(sys.argv[1]) if len(sys.argv)>1 else HERE/"times.csv")
