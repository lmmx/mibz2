"""Apply the scorer's paired estimator to candidate and opend from identical rounds."""
from pathlib import Path
import json,sys
from statistics import median
d=Path(__file__).resolve().parent
sys.path.insert(0,str(d.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio

rows=[json.loads(line) for line in (d/"score/runs.jsonl").read_text().splitlines()]
results={}
for workers in (1,4):
    arms={key: sorted([r for r in rows if r["arm"]==key and r["workers"]==workers],
                       key=lambda r:r["round"]) for key in ("candidate","opend","lazy")}
    assert all([r["round"] for r in arm]==list(range(9)) for arm in arms.values())
    result={}
    for ref in ("lazy","opend"):
        candidate=arms["candidate"]
        control=arms[ref]
        def paired(field):
            return paired_ratio([r[field] for r in candidate],[r[field] for r in control])
        result[ref]={
            "wall_ratio":paired("wall_s"),
            "cpu_ratio":paired("cpu_s"),
            "median_peak_rss_kb":{"candidate":median(r["peak_rss_kb"] for r in candidate),
                                  ref:median(r["peak_rss_kb"] for r in control)},
            "paired_peak_rss_ratio":median(x["peak_rss_kb"]/y["peak_rss_kb"] for x,y in zip(candidate,control))
        }
    results[str(workers)]=result
print(json.dumps(results,indent=2))
