# D077 noninferiority design — independent review of inbox 6756614616

**REVISE before freeze. PROCEED with one explicitly pre-freeze design pilot; no qualification dispatch yet.** Sharding and the intersection–union rule are defensible for the stated conditional estimands. The present batch assumptions are unvalidated, and the judge does not enforce the attempt/failure rule it describes.

Reviewed head **eb46b5a80bc1e73f226ebb22a31ef04d3b869f49**, PR #316; assignment [6756614616](https://github.com/lmmx/mibz2/pull/162#issuecomment-6756614616), dispatching [6756572763](https://github.com/lmmx/mibz2/pull/145#issuecomment-6756572763). Instructions: docs/team at **a2da49638ac54d7763de1c167bd054f67ffdc763**. Read the registration, target-gated-ni.yml, service_tg.py, service_tg_power.py, test_service_tg.py and its helpers, D077's channel ruling and the supplied smoke run metadata. Smoke 42736742147 is a successful push run at 7e325dd0b106140e223d6039c9704e6cda16d77c, not qualification at the reviewed head. Its raw artifact was not replayed.

## Findings that decide readiness

1. **Thirty rounds are about 35–67 seconds at the tabulated pilot pace, not 5–8 minutes.** At the 1.75× allowance they are about 61–117 seconds. The assumption that dependence decays inside the batch must be justified at that time scale.
2. **A correct t quantile is not a calibrated test for dependent batch means.** Constant mean and normal marginal observations alone do not establish the claimed 5% level. The power table assumes iid paired logs; the estimator is being proposed precisely because those logs may not be iid.
3. **The artifact judge admits forbidden histories.** It archives jobs but never uses them, identifies measured attempts only by a start-marker file, and does not bind several recorded identities. Concrete reproductions below.
4. **R=900 with B=20 means 45-round batches.** Those batches cannot each balance a two-round arm-order cycle as §2.3 promises. A prospective longer design should preserve even batch length (for example R=960, B=20), and still requires calibration.
5. **Mode adjustment must follow the estimand and causal position of the mode.** A plan-induced state belongs to the total plan effect; an outcome-derived fast/slow label is not an admissible primary adjustment merely because it reduces variance.

## 1. Batch means and the trend check

For cell logs X, let m=R/B and Y_b be the consecutive batch means. The code computes

    U = exp(mean(X) + t_.95,B-1 * sd(Y)/sqrt(B)).

I independently checked its arithmetic. The iid normal Y case gives the stated t law. For dependent X, the relevant variance is the long-run variance, gamma(0)+2 sum_{h>=1} gamma(h), not the marginal ten-round sd squared. Even when dependence decays within 30 rounds, adjacent batches share correlated boundary observations. Finite batch length matters.

**Counterexample, simulated rather than observed on CI.** Stationary Gaussian AR(1), E[X]=log(1.02), Cov(X_i,X_j)=rho^|i-j|; R=600, B=20, m=30. I formed the exact covariance matrix of the twenty batch means and drew 100,000 Gaussian vectors with fixed seed 6756614616. Applied the registered one-sided bound at the null boundary:

| rho | boundary pass rate | Monte Carlo SE | E[estimated SE²] / actual Var(mean) |
|---:|---:|---:|---:|
| 0 | 0.05018 | 0.00069 | 1.0000 |
| 0.5 | 0.05355 | 0.00071 | 0.9555 |
| 0.9 | 0.08467 | 0.00088 | 0.6935 |
| 0.99 | 0.28657 | 0.00143 | 0.1198 |

This is a falsifier for a distribution-free “size 0.05 whatever sigma, R and B” reading, not evidence CI has rho=0.9. For rho=0.9 the lag-30 correlation is already only about 0.042; saying dependence is “short range” does not make the finite-sample error negligible.

Independent algebra used for the SE check: if C is the B×B covariance matrix of Y, Var(mean Y)=1'C1/B² and E[s_Y²/B]=(tr(C)-B Var(mean Y))/(B(B-1)). This explains the understatement without relying on simulation.

**Recommendation:** retain batch means as a candidate, not certify 20×30 yet. Use the pilot to examine signed order effects, lag dependence over more than lag 1, batch-mean dependence, trends/change points and sensitivity of long-run variance to several predeclared block lengths. Longer batches trade reduced dependence bias for fewer effective units; changing only B or R has no universal answer. “Heavy tails tolerated” also needs finite moments and adequate block averaging, not just 30 observations.

**Trend as a validity condition:** yes, prospectively specified gross model failure should prevent a qualification claim. Freeze a model-adequacy failure rule, its numerical thresholds, its scope and disposition before qualification; retain all rows, mark the affected qualification unestablished/invalid, and do not retry or select a different estimator. A non-significant trend test or low lag-1 correlation is not a proof of validity. A diagnostic veto can narrow an already valid test's rejection set; it cannot calibrate an otherwise invalid bound. Do not condition reported 95% coverage on passing a diagnostic screen without justifying that combined procedure.

Source context, not a theorem about this CI process: Flegal & Jones, [Annals of Statistics 2010](https://arxiv.org/pdf/0811.1729), studies batch/spectral variance estimators with explicit dependence, moment and batch-size assumptions and finite-sample limits. It does not license arbitrary fixed 20×30 batches. [NIST's mean-interval treatment](https://www.itl.nist.gov/div898/handbook/eda/section3/eda352.htm) supplies the ordinary t construction; it supplies no independence certificate for these batches.

## 2. p90 planning, R=600 versus a longer run

I fetched links.jsonl and verified the bytes at **bench-results f99e2f358ba7d0885b8129e1dc7f60e59f30a39a**. The script selects exactly **47 runs, 42 cells**, 2026-10-28 through 2026-11-09, PR69 through PR311. Its displayed medians/p90s and the published R600/R900 p90-at-null figures reproduce.

I also recomputed power independently by integrating
E_V[Phi(lambda - t_.95,19 sqrt(V/19))], V~chi-square(19),
lambda=(log1.02-log(true ratio))*sqrt(R)/sigma. Thus the below +0.5% means exactly ratio 1.005:

| R / B / m | true ratio | all-cell lower bound | independence product |
|---|---:|---:|---:|
| 600 / 20 / 30 | 1.000 | 0.8069 | 0.8193 |
| 600 / 20 / 30 | 1.005 | 0.0015 | 0.3344 |
| 900 / 20 / 45 | 1.000 | 0.9649 | 0.9652 |
| 900 / 20 / 45 | 1.005 | 0.6557 | 0.6973 |
| 960 / 20 / 48 | 1.000 | 0.9747 | 0.9749 |
| 960 / 20 / 48 | 1.005 | 0.7192 | 0.7465 |

The supplied script uses mu=0.005/0.01 as log effects rather than log(1.005)/log(1.01). That small labeling difference explains the R600 lower bound rounded to zero; it does not change the decision.

**p90 at every cell is a sensitivity scenario, not a confidence envelope or guaranteed pessimistic future.** Each historical sd is estimated from ten rounds; under iid normal sampling alone its 95% interval for the population sd is approximately [0.688s,1.826s]. The p90 of 47 such estimates mixes estimation error, runner and candidate heterogeneity, and selected history of merged PRs' final runs. It cannot estimate minute-scale dependence from ten-point summaries.

For a specified stationary process, marginal sigma must be replaced by a justified effective/long-run variance in power planning. Positive dependence need not merely reduce power: when the SE misses it, nominal pass rates can rise through anticonservatism. Separate correct uncertainty from apparent power.

**Decision on R:** do not freeze 600 merely because null power looks adequate. Define the desired power at a scientifically relevant small slowdown, such as 1.005, then plan against plausible long-run variance. If that objective is about 90% all-cell power in the p90 scenario, neither 900 nor 960 establishes it, even before dependence. R960 is a useful even-batch candidate, not approval of that design. The current slowest-shard allowance scales to about 15+38.9×1.6=77.2 minutes, leaving only 12.8 minutes of the 90-minute budget; pilot the real setup and tails before treating that as feasible. Another prospective partition may be needed.

**Numerical check:** on this runtime (SciPy 1.17.0), the published noncentral-t CDF call produced NaN for at least one R900/median/null cell. The script still printed a zero lower bound through max(0,NaN), with NaN product. Reject non-finite cell powers; use a validated tail/integration calculation. The independent integral above avoids that failure. Do not turn it into a claimed low-power result.

## 3. A pilot is worth its cost

**PROCEED with a bounded, clearly labelled pre-freeze pilot design.** The historical question is ten-round marginal noise; the intended study is a 20–40 minute serial process on a differently heated, single-corpus shard. That missing information changes both size and cost. R=2 smoke answers plumbing only.

Concrete next experiment: one fixed R600 pilot dispatch of all nine shards at the proposed placement/settle/ordering, with raw per-run sequence and timestamps and exact provenance retained. Label it pilot explicitly, reject it from qualification, and disclose it alongside every earlier exploratory run. No extra rounds because a pilot estimate is inconvenient. Estimate variance/dependence and budget, not whether the candidate already “passes.” A separate matched A/A check is useful for calibration of systematic order/carryover effects if those appear; it is not a substitute for the candidate/base covariance and interactions.

Before that pilot, freeze its information objective and analysis menu: physical batch durations, within/between-batch dependence at several lags, order-stratified contrasts, long-run variance sensitivity, trend/change-point diagnostics and shard/setup duration. Then set one final R, batch length, allocation/order mechanism, adequacy veto and calibration record for **fresh** qualification observations. A single pilot cannot prove mixing or cover all hosted runners; state the residual assumption and stress-test it rather than claiming a guarantee. C6 permits this prospective design work; no qualification data may be retrofitted.

## 4. Estimand across runner models

**Recording model per shard is sufficient for the explicitly narrow drawn-runner estimand; balancing models is not required by the IUT.** If each marginal test has size at most alpha conditional on the assigned design, P(all reject) <= P(reject any selected true null) <= alpha, irrespective of dependence across cells. The Guardian's D077 argument holds on that premise.

The claim is a conjunction of 42 conditional cell claims on the runners/conditions actually drawn. It is not one common hardware response surface, a fleet mean, or universal non-target coverage. Random model assignment cannot establish safety on an unobserved model. “Balanced models across different corpora” would still confound corpus with model. For a model-general claim, use a prospectively crossed cell×model design or a justified mechanistic extrapolation and reviewer-accepted coverage scope; do not selectively rerun shards until a convenient model appears.

Record model, topology, image, toolchain, time and relevant resource limits. The same applies to confirmation: a new runner can test recurrence across draws, but is not a literal replication of the first runner's conditional estimand. State which interpretation condition 7 uses.

## 5. Setup-only recovery: acceptable principle, not enforced yet

A verified setup failure before the measurement step begins may be recovered under the predeclared rule, even if other shards have started. Never choose recovery from the other shards' performance. Retain each attempt and reason. Unknown evidence of whether measurement began must fail closed. “No qualification observations” is the relevant distinction; preflight itself does decode work.

**Independent synthetic probes against the unchanged pinned judge, using its own fixtures:** all four returned valid=true, passes=true, problems=[]:
- a cancelled job with a complete-looking artifact;
- attempt1 containing runs but no measurement-started marker, followed by attempt2;
- an earlier cancelled measurement job with no artifact, followed by a good artifact from attempt2;
- a wrong workflow_sha and inconsistent shard run_attempt.

These are admission counterexamples, not allegations that smoke or real CI artifacts were forged. The small fixture uses R40/B4 and disables only the local git ancestry check; the affected admission branches are identical at R600/B20.

Cause: ni_judge never consults man.jobs; measured means only “marker name appears in manifest files.” manifest iterates downloaded directories, so an API-listed attempt with no downloaded artifact can disappear. shard_problems ignores workflow_sha/run_attempt and merely substring-checks the workflow ref. Hashing the downloaded files when building a manifest checks later stability, not completeness of the original attempt history.

**Repair before pilot/qualification admission:**
- Reconcile API-listed jobs/artifacts/attempts with downloaded artifacts, including missing downloads; preserve measurement-step status/timestamps in jobs.jsonl (the workflow currently strips the steps).
- Bind shard, attempt, run, repository, workflow identity and expected source; check authoritative job completion and measurement status.
- Any run rows OR marker OR authoritative measurement-start evidence means measurement occurred; absent artifacts/unknown status are not permission to retry.
- Require a terminal successful measured job and complete expected files; a job failed/cancelled after measurement fails per §2.5.
- Keep original qualification and confirmation run IDs in the judge output. confirm currently accepts the same verdict twice as its own confirmation. It must require a distinct registered confirmation dispatch, while keeping condition2 immutable. This is provenance, not a reason to broaden the 42-cell test.

All **16 supplied unit tests passed** locally. They validate several ordinary refusals; they do not cover these cases. Add adversarial fixtures for the above, not merely more happy paths.

## 6. On-target modes: register the causal choice, not an exclusion after the fact

Waiting for the discriminator before freezing on-target inference is correct. Its identification itself still needs review.

- If a pre-existing state multiplies both arms equally within a pair, logs cancel it. If effects differ by state, the estimand is a state-weighted plan effect; if a switch is position/arm-dependent, alternating order alone need not remove bias or serial dependence.
- A mode induced by the candidate is a mediator of its total deployment effect. Adjusting it away answers a different question and can erase real costs/benefits.
- A mode inferred from the very decoded wall time being tested is outcome-derived; excluding “slow” observations or conditioning on that label can manufacture an effect.
- An independently measured **pre-treatment** mode can be a registered blocking/covariate variable. Specify timing, classifier, missingness handling, arm×mode interactions, deployment weights and estimand before any qualification data. Preserve switch pairs. Model-standardized and unadjusted estimates must be distinguished.
- An external state is not automatically irrelevant to deployed throughput just because it was not caused by the arm. The target usage regime determines whether its distribution belongs in the estimand.
- If short-process behaviour caused the modes, choose the intended fresh-process or sustained-decode regime explicitly. If long state persistence remains, 128 paired ratios do not automatically supply 128 independent observations. Freeze valid uncertainty at the right unit; do not import the old iid variance merely because the mode has a name.

Prefer a prospective balanced randomized order within fixed blocks, with seed/schedule archived and carryover addressed, if compatible with the intended experiment. Randomization is useful protection against deterministic time/order aliasing, not a cure for interference or arbitrary model error. A mode exclusion is not presently justified.

## Disposition and next value

Expected: a concrete D077 design that makes the unchanged 2% bound testable. Achieved: reproduced the published independent-model power arithmetic; sharding gives viable wall-clock budgeting and narrow, interpretable per-runner claims. Remaining gap: actual long-run variance/calibration and a judge that enforces the specified attempt history.

Next value: first fix provenance, then the one pre-freeze pilot, then choose/freeze calibrated batches and R. Do not spend a qualification attempt on an untested 20×30 assumption or call an ignored-job manifest a no-retry guarantee. The margin, all-cell rule, old results, and no-retry policy stay unchanged.

This review does not decide action pinning, workflow-to-master integration, spending or coverage questions assigned to the Guardian; nor run the pilot, alter production, merge, or amend authority. Research scripts, simulated evidence and records only. Migration check passed; routing audit after6756608043 found no new issue in the pinned five-channel scope. The task remains enabled.
