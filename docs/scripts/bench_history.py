#!/usr/bin/env python3
"""Turn the benchmark's results store into the site's benchmark data.

    python3 docs/scripts/bench_history.py [--store DIR | --ref origin/bench-results]

The store is the `bench-results` branch (bench/report.py record): master's commits only,
one result each. Writes:

- docs/site/assets/bench-history.json, read by the History page's charts
  (docs/site/assets/javascripts/bench.js): for each corpus and thread count, mibz2's
  throughput (MB/s of output), CPU time and peak RSS at each recorded commit, the mean of
  its rounds and the half-width of its 95% interval. A series holds only commits whose
  bench versions share the major that last set these figures' meaning
  (report.meaning_major("decoder", ...): bench/CHANGELOG.md), one runner CPU model and
  core count, and one set of corpora: each such class is a line of its own, never joined
  to another. Within a class, a corpus's points join only where its content (its sha256)
  is the same. Each commit keeps the bench version that measured it;
- docs/generated/alternatives.md, included by the Alternatives page: the latest
  master result, mibz2 against every other decoder it ran (bench/report.py's tables);
- docs/site/assets/alternatives.json, read by that page's chart (bench.js): the same
  result's figures, every decoder on every corpus and thread count, and whether each
  decodes blocks in parallel (`parallel`: a single-threaded decoder's figures at any
  thread count are its 1-thread runs).

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
import links  # noqa: E402  (bench/links.py: the pull requests' paired runs, as links)

REPO = "https://github.com/lmmx/mibz2"
ARM = "mibz2"


def git(*args: str) -> str:
    """A git command's output, run in the repository."""
    return subprocess.run(
        ["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True
    ).stdout


def line_class(res: dict) -> tuple[str, str]:
    """A result's line in the history, as a key and a label.

    The key is the bench major that last set the decoder's figures' meaning (not the
    result's own major: report.meaning_major), the runner's CPU model and core count, and
    the set of corpora. Results of different keys are never joined; each keeps the bench
    version that measured it.
    """
    major = report.meaning_major("decoder", res["bench"])
    machine = res["machine"]
    label = (
        f"bench {major}.0+ · {cpu_name(machine['cpu'])} · {machine['cores']} cores"
        f" · {len(res['corpora'])} corpora"
    )
    key = f"{major}|{machine['cpu']}|{machine['cores']}|{','.join(sorted(res['corpora']))}"
    return key, label


def line_key(res: dict, corpus: str) -> tuple[str, str | None]:
    """A series' line for one result: its class (line_class) and the corpus's content.

    The content is the corpus's recorded sha256: a corpus of one name and different bytes
    is a different workload, and its points are never joined (`text` is generated from
    this crate's own sources, so it changes with them).
    """
    sha = res["corpora"].get(corpus, {}).get("sha256")
    return line_class(res)[0], sha[:12] if sha else None


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


FIGS = ("wall", "cpu", "rss")


def half(cell: list[float], n: int) -> float:
    """A link's 95% half-width on the log scale: t(n - 1) sd / sqrt(n)."""
    return report.t95(n - 1) * cell[1] / math.sqrt(n) if n > 1 else math.inf


def link_entries(read, order: dict, meta: dict) -> list[dict]:
    """The store's links on master's first-parent line, in its order.

    A merge linked twice keeps its last link.
    """
    seen = {}
    for text in (read("links.jsonl") or "").splitlines():
        if text.strip():
            ln = json.loads(text)
            if ln["commit"] in order:
                seen[ln["commit"]] = ln
    return sorted(seen.values(), key=lambda ln: order[ln["commit"]])


def gaps(log: list[list[str]], linked: set[str], since: str | None) -> list[dict]:
    """The decoder changes from `since` on that have no link.

    These are first-parent commits that changed the decoder's source, or its Cargo files
    beyond a version bump. The chain cannot cross them.
    """
    if since is None:
        return []
    out, on = [], False
    for sha, date, subject in reversed(log):
        on = on or sha == since
        if not on or sha in linked:
            continue
        parent = git("rev-parse", "-q", "--verify", f"{sha}^1").strip()
        if parent and links.decoder_changed(parent, sha):
            pr = re.search(r"\(#(\d+)\)\s*$", subject)
            out.append(
                {
                    "sha": sha,
                    "date": date,
                    "subject": subject,
                    "pr": int(pr.group(1)) if pr else None,
                }
            )
    return out


def chains(entries: list[dict], start: int | None, gap_after: set[int]) -> dict:
    """Per corpus and thread count, each figure's change since the chain's start.

    At each link from `start`, it is the sum of the decoder links' mean log ratios, and
    the root of the sum of their squared 95% half-widths.
    - A link that left the decoder alone (an A/A pair: harness, docs or CI) adds nothing.
    - A gap before a link (a decoder change with no link) starts a new segment.
    """
    out = {}
    if start is None:
        return out
    keys = sorted({(c, t) for e in entries for c, ts in e["change"].items() for t in ts})
    for corpus, t in keys:
        series = {}
        for fig in FIGS:
            pts, L, V, seg = [None] * len(entries), 0.0, 0.0, 0
            for i in range(start, len(entries)):
                e = entries[i]
                if i in gap_after:
                    L, V, seg = 0.0, 0.0, seg + 1
                cell = e["change"].get(corpus, {}).get(t, {})
                if e.get("decoder"):
                    if fig not in cell:
                        pts[i] = None  # (not measured: the chain stops here)
                        L, V, seg = 0.0, 0.0, seg + 1
                        continue
                    L += cell[fig][0]
                    V += half(cell[fig], cell["n"]) ** 2
                pts[i] = [round(L, 6), round(math.sqrt(V), 6), seg]
            series[fig] = pts
        out[f"{corpus}|{t}"] = series
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
        key, cls = line_class(res)
        classes.setdefault(key, cls)
        for (corpus, t), p in points(res, raw).items():
            series[(corpus, t)][line_key(res, corpus)].append({"c": i, **p})
    keys = list(classes)
    meta_by_key = {}
    for rec, res, raw in ordered:
        key = line_class(res)[0]
        meta_by_key.setdefault(
            key,
            {
                "meaning": report.meaning_major("decoder", res["bench"]),
                "cpu": cpu_name(res["machine"]["cpu"]),
                "cores": res["machine"]["cores"],
            },
        )
    out_series = []
    for (corpus, t), by_line in sorted(
        series.items(), key=lambda kv: (kv[0][0] != "wikidata", kv[0][0], kv[0][1])
    ):
        out_series.append(
            {
                "corpus": corpus,
                "threads": t,
                "lines": [
                    {"class": keys.index(k), "content": sha, "points": pts}
                    for (k, sha), pts in by_line.items()
                ],
            }
        )
    # The pull requests' paired runs (bench/links.py), chained from the first whose arms
    # ran in balanced order (bench 3.0.0): before it the head ran first in every round
    entries = link_entries(read, order, meta)
    start = next((i for i, e in enumerate(entries) if e.get("balanced")), None)
    since = entries[start]["commit"] if start is not None else None
    gap_list = gaps(log, {e["commit"] for e in entries}, since)
    gap_after = set()
    for g in gap_list:  # (a gap breaks the chain before the next link)
        nxt = next((i for i, e in enumerate(entries) if order[e["commit"]] > order[g["sha"]]), None)
        if nxt is not None:
            gap_after.add(nxt)
    changes = {}
    for i, e in enumerate(entries):
        for corpus, ts in e["change"].items():
            for t, cell in ts.items():
                c = changes.setdefault(f"{corpus}|{t}", {f: [None] * len(entries) for f in FIGS})
                for f in FIGS:
                    if f in cell:
                        c[f][i] = [cell[f][0], round(half(cell[f], cell["n"]), 6), cell["n"]]
    newest = max((m["meaning"] for m in meta_by_key.values()), default=None)
    counts = defaultdict(int)
    for rec, res, raw in ordered:
        if report.meaning_major("decoder", res["bench"]) == newest:
            counts[cpu_name(res["machine"]["cpu"])] += 1
    data = {
        "generated": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "repo": REPO,
        "commits": commits,
        "classes": [classes[k] for k in keys],
        "classes_meta": [meta_by_key[k] for k in keys],
        # (the runner with the most records under the newest meaning: the default view)
        "default_cpu": max(counts, key=counts.get) if counts else None,
        "series": out_series,
        "links": [
            {
                "sha": e["commit"],
                "date": meta[e["commit"]][0],
                "subject": meta[e["commit"]][1],
                "pr": e["pr"],
                "run": e["run"],
                "cpu": cpu_name(e["cpu"]),
                "bench": e["bench"],
                "meaning": e["meaning"],
                "balanced": e["balanced"],
                "decoder": e.get("decoder"),
                "base_ok": e.get("base_ok"),
                "head_ok": e.get("head_ok"),
                "corpora": e.get("corpora", {}),
            }
            for e in entries
        ],
        "changes": changes,
        "chain": {"start": start, "series": chains(entries, start, gap_after)},
        "gaps": gap_list,
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
    classes = res.get("parallel") or {report.label(a): True for a in res["arms"]}
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
        # (each arm as the figures name it, in the order shown; a record before bench
        # 1.1 ran block-parallel arms only)
        "arms": list(classes),
        "parallel": classes,
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
