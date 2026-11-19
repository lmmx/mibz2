# Astra E: the brief, as sent

The Codex session Astra E (6.2 Ultra) runs the moonshot campaign on
`research/astra-e-moonshot` (draft PR #607). This file holds everything it was told, verbatim
and in order: the base brief, then each supplement. To resume E in a fresh session, send the
base brief and both supplements as one message, then "continue from the handoff".

| # | Sent (UTC) | What |
|---|---|---|
| 1 | 2026-11-18 ~23:30 | The base brief (the scorer at v0, #606) |
| 2 | 2026-11-19 ~00:05 | Supplement: the formulation track and its ledger |
| 3 | 2026-11-19 ~00:25 | Adjustment: depth, not breadth, on the formulation track |

The scorer has since moved to v1 (#608, 604dac3f); the brief's refresh command picks that up. After message 3 the steering stopped. Lint, CI, merges and queue plumbing for Astra's branches
belong to the Coordinator, not to Astra (Louis, 2026-11-19), so no housekeeping requests go to
it either.

---

## 1. The base brief

### Astra E — Moonshot: a materially faster exact bzip2 decoder

I want to give you a sustained, autonomous research-and-engineering challenge: **discover, implement and verify a substantially faster way to reconstruct bzip2 blocks.**

This is not a bounded mathematical study. It is an iterative optimisation campaign with a measurable destination.

#### THE OBJECTIVE

**Stretch target: at least 1.25× the sustained throughput of mibz2 0.5.6 on real Wikidata blocks**, corresponding to a wall-time ratio of 0.80 or lower, at the same concurrency and no greater peak memory.

The target machine is a ten-core Intel i9-10900K running 7–10 workers.

This is an ambitious objective, not an assumption that such an improvement exists.

**Do not voluntarily conclude the investigation just because a candidate fails, a mathematical direction proves unhelpful, or an improvement falls short of 1.25×.**

A 5% improvement is progress. A 10% improvement is further progress. Preserve it, make it the new candidate to beat, and continue searching for the remaining gain.

The programme reaches its goal only when the improvement is independently verified under the target-machine performance and memory requirements—not merely when a local proxy predicts success.

#### THE KNOWN BASELINE

Read the repository's measured i9 evidence, rather than assuming operation count, DRAM traffic or container timing alone predicts performance.

- mibz2 0.5.6 uses LazyCells at 7–10 workers.
- The Open-Return kernel (`opend`, on `bench/open-return`) runs at approximately 0.93–0.95 of LazyCells' wall time across 2–10 workers, with lower peak memory.
- This is a measured kernel gain, not primarily a contention effect. MONODROMY's record 76 attributes it to approximately 24% fewer instructions per byte, partly offset by lower IPC and more lower-cache stalls.
- DRAM traffic does not independently explain that win.
- At 3–4 workers, eager Cells is substantially faster than LazyCells, and the production policy uses eager below eight threads.

The 1.25× objective is relative to 0.5.6. Beating the current best measured candidate, including opend, is an important intermediate milestone.

#### YOUR SCORER — USE IT CONTINUOUSLY

`bench/moonshot/` on master (PR #606) provides an executable scorer. Read its README first.

The scorer is live and will be recalibrated during your investigation.

**Before every scoring round, refresh it from master:**

```bash
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
```

Run this in the designated research worktree, without discarding unrelated experiment changes.

Record the scorer version using:

```bash
git log -1 --format=%h origin/master -- bench/moonshot
```

Also record the version printed by the scorer.

Compare candidates only under the same scorer version. When a scorer update arrives, re-score the current baseline and the best preserved candidate alongside the new candidate.

If an update breaks the scorer or candidate interface, preserve the last working scorer and continue using it until the incompatibility is resolved. Document the problem rather than abandoning a promising candidate.

The scorer reports:

- **Correctness — authoritative:** byte-exact output against libbzip2, including block and stream CRCs, periodic blocks and multiple LF cycles. Any mismatch is a failure.
- **Paired timing — authoritative for the local environment:** comparison against 0.5.6 at the worker counts available locally. This is not proof of i9 performance.
- **Advisory proxies:** cache-line transfers, dependent-miss chains, useful overlap, instruction count and other reported metrics, separately rather than combined.

Don't optimise a proxy for its own sake. Preserve disagreements between proxy predictions and measured execution.

#### THE OUTER VERIFICATION LOOP

1. Develop and test candidates locally using the scorer.
2. For promising experimental implementations, open a PR touching `src/`, a research feature or an example. The CI paired benchmark evaluates the appropriate source changes against master at 1, 2 and 4 threads.
3. For a candidate with a credible i9 advantage, request an independent workstation run through the Coordinator, with a frozen recipe, a preregistered decision rule and an untimed dry run.

Only the Coordinator queues i9 runs. Only CI and the target i9 establish production performance.

**Do not wait for an i9 run to continue mathematical exploration.** Preserve the candidate awaiting validation and work independently on the next hypothesis.

#### WORK BACKWARD FROM THE RESULT

Don't begin by choosing a favourite mathematical framework.

Begin with the decoder's exact requirements, its measured costs and the improvements necessary to reach the target.

You may change the representation of LF, the reconstruction order, the way output positions are recovered, the use of memory, the relationship between input processing and output production, or the unit of computation.

You may retain part of Cells while replacing another part.

The objective is a faster exact computation, not novelty for its own sake.

Read the repository's earlier mathematical investigations, independent reviews and production measurements, pinned to their source commits. These include Cells, adjacent-rank induction, Open-Return, forest sections, boundary-orbit profiles, copying schemes, grammar ownership, both writer schedules, endpoint sweeps and Astra studies A–D.

Their negative results are useful evidence, not universal impossibility theorems.

#### THE ITERATIVE CAMPAIGN — DO NOT STOP AT A PARTIAL RESULT

Maintain a continuing cycle:

**Hypothesise → falsify cheaply → prove or test correctness → prototype → score → preserve → compare → pursue the next improvement.**

Generate competing mathematical hypotheses. Use exact structural tests on the frozen block and representative Wikidata blocks to reject approaches whose construction, ordering or reconstruction costs erase their apparent advantage.

When a candidate survives, build an isolated experimental implementation and score it.

If it loses, diagnose the reason, record the negative result and change direction. Do not endlessly tune a fundamentally losing construction.

If it wins but remains below the stretch target, **promote it to a preserved milestone and continue**.

Look for opportunities to combine independently useful improvements, but do not assume their gains add. Recheck correctness, memory and performance on the combined implementation.

You are not required to follow a fixed sequence of ideas. You are required to maintain a continuously improving body of evidence.

#### PRESERVE EVERY MEANINGFUL ADVANCE

A result is valuable even if it doesn't achieve the final target.

Maintain a durable `research/astra-e-moonshot` branch, draft PR and research journal.

In addition, maintain a **best-so-far register** identifying:

- The strongest correct implementation measured locally.
- Its immutable commit and exact reproduction command.
- The baseline and scorer versions used.
- Wall-time ratios at each tested worker count.
- Peak memory, instruction and cache-proxy results.
- The exact inputs and correctness checks passed.
- Whether CI or the i9 independently verified it.
- Known weaknesses, regressions or unverified conditions.

Retain separate leading candidates when one is faster but another has a better memory or dependency profile. Do not discard a candidate merely because it loses on one advisory proxy.

**Commit after each consequential proof, counterexample, completed experiment or demonstrated gain.** Preserve programs and raw results, not only their summaries.

Before replacing an implementation with a new hypothesis, ensure the strongest previous implementation can be rebuilt and scored from its recorded commit.

Do not leave important discoveries solely in terminal output, reasoning traces or an unfinished final handoff.

A later experiment must never silently overwrite or erase an earlier verified gain.

#### RESOURCE ACCOUNTING

Account for the complete computation, including construction of intermediate objects, recognising applicable shortcuts, ordering dependencies, output reconstruction and memory use.

Distinguish:

- Instruction work and IPC.
- Dependent memory latency.
- Independent work available to overlap misses.
- Cache transfers and DRAM traffic.
- Working-set and peak-memory requirements.
- Serial stages, worker concurrency and output costs.

Extra arithmetic can be worthwhile if it reduces expensive dependency stalls. Fewer bytes moved can still be worse if the algorithm introduces a longer serial chain.

No abstract operation count or cache model is a substitute for measured performance.

#### CORRECTNESS AND IMPLEMENTATION BOUNDARIES

Preserve the exact bzip2 input contract, marked LF semantics, periodic and multiple-cycle cases, RLE1, block CRCs and combined stream CRCs.

Test independently against libbzip2 and use adversarial examples.

Do not modify the shipping decoder's default path, Pathfinder's branches or the release process. Do not change allocator behaviour.

Keep the experimental implementation isolated until it is independently reviewed.

#### CONTINUATION AND STOPPING CONDITIONS

**Do not stop because the first promising method fails. Do not stop because a partial gain is below the target. Do not stop merely because a mathematical investigation has produced an elegant theorem.**

A failed candidate is a reason to revise the search.

A successful partial candidate is a reason to preserve it and continue.

Continue the research-and-prototype loop while the session has execution capacity, useful hypotheses remain, and no correctness or tooling blocker prevents progress.

If time, context, tools or execution limits force the session to end before the target is reached, produce a **continuation handoff**, not a declaration that the campaign is finished.

That handoff must identify the best preserved implementation, the current scorer version, the hypotheses already tested, the strongest unresolved candidates, and the exact next experiment. Commit the handoff and its supporting artifacts.

If no credible hypotheses remain, document the explored space and why the available evidence provides no plausible next experiment. Do not invent performance claims or repeat failed approaches merely to keep working.

The campaign's ultimate success condition remains independently verified achievement of the throughput and memory objective.

#### DELIVERABLES

Keep the research record current throughout the investigation.

At each major milestone preserve:

1. The best correct candidate so far and its commit.
2. Its measured improvement and limitations.
3. Relevant proofs and exact structural evidence.
4. Failed approaches and why they failed.
5. The strongest next hypothesis.
6. The decisive independent benchmark needed.

At the end of any execution session, report the campaign's current state, whether the objective has been reached, and how to resume without repeating completed work.

**The governing question is: what would actually make exact marked BWT reconstruction substantially faster on the i9 that matters—and how far can you move toward that result through independently verifiable mathematical and engineering progress?**

---

## 2. Supplement: the formulation track

Supplement to the Astra E campaign — not a replacement.

Keep the engineering optimisation track, the scorer, and the best-so-far register exactly as they are. Add a second, substantial track dedicated to fundamental mathematical reinvention.

**At least every other research round must investigate a formulation-level hypothesis:** one that changes what is reconstructed, fundamentally changes the decomposition, or eliminates an entire computational phase, rather than merely accelerating an existing phase.

A research round means a distinct hypothesis pursued through its cheapest meaningful mathematical or computational falsifier. It does not necessarily require a full implementation or benchmark.

The ambition is a discovery on the conceptual scale of the original Rauzy–Veech insight behind Cells. No particular mathematical theory or construction is preferred. Explore freely rather than treating the existing decoder architecture as a constraint.

Maintain a durable **formulation ledger** alongside the performance register. For every formulation-level hypothesis, record what mathematical object it reconstructs, which computational phase it replaces or eliminates, its decisive invariant or proposed theorem, the cheapest exact falsifier, and what the evidence establishes. Preserve proofs, counterexamples and partial discoveries, even when they don’t produce an immediate speedup.

The anchor writer’s elimination of 81% of serial residual hops without a corresponding throughput win is a reason to question the need to construct the residual representation at all—not merely to accelerate its traversal further.

Continue pursuing the 1.25× throughput objective, preserving and combining engineering wins. Fundamental discoveries are also valuable independent research milestones.

One measurement rule: three-round local pilots are screening evidence, not decisive evidence. Increase repetitions when a conclusion depends on a close result, and use independent CI or target-machine measurements for proposed gains below approximately 5%. Treat scorer proxies as diagnostic, not authoritative rankings of close candidates.

Keep both tracks moving. Don’t terminate either because the other produces an interesting result.

---

## 3. Adjustment: depth, not breadth

Adjustment to the formulation track: its job is depth, not breadth.

When a formulation-level idea survives its first falsifier, as the delimiter construction just did, stay with it. Develop its theory, structural invariants, generalisations and limitations; derive upper and lower bounds on construction cost where possible; and investigate whether it supports an exact, computationally useful reconstruction algorithm.

Give promising ideas sustained attention across as many research steps as their mathematical development warrants. The every-other-round allocation remains, but a single sustained formulation programme can occupy the mathematical track throughout that period.

Don’t abandon a surviving mathematical direction because its first construction is expensive or its next test is ambiguous. Distinguish an inherent obstruction from a limitation of the current construction.

Equally, don’t force a proof or continue indefinitely without mathematical progress. When an approach genuinely reaches diminishing returns, preserve its theorems, counterexamples, unresolved questions and potential future directions before moving on.

Continue the engineering track and its performance register. Close performance comparisons belong in sufficiently powered paired measurements, independent CI or the i9—not in speculative interpretations of noisy pilots.
