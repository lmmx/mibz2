#!/usr/bin/env python3
"""Turn the benchmark's results store into the site's benchmark data.

    python3 docs/scripts/bench_history.py [--store DIR | --ref origin/bench-results]

The store is the `bench-results` branch (bench/report.py record): master's commits only,
one result each. Writes:

- docs/site/assets/bench-history.json, read by the History page's charts
  (docs/site/assets/javascripts/bench.js): for each corpus and thread count, mibz2's
  throughput (MB/s of output), CPU time and peak RSS at each recorded commit, the mean of
  its rounds and the half-width of its 95% interval. A series holds only commits of one
  major bench version, one runner CPU model and core count, and one set of corpora: each
  such class is a line of its own, never joined to another;
- docs/generated/alternatives.md, included by the Alternatives page: the latest
  master result, mibz2 against pbz2 and lbzip2 (bench/report.py's tables);
- docs/site/assets/alternatives.json, read by that page's chart (bench.js): the same
  result's figures, every decoder on every corpus and thread count.

Commits are ordered by `git log --first-parent` of --master (default origin/master, else
HEAD). Without --store, the store is read from --ref; with neither, both files say that
nothing is recorded yet.
"""

import argparse
import json
import math
import re
import statistics
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "site" / "assets" / "bench-history.json"
ALTERNATIVES = ROOT / "docs" / "site" / "assets" / "alternatives.json"
GENERATED = ROOT / "docs" / "generated"
sys.path.insert(0, str(ROOT / "bench"))
import report  # noqa: E402  (bench/report.py: the tables, as the pull request comment has them)

REPO = "https://github.com/lmmx/mibz2"
ARM = "mibz2"


def git(*args: str) -> str:
    """A git command's output, run in the repository."""
    return subprocess.run(
        ["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True
    ).stdout


def reader(store: Path | None, ref: str):
    """A function reading a path of the store (None if it is not there)."""

    def read(path: str) -> str | None:
        if store:
            p = store / path
            return p.read_text() if p.exists() else None
        try:
            return git("show", f"{ref}:{path}")
        except subprocess.CalledProcessError:
            return None

    return read


def cpu_name(cpu: str) -> str:
    """A runner's CPU model, short: "EPYC 7763", "Xeon 8370C"."""
    if m := re.search(r"EPYC (\w+)", cpu):
        return f"EPYC {m.group(1)}"
    m = re.search(r"(?i)xeon\(r\)(?: platinum)? (\w+(?:-\w+)?)", cpu)
    if m and m.group(1).lower() != "processor":
        return f"Xeon {m.group(1).upper()}"
    return cpu.strip()


def stat(values: list[float]) -> list[float] | None:
    """[mean, half-width of its 95% interval] (rounded), or None."""
    v = [x for x in values if x is not None and math.isfinite(x)]
    if not v:
        return None
    m, h = report.mean_ci(v)
    return [float(f"{m:.4g}"), float(f"{h:.2g}")]


def points(result: dict, raw: str | None) -> dict:
    """(corpus, threads) -> mibz2's figures at this commit, from its rounds."""
    out = {}
    runs = [json.loads(l) for l in (raw or "").splitlines() if l.strip()]
    by = defaultdict(list)
    for r in runs:
        if r["arm"] == ARM and not r.get("err"):
            by[(r["corpus"], r["threads"])].append(r)
    for corpus, cells in result["figures"].items():
        for t, cell in cells.items():
            f = cell["arms"].get(ARM)
            if not f or not f.get("n"):
                continue
            rs = by.get((corpus, int(t)))
            if rs:
                p = {
                    "mb_s": stat([r["out"] / 1e6 / r["wall"] for r in rs if r["wall"]]),
                    "cpu_s": stat([r["cpu"] for r in rs]),
                    "wall_s": stat([r["wall"] for r in rs]),
                    "rss_mb": stat([r["rss_kb"] / 1024 for r in rs]),
                    "n": len(rs),
                }
            else:  # (no raw runs: the medians, without an interval)
                p = {
                    "mb_s": [f["mb_s"], 0],
                    "cpu_s": [f["cpu"], 0],
                    "wall_s": [f["wall"], 0],
                    "rss_mb": [f["rss_mb"], 0],
                    "n": f["n"],
                }
            p["same_bytes"] = cell["same_bytes"]
            out[(corpus, int(t))] = p
    return out


def build(read, master: str) -> tuple[dict, dict | None]:
    """The history's data, and the latest master result (None without one)."""
    log = [
        line.split("\x00")
        for line in git("log", "--first-parent", "--format=%H%x00%cI%x00%s", master).splitlines()
    ]
    order = {sha: i for i, (sha, _, _) in enumerate(reversed(log))}
    meta = {sha: (date, subject) for sha, date, subject in log}
    records = [json.loads(l) for l in (read("records.jsonl") or "").splitlines() if l.strip()]
    results = []
    for rec in records:
        if rec["commit"] not in order:  # (not on master's first-parent line)
            continue
        text = read(f"runs/{rec['commit']}.json")
        if text:
            results.append((rec, json.loads(text), read(f"raw/{rec['commit']}.jsonl")))
    # a commit recorded more than once: its latest record
    latest = {}
    for rec, res, raw in results:
        latest[rec["commit"]] = (rec, res, raw)
    ordered = sorted(latest.values(), key=lambda x: order[x[0]["commit"]])
    commits = []
    series = defaultdict(lambda: defaultdict(list))  # (corpus, threads) -> class -> points
    classes = {}
    for i, (rec, res, raw) in enumerate(ordered):
        sha = rec["commit"]
        date, subject = meta[sha]
        pr = re.search(r"\(#(\d+)\)\s*$", subject)
        commits.append(
            {
                "sha": sha,
                "date": date,
                "subject": subject,
                "pr": int(pr.group(1)) if pr else None,
                "run": res.get("run"),
                "bench": res["bench"],
            }
        )
        major = res["bench"].split(".")[0]
        cls = (
            f"bench {major}.x · {cpu_name(res['machine']['cpu'])} · {res['machine']['cores']} cores"
            f" · {len(res['corpora'])} corpora"
        )
        corpus_set = ",".join(sorted(res["corpora"]))
        key = f"{major}|{res['machine']['cpu']}|{res['machine']['cores']}|{corpus_set}"
        classes.setdefault(key, cls)
        for (corpus, t), p in points(res, raw).items():
            series[(corpus, t)][key].append({"c": i, **p})
    keys = list(classes)
    out_series = []
    for (corpus, t), by_class in sorted(
        series.items(), key=lambda kv: (kv[0][0] != "wikidata", kv[0][0], kv[0][1])
    ):
        out_series.append(
            {
                "corpus": corpus,
                "threads": t,
                "lines": [{"class": keys.index(k), "points": pts} for k, pts in by_class.items()],
            }
        )
    data = {
        "generated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "repo": REPO,
        "commits": commits,
        "classes": [classes[k] for k in keys],
        "series": out_series,
    }
    return data, (ordered[-1][1] if ordered else None)


def alternatives(res: dict | None) -> str:
    """The Alternatives page's tables, from the latest master result."""
    if res is None:
        return (
            '!!! note "No master commit is recorded yet"\n\n'
            "    The first record appears once the Benchmark workflow has run on a push to master.\n"
        )
    text = report.markdown(res)
    lines = text.splitlines()
    # (the comment's marker and title dropped; its sections one level up)
    body = "\n".join(lines[2:]).replace("\n### ", "\n## ")
    sha = res["head"]
    head = f"The latest record: [`{sha[:7]}`]({REPO}/commit/{sha}) ({res['date'][:10]}).\n"
    return head + body.replace(f"**`{sha[:7]}`, ", "**", 1).rstrip("\n") + "\n"


def alternatives_data(res: dict | None, data: dict) -> dict | None:
    """The Alternatives page's chart data: the latest master result's figures."""
    if res is None:
        return None
    pr = next((c.get("pr") for c in data["commits"] if c["sha"] == res["head"]), None)
    return {
        "repo": REPO,
        "head": res["head"],
        "pr": pr,
        "date": res["date"],
        "run": res.get("run"),
        "bench": res.get("bench"),
        "rounds": res.get("rounds"),
        "cpu": cpu_name(res["machine"]["cpu"]),
        "cores": res["machine"].get("cores"),
        "toolchain": res.get("toolchain", {}),
        "reference": res.get("reference", "mibz2"),
        "arms": res["arms"],
        "corpora": {
            k: {"bytes": v.get("bytes"), "out": v.get("out")} for k, v in res["corpora"].items()
        },
        "figures": res["figures"],
    }


def main() -> None:
    """Write the history's data and the alternatives' tables."""
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--store", type=Path, help="a checkout of the bench-results branch")
    ap.add_argument("--ref", default="origin/bench-results")
    ap.add_argument("--master")
    a = ap.parse_args()
    master = a.master
    if master is None:
        verify = ["git", "-C", str(ROOT), "rev-parse", "--verify", "-q", "origin/master"]
        master = (
            "origin/master"
            if subprocess.run(verify, capture_output=True).returncode == 0
            else "HEAD"
        )
    data, latest = build(reader(a.store, a.ref), master)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(data, separators=(",", ":"), allow_nan=False))
    GENERATED.mkdir(parents=True, exist_ok=True)
    (GENERATED / "alternatives.md").write_text(alternatives(latest))
    ALTERNATIVES.write_text(
        json.dumps(alternatives_data(latest, data), separators=(",", ":"), allow_nan=False)
    )
    print(
        f"{OUT.relative_to(ROOT)}: {len(data['commits'])} commits, {len(data['series'])} series, "
        f"{len(data['classes'])} classes; docs/generated/alternatives.md",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
