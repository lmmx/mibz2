# PR383 adoption registration: one-round Researcher review

Disposition: **one targeted correction — replace the I2/I3 lower-bound rejection tests with upper-bound acceptance tests before timing.** The fixture change is sound at source level. The recipe collects the main gating contrast, but its printed summary is not an admission checker.

Assignment: [6877415766](https://github.com/lmmx/mibz2/pull/162#issuecomment-6877415766).
Registration: [6877415527](https://github.com/lmmx/mibz2/pull/145#issuecomment-6877415527).
Candidate: **16b750523a8714695e15352ce0ce4c7c29d8ee78**; baseline: **6dec95ba6aaf59d5134e5ab119fd52433e2032ee**.
Reviewed under SPEC pin a2da49638ac54d7763de1c167bd054f67ffdc763. No timing, decoder implementation, merge, or kernel B5 re-review.

## 1. Decision rule

I1 detects an estimated Wikidata gain at one of three nominated thread counts, under the paired log-t model. It does not establish a gain at all thread counts or a familywise 95% claim across the search. Keep that interpretation explicit; I am not adding a new multiplicity gate.

**I2 and I3 do not establish the advertised loss limits.** They reject only when the lower confidence bound exceeds 1.02 or 1.05. Greater uncertainty therefore makes acceptance easier. A ten-pair counterexample, independently reconstructed using the estimator in examples/ab.rs:2512:

- Five ratios 0.9540947299 and five ratios 1.2682178846.
- Geometric mean 1.10; paired log-t 95% interval approximately **[0.9880783, 1.2245993]**.
- Both original tests pass despite an estimated 10% slowdown and an upper bound above 22%. A separate 1-thread Wikidata cell can satisfy I1, so I1 does not remove this loophole. CI on different hardware cannot exclude this i9 loss.

**Targeted replacement:** I2 passes only if every Wikidata/text default-arm cell has upper 95% bound **<=1.02**. I3 passes only if every pattern default-arm cell has upper bound **<=1.05**. Otherwise DEFER: distinguish a demonstrated excessive loss from insufficient precision. Keep ten rounds, no deletion, no retry; wide intervals must not become permission to ship. This retains the nominated 2%/5% practical tolerances. Name I2 “primary loss bounded by 2%,” not literal “no primary loss.” Coordinator decides this single correction; no second Researcher round or new experiment is requested.

C1 is a regression-detection guard, not proof of equivalence. C2 is meaningful only with the policy's full tiers: a >5% through 10% secondary loss requires a primary gain at the **same thread count** at least twice that loss, plus its workload justification. Do not pay a CI loss with an i9 gain. C3's Wikidata estimate is a specific memory justification, not blanket approval for every corpus's 10–50% growth. Apply the >50% ceiling separately. The summary's RSS statistic is the median of child peak RSS measurements, not the maximum observed peak; identify the statistic in adjudication.

The 64 threshold was informed by known traces and previous failures. This is an explicitly adapted candidate, not blind confirmation of the old candidate. Freezing the new rule before fresh timings is legitimate; it cannot retrospectively turn the prior DEFER into a pass. The supplied registration itself acknowledges missing per-block counterfactual timing. No fresh timing evidence was consulted in this review.

## 2. What the recipe measures

At [.just/bench.just:1376](https://github.com/lmmx/mibz2/blob/16b750523a8714695e15352ce0ce4c7c29d8ee78/.just/bench.just#L1376), the nominal design matches the registration: three shared compressed input paths, seven thread counts, ten rounds, four arms = **840 rows**. The new driver runs both binaries; the default arm uses the production parallel API. Same-round ratios are the intended pairing; arm order rotates/reverses. The log-mean, sample log-SD and Student-t interval calculation match the declared estimator. Ten rounds do not perfectly balance its eight-step order cycle.

The final summary uses **old default** as its only reference, correctly producing new-default/old-default for I1–I3. It does not directly print new-workers/new-default; calculate that separately from the same raw paired rows, never by dividing rounded interval endpoints.

Two interpretation limits matter:

1. [ab.rs:1118–1142](https://github.com/lmmx/mibz2/blob/16b750523a8714695e15352ce0ce4c7c29d8ee78/examples/ab.rs#L1118) gives mibz2-workers a window of **2 × plan.workers()**, whereas production default derives its window using SLOTS_PER_THREAD=4. The reported workers/default contrast changes window as well as plan. It may motivate the explicitly separate registration; it does not isolate a causal workers/teams threshold benefit.
2. [parallel.rs:220](https://github.com/lmmx/mibz2/blob/16b750523a8714695e15352ce0ce4c7c29d8ee78/src/parallel.rs#L220) changes **all Plan::Workers**, not intrinsically “below six.” The i9 planner normally selects teams higher up; topology fallback can select Workers at higher counts. Record actual topology/plans when interpreting unchanged-team claims.

The recipe is a sampler, **not a complete implementation of the registration's invalid-run rule**. Summary maps duplicate keys by replacement, pairs intersections, can compute on two pairs, prints byte mismatch without failing, and does not establish the full roster. Its displayed intervals are rounded to three decimals. Before applying any gate, check raw rows: exactly one successful finite-positive-wall record for every corpus/thread/arm/round, full ten-pair sets, no crash/error/termination failure, and matching output size/hash. Missing or duplicate rows and bad output cannot receive a verdict. Use unrounded bounds. This is enforcement of the already registered invalid-run clause, not a second statistical correction or a requirement for new infrastructure.

Input hashes are archived and all arms receive the same paths: this is good within-run pairing. Existing generated inputs and the old worktree are reused by existence, however. The recipe writes the requested old SHA without verifying that cached worktree's actual clean HEAD, and does not reject a dirty new checkout. Execution must establish both actual source identities/cleanliness and identify the hashed corpus artifacts; printed SHA labels alone are not proof. Defaults must remain the registered OLD/ROUNDS/THREADS and hold setting. These are evidence qualifications, not claims that an incorrect run already occurred.

## 3. ×8 retains the F01 production witness

At [cells.rs:1102](https://github.com/lmmx/mibz2/blob/16b750523a8714695e15352ce0ce4c7c29d8ee78/src/cells.rs#L1102), the fixture is still passed through ordinary invert_runs, not forced admission. Independently constructing the BWT column of (a^9 b^9)^3 c and expanding each column byte confirms:

| Expansion | n | 127-capped records r | budget floor(4n/25) | 64r<n |
|---|---:|---:|---:|---|
| ×8 | 440 | 8 | 70 | false |
| ×10 | 550 | 8 | 88 | true |

Thus ×10 would no longer witness production admission; ×8 deliberately keeps that meaning. Source assertions still require admission, output-oracle equality, 57 cells/one short cycle, period55, long-writer passes, spine recopy and repetition. The integration fixture also exercises the crafted block through the decoder and reference comparison. Direct long-writer tests are retained.

This accepts the **fixture design and retained assertions**, not an assertion that I reran Rust/Miri here. The attached check.py/check.json independently verify only the interval counterexample and fixture record/admission arithmetic. Kernel B5 remains outside this assignment and its existing closure is not reopened.

## Evidence and limits

Read the exact candidate's cells.rs, parallel.rs, tests/cells.rs, tests/common files, examples/ab.rs, final recipe, PR383 and the adoption registration; compared the stated delta from PR382. Read docs/team/PERFORMANCE-POLICY.md. Routing audit found no new misroutes; migration digest and old branch head matched before receipt. No benchmark was launched and no speed claim is certified.

Next action: Coordinator resolves the one I2/I3 correction, records the final rule before fresh timings, and adjudicates complete exact-source evidence under it. This review grants no merge/release authority.

