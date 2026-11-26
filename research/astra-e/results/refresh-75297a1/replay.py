"""Replay every paired contrast of the single six-arm rescoring series."""
import csv
import json
import math
from pathlib import Path
from statistics import median
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[3]/"bench/moonshot"))
from moonshot import paired_ratio

names = ("combined", "final-slot", "edge", "guarded", "preload", "compact")
writer = None
for name in names:
    folder = HERE.parent / f"v1p2-{name}-confirm"
    summary = json.loads((folder/"score/summary.json").read_text())
    assert "scorer path revision 75297a1" in (folder/"manifest.txt").read_text()
    assert "scorer v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04" in (folder/"score/summary.txt").read_text()
    assert "PASS (338 cases, 0 failed)" in (folder/"score/summary.txt").read_text()
    raw = [json.loads(s) for s in (folder/"score/runs.jsonl").read_text().splitlines()]
    by_id = {(r["workers"], r["round"], r["arm"]): r for r in raw}
    assert len(by_id) == len(raw) == 54 and all(r["out_bytes"] == 419400392 for r in raw)
    for w in (1, 4):
        arm = {a:[by_id[w,i,a] for i in range(9)] for a in ("candidate", "opend", "lazy")}
        for ref in ("lazy", "opend"):
            pairs = list(zip(arm["candidate"], arm[ref]))
            ratio = lambda key: paired_ratio([a[key] for a,b in pairs], [b[key] for a,b in pairs])
            wall, cpu = ratio("wall_s"), ratio("cpu_s")
            peak = median(a["peak_rss_kb"] for a,b in pairs) / median(b["peak_rss_kb"] for a,b in pairs)
            if ref == "lazy":
                x = summary["timing"][str(w)]["candidate"]
                assert all(math.isclose(x[k], wall[k]) for k in ("geomean","lo","hi"))
                assert math.isclose(peak, x["peak_rss_ratio"])
            out = dict(candidate=name, workers=w, reference=ref,
                       wall=wall["geomean"], lo=wall["lo"], hi=wall["hi"],
                       median_peak=peak, cpu_s_ratio=cpu["geomean"],
                       largest_observed_pair=max(a["wall_s"]/b["wall_s"] for a,b in pairs),
                       slowest_candidate_wall_s=max(a["wall_s"] for a,b in pairs),
                       slowest_reference_wall_s=max(b["wall_s"] for a,b in pairs))
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=out, lineterminator="\n")
                writer.writeheader()
            writer.writerow(out)
