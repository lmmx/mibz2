"""Team size against live tables: what holds mibz2 at its 8-thread ceiling on a many-core
machine (docs/journal/2026-11-06-team-size.md, registered before any run).

The cooperative decoder runs `teams` teams of `k` threads, each team inverting one block, so
`teams` T-vectors (3.6 MB each for a full block) are live at once. The policy caps `teams` by
the L3 (4 on the i9-10900K), so from 8 threads up it runs the same 4 x 2, whatever a caller
asks for. This separates the two things a thread count changes:

- `teams`, the live tables (the cache's load);
- `k`, the threads on each table (the work each table gets).

`ab run`'s arms `mibz2-coop2`, `-coop3`, `-coop4` at T threads run T // k teams of k; the
`mibz2` arm is the policy. Two steps, both from `just bench-team-size`:

    python3 bench/team_size.py counters AB FILE OUT.jsonl --cells ARM@T,.. [--reps 3] [--loops 5]
    python3 bench/team_size.py report RUNS.jsonl [--counters C.jsonl] [--ref mibz2@8] [--out STEM]

`counters` wraps `ab child ARM T FILE` (one decode) in `perf stat`, process-scoped, --loops
decodes a reading, --reps readings a cell, the cells' order rotated each rep: cycles,
instructions, L3 hits and misses of retired loads, and cycles stalled with an L3 miss
outstanding (Skylake-family names, the i9's; three general counters, so nothing is
multiplexed). `report` reads the timed runs (and the counters), and writes STEM.md and
STEM.json: per cell, its plan as the threads' names show it, throughput, its paired ratio
against the reference cell by round, each kind of thread's CPU, and the counters.
"""

import argparse
import json
import math
import os
import re
import statistics
import subprocess
import sys
from collections import defaultdict

# Student's t, 97.5% quantile, df 1-30
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip
EVENTS = [
    "cycles",
    "instructions",
    "mem_load_retired.l3_hit",
    "mem_load_retired.l3_miss",
    "cycle_activity.stalls_l3_miss",
]
TABLE_MB = 3.6  # a full block's T-vector: 900,000 rows x 4 bytes


def geo_ci(ratios):
    """The geometric mean of `ratios` and its 95% t-interval, (mean, low, high)"""
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    if len(logs) < 2:
        return math.exp(m), math.nan, math.nan
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def threads_of(thread_cpu):
    """A run's threads by kind, from ab's `thread_cpu` ("name=seconds ..."): the team
    leaders or workers (`mibz2-N`), the helpers (`mibz2-coop-*`), the scanner, the rest"""
    kinds = defaultdict(list)
    for part in thread_cpu.split():
        name, _, sec = part.rpartition("=")
        try:
            s = float(sec)
        except ValueError:
            continue
        if re.fullmatch(r"mibz2-\d+", name):
            kinds["leader"].append(s)
        elif name.startswith("mibz2-coop"):
            kinds["helper"].append(s)
        elif name == "mibz2-scan":
            kinds["scan"].append(s)
        else:
            kinds["other"].append(s)
    return kinds


def plan_of(kinds):
    """(teams, k) as the run's threads show it: a team is a leader and k - 1 helpers"""
    teams = len(kinds["leader"])
    if not teams:
        return 0, 1
    return teams, 1 + round(len(kinds["helper"]) / teams)


def cell(arm, threads):
    return f"{arm}@{threads}"


def counters(a):
    cells = [c.rsplit("@", 1) for c in a.cells.split(",")]
    with open(a.out, "w") as out:
        for rep in range(a.reps):
            order = cells[rep % len(cells) :] + cells[: rep % len(cells)]
            for arm, t in order:
                child = [os.path.abspath(a.ab), "child", arm, t, os.path.abspath(a.file)]
                chain = ["sh", "-c", 'for i in $(seq "$0"); do "$@" > /dev/null || exit 1; done']
                cmd = ["perf", "stat", "-x", ",", "-e", ",".join(EVENTS), "--", *chain]
                p = subprocess.run(cmd + [str(a.loops), *child], capture_output=True, text=True)
                if p.returncode != 0:
                    sys.exit(f"team_size counters: {arm}@{t} failed:\n{p.stderr[-2000:]}")
                row = {"rep": rep, "arm": arm, "threads": int(t), "loops": a.loops}
                for line in p.stderr.splitlines():
                    f = line.split(",")
                    if len(f) >= 3 and f[2] in EVENTS:
                        row[f[2]] = (
                            float(f[0])
                            if f[0] not in ("<not counted>", "<not supported>", "")
                            else None
                        )
                        if len(f) >= 5 and f[4] not in ("", "100.00"):
                            row.setdefault("counted_pct", {})[f[2]] = f[4]
                print(json.dumps(row), file=out, flush=True)
                print(f"counters rep {rep} {arm}@{t}: done", file=sys.stderr)


def report(a):
    runs = [json.loads(x) for x in open(a.runs) if x.strip()]
    runs = [r for r in runs if r.get("corpus") == a.corpus]
    bad = [r for r in runs if r.get("err")]
    hashes = {r["hash"] for r in runs if not r.get("err")}
    by = defaultdict(dict)  # cell -> round -> run
    for r in runs:
        if not r.get("err"):
            by[cell(r["arm"], r["threads"])][r["round"]] = r
    if a.ref not in by:
        sys.exit(f"team_size report: no runs of the reference {a.ref}")
    ref = by[a.ref]
    ctr = defaultdict(list)
    if a.counters:
        for x in open(a.counters):
            if x.strip():
                c = json.loads(x)
                ctr[cell(c["arm"], c["threads"])].append(c)

    rows = []
    for c, rounds in by.items():
        arm, t = c.rsplit("@", 1)
        xs = list(rounds.values())
        out = statistics.median(r["out"] for r in xs)
        wall = statistics.median(r["wall"] for r in xs)
        cpu = statistics.median(r["cpu"] for r in xs)
        kinds = [threads_of(r.get("thread_cpu", "")) for r in xs]
        teams, k = plan_of(kinds[0])
        plans = {plan_of(kd) for kd in kinds}
        paired = [ref[i]["wall"] / r["wall"] for i, r in rounds.items() if i in ref]
        g, lo, hi = geo_ci(paired) if paired else (math.nan,) * 3

        def per(kind):
            v = [statistics.fmean(kd[kind]) for kd in kinds if kd[kind]]
            return statistics.median(v) if v else None

        row = {
            "cell": c,
            "arm": arm,
            "threads": int(t),
            "teams": teams,
            "k": k,
            "plans_seen": sorted(plans),
            "tables_mb": round(teams * TABLE_MB, 1),
            "rounds": len(xs),
            "wall": wall,
            "cpu": cpu,
            "busy": cpu / wall,
            "mb_s": out / wall / 1e6,
            "vs_ref": [g, lo, hi],
            "leader_cpu": per("leader"),
            "helper_cpu": per("helper"),
            "scan_cpu": per("scan"),
            "rss_mb": statistics.median(r["rss_kb"] for r in xs) / 1024,
        }
        cs = ctr.get(c, [])
        if cs:

            def tot(e):
                v = [x.get(e) for x in cs]
                return None if any(z is None for z in v) else sum(v)

            n_out = out * sum(x["loops"] for x in cs)
            hit, miss = tot("mem_load_retired.l3_hit"), tot("mem_load_retired.l3_miss")
            cyc, ins, st = tot("cycles"), tot("instructions"), tot("cycle_activity.stalls_l3_miss")
            row["counters"] = {
                "readings": len(cs),
                "l3_hit_share": hit / (hit + miss)
                if hit is not None and miss is not None and hit + miss
                else None,
                "l3_miss_per_kb": miss / (n_out / 1024) if miss is not None else None,
                "cycles_per_byte": cyc / n_out if cyc is not None else None,
                "ipc": ins / cyc if ins is not None and cyc else None,
                "stall_l3_miss_share": st / cyc if st is not None and cyc else None,
                "multiplexed": [x.get("counted_pct") for x in cs if x.get("counted_pct")],
            }
        rows.append(row)
    rows.sort(key=lambda r: (r["teams"], r["k"], r["threads"], r["arm"]))

    def f(v, fmt):
        return "–" if v is None or (isinstance(v, float) and math.isnan(v)) else format(v, fmt)

    lines = [
        f"# Team size against live tables: {a.corpus}",
        "",
        f"Runs: {len(runs)}; errors: {len(bad)}; output hashes: {len(hashes)} "
        f"({'same bytes' if len(hashes) == 1 and not bad else 'NOT the same bytes'}). "
        f"Reference: `{a.ref}`. Paired ratio: the reference's wall over the cell's, by round "
        "(above 1 is faster), its geometric mean and 95% interval.",
        "",
        "| teams × k | arm @ threads | tables | MB/s | vs ref | busy cores | leader CPU | helper CPU | scanner CPU | peak RSS | L3 hit share | L3 misses /KB | cycles /B | IPC | L3-miss stall share |",
        "|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|",
    ]
    for r in rows:
        g, lo, hi = r["vs_ref"]
        c = r.get("counters", {})
        lines.append(
            f"| {r['teams']} × {r['k']} | `{r['cell']}` | {r['tables_mb']} MB | {r['mb_s']:,.0f} | "
            f"{f(g, '.3f')} ({f(lo, '.3f')}–{f(hi, '.3f')}) | {r['busy']:.1f} | "
            f"{f(r['leader_cpu'], '.3f')} | {f(r['helper_cpu'], '.3f')} | {f(r['scan_cpu'], '.3f')} | "
            f"{r['rss_mb']:.0f} MB | {f(c.get('l3_hit_share'), '.2f')} | {f(c.get('l3_miss_per_kb'), '.1f')} | "
            f"{f(c.get('cycles_per_byte'), '.2f')} | {f(c.get('ipc'), '.2f')} | {f(c.get('stall_l3_miss_share'), '.2f')} |"
        )
    mixed = [r["cell"] for r in rows if len(r["plans_seen"]) > 1]
    if mixed:
        lines += ["", f"Cells whose runs showed more than one plan: {', '.join(mixed)}."]
    md = "\n".join(lines) + "\n"
    stem = a.out or os.path.splitext(a.runs)[0] + "-report"
    open(stem + ".md", "w").write(md)
    json.dump(
        {
            "ref": a.ref,
            "corpus": a.corpus,
            "hashes": len(hashes),
            "errors": len(bad),
            "cells": rows,
        },
        open(stem + ".json", "w"),
        indent=1,
    )
    print(md)


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("counters")
    c.add_argument("ab")
    c.add_argument("file")
    c.add_argument("out")
    c.add_argument("--cells", required=True, help="ARM@T,... as ab's arms and thread counts")
    c.add_argument("--reps", type=int, default=3)
    c.add_argument("--loops", type=int, default=5, help="decodes in each reading")
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--counters")
    r.add_argument("--ref", default="mibz2@8")
    r.add_argument("--corpus", default="wikidata")
    r.add_argument("--out")
    a = ap.parse_args()
    counters(a) if a.cmd == "counters" else report(a)


if __name__ == "__main__":
    main()
