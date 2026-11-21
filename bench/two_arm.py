#!/usr/bin/env python3
"""A two-arm adoption run's verdict, read from the run's records only after they are checked
whole: a candidate against a reference on a main corpus (the gain family) and a guard corpus,
at judged thread counts, with control counts where the two arms build the same decoder.
Standard library only. Generalised from the stop-and-walk adjudicator (the Researcher's
correction, #145 REVIEW 6921090225), for `just bench-cores-less-one` (#481) and after.

    two_arm.py RUNS.jsonl --ref ARM --cand ARM [--cand ARM ...] --main CORPUS --guard CORPUS[,CORPUS..]
               --judged 12,16,20 --controls 9,10 --env ENV.json [--rounds 10]
               [--cores 10 --cpus 20] [--hold-max-threads 9] [--context ARM ...]
               [--judge-at ARM=COUNTS ... [--sparse] [--pair ARM/ARM ...]]
               [--ref-at ARM=COUNTS ... in place of --ref] [--unpinned]
               [--alpha 0.05] [--primary CORPUS,..] [--check-workers]

**Guards** (`--guard`, one guard corpus or several, comma-separated; `just bench-admit`'s and
`just bench-recalibrate`'s text and random): each is screened for loss and memory as the one
guard is. With one, nothing here changes.

**The family's α** (`--alpha`, 0.05 by default, and then nothing here changes; `just
bench-recalibrate`): Holm's family α. A run whose cells are judged in two calls of this script
(pinned and unpinned, whose records share keys) gives each call half of 0.05, so the two
families together keep 0.05 (Bonferroni over the two Holm families).

**Memory traded for speed** (`--primary CORPUS,..`, with `--judge-at` only; none by default, and
then nothing here changes; PERFORMANCE-POLICY rule 4 as amended by #558, `just
bench-recalibrate`): a candidate with a gain (Holm) and no judged cell SLOWER, whose peak memory
is above the gate on one corpus, ADOPTS BY TRADE where, at that count, that one corpus is a
primary corpus (`--primary`), with a wall gain there of at least 5% beyond noise (its step FASTER
and its geometric mean at most 0.95), memory growth in percent at most twice that gain, and growth
of at most 64 MB (62,500 KiB) of peak RSS (medians). Every other corpus stays within the gate:
#558 lets no other corpus grow past 10%, so two corpora above the gate never trade, however each
qualifies alone (the Researcher's REVIEW 6978182129). Otherwise it is HELD ON MEMORY, the
Coordinator's to weigh, as before. An arm that ADOPTS BY TRADE counts as one that ADOPTS: of those
at a count, the choice is the lowest main-corpus step, as before.

**Rounds by count** (`--rounds-at T=N`, repeated, with `--sparse` only; none by default, and
then nothing here changes; `just bench-recalibrate`'s pinned 8 threads, the cell the 0.5.5
crossover put nearest the line): N paired rounds at that judged count in place of `--rounds`.
Its roster and its steps are read over its own N rounds; Holm's family is unchanged.

**The workers, checked** (`--check-workers`; off by default, and then nothing here changes;
`just bench-recalibrate`): every record of an occupancy arm (`mibz2-occ-N-S`, `mibz2-oce-N-S`)
must name exactly N worker threads (`mibz2-0` to `mibz2-(N-1)`) among its `observed_threads`,
or the run is VOID: the arm measured is the arm named.

**Several candidates** (`--cand` repeated; `just bench-workers-window`): each is read against the one
reference as below, but Holm's family is every candidate's main-corpus judged cells together, and
each candidate gets its own verdict. Where more than one ADOPTS, the one named is the one with the
lowest geometric mean of its main-corpus steps. With one candidate, all of this is as before.

**The reference** is whichever arm `--ref` names: production's `mibz2` in `bench-workers-window`,
an occupancy arm (`mibz2-occ-8-32`) in `just bench-workers-low`. Every ratio, the memory gate and
the choice are against it; where none ADOPTS, it is the reference that stays.

**Context arms** (`--context`, repeated; none by default, and then nothing here changes): arms
run beside the others and checked with them (the roster, the records, the pinning, the
spinner), whose steps over the reference are reported but not judged. They enter no Holm family,
no loss screen, no memory gate, no verdict and no choice (`bench-workers-low`'s production
`mibz2`, after #510 no longer the reference).

**Candidates judged by count** (`--judge-at ARM=T[,T...]`, repeated, one for every candidate;
none by default, and then nothing here changes; `just bench-workers-at-cores`): every arm runs
at every judged count, as `ab run` crosses them, but a candidate is judged only at the counts
its `--judge-at` names, and each count is read on its own, since production's plan is chosen
count by count. At each judged count, each candidate judged there has its own verdict, read
from its two cells at that count alone (its main-corpus step for the gain, both corpora for
the loss screen and the memory gate), and the choice is made among those that ADOPT there, by
the lowest step (the geometric mean of the paired ratios) on the main corpus. Holm's family is
still one: every candidate's main-corpus cell at every count it is judged at, together. An arm's
cell at a count where it is not judged (a candidate's elsewhere, and every context arm's) is
reported beside them, not judged. A `--judge-at` arm that is not a candidate, a count that is not
judged, a candidate without one, or a judged count with no candidate is VOID.

**A sparse roster** (`--sparse`, with `--judge-at` and no control counts only; off by default,
and then nothing here changes; `just bench-workers-sweep-low`): the arms are not crossed with the
counts. At each judged count the roster is the reference, the candidates judged there and every
context arm, and no other arm: a candidate's record at a count it is not judged at is not in the
roster (VOID), and the report has no cell for it there. The run makes one `ab run` a count, each
with that count's arms alone, so no time goes on cells that judge nothing (2 workers at 8
threads, say).

**Pairs, reported and not judged** (`--pair A/B`, repeated, with `--judge-at` only; none by
default, and then nothing here changes; `just bench-eager-trim`): at each judged count where both
arms are in the roster, A's paired rounds' wall ratios over B's (geometric mean and 95% interval)
and A's peak memory over B's, in both corpora. A pair enters no Holm family, no loss screen, no
memory gate, no verdict and no choice. A pair naming an arm that is not the reference, a
candidate or a context arm, naming one arm twice, or present at no judged count is VOID.

**A reference by count** (`--ref-at ARM=T[,T...]`, repeated, in place of `--ref`; with `--judge-at`
and `--sparse` only; none by default, and then nothing here changes; `just bench-unpinned`): each
judged count has its own reference arm, named by exactly one `--ref-at`, and every ratio, the
memory gate and the choice at that count are against it. At 10 threads lazy 10/40, at 6 eager 6/24,
say: the untrimmed plan at each count, as explicit arms. A reference arm may be a candidate at
another count (`just bench-recalibrate`: lazy 6/24 the shipped trim at 8 threads and one worker
fewer at 10). A reference arm judged as a candidate at its own count, a context arm that is also
a reference or a candidate, a judged count with none or two, or a count that is not judged is
VOID.

**Unpinned runs** (`--unpinned`; off by default, and then nothing here changes; `just
bench-unpinned`, `ab run --unpinned`): every record, at every count, must have run on every CPU of
the registered profile: its `cpus` a list naming each of the `--cpus` CPUs once (not `all`, which
says only that the run inherited the harness's affinity), and no spinner (`--hold-max-threads 0`
required: an unpinned decode may run on any CPU, so no core is free to hold).

**Arms.** A record's arm is canonical: `NAME` for this build's harness, and `NAME@build-X`
for another build's (`ab run`'s `NAME@EXE` arm, EXE under a directory `build-X`).

**The rule** (frozen per run, before its data; `bench-cores-less-one`'s is #481's adoption
registration on #145).
- *A cell's step:* the rounds' paired wall ratios, candidate over reference: their geometric
  mean and its 95% interval (Student's t on the logs). FASTER if the interval lies below 1,
  SLOWER if above, else INCONCLUSIVE (not equal).
- *Gain:* Holm's one-sided tests (the mean log ratio below 0) over the main corpus's judged
  cells, family α = 0.05. The gain is established if Holm rejects at one cell or more.
- *Loss screen:* no judged cell, main or guard, is SLOWER. (A noisy loss can still pass a
  screen that finds only the losses its intervals resolve.)
- *Memory gate:* at every judged cell, main and guard, the median over the rounds of each run's
  peak resident memory, candidate over reference, is at most 1.10.
- *Controls:* at a control count the two arms build the same decoder. A control cell, main or
  guard, FASTER or SLOWER means the harness, not the change, moved: the run is VOID.
- **ADOPTS:** gain established, loss screen clean, memory gate passed.
- **HELD ON MEMORY:** gain established and loss screen clean, memory gate failed: the
  Coordinator weighs it under PERFORMANCE-POLICY rule 4.
- **A FINDING, NOT ADOPTED:** gain established and a judged cell SLOWER.
- **NOT ADOPTED:** otherwise.
- **VOID** (exit 1, no verdict): the file missing or unreadable; any record not a successful
  run with finite positive wall, CPU and peak memory; any key (corpus, arm, threads, round)
  duplicated, unexpected or missing from the full roster; a corpus whose outputs (size and
  hash) differ; a run beside a spinner sharing a decoder core; a control cell that moved.
- **VOID too, on the machine's and the runs' metadata** (the Researcher's correction, #145
  REVIEW 6924511372: an absent overlap flag is no proof of a free core):
  - *the topology* (`ab env`'s record, `--env`): missing or malformed, or not the profile
    registered: `--cores` physical cores and `--cpus` CPUs in the affinity, each core's CPUs
    listed and together all of them, and no CPU quota;
  - *the pinning:* a record's `cpus` missing or malformed; at a count up to the cores, not one
    CPU on each of that many distinct cores (`ab run --spread`); past the cores, not `all`; with
    `--unpinned`, at any count, not every registered CPU;
  - *the spinner:* at a count up to `--hold-max-threads`, a record's `held_spinner` missing or
    malformed (its `cpu`, `overlap`, `decoder_cpus` and `owner`), its `decoder_cpus` not the
    record's own, or its CPU on a core of the decode's; past it, a spinner recorded.
"""
import argparse
import json
import math
import os
import re
import statistics
import sys

ALPHA, RSS_GATE = 0.05, 1.10
# rule 4's trade (#558): the gain at least 5%, the growth at most twice it and 64 MB (in KiB)
TRADE_GAIN, TRADE_RATIO, TRADE_KIB = 0.95, 2.0, 62_500
# Student's t, two-sided 95%, by degrees of freedom (1..30); 1.96 beyond
T95 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
       2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
       2.056, 2.052, 2.048, 2.045, 2.042]


def canonical(arm):
    """`NAME`, or `NAME@build-X` for another build's harness"""
    name, at, exe = str(arm).partition("@")
    if not at:
        return name
    m = re.search(r"(build-[^/]+)/", exe)
    return f"{name}@{m.group(1) if m else exe}"


def betainc(a, b, x):
    """The regularized incomplete beta function I_x(a, b) (a continued fraction, Lentz)"""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    if x > (a + 1) / (a + b + 2):
        return 1.0 - betainc(b, a, 1.0 - x)
    lfront = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log1p(-x)
    tiny = 1e-300
    c, d = 1.0, 1.0 - (a + b) * x / (a + 1)
    d = 1.0 / (d if abs(d) > tiny else tiny)
    f = d
    for m in range(1, 400):
        for num in (m * (b - m) * x / ((a + 2 * m - 1) * (a + 2 * m)),
                    -(a + m) * (a + b + m) * x / ((a + 2 * m) * (a + 2 * m + 1))):
            d = 1.0 + num * d
            d = 1.0 / (d if abs(d) > tiny else tiny)
            c = 1.0 + num / c
            c = c if abs(c) > tiny else tiny
            f *= c * d
        if abs(c * d - 1.0) < 1e-15:
            break
    return math.exp(lfront) * f / a


def t_cdf(t, df):
    """P(T <= t), T Student's t with df degrees of freedom"""
    if math.isinf(t):
        return 0.0 if t < 0 else 1.0
    tail = 0.5 * betainc(df / 2, 0.5, df / (df + t * t))
    return tail if t < 0 else 1.0 - tail


def step(logs):
    """(geometric mean, 95% low, 95% high, one-sided p that the mean log is below 0)"""
    n = len(logs)
    mu = statistics.mean(logs)
    sd = statistics.stdev(logs)
    t95 = T95[n - 2] if n - 1 <= len(T95) else 1.96
    h = t95 * sd / math.sqrt(n)
    p = (0.0 if mu < 0 else 1.0) if sd == 0 else t_cdf(mu / (sd / math.sqrt(n)), n - 1)
    return math.exp(mu), math.exp(mu - h), math.exp(mu + h), p


def label(lo, hi):
    return "FASTER" if hi < 1 else "SLOWER" if lo > 1 else "INCONCLUSIVE"


def holm(ps, alpha):
    """Which of `ps` (a dict) Holm's step-down procedure rejects at family `alpha`"""
    rejected, m = set(), len(ps)
    for i, (key, p) in enumerate(sorted(ps.items(), key=lambda kv: kv[1])):
        if p > alpha / (m - i):
            break
        rejected.add(key)
    return rejected


def cpu_list(s):
    """A comma-separated list of CPU numbers, or None if it is not one"""
    if not isinstance(s, str) or not s:
        return None
    try:
        cpus = [int(x) for x in s.split(",")]
    except ValueError:
        return None
    return cpus if all(c >= 0 for c in cpus) else None


def topology(path, cores, cpus, errors):
    """The CPU each core of `ab env`'s record holds ({cpu: core}), after checking it is the
    registered profile; None if it is not (errors appended)"""
    name = os.path.basename(path)
    try:
        env = json.load(open(path))
    except (OSError, ValueError) as e:
        errors.append(f"{name}: no machine record ({e})")
        return None
    if not isinstance(env, dict):
        errors.append(f"{name}: not a machine record")
        return None
    bad = []
    if env.get("cores") != cores or isinstance(env.get("cores"), bool):
        bad.append(f"cores {env.get('cores')!r}, not {cores}")
    if env.get("cpus") != cpus or isinstance(env.get("cpus"), bool):
        bad.append(f"cpus {env.get('cpus')!r}, not {cpus}")
    if "cpu_quota" not in env or env["cpu_quota"] is not None:
        bad.append(f"cpu_quota {env.get('cpu_quota', 'missing')!r}, not none")
    core_of = {}
    cc = env.get("core_cpus")
    if not isinstance(cc, list) or len(cc) != cores:
        bad.append(f"core_cpus {cc!r}: not {cores} cores")
    else:
        for i, s in enumerate(cc):
            lst = cpu_list(s)
            if lst is None:
                bad.append(f"core_cpus[{i}] {s!r} malformed")
                continue
            for c in lst:
                if c in core_of:
                    bad.append(f"CPU {c} on two cores")
                core_of[c] = i
        if not bad and sorted(core_of) != list(range(cpus)):
            bad.append(f"the cores' CPUs are not CPUs 0 to {cpus - 1}")
    if bad:
        errors.append(f"{name}: not the registered profile: {'; '.join(bad)}")
        return None
    return core_of


def check_metadata(where, key, r, core_of, cores, hold_max, errors, unpinned=False):
    """A record's pinning and spinner, against the machine (`core_of`) and the hold limit;
    `unpinned`: every registered CPU at every count (`ab run --unpinned`)"""
    t, s = key[2], r.get("cpus")
    if unpinned:
        lst = cpu_list(s)
        if lst is None:
            errors.append(f"{where}: {key} its CPUs {s!r} missing or malformed, not every CPU listed")
        elif sorted(lst) != sorted(core_of) or len(set(lst)) != len(lst):
            errors.append(f"{where}: {key} its CPUs {s!r} are not every one of the {len(core_of)} CPUs (unpinned)")
    elif t > cores:
        if s != "all":
            errors.append(f"{where}: {key} pinned {s!r} past the {cores} cores, not unpinned ('all')")
    else:
        lst = cpu_list(s)
        if lst is None:
            errors.append(f"{where}: {key} its CPUs {s!r} missing or malformed")
        elif len(lst) != t or len(set(lst)) != t or any(c not in core_of for c in lst) \
                or len({core_of[c] for c in lst}) != t:
            errors.append(f"{where}: {key} its CPUs {s!r} are not one on each of {t} cores")
    hs = r.get("held_spinner")
    if t > hold_max:
        if hs is not None:
            errors.append(f"{where}: {key} a spinner recorded past the hold limit ({hold_max} threads)")
        return
    if not isinstance(hs, dict):
        errors.append(f"{where}: {key} no spinner recorded ({hs!r})")
        return
    cpu, overlap, dec, owner = hs.get("cpu"), hs.get("overlap"), hs.get("decoder_cpus"), hs.get("owner")
    if isinstance(cpu, bool) or not isinstance(cpu, int) or cpu not in core_of:
        errors.append(f"{where}: {key} the spinner's CPU {cpu!r} malformed")
        return
    if not isinstance(overlap, bool):
        errors.append(f"{where}: {key} the spinner's overlap {overlap!r} malformed")
    if owner != "run":
        errors.append(f"{where}: {key} the spinner's owner {owner!r}, not the run")
    if dec != s:
        errors.append(f"{where}: {key} the spinner's decoder CPUs {dec!r} are not the run's {s!r}")
    lst = cpu_list(s) or []
    if overlap is not False or any(core_of.get(c) == core_of[cpu] for c in lst):
        errors.append(f"{where}: {key} ran beside a spinner sharing a decoder core (CPU {cpu})")


def occupancy_workers(arm):
    """An occupancy arm's workers (`mibz2-occ-N-S`, `mibz2-oce-N-S`), else None"""
    m = re.fullmatch(r"mibz2-oc[ce]-(\d+)-(\d+)", arm)
    return int(m.group(1)) if m else None


def worker_names(r):
    """The worker threads a record observed (`mibz2-0`, `mibz2-1`, ...), as a sorted list"""
    names = [p.partition("=")[0] for p in str(r.get("observed_threads") or "").split()]
    return sorted(n for n in names if re.fullmatch(r"mibz2-\d+", n))


def load(path, roster, errors, meta=None, check_workers=False):
    """The file's records by key (corpus, canonical arm, threads, round), after checking each
    record and the roster; errors appended. `meta`: (core_of, cores, hold_max[, unpinned]) to
    check each record's pinning and spinner against"""
    name, rows, outputs = os.path.basename(path), {}, {}
    if not os.path.exists(path):
        errors.append(f"{name}: missing")
        return rows
    with open(path) as f:
        for ln, line in enumerate(f, 1):
            if not line.strip():
                continue
            where = f"{name}:{ln}"
            try:
                r = json.loads(line)
            except ValueError as e:
                errors.append(f"{where}: not JSON ({e})")
                continue
            if not isinstance(r, dict):
                errors.append(f"{where}: not a record")
                continue
            try:
                key = (str(r["corpus"]), canonical(r["arm"]), int(r["threads"]), int(r["round"]))
            except (KeyError, TypeError, ValueError):
                errors.append(f"{where}: no corpus, arm, threads or round")
                continue
            if key in rows:
                errors.append(f"{where}: {key} duplicated")
                continue
            if key not in roster:
                errors.append(f"{where}: {key} not in the roster")
            if r.get("err") or r.get("termination") != "success":
                errors.append(f"{where}: {key} did not succeed ({r.get('termination')!r}, {r.get('err')!r})")
            for field in ("wall", "cpu", "rss_kb"):
                v = r.get(field)
                if isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) or v <= 0:
                    errors.append(f"{where}: {key} {field} {v!r} is not a finite positive number")
            out, digest = r.get("out"), r.get("sha256") or r.get("hash")
            if isinstance(out, bool) or not isinstance(out, int) or out <= 0 or not isinstance(digest, str) or not digest:
                errors.append(f"{where}: {key} output size {out!r} or hash {digest!r} missing")
            else:
                outputs.setdefault(key[0], set()).add((out, digest))
            hs = r.get("held_spinner")
            if isinstance(hs, dict) and hs.get("overlap"):
                errors.append(f"{where}: {key} ran beside a spinner sharing a decoder core")
            n = occupancy_workers(key[1]) if check_workers else None
            if n is not None and worker_names(r) != sorted(f"mibz2-{i}" for i in range(n)):
                errors.append(f"{where}: {key} observed workers {worker_names(r)}, not the arm's {n}")
            if meta is not None:
                core_of, cores, hold_max, *unpinned = meta
                check_metadata(where, key, r, core_of, cores, hold_max, errors, *unpinned)
            rows[key] = r
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{name}: {len(missing)} of {len(roster)} roster keys missing, e.g. {missing[:3]}")
    for corpus, outs in sorted(outputs.items()):
        if len(outs) != 1:
            errors.append(f"{corpus}: the runs' outputs differ: {sorted(outs)[:3]}")
    return rows


def parse_judge_at(specs, cands, judged, errors):
    """`--judge-at ARM=T[,T...]`'s map {candidate: its judged counts}; {} without any (errors
    appended for a malformed or incomplete set)"""
    out = {}
    for spec in specs:
        arm, eq, counts = spec.rpartition("=")
        try:
            ts = {int(t) for t in counts.split(",") if t}
        except ValueError:
            ts = set()
        if not eq or not arm or not ts:
            errors.append(f"--judge-at {spec!r}: not ARM=COUNTS")
            continue
        if arm not in cands:
            errors.append(f"--judge-at {spec!r}: {arm} is not a candidate")
        if arm in out:
            errors.append(f"--judge-at {spec!r}: {arm} named twice")
        if ts - set(judged):
            errors.append(f"--judge-at {spec!r}: {sorted(ts - set(judged))} not judged counts")
        out[arm] = ts
    if out:
        for c in cands:
            if c not in out:
                errors.append(f"the candidate {c} has no --judge-at")
        for t in judged:
            if not any(t in ts for ts in out.values()):
                errors.append(f"the judged count {t} judges no candidate")
    return out


def parse_ref_at(specs, judged, errors):
    """`--ref-at ARM=T[,T...]`'s map {count: its reference arm}; errors appended for a malformed
    spec, a count that is not judged, and a judged count with no reference or two"""
    out = {}
    for spec in specs:
        arm, eq, counts = spec.rpartition("=")
        try:
            ts = [int(t) for t in counts.split(",") if t]
        except ValueError:
            ts = []
        if not eq or not arm or not ts:
            errors.append(f"--ref-at {spec!r}: not ARM=COUNTS")
            continue
        for t in ts:
            if t not in judged:
                errors.append(f"--ref-at {spec!r}: {t} is not a judged count")
            elif t in out:
                errors.append(f"--ref-at {spec!r}: {t} has two references ({out[t]} and {arm})")
            else:
                out[t] = arm
    for t in judged:
        if t not in out and specs:
            errors.append(f"the judged count {t} has no --ref-at")
    return out


def parse_pairs(specs, arms, judge_at, present, judged, errors):
    """`--pair A/B`'s list of (A, B); errors appended for a malformed pair, one not among the
    arms, or one present at no judged count (`present(t)`: the arms in the roster at t)"""
    out = []
    if specs and not judge_at:
        errors.append("--pair: with --judge-at only")
        return out
    for spec in specs:
        parts = spec.split("/")
        if len(parts) != 2 or not all(parts) or parts[0] == parts[1]:
            errors.append(f"--pair {spec!r}: not ARM/ARM, two arms")
            continue
        if any(x not in arms for x in parts):
            errors.append(f"--pair {spec!r}: {[x for x in parts if x not in arms]} not an arm of the run")
            continue
        if not any(set(parts) <= set(present(t)) for t in judged):
            errors.append(f"--pair {spec!r}: the two arms run together at no judged count")
            continue
        out.append(tuple(parts))
    return out


def by_count(a, refs, cands, context, judge_at, judged, corpora, cell, peak, pairs=(), present=None, pcell=None):
    """The verdicts with candidates judged by count (`--judge-at`): one Holm family over every
    judged main-corpus cell, then at each count a verdict for each candidate judged there and
    the choice among those that ADOPT. `refs`: the reference arm at each count ({t: arm})"""
    steps, ps, mem, gm, gall, grow = {}, {}, {}, {}, {}, {}
    here = {t: [c for c in cands if t in judge_at[c]] for t in judged}
    primary = [c for c in (a.primary or "").split(",") if c]

    def trade_ok(cand, t, c):
        g = gall[(cand, t, c)]
        return (c in primary and steps[(cand, t, c)] == "FASTER" and g <= TRADE_GAIN
                and mem[(cand, t, c)] - 1 <= TRADE_RATIO * (1 - g) and grow[(cand, t, c)] <= TRADE_KIB)

    def traded(cand, t, over):
        # rule 4 (#558): one corpus may trade; no other corpus may be above the gate
        return bool(primary) and len(over) == 1 and trade_ok(cand, t, over[0])

    def trade_line(cand, t, c):
        return (f"{c}{' (primary)' if c in primary else ' (not primary)'} gain {100 * (1 - gall[(cand, t, c)]):.1f}% "
                f"{steps[(cand, t, c)]}, growth {100 * (mem[(cand, t, c)] - 1):.1f}% and "
                f"{grow[(cand, t, c)] / 1024:.1f} MiB: {'trades' if trade_ok(cand, t, c) else 'does not trade'}")
    for t in judged:
        ref = refs[t]
        print(f"\n{t} threads: the judged cells, each candidate over {ref}, the paired rounds' wall ratios: "
              f"geometric mean [95% interval]; one-sided p (gain); peak memory (median of the runs' peaks) "
              f"over {ref}'s")
        for cand in here[t]:
            print(f"  {cand}")
            for corpus in corpora:
                g, lo, hi, p = step(cell(corpus, t, cand))
                steps[(cand, t, corpus)] = label(lo, hi)
                mem[(cand, t, corpus)] = peak(corpus, cand, t) / peak(corpus, ref, t)
                gall[(cand, t, corpus)] = g
                grow[(cand, t, corpus)] = peak(corpus, cand, t) - peak(corpus, ref, t)
                if corpus == a.main:
                    ps[(cand, t)], gm[(cand, t)] = p, g
                print(f"    {corpus}{' (gain family)' if corpus == a.main else ' (loss and memory guard)'}: "
                      f"{g:.4f} [{lo:.4f}, {hi:.4f}] {steps[(cand, t, corpus)]}"
                      + (f"; p {p:.2e}" if corpus == a.main else "") + f"; memory {mem[(cand, t, corpus)]:.3f}")
        others = [arm for arm in (*([] if a.sparse else cands), *context) if arm not in here[t]]
        if others:
            print(f"  Context at {t} threads, reported and not judged:")
            for arm in others:
                for corpus in corpora:
                    g, lo, hi, _ = step(cell(corpus, t, arm))
                    print(f"    {arm} {corpus}: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; "
                          f"memory {peak(corpus, arm, t) / peak(corpus, ref, t):.3f}")
        here_pairs = [(x, y) for x, y in pairs if {x, y} <= set(present(t))]
        if here_pairs:
            print(f"  Pairs at {t} threads, reported and not judged: the first arm over the second, the paired "
                  f"rounds' wall ratios; peak memory over the second's")
            for x, y in here_pairs:
                for corpus in corpora:
                    g, lo, hi, _ = step(pcell(corpus, t, x, y))
                    print(f"    {x} over {y} {corpus}: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; "
                          f"memory {peak(corpus, x, t) / peak(corpus, y, t):.3f}")
    rejected = holm(ps, a.alpha)
    print(f"\nHolm at family α {a.alpha} over the {len(ps)} main-corpus cells: "
          + ", ".join(f"{c} at {t} p {ps[(c, t)]:.2e} {'rejects' if (c, t) in rejected else 'does not'}"
                      for c, t in sorted(ps, key=ps.get)))
    for t in judged:
        adopts = []
        for cand in here[t]:
            slower = [c for c in corpora if steps[(cand, t, c)] == "SLOWER"]
            over = [c for c in corpora if mem[(cand, t, c)] > RSS_GATE]
            head = f"VERDICT for {cand} at {t} threads"
            print()
            if (cand, t) in rejected and not slower and not over:
                adopts.append(cand)
                print(f"{head}: ADOPTS. A gain (Holm), " + ("neither cell" if len(corpora) == 2 else "no cell")
                      + f" SLOWER, and peak memory within {RSS_GATE:.2f} of the reference's in "
                      + ("both corpora" if len(corpora) == 2 else "every corpus"))
            elif (cand, t) in rejected and not slower and traded(cand, t, over):
                adopts.append(cand)
                print(f"{head}: ADOPTS BY TRADE (PERFORMANCE-POLICY rule 4). A gain (Holm), no cell SLOWER, and "
                      f"peak memory above {RSS_GATE:.2f} only on {over}, each a primary corpus with a gain of at "
                      f"least 5% beyond noise, growth at most twice it and at most 64 MB: "
                      + "; ".join(trade_line(cand, t, c) for c in over))
            elif (cand, t) in rejected and not slower:
                if a.primary and len(over) > 1:
                    print(f"  (rule 4's trade does not hold: peak memory is above {RSS_GATE:.2f} on {len(over)} "
                          f"corpora, and no other corpus may be: "
                          + "; ".join(trade_line(cand, t, c) for c in over) + ")")
                elif a.primary:
                    print(f"  (rule 4's trade does not hold: "
                          + "; ".join(trade_line(cand, t, c) for c in over) + ")")
                print(f"{head}: HELD ON MEMORY. A gain (Holm) and neither cell SLOWER, but peak memory is above "
                      f"{RSS_GATE:.2f} of the reference's on {over}: PERFORMANCE-POLICY rule 4, the Coordinator's to weigh")
            elif (cand, t) in rejected:
                print(f"{head}: A FINDING, NOT ADOPTED. A gain (Holm), but SLOWER on {slower}")
            else:
                print(f"{head}: NOT ADOPTED. No gain established (Holm does not reject)"
                      + (f"; SLOWER on {slower}" if slower else "") + (f"; memory above the gate on {over}" if over else ""))
        print()
        if adopts:
            best = min(adopts, key=lambda c: gm[(c, t)])
            print(f"THE CHOICE at {t} threads: {best}, of those that ADOPT there ({', '.join(adopts)}), the lowest "
                  f"geometric mean of its main-corpus step ({gm[(best, t)]:.4f})")
        else:
            print(f"THE CHOICE at {t} threads: none; the reference stays")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--ref")
    ap.add_argument("--ref-at", action="append", default=[], metavar="ARM=COUNTS",
                    help="in place of --ref, with --judge-at and --sparse: the reference at these counts")
    ap.add_argument("--unpinned", action="store_true",
                    help="every record ran on every registered CPU (ab run --unpinned), with no spinner")
    ap.add_argument("--cand", required=True, action="append")
    ap.add_argument("--main", required=True)
    ap.add_argument("--guard", required=True)
    ap.add_argument("--judged", required=True)
    ap.add_argument("--controls", default="")
    ap.add_argument("--rounds", type=int, default=10)
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--cores", type=int, default=10)
    ap.add_argument("--cpus", type=int, default=20)
    ap.add_argument("--hold-max-threads", type=int, default=9,
                    help="the spinner's limit as the run had it; 0 for a run without one")
    ap.add_argument("--context", action="append", default=[],
                    help="an arm checked with the run and reported over the reference, not judged")
    ap.add_argument("--judge-at", action="append", default=[], metavar="ARM=COUNTS",
                    help="a candidate judged at these counts alone; with any, each judged count is read alone")
    ap.add_argument("--sparse", action="store_true",
                    help="with --judge-at: each count's roster is the reference, its judged candidates and "
                         "the context arms alone")
    ap.add_argument("--pair", action="append", default=[], metavar="ARM/ARM",
                    help="with --judge-at: the first arm over the second at each judged count where both run, "
                         "reported and not judged")
    ap.add_argument("--alpha", type=float, default=ALPHA,
                    help="Holm's family α (0.05; half each where a run is judged in two calls)")
    ap.add_argument("--primary", default="",
                    help="with --judge-at: the primary corpora, comma-separated, where rule 4 may trade memory for speed")
    ap.add_argument("--rounds-at", action="append", default=[], metavar="T=N",
                    help="with --sparse: N paired rounds at judged count T in place of --rounds")
    ap.add_argument("--check-workers", action="store_true",
                    help="every occupancy arm's record names exactly its N worker threads")
    a = ap.parse_args(argv[1:])
    judged = [int(t) for t in a.judged.split(",")]
    controls = [int(t) for t in a.controls.split(",") if t]
    R, ref, cands, context = a.rounds, a.ref, a.cand, a.context
    errors = []
    if R < 2:
        errors.append(f"--rounds {R}: at least 2")
    rounds_at = {}
    for spec in a.rounds_at:
        t, eq, n = spec.partition("=")
        try:
            t, n = int(t), int(n)
        except ValueError:
            errors.append(f"--rounds-at {spec!r}: not T=N")
            continue
        if not eq or n < 2 or t not in judged or t in rounds_at or not a.sparse:
            errors.append(f"--rounds-at {spec!r}: a judged count once, at least 2 rounds, with --sparse")
            continue
        rounds_at[t] = n

    def rounds(t):
        return rounds_at.get(t, R)
    if (ref is None) == (not a.ref_at):
        errors.append("the reference: --ref or --ref-at, exactly one of them")
    if a.ref_at and not (a.judge_at and a.sparse):
        errors.append("--ref-at: with --judge-at and --sparse only")
    refs = parse_ref_at(a.ref_at, judged, errors) if a.ref_at else {t: ref for t in judged + controls}
    ref_arms = list(dict.fromkeys(refs[t] for t in judged if t in refs)) if a.ref_at else [ref]
    if a.unpinned and a.hold_max_threads != 0:
        errors.append(f"--unpinned: with --hold-max-threads 0 only (no spinner), not {a.hold_max_threads}")
    judge_at = parse_judge_at(a.judge_at, cands, judged, errors)
    if a.ref_at:
        # a reference by count may be a candidate at another count (`just bench-recalibrate`:
        # 6/24 the shipped trim at 8 threads, one worker fewer at 10), never at its own
        arms = list(dict.fromkeys([*ref_arms, *cands, *context]))
        one = (len(set(cands)) != len(cands) or len(set(context)) != len(context)
               or set(context) & (set(ref_arms) | set(cands))
               or any(refs.get(t) == c for c, ts in judge_at.items() for t in ts))
    else:
        arms = [*ref_arms, *cands, *context]
        one = len(set(arms)) != len(arms)
    if set(judged) & set(controls) or one:
        errors.append("the judged and control counts overlap, or two arms are one")
    guards = [g for g in a.guard.split(",") if g]
    if not guards or a.main in guards or len(set(guards)) != len(guards):
        errors.append(f"--guard {a.guard!r}: one guard corpus or more, distinct, none the main corpus")
    corpora = [a.main] + guards
    if not 0 < a.alpha <= ALPHA:
        errors.append(f"--alpha {a.alpha}: above 0 and at most {ALPHA}")
    primary = [c for c in a.primary.split(",") if c]
    if primary and (not judge_at or any(c not in corpora for c in primary)):
        errors.append(f"--primary {a.primary!r}: with --judge-at only, each a corpus of the run")
    if a.sparse and (not judge_at or controls):
        errors.append("--sparse: with --judge-at and no control counts only")
    if a.sparse and judge_at:
        here = {t: [refs.get(t, ref), *(c for c in cands if t in judge_at.get(c, ())), *context] for t in judged}
        roster = {(c, arm, t, i) for c in corpora for t in judged for arm in here[t] for i in range(rounds(t))}
    else:
        here = {t: arms for t in judged}
        roster = {(c, arm, t, i) for c in corpora for arm in arms for t in judged + controls for i in range(R)}
    pairs = parse_pairs(a.pair, arms, judge_at, here.get, judged, errors)
    core_of = topology(a.env, a.cores, a.cpus, errors)
    rows = load(a.runs, roster, errors,
                None if core_of is None else (core_of, a.cores, a.hold_max_threads, a.unpinned),
                a.check_workers)
    ref_name = ref if not a.ref_at else ("the reference by count: "
                                         + ", ".join(f"{refs[t]} at {t}" for t in judged if t in refs))
    print(f"records {len(rows)}; roster {len(roster)} ({ref_name} and {' and '.join(cands)}"
          + (f"; context {', '.join(context)}" if context else "") + f"; {a.main} and {', '.join(guards)}; "
          f"judged {judged}, controls {controls}; {R} rounds"
          + "".join(f", {n} at {t} threads" for t, n in rounds_at.items()) + ")"
          + (("; judged by count: " + ", ".join(f"{c} at {sorted(ts)}" for c, ts in judge_at.items()))
             if judge_at else "") + ("; sparse: each count's own arms alone" if a.sparse else "")
          + ("; unpinned: every run on every CPU" if a.unpinned else ""))

    def cell(corpus, t, cand):
        return [math.log(rows[(corpus, cand, t, i)]["wall"] / rows[(corpus, refs[t], t, i)]["wall"]) for i in range(rounds(t))]

    def pcell(corpus, t, x, y):
        return [math.log(rows[(corpus, x, t, i)]["wall"] / rows[(corpus, y, t, i)]["wall"]) for i in range(rounds(t))]

    def peak(corpus, arm, t):
        return statistics.median(rows[(corpus, arm, t, i)]["rss_kb"] for i in range(rounds(t)))

    if not errors:
        moved = []
        for cand in cands:
            for corpus in corpora:
                for t in controls:
                    g, lo, hi, _ = step(cell(corpus, t, cand))
                    who = f" {cand}" if len(cands) > 1 else ""
                    print(f"  control{who} {corpus} {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}")
                    if label(lo, hi) != "INCONCLUSIVE":
                        moved.append((corpus, t) if len(cands) == 1 else (cand, corpus, t))
        if moved:
            errors.append(f"control cells moved {moved}: the two arms build the same decoder there")
    if errors:
        print(f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict")
        for e in errors[:25]:
            print(f"  {e}")
        if len(errors) > 25:
            print(f"  ... and {len(errors) - 25} more")
        return 1
    if judge_at:
        return by_count(a, refs, cands, context, judge_at, judged, corpora, cell, peak, pairs, here.get, pcell)

    steps, ps, mem, gm = {}, {}, {}, {}
    for cand in cands:
        print(f"\nThe judged cells: {cand} over {ref}, the paired rounds' wall ratios: geometric mean "
              f"[95% interval]; one-sided p (gain); peak memory (median of the runs' peaks) over {ref}'s")
        for corpus in corpora:
            print(f"\n{corpus}{' (gain family)' if corpus == a.main else ' (loss and memory guard)'}")
            for t in judged:
                g, lo, hi, p = step(cell(corpus, t, cand))
                steps[(cand, corpus, t)] = label(lo, hi)
                mem[(cand, corpus, t)] = peak(corpus, cand, t) / peak(corpus, ref, t)
                if corpus == a.main:
                    ps[(cand, t)] = p
                    gm.setdefault(cand, []).append(math.log(g))
                print(f"  {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {steps[(cand, corpus, t)]}"
                      + (f"; p {p:.2e}" if corpus == a.main else "") + f"; memory {mem[(cand, corpus, t)]:.3f}")
    for arm in context:
        print(f"\nContext, reported and not judged: {arm} over {ref}, the paired rounds' wall ratios: geometric "
              f"mean [95% interval]; peak memory over {ref}'s")
        for corpus in corpora:
            print(f"\n{corpus}")
            for t in judged:
                g, lo, hi, _ = step(cell(corpus, t, arm))
                print(f"  {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}; "
                      f"memory {peak(corpus, arm, t) / peak(corpus, ref, t):.3f}")
    rejected = holm(ps, a.alpha)
    single = len(cands) == 1

    def key(k):
        return k[1] if single else f"{k[0]} at {k[1]}"

    print(f"\nHolm at family α {a.alpha} over the {len(ps)} main-corpus cells: "
          + ", ".join(f"{key(k)} p {ps[k]:.2e} {'rejects' if k in rejected else 'does not'}"
                      for k in sorted(ps, key=ps.get)))
    adopts = []
    for cand in cands:
        gains = sorted(t for (c, t) in rejected if c == cand)
        slower = sorted((k[1], k[2]) for k, v in steps.items() if k[0] == cand and v == "SLOWER")
        over = sorted((k[1], k[2]) for k, v in mem.items() if k[0] == cand and v > RSS_GATE)
        head = "VERDICT" if single else f"VERDICT for {cand}"
        print()
        if gains and not slower and not over:
            adopts.append(cand)
            print(f"{head}: ADOPTS. A gain at {gains} threads (Holm), no judged cell SLOWER, and peak memory "
                  f"within {RSS_GATE:.2f} of the reference's at every judged cell")
        elif gains and not slower:
            print(f"{head}: HELD ON MEMORY. A gain at {gains} threads (Holm) and no judged cell SLOWER, but peak "
                  f"memory is above {RSS_GATE:.2f} of the reference's at {over}: PERFORMANCE-POLICY rule 4, the Coordinator's to weigh")
        elif gains:
            print(f"{head}: A FINDING, NOT ADOPTED. A gain at {gains} threads (Holm), but SLOWER at {slower}")
        else:
            print(f"{head}: NOT ADOPTED. No gain established (Holm rejects at no main-corpus cell)"
                  + (f"; SLOWER at {slower}" if slower else "") + (f"; memory above the gate at {over}" if over else ""))
    if not single:
        print()
        if adopts:
            best = min(adopts, key=lambda c: statistics.mean(gm[c]))
            print(f"THE CHOICE: {best}, of those that ADOPT ({', '.join(adopts)}), the lowest geometric mean of "
                  f"its main-corpus steps ({math.exp(statistics.mean(gm[best])):.4f})")
        else:
            print("THE CHOICE: none; the reference stays")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
