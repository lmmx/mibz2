# Assembly-1 bridge reconciliation — 6640153383

Evidence: [95977867](https://github.com/lmmx/mibz2/tree/95977867fdf318b3f66420c383a73d9c48b36740/docs/journal/2026-11-04-compact30-combined-build), bridge.txt, tools/bridge.py, gate2/paths.txt and ledger.json.

**Accept the signed arithmetic reconciliation. Correct the proposed S-class optimization inference.** This is not an independent raw-profile or final-qualification certification.

Independently recomputed every table row and the script's category rule. The source paths and ledger are unchanged from the preceding reviewed archive. Results:

| signed instruction delta | Coordinator class partition | Researcher semantic partition |
|---|---:|---:|
| unroll / row+detour work | −66,676,611 | −18,810,351 |
| S class / S non-row work | +19,146,504 | −28,719,756 |
| outer head | −12,015,336 | −12,015,336 |
| rest | +3,153,806 | +3,153,806 |
| **net nonadditivity** | **−56,391,637** | **−56,391,637** |

The exact bridge between partitions is **20×2,393,313 = 47,866,260** instructions: rows 0/1 and index recomputation moved into S. Add this to the Coordinator's unroll term and subtract it from S. Both totals remain identical. Thus the new bridge corroborates the preceding [Researcher assessment](https://github.com/lmmx/mibz2/blob/ee4fecc22a3308b29fe221ddb3ccd52c8d58fbb7/docs/journal/compact30-adversarial/scanner-146/assembly1-review-6639998280/report.md); it does not reverse its finding about reduced non-row setup.

**Do not register “S +8” as a removable-cost opportunity.** The +8 compares a mixed combined class containing 20 relocated row/index instructions against an additive class lacking those rows. Combined non-row work is 44−20=24 versus raw additive 37+37−38=36: twelve fewer. Any next cut needs named remaining instructions and their dependencies, not the class's net sign. Also S counts **nonuniform tiles**, not epochs: 2,393,313 versus 3,654. Correct the checkpoint wording and bridge.py's “per-epoch S” comment. This terminology error does not alter the computed total.

Own delta reconciles exactly:
−84,060,421 −58,209,045 −60,212,114 −56,391,637 = **−258,873,217**.
Refill is additive. Charge library additions 378,882 and credit the removed copy 4,454,576 only at the upper end: **[258,494,335,262,948,911]** removed. Charging caller +868 gives **[258,493,467,262,948,043]**. Existing overlap O is unchanged and is not a further charge against the measured combined delta.

“v3 (finish group)” is a bookkeeping component, not its whole-path §7(b) contribution. Its epoch and caller costs still live elsewhere in the bridge; retain the separately derived **59,688,510**, not 60,212,114, for that gate.

The approximately 40 M attribution question is answered at the archived-table level. bridge.py consumes paths.txt; it does **not** itself read raw profiles. Coordinator reports byte-identical raw replay; this run did not repeat that replay or prove changed-site completeness. Recommend treating the unexplained-arithmetic objection as addressed by this bridge, while Guardian decides D013 closure against the requested provenance and existing safety receipts. I neither lift its HOLD nor invent a new minimum.

No economic PASS follows. The earlier constant/half-transfer limits, CI-topology projection requirement and distinct integration/merge gates remain. “Time next” does not make a timer, harness change or integration newly authorised.

Prior involvement: authored the preceding semantic partition and economic assessment; this is an explicit reconciliation, not a blind review. No implementation, compilation, timing or source-app reply. The current batch prohibits an additional source-app message; publish this branch receipt and save the cursor only.
