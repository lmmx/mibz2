# After P87: next construction at e26e3d9c

Inbox: [6646251725](https://github.com/lmmx/mibz2/pull/160#issuecomment-6646251725).
Baseline: **e26e3d9cbac6bcd2106b7070c869dd91fcd7217d**, P87 merge #164.
Instructions: SPEC.md and SPEC.OPTIMIZER.md at **beb7df121fddf69938fdf8c972a8c7d964267ac6**.
Date: 2026-11-05.

## Decision

**Build true scalar tiled B next**, as a bounded proposal with an ordered fallback and an instruction-accounting checkpoint before any performance claim. Retain production's 32 chains and prefetch policy; change deferred flag checking and eight-byte commits, not the representation or the queue at the same time.

Order of the remaining work: **B; handoff diagnosis; gather C; compact-table parity; Precopy deferred**. Handoff is a diagnostic priority, not an instruction to rewrite concurrency. This is a risk-adjusted ordering, not a list of established gains. B's advantage over the new baseline could be small or negative once replay is charged. It is first because it attacks the largest measured decoder component, has a separable mechanism, and is the necessary control for C.

P87 is now baseline, not a future opportunity. None of its benefit can be credited to B or added again. This review previously recommended the P87 validation and shapes B: it is **not independent qualification** of either proposal.

## 1. What the P87 checkpoint establishes

Evidence: [validation at a995ce87](https://github.com/lmmx/mibz2/blob/a995ce87/docs/journal/2026-11-05-p87-production-validation.md), [original registration at d6401b55](https://github.com/lmmx/mibz2/blob/d6401b55/docs/journal/2026-11-05-p87-production-validation.md), and [merge/source at e26e3d9c](https://github.com/lmmx/mibz2/commit/e26e3d9cbac6bcd2106b7070c869dd91fcd7217d).

- **Instruction condition: PASS in the checked archive scope.** I parsed the named caller/callee arcs in all fourteen raw Callgrind files. The published table is inclusive `c30cg::measured`, which includes exactly 16 Ir outside `decode_block` in every arm/block. The difference between arms is unchanged by normalization.
- **Production benefit: supported by the recorded results.** Four-thread wall is −2.2% [−4.0,−0.5] in run 1 and −1.1% [−2.3,+0.2] in run 2. CPU intervals cross zero in both. RSS is 71→66 MB, flagged less in both. The original CPU-or-RSS rule and amended CPU-or-wall rule each pass on their own terms.
- **No-harm criterion: supported as registered, not a proof of no harm.** No wall row is flagged slower in either run. Several bounds remain wide: e.g. random w1 upper +17.7% and records w1 +9.3% in run 1. No third run is called for by the amended rule.
- **Registration provenance qualification:** the amendment came after the 00:16:10 Running comment, violating the literal “before first bench comment” cutoff. The write-up discloses it and reports no figures/artifacts available at amendment time. Evaluating the original rule separately avoids silently replacing that record. This note does not overturn the recorded merge disposition.
- **Correctness coverage is reported, not rerun here:** CI/Miri, sample differential and reuse/fallback tests, and matching bytes on all 21 bench rows. Source inspection supports the stated last-use design: build consumes `ll`, the walk uses T and segment buffers, then assembly overwrites `ll`; unsupported kernels retain the unchanged fallback. This is not a fresh complete safety certification.
- **Mechanism remains unresolved.** Destination reuse and allocation removal change together. The phase harness does not call `invert_in_place`; its RLE1 slowdowns of +2.6% and +3.9% are unexplained between-build sensitivity, not evidence that the changed path slows RLE1. The old equal-allocation diagnostic stays closed and INCONCLUSIVE.

The two new runners are Xeon Platinum 8573C and Xeon 6975P-C. Do not transfer the historical EPYC/Zen macro-op model or an i9 four-physical-core interpretation to them.

## 2. Current whole-decoder budget

[Machine-readable arithmetic and all profile blob IDs](arithmetic.json); [archive-only replay script](replay.py).
Raw profiles are under [e26e3d9c: production-validation/cg](https://github.com/lmmx/mibz2/tree/e26e3d9c/docs/journal/2026-11-05-p87-production-validation/cg).
The script reads existing files only; nothing was compiled, decoded, benchmarked or timed in this review.

| Block | Pre-P87 decode Ir | P87 decode Ir | Saving | P87 build | P87 room walk | P87 rest |
|---|---:|---:|---:|---:|---:|---:|
| 0 | 7,557 | 7,280 | 277 | 3,150 | short path | — |
| 1 | 20,184,740 | 20,184,340 | 400 | 5,429,062 | 9,859,825 | 4,895,453 |
| 41 | 19,480,494 | 19,480,094 | 400 | 5,280,730 | 9,676,664 | 4,522,700 |
| 97 | 7,926 | 7,656 | 270 | 3,318 | short path | — |
| 233 | 19,989,569 | 19,989,137 | 432 | 5,215,492 | 9,831,055 | 4,942,590 |
| 400 | 19,490,568 | 19,490,026 | 542 | 5,203,460 | 9,735,075 | 4,551,491 |
| 472 | 8,764 | 8,494 | 270 | 3,539 | short path | — |

Rest = decode − build − room walk, **including assembly**. Build and room-walk arcs are equal between arms on every full block. Assembly reductions are 492/479/511/640 Ir; remaining changes offset part of them. Do not book the assembly reduction and whole-decoder reduction together. A fresh Scratch includes first growth: these 400–542-Ir savings must not be multiplied by every steady-state block.

Block 1's instruction shares are walk 48.85%, build 26.90%, rest 24.25%. They are not elapsed-time shares. Even eliminating all 178,330 assembly Ir would remove only 0.88% of this total; a fusion proposal also induces new RLE ordering, run-boundary and caller costs. I do not rank an unpriced fusion above B.

The previous [whole-decoder search and prior record](https://github.com/lmmx/mibz2/blob/dc5e9501fb7aaadc1aaaf34ecf18374ee742c14d/docs/journal/compact30-research/reviews/inbox-6645643541/report.md) remains the starting point: it cites the portfolio, published MLP analysis, rejected compact port and Precopy outcomes. This update re-examines current bwt/parallel source, the archived unrolled walk listing and the new production profiles. It does not claim a newly generated e26e3d9c disassembly or a fresh per-instruction changed-site ledger.

## 3. First: true tiled B, with explicit replay economics

**Prior verdict.** B is unbuilt, not #157 and not a failed B experiment. The [B2 analysis at 9723f068](https://github.com/lmmx/mibz2/blob/9723f068/docs/journal/2026-10-30-b2-gathered-walk.md) used old A at 14.8 macro-ops/byte and estimated B at 8–9. Its −10–11% decoder-time projection is obsolete for this baseline. The earlier unroll's lack of detected four-worker gain is adverse precedent, not proof that this different construction cannot help.

**Assembly anchor.** In [b0f6e89d's room_segments listing](https://github.com/lmmx/mibz2/blob/b0f6e89d/docs/journal/compact30-channel/evidence/walk-75b13f00/listings/walk-room_segments.asm), 0x34ee9–0x34f0c is one normal nine-instruction step: row load, T load, test, branch, base load, byte store, shift, row store, prefetch. The turn controls at 0x34ed5–0x34ee3 add four instructions per eight lanes. Current [room_segments source](https://github.com/lmmx/mibz2/blob/e26e3d9c/src/bwt.rs) still writes each byte immediately and handles flags once per offset. Eight lanes unrolled is not eight successive bytes tiled.

**Proposed construction.** Speculate eight rounds across the 32 lanes into private byte tiles, ORing flags; follow only masked links. Commit an eight-byte lane tile only if the entire accepted domain is free of flags and has eight bytes of reserved capacity. Keep build, segmentation, prefetch, final assembly and P87's output destination unchanged.

For the first bounded version, use an **all-lane snapshot and ordered scalar fallback** on a flagged tile. Restore row state, discard all speculative bytes, and run the existing offset-then-lane-order path through that interval (or to its existing full-phase exit), consuming the encountered boundary before attempting another tile. No speculative output length, next link, queue claim or ownership change is committed. This is a deliberately simple correctness reference, not a claim that its replay cost is cheap.

**Why the ordering clause matters.** Suppose lane 0 first sees START at offset 7 and lane 1 at offset 2. Replaying failed lanes in lane-number order assigns the next queued segment differently from production. A changed schedule may still be mathematically valid, but the current ordered-handover proof and tail accounting no longer apply. A selective-replay refinement must prove its event ordering/equivalence explicitly, including queue exhaustion. A flag OR alone does not encode the first event. It is unsafe to reuse the old cost estimate without specifying this algorithm.

**Whole-decoder price.** Let E be useful normal rows actually replaced by accepted tiles, u the generated steady tile cost per such row including normal commit/loop work, Q discarded speculative row steps, and H all snapshot, restore, boundary, tail, dispatch, spill and changed-rest costs beyond the matching baseline. Then a *conditional instruction model* is:

`saving = E × (9.5 − u) − Q × u_spec − H`.

The 9.5 reference is the archived normal-step construction, not a substitute for a current dynamic listing count. A falsifiable initial assembly target is **u ≤ 8.5 retired instructions/useful row**, using matched counting boundaries. That would expose at most about 0.9M Ir (4.46% of block 1) before Q/H if every row were eligible; it is neither a lower bound nor a forecast. BMI1 link extraction, if used, needs a supported-feature contract; a mask plus shift, extra addressing or spills must be charged.

**Replay can consume that entire margin.** An all-32-lane, eight-round failed tile discards up to 256 steps, not eight. With at most 512 segment finishes and a fallback that consumes a new finish each time, Q ≤ 131,072 is a coarse bound; at 8.5 instructions/step that alone is 1,114,112 Ir, before H. Thus this construction has **no positive conservative net bound yet**. The old 1,758-byte mean gives an illustrative independent-boundary probability 1−exp(−256/1758)≈13.6%, not the old 3.6% eight-lane-group estimate; neither is measured current geometry. Do not use that probability as a gate.

**Memory/dependencies.** Successful tiles retain one random T read per emitted byte and the lane-state recurrence. They replace repeated output-base loads with commits but add private tile writes/reads and snapshots. Failed speculation adds random T reads; no fewer-miss claim follows from fewer branches or instructions.

**Falsifier and next-build deliverable.** The Coordinator's bounded prototype should first archive the current baseline and B listings, all seven whole-decoder totals, E/Q/tile-failure counters, exact output/CRC and tail/slow-path coverage. Reject this B formulation as an instruction reduction if the measured net is non-positive, or the prospective registered minimum is missed. Revisit the formulation rather than calling that a proof of a hardware miss cap. Only a separately authorised, registered production comparison can establish elapsed-time value; `ΔT/T = f_walk(r_walk−1)` requires a matching current time fraction, not 48.85% Ir.

Adversarial obligations: START at every offset, several flags with different/same offsets, short cycles, exactly eight versus seven spare bytes, growth/reallocation, queue exhaustion mid-interval, fallback into the draining tail, and in-place output equivalence. Mask before speculative T access; every tile byte read by a wide commit must have been initialized; no output beyond capacity. These are proposal proof obligations, not completed tests.

## 4. Second: diagnose handoff before replacing the queue

**Prior:** the recorded delay is ~11% of worker time (34.7% of Q2 idle), with a 980µs versus 62µs conditional contrast. Convoy is plausible, not proved. P87's positive engineering result supplies no new queue attribution.

**Current anchor:** [parallel.rs at e26e3d9c](https://github.com/lmmx/mibz2/blob/e26e3d9c/src/parallel.rs), `worker`, holds the receiver mutex during blocking `recv` and releases it before decode. Costs occur per job, not per T row.

**Whole-decoder delta:** a queue-only change has zero direct `decode_block` instruction saving. Whole-process work and wall time are unknown; all queue/dispatch/publication overhead must be counted. The historical 11/70≈15.7% useful-capacity illustration is not a current throughput forecast.

**Falsifier:** establish queued work, an available worker, lock owner and scheduling state together. If delays are mainly off-CPU, publishing or buffer work, the proposed claim bottleneck is refuted. If a queue change removes claim delay without total wall benefit, it has not solved the production limiter. Any implementation must preserve exactly-once jobs, cancellation/drain, ordering and bounded memory. No queue rewrite is recommended before that diagnostic.

## 5. Third: gather C, priced against true B

**Prior:** unbuilt; the old calibrated C−B benefit of 0.13–0.89 c/B is not a current prediction. Unlike rejected LF², C adds no whole-table random-read construction pass or doubled table. That distinction does not establish a gain.

**Whole-decoder price:** `I_C−I_B = walk_C−walk_B + changed_rest`; compare both to e26e3d9c and never add separate B and C credits. Count packing, masks, extracts, gather setup, spills, replay and draining tail. Each emitted symbol still needs one T element. C's group completion waits for its slowest element, and missing elements may serialize issue.

**Falsifier:** reject the extra gather complexity unless it meets a prospectively declared incremental benefit over B with complete-block/control costs. Historical C≤B−.05A and B/A≥.95 thresholds belong to their old A and topology; they cannot identify today's bottleneck. Xeon gather behaviour needs its own evidence. All active/inactive gather indices and feature-gated fallbacks require explicit safety coverage. Current whole-decoder gain: **unknown, zero verified credit**.

## 6. Fourth: compact table must first reach production parity

The [rejected #155 port at c493a45d](https://github.com/lmmx/mibz2/blob/c493a45d/docs/journal/compact30-channel/evidence/port-b4786f66/README.md) still has block-1 compact build 4,369,204, walk 20,207,700 and rest 4,953,555 Ir.

Holding that archived compact build/rest fixed, the new normalized budget is:

`walk_compact ≤ 20,184,340 − 4,369,204 − 4,953,555 = 10,861,581 Ir`

That is **12.0688122 Ir/row** at 899,971 rows, requiring **9,346,119 Ir (46.25%)** off its existing walk merely for whole-decoder parity. It replaces the prior conditional 10,861,938 budget; neither the old 18.26/row figure nor constructor savings alone suffice. If it reached current walk cost, its conditional whole saving would be 1,001,756 Ir (~4.96%); that is a target, not a construction.

**Prior verdict:** existing port REJECTED; extra offset/header reconstruction overwhelms smaller storage. P87 does not remove that dependency. A compact kernel's own in-place support could change its rest cost, so do not mechanically subtract P87's 400 Ir from this old arm.

**Falsifier:** failure to reach whole-decoder parity or the prospectively registered production benefit, including controls. A new representation must provide a new reconstruction path and exact permutation/header/RAW/PARK/reuse proofs plus generated code. Storage reduction alone is no basis to rebuild the rejected port.

## 7. Fifth: Precopy remains deferred

**Prior:** forward chains copied 31.5% but increased CI wall about 18%/6% at one/four threads; backward variants were worse. The register's reopening condition is 8–10 workstation workers, not another four-thread CI attempt. No new evidence meets it.

**Whole-decoder model:** copied steps × avoided current step cost, minus bookkeeping on all walked rows, chain construction and copy/setup costs. Actual copyable-chain frequencies are required; the offline 65.1% census is not a decoder saving. The post-unroll baseline makes the bookkeeping harder to repay.

**Falsifier/reopening:** only a materially different cost construction or the separately authorised recorded condition could change that verdict. Preserve overlap-safe copies and dependencies. Current expected benefit has a negative prior; no positive forecast or verified credit. Do not reopen it for this brief.

## Disposition

A7/C11 records the scoped P87 archive saving and reported engineering outcome; it is already included in e26e3d9c. C12–C16 remain CONDITIONAL or historically REJECTED, with zero new verified net credit. Historical frozen-constructor A1–A6, D013 and the closed P87 diagnostic are unchanged. The next useful response is pinned current-baseline B code/listing/count evidence with the replay domain made explicit, not another fit of the old B2 timing model.
