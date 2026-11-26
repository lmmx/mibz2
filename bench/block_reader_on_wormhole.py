#!/usr/bin/env python3
"""bench-block-reader-on-wormhole's verdict (`just bench-block-reader-on-wormhole`): the block
reader (#745) re-qualified on master with Wormhole's plateau stack (#739, 3296195e). The rule
is the registration's, frozen before the data:
docs/journal/2026-11-22-block-reader-on-wormhole-registration.md. It is bench-parscan-c37's
Decision 2, BLOCK-READER, unchanged: this is a thin wrapper around bench/parscan_c37.py
(`--decision block-reader`: head `mibz2` against base `mibz2@build-base`, judged at 1 thread,
6, 8 and 10 supporting, peak RSS gated at 10, the same VOID checks), with one report added
that decides nothing: the 10-thread step against Decision 2's on the old base.

    block_reader_on_wormhole.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt [--rounds 16]
                    [--corpus wikidata-x4] [--out-bytes N] [--prior 0.9320]

**The composition report** (reported; decides nothing): the 10-thread step here, g, with its
95% interval, beside Decision 2's 0.9320 [0.9233, 0.9407] on base 96a24d3e (bundle
parscan-c37/run-20261122T082138Z), and g / 0.9320 with the same transform of g's interval. Near
1 the block reader's gain at 10 threads is the same on Wormhole's master as it was before it
(the two gains compose, multiplying); above 1 Wormhole took part of it; below 1 the two
reinforce. The interval is this run's alone: the prior is treated as fixed.
"""

import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parscan_c37 as pv  # noqa: E402
import two_arm as ta  # noqa: E402

# bench-parscan-c37's Decision 2 at 10 threads (evidence 5fed1ffc, the bundle's verdict.txt)
PRIOR = 0.9320
PRIOR_CI = (0.9233, 0.9407)
THREADS = (1, 6, 8, 10)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("runs")
    ap.add_argument("--env", required=True)
    ap.add_argument("--lscpu", required=True)
    ap.add_argument("--rounds", type=int, default=16)
    ap.add_argument("--corpus", default="wikidata-x4")
    ap.add_argument("--out-bytes", type=int, default=pv.WIKIDATA_X4_OUT)
    ap.add_argument("--prior", type=float, default=PRIOR)
    a = ap.parse_args(argv)
    print("bench-block-reader-on-wormhole: the block reader (#745) on master 3296195e (#739)")
    rc = pv.main(
        [
            a.runs,
            "--env",
            a.env,
            "--lscpu",
            a.lscpu,
            "--decision",
            "block-reader",
            "--ref-arm",
            "mibz2",
            "--arm",
            "mibz2",
            "--judged",
            "1",
            "--supporting",
            "6,8,10",
            "--reported",
            "",
            "--rss-at",
            "10",
            "--rounds",
            str(a.rounds),
            "--corpus",
            a.corpus,
            "--out-bytes",
            str(a.out_bytes),
        ]
    )
    if rc != 0:
        return rc
    # (pv.main checked every record; read them again for the 10-thread step)
    errors = []
    got = pv.load(a.runs, "mibz2", "mibz2", THREADS, a.rounds, a.corpus, a.out_bytes, errors)
    if errors:
        print("composition: not reported (the records changed while read)")
        return 1
    pairs = [(got[("mibz2", 10, r)], got[("mibz2@build-base", 10, r)]) for r in range(a.rounds)]
    g, lo, hi, _ = ta.step([math.log(c["wall"] / b["wall"]) for c, b in pairs])
    p = a.prior
    print(
        f"composition at 10 threads (reported; decides nothing): here {g:.4f} [{lo:.4f}, {hi:.4f}]; "
        f"Decision 2 on 96a24d3e {p:.4f} [{PRIOR_CI[0]:.4f}, {PRIOR_CI[1]:.4f}]; "
        f"here / there {g / p:.4f} [{lo / p:.4f}, {hi / p:.4f}]"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
