# Run-fed STOP: remove costs without inventing savings

Brief [6730769410](https://github.com/lmmx/mibz2/pull/160#issuecomment-6730769410). Specs a4f97714cc14366984015869d9d094d89488ac57. Stage source **fd60fffada25d475bbcc2b54165b96cf5fd7a92f**; [result journal](https://github.com/lmmx/mibz2/blob/66edecbe2b1924594e485e1f956dbee8ed9beda2/docs/journal/2026-11-09-run-fed-stage-result.md); direct/helper source **ff6c2f74677af80ea9813bd081d34a68d07e6f1b**.

## Decision and evidence boundary

C31 remains the first construction, but **sort removal alone does not predict a one-worker stage win**. First isolate the fallback anomaly and emitter instruction difference; keep the cheap record format until those counts justify a representation change. Next build a prospective block-cost selector from literal/run statistics, preferably collected during existing symbol decoding. Full Huffman/MTF fusion is last: it changes the pipeline's serial service, not merely a store.

The registered journal reports run42549324025 VALID, Q1 STOP, R_stage1.1592 [1.1387,1.1801], failing synthetic/fallback screens. Retained unchanged. I inspected source and published summaries, not the raw artifact or PMU records; no independent validation/replay of that run is claimed. Nominal c/B is wall×sampled clock, not counted cycles. No implementation, compilation, timing or safety certification in this review.

D049 holds #270 timing, including the frozen primary dir. The k=66 boundary witness is outside the3×3 plan, but composition/recovery evidence gaps apply to that plan too. Repair and untimed validation continue; no construction proposed here lifts that HOLD. The brief's §11 reference is not in the authenticated protocol pin (which ends at§10); this research fits the existing Optimiser mandate without importing it.

## 1. Arithmetic: the sort is necessary, not sufficient

From the displayed phase medians, in nominal cycles per **total decoded output byte**:

    A = 1.290 + 1.432 + 0.971 = 3.693
    B = 1.508 + 0.317 + 1.332 + 1.071 = 4.228
    B−A = +0.535
    no sort, unchanged fill = +0.218
    also remove fallback gap = +0.118
    also remove emitter gap = −0.100.

Thus even an ideal free replacement for sorting leaves a loss. If delta_d is direct fill/setup minus the present1.332 fill, e is recovered emitter cost and f recovered fallback cost, the phase sensitivity is:

    delta_stage ≈ 0.218 + delta_d − e − f.

With f=.100 and e=0, direct placement must save another .118; with e=.218 and f=.100 it can afford less than .100 extra direct overhead. These are **budget sensitivities**, not predicted observed ratios or new acceptance thresholds.

Entry medians are3.676/4.291, a gap.615 rather than.535. Subtracting all three claimed losses (.635) from B's entry suggests only.020 nominal c/B, while phase sums suggest.100. Neither is a valid recomputed paired outcome: separate sweeps, brackets, medians and covariance differ. R_stage is the paired geometric-mean statistic, not4.291/3.676. Likewise1.508/1.290 is about1.169, not the journal's separately computed R_front1.185. Do not force these summaries to agree by changing the verdict.

The table's c/B denominator is decoded output length in examples/run_fed_stage.rs, not accepted BWT rows. The79.8% accepted share is BWT-row weighted. It cannot directly rescale these phase medians into accepted-row costs without the actual row/output counts.

## 2. C31: what direct placement really costs

[fill_direct_at](https://github.com/lmmx/mibz2/blob/ff6c2f74677af80ea9813bd081d34a68d07e6f1b/src/parallel/run_team.rs) keeps lcur[256]. A symbol's first interval in a share does a predecessor search; subsequent intervals resume that symbol's L cursor. Each symbol's F intervals are adjacent in its bucket, so resumed cursors do not rescan from zero. The removed work is F-record counting/scatter/materialization, not all prefix/validation work.

The same-pin journal's **bench-corpus**, two-helper C35 census gives163 searches/block, r/n=.0620 and mean n899,995 over186 team blocks. This is distinct from the2,614-search small-file smoke. The one-worker lookup count for the384 accepted stage blocks was not supplied. At most256 searches per single share; the two-helper163 is not its measurement.

A useful lookup-only bound/model: r≈55,800 in a full team block; binary predecessor search needs roughly16 comparisons/probes.163×16/900000≈.00290 probes per BWT row. Assuming an *amortized exposed* d cycles/probe gives .00290d c/BWT-row: .029 at d10, .145 at d50. Those are deliberately hypothetical sensitivity points, not measured cache latency; comparisons, branches, prefix scans, checks and cell traversal add cost. No conversion to the journal's output-byte denominator is made.

More consequential than the initial searches may be interleaving up to256 destination streams, keeping per-byte cursors live, reloading L records, ownership-edge scalar tails and helpers sharing cache lines. Current sorted fill is globally sequential; dir is sequential within byte streams. **1.332 is the sorted fill, not a measured no-sort fill.**

Cheapest falsifier: after D049 closure, a registered matched single-worker dir-versus-sort+fill diagnostic with the same checked records, counts, buffers, following walk and total stage. Before timing, archive native paths and dynamic counts of searches, cells, prefix work, stores and ownership tails. Reject the claim “sort removal is enough” unless delta_d−e−f<−.218 in a matched phase model and the actual paired entry confirms it. Preserve the official gate separately.

## 3. Rank emitter constructions by work, not stored byte width

The source AVX2 emitter uses the same register-list MTF machinery as plain MTF; it tracks n and record index m, shifts n by8, ORs the byte and writes u32 per literal. Runs advance n and counts without filling L. At r/n=.062, record payload is about **.248n bytes**, already below a full n-byte column. “Four-byte store per literal” is not four times total traffic, nor proof that stores cause the entire+.218. Added integer state/dependencies, register pressure and code layout also need isolation.

| Rank | Construction | Conditional nominal c/B price | Risk / cheapest falsifier |
|---|---|---|---|
| 1 | Keep u32 records; minimize emitter address/index/packing work with matched modern-list control | Recover e from the observed+.218 gap; e=.218 is a target, not a demonstrated removable amount | Native loop/register/spill and per-literal instruction counts first. Run and literal checks/count updates must remain. Reject a claimed store bottleneck without load/store/issue evidence |
| 2 | Literal byte stream plus sparse run-extension descriptors | Replace4l literal-record bytes with l bytes plus k*h descriptor bytes; front changes by packing saved minus descriptor/control cost | The fill still needs L-order starts and predecessor access. Include reconstruction/index build and fill changes; reject if they repay the front saving |
| 3 | More tightly packed run records | Potential memory reduction, unknown cycle benefit; cannot credit a fixed fraction of.218 | A20-bit start+8-bit byte needs28 bits for900k; three bytes do not fit. Variable deltas need restart index and dependent decoding. Listing/lookup count before a timing arm |
| 4 | Fuse MTF emission into Huffman decoding | Removes intermediate symbol stores/reads and loop boundary; no defensible c/B forecast from this stage-only run | Moves MTF work to scanner, adds state/register pressure and changes overlap/errors. First a pipeline service-demand bound; whole-decoder evidence required |

For a literal stream/extension representation, give each literal one byte and record only the extra RUNA/RUNB length attached to that byte, plus a leading-run case. With k-byte descriptors and h extended literals, traffic is l+k*h versus4l, so h<3l/k is a necessary payload saving, **not a speed threshold**. Eight-byte descriptors require h<.375l. The source has not supplied h. A full-column-plus-run-index variant writes n plus metadata, trades cheaper literal emission for run expansion, and may help a plain dense build; it does not retain the record fill for free.

The cheap *complete* emitter today may remain u32 records: they already provide random-access starts to C31. A hybrid that later expands them back has shifted work rather than removed it. Prefer a bounded descriptor/index proposal only after a census of literal count, extensions, lengths and lookup requirements shows a net budget.

Fusion into Huffman is particularly risky for three teams: the scanner currently produces symbols for downstream work. A faster fused front in isolation can starve all teams if it raises serial scanner service demand. Collecting a handful of admission counters there is much smaller than moving the entire MTF recurrence.

## 4. Fallback: the timer rules out the simple story

[stage and entry](https://github.com/lmmx/mibz2/blob/fd60fffada25d475bbcc2b54165b96cf5fd7a92f/examples/run_fed_stage.rs), [takes_runs/build](https://github.com/lmmx/mibz2/blob/fd60fffada25d475bbcc2b54165b96cf5fd7a92f/src/run_stage.rs):

- takes_runs is the constant-work predicate max_len<2^24 &&8*syms.len()<=max_len. It is not a symbol scan.
- entry B calls it inside the judged bracket; rejected blocks go straight to plain mtf then bwt::build. Accepted-then-invalid-sort can redo MTF, but validation asserts B's actual path equals takes_runs. That is not the explanation for these47 ordinary rejected blocks.
- In the **phase** sweep, takes_runs and match selection occur before time(3). A and B fallback closures both call the same mtf and build. Hence+.100 in the phase table cannot simply be charged to timed selector work.
- Each arm retains distinct Bufs, and accepted blocks leave different histories: A warms ll on all blocks; B's accepted path uses records instead. Allocation growth, address placement, cache state, compiler call layout and order are plausible explanations. None is established by this source review. The entry R_fb also includes dispatch; do not confuse the two measurements.

First falsifier: a matched fallback-only control with precomputed path flags, equivalent capacities, swapped buffer identities and otherwise unchanged bracket/arm order. Count selector instructions separately and inspect native call paths. If the penalty follows buffer history rather than the branch, changing admission arithmetic will not remove it. Keep this diagnostic distinct from a production-shaped pass that preserves real histories.

Zero **wasted emitter work** on predetermined reject paths already holds. Zero mathematical overhead is not promised for a conditional dispatch. Hoist an immutable decision to scanner metadata and share the same fallback call site where beneficial; charge its acquisition and subsequent cache history in total decode. Do not erase the measured screen failure by subtracting an estimate.

## 5. Admission: use actual lengths and operation counts

The current rule uses syms.len()/max_len, neither actual r/n nor actual n. Short partial blocks can look artificially run-heavy. RUNA/RUNB encode length logarithmically; symbol count mixes run-code bits and literals. Two blocks with the same r/n can have different numbers of long runs, cell crossings, tails and MTF ranks.

Before byte identities are known, the symbol stream already permits computing n (bounded run-weight accumulation plus literals), literal count l, leading-run presence, extended-literal count h, run-length bins and long-run output share. Ordinary valid-map literals change the MTF front and start a new L run; malformed-map behavior must retain validation. Statistics can be accumulated during existing Huffman symbol emission, with overflow/error semantics preserved. A separate pre-MTF full scan is not free; “available” does not mean already stored.

Proposed C41: a small prospective cost predictor, not a newly tuned threshold:

    gain_hat = avoided column/run expansion + avoided dense-build work
               − record emission − direct prefix/search/cell work − dispatch.

Use n,l,h and length bins first; add symbol-rank tail counts if front costs matter. Exact F/L intersection cells and byte distribution require MTF identities, so are post-emission attribution features, not free pre-admission features. Count W/cells when available to test the model, but R_floor does not justify making W the sole predictor.

Freeze coefficients and decision threshold on separately identified exploratory data, evaluate on fresh registered paired data; do not select a threshold from these seven observed corpus outcomes and call the same run confirmation. If admission occurs during MTF, use a bounded prefix with one-way fallback and a resumable representation; include sunk emission and conversion costs. No restart of the whole block marketed as zero-cost fallback.

Cheapest falsifier: untimed per-block n,l,h/bins census and matched archived per-block costs where available; aggregate corpus timings alone cannot fit a reliable block selector. Then registered held-out whole-stage comparison, keeping fallback/synthetic screens unchanged. One-byte/pattern gains of36% support a special-case opportunity, not a threshold at r/n=.18.

## 6. What transfers to #270 and the whole decoder

#269 measures serial emit+sort+fill versus plain MTF+build. #270 uses a checkpointed production baseline, helpers, a different schedule, C35 partitioning and direct placement. Its critical completion is approximately max(leader completion, latest helper completion), with overlap and publication charged—not the sum of #269 phases. Removing leader sort can matter even when total service rises; speeding an already-early leader may buy nothing when helpers bind.

C31/C35 therefore remain plausible. The census's1.007 work-weighted store imbalance is not a worst-block readiness bound. Prefix gates, cell/control work, share-edge tails and false sharing may dominate the last helper. Use common-origin ready timestamps and matched interventions, not importing.317 or1.332 wholesale into the helper model. D049 forbids timing until its scoped repair/evidence closes.

R_floor=.321 and R_warm=1.010 weaken the simple “unavoidable table-store floor” explanation. They do **not** uniquely identify instructions versus latency, or exclude store-address dependencies, port pressure, overstore effects or different cache histories. Store-only reference and pre-touch are interventions with their own work; PMU/listing evidence must locate the next removable dependency.

Expected versus observed: the sorted run stage was expected to remove column/build work; observed fill saves.100 in phase summaries, outweighed by front/sort/fallback. Gap closed by this review: no new measured cycles, but sort-only optimism and fallback-selector attribution are corrected. Next value: fallback-control diagnosis and C31 direct matched work, then emitter reduction; packing/fusion remain lower priority. Existing service safety closure/C29 controls still offer lower representation risk than a new hybrid. The current direction is not disproved, but a single-worker win is not yet economically supported.

Migration SHA/digest verified; routing audit found only five explicitly resolved historical issues. Publication uses route/check and exact author/body/destination readback; pinned verify hardcodes coordinator and is not claimed to pass for an Optimiser message.
