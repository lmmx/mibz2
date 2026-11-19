"""Audit the official evaluator's joins, without modifying a forecast or duplicating a receipt."""
from collections import Counter
from pathlib import Path
from zipfile import ZipFile
import argparse
import csv
import hashlib
import importlib.util
import json
import math
import re
import subprocess

ROOT = Path("research/astra-e")
EVALUATOR = Path("bench/moonshot/eval.py")
UPSTREAM = "75297a1d57b2bdf262c3a19f3064848dedf2c493"
DATA = "d7703b020c10bf41099501d40b07dae92db5aa9a"
I9_FORECASTS = {"v0": "combined-full-proxy", "v1": "combined-v1-proxy"}


def git(*args):
    return subprocess.check_output(["git", *args])


def sha(data):
    return hashlib.sha256(data).hexdigest()


def json_file(path, obj):
    path.write_text(json.dumps(obj, indent=2, allow_nan=False) + "\n")


def receipt(source):
    path, key = source.split(":")
    record = json.loads(Path(path).read_text())[key]
    digest = record.get("candidate_source_sha256")
    if digest is None:
        archive, = (ROOT / "ci/evidence").glob(f"*-{record['id']}.zip")
        assert sha(archive.read_bytes()) == record["artifact_sha256"]
        with ZipFile(archive) as z:
            line = re.search(rb"^([a-f0-9]{64})  (research/\S+)$", z.read("manifest.txt"), re.M)
        digest, entry = (v.decode() for v in line.groups())
        assert sha(git("show", f"{record['head']}:{entry}")) == digest
    return str(record["id"]), digest


def forecast_digest(row):
    if row.get("sha256"):
        return row["sha256"]
    if row.get("source"):
        m = Path(row["dir"]) / "manifest.txt"
        if m.exists():
            text = m.read_text()
            match = re.search(r"^([a-f0-9]{64})  (research/\S+)$", text, re.M)
            if match:
                digest, entry = match.groups()
                commit = text.splitlines()[0]
                assert sha(git("show", f"{commit}:{entry}")) == digest
                return digest
    return None


def proxy_replay(ev, rows):
    out = {}
    # Check the candidate and all source directories used by its deterministic generators.
    frozen_paths = ["src", "Cargo.toml", "docs/journal/2026-11-18-open-return"] + [
        str(ROOT / d) for d in ("narrow-short", "narrow-endpoint", "short-words", "lookahead")]
    for version, study in I9_FORECASTS.items():
        matches = sorted((r for r in rows if r["name"] == study and r.get("proxy") is not None
                          and r.get("indep") and r["indep"]["i9"]), key=lambda r: int(r["w"]))
        assert [r["w"] for r in matches] == ["7", "8", "10"]
        forecast_commit = (ROOT / "results" / study / "manifest.txt").read_text().splitlines()[0]
        measured_commit = matches[0]["indep"]["head"]
        assert not git("diff", "--name-only", forecast_commit, measured_commit, "--", *frozen_paths)
        points = []
        for row in matches:
            pred, measured = row["proxy"], row["indep"]["ratio"]
            rid, digest = receipt(row["indep"]["source"])
            assert forecast_digest(row) == digest
            points.append({"w": int(row["w"]), "proxy": pred, "i9_ratio": measured,
                           "signed_log_error": math.log(pred / measured)})
        out[version] = {"forecast_study": study, "forecast_build": forecast_commit,
                        "target_build": measured_commit, "run": rid, "entry_sha256": digest,
                        "unchanged_candidate_and_generator_paths": frozen_paths, "cells": points,
                        "accuracy": ev.accuracy(matches, "proxy")}
    return out


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("output_directory", type=Path)
    args = p.parse_args()
    if not __debug__:
        raise SystemExit("Audit requires Python assertions.")
    assert git("rev-parse", "HEAD").decode().strip() == DATA, "run in the frozen data checkout"
    assert EVALUATOR.read_bytes() == git("show", f"{UPSTREAM}:{EVALUATOR}")
    spec = importlib.util.spec_from_file_location("moonshot_eval", EVALUATOR)
    ev = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(ev)
    forecasts, results = ev.forecasts(ROOT), ev.results(ROOT)
    rows = sorted(ev.join(forecasts, results), key=lambda r: (r["name"], int(r["w"])))
    records = {r["source"]: receipt(r["source"]) for r in results}
    assert len({rid for rid, _ in records.values()}) == len(records)
    assert len({(records[r["source"]][0], r["w"]) for r in results}) == len(results)
    out = args.output_directory
    out.mkdir(parents=True, exist_ok=True)
    counts, evidence = Counter(), Counter()
    versions = {}
    with (out / "joins.csv").open("w") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(("forecast", "forecast_build", "forecast_artifact", "version", "w", "measurement",
                         "run", "measurement_head", "host", "official_join", "official_i9_calibration",
                         "forecast_entry_sha256", "measurement_entry_sha256", "audited_entry_match",
                         "local", "proxy", "independent", "independent_lo", "independent_hi"))
        for r in rows:
            res = r.get("indep")
            if not res:
                continue
            rid, independent_sha = records[res["source"]]
            source_sha = forecast_digest(r)
            same_sha = source_sha == independent_sha if source_sha else None
            how = r.get("how", "no forecast")
            counts[how] += 1
            if same_sha is not None:
                evidence[str(same_sha)] += 1
            writer.writerow((r["name"], r.get("commit"), r.get("source"), r["version"], r["w"], res["source"],
                             rid, res["head"], res["host"], how, bool(r.get("proxy_scored")), source_sha,
                             independent_sha, same_sha, r.get("local"), r.get("proxy"), res["ratio"],
                             res["lo"], res["hi"]))
    groups = sorted({r["version"] for r in rows if r.get("indep") and r["version"] != "-"})
    for version in groups:
        cells = [r for r in rows if r["version"] == version and r.get("indep")]
        cal = [r for r in cells if r.get("proxy_scored")]
        detail = {"official_i9_proxy": ev.accuracy(cal, "proxy")}
        for k in ("proxy", "local"):
            cross = [r for r in cells if r[k] is not None and not r.get("proxy_scored")]
            detail["official_other_" + k] = ev.accuracy(cross, k)
            unique = {(r.get("commit"), r["candidate"], r["w"], r[k], r["indep"]["source"]) for r in cross}
            detail["distinct_forecast_result_tuples_" + k] = len(unique)
        versions[version] = detail
    audit = {"snapshot": DATA, "evaluator_upstream": UPSTREAM, "evaluator_sha256": sha(EVALUATOR.read_bytes()),
             "forecasts": len(forecasts), "distinct_independent_runs": len(records), "independent_cells": len(results),
             "official_output_rows": len(rows), "rows_without_independent_measurement": sum(not r.get("indep") for r in rows),
             "rows_with_independent_measurement_by_official_join": dict(sorted(counts.items())),
             "audited_entry_digest_matches_on_joined_rows": dict(evidence), "official_accuracy": versions,
             "separate_source_audited_proxy_comparison": proxy_replay(ev, rows)}
    json_file(out / "audit.json", audit)


if __name__ == "__main__":
    main()
