#!/usr/bin/env python3
"""The compact30 construction-cost comparison's analysis
(docs/journal/2026-11-03-compact30-construction-protocol.md, draft 3).

    python3 bench/c30_construction.py RUN.jsonl [--json OUT.json] [--identity-ok yes|no]
        [--rss RSS.json ...]

It reads `c30walk cbuild`'s records in three stages; a stage runs only if the one before
it passed.

1. **The schedule (S0–S10).** The records must be exactly the registered experiment's.
   The expected set is derived from the protocol and the block rows in the meta record:
   - the snake assignment, the round count;
   - every (round, regime, mode, arm, worker, step) with its block and rows;
   - the arm order and the pass order;
   - the barriers' ordering and the CPUs;
   - the calibration evidence;
   - the untimed preamble's per-block records.

   Missing, extra, duplicated or misassigned records, a wrong denominator, an unregistered
   round count, an ordering violation or a failure record make the run invalid, and no
   ratio is computed.
2. **The gates (G1–G10),** on a schedule that passed:
   - stability (G1), simultaneous start (G2), identity (G3, from the recipe), allocation
     (G4), clock calls (G5);
   - exact output (G6), round −1's walker and initialisation checks (G7), production's
     one-pass precondition (G8);
   - memory (G9) and major faults (G10).
3. **The figures and the decision,** on a valid capture only.

A dry run (rounds −1, 0 and 1) runs stage 1, G3, G4 and G6–G10. It suppresses every timing
figure, G1, G2, G5 and the decision.

**The decision** reads the warm regime's four-core ratios over the complete corpus:
- b4: Σ worker build CPU ns ÷ Σ rows, C ÷ P;
- e4: Σ over steps of (latest build end − earliest build start), C ÷ P.

Each is the geometric mean of the 20 per-round ratios, with a 95% t interval on the logs.

| outcome | condition |
|---|---|
| PASS | b4's upper ≤ 1.06714 and e4's upper ≤ 1.02 |
| FAIL | b4's lower > 1.06714 or e4's lower > 1.02 |
| INCONCLUSIVE | otherwise |

Everything else is reported and decides nothing: b1, the cold regime, the full- and
short-block economics, faults, capacities, peak RSS and the slow-path tables.
"""

import argparse
import json
import math
import statistics
import sys

REGISTERED_ROUNDS = 20
DRY_ROUNDS = 2
CEILING = 1.06714  # b4: the construction CPU ceiling (report 186)
COMPLETION_TOL = 1.02  # e4: the completion-time no-harm allowance (report 191)
STABILITY = 1.05  # G1
START_SKEW = 0.02  # G2
CLOCK_SHARE = 0.005  # G5
CPU_SLACK_NS = 2000
FULL = 899_000
GIB = 1 << 30
CALIBRATION_N = 20_000
T975 = {19: 2.093}
# the traversal capture's w4, for the implied saving S (report 185)
W4 = {"point": 0.84047, "lo": 0.83781, "hi": 0.84313}
REGIMES = ("warm", "cold")
PASSES = (("warm", "w1"), ("warm", "w4"), ("cold", "w1"), ("cold", "w4"))
KNOWN = {"meta", "preflight", "onepass", "counters", "calibration", "build", "check0",
         "postflight", "failure"}  # fmt: skip


def interval(ratios):
    """Geometric mean and 95% t interval; only the registered sample size is accepted"""
    if len(ratios) != REGISTERED_ROUNDS:
        raise ValueError(f"{len(ratios)} paired rounds; the protocol registers {REGISTERED_ROUNDS}")
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    se = statistics.stdev(logs) / math.sqrt(len(logs))
    t = T975[len(logs) - 1]
    return math.exp(m), math.exp(m - t * se), math.exp(m + t * se)


def load(path):
    recs = {k: [] for k in KNOWN}
    other = []
    for line in open(path):
        r = json.loads(line)
        k = r.get("kind")
        (recs[k] if k in recs else other).append(r)
    return recs, other


def snake(block_rows):
    order = sorted(range(len(block_rows)), key=lambda i: (-block_rows[i], i))
    lists = [[], [], [], []]
    for k in range(0, len(order), 4):
        for j, b in enumerate(order[k : k + 4]):
            lists[j if (k // 4) % 2 == 0 else 3 - j].append(b)
    return lists


def arm_order(r):
    return ("P", "C") if r % 2 == 0 else ("C", "P")


def expected_records(meta, rounds):
    br = meta["block_rows"]
    lists = snake(br)
    want = {}
    for r in range(-1, rounds):
        for g in REGIMES:
            for a in ("P", "C"):
                for i, n in enumerate(br):
                    want[(r, g, "w1", a, 0, i)] = (i, n)
                for k, l in enumerate(lists):
                    for s, b in enumerate(l):
                        want[(r, g, "w4", a, k, s)] = (b, br[b])
    return want


def _int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def check_schedule(meta, recs, other):
    """Stage 1: whether the records are exactly the registered experiment's"""
    c = {}
    c["S0_no_failure"] = not recs["failure"]
    c["S0_no_unknown_records"] = not other
    br = meta.get("block_rows") or []
    lists = snake(br)
    c["S1_meta"] = (
        len(recs["meta"]) == 1
        and all(_int(n) and n >= 0 for n in br)
        and len(br) == meta.get("blocks")
        and sum(br) == meta.get("rows")
        and sum(1 for n in br if n >= FULL) == meta.get("full_blocks")
        and meta.get("w4_lists") == lists
        and meta.get("w4_rows") == [sum(br[b] for b in l) for l in lists]
        and isinstance(meta.get("cpus"), list)
        and len(meta["cpus"]) == 4
        and len(set(meta["cpus"])) == 4
        and meta.get("regimes") == list(REGIMES)
        and len(meta.get("block_raw_groups") or []) == len(br)
        and len(recs["preflight"]) == 1
        and len(recs["postflight"]) == 1
        and _int(meta.get("memory_estimate"))
    )
    dry = meta.get("dry_run")
    rounds = meta.get("rounds")
    c["S2_rounds"] = (dry is True and rounds == DRY_ROUNDS) or (
        dry is False and rounds == REGISTERED_ROUNDS and meta.get("inject") is None
    )
    if not (c["S1_meta"] and _int(rounds) and rounds >= 0):
        c["S3_record_set"] = False
        return c
    want = expected_records(meta, rounds)
    got = {}
    dup = False
    fields = ("prep_start", "prep_end", "start", "end", "cpu_start", "cpu_end", "check_start",
              "check_end", "minflt", "majflt", "cpu_before", "cpu_after")  # fmt: skip
    for w in recs["build"]:
        k = (w.get("round"), w.get("regime"), w.get("mode"), w.get("arm"), w.get("worker"),
             w.get("step"))  # fmt: skip
        dup |= k in got
        got[k] = w
    c["S3_record_set"] = (
        not dup
        and set(got) == set(want)
        and all((got[k].get("block"), got[k].get("rows")) == want[k] for k in want)
        and all(all(_int(w.get(f)) for f in fields) for w in recs["build"])
    )
    if not c["S3_record_set"]:
        return c
    builds = recs["build"]
    cpus = meta["cpus"]
    c["S4_placement"] = all(
        w["cpu_before"] == w["cpu_after"] == cpus[w["worker"] if w["mode"] == "w4" else 0]
        for w in builds
    )
    c["S5_windows"] = all(
        w["prep_start"] <= w["prep_end"] <= w["start"] <= w["end"] <= w["check_start"]
        <= w["check_end"]
        and w["cpu_start"] <= w["cpu_end"]
        and w["cpu_end"] - w["cpu_start"] <= (w["end"] - w["start"]) + CPU_SLACK_NS
        and w["minflt"] >= 0
        and w["majflt"] >= 0
        for w in builds
    )  # fmt: skip
    ok = True
    for r in range(-1, rounds):
        for g in REGIMES:
            for a in ("P", "C"):
                seq = [got[(r, g, "w1", a, 0, i)] for i in range(len(br))]
                ok &= all(x["check_end"] <= y["prep_start"] for x, y in zip(seq, seq[1:]))
    c["S6_w1_sequence"] = ok
    start_b = completion_b = cross = True
    nsteps = max(len(l) for l in lists)
    for r in range(-1, rounds):
        for g in REGIMES:
            for a in ("P", "C"):
                prev = None
                for s in range(nsteps):
                    v = [got[(r, g, "w4", a, k, s)] for k in range(4) if s < len(lists[k])]
                    start_b &= max(x["prep_end"] for x in v) <= min(x["start"] for x in v)
                    completion_b &= max(x["end"] for x in v) <= min(x["check_start"] for x in v)
                    if prev is not None:
                        cross &= max(x["end"] for x in prev) <= min(x["prep_start"] for x in v)
                        cross &= max(x["check_end"] for x in prev) <= min(x["start"] for x in v)
                        cross &= all(
                            got[(r, g, "w4", a, x["worker"], s - 1)]["check_end"] <= x["prep_start"]
                            for x in v
                            if s - 1 < len(lists[x["worker"]])
                        )
                    prev = v
    c["S7_start_barrier"] = start_b
    c["S7_completion_barrier"] = completion_b
    c["S7_cross_step"] = cross
    spans = []
    for r in range(-1, rounds):
        for g, m in PASSES:
            for a in arm_order(r):
                v = [w for k, w in got.items() if k[:4] == (r, g, m, a)]
                spans.append((min(x["prep_start"] for x in v), max(x["check_end"] for x in v)))
    c["S8_pass_order"] = all(x[1] <= y[0] for x, y in zip(spans, spans[1:]))
    c["S9_calibration"] = check_calibration(meta, recs["calibration"])
    nb = len(br)
    c["S10_preamble"] = (
        sorted(o.get("block") for o in recs["onepass"]) == list(range(nb))
        and sorted(o.get("block") for o in recs["counters"]) == list(range(nb))
        and all(o.get("rows") == br[o["block"]] for o in recs["onepass"] + recs["counters"])
        and sorted((o.get("arm"), o.get("block")) for o in recs["check0"])
        == sorted([("C", b) for b in range(nb)] + [("P", b) for b in range(nb)])
    )
    return c


def _quantiles(q):
    return (
        isinstance(q, list)
        and len(q) == 5
        and all(isinstance(x, (int, float)) and not isinstance(x, bool) for x in q)
        and all(math.isfinite(x) and x >= 0 for x in q)
        and all(a <= b for a, b in zip(q, q[1:]))
    )


def check_calibration(meta, calib):
    """One `before` and one `after` batch; in each, the run's four CPUs once each, 20,000
    windows a CPU, five well-formed quantiles for each clock. Never raises."""
    try:
        cpus = meta.get("cpus")
        if not isinstance(cpus, list):
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
                if not _int(c.get("n")) or c.get("n") != CALIBRATION_N:
                    return False
                if not (_quantiles(c.get("cpu_ns")) and _quantiles(c.get("wall_ns"))):
                    return False
        return True
    except (AttributeError, TypeError, ValueError):
        return False


def check_memory(pre, post):
    """G9, from the recorded numbers (the collector's own verdict is not trusted alone)"""
    try:
        req = pre["required"]
        margin = max(2 * GIB, req // 4)
        pre_ok = (
            pre.get("passed") is True
            and pre.get("injected") is False
            and req >= pre["estimate"]
            and pre["margin"] == margin
            and pre["mem_available"] >= req + margin
            and pre["swap_used"] == 0
            and (not pre["psi_present"]
                 or (pre["psi_some_avg10"] == 0 and pre["psi_full_avg10"] == 0))
        )  # fmt: skip
        post_ok = (
            post["swap_used"] == 0
            and (not post["psi_present"]
                 or (post["psi_some_avg10"] == 0 and post["psi_full_avg10"] == 0))
            and post["pswpin_delta"] == 0
            and post["pswpout_delta"] == 0
        )  # fmt: skip
        return pre_ok, post_ok
    except (KeyError, TypeError):
        return False, False


def analyse(recs, other=(), identity_ok=None, rss=()):
    metas = recs["meta"]
    meta = metas[0] if metas else None
    out = {"dry_run": bool(meta and meta.get("dry_run")), "schedule": {}, "gates": {}}
    if recs["failure"]:
        out["failures"] = list(recs["failure"])
    if meta is None:
        out["schedule"]["S1_meta"] = False
        out["valid"] = False
        out["decision"] = "INVALID: no meta record"
        return out
    sch = out["schedule"] = check_schedule(meta, recs, other)
    g = out["gates"]
    g["G3_identity"] = identity_ok
    br = meta.get("block_rows") or []
    if sch.get("S1_meta"):
        lists = meta["w4_lists"]
        rows = meta["w4_rows"]
        full = [sum(1 for b in l if br[b] >= FULL) for l in lists]
        g["G4_allocation"] = max(rows) - min(rows) <= max(br) and max(full) - min(full) <= 1
        out["allocation"] = {"w4_rows": rows, "w4_blocks": [len(l) for l in lists],
                             "w4_full_blocks": full}  # fmt: skip
    if not all(v is True for v in sch.values()):
        out["valid"] = False
        out["decision"] = (
            "INVALID: the records are not the registered schedule; no figures computed"
        )
        return out
    rounds = meta["rounds"]
    builds = recs["build"]
    g["G6_exact_output"] = all(w.get("exact") is True for w in builds)
    g["G7_round_minus_one"] = all(
        all(v is True for k, v in o.items() if k not in ("kind", "arm", "block")) and len(o) > 3
        for o in recs["check0"]
    )
    g["G8_one_pass"] = all(
        o.get("histogram_equal") is True and o.get("below_2_24") is True for o in recs["onepass"]
    )
    pre, post = recs["preflight"][0], recs["postflight"][0]
    pre_ok, post_ok = check_memory(pre, post)
    g["G9_memory"] = pre_ok and post_ok
    measured = [w for w in builds if w["round"] >= 0]
    g["G10_major_faults"] = all(w["majflt"] == 0 for w in measured)
    out["memory"] = {
        "estimate": pre.get("estimate"),
        "required": pre.get("required"),
        "margin": pre.get("margin"),
        "mem_available_preflight": pre.get("mem_available"),
        "peak_rss_vm_hwm": post.get("vm_hwm"),
        "peak_over_estimate": (post.get("vm_hwm") or 0) / max(pre.get("estimate") or 1, 1),
        "preflight_ok": pre_ok,
        "postflight_ok": post_ok,
    }
    out["faults"] = {
        "major_in_measured_builds": sum(w["majflt"] for w in measured),
        "major_in_round_minus_one": sum(w["majflt"] for w in builds if w["round"] == -1),
        "minor_by_regime_arm": {
            f"{rg}_{a}": sum(w["minflt"] for w in measured if w["regime"] == rg and w["arm"] == a)
            for rg in REGIMES
            for a in ("P", "C")
        },
    }
    caps = {}
    for rg in REGIMES:
        for a in ("P", "C"):
            v = [w for w in builds if w["regime"] == rg and w["arm"] == a]
            caps[f"{rg}_{a}"] = {
                "cap_bytes_max": max((w.get("cap_bytes", 0) for w in v), default=0),
                "usable_bytes_max": max((w.get("usable_bytes", 0) for w in v), default=0),
            }
    out["capacities"] = caps
    if rss:
        by = {}
        for r in rss:
            by.setdefault(r.get("arm"), []).append(r.get("growth_over_baseline"))
        out["peak_rss_processes"] = {
            a: {"median": statistics.median(v), "min": min(v), "max": max(v), "n": len(v)}
            for a, v in by.items()
        }
    if meta["dry_run"]:
        out["valid"] = None
        out["note"] = (
            "dry run: the schedule, G3, G4 and G6-G10 are checked; every timing figure, G1, G2, "
            "G5 and the decision are suppressed"
        )
        return out
    lists = meta["w4_lists"]
    nsteps = max(len(l) for l in lists)
    total_rows = sum(br)
    by = {}
    for w in builds:
        by.setdefault((w["round"], w["regime"], w["mode"], w["arm"]), []).append(w)
    cpu = lambda w: w["cpu_end"] - w["cpu_start"]  # noqa: E731
    wall = lambda w: w["end"] - w["start"]  # noqa: E731
    span = lambda v: max(x["end"] for x in v) - min(x["start"] for x in v)  # noqa: E731

    def figures(w1, w4, keep=lambda b: True):
        rows = sum(n for b, n in enumerate(br) if keep(b))
        w1 = [x for x in w1 if keep(x["block"])]
        w4k = [x for x in w4 if keep(x["block"])]
        steps = [[x for x in w4 if x["step"] == s] for s in range(nsteps)]
        steps = [v for v in steps if v and all(keep(x["block"]) for x in v)]
        full_steps = [v for v in steps if len(v) == 4 and all(x["rows"] >= FULL for x in v)]
        fr = sum(sum(x["rows"] for x in v) for v in full_steps)
        return {
            "w1_cpu_ns_row": sum(cpu(x) for x in w1) / rows,
            "w4_cpu_ns_row": sum(cpu(x) for x in w4k) / rows,
            "w4_completion_ns": sum(span(v) for v in steps),
            "w1_running_fraction": sum(cpu(x) for x in w1) / max(sum(wall(x) for x in w1), 1),
            "w4_running_fraction": sum(cpu(x) for x in w4k) / max(sum(wall(x) for x in w4k), 1),
            "w4_start_skew": (
                sum(sum(x["rows"] for x in v)
                    * (max(x["start"] for x in v) - min(x["start"] for x in v)) / span(v)
                    for v in full_steps) / fr
                if fr else 0.0
            ),
        }  # fmt: skip

    per = {}
    for r in range(rounds):
        per[r] = {
            rg: {a: figures(by[(r, rg, "w1", a)], by[(r, rg, "w4", a)]) for a in ("P", "C")}
            for rg in REGIMES
        }
    out["rounds"] = per
    for rg in REGIMES:
        for key in ("w1_cpu_ns_row", "w4_cpu_ns_row", "w4_completion_ns"):
            ps = [per[r][rg]["P"][key] for r in per]
            g[f"G1_{rg}_{key}"] = max(ps) / min(ps) <= STABILITY
    sks = [per[r][rg][a]["w4_start_skew"] for r in per for rg in REGIMES for a in ("P", "C")]
    g["G2_simultaneous_start"] = all(x <= START_SKEW for x in sks)
    out["start_skew_max"] = max(sks)
    o = max(c["cpu_ns"][1] for k in recs["calibration"] for c in k["cpus"])
    shares, short = [], []
    for r in range(rounds):
        for rg, m in PASSES:
            for a in ("P", "C"):
                v = by[(r, rg, m, a)]
                shares.append(len(v) * o / max(sum(cpu(x) for x in v), 1))
                s = [x for x in v if x["rows"] < FULL]
                if s:
                    short.append(len(s) * o / max(sum(cpu(x) for x in s), 1))
    g["G5_clock_overhead"] = max(shares) <= CLOCK_SHARE
    out["clock"] = {"empty_window_cpu_ns_median_max": o, "share_max": max(shares),
                    "short_blocks_share_max": max(short) if short else None}  # fmt: skip
    rat = {}
    for rg in REGIMES:
        for name, key in (("b1", "w1_cpu_ns_row"), ("b4", "w4_cpu_ns_row"),
                          ("e4", "w4_completion_ns")):  # fmt: skip
            pt, lo, hi = interval([per[r][rg]["C"][key] / per[r][rg]["P"][key] for r in per])
            rat[f"{rg}_{name}"] = {"point": pt, "lo": lo, "hi": hi}
    out["ratios"] = rat
    # the full- and short-block economics (reported only)
    econ = {}
    for label, keep in (("full", lambda b: br[b] >= FULL), ("short", lambda b: br[b] < FULL)):
        if not any(keep(b) for b in range(len(br))):
            continue
        sub = {}
        for r in range(rounds):
            sub[r] = {a: figures(by[(r, "warm", "w1", a)], by[(r, "warm", "w4", a)], keep)
                      for a in ("P", "C")}  # fmt: skip
        e = {}
        for name, key in (("b1", "w1_cpu_ns_row"), ("b4", "w4_cpu_ns_row"),
                          ("e4", "w4_completion_ns")):  # fmt: skip
            try:
                pt, lo, hi = interval([sub[r]["C"][key] / sub[r]["P"][key] for r in sub])
                e[name] = {"point": pt, "lo": lo, "hi": hi}
            except (ZeroDivisionError, ValueError):
                e[name] = None
        econ[label] = e
    out["block_economics"] = econ
    # the slow paths: per-block warm W1 ratio (median over rounds) by deciles of the
    # one-row share (reported only)
    ctr = {o["block"]: o for o in recs["counters"]}
    pb = []
    for b in range(len(br)):
        if br[b] < FULL:
            continue
        rs = []
        for r in range(rounds):
            cp = next(x for x in by[(r, "warm", "w1", "C")] if x["block"] == b)
            pp = next(x for x in by[(r, "warm", "w1", "P")] if x["block"] == b)
            rs.append(cpu(cp) / max(cpu(pp), 1))
        c = ctr[b]
        pb.append((c["rows_scalar"] / br[b], c["rows_fragment"] / br[b], c["wide_promotions"],
                   statistics.median(rs)))  # fmt: skip
    pb.sort()
    dec = []
    for d in range(10):
        part = pb[d * len(pb) // 10 : (d + 1) * len(pb) // 10]
        if part:
            dec.append({
                "one_row_share_max": part[-1][0],
                "fragment_share_mean": statistics.fmean(x[1] for x in part),
                "promotions_mean": statistics.fmean(x[2] for x in part),
                "warm_w1_ratio_median": statistics.median(x[3] for x in part),
                "blocks": len(part),
            })  # fmt: skip
    out["slow_paths_by_one_row_decile"] = dec
    b4, e4 = rat["warm_b4"], rat["warm_e4"]
    out["implied_saving_s"] = {
        f"b4_{bk}_w4_{wk}": 0.592 * (1 - b4[bk]) + 1.847 * (1 - W4[wk])
        for bk in ("lo", "point", "hi")
        for wk in ("lo", "point", "hi")
    }
    valid = all(v is True for v in g.values())
    out["valid"] = valid
    if not valid:
        out["decision"] = "INVALID: a validity gate failed; no decision"
    elif b4["hi"] <= CEILING and e4["hi"] <= COMPLETION_TOL:
        out["decision"] = "PASS"
    elif b4["lo"] > CEILING or e4["lo"] > COMPLETION_TOL:
        out["decision"] = "FAIL"
    else:
        out["decision"] = "INCONCLUSIVE"
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--json")
    ap.add_argument("--identity-ok", choices=["yes", "no"])
    ap.add_argument("--rss", nargs="*", default=[])
    a = ap.parse_args()
    recs, other = load(a.run)
    rss = [json.loads(open(p).read()) for p in a.rss]
    ident = None if a.identity_ok is None else a.identity_ok == "yes"
    res = analyse(recs, other, ident, rss)
    if a.json:
        with open(a.json, "w") as f:
            json.dump(res, f, indent=1)
    for fl in res.get("failures", []):
        print("FAILED RUN:", fl.get("what"), "-", fl.get("detail"))
    print("schedule:", json.dumps(res["schedule"]))
    print("gates:", json.dumps(res["gates"]))
    for k in ("allocation", "memory", "faults"):
        if k in res:
            print(f"{k}:", json.dumps(res[k]))
    if res.get("note"):
        print(res["note"])
        ok = all(v is True for v in res["schedule"].values()) and all(
            v is not False for v in res["gates"].values()
        )
        return 0 if ok else 1
    if "ratios" in res:
        print("ratios (C / P):", json.dumps(res["ratios"]))
        print("clock:", json.dumps(res["clock"]))
    print("decision:", res["decision"])
    return 0 if res["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
