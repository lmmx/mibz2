"""bench/clamp.py's reading on synthetic runs: each verdict where the rule puts it, and a run
whose arms' outputs differ, or whose A/A control fails, is not read.

    python3 bench/test_clamp.py
"""

import contextlib
import io
import json
import random
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import clamp  # noqa: E402

THREADS = [8, 9, 10, 12, 16, 20]
CORPORA = ["wikidata", "wikidata-x8"]


def grid(gain=1.0, aa=1.0, seed=5, noise=0.004, differ=False):
    """10 rounds; mibz2-clamp's wall is mibz2's times `gain` above 10 threads and `aa` at
    10 and below"""
    rng = random.Random(seed)
    rows = []
    for rnd in range(10):
        for c in CORPORA:
            for t in THREADS:
                base = (0.11 if c == "wikidata" else 0.85) * (1 + 0.01 * (t > 10))
                for arm in ("mibz2", "mibz2-clamp"):
                    f = 1.0 if arm == "mibz2" else (gain if t > 10 else aa)
                    out = 1000 + (1 if differ and arm == "mibz2-clamp" else 0)
                    rows.append({
                        "round": rnd, "corpus": c, "threads": t, "arm": arm,
                        "wall": base * f * (1 + rng.gauss(0, noise)),
                        "rss_kb": 190 * 1024, "out": out, "hash": f"{out:016x}", "err": "",
                    })
    return rows


def run(rows):
    with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
        f.write("\n".join(json.dumps(r) for r in rows) + "\n")
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        status = clamp.main(["clamp.py", f.name])
    Path(f.name).unlink()
    return status, buf.getvalue()


def main():
    status, out = run(grid(gain=0.94))
    assert status == 0 and "verdict: ADOPT" in out, out
    status, out = run(grid(gain=1.0))
    assert status == 0 and "verdict: KEEP" in out, out
    # faster on one corpus only: keep
    rows = [dict(r, wall=r["wall"] * (0.94 if r["corpus"] == "wikidata" and r["arm"] == "mibz2-clamp" and r["threads"] > 10 else 1)) for r in grid()]
    status, out = run(rows)
    assert "verdict: KEEP" in out, out
    # the A/A control off: not read
    status, out = run(grid(gain=0.94, aa=0.95))
    assert status == 1 and "NOT READ" in out, out
    # outputs differ: not read
    status, out = run(grid(gain=0.94, differ=True))
    assert status == 1 and "NOT READ" in out and "verdict" not in out, out
    # the 8-against-10 table is there for both corpora
    status, out = run(grid(gain=0.94))
    assert out.count("   8 / 10") == 2 and out.count("   9 / 10") == 2, out
    print("test_clamp: ok")


if __name__ == "__main__":
    main()
