# Layout is a live confound, not an identified 4 KiB cause

Brief [6734039567](https://github.com/lmmx/mibz2/pull/160#issuecomment-6734039567); journal reviewed at **adb16fb75cb7e25eb734aeebf4cc9e7bf28ff8c5**, blob **16548a78cc313b2d6800da8fe06b2db0d42fc376**. Governing specs: **a4f97714cc14366984015869d9d094d89488ac57**.

**Keep C29 first. Put corrected layout attribution ahead of C43 and include it in the crossed-host diagnostic. B remains a diagnostic baseline, not a production promotion.** D's registered STOP and all existing screens remain unchanged. The evidence supports investigating address geometry; it does not yet support “the penalty is harness layout,” “4K aliasing confirmed,” or deleting the measured fallback cost.

## Evidence and mechanism

The pinned journal usefully narrows the explanation: its local executable has one instance each of mtf_avx2 and bwt::build, shared by all arms; B performs no record construction on fallback-only files. The common phase closure still shows the random/records gap. A few dozen wrapper instructions per block are not a credible direct instruction-volume explanation for that gap.

Those observations have limits:
- The listing, stack addresses, allocation counts and Cachegrind runs come from a local rebuild/probe on an Intel machine, not the archived EPYC executable. Matching source/toolchain is helpful, but executable identity and runtime addresses on the target must be checked before transferring exact geometry. The text probe includes changed source text.
- Equal simulated misses do not establish equal hardware misses, TLB behaviour, physical placement or exposed stalls. Cachegrind's combined entry/phase/walk counts are not phase-specific PMU measurements.
- Rotation across 15 rounds argues against a simple execution-position effect; it does not exclude persistent buffer-specific cache history or placement. Contended copies are slower absolutely and can mask a latency difference. Their equality neither proves nor refutes aliasing, and SMT placement was not recorded.
- Text's entry/phase discrepancy requires a separate account. “Frame depth caused it” is still a hypothesis, not an identified mechanism.

### The 1 KiB complication

[Agner Fog's primary microarchitecture measurements, §24.16–24.17](https://www.agner.org/optimize/microarchitecture.pdf) report Zen 4 false dependencies at **multiples of 1024 bytes**. They separately describe ordinary forwarding and partial-overlap costs; those are not a 50-cycle false-alias-event measurement. [AMD's Zen 4 guide, publication 57647, §2.12](https://www.numberworld.org/blogs/2024_8_7_zen5_avx512_teardown/57647_zen4_sog.pdf) describes the load/store machinery but does not supply the claimed EPYC9V74 50-cycle alias constant. The PDF is AMD-authored, served by a mirror; its cover says revision1.01 while internal headers say1.00.

This materially weakens the journal's “only B overlaps” discriminator. A contiguous 1024-byte counts array covers every byte residue modulo1024. A and D being clear of the frame modulo4096 does **not** make them clear under the reported narrower dependency relation. Which counts slots meet which frame accesses still changes with relative offset and data distribution. Recompute actual access-pair exposure modulo1024 as well as4096; do not merely relabel the current census.

**About 50 nominal cycles/exposed literal is an effective fitted charge, not an event latency.** The denominator counts candidate literals, not actual overlapping in-flight store/load pairs. One deep literal can encounter multiple accesses; self/pair classes can overlap; temporal overlap and predictor behaviour are absent. The numerator is elapsed time scaled by nominal GHz, not retired or reference hardware cycles. Similar fits on random/records make the hypothesis worth testing, but the failed text prediction and contended contrast prevent selecting H1 over H2. Neither the magnitude nor its proximity to any cache latency identifies a mechanism.

## Correct the proposed control before pricing its results

Use a bounded diagnostic proposal, with no retrospective change to #277:

1. **One call site as well as one callee.** A shared inline-never fallback reached through different A/B/D wrappers still inherits their different caller stack depths. For an isolation arm, converge the dispatch onto a common outer fallback call site/frame, passing explicit buffers, and verify emitted call chains and runtime stack residues. Preserve selectors and dispatch in full-entry costs. Keep an unchanged-entry reference: this diagnostic adapter changes the production-shaped path and cannot replace its result.
2. **Separate allocation identity from offset.** Give all normal counts arrays a documented common offset, but implement the deliberately conflicting offset as a separate diagnostic factor, not an accidental property of one privileged buffer set. Each backing set must serve both offset states and every algorithm, with identical capacity/initialisation policy. Use guarded/preallocated arenas and record exact touched ranges/pages; avoid changing allocation identity whenever offset changes. Offset changes also change cache indexing, so this alone is still not unique proof of store/load aliasing.
3. **Cross order independently.** Balance arm, backing-set identity, offset and execution position in a frozen schedule. Do not reuse one rotation counter as the entire design. With three arms, position j, arm a=(j+r)%3 and set s=(a+r)%3 imply s=(2a-j)%3: only9 of27 arm/set/position combinations occur. Adding A′ requires a new four-arm schedule; modulo3 cannot handle it. A/A′ must cross the same sets/offsets. A same-set matched A/A′ comparison is also useful because an independent-set null alone mixes algorithm identity and layout.
4. **Use reciprocal controls.** A penalty following the “aliased set” fits BOTH address geometry and that set's physical placement. Require a reversible offset-associated effect within backing sets, reproduction across sets/processes, actual access geometry, and stable code/input identity. If offsets are selected empirically, record the entire predeclared scan; do not select a fast offset and present its subsequent ratio as an unbiased comparison. A modulo1KiB analysis is necessary before calling anything “clean.”
5. **Record the target, not only the local probe.** Archive executable digest, wrapper/kernel bytes and addresses, counts/frame/ll/tt addresses modulo1KiB/4KiB/2MiB, capacities, touched-page ranges, affinity and sibling topology, THP policy and mapping coverage. Record unavailable physical mapping/PMU evidence as unavailable. smaps AnonHugePages is mapping-level information, not proof about each suballocation or host physical colouring. Collect metadata outside the measured region.

A disappearing B penalty after changing several things at once supports the combined remedy, not a unique cause. Conversely, a penalty tied to B after set/offset crossing warrants another code-path/history investigation; it does not automatically prove per-process placement.

### Cost account

| Proposed item | Direct cost and limitation |
|---|---|
| Shared fallback boundary | O(fallback blocks) extra call/return/argument work, not O(literals). If its added cost is h per block, charge h times fallback-block count divided by output bytes. No cycle constant measured here; large blocks amortise it, small blocks need explicit coverage. Stack/code-layout consequences can dominate the call cost. |
| Heap counts | Allocate once per buffer set outside repeated brackets; retain required 1KiB zeroing per fallback. A standalone align4096 counts object occupies at least4096 bytes rather than1024: about3KiB padding per set,12KiB for four sets, plus allocator overhead. Whole Bufs size must be measured separately. |
| Aligned Bufs header | Establishes inline counts placement only with a verified field layout. It does NOT align the backing storage of embedded Vecs. ll/tt alignment needs separate allocation contracts; capacity growth must preserve them. |
| Rotation | Swap references/indices outside timing, without copying contents. A′ adds another sweep: roughly one-third more arm work than A/B/D before offset/process factors. Reset/pre-touch policy affects cache history even when untimed. |
| Offset/process crossing | More diagnostic observations and state, not a production decoder improvement. The number of processes, order, statistics, multiplicity and equivalence margin must be frozen under C6, not adopted from “three processes” or “±2%” in the journal. |

On the binding critical worker, heap setup/first-touch still costs time and memory even if a prebuilt-stage bracket excludes it. For worker count W, persistent counts padding scales with W; multi-worker memory bandwidth and placement may swamp a single-thread gain. Whole-decoder credit requires the existing target-machine/thread-grid evidence, not this harness improvement. C5 workstation execution remains Louis's action.

## How much of B's .9763 can be layout?

**The observed 2.373% lead has no demonstrated layout-free lower bound.** Layout could account for all of it, or hide a larger genuine benefit; the current experiment cannot partition accepted-path and fallback effects.

There is a useful, explicitly noncausal counterfactual. I rechecked the archived #277 artifact hash and replayed its 15 paired pinned rounds, replacing only B's fallback seconds with same-round A fallback seconds:

- observed geometric B/A: **.976269848**;
- equal-fallback counterfactual: **.970909695**;
- difference: **.005360153 of A total**, approximately0.536 percentage points;
- A fallback share: geometric mean **.254549257**; fallback B/A **1.021067446**.

This is arithmetic, not a corrected benchmark or interval. The observed fallback excess hurts B; removing only that excess would increase its apparent advantage from2.373% to2.909%. Thus the specific observed fallback excess cannot explain the positive lead. Layout elsewhere, including A's accepted path and B's accepted construction, remains unconstrained. Do not extrapolate synthetic28–37% penalties onto Wikidata or subtract a fitted50-cycle charge.

The replay uses the earlier committed raw [#277 evidence](https://github.com/lmmx/mibz2/tree/ec8217f3d6a84205c0a9a1b7f9e49a6fda7ad939/state/records/reviews/inbox-6733116723), artifact13651923399 SHA256 **00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94**. No new measurements.

## Ranking, host crossing and default hardening

1. **C29 remains first for production economics.** Existing matched front-work evidence and production room-consumer path outrank an unqualified2.37% prebuilt-stage lead. This inquiry does not establish a better whole-decoder direction.
2. **Layout diagnosis before construction redesign.** Extend the previous common-binary host/build proposal with a crossed layout factor, not one special “layout arm” attached to B. Carry unchanged and normalised entry paths; A/A′ and reciprocal set/offset controls run on each eligible host. Preserve input/binary identity and within-host pairing. One VM per family remains an environment comparison, not isolated CPU-family causality. Missing host strata stay missing.
3. **B is the stable reference for C43, not newly promoted.** C43 retains its structural proposal:4r index ordering instead of8r pairs, global ascending fill, and dependent source-record gathers. Its prior **G + incremental_fill + H < .318992 nominal c/output B**, or6.43878/run, is a provisional same-run allowance. It must be re-priced under matched layout; do not call it recovered performance. Reducing metadata may simply exchange streaming bytes for dependent loads.
4. Keep already frozen LS/direct-F transfer and historical STOPs intact. Do not spend further D implementation effort before the confound is resolved.

**Cheap hardening worth proposing as a future default:** preallocated persistent buffers, explicit counts layout, logged actual addresses/capacities, no hidden timed allocation, matched dispatch and initialisation, independent balanced set/order assignment, and an A/A′ sanity control. Heap counts is a reasonable stabilisation step. “Align every buffer to4096” is not a universal cure: it can create repeatable low-bit collisions between streams and does not fix cache colouring, THP, frame residue or Vec backing allocations. A fixed alignment needs deliberate, tested relative offsets and a diagnostic sensitivity check. Do not silently amend existing registrations or impose a new acceptance margin.

**OBSERVER:** replace “#277's fallback penalty is harness layout/4K aliasing” with “local same-kernel evidence narrows extra-work explanations; low-bit address geometry and process placement remain live hypotheses, with 1KiB dependency evidence requiring a revised census.” Keep B's Wikidata reading descriptive, not layout-corrected or CPU-causal.

Scope: pinned-source/proposal audit, primary-source research and archived arithmetic only. No implementation, compilation, timing, safety certification, merge, acceptance change or measured credit. Migration checks passed. Pinned routing audit reports five resolved historical findings and no unresolved misroutes. Newer protocol references do not widen the pinned authority.
