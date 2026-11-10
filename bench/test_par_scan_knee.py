"""bench/par_scan_knee.py: admission is exact, and the reading follows the registered thresholds."""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent
ROUNDS, WORKERS = 3, [1, 4, 10]
SIZE = 500_000_000


def grid(top=1.5, mutate=None):
    """A complete grid: find 1 s a cell (500 MB/s), scan `top` times that at the top count."""
    rows = []
    for i in range(ROUNDS):
        for t in WORKERS:
            ratio = {1: 1.0, 4: 0.8, 10: top}[t]
            for arm in ("scan", "find"):
                rows.append(
                    {
                        "round": i,
                        "arm": arm,
                        "workers": t,
                        "wall_ns": round(ratio * 10**9) if arm == "scan" else 10**9,
                        "cpu_ns": 2 * 10**9 if arm == "scan" else 3 * 10**9,
                        "bytes": SIZE,
                    }
                )
    if mutate:
        mutate(rows)
    return "".join(json.dumps(r) + "\n" for r in rows)


def run(tmp_path, text, expected=SIZE):
    p = tmp_path / "runs.jsonl"
    p.write_text(text)
    out = tmp_path / "report.json"
    out.unlink(missing_ok=True)
    args = [str(p), str(ROUNDS), "1,4,10", str(out)]
    if expected is not None:
        args.append(str(expected))
    r = subprocess.run(
        [sys.executable, str(HERE / "par_scan_knee.py"), *args],
        capture_output=True,
        text=True,
    )
    return r, (json.loads(out.read_text()) if out.exists() else None)


def reading(tmp_path, **kw):
    r, v = run(tmp_path, grid(**kw))
    assert r.returncode == 0, r.stderr
    assert r.stdout.rstrip().endswith(f"**Reading: {v['reading']}**")
    return v["reading"].split(":")[0]


def test_readings(tmp_path):
    assert reading(tmp_path, top=1.5) == "FASTER"
    assert reading(tmp_path, top=1.15) == "FASTER"
    assert reading(tmp_path, top=1.149) == "MIXED"
    assert reading(tmp_path, top=1.1) == "MIXED"
    assert reading(tmp_path, top=1.051) == "MIXED"
    assert reading(tmp_path, top=1.05) == "NOT FASTER"
    assert reading(tmp_path, top=1.0) == "NOT FASTER"
    assert reading(tmp_path, top=0.7) == "NOT FASTER"


def test_figures(tmp_path):
    # the top count's scan cells: 1 s, 1.25 s and 1.5 s over the rounds, against find's 1 s
    def spread(rows):
        for r in rows:
            if r["workers"] == 10 and r["arm"] == "scan":
                r["wall_ns"] = [1_000_000_000, 1_250_000_000, 1_500_000_000][r["round"]]

    r, v = run(tmp_path, grid(mutate=spread))
    assert r.returncode == 0, r.stderr
    assert v["top_workers"] == 10 and v["ratio"] == 1.25 and v["ratio_range"] == [1.0, 1.5]
    assert v["reading"].startswith("FASTER: at 10 workers find ran at 1.250 of scan's rate")
    # scan's median 500 MB / 1.25 s = 400 MB/s, find's 500; CPU 2 s and 3 s
    assert "| 10 | 400 | 500 | 2.000 | 3.000 | 1.250 | 1.000–1.500 |" in r.stdout
    # at 4 workers scan took 0.8 s: 625 MB/s, and find ran at 0.8 of it
    assert "| 4 | 625 | 500 | 2.000 | 3.000 | 0.800 | 0.800–0.800 |" in r.stdout
    assert "| 1 | 500 | 500 | 2.000 | 3.000 | 1.000 | 1.000–1.000 |" in r.stdout


def refused(tmp_path, mutate, word):
    r, v = run(tmp_path, grid(mutate=mutate))
    assert r.returncode == 1 and word in r.stderr, (r.returncode, r.stderr)
    assert v is None and r.stdout == ""


def test_admission(tmp_path):
    refused(tmp_path, lambda rows: rows.pop(5), "missing")
    refused(tmp_path, lambda rows: rows.append(dict(rows[5])), "twice")
    refused(tmp_path, lambda rows: rows[5].update(workers=7), "not the registered grid")
    refused(tmp_path, lambda rows: rows[5].update(round=ROUNDS), "not the registered grid")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=-1), "no time")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=0), "no time")
    refused(tmp_path, lambda rows: rows[5].update(cpu_ns=0), "no CPU time")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=float("nan")), "whole number")
    refused(tmp_path, lambda rows: rows[5].update(cpu_ns=float("inf")), "whole number")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=1.5e9), "whole number")
    refused(tmp_path, lambda rows: rows[5].update(bytes="500000000"), "whole number")
    refused(tmp_path, lambda rows: rows[5].update(round=True), "whole number")
    refused(tmp_path, lambda rows: rows[4].update(bytes=SIZE + 1), "not one size")
    refused(tmp_path, lambda rows: rows[4].update(bytes=0), "decoded bytes")
    refused(tmp_path, lambda rows: rows[5].update(arm="lib"), "arm")
    refused(tmp_path, lambda rows: rows[5].update(extra=1), "exactly the fields")
    refused(tmp_path, lambda rows: rows[5].pop("workers"), "exactly the fields")
    r, _ = run(tmp_path, grid() + "not json\n")
    assert r.returncode == 1 and "not JSON" in r.stderr


def test_expected_size(tmp_path):
    # one size in every cell, but not the input's: uniformly wrong sizes are refused, a
    # reading of FASTER included
    for wrong in (1, SIZE - 1, SIZE + 1, 2 * SIZE):

        def uniform(rows, wrong=wrong):
            for r in rows:
                r["bytes"] = wrong

        r, v = run(tmp_path, grid(top=1.5, mutate=uniform))
        assert r.returncode == 1 and "not the expected" in r.stderr, (wrong, r.stderr)
        assert v is None and r.stdout == ""
    # the right size is read, and the report records what it was checked against
    r, v = run(tmp_path, grid(top=1.5))
    assert r.returncode == 0 and v["expected_bytes"] == SIZE
    # no expected size, or a nonsensical one: refused
    for bad in (None, 0, -5):
        r, v = run(tmp_path, grid(), expected=bad)
        assert r.returncode == 1 and v is None, (bad, r.stderr)
