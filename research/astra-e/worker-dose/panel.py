"""Use the live scorer's paired process runner with a separately compiled edge reference."""
import argparse
import csv
import json
from pathlib import Path
import statistics
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"ci"))
import paired
from moonshot import order, paired_ratio

ARMS=["lazy","cells","edge","candidate"]
WORKERS=[2,4]
ROUNDS=16

def compare(out):
    rows=[json.loads(line) for line in (out/"runs.jsonl").read_text().splitlines()]
    expected=[(i,w,a) for i in range(ROUNDS) for w in WORKERS for a in order(ARMS,i+w)]
    assert expected==[(r["round"],r["workers"],r["arm"]) for r in rows]
    assert all(r["out_bytes"]==419400392 for r in rows)
    result=[]
    for w in WORKERS:
        arms={a:[r for r in rows if r["workers"]==w and r["arm"]==a] for a in ARMS}
        for ref in ARMS[:-1]:
            def series(arm,k):return [r[k] for r in arms[arm]]
            wall=paired_ratio(series("candidate","wall_s"),series(ref,"wall_s"))
            cpu=paired_ratio(series("candidate","cpu_s"),series(ref,"cpu_s"))
            peak=paired_ratio(series("candidate","peak_rss_kb"),series(ref,"peak_rss_kb"))
            result.append(dict(workers=w,reference=ref,wall=wall["geomean"],lo=wall["lo"],hi=wall["hi"],
                        cpu=cpu["geomean"],median_peak=statistics.median(series("candidate","peak_rss_kb"))/statistics.median(series(ref,"peak_rss_kb")),
                        peak_lo=peak["lo"],peak_hi=peak["hi"]))
    with (out/"comparison.csv").open("w") as f:
        writer=csv.DictWriter(f,fieldnames=result[0],lineterminator="\n");writer.writeheader();writer.writerows(result)

if __name__=="__main__":
    p=argparse.ArgumentParser()
    for key in ("edge","dose","corpus","out"):
        p.add_argument(key,type=Path)
    p.add_argument("--replay",action="store_true")
    a=p.parse_args()
    if not a.replay:
        runner=paired.Paired(a.edge,a.dose,419400392,a.edge)
        runner.binaries["edge"]=(a.edge,"candidate","edge")
        runner.arms=ARMS
        runner.a.workers=WORKERS;runner.a.rounds=ROUNDS;runner.a.repeat=2
        runner.timing(a.corpus,a.out,ARMS)
    compare(a.out)
