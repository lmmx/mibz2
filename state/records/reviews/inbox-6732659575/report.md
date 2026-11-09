# #270: separate the kernel, the record path, and the exposed leader

Inbox [6732659575](https://github.com/lmmx/mibz2/pull/160#issuecomment-6732659575). Specs: `a4f97714cc14366984015869d9d094d89488ac57`. Result journal: `d1260c59047846401ed0e717ba6c66d33b699f70`. Measured source: `03e52206ac248737bc856f064928adf238265917`, implementation `f67085670ee696340337aa37377afab886dded3a`.

**Rank: C29 in the shipped worker first; a fresh leader-sort versus matched-C29 registration second; pipeline-context placement third. Keep dir as the primary of the existing workstation transfer.** The 9-thread lead reproduces. It does not price the i9 or qualify production. Crucially, leader-sort already loses to C29 in the less oversubscribed, one-team cell. Its shorter team-run span hides a larger cost outside those runs.

## 1. Evidence and replay

Downloaded Actions artifact **13650328398**, `sweep-03e52206ac248737bc856f064928adf238265917`, from run **42564217298**. ZIP SHA-256 **97f370cbb4ecbb8b2ed6262e1358d4d608ad497b3c1a1c4e082a26a696d00f4d** equals GitHub's digest. Archived raw JSONL, identity, environment and report alongside this review. Ran the pinned Python reporter, not a decoder: **VALID (JUDGED), attributed LEAD**, all 210 records, 15 rounds, matching output. This replays admission and calculations; it is not an independent proof of every execution or safety claim. Reporter dependency `team_size.py` was fetched after the first invocation reported `ModuleNotFoundError: No module named 'team_size'`; the complete invocation then succeeded.

The additional calculations use the same paired log-ratio t interval, n=15, df=14; differences use a paired arithmetic-mean t interval. They are **exploratory, unadjusted mechanism contrasts**, not new registered verdicts or a new multiplicity policy. Three-thread `default33` is excluded from our contrasts: it still forces three teams, unlike the other one-team arms.

| Contrast | 9-thread wall ratio [95%] | 3-thread wall ratio [95%] |
|---|---:|---:|
| reg / ctl | .973935 [.968233, .979671] | .958780 [.952761, .964836] |
| dir / reg | .992134 [.986550, .997750] | .999362 [.992309, 1.006464] |
| ls / reg | .974647 [.969726, .979593] | **1.015168 [1.001754, 1.028762]** |
| ls / dir | .982374 [.977432, .987342] | **1.015817 [1.001112, 1.030738]** |
| ls / hs | .969765 [.962592, .976991] | .992316 [.973683, 1.011306] |

Registered dir/default33 = **.966859 [.959421, .974355]**, CPU **.963958 [.960188, .967744]**. ctl/default33 interval is inside [.98,1.02], and dir/ctl high <1: the registration's attribution condition holds. Preserve that result exactly. `ls` remains secondary; its data cannot retrospectively make it the judged primary.

## 2. What can actually be attributed

### C29 is a kernel package, not a list-only intervention

At the measured source, `src/mtf.rs:mtf_checkpoints_reg_avx2` changes both:

- the first 32 MTF entries from a load/store memory list to register halves, holding output bytes directly; and
- short-run writes from `write_bytes` to two 32-byte stores when `len<=64 && n+64<=max`.

It preserves checkpoint semantics but also changes lookup/store dependencies. **reg/ctl estimates this combined checkpoint-kernel replacement in the instrumented worker.** No arm isolates register-list work from short-run stores. A split would require a separately frozen factorial or one-change contrast; it is unnecessary before trying the intact package in production.

At 9 threads, reg reduces wall 2.6065% versus ctl and CPU 2.9730%. Its in-run front falls **.18003 ms/team-run [.13861,.22146]**, 27.4% of ctl's mean. Total leader work falls .19530 ms; team run falls only **.08817 ms [.05065,.12568]**. Earlier leader completion moves work into wait/overlap, and changes scheduling of the helpers. The front reduction is not an additive wall saving.

An exact *descriptive* factorization of geometric-mean points is `dir/ctl=(reg/ctl)*(dir/reg)`: 2.6065% then .7866%, giving 3.3727% altogether. In log space reg/ctl is 77.0% of dir/ctl's observed reduction. For ls/ctl it is 50.7%, followed by ls/reg's 2.5353%. These percentages are an algebraic allocation of observed contrasts, **not fractions causally mediated by leader release**. Registered comparisons to default33 give the slightly different 2.5475%, 3.3141%, and 5.0182% reductions because ctl/default33 is not exactly one.

### The additional contrast is not records alone

`run_team.rs:front_stage` selects reg for **all** CTLR front stages; LS/HS/DIR select `mtf_runs` when `takes_runs` holds and retain the **old** `mtf_checkpoints` otherwise. Their small-block path also differs. Thus dir/reg and ls/reg combine records, construction, prefix/synchronization, memory layout, rejected-block kernel choice, small-block costs and different exposure to overlap. They do not isolate an emitter or a builder. The next matched experiment should use C29 on every rejected column path in every arm, and identical small-block policy, or explicitly register the remaining differences as part of the candidate.

The phase labels need two further corrections:

1. DIR's **.754 ms `helper_sort_ns` is not a .754-ms sorting algorithm**. In `build_share`, it brackets byte counting, the counted gate, prefix/checking, and waiting. Direct mode skips the F-order scatter and its second gate. This is a sum across helpers, including waits; it cannot be subtracted from the leader's .215 ms as if both were serial service. DIR's 163.097 predecessor lookups/team-record block and resumed-cursor work are in the fill side, not the entire prefix cost.
2. `rule_front_ns/rule_blocks` restricts to rule-accepted blocks **staged inside runs**, not a fixed set of block identities. At 9 threads these counts range ctl228–252, reg222–258, ls223–253, dir212–251 across rounds; all arms see the same file but expose different subsets. The complete accepted population is 384 blocks (186 team,198 small). Aggregate phase ratios do not establish paired same-block release. Future diagnostics need block IDs and separate in-run/drained front costs.

### The timeline explains why release alone is the wrong objective

Means, milliseconds per team run; end times share the source's corrected t0. `helper_*` sums are not critical-path times.

| At 9 threads | ctl | reg | ls | dir |
|---|---:|---:|---:|---:|
| leader work | .962 | .766 | 1.034 | .823 |
| leader start | .434 | .415 | .338 | .302 |
| leader end | 1.396 | 1.181 | 1.371 | 1.125 |
| build end | 1.874 | 1.808 | 1.621 | 1.867 |
| join wait | 1.632 | 1.709 | 1.458 | 1.769 |
| run span | 4.769 | 4.681 | 4.435 | 4.628 |
| leader late fraction | .257 | .209 | .295 | .192 |

LS wins despite **more leader work and more late leaders** than reg. It pays sorting while preparing the next block, removes helper-side coordination on that block, and brings the measured build end forward .18706 ms versus reg. DIR has *less* leader work than LS and earlier leader completion, yet a later build end. Neither smallest `leader_late` nor smallest leader duration is the economic target. `join_wait` also includes wake-up/descheduling; even mean B minus mean leader end is not its mean, since the wait is a positive part plus scheduling effects and the leader can arrive after the build.

### A directly recoverable explanation of the 3-thread reversal

There is one team, so its runs are sequential. Separate their summed spans from `drained_front_ns`, which times front preparation outside the pool runs (including leader sort). The following are paired arithmetic differences **ls minus reg**, milliseconds per whole file:

| Component | Difference [95%] |
|---|---:|
| sum of team-run spans | **−10.874 [−17.003, −4.745]** |
| drained front | **+18.239 [16.962, 19.515]** |
| remaining wall residual | −.385 [−.845, .076] |
| whole elapsed | **+6.980 [.702, 13.258]** |

This is an accounting identity from the archived totals, not a counterfactual assertion that every extra drained nanosecond is caused by the sort. The different block partition, scanner availability and scheduler can contribute. Nevertheless the lost time is located: **the run-span win is smaller than the exposed front penalty**. At 3 threads LS/reg CPU is .976939 [.967957,.986005] while wall is worse: lower scheduled service does not suffice.

C29's corresponding ctl-to-reg differences are −5.742 ms inside runs and **−14.339 ms drained**, with −19.487 ms whole elapsed. Its advantage survives both placements. Do not divide the 9-thread sum of team spans by three and call the answer whole decode: those teams overlap and have different critical completions.

## 3. C29 production prototype: where it runs and its B5 surface

Source audit of production at **d1260c59** (`src/parallel.rs`, `examples/ab.rs`) and the measured branch's `bench.yml`:

| Bench / topology | Actual path | Direct expected effect of a pipelined-front-only C29 swap |
|---|---|---|
| standard `mibz2`, 1 thread | `BzReader::new`, serial decoder | **0**; candidate not called |
| standard `mibz2`, 2 threads | `Plan::Workers(2)` | **0**; no checkpointed pipelined front |
| standard `mibz2`, 4 threads | `Plan::Workers(4)` | **0**; fewer than six usable physical cores |
| i9 10-core/20MiB L3, budget10+ | default three pipelined teams of3, park+small | candidate front called, including drained and small front stages |
| other cooperative/default plans | plain cooperative worker; not default pipelined33 | no effect if change is scoped to the pipelined call site |
| explicit pipelined APIs / forced33 diagnostic | pipelined front | exercised even where default would choose Workers |

The normal CI runner has four vCPUs/two cores in `rfl-env.json`. `bench.yml` uses the default `mibz2` arm, builds without `run-fed`, and runs 1,2,nproc. **Expect ratio≈1 at all three cells, not a 2.5% win.** Layout/LTO noise or unintended shared changes remain possible; neutrality is not evidence that the intended kernel has no benefit. Conversely a favourable default1/2/4 reading cannot be credited to a path that never ran. Do not silently force teams or add a thread cell to the canonical benchmark: RULES C6 reserves its workload/grid. Keep the ordinary regression A/B intact; use a separately registered targeted diagnostic and the submitted target-machine evidence under C5. A neutral CI grid alone cannot demonstrate the required production benefit on the changed path; the integration evidence route must be resolved without relaxing the acceptance rule.

**Narrow construction:** port the intact C29 kernel into the shipped pipelined front, keep its current counts, checkpoint spacing, buffers, pool, publication and recovery. Retain the old checkpoint entry for other callers if their scope is not part of the registration. Do not copy `run_team` into production or carry its phase atomics. Plain workers already use the modern list; this is not a global MTF upgrade.

**Price:** observed front ratio .726 at9 and .750 at3 is useful for planning, not a forecast interval. If e is the fraction of *target critical wall* exposed to these calls and the target has front reduction q, first-order benefit is `e*q`, before changes in helper joining, scanner supply and cache interference. q=.25–.274 with e=.10 gives only2.5–2.74% as a sensitivity example. e is unmeasured on the i9. C29 is preferable because it removes front work in both overlapped and drained contexts without adding a representation or barrier.

**B5 is narrow but real.** Transfer only unchanged component evidence, and review the new shipping pin and call boundaries:

- AVX2 dispatch on supported CPUs; scalar/non-x86 fallback. Do not widen the acknowledged forgeable-`Kernels` issue into approval for arbitrary feature claims.
- `reserve(max)`, no stale raw pointer after vector reallocation, `set_len(n)` only for initialized n≤max; stores stay within allocation even when exceeding the logical run/checkpoint. Future bytes are rewritten before publication, and helpers never observe an unfinished front buffer.
- Register shuffle tables/alignment, `second` tail bounds, symbol/byte-map invariants. Preserve malformed-input error order, output prefix and counts; successful checkpoints equal cumulative counts at exact multiples. Checkpoints on error are not promised complete.
- Runs split across one/multiple checkpoints; byte exactly at boundary; end-of-block exactly at boundary; len0 remnants and63/64/65 stores at allocation ends; RunTooLong/BlockTooLong, no-EOB and empty cases; reused buffers and recovery.
- Independent adversarial review, differential and fuzz coverage of the **shipped** pipelined entry and its invalid/error compositions; scalar and actual AVX2 Miri selection; mutations targeted at checkpoint-before-count, dropped final checkpoint and `n+64<=max` guard. #270 R2.3 documents such tests and C29a/b/c kills, including Miri UB for C29c. They are attributed evidence, not runs repeated here, and do not automatically certify integration.

Suggested separate C29 diagnostic: one prospective primary, shipped-C29/unchanged shipped pipeline on the intended plan; target a ≥1% wall reduction (point≤.99, upper95%<1), CPU upper≤1.05, all validity and safety predicates fixed before new data. These are **proposed diagnostic decision thresholds**, not a replacement for bench.yml or a production admission rule. Keep counts/listings for the whole changed decoder, especially spills and short-run stores; instruction decrease alone is not qualification.

## 4. LS versus DIR: next registration and next construction

**Yes: LS deserves the next records-primary registration, after C29 has a matched reference. No: it should not replace DIR in #270's existing result or frozen-transfer permission.** The choice is informed by these data, explicitly; qualify only on new prospective data. LS is strongest at9; DIR is indistinguishable from reg at3, and LS is slower. The i9 winner is not determined.

For a new record experiment, set common fallback and small-block kernels/policy to C29 in LS and the reference. Make LS/matched-C29 the **single primary**; retain DIR and HS as fixed exploratory secondaries. Suggested economic screen on a separately registered9-thread diagnostic: point≤.98, upper95%<1 and CPU upper≤1.02; stop if lower95%>.99; otherwise inconclusive. A2% incremental target prices a second representation, sorting and safety surface against C29's much cheaper package; it is a proposed engineering hurdle informed by the exploratory2.535% result. The Coordinator/Researcher must freeze the exact estimand, repeat/multiplicity policy and any required authorization before a run. Do not reinterpret the old samples or change the standard bench rules. Three-thread exposure remains reported, and a workstation primary/threshold must be prospectively fixed, never selected after its results.

Controls that distinguish records from C29:

1. The primary against matched-C29, not old ctl. Record all front stages by block ID and rule eligibility, including drained, RLE/CRC and small blocks.
2. LS/HS holds the sorted-table representation and equal-record partition fixed, and moves prefix/scatter work between leader and helpers. It estimates placement **plus its induced gates/scheduling**, not pure instructions. LS/DIR additionally changes the representation, bucket streams and C35 partition; it cannot isolate sort placement.
3. Whole decode and actual critical-chain/run readiness first. Keep helper service, off-CPU waits and readiness skew separate. LS's worse store imbalance (largest/mean1.256 against DIR1.007) coexists with better wall; eliminating the maximum stored-word count is not the same as minimizing completion time.

**Next construction, C42: make sorting placement respect pipeline exposure.** LS already sorts the *next* block during the current block's helper work. “Move it earlier” is not new unless it creates additional slack. Its weakness is sorting when no helper work hides it. A proposal worth a bounded diagnostic is leader-sorted F records only when preparing a successor during a live run, direct-record fill for fronts prepared with the pipeline drained. Path identity must travel with the prepared block; both paths retain checked construction and exact ownership. This is not permission to implement, and not part of §8's unchanged recipe. Its first falsifier is whether avoiding the measured drained-sort penalty outweighs the newly introduced helper prefix/fill cost at those transitions. The raw totals lack per-block sort/drain identities, so no recovered-ms claim is made.

For a block with leader arrival A, other leader work L, potential added sort S and build completion B, the fixed-schedule exposed sort penalty in the simple pre-walk max is `max(B,A+L+S)-max(B,A+L)`, between0 andS. The real price additionally includes lost leader walk participation, next-block readiness, scanner gaps and changed B. Remove helper work only where its critical-end saving exceeds that total. LS versus DIR cannot be decided by .215 versus .754.

Emitter fusion is lower priority: F offsets need final byte counts/prefixes. Emission can accumulate byte lengths, but immediate final F scatter requires deferred placement or per-byte buffering and a copy. Charge it. A one-pass per-byte-chain proposal that merely renames the scatter does not remove it. Existing serial #277 direct-F work remains useful for fill/emitter costs but cannot by itself predict this pipeline's span; D055 still holds its timing.

## 5. CI-to-i9 transfer and the frozen recipe

No “divide the phases by9/4” prediction. Measured topology is four logical CPUs/two cores; nine decode threads plus scanner/read/sampling contend. At3 there is one team but still four logical CPUs on two physical cores, **not** three dedicated physical cores, and fewer competing tables than on the i9. Its reversal is a falsifier of universal ranking, not a substitute workstation run.

Conditional prediction: C29 remains the lowest-risk positive mechanism; DIR's incremental gain is fragile; LS saves service but is sensitive to exposed leader work, pipeline drainage and arrival. On the i9, reduced descheduling may remove much of DIR's helper-prefix waiting, shrinking LS's advantage, while dedicated cores and three live tables change memory pressure. Neither the sign nor percentage of the incremental record benefit is identified. CI's leader-late fraction is not a target-machine admission threshold.

The existing §8 transfer should retain its seven arms, DIR primary, LS/HS secondary, fixed default33 reference,20 rounds at10 threads and frozen source/build/corpus identity. Its future admission must require the actual plan/topology and output, no fallback, complete finite records/order, stable binaries/corpus and scheduled-service accounting. Do not run the9/3-thread reporter on10-thread data and call it judged. Freeze a10-thread verifier and explicit target thresholds before collection; retaining .97 wall/upper<1 and CPU-upper≤1.05 with the existing [.98,1.02] control-fidelity band is a coherent proposal for that DIR question, not an already registered or approved target verdict.

The run needs to show:

- positive **whole** decode against the shipped plan, and the separately reported DIR/REG incremental contrast; a records-only claim cannot be earned by C29's win;
- where the critical worker waits: common-origin build/leader/run events, actual leader entry to walking, scanner supply, startup/drained gaps and tail; ctl/dir occupancy traces required by §8, and a separately authorized matched-C29/LS trace if its mechanism is to be resolved;
- enough CPU/affinity/topology evidence to distinguish dedicated cores from SMT/time-sharing, no hidden plan substitution; accounting for three table footprints, metadata/RSS, parked waits and frequency differences;
- exact source B5/admission, one reproducible bundled recipe, and Louis's own C5 submission. The CI lead authorizes preparation of that recipe, **not an agent workstation run**.

## 6. Decisions and scope

Expected a build/leader tradeoff; observed a registered DIR lead with much of the contrast reproduced by C29, and a larger exploratory LS lead that reverses at3. The gap closed is3.314% versus forced incumbent on this CI cell, not a production or i9 gain. Mechanism advance: C29 is a compound kernel change; rejected-path mismatch blocks a pure-record decomposition; the one-team wall loss is accounted for by drained front exposure. Next marginal value: narrow C29 integration prototype and matched LS registration, above another pair-consumer construction or unisolated helper-sort tuning. Keep #277's serial question separate, with its HOLD intact.

Migration checked before receipt: old head `e6eea5889432aef28f4d9e450cee87dd5bf7bacf`, manifest SHA256 `86d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde`. Routing audit: only the five historical resolved entries. Current channel records D056's scoped #270 CI permission; D057 lifts the separate service evidence hold for its named snapshots, not this proposal's integration obligations. No constructor implementation, compilation, timing, safety certification, merge, new speed credit or automation change performed. All proposals remain proposals.

Reproduction: `python analyse.py` from this directory (raw data under `rfl-raw/`), and `python run_fed_leader.py report rfl-raw/rfl-runs.jsonl --head 03e52206ac248737bc856f064928adf238265917 --identity rfl-raw/rfl-identity.json`. Python arithmetic only.
