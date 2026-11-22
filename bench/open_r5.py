#!/usr/bin/env python3
"""bench-open-r5's verdict (`just bench-open-r5`), read from the run's records only after
they are checked whole. The rule is the registration's, frozen before the data:
docs/journal/2026-11-22-open-r5-registration.md. Standard library only; the statistics are
bench/two_arm.py's (`step`, `label`, `topology`). A copy of bench-open-stack's
bench/open_stack.py (#720), itself bench-open-graded's bench/open_graded.py (#715),
bench-open-relay8's (#707), bench-open-diet's (#693) and bench-open-r4's (#680), with the same
rule, checks and output, the median peak RSS report included. Only the names differ.
bench-open-r5 runs no nobi pass; `--nobi` is kept unchanged from bench/open_stack.py, and the
recipe does not use it.

    open_r5.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt
                    [--arm mibz2] [--judged 10] [--supporting 6,8] [--rounds 16]
                    [--corpus wikidata-x4] [--out-bytes N] [--diagnostic [--nobi]]

**Arms.** The reference is ARM at the base (`ARM@build-base`, #734's merge-base with master
4a79f063 at the re-run, master with #739 and #745; 23c1e283 at the first run), the candidate ARM
at the head (`ARM`, this harness's own build: #734's head, e8f1afa2 at the re-run, 8be1f140 at
the first).

**A count's step:** the rounds' paired wall ratios, head over base: their geometric mean and its
95% interval (Student's t on the logs). FASTER if the interval lies below 1, SLOWER if above,
else INCONCLUSIVE.

**The verdict** (not with `--diagnostic`):
- **ADOPTS:** the judged count (10 threads) FASTER, no supporting count (6, 8) SLOWER, and at
  every count the median over the rounds of each run's peak resident memory, head over base,
  at most 1.10 (PERFORMANCE-POLICY rule 4's trigger).
- **HELD ON MEMORY:** as ADOPTS, but the memory gate failed at a count: the Coordinator weighs
  the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** otherwise (the judged count not FASTER, or a supporting count SLOWER).
- **VOID** (exit 1, no verdict): the file missing or unreadable; the machine not an i9-10900K
  with 10 cores, 20 CPUs and no CPU quota; any record not a successful run (termination
  `success`, no `err`) with finite positive wall, CPU and peak memory; a record not unpinned
  (`cpus` `all`) or beside a spinner; any key (arm, threads, round) duplicated, unexpected or
  missing from the full roster; the corpus not the one registered; any run's output size not
  the registered one, or the outputs' hashes not all one.

`--diagnostic`: the same checks and steps, for the `mibz2-workers-lazy` arm at 6 and 8 threads;
it reports the steps and decides nothing.

`--diagnostic --nobi` (unused by bench-open-r5, kept as bench/open_stack.py has it): the nobi
pass, a third arm `ARM@build-nobi` beside the base and the head at every count, with the same
checks over all three arms' full roster. It reports, at each count, the steps head/base,
head/nobi and nobi/base, each with its CPU step and median peak RSS ratio, and decides nothing.
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


def load(path, arm, threads, rounds, corpus, out_bytes, errors, builds=("base",)):
    """{(arm, threads, round): record} once every record is checked; errors appended. The roster
    is ARM at each of `builds` (`ARM@build-X`) and ARM itself"""
    arms = [f"{arm}@build-{b}" for b in builds] + [arm]
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


def cell(got, arm, t, rounds, cand=None, ref=None):
    """(wall step, CPU step, median peak-memory ratio, base and head median peak memory in KiB),
    head over base, at t threads; or `cand` over `ref` (arm names as recorded, canonical)"""
    ref = ref or f"{arm}@build-base"
    cand = cand or arm
    pairs = [(got[(cand, t, r)], got[(ref, t, r)]) for r in range(rounds)]
    wall = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    cpu = ta.step([math.log(c["cpu"] / b["cpu"]) for c, b in pairs])
    head_kb = statistics.median(c["rss_kb"] for c, _ in pairs)
    base_kb = statistics.median(b["rss_kb"] for _, b in pairs)
    return wall, cpu, head_kb / base_kb, base_kb, head_kb


def nobi_report(got, arm, threads, rounds, corpus):
    """The nobi pass's steps: head/base, head/nobi and nobi/base at each count; no verdict"""
    base, nobi = f"{arm}@build-base", f"{arm}@build-nobi"
    print(
        f"{arm} at the head, at nobi ({nobi}) and at the base ({base}), {corpus}, unpinned, "
        f"{rounds} rounds (diagnostic, the nobi pass: decides nothing)"
    )
    print(
        f"{'threads':>7}  {'ratio':<10} {'wall [95%]':<30} {'step':<12} "
        f"{'CPU [95%]':<28} {'peak RSS (median)':>17}"
    )
    for t in threads:
        for name, cand, ref in (
            ("head/base", arm, base),
            ("head/nobi", arm, nobi),
            ("nobi/base", nobi, base),
        ):
            (g, lo, hi, _), (cg, clo, chi, _), rss, _, _ = cell(got, arm, t, rounds, cand, ref)
            print(
                f"{t:>7}  {name:<10} {g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {ta.label(lo, hi):<12} "
                f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {rss:>17.4f}"
            )
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--arm", default="mibz2")
    ap.add_argument("--judged", default="10")
    ap.add_argument("--supporting", default="6,8")
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    ap.add_argument("--diagnostic", action="store_true", help="report the steps only: no verdict")
    ap.add_argument(
        "--nobi",
        action="store_true",
        help="with --diagnostic: the nobi pass, ARM@build-nobi beside the base and the head",
    )
    a = ap.parse_args(argv)
    judged, supporting = counts(a.judged), counts(a.supporting)
    threads = sorted(set(judged + supporting))
    errors = []
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    if set(judged) & set(supporting):
        errors.append("a count both judged and supporting")
    if a.nobi and not a.diagnostic:
        errors.append("--nobi without --diagnostic: the nobi pass decides nothing")
    ta.topology(a.env, 10, 20, errors)
    machine(a.lscpu, errors)
    builds = ("base", "nobi") if a.nobi else ("base",)
    got = load(a.runs, a.arm, threads, a.rounds, a.corpus, a.out_bytes, errors, builds)
    if errors:
        print("VOID: no verdict")
        for e in errors[:50]:
            print(f"  {e}")
        if len(errors) > 50:
            print(f"  ... and {len(errors) - 50} more")
        return 1
    if a.nobi:
        return nobi_report(got, a.arm, threads, a.rounds, a.corpus)
    print(
        f"{a.arm} at the head against {a.arm}@build-base, {a.corpus}, unpinned, {a.rounds} paired rounds"
        + (" (diagnostic: decides nothing)" if a.diagnostic else "")
    )
    print(
        f"{'threads':>7}  {'role':<10} {'wall head/base [95%]':<30} {'step':<12} "
        f"{'CPU head/base [95%]':<28} {'peak RSS (median)':>17}"
    )
    steps, rss_ok, mem = {}, True, []
    for t in threads:
        (g, lo, hi, _), (cg, clo, chi, _), rss, base_kb, head_kb = cell(got, a.arm, t, a.rounds)
        mem.append((t, base_kb, head_kb, rss))
        steps[t] = ta.label(lo, hi)
        rss_ok &= rss <= RSS_GATE
        role = "diagnostic" if a.diagnostic else "judged" if t in judged else "supporting"
        print(
            f"{t:>7}  {role:<10} {g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {steps[t]:<12} "
            f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {rss:>17.4f}"
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
    if gain and not loss and rss_ok:
        verdict = "ADOPTS"
    elif gain and not loss:
        verdict = f"HELD ON MEMORY (peak RSS above {RSS_GATE} at a count: the Coordinator weighs it, PERFORMANCE-POLICY rule 4)"
    else:
        why = []
        if not gain:
            why.append(f"{'/'.join(map(str, judged))} threads not FASTER")
        if loss:
            why.append(f"SLOWER at {', '.join(map(str, loss))} threads")
        verdict = f"NOT ADOPTED ({'; '.join(why)})"
    print(f"verdict: {verdict}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
