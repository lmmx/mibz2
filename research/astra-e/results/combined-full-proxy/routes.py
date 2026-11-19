"""Cross-check sample versus corpus output weights with the archived route census."""
import csv
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(root / "bench/moonshot"))
import moonshot

census = root / "docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv"
with census.open() as source:
    rows = list(csv.DictReader(source))
assert [int(row["ordinal"]) for row in rows] == list(range(len(rows)))
sample = moonshot.sample_blocks(len(rows), 16)
groups = {"frozen": [1], "density-pair": [41, 85], "official-16": sample, "all": range(len(rows))}
report = {}
for name, ordinals in groups.items():
    counts = {route: {"blocks": 0, "output_bytes": 0} for route in ["open", "lazy", "column"]}
    for ordinal in ordinals:
        row = rows[ordinal]
        route = "column"
        if row["run_front"] == "1":
            route = "open" if 20 * int(row["records"]) < 3 * int(row["rows"]) else "lazy"
        counts[route]["blocks"] += 1
        counts[route]["output_bytes"] += int(row["output"])
    total = sum(x["output_bytes"] for x in counts.values())
    report[name] = {"blocks": len(ordinals), "routes": counts, "output_bytes": total,
                    "admitted_byte_fraction": counts["open"]["output_bytes"] / total}
print(json.dumps(report, indent=2))
