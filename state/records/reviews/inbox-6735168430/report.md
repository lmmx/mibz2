# C29 remains first; a CI lead would not yet qualify the default

Brief [6735168430](https://github.com/lmmx/mibz2/pull/160#issuecomment-6735168430). Reviewed source/registration **1197bbcced86eae74c6d99eb8096b5c816a834d7**, under specs **a4f97714cc14366984015869d9d094d89488ac57**. D060 read at **471aabefdf20848d93d8baeff778ed1b9f67fb07**, state/decisions.md.

**C29 stays first for the next bounded performance investigation, not for immediate adoption.** Run42582708226 stays terminal **VALID, UNJUDGED (FALLBACK)**. Its reported .9700 is evidence motivating investigation, not a LEAD, proof of available CPU capacity, or a reusable sample for the next verdict. No repair of today's collector changes yesterday's admission.

## 1. What the result changes—and what it does not

Expected: the register-list/short-store treatment should retain its research-worker benefit after moving into the shipped pipeline, without the reach-counter atomic. Reported: wall .9700 [.9647,.9753] and CPU .9737 [.9704,.9770] at9 threads; at3, wall .9714 [.9504,.9929]. The new whole-decode reading is directionally consistent with the prior matched control, but the required quota fact is absent. **Qualified performance gap closed: zero.** The implementation/measurement gap narrowed: the treatment now runs through the actual shipped worker, in one binary, instead of the copied worker.

The workflow API confirms the requested full head and a completed, failed workflow. Failure alone is not a speed verdict; the channel reports reporter exit3/unknown quota. I read the pinned source, reporter and registration and the Coordinator's reported figures; I did **not** retrieve/hash/recompute this run's80 raw records. Numeric figures below are explicitly reported or hypothetical, not independently replayed measurements.

Priority:
1. Close the collector's evidenced defect untimed; then the one separately reviewed, frozen follow-up if admitted.
2. Close D060's targeted default-on safety/reach obligations in parallel with that preparation; they require no performance sampling.
3. Fold adoption-relevant layout controls into C29's transfer design. The separate B fallback-layout investigation retains information value but does not need to block a narrowly scoped C29 diagnostic.
4. C43 stays lower: it redesigns construction and spends a tight grouping/gather budget; C29 removes established front work with less architectural change.

What changes this order: a valid judged STOP or weak/inconclusive follow-up reduces C29's marginal value; failure to establish admission suspends timing on that configuration; switched-path safety defects hold affected work; a robust target default regression blocks adoption. Conversely, credible matched whole-decoder evidence for an alternative can outrank C29. No need to prove C29 impossible before redirecting. The service-core and leader workstation results now have unresolved attribution/clock questions in the channel; they do not supply a clean competing production gain in this review.

The Researcher's later REVIEW6735626248 and Coordinator LOG6735738288 narrow the next attempt: preserve the original, untimed hierarchy/coverage validation first, one new attempt, suspend after another INVALID/UNJUDGED. QUEUED6736130294 reports #288 preflight PASS but still awaits review/freeze. I neither re-audit that collector nor treat its report as dispatch permission.

## 2. Transfer is three separate claims

### Shipped worker is not shipped entry selection

At the reviewed pin:
- PIPELINED_LEVERS.register_list is false.
- StreamDecoder::new calls Plan::for_threads and defaulted; only exact PIPELINED_PLAN (3 teams of3) selects pipelined_levers with those defaults.
- BzReader::parallel delegates through parallel_with to StreamDecoder::new; decode_parallel also calls StreamDecoder::new.
- The explicit diagnostic arm bypasses default planning with parallel_pipelined_levers and sets register_list=true.
- The production special case requires10 physical cores,20MiB L3 and requested threads>=10. Thus **“9 threads” in the forced CI arm is not proof that the public default at9 selects that plan.** The i9's10 requested threads and9 decode-team threads must remain distinct in records.

An adoption candidate must demonstrate actual C29 kernel selection, not merely byte equality or a selected-attempt counter, through all three public constructors at3x3. Force the plan for portable untimed tests, then separately demonstrate natural planner reach on the target topology. Keep neighbouring/nonchanged plans and scalar fallback controls. Tests use feature-gated reach instrumentation; timed builds do not. The same selected call must reach both initial and prefetched stages.

### Oversubscribed CI is not the i9

Nine team threads on four allocated CPUs can benefit from reducing total CPU demand or scheduling pressure even if a larger target is bottlenecked by scanner supply, ordered retirement, memory traffic or another stage. The reported CPU reduction is encouraging; it does not distinguish those mechanisms, and CPU time is not frequency-normalised cycles.

For a simple serial critical-path model, let f be the fraction of baseline elapsed time exposed to the changed front work and r its local cost ratio. Then whole time ratio is 1−f(1−r), before induced costs. A .970 whole ratio demands f(1−r)=.030. Even eliminating the work needs f>=.030; if f=.10, the local reduction must be30%. In an overlapping pipeline, aggregate front CPU fraction is NOT f: faster front work only saves wall while it shortens the critical worker's completion or releases a binding shared resource. Charge dispatch, cache/overstore traffic and any moved bottleneck.

Use the exact proposed default-on candidate against the then-current shipped baseline on Louis's i9, via an independently frozen C5 recipe and his submission. Keep the natural10-thread/default plan and full read/decode/CRC/output cost. Evidence from forced3x3 on CI does not waive this. Any new thread grid/workload/repeat or acceptance policy is C6-reserved. Instrumentation can record machine state; deleting slow rounds or retrying until a clock state is favourable is not delegated.

### One-build mechanism contrast is not a two-build release contrast

Within one diagnostic binary, shared code placement is fixed, but the two kernels have different addresses, code bodies and possibly stack footprints. Common binary does not cancel treatment-by-layout interaction. The release comparison adds another change: default selection and compiler/linker context, including any dead-code elimination. Keep both contrasts:
- runtime on/off in a matched diagnostic build, to price selection of C29;
- actual candidate-default versus baseline-default release artifacts, to price shipping the change.

The current PR benches at1/2/nproc on the4-CPU runner principally measure paths where C29 is off. Their mixed flags cannot qualify the changed3x3 path, and a judged9-thread diagnostic cannot erase their regressions.

## 3. Off-path layout: the minimum discriminating crossing

The API compare from7ed5b29349e4281247dc77ea9391bc5f71317836 to1197bbcc lists workflow, bench/reporting, journal and examples/ab.rs changes, **no src/**. That substantiates the brief's source-stability premise, not binary identity. The frozen journal documents identical normalised hot instructions with shifted addresses in earlier builds and a limited placebo. It explicitly keeps off-path adoption unresolved. Different hosts, environments and sampling also changed; the signs alone do not prove a pure layout cause.

Propose a compact factorial design, frozen before readings:

| Axis | Required comparison | Confound addressed |
|---|---|---|
| Semantics | old selected / C29 selected with matched dispatch, plus unchanged-path control | useful kernel work versus generic binary movement |
| Build/layout | common harness at baseline and candidate contexts; at least one independently chosen perturbation shared by both semantics | semantic effect versus favourable single placement |
| Entry | explicit lever path and actual public default release path | harness benefit versus shipped selection/compiler context |
| Environment | admitted CI setting and the target workstation, kept separate | oversubscription/topology transfer |
| Data geometry | matched capacities/reset; arm/set/order crossed where buffers differ; actual addresses logged | persistent backing-set/stack/heap effects |

Do not demand every Cartesian product or declare a new official grid here. First establish the public-path mapping and emitted-code fingerprints untimed. A bounded adoption design can combine the semantic/layout diagnostic with the target campaign; label each contrast and preserve the existing primary. If additional layouts become repeated inferential tests, their repeat/multiplicity treatment needs C6 approval before execution.

For each layout l, inspect log(T_on,l/T_off,l); compare these contrasts across l. Independently inspect the off-path old/new ratio. Same-direction absolute movement of both arms does not necessarily bias the ratio; changed semantic ratios show interaction. An identical-kernel A/A′ control crossing sets/order helps detect residual harness effects. A never-called padding function is a useful stressor, not proof that release layout is harmless. Never search padding variants and retain the fastest, nor average away a release artifact's regression. Archive binary/code hashes, hot-entry/loop offsets, compiler/link flags, actual frame/data geometry and allocation/reset policy.

Do **not** transplant B's alleged4KiB mechanism into C29. Instruction-fetch placement and data low-bit aliasing are different hypotheses; earlier review6734171410 already rejects assuming a “clean”4KiB set. Fixed4096 alignment alone is not adoption protection.

A deployment cost illustration, not a new gate: if exposed use saves3% and unaffected use instead regresses1.1%, then a baseline-time-weighted exposed fraction p must exceed .011/(.030+.011)=**26.83%** just to break even. With2.4% off-path regression it is44.44%. These reported cells are not estimates of a universal regression, and such weighted arithmetic is not the bench's geometric estimand. It shows why a small exposed gain cannot buy unspecified off-path harm without a workload model.

## 4. D060 minimum, priced in campaigns and decoder executions

D060 does not demand an arbitrary number of favourable reruns. It demands distinct evidence obligations. Cheapest route is to combine compatible obligations on one exact candidate, not repeat the entire old mutation campaign.

**Before timing the default-on candidate:** switched held-prior/current/next panic/retirement/reuse tests; partial-front construction unwind and subsequent reuse; exact default-constructor reach at3x3 plus nonchanged controls; current-source B5/targeted Miri and independent review. Reuse unchanged kernel evidence only after blob/proof-scope reconciliation. A suite finishing with relevant tests filtered out is not coverage. These are untimed correctness runs, not timed performance repeats. No fixed test count substitutes for the named states.

**Beyond a hypothetical judged LEAD, two distinct performance evidence packages remain at minimum:**
1. One separately registered **target-workstation adoption comparison**, using actual defaults and the exact candidate/baseline. Louis runs it. Entry-API equality/reach should mostly be discharged untimed, not by three duplicate timing campaigns.
2. One **normal CI adoption bench** on that exact integration candidate against its pinned current baseline, retaining the standing corpus/grid/statistics and all off-path cells. This covers release regressions even when it does not exercise3x3.

These are a planning lower bound, not a promise two samples will qualify. A material layout objection can require a bounded diagnostic within the target package or one additional approved campaign; failure yields a finding, not permission for automatic repeats. The target package needs enough treatment/layout separation to answer its declared question. No need for a second CI mechanism rerun merely because the candidate is called “default-on” if exact-source/dispatch evidence justifies reuse; changes that alter the measured treatment require fresh evidence.

From **today**, add the one admissible repaired CI diagnostic:80 decodes (2 arms ×2 thread cells ×20 rounds), only after accepted preflight/freeze. A hypothetical two-arm target package with R approved rounds and K approved workload/thread cells costs2RK decodes; four treatment/layout combinations cost4RK, before any reference arm. This formula is budgeting, not authorising R or K. Pinned bench.yml uses10 rounds and its normal1/2/nproc grid: the baseline/head contribution is2×10×C×G decodes for C corpora and G distinct thread cells, plus its existing stage/other work. Keep that workflow intact. Three timing packages from today is therefore a lower-bound plan, conditional on each earlier gate; exact decoder count cannot be specified without the target registration.

Do not use a future LEAD to grandfather current1197bbcc as default-on: it is explicitly default-off. Do not use a same-output test as performance or reach evidence. No merge is proposed by this report.

## Disposition and evidence scope

Expected improvement survives as a plausible priority; admission failed exactly as designed. New learning: shipped-worker transfer is closer, but default routing, release layout and machine-resource transfer remain independent gaps. Highest-value next move is the bounded collector repair and fresh diagnostic, while finishing the small D060 safety delta. Construction redesign C43 remains economically inferior as the next bet absent stronger matched evidence.

**OBSERVER:** distinguish “C29 first for investigation” from “C29 qualified”; .9700 remains reported/UNJUDGED. A later judged CI LEAD still leaves exact-default safety/reach, target C5 adoption evidence and release/off-path CI evidence.

No native execution, compilation, timing, implementation, merge, safety certification or changed acceptance. Migration head/digest checked before receipt. Pinned routing audit: five resolved historical findings, zero unresolved. No unpinned §11 authority adopted.
