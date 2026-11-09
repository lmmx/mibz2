"""The service core at two teams on the workstation
(docs/journal/2026-11-08-service-2x3-registration.md, registered before any run): the report
and its verdicts, each predicate checked here.

    python3 bench/service_2x3.py report RUNS.jsonl [--occupancy W/occupancy-*.json] [--out STEM]

A run is VALID only as the registered one: the registered 20 rounds (whatever the recipe was
asked for), from the attempt's directory (RUNS.jsonl's) a clean tree (`git-status.txt`
empty), no `NOT-FROZEN.txt` or `DRY-RUN.txt`, `commit.txt` the checkout's head, the ten-core
topology, every run the same bytes, every registered cell's rounds once each with its
registered plan, no other cell, the noise control N1 within 1 +/- 0.03, and the registered
trace complete.

The budget is the decode threads (the teams' threads plus the service threads), and every
cell runs at a budget of 10, in one `ab run`, interleaved by round. The registered cells
(ARM@BUDGET: team leaders, helpers, service threads):

    mibz2@10            3, 6, 0   today's default (0.2.6): 3 x 3 pipelined, park and small
    mibz2-svc2@10       2, 4, 2   the service core: 2 x 3 + 2
    mibz2-team2x3@10    2, 4, 0   2 x 3, plain (Plan::Teams { teams: 2, k: 3 })
    mibz2-pipe2x3@10    2, 4, 0   2 x 3 pipelined, park and small (the default's team)
    mibz2-team3@10      3, 6, 0   3 x 3, plain
    mibz2-team3-twin@10 3, 6, 0   mibz2-team3 under another name: the noise control

The predicates shared with the service core's registration (the outcome map, the plan count
from ab's per-thread CPU, the extra-thread label, the traces' failure modes and their
thresholds) are bench/service_core.py's, imported unchanged.

Revised for the Researcher's HOLD 6729358962 (the registration's revision section): VALID also
requires usable output evidence (every run's hash a hex digest, equal across every run, and
its output the registered sample's decoded size), finite positive wall, CPU and peak RSS, the
registered workload (every run's corpus `wikidata`, and `corpus.sha256` the registered
corpus's digest from bench/wikidata.json), and the registered trace: its configuration (10
threads, the service core's svc2 plan, 2 service threads, its commit the run's), its parity
and output size, and every metric the report reads finite and in range, so that saturation is
never UNRESOLVED in a VALID run. A malformed record makes the run INVALID with a reason, never
an exception. The readings' wording is narrowed (P1, C1, P2 and S1 are separate findings;
NULL is bounded uncertainty; CPU INCONCLUSIVE is no detected cost; saturation's capacity
reading is a hypothesis); no judged comparison, threshold or rule changes.
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
    extra_thread_label,
    failure_modes,
    geo_ci,
    is_gain,
    outcome,
    threads_of,
)

REGISTERED_ROUNDS = 20
# cell -> (leaders, helpers, service threads)
CELLS = {
    "mibz2@10": (3, 6, 0),
    "mibz2-svc2@10": (2, 4, 2),
    "mibz2-team2x3@10": (2, 4, 0),
    "mibz2-pipe2x3@10": (2, 4, 0),
    "mibz2-team3@10": (3, 6, 0),
    "mibz2-team3-twin@10": (3, 6, 0),
}
# the registered comparisons (name, arm, reference): judged, speed and CPU co-primary
JUDGED = [
    # (a) the product question: the service core at 2 x 3 + 2 against today's default
    ("P1", "mibz2-svc2@10", "mibz2@10"),
    # (b) the causal question: the same two teams of three, with and without the service
    ("C1", "mibz2-svc2@10", "mibz2-team2x3@10"),
    # (c) two teams under today's decoder (the pipelined team, park and small)
    ("P2", "mibz2-pipe2x3@10", "mibz2@10"),
]
# reported, never judged
SECONDARY = [
    ("S1", "mibz2-svc2@10", "mibz2-pipe2x3@10"),
    ("S2", "mibz2-team2x3@10", "mibz2@10"),
    ("S3", "mibz2-pipe2x3@10", "mibz2-team2x3@10"),
    ("S4", "mibz2-team2x3@10", "mibz2-team3@10"),
    ("S5", "mibz2-svc2@10", "mibz2-team3@10"),
    ("R1", "mibz2@10", "mibz2-team3@10"),
]
# the noise control, as the service core's registration has it, unchanged
CONTROL = ("N1", "mibz2-team3-twin@10", "mibz2-team3@10")
# the registered trace (the occupancy pass's name), which must be complete
TRACES = ["occupancy-t10-svc2"]
TRACE = "occupancy-t10-svc2"

# the registered workload (bench/wikidata.json, a measured input): every run's `corpus` is
# ab's name for the attempt's `wikidata.bz2` (its stem), and the attempt's `corpus.sha256`
# must be the registered corpus's digest, so no other corpus (CORPUS=) can pass as it
HERE = os.path.dirname(os.path.abspath(__file__))
REGISTERED = json.load(open(os.path.join(HERE, "wikidata.json")))
REGISTERED_CORPUS = "wikidata"
REGISTERED_CORPUS_SHA256 = REGISTERED["corpus_sha256"]
REGISTERED_OUT = REGISTERED["decoded_bytes"]
REFERENCE = "mibz2@10"
# ab's output digest: a hex string (16 digits as ab writes it), never empty
HASH = re.compile(r"[0-9a-f]{16,}")
# the registered trace's configuration, as examples/occupancy.rs records it
TRACE_THREADS = 10
TRACE_SERVICES = 2
TRACE_PLAN = "the service core, mibz2-svc2's plan"
# every trace metric the report reads, each finite: shares and utilisations in [0, 1],
# waits, backlogs and times at least 0
TRACE_SHARES = ["svc.util.max", "svc.util.0", "svc.util.1", "svc.team.idle_prep_share", "svc.team.idle_back_share"]
TRACE_NONNEG = [
    "svc.queue.prep.median_ms", "svc.queue.prep.p90_ms", "svc.queue.finish.median_ms", "svc.queue.finish.p90_ms",
    "svc.queue.job.median_ms", "svc.backlog.prep_mean", "svc.backlog.finish_mean", "svc.backlog.prep_max",
    "svc.backlog.finish_max", "svc.team.ms.run_wait", "blocks",
]


def number(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def positive(x):
    return number(x) and x > 0


def row_problems(r):
    """What makes a run's record unusable as evidence (empty if it is usable)"""
    p = []
    if not isinstance(r.get("arm"), str) or not r.get("arm"):
        p.append("no arm")
    for k in ("threads", "round"):
        if not (number(r.get(k)) and float(r[k]).is_integer() and r[k] >= 0):
            p.append(f"`{k}` {r.get(k)!r} not a whole number")
    if r.get("err"):
        return p
    for k in ("wall", "cpu", "rss_kb"):
        if not positive(r.get(k)):
            p.append(f"`{k}` {r.get(k)!r} not finite and positive")
    h = r.get("hash")
    if not (isinstance(h, str) and HASH.fullmatch(h)):
        p.append(f"`hash` {h!r} not a hex digest")
    if not (number(r.get("out")) and r["out"] == REGISTERED_OUT):
        p.append(f"`out` {r.get('out')!r} not the registered {REGISTERED_OUT} bytes")
    if r.get("corpus") != REGISTERED_CORPUS:
        p.append(f"`corpus` {r.get('corpus')!r} not the registered `{REGISTERED_CORPUS}`")
    if not isinstance(r.get("thread_cpu"), str):
        p.append("no `thread_cpu`")
    return p


def load_trace(paths, commit):
    """The registered trace's timeline and its problems: its configuration, parity, output
    and every metric the report reads (an empty list: the trace is usable)"""
    found = [x for x in paths if os.path.basename(x).removesuffix(".json") == TRACE]
    if len(found) != 1:
        return {}, [f"{len(found)} files named `{TRACE}.json` (one registered)"]
    try:
        o = json.load(open(found[0]))
    except (OSError, ValueError) as e:
        return {}, [f"unreadable: {e}"]
    if not isinstance(o, dict):
        return {}, ["not a trace report"]
    p = []
    res = o.get("results")
    if not (isinstance(res, list) and len(res) == 1 and isinstance(res[0], dict)):
        return {}, ["not exactly one result (one thread count registered)"]
    r = res[0]
    tl = r.get("timeline") if isinstance(r.get("timeline"), dict) else {}
    st = o.get("settings") if isinstance(o.get("settings"), dict) else {}
    meta = o.get("meta") if isinstance(o.get("meta"), dict) else {}
    if r.get("threads") != TRACE_THREADS or st.get("threads") != [TRACE_THREADS]:
        p.append(f"threads {r.get('threads')!r} / {st.get('threads')!r}, not {TRACE_THREADS}")
    if st.get("teams") != TRACE_PLAN:
        p.append(f"plan {st.get('teams')!r}, not {TRACE_PLAN!r}")
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
    bad += [k for k in TRACE_NONNEG if not (number(tl.get(k)) and tl[k] >= 0)]
    if bad:
        p.append("metrics absent or out of range: " + ", ".join(bad))
    return {k: v for k, v in tl.items() if number(v)}, p


def decode_threads(plan):
    leaders, helpers, services = plan
    return leaders + helpers + services


def cell(r):
    return f"{r['arm']}@{r['threads']}"


def cpu_phrase(co):
    """A CPU outcome as what it shows: NULL is bounded, INCONCLUSIVE detects no cost"""
    if co == "NULL":
        return "CPU NULL (within ±3%: a small difference is not excluded)"
    if co.startswith("INCONCLUSIVE"):
        return "no CPU cost detected (CPU INCONCLUSIVE: not shown to cost nothing)"
    return co


def speed_phrase(word):
    if word == "NULL":
        return "NULL: within ±3%, a small effect is not excluded"
    return word


def readings(by):
    """The registration's readings, mechanically, from P1, C1 and P2 (the judged family and
    the registered order, unchanged), with S1 reported beside them, never judged. P1, C1, P2
    and S1 are separate findings: C1 is the service bundle against the plain 2 x 3 (two
    service threads, the serial work they take, the feeding, the hub's synchronisation and
    hand-offs, together), not the cause of P1; and no reading prefers svc2@10 where S1 does
    not show it faster than the simpler pipe2x3@10."""
    p1, c1, p2 = by["P1"], by["C1"], by["P2"]
    s1 = by.get("S1")
    s = p1["speed_outcome"]
    cpu = cpu_phrase(p1["cpu_outcome"])
    if is_gain(s):
        product = (
            "A TRADE (P1 a speed gain for a CPU COST)" if p1["cpu_outcome"] == "CPU COST"
            else f"SVC2@10 BEATS THE DEFAULT (P1 {s}; {cpu})"
        )
    elif s == "NULL":
        product = f"NOT REPLICATED (P1 {speed_phrase(s)})"
    elif s == "REGRESSION":
        product = "NOT REPLICATED (P1 REGRESSION: svc2@10 slower than the default)"
    else:
        product = "INCONCLUSIVE"
    c = c1["speed_outcome"]
    if c == "GAIN":
        cause = "THE SERVICE BUNDLE BEATS THE PLAIN 2 × 3 (C1 GAIN, at the same teams and window; no one mechanism isolated)"
    elif is_gain(c):
        cause = "A SMALL GAIN OF THE SERVICE BUNDLE OVER THE PLAIN 2 × 3 (C1 SMALL GAIN)"
    elif c == "NULL":
        cause = f"NO SERVICE-BUNDLE GAIN DETECTED OVER THE PLAIN 2 × 3 (C1 {speed_phrase(c)})"
    elif c == "REGRESSION":
        cause = "THE SERVICE BUNDLE IS SLOWER THAN THE PLAIN 2 × 3 (C1 REGRESSION)"
    else:
        cause = "INCONCLUSIVE"
    q = p2["speed_outcome"]
    if is_gain(q):
        two = f"TWO PIPELINED TEAMS BEAT THE DEFAULT (P2 {q})"
    elif q == "NULL":
        two = f"TWO PIPELINED TEAMS NOT SHOWN TO BEAT THE DEFAULT (P2 {speed_phrase(q)})"
    elif q == "REGRESSION":
        two = "TWO PIPELINED TEAMS DON'T BEAT THE DEFAULT (P2 REGRESSION)"
    else:
        two = "INCONCLUSIVE"
    # S1, reported (never judged): the two candidate products head to head
    if not s1 or not number(s1.get("speed", [math.nan])[0]):
        versus, svc2_ahead = "S1 NOT AVAILABLE: svc2@10 against pipe2x3@10 not shown", False
    else:
        g, lo, hi = s1["speed"]
        so = outcome(g, lo, hi)
        fig = f"S1 {g:.3f} [{lo:.3f}, {hi:.3f}], {so}; reported, not judged"
        svc2_ahead = is_gain(so)
        if svc2_ahead:
            versus = f"SVC2@10 FASTER THAN PIPE2X3@10 ({fig})"
        elif g < 1:
            versus = f"PIPE2X3@10 FASTER THAN SVC2@10 on the geometric mean ({fig})"
        else:
            versus = f"SVC2@10 NOT SHOWN FASTER THAN PIPE2X3@10 ({fig})"
    if s in ("NULL", "REGRESSION"):
        overall = f"NOT REPLICATED (registered step 1: P1 {speed_phrase(s)}; svc2@10 not shown to beat today's default)"
    elif not is_gain(s):
        overall = "NONE OF THE FOUR (registered step 2: P1 inconclusive)"
    elif c in ("NULL", "REGRESSION"):
        overall = (
            f"P1 A GAIN WITHOUT A DETECTED SERVICE-BUNDLE GAIN (registered step 3: C1 {speed_phrase(c)}). "
            "The plan of two teams may account for svc2@10's advantage; that is not shown by C1 alone, "
            f"and the contributors' roles are unresolved. P2: {two}. S1: {versus}"
        )
    elif not is_gain(c):
        overall = "NONE OF THE FOUR (registered step 4: C1 inconclusive)"
    elif p1["cpu_outcome"] == "CPU COST":
        overall = f"A TRADE (registered step 5: P1 a gain for a CPU COST, C1 a gain: for Louis to weigh). S1: {versus}"
    elif svc2_ahead:
        overall = (
            f"FOR SVC2@10 (registered step 6: P1 {s}, {cpu}, C1 {c}; "
            f"and {versus}). C1 is the service bundle's gain over the plain 2 × 3, not a single cause"
        )
    else:
        overall = (
            f"REGISTERED STEP 6 MET, BUT SVC2@10 NOT SHOWN BETTER THAN THE SIMPLER PIPE2X3@10 "
            f"(P1 {s}, {cpu}, C1 {c}; {versus}). Against the default and the plain 2 × 3 the "
            "readings favour svc2@10; against two pipelined teams they don't: for Louis to weigh"
        )
    return {"product": product, "cause": cause, "two_teams": two, "versus_pipe2": versus, "overall": overall}


DECOMPOSED = ("mibz2@10", "mibz2-svc2@10", "mibz2-team2x3@10", "mibz2-pipe2x3@10")


def decomposition(by, key="wall"):
    """On the rounds every one of the four cells ran (the common paired rounds), the
    geometric means of P1, S1, P2, C1 and S2, and the identities P1 = S1 x P2 = C1 x S2 they
    obey exactly (round by round, then on the log scale). A decomposition of comparisons, not
    of causes; and the intervals' endpoints don't multiply (the shared rounds correlate them)."""
    common = sorted(set.intersection(*(set(by.get(c, {})) for c in DECOMPOSED))) if by else []
    if not common:
        return {"rounds": 0}
    d, s, t, p = DECOMPOSED

    def gm(arm, ref):
        return math.exp(statistics.fmean(math.log(by[ref][i][key] / by[arm][i][key]) for i in common))

    g = {"P1": gm(s, d), "S1": gm(s, p), "P2": gm(p, d), "C1": gm(s, t), "S2": gm(t, d)}
    g["S1xP2"] = g["S1"] * g["P2"]
    g["C1xS2"] = g["C1"] * g["S2"]
    g["rounds"] = len(common)
    return g


def counterexample(fm, c1):
    """The registered counterexample, service saturation at 2 x 3 + 2, read with C1: the
    numeric flag as registered; what it would mean for capacity is a hypothesis"""
    if fm["saturated"] == "UNRESOLVED":
        return "UNRESOLVED (svc2@10's trace missing or incomplete)"
    if fm["saturated"] == "SATURATED":
        if c1["speed_outcome"] in ("NULL", "REGRESSION"):
            return (
                "SATURATION BINDS (the numeric flag: svc2@10 SATURATED, and C1 NULL or REGRESSION). "
                "That the service's capacity is what binds is a hypothesis these figures are consistent "
                "with, not one they identify: load imbalance between the service threads, or queue and "
                "scheduling effects, remain possible"
            )
        if is_gain(c1["speed_outcome"]):
            return (
                "SATURATED, A GAIN NONETHELESS (svc2@10 SATURATED, and C1 a gain; "
                "that more service capacity would add more is a hypothesis)"
            )
        return "SATURATED, C1 INCONCLUSIVE"
    if fm["prep"] == "PREP STARVATION":
        return (
            "NOT SATURATED at 2 × 3 + 2 (PREP STARVATION without saturation: consistent with the "
            "prepared queue's lookahead binding, not shown to be it)"
        )
    return "NOT SATURATED at 2 × 3 + 2"


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
    lines = ["# The service core at two teams: the workstation run's report", ""]
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
        seen = ", ".join(f"{p[0]} leaders, {p[1]} helpers, {p[2]} service" for p in sorted(plans)) or "none"
        checks.append(
            (
                f"`{c}`: rounds 0-{REGISTERED_ROUNDS - 1} once each, every run {plan[0]} leaders, "
                f"{plan[1]} helpers, {plan[2]} service threads (seen: {seen})",
                ok,
            )
        )
    unexpected = sorted(set(by) - set(CELLS))
    checks.append((f"no cell beyond the registered ones (seen: {', '.join(unexpected) or 'none'})", not unexpected))
    tl, trace_problems = load_trace(a.occupancy or [], commit)
    unreadable = [f"{TRACE}: {x}" for x in trace_problems if x.startswith("unreadable")]
    checks.append(
        (
            f"the trace `{TRACE}` complete and as registered ({TRACE_THREADS} threads, {TRACE_PLAN}, "
            f"{TRACE_SERVICES} service threads, the run's commit, parity, the registered output, every metric "
            f"read finite and in range; rounds: {tl.get('complete_rounds', 0)}"
            f"{'; ' + '; '.join(trace_problems) if trace_problems else ''})",
            not trace_problems,
        )
    )
    fm = failure_modes(tl)
    checks.append((f"saturation resolved from the trace (seen: {fm['saturated']})", fm["saturated"] != "UNRESOLVED"))
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
        "## The registered comparisons (equal budget of 10; judged; speed and CPU co-primary)",
        "",
        "| | arm | against | decode threads | speed [95% CI] | speed outcome | CPU [95% CI] | CPU outcome | the extra threads |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for name, arm, ref in JUDGED:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        ta, tr = decode_threads(CELLS[arm]), decode_threads(CELLS[ref])
        if verdict == "VALID":
            so = outcome(*speed)
            co = outcome(*cpu, gain="CPU SAVING", loss="CPU COST")
            xl = extra_thread_label(speed, ta, tr) if is_gain(so) else "(no gain)"
        else:
            so = co = xl = "NONE (the run is invalid)"
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {ta} against {tr} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | {so} | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] | {co} | {xl} |"
        )
        out["judged"].append(
            {"name": name, "arm": arm, "ref": ref, "threads": [ta, tr], "speed": list(speed), "cpu": list(cpu),
             "rounds": n, "speed_outcome": so, "cpu_outcome": co, "extra_thread": xl}
        )
    lines.append("")
    jd = {c["name"]: c for c in out["judged"]}
    s1 = dict(zip(("name", "arm", "ref"), SECONDARY[0]), speed=list(ratio(SECONDARY[0][1], SECONDARY[0][2])[0]))
    if verdict == "VALID":
        out["readings"] = readings({**jd, "S1": s1})
    else:
        out["readings"] = {k: "NONE (the run is invalid)" for k in ("product", "cause", "two_teams", "versus_pipe2", "overall")}
    rd = out["readings"]
    dw, dc = decomposition(by), decomposition(by, "cpu")
    out["decomposition"] = {"speed": dw, "cpu": dc}
    lines += [
        "**The findings** (P1, C1 and P2 judged as registered; S1 reported, never judged; each its own finding):",
        "",
        f"- the product (P1, svc2@10 against the default): **{rd['product']}**",
        f"- the service bundle (C1, svc2@10 against the plain 2 × 3 at the same teams and window): **{rd['cause']}** "
        "(C1's extra decode threads are the service threads themselves: its label is reported, not required. C1 "
        "measures the bundle, the two service threads with the serial work they take, the feeding, the hub's "
        "synchronisation and hand-offs, not any one of them)",
        f"- two teams under today's decoder (P2): **{rd['two_teams']}**",
        f"- the two candidates head to head (S1, svc2@10 against pipe2x3@10): **{rd['versus_pipe2']}**",
        f"- overall, by the registered order: **{rd['overall']}**",
        "",
        "NULL is an interval inside [0.97, 1.03]: bounded uncertainty, not an effect shown to be zero. "
        "CPU INCONCLUSIVE is no detected CPU cost, not a CPU cost shown to be absent.",
        "",
    ]
    if dw.get("rounds"):
        lines += [
            f"**The decomposition** (geometric means over the {dw['rounds']} common paired rounds; "
            "P1 = S1 × P2 = C1 × S2 holds round by round, so on these means exactly; the intervals' "
            "endpoints don't multiply, since the shared rounds correlate them):",
            "",
            "| | P1 | S1 | P2 | S1 × P2 | C1 | S2 | C1 × S2 |",
            "|---|---|---|---|---|---|---|---|",
        ]
        for nm, d in (("speed", dw), ("CPU", dc)):
            lines.append(
                f"| {nm} | {d['P1']:.4f} | {d['S1']:.4f} | {d['P2']:.4f} | {d['S1xP2']:.4f} | "
                f"{d['C1']:.4f} | {d['S2']:.4f} | {d['C1xS2']:.4f} |"
            )
        lines.append("")
    lines += [
        "## Reported, never judged",
        "",
        "| | arm | against | decode threads | speed [95% CI] | CPU [95% CI] |",
        "|---|---|---|---|---|---|",
    ]
    for name, arm, ref in SECONDARY:
        speed, n = ratio(arm, ref)
        cpu, _ = ratio(arm, ref, "cpu")
        ta, tr = decode_threads(CELLS[arm]), decode_threads(CELLS[ref])
        lines.append(
            f"| {name} | `{arm}` | `{ref}` | {ta} against {tr} | {speed[0]:.3f} [{speed[1]:.3f}, {speed[2]:.3f}] | "
            f"{cpu[0]:.3f} [{cpu[1]:.3f}, {cpu[2]:.3f}] |"
        )
        out["secondary"].append({"name": name, "arm": arm, "ref": ref, "speed": list(speed), "cpu": list(cpu)})
    tl = {k: v for k, v in tl.items() if number(v)}
    utils = sorted((k, v) for k, v in tl.items() if re.fullmatch(r"svc\.util\.\d+", k))
    u = ", ".join(f"{v:.2f}" for _, v in utils)
    u = f"{u}; max {tl.get('svc.util.max', float('nan')):.2f}" if utils else "–"
    lines += [
        "",
        "(Speed: the reference's wall over the arm's, > 1 the arm is faster. CPU: the reference's CPU over the arm's, > 1 the arm spends less.)",
        "",
        "## Every cell (medians)",
        "",
        "| cell | decode threads (leaders + helpers + service) | wall s | CPU s | peak RSS MB | rounds | service utilisation (each; max) |",
        "|---|---|---|---|---|---|---|",
    ]
    for c in sorted(by):
        rs = list(by[c].values())
        if not rs:
            continue
        plan = CELLS.get(c)
        lines.append(
            f"| `{c}`{'' if plan else ' (unregistered)'} | {'+'.join(map(str, plan)) if plan else '?'} | "
            f"{statistics.median(r['wall'] for r in rs):.3f} | {statistics.median(r['cpu'] for r in rs):.3f} | "
            f"{statistics.median(r['rss_kb'] for r in rs) / 1024:.0f} | {len(rs)} | {u if c == 'mibz2-svc2@10' else '–'} |"
        )
    lines += [
        "",
        "## The trace (occupancy, medians over complete rounds)",
        "",
        "| trace | rounds | blocks | util max | prep wait median / p90 ms | finish wait median / p90 ms | job wait median ms | backlog prep / finish (mean; max) | teams idle: prep / back share | run wait ms | saturation | prep starvation | finish backlog |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    show = verdict == "VALID"
    if tl:
        g = lambda k: tl.get(k, float("nan"))  # noqa: E731
        out["traces"][TRACE] = {"timeline": {k: v for k, v in tl.items() if k.startswith("svc.") or k in ("blocks", "complete_rounds")}, "modes": fm}
        lines.append(
            f"| {TRACE} | {g('complete_rounds')} | {g('blocks')} | {g('svc.util.max'):.2f} | "
            f"{g('svc.queue.prep.median_ms'):.3f} / {g('svc.queue.prep.p90_ms'):.3f} | "
            f"{g('svc.queue.finish.median_ms'):.3f} / {g('svc.queue.finish.p90_ms'):.3f} | {g('svc.queue.job.median_ms'):.3f} | "
            f"{g('svc.backlog.prep_mean'):.2f} / {g('svc.backlog.finish_mean'):.2f} ({g('svc.backlog.prep_max'):.0f}; {g('svc.backlog.finish_max'):.0f}) | "
            f"{g('svc.team.idle_prep_share'):.3f} / {g('svc.team.idle_back_share'):.3f} | {g('svc.team.ms.run_wait'):.1f} | "
            f"{fm['saturated'] if show else 'NONE'} | {fm['prep'] if show else 'NONE'} | {fm['finish'] if show else 'NONE'} |"
        )
    lines += [f"| {x} | | | | | | | | | | | | |" for x in unreadable]
    out["saturation"] = counterexample(fm, jd["C1"]) if show else "NONE (the run is invalid)"
    out["saturation_flag"] = fm["saturated"] if show else "NONE (the run is invalid)"
    lines += ["", f"**The registered counterexample, service saturation at 2 × 3 + 2:** **{out['saturation']}**"]
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
