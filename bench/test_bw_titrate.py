"""bench/bw_titrate.py: admission is exact, and the reading follows the registered thresholds."""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent
ROUNDS, THREADS, LEVELS, HOGS = 3, [1, 10], [0, 50, 100], 2
SIZE = 1_000_000


def grid(keep10=0.5, keep1=1.0, hog_gbs=12.0, mutate=None):
    """A complete grid: each cell 1 s long, the plan keeping `keep10` of its rate at the top."""
    rows = []
    for i in range(ROUNDS):
        for t in [0, *THREADS]:
            for lv in LEVELS:
                keep = {0: 1.0, 1: keep1, 10: keep10}[t]
                rate = 1.0 - (1.0 - keep) * lv / 100  # decodes a second, against level 0's one
                wall = 10**9
                r = {
                    "round": i,
                    "threads": t,
                    "level": lv,
                    "hogs": HOGS,
                    "wall_ns": wall,
                    "bytes": 0 if t == 0 else SIZE,
                    "hog_bytes": int(hog_gbs * 1e9 * lv / 100),
                }
                if t > 0:
                    r["wall_ns"] = int(wall / rate)
                    r["hog_bytes"] = int(hog_gbs * r["wall_ns"] * lv / 100)
                rows.append(r)
    if mutate:
        mutate(rows)
    return "".join(json.dumps(r) + "\n" for r in rows)


def run(tmp_path, text):
    p = tmp_path / "runs.jsonl"
    p.write_text(text)
    out = tmp_path / "report.json"
    r = subprocess.run(
        [
            sys.executable,
            str(HERE / "bw_titrate.py"),
            str(p),
            str(ROUNDS),
            "1,10",
            "0,50,100",
            str(HOGS),
            str(out),
        ],
        capture_output=True,
        text=True,
    )
    return r, (json.loads(out.read_text()) if out.exists() else None)


def reading(tmp_path, **kw):
    r, v = run(tmp_path, grid(**kw))
    assert r.returncode == 0, r.stderr
    return v["reading"].split(":")[0]


def test_readings(tmp_path):
    assert reading(tmp_path, keep10=0.5) == "BOUND"
    assert reading(tmp_path, keep10=0.80) == "BOUND"
    assert reading(tmp_path, keep10=0.95) == "NOT BOUND"
    assert reading(tmp_path, keep10=0.86) == "MIXED"
    assert reading(tmp_path, keep10=0.5, keep1=0.85) == "CONFOUNDED"
    assert reading(tmp_path, keep10=0.5, hog_gbs=5.0) == "UNDERPOWERED"


def test_figures(tmp_path):
    r, v = run(tmp_path, grid(keep10=0.5))
    assert r.returncode == 0, r.stderr
    assert abs(v["plan_ratio"] - 0.5) < 1e-6 and abs(v["control_ratio"] - 1.0) < 1e-6
    # 12 GB/s of hog traffic cost half of 1 MB/s: 24,000 bytes of traffic a decoded byte
    assert "| 10 | 100 | 12.00 | 0 | 0.500 | 24000.0 |" in r.stdout


def refused(tmp_path, mutate, word):
    r, _ = run(tmp_path, grid(mutate=mutate))
    assert r.returncode == 1 and word in r.stderr, (r.returncode, r.stderr)


def test_admission(tmp_path):
    refused(tmp_path, lambda rows: rows.pop(5), "missing")
    refused(tmp_path, lambda rows: rows.append(dict(rows[5])), "twice")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=-1), "no time")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=float("nan")), "whole number")
    refused(tmp_path, lambda rows: rows[5].update(wall_ns=1.5e9), "whole number")
    refused(tmp_path, lambda rows: rows[4].update(bytes=SIZE + 1), "not one size")
    refused(tmp_path, lambda rows: rows[4].update(bytes=0), "decoded bytes")
    refused(tmp_path, lambda rows: rows[0].update(bytes=7), "decoded bytes")
    refused(tmp_path, lambda rows: rows[3].update(hog_bytes=5), "hog bytes")
    refused(tmp_path, lambda rows: rows[4].update(hog_bytes=0), "hog bytes")
    refused(tmp_path, lambda rows: rows[5].update(hogs=3), "hogs")
    refused(tmp_path, lambda rows: rows[5].update(extra=1), "exactly the fields")
    refused(tmp_path, lambda rows: rows[5].pop("level"), "exactly the fields")
    refused(tmp_path, lambda rows: rows[5].update(round=True), "whole number")
