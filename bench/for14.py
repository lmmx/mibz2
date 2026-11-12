#!/usr/bin/env python3
"""FOR-14/raw stage 1: the frozen analysis of `just bench-for14`'s grid
(docs/journal/2026-11-01-for14-stage1.md §6).

    python3 bench/for14.py coop-<UTC time>.jsonl [--json OUT.json]

Reads one grid's rounds (arms p1, o1, h1, p4, o4, h4) and reports, per round and paired:
- the walk phase's speed ratios h/o (the primary figures) and o/p (the calibration),
  as walk seconds a worker, o over h (above 1: h faster);
- the whole T-vector phase's h/o, reported only (h's build is a copy of precomputed
  records, so its build time is not a builder's).

Each ratio's mean over rounds of log ratios, with a t interval at 95% widened by the
rounds' lag-1 autocorrelation when it is positive (factor sqrt((1 + r1) / (1 - r1))).

The gates and the decision are the protocol's, fixed before any i9 run:
- G0 every round pinned and placed (`placement_ok`); the grid's round -1 has already
  checked every arm's every block by its hash, and the records row by row.
- G1 calibration: the walk's o1/p1 and o4/p4 each within [0.95, 1.05], their intervals'
  half-widths each below 0.03. Failing G0 or G1 voids the run: no decision.
- The decision, in order:
  1. STOP if the walk's h1/o1 (W1) has its upper bound below 0.774. The bound: the
     same walk's CPU per byte at four workers is at least its CPU per byte at one
     (contention and lower all-core clocks only add), so h's four-worker traversal costs
     at least o's one-worker cost over W1, 1.236 / W1 CPU-s/GB (the decoder's traversal
     plus assembly, report 131 §II). A 0.25 CPU-s/GB saving against o's four-worker
     1.847 needs 1.236 / W1 <= 1.597, so W1 >= 0.774, whatever the memory saving.
     (Break-even, 0.669, is reported.)
  2. STOP if the walk's h4/o4 (W4) has its upper bound below 1.00 (halving the table
     does not repay its instructions at four blocks, in the grid).
  3. EARNS STAGE 2 if W4's lower bound is at least 1.156 (the 13.5% traversal reduction
     that a 0.25 CPU-s/GB saving needs with construction unchanged; report 131 §II).
  4. Otherwise INCONCLUSIVE, which stops this format too unless the reviewers decide
     otherwise (no automatic next variant).
"""

import argparse
import json
import math
import sys
from collections import defaultdict

ARMS = ("p1", "o1", "h1", "p4", "o4", "h4")
STOP_BELOW = 1.00
EARN_AT = 1.156
# the decoder's traversal plus assembly, CPU-s/GB at one and four workers (report 131 §II)
O_W1, O_W4, SAVING = 1.236, 1.847, 0.25
W1_MIN = O_W1 / (O_W4 - SAVING)  # 0.774
W1_EVEN = O_W1 / O_W4  # 0.669
CAL_BAND = (0.95, 1.05)
CAL_HALF = 0.03

# two-sided 95% t quantiles by degrees of freedom (30 and above: 2.042 .. 1.96)
T95 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306,
       9: 2.262, 10: 2.228, 11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131,
       16: 2.120, 17: 2.110, 18: 2.101, 19: 2.093, 20: 2.086, 21: 2.080, 22: 2.074,
       23: 2.069, 24: 2.064, 25: 2.060, 26: 2.056, 27: 2.052, 28: 2.048, 29: 2.045}


def t95(df):
    return T95.get(df, 1.96 if df > 60 else 2.0)


def interval(logs):
    """mean, low, high of exp(log ratios), and the lag-1 autocorrelation used"""
    n = len(logs)
    m = sum(logs) / n
    if n < 3:
        return math.exp(m), float("nan"), float("nan"), float("nan")
    d = [x - m for x in logs]
    var = sum(x * x for x in d) / (n - 1)
    r1 = sum(d[i] * d[i + 1] for i in range(n - 1)) / sum(x * x for x in d) if var > 0 else 0.0
    widen = math.sqrt((1 + r1) / (1 - r1)) if 0 < r1 < 1 else 1.0
    half = t95(n - 1) * math.sqrt(var / n) * widen
    return math.exp(m), math.exp(m - half), math.exp(m + half), r1


def load(path):
    rounds = defaultdict(dict)
    for line in open(path):
        r = json.loads(line)
        rounds[r["round"]][r["arm"]] = r
    return rounds


def analyse(rounds):
    full = [r for _, r in sorted(rounds.items()) if all(a in r for a in ARMS)]
    if not full:
        raise SystemExit("for14: no round holds every arm of " + ",".join(ARMS))
    placed = all(r[a]["pinned"] and r[a]["placement_ok"] for r in full for a in ARMS)

    def ratio(num, den, key):
        # speed ratio num/den = den's seconds over num's
        return interval([math.log(r[den][key] / r[num][key]) for r in full])

    out = {
        "rounds": len(full),
        "placed": placed,
        "walk": {
            "h1/o1": ratio("h1", "o1", "walk_s_a_worker"),
            "h4/o4": ratio("h4", "o4", "walk_s_a_worker"),
            "o1/p1": ratio("o1", "p1", "walk_s_a_worker"),
            "o4/p4": ratio("o4", "p4", "walk_s_a_worker"),
        },
        "whole": {
            "h1/o1": ratio("h1", "o1", "wall_s"),
            "h4/o4": ratio("h4", "o4", "wall_s"),
        },
    }
    cal = []
    for k in ("o1/p1", "o4/p4"):
        m, lo, hi, _ = out["walk"][k]
        ok = CAL_BAND[0] <= m <= CAL_BAND[1] and (hi - lo) / 2 < CAL_HALF
        cal.append((k, ok))
    out["calibration"] = dict(cal)
    if not placed:
        out["decision"] = "VOID: a round was not pinned or not placed (G0)"
    elif not all(ok for _, ok in cal):
        out["decision"] = "VOID: the control's walk does not reproduce production's (G1)"
    else:
        _, _, hi1, _ = out["walk"]["h1/o1"]
        m, lo, hi, _ = out["walk"]["h4/o4"]
        if hi1 < W1_MIN:
            out["decision"] = (f"STOP: h1/o1's upper bound is below {W1_MIN:.3f}"
                               " (no 0.25 CPU-s/GB saving is reachable at four workers)")
        elif hi < STOP_BELOW:
            out["decision"] = "STOP: h4/o4's upper bound is below 1.00"
        elif lo >= EARN_AT:
            out["decision"] = "EARNS STAGE 2: h4/o4's lower bound is at least 1.156"
        else:
            out["decision"] = "INCONCLUSIVE: stops this format unless the reviewers decide otherwise"
    return out


def report(out):
    print(f"{out['rounds']} rounds with every arm; pinned and placed: {out['placed']}")
    for part in ("walk", "whole"):
        print(f"\n{part} phase (speed ratios, mean [95% interval], lag-1 r)")
        for k, (m, lo, hi, r1) in out[part].items():
            print(f"  {k}: {m:.3f} [{lo:.3f}, {hi:.3f}]  r1 {r1:+.2f}")
    print("\ncalibration (G1): " + ", ".join(f"{k} {'holds' if v else 'fails'}"
                                             for k, v in out["calibration"].items()))
    m1 = out["walk"]["h1/o1"][0]
    print(f"W1 against the bounds: {m1:.3f} (0.25 saving needs {W1_MIN:.3f}; break-even {W1_EVEN:.3f})")
    print("decision: " + out["decision"])


def self_test():
    """The decision rules on synthetic rounds (no measurement)"""
    def rounds(h4_speed, o_p=1.0, n=20, h1_speed=0.8):
        rs = {}
        for i in range(n):
            j = 1 + 0.004 * ((i * 7) % 5 - 2)
            rs[i] = {a: {"pinned": True, "placement_ok": True, "wall_s": 1.0,
                         "walk_s_a_worker": 1.0 * j} for a in ARMS}
            for a in ("o1", "o4"):
                rs[i][a]["walk_s_a_worker"] = j / o_p
            rs[i]["h4"]["walk_s_a_worker"] = j / o_p / h4_speed
            rs[i]["h1"]["walk_s_a_worker"] = j / o_p / h1_speed
        return rs
    assert analyse(rounds(0.7))["decision"].startswith("STOP: h4/o4")
    assert analyse(rounds(1.25, h1_speed=0.6))["decision"].startswith("STOP: h1/o1")
    assert analyse(rounds(1.25))["decision"].startswith("EARNS")
    assert analyse(rounds(1.05))["decision"].startswith("INCONCLUSIVE")
    assert analyse(rounds(1.25, o_p=0.9))["decision"].startswith("VOID")
    r = rounds(1.25)
    r[3]["h4"]["placement_ok"] = False
    assert analyse(r)["decision"].startswith("VOID")
    print("for14: self-test passed")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("jsonl", nargs="?")
    ap.add_argument("--json")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return
    if not a.jsonl:
        sys.exit("for14: a grid's .jsonl, or --self-test")
    out = analyse(load(a.jsonl))
    report(out)
    if a.json:
        json.dump(out, open(a.json, "w"), indent=1)


if __name__ == "__main__":
    main()
