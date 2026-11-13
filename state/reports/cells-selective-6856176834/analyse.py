"""Recompute the frozen comparison with the repository's standing paired intervals.

Usage: python3 analyse.py ARTIFACT_DIRECTORY REPOSITORY
No measurements are collected. Every row must be present and valid.
"""

import hashlib
import importlib.util
import json
import statistics
import sys
from pathlib import Path


def main():
    """Validate all 210 samples, then emit exact per-arm and paired figures."""
    data, repo = map(Path, sys.argv[1:])
    spec = importlib.util.spec_from_file_location("report", repo / "bench/report.py")
    report = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(report)
    pins = json.loads((repo / "bench/cells-selective/inputs.json").read_text())
    manifest = json.loads((data / "cells-inputs.json").read_text())
    assert manifest["inputs"] == pins
    assert manifest["generator"] == "57f51bbc0ca62ac3ee1cc148a1010842eea6222a"
    assert manifest["scale"] == 4
    rows = [json.loads(line) for line in (data / "cells.jsonl").read_text().splitlines()]
    assert len(rows) == 210
    arms = ["mibz2@bin/ab", "mibz2-cells@bin/original-cells-ab", "mibz2-cells"]
    indexed = {}
    for row in rows:
        assert row["termination"] == "success" and not row["err"]
        assert row["threads"] == 1 and row["out"] == pins[row["corpus"]][1]
        assert row["arm"] in arms and row["round"] in range(10)
        assert all(row[key] > 0 for key in ["wall", "cpu", "rss_kb"])
        key = row["corpus"], row["arm"], row["round"]
        assert key not in indexed
        indexed[key] = row
    assert len({r["cpus"] for r in rows}) == 1
    table = {}
    for name in pins:
        assert len({r["hash"] for r in rows if r["corpus"] == name}) == 1
        cell = {"arms": {}, "pairs": {}}
        for arm in arms:
            samples = [indexed[name, arm, i] for i in range(10)]
            cell["arms"][arm] = {
                key: {
                    "median": statistics.median(r[key] for r in samples),
                    "min": min(r[key] for r in samples),
                    "max": max(r[key] for r in samples),
                }
                for key in ["wall", "cpu", "rss_kb"]
            }
        for candidate, reference in [(arms[2], arms[1]), (arms[1], arms[0]), (arms[2], arms[0])]:
            cell["pairs"][candidate + " / " + reference] = {
                key: report.ratio_ci(
                    [
                        indexed[name, candidate, i][key] / indexed[name, reference, i][key]
                        for i in range(10)
                    ]
                )
                for key in ["wall", "cpu", "rss_kb"]
            }
        table[name] = cell
    result = {
        "samples": len(rows),
        "paired_method_sha256": hashlib.sha256((repo / "bench/report.py").read_bytes()).hexdigest(),
        "affinity": rows[0]["cpus"],
        "table": table,
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
