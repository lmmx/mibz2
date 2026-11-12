# External research report 113: the researcher's reading of the i9 dispatch archive: do not proceed, scheduling delay larger (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the researcher.
- **In reply to:** the i9 dispatch-causality archive (`i9-10900K.tar.gz`).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
This is a successful diagnostic, even though it rules against the proposed optimization. The i9 capture is valid, the result is consistent across all ten runs, and we can now separate receiver-mutex obstruction from a much larger scheduling effect.

I've inspected the uploaded archive, including `result.json`, the per-run classifications, scheduler traces, CPU occupancy and reader wake-up records.

## 1. The registered decision is unambiguous

Direct receiver obstruction — 4 workers

# 1.90%

Required: ≥5%

0 of 10 runs met the 5% materiality threshold. All validity checks passed.

The 1.90% comprises approximately:

- 1.25% while another worker holds the receiver mutex.
- 0.65% in the registered asleep handoff category, with no current owner.
- Negligible additional time classified as running while another worker holds it.

The registered result is therefore do not proceed to MPMC. This doesn't establish that MPMC could never help; it establishes that the measured receiver-specific mechanism hasn't earned the investment.

## 2. The more interesting result is runnable scheduling delay

I recalculated the time totals across the ten four-worker runs:

| Category                                | Share of aggregate worker time |
| --------------------------------------- | ------------------------------ |
| Runnable in receiver-acquisition phase  | 8.05%                          |
| Direct receiver obstruction             | 1.90%                          |
| Runnable in receive phase               | 1.06%                          |
| All other queued nondecoding categories | \~0.48%                        |
| Total queued nondecoding time           | 11.48%                         |

Of the 8.05% in the receiver-acquisition phase, 5.65 percentage points coincide with at least one idle allocated CPU.

That's the finding I'd preserve for future architecture work.

However, it doesn't mean 5.65% of runtime is automatically recoverable. The receiver-acquisition phase includes worker scheduling delays, potentially after mutex wake-up or preemption. The CPU scheduler can leave a task runnable while another permitted CPU is momentarily idle.

### A further check I performed

I independently matched `sched_waking` to subsequent `sched_switch` events for worker threads across the ten captured runs.

Exploratory wake-to-running analysis — 4,523 matched worker wake-ups

| Allocated CPU occupancy at wake-up | Median wake-to-run | Wake-ups taking over 100 µs |
| ---------------------------------- | ------------------ | --------------------------- |
| At least one CPU idle              | \~40 µs            | 3.7%                        |
| All four CPUs busy                 | \~10 µs            | 37.4%                       |

The busy-core group has a very long tail: its 90th-percentile wake-to-run delay is approximately 2.4 ms.

This is a separate exploratory reconstruction, not part of the registered MPMC decision. It indicates that competition for CPU time is strongly associated with the long wake-up tail. It does not identify which scheduling policy or wake-up history is responsible.

That distinction matters because the decoder runs four workers plus a scanner, feeder and reader within four physical cores.

## 3. The reader result points in a similar direction

For 962 reader handoffs whose transitions could be fully reconstructed:

| Phase                          | Total across ten runs |
| ------------------------------ | --------------------- |
| Publication → reader wake-up   | 2.3 ms                |
| Reader wake-up → running       | 134.4 ms              |
| Running → lock reacquired      | 2.4 ms                |
| Lock reacquired → output taken | 1.7 ms                |

Most of the reconstructed delay is therefore waiting to run, rather than reacquiring the shared mutex.

There are another 331 unresolved handoffs, so this is not an unconditional statement about all reader delays. Nevertheless, it weakens the immediate case for splitting the condition variable.

## 4. What should happen next?

My recommendation is to close the receiver-replacement hypothesis as an immediate priority and return to the inverse-BWT memory-cost programme.

The completed experiments have established an important hierarchy:

| Opportunity                             | Current evidence                                                | Decision                     |
| --------------------------------------- | --------------------------------------------------------------- | ---------------------------- |
| MPMC receiver                           | Direct obstruction below threshold                              | Park                         |
| Split reader/scanner condition variable | Reconstructed reader delay dominated by scheduling              | Park                         |
| Scheduler and wake-up placement         | Substantial runnable delay, but recoverable benefit uncertain   | Retain as secondary research |
| Inverse-BWT memory efficiency           | \~78–83% of four-worker CPU inflation attributable to inversion | Primary research direction   |

I wouldn't launch another major scheduling diagnostic immediately. One inexpensive read-only follow-up could inspect the already captured `sched_waking.target_cpu` and `power:cpu_idle` events to determine whether waking threads are repeatedly targeted at occupied cores despite another core being idle. My exploratory figures suggest this is measurable from the existing archive; a new i9 capture is not the starting point.

But the larger priority is still to understand why concurrent inversion generates so much additional memory-system cost.

The next high-value research question is:

Can we reduce dirty-line traffic, intermediate-buffer occupancy or T-vector contention while retaining the fast four-byte Ψ lookup?

That is more promising for the 1.5–2× ambition than another receiver or notification redesign.

I'd now hand Claude this archive for its frozen, reproducible interpretation, preserve the registered negative MPMC decision, and ask it to propose one bounded inverse-BWT memory experiment—not implement N1 or another compressed representation yet.

You've obtained a useful answer from the i9: the mutex is not the dominant directly measured obstruction. The remaining performance opportunity is more fundamental, and the research can now proceed without spending time on an insufficiently supported queue replacement.
````
