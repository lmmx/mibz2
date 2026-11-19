"""Rehearse the registered CLI with synthetic data and validate real gate transcripts."""

from contextlib import chdir, redirect_stderr, redirect_stdout
import io
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

import assess
from refresh import build_incompatibility, refresh
import moonshot


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True, stderr=subprocess.PIPE).strip()


def commit(repo, message):
    git(repo, "add", "-A")
    git(repo, "-c", "user.name=Astra test", "-c", "user.email=test@example.invalid", "commit", "-qm", message)
    return git(repo, "rev-parse", "HEAD")


def test_refresh(tmp):
    remote, caller, detached = [tmp / name for name in ("upstream", "frozen", "measured")]
    remote.mkdir()
    git(remote, "init", "-q", "-b", "master")
    for path in ("Justfile", ".just/old.just", "bench/moonshot/old.rs"):
        (remote / path).parent.mkdir(parents=True, exist_ok=True)
        (remote / path).write_text("old\n")
    old = commit(remote, "upstream")
    git(tmp, "clone", "-q", str(remote), str(caller))
    git(caller, "switch", "-qc", "dedicated-recipe")
    (caller / ".just/registered.just").write_text("bench-astra-e:\n    true\n")
    (caller / "Justfile").write_text("import '.just/registered.just'\n")
    frozen = commit(caller, "registered wrapper")
    git(caller, "worktree", "add", "-q", "--detach", str(detached), frozen)
    pin = dict(last_working_path_revision=old)
    with chdir(detached):

        def probe(out, label):
            assert not Path(".just/registered.just").exists()
            active.append(label)
            return failures.get(label)

        def run(expected):
            active.clear()
            try:
                selected = refresh(pin, tmp, probe)
            except RuntimeError:
                assert expected is None
            else:
                assert selected == expected
            receipt = json.loads((tmp / "scorer-refresh.json").read_text())
            assert receipt["proceed"] == bool(expected)
            assert active == [a["build"] for a in receipt["attempts"]]
            assert all(a["failure"] == failures.get(a["build"]) for a in receipt["attempts"])
            return receipt

        failures, active = {}, []
        assert run("fresh")["selected_path_revision"] == old
        failures["fresh"] = {"stage": "build", "exit_code": 1}
        run(None)
        assert active == ["fresh"]
        (remote / "bench/moonshot/old.rs").unlink()
        (remote / "bench/moonshot/new.rs").write_text("new\n")
        new = commit(remote, "replacement scorer")
        failures.clear()
        assert run("fresh")["selected_path_revision"] == new
        assert Path("bench/moonshot/new.rs").is_file() and not Path("bench/moonshot/old.rs").exists()
        for stage in ("build", "interface"):
            failures["fresh"] = {"stage": stage, "exit_code": 1}
            assert run("last-working")["selected_path_revision"] == old
            assert Path("bench/moonshot/old.rs").is_file() and not Path("bench/moonshot/new.rs").exists()
        failures["last-working"] = {"stage": "build", "exit_code": 1}
        run(None)
    git(caller, "worktree", "remove", "--force", str(detached))
    assert git(caller, "status", "--short") == ""
    print(
        "Detached refresh passed current, upgraded, failed-current, fallback-build, fallback-CLI and both-failed cases."
    )


def test_transcripts(tmp):
    path = tmp / "fixture.log"
    path.write_text("failed to download from registry due to a network error")
    assert not build_incompatibility(path)
    path.write_text(json.dumps({"message": {"level": "error", "spans": [{"file_name": "src/main.rs"}]}}))
    assert build_incompatibility(path)
    path.write_text("failed to parse manifest at manifest.toml")
    assert build_incompatibility(path)
    wikidata = {"case": "wikidata fixture", "pass": True, "bytes": 209700196}
    end = {"arm": "opend", "verdict": "PASS", "failures": 0}

    def transcript(records):
        path.write_text("".join(json.dumps(r) + "\n" for r in records))

    transcript([wikidata, end])
    assert check_gate(path, "opend") == 1
    for records in (
        [wikidata],
        [end],
        [dict(wikidata, **{"pass": False}), end],
        [wikidata, dict(end, arm="lazy")],
    ):
        transcript(records)
        try:
            check_gate(path, "opend")
        except ValueError:
            continue
        raise AssertionError("accepted missing, failed or wrong-arm gate evidence")
    print(
        "Build diagnostics distinguished source/manifest rejection from network failure. Missing, failed and wrong-arm byte gates rejected."
    )


def expected_roster():
    arms = ["lazy", "opend", "open", "candidate"]
    rows = []
    for rnd in range(16):
        for w in (7, 8, 9, 10):
            offset = (rnd + w) % 4
            order = arms[offset:] + arms[:offset]
            if ((rnd + w) // 4) % 2:
                order.reverse()
            rows.extend((rnd, w, arm) for arm in order)
    return rows


def check_gate(path, arm):
    records = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if not records or records[-1] != {"arm": arm, "verdict": "PASS", "failures": 0}:
        raise ValueError(f"{arm} has no final passing verdict in {path}")
    cases = records[:-1]
    if not all(c.get("pass") is True and c.get("failure") is None for c in cases):
        raise ValueError(f"{arm} reported a failed correctness case in {path}")
    if not any("wikidata" in c["case"] and c["bytes"] == 209700196 for c in cases):
        raise ValueError(f"{arm} omitted the Wikidata byte gate in {path}")
    return len(cases)


def interface(tmp, target=None):
    calls = {}

    def synthetic(scorer, arm, workers, corpus, perf=None):
        key = (arm, workers)
        rnd = calls.get(key, 0)
        calls[key] = rnd + 1
        candidate = arm == "candidate"
        return dict(
            arm=arm,
            workers=workers,
            out_bytes=838800784,
            wall_s=0.73 + 0.001 * rnd if candidate else 1.0,
            peak_rss_kb=87 + rnd % 6 if candidate else 100,
            cpu_s=1.0,
        )

    corpus = tmp / "fixture"
    corpus.write_bytes(b"")
    target = target or tmp / "target"
    # The built binary reports the arms, candidate source path and worker quota.
    # In the unit suite only this discovery call is mocked.
    info = json.loads(subprocess.check_output([target / "release/moonshot", "info"], text=True))
    required = {"lazy", "opend", "open", "candidate"}
    assert required <= set(info["arms"]) and info["cpus"] > 0
    assert Path(info["candidate"]).resolve() == Path(os.environ["MOONSHOT_CANDIDATE"]).resolve()
    common = ["--target-dir", str(target), "--corpus", str(corpus)]
    gate = {"verdict": "PASS", "cases": 1, "failed": [], "exit": 0}
    score_out, sweep_out = tmp / "gate", tmp / "rotor"
    with (
        patch.object(moonshot.Scorer, "gate", return_value=gate),
        patch.object(moonshot.Scorer, "run_once", synthetic),
        patch.object(moonshot.time, "sleep", lambda _: None),
        redirect_stdout(io.StringIO()),
        redirect_stderr(io.StringIO()),
    ):
        assert (
            moonshot.main(
                [
                    "score",
                    info["candidate"],
                    *common,
                    "--no-build",
                    "--no-timing",
                    "--no-proxy",
                    "--out",
                    str(score_out),
                ]
            )
            == 0
        )
        assert (
            moonshot.main(
                [
                    "sweep",
                    *common,
                    "--out",
                    str(sweep_out),
                    "--arms",
                    "lazy,opend,open,candidate",
                    "--workers",
                    "7,8,9,10",
                    "--rounds",
                    "16",
                    "--settle",
                    "2",
                ]
            )
            == 0
        )
    summary = json.loads((score_out / "summary.json").read_text())
    assert summary["gate"]["verdict"] == "PASS" and not calls.get(("candidate", 1))
    assert "timing" not in summary and "proxy" not in summary
    rows = [json.loads(line) for line in (sweep_out / "runs.jsonl").read_text().splitlines()]
    assert [(r["round"], r["workers"], r["arm"]) for r in rows] == expected_roster()
    result = assess.assess(rows)
    assert result["i9_criterion_met"]
    for w in (7, 8, 9, 10):
        assert set(result["workers"][w]) == {"target_met", "lazy", "opend", "open"}
        for arm in ("lazy", "opend", "open"):
            cell = result["workers"][w][arm]
            for metric, ratios in [
                ("wall", [0.73 + 0.001 * r for r in range(16)]),
                ("rss", [(87 + r % 6) / 100 for r in range(16)]),
            ]:
                logs = [math.log(r) for r in ratios]
                avg = statistics.mean(logs)
                width = 2.13144954556 * statistics.stdev(logs) / math.sqrt(16)
                for field, exp in [("geomean", avg), ("lo", avg - width), ("hi", avg + width)]:
                    assert math.isclose(cell[metric][field], math.exp(exp), rel_tol=2e-10)
            assert cell["median_peak_ratio"] == statistics.median([87 + r % 6 for r in range(16)]) / 100
            assert cell["max_peak_kb"] == 92 and cell["ref_max_peak_kb"] == 100
    for change in ("missing", "order", "length", "zero_rss"):
        altered = [dict(r) for r in rows]
        if change == "missing":
            altered.pop()
        if change == "order":
            altered[0], altered[1] = altered[1], altered[0]
        if change == "length":
            altered[0]["out_bytes"] -= 1
        if change == "zero_rss":
            altered[0]["peak_rss_kb"] = 0
        try:
            assess.assess(altered)
        except ValueError:
            continue
        raise AssertionError(f"accepted {change}")
    # Each single losing worker rejects the intersection. The outlier-only
    # RSS case isolates the maximum rule from the geometric mean and median.
    for metric, value in [("wall_s", 0.81), ("peak_rss_kb", 101)]:
        for w in (7, 8, 9, 10):
            altered = [dict(r) for r in rows]
            for r in altered:
                if r["arm"] == "candidate" and r["workers"] == w:
                    if metric == "wall_s" or r["round"] == 0:
                        r[metric] = value
            assert not assess.assess(altered)["i9_criterion_met"]
    return {
        "info": info,
        "scorer": summary.get("scorer"),
        "rows": len(rows),
        "rule": "paired log t intervals, median peaks, maximum peaks, all four workers",
    }


if __name__ == "__main__":
    if sys.argv[1:2] == ["--gate"]:
        print(check_gate(Path(sys.argv[2]), sys.argv[3]))
    elif sys.argv[1:2] == ["--interface"]:
        with TemporaryDirectory() as tmp:
            result = interface(Path(tmp), Path(sys.argv[2]))
            Path(sys.argv[3]).write_text(json.dumps(result, indent=2) + "\n")
    else:
        with TemporaryDirectory() as d:
            tmp = Path(d)
            test_refresh(tmp)
            test_transcripts(tmp)
            candidate = tmp / "fake.rs"
            info = {
                "arms": ["lazy", "opend", "open", "candidate"],
                "cpus": 20,
                "candidate": str(candidate),
                "candidate_type": "fixture",
            }
            with (
                patch.dict(os.environ, MOONSHOT_CANDIDATE=str(candidate)),
                patch("subprocess.check_output", return_value=json.dumps(info)),
                patch.object(moonshot.Scorer, "info", return_value=info),
            ):
                assert interface(tmp)["rows"] == 256
            print(
                "Both scorer CLI commands, the fixed 256-sample schedule, confidence intervals and decision rule passed."
            )
