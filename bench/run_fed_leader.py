"""The run-fed leader's A/B (docs/journal/2026-11-09-run-fed-leader.md, a DRAFT
registration, NOT FROZEN): its admission, its paired ratios, the control contrasts, the
leaders' phases and the shares' census, and the reading the registration sets.

    python3 bench/run_fed_leader.py report RUNS.jsonl --head SHA --identity ID.json [--out STEM]
    python3 bench/run_fed_leader.py report RUNS.jsonl --unjudged [--rounds N] [--identity ID.json] [--out STEM]

The first form is the judged run: bound to the registration (the frozen head, the build's
identity, the corpus's digest, ROUNDS rounds, the arms in ab's rotation, at THREADS
threads and the 3 reported). The second is any other run (a smoke, a run before the
freeze): the same admission but for the head and the round count, and its reading is
labelled UNJUDGED.

RUNS.jsonl is one `ab run` of the arms below, interleaved by round, from a build of the
harness with the `run-fed` feature. ID.json is the run's identity, written by the sweep
workflow around the run (`schema` IDENTITY_SCHEMA): the head, a clean tree, the build's
features and profile, rustc, and the harness binary's and the corpus's SHA-256 before and
after the run.

    mibz2-default33          the incumbent, the fixed reference of every judged ratio:
                             `BzReader::parallel` with PIPELINED_PLAN forced
    mibz2-pipe3-both         the shipped worker through the research constructor
    mibz2-pipe3-runfed-ctl   this module's worker, no records, the shipped MTF kernel
    mibz2-pipe3-runfed-reg   no records, the register-list checkpointed MTF (C29)
    mibz2-pipe3-runfed-ls    the sort on the leader
    mibz2-pipe3-runfed-hs    the sort on the helpers (C30): secondary
    mibz2-pipe3-runfed-dir   no F-order array (C31), shares cut by C35: the PRIMARY arm

VALID only if every predicate holds; any failure prints INVALID with its reasons, exits 2,
and no reading is given. Every measurement is checked for presence and type before any
arithmetic, so an absent or malformed one is a reason, never a crash. See `admit`.

The reading is always the primary arm against the incumbent (never another denominator).
The control contrasts (each record arm, and reg, over ctl; ctl over the incumbent and over
pipe3-both) are reported apart, and attribution to the run-fed construction is stated
explicitly: it holds only if ctl's wall interval lies inside the fidelity band and the
primary's wall over ctl's excludes 1 from above; otherwise the reading says so.
"""

import argparse
import json
import math
import re
import statistics
import sys
from collections import defaultdict

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from team_size import T975  # noqa: E402

REF = "mibz2-default33"
BOTH = "mibz2-pipe3-both"
CTL = "mibz2-pipe3-runfed-ctl"
REG = "mibz2-pipe3-runfed-reg"
ARMS = [
    BOTH,
    CTL,
    REG,
    "mibz2-pipe3-runfed-ls",
    "mibz2-pipe3-runfed-hs",
    "mibz2-pipe3-runfed-dir",
]
# ab's --arms order, which its rotation indexes
REGISTERED_ARMS = [REF] + ARMS
RUNFED = [a for a in ARMS if "-runfed-" in a]
RECORD_ARMS = ["mibz2-pipe3-runfed-ls", "mibz2-pipe3-runfed-hs", "mibz2-pipe3-runfed-dir"]
PRIMARY = "mibz2-pipe3-runfed-dir"
SECONDARY = ["mibz2-pipe3-runfed-hs", "mibz2-pipe3-runfed-ls"]

# The registration (DRAFT, NOT FROZEN)
ROUNDS = 15
THREADS = 9  # judged: three teams of three
REPORTED_THREADS = 3  # reported, never judged
CORPUS = "wikidata"
CORPUS_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
IDENTITY_SCHEMA = "run_fed_leader/identity/1"
FEATURES = ["run-fed"]
PROFILE = "release"
# (leaders, plain helpers, pipelined helpers) every 9-thread run must be seen running
PLAN = (3, 0, 6)
POSITIVE_WALL = 0.97  # LEAD: the geometric mean at most this, its interval's high end below 1
CPU_CAP = 1.05  # and the CPU ratio's high end at most this
NEGATIVE_WALL = 0.99  # STOP: the wall ratio's low end above this (no 1% cut)
FIDELITY = (0.98, 1.02)  # ctl over the incumbent: its whole wall interval inside this

# Every phase the run-fed arms report (src/parallel/run_team.rs's `phases!`; its test
# `phases_are_the_reporters` checks this list against it)
PHASES = [
    "runs", "team_records", "team_column", "small_records", "small_column", "fallbacks",
    "lead_ns", "lead_front_ns", "lead_sort_ns", "lead_rle_ns", "lead_small_ns",
    "join_wait_ns", "lead_start_ns", "lead_end_ns", "build_end_ns", "run_ns", "leader_late",
    "helper_sort_ns", "helper_fill_ns", "helper_fused_ns", "drained_front_ns", "records",
    "rows", "stores", "lookups", "cells", "shares", "share_records_max", "share_rows_max",
    "share_cells_max", "share_stores_max", "ready_last_ns", "ready_skew_ns", "rule_blocks",
    "rule_front_ns", "rule_sort_ns",
]  # fmt: skip
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")


def is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def positive(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def geo_ci(ratios):
    """The geometric mean and its 95% t-interval; None where it can't be computed"""
    if len(ratios) < 2 or any(not positive(r) for r in ratios):
        return None
    logs = [math.log(r) for r in ratios]
    m = statistics.fmean(logs)
    h = T975[min(len(logs) - 1, 30) - 1] * statistics.stdev(logs) / math.sqrt(len(logs))
    out = (math.exp(m), math.exp(m - h), math.exp(m + h))
    return out if all(math.isfinite(x) for x in out) else None


def arm_order(rnd, n):
    """ab's `arm_order`: rotated a step a round, reversed in every other cycle of n"""
    order = list(range(n))
    if n == 0:
        return order
    order = order[rnd % n :] + order[: rnd % n]
    if (rnd // n) % 2 == 1:
        order.reverse()
    return order


def plan_of(names):
    """(leaders, plain helpers, pipelined helpers) from ab's `observed_threads`"""
    ns = [p.partition("=")[0] for p in names.split()]
    return (
        sum(1 for n in ns if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in ns if re.fullmatch(r"mibz2-coop-\d+", n)),
        sum(1 for n in ns if re.fullmatch(r"mibz2-coop-p\d+", n)),
    )


def parse_phases(s):
    """The phases string's values, or (None, reason): every token `name=integer`, every
    registered name once, no other"""
    if not isinstance(s, str):
        return None, f"phases {type(s).__name__}, not a string"
    out = {}
    for tok in s.split():
        k, eq, v = tok.partition("=")
        if not eq or not re.fullmatch(r"-?\d+", v):
            return None, f"phase token {tok!r} is not name=integer"
        if k in out:
            return None, f"phase {k} twice"
        out[k] = int(v)
    missing = [k for k in PHASES if k not in out]
    extra = [k for k in out if k not in PHASES]
    if missing:
        return None, f"phases missing: {', '.join(missing)}"
    if extra:
        return None, f"unregistered phases: {', '.join(extra)}"
    neg = [k for k, v in out.items() if v < 0]
    if neg:
        return None, f"negative phases: {', '.join(neg)}"
    return out, None


def phase_problems(arm, p):
    """A run-fed arm's phases: consistent counts, present timings"""
    bad = []
    runs = p["runs"]
    if runs == 0:
        return ["no team runs"]
    if p["team_records"] + p["team_column"] != runs:
        bad.append(
            f"team_records {p['team_records']} + team_column {p['team_column']} != runs {runs}"
        )
    if p["fallbacks"] != 0:
        bad.append(f"{p['fallbacks']} fallbacks (a refused check's rebuild: a bug)")
    if p["leader_late"] > runs:
        bad.append(f"leader_late {p['leader_late']} > runs {runs}")
    for k in ("lead_ns", "lead_end_ns", "build_end_ns", "run_ns"):
        if p[k] == 0:
            bad.append(f"{k} absent (0) with {runs} runs")
    if p["lead_end_ns"] < p["lead_start_ns"]:
        bad.append("lead_end_ns < lead_start_ns")
    if arm in RECORD_ARMS:
        if p["team_records"] == 0:
            bad.append("no team block by records")
        else:
            for k in ("records", "rows", "shares", "cells", "ready_last_ns"):
                if p[k] == 0:
                    bad.append(f"{k} absent (0) with {p['team_records']} team blocks by records")
            if p["stores"] < p["rows"]:
                bad.append(f"stores {p['stores']} < rows {p['rows']} (W >= n)")
            if p["share_rows_max"] > p["rows"]:
                bad.append("share_rows_max > rows")
            if p["ready_skew_ns"] > p["ready_last_ns"]:
                bad.append("ready_skew_ns > ready_last_ns")
            if arm.endswith("-dir") and p["lookups"] == 0:
                bad.append("C31 made no lookups")
    elif p["team_records"] != 0 or p["records"] != 0:
        bad.append("a control took records")
    return bad


def record_problems(rec, where, judged_threads):
    """One run's typed raw evidence"""
    bad = []
    if rec.get("broken"):
        return [f"{where}: a broken line (the child printed none)"]
    if not (is_int(rec.get("exit")) and rec["exit"] == 0):
        bad.append(f"{where}: exit {rec.get('exit')!r}")
    if rec.get("err") != "":
        bad.append(f"{where}: err {rec.get('err')!r}")
    if not (isinstance(rec.get("hash"), str) and rec["hash"]):
        bad.append(f"{where}: hash {rec.get('hash')!r}")
    if not (is_int(rec.get("out")) and rec["out"] > 0):
        bad.append(f"{where}: out {rec.get('out')!r}")
    for k in ("wall", "cpu"):
        if not positive(rec.get(k)):
            bad.append(f"{where}: {k} {rec.get(k)!r}, not finite and positive")
    if rec.get("corpus") != CORPUS:
        bad.append(f"{where}: corpus {rec.get('corpus')!r}, not {CORPUS!r}")
    obs = rec.get("observed_threads")
    if not isinstance(obs, str):
        bad.append(f"{where}: no observed_threads")
    elif rec.get("threads") == judged_threads and plan_of(obs) != PLAN:
        bad.append(f"{where}: ran {plan_of(obs)} (leaders, plain, pipelined helpers), not {PLAN}")
    if rec.get("arm") in RUNFED:
        p, why = parse_phases(rec.get("phases"))
        if p is None:
            bad.append(f"{where}: {why}")
        else:
            bad += [f"{where}: {b}" for b in phase_problems(rec["arm"], p)]
    return bad


def load(path):
    """Every record in file order, and the lines that aren't JSON objects"""
    recs, problems = [], []
    try:
        f = open(path)
    except OSError as e:
        return [], [f"the runs file does not open: {e}"]
    with f:
        for i, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                problems.append(f"line {i}: not JSON")
                continue
            if not isinstance(rec, dict):
                problems.append(f"line {i}: not an object")
                continue
            rec["_line"] = i
            recs.append(rec)
    return recs, problems


def identity_problems(ident, head, judged, rounds):
    """The run's identity against the registration"""
    bad = []
    if not isinstance(ident, dict):
        return ["the identity is not an object"]
    if ident.get("schema") != IDENTITY_SCHEMA:
        bad.append(f"identity schema {ident.get('schema')!r}, not {IDENTITY_SCHEMA}")
    ih = ident.get("head")
    if not (isinstance(ih, str) and HEX40.fullmatch(ih)):
        bad.append(f"the run's head {ih!r} is not a full 40-character commit")
    elif judged and ih != head:
        bad.append(f"the run's head {ih} is not the registered {head}")
    if ident.get("tree_clean") is not True:
        bad.append("the checkout's tree was not clean")
    if ident.get("features") != FEATURES:
        bad.append(f"features {ident.get('features')!r}, not {FEATURES}")
    if ident.get("profile") != PROFILE:
        bad.append(f"profile {ident.get('profile')!r}, not {PROFILE}")
    if not (isinstance(ident.get("rustc"), str) and ident["rustc"].startswith("rustc ")):
        bad.append(f"rustc {ident.get('rustc')!r} not recorded")
    b0, b1 = ident.get("binary_sha256_before"), ident.get("binary_sha256_after")
    if not (isinstance(b0, str) and HEX64.fullmatch(b0) and b0 == b1):
        bad.append(f"the harness binary's SHA-256 {b0!r} before, {b1!r} after: not one build")
    c0, c1 = ident.get("corpus_sha256_before"), ident.get("corpus_sha256_after")
    if not (c0 == c1 == CORPUS_SHA256):
        bad.append(
            f"the corpus's SHA-256 {c0!r} before, {c1!r} after, not the registered {CORPUS_SHA256}"
        )
    if ident.get("arms") != REGISTERED_ARMS:
        bad.append(f"arms {ident.get('arms')!r}, not the registered {REGISTERED_ARMS}")
    if ident.get("threads") != [THREADS, REPORTED_THREADS]:
        bad.append(f"threads {ident.get('threads')!r}, not {[THREADS, REPORTED_THREADS]}")
    if ident.get("rounds") != rounds:
        bad.append(f"the identity's rounds {ident.get('rounds')!r}, not {rounds}")
    return bad


def admit(recs, rounds):
    """The records' problems (none: admitted), and the runs by (threads, arm, round)"""
    bad = []
    runs = defaultdict(dict)
    order = defaultdict(list)  # (threads, round) -> arms in file order
    for rec in recs:
        t, arm, rnd = rec.get("threads"), rec.get("arm"), rec.get("round")
        where = f"line {rec['_line']} ({arm} t{t} r{rnd})"
        if not (is_int(t) and t in (THREADS, REPORTED_THREADS)):
            bad.append(f"{where}: threads {t!r}, not {THREADS} or {REPORTED_THREADS}")
            continue
        if arm not in REGISTERED_ARMS:
            bad.append(f"{where}: an unregistered arm")
            continue
        if not (is_int(rnd) and 0 <= rnd < rounds):
            bad.append(f"{where}: round {rnd!r}, not 0..{rounds - 1}")
            continue
        if rnd in runs[(t, arm)]:
            bad.append(f"{where}: twice")
            continue
        runs[(t, arm)][rnd] = rec
        order[(t, rnd)].append(arm)
        bad += record_problems(rec, where, THREADS)
    for t in (THREADS, REPORTED_THREADS):
        for arm in REGISTERED_ARMS:
            got = set(runs[(t, arm)])
            if got != set(range(rounds)):
                missing = sorted(set(range(rounds)) - got)
                bad.append(f"{arm} t{t}: rounds missing {missing}")
        for rnd in range(rounds):
            want = [REGISTERED_ARMS[i] for i in arm_order(rnd, len(REGISTERED_ARMS))]
            if order[(t, rnd)] and order[(t, rnd)] != want:
                bad.append(f"t{t} r{rnd}: the arms ran {order[(t, rnd)]}, not ab's rotation {want}")
    # one output: every run's hash and length the incumbent's in its round, and the
    # incumbent's the same in every round
    ref = runs[(THREADS, REF)]
    outs = {
        (r.get("hash"), r.get("out"))
        for t in (THREADS, REPORTED_THREADS)
        for r in runs[(t, REF)].values()
    }
    if len(outs) > 1:
        bad.append(f"{REF}: its outputs differ across rounds or threads: {sorted(map(str, outs))}")
    for t in (THREADS, REPORTED_THREADS):
        for arm in ARMS:
            for rnd, r in runs[(t, arm)].items():
                ref_r = ref.get(rnd)
                if ref_r is not None and (r.get("hash"), r.get("out")) != (
                    ref_r.get("hash"),
                    ref_r.get("out"),
                ):
                    bad.append(f"{arm} t{t} r{rnd}: output differs from the incumbent's")
    return bad, runs


def paired(runs, t, num, den, key, rounds):
    return [runs[(t, num)][i][key] / runs[(t, den)][i][key] for i in range(rounds)]


def reading_of(w, c):
    if w[0] <= POSITIVE_WALL and w[2] < 1 and c[2] <= CPU_CAP:
        return "LEAD"
    if w[1] > NEGATIVE_WALL:
        return "STOP"
    return "INCONCLUSIVE"


def phases_of(runs, arm, rounds):
    """The arm's phase totals, each a mean over rounds of the decode's totals"""
    per = [parse_phases(runs[(THREADS, arm)][i]["phases"])[0] for i in range(rounds)]
    return {k: statistics.fmean(p[k] for p in per) for k in PHASES}


def fmt(x):
    return "n/a" if x is None else f"{x[0]:.4f} [{x[1]:.4f}, {x[2]:.4f}]"


def report(args):
    judged = not args.unjudged
    rounds = ROUNDS if judged else args.rounds
    bad = []
    if judged:
        if args.rounds not in (None, ROUNDS):
            bad.append(f"a judged run is {ROUNDS} rounds, not {args.rounds}")
        if not (isinstance(args.head, str) and HEX40.fullmatch(args.head)):
            bad.append(f"the registered head {args.head!r} is not a full 40-character commit")
        if not args.identity:
            bad.append("a judged run needs its identity (--identity)")
    if not (is_int(rounds) and rounds >= 2):
        bad.append(f"rounds {rounds!r}: at least 2")
        rounds = 0
    ident = None
    if args.identity:
        try:
            with open(args.identity) as f:
                ident = json.load(f)
        except (OSError, ValueError) as e:
            bad.append(f"the identity does not load: {e}")
        else:
            bad += identity_problems(ident, args.head, judged, rounds)
    recs, problems = load(args.runs)
    bad += problems
    runs = {}
    if rounds:
        b, runs = admit(recs, rounds)
        bad += b
    label = "JUDGED" if judged else "UNJUDGED"
    raw = [{k: v for k, v in r.items() if k != "_line"} for r in recs]
    out = {
        "label": label,
        "head": args.head,
        "identity": ident,
        "threads": THREADS,
        "rounds": rounds,
        "invalid": bad,
        "raw": raw,  # every record, in file order (the arms' actual order)
    }
    cells = {}
    if not bad:
        for t in (THREADS, REPORTED_THREADS):
            ref = REF if t == THREADS else BOTH
            for arm in REGISTERED_ARMS:
                if arm == ref:
                    continue
                w = geo_ci(paired(runs, t, arm, ref, "wall", rounds))
                c = geo_ci(paired(runs, t, arm, ref, "cpu", rounds))
                if t == THREADS and (w is None or c is None):
                    bad.append(f"{arm} over {ref}: an absent or non-finite interval")
                cells[f"{arm}/{ref}@{t}"] = {"wall": w, "cpu": c}
        contrasts = {}
        for num, den in [(a, CTL) for a in RECORD_ARMS] + [(REG, CTL), (CTL, BOTH)]:
            contrasts[f"{num}/{den}"] = {
                "wall": geo_ci(paired(runs, THREADS, num, den, "wall", rounds)),
                "cpu": geo_ci(paired(runs, THREADS, num, den, "cpu", rounds)),
            }
    if bad:
        out["invalid"] = bad
        print("INVALID")
        for b in bad:
            print(f"  {b}")
        if args.out:
            with open(args.out + ".json", "w") as f:
                json.dump(out, f, indent=1)
        return 2

    print(f"VALID ({label}): {rounds} rounds at {THREADS} threads, against {REF} (fixed)")
    print(f"{'arm / ' + REF:<34} {'wall ratio [95%]':<26} {'CPU ratio [95%]':<26}")
    for arm in ARMS:
        x = cells[f"{arm}/{REF}@{THREADS}"]
        tag = " (PRIMARY)" if arm == PRIMARY else " (secondary)" if arm in SECONDARY else ""
        print(f"{arm + tag:<34} {fmt(x['wall']):<26} {fmt(x['cpu']):<26}")
    print("\nControl contrasts (mechanism, never the judged estimand):")
    for k, x in contrasts.items():
        print(f"  {k:<44} wall {fmt(x['wall'])}  CPU {fmt(x['cpu'])}")
    ctl = cells[f"{CTL}/{REF}@{THREADS}"]["wall"]
    fidelity = FIDELITY[0] <= ctl[1] and ctl[2] <= FIDELITY[1]
    p_ctl = contrasts[f"{PRIMARY}/{CTL}"]["wall"]
    beats_ctl = p_ctl is not None and p_ctl[2] < 1

    print(f"\nPhases, {THREADS} threads (mean over rounds; ms a team run unless a count):")
    ph = {arm: phases_of(runs, arm, rounds) for arm in RUNFED}
    print(f"{'':<26}" + "".join(f"{a.rsplit('-', 1)[1]:>9}" for a in RUNFED))

    def row(name, f):
        vals = []
        for a in RUNFED:
            try:
                vals.append(f"{f(ph[a]):>9.3f}")
            except ZeroDivisionError:
                vals.append(f"{'-':>9}")
        print(f"{name:<26}" + "".join(vals))

    per_run = lambda k: lambda p: p[k] / p["runs"] / 1e6  # noqa: E731
    for k in ("lead_ns", "lead_front_ns", "lead_sort_ns", "lead_rle_ns", "lead_small_ns",
              "lead_start_ns", "lead_end_ns", "build_end_ns", "join_wait_ns", "run_ns",
              "helper_sort_ns", "helper_fill_ns", "helper_fused_ns"):  # fmt: skip
        row(k, per_run(k))
    row("leader_late / runs", lambda p: p["leader_late"] / p["runs"])
    row("team_records / runs", lambda p: p["team_records"] / p["runs"])
    row("ready_last_ns (by recs)", lambda p: p["ready_last_ns"] / p["team_records"] / 1e6)
    row("ready_skew_ns (by recs)", lambda p: p["ready_skew_ns"] / p["team_records"] / 1e6)
    # the census: a run's largest share over its mean share, summed over runs
    h = lambda p: p["shares"] / p["team_records"]  # noqa: E731
    for k, tot in (
        ("records", "records"),
        ("rows", "rows"),
        ("cells", "cells"),
        ("stores", "stores"),
    ):
        row(f"{k}_h largest / mean", lambda p, k=k, tot=tot: p[f"share_{k}_max"] * h(p) / p[tot])
    row("W / n", lambda p: p["stores"] / p["rows"])
    row("lookups a block", lambda p: p["lookups"] / p["team_records"])
    print(
        "  (join_wait is the leader's wait for the build after its work: descheduling and\n"
        "  wake-up included, not construction alone. Every phase clock includes descheduling\n"
        "  on an oversubscribed runner; CPU is scheduled service (spinning, SMT, frequency\n"
        "  and cache interference included), not invariant work. leader_late compares the\n"
        "  leader's end with the build's end, both from the run's start.)"
    )

    w, c = cells[f"{PRIMARY}/{REF}@{THREADS}"]["wall"], cells[f"{PRIMARY}/{REF}@{THREADS}"]["cpu"]
    reading = reading_of(w, c)
    if reading == "LEAD":
        if fidelity and beats_ctl:
            attribution = "attributed: ctl matches the incumbent, and the primary beats ctl"
        else:
            why = []
            if not fidelity:
                why.append(f"ctl/{REF} wall {fmt(ctl)} not inside {list(FIDELITY)}")
            if not beats_ctl:
                why.append(f"{PRIMARY}/ctl wall {fmt(p_ctl)} does not exclude 1 from above")
            attribution = (
                "ATTRIBUTION FAILED: not attributable to the run-fed construction ("
                + "; ".join(why)
                + ")"
            )
    else:
        attribution = (
            "control fidelity held"
            if fidelity
            else f"control fidelity FAILED: ctl/{REF} wall {fmt(ctl)}"
        )
    print(f"\nPrimary ({PRIMARY} / {REF}, {label}): {reading}; {attribution}")
    for arm in SECONDARY:
        x = cells[f"{arm}/{REF}@{THREADS}"]
        print(f"  secondary ({arm}, never judged): {reading_of(x['wall'], x['cpu'])}")
    print(
        "  (environment-specific: a CI reading is this runner's; a STOP pauses, it cannot close\n"
        "  the target-machine line, PROTOCOL §11)"
    )
    print(f"\n{REPORTED_THREADS}-thread cells (one team), against {BOTH}, reported, never judged:")
    for arm in RUNFED:
        x = cells[f"{arm}/{BOTH}@{REPORTED_THREADS}"]
        print(f"  {arm:<30} wall {fmt(x['wall'])}  CPU {fmt(x['cpu'])}")
    out.update(
        {
            "cells": cells,
            "contrasts": contrasts,
            "phases": ph,
            "reading": reading,
            "attribution": attribution,
            "fidelity": fidelity,
        }
    )
    if args.out:
        with open(args.out + ".json", "w") as f:
            json.dump(out, f, indent=1)
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("runs")
    r.add_argument("--head", help="the registered (frozen) 40-character head: a judged run")
    r.add_argument("--identity", help="the run's identity (the sweep workflow's)")
    r.add_argument("--unjudged", action="store_true", help="not the registered run")
    r.add_argument("--rounds", type=int, help=f"an unjudged run's rounds (judged: {ROUNDS})")
    r.add_argument("--out")
    args = ap.parse_args()
    if args.unjudged and args.rounds is None:
        args.rounds = ROUNDS
    try:
        code = report(args)
    except Exception as e:  # (a reporter's bug is never a VALID: INVALID, its reason)
        print("INVALID")
        print(f"  the reporter failed: {type(e).__name__}: {e}")
        code = 2
    sys.exit(code)


if __name__ == "__main__":
    main()
