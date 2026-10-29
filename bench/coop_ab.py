"""The #77 experiment's paired A/B: each `r` arm against its frozen twin (research only).

The `r` arms (examples/coop.rs, `traverse_room`) are the frozen cooperative kernel (c2620b8)
with one change: each thread's walk runs in room passes, as production's room walk does. The
build is the frozen one, instruction for instruction. This reads the grid's rounds
(COOP_JSONL, written by bench/coop.sh) and pairs, round by round, each `rX` with `X`:

- **MB/s:** the arm's paired ratio, mean ±95% (Student's t), its range, and the rounds won;
- **walk:** the walk phase's paired ratio, a worker's walk seconds X over rX;
- **build, the control:** the same ratio for the build, which the change does not touch. A
  build ratio away from 1 shows drift or interference, not the change;
- **against p4:** each arm's paired MB/s against production's, where p4 ran.

The verdicts are the journal's (§12), read mechanically. It writes coop-ab-<UTC time>.md.

  python3 bench/coop_ab.py [coop-<UTC time>.jsonl]   (default: the newest in this directory)
"""

import datetime
import glob
import json
import math
import os
import statistics
import sys

# Student's t, 97.5% quantile, df 1-30 (as examples/coop.rs's t975)
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip
# the predictions registered in the journal (§12.2): (low, high) of the paired MB/s ratio
PREDICTED = {"r4x1": (1.05, 1.15), "r3x3": (1.04, 1.14)}


def interval(x):
    """mean and half-width of its 95% interval"""
    n = len(x)
    if n < 2:
        return (x[0] if x else math.nan), math.nan
    t = T975[n - 2] if n - 1 <= 30 else 1.96
    return statistics.fmean(x), t * statistics.stdev(x) / math.sqrt(n)


def paired(rounds, a, b, key):
    """b's figure over a's, round by round (key: the field and whether lower is better)"""
    field, lower_better = key
    out = []
    for r, arms in sorted(rounds.items()):
        if a in arms and b in arms:
            x, y = arms[a][field], arms[b][field]
            out.append(x / y if lower_better else y / x)
    return out


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else None
    if path is None:
        found = sorted(glob.glob("coop-*.jsonl"), key=os.path.getmtime)
        found = [f for f in found if not f.startswith(("coop-counters", "coop-ab", "coop-profile"))]
        if not found:
            sys.exit("coop-ab: no coop-*.jsonl here; run bench/coop.sh first")
        path = found[-1]
    rounds = {}
    for line in open(path):
        d = json.loads(line)
        rounds.setdefault(d["round"], {})[d["arm"]] = d
    arms = sorted({a for r in rounds.values() for a in r})
    pairs = [(a[1:], a) for a in arms if a.startswith("r") and a[1:] in arms]
    if not pairs:
        sys.exit(f"coop-ab: no rX arm with its X in {path}")
    out = []
    w = out.append
    w(f"# The #77 room-walk A/B ({os.path.basename(path)})")
    w("")
    w(f"- **Rounds:** {len(rounds)}; arms {', '.join(arms)}")
    w("- **The change:** each cooperative thread's walk in room passes (`traverse_room`); the")
    w("  build and everything else the frozen kernel's (c2620b8)")
    w("")
    w("## Each r arm against its frozen twin, paired round by round")
    w("")
    w(
        "| arm | vs | MB/s, medians | MB/s ratio: mean ±95% (min-max) | rounds won | walk ratio | build ratio (control) | predicted |"
    )
    w("|---|---|--:|--:|--:|--:|--:|--:|")
    verdicts = []
    for base, arm in pairs:
        mb = paired(rounds, base, arm, ("mb_s", False))
        wk = paired(rounds, base, arm, ("walk_s_a_worker", True))
        bd = paired(rounds, base, arm, ("build_s_a_worker", True))
        m, h = interval(mb)
        wm, wh = interval(wk)
        bm, bh = interval(bd)
        med = lambda a: statistics.median(r[a]["mb_s"] for r in rounds.values() if a in r)  # noqa: E731
        lo, hi = PREDICTED.get(arm, (math.nan, math.nan))
        said = f"{lo:.2f}-{hi:.2f}" if arm in PREDICTED else "(none)"
        w(
            f"| {arm} | {base} | {med(base):.0f} → {med(arm):.0f} | {m:.3f} ±{h:.3f} ({min(mb):.3f}-{max(mb):.3f}) | {sum(x > 1 for x in mb)}/{len(mb)} | {wm:.3f} ±{wh:.3f} | {bm:.3f} ±{bh:.3f} | {said} |"
        )
        gain = (
            "a gain: its interval excludes 1"
            if m - h > 1
            else "no gain shown: its interval reaches 1"
            if m + h >= 1
            else "a loss: its interval is below 1"
        )
        inside = (
            f"the mean is {'inside' if lo <= m <= hi else 'outside'} the predicted {said}"
            if arm in PREDICTED
            else "no prediction registered"
        )
        control = (
            "the control holds (build within ±3%)"
            if abs(bm - 1) <= 0.03
            else f"**the control moved ({bm:.3f}): drift or interference; read the gain with care**"
        )
        verdicts.append(f"- **{arm}:** {gain}; {inside}; {control}")
    if "p4" in arms:
        w("")
        w("## Against production (p4), paired")
        w("")
        w("| arm | MB/s ratio to p4: mean ±95% (min-max) |")
        w("|---|--:|")
        for a in arms:
            if a == "p4":
                continue
            r = paired(rounds, "p4", a, ("mb_s", False))
            if r:
                m, h = interval(r)
                w(f"| {a} | {m:.3f} ±{h:.3f} ({min(r):.3f}-{max(r):.3f}) |")
    w("")
    w("## §12's verdicts, read mechanically (the journal interprets them)")
    w("")
    out.extend(verdicts)
    w("")
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    md = f"coop-ab-{stamp}.md"
    open(md, "w").write("\n".join(out) + "\n")
    print("\n".join(out))
    print(f"coop-ab: wrote {md}", file=sys.stderr)


if __name__ == "__main__":
    main()
