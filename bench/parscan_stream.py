#!/usr/bin/env python3
"""bench-parscan-stream's verdict (`just bench-parscan-stream`): the push-streaming parallel
front end as the default decoder runs it on the i9 from 8 threads (perf/parscan-stream). The
rule is the registration's, frozen before the data:
docs/journal/2026-11-22-parscan-stream-registration.md. It is bench/parscan_verdict.py's
PARSCAN rule with the arms of a default-path change: head `mibz2` (the PR's default decoder)
against base `mibz2@build-base` (master's), the verdict line named STREAMING FRONT END. The
deciding cells are per corpus and per count, never pooled:
- **wikidata-x4:** FASTER at 10 threads is needed to adopt; SLOWER at any of 6, 8, 12, 16 or
  20 is NOT ADOPTED (`--supporting 6,8,12,16,20`); 1, 2 and 4 are reported (CI's paired
  bench gates them, PERFORMANCE-POLICY rule 2).
- **text** (the loss guard, `ab gen DIR 16`'s, checked whole as Wikidata's is, its decoded
  bytes `--text-out-bytes`, the recipe's count by Python's bz2): SLOWER at any of 6, 8, 10, 12,
  16 or 20 is NOT ADOPTED (GUARD_LOSS).
- **The memory gate:** HELD ON MEMORY if the median peak-RSS ratio, head over base, is above
  1.10 at any of 8, 10, 12, 16 or 20 threads on either corpus (RSS_AT); the line names every
  cell over. A loss outranks it.
The transfer (reported, deciding nothing) is against the handoff's own 10-thread claim on
wikidata-l9, 0.9523.

Why every count where the front end runs (the acting Researcher's review, #145 comment
7088432559, and the Coordinator's LOG 7088432851, before any data): from 10 threads the head
runs 8 lazy workers with a window of 32 where master runs 7/28, which the recalibration of 22
November (#145 LOG 7085615681, evidence `472ce552`) measured as 1.10 SLOWER on text at 10 and
1.06-1.10 at 12, 16 and 20; Python's `threads=None` runs 20 threads on the i9.

    parscan_stream.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt --text-out-bytes N
                    [--rounds 16] [--corpus wikidata-x4] [--out-bytes N] [--guard-corpus text]
                    [--preflight]

`--preflight`: the untimed preflight's check (one round of every cell, before the timed run):
the machine (lscpu's model, and `ab env`'s topology: 10 cores, 20 CPUs, no quota), every record
complete, successful, unpinned above one thread and exact (bench/parscan_verdict.py's checks at
one round), and the front end's activation by `ab`'s thread census (`observed_threads`,
`activation`). No step and no verdict: exit 0 if it passes, 1 if not, and the recipe stops
before the timed run. The verdict prints the same census, deciding nothing.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parscan_verdict as pv  # noqa: E402
import two_arm as ta  # noqa: E402

# the handoff's 10-thread wall ratio on wikidata-l9 (its section 3.4, Cand / MasterA), on a
# 2-core / 4-vCPU host: the transfer's reference, deciding nothing
CLAIMED = 0.9523
THREADS = (1, 2, 4, 6, 8, 10, 12, 16, 20)
# Wikidata's counts where SLOWER is NOT ADOPTED (10, judged, must be FASTER), and its reported
SUPPORTING = "6,8,12,16,20"
REPORTED = "1,2,4"
# the text loss guard's counts: SLOWER at any of them is NOT ADOPTED
GUARD_LOSS = "6,8,10,12,16,20"
# the memory gate's counts, on both corpora: above 1.10 at any is HELD ON MEMORY
RSS_AT = "8,10,12,16,20"
NAME = "STREAMING FRONT END"
# the front end's threads by name (src/parallel.rs at the head, 2618c54c): workers `mibz2-{i}`,
# the lead scanner `mibz2-scan`, its helpers `mibz2-scan-1` and `mibz2-scan-2`. Master's serial
# scanner is `mibz2-scan` too, and has no helper
HELPER = "mibz2-scan-1"
FRONT_FROM = 8  # PARSCAN_FROM_THREADS: the front end from 8 threads asked
EIGHT_FROM = 10  # PARSCAN_TRIMMED_WORKERS (8 workers, `mibz2-0`..`mibz2-7`) from 10


def names(r):
    """The thread names `ab`'s census saw in a run (`observed_threads`), or None"""
    o = r.get("observed_threads")
    return set(o.split()) if isinstance(o, str) else None


def activation(got_all, ref, cand, errors=None):
    """The front end's activation census: {(corpus, threads): {arm: (runs, with mibz2-scan-1,
    with mibz2-7, with mibz2-8)}}. With ERRORS, appends every record against the registered
    activation: every head record from 8 threads has `mibz2-scan-1`; every head record from
    10 has `mibz2-7` and no `mibz2-8` (8 workers); no base record and no head record below 8
    has `mibz2-scan-1`"""
    base = f"{ref}@build-base"
    census = {}
    for corpus, got in got_all.items():
        for (arm, t, rnd), r in sorted(got.items(), key=lambda kv: (kv[0][1], kv[0][0], kv[0][2])):
            seen = names(r)
            cell = census.setdefault((corpus, t), {}).setdefault(arm, [0, 0, 0, 0])
            cell[0] += 1
            if seen is None:
                if errors is not None:
                    errors.append(f"{corpus} {(arm, t, rnd)}: no thread census (observed_threads)")
                continue
            helper, w7, w8 = HELPER in seen, "mibz2-7" in seen, "mibz2-8" in seen
            cell[1] += helper
            cell[2] += w7
            cell[3] += w8
            if errors is None:
                continue
            key = f"{corpus} {(arm, t, rnd)}"
            if arm == cand and t >= FRONT_FROM and not helper:
                errors.append(f"{key}: no {HELPER}: the front end did not run")
            if arm == cand and t >= EIGHT_FROM and (not w7 or w8):
                errors.append(
                    f"{key}: workers mibz2-7 {'seen' if w7 else 'absent'}, mibz2-8 "
                    f"{'seen' if w8 else 'absent'}: not 8 workers"
                )
            if (arm == base or (arm == cand and t < FRONT_FROM)) and helper:
                errors.append(f"{key}: {HELPER} where the front end must not run")
    return census


def print_census(census, ref, cand):
    print(
        f"the front end's activation by ab's thread census (reported; decides nothing): runs with "
        f"{HELPER}, mibz2-7, mibz2-8"
    )
    for (corpus, t), arms in sorted(census.items()):
        cells = []
        for arm in (f"{ref}@build-base", cand):
            n, h, w7, w8 = arms.get(arm, [0, 0, 0, 0])
            cells.append(f"{'base' if arm != cand else 'head'} {h}/{n}, {w7}/{n}, {w8}/{n}")
        print(f"  {corpus:<12} {t:>2} threads: {'; '.join(cells)}")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True)
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=pv.WIKIDATA_X4_OUT)
    ap.add_argument("--guard-corpus", default="text")
    ap.add_argument("--text-out-bytes", type=int, required=True)
    ap.add_argument("--preflight", action="store_true")
    a = ap.parse_args(argv)
    if a.preflight:
        errors = []
        pv.machine(a.lscpu, errors)
        if a.guard_corpus == a.corpus or a.text_out_bytes <= 0:
            errors.append(
                f"the guard {a.guard_corpus!r}, {a.text_out_bytes} bytes: not a second corpus"
            )
        ta.topology(a.env, 10, 20, errors)
        corpora = {a.corpus: a.out_bytes, a.guard_corpus: a.text_out_bytes}
        got_all = pv.load_all(a.runs, "mibz2", "mibz2", THREADS, 1, corpora, errors)
        activation(got_all, "mibz2", "mibz2", errors)
        for e in errors:
            print(f"preflight: {e}")
        print("preflight: " + ("FAILED" if errors else "OK (untimed; not evidence)"))
        return 1 if errors else 0
    print(
        "bench-parscan-stream: the default decoder's push-streaming front end (perf/parscan-stream)"
    )
    rc = pv.main(
        [
            a.runs,
            "--env",
            a.env,
            "--lscpu",
            a.lscpu,
            "--decision",
            "parscan",
            "--ref-arm",
            "mibz2",
            "--arm",
            "mibz2",
            "--judged",
            "10",
            "--supporting",
            SUPPORTING,
            "--reported",
            REPORTED,
            "--rss-at",
            RSS_AT,
            "--name",
            NAME,
            "--rounds",
            str(a.rounds),
            "--corpus",
            a.corpus,
            "--out-bytes",
            str(a.out_bytes),
            "--claimed",
            str(CLAIMED),
            "--guard",
            a.guard_corpus,
            "--guard-out-bytes",
            str(a.text_out_bytes),
            "--guard-loss",
            GUARD_LOSS,
        ]
    )
    if rc == 0:
        # (the records were checked whole by the verdict; the census decides nothing)
        corpora = {a.corpus: a.out_bytes, a.guard_corpus: a.text_out_bytes}
        got_all = pv.load_all(a.runs, "mibz2", "mibz2", THREADS, a.rounds, corpora, [])
        print_census(activation(got_all, "mibz2", "mibz2"), "mibz2", "mibz2")
    return rc


if __name__ == "__main__":
    sys.exit(main())
