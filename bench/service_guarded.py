"""The guarded service default's workstation A/B (docs/journal/2026-11-09-service-guarded-registration.md,
DRAFT, not frozen): the candidate build's default decoder (`BzReader::parallel`, which runs the
service core, svc2, at exactly 10 threads on its target) against the current default's build
(the base), through the same public entry point (`ab`'s `mibz2` arm), on the bench's
Wikidata sample, paired by round. It judges an attempt's directory, as the recipe leaves it
(`just bench-service-guarded`), not a bare runs file:

    python3 bench/service_guarded.py report ATTEMPT_DIR --out ATTEMPT_DIR/service-guarded

**What it admits, and how** (the earlier service-2x3 report's evidence boundary, at
3e201246's bench/service_2x3.py, restored; the Researcher's REVIEW 6742157313):
- **identity:** `commits.txt`'s candidate and base are `bench/service-guarded.frozen`'s, and
  the base is an ancestor of the candidate; no `DRY-RUN.txt`; `arms.txt` names exactly the
  three arms the recipe ran (`mibz2`, `mibz2@<attempt>/bin/ab-base`, `mibz2@<attempt>/bin/ab-base-twin`)
  and `inputs.txt` gives a SHA-256 for both base paths, the same digest;
- **workload:** `corpus.sha256` is the registered corpus's (`bench/wikidata.json`); every run
  is the registered corpus (`wikidata`), its output exactly the registered decoded bytes
  (209700196), its digest a hex string, one digest across every run;
- **rows:** every row a JSON object with the arm one of `arms.txt`'s; `threads` and `round`
  integers (never booleans); `err` present and empty; no `broken` (ab's mark for a child that
  printed no record: a crash), and no `exit`/`status` other than 0 where a row carries one;
  finite positive wall, CPU and peak RSS; both thread records;
- **the preflight:** `preflight.json` and its exit status (`preflight.exit`) checked whole by
  bench/svc_target_check.py: TARGET, not merely `ok`;
- **the plans:** every run's plan as expected, from both thread records.

**The grid** is the pipelined default's authorised one, exactly (8, 9, 10, 11, 12, 16 and 20
threads; docs/journal/2026-11-07-pipelined-default-registration.md), with its rules for the
budgets where neither build changes plan: identity checks at 8 and 9 (reported, the ±2%
flag), no harm at 11, 12, 16 and 20 (each head-over-base wall ratio's one-sided upper bound,
Bonferroni over the four, at most 1.02; judged, and reported beside P1).

These are checks of the archived records. They do not prove what the binaries were built from
(the recipe's refusals do that), that the window or kernels are as the plan implies (that
follows only from the pinned dispatch code), or that equal plans at the other budgets mean
equal binaries there (no harm is its own judged predicate).

**The rule** (the service-core registrations', unchanged where it applies): P1 at 10 threads,
speed (base wall over head wall) and CPU (base CPU over head CPU), each the geometric mean of
the 20 rounds' paired ratios with its two-sided 95% t-interval, co-primary, each by the
outcome map. N1 (the base against its byte-identical twin at 10) both bounds within
[0.97, 1.03], a validity condition; the twins at the other budgets reported. Peak RSS (each
run its own process) reported, not judged.

A failed attempt says which kind of failure it is (`failure`): `identity`, `workload`,
`preflight`, `structural` (rows, rounds, plans) or `n1`. None is a reason to retry.
"""

import argparse
import json
import math
import os
import re
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import svc_target_check  # noqa: E402

ROUNDS = 20
# Student's t at 19 degrees of freedom: the two-sided 95% quantile (scipy.stats.t.ppf(0.975, 19))
T_975 = 2.0930
# the one-sided quantile for a familywise 0.05 over the four no-harm cells (Bonferroni, each at
# 1 - 0.05/4), as the pipelined default's registration has it
T_9875 = 2.4334
# the pipelined default's authorised grid (docs/journal/2026-11-07-pipelined-default-registration.md),
# carried over exactly: identity checks at 8 and 9 (reported, not judged), the primary at 10,
# no harm at 11, 12, 16 and 20 (its rule)
CONTROLS = (8, 9)
PRIMARY = 10
NO_HARM = (11, 12, 16, 20)
NO_HARM_MAX = 1.02
THREADS = CONTROLS + (PRIMARY,) + NO_HARM
# plans as (leaders, plain helpers, pipelined helpers, service workers)
FOUR_BY_TWO = (4, 4, 0, 0)
PIPED_3X3 = (3, 0, 6, 0)
SVC2 = (2, 4, 0, 2)
EXPECT = {
    t: (
        {"base": FOUR_BY_TWO, "head": FOUR_BY_TWO, "twin": FOUR_BY_TWO}
        if t in CONTROLS
        else {"base": PIPED_3X3, "head": SVC2 if t == PRIMARY else PIPED_3X3, "twin": PIPED_3X3}
    )
    for t in THREADS
}
CONTROL_BAND = (0.98, 1.02)
N1_BAND = (0.97, 1.03)
CORPUS = "wikidata"
HASH = re.compile(r"[0-9a-f]{16,}")
BASE_SUFFIX, TWIN_SUFFIX = "/bin/ab-base", "/bin/ab-base-twin"


def registered(root=None):
    """The registered workload (bench/wikidata.json) and the frozen commits
    (bench/service-guarded.frozen), from the checkout the judge runs in"""
    root = root or os.path.dirname(HERE)
    with open(os.path.join(root, "bench", "wikidata.json")) as f:
        w = json.load(f)
    with open(os.path.join(root, "bench", "service-guarded.frozen")) as f:
        lines = f.read().splitlines()
    frozen = {}
    for line in lines:
        f = line.split()
        if len(f) == 2 and f[0] in ("candidate", "base") and re.fullmatch(r"[0-9a-f]{40}", f[1]):
            frozen[f[0]] = f[1]
    return {"corpus_sha256": w["corpus_sha256"], "decoded_bytes": w["decoded_bytes"], "frozen": frozen}


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


def paired(ref, arm, key):
    """The paired geometric mean of ref[key]/arm[key] over the rounds, and its 95% interval"""
    logs = [math.log(ref[i][key] / arm[i][key]) for i in range(ROUNDS)]
    m, se = statistics.fmean(logs), statistics.stdev(logs) / math.sqrt(ROUNDS)
    return {"ratio": math.exp(m), "lo": math.exp(m - T_975 * se), "hi": math.exp(m + T_975 * se)}


def outcome(r, cpu=False):
    """The service-core registrations' outcome map, unchanged, on a ratio's 95% interval"""
    lo, hi = r["lo"], r["hi"]
    if lo > 1.02:
        return "CPU SAVING" if cpu else "GAIN"
    if lo > 1.0:
        return "SMALL CPU SAVING" if cpu else "SMALL GAIN"
    if lo >= 0.97 and hi <= 1.03 and lo <= 1.0 <= hi:
        return "NULL"
    if hi < 1.0:
        return "CPU COST" if cpu else "REGRESSION"
    return "INCONCLUSIVE"


def reading(speed, cpu):
    """P1's registered reading, from its two outcomes"""
    gain = speed in ("GAIN", "SMALL GAIN")
    if gain and cpu != "CPU COST":
        return "THE SERVICE DEFAULT BEATS THE CURRENT DEFAULT"
    if gain:
        return "A TRADE"
    if speed in ("NULL", "REGRESSION"):
        return "NOT REPLICATED"
    return "INCONCLUSIVE"


# the qualifications each label carries in the report (the earlier service-core reports')
QUALIFY = {
    "INCONCLUSIVE": "the interval spans gain and harm or is too wide: nothing detected either way",
    "NULL": "within ±3% with 1 inside: a small effect is not excluded, and no effect is not shown",
    "CPU INCONCLUSIVE": "no CPU cost detected, not shown to cost nothing",
    "CPU NULL": "within ±3%: a small CPU difference is not excluded",
}


def bundle_problems(files, reg):
    """(kind, problem) for the attempt's identity and workload files"""
    p = []
    if "DRY-RUN.txt" in files:
        p.append(("identity", "a DRY run: never evidence"))
    commits = dict(l.split()[:2] for l in files.get("commits.txt", "").splitlines() if len(l.split()) >= 2)
    for k in ("candidate", "base"):
        if not reg["frozen"].get(k):
            p.append(("identity", f"bench/service-guarded.frozen names no {k}"))
        elif commits.get(k) != reg["frozen"][k]:
            p.append(("identity", f"commits.txt's {k} {commits.get(k)!r} is not the frozen {reg['frozen'][k]}"))
    arms = [a for a in files.get("arms.txt", "").splitlines() if a.strip()]
    if not (
        len(arms) == 3
        and arms[0] == "mibz2"
        and arms[1].startswith("mibz2@/")
        and arms[1].endswith(BASE_SUFFIX)
        and arms[2] == arms[1] + "-twin"
    ):
        p.append(("identity", f"arms.txt is not the registered three arms: {arms}"))
        arms = []
    digests = {}
    for l in files.get("inputs.txt", "").splitlines():
        f = l.split(None, 1)
        if len(f) == 2 and re.fullmatch(r"[0-9a-f]{64}", f[0]):
            digests[f[1].strip()] = f[0]
    if arms:
        base, twin = arms[1][len("mibz2@"):], arms[2][len("mibz2@"):]
        if base not in digests or twin not in digests:
            p.append(("identity", "inputs.txt gives no digest for the base and twin executables run"))
        elif digests[base] != digests[twin]:
            p.append(("identity", "the twin is not the base's bytes"))
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
    """The attempt's verdict. `files`: the attempt's small files by name (commits.txt,
    arms.txt, inputs.txt, corpus.sha256, preflight.json, preflight.exit, DRY-RUN.txt if
    present); `rows`: runs.jsonl's parsed lines; `unreadable`: its lines that weren't JSON"""
    reg = reg or registered()
    v = {"valid": False, "failure": None, "problems": [], "cells": {}, "n1": None, "p1": None,
         "no_harm": None, "preflight": None}

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
    build = {arms[0]: "head", arms[1]: "base", arms[2]: "twin"}
    by = {}
    for r in rows:
        by.setdefault((build[r["arm"]], r["threads"]), []).append(r)
    if len({r["hash"] for r in rows}) != 1:
        fail("structural", ["more than one output digest"])
    for t in THREADS:
        runs = {b: by.get((b, t), []) for b in ("base", "head", "twin")}
        complete = all(sorted(r["round"] for r in runs[b]) == list(range(ROUNDS)) for b in runs)
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
                f"{t} threads: ran base {label(seen['base'])}, head {label(seen['head'])}, "
                f"twin {label(seen['twin'])}; expected base {label([EXPECT[t]['base']])}, "
                f"head {label([EXPECT[t]['head']])}"
            ])
        if complete:
            base = {r["round"]: r for r in runs["base"]}
            head = {r["round"]: r for r in runs["head"]}
            twin = {r["round"]: r for r in runs["twin"]}
            c["speed"] = paired(base, head, "wall")
            c["cpu"] = paired(base, head, "cpu")
            # (head over base: above 1 is more memory)
            c["rss"] = paired(head, base, "rss_kb")
            c["twin_speed"] = paired(base, twin, "wall")
            # the head's wall over the base's, its one-sided Bonferroni upper bound (the
            # pipelined default's no-harm rule): from the speed's mean and standard error
            sp = c["speed"]
            se = math.log(sp["hi"] / sp["ratio"]) / T_975
            c["harm_upper"] = math.exp(-math.log(sp["ratio"]) + T_9875 * se)
            c["median"] = {
                b: {k: statistics.median(r[k] for r in runs[b]) for k in ("wall", "cpu", "rss_kb")}
                for b in runs
            }
        v["cells"][t] = c
    for t in CONTROLS:
        s = v["cells"][t].get("speed")
        v["cells"][t]["flag"] = s is not None and (s["lo"] > CONTROL_BAND[1] or s["hi"] < CONTROL_BAND[0])
    if v["failure"]:
        return v
    n1 = v["cells"][PRIMARY]["twin_speed"]
    v["n1"] = n1
    if not (N1_BAND[0] <= n1["lo"] and n1["hi"] <= N1_BAND[1]):
        fail("n1", [f"N1 at 10 threads: {n1['ratio']:.3f} [{n1['lo']:.3f}, {n1['hi']:.3f}] not within [0.97, 1.03]"])
        return v
    c = v["cells"][PRIMARY]
    so, co = outcome(c["speed"]), outcome(c["cpu"], cpu=True)
    v["p1"] = {"speed": so, "cpu": co, "reading": reading(so, co)}
    # no harm at 11, 12, 16 and 20, as the pipelined default's rule: each upper bound at most
    # 1.02; an interval too wide to show it is not shown
    v["no_harm"] = {t: v["cells"][t]["harm_upper"] <= NO_HARM_MAX for t in NO_HARM}
    v["valid"] = True
    return v


def fmt(r):
    return f"{r['ratio']:.3f} [{r['lo']:.3f}, {r['hi']:.3f}]" if r else "–"


def markdown(v):
    out = [
        "# The guarded service default against the current default",
        "",
        "Speed and CPU are the base's over the head's (above 1: the head is faster, or spends "
        "less); peak RSS is the head's over the base's (above 1: more memory), each run its own "
        "process. Paired geometric means over 20 rounds, 95% t-intervals.",
        "",
        "| threads | role | plans, base → head | speed | head/base wall, upper bound | CPU | peak RSS (reported) | twin over base |",
        "|--:|---|---|--:|--:|--:|--:|--:|",
    ]
    for t in THREADS:
        c = v["cells"].get(t, {})
        role = (
            "P1 (judged); N1 (validity)"
            if t == PRIMARY
            else "identity (reported)" if t in CONTROLS else "no harm (judged)"
        )
        seen = c.get("seen", {})
        plans = f"{label(seen.get('base', []))} → {label(seen.get('head', []))}"
        flag = " ±2% FLAG" if c.get("flag") else ""
        upper = f"{c['harm_upper']:.3f}" if "harm_upper" in c else "–"
        out.append(
            f"| {t} | {role} | {plans} | {fmt(c.get('speed'))}{flag} | {upper} "
            f"| {fmt(c.get('cpu'))} | {fmt(c.get('rss'))} | {fmt(c.get('twin_speed'))} |"
        )
    pf = v.get("preflight") or {}
    out += ["", f"**The preflight:** {pf.get('verdict', '–')}."]
    out += ["", f"**Valid:** {'yes' if v['valid'] else 'no' + (' (' + v['failure'] + ')' if v['failure'] else '')}"]
    out += [f"- {p}" for p in v["problems"]]
    if v["p1"]:
        s, c = v["p1"]["speed"], v["p1"]["cpu"]
        c10 = v["cells"][PRIMARY]
        out += [
            "",
            f"**P1:** speed {s} {fmt(c10['speed'])}; CPU {c} {fmt(c10['cpu'])}: **{v['p1']['reading']}**.",
        ]
        nh = v["no_harm"] or {}
        failed = [str(t) for t, ok in nh.items() if not ok]
        out.append(
            "**No harm at 11, 12, 16 and 20** (each head/base wall upper bound at most 1.02, "
            "Bonferroni over the four): "
            + ("**shown at every one**." if not failed else f"**not shown at {', '.join(failed)}** "
               "(an interval too wide to show it counts as not shown).")
        )
        if s in QUALIFY:
            out.append(f"- Speed {s}: {QUALIFY[s]}.")
        if f"CPU {c}" in QUALIFY:
            out.append(f"- CPU {c}: {QUALIFY['CPU ' + c]}.")
        out += [
            "- Speed and CPU are co-primary, each with its own 95% interval: together they are not "
            "a simultaneous 95% statement.",
            "- A product gain does not identify the service threads as its cause: svc2 against the "
            "current default changes the plan, the teams, the window and the decoding threads together.",
            "- The plans are the sampled thread counts, not a proof of the window or the kernels; "
            "equal plans at the other budgets are a fallback check, not binary identity; no harm "
            "there is its own predicate, above.",
            "- Peak RSS is reported, not judged: no memory condition was registered, and none is met here.",
            "- One i9 result does not qualify the predicate's class, and CI's off-path bench does not "
            "qualify the service default's gain.",
        ]
    return "\n".join(out) + "\n"


FILES = ("commits.txt", "arms.txt", "inputs.txt", "corpus.sha256", "preflight.json", "preflight.exit", "DRY-RUN.txt")


def load(attempt):
    files = {}
    for n in FILES:
        try:
            with open(os.path.join(attempt, n), errors="replace") as f:
                files[n] = f.read()
        except OSError:
            pass
    rows, unreadable = [], []
    try:
        with open(os.path.join(attempt, "runs.jsonl"), errors="replace") as f:
            lines = f.read().splitlines()
    except OSError:
        lines, unreadable = [], [0]
    for n, line in enumerate(lines, 1):
        if line.strip():
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError:
                unreadable.append(n)
    return files, rows, unreadable


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    r.add_argument("attempt")
    r.add_argument("--out", required=True)
    a = p.parse_args(argv)
    files, rows, unreadable = load(a.attempt)
    v = judge(files, rows, unreadable)
    with open(a.out + ".json", "w") as f:
        json.dump(v, f, indent=1, sort_keys=True)
    md = markdown(v)
    with open(a.out + ".md", "w") as f:
        f.write(md)
    print(md)
    return 0 if v["valid"] else 1


if __name__ == "__main__":
    sys.exit(main())
