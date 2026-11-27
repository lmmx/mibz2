#!/usr/bin/env python3
"""bench-scanner-adaptive-seam's verdict (`just bench-scanner-adaptive-seam`), read from the
run's records only after they are checked whole. The rule is the frozen C6 adoption registration:
docs/journal/2026-11-27-scanner-adaptive-seam-window/registration.md §5 (verbatim from
frontier/claims/scanner-adaptive-seam-window.toml). Standard library only; paired statistics are
bench/two_arm.py's (`step`, `label`, `topology`).

    scanner_adaptive_seam.py RUNS.jsonl --identity ID.json --env ENV.json --lscpu LSCPU.txt
                           --in-maint IN_MAINT.json
                           [--arm mibz2] [--judged 10,12] [--supporting 6,8,9]
                           [--rounds 16] [--corpus wikidata-x4] [--out-bytes N]
                           [--diagnostic] [--dry]

**The verdict** (not with `--diagnostic`):
- **ADOPTS:** traced in_maint at 8 workers drops by >= 85% (head/base <= 0.15) with peak window
  capacity head/base <= 1.00 (bench/scanner_adaptive_seam_trace.py's summary), BOTH judged
  counts (10 and 12 threads, production's 8 workers) are FASTER (95% CI upper bound < 1.000),
  NONE of the supporting counts (6, 8, 9 threads) is SLOWER (95% CI lower bound > 1.000), and
  median peak RSS head/base <= 1.05 at every count.
- **NOT ADOPTED:** otherwise. Final for this mechanism under this production; record in
  `frontier/dead-ends.tsv`.
- **VOID** (exit 1, no verdict): missing/unreadable file; machine not an i9-10900K with
  10 cores, 20 CPUs and no CPU quota; any record not `success` with finite positive wall,
  CPU and peak RSS; any record not unpinned (`cpus == "all"`) or beside a spinner; any
  (arm, threads, round) key duplicated, unexpected or missing; wrong corpus or decoded
  byte count, or differing output hashes; a missing or malformed in_maint summary; median observed worker count (`mibz2-N`) differing
  between base and head at any thread count; or any undefined 95% confidence bound.
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
from scanner_adaptive_seam_identity import OUTPUT_BYTES  # noqa: E402

RSS_GATE = 1.05
MODEL = "i9-10900K"
WIKIDATA_X4_OUT = OUTPUT_BYTES
WORKER = re.compile(r"^mibz2-\d+$")


def counts(s):
    return [int(x) for x in s.split(",") if x]


def machine(lscpu, errors):
    """The model name lscpu recorded must be the registered machine's."""
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
    """The decoder's worker threads a run's record observed (`mibz2-0`, `mibz2-1`, ...), or None."""
    names = r.get("observed_threads")
    if not isinstance(names, str):
        return None
    return sum(1 for n in names.split() if WORKER.match(n))


def load(path, arm, threads, rounds, corpus, out_bytes, expected_hash, errors):
    """{(arm, threads, round): record} once every record is checked; errors appended."""
    ref, cand = f"{arm}@build-base", arm
    want = {(a, t, r) for a in (ref, cand) for t in threads for r in range(rounds)}
    got, hashes = {}, set()
    try:
        with open(path) as f:
            lines = f.read().splitlines()
    except OSError as e:
        errors.append(f"{path}: unreadable ({e})")
        return got
    for lineno, line in enumerate(lines, 1):
        where = f"{os.path.basename(path)}:{lineno}"
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except json.JSONDecodeError as e:
            errors.append(f"{where}: invalid JSON ({e})")
            continue
        if not isinstance(r, dict):
            errors.append(f"{where}: record is not an object")
            continue
        if r.get("corpus") != corpus:
            errors.append(f"{where}: corpus {r.get('corpus')!r}, not {corpus!r}")
        key = (ta.canonical(r.get("arm")), r.get("threads"), r.get("round"))
        if key not in want:
            errors.append(f"{where}: unexpected key {key}")
            continue
        if key in got:
            errors.append(f"{where}: duplicate key {key}")
            continue
        got[key] = r
        if r.get("termination") != "success" or r.get("err") != "":
            errors.append(
                f"{where}: {key} failed (termination={r.get('termination')!r}, err={r.get('err')!r})"
            )
        for field in ("wall", "cpu", "rss_kb"):
            if not finite_pos(r.get(field)):
                errors.append(f"{where}: {key} {field} {r.get(field)!r} not finite and positive")
        if r.get("cpus") != "all":
            errors.append(f"{where}: {key} pinned {r.get('cpus')!r}, not unpinned ('all')")
        if r.get("held_spinner") is not None:
            errors.append(f"{where}: {key} ran beside a spinner")
        if r.get("out") != out_bytes:
            errors.append(f"{where}: {key} wrote {r.get('out')!r} bytes, not {out_bytes}")
        if not workers(r):
            errors.append(f"{where}: {key} observed_threads {r.get('observed_threads')!r} missing")
        hashes.add(r.get("hash"))
    for key in sorted(want - set(got)):
        errors.append(f"missing {key}")
    if hashes != {expected_hash}:
        errors.append(f"the outputs differ: hashes {sorted(map(str, hashes))}")
    return got


def plan_check(got, arm, threads, rounds, errors):
    """{t: (base workers, head workers)}, medians over the rounds; a difference is an error."""
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
    """(wall step, CPU step, median peak-memory ratio, base and head median peak memory in KiB)."""
    ref = f"{arm}@build-base"
    pairs = [(got[(arm, t, r)], got[(ref, t, r)]) for r in range(rounds)]
    wall = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    cpu = ta.step([math.log(c["cpu"] / b["cpu"]) for c, b in pairs])
    head_kb = statistics.median(c["rss_kb"] for c, _ in pairs)
    base_kb = statistics.median(b["rss_kb"] for _, b in pairs)
    return wall, cpu, head_kb / base_kb, base_kb, head_kb


def clock_split(got, arm, t, rounds):
    """Journal §81's registered secondary (decides nothing): clock and common-clock wall ratios."""
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


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--identity", required=True, help="the untimed SHA-256 reference JSON")
    ap.add_argument("--env", required=True, help="the machine as `ab env` recorded it")
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--in-maint", help="bench/scanner_adaptive_seam_trace.py's summary (required for a verdict)")
    ap.add_argument("--arm", default="mibz2")
    ap.add_argument("--judged", default="10,12")
    ap.add_argument("--supporting", default="6,8,9")
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
    if not a.diagnostic and not judged:
        errors.append("no judged counts")
    host = []
    ta.topology(a.env, 10, 20, host)
    machine(a.lscpu, host)
    if a.dry:
        for e in host:
            print(f"DRY: not applied: {e}")
    else:
        errors += host
    expected_hash = None
    try:
        with open(a.identity) as f:
            identity = json.load(f)
        if (not isinstance(identity, dict) or identity.get("out") != a.out_bytes
                or not re.fullmatch("[0-9a-f]{64}", str(identity.get("sha256")))
                or not re.fullmatch("[0-9a-f]{16}", str(identity.get("hash")))):
            raise ValueError("missing or malformed digest or output length")
        expected_hash = identity["hash"]
    except (OSError, ValueError) as e:
        errors.append(f"invalid output reference: {e}")
    maint = None
    if not a.diagnostic:
        try:
            with open(a.in_maint or "") as f:
                maint = json.load(f)
            for k in ("in_maint_ratio", "window_cap_ratio"):
                if not finite_pos(maint.get(k)):
                    raise ValueError(f"{k} {maint.get(k)!r} not finite and positive")
            if not a.dry and (maint["base"]["workers"], maint["base"]["slots"]) != (8, 32):
                raise ValueError("the traced plan is not 8 workers / 32 slots")
        except (OSError, ValueError, KeyError, TypeError, AttributeError) as e:
            errors.append(f"invalid in_maint summary: {e}")
    got = load(a.runs, a.arm, threads, a.rounds, a.corpus, a.out_bytes, expected_hash, errors)
    plan = {} if errors else plan_check(got, a.arm, threads, a.rounds, errors)
    cells = {}
    if not errors:
        for t in threads:
            res = cell(got, a.arm, t, a.rounds)
            (g, lo, hi, _), _, rss, _, _ = res
            if not (finite_pos(g) and finite_pos(lo) and finite_pos(hi) and finite_pos(rss)):
                errors.append(f"at {t} threads: undefined confidence bound or RSS ratio")
            cells[t] = res
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
        (g, lo, hi, _), (cg, clo, chi, _), rss, base_kb, head_kb = cells[t]
        mem.append((t, base_kb, head_kb, rss))
        steps[t] = ta.label(lo, hi)
        if rss > RSS_GATE:
            over.append((t, rss))
        role = "diagnostic" if a.diagnostic else "judged" if t in judged else "supporting"
        print(
            f"{t:>7}  {role:<10} {plan[t][1]:>7g}  {g:.4f} [{lo:.4f}, {hi:.4f}]{'':<8} {steps[t]:<12} "
            f"{cg:.4f} [{clo:.4f}, {chi:.4f}]{'':<6} {rss:>17.4f}"
        )
    print(
        "the clock split (journal §81, diagnostic; decides nothing): clock and common-clock wall, "
        "head/base [95%]:"
    )
    for t in threads:
        split = clock_split(got, a.arm, t, a.rounds)
        if split is None:
            print(f"{t:>7}  not recorded (no --machine-state, or no PMU)")
        else:
            (kg, klo, khi, _), (wg, wlo, whi, _) = split
            print(
                f"{t:>7}  clock {kg:.4f} [{klo:.4f}, {khi:.4f}]  "
                f"wall at a common clock {wg:.4f} [{wlo:.4f}, {whi:.4f}]"
            )
    print("peak RSS, median over the rounds (gated at <= 1.05 at every count):")
    for t, base_kb, head_kb, rss in mem:
        print(
            f"{t:>7}  base {base_kb / 1024:.1f} MiB, head {head_kb / 1024:.1f} MiB, "
            f"head - base {(head_kb - base_kb) / 1024:+.1f} MiB ({rss:.4f})"
        )
    if a.diagnostic:
        return 0
    m_ok = maint["in_maint_ratio"] <= 0.15
    cap_ok = maint["window_cap_ratio"] <= 1.00
    print(
        f"traced in_maint at 8 workers: head/base {maint['in_maint_ratio']:.4f} "
        f"(drop {100 * (1 - maint['in_maint_ratio']):.2f}%, gate >= 85%); peak window capacity "
        f"head/base {maint['window_cap_ratio']:.4f} (gate <= 1.00)"
    )
    gain = all(steps[t] == "FASTER" for t in judged)
    loss = [t for t in supporting if steps[t] == "SLOWER"]
    if m_ok and cap_ok and gain and not loss and not over:
        verdict = "ADOPTS"
    else:
        why = []
        if not m_ok:
            why.append(f"in_maint drop {100 * (1 - maint['in_maint_ratio']):.2f}% < 85%")
        if not cap_ok:
            why.append(f"window capacity {maint['window_cap_ratio']:.4f} > 1.00")
        if not gain:
            not_faster = [str(t) for t in judged if steps[t] != "FASTER"]
            why.append(f"{'/'.join(not_faster)} threads not FASTER")
        if loss:
            why.append(f"SLOWER at {', '.join(map(str, loss))} threads")
        if over:
            why.append(
                f"peak RSS > {RSS_GATE:.2f} at {', '.join(f'{t} ({r:.4f})' for t, r in over)} threads"
            )
        verdict = f"NOT ADOPTED ({'; '.join(why)})"
    print(f"{'DRY verdict (not evidence)' if a.dry else 'verdict'}: {verdict}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
