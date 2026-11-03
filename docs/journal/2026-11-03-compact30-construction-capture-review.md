# Compact30: the guardian's review of the construction-cost capture (report 201)

**Formal outcome: INVALID.** The registered capture
([2026-11-03-compact30-construction-capture.md](2026-11-03-compact30-construction-capture.md),
archive `c30-construction-20261103T172906Z.tar.gz`, sha256 `47c61c87…bfed5f3`, kept in full in
[its folder](2026-11-03-compact30-construction-capture/)) failed two validity gates, so the
protocol gives it no decision. It is not reclassified as FAIL.

**Investment judgment: STOP for this direct constructor.** This is the guardian's engineering
recommendation, kept separate from the formal outcome: no further capture, threshold change
or constructor work.

**Unchanged: the VALID ADVANCE traversal result** (four-worker CPU cost of the prepared
compact30 walk 0.840 [0.838, 0.843] of the conventional table's) stands. This capture bears
on the cost of building compact30's tables with this constructor, not on the representation,
the walker or that result.

## The validity failures

- **G1 (stability):** production's one-worker CPU cost varied by max ÷ min 1.227 (warm) and
  1.223 (cold) across the 20 rounds, against 1.05. Its four-worker cost varied by about 2.5%
  (warm) and 2.4% (cold).
- **G2 (per-step start skew):** one of 4,560 all-full four-worker steps exceeded 2%: round 7,
  warm, production arm, step 1. Its workers started 35.1 µs apart over an 851.7 µs completion
  span, a skew of 4.125%. The largest row-weighted aggregate skew was 0.091%.
- **All other gates passed:** S0–S10, G3–G10 apart from G1 and G2 as above, with exact output
  on every build.

## Reproduced independently

- The analyser at `14d9ecb`, replayed on the archive's records, reproduces its analysis
  exactly (this repository, `c30-analysis-replayed.json`).
- The guardian independently verified all 69 manifest-listed file checksums and recomputed
  the six paired ratios from the raw records; they match.

## The adverse descriptive evidence (from an invalid run; not a registered result)

| ratio, C ÷ P | point | 95% interval | registered PASS limit |
|---|---:|---:|---:|
| warm b4, four-worker CPU cost | 1.31583 | [1.3108, 1.3208] | upper ≤ 1.06714 |
| warm e4, four-worker completion | 1.36409 | [1.3597, 1.3685] | upper ≤ 1.02 |
| warm b4, full blocks only | 1.31506 | [1.3101, 1.3201] | (reported) |

Every measured warm four-worker round had a cost ratio between about 1.295 and 1.336, and the
full-block-only figure rules out the 204 short blocks as the explanation.

**An illustrative model calculation**, not a measurement: putting this invalid run's warm
four-worker construction ratio into the earlier planning model, beside the validated
traversal ratio, gives about 0.108 CPU-seconds saved per decoded GB, against the 0.25 CPU-s/GB
investment target. It uses invalid-run construction data, and it is not a measured decoder
saving.

## A correction to the review's record

The review states that the last authorisation covered the dry run only. The capture was
authorised: report 200 (in
[2026-11-03-compact30-construction-i9-dry-run.md](2026-11-03-compact30-construction-i9-dry-run.md))
authorised "one registered 20-round i9 construction-cost capture" with exactly the command
that was run and the verified dry-run archive `f75b00e8…40f9f9`. The capture was run once,
as authorised, and not repeated.

## Not authorised by this review

No i9 run, constructor change, walker tuning, gate change, production integration, or claim
of a measured end-to-end decoder result. No further experiment is authorised.

## Report 202: the guardian's final disposition (as relayed)

The guardian reviewed the archived capture report at `4adabf1`, and closes this line of work:

| | Conclusion |
|---|---|
| Formal experimental outcome | INVALID — G1 stability and G2 synchronization failed |
| Engineering investment decision | STOP — the observed construction overhead is far beyond the economic budget |

The distinction matters. We must not relabel INVALID as FAIL. Equally, an invalid capture doesn't oblige us to spend more workstation time pursuing a formal FAIL when the directional evidence is this consistent.

The warm four-worker cost ratio was 1.316×, against an allowable 1.067×. Completion was 1.364×, against 1.02×. Every measured round showed a substantial disadvantage, including the full-block workload. The isolated G2 interruption and unexplained one-worker production variability are worth documenting, but neither provides a compelling reason to expect the four-worker construction disadvantage to disappear.

The results that remain, kept separate:

- **Compact30 traversal: VALID ADVANCE.** Its four-worker advantage is real within the frozen traversal experiment.
- **Direct compact30 constructor:** correct, independently audited, but economically unpromising on the measured workload.
- **Whole-decoder performance:** not established.

**Final disposition:** Formal outcome INVALID. Engineering decision: STOP this direct-constructor investment. The archived evidence and its reported validity failures are accepted, and the capture is not relabelled as FAIL. The observed four-worker construction disadvantage is sufficiently large and consistent that another capture is not justified for this candidate. The closed VALID ADVANCE traversal result and the audited constructor are preserved as research evidence.

**Not authorised:** further i9 runs, investigation of the W1 bimodality for this investment, constructor tuning, walker changes or production integration. No new experiment is requested. This line of work is closed.
