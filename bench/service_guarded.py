"""The guarded service default's workstation A/B (docs/journal/2026-11-09-service-guarded-registration.md,
DRAFT, not frozen): the candidate build's default decoder (`BzReader::parallel`, which runs the
service core, svc2, at exactly 10 threads on its target) against the current default's build
(the base: master's default, the pipelined three teams of three there), through the same
public entry point (`ab`'s `mibz2` arm), on the bench's Wikidata sample, paired by round.

The rule, fixed with the registration before any run (as the service-core registrations':
docs/journal/2026-11-08-service-2x3-registration.md, unchanged where it applies):

- **Validity**, every cell: 20 rounds, each once, no failed run; one output hash and one
  length across every run; every run's plan the one expected (`EXPECT`), read both from the
  threads that ran (`thread_cpu`) and from every thread the parent `ab` saw (`observed_threads`);
  the noise control **N1** (the base binary against a byte-identical copy of itself under
  another path, at 10 threads) with both bounds of its 95% interval within [0.97, 1.03]; the
  preflight passed (`--preflight`, its JSON's `ok`).
- **Identity checks**, 9 and 11 threads, where both builds run the same plan (four plain teams
  of two; three pipelined teams of three): reported with the ±2% flag, not judged.
- **P1, judged, at 10 threads:** speed (base wall over head wall) and CPU (base CPU over head
  CPU), each the geometric mean of the 20 rounds' paired ratios with its two-sided 95%
  t-interval, co-primary, each mapped by the service-core registrations' outcome map.
- **Peak RSS** (each run's own process: `ab` runs every run as a child process of its own, so
  `ru_maxrss` is that run's): the paired geometric-mean ratio of head over base with its
  interval, and the medians, at every cell, **reported, not judged**.

    python3 bench/service_guarded.py report RUNS.jsonl --preflight PREFLIGHT.json --out DIR/service-guarded
"""

import argparse
import json
import math
import re
import statistics
import sys

ROUNDS = 20
# Student's t at 19 degrees of freedom: the two-sided 95% quantile (scipy.stats.t.ppf(0.975, 19))
T_975 = 2.0930
CONTROLS = (9, 11)  # identity checks: reported, not judged
PRIMARY = 10
THREADS = (9, PRIMARY, 11)
# plans as (leaders, plain helpers, pipelined helpers, service workers)
FOUR_BY_TWO = (4, 4, 0, 0)
PIPED_3X3 = (3, 0, 6, 0)
SVC2 = (2, 4, 0, 2)
EXPECT = {
    9: {"base": FOUR_BY_TWO, "head": FOUR_BY_TWO, "twin": FOUR_BY_TWO},
    10: {"base": PIPED_3X3, "head": SVC2, "twin": PIPED_3X3},
    11: {"base": PIPED_3X3, "head": PIPED_3X3, "twin": PIPED_3X3},
}
CONTROL_BAND = (0.98, 1.02)
N1_BAND = (0.97, 1.03)
CORPUS = "wikidata"
# the recipe's names for the two base binaries (the same bytes, two paths)
BASE_SUFFIX, TWIN_SUFFIX = "/bin/ab-base", "/bin/ab-base-twin"


def plan_of(threads):
    """(leaders, plain helpers, pipelined helpers, service workers) from ab's `thread_cpu`
    ("name=seconds ...") or `observed_threads` ("name ..."): leaders or workers `mibz2-N`, a
    plain team's helpers `mibz2-coop-N`, a pipelined team's `mibz2-coop-pN`, service workers
    `mibz2-svc-N` (the hub's feeder, `mibz2-svc-feed`, is not counted)"""
    names = [p.partition("=")[0] for p in threads.split()]
    return (
        sum(1 for n in names if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-p\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-svc-\d+", n)),
    )


def build_of(arm):
    """ "head" for the candidate's own `mibz2`, "base" and "twin" for the two base binaries"""
    if not isinstance(arm, str):
        return None
    if arm == "mibz2":
        return "head"
    if arm.startswith("mibz2@") and arm.endswith(TWIN_SUFFIX):
        return "twin"
    if arm.startswith("mibz2@") and arm.endswith(BASE_SUFFIX):
        return "base"
    return None


def label(plans):
    out = []
    for leaders, plain, piped, svc in plans:
        helpers = plain + piped
        kind = "pipelined" if piped and not plain else "plain" if plain and not piped else "mixed"
        s = f" + {svc} service" if svc else ""
        if helpers == 0 and not svc:
            out.append(f"{leaders} workers")
        elif leaders and helpers % leaders == 0:
            out.append(f"{leaders}×{1 + helpers // leaders} {kind}{s}")
        else:
            out.append(f"{leaders} leaders, {plain} plain, {piped} pipelined, {svc} service")
    return " / ".join(out) or "nothing"


def positive(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def paired(ref, arm, key):
    """The paired geometric mean of ref[key]/arm[key] over the rounds, and its 95% interval"""
    logs = [math.log(ref[i][key] / arm[i][key]) for i in range(ROUNDS)]
    m, se = statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(ROUNDS)
    return {"ratio": math.exp(m), "lo": math.exp(m - T_975 * se), "hi": math.exp(m + T_975 * se)}


def outcome(r, cpu=False):
    """The service-core registrations' outcome map, unchanged, on a ratio's 95% interval"""
    lo, hi = r["lo"], r["hi"]
    if lo > 1.02:
        return "CPU SAVING" if cpu else "GAIN"
    if lo > 1.0:
        return "SMALL CPU SAVING" if cpu else "SMALL GAIN"
    if lo >= 0.97 and hi <= 1.03 and lo <= 1.0 <= hi:
        return "NULL"
    if hi < 1.0:
        return "CPU COST" if cpu else "REGRESSION"
    return "INCONCLUSIVE"


def reading(speed, cpu):
    """P1's registered reading, from its two outcomes"""
    gain = speed in ("GAIN", "SMALL GAIN")
    if gain and cpu != "CPU COST":
        return "THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT"
    if gain:
        return "A TRADE"
    if speed in ("NULL", "REGRESSION"):
        return "NOT REPLICATED"
    return "INCONCLUSIVE"


def row_problems(rows):
    problems = []
    if not all(isinstance(r, dict) for r in rows):
        return [f"{sum(not isinstance(r, dict) for r in rows)} row(s) not a JSON object"]
    failed = [r for r in rows if r.get("err")]
    if failed:
        problems.append(f"{len(failed)} failed run(s)")
    for b in ("base", "twin"):
        arms = sorted({r.get("arm") for r in rows if build_of(r.get("arm")) == b})
        if len(arms) != 1:
            problems.append(f"{len(arms)} {b} binaries ({', '.join(map(str, arms)) or 'none'})")
    stray = sorted({str(r.get("arm")) for r in rows if build_of(r.get("arm")) is None})
    if stray:
        problems.append(f"rows from undeclared arms ({', '.join(stray)})")
    if any(r.get("threads") not in THREADS for r in rows):
        problems.append("rows at unregistered thread counts")
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
        problems.append(f"{len(bad)} malformed row(s)")
    return problems


def judge(rows, preflight, unreadable=()):
    problems = [f"line {n} of the runs file is not JSON" for n in unreadable][:5]
    if not (isinstance(preflight, dict) and preflight.get("ok") is True):
        why = preflight.get("why") if isinstance(preflight, dict) else None
        problems.append(f"the preflight did not pass ({why or 'no preflight record'})")
    if not unreadable:
        problems += row_problems(rows)
    empty = {
        "valid": False,
        "problems": problems,
        "cells": {},
        "n1": None,
        "p1": None,
    }
    if problems:
        return empty
    by = {}
    for r in rows:
        by.setdefault((build_of(r["arm"]), r["threads"]), []).append(r)
    if len({(r["hash"], r["out"]) for r in rows}) != 1:
        problems.append("more than one distinct output")
    cells = {}
    for t in THREADS:
        runs = {b: by.get((b, t), []) for b in ("base", "head", "twin")}
        complete = all(sorted(r["round"] for r in runs[b]) == list(range(ROUNDS)) for b in runs)
        seen = {
            b: sorted({plan_of(r[k]) for r in runs[b] for k in ("thread_cpu", "observed_threads")})
            for b in runs
        }
        plans_ok = all(seen[b] == [EXPECT[t][b]] for b in runs)
        c = {"threads": t, "complete": complete, "plans": plans_ok, "seen": seen}
        if not complete:
            problems.append(f"{t} threads: rounds incomplete or repeated")
        if not plans_ok:
            problems.append(
                f"{t} threads: ran base {label(seen['base'])}, head {label(seen['head'])}, "
                f"twin {label(seen['twin'])}; expected base {label([EXPECT[t]['base']])}, "
                f"head {label([EXPECT[t]['head']])}"
            )
        if complete:
            base = {r["round"]: r for r in runs["base"]}
            head = {r["round"]: r for r in runs["head"]}
            twin = {r["round"]: r for r in runs["twin"]}
            c["speed"] = paired(base, head, "wall")
            c["cpu"] = paired(base, head, "cpu")
            # (head over base: above 1 is more memory)
            c["rss"] = paired(head, base, "rss_kb")
            c["twin_speed"] = paired(base, twin, "wall")
            c["median"] = {
                b: {k: statistics.median(r[k] for r in runs[b]) for k in ("wall", "cpu", "rss_kb")}
                for b in runs
            }
        cells[t] = c
    n1 = cells[PRIMARY].get("twin_speed")
    n1_ok = n1 is not None and N1_BAND[0] <= n1["lo"] and n1["hi"] <= N1_BAND[1]
    if n1 is not None and not n1_ok:
        problems.append(
            f"N1 at 10 threads: {n1['ratio']:.3f} [{n1['lo']:.3f}, {n1['hi']:.3f}] not within [0.97, 1.03]"
        )
    for t in CONTROLS:
        s = cells[t].get("speed")
        cells[t]["flag"] = s is not None and (s["lo"] > CONTROL_BAND[1] or s["hi"] < CONTROL_BAND[0])
    valid = not problems
    p1 = None
    if valid:
        c = cells[PRIMARY]
        so, co = outcome(c["speed"]), outcome(c["cpu"], cpu=True)
        p1 = {"speed": so, "cpu": co, "reading": reading(so, co)}
    return {"valid": valid, "problems": problems, "cells": cells, "n1": n1, "p1": p1}


def fmt(r):
    return f"{r['ratio']:.3f} [{r['lo']:.3f}, {r['hi']:.3f}]" if r else "–"


def markdown(v):
    out = [
        "# The guarded service default against the current default",
        "",
        "Speed and CPU are the base's over the head's (above 1: the head is faster, or spends "
        "less); RSS is the head's over the base's (above 1: more memory). Paired geometric "
        "means over 20 rounds, 95% t-intervals.",
        "",
        "| threads | role | plans, base → head | speed | CPU | peak RSS | twin (N1 at 10) |",
        "|--:|---|---|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        role = "P1 (judged)" if t == PRIMARY else "identity (reported)"
        seen = c.get("seen", {})
        plans = f"{label(seen.get('base', []))} → {label(seen.get('head', []))}"
        flag = " ±2% FLAG" if c.get("flag") else ""
        out.append(
            f"| {t} | {role} | {plans} | {fmt(c.get('speed'))}{flag} | {fmt(c.get('cpu'))} "
            f"| {fmt(c.get('rss'))} | {fmt(c.get('twin_speed'))} |"
        )
    out += ["", f"**Valid:** {'yes' if v['valid'] else 'no'}"]
    out += [f"- {p}" for p in v["problems"]]
    if v["p1"]:
        out += [
            "",
            f"**P1:** speed {v['p1']['speed']}, CPU {v['p1']['cpu']}: **{v['p1']['reading']}**.",
            "",
            "Peak RSS is reported, not judged. One i9 result does not qualify the predicate's "
            "class, nor does CI's off-path bench qualify the service default's gain.",
        ]
    return "\n".join(out) + "\n"


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--preflight", required=True)
    r.add_argument("--out", required=True)
    a = p.parse_args(argv)
    rows, unreadable = [], []
    for n, line in enumerate(open(a.runs, errors="replace"), 1):
        if line.strip():
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                unreadable.append(n)
    try:
        preflight = json.load(open(a.preflight))
    except (OSError, json.JSONDecodeError):
        preflight = None
    v = judge(rows, preflight, unreadable)
    with open(a.out + ".json", "w") as f:
        json.dump(v, f, indent=1, sort_keys=True)
    md = markdown(v)
    with open(a.out + ".md", "w") as f:
        f.write(md)
    print(md)
    return 0 if v["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
