"""Four-arm edge-budget i9 follow-up using the direct two-binary CI runner."""

from contextlib import chdir
import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

import refresh

HERE = Path(__file__).resolve().parent
ARMS = ["lazy", "opend", "predecessor", "candidate"]
PREDECESSOR = "research/astra-e/narrow-short/ahead.rs"


def select(out, other):
    """Select one projection that compiles and rehearses both frozen sources."""
    pin = json.loads((HERE / "scorer-pin.json").read_text())
    baseline = out / "predecessor-build"
    baseline.mkdir()
    target = Path(os.environ["CARGO_TARGET_DIR"])

    def both(directory, label):
        failure = refresh.compatibility(directory, label)
        if failure:
            return failure
        revision = (
            refresh.git("rev-parse", "origin/master")
            if label == "fresh"
            else pin["last_working_path_revision"]
        )
        with (
            chdir(other),
            patch.dict(
                os.environ,
                CARGO_TARGET_DIR=str(target / "predecessor"),
                MOONSHOT_CANDIDATE=str(other / PREDECESSOR),
            ),
        ):
            refresh.project(revision)
            failure = refresh.compatibility(baseline, label)
        if failure:
            return dict(failure, log="predecessor-build/" + failure["log"])
        log = directory / f"{label}-matched-interface.log"
        with log.open("w") as f:
            proc = subprocess.run(
                [sys.executable, str(HERE / "edge.py"), "probe"], stdout=f, stderr=subprocess.STDOUT
            )
        if proc.returncode:
            return {"stage": "matched-interface", "exit_code": proc.returncode, "log": log.name}
        return None

    refresh.refresh(pin, out, both)


def paired_runner(edge, predecessor):
    import assess

    sys.path.insert(0, str(HERE.parent / "ci"))
    from paired import Paired

    scorer = Paired(predecessor, edge, assess.BYTES, predecessor)
    scorer.arms = ARMS
    scorer.a.workers = assess.WORKERS
    scorer.a.repeat = 1
    scorer.a.rounds = assess.ROUNDS
    return scorer


def decision(rows):
    import assess

    builds = {"lazy": "base", "opend": "base", "predecessor": "frozen predecessor", "candidate": "head"}
    for r in rows:
        raw = "candidate" if r["arm"] == "predecessor" else r["arm"]
        if r["compiled_arm"] != raw or r["build"] != builds[r["arm"]]:
            raise ValueError(f"binary or compiled arm mismatch: {r}")
    result = assess.assess(rows, ARMS)
    for cell in result["workers"].values():
        p = cell["predecessor"]
        cell["lower_predecessor_peak"] = (
            p["rss"]["hi"] < 1 and p["median_peak_ratio"] < 1 and p["max_peak_kb"] <= p["ref_max_peak_kb"]
        )
        cell["predecessor_wall"] = (
            "faster" if p["wall"]["hi"] < 1 else "slower" if p["wall"]["lo"] > 1 else "unresolved"
        )
    result["lower_predecessor_peak_at_every_w"] = all(
        cell["lower_predecessor_peak"] for cell in result["workers"].values()
    )
    return result


def read_rows(out):
    return [json.loads(line) for line in (out / "runs.jsonl").read_text().splitlines()]


def probe():
    import assess
    from test_protocol import expected_roster
    import moonshot

    edge, predecessor = Path("edge"), Path("combined")
    scorer = paired_runner(edge, predecessor)

    def synthetic(s, arm, w, corpus, perf=None):
        identity = s.bin, arm
        wall, rss = {
            (edge, "candidate"): (0.75, 80),
            (predecessor, "candidate"): (0.70, 90),
            (predecessor, "lazy"): (1, 100),
            (predecessor, "opend"): (0.95, 95),
        }[identity]
        return dict(arm=arm, workers=w, out_bytes=838800784, wall_s=wall, cpu_s=1.0, peak_rss_kb=rss)

    with (
        TemporaryDirectory() as tmp,
        patch.object(moonshot.Scorer, "run_once", synthetic),
        patch.object(moonshot.time, "sleep", lambda _: None),
    ):
        out = Path(tmp)
        scorer.timing(out / "unused", out, scorer.arms)
        rows = read_rows(out)
    expected = [(r, w, "predecessor" if a == "open" else a) for r, w, a in expected_roster()]
    assert [(r["round"], r["workers"], r["arm"]) for r in rows] == expected == assess.roster(ARMS)
    res = decision(rows)
    assert res["i9_criterion_met"] and res["lower_predecessor_peak_at_every_w"]
    assert all(v["predecessor_wall"] == "slower" for v in res["workers"].values())
    # A failure at one W must reject an all-worker claim.
    for r in rows:
        if r["arm"] == "candidate" and r["workers"] == 9:
            r.update(wall_s=0.81, peak_rss_kb=96)
    res = decision(rows)
    assert not res["i9_criterion_met"] and not res["lower_predecessor_peak_at_every_w"]
    print(
        "256 ordered samples exercised four true binary/arm identities; both all-worker claims rejected a losing W=9."
    )


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "select":
        select(Path(sys.argv[2]), Path(sys.argv[3]))
    elif mode == "probe":
        probe()
    elif mode == "plan":
        import assess

        print(json.dumps(assess.roster(ARMS)))
    elif mode == "sweep":
        edge, predecessor, corpus, out = map(Path, sys.argv[2:])
        scorer = paired_runner(edge, predecessor)
        scorer.timing(corpus, out, scorer.arms)
        (out / "decision.json").write_text(
            json.dumps(decision(read_rows(out)), indent=2, allow_nan=False) + "\n"
        )
    elif mode == "assess":
        print(json.dumps(decision(read_rows(Path(sys.argv[2]))), indent=2, allow_nan=False))
    else:
        raise SystemExit("expected select, probe, plan, sweep or assess")
