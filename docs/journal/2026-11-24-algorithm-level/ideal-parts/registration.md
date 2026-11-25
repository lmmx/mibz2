# The rule, frozen before any data (RULES C1a: diagnostic; it decides nothing about production)

*Written 25 November 2026, before any run of this design. A change after the data is a new registration with its reason.*

**For each part P and count k,** its room is the wall with P alone ideal over all real, a geometric mean of the paired ratios with its 95% t-interval. Its label:
- **ROOM** iff the interval's upper bound is ≤ 0.95: making P free would save at least 5% at k;
- **NO ROOM** iff the interval lies within [0.98, 1.02];
- **MASKED** iff P has NO ROOM alone but its interaction with some part Q (the 2fi on log wall) has an interval wholly below 0, and P is ROOM with Q ideal. Its room appears only behind Q;
- **UNRESOLVED** otherwise.

**For each pair P, Q and count k,** the interaction (log scale) is labelled:
- **INTERACTS** iff its interval excludes 0;
- **ADDITIVE** iff the interval lies within ±0.01;
- **UNRESOLVED** otherwise.

**The floor** (all ideal over all real) at each count is reported, as the decoder's residual: memory, synchronisation and the reader.

**VOID** (the whole run) if any of these:
- any arm's output is not byte-identical to the shipped decoder's;
- a removed part shows more than 5% of its real busy time in its ideal arm's ledgers;
- the machine is not the i9-10900K;
- the pin is broken in more than 10% of traced decodes (2b's PIN-BROKEN label, here a VOID).

**INCOMPLETE** if any identity fails in a traced decode.

**Reported beside the labels,** never part of them: the oracle's memory and handover cost; the clock split; the unpinned link.

## Clarifications, 25 November 2026, before any data (the Pathfinder's questions on slice A, PR 889)

1. **The interaction's scale:** Jain's effect coefficient on log wall, q_PQ = (y₀ − y_P − y_Q + y_PQ)/4 for a 2² (and its sign-table equivalent in the fraction).
   - **ADDITIVE's ±0.01 applies to q_PQ.** On the difference of differences (4·q), that is ±0.04.
   - Both are reported.
   - **INTERACTS** iff q_PQ's interval excludes 0.
2. **MASKED,** as written:
   - NO ROOM alone;
   - q_PQ's interval wholly below 0;
   - ROOM (upper bound ≤ 0.95) with Q ideal.
3. **The removed-part VOID.** Per count, the mean busy time of the part's work in its ideal arms' traces, over the mean in its real arms' traces, must be ≤ 5%.
   - For S, the work is the scanner's A2 + A3.
   - For E, it is the workers' A7, the walk.
4. **Memory:** the oracle's store (about 149 MiB on wikidata ×4) is recorded and held in every arm, real ones included, so process history is equal. Peak RSS is compared within the binary only.

## Clarification, 25 November 2026, before any data: the oracles' handover cost (Neon's PR 895)

5. **Handover is charged apart from the part.** Each oracle's handover is the copy of its recorded output into the decoder. It is timed in its own ledger entry, not in the removed part's activities.
   - **The removed-part VOID** reads only the part's own work:
     - C, construction's A5 and A6a–c;
     - M, the MTF/RLE2 activity;
     - R, RLE1 and the block CRC;
     - D, the reader's copy to the caller.
6. **The room is measured as it is, handover included,** and labelled on that. Beside it the report gives:
   - the handover's time per decode;
   - the room net of handover, (wall − handover) / all real, as a reported figure only.

   For R the handover is large: about 55% of the real stage's time on the build VM, a one-copy handover of about 210 MB from a 200 MiB store. So R's measured room understates the ideal, and its net figure is the one to read alongside.
7. **Store memory:** each oracle's store is recorded and held in every arm of a run that includes it, as for S (clarification 4). The stores on wikidata ×4 are C 114 MiB, M 32 MiB and R 200 MiB, about 495 MiB with S's 149 MiB. Peak RSS is compared within the binary only.
