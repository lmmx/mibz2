#!/usr/bin/env python3
"""A two-arm adoption run's verdict, read from the run's records only after they are checked
whole: a candidate against a reference on a main corpus (the gain family) and a guard corpus,
at judged thread counts, with control counts where the two arms build the same decoder.
Standard library only. Generalised from the stop-and-walk adjudicator (the Researcher's
correction, #145 REVIEW 6921090225), for `just bench-cores-less-one` (#481) and after.

    two_arm.py RUNS.jsonl --ref ARM --cand ARM --main CORPUS --guard CORPUS
               --judged 12,16,20 --controls 9,10 --env ENV.json [--rounds 10]
               [--cores 10 --cpus 20] [--hold-max-threads 9]

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
    CPU on each of that many distinct cores (`ab run --spread`); past the cores, not `all`;
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


def check_metadata(where, key, r, core_of, cores, hold_max, errors):
    """A record's pinning and spinner, against the machine (`core_of`) and the hold limit"""
    t, s = key[2], r.get("cpus")
    if t > cores:
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


def load(path, roster, errors, meta=None):
    """The file's records by key (corpus, canonical arm, threads, round), after checking each
    record and the roster; errors appended. `meta`: (core_of, cores, hold_max) to check each
    record's pinning and spinner against"""
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
            if meta is not None:
                check_metadata(where, key, r, *meta, errors)
            rows[key] = r
    missing = sorted(roster - set(rows))
    if missing:
        errors.append(f"{name}: {len(missing)} of {len(roster)} roster keys missing, e.g. {missing[:3]}")
    for corpus, outs in sorted(outputs.items()):
        if len(outs) != 1:
            errors.append(f"{corpus}: the runs' outputs differ: {sorted(outs)[:3]}")
    return rows


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("runs")
    ap.add_argument("--ref", required=True)
    ap.add_argument("--cand", required=True)
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
    a = ap.parse_args(argv[1:])
    judged = [int(t) for t in a.judged.split(",")]
    controls = [int(t) for t in a.controls.split(",") if t]
    R, ref, cand = a.rounds, a.ref, a.cand
    errors = []
    if R < 2:
        errors.append(f"--rounds {R}: at least 2")
    if set(judged) & set(controls) or ref == cand:
        errors.append("the judged and control counts overlap, or the two arms are one")
    corpora = [a.main, a.guard]
    roster = {(c, arm, t, i) for c in corpora for arm in (ref, cand) for t in judged + controls for i in range(R)}
    core_of = topology(a.env, a.cores, a.cpus, errors)
    rows = load(a.runs, roster, errors, None if core_of is None else (core_of, a.cores, a.hold_max_threads))
    print(f"records {len(rows)}; roster {len(roster)} ({ref} and {cand}; {a.main} and {a.guard}; "
          f"judged {judged}, controls {controls}; {R} rounds)")

    def cell(corpus, t):
        return [math.log(rows[(corpus, cand, t, i)]["wall"] / rows[(corpus, ref, t, i)]["wall"]) for i in range(R)]

    def peak(corpus, arm, t):
        return statistics.median(rows[(corpus, arm, t, i)]["rss_kb"] for i in range(R))

    if not errors:
        moved = []
        for corpus in corpora:
            for t in controls:
                g, lo, hi, _ = step(cell(corpus, t))
                print(f"  control {corpus} {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {label(lo, hi)}")
                if label(lo, hi) != "INCONCLUSIVE":
                    moved.append((corpus, t))
        if moved:
            errors.append(f"control cells moved {moved}: the two arms build the same decoder there")
    if errors:
        print(f"VOID: the records do not make the registered run ({len(errors)} problems); no verdict")
        for e in errors[:25]:
            print(f"  {e}")
        if len(errors) > 25:
            print(f"  ... and {len(errors) - 25} more")
        return 1

    print(f"\nThe judged cells: {cand} over {ref}, the paired rounds' wall ratios: geometric mean "
          f"[95% interval]; one-sided p (gain); peak memory (median of the runs' peaks) over {ref}'s")
    steps, ps, mem = {}, {}, {}
    for corpus in corpora:
        print(f"\n{corpus}{' (gain family)' if corpus == a.main else ' (loss and memory guard)'}")
        for t in judged:
            g, lo, hi, p = step(cell(corpus, t))
            steps[(corpus, t)] = label(lo, hi)
            mem[(corpus, t)] = peak(corpus, cand, t) / peak(corpus, ref, t)
            if corpus == a.main:
                ps[t] = p
            print(f"  {t:>2} threads: {g:.4f} [{lo:.4f}, {hi:.4f}] {steps[(corpus, t)]}"
                  + (f"; p {p:.2e}" if corpus == a.main else "") + f"; memory {mem[(corpus, t)]:.3f}")
    rejected = holm(ps, ALPHA)
    print(f"\nHolm at family α {ALPHA} over the {len(ps)} main-corpus cells: "
          + ", ".join(f"{t} p {ps[t]:.2e} {'rejects' if t in rejected else 'does not'}"
                      for t in sorted(ps, key=ps.get)))
    gain = bool(rejected)
    slower = sorted(k for k, v in steps.items() if v == "SLOWER")
    over = sorted(k for k, v in mem.items() if v > RSS_GATE)
    print()
    if gain and not slower and not over:
        print(f"VERDICT: ADOPTS. A gain at {sorted(rejected)} threads (Holm), no judged cell SLOWER, and peak memory "
              f"within {RSS_GATE:.2f} of the reference's at every judged cell")
    elif gain and not slower:
        print(f"VERDICT: HELD ON MEMORY. A gain at {sorted(rejected)} threads (Holm) and no judged cell SLOWER, but peak "
              f"memory is above {RSS_GATE:.2f} of the reference's at {over}: PERFORMANCE-POLICY rule 4, the Coordinator's to weigh")
    elif gain:
        print(f"VERDICT: A FINDING, NOT ADOPTED. A gain at {sorted(rejected)} threads (Holm), but SLOWER at {slower}")
    else:
        print("VERDICT: NOT ADOPTED. No gain established (Holm rejects at no main-corpus cell)"
              + (f"; SLOWER at {slower}" if slower else "") + (f"; memory above the gate at {over}" if over else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
