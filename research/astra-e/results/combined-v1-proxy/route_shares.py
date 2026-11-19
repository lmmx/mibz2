"""Charge v1's weighted proxy counters to the full blocks admitted by the wider edge policy."""
import argparse
from collections import Counter, defaultdict
import csv
import gzip
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CENSUS = ROOT / "docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv"


def load_runs(path):
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt") as f:
        return [json.loads(line) for line in f if line.endswith("\n")]


def population(ids, census, weights=None):
    groups = defaultdict(Counter)
    for b in ids:
        row = census[b]
        if row["run_front"] != "1":
            route = "column"
        elif 20 * int(row["records"]) < 3 * int(row["rows"]):
            route = "open"
        elif int(row["rows"]) >= 65536:
            route = "large_job_on_lazy"
        else:
            route = "tiny_job_on_lazy"
        groups[route]["blocks"] += 1
        groups[route]["bytes"] += int(row["output"]) * (weights[b] if weights else 1)
    return groups


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("runs", type=Path)
    p.add_argument("--scorer-tree", type=Path, default=ROOT)
    args = p.parse_args()
    provenance = json.loads((HERE / "checkpoint/provenance.json").read_text())
    for name in ("bench/moonshot/moonshot.py", "bench/moonshot/calibration.json"):
        if hashlib.sha256((args.scorer_tree / name).read_bytes()).hexdigest() != provenance["source"][name]:
            raise ValueError(f"this diagnostic requires the recorded v1 revision of {name}")
    sys.path.insert(0, str(args.scorer_tree / "bench/moonshot"))
    import moonshot

    spec = moonshot.proxy_sample(json.loads(moonshot.CALIBRATION.read_text()), "48+16")
    full, tiny = spec["full"], spec["tiny"]
    with CENSUS.open() as f:
        census = {int(row["ordinal"]):row for row in csv.DictReader(f)}
    weights = {**dict.fromkeys(full, spec["n_full"]/len(full)), **dict.fromkeys(tiny, spec["n_tiny"]/len(tiny))}
    dense = [b for b in census if int(census[b]["rows"]) >= 65536 and census[b]["run_front"] == "1"
             and 20*int(census[b]["records"]) >= 3*int(census[b]["rows"])]
    by = defaultdict(dict)
    for r in load_runs(args.runs):
        if r["kind"] == "base":
            data = r["counters"]
            if data["out_bytes"] != int(census[r["block"]]["output"]):
                raise ValueError("per-block decoded byte count disagrees with route census")
            by[r["workers"], r["arm"]][r["block"]] = data
    shares = {}
    for (w, arm), obs in sorted(by.items()):
        if set(obs) != set(weights):
            continue
        block_keys = ("cycles", "instructions", "dram_read_lines", "l2_transfers", "l3_transfers",
                      "dep_past_l2_loads", "dep_dram_loads", "out_bytes")
        totals = Counter()
        dense_totals = Counter()
        for b, data in obs.items():
            counters = {k: data[k] for k in block_keys}
            counters["dram_total_lines"] = sum(data[k] for k in ("dram_read_lines", "dram_write_lines", "dram_drain_lines"))
            charged = {k: v*weights[b] for k,v in counters.items()}
            totals.update(charged)
            if b in dense:
                dense_totals.update(charged)
        shares[f"{arm}_w{w}"] = {k: dense_totals[k]/v if v else None for k,v in totals.items()}
    print(json.dumps({"actual_corpus": population(list(census),census), "actual_dense": dense,
                      "stratified_weighted": population(full+tiny,census,weights),
                      "sampled_dense": sorted(set(full)&set(dense)),
                      "slope_dense": sorted(set(spec["slope"])&set(dense)), "dense_nominal_shares": shares}, indent=2))


if __name__ == "__main__":
    main()
