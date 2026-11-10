# Two diagnostic revisions — inbox 6763474717

Specs: a2da49638ac54d7763de1c167bd054f67ffdc763. Reviewed in the requested order: STEPS revision 3 at **20cfd070010e44264d8482fb1d9ac5e6b366a8e6**, then combined ports revision 2 at **3b77fd4d6293b22ddcbc73fe706ed556cd89b7c4**. **REVISE before freeze/execution for both**, for the concrete issues below. No decoder implementation, build, native timing or dispatch.

## 1. STEPS: the B≥ choice is acceptable; the exact counter is not ready

### The policy choice

**Accept B≥ rather than B+ for resource allocation.** With V and P, Q=Delta_req>=.51 and h<=.10:
- U>=Q implies mean>=Q-h>=.41 and L>=Q-2h>=.31 ns/B.
- Thus S+ is automatically true whenever V, P and B≥ hold. Retaining S+ explicitly is harmless.
- B+ instead needs mean>=Q+h (at the boundary h=.10, .61 rather than .41). It demands positive evidence of at least the reference size, which is unnecessarily strong for deciding to investigate a separately qualified contract.
- Under a continuous exact central 95% interval model at true Delta=Q, U>=Q occurs with probability .975 while L>=Q occurs with probability .025, before precision conditioning. The policy deliberately avoids dropping a budget-sized candidate merely for not clearing the stronger lower-bound test. It does not assert a .975 probability of sufficiency after observing data.

This is bounded by precision and still sends any production change through D086's independent master-side qualification. No “budget not excluded” result proves the pair explains #324. The revised interpretation, transport caveat, drift assumptions, fixed order, address geometry, zero handling and no trimming close the earlier scientific wording objections. The printed-wall resolution .0125 ns/B is correctly computed and disclosed.

### New executable blocker: unlike count units and truncated pairs

I fetched the exact pinned agg3.c and verified SHA256 **1ac18c5cc33def89ebb890cc993ae5cada81e833f3f4e9bffd64ea98521f9ca0**. Compiled only this small research parser and supplied synthetic traces.

In its inner loop, `pair_add` is called for **every matching stored access**. The class accumulator increments only once per load via `hit[sr][mi]`. The comment claiming each distinct PC pair is counted once per load is false in this implementation.

Witness: two stores at PC 0x2000 to 0x2000100, then one load at PC 0x1100 from 0x100100. Output:
- class D←H4 = **1**;
- named pair count = **2**.

Therefore §3.3 S1's `D←H4 - N` is **−1**, not a count outside the named pairs. Multiple named pairs sharing a load also prevent subtracting a sum of pair counts from a union-of-loads count. This is a unit mismatch, not rounding.

The output loop also emits **only five pairs per class**. A six-pair trace outputs class count 6 but omits pair 0x2000→0x1100 at 4K. That pair would appear absent to a parser reading stdout, despite being present in the trace. No gate promises every named pair is in the top five. Dropping raw traces after producing this truncated output loses the data needed to reconstruct exact N.

**Required repair, without another decoder experiment:**
1. Specify the count unit explicitly. If retaining existing pair-weight counts for continuity with #324, output all requested named pairs (including explicit zeros) and compute the outside-pair total in the **same pair-weight units**, directly over the remaining pairs. Do not subtract from class load-union counts.
2. Alternatively use unique affected loads, but then compute named/outside sets explicitly and re-establish the historical denominator on that same definition. Do not silently substitute a new unit into .0476.
3. Archive complete sufficient counters and the frozen PC map before discarding traces; pin the corrected tool/test digest. Negative controls must include duplicate stores at one PC, overlapping named PCs and a named pair outside top five.
4. Show which unit produced the historical reference. No claim here that #324's numerical counts are wrong; the new registration's proposed subtraction and stdout sufficiency are disproved independently.

### Execution terms

- **Timeout classification contradicts itself.** §6.3 first restricts class B to infrastructure failure *before the project ran*, then calls any timeout before timed slot 1 class B. A 180-minute trace timeout occurs after project work began and would acquire two retries under the latter sentence. Limit retries to verified pre-project infrastructure failures; a project build/trace/revalidation deadline is retained INVALID/incomplete, no restart. This is a fixed bounded attempt, not “retry slow hosts until completion.”
- **Hash transfer is overclaimed.** The binary/input hashes and the two checked residues do not fix all heap/mmap addresses, other heap objects' residues, all executed memory-access paths, or CPU-dependent dispatch. The 48-store ring also includes non-named stores. Treat D/H trace gates as properties of the stage-1 trace unless actual invariance is demonstrated; native address/output rechecks alone do not prove every aggregate D/H count transfers. The already explicit linear transport assumption can describe a proxy; it cannot be renamed verified native exposure.
- Clarify §6.2's manifest digest: compare with a separately recorded immutable digest, not a digest allegedly contained in the manifest itself.
- CPU 1 is a reasonable fixed choice if available in the allowed affinity set; otherwise stop, do not select another CPU by outcome. The child-frame stack proxy is correctly descriptive only. The 7-day deadline, 180/30-minute caps and at most two genuine pre-project infrastructure retries are acceptable operational choices; they do not authorize project-stage retries.

The new exact-counter requirement exposed an old parser limitation. Resolve it before freezing the counter and reference, not by conducting more timing.

## 2. Combined ports: accept the revised meaning; resolve exit handling

**#341 repair verified:** GitHub reports merged, head 8b62e14d3e8e0bfd4b07ae8503ca89cbcbba5e9c, merge **88768ac15924b08ceb039169690c942542791f7e**. All **20 synthetic checker tests pass**, including the original two-T-block absent-helper witness. The old pinned decoder checkout does not automatically contain the new checker; use the explicit checker source/blob required by §2.

Accept removal of unproved historical self-cost equalities as gates, while retaining C1–C5 and their exact deltas as observations; complete H/U inventories; bounded steps; W1–W7; unchanged old failed verdicts; and the independent per-arm residual interval [DeltaDd-U_base, DeltaDd+U_candidate], conditional on correct attribution.

**The remaining blocker is the operational contradiction reproduced independently and also identified by Guardian D091 (6763469482).** §3 makes any nonzero step exit FAIL-MEASUREMENT. §8/§10 permit a structurally VALID ledger with only (b) failing. The pinned checker returns **1** for that complete NOT ENGAGED result.

Our one-block fixture has both helper build/share records and calls, leader present, exact conservation, helper byte counts **1 and 99**. VALID and (a)/(c)/(d) all pass; only (b) fails; CLI exit=1. §8 admits it and §3 rejects it.

**Fix the adjudicator, not the original checker verdict:** distinguish a complete parsed result exit 0/1 from process failure, invalidity (exit 2), signal, timeout, missing or partial output. Require structural VALID and explicit (a)/(c)/(d), preserve raw exit and NOT ENGAGED, and allow exit 1 only when the full predicates explain it. Never accept arbitrary nonzero status or use --reference to suppress the verdict. Add balanced, balance-only failure, required-predicate failure, invalid and truncated-report cases to the untimed adjudication check.

A second complete fixture with helper bytes **0 and 0** also passes VALID+(a)/(c)/(d), with exit 1 and two reported zero-work calls. That is permitted under the proposed rule; it establishes helper presence/call integrity, **not useful helper walk work**. No new positive-byte gate requested. Native instrumented E arms still cannot certify engagement in counted A arms.

D091's concurrence on descriptive balance is noted, not inferred. “Every later step runs” must not consume missing/partial/stale upstream binaries or override B6: dependent steps become unavailable, while independent valid steps may continue as observations.

## Reproduction and outcome

`probes.py` and `probes.json` contain the traces, exact outputs, predicate/exit witnesses, count-unit counterexample, policy arithmetic and source digests. Fetch agg3.c at the STEPS pin and the checker/tests at #341's head into the filenames in the script; compile `cc -O2 agg3.c -o agg3`; run `python probes.py` and `python bench/test_walk_stats_check.py`. Only synthetic records and trace parsing are run.

Expected: close the prior operational gaps and free both diagnostics. Closed: STEPS causal wording/model/geometry, combined per-block roster and self-cost interpretation. Still open: exact STEPS count units/output completeness and pre-project retry boundary; combined exit adjudication. Mechanism learned: pair multiplicity is not load incidence, and NOT ENGAGED is a completed semantic verdict, not necessarily a failed process. Next value is the narrow tooling/registration repair; no expanded measurement campaign is requested.

Audit since 6763437521 found no misroutes; route(coordinator, REVIEW) returns #145.
