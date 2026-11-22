"""Repeat the primary W4 whole-corpus comparison with more passes per round."""
import gzip
import os
from pathlib import Path
import subprocess
import sys

binary, corpus, six, dest = map(lambda p: Path(p).resolve(), sys.argv[1:5])
dest.mkdir(parents=True, exist_ok=True)
for round_id in range(12):
    env = dict(os.environ, MODE="confirm", ROUND=str(round_id), ADMIT_BZ2=str(corpus),
               MIBZ2_ADMITTED_BWT=str(six))
    log = subprocess.check_output(["taskset", "-c", sys.argv[5] if len(sys.argv) > 5 else "0-3",
                                   binary, "open::trial::measure", "--exact", "--ignored",
                                   "--nocapture", "--test-threads=1"], text=True, env=env)
    rows = [line for line in log.replace("test open::trial::measure ... ", "").splitlines()
            if line.startswith("ROW ")]
    assert len(rows) == (431 + 2) * 3
    with gzip.open(dest / f"{round_id:02d}-four.gz", "wt") as f:
        f.write("\n".join(rows) + "\n")
    print(round_id, flush=True)
