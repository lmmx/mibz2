#!/usr/bin/env python3
"""bench-open-route-adaptive's verdict (`just bench-open-route-adaptive`): the Open kernel's
route by block size on the i9 (perf/open-route-adaptive, #783): up to 98,304 retained rows a
one-ended walk with coalesced bands, past it master's two-ended walk with the mark search left
out of every queue but the marked row's byte's. The rule is the registration's, frozen before
the data: docs/journal/2026-11-23-open-route-adaptive-registration.md. It is
bench/parscan_verdict.py's PARSCAN rule (copied unchanged from bench-parscan-stream's) with the
arms of a default-path change: head `mibz2` (the PR's default decoder) against base
`mibz2@build-base` (master's), the verdict line named OPEN KERNEL ROUTE. The deciding cells are
per corpus and per count, never pooled:
- **wikidata-x4:** FASTER at 10 threads is needed to adopt; SLOWER at any of 6, 8, 12, 16 or
  20 is NOT ADOPTED (`--supporting 6,8,12,16,20`); 1, 2 and 4 are reported (CI's paired
  bench gates them, PERFORMANCE-POLICY rule 2).
- **text** (the loss guard, `ab gen DIR 16`'s, checked whole as Wikidata's is, its decoded
  bytes `--text-out-bytes`, the recipe's count by Python's bz2): SLOWER at any of 6, 8, 10, 12,
  16 or 20 is NOT ADOPTED (GUARD_LOSS).
- **The memory gate:** HELD ON MEMORY if the median peak-RSS ratio, head over base, is above
  1.10 at any of 8, 10, 12, 16 or 20 threads on either corpus (RSS_AT); the line names every
  cell over. A loss outranks it.
The transfer (reported, deciding nothing) is against the handoff's advisory contention proxy
for the stack at 10 workers, 0.980 (a model, not a measurement).

    open_route.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt --text-out-bytes N
                  [--rounds 16] [--corpus wikidata-x4] [--out-bytes N] [--guard-corpus text]
                  [--preflight]

`--preflight`: the untimed preflight's check (one round of every cell, before the timed run):
the machine (lscpu's model, and `ab env`'s topology: 10 cores, 20 CPUs, no quota), and every
record complete, successful, unpinned above one thread and exact, with one output hash a corpus
across both arms (bench/parscan_verdict.py's checks at one round). No step and no verdict: exit
0 if it passes, 1 if not, and the recipe stops before the timed run. There is no activation
census: the route is chosen per block inside the workers, which `ab` does not see (the
registration's "Where the code runs").
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parscan_verdict as pv  # noqa: E402
import two_arm as ta  # noqa: E402

# the handoff's advisory contention proxy for AdaptiveLeanOpen<98304> at 10 workers (its
# register's row 16, `1W 0.982 / 8W 0.966 / 10W 0.980`): the transfer's reference, deciding
# nothing
CLAIMED = 0.980
THREADS = (1, 2, 4, 6, 8, 10, 12, 16, 20)
# Wikidata's counts where SLOWER is NOT ADOPTED (10, judged, must be FASTER), and its reported
SUPPORTING = "6,8,12,16,20"
REPORTED = "1,2,4"
# the text loss guard's counts: SLOWER at any of them is NOT ADOPTED
GUARD_LOSS = "6,8,10,12,16,20"
# the memory gate's counts, on both corpora: above 1.10 at any is HELD ON MEMORY
RSS_AT = "8,10,12,16,20"
NAME = "OPEN KERNEL ROUTE"


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
        pv.load_all(a.runs, "mibz2", "mibz2", THREADS, 1, corpora, errors)
        for e in errors:
            print(f"preflight: {e}")
        print("preflight: " + ("FAILED" if errors else "OK (untimed; not evidence)"))
        return 1 if errors else 0
    print(
        "bench-open-route-adaptive: the Open kernel's route by block size "
        "(perf/open-route-adaptive)"
    )
    return pv.main(
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


if __name__ == "__main__":
    sys.exit(main())
