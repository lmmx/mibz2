#!/usr/bin/env python3
"""bench-wormhole-plateau's verdict (`just bench-wormhole-plateau`), read from the run's records
only after they are checked whole. The rule is the registration's, frozen before the data:
docs/journal/2026-11-22-wormhole-plateau-registration.md. Standard library only; the statistics
are bench/two_arm.py's (`step`, `label`, `topology`). bench-open-r5's bench/open_r5.py (#735)
with its nobi pass removed, its counts and rounds changed, and the memory route
(PERFORMANCE-POLICY.md §2, "a memory gain is a gain") added after the wall route.

    wormhole_plateau.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt
                    [--arm mibz2] [--judged 10] [--supporting 6,8,12,16,20] [--rounds 12]
                    [--corpus wikidata-x4] [--out-bytes N] [--diagnostic]

**Arms.** The reference is ARM at the base (`ARM@build-base`), the candidate ARM at the head
(`ARM`, this harness's own build).

**A count's step:** the rounds' paired wall ratios, head over base: their geometric mean and its
95% interval (Student's t on the logs). FASTER if the interval lies below 1, SLOWER if above,
else INCONCLUSIVE. **Its memory step:** the same of the paired peak-RSS ratios.

**The verdict** (not with `--diagnostic`), the routes in this order:
- **ADOPTS (wall):** the judged count (10 threads) FASTER, no supporting count (6, 8, 12, 16,
  20) SLOWER, and at every count the median over the rounds of each run's peak RSS, head over
  base, at most 1.10.
- **ADOPTS (memory):** where the wall route does not hold: the judged count's memory step's
  interval wholly below 0.90 (peak RSS at least 10% lower beyond noise), no count SLOWER in
  wall time, the judged one included, and the 1.10 gate held at every count.
- **HELD ON MEMORY:** the wall route's speed holds but the 1.10 gate fails at a count (and the
  memory route does not hold): the Coordinator weighs it, PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** otherwise.
- **VOID** (exit 1, no verdict): the file missing or unreadable; the machine not an i9-10900K
  with 10 cores, 20 CPUs and no CPU quota; any record not a successful run (termination
  `success`, no `err`) with finite positive wall, CPU and peak memory; a record not unpinned
  (`cpus` `all`) or beside a spinner; any key (arm, threads, round) duplicated, unexpected or
  missing from the full roster; the corpus not the one registered; any run's output size not
  the registered one, or the outputs' hashes not all one.

`--diagnostic`: the same checks and steps; it reports them and decides nothing.
"""

import argparse
import json
import math
import os
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import two_arm as ta  # noqa: E402

RSS_GATE = 1.10
# the memory route's line: the judged count's peak-RSS interval wholly below it
MEM_GAIN = 0.90
MODEL = "i9-10900K"
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196


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


def load(path, arm, threads, rounds, corpus, out_bytes, errors):
    """{(arm, threads, round): record} once every record is checked; errors appended. The roster
    is ARM at the base (`ARM@build-base`) and ARM itself"""
    arms = [f"{arm}@build-base", arm]
    want = {(a, t, r) for a in arms for t in threads for r in range(rounds)}
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
        hashes.add(r.get("hash"))
    for key in sorted(want - set(got)):
        errors.append(f"missing {key}")
    if len(hashes) > 1 or None in hashes:
        errors.append(f"the outputs differ: hashes {sorted(map(str, hashes))}")
    return got


def cell(got, arm, t, rounds):
    """(wall step, CPU step, peak-memory step, median peak-memory ratio, base and head median
    peak memory in KiB), head over base, at t threads"""
    ref = f"{arm}@build-base"
    pairs = [(got[(arm, t, r)], got[(ref, t, r)]) for r in range(rounds)]
    wall = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    cpu = ta.step([math.log(c["cpu"] / b["cpu"]) for c, b in pairs])
    mem = ta.step([math.log(c["rss_kb"] / b["rss_kb"]) for c, b in pairs])
    head_kb = statistics.median(c["rss_kb"] for c, _ in pairs)
    base_kb = statistics.median(b["rss_kb"] for _, b in pairs)
    return wall, cpu, mem, head_kb / base_kb, base_kb, head_kb


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--arm", default="mibz2")
    ap.add_argument("--judged", default="10")
    ap.add_argument("--supporting", default="6,8,12,16,20")
    ap.add_argument("--rounds", type=int, default=12)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    ap.add_argument("--diagnostic", action="store_true", help="report the steps only: no verdict")
    a = ap.parse_args(argv)
    judged, supporting = counts(a.judged), counts(a.supporting)
    threads = sorted(set(judged + supporting))
    errors = []
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    if set(judged) & set(supporting):
        errors.append("a count both judged and supporting")
    if not a.diagnostic and len(judged) != 1:
        errors.append(f"--judged {a.judged!r}: one count")
    ta.topology(a.env, 10, 20, errors)
    machine(a.lscpu, errors)
    got = load(a.runs, a.arm, threads, a.rounds, a.corpus, a.out_bytes, errors)
    if errors:
        print("VOID: no verdict")
        for e in errors[:50]:
            print(f"  {e}")
        if len(errors) > 50:
            print(f"  ... and {len(errors) - 50} more")
        return 1
    print(
        f"{a.arm} at the head against {a.arm}@build-base, {a.corpus}, unpinned, {a.rounds} paired rounds"
        + (" (diagnostic: decides nothing)" if a.diagnostic else "")
    )
    print(
        f"{'threads':>7}  {'role':<10} {'wall head/base [95%]':<30} {'step':<12} "
        f"{'CPU head/base [95%]':<28} {'peak RSS head/base [95%]':<28} {'median':>7}"
    )
    steps, rss_ok, mem, mem_hi = {}, True, [], {}
    for t in threads:
        (g, lo, hi, _), (cg, clo, chi, _), (mg, mlo, mhi, _), rss, base_kb, head_kb = cell(
            got, a.arm, t, a.rounds
        )
        mem.append((t, base_kb, head_kb, rss))
        mem_hi[t] = mhi
        steps[t] = ta.label(lo, hi)
        rss_ok &= rss <= RSS_GATE
        role = "diagnostic" if a.diagnostic else "judged" if t in judged else "supporting"
        print(
            f"{t:>7}  {role:<10} {g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {steps[t]:<12} "
            f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {mg:.4f} [{mlo:.4f}, {mhi:.4f}]{'':<6} "
            f"{rss:>7.4f}"
        )
    print("peak RSS, median over the rounds:")
    for t, base_kb, head_kb, rss in mem:
        print(
            f"{t:>7}  base {base_kb / 1024:.1f} MiB, head {head_kb / 1024:.1f} MiB, "
            f"head - base {(head_kb - base_kb) / 1024:+.1f} MiB ({rss:.4f})"
        )
    if a.diagnostic:
        return 0
    (j,) = judged
    gain = steps[j] == "FASTER"
    loss = [t for t in supporting if steps[t] == "SLOWER"]
    slower = [t for t in threads if steps[t] == "SLOWER"]
    mem_gain = mem_hi[j] < MEM_GAIN
    if gain and not loss and rss_ok:
        verdict = "ADOPTS (wall: 10 threads FASTER, no count SLOWER, peak RSS within 1.10)"
    elif mem_gain and not slower and rss_ok:
        verdict = (
            f"ADOPTS (memory: peak RSS at {j} threads below {MEM_GAIN} of the base beyond noise, "
            "no count SLOWER, peak RSS within 1.10 at every count; PERFORMANCE-POLICY §2)"
        )
    elif gain and not loss:
        verdict = f"HELD ON MEMORY (peak RSS above {RSS_GATE} at a count: the Coordinator weighs it, PERFORMANCE-POLICY rule 4)"
    else:
        why = []
        if not gain:
            why.append(f"{j} threads not FASTER")
        if slower:
            why.append(f"SLOWER at {', '.join(map(str, slower))} threads")
        if not mem_gain:
            why.append(f"peak RSS at {j} threads not below {MEM_GAIN} beyond noise")
        if not rss_ok:
            why.append(f"peak RSS above {RSS_GATE} at a count")
        verdict = f"NOT ADOPTED ({'; '.join(why)})"
    print(f"verdict: {verdict}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
