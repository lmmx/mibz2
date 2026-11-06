# External research report 174: the guardian's three amendments to the compact30 traversal protocol, and approval of harness preparation (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the draft protocol at PR #134, `38b9a81`
  (`docs/journal/2026-11-02-compact30-traversal-protocol.md`).

**Evidence cutoff:** 2 November 2026. A reading of the draft protocol.

**This authorises:**
- revising the protocol;
- implementing the minimum measurement harness and recipe;
- verifying them by correctness checks, counter-free measurement-path validation and an
  untimed dry run.

**It does not authorise:** reporting VM performance results, any walker change, a
constructor, an alternative layout, a tuning arm, a production change, or the i9 capture.
The capture waits for approval of the frozen command.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
We are at the final experimental-design checkpoint before compact30's first performance test on your i9.

Claude has completed the two repairs we requested. Its report says the actual walker remains unchanged, the correct ordinary-step count is 21 instructions, and the uncounted implementation now passes cold-start, repeated-use and marker-restoration tests.

That's good progress. The next question is no longer whether compact30 is correct or compiles into a reasonably simple loop. It's:

Does its smaller navigation table make the complete traversal sufficiently faster under four-core contention to justify developing a real constructor?

I checked Claude's draft protocol in PR #134 at `38b9a81`. The comparison is well structured, but I would amend three points before freezing it. None requires changing the walker or conducting more architectural research.

## Three protocol changes I'd request

### 1. Fix the stopping rule's assumption about construction

Claude proposes stopping compact30 whenever the lower confidence bound of its four-core traversal-cost ratio is above `0.865`.

The reasoning is that such a result would require a constructor faster than production's.

That is not enough to establish impossibility. Production has an efficient constructor, but Astra has already identified a plausible direct-construction strategy that might have different costs. We cannot assume a compact constructor must be slower without measuring or bounding it.

For example, the planning model gives:

| Traversal cost ratio | Maximum constructor cost ratio to save 0.25 CPU-s/GB |
| -------------------- | ---------------------------------------------------- |
| 0.80                 | 1.20                                                 |
| 0.865                | 1.00                                                 |
| 0.90                 | 0.89                                                 |
| 1.00                 | 0.58                                                 |

A result of `0.90` would not meet the current advance criterion, but it would not establish that further gains are mathematically impossible.

Keep 0.865 as the minimum-investment screen. If the result misses it, the default should be STOP under the project's chosen investment policy—not the stronger assertion that no credible constructor could compensate. That preserves the useful discipline without turning an engineering judgment into a proof.

### 2. Report completed traversal throughput as well as worker CPU cost

The draft defines its four-core result using:

\\[ \frac{\text{sum of worker traversal times}}{\text{sum of rows}}. \\]

That's a useful approximation of concurrent traversal worker cost per row.

But it is not the elapsed time to finish the workload with four cores.

Our objective requires both throughput and CPU efficiency. The harness should therefore record the barrier-to-last-completion wall time for each arm, as well as aggregate worker time, and report both candidate/control ratios.

I would keep the CPU-cost ratio as the construction-budget input, since the planning model is denominated in CPU-s/GB. But the advancement decision should also check that completed-work throughput is not materially worse. Predeclare that check rather than deciding after seeing the numbers.

### 3. Make the four-worker workload and post-construction boundary explicit

Claude proposes four workers, each receiving a quarter of the corpus. That's fine in principle, but the quartering needs to be balanced by BWT rows and representative block mix, not simply by block count. Otherwise the four-core elapsed time can be influenced by avoidable workload imbalance.

There is also an important cache-state caveat.

The production arm builds its actual T-vector just before traversal. Compact30 copies oracle-built records into its arena just before traversal. Those operations prepare different cache states even though both happen outside the timer.

The draft acknowledges this, which is good. I would make it an explicit interpretation limitation:

> This compares the measured traversal after each arm's specified table preparation. It does not isolate the table representation under identical cache state, nor reproduce the cache state of a future direct compact30 constructor.

We do not need to build a constructor to fix that limitation. We need to ensure the protocol doesn't claim more than it measures.

## What happens next

1. Claude revises the protocol.

   No walker changes, new variants or additional correctness work are needed unless the timing harness introduces a problem.
2. Claude implements and dry-runs the measurement harness on its development machine.

   The purpose is to verify arm equivalence, correct workload allocation, synchronization, recording, validity gates and artifact packaging—not obtain performance results.
3. We review and freeze one source pin and one command.

   No changing the experiment after seeing its performance numbers.
4. You run the approved command on the i9 and upload its archive.

   We independently check validity before interpreting performance.
5. The registered decision determines the next investment.

   A sufficiently strong result earns direct-constructor feasibility work. A weak result stops compact30 under the agreed investment policy. An inconclusive result triggers review, not an automatic sweep.

## Send Claude this

I approve proceeding with compact30's measurement-harness preparation, subject to three amendments to the draft protocol at commit `38b9a81`.

**1. Correct the interpretation of the 0.865 stopping threshold.**

Keep 0.865 as the registered minimum-investment screen derived from the 0.25 CPU-s/GB planning target and unchanged-constructor assumption.

However, do not claim that a faster compact30 constructor is impossible or inherently incredible. The direct-constructor proposal has not been measured. Missing this threshold may justify STOP as an engineering investment decision, but it is not a proof of architectural impossibility.

Preserve a separately reported constructor budget across the full observed confidence interval.

**2. Include both four-core worker cost and completed-work throughput.**

The summed worker-time-per-row ratio is appropriate for the CPU-cost planning model, but it is not the four-core wall-time throughput ratio.

Record both aggregate worker traversal time and barrier-to-last-completion elapsed time, with paired candidate/control ratios and intervals.

Retain the worker-cost ratio for the construction-budget screen. Add a predeclared no-harm requirement for completed-work throughput, with a justified tolerance and handling of inconclusive intervals. We should not advance a candidate whose CPU-cost ratio looks attractive but whose four-core completion performance is materially worse.

**3. Specify allocation and cache-state limitations.**

Balance the four-worker workload by total BWT rows and representative block mix, with a deterministic recorded assignment. Include the actual assignments, per-worker rows, work times and overlap intervals in the archive.

The control builds its T-vector before timing; the candidate copies an oracle-built compact table before timing. These prepare potentially different cache states. Preserve that asymmetry in the protocol and explicitly limit the interpretation. Do not claim this captures the cache state or construction economics of a future direct constructor.

After revising the protocol, implement the minimum measurement harness and recipe, then verify it using correctness checks, counter-free measurement-path validation and untimed dry-run mode. Do not report VM performance results.

Keep the compact30 walker unchanged at its reviewed 21-instruction ordinary path. No constructor, alternate layout, extra tuning arm or production modification.

Return the revised protocol, validity checks, source identity and one proposed i9 capture command for approval. Do not run the actual performance capture until that approval.

## Where this leaves us

We're close to the first decisive result for the improved compact representation.

The important progress is that we no longer need to speculate about whether compact30 can produce the intended machine code. Claude has demonstrated that it can, with five more ordinary-step instructions than production instead of FOR-14's much larger gap.

What we don't know is whether the memory-system advantage outweighs those extra instructions and exceptional-path costs.

That's exactly what the next i9 experiment should answer.

Nothing for you to run locally yet. I would send the protocol corrections to Claude, let it finish the harness, and review the final frozen experiment before authorizing your workstation.
`````
