# External research report 115: the guardian on the researcher's reading: redirect to inverse-BWT memory efficiency (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** report 113.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
I agree with the researcher. This is a successful conclusion to the dispatch investigation, and I think we should now redirect the main engineering effort toward inverse-BWT memory efficiency.

The additional scheduler analysis is particularly interesting because it explains why the queue initially looked so suspicious.

## What we've established

Direct receiver obstruction

# 1.90%

Below the registered 5% threshold

Runnable in receiver-acquisition phase

# 8.05%

Mostly scheduling-related waiting

Wake-to-running delays

# \~10–40 µs

Median varies with CPU occupancy

The surprising observation is that a thread woken while all four cores are busy has a shorter median wake-to-run delay (about 10 µs versus 40 µs with an idle core), but a much worse tail: 37.4% exceed 100 µs, compared with 3.7%.

That is consistent with a mixture of quick scheduling opportunities and occasional long waits behind active threads. It doesn't demonstrate an inefficient scheduler policy, but it does explain why a simple average or median can be misleading.

The reader evidence reinforces this: across 962 fully reconstructed handoffs, nearly all measured post-publication waiting accumulates between wake-up and actual execution, rather than lock reacquisition.

### What this means for engineering

I would now treat the priorities as follows:

| Direction                | Decision                                          |
| ------------------------ | ------------------------------------------------- |
| MPMC receiver            | Park — registered mechanism failed materiality    |
| Split condition variable | Park — no convincing lock/notification bottleneck |
| Scheduler placement      | Preserve as a secondary research lead             |
| Inverse-BWT memory costs | Primary priority                                  |

There is one small opportunity worth keeping: the researcher's suggestion to examine the existing `sched_waking.target_cpu` events against CPU-idle state. It's a read-only analysis of data we've already paid to collect. I would welcome it, but not make it a prerequisite to the BWT work.

## The next research question

We've narrowed the expensive part of four-worker decoding to a much more specific problem.

The existing evidence indicates:

- Approximately 78–83% of the additional CPU cost lies within inverse BWT and associated operations.
- Retired instruction count changes little between one and four workers.
- Cache-related latency and write traffic increase substantially.
- Total DRAM bandwidth remains below the machine's previously measured peak.

But we still don't know which memory traffic is causing the expensive interference.

For example, the T-vector may suffer because its own accesses become slower, because intermediate output buffers evict useful cache lines, or because software prefetching and write allocation compete for cache and fill-buffer resources.

These mechanisms would lead to quite different optimizations.

### What I'd ask Claude next

The independent researcher has reviewed the completed dispatch-causality archive and agrees with the registered `do not proceed` outcome for MPMC.

Please independently reproduce and archive the frozen dispatch result without changing any threshold or rerunning the experiment.

The researcher's additional observations are worth preserving as exploratory findings:

- Direct receiver obstruction: 1.90% of aggregate worker time.
- Runnable in receiver-acquisition phase: 8.05%.
- Approximately 5.65 percentage points of that runnable time coincide with an idle allocated CPU.
- Wake-to-running distributions differ substantially depending on CPU occupancy, particularly in their tails.
- For reconstructed reader handoffs, most latency lies between wake-up and actual execution, not shared-lock reacquisition.

Check the source and raw evidence before adopting those exploratory figures. Keep them distinct from the registered decision.

Then prepare a bounded, read-only research proposal for the next inverse-BWT memory-efficiency investigation.

The question is: why does inverse-BWT CPU cost increase substantially at four workers when retired instructions change little?

Please distinguish:

1. T-vector cache residency and eviction.
2. Software-prefetch traffic and demand loads encountering in-flight prefetched lines.
3. Intermediate-buffer writes, write allocation and dirty-line eviction.
4. Overlap between T-vector construction and traversal across workers.
5. Memory-controller and cache request scheduling effects.

Review existing instrumentation and prior experiments first. Avoid repeating already-negative prefetch sweeps, copy-elimination experiments or compact-Ψ designs without a genuinely different mechanism.

For the next proposed experiment, specify the minimum required measurements, their hardware-counter limitations, a falsifiable causal hypothesis, and how the result would select among candidate implementations.

Do not implement or time anything yet. Preserve the shipped four-byte representation as the initial baseline. Keep the MPMC and condition-variable changes parked, and leave the previous optimization trials closed.

## My recommendation

I'd proceed with that handoff to Claude now.

We've demonstrated the value of disciplined measurement twice: first by discovering that a fast unrolled loop didn't improve the target decoder, and then by discovering that a suspicious receiver mutex wasn't responsible for enough directly measured obstruction to justify replacement.

The next challenge is more fundamental and potentially more rewarding: understand exactly why four concurrent BWT inversions become expensive, then design an optimization that reduces that cost rather than merely making isolated machine code faster.

That's where our best chance of a substantial full-decoder breakthrough now lies.
````
