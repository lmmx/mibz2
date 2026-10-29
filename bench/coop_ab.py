"""The cooperative kernel's paired A/Bs: each changed arm against its twin (research only).

Each change is one arm kind against the kind it changes, round by round in one grid
(COOP_JSONL, written by bench/coop.sh):

- **`rX` against `X` (#77, journal §12):** each thread's walk in room passes
  (`traverse_room`); the changed phase is the walk, the control the build;
- **`qX` against `rX` (#78's bound, journal §13):** the build given each range's counts
  from the block's checkpoints, made outside the timing (`coop_build_given`); the changed
  phase is the build, the control the walk;
- **with the MTF stage timed (COOP_WITH_MTF=1, #78's stage 2, journal §14):** `nX` against
  `rX`, the net (the checkpoints made by the worker's own `mtf_cp`, then the given build),
  and `nX` against `qX`, the checkpoints' cost (the same build, its checkpoints free or
  made); there each pair's seconds a worker are given per phase too, so the build's saving
  and the MTF stage's cost are read in seconds, not only as a ratio diluted by the MTF.

For each pair:
- **MB/s:** the paired ratio, mean ±95% (Student's t), its range, and the rounds won;
- **the changed phase:** its paired ratio, a worker's seconds in it, twin over arm;
- **the control:** the same for the phase the change does not touch. A control away from 1
  shows drift or interference, not the change;
- **against p4:** each arm's paired MB/s against production's, where p4 ran.

The verdicts are the journal's, read mechanically. It writes coop-ab-<UTC time>.md.
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
# the predictions registered in the journal (§12.2, §13.2, §14.2): (low, high) of the
# paired MB/s ratio, by (arm, twin); the stage-2 ones apply to a run with the MTF timed
PREDICTED = {
    ("r4x1", "4x1"): (1.05, 1.15),
    ("r3x3", "3x3"): (1.04, 1.14),
    ("q4x1", "r4x1"): (0.98, 1.02),
    ("q3x3", "r3x3"): (1.06, 1.14),
    # #81's cooperative test (§16.2)
    ("x3x3", "q3x3"): (1.02, 1.06),
    ("x4x2", "q4x2"): (1.01, 1.05),
    ("x4x1", "q4x1"): (0.99, 1.02),
}
PREDICTED_WITH_MTF = {
    ("q3x3", "r3x3"): (1.02, 1.05),
    ("n3x3", "r3x3"): (1.02, 1.05),
    ("n3x3", "q3x3"): (0.99, 1.01),
    ("n4x1", "r4x1"): (0.98, 1.02),
    # #78's spill fix (§15.2)
    ("m3x3", "r3x3"): (1.03, 1.06),
    ("f3x3", "r3x3"): (1.04, 1.06),
    ("m3x3", "n3x3"): (1.00, 1.03),
    ("f3x3", "n3x3"): (1.01, 1.03),
    ("f3x3", "q3x3"): (0.99, 1.01),
    ("f4x1", "r4x1"): (0.98, 1.02),
    # the integration's PR 0 (§17.2): the fused build with made checkpoints, MTF timed
    ("y3x3", "f3x3"): (1.05, 1.09),
    ("y4x2", "f4x2"): (1.05, 1.10),
    ("y3x3", "x3x3"): (0.98, 1.00),
    ("y4x2", "x4x2"): (0.98, 1.00),
}
# an arm kind: [(its twin's prefix, the phase it changes, its control, what it is)]
CHANGES = {
    "r": [("", "walk", "build", "#77: the walk in room passes")],
    "q": [("r", "build", "walk", "#78's bound: the build given each range's counts")],
    "n": [
        ("r", "build", "walk", "#78's net: the checkpoints made by the MTF, then the build"),
        ("q", "mtf", "walk", "#78's checkpoint cost: the same build, its checkpoints made"),
    ],
    "m": [
        ("r", "build", "walk", "#78's net with the spill fix (`mtf_cp2`)"),
        ("n", "mtf", "walk", "the spill fix: `mtf_cp2` against `mtf_cp`, the same build"),
    ],
    "x": [("q", "build", "walk", "#81: the given build in one pass, L's byte taken from L")],
    "f": [
        ("r", "build", "walk", "#78's net with the spill fix, store order pinned (`mtf_cp2p`)"),
        ("n", "mtf", "walk", "the spill fix, pinned: `mtf_cp2p` against `mtf_cp`, the same build"),
        ("q", "mtf", "walk", "the fix's checkpoint cost: `mtf_cp2p` against free checkpoints"),
        ("m", "mtf", "walk", "the store order alone: `mtf_cp2p` against `mtf_cp2`"),
    ],
    "y": [
        ("f", "build", "walk", "PR 0: the fused build, its checkpoints made by `mtf_cp2p`"),
        ("x", "mtf", "walk", "PR 0: the fused build's checkpoints made, not free"),
    ],
}


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
    with_mtf = any(
        r.get("mtf_s_a_worker", 0) > 0 for arms_ in rounds.values() for r in arms_.values()
    )
    predicted = PREDICTED_WITH_MTF if with_mtf else PREDICTED
    pairs = [
        (twin + a[1:], a, changed, control, what)
        for a in arms
        for twin, changed, control, what in CHANGES.get(a[0], [])
        if twin + a[1:] in arms
    ]
    if not pairs:
        sys.exit(f"coop-ab: no changed arm with its twin in {path}")
    out = []
    w = out.append
    w(f"# The cooperative kernel's A/Bs ({os.path.basename(path)})")
    w("")
    w(f"- **Rounds:** {len(rounds)}; arms {', '.join(arms)}")
    w(
        f"- **The MTF stage:** {'timed, in every arm (COOP_WITH_MTF=1)' if with_mtf else 'outside the timing'}"
    )
    for base, arm, changed, control, what in pairs:
        w(f"- **{arm} against {base}:** {what}; changed phase the {changed}, control the {control}")
    w("")
    w("## Each changed arm against its twin, paired round by round")
    w("")
    w(
        "| arm | vs | MB/s, medians | MB/s ratio: mean ±95% (min-max) | rounds won | changed phase ratio | control phase ratio | predicted |"
    )
    w("|---|---|--:|--:|--:|--:|--:|--:|")
    verdicts = []
    for base, arm, changed, control, _ in pairs:
        mb = paired(rounds, base, arm, ("mb_s", False))
        wk = paired(rounds, base, arm, (f"{changed}_s_a_worker", True))
        bd = paired(rounds, base, arm, (f"{control}_s_a_worker", True))
        m, h = interval(mb)
        wm, wh = interval(wk)
        bm, bh = interval(bd)
        med = lambda a: statistics.median(r[a]["mb_s"] for r in rounds.values() if a in r)  # noqa: E731
        lo, hi = predicted.get((arm, base), (math.nan, math.nan))
        said = f"{lo:.2f}-{hi:.2f}" if (arm, base) in predicted else "(none)"
        w(
            f"| {arm} | {base} | {med(base):.0f} → {med(arm):.0f} | {m:.3f} ±{h:.3f} ({min(mb):.3f}-{max(mb):.3f}) | {sum(x > 1 for x in mb)}/{len(mb)} | {changed} {wm:.3f} ±{wh:.3f} | {control} {bm:.3f} ±{bh:.3f} | {said} |"
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
            if (arm, base) in predicted
            else "no prediction registered"
        )
        control = (
            f"the control holds ({control} within ±3%)"
            if abs(bm - 1) <= 0.03
            else f"**the control moved ({bm:.3f}): drift or interference; read the gain with care**"
        )
        verdicts.append(f"- **{arm} against {base}:** {gain}; {inside}; {control}")
    if with_mtf:
        w("")
        w("## A worker's seconds by phase, paired: twin minus arm (+ is the arm's saving)")
        w("")
        w("| arm | vs | MTF | build | walk | all three | the MTF's share of the twin's |")
        w("|---|---|--:|--:|--:|--:|--:|")
        for base, arm, _, _, _ in pairs:
            cols = []
            for f in ("mtf", "build", "walk"):
                d = [
                    r[base][f"{f}_s_a_worker"] - r[arm][f"{f}_s_a_worker"]
                    for r in rounds.values()
                    if base in r and arm in r
                ]
                cols.append(d)
            tot = [sum(x) for x in zip(*cols)]
            share = statistics.median(
                r[base]["mtf_s_a_worker"]
                / (
                    r[base]["mtf_s_a_worker"]
                    + r[base]["build_s_a_worker"]
                    + r[base]["walk_s_a_worker"]
                )
                for r in rounds.values()
                if base in r
            )
            ms = lambda d: "{:+.2f} ±{:.2f} ms".format(*(1e3 * x for x in interval(d)))  # noqa: E731
            w(
                f"| {arm} | {base} | {ms(cols[0])} | {ms(cols[1])} | {ms(cols[2])} | {ms(tot)} | {share:.2f} |"
            )
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
    w("## The registered verdicts, read mechanically (the journal interprets them)")
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
