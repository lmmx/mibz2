# Addressed-width draft: REVISE before freeze

Inbox [6760363087](https://github.com/lmmx/mibz2/pull/160#issuecomment-6760363087). Draft **f918f783aa6b9f683d867c057a4c4937c99392ca**, docs/journal/2026-11-10-width-intervention-draft.md, blob **d16f1c6bc8e1f645616df9d8f62cc947e25c6c20**. Specs a2da49638ac54d7763de1c167bd054f67ffdc763.

**Keep the eight-cell layout experiment; revise its machine-equivalence claims and decision interpretations.** The hot recurrence can be instruction-identical using scaled physical indices. Whole-walk equality is unproved, and construction is changed in both arms. B4/A4 is a low-concurrency layout contrast, not an isolated side-effect control. exp(I)<1.15 cannot kill every compact-table premise; exp(I)>3.5 cannot restore the withdrawn law.

An untimed, matched count/semantic admission stage comes first. 210 timed slots remains correct arithmetic, not a complete runtime budget or readiness verdict. Team ports 2 then 3 retain priority as D085/D087/B5 allow.

## Evidence and limits

Source at the draft pin: src/bwt.rs blob **dce242e882f23369a53ea7e3e0a28ce8b9340ffe**, src/rle.rs **5f3bc36af1cc2f514d293f5c3b38be12413e78cb**. The BWT source matches the prior review's 4ce34b6093 pin. It contains production kernels, not the proposed stride implementation: the draft expressly says nothing built.

Historical variance replay uses share-nothing/20261110T052925Z, archive SHA-256 **1e347594515a2215e63fb186c5681f329046844a62ff71f1617763d2f68136ad**, rechecked here. Joined slots to runs by recorded line and excluded round0. The registered historical judge is bench/share_nothing.py at **482df588e2a97b99c6797c7bc2a24098015968dd**, blob **7c68d4316a3f3e324a7834d7f152c4889504e761**. It uses a moving-block bootstrap over chronological rounds, block5 with1/10 sensitivity. No new decoder execution, timing, build, candidate implementation or safety certification.

## 1. Scaled links: valid recurrence, incomplete equality claim

Let stride s=4k bytes, k=1 or4, logical row r, physical u32 index j=kr. Store at word j:

q = byte | ((k*next_row)<<8).

Then both arms may execute the same recurrence: q=load(base+4*j); j=q>>8, with the same START-bit test and the same prefetch address base+4*j. The shift count is8 in both. This makes an identical dependent-load/prefetch sequence **possible**, not a demonstrated listing property.

On x86 the physical index fits the scale4 addressing form. A generic implementation that instead computes base+stride*logical_row needs the stride16 address calculation; a runtime stride may require a multiply or extra address operation in both arms. Const-generic specializations need not emit identical instructions or placement. Do not blindly scale both stored links and typed record-pointer indexing: with a 16-byte element pointer, indexing by4r addresses64r, not16r.

The existing room walk also:

- reads the primary entry at orig_ptr;
- calls random_rows(n), adds the first linked row, sorts/deduplicates starts and marks their table entries;
- looks up a segment by binary search on starts;
- takes segments, prefetches links, handles tails, clears flags and assembles output;
- derives buffer sizes, stopping conditions and the short-block fallback from n.

All row-valued quantities must use compatible units. Preserve **logical n**, original pseudo-random logical starts, segment identities and all control paths. Convert starts to physical indices once, or decode at boundaries; count whichever conversions are chosen. n must not become the backing word count kn: that changes random starts, short-block selection, buffer capacities and work. orig_ptr needs its initial k scaling; flag cleanup, fallback and lookup must agree. Positive scaling preserves sort order, but only if every searched key is scaled consistently.

The flag proof is k*(n−1)<2^23, because shifted link bit23 occupies bit31. k4 permits n<=2^21; level2's below2^18 rows is safe for that arithmetic. The production assertion n<=2^23 alone does not prove the scaled representation. Arithmetic fit is not an unsafe-memory proof: alignment, allocation extent, initialized addressed words, pointer validity and capacity growth remain separate obligations.

**Construction does not inherit the hot-walk equality.** The source's one-row step stores ll[s] | (i<<8) at logical destination s. At width16 it stores ll[s] | (i<<10) at physical word4s. Changing a constant shift8→10 need not add an instruction. Keeping s logical may add a shift/address operation for the store; keeping s physical may instead add a decode for ll[s]. Scaled bucket bounds/cursors and tails also cost work. A compiler may absorb some of this; no fixed numeric delta is knowable without the implementation and listings.

Charge a possible extra operation per scatter as **N extra dynamic instructions per decode** if the listing/count establishes one per row, plus conversions/setup/callees. Do not book that as achieved or inevitable. A shared physical-index hot walker is worth considering to minimize code differences; wrappers and build can remain specialized. That is a proposal, not code here.

### Count and semantic admission before workstation preparation

Require the same source/build/toolchain/flags, pinned executable hashes, same input blocks, logical segment starts, lanes, prefetch policy, tile size, allocator policy and markers-off. Archive matched dynamic Ir for:

1. build including bucket checks, allocation/initialization and fallback;
2. full walk including starts, flag work, take/finish/grow, tail and assembly;
3. inclusive decode_block, with MTF/RLE/CRC and callees reconciled.

Keep BWT-row and decoded-byte denominators distinct. Count corresponding blocks and whole-input totals, including short/repetitive/tail cases, not a favorable hot-loop listing alone. Separately compare the new one-row A arm to current production to expose the cost of removing run stores. Equal counts are not equal latency, but unequal counts must enter the price. Calls outlined by instrumentation must not disappear from attribution.

Before any native run, byte-exact oracle comparisons and malformed/error behavior must cover scaled primary/start/flag boundaries and allocation limits. If uninitialized padding is retained, the backing representation must not expose uninitialized u32 values through a fully initialized slice/Vec contract. Zero-filling padding instead adds writes and contradicts “never written”; pick and prove one design. B5 remains applicable.

The draft's “every record visited once a walk” is not a general source invariant. Source documentation explicitly allows multiple cycles; assembly can repeat the primary cycle, and segments on other cycles need not cover every possible row. A periodic/self-loop table is a counterexample to unconditional all-row traversal. The builder initializes every logical row; **walk coverage and repeated marker reads need their own counts**. Do not infer touched walk footprint merely from allocated span. This does not establish a coverage defect on this particular Wikidata input; that is unmeasured here.

## 2. Side effects and corrected byte arithmetic

For n=200,000 logical rows and 64-byte-aligned storage, addressed-line spans are 12,500 versus50,000 lines. Payload stores by a truly one-row builder are **4n bytes in either arm**, not16n in B. Unique cache lines dirtied, line transfers, repeated ownership acquisitions and DRAM traffic are different quantities.

For block rows n_i and N=sum(n_i), the unique line-byte sum per build over the decode is:

A: 64*sum ceil(n_i/16); B: 64*sum ceil(n_i/4).

This counts the same reused line again when another block is built, deliberately; it is not the distinct physical allocation size for the process. It excludes marker read-modify-writes and does not count repeated fills/evictions during one build. Actual traffic may exceed it, while memory-level traffic can also be hidden by cache residency.

**1.5→5.9 GB is 1,831 times a full200k-row table, not the observed sum of rows.** An independent bound already refutes those values as unique line-byte sums for this input. RLE1 consumes either one encoded byte for one output byte, or five encoded bytes for at least four output bytes. Hence for a successful decode N<=5D/4. With D=209,700,196, N<=262,125,245. With at most63 bytes line-rounding per block, the two upper budgets are **1,048,616,333 B and4,194,119,273 B** (~1.049/4.194 GB). Actual row sums should replace these loose bounds. Larger measured memory traffic would not contradict this bound: repeated line transfers are a different metric.

At exactly200k rows, 4KiB page counts with page alignment are **196/782**; with only64B alignment they may be **197/783**. “200/800” is a rough approximation, not a measured page census. THP/huge pages, actual page offsets and allocation slack must be recorded.

Costs still to charge:

- L1/L2 line density, shared-cache set conflicts/replacement, prefetch effectiveness and outstanding misses, not just L3 aggregate capacity;
- TLB/page walks, page-table cache footprint, page size and first-touch placement;
- write allocation/ownership, dirty evictions, write-combining/store-buffer behavior, up to256 bucket streams, and their dependence on W;
- initialization/zeroing, allocation growth and old+new peak, minor faults at first touch, kernel CPU, and RSS;
- table plus ll, symbols, segment buffers, tile, outputs, queue-window retention and scanner/consumer data;
- construction policy's effect on phase ratios, feed pressure, tail and concurrent worker activity.

Scratch is reused across blocks. Page faults are not automatically1831 times a full-table page count: many pages are first-touched once per worker/process and reused. Conversely capacity growth can temporarily retain old/new allocations. +2.4MB/worker describes the maximum logical span difference, not necessarily measured resident or peak memory. “Four times as fast in lines” means four times the potential line demand per row, not a fourfold measured bandwidth rate.

### Why B4/A4 does not remove these costs

Four maximum B tables occupy12.8MB nominally, but that is not a proof that all active data fit in a20MB LLC. Other buffers and threads, private-cache behavior, mapping and retained capacities remain. E4 includes table locality effects too. A known fitting aggregate would still not isolate TLB from build traffic or construction instructions.

The interaction subtracts **log effects**, not components of CPU time. Exact cancellation requires a common multiplicative factor on the measured whole-process CPU. Even a W-invariant multiplicative penalty k on worker work V does not generally cancel when total CPU C=H+V:

width factor at W = (H_W+kV_W)/(H_W+V_W).

Its value changes with the non-worker share. Using the draft's rounded totals with reconciled residuals H4=.1005,H10=.1099,V4=.582,V10=.683, k=2 gives exp(I)=1.00467 despite no W-dependent penalty on worker work. An additive cost also does not generally cancel. Traffic/page-walk costs can themselves scale nonlinearly with W and mimic a knee. State the identifying assumption as unresolved; do not use E4 as a causal subtraction.

The new scalar builder also changes A's instruction cost, f and occupancy relative to archived production. Source/semantic input equality does not hold those quantities fixed. Rounded prediction factors reproduce approximately1.415,2.853,4.411 using the supplied worker multipliers, but the multipliers and archived baseline are unvalidated for this new kernel. They are scenario calculations, not discriminating model distributions.

## 3. Price and order

**210 timed slots still holds:** (8 cells+2sentinels)*(20retained+1warm). The layout does not change that integer. Optional2 -9 on-only references add42→252. If a current-production -2 W4/W10 native anchor is wanted because A's constructor changed, it is another2 cells/add42; both additions give294. Archived production is context, not a matched contemporaneous control.

The untimed stage above is additional work: implementation by the Coordinator, correctness/safety evidence, build and listings, matched profiles and parser reconciliation. It cannot be hidden in210 slots. No exact build/profiling duration or instruction delta exists yet. Prefer two sequential marker-off matched counting arms and the production anchor before proposing more native cells; required repetitions/coverage belong to the prospective count plan.

The draft now has **five** clocks/block: MTF/build/walk starts, walk end, decode end. At1,831 blocks that is **9,155 reads/decode**, nominal228.875 microseconds summed CPU at25ns/read, plus logging, allocation, placement and dump costs. Four marker-on cells over21rounds produce769,020 reads, nominal19.2255ms summed clock CPU; this is not exposed wall overhead. Record capacity/validity, and define optional -9 geometry safely: its 900k-row blocks cannot be silently sent through a level2-only layout contract.

A clock envelope measures phase-active wall intervals, not running cores. CPU/wall estimates average CPU occupancy over its stated denominator, not an instantaneous active-worker count. Per-thread sampled CPU has quantization and end-of-thread omission; retained exact accounting must say which numerator and interval it uses. Trace corruption may invalidate the trace separately from clean marker-off results only if that split is frozen; semantic/output/identity failures invalidate affected performance comparisons. “Material marker movement” needs a prospective threshold and uncertainty rule.

Before freeze the missing slot bounds, timeout/cleanup behavior, failed/duplicate/missing-row admission, complete round order, clock/log schema, hashes and analysis code must be fixed. The archived schedule was seeded per-round shuffle, not a guarantee of exact positional balance; define the new schedule rather than claiming it is literally the old one. No workstation queue or timings authorized by this review.

## 4. Kill bands: keep a bounded decision, remove universal inference

I is a **width-by-W interaction** on this changed-constructor, fixed-input decoder. It is not the width main effect, a decomposition of -9, or a test of every compact representation.

Counterexample to the proposed universal kill: B4=2*A4 and B10=2*A10 gives exp(I)=1 while width doubles CPU at both worker counts. That is a large width cost without a differential W4→10 knee. It already shows why all compact-table premises do not die below1.15. A negligible effect for expanding0.8→3.2MB on -2 also need not transfer to shrinking3.6MB on -9, another constructor or other machines. Conversely, a large positive interaction can come from traffic/TLB/fault behavior, so >3.5 does not prove the literal law or explain away its recorded predictive failure.

Proposed replacement interpretation:

- If a prospectively valid interval's upper end is below1.15, rule out an interaction of15% or more **for this layout/input/W contrast**, under its stated uncertainty method. Report E4 and E10 separately; do not kill compact representations.
- If its lower end exceeds1.15, observe a material positive interaction of this complete layout intervention. Its mechanism is unresolved.
- Otherwise, precision insufficient for that15% decision. Negative effects are reported explicitly.
- Treat1.4/2.8/4.4 as labeled scenarios, not causal model-selection bins. A matching magnitude is not model identification.
- If economic continuation is tied to15%, derive it from a named candidate's complete construction/scratch costs and benefit target. No such derivation is in the draft. A15% information threshold can still be chosen prospectively, but “chosen” is not “proven to kill a line.”

There is also an uncovered2.8–3.5 range. Assign it an explicit descriptive outcome, not an implicit preferred model. Since E4 is a log difference, “E4 above1.5” is ambiguous: exp(E4)>1.5 means E4>log1.5≈.4055; E4>1.5 means a4.48-fold ratio. Specify units. No E4 cutoff isolates side effects from genuine cache effects.

### Precision replay

Using all60 retained historical rounds:

| series | sample SD |
|---|---:|
| log CPU(-2,W10/W4) | .105273 |
| log CPU(-9,W10/W4) | .057699 |
| paired difference of these logs | .111984 |

-2 round26 is .794156 versus median .150216; it is retained, not discarded. “Near.05 at each level” is false for the actual retained -2 series. Taking the historical paired SD as an iid planning proxy gives SE20=.02504, not.022. A normal1.96 half-width is .04908log (roughly5%); a t19 multiplier gives about.0524log. Thus ±5% is a plausible rough scenario, **not** coverage or power evidence.

These levels are not new width arms; their covariance does not identify the new four-cell covariance. The archived moving-block method with block5/10 and only20 rounds resamples effectively few long blocks; inherited validity does not follow. The Researcher should fix the prospective joint uncertainty treatment for E4,E10,I, dependence sensitivity and any multiple decision rules. Interval-inside-band rules still need calibrated intervals. No causal conclusion is licensed by narrowing them.

## Four answers and next value

Expected: scaled links eliminate instruction confounding, four-worker contrast removes side effects, bands distinguish models. Found: only the hot recurrence has an algebraic equivalence; construction and boundary work differ, side effects interact with W, and the proposed bands overreach. Closed: physical-index contract, exact nominal page/clock/slot arithmetic, a row-volume bound, historical variance correction, and explicit null-interaction counterexample. Unclosed: actual Ir/codegen, addressed walk coverage, retained/peak footprint, perturbation, precision and candidate economic headroom.

Next: keep the eight-cell design, first obtain its matched untimed count/semantic evidence, and rewrite the decision as a scoped layout interaction. This is worth a bounded falsifier; a new standalone census is not. Team-port admission repairs remain ahead of it. No build, timing, implementation, safety discharge, model rehabilitation or production qualification by this role.
