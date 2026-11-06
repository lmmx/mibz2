# Three-team policy: science review before freeze
Assignment [6694153375](https://github.com/lmmx/mibz2/pull/162#issuecomment-6694153375).
Candidate reviewed: **40c2578095cbfe67a30a9ce2c10d0034b66e859c**, PR #234.
Disposition: **HOLD to freeze**, limited to validity/provenance defects below. The default-versus-default question and interval algebra are sound. No timing, implementation, merge or authority change.

## Evidence
At the candidate pin, read [registration](https://github.com/lmmx/mibz2/blob/40c2578095cbfe67a30a9ce2c10d0034b66e859c/docs/journal/2026-11-06-three-team-policy.md), [judge](https://github.com/lmmx/mibz2/blob/40c2578095cbfe67a30a9ce2c10d0034b66e859c/bench/policy_ab.py), its synthetic tests, .just/bench.just, examples/ab.rs, src/parallel.rs and decisions.md. Read #220's result/confirmation at 544462881497fc957fe17c35ae7508ca1bac23d5. The recorded Louis decision is scoped to this policy, not a general change to CI admission. This review assesses the proposed experiment against that recorded scope; it grants no authority.

Prior evidence supports an approximately 8% prediction, not a causal cache claim. #220's journal is evidence of reported results; no independent replay of its raw bundles was performed here.

## Required before freeze

**H1 — fail closed on missing output evidence and inconsistent arm identity.** I ran the unmodified judge on its existing synthetic passing grid, then changed only the indicated fields:

| Input | valid | pass |
|---|---|---|
| Existing clean grid | true | true |
| Remove hash and out from every row | true | true |
| Alternate the base executable identifier on odd rounds | true | true |
| Set every output length to zero | true | true |

The first defect follows directly from outputs = {(r.get("hash"), r.get("out"))}: one repeated (None, None) is accepted as one output. The second follows from build_of collapsing every mibz2@... to “base”. Those observations do not accuse the collector of producing bad data; they show that the admission checker cannot establish the conditions it claims to check.

Minimum repair: require all needed fields and valid types/ranges, including nonempty hash and positive output length for this known nonempty corpus, finite positive wall/CPU/RSS; require exactly the declared head and one exact base arm, the registered corpus and the complete 2 × 7 × 20 grid. Reject unknown rows rather than silently ignoring them. Bind the data to the frozen build manifest. Keep the existing duplicate-round rejection. Add rejection cases for absent output fields and mixed baseline binaries; run existing tests unchanged as well. An expected corpus/output digest or length should be recorded where available; equality between two arms alone is not an independent correctness oracle.

**H2 — freeze the actual comparison, not only the recipe name.** The recipe fetches origin/master and recomputes merge-base at execution time. It records HEAD but builds the current checkout; it neither rejects dirty tracked/untracked build inputs nor checks an existing base worktree's state. Thus “same commit” in commits.txt does not prove the same executable sources. Recording a moving baseline after collection is not pre-registration of that baseline.

At freeze, record exact baseline SHA plus the final candidate/recipe/judge revisions (an external freeze record can name the final commit without a self-reference problem). Both attempts must assert those same values and clean build inputs, including any reused worktree. Keep toolchain/flags/corpus provenance with the bundles. Don't substitute a newly computed merge base during confirmation. No new performance threshold is needed.

**Clarify the bounded retry now.** “No third run” and “one replacement run in the whole experiment” need one explicit algorithm: first and confirmation slots; at most one structurally invalid attempt replaced, hence at most three total attempts; a valid failed primary, control or no-harm bound is never replaceable. Preserve every attempt, and fix invalidity predicates before performance is inspected. Otherwise “invalid” can become a result-dependent restart. The judge's process exit code is 0 for a valid FAIL, so the consumer must read the pass field, not infer admission from shell success. That exit convention is not itself a statistical defect.

These are bounded corrections before data exist. HOLD does not demand a redesigned estimand, stronger control threshold, new hardware or a mechanism experiment.

## Estimand and prediction

The right adoption question is the new default versus old default at the **same requested count**, with production default windows and ordinary unpinned scheduling. It estimates the package users receive: team count, team size, window and their scheduling effects together. It does not isolate any one mechanism. Full end-to-end harness wall includes hashing/instrumentation overhead and describes this benchmark environment, not every application's service time.

Let d_i = log(B_i/H_i). The estimand is exp(E[d_i]), estimated by exp(mean(d_i)); this is a geometric mean of paired wall ratios, not the ratio of arithmetic mean walls or an average user's latency under all workloads.

#220 reports 3×3 at 10 relative to 4×2 at 8 of 1.076 and 1.087; old-default@10 relative to @8 is reported about 0.999. If the latter is a speed ratio, transferring gives about 1.076/0.999 and 1.087/0.999, still approximately 1.08. That is a sensible prior prediction. It is not a same-build head/base estimate with a transferable confidence interval. New binary layout, compiler effects, entry-path differences and scheduler state can change it. Same plans at 11–20 justify a similar expectation, not an observed equality.

## Statistics

For 20 complete pairs, s is the sample standard deviation of d, SE=s/sqrt(20).
- Two-sided speed interval: exp(mean(d) ± t(.975,19) SE).
- One-sided upper bound for head/base wall: exp(-mean(d) + t(.9875,19) SE).
- Independently evaluated quantiles: **2.0930240544** and **2.4334402114**. The script's 2.0930 and 2.4334 are correct to four decimals; rounding down is tiny, not a material miscalibration. Full precision avoids an unnecessary near-boundary discrepancy.
- Four one-sided intervals at 98.75% give at least 95% simultaneous coverage by Bonferroni when each interval is calibrated, regardless of dependence between cells. Requiring every no-harm cell to pass is an intersection rule; the correction is conservative for the single global “all acceptable” admission decision, but matches the stated simultaneous bounds. Keep it.
- The primary's lower bound >1 tests improvement; the point threshold ≥1.03 does **not** establish that the population benefit is at least 3%. Describe it as observed gain plus evidence of a positive effect.
- CPU/RSS predicates compare medians, without uncertainty bounds. Do not call them confidence guarantees.

Student t coverage is exact for independent normal paired log ratios; n=20 does not create independence or normality. The harness balances arm position over four-round cycles (with two arms), but deterministic balance does not remove serial scheduler/thermal correlation. Retain all rounds and report order/time diagnostics; don't select a new interval method after seeing which passes. State nominal, model-conditional confidence rather than a distribution-free guarantee. This is a limitation, not evidence that these data will violate the model.

**Controls:** code matches the inherited screen: fail only if the entire speed interval lies above 1.02 or below 0.98. A wide interval spanning that band passes even if it also allows large drift. For example, an interval [0.95,1.15] passes. This is detection of clear drift, not proof of equivalence or no harm. The separate no-harm rule correctly fails when too wide. Changing the control to an equivalence gate would be a substantive admission change; I do not require or silently make it.

**Two runs:** requiring both separately to pass, without pooling, switching or selecting a best pair, blocks a major forking path. Do not claim squared error probabilities: independence between runs is not established. Confirmation supports reproducibility of a recipe on this machine, not a sampled population of CPUs. Replacement invalidation can select results if correlated with effects; do not claim unconditional experiment-wide 5% coverage from fixed-run intervals.

## Extrapolations to state explicitly

The existing disclosure is honest but incomplete:
- Unmeasured requests include **13–15, 17–19 and every count above 20**, not just a vague “11 or more”. Plan equality motivates prediction; the test does not measure every request.
- The selector identifies cores/cache, not CPU model, microarchitecture, cache-sharing/NUMA structure, affinity restrictions or quota. One i9 cannot establish speed on all matching-topology environments.
- All measured requests use the default window and one nonempty level-9 Wikidata workload. Small inputs, other block sizes/levels, incompressible or highly repetitive data, concatenated streams and custom windows do not gain performance coverage from correctness tests.
- Unpinned here means under this machine's current allowed CPUs, scheduler and background-load policy. It does not cover every placement or contended deployment.
- 8 and 9 controls test unchanged plans, not all unchanged machines and thread counts. CI on four cores remains valuable regression/correctness evidence but cannot observe this selection branch's target benefit.

No additional workload is demanded by this review. Add these limitations so the scope of the agreed admission is not mistaken for universal no-harm evidence.

## Consequence

Expected: about 8% at 10, similar for unchanged chosen plans. Observed here: no timing; formulas verified, two admission-validity counterexamples reproduced, immutable-build gap identified. The review closes checker/provenance uncertainty, not the performance or mechanism gap. After H1/H2 and the retry wording are fixed, the frozen same-request A/B has greater marginal value than another mechanistic detour: it answers the exact adoption question the prior experiment did not.

Migration source head and manifest SHA256 matched. Pinned routing logic run on fresh channel/inbox connector snapshots: REVIEW routes to #145, no unresolved misroutes. Research-only state changes.