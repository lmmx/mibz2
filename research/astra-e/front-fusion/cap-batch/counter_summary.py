"""Replay client-request model counters from the sixteen cold-modeled geometry regions."""
import csv
import re
import sys
import tarfile
from pathlib import Path
from statistics import median

d=Path(__file__).resolve().parent
with tarfile.open(d/"counters.tar.xz") as archive:
    def read(path):
        return archive.extractfile(path).read().decode()
    writer=csv.writer(sys.stdout,lineterminator="\n")
    fields=["Ir","Dr","Dw","I1mr","D1mr","D1mw","ILmr","DLmr","DLmw","Bc","Bcm","Bi","Bim"]
    writer.writerow(["stream","arm"]+fields)
    for arm in range(4):
        ids=list(map(int,re.findall(r'block (\d+), \d+ checked decoded bytes', read(f"run.{arm}"))))
        assert ids==[1,19,41,56,61,108,150,165,194,240,270,278,282,329,371,407]
        for dump,idx in enumerate(ids,1):
            s=read(f"ir.{arm}.{dump}")
            assert "Trigger: Client Request: geometry" in s
            assert re.search(r"^events: (.*)",s,re.M)[1].split()==fields
            values=[int(v) for v in re.search(r"^summary: (.*)",s,re.M)[1].split()]
            values += [0]*(len(fields)-len(values))
            writer.writerow([idx,arm]+values)
