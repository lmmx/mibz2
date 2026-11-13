# Cells measurement handoff — owner priority change, 13 November 2026

Direct owner instruction prioritises the existing matched Cells live diagnostic and one evidence-led implementation change over further housekeeping/readiness studies. Coordinator remains sole orchestrator. Pending inbox6848164049 concerns a different onecore dossier: not executed or marked complete; ask Coordinator to reconcile its deferral under this newer instruction.

## Verified starting point

373 remains unmerged at8329f0c702a1c5598e39e7f2413db55a12ce8884. Fresh exact-head Actions all SUCCESS: Test43393348451, packaging43393348433, Occupancy43393349525, Benchmark43393348524, preparation43393348575; hosted legacy SUCCESS. Both macOS builds passed. No new lint or general review pass required.

Researcher6821626797 required one R1 resource-retention correction, found no new B5 blocker; Coordinator6822234208 accepted the correction after three focused mocked tests. No second full review is outstanding. Independent F01 acceptance stands. Source/compiler/experiment are still frozen; changing future performance code requires its own safety evidence.

The current373 benchmark compares ordinary decoder head/base on EPYC, not the requested i9 four-arm experiment. It reports all output hashes equal and all wall intervals spanning zero (e.g. Wikidata1t0.433→0.431s, −1.1%[−5.4,+3.4]; pattern2t+6.0%[−10.9,+26.0]). These are not Cells speedup measurements.

Read the repository source of the published technical companion,365 description/source evidence,367 HANDOFF and373 PLAN. The old research0.815/0.827/0.818 inversion ratios and CI whole-decoder1.000 are different hardware/conditioning/implementations.84.117% admitted rows makes simple admission dilution inadequate. No measured explanation or writer bottleneck yet.

## Concrete execution defect fixed, not a new experiment

Actual run.sh emits repository-root cells-live-<stamp>.tar.gz; actual just submit discovers only target/*/*.tar.gz. Reproduced using real Just submit/recipe and bundler with fake children/external submission: zero discovered bundles after either successful work or child exit19. This would spend the full run budget and lose automatic submission.

Isolated PR376, base373/8329f0c, head **5f9b4fade9e2bfaa061b625a0a55f7b3f8d9d34a**:
- create target/cells-live-path;
- put attempt at target/cells-live-path/<UTC stamp>;
- add one regression covering success and failure through the actual submission path.

All14 existing+new Python tests pass/no skips with actual Just; source shell syntax and scoped lint pass.1572 other tracked files identical: no PLAN, driver, schedule, estimator, build, hooks, production, unsafe, lock or source-pin change. Before-failure and after-pass logs retained. Actual run remains120 measured+9untimed/oracle; no warm20+20.

PR376 hosted legacy, packaging43396306222, preparation43396306200 and Benchmark43396306235 pass. Test43396306206 fails in Rust job131787940907: the actual log checks out merge6fb3be5 of376 into the old373 branch and reports examples/mtf_two_level.rs:273 needless_range_loop. That branch lacks the already-integrated375 correction; master8138e014 contains the corrected iterator. This is a demonstrated inherited stacked-base dependency, not a new lint task or an inferred infrastructure failure. Other Test jobs pass. Preserve this failure; after authorised373 integration, retarget376 to master and inspect its normal changed-base checks. Do not reimplement375 or retry the same failed check. Full actual job log and final Actions snapshot retained here.

No manually dispatched benchmark/retry or workstation execution. This small follow-up requires ordinary targeted review/integration; it does not reopen the harness's full independent review.

## Exact next execution prerequisites

1. Owner/authorised deliberate Coordinator integration of373 and submission fix376 after corresponding checks; no merge permission inferred from isolated-performance-change authority.
2. Actual i9 preflight: current CPU model/topology, allowed affinity and sibling groups; absolute directory containing the exact three compressed corpora. No archived CPU/path is used. This cloud executor has no configured remote i9 tool/outbound identity. User has been asked whether they will run the read-only preflight or provide an approved connection.
3. Select/approve one allowed logical CPU and bind that CPU plus verified absolute corpus directory in an argument-free wrapper of the existing parameterised recipe. No selecting defaults at runtime; no measurement-protocol change. Until machine facts arrive, no concrete runnable queue line can honestly be frozen.
4. Coordinator check exact recipe/source CI and set appropriate queue/ready state under PROTOCOL9, then owner explicitly authorises/runs the one frozen experiment. Current master queue empty; no ready/hold tag for bench-cells-live-path observed. Historical ready/bench-cells-vs-prod is another experiment and grants nothing here.

The supplied i9-preflight.py only reads topology/affinity and compressed-file hashes; no BWT decode, clock sampling, CPU selection, queue/signal, build, data generation or network. Syntax/help checked locally; actual workstation facts have NOT been collected. Run with an actual absolute corpus directory:
python3 i9-preflight.py /ACTUAL/ABSOLUTE/CORPUS/DIRECTORY
The placeholder is not a suggested/historical binding. Return JSON including mismatches. Read-only hashing does not consume any of the9 decoder/oracle prechecks.

## Work after the one authorised run

Use off-build paired complete-decode wall/CPU/RSS first; retain all120 raw measured records and9 checks. Compute planned geometric ratios and95% log-t intervals, then compare off/on inflation and categories. Charge exclusive scanner/front/order/induction/writer/fallback/finish/remaining-reader plus process-minus-reader time without summing inclusive inversion again. On-build attribution is not causal off-build savings. If inversion gains persist but total does not, quantify other stages; if the port loses inside inversion, locate largest actionable stage before touching it.

Then implement one isolated change supported by those results. General-purpose versus specialised copies remains a candidate, not the selected culprit. Preserve original-row phase, periodic cycles, RLE1/CRC, fallback and bounds. A subsequent finite paired A/B budget and hardware matching need separate authorisation/freeze; correctness tests and operation counts alone will not qualify a speedup.

No new performance data, no optimisation claim, no merge/release/default/gate/queue/Run action. This is a durable checkpoint at the actual operational boundary, not completion of the owner's requested performance objective.

