"""Join instrumented inline rule counts with the archived width-16 endpoint profile."""
import csv
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
f = root / "docs/journal/2026-11-18-open-return/evidence/blocks.csv"
historical = {int(x["ordinal"]): x for x in csv.DictReader(f.open()) if x["corpus"] == "wikidata"}
ordinals = [1, 19, 61, 108, 150, 194, 240, 282, 329, 371, 407]
print("ordinal,rows,primary,steps,old_rules,new_rules,wide_spelling_bytes,retained")
for ordinal, line in zip(ordinals, Path(sys.argv[1]).read_text().splitlines(), strict=True):
    tag, *fields = line.split(",")
    assert tag == "words"
    n, primary, steps, new, traced, retained = map(int, fields)
    old = historical[ordinal]
    assert (n, steps, traced, retained) == tuple(int(old[k]) for k in ("n", "steps", "induced_U", "m"))
    print(f"{ordinal},{n},{primary},{steps},{old['rules']},{new},{traced},{retained}")
