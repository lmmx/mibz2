# External research report 84: spend Astra's quota on the BWT walker, not P74-C (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:28 UTC.
- **Source:** the researcher.
- **In reply to:** planning the next step.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
researcher:

I'd spend Astra's scarce quota primarily on the BWT walker, not P74-C. Guardian has the right specialization but slightly the wrong ordering.

P74-C already demonstrates its mechanism. The next question is whether it improves the complete decoder. An assembly review before that measurement risks optimizing a 5% opportunity while the 52%-of-runtime BWT walk remains largely untouched.

There is also an important target correction: the four-worker production decoder uses plain MTF, not checkpoint MTF. If full-decoder integration reveals a P74 code-generation regression, Astra should inspect the plain production kernel, not the checkpoint kernel whose 4,741→4,091-byte reduction attracted attention.

### Astra allocation: one high-value problem at a time

| Priority | Assignment                                     | Trigger                                             |
| -------- | ---------------------------------------------- | --------------------------------------------------- |
| 1        | BWT scalar-walk critical-path analysis         | Now, read-only                                      |
| 2        | One targeted walker code-generation correction | Only if #1 identifies a concrete defect             |
| 3        | P74-C plain kernel register/layout repair      | Only if the full-decoder trial exposes a regression |
| Reserve  | Follow-up assembly verification                | Only after a promising change                       |

Why the walk dominates the opportunity: on the EPYC 7763, total decoding costs 12.93 c/B, of which walking costs 6.69 c/B. Every 1 c/B saved in the walk removes 7.7% of total decoder time.

Assuming P74 saves 0.65 c/B, reaching a combined 20% throughput improvement requires approximately:

\\[ \Delta c\_{\rm walk}\ge 12.93(1-1/1.20)-0.65 =\boxed{1.50\ \mathrm{c/B}}. \\]

That is a 22.4% reduction in walk time. For 20% less decoder time, the walk needs to save 1.94 c/B, or 29.0%.

### The first Astra assignment I would send

Inspect the optimized machine code for `src/bwt.rs::room_segments::<32, PF_L1>`, particularly the full-lane inner loop.

Find whether avoidable instruction dependencies and bookkeeping prevent 32 independent T-vector chains from saturating the core. Specifically analyze:

- `Lane.row` load → T-vector load → next-row recurrence;
- per-byte prefetch cost and load/address-generation pressure;
- repeated loads/stores of `Lane` state and output pointers;
- flag branches, register spills and instruction-window occupancy;
- whether an 8-lane scalar tile could amortize bookkeeping before introducing AVX2 gathers.

Return one supported transformation, its expected assembly change, the estimated cycles/B saving and a falsification criterion. If no plausible path saves roughly 1.5 c/B at the decoder's walk boundary, say so. Do not patch speculatively or benchmark without authorization.

Claude should supply exact rustc flags, disassembly, stage timings and target CPU beforehand. Astra's quota should go into reasoning about the generated dependency graph, not context collection.

On P74-C: don't spend Astra calls trying to remove the extra `lea` merely because it exists. First establish whether the frozen plain integration loses real decoder throughput to register allocation. The checkpoint function shrinking by 13.7% is interesting but not a reason to optimize it ahead of the four-worker target.

Management decision: spend one Astra call on BWT diagnosis now. Reserve further calls for a concrete assembly-level hypothesis. Continue the frozen P74 full-decoder gate independently. This maximizes Astra's expected contribution to a breakthrough instead of consuming its quota polishing an already-successful stage optimization.

guardian:


I agree with the researcher on Astra: spend its scarce quota on the BWT walker, while Claude advances P74-C to the full-decoder trial independently.

Claude's clarification is important, too. The Astra-derived error-state optimization was already included in every P74-C benchmark we reviewed. We shouldn't spend another call investigating that same improvement.

There are now two complementary paths:

- P74-C: establish whether the measured MTF gains improve the actual decoder enough to justify shipping.
- BWT traversal: investigate a much larger potential performance opportunity through machine-code analysis.

I would proceed with both, but there are two details in Claude's proposed full-decoder protocol I would adjust before freezing it.

## My decisions on Claude's four questions

| Decision                                              | Recommendation           |
| ----------------------------------------------------- | ------------------------ |
| Off-by-default P74-C integration on a research branch | Approve                  |
| One full-decoder i9 trial plus exploratory CI         | Approve                  |
| Approximately 12 MB of new held-out Wikidata data     | Approve                  |
| Proposed performance thresholds                       | Approve with refinements |

### Refinement 1: measure three workers

The researcher identified that four decoder workers also compete with the scanner and producer.

Claude proposes measuring 1, 2 and 4 workers. I'd add 3 workers, without changing the primary four-worker acceptance criterion.

That gives us the complete scaling curve within a four-physical-core allocation, and may reveal whether three workers are economically preferable when the scanner needs CPU time.

### Refinement 2: make the CPU/RSS criterion physically meaningful

The proposed +1% upper confidence bound is reasonable as an ambition, but peak RSS is a high-water-mark measurement rather than an ordinary smooth timing sample. Applying the same confidence-interval machinery to RSS without considering allocation variability may be misleading.

I'd keep a +1% peak-RSS ceiling, but require the protocol to state exactly how it's aggregated and compared. For CPU-seconds/GB, +1% is a sensible practical tolerance, although it is not strictly equivalent to the original goal of non-increasing CPU consumption.

The four-core throughput requirement—at least a 2% gain at the lower confidence bound across all three Wikidata corpora—is demanding but defensible given the projected 3.5–4.5% gain.

Importantly, all three corpora need not be treated as statistically independent evidence, since two are from the same dump. They can be separately required without pooling their estimates.

## What Astra should actually investigate

The researcher has identified the right function: `room_segments::<32, PF_L1>` in `src/bwt.rs`.

The most valuable question is not whether the source code has too many instructions. It's whether the compiled loop fails to sustain enough independent BWT traversal chains because of avoidable dependencies.

I'd ask Astra for an annotated dependency graph of the full-lane loop, separating the unavoidable T-vector pointer-chasing latency from unnecessary address calculations, state reloads, stores, branches and prefetch overhead.

One caution: 32 independent logical chains do not imply 32 simultaneously outstanding memory accesses. The CPU's reorder buffer, register allocation and scheduler may limit how much independence the code actually exposes.

That is precisely where specialized machine-code analysis can outperform general algorithmic reasoning.

I'd also have Astra assess whether the suggested eight-lane batching is likely to improve or reduce effective memory-level parallelism. It should not assume batching helps merely because it reduces branch-handling frequency.

## Two instructions I'd use

I approve the off-by-default integration of frozen P74-C on a new research branch, the single production-equivalent full-decoder experiment on my i9 and exploratory CI, and fetching the approximately 12 MB of new held-out Wikidata windows.

Before implementation or timing, please finalize and commit the preregistration.

Keep the frozen inclusive selector policy and the implementation from 8b6caf1, including the already-measured error-state optimization. Do not introduce additional code-generation tuning.

Measure 1, 2, 3 and 4 workers with the entire process constrained to the same four physical cores, leaving SMT siblings idle where possible. Four workers remains the primary acceptance configuration.

Keep the proposed 2% lower-bound four-core throughput gain, 3% negative-control regression limit, and +1% CPU-seconds/GB and peak-RSS limits. Define the RSS aggregation explicitly; report any sensitivity to the strict +1% CPU tolerance. Treat the three Wikidata corpora separately, without pooling them as independent datasets.

Preserve exact output, CRC and adversarial correctness, the same-binary A/B design, and the A/A calibration.

Do not enable P74-C by default or merge it into master. Stop after reporting the full-decoder results for review.

Conduct a read-only x86-64 machine-code critical-path analysis of the existing BWT room walker, specifically `src/bwt.rs::room_segments::<32, PF_L1>` and its full-lane inner loop.

Use the exact production revision, compiler flags, disassembly and relevant measured stage timings supplied by Claude.

Identify the dependency chains that limit useful simultaneous T-vector traversal, distinguishing unavoidable pointer-chasing latency from compiler-generated serialization, register spills, lane-state bookkeeping, address-generation dependencies, prefetch overhead and branch costs.

Assess whether scalar eight-lane batching would preserve the effective overlap of 32 chains. Do not recommend AVX2 gathers without compelling machine-code evidence.

Return at most one concrete high-value transformation with an expected disassembly signature, a conservative performance hypothesis and a falsification criterion. If no plausible mechanism supports approximately 1.5 cycles/output byte of walk savings, say so explicitly.

No implementation, benchmarks, CI or production changes. This is a focused diagnostic review, not a broad optimization campaign.

The first is the recommendation for Claude, not authorization until you send it. The second is intended for Astra once Claude has assembled the machine-code evidence.

My overall position is more optimistic about P74-C shipping than before. We've eliminated uncertainty about the Astra optimization's provenance, and the remaining experiment now measures the deployment configuration that actually matters.

And strategically, the researcher is right: the next substantial breakthrough is more likely to emerge from reducing the cost of the BWT walk than from further polishing an MTF optimization whose main benefit has already been demonstrated.
````
