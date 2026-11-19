"""Verify the public CI ZIP, export its official evidence record and price mixed service."""
import csv
import json
from pathlib import Path
import sys
from zipfile import ZipFile
from envelope import paired_ratio, study

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/"retrospective"))
from receipts import ci_receipt

STEMS={
    "primary": ("worker-dose-44448175373","c783b7820a2d89af8d2ddfb77e63f9c017e5247b"),
    "replication": ("worker-dose-44448785057","ecd26dd331c74bdfba1384f8fad6088d1b6dfd13"),
}
STEM,HEAD=STEMS[sys.argv[2] if len(sys.argv)>2 else "primary"]
REF="badb2aec3624bd65606c429b2e6f79e9d9d0050f"
receipt=ci_receipt(STEM)
assert receipt["head"]==HEAD
assert receipt["selected"]=="research/astra-e/worker-dose/candidate.rs"
z=Path(receipt["artifact"])
with ZipFile(z) as archive:
    assert f"matched predecessor {REF} research/astra-e/edge-budget/candidate.rs" in archive.read("manifest.txt").decode().splitlines()
    rows=list(map(json.loads,archive.read("runs.jsonl").splitlines()))
    assert len(rows)==300
    for arm,compiled in (("candidate","candidate"),("predecessor","candidate"),("lazy","lazy"),("opend","opend"),("cells","cells")):
        probes=[r for r in rows if r["arm"]==arm]
        assert len(probes)==60 and all(r["compiled_arm"]==compiled and r["out_bytes"]==838800784 for r in probes)
out=Path(sys.argv[1])
out.mkdir(exist_ok=True,parents=True)
results=study(rows,"candidate","predecessor","cells",lambda w:(w+3)//4)
with (out/"envelope.csv").open("w") as f:
    writer=csv.DictWriter(f,fieldnames=results[0],lineterminator="\n");writer.writeheader();writer.writerows(results)
with (out/"maxima.csv").open("w") as f:
    keys=["workers","arm","min_wall","max_wall","candidate_max_peak_kib","reference_max_peak_kib","rss_upper","cpu_ratio","faster_pairs","smaller_pairs"]
    writer=csv.DictWriter(f,fieldnames=keys,lineterminator="\n");writer.writeheader()
    for w in (1,2,4):
        arms={a:sorted([r for r in rows if r["workers"]==w and r["arm"]==a],key=lambda r:r["round"]) for a in ("candidate","predecessor","lazy","cells","opend")}
        mixed=arms["candidate"]
        for a in ("predecessor","lazy","cells","opend"):
            ref=arms[a]
            writer.writerow(dict(workers=w,arm=a,min_wall=min(r["wall_s"] for r in ref),max_wall=max(r["wall_s"] for r in ref),
                           candidate_max_peak_kib=max(r["peak_rss_kb"] for r in mixed),reference_max_peak_kib=max(r["peak_rss_kb"] for r in ref),
                           rss_upper=paired_ratio([r["peak_rss_kb"] for r in mixed],[r["peak_rss_kb"] for r in ref])["hi"],
                           cpu_ratio=paired_ratio([r["cpu_s"] for r in mixed],[r["cpu_s"] for r in ref])["geomean"],
                           faster_pairs=sum(x["wall_s"]<y["wall_s"] for x,y in zip(mixed,ref)),
                           smaller_pairs=sum(x["peak_rss_kb"]<y["peak_rss_kb"] for x,y in zip(mixed,ref))))
(out/"ci-receipt.json").write_text(json.dumps({"worker_dose_ci_run":receipt},indent=2)+"\n")
