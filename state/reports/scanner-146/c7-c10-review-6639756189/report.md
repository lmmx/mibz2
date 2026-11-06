# C7–C10: independent §6.4 assessment

Trigger: [6639756189](https://github.com/lmmx/mibz2/pull/145#issuecomment-6639756189).
Proposal: [82bc69b5](https://github.com/lmmx/mibz2/blob/82bc69b5cdad78fe7e90243d76b4418b0b4bbef5/docs/journal/compact30-research/proposals/ranked-next-cuts-6639496192/report.md).
Disposition: **recommend corrected C7 for bounded registration; revise C8 before registering its claimed payoff. C9 remains conditional; C10 needs an expanded census and cost model. Zero achieved credit.**

## Evidence and independence

Read component source directly: v3 `dbf5139e12ae60b5f5bff40059b88d65ddccea41:examples/c30walk/ctor3.rs`, descriptor baseline `7620c0c667018772a0ba3be28eb62263b55037f8:examples/c30walk/ctor2.rs`, scanner U2 `ba261eff245f45e1c07fdfb05fa2fe6ff1ced835:examples/c30walk/ctor2.rs`; v3's archived leaf listing and journal at its pin; proposal and ledger at 82bc69b5. Policy/economic provenance: charter §6.4/§8.4 and reassessment journal at `2210e4457b383e4566df99eb881b4ca0401c4cd3`. ROLES at established 0ed278d7; MODE at e44d87f9.

Prior involvement: Researcher previously advocated conditional v3 inclusion, reviewed scanner/counter accounting and C6, and drafted campaign navigation. This is an informed independent assessment, not blind review. No candidate implemented, compiled or timed; no raw corpus/profile replay. Component source is not a substitute for the actual combined Assembly-1 listing. Attached audit.json records arithmetic/finite tag checks and a source-derived C8 counterexample.

## C7 — viable, with two semantic repairs and corrected frequency

The archived v3 journal distinguishes **1,647,375 leaf finishes** from **1,647,416 v2 fin_fast**, the latter including **41 same-epoch partial groups**. Source explicitly asserts fin_fast = leaf_fast + slow_ordinary_partial_same_epoch. Use N_leaf from the combined counted build; do not silently substitute fin_fast.

On the supplied portion count P = 1,795,482, corrected illustrative targets are:

- 6N_leaf − 2P − H7 = **6,293,286 − H7**;
- 6N_leaf − P − H7 = **8,088,768 − H7**.

The original targets overcredit 246 instructions. Small numerically, material as an event-map error.

A spare bit exists: ORD is bit 31, epoch occupies 24–27. Bit 30 can represent full *capacity*, provided all readers are audited. Invariant at open: ordinary, aligned group, end − start = 30, opening epoch recorded before the first append. At finish: the caller proves write == end. Capacity alone does not establish initialized fields; that caller invariant is essential. Refill calls close only at end; run closes after exhausting room; the final sweep first verifies every bucket rank equals its limit.

**Do not retain the old header expression `tag & !ORD`.** With FULL in bit 30 it corrupts the header. Emit only epoch bits, e.g. tag & 0x0f000000, before combining u0 and c. Likewise mask epoch extraction in the slow path and revise raw-tag equality diagnostics. Preserve RAW high-bit discrimination, unopened sentinel separation, old-epoch fallback and partial groups. The audit checks all 16 epochs × both capacity states: the old header mask contaminates every FULL tag.

The archived fullness subtree has seven instructions; folding it to the one necessary indexed w0 load can save six. This is a compiler target, not an observed combined saving. **w0 remains required** for the header and normalization. The proposal's claim to remove both pointer loads is wrong: the write load can disappear; w0 cannot simply disappear. Any additional memory reduction needs its own listing evidence.

H7 includes active_full_tag construction/stores per epoch, every ordinary open and new C8/3b path, modified slow guards, caller addressing, spills and header masking changes. Charge all new sites; do not count P as a measured opening census without reconciliation.

Recommendation: Coordinator may register the corrected design with an explicit Assembly-1 source/listing pin and **4 M incremental minimum**, below the corrected conditional estimate. With six removed and two/open, H7 must be ≤2,293,286 for that minimum. If the combined baseline changes N/P or leaf savings, revise the prospective payoff case before registration; never lower a registered minimum after a miss. Tests must cover full/partial same epoch, old epoch, forced/crossing RAW, malformed counts, early rejected blocks, reuse, and mutations of FULL/header mask/caller fullness. Existing exact, Miri, fuzz and accounting gates remain.

## C8 — natural mode is insufficient

Concrete counterexample from source: n=60, counts A=31/B=29, an A run of 31. Natural scatter premarks group 1 RAW because the bucket boundary is rank 31. complete emits group 0; residual open starts at rank 30 with one row left in A. It is natural mode and follows whole-group emission, yet its group is crossing/RAW. The original generic open deliberately consults the bitmap here.

Narrow the shortcut to a proven aligned, nonforced interior group: r mod 30 = 0 and r + 30 ≤ bucket_limit, with r within the current bucket, valid destination allocation and the run's dominating rank + m ≤ limit check. Use overflow-safe bounds, initialize every descriptor field and the C7 tag, preserve the generic path otherwise. These conditions exclude every internal bucket boundary; natural RAW premarking then cannot select that group. No reference to uninitialized Rec is needed.

The 496,004 count is **all run opens**, not demonstrated eligible interior opens. Let E8 be eligible and D8 the cost of testing dispatch across all considered opens:

**saving8 = (C_old − C_special) E8 − D8 − H8**.

33→16 gives 17E8, not automatically 8.43 M. Count eligible, bucket-end crossing, partial-final, unaligned and forced classes separately; partition run opens from scalar 3b refills. H8 includes new guard/fallback, retained pointer computation, tag work, descriptor stores and spills. C7 open cost must be charged exactly once in the combined ledger. The unknown E8/H8 prevent endorsement of the present net estimate or a numeric minimum. **C7+C8 are not jointly ready as written.** Register C8 only after the guarded formulation, eligibility census and positive prospective minimum are pinned; no need for user reapproval of a properly bounded §6.4 proposal.

## C9 — plausible amortization, missing tail distribution

Arithmetic 51,273,000 − 8·5,127,300 = 10,254,600 is correct conditional on an *all-events average* cap of eight. A four-record kernel alone does not prove that average.

For each complete call with length L, count Q=Σfloor(L/4), R=Σ(L mod 4), and call count K. Evaluate old_cost − c4·Q − ct·R − setup(K) − H9, with actual paths. Short runs may erase the unroll gain. Four 16-byte stores are a constraint of this proposed store formulation, not a universal mathematical lower bound on instructions.

Safety: four records contained in allocation and whole count, full initialization exactly once, nonoverlap, no reference to uninitialized memory, bounded header arithmetic within the source epoch, correct tail and unchanged forced fallback. Count register pressure from four header values and expanded callers. Unrolling may help low-bulk workers too; “bulk-only site” does not prove zero completion benefit.

## C10 — prefix histogram necessary, insufficient

Measure the joint distribution of prefix length 0–15, source phase, destination room/rank, and ordinary/RAW/forced class on the original U2 terminal tiles. Count before/after fill calls and rows, complete groups, opens, closes/finish classes, 3b eligibility, and remainder-tile advance paths. E_f alone cannot price those transformations.

The prefix's destination writes do not vanish: **fill and phase advancement cost work even with no added boundary** (v3 fill lines 744–756). Use

**saving10 = 10E_f − scanner_cost − Δfill/phase/remainder_cost − Δboundary_cost − H10**,

with signed changes relative to the same baseline and explicit overlap; assign each site once. The proposed boundary term is adequate only if it explicitly incorporates all these nonboundary changes elsewhere. 9,155,340 is a maximum, not a forecast; no basis exists for the Coordinator's approximately 35 M total. Even the reported C7+C8+C9 conditional sum is only 24,980,200 before H, and C8 eligibility remains unresolved.

Reuse the held terminal tile if possible. Preserve the original 16-byte grid: consume prefix through run, process exactly its remaining suffix, then advance to the original next tile. Otherwise changing tile alignment changes U2 event frequencies and invalidates the proposed incremental model. Source loads must stay within the epoch, not merely the allocation. Define zero/all-equal bit-scan behavior, label holes, end-of-input and rank overflow handling. Prefer safe loads; unsafe operations require the named §6.4 argument and targeted tests.

## Completion: correct direction, unsupported attribution

The archived capture explicitly remains **INVALID**. CPU 1.08619319 and completion 1.11436908 are descriptive v2/P ratios, not measurements of Assembly 1. Applying the whole old deficit to only new C7–C10 cuts ignores already changed work. Ratios suggest needed reductions of 1.767% CPU versus 8.468% completion from that old state, but establish no instruction-to-cycle conversion.

“33 cycles × 1.65 M finishes” is an allocated requirement under assumptions, not a measured 54 M-cycle fast-finish bottleneck. Source/assembly support eliminating a dependency chain; correlation between low bulk and slow workers does not distinguish finish latency, scalar pointer updates, cache misses or scheduling. A cross-worker aggregate finish count is not a critical-path event count.

The weighted skew factors are candidate 1.07331477 versus production 1.04670176. A crude constant-mean sensitivity would move completion to ~1.08674 if skew matched production, or ~1.03825 if candidate skew were ideal. These products mix aggregate definitions and omit start spread; they are neither a valid estimator nor a rigorous lower bound. They nevertheless warn against promising that skew-only repair closes 1.02.

Use archived per-step starts/ends and per-block event vectors to model
T'_s = max_j(start_sj + duration_sj − d_sj) − min_j(start_sj).
Recompute the maximizing worker after each hypothetical reduction. Vary effective savings per event; do not equate retired instructions with cycles. Whole-step improvement saturates when the runner-up becomes critical. Separate mean-worker work, imbalance and launch spread; thread CPU and wall duration are different observables.

**Next decision:** pursue corrected C7's direct serial-chain experiment; census guarded C8; price C10's full transition and scalar-escape opportunity. C5's already-proposed normalization refinement is another distinct finish mechanism, with its own minimum and overlap. If the corrected combined structural reductions cannot reach critical low-bulk workers, redirect toward scalar/fragment dependency or latency hiding rather than endless bulk-only count cuts. Do not introduce scheduling/harness changes under this assessment. No evidence yet proves a wholly different mechanism is necessary, and no evidence proves these cuts close either production gate. Continue bounded research; reject the unqualified economic forecast, not the entire line.
