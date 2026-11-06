# 1. Scaling beyond eight threads on the i9

- **Doing:** checking repaired evidence collection for an implemented production policy.
- **In order to:** 0, faster correct production decoding. Recoverable headroom and its mechanism remain questions.
- **Status:** implementation open; benchmark not frozen and no result recorded, pending Researcher and Guardian re-review. No adoption. Mechanism and other performance investigations have no new recorded dispatch.
- **Known:** the [team-size screen](https://github.com/lmmx/mibz2/blob/3421fe600f544724425db665caf06fa89dd25f06/docs/journal/2026-11-06-team-size.md) completed 320 runs with matching output, a complete grid and an unflagged control. Its registered primaries returned 1.017 (95% interval 1.008–1.026) and 1.003 (0.993–1.014), below the 1.03 minimum. The run was done as intended; neither primary qualified. Both point estimates fell within their broad predicted ranges. No confirmation was due for either failed primary.
- **Exploration, not qualification:** three teams at 10 and 12 requested threads returned 1.075 and 1.097, with about 18% less peak memory. These cells were selected after seeing the data. The [screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md) separates table-count association from causation: concurrency and window change too; the last-level cache explanation is a hypothesis. Throughput divided by tables is not measured block service rate.

## 1.1 Three-team confirmation (#220)

- **Doing:** completed two registered runs comparing three groups of four at 12 threads and three groups of three at 10 requested threads (nine decoding threads) with four groups of two at eight.
- **In order to:** 1, identify production-policy candidates.
- **Status:** done as intended, as recorded in the [result journal](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#result); both arrangements qualify under the unchanged rule. This records the Coordinator's analysis, not a fresh Oracle adjudication.

| Arrangement | Run 1 throughput ratio (95% interval) | Run 2 throughput ratio (95% interval) | Peak memory, runs 1 / 2 | Processor time, runs 1 / 2 |
|---|---|---|---|---|
| Three groups of four | 1.090 (1.072–1.107) | 1.086 (1.062–1.111) | 0.82 / 0.83 | 1.04 / 1.04 |
| Three groups of three | 1.076 (1.058–1.095) | 1.087 (1.074–1.100) | 0.82 / 0.82 | 0.90 / 0.89 |

- **Prediction separate from qualification:** groups of four were predicted at 1.04–1.09 and both point estimates lie in that range; groups of three were predicted at 1.03–1.07 and both exceeded its upper end. Memory and processor-time directions agree with predictions. No mechanism follows from that agreement.
- **Rule and outcome:** the same cell passed all predicates and resource limits twice; no switching, pooling or replacement. Both qualify, but the rule neither ranks them nor adopts either. Scope: i9-10900K and the level-9 Wikidata corpus. The nominal fixed-run error bound is 0.05 for this experiment under its model, not all future work ([rule and outcome](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md)).
- **Provenance:** Louis handed over confirmation bundle 20261106T190811Z, collected at a3880ce8 with executable inputs unchanged from freeze 656d2794. The Coordinator records report reproduction and independent recomputation from fresh raw runs. Hardware counters were conditionally unavailable in both runs. This account does not independently recalculate the data.
- **Diagnostics:** placement effects remain whole-placement comparisons. The journal has corrected its tracing and sibling-sharing wording. Neither run establishes the table-count mechanism.

## 1.4 Production-policy comparison

- **Doing:** reviewing evidence-path repairs before freezing a same-requested-thread-count comparison of new and old defaults.
- **In order to:** 1 and 0, translate the candidate gains into a safely validated production improvement.
- **Status:** design completed; implementation open at [7c341a875049f8c95d27448046eb4923387687cf](https://github.com/lmmx/mibz2/pull/234). Candidate and base are explicitly unfrozen. Researcher and Guardian re-review requested; neither acceptance nor a collected result is covered by this version.
- **Design and premise:** [Optimiser's report](https://github.com/lmmx/mibz2/blob/9101edf77c28fc6d52af0951abbf76e4ec9df490/state/records/reviews/inbox-6693507432/report.md) selects three groups of three for ten physical cores, 20 MiB last-level cache and at least ten requested threads; default window twelve blocks. All other selections retain the old rule, including four groups of two at eight/nine requested threads. Similar measured speed with less processor time motivates the choice; the earlier experiment did not statistically rank three-by-three above three-by-four.
- **Scope:** higher requested counts and other machines with the same detected topology are extrapolations, not findings. The selection does not check CPU model. Unmeasured counts, other workloads, custom windows and contention remain disclosed limits.
- **Admission route:** Louis's [recorded, policy-specific decision](https://github.com/lmmx/mibz2/blob/7c341a875049f8c95d27448046eb4923387687cf/docs/journal/decisions.md) permits workstation benefit evidence, automated no-regression evidence and correctness tests that fake the ten-core topology. It does not change all future admission rules. This decision is on the unmerged policy branch, not yet master.
- **Registered comparison:** [journal](https://github.com/lmmx/mibz2/blob/7c341a875049f8c95d27448046eb4923387687cf/docs/journal/2026-11-06-three-team-policy.md) and recipe compare new versus old defaults at equal requested counts, unpinned, twenty rounds at 8, 9, 10, 11, 12, 16 and 20. Ten is primary: throughput ratio at least 1.03 and lower 95% bound above 1, peak-memory ratio at most 1.15 and processor-time ratio at most 1.5. Four higher-count no-harm checks require simultaneous upper wall-time ratios at most 1.02 (Bonferroni correction); 8/9 are drift screens, not equivalence proofs.
- **Run versus outcome:** two valid slots, initial plus confirmation; at most one structurally invalid replacement across them, at most three attempts, all retained. A valid negative result cannot be replaced. No pooling. No freeze or run is recorded at this pin.
- **Why held:** [Researcher HOLD](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694256616) found missing/mixed evidence could pass and the baseline could move; [Guardian D036](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694299257) additionally required actual-plan evidence and retention of invalid attempts. These hold freeze/admission evidence, not the statistical rule or unrelated work.
- **Repairs versus acceptance:** [Coordinator's repair log](https://github.com/lmmx/mibz2/pull/145#issuecomment-6694798888) reports schema/hash/metric rejection tests; exact candidate/base and cleanliness checks; a full observed-thread census alongside the unchanged busy-thread list; and bundles/reports for failed attempts. The block window is inferred from frozen source and observed plan, not directly observed. Both reviewers must decide whether the repairs meet their findings; the Oracle does not lift them.
- **Next dependency:** re-reviews, then exact freeze, then Louis's workstation collection and confirmation. No restart of the separate paused investigations follows.

## 1.2 Mechanism

- **Doing:** no newly dispatched experiment; purpose is to explain the slowdown so any intervention targets its cause.
- **In order to:** 1 directly, whatever 1.1 finds.
- **Status:** paused by the Coordinator ([6683240055](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683240055), qualified by [6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024)).
- **Known:** the Optimiser's active-set analysis identifies roughly 3.6 MB of randomly read T-vector per full block plus written segment prefixes, rather than treating all retained buffers as active cache use ([6683232182](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683232182)).
- **Premise retracted:** a 25% footprint reduction equalises four teams with three; it is not necessary for a benefit. The Coordinator withdrew it as a reopening threshold after the Researcher's counterexample. Only the dispatch pause survives. C15's measured failure remains; no i9 gain is implied ([6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024)).

## 1.3 Placement and pipelining

- **Doing:** no new dispatch.
- **In order to:** 1, use cores or overlap differently.
- **Status:** paused by the same Coordinator decision, not blocked by 1.1.
- **Premise:** placement has an observed packed/spread whole-process ratio of 0.778; physical-core count changes too, so it is not isolated sibling-sharing cost. Stage pipelining still needs a quantified recoverable-time premise. Neither is qualified for adoption ([screen and corrected interpretation](https://github.com/lmmx/mibz2/blob/838d765c7de4956ada5a29c61ee30a436b1cf605/docs/journal/2026-11-06-team-size.md)).

## Accepted, not aimed at

Both registered runs saved about 17–18% peak memory while pursuing throughput. That accepted side effect accompanies the qualified candidates; it is not independent approval for production adoption ([outcome](https://github.com/lmmx/mibz2/blob/544462881497fc957fe17c35ae7508ca1bac23d5/docs/journal/2026-11-06-three-tables.md#outcome)).
