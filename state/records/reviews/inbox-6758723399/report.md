# Compact30 plain workers: conditional diagnostic, not a measured tie

Inbox [6758723399](https://github.com/lmmx/mibz2/pull/160#issuecomment-6758723399); dispatch QUEUED6758603004. Specs: a2da49638ac54d7763de1c167bd054f67ffdc763. All results below are source inspection or arithmetic on historical records. No implementation, compilation, new timing or safety certification.

**Disposition:** worth a small, separately registered diagnostic of the existing compact30 kernel. Do not append it to #326's workload-interaction experiment. The model reproduces approximately2350MB/s at seven workers, but construction on the target remains unmeasured. Preserve D4-delta's filters and #195's failed verdict/reopening restriction.

## 1. What the archive actually establishes

Pins inspected:
- [Pathfinder register](https://github.com/lmmx/mibz2/blob/3a63542b2446071726bba30870e6af7012662d5b/docs/journal/2026-11-10-hypothesis-register.md), with section B and section R at the same pin.
- [Historical grid](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-compact-psi/coop-20261029T215136Z.jsonl), Git blob3b7ed3f85dac17e24d4a80d15c77a50fca93b819; SHA-256 39f7cf800fd16b836fe53914892c70074c3ff7104800f54ac644038d66de0684.
- [Measured harness source](https://github.com/lmmx/mibz2/blob/b99b92bcd89f43c22003018231e866f631ae57c3/examples/coop.rs), blob7ffebeaacc03febc19921932cf529293db14287c; compact helper blobabb39cc67b762714e38b27f107a13720b13cdcf8.
- [Compact30 source](https://github.com/lmmx/mibz2/blob/ba94ff5ceda1b68a66bb2978af9674604569847d/src/c30.rs), blobc60210095660e801cbabff872b4a9c033bea7d72; bwt, ab and phases at that pin.
- [#195 registration/result](https://github.com/lmmx/mibz2/blob/3a63542b2446071726bba30870e6af7012662d5b/docs/journal/2026-11-05-compact30-staged.md).
- [#326 draft](https://github.com/lmmx/mibz2/blob/482df588e2a97b99c6797c7bc2a24098015968dd/docs/journal/2026-11-10-share-nothing-pricing.md), blob88aaba08899db829c81ddfc3dfe39f9384d4a504.

[Replay code](loadlaw-replay.py) and [numerical output](loadlaw-replay.json) accompany this report. Place the pinned JSONL beside the script as loadlaw-grid.jsonl; it reads records and performs arithmetic only. There are420 measured records,21 arms,20 rounds per arm. Match the harness's sorted[n/2] upper-middle convention, rather than averaging the two central samples. This refines the preliminary checkpoint's approximately2350 result to2350.48; it changes no decision.

For ordinary workers the fitted expression is

T(W,s,g,a) = W p1 / [g + a X(Ws)],

where p1=544.135MB/s, and X(3.6W)=W p1/pW−1. X is dimensionless in units of one production worker's inverse-phase time per byte, not nanoseconds per load. Linear interpolation gives:

| live nominal table MB | 3.6 | 7.2 | 14.4 | 18 | 21.6 | 28.8 | 36 |
|---|---:|---:|---:|---:|---:|---:|---:|
| X | 0 | .01536 | .48993 | 1.09290 | 1.96325 | 5.24793 | 8.62725 |

For u24, g=p1/u1=1.466272. The observed/predicted ratios at W=1,2,4,5,6,8,10 are1.000,1.0049,1.0367,1.0309,1.1171,.9553,.8720. The register's rounded result reproduces. The one-worker point is exact by calibration, not an out-of-sample success. The remaining points test a useful transfer to one other representation on one host in one run.

**The transfer is not identification of a law depending on bytes alone.** In p, worker count and total table bytes move together. Clock, concurrent builders, stores, other buffers, memory-level parallelism, output traffic, imbalance and launch costs are folded into X. u24 changes representation, extraction work and footprint simultaneously. Its successful intermediate predictions do not identify each contribution, or prove that extra instructions cannot hide misses. The late-grid errors are material; their extrema are not a confidence interval for compact30.

The whole-decoder release ratios cited in the brief likewise show those versions' outcomes, not a controlled instruction-only intervention. They cannot establish that instructions never buy time. I did not replay the release-scaling bundle in this review.

## 2. Construction is inside the model's boundary, but compact30's cost is not measured there

The old harness explicitly reports “T-vector phase only (build + walk).” The LL input is prepared outside that phase; u builds its three-byte links directly rather than receiving a free converted production table. Thus g must cover the complete measured inverse phase, and X inherits building costs too.

Section B obtains compact30's g from CI whole-decoder time increases11.4% and13.7%, assuming the inverse phase is70% of baseline time:

g = 1 + delta / .7 = 1.16286 to1.19571.

This is an inferred residual from EPYC9V74/7763, not an i9 single-worker build+walk observation. It requires unchanged non-inversion work and a valid same-host phase fraction, then transfers both the residual and a changed production baseline across hosts. It conceptually charges construction as part of the changed inverse phase, but cannot establish its separate cost or concurrent inflation. The i9 four-worker .840 CPU/row evidence is walk-only. The older v2 builder capture failed its validity gate. Neither supplies this missing denominator.

The candidate at ba94ff5c uses Ctor2c: v2's U2 scanner, split descriptor arrays, v3's epoch-tagged finaliser, plus canonicalisation. Reusing an earlier v2 build factor as the cost of this complete candidate is invalid.

Independent phase arithmetic demonstrates why “loads × X” needs qualification:

| production workers | effective build inflation | effective walk inflation |
|---|---:|---:|
| 1 | 1.000 | 1.000 |
| 2 | 1.033 | 1.006 |
| 4 | 1.321 | 1.551 |
| 5 | 1.903 | 2.155 |
| 6 | 2.841 | 2.980 |
| 8 | 5.191 | 6.661 |
| 10 | 7.542 | 10.422 |

Each entry is W times the recorded phase's median seconds per worker divided by its one-worker value. These are effective phase inflation factors, not a claim of isolated cache-miss latency or critical-worker equality. Medians of phases need not add to the median total.

Reducing navigation loads by k does not justify reducing builder contention by k. A transferable model needs separate build and walk residuals (plus unaccounted scheduling/assembly), or an explicit empirical test of their joint transfer. For compact30's a=1 this is still a kernel-transfer assumption. For proposed k-step a<1 it is a potentially large unearned credit.

Construction and retained storage to charge:
- scan the complete LL; open/fill/finalise descriptors, epoch snapshots, ordinary fields and RAW groups; canonicalise RAW order; validate/fallback where applicable;
- install/save segment starts, walk ordinary/RAW/START paths, park/resolve RAW arrivals, grow/finish segments, restore marked fields and assemble output;
- rec:64 bytes per30-row group, including the source's extra-record allocation; raw:128 bytes per RAW group; bitmap, epoch snapshots of256 u32s, descriptor arrays;
- segment capacities and outer allocation, starts/next/saved/succ, lane state and8KiB staging tile; retained fallback capacities if exercised;
- at the decoder boundary add LL/MTF/Huffman/input/output/queues and worker high-water capacities.

The nominal1.95MB is a representation footprint estimate, not the complete worker's retained capacity or active cache footprint. Seven times1.95MB is a useful model coordinate, not a proof seven complete workers fit a cache. Record actual block lengths, RAW fractions and capacities, including heterogeneous blocks and retained high-water state. Capacity, touched bytes and DRAM traffic are different quantities.

Compact30's ordinary field/header loads share a64-byte record; RAW needs a further payload access. Three scalar load instructions do not mean three random cache lines, and a=1 does not mean literally one load instruction per row. Charge dependency depth, distinct lines and RAW/tail distribution separately.

## 3. What seven workers must deliver

Under s=1.95MB, a=1 and g=1.18, linear interpolation predicts:

| workers | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---:|---:|---:|---:|---:|---:|---:|
| model MB/s | 1763 | 1995 | 2188 | 2350 | 2327 | 2229 | 2065 |

This is a candidate selection calculation. There was no compact30 timing in the replay. The register's three-team reference2430–2584MB/s comes from its separate archive check; those three-team cells are not in these420 rows.2350 is3.3–9.0% below that range, not a demonstrated tie. Equivalence needs a prospective margin and matched comparison. The u24 residual range cannot supply it.

At seven workers and the same X:
- to reach2430MB/s needs g≤1.12697;
- to reach2584MB/s needs g≤1.03355;
- g=1.30,1.50,2.00 instead predicts2188,1963,1561MB/s.

The model has only a small margin for extra intrinsic work before it loses to the team reference. A nominally cheaper builder can matter; its full cost must be measured/count-priced on the exact candidate, not inferred from the failed v2 capture.

**Whole-decoder price:** none of these numbers is decoder throughput. For completed bytes B, the relevant worker service time is the maximum over workers of the sum of their complete block costs, including build, walk, decode and assembly, plus exposed scheduling effects. Throughput is additionally bounded by supply/scanning and output service. Seven ordinary workers and three teams of three consume different numbers of cores; preserve both the target absolute rate and resource cost. Report CPU/output byte, wall, block distribution and the busiest worker, not only average build/walk seconds. A kernel gain has value only where the decoder can continuously supply it and where its additional workspace does not restore contention.

## 4. Separate recipe; bounded information before a larger grid

#326 is NOT FROZEN at inspected head482df588. It asks G2/G9 and their interaction in today's whole decoder after recompressing the same content, not compact30 versus production on the original inversion corpus. It has no one-worker anchor or seven-worker cell. Its future outcome cannot qualify a compact table or identify table capacity alone.

Its schedule is61 rounds ×(10 cells+2 sentinels)=732 slots. Adding compact30 at the five worker counts for only level9 costs305 more slots (+41.7%); at both levels610 (+83.3%). Seven-worker and one-worker controls add more. Source build, schedule digest, mode/sentinel meaning, analysis and deadlines must all change. The draft can be revised prospectively, but this is not a free second arm. Its0.7s/slot development-VM estimate is not an i9 wall-time guarantee.

**Prefer a separate versioned diagnostic, sharing vetted preflight/bundle machinery only.** Proposed scope, not a frozen registration:
1. Untimed adapter and correctness/count/capacity check of the exact existing C kernel and matched ordinary control. The archived coop.rs has no compact30 arm; ba94ff5c has no matching coop.rs. A common frozen harness is required. Preserve the candidate kernel, identify any adapter changes and obtain the applicable independent safety review. Compare identical LL/counts/primary indices, segments, output, corpus and allocation policy; account for fallback.
2. Five-cell decision pilot: P1,C1,P7,C7 and a matched three-team-of-three reference. This calibrates g and tests the predicted seven-worker opportunity against same-harness controls.20 measured rounds plus one recorded warm-up would be105 kernel slots before any separately specified sentinels/preflight. This is a proposed cap for a descriptive pilot, not a power calculation or confidence/equivalence promise. Researcher assessment and Coordinator registration must fix the exact repeat count, order, information objective, exclusions/refusals and deadlines before data.
3. Distinguish this from validation of X's full curve. A fresh p grid at1,2,4,5,6,8,10 plus P7,C1,C7 and team9 is11 cells (231 slots at the same illustrative21 rounds). Do not call the five-cell pilot a new fit of the full law. If6/8/10 compact30 are wanted, choose those additional cells prospectively; do not search the best worker count after seeing seven.
4. Compare build+walk end to end, with disjoint phases and unaccounted time recorded. The existing sweep builds records outside its walk interval; that sweep alone cannot answer this question. Keep historical p versus current p explicit: a new common source head needs its own calibration. An old-p/new-C comparison silently credits all intervening source/harness changes.
5. Freeze power-state/gap treatment after #318's already-routed regime question; preserve all attempts and report modes rather than retrying into one. No workstation run is authorised by this proposal. A later whole-decoder paired diagnostic is separate and conditional on useful pilot information.

This is cheaper than inventing another representation, not zero-cost instrumentation. Do not port compact30 into a new production default. #195 remains FAIL; its restriction on further production builds unless the sibling-pair matrix reopens the line is untouched. The proposed historical-kernel diagnostic cannot itself waive that restriction or qualify adoption.

## 5. D4-delta and next marginal investment

**No change to the registered D4 filters:** walk≤15Ir/output byte, build≤7.35Ir/row and total representation budget≤1.2MB, with the already-required tails, directory, validation and complete loop costs. Loads/row are additional descriptive evidence, not a substitute gate. If another future registration prices a slower but smaller construction, it must be prospective; it cannot rewrite the D4 verdict.

Report at least scalar loads, distinct requested lines, dependent load stages, prefetch/RAW arrivals and exposed latency evidence separately. Neither instruction reduction nor fewer logical table accesses proves whole-decoder speed. RULES C4 already makes that distinction; this fit does not create new authority.

Rank:
1. Complete the already-routed regime diagnostic and default-team census follow-through; preserve service/emitter work and the scoped #324 geometry rescue. They explain the current production binding path.
2. Compact30's separate five-cell diagnostic is a useful inexpensive test of an existing smaller representation once the matched harness/count/capacity prerequisites are ready. Its information value exceeds a new speculative kernel or an unqualified large extension of #326.
3. Keep D4's bounded count probe; no new D4 timing or altered thresholds on the strength of this extrapolation. k-step projections require separate builder pricing and dependency accounting.

Expected versus found: the register's≈2350 arithmetic holds; “measured construction included” and “tie” do not. Gap closed: model provenance and builder-boundary uncertainty are now explicit, but zero new measured performance gap was closed. Mechanism learned: the fitted congestion term penalises building materially, so scaling it by walk loads alone is not yet defensible. Next value: measure the complete existing kernel against matched controls, without conflating that information with a production reopening.

Migration checked before receipt; receipt5ab9bb2d and substantive checkpoint539b2792 precede this completed report. Existing history retained. No constructor, timing, merge, gate relaxation or adoption action taken.
