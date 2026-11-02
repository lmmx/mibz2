#!/usr/bin/env python3
"""The compact30 traversal comparison's analysis (docs/journal/2026-11-02-compact30-traversal-protocol.md).

    python3 bench/c30_traversal.py RUN.jsonl [--json OUT.json] [--listing-ok yes|no]

Reads `c30walk grid`'s records and computes the validity gates and the registered decision:
- G0, outputs: every walk is exact and complete. The harness asserts each walk, so a run
  that finished has none wrong; here every (round, mode, arm, block) must be present once.
- G1, stability: control P's per-round figure, max / min ≤ 1.05, for the W1 cost, the W4
  cost and the W4 elapsed time.
- G2, simultaneous start: at W4, in every step whose four blocks are all full blocks, the
  four walks' start skew (latest start − earliest start) over the step's span (earliest
  start to latest end). Its row-weighted mean must be ≤ 0.02 in every arm and round. This
  checks that the barrier releases the four workers together. It does not ask their walks
  to take equally long: a candidate's uneven walks are a result, which the elapsed figure
  e4 carries, not an invalid run.
- G3, identity: the recipe passes `--listing-ok`, whether `room::<false>`'s instructions
  match the frozen listing. The corpus hash is checked by the recipe before the run.
- G4, allocation: every block appears once in the W4 lists; blocks are indivisible, so
  the per-worker rows may differ by at most the largest block's rows, and the per-worker
  full-block counts by at most one. The ratio, the full-block counts and each worker's
  RAW groups (the candidate's exceptional path) are reported as the mix.
- Path, measurement-path checks: every walk's preparation ends before its walk starts;
  in every W4 step, every walk starts after every worker's preparation has ended (the
  barrier); each W4 worker ran on its own listed CPU; W1 ran on the first.

The figures, per round r (rounds ≥ 0; round −1 is untimed):
- W1 cost: Σ walk ns / Σ rows, per arm. w1 = C / P.
- W4 cost: Σ over workers of walk ns / Σ rows. w4 = C / P, the construction-budget input.
- W4 elapsed: Σ over steps of (latest end − earliest start), from the barrier's release
  to the step's last completion. e4 = C / P, completed-work time; a ratio above 1 is
  slower.
- Reported, deciding nothing: each W4 worker's summed walk time, and the four walks'
  common overlap in the all-full steps.

Each ratio is the geometric mean of the per-round ratios, with a 95% t interval on the
logs. The decision is the registered one (§5 of the protocol). A dry run (`dry_run` in the
meta record) checks the gates that need no timing (G0, G3, G4, path) and suppresses every
timing figure and the decision.
"""

import argparse
import json
import math
import statistics
import sys

SCREEN = 0.865  # w4: the 0.25 CPU-s/GB minimum-investment screen at an unchanged constructor
THROUGHPUT_TOL = 1.02  # e4: completed-work time no more than 2% worse
T975 = {
    1: 12.706,
    2: 4.303,
    3: 3.182,
    4: 2.776,
    5: 2.571,
    6: 2.447,
    7: 2.365,
    8: 2.306,
    9: 2.262,
    10: 2.228,
    11: 2.201,
    12: 2.179,
    13: 2.160,
    14: 2.145,
    15: 2.131,
    16: 2.120,
    17: 2.110,
    18: 2.101,
    19: 2.093,
    20: 2.086,
    21: 2.080,
    22: 2.074,
    23: 2.069,
    24: 2.064,
    25: 2.060,
    29: 2.045,
    30: 2.042,
}
FULL = 899_000


def b_max(w):
    """The constructor cost ratio that still saves 0.25 CPU-s/GB at traversal ratio w"""
    return 1 + (1.847 * (1 - w) - 0.25) / 0.592


def interval(ratios):
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    if len(logs) < 2:
        return math.exp(m), math.exp(m), math.exp(m)
    se = statistics.stdev(logs) / math.sqrt(len(logs))
    t = T975.get(len(logs) - 1, 1.96)
    return math.exp(m), math.exp(m - t * se), math.exp(m + t * se)


def load(path):
    meta, walks = None, []
    for line in open(path):
        r = json.loads(line)
        if r["kind"] == "meta":
            meta = r
        else:
            walks.append(r)
    return meta, walks


def analyse(meta, walks, listing_ok=None):
    out = {"dry_run": meta["dry_run"], "gates": {}, "path": {}}
    rounds = meta["rounds"]
    nblocks = meta["blocks"]
    g = out["gates"]
    # G0: completeness (the harness asserts exactness)
    seen = {}
    for w in walks:
        k = (w["round"], w["mode"], w["arm"], w["block"])
        seen[k] = seen.get(k, 0) + 1
    want = {
        (r, m, a, b)
        for r in range(-1, rounds)
        for m in ("w1", "w4")
        for a in ("P", "C")
        for b in range(nblocks)
    }
    g["G0_outputs"] = (
        all(w["exact"] for w in walks) and set(seen) == want and all(v == 1 for v in seen.values())
    )
    # G4: allocation. Blocks are indivisible, so the bound is one block: the per-worker rows
    # may differ by at most the largest block, and the full-block counts by at most one.
    lists = meta["w4_lists"]
    flat = sorted(b for l in lists for b in l)
    rows = meta["w4_rows"]
    brows = meta.get("block_rows")
    braw = meta.get("block_raw_groups")
    full = [sum(1 for b in l if brows[b] >= FULL) for l in lists] if brows else None
    g["G4_allocation"] = (
        flat == list(range(nblocks))
        and (brows is None or rows == [sum(brows[b] for b in l) for l in lists])
        and max(rows) - min(rows) <= (max(brows) if brows else 0)
        and (full is None or max(full) - min(full) <= 1)
    )
    out["allocation"] = {
        "w4_rows": rows,
        "rows_max_over_min": max(rows) / max(min(rows), 1),
        "rows_max_minus_min": max(rows) - min(rows),
        "largest_block_rows": max(brows) if brows else None,
        "w4_blocks": [len(l) for l in lists],
        "w4_full_blocks": full,
        "w4_raw_groups": [sum(braw[b] for b in l) for l in lists] if braw else None,
        "w4_raw_groups_per_mrow": (
            [1e6 * sum(braw[b] for b in l) / r for l, r in zip(lists, rows)] if braw else None
        ),
    }
    # G3: identity (from the recipe)
    g["G3_identity"] = listing_ok
    # path checks
    p = out["path"]
    p["prep_before_walk"] = all(
        w["prep_end"] <= w["start"] and w["prep_start"] <= w["prep_end"] and w["start"] <= w["end"]
        for w in walks
    )
    steps = {}
    for w in walks:
        if w["mode"] == "w4":
            steps.setdefault((w["round"], w["arm"], w["step"]), []).append(w)
    p["barrier"] = all(
        min(x["start"] for x in v) >= max(x["prep_end"] for x in v) for v in steps.values()
    )
    cpus = meta["cpus"]
    p["cpus"] = all(
        w["cpu"] == (cpus[w["worker"]] if w["mode"] == "w4" else cpus[0]) for w in walks
    )
    p["w4_steps_lockstep"] = all(len({x["worker"] for x in v}) == len(v) for v in steps.values())
    if meta["dry_run"]:
        out["note"] = (
            "dry run: the measurement path and the untimed gates are checked; every "
            "timing figure, G1, G2 and the decision are suppressed"
        )
        return out
    # the figures
    per = {}
    for r in range(rounds):
        f = {}
        for a in ("P", "C"):
            w1 = [w for w in walks if w["round"] == r and w["mode"] == "w1" and w["arm"] == a]
            w4 = [w for w in walks if w["round"] == r and w["mode"] == "w4" and w["arm"] == a]
            rows1 = sum(w["rows"] for w in w1)
            rows4 = sum(w["rows"] for w in w4)
            f[a] = {
                "w1_ns_row": sum(w["end"] - w["start"] for w in w1) / rows1,
                "w4_cost_ns_row": sum(w["end"] - w["start"] for w in w4) / rows4,
                "w4_elapsed_ns": sum(
                    max(x["end"] for x in v) - min(x["start"] for x in v)
                    for (rr, aa, _), v in steps.items()
                    if rr == r and aa == a
                ),
            }
            sk, wt = 0.0, 0
            for (rr, aa, _), v in steps.items():
                if rr == r and aa == a and len(v) == 4 and all(x["rows"] >= FULL for x in v):
                    span = max(x["end"] for x in v) - min(x["start"] for x in v)
                    skew = max(x["start"] for x in v) - min(x["start"] for x in v)
                    rws = sum(x["rows"] for x in v)
                    sk += rws * skew / span
                    wt += rws
            f[a]["w4_start_skew"] = sk / wt if wt else None
            # reported, deciding nothing: each worker's summed walk time, and the four walks'
            # common overlap (earliest end − latest start) over the span, row-weighted over
            # the all-full steps
            f[a]["w4_worker_ns"] = [
                sum(w["end"] - w["start"] for w in w4 if w["worker"] == k)
                for k in range(len(meta["cpus"][:4]))
            ]
            ov, wt = 0.0, 0
            for (rr, aa, _), v in steps.items():
                if rr == r and aa == a and len(v) == 4 and all(x["rows"] >= FULL for x in v):
                    span = max(x["end"] for x in v) - min(x["start"] for x in v)
                    common = min(x["end"] for x in v) - max(x["start"] for x in v)
                    rws = sum(x["rows"] for x in v)
                    ov += rws * max(common, 0) / span
                    wt += rws
            f[a]["w4_overlap"] = ov / wt if wt else None
        per[r] = f
    out["rounds"] = per
    for key in ("w1_ns_row", "w4_cost_ns_row", "w4_elapsed_ns"):
        ps = [per[r]["P"][key] for r in per]
        g[f"G1_{key}"] = max(ps) / min(ps) <= 1.05
    sks = [per[r][a]["w4_start_skew"] for r in per for a in ("P", "C")]
    g["G2_simultaneous_start"] = all(x is not None and x <= 0.02 for x in sks)
    out["start_skew_max"] = max((x for x in sks if x is not None), default=None)
    rat = {}
    for name, key in (("w1", "w1_ns_row"), ("w4", "w4_cost_ns_row"), ("e4", "w4_elapsed_ns")):
        pt, lo, hi = interval([per[r]["C"][key] / per[r]["P"][key] for r in per])
        rat[name] = {"point": pt, "lo": lo, "hi": hi}
    out["ratios"] = rat
    w4, e4 = rat["w4"], rat["e4"]
    out["constructor_budget"] = {
        "b_max_at_w4_lo": b_max(w4["lo"]),
        "b_max_at_w4": b_max(w4["point"]),
        "b_max_at_w4_hi": b_max(w4["hi"]),
    }
    valid = all(v is True for v in g.values()) and all(p.values())
    out["valid"] = valid
    if not valid:
        out["decision"] = "INVALID: no decision"
    elif w4["hi"] <= SCREEN and e4["hi"] <= THROUGHPUT_TOL:
        out["decision"] = "ADVANCE"
    elif w4["lo"] > SCREEN or e4["lo"] > THROUGHPUT_TOL:
        out["decision"] = "STOP"
    else:
        out["decision"] = "INCONCLUSIVE"
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--json")
    ap.add_argument("--listing-ok", choices=["yes", "no"])
    a = ap.parse_args()
    meta, walks = load(a.run)
    res = analyse(meta, walks, None if a.listing_ok is None else a.listing_ok == "yes")
    if a.json:
        with open(a.json, "w") as f:
            json.dump(res, f, indent=1)
    print("gates:", json.dumps(res["gates"]))
    print("measurement path:", json.dumps(res["path"]))
    print("allocation:", json.dumps(res["allocation"]))
    if res["dry_run"]:
        print(res["note"])
        return (
            0
            if all(v is not False for v in res["gates"].values()) and all(res["path"].values())
            else 1
        )
    print("ratios (C / P):", json.dumps(res["ratios"]))
    print("constructor budget:", json.dumps(res["constructor_budget"]))
    print("decision:", res["decision"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
