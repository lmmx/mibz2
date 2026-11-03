#!/usr/bin/env python3
"""The compact30 construction-cost comparison's analysis
(docs/journal/2026-11-03-compact30-construction-protocol.md, draft 4).

    python3 bench/c30_construction.py RUN.jsonl [--json OUT.json] [--identity-ok yes|no]
        [--rss RSS.json ...] [--dry-run-evidence DIR] [--protocol v1|v2]

`--protocol v2` analyses constructor v2's records under its own protocol
(docs/journal/2026-11-03-compact30-construction-protocol-v2.md, report 211; `analyse_v2`
below). The default, v1, is draft 4's analysis as reviewed, and refuses v2's records.

Draft 4's repairs (report 197):
- G2 holds every all-full W4 step to the 2% start skew, not only the row-weighted
  aggregate, which is reported beside it.
- G9 requires a capture to carry a verified dry run's evidence (DIR, re-verified here from
  its files against the capture's own records). The required memory must be
  max(estimate, verified peak), and memory sources that couldn't be read count as failures.
- G7 reads the recorded poison coverage and the untimed poison pass's records.
- The six peak-RSS records are validated before any summary; otherwise the diagnostic is
  marked unavailable.
- The step mixing full and short blocks is reported on its own.

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
import hashlib
import json
import math
import os
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
KNOWN = {"meta", "preflight", "onepass", "counters", "poisoncheck", "calibration", "build",
         "check0", "postflight", "failure"}  # fmt: skip
# the dry-run evidence's gates, each of which must have passed (G9, report 197)
DRY_GATES = ("G3_identity", "G4_allocation", "G6_exact_output", "G7_round_minus_one",
             "G8_one_pass", "G9_memory", "G10_major_faults")  # fmt: skip


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
        and isinstance(meta.get("corpus_sha256"), str)
        and isinstance(meta.get("binary_sha256"), str)
        and isinstance(meta.get("machine"), dict)
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
        and sorted(o.get("block") for o in recs["poisoncheck"]) == list(range(nb))
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


def _pressure_free(m):
    """PSI read as zero, or absent; never when unreadable"""
    st = m.get("psi_status")
    if st == "absent":
        return True
    return st == "read" and m.get("psi_some_avg10") == 0 and m.get("psi_full_avg10") == 0


def check_memory(pre, post, meta, evidence=None):
    """G9, from the recorded numbers (the collector's own verdict is not trusted alone).
    Returns (preflight ok, postflight ok, evidence ok, reasons). A capture must carry a
    verified dry run's evidence, re-verified here from its files (`evidence`, as
    `load_evidence` returns it); a dry run must carry none."""
    why = []
    try:
        req, est, peak = pre["required"], pre["estimate"], pre.get("verified_peak")
        margin = max(2 * GIB, req // 4)
        pre_ok = (
            pre.get("passed") is True
            and pre.get("injected") is False
            and pre.get("readable") is True
            and _int(peak)
            and req == max(est, peak)
            and pre["margin"] == margin
            and _int(pre.get("mem_available"))
            and pre["mem_available"] >= req + margin
            and pre.get("swap_used") == 0
            and _pressure_free(pre)
        )  # fmt: skip
        if not pre_ok:
            why.append("preflight")
        post_ok = (
            post.get("readable") is True
            and post.get("swap_used") == 0
            and _pressure_free(post)
            and post.get("pswpin_delta") == 0
            and post.get("pswpout_delta") == 0
        )  # fmt: skip
        if not post_ok:
            why.append("postflight")
        ev = pre.get("dry_run_evidence")
        if meta.get("dry_run") is True:
            ev_ok = ev is None and peak == 0
        else:
            ev_ok = _evidence_ok(pre, meta, evidence, why)
        if not ev_ok:
            why.append("dry-run evidence")
        return pre_ok, post_ok, ev_ok, why
    except (KeyError, TypeError, AttributeError):
        return False, False, False, ["malformed memory records"]


def load_evidence(d):
    """A dry run's evidence directory, read for re-verification: its three files' contents
    and checksums, or None if any is missing"""
    out = {}
    for f in ("c30-construction.jsonl", "c30-analysis.json", "MANIFEST.json"):
        try:
            data = open(os.path.join(d, f), "rb").read()
        except OSError:
            return None
        out[f] = (data, hashlib.sha256(data).hexdigest())
    return out


def _evidence_ok(pre, meta, evidence, why):
    """A capture's dry-run evidence: recorded by the collector, and re-verified from the
    files themselves against the capture's own meta record"""
    ev = pre.get("dry_run_evidence")
    if not isinstance(ev, dict) or evidence is None:
        why.append("no dry-run evidence recorded or supplied")
        return False
    jl, an, mf = (
        evidence[f] for f in ("c30-construction.jsonl", "c30-analysis.json", "MANIFEST.json")
    )
    if (jl[1], an[1], mf[1]) != (
        ev.get("jsonl_sha256"),
        ev.get("analysis_sha256"),
        ev.get("manifest_sha256"),
    ):
        why.append("the evidence files are not the ones the collector verified")
        return False
    try:
        man = json.loads(mf[0])
        ana = json.loads(an[0])
        recs = [json.loads(line) for line in jl[0].splitlines() if line.strip()]
    except (ValueError, UnicodeDecodeError):
        why.append("the evidence files are not valid JSON")
        return False
    dmeta = next((r for r in recs if r.get("kind") == "meta"), None)
    dpost = next((r for r in recs if r.get("kind") == "postflight"), None)
    files = {f.get("path"): f.get("sha256") for f in man.get("files", [])}
    peak = pre.get("verified_peak")
    checks = {
        "manifest exit 0": man.get("exit_status") == 0,
        "manifest clean source": man.get("source", {}).get("tracked_changes") == [],
        "manifest lists the records": files.get("c30-construction.jsonl") == jl[1],
        "manifest lists the analysis": files.get("c30-analysis.json") == an[1],
        "same binary": (man.get("binary") or {}).get("sha256") == meta["binary_sha256"],
        "same corpus": [c.get("sha256") for c in man.get("corpora", [])] == [meta["corpus_sha256"]],
        "a dry run's analysis": ana.get("dry_run") is True and "failures" not in ana,
        "its schedule passed": bool(ana.get("schedule")) and all(v is True for v in ana["schedule"].values()),
        "its gates passed": all(ana.get("gates", {}).get(k) is True for k in DRY_GATES),
        "a completed dry run": dmeta is not None and dpost is not None
        and not any(r.get("kind") == "failure" for r in recs),
        "its meta matches": dmeta is not None
        and dmeta.get("dry_run") is True
        and dmeta.get("inject") is None
        and all(dmeta.get(k) == meta.get(k) for k in
                ("corpus", "corpus_sha256", "binary_sha256", "cpus", "machine", "rows", "block_rows")),
        "the peak is the records'": dpost is not None and _int(dpost.get("vm_hwm"))
        and dpost["vm_hwm"] == peak == ev.get("peak_rss")
        and (ana.get("memory") or {}).get("peak_rss_vm_hwm") == peak,
        "the peak is plausible": _int(peak) and peak >= 2 * meta["rows"],
    }  # fmt: skip
    bad = [k for k, v in checks.items() if not v]
    why.extend(f"evidence: {k}" for k in bad)
    return not bad


def check_rss(rss, meta):
    """The six peak-RSS records (§5): three distinct processes per arm, each of the
    capture's corpus and CPUs, with the high-water mark reset and sensible values.
    Returns (summary or None, problems)."""
    probs = []
    if len(rss) != 6:
        probs.append(f"{len(rss)} records, not 6")
    by = {}
    seen = set()
    for r in rss:
        a = r.get("arm")
        key = (r.get("pid"), r.get("started_unix_ns"))
        if key in seen or not all(_int(x) for x in key):
            probs.append(f"a duplicate or unidentified process {key}")
        seen.add(key)
        ok = (
            r.get("kind") == "rss"
            and a in ("P", "C")
            and r.get("corpus") == meta.get("corpus")
            and r.get("corpus_sha256") == meta.get("corpus_sha256")
            and r.get("cpus") == meta.get("cpus")
            and r.get("blocks") == meta.get("blocks")
            and r.get("hwm_reset") is True
            and all(_int(r.get(k)) and r[k] >= 0 for k in
                    ("baseline_rss", "hwm_after_reset", "hwm_after_pass", "growth_over_baseline",
                     "arena_capacity_bytes"))
            and r["hwm_after_pass"] >= r["hwm_after_reset"]
            and r["baseline_rss"] > 0
            and r["arena_capacity_bytes"] > 0
        )  # fmt: skip
        if not ok:
            probs.append(f"an invalid record (arm {a})")
            continue
        by.setdefault(a, []).append(r["growth_over_baseline"])
    if sorted((a, len(v)) for a, v in by.items()) != [("C", 3), ("P", 3)]:
        probs.append("not three valid records per arm")
    if probs:
        return None, probs
    return {
        a: {"median": statistics.median(v), "min": min(v), "max": max(v), "n": len(v),
            "capacity_scope": "tt" if a == "P" else "rec+raw"}
        for a, v in by.items()
    }, []  # fmt: skip


def analyse(recs, other=(), identity_ok=None, rss=(), evidence=None):
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

    # G7: the exact and walker checks for both arms; C's poison coverage as recorded (its
    # flag counts, it does not gate); and the untimed poison pass, which must cover every
    # block with a surviving poisoned allocation
    def check0_ok(o):
        if o.get("arm") == "P":
            return all(o.get(k) is True for k in ("definition", "walk", "restored"))
        return (
            all(o.get(k) is True for k in ("exact", "walk", "restored"))
            and _int(o.get("poisoned_capacity"))
            and _int(o.get("needed"))
            and isinstance(o.get("poison_survived"), bool)
            and (not o["poison_survived"] or o["poisoned_capacity"] >= o["needed"] > 0)
        )

    g["G7_round_minus_one"] = all(check0_ok(o) for o in recs["check0"]) and all(
        o.get("exact") is True
        and o.get("poison_survived") is True
        and _int(o.get("poisoned_capacity"))
        and _int(o.get("needed"))
        and o["poisoned_capacity"] >= o["needed"] > 0
        for o in recs["poisoncheck"]
    )
    c0 = [o for o in recs["check0"] if o.get("arm") == "C"]
    out["poison_coverage"] = {
        "round_minus_one_blocks_poison_survived": sum(o.get("poison_survived") is True for o in c0),
        "round_minus_one_blocks": len(c0),
        "poison_pass_blocks_survived": sum(
            o.get("poison_survived") is True for o in recs["poisoncheck"]
        ),
        "poison_pass_blocks": len(recs["poisoncheck"]),
    }
    g["G8_one_pass"] = all(
        o.get("histogram_equal") is True and o.get("below_2_24") is True for o in recs["onepass"]
    )
    pre, post = recs["preflight"][0], recs["postflight"][0]
    pre_ok, post_ok, ev_ok, why = check_memory(pre, post, meta, evidence)
    g["G9_memory"] = pre_ok and post_ok and ev_ok
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
        "dry_run_evidence_ok": ev_ok,
        "verified_peak": pre.get("verified_peak"),
        "problems": why,
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
                "scope": "tt" if a == "P" else "rec+raw (the bitmap's requested size apart)",
                "cap_bytes_max": max((w.get("cap_bytes", 0) for w in v), default=0),
                "usable_bytes_max": max((w.get("usable_bytes", 0) for w in v), default=0),
                "bitmap_req_bytes_max": max((w.get("bitmap_req_bytes", 0) for w in v), default=0),
            }
    out["capacities"] = caps
    summary, probs = check_rss(list(rss), meta)
    out["peak_rss_processes"] = (
        summary if summary is not None else {"unavailable": True, "problems": probs}
    )
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
            "w4_start_skew_steps": [
                (max(x["start"] for x in v) - min(x["start"] for x in v)) / span(v)
                for v in full_steps
            ],
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
    # G2 (draft 4, report 197): every all-full W4 step's own start skew, in every arm, regime
    # and measured round; the row-weighted aggregate is reported beside it
    steps_sk = [x for r in per for rg in REGIMES for a in ("P", "C")
                for x in per[r][rg][a]["w4_start_skew_steps"]]  # fmt: skip
    sks = [per[r][rg][a]["w4_start_skew"] for r in per for rg in REGIMES for a in ("P", "C")]
    g["G2_simultaneous_start"] = bool(steps_sk) and all(x <= START_SKEW for x in steps_sk)
    out["start_skew"] = {
        "per_step_max": max(steps_sk) if steps_sk else None,
        "per_step_over_threshold": sum(x > START_SKEW for x in steps_sk),
        "steps_checked": len(steps_sk),
        "row_weighted_aggregate_max": max(sks),
    }
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
    # the steps that mix full and short blocks, reported on their own (A7)
    mixed = [s for s in range(nsteps)
             if len({br[lists[k][s]] >= FULL for k in range(4) if s < len(lists[k])}) > 1]  # fmt: skip
    if mixed:
        rs = []
        for r in range(rounds):

            def msp(a):
                v = by[(r, "warm", "w4", a)]
                return sum(span([x for x in v if x["step"] == s]) for s in mixed)

            rs.append(msp("C") / max(msp("P"), 1))
        pt, lo, hi = interval(rs)
        econ["mixed_steps"] = {"steps": mixed, "e4": {"point": pt, "lo": lo, "hi": hi}}
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


# ---------------------------------------------------------------------------------------
# Constructor v2's protocol (docs/journal/2026-11-03-compact30-construction-protocol-v2.md,
# report 211). Everything above is draft 4's analysis, unchanged; v2 runs it on v2's records
# and then applies v2's registered differences:
# - the records must carry v2's identifier (S1_protocol), and so must a capture's dry-run
#   evidence and its peak-RSS records;
# - G1 gates P's warm W4 cost and warm W4 completion time only; the other stability ratios
#   are reported;
# - G2 is three conditions on each arm's warm W4 all-full steps over the measured rounds:
#   the row-weighted aggregate, the outlier allowance and the maximum step; cold steps are
#   reported.
# The economic limits and the decision table are draft 4's.
# ---------------------------------------------------------------------------------------

PROTOCOL_V2 = "c30-construction-v2"
CANDIDATE_V2 = "ctor2"
G2_AGGREGATE = 0.02  # v2 G2(a): each arm's row-weighted aggregate start skew
G2_STEP = 0.02  # v2 G2(b): a step above this is an outlier
G2_ALLOWANCE_PER_MILLE = 1  # v2 G2(b): at most ⌊N / 1000⌋ outliers among an arm's N steps
G2_MAX = 0.10  # v2 G2(c): no step above this
V2_G1 = ("w4_cpu_ns_row", "w4_completion_ns")  # v2 G1: P's warm W4 measures


def step_skews(builds, meta, regime, rounds):
    """Draft 4's per-step start skew of every all-full W4 step of `regime` in the measured
    rounds: (latest start − earliest start) ÷ (latest end − earliest start) over the step's
    four builds, all of full blocks. Returns {arm: [(round, step, rows, skew)]}."""
    lists = meta["w4_lists"]
    nsteps = max(len(l) for l in lists)
    by = {}
    for w in builds:
        if w["regime"] == regime and w["mode"] == "w4" and 0 <= w["round"] < rounds:
            by.setdefault((w["arm"], w["round"], w["step"]), []).append(w)
    out = {}
    for a in ("P", "C"):
        v = []
        for r in range(rounds):
            for s in range(nsteps):
                x = by.get((a, r, s), [])
                if len(x) == 4 and all(y["rows"] >= FULL for y in x):
                    st = min(y["start"] for y in x)
                    span = max(y["end"] for y in x) - st
                    sk = (max(y["start"] for y in x) - st) / span
                    v.append((r, s, sum(y["rows"] for y in x), sk))
        out[a] = v
    return out


def g2_summary(steps):
    """One arm's v2 G2 figures from its steps: N, the row-weighted aggregate (the rows-weighted
    mean of draft 4's per-step skew, Σ rows·skew ÷ Σ rows, pooled over the measured rounds),
    the outliers (> 2%) with their locations, the integer allowance ⌊N/1000⌋, the maximum"""
    n = len(steps)
    rows = sum(x[2] for x in steps)
    agg = sum(x[2] * x[3] for x in steps) / rows if rows else None
    over = [{"round": r, "step": s, "skew": k} for r, s, _, k in steps if k > G2_STEP]
    return {
        "steps": n,
        "row_weighted_aggregate": agg,
        "max_step_skew": max((x[3] for x in steps), default=None),
        "outliers_over_2pct": len(over),
        "outlier_allowance": n * G2_ALLOWANCE_PER_MILLE // 1000,
        "outliers": over,
        "steps_over_10pct": sum(x[3] > G2_MAX for x in steps),
    }


def _v2_evidence_problems(evidence):
    """A capture's dry-run evidence must be v2's: its analysis and its meta record carry the
    identifier (draft 4's conditions are checked by `analyse`)"""
    if evidence is None:
        return []
    probs = []
    try:
        ana = json.loads(evidence["c30-analysis.json"][0])
        recs = [
            json.loads(x) for x in evidence["c30-construction.jsonl"][0].splitlines() if x.strip()
        ]
    except (ValueError, UnicodeDecodeError, KeyError):
        return ["evidence: unreadable for the protocol check"]
    if ana.get("protocol") != PROTOCOL_V2:
        probs.append("evidence: the dry run's analysis is not protocol v2's")
    dmeta = next((r for r in recs if r.get("kind") == "meta"), {})
    if dmeta.get("protocol") != PROTOCOL_V2 or dmeta.get("candidate") != CANDIDATE_V2:
        probs.append("evidence: the dry run's records are not protocol v2's")
    return probs


def analyse_v2(recs, other=(), identity_ok=None, rss=(), evidence=None):
    """Constructor v2's analysis: draft 4's `analyse` on v2's records, then v2's G1 and G2,
    the protocol checks, v2's diagnostics and draft 4's decision table"""
    metas = recs["meta"]
    meta = metas[0] if metas else None
    if meta is None:
        # no meta record (a run that failed before writing it): draft 4's verdict, as such
        out = {"protocol": PROTOCOL_V2}
        out.update(analyse(recs, other, identity_ok, rss, evidence))
        out["schedule"]["S1_protocol"] = False
        return out
    if meta.get("protocol") != PROTOCOL_V2 or meta.get("candidate") != CANDIDATE_V2:
        out = {"protocol": PROTOCOL_V2, "dry_run": bool(meta.get("dry_run")),
               "schedule": {"S1_protocol": False}, "gates": {}, "valid": False,
               "decision": "INVALID: the records are not protocol v2's (constructor v2); "
                           "no figures computed"}  # fmt: skip
        if recs["failure"]:
            out["failures"] = list(recs["failure"])
        return out
    v = dict(recs)
    # draft 4's slow-path table reads `wide_promotions`: v2's wide groups promoted to RAW at
    # their finish (`fin_wide`)
    v["counters"] = [dict(o, wide_promotions=o.get("fin_wide")) for o in recs["counters"]]
    rss = list(rss)
    rss_bad = [r for r in rss if r.get("protocol") != PROTOCOL_V2]
    out = {"protocol": PROTOCOL_V2, "candidate": CANDIDATE_V2}
    out.update(analyse(v, other, identity_ok, rss, evidence))
    out["schedule"]["S1_protocol"] = True
    if rss_bad and "peak_rss_processes" in out and not out["peak_rss_processes"].get("unavailable"):
        out["peak_rss_processes"] = {
            "unavailable": True,
            "problems": [f"{len(rss_bad)} records are not protocol v2's"],
        }
    sch = out["schedule"]
    if not all(x is True for x in sch.values()):
        return out
    g = out["gates"]
    ev_probs = [] if meta.get("dry_run") else _v2_evidence_problems(evidence)
    if ev_probs:
        g["G9_memory"] = False
        out["memory"]["dry_run_evidence_ok"] = False
        why = list(out["memory"]["problems"]) + ev_probs
        out["memory"]["problems"] = why + (
            [] if "dry-run evidence" in why else ["dry-run evidence"]
        )
    if meta["dry_run"]:
        return out
    rounds = meta["rounds"]
    per = out["rounds"]
    # G1 (v2): P's warm W4 cost and completion time gate; the rest is reported
    stab = {}
    for rg in REGIMES:
        for a in ("P", "C"):
            for key in ("w1_cpu_ns_row", "w4_cpu_ns_row", "w4_completion_ns"):
                xs = [per[r][rg][a][key] for r in per]
                stab[f"{rg}_{a}_{key}"] = {
                    "max_over_min": max(xs) / min(xs),
                    "min_round": min(per, key=lambda r: per[r][rg][a][key]),
                    "max_round": max(per, key=lambda r: per[r][rg][a][key]),
                    "gates": rg == "warm" and a == "P" and key in V2_G1,
                }
    for k in [k for k in g if k.startswith("G1_") or k == "G2_simultaneous_start"]:
        del g[k]
    for key in V2_G1:
        g[f"G1_warm_{key}"] = stab[f"warm_P_{key}"]["max_over_min"] <= STABILITY
    out["stability"] = stab
    # G2 (v2): each arm's warm W4 all-full steps over the measured rounds
    warm = step_skews(recs["build"], meta, "warm", rounds)
    cold = step_skews(recs["build"], meta, "cold", rounds)
    sw = {a: g2_summary(warm[a]) for a in ("P", "C")}
    sc = {a: g2_summary(cold[a]) for a in ("P", "C")}
    for a in ("P", "C"):
        for o in sw[a]["outliers"] + sc[a]["outliers"]:
            o["arm"] = a
    g["G2_aggregate"] = all(
        sw[a]["steps"] > 0 and sw[a]["row_weighted_aggregate"] <= G2_AGGREGATE for a in ("P", "C")
    )
    g["G2_outlier_allowance"] = all(
        sw[a]["steps"] > 0 and sw[a]["outliers_over_2pct"] <= sw[a]["outlier_allowance"]
        for a in ("P", "C")
    )
    g["G2_max_step"] = all(
        sw[a]["steps"] > 0 and sw[a]["max_step_skew"] <= G2_MAX for a in ("P", "C")
    )
    out["start_skew"] = {
        "warm_gated": sw,
        "cold_reported": sc,
        "per_round_aggregate_max": {
            f"{rg}_{a}": max(per[r][rg][a]["w4_start_skew"] for r in per)
            for rg in REGIMES
            for a in ("P", "C")
        },
    }
    # v2's slow paths: the per-block warm W1 ratio by deciles of the one-row share, with
    # v2's finishing classes (reported only)
    by = {}
    for w in recs["build"]:
        if w["round"] >= 0 and w["regime"] == "warm" and w["mode"] == "w1":
            by[(w["arm"], w["round"], w["block"])] = w
    cpu = lambda w: w["cpu_end"] - w["cpu_start"]  # noqa: E731
    br = meta["block_rows"]
    ctr = {o["block"]: o for o in recs["counters"]}
    pb = []
    for b in range(len(br)):
        if br[b] < FULL:
            continue
        rs = [cpu(by[("C", r, b)]) / max(cpu(by[("P", r, b)]), 1) for r in range(rounds)]
        c = ctr[b]
        pb.append((c["rows_scalar"] / br[b], c["rows_fragment"] / br[b],
                   c.get("fin_ordinary", 0), c.get("fin_wide", 0), c.get("fin_crossing", 0),
                   c.get("norm_fields", 0) / br[b], statistics.median(rs)))  # fmt: skip
    pb.sort()
    dec = []
    for d in range(10):
        part = pb[d * len(pb) // 10 : (d + 1) * len(pb) // 10]
        if part:
            dec.append({
                "one_row_share_max": part[-1][0],
                "fragment_share_mean": statistics.fmean(x[1] for x in part),
                "nonfast_ordinary_finishes_mean": statistics.fmean(x[2] for x in part),
                "wide_finishes_mean": statistics.fmean(x[3] for x in part),
                "crossing_portions_mean": statistics.fmean(x[4] for x in part),
                "normalised_fields_per_row_mean": statistics.fmean(x[5] for x in part),
                "warm_w1_ratio_median": statistics.median(x[6] for x in part),
                "blocks": len(part),
            })  # fmt: skip
    out["slow_paths_v2_by_one_row_decile"] = dec
    rat = out["ratios"]
    b4, e4 = rat["warm_b4"], rat["warm_e4"]
    valid = all(x is True for x in g.values())
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
    ap.add_argument("--dry-run-evidence", help="a capture's dry-run evidence directory")
    ap.add_argument("--protocol", choices=["v1", "v2"], default="v1",
                    help="v1: draft 4 (constructor v1); v2: constructor v2 (report 211)")  # fmt: skip
    a = ap.parse_args()
    recs, other = load(a.run)
    if a.protocol == "v1" and any(m.get("protocol") for m in recs["meta"]):
        print("c30_construction: these records are protocol v2's; analyse them with --protocol v2")
        return 2
    rss = []
    for p in a.rss:
        try:
            rss.append(json.loads(open(p).read()))
        except (OSError, ValueError):
            rss.append({"unreadable": p})
    ident = None if a.identity_ok is None else a.identity_ok == "yes"
    ev = load_evidence(a.dry_run_evidence) if a.dry_run_evidence else None
    res = (analyse_v2 if a.protocol == "v2" else analyse)(recs, other, ident, rss, ev)
    if a.json:
        with open(a.json, "w") as f:
            json.dump(res, f, indent=1)
    for fl in res.get("failures", []):
        print("FAILED RUN:", fl.get("what"), "-", fl.get("detail"))
    if a.protocol == "v2":
        print("protocol:", res["protocol"])
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
    if a.protocol == "v2" and "stability" in res:
        print("start skew (warm, gated):", json.dumps(
            {k: {x: y for x, y in v.items() if x != "outliers"} for k, v in res["start_skew"]["warm_gated"].items()}))  # fmt: skip
        print("start-skew outliers (warm):", json.dumps(
            [o for v in res["start_skew"]["warm_gated"].values() for o in v["outliers"]]))  # fmt: skip
    print("decision:", res["decision"])
    return 0 if res["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
