# Run-fed construction: make the leader saving survive the helpers

Assignment [6729426658](https://github.com/lmmx/mibz2/pull/160#issuecomment-6729426658).
Reviewed source: **187c0ba517c3324aa28c5e7ccd2667f6c0572145**, #269.
Governing specs: **a4f97714cc14366984015869d9d094d89488ac57**.
Disposition: **continue with bounded diagnostics and a distinct sort-free proposal; no production qualification.**

## Decision

First isolate the cheaper MTF list kernel. Then compare two run-fed helper constructions: parallel stable sort plus bounded row fill, and **direct F-interval placement without sorting**. The latter removes the sorted-record materialization, while retaining the existing dense T table and walk. It pays for L-byte predecessor lookup and scattered output streams. That is a concrete rival with removable work, not a promise that compression wins.

Keep #269's one-worker result and its leader-with-sort result scoped to their actual candidates. Neither closes helper-side construction. Conversely, an isolated 25% emitter saving cannot qualify a pipelined change.

Outside this front, fixing and resolving #258's admission HOLD remains a cheaper existing lead than a replacement walker. Researcher 6729358962 and Coordinator 6729367020 now explicitly HOLD queue admission; the earlier description “ready” is obsolete. C25 selected-boundary validation remains a narrow alternative that preserves the current representation. Neither inherits measured credit from this report. D042's separate pipeline B5 HOLD remains unresolved by these proposals.

## 1. What the pin actually establishes

Source read: run_stage.rs, mtf.rs, examples/run_fed_stage.rs, parallel.rs and coop.rs, plus both run-fed journals. No candidates compiled, no tests or timings run, no published timing bundle replayed. Journal test outcomes are reported claims; this is not B5 certification.

**MTF confound.** mtf_runs and plain mtf_avx2 share the two-register list kernel. mtf_checkpoints_avx2 uses a different memory-list implementation. The journal discloses this in §5. Therefore B-emit/C-front combines list-kernel changes, removed checkpoint operations and changed output stores. “SIMD against SIMD” is insufficient attribution. Propose a modern-list checkpointed control preserving existing output and checkpoint semantics, independently of any run-table change. Its possible benefit is bounded by the replaced MTF cost, not all leader work; magnitude unknown.

**Overstores.** The AVX2 fill writes 8 entries unconditionally per cell, continuing in groups of 8. For cell lengths ell_i and cell count m:

W = sum_i 8 ceil(ell_i/8), with n <= W <= n + 7m.

The table has 4n logical bytes; the store instructions cover 4W bytes, including repeated addresses. Using #266's pooled m/n=.0881 only as an illustrative population gives W/n <=1.6167, not an observed ratio for #269's selected blocks. Obtain the actual W census. The 91.7% short-cell figure does not determine W. Extra stores need not become extra DRAM traffic, but can consume store-buffer/port capacity.

Serial overwrite order justifies the current overstores. Independently executing shares cannot write past their ownership limits, even if a later store would repair the value. fill_rows is a scalar Vec append interface, and its tests append shares serially. They do not establish concurrent AVX2 writes. A proposal needs exact bounds, complete initialization and the existing build-to-walk publication barrier; use bounded vector bodies and tails or disjoint aligned tile ownership.

**“Floor” is a probe, not an architecture theorem.** floor_avx2 is a scalar Rust enumerate/write loop under an AVX2 target attribute, not an explicit copy of fill's vector-store loop. In the diagnostic its contents are overwritten by build and the vector length remains zero. Check the existing compiled listing for emitted stores and dead-store elimination before interpreting it. The harness times each phase between Instant calls, but there is no explicit black_box of the floor contents. The warm intervention also changes cache ownership and immediately precedes build. R_floor around .5 and R_warm around 1 are disclosed two-round local indications only. They do not identify the cause of the remaining half, nor prove that non-temporal stores cannot help. M3's cycles/cell cannot alone isolate an iterator dependency chain; vector output, contention and stalls remain in its numerator. Nominal-GHz-scaled elapsed “cycles” and PMU cycles must stay distinct.

**Fallback.** takes_runs uses 8*symbols <= max_len, not actual n. A short block may take the record path at a poor r/n. Actual record use is 4(r+1) plus 8(r+1) F records, before capacities and output buffers. Same final 4n table does not imply equal front working sets or identical traffic. Persistent high-water capacities and two front buffers must be charged.

## 2. Price the shipped 3×3 path

parallel.rs runs the leader's previous-block RLE1/CRC, next-block MTF, and possibly tiny whole blocks while two helpers construct and invert the current block. coop.rs does BuildSetup before pool dispatch; the leader can join walking after its work, so helper completion depends on leader arrival.

Use the explicit model:

T0 = U0 + max(L0, P0(L0)) + D0
T1 = U1 + max(L0 - (M-E) + A, P1(L1)) + D1.

M is checkpointed MTF, E run emission, A any new leader-side validation/prefix/allocation, U is pre-dispatch setup, P includes helper construction, barriers, walking and assembly, and D covers drain/publication effects not included elsewhere. P(L) is arrival-dependent; this is not a fixed two-stage roofline. Scanner supply and ordered output can cap the whole decoder further.

Moving sort off the leader releases M-E-A there. Under the disclosed LOCAL hypothesis E=.75M, the gross release is .25M, not 25% of leader time or decode wall. A serial helper sort still adds S on the helper critical path. Parallel sort adds histogram/prefix/scatter/barriers and skew, not S/2 by assumption. Whole CPU changes by E-M plus helper construction CPU minus old build CPU, plus setup/synchronization and changed joining/parking. A shorter leader may help the walk, but cannot spend the same saved cycles twice.

Illustration only: set one-thread old build B=1, fill F=1 and sort S=.25, all from the unqualified shape of the local report. At two perfectly balanced helpers, old build costs .5. Serial sort then parallel fill costs .75; ideally parallelized sort then fill costs .625. Added helper latency is .25 or .125 before barriers. The leader release must survive this and the subsequent walk. These are normalized hypotheses, not i9 forecasts.

On Plan::Workers there is no hidden helper sort: compare E+S+F against plain MTF+build. On a four-core CI runner, forced 3×3 can establish correctness but oversubscription cannot establish the i9 ten-core benefit. Keep the production CI grid and acceptance rule unchanged; any representative workstation evidence needs C5's exact registered recipe and Louis's run.

## 3. Ranked constructions and cheapest falsifiers

Order is marginal information value, with construction risk charged. All are proposals.

| Rank / ledger ID | Mechanism and possible saving | Added cost / risk | Cheapest CI falsifier proposed |
|---|---|---|---|
| 1 / C29 | Modern-list checkpointed MTF control: remove list-memory operations while keeping L/checkpoints and existing build | Checkpoint boundaries and malformed-stream semantics; benefit may already be negligible | First compare existing listings and source-path counts. A separately registered matched phase control then attributes B/C; no checkpoint-removal claim if the modern-list control explains the gap |
| 2 / C30 | Leader emits records; helpers parallel-sort then fill row-balanced disjoint shares. Releases M-E-A; can remove old checkpoint setup | O(r) extra histogram/scatter, prefix barrier, tail handling, front lifetime, helper delay | Untimed partition/ownership and word-equivalence checks first; then registered construction phases including both barriers. Reject the proposed stage on measured whole-path loss, not emitter ratio |
| 3 / C31 | Direct F-interval placement: eliminate F-order buffer, scatter and reload; keep dense table/walk | L predecessor queries, bucket-stream stores, boundary false sharing; run-length load imbalance | Static census of per-run predecessor work, cell count, stores and helper row/record weights. Do not build if this already exceeds the removed sort budget; otherwise one registered matched helper-stage contrast |
| 4 / C32 | Bounded tiled fill or multi-cell batching removes overstores and exposes independent cells | Tile setup/masks, packing, dependencies or L1 scratch; fewer stores can still mean more instructions | Census W/n first. Existing compiled store/uop shape then decides whether bounded fill plausibly pays. Registered fill+walk contrast, not store-only throughput |
| 5 / C33 | Stream long full cache lines non-temporally, retain ordinary writes at boundaries; or tile high/low-field production | Immediate walk consumes table; fence/publication, RFO savings versus cold reads, concurrent teams' cache effects | Listing/ownership review first. Only after evidence of exposed write traffic, a registered cached/streaming construction+walk contrast with same buffers and table sizes |
| 6 / C34 | Lazy materialization or hybrid implicit interiors reduces eagerly written entries | Per-step state/predecessor/decode, first-touch stalls and marker synchronization; likely sacrifices current walk's simplicity | Untimed touched-line/interior-hit census and source instruction budget first. No new walker until omitted-write benefit exceeds dependent lookup and coordination cost |

### C30: a parallel stable counting sort needs two histograms

Split the original records into ordered contiguous ranges. For helper h and byte c compute:

- H_h[c]: number of records;
- L_h[c]: sum of their run lengths.

For each byte, O_c is the prefix of global RECORD counts across bytes, while C(c) is the prefix of global ROW counts. A run's destination slot is O_c + sum(g<h) H_g[c] + its local occurrence number; its a is C(c) + sum(g<h) L_g[c] + its local preceding length sum.

Counting records alone gives placement but not a. Counting rows alone gives a but not the compact record slot. Two u32 histograms cost 2 KiB/helper (4 KiB for two), before offsets/padding. Charge histogram scan, reduction/prefix, scatter, visibility barrier and the subsequent fill. Validate rising L starts, terminal n, byte sums, overflow/range limits before the first unsafe destination write; keep the table unpublished until every share completes.

Split fill by q rows, preferably cache-line boundaries, not byte classes. Byte-class work can be heavily skewed; max_c r_c bounds indivisible class balancing, and naive class assignment rereads all input per helper. Stable record-range sorting avoids that reread, but its bucket boundaries can share cache lines. In a 3-thread team only two helpers do construction: a “3-way” sort would steal the leader's overlap.

### C31: remove the sort, not just move it

A direct scatter can construct the SAME T. For ordered helper ranges define L_h[c] as above. For a run (c,p,ell) owned by h:

a = C(c) + sum(g<h) L_g[c] + t_h,c,

where t_h,c is the length of c-runs previously processed by h. The interval [a,a+ell) belongs to this run and Ψ(q)=p+q-a there.

These intervals partition every F bucket because lengths prefix in stable L order. All buckets partition [0,n). No sorted record array is required; only per-helper length histograms and prefixes. Original L records remain read-only. For each assigned F interval, locate the L run containing a, advance through its L boundaries, and emit exactly:

T[q] = L[q] | ((p+q-a) << 8).

**The low byte is L[q], not c.** Confusing them yields a plausible but wrong table. The total number of nonempty intersections is still m<=2r-1: processing order does not change the refinement. What changes is the r predecessor starts and output order. This removes 8(r+1) live bytes of F-order records and their construction/read traffic, plus the record-count histogram; it retains 4(r+1) original records and 4n table bytes. Capacities and fallback storage still matter.

Binary search costs O(r log r) comparisons and potentially dependent loads. A directory D[j] giving the L-run index at row jK bounds each lookup to the runs within one K-row bucket. Search between adjacent directory indices; build the directory in O(r+n/K), charge 4 ceil(n/K) bytes plus a sentinel. For K=256 and n=900000 that is about 14.1 KB, with at most roughly K starts in a bucket; no uniform-density assumption is needed for the bound. It is not free: count actual comparisons/cache lines and directory construction before selecting K. The directory is proposal metadata, not measured benefit.

Helpers owning contiguous L-record ranges own, within each byte bucket, contiguous F portions. This limits cross-helper bucket boundaries to at most 256(k-1), but false sharing and bounded tails still need treatment. Equal record counts do not imply equal row writes. Partition by a cost weight combining records and rows; a huge run can be virtually split at a helper boundary, preserving the c-length prefix, at additional metadata cost and up to one additional refinement boundary per cut. Never inherit the whole-table overstore kernel for disjoint intervals.

This rival trades the monotone sweep for the same bucket streams production already uses, but at run granularity and without L materialization. #266's 82% existing run-store coverage is precisely why speed remains uncertain. Its attraction is the removed sort materialization and shorter leader path; its falsifier is lookup/scatter cost exceeding that saving.

### C32–C33: store count before store policy

A global high-field pass followed by low-byte read/modify/write is approximately 4n write +4n read +4n write =12n logical table bytes, versus a single exact 4n write. It can nevertheless expose parallelism; it must earn that extra traffic. An L1-sized scratch tile can form both fields then write T once, but adds scratch traffic and needs efficient interval access per tile. The proposal is cell batching/bounded tiling first, not unconditional whole-table two-pass filling.

For streaming writes, give full cache lines a unique owner, use a justified completion fence before readers, and keep marker writes ordered after initialization. It may save write-allocate traffic and preserve other teams' cache lines; it may equally penalize the immediate room walk. Arm D alone cannot decide this. Smaller entries require changing the consumer and marker representation. A 3-byte layout reduces logical table bytes by 25% but adds packed-load extraction and boundary work; it is a distinct representation experiment and earns no credit from “4n floor.”

### C34: first touch and implicit interiors

For a block whose Ψ walk visits all n rows, lazy filling cannot eliminate the final initialized table: it only changes when rows are written. Periodic/repeated traversal can differ, so census distinct touched rows/cache lines using the actual room walk before claiming savings. Segment markers are written before walking; lazy initialization must not overwrite them. A line state needs exclusive construction and ready publication; a waiting lane loses memory-level parallelism. This is a large ownership change for an uncertain omitted-write prize.

A head-plus-offset table still needs to identify the F interval containing arbitrary q and recover L[q]. A lane's previous interval cannot generally identify the next one: Ψ jumps. A coarse directory plus exceptions is a plausible hybrid, but must price every dependent load, extraction, marker and fast-forward. Smaller footprint alone does not repeal the prior move-walk result. Preserve that verdict; a different direct-index hybrid needs its own count budget before implementation.

## 4. Additional structure worth using

RUNA/RUNB determine expanded lengths independently of MTF's byte permutation. Thus actual n can in principle be recovered without resolving byte values. It is not free: an extra symbols pass charges O(|syms|), while fusing it into Huffman scanning adds work on the shared scanner. Do not trade leader work for a new scanner bottleneck. A post-emission fallback can expand already decoded records into L instead of repeating MTF, but still pays the record emission and expansion and may retain both buffers. Compare it with today's up-front rule, especially final/short blocks.

Similarly, accumulating per-byte run counts in the emitter might remove a sort histogram pass, but adds dependent counter updates on the exact leader loop being shortened. It cannot supply per-helper prefixes without more state. C31 avoids the record-count histogram entirely; prioritize that formulation over moving a histogram onto the leader.

## 5. What to freeze next, and what this report changes

Propose a bounded diagnostic sequence under C1: (a) count exact stores and inspect emitted floor/control code; (b) distinguish the list-kernel control; (c) choose between C30 and C31 using lookup/partition census; (d) only then a paired whole-stage/pipeline experiment with setup, waits, CPU, RSS, fallback and correctness included. Coordinator registers; Researcher assesses. No dispatch or new workload/thread grid is authorized here.

#269 is NOT FROZEN at this pin. Its Q1 STOP and Q2 NO LEADER LEAD conjunction should not be worded as closing all helper-side formulations: Q2 includes the leader's sort. Record any later registered outcome unchanged; a new construction gets a separate registered question. Its current instrumentation can provide R_emit as an input, not a production verdict.

Four C3 questions:
- Expected: a checkpoint-free leader might pay for extra run construction. Found: the emitter comparison is confounded and the wide fill stores more than n words; helper construction is still unmeasured.
- Gap closed: exact source ownership/cost terms and a sort-eliminating derivation. **Zero measured decode-wall gap closed; zero new speed credit.**
- Mechanism learned: dense table semantics do not require F-order record materialization; the price is predecessor work and bucket-stream locality. Leader release and helper delay must be compared on the same schedule.
- Next value: cheap control and census distinguish these formulations before a parallel unsafe integration. This outranks a lazy/move walker; #258 HOLD remediation and C25 remain economically competitive outside the front.

## Evidence and scope

All source links below are immutable at the reviewed pin:
- [run_stage.rs](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/src/run_stage.rs)
- [mtf.rs](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/src/mtf.rs)
- [diagnostic](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/examples/run_fed_stage.rs)
- [pipeline](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/src/parallel.rs), [cooperative relay](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/src/coop.rs)
- [#269 draft](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/docs/journal/2026-11-09-run-fed-stage.md)
- [#266 census and models](https://github.com/lmmx/mibz2/blob/187c0ba517c3324aa28c5e7ccd2667f6c0572145/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md)

C31 and the cost equations are derivations here, not literature claims or tested constructors. Existing census numbers are attributed to the pinned journal, not independently rerun. Routing audit found only the five recorded resolved historical incidents; no new misroute. Migration manifest and old head verified before receipt. State-only commits; no source change, timing, test execution, CI dispatch, merge or automation change.
