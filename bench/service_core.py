"""The service core on the workstation (docs/journal/2026-11-07-service-core-registration.md,
registered before any run): the report and its verdicts, each predicate checked here.

    python3 bench/service_core.py report RUNS.jsonl [--occupancy W/occupancy-*.json] [--out STEM]

A run is VALID only as the registered one: the registered 20 rounds (whatever the recipe was
asked for), from the attempt's directory (RUNS.jsonl's) a clean tree (`git-status.txt`
empty), no `NOT-FROZEN.txt` or `DRY-RUN.txt`, `commit.txt` the checkout's head, every run
the same bytes, every registered cell's rounds once each with its registered plan, no other
cell, the noise control N1 within 1 +/- 0.03, and every registered trace complete.

The budget is the decode threads: the teams' threads plus the service threads. RUNS.jsonl
holds two `ab run`s, appended: the arms at a budget of 10, then at 12, each interleaved by
round. The registered cells (ARM@BUDGET: team leaders, helpers, service threads):

    mibz2@10            3, 6, 0   today's default (0.2.6): 3 x 3 pipelined, park and small
    mibz2@12            3, 6, 0   today's default: the same 3 x 3 pipelined (9 threads)
    mibz2-team3@10      3, 6, 0   3 x 3, plain (not pipelined)
    mibz2-team3-twin@10 3, 6, 0   mibz2-team3 under another name: the noise control
    mibz2-team3@12      4, 8, 0   4 x 3, plain
    mibz2-svc1@10       3, 6, 1   3 x 3 + 1
    mibz2-svc1@12       3, 6, 1   3 x 3 + 1 (the teams capped at 3)
    mibz2-svc2@10       2, 4, 2   2 x 3 + 2 (3 teams don't fit 10 with 2 service threads)
    mibz2-svc2@12       3, 6, 2   3 x 3 + 2
    mibz2-svc1-t2@10    3, 3, 1   3 x 2 + 1
    mibz2-svc1-t2@12    3, 3, 1   3 x 2 + 1

`mibz2` is the pipelined default only on the ten-core topology (10 physical cores, a 20 MiB
L3: `Plan::for_threads` and `StreamDecoder::new`); the report checks the machine's
`lscpu.txt` for it, so `mibz2@10` and `mibz2@12` name the shipped default unambiguously.

Each comparison is paired by round: the speed ratio is the reference's wall over the arm's
(> 1: the arm is faster), the CPU ratio the reference's CPU over the arm's (> 1: the arm
spends less). Both are co-primary: each has its geometric mean and 95% t-interval over the
registered rounds, and its own registered outcome.
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
from team_size import geo_ci  # noqa: E402

REGISTERED_ROUNDS = 20
# cell -> (leaders, helpers, service threads)
CELLS = {
    "mibz2@10": (3, 6, 0),
    "mibz2@12": (3, 6, 0),
    "mibz2-team3@10": (3, 6, 0),
    "mibz2-team3-twin@10": (3, 6, 0),
    "mibz2-team3@12": (4, 8, 0),
    "mibz2-svc1@10": (3, 6, 1),
    "mibz2-svc1@12": (3, 6, 1),
    "mibz2-svc2@10": (2, 4, 2),
    "mibz2-svc2@12": (3, 6, 2),
    "mibz2-svc1-t2@10": (3, 3, 1),
    "mibz2-svc1-t2@12": (3, 3, 1),
}
# the causal comparisons, at an equal budget (name, arm, reference): judged, co-primary
CAUSAL = [
    ("C1", "mibz2-svc1@10", "mibz2-team3@10"),
    ("C2", "mibz2-svc1@10", "mibz2@10"),
    ("C3", "mibz2-svc2@12", "mibz2-team3@12"),
    ("C4", "mibz2-svc2@12", "mibz2@12"),
    # K's own effect at a fixed plan of teams: the saturation's causal test
    ("D1", "mibz2-svc2@12", "mibz2-svc1@12"),
]
# (the product comparisons, against today's default at the same budget, are C2 and C4
# themselves since the default is the pipelined team: no separate P1 or P2)
# reported, never judged
SECONDARY = [
    ("S1", "mibz2-svc1@12", "mibz2-team3@12"),
    ("S2", "mibz2-svc1@12", "mibz2@12"),
    ("S3", "mibz2-svc2@10", "mibz2-team3@10"),
    ("S4", "mibz2-svc2@10", "mibz2@10"),
    ("S5", "mibz2-svc1-t2@10", "mibz2-team3@10"),
    ("S6", "mibz2-svc1-t2@12", "mibz2-team3@12"),
    ("S7", "mibz2-svc2@10", "mibz2-svc1@10"),
    ("R1", "mibz2@10", "mibz2-team3@10"),
    ("R2", "mibz2@12", "mibz2-team3@12"),
]
CONTROL = ("N1", "mibz2-team3-twin@10", "mibz2-team3@10")
GAIN = 1.02
EQUIV = 0.03
CONTROL_EQUIV = 0.03
# the registered traces (occupancy pass names), each of which must be complete
TRACES = [f"occupancy-t{t}-svc{s}" for t in (10, 12) for s in ("1", "2", "1t2")]
# the failure modes' thresholds (medians over a trace's complete rounds)
SATURATED_UTIL = 0.90
PREP_STARVED = 0.10  # the teams' idle share waiting for a prepared block
FINISH_WAIT_MS = 0.5  # the finishing queue's median wait
BACK_STARVED = 0.05  # the teams' idle share waiting for a back set


def outcome(g, lo, hi, gain="GAIN", loss="REGRESSION"):
    """The registered outcome of a ratio's interval (speed, or CPU with its own words)"""
    if any(math.isnan(x) for x in (g, lo, hi)):
        return "INCONCLUSIVE (no interval)"
    if lo > GAIN:
        return gain
    if lo > 1.0:
        return f"SMALL {gain} (above 1, not above the threshold)"
    if hi < 1.0:
        return loss
    if 1 - EQUIV <= lo and hi <= 1 + EQUIV:
        return "NULL"
    return "INCONCLUSIVE (the interval spans gain and harm)"


def is_gain(word):
    return word == "GAIN" or word.startswith("SMALL GAIN")


def decode_threads(plan):
    leaders, helpers, services = plan
    return leaders + helpers + services


def extra_thread_label(speed, arm_threads, ref_threads):
    """A speed gain at an equal budget where the arm runs more decode threads than its
    reference: 'beyond its threads' only if the interval's lower bound clears the threads'
    ratio (as if speed scaled linearly with threads: the reference's most generous reading)"""
    if arm_threads <= ref_threads:
        return "no more decode threads than the reference"
    bound = arm_threads / ref_threads
    if speed[1] > bound:
        return f"BEYOND ITS THREADS (lower bound above {bound:.3f})"
    return f"NOT SHOWN BEYOND ITS THREADS ({arm_threads} against {ref_threads}: needs a lower bound above {bound:.3f})"


def threads_of(thread_cpu):
    """(leaders, helpers, service threads) as ab's `thread_cpu` names them"""
    leaders = helpers = services = 0
    for part in thread_cpu.split():
        name = part.rpartition("=")[0]
        if re.fullmatch(r"mibz2-\d+", name):
            leaders += 1
        elif name.startswith("mibz2-coop"):
            helpers += 1
        elif re.fullmatch(r"mibz2-svc-\d+", name):
            services += 1
    return leaders, helpers, services


def cell(r):
    return f"{r['arm']}@{r['threads']}"


def timelines(paths):
    out, bad = {}, []
    for path in paths:
        name = path.rsplit("/", 1)[-1].removesuffix(".json")
        try:
            o = json.load(open(path))
        except (OSError, ValueError) as e:
            bad.append(f"{name}: unreadable: {e}")
            continue
        for res in o.get("results", []):
            tl = res.get("timeline")
            if isinstance(tl, dict):
                out[name] = dict(tl)
    return out, bad


def failure_modes(tl):
    """The registered failure modes, from a trace's medians"""
    g = lambda k: tl.get(k, float("nan"))  # noqa: E731
    util, prep, fin, back = (
        g("svc.util.max"),
        g("svc.team.idle_prep_share"),
        g("svc.queue.finish.median_ms"),
        g("svc.team.idle_back_share"),
    )
    if any(math.isnan(x) for x in (util, prep, fin, back)) or not tl.get("complete_rounds"):
        return {"saturated": "UNRESOLVED", "prep": "UNRESOLVED", "finish": "UNRESOLVED"}
    starved = prep >= PREP_STARVED
    backlog = fin >= FINISH_WAIT_MS or back >= BACK_STARVED
    return {
        "saturated": "SATURATED" if util >= SATURATED_UTIL and (starved or backlog) else "not saturated",
        "prep": "PREP STARVATION" if starved else "no prep starvation",
        "finish": "FINISH BACKLOG" if backlog else "no finish backlog",
    }


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
    lines = ["# The service core: the workstation run's report", ""]
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
    out = {"verdict": verdict, "checks": checks, "causal": [], "secondary": [], "traces": {}}

    def judged(section, rows, key):
        lines.extend(
            [
                f"## {section}",
                "",
                "| | arm | against | decode threads | speed [95% CI] | speed outcome | CPU [95% CI] | CPU outcome | the extra thread |",
                "|---|---|---|---|---|---|---|---|---|",
            ]
        )
        for name, arm, ref in rows:
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
            out[key].append(
                {"name": name, "arm": arm, "ref": ref, "threads": [ta, tr], "speed": list(speed), "cpu": list(cpu),
                 "rounds": n, "speed_outcome": so, "cpu_outcome": co, "extra_thread": xl}
            )
        lines.append("")

    judged("The causal comparisons (equal budget; judged; C2 and C4 are also the product comparisons, against today's default)", CAUSAL, "causal")
    lines += [
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
    lines += [
        "",
        "(Speed: the reference's wall over the arm's, > 1 the arm is faster. CPU: the reference's CPU over the arm's, > 1 the arm spends less.)",
        "",
        "## Every cell (medians), with the service workers' utilisation",
        "",
        "Utilisation is each service thread's busy time over the run's span, from its trace (the occupancy pass of the same arm and budget, medians over its complete rounds): the headroom left, whatever the speed outcome.",
        "",
        "| cell | decode threads (leaders + helpers + service) | wall s | CPU s | peak RSS MB | rounds | service utilisation (each; max) |",
        "|---|---|---|---|---|---|---|",
    ]
    for c in sorted(by):
        rs = list(by[c].values())
        plan = CELLS.get(c)
        arm, t = c.rsplit("@", 1)
        tl = tls.get(f"occupancy-t{t}-svc{arm.removeprefix('mibz2-svc').replace('-t', 't')}", {}) if "svc" in arm else {}
        utils = sorted((k, v) for k, v in tl.items() if re.fullmatch(r"svc\.util\.\d+", k))
        u = ", ".join(f"{v:.2f}" for _, v in utils)
        u = f"{u}; max {tl.get('svc.util.max', float('nan')):.2f}" if utils else "–"
        lines.append(
            f"| `{c}`{'' if plan else ' (unregistered)'} | {'+'.join(map(str, plan)) if plan else '?'} | "
            f"{statistics.median(r['wall'] for r in rs):.3f} | {statistics.median(r['cpu'] for r in rs):.3f} | "
            f"{statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} | {u} |"
        )
    if tls or unreadable:
        lines += [
            "",
            "## The traces (occupancy, medians over complete rounds)",
            "",
            "| trace | rounds | blocks | util max | prep wait median / p90 ms | finish wait median / p90 ms | job wait median ms | backlog prep / finish (mean; max) | teams idle: prep / back share | run wait ms | saturation | prep starvation | finish backlog |",
            "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
        ]
        for name in sorted(tls):
            tl = tls[name]
            g = lambda k: tl.get(k, float("nan"))  # noqa: E731
            fm = failure_modes(tl)
            out["traces"][name] = {"timeline": {k: v for k, v in tl.items() if k.startswith("svc.") or k in ("blocks", "complete_rounds")}, "modes": fm}
            lines.append(
                f"| {name} | {g('complete_rounds')} | {g('blocks')} | {g('svc.util.max'):.2f} | "
                f"{g('svc.queue.prep.median_ms'):.3f} / {g('svc.queue.prep.p90_ms'):.3f} | "
                f"{g('svc.queue.finish.median_ms'):.3f} / {g('svc.queue.finish.p90_ms'):.3f} | {g('svc.queue.job.median_ms'):.3f} | "
                f"{g('svc.backlog.prep_mean'):.2f} / {g('svc.backlog.finish_mean'):.2f} ({g('svc.backlog.prep_max'):.0f}; {g('svc.backlog.finish_max'):.0f}) | "
                f"{g('svc.team.idle_prep_share'):.3f} / {g('svc.team.idle_back_share'):.3f} | {g('svc.team.ms.run_wait'):.1f} | "
                f"{fm['saturated'] if verdict == 'VALID' else 'NONE'} | {fm['prep'] if verdict == 'VALID' else 'NONE'} | {fm['finish'] if verdict == 'VALID' else 'NONE'} |"
            )
        lines += [f"| {u} | | | | | | | | | | | | |" for u in unreadable]
        # the saturation counterexample's reading (registered)
        d1 = next((c for c in out["causal"] if c["name"] == "D1"), None)
        s1 = failure_modes(tls.get("occupancy-t12-svc1", {}))["saturated"]
        if verdict != "VALID" or d1 is None:
            word = "NONE (the run is invalid)"
        elif s1 == "SATURATED" and is_gain(d1["speed_outcome"]):
            word = "SATURATION BINDS AT K = 1 (svc1@12 saturated, and K = 2 faster at the same teams)"
        elif s1 == "SATURATED":
            word = "SATURATION MEASURED, NOT SHOWN TO BIND (K = 2 not faster)"
        elif s1 == "UNRESOLVED":
            word = "UNRESOLVED (svc1@12's trace missing or incomplete)"
        else:
            word = "NOT SATURATED at K = 1"
        out["saturation"] = word
        lines += ["", f"**The registered counterexample, service-worker saturation:** **{word}**"]
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
