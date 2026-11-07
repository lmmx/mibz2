"""The pipelined default's workstation A/B (docs/journal/2026-11-07-pipelined-default-registration.md):
the head build's default decoder against v0.2.5's (the base build), at the same requested
thread count, on the bench's Wikidata sample, paired by round. Its verdict follows the rule
fixed with the registration, before any run, modelled on the three-team policy's
(bench/policy_ab.py, docs/journal/2026-11-06-three-team-policy.md):

- **Validity**, for every cell: 20 rounds, each once, no failed run; one output hash and one
  length across every run; every run's plan the one the registration expects (below), read
  both from the threads that ran (`thread_cpu`, the child's own) and from every thread the
  parent `ab` saw it run, busy or not (`observed_threads`, the same observer for both
  builds), so an idle helper still counts. A plain team's helpers are `mibz2-coop-N`, a
  pipelined team's `mibz2-coop-pN`, so the plan read says which path ran.
- **Identity checks**, 8 and 9 threads, where both builds run four plain teams of two: their
  plans count toward validity; their speed is reported, with the ±2% flag of #220's
  control, and not judged.
- **Primary**, 10 threads: speed ratio (base wall over head wall, geometric mean of the
  rounds' ratios) at least 1.03, its two-sided 95% lower bound above 1.00.
- **No harm**, 11, 12, 16 and 20 threads: each head-over-base wall ratio's one-sided upper
  bound, Bonferroni over the four (each at 1 - 0.05/4), at most 1.02. Too wide is
  inconclusive, which fails.
- **CPU and peak RSS** (the head's median over the base's) are reported at every cell, and
  not judged.

A run passes only if every predicate holds. One run, nothing pooled or switched.

    python3 bench/pipelined_default.py report RUNS.jsonl --out DIR/pipelined-default
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
CONTROLS = (8, 9)  # the identity checks: reported, not judged
PRIMARY = 10
NO_HARM = (11, 12, 16, 20)
THREADS = CONTROLS + (PRIMARY,) + NO_HARM
# the plans each build must run, as (leaders, plain helpers, pipelined helpers): four plain
# teams of two; three plain teams of three (v0.2.5's); three pipelined teams of three
OLD, PLAIN, PIPED = (4, 4, 0), (3, 6, 0), (3, 0, 6)
EXPECT = {
    t: {"base": OLD, "head": OLD} if t in CONTROLS else {"base": PLAIN, "head": PIPED}
    for t in THREADS
}
MIN_SPEED, NO_HARM_MAX, CONTROL_BAND = 1.03, 1.02, (0.98, 1.02)
CORPUS = "wikidata"


def plan_of(threads):
    """(leaders, plain helpers, pipelined helpers) from ab's `thread_cpu` ("name=seconds ...")
    or `observed_threads` ("name ..."): team leaders or workers are `mibz2-N`, a plain team's
    helpers `mibz2-coop-N`, a pipelined team's `mibz2-coop-pN`"""
    names = [p.partition("=")[0] for p in threads.split()]
    return (
        sum(1 for n in names if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-p\d+", n)),
    )


def build_of(arm):
    """ "head" for the head build's own `mibz2`, "base" for `mibz2@<base binary>`"""
    if not isinstance(arm, str):
        return None
    if arm == "mibz2":
        return "head"
    if arm.startswith("mibz2@"):
        return "base"
    return None


def label(plans):
    """ "3×3 pipelined", from (leaders, plain helpers, pipelined helpers); workers alone as
    "8 workers"; several, joined"""
    out = []
    for leaders, plain, piped in plans:
        helpers = plain + piped
        kind = "pipelined" if piped and not plain else "plain" if plain and not piped else "mixed"
        if helpers == 0:
            out.append(f"{leaders} workers")
        elif leaders and helpers % leaders == 0:
            out.append(f"{leaders}×{1 + helpers // leaders} {kind}")
        else:
            out.append(f"{leaders} leaders + {plain} plain and {piped} pipelined helpers")
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
    if not all(isinstance(r, dict) for r in rows):
        n = sum(not isinstance(r, dict) for r in rows)
        return [f"{n} row(s) not a JSON object"]
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
            and isinstance(r.get("observed_threads"), str)
        )
    ]
    if bad:
        problems.append(
            f"{len(bad)} malformed row(s): a missing hash or length, or a "
            "wall, CPU or RSS not finite and positive"
        )
    return problems


def judge(rows, unreadable=()):
    """The run's verdict: a dict of every cell's figures and every predicate. `unreadable`
    names lines of the runs file that weren't JSON: any makes the run invalid"""
    cells = {}
    problems = [f"line {n} of the runs file is not JSON" for n in unreadable][:5]
    if len(unreadable) > 5:
        problems.append(f"... and {len(unreadable) - 5} more unreadable line(s)")
    if not problems:
        problems = row_problems(rows)
    if problems:
        # the figures below are only computed from rows that are all well formed
        return {
            "valid": False,
            "problems": problems,
            "cells": {},
            "controls": {t: False for t in CONTROLS},
            "primary": {k: False for k in ("speed", "lower bound")},
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
            for k in ("thread_cpu", "observed_threads")
        )
        seen = {
            b: sorted({plan_of(r[k]) for r in runs[b] for k in ("thread_cpu", "observed_threads")})
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
    }
    no_harm = {t: ok(t, lambda c: c["harm_upper"] <= NO_HARM_MAX) for t in NO_HARM}
    # (the identity checks, `controls`, are reported and not judged)
    passed = valid and all(primary.values()) and all(no_harm.values())
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
        "# The pipelined default: head default against v0.2.5's default",
        "",
        "Speed is the base build's wall over the head's, paired by round (above 1 is faster); "
        "CPU and RSS are the head's median over the base's.",
        "",
        "| threads | role | plans run, base → head | speed (95%) | head/base wall, upper bound | CPU | RSS |",
        "|--:|---|---|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        role = "identity" if t in CONTROLS else "primary" if t == PRIMARY else "no harm"
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
        out.append(
            f"- Identity check at {t} (reported, not judged): 95% interval not wholly "
            f"beyond ±2%: {yes(b)}"
        )
    for k, b in v["primary"].items():
        out.append(f"- Primary at 10, {k}: {yes(b)}")
    for t, b in v["no_harm"].items():
        out.append(f"- No harm at {t}: upper bound ≤ 1.02: {yes(b)}")
    out += [
        "",
        f"**This run: {'PASS' if v['pass'] else 'FAIL'}.** CPU and RSS are reported, not "
        "judged. One run, as registered: nothing pooled or switched.",
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
    rows, unreadable = [], []
    for n, line in enumerate(open(a.runs, errors="replace"), 1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            unreadable.append(n)
    v = judge(rows, unreadable)
    with open(a.out + ".json", "w") as f:
        json.dump(v, f, indent=1, sort_keys=True)
    md = markdown(v)
    with open(a.out + ".md", "w") as f:
        f.write(md)
    print(md)
    return 0 if v["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
