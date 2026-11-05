#!/usr/bin/env python3
"""Each merged pull request's own paired benchmark, kept in the results store as a link of
the history's chain.

    python3 bench/links.py sync STORE [--master origin/master] [--repo lmmx/mibz2]
    python3 bench/links.py add STORE --commit SHA --pr N --result result.json --runs runs.jsonl

A pull request's benchmark (.github/workflows/bench.yml) runs its head against its base,
both builds on one runner, in alternating order, round by round: a paired comparison,
free of the runner's own speed. Master's records (`report.py record`) are absolute figures
on whichever runner a push drew, and runners differ by up to 2x, so the history chains the
pull requests' paired changes instead, from the artifact (`bench-pr-<head>`) of each merged
pull request's last run.

`sync` links every first-parent master commit that changed a benchmarked path, names its
pull request (`(#N)` at the end of its subject) and has no link yet, whose artifact is still
kept (90 days): idempotent, so a push links its own merge and a dispatch backfills the
rest. `add` links one commit from files at hand. Each link writes:

- `pr/<commit>.json` and `pr-raw/<commit>.jsonl`: the pull request run's result and runs,
  as they were;
- a line of `links.jsonl`: the merge commit and its parent, the pull request, its head and
  base, the run, its bench version and the major that last set the decoder's figures'
  meaning, whether its arms' order was balanced (bench 3.0.0 on), the runner, the corpora's
  contents, whether the base was the parent and the head the merge (on the benchmarked
  paths, a version-only bump aside), whether the decoder's code changed, and `change`: per
  corpus and thread count, the mean and standard deviation of the rounds' log ratios, head
  over base, of wall, CPU and peak RSS, with their count.

`sync` needs `gh` (GH_TOKEN) and a clone with every master commit.
"""

import argparse
import json
import math
import re
import statistics
import subprocess
import sys
import tempfile
import zipfile
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import report  # noqa: E402

# bench.yml's own path filter: what a pull request's benchmark runs on
PATHS = [
    "src",
    "Cargo.toml",
    "Cargo.lock",
    "examples/ab.rs",
    "examples/windows.rs",
    "examples/phases.rs",
    "tests/common",
    "bench",
    ".github/workflows/bench.yml",
]
HEAD, BASE = "mibz2", "mibz2@base/ab"
FIGURES = (("wall", "wall"), ("cpu", "cpu"), ("rss", "rss_kb"))


def git(*args: str, check: bool = True) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=check).stdout


def change(runs: list[dict]) -> dict:
    """Per corpus and thread count: the rounds' log ratios, head over base, of each figure,
    as [mean, sd] with their count n. A round counts where both arms ran without error."""
    by = defaultdict(dict)
    for r in runs:
        if not r.get("err"):
            by[(r["corpus"], r["threads"], r["round"])][r["arm"]] = r
    cells = defaultdict(lambda: defaultdict(list))
    for (corpus, t, _), arms in sorted(by.items()):
        if HEAD in arms and BASE in arms:
            cells[(corpus, t)]["pairs"].append((arms[HEAD], arms[BASE]))
    out = {}
    for (corpus, t), c in sorted(cells.items()):
        pairs = c["pairs"]
        cell = {"n": len(pairs)}
        for name, key in FIGURES:
            logs = [math.log(h[key] / b[key]) for h, b in pairs if h[key] > 0 and b[key] > 0]
            if len(logs) >= 2:
                cell[name] = [round(statistics.fmean(logs), 6), round(statistics.stdev(logs), 6)]
        out.setdefault(corpus, {})[str(t)] = cell
    return out


def version_only(diff: str) -> bool:
    """Whether a diff's changed lines are all `version = "..."` lines (a release bump)."""
    lines = [ln for ln in diff.splitlines() if ln[:1] in "+-" and not ln.startswith(("+++", "---"))]
    return all(re.match(r'^[+-]version = "[^"]*"$', ln) for ln in lines)


def same_code(a: str, b: str) -> bool:
    """Whether two commits agree on every benchmarked path, a version-only bump of a
    Cargo file aside."""
    for name in git("diff", "--name-only", "--no-renames", a, b, "--", *PATHS).split():
        if Path(name).name not in ("Cargo.toml", "Cargo.lock"):
            return False
        if not version_only(git("diff", "-U0", a, b, "--", name)):
            return False
    return True


def decoder_changed(a: str, b: str) -> bool:
    """Whether the decoder itself differs between two commits: its source, or its Cargo
    manifest or lock beyond a version-only bump (a dependency or a build profile)."""
    if git("diff", "--name-only", a, b, "--", "src").strip():
        return True
    return any(
        not version_only(git("diff", "-U0", a, b, "--", name))
        for name in git("diff", "--name-only", a, b, "--", "Cargo.toml", "Cargo.lock").split()
    )


def line(commit: str, pr: int, res: dict, runs: list[dict], source: str) -> dict:
    parent = git("rev-parse", f"{commit}^1").strip()
    head, base = res["head"], res["base"]
    known = all(git("cat-file", "-e", f"{s}^{{commit}}", check=False) == "" for s in (head, base))
    machine = res["machine"]
    return {
        "commit": commit,
        "parent": parent,
        "date": git("show", "-s", "--format=%cI", commit).strip(),
        "pr": pr,
        "pr_head": head,
        "pr_base": base,
        "run": res["run"],
        "bench": res["bench"],
        "meaning": report.meaning_major("decoder", res["bench"]),
        # (before 3.0.0 the head ran first in every round: bench/CHANGELOG.md)
        "balanced": int(res["bench"].split(".")[0]) >= 3,
        "cpu": machine["cpu"],
        "cores": machine.get("cores"),
        "rounds": res["rounds"],
        "threads": res.get("threads"),
        "parity": res.get("parity"),
        "corpora": {k: v.get("sha256", "")[:12] for k, v in sorted(res["corpora"].items())},
        "base_ok": same_code(base, parent) if known else None,
        "head_ok": same_code(head, commit) if known else None,
        "decoder": decoder_changed(base, head) if known else None,
        "change": change(runs),
        "source": source,
    }


def write(store: Path, commit: str, pr: int, result: str, runs_text: str, source: str) -> dict:
    res = json.loads(result)
    if res.get("mode") != "pr" or not res.get("base"):
        raise SystemExit(f"{commit[:7]}: not a paired result (mode {res.get('mode')})")
    runs = [json.loads(ln) for ln in runs_text.splitlines() if ln.strip()]
    entry = line(commit, pr, res, runs, source)
    for sub in ("pr", "pr-raw"):
        (store / sub).mkdir(parents=True, exist_ok=True)
    (store / "pr" / f"{commit}.json").write_text(result)
    (store / "pr-raw" / f"{commit}.jsonl").write_text(runs_text)
    with open(store / "links.jsonl", "a") as f:
        f.write(json.dumps(entry) + "\n")
    readme = store / "README.md"
    if not readme.exists() or readme.read_text() != report.STORE_README:
        readme.write_text(report.STORE_README)
    return entry


def gh(path: str) -> dict:
    out = subprocess.run(["gh", "api", path], capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def artifact(repo: str, head: str) -> tuple[dict, dict] | None:
    """The pull request run's kept artifact for `head`, the latest if several, and its run."""
    found = gh(f"repos/{repo}/actions/artifacts?name=bench-pr-{head}&per_page=100")
    kept = [a for a in found.get("artifacts", []) if not a.get("expired")]
    if not kept:
        return None
    a = max(kept, key=lambda a: a["created_at"])
    run = gh(f"repos/{repo}/actions/runs/{a['workflow_run']['id']}")
    return a, run


def sync(a: argparse.Namespace) -> None:
    store = Path(a.store)
    links = store / "links.jsonl"
    done = (
        {json.loads(x)["commit"] for x in links.read_text().splitlines()}
        if links.exists()
        else set()
    )
    log = git("log", "--first-parent", "--format=%H%x00%P%x00%s", a.master).splitlines()
    missing = []
    for commit, parents, subject in (x.split("\x00") for x in reversed(log)):
        m = re.search(r"\(#(\d+)\)\s*$", subject)
        if commit in done or not m or not parents:
            continue
        parent = parents.split()[0]
        if not git("diff", "--name-only", parent, commit, "--", *PATHS).strip():
            continue
        pr = int(m.group(1))
        head = gh(f"repos/{a.repo}/pulls/{pr}")["head"]["sha"]
        found = artifact(a.repo, head)
        if not found:
            print(f"::warning::{commit[:7]} (#{pr}): no kept artifact bench-pr-{head[:7]}")
            missing.append(commit)
            continue
        art, run = found
        if run.get("event") != "pull_request" or run.get("conclusion") != "success":
            print(
                f"::warning::{commit[:7]} (#{pr}): its run {run['id']} is {run.get('event')}, {run.get('conclusion')}"
            )
            missing.append(commit)
            continue
        git("fetch", "-q", "origin", f"+refs/pull/{pr}/head:refs/pr/{pr}", check=False)
        with tempfile.TemporaryDirectory() as tmp:
            zpath = Path(tmp) / "a.zip"
            with open(zpath, "wb") as f:
                subprocess.run(
                    ["gh", "api", f"repos/{a.repo}/actions/artifacts/{art['id']}/zip"],
                    stdout=f,
                    check=True,
                )
            with zipfile.ZipFile(zpath) as z:
                result = z.read("result.json").decode()
                runs_text = z.read("runs.jsonl").decode()
        res = json.loads(result)
        if res.get("head") != head:
            print(
                f"::warning::{commit[:7]} (#{pr}): artifact's head {res.get('head', '')[:7]} is not {head[:7]}"
            )
            missing.append(commit)
            continue
        # (on a push, only the pushed merge is linked as it lands; any earlier is a backfill)
        tip = git("rev-parse", a.master).strip()
        source = a.source if commit == tip else "backfill"
        e = write(store, commit, pr, result, runs_text, source)
        w4 = e["change"].get("wikidata", {}).get("4", {}).get("wall")
        print(
            f"{commit[:7]} #{pr} bench {e['bench']} {report_cpu(e['cpu'])}: "
            f"wikidata 4t wall {100 * (math.exp(w4[0]) - 1):+.1f}%"
            if w4
            else f"{commit[:7]} #{pr}: no wikidata 4t"
        )
    if missing:
        print(f"{len(missing)} commit(s) left unlinked")


def report_cpu(cpu: str) -> str:
    m = re.search(r"EPYC (\w+)", cpu) or re.search(
        r"(?i)xeon\(r\)(?: platinum)? (\w+(?:-\w+)?)", cpu
    )
    return m.group(0) if m else cpu


def add(a: argparse.Namespace) -> None:
    e = write(
        Path(a.store),
        a.commit,
        a.pr,
        Path(a.result).read_text(),
        Path(a.runs).read_text(),
        a.source,
    )
    print(json.dumps({k: v for k, v in e.items() if k != "change"}))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("sync")
    s.add_argument("store")
    s.add_argument("--master", default="origin/master")
    s.add_argument("--repo", default="lmmx/mibz2")
    s.add_argument("--source", default="record", choices=["record", "backfill"])
    d = sub.add_parser("add")
    d.add_argument("store")
    d.add_argument("--commit", required=True)
    d.add_argument("--pr", type=int, required=True)
    d.add_argument("--result", required=True)
    d.add_argument("--runs", required=True)
    d.add_argument("--source", default="backfill", choices=["record", "backfill"])
    a = ap.parse_args()
    sync(a) if a.cmd == "sync" else add(a)


if __name__ == "__main__":
    main()
