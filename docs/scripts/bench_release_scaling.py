#!/usr/bin/env python3
"""Turn the release-scaling runs (bench/release-scaling/*/, bench/release_scaling.py record) into the
History page's Release scaling view and tables.

    python3 docs/scripts/bench_release_scaling.py

Writes, for each run, newest first:

- docs/site/assets/release-scaling.json, read by the History page's hero (bench.js), which
  offers a Release scaling view only when a run is recorded: the machine, and per corpus each
  arm's throughput (MB/s of output, the median of its rounds) at each thread count, and its
  paired wall change against the reference arm;
- docs/generated/release-scaling.md, included by the History page: per corpus, each arm's
  median wall and, against the reference arm, the geometric mean of the rounds' paired wall
  ratios with its 95% interval. Arms whose outputs differ from the reference's are named.
  With no runs, it says none is recorded yet.

The asset's `lineage` section is every release against every other, inferred from the
pairwise runs without re-running old releases (lineage()): lbzip2 is the same in every run,
so it is a fixed anchor. Per thread count, each release's log wall time relative to lbzip2 is
the inverse-variance weighted mean of its paired log ratios over the runs it appears in, and
any two releases compare as the difference of their figures. The chain of each run's direct
pair is kept as a cross-check, and a release measured in two runs gives a closure residual
(its two log ratios against lbzip2) as a drift diagnostic.
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


def _version(v: str) -> tuple[int, ...] | None:
    parts = v.split(".")
    return tuple(int(x) for x in parts) if len(parts) == 3 and all(x.isdigit() for x in parts) else None


def _mean_var(logs: list[float]) -> tuple[float, float] | None:
    """A paired log ratio's mean over the rounds, and that mean's variance (var/rounds)."""
    if len(logs) < 2:
        return None
    return statistics.fmean(logs), statistics.variance(logs) / len(logs)


def lineage(dirs: list[Path]) -> list[dict]:
    """Every release against every other, from the pairwise runs, per machine and corpus.

    A run has three arms: the new release (the reference arm), the release before it, and an
    anchor that is not a release (lbzip2), the same in every run of a group. Per thread count
    and run, each release gives the mean over the rounds of its paired log(wall/anchor wall)
    and that mean's variance. A release's fitted figure x is the inverse-variance weighted
    mean of its observations (two for a release measured in two runs), with variance v, one
    over the summed weights. Two releases a and b compare as exp(x_a - x_b) with variance
    v_a + v_b: conservative where both come from one run, whose shared anchor rounds make
    their errors positively correlated. The interval uses Student's t on one run's rounds.

    The cross-check is the chain: per thread, the cumulative sum of each run's direct mean
    log(old wall/new wall) from the oldest release, with its summed variances. The drift
    diagnostic is the closure residual: a release measured in two runs, its log ratio
    against the anchor in the later run minus that in the earlier one, with its standard
    error. Runs on different machines, corpora or anchors are never combined.
    """
    groups = defaultdict(list)  # (cpu, corpus, anchor) -> [run]
    for d in sorted(dirs):
        meta = json.loads((d / "meta.json").read_text())
        labels, ref = meta["arms"], meta["ref"]
        new = labels[ref]
        olds = [a for a in labels if a != ref and _version(labels[a])]
        anchors = [a for a in labels if a != ref and not _version(labels[a])]
        if not _version(new) or len(olds) != 1 or len(anchors) != 1:
            continue
        old, anchor = olds[0], anchors[0]
        rows = [json.loads(line) for line in (d / "runs.jsonl").read_text().splitlines() if line.strip()]
        rows = [r for r in rows if not r.get("err")]
        cells = defaultdict(dict)  # (corpus, threads, round) -> arm -> row
        for r in rows:
            cells[(r["corpus"], r["threads"], r["round"])][r["arm"]] = r
        cpu = meta.get("cpu") or meta.get("machine") or d.name
        for corpus in sorted({k[0] for k in cells}):
            per_t = {}
            for t in sorted({k[1] for k in cells if k[0] == corpus}):
                rounds = [c for k, c in cells.items() if k[0] == corpus and k[1] == t and {ref, old, anchor} <= c.keys()]
                lw = lambda a, b: [math.log(c[a]["wall"] / c[b]["wall"]) for c in rounds]  # noqa: E731
                obs = {"new": _mean_var(lw(ref, anchor)), "old": _mean_var(lw(old, anchor)), "link": _mean_var(lw(old, ref))}
                if None in obs.values():
                    continue
                obs["anchor_mbps"] = math.exp(statistics.fmean(math.log(c[anchor]["out"] / c[anchor]["wall"] / 1e6) for c in rounds))
                per_t[t] = obs
            if per_t:
                groups[(cpu, corpus, labels[anchor])].append(
                    {"id": d.name, "date": meta["date"], "new": new, "old": labels[old], "rounds": meta["rounds"], "t": per_t})
    out = []
    for (cpu, corpus, anchor), runs in groups.items():
        releases = sorted({r["new"] for r in runs} | {r["old"] for r in runs}, key=_version)
        if len(releases) < 3:
            continue
        runs.sort(key=lambda r: (_version(r["new"]), r["date"]))
        newest = runs[-1]
        tc = t975(min(r["rounds"] for r in runs) - 1)
        threads = sorted({t for r in runs for t in r["t"]})
        per = {}
        for t in threads:
            obs = defaultdict(list)  # release -> [(run, mean, var)]
            for r in runs:
                if t in r["t"]:
                    obs[r["new"]].append((r["id"], *r["t"][t]["new"]))
                    obs[r["old"]].append((r["id"], *r["t"][t]["old"]))
            x, v = [], []
            for rel in releases:
                o = obs.get(rel)
                if not o:
                    x.append(None), v.append(None)
                    continue
                w = sum(1 / ov for _, _, ov in o)
                x.append(round(sum(m / ov for _, m, ov in o) / w, 6))
                v.append(round(1 / w, 9))
            # the chain from the oldest release, through each run's direct pair
            chain, cv, acc, accv = [0.0], [0.0], 0.0, 0.0
            for a, b in zip(releases, releases[1:]):
                link = next((r["t"][t]["link"] for r in runs if r["old"] == a and r["new"] == b and t in r["t"]), None)
                if link is None or acc is None:
                    acc = accv = None
                else:
                    acc, accv = acc + link[0], accv + link[1]
                chain.append(None if acc is None else round(acc, 6))
                cv.append(None if accv is None else round(accv, 9))
            closure = []
            for rel in releases:
                o = sorted(obs.get(rel, []), key=lambda ob: next(i for i, r in enumerate(runs) if r["id"] == ob[0]))
                if len(o) == 2:
                    d, se = o[1][1] - o[0][1], math.sqrt(o[0][2] + o[1][2])
                    closure.append({"release": rel, "runs": [o[0][0], o[1][0]], "d": round(d, 6), "se": round(se, 6)})
            per[str(t)] = {
                "x": x, "v": v, "n": [len(obs.get(rel, [])) for rel in releases],
                "chain": chain, "chain_v": cv, "closure": closure,
                "anchor_mbps": round(newest["t"][t]["anchor_mbps"], 1) if t in newest["t"] else None,
            }
        out.append({
            "cpu": cpu, "corpus": corpus, "anchor": anchor, "releases": releases, "threads": threads,
            "t975": tc, "newest_run": newest["id"],
            "runs": [{"id": r["id"], "date": r["date"], "new": r["new"], "old": r["old"]} for r in runs],
            "per_thread": per,
        })
    # the newest run's machine first
    out.sort(key=lambda g: max(r["date"] for r in g["runs"]), reverse=True)
    return out


def main() -> None:
    dirs = sorted((p for p in RUNS.glob("*/") if (p / "meta.json").exists()), reverse=True) if RUNS.exists() else []
    OUT.parent.mkdir(parents=True, exist_ok=True)
    ASSET.write_text(json.dumps({"runs": [run_data(d) for d in dirs], "lineage": lineage(dirs)}, separators=(",", ":"), allow_nan=False))
    if not dirs:
        OUT.write_text("No release-scaling run is recorded yet.\n")
    else:
        OUT.write_text("\n\n".join(run_section(d) for d in dirs) + "\n")
    print(f"{OUT.relative_to(ROOT)}, {ASSET.relative_to(ROOT)}: {len(dirs)} release-scaling runs")


if __name__ == "__main__":
    main()
