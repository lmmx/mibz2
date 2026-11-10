# Pilot-path delta review: PROCEED on both requested scopes

Inbox **6757791774**, dispatching queued 6757715565; previous REVISE 6757361275. Exact heads: #319 `ba5a363e8364ced3970374479e7fa736573f8e7f`; #316 `30a6620d0a8733536284e82c91ff19be0185f148`. Instructions: docs/team/SPEC.md and SPEC.RESEARCHER.md at `a2da49638ac54d7763de1c167bd054f67ffdc763`, with MODE/RULES/PROTOCOL freshly read.

**(a) PROCEED on #319's measurement-tooling delta. (b) PROCEED on #316 §2.8's R2 structural smoke, followed only after its success by the one R600 nonqualifying pilot.** These dispositions do not approve the final qualification repeat count, confidence coverage, adoption, or the separately gated on-target workstation run.

## Independent replay

The revised code admits a synthetic clean R600 run with **42 cells** and all five menu entries. It rejects the old counterexamples:

| Probe | Revised result |
|---|---|
| Missing row-time/start data, with updated manifest hash | INVALID: missing start and row coverage |
| Changed run row after manifest | INVALID: digest mismatch |
| Complete timestamp file with backwards observation, hash updated | INVALID: row times go backwards |
| stamper exit 1, hash updated | INVALID: stamper did not exit 0 |
| Unlisted downloaded artifact | INVALID |
| Missing API job start | INVALID: setup unknown |
| Empty smoke/pilot with qualification-label refusal only | gate exit 1 |
| 29 one-second rounds plus a 100-second round per batch | actual span **129 s**, median/p95/max all 129 |
| m=48 requested over 600 without explicit subrange | ValueError rather than silent tail loss |

A separate clean R2 smoke fixture passes structural admission for all nine shards, with the text corpus explicitly marked unchecked before freeze. This distinguishes working structural plumbing from a gate that refuses everything.

The supplied three unittest modules passed **35 tests** together. The workflow fixture was pointed to its fetched pinned copy because the scratch layout differs from repository bench/. An initial invocation had four FileNotFoundError cases for that missing workflow path; after supplying it and redirecting that fixture path, all passed. The service test module was also invoked directly: 18 tests passed (overlapping coverage, not an additional independent suite). No production decoder timing or workflow dispatch was performed.

Reproductions: probes.py and probe-results.json, tests.txt. Source references and comparison/CI snapshots are in evidence.json.

## Seven requested corrections

1. **Shared admission.** Qualification and pilot now call `service_tg.structural`; the structural smoke command calls it too. The shared path checks manifest schema, unlisted artifacts, history, file digests and shard requirements. The pilot no longer omits those checks. Invalid pilot results clear cells/shards before reporting; no partial favourable analysis is published as admitted. This closes the demonstrated divergence, not a claim that every malformed Python object is converted into a polished diagnostic.

2. **Timing.** Required files include row-times.txt and stamp.exit. The parser requires finite start/row observations, unique full row-index coverage and nondecreasing times; admission checks measurement-marker bounds. The workflow waits for the stamper, records its status and fails on nonzero status rather than masking it. Missing API job start is an explicit pilot problem. CLOCK_REALTIME, millisecond formatting and the 0.2-second polling interval are now correctly described as observation timing, with no scheduling-latency guarantee. Equal timestamps are legitimate. A passing consistency check does not establish a monotonic physical clock or exact emission times; that limitation remains relevant when interpreting very short intervals.

3. **Physical duration.** The estimator partitions observed round ends and subtracts the preceding boundary, including the first batch's stamper start. It therefore measures a batch span rather than multiplying a median. The deterministic 129-second witness now reports correctly. Setup is one observation per shard, not a measured future setup-tail distribution. These are elapsed observation spans including settle/polling, not pure decoder latency.

4. **m=48.** The prospective registration resolves the arithmetic: 600 = 12*48 + 24. Only that diagnostic uses the first 576 rounds. The last 24 remain archived and present in the other diagnostics. The output labels m=48 with rounds=576 and batches=12; the other four lengths use 600. Choosing this subrange before any pilot data is acceptable. Its estimate can differ because both window length and sampled extent differ; do not interpret an m=48 discrepancy as dependence-scale evidence alone.

5. **Smoke prerequisite.** The gate requires both explicit qualification-label refusal (exit2) and a valid structural result with the dispatched label. The workflow produces structure.json, uploads it, and passes it to the final gate. The structure command's exit is intentionally not the final policy decision; absent/invalid JSON fails the gate. Empty artifacts no longer pass on label refusal alone. The pre-freeze smoke's text digest is disclosed as recorded but unchecked. Offline pilot analysis separately uses the supplied exact rustc and corpus digest.

6. **LRV normalisation.** For k batches of length m and n=km, sample variance s_b² gives SE²=s_b²/k and n*SE²=m*s_b². The revised implementation satisfies this identity for every entry in the clean R600 replay, including n=576. The ratio uses the marginal variance over that same subrange. Selecting the largest of these point estimates is explicitly a sensitivity choice; it is not a confidence bound. Low-frequency persistence may exceed all tested windows. The pilot cannot by itself establish Var(mean)≈LRV/R or guarantee 80% all-cell power. Those questions remain for the prospective final design review.

7. **Statistical wording.** The power module no longer promises batch-means validity under arbitrary positive dependence. Cross-batch dependence can make the interval anticonservative; that is now explicit in code and registration. Trend/order/CUSUM are descriptive screens, not calibrated permissions to select favourable data or add retries.

## Identity, CI and scope

All **nine changed tooling/workflow/test files** in the f5e3f0ad→ba5a363e delta have identical Git blob IDs at #316's 30a6620d head. The registration delta from 7c582152 adds the above choices prospectively. The #319 delta contains no decoder build input. This carries forward the earlier measurement-only scope finding; no acceptance rule is relaxed after data.

Fetched check-run summaries for both exact heads show applicable checks completed successfully, with intentional release/publish/deployment skips. These summaries are not independent audits of every log. No smoke/pilot result was available or inferred from ordinary CI green.

Expected: repair the pilot's measurement/admission path before spending a dispatch. Achieved: the earlier false admissions are refused, a clean synthetic control passes, batch timing and the menu now match their stated meaning. **No empirical dependence estimate or performance gap has been measured or closed.** Next information value is the registered smoke, then the fixed pilot; its result can inform, but cannot predetermine, the final qualification design.

Migration head/manifest checks passed before receipt; pinned coord.audit on fresh channel snapshots returned no routing violations. Other QUEUED work was not started. Research only: no code implementation, merge, dispatch, queue change or automation change.
