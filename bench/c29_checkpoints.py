"""C29's exposed-path diagnostic (docs/journal/2026-11-09-c29-register-checkpoints.md, a
DRAFT registration, NOT FROZEN): its admission, its paired ratios and the reading the
registration sets. Shaped on #270's reporter (research/run-fed-leader's
bench/run_fed_leader.py): the same identity, the same admission, the same reading rule.

    python3 bench/c29_checkpoints.py report RUNS.jsonl --head SHA --identity ID.json [--out STEM]
    python3 bench/c29_checkpoints.py report RUNS.jsonl --unjudged [--rounds N] [--identity ID.json] [--out STEM]

The first form is the judged run: bound to the registration (the frozen head, the build's
identity, the corpus's digest, ROUNDS rounds, the two arms in ab's rotation, at 9 and 3
threads). The second is any other run (a smoke, a run before the freeze): the same
admission but for the head and the round count, its reading labelled UNJUDGED.

RUNS.jsonl is one `ab run` of the two arms, interleaved by round, one build of the
harness (no features). ID.json is the run's identity, written by the sweep workflow
(`tool: c29`) around the run (`schema` IDENTITY_SCHEMA).

    mibz2-pipe3-both       the shipped pipelined teams (PIPELINED_LEVERS, park and small),
                           the shipped kernel `mtf_checkpoints`: the control, the
                           denominator of every ratio
    mibz2-pipe3-both-c29   the same worker, levers and window, `Levers::register_list` set:
                           the leader's front stage by `mtf_checkpoints_reg` (C29)

VALID only if every predicate holds; any failure prints INVALID with its reasons, exits 2,
and no reading is given. Every measurement is checked for presence and type before any
arithmetic, so an absent or malformed one is a reason, never a crash. See `admit`.

The reading is C29 over the control at 9 threads (three teams of three, the shipped
pipelined plan): LEAD, STOP or INCONCLUSIVE by the thresholds below. The 3-thread cell
(one team of three) gets the same rule's reading, reported as secondary, never judged.
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

CTL = "mibz2-pipe3-both"
C29 = "mibz2-pipe3-both-c29"
# ab's --arms order, which its rotation indexes
REGISTERED_ARMS = [CTL, C29]

# The registration (DRAFT, NOT FROZEN)
ROUNDS = 20
THREADS = 9  # judged: three teams of three
REPORTED_THREADS = 3  # one team of three: the same rule's reading, secondary, never judged
CORPUS = "wikidata"
CORPUS_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
IDENTITY_SCHEMA = "c29_checkpoints/identity/1"
FEATURES = []
PROFILE = "release"
# (leaders, plain helpers, pipelined helpers) each run must be seen running
PLANS = {THREADS: (3, 0, 6), REPORTED_THREADS: (1, 0, 2)}
POSITIVE_WALL = 0.98  # LEAD: the geometric mean at most this, its interval's high end below 1
CPU_CAP = 1.02  # and the CPU ratio's high end at most this
NEGATIVE_WALL = 0.99  # STOP: the wall ratio's low end above this (no 1% cut)
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


def record_problems(rec, where):
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
    want = PLANS.get(rec.get("threads"))
    if not isinstance(obs, str):
        bad.append(f"{where}: no observed_threads")
    elif want is not None and plan_of(obs) != want:
        bad.append(f"{where}: ran {plan_of(obs)} (leaders, plain, pipelined helpers), not {want}")
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
        if not (is_int(t) and t in PLANS):
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
        bad += record_problems(rec, where)
    for t in PLANS:
        for arm in REGISTERED_ARMS:
            got = set(runs[(t, arm)])
            if got != set(range(rounds)):
                missing = sorted(set(range(rounds)) - got)
                bad.append(f"{arm} t{t}: rounds missing {missing}")
        for rnd in range(rounds):
            want = [REGISTERED_ARMS[i] for i in arm_order(rnd, len(REGISTERED_ARMS))]
            if order[(t, rnd)] and order[(t, rnd)] != want:
                bad.append(f"t{t} r{rnd}: the arms ran {order[(t, rnd)]}, not ab's rotation {want}")
    # one output: every run's hash and length the control's, and the control's the same in
    # every round and at both thread counts
    outs = {(r.get("hash"), r.get("out")) for t in PLANS for r in runs[(t, CTL)].values()}
    if len(outs) > 1:
        bad.append(f"{CTL}: its outputs differ across rounds or threads: {sorted(map(str, outs))}")
    for t in PLANS:
        for rnd, r in runs[(t, C29)].items():
            ref_r = runs[(t, CTL)].get(rnd)
            if ref_r is not None and (r.get("hash"), r.get("out")) != (
                ref_r.get("hash"),
                ref_r.get("out"),
            ):
                bad.append(f"{C29} t{t} r{rnd}: output differs from the control's")
    return bad, runs


def paired(runs, t, num, den, key, rounds):
    return [runs[(t, num)][i][key] / runs[(t, den)][i][key] for i in range(rounds)]


def reading_of(w, c):
    if w[0] <= POSITIVE_WALL and w[2] < 1 and c[2] <= CPU_CAP:
        return "LEAD"
    if w[1] > NEGATIVE_WALL:
        return "STOP"
    return "INCONCLUSIVE"


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
            w = geo_ci(paired(runs, t, C29, CTL, "wall", rounds))
            c = geo_ci(paired(runs, t, C29, CTL, "cpu", rounds))
            if w is None or c is None:
                bad.append(f"{C29} over {CTL} at {t}: an absent or non-finite interval")
            cells[t] = {"wall": w, "cpu": c}
    if bad:
        out["invalid"] = bad
        print("INVALID")
        for b in bad:
            print(f"  {b}")
        if args.out:
            with open(args.out + ".json", "w") as f:
                json.dump(out, f, indent=1)
        return 2

    print(f"VALID ({label}): {rounds} rounds, {C29} against {CTL}, paired by round")
    print(f"{'threads':<12} {'wall ratio [95%]':<26} {'CPU ratio [95%]':<26} reading")
    readings = {}
    for t in (THREADS, REPORTED_THREADS):
        x = cells[t]
        readings[t] = reading_of(x["wall"], x["cpu"])
        tag = "judged" if t == THREADS else "secondary, never judged"
        print(f"{t:<12} {fmt(x['wall']):<26} {fmt(x['cpu']):<26} {readings[t]} ({tag})")
    print(
        f"\nReading ({label}, {THREADS} threads): {readings[THREADS]}. LEAD: wall <= "
        f"{POSITIVE_WALL} with its upper end < 1 and CPU's upper end <= {CPU_CAP}; STOP: wall's "
        f"lower end > {NEGATIVE_WALL}; else INCONCLUSIVE."
    )
    print(
        "  (environment-specific: a CI reading is this runner's; a diagnostic, not a production\n"
        "  gate: a LEAD supports a default-on proposal, which needs its own gate)"
    )
    out.update({"cells": {str(t): v for t, v in cells.items()}, "readings": readings})
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
