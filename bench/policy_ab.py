"""The three-team policy's workstation A/B (docs/journal/2026-11-06-three-team-policy.md): the
head build's default plan against the base build's, at the same requested thread count, on
the bench's Wikidata sample, paired by round. Its verdict follows the rule Louis approved in
the Coordinator's session on 2026-11-06 (docs/journal/decisions.md), fixed before any run:

- **Validity**, for every cell: 20 rounds, each once, no failed run; one output hash and one
  length across every run; every run's plan, read from its thread names, the one the
  registration expects (below).
- **Controls**, 8 and 9 threads, where both builds pick four teams of two: as #220's control,
  flagged when a speed ratio's 95% interval lies wholly beyond ±2% (lower bound above 1.02,
  or upper below 0.98). A flagged run fails.
- **Primary**, 10 threads: speed ratio (base wall over head wall, geometric mean of the
  rounds' ratios) at least 1.03, its two-sided 95% lower bound above 1.00; the head's median
  CPU-seconds at most 1.5 times the base's and its median peak RSS at most 1.15 times.
- **No harm**, 11, 12, 16 and 20 threads: each head-over-base wall ratio's one-sided upper
  bound, Bonferroni over the four (each at 1 - 0.05/4), at most 1.02. Too wide is
  inconclusive, which fails.

A run passes only if every predicate holds. The policy is admitted when one run and its
predeclared confirmation both pass, with nothing pooled or switched.

    python3 bench/policy_ab.py report RUNS.jsonl --out DIR/policy-ab
"""

import argparse
import json
import math
import re
import statistics
import sys

ROUNDS = 20
# Student's t at 19 degrees of freedom (20 paired rounds): the two-sided 95% quantile, and the
# one-sided quantile for a familywise 0.05 over four cells (scipy.stats.t.ppf, rounded)
T_975 = 2.0930
T_9875 = 2.4334
CONTROLS = (8, 9)
PRIMARY = 10
NO_HARM = (11, 12, 16, 20)
THREADS = CONTROLS + (PRIMARY,) + NO_HARM
# the plans each build must run, as (leaders, helpers): four teams of two, three of three
OLD, NEW = (4, 4), (3, 6)
EXPECT = {t: {"base": OLD, "head": OLD if t in CONTROLS else NEW} for t in THREADS}
MIN_SPEED, NO_HARM_MAX, CONTROL_BAND = 1.03, 1.02, (0.98, 1.02)
RSS_MAX, CPU_MAX = 1.15, 1.5


def plan_of(thread_cpu):
    """(leaders, helpers) from ab's `thread_cpu` ("name=seconds ..."): team leaders or
    workers are `mibz2-N`, helpers `mibz2-coop-*`"""
    names = [p.rpartition("=")[0] for p in thread_cpu.split()]
    return (
        sum(1 for n in names if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in names if n.startswith("mibz2-coop")),
    )


def build_of(arm):
    """ "head" for the head build's own `mibz2`, "base" for `mibz2@<base binary>`"""
    if arm == "mibz2":
        return "head"
    if arm.startswith("mibz2@"):
        return "base"
    return None


def log_stats(ratios):
    logs = [math.log(r) for r in ratios]
    return statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(len(logs))


def judge(rows):
    """The run's verdict: a dict of every cell's figures and every predicate"""
    cells, problems = {}, []
    by = {}
    for r in rows:
        b = build_of(r["arm"])
        if b is None or r["threads"] not in THREADS:
            continue
        by.setdefault((b, r["threads"]), []).append(r)
    outputs = {(r.get("hash"), r.get("out")) for r in rows if not r.get("err")}
    failed = [r for r in rows if r.get("err")]
    valid = not failed and len(outputs) == 1
    if failed:
        problems.append(f"{len(failed)} failed run(s)")
    if len(outputs) != 1:
        problems.append(f"{len(outputs)} distinct outputs")
    for t in THREADS:
        runs = {b: by.get((b, t), []) for b in ("base", "head")}
        rounds = {b: sorted(r["round"] for r in runs[b]) for b in runs}
        complete = all(rounds[b] == list(range(ROUNDS)) for b in runs)
        plans_ok = all(plan_of(r["thread_cpu"]) == EXPECT[t][b] for b in runs for r in runs[b])
        c = {"threads": t, "complete": complete, "plans": plans_ok}
        if not complete:
            problems.append(f"{t} threads: rounds incomplete or repeated")
        if not plans_ok:
            seen = sorted({(b, plan_of(r["thread_cpu"])) for b in runs for r in runs[b]})
            problems.append(f"{t} threads: unexpected plan {seen}")
        valid = valid and complete and plans_ok
        if complete:
            base = {r["round"]: r for r in runs["base"]}
            head = {r["round"]: r for r in runs["head"]}
            speed = [base[i]["wall"] / head[i]["wall"] for i in range(ROUNDS)]
            m, se = log_stats(speed)
            c.update(
                speed=math.exp(m),
                lo95=math.exp(m - T_975 * se),
                hi95=math.exp(m + T_975 * se),
                # the head's wall over the base's, its one-sided Bonferroni upper bound
                harm_upper=math.exp(-m + T_9875 * se),
                cpu=statistics.median(r["cpu"] for r in runs["head"])
                / statistics.median(r["cpu"] for r in runs["base"]),
                rss=statistics.median(r["rss_kb"] for r in runs["head"])
                / statistics.median(r["rss_kb"] for r in runs["base"]),
            )
        cells[t] = c

    def ok(t, test):
        return t in cells and "speed" in cells[t] and test(cells[t])

    controls = {
        t: ok(t, lambda c: c["lo95"] <= CONTROL_BAND[1] and c["hi95"] >= CONTROL_BAND[0])
        for t in CONTROLS
    }
    primary = {
        "speed": ok(PRIMARY, lambda c: c["speed"] >= MIN_SPEED),
        "lower bound": ok(PRIMARY, lambda c: c["lo95"] > 1.0),
        "cpu": ok(PRIMARY, lambda c: c["cpu"] <= CPU_MAX),
        "rss": ok(PRIMARY, lambda c: c["rss"] <= RSS_MAX),
    }
    no_harm = {t: ok(t, lambda c: c["harm_upper"] <= NO_HARM_MAX) for t in NO_HARM}
    passed = valid and all(controls.values()) and all(primary.values()) and all(no_harm.values())
    return {
        "valid": valid,
        "problems": problems,
        "cells": cells,
        "controls": controls,
        "primary": primary,
        "no_harm": no_harm,
        "pass": passed,
    }


def markdown(v):
    yes = lambda b: "**yes**" if b else "**no**"  # noqa: E731
    out = [
        "# The three-team policy: head default against base default",
        "",
        "Speed is the base build's wall over the head's, paired by round (above 1 is faster); "
        "CPU and RSS are the head's median over the base's.",
        "",
        "| threads | role | base plan → head plan | speed (95%) | head/base wall, upper bound | CPU | RSS |",
        "|--:|---|---|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        role = "control" if t in CONTROLS else "primary" if t == PRIMARY else "no harm"
        plans = "4×2 → " + ("4×2" if t in CONTROLS else "3×3")
        if "speed" not in c:
            out.append(f"| {t} | {role} | {plans} | – | – | – | – |")
            continue
        out.append(
            f"| {t} | {role} | {plans} | {c['speed']:.3f} ({c['lo95']:.3f}–{c['hi95']:.3f}) "
            f"| {c['harm_upper']:.3f} | {c['cpu']:.2f} | {c['rss']:.2f} |"
        )
    out += [
        "",
        "## The predicates",
        "",
        f"- Valid (rounds, outputs, plans): {yes(v['valid'])}"
        + (f" ({'; '.join(v['problems'])})" if v["problems"] else ""),
    ]
    for t, b in v["controls"].items():
        out.append(f"- Control at {t}: 95% interval not wholly beyond ±2%: {yes(b)}")
    for k, b in v["primary"].items():
        out.append(f"- Primary at 10, {k}: {yes(b)}")
    for t, b in v["no_harm"].items():
        out.append(f"- No harm at {t}: upper bound ≤ 1.02: {yes(b)}")
    out += [
        "",
        f"**This run: {'PASS' if v['pass'] else 'FAIL'}.** The policy is admitted only if one "
        "run and its predeclared confirmation both pass.",
        "",
    ]
    return "\n".join(out)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--out", required=True)
    a = p.parse_args(argv)
    rows = [json.loads(line) for line in open(a.runs) if line.strip()]
    v = judge(rows)
    with open(a.out + ".json", "w") as f:
        json.dump(v, f, indent=1, sort_keys=True)
    md = markdown(v)
    with open(a.out + ".md", "w") as f:
        f.write(md)
    print(md)
    return 0 if v["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
