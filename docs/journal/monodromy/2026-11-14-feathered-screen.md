# 2026-11-14 · MONODROMY 3 · Feathered, matching-driven first-return induction: an untimed screen on 196 full Wikidata blocks. It fails the rule set before the data

Assignment: QUEUED 6879452632 on #145 (relay 6877865220; LOG 6879443839, item 3). Counts only:
no timing, nothing in `src/`, no workstation run. Labels: **P** proved · **C** counted (exact
for its input) · **Conj** conjecture. Method, proofs and the full tables are in
`2026-11-14-feathered-screen/method.md` and `summary.md`; the programs are beside them.

## The rule, as stated before the data

A parallel prototype is considered only if the median full block needs about 200 or fewer
rounds **and** no more than 1.3× the serial work. Otherwise the screen ends as this entry.

## Result: not met by any policy

| matching policy | median rounds (≤ ~200) | median total work / S (≤ 1.3) | blocks ≤ 200 rounds | blocks ≤ 1.3 S |
|---|--:|--:|--:|--:|
| P1, greedy in row order | 11,001 | 882 | 0 / 196 | 0 / 196 |
| **P2, largest cell first** | **108** | **6.08** | 169 / 196 | **0 / 196** |
| P3, least fragmentation, then row order | 10,830 | 867 | 0 / 196 | 0 / 196 |
| P3L, least fragmentation, then largest | 4,550 | 330 | 0 / 196 | 0 / 196 |

- S is the serial right-end induction's step count for the block (pooled 1.844 r; it equals the
  census's `cells` on 189 blocks, the shipped kernel has 2 to 8 more on the other 7).
- Total work = Σ over rounds of (cells scanned for the matching + letters written for the new
  exchange). The least on any block under any policy is **5.48 S** (P2). The first round alone
  costs 0.98 to 1.11 S under every policy.
- **C:** all 784 runs (196 blocks × 4 policies) finished, each to one letter and one point, and
  the final word's hash equals the walked block's on all 784. The corpus's sha256 equals the
  census branch's pinned input (`research/cells-census` b5617a8).

## The proposal's claims

| claim | verdict | evidence |
|---|---|---|
| T(D) ∩ D = ∅ | **P**, and C: no counterexample | a selected E_ab lies in J_b, its image in J_a; a is in no other selected cell |
| return depth ≤ 2 | **P**; exactly 2 in every round with m ≥ 1 | from the first; C: 596,236 rounds against a brute-force first-return oracle |
| r_new ≤ r + 4m | **P in the stronger form r_new ≤ r + m**, tight | each end of I_a ∩ J_b is an end of I_a or of J_b, so a cell changes the letter count by −1 to +1 |
| fragment growth is "the central challenge" | **not on these blocks (C)** | the letter count never exceeded r on any block; under P2 it never grew in any of 24,265 rounds |

The oracle (`feather_ref.py.txt`; output `ref_test_output.txt`) checks every round's map, words
and depth, and the text against the walk at the end: all primitive words over 2 letters to
length 12 and 3 to 8, and 20,000 arbitrary columns, under six matchings. The Rust counter
agrees with it round for round on 23,036 columns. Orientation: the new word is w_a w_b in text
order (the proposal writes w_b w_a, the order of travel).

## Why it fails: rescanning, not fragmentation

- Under P2 the **rows** go fast: 0.37 n after one round, 0.80 n after 4, 0.99 n after 16. The
  **letters** go slowly: 0.70 r, 0.26 r, 0.085 r left. Each round scans every cell that remains
  and rewrites every letter, so the work is Σ over rounds of the letters left: 3.9 S of cells
  scanned and 2.2 S of letters written, for 0.80 S compositions.
- **Hub letters starve the matching** under row order and class order. Block 1 under P1 at round
  1,001 has 4,261 letters and 8,516 cells, all with a ≠ b, and a maximal matching of only 6
  cells: every cell touches one of the 12 matched labels. Later rounds have m = 2 with about
  3,500 letters left.
- **One figure in the proposal's favour:** P2 makes 0.805 S compositions (median), fewer
  grammar nodes than the serial induction has steps.
- **Conj, not built:** a structure that avoids rescanning would bring the work towards the first
  scan plus Σm (about 0.67 S + 0.80 S under P2's order) before its own upkeep. That is the
  dynamic worklist the proposal set out to avoid, and the first entry already priced one for
  free-first orders (§8 item 2: it did not pay on one thread).

## Limits

- Wikidata's 196 admitted blocks only; the periodic adversaries and random-run blocks were run
  only as small oracle inputs.
- No weighted-optimal matching, no non-maximal policy, no worklist. P2's sort is not counted in
  its work, which favours it.
- The induced map on real blocks is checked at the end (one letter, one point, the hash), not
  round by round; the depth there is 2 by the proof.
- Not run on the census's `phases-wikidata.jsonl` units: the serial baseline here is S steps,
  not the census's 6.55 Ir per output byte. A step and a scanned cell are both counted as one;
  they are not the same instruction cost, but at 5.5 S and up the verdict does not hinge on it.
