#!/usr/bin/env python3
"""bench-scanner-plumbing's verdict (`just bench-scanner-plumbing`), read from the run's records
only after they are checked whole. The rule is the registration's, frozen before the data:
docs/journal/2026-11-25-scanner-plumbing-registration.md. Standard library only; the statistics
are bench/two_arm.py's (`step`, `label`, `topology`). A copy of bench-open-relay8's
bench/open_relay8.py (#707), itself bench-open-diet's, bench-open-r4's, bench-open-lanes's and
bench-open-vec16's, with the same step, checks and peak RSS report. What differs: the counts
(judged 9, supporting 6, 8, 10 and 12), the workers each run used (reported, and a VOID if the
head changed them), PERFORMANCE-POLICY rule 4's trade (reported under HELD ON MEMORY), and a
`--dry` mode for the recipe's untimed check off the i9.

    scanner_plumbing.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt
                    [--arm mibz2] [--judged 9] [--supporting 6,8,10,12] [--rounds 16]
                    [--corpus wikidata-x4] [--out-bytes N] [--diagnostic] [--dry]

**The counts:** the defaults (judged 9; supporting 6, 8, 10 and 12) are the first registration's,
as its run (#850 at b39c2cfc, journal §79) was judged. Journal §84's test 2 (the combined fix with
master merged in, the spin warm-up) moves the judged counts to production's defaults, decided
before its data: judged 10 and 12, supporting 6, 8 and 9. The recipe passes those.

**The clock split** (journal §81, a registered secondary that decides nothing): where every run
carries `cycles_per_ref` (the recipe's PRESPIN passes --machine-state), each count's clock ratio
and its wall at a common clock, head over base, are reported beside the wall.

**Arms.** The reference is ARM at the base (`ARM@build-base`, the measured head's merge-base
with master), the candidate ARM at the head (`ARM`, this harness's own build of the head the
recipe was given). Which head and base a run measured is the bundle's builds.txt.

**A count's step:** the rounds' paired wall ratios, head over base: their geometric mean and its
95% interval (Student's t on the logs). FASTER if the interval lies below 1, SLOWER if above,
else INCONCLUSIVE.

**The verdict** (not with `--diagnostic`):
- **ADOPTS:** the judged count (9 threads) FASTER, no supporting count (6, 8, 10, 12) SLOWER,
  and at every count the median over the rounds of each run's peak resident memory, head over
  base, at most 1.10 (PERFORMANCE-POLICY rule 4's trigger).
- **HELD ON MEMORY:** as ADOPTS, but the memory gate failed at a count: the Coordinator weighs
  the growth under PERFORMANCE-POLICY rule 4. Reported beside it, deciding nothing on its own:
  whether rule 4's "accepted by rule" trade holds at each count over the gate (a wall gain of
  at least 5% with its interval below 1, the growth in percent at most twice the gain, and at
  most 64 MB).
- **NOT ADOPTED:** otherwise (the judged count not FASTER, or a supporting count SLOWER).
- **VOID** (exit 1, no verdict): the file missing or unreadable; the machine not an i9-10900K
  with 10 cores, 20 CPUs and no CPU quota; any record not a successful run (termination
  `success`, no `err`) with finite positive wall, CPU and peak memory; a record not unpinned
  (`cpus` `all`) or beside a spinner; any key (arm, threads, round) duplicated, unexpected or
  missing from the full roster; the corpus not the one registered; any run's output size not
  the registered one, or the outputs' hashes not all one; at a count, the median number of
  decoder workers the runs used (`observed_threads`' `mibz2-N` threads) not the same at the
  base and the head (the head changed the plan, which this registration does not measure).

`--diagnostic`: the same checks and steps, for the `mibz2-workers-lazy` arm at 8, 9 and 10
workers; it reports the steps and decides nothing.

`--dry`: the recipe's untimed check off the i9. The machine's checks are reported, not
applied, and every result is labelled DRY: it is never evidence.
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


def plan_check(got, arm, threads, rounds, errors):
    """{t: (base workers, head workers)}, medians over the rounds; a difference is an error"""
    ref, out = f"{arm}@build-base", {}
    for t in threads:
        b = statistics.median(workers(got[(ref, t, r)]) for r in range(rounds))
        h = statistics.median(workers(got[(arm, t, r)]) for r in range(rounds))
        out[t] = (b, h)
        if b != h:
            errors.append(
                f"at {t} threads the base ran {b:g} workers and the head {h:g}: the plan moved"
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


def clock_split(got, arm, t, rounds):
    """Journal §81's registered secondary (it decides nothing): each round's wall ratio, head over
    base, split into the clock ratio (cycles_per_ref, cycles over reference cycles, recorded with
    --machine-state) and the wall at a common clock (the wall times the clock: what the head
    would take at the base's clock). (clock step, common-clock step), or None where a record
    lacks the clock"""
    ref = f"{arm}@build-base"
    clock, common = [], []
    for r in range(rounds):
        c, b = got[(arm, t, r)], got[(ref, t, r)]
        gc, gb = c.get("cycles_per_ref"), b.get("cycles_per_ref")
        if not (finite_pos(gc) and finite_pos(gb)):
            return None
        clock.append(math.log(gc / gb))
        common.append(math.log(c["wall"] / b["wall"]) + clock[-1])
    return ta.step(clock), ta.step(common)


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
    ap.add_argument("--arm", default="mibz2")
    ap.add_argument("--judged", default="9")
    ap.add_argument("--supporting", default="6,8,10,12")
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    ap.add_argument("--diagnostic", action="store_true", help="report the steps only: no verdict")
    ap.add_argument("--dry", action="store_true", help="the untimed check: machine not applied")
    a = ap.parse_args(argv)
    judged, supporting = counts(a.judged), counts(a.supporting)
    threads = sorted(set(judged + supporting))
    errors = []
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    if set(judged) & set(supporting):
        errors.append("a count both judged and supporting")
    if not threads:
        errors.append("no counts")
    host = []
    ta.topology(a.env, 10, 20, host)
    machine(a.lscpu, host)
    if a.dry:
        for e in host:
            print(f"DRY: not applied: {e}")
    else:
        errors += host
    got = load(a.runs, a.arm, threads, a.rounds, a.corpus, a.out_bytes, errors)
    plan = {} if errors else plan_check(got, a.arm, threads, a.rounds, errors)
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
        + (" (diagnostic: decides nothing)" if a.diagnostic else "")
    )
    print(
        f"{'threads':>7}  {'role':<10} {'workers':>7}  {'wall head/base [95%]':<30} {'step':<12} "
        f"{'CPU head/base [95%]':<28} {'peak RSS (median)':>17}"
    )
    steps, over, mem = {}, [], []
    for t in threads:
        (g, lo, hi, _), (cg, clo, chi, _), rss, base_kb, head_kb = cell(got, a.arm, t, a.rounds)
        mem.append((t, base_kb, head_kb, rss))
        steps[t] = ta.label(lo, hi)
        if rss > RSS_GATE:
            over.append((t, trade(g, hi, rss, base_kb, head_kb)))
        role = "diagnostic" if a.diagnostic else "judged" if t in judged else "supporting"
        print(
            f"{t:>7}  {role:<10} {plan[t][1]:>7g}  {g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {steps[t]:<12} "
            f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {rss:>17.4f}"
        )
    print(
        "the clock split (journal §81, a registered secondary; decides nothing): the clock and the "
        "wall at a common clock, head/base [95%]:"
    )
    for t in threads:
        split = clock_split(got, a.arm, t, a.rounds)
        if split is None:
            print(f"{t:>7}  not recorded (a run without the clock: no --machine-state, or no PMU)")
        else:
            (kg, klo, khi, _), (wg, wlo, whi, _) = split
            print(
                f"{t:>7}  clock {kg:.4f} [{klo:.4f}, {khi:.4f}]  "
                f"wall at a common clock {wg:.4f} [{wlo:.4f}, {whi:.4f}]"
            )
    print("peak RSS, median over the rounds (reported; decides nothing beyond the 1.10 gate):")
    for t, base_kb, head_kb, rss in mem:
        print(
            f"{t:>7}  base {base_kb / 1024:.1f} MiB, head {head_kb / 1024:.1f} MiB, "
            f"head - base {(head_kb - base_kb) / 1024:+.1f} MiB ({rss:.4f})"
        )
    if a.diagnostic:
        return 0
    gain = all(steps[t] == "FASTER" for t in judged)
    loss = [t for t in supporting if steps[t] == "SLOWER"]
    if gain and not loss and not over:
        verdict = "ADOPTS"
    elif gain and not loss:
        verdict = (
            f"HELD ON MEMORY (peak RSS above {RSS_GATE} at "
            f"{', '.join(str(t) for t, _ in over)} threads: the Coordinator weighs it, "
            "PERFORMANCE-POLICY rule 4)"
        )
    else:
        why = []
        if not gain:
            why.append(f"{'/'.join(map(str, judged))} threads not FASTER")
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
