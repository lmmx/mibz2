"""The guarded service default under the target-gated route (docs/STEWARDSHIP.md, "Target-gated
changes", conditions 1-7, enacted at 3e1a7ec3): the judge of its registration,
docs/journal/2026-11-10-service-guarded-target-gated-registration.md (DRAFT, not frozen; on
#316, the candidate's branch, with the workstation recipe `.just/service-tg.just` and the
preflight `examples/svc_target.rs`). This file and its CI workflow are measurement tooling on
master (the Guardian's D078); the candidate's decoder is not. Its parts, fixed with the freeze:

    python3 bench/service_tg.py report ATTEMPT_DIR --out ATTEMPT_DIR/service-tg
    python3 bench/service_tg.py manifest SHARDS_DIR [--artifacts A.jsonl --jobs J.jsonl --run ID --attempt N --label L]
    python3 bench/service_tg.py ni SHARDS_DIR --manifest MANIFEST.json --head SHA --out PREFIX
    python3 bench/service_tg.py replicated --dispatch ROOT1 MANIFEST1 ... --dispatch ROOT12 MANIFEST12 --head SHA --out PREFIX
    python3 bench/service_tg.py confirm REGISTERED.json --confirmation CONFIRMATION.json --out PREFIX

**`report`: the workstation A/B** (conditions 3, 4 and its share of 7), judged from the
attempt's directory as `just bench-service-tg` leaves it: the candidate's default decoder
(`BzReader::parallel`, ab's `mibz2` arm) against the base's (`mibz2@<attempt>/bin/ab-base`),
two arms, no twin, at 9, 10 and 11 threads, `ROUNDS` paired rounds, on the bench's Wikidata
sample. Admission is the earlier attempt's (bench/service_guarded.py, restored here for two
arms): identity (the frozen candidate, base, base fetch time and toolchain), workload (the
registered corpus and decoded bytes, one digest), the preflight (TARGET, checked whole by
bench/svc_target_check.py with its exit status), and structure (every row's arm, integer
threads and round, `termination` exactly "success", an empty `err`, finite positive wall,
CPU and RSS, every round once per arm and budget, every run's plan as expected).

- **Condition 3, on target** (N = 1 cell: the i9 × Wikidata × 10 threads; K = 2N = 2): wall
  speed, base over head, its two-sided interval at confidence 1 - 0.05/K (t at 0.9875), lower
  bound > 1.02; CPU time, head over base, its one-sided upper bound at confidence 1 - 0.05/K
  (t at 0.975), at most 1.02. Both must pass.
- **Condition 4, fallback no harm** (M = 2 contrasts: 9 and 11 threads, wall time): head over
  base, one-sided upper bound at confidence 1 - 0.05/M (t at 0.975), at most 1.02 at each.
  NOT SHOWN fails.
- **Condition 7's workstation criteria** (fixed before data): at 9 and 11, CPU time head over
  base is a demonstrated regression if its two-sided 95% lower bound exceeds 1.02; at every
  cell, peak RSS head over base is a demonstrated regression if its two-sided 95% lower bound
  exceeds 1.10. Either fails. Everything else is reported.

**`ni` and `replicated`: the CI off-target noninferiority** (condition 2, and condition 7's CI
criteria; registration §2.11, the replicated programme), from the sharded workflow
(.github/workflows/target-gated-ni.yml, the Guardian's D077). The standing grid's 42 cells (the
4-CPU, 2-core runner × the seven corpora × 1, 2 and 4 threads × wall and CPU time) in nine fixed
shards (`SHARDS`: a corpus each, `wikidata` split by thread count), each shard one runner.
- **The unit is a dispatch.** The programme is `NI_DISPATCHES` (k = 12) distinct, sequential
  dispatches of the workflow, each `NI_ROUNDS` (R = 240) paired rounds a cell on its own shard's
  runner. `ni` admits one dispatch and writes its dispatch-level estimates D_{c,j}: the mean of
  cell c's R paired log ratios, head over base. Nothing in a dispatch is a verdict.
- **`replicated`** takes all k dispatch verdicts and, per cell, the bound
  U_c = exp(mean_j D_{c,j} + t(0.95, k − 1) · sd_j(D_{c,j}) / √k) ≤ 1.02 at all 42 cells (an
  intersection-union test, nominal 5%). **The model, explicitly conditional:** the k dispatch
  contrasts of a cell are independent, identically distributed and approximately normal. Fresh
  VMs, sequential launches and fixed spacing do not prove that, nor a stable runner mix; under
  a non-normal or shifting mixture the size can exceed 5%, and the intersection-union
  construction does not repair a cell's size. The estimand is the scheduled-period runner-draw
  mean under these assumptions, not a guaranteed fleet-wide bound. No within-dispatch
  dependence model is used, and no trend veto: within-dispatch structure enters only through
  each D_{c,j}.
- **The programme's admission** (`replicated`): exactly k dispatch verdicts, each admitted, of
  the same head, with k distinct run ids; every cell's k estimates present and finite; the
  frozen spacing (`SPACING_MIN`): each dispatch's first job starts at least 60 minutes after
  the previous one's first job and not before the previous one's last shard job has completed
  (an overrun delays the next dispatch; it never skips or replaces one). Any dispatch INVALID,
  missing, duplicated or out of spacing: the programme is INVALID. No replacement, no retry.
- **The schedule within a dispatch:** `ab`'s arm order with two arms is head-first, base-first,
  base-first, head-first, repeated (`ORDER_CYCLE`, from round 0); R = 240 is 60 whole cycles,
  so a constant or linearly varying order coefficient cancels in each D_{c,j}. The judge
  refuses a row order that is not the registered traversal (the chronology, before grouping).
- **All rows are kept:** no trimming, no robust estimator. A row's `waited_s` is the settle
  wait plus the child's own execution, rounded to whole seconds (`ab`): it is never summed as
  pure waiting nor subtracted from wall.
- **Admission, every shard:**
  - the run's whole history reconciled, fail-closed (`history_problems`): every job of every
    attempt (the API's, with its steps) and every artifact GitHub lists, downloaded or not. An
    attempt counts as measured if its artifact holds rows or a start marker, or the API shows
    its measurement step began. Exactly one attempt measured, the shard's last, its job and
    step succeeded, its artifact listed and downloaded. An earlier attempt passes only as a
    verified setup failure (job failed, measurement step skipped, no rows or marker).
    Anything unknown refuses;
  - every file matching the manifest's SHA-256;
  - labelled `qualification` (a `smoke` run is never evidence);
  - the registered head (the candidate plus the frozen file alone), base, run, rounds, corpus
    and threads, and the frozen CI toolchain;
  - a runner of 4 CPUs on 2 cores, two a core, whose lscpu model is not the target's;
  - the predicate read back NOT TARGET on that runner (svc_target, checked whole);
  - the registered corpus digest;
  - every run successful, placed as `--spread` places it, every round once per arm and
    thread, one output digest.
- **Anything missing or failing:** INVALID, and the route fails (§2.5).
- **Condition 7 on CI:** a wall cell flagged slower (its two-sided 95% dispatch-level interval
  above 1) is a harm signal, for `confirm`. Peak RSS head over base whose two-sided 95%
  dispatch-level lower bound exceeds 1.10 fails. Condition 7's confirmation under replication
  is the Guardian's to rule; `confirm` is unchanged in kind.

`manifest` writes the aggregate's manifest; the workflow's `aggregate` job runs it and `ni`.

**`confirm`: the inherited harm-confirmation rule**, on the one confirmation allowed (§2.6),
which must be of distinct dispatches (no run id shared) of the same head. A wall cell flagged slower in the registered run and flagged slower again in the
confirmation run on the same head is a confirmed harm, and fails condition 7. A cell not
flagged again is unresolved, and reported. The confirmation run never enters condition 2: it
cannot rescue a failed cell, replace the registered run, or be pooled with it.

None of the outcomes is a reason to retry: one attempt (condition 6).
"""

import argparse
import datetime
import json
import math
import os
import re
import statistics
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import svc_target_check  # noqa: E402

# (helpers shared with bench/service_guarded.py's admission, carried here so this judge stands
# alone on master: the earlier attempt's judge is #302's, not this tooling's)
HASH = re.compile(r"[0-9a-f]{16,}")
# the time master was fetched at the freeze (UTC), beside the base it names
FETCHED = re.compile(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ")


def is_int(x):
    return isinstance(x, int) and not isinstance(x, bool)


def positive(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def plan_of(threads):
    """(leaders, plain helpers, pipelined helpers, service workers) from ab's `thread_cpu`
    ("name=seconds ...") or `observed_threads` ("name ..."): leaders or workers `mibz2-N`, a
    plain team's helpers `mibz2-coop-N`, a pipelined team's `mibz2-coop-pN`, service workers
    `mibz2-svc-N` (the hub's feeder, `mibz2-svc-feed`, is not counted)"""
    names = [p.partition("=")[0] for p in threads.split()]
    return (
        sum(1 for n in names if re.fullmatch(r"mibz2-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-coop-p\d+", n)),
        sum(1 for n in names if re.fullmatch(r"mibz2-svc-\d+", n)),
    )


def label(plans):
    out = []
    for leaders, plain, piped, svc in plans:
        helpers = plain + piped
        kind = "pipelined" if piped and not plain else "plain" if plain and not piped else "mixed"
        s = f" + {svc} service" if svc else ""
        if helpers == 0 and not svc:
            out.append(f"{leaders} workers")
        elif leaders and helpers % leaders == 0:
            out.append(f"{leaders}×{1 + helpers // leaders} {kind}{s}")
        else:
            out.append(f"{leaders} leaders, {plain} plain, {piped} pipelined, {svc} service")
    return " / ".join(out) or "nothing"


FROZEN = "service-tg.frozen"
MARGIN = 1.02
RSS_MARGIN = 1.10
# the workstation A/B
ROUNDS = 128
TARGET = 10
FALLBACK = (9, 11)
THREADS = (9, 10, 11)
K = 2  # condition 3's intervals: N = 1 cell, two tests
M = len(FALLBACK)  # condition 4's contrasts
# plans as (leaders, plain helpers, pipelined helpers, service workers)
FOUR_BY_TWO = (4, 4, 0, 0)
PIPED_3X3 = (3, 0, 6, 0)
SVC2 = (2, 4, 0, 2)
EXPECT = {
    9: {"base": FOUR_BY_TWO, "head": FOUR_BY_TWO},
    10: {"base": PIPED_3X3, "head": SVC2},
    11: {"base": PIPED_3X3, "head": PIPED_3X3},
}
CORPUS = "wikidata"
BASE_SUFFIX = "/bin/ab-base"
# the CI grid at the pinned bench revision (.github/workflows/bench.yml, examples/ab.rs `gen`),
# measured by .github/workflows/target-gated-ni.yml in nine fixed shards (D077)
CI_CORPORA = ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata")
CI_THREADS = (1, 2, 4)
CI_CPUS = 4
CI_CORES = 2
CI_ARMS = ("mibz2", "mibz2@base/ab")
CI_METRICS = ("wall", "cpu")
NI_ROUNDS = 240  # R: every cell's paired rounds in one dispatch, fixed (registration §2.11)
NI_DISPATCHES = 12  # k: the programme's distinct sequential dispatches (§2.11)
SPACING_MIN = 60  # minutes, at least, from one dispatch's first job to the next's (§2.11)
# `ab`'s arm order with two arms (examples/ab.rs `arm_order`): whether the head runs first in
# round r is ORDER_CYCLE[r % 4]. Sum of signs 0 and sum of i * sign 0 over a cycle: a constant
# or linear order coefficient cancels within each complete cycle
ORDER_CYCLE = (True, False, False, True)
assert NI_ROUNDS % len(ORDER_CYCLE) == 0, "a dispatch is whole order cycles"
SHARDS = {c: (c, (1, 2, 4)) for c in CI_CORPORA if c != "wikidata"}
SHARDS.update({f"wikidata-{t}": ("wikidata", (t,)) for t in CI_THREADS})
# the corpora's SHA-256 (ab gen at scale 4; bench/wikidata.json's): `text` is built from the
# candidate's own sources, so its digest is the freeze's (`corpus_text`)
CORPUS_SHA256 = {
    "onebyte": "22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e",
    "pattern": "0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407",
    "random": "10d444e6204289af739688e54a348940d7915957172ea65035b08b72dfdc3f18",
    "records": "ff8b5f6ed3dd44459908a01a588f6979b2544c6a444c9520741a2aa96c391f8c",
    "runs": "af3be4d130faf312fe087baee037ca0d802dcf9cd6aefeefe9d9373b25a92fb8",
    "wikidata": "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915",
}
SHARD_FILES = ("shard.json", "rustc.txt", "binaries.sha256", "lscpu.txt", "env.json", "preflight.json",
               "preflight.exit", "preflight-check.json", "corpus.sha256", "measurement-started.txt",
               "measurement-ended.txt", "runs.jsonl", "row-times.txt", "stamp.exit")
# the row timestamps (bench/tg_ni_stamp.py): the runner's CLOCK_REALTIME (Python's time.time()),
# written to the millisecond, each the time the poller (every 0.2 s) saw the row, not when ab
# wrote it: an observation time, with no hard bound on scheduling latency. The measurement's
# own markers are `date -u` to the second, so a row's time may fall up to TIME_SLACK_S outside
# them before the timing is refused as inconsistent
TIME_SLACK_S = 2.0
# the target's processor name (src/parallel.rs SERVICE_NAME): a runner reporting another is
# off the target by that fact alone
TARGET_NAME = svc_target_check.NAME


# ---------------------------------------------------------------------------------------
# Student's t, exactly enough: the regularized incomplete beta by its continued fraction
# (Numerical Recipes' betacf), the quantile by bisection. Checked against scipy in the tests.


def _betacf(a, b, x):
    tiny, qab, qap, qam = 1e-300, a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) > tiny else tiny)
    h = d
    for m in range(1, 400):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > tiny else tiny)
        c = 1.0 + aa / c
        c = c if abs(c) > tiny else tiny
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > tiny else tiny)
        c = 1.0 + aa / c
        c = c if abs(c) > tiny else tiny
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-15:
            break
    return h


def _betai(a, b, x):
    if x <= 0.0:
        return 0.0
    if x >= 1.0:
        return 1.0
    lb = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log1p(-x)
    if x < (a + 1.0) / (a + b + 2.0):
        return math.exp(lb) * _betacf(a, b, x) / a
    return 1.0 - math.exp(lb) * _betacf(b, a, 1.0 - x) / b


def t_cdf(t, df):
    p = 0.5 * _betai(df / 2.0, 0.5, df / (df + t * t))
    return 1.0 - p if t > 0 else p


def t_ppf(q, df):
    """Student's t quantile, q in (0.5, 1)"""
    lo, hi = 0.0, 1e3
    for _ in range(200):
        mid = (lo + hi) / 2
        if t_cdf(mid, df) < q:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def paired_logs(num, den, key, rounds):
    """log(num[key] / den[key]) per round"""
    return [math.log(num[i][key] / den[i][key]) for i in rounds]


def summary(logs):
    n = len(logs)
    return statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(n), n


def two_sided(logs, conf):
    m, se, n = summary(logs)
    t = t_ppf(1 - (1 - conf) / 2, n - 1)
    return {"ratio": math.exp(m), "lo": math.exp(m - t * se), "hi": math.exp(m + t * se), "conf": conf}


def upper(logs, conf):
    """The one-sided upper bound at confidence `conf` of the geometric mean"""
    m, se, n = summary(logs)
    return {"ratio": math.exp(m), "upper": math.exp(m + t_ppf(conf, n - 1) * se), "conf": conf}


# ---------------------------------------------------------------------------------------
# The frozen file


def registered(root=None):
    """The registered workload (bench/wikidata.json), the frozen commits, fetch time,
    workstation and CI toolchains and the `text` corpus's digest (bench/service-tg.frozen) and
    the pinned bench version (bench/report.py's BENCH), from the checkout the judge runs in"""
    root = root or os.path.dirname(HERE)
    with open(os.path.join(root, "bench", "wikidata.json")) as f:
        w = json.load(f)
    frozen = {}
    try:
        with open(os.path.join(root, "bench", FROZEN)) as f:
            lines = f.read().splitlines()
    except OSError:
        lines = []
    for line in lines:
        if line.startswith("#"):
            continue
        key, _, value = line.partition(" ")
        value = value.strip()
        if key in ("candidate", "base") and re.fullmatch(r"[0-9a-f]{40}", value):
            frozen[key] = value
        elif key == "base_fetched" and FETCHED.fullmatch(value):
            frozen[key] = value
        elif key == "rustc" and value.startswith("rustc "):
            frozen[key] = value
        elif key == "ni_rustc" and value.startswith("rustc "):
            frozen[key] = value
        elif key == "corpus_text" and re.fullmatch(r"[0-9a-f]{64}", value):
            frozen[key] = value
    bench = None
    try:
        with open(os.path.join(root, "bench", "report.py")) as f:
            m = re.search(r'^BENCH = "([^"]+)"', f.read(), re.M)
            bench = m and m.group(1)
    except OSError:
        pass
    return {"corpus_sha256": w["corpus_sha256"], "decoded_bytes": w["decoded_bytes"],
            "frozen": frozen, "bench": bench}


# ---------------------------------------------------------------------------------------
# The workstation A/B


def bundle_problems(files, reg):
    """(kind, problem) for the attempt's identity and workload files"""
    p = []
    if "DRY-RUN.txt" in files:
        p.append(("identity", "a DRY run: never evidence"))
    commits = dict(l.split()[:2] for l in files.get("commits.txt", "").splitlines() if len(l.split()) >= 2)
    for k in ("candidate", "base", "base_fetched"):
        if not reg["frozen"].get(k):
            p.append(("identity", f"bench/{FROZEN} names no {k}"))
        elif commits.get(k) != reg["frozen"][k]:
            p.append(("identity", f"commits.txt's {k} {commits.get(k)!r} is not the frozen {reg['frozen'][k]}"))
    rustc = reg["frozen"].get("rustc")
    if not rustc:
        p.append(("identity", f"bench/{FROZEN} names no rustc"))
    elif rustc not in [l.strip() for l in files.get("inputs.txt", "").splitlines()]:
        p.append(("identity", f"inputs.txt's toolchain is not the frozen {rustc!r}"))
    arms = [a for a in files.get("arms.txt", "").splitlines() if a.strip()]
    if not (len(arms) == 2 and arms[0] == "mibz2" and arms[1].startswith("mibz2@/") and arms[1].endswith(BASE_SUFFIX)):
        p.append(("identity", f"arms.txt is not the registered two arms: {arms}"))
        arms = []
    digests = {}
    for l in files.get("inputs.txt", "").splitlines():
        f = l.split(None, 1)
        if len(f) == 2 and re.fullmatch(r"[0-9a-f]{64}", f[0]):
            digests[f[1].strip()] = f[0]
    if arms and arms[1][len("mibz2@"):] not in digests:
        p.append(("identity", "inputs.txt gives no digest for the base executable run"))
    seen = files.get("corpus.sha256", "").split()
    if not seen or seen[0] != reg["corpus_sha256"]:
        p.append(("workload", f"corpus.sha256 {seen[0] if seen else 'missing'} is not the registered corpus's"))
    return p, arms


def row_problems(rows, arms, reg):
    p = []
    for i, r in enumerate(rows):
        if not isinstance(r, dict):
            p.append(f"row {i}: not a JSON object")
            continue
        q = []
        if r.get("arm") not in arms:
            q.append(f"arm {r.get('arm')!r} not one of arms.txt's")
        if not (is_int(r.get("threads")) and r["threads"] in THREADS):
            q.append(f"threads {r.get('threads')!r}")
        if not (is_int(r.get("round")) and 0 <= r["round"] < ROUNDS):
            q.append(f"round {r.get('round')!r} not an integer in 0..{ROUNDS - 1}")
        if r.get("termination") != "success":
            q.append(f"termination {r.get('termination')!r}, not 'success'")
        if r.get("err") != "":
            q.append(f"err {r.get('err')!r}: a failed run (or no err field)")
        if "broken" in r:
            q.append("broken: the child printed no record")
        for k in ("exit", "status"):
            if k in r and r[k] != 0:
                q.append(f"{k} {r[k]!r}")
        for k in ("wall", "cpu", "rss_kb"):
            if not positive(r.get(k)):
                q.append(f"{k} {r.get(k)!r} not finite and positive")
        if not (isinstance(r.get("hash"), str) and HASH.fullmatch(r["hash"])):
            q.append(f"hash {r.get('hash')!r} not a hex digest")
        if not (is_int(r.get("out")) and r["out"] == reg["decoded_bytes"]):
            q.append(f"out {r.get('out')!r} not the registered {reg['decoded_bytes']} bytes")
        if r.get("corpus") != CORPUS:
            q.append(f"corpus {r.get('corpus')!r}")
        if not (isinstance(r.get("thread_cpu"), str) and isinstance(r.get("observed_threads"), str)):
            q.append("a thread record missing")
        if q:
            p.append(f"row {i}: " + ", ".join(q))
    return p


def judge(files, rows, unreadable=(), reg=None):
    """The workstation attempt's verdict (see the module's doc)"""
    reg = reg or registered()
    v = {"valid": False, "failure": None, "problems": [], "cells": {}, "preflight": None,
         "target": None, "fallback": None, "other": None, "qualifies": False, "reasons": []}

    def fail(kind, problems):
        v["failure"] = v["failure"] or kind
        v["problems"] += problems

    bp, arms = bundle_problems(files, reg)
    for kind in ("identity", "workload"):
        if any(k == kind for k, _ in bp):
            fail(kind, [m for k, m in bp if k == kind])
    try:
        record = json.loads(files.get("preflight.json", ""))
    except json.JSONDecodeError:
        record = None
    try:
        status = int(files.get("preflight.exit", "").strip())
    except ValueError:
        status = None
    verdict, why = svc_target_check.check(record, status)
    v["preflight"] = {"verdict": verdict, "problems": why}
    if verdict != "TARGET":
        fail("preflight", [f"the preflight is {verdict}" + (f": {'; '.join(why)}" if why else "")])
    rp = [f"line {n} of runs.jsonl is not JSON" for n in unreadable] + row_problems(rows, arms, reg)
    if rp:
        fail("structural", rp[:8] + ([f"... and {len(rp) - 8} more"] if len(rp) > 8 else []))
    if v["failure"]:
        return v
    build = {arms[0]: "head", arms[1]: "base"}
    by = {}
    for r in rows:
        by.setdefault((build[r["arm"]], r["threads"]), []).append(r)
    if len({r["hash"] for r in rows}) != 1:
        fail("structural", ["more than one output digest"])
    rounds = range(ROUNDS)
    for t in THREADS:
        runs = {b: by.get((b, t), []) for b in ("base", "head")}
        complete = all(sorted(r["round"] for r in runs[b]) == list(rounds) for b in runs)
        seen = {
            b: sorted({plan_of(r[k]) for r in runs[b] for k in ("thread_cpu", "observed_threads")})
            for b in runs
        }
        plans_ok = all(seen[b] == [EXPECT[t][b]] for b in runs)
        c = {"threads": t, "complete": complete, "plans": plans_ok, "seen": seen}
        if not complete:
            fail("structural", [f"{t} threads: rounds incomplete or repeated"])
        if not plans_ok:
            fail("structural", [
                f"{t} threads: ran base {label(seen['base'])}, head {label(seen['head'])}; "
                f"expected base {label([EXPECT[t]['base']])}, head {label([EXPECT[t]['head']])}"
            ])
        if complete:
            base = {r["round"]: r for r in runs["base"]}
            head = {r["round"]: r for r in runs["head"]}
            wall_hb = paired_logs(head, base, "wall", rounds)
            cpu_hb = paired_logs(head, base, "cpu", rounds)
            c["speed95"] = two_sided([-x for x in wall_hb], 0.95)
            c["cpu95"] = two_sided(cpu_hb, 0.95)
            c["rss95"] = two_sided(paired_logs(head, base, "rss_kb", rounds), 0.95)
            if t == TARGET:
                c["speed_k"] = two_sided([-x for x in wall_hb], 1 - 0.05 / K)
                c["cpu_upper_k"] = upper(cpu_hb, 1 - 0.05 / K)
            else:
                c["wall_upper_m"] = upper(wall_hb, 1 - 0.05 / M)
            c["median"] = {
                b: {k: statistics.median(r[k] for r in runs[b]) for k in ("wall", "cpu", "rss_kb")}
                for b in runs
            }
        v["cells"][t] = c
    if v["failure"]:
        return v
    v["valid"] = True
    c = v["cells"][TARGET]
    v["target"] = {
        "wall": c["speed_k"]["lo"] > MARGIN,
        "cpu": c["cpu_upper_k"]["upper"] <= MARGIN,
    }
    v["fallback"] = {t: v["cells"][t]["wall_upper_m"]["upper"] <= MARGIN for t in FALLBACK}
    cpu_harm = [t for t in FALLBACK if v["cells"][t]["cpu95"]["lo"] > MARGIN]
    rss_harm = [t for t in THREADS if v["cells"][t]["rss95"]["lo"] > RSS_MARGIN]
    v["other"] = {"cpu_regression": cpu_harm, "rss_regression": rss_harm}
    if not v["target"]["wall"]:
        v["reasons"].append(f"condition 3: wall speed at 10 threads, lower bound {c['speed_k']['lo']:.4f}, not above {MARGIN}")
    if not v["target"]["cpu"]:
        v["reasons"].append(f"condition 3: CPU at 10 threads, upper bound {c['cpu_upper_k']['upper']:.4f}, above {MARGIN}")
    for t in FALLBACK:
        if not v["fallback"][t]:
            v["reasons"].append(
                f"condition 4: NOT SHOWN at {t} threads, wall upper bound {v['cells'][t]['wall_upper_m']['upper']:.4f} above {MARGIN}")
    for t in cpu_harm:
        v["reasons"].append(f"condition 7: CPU regression demonstrated at {t} threads (lower bound above {MARGIN})")
    for t in rss_harm:
        v["reasons"].append(f"condition 7: peak RSS regression demonstrated at {t} threads (lower bound above {RSS_MARGIN})")
    v["qualifies"] = not v["reasons"]
    return v


def fmt(r):
    return f"{r['ratio']:.4f} [{r['lo']:.4f}, {r['hi']:.4f}]" if r else "–"


def markdown(v):
    out = [
        "# The guarded service default under the target-gated route: the workstation A/B",
        "",
        f"Paired geometric means over {ROUNDS} rounds. Speed is base wall over head wall (above 1: the head "
        "is faster); CPU and peak RSS are head over base (above 1: the head spends more). Condition 3 at 10 "
        f"threads: speed's two-sided {100 * (1 - 0.05 / K):.1f}% interval, lower bound above {MARGIN}; CPU's "
        f"one-sided {100 * (1 - 0.05 / K):.1f}% upper bound at most {MARGIN}. Condition 4 at 9 and 11: wall "
        f"head over base, one-sided {100 * (1 - 0.05 / M):.1f}% upper bound at most {MARGIN}.",
        "",
        "| threads | role | plans, base → head | speed (95%) | judged bound | CPU (95%) | peak RSS (95%) |",
        "|--:|---|---|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        seen = c.get("seen", {})
        plans = f"{label(seen.get('base', []))} → {label(seen.get('head', []))}"
        if t == TARGET:
            role = "on target (condition 3)"
            judged = (f"speed lo {c['speed_k']['lo']:.4f}; CPU up {c['cpu_upper_k']['upper']:.4f}"
                      if "speed_k" in c else "–")
        else:
            role = "fallback (condition 4)"
            judged = f"wall up {c['wall_upper_m']['upper']:.4f}" if "wall_upper_m" in c else "–"
        out.append(f"| {t} | {role} | {plans} | {fmt(c.get('speed95'))} | {judged} | {fmt(c.get('cpu95'))} "
                   f"| {fmt(c.get('rss95'))} |")
    pf = v.get("preflight") or {}
    out += ["", f"**The preflight:** {pf.get('verdict', '–')}."]
    out += ["", f"**Valid:** {'yes' if v['valid'] else 'no' + (' (' + v['failure'] + ')' if v['failure'] else '')}"]
    out += [f"- {p}" for p in v["problems"]]
    if v["valid"]:
        out += ["", "**The workstation conditions (3, 4 and 7's workstation criteria):** "
                + ("**met**." if v["qualifies"] else "**not met**:")]
        out += [f"- {r}" for r in v["reasons"]]
        out += [
            "",
            "- This is the workstation half only: qualification also needs condition 2 on CI "
            "(`service_tg.py ci`), conditions 1, 5 and 6 as registered, and condition 7's tests, parity and green CI.",
            "- NOT SHOWN at a fallback budget is not demonstrated harm; it fails qualification all the same.",
            "- The intervals are a Bonferroni family within each condition (K = 2, M = 2), not one simultaneous "
            "statement across conditions.",
            "- One i9-10900K, the bench's Wikidata sample: the coverage argument for the rest of the deployment "
            "class is the registration's, not this run's.",
        ]
    return "\n".join(out) + "\n"


FILES = ("commits.txt", "arms.txt", "inputs.txt", "corpus.sha256", "preflight.json", "preflight.exit", "DRY-RUN.txt")


def read_jsonl(path):
    rows, unreadable = [], []
    try:
        with open(path, errors="replace") as f:
            lines = f.read().splitlines()
    except OSError:
        return [], [0]
    for n, line in enumerate(lines, 1):
        if line.strip():
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                unreadable.append(n)
    return rows, unreadable


def load(attempt):
    files = {}
    for n in FILES:
        try:
            with open(os.path.join(attempt, n), errors="replace") as f:
                files[n] = f.read()
        except OSError:
            pass
    rows, unreadable = read_jsonl(os.path.join(attempt, "runs.jsonl"))
    return files, rows, unreadable


# ---------------------------------------------------------------------------------------
# CI: condition 2, by the sharded workflow (.github/workflows/target-gated-ni.yml)


def head_problems(head, cand, root=None):
    """The registered CI head must be the frozen candidate plus the frozen file and nothing
    else: the candidate an ancestor of it, and the only path that differs bench/service-tg.frozen"""
    root = root or os.path.dirname(HERE)
    try:
        subprocess.run(["git", "merge-base", "--is-ancestor", cand, head], cwd=root, check=True,
                       capture_output=True)
        diff = subprocess.run(["git", "diff", "--name-only", cand, head], cwd=root, check=True,
                              capture_output=True, text=True).stdout.split()
    except (OSError, subprocess.CalledProcessError) as e:
        return [f"git could not relate the head {head} to the candidate {cand}: {e}"]
    other = [d for d in diff if d != f"bench/{FROZEN}"]
    return [f"the head differs from the candidate beyond the frozen file: {other}"] if other else []


def sha256_file(path):
    import hashlib

    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


MEASURE_STEP = "Run (the measurement)"
ARTIFACT = re.compile(r"tg-ni-(.+)-a([0-9]+)")


def manifest(root, artifacts=(), jobs=(), run=None, attempt=None, label=None):
    """The run's whole history, for the judge: every shard artifact GitHub lists (`artifacts`,
    the API's records, all attempts), each with the files downloaded under `root` (as
    `gh run download` leaves them: `tg-ni-<shard>-a<attempt>/<shard>/...`) and each file's
    SHA-256, or `files: null` where the listed artifact was not downloaded; any downloaded
    directory the API did not list (`unlisted`); and every job of every attempt with its steps
    (`jobs`, the API's `filter=all`)"""
    listed = {a.get("name"): a for a in artifacts if ARTIFACT.fullmatch(str(a.get("name", "")))}
    on_disk = sorted(os.listdir(root)) if os.path.isdir(root) else []
    out = {"schema": "mibz2.tg_ni_manifest/2", "run": run, "attempt": attempt, "label": label,
           "artifacts": [], "unlisted": [], "jobs": list(jobs)}
    for name in sorted(set(listed) | {n for n in on_disk if ARTIFACT.fullmatch(n)}):
        m = ARTIFACT.fullmatch(name)
        if m.group(1) == "aggregate":
            continue
        d = os.path.join(root, name, m.group(1))
        files = None
        if name in on_disk and os.path.isdir(d):
            files = {f: sha256_file(os.path.join(d, f)) for f in sorted(os.listdir(d))
                     if os.path.isfile(os.path.join(d, f))}
        if name not in listed:
            out["unlisted"].append(name)
        a = listed.get(name, {})
        out["artifacts"].append({"name": name, "shard": m.group(1), "attempt": int(m.group(2)),
                                 "id": a.get("id"), "digest": a.get("digest"),
                                 "expired": a.get("expired"), "files": files})
    return out


def shard_jobs(jobs, sid):
    """The API's jobs of one shard, every attempt: the matrix job is named `shard (<id>, ...)`"""
    return [j for j in jobs if str(j.get("name", "")).startswith(f"shard ({sid},")]


def measure_step(job):
    """The job's measurement step as the API records it, or None if its steps are unknown"""
    steps = job.get("steps")
    if not isinstance(steps, list) or not steps:
        return None
    hit = [s for s in steps if s.get("name") == MEASURE_STEP]
    return hit[0] if len(hit) == 1 else {"name": MEASURE_STEP, "conclusion": "absent"}


def history_problems(sid, jobs, arts, head=None, run=None):
    """§2.5's attempt rule, fail-closed, from the authoritative history: (problems, the measured
    attempt's artifact record). An attempt counts as measured if its artifact holds rows or a
    start marker, OR the API shows its measurement step began; exactly one attempt may have
    measured, it must be the shard's last, its job and step must have succeeded, and its
    artifact must be listed and downloaded. An earlier attempt is a recoverable setup failure
    only on positive evidence: its job failed and its measurement step was skipped, and its
    artifact (if any) holds no rows or marker. Anything unknown refuses"""
    p = []
    js = shard_jobs(jobs, sid)
    by_attempt = {}
    for j in js:
        k = j.get("run_attempt")
        if not is_int(k) or k in by_attempt:
            p.append(f"{sid}: job attempts unknown or repeated ({k!r})")
            continue
        if head is not None and j.get("head_sha") != head:
            p.append(f"{sid}: attempt {k}'s job ran at {j.get('head_sha')!r}, not the head")
        if run is not None and j.get("run_id") != run:
            p.append(f"{sid}: attempt {k}'s job belongs to run {j.get('run_id')!r}, not {run}")
        by_attempt[k] = j
    for a in arts:
        if a["attempt"] not in by_attempt:
            p.append(f"{sid}: an artifact of attempt {a['attempt']} with no job of that attempt")
    if not by_attempt:
        return p + [f"{sid}: no job in the run's history"], None
    attempts = sorted(by_attempt)
    art = {a["attempt"]: a for a in arts}
    measured = []
    for k in attempts:
        j, a = by_attempt[k], art.get(k)
        step = measure_step(j)
        files = (a or {}).get("files")
        has_data = bool(files) and ("runs.jsonl" in files or "measurement-started.txt" in files)
        if step is None:
            p.append(f"{sid}: attempt {k}: the job's steps are unknown")
            continue
        began = step.get("conclusion") not in ("skipped", "absent") or bool(step.get("started_at"))
        if has_data or began:
            measured.append(k)
            continue
        # (a setup failure: positive evidence it never measured)
        if not (j.get("status") == "completed" and j.get("conclusion") == "failure"
                and step.get("conclusion") == "skipped"):
            p.append(f"{sid}: attempt {k} neither measured nor a verified setup failure "
                     f"(job {j.get('status')}/{j.get('conclusion')}, step {step.get('conclusion')})")
        if a is not None and files is None:
            p.append(f"{sid}: attempt {k}'s listed artifact was not downloaded: its contents are unknown")
    if p:
        return p, None
    if len(measured) != 1:
        return [f"{sid}: {len(measured)} attempts that started measuring, not one"], None
    k = measured[0]
    if k != attempts[-1]:
        return [f"{sid}: an attempt after the measured one ({attempts})"], None
    j, a, step = by_attempt[k], art.get(k), measure_step(by_attempt[k])
    if not (j.get("status") == "completed" and j.get("conclusion") == "success"):
        p.append(f"{sid}: the measured attempt's job ended {j.get('status')}/{j.get('conclusion')}, not success")
    if step.get("conclusion") != "success":
        p.append(f"{sid}: the measurement step ended {step.get('conclusion')!r}, not success")
    if a is None or a.get("id") is None:
        p.append(f"{sid}: the measured attempt's artifact is not listed by GitHub")
    elif not a.get("files"):
        p.append(f"{sid}: the measured attempt's artifact was not downloaded")
    elif a.get("expired"):
        p.append(f"{sid}: the measured attempt's artifact expired")
    return p, (None if p else a)


def traversal(corpus, threads, rounds):
    """The registered file order of one shard's rows (§2.9): rounds 0 … R − 1; in each, the
    shard's thread counts in their registered order; at each, the two arms in `ORDER_CYCLE`'s
    order. Each item is (round, corpus, threads, arm)"""
    for rnd in range(rounds):
        arms = CI_ARMS if ORDER_CYCLE[rnd % len(ORDER_CYCLE)] else CI_ARMS[::-1]
        for t in threads:
            for arm in arms:
                yield (rnd, corpus, t, arm)


def sequence_problems(sid, rows, corpus, threads, rounds):
    """Why the rows, in file order, are not the registered traversal (empty: they are). Checked
    before any grouping by round, so that batches are consecutive in execution, not in labels:
    a permuted, reversed or regrouped file is refused, never sorted back"""
    want = list(traversal(corpus, tuple(threads), rounds))
    got = [(r.get("round"), r.get("corpus"), r.get("threads"), r.get("arm")) for r in rows]
    if got == want:
        return []
    bad = [i for i, (g, w) in enumerate(zip(got, want)) if g != w]
    first = bad[0] if bad else min(len(got), len(want))
    seen = got[first] if first < len(got) else "end of file"
    expected = want[first] if first < len(want) else "end of file"
    return [f"{sid}: rows not in the registered order (round, thread counts {list(threads)}, ab's arm cycle): "
            f"{len(got)} rows for {len(want)}, {len(bad) + abs(len(got) - len(want))} out of place, "
            f"first at row {first}: {seen}, expected {expected}"]


def rounds_problems(rounds):
    """Why `rounds` is not a dispatch of whole four-round order cycles (empty: it is)"""
    if not (is_int(rounds) and rounds > 0):
        return [f"R {rounds!r}: not a number of rounds"]
    if rounds % len(ORDER_CYCLE):
        return [f"R {rounds} is not whole {len(ORDER_CYCLE)}-round order cycles"]
    return []


def diagnostics(logs):
    """Reported, never judged: lag-1 autocorrelation of the paired logs in round order, and the
    two halves' geometric means"""
    n = len(logs)
    m = statistics.fmean(logs)
    var = sum((x - m) ** 2 for x in logs)
    r1 = sum((logs[i] - m) * (logs[i + 1] - m) for i in range(n - 1)) / var if var else 0.0
    h = n // 2
    return {"lag1": r1, "first_half": math.exp(statistics.fmean(logs[:h])),
            "second_half": math.exp(statistics.fmean(logs[h:]))}


def read_text(d, name):
    try:
        with open(os.path.join(d, name), errors="replace") as f:
            return f.read()
    except OSError:
        return None


def utc(stamp):
    """A `date -u +%Y-%m-%dT%H:%M:%SZ` marker as POSIX seconds, or None"""
    try:
        return datetime.datetime.strptime(stamp.strip(), "%Y-%m-%dT%H:%M:%SZ").replace(
            tzinfo=datetime.timezone.utc).timestamp()
    except (AttributeError, ValueError):
        return None


def read_row_times(d):
    """The stamper's file: (start time, {row index: time}) after its last truncation, or None
    if it is missing or malformed. Strict: every line `# start T`, `# truncated` or `I T`"""
    text = read_text(d, "row-times.txt")
    if text is None:
        return None
    start, times = None, {}
    for line in text.splitlines():
        f = line.split()
        if f[:2] == ["#", "start"] and len(f) == 3:
            try:
                start = float(f[2])
            except ValueError:
                return None
        elif f == ["#", "truncated"]:
            times = {}
        elif len(f) == 2:
            try:
                i, t = int(f[0]), float(f[1])
            except ValueError:
                return None
            if i in times:
                return None
            times[i] = t
        else:
            return None
    return start, times


def timing_problems(sid, d, n_rows):
    """The timing data, required complete and coherent (the Researcher's §2): the stamper exited
    0; its file has a start time and exactly one finite time for each of the n rows, 0 .. n - 1,
    nondecreasing in row order (equal times are legitimate under polling); and every time lies
    within the measurement's markers, give or take TIME_SLACK_S. Missing timing is a problem,
    never a zero"""
    p = []
    if (read_text(d, "stamp.exit") or "").strip() != "0":
        p.append(f"{sid}: the row stamper did not exit 0 ({(read_text(d, 'stamp.exit') or 'missing').strip()!r})")
    got = read_row_times(d)
    if got is None:
        return p + [f"{sid}: row-times.txt missing or malformed"]
    start, times = got
    if start is None or not math.isfinite(start):
        p.append(f"{sid}: row-times.txt has no start time")
    if sorted(times) != list(range(n_rows)):
        return p + [f"{sid}: row-times.txt covers {len(times)} rows, not 0 .. {n_rows - 1} once each"]
    seq = [times[i] for i in range(n_rows)]
    if not all(math.isfinite(t) for t in seq):
        return p + [f"{sid}: a row time is not finite"]
    if any(b < a for a, b in zip(seq, seq[1:])):
        p.append(f"{sid}: row times go backwards")
    lo, hi = utc(read_text(d, "measurement-started.txt") or ""), utc(read_text(d, "measurement-ended.txt") or "")
    if lo is None or hi is None:
        p.append(f"{sid}: the measurement markers are unreadable")
    elif seq and (seq[0] < lo - TIME_SLACK_S or seq[-1] > hi + TIME_SLACK_S
                  or (start is not None and (start < lo - TIME_SLACK_S or start > seq[0]))):
        p.append(f"{sid}: row times outside the measurement's markers")
    return p


def shard_problems(sid, d, reg, head, run_id, expect_rounds, attempt=None, repository="lmmx/mibz2",
                   label="qualification"):
    """What keeps one shard's measured attempt from being admitted (empty: admitted), and its
    rows by (corpus, threads, arm) and round"""
    p = []
    fz = reg["frozen"]
    corpus, threads = SHARDS[sid]
    missing = [f for f in SHARD_FILES if read_text(d, f) is None]
    if missing:
        return [f"{sid}: missing {missing}"], {}
    try:
        info = json.loads(read_text(d, "shard.json"))
    except json.JSONDecodeError:
        return [f"{sid}: shard.json unreadable"], {}
    want = {"schema": "mibz2.tg_ni_shard/1", "label": label, "shard": sid, "corpus": corpus,
            "threads": list(threads), "rounds": expect_rounds, "head": head, "base": fz.get("base"),
            "github_sha": head, "workflow_sha": head, "run_id": run_id, "run_attempt": attempt,
            "repository": repository}
    for k, v in want.items():
        if info.get(k) != v:
            p.append(f"{sid}: shard.json {k} {info.get(k)!r}, not {v!r}")
    if not str(info.get("workflow_ref", "")).startswith(f"{repository}/.github/workflows/target-gated-ni.yml@"):
        p.append(f"{sid}: not run by target-gated-ni.yml ({info.get('workflow_ref')!r})")
    rustc = read_text(d, "rustc.txt").strip()
    if fz.get("ni_rustc"):
        if rustc != fz["ni_rustc"]:
            p.append(f"{sid}: toolchain {rustc!r}, not the frozen {fz['ni_rustc']!r}")
    elif fz.get("ni_toolchain"):
        if not rustc.startswith(f"rustc {fz['ni_toolchain']} "):
            p.append(f"{sid}: toolchain {rustc!r}, not the dispatched {fz['ni_toolchain']!r}")
    else:
        p.append(f"{sid}: no registered toolchain to check {rustc!r} against")
    try:
        env = json.loads(read_text(d, "env.json"))
    except json.JSONDecodeError:
        env = {}
    core_cpus = env.get("core_cpus") or []
    if not (env.get("cpus") == CI_CPUS and env.get("cores") == CI_CORES and len(core_cpus) == CI_CORES
            and all(len(str(c).split(",")) == 2 for c in core_cpus)):
        p.append(f"{sid}: the runner is not 4 CPUs on 2 cores, two a core: {env.get('cpus')!r} CPUs, "
                 f"{env.get('cores')!r} cores, {core_cpus!r}")
    model = next((l.split(":", 1)[1].strip() for l in read_text(d, "lscpu.txt").splitlines()
                  if l.startswith("Model name:")), None)
    if not model or model == TARGET_NAME:
        p.append(f"{sid}: lscpu's model {model!r} is not shown off the target")
    try:
        record = json.loads(read_text(d, "preflight.json"))
        status = int(read_text(d, "preflight.exit").strip())
    except (json.JSONDecodeError, ValueError):
        record, status = None, None
    verdict, why = svc_target_check.check(record, status)
    if verdict != "NOT TARGET":
        p.append(f"{sid}: the predicate's read-back is {verdict}, not NOT TARGET: {why}")
    digest = read_text(d, "corpus.sha256").split()
    want_digest = fz.get("corpus_text") if corpus == "text" else CORPUS_SHA256[corpus]
    if corpus == "text" and want_digest is None and fz.get("text_unchecked"):
        want_digest = digest[0] if digest else None  # (a smoke check before the freeze: recorded, unchecked)
    if not digest or not want_digest or digest[0] != want_digest:
        p.append(f"{sid}: corpus digest {digest[:1]}, not the registered {want_digest!r}")
    rows, unreadable = read_jsonl(os.path.join(d, "runs.jsonl"))
    if unreadable:
        p.append(f"{sid}: runs.jsonl lines not JSON: {unreadable[:5]}")
    p += timing_problems(sid, d, len(rows))
    # the chronology (§2.9), before anything is grouped: the file's rows, in file order, must be
    # exactly the registered nested traversal. Refused, never sorted or repaired
    q = sequence_problems(sid, rows, corpus, threads, expect_rounds)
    if q:
        return (p + q)[:20], {}
    by, at = {}, {}
    cores = [set(str(c).split(",")) for c in core_cpus]
    for i, r in enumerate(rows):
        key = (r.get("corpus"), r.get("threads"), r.get("arm"))
        q = []
        if key[0] != corpus or not is_int(key[1]) or key[1] not in threads or key[2] not in CI_ARMS:
            q.append(f"a cell outside the shard {key}")
        if r.get("termination") != "success":
            q.append(f"termination {r.get('termination')!r}")
        if r.get("err") != "":
            q.append(f"err {r.get('err')!r}")
        for k in ("wall", "cpu", "rss_kb"):
            if not positive(r.get(k)):
                q.append(f"{k} {r.get(k)!r}")
        if not (is_int(r.get("round")) and 0 <= r["round"] < expect_rounds):
            q.append(f"round {r.get('round')!r}")
        if not (isinstance(r.get("hash"), str) and HASH.fullmatch(r["hash"])):
            q.append(f"hash {r.get('hash')!r}")
        placed = str(r.get("cpus", ""))
        t = key[1]
        if t == 1 and not re.fullmatch(r"[0-9]+", placed):
            q.append(f"1 thread placed on {placed!r}")
        if t == 2 and not (len(placed.split(",")) == 2
                           and all(sum(c in core for c in placed.split(",")) == 1 for core in cores)):
            q.append(f"2 threads placed on {placed!r}, not one CPU a core")
        if t == 4 and placed != "all":
            q.append(f"4 threads placed on {placed!r}")
        if q:
            p.append(f"{sid} row {i}: " + ", ".join(q))
            continue
        cell = by.setdefault(key, {})
        if r["round"] in cell:
            p.append(f"{sid} row {i}: round {r['round']} repeated")
        cell[r["round"]] = r
        at[(key, r["round"])] = i
    if len({r.get("hash") for r in rows}) != 1:
        p.append(f"{sid}: {len({r.get('hash') for r in rows})} output digests, not one")
    for t in threads:
        for arm in CI_ARMS:
            if sorted(by.get((corpus, t, arm), {})) != list(range(expect_rounds)):
                p.append(f"{sid}: {corpus} {t}t {arm}: {len(by.get((corpus, t, arm), {}))} rounds, "
                         f"not 0..{expect_rounds - 1} once each")
    # (implied by the chronology above; kept as a second, local statement of it) the schedule
    # (§2.9): in each round and thread count, the two arms back to back, the head
    # first exactly in the rounds ORDER_CYCLE names
    for t in threads:
        bad = []
        for rnd in range(expect_rounds):
            h, b = at.get(((corpus, t, CI_ARMS[0]), rnd)), at.get(((corpus, t, CI_ARMS[1]), rnd))
            if h is None or b is None:
                continue
            if abs(h - b) != 1:
                bad.append(f"round {rnd}: the arms not back to back (rows {h}, {b})")
            elif (h < b) != ORDER_CYCLE[rnd % len(ORDER_CYCLE)]:
                bad.append(f"round {rnd}: {'head' if h < b else 'base'} first")
        if bad:
            p.append(f"{sid}: {corpus} {t}t: {len(bad)} rounds off ab's four-round schedule (head, base, base, "
                     f"head, the arms back to back), first {bad[0]}")
    return p[:20], by


def structural(root, man, head, reg, rounds, label, check_git=True):
    """The structural admission every use of a run shares (qualification, the pilot, the smoke
    check): the manifest's schema; nothing downloaded that GitHub didn't list; every registered
    shard present, its history reconciled fail-closed; every file of its measured attempt
    rehashed against the manifest; and its files, runner, read-back, corpus, timing and rows.
    (problems, {shard: (artifact record, rows by cell)})"""
    fz = reg["frozen"]
    p, out = [], {}
    if man.get("schema") != "mibz2.tg_ni_manifest/2":
        p.append(f"manifest schema {man.get('schema')!r}: not one that records the run's whole history")
    if man.get("label") != label:
        p.append(f"the run is labelled {man.get('label')!r}: not {label}")
    for name in man.get("unlisted") or []:
        p.append(f"{name}: downloaded but not listed by GitHub")
    if check_git and fz.get("candidate"):
        p += head_problems(head, fz["candidate"])
    run_id, arts, jobs = man.get("run"), man.get("artifacts", []), man.get("jobs") or []
    for sid in SHARDS:
        q, a = history_problems(sid, jobs, [x for x in arts if x["shard"] == sid], head=head, run=run_id)
        if q:
            p += q
            continue
        d = os.path.join(root, a["name"], sid)
        for f, h in a["files"].items():
            path = os.path.join(d, f)
            if not os.path.isfile(path) or sha256_file(path) != h:
                p.append(f"{sid}: {f} does not match the manifest's digest")
        q, by = shard_problems(sid, d, reg, head, run_id, rounds, attempt=a["attempt"], label=label)
        p += q
        if not q:
            out[sid] = (a, by)
    return p, out


def window(man):
    """A dispatch's (first job start, last shard-job completion) as epoch seconds, from the
    manifest's API record of the run's jobs (every attempt); None where unknown"""
    jobs = man.get("jobs") or []
    starts = [utc(j.get("started_at")) for j in jobs if j.get("started_at")]
    ends = [utc(j.get("completed_at")) for j in jobs
            if j.get("completed_at") and str(j.get("name", "")).startswith("shard (")]
    starts, ends = [x for x in starts if x is not None], [x for x in ends if x is not None]
    return (min(starts) if starts else None, max(ends) if ends else None)


def ni_judge(root, man, head, reg=None, check_git=True, rounds=NI_ROUNDS):
    """One dispatch of the replicated programme (registration §2.11): the structural admission
    (`structural`, label qualification, the frozen values, the chronology), then each cell's
    dispatch-level estimate D, the mean of its R paired log ratios on its own shard's runner.
    Not a verdict: `replicated` judges condition 2 from all k dispatches"""
    reg = reg or registered()
    fz = reg["frozen"]
    start, end = window(man)
    v = {"schema": "mibz2.tg_ni_dispatch/1", "valid": False, "problems": [], "cells": {}, "shards": {},
         "run": man.get("run"), "head": head, "label": man.get("label"), "rounds": rounds,
         "started": start, "completed": end}
    p = v["problems"]
    p += rounds_problems(rounds)
    for k in ("candidate", "base", "ni_rustc", "corpus_text"):
        if k not in fz:
            p.append(f"bench/{FROZEN} names no {k}")
    if man.get("label") != "qualification":
        p.append(f"the run is labelled {man.get('label')!r}: not qualification (a smoke check or the pilot is never evidence)")
    if not is_int(man.get("run")):
        p.append(f"the manifest names no run id ({man.get('run')!r})")
    q, admitted = structural(root, man, head, reg, rounds, "qualification", check_git=check_git)
    p += [x for x in q if not x.startswith("the run is labelled")]
    for sid, (a, by) in admitted.items():
        model = next((l.split(":", 1)[1].strip() for l in (read_text(os.path.join(root, a["name"], sid), "lscpu.txt") or "").splitlines()
                      if l.startswith("Model name:")), None)
        v["shards"][sid] = {"attempt": a["attempt"], "artifact": a.get("id"), "digest": a.get("digest"), "model": model}
        corpus, threads = SHARDS[sid]
        if rounds_problems(rounds):
            continue
        for t in threads:
            h, b = by[(corpus, t, CI_ARMS[0])], by[(corpus, t, CI_ARMS[1])]
            cell = {"shard": sid}
            for metric in CI_METRICS + ("rss_kb",):
                logs = paired_logs(h, b, metric, range(rounds))
                cell[metric] = {"d": statistics.fmean(logs), "diagnostics": diagnostics(logs)}
            v["cells"][f"{corpus}/{t}"] = cell
    if p:
        v["cells"] = {}
        return v
    v["valid"] = True
    return v


def dispatch_stats(ds, conf_upper=0.95, conf_two=0.95):
    """The dispatch-level bound (§2.11): k estimates D_j of one cell and metric; the mean, its
    standard error sd/sqrt(k), t with k - 1 df"""
    k = len(ds)
    m = statistics.fmean(ds)
    se = statistics.stdev(ds) / math.sqrt(k)
    tu, tt = t_ppf(conf_upper, k - 1), t_ppf(1 - (1 - conf_two) / 2, k - 1)
    return {"ratio": math.exp(m), "upper": math.exp(m + tu * se), "lo95": math.exp(m - tt * se),
            "hi95": math.exp(m + tt * se), "se": se, "d": list(ds)}


def replicated(dispatches, head, k=NI_DISPATCHES, spacing_min=SPACING_MIN, rounds=NI_ROUNDS):
    """Condition 2 from the whole replicated programme (§2.11): `dispatches`, every dispatch's
    `ni_judge` verdict, in dispatch order. Admitted only as all k, each valid, of `head`, with
    distinct run ids, every cell's estimates finite, at the frozen spacing; then per cell and
    metric U = exp(mean D + t(0.95, k - 1) sd(D)/sqrt(k)) <= 1.02 at all 42"""
    runs = [d.get("run") if isinstance(d, dict) else None for d in dispatches]
    v = {"schema": "mibz2.tg_ni_replicated/1", "valid": False, "problems": [], "cells": {}, "flags": [],
         "rss_regression": [], "passes": False, "failing": [], "condition2": "INVALID", "k": k,
         "runs": runs, "head": head, "models": {}}
    p = v["problems"]
    if len(dispatches) != k:
        p.append(f"{len(dispatches)} dispatches, not the registered {k}")
    for i, d in enumerate(dispatches):
        if not isinstance(d, dict):
            p.append(f"dispatch {i + 1}: no readable verdict")
            continue
        if not d.get("valid"):
            p.append(f"dispatch {i + 1} (run {d.get('run')!r}) is INVALID: "
                     + "; ".join(map(str, (d.get("problems") or ["not admitted"])[:3])))
        if d.get("head") != head:
            p.append(f"dispatch {i + 1} (run {d.get('run')!r}) is of head {d.get('head')!r}, not {head}")
        if d.get("rounds") != rounds:
            p.append(f"dispatch {i + 1} (run {d.get('run')!r}): R {d.get('rounds')!r}, not {rounds}")
    if not all(is_int(r) for r in runs) or len(set(runs)) != len(runs):
        p.append(f"the run ids are not {len(runs)} distinct dispatches: {runs}")
    # (the frozen spacing: starts at least `spacing_min` apart, never before the previous
    # dispatch's last shard job completed; an overrun delays, it never skips or replaces)
    for i in range(1, len(dispatches)):
        a, b = dispatches[i - 1], dispatches[i]
        if not (isinstance(a, dict) and isinstance(b, dict)):
            continue
        s0, e0, s1 = a.get("started"), a.get("completed"), b.get("started")
        if not all(isinstance(x, (int, float)) for x in (s0, e0, s1)):
            p.append(f"dispatches {i} and {i + 1}: their times are unknown, so the spacing is unshown")
            continue
        if s1 < s0 + 60 * spacing_min:
            p.append(f"dispatch {i + 1} started {(s1 - s0) / 60:.1f} minutes after dispatch {i}, not {spacing_min}")
        if s1 < e0:
            p.append(f"dispatch {i + 1} started before dispatch {i}'s shards completed")
    if not p:
        for key in (f"{c}/{t}" for c in CI_CORPORA for t in CI_THREADS):
            cell = {}
            for metric in CI_METRICS + ("rss_kb",):
                ds = [((d["cells"].get(key) or {}).get(metric) or {}).get("d") for d in dispatches]
                if not all(isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) for x in ds):
                    p.append(f"{key} {metric}: not {k} finite dispatch estimates")
                    continue
                cell[metric] = dispatch_stats(ds)
            v["cells"][key] = cell
        for i, d in enumerate(dispatches):
            for sid, sh in (d.get("shards") or {}).items():
                v["models"].setdefault(sid, []).append(sh.get("model"))
    if p:
        v["cells"] = {}
        return v
    v["valid"] = True
    for key, cell in v["cells"].items():
        for metric in CI_METRICS:
            cell[metric]["pass"] = cell[metric]["upper"] <= MARGIN
            if not cell[metric]["pass"]:
                v["failing"].append(f"{key} {metric} upper {cell[metric]['upper']:.4f}")
        if cell["wall"]["lo95"] > 1.0:
            v["flags"].append(key)
        if cell["rss_kb"]["lo95"] > RSS_MARGIN:
            v["rss_regression"].append(key)
    v["passes"] = not v["failing"] and not v["rss_regression"]
    v["condition2"] = "PASS" if not v["failing"] else "NOT SHOWN"
    return v


def structure(root, man, head, base, rounds, label, toolchain, corpus_text=None, check_git=False):
    """The structural admission alone, for a smoke check or the pilot before the freeze (the
    Researcher's §4): what `structural` checks, against the dispatch's own base, rounds and
    toolchain. The `text` corpus's digest is checked against `corpus_text` if given, and
    otherwise recorded as unchecked. Green plumbing means this is valid, not merely that the
    judge refused the label"""
    reg = {"frozen": {"base": base, "ni_toolchain": toolchain, "corpus_text": corpus_text,
                      "text_unchecked": corpus_text is None}, "bench": None}
    p, admitted = structural(root, man, head, reg, rounds, label, check_git=check_git)
    return {"schema": "mibz2.tg_ni_structure/1", "valid": not p and len(admitted) == len(SHARDS),
            "label": label, "rounds": rounds, "run": man.get("run"), "head": head,
            "shards": sorted(admitted), "problems": p,
            "unchecked": [] if corpus_text else ["the text corpus's digest (no corpus_text before the freeze)"]}


def dispatch_markdown(v):
    out = [f"# Target-gated NI: one dispatch of the replicated programme (run {v.get('run')}): not a verdict", ""]
    if not v["valid"]:
        out += ["**Not admitted** (INVALID: the programme fails; nothing is re-run or replaced):"]
        out += [f"- {p}" for p in v["problems"]]
        return "\n".join(out) + "\n"
    out += [f"Admitted: all nine shards, R {v['rounds']}. Each cell's dispatch-level estimate D (the mean of its "
            f"paired log ratios) is in the JSON. Condition 2 is judged only from all {NI_DISPATCHES} dispatches "
            "(`service_tg.py replicated`).", "",
            "| shard | runner model |", "|---|---|"]
    out += [f"| {sid} | {sh.get('model')} |" for sid, sh in v["shards"].items()]
    return "\n".join(out) + "\n"


def ni_markdown(v):
    out = ["# The guarded service default under the target-gated route: CI condition 2 (replicated)", ""]
    if not v["valid"]:
        out += ["**Not admitted** (INVALID: the route fails; nothing is re-run or replaced):"]
        out += [f"- {p}" for p in v["problems"]]
        return "\n".join(out) + "\n"
    out += [
        f"Head over base, per cell: {v['k']} dispatch-level estimates, each the mean of {NI_ROUNDS} paired log ratios "
        f"on that dispatch's runner; the one-sided 95% upper bound exp(mean + t(0.95, {v['k'] - 1}) sd/sqrt({v['k']})), "
        f"at most {MARGIN} at every one of the 42 cells (an intersection-union test, nominal 5%, conditional on the "
        "dispatch model: the k contrasts independent, identically distributed and approximately normal. The "
        "intersection-union construction needs no independence across cells, but it does not repair a cell's "
        "size; the estimand is the scheduled-period runner-draw mean under these assumptions; registration §2.11).",
        f"Runs: {', '.join(map(str, v['runs']))}.",
        "",
        "| cell | wall | wall upper | CPU | CPU upper | peak RSS (95%) |",
        "|---|--:|--:|--:|--:|--:|",
    ]
    for key, c in v["cells"].items():
        w, cp, r = c["wall"], c["cpu"], c["rss_kb"]
        mark = lambda x: "" if x["pass"] else " **FAIL**"  # noqa: E731
        out.append(f"| {key} | {w['ratio']:.4f} | {w['upper']:.4f}{mark(w)} | {cp['ratio']:.4f} "
                   f"| {cp['upper']:.4f}{mark(cp)} | {r['ratio']:.4f} [{r['lo95']:.4f}, {r['hi95']:.4f}] |")
    c2 = "**not met** (NOT SHOWN): " + "; ".join(v["failing"]) if v["failing"] else "**met** at every cell"
    out += ["", f"**Condition 2:** {c2}."]
    out += [f"**Wall flags (harm signals, §2.6; condition 7 under replication is the Guardian's):** {', '.join(v['flags']) or 'none'}."]
    out += [f"**Peak RSS regression (condition 7):** {', '.join(v['rss_regression']) or 'none'}."]
    out += ["", "| shard | runner models, dispatch order |", "|---|---|"]
    out += [f"| {sid} | {', '.join(map(str, m))} |" for sid, m in v["models"].items()]
    return "\n".join(out) + "\n"


def run_ids(v):
    return v["runs"] if isinstance(v.get("runs"), list) else [v.get("run")]


def confirm(registered_v, confirmation_v):
    """The inherited harm-confirmation rule (§2.6): each wall flag of the registered verdict,
    flagged again (confirmed harm: fails condition 7) or not (unresolved). The two must be of
    distinct dispatches (no run id shared) of the same head, both valid; the same verdict twice
    is not a confirmation. It never enters condition 2. Under replication its unit and need are
    the Guardian's to rule (§2.11); this applies the rule as it stands"""
    no = {"valid": False, "confirmed": [], "unresolved": []}
    if not (registered_v.get("valid") and confirmation_v.get("valid")):
        return dict(no, problems=["a run is invalid"])
    r1, r2 = run_ids(registered_v), run_ids(confirmation_v)
    if not all(is_int(r) for r in r1 + r2) or set(r1) & set(r2):
        return dict(no, problems=[f"not distinct dispatches: runs {r1!r} and {r2!r}"])
    if registered_v.get("head") != confirmation_v.get("head"):
        return dict(no, problems=["the confirmation is of another head"])
    again = set(confirmation_v["flags"])
    flags = registered_v["flags"]
    one = lambda r: r[0] if len(r) == 1 else r  # noqa: E731
    return {"valid": True, "registered_run": one(r1), "confirmation_run": one(r2),
            "confirmed": [f for f in flags if f in again], "unresolved": [f for f in flags if f not in again]}


def write(prefix, v, md):
    with open(prefix + ".json", "w") as f:
        json.dump(v, f, indent=1, sort_keys=True)
    with open(prefix + ".md", "w") as f:
        f.write(md)
    print(md)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("attempt")
    r.add_argument("--out", required=True)
    m = sub.add_parser("manifest")
    m.add_argument("root")
    m.add_argument("--artifacts")
    m.add_argument("--jobs")
    m.add_argument("--run", type=int)
    m.add_argument("--attempt", type=int)
    m.add_argument("--label")
    n = sub.add_parser("ni")
    n.add_argument("root")
    n.add_argument("--manifest", required=True)
    n.add_argument("--head", required=True)
    n.add_argument("--out", required=True)
    rp = sub.add_parser("replicated")
    rp.add_argument("--dispatch", nargs=2, action="append", metavar=("ROOT", "MANIFEST"), required=True,
                    help="each dispatch's artifacts and manifest, in dispatch order, all k")
    rp.add_argument("--head", required=True)
    rp.add_argument("--out", required=True)
    st = sub.add_parser("structure")
    st.add_argument("root")
    st.add_argument("--manifest", required=True)
    st.add_argument("--head", required=True)
    st.add_argument("--base", required=True)
    st.add_argument("--rounds", type=int, required=True)
    st.add_argument("--label", required=True)
    st.add_argument("--toolchain", required=True)
    st.add_argument("--corpus-text")
    st.add_argument("--out", required=True)
    f = sub.add_parser("confirm")
    f.add_argument("registered", help="the registered verdict's JSON (`replicated`'s)")
    f.add_argument("--confirmation", required=True, help="the confirmation verdict's JSON")
    f.add_argument("--out", required=True)
    a = p.parse_args(argv)
    if a.cmd == "report":
        files, rows, unreadable = load(a.attempt)
        v = judge(files, rows, unreadable)
        write(a.out, v, markdown(v))
        return 0 if v["valid"] else 1
    if a.cmd == "manifest":
        arts = read_jsonl(a.artifacts)[0] if a.artifacts else []
        jobs = read_jsonl(a.jobs)[0] if a.jobs else []
        json.dump(manifest(a.root, arts, jobs, a.run, a.attempt, a.label), sys.stdout, indent=1, sort_keys=True)
        print()
        return 0
    if a.cmd == "structure":
        with open(a.manifest) as fh:
            man = json.load(fh)
        v = structure(a.root, man, a.head, a.base, a.rounds, a.label, a.toolchain, a.corpus_text)
        md = (f"# Structural admission ({a.label}, R {a.rounds})\n\n"
              + ("**Admitted:** all nine shards.\n" if v["valid"] else "**Not admitted:**\n\n"
                 + "".join(f"- {x}\n" for x in v["problems"]))
              + "".join(f"- unchecked: {x}\n" for x in v["unchecked"]))
        write(a.out, v, md)
        return 0 if v["valid"] else 2
    if a.cmd == "ni":
        with open(a.manifest) as fh:
            man = json.load(fh)
        v = ni_judge(a.root, man, a.head)
        write(a.out, v, dispatch_markdown(v))
        return 0 if v["valid"] else 2
    if a.cmd == "replicated":
        ds = []
        for root, mpath in a.dispatch:
            with open(mpath) as fh:
                ds.append(ni_judge(root, json.load(fh), a.head))
        v = replicated(ds, a.head)
        write(a.out, v, ni_markdown(v))
        return 0 if v["valid"] and v["passes"] else 1 if v["valid"] else 2
    def load_v(path):
        with open(path) as fh:
            return json.load(fh)
    v = confirm(load_v(a.registered), load_v(a.confirmation))
    md = ("# Harm confirmation\n\n" + (f"Confirmed: {', '.join(v['confirmed']) or 'none'}. "
          f"Unresolved: {', '.join(v['unresolved']) or 'none'}.\n" if v["valid"] else "Not admitted: a run is invalid.\n"))
    write(a.out, v, md)
    return 0 if v["valid"] and not v["confirmed"] else 1


if __name__ == "__main__":
    sys.exit(main())
