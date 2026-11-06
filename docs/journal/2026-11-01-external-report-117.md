# External research report 117: the guardian's consolidated prompt: close dispatch, propose one discriminating inverse-BWT memory experiment (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** report 116.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
Here is the complete, consolidated prompt for Claude Code, incorporating the researcher's refinements. It covers closing the dispatch investigation, preserving the new scheduler findings, and preparing a genuinely discriminating inverse-BWT memory experiment.

You can send it as-is. No additional instructions are needed.

We've completed the i9 dispatch-causality investigation, and both the independent researcher and Guardian agree on the next direction.

I want you to close and archive the dispatch experiment properly, then prepare **one bounded, causally discriminating inverse-BWT memory-system experiment proposal**.

The goal is not another broad performance survey or a list of plausible optimizations. It is to identify which physical cost we can actually remove from four-worker decoding, and which engineering approach that evidence would justify.

**Do not implement or run a new experiment yet.**

## Part 1 — Close and archive dispatch causality

The frozen i9 capture completed with all registered validity checks passing.

Its registered outcome is **`do not proceed`** for MPMC receiver replacement:

- Mean direct receiver obstruction: **1.90%** of aggregate worker time.
- Registered materiality threshold: **5%**.
- Runs meeting that threshold: **0 of 10**.
- Receiver-associated exploratory share: **9.95%**, including runnable-but-unscheduled time. This is not a second acceptance criterion and must not be counted as direct mutex obstruction.

Please independently reproduce the frozen result from the raw archive, confirm its checks and preserve the full evidence in the research records. Do not change the verdict, analysis thresholds, or archived measurements.

Also independently verify and document the researcher's additional exploratory findings:

- Approximately **8.05%** of aggregate worker time is runnable in the receiver-acquisition phase.
- Approximately **5.65 percentage points** of that time coincide with at least one idle allocated CPU.
- Approximately **1.06%** of aggregate worker time is runnable in the receive phase.
- Across 4,523 matched worker wake-ups, median wake-to-running delay is approximately 40 µs when an allocated CPU is idle and approximately 10 µs when all four are occupied, but the busy-core case has a much longer tail.
- For 962 fully reconstructed reader handoffs, publication-to-reader-wake and lock-reacquisition intervals are small relative to wake-to-running delay. Another 331 handoffs remain unresolved.

Treat all these as exploratory measurements, not registered proof of a recoverable throughput gain. Explain their limitations, especially that runnable-but-not-running time is not necessarily wasted CPU capacity.

If useful, perform a brief **read-only** inspection of the already-captured `sched_waking.target_cpu` and CPU-idle events to see whether wake-up placement contributes to the observed delays. Do not launch another scheduling experiment.

The research decision is to **park both MPMC receiver replacement and the reader/scanner condition-variable split**. Neither has earned an implementation trial on the current evidence.

## Part 2 — Establish the inverse-BWT opportunity precisely

Our next primary research problem is the additional CPU cost of inverse BWT when scaling from one to four workers on the i9-10900K.

The shipped-decoder attribution experiment established:

- CPU-seconds/GB increases approximately **32.2%**.
- Cycles/decoded byte increases approximately **30.1%**.
- Retired instructions/decoded byte increases only **1.3%**.
- Frequency changes explain only approximately **5.9%** of the CPU-cost increase.
- Inverse BWT, including associated assembly copies and integrated kernel time, accounts for approximately **78–83% of the added CPU cost**. The narrower build-plus-walk function-only estimate is approximately 66.5%.

**Important quantitative distinction:** 78–83% refers to the *incremental scaling cost*, not 78–83% of total decoder CPU time.

The corresponding inverse-BWT scaling penalty is approximately **0.68–0.72 CPU-seconds/GB**, or roughly **19–20% of the entire four-worker decoder's CPU cost**.

Use that incremental cost as the opportunity budget. Do not exaggerate potential gains by applying the 78–83% attribution to the entire decoder.

Recovering the whole penalty would still not automatically yield 1.5× throughput: core utilization would also need to improve. Our longer-term objective remains 1.5–2× full-decoder throughput without increasing CPU-seconds/GB or peak RSS.

Initially, **preserve the existing efficient four-byte T-vector/Ψ representation**. We want to understand why an already-efficient inversion algorithm becomes more expensive under concurrency before committing to another representation.

## Part 3 — Distinguish the competing physical mechanisms

Investigate five leading hypotheses, using the existing source, archived counter results and prior research records.

For each hypothesis, explicitly state:

1. What the existing measurements directly establish.
2. What remains inferred or unidentifiable.
3. What competing mechanism could produce the same observations.
4. The smallest measurement or controlled experiment that would discriminate between them.
5. What result would support or falsify that hypothesis.

The mechanisms are:

**H1 — T-vector cache residency and eviction**

Concurrent inversion may increase the cost of demand accesses to the T-vector because useful cache lines are displaced.

Distinguish actual demand-access costs from general L3 misses and cache-capacity estimates. Average active T-vector phases are not equivalent to measured cache residency.

**H2 — Software-prefetch interaction**

The 32-lane walker prefetches successor rows. Prefetch traffic may hide dependent-load latency, but can also consume fill buffers, cache capacity, request bandwidth or uncore resources.

Distinguish beneficial prefetch overlap from unnecessary traffic or eviction. Do not treat `OFFCORE_REQUESTS.ALL_DATA_RD` as a demand-load-only event: it includes prefetches. Do not treat the 41→58 cycles/request occupancy proxy as directly measured individual demand-load latency.

**H3 — Intermediate-buffer writes and dirty-line eviction**

Segment assembly and subsequent decoding may generate write allocations and dirty-line traffic that compete with inversion's critical accesses.

Four-worker DRAM writes increased markedly in the earlier attribution experiment, but **increased write traffic does not prove those writes caused the slowdown**.

Establish what buffers or phases plausibly generate the writes, and which measurement would show whether those writes materially delay useful inversion.

**H4 — Overlap between BWT construction and traversal**

Concurrent workers may be constructing T-vectors while others traverse them. The combined traffic and cache effects could exceed what either phase causes independently.

Distinguish build/build, walk/walk and build/walk overlap, accounting for the actual lifetimes and sizes of their working sets.

**H5 — Memory-controller and uncore request contention**

Latency or execution stalls may increase because of request scheduling, bank conflicts, read/write turnaround, queueing or related resource contention, even without aggregate DRAM bandwidth saturation.

Separate this from LLC misses and dirty-line eviction where the hardware events permit. Identify which conclusions the i9's PMU can and cannot support.

These mechanisms **are not necessarily mutually exclusive**. Dirty-line traffic and higher L3 miss rates could be consequences of the same working-set pressure.

Do not force a single explanation or sum overlapping stall counters.

## Part 4 — Design one experiment that changes our engineering decision

Review the existing instrumentation and research history before proposing new measurement infrastructure.

In particular, account for:

- The shipped one-pass T-vector construction.
- Historical prefetch experiments, including ineffective L2 hints and regressions from disabled/non-temporal prefetch.
- Earlier segment-assembly/copy-elimination experiments and their relatively small direct benefits.
- Pair walking's poor one-to-four-worker economics.
- Compact Ψ's extra per-core decoding costs.
- N1's unresolved construction, traversal and retained-memory economics.
- The negative P74-C and unrolled-walker full-decoder results.

Do not simply revive one of those experiments without a genuinely new mechanism and supporting evidence.

**Choose one preferred next experiment with high causal discrimination and high decision value.**

Before recommending it, specify how different outcomes would affect our engineering direction:

- **Retain the four-byte T-vector and reduce memory traffic** if measurements implicate avoidable prefetch, copy, write-allocation or dirty-line interference.
- **Change the scheduling or ownership of build/walk phases** if overlap between construction and traversal creates a material extra cost.
- **Reconsider compact or composed Ψ, including N1**, only if the evidence indicates that the existing representation's unavoidable lookup/footprint costs dominate and a replacement has a credible construction-plus-walk CPU and RSS budget.
- **Investigate another measured owner** if none of these hypotheses explains a material part of the scaling penalty.

Make the proposed experiment discriminating enough that plausible results would select between those approaches, rather than merely leaving all five explanations standing.

For the chosen experiment, provide:

1. A precise causal hypothesis and its strongest alternative explanation.
2. The smallest instrumentation or controlled intervention needed.
3. Available and unavailable hardware events, with correct Intel event semantics.
4. The target workload, one-versus-four-worker allocation and suitable controls.
5. Whether measurements can reuse existing binaries and archived data, or require a separate registered capture.
6. Expected signatures under the competing hypotheses.
7. Prespecified early falsification and quality criteria.
8. How CPU-seconds/GB, full-decoder throughput, peak RSS and correctness will be protected or evaluated.
9. The engineering actions each possible result would justify.

Quantify potential recovery against the measured **0.68–0.72 CPU-s/GB incremental inversion cost**, not against total decoder time.

Distinguish **correlation from causation** throughout. A counter movement alone is not proof of the resource limiting the critical path. Where an intervention changes multiple resources, explain what causal interpretation remains possible.

## Part 5 — Deliverables and authorization boundary

Please produce:

- A complete, reproducible dispatch-causality archive and postmortem, with the registered `do not proceed` outcome preserved.
- Verification or correction of the researcher's exploratory scheduling and reader figures, clearly separated from the registered verdict.
- A concise review of what existing inverse-BWT evidence establishes and where the causal gaps remain.
- **One preferred inverse-BWT memory-system experiment proposal**, with an explicit hypothesis-discrimination and engineering-decision matrix.
- A proposed frozen protocol, resource requirements and i9 runner command for that experiment, if new measurement is necessary.
- A recommendation on whether the proposed experiment is worth authorizing, and exactly what we expect to learn.

**Current authorization is read-only research and documentation only.**

Do not implement new instrumentation or an optimization, run a new benchmark, launch CI, modify production dispatch, change any experimental gate, or revive a closed trial.

Stop after presenting the research proposal for review.

The principle guiding this next stage is:

**We are no longer asking which optimization looks promising. We are asking which physical cost can be demonstrably removed from the four-worker decoder, and which experiment will establish that before we invest in engineering it.**

This is ready to send to Claude as one message. It keeps the larger 1.5–2× ambition in view while ensuring that the next experiment has a precise, falsifiable purpose.
````
