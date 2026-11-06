# PR #146: independent accounting and register-pressure reconstruction

Evidence pin: `7f74e9f30e31af33b0d66fec3fe89af1758307d9`. This analysis read the source, complete archived release listings, corpus counters and archived Callgrind records before consulting the checkpoint's conclusions. The proposal-site section subsequently inspected the U1–U3 table. It did not compile or execute a constructor or collect any new profile. It is advisory and grants no implementation authority.

Reproduce from a checkout of the evidence pin:

```sh
python3 reconstruct.py --root /path/to/evidence-checkout --output reconstruction.json
```

`reconstruction.json` pins every consumed file with SHA-256 and gives exact address/line anchors, frequencies, full-function instruction counts, direct-callee attribution and direct-`%rsp` operand counts. The replay parser reads both `instr` and `line` positions, accumulates repeated address records, and separates call-edge costs from self costs.

## What is established

The primary structural record contains 434 blocks and 207,006,515 rows. Its exact scanner counters are A=2,396,084 fresh pairs, J=10,541,706 lookahead pairs, H=607,750 held reuses, Q=2,771 runs without another full tile, S=2,393,313 nonuniform tiles, X=1,116,556 second segments, and 1,875 tail rows. Q is observed here; a historical bound must not be reused as an observed value. `16*S + 1,875 = 38,294,883` scalar rows and `A=S+Q` hold.

The raw instruction archive reproduces the saved first run exactly:

| Quantity | Same-binary v2 | Scanner | v2 minus scanner |
|---|---:|---:|---:|
| Build self Ir | 1,026,601,670 | 970,405,853 | 56,195,817 |
| Build inclusive Ir | 1,294,414,196 | 1,238,210,529 | 56,203,667 |

The second saved summary gives 56,205,651 inclusive saving, with the same self saving. The endpoints of two runs do not establish a conservative whole-constructor interval. The reproducible self result remains 3,804,183 below the 60 M qualification minimum; the first observed inclusive result remains 3,796,333 below it. Neither shortfall turns instruction counts into a CPU-time prediction.

The B2 archived build listing and the same-binary v2 listing each contain 1,431 instructions. They match after normalizing instruction addresses, branch targets to instruction ordinals, and RIP relocations. B2 `finish` and scanner `finish` match under the same normalization (379 instructions); B2 `refill` and scanner `refill` also match (67). This is a strong bridge for these bodies, not proof of identical relocated data, external allocator/library code, allocation state, compiler invocation, or all callers. Those remain provenance obligations for the whole-constructor ledger.

Direct inclusive callees account for the difference between self and inclusive saving. In particular, scanner `memcpy` costs 1,840 more Ir in this archive, while allocator/free edge costs differ in the opposite direction. Equal `finish`/`refill` self and inclusive costs in this execution do not justify dropping other callees or using allocator variability as a certified credit.

## New attribution: the write regression is almost entirely a refill preservation cost

The two ordinary full-tile row loops are 12 instructions in both v2 and scanner. The first runs 29,357,924 times and the second 8,935,084 times. Their sum is `16*S=38,293,008`. No additional instruction or memory operand appears on these common no-refill paths.

The out-of-line refill call wrappers are different:

| Wrapper | v2 address, length | Scanner address, length | Calls | Added Ir |
|---|---|---|---:|---:|
| First segment | `3babe..3baec`, 10 | `41bae..41bf2`, 14 | 996,634 | 3,986,536 |
| Second segment | `3bb4d..3bb77`, 9 | `41c6e..41ca0`, 12 | 302,049 | 906,147 |
| Total | | | | **4,892,683** |

The first scanner wrapper preserves `%r10`, `%r9`, `%r8` in frame slots `0x30`, `0x18`, `0x50` and reloads them. v2 preserves only the analogous source pointer and reloads its epoch end. Relative to v2 this adds two direct stack stores and two direct stack reads per first-segment refill: **1,993,268 additional stores and 1,993,268 additional reads**. The second scanner wrapper instead makes four register copies around the call and removes one stack reload, yielding three extra instructions per hit.

The first wrapper alone explains **1,993,268 of the 1,995,539 additional explicit self writes (99.886%)**. The residual is 2,271 writes. This gives a concrete compiler-cost target rather than attributing the entire regression vaguely to pressure in the hot scalar loop.

The independent weighted assembly-operand model gives:

| Explicit direct `%rsp` operations in build | v2 | Scanner | v2 minus scanner |
|---|---:|---:|---:|
| Reads | 36,881,867 | 33,492,392 | 3,389,475 |
| Writes | 17,015,834 | 19,011,373 | -1,995,539 |

All modeled explicit-memory differences occur at direct `%rsp` operands; non-`%rsp` explicit operand totals are equal. `push`, `pop`, `call`, and `ret` implicit stack operations are counted separately and have zero net difference. These are architectural operand counts, with read-modify-write classified as one read plus one write; they are **not** absolute Callgrind Dr/Dw values. Their differences match the saved self Dr/Dw differences, which is corroboration. The raw archive contains only Ir, so it cannot reproduce raw Dr/Dw events, inclusive callee traffic, bytes moved or cache behavior.

Anchors: [scanner wrappers and row loops](https://github.com/lmmx/mibz2/blob/7f74e9f30e31af33b0d66fec3fe89af1758307d9/docs/journal/2026-11-04-compact30-scanner-prototype/listings/ctor2s-build-false.asm#L556); [v2 wrappers and row loops](https://github.com/lmmx/mibz2/blob/7f74e9f30e31af33b0d66fec3fe89af1758307d9/docs/journal/2026-11-04-compact30-scanner-prototype/listings/v2-build-false-same-binary.asm#L1054).

## Falsifiable register-pressure target and its limit

A useful research target is to recover wrappers of at most 10 and 9 instructions while retaining the two 12-instruction ordinary row loops, unchanged row/call frequencies, and no displaced cost elsewhere. Recovering only those existing wrapper penalties has a **gross exposure of 4,892,683 Ir**, not a verified net saving. It is enough in arithmetic to bridge the current self shortfall, but no design or compiler result establishing that recovery exists in this review.

The archived attempts show why the row-loop condition matters. Attempts 1 and 2 each contain 1,355 instructions; they differ in two instruction scheduling groups but have the same row-loop lengths. Their first full-tile row loop `41910..41942` is 13 instructions, including a descriptor-base `lea` per row; their second `419a0..419d3` is also 13 instructions, with an additional address-generation step. Applying the final structural row frequencies gives **38,293,008 Ir gross exposure for one added instruction per full-tile row**. The cheaper refill wrappers in those attempts do not make this per-row cost vanish. Re-expressing a scanner can move pressure between common rows and rarer refill paths; comparing only one of them is unsound.

This is a compiler target, not an authorization or an implemented transformation. Any later design developed from this review requires its own independence disclosure and fresh formal evidence review.

## U1–U3: disjoint present sites, no additive future guarantee

The eight candidate U1/U2 instructions are `41b0c`, `41b12`, `41b15`, `41c00`, `41c04`, `41c07`, `41c0d`, `41c15`. The U3 phase comparison/branch is `41b23`, `41b27`. Each site executes exactly S times in the raw archive. Thus the present-site sets are disjoint, with gross exposures **19,146,504** and **4,786,626**, totaling **23,933,130** Ir. There is no address-level double credit in that narrow enumeration.

This is not a lower bound on a future saving. The phase branch may be replaced by a jump, some pointer calculations may remain needed, unsafe preconditions may change compiler assumptions, and register allocation can add costs elsewhere. Bounds-proof and phase-proof expressions may also enable the same downstream simplification, so hypothetical gains from alternative rewrites cannot be added. Count an eliminated old instruction once and recount every new changed path. The phase-state proposal must not independently credit these phase-check sites again.

Anchors: [bounds and phase checks](https://github.com/lmmx/mibz2/blob/7f74e9f30e31af33b0d66fec3fe89af1758307d9/docs/journal/2026-11-04-compact30-scanner-prototype/listings/ctor2s-build-false.asm#L524); [first/second segment source](https://github.com/lmmx/mibz2/blob/7f74e9f30e31af33b0d66fec3fe89af1758307d9/examples/c30walk/ctor2s.rs#L600).

## Disposition

- **Reproduced:** primary frequencies; complete build self Ir; the first archived inclusive Ir; ordinary row-loop equivalence; exact refill-wrapper penalty; direct stack traffic delta; normalized B2 internal-body bridge; distinct present U1–U3 site exposures.
- **HOLD for verified gate-3 qualification:** a complete charter-method ledger, compiler/source and external-callee provenance, missing raw memory events, and bounds on all variable/unresolved costs. The observed archive supports a below-minimum result but is not itself a certified conservative whole-constructor interval.
- **Advisory research only:** wrapper-cost recovery and phase/state simplifications. No credited net saving, scope expansion, unsafe implementation approval or new performance claim.
