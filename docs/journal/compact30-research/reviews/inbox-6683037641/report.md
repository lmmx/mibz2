# Live bytes per block: inbox 6683037641

Reviewed 2026-11-06. [Brief](https://github.com/lmmx/mibz2/pull/160#issuecomment-6683037641).
**Disposition: proposals only; zero new verified instruction or wall-time credit.**
The screen supports fewer concurrent teams as a performance lead. It does not identify allocated capacity as the cache demand, nor establish cache capacity as the unique cause.

## Pins and replay

- Tested source and frozen team-size registration: **f4f99ab82652411feb4a5a6e2ccb96d04902fb58**.
- Result archive: **cbc6eb14b065fdb7092911147ce3374f63541467**, PR #219. Its `commit.txt` and occupancy metadata both name the tested source above. The journal at the source pin still says “Not yet run”; results necessarily arrived in a later commit.
- Three-table registration: **656d2794cf0146e65af9223b8a10c31297518f35**, PR #220; “Result: Not yet run” at this reviewed pin. No later result is assumed.
- Governing SPEC / SPEC.OPTIMIZER: **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**.
- Source locations below refer to the tested source, not moving master.

[arithmetic.json](arithmetic.json) independently replays all 320 archived `runs.jsonl` records: one hash `a8830caee5c511a6`, output 209700196 bytes, no errors, all 16 cells with rounds 0–19 once, and exact leader/helper counts. Paired log ratios use the archive's t critical 2.093 at df19. Central estimates and intervals reproduce the stored report. This is analysis of existing measurements, not a new timed run or independent safety review.

| Plan | Paired speed ratio to 4×2 | 95% interval | CPU ratio | Decision |
|---|---:|---:|---:|---|
| 4×2 entry-point control | 1.000579 | 0.989450–1.011833 | 1.0060 | Not flagged; equivalence not proved |
| 4×3 | 1.016955 | 1.007929–1.026062 | 1.2572 | Fails 1.03 point gate |
| 4×4 | 1.003463 | 0.992881–1.014157 | 1.5245 | Fails benefit and CPU adoption limit |
| 3×3 | 1.075272 | 1.058068–1.092755 | 0.8912 | Exploratory, unqualified |
| 3×4 | 1.096728 | 1.083018–1.110611 | 1.0401 | Exploratory, unqualified |

Perf did not count. The trace's 96% decoding includes build, walk, assembly, RLE and CRC, not 96% random walking. Its inversion span includes assembly. Four is a concurrency cap: spread 4×2 has mean traced T occupancy about **2.747**, mean 9.885 MB and high-water 14.4 MB. Those are lifetime estimates, not cache-residency measurements. Neither the sum of endpoint capacities nor the sum of separate high-water marks is a simultaneous working set.

The three-table change also changes k, decoding threads, segment count and total window (16 to 12). #220 explicitly preserves that attribution limitation. Its policy comparison is useful without being a pure cache experiment. Packed/spread is a CPU-mask intervention, not fixed assignment of helpers to siblings. At the reviewed 4×2 trace the paired packed/spread ratio is 0.777876 (0.755056–0.801386); this does not transfer as the gain of pinning an already unpinned release decoder.

## Counted access ledger

Let n be BWT rows (at most 900000), m decoded output bytes, s actual deduplicated segment starts (at most 512k), and A the sum of freshly written active segment-prefix lengths. For the permutation table, marked cycles are partitioned by the starts: A ≤ n, with A=n for a single covered cycle. The original cycle can be shorter and assembly repeats it until n bytes. Do not assume every segment read by assembly was written only once per output occurrence; do not include stale vectors beyond s in A.

| Object | Allocation observed at spread 4×2 endpoint | Accesses and lifetime at tested source | Correct price |
|---|---:|---|---|
| T / tt | 5.399846 MB | `coop::build_fused` writes 4n bytes; walk reads data-dependent u32 rows and issues prefetches. Start marking/unmarking occurs before/after helpers | Current table footprint ≤4n=3.6 MB. Capacity tail is not a walk input |
| seg.bufs | 6.118284 MB | Each taken segment writes a sequential prefix, many interleaved streams (32 lanes per thread); growth can copy prefixes. Assembly reads segments in successor order | A payload stores, then n payload reads; hot line frontier and reuse distances matter, not 6.1 MB reserved |
| ll | 0.900000 MB | MTF writes n; fused build reads it both by row and by bucket slot; no ll access after build during cooperative traversal | Dead input after successful build; remains allocated. Freeing does not itself evict cache lines or remove walk references |
| rle | 1.349962 MB | Untouched during cooperative traversal; `SegScratch::assemble` clears it and writes n after the helper barrier; RLE then reads it | Write-once/read phase, not a second random array. Old contents may still be cached, which is different from active demand |
| out | Variable, queued ownership | RLE writes m; CRC reads m; reader later consumes m, with output-buffer reuse | Real competing traffic from other teams/phases and reader; cannot subtract m by reusing ll |
| starts/next/headers/checkpoints | Small but nonzero | Start selection/sort, flags, binary searches, segment handoff atomic, per-segment metadata; checkpoints used in build | Include in setup and handoff; do not silently multiply per helper or treat as zero |
| compressed window/symbols | Window live high-water 6.271842 MB | Scanner input and symbol production, MTF symbol reads, queues | Streaming/lifetime competition; high-water is not all hot in L3 throughout a walk |

Static source anchors: `src/coop.rs:283–366,453–654`; `src/bwt.rs:193–231,448–470`; `src/lib.rs:314–360`; `src/rle.rs`; `src/trace.rs:414–503`.

A useful payload count for a completed traversal is **A+s u32 demand loads** (A emission loads including each segment's initial load, plus one terminal marked-row load per segment), plus the r0 load and setup/cleanup flag accesses. Thus approximately 4(A+s) demand-load bytes, A stores, plus prefetches, metadata and any realloc-copy traffic. It is NOT 64 bytes of DRAM per row. Cache lines can serve multiple accesses; prefetches can hit or race demand.

Assembly adds n source payload reads and n destination writes. RLE's unique input coverage is n, but this is not its load-instruction count: the AVX2 no-run path reads four overlapping 32-byte windows per 32 positions and also copies literals. CRC adds m bytes of coverage. “Remove 2n” below means eliminate the intermediate payload write/read, not exactly 2n fewer retired-load bytes or memory-controller bytes.

At n=900000, the initial requested reserve is `s*(floor(2*n/s)+64)`:
- s=512: **1,832,448 bytes**;
- k=2, s=1024: **1,864,704 bytes**;
- k=3, s=1536: **1,896,960 bytes**;
- k=4, s=2048: **1,929,216 bytes**.

These are requests before growth/allocator rounding, assuming that exact s; actual deduplication changes s. They do not explain 6.1 MB by multiplication alone. `clear()` keeps capacity; each segment retains its longest prior allocation, overflow calls `reserve(b.capacity())`, and the outer array retains old slots. The 6.1 MB is the measured retained-capacity sum. An earlier progress note's 1,865,728 figure is corrected here to **1,864,704**.

For 64-byte lines and arbitrary buffer alignment, active-prefix line coverage is below A+128s bytes, ignoring shared-line overlap (a deliberately loose per-allocation boundary bound). At k=2 and A=n that is below 1.031072 MB, far below 6.1 MB. Active writer-front lines at one instant are at most about 64×32k bytes before other live lines, prefetching, growth and assembly are considered: 4096 bytes for k=2. The cumulative prefix footprint is not the instantaneous writer frontier, and neither establishes residency. Fragmentation can still increase page/TLB demand despite bounded active cache-line coverage.

## What can disappear without deleting required work?

**C23: cooperative ll reuse, one construction.** Build T with ll/checkpoints as now; after build completes and all its borrows end, assemble the already walked segments over ll, then feed that slice to the unchanged RLE and CRC path. `decode_block` currently guards in-place inversion with `!cooperative`, while the cooperative interface takes ll immutably and rle separately. Merely toggling that guard is not a sound implementation. A future cooperative in-place entry point must retain the existing checkpointed build and join helpers before overwriting ll; its fallback must also finish every ll read before overwrite.

This can eliminate the separate rle allocation, approximately **1.35 MB/team** in the archive (5.4 MB at four, 4.05 MB at three if capacities transfer). It does not remove assembly, RLE, CRC, the 4n T or A stores. It can reuse up to n previously touched addresses across phases; cache benefit during other teams' walks is conditional. No per-block fresh-allocation credit: steady buffers already persist. Do not shrink/deallocate ll on every block. Validate original-cycle repetition, tiny blocks/fallback, helper panic/join, invalid input and CRC rollback. **Falsifier:** allocation eliminated but fixed-plan whole decode does not improve under a predeclared benefit rule; then retain only a separately justified memory objective. Highest-confidence new engineering opportunity, not highest measured speed forecast.

**C24: reusable page arena for segment output, one construction, held.** Replace per-segment high-water Vec capacity with team-owned fixed-size pages, lane-owned current page and a per-segment page chain. Reset ownership after assembly; recycle pages across blocks. Leave T, starts, segment count and lane count unchanged. This avoids retaining a large private slack tail for every historical segment and can avoid copying old prefixes on growth; it does not eliminate A bytes written or n read. Charge page-chain metadata, pointer transitions, synchronization, assembly gathers, false sharing, touched-page counts and retained arena high-water. Page size is a prospective choice, not a sweep to select after timing. **Falsifier:** if per-block growth-copy bytes are negligible and touched-page footprint does not fall, the claimed throughput mechanism is absent; a smaller RSS alone cannot revive it. Do not build until that counted-access census identifies a material cost. The existing 2n reserve is overflow headroom, not a correctness requirement that every byte be touched; lowering it alone can increase copies and stalls.

**C17 fusion stays closed under its old rule.** A streaming ordered-segment RLE consumer could remove the n-byte intermediate write/read, and CRC fusion could remove an m-byte pass. Unlike C23 this changes the post-walk algorithm and segment-boundary/run state. Its old registered four-thread failure is preserved. A changed i9/team regime is a possible future explicitly registered question, not evidence that the old failure disappeared. Do not issue a duplicate build now.

**Capacity trimming alone is not a speed candidate.** Shrinking tt from 5.4 to 3.6 MB leaves the same accessed 4n bytes. Shrinking seg reserves leaves the same A stores, potentially adds growth copies. Releasing dead ll does not make the hardware forget its cache lines; it can incur allocator/first-touch costs. Reuse with matched lifetimes is preferable to repeated frees.

## Price per table: a sensitivity model, not a fitted hit rate

The machine archive says **20 MiB = 20,971,520 bytes**, not 20,000,000. Model only a hypothetical uniform reference/replacement case:
`W = p*(4n + S) + G; h = min(1, Ceff/W)`.
S is effective per-team competing bytes within the relevant reuse window, G shared competition, and Ceff available capacity after other users. This is intentionally a sensitivity calculation. Array allocation, hit ratio and effective reuse-window footprint are not interchangeable. T has structured line reuse and prefetch, other streams have different rates, and phase overlap is not uniform. A reuse-distance/access census is the better next model.

At full n=900000 and Ceff=20 MiB, [model.json](model.json) gives:

| S per team | G | h at four | h at three |
|---:|---:|---:|---:|
| 0 | 0 | 1.000 | 1.000 |
| 0.9 MB | 0 | 1.000 | 1.000 |
| 1.8 MB | 0 | 0.971 | 1.000 |
| 0 | 6.271842 MB | 1.000 | 1.000 |
| 0.9 MB | 6.271842 MB | 0.864 | 1.000 |
| 1.8 MB | 6.271842 MB | 0.752 | 0.933 |

G here deliberately treats the window high-water as competing cache content to show sensitivity, NOT as a measured simultaneous hot set. Using the entire 13.9 MB capacity/team instead gives h≈0.377/0.503 and is unjustified. T alone fits at both p; even full segment prefixes plus T fit at four absent other traffic. Thus the capacity account needs additional phase/stream competition, reduced effective cache, conflict/locality effects, or a rival service mechanism.

A latency-aware sketch is
`d = d_fixed + lambda*((h*Lhit + (1-h)*Lmiss)/MLPeff) + d_stream + d_sync`.
For a proposed Δh, the removable latency term is `lambda*Δh*(Lmiss-Lhit)/MLPeff`, before overhead and before competition changes MLP or service times. None of lambda, Lhit/Lmiss, MLPeff or exposed phase time was measured here. There is **no identified numerical speedup per MB freed**.

At fixed G and otherwise identical behavior, matching the aggregate footprint of three old teams with four new ones requires **25% less effective per-team footprint**, not 25% less allocation. Three versus two needs 33.3%. C23 does not remove 25% of the walk's 4n+A bytes; do not claim it recreates the three-table result.

Per-table observed whole-decoder rates are 319.08 MB/s (4×2), 466.31 (3×4), and 579.28–582.82 (2×4). Transferring the three-table rate unchanged to four teams would imply ~1865 MB/s, **~31.6% wall reduction** from 4×2; transferring the two-table rate to three implies ~19.5–20.0% wall reduction from 3×4. These are explicitly unsupported resource-pool scenarios, not predictions, bounds or additive savings. They change the saturation point and ignore changed k/window/placement/CPU service. The actual paired screen's 3×4 ratio corresponds to **8.82% wall reduction**, not 9.67% wall reduction (9.67% is the speed increase).

## Ranked action and discriminators

| Priority | At 4×2 | At the best three-table plan | Single construction / rejection |
|---|---|---|---|
| 1, existing #220 | Largest directly observed plan-level lead: exploratory 1.0967 speed ratio | Baseline candidate, no further gain | Finish registered three-table test and same-cell confirmation; do not reuse screen data |
| 2, C23 ll reuse | ~5.4 MB capacity reduction, unchanged direct walk traffic; speed unknown | ~4.05 MB conditional capacity reduction; smaller cache-headroom case may mean smaller effect | One cooperative in-place assembly arm, fixed plan/window; failure of whole-decode benefit closes throughput claim |
| 3, existing placement lead | Strong mask contrast; unpinned→fixed-placement gain unpriced | Separate baseline/placement interaction unknown | One fixed thread-assignment arm, same table count/window; no gain or gain only from changed cores falsifies proposed placement benefit |
| 4, C24 segment arena | Removes historical slack/copies only if census shows them consequential | More segments at k=4, potentially more metadata/frontiers despite fewer tables | One page-arena formulation; no fall in copy/page demand closes its throughput premise |
| Deferred | C17 fusion; C15 compact already rejected | No automatic reopening | New causal evidence required; no port or constructor proposed |

For the resource rivalry: (a) capacity/reuse predicts a fall in T miss/service demand when truly touched competing lines are reduced at fixed concurrency; (b) core/queue/MLP saturation predicts gains from placement or reduced issued/prefetch traffic even without a fall in line misses; (c) allocation/TLB predicts copy/page/fault changes without requiring T-cache changes. Do not send identical agents at these hypotheses. C23/C24 and the existing placement arm are distinct, and their gains cannot be summed.

Before any candidate, the narrow census should record per block n,m,s,A, per-segment initial/final len/cap, grows and bytes copied, touched line/page counts by object and phase, plus instantaneous team-phase overlap. Keep it an untimed diagnostic or static replay separate from timing. Allocation is not traffic. If counters become available, require valid full counting fractions and correct decodes under the registered protocol; hardware validation is the Coordinator's work, not performed here.

For a future C23 comparison, fix 4×2 and 3×4 as separate strata, same windows, inputs/build/placement within each; preserve hashes/errors and report build/walk/assembly/rest separately. Predeclare a benefit threshold and CPU/RSS limits, including a matched control and confirmation; this report grants no execution authority. In particular, no new 96%-busy Amdahl credit, no subtraction of old C17/C15 costs, and no addition to frozen A1–A7/D013.

## Prior art and limits

[Denning, The Working Set Model for Program Behavior (1968)](https://www.denninginstitute.com/pjd/PUBS/WSModel_1968.pdf) defines working sets through recent references, motivating the phase-sensitive distinction from allocation. This report's cache-line/phase application is an inference, not a calibrated prediction from that paging paper.

[Rust Vec documentation](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.clear) specifies retained capacity on clear; reserve may overallocate and its growth strategy is not guaranteed. The tested source and measured capacities, rather than an assumed allocator doubling rule, ground the arithmetic here.

Evidence boundaries: static source audit, archived run replay and read occupancy summaries; no per-instruction profile replay, hardware counters, new timing, constructor, candidate compilation, fresh independent qualification, or production changes.
