# Assembly 1 signed-bridge audit

Trigger: PR #145 comment 6640153383.  
Reconciliation pin: `95977867fdf318b3f66420c383a73d9c48b36740`.  
Combined-build checkpoint: `888eccfc86d5d771e881b0f87d46a21e2cf1512a`.  
Profiled source: `c4e45d30`; baseline B2: `8beffe28bec9b539517b442350d8a3fda62164c8`.

## Scope and result

The signed additivity bridge reproduces exactly from the pinned same-binary class table. It closes the previously described “about 40M unexplained” instruction discrepancy inside the checkpoint's accounting model. The effect is compiler nonadditivity, not new credit and not an arithmetic artefact.

This audit did not compile a candidate, run Callgrind, replay Miri or mutants, or time anything. It independently recomputed the bridge and endpoint arithmetic from the pinned tables. Guardian D013's qualification HOLD remains binding until the Guardian lifts or revises it. No economic, integration, timing or merge conclusion is made.

## Exact own-code identity

From `gate2/paths.txt`:

| group | scanner Δ | descriptor Δ | combined Δ | nonadditive Δ |
|---|---:|---:|---:|---:|
| build | −84,060,421 | −67,268,848 | −207,720,906 | −56,391,637 |
| refill | 0 | +9,059,803 | +9,059,803 | 0 |
| finish | 0 | 0 | −60,212,114 | v3 finish group |

The complete identity is:

```
−258,873,217
= −84,060,421
  + (−67,268,848 + 9,059,803)
  + −60,212,114
  + −56,391,637
```

Thus descriptor own code is −58,209,045, refill is exactly additive, and build nonadditivity is −56,391,637. The table contains 86 rows including unchanged comparison rows; the checkpoint ledger has 83 changed classes. Forty-two build classes have nonzero nonadditivity.

The bridge's signed class totals also reproduce:

```
removed beyond additive  −318,552,365
added beyond additive    +262,160,728
net                       −56,391,637
```

## Mechanism partition

The four categories are disjoint by the bridge script's predicates and sum exactly:

| mechanism | Δ instructions | interpretation |
|---|---:|---|
| unrolled row/detour classes | −66,676,611 | looped classes emptied; 16 position-specific row and refill-detour classes added |
| avoided descriptor outer head | −12,015,336 | 3,003,834 executions × four instructions avoided; combined remains at scanner's 11 rather than additive 15 |
| S class | +19,146,504 | 2,393,313 executions × eight instructions more than the raw additive counterfactual; a cost |
| all remaining classes | +3,153,806 | signed sum of the other 39 nonzero interaction classes; a cost |
| **total** | **−56,391,637** | exact build nonadditivity |

This partition resolves the apparent discrepancy. It must not be added to the combined measured total: it is the bridge from isolated own-code deltas to that total.

The S class deserves careful wording. The combined class is 44 instructions versus raw additive `37 + 37 − 38 = 36`, hence `+8 × 2,393,313 = +19,146,504`. The separate scanner-only comparison of 13 instructions against the combined non-row part and the additive 12-instruction comparison differ by exactly the existing O class frequency, 2,393,313. Applying O once reconciles them; no extra overlap discount is registered.

## Inclusive interval and floor

The library convention closes both endpoints:

```
own saving                              258,873,217
minus charged added library work           378,882
= conservative lower                    258,494,335

lower + removed memcpy upper-only         4,454,576
= inclusive upper                       262,948,911
```

Charging the external harness caller's +868 gives `[258,493,467, 262,948,043]`; it does not change either gate comparison.

The registered overlap remains `O = 2,393,313`. The additional 3,654 is not used. With v3:

- `F = 147,606,687`;
- lower-end margin over F: 110,887,648;
- lower-end margin over best isolated component 84,060,421: 174,433,914.

The combined v3 finish-group saving, 60,212,114, is not its §7(b) contribution by itself. Charging active-tag work 14,616 and the run-call-site reload 508,988 gives `59,688,510`. The pinned leaf remains 44 instructions including `ret`.

## Memory and dependency record

The checkpoint ledger reports own-code reads −20,479,638 and writes −8,815,056. The aggregate improvement does not erase the local refill regression: refill reads are +2,562,499 and writes +5,197,912. The unrolled first-segment middle detours also spill and reload three registers. These are already charged in the instruction classes and remain relevant to any later timing model.

## Classification

- Gate 1: **reported PASS at the checkpoint; not independently replayed here**.
- Gate 2: **diagnostic findings retained**, especially unrolling, refill detour spills and the +508,988 run-finish reload.
- Gate 3 arithmetic and signed reconciliation: **PASS in the checked ledger scope**. The class identity, interval, F and both margins reproduce.
- Final combined qualification: **HOLD under Guardian D013** at the time of this publication. This audit supplies the requested signed bridge but does not impersonate the Guardian or lift its decision.
- Economics: **unresolved**. The interval is an instruction result, not a time result; historical timing captures remain INVALID.

## Residual limits

The bridge localizes every signed difference in the archived class table. It does not by itself prove raw-profile completeness: equal-frequency additions and removals can cancel inside a class, and this run did not independently replay the compressed raw profiles. The checkpoint's listing map, counters, normalized uncounted closure and own/inclusive identity remain the evidence for that coverage.

No constructor was implemented, compiled, timed or merged in this audit.
