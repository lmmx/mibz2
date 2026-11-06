# PR #146: independent adversarial supplement

**Disposition: the safe scanner remains nonqualified.** The independently replayed saving is **56,195,817 own-body instructions**, not a verified whole-constructor interval. The useful new results are a precise spill budget, a narrower interpretation of the proposed U1/U2 mutation, complete phase proof obligations, and competing static hypotheses for reducing the remaining state-management cost. This is advisory research, not implementation permission or charter §6.2 acceptance.

## Pins and independence

- Reviewed checkpoint: [`7f74e9f30e31af33b0d66fec3fe89af1758307d9`](https://github.com/lmmx/mibz2/commit/7f74e9f30e31af33b0d66fec3fe89af1758307d9), PR #146.
- Publication base and current operating documents: `4d814ce56be1487b728ba0a75ada6eb57978b911`. ACTIVE charter blob `31ec5719ca24cd3204b34595401c68ffd31bb57f`; REVIEWER blob `2839405784ec4e63d2f4cc7bd406e5dff4dd1029`; PROTOCOL blob `01d5391874e756ff90bbd9a822913b201cd9dece`.
- Frozen v2 `8beffe28bec9b539517b442350d8a3fda62164c8`, source blob `168c2bac9ad0d8da4d9f82788f009d32f419608e`; frozen v3 `dbf5139e12ae60b5f5bff40059b88d65ddccea41`, blob `01f91419904a605b721dc1742ba8087fe79f74cf`. Both match their PR #146 blobs exactly.
- Source, complete archived release listings, raw structural records, correctness logs, Miri and mutations were inspected first. Separate evidence-only accounting and safety reconstructions preceded consultation of PR comments and Astra's canonical ledger. The checkpoint journal supplied its registered obligations and claims; its conclusions were checked against the raw material.
- The existing review #148 and accepted comments `6630265636` / `6630284404` already cover the unsupported interval, parser defect, missing raw memory profiles and +1,840 memcpy term. Those are corroborated here, **not submitted as new findings**. The later canonical-ledger snapshot consulted only to avoid duplication was `27473ae1533c4eb2ef31da462bc8a324fa0fc660`; it was not modified.
- The initial channel-branch documents at `85d9ec48e02d160aa29648665d9c5eea040987b2` were also read. The current constructor-branch documents above supersede them, including permission for exact frozen-pin recompilation. This review nevertheless uses archived assembly only.

All locally materialized evidence is checked by Git blob identity and SHA-256 in `input-manifest.json`. Its digest, reviewed commits and outgoing messages belong to this review's own cursor, separate from Astra's records.

## Gate assessment and what is already settled

| Gate | Assessment | Scope |
|---|---|---|
| 1: current safe scanner | PASS on inspected archived checkpoint evidence | Source partition proof, frozen-source identity, completed registered Miri classes and caught mutations. No fresh runtime test or universal safety proof is claimed. |
| 2: machine code | Recorded diagnostic partial miss preserved | Read-side improvement does not imply tile-side success; the full listings and refill detours are counted. |
| 3: qualification | REJECT qualification; claimed whole-constructor interval remains HOLD/withdrawn | Own-body 56,195,817 is reproducible. No bound on unresolved caller/callee effects is invented. |
| 4: economics | Outside scope | No cycles, speedup, benchmark, perf, i9 or completion-time conclusion. |

The source-only scanner change leaves the inherited destination emitter obligations in force. The [safety assessment](safety/SAFETY.md) explains the scope of that conclusion rather than claiming to reprove every frozen pointer operation.

The two observed inclusive savings, 56,203,667 and 56,205,651, are observations. The first is replayable from the archived instruction profile; the second is summary-only. The archived profile declares `positions: instr line` and `events: Ir`. It cannot reproduce missing raw `Dr`/`Dw` or the absent second profile. Equal allocation call counts do not establish equal allocator work; observed allocator variation is not a bound. These points remain the accepted F1/F2 disposition.

## A1 — locate register-allocation costs at their actual frequency

**Severity: MEDIUM for cost attribution; disposition: scoped result established.** A per-row spill penalty would be the wrong model for the final safe attempt. Its ordinary full-tile row loops still contain 12 instructions each. The costly changes occur when those loops detour through refill:

| Full-tile refill detour | v2 → scanner instructions | Observed executions | Added instructions |
|---|---:|---:|---:|
| First segment | 10 → 14 | 996,634 | 3,986,536 |
| Second segment | 9 → 12 | 302,049 | 906,147 |
| Total | | | **4,892,683** |

Exact scanner anchors are `0x41bae–0x41bf2` (listing lines 569–582) and `0x41c6e–0x41ca0` (612–623); same-binary v2 anchors are `0x3babe–0x3baec` (1067–1076) and `0x3bb4d–0x3bb77` (1103–1111). These regions include the call instruction and exclude the callee body. Ordinary row loops are `0x41b80–0x41baa` and `0x41c40–0x41c67`.

The first detour adds two stack stores and two reloads per event. Its **1,993,268 extra stores** explain about **99.886%** of the own-body write increase of 1,995,539. The second detour's added register moves are real instructions but do not add corresponding stack stores. This distinguishes register shuffling, stack traffic and per-row descriptor accesses.

An independent assembly-operand classification, weighted by archived `Ir`, obtains own-body explicit reads **−3,389,475** and explicit writes **+1,995,539** (scanner minus same-binary v2). The net differences equal those at direct `%rsp` operands; non-`%rsp` totals cancel. This is a scoped architectural-operand reconstruction, **not recovery of missing whole-constructor raw memory profiles**, cache traffic or DRAM traffic. Implicit call/return/push/pop traffic is reported separately; a memory RMW is one instruction with one read and one write. See [accounting details](accounting/ACCOUNTING.md) and machine-readable reconstruction for conventions and limits.

**Implication:** an unchecked source range can remove checks without removing these live values or their preservation costs. Conversely, a transformation may reduce stack work without changing the 12-instruction ordinary loop. Require a complete new listing and event-weighted recount before assigning either effect. No estimated spill saving is added to the achieved 56,195,817.

## A2 — U1/U2's negative control does not isolate both unsafe operations

**Severity: LOW coverage clarification; disposition: strengthen operation-specific evidence, no current-code safety failure.** The checkpoint [§7](https://github.com/lmmx/mibz2/blob/7f74e9f30e31af33b0d66fec3fe89af1758307d9/docs/journal/2026-11-04-compact30-scanner-prototype.md#7-the-escalation-82-the-unsafe-operations-that-would-be-needed) proposes forcing `n1=17`, with an expected out-of-bounds Miri report.

For `ll.len()=32, i=0`, U1's `[0,17)` remains a valid range of `ll`, although it crosses the intended 16-byte tile. If execution reaches U2, `[17,16)` is reversed, or `16-n1` underflows in the alternative length expression. That failure tests U2/range order, not U1's parent-slice bound. An emitter rejection may also stop before U2. A semantic tile-partition fault and an invalid slice are different failures. The original promise is for a combined mutant, so this does not refute that promise; it prevents overinterpreting the result as separate coverage of both bounds.

Separate falsifiable controls are described in [the safety report](safety/SAFETY.md): U1 on a final full tile with the first endpoint changed to `ll.len()+1`; U2 with `ll.len=32,i=16,p=16,n1=14`, preserving `[16,30)` and changing `[30,32)` to `[30,33)`. Require the diagnostic to identify the intended operation. These are recommendations, not edits to the registered experiment or permission to run it.

The positive U1/U2 argument is otherwise sound **conditionally on the exact formulation**: `rest.len()>=16` establishes `i+16<=end<=ll.len`; `0<=n1<=16` gives two ordered subranges of initialized immutable bytes; `n<=983010` rules out offset overflow on the recorded target. The helper also handles tails. An unchecked full-tile formula applied indiscriminately to a one-byte tail is invalid. The future implementation must exhibit the dominating full-tile contract and keep tail behavior justified.

## A3 — phase proof and compiler exposure are separate claims

**Severity: LOW proof clarification; disposition: current induction established, future unsafe operation unreviewed.** Current `advance(k)` subtracts 30 at most once. `p<30` alone is insufficient: `p=29,k=31` would leave `p'=30`. All current call sites satisfy `0<=k<=30`: full tiles use 16, tails use 1, and run fragments use at most the open portion's 30 slots. Bulk completion advances the source by `30*whole` while leaving `p` unchanged. Initialization and reuse restore `p=0`. Those facts complete the induction; the written proof must name the bulk case and fragment capacity premise.

U1/U2's source-range proof does not require introducing U3. `min(16,x)<=16` bounds the split even if a wrapping phase subtraction were considered; semantic phase correctness remains separately necessary. A future `assert_unchecked(p<30)` would create an additional UB obligation. Current Miri logs cover no such future intrinsic.

The existing listing contains eight distinct U1/U2 candidate sites, each executed `S=2,393,313` times, for **19,146,504 instructions of current-site exposure**. U3's two phase-test sites at `0x41b23/0x41b27` are disjoint and expose 4,786,626 more. Thus 23,933,130 is a valid sum of those ten *existing* sites; it is not a guaranteed future elimination or a net bound. Source re-expression may retain some work, move it, alter fall-through or change spills. The script enumerates every address and observed count.

The later coordinator record, report 259 at `4d814ce56be1487b728ba0a75ada6eb57978b911`, records a user-relayed single U1/U2 trial and explicitly excludes U3. This supplement neither broadens nor reissues that authority. Assessment of the future trial requires its own immutable checkpoint.

## Competing opportunities and overlap

The [static research note](phase/proposal.md) develops two hypotheses from the actual remaining listing: discard redundant split state across refill, and derive `p` from the encoded label's low five bits. It includes mathematical equivalence, safety constraints, adverse cases, conditional arithmetic and compiler targets. Neither is an implemented constructor or an achieved saving.

| Item | Accounting status | Relationship |
|---|---|---|
| Current safe scanner own-body saving | Achieved within named scope: 56,195,817 | Already present in Astra's ledger; this review adds no cumulative credit. |
| U1/U2/U3 existing-site exposure | Conditional diagnostic only | Replaces scanner-check work; not an additive whole-constructor saving. U3 excluded by recorded trial scope. |
| Split-state / phase-state hypotheses | Conditional; unresolved whole-function residual | May share future compiler effects with U1/U2, descriptor arrays or direct-refill work. Never sum their caps with isolated achieved totals. |

This reviewer helped develop the hypotheses and suggested negative controls. Any later formal audit of an adopted transformation or validation design must disclose that involvement and use a fresh evidence-only reviewer when independence is required. A fresh critique of this static package is archived separately; it is not a §6.2 review of a compiled implementation.

## Evidence, limitations and next trigger

The [replay instructions](README.md) run only Python parsers and finite mathematical models. No constructor source was edited, compiled or executed; no new capture, benchmark, perf or i9 work was run. The archived release listings are assessed under the recorded rustc 1.97.0 / LLVM 22.1.6 contract; their compilation was not independently reproduced. Miri's archived nightly is a separate toolchain.

The primary corpus denominator is 434 blocks and 207,006,515 rows. Multi-configuration correctness records are not additional unique corpus blocks. The safety replay checks 436,880 abstract scanner partitions and all 930 legal `(p,k)` advance pairs; archived targeted Miri totals 105 cases over all eight classes. These models support the written inductions and do not substitute for Rust UB testing.

Preserved outcomes: v1 STOP; v3 compiler STOP; both construction captures INVALID; traversal VALID ADVANCE. Nothing here predicts four-worker critical-path improvement from aggregate instruction counts.

After this supplement, **WAITING** means waiting for a new pinned scanner correction/U1–U2 checkpoint with an evidence digest, a rebuttal to these specific findings, or the completed-sequence §6.2 evidence index. A user nudge or actual delivered notification must resume this thread. No automatic GitHub wake-up or polling loop is assumed. The broader programme is not declared complete.
