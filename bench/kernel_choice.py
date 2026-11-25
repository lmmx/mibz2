#!/usr/bin/env python3
"""bench-kernel-choice's verdict (`just bench-kernel-choice`), read from the run's records only
after they are checked whole. The rule is the registration's, frozen before the data:
docs/journal/2026-11-25-kernel-choice-registration.md. Standard library only; the statistics are
bench/two_arm.py's (`step`, `label`, `topology`). A sibling of bench-scanner-plumbing's
bench/scanner_plumbing.py (#850), itself bench-open-relay8's, with the same step, checks, peak RSS
report and rule 4 trade. What differs: the counts (3, 4, 5, 7, 8, 10 and 12; 5, 7 and 10 must be
FASTER), and the plan check. This change is meant to change the plan, so the workers are not
required to match base to head. Instead each build must have built exactly the plan the
registration names: the base the old one, the head the one the PR intends (`BASE_PLAN`,
`HEAD_PLAN`), as each build's probe recorded it (`--plan`), and the timed runs' observed workers
must match their build's probe.

    kernel_choice.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt --plan PLAN.jsonl
                    [--expect lazy-from-four|lazy-four-to-seven] [--arm mibz2]
                    [--judged 5,7,10] [--supporting 3,4,8,12] [--rounds 16]
                    [--corpus wikidata-x4] [--out-bytes N] [--dry]

**The expectation** (`--expect`, the recipe's EXPECT): which head was registered, so which plan
the head must build and which counts are judged (`EXPECTATIONS`). Each is its own registration,
frozen before its data:
- `lazy-from-four` (the default; #860, the registration's first head): the head lazy from 4
  threads at every count and 8 workers from 10 (`HEAD_PLAN`); judged 5, 7 and 10, supporting 3,
  4, 8 and 12.
- `lazy-four-to-seven` (journal §80, the registration's second head: #860 without the trim to 8
  from 10): lazy from 4 to 7 threads, otherwise master's plan, 7 lazy workers from 10
  (`HEAD_PLAN_FOUR_TO_SEVEN`); judged 5 and 7, supporting 3, 4, 8, 10 and 12.
`--judged` and `--supporting`, if given, must be the expectation's registered counts.

**Arms.** The reference is ARM at the base (`ARM@build-base`, the measured head's merge-base with
master), the candidate ARM at the head (`ARM`, this harness's own build of the head).

**A count's step:** the rounds' paired wall ratios, head over base: their geometric mean and its
95% interval (Student's t on the logs). FASTER if the interval lies below 1, SLOWER if above,
else INCONCLUSIVE.

**The verdict:**
- **ADOPTS:** every judged count (5, 7, 10 threads; 5 and 7 for `lazy-four-to-seven`) FASTER,
  no supporting count (3, 4, 8, 12; 3, 4, 8, 10 and 12 for `lazy-four-to-seven`) SLOWER, and at every count the median over the rounds of each run's peak resident memory, head
  over base, at most 1.10 (PERFORMANCE-POLICY rule 4's trigger).
- **HELD ON MEMORY:** as ADOPTS, but the memory gate failed at a count; rule 4's trade reported.
- **NOT ADOPTED:** otherwise.
- **VOID** (exit 1, no verdict): the file missing or unreadable; the machine not an i9-10900K
  with 10 cores, 20 CPUs and no CPU quota; any record not a successful run with finite positive
  wall, CPU and peak memory; a record pinned or beside a spinner; any key duplicated, unexpected
  or missing; the corpus not the one registered; any output size not the registered one, or the
  hashes not all one; the plan file missing, malformed, or without exactly one probe per build
  and count; a build's probe not its registered plan (`BASE_PLAN`, the expectation's head plan);
  `--judged` or `--supporting` not the expectation's registered counts; a count's median
  observed workers (`mibz2-N` threads) at a build not its probe's threads.

`--dry`: the recipe's untimed check off the i9. The machine's checks and the registered plans are
reported, not applied (off the i9 neither build trims or chooses the lazy kernel below 8 workers),
and every result is labelled DRY: it is never evidence. The probe-against-observed-workers check
still applies.
"""

import argparse
import json
import math
import os
import re
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

RSS_GATE = 1.10
# PERFORMANCE-POLICY rule 4, accepted by rule: at least a 5% gain, growth at most twice it, at
# most 64 MB a process
TRADE_GAIN, TRADE_MB = 0.05, 64
MODEL = "i9-10900K"
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196
WORKER = re.compile(r"^mibz2-\d+$")

# What `BzReader::parallel(r, t)` builds on the i9 unpinned, at 4 blocks a thread asked:
# (decoding threads, window, lazy). The base is master before the PR (src/parallel.rs:
# `unpinned_cell` with lazy cells at 4 and 6 alone, `Plan::lazy` from 8, the trim to 7 from 10);
# the head is the PR's intent (lazy from 4 at every count, the trim to 8 from 10).
BASE_PLAN = {
    3: (3, 12, False),
    4: (4, 16, True),
    5: (5, 20, False),
    7: (7, 28, False),
    8: (8, 32, True),
    10: (7, 28, True),
    12: (7, 28, True),
}
HEAD_PLAN = {
    3: (3, 12, False),
    4: (4, 16, True),
    5: (5, 20, True),
    7: (7, 28, True),
    8: (8, 32, True),
    10: (8, 32, True),
    12: (8, 32, True),
}
# The second head (journal §80): #860 without the trim to 8 from 10, so lazy at 4 to 7 threads
# (the plan's own workers on the lazy kernel) and master's plan everywhere else, 7 lazy workers
# with 28 from 10 threads
HEAD_PLAN_FOUR_TO_SEVEN = {
    3: (3, 12, False),
    4: (4, 16, True),
    5: (5, 20, True),
    7: (7, 28, True),
    8: (8, 32, True),
    10: (7, 28, True),
    12: (7, 28, True),
}
# EXPECT: (the head's registered plan, judged counts, supporting counts)
EXPECTATIONS = {
    "lazy-from-four": (HEAD_PLAN, (5, 7, 10), (3, 4, 8, 12)),
    "lazy-four-to-seven": (HEAD_PLAN_FOUR_TO_SEVEN, (5, 7), (3, 4, 8, 10, 12)),
}
DEFAULT_EXPECT = "lazy-from-four"


def counts(s):
    return [int(x) for x in s.split(",") if x]


def machine(lscpu, errors):
    """The model name lscpu recorded must be the registered machine's"""
    try:
        with open(lscpu) as f:
            text = f.read()
    except OSError as e:
        errors.append(f"{os.path.basename(lscpu)}: no lscpu record ({e})")
        return
    names = [
        line.split(":", 1)[1].strip()
        for line in text.splitlines()
        if line.split(":", 1)[0].strip() == "Model name"
    ]
    if not names or not any(MODEL in n for n in names):
        errors.append(f"lscpu: model {names!r}, not the {MODEL}")


def finite_pos(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x) and x > 0


def workers(r):
    """The decoder's worker threads a run's record observed (`mibz2-0`, `mibz2-1`, ...), or None"""
    names = r.get("observed_threads")
    if not isinstance(names, str):
        return None
    return sum(1 for n in names.split() if WORKER.match(n))


def load(path, arm, threads, rounds, corpus, out_bytes, errors):
    """{(arm, threads, round): record} once every record is checked; errors appended"""
    ref, cand = f"{arm}@build-base", arm
    want = {(a, t, r) for a in (ref, cand) for t in threads for r in range(rounds)}
    got, hashes = {}, set()
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: unreadable ({e})")
        return got
    for i, line in enumerate(lines, 1):
        where = f"{os.path.basename(path)}:{i}"
        try:
            r = json.loads(line)
        except ValueError:
            errors.append(f"{where}: not JSON")
            continue
        if not isinstance(r, dict):
            errors.append(f"{where}: not a record")
            continue
        key = (ta.canonical(r.get("arm")), r.get("threads"), r.get("round"))
        if key not in want:
            errors.append(f"{where}: unexpected key {key}")
            continue
        if key in got:
            errors.append(f"{where}: {key} duplicated")
            continue
        got[key] = r
        if r.get("corpus") != corpus:
            errors.append(f"{where}: corpus {r.get('corpus')!r}, not {corpus}")
        if r.get("termination") != "success" or r.get("err"):
            errors.append(
                f"{where}: {key} not a successful run ({r.get('termination')!r}, err {r.get('err')!r})"
            )
        for f in ("wall", "cpu", "rss_kb"):
            if not finite_pos(r.get(f)):
                errors.append(f"{where}: {key} {f} {r.get(f)!r} not finite and positive")
        if r.get("cpus") != "all":
            errors.append(f"{where}: {key} pinned {r.get('cpus')!r}, not unpinned ('all')")
        if r.get("held_spinner") is not None:
            errors.append(f"{where}: {key} ran beside a spinner")
        if r.get("out") != out_bytes:
            errors.append(f"{where}: {key} wrote {r.get('out')!r} bytes, not {out_bytes}")
        if workers(r) is None:
            errors.append(f"{where}: {key} observed_threads {r.get('observed_threads')!r} missing")
        hashes.add(r.get("hash"))
    for key in sorted(want - set(got)):
        errors.append(f"missing {key}")
    if len(hashes) > 1 or None in hashes:
        errors.append(f"the outputs differ: hashes {sorted(map(str, hashes))}")
    return got


def probes(path, threads, errors):
    """{(build, t): (threads, window, lazy)} from the probes' records (one per build and count,
    neither the service core nor the pipelined teams); errors appended"""
    out = {}
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{os.path.basename(path)}: no plan record ({e})")
        return out
    for i, line in enumerate(lines, 1):
        where = f"{os.path.basename(path)}:{i}"
        try:
            r = json.loads(line)
        except ValueError:
            errors.append(f"{where}: not JSON")
            continue
        if not isinstance(r, dict) or r.get("kind") != "built":
            errors.append(f"{where}: not a probe record")
            continue
        key = (r.get("build"), r.get("asked"))
        if key[0] not in ("base", "head") or key[1] not in threads:
            errors.append(f"{where}: unexpected probe {key}")
            continue
        if key in out:
            errors.append(f"{where}: probe {key} duplicated")
            continue
        if r.get("service") is not False or r.get("pipelined") is not False:
            errors.append(f"{where}: probe {key} built the service core or pipelined teams")
        n, w, lazy = r.get("threads"), r.get("window"), r.get("lazy")
        if not (isinstance(n, int) and isinstance(w, int) and isinstance(lazy, bool)):
            errors.append(f"{where}: probe {key} malformed")
            continue
        out[key] = (n, w, lazy)
    for b in ("base", "head"):
        for t in threads:
            if (b, t) not in out:
                errors.append(f"missing probe ({b!r}, {t})")
    return out


def fmt(p):
    n, w, lazy = p
    return f"{n}/{w} {'lazy' if lazy else 'eager'}"


def plan_check(built, threads, dry, errors, notes, head_plan=HEAD_PLAN):
    """Each build's probe against its registered plan: an error, or with `dry` a note"""
    for b, plan in (("base", BASE_PLAN), ("head", head_plan)):
        for t in threads:
            want, got = plan.get(t), built[(b, t)]
            if want is None:
                errors.append(f"no registered {b} plan at {t} threads")
            elif got != want:
                msg = f"at {t} threads the {b} built {fmt(got)}, not its registered {fmt(want)}"
                (notes if dry else errors).append(msg)


def observed_check(got, built, arm, threads, rounds, errors):
    """Each build's median observed workers at a count must be its probe's threads"""
    ref = f"{arm}@build-base"
    out = {}
    for t in threads:
        b = statistics.median(workers(got[(ref, t, r)]) for r in range(rounds))
        h = statistics.median(workers(got[(arm, t, r)]) for r in range(rounds))
        out[t] = (b, h)
        for name, seen in (("base", b), ("head", h)):
            if seen != built[(name, t)][0]:
                errors.append(
                    f"at {t} threads the {name}'s runs used {seen:g} workers, its probe "
                    f"{built[(name, t)][0]}: the probe is not what was timed"
                )
    return out


def cell(got, arm, t, rounds):
    """(wall step, CPU step, median peak-memory ratio, base and head median peak memory in KiB),
    head over base, at t threads"""
    ref = f"{arm}@build-base"
    pairs = [(got[(arm, t, r)], got[(ref, t, r)]) for r in range(rounds)]
    wall = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    cpu = ta.step([math.log(c["cpu"] / b["cpu"]) for c, b in pairs])
    head_kb = statistics.median(c["rss_kb"] for c, _ in pairs)
    base_kb = statistics.median(b["rss_kb"] for _, b in pairs)
    return wall, cpu, head_kb / base_kb, base_kb, head_kb


def trade(g, hi, rss, base_kb, head_kb):
    """PERFORMANCE-POLICY rule 4's trade at a count over the gate: (holds, why)"""
    gain = 1 - g
    growth = rss - 1
    mb = (head_kb - base_kb) * 1024 / 1e6
    why = (
        f"gain {100 * gain:.1f}% (interval {'below' if hi < 1 else 'not below'} 1), "
        f"growth {100 * growth:.1f}% and {mb:+.1f} MB"
    )
    ok = hi < 1 and gain >= TRADE_GAIN and growth <= 2 * gain and mb <= TRADE_MB
    return ok, why


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--plan", required=True, help="each build's probe records (plan.jsonl)")
    ap.add_argument(
        "--expect",
        default=DEFAULT_EXPECT,
        choices=sorted(EXPECTATIONS),
        help="the registered head: its plan and its judged and supporting counts",
    )
    ap.add_argument("--arm", default="mibz2")
    ap.add_argument("--judged", help="the expectation's judged counts (checked)")
    ap.add_argument("--supporting", help="the expectation's supporting counts (checked)")
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    ap.add_argument("--dry", action="store_true", help="the untimed check: machine not applied")
    a = ap.parse_args(argv)
    head_plan, reg_judged, reg_supporting = EXPECTATIONS[a.expect]
    errors, notes = [], []
    judged = list(reg_judged) if a.judged is None else counts(a.judged)
    supporting = list(reg_supporting) if a.supporting is None else counts(a.supporting)
    if sorted(judged) != sorted(reg_judged) or sorted(supporting) != sorted(reg_supporting):
        errors.append(
            f"judged {judged} and supporting {supporting}: not {a.expect}'s registered judged "
            f"{list(reg_judged)} and supporting {list(reg_supporting)}"
        )
    threads = sorted(set(judged + supporting))
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    if set(judged) & set(supporting):
        errors.append("a count both judged and supporting")
    if not judged or not threads:
        errors.append("no judged counts")
    host = []
    ta.topology(a.env, 10, 20, host)
    machine(a.lscpu, host)
    if a.dry:
        notes += host
    else:
        errors += host
    got = load(a.runs, a.arm, threads, a.rounds, a.corpus, a.out_bytes, errors)
    built = probes(a.plan, threads, errors)
    observed = {}
    if not errors:
        plan_check(built, threads, a.dry, errors, notes, head_plan)
        observed = observed_check(got, built, a.arm, threads, a.rounds, errors)
    for e in notes:
        print(f"DRY: not applied: {e}")
    if errors:
        print(("DRY: " if a.dry else "") + "VOID: no verdict")
        for e in errors[:50]:
            print(f"  {e}")
        if len(errors) > 50:
            print(f"  ... and {len(errors) - 50} more")
        return 1
    print(
        ("DRY (untimed, not evidence): " if a.dry else "")
        + f"{a.arm} at the head against {a.arm}@build-base, {a.corpus}, unpinned, {a.rounds} paired rounds"
        # (the default's line as the first head's run printed it)
        + ("" if a.expect == DEFAULT_EXPECT else f", expectation {a.expect}")
    )
    print(
        f"{'threads':>7}  {'role':<10} {'base plan':<12} {'head plan':<12} "
        f"{'wall head/base [95%]':<30} {'step':<12} {'CPU head/base [95%]':<28} "
        f"{'peak RSS (median)':>17}"
    )
    steps, over, mem = {}, [], []
    for t in threads:
        (g, lo, hi, _), (cg, clo, chi, _), rss, base_kb, head_kb = cell(got, a.arm, t, a.rounds)
        mem.append((t, base_kb, head_kb, rss))
        steps[t] = ta.label(lo, hi)
        if rss > RSS_GATE:
            over.append((t, trade(g, hi, rss, base_kb, head_kb)))
        role = "judged" if t in judged else "supporting"
        print(
            f"{t:>7}  {role:<10} {fmt(built[('base', t)]):<12} {fmt(built[('head', t)]):<12} "
            f"{g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {steps[t]:<12} "
            f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {rss:>17.4f}"
        )
    print("observed workers, median over the rounds (base, head):")
    print("  " + ", ".join(f"{t}: {b:g}, {h:g}" for t, (b, h) in observed.items()))
    print("peak RSS, median over the rounds (reported; decides nothing beyond the 1.10 gate):")
    for t, base_kb, head_kb, rss in mem:
        print(
            f"{t:>7}  base {base_kb / 1024:.1f} MiB, head {head_kb / 1024:.1f} MiB, "
            f"head - base {(head_kb - base_kb) / 1024:+.1f} MiB ({rss:.4f})"
        )
    gain = [t for t in judged if steps[t] != "FASTER"]
    loss = [t for t in supporting if steps[t] == "SLOWER"]
    if not gain and not loss and not over:
        verdict = "ADOPTS"
    elif not gain and not loss:
        verdict = (
            f"HELD ON MEMORY (peak RSS above {RSS_GATE} at "
            f"{', '.join(str(t) for t, _ in over)} threads: the Coordinator weighs it, "
            "PERFORMANCE-POLICY rule 4)"
        )
    else:
        why = []
        if gain:
            why.append(f"{', '.join(map(str, gain))} threads not FASTER")
        if loss:
            why.append(f"SLOWER at {', '.join(map(str, loss))} threads")
        verdict = f"NOT ADOPTED ({'; '.join(why)})"
    print(f"{'DRY verdict (not evidence)' if a.dry else 'verdict'}: {verdict}")
    for t, (ok, why) in over:
        print(
            f"rule 4's trade at {t} threads (reported; the Coordinator weighs it): "
            f"{'met' if ok else 'not met'}: {why}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
