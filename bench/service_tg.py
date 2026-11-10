"""The guarded service default under the target-gated route (docs/STEWARDSHIP.md, "Target-gated
changes", conditions 1-7, enacted at 3e1a7ec3): the judge of its registration,
docs/journal/2026-11-10-service-guarded-target-gated-registration.md (DRAFT, not frozen).
Two parts, both fixed with the freeze:

    python3 bench/service_tg.py report ATTEMPT_DIR --out ATTEMPT_DIR/service-tg
    python3 bench/service_tg.py ci ARTIFACT_DIR --head SHA --out PREFIX
    python3 bench/service_tg.py confirm REGISTERED_DIR CONFIRMATION_DIR --head SHA --out PREFIX

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

**`ci`: the CI off-target noninferiority** (condition 2, and condition 7's CI criteria), judged
from the registered Benchmark run's artifact (`runs.jsonl` and `result.json`): the standing
end-to-end grid at the pinned bench revision (the 4-CPU runner × the seven corpora × 1, 2 and 4
threads × wall and CPU time: 42 cells), every cell's head-over-base paired geometric mean and
its one-sided 95% upper bound (t at 0.95, n - 1 degrees of freedom) at most 1.02; every cell
must pass. The run is admitted only as registered: mode `pr`, its head the one given (whose
only difference from the frozen candidate is the frozen file), its base the frozen base, the
pinned bench version, `ci_rounds` rounds, the runner's 4 CPUs, a runner CPU that is not the
target's (the predicate false there by its processor name alone), both arms complete, every
run successful, one output digest a corpus. Condition 7: a wall cell flagged slower (its
two-sided 95% interval above 1) is a harm signal, for `confirm`; peak RSS head over base with a
two-sided 95% lower bound above 1.10 fails.

**`confirm`: the inherited harm-confirmation rule**, on the one confirmation run allowed: a
wall cell flagged slower in the registered run and flagged slower again in the confirmation
run on the same head is a confirmed harm, and fails condition 7. A cell not flagged again is
unresolved, reported. The confirmation run never enters condition 2: it cannot rescue a
failed cell or replace the registered run.

None of the outcomes is a reason to retry: one attempt (condition 6).
"""

import argparse
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
from service_guarded import FETCHED, HASH, discovery, is_int, label, plan_of, positive  # noqa: E402,F401

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
# the CI grid at the pinned bench revision (.github/workflows/bench.yml, examples/ab.rs `gen`)
CI_CORPORA = ("onebyte", "pattern", "random", "records", "runs", "text", "wikidata")
CI_THREADS = (1, 2, 4)
CI_CPUS = 4
CI_ARMS = ("mibz2", "mibz2@base/ab")
CI_METRICS = ("wall", "cpu")
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
    toolchain and CI rounds (bench/service-tg.frozen) and the pinned bench version
    (bench/report.py's BENCH), from the checkout the judge runs in"""
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
        elif key == "ci_rounds" and re.fullmatch(r"[1-9][0-9]*", value):
            frozen[key] = int(value)
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
# CI: condition 2


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


def ci_judge(result, rows, unreadable=(), head=None, reg=None, check_git=True):
    """Condition 2 on the registered Benchmark run (see the module's doc)"""
    reg = reg or registered()
    fz = reg["frozen"]
    v = {"valid": False, "problems": [], "cells": {}, "flags": [], "rss_regression": [], "passes": False,
         "failing": []}
    p = v["problems"]
    for k in ("candidate", "base", "ci_rounds"):
        if k not in fz:
            p.append(f"bench/{FROZEN} names no {k}")
    if not isinstance(result, dict):
        p.append("result.json unreadable")
        return v
    if result.get("mode") != "pr":
        p.append(f"mode {result.get('mode')!r}, not a head-against-base run")
    if not head or result.get("head") != head:
        p.append(f"result.json's head {result.get('head')!r} is not the registered head {head!r}")
    elif check_git and "candidate" in fz:
        p += head_problems(head, fz["candidate"])
    if "base" in fz and result.get("base") != fz["base"]:
        p.append(f"result.json's base {result.get('base')!r} is not the frozen base {fz['base']}")
    if result.get("bench") != reg["bench"]:
        p.append(f"bench {result.get('bench')!r}, not the pinned {reg['bench']!r}")
    if "ci_rounds" in fz and result.get("rounds") != fz["ci_rounds"]:
        p.append(f"{result.get('rounds')!r} rounds, not the registered {fz['ci_rounds']}")
    if result.get("threads") != list(CI_THREADS):
        p.append(f"threads {result.get('threads')!r}, not {list(CI_THREADS)}")
    machine = result.get("machine") or {}
    if machine.get("cores") != CI_CPUS:
        p.append(f"a runner of {machine.get('cores')!r} CPUs, not the standing {CI_CPUS}")
    if not isinstance(machine.get("cpu"), str) or not machine["cpu"] or machine["cpu"].strip() == TARGET_NAME:
        p.append(f"the runner's CPU {machine.get('cpu')!r} is not shown off the target")
    if unreadable:
        p.append(f"runs.jsonl lines not JSON: {list(unreadable)[:5]}")
    n = fz.get("ci_rounds")
    by = {}
    for i, r in enumerate(rows):
        if not isinstance(r, dict):
            p.append(f"row {i}: not a JSON object")
            continue
        key = (r.get("corpus"), r.get("threads"), r.get("arm"))
        if key[0] not in CI_CORPORA or key[1] not in CI_THREADS or key[2] not in CI_ARMS or not is_int(key[1]):
            p.append(f"row {i}: a cell outside the registered grid {key}")
            continue
        q = []
        if r.get("termination") != "success":
            q.append(f"termination {r.get('termination')!r}")
        if r.get("err") != "":
            q.append(f"err {r.get('err')!r}")
        for k in ("wall", "cpu", "rss_kb"):
            if not positive(r.get(k)):
                q.append(f"{k} {r.get(k)!r}")
        if not is_int(r.get("round")):
            q.append(f"round {r.get('round')!r}")
        if q:
            p.append(f"row {i} {key}: " + ", ".join(q))
            continue
        by.setdefault(key, {})
        if r["round"] in by[key]:
            p.append(f"row {i} {key}: round {r['round']} repeated")
        by[key][r["round"]] = r
    for corpus in CI_CORPORA:
        hashes = {r.get("hash") for (c, _, _), rs in by.items() if c == corpus for r in rs.values()}
        if len(hashes) != 1:
            p.append(f"{corpus}: {len(hashes)} output digests, not one")
        for t in CI_THREADS:
            for arm in CI_ARMS:
                got = sorted(by.get((corpus, t, arm), {}))
                if n is not None and got != list(range(n)):
                    p.append(f"{corpus} at {t} threads, {arm}: rounds {len(got)}, not 0..{n - 1} once each")
    if p:
        return v
    v["valid"] = True
    for corpus in CI_CORPORA:
        for t in CI_THREADS:
            head_r, base_r = by[(corpus, t, CI_ARMS[0])], by[(corpus, t, CI_ARMS[1])]
            rounds = range(n)
            cell = {}
            for metric in CI_METRICS:
                logs = paired_logs(head_r, base_r, metric, rounds)
                u = upper(logs, 0.95)
                two = two_sided(logs, 0.95)
                cell[metric] = {"ratio": u["ratio"], "upper": u["upper"], "lo95": two["lo"], "hi95": two["hi"],
                                "pass": u["upper"] <= MARGIN}
                if not cell[metric]["pass"]:
                    v["failing"].append(f"{corpus} {t}t {metric} upper {u['upper']:.4f}")
                if metric == "wall" and two["lo"] > 1.0:
                    v["flags"].append(f"{corpus}/{t}")
            rss = two_sided(paired_logs(head_r, base_r, "rss_kb", rounds), 0.95)
            cell["rss"] = rss
            if rss["lo"] > RSS_MARGIN:
                v["rss_regression"].append(f"{corpus}/{t}")
            v["cells"][f"{corpus}/{t}"] = cell
    v["passes"] = not v["failing"] and not v["rss_regression"]
    return v


def ci_markdown(v):
    out = ["# The guarded service default under the target-gated route: CI condition 2", ""]
    if not v["valid"]:
        out += ["**Not admitted** (the run is not the registered one, or incomplete):"]
        out += [f"- {p}" for p in v["problems"]]
        return "\n".join(out) + "\n"
    out += [
        "Head over base, paired geometric mean over the rounds; the one-sided 95% upper bound at most "
        f"{MARGIN} at every one of the 42 cells (an intersection-union test at level 5%). Flags: a wall "
        "cell whose two-sided 95% interval lies above 1 (a harm signal, settled by `confirm`).",
        "",
        "| corpus | threads | wall | wall upper | CPU | CPU upper | peak RSS (95%) |",
        "|---|--:|--:|--:|--:|--:|--:|",
    ]
    for key, c in v["cells"].items():
        corpus, t = key.split("/")
        w, cp = c["wall"], c["cpu"]
        mark = lambda x: "" if x["pass"] else " **FAIL**"  # noqa: E731
        out.append(f"| {corpus} | {t} | {w['ratio']:.4f} | {w['upper']:.4f}{mark(w)} | {cp['ratio']:.4f} "
                   f"| {cp['upper']:.4f}{mark(cp)} | {fmt(c['rss'])} |")
    out += ["", f"**Condition 2:** {'**met** at every cell' if not v['failing'] else '**not met**: ' + '; '.join(v['failing'])}."]
    out += [f"**Wall flags (harm signals, one confirmation run):** {', '.join(v['flags']) or 'none'}."]
    out += [f"**Peak RSS regression (condition 7):** {', '.join(v['rss_regression']) or 'none'}."]
    return "\n".join(out) + "\n"


def confirm(registered_v, confirmation_v):
    """The inherited harm-confirmation rule on the one confirmation run: each wall flag of the
    registered run, flagged again (confirmed harm: fails condition 7) or not (unresolved)"""
    if not (registered_v["valid"] and confirmation_v["valid"]):
        return {"valid": False, "confirmed": [], "unresolved": []}
    again = set(confirmation_v["flags"])
    flags = registered_v["flags"]
    return {"valid": True, "confirmed": [f for f in flags if f in again],
            "unresolved": [f for f in flags if f not in again]}


def load_artifact(d):
    try:
        with open(os.path.join(d, "result.json")) as f:
            result = json.load(f)
    except (OSError, json.JSONDecodeError):
        result = None
    rows, unreadable = read_jsonl(os.path.join(d, "runs.jsonl"))
    return result, rows, unreadable


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
    c = sub.add_parser("ci")
    c.add_argument("artifact")
    c.add_argument("--head", required=True)
    c.add_argument("--out", required=True)
    f = sub.add_parser("confirm")
    f.add_argument("registered")
    f.add_argument("confirmation")
    f.add_argument("--head", required=True)
    f.add_argument("--out", required=True)
    a = p.parse_args(argv)
    if a.cmd == "report":
        files, rows, unreadable = load(a.attempt)
        v = judge(files, rows, unreadable)
        write(a.out, v, markdown(v))
        return 0 if v["valid"] else 1
    if a.cmd == "ci":
        v = ci_judge(*load_artifact(a.artifact), head=a.head)
        write(a.out, v, ci_markdown(v))
        return 0 if v["valid"] and v["passes"] else 1 if v["valid"] else 2
    first = ci_judge(*load_artifact(a.registered), head=a.head)
    second = ci_judge(*load_artifact(a.confirmation), head=a.head)
    v = confirm(first, second)
    md = ("# Harm confirmation\n\n" + (f"Confirmed: {', '.join(v['confirmed']) or 'none'}. "
          f"Unresolved: {', '.join(v['unresolved']) or 'none'}.\n" if v["valid"] else "Not admitted: a run is invalid.\n"))
    write(a.out, v, md)
    return 0 if v["valid"] and not v["confirmed"] else 1


if __name__ == "__main__":
    sys.exit(main())
