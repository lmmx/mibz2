"""Exercise a real detached Git refresh and the official timing rotor without measuring."""
import argparse
from contextlib import chdir
import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory

import assess
from refresh import refresh
from moonshot import Scorer


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True,
                                   stderr=subprocess.PIPE).strip()


def commit(repo, message):
    git(repo, "add", "-A")
    git(repo, "-c", "user.name=Astra test", "-c", "user.email=test@example.invalid",
        "commit", "-qm", message)
    return git(repo, "rev-parse", "HEAD")


def test_refresh(tmp):
    remote, caller, detached = [tmp / name for name in ("upstream", "frozen", "measured")]
    remote.mkdir()
    git(remote, "init", "-q", "-b", "master")
    for path in ("Justfile", ".just/old.just", "bench/moonshot/old.rs"):
        (remote / path).parent.mkdir(parents=True, exist_ok=True)
        (remote / path).write_text("old\n")
    old = commit(remote, "old upstream")
    git(tmp, "clone", "-q", str(remote), str(caller))
    git(caller, "switch", "-qc", "dedicated-recipe")
    (caller / ".just/registered.just").write_text("bench-astra-e:\n    true\n")
    (caller / "Justfile").write_text("import '.just/registered.just'\n")
    (caller / "failure.txt").write_text("fixture interface failure\n")
    frozen = commit(caller, "preregistered wrapper with test evidence")
    git(caller, "worktree", "add", "-q", "--detach", str(detached), frozen)
    pin = dict(last_working_path_revision=old, incompatible_successor=None)

    def project_expected():
        assert (detached / "Justfile").read_text() == "old\n"
        assert (detached / "bench/moonshot/old.rs").is_file()
        assert not (detached / "bench/moonshot/new.rs").exists()
        assert not (detached / ".just/registered.just").exists()
        assert (caller / ".just/registered.just").exists()
        assert git(caller, "status", "--short") == ""

    with chdir(detached):
        assert refresh(pin, tmp)
        project_expected()
        (remote / "bench/moonshot/old.rs").unlink()
        (remote / "bench/moonshot/new.rs").write_text("new\n")
        new = commit(remote, "replacement scorer")
        assert not refresh(pin, tmp)
        project_expected()
        receipt = json.loads((tmp / "scorer-refresh.json").read_text())
        assert receipt["fetched_path_revision"] == new and receipt["selected_path_revision"] == old
        pin["incompatible_successor"] = dict(path_revision=new, evidence="failure.txt")
        assert refresh(pin, tmp)
        project_expected()
        pin["last_working_path_revision"] = new
        pin["incompatible_successor"] = None
        assert refresh(pin, tmp)
        assert (detached / "bench/moonshot/new.rs").is_file()
        assert not (detached / "bench/moonshot/old.rs").exists()
    git(caller, "worktree", "remove", "--force", str(detached))
    assert git(caller, "status", "--short") == ""
    print("Four detached Git refresh cases passed; the registered caller remained clean.")


def test_roster(tmp):
    # Run the scorer's own timing method with synthetic observations. This checks
    # assess.roster against the actual arm ordering rather than duplicating its loop.
    scorer = Scorer(argparse.Namespace(rounds=16, workers=[7, 8, 9, 10], settle=0,
                                       repeat=1, target_dir=None))
    def synthetic(arm, w, corpus, perf):
        fast = arm == "candidate"
        return dict(arm=arm, workers=w, out_bytes=assess.BYTES, wall_s=0.75 if fast else 1.0,
                    cpu_s=1.0, peak_rss_kb=90 if fast else 100)
    scorer.run_once = synthetic
    scorer.timing(tmp / "unused", tmp, arms=assess.ARMS)
    rows = [json.loads(line) for line in (tmp / "runs.jsonl").read_text().splitlines()]
    assert len(rows) == 256 and assess.assess(rows)["i9_criterion_met"]
    for change in ("missing", "order", "length", "zero_rss"):
        altered = [dict(row) for row in rows]
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
        raise AssertionError(f"unexpected acceptance of {change}")
    for row in rows:
        if row["arm"] == "candidate" and row["workers"] == 9:
            row["wall_s"] = 0.81
    assert not assess.assess(rows)["i9_criterion_met"]
    print("The official 256-sample schedule, four malformed rosters and a losing W=9 cell passed.")


if __name__ == "__main__":
    with TemporaryDirectory() as tmp:
        test_refresh(Path(tmp))
        test_roster(Path(tmp))
