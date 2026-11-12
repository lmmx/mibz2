# External research report 121: the researcher: MPMC failed its test, wake placement is untested; let the BWT proposal decide (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the researcher.
- **In reply to:** report 120.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
I agree with Guardian's assessment. I think we've reached a good point to stop coordinating the dispatch investigation and let the next piece of research determine the direction.

There's one distinction I'd preserve particularly carefully:

The MPMC hypothesis failed its registered test. The wake-placement hypothesis has not been tested.

Those are not contradictory findings. The receiver mutex might cause only 1.90% directly measurable obstruction while still creating a wake-up pattern that interacts poorly with Linux's scheduler. That second effect could potentially be addressed through thread placement, queue design or scheduling policy—but we haven't established which intervention would improve throughput.

The most interesting observation is still the combination of long wake-to-run delays and another allocated core sitting idle. It's compelling evidence that CPU scheduling deserves attention, though not proof that the delay is avoidable or that changing affinity will improve decoding performance.

### One consideration for the future affinity experiment

I would be careful about simply pinning one worker to each of the four cores. The decoder also has scanner, feeder and reader activity competing for those cores. Rigid worker pinning might shorten some wake-up delays while making scanner starvation or cache contention worse.

A good future experiment should preserve the entire four-core budget, examine placement of all the relevant threads, and judge success by full-decoder throughput and CPU-seconds/GB—not by wake latency alone.

That said, I wouldn't design that experiment yet.

### Where I'd focus now

The BWT investigation has the stronger case for substantial gains:

Inverse-BWT memory efficiency

Primary

Explain and ultimately reduce the measured 0.68–0.72 CPU-s/GB scaling penalty. Determine whether the critical cost is T-vector access, prefetch interactions, writebacks or overlapping construction and traversal.

Wake-affine scheduling

Secondary

Preserve the evidence and revisit with a controlled four-core experiment, potentially offering a smaller, cheaper improvement.

The crucial next deliverable from Claude is a BWT memory-system experiment that can discriminate between physical causes, not just confirm that memory stalls exist. We've already established that much.

I'd let Claude finish that proposal before issuing another instruction. Then we can assess whether it is genuinely capable of selecting an engineering intervention, or whether a smaller measurement would be more revealing.

One useful lesson from this whole exercise is that the negative results have been narrowing the search rather than merely consuming time. We now know that a faster MTF stage, a faster standalone BWT loop and an apparently problematic receiver mutex aren't sufficient by themselves to deliver the improvement you're targeting.

The next breakthrough is more likely to come from understanding the interaction between concurrent inverse-BWT workloads and the memory hierarchy—with wake placement retained as a credible independent opportunity.
````
