# Pair STOP: measured residuals, construction budgets, admission

Brief [6731656628](https://github.com/lmmx/mibz2/pull/160#issuecomment-6731656628). Source/run pin **1c4d0f0714183f83798a0b746463ae34535fbc50**; result journal **96b0794d9605614e691b4356032a9120f1bb6baa**. Specs a4f97714cc14366984015869d9d094d89488ac57. No implementation, compilation or timing.

## 1. Replay and the measurement that is absent

Downloaded artifact13647977266 from [run42556998342](https://github.com/lmmx/mibz2/actions/runs/42556998342). ZIP SHA256 **d5ae792b4cc8e5637d923e63202c8687d83037071dfac7261d69a80843c496a7** matches GitHub metadata. Plan identifies fifteen rounds, five copies, fixed head and corpus digest. Archived CSVs/plan/validity, arithmetic script and results accompany this report. Replay of the registered pinned-copy ratios agrees:

- Q1 run/row **.686538 [.684484,.688598]**, INCONCLUSIVE.
- Q2 R/P **1.894994 [1.885046,1.904994]**, NEGATIVE STOP.
- M1 run−one **3.457402 [3.451597,3.463207]** nominal cycles/paired BWT row, NOT MET.

All five copies were parsed as complete fifteen-round six-arm matrices; exploratory copies remain separate, never pooled into the judged result. Replay does not independently certify all native safety or the workflow validator's contracts.

For each round i I calculated, using *seconds before scaling*:

    d_Ri = (T_Ri − T_runi) × 3.3091e9 / output_bytes
    d_Pi = (T_Pi − T_onei) × 3.3091e9 / output_bytes.

Intervals below are exploratory arithmetic-mean t intervals with14 degrees of freedom, paired by round; same-round gap uses d_Ri−d_Pi. These describe repeat variation in these residuals, not causal uncertainty.

| Wikidata quantity | Nominal c/output byte, mean [95% interval] |
|---|---|
| R−run | **6.07213 [6.02397,6.12029]** |
| O−row | **6.16865 [6.14005,6.19726]** |
| P−one | **3.98378 [3.96314,4.00443]** |
| (R−run)−(P−one) | **2.08834 [2.03103,2.14566]** |

**There is no pre-built-table consumer measurement in these CSVs.** The requested consumer-only number and interval are not identifiable. The harness records six totals per corpus/round, not walk-only or per-block costs. Build and whole-entry sweeps use different scratch objects and cache histories. Build arms cover233 paired blocks; R accepts231 and declines2. Whole arms also include short-block paths, setup/marking, output buffers and assembly. Build rows209,697,662 differ from decoded output209,700,196; converting seconds with the output denominator avoids even that small scaling mismatch.

An internal counterexample makes the distinction concrete: synthetic text declines all four pair blocks, and R−run is **−9.43424 [−9.51898,−9.34951]**. That cannot be a negative consumer time; we subtracted work R did not perform. Wikidata's mismatch is smaller, not known to be zero. Even O's better-matched path retains setup/cache differences. These intervals must not be relabelled “pair walk” or “room walk”.

Consequently “no builder can carry Q2” is not proved by the6.0 subtraction. The result demonstrates the actual combined candidate loses. A free-build counterfactual changes cache state/setup and is unmeasured; it may be a useful heuristic, not a measured impossibility.

## 2. What consumer would be worth building?

Using residual means only as an explicit planning proxy, pair residual must drop from6.072 to below3.984: **over2.088 nominal c/B, about34.4%**, while no excluded/setup term grows. Ordinary pair-step budget would be roughly **below7.968 nominal cycles/two emitted bytes**, versus a12.144 residual-equivalent step. Neither is a measured hot-loop budget. Nominal clock-scaled wall is not actual retirement cycles.

The stronger **whole-entry** constraint remains extra build plus extra assembly/setup:

    w_pair < w_one − delta_build − delta_setup/assembly.

Using the printed medians only for a severe sensitivity check, P6.184 minus run-build5.668 leaves about **.516 c/B** for everything else if that standalone build price carried over. That is NOT a valid counterfactual confidence bound. It demonstrates why “pair walk just beats room” is insufficient with this builder. Even a hypothetical50% cut of the6.072 residual leaves roughly3c/B, far above that residual budget. Improve or replace construction too.

Instruction and latency budgets for a separately registered consumer diagnostic:
- Let u1/u2 be effective resource demand per ordinary one-hop/pair step. For issue-bound improvement require **u2<2u1**, with marker/tail/flush overhead added separately. An illustrative8-instruction one-hop loop allows fewer than16 comparable pair instructions; current emitted assembly must establish the actual u1/u2 and port pressure. Raw instruction equality alone does not prove throughput equality.
- For latency-bound navigation require **lambda2/(2M2)<lambda1/M1**, where M is effective overlapping misses, not configured lanes. At32 lanes, pair emission does not double independent miss slots. A directory load can consume load bandwidth even if it is not on the next-link recurrence.
- Preserve demand/cache state and marker setup across walk arms; time only the justified consumer boundary, then time the joined whole entry. Do not repeatedly walk a mutated or artificially warm table as if it were decode state. Tiny periodic cases, odd handoffs, depleted lanes and final truncation need their own correctness coverage.

**Rank within consumers: C37, then C38 combined with it.** C37 removes per-lane pointer/capacity updates via shared-offset staged rooms (128 pair rounds per256-byte lane tile). Flags,32 lanes and next-row prefetch already exist. C38 removes the *common dependent bucket-end load*, not the whole symbol lookup: a3,516-byte high-symbol directory load remains. Bits28–29 are available; bit30 is LF marker and31 START. Pointer masks, tag extraction and exceptional exact lookup are charged. At most63 exceptional256-row blocks is a static bound, not dynamic segmented frequency.

The .002 correction iterations do not bound common-end-load cost, but also do not show C38 can recover2.088c/B. **C38 alone is not a credible quantified solution yet.** C37+C38 have a mechanical path to lower work, with no demonstrated34.4% budget recovery. First falsifier: emitted loop, counted dynamic loads/stores/spills/flagged visits and matched consumer-only timing after safety requirements. Retain the earlier report's marker and periodicity obligations. D052 closes the old D046/D051 registration STOP according to the published record; it does not pre-certify a new consumer.

## 3. Which pair-build work survives in production one-hop construction?

At the run pin, build_pairs_runs:
1. validates inputs and initializes F buckets/first-symbol directory;
2. scans L to reconstruct runs, stores p/a/c and counts per byte;
3. counting-sorts F descriptors and fi;
4. merges L starts with F intervals to construct js;
5. fills Psi² through image intersections, reading L[q] for each low byte;
6. computes LF rows for segment starts and Psi(orig_ptr).

A one-hop direct-F construction from MTF records is **not this5.668c/B builder with one arithmetic instruction removed**.

| Work | One-hop direct-F disposition |
|---|---|
| L scan/run reconstruction | Eliminated only when MTF supplies valid records; otherwise survives |
| first-symbol directory, pair LF markers and pair start | Removed; production start/mark setup still costs |
| fo/fi scatter and js composition map | Removed by direct byte prefixes plus resumed L cursors; replacement prefix/validation/search work is charged |
| second Psi composition/image intersections | Removed; one-hop still intersects each F interval with L runs to obtain low byte |
| per-row L[q] low-byte gather/load | Replaced by constant L-run byte over each intersection cell when using records |
| counts, tiling/length checks, allocation, vector fill and tails | Survive in appropriate checked representation |
| parallel ownership | Exact physical disjointness needed; never transplant pair serial overstore repair across helper boundaries |

Use production T[q]=L[q] | Psi(q)<<8. A run starting p with F interval[a,a+len) supplies Psi(q)=p+q−a; intersect that interval with L runs for L[q]. Per-symbol resumed cursors eliminate per-run binary searches, but not cell traversal. Previously read #270 census163 initial lookups/full team block is not a timing or universal one-worker count.

**Global rank1 remains C31 one-hop direct placement**, keeping production's fast consumer. Compare with #269's budget .218+delta_direct−e−f, not #272's pair premium. With no emitter or fallback savings, direct replacement must recover another.218 beyond ideal sort removal; with fallback.100 removed, another.118. The earlier1.332 sorted fill is not direct-fill timing. Different runners/denominators also forbid importing this run's2.213 into #269's budget.

Rank2 is the matched emitter/fallback work already specified in review6730842547. Rank3 is a bounded C37 consumer control; rank4 adds C38 only if the common-path budget survives. A new wide-word or mixed one-hop/pair representation ranks below these: changed footprint/lookup/markers risk paying more than the identifiable saving.

## 4. Admission: define what “wins” means

Raw pinned-copy **run-pair build / production one-hop build**:

| Corpus with paired blocks | Ratio [95% interval] |
|---|---|
| Wikidata | **2.56274 [2.55646,2.56904]** |
| text | 3.76093 [3.73721,3.78479] |
| random | 15.16932 [14.83472,15.51146] |
| one byte | **2.27402 [2.18462,2.36706]** |
| pattern | **1.00913 [1.00287,1.01544]** |
| records | 16.10499 [15.92924,16.28268] |

Synthetic runs has no paired blocks, so no builder ratio. The .239/.343 successes were **run-pair versus row-pair**, not versus production. None of these aggregate paired-block corpora shows the pair-run builder beating production one-hop. The one-byte data has98,040 paired BWT rows but5,000,000 decoded bytes: output-normalized cost there is not directly comparable to Wikidata.

For choosing between **two builders of the same pair table**, a rule can compare predicted costs in r,n,cells, with validation/admission included; for choosing production one-hop, the baseline and table semantics differ. No selection of a pair builder fixes the consumer automatically.

The CSV contains **corpus/round/arm totals**, not per-block build times or per-block r/cells. Therefore a per-block threshold or its recoverable Wikidata gain cannot be inferred. Two blocks can share r/n,cells/r but differ in run lengths, vector-tail inflation, byte-stream distribution, table residency and validation costs. Aggregate averages also allow hidden winning/losing blocks; they cannot establish that every Wikidata block loses.

An oracle's builder gain would be sum_b max(0,T_baseline,b−T_candidate,b), minus acquisition/dispatch costs. These data provide the sum of differences, not the sum of their positive parts. **Recoverable Wikidata savings: unmeasured, not zero and not31%.** A corpus-level retrospective choice chooses run over row on Wikidata; it saves the reported31% pair-build time, already obtained by the current R path, not incremental admission credit.

Buildable next proposal: retain C41's n/literal/run-extension/length-bin counters, and add cell/tail/active-byte estimates when they can be acquired economically. Exact cells/r requires construction-related work, so do not call it a free early selector. Fit a prospective cost rule on separately labelled exploratory per-block records; validate on fresh registered data, keeping existing screens and avoiding timing an oracle in production. Begin with a census/attribution acquisition, not an invented cutoff. On the present aggregate evidence, default production one-hop remains the conservative choice.

## 5. Handoff and limits

Expected: current pair consumer would fail to repay build; observed Q2 loses89.5%, M1 misses the plus-one condition. Understanding gained: reproducible residual intervals, exact failure of consumer identification, and strong synthetic savings against the wrong baseline for production admission. Measured gap closed by this review: none.

The economical next construction is one-hop direct-F plus the existing emitter/fallback controls. Pair-room work is a secondary information experiment whose separate consumer gate cannot qualify whole entry. #270's latest recorded D049 restriction is not lifted here; repairs/new pins retain their own evidence obligations. No gate, threshold or historical verdict changed.

Five routing-audit findings were explicitly resolved historical items. Migration head and manifest SHA256 matched. Pinned protocol verify hardcodes FROM coordinator; route/check plus exact author/body/destination readback used without claiming optimizer verify-pass. The unpinned §11 reference grants no extra authority.
