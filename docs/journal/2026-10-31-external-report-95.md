# External research report 95: the unrolled walk's stage PASS: 12.3% on Zen 3, replicated on Zen 4 (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 17:22 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** CI run 41222735026.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Worked for 16s

This is a breakthrough! Astra identified a machine-code inefficiency, Claude implemented the proposed transformation, and the first registered performance experiment passed.

More importantly, the improvement is larger than Astra's conservative prediction—and it replicates on Zen 4.

### The measured result

COMPLETE BWT WALK — SINGLE COPY

### 12.3%

Faster on counted Zen 3 job

Time ratio 0.877 \[0.863, 0.891\]

### 13.1%

Less time on Zen 4

Time ratio 0.869 \[0.851, 0.888\]

\*r1 was void under the preregistered A/A rule; it is shown for context, not counted toward the formal verdict.

The particularly satisfying part is that we didn't need a new BWT algorithm. We removed dynamic lane-index bookkeeping, kept all 32 independent chains, and let the CPU process the same traversal with substantially less instruction overhead.

This is exactly the class of optimization you hoped Astra could rescue.

One terminology detail: the counted Zen 3 result is a 12.3% reduction in walk time, equivalent to roughly 14% higher walk throughput, not a 12.3% complete-decoder speedup.

### What I recommend next

Approve preparation of a separate full-decoder trial for the unrolled walker, but run the frozen P74-C i9 experiment first.

We have two promising improvements now, and it's important to establish their independent contributions before combining them.

| Experiment                     | Current status                       | Next decision                                           |
| ------------------------------ | ------------------------------------ | ------------------------------------------------------- |
| P74-C MTF                      | Positive exploratory full-decoder CI | Run the frozen four-core i9 trial                       |
| Constant-index BWT walk        | Registered stage PASS                | Prepare a decoder-level integration and preregistration |
| Combined P74-C + unrolled walk | Not measured                         | Consider only after separate decoder results            |

The Walker trial should measure not just throughput but CPU-seconds/GB and peak RSS, using a real decoder configuration with four physical cores. The secondary SMT result is particularly important: we can't simply apply the 12% walk saving to every parallel configuration.

On the historical 10.78 c/B decoder, a 0.43–0.77 c/B saving corresponds roughly to 4–8% potential complete-decoder throughput improvement, assuming the saving translates without additional scheduling or memory costs. That's worth validating.

### Message to Claude

Excellent result. I approve moving the constant-index BWT walker to the next research phase.

Please preserve CI run 41222735026 and its registered PASS, including the void Zen 3 job, the Zen 4 replication, the exact binary and the amended structural-audit record.

Prepare a separate, production-equivalent full-decoder trial for the unrolled walker, following the lessons of P74-C: identical scheduling and worker configurations, same-binary shipped/unrolled/A-A comparison, correctness and affinity verification, CPU-seconds/GB, throughput and peak RSS, with 1–4 workers on four physical cores where available.

First, establish how the new walker would be selected in the actual production decode paths and ensure it does not silently alter the MTF kernel or other stages.

Draft and preregister the decoder-level hypothesis and acceptance criteria before timing. Keep the unrolled walker research-only and off by default. Do not change the P74-C frozen branch or run another i9 experiment yet.

P74-C's existing i9 trial takes priority when I next have workstation access. Do not combine P74-C and the unrolled walker until their independent full-decoder effects are established.

Stop after preparing the trial protocol and implementation plan for review; no production merge or default change is authorized.

### Astra's contribution

I'd consider this a genuine demonstration that the specialist workflow works:

Machine-code diagnosis → minimal transformation → exact-binary audit → correctness → preregistered performance improvement.

And we achieved it without using Astra to iterate on compiler output repeatedly.

I'd save its remaining quota for the next obstacle rather than asking it to polish this already-successful kernel.

The exciting development is that we may now have two independent decoder improvements: P74-C in MTF and constant-index expansion in the BWT walk. The next task is to determine how much of each survives the complete production pipeline.
````
