# External research report 130: the maintainer's brief to a new Astra session: a research-architecture review for the next throughput breakthrough (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01.
- **Source:** the maintainer. This is the prompt given to a new Astra session (Codex, at its
  maximum reasoning effort), recorded at the maintainer's request.
- **In reply to:** the P87 four-core result and its post-capture record (#122), and the
  change of mandate toward larger architectural gains (the ranked plan, #129).

**Evidence cutoff:** 1 November 2026. This is after the P87 four-core i9 capture and its
post-capture record, and before any response from this Astra session. It is a brief, not a
performance result.

It is kept exactly as given.

## The prompt, as given

`````text
# Research architecture review: finding the next major mibz2 throughput breakthrough

You are conducting a fresh, independent, high-reasoning architectural review of `lmmx/mibz2`, a high-performance parallel bzip2 decoder.

**Your assignment is to exercise research taste, not merely to suggest another incremental optimization.**

We have accumulated an unusually detailed body of measured successes, negative experiments, architecture investigations, independent critiques and frozen protocols. The experimental process is mature. The missing ingredient may be identifying a fundamentally more promising direction.

We are looking for a credible route toward **1.5–2× the current four-physical-core decoding throughput**, while maintaining or improving CPU efficiency and avoiding unacceptable memory growth.

An earlier Astra investigation proposed the P87 intermediate-buffer-reuse experiment. That was a useful intervention: it produced a positive measured result, although it did not meet its registered production-promotion gates. This is a **new Astra session** so that you can reconsider the architectural search space independently rather than defend earlier recommendations.

Do not assume our current priorities are correct. Challenge them.

## 1. Current decoder and performance objective

Repository: https://github.com/lmmx/mibz2

The principal test platform is an Intel Core i9-10900K with four allocated physical cores, using the Wikidata decoding workload.

The shipped decoder uses:

- Parallel decoding of independent bzip2 blocks.
- A four-byte-per-row inverse-BWT navigation table (T-vector).
- 32 independent, interleaved dependent traversal chains per worker.
- Software prefetching to overlap dependent memory lookups.
- A fused, efficient T-vector builder.
- Segmented traversal followed by segment assembly.
- Persistent scratch buffers for MTF output, T-vector construction, traversal segments and RLE1 input.

A full-sized block has 900,000 BWT rows. Its four-byte T-vector therefore occupies approximately 3.6 MB.

Four simultaneously live full-sized T-vectors nominally occupy 14.4 MB. The i9 has a 20 MB shared L3 cache, but other scratch allocations, cache replacement, simultaneous activity and effective residency must be considered.

Do not infer actual cache residency merely by comparing nominal footprint with nominal cache capacity.

### Measured baseline

Four workers:

- Approximately 869–870 MB/s decoded throughput.
- 3.579 CPU-seconds per decoded GB.
- Approximately 3.11–3.14 cores of aggregate CPU activity out of four allocated physical cores.

One worker:

- 2.707 CPU-seconds per decoded GB.

Moving from one to four workers:

- CPU cost per decoded byte rises approximately 32.2%.
- Cycles per output byte rise from approximately 13.46 to 17.51.
- Instructions per output byte rise only from approximately 32.63 to 33.06.
- IPC falls approximately 2.42 to 1.89.
- Clock frequency changes only modestly.

The decoder largely executes the same instructions but consumes many more cycles per output byte.

The initial full-decoder attribution placed approximately 0.68–0.72 CPU-s/GB of the additional four-worker cost in inverse BWT and associated operations.

A separate whole-corpus reconstruction measured approximately 0.61 CPU-s/GB of incremental traversal-plus-segment-assembly cost. It is essential that you retain this qualification: the original recorded walk interval includes assembly; it is not an isolated pointer-chasing measurement.

### The systems-level performance ceiling

We need to distinguish CPU efficiency from CPU utilization.

Approximate, illustrative scenarios using the original baseline:

| Scenario | Throughput | Relative |
|---|---:|---:|
| Current four-worker decoder | 870 MB/s | 1.00× |
| Four cores fully utilized at current CPU cost | 1,118 MB/s | 1.29× |
| One-worker CPU efficiency at existing utilization | 1,149 MB/s | 1.32× |
| One-worker efficiency and four fully utilized cores | 1,478 MB/s | 1.70× |

These are simplified ceilings, not predictions or additive benefits.

To reach 1.5× baseline throughput, we need approximately 1,305 MB/s. At four fully utilized cores, that requires CPU cost no higher than approximately 3.07 CPU-s/GB.

To reach 2× baseline throughput, approximately 1,740 MB/s, four fully utilized cores would require around 2.30 CPU-s/GB: about 15% lower cost than our measured one-worker decoder.

Therefore a 2× result probably cannot come solely from eliminating the observed concurrency penalty. It also requires improving underlying decoding economics and/or removing other bottlenecks.

You must reason about full-decoder throughput, CPU efficiency and utilization together.

## 2. What the memory investigations established

The earlier attribution found a large shift in memory-system behaviour under concurrency.

The corrected P87 diagnostic's baseline memory-controller measurements were approximately:

| Measurement | One worker | Four workers |
|---|---:|---:|
| DRAM reads per decoded byte | 0.38 B/B | 9.01 B/B |
| DRAM writes per decoded byte | 0.27 B/B | 6.69 B/B |

This is approximately a 24-fold change in DRAM reads per decoded output byte.

These are machine-wide, idle-adjusted memory-controller observations. They are not counts of individual T-vector demand loads, nor proof that most navigation entries are served directly from DRAM.

The earlier attribution also observed more retired-demand LLC misses and larger memory-stall indicators, while counts of mixed demand-and-prefetch requests leaving L2 remained approximately stable.

Possible physical mechanisms include:

- Reduced useful shared-cache residency.
- Cache-set and replacement interactions.
- Increased request-service or queueing latency.
- Software prefetches arriving too late.
- Finite memory-level parallelism or memory-request resources.
- Dirty-line eviction, ownership traffic and writeback interference.
- Construction, traversal and assembly phase overlap.
- TLB or page-walk effects.
- Combinations of several mechanisms.

These remain hypotheses.

An earlier estimate of approximately six useful requests in flight was based on an inferred model, not a direct walker-specific measurement. An asserted 98% software-prefetch share was unsupported by incompatible counter populations and was withdrawn.

Do not resurrect either as established fact.

### Observational phase-overlap analysis

Claude's reconstruction found that the measured walk-plus-assembly phase was more expensive when it overlapped additional workers' walks.

The association was approximately 1.29×, 1.41×, 1.54× and 1.66× for zero through three other workers walking.

Those observations were not independent randomized interventions. Longer walks are more likely to overlap other walks, the extreme bins are sparse, and observations repeat block identities.

The association motivates a causal experiment; it does not prove walker-to-walker interference.

## 3. P87: a completed, informative but modest intervention

The earlier Astra proposed reusing the last-column `ll` byte buffer as the destination for inverse-BWT segment assembly, after construction has consumed the original bytes.

The original decoder assembles into a separate persistent `rle` buffer.

The completed four-core diagnostic preserved the construction, traversal, prefetch and assembly kernels. Both arms used one compiled diagnostic path; only the chosen output allocation differed. Both allocations were retained and initially prefaulted to control footprint-related differences.

The results on the primary Wikidata workload at four workers were:

- Throughput B/A: 1.01348, or +1.35%, with 95% CI +0.29% to +2.41%.
- CPU cost B/A: 0.98286, or −1.71%, with 95% CI approximately −2.37% to −1.06%.
- Concurrency-specific difference-in-differences saving D: +0.078 CPU-s/GB, with 95% CI +0.042 to +0.114.
- Corrected DRAM reads: 9.01 to 8.11 bytes/output byte, approximately −10%.
- Corrected DRAM writes: 6.69 to 5.82 bytes/output byte, approximately −13%.

The phase split showed a resolved segment-assembly improvement. The dependent traversal improvement was estimated at approximately 0.036 CPU-s/GB at four workers, but its confidence interval included zero.

The formal registered outcomes were:

- Mechanism: INCONCLUSIVE, because D's confidence interval crossed the predeclared 0.10 CPU-s/GB materiality threshold.
- Engineering: NOT DEMONSTRATED, because five additional no-harm checks were inconclusive, despite a demonstrated primary Wikidata improvement.
- Historical six-to-ten-worker P87 criterion: NOT EVALUATED.

A later correction fixed a perf MiB-unit conversion bug that had understated the absolute DRAM figures by a factor of 16,384. The original archive and verdict were preserved, and the corrected figures were reproduced offline.

Interpretation:

P87 establishes that buffer-lifetime changes can improve the diagnostic decoder's four-worker economics and alter memory traffic substantially.

It does NOT show that buffer reuse directly removes the main dependent-traversal bottleneck.

It does NOT constitute a measured improvement against the shipped production binary.

It does NOT establish the physical cause of the traffic reduction or performance gain.

We may eventually conduct one separately registered production-shaped validation, reusing `ll` and removing `rle`. But a potential 1–2% improvement must not dominate our architectural research agenda.

Do not propose a long succession of additional P87 variants.

## 4. The original succinct-navigation research must be reconsidered

This is an important priority correction from the independent researcher.

Our investigation originally asked whether inverse BWT's mathematical structure could support a substantially more compact, efficiently traversable navigation representation.

We later became increasingly focused on diagnosing why the current four-byte T-vector behaves badly under concurrency.

Those are related but distinct research programmes.

The original succinct-structure work is still materially unfinished.

### Mathematical redundancy

The original census found that approximately 77.39% of rows in the measured full-block corpus belonged to simple consecutive navigation groups.

This implies that many navigation entries carry structural redundancy. It raises the possibility of storing navigation rules rather than explicit full-width links for every row.

But information-theoretic compactness is useful only if the encoding preserves fast navigation and affordable construction.

### Historical representation candidates

| Representation | Approximate space per full block | Evidence |
|---|---:|---|
| Shipped 32-bit T-vector | 3.60 MB | Fast baseline; next pointer and reconstructed byte recovered together |
| 24-bit links | 2.70 MB | Tested and substantially slower despite improved residency |
| P40 fixed 64-byte records | 1.82 MB | Census validated; construction and traversal not yet timed |
| Two-tier succinct directory | 0.64 MB on measured corpus | Very compact, but 12–36% of steps require a second dependent lookup; not timed |

The negative 24-bit result is particularly important. Its strongest tested configuration achieved only approximately 0.687× the reference kernel's throughput in that comparison, despite improving cache residency.

A key reason was additional dependent work associated with recovering the reconstructed byte separately from the navigation link.

**The important lesson is not that compact navigation is hopeless. It is that smaller representations must preserve cheap next-address and byte recovery.**

### Specific unfinished ideas

**P40 — fixed 64-byte records**

Explore organizing approximately 32 navigation entries inside each cache-line-sized record, with predictable record addressing and inexpensive recovery.

The proposed footprint is around half of the current T-vector.

The census was promising, but neither navigation nor construction was benchmarked.

It remains an unverified design, not an optimization result. You must inspect the actual representation and its exceptional cases rather than infer per-lookup cost from its nominal footprint.

**P38 — cheaper reconstructed-byte recovery**

The failure of 24-bit links suggests that next-pointer recovery and reconstructed-byte recovery must be designed together.

Investigate how the byte could be obtained without introducing another expensive dependent memory access.

A design with a smaller pointer but an additional random load per step is likely to repeat an already observed failure.

**P84 — direct succinct construction**

A compact representation should ideally be built directly from the BWT last column rather than by constructing a full 32-bit T-vector and then compressing it.

However, the shipped fused builder is highly optimized.

A new direct builder must justify its additional instructions, data passes, synchronization, temporary storage and RSS against the baseline. A smaller final output does not imply faster construction.

**Two-tier directory**

Its approximately 0.64 MB footprint is attractive, but 12–36% of navigation steps requiring an additional dependent lookup is a serious concern in a latency-sensitive walker.

Give this design its proper opportunity, but do not prioritize it purely because it has the best compression ratio.

### Your succinct-design assignment

Reassess P40, P38 and P84 as potential components of a single new architecture.

Do not assume they combine successfully. Investigate the actual mathematical invariants, required metadata, exceptional cases, byte recovery, record addressing, builder dataflow and branch or dependent-load behaviour.

Compare them to the existing shipped baseline and the actual previously failed variants.

For each viable candidate, estimate:

- Bytes per BWT row and total resident working set.
- Navigation instructions per reconstructed byte.
- Dependent versus independent memory loads per step.
- Cache lines accessed per useful step.
- Branching and exceptional-case frequency.
- Cost of reconstructing both the next link and decoded byte.
- Construction passes, CPU cycles and memory traffic.
- Temporary allocations and peak RSS.
- Effect on the existing 32-lane traversal scheme.
- Best-case and realistic full-decoder performance opportunity.

Critically, ask whether a record-based representation could become sufficiently compact without needing another dependent lookup.

The breakthrough might involve an entirely different representation from P40. Do not force the original proposal to win.

## 5. Negative experiments that must constrain new ideas

Read the actual historical experiments, not just their names.

Important results include:

**Dispatch**

Replacing the shared job receiver with an MPMC architecture was rejected under a registered causal threshold. Direct receiver obstruction was approximately 1.90% of aggregate worker time, below the criterion for pursuing that replacement.

Do not reopen that specific optimization without a new causal reason.

Runnable-but-not-running delays and Linux wake-placement effects remain separate, unresolved utilization hypotheses.

**Fully unrolled 32-lane walker**

This obtained a meaningful standalone speedup on Zen 3 but no material improvement to four-worker i9 throughput. The i9 result was approximately neutral, with somewhat worse CPU cost.

Standalone kernel speedups are not sufficient.

**Compact-Ψ and composed variants**

Several variants reduced representation size but paid heavily in build cost, access complexity, or peak memory. These paths were closed under their recorded criteria.

A revived succinct idea must address exactly which cost made the earlier implementations fail.

**Pair-walk**

Extra construction work and other overheads outweighed its potential traversal benefit.

**Prefetch hints**

L1 prefetch is shipped. Other hint configurations failed to deliver an i9 improvement, and non-temporal prefetch was substantially worse.

Prefetch distance has not been comprehensively explored. But another prefetch change must have a convincing model of service latency and useful overlap before commanding major engineering investment.

**Assembly-copy elimination**

Previously investigated separately, without a material production-scale win.

P87's positive diagnostic was a buffer-lifetime intervention that preserved the assembly copy, not a contradiction of that result.

Treat negative experiments as constraints on architectural proposals, not as evidence that architectural innovation is impossible.

## 6. Utilization must remain a first-class opportunity

Our current approximate aggregate CPU use is 3.11–3.14 physical cores of the four allocated.

Not all unused core time is necessarily recoverable by changing the same decoder component. Scanner, reader, feeder, worker wakeups, scheduling and pipeline admission may contribute.

Prior dispatch research identified runnable-but-not-running time that was not direct mutex obstruction.

Potential high-impact directions include:

- Better separation of active decode work from scanner/reader overhead.
- Pipeline restructuring.
- Reducing scheduling gaps.
- Improved core placement and wakeup behaviour.
- Different block-level concurrency choices.
- Avoiding simultaneously active working sets that reduce total throughput.

However, every scheduling experiment must measure aggregate completed output, CPU cost, worker utilization and synchronization overhead.

A faster traversal that idles other cores is not a throughput optimization.

Consider whether representation, pipeline organization and utilization can be jointly improved.

For example, a more compact table might allow more useful overlapping work without increased memory interference. Conversely, a new decoding arrangement might change when full-sized tables coexist without requiring compression.

Do not assume these are beneficial; derive their cost and concurrency constraints.

## 7. The wider architectural hypotheses

We want you to reconsider at least these classes of opportunity:

A. Succinct or record-based inverse-BWT navigation with inexpensive byte recovery and direct construction.

B. Better useful cache residency and simultaneous working-set organization, possibly without changing the navigation representation.

C. Lower request-service cost, better software-prefetch timing, or increased useful independent memory work.

D. Phase scheduling, pipeline restructuring and changes to when workers compete for shared resources.

E. Fundamental changes in the algorithm, traversal order, output reconstruction or unit of parallelism that might eliminate expensive operations rather than merely move their data.

You may reject any of these or add others.

Do not constrain your reasoning to a binary choice between cache capacity and request queueing.

Question whether our current organization of inverse BWT is itself responsible for the large scaling penalty.

At the same time, respect bzip2's mathematical and codec correctness constraints.

A proposed transformation must preserve exact decoded output, including short blocks, run-length expansion, CRC and stream semantics.

## 8. What constitutes a breakthrough candidate?

Prioritize architectural ideas with a credible path to recovering approximately 0.25–0.50 CPU-s/GB of the original four-worker CPU penalty, or to substantially increasing useful four-core utilization.

These are prioritization targets, not acceptance thresholds or performance predictions.

For each candidate, build an approximate cost model.

Separate:

1. Directly measured quantities.
2. Quantities inferred from existing measurements.
3. Assumptions.
4. Optimistic upper bounds.
5. Your realistic expectation, including uncertainty.

Avoid adding incompatible gains from separate experiments or presenting a microbenchmark improvement as a full-decoder prediction.

Explicitly account for build time, traversal, assembly, RLE1, memory traffic, scanner/reader work and utilization.

If a proposed architecture cannot plausibly deliver a substantial full-decoder improvement, say so, however elegant its mathematical representation may be.

A clever data structure that is slower to build and traverse is not a breakthrough.

## 9. Your primary intellectual task: exercise research taste

We have been very rigorous about measurement. We have been less selective about how much potential upside a hypothesis must offer before receiving substantial engineering effort.

We are at risk of repeatedly validating 1% improvements while leaving the original four-worker cost largely intact.

We want you to identify ideas that are genuinely worth investigating.

Please answer these questions directly:

- What is the most important assumption our existing research programme may have accepted too readily?
- Are we focusing on a symptom of the current representation rather than the deeper decoding problem?
- Is the existing 32-lane T-vector walker actually close to the right architectural limit, or is there room for a different memory-access model?
- Have we prematurely abandoned the succinct-data-structure approach?
- Which of P40, P38 and P84 changes the opportunity most if implemented successfully?
- Is there a way to exploit the observed structural redundancy without paying another dependent load?
- What is the most credible way to recover 0.25 CPU-s/GB or more?
- Does the large DRAM-traffic transition indicate an actionable cache-capacity boundary, or could request pressure dominate despite similar residency?
- Could better utilization deliver more than another traversal micro-optimization?
- Is 1.5× plausible? Is 2× plausible? Under which explicitly stated assumptions?

Do not simply restate the documented hypotheses. Identify which assumptions you would actively challenge.

We value a defensible negative conclusion more than speculative optimism.

## 10. Choose the next experiment for information value

Your final recommendation must select **one primary architectural investigation**.

You may recommend a compact-navigation feasibility experiment, a carefully calibrated real-walker interference test, a scheduling discriminator, or another idea.

You are not required to select the experiment that previous reviewers preferred.

But the experiment must:

- Address an opportunity large enough to matter.
- Distinguish at least two consequential engineering directions.
- Have a relatively inexpensive first falsification stage.
- State what measurement would invalidate the premise before a full implementation.
- Preserve relevant shipped decoder behaviour.
- Avoid building a large harness before calibration establishes its necessity.
- Have an explicit stop condition and maximum initial engineering scope.
- Explain how each possible result changes our next architectural decision.

If you favor compact navigation, specify the cheapest way to learn whether reduced table footprint can repay its additional navigation and construction work.

If you favor memory-interference diagnosis, explain why the selected pressure conditions are discriminating rather than merely correlated.

If you favor utilization, show how the recovered capacity translates into whole-decoder throughput and why previous dispatch results don't already refute it.

Do not default to a full seven-condition synthetic neighbour matrix.

The independent researcher's proposed minimal calibration sequence is available: target walker alone; target with real decoder neighbours; target with walker-only neighbours; then a more specific contrast if calibration succeeds.

You may improve or reject that sequence, but explain your choice.

## 11. Research-effort priorities and stopping rules

We are considering a substantial strategic reallocation:

- Approximately 70% toward high-upside architectural research.
- Approximately 20% toward utilization and scheduling.
- Approximately 10% toward small engineering improvements such as production P87.

These are indicative priorities, not binding percentages.

Your responsibility is to recommend where attention actually has the greatest expected value.

P87 remains a credible production optimization candidate. One production-shaped validation may be worthwhile, but it must not monopolize the architectural research budget.

We do not want an indefinite sequence of P87 variants.

Likewise, we do not want to reopen failed succinct structures merely because their reduced memory footprints are attractive.

Every investigation should have a clear stopping rule, including when to abandon an idea despite mathematically elegant structure or positive microbenchmark results.

## 12. Required deliverable

Produce a **research-architecture memorandum**, not another chronological journal.

Structure it as follows.

### I. Executive recommendation

Your most important architectural insight, the best next experiment, and the reason.

State plainly if no current idea has a credible path to a breakthrough.

### II. Performance opportunity model

Decompose the throughput target into CPU efficiency, worker utilization, and irreducible or currently unaddressed costs.

Provide quantitative ceilings and sensitivity estimates, distinguishing measured data from assumptions.

### III. Where the existing research may be misdirected

Identify the assumptions, measurement interpretations or inherited implementation choices you would challenge first.

### IV. Succinct-navigation reassessment

Compare the shipped T-vector, failed 24-bit implementation, P40 fixed records, P38 byte recovery, P84 direct construction, two-tier design and any new representation you find credible.

Include mathematical invariants, data-access cost, construction economics, projected memory footprint and full-decoder implications.

Explain precisely what would prevent a new succinct candidate from repeating earlier failures.

### V. Ranked breakthrough candidates

Present three to five genuinely distinct candidates.

For each provide:

- Architecture and mechanism.
- Measured bottleneck attacked.
- Plausible CPU-s/GB recovery and throughput effect.
- Best-case versus realistic expectations.
- Why it is not already refuted.
- Construction, traversal, RSS, memory and utilization risks.
- Cheapest decisive falsification.
- Probability of engineering success, explicitly labeled as judgment rather than measured fact.
- Expected information value and implementation cost.

Rank for expected full-decoder engineering benefit, not elegance or novelty.

### VI. The single best next experiment

Give a concrete staged design with calibration checks, discriminating outcomes, time or engineering-cost cap, and termination rules.

Explain why this is more valuable than the next-best experiment.

### VII. What to stop doing

List specific research directions you would close, defer or refuse to restart.

Be candid about marginal-optimization traps and experimental overengineering.

### VIII. Assessment of the 1.5–2× objective

Evaluate 1.5× and 2× separately.

State which combination of CPU-efficiency and utilization improvements would be required and whether you consider those improvements physically credible.

### IX. Questions for the independent reviewer

Identify the three most important assumptions, numerical models or architectural claims that ought to be challenged before implementation.

## 13. Sources and evidence discipline

Prioritize the actual repository source code and experimental research documents.

Relevant records include:

- Repository: https://github.com/lmmx/mibz2
- Original compact-navigation research: https://lmmx.github.io/mibz2/blog/2026-10-29-compact-navigation-tables/
- Research portfolio, including P38, P40, P84 and P87: https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md
- BWT memory proposal and subsequent corrections: https://github.com/lmmx/mibz2/blob/docs/bwt-memory-proposal/docs/journal/2026-11-01-bwt-memory-proposal.md
- Original decoder attribution: https://github.com/lmmx/mibz2/tree/docs/decoder-attribution-records/docs/journal/2026-11-01-decoder-attribution-i9-10900K
- Dispatch-causality research: https://github.com/lmmx/mibz2/tree/docs/dispatch-causality-records
- Completed P87 experiment: https://github.com/lmmx/mibz2/pull/122
- P87 post-capture report: `docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md` on `research/p87-buffer-reuse`
- Historical detailed experimental notes and trial reports within the repository.

Please find and read the relevant primary evidence and source implementation before final ranking, particularly the actual P40/P38/P84 designs and the negative compact-navigation trials.

Do not rely on this prompt's summaries where primary documentation can resolve an important uncertainty.

If some records cannot be accessed, state exactly what is unavailable and qualify the affected conclusions.

Preserve disagreements between historical researchers instead of rewriting them as consensus.

Do not assume that a negative result on another CPU architecture necessarily transfers to the i9, but do not dismiss a negative full-decoder result because of a promising standalone benchmark.

**Scope boundary:** This is an architectural selection and falsification-design assignment only. Do not implement code, modify research records, start benchmarks, authorize hardware captures or change production defaults.

Be ambitious about architectural ideas and conservative about claimed evidence.

The purpose is to determine what could make `mibz2` dramatically faster—not merely to produce another well-controlled 1% experiment.
`````
