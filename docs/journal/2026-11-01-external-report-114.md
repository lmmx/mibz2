# External research report 114: the guardian on the i9 archive: the MPMC hypothesis did not pass (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** the i9 dispatch-causality archive.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
## We have our answer: the MPMC hypothesis did not pass

Diagnostic valid

Frozen i9-10900K capture · 10 four-worker runs · all validity checks passed

This is a significant result. The shared receiver mutex does cause some genuine obstruction, but not enough to justify replacing the queue under our registered criteria.

Direct receiver obstruction

# 1.90%

Against 5% threshold

Runs meeting threshold

# 0 / 10

Required repeatability not met

Runnable share of queued nondecoding time

# 80.2%

Not necessarily recoverable idle capacity

I checked the uploaded archive's `result.json`. It confirms the console verdict, including complete scheduler traces, valid clock alignment and acceptable instrumentation perturbation.

### What I find particularly interesting

The exploratory statistic is much larger than direct obstruction:

- 1.90% of aggregate worker time is classified as direct receiver obstruction.
- 9.95% is receiver-associated time when we also include runnable-but-unscheduled time during the receiver-mutex wait phase.

That is a meaningful distinction. The mutex is associated with substantial waiting, but the kernel-level reconstruction doesn't establish that all this time could be removed by changing the mutex.

The result vindicates Astra's insistence on separating actual lock ownership from scheduler delay.

### There's also an interesting reader finding

The completed reader observations suggest that once the reader is notified, its own lock reacquisition is generally quick. For resolved cases, notification-to-wake and post-scheduling lock acquisition are typically only a few microseconds each, while wake-to-running medians are around 40–46 µs.

That argues against treating the shared condition variable itself as the obvious optimization. Some reader observations remain unresolved, so this isn't a complete causal explanation.

## What should happen next?

I'd have Claude independently reproduce the archive's verdict, document the cause breakdown and close this diagnostic under its registered do-not-proceed outcome.

Then I'd return to our larger opportunity:

Why does concurrent inverse-BWT decoding incur so much additional memory-system cost, and can we reduce it without changing the efficient four-byte link representation?

The next research direction should investigate the measured traffic and cache effects—particularly software prefetch interaction, intermediate-buffer writebacks and the costs of overlapping BWT construction and traversal—before selecting another implementation.

I would not authorize an MPMC experiment from this result, and I wouldn't immediately authorize a condition-variable split either.

### Message for Claude

The frozen i9 dispatch-causality diagnostic completed successfully. The uploaded archive confirms all registered validity checks passed and the outcome is `do not proceed`.

Direct receiver obstruction averaged 1.90% of worker time, with zero of ten runs reaching the 5% materiality threshold. The receiver-associated exploratory share was 9.95%, but includes runnable delay and must not be reclassified as direct obstruction.

Please independently reproduce the frozen result from the raw archive, reconcile the receiver/futex/scheduler classifications, and document the remaining reader-path uncertainties. Preserve the registered result without changing its thresholds.

Archive the complete evidence and close the MPMC hypothesis as an immediate engineering priority. Do not implement MPMC, split the condition variable, rerun benchmarks or modify production dispatch.

Then provide a short, read-only recommendation for the next *inverse-BWT memory-system attribution* question, based on the earlier one-versus-four-worker counters and the existing research history. Distinguish traffic volume, cache residency, prefetch behavior and dirty-line eviction. Do not implement or time anything yet.

This is a productive negative result. We had a plausible source-level bottleneck, investigated it using properly validated Linux scheduler evidence, and established that it isn't the major opportunity we'd hoped for.

We've avoided another potentially expensive detour. The larger performance opportunity remains in making the inverse-BWT workload cheaper under four-core concurrency.
````
