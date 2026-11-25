# The design

## Factors and levels

Six factors (`parts.md`): **S** (the Huffman scan), **E** (emission), **C** (construction), **M** (MTF/RLE2), **R** (RLE1, CRC and output), **D** (delivery). Each has two levels, real (−) and ideal (+).

## The matrix

- **Slice A** (the oracles that exist): S × E, a full 2² factorial, the four-arm test redone right.
  - **One binary:** every arm runs in it, and ratios are taken within it. 2a's four-arm oracle was VOID only because its binary was compared with a different production binary (+10.9%); within one binary that problem doesn't arise.
- **The full matrix:** 2^(6−1), resolution VI, 32 arms a count, defining relation I = SECMRD.
  - **What it resolves:** every main effect and every two-factor interaction, clear of each other. Three-factor interactions are aliased in pairs.
  - **If an effect is ambiguous,** a fold-over run follows: a new registration, run once.
- **Two anchor arms:** all real (the shipped decoder) and all ideal (the floor: what is left when every part is free). Both are in the fraction.

## Counts

- **1 worker:** pure demand, with no queueing: each part's share of the work.
- **6:** below the knee, where the workers control.
- **8:** the knee and balance point, and production from 10 threads asked (§85).
- **10:** above the knee.

## Controls

- the 100 ms pre-spin on every CPU before each decode, with the clock recorded per decode (§81);
- the scanner pinned to its fastest core, sibling idle, the rest masked off (stage 2b);
- **one unpinned link** at 8 workers (all real and all ideal), to carry the pinned results back to production's regime;
- decodes randomised within each block of rounds; wikidata ×4, as in every run since stage 1.

## Rounds

- **Sizing:** pinned and spun, the paired SD at 8 to 10 workers was about 4–5% (§86), about 1% at 6.
- **The effects:** slice A's are known to be large (the four-arm test: 0.976, 0.806 and 0.741 at 10).
- **Rounds per arm:** 12, which gives about ±2.5% at 8 to 10. That resolves the effects worth having, those over 5%, and calls nothing smaller.
- **Time:** slice A is 4 arms × 4 counts × 12 rounds, about 15 minutes; the full matrix is 32 × 4 × 12, about 1.5 hours.

## Measures, per decode

- **wall** (primary);
- **cycles and the clock** (cycles per reference cycle): the split as a registered secondary (§81);
- **CPU** time, and **peak RSS**, reported with the oracle's own share;
- **per-thread activity ledgers** (stage 1's instrument) in two traced rounds per arm, so each part's busy time shows its removal;
- **the identities** (conservation, Little's law, forced flow), as in stages 1 and 2.

## The analysis

The program is written before the data and lives in `results/`.
1. **Effects:** for each count, Jain's sign table on log wall gives each main effect and each two-factor interaction, with t-intervals over rounds.
2. **Allocation of variation:** the share of the variation in log wall each effect explains.
3. **The ceiling table:**
   - for each part at each count, the wall with that part alone ideal, over all real: its *room*;
   - the wall with every part ideal *except* that one, over all ideal: its share of the floor.
4. **The masking map:** each interaction that is real, read as "this part's room appears only when that one is ideal".
5. **The validity checks:** byte-identity in every arm; each oracle's removed part shows near-zero busy time in the ledgers; and the oracle's memory and handover cost are reported.

## What it feeds

- **The ceiling table** goes to MONODROMY, which fits the max and soft-max models (§78, record 132) against it, held out, and to Antigravity's second pass (`antigravity/`).
- **`../math-targets/targets.tsv`** gets its unmeasured ceilings filled (T2, T3, T6), and every target is re-ranked by measured room.
