# External research report 128: the independent researcher's addendum: the standards of evidence behind the P87 selection (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 16:40 UTC.
- **Source:** the researcher.
- **In reply to:** the maintainer's request: "I am consolidating a record of what just happened in this past few steps in which both claude and astra worked and then both the guardian and you (the researcher) gave feedback on it. here's what the guardian wants to record, what would you like to record? I am going to store both of your answers via claude in the repo" (with report 127).

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment: a proposal and its review, not a performance result.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
Guardian's record is comprehensive. I wouldn't write a second version of the same history. Instead, I'd preserve a separate independent researcher addendum: what I verified, which interpretations I challenged, why I ultimately preferred Astra's proposal, and what future researchers must not accidentally treat as established fact.

That distinction is worth retaining. The record should make it possible for someone revisiting `mibz2` months later to understand not just what we decided, but why the competing evidence did not justify a different decision.

Here's the entry I'd want Claude to preserve verbatim.

# Independent researcher's record: The inverse-BWT investigation and P87 selection

**Date:** 1 November 2026\
**Role:** Independent performance research and adversarial review\
**Status:** P87 selected for diagnostic preparation; no timed P87 experiment completed or authorized at the time of this record.

## Purpose of this record

This is a companion to Guardian's consolidated technical history, not a replacement for it.

Its purpose is to preserve the independent researcher's evidentiary judgments: which findings were independently checked, which attractive explanations exceeded their supporting measurements, which disagreements between Claude and Astra remain meaningful, and why the P87 buffer-lifetime experiment was recommended first.

The central methodological principle is:

**A performance explanation is valuable only insofar as its evidence supports a reliable engineering decision. A plausible numerical model, a strong correlation and a successful intervention are three different things.**

## 1. The evidence that materially changed the research direction

The original shipped-decoder attribution experiment measured a rise from approximately 2.707 to 3.579 CPU-s/GB between one and four workers: an additional 0.872 CPU-s/GB.

Retired instructions per output byte barely changed, whereas execution cycles increased substantially.

Approximately 0.68–0.72 CPU-s/GB of the incremental cost was attributable to inverse-BWT work and associated operations.

These results justify concentrating on the efficiency of concurrent inversion. They do not justify assigning the increase to LLC capacity alone.

The original interpretation had overstated several counter-derived conclusions:

- `OFFCORE_REQUESTS.ALL_DATA_RD` describes a mixed request population including prefetch traffic, not demand loads alone.
- The outstanding-request occupancy ratio is not a direct measurement of individual demand-load latency.
- Retired demand-load L3 hit and miss counts cannot characterize all offcore requests.
- The average 2.19 active T-vector phases measures phase overlap, not actual LLC residency.
- Increased memory-controller writes do not identify the responsible buffers or prove that their writebacks are on the traversal's critical path.

The strongest supported diagnosis remains that **concurrency makes inverse-BWT memory-related work substantially more expensive**. Cache residency, prefetch timeliness, request scheduling, dirty-line displacement and page translation remain possible interacting causes.

This uncertainty is not a weakness to conceal. It defines what the next experiment must resolve.

## 2. What Claude's new phase analysis established

Claude independently revisited the archived attribution traces. The phase results reproduced under independent review.

For 197 full-sized blocks, the phase interval between `WalkStart` and `InvertEnd` increased from approximately 1.234 to 1.806 ns per BWT row: a 1.46× increase, or 0.572 ns per row.

Including the shorter blocks and normalizing against final decoded output gave an approximately 0.609 CPU-s/GB increase across the whole corpus.

**These figures describe traversal plus segment assembly, not traversal alone.**

The existing phase markers include `SegScratch::assemble()` within that interval. Future documents must not silently relabel the entire increase as dependent pointer-chasing cost.

This distinction is important because earlier sampling found substantial L3-miss-associated stalls in `memmove`, despite assembly's relatively small standalone execution cost.

The defensible conclusion is that the broad walk-and-assembly phase is the principal measured contributor to the incremental cost. The exact division between traversal, assembly and their indirect cross-worker effects remains to be measured.

### Concurrent-walk association

Claude also found that the phase's slowdown increased with the number of simultaneously walking peers:

| Other workers walking | Observations | Median slowdown |
| --------------------- | ------------ | --------------- |
| 0                     | 81           | 1.294×          |
| 1                     | 878          | 1.408×          |
| 2                     | 982          | 1.535×          |
| 3                     | 29           | 1.658×          |

This is credible evidence of an association, not yet causal identification.

There are three reasons for restraint:

1. Longer walks have more opportunity to overlap other walks, introducing potential reverse causation.
2. The 1,970 observations repeat the same 197 block identities across ten runs; they are not 1,970 independent experimental units.
3. The extreme concurrency bins are sparsely represented, and the regression explains only part of the observed variation.

The evidence warrants investigating traversal-related shared-memory interference. It does not establish that concurrent construction is harmless or that walker-to-walker interference is exclusively responsible.

## 3. Why the six-outstanding-request model was not accepted as a measurement

Claude offered a numerically attractive explanation of the phase slowdown using approximately 0.93 offcore requests per row, a process-wide residence proxy rising from approximately 39 to 58 cycles, and an inferred six requests in flight.

The arithmetic was consistent.

However, the approximately 0.93 walker-specific request rate had not been directly measured. The residence proxy came from mixed, process-wide hardware counters. The timing denominator included segment assembly.

Most importantly, effective overlap was inferred using the same timing and request quantities that were subsequently used to explain the slowdown.

**The numerical fit was therefore not an independent validation of constant memory-level parallelism.**

The permissible statement is narrower:

> The observed increase in walk-plus-assembly CPU cost is consistent with slower memory-request servicing at roughly unchanged effective overlap, but walker-specific request rates, useful outstanding traversal operations and prefetch lead time have not been independently measured.

The same caution applies to the proposed 98% software-prefetch traffic share.

Comparing retired demand-load L3 events against all offcore data requests does not isolate software-prefetch transactions. The event populations differ, and some Intel prefetch categories combine software prefetches with hardware prefetches.

The decoder is demonstrably designed around software prefetching. The specific share of traffic successfully supplied because of those prefetches remains unresolved.

This correction matters because future proposals concerning prefetch distance, T-vector residency or request queueing must not inherit an unverified six-request or 98% baseline as fact.

## 4. The genuinely different contributions from Astra and Claude

The two proposals were developed independently and answered different questions.

**Astra proposed P87**, a controlled modification to the existing decoder: assemble segmented BWT output into the former last-column `ll` allocation after construction has finished reading it, instead of into the separate `rle` allocation.

The representation, walker, software-prefetch policy, segment generation and logical assembly copy remain unchanged.

This tests whether changing the reuse and dirtying history of an intermediate byte arena improves actual four-worker decoder economics.

**Claude proposed a controlled-neighbour harness**, measuring a target walker while the other three cores generated different kinds of interference: real decoding, traversal, construction, clean-footprint pressure, dirty writes, LLC request traffic and DRAM streaming.

This sought broader physical attribution and potentially larger architectural opportunities.

Both proposals were serious. The choice between them was not a vote on whose analysis was more sophisticated.

The decision criterion was **expected engineering information value relative to implementation cost, calibration risk and the possibility of a misleading causal conclusion**.

## 5. Why the neighbour matrix was not selected first

Claude's harness may eventually be the more informative investigation into the larger walker-scaling penalty.

However, its proposed workloads are not independent controls for individual hardware resources.

A clean cache holder does not necessarily retain its nominal data set in LLC. Increasing its access rate to preserve residency changes request pressure.

A dirty holder changes ownership requests, store activity, replacement and writebacks simultaneously.

A small-footprint L3 request generator can use different cache slices, dependencies and request patterns from the actual walker.

A DRAM-streaming workload can match aggregate bandwidth while differing in burst structure, bank conflicts and read/write turnaround.

The proposed real-decoder-neighbour calibration is also not identical to the shipped decoder's scheduling and phase overlap.

Claude's initial rule required that real-decoder neighbours reproduce at least 70% of the production walk penalty. That is a reasonable reason to stop and reconsider an unsuccessful calibration.

But failing that threshold would **not** establish that shared-memory interference is absent in the production decoder. It could merely establish that the synthetic harness failed to reproduce the relevant production conditions.

A future neighbour experiment should therefore treat calibration failure as *invalid representativeness*, not as proof against the memory hypothesis.

The retained research direction is to build a smaller, carefully calibrated harness first, then expand only when individual workload comparisons can alter an engineering decision.

Its outputs should be interpreted as sensitivities to controlled interference, not automatically as additive fractions of the production slowdown.

## 6. Why P87 was selected first

P87 was preferred because it is a narrower intervention on the real decoder with clearer engineering consequences.

The experiment asks whether one existing allocation can perform two nonoverlapping roles more efficiently than two separate allocations.

It deliberately preserves the assembly copy. This avoids conflating elimination of copy instructions with the cost of touching a distinct destination buffer.

The intervention cannot directly accelerate the same block's already-completed traversal. Any measured traversal improvement would have to arise through effects on other workers, subsequent blocks or persistent memory-system state.

That temporal separation is useful.

A successful P87 result would establish an actionable memory-lifetime improvement even if it did not uniquely distinguish LLC occupancy from dirty-line coalescing.

A negative result would close this specific four-core intervention without implying that all intermediate-buffer traffic or all cache contention is irrelevant.

There is no evidence-based point prediction of P87's benefit. Zero remains plausible. Its direct scope is limited compared with the full walk-plus-assembly scaling penalty.

**P87 was selected because its result is more directly actionable, not because it has been proved to address the dominant physical bottleneck.**

## 7. Two decisions must be made from the P87 experiment

Astra proposed a one-versus-four-worker difference-in-differences statistic:

[\
D=(C\_{A,4}-C\_{B,4})-(C\_{A,1}-C\_{B,1}),\
]

where A uses separate assembly storage, B reuses `ll`, and (C) is CPU-s/GB.

The proposed concurrency-materiality threshold is 0.10 CPU-s/GB.

The researcher identified that a positive (D) can occur even when B makes the decoder slower at four workers, provided it makes the one-worker configuration even slower.

Consequently, **two independent judgments are mandatory**:

**Mechanism decision:** Does B reduce the concurrency-specific CPU penalty by a material amount, with uncertainty assessed using independent paired executions?

**Engineering decision:** Does B improve actual four-worker full-decoder throughput and CPU-s/GB, while satisfying correctness and memory-footprint requirements?

A positive mechanism result cannot substitute for a real four-worker improvement.

Conversely, a useful local four-worker improvement should not be misrepresented as confirmation of a large concurrency-specific memory mechanism if (D) is small.

The original P87 research-register gate concerning the 6–10-worker occupancy curve and DRAM writes remains historical and separate. The proposed four-core diagnostic must not silently replace it.

## 8. Minimum controls before an i9 run

The researcher's recommended authorization boundary is protocol design and isolated diagnostic implementation preparation only.

Before timing, the protocol should establish:

- Identical builder and traversal semantics, with machine-code comparison sufficient to expose material instruction-layout differences.
- Separately measurable construction, segment traversal, assembly and RLE1/CRC phases.
- Equalized buffer capacities, allocation and initial page-residency conditions, without repeatedly warming an arena just to make its access history appear favourable.
- Preserved worker count, affinity, dispatch, admission window, ordering, software-prefetch policy and codec behaviour.
- Full correctness and CRC validation, including short-block fallback, repeated scratch reuse and capacity-growth cases.
- Paired A/B and A/A measurements, a prespecified confidence procedure, and separate mechanism and absolute engineering decisions.
- A distinction between diagnostic memory parity and any later production RSS saving.
- An untimed dry run and frozen evidence trail before a separately authorized i9 capture.

Optional detailed counter collection should follow the causal question. A specialised PMU facility must not become a requirement merely because its name sounds relevant.

A decline in DRAM writes without a decline in walk or decoder CPU cost is an informative negative result for this intervention. It does not prove that writebacks never affect inversion.

## 9. What remains open after selecting P87

Selection is not resolution.

The following remain genuine research questions:

- Which physical level supplies the T-vector accesses under concurrent traversal?
- How often do demand loads encounter prefetched lines still in flight?
- Does prefetch lead time become inadequate as memory service slows?
- How much of the slowdown reflects LLC residency versus request scheduling?
- Are the increased DRAM writebacks causal interference or consequences of other displacement?
- Is TLB or page-walk pressure material?
- Can a cheaper memory-lifetime change materially improve the decoder, or must the traversal representation eventually change?

A failed P87 experiment would not answer these questions collectively.

It would justify moving to a better-calibrated interference investigation, potentially followed by a targeted prefetch-distance, address-translation or representation experiment if the physical evidence warrants one.

The existing negative P74-C, unrolled-walker, compact-Ψ and pair-walk implementations remain closed. They must not be revived without a genuinely new mechanism and a complete construction-plus-traversal economic case.

## 10. Relationship to the dispatch investigation

The earlier dispatch-causality capture met its registered validity requirements and rejected immediate MPMC work: direct receiver obstruction averaged 1.90% of aggregate worker time, with zero of ten runs meeting the 5% materiality threshold.

That result must not be generalized into a claim that all dispatch scheduling effects are negligible.

The post-hoc traces exposed substantial runnable delay and wake-affine placement that remain independently testable.

Those findings are retained as a separate secondary research lead, not mixed into P87 or the proposed memory-interference experiment.

This separation matters because changing thread placement and changing buffer lifetime together would make a positive result difficult to attribute.

## 11. Final research decision

**Recommended first experiment:** The isolated four-core P87 buffer-lifetime A/B diagnostic.

**Required safeguards:** Distinguish the difference-in-differences mechanism result from absolute four-worker improvement; separate traversal from assembly; preserve machine-code comparability and equalized allocation conditions; retain the original P87 historical gate.

**Deferred experiment:** Claude's controlled-neighbour interference matrix, with stronger production-representativeness checks and more carefully interpreted synthetic controls.

**Not authorized by this recommendation:** A new i9 timing run, production buffer-reuse deployment, compact-Ψ redevelopment, prefetch retuning, MPMC, condition-variable splitting or worker-affinity changes.

The researcher's final position is:

> The next experiment should establish whether a real, inexpensive buffer-lifetime intervention improves the shipped four-worker decoder. It need not solve the entire cache hierarchy to justify engineering work. But no numerical fit, observed phase overlap or reduced hardware-counter value should be promoted into a causal claim that the measurements cannot support.

This is why P87 was chosen first, and what its eventual result must be allowed to prove—or fail to prove.

---

### Source records

- Shipped-decoder attribution and erratum: `docs/decoder-attribution-records`
- Claude's independent proposal, scripts and phase evidence: `docs/bwt-memory-proposal`
- Historical P87 registration: `docs/research-portfolio`, research-register entry P87
- Completed dispatch diagnostic and exploratory scheduling findings: `docs/dispatch-causality-records`
- Independent researcher and Astra messages: external reports preserved through PR #120

**Record status:** Independent assessment completed. P87 preparation recommended; its causal and engineering outcomes remain unmeasured.

The difference between these two records is deliberate: Guardian's document tells the complete technical and chronological story; this addendum records the standards of evidence that governed the decision. I would preserve both, without merging away their different perspectives.
`````
