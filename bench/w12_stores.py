#!/usr/bin/env python3
"""W12's store-traffic diagnostic (`just bench-w12-stores`): is store traffic the lever behind
12-byte rules (#506)? Reads the records of `bench/counters.py run --events stores` (or
`stores-alt`), master's decoder (the reference) and `mibz2-w12` (the candidate) at 10 threads
on the Wikidata sample, and applies the rule posted on #145 before any data. Standard library
only.

    w12_stores.py COUNTERS.jsonl --ref A --cand B [--threads 10] [--reps 10] [--events stores]

**The statistic.** For each event, each rep's count a byte of output, the candidate's over the
reference's (the arms alternate in order from rep to rep, so each rep is a pair). The ratio is
the geometric mean of the reps' paired ratios, with its 95% Student t interval on the logs.

**The rule** (posted before the data): store traffic is the lever if the W12/lazy ratios of
`l2_lines_out.non_silent` and `resource_stalls.sb` are both at most 0.977, each with its 95%
interval entirely below 1. Otherwise: NOT ESTABLISHED. Under `stores-alt` (perf here has no
`l2_lines_out.non_silent`, as the recipe found from `perf list`), `l2_trans.l2_wb` (L1D
write-backs into L2) stands in for it, and the VERDICT line says so.

**VOID** (exit 1, a VERDICT: VOID line), checked before anything is read:
- the counters file missing, a line not a record, or fewer than 2 reps;
- any (arm, threads, rep, group) missing, duplicated or outside the roster;
- a record's events not the set's group;
- a run whose perf or decode failed;
- any event not counted, unsupported or multiplexed (enabled below 100%);
- no output size, or the arms' outputs differing;
- any event counted as zero in a rep (its ratio undefined).

Exit 0 with a verdict (either), 1 with VOID.
"""

import argparse
import json
import math
import os
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import counters  # noqa: E402

# the rule's two events, by set: the L2 write traffic and the store-buffer stalls
RULE_EVENTS = {
    "stores": ("l2_lines_out.non_silent", "resource_stalls.sb"),
    "stores-alt": ("l2_trans.l2_wb", "resource_stalls.sb"),
}
THRESHOLD = 0.977
# the groups `bench/counters.py run --events SET` counts, as it defines them
EVENT_SETS = {k: counters.EVENT_SETS[k] for k in RULE_EVENTS}

# Student's t, 97.5% quantile, df 1-30
T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042]  # fmt: skip


def geo_ci(ratios):
    """The geometric mean of `ratios` and its 95% t interval, (mean, low, high)"""
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    return math.exp(m), math.exp(m - h), math.exp(m + h)


def is_lever(ratio, hi):
    """The rule for one event: at most THRESHOLD (to within float rounding), its interval
    entirely below 1"""
    return ratio <= THRESHOLD + 1e-9 and hi < 1.0


def load(path, ref, cand, threads, reps, events):
    """The records keyed by (arm, threads, rep, group), and every problem found"""
    groups = EVENT_SETS[events]
    roster = {
        (arm, threads, r, g) for arm in (ref, cand) for r in range(reps) for g in range(len(groups))
    }
    rows, errors, outputs = {}, [], set()
    if reps < 2:
        errors.append(f"{reps} reps: an interval needs at least 2")
    if not os.path.exists(path):
        return rows, errors + [f"{path}: missing"]
    for ln, line in enumerate(open(path), 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            key = (r["arm"], int(r["threads"]), int(r["rep"]), int(r["group"]))
        except (ValueError, KeyError, TypeError):
            errors.append(f"line {ln}: not a record")
            continue
        if key in rows:
            errors.append(f"{key} duplicated")
            continue
        if key not in roster:
            errors.append(f"{key} not in the roster")
            continue
        rows[key] = r
        if r.get("events") != groups[key[3]]:
            errors.append(f"{key}: events {r.get('events')}, not the {events} set's group")
            continue
        if r.get("err") or r.get("rc") != 0 or r.get("child_err"):
            errors.append(
                f"{key}: the run failed: {r.get('err') or r.get('child_err') or r.get('rc')}"
            )
            continue
        outputs.add((r.get("out"), r.get("hash")))
        if not isinstance(r.get("out"), int) or r["out"] <= 0:
            errors.append(f"{key}: no output size")
        counts = r.get("counts") or {}
        for e in groups[key[3]]:
            c = counts.get(e)
            if c is None:
                errors.append(f"{key}: {e} missing")
            elif c[0] is None or c[0] < 0:
                errors.append(f"{key}: {e} not counted ({c[2]!r})")
            elif c[1] < 100.0:
                errors.append(f"{key}: {e} multiplexed ({c[1]}% enabled)")
            elif c[0] == 0:
                errors.append(f"{key}: {e} counted zero; its ratio is undefined")
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{len(missing)} of {len(roster)} runs missing, e.g. {missing[:3]}")
    if len(outputs) > 1:
        errors.append(f"the arms' outputs differ: {sorted(outputs, key=str)[:3]}")
    return rows, errors


def analyse(a):
    groups = EVENT_SETS[a.events]
    rows, errors = load(a.runs, a.ref, a.cand, a.threads, a.reps, a.events)
    if errors:
        print(
            f"VERDICT: VOID: the counters do not make the registered run ({len(errors)} problems)"
        )
        for e in errors[:25]:
            print(f"  {e}")
        return 1

    def per_byte(arm, r, e):
        g = next(i for i, ev in enumerate(groups) if e in ev)
        rec = rows[(arm, a.threads, r, g)]
        return rec["counts"][e][0] / rec["out"]

    events = []
    for ev in groups:
        events += [e for e in ev if e not in events]
    print(
        f"{a.threads} threads, {a.reps} paired reps, events {a.events}: the median a byte of "
        f"output; {a.cand} over {a.ref} as the geometric mean of the reps' paired ratios, "
        f"95% interval"
    )
    print(f"  {'event':30s} {'ref':>14s} {'cand':>14s}   {'ratio':>7s}  95% interval")
    got = {}
    for e in events:
        ref = [per_byte(a.ref, r, e) for r in range(a.reps)]
        cand = [per_byte(a.cand, r, e) for r in range(a.reps)]
        got[e] = geo_ci([c / x for c, x in zip(cand, ref)])
        g, lo, hi = got[e]
        print(
            f"  {e:30s} {statistics.median(ref):14.6f} {statistics.median(cand):14.6f}   "
            f"{g:7.4f}  [{lo:.4f}, {hi:.4f}]"
        )
    for arm in (a.ref, a.cand):

        def med(e):
            return statistics.median(per_byte(arm, r, e) for r in range(a.reps))

        cyc, ins = med("cycles"), med("instructions")
        stores, sb = med("mem_inst_retired.all_stores"), med("resource_stalls.sb")
        print(
            f"  {arm}: IPC {ins / cyc:.3f}; stores {stores / ins:.1%} of instructions; "
            f"store-buffer stalls {sb / cyc:.1%} of cycles"
        )
    l2, sb = RULE_EVENTS[a.events]
    holds = {e: is_lever(got[e][0], got[e][2]) for e in (l2, sb)}
    print(
        f"\nThe rule: {l2} and {sb} both at most {THRESHOLD}, each with its 95% interval "
        f"entirely below 1"
    )
    for e in (l2, sb):
        g, lo, hi = got[e]
        print(f"  {e}: {g:.4f} [{lo:.4f}, {hi:.4f}]: {'meets it' if holds[e] else 'does not'}")
    sub = (
        " (l2_trans.l2_wb standing in for l2_lines_out.non_silent, absent from this perf)"
        if a.events == "stores-alt"
        else ""
    )
    if all(holds.values()):
        print(f"VERDICT: STORE TRAFFIC IS THE LEVER{sub}")
    else:
        print(f"VERDICT: NOT ESTABLISHED{sub}")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--ref", required=True)
    ap.add_argument("--cand", required=True)
    ap.add_argument("--threads", type=int, default=10)
    ap.add_argument("--reps", type=int, default=10)
    ap.add_argument("--events", choices=sorted(EVENT_SETS), default="stores")
    return analyse(ap.parse_args(argv[1:]))


if __name__ == "__main__":
    sys.exit(main(sys.argv))
