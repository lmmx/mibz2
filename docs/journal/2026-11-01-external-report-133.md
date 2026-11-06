# External research report 133: the guardian on Astra's memorandum: authorize Stage 1 of FOR-14/raw, VM-side only (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, together with report 134.
- **Source:** the guardian.
- **In reply to:** report 131 (Astra's memorandum) and the coordinator's assessment of it.

**Evidence cutoff:** 1 November 2026. A decision and an instruction, not a result. It
authorizes VM-side preparation only: no timed i9 run, production change or new
experimental variant.

It is kept exactly as received (rendered text).

## The message, as received

`````text
I would authorize Stage 1 of Astra's compact-navigation proposal, but not the i9 experiment yet.

We've finally reached a useful decision point. Astra has proposed a genuinely different architecture; Claude has checked its important claims against the source and identified an existing harness that can test it; and the independent researcher has already highlighted the risks.

We don't need another broad architectural review. We need to find out whether this representation deserves to exist.

## The next move

P40/P38/P84 — FOR-14 feasibility

Next priority

1. Re-run the structural census on the current corpus.

   Measure row- and traversal-weighted exception frequencies, the actual footprint, and the proportion of records that require fallback. The historical 1.89% exception estimate is a hypothesis about the current data, not a guarantee.
2. Specify the physical record format before optimizing it.

   Resolve how exceptions are located without a second dependent lookup on the common path. Prove that the header, packed offsets and exception metadata actually fit their intended bytes.
3. Build one FOR-14/raw arm in the existing compact-Ψ harness.

   Use the full T-vector only to generate an offline correctness oracle. Preserve the current 32-lane scheduling and prefetch policy, and ensure the full T-vector is absent from the timed working set.
4. Validate correctness and inspect generated code on the VM.

   Test normal, short, periodic, mixed and exception-heavy cases, including the exact reconstructed-byte order and segment boundaries. Report instruction count and dependent-load structure.
5. Stop before any timed i9 capture.

   Return the census, format specification, correctness results and a frozen proposed measurement protocol for independent review.

The distinction is important: the census alone should not be an automatic kill switch at 2% exceptions. A somewhat higher exception rate may still leave a very substantial footprint reduction. What matters is whether exceptions create a larger memory footprint, additional dependent loads or excessive execution overhead.

I would also insist on distinguishing exceptional rows from exceptional records. A single exceptional row that forces an entire 32-row record into a slower format has very different economics from a individually addressable exception.

## One concern I'd address before implementation

Astra's proposed FOR-14 record is promising, but its description of raw exceptions is not yet a complete executable format.

We need an explicit answer to this question:

How does the walker find the raw successor for an exceptional row without paying an extra lookup on every ordinary row?

A per-record exception mask, an inline exception slot, a separate raw arena and a whole-record fallback all have different access costs. The initial specification must choose one and account for its actual bytes and dependencies.

That may prove more consequential than whether the historical exception rate is 1.89% or 3%.

The byte-carry transformation also needs exact validation. It is a clever way to reconcile the shipped `L[j]` orientation with the compact record's natural `F[j]` byte, but a one-step output shift or a segment-start error would make an otherwise fast representation unusable.

## What about the other work?

| Direction                           | Decision now                                                   |
| ----------------------------------- | -------------------------------------------------------------- |
| FOR-14/raw                          | Authorize bounded VM feasibility preparation                   |
| P87 production candidate, PR #128   | Keep parked; no new timed run                                  |
| Wake-placement investigation        | Preserve as the second priority; do not implement concurrently |
| R2/P85 assembly removal             | Design-level reserve, not the main effort                      |
| R1 cooperative teams                | Defer until there is a convincing four-core throughput model   |
| Neighbour harness                   | Do not build                                                   |
| Further architectural brainstorming | Stop for now; test the leading idea                            |

I would concentrate Claude on the FOR-14 experiment rather than launching three concurrent implementation tracks. We want quick, decisive evidence, not another expanding portfolio.

## The instruction I'd give Claude

Astra's architectural recommendation is accepted as the next bounded feasibility investigation. Please begin Stage 1 of P40/P38/P84: FOR-14/raw fixed-record inverse-BWT navigation with co-located byte recovery.

**Authorization covers VM-side preparation only. No timed i9 run, production change or new experimental variant is authorized.**

Proceed in this order:

1. Re-run the existing succinct-navigation census on the current 434-block Wikidata corpus. Report row-weighted and, where meaningfully distinguishable, dynamic traversal-weighted format frequencies; actual record-level exception rates; format bytes per row; and total footprint. Compare against the historical 233-block census and identify any changed corpus characteristics.
2. Write an exact 64-byte record specification before implementing the optimized walker. Resolve the physical layout of FOR-14 offsets, start mask, byte header, exceptional rows, raw successor storage, indexing and worst-case fallback. Account for every byte and every additional load, especially on the common path. Do not assume row-level exception rates translate directly into record-level storage cost.
3. Implement one FOR-14/raw candidate in the existing compact-Ψ grid infrastructure, using precomputed records derived from a full T-vector as an offline oracle. The candidate's measured traversal must not retain the full T-vector as part of its live working set.
4. Preserve the shipped 32-lane traversal organization, prefetch policy, segment starts, termination behavior and output semantics. Validate carried-byte orientation, short blocks, periodic data, exceptional records, truncated segments, mixed inputs, exact output and CRC.
5. Inspect disassembly for ordinary versus exception paths. Report actual navigation instruction counts, dependent-load chains, cache-line references and additional execution work relative to the shipped packed T-vector.
6. Prepare a frozen, minimally scoped i9 navigation-comparison protocol at one and four workers, with predeclared calibration, correctness and stopping gates. Make explicit that precomputed records provide an optimistic traversal-only test and that any successful result still needs a construction-cost budget before a direct builder can be justified.
7. Cap the investigation at Astra's initial three-engineer-day allocation. If the record layout is infeasible, correctness fails structurally, or there is no credible path to an affordable common navigation path, stop early and document the reason.

Return the census, format specification, code audit, test results and frozen proposed protocol for review. Do not launch the i9 benchmark or undertake direct P84 builder implementation yet.

Keep P87 production, R2/P85, wake-placement and the neighbour harness on hold. Do not broaden this trial into another format sweep.

### What happens after Claude reports back?

If the format is valid and the candidate looks plausible, we review the protocol and authorize one small pinned i9 comparison. That measurement should tell us whether halving the table actually compensates for the added navigation instructions at one and four workers.

If it doesn't, we stop this particular format. We don't automatically launch FOR-13, FOR-15, EF, two-tier, another lane count and three prefetch variants.

If it does, Claude gets a tightly bounded P84 direct-builder stage, with the measured traversal saving determining how much construction overhead we can afford.

And if the builder economics work too, then we investigate full-decoder gains.

My recommendation: send Claude the instruction above and let it begin. This is a high-upside architectural bet with a clear opportunity to fail cheaply—which is exactly what we wanted when we decided to stop chasing marginal optimizations.
`````
