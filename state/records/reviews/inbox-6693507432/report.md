# Three-team policy design — inbox 6693507432

2026-11-06. Proposal, not implementation or adoption.
[Assignment](https://github.com/lmmx/mibz2/pull/160#issuecomment-6693507432).
[Evidence PR #220](https://github.com/lmmx/mibz2/pull/220), reviewed at **544462881497fc957fe17c35ae7508ca1bac23d5**.
Authority: SPEC / SPEC.OPTIMIZER / RULES / PROTOCOL at **a6a703cefcc40ac4115803f44a8a29ad76a3a7ef**.

## Decision

Choose **three teams of three**, not automatic escalation to three teams of four. Restrict the first policy candidate to the **10 physical cores / 20 MiB L3 topology bucket**, at requests of **10 or more decoding threads**. Leave every other branch of the current policy unchanged.

This is an empirical exception, not a new universal cache formula. It chooses a qualified plan with lower CPU cost and no demonstrated persistent wall disadvantage. The registration qualifies both plans against 4×2; it does not rank them against each other. No cache, SMT or table-count causal claim is made.

**No production PR was opened.** The brief's request for a PR link conflicts with the pinned role's write boundary and prohibition on implementation. The deliverable is this complete policy/evidence design for the Coordinator's PR; #220 above is the evidence PR, not a new policy PR. No state-branch PR is created.

## Evidence and its exact boundary

The pinned journal records the two authorised handoffs, distinct runs, unchanged executable source from freeze, byte parity, complete 320-row grids, controls not flagged, exact plans, both benefit predicates and both resource limits passing twice. That supports the journal's recorded qualification. It is not fresh independent reproduction here.

The recursive tree at 544462881497fc957fe17c35ae7508ca1bac23d5 contains the three-table journal but **no three-table raw runs or confirmation bundles**. #220's conversation supplies CI output, not those workstation rows. Accordingly:
- no raw 640-row replay or independent archive-digest verification is claimed;
- no head-to-head confidence interval is inferred from marginal intervals;
- no missing artifact is invented or silently substituted with #219's screen;
- the Coordinator should attach durable, exact bundle/raw/report references for the policy PR. Preserve the full bundle hashes, tested source pins and frozen recipe identity.

The reviewed `src/parallel.rs` blob is **7ea7a69e4bb025711361a9e960bba1398b07ac9d**, identical at evidence and governing-spec pins. The existing policy uses min(request, physical cores), enters teams at six usable cores, fixes k=2 and caps teams at round(0.7×L3/3.6e6), default cap4 when cache is unknown. Its L3 reader is cpu0's sysfs cache, not an affinity-scoped shared-cache discovery mechanism. Do not quietly describe that as the cache available to this process.

| Plan | Run1 speed / old reference | Run2 speed / old reference | CPU / reference | Peak RSS / reference |
|---|---:|---:|---:|---:|
| 3×3 at requested10 | 1.076 | 1.087 | 0.90 / 0.89 | 0.82 / 0.82 |
| 3×4 at requested12 | 1.090 | 1.086 | 1.04 / 1.04 | 0.82 / 0.83 |

From these rounded reported values alone, 3×4/3×3 speed is approximately **1.0130 then 0.9991**; CPU is **1.1556 then 1.1685**. These quotients are descriptive, not a new registered comparison or significance test. Thus do not spend three extra decoding threads automatically for an unestablished wall advantage. This is a resource-aware engineering preference, not statistical equivalence or Pareto dominance.

3×3's reported speed ratios imply **7.06% and 8.00% less wall time** than the eight-thread reference. They close that much of the baseline time, not that much of a known optimum gap. No remaining optimum has been measured. Expected gains were 3–7% in speed for3×3 and4–9% for3×4; results lie near/above those ranges. The result advances policy qualification, not mechanism identification.

## Exact proposed selection table

Let T=max(requested threads,1). Use the existing physical-core and L3 discovery; failure leaves the existing fallback. Before the old teams selection, select Teams{teams:3,k:3} **only if** physical cores==10, reported L3 bytes==20×1048576, and T>=10. Otherwise apply the current policy exactly.

| Input on10-core/20-MiB bucket | Candidate | Old | Default reader window |
|---|---|---|---|
| T=0..5 | Existing Workers(max(T,1)) | Same | Unchanged |
| T=6,7 | 3×2 | Same |12 |
| T=8,9 | 4×2 | Same |16 |
| T=10,11 | 3×3 |4×2 |12 |
| T=12..20 and greater |3×3 |4×2 |12 |
| Other physical-core/cache tuple, either missing |Existing policy |Same |Unchanged |

Nine decoding threads fit within every changed request. The request is a budget, not a promise to spawn T workers; scanner/reader/feeder support threads remain outside that accounting as before. Keeping T=9 unchanged avoids moving an untested request boundary in this first patch. T=8 cannot run3×3 without violating the budget; merely cutting it to3×2 is a different, unqualified plan. Do not do either.

No automatic3×4 at T>=12. Preserve explicit team4 experiment arms for future comparisons; their availability is not a default recommendation.

Keep the existing window function: for default slots=4T it yields4×teams=12, reproducing the qualified shape. For custom slot counts use the existing saturating formula and minimum-one-slot/team behavior, not a new unconditional12-slot override. Add checks for small slots, zero handling at the public boundary and usize::MAX saturation. Changing the plan also changes custom-window results; test those semantics explicitly.

**Generalisation disclosed.**
- Direct qualified cell:3×3 with request10, i9-10900K, Wikidata level9, default window.
- Requests11,12,16,20 and higher map to the same decoding plan and default window; still new API/policy cases, not individually qualified workstation comparisons.
- Other CPUs with exactly10 allowed physical cores and20MiB detected L3 match the bucket. That is unmeasured transfer, not an i9 model check. The existing API has no CPU-model identity. Include this in the PR description; if unmeasured transfer is unacceptable, keep the candidate experimental until explicit model/cache-domain gating is designed, rather than calling this bucket i9-only.
- Other compression levels/corpora and custom windows are unqualified performance domains. The planner has no block-level argument and must not claim it adapts to block size.
- Unknown L3, multi-domain/cpuset mismatches, other core counts and cache sizes get no new formula. Do not replace0.7 with0.5 globally or use floor(L3/table) as if measured.
- Current discovery's cpu0 cache limitation remains a named limitation. Fixing topology discovery would be a separate change with its own affected-domain analysis.

## PR contents for the Coordinator

Proposed title: **Prefer three cooperative teams on the measured ten-core topology**.

Keep the change to the plan rule, its documentation and focused tests. No walker, constructor, allocator, affinity, input, statistical method or CI threshold change. Preserve original #220 results. Pin the actual production base and candidate full SHA when the Coordinator creates it; 544462881497fc957fe17c35ae7508ca1bac23d5 is evidence, not an invented future measured head.

The PR description should lead with the concrete trigger: on a10-core/20-MiB topology, requests>=10 previously selected4×2/window16; candidate selects3×3/window12. Record qualifications, extrapolations and admission gap below. Archive matched whole-decoder counts (build/walk/rest) under RULES S2 if this default-path change falls under the kernel-change requirement; unchanged per-block kernels are not a substitute for measuring altered team/segment/control costs. Charge support threads and total process CPU; do not count lower thread count as instruction saving.

## Evidence fixed before any new timing

### Untimed CI coverage of changed code

The current runner does not reach this default branch (actual CPU/core topology must be recorded, not inferred from the runner label). These checks stand in for **execution and correctness coverage**, never target-machine speed evidence:

1. Pure plan table tests with injected phys/L3 values: every row above, T=9/10 boundary,6-core threshold, unknown topology/cache, cache values just below/at/above20MiB, phys9/10/11, budgets0/1/8/9/10/11/12/16/20/large, and existing16-core/32-MiB behavior. Assert no plan exceeds max(T,1).
2. Force old4×2 and candidate3×3 through the existing planned decoder in **untimed** differential/parity tests, even on small CI. Exercise public default selection through a test-only topology injection; explicit planned-arm tests alone do not cover dispatch.
3. Byte-exact corpus tests against established oracle, levels1–9, tiny/empty/truncated/corrupted inputs, repeated/periodic/multi-block/concatenated streams, CRC errors and long RLE expansion. Check error rollback, early close/cancellation, helper spawn failure/fallback and panic/join; unchanged unsafe code still has newly selected paths.
4. Exercise custom slots below teams, default4T, large/saturating slots; verify ordering, progress and no lost wake/deadlock. Require existing safety/differential/fuzz/mutation suites applicable to cooperative dispatch. A timeout is partial coverage, not PASS. If implementation introduces unsafe code, full B5 argument and independent adversarial review remain mandatory.

No performance inference from deliberately oversubscribed forced-plan CI tests.

### Ordinary CI bench and admission gap

Run the **unchanged** paired head/base CI bench and existing parity/safety checks on the exact candidate head. Keep existing corpora,1/2/nproc grid, estimand, intervals, repeat/multiplicity policy and acceptance rule. Record the actual selected plan for each cell. On current runners expect the old worker path on both builds; any observed change needs explanation.

**This is regression evidence, not benefit evidence for the new branch.** A CI job being green is not STEWARDSHIP's positive end-to-end95%-interval gate. The current rule cannot be satisfied scientifically by mining an unrelated noise win on an unchanged path. Do not substitute workstation qualification, unit tests or forced oversubscribed timings for that gate.

Consequently the design can proceed, but **production admission is unresolved under the current CI coverage and rule**. Coordinator must escalate the exact coverage/acceptance decision under C6/D4: a maintainer-approved representative CI grid/runner coverage or an explicitly approved policy-admission rule. New infrastructure additionally needs D6. Until then, leave the policy candidate unmerged. This report changes neither rule nor infrastructure.

### Targeted regression proposal, requiring Louis's exact-recipe authorisation

Freeze base/head, compiler/flags, corpus hashes, output oracle, selected plan/window, requested threads, placement and environment before capture. Compare actual **new default versus old default at the same requested count**, not only a forced3×3 arm at10 against old default8. Qualification of #220's different estimand cannot answer every same-request question.

Proposed affected request grid:10,11,12,16,20 plus unchanged controls8,9. Cover the existing seven corpora (wikidata,onebyte,pattern,random,records,runs,text) and compression-level correctness boundaries; record default and constrained-window cases in correctness tests separately. Any expansion of the timing workload/thread grid/repeat policy is a **proposal requiring C6 approval**, not delegated harness mechanics.

For a concrete registration to consider: one20-pair run and one same-build20-pair confirmation, no pooling/switching, fixed alternating/rotated order. Primary effect new/old wall at10 on level9 Wikidata: speed ratio>=1.03 and lower two-sided95% bound>1, CPU<=1.5 and RSS<=1.15 ratios of medians, all bytes/plans/rounds valid. Other changed timing cells: predeclared non-inferiority margin2% wall harm, with simultaneous one-sided familywise95% upper bounds (Bonferroni over the fixed regression family) <=1.02 in both runs; insufficient precision is inconclusive, not PASS. This is stricter new proposed admission evidence, not a reinterpretation of #220. Numerical margin, family and repeat policy require approval before use. If authorisation differs, freeze that separately before results; do not tune to obtained intervals.

Louis running and handing over one frozen recipe authorises that run only (C5). No recipe is executed or dispatched here. No performance guarantee is made for unseen same-bucket CPUs; broaden the policy only after evidence on those targets.

## Opportunity cost and prior record

The confirmed policy now has a direct observed whole-decoder lead; C23 ll reuse had only a memory-capacity price, C24 needed a growth/page census, C15 and C17 retain their registered failures. Thus policy preparation/admission resolution outranks speculative buffer or compact-table rebuilding. If the CI admission route cannot be resolved, resume a separately priced broadly applicable candidate rather than spin indefinitely on unchanged CI; no automatic reopening of a closed claim.

Correction carried forward: the old25% figure equalises aggregate footprint of four and three teams; it is **not necessary for cache fit or speed gain**. Researcher6683945162 supplied a2.90963% counterexample within our sensitivity model; Coordinator6683965024 retracted the fixed reopening threshold. No fixed25% gate survives. This preserves the algebra and corrects its overgeneralisation; it predicts no hardware gain.

Four C3 questions: expected3–7%/4–9% speed; observed two confirmed favorable cells; about7–8% baseline wall removed by the preferred plan, optimum gap unknown; mechanism remains unisolated. Next marginal value is testing actual dispatch/regressions and resolving admission, not another screen or another unpriced cache hypothesis.

## Checks performed here

Migration previous head equals e6eea5889432aef28f4d9e450cee87dd5bf7bacf; exact MANIFEST SHA256 matches86d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde. Receipt committed before work. Pinned coord.py routing audit evaluated on connector-supplied comment headers across all five channels: only its five already-resolved historical misroutes; REVIEW route is145. No unresolved routing issue. Its verify subcommand hard-codes FROM:coordinator, so publication checking for this role uses its misrouted/check logic plus explicit optimizer/header/ID checks instead of falsifying sender identity.

No candidate implemented, compiled or timed; no new independent raw-run replay or safety certification. Design finished; implementation, production PR creation and admission remain Coordinator work.
