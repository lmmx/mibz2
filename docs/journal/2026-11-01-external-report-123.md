# External research report 123: the guardian's message to the coordinator: dispatch closed, one inverse-BWT memory experiment proposal (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** the maintainer's request for a message to the coordinator.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
Tell Claude that dispatch is closed for now, wake placement remains an open research lead, and its next deliverable is one causally discriminating inverse-BWT memory experiment proposal.

Since Claude has already archived the dispatch results, we shouldn't make it repeat that work. Here's the message I'd send.

Claude, we've reviewed your dispatch-causality postmortem with Guardian and the independent researcher, and we're aligned on the next step.

## 1. Dispatch: close the present investigation

Your archived analysis on `docs/dispatch-causality-records` is accepted as the current evidence.

Preserve two distinct conclusions:

- **MPMC receiver replacement failed its registered test.** Direct receiver obstruction averaged 1.90%, with 0/10 runs reaching the 5% threshold. No MPMC implementation is authorised.
- **Wake-affine scheduling remains an untested optimization hypothesis.** The long runnable delays, same-CPU wake placement and intervals with another allocated CPU idle are credible evidence of a scheduling interaction, but do not establish recoverable throughput.

Retain the exploratory findings in the postmortem. Do not erase them under the negative MPMC conclusion, and do not reinterpret them as a positive MPMC result.

The scanner's runnable-but-not-running time also needs to be interpreted in terms of whether the admission window is actually open. Waking the scanner earlier while the window is full may accomplish nothing.

Park worker affinity and condition-variable redesign for now. If we revisit affinity, the entire scanner/feeder/reader/worker system must fit within the same four physical cores; rigidly assigning one worker per core is not automatically beneficial.

**No more dispatch experiments at present.**

## 2. Main task: inverse-BWT memory-system investigation

Please now prepare **one bounded, falsifiable experiment proposal** that can distinguish the physical causes of the additional inverse-BWT cost under four-worker concurrency.

The measured opportunity is approximately **0.68–0.72 CPU-s/GB**, representing roughly 78–83% of the *incremental scaling penalty*, not 78–83% of total decoder CPU time.

The existing attribution showed approximately 30% more cycles per byte at four workers, with almost no increase in retired instructions. We need to identify what makes those instructions more expensive.

Investigate these competing, potentially overlapping explanations:

1. T-vector cache residency and eviction.
2. Software prefetch interaction with demand loads, cache capacity and outstanding memory requests.
3. Intermediate-buffer writes, write allocation and dirty-line eviction.
4. Concurrent BWT construction versus traversal, including phase-overlap interference.
5. Uncore and memory-request scheduling effects.

For each, distinguish what our existing measurements establish, what remains uncertain, and which observation or controlled intervention would discriminate it from alternatives.

Do not treat increased DRAM writes as proof that writebacks cause the slowdown, or increased LLC misses as proof that cache capacity alone is responsible. Correlation is not causal identification.

## 3. Propose an experiment that changes an engineering decision

Please review our existing implementation, attribution records and previous negative experiments before selecting the next test.

Prioritize **one preferred experiment**, not a broad new benchmarking campaign.

Specify:

- Its precise causal hypothesis and strongest alternative explanation.
- What can be learned from existing archived evidence before collecting anything new.
- The smallest required measurement or controlled intervention.
- Relevant i9 hardware events and their interpretation limits.
- The expected outcomes and how each would change our engineering direction.
- Correctness, CPU-s/GB, full-decoder throughput and RSS safeguards.
- The potential recovery relative to the measured 0.68–0.72 CPU-s/GB penalty.

In particular, explain what evidence would favour:

- Keeping the four-byte T-vector and reducing avoidable memory traffic.
- Changing build/walk phase overlap or scheduling.
- Reopening compact or composed Ψ research despite its construction and traversal costs.
- Pursuing a different explanation entirely.

Avoid rerunning established negative prefetch, copy-elimination, pair-walking, MTF or unrolled-walker experiments unless the new proposal identifies a materially different causal mechanism.

## 4. Deliverable and stopping point

Produce a concise research note containing:

1. The existing evidence and unresolved causal questions.
2. A hypothesis-discrimination matrix.
3. Your **single preferred next experiment**, with its expected observations and engineering decisions.
4. A proposed frozen protocol and runner approach if new i9 measurements are necessary.
5. Your recommendation on whether the experiment merits authorization.

**Research and proposal only. Do not implement new instrumentation, modify production code, or run new benchmarks yet.**

Once the proposal is ready, stop and bring it back for review. We will assess whether the experiment can genuinely distinguish mechanisms before authorizing it.

Our guiding question is:

**Which physical cost can we demonstrably remove from four-worker inverse-BWT decoding, and what is the smallest decisive experiment that tells us how?**

That's all I'd send. No need for another coordination round before Claude produces the proposal.

We're retaining the valuable wake-placement discovery without allowing it to distract from the larger measured BWT opportunity.
````
