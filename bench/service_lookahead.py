"""The service core's prepared-queue admission threshold on the workstation
(docs/journal/2026-11-09-service-lookahead-registration.md, registered before any run): the
report and its verdicts, each predicate checked here.

    python3 bench/service_lookahead.py report RUNS.jsonl [--occupancy W/occupancy-*.json] [--out STEM]

The question: does svc2@10 (the service core at 2 x 3 + 2, on a budget of 10 decode threads)
gain from a higher prepared-queue admission threshold, 3 or 4 against the 2 (`teams`) it ran
at in bench-service-2x3? Its trace there showed PREP STARVATION without saturation.

The threshold (the hub's `lookahead`, `StreamDecoder::service_admission`) is not a cap on the
queue: a service worker starts another preparation while fewer than it are prepared, and
preparations under way are not counted, so with K service threads the prepared count P can
reach threshold + K - 1 (Guardian D045). Each trace's P (max, and the median over every
preparation's push) is reported per arm.

A run is VALID only as the registered one, with bench/service_2x3.py's strict checks
(imported unchanged): the registered 20 rounds, from the attempt's directory a clean tree, no
`NOT-FROZEN.txt` or `DRY-RUN.txt`, `commit.txt` the checkout's head, the ten-core topology,
every run's record usable (a hex digest, the registered decoded size, finite positive wall,
CPU and peak RSS, the registered corpus), every run the same bytes as the reference's,
`corpus.sha256` the registered corpus's, every registered cell's rounds once each with its
registered plan, no other cell, every registered trace complete and as registered (each at
its own threshold and window), saturation resolved in each, and the noise control N1 within
1 +/- 0.03.

The registered cells (ARM@BUDGET: team leaders, helpers, service threads; admission
threshold; window):

    mibz2@10                3, 6, 0   -   12   today's default (0.2.6): 3 x 3 pipelined, park and small
    mibz2-svc2@10           2, 4, 2   2    8   the service core, as bench-service-2x3 ran it
    mibz2-svc2-a3@10        2, 4, 2   3    8   threshold 3
    mibz2-svc2-a4@10        2, 4, 2   4    8   threshold 4
    mibz2-svc2-w12@10       2, 4, 2   2   12   the window alone
    mibz2-svc2-a4-w12@10    2, 4, 2   4   12   both (the joint arm)
    mibz2-team3@10          3, 6, 0   -   12   3 x 3, plain
    mibz2-team3-twin@10     3, 6, 0   -   12   mibz2-team3 under another name: the noise control

The outcome map, the plan count from ab's per-thread CPU and the traces' failure modes with
their thresholds are bench/service_core.py's, unchanged.

Revised 9 November for the Researcher's REVIEW 6730484783 (the registration's revision
section), with stricter evidence checks and no changed threshold or N1:
- H1: every complete timeline round's prepared count, from the raw runs, within
  1 <= median <= max <= threshold + services - 1, with the registered 431 blocks, before any
  aggregate is read; the summary reconciled with those rounds (it is the median of round
  maxima and of round medians, so labelled); the per-round evidence kept in the JSON;
- H2: every scheduled child (requested rounds x 3 modes) in the raw runs once, each with exit
  status 0, and occupancy's own `children_ok`; empty raw runs INVALID; the inherited rule of at
  least one complete trace round kept;
- the second revision (REVIEW 6730774618): every pass the registered 6 rounds (18 children);
  parity, output and error recomputed from each child's own line; every required metric of
  every complete round present, finite and in its domain before reconciliation (tolerance
  1e-6 x max(1, |median|)), a NaN never dropped;
- H3: the gaining arms and the relieved arms named separately, a concordant reading only
  where they overlap; a reading is the effect of the implemented higher-threshold policy, not
  proof of a queue bottleneck.
"""

import argparse
import json
import math
import os
import re
import statistics
import subprocess
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from service_core import (  # noqa: E402
    CONTROL_EQUIV,
    failure_modes,
    geo_ci,
    is_gain,
    outcome,
    threads_of,
)
from service_2x3 import (  # noqa: E402
    HASH,
    REGISTERED_CORPUS,
    REGISTERED_CORPUS_SHA256,
    REGISTERED_OUT,
    TRACE_NONNEG,
    TRACE_PLAN,
    TRACE_SERVICES,
    TRACE_SHARES,
    TRACE_THREADS,
    cell,
    cpu_phrase,
    decode_threads,
    number,
    row_problems,
    speed_phrase,
)

REGISTERED_ROUNDS = 20
SERVICES = 2
# cell -> (leaders, helpers, service threads)
CELLS = {
    "mibz2@10": (3, 6, 0),
    "mibz2-svc2@10": (2, 4, 2),
    "mibz2-svc2-a3@10": (2, 4, 2),
    "mibz2-svc2-a4@10": (2, 4, 2),
    "mibz2-svc2-w12@10": (2, 4, 2),
    "mibz2-svc2-a4-w12@10": (2, 4, 2),
    "mibz2-team3@10": (3, 6, 0),
    "mibz2-team3-twin@10": (3, 6, 0),
}
# each cell's window (blocks in flight), as examples/ab.rs builds it at 10 threads
WINDOW = {
    "mibz2@10": 12,
    "mibz2-svc2@10": 8,
    "mibz2-svc2-a3@10": 8,
    "mibz2-svc2-a4@10": 8,
    "mibz2-svc2-w12@10": 12,
    "mibz2-svc2-a4-w12@10": 12,
    "mibz2-team3@10": 12,
    "mibz2-team3-twin@10": 12,
}
# each service-core cell's admission threshold (the hub's `lookahead`; 2 is `teams`)
ADMISSION = {
    "mibz2-svc2@10": 2,
    "mibz2-svc2-a3@10": 3,
    "mibz2-svc2-a4@10": 4,
    "mibz2-svc2-w12@10": 2,
    "mibz2-svc2-a4-w12@10": 4,
}
SVC2 = list(ADMISSION)


def label(c):
    """A cell's threshold and window, as the tables print them"""
    return f"a{ADMISSION[c]} w{WINDOW[c]}" if c in ADMISSION else f"w{WINDOW[c]}"


# the registered comparisons (name, arm, reference): judged, speed and CPU co-primary
JUDGED = [
    ("LA3", "mibz2-svc2-a3@10", "mibz2-svc2@10"),
    ("LA4", "mibz2-svc2-a4@10", "mibz2-svc2@10"),
]
# reported, never judged
SECONDARY = [
    ("W12", "mibz2-svc2-w12@10", "mibz2-svc2@10"),
    ("J", "mibz2-svc2-a4-w12@10", "mibz2-svc2@10"),
    ("J4", "mibz2-svc2-a4-w12@10", "mibz2-svc2-a4@10"),
    ("D-a2w8", "mibz2-svc2@10", "mibz2@10"),
    ("D-a3w8", "mibz2-svc2-a3@10", "mibz2@10"),
    ("D-a4w8", "mibz2-svc2-a4@10", "mibz2@10"),
    ("D-a2w12", "mibz2-svc2-w12@10", "mibz2@10"),
    ("D-a4w12", "mibz2-svc2-a4-w12@10", "mibz2@10"),
    ("R1", "mibz2@10", "mibz2-team3@10"),
]
# the svc2 arms against the default, of which the best is reported (never judged)
AGAINST_DEFAULT = ["D-a2w8", "D-a3w8", "D-a4w8", "D-a2w12", "D-a4w12"]
# the noise control, as the service core's registration has it, unchanged
CONTROL = ("N1", "mibz2-team3-twin@10", "mibz2-team3@10")
REFERENCE = "mibz2@10"
# the registered traces (the occupancy passes' names) -> their cell; occupancy's `--slots S`
# at 10 threads gives 2 teams a window of S * 2 / 10, so each pass's slots are 5 x its window,
# and `--admission N` is given only where the threshold isn't the default's (`teams`), so the
# default's passes run `parallel_service` itself, as its arms do
TRACES = {f"occupancy-t10-svc2-a{ADMISSION[c]}-w{WINDOW[c]}": c for c in SVC2}
TRACE_SLOTS = {name: 5 * WINDOW[c] for name, c in TRACES.items()}
DEFAULT_ADMISSION = "teams (the service core's own)"
TRACE_ADMISSION = {name: (ADMISSION[c] if ADMISSION[c] != 2 else DEFAULT_ADMISSION) for name, c in TRACES.items()}
# the prepared count P, as the trace reads it, each round: its max, and its median over every
# preparation's push; the trace's summary is the median of the complete rounds' values (the
# median of round maxima, and the median of round medians), not the pass's max or a pooled
# median, so the report checks every round's against the invariant before reading them
TRACE_P = ["svc.backlog.prep_max", "svc.backlog.prep_median"]
# the registered workload's blocks, as the scanner issues them (`t.blocks`): an explicit
# expected count for the corpus whose digest is fixed (REGISTERED_CORPUS_SHA256), from earlier
# traces (#258's workstation trace, this registration's dry runs); bench/wikidata.json doesn't
# hold it
REGISTERED_BLOCKS = 431
# the occupancy modes, each round's children (examples/occupancy.rs's MODES)
TRACE_MODES = ("off", "timeline", "full")
# the rounds each pass is registered to request (the recipe's trace_rounds), so 18 children a
# pass; at least one of them a complete timeline round is the inherited rule, unchanged
TRACE_ROUNDS = 6
# the summary keys checked against the rounds they summarise (the medians of the complete
# timeline rounds' values)
RECONCILED = TRACE_SHARES + TRACE_NONNEG + TRACE_P


def ceiling(c):
    """The prepared count's invariant at a cell: threshold + service threads - 1 (Guardian
    D045: the threshold admits, and preparations under way aren't counted)"""
    return ADMISSION[c] + SERVICES - 1


def line_kv(line):
    return dict(x.split("=", 1) for x in line.split() if "=" in x) if isinstance(line, str) else {}


def fnum(v):
    try:
        x = float(v)
    except (TypeError, ValueError):
        return math.nan
    return x if math.isfinite(x) else math.nan


def check_raw(o, name, tl):
    """The trace's raw evidence (revision of 9 November, H1 and H2): every scheduled child
    accounted for, once, with exit status 0; every complete timeline round's prepared count
    within its invariant, its median at most its max, the registered blocks; and the summary
    reconciled with those rounds. Returns (per-round evidence, problems)."""
    p = []
    st = o.get("settings") if isinstance(o.get("settings"), dict) else {}
    req = st.get("rounds")
    if not (number(req) and float(req).is_integer() and req >= 1):
        return [], [f"requested rounds {req!r}, not a whole number of at least 1"]
    req = int(req)
    if req != TRACE_ROUNDS:
        p.append(f"requested rounds {req}, not the registered {TRACE_ROUNDS} ({TRACE_ROUNDS * len(TRACE_MODES)} children)")
    runs = o.get("runs")
    if not isinstance(runs, list) or not runs:
        return [], ["no raw runs"]
    if o.get("children_ok") is not True:
        p.append(f"children_ok {o.get('children_ok')!r}, not true")
    if o.get("children_expected") != req * len(TRACE_MODES):
        p.append(f"children expected {o.get('children_expected')!r}, not {req * len(TRACE_MODES)}")
    want = {(r, m) for r in range(TRACE_ROUNDS) for m in TRACE_MODES}
    seen = []
    hashes = set()
    for x in runs:
        if not isinstance(x, dict):
            p.append("a raw run not a record")
            continue
        seen.append((x.get("round"), x.get("mode")))
        if x.get("status") != 0:
            p.append(f"round {x.get('round')!r} {x.get('mode')!r}: exit status {x.get('status')!r}, not 0")
        if x.get("threads") != TRACE_THREADS or x.get("slots") != TRACE_SLOTS[name]:
            p.append(f"round {x.get('round')!r} {x.get('mode')!r}: threads {x.get('threads')!r}, slots {x.get('slots')!r}")
        # parity, output and error from the child's own line, not the summary's flag
        kv = line_kv(x.get("line"))
        h = kv.get("hash")
        if not (isinstance(h, str) and HASH.fullmatch(h)):
            p.append(f"round {x.get('round')!r} {x.get('mode')!r}: hash {h!r} not a hex digest")
        else:
            hashes.add(h)
        if fnum(kv.get("out")) != REGISTERED_OUT:
            p.append(f"round {x.get('round')!r} {x.get('mode')!r}: out {kv.get('out')!r}, not the registered {REGISTERED_OUT}")
        if kv.get("err") != "-":
            p.append(f"round {x.get('round')!r} {x.get('mode')!r}: err {kv.get('err')!r}, not none")
    if sorted(seen, key=repr) != sorted(want, key=repr) or len(seen) != len(set(seen)):
        p.append(f"{len(seen)} raw runs, not the {len(want)} scheduled ({TRACE_ROUNDS} rounds x {len(TRACE_MODES)} modes) once each")
    if len(hashes) > 1:
        p.append(f"the children's output digests differ ({', '.join(sorted(hashes))})")
    c = TRACES[name]
    top = ceiling(c)
    rounds = []
    for x in runs:
        if not isinstance(x, dict) or x.get("mode") != "timeline":
            continue
        kv = line_kv(x.get("line"))
        if kv.get("t.complete") != "yes":
            continue
        pm, pmed, blocks = fnum(kv.get("t.svc.backlog.prep_max")), fnum(kv.get("t.svc.backlog.prep_median")), fnum(kv.get("t.blocks"))
        rounds.append({"round": x.get("round"), "p_max": pm, "p_median": pmed, "blocks": blocks, "kv": kv})
        # every required metric present, finite and in its domain in this complete round,
        # before any median: shares and utilisations in [0, 1], the rest at least 0
        bad = [k for k in TRACE_SHARES if not (0 <= fnum(kv.get("t." + k)) <= 1)]
        bad += [k for k in TRACE_NONNEG + TRACE_P if not (fnum(kv.get("t." + k)) >= 0)]
        if fnum(kv.get("t.svc.n.services")) != TRACE_SERVICES:
            bad.append("svc.n.services")
        if bad:
            p.append(f"round {x.get('round')!r}: metrics absent, not finite or out of domain: {', '.join(bad)}")
        if blocks != REGISTERED_BLOCKS:
            p.append(f"round {x.get('round')!r}: {blocks!r} blocks, not the registered {REGISTERED_BLOCKS}")
        if not (1 <= pmed <= pm <= top):
            p.append(
                f"round {x.get('round')!r}: prepared count max {pm!r}, median {pmed!r}, "
                f"not 1 <= median <= max <= {top} (threshold {ADMISSION[c]} + {SERVICES} - 1)"
            )
    if not rounds:
        p.append("no complete timeline round in the raw runs")
    elif tl.get("complete_rounds") != len(rounds):
        p.append(f"summary complete rounds {tl.get('complete_rounds')!r}, raw {len(rounds)}")
    else:
        for k in RECONCILED:
            v = [fnum(r["kv"].get("t." + k)) for r in rounds]
            # (a NaN in a complete round is already its own problem above; never dropped here)
            m = statistics.median(v) if v and not any(math.isnan(x) for x in v) else math.nan
            if not (number(tl.get(k)) and not math.isnan(m) and abs(tl[k] - m) <= 1e-6 * max(1.0, abs(m))):
                p.append(f"summary {k} {tl.get(k)!r}, not the median of its complete rounds ({m!r})")
    for r in rounds:
        r.pop("kv")
    return rounds, p


def load_trace(paths, name, commit):
    """A registered trace's timeline, its complete rounds' prepared counts, and its problems:
    bench/service_2x3.py's checks (configuration, parity, output and every metric the report
    reads), its slots the registered ones for its window, its admission threshold the
    registered one, and its raw evidence (`check_raw`) (an empty list: the trace is usable)"""
    found = [x for x in paths if os.path.basename(x).removesuffix(".json") == name]
    if len(found) != 1:
        return {}, [], [f"{len(found)} files named `{name}.json` (one registered)"]
    try:
        o = json.load(open(found[0]))
    except (OSError, ValueError) as e:
        return {}, [], [f"unreadable: {e}"]
    if not isinstance(o, dict):
        return {}, [], ["not a trace report"]
    p = []
    res = o.get("results")
    if not (isinstance(res, list) and len(res) == 1 and isinstance(res[0], dict)):
        return {}, [], ["not exactly one result (one thread count registered)"]
    r = res[0]
    tl = r.get("timeline") if isinstance(r.get("timeline"), dict) else {}
    st = o.get("settings") if isinstance(o.get("settings"), dict) else {}
    meta = o.get("meta") if isinstance(o.get("meta"), dict) else {}
    if r.get("threads") != TRACE_THREADS or st.get("threads") != [TRACE_THREADS]:
        p.append(f"threads {r.get('threads')!r} / {st.get('threads')!r}, not {TRACE_THREADS}")
    if st.get("teams") != TRACE_PLAN:
        p.append(f"plan {st.get('teams')!r}, not {TRACE_PLAN!r}")
    want = TRACE_SLOTS[name]
    if r.get("slots") != want or st.get("slots") != want:
        p.append(f"slots {r.get('slots')!r} / {st.get('slots')!r}, not {want} (a window of {WINDOW[TRACES[name]]})")
    if st.get("admission") != TRACE_ADMISSION[name]:
        p.append(f"admission {st.get('admission')!r}, not {TRACE_ADMISSION[name]!r}")
    if tl.get("svc.n.services") != TRACE_SERVICES:
        p.append(f"service threads {tl.get('svc.n.services')!r}, not {TRACE_SERVICES}")
    if not commit or meta.get("commit") != commit:
        p.append(f"its commit {meta.get('commit')!r} not the run's ({commit})")
    if o.get("parity") is not True:
        p.append(f"parity {o.get('parity')!r}, not true")
    if o.get("output_bytes") != REGISTERED_OUT:
        p.append(f"output {o.get('output_bytes')!r} bytes, not the registered {REGISTERED_OUT}")
    cr = tl.get("complete_rounds")
    if not (number(cr) and float(cr).is_integer() and cr >= 1):
        p.append(f"complete rounds {cr!r}, not at least 1")
    bad = [k for k in TRACE_SHARES if not (number(tl.get(k)) and 0 <= tl[k] <= 1)]
    bad += [k for k in TRACE_NONNEG + TRACE_P if not (number(tl.get(k)) and tl[k] >= 0)]
    if bad:
        p.append("metrics absent or out of range: " + ", ".join(bad))
    if number(tl.get("blocks")) and tl["blocks"] != REGISTERED_BLOCKS:
        p.append(f"blocks {tl['blocks']!r}, not the registered {REGISTERED_BLOCKS}")
    pm, pmed = tl.get("svc.backlog.prep_max"), tl.get("svc.backlog.prep_median")
    if number(pm) and number(pmed) and not (1 <= pmed <= pm <= ceiling(TRACES[name])):
        p.append(f"summary prepared count: median of round maxima {pm!r}, median of round medians {pmed!r}, not within 1 <= median <= max <= {ceiling(TRACES[name])}")
    rounds, rp = check_raw(o, name, tl)
    p += rp
    out = {k: v for k, v in tl.items() if number(v)}
    # (the rounds asked for, reported beside the complete ones: at least one complete round
    # is the inherited rule, and fewer than asked is lost precision, not invalidity)
    if number(st.get("rounds")):
        out["requested_rounds"] = st["rounds"]
    return out, rounds, p


def fig(c):
    g, lo, hi = c["speed"]
    return f"{c['name']} {g:.3f} [{lo:.3f}, {hi:.3f}], {speed_phrase(c['speed_outcome'])}; {cpu_phrase(c['cpu_outcome'])}"


def readings(by):
    """The registration's reading, mechanically, from LA3 and LA4 (svc2 at admission
    thresholds 3 and 4 against 2, all at window 8), in the registered order"""
    both = [by["LA3"], by["LA4"]]
    words = [c["speed_outcome"] for c in both]
    gains = [c for c in both if is_gain(c["speed_outcome"])]
    regressions = [c for c in both if c["speed_outcome"] == "REGRESSION"]
    figs = "; ".join(fig(c) for c in both)
    note = (
        "The threshold admits preparations (a service worker starts one while fewer than it are "
        f"prepared); it doesn't cap the queue, which can hold up to threshold + {SERVICES - 1}: "
        "see each trace's prepared count. A reading is the effect of the implemented higher-threshold "
        "policy (which also sizes more front sets), not proof that the prepared queue's supply is the "
        "bottleneck; each contrast is at its own nominal 95%, not a familywise claim for either"
    )
    if gains and not regressions:
        if any(c["cpu_outcome"] == "CPU COST" for c in gains):
            overall = f"A HIGHER ADMISSION THRESHOLD IS FASTER FOR A CPU COST (registered step 1b: {figs}): for Louis to weigh. {note}"
        else:
            at = " and ".join(f"{ADMISSION[c['arm']]}" for c in gains)
            overall = f"THE ADMISSION THRESHOLD BINDS SVC2@10 AT 2 (registered step 1a: a gain at {at}; {figs}). {note}"
    elif gains and regressions:
        overall = f"INCONCLUSIVE (registered step 2: MIXED, a gain at one threshold and a regression at the other; {figs})"
    elif any(w.startswith("INCONCLUSIVE") for w in words):
        overall = f"INCONCLUSIVE (registered step 3: {figs})"
    elif all(w == "REGRESSION" for w in words):
        overall = f"A HIGHER ADMISSION THRESHOLD IS SLOWER (registered step 4: {figs}). {note}"
    else:
        overall = (
            f"NO GAIN DETECTED FROM A HIGHER ADMISSION THRESHOLD (registered step 5: {figs}). NULL is bounded "
            f"uncertainty: a small effect is not excluded. {note}"
        )
    return {"overall": overall}


def best_arm(sec):
    """The svc2 arm fastest against the default on the geometric mean (the five D-), reported,
    never judged: chosen after the run from five, so its interval is optimistic"""
    rows = [c for c in sec if c["name"] in AGAINST_DEFAULT and number(c["speed"][0])]
    if not rows:
        return "NOT AVAILABLE"
    b = max(rows, key=lambda c: c["speed"][0])
    g, lo, hi = b["speed"]
    cg, clo, chi = b["cpu"]
    return (
        f"{label(b['arm'])} (`{b['arm']}`): speed {g:.3f} [{lo:.3f}, {hi:.3f}], {outcome(g, lo, hi)}; "
        f"CPU {cg:.3f} [{clo:.3f}, {chi:.3f}], {outcome(cg, clo, chi, gain='CPU SAVING', loss='CPU COST')} "
        "against the default. Reported, never judged: the best of five chosen after the run, so its interval is optimistic"
    )


def mechanism(modes, judged, overall):
    """The traces read beside the reading (modes: cell -> failure modes; judged: LA3 and
    LA4), reported, never judged. The gaining arms and the relieved arms are named
    separately; concordant evidence is printed only for arms that are both (revision of
    9 November, H3), and even then it is consistency, not mediation."""
    base = "mibz2-svc2@10"
    starved = [c for c, m in modes.items() if m["prep"] == "PREP STARVATION"]
    saturated = [label(c) for c, m in modes.items() if m["saturated"] == "SATURATED"]
    sat = f" SATURATED at {', '.join(saturated)}." if saturated else ""
    at = ", ".join(label(c) for c in starved) or "none"
    higher = [j["arm"] for j in judged]
    gaining = [j["arm"] for j in judged if is_gain(j["speed_outcome"])]
    relieved = [c for c in higher if base in starved and c in modes and c not in starved]
    both = [c for c in gaining if c in relieved]
    names = lambda cs: ", ".join(label(c) for c in cs) or "none"  # noqa: E731
    arms = f"Gaining: {names(gaining)}. Relieved of prep starvation (starved at a2 w8): {names(relieved)}."
    if overall.startswith(("THE ADMISSION THRESHOLD BINDS", "A HIGHER ADMISSION THRESHOLD IS FASTER")):
        if both:
            return (
                f"CONCORDANT AT {names(both)}: a gain and relief of prep starvation in the same arm. {arms} "
                "Consistent with the higher threshold acting on the teams' supply, not shown to be it "
                f"(the policy also sizes more front sets).{sat}"
            )
        return (
            f"NOT CONCORDANT: no arm both gains and is relieved. {arms} No combined reading. "
            f"PREP STARVATION at {at}.{sat}"
        )
    if overall.startswith(("NO GAIN DETECTED", "A HIGHER ADMISSION THRESHOLD IS SLOWER")) and all(
        c in starved for c in [base] + higher
    ):
        return (
            "PREP STARVATION PERSISTS AT EVERY THRESHOLD: not relieved by admitting more preparations. The "
            f"service workers' order of work (finishing first) remains a candidate, untested here. {arms}{sat}"
        )
    return f"PREP STARVATION at {at}: reported, not read. {arms}{sat}"


def report(a):
    runs, unparsed = [], 0
    for x in open(a.runs):
        if x.strip():
            try:
                o = json.loads(x)
            except ValueError:
                o = None
            if isinstance(o, dict):
                runs.append(o)
            else:
                unparsed += 1
    problems = {i: row_problems(r) for i, r in enumerate(runs)}
    malformed = [(i, p) for i, p in problems.items() if p]
    usable = [r for i, r in enumerate(runs) if not problems[i] and not r.get("err")]
    bad = [r for r in runs if r.get("err")]
    hashes = {r["hash"] for r in usable}
    lengths = {r["out"] for r in usable}
    ref_hashes = {r["hash"] for r in usable if cell(r) == REFERENCE}
    by = defaultdict(dict)
    dupes = defaultdict(int)
    seen = defaultdict(set)
    for i, r in enumerate(runs):
        if "no arm" in problems[i] or any(x.startswith(("`threads`", "`round`")) for x in problems[i]):
            continue
        c = cell(r)
        if r["round"] in seen[c]:
            dupes[c] += 1
            continue
        seen[c].add(r["round"])
        if not problems[i] and not r.get("err"):
            by[c][r["round"]] = r
    want = list(range(REGISTERED_ROUNDS))
    w = os.path.dirname(os.path.abspath(a.runs))

    def text(name):
        try:
            return open(os.path.join(w, name)).read().strip()
        except OSError:
            return None

    try:
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=True
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        head = None
    rounds_seen = len({r.get("round") for r in runs})
    lines = ["# The service core's admission threshold: the workstation run's report", ""]
    checks = []
    checks.append(
        (f"the registered {REGISTERED_ROUNDS} rounds (seen: {rounds_seen})", rounds_seen == REGISTERED_ROUNDS)
    )
    checks.append(("a clean tree (`git-status.txt` present and empty)", text("git-status.txt") == ""))
    checks.append(("no `NOT-FROZEN.txt` (the frozen inputs unchanged)", text("NOT-FROZEN.txt") is None))
    checks.append(("no `DRY-RUN.txt` (not a dry run)", text("DRY-RUN.txt") is None))
    commit = text("commit.txt")
    checks.append((f"`commit.txt` ({commit}) is the checkout's head ({head})", bool(commit) and commit == head))
    lscpu = text("lscpu.txt") or ""

    def lscpu_field(name):
        m = re.search(rf"^{name}:\s*(.+)$", lscpu, re.M)
        return m.group(1).strip() if m else None

    sockets, per_socket, l3 = lscpu_field("Socket\\(s\\)"), lscpu_field("Core\\(s\\) per socket"), lscpu_field("L3 cache") or lscpu_field("L3")
    ten = sockets == "1" and per_socket == "10" and bool(l3) and l3.split()[0:2] in (["20", "MiB"], ["20"])
    checks.append(
        (
            f"the ten-core topology, so `mibz2` is the pipelined default (lscpu: {sockets} socket, {per_socket} cores, L3 {l3})",
            ten,
        )
    )
    why = "; ".join(f"run {i}: {', '.join(p)}" for i, p in malformed[:3]) + ("; ..." if len(malformed) > 3 else "")
    checks.append(
        (
            f"every run's record usable: a hex output digest, the registered {REGISTERED_OUT} bytes out, finite "
            f"positive wall, CPU and peak RSS, the registered corpus `{REGISTERED_CORPUS}` "
            f"({len(malformed)} unusable, {unparsed} unparsed{': ' + why if why else ''})",
            bool(runs) and not malformed and not unparsed,
        )
    )
    checks.append(
        (
            f"every run the same bytes as the reference `{REFERENCE}`'s, none failed "
            f"(digests: {', '.join(sorted(hashes)) or 'none'})",
            len(hashes) == 1 and hashes == ref_hashes and lengths == {REGISTERED_OUT} and not bad,
        )
    )
    digest = (text("corpus.sha256") or "").split()
    checks.append(
        (
            f"`corpus.sha256` the registered corpus's ({REGISTERED_CORPUS_SHA256[:12]}..., bench/wikidata.json; "
            f"seen: {digest[0][:12] + '...' if digest else 'none'})",
            bool(digest) and digest[0] == REGISTERED_CORPUS_SHA256,
        )
    )
    for c, plan in CELLS.items():
        rs = by.get(c, {})
        plans = {threads_of(r.get("thread_cpu", "")) for r in rs.values()}
        ok = sorted(rs) == want and not dupes[c] and plans == {plan}
        seen_ = ", ".join(f"{p[0]} leaders, {p[1]} helpers, {p[2]} service" for p in sorted(plans)) or "none"
        checks.append(
            (
                f"`{c}`: rounds 0-{REGISTERED_ROUNDS - 1} once each, every run {plan[0]} leaders, "
                f"{plan[1]} helpers, {plan[2]} service threads (seen: {seen_})",
                ok,
            )
        )
    unexpected = sorted(set(by) - set(CELLS))
    checks.append((f"no cell beyond the registered ones (seen: {', '.join(unexpected) or 'none'})", not unexpected))
    tls, modes, unreadable, per_round = {}, {}, [], {}
    for name, c in TRACES.items():
        tl, rounds_, tp = load_trace(a.occupancy or [], name, commit)
        tls[name] = tl
        per_round[name] = rounds_
        unreadable += [f"{name}: {x}" for x in tp if x.startswith("unreadable")]
        checks.append(
            (
                f"the trace `{name}` complete and as registered ({TRACE_THREADS} threads, {TRACE_PLAN}, "
                f"{TRACE_SERVICES} service threads, slots {TRACE_SLOTS[name]} (a window of {WINDOW[c]}), admission "
                f"{TRACE_ADMISSION[name]}, the run's commit, every scheduled child once with exit status 0, every complete "
                f"round's prepared count within 1 <= median <= max <= {ceiling(c)} and {REGISTERED_BLOCKS} blocks, the summary "
                f"the median of those rounds, "
                f"parity, the registered output, every metric read finite and in range; rounds: "
                f"{tl.get('complete_rounds', 0)}{'; ' + '; '.join(tp) if tp else ''})",
                not tp,
            )
        )
        modes[c] = failure_modes(tl)
        checks.append(
            (f"saturation resolved from `{name}` (seen: {modes[c]['saturated']})", modes[c]["saturated"] != "UNRESOLVED")
        )
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
        f"{g:.3f} [{lo:.3f}, {hi:.3f}] over {n} rounds, within 1 ± {CONTROL_EQUIV}: **{'yes' if control_ok else 'NO'}**"
    )
    verdict = "VALID" if valid and control_ok else "INVALID"
    lines += ["", f"Run: **{verdict}**", ""]
    out = {"verdict": verdict, "checks": checks, "judged": [], "secondary": [], "traces": {}}

    lines += [
        "## The registered comparisons (svc2 at 2 × 3 + 2 on 10, window 8: a higher admission threshold against 2; judged; speed and CPU co-primary)",
        "",
        "| | arm | against | threshold, window | speed [95% CI] | speed outcome | CPU [95% CI] | CPU outcome |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for name, arm, ref in JUDGED:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        if verdict == "VALID":
            so = outcome(*speed)
            co = outcome(*cpu, gain="CPU SAVING", loss="CPU COST")
        else:
            so = co = "NONE (the run is invalid)"
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {label(arm)} against {label(ref)} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | "
            f"{so} | {cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] | {co} |"
        )
        out["judged"].append(
            {"name": name, "arm": arm, "ref": ref, "admission": [ADMISSION[arm], ADMISSION[ref]],
             "windows": [WINDOW[arm], WINDOW[ref]], "speed": list(speed),
             "cpu": list(cpu), "rounds": n, "speed_outcome": so, "cpu_outcome": co}
        )
    jd = {c["name"]: c for c in out["judged"]}
    for name, arm, ref in SECONDARY:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        so = outcome(*speed) if verdict == "VALID" else "NONE (the run is invalid)"
        out["secondary"].append(
            {"name": name, "arm": arm, "ref": ref, "speed": list(speed), "cpu": list(cpu), "speed_outcome": so}
        )
    show = verdict == "VALID"
    if show:
        out["readings"] = readings(jd)
        out["best_arm"] = best_arm(out["secondary"])
        out["mechanism"] = mechanism(modes, out["judged"], out["readings"]["overall"])
    else:
        out["readings"] = {"overall": "NONE (the run is invalid)"}
        out["best_arm"] = out["mechanism"] = "NONE (the run is invalid)"
    lines += [
        "",
        f"**The reading, by the registered order:** **{out['readings']['overall']}**",
        "",
        "NULL is an interval inside [0.97, 1.03]: bounded uncertainty, not an effect shown to be zero. "
        "CPU INCONCLUSIVE is no detected CPU cost, not a CPU cost shown to be absent.",
        "",
        f"**The best svc2 arm against the default (reported, never judged):** {out['best_arm']}",
        "",
        f"**The traces beside it (reported, never judged):** {out['mechanism']}",
        "",
        "## Reported, never judged",
        "",
        "| | arm | against | decode threads | threshold, window | speed [95% CI] | speed outcome | CPU [95% CI] |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for c in out["secondary"]:
        arm, ref, speed, cpu = c["arm"], c["ref"], c["speed"], c["cpu"]
        lines.append(
            f"| {c['name']} | `{arm}` | `{ref}` | {decode_threads(CELLS[arm])} against {decode_threads(CELLS[ref])} | "
            f"{label(arm)} against {label(ref)} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | {c['speed_outcome']} | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] |"
        )
    lines += [
        "",
        "(Speed: the reference's wall over the arm's, > 1 the arm is faster. CPU: the reference's CPU over the arm's, > 1 the arm spends less.)",
        "",
        "## Every cell (medians)",
        "",
        "| cell | decode threads (leaders + helpers + service) | threshold, window | wall s | CPU s | peak RSS MB | rounds |",
        "|---|---|---|---|---|---|---|",
    ]
    for c in sorted(by):
        rs = list(by[c].values())
        if not rs:
            continue
        plan = CELLS.get(c)
        lines.append(
            f"| `{c}`{'' if plan else ' (unregistered)'} | {'+'.join(map(str, plan)) if plan else '?'} | "
            f"{label(c) if plan else '?'} | {statistics.median(r['wall'] for r in rs):.3f} | "
            f"{statistics.median(r['cpu'] for r in rs):.3f} | {statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} |"
        )
    lines += [
        "",
        "## The traces (occupancy, svc2 at 10 threads, one per arm; medians over complete rounds)",
        "",
        f"The prepared count P is what each preparation's push left; it can exceed the admission threshold by up to {SERVICES - 1} "
        "(overlapping preparations), so its ceiling is threshold + services - 1, checked in every complete round. "
        "P max is the most any complete round reached; the trace's own summaries are the median of the rounds' maxima "
        "and the median of the rounds' push medians (not a pooled median, and it may be half-integral). Reaching the "
        "ceiling shows it is reachable, not that it binds often.",
        "",
        "| trace | threshold, window | P: max over rounds / median of round maxima / median of round medians (ceiling) | complete rounds of requested | blocks | util (each; max) | prep wait median / p90 ms | finish wait median / p90 ms | job wait median ms | backlog prep / finish (mean; max) | teams idle: prep / back share | run wait ms | saturation | prep starvation | finish backlog |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for name, c in TRACES.items():
        tl = tls[name]
        if not tl:
            continue
        g = lambda k: tl.get(k, float("nan"))  # noqa: E731
        fm = modes[c]
        pr = per_round.get(name, [])
        p_max = max((r["p_max"] for r in pr), default=float("nan"))
        out["traces"][name] = {
            "cell": c,
            "admission": ADMISSION[c],
            "window": WINDOW[c],
            "p_max": p_max,
            "p_median_of_round_maxima": g("svc.backlog.prep_max"),
            "p_median_of_round_medians": g("svc.backlog.prep_median"),
            "p_ceiling": ceiling(c),
            "rounds": pr,
            "requested_rounds": g("requested_rounds"),
            "timeline": {k: v for k, v in tl.items() if k.startswith("svc.") or k in ("blocks", "complete_rounds")},
            "modes": fm,
        }
        utils = sorted((k, v) for k, v in tl.items() if re.fullmatch(r"svc\.util\.\d+", k))
        u = ", ".join(f"{v:.2f}" for _, v in utils)
        lines.append(
            f"| {name} | {label(c)} | {p_max:.0f} / {g('svc.backlog.prep_max'):.1f} / {g('svc.backlog.prep_median'):.1f} ({ceiling(c)}) | "
            f"{g('complete_rounds'):.0f} of {g('requested_rounds'):.0f} | {g('blocks')} | {u}; max {g('svc.util.max'):.2f} | "
            f"{g('svc.queue.prep.median_ms'):.3f} / {g('svc.queue.prep.p90_ms'):.3f} | "
            f"{g('svc.queue.finish.median_ms'):.3f} / {g('svc.queue.finish.p90_ms'):.3f} | {g('svc.queue.job.median_ms'):.3f} | "
            f"{g('svc.backlog.prep_mean'):.2f} / {g('svc.backlog.finish_mean'):.2f} ({g('svc.backlog.prep_max'):.0f}; {g('svc.backlog.finish_max'):.0f}) | "
            f"{g('svc.team.idle_prep_share'):.3f} / {g('svc.team.idle_back_share'):.3f} | {g('svc.team.ms.run_wait'):.1f} | "
            f"{fm['saturated'] if show else 'NONE'} | {fm['prep'] if show else 'NONE'} | {fm['finish'] if show else 'NONE'} |"
        )
    lines += [f"| {x} | | | | | | | | | | | | | | |" for x in unreadable]
    text_ = "\n".join(lines) + "\n"
    print(text_)
    if a.out:
        open(a.out + ".md", "w").write(text_)
        json.dump(out, open(a.out + ".json", "w"), indent=1)
    return 0 if verdict == "VALID" else 1


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--occupancy", nargs="*", default=[])
    r.add_argument("--out")
    a = p.parse_args()
    sys.exit(report(a))


if __name__ == "__main__":
    main()
