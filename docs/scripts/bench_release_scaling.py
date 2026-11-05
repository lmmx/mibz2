#!/usr/bin/env python3
"""Turn the workstation runs (bench/release-scaling/*/, bench/release_scaling.py record) into the
History page's workstation view and tables.

    python3 docs/scripts/bench_release_scaling.py

Writes, for each run, newest first:

- docs/site/assets/release-scaling.json, read by the History page's hero (bench.js), which
  offers a Workstation view only when a run is recorded: the machine, and per corpus each
  arm's throughput (MB/s of output, the median of its rounds) at each thread count, and its
  paired wall change against the reference arm;
- docs/generated/release-scaling.md, included by the History page: per corpus, each arm's
  median wall and, against the reference arm, the geometric mean of the rounds' paired wall
  ratios with its 95% interval. Arms whose outputs differ from the reference's are named.
  With no runs, it says none is recorded yet.
"""

import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUNS = ROOT / "bench" / "release-scaling"
OUT = ROOT / "docs" / "generated" / "release-scaling.md"
ASSET = ROOT / "docs" / "site" / "assets" / "release-scaling.json"

# Student's t, two-sided 95%, by degrees of freedom (then the normal's 1.96)
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201,
        2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074,
        2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042]


def t975(df: int) -> float:
    return T975[df - 1] if 1 <= df <= len(T975) else 1.96


def ratio(logs: list[float]) -> tuple[float, float, float] | None:
    """The geometric mean of paired ratios and its 95% interval, from their logs."""
    if len(logs) < 2:
        return None
    m = statistics.fmean(logs)
    hw = t975(len(logs) - 1) * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - hw), math.exp(m + hw)


def fmt_change(r: tuple[float, float, float] | None) -> str:
    if r is None:
        return "–"
    est, lo, hi = ((x - 1) * 100 for x in r)
    mark = " **faster**" if hi < 0 else " **slower**" if lo > 0 else ""
    sign = lambda x: f"{x:+.1f}".replace("-", "\u2212")  # noqa: E731
    return f"{sign(est)}% ({sign(lo)}, {sign(hi)}){mark}"


def run_section(d: Path) -> str:
    meta = json.loads((d / "meta.json").read_text())
    rows = [json.loads(line) for line in (d / "runs.jsonl").read_text().splitlines() if line.strip()]
    rows = [r for r in rows if not r.get("err")]
    ref, labels = meta["ref"], meta["arms"]
    arms = [ref] + sorted(a for a in labels if a != ref)
    name = lambda a: labels.get(a, a)  # noqa: E731
    cores = f"{meta['cores']} cores, {meta['cpus']} threads" if meta.get("cores") else f"{meta.get('cpus')} CPUs"
    out = [f"### {meta['date']}: {meta.get('cpu') or meta.get('machine') or d.name}", ""]
    out.append(
        f"{cores} · {meta['rounds']} paired rounds · the reference arm is **{name(ref)}** · "
        f"[the records](https://github.com/lmmx/mibz2/tree/master/bench/release-scaling/{d.name})"
    )
    if meta.get("note"):
        out += ["", meta["note"]]
    by = defaultdict(lambda: defaultdict(list))  # (corpus, threads) -> arm -> [rows]
    for r in rows:
        by[(r["corpus"], r["threads"])][r["arm"]].append(r)
    for corpus in sorted({c for c, _ in by}):
        threads = sorted(t for c, t in by if c == corpus)
        out += ["", f"**`{corpus}`**", ""]
        head = "| threads | " + " | ".join(f"{name(a)} wall (median)" for a in arms) + " | " + " | ".join(
            f"{name(a)} against {name(ref)}" for a in arms[1:]) + " |"
        out += [head, "|" + "---:|" * (1 + 2 * len(arms) - 1)]
        mismatched = set()
        for t in threads:
            cell = by[(corpus, t)]
            walls = [f"{statistics.median(r['wall'] for r in cell[a]):.3f} s" if cell.get(a) else "–" for a in arms]
            refs = {r["round"]: r for r in cell.get(ref, [])}
            changes = []
            for a in arms[1:]:
                logs = [math.log(r["wall"] / refs[r["round"]]["wall"]) for r in cell.get(a, []) if r["round"] in refs]
                changes.append(fmt_change(ratio(logs)))
                if {r["hash"] for r in cell.get(a, [])} - {r["hash"] for r in refs.values()}:
                    mismatched.add(name(a))
            out.append(f"| {t} | " + " | ".join(walls) + " | " + " | ".join(changes) + " |")
        out += ["", "Every arm's output was the same bytes as the reference's." if not mismatched
                else f"**Different output from the reference:** {', '.join(sorted(mismatched))}."]
    return "\n".join(out)


def run_data(d: Path) -> dict:
    """One run for the hero: the machine, and per corpus each arm's MB/s and change."""
    meta = json.loads((d / "meta.json").read_text())
    rows = [json.loads(line) for line in (d / "runs.jsonl").read_text().splitlines() if line.strip()]
    rows = [r for r in rows if not r.get("err")]
    ref, labels = meta["ref"], meta["arms"]
    arms = [ref] + sorted(a for a in labels if a != ref)
    by = defaultdict(lambda: defaultdict(list))
    for r in rows:
        by[(r["corpus"], r["threads"])][r["arm"]].append(r)
    corpora = {}
    for corpus in sorted({c for c, _ in by}):
        threads = sorted(t for c, t in by if c == corpus)
        mbps, change, same = {}, {}, True
        for a in arms:
            mbps[labels[a]] = {}
            change[labels[a]] = {}
            for t in threads:
                cell = by[(corpus, t)]
                if not cell.get(a):
                    continue
                mbps[labels[a]][t] = round(statistics.median(r["out"] / r["wall"] / 1e6 for r in cell[a]), 1)
                refs = {r["round"]: r for r in cell.get(ref, [])}
                if a != ref:
                    logs = [math.log(r["wall"] / refs[r["round"]]["wall"]) for r in cell[a] if r["round"] in refs]
                    rr = ratio(logs)
                    if rr:
                        change[labels[a]][t] = [round((x - 1) * 100, 2) for x in rr]
                    if {r["hash"] for r in cell[a]} - {r["hash"] for r in refs.values()}:
                        same = False
        corpora[corpus] = {"threads": threads, "mbps": mbps, "change": change, "same_bytes": same}
    return {
        "id": d.name, "date": meta["date"], "cpu": meta.get("cpu") or meta.get("machine") or d.name,
        "cores": meta.get("cores"), "cpus": meta.get("cpus"), "rounds": meta["rounds"],
        "ref": labels[ref], "arms": [labels[a] for a in arms], "note": meta.get("note"),
        "corpora": corpora,
    }


def main() -> None:
    dirs = sorted((p for p in RUNS.glob("*/") if (p / "meta.json").exists()), reverse=True) if RUNS.exists() else []
    OUT.parent.mkdir(parents=True, exist_ok=True)
    ASSET.write_text(json.dumps({"runs": [run_data(d) for d in dirs]}, separators=(",", ":"), allow_nan=False))
    if not dirs:
        OUT.write_text("No workstation run is recorded yet.\n")
    else:
        OUT.write_text("\n\n".join(run_section(d) for d in dirs) + "\n")
    print(f"{OUT.relative_to(ROOT)}, {ASSET.relative_to(ROOT)}: {len(dirs)} workstation runs")


if __name__ == "__main__":
    main()
