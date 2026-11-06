# Shadow pilot replay

Read [decisions.md](decisions.md) for scope, outcomes and limitations. The scripts only parse archived evidence and static source/listings. They do not compile or execute constructors, run Miri, measure performance, or modify inputs.

With the pinned commits available in a repository:

```sh
python docs/journal/compact30-guardian-shadow/replay_p1.py --repo /path/to/mibz2 --out /tmp/guardian-p1
python docs/journal/compact30-guardian-shadow/replay_p2.py --repo /path/to/mibz2 --out /tmp/guardian-p2
python docs/journal/compact30-guardian-shadow/replay_p4.py --out /tmp/guardian-p4
```

Compare the generated results and manifest with the committed JSON files. Python assertions must remain enabled. The manifest SHA-256 is the evidence digest in the cursor; it covers the exact pinned files, including whole compressed capture archives.

The supplemental Astra v3 replay was inspected and run from `0ac2e67e6826b9f2b1940cb876c3003e7a022144`, path `docs/journal/compact30-research/reviews/v3/replay.py`, using `--repo /path/to/mibz2`. Its result is saved as `astra-v3-replayed.json`; its manifest digest is embedded there. These are archive checks, not a fresh §6.2 review.

The same pin's `docs/journal/compact30-research/reviews/scanner-146/reproduce.py` was inspected and run with `--repo`, `--evidence-dir` and `--output-dir` pointing into scratch. All three archived output comparisons passed: full instruction replay, complete normalized listing comparisons and safety/counter reconstruction. Those results are in `peer-scanner/`. This is reuse and verification of published replay code, not delegation to an agent or a fresh constructor run.

P4 uses only the saved outgoing comment, observed receipt/ACK snapshot and exact channel-log bytes at `35c7cdcb68ffc5ecbcf0eeb37b9307a8ccf207fb`. Its replay verifies the log's Git blob identity, exact message equality and comment-ID correlation. It does not authenticate role headers or measure notification latency. The 3-second source-timestamp discrepancy is reported as D3; P1's blindness defect and P2's evidence defects remain recorded.
