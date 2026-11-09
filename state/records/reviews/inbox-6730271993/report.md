# Making a pair consumer pay

Inbox [6730271993](https://github.com/lmmx/mibz2/pull/160#issuecomment-6730271993). Source #272 at **280643cef83b09ac89707b747670e8198b21b68e**. Operating specs **a4f97714cc14366984015869d9d094d89488ac57**. Source audit and prospective constructions only: no implementation, compilation, test execution, timing, dispatch, safety certification or production qualification.

## Decision

**Within the pair line: shared-offset staged rooms first (C37), then a flag-compatible symbol directory (C38).** The pair consumer already has 32 lanes, flags and next-row prefetch. Re-adding those changes nothing. Its removable work is the per-lane output-pointer/capacity machinery and the ordinary symbol-lookup dependency. Price the combination against modern production, retaining the unchanged pair consumer as attribution control.

**Across the decoder: C29's matched MTF-list control and C31's helper-side one-hop construction remain better engineering bets.** They preserve the current consumer and avoid paying a pair-build premium. Pair rooms are worth a bounded diagnostic, not a new primary direction before their common path is competitive. D045 now holds further service-core timing/admission; resolving its evidence is still valuable, not permission to rerun it. D042 and D044 retain their separate scopes.

The brief's Q1 INCONCLUSIVE/Q2 NEGATIVE STOP are stated expectations at this pin, not received measurements. The journal's registration is still DRAFT and explicitly says no CI timing has run. Preserve any later registered outcome; this review supplies no advance verdict.

## 1. What actually executes

[Pair source](https://github.com/lmmx/mibz2/blob/280643cef83b09ac89707b747670e8198b21b68e/src/pair_runs.rs), functions first_byte, setup and walk; [production source](https://github.com/lmmx/mibz2/blob/280643cef83b09ac89707b747670e8198b21b68e/src/bwt.rs), walk_segments_room_with.

A current entry is D[r]=L[r] | (Psi²(r)<<8), starting at Psi(orig_ptr). It emits L[r], then F(r)=L[Psi(r)]. This differs from the Researcher's F-coordinate proposal. Keep that distinction through tags, starting row and segment assembly.

Ordinary pair work includes:
- the table load and START test;
- successor extraction and L1 prefetch;
- first[r>>fshift], then **f[c+1]**, compare and possible correction;
- one unaligned u16 output store;
- per-lane row/pointer updates and a two-byte capacity check.

The reported .002 correction iterations/step describes the loop body, **not** its initial ends load. Even with zero corrections the source has two symbol-recovery loads, with the second dependent on the first. Removing only repeated corrections is a tiny prize; removing the common ends access is a different prize. At an ideal ordinary pair rate it removes approximately 0.5 such loads per output byte, before exceptions. This is source work, not a claim about emitted instructions or cache misses.

Both lookup and pair-table addresses use the already-known r. A compiler can overlap their accesses; the symbol path is not automatically on the next-link recurrence. The benefit may be load/issue pressure or L1 occupancy instead of exposed navigation latency. The 28,125-byte directory plus 1,028-byte ends and an 8,192-byte staged tile exceed 32 KiB before row state; the proposed directory reduces that pressure, but does not prove any actual miss.

### Two corrections to the journal's model

1. “One load and one compare” omits the dependent ends load above.
2. 12.4 minus 6.2 Ir/row does not imply a build difference of one cycle requires IPC above six. Different memory behavior, overlap and IPC apply to the *whole* alternatives: delta b=I_R/IPC_R−I_P/IPC_P, not delta I divided by a common observed IPC. Illustrative algebra: 12.4/4−6.2/2=0 cycles despite neither IPC exceeding six. This is a counterexample to the inference, not a performance estimate.

Thus the counts establish neither MET nor NOT MET for the archive's plus-one-c/B reopening condition.

## 2. Ranked constructions and prices

Let w_R be the pinned pair walk cost per BWT output byte, w_1 the matched production walk; b_R,b_1 include setup/marking; a_R,a_1 assembly. All must be measured on the same host, corpus, thread/placement plan and state. Define s_room as removed common lane work minus new staging/flush cost, s_tag as removed symbol-path cost minus extraction/masking/exception cost. Neither is measured here.

| Rank / candidate | Mechanism and expected c/B | Principal risk | Cheapest prospective CI falsifier |
|---|---|---|---|
| 1 / C37 staged pair rooms | w_R−s_room; expect a reduction from the old pair loop, not automatically below w_1. Compact rows, shared byte offset, capacity decisions per room; 128 pair rounds in a 256-byte tile. | Extra tile copy, spills, odd handoffs and depleted lanes erase saved control. | First archive ordinary/flagged native paths and untimed dynamic counts: row state, pointer updates, capacity tests, spills, flush bytes. Reject this formulation if common work does not fall after flush amortization. Then registered paired current-pair/room/production inversion comparison. |
| 2 / C38 two-bit symbol tags; three-bit variant only with marker redesign | w_R−s_room−s_tag. Expect common ends load removal; no justified absolute cycle saving yet. The .002 correction count does not bound s_tag. | Mask/extract/OR and exceptions offset the removed load; tags collide with flags if transplanted literally. | Exhaustive byte/marker/odd-cycle differential checks, then listing plus actual segmented exception/visit counts. Reject if common dependency/issue work fails to improve or exact net counts make the target implausible. |
| 3 / same 32 lanes, two bytes/load | Not an independent code change: it is already the pair recurrence. In a navigation-latency-dominated regime, similar latency/occupancy permits about 0.5w_1 navigation cost; an issue-bound pair step costing twice a one-hop step gives about w_1. | 32 lanes do not become 64 independent misses. Shorter round can expose prefetch latency; frontend/store/lookup limits remain. | Matched solo and concurrent-copy contrast on the existing CI mechanism; report effective demand/miss/occupancy and loop work, not random-access count alone. No blanket timing authorization. |
| 4 / C39 full two-byte payload in a wider word | Remove symbol directory entirely. Conditional same-latency walk target near half the navigation term; net expectation weak at current table-resident team point. | 20-bit link+16 output bits+two flags needs 38 bits. Eight-byte entries double table footprint; five-byte entries add address/decode/straddle work. | Static layout/traffic and ownership accounting first. If pursued, fixed-plan cache-pressure/whole-entry control; reject if larger-table penalty plus build exceeds recovered lookup/walk cost. |
| 5 / C40 pair/one-hop hybrid | For a fraction f of output served by pairs, ideal traversal cost (1−f)w_1+f w_2; add switching, representation and setup h. No gain unless f(w_1−w_2)>h plus build premium. | Long row cells are not long trajectories; mixed link meanings/byte recovery add work. | Untimed actual-trajectory census with markers/output limits; row length or r/n alone cannot admit it. Reject if the attainable weighted savings cannot pay measured common-path overhead. |

These are conditional prices, not invented PMU c/B. The local P 6–9/R 8–11 figures include inversion setup/build/walk/assembly, not matched walk phases. Subtracting their endpoints to forecast a consumer would be invalid. None of these ranges constitutes a new gate or replaces #272's frozen rule once registered.

## 3. C37: port the absent work, keep pair semantics

Retain 32 independent row chains, existing pair load, START test and next-pair prefetch. Stage pairs into a byte tile and advance one shared offset by two. A 32×256-byte tile remains 8 KiB, with 128 pair rounds; 256 *pair* rounds would silently double it. A conservative room is bounded by floor(spare_bytes/2) for every active lane and the tile remainder. Reserve/grow before the room, then flush written prefixes and advance each output pointer once.

A marker may stop a segment before emitting either byte (start) or after emitting one (LF-of-start). Flush the preceding even prefix, handle the exceptional byte and replacement segment in the slow path, and restart/recompute a valid room. Do not pretend the odd handoff occupies a normal two-byte slot. Retain start filtering, take's intentional bypass of its own marker, final n-byte truncation, one-cycle and even/odd periodic orbits. Tail compaction and segment distribution affect effective lane count.

The existing pair prefetch already requests Psi²(r) after loading D[r]. More lead distance requires knowing a later successor, hence another dependent access or retained lookahead state. It is not free. Keep current prefetch for the first control; isolate hint/distance only after room/tag costs are understood.

## 4. C38: a workable tag, with its bill

**Bit 30 is PAIR_LF; bit 31 is START. Only bits 28–29 are immediately spare.** A literal three-bit tag corrupts marker kind and can make q>>8 an out-of-range link. Existing PAIR_LINK removes bits30/31 but would retain tags28/29.

A first safe *design*, still requiring B5 evidence:
- store F(r)&3 in bits28–29, preserving L[r] and the 20-bit successor;
- one byte H[b] for each 256-row block: F(256b)&252, with bit0 an exception flag;
- unflagged symbol: (H[r>>8]&252) | ((D[r]>>28)&3);
- successor always (D[r]>>8)&0xfffff, in ordinary, take and tail paths;
- flag a block when first/last F symbols differ in their high six bits; use exact bucket lookup on exceptions, including empty symbol buckets and the partial last block.

Sorted F has at most63 high-group transitions, so at most63 exceptional blocks: at most16,128 distinct rows, 1.792% of900k, and H occupies3,516 bytes. These are conservative *static* bounds, not segmented execution rates. Periodic orbits and repeated visits invalidate conversion to dynamic percentage. The existing three-bit census (.343% one-chain Wikidata visits) does not measure this two-bit scheme.

Three bits can be recovered by using START as the union marker and classifying kind only in the flagged slow path through existing start/LF metadata. Fixed points require care: lfs includes self-LF rows although setup deliberately leaves PAIR_LF clear there. Classification must reproduce the original retained-start/actual-LF-flag distinction, not “present in lfs means emit one.” The benefit is at most31 exceptional blocks (7,936 rows); cost is extra slow-path classification plus a changed marker proof. Compare measured visit-weighted exception cost before preferring it over two bits.

F(r)'s tag is constant across each F run interval; fill-time tag work can be amortized there even though the current low byte L[r] is not constant on the pair-intersection cell. Directory construction and all tag masks/restoration remain charged. The Researcher's three-bit derivation is mathematically valid under its one-marker assumption; that assumption is not the pinned consumer.

## 5. Latency/MLP and whole-entry break-even

A useful diagnostic roof for each ordinary step is

    C_i >= max(U_i / throughput, lambda_i / M_i, traffic_i / bandwidth)

where U_i is relevant issue/resource demand, lambda_i exposed navigation service latency under that plan, and M_i effective overlapping navigation misses (bounded by active lanes and hardware resources). Traffic is actual transfers, not four bytes times random lookups. For one-hop w_1≈C_1+tails; for pairs w_2≈C_2/2+tails/markers/flush. Auxiliary symbol accesses contribute resource demand and may reduce M_2.

If lambda and M are unchanged and navigation dominates, pairs halve that term. If the pair step doubles U and issue dominates, per-byte benefit disappears. Fewer steps can also reduce shared fill-buffer/bandwidth contention: lambda_2 and M_2 need not equal their baseline values. Conversely a short loop can reduce one-round prefetch lead, or tails reduce M_2. These are causal hypotheses to distinguish, not architecture-wide predictions.

The archive supports pair benefit most plausibly with many independent live tables beyond the cache knee. It does not establish that current three cooperative teams benefit; those teams deliberately avoid that regime. CI's later walk census reversed the old universal “instruction-bound” explanation, while retaining the actual negative pair results. Do not revive that obsolete explanation as a proof of impossibility.

Whole-entry:

    b_R + w_2 + a_R < b_1 + w_1 + a_1
    w_2 < w_1 − (b_R−b_1) − (a_R−a_1).

A numerical sensitivity, **not a forecast**: b_1=2,w_1=6,b_R=3,a_1=a_R=0 c/B requires w_2<5 to win; a separately chosen10% time-reduction target requires w_2<=4.2 (8.4 cycles/pair step). Thus “two bytes/load” can win while still far from a 2× walk, but equal per-byte walk cost cannot repay a positive build premium.

At whole-decoder scope charge MTF/run extraction, admission scan, helper setup/synchronization, RLE/CRC and the critical worker. If inversion occupies fraction phi of serial wall, an entry ratio x gives only 1−phi(1−x) total-time ratio under unchanged nonoverlapped phases. In teams, completion is a max/barrier schedule, not summed work; faster noncritical helpers may buy no wall reduction. #272's inverse entry times RLE/CRC outside the interval and is not that whole-decoder test.

## 6. Why the hybrid and lookup-free alternatives rank lower

A long L/F row interval with Psi(q)=q+d does not show that Psi(q) remains in that interval. Even when it does, useful arithmetic steps are bounded by the distance to the interval edge, markers and remaining output. Count *consecutive trajectory* coverage. A Psi²-only word cannot supply Psi for a one-hop fallback. Dual full tables cost8n, while one mixed table needs a mode, byte semantics and proof that either successor lands on a valid next entry. All can consume the spare bits just allocated to symbols.

Keeping a current F-bucket interval per lane can skip lookup only while next rows remain in that interval. It adds bound tests/state and exact failure lookup; no current census prices this locality. It removes no mandatory work without that condition.

A wide word is the clean lookup-free control: pack both bytes plus link. Logical8-byte loads per pair equal4 bytes per output byte, yet the resident table doubles from3.6 to7.2MB at900k; physical line traffic is not determined by that logical equality. Three such tables are21.6MB before output and metadata, already beyond the archive's reported20MiB L3. This explains its lower priority at the current team point, without claiming it cannot win elsewhere.

## 7. Archive boundary and next decision

[Archive check](https://github.com/lmmx/mibz2/blob/280643cef83b09ac89707b747670e8198b21b68e/docs/journal/2026-11-07-left-field-archive-check.md) allows pair reopening for a target plan keeping six or more independent tables live, **or a build no more than current one-pass plus1c/B**, followed by registered CI A/B. Instruction counts alone have not established that cycle condition; its “untimed (callgrind)” wording needs an explicit cycle model and separately labelled observed-cycle diagnostic, not Ir masquerading as cycles.

[#42's move-walk journal](https://github.com/lmmx/mibz2/blob/280643cef83b09ac89707b747670e8198b21b68e/docs/journal/2026-10-27-move-structure-walk.md) is a different result: 41 versus16 ordinary-step instructions; best move walk13.5–17.1c/B already exceeded then-current build+walk12.0. Even free construction could not win that measured criterion. It left four-worker shared-cache contention and slower-miss hosts open. Warm best-of-three local measurements limit transfer. Neither that historical12.0 nor its instruction count is today's pair target.

Expected: cheaper run construction alone remains unlikely to pay the old consumer. Learned: common symbol recovery and per-lane capacity control remain removable; three spare bits do not. Gap closed: **no measured cycle gap**, only identified work and exact representation bounds. Next value: one bounded room control, then two-bit tags only if its listing/visit census supports net gain. Do not authorize a broad sweep or reinterpret #272/#34 verdicts.

Routing audit on this run found only five explicitly resolved historical issues. The brief's reference to PROTOCOL §11 is outside the authenticated pin (ends at§10); the technical work is independently within SPEC.OPTIMIZER. No wider authority is imported. Publication will use the pinned route/check and exact readback; pinned verify hardcodes FROM coordinator, so no optimizer verify-pass is claimed.
