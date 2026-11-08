"""The service core at two teams on the workstation
(docs/journal/2026-11-08-service-2x3-registration.md, registered before any run): the report
and its verdicts, each predicate checked here.

    python3 bench/service_2x3.py report RUNS.jsonl [--occupancy W/occupancy-*.json] [--out STEM]

A run is VALID only as the registered one: the registered 20 rounds (whatever the recipe was
asked for), from the attempt's directory (RUNS.jsonl's) a clean tree (`git-status.txt`
empty), no `NOT-FROZEN.txt` or `DRY-RUN.txt`, `commit.txt` the checkout's head, the ten-core
topology, every run the same bytes, every registered cell's rounds once each with its
registered plan, no other cell, the noise control N1 within 1 +/- 0.03, and the registered
trace complete.

The budget is the decode threads (the teams' threads plus the service threads), and every
cell runs at a budget of 10, in one `ab run`, interleaved by round. The registered cells
(ARM@BUDGET: team leaders, helpers, service threads):

    mibz2@10            3, 6, 0   today's default (0.2.6): 3 x 3 pipelined, park and small
    mibz2-svc2@10       2, 4, 2   the service core: 2 x 3 + 2
    mibz2-team2x3@10    2, 4, 0   2 x 3, plain (Plan::Teams { teams: 2, k: 3 })
    mibz2-pipe2x3@10    2, 4, 0   2 x 3 pipelined, park and small (the default's team)
    mibz2-team3@10      3, 6, 0   3 x 3, plain
    mibz2-team3-twin@10 3, 6, 0   mibz2-team3 under another name: the noise control

The predicates shared with the service core's registration (the outcome map, the plan count
from ab's per-thread CPU, the extra-thread label, the traces' failure modes and their
thresholds) are bench/service_core.py's, imported unchanged.
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

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from service_core import (  # noqa: E402
    CONTROL_EQUIV,
    extra_thread_label,
    failure_modes,
    geo_ci,
    is_gain,
    outcome,
    threads_of,
    timelines,
)

REGISTERED_ROUNDS = 20
# cell -> (leaders, helpers, service threads)
CELLS = {
    "mibz2@10": (3, 6, 0),
    "mibz2-svc2@10": (2, 4, 2),
    "mibz2-team2x3@10": (2, 4, 0),
    "mibz2-pipe2x3@10": (2, 4, 0),
    "mibz2-team3@10": (3, 6, 0),
    "mibz2-team3-twin@10": (3, 6, 0),
}
# the registered comparisons (name, arm, reference): judged, speed and CPU co-primary
JUDGED = [
    # (a) the product question: the service core at 2 x 3 + 2 against today's default
    ("P1", "mibz2-svc2@10", "mibz2@10"),
    # (b) the causal question: the same two teams of three, with and without the service
    ("C1", "mibz2-svc2@10", "mibz2-team2x3@10"),
    # (c) two teams under today's decoder (the pipelined team, park and small)
    ("P2", "mibz2-pipe2x3@10", "mibz2@10"),
]
# reported, never judged
SECONDARY = [
    ("S1", "mibz2-svc2@10", "mibz2-pipe2x3@10"),
    ("S2", "mibz2-team2x3@10", "mibz2@10"),
    ("S3", "mibz2-pipe2x3@10", "mibz2-team2x3@10"),
    ("S4", "mibz2-team2x3@10", "mibz2-team3@10"),
    ("S5", "mibz2-svc2@10", "mibz2-team3@10"),
    ("R1", "mibz2@10", "mibz2-team3@10"),
]
# the noise control, as the service core's registration has it, unchanged
CONTROL = ("N1", "mibz2-team3-twin@10", "mibz2-team3@10")
# the registered trace (the occupancy pass's name), which must be complete
TRACES = ["occupancy-t10-svc2"]
TRACE = "occupancy-t10-svc2"


def decode_threads(plan):
    leaders, helpers, services = plan
    return leaders + helpers + services


def cell(r):
    return f"{r['arm']}@{r['threads']}"


def readings(by):
    """The registration's readings, mechanically, from P1, C1 and P2"""
    p1, c1, p2 = by["P1"], by["C1"], by["P2"]
    s = p1["speed_outcome"]
    if is_gain(s):
        product = "A TRADE (a speed gain for a CPU COST)" if p1["cpu_outcome"] == "CPU COST" else "SVC2@10 BEATS THE DEFAULT"
    elif s in ("NULL", "REGRESSION"):
        product = f"NOT REPLICATED ({s})"
    else:
        product = "INCONCLUSIVE"
    c = c1["speed_outcome"]
    if c == "GAIN":
        cause = "THE SERVICE THREADS EARN THEIR KEEP"
    elif is_gain(c):
        cause = "A SMALL EFFECT OF THE SERVICE THREADS"
    elif c in ("NULL", "REGRESSION"):
        cause = f"THE SERVICE THREADS DON'T EARN THEIR KEEP ({c})"
    else:
        cause = "INCONCLUSIVE"
    q = p2["speed_outcome"]
    if is_gain(q):
        two = "TWO PIPELINED TEAMS BEAT THE DEFAULT"
    elif q in ("NULL", "REGRESSION"):
        two = f"TWO PIPELINED TEAMS DON'T BEAT THE DEFAULT ({q})"
    else:
        two = "INCONCLUSIVE"
    if s in ("NULL", "REGRESSION"):
        overall = "NOT REPLICATED (P1: svc2@10 doesn't beat today's default)"
    elif not is_gain(s):
        overall = "NONE OF THE FOUR (P1 inconclusive)"
    elif c in ("NULL", "REGRESSION"):
        overall = "TWO TEAMS, NOT THE SERVICE THREADS (P1 a gain, C1 NULL or REGRESSION)"
    elif not is_gain(c):
        overall = "NONE OF THE FOUR (C1 inconclusive)"
    elif p1["cpu_outcome"] == "CPU COST":
        overall = "A TRADE (P1 a gain for a CPU COST, C1 a gain: for Louis to weigh)"
    else:
        overall = "FOR SVC2@10 (P1 a gain, CPU not a CPU COST, and C1 a gain: the service threads cause it)"
    return {"product": product, "cause": cause, "two_teams": two, "overall": overall}


def counterexample(fm, c1):
    """The registered counterexample, service saturation at 2 x 3 + 2, read with C1"""
    if fm["saturated"] == "UNRESOLVED":
        return "UNRESOLVED (svc2@10's trace missing or incomplete)"
    if fm["saturated"] == "SATURATED":
        if c1["speed_outcome"] in ("NULL", "REGRESSION"):
            return "SATURATION BINDS (svc2@10 saturated, and C1 NULL or REGRESSION)"
        if is_gain(c1["speed_outcome"]):
            return "SATURATED, A GAIN NONETHELESS (svc2@10 saturated, and C1 a gain)"
        return "SATURATED, C1 INCONCLUSIVE"
    return "NOT SATURATED at 2 x 3 + 2"


def report(a):
    runs = [json.loads(x) for x in open(a.runs) if x.strip()]
    bad = [r for r in runs if r.get("err")]
    hashes = {r["hash"] for r in runs if not r.get("err")}
    lengths = {r["out"] for r in runs if not r.get("err")}
    by = defaultdict(dict)
    dupes = defaultdict(int)
    for r in runs:
        c = cell(r)
        if r["round"] in by[c]:
            dupes[c] += 1
            continue
        if not r.get("err"):
            by[c][r["round"]] = r
    want = list(range(REGISTERED_ROUNDS))
    w = os.path.dirname(os.path.abspath(a.runs))

    def text(name):
        try:
            return open(os.path.join(w, name)).read().strip()
        except OSError:
            return None

    try:
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        head = None
    rounds_seen = len({r["round"] for r in runs})
    lines = ["# The service core at two teams: the workstation run's report", ""]
    checks = []
    checks.append(
        (f"the registered {REGISTERED_ROUNDS} rounds (seen: {rounds_seen})", rounds_seen == REGISTERED_ROUNDS)
    )
    checks.append(("a clean tree (`git-status.txt` present and empty)", text("git-status.txt") == ""))
    checks.append(("no `NOT-FROZEN.txt` (the frozen inputs unchanged)", text("NOT-FROZEN.txt") is None))
    checks.append(("no `DRY-RUN.txt` (not a dry run)", text("DRY-RUN.txt") is None))
    commit = text("commit.txt")
    checks.append((f"`commit.txt` ({commit}) is the checkout's head ({head})", bool(commit) and commit == head))
    lscpu = text("lscpu.txt") or ""

    def lscpu_field(name):
        m = re.search(rf"^{name}:\s*(.+)$", lscpu, re.M)
        return m.group(1).strip() if m else None

    sockets, per_socket, l3 = lscpu_field("Socket\\(s\\)"), lscpu_field("Core\\(s\\) per socket"), lscpu_field("L3 cache") or lscpu_field("L3")
    ten = sockets == "1" and per_socket == "10" and bool(l3) and l3.split()[0:2] in (["20", "MiB"], ["20"])
    checks.append(
        (
            f"the ten-core topology, so `mibz2` is the pipelined default (lscpu: {sockets} socket, {per_socket} cores, L3 {l3})",
            ten,
        )
    )
    checks.append(("every run the same bytes, none failed", len(hashes) == 1 and len(lengths) == 1 and not bad))
    for c, plan in CELLS.items():
        rs = by.get(c, {})
        plans = {threads_of(r.get("thread_cpu", "")) for r in rs.values()}
        ok = sorted(rs) == want and not dupes[c] and plans == {plan}
        seen = ", ".join(f"{p[0]} leaders, {p[1]} helpers, {p[2]} service" for p in sorted(plans)) or "none"
        checks.append(
            (
                f"`{c}`: rounds 0-{REGISTERED_ROUNDS - 1} once each, every run {plan[0]} leaders, "
                f"{plan[1]} helpers, {plan[2]} service threads (seen: {seen})",
                ok,
            )
        )
    unexpected = sorted(set(by) - set(CELLS))
    checks.append((f"no cell beyond the registered ones (seen: {', '.join(unexpected) or 'none'})", not unexpected))
    tls, unreadable = timelines(a.occupancy) if a.occupancy else ({}, [])
    for name in TRACES:
        tl = tls.get(name, {})
        checks.append((f"the trace `{name}` complete (rounds: {tl.get('complete_rounds', 0)})", tl.get("complete_rounds", 0) > 0))
    valid = all(ok for _, ok in checks)

    def ratio(arm, ref, key="wall"):
        x, y = by.get(arm, {}), by.get(ref, {})
        rr = [y[i][key] / x[i][key] for i in want if i in x and i in y]
        return (geo_ci(rr) if rr else (math.nan,) * 3), len(rr)

    lines += ["## Validity", ""]
    lines += [f"- {t}: **{'yes' if ok else 'NO'}**" for t, ok in checks]
    (g, lo, hi), n = ratio(CONTROL[1], CONTROL[2])
    control_ok = valid and 1 - CONTROL_EQUIV <= lo and hi <= 1 + CONTROL_EQUIV
    lines.append(
        f"- the noise control {CONTROL[0]}, `{CONTROL[1]}` against `{CONTROL[2]}` (the same plan): "
        f"{g:.3f} [{lo:.3f}, {hi:.3f}] over {n} rounds, within 1 ± {CONTROL_EQUIV}: **{'yes' if control_ok else 'NO'}**"
    )
    verdict = "VALID" if valid and control_ok else "INVALID"
    lines += ["", f"Run: **{verdict}**", ""]
    out = {"verdict": verdict, "checks": checks, "judged": [], "secondary": [], "traces": {}}

    lines += [
        "## The registered comparisons (equal budget of 10; judged; speed and CPU co-primary)",
        "",
        "| | arm | against | decode threads | speed [95% CI] | speed outcome | CPU [95% CI] | CPU outcome | the extra threads |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for name, arm, ref in JUDGED:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        ta, tr = decode_threads(CELLS[arm]), decode_threads(CELLS[ref])
        if verdict == "VALID":
            so = outcome(*speed)
            co = outcome(*cpu, gain="CPU SAVING", loss="CPU COST")
            xl = extra_thread_label(speed, ta, tr) if is_gain(so) else "(no gain)"
        else:
            so = co = xl = "NONE (the run is invalid)"
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {ta} against {tr} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | {so} | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] | {co} | {xl} |"
        )
        out["judged"].append(
            {"name": name, "arm": arm, "ref": ref, "threads": [ta, tr], "speed": list(speed), "cpu": list(cpu),
             "rounds": n, "speed_outcome": so, "cpu_outcome": co, "extra_thread": xl}
        )
    lines.append("")
    jd = {c["name"]: c for c in out["judged"]}
    if verdict == "VALID":
        out["readings"] = readings(jd)
    else:
        out["readings"] = {k: "NONE (the run is invalid)" for k in ("product", "cause", "two_teams", "overall")}
    rd = out["readings"]
    lines += [
        "**The readings** (the registration's rules, computed from P1, C1 and P2):",
        "",
        f"- the product (P1): **{rd['product']}**",
        f"- the cause (C1): **{rd['cause']}** (C1's extra decode threads are the service threads themselves: its label is reported, not required)",
        f"- two teams under today's decoder (P2): **{rd['two_teams']}**",
        f"- overall: **{rd['overall']}**",
        "",
        "## Reported, never judged",
        "",
        "| | arm | against | decode threads | speed [95% CI] | CPU [95% CI] |",
        "|---|---|---|---|---|---|",
    ]
    for name, arm, ref in SECONDARY:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        ta, tr = decode_threads(CELLS[arm]), decode_threads(CELLS[ref])
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {ta} against {tr} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] |"
        )
        out["secondary"].append({"name": name, "arm": arm, "ref": ref, "speed": list(speed), "cpu": list(cpu)})
    tl = tls.get(TRACE, {})
    utils = sorted((k, v) for k, v in tl.items() if re.fullmatch(r"svc\.util\.\d+", k))
    u = ", ".join(f"{v:.2f}" for _, v in utils)
    u = f"{u}; max {tl.get('svc.util.max', float('nan')):.2f}" if utils else "–"
    lines += [
        "",
        "(Speed: the reference's wall over the arm's, > 1 the arm is faster. CPU: the reference's CPU over the arm's, > 1 the arm spends less.)",
        "",
        "## Every cell (medians)",
        "",
        "| cell | decode threads (leaders + helpers + service) | wall s | CPU s | peak RSS MB | rounds | service utilisation (each; max) |",
        "|---|---|---|---|---|---|---|",
    ]
    for c in sorted(by):
        rs = list(by[c].values())
        plan = CELLS.get(c)
        lines.append(
            f"| `{c}`{'' if plan else ' (unregistered)'} | {'+'.join(map(str, plan)) if plan else '?'} | "
            f"{statistics.median(r['wall'] for r in rs):.3f} | {statistics.median(r['cpu'] for r in rs):.3f} | "
            f"{statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} | {u if c == 'mibz2-svc2@10' else '–'} |"
        )
    lines += [
        "",
        "## The trace (occupancy, medians over complete rounds)",
        "",
        "| trace | rounds | blocks | util max | prep wait median / p90 ms | finish wait median / p90 ms | job wait median ms | backlog prep / finish (mean; max) | teams idle: prep / back share | run wait ms | saturation | prep starvation | finish backlog |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    fm = failure_modes(tl)
    show = verdict == "VALID"
    if tl:
        g = lambda k: tl.get(k, float("nan"))  # noqa: E731
        out["traces"][TRACE] = {"timeline": {k: v for k, v in tl.items() if k.startswith("svc.") or k in ("blocks", "complete_rounds")}, "modes": fm}
        lines.append(
            f"| {TRACE} | {g('complete_rounds')} | {g('blocks')} | {g('svc.util.max'):.2f} | "
            f"{g('svc.queue.prep.median_ms'):.3f} / {g('svc.queue.prep.p90_ms'):.3f} | "
            f"{g('svc.queue.finish.median_ms'):.3f} / {g('svc.queue.finish.p90_ms'):.3f} | {g('svc.queue.job.median_ms'):.3f} | "
            f"{g('svc.backlog.prep_mean'):.2f} / {g('svc.backlog.finish_mean'):.2f} ({g('svc.backlog.prep_max'):.0f}; {g('svc.backlog.finish_max'):.0f}) | "
            f"{g('svc.team.idle_prep_share'):.3f} / {g('svc.team.idle_back_share'):.3f} | {g('svc.team.ms.run_wait'):.1f} | "
            f"{fm['saturated'] if show else 'NONE'} | {fm['prep'] if show else 'NONE'} | {fm['finish'] if show else 'NONE'} |"
        )
    lines += [f"| {x} | | | | | | | | | | | | |" for x in unreadable]
    out["saturation"] = counterexample(fm, jd["C1"]) if show else "NONE (the run is invalid)"
    lines += ["", f"**The registered counterexample, service saturation at 2 × 3 + 2:** **{out['saturation']}**"]
    text_ = "\n".join(lines) + "\n"
    print(text_)
    if a.out:
        open(a.out + ".md", "w").write(text_)
        json.dump(out, open(a.out + ".json", "w"), indent=1)
    return 0 if verdict == "VALID" else 1


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--occupancy", nargs="*", default=[])
    r.add_argument("--out")
    a = p.parse_args()
    sys.exit(report(a))


if __name__ == "__main__":
    main()
