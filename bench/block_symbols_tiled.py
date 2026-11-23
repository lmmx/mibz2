#!/usr/bin/env python3
"""bench-block-symbols-tiled-zero-fill's verdict (`just bench-block-symbols-tiled-zero-fill`):
the block reader's symbols zero-filled in tiles of 8,192 just ahead of the groups that write
them, in place of one fill to `max_out` before the first group (perf/block-symbols-tiled-zero-
fill). The rule is the registration's, frozen before the data:
docs/journal/2026-11-23-block-symbols-tiled-zero-fill-registration.md. It is
bench/parscan_verdict.py's rule (copied unchanged from bench/parscan-stream, #771) with the arms
of a default-path change: head `mibz2` (the PR's default decoder) against base
`mibz2@build-base` (master's), the verdict line named TILED ZERO-FILL. The deciding cells are
per corpus and per count, never pooled:
- **wikidata-x4:** FASTER at 6 threads is needed to adopt (judged); SLOWER at any of 1, 2, 4, 8
  or 10 is NOT ADOPTED (`--supporting 1,2,4,8,10`).
- **text** (the loss guard, `ab gen DIR 16`'s, checked whole as Wikidata's is, its decoded
  bytes `--text-out-bytes`, the recipe's count by Python's bz2): SLOWER at any of 1, 2, 4, 6, 8
  or 10 is NOT ADOPTED (GUARD_LOSS).
- **The memory gate:** HELD ON MEMORY if the median peak-RSS ratio, head over base, is above
  1.10 at any count on either corpus (RSS_AT); the line names every cell over. A loss outranks
  it. HELD ON MEMORY is not ADOPTS.
No transfer is reported (no claimed figure to transfer: the judged count's earlier 0.977 is the
effect this run confirms, not a claim from another machine).

    block_symbols_tiled.py RUNS.jsonl --env ENV.json --lscpu LSCPU.txt --text-out-bytes N
                    [--rounds 16] [--corpus wikidata-x4] [--out-bytes N] [--guard-corpus text]
                    [--preflight]

`--preflight`: the untimed preflight's check (one round of every cell, before the timed run):
the machine (lscpu's model, and `ab env`'s topology: 10 cores, 20 CPUs, no quota), every record
complete, successful, unpinned above one thread and exact (bench/parscan_verdict.py's checks at
one round), and the dispatch by `ab`'s thread census (`observed_threads`): above one thread,
both arms run master's serial scanner (`mibz2-scan`) and no helper scanner (`mibz2-scan-1`), so
that the two builds differ in the block reader alone. No step and no verdict: exit 0 if it
passes, 1 if not, and the recipe stops before the timed run.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import parscan_verdict as pv  # noqa: E402
import two_arm as ta  # noqa: E402

THREADS = (1, 2, 4, 6, 8, 10)
# Wikidata's judged count (it must be FASTER: where #770's run measured 0.977) and the counts
# where SLOWER is NOT ADOPTED
JUDGED = "6"
SUPPORTING = "1,2,4,8,10"
# the text loss guard's counts: SLOWER at any of them is NOT ADOPTED
GUARD_LOSS = "1,2,4,6,8,10"
# the memory gate's counts, on both corpora: above 1.10 at any is HELD ON MEMORY
RSS_AT = "1,2,4,6,8,10"
NAME = "TILED ZERO-FILL"
# master's serial scanner's thread, and the streaming front end's first helper (#770), which
# neither build has
SCANNER = "mibz2-scan"
HELPER = "mibz2-scan-1"


def dispatch(got_all, errors):
    """Appends every record above one thread whose census is missing, lacks the serial
    scanner, or has a helper scanner"""
    for corpus, got in got_all.items():
        for (arm, t, rnd), r in sorted(got.items(), key=lambda kv: (kv[0][1], kv[0][0], kv[0][2])):
            if t == 1:
                continue
            o = r.get("observed_threads")
            key = f"{corpus} {(arm, t, rnd)}"
            if not isinstance(o, str):
                errors.append(f"{key}: no thread census (observed_threads)")
                continue
            seen = set(o.split())
            if SCANNER not in seen:
                errors.append(f"{key}: no {SCANNER}: not the serial scanner")
            if HELPER in seen:
                errors.append(f"{key}: {HELPER}: a helper scanner, not the serial scanner")


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
        dispatch(got_all, errors)
        for e in errors:
            print(f"preflight: {e}")
        print("preflight: " + ("FAILED" if errors else "OK (untimed; not evidence)"))
        return 1 if errors else 0
    print(
        "bench-block-symbols-tiled-zero-fill: the block reader's symbols zero-filled in tiles "
        "(perf/block-symbols-tiled-zero-fill)"
    )
    return pv.main(
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
            JUDGED,
            "--supporting",
            SUPPORTING,
            "--reported",
            "",
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
