"""Condition v1's 400 paired W7 bootstrap replicates on multiplicity of the sampled dense block."""
import csv
from collections import defaultdict
import gzip
import hashlib
import json
from pathlib import Path
import random
import statistics
import sys
import tempfile

HERE = Path(__file__).resolve().parent
root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else HERE.parents[3]
meta = json.loads((HERE / "checkpoint/provenance.json").read_text())
for filename in ("bench/moonshot/moonshot.py", "bench/moonshot/calibration.json"):
    assert hashlib.sha256((root/filename).read_bytes()).hexdigest() == meta["source"][filename]
sys.path.insert(0, str(root / "bench/moonshot"))
import moonshot

cal = json.loads(moonshot.CALIBRATION.read_text())
model = cal["model"]
sample = moonshot.proxy_sample(cal, "48+16")
[dense] = json.loads((HERE/"checkpoint/route-shares.json").read_text())["sampled_dense"]
with tempfile.NamedTemporaryFile(mode="w+") as f:
    with gzip.open(HERE/"checkpoint/proxy-runs.jsonl.gz", "rt") as source:
        for line in source:
            if json.loads(line)["workers"] == 7:
                f.write(line)
    f.flush()
    runs, _ = moonshot.load_runs(Path(f.name))
for a in ("candidate", "lazy"):
    assert set(runs["7"][a]["base"]) == set(sample["full"] + sample["tiny"])
    assert set(runs["7"][a]["hi"]) == set(sample["slope"])
# W7 is the bandwidth anchor. Its fixed point only uses the W7 observations.
sols = moonshot.bootstrap_cells(runs, sample, model, reps=400)
ratios = [moonshot.ratio(s, "candidate", 7, 7) for s in sols]
published = json.loads((HERE/"checkpoint/partial-prediction.json").read_text())["model"]["7"]
assert moonshot.quantiles(ratios) == (published["lo"], published["hi"])
rng = random.Random(1)
groups = defaultdict(list)
for ratio in ratios:
    # Match the full-then-tiny draw order in bootstrap_cells.
    full = [rng.choice(sample["full"]) for _ in sample["full"]]
    tiny = [rng.choice(sample["tiny"]) for _ in sample["tiny"]]
    obs = runs["7"]["candidate"]["base"]
    total = moonshot.corpus_total(obs, full, tiny, sample["n_full"], sample["n_tiny"])
    groups[min(2,full.count(dense))].append((ratio, total["dep_dram_loads"] * 1e6 / total["out_bytes"]))
out = csv.writer(sys.stdout, lineterminator="\n")
out.writerow(("copies_of_ordinal_165", "draws", "model_ratio_median", "model_ratio_2.5pct", "model_ratio_97.5pct",
              "dep_DRAM_links_per_Mbyte_median", "dep_DRAM_links_per_Mbyte_2.5pct", "dep_DRAM_links_per_Mbyte_97.5pct"))
for k, records in sorted(groups.items()):
    a,b=map(list, zip(*records))
    out.writerow((str(k) if k<2 else "2 or more", len(a), statistics.median(a), *moonshot.quantiles(a),
                  statistics.median(b), *moonshot.quantiles(b)))
