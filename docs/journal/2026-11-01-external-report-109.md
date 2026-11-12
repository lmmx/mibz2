# External research report 109: the consolidated instruction for the dispatch-causality diagnostic (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 12:50 UTC.
- **Source:** the guardian.
- **In reply to:** report 108.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
guardian (on the researcher's response):

The researcher and Astra are now in agreement, and I think we should stop the consultation loop and let Claude build the diagnostic.
The researcher's three refinements are worthwhile. They strengthen Astra's experimental design without changing its recommendation.
The most important addition is clock synchronization. If Claude combines Rust `Instant` timestamps with Linux scheduler tracepoints without aligning their clocks, we could end up drawing incorrect conclusions from an otherwise well-designed experiment.
I also strongly agree that runnable-but-unscheduled time must not automatically be counted as recoverable CPU capacity.
Here's the complete instruction I'd send Claude, incorporating Astra's recommendation and the researcher's refinements.
Astra and the independent researcher agree on the next step: a bounded, diagnostic-only dispatch-causality experiment. We are not authorizing an MPMC queue, condition-variable split, or other performance optimization yet.
Please implement and validate the diagnostic on a separate research branch, following our established protocol → implementation → freeze → untimed dry-run sequence. Stop before real i9 capture for my authorization.
First: independently verify the existing evidence.
Reconstruct Astra's scheduler accounting from the archived ten four-worker traces:

* 33.5% runnable-but-not-running time across all inter-decode gaps.
* 78.0% across the 120 distinct inter-decode gaps encompassing the 149 zero-backlog `JobLock` observations.

Keep encompassing-gap scheduler measurements distinct from individual enqueue-to-claim delays. Do not modify the archived results.
Instrument the shipped dispatch mechanism without changing its behavior.
Use bounded, preallocated per-thread records with OS thread IDs to capture:

* Send-call entry and return;
* Receiver mutex acquisition attempt and successful acquisition;
* Receive entry and return;
* Receiver guard release;
* Job claim and decoding boundaries;
* Result publication and notification boundaries;
* Reader wake-up and output-taking boundaries.

Preserve the existing receiver mutex lifetime, FIFO semantics, admission window, worker count, cancellation, errors, shutdown, buffer ownership, output ordering and bounded memory.
Correlate user-space events with Linux scheduler wake-up, context-switch and futex activity, identifying receiver-mutex and channel-wait futex addresses where feasible.
Three additional requirements from the independent review:

1. Clock alignment: Rust `Instant` and kernel tracing timestamps must be demonstrably comparable. Use a common monotonic clock where possible, or record synchronization anchors and quantify alignment uncertainty. Reject interpretations when timestamp uncertainty is too large to distinguish the candidate mechanisms.
2. CPU availability: For every suspected receiver-ownership obstruction, distinguish actual mutex blocking, runnable-but-unscheduled time and CPU availability within the four-core allocation. A runnable worker delayed while all four cores are productively occupied is not automatically recoverable idle capacity.
3. Nonselective stopping: Start with three diagnostic repetitions and expand to ten only based on trace completeness, clock alignment, event correlation and perturbation checks—not whether the preliminary findings favor MPMC. Treat the 90% classification target as a validity objective; report unresolved time rather than forcing an explanation.

Predeclare the causal decision rules.
Promote an MPMC experiment only if actual receiver ownership or serial receiver wake handoff repeatably obstructs an otherwise usable worker for a material amount of time, provisionally at least 5% of aggregate worker time.
If the receiver is generally free and the observed delays instead reflect scheduler placement, CPU availability, FIFO backlog or other worker activity, report that finding and do not proceed to an MPMC prototype.
If publication, reader notification or shared-state locking dominates, identify the mechanism separately without changing it during this trial.
Require correctness, trace completeness, clock-alignment validation, bounded trace memory and assessment of tracing overhead against the 3% throughput/CPU-cost threshold. Preserve the shipped decoder's machine code in uninstrumented builds and record any unsupported OS tracing facilities explicitly.
Use frozen Wikidata as primary, with the existing one-worker reference and appropriate correctness controls. Keep diagnostic interpretation separate from any future production performance gate.
Deliverables before real timing: the protocol and decision rules, source-level instrumentation review, test results, frozen hashes/commit, exact i9 command, expected archive contents, privilege requirements and an untimed dry-run report.
Do not launch the real i9 capture or any new optimization benchmark. Both P74-C and unrolled-walker experiments remain closed.
One final observation: this collaboration has now delivered something more useful than another speculative optimization proposal. We have a concrete causal question, competing explanations, and instrumentation designed to distinguish them.
My recommendation: send this to Claude, let it complete the preparation, and reserve further Astra involvement for interpreting the resulting evidence.
````
