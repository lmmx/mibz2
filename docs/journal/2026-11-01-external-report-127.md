# External research report 127: the guardian's consolidated technical history: from the four-core slowdown to the P87 experiment (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 16:40 UTC.
- **Source:** the guardian.
- **In reply to:** the maintainer's request: "ok and can you tell me any more technical details about what astra / claude came up with and the feedback on it from you and the researcher? as much detail as possible, all consolidated in one place (I am gonna record it all for posterity)".

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment: a proposal and its review, not a performance result.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
# mibz2 research record: From four-core slowdown to the P87 experiment

Consolidated technical history · 1 November 2026 · Status: P87 selected for diagnostic preparation; no new i9 capture authorized

## Executive summary

Our central performance problem is that `mibz2`'s four-worker bzip2 decoder performs almost the same number of instructions per decoded byte as its one-worker configuration, but consumes substantially more CPU cycles per byte.

The original attribution experiment measured:

- 32.2% more CPU time per GB, rising from 2.707 to 3.579 CPU-s/GB.
- 30.1% more cycles per byte, despite only 1.3% more retired instructions per byte.
- Approximately 0.68–0.72 CPU-s/GB of additional inverse-BWT cost, including associated operations, accounting for 78–83% of the total one-to-four-worker CPU-cost increase.
- Substantially increased LLC misses, stalled cycles and DRAM traffic under concurrency.

That tells us where much of the additional expense occurs, but not which physical mechanism causes it.

The subsequent independent investigations produced two proposals:

Astra's P87 proposal: Keep the four-byte T-vector, 32-lane walker, software prefetching and segment-assembly copy exactly as they are, but change the destination of the assembled bytes. Reuse the last-column buffer `ll` after T-vector construction finishes, instead of writing to the separate `rle` buffer.

Claude's interference proposal: Pin a target walker to one CPU and surround it with controlled workloads on the other three CPUs—other walkers, builders, clean cache holders, dirty writers, LLC request generators and DRAM streams—to identify which forms of contention reproduce the slowdown.

The researcher independently audited both.

The resulting decision is:

Run the smaller, production-relevant P87 buffer-lifetime diagnostic first. Preserve Claude's broader interference investigation as the next substantial option if P87 fails to deliver a meaningful improvement.

This is not a rejection of Claude's research. Its new phase analysis materially narrowed the problem and revealed weaknesses in some of our earlier microarchitectural interpretations.

## 1. The decoder architecture and the physical problem

### 1.1 The decoding pipeline

At a high level, the parallel decoder runs a compressed-stream front end and distributes prepared blocks to decoding workers.

Within the relevant worker path, the data flows through:

MTF decoding → `ll`

Last-column byte sequence

T-vector construction

Reads `ll`; constructs four-byte inverse-BWT links

32-lane BWT traversal

Follows dependent links with software prefetching; writes segmented output

Segment assembly → `rle`

Copies segment bytes into contiguous inverse-BWT output

RLE1 expansion and CRC

Writes final decompressed output and verifies integrity

For a full 900,000-row bzip2 block, the T-vector contains approximately 3.6 MB of four-byte links. The `ll` and `rle` byte buffers each have a capacity on the order of 0.9 MB.

The walker is unusual because its memory accesses are dependent. A future link address depends on a preceding lookup. The decoder compensates by maintaining 32 traversal chains in flight and prefetching anticipated link locations.

This creates two kinds of concurrency:

- Within each worker: 32 independent chains help overlap dependent memory accesses.
- Between workers: several T-vectors and their supporting buffers compete for shared caches, memory request resources and memory bandwidth.

The i9-10900K used for the measurements has a 20 MB shared last-level cache. Four full T-vectors alone nominally represent approximately 14.4 MB of data, before considering segments, `ll`, `rle`, final output, scanner buffers and other activity.

That arithmetic makes shared-cache pressure plausible. It does not establish how many T-vector lines are actually resident in LLC at any particular instant.

### 1.2 The one-to-four-worker penalty

The original shipped-decoder attribution measurements were:

| Metric                   | One worker | Four workers | Change |
| ------------------------ | ---------- | ------------ | ------ |
| CPU-s/GB                 | 2.707      | 3.579        | +32.2% |
| Cycles/output byte       | 13.46      | 17.51        | +30.1% |
| Instructions/output byte | 32.63      | 33.06        | +1.3%  |
| Instructions per cycle   | 2.42       | 1.89         | −22%   |
| Effective core clock     | 4.97 GHz   | 4.89 GHz     | −1.6%  |

This pattern immediately argues against looking only for instructions to eliminate.

The decoder is executing almost the same instructions, but retiring them less efficiently.

The core-frequency change accounts for only approximately 5.9% of the CPU-cost inflation. Most of the increase therefore arises from additional cycles spent executing or waiting, not simply a slower clock.

The original phase attribution assigned approximately 78–83% of the incremental CPU cost to inverse BWT and associated work.

That qualification matters. It does not mean inverse BWT accounts for 78–83% of total decoder CPU time.

The added total CPU cost is:

\\[ 3.579-2.707=0.872\ \text{CPU-s/GB} \\]

Applying the attribution range gives approximately:

\\[ 0.872\times(0.78\text{–}0.83) \approx0.68\text{–}0.72\ \text{CPU-s/GB} \\]

That is the broad inverse-BWT scaling penalty we are trying to understand.

## 2. What the original memory measurements established

The i9 attribution experiment found the following differences between one and four workers.

| Measurement                                           | One worker | Four workers      | Interpretation                                 |
| ----------------------------------------------------- | ---------- | ----------------- | ---------------------------------------------- |
| Stalled execution cycles per output byte              | 2.95       | 6.31              | Substantially more execution time stalled      |
| Cycles/byte stalled with a load outstanding           | 2.25       | 5.18              | Strong memory-related signature                |
| Cycles/byte stalled with a demand L3 miss outstanding | 0.10       | 1.13              | More demand-miss-associated stalls             |
| Retired-demand L3 misses/KB                           | 0.22       | 3.61              | Large increase                                 |
| Retired-demand L3 hit fraction                        | 98.9%      | 81.1%             | Demand-load service mix changes                |
| Reads past L2/KB, including prefetch                  | 1,454      | 1,420             | Approximately stable                           |
| Mixed outstanding-request occupancy proxy             | 41         | 58 cycles/request | Increased, but not measured individual latency |
| Fill-buffer-full events/1,000 cycles                  | 146        | 177               | Increased pressure                             |
| DRAM reads/output byte                                | 1.6 B      | 9.8 B             | Substantially higher                           |
| DRAM writes/output byte                               | 0.6 B      | 6.9 B             | Substantially higher                           |

The trace also showed that the additional execution stalls dominate the extra cycle cost.

However, there are important interpretive limits.

`OFFCORE_REQUESTS.ALL_DATA_RD` includes demand and prefetch requests. Stable values cannot be interpreted as stable demand-load traffic.

The mixed outstanding-request occupancy ratio is not a direct latency measurement for a T-vector load.

The relevant L1 fill-buffer counters do not establish how many useful independent BWT lookups are simultaneously advancing.

And a rise in DRAM writes does not, by itself, tell us which buffer generated the writes or whether they caused the slowdown.

That last distinction ultimately motivated Astra's proposal.

### 2.1 The unresolved causal chain

Astra suggested that several apparent bottlenecks may belong to one feedback loop:

Construction and output activity displace useful cache lines

Loads and ownership requests take longer

Memory requests occupy finite resources for longer

Existing prefetch lead time becomes less adequate

Dependent traversal becomes more expensive

A plausible mechanism proposed by Astra, not an experimentally established causal chain.

The important insight is that cache eviction, dirty-line writebacks, prefetch pressure and slower traversal need not be competing explanations. They may reinforce one another.

An optimization must therefore be judged by whether it reduces actual decoder cost, not merely whether a particular counter changes.

## 3. Claude's independent discovery: the walk-plus-assembly phase dominates

Claude revisited the original attribution archive and created read-only analyses of phase timings and overlap between workers.

The researcher independently reproduced those calculations and corrected their interpretation.

### 3.1 Phase-by-phase scaling

Across the 197 full-sized 900,000-row blocks, using block-matched observations:

| Phase                      | One worker, ns/row | Four workers, ns/row | Slowdown |
| -------------------------- | ------------------ | -------------------- | -------- |
| MTF                        | 0.456              | 0.487                | 1.07×    |
| T-vector construction      | 0.497              | 0.571                | 1.15×    |
| Walk plus segment assembly | 1.234              | 1.806                | 1.46×    |
| RLE1 and CRC               | 0.134              | 0.180                | 1.35×    |

Scaling penalty by decoding phase

Four-worker phase cost divided by one-worker phase cost, full-sized blocks.

0×0.4×0.8×1.2×1.6×MTFT-vector buildWalk + assemblyRLE1 / CRC

The walk-plus-assembly phase is by far the largest contributor to extra CPU time.

But there are two different absolute estimates, with different denominators:

Full-sized blocks only

# 0.572

ns/row increase, numerically equal to CPU-s per billion BWT rows

Whole corpus

# 0.609

CPU-s/GB, using the researcher's whole-corpus normalization

The full-block result comes from:

\\[ 1.806-1.234=0.572\ \text{ns/row} \\]

The researcher subsequently included all 434 blocks per run, including 237 shorter blocks, and obtained a whole-corpus walk-plus-assembly cost increase from approximately 1.241 to 1.850 CPU-s/GB.

This produced the corrected 0.609 CPU-s/GB whole-corpus figure.

The distinction is important for posterity because it separates the full-block per-row comparison from the more representative overall decoder accounting. It also keeps the denominator explicit: BWT rows and final decoded bytes are not necessarily equal because of RLE expansion.

### 3.2 A crucial phase-label correction

Claude initially described this as the walk penalty.

The researcher pointed out that the existing trace bracket:

`WalkStart → InvertEnd`

includes both:

- Pointer traversal through the T-vector.
- Segment assembly, including the copies that produce contiguous inverse-BWT output.

Therefore, 0.609 CPU-s/GB is not an independently measured pointer-chasing-only penalty.

This matters especially because our prior sampled data attributed a substantial share of L3-miss-associated stalls to `memmove`, often arising from assembly.

A future experiment must separate traversal from assembly before claiming that a memory-layout change makes the dependent walk itself cheaper.

### 3.3 The relationship with concurrent walks

Claude then measured how a block's inversion phase behaved while other workers were also walking.

The researcher reproduced these bins:

| Other workers walking | Observations | Median slowdown relative to same block at one worker |
| --------------------- | ------------ | ---------------------------------------------------- |
| 0                     | 81           | 1.294×                                               |
| 1                     | 878          | 1.408×                                               |
| 2                     | 982          | 1.535×                                               |
| 3                     | 29           | 1.658×                                               |

Observed phase slowdown versus concurrent walkers

Repeated block observations; overlap was not experimentally randomized.

1×1.2×1.4×1.6×1.8×0123

This was one of Claude's most valuable findings.

It suggests that competing traversal activity is more closely associated with the slowdown than concurrent T-vector construction.

But the researcher identified three qualifications.

First, reverse causation: a walk that takes longer is more likely to overlap additional walks. Longer duration can create the measured association, rather than necessarily being caused by it.

Second, repeated observations: the 1,970 records are repeated measurements of 197 blocks across ten runs. They aren't 1,970 independent experiments. Treating them that way would underestimate uncertainty.

Third, uneven bins: the extreme categories contain only 81 and 29 observations. The one- and two-other-walker categories are substantially better populated.

Claude's regression had an \\(R^2\\) of approximately 0.23, meaning most variation remained unexplained by its selected overlap measure.

Consolidated conclusion: expensive concurrent traversal deserves priority, but the regression does not establish that walk-on-walk interference is the sole cause, or that construction interference can be ruled out.

## 4. The correction to Claude's memory-level parallelism model

Claude proposed a particularly elegant numerical explanation for the observed slowdown.

Its rough argument was:

- The walker issues approximately 0.93 relevant offcore requests per row.
- The mixed request occupancy ratio increases from about 38.6 to 57.5 cycles.
- Walk-plus-assembly execution cost increases from approximately 6.1 to 8.8 cycles per row.
- These figures imply approximately six concurrent outstanding requests at either worker count.
- Therefore, longer request service time at constant memory-level parallelism explains the additional walk cycles.

The approximate relationship was:

\\[ \text{effective overlap} \approx \frac{\text{requests/row}\times\text{cycles/request}} {\text{cycles/row}} \\]

The researcher's reproduction was:

| Assumed or derived quantity   | One worker  | Four workers |
| ----------------------------- | ----------- | ------------ |
| Assumed relevant requests/row | 0.93        | 0.93         |
| Mixed request occupancy proxy | 38.6 cycles | 57.5 cycles  |
| Walk-plus-assembly cycles/row | \~6.1       | \~8.8        |
| Inferred overlap              | \~5.9       | \~6.1        |

### Why this is not proof of six outstanding walker requests

The arithmetic is consistent, but the quantities don't all describe the same measured population.

The 0.93 requests per row is an estimate, not a walker-specific counter measurement.

The offcore request ratio was collected over the whole decoder and mixes different request types.

The elapsed walk interval includes assembly, whereas the request model is intended to explain traversal.

And, crucially, the inferred overlap is calculated using the same service-cost and execution-cost quantities subsequently used to explain the slowdown.

Substituting those derived values back into the model risks obtaining a numerical fit by construction.

The correct formulation is:

> The measured increase in walk-plus-assembly cost is quantitatively consistent with slower memory-request service at roughly unchanged effective overlap, but walker-specific request rates and memory-level parallelism have not been independently measured.

That leaves at least two distinct possibilities open:

- Requests take longer to complete because the memory system is more heavily contended.
- Effective overlap deteriorates because prefetches become less timely, dependent chains stall, or hardware resources cannot sustain the previous degree of concurrency.

Both may be happening.

### 4.1 The 98% software-prefetch claim

Claude also proposed that approximately 98% of relevant walker traffic was handled through software prefetching.

The researcher rejected this exact figure.

The argument compared approximately 0.019 retired-demand L3 events per byte with approximately 1.42 mixed offcore requests per byte.

These are not interchangeable counts. They have different scopes, event semantics and populations.

Intel's relevant prefetch categories can combine hardware L1 prefetching with software prefetching, while the total offcore event includes traffic from outside the walker.

Thus the existing data support a prefetch-heavy traversal, but not a precisely measured 98% software-prefetch traffic share.

This correction is important because the optimizer might otherwise conclude that changing prefetch distance cannot help, or conversely that prefetching itself is the proven source of contention.

Neither follows from the archived counters.

## 5. Astra's independent proposal: P87 buffer-lifetime reuse

Astra approached the problem from a different angle.

Rather than first building a detailed model of every cache or uncore bottleneck, it asked:

Can we alter one real buffer lifetime, without changing the expensive traversal algorithm, and make four-worker decoding cheaper?

That became the P87 proposal.

### 5.1 The existing scratch-buffer lifetimes

The relevant worker scratch state contains separate byte vectors for:

```
ll: Vec<u8>,  // MTF's last-column output
rle: Vec<u8>, // Assembled inverse-BWT output
```

Their ordinary use is sequential.

`ll` is written by MTF and read during T-vector construction.

The segmented walker subsequently follows T-vector links and writes segment buffers.

Those segments are then assembled into `rle`.

RLE1 reads `rle` to produce the final output.

The important observation is that, for the ordinary segmented path, the old contents of `ll` are no longer needed after T-vector construction completes.

The allocation could therefore have a second role later in the block.

### 5.2 The proposed A/B comparison

A — Separate arenas

`ll`

MTF writes and build reads

Unchanged T-vector walk and segment writes

`rle`

Assembly destination

B — Reused arena

`ll`

MTF writes and build reads

Unchanged T-vector walk and segment writes

Reuse `ll`

Assembly destination

The causal diagnostic keeps both allocations available in both arms. The intended distinction is which previously used memory region assembly writes.

Both arms must still:

- Construct the same four-byte links.
- Perform the same 32-lane dependent traversal.
- Use the same software-prefetch policy.
- Write the same segment bytes.
- Perform the same logical assembly copy.
- Run RLE1 and CRC checking on the same decoded data.

Astra deliberately avoided copy elimination.

Otherwise, removing a copy could change instruction counts, branch behaviour, buffer residence and output timing simultaneously, obscuring the intended lifetime mechanism.

### 5.3 Why buffer reuse could help

At full block size, the existing arrangement maintains two separate approximately 0.9 MB byte arenas for successive roles.

In B, assembly writes memory that was already used as the last-column storage.

Potential benefits include:

Cache-line reuse. Some `ll` lines may still exist in a favourable cache state when assembly begins. Writing them might generate less expensive ownership or allocation activity than writing the separate `rle` arena.

Dirty-line coalescing. Reusing physical cache lines could reduce the number of distinct lines dirtied over the sequence of processing phases, potentially reducing later writebacks.

Cache occupancy. If the separate assembly arena competes with useful T-vector lines, touching fewer distinct memory regions over time may reduce cache displacement.

Cross-worker effects. Less memory activity from one worker's assembly phase could make another worker's simultaneous traversal cheaper.

Persistent buffer-state effects. Scratch allocations survive across blocks, so changes in one block's memory access history might influence later decoding phases.

None of these benefits is guaranteed.

### 5.4 Why buffer reuse might do nothing

The researcher pointed out a subtle issue: both `ll` and `rle` are already persistent scratch buffers reused between blocks.

In the shipped implementation:

- `ll` was recently written by MTF and read during construction.
- `rle` was read by RLE1 during the preceding block.
- Both may have been displaced from useful caches by subsequent T-vector construction and traversal.

If `ll` is no longer resident by the time assembly starts, writing it may be little better than writing `rle`.

It's therefore inaccurate to say that the proposed change necessarily saves one block's worth of DRAM writes.

It changes the destination's access history and which lines are touched during each phase.

Astra's experiment is valuable precisely because it tests whether that distinction matters.

### 5.5 Why traversal improvement would be particularly revealing

Assembly occurs after a worker's own traversal.

Therefore, a worker's assembly destination cannot retroactively accelerate the traversal that has already completed.

If B shows cheaper traversal, despite matching traversal instructions, the effect must be indirect.

Possible pathways include:

- Another worker generates less interfering traffic during the measured walk.
- Previous blocks leave a more favourable cache or memory-system state.
- Changes in phase durations alter the pattern of overlapping work.
- Shared-cache or uncore resource contention changes.

That would be evidence of a real cross-phase or cross-worker interaction.

It would not, by itself, distinguish cache capacity relief from writeback reduction or request timing.

And if only assembly or RLE1 becomes faster, we may still have found a useful optimization—but not evidence that P87 solves the expensive dependent-traversal problem.

## 6. Astra's proposed statistical experiment

Astra proposed a two-by-two comparison:

|                               | One worker     | Four workers   |
| ----------------------------- | -------------- | -------------- |
| A: original `rle` destination | \\(C\_{A,1}\\) | \\(C\_{A,4}\\) |
| B: reused `ll` destination    | \\(C\_{B,1}\\) | \\(C\_{B,4}\\) |

Here \\(C\_{X,w}\\) means CPU-seconds per GB for arm \\(X\\), using \\(w\\) workers.

The principal mechanism statistic is:

\\[ D=(C\_{A,4}-C\_{B,4})-(C\_{A,1}-C\_{B,1}) \\]

This is a difference-in-differences comparison.

It asks whether buffer reuse saves more CPU time at four workers than at one worker.

Suppose both arms perform exactly the same traversal work, but the reused layout reduces memory interference only under concurrency. We'd expect a larger saving at four workers.

If the change merely accelerates local assembly to the same degree at one and four workers, \\(D\\) would be small even if B is faster overall.

### 6.1 The proposed materiality threshold

Astra suggested:

\\[ D \geq 0.10\ \text{CPU-s/GB} \\]

This threshold represents approximately 14% of the originally estimated 0.68–0.72 CPU-s/GB inverse-BWT scaling penalty.

It also represents approximately 2.8% of the existing 3.579 CPU-s/GB four-worker decoder cost.

It is a decision threshold, not Astra's prediction of what the experiment will achieve. The plausible improvement includes zero.

Astra proposed evaluating uncertainty around \\(D\\):

- A lower confidence bound above 0.10 supports a material concurrency-specific benefit.
- An upper confidence bound below 0.10 rejects that materiality claim.
- A confidence interval crossing 0.10 produces an inconclusive mechanism result.

The researcher emphasized that confidence intervals must reflect independent paired runs, not individual blocks treated as independent replicates.

The exact CI procedure and treatment of borderline equality remain protocol-design details for Claude to freeze before timing.

### 6.2 The researcher's most important amendment: two independent decisions

The researcher identified a flaw in using \\(D\\) alone as a success criterion.

Consider this hypothetical result:

| CPU-s/GB     | A: original | B: reuse |
| ------------ | ----------- | -------- |
| One worker   | 2.71        | 3.01     |
| Four workers | 3.58        | 3.63     |

Here B is worse at both worker counts.

But:

\\[ D=(3.58-3.63)-(2.71-3.01) \\]

\\[ D=-0.05-(-0.30)=+0.25 \\]

So B clears the proposed 0.10 mechanism threshold despite slowing down the actual four-worker decoder.

This exposes the distinction between reducing the concurrency-specific penalty and improving absolute performance.

The finalized design therefore needs two separate decisions:

Mechanism gate

Is the concurrency-specific saving \\(D\\) materially positive, exceeding the 0.10 CPU-s/GB threshold with prespecified confidence?

Engineering gate

Does B actually improve four-worker CPU-s/GB and complete-decoder throughput, while preserving correctness and acceptable RSS?

This distinction also creates an informative middle outcome.

B might achieve a useful absolute four-worker speedup but fail the concurrency-specific threshold.

That would be a local optimization, not confirmation of the proposed scaling mechanism.

Conversely, a positive \\(D\\) accompanied by worse four-worker performance should not advance to production.

### 6.3 The statistical outcome matrix

| Mechanism result                     | Four-worker engineering result | Interpretation                                                           |
| ------------------------------------ | ------------------------------ | ------------------------------------------------------------------------ |
| Material positive \\(D\\)            | CPU and throughput improve     | Strong candidate for production-oriented validation                      |
| Material positive \\(D\\)            | Four-worker regression         | Concurrency effect exists, but proposed implementation is not beneficial |
| Nonmaterial \\(D\\)                  | Absolute improvement           | Possible useful local optimization                                       |
| Nonmaterial \\(D\\)                  | No improvement                 | P87 four-core mechanism not supported                                    |
| Inconclusive \\(D\\)                 | Any                            | Preserve uncertainty; don't claim the mechanism was established          |
| Invalid code or measurement controls | Any                            | No usable causal verdict                                                 |

A positive experimental outcome would still need a subsequent release-oriented validation before shipping.

## 7. The historical P87 criterion: why we must preserve it

The research register already contains this optimization concept.

Its recorded identifier is P87: Walk output in L's freed buffer, also known as E9 in an earlier research matrix.

It is listed as open and untested.

The historical criterion concerns improving the occupancy curve at 6–10 workers by at least 3%, with accompanying DRAM-write measurements.

That was a different performance question.

The new proposed experiment focuses on one-versus-four-worker concurrency-specific CPU cost, because the project's current objective is substantial efficiency and throughput improvement on four physical cores.

We cannot silently redefine the historical experiment's outcome.

The agreed solution is to register the new test as a distinct four-core causal diagnostic of P87's mechanism.

This means:

- A negative four-worker diagnostic does not automatically close the original six-to-ten-worker hypothesis.
- A positive four-worker diagnostic does not automatically satisfy the old occupancy-curve criterion.
- The new experiment gets its own immutable rules, corpus, confidence method and decision record.

That preserves the interpretability of the historical portfolio.

## 8. Why Claude's broader interference harness was not selected first

Claude's independently proposed experiment was more ambitious.

It aimed to identify which physical resource was responsible for the concurrent walk slowdown by building a controlled target-and-neighbours environment.

The target would be a pinned decoding worker whose walk phase is measured with in-process counters.

The other three CPUs would execute different workloads intended to create distinct kinds of interference.

### 8.1 The proposed experiment arms

| Arm                         | Neighbour workload                 | Intended question                                                        |
| --------------------------- | ---------------------------------- | ------------------------------------------------------------------------ |
| Real decoders               | Three other decoder loops          | Can a controlled four-core environment reproduce the production penalty? |
| Walk-only                   | Repeated T-vector traversals       | Does concurrent traversal reproduce the cost?                            |
| Build-only                  | Repeated T-vector construction     | How much does construction interfere?                                    |
| Clean cache holders         | Arrays of several sizes            | How sensitive is the walk to cache-footprint pressure?                   |
| Dirty holders               | Write-heavy data sets              | How sensitive is the walk to write allocation and dirty eviction?        |
| Small-footprint LLC hammers | High request rate, small footprint | Does request queueing matter independently of capacity?                  |
| DRAM streams                | Streaming bandwidth consumers      | Does external memory traffic slow the walk?                              |

Claude also proposed examining the level that supplied prefetches, fill-buffer hits and page-walk activity.

The last item matters: TLB misses and page walks represent another possible source of costly dependent-access servicing, beyond the original five memory hypotheses.

### 8.2 The attraction of this design

If controlled successfully, the harness could explore mechanisms that P87 cannot distinguish.

For example:

- If large clean data sets slow the target while small high-request-rate workloads do not, cache footprint becomes a stronger candidate.
- If a small-footprint request generator slows the target without requiring much cache capacity, LLC or uncore request contention becomes more plausible.
- If walk-only neighbours reproduce the effect while build-only neighbours do not, concurrent traversal becomes a stronger causal candidate.
- If page-walk counters increase materially, page-size interventions could become worth testing.

This investigation could eventually point toward much larger changes than P87, such as reducing unavoidable T-vector lookups or improving prefetch lead time.

It directly addresses the approximately 0.609 CPU-s/GB walk-plus-assembly scaling penalty, which P87 might only reduce modestly.

### 8.3 Why the researcher challenged it

The essential problem is that these synthetic workloads change multiple resources at once.

Clean cache holders

A data set's nominal size does not prove it occupies that much LLC. The target may evict it. Increasing how often the holder touches the data to keep it resident also increases request traffic.

Thus the arm changes both footprint and access pressure.

Dirty holders

Stores create more than dirty-line writebacks. They can create ownership requests, read-for-ownership traffic, write allocations, cache replacement pressure and request scheduling effects.

Any performance difference between clean and dirty holders cannot be attributed uniquely to writebacks.

Small-footprint LLC hammers

They may exercise LLC request resources, but their address mapping, request dependencies, prefetch behaviour and LLC-slice distribution may differ considerably from the actual T-vector walk.

DRAM streams

Matching total memory bandwidth does not match the decoder's read/write mix, bursts, bank behaviour, request dependencies or request latency.

Real-decoder neighbours

Four independently pinned decoding loops are not the shipped four-worker pipeline.

The shipped application also contains scanner, reader, feeder, queueing and variable build/walk overlap. It has approximately 2.19 active T-vector phases on average, rather than four workers continuously performing the same phase.

The resulting memory-system environment could be substantially different.

### 8.4 The 70% calibration flaw

Claude proposed an early stop if the real-decoder neighbour arm failed to reproduce at least 70% of the production walk-scaling penalty.

Stopping could be sensible.

But Claude's original interpretation was too strong: it proposed treating that failure as evidence against separate-core memory interference being the explanation.

The researcher correctly identified the alternative:

The harness may simply have failed to reproduce production conditions.

A synthetic experiment's failure to recreate a slowdown is not proof that the slowdown lacks a memory-system cause.

The correct outcome would be:

> Calibration failed. The neighbour arms cannot yet be interpreted as explaining production decoder behaviour.

The researcher recommended substantially stronger representativeness checks, including:

- Stage-specific CPU costs.
- Actual request rates and request types.
- Buffer lifetimes and allocation reuse.
- Distribution of concurrent T-vector exposure.
- Walk-cost distribution, rather than only an average slowdown.
- The relative amount of scanner, reader and other application activity.

These requirements increase the implementation and validation cost beyond the apparent simplicity of a roughly 300-line harness.

### 8.5 Why we preserved it

We did not conclude that the neighbour experiment is inherently unsound.

It is a credible research direction, especially if P87 produces little benefit while the expensive traversal penalty remains.

The recommended future version is staged:

1. First reproduce a sufficiently production-like target and real-decoder-neighbour environment.
2. Then compare genuine walk-only interference against clean-footprint pressure and small-footprint request pressure.
3. Add build-only, dirty and DRAM-bandwidth arms only when the initial comparisons establish that they can distinguish useful hypotheses.

The harness's job would then be to test specific suspected mechanisms, not decompose the entire slowdown into independent additive percentages.

The selection of P87 first is therefore about decision value and experimental cost, not a claim that P87 addresses the largest possible opportunity.

## 9. The five memory-system hypotheses, consolidated

Astra, Claude, the researcher and I ultimately converged on the following interpretation.

| Hypothesis                                 | What we know                                                                          | What we don't know                                                                            | Most relevant experiment                                                                                                          |
| ------------------------------------------ | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| H1: T-vector eviction and cache residency  | Demand LLC misses rise under concurrency; several large tables share LLC              | Actual T-vector residency, service latency and the responsible evictors                       | P87 may perturb neighbouring footprint; address-specific measurements or controlled footprint pressure would discriminate further |
| H2: Prefetch interference                  | The walker uses software prefetches; fill-buffer pressure increases                   | Which prefetches are timely, wasted or causing contention                                     | Phase-specific request measurements; potentially a carefully registered prefetch-distance intervention                            |
| H3: Intermediate writes and dirty eviction | DRAM writes rise markedly, and assembly copies hold substantial L3-miss stall samples | Whether the writes cause traversal latency or follow from shared-cache pressure               | P87 directly changes assembly destination and buffer lifetime                                                                     |
| H4: Build/walk overlap                     | Both occur concurrently; observed slowdown tracks concurrent walks                    | Whether particular phase combinations cause the cost                                          | A controlled phase-order experiment, not merely overlap correlation                                                               |
| H5: LLC/uncore request contention          | Request occupancy proxy and memory-related stalls increase                            | Whether cache hits themselves become slower, or which shared request resource limits progress | A calibrated walker versus small-footprint request-pressure experiment                                                            |

There is also a sixth candidate introduced by Claude's proposal:

H6: TLB/page-walk effects. Traversing multiple large, dispersed memory regions may increase translation pressure under concurrency. The currently discussed data do not establish this as a cause; an appropriately supported page-walk measurement would be needed before prioritizing huge pages.

### 9.1 Why not immediately retune prefetching?

Claude's historical survey established:

- The shipped walker uses the L1-oriented prefetch approach.
- An L2-oriented hint was approximately equivalent on the i9.
- An NTA variant performed extremely poorly, around 0.61× in the referenced comparison.
- A genuine prefetch-distance sweep had apparently never been run.

This leaves a meaningful unanswered question. If memory service becomes slower, perhaps the walker needs to prefetch further ahead.

But changing prefetch distance could alter instruction placement, live lane state, cache insertion timing, useful overlap and resource pressure simultaneously.

Our present evidence does not establish that insufficient prefetch distance, rather than increased resource contention, is the primary problem.

So prefetch distance is a valid future hypothesis, not the selected first intervention.

### 9.2 Why not compact the four-byte T-vector immediately?

The existing four-byte representation has important advantages:

- Direct, inexpensive lookup.
- Simple address computation.
- No additional byte-recovery chain per transition.
- An already optimized one-pass builder.
- Efficient use of the 32-lane traversal design.

Earlier compact-Ψ work suffered substantial extra per-core cost. The research record includes a compact `u` variant around 0.68× the existing per-core performance.

That can erase the gain from a smaller memory footprint.

A compact representation might become valuable if a sufficiently large fraction of the four-worker penalty is proved to come from unavoidable T-vector footprint or lookup traffic, and if the extra construction and navigation costs fit within the performance budget.

P87 deliberately avoids taking that risk.

### 9.3 Why not eliminate the assembly copy?

Earlier copy-elimination work did not establish a major decoder-level improvement.

More importantly, eliminating the copy would answer a different question.

P87 preserves the same logical copy volume, but changes where the copy writes.

That separates the cost of performing a copy from the memory-system consequences of touching one allocation rather than another.

The researcher considered this separation one of Astra's strongest experimental design choices.

## 10. The controls P87 must satisfy

This is where much of the independent feedback became especially valuable.

A buffer-reuse experiment sounds trivial, but there are several ways a naïve A/B implementation could produce a misleading result.

### 10.1 Rust borrowing and phase separation

The existing `InverseBwt::invert()` interface consumes an immutable `ll` reference while performing inversion into an output vector.

In the existing segmented implementation, the high-level sequence is effectively:

```
build(ll, counts, &mut self.tt);
walk_segments_room(&mut self.tt, orig_ptr, out, &mut self.seg);
```

Writing assembly output into `ll` requires ending the immutable input borrow after the builder has consumed it.

Claude will therefore need a carefully separated build/traversal/assembly path.

The repository already has useful components such as `walk_segments_room_parts()` and `SegScratch::assemble()`, reducing the need to rewrite the actual 32-lane algorithm.

But the interface change is still nontrivial: an apparently harmless refactor might change code generation, buffer capacities or phase boundaries.

### 10.2 Machine-code equivalence

Both arms must execute equivalent T-vector construction and traversal instructions.

Checking that two Rust paths are logically identical is insufficient.

In particular, reorganizing the inversion interface can affect:

- Inlining decisions.
- Register allocation.
- Loop layout.
- Instruction scheduling.
- Function boundaries.
- Branch placement.

The experiment must audit generated machine code and identify any material deviations.

We agreed that perfect equivalence should not be pursued at the cost of introducing a more serious new confounder. If strict identity proves impossible, the protocol must disclose and account for the difference rather than conceal it.

The most important code-equivalence requirement concerns the builder and the pointer-chasing walker, because those are the stages whose concurrency scaling we want to explain.

### 10.3 Equal memory allocation and page residency

An ordinary production version of P87 might eliminate a redundant approximately 0.9 MB allocation.

That would be attractive for RSS.

But doing so in the initial diagnostic would mix two effects:

1. Different logical buffer lifetime and access history.
2. Different allocation volume, page faults, virtual layout and retained memory.

Astra therefore proposed allocating both arenas in both arms, with matching capacities and initial page residency.

In B, the unused `rle` arena remains allocated.

This prevents any speedup from being attributed simply to reduced allocation or initial page-fault costs.

The researcher added a critical nuance: do not repeatedly touch, warm or dirty the unused B arena during measurement merely to maintain artificial equivalence.

That would create an extra memory workload absent from the actual optimization.

Equalize the initial conditions, then let each arm execute its naturally defined access pattern.

### 10.4 Address placement

The physical placement of `ll`, `rle` and other allocations may influence cache-set mapping, page colouring, TLB activity and reuse.

If the B arm happens to receive a favourable mapping, it could appear faster for reasons unrelated to the lifetime hypothesis.

Astra suggested balancing or varying arena placement across paired repetitions where feasible.

That control must itself be specified in advance, without selecting a favourable arrangement after results are observed.

### 10.5 No changes to the scheduling environment

The experiment must retain:

- The same four physical cores.
- The same worker count.
- The same scanner and reader behaviour.
- The same admission and output-ordering policies.
- The same worker affinity policy.
- The same software-prefetch configuration.

This matters because our dispatch investigation discovered meaningful wake-affine scheduling effects.

If the P87 experiment also changed thread placement, we'd no longer know whether a throughput difference came from buffer reuse or scheduling.

### 10.6 Correctness

The change affects storage ownership, and correctness is not merely a matter of comparing typical large blocks.

Claude's preparation must test:

- Full-sized blocks.
- Short blocks and the plain-walk fallback.
- Mixed block sizes.
- Repeated reuse of scratch buffers.
- Capacity growth and reallocation.
- Segment boundaries.
- RLE1 output equivalence.
- CRC verification.
- Error and cancellation paths where relevant.

The diagnostic must not corrupt `ll` before the final construction read completes.

And output comparisons must cover complete decoded data, not merely checksums or selected blocks.

### 10.7 The instrumentation must be staged

Astra initially proposed a fairly extensive PMU suite, including:

| Category                  | Suggested measurements                             | Important limitation                                                   |
| ------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------- |
| Core execution            | `cycles`, `ref-cycles`, `instructions`, task-clock | Core clocks do not directly measure uncore frequency                   |
| Ownership traffic         | L2 RFO misses                                      | Ownership requests are not DRAM writes                                 |
| Dirty-line movement       | Modified L2 line evictions                         | L2-to-LLC eviction does not imply immediate DRAM writeback             |
| Stalls and resources      | Memory stall counters and `FB_FULL`                | They show pressure, not necessarily the bottleneck                     |
| Memory-controller traffic | IMC reads and writes                               | Machine-wide; cannot attribute traffic to a particular buffer directly |
| T-vector load service     | PEBS load-latency/address sampling                 | Thresholded, selected samples are not an unbiased mean latency         |
| Fill-buffer hits          | `MEM_LOAD_RETIRED.FB_HIT` where supported          | A fill-buffer hit does not prove useful software prefetching           |

The researcher recommended a lighter initial protocol.

The first question is simply: does buffer reuse improve the actual decoder, and where does the benefit appear?

Therefore, the initial experiment should prioritize:

1. Uninstrumented paired A/B full-decoder throughput and CPU-s/GB.
2. Independent A/A controls.
3. Separate build, traversal, assembly and RLE1/CRC timing.
4. Comparable whole-work DRAM reads and writes.
5. Correctness and machine-code equivalence.
6. Perturbation and clock/placement checks.

More elaborate PEBS, address-range attribution and ownership counters should follow only if their results can distinguish between remaining explanations of a measured benefit or failure.

This avoids creating an expensive PMU campaign for a mechanism whose plausible gain may be zero.

### 10.8 Why full-work memory boundaries matter

If B generates fewer dirty lines during a measured interval, those lines might simply be evicted later.

A counter window that ends before the deferred eviction would make the result look like a permanent traffic reduction.

Astra therefore recommended equivalent complete-work measurement boundaries, plus steady-state checks where useful.

No explicit cache flushing should be inserted into the timed decoder path merely to force comparable counters; that would alter the behaviour we're trying to study.

## 11. Interpreting the eventual P87 outcomes

The resulting engineering decision matrix is more nuanced than a simple pass/fail.

| Observed result                                                                                        | What it would support                                                                       | What we would do                                                                                      |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Material positive \\(D\\), four-worker CPU and throughput improvement, and cheaper unchanged traversal | Buffer lifetime contributes to the concurrency penalty                                      | Advance the layout toward production validation; investigate other intermediate lifetimes selectively |
| DRAM writes decline but traversal CPU cost does not                                                    | Removed write traffic is not a material traversal-latency cause under this intervention     | Stop treating this particular write stream as a major path to a breakthrough                          |
| Four-worker CPU improves, but \\(D\\) is not material                                                  | Local assembly/cache behaviour improves more than concurrency scaling                       | Evaluate as a small standalone optimization                                                           |
| \\(D\\) improves but four-worker performance regresses                                                 | Scaling relationship improves without a useful absolute decoder result                      | Do not promote the implementation                                                                     |
| CPU improves but DRAM writes remain unchanged                                                          | Benefit may arise through cache occupancy, ownership latency, TLB effects or request timing | Preserve the improvement without claiming writeback reduction as the cause                            |
| No meaningful CPU or traffic change                                                                    | Reusing `ll` did not materially perturb the relevant resource                               | Close this specific four-core intervention; retain broader memory-interference hypotheses             |
| Expensive traversal remains and tracks concurrent walks                                                | Shared traversal pressure remains a serious candidate                                       | Consider Claude's refined interference experiment                                                     |
| Measurement or code-generation controls fail                                                           | The comparison does not isolate the intended change                                         | Repair the diagnostic before drawing conclusions                                                      |

The distinction between traffic reduction and critical-path acceleration is foundational.

A decoder could write fewer bytes to DRAM without getting faster if those writes were asynchronous, decoupled or not blocking useful traversal.

Equally, a decoder could become faster without reducing total DRAM traffic if its requests are scheduled more favourably or if cache-line ownership becomes cheaper.

Therefore, PMU counters are supporting evidence. The primary engineering outcome remains actual four-worker decoder performance.

## 12. What this means for the 1.5–2× objective

The long-term target is much more ambitious than the expected gain from one buffer-lifetime experiment.

Our baseline is approximately:

Current throughput

# 869

MB/s, four workers

CPU cost

# 3.579

CPU-s/GB

Core occupancy

# 3.14 / 4

Average allocated cores busy

If we could eliminate the entire measured 0.68–0.72 CPU-s/GB inverse-BWT scaling penalty, CPU cost would fall to approximately 2.86–2.90 CPU-s/GB.

That would be an enormous achievement.

Nevertheless, with four physical cores, the throughput ceiling from that CPU efficiency alone would be approximately 1.59–1.61× today's measured rate, and reaching that ceiling would require essentially perfect utilization.

It is therefore important to distinguish CPU efficiency from utilization.

The concurrency memory penalty is the primary efficiency problem. Wake placement remains a credible, separate utilization opportunity.

For comparison, Astra's proposed 0.10 CPU-s/GB materiality threshold is:

- Approximately 14% of the broad inverse-BWT incremental penalty.
- Approximately 16% of the more narrowly defined 0.609 CPU-s/GB walk-plus-assembly penalty.
- Approximately 2.8% of current decoder CPU cost.

Even a clean positive result at that threshold would be a small fraction of the improvement needed to reach 1.5×.

But it could establish an entire family of useful memory-lifetime optimizations while preserving the efficient four-byte link representation.

That is why we have chosen to test it before pursuing a substantially more expensive new representation.

## 13. Where the dispatch investigation fits into this history

The memory programme was selected after a separate investigation of idle workers and job handoff.

That investigation deserves to remain in the research record because it eliminated a plausible distraction and uncovered a potentially useful secondary opportunity.

### 13.1 The registered MPMC result

The four-worker dispatcher shares a receiver protected by a mutex.

The original traces showed long delays around `JobLock`, suggesting that receiver ownership might be obstructing job distribution.

After an independent audit found FIFO backlog confounding those observations, Claude and Astra designed a kernel-level diagnostic incorporating scheduler and futex events.

The valid i9 capture found:

| Result                                                         | Measurement            |
| -------------------------------------------------------------- | ---------------------- |
| Direct receiver obstruction                                    | 1.90% of worker time   |
| Registered materiality threshold                               | 5%                     |
| Runs reaching threshold                                        | 0 of 10                |
| Receiver-associated exploratory share, including runnable time | 9.95%                  |
| Registered verdict                                             | Do not proceed to MPMC |

The diagnostic itself underwent substantial validation: Claude found and corrected missing scheduler-coverage checks, silent loss reporting and incomplete scheduler traces in the VM.

The real i9 capture passed the quality gates.

### 13.2 The untested scheduling effect

The subsequent postmortem found that workers could remain runnable for milliseconds after wake-up, often targeted at the same CPU as the thread that woke them while another allocated CPU became idle.

This suggests a possible interaction between receiver-mutex handoff and Linux wake-affine scheduling.

The researcher additionally separated scanner delays according to whether the admission window was open.

The actionable-looking category was approximately 3.6% of aggregate worker time: a worker has no job, and the scanner is runnable with an open admission window but is not executing.

Neither observation establishes that a different affinity policy would improve throughput.

The separate conclusion is:

MPMC's registered direct-obstruction hypothesis failed. Wake-affine scheduling remains untested.

The next scheduling experiment, if eventually selected, must account for all worker, scanner, reader and feeder activity within the same four physical-core budget.

We have deliberately parked it during P87 so that the two optimization questions remain experimentally separate.

## 14. The division of intellectual contributions

The collaboration produced something more useful than a unanimous recommendation: it created a chain of independently testable claims and corrections.

| Participant            | Principal contribution                                                                                                                                         | Important limitation or correction                                                                                  |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Astra                  | Identified P87 as a small, directly actionable buffer-lifetime intervention; proposed the 2×2 design, \\(D\\), physical hypotheses and decision matrix         | Cannot uniquely distinguish LLC capacity from dirty-line effects; initial instrumentation plan was relatively broad |
| Claude Code            | Independently reanalyzed phase timing and overlap, locating the large walk-plus-assembly penalty; proposed controlled neighbour experiments                    | Six-request MLP and 98% software-prefetch interpretations were too strong; synthetic calibration required revision  |
| Independent researcher | Reproduced phase figures; corrected whole-corpus accounting; audited PMU semantics, overlap bias, synthetic controls and P87 gates                             | Its preferred interpretation remains a recommendation, not new timed evidence for P87                               |
| Guardian               | Compared the independent approaches, emphasized decision value, preserved historical registrations and consolidated the executable P87 protocol                | Did not independently establish that P87 will improve throughput                                                    |
| You                    | Maintained experimental boundaries, obtained independent assessments, preserved the research history and selected when to move from analysis to implementation | The next experimental result remains unknown                                                                        |

Several methodological insights emerged:

Source-level plausibility isn't sufficient. The receiver mutex appeared suspicious, but kernel traces established that direct obstruction was too small under the registered criterion.

Faster isolated code isn't sufficient. The unrolled walker improved standalone stage timing substantially on AMD, but did not improve the target i9 four-worker decoder.

A numerical fit isn't necessarily independent validation. Claude's request-occupancy model reproduced the walk slowdown, but partly because its assumed effective overlap was inferred from the quantities being explained.

A strong correlation isn't necessarily causal. More concurrent walks were associated with slower traversal, but longer traversals also create more measured overlap.

An intervention needn't identify every microscopic cause to be worthwhile. P87 may select a useful implementation even if cache residency and dirty-line effects remain entangled.

Engineering benefit and mechanism evidence are separate decisions. A favourable difference-in-differences statistic cannot substitute for actual decoder improvement.

Those principles now underpin the P87 protocol.

## 15. The current authorization and exact next stage

Decision complete: P87 selected as the next controlled experiment

Claude Code — active

Preparing the isolated A/B implementation, frozen protocol, correctness checks, phase split, code-equivalence audit and untimed verification.

Astra — standby

Architectural proposal received and incorporated. Re-engage only if a difficult machine-code or memory-system issue requires specialist adjudication.

Researcher — standby

Independent audit complete. Available to examine the frozen P87 protocol if a specific validity concern arises.

i9 workstation — no task

No timed P87 capture has been authorized. The workstation can be used normally.

### Claude's next deliverables

The consolidated instruction asks Claude to return:

1. An isolated, safe P87 A/B implementation.
2. A frozen experimental protocol with the original P87 historical gate preserved separately.
3. A code-generation audit of construction and traversal.
4. Correctness verification across full blocks, short blocks and repeated scratch reuse.
5. A clean separation of build, traversal, assembly and post-processing timing.
6. Paired A/B and A/A analysis rules, including both the mechanism and engineering gates.
7. Equal-allocation and initial-residency controls without artificial per-block memory warming.
8. An untimed runner dry run and validated analysis tests.
9. The implementation pin, freeze record, precise i9 command and any unresolved caveats.

Only after reviewing that package will we decide whether to execute the real i9 capture.

## 16. The research record and source locations

The principal repository references discussed in this investigation are:

| Record                                                                                                                                                                                   | Purpose                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| [Shipped-decoder attribution records](https://github.com/lmmx/mibz2/tree/docs/decoder-attribution-records/docs/journal/2026-11-01-decoder-attribution-i9-10900K?utm_source=chatgpt.com)  | Original one-versus-four-worker CPU, memory and PMU measurements   |
| [Claude's BWT memory proposal](https://github.com/lmmx/mibz2/blob/docs/bwt-memory-proposal/docs/journal/2026-11-01-bwt-memory-proposal.md?utm_source=chatgpt.com)                        | Phase analysis, concurrency overlap and proposed neighbour harness |
| [Dispatch causality records](https://github.com/lmmx/mibz2/tree/docs/dispatch-causality-records?utm_source=chatgpt.com)                                                                  | Frozen MPMC diagnostic, reproduction and wake-placement postmortem |
| [Research portfolio register](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md?utm_source=chatgpt.com) | Historical P87 definition and 6–10-worker criterion                |
| [Decoder scratch and block path](https://github.com/lmmx/mibz2/blob/45b366d1455a487f1655def689ebd8c87a31f135/src/lib.rs?utm_source=chatgpt.com#L310)                                     | `ll`, `rle`, inversion and RLE1 flow                               |
| [Segmented inverse-BWT implementation](https://github.com/lmmx/mibz2/blob/45b366d1455a487f1655def689ebd8c87a31f135/src/bwt.rs?utm_source=chatgpt.com#L79)                                | T-vector construction and segmented walker                         |
| [External research reports archive](https://github.com/lmmx/mibz2/pull/120?utm_source=chatgpt.com)                                                                                       | Preserved independent reviews and coordination history             |

The independent Astra and researcher messages should also be retained verbatim alongside these records. Their wording, particularly where they distinguish inference from measurement, is part of the scientific history.

## 17. The consolidated research thesis

The story of this stage of `mibz2` research is not simply that we found a promising buffer optimization.

It is that we progressively distinguished three fundamentally different kinds of performance claim.

### A. Where is the extra CPU time spent?

The attribution study established that inverse BWT dominates the additional CPU cost when scaling to four workers.

Claude narrowed this further to the walk-plus-assembly interval and quantified its whole-corpus penalty at approximately 0.609 CPU-s/GB.

This is the strongest measurement-backed answer we currently have.

### B. Why does that work become more expensive?

Several physical mechanisms remain plausible: cache eviction, write allocation, prefetch timing, request queueing, phase interference and possibly address-translation pressure.

The old counters cannot uniquely distinguish them.

Claude's inferred constant-overlap model and concurrent-walk correlation are useful hypotheses, but not independent causal demonstrations.

This question remains open.

### C. Which expense can we actually eliminate?

Astra's P87 proposal changes the lifetime and destination of an intermediate byte buffer while retaining the expensive traversal algorithm.

A controlled positive result would show that the real decoder can reduce some concurrency-related cost through memory-layout choices.

A controlled negative result would show that this particular buffer-lifetime intervention is insufficient, while leaving the broader T-vector memory-system problem open.

That is why P87 is the next experiment.

It moves us from observing expensive memory behaviour toward deliberately changing one part of that behaviour and measuring whether the full decoder benefits.

The intended research sequence is now:

Measured decoder penalty

3.579 versus 2.707 CPU-s/GB

Independent architectural proposals

Astra P87 · Claude neighbour harness

Adversarial audit

Correct PMU claims, statistical gates and causal controls

Current: Claude prepares frozen P87 diagnostic

No timed i9 capture yet

Future: controlled four-core A/B

Mechanism and engineering decisions assessed independently

Next architectural choice

Validate buffer reuse, or pursue calibrated traversal-interference research

### Final status

We've completed the broad attribution, dispatch-causality investigation, independent microarchitectural analysis, adversarial comparison and experiment selection.

The active work is now serial: Claude Code is preparing the P87 diagnostic. Astra's proposal, Claude's discoveries and the researcher's corrections have been consolidated into that task, while Claude's broader neighbour-harness design remains preserved as a separate future option.

The next milestone is not another opinion or another speculative optimization. It is a frozen, auditable P87 experiment that can tell us whether changing intermediate-buffer lifetime improves the decoder we actually care about.

That is the precise point the research has reached, and the evidence supporting it.
`````
