# Independent review: compact30's first valid four-core traversal capture

**Evidence:** the user-supplied `c30-traversal-20261102T200732Z.tar.gz`, capture on 2 November 2026. Analysis performed from its raw JSONL, the archive's machine-code text and manifest, plus the pinned protocol and example source. **No benchmark, decoder or PMU capture was run in this review.**

## Disposition

**ADVANCE, within the registered experimental boundary.** The capture is complete and meets every registered gate, and I independently reconstructed the decision metrics from the individual walks. This authorises *separately registered direct-constructor feasibility work*, not production integration or a claim of full-decoder speedup.

## Integrity and identity

- Uploaded archive SHA-256: `4764393e1d07c426ed5c57acc40712cd1e4c548672a0ebc1aef3c114ec635444`, identical to the value reported by the workstation.
- Manifest names clean source pin `f1dafa31d88d7351a98afb005a84f23d5a2312bd`, expected Wikidata corpus SHA-256 `cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe`, exit status 0, and binary SHA-256 `ccaf37ad21fb4c171caaa39304f815b795ddad8e4fa85a9922ee7aaa867dc9ae`.
- Every one of the eight archived payloads matches its manifest SHA-256 **and** byte length.
- The executable itself and corpus were not included in the archive. Their manifest hashes and recorded checking procedures are verifiable as provenance statements, but the underlying bytes cannot be independently rehashed from this upload.
- Archived `room-false.diff` is empty. `room-false.bounds` says 1,552 instructions are compared over the function's true `0x20aa` byte extent, with a permitted external alignment NOP. The recorded hot path remains 21 instructions (8 on address/classification + 13 on continuation). This review inspected the recorded assembly but did not rebuild the executable.
- Toolchain recorded as rustc 1.97.0 / LLVM 22.1.6; four pinned CPUs are 1, 2, 3, 4. The recipe is on branch checkpoint `5e2888f`, and selects the frozen pin in its own worktree; these are distinct by design.

## Independently reconstructed capture structure

- 1 meta, 2 complete calibration batches, 36,456 walk records, zero failure or unknown records.
- 434 source blocks; 207,006,515 BWT rows. Of these, 230 large blocks have 206,998,191 rows, while 204 short blocks account for only 8,324 rows.
- 20 measured paired rounds, plus warm-up round -1; every block appears in each arm and mode exactly once each round.
- Reconstructed snake assignments exactly match metadata. Per-worker row totals are `[51,301,938, 51,301,991, 52,201,043, 52,201,543]`.
- Independent checks passed: complete expected `(round, mode, arm, worker, step, block, rows)` set; exact CPU before/after; per-walk preparation/timing/check ordering; synchronized preparation and completion boundaries; no step overlap; pass/arm order.
- Every calibration has exactly the specified four CPUs, each with 20,000 samples and ordered five-quantile summaries. Largest median empty CPU window = **149 ns**.
- Control stability ratios (maximum round/minimum round): W1 **1.00654**, W4 **1.01005**, E4 **1.01009**, all below G1's **1.05**.
- Across all 40 measured arm-rounds, maximum row-weighted start-skew ratio is **0.000231 = 0.0231%**, median **0.00710%**; G2 allows **2%**. The separately registered spinning-barrier correction has removed the prior sleeping-barrier failure mode.
- Maximum estimated clock-overhead share across the four passes and 20 rounds is **0.0002565 = 0.02565%** against G5's **0.5%**. This is a *median-calibration estimate*, not a bound on every call.
- Independently recalculated geometric means and 95% paired t intervals (19 degrees of freedom) match the archived analysis to within rounding.

## Registered results

Candidate/control cost ratio; values above 1 are slower for compact30.

| Metric | Ratio | 95% interval | Reading |
|---|---:|---:|---|
| W1, CPU time per row | **1.14845** | **[1.14743, 1.14948]** | compact30 costs **14.85% more CPU per row**, single core |
| W4, sum of four threads' CPU time per row | **0.84047** | **[0.83781, 0.84313]** | compact30 saves **15.95% CPU per row** |
| E4, sum of lockstep step-completion windows | **0.85601** | **[0.85239, 0.85965]** | compact30 reduces the registered completion time by **14.40%**, ~**16.82%** more work per unit of this time |

Both ADVANCE conditions pass: `W4.upper < 0.865`, `E4.upper < 1.02`. All **20** round-by-round W4 ratios are <1 (range **0.83195–0.85591**); all **20** E4 ratios are <1 (range **0.84786–0.87946**). All **20** W1 ratios are >1.

Mean absolute levels across measured rounds:

| CPU cost / completion | Shipped P | Compact30 C |
|---|---:|---:|
| W1 CPU ns / BWT row | **1.22061** | **1.40182** |
| W4 CPU ns / BWT row | **2.05499** | **1.72717** |
| E4 completion over all 109 steps | **114.449 ms** | **97.973 ms** |

Worker CPU duration is not conflated with wall duration: mean CPU/wall running fractions are about **99.9%** in both arms/modes.

The loss from running four simultaneous walkers versus a single walker is larger for production (W4/W1 CPU cost **1.684×**) than compact30 (**1.232×**). This is a descriptive cross-condition comparison, not a controlled isolation of cache capacity, CPU frequency or memory traffic.

## Is the win broad or concentrated?

- **All 230 full-sized blocks** are faster with C on W4 when each block's CPU time is averaged across 20 rounds. Full-block W4 cost ratio: **0.84043**; median individual block ratio **0.84763**, 10th–90th percentile approximately **0.814–0.865**.
- Of **1,140** measured full-size four-worker steps (57 per round ×20), C uses less aggregate thread CPU time in **1,138** and less completion time in **1,137**.
- Full-size steps account for **98.76%** of production's E4 completion time and **98.58%** of compact30's. Their completion ratio is **0.85446**. There is one mixed-size step, and 51 short-only steps per round.
- On short blocks, C is actually worse: W4 aggregate CPU ratio **1.266**, W1 ratio **1.329**, but those blocks have only **8,324 / 207,006,515 rows = 0.0040%** of useful rows. Their contribution to E4 time is tiny (~0.027 ms P versus ~0.032 ms C per round).
- The W4 ratio changes modestly with arm order: P-first **0.83849**, C-first **0.84248** (arithmetic means of paired round ratios). E4 likewise **0.85222** versus **0.85988**. Both positions strongly favour C.

**The full-block advantage is not an artefact of an outlier, the tiny blocks, one alternating arm position, or an unbounded clock-calibration correction.** These robustness checks all use the registered capture, not additional trials. The source cases repeat across rounds, so block counts are descriptive checks, not 230 independent experimental replications.

## Memory/layout account

Using the archive's per-block rows and RAW-group counts, and the actual prototype formula `64 × (ceil(n/30)+1) + 128 × RAW_groups`, I obtain **448,002,944 logical bytes** across the whole corpus, **2.1641973 bytes/BWT row**, against **4 bytes/row** in the shipped T-vector: **45.895% smaller logical navigation storage**. Across a maximum-sized 900k-row block the typical compact arena is approximately **1.95 MB** versus **3.6 MB** shipped, or roughly **7.79 MB vs 14.4 MB** of navigation arrays at four simultaneous workers. This is **not measured RSS, cache-hit rate, or measured DRAM traffic**; the current RAW groups are **padded to 128 bytes**, not the theoretical dense 120-byte alternative.

## Construction-budget implications

The registered approximate phase model is

`S [CPU-s/decoded GB] ≈ 0.592(1 - b) + 1.847(1 - w)`,

where `b` is constructor CPU cost relative to production and `w` is traversal/assembly CPU cost relative to production. Using the W4 geometric-mean ratio **w = 0.8404655**:

- At **b=1**, predicted saving is **0.29466 CPU-s/GB**, above the **0.25** policy target by **0.04466 CPU-s/GB**.
- Maximum constructor cost ratio `b_max(w4_point)` = **1.07544**, i.e. about **7.5% slower** than production's builder.
- At the conservative **W4 upper 95% bound**, `b_max` = **1.06714**, approximately **6.7% slower**.
- At the optimistic lower bound, `b_max` = **1.08371**.
- There is **no measured compact30 direct-constructor time**. Its current oracle-built table and copy are specifically excluded from the traversal timer.

This is an **engineering investment budget**, not a validated full-decoder prediction. The model's phase coefficients are approximate, and the measured W4 ratio may not transfer unchanged after introducing a direct constructor and the real decoder's phase mixture.

## What the experiment demonstrates and what remains unresolved

**Demonstrates:** under the fixed prepared-table workload and simultaneous four-walker scheduling on this workstation/corpus, compact30 reduces both CPU traversal cost and lockstep completion time despite losing by ~15% on one core; the specific multi-core win is unusually broad and stable.

**Does not demonstrate:** why compact30 wins (smaller arena, cache service, TLB, load issue, construction-induced cache state), a faster complete decoder, constructor affordability, one/two/three-worker benefit, other corpora, actual memory-footprint/RSS savings, or the cache state of a direct builder. Control builds its actual T-vector immediately before the walk; candidate copies a prebuilt oracle table. Both preparations are untimed, but they leave different caches warm.

**External-validity warning:** four walkers are forced to overlap with identical arm and no build/copy inside the walk windows. Real decoding mixes BWT work with MTF, construction, output and queue activity and does not maintain four simultaneous walks. Moreover, the older phase study's four-worker shipped traversal+assembly was ~1.806 CPU ns/row on a matched full-block subset, whereas this lockstep control is ~2.055 CPU ns/row on full blocks; these are different protocols/populations, not a controlled causal comparison. This highlights how the stress conditions and their extrapolation matter.

**Single-core guardrail:** at one worker compact30 costs 14.85% more CPU, and short blocks are also worse. A production default cannot be inferred from this four-core ADVANCE. A later decoder integration would have to preserve acceptable low-concurrency performance, potentially via conditions/dispatch, without assuming the four-core benefit applies to idle cores.

**Next authorised investment:** evaluate the *direct-constructor feasibility and CPU budget* in its own registered step. Preserve this archive and the frozen traversal baseline. Do not tune the walk, claim the cause, or promote compact30 into the production default on the strength of this test alone. Subsequent integrated testing must include build CPU cost, actual cache state, completion throughput and memory footprint.

## Source pointers

- Protocol and capture history at branch checkpoint [`5e2888f`](https://github.com/lmmx/mibz2/blob/5e2888f/docs/journal/2026-11-02-compact30-traversal-protocol.md).
- Frozen collector and compact30 walker [`f1dafa31…`](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/examples/c30walk.rs).
- Lockstep spinning start barrier in that file, plus `std::sync::Barrier` completion barrier. Neither clock includes the barrier wait.
- Earlier registered decoder-stage attribution, [corrected BWT memory proposal](https://github.com/lmmx/mibz2/blob/0156e1dad5ce4ebca78b4cb3d00276d3b6d64611/docs/journal/2026-11-01-bwt-memory-proposal.md).
- Independent numeric reconstruction: `independent_stats.json`, reimplementation in `recheck.py`. Source archive remains unmodified.
