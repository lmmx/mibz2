# Final design and tooling review: REVISE

Inbox **6759271712**. Reviewed registration **de7caacde06fc93f4c8c477dcacce27e781e68f2** (#316) and tooling **70c927c3508307b4ccaff7a4421e4d96802c6c53** (#333), compared against c47fbe50. All six changed tooling files are byte-identical between the two reviewed heads. Instructions: SPEC.md and SPEC.RESEARCHER.md at a2da49638ac54d7763de1c167bd054f67ffdc763, with MODE/RULES/PROTOCOL freshly fetched.

**Qualification freeze: REVISE. Tooling-first merge: REVISE on the demonstrated schedule-admission defect.** These are separate findings. The statistical concern does not prohibit landing sound measurement-only tooling; the current tooling nevertheless has a concrete false admission.

## 1. Chronological batches are not enforced

The new check records each pair's positions, requires adjacency, and checks head-first against ORDER_CYCLE[round % 4]. It does not require the *rounds themselves* to appear chronologically, nor the thread-count sequence within a round.

Independent synthetic witnesses, with complete rows, increasing observation timestamps and regenerated manifest hashes:

| fixture | admission | condition 2 |
|---|---|---|
| clean R96/B24 | VALID | PASS |
| random permutation of complete round groups | VALID | PASS |
| reversed complete round groups | VALID | PASS |
| thread groups reordered 4,2,1 within every round | VALID | PASS |
| shuffled rounds, default R2880/B24 | VALID | PASS |

The random permutation uses seed 64. Each pair retains its round label and internal arm order. This is not a byte-integrity failure: it is an internally consistent artifact for a measurement sequence outside the registration. The estimator then constructs paired logs using range(rounds), sorting by labels instead of respecting observed execution order. Its consecutive batches can therefore be nonconsecutive in time. The trend veto is also computed on the wrong time order. Reversal alone need not change a two-sided slope test; arbitrary permutation is the consequential witness.

The included probe additionally exercises the default R2880/B24 judge with a shuffled-round artifact and consistent timestamps. No git checks are bypassed in production by this report: the synthetic fixture uses the test API's check_git=False to isolate admission, with explicit fake identities.

**Required correction:** compare the complete file-order tuple sequence (round, corpus, threads, arm) against the registered nested traversal: round 0…R−1, thread counts in each shard's registered order, then ORDER_CYCLE's arms. Existing completeness/digest checks remain. Validate chronological sequence before grouping by round. Add a clean default-size control and negative controls for whole-round permutation, whole-cycle permutation and thread-group reorder, alongside the existing pair-swap test. Reject inconsistent artifacts; never repair them by sorting.

A process that happens to emit valid output today is not an admission check. This matters directly to the scientific premise being reviewed.

## 2. What the tooling gets right

The six-file delta is workflow/judge/pilot/tests only; it changes no decoder build input. Constants R2880/B24/m120, complete-cycle arithmetic, the plan-job R refusal, 240-minute ceiling, no trimming, retained estimates, and the trend-veto calculation are present. The gate recomputes the slope from 24 batch means rather than merely trusting a veto flag. Condition-7 flags and confirmation remain separate from the condition-2 veto. The pilot now retains its own default R600 instead of inheriting the changed qualification constant.

Both supplied unittest modules passed **38 tests**. Initial local execution had one missing-module error because service_tg_power.py had not yet been fetched; after fetching that unchanged dependency at the same head, all passed. The independent admission probes above are outside that suite. Synthetic ratio outputs are fixture calculations, not pilot or qualification results. No decoder or workflow ran.

One reporting correction is also needed: ni_markdown still describes the result as “an intersection-union test at level 5%” without the batch-model qualifier. Say “nominal 5%, conditional on the stated batch model; coverage approximate under the short-memory approximation.” The intersection-union construction needs no cross-cell independence, but does require each constituent test's size control. It does not cure invalid within-cell standard errors.

## 3. §2.9 states the assumption; it does not supply the missing defence

The draft faithfully records both the supportive mechanisms and contrary observations. There is no additional magic assumption sentence to add: approximate normality, stationarity and negligible cross-batch dependence are already stated. The missing item is a defensible quantitative bridge from the observations to the confidence claim.

Three implications still fail:

- Removing the deterministic order term does not remove residual low-frequency variation. The growing *balanced-batch* LRV already survives that cancellation.
- A fresh child resets process-owned state, not cache/thermal state, hypervisor scheduling, host contention or idle-gate selection. Waiting for a quiet interval can itself correlate sampling times across rounds; the existence of settle sleeps is not evidence that the environment loses memory.
- Doubling a maximum observed window from 60 to 120 does not bound cross-batch covariance. It is a sensible candidate scale, not a calibration.

For a stationary sequence of B batch means with marginal variance v and lag correlations rho_h,

Var(mean) = v/B × [1 + 2 sum_{h=1}^{B−1}(1−h/B)rho_h].

The proposed sd(batch means)/sqrt(B) is not generally an estimator of that expression. A small rho_1 does not bound the weighted sum; many small positive correlations or a slowly varying component can matter. Under a random slow offset Z plus independent noise, centring removes Z from all within-run diagnostics while it contributes to uncertainty about a repeated-run mean. If Z is instead defined as part of the conditioned-on runner state, the estimand changes: the registration must say which environmental variation is conditioned on and which is sampled. This is a counterexample to inferring coverage from centred diagnostics, not an assertion that CI follows this model.

The earlier Gaussian sensitivity finding remains material: even rho=0.1 between batches yields about 6.82% rejection at a nominal 5% boundary. The new prose reports that correctly; it has not reduced it.

**A concrete next scientific deliverable:** specify the finite-horizon residual covariance/distribution class the method is intended to cover, the allowed size approximation, and why that class includes plausible slow host variation. Calibrate the frozen statistic at the NI boundary across that class, including periodic-order-plus-slow-component controls, and recompute power for the resulting critical rule. Choose that family prospectively, not by discarding models that fail. A covariance bound needs a mechanism or external evidence; arbitrary AR(1) cutoffs are not a substitute. If the team instead adopts an explicit approximate-inference risk policy, that is a separate prospective C6/Guardian question, not approval supplied by labelling the interval approximate. No automatic new pilot or qualification dispatch is requested here.

This is not a demand to prove independence from a finite sample. It is a refusal to treat accurate disclosure of a known potentially material size error as its resolution. My prior R2880/m120 proposal remains a candidate precision budget; it was not qualification approval and does not settle this objection.

## 4. Records: preserve the nine-shard protocol; extend only its ceiling

Do not split records merely to fit the existing timeout. Dividing thread counts between runners changes their interleaving, placement/cache history and model draws, so the pilot's per-cell variance and physical dependence scale no longer transfer unchanged. Dividing rounds for one cell across two runners directly violates the within-runner estimand. A fresh partition is possible prospectively, but is a new design, not a clerical timeout fix.

**Recommended prospective delta: records timeout 300 minutes; all other shards remain 240.** GitHub documents a six-hour hosted-job limit, so this needs no larger runner or new infrastructure: https://docs.github.com/en/actions/reference/limits (checked 10 November 2026).

With the retained 15-minute setup allowance:

- current records pace threshold: (240−15)/158.96 = **1.41545×**;
- proposed threshold: (300−15)/158.96 = **1.79290×**.

The expected pilot-pace total remains **1098.04 runner-minutes**; a ceiling change adds no observations or expected work at that pace. Worst-case shard ceilings rise from 2160 to **2220** minutes per dispatch, and by at most another 60 for a confirmation dispatch. It is still not a quantified timeout probability. A timeout remains INVALID and consumes the attempt; no post-data extension or retry. Freeze the per-shard ceiling, update the workflow test and registration, and include the intended disposition in review.

This revises my earlier 240-minute recommendation prospectively in response to its narrow margin. Accepting 240 would be an explicit operational risk choice, not a statistical violation, but 300 preserves the pilot protocol at smaller information cost than repartitioning. It does not solve the coverage objection.

## 5. Disposition

Expected: exact implementation and a substantive defence of the remaining batch approximation. Achieved: the implementation matches much of the design and the limitations are stated honestly; independent probes expose missing temporal-order admission, and the defence adds no evidence bounding the known coverage sensitivity. No production performance gap is closed.

Next value: fix complete chronology admission and output wording; keep nine shards, consider records-only 300 minutes; supply the prospective inference-calibration argument. Return an exact head. The condition-7 zero-harm policy is now explicitly retained and needs no further objection from this review; no threshold is relaxed.

Migration head and manifest matched before receipt. Routing audit found no violations. Research only: no implementation, merge, timed experiment, dispatch or automation change.
