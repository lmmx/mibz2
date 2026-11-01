# External research report 131: Astra's research-architecture memorandum: fixed navigation records first (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01.
- **Source:** Astra (a new Codex session, at its maximum reasoning effort).
- **In reply to:** report 130 (the maintainer's brief). It also answers the guardian's
  supplementary evidence note, which pointed it to the architecture plan (#129), its
  corrections and their primary references.

**Evidence cutoff:** 1 November 2026. This is after the P87 four-core i9 capture and the
architecture plan's corrections. By its own account it is source review and arithmetic only:
nothing was implemented, modified or run.

It is kept exactly as received (rendered text).

## The memorandum, as received

`````text
**I. Executive recommendation**

**Make P40/P38/P84 a single, tightly capped feasibility investigation: fixed 64-byte navigation records, byte recovery from the same record, and direct construction. Start with FOR-14 plus raw exceptions.**

This remains my recommendation after reviewing Claude’s [PR #129](https://github.com/lmmx/mibz2/pull/129), its primary references, and the newly supplied researcher review.

The architectural opportunity is to obtain approximately the table footprint of two cooperative teams **while retaining four independent decoding pipelines**. On the historical census, the proposed first format would occupy about **1.87 MB per full block**, or **7.47 MB for four tables**, versus 14.4 MB today. Whether that saves cycles depends on its additional navigation and construction work; the footprint alone establishes no performance benefit.

The researcher review’s principal objections hold. My audit adds two consequential qualifications:

- The historical **891 versus 1007 MB/s** comparison used four decoding threads **unpinned on the full i9**, not a process confined to four physical cores.
- The **22–28% serial-MTF shares** came from different cooperative configurations, not that two-teams-of-two full-decoder comparison.

These qualifications weaken the case for making a redesigned cooperative pipeline the first investment.

Segment-aware RLE1 is technically feasible and deserves a place on the shortlist. However, its direct opportunity is approximately **0.147 CPU-s/GB**, before boundary-processing overhead. Recoveries of 0.25–0.35 require additional, unproved effects on other phases. It is substantially the earlier P85 direction, reopened by better evidence about the current concurrency regime rather than by a new algorithm.

**No candidate currently establishes a credible standalone 1.5× gain.** Compact navigation has a defensible chance of recovering a material fraction of the CPU penalty; utilization improvements would still be needed. A 2× result requires a further reduction in underlying decoding work that none of these proposals has demonstrated.

This memorandum reports source review and arithmetic only. I did not implement changes, modify research records, or run benchmarks.

**II. Performance opportunity model**

Use decimal decoded GB throughout. Let:

- `C` be aggregate process CPU-seconds per decoded GB;
- `U` be average aggregate CPU activity, including scanner, feeder, reader and workers;
- `T = U / C` be decoded GB/s.

For the original baseline, `C = 3.579` and `T ≈ 0.870`, giving `U ≈ 3.114`.

This identity separates accounting terms, not independent engineering knobs. More concurrent useful work can increase contention and therefore `C`; spinning can increase `U` without increasing throughput.

The [whole-corpus reconstruction](https://github.com/lmmx/mibz2/blob/0156e1dad5ce4ebca78b4cb3d00276d3b6d64611/docs/journal/2026-11-01-bwt-memory-proposal.md) gives the following approximate phase costs:

| Phase | One worker | Four workers | Increase |
|---|---:|---:|---:|
| MTF | 0.472 | 0.505 | 0.034 |
| T-vector construction | 0.501 | 0.592 | 0.091 |
| **Traversal plus assembly** | **1.236** | **1.847** | **0.611** |
| RLE1 plus CRC | 0.137 | 0.196 | 0.059 |
| Recorded block phases | 2.346 | 3.140 | 0.794 |

All entries are CPU-s/decoded GB. The traversal interval includes assembly.

The approximately 0.44 difference between the four-worker phase sum and the original 3.579 total is a **planning residual**, not an exact attribution: the measurements were separately instrumented. Scanner/Huffman work, reader/feeder work and runtime overhead must remain in the full-decoder budget.

For compact navigation, approximately **1.14 CPU-s/GB remains currently unaddressed**: MTF, RLE1/CRC and that residual. This is not a claim of fundamental irreducibility.

The throughput sensitivity is:

| CPU saving from 3.579 | At current `U ≈ 3.114` | At `U = 3.8` | At `U = 4.0` |
|---|---:|---:|---:|
| None | 870 MB/s | 1062 MB/s | 1118 MB/s |
| 0.25 CPU-s/GB | 935 | 1141 | 1202 |
| 0.50 | 1011 | 1234 | 1299 |
| 0.70 | 1082 | 1320 | 1389 |
| Entire measured 0.872 penalty | 1150 | 1404 | 1478 |

These are conditional arithmetic, not predictions.

Two implications determine the research priorities:

1. Recovering **0.25–0.50 CPU-s/GB alone gives approximately 8–16% throughput**, at unchanged utilization.
2. At `U = 3.8`, reaching 1.5× requires saving about **0.667 CPU-s/GB**. A promising 0.25 saving is a useful component, not a breakthrough by itself.

For the compact candidate, let `b` and `w` be new/old ratios for construction and traversal-plus-assembly. Assuming other costs remain unchanged:

`Saving ≈ 0.592 × (1 − b) + 1.847 × (1 − w)`.

Therefore:

| Construction change | Traversal-plus-assembly reduction needed for 0.25 saving | Reduction needed for 0.50 saving |
|---|---:|---:|
| Unchanged | 13.5% | 27.1% |
| 10% more expensive | 16.7% | 30.3% |
| 20% more expensive | 19.9% | 33.5% |

This is the relevant bargain. “Half-sized table” is not the acceptance criterion.

**III. Where the existing research may be misdirected**

The assumption I would challenge first is that the shipped packed T-vector’s access economics are already close to the best achievable representation.

The 32-lane implementation may be highly developed **within its current representation**. Failed unrolling and hint changes support that narrower conclusion. They do not establish that four explicit bytes per row are the right architectural limit.

Conversely, succinct research has sometimes accepted the opposite assumption too readily: that fewer table bytes will repay extra instructions. The failed implementations strongly contradict that shortcut.

Several evidence interpretations need correction.

**The memory measurements establish a changed operating regime, not its physical cause.** The corrected P87 observations—approximately 0.38 to 9.01 DRAM-read bytes per decoded byte, and 0.27 to 6.69 writes—are machine-wide, idle-adjusted traffic. They do not identify T-vector demand loads.

Similarly:

- Mixed offcore occupancy is not demand-load latency.
- Retired-demand hit fractions and demand-plus-prefetch request counts have different populations.
- Neither “six useful requests in flight” nor “98% software prefetch” is an established walker measurement.
- The older approximately seven reads in flight and fill-buffer-full figures cannot establish the current production walker’s remaining prefetch headroom.

**The 44% `memmove` stall attribution is a reason to investigate copying, not a removable-cost estimate.** It is a share of one sampled demand-L3-miss-stall population. It includes copying outside segment assembly, and prefetching changes where demand-miss samples appear. The walker’s 4% share does not imply negligible memory sensitivity. The more useful direct anchor for R2 is the separately timed assembly phase.

**Mean activity is not cache residency.** The reported average 2.19 active T-vector phases invalidates a picture of four continuously active full tables. It does not show that only 2.19 tables’ lines occupy cache: allocations persist, earlier phases leave cache contents, and short periods of greater overlap may matter disproportionately. Neither “14.4 MB fits in 20 MB” nor “total scratch exceeds 20 MB” resolves the issue.

**P87 supplies evidence for a modest intervention, not a multiplier for future copy removal.** Its [report](https://github.com/lmmx/mibz2/blob/a0094cfa0cef985e9f57fa9b384656b7ae5bcaad/docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md) preserves important distinctions:

- Uninstrumented four-worker process CPU fell from **3.843 to 3.777**, a saving of 0.066.
- Separately traced phase savings summed to approximately 0.098 thread-CPU-s/GB.
- Assembly improved by approximately 0.026.
- Traversal’s estimated 0.036 improvement had an interval including zero.

Dividing the traced “other-phase saving” by assembly’s saving does not produce a transferable causal multiplier. Its substantial traffic reduction alongside only 1.35% higher throughput also warns against predicting speed from traffic volume alone.

The formal outcomes remain **mechanism inconclusive; engineering not demonstrated; historical high-worker criterion not evaluated**.

**The proposed neighbour comparison is not yet a footprint-versus-request-pressure discriminator.** Recompressing the same output at `-3` changes block boundaries, navigation permutations, phase cadence, compression ratio and request rates. Equal requests per decoded byte do not imply equal requests per second. Matching an aggregate L2-miss rate does not match demand/prefetch/writeback mix, burstiness or occupancy.

Three independent decoder processes also introduce different scanner and reader activity from one four-worker process. Calibration can establish usefulness; identical decoder code cannot establish fidelity by itself.

I would defer the proposed harness. If later needed, use the minimal sequence—target alone, real decoder neighbours, walker-only neighbours—before selecting **one** further contrast. Interpret smaller-block effects as evidence consistent with footprint sensitivity, not exclusive proof.

**IV. Succinct-navigation reassessment**

The mathematical invariant matters because byte recovery was central to the previous failure.

Let `L` be the BWT last column, `F` its sorted first column and `C[c]` the start of character `c`’s bucket. Within that bucket:

`Ψ(j) = select_c(L, j − C[c])`, and `L[Ψ(j)] = F[j]`.

Thus Ψ is increasing inside each `F` bucket.

The shipped [builder](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/bwt.rs) stores:

`T[j] = L[j] | (Ψ(j) << 8)`.

The low byte is **L[j], not F[j]**. A new representation must preserve that orientation or explicitly adjust traversal.

A compact walker can retain the exact segment starts by carrying one byte per lane: at row `j`, emit the carried `L[j]`, obtain `F[j]` from the current record, and carry it into the step at `Ψ(j)`. Initial segment bytes can be seeded from `L[start]`. Simply emitting the header’s `F[j]` at the existing starts would shift the output.

The [census and dependency analysis](https://github.com/lmmx/mibz2/blob/c32042b655e371c16c0e53f7299ff303078d49b3/docs/journal/2026-10-29-succinct-review.md) support this comparison:

| Representation | Approximate bytes/row | Full-block table | Navigation consequence |
|---|---:|---:|---|
| Shipped packed T | 4.00 | 3.60 MB | One load supplies link, byte and marker |
| Failed 24-bit links | 3.00, plus byte directory | 2.70 MB plus directory | Additional byte-recovery loads and instructions |
| Full P40 FOR/EF/raw | 2.020 | 1.818 MB | Ordinary header and payload in one line; rare raw spill |
| **Proposed first FOR-14/raw format** | **2.076** | **1.868 MB** | Simpler common decoder; 1.89% raw exceptions on historical census |
| Two-tier directory | About 0.712 | 0.641 MB | Extra dependent lookup on roughly 12–36% of steps |

These sizes describe the historical corpus, not a newly measured current workload.

**What failed in the 24-bit implementation**

The [compact-Ψ experiment](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-compact-psi.md) did demonstrate a shifted residency knee. It also demonstrated that its access overhead overwhelmed that benefit.

The strongest directly relevant same-configuration warning is approximately **0.681× at one worker against its control**. Byte recovery cost approximately 28% of walk performance at one block and 16% at four. The common path used a window lookup followed by a dependent boundary-table lookup, comparison and branch—even though actual boundary correction was rare.

Those two byte-recovery loads were dependent on each other, though not necessarily on the link load. Being off the next-link dependency chain did not make them free.

The often-quoted **0.687×** comparison is compact `u6` against cooperative `l4x2`: different configurations and core counts. It is relevant to that historical throughput frontier, not a same-four-core full-decoder result.

P38’s useful distinction is therefore concrete: **place F’s byte beside the link information**, eliminating the separate byte-address stream. A larger side directory with byte-plus-boundary flags may improve the old implementation, but does not solve the same problem as co-location.

**The smallest worthwhile P40 experiment**

Use a 64-byte-aligned record for 32 rows:

- 64-bit header: 20-bit base, 8-bit `F` byte, format tag and full 32-bit start mask.
- 56-byte payload: thirty-two 14-bit offsets.
- Ordinary successor: `base + offset[slot]`.
- Record address: `records + 64 × (row >> 5)`.
- Raw exceptions for groups crossing character buckets or exceeding the offset span.

RUN groups use the same FOR-14 path, with offsets 0–31. A separate RUN branch is unnecessary initially.

On the historical census, RUN plus FOR-14 covers **98.11% of rows**. Treating the remaining 1.89% as raw gives:

`2 + 4 × 0.0189 = 2.0756 bytes/row`.

Omitting EF therefore costs only approximately **50 KB per full block** relative to the full hybrid. That is a reasonable price for first removing EF decoding and construction complexity.

This simplification is workload-sensitive. The historical ELF input would have approximately **10.4% raw exceptions** without EF. The census’s “exceptions at most 2%” statement applies to the full hybrid, not this simplified format. Controls and a current-corpus census remain necessary.

Ordinary header and payload addresses both follow directly from the current row. They can be independent loads from the **same cache line**, without a second dependent memory lookup. But:

- One cache line does not mean one scalar load.
- Packed-field extraction must stay inside the record, including the final slots.
- Mask extraction, byte carry and format handling add instructions.
- Raw exceptions require a further dependent access.
- Row-weighted format frequencies need not equal dynamic lookup frequencies on periodic inputs.

On ordinary historical blocks, the nominal navigation-line count is approximately **1.019 lines per step** for FOR/raw, versus approximately 1.005 for the full hybrid. These are line references, not physical cache misses.

The old analysis estimated approximately 25 instructions per step versus roughly 17 for the packed walker. Treat that as a sketch. A reasonable design budget is roughly **24–32**, pending code generation and measurement. Eight additional instructions per row, if they consume execution capacity at 2–4 IPC and approximately 4.9 GHz, represent roughly **0.4–0.8 CPU-s per billion rows**. Conversion to decoded GB requires the row/output ratio. That opportunity cost can erase the hoped-for saving.

The documented claim that a 77/21 format split implies approximately one-fifth branch mispredictions is also not established by the census. Frequencies do not determine predictability. I would omit the branch to simplify the first test, not because its misprediction rate has been measured.

**P84 is the production veto**

Counts already exist after MTF. A scan of `L` supplies each character’s occurrence positions in increasing order, so direct Ψ construction requires no sorting.

A plausible builder buffers up to 32 positions per character and emits complete records. That needs approximately **32 KiB of position staging**, plus metadata—already comparable with the i9’s L1 data cache.

The difficult details are substantive:

- Global 32-row groups can cross character buckets. Per-character partial groups cannot simply be emitted as globally aligned records.
- Crossing groups need preassigned raw storage and per-row bytes.
- Complete records should use wide stores; per-entry packed read-modify-write would repeat a known failure.
- The shipped builder batches long equal-byte runs. A replacement must preserve that advantage where possible.

A naïve staging implementation writes and rereads four-byte positions: approximately **8 logical bytes per row**, before reading `L` and writing the approximately two-byte final record. That can exceed **11 logical bytes/row**, compared with approximately six for the shipped fused builder. These are logical element accesses, not DRAM traffic.

Run-aware bulk construction could avoid much of that staging. It is a hypothesis requiring an operation budget, not a free consequence of compactness.

Building full T and then compressing it is acceptable as an offline oracle for the first test. It is not a production construction strategy: it adds passes and retains both representations during construction.

Memory also needs an explicit bound. All-raw fixed records would cost six bytes per row. A production candidate therefore needs a capped shared arena and fallback to the original representation, charging partial construction work. Retaining both tables is unacceptable.

For orientation, current full-block scratch reservations are approximately **7.23 MB per worker**, including about 1.83 MB of segment capacity. Compact active regions would be approximately **5.50 MB**, plus staging. Persistent fallback allocations can keep peak RSS near the original value; these figures are neither measured RSS nor cache residency.

**Why this is a distinct, still-open hypothesis**

The proposed combination specifically changes the costs that defeated earlier compact forms:

- P38 removes the separate common byte lookup.
- P40 gives a directly addressed common record with no second dependent line.
- P84 avoids a full-table compression pass and packed per-entry updates.

They do not automatically combine successfully. P40 supplies the largest footprint opportunity; **P38 makes it plausible; P84 decides whether it survives end to end**.

The alternatives remain less attractive initially:

- Two-tier storage buys additional compression by placing a second dependency on a much larger fraction of steps.
- Dense 20-bit and split-link pilots suffered severe build penalties; those measurements were on the VM, not a definitive i9 rejection of every packed representation.
- Move/composed structures can eliminate transitions, but the recorded implementations lost on multi-lane build-plus-walk economics.
- The later [N1 construction analysis](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-n1-construction-economics.md) supplied no convincing cheaper construction path. Its closure was an economics decision, not an i9 falsification of all composition.
- The 77.39% translation structure does **not** mean 77.39% of decoded output can be copied without navigation. Spatially consecutive Ψ entries are not necessarily consecutive traversal steps.

**V. Ranked breakthrough candidates**

The probabilities below are **subjective engineering judgments**, not measured frequencies. “Material success” means either recovering at least approximately 0.25 CPU-s/GB, or increasing full-decoder throughput by at least 10% without worse CPU efficiency or unacceptable RSS.

| Rank | Candidate | Conditional opportunity | Judgment: material success | Initial cost / information value |
|---|---|---|---:|---|
| **1** | Fixed records, co-located byte, direct build | 0.25–0.50 CPU-s/GB if access and construction costs cooperate; regression remains a serious possibility | **25–35%** | Capped feasibility work; directly resolves a major unfinished architecture |
| **2** | Wake placement and pipeline admission | Approximately 0.15–0.35 additional useful cores is a plausible investigation range: roughly 5–11% throughput at unchanged CPU cost | **25–40%** | Small discriminator first; substantial value without replacing receiver architecture |
| **3** | Segment-aware RLE1 consumer, R2/P85 | Approximately 0.08–0.20 CPU-s/GB if successful; 0.25–0.35 requires substantial indirect savings | **10–20%** | Local feasibility is cheap; full production benefit remains uncertain |
| **4** | Work-conserving two-teams-of-two pipeline, R1 | Historical CPU advantage worth checking; MTF-idle removal alone models near parity to a modest gain | **10–20%** | Cheap validation of existing configuration first; redesign is expensive |

These are competing investments, not additive gains.

**1. Fixed records**

This attacks the largest identified cost pool while preserving four independently schedulable blocks. Its approximately 7.47 MB of four-table storage is close to two full T-vectors’ 7.2 MB, without requiring cooperative inversion.

The optimistic opportunity is substantial recovery of the approximately 0.70 CPU-s/GB construction-plus-traversal/assembly concurrency increase, less new overhead. That is an opportunity bound, not an expectation. The most likely outcome may still be rejection or a modest gain.

Its principal risks are execution throughput, longer link arithmetic, staging traffic, exception handling and fallback RSS. Construction can defeat it even if traversal improves. The cheapest decisive test is specified in section VI.

**2. Utilization through wake placement and admission**

The [dispatch study](https://github.com/lmmx/mibz2/blob/4ef5f36d5cf0193d9183b8f599d1105f7719d66d/docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis.md) rejected direct receiver obstruction as a sufficiently large target: approximately **1.90% of aggregate worker time**, below its registered criterion. That does not refute a placement intervention.

More relevant observations include:

- Approximately 5.65% of aggregate worker time in the identified runnable/idle-core category: roughly **0.226 core-equivalents** across four workers, as an upper bound.
- Scanner-open runnable time corresponding to approximately another **0.145 core-equivalents** under the report’s categories.

These are observational pools, potentially overlapping and not fully recoverable. They must not simply be added.

The cheapest discriminator would change wake/placement or bounded admission behavior while retaining the receiver and decoding kernels. It must establish that idle-core runnable delays fall and **completed throughput increases**, counting all process threads inside the four-core allocation.

Construction and navigation remain unchanged, but their overlap changes; that can increase CPU cost. Backlog and output slots must remain bounded. The scanner’s approximately **3.1 GB/s standalone ceiling** argues against interpreting all 18.9% no-job time as a need for parallel scanning.

This is a better utilization investment than another traversal micro-optimization, provided it targets measured scheduling gaps.

**3. Segment-aware RLE1: feasible, but with a smaller direct budget**

The existing [RLE1 implementation](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/src/rle.rs) tests 32 possible run starts using AVX2 while at least 35 input bytes remain. Long non-run spans are copied in bulk.

A segment-aware consumer can preserve that structure:

1. Follow the existing segment order, consuming exactly the block’s RLE-coded length.
2. Process long contiguous interiors using the existing vectorized logic.
3. Retain at most a small unfinished suffix—up to four input bytes—and resolve the boundary with a small prefix from the next slice.
4. Resume the vector path after the boundary token is resolved.

The state must retain **parse position**, not just “last byte and repeat count”: after four equal bytes, the next byte is a count even when it equals the repeated byte.

A small stitch buffer, on the order of nine bytes, is sufficient for this local boundary resolution. With roughly 512 slices in an ordinary 900,000-row block, that is around 0.5% copied boundary bytes. It is **not** a CPU-overhead bound: SIMD tails, scalar work and per-slice handling also matter. Tiny slices and periodic cases need separate attention.

The consumer must reproduce existing assembly semantics for repeated segment cycles and a truncated final segment. It cannot assume that each segment or row is visited exactly once.

CRC should initially remain over the **contiguous expanded output**, using the existing accelerated implementation. The segment bytes are RLE-coded data, not the CRC’s byte stream. Streaming CRC fusion is a separate change requiring persistent folding state; repeatedly finalizing CRC per slice is incorrect, and replacing the accelerated whole-block path with small scalar updates may lose performance.

The cost model is:

`R2 saving = 0.147 − boundary/fragmentation overhead + indirect savings elsewhere`.

Removing assembly eliminates approximately one read and one write of the RLE-coded block and its persistent input buffer—about 0.9 MB per worker at maximum block size. It does not remove RLE expansion or CRC work. Physical DRAM savings need not equal logical traffic removed.

Why reopen P85? The [register](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md) records that it **stopped at the cost gate**, rather than a completed segmented consumer failing its production trial. Earlier assembly shares were generally only a few percent.

The new argument is the current four-core memory environment and the possibility of collateral benefits. Even now, approximately 0.147 is only about 4% of diagnostic CPU cost. This strengthens a bounded reconsideration; it does not establish Claude’s planning range.

The cheapest future falsification is a matched post-walk comparison: existing assembly plus RLE1 plus unchanged CRC, against slice-consuming RLE1 plus the same CRC. If fragmentation consumes the direct saving, stop. Any claim above the direct budget must then survive a whole-decoder comparison and identify the additional phase savings.

Reader-drain elimination should not be bundled into this test. It changes ownership and API behavior and has a separate cost model.

**4. Two teams of two: the serial-MTF repair has a limited ceiling**

The [historical full-decoder report](https://github.com/lmmx/mibz2/blob/13e9854805eed8944ecdd7c9330e30971b6bf52d/docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T174657Z.md) reports:

| Configuration | Throughput | CPU time for that corpus |
|---|---:|---:|
| Four independent workers | 1007 MB/s | 0.84 s |
| Two teams of two | 891 MB/s | 0.73 s |

Those CPU figures are **seconds for the corpus, not CPU-s/GB**.

The ratio of displayed throughput summaries is 0.885; the report’s paired ratio is **0.904**. They are different estimators and should remain distinct.

The run was unpinned on the full i9. Its separate four-core check compared production with auto-selection, which selected independent workers; it did not establish forced two-teams-of-two performance under that constraint.

The instruction claim is also assembled from different evidence. The older approximately **35 versus 23.8 instructions/byte** came from an earlier cooperative kernel and different team configurations. The [later rematch](https://github.com/lmmx/mibz2/blob/f484bd50e0b8a78690861b7dccecb9a80a96b1b1/docs/journal/2026-10-29-libsais-rematch.md) reports approximately **27.5–27.8 versus 23.7**, including synchronization effects. Neither is an instruction measurement from the exact 891/1007 full-decoder comparison.

For the scheduling ceiling, let:

- `m` be serial MTF duration;
- `b` be inversion duration using both team cores;
- `f = m / (m + b)`.

Current team-round wall time is `m + b`, while required core-time is `m + 2b`. Even perfect scheduling cannot complete that work faster than half its core-time:

`maximum utilization-only speedup = 1 / (1 − f/2)`.

For `f = 0.22–0.28`, this gives approximately **1.124–1.163×**, assuming unchanged phase costs.

Applied illustratively to the historical throughput summaries, this yields approximately **0.99–1.03×** the independent-worker reference. Applied to the paired 0.904 ratio, it yields approximately **1.02–1.05×**. These are conditional arithmetic using MTF shares from other configurations, not predictions or confidence intervals.

Both cores are already occupied during cooperative inversion. “Run the next MTF on the idle helper while inversion proceeds” therefore needs a different schedule. One possible schedule batches two independent MTF operations concurrently, then performs their cooperative inversions. Another changes inversion’s core allocation. Neither makes MTF’s core-time disappear.

A redesign also needs additional pending `ll` and symbol state, bounded output buffering, and a guarantee that it does not reintroduce more active full tables.

**There is enough historical evidence to justify a cheap contemporary check of the existing two-team configuration. There is not enough to justify a redesigned worker pipeline yet.**

Its first falsification should be that existing configuration under the actual four-physical-core constraint, followed by an explicit work-conserving schedule bound using its measured stages. If the CPU advantage disappears, or even the ideal schedule cannot produce a material net gain, stop before redesign.

**VI. The single best next experiment**

**Select one FOR-14/raw compact-navigation feasibility test, with byte co-location and a direct-builder budget.** This is a proposed future experiment, not authorization to execute it.

Cap the initial investigation at **three engineer-days**. Allow at most **two further days** for a direct builder and full-decoder decision, only if the first stage earns them. These are stopping budgets, not estimates that production-quality implementation is guaranteed within five days.

**Stage 1: specify and test the cheapest viable common path**

Use one candidate and one shipped control. Preserve:

- Exact segment starts and termination behavior.
- The 32-lane organization and current prefetch policy.
- Segment output, assembly and RLE1/CRC.
- Short-block fallback and existing stream/error semantics.

Use FOR-14/raw only: no EF, separate RUN branch, two-tier directory, lane sweep or simultaneous scheduling change.

First check the format mix on the actual target corpus. The historical census covered 233 full blocks; the later diagnostic corpus contained 434 blocks with different size composition. Transferring its percentages silently would be a mistake.

An offline full T-vector may supply a correctness oracle and precomputed records. It must not remain alongside the candidate during the timed navigation comparison. Keep the number and lifetime of live blocks bounded like the production path.

Reuse existing stage instrumentation. Before interpreting a four-worker result, require the control to reproduce contemporary shipped stage costs reasonably closely—for example within 5%, with instrumentation perturbation below 3%. If calibration cannot be achieved within the cap, stop rather than building a larger harness.

Precomputed construction makes this an **optimistic navigation test**. A positive result only earns a builder; it is not a decoder speedup.

**Stage 2: turn the measured traversal saving into a builder budget**

Let `G` be the measured saving in traversal plus assembly. For a 0.25 CPU-s/GB architectural opportunity:

`new construction cost ≤ old construction cost + G − 0.25`.

Using the illustrative 0.592 baseline:

- `G = 0.25` permits no construction regression.
- `G ≈ 0.31` permits approximately 10% more construction cost.
- A 20% traversal-plus-assembly reduction permits approximately 20% more construction cost.

This makes the experiment discriminating. If navigation saves too little, the proposal survives only by identifying a sufficiently cheaper builder—not by promising that compactness will help later.

Reject the candidate if even its optimistic common path leaves no credible combined budget. An instruction count alone is not a performance verdict, but a large measured execution penalty is a strong early falsification.

**Stage 3: direct construction and whole-decoder confirmation**

Only after passing the preceding budget should a direct builder be attempted. Charge all staging, packing, initialization, allocation, exception handling and abandoned fallback work against the shipped fused builder.

The full-decoder comparison must include:

- Throughput, aggregate process CPU and utilization on four physical cores.
- Construction, traversal and assembly separately.
- RLE1/CRC and scanner/reader effects.
- Actual peak RSS, including fallback high-water allocations.
- Exact output, CRC and error behavior, including short and periodic blocks.
- Contemporaneous one-worker and relevant control results.

The outcome changes the next decision as follows:

| Outcome | Decision |
|---|---|
| Common navigation path cannot repay its overhead | Defer this fixed-record architecture; no direct builder |
| Navigation succeeds, construction exceeds its budget | Stop unless one identified bulk-construction fix fits the remaining cap |
| Only exception-heavy controls fail | Treat this as a limitation of FOR/raw; do not claim all P40 variants disproved or expand automatically |
| Stages improve, full decoder does not | Stop promotion; investigate the exposed pipeline constraint |
| Full decoder gains materially with bounded CPU/RSS | Advance compact navigation; then test whether utilization can improve without restoring contention |

This has greater immediate information value than the neighbour harness because it tests an actionable architecture, including the precise costs that previously defeated compactness. It has a larger direct opportunity than R2 and fewer unverified scheduling premises than R1.

**VII. What to stop doing**

- **Stop treating all compact navigation as closed.** Close particular implementations for their measured costs. P38/P40/P84 remain unfinished.
- **Stop treating compactness as evidence of speed.** A build-plus-walk budget is mandatory.
- **Do not restart MPMC receiver replacement** on the existing evidence. Placement and runnable-delay hypotheses are different.
- **Do not start the full neighbour harness now.** Its proposed signatures do not uniquely identify physical causes.
- **Do not use P87’s indirect/direct ratio as a scaling law.** Retain one bounded production-shaped validation proposal, not an indefinite family of variants or an indefinite hold awaiting R2.
- **Defer prefetch-distance work until it has a contemporary service/overlap model.** The old in-flight figures neither prove ample headroom nor establish Claude’s small ceiling.
- **Do not revive unchanged dense packing, pair-walk, move structures, precopy or fully unrolled traversal.** Each needs an explicit answer to its recorded construction, instruction or full-decoder failure.
- **Do not serialize walks merely to reduce overlap.** The old permit-gating losses and the large traversal share make idle-core costs central.
- **Do not infer a need for parallel scanning from no-job time alone.**

One historical correction matters here: the [i9 unroll report](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/analysis.md) found approximately neutral primary throughput and slightly improved primary CPU, with a CPU regression on a control workload. “Worse CPU everywhere” would misstate the negative evidence.

I would retain roughly the proposed **70/20/10 allocation**, with the architectural share released in stages. Failure of the compact feasibility gate should release its budget, not trigger an unlimited succession of encodings.

**VIII. Assessment of the 1.5–2× objective**

**1.5× is physically credible, but no present candidate establishes the route.**

The target is approximately 1305 MB/s. Required CPU cost is:

| Aggregate CPU activity | Maximum CPU-s/GB for 1.5× |
|---|---:|
| Current, approximately 3.114 | 2.386 |
| 3.8 cores | 2.912 |
| 4.0 cores | 3.065 |

A plausible conditional route is approximately **0.67–0.70 CPU-s/GB recovery plus 3.8 useful cores**. That means recovering most of the identified construction/traversal concurrency increase while substantially reducing pipeline gaps. Both improvements are ambitious, and they may interfere.

A compact candidate saving 0.50 combined with `U = 3.8` reaches only approximately **1234 MB/s, or 1.42×**. It would be a strong result without meeting the objective.

**2× is not presently a defensible engineering forecast.**

At 1740 MB/s:

- Four fully used cores require **2.299 CPU-s/GB**.
- At 3.8 cores, the requirement is **2.184 CPU-s/GB**.

Even the first is about **15% cheaper than the measured one-worker decoder**. Recovering all observed concurrency loss and all utilization loss reaches only approximately 1.70×.

Neither assembly removal nor helper-idle elimination supplies the missing reduction in fundamental work. Fixed records ordinarily still perform one navigation step per BWT row. A credible 2× proposal would need additional operations eliminated or substantially cheaper construction, traversal or output processing—demonstrated against the current fused baseline.

The new historical evidence therefore changes the **secondary shortlist and the scrutiny applied to copying**, but not the primary recommendation. R2 has a smaller established cost pool. R1 has an attractive historical CPU observation but an unvalidated four-core transfer and a modest MTF-only throughput ceiling. Neither presently outweighs the unfinished fixed-record hypothesis.

**IX. Questions for the independent reviewer**

1. **Can the fixed-record common path actually retain one cache-line dependency while producing the correct byte, successor and marker?** Challenge the byte-carry orientation, bounded field loads, exception frequencies and instruction budget—not just the footprint.

2. **Can direct construction fit the measured savings budget without full-table staging or excessive fallback memory?** Challenge whether run batching genuinely offsets position-buffer traffic and packing work relative to the shipped fused builder.

3. **Will the gain survive the whole four-core pipeline?** Challenge the transfer of historical team results, any R2 saving beyond assembly’s measured cost, and the assumption that improved CPU efficiency can coexist with higher useful utilization.
`````
