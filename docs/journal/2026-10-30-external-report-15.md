# External research report 15: the researcher's proposal for N1's cost validation, and the maintainer's instruction to continue the burn-down (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 14 (`2026-10-30-external-report-14.md`).

**What it contains:** the researcher's plan to turn N1 into a falsifiable CPU experiment.
- **Three gates:**
  - A, representation economics, over the 233-block census with negative controls;
  - B, construction economics;
  - C, dependent-navigation economics.
- **A risk named:** the rank-indexed lookup may chain two dependent loads per jump. The
  plan compares lookup designs, including a direct row-to-record index as an
  experimental ceiling.
- **Pre-registered measurements,** and two acceptance rules: a screening rule, and a
  promotion rule.

**The maintainer's instruction,** sent with it and recorded below: the burn-down of the
open questions is not finished, and the work does not move off it until it is complete.
The portfolio therefore records Gates A–C as the plan for a future proposal, with no
implementation planned yet. Gate A's census is also the burn-down's own measurement, so
it continues as part of the burn-down.

This is source material. Its claims are checked in `2026-10-30-research-portfolio.md`,
not taken as findings.

The text reached us as rendered text (some card labels flattened) and is kept as
received.

## The proposal, as received

````text
# Next stage: turn N1 into a falsifiable CPU experiment

We've done enough mathematics to stop treating N1 as an open-ended research hypothesis. The next stage is cost validation, followed—only if the cost model survives—by a controlled implementation experiment.

I checked [PR #117](https://github.com/lmmx/mibz2/pull/117?utm_source=chatgpt.com). Its accessible branch still contains the corrected composition theory and preliminary three-block results, but not yet the completed 233-block census. I won't present those preliminary numbers as corpus-wide measurements.

## The experiment we should actually run

Test whether a run-native Ψ⁸ decoder can beat the shipped T-vector in total build-plus-walk cost.

Not just walk speed. Not just record size. Not just a kernel microbenchmark.

The decisive comparison is:

\\[ T\_{\mathrm{build8}}+T\_{\mathrm{walk8}} \quad\text{versus}\quad T\_{\mathrm{buildT}}+T\_{\mathrm{walkT}}. \\]

Both must include segment-boundary handling, with identical decoded output.

The comparison should use the shipped mibz2 0.2.1 implementation as its control.

## 1. Three gates, in order

Gate A — Representation economics

Complete the 233-block census and derive, for each block, \\(m_1,m_2,m_4,m_8\\), record payload, gap distribution, split-parent counts and temporary memory. Include random and records corpora as negative controls.

Pass condition: Ψ⁸ is economically plausible for a substantial fraction of the workload; identify fallback blocks explicitly rather than averaging their costs away.

Gate B — Construction economics

Isolate the actual 1→2→4→8 builder, using the corrected two-pass interval join and image-order prefix sums.

Account for run generation, interval-join operations, temporary ordered views, segment caps, final descriptor packing and rank-directory construction.

Pass condition: complete construction cost leaves enough room for a meaningful net build-plus-walk gain. A tiny descriptor array does not count as success if its construction is expensive.

Gate C — Dependent-navigation economics

Measure the complete Ψ⁸ lookup, including rank lookup, descriptor load, output word, successor update and cooperative segment termination.

Pass condition: the reduction in dependent navigation pays for the builder's extra work on the same hardware and same blocks.

## 2. The issue I would investigate first: two dependent loads

Our proposed Ψ⁸ lookup has a possible weakness that record-count mathematics cannot resolve.

The direct T-vector walk essentially retrieves the next state with one dependent lookup.

The rank-indexed Ψ⁸ walker may require:

\\[ j \rightarrow\text{rank-directory entry} \rightarrow\text{descriptor index} \rightarrow\text{descriptor} \rightarrow j'. \\]

That is potentially two serialized memory accesses per eight output bytes.

The eightfold reduction in transition count is valuable, but a longer dependency chain per transition can reduce the gain.

This is especially important when the cooperative walk already overlaps many outstanding memory requests.

The most productive next design comparison is therefore not different shortcut lengths alone. It's different ways of locating the same Ψ⁸ descriptor.

| Lookup design                           | Strength                                  | Risk                                  |
| --------------------------------------- | ----------------------------------------- | ------------------------------------- |
| Rank directory + packed descriptors     | Small table; straightforward construction | Directory-to-record dependency        |
| Fixed row-group directory + descriptors | Direct first-stage address                | More metadata or exceptional handling |
| Direct row-to-record index              | Potentially cheaper lookup                | Adds substantial per-row storage      |
| Current T-vector                        | One direct indexed transition             | One dependent transition per byte     |

The direct row-to-record version is particularly useful as an experimental ceiling: it can help distinguish an expensive rank operation from an expensive multi-step descriptor itself.

It needn't be a viable final design. Its purpose is to answer an existing question—#44, how cheaply we can obtain the successor—with a controlled comparison.

## 3. What I'd pre-register before any optimization

For every experimental variant, collect these at the same measurement boundary:

| Measurement                                 | Why it matters                                        |
| ------------------------------------------- | ----------------------------------------------------- |
| Builder cycles per decoded byte             | Determines whether the new construction pays back     |
| Walker cycles per decoded byte              | Measures actual navigation savings                    |
| Executed descriptor lookups per output byte | Captures segment caps and shorter terminal records    |
| Instructions per output byte                | Distinguishes less work from simply more parallelism  |
| Peak RSS and live bytes per block           | Captures temporary build arrays, not only final table |
| Full-decoder MB/s and CPU-seconds/GB        | Our actual success criteria                           |
| Random/records corpus performance           | Guards against recreating #25-style regressions       |

I'd use two separate acceptance rules.

For an initial builder/walker proof of concept, require at least a 10% paired improvement in build-plus-walk time over the shipped control. That is a proposed screening threshold, not an already-approved project rule.

For promotion to the actual decoder, require a statistically credible improvement in full-decoder throughput, with equal or lower CPU-seconds per GB and no material RSS regression on the four-core target. The i9 remains a validation platform, not a substitute for that target.

The key distinction is that an eight-step walker might be exceptionally fast in isolation while its construction makes the complete decoder slower. We must be willing to reject that implementation.

## 4. What this means for the research-question burn-down

We're now in a position to settle several existing questions with one controlled experimental family rather than separate speculative implementations.

| Existing question                                            | How N1's next stage addresses it               |
| ------------------------------------------------------------ | ---------------------------------------------- |
| #44 — Arithmetic instead of another dependent successor load | Complete Ψ⁸ lookup comparison                  |
| #53 — Direct compact construction                            | Run-native builder cost                        |
| #54 — Best-concurrency throughput                            | Four-core full-decoder measurements            |
| #76 — Compact information near flat-table speed              | Packed lookup against direct-index control     |
| N3 — Eliminate L and T-vector materialization                | Full run-native pipeline, only after N1 passes |
| #43 / #51 — Exceptional lookup and load costs                | Dependent-load attribution                     |

These questions should stay open until the appropriate measurements exist, even though their mathematical components have advanced substantially.

## 5. What I'd hand Claude next

## N1: transition from theory to cost validation

The corrected Ψ⁸ partition, descriptor composition and segment-cap invariants have now earned a staged CPU cost evaluation. Do not start the i9 queue or modify the production decoder.

**First, complete the corpus report.** For all 233 full Wikidata blocks, report per-block m₁/m₂/m₄/m₈, Ψ⁸ descriptor bytes, gap distributions, child-count distributions, capped lookup counts per decoded byte, and modeled peak construction RSS. Preserve distributions and outliers, not only averages. Add the random and records controls where existing inputs allow.

**Next, prepare a standalone builder/walker proof-of-concept plan, subject to maintainer approval.** Its baseline is mibz2 0.2.1. The builder starts directly from L-runs, forms records by 1→2→4→8 composition, and uses the corrected two-pass counting construction: source offsets in source-parent order, destination offsets in second-parent image order. Include the tested construction-time caps for flagged cooperative segment endpoints.

The walk comparison must include both packed rank-indexed descriptors and a direct-index control that isolates rank-directory overhead. Count full construction, directory building, descriptor lookups, output and termination handling.

Pre-register stage-level build-plus-walk time, full-decoder throughput, CPU-seconds/GB, peak RSS and parity across Wikidata, random and records corpora. Treat CI performance as screening evidence; reserve cache-sensitive decisions for paired measurements on relevant CPU hardware.

**Do not claim a performance breakthrough from the representation census.** The next decisive question is whether the complete builder plus walker beats the shipped path, not whether Ψ⁸ is small or mathematically correct.

Retain the existing 84-question inventory; annotate the questions resolved or narrowed by each result instead of creating new entries.

## Where we are now

N1 readiness

Research stage

Mathematical model establishedPerformance not established

We have moved beyond asking whether a multi-byte BWT representation exists. We know how to define and construct one.

The next stage is to make it compete against the real decoder, with the builder included.

If it loses, we'll learn which dependency or construction cost prevents the improvement. If it wins, we'll have something much more valuable than another promising theorem: a measured CPU algorithm that changes mibz2's decoding cost.
````

## The maintainer's instruction, as received

````text
I'll send that but I didn't say stop with the burn down, there is still plenty left to get through there. We are not ready to move off it until it is completed
````
