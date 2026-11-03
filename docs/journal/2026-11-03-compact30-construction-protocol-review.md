# Compact30: the guardian's review of the handoff and the protocol draft (report 191)

Kept as relayed. It reviews the documents at `6a097d8`:
[the review handoff](2026-11-03-compact30-constructor-review-handoff.md) and
[draft 1 of the construction-cost protocol](2026-11-03-compact30-construction-protocol.md).

## Report 191

Both drafts are ready for the next review stage, but I would revise the construction-cost protocol before authorizing its measurement harness.

I inspected the documents at `6a097d8`. Claude has separated the two tasks correctly: an independent audit of the new constructor's correctness and memory safety, and a proposed experiment to determine whether its CPU cost fits the budget.

The reviewer handoff is particularly useful because it identifies potential blind spots rather than simply repeating successful test counts. It explicitly flags the uninitialized record arena, release-mode assertions, RAW permutation assumptions and Miri's incomplete coverage.

I approve sending that handoff to the researcher now. The researcher should inspect `ctor.rs` independently, not rely on the subagent's explanation.

For the cost protocol, the foundation is sound. I agree with using the production `bwt::build` interface, keeping the primary corpus, measuring both reused and fresh allocation, and retaining the conservative 1.06714× ceiling.

There are a few decisions worth settling before Claude writes another measurement harness.

### Decisions on Claude's eight open questions

| Question | Guardian decision | Rationale |
|---|---|---|
| 1. Should cold allocation gate? | No — report separately | Steady-state reuse is the primary investment case. Cold costs must remain visible. |
| 2. Use public `bwt::build`? | Yes | This is the production interface. Confirm that the measured corpus takes its one-pass path. |
| 3. Add the held-out corpus? | No | Keep one primary registered workload. The held-out corpus already contributes correctness evidence. |
| 4. Flush caches between builds? | No | Flushing introduces an artificial memory condition and another source of interference. |
| 5. Add a completion-time gate? | Yes — warm e4 ≤ 1.02 | CPU savings should not justify a material regression in completed-work time. |
| 6. Decide using full blocks only? | No | The full corpus remains primary. Report full-block and short-block results separately. |
| 7. Re-review any constructor change? | Yes | New source, assembly and correctness evidence would be required. |
| 8. Is the estimated input memory acceptable? | Conditionally | Confirm sufficient free RAM and no swapping or memory pressure before capture. |

#### Two protocol details I would strengthen

First, preserve the exact build boundary. The draft correctly includes cursor initialization, bitmap operations, allocation, the complete-record path, promotion, RAW canonicalization, padding and finalization. None of that should migrate into untimed preparation.

However, this benchmark will still have an important cache-state asymmetry: production and compact30 use different table allocations and their previous output checks warm those allocations differently. I would retain this natural distinction and document it clearly, rather than introducing a cache flush.

Second, distinguish construction-only throughput from end-to-end performance. The proposed `e4` measures synchronized construction-step completion. It is useful as a no-harm control, but it is not full-decoder throughput.

I recommend the following final decision table for the warm four-core regime:

| Outcome | Registered condition |
|---|---|
| PASS | Upper 95% CI of `b4` ≤ 1.06714 and upper 95% CI of warm `e4` ≤ 1.02 |
| FAIL | Lower CI of `b4` > 1.06714 or lower CI of warm `e4` > 1.02 |
| INCONCLUSIVE | Neither PASS nor FAIL; no automatic extra rounds |
| INVALID | Any schedule, correctness, identity, stability or calibration gate fails |

This preserves the CPU-cost investment screen while preventing a constructor with worse completion behaviour from advancing without scrutiny.

The 2% allowance is a policy choice, adopted before any construction timing. It is not an assertion that completion performance is identical.

### One issue to watch during the independent code audit

The review handoff correctly identifies a subtle risk in RAW canonicalization: the release implementation assumes provisional RAW indices form a permutation.

Its final count check alone does not prove uniqueness. The constructor appears to establish uniqueness structurally, because every new RAW group gets an appended entry and its own index, but that reasoning needs independent verification.

Similarly, Miri covered small cases but not wide promotion. I would ask the researcher to distinguish between a genuine missing safety guarantee and merely missing runtime coverage.

Neither issue requires changing the constructor before review.

### The instruction

I have reviewed both documents at `6a097d8`.

**The independent review handoff is approved.** Send it to the researcher now. The researcher should inspect the actual `examples/c30walk/ctor.rs` source at `e003a32`, focusing particularly on initialization before `set_len`, bounds assumptions in release builds, RAW promotion, canonical permutation, reused allocations and possible oracle-test blind spots. No code changes or benchmarks are authorized by that request.

**The construction-cost protocol is provisionally accepted, subject to these decisions:**

1. Warm reused-capacity construction is primary. Cold/growth costs are reported separately, not used as automatic investment gates.
2. Use production's public `bwt::build` as the control, with an explicit check that the tested corpus exercises `build_one_pass` successfully.
3. Use only the primary Wikidata corpus for the registered performance capture. Keep held-out data in the correctness evidence.
4. Do not introduce cache flushing. Preserve the natural per-arm preparation and validation environment and disclose its cache-state asymmetry.
5. Add a warm four-core completion-time no-harm rule: e4 upper 95% CI ≤ 1.02 for PASS; e4 lower CI > 1.02 for FAIL. Otherwise follow the existing INCONCLUSIVE rule. The construction CPU ceiling remains 1.06714.
6. Decide on the complete primary corpus, while separately reporting full-block and short-block economics.
7. If the independent review changes the constructor, freeze and revalidate the new source and assembly before measurement.
8. Confirm the available workstation memory can support the intended input and reference buffers without swapping. Record that preflight. Do not change the corpus or allocation policy to accommodate pressure silently.

Keep every operation required to produce a usable table inside its timed construction call, including initialization, capacity growth when applicable, promotion, RAW ordering and finalization.

Retain CPU-time and completion-time measurements separately, and report the clock-call overhead. Include the deterministic assignment, complete schedule validation, explicit correctness gates, binary identity checks and failure archiving.

Please revise the protocol accordingly. **Do not implement the timing harness yet.** Wait for the independent constructor code review, since that may change the source and the measurement pin.

No i9 work, walker changes, alternate constructor variants, production integration or performance claims are authorized at this point.

### The next sequence

The researcher audits the constructor while Claude revises the protocol. We then resolve any material correctness or memory-safety findings before authorizing the measurement harness.

If the audit passes, the next task is engineering the constructor-cost capture without another cycle of avoidable preflight failures.

The overall research position is good: 74.34% of the primary corpus reaches the intended bulk path, and exact array equivalence has passed extensive testing. The crucial unanswered question remains whether the full constructor—including its 25.66% general path and RAW ordering—fits inside the available 6.714% construction-cost margin.

That is worth measuring once the code audit and protocol are settled.
