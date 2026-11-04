# Assembly 1: attribution and admissible performance evidence

Trigger: [6639998280](https://github.com/lmmx/mibz2/pull/145#issuecomment-6639998280).
Evidence: [combined archive @ 888eccfc](https://github.com/lmmx/mibz2/tree/888eccfc86d5d771e881b0f87d46a21e2cf1512a/docs/journal/2026-11-04-compact30-combined-build).
**Result:** the approximate 40 M is attributable in the archived class tables, principally to tile setup and outer-loop integration effects. Constant IPC is a useful sensitivity, not a defensible performance forecast. Continue toward a hardware-specific projection and, if it supports a credible win, an unmerged production-integration candidate tested by existing CI. This review does not certify the integration prerequisite already met.

## 1. The residual has identifiable sites

I recomputed all 83 archived class products and sums. They agree with ledger.json:
build −207,720,906; refill +9,059,803; finish group −60,212,114; own total −258,873,217.
Charging added library work 378,882 gives the lower saving 258,494,335. Crediting removed library work 4,454,576 gives the upper 262,948,911. The interval width is not statistical uncertainty.

The counted n1 histogram sums to 2,393,313 nonuniform tiles and 29,357,924 first-segment rows. The 55 listed candidate instructions in the two key classes match their offsets and text in ctor2c-build-false.asm. All archived counted_each values equal their corresponding class frequency. These checks establish internal arithmetic, histogram and sampled listing consistency; I did not decompress/replay raw Callgrind profiles, rerun safety suites, or prove the completeness of the equal-frequency class mapping. Optimiser's independent raw-ledger review remains separate.

Let B,S,D,C be the instruction counts in the B2/scanner/descriptor/combined columns of gate2/paths.txt. Define the additive own-code counterfactual S+D−B. Summing frequency·[C−(S+D−B)] over build+refill gives **−56,391,637**. Finish is handled separately, not counted again as an unexplained integration benefit.

A disjoint decomposition of that interaction:

| Compared with the additive own-code counterfactual | Extra saving (+) / cost (−) |
|---|---:|
| First-segment rows and their refill detours, including row 0/1 work relocated into S | +18,810,351 |
| S-class non-row setup/control | +28,719,756 |
| Outer iteration overhead | +12,015,336 |
| All remaining build/refill interactions | −3,153,806 |
| **Total** | **+56,391,637** |

Derivation, not a fitted residual:

- The unrolled position classes total 222,418,439 instructions. Rows 0/1 plus the associated index recomputation contribute 20·2,393,313 = 47,866,260 inside S; the 16 detours contribute 16,443,798. Total: **286,728,497**.
- Descriptor-only rows+detours cost 10·29,357,924 + 10·996,634 = **303,545,580**. Difference **16,817,083**, reproducing the Coordinator's 16.8 M.
- The additive counterfactual charges 12, not 10, per detour: scanner adds two relative to B2. Its row/detour difference is therefore **18,810,351**. Do not mix these comparison baselines.
- S has B=38, scanner=37, descriptor=37, combined=44. Removing the already-accounted 20 row/index instructions leaves 24 combined non-row instructions against 37+37−38=36. Thus **12·2,393,313 = 28,719,756**.
- Outer iterations have B=13, scanner=11, descriptor=17, combined=11. The additive count is 15; combined avoids four: **4·3,003,834 = 12,015,336**. In particular it does not inherit the descriptor's four-instruction per-iteration cost.

Those last two terms total **40,735,092**. They locate the approximate 40 M without inventing a new optimization or allocating it to cache behavior. The unrolling affects setup/control as well as per-row instructions; describing its benefit only as 10→9 per row misses that.

This comparison intentionally uses raw additive own-code counts. Applying overlap O increases the non-double-counted predicted cost; it does not create additional observed instructions removed. Do not add these interaction terms to the already measured combined saving, nor mistake the translated minimum F for an isolated achieved-saving estimate. The v3-specific terms, library conventions and overlap adjustments must retain their own comparison bases. audit.json includes every nonzero interaction row so the exact sum can be reproduced.

**Finding (1):** attribution is now concrete at the archived-table level. “40 M unattributed” need not remain the working explanation. Independent raw-profile coverage/replay still decides whether these counted changes are trustworthy; arithmetic reconciliation alone is not Gate 3 certification.

## 2. What constant IPC actually assumes

With fixed frequency, time is I/IPC. Let q = I_combined/I_v2 and k = IPC_combined/IPC_v2. Then the hypothetical build ratio is R_new = R_old·q/k. Setting k=1 is an assumption, not an inference from I.

Using the conservative lower instruction saving gives q=0.8003032; the inclusive simulator difference gives q=0.7968619. The latter includes the removed rep-movsb library convention; neither is a hardware retired-instruction measurement. On the *INVALID*, historical v2 ratios:

| Hypothesis | CPU ratio | Completion ratio |
|---|---:|---:|
| Constant IPC, conservative saving | 0.86928 | 0.89183 |
| Constant IPC, inclusive difference | 0.86555 | 0.88800 |
| Half proportional time transfer, conservative saving | 0.97774 | 1.00310 |
| Half proportional time transfer, inclusive difference | 0.97587 | 1.00118 |

“Half transfer” means time ratio 1−s/2, not half IPC. These are two different model parameters.

Define transfer f through R_new = R_old(1−fs). The old point ratios need approximately **8.7–8.9% transfer for CPU** and **41.7–42.4% for completion** to reach 1.067/1.02. Stressing with the invalid capture's upper ratios gives **12.7–12.9%** and **50.6–51.5%**, respectively. At half transfer the upper completion sensitivity is **1.02145–1.02341**, still above 1.02. These transformed historical upper values are not a new confidence interval.

Equivalently, the point-ratio constant-frequency model permits k down to about 0.811–0.815 for CPU but only 0.871–0.874 for completion. That is a sensitivity threshold, not a measured safe margin.

Reasons k and transfer can change: removal of cheap/overlapped instructions; altered branch/front-end behavior; a larger build body; new refill spills; cache and memory bandwidth; unchanged logical scatter accesses; changed instruction mix and worker competition. The archive actually reports fewer explicit reads/writes and finish stack operations, so “memory-bound scatter untouched” should mean the logical scatter structure, not that all memory work is unchanged. No measured IPC or stall evidence here establishes memory-bound dominance.

Most importantly, synchronized completion is a maximum, not aggregate CPU. Recompute each archived step as max_j(start_j+duration_j−d_j)−min_j(start_j), with path-specific reductions for every worker and winner switching. The old 33-cycle-per-finish number is a solved requirement under assumptions, not a measured chain cost. The current total instruction reduction alone cannot supply d_j. State correlations and uncertainty; never label modeled time as a measured saving.

**Finding (2):** strong structural progress and a credible hypothesis; no numerical economic gap closure is established. Constant IPC is acceptable as one sensitivity case, not as the sole justification for “can win on CI.”

## 3. Cheapest admissible route: model first, existing CI second

Policy read at charter 2210e4457b383e4566df99eb881b4ca0401c4cd3; entry resolved at channel cb76e75d8d0112fa4004587a2f896ef690129f0c. The combined branch carries an older charter; it is not the source for current Coordinator delegation. Permanent stewardship and actual bench workflow read at master c4b69de363dba4485567142a0d9cec4034fba6e2.

The entry permits an integration PR when the combined evidence and cost model project a win on CI hardware. It does not require an already measured production win before opening that PR; the merge is gated by CI. Conversely, a generic i9-derived constant-IPC product is not a CI-hardware projection.

1. **Lowest-cost next evidence:** finish the independent ledger review; use existing per-block path counts/listings and archived worker records to expose critical-worker sensitivity, scalar/fragment versus bulk contributions, and winner switching. Use the conservative instruction end and explicit adverse-transfer scenarios. Inspect the CI target and integration codegen; compiler-dependent unrolling may change. This is untimed analysis, not a new local capture. Report what would falsify the model and distinguish all model inputs from hardware observations.
2. **Project the complete decoder on the CI topology.** The current workflow runs 1, 2 and runner-CPU-count threads, with spread, on ubuntu-latest; its comments describe two physical cores with SMT. Use recorded env.json for the actual runner, not a permanent core-count assumption. The four-worker construction limits and old walk figures do not automatically transfer to this topology or the one-thread case. Account for constructor, walk, unchanged decoding stages and memory interactions. Amdahl: if only a time fraction a changes by ratio r, end-to-end ratio is (1−a)+ar, under that partition's assumptions.
3. **If that projection credibly wins and composition/safety findings are closed:** the Coordinator can make the intended default-kernel replacement on an unmerged integration branch/PR under its existing delegation. No feature flag, merge or release is needed to obtain the measurement. The existing CI builds exact PR head and merge base and runs paired end-to-end A/B; parity, green CI, beneficial 95% interval and no unacceptable regression govern any later merge at the measured SHA.
4. **If the projection cannot credibly win:** remain in research, improve the mechanistic model or make a targeted bounded cut such as corrected C7. If direct constructor timing is indispensable and the existing measurement cannot supply it, that is a separate reserved measurement/harness decision for Louis—not permission to improvise a local timer or modify CI.

I inspected .github/workflows/bench.yml at the combined pin and current master; contents match. It builds examples/ab, windows and phases, not c30walk. The paired arms exercise the library decoder; the stage diagnostics are single-threaded. Dispatching that workflow on an example-only research ref therefore does **not** measure the combined constructor. Its existing result also cannot certify the exact isolated four-worker construction thresholds by substituting a stage metric or end-to-end result. Preserve the model-to-integration prerequisite and the separate production merge gate.

**Finding (4b):** this is not “model or CI.” The model is the inexpensive admission argument; existing CI is the performance authority for the integrated decoder. No reason yet to purchase a harness change or use the i9. No unconditional integration recommendation from the current aggregate model alone.

## 4. Disposition and limits

Continue: the attributed reductions support a materially better structural trajectory. No evidence justifies stopping the line, but “timing, not instructions” is premature if it means dropping dependency/caller/critical-worker analysis. Redirect if the hardware-specific model shows savings concentrated off the relevant paths or sustained IPC loss erases the margin. Record misses honestly; do not relax gates.

Prior involvement: Researcher reviewed v3/scanner/counter/C6 work, assessed C7–C10, and drafted the entry navigation. This review is informed, not blind. No implementation, compilation, benchmark, workflow dispatch, merge, release or authority change was performed. Guardian composition/gate disposition and Optimiser's canonical ledger remain separate.

## 5. D013 received during this assessment

[Guardian comment 6640069360](https://github.com/lmmx/mibz2/pull/145#issuecomment-6640069360), decision at 88bc570db02d8e617652ea56e081e4b4c6eae0c4, places a narrow HOLD on final combined qualification pending the signed bridge to raw-profile totals. Honoured. This report supplies a disjoint archived-table bridge, not raw-profile closure, and does not lift that HOLD.

The Guardian's S estimate 13S=31,113,069 compares scanner overhead 37 with combined non-row 24. My 12S=28,719,756 compares raw additive overhead 37+37−38=36 with 24. Their difference is S=2,393,313, exactly the existing shared-removal term. They are compatible counterfactuals, not competing observed totals. If allocating existing O to S, use 13S in the overlap-corrected bridge and do not subtract/add O again elsewhere. The corresponding build+refill interaction gain becomes 56,391,637+2,393,313=58,784,950. Further proposed O=3,654 is not used.

No demand for a new minimum or equality with isolated savings is warranted; D013 explicitly makes neither demand. Raw-profile replay and complete changed-site attribution remain the Optimiser/Coordinator closure evidence already requested.
