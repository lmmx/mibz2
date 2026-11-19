"""Bound the complete edge table; report additional large admissions in each corpus."""
import csv
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[3]
journal = root / "docs/journal"
d = lambda f: list(csv.DictReader(f.open()))
route = {int(x["ordinal"]):x for x in d(journal/"2026-11-18-astra-d-memory/evidence/corpus-routes.csv")}
writer = csv.writer(sys.stdout,lineterminator="\n")
writer.writerow(["input", "ordinal", "rows", "original_records_or_prose_lower_bound", "retained", "strips", "guaranteed_edge_bytes", "extra_open"])
for x in d(journal/"2026-11-18-open-return/evidence/blocks.csv"):
    if x["corpus"] not in ["wikidata","prose"] or int(x["n"]) < 65536:
        continue
    ordinal,n,m,h = (int(x[k]) for k in ["ordinal","n","m","H"])
    recs = int(route[ordinal]["records"] if x["corpus"]=="wikidata" else x["r"])
    if recs * 20 < n * 3:
        continue
    bound = (4 if 511+3*h <= 0xffffffff >> max(1,(m-1).bit_length()) else 8) * m
    writer.writerow([x["corpus"],ordinal,n,recs,m,h,bound,int(bound<=1<<20)])
