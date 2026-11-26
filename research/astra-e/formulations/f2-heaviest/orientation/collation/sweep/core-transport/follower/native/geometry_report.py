from collections import Counter,defaultdict
from pathlib import Path
import csv
import sys

for path in map(Path,sys.argv[1:]):
    rows=defaultdict(Counter)
    with path.open() as f:
        for r in csv.DictReader(f):
            rows[r["arm"]].update({k:int(v) for k,v in r.items() if k.startswith("second_")})
    reference=rows["compact_frequent"]
    for arm in ("original_h0","original_h4","original_h8","physical_h0"):
        assert rows[arm]==reference
    for arm in ("compact_frequent","physical_h4","physical_h8"):
        print(path.name,arm,dict(rows[arm]))
