# External research report 26: research burn-down round 34, #75 from #74's lane data (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 25 (`2026-10-30-external-report-25.md`).

**What it contains:** prevalence against bunching.
- **The model:** with 32 independent lanes and a per-lane exception rate p,
  P(X ≥ 1) = 1 − (1 − p)³².
  - About 87.7% of full rounds for breakpoint rows (p = 0.0634).
  - About 99.97% for compact-Ψ slow rows (p = 0.225).
- **The consequence:** any round-wide fallback would pay on almost every round. Handling
  each lane independently costs 7.2 slow operations a full round on average.
- **It is recorded under the existing #75,** as probability-model results, not measured
  stalls.

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §9, row #75.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols and a figure caption
flattened) and is kept as received.

## Round 34, as received

````text
Burn-down round 34: the hidden implication of #74’s negative result

I found a consequence of Claude’s lane-coincidence measurements that changes how we should evaluate compact Ψ.

The exceptional lookups do not bunch together, but they are so common that almost every 32-lane round contains at least one.

That’s not a contradiction. It’s the difference between correlation and prevalence.

1. The numbers

Claude measured two exceptional-lookup populations under mibz2’s shipped 32-lane scheduler:

Population	Mean exceptional lanes per full round	Per-lane rate
Translation breakpoint rows	2.03	6.34%
Compact-Ψ slow rows	7.2	22.5%

The observed cross-lane variance matches independent lanes closely. Under that independent-lane model, the probability of at least one exception in a full round is

P(X\ge1)=1-(1-p)^{32}.

For breakpoint rows, that is approximately 87.6%.

For the compact-Ψ slow path, it is approximately 99.97%.

Modeled probability that a 32-lane round encounters an exception

Breakpoint case

87.6%

Compact Ψ

99.97%

Independent-lane predictions for full 32-lane rounds, not measured CPU stall probabilities.

The distinction matters for the original #75 question: which structural properties predict expensive dependent loads under 32 lanes?

The average exception rate already predicts that nearly every round requires some slow-path handling. The absence of excess cross-lane correlation only tells us there are no unexpectedly large simultaneous bursts.

2. This rules out a misleading optimization argument

Suppose we try to optimize a 32-lane decoder by providing a fast operation for pure translation groups and a slower operation for exceptional groups.

It would be wrong to argue:

Slow cases don’t cluster across lanes, so the fast path dominates execution.

For compact Ψ, around seven lanes per full round require the slow operation.

Whether that matters depends on the execution model:

* Per-lane independent operations: total extra work scales approximately with the number of slow lanes.
* Round-wide fallback: nearly every round takes the fallback, destroying much of the fast path’s advantage.
* Divergent scalar branches: unpredictable dispatch and uneven dependency completion may increase instruction cost even when exceptions are independent.

Thus there are two distinct structural questions:

\text{How many exceptional operations?}

and

\text{How are those operations scheduled?}

The first is now measured. The second depends on the actual lookup implementation.

3. A sharper bound for designs with round-wide fallback

Suppose one synchronized 32-lane iteration incurs an additional cost whenever any lane encounters an exception.

The expected penalty under the independent-lane model is

E\\text{penalty}

F1-(1-p)^{32}.

For the compact-Ψ case:

E\\text{penalty}\approx0.9997F.

So a round-wide fallback would pay essentially its full penalty every iteration.

By contrast, if each exception costs and is handled independently, the expected additional work is

E\\text{penalty}=32pf=7.2f.

These formulas describe different implementation strategies, not two estimates of the same CPU time.

The design lesson is concrete: a compact Ψ representation with a 22.5% exceptional-lane rate should avoid any mechanism that makes all 32 lanes pay because one lane is exceptional.

4. How this changes the status of #75

The existing ledger said that exceptional bursts along trajectories were likely to be the relevant structural predictor.

That was incomplete.

The results now separate three possible predictors:

Predictor	Evidence	Significance
Per-lane exception prevalence	Measured	Determines expected exceptional work
Cross-lane bunching	Measured negative	No additional synchronization concentration
Within-lane burst lengths	Measured clustering	Matters for repeated dependencies and cache locality

For a round-wide fallback, prevalence is overwhelmingly decisive: practically every full round encounters one.

For independently processed lanes, within-lane bursts may matter more, especially if successive exceptional records access related cache lines.

We should therefore not infer that the lack of bunching makes compact Ψ cheap. It merely removes one potential source of additional cost.

Round 34 conclusion

This gives us a stronger disposition for #75 — Which structural properties predict expensive dependent loads under 32 lanes?

We now know:

* The independent-lane exception model explains the measured simultaneous counts.
* Compact Ψ’s 22.5% slow-path rate implies almost universal presence of exceptions across full 32-lane rounds.
* A round-wide fallback is therefore structurally unattractive.
* Independently handled exceptions still require CPU-cost attribution; their cycles cannot be inferred from the distribution alone.

This is a new, quantitative constraint on compact-Ψ lookup designs, derived entirely from measurements already obtained.

Relay to Claude

Research round 34 — #75 from #74’s measured data

The absence of cross-lane bunching does not imply exceptional work is rare at the round level.

For 32 independent active lanes with exception probability p, P(at least one exception)=1−(1−p)^32.

Using the measured shipped-scheduler averages:

* Breakpoint rows: p=2.03/32≈0.0634, giving approximately 87.7% of full rounds with an exception.
* Compact-Ψ slow rows: p=7.2/32=0.225, giving approximately 99.97% of full rounds with an exception.

Therefore any round-wide fallback that charges all lanes when one lane is slow would pay almost every round for compact Ψ. The absence of excess variance only rules out additional cross-lane concentration.

For independent per-lane handling, mean exceptional work is 7.2 operations per full 32-lane round. Within-lane burst structure remains relevant to dependency and cache costs.

Please record this under existing #75. These are independent-lane probability-model results, not measured CPU stalls. Do not interpret the 99.97% figure as the percentage of cycles stalled or imply the shipped decoder currently implements a round-wide fallback.

This is a substantive additional burn-down result: #74’s negative finding has now yielded a quantitative constraint on #75, without a new benchmark or another speculative algorithm.

We can continue toward the remaining CPU-layout questions while Claude finishes the code-generation and literature checks.
````
