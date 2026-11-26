#!/usr/bin/env python3
"""bench-parscan-c37's verdict (`just bench-parscan-c37`), read from the run's records only
after they are checked whole. The rule is the registration's, frozen before the data:
docs/journal/2026-11-22-parscan-c37-registration.md. Standard library only; the statistics are
bench/two_arm.py's (`step`, `label`, `topology`). bench/open_r5.py's checks and output with two
arms of different names (the reference at the base, the candidate at the head), a wider grid
with reported counts, the memory gate at the judged count alone, and the handoff's transfer
ratio reported.

    parscan_c37.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt
                    [--ref-arm mibz2] [--arm mibz2-parscan] [--judged 10] [--supporting 6,8]
                    [--reported 1,2,4,12,16,20] [--rounds 16] [--corpus wikidata-x4]
                    [--out-bytes N] [--claimed 0.9333] [--decision parscan|block-reader]
                    [--diagnostic]

**Two decisions, one rule each, both frozen in the registration.** `--decision` sets the
arms and the counts' roles, and the verdict line names the decision:
- **Decision 1, PARSCAN** (`--decision parscan`, the default): `mibz2-parscan` at the head
  against `mibz2` at the base; judged 10, supporting 6 and 8, reported 1, 2, 4, 12, 16 and
  20; memory gated at 10; the transfer reported.
- **Decision 2, BLOCK-READER** (`--decision block-reader`): `mibz2` at the head (the default
  decoder, on the PR's block reader) against `mibz2` at the base; judged 1 (it must be
  FASTER), supporting 6, 8 and 10 (none SLOWER); memory gated at 10; no transfer.
Explicit `--arm`, `--judged`, `--supporting`, `--reported` and `--rss-at` override them.

**Arms.** The reference is REF_ARM at the base (`REF_ARM@build-base`: master's 96a24d3e, the
PR's merge-base), the candidate ARM at the head (`ARM`, this harness's own build: the PR's
head).

**A count's step:** the rounds' paired wall ratios, head over base: their geometric mean and its
95% interval (Student's t on the logs). FASTER if the interval lies below 1, SLOWER if above,
else INCONCLUSIVE.

**The verdict** (not with `--diagnostic`):
- **ADOPTS:** the judged count (10 threads; 1 for BLOCK-READER) FASTER, no supporting count
  (6, 8; 6, 8 and 10 for BLOCK-READER) SLOWER, and at the memory count (10 threads, `--rss-at`)
  the median over the rounds of each run's peak resident memory, head over
  base, at most 1.10 (PERFORMANCE-POLICY rule 4's trigger).
- **HELD ON MEMORY:** as ADOPTS, but the memory gate failed at the memory count: the Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** otherwise (the judged count not FASTER, or a supporting count SLOWER).
- The **reported** counts (1, 2, 4, 12, 16, 20) are printed with their steps and decide nothing.
- The **transfer** at the judged count (PARSCAN only) is printed and decides nothing: ln(step) / ln(CLAIMED),
  the share of the handoff's claimed 10-worker log gain (0.9333x wall, on a 2-vCPU VM) that the
  i9 shows, with the same transform of the step's interval.
- **VOID** (exit 1, no verdict): the file missing or unreadable; the machine not an i9-10900K
  with 10 cores, 20 CPUs and no CPU quota; any record not a successful run (termination
  `success`, no `err`) with finite positive wall, CPU and peak memory; a record at more than one
  thread not unpinned (`cpus` `all`), or a one-thread record not on a single CPU; a record beside
  a spinner; any key (arm, threads, round) duplicated, unexpected or missing from the full
  roster; the corpus not the one registered; any run's output size not the registered one, or
  the outputs' hashes not all one.

`--diagnostic`: the same checks and steps, reported, deciding nothing (bench-parscan-c37 does
not use it since BLOCK-READER became a decision).
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
# the handoff's claimed 10-worker wall ratio on wikidata-l9 (its section 4.1)
CLAIMED = 0.9333
# bench/wikidata.json's decoded bytes (the corpus's 209,700,196), four copies
WIKIDATA_X4_OUT = 4 * 209_700_196
# each decision's arm and counts' roles (judged, supporting, reported, memory count) and whether
# the transfer is reported
DECISIONS = {
    "parscan": ("Decision 1, PARSCAN", "mibz2-parscan", "10", "6,8", "1,2,4,12,16,20", 10, True),
    "block-reader": ("Decision 2, BLOCK-READER", "mibz2", "1", "6,8,10", "", 10, False),
}


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


def one_cpu(s):
    """Whether `s` names a single CPU (ab's one-thread pin, `--pin`, default 1)"""
    lst = ta.cpu_list(s)
    return lst is not None and len(lst) == 1


def load(path, ref, cand, threads, rounds, corpus, out_bytes, errors):
    """{(arm, threads, round): record} once every record is checked; errors appended. The roster
    is REF at the base (`REF@build-base`) and CAND itself"""
    arms = [f"{ref}@build-base", cand]
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
        if key[1] == 1:
            if not one_cpu(r.get("cpus")):
                errors.append(
                    f"{where}: {key} on {r.get('cpus')!r}, not one CPU (ab's 1-thread pin)"
                )
        elif r.get("cpus") != "all":
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


def cell(got, ref, cand, t, rounds):
    """(wall step, CPU step, median peak-memory ratio, base and head median peak memory in KiB),
    head over base, at t threads"""
    pairs = [(got[(cand, t, r)], got[(f"{ref}@build-base", t, r)]) for r in range(rounds)]
    wall = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    cpu = ta.step([math.log(c["cpu"] / b["cpu"]) for c, b in pairs])
    head_kb = statistics.median(c["rss_kb"] for c, _ in pairs)
    base_kb = statistics.median(b["rss_kb"] for _, b in pairs)
    return wall, cpu, head_kb / base_kb, base_kb, head_kb


def transfer(x, claimed):
    """ln(x) / ln(claimed): 1 the claimed gain carried whole, 0 none, below 0 reversed"""
    return math.log(x) / math.log(claimed)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--ref-arm", default="mibz2")
    ap.add_argument("--decision", choices=sorted(DECISIONS), default="parscan")
    ap.add_argument("--arm", default=None)
    ap.add_argument("--judged", default=None)
    ap.add_argument("--supporting", default=None)
    ap.add_argument("--reported", default=None)
    ap.add_argument("--rss-at", type=int, default=None, help="the memory gate's count")
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=WIKIDATA_X4_OUT)
    ap.add_argument("--claimed", type=float, default=CLAIMED)
    ap.add_argument("--diagnostic", action="store_true", help="report the steps only: no verdict")
    a = ap.parse_args(argv)
    name, arm, j, sup, rep_, rss_t, with_transfer = DECISIONS[a.decision]
    a.arm = arm if a.arm is None else a.arm
    a.judged = j if a.judged is None else a.judged
    a.supporting = sup if a.supporting is None else a.supporting
    a.reported = rep_ if a.reported is None else a.reported
    rss_t = rss_t if a.rss_at is None else a.rss_at
    judged, supporting, reported = counts(a.judged), counts(a.supporting), counts(a.reported)
    threads = sorted(set(judged + supporting + reported))
    errors = []
    if a.rounds < 2:
        errors.append(f"--rounds {a.rounds}: at least 2")
    roles = [set(judged), set(supporting), set(reported)]
    if any(roles[i] & roles[j] for i in range(3) for j in range(i + 1, 3)):
        errors.append("a count in two roles (judged, supporting, reported)")
    if not a.diagnostic and len(judged) != 1:
        errors.append(f"--judged {a.judged!r}: one count")
    if not a.diagnostic and rss_t not in threads:
        errors.append(f"--rss-at {rss_t}: not one of the counts")
    if not 0 < a.claimed < 1:
        errors.append(f"--claimed {a.claimed}: a gain, between 0 and 1")
    ta.topology(a.env, 10, 20, errors)
    machine(a.lscpu, errors)
    got = load(a.runs, a.ref_arm, a.arm, threads, a.rounds, a.corpus, a.out_bytes, errors)
    if errors:
        print("VOID: no verdict")
        for e in errors[:50]:
            print(f"  {e}")
        if len(errors) > 50:
            print(f"  ... and {len(errors) - 50} more")
        return 1
    print(f"{name}" + ("" if not a.diagnostic else " (diagnostic)"))
    print(
        f"{a.arm} at the head against {a.ref_arm}@build-base, {a.corpus}, unpinned (1 thread on "
        f"one CPU), {a.rounds} paired rounds"
        + (" (diagnostic: decides nothing)" if a.diagnostic else "")
    )
    print(
        f"{'threads':>7}  {'role':<10} {'wall head/base [95%]':<30} {'step':<12} "
        f"{'CPU head/base [95%]':<28} {'peak RSS (median)':>17}"
    )
    steps, walls, rss_at, mem = {}, {}, {}, []
    for t in threads:
        (g, lo, hi, _), (cg, clo, chi, _), rss, base_kb, head_kb = cell(
            got, a.ref_arm, a.arm, t, a.rounds
        )
        mem.append((t, base_kb, head_kb, rss))
        steps[t], walls[t], rss_at[t] = ta.label(lo, hi), (g, lo, hi), rss
        role = (
            "diagnostic"
            if a.diagnostic
            else "judged"
            if t in judged
            else "supporting"
            if t in supporting
            else "reported"
        )
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
    (t_j,) = judged
    g, lo, hi = walls[t_j]
    # (the interval's ends swap: a lower ratio is a larger share of the claimed gain)
    if with_transfer:
        print(
            f"transfer at {t_j} threads (reported; decides nothing): ln({g:.4f}) / ln({a.claimed}) = "
            f"{transfer(g, a.claimed):.3f} [{transfer(hi, a.claimed):.3f}, {transfer(lo, a.claimed):.3f}] "
            f"of the handoff's claimed gain"
        )
    gain = steps[t_j] == "FASTER"
    loss = [t for t in supporting if steps[t] == "SLOWER"]
    rss_ok = rss_at[rss_t] <= RSS_GATE
    if gain and not loss and rss_ok:
        verdict = "ADOPTS"
    elif gain and not loss:
        verdict = (
            f"HELD ON MEMORY (peak RSS {rss_at[rss_t]:.4f} at {rss_t} threads, above {RSS_GATE}: "
            "the Coordinator weighs it, PERFORMANCE-POLICY rule 4)"
        )
    else:
        why = []
        if not gain:
            why.append(f"{t_j} threads not FASTER")
        if loss:
            why.append(f"SLOWER at {', '.join(map(str, loss))} threads")
        verdict = f"NOT ADOPTED ({'; '.join(why)})"
    print(f"verdict ({name}): {verdict}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
