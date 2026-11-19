"""Registered five-arm i9 cap-counter rotor and matched decisions."""
import argparse
from collections import Counter
from contextlib import chdir, redirect_stderr
import io
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import assess
import refresh
import moonshot
from test_protocol import check_gate

ARMS = ["lazy", "opend", "edge", "scalar", "both"]
WORKERS = [7, 8, 9, 10]
ROUNDS = 40
SOURCES = {
    "edge": ("badb2aec3624bd65606c429b2e6f79e9d9d0050f", "research/astra-e/edge-budget/candidate.rs"),
    "scalar": ("d0a42980e896211906234c7679b38760ae44a46e", "research/astra-e/front-fusion/cap-batch/worker/candidate.rs"),
    "both": ("50b8ecb8946a498601e3fd894ed84a2687df6cc8", "research/astra-e/front-fusion/cap-counter/both.rs"),
}
PIN = "75297a1d57b2bdf262c3a19f3064848dedf2c493"
WIKI_BYTES = 209700196


def identity(label):
    return (label, "candidate") if label in SOURCES else ("edge", label)


def roster():
    return [(r, w, a) for r in range(ROUNDS) for w in WORKERS for a in moonshot.order(ARMS, r + w)]


def dump(path, content):
    path.write_text(json.dumps(content, indent=2, allow_nan=False) + "\n")


class Runner(moonshot.Scorer):
    def __init__(self, binaries):
        super().__init__(argparse.Namespace(workers=WORKERS, rounds=ROUNDS, settle=2, repeat=1, target_dir=None))
        self.paths = binaries

    def run_once(self, label, workers, corpus, perf=None):
        build, compiled = identity(label)
        self.bin = self.paths[build]
        row = super().run_once(compiled, workers, corpus, perf)
        if (row["arm"], row["workers"], row["out_bytes"]) != (compiled, workers, 4 * WIKI_BYTES):
            raise ValueError(f"unexpected decoder receipt {row}")
        return dict(row, arm=label, compiled_arm=compiled, build=build)


def decision(rows):
    if [(r["round"], r["workers"], r["arm"]) for r in rows] != roster():
        raise ValueError("incomplete, unexpected or out-of-order roster")
    if any((r["build"], r["compiled_arm"]) != identity(r["arm"]) for r in rows):
        raise ValueError("wrong binary or compiled arm")
    # Use the historical stretch and three Lazy memory tests with their original names.
    with patch.object(assess, "ROUNDS", ROUNDS):
        result = assess.assess([dict(r, arm="candidate" if r["arm"] == "both" else r["arm"]) for r in rows],
                               ARMS[:-1] + ["candidate"])
    times = {(r["round"], r["workers"], r["arm"]): r["wall_s"] for r in rows}
    direct = {}
    for ref in ("edge", "scalar"):
        ratios = [math.exp(statistics.mean(math.log(times[i,w,"both"]/times[i,w,ref]) for w in WORKERS))
                  for i in range(ROUNDS)]
        interval = moonshot.paired_ratio(ratios, [1.]*ROUNDS)
        guards = {w: result["workers"][w][ref]["wall"]["hi"] <= 1.02 for w in WORKERS}
        direct[ref] = dict(equal_weight_round_ratios=ratios, wall=interval, per_worker_guard=guards,
                           wall_screen_met=interval["hi"] < 1. and all(guards.values()))
    memory = {}
    for w in WORKERS:
        lazy = result["workers"][w]["lazy"]
        memory[w] = {"paired_upper": lazy["rss"]["hi"] <= 1,
                     "median": lazy["median_peak_ratio"] <= 1,
                     "maximum": lazy["max_peak_kb"] <= lazy["ref_max_peak_kb"]}
    result.update(direct=direct, peak_vs_lazy=memory,
                  joint_incremental_met=all(x["wall_screen_met"] for x in direct.values())
                  and all(all(x.values()) for x in memory.values()))
    return result


def select(out, trees):
    root = Path(os.environ["CARGO_TARGET_DIR"])
    for arm in SOURCES:
        (out / ("build-" + arm)).mkdir()

    def compile_set(directory, label):
        revision = refresh.git("rev-parse", "origin/master") if label == "fresh" else PIN
        for arm, (pin, entry) in SOURCES.items():
            with chdir(trees[arm]), patch.dict(os.environ, CARGO_TARGET_DIR=str(root / arm),
                                              MOONSHOT_CANDIDATE=str(trees[arm] / entry)):
                assert refresh.git("rev-parse", "HEAD") == pin
                refresh.project(revision)
                failure = refresh.compatibility(directory / ("build-" + arm), label)
                if failure:
                    return dict(failure, arm=arm)
        log = directory / (label + "-matched-interface.log")
        with log.open("w") as f:
            call = subprocess.run([sys.executable, str(HERE/"panel.py"), "probe"], stdout=f, stderr=subprocess.STDOUT)
        if call.returncode:
            return dict(stage="matched-interface", log=log.name, exit_code=call.returncode)
        return None
    refresh.refresh({"last_working_path_revision": PIN}, out, compile_set)


def dry(binaries, corpus, out):
    cpus = sorted(os.sched_getaffinity(0))
    target = "i9-10900K" in Path("/proc/cpuinfo").read_text()
    explicit, affinity = [], []
    for w in WORKERS:
        for a in ARMS:
            build, compiled = identity(a)
            row = json.loads(subprocess.check_output([binaries[build], "run", "--arm", compiled, "--workers", str(w), "--file", corpus]))
            assert (row["workers"], row["arm"], row["out_bytes"]) == (w, compiled, WIKI_BYTES)
            explicit.append(dict(label=a, build=build, compiled_arm=compiled, workers=w, slots=4*w, out_bytes=WIKI_BYTES))
        prefix = ["taskset", "-c", ",".join(map(str, cpus[:w]))]
        avail = {build: json.loads(subprocess.check_output([*prefix, binary, "info"]))["cpus"] for build,binary in binaries.items()}
        if len(cpus) < w or any(n != w for n in avail.values()):
            if target:
                raise RuntimeError(f"target dry cannot exercise W{w} on every binary: {avail}")
            continue
        for a in ARMS:
            build, compiled = identity(a)
            path = out / f"gate-{a}-w{w}.jsonl"
            with path.open("w") as f:
                subprocess.run([*prefix, binaries[build], "check", "--quick", "--arm", compiled, "--wikidata", corpus], stdout=f, check=True)
            assert check_gate(path, compiled) >= 98
        affinity.append(w)
    dump(out / "dry-coverage.json", dict(target_machine=target, explicit_workers=explicit, affinity_gate_worker_counts=affinity))


def probe():
    source = Path("bench/moonshot/src/main.rs").read_text()
    assert re.search(r"parallel_workers_with::<K>\(\s*std::io::Cursor::new\(data\),\s*workers,\s*(?:4\s*\*\s*workers|workers\s*\*\s*4),", source)
    paths = {a: Path(a) for a in SOURCES}
    identities = {(paths[b], c):a for a in ARMS for b,c in [identity(a)]}
    def synthetic(s, arm, w, corpus, perf=None):
        label = identities[s.bin, arm]
        wall = dict(lazy=1., opend=.94, edge=.82, scalar=.80, both=.75)[label]
        return dict(arm=arm, workers=w, out_bytes=4*WIKI_BYTES, wall_s=wall, cpu_s=1., peak_rss_kb=100 if label=="lazy" else 85)
    with TemporaryDirectory() as temp, patch.object(moonshot.Scorer, "run_once", synthetic), patch.object(moonshot.time, "sleep", lambda _:None), redirect_stderr(io.StringIO()):
        Runner(paths).timing(Path(temp)/"fixture", Path(temp), ARMS)
        rows = list(map(json.loads, (Path(temp)/"runs.jsonl").read_text().splitlines()))
    for w in WORKERS:
        positions = Counter((r["arm"], i%5) for i,r in enumerate(row for row in rows if row["workers"]==w))
        assert len(positions)==25 and set(positions.values())=={8}
    good = decision(rows)
    assert good["i9_criterion_met"] and good["joint_incremental_met"]
    for ref in ["scalar", "edge"]:
        # Favorable pooled contrast with a sufficiently bad single-worker result.
        change = [dict(r, wall_s=.70) if (r["arm"],r["workers"])==(ref,9) else r for r in rows]
        d = decision(change)
        assert d["direct"][ref]["wall"]["hi"]<1 and not d["direct"][ref]["per_worker_guard"][9]
        assert not d["joint_incremental_met"] and d["i9_criterion_met"]
        # A win over one direct reference cannot compensate for losing to the other.
        d = decision([dict(r,wall_s=.73) if r["arm"]==ref else r for r in rows])
        assert not d["joint_incremental_met"] and sum(c["wall_screen_met"] for c in d["direct"].values())==1
    d = decision([dict(r,peak_rss_kb=101) if (r["arm"],r["workers"],r["round"])==("both",7,0) else r for r in rows])
    assert d["peak_vs_lazy"][7]==dict(paired_upper=True,median=True,maximum=False) and not d["joint_incremental_met"] and not d["i9_criterion_met"]
    for invalid in [rows[:-1], [dict(rows[0], build="wrong"), *rows[1:]], [dict(rows[0], out_bytes=0), *rows[1:]]]:
        try:
            decision(invalid)
        except ValueError:
            continue
        raise AssertionError("accepted invalid receipt")
    print("800 ordered samples, five build/arm mappings, eight observations per slot per W, and all joint, one-worker and outlying peak rejection tests passed.")


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "probe":
        probe()
    elif mode == "plan":
        print(json.dumps(roster()))
    elif mode == "select":
        out, scratch = map(Path, sys.argv[2:])
        select(out, {a:scratch/a for a in SOURCES})
    elif mode in ("dry", "sweep"):
        targets, corpus, out = map(Path, sys.argv[2:])
        selected = json.loads((out/"scorer-refresh.json").read_text())["selected_build"]
        binaries = {a:targets/a/selected/"release/moonshot" for a in SOURCES}
        if mode=="dry":
            dry(binaries, corpus, out)
        else:
            Runner(binaries).timing(corpus, out, ARMS)
            dump(out/"decision.json", decision(list(map(json.loads, (out/"runs.jsonl").read_text().splitlines()))))
    else:
        raise SystemExit("expected probe, plan, select, dry or sweep")
