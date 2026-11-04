# Checkpoint 6634996672 — canonical B2 overlap map

## Scope and verdict

This is an Optimizer ledger audit of the coordinator checkpoint. It does not
implement, compile, time, merge, or grant authority. The map is pinned to:

- scanner U2 49cb46b65070059af49e906fa8d98693b808bda2;
- descriptor arrays 29dd071f8787a28560a7a081fd7ac0ef13991466;
- common B2 source 8beffe28bec9b539517b442350d8a3fda62164c8;
- B2 listings cd14fa90e45c39d7d42b4eb71efa6e3a9b28f78f;
- independent review S62-F02
  e62fc15f91311823ab0c48a6f6e9c7dd35a28d10.

The attached [machine-readable map](overlap-map.json) has one row for every
changed execution-count class in each pinned build::<false> listing: 31 scanner
classes and 32 descriptor classes. Every row enumerates the exact B2 and variant
function-relative offsets. The tables below are a compact rendering; rows with
many offsets show their count and span, while the JSON preserves the complete
lists.

The scanner build delta is −84,060,421 Ir. The descriptor build delta is
−67,268,848 Ir; descriptor refill::<false> adds 7,796,868 + 1,262,935 =
9,059,803 Ir, giving the audited own-code delta −58,209,045. Scanner refill is
unchanged.

## Overlap bound and §6.3 arithmetic

S62-F02 proves one shared removed B2 instruction: build +0x12b5,
mov 0x8(%rsp),%r13, executed 2,393,313 times. Therefore:

- demonstrated lower bound: **O ≥ 2,393,313**;
- conservative set-theoretic upper bound from the accepted 60M scanner and 40M
  descriptor minima: **O ≤ min(60,000,000, 40,000,000) = 40,000,000**.

No smaller upper bound is booked. The exact-frequency coincidences at 3,575,
106 and 79 executions are not instruction identity; the map does not turn them
into a discount.

Hence 60,000,000 + 40,000,000 − O lies in:

| Case | O used | Non-double-counted subtotal |
|---|---:|---:|
| Least favourable / certified floor | 40,000,000 | **60,000,000** |
| Demonstrated-overlap-only endpoint | 2,393,313 | **97,606,687** |

The second line is the favourable endpoint under the currently demonstrated
overlap, not a certified lower floor. The prior 97,606,687 wording is retained
as that endpoint and is no longer presented as exhaustive overlap.

Conditional increments, kept separate because their cross-overlap is not
bounded here:

| Included minima | Zero-additional-overlap arithmetic | Certified floor without a new cross-overlap bound |
|---|---:|---:|
| v3 +50M | 110,000,000…147,606,687 | 60,000,000 |
| SSE2 +4M | 64,000,000…101,606,687 | 60,000,000 |
| v3 +50M and SSE2 +4M | 114,000,000…151,606,687 | 60,000,000 |

These are arithmetic scenarios only. v3 remains conditional under its compiler
STOP and §7 inclusion rules; SSE2 remains a proposal with unknown net.

## Frequency evidence flags

Rows marked **profile-only** are exactly the classes still lacking the
counted-instantiation counter/path-name evidence required by §5.3(b):

- scanner: added Ir 4,359,203 and removed Ir 17,203;
- descriptor: added Ir 1,173,383 and removed Ir 3,279,857.

Those totals reproduce S62-F01. Their frequencies are usable to audit the
numerical interval, but their path names remain provisional until counted-only
counters are archived and the uncounted listing is shown unchanged.

## Scanner U2 build classes

| Frequency | B2 → U2 instructions | ΔIr | Path/class | Offsets | Evidence |
|---:|---:|---:|---|---|---|

| 10,544,477 | 5 → 0 | -52,722,385 | fresh/lookahead window advance checks | B2 6 offsets (+0xa30…+0xae0); variant none | raw profile + counter/structural |
| 10,541,706 | 9 → 5 | -42,166,824 | lookahead checked-slice setup | B2 12 offsets (+0xa44…+0xa6f); variant 6 offsets (+0x980…+0xb1a) | raw profile + counter/structural |
| 2,396,084 | 10 → 2 | -19,168,672 | matched-prefix second-slice setup | B2 13 offsets (+0x9e8…+0xa10); variant +0x947, +0x94a | raw profile + counter/structural |
| 3,003,834 | 13 → 11 | -6,007,668 | outer scan iteration | B2 16 offsets (+0x9d2…+0xa80); variant 13 offsets (+0x93d…+0x9c0) | raw profile + counter/structural |
| 1,116,556 | 8 → 4 | -4,466,224 | full-tile fill/advance join | B2 9 offsets (+0x12ca…+0x990); variant 5 offsets (+0xa8f…+0x8f0) | raw profile + counter/structural |
| 2,393,313 | 38 → 37 | -2,393,313 | nonuniform full-tile setup | B2 44 offsets (+0x990…+0x990); variant 40 offsets (+0x8f0…+0x8f0) | raw profile + counter/structural |
| 610,521 | 32 → 31 | -610,521 | run entry | B2 35 offsets (+0xa29…+0x9d2); variant 35 offsets (+0x969…+0x93d) | raw profile + counter/structural |
| 607,750 | 4 → 3 | -607,750 | lookahead mismatch join | B2 5 offsets (+0xa6f…+0xae5); variant +0xb1a, +0xb1f, +0xb24 | raw profile + counter/structural |
| 3,424 | 3 → 0 | -10,272 | low-frequency/multi-region build class | B2 +0x7d0, +0x7d3, +0x7d6, +0x150f; variant none | profile-only; counter/name pending |
| 3,575 | 5 → 4 | -3,575 | low-frequency/multi-region build class | B2 6 offsets (+0x970…+0x9e3); variant 5 offsets (+0x8db…+0x93d) | profile-only; counter/name pending |
| 786 | 12 → 8 | -3,144 | low-frequency/multi-region build class | B2 14 offsets (+0x930…+0x918); variant 10 offsets (+0x13d1…+0x13b8) | profile-only; counter/name pending |
| 434 | 204 → 200 | -1,736 | per-block common path | B2 224 offsets (+0x0…+0x1579); variant 220 offsets (+0x0…+0x14ef) | raw profile + counter/structural |
| 230 | 41 → 39 | -460 | low-frequency/multi-region build class | B2 49 offsets (+0x277…+0x150f); variant 46 offsets (+0x285…+0x1304) | raw profile + counter/structural |
| 106 | 12 → 11 | -106 | low-frequency/multi-region build class | B2 14 offsets (+0x13d0…+0x13d0); variant 13 offsets (+0x1420…+0x1420) | profile-only; counter/name pending |
| 79 | 3 → 2 | -79 | low-frequency/multi-region build class | B2 +0x8a7, +0x8af, +0x8b2; variant +0x12a0, +0x12a3 | profile-only; counter/name pending |
| 9 | 12 → 9 | -27 | low-frequency/multi-region build class | B2 14 offsets (+0x1400…+0x13e8); variant 11 offsets (+0x144b…+0x1438) | profile-only; counter/name pending |
| 12 | 0 → 1 | 12 | low-frequency/multi-region build class | B2 none; variant +0x7c7 | profile-only; counter/name pending |
| 22 | 4 → 8 | 88 | low-frequency/multi-region build class | B2 5 offsets (+0x13c1…+0x1379); variant 9 offsets (+0x13f9…+0x131a) | profile-only; counter/name pending |
| 121 | 0 → 1 | 121 | low-frequency/multi-region build class | B2 none; variant +0x138a, +0x7e7 | profile-only; counter/name pending |
| 218 | 6 → 7 | 218 | low-frequency/multi-region build class | B2 7 offsets (+0x145b…+0x14d1); variant 8 offsets (+0x1333…+0x7cc) | profile-only; counter/name pending |
| 165 | 0 → 3 | 495 | low-frequency/multi-region build class | B2 none; variant +0xb10, +0xb15, +0xb18, +0xb29 | profile-only; counter/name pending |
| 3,654 | 33 → 35 | 7,308 | snapshot/epoch path | B2 36 offsets (+0x7dc…+0x7d0); variant 39 offsets (+0x7f6…+0x7f6) | raw profile + counter/structural |
| 2,606 | 0 → 4 | 10,424 | low-frequency/multi-region build class | B2 none; variant 5 offsets (+0x9aa…+0xb29) | profile-only; counter/name pending |
| 18,684 | 36 → 37 | 18,684 | open-block path | B2 38 offsets (+0x4b0…+0x4b0); variant 39 offsets (+0x4b0…+0x4b0) | profile-only; counter/name pending |
| 23,559 | 8 → 9 | 23,559 | finish-call path | B2 9 offsets (+0x15e7…+0x15b0); variant 10 offsets (+0x1577…+0x1540) | profile-only; counter/name pending |
| 296,405 | 43 → 44 | 296,405 | whole-group branch | B2 47 offsets (+0xcc7…+0x1339); variant 48 offsets (+0xd04…+0x1267) | profile-only; counter/name pending |
| 302,049 | 9 → 11 | 604,098 | second-segment refill detour | B2 11 offsets (+0x130d…+0x12f7); variant 13 offsets (+0xad1…+0xab8) | profile-only; counter/name pending |
| 610,356 | 0 → 1 | 610,356 | run-path alignment nop | B2 none; variant +0x973 | profile-only; counter/name pending |
| 801,475 | 18 → 19 | 801,475 | vector-fill entry | B2 20 offsets (+0xbd9…+0xbf2); variant 21 offsets (+0xc18…+0xc31) | profile-only; counter/name pending |
| 996,634 | 10 → 12 | 1,993,268 | first-segment refill detour | B2 12 offsets (+0x127e…+0x1268); variant 14 offsets (+0xa3d…+0xa28) | profile-only; counter/name pending |
| 9,933,956 | 0 → 4 | 39,735,824 | fresh whole-tile load | B2 none; variant 5 offsets (+0x99c…+0x980) | raw profile + counter/structural |

## Descriptor-array build classes

| Frequency | B2 → descriptor instructions | ΔIr | Path/class | Offsets | Evidence |
|---:|---:|---:|---|---|---|

| 29,357,924 | 12 → 10 | -58,715,848 | scalar append row loop A | B2 14 offsets (+0x1250…+0x1250); variant 12 offsets (+0x1370…+0x1370) | raw profile + counter/structural |
| 8,935,084 | 12 → 10 | -17,870,168 | scalar append row loop B | B2 14 offsets (+0x12e0…+0x12e0); variant 12 offsets (+0x1410…+0x1410) | raw profile + counter/structural |
| 467,712 | 14 → 0 | -6,547,968 | descriptor initialization path | B2 15 offsets (+0x810…+0x810); variant none | raw profile + counter/structural |
| 2,393,313 | 38 → 37 | -2,393,313 | nonuniform full-tile setup | B2 44 offsets (+0x990…+0x990); variant 43 offsets (+0xa50…+0xa50) | raw profile + counter/structural |
| 597,255 | 2 → 0 | -1,194,510 | open join | B2 +0x1230, +0x1235, +0x1346; variant none | profile-only; counter/name pending |
| 1,085,833 | 25 → 24 | -1,085,833 | fill path | B2 28 offsets (+0xb9d…+0xc49); variant 27 offsets (+0xc8c…+0xd3a) | profile-only; counter/name pending |
| 496,004 | 34 → 33 | -496,004 | open path | B2 36 offsets (+0xb5c…+0xb50); variant 35 offsets (+0xc4c…+0xc40) | profile-only; counter/name pending |
| 302,049 | 9 → 8 | -302,049 | second-segment refill detour | B2 11 offsets (+0x130d…+0x12f7); variant 10 offsets (+0x1441…+0x1426) | profile-only; counter/name pending |
| 27,776 | 35 → 31 | -111,104 | initialization loop | B2 37 offsets (+0x2f0…+0x380); variant 33 offsets (+0x2e0…+0x340) | profile-only; counter/name pending |
| 55,986 | 3 → 2 | -55,986 | sweep path | B2 +0x1520, +0x1524, +0x152b, +0x158b; variant +0x1620, +0x1626, +0x1690 | profile-only; counter/name pending |
| 13,266 | 2 → 0 | -26,532 | refill detour | B2 +0x1339, +0x133e; variant none | profile-only; counter/name pending |
| 3,575 | 5 → 3 | -7,150 | refill detour | B2 6 offsets (+0x970…+0x9e3); variant +0xa40, +0xa42, +0xa4a, +0xab3 | profile-only; counter/name pending |
| 1,769 | 12 → 10 | -3,538 | scalar append row loop C | B2 14 offsets (+0x900…+0x900); variant 12 offsets (+0x9c0…+0x9c0) | raw profile + counter/structural |
| 432 | 6 → 5 | -432 | low-frequency/multi-region build class | B2 7 offsets (+0x263…+0x299); variant 6 offsets (+0x258…+0x291) | profile-only; counter/name pending |
| 79 | 3 → 0 | -237 | low-frequency/multi-region build class | B2 +0x8a7, +0x8af, +0x8b2; variant none | profile-only; counter/name pending |
| 106 | 12 → 10 | -212 | scalar append row loop D | B2 14 offsets (+0x13d0…+0x13d0); variant 12 offsets (+0x14d0…+0x14d0) | raw profile + counter/structural |
| 8 | 15 → 13 | -16 | low-frequency/multi-region build class | B2 17 offsets (+0x1734…+0x239); variant 15 offsets (+0x1877…+0x237) | profile-only; counter/name pending |
| 1 | 16 → 14 | -2 | low-frequency/multi-region build class | B2 18 offsets (+0x16e7…+0x299); variant 16 offsets (+0x1827…+0x291) | profile-only; counter/name pending |
| 2 | 28 → 27 | -2 | low-frequency/multi-region build class | B2 31 offsets (+0x16df…+0x277); variant 30 offsets (+0x1822…+0x269) | profile-only; counter/name pending |
| 9 | 12 → 14 | 18 | low-frequency/multi-region build class | B2 14 offsets (+0x1400…+0x13e8); variant 16 offsets (+0x1504…+0x14e7) | profile-only; counter/name pending |
| 230 | 41 → 42 | 230 | low-frequency/multi-region build class | B2 49 offsets (+0x277…+0x150f); variant 50 offsets (+0x269…+0x160e) | profile-only; counter/name pending |
| 233 | 10 → 11 | 233 | low-frequency/multi-region build class | B2 10 offsets (+0x1625…+0x164c); variant 11 offsets (+0x177d…+0x17aa) | profile-only; counter/name pending |
| 786 | 12 → 13 | 786 | refill detour | B2 14 offsets (+0x930…+0x918); variant 15 offsets (+0x9f4…+0x9d7) | profile-only; counter/name pending |
| 434 | 204 → 211 | 3,038 | per-block common path | B2 224 offsets (+0x0…+0x1579); variant 231 offsets (+0x0…+0x166f) | raw profile + counter/structural |
| 18,684 | 36 → 37 | 18,684 | open-block path | B2 38 offsets (+0x4b0…+0x4b0); variant 40 offsets (+0x509…+0x4b0) | profile-only; counter/name pending |
| 55,552 | 13 → 14 | 55,552 | sweep path | B2 15 offsets (+0x152d…+0x1520); variant 16 offsets (+0x1628…+0x1620) | profile-only; counter/name pending |
| 3,654 | 33 → 53 | 73,080 | snapshot/epoch path | B2 36 offsets (+0x7dc…+0x7d0); variant 56 offsets (+0x7cc…+0x7c0) | raw profile + counter/structural |
| 296,405 | 43 → 44 | 296,405 | whole-group branch | B2 47 offsets (+0xcc7…+0x1339); variant 48 offsets (+0xdbe…+0x1330) | profile-only; counter/name pending |
| 801,475 | 18 → 19 | 801,475 | vector-fill entry | B2 20 offsets (+0xbd9…+0xbf2); variant 21 offsets (+0xcc8…+0xce1) | profile-only; counter/name pending |
| 610,521 | 32 → 35 | 1,831,563 | run entry | B2 35 offsets (+0xa29…+0x9d2); variant 38 offsets (+0xafe…+0xa92) | raw profile + counter/structural |
| 230,202 | 0 → 28 | 6,445,656 | descriptor setup path | B2 none; variant 29 offsets (+0x7f0…+0x7f0) | raw profile + counter/structural |
| 3,003,834 | 13 → 17 | 12,015,336 | outer scan iteration | B2 16 offsets (+0x9d2…+0xa80); variant 20 offsets (+0xa92…+0xb60) | raw profile + counter/structural |

## Canonical overlap decision

Book only O_lower = 2,393,313 as demonstrated address identity and
O_upper = 40,000,000 as the conservative bound. Do not claim the class map
exhausts compiler interactions. A fresh combined constructor listing and ledger
remain required before any combined qualification decision.


