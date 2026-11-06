# External research report 112: the guardian adopts the researcher's validation fix; the capture's authorisation stands once it passes (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 14:09 UTC.
- **Source:** the guardian.
- **In reply to:** report 111.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
The researcher has caught a genuine protocol-enforcement bug. I would adopt its recommendation and correct that before running the i9 capture.

This is a valuable catch, not unnecessary procedural friction.

The distinction is important: the diagnostic's initial quality gate apparently checks scheduler coverage, but its final verdict calculation doesn't enforce the same checks on every run. That could allow incomplete scheduler evidence to support a confident conclusion about mutex ownership.

We specifically designed this experiment to distinguish mutex contention from scheduling delay. Missing scheduler events could undermine the central result.

I also agree with the researcher's answers to all three earlier questions: don't count mutex-associated runnable delay toward the primary 5% threshold, don't introduce another fallback tier, and don't preregister a condition-variable optimization yet.

## What I'd tell Claude

The independent researcher reviewed the frozen dispatch-causality protocol and identified one final validation issue worth correcting before the i9 capture.

I agree with its recommendation. Please make a narrowly scoped protocol-enforcement correction, without changing the hypotheses, thresholds, causal classifications or decoder behavior.

Specifically:

1. In `bench/dispatch.py`, verify the reported mismatch: the initial quality check requires `threads_in_sched` and `sched_coverage >= 99%`, but the final `summary()` validity calculation apparently omits these checks. If confirmed, enforce both requirements for every four-worker capture used in the final causal decision, including runs after expansion.
2. Verify that lost kernel trace events, parser omissions and incomplete scheduling streams cannot silently produce a valid causal verdict. Distinguish recognized-but-irrelevant events from actual trace loss. Any material loss affecting scheduler reconstruction must invalidate the affected interpretation.
3. Add focused tests that deliberately supply incomplete scheduler coverage, missing thread IDs or reported trace loss, and demonstrate that the final summary refuses a definitive decision. Preserve the distinction between invalid evidence and an ordinary negative result for the MPMC hypothesis.
4. Keep the original frozen 5% direct receiver-obstruction threshold and 80% repeatability requirement unchanged. Report mutex-associated runnable delay separately as exploratory context, not as additional obstruction.
5. Do not add a third capture tier or preregister a condition-variable change.

If the bug is confirmed, document it as a validation-only amendment, update the implementation pin and freeze, and complete the necessary untimed verification. Stop after reporting the updated frozen commit and readiness.

Once those checks pass, the existing authorization for the single i9 diagnostic capture stands. No MPMC implementation, notification change, algorithm optimization or new performance experiment is authorized.

## Why this is the right last correction

The researcher has identified a failure mode that could affect the validity of the causal conclusion, rather than merely suggesting another analysis or optimization.

It's worth fixing before measurement because scheduler traces are the evidence that will distinguish:

- A receiver genuinely obstructed by mutex ownership.
- A worker runnable but unable to get CPU time.
- A worker legitimately waiting behind queued work.

We should also be careful not to turn this into an endless review process. After Claude verifies this issue, fixes it if necessary, tests it and re-freezes, I'd run the diagnostic.

There is no need for another Astra consultation first.

The larger direction remains unchanged: establish whether the MPMC mechanism is real, then return to the substantial inverse-BWT memory-efficiency problem.
````
