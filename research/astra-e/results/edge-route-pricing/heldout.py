"""Recreate a frozen six-run route forecast and compare it with the accepted raw i9 samples."""

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[4]
STUDY = Path("research/astra-e/results/edge-route-pricing")
HISTORICAL = Path("research/astra-e/results/combined-v1-proxy/score")
FREEZE = "a6baa5ebd58c5aa5c67cd4e2af06847df09d58da"
BASE = "23378304c93e85b131afb85f5e86adf6bd71d099"
EVIDENCE = "683c22e98aa41e42f90209063aa3650b224c595f"
RUN = "run-20261119T022117Z"
ARCHIVE = Path("research/astra-e/workstation/evidence/edge-i9-683c22e.tar.gz")
SCORER = "75297a1d57b2bdf262c3a19f3064848dedf2c493"
EDGE = "badb2aec3624bd65606c429b2e6f79e9d9d0050f"
COMBINED = "8c9640d8ab186f81a0500f48e606e0c21c2a6669"
ENTRY = "research/astra-e/edge-budget/candidate.rs"
ARMS = ["lazy", "opend", "predecessor", "candidate"]


def source(rev, path):
    return subprocess.check_output(["git", "show", f"{rev}:{path}"], cwd=ROOT)


def lines(raw):
    return [json.loads(row) for row in raw.splitlines()]


def frozen_points(tmp):
    recorded = json.loads(source(FREEZE, STUDY / "counterfactual.json"))
    manifest = json.loads(source(FREEZE, STUDY / "manifest.json"))
    assert manifest["upstream_scorer"] == SCORER and recorded["old_rows_reused_from"] == BASE
    assert recorded["carried_rows_in_each_counterfactual"] == 957 and recorded["fresh_rows_in_each"] == 3
    paths = [
        (FREEZE, STUDY / "read.py"),
        (FREEZE, STUDY / "runs.jsonl"),
        (BASE, HISTORICAL / "proxy.json"),
        (BASE, HISTORICAL / "proxy-runs.jsonl"),
        (SCORER, Path("bench/moonshot/moonshot.py")),
        (SCORER, Path("bench/moonshot/calibration.json")),
    ]
    for rev, path in paths:
        dst = tmp / path
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_bytes(source(rev, path))
    fresh = lines((tmp / STUDY / "runs.jsonl").read_bytes())
    assert {(x["block"], x["arm"], x["workers"], x["kind"]) for x in fresh} == {
        (165, arm, w, "base") for arm in ["edge", "combined"] for w in [7, 8, 10]
    }
    assert len(fresh) == 6 and len(lines((tmp / HISTORICAL / "proxy-runs.jsonl").read_bytes())) == 960
    for path in ("bench/moonshot/calibration.json", "bench/moonshot/moonshot.py"):
        assert hashlib.sha256((tmp / path).read_bytes()).hexdigest() == manifest["core_sha256"][path]
    rebuilt = json.loads(
        subprocess.check_output(
            [sys.executable, "-B", tmp / STUDY / "read.py", tmp, tmp / STUDY / "runs.jsonl"]
        )
    )
    assert rebuilt == recorded
    return rebuilt["cells"]


def paired(values):
    logs = [math.log(x) for x in values]
    m = statistics.mean(logs)
    # Fixed two-sided 95% t(15) half width for the sixteen preregistered pairs.
    width = 2.131449545559323 * statistics.stdev(logs) / math.sqrt(len(logs))
    assert len(logs) == 16
    return {"n": 16, "geomean": math.exp(m), "lo": math.exp(m - width), "hi": math.exp(m + width)}


def compare(record, replay):
    assert record.keys() == replay.keys()
    for k in record:
        assert math.isclose(record[k], replay[k], rel_tol=2e-12), (k, record[k], replay[k])


def measurement():
    archive_sha = hashlib.sha256((ROOT / ARCHIVE).read_bytes()).hexdigest()
    digest = source(EVIDENCE, f"astra-e/{RUN}.tar.gz.sha256").decode().split()[0]
    assert digest == archive_sha
    assert hashlib.sha256(source(EVIDENCE, f"astra-e/{RUN}.tar.gz")).hexdigest() == digest
    with tarfile.open(ROOT / ARCHIVE) as bundle:

        def raw(name):
            return bundle.extractfile(f"{RUN}/{name}").read()

        assert raw("mode.txt").strip() == b"measure"
        machine = [
            r.split(":", 1)[1].strip()
            for r in raw("lscpu.txt").decode().splitlines()
            if r.startswith("Model name:")
        ][0]
        assert "i9-10900K" in machine
        src = raw("matched-sources.txt").decode().splitlines()
        assert f"edge {EDGE} {ENTRY}" in src and any(
            line.startswith("measured combined " + COMBINED + " ") for line in src
        )
        candidate_sha = hashlib.sha256(source(EDGE, ENTRY)).hexdigest()
        assert any(line.startswith(candidate_sha) and line.endswith("/" + ENTRY) for line in src)
        refresh = json.loads(raw("scorer-refresh.json"))
        assert refresh["selected_path_revision"] == SCORER and refresh["proceed"]
        assert raw("scorer-blobs.txt") == raw("predecessor-scorer-blobs.txt")
        assert (
            raw("inputs.sha256")
            .splitlines()[1]
            .startswith(b"0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e")
        )
        for folder in ("candidate-gate", "predecessor-gate"):
            cases = lines(raw(folder + "/check.jsonl"))
            assert len(cases) == 339 and all(c["pass"] is True for c in cases[:-1])
            assert cases[-1] == {"arm": "candidate", "verdict": "PASS", "failures": 0}
        printed = json.loads(raw("candidate-gate/summary.json"))["scorer"]
        rows = lines(raw("runs.jsonl"))
        decision = json.loads(raw("decision.json"))
        run_roster = json.loads(raw("roster.json"))
    expected = []
    for rnd in range(16):
        for w in [7, 8, 9, 10]:
            offset = (rnd + w) % 4
            seq = ARMS[offset:] + ARMS[:offset]
            if ((rnd + w) // 4) % 2:
                seq.reverse()
            expected.extend([[rnd, w, a] for a in seq])
    assert expected == run_roster == [[r["round"], r["workers"], r["arm"]] for r in rows]
    name = {"lazy": "base", "opend": "base", "candidate": "head", "predecessor": "frozen predecessor"}
    assert all(
        r["out_bytes"] == 838800784
        and r["build"] == name[r["arm"]]
        and r["compiled_arm"] == ("candidate" if r["arm"] == "predecessor" else r["arm"])
        for r in rows
    )
    receipt = {
        "id": RUN,
        "evidence_commit": EVIDENCE,
        "artifact": str(ARCHIVE),
        "artifact_sha256": digest,
        "head": EDGE,
        "recipe_commit": src[0].split()[-1],
        "predecessor_commit": COMBINED,
        "selected": ENTRY,
        "candidate_source_sha256": candidate_sha,
        "scorer_path_revision": SCORER,
        "printed_scorer_version": f"scorer {printed['version']} at {printed['commit']}",
        "machine": machine,
        "i9_criterion_met": decision["i9_criterion_met"],
        "candidate_over_reference": {},
    }
    anchors = {}
    for w, cell in decision["workers"].items():
        times = {a: [r["wall_s"] for r in rows if r["arm"] == a and r["workers"] == int(w)] for a in ARMS}
        peaks = {
            a: [r["peak_rss_kb"] for r in rows if r["arm"] == a and r["workers"] == int(w)] for a in ARMS
        }
        anchors[w] = paired([p / b for p, b in zip(times["predecessor"], times["lazy"])])
        for arm, label in [
            ("lazy", "master LazyCells"),
            ("opend", "master opend"),
            ("predecessor", "frozen combined"),
        ]:
            data = cell[arm]
            compare(data["wall"], paired([p / b for p, b in zip(times["candidate"], times[arm])]))
            compare(data["rss"], paired([p / b for p, b in zip(peaks["candidate"], peaks[arm])]))
            assert data["median_peak_ratio"] == statistics.median(peaks["candidate"]) / statistics.median(
                peaks[arm]
            )
            assert (data["max_peak_kb"], data["ref_max_peak_kb"]) == (
                max(peaks["candidate"]),
                max(peaks[arm]),
            )
            receipt["candidate_over_reference"].setdefault(label, {})[w] = dict(
                data["wall"],
                **{k: data[k] for k in ("rss", "median_peak_ratio", "max_peak_kb", "ref_max_peak_kb")},
            )
    return receipt, anchors


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("out", type=Path, help="output directory for the CSV and scorer-compatible receipt")
    args = p.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    with TemporaryDirectory() as tmp:
        forecasts = frozen_points(Path(tmp))
    receipt, anchors = measurement()
    (args.out / "i9-receipt.json").write_text(json.dumps({"edge_i9_run": receipt}, indent=2) + "\n")
    table = receipt["candidate_over_reference"]
    with (args.out / "heldout-i9.csv").open("w", newline="") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(
            [
                "workers",
                "comparison",
                "conditional_feedback",
                "diagnostic_nominal",
                "observed_wall",
                "observed_lo",
                "observed_hi",
                "feedback_minus_observed",
                "log_feedback_over_observed",
                "feedback_relative_error_pct",
                "log_nominal_over_observed",
            ]
        )
        for w in ["7", "8", "9", "10"]:
            cfg = forecasts.get(w)
            arms = cfg["counterfactual_drivers"] if cfg else None
            edge_nom = arms["edge"]["at_nominal_latency"] if arms else None
            pred_nom = arms["combined"]["at_nominal_latency"] if arms else None
            for label, key, nom, obs in [
                ("edge/lazy", "counterfactual_edge_over_lazy", edge_nom, table["master LazyCells"][w]),
                (
                    "edge/combined",
                    "counterfactual_edge_over_rerun_combined",
                    edge_nom / pred_nom if cfg else None,
                    table["frozen combined"][w],
                ),
                ("combined/lazy", "counterfactual_rerun_combined_over_lazy", pred_nom, anchors[w]),
            ]:
                forecast = cfg[key] if cfg else None
                measured = obs["geomean"]
                errors = (
                    [
                        forecast - measured,
                        math.log(forecast / measured),
                        100 * (forecast / measured - 1),
                        math.log(nom / measured),
                    ]
                    if cfg
                    else [None] * 4
                )
                writer.writerow([w, label, forecast, nom, measured, obs["lo"], obs["hi"], *errors])


if __name__ == "__main__":
    main()
