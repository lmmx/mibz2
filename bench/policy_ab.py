"""The three-team policy's workstation A/B (docs/journal/2026-11-06-three-team-policy.md): the
head build's default plan against the base build's, at the same requested thread count, on
the bench's Wikidata sample, paired by round. Its verdict follows the rule Louis approved in
the Coordinator's session on 2026-11-06 (docs/journal/decisions.md), fixed before any run:

- **Validity**, for every cell: 20 rounds, each once, no failed run; one output hash and one
  length across every run; every run's plan the one the registration expects (below), read
  both from the threads that ran (`thread_cpu`) and from every thread the sampler saw, busy
  or not (`thread_names`), so an idle helper still counts.
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
CORPUS = "wikidata"


def plan_of(threads):
    """(leaders, helpers) from ab's `thread_cpu` ("name=seconds ...") or `thread_names`
    ("name ..."): team leaders or workers are `mibz2-N`, helpers `mibz2-coop-*`"""
    names = [p.partition("=")[0] for p in threads.split()]
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


def label(plans):
    """ "3×3", from (leaders, helpers); workers alone as "8 workers"; several, joined"""
    out = []
    for leaders, helpers in plans:
        if helpers == 0:
            out.append(f"{leaders} workers")
        elif leaders and helpers % leaders == 0:
            out.append(f"{leaders}×{1 + helpers // leaders}")
        else:
            out.append(f"{leaders} leaders + {helpers} helpers")
    return " / ".join(out) or "nothing"


def log_stats(ratios):
    logs = [math.log(r) for r in ratios]
    return statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(len(logs))


def positive(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def row_problems(rows):
    """What makes the rows, as a whole, not the registered run: anything but the Wikidata
    sample at the registered thread counts, from the head's `mibz2` and one base binary,
    each run with a hash, a positive output length and finite positive wall, CPU and RSS"""
    problems = []
    failed = [r for r in rows if r.get("err")]
    if failed:
        problems.append(f"{len(failed)} failed run(s)")
    bases = sorted({r.get("arm") for r in rows if build_of(r.get("arm", "")) == "base"})
    if len(bases) != 1:
        problems.append(f"{len(bases)} base binaries ({', '.join(map(str, bases)) or 'none'})")
    stray = [r for r in rows if build_of(r.get("arm", "")) is None]
    if stray:
        arms = sorted({str(r.get("arm")) for r in stray})
        problems.append(f"{len(stray)} row(s) from undeclared arms ({', '.join(arms)})")
    off = [r for r in rows if r.get("threads") not in THREADS]
    if off:
        problems.append(f"{len(off)} row(s) at unregistered thread counts")
    corpora = sorted({str(r.get("corpus")) for r in rows} - {CORPUS})
    if corpora:
        problems.append(f"rows from other corpora ({', '.join(corpora)})")
    bad = [
        r
        for r in rows
        if not r.get("err")
        and not (
            isinstance(r.get("hash"), str)
            and r["hash"]
            and isinstance(r.get("out"), int)
            and not isinstance(r.get("out"), bool)
            and r["out"] > 0
            and all(positive(r.get(k)) for k in ("wall", "cpu", "rss_kb"))
            and isinstance(r.get("round"), int)
            and isinstance(r.get("thread_cpu"), str)
            and isinstance(r.get("thread_names"), str)
        )
    ]
    if bad:
        problems.append(
            f"{len(bad)} malformed row(s): a missing hash or length, or a "
            "wall, CPU or RSS not finite and positive"
        )
    return problems


def judge(rows):
    """The run's verdict: a dict of every cell's figures and every predicate"""
    cells = {}
    problems = row_problems(rows)
    if problems:
        # the figures below are only computed from rows that are all well formed
        return {
            "valid": False,
            "problems": problems,
            "cells": {},
            "controls": {t: False for t in CONTROLS},
            "primary": {k: False for k in ("speed", "lower bound", "cpu", "rss")},
            "no_harm": {t: False for t in NO_HARM},
            "pass": False,
        }
    by = {}
    for r in rows:
        by.setdefault((build_of(r["arm"]), r["threads"]), []).append(r)
    outputs = {(r["hash"], r["out"]) for r in rows}
    valid = len(outputs) == 1
    if len(outputs) != 1:
        problems.append(f"{len(outputs)} distinct outputs")
    for t in THREADS:
        runs = {b: by.get((b, t), []) for b in ("base", "head")}
        rounds = {b: sorted(r["round"] for r in runs[b]) for b in runs}
        complete = all(rounds[b] == list(range(ROUNDS)) for b in runs)
        plans_ok = all(
            plan_of(r[k]) == EXPECT[t][b]
            for b in runs
            for r in runs[b]
            for k in ("thread_cpu", "thread_names")
        )
        seen = {
            b: sorted({plan_of(r[k]) for r in runs[b] for k in ("thread_cpu", "thread_names")})
            for b in runs
        }
        c = {"threads": t, "complete": complete, "plans": plans_ok, "seen": seen}
        if not complete:
            problems.append(f"{t} threads: rounds incomplete or repeated")
        if not plans_ok:
            problems.append(
                f"{t} threads: base ran {label(seen['base'])}, head {label(seen['head'])}, "
                f"not {label([EXPECT[t]['base']])} and {label([EXPECT[t]['head']])}"
            )
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
        "| threads | role | plans run, base → head | speed (95%) | head/base wall, upper bound | CPU | RSS |",
        "|--:|---|---|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        role = "control" if t in CONTROLS else "primary" if t == PRIMARY else "no harm"
        seen = c.get("seen", {})
        plans = f"{label(seen.get('base', []))} → {label(seen.get('head', []))}"
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
