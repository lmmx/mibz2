# External research report 108: agreeing with Astra; three refinements (clock alignment, CPU availability, nonselective stopping) (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 12:49 UTC.
- **Source:** the researcher.
- **In reply to:** reports 106-107.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
researcher:

I agree with Astra's decision. It has improved the research programme by refusing to turn a plausible mechanism into a premature implementation.

The most valuable new finding is the distinction between worker time spent waiting and worker time spent runnable but unable to get CPU time.

The existing 2.7% run-queue figure measured scheduling delay inside decoding intervals. Astra's reconstruction finds much more scheduling delay between decoding intervals. Those are different questions.

The second finding is that the receiver mutex may still be problematic, but the original 977 µs claim-delay statistic was confounded by queued jobs and scheduling. We now have a way to distinguish those causes.

## My assessment

Strong conclusion: diagnose dispatch before replacing it.

The source confirms `Mutex<Receiver<Job>>` is held across `recv()`. That establishes serialized receiver ownership, but not that this serialization causes a material throughput loss. Astra correctly separates those claims.

Strong conclusion: inverse-BWT memory efficiency remains the principal architectural problem.

Roughly 78–83% of the additional CPU cost from one to four workers is associated with inversion. But LLC capacity, prefetch interference and dirty-line traffic remain competing explanations—not settled causes.

One remaining concern: avoid overinterpreting scheduler time.

Runnable-but-not-running time may be caused by productive scanner, feeder or worker execution occupying the four-core allocation. It is not necessarily recoverable capacity. The diagnostic must distinguish receiver obstruction from ordinary CPU oversubscription.

I verified the relevant worker ordering in [src/parallel.rs](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/src/parallel.rs?utm_source=chatgpt.com) and the existing timing implementation in [src/trace.rs](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/src/trace.rs?utm_source=chatgpt.com).

## Three refinements before Claude builds the diagnostic

First, align user-space and kernel timestamps. The existing trace timestamps use a shared Rust `Instant` epoch. Scheduler tracepoints use the kernel's tracing clock. Simply joining the two by numerical timestamps could introduce alignment errors. Claude should either use a common clock or record synchronization anchors and validate their offset.

Second, distinguish blocking from CPU availability. For every suspected ownership obstruction, establish whether the blocked worker could actually have run on one of the allocated physical cores, or whether all four were occupied. A mutex obstruction with no available CPU may still matter, but it is not equivalent to idle-core wastage.

Third, keep the diagnostic stopping rule about instrumentation quality, not promising performance data. Astra suggests three repetitions followed by up to ten. That is reasonable if expansion depends on trace completeness, event correlation and perturbation—not whether the preliminary result favours MPMC. Report unresolved intervals rather than forcing them into a cause to reach the 90% classification target.

## The next engineering decision

| Evidence from diagnostic                                                        | Action                                                          |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| Receiver held by another worker while a runnable receiver is materially delayed | Prototype MPMC independently                                    |
| Receiver normally free, worker delayed by scheduler                             | Investigate wake-to-run latency and four-core scheduling        |
| Significant delay occurs in publishing results or output admission              | Investigate shared-state notifications and ownership separately |
| Backlog explains most delay and all cores are productively occupied             | Stop dispatch research; prioritize BWT efficiency               |

My preference is to authorize one diagnostic-only dispatch capture, exactly as Astra proposes, with the three refinements above.

Crucially, this should be a short causal investigation, not another open-ended performance project. If receiver ownership accounts for less than the proposed 5% of worker time, retire MPMC as an immediate priority.

After that, the larger research investment should remain the four-byte inverse-BWT representation and the question of which memory traffic is actually creating the extra scaling cost. Avoiding unnecessary intermediate materialization is a credible candidate, but the prior measurements cap its direct benefit to a few percent; a larger win requires demonstrable reductions in concurrent memory-system contention.

Research decision: Astra has earned its quota here. It didn't simply recommend a clever optimization; it identified the evidence needed to avoid an expensive wrong turn. I would stop architectural consultation for the moment and have Claude implement this one bounded diagnostic, then return with the results.
````
