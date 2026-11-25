# Slice A: the scanner (S) × emission (E)

**The run:** `ideal-parts/run-20261125T105502Z`, recipe PR 889 at 23e11fef. It is evidence 04c9ecd9 on the `evidence` branch, SHA-256 7dd3b836…87f0, checked.
- **Validity:** every phase passed. The pin held in 32 of 32 traces; the removed parts' busy time is 0.00% at every count; every arm was byte-identical.
- **Files:** the report (`report.txt`) and `ideal-parts.tsv` are here. The analysis program is `../analysis/ideal_parts.py`, on PR 889.

## The ceiling table (wall, part ideal over all real; 95% intervals; labels per `../../registration.md`)

| workers | S alone | E alone | S with E ideal | E with S ideal | both ideal (floor) | q_SE | interaction |
|---|---|---|---|---|---|---|---|
| 1 | 1.001 [0.987, 1.014] NO ROOM | **0.827 [0.817, 0.838] ROOM** | 0.992 | 0.820 | 0.820 | −0.002 | ADDITIVE |
| 6 | 0.983 [0.959, 1.007] UNRESOLVED | **0.849 [0.828, 0.871] ROOM** | 0.962 | 0.831 | 0.817 | −0.005 | UNRESOLVED |
| 8 | 0.964 [0.937, 0.991] UNRESOLVED | 0.960 [0.938, 0.982] UNRESOLVED | **0.862** | **0.859** | 0.827 | **−0.028 [−0.038, −0.018]** | INTERACTS |
| 10 | **0.919 [0.900, 0.938] ROOM** | 0.975 [0.965, 0.986] UNRESOLVED | 0.859 | 0.912 | 0.838 | **−0.017 [−0.023, −0.011]** | INTERACTS |

**The allocation of variation** in log wall (S / E / SE / residual):

| workers | S | E | SE | residual |
|---|---|---|---|---|
| 1 | 0% | 98% | 0.1% | 2% |
| 6 | 2% | 92% | 0.4% | 5% |
| 8 | 37% | 40% | 13% | 11% |
| 10 | 70% | 17% | 6% | 7% |

## What it shows

1. **Below the knee (1 and 6 workers), only emission has room.** Supplied codes save 15–17%, and the scanner saves nothing.
2. **At the knee (8), the scanner and emission co-limit.** Neither has resolvable room alone (each about 0.96), but each saves about 14% once the other is ideal. This is masking, measured: the four-arm test's "0.9 and 0.9 make 0.8" (§44), now at the balance point, with both parts' shares resolved.
   - The labels say UNRESOLVED rather than MASKED, because MASKED requires NO ROOM alone, and each part alone is 0.96, not within ±2%.
3. **Above the knee (10), the scanner has room** (0.919), and emission's room appears mostly behind it (0.912 with S ideal).
4. **The floor is high.** With both S and E ideal, the decoder still takes 0.82–0.84 of its wall at every count. Most of the remaining time is in the parts slice A leaves real: construction (C), MTF/RLE2 (M), RLE1/CRC/output (R) and delivery (D). Then come E's handover and the copy work its oracle leaves in the clock. This is what the full matrix measures next.
5. **E's handover is not small.** The supplied-codes oracle reports a handover of 286 ms (1 worker) to 418 ms (10), summed over workers.
   - **Net of handover:** at 1 worker, where the sum is wall time, E's room net of handover would be about (0.827 × 1.33 s − 0.286 s) / 1.33 s, or about 0.61. That is a reported figure only (clarification 6), using the median wall.
   - **Above 1 worker** the handover is spread across threads, so no simple net figure applies. This is Antigravity's point: the word-copying half of A7 stays in the clock.
6. **The clock ratio is 1.000** in every arm, so the pre-spin worked.
7. **Placement:**
   - **Pinned and unpinned agree:** pinned all-real over unpinned all-real at 8 is 0.998 [0.967, 1.031].
   - **Unpinned, the floor is lower:** all ideal is 0.811 unpinned, against 0.827 pinned.
   - **The four-arm test's unpinned scanner room was larger:** 0.806 at 10 (§44), against 0.919 here, pinned. A pinned scanner is faster, so it has less room to give.

## Antigravity's predictions, scored (`scoring-neon-1.tsv`)

**Values:** 8 of 9 hit. The measured interval contains the predicted point, or overlaps the predicted range.
- S at 1, 6, 8 and 10 all hit, including 10 (0.885 [0.845, 0.925] against 0.919 [0.900, 0.938]).
- E at 1, 6 and 8 hit. E at 10 missed: 0.990 predicted against 0.975 [0.965, 0.986].
- S+E at 8 hit: 0.81 predicted against 0.827.

**Labels:** 0 of 2 hit. Antigravity predicted E would be MASKED at 8 and 10, and it came out UNRESOLVED at both. The pattern is masking-like at 8, but E alone has more room than MASKED allows (0.960 and 0.975, against a predicted 0.98–0.99).

## Next

The full matrix needs C, M, R and D (Antigravity's PR 895) integrated by the Pathfinder: the 2^(6−1) fraction at 1, 6, 8 and 10 workers.
