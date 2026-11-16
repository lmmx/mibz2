# Cap claim: direction confirmed, no uncapping gain established

Assignment [6926192217](https://github.com/lmmx/mibz2/pull/162#issuecomment-6926192217), narrowed by [6926214255](https://github.com/lmmx/mibz2/pull/162#issuecomment-6926214255). Independent arithmetic review, 16 November 2026. No timing, implementation or registration.

**Measured:** lazy-occupancy/run-20261116T032432Z, source commit **c2d1c68244a21396159b261e8bc0558f2920e0a8**, Wikidata ×8, requested 20 threads, ten paired rounds: **capped/uncapped wall time = 0.775636 [0.772849, 0.778433]** (two-sided 95% paired-log t interval). This is the 0.776 in [LOG 6906620039](https://github.com/lmmx/mibz2/pull/145#issuecomment-6906620039), cited by [MONODROMY 38](https://github.com/lmmx/mibz2/blob/454e868d198ad1f7dfa0b325c55e1fa32d412ddd/docs/journal/monodromy/2026-11-16-where-gains-are.md).

The cap uses **22.44% less time**; equivalently uncapped delivers 22.44% less throughput, or takes **28.93% more time** (uncapped/cap **1.289264 [1.284632, 1.293914]**). “Uncapped loses 22%” needs that denominator. The original checkpoint reversed the direction; its correction restores it.

## Exact arms and baseline limits

At that source pin, `examples/ab.rs` implements `mibz2-workers-lazy-cap` as min(requested, physical cores) lazy workers with four slots per worker: **10 workers / 40 slots**. `mibz2-workers-lazy` uses requested workers and four slots each: **20 / 80**. Raw observed thread names confirm ten versus twenty workers in every main comparison. Both have the same output byte count/hash and successful termination.

The then-shipped `mibz2` arm was a **team plan** above the physical cores, not today's capped production. Its losses to the capped candidate are historical adoption evidence, not a newly available gain over production. Furthermore, at stop-walk commit f3baa922, production has ten workers but retains the requested-thread window (80 slots at requested20); the older 10/40 candidate is not identical to that 10/80 production plan. Thus 0.776 is an exact measured **plan** comparison, not an isolated worker-count effect or an exact contemporary-production ratio.

## Does any tested count 10–20 beat capped production at requested11–20?

**No such gain is established in these bundles.** Keep the existing #481 cores-less-one question separate; no new registration is justified by the reversed claim.

| Source bundle | Relevant measured contrast |
|---|---|
| lazy-cap/024424, commit 67905f2d2ce9104ea83513baad07f1113a49925b | Cap/uncapped at requested20: **0.775180 [0.760004, 0.790659]**. Cap also wins at12/14/16. Gains against its “shipped” arm are against the older team baseline. |
| lazy-crossover/014443, commit c3ceea21b0ac449674ac7807e95c02d9bdc489c7 | Uncapped/then-shipped at12: 1.001 [0.974,1.029]; at14/16/20 slower. |
| lazy-crossover/021858, commit dab66ed212a6eb587165dfb334d147c902cbc22c | Uncapped/then-shipped at12/14/16/20 all slower; at20 **1.214 [1.192,1.235]**. |
| lazy-crossover/093528, commit 809515d550bce621f0a64e99c765631f2b50cbc9 | Against the later capped production, uncapped at12 **1.089 [1.064,1.115]**,14 **1.160 [1.140,1.181]**,16 **1.194 [1.167,1.222]**,20 **1.265 [1.245,1.287]**. |
| stop-walk/162328, commit f3baa922a25c2144b82d98c3d88c8ec298d1333b | Forced20 stop15/forced20 lazy **0.965 [0.957,0.973]** within its paired diagnostic. This improves a losing uncapped baseline, not production. |

Re-pairing stop-walk forced and judged records by round number reproduces the quoted forced20 stop15/production **1.260332** (descriptive interval **[1.247479,1.273318]**); forced20 lazy/production is **1.306139**. Forced12 stop15/production is **1.065432**. **These cross-phase intervals are descriptive only:** forced and judged phases were separate, not a randomized within-round production comparison. No cross-bundle pooling is used.

The stop-walk judged production-plan experiment separately finds no significant gain (Holm rejects no main-corpus cell). These results do not exhaust worker counts 11,13,15,17,18,19, or independently compare every fixed cap at every requested thread count. Nor do they reject a window-only change. #481 studies nine workers plus a smaller window at requested12/16/20, not uncapping to 11–20; its scoped review remains [6924511372](https://github.com/lmmx/mibz2/pull/145#issuecomment-6924511372). No registration draft or toolchain migration decision follows from this check.

## Evidence and replay

All six source archives were fetched as base64 and decoded; every archive SHA-256 matched its companion digest. See `evidence.json` for archive paths, Git blob identities and full SHA-256. Immutable evidence revision: [14b969f4ef2d9c8da4c7ac77643babbddc28b6c6](https://github.com/lmmx/mibz2/tree/14b969f4ef2d9c8da4c7ac77643babbddc28b6c6).

`replay.py` independently reconstructs paired geometric means and intervals from archived raw JSONL (t9=2.262, matching pinned ab.rs), checks ten complete round pairs, successful termination and within-corpus byte/hash identity. `replay.json` preserves results. The source bundles remain the evidence; this is arithmetic replay, not new execution evidence. The five earlier archives are supplementary comparisons; the exact 0.776 provenance is the occupancy archive.
