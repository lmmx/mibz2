# External research report 116: the researcher's refinement: causal discrimination, and the 0.68–0.72 CPU-s/GB budget (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the researcher.
- **In reply to:** reports 114 and 115.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
I agree with Guardian's direction, and I think we're now at a particularly interesting point in the mibz2 research.

The dispatch investigation has done its job. We should close it cleanly and turn the next investigation into a causal test of inverse-BWT memory costs.

There is one refinement I'd make to Guardian's instructions.

## What I'd change

Guardian asks Claude to distinguish five possible causes of the memory slowdown. That's right, but I would make Claude go one step further: identify which observations would actually discriminate between competing causes, rather than produce another report in which all five remain plausible.

For example, these are different explanations:

| Hypothesis                    | What would support it                                                                                |
| ----------------------------- | ---------------------------------------------------------------------------------------------------- |
| T-vector cache eviction       | More costly demand accesses to the T-vector specifically as concurrent inversions increase           |
| Prefetch interference         | Prefetch traffic occupies resources or evicts useful data without sufficiently hiding demand latency |
| Dirty intermediate buffers    | Writebacks and write allocations impose measurable contention on inversion's critical path           |
| Build/walk phase interference | Concurrent construction and traversal perform worse than their individual resource demands predict   |
| Uncore request contention     | Latency increases even where cache-hit and traffic patterns do not explain it                        |

Importantly, these aren't necessarily mutually exclusive. Dirty-line traffic could be a consequence of cache pressure rather than its original cause. An experiment that merely shows fewer DRAM writes after a change wouldn't automatically establish that the writes were the bottleneck.

## The quantitative opportunity

The 78–83% figure refers to the additional CPU cost when scaling from one worker to four, not to 78–83% of the entire decoder.

The actual opportunity is approximately:

Inverse-BWT scaling penalty

# 0.68–0.72 CPU-s/GB

Approximately 19–20% of the four-worker decoder's total CPU cost

That's large enough to be worth serious research. But recovering it entirely would still require better CPU utilization to reach the 1.5× throughput target.

This is why I favour preserving the four-byte T-vector while investigating the memory system. We have an existing efficient algorithm that becomes more expensive under concurrency. Understanding why may reveal a cheaper intervention than rebuilding its data representation.

## My suggested addition to Claude's handoff

I'd send Guardian's second, more detailed message, with this paragraph appended:

One additional requirement for the inverse-BWT research proposal: please distinguish **correlation from causal discrimination**.

For each leading memory-system hypothesis, specify what the existing measurements already establish, what remains unidentifiable, and the smallest measurement or controlled experiment that could distinguish it from competing explanations.

In particular, do not treat increased DRAM writes as proof that writebacks cause the slowdown, or increased L3 misses as proof that LLC capacity alone is responsible. Consider whether both could be consequences of the same working-set pressure.

Prioritize one experiment that would change our engineering decision. Define in advance what result would favour retaining the four-byte T-vector with reduced traffic, what would favour a different scheduling of build/walk work, and what would justify reopening compact or composed Ψ research.

Quantify the potential recovery against the measured 0.68–0.72 CPU-s/GB incremental inversion cost, not against the entire decoder time. No implementation or new timing yet.

One other point: I'd preserve the wake-to-run and CPU-idle findings in the dispatch postmortem, but not allow them to generate another scheduling campaign immediately. They're an intriguing secondary lead, not an established throughput opportunity.

My preferred sequence is now straightforward: Claude archives and closes dispatch, delivers one discriminating memory experiment proposal, we review whether it can actually choose an engineering direction, and only then authorize the measurement.

The most important shift is that we're no longer asking “What optimization looks promising?” We're asking “Which physical cost can we demonstrably remove from the four-worker decoder?”

That's a much stronger basis for pursuing a genuine breakthrough.
````
