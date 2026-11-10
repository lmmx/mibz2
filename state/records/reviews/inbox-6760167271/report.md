# Phase census versus addressed-width intervention

Inbox [6760167271](https://github.com/lmmx/mibz2/pull/160#issuecomment-6760167271). Specs a2da49638ac54d7763de1c167bd054f67ffdc763. Proposal and source audit only; no implementation, compilation, decoder execution or new timing.

## Decision

Merge a **bounded phase census into the addressed-width intervention**, with matched instrumentation controls. Do not spend a separate 60-round, two-level grid to select a causal law from observational overlap. Four timestamps are useful phase occupancy data; they do not identify walking-table versus held-table versus core/bandwidth causation.

Price scanner and representation together on the whole decoder, but retain their separate marginal costs. The claim that every small table needs parallel scanning is not established: even the quoted proxy contradicts it for g=1.18. Team ports 2 then 3 remain ahead of new three-table constructions, subject to D085/B5. A diagnostic can inform the latter without qualifying any candidate or relaxing its count/safety restrictions.

## Pins and evidence

- Pathfinder [proposal 6760158147](https://github.com/lmmx/mibz2/pull/145#issuecomment-6760158147), [correction 6760156549](https://github.com/lmmx/mibz2/pull/145#issuecomment-6760156549).
- Hypothesis register at **4ce34b6093a0f7a10885004dc20c8138602b1ae2**, blob **788485e8e8a78eab1481a0dce9677439f650aca3**.
- Same source pin: examples/ab.rs **f7f4dfbfa7b84d656b388f88dd09c49360519d13**; src/parallel.rs **a425a3cd3939f74d9091fca7906122766097e84a**; src/lib.rs **fa47ab3f02d9d7fd37485cdb6b83aa4cffaaa1c7**; src/bwt.rs **dce242e882f23369a53ea7e3e0a28ce8b9340ffe**.
- Previous [replay/report](https://github.com/lmmx/mibz2/blob/39c1cd79ffb1f14124aae0a0fb7df8d7c1bcfb21/state/records/reviews/inbox-6759983287/report.md), [REVIEW 6760081653](https://github.com/lmmx/mibz2/pull/145#issuecomment-6760081653): bundle SHA verified and 732 historical slots replayed then. No additional measurement here.
- Correction's worker-CPU normalization to W2 is distinct from that replay's whole-process normalization to W4. Do not merge their ratios into one series.
- #335 reconciliation 6759906409 reports duplicated raw/gzip profiles resolved; D085 FAIL-MEASUREMENT remains. Not an independent raw-profile audit here.

## 1. Complete instrumentation price

Four reads per block, once per assigned worker, gives the following. Do not multiply by W again.

| input | blocks/decode | reads/decode | read-only CPU at assumed 25 ns | record storage at illustrative 64–96 B/block |
|---|---:|---:|---:|---:|
| -9 | 421 | 1,684 | 42.1 microseconds | 26,944–40,416 B |
| -2 | 1,831 | 7,324 | 183.1 microseconds | 117,184–175,776 B |

These are arithmetic prices, not calibrated overhead. At 100 ns/read they are four times larger. At the previous -2 W8 wall approximately .113 s, 183.1 microseconds is about .162% of wall **as summed read CPU divided by wall**, not predicted wall regression; reads overlap across workers. The critical finishing worker's reads, scanner feed, queue and output tail determine exposed cost.

Eight measured cells (two levels, W4/6/8/10) times 60 rounds give 2,161,920 reads: .054048 s summed clock CPU at 25 ns, plus 34.59–51.89 MB of illustrative record payload across decodes. Warmups, headers, capacity slack, allocations and serialized formatting are additional. Four uint64 timestamps alone are 32 B; block identity, worker identity, row count, validity and capacity/epoch metadata determine the real record. A 64–96 B record is a budgeting assumption, not source-defined storage.

The new eight-cell census with two sentinels and one warm round plus 60 retained rounds is **610 slots**, not the old grid's 732. The old approximately 395 s covered its own 12 slots/round; it cannot be asserted unchanged here. Dumping at process exit may be outside the decoder interval but remains inside runner cost unless explicitly excluded. Document both.

**Instruction and placement cost is not four clock calls alone.** Include flag checks, clock implementation/serialization, return-value moves, stores, pointer/capacity updates, per-worker setup, final gathering and errors. A library clock may use a userspace fast path with retries; 25 ns does not establish instruction count. A per-event Vec growth, TLS lookup, shared atomic cursor, mutex or syscall would change the price materially. Preallocate private worker records; overflow must invalidate the diagnostic, never silently truncate or wrap. Resolve the flag outside hot loops where possible. No marker inside a per-row loop.

Boundary calls can alter inlining, register allocation/spills, branch alignment, caller placement and data allocation geometry. Changes to whole-decoder performance cannot be bounded by summed clock nanoseconds. Keep symbol addresses, relevant listings, build flags and actual allocation/capacity ledger; compare marker-on/off in the same binary, and inspect against the pre-instrumentation reference. Same-binary off still contains placement changes, so it is not a pristine baseline. The existing room kernel is inline(never), which helps preserve its own instruction body but does not freeze caller/data geometry.

Use one monotonic, cross-thread clock. Unserialized or unsynchronized per-core counters are not a substitute. Record failures, block rows, stable block index, worker id, process/round id and start/stop completeness. Validate ordering and match all successful blocks to the source stream. State whether measurement includes logging and dump; preserve byte/CRC/error checks. Per-worker buffers must not share written cache lines.

### What four edges actually measure

ab's workers arm uses Plan::Workers and **2*workers slots**; the team arms use SLOTS_PER_THREAD*teams. Preserve and name this window; do not attribute a window change to width or occupancy. Each parallel worker creates Scratch once and reuses it across jobs. decode_block executes MTF, inversion, then RLE1 and CRC. Segmented inversion executes build then walk_segments_room; the latter includes segment setup, traversal and assembly (or a short-block fallback).

Consequences:

1. A marker around walk_segments_room measures that full envelope, not pure dependent loads. Calling the split wrapper to exclude assembly changes the measured boundary and must retain assembly elsewhere in accounting.
2. Four timestamps stop before RLE1/CRC and omit post-walk work, output publication and queue waits. They cannot report all running workers. Wall intervals include descheduling; an active phase is not proof that a CPU is executing it.
3. build-start through walk-end is an **inversion-phase interval**, not allocated-table lifetime. Scratch/table capacity persists into RLE, waiting and the next block. Allocation is also not cache residency: stale idle pages do not prove current LLC occupation.
4. Therefore label the minimal result *build-active and walk-envelope overlap*. To test a held-allocation model, add capacity/allocation epochs and first/last-use semantics, not just clocks. To include remaining worker activity add decode-end (one more clock/block: +25% clock reads) and job boundaries as needed. To isolate assembly requires its own edge or an explicitly broader walk label.
5. MTF slowdown against overlapping wall intervals on -2 cannot distinguish execution-core competition from memory bandwidth, frequency, scheduling or other buffers. Low nominal table size does not remove those confounders.

Instrument the exact question first. Do not rename a four-edge trace into measurements it never made.

## 2. Identification and power

For block i in round r define y_ir = walk-envelope duration / BWT rows, and zB_ir,zW_ir as time-averaged overlap of other build and walk envelopes during its walk. This removes a block's fixed content difficulty only if block identity and source input are fixed. It does **not** make z exogenous.

Counterexample: target walk starts at 0; an unrelated neighbor is active on [1,3]. The target finishes at T=2 or T=3 because of an independent pause. Its measured mean overlap is (T-1)/T: .5 or .667. Its time/row rises with overlap although the neighbor has **zero causal effect**. The response determines the integration window. Block fixed effects leave this counterexample intact. Common frequency/scheduling shocks and interactions between every worker's duration add further simultaneous causation.

Also zB+zW is constrained by worker count and other phase occupancy. Similar phase ratios across rounds can make their columns nearly dependent. At fixed W, retained allocation may be almost constant: then a held-capacity coefficient has no within-block support at all. Across-W contrasts reintroduce machine scheduling, core use and supply differences.

Thus:
- flat association does not kill table contention when conditional overlap variation is tiny, range misses a threshold, noise dominates, or opposing confounding cancels it;
- walking-only association does not prove walking-only causation;
- both coefficients positive do not prove a held-table law;
- no observational sample size alone makes these three causal explanations identifiable.

The useful output is **support and predictive association**. Publish per cell: within-block residual overlap ranges/SDs, partial covariance and design rank/condition, scatter/binned response with counts, round effects, uncertainty, and how much of time is in each measured envelope. Do not extrapolate X(n) outside its supported overlap region. A time-average n is additionally insufficient for a nonlinear law: generally mean X(n(t)) differs from X(mean n(t)). Retain the time spent at each discrete overlap count/pair; equal means can hide bursts and threshold crossings.

Sixty rounds are 60 repeated decodes, not 60*421 or 60*1831 independent machine experiments. Blocks within each decode interact. Use decode/round clusters and preserve chronological drift; block fixed effects are nested within input. Do not match block identities between recompressions. Use held-out chronological rounds for descriptive prediction, not random block splits from the same decode.

There is **no honest numeric 60-round power claim** without the overlap design and residual covariance. In a simplified one-regressor design:
SE(beta) approximately sigma_residual / (sqrt(N_eff)*SD(z_residualized)).
An illustrative 5% two-sided, 80% target would need effect approximately 2.80*SE before small-sample, clustering and multiple-model corrections. This is a sensitivity formula, not a registered test. N_eff is limited by independent round information, potentially below 60. With build/walk collinearity, residualized SD approaches zero and required effect diverges. Report inconclusive when support or precision fails. A 60-round trace can estimate descriptive slopes if support permits; it cannot promise causal separation.

## 3. One bounded recipe, not census then width grid

Recommended design for Coordinator registration; no execution authorization here:

1. Untimed marker/data-integrity and assembly review first. Pin exact boundaries, fields, width addressing, source/build identities, windows, correctness checks and permitted markers. An empty/partial log is a failed measurement. No full 60-round slot before the estimand is coherent.
2. Use the same -2 file, explicit addressed 4-byte and 16-byte record variants at W4 and W10. Padding never addressed by accesses is not this intervention. W4/10 supply endpoint contrast at small cost; do not add W6/8 by inertia.
3. Cross each of those four cells with marker **on/off**, in balanced round order: **8 cells**. Preserve two named sentinels, if the prior harness uses them. At illustrative 20 retained rounds plus one warm round: **210 slots**, versus the previous uninstrumented **126**. This +84-slot price buys width-dependent perturbation checks, not statistical power guaranteed by 20.
4. If an original -9 occupancy reference is essential to the information objective, add only its W4/10 marker-on cells initially: **252 slots** with the same round convention. These two references do not estimate -9 instrumentation bias; a matched off contrast would cost further slots. Do not call the 210-slot -2-only design a replacement causal explanation of all -9 behavior.
5. Freeze primary responses: paired whole-decoder CPU and wall, width effect at each W and width-by-W interaction; keep marker-off estimates primary, on/off interaction a contamination diagnostic. Pre-register a practically relevant information threshold and uncertainty rule using historical variability; the numbers here are slot arithmetic, not a ready workstation entry.
6. Phase envelopes are secondary. Analyze the trace's support before fitting predictive models. Do not adjust the primary width effect for overlap: width can itself change overlap, so it is a post-treatment mediator.
7. Width assignment tests an **addressed layout intervention**, not cache bytes alone: cache-line density, TLB/page footprint, build writes, prefetch distance, address arithmetic, allocation geometry and code generation may change. Count/include build+walk, segmentation, assembly and retained buffers. Inspect listings for widened-row multiply/shift and spill changes. Same lanes and work do not automatically mean equal instructions.
8. If on/off changes the width contrast materially, preserve the clean effect and reject the trace for causal attribution; reduce instrumentation or revise prospectively. Do not select whichever instrumented result agrees with the old law. A null endpoint contrast with a broad interval remains inconclusive.

Even a cheaper census-only fallback is useful for finding whether overlap varies enough to fit anything; its information objective must say that, not promise model discrimination. The intervention supplies more decisive information per added workstation slot. It still does not replace a complete candidate comparison. Keep the already implemented team-port route ahead of speculative layout+finder engineering where its holds permit.

## 4. Small table and parallel scan: joint economics, separate effects

The cited new f=.78 proxy is S = 1862/[1+.78*(g-1)]:

| g | proxy MB/s |
|---:|---:|
| 1.18 | 1632.76 |
| 1.44 | 1386.24 |
| 1.70 | 1204.40 |

Against 1499, its own break-even is g < **1.310463**. Hence “none beats default” is arithmetically false for g=1.18. This corrects the register's new assertion; it does not restore the withdrawn load law. f, candidate g, feed equivalence and retained footprint are unvalidated. 1862 comes from another block stream/worker regime and is neither a rigorous candidate ceiling nor a transferable measurement. At g=1.18 the proxy offers only ~8.9% headroom, which omitted cost can consume.

Parallel scan is worth pricing **jointly** because removing one bottleneck can expose another; it is not established as a mandatory component of every smaller table. For a viable named representation, use a prospective 2x2:
ordinary/finder supply × ordinary/small table, same compressed file and worker/core budget, plus the production default reference. Differences on elapsed-time or resource-cost scale reveal supply effect, representation effect and their interaction; compare the complete pair directly with default. Do not add throughput gains.

Charge scanner/finder search, boundary validation, duplicate parse work where present, Huffman placement, extra workers/cores, synchronization, job queues/windows, retained symbols, table constructor, all temporaries, peak allocations, RLE/CRC, consumer and tail. Parallel supply consumes cores and bandwidth formerly available to walks. Idealized throughput min(feed,worker-service,output) is only a planning model: these capacities themselves change under shared-resource contention.

Before any joint build: require each representation's full construction/metadata/fallback price. D4F2 still fails its construction filter; chunked full fill remains unpriced; compact30 g is not measured native build+walk. “One proposal” must not conceal a second unvalidated component or bypass D085/B5/C5.

## Disposition and progress

Expected: cheap census separates three mechanisms. Found: timestamp count understated on -2, boundaries omit held lifetime and residual worker work, overlap is outcome-dependent, and 60-round causal power is not identified. Closed: exact nominal costs and slot counts, source phase boundaries, a concrete endogeneity counterexample, and the arithmetic error behind the compulsory-scan claim. Unclosed: actual overhead/code-placement effect, overlap support, causal contention share, native candidate g and joint feed economics.

Next value: bounded width intervention with trace controls supplies a reproducible layout contrast plus descriptive occupancy, at 210 illustrative slots rather than a standalone 610-slot census followed by a width run. No claimed speedup, safety pass, restriction release or workstation execution. State and publication follow the pinned role spec.
