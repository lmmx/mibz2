# Three-table screen: strong follow-up candidate, unidentified capacity mechanism

Researcher, 2026-11-06. Response to [6682938532](https://github.com/lmmx/mibz2/pull/162#issuecomment-6682938532).
Reviewed archive/registration pin: `1091663e7beb6996eafb05a9896ac62b892600f1`. Instruction pin: `0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0`.

## Recommendation

Keep the fresh throughput grid and qualification rule. The screen justifies testing the selected three-table configurations, not declaring that L3 capacity is the cause or that the policy cap is already proven wrong. **The proposed occupancy set cannot answer its stated same-k question:** it contains 4×2, 3×3 and 3×4, with no k common to the three- and four-team plans.

Only a narrow diagnostic amendment is needed if that question is retained: include a contemporaneous four-team counterpart, e.g. 4×3 alongside 3×3, under a common explicit allowed-CPU mask and consistent instrumentation. Pair/interleave those configurations within the same diagnostic rounds. No new throughput primary or helper-side tracing is necessary. Alternatively keep the run exactly as written and explicitly defer the same-k block-time claim.

## Numerical reconstruction

I recomputed the log-paired geometric ratios and t intervals from [runs.jsonl](https://github.com/lmmx/mibz2/blob/1091663e7beb6996eafb05a9896ac62b892600f1/docs/journal/2026-11-06-team-size/runs.jsonl), independently of the report script. There are 320 rows, sixteen cells with twenty distinct rounds apiece, one hash, one output length (209,700,196), and no reported decode errors. The following reproduces the archived report:

| Comparison, numerator configuration's throughput | Ratio | 95% interval |
|---|---:|---:|
| 4×3 / default 4×2 | 1.016955 | 1.007929–1.026062 |
| 4×4 / default 4×2 | 1.003463 | 0.992881–1.014157 |
| 3×3 / default 4×2 | 1.075272 | 1.058068–1.092755 |
| 3×4 / default 4×2 | 1.096728 | 1.083018–1.110611 |
| **3×3 / 4×3 (same k, exploratory)** | **1.057344** | **1.038522–1.076508** |
| **3×4 / 4×4 (same k, exploratory)** | **1.092943** | **1.076409–1.109732** |

The first primary fails the 3% point-estimate threshold despite evidence of a smaller positive effect. The second fails that threshold and its interval includes no improvement; its archived CPU ratio 1.5245 also exceeds 1.5. “Neither qualifies” is correct. “No faster” or “do not raise the ceiling” should mean no qualifying benefit, not proven zero effect.

The direct same-k comparisons strengthen the descriptive association with team count. They are post-hoc comparisons, not newly qualified results. They change 4k to 3k decoding threads and sixteen to twelve slots, as well as team/table count. They identify a configuration effect, not an isolated cache-capacity effect.

The registration appropriately uses fresh data and requires confirmation without reusing the screen. Selection can bias the winning estimates upward; it does not prove that either particular estimate is inflated. Likewise, a future failure to pass does **not** establish that the screen was “selection or noise”: failure may reflect a genuine smaller gain, uncertainty, an adoption limit, or a validation failure. Report which predicate failed and the resulting interval.

## What the screen supports

**The table-count pattern is a good lead, not a complete explanation.** The strongest narrow statement is: on this input and host, three-team configurations outperform the four-team configurations at k=3 and k=4, while additional helpers at four teams produce little improvement. The result does not show that k is generally irrelevant: the two-table cells differ more, and not every G×k combination was measured.

Throughput divided by team count is not a direct measurement of each table's service rate. It also depends on utilization, block mix, pipeline overlap and startup/drain. For B blocks, average decoding concurrency A and mean block span L, the accounting relation is B/span = A/L. A whole-run speedup with fewer teams does not alone reveal L. With equal utilization, a three-versus-four-team throughput ratio R would imply L3/L4 approximately 3/(4R); equal utilization is an assumption, not evidence. The proposed trace can measure the missing terms.

**Four live T-vectors is a maximum phase overlap, not constant residency.** In the spread timeline summaries, `q.tvec_hw=4`, but `q.tvec_mean` is approximately 2.75, 2.61 and 2.59 for k=2,3,4. These describe inversion-call overlap, not hardware cache contents. Allocated tables may persist outside their active inversion intervals.

**Other buffers are plausible contributors, but the quoted ledger needs care.** In [occupancy.rs, footprint](https://github.com/lmmx/mibz2/blob/1091663e7beb6996eafb05a9896ac62b892600f1/examples/occupancy.rs), `f.w.cap_mb` is mean recorded capacity **per worker/team leader**, including all recorded buffers; `f.w.cap_mb_max` is the largest worker total. For spread 4×2 these are 13.892 and 16.173 MB, and the total already includes 5.400 MB of T-vector capacity per worker. They are neither “14–16 MB of other buffers” nor measured concurrent cache footprint. Allocation capacity, active lengths, phase overlap and cache residency must remain separate. Do not add this total to T-vector sizes and call the sum resident cache pressure.

Consequently, “the cap is one table too many now” is a useful tuning hypothesis. “The optimum moved because other buffers consume the remaining L3” remains untested. Shared service/request contention, physical-core resources and changed pipeline buffering remain possible. No perf evidence was obtained, and the October/current comparison is not a controlled experiment on the cause of any shift.

## Occupancy findings and the minimum missing comparison

The archive supports a decoder-side limitation in the observed runs: spread leaders are inside decode about 96% of their aggregate lifetime and idle about 2.8%, with mean block spans 1.3868, 1.3789 and 1.3977 ms. This is useful evidence against large average upstream starvation. It does not prove every helper is busy or every marked decode interval is useful computation; inversion includes waits and serial assembly.

The new [recipe](https://github.com/lmmx/mibz2/blob/1091663e7beb6996eafb05a9896ac62b892600f1/.just/bench.just) calls occupancy at `8:2 10:3 12:4`. All three points change k when comparing three versus four teams. Archived 4×3/4×4 traces from the previous run can be historical context, but are not a contemporaneous paired control for the new three-team traces.

If the same-k question is to be answered now, add **one** 4×3 diagnostic counterpart and compare with 3×3:
- same corpus and build, same trace mode, same explicit CPU mask for each comparison;
- retain the intended default-window policy, while acknowledging twelve versus sixteen slots;
- retain block counts, elapsed decode spans, decoding concurrency and queue/wait decomposition;
- compute uncertainty across independent process rounds, not by treating the 431 blocks sharing a process as independent replicates.

Under the default generated spread/packed masks, requested T=10 and T=12 offer different CPU sets. Matching the mask removes that additional placement change. Total runnable work still differs by design; this tests block service under the two configurations, not cache capacity alone. A constant-total-window or hardware diagnostic can follow only if the mechanism warrants it. The registered throughput screen does not require any of these extra causal controls to test its primaries.

## Placement and perturbation claims

Placement is a real second lead. The archived paired off-mode packed/spread ratios are:
- 4×2: **0.777876 (0.755056–0.801386)**;
- 4×3: **0.916761 (0.888510–0.945910)**;
- 4×4: **0.982072 (0.967919–0.996432)**.

For 4×2, packing restricts the process to four physical cores versus eight for spread. Thus the effect is of the whole-process topology mask, not a separately identified cost of helper sibling sharing. The trace-feature binary's spread-versus-unpinned production difference remains only a lead, as the write-up acknowledges.

Replace “tracing did not perturb the runs” with “no perturbation was detected.” For example, spread 4×2 full/off throughput is 0.984267 with interval 0.965193–1.003718: a roughly 3.5% slowdown remains compatible with this interval. Packed 4×4 timeline/off permits about 3.8% slowdown. These limits matter when interpreting one-percent block-time differences. No further diagnostic is needed merely to report that uncertainty.

The scanner's quoted 38–46% corresponds roughly to slot waits with the earliest block decoding, not total slot waits: spread 4×2 also has about 8.2% with the earliest block ready. Use the named components or the actual combined per-run measure. This does not reverse the backpressure interpretation.

## Scope and disposition

Sources inspected: both pinned journals, recipe, raw timing rows, reported verdict, all three occupancy JSON artifacts, and footprint accounting source. Recomputed six paired timing comparisons; inspected trace completeness/parity summaries and perturbation intervals. This is not a native replay or exhaustive audit of every registered plan predicate. No new measurements, implementation, policy change or merge.

Proceed with the fresh configuration test. Narrow the capacity and no-perturbation claims. Add the single matched diagnostic counterpart only if claiming a same-k three-versus-four-team block-time result from this run; otherwise state that this causal question is deferred.
