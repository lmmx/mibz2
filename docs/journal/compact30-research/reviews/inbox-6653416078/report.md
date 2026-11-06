# Compact30 after the fused-pass falsifier

Inbox [6653416078](https://github.com/lmmx/mibz2/pull/160#issuecomment-6653416078).
Baseline **3c4ed7939d107a7b7b2633ca913f2e640023f15a**. Spec **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**.
2026-11-05. Proposal and scoped evidence audit, not independent qualification.

## Decision

**The distinction survives, but “compact removes random lines” is too strong.** Compact30 reduces the main table's footprint and can reduce repeated random line fills. It does not remove the ordinary step's random table reference. Its extra reconstruction work and RAW accesses can consume the benefit.

#183 lowers confidence in “fewer total lines means faster four-thread decoding”. It does not directly falsify reducing exposed random-walk service demand. My prior for a production win is nevertheless **weak and conditional**, primarily because #155 already lost in production and today's plain walker is better. Eliminating the other candidates does not increase compact30's absolute probability of success.

**Name one build: a bounded staged-compact30 parity probe**, with the existing direct constructor and encoding, current 32-lane/256-offset staging, L1 prefetch, 512 segments and window 16. Include the equal-size layout-only control below. This is a retest of compact30 with a materially changed walker, not a new format or an instruction-saving constructor project. Do not reopen the old port unchanged. No positive whole-decoder percentage is established; verified credit remains zero.

## What the new evidence establishes

I read the three pinned entries and all four #183 run summaries at [the baseline](https://github.com/lmmx/mibz2/tree/3c4ed7939d107a7b7b2633ca913f2e640023f15a/docs/journal), rather than treating the inbox summary as a fresh measurement.

| Item | Disposition and limitation |
|---|---|
| #183 fusion | **REJECT under its registered four-thread rule.** PR runs: wikidata 4t −4.2% (−10.0,+1.8) and +1.5% (−3.9,+7.2). Neither resolves a gain; run 2's point estimate is slower. |
| #183 controls | runs 4t harm is unresolved after the second PR run. Fusion-alone has a resolved −1.5% (−2.4,−0.6) 4t gain but flags text 2t; attribution fails as written. The missing settling clause cannot be repaired retrospectively. |
| Fusion inference | My earlier falsifier closes this formulation for the stated objective. It does **not** establish exactly zero four-thread effect, or prove sequential traffic never binds. The fusion-alone result, changes in code, runner differences and interval widths preclude that stronger conclusion. |
| #185 split condvars | Accept closure before a rewrite: the local window-16 counts give little reason to expect a worthwhile gain. This is not a matched CI SMT experiment. |
| Co-resident 16/L2 | Accept “not reproduced”: all eight point estimates slower; seven resolved. The 59-cell exploration supplies no demonstrated better setting, not proof of a universal grid optimum. |
| Byte budget / self-scanning | Deprioritize as throughput builds. Scanner backpressure is evidence against scanner starvation as the dominant steady-state problem here; it is not a quantitative bound on all tails, workloads or memory benefits. |

#183's simulator reports 38,669 fewer misses for wikidata block 1, both in post-walk and whole decode: **−57.4% of post-walk, but −4.61% of the whole decoder**. The runs corpus has a much larger simulated whole-decoder reduction without a settled four-thread wall benefit. These are Callgrind simulated misses with a 1 MiB LL, not measured two-sibling L2 misses, queue occupancy or hardware prefetch traffic. The result tests an implementation and its registered wall criterion; it cannot calibrate “nanoseconds per removed line”.

Likewise, #185's 0.2–0.4% is an illustrative CPU-cost estimate using assumed wake costs, not a hard wall bound. A high fraction of wrong-predicate wakes can coexist with almost entirely necessary waiting. Ordered completion can make the reader wait even when some later blocks are finished. These observations support closing the proposed wake optimization without identifying the walk as the only remaining bottleneck.

## Recovering the prior before proposing a retest

- [The compact traversal result](https://github.com/lmmx/mibz2/blob/3c4ed7939d107a7b7b2633ca913f2e640023f15a/docs/site/blog/2026-11-02-revisiting-space-time-tradeoff-bwt-inversion.md), §§6–9: old four-physical-core traversal CPU ratio about 0.840 and completion ratio 0.856; one-core CPU ratio 1.148. It excluded the direct constructor and prepared the two tables differently. This remains VALID ADVANCE in that scope, not a forecast for two cores with SMT.
- Same record: FOR-14 halved storage but lost four-worker traversal, about 1.45×. Packed extraction and recovery costs matter.
- [Constructor history](https://github.com/lmmx/mibz2/blob/3c4ed7939d107a7b7b2633ca913f2e640023f15a/docs/site/blog/2026-11-04-compact30-constructor.md), §8: provisional fields are read and rewritten, RAW is canonicalized, and block-dependent build tails matter. Historical invalid captures remain invalid.
- [Production-port archive](https://github.com/lmmx/mibz2/blob/c493a45d3f24e9c07657b3183cd2ab59fd66f5b2/docs/journal/compact30-channel/evidence/port-b4786f66/README.md): block-1 build 5,429,062→4,369,204 Ir, walk 15,426,892→20,207,700, decode 25,751,794→29,530,459. Those scopes exclude the driver's 16 Ir. The portfolio records #155 as rejected, with CI wall +7.3% at four threads. Constructor savings did not buy decoder parity.
- The portfolio's P37/P38/P117 retain the cheaper-byte-recovery question and untested request-cost explanation. A high cache hit share is not a zero-cost request stream. No old 18.26- or 12.07-Ir-per-row ceiling is a current time budget.
- [MLP discussion](https://github.com/lmmx/mibz2/blob/3c4ed7939d107a7b7b2633ca913f2e640023f15a/docs/site/blog/2026-11-02-memory-level-parallelism.md), §§4–5: a lane plateau and near-constant instruction count do not identify the shared limiting resource.

The changed fact that can justify a bounded retest is **current staging and lane code**, not #183's rejection. The old compact source writes directly into per-lane segment buffers. Porting current staging removes that known confound with today's plain walker; it does not guarantee a win.

## Source and dependency audit

At the baseline, src/bwt.rs:288–351 constructs the plain table in one checked scatter, including the run16 path. The current room kernel (887 onward; hot step 1027–1043) reads a 32-bit entry, tests START, emits its byte into the tile, shifts the successor and prefetches it. TILE is 256. src/lib.rs:314–357 retains separate inverse BWT, RLE1 and CRC; the rejected fusion is not silently included in the proposed baseline.

At [compact source b4786f66366c9e404702c800cd360dc793b1a7a5](https://github.com/lmmx/mibz2/blob/b4786f66366c9e404702c800cd360dc793b1a7a5/src/c30.rs):
- lines 62–80: 30 u16 fields and one u32 header occupy one aligned 64-byte record; RawG is 128 bytes;
- 1008–1017 and 1224–1233: header and field are addressed from the current label independently, then successor = (header >> 8) + field;
- 1162–1191: START precedes PARK; RAW saves a payload index and prefetches it, then resumes on a later turn;
- 734–820 and 930–963: allocation, provisional fields, snapshots, finishing, last-record/PARK initialization and RAW permutation are real builder work.

The old [walker listing and dependency audit](https://github.com/lmmx/mibz2/blob/f1dafa31d88d7351a98afb005a84f23d5a2312bd/docs/journal/2026-11-02-compact30-walker.md), §5, confirms independently addressed field/header loads, followed by shift/add, with no global F directory. Two table loads on an ordinary step usually share one fill; they still consume load issue and reconstruction resources. RAW introduces a genuinely dependent second address and a non-emitting turn.

The staged spec_offset-P0.asm archived at today's pin confirms the #171 tile stores and prefetch pattern. It predates #181's changed start test and is **not** a current complete decoder listing. I have not manufactured current instruction counts from it. The new probe needs its own full build/walk/caller listings.

## Price in the resource that might bind

Let n be inverse-BWT rows, m final bytes, a=n/m, and e the actual emitted walk steps per row. Periodic inputs can have e<1 because assembly repeats a short cycle; count e instead of assuming every stored row is stepped.

Main storage for a full n=900,000 block is 3.6 MB plain versus 1.92 MB compact, plus compact PARK/RAW/metadata. Two sibling tables are **7.2 MB versus at least 3.84 MB**. Neither pair fits the supplied 2 MiB L2. Four active workers do not each own an independent full L2.

Distinct main-line density is 1/16 versus 1/30 per stored row. **Neither is a dynamic random miss probability.** If reuse arrives too late, both can miss on nearly every useful step. If the plain table's relevant working set already hits, smaller storage can save almost no below-L2 requests.

For orientation only, the previous independent-uniform toy model, assigning the entire cache to two main tables, gives:

| Shared L2 sensitivity | Plain miss probability | Compact main miss probability | Main misses saved / step |
|---|---:|---:|---:|
| 512 KiB | .9272 | .8635 | .0637 |
| 1 MiB | .8544 | .7269 | .1274 |
| 2 MiB | .7087 | .4539 | .2549 |

Multiply step rates by a·e for final-byte rates. These are illustrative full-block sensitivities, **not measurements or guaranteed bounds**, and are not updated hardware predictions after #183. RAW, other buffers, associativity, actual trace order, prefetching and sibling phase overlap all change them.

A useful accounting identity for below-L2 random fills per emitted step is:

**δq = q_plain_main − q_compact_main − ρ·q_RAW − δq_metadata.**

Here ρ is dynamic RAW arrivals per emitted step, and q_RAW counts payload fills conditional on such an arrival. Include useful and wasted prefetch fills without double-counting merged demand/prefetch requests. RAW storage fraction is not automatically ρ. The historical walker counted approximately .00713 RAW arrivals per emitted byte on its own corpus; that cannot substitute for the current corpus and staging trace.

Under the toy 512 KiB case, one added payload miss per RAW step at ρ=.0637 would consume the entire .0637 main-line saving before metadata or reconstruction costs. At historical ρ≈.00713, the simple subtraction would leave about .0566; still not a measured gain. On less compressible blocks ρ, chain bunching and the constructor can be much worse. Each payload operand is four bytes: a 128-byte RAW allocation does not mean two line fills on every access.

For approximately unchanged main-table L1 misses, a weighted service-demand model is:

**a·e·[(q_plain_main−q_compact_main)·(ℓ_below−ℓ_L2) − ρ·D_RAW − ΔD_metadata] request-cycles/output-byte.**

D_RAW is the incremental payload-request occupancy per RAW arrival, including its actual service levels; ΔD_metadata is incremental metadata occupancy per emitted step. These terms are charged once, not subtracted again from the already net line-count δq. Changed prefetch behavior needs its own accounting.

This is a service-demand diagnostic, not additive wall time. Sustainable concurrency, load issue, reconstruction dependence, bandwidth and cache/TLB behavior also matter. Header load and field load overlap; the added successor add cannot be removed by claiming one fill. Four-thread throughput is set by the limiting combined resource, so reducing a non-limiting term can have zero benefit. No runner-specific latency or effective-concurrency values are established here.

### Charge construction and the rest

The ideal output bytes are 4n plain versus 64·ceil(n/30)+128R+64 compact, R RAW groups. A cold write-allocate/writeback main-output model gives about 8 versus 4.267 bytes/row **before** compact's other work. The difference is an upper opportunity on that output term, not a measured net saving. Provisional-field reads/rewrites, snapshots, descriptor traffic, RAW conversion/permutation, allocation and high-water capacities all count.

Even storage parity is conditional: ignoring edges/auxiliary state, compact is smaller only when R/G<7/16. Useful performance parity is far stricter. Retained arenas affect RSS and cache state differently: untouched capacity is not an occupied cache line.

The real accounting target is **Δ(build + walk + rest)** for the production stream, including all interactions at four workers. Report per-block CPU/phase costs and ordered completion tails, but do not sum overlapping per-worker elapsed times to predict stream wall. Unchanged rest code can run slower because cache state and phase overlap changed. No positive lower bound on four-thread benefit can be supplied now.

## One bounded build and its controls

Proposed name: **staged compact30, memory parity at 3c4ed793**. Three arms, one registered comparison:

1. **P: pinned production.** Existing plain builder and current staged walker, separate post-walk passes, window 16.
2. **C: compact30 with the same staging policy.** Reuse the direct constructor and exact compact encoding from the rejected port, adapting the walker to the current 32-lane/256-offset tile and present handover policy. Preserve RAW parking and saved-start semantics rather than speculating across them. Match starts in logical coordinates, segment count, fallback thresholds, output destinations, scratch lifetime and prefetch intent; document the F-emission versus production L-emission rotation explicitly.
3. **L: same-byte, equal-size layout-only compact control.** Keep exactly C's offset/header values, RAW bytes, labels, algorithm and total record-arena size, but separate fields from headers inside that arena. With N=G+1 including PARK, fields occupy [0,60N), header words [60N,64N). Field (g,j) is at 60g+2j; header g is at 60N+4g. Total bytes remain **64N**, exactly C's size, and RAW storage is unchanged. This is a bijective relocation of the same payload bytes, not a new compression scheme or extra directory.

C−L tests **co-location/locality at equal bytes**: C's header shares its field's line; L's header stream may instead stay hot in its small header region. Either direction is possible. Addresses remain derivable from the current label, with no dependent directory lookup. Address arithmetic and generated code must be compared; this is not a magically pure hardware intervention. It also does **not** isolate footprint reduction from reconstruction in C−P. That remaining attribution limit must be reported.

L is diagnostic, not a production candidate to tune. If the simplest implementation repacks after construction, charge that pass, temporary storage and its cache effect in L's whole-decoder result. Prepared-table walk comparisons must state their preparation and cannot qualify C. Never compare an oracle-built/warm C with a directly built P and call it decoder parity. A padded arena alone is not a capacity control: unused gaps do not increase the number of live cache lines.

**Safety and assembly targets, not achieved claims:** preserve φ's inverse mapping, F/Ψ output equivalence, current starts/handovers, one emission per successful turn, RAW pending ownership, PARK-before-index ordering, marker restoration, partial groups, count validation and error reuse. Test forced RAW, clustered RAW, tiny/periodic/multicycle inputs, chunk-edge starts, growth and high expansion. For L, u16 fields are even-aligned, header base is 4-byte aligned, all accesses remain within the 64N-byte arena; the final PARK record is relocated too. Any SIMD finish/load that assumed contiguous 64-byte records must be adapted or replaced and re-audited—do not cast split storage to Rec.

The ordinary C step must retain independently addressed field/header loads with no added pointer directory, packed extraction or per-row allocation. Archive the complete compiled body, callers, spills, tails, RAW paths and constructor before interpretation. Count additional waiting turns and actual emitted steps. Passing these static targets earns no wall-time credit.

## Falsifier and bounded decision

The Coordinator can register this proposal through the existing process; it grants no new authority and does not replace a standing acceptance gate.

- **Mechanism:** trace both siblings with build-to-walk handoff, actual block order, staging writes and RAW accesses. Report demand/prefetch/RAW/metadata separately and per final byte. If only single-thread cache simulation is available, label it as such; it cannot establish actual SMT resource relief. Reject the random-line explanation if net random service demand does not fall, or if dependence/load-issue costs absorb the reduction.
- **Economic primary:** fresh paired **C versus P**, full production stream, four workers on two confirmed SMT cores, same pinned head/window/toolchain/harness. Include builder, walk and rest at this topology, not the existing plain-only phases helper. Predeclare the corpus and benefit/harm rules, number of runs and all settling rules before results. As a proposed bounded rule: at least two fresh paired runs, a resolved faster wikidata 4t interval in one and a faster point estimate in the other; every flagged control row gets the same predeclared settling treatment, including attribution arms. Cap the total in advance. Existing stricter gates still apply.
- **Attribution:** C versus L at equal bytes, plus phase/RAW data, determines whether co-location helps. A null C−L weakens that explanation, not necessarily all density effects. A win only over L cannot rescue a loss against P.
- **Stop:** if C−P fails the registered whole-decoder benefit rule, even with smaller storage or fewer simulated misses, close this staged formulation. If the walk wins but construction/rest/tails erase it, record that decomposition and decline further work on this build absent a separately justified construction. Do not use 1-thread gains to rescue the four-thread claim or keep sweeping layouts after a null.
- **Scope:** CPU, RSS and controls at 1/2/4 threads accompany the primary. Fresh correctness, Miri and mutation evidence is required for the changed unsafe paths. The Coordinator's MTF constructions are separate; no duplication is proposed.

No safety certification, raw benchmark replay, implementation, compilation or timing was performed in this review. Findings are based on pinned source, historical listings and archived summaries. My involvement in the earlier compact/fusion proposals is disclosed. A1–A6 and D013 are unchanged; C15 remains CONDITIONAL with zero new verified credit.
