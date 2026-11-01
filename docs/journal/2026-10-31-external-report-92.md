# External research report 92: Astra's first result: a specific, removable machine-code cost (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 16:10 UTC.
- **Source:** the guardian.
- **In reply to:** Astra's evidence-pack review.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Guardian:

That's a strong result from Astra. Not because it found a miraculous 25% improvement, but because it identified a specific, removable machine-code cost without inventing a memory-level parallelism problem that the evidence doesn't establish.

I would authorize one bounded implementation experiment. Astra has done its job; Claude Code should now take over.

### What Astra established

The hot loop processes each of the 32 independent BWT chains using 16 instructions per lane step. Six of those instructions are dynamic lane-loop bookkeeping.

Astra proposes expanding the entire 32-lane round into constant-index steps, removing the bookkeeping while retaining all 32 chains, the memory-backed lane state and the existing prefetch strategy.

ORDINARY LANE STEP — EXPECTED MACHINE CODE

Current

### 16

Instructions per lane step

Proposed

### 10

Instructions per lane step

Memory accesses

Unchanged

Independent chains

32 retained

The predicted saving is modest but worthwhile to test.

| Metric                                                                 | Historical walker |
| ---------------------------------------------------------------------- | ----------------- |
| Existing                                                               | 4.49 c/B          |
| Astra's conditional estimate                                           | 4.0–4.3 c/B       |
| Potential saving                                                       | 0.2–0.5 c/B       |
| Approximate full-decoder throughput upside, assuming the same baseline | 2–5%              |

That final percentage is a model-based estimate, not a measured result.

### What I particularly like

Astra corrected our central hypothesis.

We originally suspected that 32 logical lanes were somehow being serialized into approximately eight effective chains. Astra found no assembly evidence for that.

Instead, the walker has independent memory chains, but it spends substantial instruction bandwidth on bookkeeping.

This is a more actionable finding. It gives us an exact expected assembly signature, a constrained implementation and a clear failure criterion.

I also agree with not attempting register residency for all 32 lanes yet. That could introduce spills and recreate the problem in a different form.

### One issue before the implementation

Astra's proposed 5% performance gate is defensible, but it should be understood as a minimum useful stage-level improvement, not a guarantee of decoder-wide value.

I'd accept it for this experiment.

The more important requirement is that the complete 32-lane round remains interleaved. Implementing four groups of eight is fine only if each round advances all 32 lanes once before returning to any lane.

Otherwise, we could accidentally sacrifice memory-level parallelism while removing instructions.

### Message Claude now

Astra has completed its read-only BWT room-walker analysis. I authorize one bounded research implementation of its proposed constant-index 32-lane round expansion.

The finding: the existing `room_segments::<32, PF_L1>` full-lane hot step contains 16 instructions, including six lane-loop bookkeeping instructions. Astra found no evidence that the 32 logical chains collapse into eight serialized chains. It proposes expanding each complete 32-lane round into constant-index scalar steps, retaining memory-backed lane state, one-hop prefetching and immediate segment-boundary handling.

Please use the existing Astra evidence pack and its conditional Codex implementation protocol. Preserve all 32 independent chains and the current T-vector representation, allocator, scheduling and production dispatch. This must be a separate research arm, with no changes to the frozen P74-C trial or production default.

Before any performance timing:
1. Implement the minimal constant-index expansion.
2. Establish byte-for-byte differential correctness and relevant Miri coverage, particularly segment termination, boundary handling, replacement, buffer growth and fewer-than-32-lane inputs.
3. Audit the exact generated binary. The expected ordinary lane step is approximately 10 instructions rather than 16, with no dynamic lane-loop bookkeeping, no additional spills, calls or per-lane status tests. Retain three loads, two stores and one prefetch per lane step.
4. Stop if LLVM fails to produce the intended structural change. Do not tune alternative implementations to force a pass.

Pre-register a paired, single-physical-core room-walk benchmark with a minimum useful 5% saving. Use the unchanged production-relevant input order, measure complete walk cost including setup and tail, and require candidate/shipped upper 95% confidence bound below 0.95 to pass.

Run the experiment on an appropriate Zen 3 CI host if available. Do not use the i9, change production code, merge or release. Report the generated assembly, measured cycles/byte and ns/byte, correctness evidence and registered verdict.

If the structural audit passes but performance doesn't, preserve the negative result and stop rather than iterating on the kernel. Keep this experiment independent of P74-C's full-decoder result.

### How I'd use the remaining Astra quota

Don't send a second Astra call yet.

Claude can implement the proposal and perform the structural audit. If LLVM produces the requested 10-instruction step, we have a valid experimental candidate.

If the result is surprisingly good—or Claude encounters a nontrivial correctness or code-generation obstacle—then Astra can provide a targeted follow-up.

One correction to our longer-term ambitions: this particular transformation probably won't deliver the entire 1.15 c/B BWT saving we were hoping for. But a measurable reduction in walker cost without changing the underlying algorithm would still be valuable, particularly if P74-C also ships.

My recommendation: approve the experiment, let Claude implement it, and preserve Astra for the next genuinely hard obstacle.
````
