#!/usr/bin/env python3
"""The compact30 traversal comparison's analysis (docs/journal/2026-11-02-compact30-traversal-protocol.md).

    python3 bench/c30_traversal.py RUN.jsonl [--json OUT.json] [--listing-ok yes|no]

It reads `c30walk grid`'s records in three stages, and a stage runs only if the one
before it passed:

1. **The schedule (S).** The records must be exactly the registered experiment's. The
   expected record set is derived from the protocol and the block rows recorded in the
   meta record (not re-read from the corpus, whose hash the recipe checks): the
   snake assignment, the round count, every (round, mode, arm, worker, step) with its block
   and rows, the arm order, the barriers' ordering and the CPUs. Missing, extra,
   duplicated or misassigned records, a wrong denominator, an unregistered round count, an
   ordering violation, a failure record, or calibration evidence that is incomplete or
   malformed (S9) make the run invalid, and no ratio is computed.
2. **The validity gates (G)**, on a schedule that passed: control stability (G1),
   simultaneous start (G2), the listing (G3), the allocation (G4) and the calibration-based
   clock-overhead estimate (G5).
3. **The figures and the decision,** on a valid run only.

A dry run (`dry_run` in the meta record, rounds −1, 0 and 1) runs stage 1, G3 and G4, and
suppresses every timing figure, G1, G2, G5 and the decision.

The figures, per round r (rounds ≥ 0; round −1 is untimed), per arm:
- W1 cost: Σ walk thread-CPU ns / Σ rows. w1 = C / P.
- W4 cost: Σ over the four workers of walk thread-CPU ns / Σ rows. w4 = C / P, the
  construction-budget input.
- W4 completion time: Σ over steps of (latest traversal end − earliest traversal start),
  on the monotonic clock. e4 = C / P; above 1 is slower. It is the summed lockstep
  traversal completion time, not the capture's or a decoder's wall time.
- Reported, deciding nothing: the same costs from the monotonic windows (elapsed worker
  time), the running fraction (CPU ÷ monotonic), each worker's CPU time, the start skew
  and the four walks' common overlap in the all-full steps.

Each ratio is the geometric mean of the 20 per-round ratios, with a 95% t interval on the
logs.
"""

import argparse
import json
import math
import statistics
import sys

REGISTERED_ROUNDS = 20  # measured paired rounds in a capture
DRY_ROUNDS = 2  # a dry run's measured rounds (0 and 1), after round -1
SCREEN = 0.865  # w4: the rounded minimum-investment screen (exactly 1 - 0.25/1.847 = 0.86465)
THROUGHPUT_TOL = 1.02  # e4: at most 2% more completion time
STABILITY = 1.05  # G1: the control's per-round max / min
START_SKEW = 0.02  # G2: row-weighted start skew over span
CLOCK_SHARE = 0.005  # G5: the clock calls' share of an arm's summed traversal CPU time
CPU_SLACK_NS = 2000  # a CPU window may exceed its enclosing monotonic one by clock noise
FULL = 899_000
T975 = {19: 2.093}


def b_max(w):
    """The constructor cost ratio that still saves 0.25 CPU-s/GB at traversal ratio w"""
    return 1 + (1.847 * (1 - w) - 0.25) / 0.592


def interval(ratios):
    """Geometric mean and 95% t interval. Only the registered sample size is accepted: an
    insufficient sample never gets a manufactured interval."""
    if len(ratios) != REGISTERED_ROUNDS:
        raise ValueError(f"{len(ratios)} paired rounds; the protocol registers {REGISTERED_ROUNDS}")
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    se = statistics.stdev(logs) / math.sqrt(len(logs))
    t = T975[len(logs) - 1]
    return math.exp(m), math.exp(m - t * se), math.exp(m + t * se)


def load(path):
    meta, walks, calib, failures, other = None, [], [], [], []
    for line in open(path):
        r = json.loads(line)
        k = r.get("kind")
        if k == "meta" and meta is None:
            meta = r
        elif k == "walk":
            walks.append(r)
        elif k == "calibration":
            calib.append(r)
        elif k == "failure":
            failures.append(r)
        else:
            other.append(r)
    return meta, walks, calib, failures, other


def snake(block_rows):
    """The registered four-worker assignment: blocks sorted by rows (descending, then
    index), dealt four to a step, the direction reversing each step"""
    order = sorted(range(len(block_rows)), key=lambda i: (-block_rows[i], i))
    lists = [[], [], [], []]
    for k in range(0, len(order), 4):
        for j, b in enumerate(order[k : k + 4]):
            lists[j if (k // 4) % 2 == 0 else 3 - j].append(b)
    return lists


def arm_order(r):
    return ("P", "C") if r % 2 == 0 else ("C", "P")


def expected_records(meta, rounds):
    """Every walk record the registered schedule produces: key -> (block, rows)"""
    br = meta["block_rows"]
    lists = snake(br)
    want = {}
    for r in range(-1, rounds):
        for a in ("P", "C"):
            for i, n in enumerate(br):
                want[(r, "w1", a, 0, i)] = (i, n)
            for k, l in enumerate(lists):
                for s, b in enumerate(l):
                    want[(r, "w4", a, k, s)] = (b, br[b])
    return want


def check_schedule(meta, walks, failures, other):
    """Stage 1: whether the records are exactly the registered experiment's"""
    c = {}
    c["S0_no_failure"] = not failures
    c["S0_no_unknown_records"] = not other
    br = meta.get("block_rows") or []
    lists = snake(br)
    c["S1_meta"] = (
        len(br) == meta.get("blocks")
        and sum(br) == meta.get("rows")
        and sum(1 for n in br if n >= FULL) == meta.get("full_blocks")
        and meta.get("w4_lists") == lists
        and meta.get("w4_rows") == [sum(br[b] for b in l) for l in lists]
        and len(meta.get("cpus", [])) == 4
        and len(set(meta.get("cpus", []))) == 4
        and len(meta.get("block_raw_groups") or []) == len(br)
    )
    dry = meta.get("dry_run")
    rounds = meta.get("rounds")
    c["S2_rounds"] = (dry is True and rounds == DRY_ROUNDS) or (
        dry is False and rounds == REGISTERED_ROUNDS and meta.get("inject") is None
    )
    if not (c["S1_meta"] and isinstance(rounds, int) and rounds >= 0):
        c["S3_record_set"] = False
        return c
    want = expected_records(meta, rounds)
    got = {}
    dup = False
    for w in walks:
        k = (w["round"], w["mode"], w["arm"], w["worker"], w["step"])
        dup |= k in got
        got[k] = w
    c["S3_record_set"] = (
        not dup
        and set(got) == set(want)
        and all((got[k]["block"], got[k]["rows"]) == want[k] for k in want)
        and all(w["exact"] is True for w in walks)
    )
    if not c["S3_record_set"]:
        return c
    cpus = meta["cpus"]
    c["S4_placement"] = all(
        w["cpu_before"] == w["cpu_after"] == cpus[w["worker"] if w["mode"] == "w4" else 0]
        for w in walks
    )
    c["S5_walk_windows"] = all(
        w["prep_start"]
        <= w["prep_end"]
        <= w["start"]
        <= w["end"]
        <= w["check_start"]
        <= w["check_end"]
        and w["cpu_start"] <= w["cpu_end"]
        and w["cpu_end"] - w["cpu_start"] <= (w["end"] - w["start"]) + CPU_SLACK_NS
        for w in walks
    )
    # W1: one walk after another
    ok = True
    for r in range(-1, rounds):
        for a in ("P", "C"):
            seq = [got[(r, "w1", a, 0, i)] for i in range(len(br))]
            ok &= all(x["check_end"] <= y["prep_start"] for x, y in zip(seq, seq[1:]))
    c["S6_w1_sequence"] = ok
    # W4: each step's barriers, and the order across steps
    start_barrier = completion_barrier = cross = True
    nsteps = max(len(l) for l in lists)
    for r in range(-1, rounds):
        for a in ("P", "C"):
            prev = None
            for s in range(nsteps):
                v = [got[(r, "w4", a, k, s)] for k in range(4) if s < len(lists[k])]
                start_barrier &= max(x["prep_end"] for x in v) <= min(x["start"] for x in v)
                completion_barrier &= max(x["end"] for x in v) <= min(x["check_start"] for x in v)
                if prev is not None:
                    cross &= max(x["end"] for x in prev) <= min(x["prep_start"] for x in v)
                    cross &= all(
                        got[(r, "w4", a, x["worker"], s - 1)]["check_end"] <= x["prep_start"]
                        for x in v
                    )
                prev = v
    c["S7_start_barrier"] = start_barrier
    c["S7_completion_barrier"] = completion_barrier
    c["S7_cross_step"] = cross
    # the passes: rounds in order; in each, W1 then W4, each in the registered arm order,
    # no pass overlapping another
    spans = []
    for r in range(-1, rounds):
        for m in ("w1", "w4"):
            for a in arm_order(r):
                v = [w for k, w in got.items() if k[:3] == (r, m, a)]
                spans.append((min(x["prep_start"] for x in v), max(x["check_end"] for x in v)))
    c["S8_pass_order"] = all(x[1] <= y[0] for x, y in zip(spans, spans[1:]))
    return c


CALIBRATION_N = 20_000  # empty windows a CPU, in each calibration


def _quantiles(q):
    """Five finite, nonnegative, non-decreasing numbers (min, p50, p90, p99, max)"""
    return (
        isinstance(q, list)
        and len(q) == 5
        and all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in q)
        and all(math.isfinite(x) and x >= 0 for x in q)
        and all(a <= b for a, b in zip(q, q[1:]))
    )


def check_calibration(meta, calib):
    """Whether the calibration evidence is complete and well formed, before any median is
    read: exactly one `before` and one `after` batch; in each, exactly the CPUs of
    `meta["cpus"]`, once each; 20,000 windows a CPU; five well-formed quantiles for both
    the CPU and the monotonic clock. Never raises on malformed records."""
    try:
        cpus = meta.get("cpus")
        if not isinstance(cpus, list) or not isinstance(calib, (list, tuple)):
            return False
        if sorted(str(k.get("when")) for k in calib) != ["after", "before"]:
            return False
        for k in calib:
            v = k.get("cpus")
            if not isinstance(v, list) or len(v) != len(cpus):
                return False
            ids = [c.get("cpu") for c in v]
            if sorted(ids, key=str) != sorted(cpus, key=str) or len(set(map(str, ids))) != len(ids):
                return False
            for c in v:
                n = c.get("n")
                if isinstance(n, bool) or n != CALIBRATION_N:
                    return False
                if not (_quantiles(c.get("cpu_ns")) and _quantiles(c.get("wall_ns"))):
                    return False
        return True
    except (AttributeError, TypeError, ValueError):
        return False


def analyse(meta, walks, listing_ok=None, calib=(), failures=(), other=()):
    out = {"dry_run": bool(meta and meta.get("dry_run")), "schedule": {}, "gates": {}}
    if failures:
        out["failures"] = list(failures)
    if meta is None:
        out["schedule"]["S1_meta"] = False
        out["valid"] = False
        out["decision"] = "INVALID: no meta record"
        return out
    sch = out["schedule"] = check_schedule(meta, walks, failures, other)
    sch["S9_calibration"] = check_calibration(meta, calib)
    g = out["gates"]
    g["G3_identity"] = listing_ok
    br = meta.get("block_rows") or []
    if sch.get("S1_meta"):
        lists = meta["w4_lists"]
        rows = meta["w4_rows"]
        braw = meta["block_raw_groups"]
        full = [sum(1 for b in l if br[b] >= FULL) for l in lists]
        g["G4_allocation"] = max(rows) - min(rows) <= max(br) and max(full) - min(full) <= 1
        out["allocation"] = {
            "w4_rows": rows,
            "rows_max_over_min": max(rows) / max(min(rows), 1),
            "rows_max_minus_min": max(rows) - min(rows),
            "largest_block_rows": max(br),
            "w4_blocks": [len(l) for l in lists],
            "w4_full_blocks": full,
            "w4_raw_groups": [sum(braw[b] for b in l) for l in lists],
            "w4_raw_groups_per_mrow": [
                1e6 * sum(braw[b] for b in l) / r for l, r in zip(lists, rows)
            ],
        }
    schedule_ok = all(v is True for v in sch.values())
    if not schedule_ok:
        out["valid"] = False
        out["decision"] = (
            "INVALID: the records or their calibration are not the registered schedule; no figures computed"
        )
        return out
    if meta["dry_run"]:
        out["valid"] = None
        out["note"] = (
            "dry run: the schedule, G3 and G4 are checked; every timing figure, G1, G2, G5 "
            "and the decision are suppressed"
        )
        return out
    rounds = meta["rounds"]
    lists = meta["w4_lists"]
    nsteps = max(len(l) for l in lists)
    by = {}
    for w in walks:
        by.setdefault((w["round"], w["mode"], w["arm"]), []).append(w)
    cpu = lambda w: w["cpu_end"] - w["cpu_start"]  # noqa: E731
    wall = lambda w: w["end"] - w["start"]  # noqa: E731
    per = {}
    for r in range(rounds):
        f = {}
        for a in ("P", "C"):
            w1, w4 = by[(r, "w1", a)], by[(r, "w4", a)]
            rows1 = sum(meta["block_rows"])
            steps = [[x for x in w4 if x["step"] == s] for s in range(nsteps)]
            full_steps = [v for v in steps if len(v) == 4 and all(x["rows"] >= FULL for x in v)]
            fr = sum(sum(x["rows"] for x in v) for v in full_steps)
            span = lambda v: max(x["end"] for x in v) - min(x["start"] for x in v)  # noqa: E731
            f[a] = {
                "w1_cpu_ns_row": sum(cpu(x) for x in w1) / rows1,
                "w4_cpu_ns_row": sum(cpu(x) for x in w4) / rows1,
                "w4_completion_ns": sum(span(v) for v in steps),
                "w1_wall_ns_row": sum(wall(x) for x in w1) / rows1,
                "w4_wall_ns_row": sum(wall(x) for x in w4) / rows1,
                "w1_running_fraction": sum(cpu(x) for x in w1) / sum(wall(x) for x in w1),
                "w4_running_fraction": sum(cpu(x) for x in w4) / sum(wall(x) for x in w4),
                "w4_worker_cpu_ns": [sum(cpu(x) for x in w4 if x["worker"] == k) for k in range(4)],
                "w4_start_skew": sum(
                    sum(x["rows"] for x in v)
                    * (max(x["start"] for x in v) - min(x["start"] for x in v))
                    / span(v)
                    for v in full_steps
                )
                / fr,
                "w4_overlap": sum(
                    sum(x["rows"] for x in v)
                    * max(min(x["end"] for x in v) - max(x["start"] for x in v), 0)
                    / span(v)
                    for v in full_steps
                )
                / fr,
            }
        per[r] = f
    out["rounds"] = per
    for key in ("w1_cpu_ns_row", "w4_cpu_ns_row", "w4_completion_ns"):
        ps = [per[r]["P"][key] for r in per]
        g[f"G1_{key}"] = max(ps) / min(ps) <= STABILITY
    sks = [per[r][a]["w4_start_skew"] for r in per for a in ("P", "C")]
    g["G2_simultaneous_start"] = all(x <= START_SKEW for x in sks)
    out["start_skew_max"] = max(sks)
    # G5: a calibration-based estimate of the clock calls' cost inside each window: the
    # largest median of the empty windows on any CPU, in either calibration (S9 has
    # checked both are complete), against each pass's summed CPU time. An estimate from
    # typical costs, not a bound on every call.
    meds = [c["cpu_ns"][1] for k in calib for c in k["cpus"]]
    o = max(meds)
    out["clock"] = {
        "calibrations": sorted(k["when"] for k in calib),
        "empty_window_cpu_ns_median_max": o,
    }
    shares, short = [], []
    for r in range(rounds):
        for m in ("w1", "w4"):
            for a in ("P", "C"):
                v = by[(r, m, a)]
                shares.append(len(v) * o / sum(cpu(x) for x in v))
                s = [x for x in v if x["rows"] < FULL]
                if s:
                    short.append(len(s) * o / max(sum(cpu(x) for x in s), 1))
    g["G5_clock_overhead"] = max(shares) <= CLOCK_SHARE
    out["clock"].update(
        {
            "share_max": max(shares),
            "short_blocks_share_max": max(short) if short else None,
        }
    )
    rat = {}
    for name, key in (
        ("w1", "w1_cpu_ns_row"),
        ("w4", "w4_cpu_ns_row"),
        ("e4", "w4_completion_ns"),
        ("w1_wall", "w1_wall_ns_row"),
        ("w4_wall", "w4_wall_ns_row"),
    ):
        pt, lo, hi = interval([per[r]["C"][key] / per[r]["P"][key] for r in per])
        rat[name] = {"point": pt, "lo": lo, "hi": hi}
    # sensitivity only, deciding nothing: w4 with the estimate subtracted from every window
    adj = []
    for r in per:
        cp = sum(max(cpu(x) - o, 0) for x in by[(r, "w4", "C")])
        pp = sum(max(cpu(x) - o, 0) for x in by[(r, "w4", "P")])
        adj.append(cp / pp)
    pt, lo, hi = interval(adj)
    out["clock"]["w4_less_median_overhead_sensitivity_only"] = {"point": pt, "lo": lo, "hi": hi}
    out["ratios"] = rat
    w4, e4 = rat["w4"], rat["e4"]
    out["constructor_budget"] = {
        "b_max_at_w4_lo": b_max(w4["lo"]),
        "b_max_at_w4": b_max(w4["point"]),
        "b_max_at_w4_hi": b_max(w4["hi"]),
    }
    valid = all(v is True for v in g.values())
    out["valid"] = valid
    if not valid:
        out["decision"] = "INVALID: a validity gate failed; no decision"
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
    meta, walks, calib, failures, other = load(a.run)
    res = analyse(
        meta, walks, None if a.listing_ok is None else a.listing_ok == "yes", calib, failures, other
    )
    if a.json:
        with open(a.json, "w") as f:
            json.dump(res, f, indent=1)
    for fl in res.get("failures", []):
        print("FAILED RUN:", fl.get("what"), "-", fl.get("detail"))
    print("schedule:", json.dumps(res["schedule"]))
    print("gates:", json.dumps(res["gates"]))
    if "allocation" in res:
        print("allocation:", json.dumps(res["allocation"]))
    if res.get("note"):
        print(res["note"])
        ok = all(v is True for v in res["schedule"].values()) and all(
            v is not False for v in res["gates"].values()
        )
        return 0 if ok else 1
    if "ratios" in res:
        print("ratios (C / P):", json.dumps(res["ratios"]))
        print("constructor budget:", json.dumps(res["constructor_budget"]))
        print("clock:", json.dumps(res["clock"]))
    print("decision:", res["decision"])
    return 0 if res["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
