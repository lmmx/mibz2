# #631 readiness check

**HOLD before freezing or queuing the i9 attempt.** The fixed counts and all 39 published checks reproduce, including the three original counterexamples. The corrected judge can still return `ADOPTS` for the wrong **judged held-out Wikidata** digest or a fractional thread label. Both violate the proposed admission rule. I ran only synthetic scorer and shell checks.

Reviewed registration [`4e86ffc8`](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/docs/journal/2026-11-19-open-port-registration.md) against CHECKPOINT [7007879370](https://github.com/lmmx/mibz2/pull/162#issuecomment-7007879370) and the Pathfinder hand-back [7007865255](https://github.com/lmmx/mibz2/pull/145#issuecomment-7007865255). The pin moved to [`9743bbe0`](https://github.com/lmmx/mibz2/commit/9743bbe0d51ba451b8b8ab9e6fd731c238940052). The compare against `3f09500e` contains only the Miri workflow and the B5 journal.

## Roster and identities

Running `python3 bench/test_open_port.py` on the pinned source gave **39 of 39 checks pass**. The judge constructs the expected 1,080 decision records (15 corpora × 3 counts × 12 pairs × 2 builds) and 16 sustained records (8 pairs × 2 builds). The absent guard roster, NaN peak RSS and empty sustained file each give VOID. The reference commit and two base Wikidata hashes are compared against constants, and the port commit is compared against the supplied `--port` pin. The normal queue runs the recipe with its registered defaults.

I changed one field or receipt entry at a time on the full synthetic fixture supplied by the suite and invoked its judge with `PORT_SHA`.

| Change to the fixture | Observed result |
| --- | --- |
| Put `not-a-digest` in the `text.bz2` SHA-256 field | exit 0, ADOPTS |
| Change the judged `ho-wikidata-l9.bz2` digest to `11` repeated 32 times, leaving `wikidata-heldout.bz2` at the registered SHA-256 | exit 0, ADOPTS |
| Set both sides of the first `text` pair at 8 threads to `threads: 8.5` | exit 0, ADOPTS |
| Set `arm: 1` on that pair | uncaught `AttributeError`, no VOID |

The held-out check has a direct invariant. [`heldout.py`](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/bench/heldout.py#L73) uses the input's compressed bytes verbatim for level 9, and the recipe copies that file to its judged path. The two receipt digests must match the same frozen value. [`identities()`](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/bench/open_port.py#L290) currently validates the base digest and checks only for the judged basename. [`load()`](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/bench/open_port.py#L206) uses `int()` before comparing the roster and calls `startswith()` before validating the arm's type.

Before freezing, require the registered held-out digest on the judged copy, require valid SHA-256 strings from the receipt for each registered input, and validate exact integer thread/round fields and the arm type before indexing records. Keep the regression cases synthetic.

## One attempt

The registration retains the first timed attempt or its failure. VOID gives no adoption evidence. An inconclusive required primary or held-out gain yields NOT ADOPTED, and neither result authorizes a retry. The workstation waiter claims the frozen recipe/commit atomically and records failures. The recipe also refuses a second invocation with a prior local timed marker or an archived `open-port/run-*.tar.gz` receipt.

I ran the exact [recipe guard](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/.just/bench.just#L2243) in temporary repositories. It refused a prior local timed marker and a fetched archive as intended. With neither a local marker nor a readable evidence ref, it allowed execution to proceed. The fetch error and the missing ref are treated as an empty archive. This leaves the archive fallback open after loss of local state. Make an unavailable evidence check stop a run that relies on the archive for its one-attempt check.

## Preflights

The registration records the earlier dry runs as timed preflight, excluded from the decision and with timings unread. That is the recorded disclosure. I did not open any earlier timing records.

The [current DRY branch](https://github.com/lmmx/mibz2/blob/4e86ffc8ab1030e9bc49b731176fbfe6028c83be/.just/bench.just#L2316) runs one child decode per build and corpus, saves output size and digest, runs the synthetic checks, bundles and exits before the timed harness, context and census. The [port's child implementation](https://github.com/lmmx/mibz2/blob/9743bbe0d51ba451b8b8ab9e6fd731c238940052/examples/ab.rs#L1448) computes and emits timing fields internally, which the DRY pipeline drops before logging the selected fields. “No timing fields retained” is the accurate scope of the source check.

The hand-back reports all 16 corpus outputs equal and no timing field in cloud bundle `run-20261119T111634Z`, SHA-256 `98ba331d4e49b8f09fbf8e05b8b5b9cde97193471860c3c9c702286da9215957`. I found no archive at that run's path on `evidence` and have no linked bundle bytes. The source verifies the filter. The checksum and contents of that exact cloud archive remain unverified. A direct link to the untimed bundle would allow both to be checked before timing.
