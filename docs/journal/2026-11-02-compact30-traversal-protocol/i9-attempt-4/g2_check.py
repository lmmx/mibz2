#!/usr/bin/env python3
"""The G2 decomposition of report 183, run on the fourth i9 capture (an unchanged copy of
i9-attempt-3/g2_audit.py, except that a table with no slow sleepers prints nan).

    python3 g2_audit.py c30-traversal.jsonl c30-analysis.json

Recomputes G2 from the raw walk records and checks it against the analysis. Then, for every
all-full W4 step (four walks of at least 899,000 rows), it splits the start skew into barrier
release and each worker's resume. The release is the last arrival at the start barrier: the
latest preparation end among the four workers. It reports the skew by the arm's position in
the round, by arm, by worker, by step and by round, and whether the skew moves the C/P ratios.
Nothing here changes a threshold or the registered outcome."""

import collections
import json
import math
import statistics as st
import sys

FULL = 899_000


def q(xs, p):
    xs = sorted(xs)
    return xs[min(int(p * len(xs)), len(xs) - 1)]


def main():
    records, analysis = sys.argv[1], json.load(open(sys.argv[2]))
    walks = [r for r in map(json.loads, open(records)) if r["kind"] == "walk"]
    w4 = collections.defaultdict(list)
    for w in walks:
        if w["mode"] == "w4" and w["round"] >= 0:
            w4[(w["round"], w["arm"], w["step"])].append(w)
    S = []
    for (rnd, arm, s), v in sorted(w4.items()):
        if len(v) != 4 or not all(x["rows"] >= FULL for x in v):
            continue
        v.sort(key=lambda x: x["worker"])
        starts, ends = [x["start"] for x in v], [x["end"] for x in v]
        span = max(ends) - min(starts)
        release = max(x["prep_end"] for x in v)
        S.append(
            dict(
                round=rnd,
                arm=arm,
                step=s,
                rows=sum(x["rows"] for x in v),
                first=(rnd % 2 == 0) == (arm == "P"),  # even rounds run P first, odd rounds C first
                skew=max(starts) - min(starts),
                span=span,
                ratio=(max(starts) - min(starts)) / span,
                last_arriver=max(v, key=lambda x: x["prep_end"])["worker"],
                first_starter=min(v, key=lambda x: x["start"])["worker"],
                resume=[x["start"] - release for x in v],
                wait=[release - x["prep_end"] for x in v],
                cpu=sum(x["cpu_end"] - x["cpu_start"] for x in v),
                moved=any(x["cpu_before"] != x["cpu_after"] for x in v),
            )
        )
    pos = lambda f: "first " if f else "second"  # noqa: E731
    sleepers = lambda x: [r for k, r in enumerate(x["resume"]) if k != x["last_arriver"]]  # noqa: E731

    print("## 1. The gate, recomputed")
    acc = collections.defaultdict(lambda: [0.0, 0])
    for x in S:
        acc[(x["round"], x["arm"])][0] += x["rows"] * x["ratio"]
        acc[(x["round"], x["arm"])][1] += x["rows"]
    gate = {k: a / b for k, (a, b) in acc.items()}
    dev = max(abs(analysis["rounds"][str(r)][a]["w4_start_skew"] - g) for (r, a), g in gate.items())
    print(
        f"arms over 0.02: {sum(g > 0.02 for g in gate.values())} of {len(gate)}; max {max(gate.values()):.4f}; "
        f"largest difference from the analysis {dev:.1e}; all-full steps {len(S)}; walks that changed CPU {sum(x['moved'] for x in S)}"
    )
    print(f"2% of the median all-full span: {0.02 * st.median(x['span'] for x in S) / 1e3:.1f} us")

    print("\n## 2. By the arm's position in the round, and by arm")
    print(
        "| steps | n | mean | median | share > 0.02 | skew median (us) | skew p90 (us) |\n|---|--:|--:|--:|--:|--:|--:|"
    )
    for name, sel in [
        ("all", lambda x: True),
        ("first arm", lambda x: x["first"]),
        ("second arm", lambda x: not x["first"]),
    ] + [
        (f"{a}, {pos(f).strip()} arm", lambda x, a=a, f=f: x["arm"] == a and x["first"] == f)
        for a in "PC"
        for f in (True, False)
    ]:
        xs = [x for x in S if sel(x)]
        r = [x["ratio"] for x in xs]
        k = [x["skew"] / 1e3 for x in xs]
        print(
            f"| {name} | {len(xs)} | {st.mean(r):.4f} | {st.median(r):.4f} | {sum(v > 0.02 for v in r) / len(r):.3f} | {st.median(k):.1f} | {q(k, 0.9):.1f} |"
        )

    print("\n## 3. Release against resume")
    same = sum(x["first_starter"] == x["last_arriver"] for x in S)
    la = [x["resume"][x["last_arriver"]] / 1e3 for x in S]
    sl = [r / 1e3 for x in S for r in sleepers(x)]
    print(
        f"the first walk to start is the last arrival's: {same} of {len(S)} ({same / len(S):.3f})"
    )
    print(
        f"last arrival, release to start (us): median {st.median(la):.2f}, p90 {q(la, 0.9):.2f}, max {max(la):.1f}"
    )
    print(
        f"the three sleepers, release to start (us): median {st.median(sl):.1f}, p90 {q(sl, 0.9):.1f}, p99 {q(sl, 0.99):.1f}, max {max(sl):.0f}"
    )
    print(f"walks starting before the release: {sum(r < 0 for x in S for r in x['resume'])}")
    print(
        "| arm position | sleepers' median (us) | p90 (us) | 1st / 2nd / 3rd sleeper to wake, medians (us) |\n|---|--:|--:|---|"
    )
    for f in (True, False):
        xs = [x for x in S if x["first"] == f]
        s = [r / 1e3 for x in xs for r in sleepers(x)]
        o = [sorted(sleepers(x)) for x in xs]
        print(
            f"| {pos(f)} | {st.median(s):.1f} | {q(s, 0.9):.1f} | "
            + " / ".join(f"{st.median(v[i] for v in o) / 1e3:.1f}" for i in range(3))
            + " |"
        )
    print(
        "| arm position | steps with a sleeper slower than 20 us | skew ratio median, none slow | skew ratio median, some slow |\n|---|--:|--:|--:|"
    )
    for f in (True, False):
        xs = [x for x in S if x["first"] == f]
        slow = [max(sleepers(x)) > 20_000 for x in xs]
        a = [x["ratio"] for x, s in zip(xs, slow) if not s]
        b = [x["ratio"] for x, s in zip(xs, slow) if s]
        print(
            f"| {pos(f)} | {sum(slow) / len(xs):.3f} | {(st.median(a) if a else float('nan')):.4f} (n={len(a)}) | {(st.median(b) if b else float('nan')):.4f} (n={len(b)}) |"
        )

    print("\n## 4. By worker")
    print(
        "| arm position | last arrival, count by worker 0-3 | sleepers' median resume by worker 0-3 (us) |\n|---|---|---|"
    )
    for f in (True, False):
        xs = [x for x in S if x["first"] == f]
        c = collections.Counter(x["last_arriver"] for x in xs)
        m = [
            st.median(x["resume"][k] / 1e3 for x in xs if k != x["last_arriver"]) for k in range(4)
        ]
        print(
            f"| {pos(f)} | {' / '.join(str(c[k]) for k in range(4))} | {' / '.join(f'{v:.1f}' for v in m)} |"
        )

    print("\n## 5. By step within the arm")
    print("| steps | first arm median | second arm median |\n|---|--:|--:|")
    for lo, hi in ((0, 5), (5, 15), (15, 30), (30, 60)):
        m = [
            st.median(x["ratio"] for x in S if x["first"] == f and lo <= x["step"] < hi)
            for f in (True, False)
        ]
        print(f"| {lo}-{hi - 1} | {m[0]:.4f} | {m[1]:.4f} |")

    print("\n## 6. By round, row-weighted (first arm / second arm, %)")
    acc = collections.defaultdict(lambda: [0.0, 0])
    for x in S:
        acc[(x["round"], x["first"])][0] += x["rows"] * x["ratio"]
        acc[(x["round"], x["first"])][1] += x["rows"]
    print(
        " ".join(
            f"r{r}: {100 * acc[(r, True)][0] / acc[(r, True)][1]:.1f}/{100 * acc[(r, False)][0] / acc[(r, False)][1]:.1f}"
            for r in range(20)
        )
    )

    print("\n## 7. The ratios against the order (even rounds run P first, odd rounds C first)")
    R = analysis["rounds"]
    print("| per-round C/P | P first: mean (min-max) | C first: mean (min-max) |\n|---|---|---|")
    for key in ("w1_cpu_ns_row", "w4_cpu_ns_row", "w4_completion_ns"):
        cell = []
        for p in (0, 1):
            v = [R[str(r)]["C"][key] / R[str(r)]["P"][key] for r in range(20) if r % 2 == p]
            cell.append(f"{st.mean(v):.4f} ({min(v):.4f}-{max(v):.4f})")
        print(f"| {key} | {cell[0]} | {cell[1]} |")
    print("| overlap, row-weighted | first arm mean | second arm mean |\n|---|--:|--:|")
    for a in "PC":
        v = [
            [R[str(r)][a]["w4_overlap"] for r in range(20) if ((r % 2 == 0) == (a == "P")) == f]
            for f in (True, False)
        ]
        print(f"| {a} | {st.mean(v[0]):.4f} | {st.mean(v[1]):.4f} |")
    by = {(x["round"], x["arm"], x["step"]): x for x in S}
    xs, ys = [], []
    for (r, a, s), c in by.items():
        if a == "C" and (r, "P", s) in by:
            p = by[(r, "P", s)]
            xs.append(c["ratio"] - p["ratio"])
            ys.append(math.log(c["cpu"] / p["cpu"]))
    mx, my = st.mean(xs), st.mean(ys)
    b = sum((u - mx) * (v - my) for u, v in zip(xs, ys)) / sum((u - mx) ** 2 for u in xs)
    print(
        f"paired steps (same round, step, blocks and workers): log(C CPU / P CPU) on the skew-ratio difference: n={len(xs)}, "
        f"slope {b:.3f}, r={b * st.pstdev(xs) / st.pstdev(ys):.3f}, implied shift at the mean difference {b * mx:+.5f}"
    )


if __name__ == "__main__":
    main()
