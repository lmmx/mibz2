"""Verify the original dry or timed archive against the registered two-binary experiment."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

import assess
import edge

READY = "e00b278656838aad9487ffa8fb7333eaa0c4705e"
EDGE = "badb2aec3624bd65606c429b2e6f79e9d9d0050f"
COMBINED = "8c9640d8ab186f81a0500f48e606e0c21c2a6669"
RELEASE = "530ed404a014a414ef0b62adfd947fa14d4533e6"
INPUTS = ["82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
          "0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e"]


def git(*args):
    return subprocess.check_output(["git", *args])


def verify(path):
    with tarfile.open(path, "r:gz") as archive:
        items = {}
        for member in archive:
            assert member.isfile()
            name = member.name.split("/", 1)[1]
            assert name not in items
            items[name] = archive.extractfile(member).read().decode()
    mode = items["mode.txt"].strip()
    assert mode in ("dry", "measure")
    assert "i9-10900K" in items["lscpu.txt"] and items["affinity.txt"].split()[-1] == "0-19"
    assert items["toolchain.txt"].startswith("rustc 1.99.1 ")
    revision = items["scorer-revision.txt"].strip()
    sources = items["matched-sources.txt"]
    assert f"recipe {READY}" in sources and f"edge {EDGE} " in sources and f"measured combined {COMBINED} " in sources
    for sha, entry in [(EDGE, "research/astra-e/edge-budget/candidate.rs"), (COMBINED, edge.PREDECESSOR)]:
        assert hashlib.sha256(git("show", f"{sha}:{entry}")).hexdigest() in sources
    commits = items["commits.txt"].split()
    assert len(commits) == 3 and commits[0] == EDGE and commits[2] == RELEASE
    refresh = json.loads(items["scorer-refresh.json"])
    assert refresh["selected_path_revision"] == revision and refresh["proceed"]
    assert refresh["selected_scorer_tree"] == git("rev-parse", revision + ":bench/moonshot").decode().strip()
    blobs = items["scorer-blobs.txt"].splitlines()
    assert blobs == items["predecessor-scorer-blobs.txt"].splitlines()
    for entry in blobs:
        meta, name = entry.split("\t")
        assert meta.split()[1] == git("rev-parse", f"{revision}:{name}").decode().strip()
    assert [s.split()[0] for s in items["inputs.sha256"].splitlines()] == INPUTS
    binaries = [s.split()[0] for s in items["binaries.sha256"].splitlines()]
    assert len(binaries) == len(set(binaries)) == 2
    report = next(s for s in items["scorer-report.txt"].splitlines() if s.startswith("scorer v"))
    assert report in items["predecessor-report.txt"].splitlines()

    def check(name, arm, number, text_bytes):
        *cases, verdict = map(json.loads, items[name].splitlines())
        assert verdict == dict(arm=arm, verdict="PASS", failures=0)
        assert len(cases) == number and all(r["pass"] and r["failure"] is None for r in cases)
        assert any(r["bytes"] == 209700196 for r in cases)
        assert [r["bytes"] for r in cases if r["case"] == "corpus text level 9"] == [text_bytes]
        return [(r["case"], r["bytes"] if r["case"] != "corpus text level 9" else None) for r in cases]

    assert check("candidate-gate/check.jsonl", "candidate", 338, 3755520) == check("predecessor-gate/check.jsonl", "candidate", 338, 3755882)
    reference = check("gate-lazy.jsonl", "lazy", 98, 3755882)
    assert reference == check("gate-opend.jsonl", "opend", 98, 3755882)
    assert json.loads(items["roster.json"]) == list(map(list, assess.roster(edge.ARMS)))
    result = dict(mode=mode, sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                  recorded_scorer=report, scorer_revision=revision, checked_projection_blobs=len(blobs),
                  binary_sha256=binaries)
    if mode == "dry":
        cover = json.loads(items["dry-coverage.json"])
        assert cover["all_target_worker_counts_tested"] and cover["target_machine"]
        assert cover["target_worker_counts_tested"] == assess.WORKERS
        for arm in edge.ARMS:
            for w in assess.WORKERS:
                assert reference == check(f"gate-{arm}-w{w}.jsonl", "candidate" if arm == "predecessor" else arm, 98,
                                          3755520 if arm == "candidate" else 3755882)
        assert "runs.jsonl" not in items
        result["worker_arm_gates"] = 16
    else:
        rows = list(map(json.loads, items["runs.jsonl"].splitlines()))
        decision = edge.decision(rows)
        assert json.loads(json.dumps(decision)) == json.loads(items["decision.json"])
        result.update(timed_rows=len(rows), decision=decision)
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    print(json.dumps(verify(parser.parse_args().archive), indent=2))
