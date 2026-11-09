"""C29's exposed-path diagnostic, its follow-up registration
(docs/journal/2026-11-09-c29-follow-up-registration.md, DRAFT, NOT FROZEN; the first,
docs/journal/2026-11-09-c29-register-checkpoints.md, is closed VALID, UNJUDGED (FALLBACK)): its
admission, its paired ratios and the reading the registration sets. Shaped on #270's reporter (research/run-fed-leader's
bench/run_fed_leader.py): the same identity, the same admission, the same reading rule.

    python3 bench/c29_checkpoints.py report RUNS.jsonl --head SHA --identity ID.json [--out STEM]
    python3 bench/c29_checkpoints.py report RUNS.jsonl --unjudged [--rounds N] [--identity ID.json] [--out STEM]
    python3 bench/c29_checkpoints.py preflight --env ENV.json --env-after ENV2.json --nproc N.txt [--out STEM]

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
and no reading is given. A judged reading further needs the registered environment shown,
untimed, in the identity and in every record (`environment_problems`): AVX2 detected (as
`Kernels::detect` reports it, the bool C29's and the control's kernels dispatch on: without
it both are the same scalar kernel) and the registered CPU allocation (REGISTERED_CPUS, by
`nproc`, the affinity mask and `available_parallelism`, unpinned; a cgroup CPU quota known
from complete coverage, bench/c29_identity.py's `coverage`: unlimited at every level to the
hierarchy's real root, or a tightest quota Q µs a period P µs with Q >= 4P, integers, no
rounding; and that configuration unchanged after the run, `cgroup_stable`), and `ab run`
itself exited 0. Incomplete coverage is unknown, never a pass, whatever the readable levels
say.
The preflight is the environment's half alone, untimed and with no decoder run: whether a
run in this environment could be judged (exit 0) or not (exit 3).
A fact absent or other than registered refuses the judged reading: the run is kept as
UNJUDGED (FALLBACK), its ratios printed under that label, exit 3, never re-run to pass.
Every measurement is checked for presence and type before any arithmetic, so an absent or
malformed one is a reason, never a crash. See `admit`.

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

# The registration (the follow-up's, DRAFT, NOT FROZEN; unchanged from the first's)
ROUNDS = 20
THREADS = 9  # judged: three teams of three
REPORTED_THREADS = 3  # one team of three: the same rule's reading, secondary, never judged
CORPUS = "wikidata"
CORPUS_SHA256 = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
IDENTITY_SCHEMA = "c29_checkpoints/identity/4"
FEATURES = []
PROFILE = "release"
# (leaders, plain helpers, pipelined helpers) each run must be seen running
PLANS = {THREADS: (3, 0, 6), REPORTED_THREADS: (1, 0, 2)}
POSITIVE_WALL = 0.98  # LEAD: the geometric mean at most this, its interval's high end below 1
CPU_CAP = 1.02  # and the CPU ratio's high end at most this
NEGATIVE_WALL = 0.99  # STOP: the wall ratio's low end above this (no 1% cut)
# The registered environment: the intended (AVX2) kernels, and CI's 4 vCPUs, which 9 threads
# oversubscribe (the registration's justification). Not timed: read around the run
REGISTERED_AVX2 = True
REGISTERED_CPUS = 4
ENV_SHOWN = 16  # the environment's reasons printed (all of them in --out's JSON)
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


def quota_problems(key, q):
    """The quota's reasons: none only for complete coverage, unlimited, or finite with
    Q >= REGISTERED_CPUS * P exactly"""
    qd = q if isinstance(q, dict) else {}
    state, cov = qd.get("state"), qd.get("coverage")
    if cov != "complete":
        why = "; ".join(str(r) for r in (qd.get("reasons") or [])[:3])
        return [f"identity: {key} {state!r}, coverage {cov!r}: not known ({why})"]
    if state == "unlimited":
        return []
    qq, pp = qd.get("quota"), qd.get("period")
    if state == "finite" and is_int(qq) and is_int(pp) and qq > 0 and pp > 0:
        if qq < REGISTERED_CPUS * pp:
            return [
                f"identity: {key} {qq}/{pp} at {qd.get('path')!r}: below {REGISTERED_CPUS} CPUs "
                f"({qq} < {REGISTERED_CPUS} x {pp})"
            ]
        return []
    return [f"identity: {key} {q!r}: not known unlimited or finite"]


def environment_problems(ident, recs, preflight=False):
    """The registered environment's facts, absent or not as registered: a reason each. Any
    reason refuses a judged reading (UNJUDGED (FALLBACK)), never an INVALID's. The preflight
    has no run, so no run status"""
    bad = []
    if not isinstance(ident, dict):
        ident = {}
    if ident.get("avx2") is not REGISTERED_AVX2:
        bad.append(f"identity: avx2 {ident.get('avx2')!r}, not {REGISTERED_AVX2}")
    for k in ("nproc", "cpus_allowed", "available_parallelism"):
        v = ident.get(k)
        if not (is_int(v) and v == REGISTERED_CPUS):
            bad.append(f"identity: {k} {v!r}, not {REGISTERED_CPUS}")
    # the quota from the census's coverage (bench/c29_identity.py's `coverage`), before and
    # after the run, and its configuration unchanged between them
    for k in ("cpu_quota", "cpu_quota_after"):
        bad += quota_problems(k, ident.get(k))
    st = ident.get("cgroup_stable")
    if not (isinstance(st, dict) and st.get("stable") is True):
        bad.append(f"identity: cgroup_stable {st!r}: the configuration not shown unchanged")
    # a failed `ab run` (its parent's status) is no judged attempt, its records complete or not
    if not preflight and not (is_int(ident.get("ab_run_status")) and ident["ab_run_status"] == 0):
        bad.append(f"identity: ab_run_status {ident.get('ab_run_status')!r}, not 0")
    for rec in recs:
        where = (
            f"line {rec.get('_line')} ({rec.get('arm')} t{rec.get('threads')} r{rec.get('round')})"
        )
        if rec.get("avx2") is not REGISTERED_AVX2:
            bad.append(f"{where}: avx2 {rec.get('avx2')!r}, not {REGISTERED_AVX2}")
        for k in ("cpus_allowed", "available_parallelism"):
            v = rec.get(k)
            if not (is_int(v) and v == REGISTERED_CPUS):
                bad.append(f"{where}: {k} {v!r}, not {REGISTERED_CPUS}")
        if rec.get("cpus") != "all":
            bad.append(f"{where}: cpus {rec.get('cpus')!r}, not 'all' (unpinned)")
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
    env = environment_problems(ident, recs)
    fallback = judged and bool(env)
    label = "JUDGED" if judged and not env else "UNJUDGED (FALLBACK)" if fallback else "UNJUDGED"
    raw = [{k: v for k, v in r.items() if k != "_line"} for r in recs]
    out = {
        "label": label,
        "head": args.head,
        "identity": ident,
        "threads": THREADS,
        "rounds": rounds,
        "invalid": bad,
        "environment": env,  # the registered environment's facts absent or not as registered
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

    if env:
        if fallback:
            print("NOT JUDGED: the registered environment is not shown (no judged reading; kept")
            print("  as UNJUDGED (FALLBACK), never re-run to pass)")
        else:
            print("(environment, not the registered one; this run is unjudged anyway)")
        # (the identity's reasons and the first records'; every one is in --out's JSON)
        for b in env[:ENV_SHOWN]:
            print(f"  {b}")
        if len(env) > ENV_SHOWN:
            print(f"  ... and {len(env) - ENV_SHOWN} more")
    print(f"VALID ({label}): {rounds} rounds, {C29} against {CTL}, paired by round")
    print(f"{'threads':<12} {'wall ratio [95%]':<26} {'CPU ratio [95%]':<26} reading")
    readings = {}
    for t in (THREADS, REPORTED_THREADS):
        x = cells[t]
        readings[t] = reading_of(x["wall"], x["cpu"])
        tag = (
            ("judged" if label == "JUDGED" else "primary, unjudged")
            if t == THREADS
            else "secondary, never judged"
        )
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
    return 3 if fallback else 0


def preflight(args):
    """The environment alone, untimed, no decoder run: would a run here be judged?"""
    import c29_identity

    def read(p):
        try:
            with open(p) as f:
                return f.read()
        except (OSError, TypeError):
            return None

    facts = c29_identity.environment(read(args.env), read(args.env_after), read(args.nproc))
    bad = environment_problems(facts, [], preflight=True)
    q = facts["cpu_quota"]
    out = {"preflight": "PASS" if not bad else "FAIL", "environment": bad, "facts": facts}
    if args.out:
        with open(args.out + ".json", "w") as f:
            json.dump(out, f, indent=1)
    if bad:
        print("PREFLIGHT FAIL: a run in this environment would not be judged (no decoder run)")
        for b in bad:
            print(f"  {b}")
    else:
        print("PREFLIGHT PASS: this environment would be judged (no decoder run)")
    print(f"  quota: {json.dumps({k: v for k, v in q.items() if k != 'reasons'})}")
    for r in q.get("reasons") or []:
        print(f"    {r}")
    print(f"  stability: {json.dumps(facts['cgroup_stable'])}")
    return 0 if not bad else 3


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
    p = sub.add_parser("preflight")
    p.add_argument("--env", required=True, help="`ab env`'s line")
    p.add_argument("--env-after", required=True, help="`ab env`'s line, read again after")
    p.add_argument("--nproc", required=True, help="`nproc`'s output")
    p.add_argument("--out")
    args = ap.parse_args()
    if args.cmd == "preflight":
        try:
            code = preflight(args)
        except Exception as e:  # (a bug is never a PASS)
            print(f"PREFLIGHT FAIL\n  the preflight failed: {type(e).__name__}: {e}")
            code = 3
        sys.exit(code)
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
