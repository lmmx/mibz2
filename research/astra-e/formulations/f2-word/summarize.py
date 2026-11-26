"""Check matching section dimensions, then show the charged costs of both paths."""
import csv
from pathlib import Path
import statistics as stats

here = Path(__file__).resolve().parent

def rows(path):
    with path.open() as source:
        return list(csv.DictReader(source))

direct = {r["stream"]: r for r in rows(here / "geometry.csv") if r["took"] == "1"}
two = {r["stream"]: r for r in rows(here.parent / "f2c/native.csv") if r["took"] == "1"}
assert direct.keys() == two.keys()
concordant = [k for k in direct if direct[k]["token_rows"] == two[k]["token_rows"]
              and direct[k]["dict_bytes"] == two[k]["dict_bytes"]]
print(f"{len(concordant)}/{len(two)} admissions have the same (token-row, dictionary-byte) counts")
systematic = [r for r in rows(here.parent / "f2c/geometry.csv")
              if r["target"] == "1" and r["stream"] in direct]
for col, expected in [("token_rows", "second_rows"), ("root_runs", "second_runs"),
                      ("words", "second_words"), ("dict_bytes", "second_dict_bytes")]:
    assert all(direct[r["stream"]][col] == r[expected] for r in systematic)
print(f"{len(systematic)} systematic admissions match token rows, token runs, distinct words and dictionary bytes")
phases = rows(here / "phase-cpu.csv")
keys = ["prep_mark_ns", "discover_ns", "merge_ns", "residual_output_ns", "total_ns"]
for blocks in [["1"], sorted(set(r["stream"] for r in phases))]:
    print(f"Streams {','.join(blocks)}")
    for arm in ("direct", "two", "lazy"):
        selected = [r for r in phases if r["arm"] == arm]
        sums = [sum(stats.median(int(r[k]) for r in selected if r["stream"] == stream)
                    for stream in blocks) for k in keys]
        print(arm, " ".join(f"{name}={v/1e6:.4f} ms" for name, v in zip(keys, sums)))
