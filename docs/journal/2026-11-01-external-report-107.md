# External research report 107: adopting Astra's dispatch-causality diagnostic; the instruction to the coordinator (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 12:49 UTC.
- **Source:** the guardian.
- **In reply to:** report 106.

It is kept exactly as received (rendered text, cards flattened). Report 106 and 107 arrived
in one message, separated here at the relay's `guardian:` label.

## The message, as received

````text
guardian:

Astra has made a strong, actionable decision: measure the dispatch mechanism before replacing it. I agree, and I think this is a better outcome than immediately implementing an MPMC queue.

The most important new finding is the scheduler accounting. Astra went beyond the researcher's queue-delay analysis and found that runnable-but-not-running time accounts for 33.5% of the time between worker decodes, rising to 78% in the inter-decode gaps associated with the suspicious zero-backlog `JobLock` observations.

That changes the engineering question considerably.

## What we now know

| Question                                                              | Current conclusion                     |
| --------------------------------------------------------------------- | -------------------------------------- |
| Does the shared receiver mutex have a potentially inefficient design? | Yes                                    |
| Is it proven to cause the observed claim delays?                      | No                                     |
| Is scheduling delay materially involved?                              | Yes, in the measured inter-decode gaps |
| Would replacing the queue eliminate that delay?                       | Unknown                                |
| Is the inverse BWT still the largest CPU-efficiency opportunity?      | Yes                                    |
| Should we implement MPMC now?                                         | No — diagnose receiver ownership first |

Astra's distinction is particularly valuable: when three workers are actively decoding, none of those workers should simultaneously hold the receiver mutex. The fourth worker's delay could reflect an earlier mutex handoff or scheduling delay, rather than present mutex contention.

That is precisely the kind of causal ambiguity worth resolving before modifying concurrency code.

## The next step

I'd give Claude Astra's complete response and ask it to prepare the narrowly scoped dispatch capture.

Crucially, we do not need another broad architectural review. Astra has identified a specific deciding observation:

Is a worker that could receive a queued job actually prevented from doing so because another thread holds the receiver mutex?

That's what the next measurement should answer.

Astra has completed its architectural adjudication. Its recommendation is to measure receiver ownership and scheduler behavior before implementing MPMC or splitting the reader/scanner condition variable.

I agree. Please prepare a bounded, diagnostic-only dispatch-causality experiment on the shipped decoder. Do not run timed measurements yet.

First, independently reproduce Astra's additional scheduler accounting from the existing ten four-worker traces:

- Approximately 33.5% runnable-but-not-running time across inter-decode gaps.
- Approximately 78% across the 120 distinct inter-decode gaps associated with the zero-backlog `JobLock` observations.

These are encompassing-gap measurements, not fractions of the enqueue-to-claim delays. Verify them without altering the existing archive.

Then design and implement the minimum instrumentation necessary to distinguish:

1. Receiver mutex owned by another thread while a worker is attempting to receive an available job.
2. A worker runnable but unscheduled while jobs are available.
3. A worker blocked inside channel receive, futex waiting or mutex acquisition.
4. FIFO backlog with no genuinely available worker.
5. Time spent publishing results, returning buffers, acquiring output buffers or waiting on the shared-state lock.

Use preallocated, per-thread diagnostic records and OS thread IDs. Capture send/receive brackets, receiver acquisition, receive return and guard release; preserve the exact lock lifetime. Correlate with supported scheduler and futex events, identifying futex addresses where feasible.

Also record result publication and notification boundaries to distinguish reader wake-up delay from lock acquisition and scheduling. Do not change notification behavior.

Keep the shipped queue, worker count, admission window, decoder kernels, affinity policy, cancellation, shutdown, output ordering and bounded memory semantics unchanged.

Prepare a registered protocol with explicit classifications for ambiguous intervals. Require explanation of at least 90% of queued-work nondecoding time, and test instrumentation perturbation against a 3% throughput/CPU-cost threshold.

Use at least three diagnostic repetitions initially, expanding to ten only under prespecified conditions. Treat receiver-specific obstruction occupying at least 5% of aggregate worker time as an investigation threshold, not a predicted throughput gain.

The decisive question is whether receiver ownership or serial wake handoff materially obstructs a worker that could otherwise receive work. If the receiver is generally free and scheduling explains the delay, the protocol must be capable of rejecting the MPMC hypothesis.

Follow our established sequence: implementation, correctness checks, exact production-equivalence check where applicable, protocol freeze and untimed dry run. Stop before the real i9 capture for my authorization.

No MPMC implementation, no condition-variable split, no production-default changes, and no reopening the P74-C or unrolled-walker experiments.

Please return the frozen commit, exact i9 command, data requirements, expected artifacts and any residual instrumentation limitations.

## Where this leaves the bigger optimization programme

I would now maintain two research tracks, but only one active experiment:

Active: Dispatch-causality measurement. It's bounded, relatively cheap and could identify a straightforward synchronization improvement—or prevent us from wasting effort on one.

Next architectural priority: Reducing inverse-BWT memory-system cost. The observed additional inversion cost of approximately 0.68–0.72 CPU-s/GB is large enough to matter for the eventual 1.5× objective. Astra's suggestion of preserving the existing four-byte representation while investigating intermediate-buffer traffic is a sensible direction, but it still needs evidence that traffic reduction actually reduces integrated CPU cost.

I wouldn't spend another Astra call until Claude has completed the new diagnostic and we have a meaningful result.

My recommendation: send Astra's response and the instruction above to Claude. We're moving from plausible explanations to a sharply falsifiable test, which is exactly where this research programme needs to be.
````
