# External research report 125: the guardian's P87 decision and instruction to the coordinator, with the independent audit's final clarification (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 16:40 UTC.
- **Source:** the guardian, relaying the independent audit's final clarification at its end.
- **In reply to:** the coordinator's inverse-BWT proposal (`docs/bwt-memory-proposal`), Astra's P87 proposal, and the researcher's audit of both.

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment: a proposal and its review, not a performance result.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
I agree with the researcher. We now have enough independent analysis to select P87 as our next experiment. I wouldn't commission another round of architectural opinions before moving forward.

Astra identified an actionable intervention, Claude independently narrowed the measured performance penalty, and the researcher identified the weaknesses in both proposals. This is precisely the kind of convergence we wanted.

## The decision

Advance P87 to frozen protocol and implementation preparation

Reuse the dead `ll` buffer for inverse-BWT segment assembly, keeping the four-byte T-vector and traversal unchanged.

Preserve Claude's neighbour-interference proposal

Do not build the full synthetic workload matrix yet. Correct its calibration assumptions before any future authorization.

No i9 benchmark yet

First prepare and audit the implementation, measurement controls and protocol.

### The crucial scientific distinction

Claude's strongest new result is that the walk-plus-assembly phase accounts for approximately 0.572 CPU-s/GB of the concurrency penalty, while the cost of that phase rises with the number of concurrent walkers.

However, the claim of six independently measured outstanding misses and the exact 98% software-prefetch share is not established. Those should be corrected in the research record.

P87 gives us an opportunity to test whether a change outside the traversal algorithm makes that expensive phase cheaper in the real decoder.

We must require two separate results:

| Gate                | Question                                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Causal mechanism    | Is the concurrency-specific saving \\(D\\) materially positive, with a 0.10 CPU-s/GB threshold?                     |
| Engineering benefit | Does the modified decoder actually improve four-worker CPU-s/GB and throughput, with correctness and RSS preserved? |

A positive \\(D\\) alone does not justify shipping the modification.

## Complete next instruction for Claude

We've now compared your independent inverse-BWT proposal with Astra's P87 buffer-reuse proposal, and the independent researcher has audited both.

The decision is to **advance P87 as the first controlled intervention**, while retaining your neighbour-interference harness as a valuable later research option.

This is not a rejection of your work. Your read-only phase analysis materially improved our understanding of the problem and will inform P87's measurements.

## 1. Preserve and qualify the new architectural findings

Keep your measured result that the recorded walk-plus-assembly phase accounts for approximately 0.572 CPU-s/GB of the one-to-four-worker CPU penalty, with 1.234 → 1.806 ns/row.

Preserve the observed relationship between the number of concurrent walks and walk-phase cost.

However, explicitly qualify three points:

- The archived `WalkStart` → `InvertEnd` interval includes segment assembly. It must not be labelled as pure pointer-chasing time.
- The walk-overlap relationship is observational and susceptible to duration/overlap selection effects and reverse causation.
- Approximately six walker-specific outstanding requests is an inferred model, not an independently measured MLP value. The 39→58-cycle metric combines request classes and cannot be treated as direct demand-load service latency. Likewise, an exact 98% software-prefetch share is not supported by the heterogeneous counter populations.

Preserve the useful hypothesis that slower memory service or reduced effective overlap explains much of the walk penalty, without promoting an unverified arithmetic model into a causal finding.

Your neighbour-interference design should be retained as a research lead. Its proposed 70% calibration rule must not conclude that memory contention is disproved when synthetic neighbours fail to reproduce production. The correct interpretation is that the harness is not yet representative.

Do not implement that harness now.

## 2. Prepare a distinct P87 four-core diagnostic

Astra proposed reusing the now-dead MTF last-column (`ll`) storage as the destination for inverse-BWT segment assembly, rather than writing assembly into a separate `rle` allocation.

This is a diagnostic of P87's buffer-lifetime mechanism, not a copy-elimination experiment.

The two arms must preserve:

- The shipped four-byte T-vector representation.
- The existing T-vector construction and 32-lane traversal.
- The existing software-prefetch policy.
- The existing segment buffers and assembly copy.
- The RLE1 and CRC semantics.
- Production dispatch, admission and ordered-output behaviour.

Arm A uses the existing `rle` assembly destination. Arm B reuses `ll` after its last read by T-vector construction.

End all outstanding immutable borrows before reusing `ll` as a mutable assembly destination. Preserve small-block fallback, buffer-capacity behaviour, error paths and correctness.

## 3. Control what the intervention changes

For the causal diagnostic, equalize allocation, capacities and initial page residency between A and B. Retain the unused arena in B throughout the measurement so that allocation or RSS differences cannot masquerade as a locality benefit.

Balance arena placement across paired trials where feasible.

Audit the generated machine code for construction and traversal. They must be instruction-equivalent between arms, not merely logically equivalent Rust implementations.

Check logical assembly-copy volume and RLE1 work separately. If changing the interface causes unavoidable code-generation differences, report them as a confounder rather than concealing them.

Remember the temporal direction: a worker assembles *after* its own traversal. Any reduction in traversal cost must arise from persistent or cross-worker effects, not a retrospective local benefit.

Do not assume that reusing `ll` necessarily reduces memory-controller writes. Both scratch allocations persist across blocks and have different cache histories.

## 4. Register two separate decisions

Use one-worker and four-worker A/B full-decoder comparisons, with the same four physical-core budget and existing thread-placement policy.

Let C(X,w) be CPU-s/GB for arm X with w workers, and define:

D = [C(A,4) - C(B,4)] - [C(A,1) - C(B,1)].

**Mechanism gate:** Astra's proposed materiality threshold is D ≥ 0.10 CPU-s/GB. A lower confidence bound above the threshold supports a material concurrency-specific benefit; an upper bound below it rejects that materiality claim; an interval crossing it is inconclusive.

**Engineering gate:** Separately establish that B actually improves four-worker CPU-s/GB and full-decoder throughput, with no unacceptable correctness or RSS regression.

A positive D with slower four-worker decoding must not promote B to production engineering.

Conversely, a small useful absolute improvement may exist even if the concurrency-specific mechanism fails; report it separately without misrepresenting the causal finding.

Prespecify the exact confidence method and practical full-decoder acceptance criteria before any timed run, using our established paired A/B and A/A methodology.

**Historical-registration safeguard:** P87 is already recorded as open with a distinct ≥3% occupancy-curve criterion at 6–10 workers and DRAM-write measurements. Preserve that historical criterion verbatim. Register this one/four-worker investigation as a separate diagnostic of P87 rather than silently replacing or closing the original proposal.

## 5. Stage the measurement

Prioritize the minimum experiment that can establish whether the intervention helps:

- Paired uninstrumented A/B and A/A full-decoder measurements at one and four workers.
- Correctness and CRC verification.
- Separate construction, traversal, assembly and post-processing phase measurements, with perturbation checked independently.
- DRAM read/write measurements over equivalent complete work boundaries.
- Machine-code equivalence checks.
- CPU-frequency, worker-placement and measurement-quality controls.
- Clear distinction between BWT-row bytes and final decoded bytes.

Probe hardware-event availability before registration. Do not make the primary decision depend on optional PEBS load-latency or address-resolved sampling.

Detailed ownership-request, fill-buffer and T-vector-specific sampling should be conditional follow-up diagnostics if their results could distinguish between explanations of an observed effect. Avoid an unnecessarily large initial PMU campaign.

## 6. Protocol and implementation boundary

Please now:

1. Inspect the exact segmented-inversion interface and establish the smallest safe A/B implementation.
2. Draft and freeze the causal experiment's hypotheses, baselines, acceptance criteria, confounders, corpus, trial counts and analysis.
3. Implement the isolated diagnostic A/B path and required minimal instrumentation, without changing the production default or optimizing anything else.
4. Add focused correctness tests, code-equivalence checks and validation of the analysis rules, including a case where D is positive but B is slower at four workers.
5. Perform an untimed build/correctness verification and an untimed runner dry run, as appropriate.
6. Return the implementation pin, freeze record, expected i9 command, instrumentation and runtime budget, plus any unresolved validity concerns.

**Do not launch the real i9 timed capture. Do not modify or rerun closed experiments. Stop when the frozen P87 diagnostic is ready for review and authorization.**

The purpose is to test one realistic memory-lifetime intervention against the decoder we actually ship, not to claim that reduced writes or lower cache misses alone prove a particular physical bottleneck.

One practical note: I'd let Claude identify any unavoidable trade-off between exact code equivalence and safe Rust borrowing before freezing the diagnostic. There is no benefit in demanding identical machine code if meeting that requirement accidentally introduces a different buffer lifetime or control-flow cost.

My recommendation is to send this now. Claude can build and validate the P87 experiment without reserving the i9, and we can scrutinize the frozen protocol before its single registered capture.

We have a concrete next step, a meaningful success criterion, and a clear boundary between experimental evidence and shipping an optimization.

One final clarification from the independent audit before freezing P87:

The original attribution archive's 0.572 CPU-s/GB walk-plus-assembly scaling increment applies to full-sized 900,000-row blocks. Across all 434 blocks, normalized by actual decoded bytes, the corresponding whole-corpus increment is approximately 0.609 CPU-s/GB. Please verify and preserve both denominators, and use the whole-corpus figure for decoder-level opportunity accounting.

Treat the walk-overlap bins as correlated observations of repeated blocks, not independent experimental replicates. The 0-walker and 3-walker extremes have relatively few observations, and longer walks can create more overlap.

For P87, construct confidence intervals from independent paired runs, not individual blocks. Equalize allocation and initial page residency without repeatedly warming or dirtying the unused arena during measured work.

The registered four-core P87 diagnostic must retain both separate decisions: material concurrency-specific D ≥ 0.10 CPU-s/GB, and an actual four-worker CPU-s/GB plus throughput improvement. Neither substitutes for the other.

These are clarifications to the existing P87 preparation instruction, not authorization for a timed i9 capture. Please continue the isolated diagnostic implementation and untimed verification, then return the freeze for review.
`````
