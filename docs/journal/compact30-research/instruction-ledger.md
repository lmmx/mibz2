# Canonical cumulative instruction ledger

Baseline B2: frozen v2 `8beffe28bec9b539517b442350d8a3fda62164c8`.
B3: frozen v3 `dbf5139e12ae60b5f5bff40059b88d65ddccea41`.
Unit: one construction of primary `wikidata-20261026.bz2`, 434 blocks,
207,006,515 rows. See [pins and replay](INDEX.md).

ACHIEVED means compiled sites and stated frequencies have been checked **within
the named scope**. It does not mean qualification or a complete whole-constructor
net. CONDITIONAL rows earn no cumulative verified net credit. The checkpoint's
own charter §5 ledger remains authoritative for its gate; any disagreement is a
review finding, never a silent replacement.

## Ranked baseline costs

| Cost center | Instructions / corpus | Scope and evidence |
|---|---:|---|
| Scalar append | 459,538,596 | 12 × 38,294,883; all four B2 loop copies replayed |
| Scanner classification | 221,503,931–222,731,873 | Ultra audit §C, bounded Q/W; per-block bounds independently replayed |
| Fast full finalization | 133,437,375 | 81 × now-exact 1,647,375 eligible finishes; entry calls excluded |
| Nonuniform tile setup/advance | 98,125,833–114,879,024 | 41S+7X; Ultra bounds, no scalar-body double credit |
| Refill wrappers | 69,605,726–111,402,124 | finish body excluded; per-block lower bound independently replayed |
| Fragment filling | 43,436,844–114,277,290 | length-dependent; missing histogram prevents a sharp total |
| Bulk emitter core | 51,273,000–51,883,521 | paired loop/odd tail; run setup excluded |
| RAW reconstruction core | 23,772,928 | 16 × 1,485,808 fields; exceptional detours excluded |

These are disjoint scoped baseline costs, not a closed whole-constructor total.
The audit's revised K is 1,130,193,036–1,261,424,207, plus unresolved E. Its
proven E floor is 25,106,954; it supplies no upper bound. Library allocation,
zeroing/copies, slow paths, canonicalization, padding/PARK and validation cannot
be assigned zero. The large scalar cost is 153,179,532 explicit reads,
76,589,766 writes, including 38,294,883 pointer RMW instructions. RMW is one
instruction, one read and one write, not three instructions.

## Achieved versus conditional savings

| ID / status | Baseline and scope | Instruction saving | Added/removed memory and dependencies | Gate/overlap |
|---|---|---:|---|---|
| A1 ACHIEVED | B3 fast leaf versus B2 fast body only | 60,952,875 gross | 5 fewer explicit reads and 2 fewer explicit writes/leaf; 6 pushes+6 pops removed/leaf; snapshot chain and frame removed | 44 vs 81 instructions. Gross only; leaf has 12 reads, 6 writes. Compiler STOP stands |
| A3 ACHIEVED | Historical B3 §6 whole-path model | **[60,049,330, 60,345,544] verified** | Corrected explicit reads −6,185,349; writes −3,290,662; push/pop −19,768,500 already in instruction credit. Tag loads, slow guards, setup/epoch stores and residual paths charged | [Independent closeout review](reviews/v3/report.md). Includes A1; never add it again. Formal compiler STOP stands; §7 inclusion remains conditional |
| A2 ACHIEVED, scoped | PR #146 same-binary build own body, `7f74e9f` | 56,195,817 | New spills and loop changes included in own-body Ir. Raw Dr/Dw profiles absent; summary-only memory figures remain conditional | Independent per-instruction replay; excludes callees. Does not clear 60M minimum |
| C2 CONDITIONAL / HOLD | PR #146 claimed whole-constructor interval | [56.18M,56.21M] reported | Allocator and memcpy deltas not bounded by two samples; read/write raw evidence incomplete | [Review F1/F2](reviews/scanner-146/report.md): interval HOLD, qualification REJECT; includes A2 |
| C3 CONDITIONAL | Original scanner handoff versus B2 | 87,622,080–87,632,430 at stated cap | Gross 88,622,080+3Q minus ≤1M **acceptance cap**, not observed overhead | Supersedes, not adds to, scanner checkpoint; safe prototype missed its target |
| C4 CONDITIONAL | Hot descriptor arrays versus B2 | 71,589,766 at ≤10 scalar instructions and ≤5M added work | Removes two LEAs/row; scattered pointer resets may add writes; calls, snapshots and spills must be charged | 40M registered minimum. No implemented checkpoint reviewed |
| C5 CONDITIONAL | SSE2 first-chunk refinement versus B3 | 6,589,500 gross; net unknown | Four fewer instructions/leaf target; setup, mask/broadcast, callers and memory count must be recounted | 4M incremental minimum; overlaps A1's replacement body, not additional B2 leaf credit |
| C6 CONDITIONAL | [Direct interior refill](proposals/refill-boundary/proposal.md) | **20,315,241 − H**, only at specified compiler caps; H unresolved | ≥1,084,913 eligible transitions, all on large blocks; 26,037,912 transient logical store bytes exposed before shifted work | Outside approved sequence; overlaps descriptor refill/helper work. No verified net credit. Future audit here non-independent |

No verified combined net is claimed. The Ultra 225,253,100 portfolio is a
conditional joint cap target, not achieved work removed and not an instruction
floor. A future combined listing must be counted afresh against B2, including
overlap and shifted work; its lower bound must clear charter §6.3's translated
floor and exceed the best single qualified component.

Critical-block relevance: the Ultra audit reports 230 large blocks (n≥899,000)
and 204 tiny blocks. All-full steps contribute approximately 98% of archived
candidate completion; slow workers are enriched in scalar rows and full finishes.
These are descriptive observations from an INVALID capture, not an economic
PASS or a causal cycles claim. They justify prioritizing scalar, tile and refill
work over tiny-block initialization and bulk-only refinements.

## Review state and next trigger

[Scanner review](reviews/scanner-146/report.md): safety PASS, diagnostic partial miss,
qualification REJECT; conservative interval HOLD pending bounded callee accounting
and a replayable raw memory/profile package. [V3 closeout review](reviews/v3/report.md):
archived safety PASS and historical instruction interval verified; three LOW reporting
corrections, compiler STOP unchanged. Neither is the final §6.2 review.

The [structural replay](structural-findings.md) verifies the per-block lower bounds
and distinguishes latest absolute end from longest worker duration. There is no
verified combined total. WAITING for a new pinned checkpoint/correction/rebuttal,
the completed-sequence evidence index, or a user nudge. No polling or automatic
Goal wake-up is assumed.
