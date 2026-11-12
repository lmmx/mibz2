# External research report 103: approve the attribution diagnostic's implementation; three refinements (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 09:53 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the attribution protocol draft.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
No, I wouldn't spend another Astra call on this protocol yet. Especially if the previous architectural review consumed roughly half your five-hour allowance.

Claude's design is already sufficiently well grounded to authorize implementation and dry-run validation. Astra has contributed the important architectural questions; Claude has translated them into a concrete measurement plan.

I would preserve Astra's remaining quota for interpreting the measurements and deciding what architecture to build next. That's where its reasoning will have much greater value.

## My review of Claude's protocol

The overall design is strong. I particularly like that Claude found existing infrastructure rather than proposing an entirely new instrumentation framework.

| Component                                     | Assessment       |
| --------------------------------------------- | ---------------- |
| Existing occupancy tracing                    | Excellent reuse  |
| Build/walk CPU attribution                    | Essential        |
| Ordered-admission and head-of-line accounting | Essential        |
| Scanner service-capacity calculation          | Essential        |
| Hardware-counter measurements                 | Valuable         |
| Trace perturbation checks                     | Essential        |
| Separate traced and uninstrumented binaries   | Correct          |
| Binary-equivalence check against master       | Strong safeguard |

There are three technical details I'd tighten before authorizing implementation.

1\. CPU-seconds and clock frequency need careful reconciliation.

Claude's R2 asks whether cycles per byte rise much less than CPU-seconds per byte.

That is the right comparison, but the actual diagnosis should account for effective frequency per measured workload, rather than treating a previously observed 4.9 GHz all-core clock as constant.

CPU time increases when processing takes longer, including because of frequency changes. Cycles per byte help separate that effect from additional execution work. We shouldn't automatically classify the residual as memory stalls.

2\. The hardware counters must be feasible on the i9.

The proposed `perf record` attribution of L3-miss stalls by thread and stage function needs checking against the available Comet Lake PMU events. Some named stall events are model-specific, and sampled cache misses don't necessarily identify the instructions responsible for most lost cycles.

Claude should establish event support and multiplexing behavior during preparation, then record unsupported metrics explicitly rather than substitute loosely related events.

3\. The decision rules should handle multiple simultaneous bottlenecks.

For example, head-of-line blocking could coexist with expensive BWT construction. Scanner capacity could constrain throughput even while the workers suffer substantial memory stalls.

The diagnostic should allow multiple hypotheses to remain supported rather than forcing a single winner.

These are implementation safeguards, not reasons to reopen the research question.

## What I'd authorize now

I approve implementing C1–C6 for the shipped-decoder resource-attribution diagnostic on `research/decoder-attribution`, followed by correctness checks, instrumentation validation, protocol freeze and an untimed dry run.

Before freezing, please make three refinements:

1. For R2, measure effective frequency alongside cycles/decoded byte and CPU-seconds/GB. Distinguish frequency changes from additional execution work without automatically attributing the residual to memory stalls.
2. Verify which hardware-counter events are actually supported on the i9-10900K. Use nonmultiplexed groups where feasible, record unavailable events explicitly and avoid treating generic cache-miss samples as proof of stall causation.
3. Allow multiple bottlenecks to be simultaneously supported. The decision rules should rank explanations by the evidence rather than force a mutually exclusive classification.

Preserve the existing R1–R7 thresholds, one- versus four-worker comparison, identical four-core allocation, frozen corpora, and separate traced versus uninstrumented counter runs.

Do not change production decoding or revive either closed optimization experiment.

After the dry run, provide the frozen commit SHA, exact i9 command, expected output artifacts, any sudo/perf permissions required and the estimated resource demands. Stop for my approval before real timing.

## How I'd spend the remaining Astra quota

Now — Claude

Implementation and validation

Implement the diagnostic, validate the counters and freeze the measurement protocol.

After the i9 measurement — Astra

Highest-value reasoning

Interpret the measured bottleneck attribution, challenge Claude's causal explanation and select the next architectural investment.

One further benefit: the diagnostic may tell us that the current 207 MB test corpus is too short for meaningful sustained-scaling conclusions. Discovering that would be worth far more than another speculative optimization.

My recommendation: approve Claude's implementation and dry run, but reserve Astra until the real counter and pipeline measurements are available. We have finally reached the point where better measurements can determine which direction deserves serious engineering effort.
````
