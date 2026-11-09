# Bimodality: failure observed, impossibility not established

Assignment: [6750333637](https://github.com/lmmx/mibz2/pull/162#issuecomment-6750333637).
Scope: prospective advice and closing language only. This is **not a replay or re-judgement** of either attempt.
Sources read: the assignment; Coordinator LOG [6750327222](https://github.com/lmmx/mibz2/pull/145#issuecomment-6750327222); registration at `1af3888a55da23e2a7ac03dd7a63a71aa159f691`, particularly fidelity and R11.6–R11.7. The raw bundle and its digest were not independently checked in this review. Numerical observations below are attributed to that LOG.

## 1. What the evidence supports

**No:** bimodality plus the load difference does not establish that the harness *cannot* reach fidelity on the i9 under its current conditions. It establishes that the two reported attempts did not, and gives a plausible source of inadequate precision. It is enough to stop this registered sequence as specified, without proving impossibility.

The missing quantity in that inference is **within-pair state agreement**, not marginal wall spread. Write a log wall as
`Y_a(t) = mu_a + S(t) + epsilon_a(t)`.
A paired contrast contains `S(t_a)-S(t_ref)`. Arbitrarily large shared modes cancel if both members experience the same state; modest state changes between them do not. Seven bimodal marginal distributions therefore neither prove nor rule out precise paired contrasts. Short streaks make pair spacing and arm order consequential; they do not establish a cause. Small marginal lag-1 summaries do not settle the relevant paired dependence.

An illustrative counterexample/calculation, **not a fit to this bundle**: suppose a common twofold state multiplier has fast probability 0.2, and the two members' states are independent. The paired log SD is
`sqrt(2 * .2 * .8) * log(2) = .3921`.
At 20 pairs, t19=2.093, a ratio centred at one has interval approximately **[0.8323, 1.2014]**. If the states instead match perfectly, this component of paired SD is **zero**, despite identical marginal bimodality. R11's centred fidelity requirement needs SD at most **0.0423**; the difference between these examples is state coupling, not the machine's identity. This explains a route to poor precision without proving a permanent limit. Real ratios may be off-centre and other variance remains.

The reported simultaneous CPU and wall increase is more informative than wall alone. Conditional on comparable child CPU accounting and identical work, pure descheduling with unchanged CPU work would lengthen wall without doubling CPU seconds. Thus that *simple* account is insufficient. Spinning, retries, cache/contention costs, frequency changes or different executed work remain possible; the observations select none uniquely. CPU seconds are not an instruction count. Endpoint H labels do not certify the frequency throughout the child interval.

The earlier service-guarded run is not a controlled quiet/busy comparison: time, registration, arms and harness context differ together. It demonstrates that a nearby run on the same host had tighter reported walls; it cannot identify load as the cause of this run's modes. Neither load1 nor others_busy is a validated sufficient statistic for the interference affecting the child.

## 2. A future registration: condition, cause, or both?

Do not prescribe a quiet threshold retrospectively from 3.1 versus 0.41, nor select fast/H runs. A quiet condition is a possible **prospective experimental control**, not a remedy already demonstrated by this contrast.

For genuinely separate future work on this harness, require a reason to expect information beyond these failed attempts. Prefer a bounded, untimed accounting/design audit first: identical corpus and decoded work, child boundaries and CPU accounting, effective affinity/quota, wait/spin paths, and what the existing telemetry can distinguish. That audit may eliminate a harness explanation without another timing run.

A complete causal diagnosis is **not logically required** before every future registration. But “wait until quiet and try again” supplies neither a causal test nor a precision argument. A justified future design could combine:
- a defined environmental eligibility rule measured before outcome observation, with fixed sampling window, finite timeout/refusal and all refusals retained;
- an explicit paired-state mechanism and fixed allocation/order, plus pre-specified diagnostics that distinguish its alternatives;
- a bounded information objective and fixed stopping rule, separate from production qualification.

Thresholds need an engineering or independent calibration basis, not selection to recover the favourable mode. If quietness defines eligibility, conclusions are about that eligible environment. A later busy period must be handled by the rule fixed in advance, not by dropping adverse pairs or restarting. Post-outcome telemetry may remain descriptive without becoming an admission gate. Any method/workload/repeat-policy change remains subject to the authenticated authority applicable when it is registered.

These are design requirements, **not a proposed third run of this line**. Honour R11.6: no third attempt, no pooling, no extension for precision. The current task still has the old C6 pin; reading the newer registration does not authenticate a policy expansion or retrospectively endorse the execution.

## 3. Closing language

Recommended:

> The registered run-fed-leader timing sequence on the i9 is closed. Each of its two separately reported attempts was VALID and primary-INCONCLUSIVE, and each failed control fidelity. Neither establishes the registered performance benefit or clean run-fed attribution. The second attempt's interval describes that attempt alone; the attempts are not pooled and no third attempt follows. The cause of the reported wall/CPU bimodality remains unresolved.

“Does not separate run-fed from the default at this precision” is acceptable only as shorthand for **inconclusive under the registered rule**. It must not imply equivalence, no effect, a campaign-wide confidence interval, or intrinsic inability of the i9/harness to resolve an effect. Prefer the explicit wording above because the fidelity failure independently limits attribution.

Expected: a control inside [0.98,1.02] and an informative primary comparison. Reported: fidelity failed and primary inconclusive twice. No performance qualification gap is closed; the new information is that unexplained state variation persists across all arms, with CPU cost moving too. The next useful move is closure documentation and, only if a distinct future question warrants it, a bounded mechanism/accounting investigation—not another repeat of this sequence.

## Provenance

Pinned specs/rules: `a4f97714cc14366984015869d9d094d89488ac57`. Migration head and manifest digest verified before receipt `c5ee369ecf15d3831e77f8bf1bd0bdf8f221577b`. Routing audit: five historically resolved findings, zero unresolved. No raw-data reanalysis, new timing, implementation, merge, authority change or automation change.
