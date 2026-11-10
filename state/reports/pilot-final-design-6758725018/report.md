# Pilot to final design: R2880 candidate; coverage remains conditional

Inbox **6758725018**, dispatching QUEUED 6758589976. Pilot run **42748681243**, head **30a6620d0a8733536284e82c91ff19be0185f148**, base **3e1a7ec361ee6c8ea6b540264aa69189d4823d9e**. Analysis source **9c6a6b070673c3773e7e52da2e428a904e09adfd**. Instructions: docs/team/SPEC.md and SPEC.RESEARCHER.md at **a2da49638ac54d7763de1c167bd054f67ffdc763**; MODE/RULES/PROTOCOL read.

**REVISE #316 before freeze.** The concrete candidate below answers the six design questions. PROCEED to prepare it for review; **no qualification-freeze PROCEED yet**. The pilot supports a useful precision calculation, not certified confidence coverage. The missing part is the justification of approximately independent 120-round batch means, plus the exact prospective registration/tooling. No additional pilot or qualification dispatch is authorised by this review.

## 1. Replayed evidence, including the omitted column

Downloaded all ten artifacts through the GitHub connector. Each ZIP's SHA-256 matches the live API's digest. Replayed the shared structural admission: valid, no problems, all 42 cells. Recomputed the complete frozen five-length menu without displaying a candidate mean, ratio or qualification bound. Manifest SHA-256:

`1fdc0b17340afdd45e883a4a920d89b536dee3a3274147f9cb77faa1f564a1b0`

Regenerated Markdown SHA-256:

`ac11e83d6875bea9a8a672439e1dab971d974201bddc0f7ae3938223358ba9bd`

These match the prefixes/suffixes in [archive comment 6758585874](https://github.com/lmmx/mibz2/pull/316#issuecomment-6758585874). `menu.json` preserves the full output; `evidence.json` identifies every artifact and source. This is an artifact/admission replay, not an independent measurement or a new API audit of every historical job.

The displayed table omits **m=20**, lags 2/10/60, and correlations between batch means. That omission matters: **pattern/2/wall's largest LRV is at m=20**, ratio 1.233071, giving planning sigma **0.20139894**. The four displayed columns would use 0.19145862. Power must use all five, as registered. Pattern/2/wall is the limiting precision cell, despite most discussion focusing on pattern/1's order effect.

## 2. Order is periodic; raw ACF is not residual dependence

At the pilot head, [examples/ab.rs](https://github.com/lmmx/mibz2/blob/30a6620d0a8733536284e82c91ff19be0185f148/examples/ab.rs#L1813) rotates and then reverses the arms. For two arms the head-first signs are

`s_i = (+1, -1, -1, +1), repeated`.

Under `X_i = mu + beta*s_i + epsilon_i`, the order contrast is `2*beta`. The phase-averaged deterministic contribution has lag-2 and lag-30 correlation negative, lag-60 positive, and lag-1/5 approximately zero. From the pilot's contrasts and marginal variances:

| pattern/1 | beta² / marginal variance | observed ACF 2 | ACF 30 | ACF 60 |
|---|---:|---:|---:|---:|
| wall | 0.24169 | −0.20085 | −0.21749 | +0.23210 |
| CPU | 0.48471 | −0.46900 | −0.45574 | +0.44369 |

The signs and approximate magnitudes follow directly from the schedule. This explains much of the apparent negative dependence; it neither establishes residual independence nor identifies the physical cause of the order contrast. An iid SE on the contrast is descriptive, not a calibrated significance test.

**Use the existing complete four-round cycles, not a newly fitted iid regression.** Set m divisible by four; require every batch to begin on a cycle boundary with the exact schedule. With m=120, each batch contains 30 cycles and 60 rounds in each order. The mean is the average of the two order-stratum means. Subtracting an estimated constant beta changes no batch mean because sum(s_i)=0, so it buys no variance reduction for this estimator. Do not subtract beta² from the already order-balanced batch LRV: that would remove the same term twice.

There is a stronger cancellation than even-length balance: within a complete cycle, both sum(s_i)=0 and sum(i*s_i)=0. Thus an order coefficient varying linearly within the cycle also cancels exactly. Nonlinear carryover, common phase effects, or rapid changes do not. Those remain part of this specific harness's estimand; balanced order does not establish a universal order-free production effect.

## 3. Proposed numbers and the precise dependence assumption

| item | proposed prospective value |
|---|---|
| rounds per cell | **R=2880**, fixed |
| batch length | **m=120** consecutive rounds; complete four-round cycles |
| batches | **B=24**; no incomplete tail |
| estimator | mean of all paired log ratios; equal-weight mean of the 24 batch means |
| NI bound | exp(mean + t(0.95,23) × sd(batch means)/sqrt(24)) ≤ 1.02 |
| planning variance | maximum LRV over all five registered pilot entries, per cell |
| sensitivity budget | also calculate with **twice** that LRV in every cell |
| cells | all existing 42; unchanged nine shards; no cross-runner pooling |

**Model:** after aggregation over complete order cycles, the residual process has a stable mean, finite long-run variance and sufficiently short memory for 120-round batch means to be approximately identically distributed, normal and independent. Under exactly iid normal batch means the t bound is exact. Under the short-memory approximation it is only approximate. No reversibility, AR(1) structure, or fleet generalisation is assumed from the raw ACF.

Why 120? It doubles the longest pilot window and keeps complete cycles; at the observed mean pace it spans roughly **125–397 seconds**, depending on shard. It leaves 24 variance units rather than buying a longer window with only ten batches. **It was not itself in the pilot menu.** Its adequacy is an extrapolation, not an observed plateau.

The Coordinator's “dependence is weak” is too strong. Pattern/1's LRV rises by **2.51× wall and 2.37× CPU** from m10 to m60. Its batch lag-1 correlation is about 0.21–0.24 at m20–30. At m60, onebyte/4 has sample batch correlations **0.417 wall, 0.435 CPU**; only ten means were available. Both noise and residual persistence remain plausible. Marginal variance also includes the periodic order term, so a low LRV/marginal ratio can conceal residual dependence.

**A concrete falsifier of unconditional coverage:** simulate Gaussian batch means with B24 and AR(1) correlation across batches; centre at the NI null boundary. Seed 6758725018, 200,000 draws per row, exact covariance and t quantile:

| batch correlation | nominal 5% NI test passes | with proposed trend veto |
|---:|---:|---:|
| 0 | 5.0195% | 5.0180% |
| 0.1 | 6.8230% | 6.8195% |
| 0.2 | 8.8760% | 8.8660% |
| 0.4 | 14.1200% | 14.0190% |

Monte Carlo SE is about 0.05–0.08 percentage points. These are mathematical sensitivity scenarios, not fitted CI correlations. More rounds or a trend screen does not repair an invalid variance model. Nor is multiplying a planning variance by two an inferential correction.

The final review must judge the stated residual-memory approximation, explicitly, against the full menu and mechanism. It must not convert the chosen m into proof. If the team cannot defend it, this candidate is not freeze-ready: revise the inference prospectively rather than run and select whichever analysis passes. Primary background: [Flegal and Jones (2010)](https://arxiv.org/abs/0811.1729), batch/spectral variance consistency under stated conditions; [Geyer's batch-means discussion](https://www.stat.umn.edu/geyer/3701/notes/mcmc-bayes.html), requiring sufficiently long and numerous batches. Neither source certifies this hosted-runner process.

## 4. A numerical trend veto, with its cost and scope

If a veto is retained, freeze this one rather than tuning to the pilot's raw |t|=3.03:

1. For each cell regress its 24 batch means on batch index 0…23 with an intercept.
2. Form the usual slope t statistic using residual df=22.
3. If **|t| > t(1−0.01/(2×42),22) = 4.3806308666**, invalidate that cell's qualification and therefore condition 2. Missing/nonfinite inputs invalidate; if residual variance is exactly zero, define t=0 for zero slope and |t|=infinity otherwise.
4. Keep every row and the original NI estimate. No detrending, removal, estimator switch, replacement shard or retry. CUSUM and raw order/trend diagnostics remain descriptive.

Under constant-mean iid Gaussian batch means, the family probability of this veto is at most 1%, without independence across cells. That gives a power-loss allowance of at most 0.01 **under that model**. It is not a 1% bound under drift/dependence, not a goodness-of-fit certificate, and not a calibrated test conditional on passing. The simulation above shows how little it corrects correlation-driven over-rejection. Pilot pattern/1 wall's descriptive change is about 1.46% over the run: relevant warning, not a reason to fit it away.

## 5. Tails: retain; budget the waiting, do not trim the decoder samples

The elapsed round span is not the child's wall time. `ab.rs` lines 1666–1670 checks load, sleeps **20 seconds** when busy, then checks again; this occurs before the measured child. Across all shards, the sets of rounds with observation span >10 s exactly equal the sets containing a row with reported `waited_s` >10: onebyte 26, records 33, runs 25, text 30, wikidata/1 2, wikidata/2 18; none in the other three. Onebyte has 27 such rows in 26 rounds, consistent with the ≈42-second round. This is strong support for settle waiting as the long-span mechanism, not evidence that decoder-time tails require trimming.

**A field-definition trap:** `waited_s` is emitted after the child as elapsed since before settle, minus one settle interval, rounded to integer seconds (`ab.rs` line 1761). It includes child/parent overhead and is **not pure wait duration**. Wikidata/1 reports 1 on 1198 rows and 21 on two. Do not sum it as exact idle waiting or subtract it from the child's wall time. A future instrumentation correction would need an explicitly reviewed measurement delta; none is implemented here.

Keep all paired log ratios, including noisy child measurements. No robust replacement estimator or tail exclusion: either would change the estimand and need its own power calculation. Preserve geometric-mean head/base time, conditional on the drawn runner and this idle-gated harness. This does not measure request latency including the harness's waiting.

Keep the **same nine shards**. Repartitioning would alter cache/thermal and placement histories, lose the pilot's variance transfer, and redraw the CPU model assignment. Extend each job timeout to **240 minutes**, prospectively, instead of splitting after a failure.

At the pilot pace, R2880 measurement totals **963.04 runner-minutes**, plus the existing conservative setup allowance of 15 minutes ×9 gives **1098.04**, excluding plan/aggregate. Records is slowest: **158.96 minutes** measurement, **173.96** with setup. At 1.4× the observed measurement pace: **237.54 minutes** including setup, just within 240; total **1483.26 runner-minutes**. This is a transparent budget sensitivity, not a future-tail quantile. Setup actually took 56–98 seconds, but one draw per shard does not justify cutting the 15-minute allowance. A harm-confirmation dispatch can roughly double cost. Job ceilings total 2160 shard-minutes per dispatch. Standard hosted compute only; no new infrastructure or spending authority.

A data-bearing timeout remains INVALID and consumes the attempt. No rescue by dropping cells, dividing them afterwards, selecting another model or adding rounds.

## 6. Power at 1.005, and a separate adoption-policy conflict

Independent integral reproduced from `service_tg_power.py`. For each cell, lambda = log(1.02/1.005) × sqrt(R/LRV); integrate over chi-square(df=B−1). All-cell lower bound is max(0,1−sum(1−power_c)), not the product. Values below assume iid normal batches and the stated pilot variance scenario:

| R / m / B | pilot max-LRV | twice pilot max-LRV |
|---|---:|---:|
| 600 / 30 / 20 | 0.52213 | 0.11668 |
| 960 / 48 / 20 | 0.70887 | 0.42306 |
| 1200 / 60 / 20 | **0.79124** | 0.52213 |
| 1440 / 60 / 24 | 0.85610 | 0.60172 |
| 2400 / 60 / 40 | 0.97098 | 0.80471 |
| 2400 / 120 / 20 | 0.96610 | **0.79124** |
| **2880 / 120 / 24** | **0.98548** | **0.85610** |

Allowing the 1% family trend-veto loss, the proposed design gives lower bounds **0.97548 / 0.84610** under the same constant-mean Gaussian model. This motivates R2880 rather than the nearly marginal R2400/m60 or the inadequate R1200. Twice the pilot variance is a sensitivity reserve, **not a confidence upper bound**. Selecting the maximum of five correlated estimates does not establish an envelope for a new runner; each m60 estimate has only nine variance df. No guaranteed power or confidence coverage is claimed.

**These are condition-2 powers, not probabilities of adoption.** Section 2.6/condition 7 separately rejects confirmed wall harm using intervals above **1.00**, not 1.02. More precision makes a true 0.5% slowdown easier to confirm. Illustratively, at the hypothetical true ratio 1.005 and the pilot's wikidata/1/wall planning sigma 0.01212046, R2880/B24 gives slower-flag noncentrality **22.08**, hence essentially certain flagging under the same model. A second runner with that same response would confirm harm despite both NI tests passing. Confirmation draws different hardware, so this is a conditional example, not a predicted actual confirmation outcome.

Do not call “80% all-pass at 1.005” an 80% adoption chance. If the policy intentionally vetoes any reproducible slowdown, retain it and say so. If the intended trade-off tolerates up to 2%, the apparent conflict needs a **separate prospective Guardian-reviewed C6 decision**, before data. This review neither removes nor relaxes the existing harm rule.

## Disposition and next consequential move

Expected: the pilot would supply dependence, noise and runtime information. It did, and the artifact replay agrees. The material advances are: phase structure explains the raw negative correlations; the omitted m20 column sets the precision bottleneck; long elapsed rounds track settle waiting; and precision for NI works against a separate zero-harm veto. **No production performance gap is measured or closed.**

Prepare one exact registration/tooling delta with R2880/m120/B24, full-cycle checks, the stated bound/model, the numeric veto, unchanged shards, 240-minute timeout, all-row retention and honest budgets/power. Correct the `waited_s` interpretation without silently changing measurement semantics. Return that exact head for review, with a reasoned defence of batch adequacy and the Guardian's disposition on any acceptance-policy question. No rerun of the pilot is requested; no qualification data yet. Conditional precision is not permission to skip the unresolved coverage argument.

Research only. Migration head and manifest digest matched before receipt; routing audit returned no violations. No decoder execution, timing, workflow dispatch, implementation, merge, queue change or automation change. Persistent task left enabled.
