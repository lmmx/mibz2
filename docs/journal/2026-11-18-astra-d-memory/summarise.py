"""Replay the machine references, preserve per-region counts, then aggregate by output."""
import csv
import subprocess
import sys
from pathlib import Path
from collections import Counter
from io import StringIO

replay, root, evidence = map(Path,sys.argv[1:4])
mode = sys.argv[4]
evidence.mkdir(parents=True,exist_ok=True)
output = evidence / (mode+"-regions.csv")
totals = []
with output.open("w") as f:
    detail = None
    for run in sorted(p for p in root.iterdir() if p.is_dir()):
        text = (run/"ranges").read_text()
        info = {}
        for line in text.splitlines():
            if line.startswith("info "):
                args=line.split()[1:]
                info.update(dict(zip(args[::2],map(int,args[1::2]))))
        ins = dict((int(p),int(n)) for p,n in csv.reader((run/"instructions").open()))
        for kib in (1024,2048,4096):
            out = subprocess.check_output([replay,run/"ranges",run/"events",str(kib)],text=True)
            table = csv.DictReader(StringIO(out))
            if detail is None:
                detail = csv.writer(f,lineterminator="\n")
                detail.writerow(["ordinal","llc_KiB",*table.fieldnames])
            total = Counter()
            for row in table:
                if kib == 2048: detail.writerow([int(run.name),kib,*row.values()])
                total.update({k:int(v) for k,v in row.items() if k not in ("phase","region")})
            assert total["l1_rmiss"]+total["l1_wmiss"] == total["l2_req"]
            assert total["l2_miss"] == total["llc_req"]
            assert total["dram_read"]+total["dram_rfo"] == total["llc_miss"]
            totals.append(dict(ordinal=int(run.name),llc_KiB=kib,output=info["output"],
                cells=info["cells"],direct_scan=int(12 in ins),instructions=sum(ins.values()),
                **total))
with (evidence/(mode+"-blocks.csv")).open("w") as f:
    w=csv.DictWriter(f,fieldnames=totals[0],lineterminator="\n")
    w.writeheader()
    w.writerows(totals)
