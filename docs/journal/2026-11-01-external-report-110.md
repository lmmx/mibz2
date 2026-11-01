# External research report 110: approving the dispatch-causality capture, and the three questions answered (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 14:09 UTC.
- **Source:** the guardian.
- **In reply to:** the coordinator's freeze and dry-run report at `3b166cc`, with its
  questions (a)–(c).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
Yes—this is exactly where we left off. Claude has completed the dispatch-causality diagnostic we agreed with Astra and the researcher, and has stopped at the correct point: implementation frozen, VM dry run completed, real i9 capture awaiting approval.

This is the experiment intended to distinguish whether queued jobs are delayed by receiver-mutex ownership, scheduler wake-up latency, or ordinary FIFO backlog—before replacing the queue with MPMC.

The preparation looks good. Particularly encouraging are the verified clock alignment, tracepoint coverage, and the fact that the dry run caught two real runner defects before the freeze.

Claude has raised three questions. Here's how I'd resolve them.

## Decisions on the three questions

(a) Should mutex wake-up time count toward the MPMC threshold?

No. Keep the existing gate unchanged.

A worker woken from mutex contention but waiting to be scheduled could potentially benefit from a different receiver design. However, simply adding all that time to the receiver-obstruction total would mistake scheduler delay for demonstrated mutex ownership cost.

Keep it reported as a separate category. We can examine a broader mutex-associated wake-up measure descriptively without treating it as proof that MPMC would recover the time.

(b) Should we add a sched-switch-only fallback if perturbation exceeds 3%?

Not now. Accept the registered provisional outcome.

The 3% limit is there to prevent heavily instrumented traces from generating misleading causal claims. A new fallback would change what we measure and might sacrifice the receiver/futex detail needed to answer the central question.

If the real run fails the perturbation gate, that is useful evidence about instrument feasibility. We can then design a separately registered, lower-overhead capture if justified.

(c) Should we preregister the condition-variable split now?

No. Keep that a separate future decision.

A reader-path finding may justify a subsequent experiment, but preregistering an implementation before seeing the diagnostic would undermine the sequence we've deliberately established.

The current experiment should answer the receiver-ownership and scheduling question. Reader notifications remain a secondary observational target.

## My recommendation: authorize the i9 capture

I wouldn't spend another Astra call or require Claude to modify the frozen implementation.

Implementation and protocol frozen at `3b166cc`

Production machine-code equivalence passed

All eight tracepoints available in the VM

Clock uncertainty \~0.6 µs, below 20 µs limit

3% perturbation gate still needs validation on the i9

That last item is a genuine risk, but the runner is explicitly designed to handle it. We should let the registered gate do its job rather than weaken it in anticipation.

You can send Claude this:

Thanks. Yes, this is the diagnostic we agreed to prepare, and I approve the frozen real i9 dispatch-causality capture.

Answers to your three questions:

(a) Keep the registered receiver-obstruction threshold unchanged. Do not count all mutex-associated wake-to-run delay as demonstrated ownership obstruction. Report it separately for interpretation.

(b) Do not add another fallback tier now. If the 3% perturbation limit cannot be met, preserve the registered provisional outcome rather than adjusting the experiment.

(c) Do not preregister or implement a condition-variable split yet. That remains a separate decision after we inspect this diagnostic.

Please preserve the frozen protocol, pin, correctness checks, source and binary identity, and the existing validity and expansion rules. No amendments, algorithm changes, queue replacement, or new optimization experiments.

The real i9 capture is authorized. Once it completes, independently reproduce the frozen analysis, archive the raw evidence, and report the causal classifications and limitations without retroactively changing any decision thresholds.

Do not interpret runnable-but-unscheduled time as automatically recoverable idle capacity. Distinguish mutex ownership, scheduling, FIFO backlog and publication/reader effects.

Stop after reporting the results. We'll decide the next engineering action from the evidence.

Since the capture runs on your workstation, you can launch it with:

```
cd ~/dev/mibz2
git fetch origin
git switch research/dispatch-causality
git pull --ff-only

cat bench/dispatch-pin
# Compare with Claude's frozen pin before running.

just bench-dispatch i9-10900K
```

The expected archive is:

`target/dispatch-results/i9-10900K.tar.gz`

One small operational point: this requires `sudo` for Linux tracing and should run with the desktop quiet, as with the previous experiments.

The exciting possibility here is that we may finally distinguish a cheap concurrency fix from a misleading symptom. If receiver ownership genuinely blocks available workers, an isolated MPMC experiment becomes justified. If scheduling accounts for the delays while the receiver is free, we can retire that hypothesis and redirect effort toward the much larger inverse-BWT memory-efficiency problem.

Either result would move the project forward.
`````
