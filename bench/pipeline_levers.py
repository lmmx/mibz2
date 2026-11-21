"""The pipelined team's two levers on the workstation
(docs/journal/2026-11-07-pipeline-levers-registration.md, registered before any run): the
report and its verdicts, each predicate checked here.

    python3 bench/pipeline_levers.py report RUNS.jsonl [--occupancy W/occupancy-*.json] [--out STEM]

A run is VALID only as the registered one: the registered 20 rounds (whatever the recipe was
asked for), from the attempt's directory (RUNS.jsonl's) a clean tree (`git-status.txt`
empty), no `NOT-FROZEN.txt` or `DRY-RUN.txt`, and `commit.txt` the checkout's head.

RUNS.jsonl holds two `ab run`s, appended: the 3 x 3 arms at 10 threads, then the 3 x 4 arms
at 12, each interleaved by round. The registered cells (ARM@THREADS, the plan its threads
must show):

    mibz2@10              3 x 3   today's default on the i9 (the noise control's reference)
    mibz2-team3@10        3 x 3   cooperative, not pipelined
    mibz2-pipe3@10        3 x 3   pipelined, as registered at f8aa9dc2 (no lever)
    mibz2-pipe3-spin2k@10 3 x 3   the spin-budget control: the waits yield after 2,000 spins
    mibz2-pipe3-park@10   3 x 3   lever A: the waits park after the same 4,096 spins
    mibz2-pipe3-small@10  3 x 3   lever B: the leader stages on after a small block
    mibz2-pipe3-both@10   3 x 3   both levers
    mibz2-team4@12        3 x 4   cooperative, not pipelined
    mibz2-pipe4@12        3 x 4   pipelined, no lever
    mibz2-pipe4-spin2k@12 3 x 4   the spin-budget control
    mibz2-pipe4-park@12   3 x 4   lever A
    mibz2-pipe4-small@12  3 x 4   lever B
    mibz2-pipe4-both@12   3 x 4   both levers

Each comparison is paired by round: the speed ratio is the reference's wall over the arm's
(> 1: the arm is faster), the CPU ratio the reference's CPU over the arm's (> 1: the arm
spends less). Both are co-primary: each has its geometric mean and 95% t-interval over the
registered rounds, and its own registered outcome.
"""

import argparse
import json
import math
import os
import statistics
import subprocess
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from team_size import geo_ci, plan_label, plan_of, threads_of  # noqa: E402

LEVERS = ("park", "spin2k", "small", "both")
CELLS = {"mibz2@10": (3, 3), "mibz2-team3@10": (3, 3), "mibz2-team4@12": (3, 4)}
for _k, _t in ((3, 10), (4, 12)):
    CELLS[f"mibz2-pipe{_k}@{_t}"] = (3, _k)
    for _l in LEVERS:
        CELLS[f"mibz2-pipe{_k}-{_l}@{_t}"] = (3, _k)
# (name, lever, arm, reference): each lever against the pipe it changes, at both plans
COMPARISONS = []
for _l, _tag in (("park", "A"), ("spin2k", "S"), ("small", "B"), ("both", "AB")):
    for _i, (_k, _t) in enumerate(((3, 10), (4, 12)), 1):
        COMPARISONS.append((f"{_tag}{_i}", _l, f"mibz2-pipe{_k}-{_l}@{_t}", f"mibz2-pipe{_k}@{_t}"))
# reported, never judged: the registered prototype's comparisons replicated, and each lever
# against the non-pipelined team (how much of the pipeline's gain it keeps or adds)
REPLICATIONS = (
    [
        ("R1", "mibz2-pipe3@10", "mibz2-team3@10"),
        ("R2", "mibz2-pipe4@12", "mibz2-team4@12"),
    ]
    + [
        (f"T-{_l}{_i}", f"mibz2-pipe{_k}-{_l}@{_t}", f"mibz2-team{_k}@{_t}")
        for _l in LEVERS
        for _i, (_k, _t) in enumerate(((3, 10), (4, 12)), 1)
    ]
    + [
        # park against the short-spin yield: both changes at once (the prototype's first park)
        (f"PS{_i}", f"mibz2-pipe{_k}-park@{_t}", f"mibz2-pipe{_k}-spin2k@{_t}")
        for _i, (_k, _t) in enumerate(((3, 10), (4, 12)), 1)
    ]
)
# the noise control: the same plan twice, the default against its forced twin
CONTROL = ("N1", "mibz2-team3@10", "mibz2@10")
# the registered thresholds, on a ratio's 95% interval [lo, hi]
GAIN = 1.02  # a gain (or a CPU saving) is an interval wholly above this
EQUIV = 0.03  # a null is an interval containing 1 and wholly within 1 +/- this
HOLDS = 0.98  # the wall "holds" when the speed interval lies wholly above this
CONTROL_EQUIV = 0.03
# lever B's mechanism: its median cross-thread overlap above the pipe's by this share of
# the blocks
B_MECHANISM = 0.10
# lever B's counterexample, a later join: its summed join delay at least this multiple of
# the pipe's, and at least this many ms more (both medians over the trace's rounds)
LATE_JOIN_RATIO = 1.2
LATE_JOIN_MS = 5.0
REGISTERED_ROUNDS = 20


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


def park_verdict(speed, cpu):
    """Lever A's registered question: does CPU fall while the wall holds?"""
    (_, slo, shi), (_, clo, chi) = speed, cpu
    if any(math.isnan(x) for x in (slo, shi, clo, chi)):
        return "INCONCLUSIVE (no interval)"
    if clo > 1.0 and slo > HOLDS:
        return "YES: CPU falls, the wall holds"
    if clo > 1.0 and shi < 1.0:
        return "TRADE: CPU falls, the wall regresses"
    if clo > 1.0:
        return "CPU FALLS, the wall not shown to hold"
    if chi < 1.0:
        return "NO: CPU rises"
    if shi < 1.0:
        return "NO: the wall regresses, CPU not shown to fall"
    return "NO: CPU not shown to fall"


def cell(r):
    return f"{r['arm']}@{r['threads']}"


def timelines(paths):
    """{trace name: timeline dict} from occupancy passes (one result each)"""
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
    lines = ["# The pipelined team's levers: the workstation run's report", ""]
    checks = []
    same = len(hashes) == 1 and len(lengths) == 1 and not bad
    checks.append(
        (
            f"the registered {REGISTERED_ROUNDS} rounds (seen: {rounds_seen})",
            rounds_seen == REGISTERED_ROUNDS,
        )
    )
    checks.append(
        ("a clean tree (`git-status.txt` present and empty)", text("git-status.txt") == "")
    )
    checks.append(
        ("no `NOT-FROZEN.txt` (the frozen inputs unchanged)", text("NOT-FROZEN.txt") is None)
    )
    checks.append(("no `DRY-RUN.txt` (not a dry run)", text("DRY-RUN.txt") is None))
    commit = text("commit.txt")
    checks.append(
        (
            f"`commit.txt` ({commit}) is the checkout's head ({head})",
            bool(commit) and commit == head,
        )
    )
    checks.append(("every run the same bytes, none failed", same))
    for c, plan in CELLS.items():
        rs = by.get(c, {})
        plans = {plan_of(threads_of(r.get("thread_cpu", ""))) for r in rs.values()}
        want_plan = (plan[0], plan[0] * (plan[1] - 1))
        ok = sorted(rs) == want and not dupes[c] and plans == {want_plan}
        seen = ", ".join(plan_label(p) for p in sorted(plans)) or "none"
        checks.append(
            (
                f"`{c}`: rounds 0-{REGISTERED_ROUNDS - 1} once each, every run {plan[0]} x {plan[1]} (seen: {seen})",
                ok,
            )
        )
    unexpected = sorted(set(by) - set(CELLS))
    checks.append(
        (
            f"no cell beyond the registered ones (seen: {', '.join(unexpected) or 'none'})",
            not unexpected,
        )
    )
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
        f"{g:.3f} [{lo:.3f}, {hi:.3f}] over {n} rounds, within 1 ± {CONTROL_EQUIV}: "
        f"**{'yes' if control_ok else 'NO'}**"
    )
    lines.append("")
    verdict = "VALID" if valid and control_ok else "INVALID"
    lines += [
        f"Run: **{verdict}**",
        "",
        "## The registered comparisons (each lever against the pipe)",
        "",
    ]
    lines.append(
        "| | lever | arm | against | speed [95% CI] | speed outcome | CPU [95% CI] | CPU outcome | lever A's question |"
    )
    lines.append("|---|---|---|---|---|---|---|---|---|")
    out = {"verdict": verdict, "checks": checks, "comparisons": [], "unjudged": [], "traces": {}}
    for name, lever, arm, ref in COMPARISONS:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        if verdict == "VALID":
            so = outcome(*speed)
            co = outcome(*cpu, gain="CPU SAVING", loss="CPU COST")
            pv = park_verdict(speed, cpu) if lever == "park" else ""
        else:
            so = co = "NONE (the run is invalid)"
            pv = "NONE" if lever == "park" else ""
        lines.append(
            f"| {name} | {lever} | `{arm}` | `{ref}` | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | {so} | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] | {co} | {pv} |"
        )
        out["comparisons"].append(
            {
                "name": name,
                "lever": lever,
                "arm": arm,
                "ref": ref,
                "speed": list(speed),
                "cpu": list(cpu),
                "rounds": n,
                "speed_outcome": so,
                "cpu_outcome": co,
                "park_verdict": pv,
            }
        )
    # the interaction check: both levers worse than one alone on that lever's own figure
    # (speed for small, CPU for park), its interval wholly below that lever's mean
    lines += ["", "**Interaction** (both against each lever alone, on the lever's own figure):", ""]
    got = {c["name"]: c for c in out["comparisons"]}
    out["interaction"] = {}
    for i in (1, 2):
        ab, a_, b_ = got[f"AB{i}"], got[f"A{i}"], got[f"B{i}"]
        speed_x = ab["speed"][2] < b_["speed"][0]
        cpu_x = ab["cpu"][2] < a_["cpu"][0]
        if verdict != "VALID":
            word = "NONE (the run is invalid)"
        elif speed_x or cpu_x:
            word = "INTERACTION: " + ", ".join(
                w for w, x in (("speed below small's", speed_x), ("CPU below park's", cpu_x)) if x
            )
        else:
            word = "none"
        out["interaction"][f"AB{i}"] = word
        lines.append(f"- AB{i}: **{word}**")
    lines += [
        "",
        "(Speed: the reference's wall over the arm's, > 1 the arm is faster. CPU: the reference's CPU over the arm's, > 1 the arm spends less.)",
        "",
        "## Reported, never judged",
        "",
        "| | arm | against | speed [95% CI] | CPU [95% CI] |",
        "|---|---|---|---|---|",
    ]
    for name, arm, ref in REPLICATIONS + [("P-park", "mibz2-pipe4-park@12", "mibz2@10")]:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] |"
        )
        out["unjudged"].append(
            {"name": name, "arm": arm, "ref": ref, "speed": list(speed), "cpu": list(cpu)}
        )
    lines += [
        "",
        "## Every cell (medians)",
        "",
        "| cell | plan | wall s | CPU s | peak RSS MB | rounds |",
        "|---|---|---|---|---|---|",
    ]
    for c in sorted(by):
        rs = list(by[c].values())
        plans = sorted({plan_of(threads_of(r.get("thread_cpu", ""))) for r in rs})
        lines.append(
            f"| `{c}`{'' if c in CELLS else ' (unregistered)'} | {', '.join(plan_label(p) for p in plans)} | "
            f"{statistics.median(r['wall'] for r in rs):.3f} | {statistics.median(r['cpu'] for r in rs):.3f} | "
            f"{statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} |"
        )
    if a.occupancy:
        tls, unreadable = timelines(a.occupancy)
        lines += ["", "## The traces (occupancy, medians over complete rounds)", ""]
        lines += [
            "| trace | complete rounds | blocks | MTFs | MTF overlapped (cross thread) | (same thread) | RLE overlapped (cross thread) | build wait ms | build + walk ms | lead ms | join delay ms | runs: late joins / missed / all |",
            "|---|---|---|---|---|---|---|---|---|---|---|---|",
        ]
        for name in sorted(tls):
            tl = tls[name]
            g = lambda k: tl.get(k, float("nan"))  # noqa: E731
            lines.append(
                f"| {name} | {g('complete_rounds')} | {g('blocks')} | {g('pipe.n.mtf')} | "
                f"{g('pipe.overlap.cross_thread')} | {g('pipe.overlap.same_thread')} | "
                f"{g('pipe.overlap.rle_cross_thread')} | {g('pipe.ms.build_wait')} | "
                f"{g('pipe.ms.build') + g('pipe.ms.walk'):.1f} | {g('pipe.ms.lead')} | "
                f"{g('pipe.ms.join_delay')} | {g('pipe.n.join_late')} / {g('pipe.n.join_missed')} / {g('pipe.n.join_runs')} |"
            )
        lines += [f"| {u} | | | | | | | | | | | |" for u in unreadable]
        lines += [
            "",
            "**Lever B's mechanism** (registered: its cross-thread overlap above the pipe's by at least "
            f"{B_MECHANISM:.0%} of the blocks), and its counterexample (a later join: the leader's summed join "
            f"delay at least {LATE_JOIN_RATIO}x the pipe's and {LATE_JOIN_MS:g} ms more). "
            "`pipeline_counts` (jobs taken in a run) is context only, never evidence:",
            "",
        ]
        for k, t in ((3, 10), (4, 12)):
            base = tls.get(f"occupancy-t{t}-pipe{k}", {})
            for lever in ("small", "both"):
                tl = tls.get(f"occupancy-t{t}-pipe{k}-{lever}", {})
                x, y = tl.get("pipe.overlap.cross_thread"), base.get("pipe.overlap.cross_thread")
                blocks = tl.get("blocks") or base.get("blocks")
                complete = tl.get("complete_rounds", 0) > 0 and base.get("complete_rounds", 0) > 0
                if None in (x, y, blocks) or not complete:
                    shown = "UNRESOLVED (a trace missing or incomplete)"
                else:
                    shown = "SHOWN" if x - y >= B_MECHANISM * blocks else "NOT SHOWN"
                jx, jy = tl.get("pipe.ms.join_delay"), base.get("pipe.ms.join_delay")
                if None in (jx, jy) or not complete:
                    late = "UNRESOLVED"
                else:
                    late = (
                        "LATER JOIN"
                        if jx >= LATE_JOIN_RATIO * jy and jx - jy >= LATE_JOIN_MS
                        else "no later join"
                    )
                lines.append(
                    f"- {lever} at 3 x {k} ({t} threads): cross-thread overlap {x} against {y} of {blocks} blocks: "
                    f"**{shown}**; the leader's join delay {jx} against {jy} ms: **{late}**"
                )
                out["traces"][f"{lever}{k}"] = {
                    "lever": x,
                    "pipe": y,
                    "blocks": blocks,
                    "mechanism": shown,
                    "join_delay": [jx, jy],
                    "join": late,
                }
        lines += [
            "",
            "A pipelined trace with no cross-thread overlap, or lever B's mechanism not shown, means no mechanism verdict for that arm (the registration's outcome map).",
        ]
    text = "\n".join(lines) + "\n"
    print(text)
    if a.out:
        open(a.out + ".md", "w").write(text)
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
