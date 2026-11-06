# Checkpoint 6637121066 — counted ledger replay and floor correction

## Scope

Ledger-only Optimizer review of the re-delivered checkpoints:

- floor correction from comment 6635380133, ruled by Guardian D010 at
  `028d7206e79fae8081cef4ea13357b4546270530`;
- counted-frequency replay from comment 6635419092, confirmed by Guardian D011
  at `080dd146353485eb2e6e7b17bc78283278563e6e`;
- scanner U2 `ba261eff245f45e1c07fdfb05fa2fe6ff1ced835`;
- descriptor arrays `7620c0c667018772a0ba3be28eb62263b55037f8`.

No constructor was implemented or compiled. Nothing was timed, benchmarked,
merged, or authorized.

Governing evidence: charter blob
`31ec5719ca24cd3204b34595401c68ffd31bb57f`, reviewer specification blob
`2839405784ec4e63d2f4cc7bd406e5dff4dd1029`, role record blob
`8ea1583944e13ecdcb48dc1cb23f31c934c700b5`.

## Archive-only replay

The common `f01_replay.py` blob
`759879843fb9cd80d380590aaaa7cabdb428852d` was run independently against:

- both raw B2 and candidate Callgrind profiles, runs 1 and 2, from the original
  scanner `49cb46b6` and descriptor `29dd071f` evidence;
- the archived B2/candidate build and refill listings;
- the counted primary records at the two new pins.

The script parses own instruction classes and library call sites, evaluates each
counter expression against the counted record, requires both raw runs to agree,
and applies the registered lower/upper conventions. The complete outputs match
the committed ledgers exactly after JSON normalization:

| Item | Changed classes | Counter support | Committed ledger blob | Exact full-output match |
|---|---:|---:|---|---|
| Scanner U2 | 31 | 31/31 | `435d8aa3113667f913a8c331daf60d1abf519fbc` | yes |
| Descriptor arrays | 34 | 34/34 | `c61c66f21e3e16f81a84ee978c9a4fde8a76158a` | yes |

## Reproduced ledger

| Model on B2 | Counted interval removed | Minimum | Lower-end margin | If +868 external caller Ir is charged | Own writes |
|---|---:|---:|---:|---:|---:|
| Scanner U2 | **[84,060,421, 84,060,421]** | 60,000,000 | 24,060,421 | [84,059,553, 84,059,553] | +3,391,432 |
| Descriptor arrays | **[57,830,163, 62,277,795]** | 40,000,000 | 17,830,163 | [57,829,295, 62,276,927] | +5,004,156 |

Scanner counter-supported own delta is −84,060,421; uncounted added and removed
terms are both zero, and all 11 library sites match.

Descriptor counter-supported own delta is −58,209,045; uncounted added and
removed terms are both zero. The lower end charges 378,882 added library Ir.
The removed per-block memcpy contributes 4,447,632 Ir only at the upper end, so
the remaining interval width is explicit rather than assumed zero.

The mapped S62-F01 counter gap is remedied for these isolated primary-corpus
models. The identity statement is **normalized uncounted instruction closure
unchanged**, not whole-binary byte identity. Frequency-class equality is not a
universal changed-site census: equal-frequency removed/added sites can cancel
before class enumeration. No omitted numerical cost is demonstrated by that
limitation; retain the pinned source/listing mapping and harden future replay to
preserve zero-net groups.

## §6.3 correction

The earlier Optimizer report used the wrong end of O and inferred an unsupported
upper bound from the minima. Charter §6.3 explicitly uses the lower bound of
demonstrated overlap:

`F = Σ translated component minima − O_lower`.

For scanner plus descriptors:

`F = 60,000,000 + 40,000,000 − 2,393,313 = 97,606,687`.

O_upper is not established by the registered minima and does not set F. Any
additional demonstrated cross-overlap is subtracted from F. With eligible v3
the current value is 147,606,687; with eligible v3 and qualified incremental
SSE2 it is 151,606,687, before any further demonstrated cross-overlap with the
scanner or descriptor diffs.

## Disposition

- Scanner isolated gate 3: **PASS** at `ba261eff`.
- Descriptor isolated gate 3: **PASS** at `7620c0c6`.
- S62-F01 mapped counter omission: **remedied** at these pins.
- Canonical floor: **F = 97,606,687** for scanner plus descriptors under the
  currently demonstrated overlap.
- §6.2 closure, combined composition, a fresh combined ledger, both §6.3 pass
  conditions, v3/SSE2 eligibility, and integration authority remain separate
  and unresolved by this review.
