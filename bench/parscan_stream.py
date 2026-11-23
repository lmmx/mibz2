#!/usr/bin/env python3
"""bench-parscan-stream's verdict (`just bench-parscan-stream`): the push-streaming parallel
front end as the default decoder runs it on the i9 from 8 threads (perf/parscan-stream). The
rule is the registration's, frozen before the data:
docs/journal/2026-11-22-parscan-stream-registration.md. It is bench/parscan_verdict.py's
PARSCAN rule with the arms of a default-path change: head `mibz2` (the PR's default decoder)
against base `mibz2@build-base` (master's); judged at 10 threads, 6 and 8 supporting, 1, 2, 4,
12, 16 and 20 reported, peak RSS gated at 10, the same VOID checks. The transfer (reported,
deciding nothing) is against the handoff's own 10-thread claim on wikidata-l9, 0.9523.

**The text loss guard** (the amendment of 23 November 2026, before any data): the same cells on
`text` (`ab gen DIR 16`'s), checked whole as Wikidata's are, its decoded bytes `--text-out-bytes`
(the recipe's count, by Python's bz2). NOT ADOPTED if `text` is SLOWER at 6, 8 or 10 threads,
and text's peak RSS joins the 1.10 gate at 10. Why: the head runs 8 lazy workers from 10
threads where master runs 7, and the recalibration of 22 November (#145 LOG 7085615681,
evidence `472ce552`) measured 8/32 over 7/28 at pinned 10 as 0.935 on Wikidata but 1.109
SLOWER on text (1.06-1.11 SLOWER unpinned at 12-20).

    parscan_stream.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt --text-out-bytes N
                    [--rounds 16] [--corpus wikidata-x4] [--out-bytes N] [--guard-corpus text]
                    [--preflight]

`--preflight`: the untimed preflight's check (one round of every cell, before the timed run):
the machine, and every record complete, successful, unpinned above one thread and exact
(bench/parscan_verdict.py's checks at one round). No step and no verdict: exit 0 if it passes,
1 if not, and the recipe stops before the timed run.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parscan_verdict as pv  # noqa: E402

# the handoff's 10-thread wall ratio on wikidata-l9 (its section 3.4, Cand / MasterA), on a
# 2-core / 4-vCPU host: the transfer's reference, deciding nothing
CLAIMED = 0.9523
THREADS = (1, 2, 4, 6, 8, 10, 12, 16, 20)
# the text loss guard's counts: SLOWER at any of them is NOT ADOPTED
GUARD_LOSS = "6,8,10"


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
        corpora = {a.corpus: a.out_bytes, a.guard_corpus: a.text_out_bytes}
        pv.load_all(a.runs, "mibz2", "mibz2", THREADS, 1, corpora, errors)
        for e in errors:
            print(f"preflight: {e}")
        print("preflight: " + ("FAILED" if errors else "OK (untimed; not evidence)"))
        return 1 if errors else 0
    print(
        "bench-parscan-stream: the default decoder's push-streaming front end (perf/parscan-stream)"
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
            "6,8",
            "--reported",
            "1,2,4,12,16,20",
            "--rss-at",
            "10",
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
