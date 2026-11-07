"""The pipelined team's workstation run (docs/journal/2026-11-07-team-pipeline-registration.md,
registered before any run): its report and verdicts, each predicate checked here.

    python3 bench/team_pipeline.py report RUNS.jsonl --rounds N [--occupancy W/occupancy-*.json]
        [--out STEM]

RUNS.jsonl is `ab run`'s, arms mibz2, mibz2-team3, mibz2-pipe3, mibz2-team4, mibz2-pipe4 at 10
and 12 threads, interleaved by round. The registered cells (ARM@THREADS, the plan its
threads must show):

    mibz2@10         3 x 3   today's default on the i9 (the product reference)
    mibz2-team3@10   3 x 3   cooperative, not pipelined
    mibz2-pipe3@10   3 x 3   pipelined
    mibz2-team4@12   3 x 4   cooperative, not pipelined
    mibz2-pipe4@12   3 x 4   pipelined

The other cells ab's grid runs (mibz2@12, the arms of 3 at 12 threads (4 x 3), of 4 at 10
(2 x 4)) are reported, never judged. Each comparison is the paired speed ratio by round,
the reference's wall over the arm's (> 1: the arm is faster), its geometric mean and 95%
t-interval over the registered rounds; and CPU and peak RSS beside it.
"""

import argparse
import json
import math
import statistics
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from team_size import geo_ci, plan_label, plan_of, threads_of  # noqa: E402

CELLS = {
    "mibz2@10": (3, 3),
    "mibz2-team3@10": (3, 3),
    "mibz2-pipe3@10": (3, 3),
    "mibz2-team4@12": (3, 4),
    "mibz2-pipe4@12": (3, 4),
}
# (name, arm, reference, kind): the causal pairs first, the product comparison apart
COMPARISONS = [
    ("C1", "mibz2-pipe3@10", "mibz2-team3@10", "causal"),
    ("C2", "mibz2-pipe4@12", "mibz2-team4@12", "causal"),
    ("P1", "mibz2-pipe4@12", "mibz2@10", "product"),
]
# the noise control: the same plan twice, the default against its forced twin
CONTROL = ("N1", "mibz2-team3@10", "mibz2@10")
# the registered thresholds on the speed ratio's 95% interval
GAIN = 1.02  # a gain is an interval wholly above this
EQUIV = 0.03  # a null is an interval containing 1 and wholly within 1 +/- this
CONTROL_EQUIV = 0.03
# the model's figures (the knee journal, H1): a 3 x 3 team's cycle per block, its leader's
# serial stage, in ms
C0, S0 = 1.056, 0.40


def outcome(g, lo, hi):
    """The registered outcome of a speed ratio's interval"""
    if any(math.isnan(x) for x in (g, lo, hi)):
        return "INCONCLUSIVE (no interval)"
    if lo > GAIN:
        return "GAIN"
    if lo > 1.0:
        return "SMALL GAIN (above 1, not above the threshold)"
    if hi < 1.0:
        return "REGRESSION"
    if 1 - EQUIV <= lo and hi <= 1 + EQUIV:
        return "NULL"
    return "INCONCLUSIVE (the interval spans gain and harm)"


def implied_alpha(speed):
    """The model's alpha at H = 0 from a speed ratio against the non-pipelined 3 x 3:
    speed = C0 / max(S0, alpha * P0), P0 = C0 - S0 (valid only where alpha * P0 > S0)"""
    p0 = C0 - S0
    return C0 / (speed * p0)


def cell(r):
    return f"{r['arm']}@{r['threads']}"


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
    want = list(range(a.rounds))
    lines = ["# The pipelined team: the workstation run's report", ""]
    checks = []
    same = len(hashes) == 1 and len(lengths) == 1 and not bad
    checks.append(("every run the same bytes, none failed", same))
    for c, plan in CELLS.items():
        rs = by.get(c, {})
        plans = {plan_of(threads_of(r.get("thread_cpu", ""))) for r in rs.values()}
        want_plan = (plan[0], plan[0] * (plan[1] - 1))
        ok = sorted(rs) == want and not dupes[c] and plans == {want_plan}
        seen = ", ".join(plan_label(p) for p in sorted(plans)) or "none"
        checks.append((f"`{c}`: rounds 0-{a.rounds - 1} once each, every run {plan[0]} x {plan[1]} (seen: {seen})", ok))
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
    lines += [f"Run: **{verdict}**", "", "## The registered comparisons", ""]
    lines.append("| | arm | against | speed ratio [95% CI] | CPU ratio [95% CI] | outcome | α implied (H = 0) |")
    lines.append("|---|---|---|---|---|---|---|")
    out = {"verdict": verdict, "checks": checks, "comparisons": []}
    for name, arm, ref, kind in COMPARISONS:
        (g, lo, hi), n = ratio(arm, ref)
        (cg, clo, chi), _ = ratio(arm, ref, "cpu")
        oc = outcome(g, lo, hi) if verdict == "VALID" else "NONE (the run is invalid)"
        alpha = implied_alpha(g) if name == "C1" and verdict == "VALID" else math.nan
        lines.append(
            f"| {name} ({kind}) | `{arm}` | `{ref}` | {g:.3f} [{lo:.3f}, {hi:.3f}] | "
            f"{cg:.3f} [{clo:.3f}, {chi:.3f}] | {oc} | {'' if math.isnan(alpha) else f'{alpha:.2f}'} |"
        )
        out["comparisons"].append(
            {"name": name, "kind": kind, "arm": arm, "ref": ref, "speed": [g, lo, hi],
             "cpu": [cg, clo, chi], "rounds": n, "outcome": oc}
        )
    lines += ["", "(CPU ratio: the reference's CPU over the arm's; < 1, the arm spends more.)", ""]
    lines += ["## Every cell (medians)", "", "| cell | plan | wall s | CPU s | peak RSS MB | rounds |", "|---|---|---|---|---|---|"]
    for c in sorted(by):
        rs = list(by[c].values())
        plans = sorted({plan_of(threads_of(r.get("thread_cpu", ""))) for r in rs})
        lines.append(
            f"| `{c}`{'' if c in CELLS else ' (unregistered)'} | {', '.join(plan_label(p) for p in plans)} | "
            f"{statistics.median(r['wall'] for r in rs):.3f} | {statistics.median(r['cpu'] for r in rs):.3f} | "
            f"{statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} |"
        )
    if a.occupancy:
        lines += ["", "## The traces (occupancy, medians over complete rounds)", ""]
        lines += ["| trace | complete rounds | MTF overlapped (same thread) | MTF overlapped (cross thread) | RLE overlapped (cross thread) | blocks |", "|---|---|---|---|---|---|"]
        for path in a.occupancy:
            try:
                o = json.load(open(path))
            except (OSError, ValueError) as e:
                lines.append(f"| {path} | unreadable: {e} | | | | |")
                continue
            for res in o.get("results", []):
                tl = dict(res.get("timeline", {})) if isinstance(res.get("timeline"), dict) else {}
                g = lambda k: tl.get(k, float("nan"))  # noqa: E731
                lines.append(
                    f"| {path.rsplit('/', 1)[-1]} t={res.get('threads')} | {g('complete_rounds')} | "
                    f"{g('pipe.overlap.same_thread')} | {g('pipe.overlap.cross_thread')} | "
                    f"{g('pipe.overlap.rle_cross_thread')} | {g('blocks')} |"
                )
        lines += ["", "No cross-block overlap in a pipelined trace (cross thread 0) means no mechanism verdict (the registration's outcome map)."]
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
    r.add_argument("--rounds", type=int, required=True)
    r.add_argument("--occupancy", nargs="*", default=[])
    r.add_argument("--out")
    a = p.parse_args()
    sys.exit(report(a))


if __name__ == "__main__":
    main()
