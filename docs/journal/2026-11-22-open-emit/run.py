"""Run from the harness checkout. Authenticate the corpus and six columns first."""
import argparse
import gzip
import hashlib
import json
import os
import subprocess
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("binary", type=Path)
p.add_argument("corpus", type=Path)
p.add_argument("six", type=Path)
p.add_argument("output", type=Path)
p.add_argument("--one-cpu", default="2")
p.add_argument("--four-cpus", default="0-3")
p.add_argument("--start-round", type=int, default=0)
args = p.parse_args()
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

assert digest(args.corpus) == "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
blocks = json.loads((args.six / "manifest.json").read_text())["blocks"]
assert len(blocks) == 6
for block in blocks:
    path = args.six / block["file"]
    assert path.stat().st_size == block["n"] and digest(path) == block["sha256"]
args.output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, ADMIT_BZ2=str(args.corpus.resolve()), MIBZ2_ADMITTED_BWT=str(args.six.resolve()))
cmd = [str(args.binary.resolve()), "open::trial::measure", "--exact", "--ignored", "--nocapture", "--test-threads=1"]
for round_id in range(args.start_round, 24):
    for mode, cpus in [("one", args.one_cpu), ("four", args.four_cpus)]:
        run = subprocess.run(["taskset", "-c", cpus, *cmd], check=True, capture_output=True,
                             env=dict(env, ROUND=str(round_id), MODE=mode), text=True)
        lines = run.stdout.replace("test open::trial::measure ... ", "").splitlines()
        rows = [line for line in lines if line.startswith("ROW ")]
        assert "ok" in run.stdout and len(rows) > 1300
        with gzip.open(args.output / f"{round_id:02d}-{mode}.gz", "wt") as out:
            out.write("\n".join(rows) + "\n")
        print(round_id, mode, len(rows), flush=True)
