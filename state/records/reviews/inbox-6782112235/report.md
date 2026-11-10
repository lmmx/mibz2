# Cells stage-to-decoder gap: recommend one live-path diagnostic

Assignment [#160/6782112235](https://github.com/lmmx/mibz2/pull/160#issuecomment-6782112235). Reviewed 2026-11-10 under SPEC + SPEC.OPTIMIZER at a2da49638ac54d7763de1c167bd054f67ffdc763. Proposal only; no implementation, decoder execution, compilation, new timing, default/cap changes, merge or B5 disposition.

**Decision for 2026-11-11:** prepare the bounded **cells-live-path-gap** diagnostic below, rather than choose a production change from these data. The two measurements are compatible but do not identify the cause of the gap. Admission coverage alone is contradicted as a sufficient explanation by the archived i9 replay. The best next value is to measure bounded Cells in decode order on the same host as its baseline, with route and allocation accounting. Do not rerun #365 or relabel its results.

## Evidence and reproduction

- #362 design and HANDOFF at **7611f52bd91c68d3b2c8a224cf7672bc1c69a312**, especially admission, refusal cost and off-default scope.
- #365 frozen **57f51bbc0ca62ac3ee1cc148a1010842eea6222a**: src/cells.rs, src/bwt.rs, src/lib.rs, src/runs.rs, src/read.rs; examples/ab.rs, ab_cells/mod.rs, phases.rs; qualification HANDOFF.
- i9 source **c79ff7d384072e5e95eda7e513227f2aa06ff3d5**, examples/rauzy.rs and .just/bench.just. Bundle [evidence d85b2074](https://github.com/lmmx/mibz2/blob/d85b2074099f027d048aa7a057da10d25f2b70bb/cells-vs-prod/20261110T175613Z.tar.gz), SHA-256 **552e26aaf3b41636afdbd5a195ad5f0ce1c66aed945ec2911a508057baed705c**, downloaded and verified, all three JSONL censuses replayed.
- Final [job 130128016844](https://github.com/lmmx/mibz2/actions/runs/42901406328/job/130128016844) decoded log read. Artifact **13802703606** downloaded and verified: SHA-256 **c36061ac6a7e8ac2991acbd82e2935fd20f9a3d075e102bd42c67b46cb575141**. Read cells.jsonl, stages.jsonl, env.json and cells-summary.md. This independently verifies archive bytes that the original handoff could not download.
- [Arithmetic](arithmetic.json) and [archive replay script](replay.py). Extract the tar to cells-raw and ZIP to cells-ci, retaining original archive filenames cells-evidence.tar.gz and cells-final.zip; run the script. It only calculates from archived records.

Raw paired wall geometric means reproduce all reported experimental ratios (10 pairs/corpus); every paired output hash matches. Reported intervals retained, not independently re-bootstrapped:

| Corpus | Replayed ratio | Reported 95% interval | Default→cells peak RSS, rounded MB |
|---|---:|---|---|
| Wikidata | 1.000093 | [0.990,1.011] | 26→34 |
| text | 1.012133 | [0.980,1.046] | 19→19 |
| random | 1.006498 | [0.999,1.014] | 23→23 |
| records | 0.996952 | [0.986,1.008] | 23→23 |
| runs | 1.004255 | [0.983,1.026] | 23→23 |
| onebyte | 1.109001 | [1.095,1.123] | 33→36 |
| pattern | 2.231663 | [2.217,2.247] | 16→19 |

Pattern's 123.2% excess and onebyte's 10.9% excess remain real findings of this experiment. Their respective approximate wall penalties are 32.6 ms and 2.0 ms using the reported rounded baseline medians; these are not phase attribution.

## What was actually compared

The i9-10900K recipe pins CPU2, makes three census processes, and invokes rauzy with every=1, minimum rows=65536, rounds=5. Each block reports the **minimum of five** warmed stage/whole spans. Cells uses the example's unbounded induction with counting disabled in the timed call, then counts cells separately. It consumes reconstructed records outside the clock. Production ns_rec is fill plus backward room walk from prebuilt run records; front is measured separately. It is not a full decoder comparison. Independent phase minima do not sum to a measured whole.

CI runs separate executables on EPYC 9V74, Rust1.99.1, CPU1, ten paired fresh-process decodes with 0.25 s settling. The cells arm is sequential CellsReader; default at threads=1 is sequential BzReader, despite the JSON adapter-capability field parallel=true. Both copy decoded output into the drain buffer and hash it; there is no demonstrated extra full-output copy unique to CellsReader. Distinct types/binaries can still change code generation. The former default-harness LTO coupling described in HANDOFF was isolated before this frozen head; it is not an established cause of this final cells result.

**Corpus mismatch is substantially ruled out:** both pinned manifests are identical and name compressed SHA-256 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915, 11,854,933 compressed bytes and 209,700,196 decoded bytes. Both build logs report that size and SHA check; CI raw output is 209,700,196. The census contains 209,697,662 pre-RLE1 rows, not decoded bytes. Its minimum-row selection and different boundary must still be recorded; do not equate row counts with decoded bytes or claim CI route counts from this census.

**Do not take f from stages.jsonl.** Frozen examples/phases.rs:62–107 explicitly executes mtf::mtf, bwt::build and forward room walk, without takes_runs/front/fill/backward walk. It is a separate timed example, even though it checks bytes against decode. Its roughly 65% build+walk share is not the live records decoder's inversion share and its total is not whole-process wall.

## Admission replay and complete cost accounting

Apply frozen rules to the three censuses, using records count (127-row fragments, not coalesced runs), B=floor(4n/25), and the post-split c_letters:

| Predicted outcome in selected census | Blocks | Rows | Row share |
|---|---:|---:|---:|
| admitted | 196 | 176,398,965 | 84.1206% |
| density: 10r ≥ n | 35 | 31,498,722 | 15.0210% |
| floor: letters−1 > B | 0 | 0 | 0 |
| budget: cells > B | 2 | 1,799,975 | 0.8584% |

All 233 had takes=true, which is only the **outer records-path gate**. This replay predicts bounded Cells admission on the archived valid columns; it is not a runtime observation at the CI head. No corresponding per-block route census exists in cells.jsonl.

Pooled i9 order/induction/write minima are respectively **0.279–0.282 / 0.745–0.750 / 0.2307–0.2309 ns/row**; independently measured whole is **1.258–1.266**, production **1.531–1.544**, front **0.1401–0.1402**. Order and writer were already included in c_whole; charging them again to explain the gap would double count.

On admitted rows in repeat1, c_whole=1.11515 vs ns_rec=1.53267. Keeping refused blocks at baseline and assuming zero attempt overhead yields aggregate inversion ratios **0.77251 / 0.78204 / 0.77522**, better than the unconditional 0.815/0.827/0.818 because density excludes expensive cells cases. A merely illustrative linear-in-cell budget-prefix price gives **0.77956 / 0.78929 / 0.78227**, with about 0.0109–0.0111 ns/all-rows refusal overhead. Prefix time is not measured, need not scale linearly and is not a rigorous bound. These scenarios are not predictions for EPYC or production.

Frozen cost boundaries:

| Component | Work that must be paid |
|---|---|
| shared decode | input/Scanner/Huffman; takes_runs; front or MTF; finish/RLE1/CRC; output copies/hash; errors and termination |
| density refusal | count comparison plus dispatch/count bookkeeping, then unchanged Segmented inversion; no cells-buffer growth |
| order | validate record lengths/tiling, find/coalesce primary run, two record passes, 257-counter work, 48 bytes of slot initialization per letter, primary mark/sentinels, allocation/growth |
| floor refusal | **after allocation and full slot writes**, then entire baseline inversion; no refund |
| induction | check budget before each cell, dependent slot loads/prefetches, both speculative 16-byte copies and 16-byte rule stores even when one representation is discarded; unequal step has 48 slot-store bytes, equal step 40, hence 96 or 88 addressed store bytes/cell including word stores. These are source writes, not machine instruction counts or DRAM traffic |
| budget refusal | all order work plus up to B cells, then full baseline inversion; output untouched until writer |
| writer | pos initialization (4 bytes/rule), three marked-chain traversals, reverse/forward rule scans, short-factor and repeated-long-factor copies, rotation and short-cycle repetition, output resize/slack; no measured copy-byte census |
| persistent state | both retained cells buffers and fallback buffers may coexist; growth frees old cells buf before allocating replacement; allocator/zeroing/page costs are not bounded by RSS alone |

Release cells buf size is **48r+80B+4528**, plus pos at most **4B** requested bytes, so combined cells scratch bound **48r+84B+4528**, excluding allocator overhead, out, fallback, runs and input/output buffers. At n=900,000 and r<90,000 this is below 16.421 MB for cells buffers alone. It is not a whole-worker RSS bound.

In archived census order the frozen size formula predicts **10 increasing buf allocations**, 14,230,096→15,778,576 bytes, **148,399,232 cumulative requested zero-initialized bytes**; pos/output growth additional. This is a replay of requested sizes, not measured physical zero stores, page faults, allocator time, or the full CI allocation sequence. Example layout depends on n alone and its same-block minima hide first-use/growth costs. Bounded layout changes region distances/cache placement too: smaller address space does not guarantee faster hot code.

For periodic pattern, budget failure after costly work is a plausible explanation; for onebyte, order/writer/first-touch and a short baseline make fixed costs plausible. **Neither path is proved by the aggregate ratios or forced-fuzz admission counts.** The existing Counts.cells only aggregates successful admissions; it cannot measure discarded induction work or distinguish blocks' r/n/n/output sizes.

## Amdahl price and three ranked explanations

Let f be the live baseline inversion fraction of whole time, s its transported time ratio, and h additional whole-time fraction not already included in s. Then R=1−f(1−s)+h. With s=.815–.827 and h=0:

| f | Whole-time reduction |
|---:|---:|
| .10 | 1.73–1.85% |
| .25 | 4.33–4.63% |
| .50 | 8.65–9.25% |
| 1 | 17.3–18.5% |

The last row is an optimistic transported upper limit (speedup at most 1.227×), not a hardware-independent bound. The ideal admission-selected i9 scenario instead offers 21.8–22.75% of inversion time. Perfectly removing inversion would save at most f, with all other work fixed. No numeric live f has been established. Pure dilution with the original s would need f≤5.4–5.8% to shrink the gain to ≤1%; at f=.25, cancellation needs about 4.3–4.6% extra whole time. These are conditional prices, not causal estimates from the two machines. On multiworker/team paths f must be critical-worker/critical-path exposure; a one-thread result establishes no W4/W10 benefit.

1. **Example→bounded/live decoder transport costs** (highest value to test): validation, per-cell checks/counting, changing layout and cold growth versus warmed minima. Cheapest falsifier: same-input in-order route/allocation census followed by matched live-phase accounting with instrumentation overhead checked. If bounded inversion remains faster and allocation/refusal costs are too small to fill the measured gap, reject this as sufficient. Potential recovery ceiling is the measured live excess, never the old 18% whole-decoder claim.
2. **Admission/fallback mix**, particularly pattern: 84.12% predicted row coverage, only 0.858% budget-refused Wikidata rows; density rejection improves the ideal i9 mix. Coverage alone is already insufficient there. Cheapest falsifier: per-block outcome + attempted-cell/copy counts for Wikidata/pattern/onebyte. Zero budget refusals rejects that mechanism for a corpus; small measured attempt time rejects it as sufficient. Removing attempt overhead can recover at most measured excess (pattern's ~32.6 ms is a total-gap ceiling if returning to baseline, not a prediction of a better algorithm).
3. **Hardware and common-work exposure/code generation:** CPU/compiler/cache-state differences change s, while front/scanner/RLE/CRC/copy/hash dilute it. Same compressed corpus weakens a corpus-change explanation. Cheapest discriminating evidence is one same-host matched live-path experiment, not subtraction of i9 nanoseconds from CI wall time. If s is near1 on the new host, there is no inversion gain to propagate there; if s<1 but f small, use the measured Amdahl ceiling. Old stages and RSS cannot settle this.

## Exact diagnostic to prepare: cells-live-path-gap

One separate diagnostic branch/registration based on frozen57f51bbc; **no edits to #365**. Coordinator admits/builds it after review. No run or workstation queue admission is granted by this report.

- Corpus set fixed to frozen CI **wikidata, pattern, onebyte** only, preserving generator/manifest and recording compressed SHA256, decoded hash, block IDs, n and decoded lengths. One thread on one pinned allowed logical CPU; record topology/toolchain/flags/executable hashes. No cross-machine pooling.
- First untimed pass: run both exact baseline and bounded paths, plus one independent existing oracle pass/corpus; check full bytes/CRC/errors. Capture per-block outer gate, r, letters, B, attempted/successful cells, refusal, period, allocation/growth/capacity, requested/written copy bytes by writer, fallback reach. Instrumentation local to diagnostic build, dumped after decode, no forced admission. Expected Wikidata census counts above are a reconciliation check, not an assertion about unobserved blocks.
- Four cells per corpus: baseline and bounded, each instrumentation **off/on**; preserve original readers, default policies, thresholds and cap. On builds record exclusive scanner/front, order, induction, writer, fallback, finish, and remaining reader/drain work, plus an inclusive inversion/whole span for reconciliation. Account nested spans without double counting; record clock-call count and logging/allocation overhead. Do not use phases.rs's substitute column path.
- **10 fixed paired rounds**, 0.25s settle, fresh child per decode, no within-block repeats/minima, balanced AB/BA order fixed before execution; off/on order similarly balanced. 3×4×10=**120 measured decodes**, plus 6 untimed path checks and 3 oracle checks. No adaptive extension, threshold tuning or rerun to chase significance. This is a diagnostic budget, not a promise of power or a merge test.
- Raw process wall/CPU/RSS and hashes retained; analyse paired whole ratios and cluster phase contributions by complete decode, not 233 independent samples. Report instrumentation inflation for each arm; reject quantitative phase attribution if it changes the off-build conclusion or phases fail reconciliation. Counts remain descriptive. Preserve unavailable counters rather than invent cycles.
- Cost scenario using these CI medians: 20×[(.442+.027+.018)+(.442+.059+.021)]≈**20.2 s** of decode wall plus120×.25=**30 s** settling. Build, oracle checks, tracing/dump overhead, queueing and host variation are additional; ~50s is arithmetic, not a runtime guarantee.
- Deliver exactly one cost reconciliation and one next-action recommendation. If the measured eligible inversion gain cannot cover additional costs on the binding decoder, park this bounded implementation with reopening condition: a source change targeting the measured dominant cost, a priced whole-decoder ceiling, and new prospective admission. If instrumentation cannot identify costs, publish that limit and park; do not expand into a causal programme.

**Expected value:** one compact registration can reject the two cheapest integration explanations, price the binding path and explain the two strong negative controls without changing MONODROMY mathematics or Pathfinder architecture. A production rewrite/default flip is economically inferior before this diagnosis; broader team/service/emitter improvements remain alternatives with whole-decoder scope, but no evidence here establishes that an unmeasured port should supersede them. This brief recommends diagnostic preparation only; B5 remains the Researcher's independent assignment.

## State/protocol

Migration legacy head and exact manifest digest matched before receipt3c1f7289. Progress committed8cb92351. Pinned routing audit replay retained five resolved historical entries and historical researcher LOG6757065549; no current assignment-routing conflict. These do not widen scope. No automation configuration changed.
