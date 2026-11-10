# Width intervention: retain the factorial contrast, revise its inference

Request [6761932309](https://github.com/lmmx/mibz2/pull/162#issuecomment-6761932309). Reviewed second draft docs/journal/2026-11-10-width-intervention-draft.md at **efe945362e728b31886691603385f56461bcce63**; role/rules at **a2da49638ac54d7763de1c167bd054f67ffdc763**.

**REVISE before registration.** Retain the eight-cell, 20-retained-round diagnostic. Its log contrasts are useful total layout effects. Do not promote their uncertainty bands into mechanism identification or a universal rejection of compact tables. The second draft has already withdrawn the latter claim; the dispatch's quoted kill band is stale.

## 1. Estimand: fix the aggregation as well as the formula

For retained round r, from marker-off whole-process CPU:

    e4_r  = log CPU(B4,r)  - log CPU(A4,r)
    e10_r = log CPU(B10,r) - log CPU(A10,r)
    i_r   = e10_r - e4_r

Use the arithmetic means of these round log contrasts as E4, E10, I; exponentiate for geometric mean ratios. A log ratio of pooled arithmetic CPU means is a different estimand. Freeze the choice. Keep wall secondary, without using it to select or rescue a CPU result. Report raw CPU and wall by cell as well as contrasts.

TLB pressure, faults, extra line traffic, allocator reuse and changed occupancy are **parts of the randomized layout treatment**, not confounders of its total effect. They confound an interpretation that names shared-cache capacity as the sole mechanism. Round pairing cannot subtract them away. Untreated order/time/load changes can confound even the total effect; randomize the within-round cell schedule prospectively, retain every slot and record order.

The sentence "a side effect common to B4 and B10 cancels in I" needs its scale stated. A common multiplicative factor on **total process CPU** cancels. An equal additive cost does not. Using the draft's A4=.6825 and A10=.7929, add .5 CPU-seconds to both B cells:

    exp(I) = ((.7929+.5)/(.6825+.5))/(.7929/.6825) = .94113.

Thus exp(I) below one can coexist with a large positive width cost at both worker counts. A common factor on worker CPU alone also need not cancel when scanner/main fractions differ. The earlier Optimiser already raised this; the second draft's sentence still overstates cancellation. Preserve E4/E10 regardless of I.

## 2. Predictions: conditional scenarios, not identified rival families

Substitution into total CPU approximately reconstructs the scenario factors: **1.416, 2.855 and 4.414**, using the draft's rounded scanner/main and worker CPU numbers. That checks the displayed arithmetic; it does not independently validate the old X, f=.78, archive values or transfer to this intervention.

F0="no shared-cache table term" does not predict I=0 if layout-induced traffic/TLB/fault costs grow with W. Conversely I=0 does not imply no table cost. Rename the 1.0 scenario **no total layout×worker interaction**, or state the extra zero-interaction nuisance assumptions.

F2's 1.4–2.8 is a pair of occupancy scenarios, **not a proved prediction interval**. Width changes occupancy downstream; 5.92 and the -9 value 8.1 do not bound the 16-byte -2 arm. Different feed/build balance can fall outside them. Retain both scenarios with their assumptions; do not reject the entire occupancy family solely for missing this interval. Likewise do not plug observed marker-on occupancy into the old curve after the fact and call that a prospectively validated prediction.

### Replace band labels with set comparisons

Let C=[L,U] be the reported interval for exp(I), with its assumptions explicitly attached.

- U<1.15 supports "interaction smaller than 15%" **only under that interval's coverage assumptions**. It does not imply proximity to one in both directions; for "negligible interaction" require C inside [1/1.15,1.15], prospectively.
- L>1.15 supports an interaction exceeding the practical threshold under the same assumptions.
- Otherwise the 15% question is unresolved.
- For a fixed scenario p, p outside C excludes that numeric prediction at the stated inferential level; p inside C means not excluded, never verified.
- For the proposed F2 scenario range, C disjoint from [1.4,2.8] contradicts those specific scenario values. Overlap establishes compatibility only. C wholly inside it still does not identify occupancy.
- **An interval wholly between 2.8 and 4.4 excludes the literal point 4.4 if U<4.4.** The draft instead says "literal law not excluded": reverse that. If the literal model needs a tolerance or parameter uncertainty band, specify it before data.
- Assign exact boundary equality explicitly; do not round an interval across a boundary.

Keep the whole interval, not just one categorical label. The E4>1.5 warning is not a causal separator: smaller side effects can scale strongly with W, and larger E4 can be the layout's intended memory effect. Report it as a practical cost annotation; do not condition statistical coverage or select the analysis on passing it.

## 3. Twenty rounds: useful diagnostic, not automatic calibrated killing

The draft's .11–.15 SD range for i_r is not established by the archived -2/-9 SDs. If two growth contrasts both had SD .105, the SD of their difference could range from **0 to .210** depending on their covariance. The new wide arm need not have .105 at all. The archived paired difference of different streams is not the covariance of new width arms. Label .11 and .15 as sensitivity scenarios, not bounds.

Under iid rounds those scenarios give SE .0246/.0335; t19 half-widths approximately .0515/.0702, before serial dependence. Pairing removes a common round shock only insofar as it really is common to all four cells.

A reproducible analytic counterexample: stationary Gaussian round contrasts with marginal variance sigma² and correlation rho^h have

    Var(mean i) = sigma²/20 * D
    D = 1 + 2 sum[h=1..19] (1-h/20) rho^h.

For rho=.5, D=2.8000 (effective n=7.14); for rho=.8, D=7.0231 (effective n=2.85). Even an oracle interval using the TRUE marginal sigma but incorrectly dividing by sqrt(20), with nominal normal 95% limits, covers only **75.85% / 54.04%**, respectively. This is an exact model counterexample, not a claim about the actual machine. Replacing the oracle variance with a sample estimate does not establish coverage. One experiment's apparent lack of autocorrelation cannot prove independence for the new experiment.

### Proposed bounded uncertainty plan

Keep **20 retained rounds, no precision-driven extension or retry**. One predetermined warm-up round remains excluded by design. Freeze schedule, timeout/invalidity, hashes and exclusions before running; no sentinel-based deletion or outcome-dependent block length. If a required cell is missing or invalid, retain it and apply the predeclared attempt-level invalidity rule rather than silently forming a selected set of complete rounds.

For a **descriptive diagnostic**, publish all chronological i_r/e4_r/e10_r, cell values, means and a predeclared dependence sensitivity:
1. Group the 20 consecutive rounds into five nonoverlapping groups of four. For each contrast compute the five group means, their sample SD s_g, and mean ± t(.975,4)*s_g/sqrt(5), t=2.776.
2. Label this a **conditional batch-t interval**: exact only for independent Gaussian group means with common mean and variance. Four rounds per group do not prove those assumptions; five groups provide little ability to diagnose them. The calculation is not a certificate against long drift.
3. Also report the ordinary round-t interval and the chronological plot as sensitivity diagnostics, never choose whichever produces a preferred band. Block bootstrap with 20 rounds and length four has roughly five blocks of information; thousands of resamples do not create more independent evidence. Do not present its percentile limits as assumption-free 95% coverage.
4. Treat only I as primary if an inferential threshold claim is intended. E4/E10 and marker contrasts are secondary descriptive intervals. If simultaneous exclusion claims are intended across them, preregister multiplicity control as well.

This plan makes 20 rounds informative **without pretending dependence has been resolved**. Under unverified dependence assumptions, phrase band conclusions as conditional/descriptive, not calibrated rejection of F1/F2. If calibrated exclusion is mandatory, freeze an independently justified dependence model or a valid randomization-based analysis and its assumptions first. Simple within-round label permutation is valid for a sharp no-effect null only under its actual random assignment and no interfering carryover; it does not automatically test a composite 15% threshold. Do not add unbounded calibration work to this diagnostic: weaken its inferential claim instead. No finite sample size alone protects against arbitrary persistent dependence.

## 4. Marker: always descriptive, with a defined clock

The marker-on traces remain descriptive **even if marker-on/off interaction estimates agree**. Non-significance of their difference does not prove absence of marker perturbation. If a trace-usability threshold is desired, predeclare a practical equivalence margin for I_on-I_off and require its entire conditional interval inside that margin; otherwise report its value and uncertainty with no pass/fail adjective. Marker effects on E4/E10 can cancel in I, so inspect all three.

Specify clocks. Simultaneous envelope counts require a **common monotonic wall-time domain**. Thread CPU timestamps alone cannot locate overlaps between workers; they stop while threads are not executing. If "a worker's clock" means thread CPU time, add monotonic timestamps for overlap and retain CPU deltas separately. Build/walk envelopes overlap while workers may be descheduled; call this active envelopes, not workers executing simultaneously, without independent scheduler evidence. These envelopes measure neither table lifetime nor cache residency. Preallocation and no shared written line reduce overhead; they do not prove it is absent.

Keep marker-off results primary, no regression adjustment for observed overlap (a treatment mediator and duration-defined quantity), no conversion of marker-on correlations into causal capacity evidence.

## Scope, evidence and next move

Expected: a width contrast discriminates the proposed scaling stories. Observed in review: the second draft fixes broad kill claims and many cost descriptions; log estimands remain useful, but its common-cost cancellation, literal-law band and precision justification still fail. No measured performance gap has closed, no cache mechanism established.

Next value: implement the already-scoped semantics/count/safety checks through the Coordinator/Optimiser, then collect one frozen diagnostic after completing §9. Freeze the aggregation, randomized schedule, missing-cell rule, conditional/descriptive uncertainty, corrected prediction labels and clock semantics. This is **REVISE the registration**, not a rejection of the intervention or of compact tables. Guardian safety/admissibility remains separate.

Evidence: pinned second draft; original proposal 6760353340; Optimiser REVIEW 6760521690; fresh channel/inbox timelines. The archive was not downloaded or rehashed here, and no native decoder, kernel build or timing was run. Arithmetic independently reproduced in check.py/results.json. No implementation or authority change. Migration checks passed and routing audit found no new misroutes.
