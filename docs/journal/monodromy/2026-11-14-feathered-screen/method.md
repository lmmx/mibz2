# Feathered (matching-driven) first-return induction: an untimed count, 14 November 2026

Screen of the relayed proposal (#145 comment 6877865220). Counts only: no timing, nothing in the
repository changed, nothing posted. Labels: **P** proved here (argument given) · **C** counted
(command given; exact for its input) · **Conj** conjecture.

Everything is in this directory (`DIR` = `(the session's scratch; the files kept are beside this note)`).

## Inputs

- Corpus `/home/user/mibz2/corpora/wikidata.bz2`, sha256
  `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`: **equal** to the census
  branch's `inputs-binaries.sha256` line for `target/census/wikidata.bz2` (C: `sha256sum`).
- Population: the 196 records with `"outcome":"admitted"` of `census-wikidata.jsonl`
  (`origin/research/cells-census` b5617a8e; copied here). All 196 are full blocks, n from 899,867
  to 900,000. A block is matched by its index among the scanner's `Event::Block`s, and its n is
  asserted equal to the census's.
- Decoder library: `git archive origin/master` (7410e2a5) in `mibz2-master/`, a path dependency.

## What was implemented

**The exchange** (journal 2026-11-10-rauzy-veech.md, sections 1 and 2): letters = the runs of L
(not cut at the walk's row); TOP order as they lie, BOTTOM order stably by byte; a letter has a
length, and a word in text order. Invariant I throughout.

**One round** (`State.cells`, `matching`, `contract`, `coalesce` in `feather_ref.py`; `round` in
`sim/src/main.rs`):
1. Cells E_ab = I_a ∩ J_b by a two-pointer merge of the tilings, in row order ("cells scanned"
   = the number of nonempty cells, a = b included).
2. A matching: candidates are the cells with a ≠ b; taken greedily in the policy's order, a cell
   is selected if neither of its labels is yet in a selected cell (as a or as b). Every policy's
   matching is therefore maximal.
   - **P1** row order.
   - **P2** largest cell first (ties: row order). Its sort of the candidates is *not* counted
     in the work.
   - **P3** least predicted fragmentation, then row order. The prediction is the exact change
     of the letter count if the cell alone were removed: (pieces of I_a left) + (pieces of I_b
     after) − 2 = −1 (the cell is all of I_a and all of J_b), 0 (all of one, at an end of the
     other) or +1 (the rest). Three passes over the cells, no sort.
   - **P3L** (an addition, not one of the three named): the same classes, largest first inside
     a class. It is the proposal's sentence "a large-weight matching, initially preferring
     contractions that do not increase the refined interval count" read literally.
3. The first-return exchange on K = X \ D. For a selected E_ab, at offset s_a in I_a and s_b in
   J_b, of length ℓ:
   - top: I_a loses [s_a, s_a + ℓ) and keeps what is left of it (0, 1 or 2 pieces, word w_a);
     I_b is cut into [0, s_b), the middle [s_b, s_b + ℓ) and the rest; the middle is a new
     letter with word **w_a w_b** (text order; the proposal writes w_b w_a, the order of
     travel);
   - bottom: J_b loses its middle; inside J_a the order is (a's left piece, b's middle, a's
     right piece).
   Fragments written = letters of this exchange ("r_new before coalescing").
4. Coalescing: two letters adjacent on top, adjacent in the same order on the bottom, and
   carrying the same word node (a node is an original run or one composition; pieces of one
   letter keep its node) are joined ("r_new after").
5. The walk's row is carried as (letter, offset, v), orig_ptr = π^v(that point); when its point
   is removed it moves to its preimage in b's middle and v grows by |w_b|.

**End.** The rounds stop when no cell with a ≠ b remains, i.e. every letter is sent to itself.
The text is the tracked letter's word E read as T[k] = E[(k − v) mod |E|].

**Counts kept (Rust, per block and policy):** n, r, S, rounds, both kinds of empty round, and
per round r, cells scanned, candidates, m, rows removed, r_new before and after coalescing
(`results/<P>-rounds.jsonl`); totals in `results/<P>-blocks.jsonl`. "Total work" = Σ over rounds
of (cells scanned + fragments written before coalescing). Words are not materialised in Rust:
a letter carries its length, its node, its word's length and a 61-bit rolling hash of its word.

**S**, the serial right-end induction's steps, is recomputed by `serial_steps` (the reference
`induce` of `examples/rauzy.rs`, counters removed).

## Proved

- **P1. T(D) ∩ D = ∅.** A selected E_ab lies in J_b; its image lies in J_a. Another selected
  E_cd lies in J_d, and d ≠ a because a is in one selected cell only; and J_a ∩ J_b = ∅ as a ≠ b.
  (Only "no selected a is a selected b" is used.)
- **P2. Return depth ≤ 2**, and exactly 2 in every round with m ≥ 1: x ∈ K with T x ∈ D has
  T² x ∉ D by P1; the preimage of a selected cell is nonempty and lies in K.
- **P3. r_new ≤ r + m before coalescing**, which implies the claimed r + 4m. A label is in at
  most one selected cell, so each cell's effect is confined to its own two letters: a keeps
  p_a ∈ {0, 1, 2} pieces, b becomes p_b ∈ {1, 2, 3}. E = I_a ∩ J_b is an intersection of two
  intervals, so each of its ends is an end of I_a or of J_b: if I_a keeps two pieces then
  E = J_b and p_b = 1; if b has three pieces then E = I_a and p_a = 0; otherwise
  p_a + p_b ≤ 3. Net change per cell ≤ +1, ≥ −1. Equality r + m occurs (C below).
- **P4. No empty round.** If T is not the identity on X some cell has a ≠ b, so a maximal
  matching has m ≥ 1 and removes at least one row. Hence rounds ≤ n − (cycles of π).
- **P5. Every cycle of π keeps a point**, so a BWT image ends at one letter with one point and
  |w| = n; an arbitrary column ends with (cycles of π) points, each letter λ cycles spelling
  its word, Σ λ|w| = n. (A cycle of T inside D would contradict P1; a fixed point lies in a
  cell E_aa, never selected.)
- **P6. Hubs.** In a round where a maximal matching has m cells, every candidate cell has a
  label among the 2m matched ones: the cell graph has a vertex cover of 2m labels.

## Verified by the oracle (C)

`python3 feather_ref.py test` → `ref_test_output.txt` ("ALL ASSERTIONS PASSED"). The oracle
(`Oracle.first_return`) iterates LF on rows from every surviving row until it is back in K,
and reads nothing of the construction. Checked after **every** round, before and after
coalescing: the induced map exact, every word exact, the greatest number of old steps in a
return, T(D) ∩ D on coordinates, the fragment count against an independent count of the
proposal's partition (old letter, image in D or not, side of the hole), Σ λ|w| = n; at the end
the text against the walk T[k] = L[π^−(k+1)(orig_ptr)].

| input | runs | rounds checked | text = walk | T(D)∩D ≠ ∅ | depth > 2 | r_new > r + m | r_new > r + 4m | r_new = r + m |
|---|---|---|---|---|---|---|---|---|
| primitive words, 2 letters, length ≤ 12 (8,032 words × P1, P2, P3, P3L, random maximal, random partial matching) | 48,192 | 265,224 | all | 0 | 0 | 0 | 0 | 3,686 |
| primitive words, 3 letters, length ≤ 8 (9,705 words × 6) | 58,230 | 249,982 | all | 0 | 0 | 0 | 0 | 2,201 |
| 20,000 random columns, length 1 to 40, 1 to 16 letters, any row (18,166 with several cycles) | 20,000 | 81,030 | all | 0 | 0 | 0 | 0 | 260 |

No counterexample to any of the three claims in 596,236 rounds; the observed depth is 2 in
every round. For arbitrary columns the induction is ended when no cell with a ≠ b remains; what
is left is one point a cycle of π (asserted) in 1 to 5 or more letters, and the tracked
letter's word gives the walk's n bytes.

## The Rust simulator against the reference (C)

- `feather_ref.py trace crosscheck_columns.txt` and `feather-sim trace crosscheck_columns.txt`
  print, for 23,036 columns (the 17,737 BWTs above, 5,000 random columns to length 300, 299
  BWTs of near-periodic words of 200 to 1,500 letters) × 4 policies, the rounds, the letters
  and points left, the compositions, and per round (r, cells, m, rows removed, r_new before,
  r_new after): **byte-identical** (`cmp crosscheck_python.txt crosscheck_rust.txt`; 92,144
  lines, 515,834 rounds). In that mode the Rust also asserts its final word's hash against a
  brute-force walk.
- On every real block and policy (196 × 4), asserted or recorded: the rounds finished; one
  letter, one point left; rows removed = n − 1; compositions = Σ m; Σ λ|w| = n after every
  round; the final word has length n and its hash equals the hash of the block's walked bytes
  (`bwt::build` + `bwt::walk`) rotated by the carried v (`hash_ok` true on all 784).
- **S:** equal to the census's `cells` on 189 blocks; on the other 7 the shipped kernel has 2
  to 8 cells more (it cuts the walk's row out: 1 or 2 letters more on 8 blocks). The equal steps are
  r − 1 on all 196. Pooled S = 20,456,267 = 1.844 r.
- Not verified on real blocks: the induced map round by round (the oracle is O(n) a row; the
  end hash, the Kac sum and the identity with the reference on small inputs stand for it), and
  the depth (2 by P2; the simulator composes at most two words per return by construction).

## Result (C; 196 admitted Wikidata blocks; min / q1 / median / q3 / max over blocks)

`summary.md` (all rows), `cumulative.md` (after k rounds). Commands:
`feather-sim blocks /home/user/mibz2/corpora/wikidata.bz2 census-wikidata.jsonl <P> results/<P>`,
then `python3 summarise.py P1 P2 P3 P3L` and `python3 cumulative.py P1 P2 P3 P3L`.

| | P1 row order | P2 largest first | P3 least fragmentation, row order | P3L least fragmentation, largest |
|---|---|---|---|---|
| rounds | 5779 / 10551 / 11001 / 11402 / 14045 | 47 / 67 / 108 / 172 / 318 | 3176 / 10030 / 10830 / 11343 / 14044 | 2039 / 4247 / 4550 / 4870 / 6955 |
| empty rounds (m = 0; no row removed) | 0; 0 | 0; 0 | 0; 0 | 0; 0 |
| total work / S | 474 / 851 / 882 / 917 / 1129 | 5.48 / 5.92 / 6.08 / 6.36 / 7.66 | 261 / 829 / 867 / 897 / 1095 | 123 / 295 / 330 / 369 / 628 |
| total work per output byte | 54 / 92 / 100 / 108 / 167 | 0.58 / 0.66 / 0.70 / 0.75 / 1.03 | 28 / 89 / 97 / 107 / 167 | 14 / 33 / 38 / 43 / 91 |
| of which cells scanned / S | 316 / 567 / 588 / 611 / 753 | 3.52 / 3.81 / 3.92 / 4.11 / 5.00 | 174 / 552 / 578 / 597 / 730 | 82 / 196 / 220 / 246 / 418 |
| of which fragments written / S | 158 / 284 / 294 / 306 / 377 | 1.96 / 2.10 / 2.16 / 2.25 / 2.67 | 87 / 276 / 289 / 299 / 365 | 41 / 98 / 110 / 123 / 209 |
| first round alone / S | 0.99 / 1.03 / 1.05 / 1.06 / 1.11 | 0.99 / 1.03 / 1.04 / 1.06 / 1.10 | 0.98 / 1.01 / 1.03 / 1.04 / 1.09 | 0.98 / 1.01 / 1.03 / 1.04 / 1.09 |
| max r reached / r | 1 on every block | 1 on every block | 1 on every block | 1 on every block |
| largest r_new / r | 0.686 / 0.707 / 0.711 / 0.714 / 0.726 | 0.678 / 0.699 / 0.703 / 0.706 / 0.719 | 0.657 / 0.676 / 0.679 / 0.682 / 0.692 | 0.657 / 0.675 / 0.678 / 0.681 / 0.691 |
| Σ m / S (compositions) | 1.219 / 1.294 / 1.306 / 1.317 / 1.353 | 0.770 / 0.792 / 0.805 / 0.814 / 0.848 | 1.118 / 1.197 / 1.216 / 1.228 / 1.267 | 0.884 / 0.946 / 0.964 / 0.980 / 1.030 |
| max over rounds of (r_new − r)/m, worst block (bound 4; proved 1) | 0.5 | 0.0 | 0.333 | 0.333 |
| rounds in which r grew (of all rounds, 196 blocks) | 871 of 2,174,999 | 0 of 24,265 | 514 of 2,064,884 | 513 of 885,288 |
| rows removed, round 1 / n | 0.216 / 0.234 / 0.240 / 0.248 / 0.263 | 0.360 / 0.368 / 0.371 / 0.374 / 0.383 | 0.255 / 0.272 / 0.279 / 0.284 / 0.308 | 0.304 / 0.318 / 0.323 / 0.326 / 0.348 |
| rows removed, rounds 1 to 4 / n | 0.525 / 0.551 / 0.560 / 0.567 / 0.645 | 0.780 / 0.798 / 0.802 / 0.807 / 0.827 | 0.565 / 0.595 / 0.603 / 0.610 / 0.672 | 0.664 / 0.687 / 0.693 / 0.701 / 0.759 |
| rows removed, rounds 1 to 16 / n | 0.680 / 0.715 / 0.720 / 0.727 / 0.804 | 0.981 / 0.989 / 0.990 / 0.991 / 0.993 | 0.717 / 0.747 / 0.755 / 0.764 / 0.839 | 0.822 / 0.848 / 0.858 / 0.869 / 0.931 |

No block was stopped: all 784 runs finished below the 50,000-round bound (r cannot exceed n:
a letter has a point).

**The rule stated before the data** ("a parallel prototype is considered only if the median
full block needs about 200 or fewer rounds and no more than 1.3× the serial work"):

| policy | median rounds (≤ about 200?) | median total work (≤ 1.3 S?) | blocks ≤ 200 rounds | blocks ≤ 1.3 S |
|---|---|---|---|---|
| P1 | 11,001: no | 882 S: no | 0 of 196 | 0 of 196 |
| P2 | 108: yes | 6.08 S: **no** | 169 of 196 | 0 of 196 |
| P3 | 10,830: no | 867 S: no | 0 of 196 | 0 of 196 |
| P3L | 4,550: no | 330 S: no | 0 of 196 | 0 of 196 |

Both conditions hold for no policy. The best, P2, meets the rounds and has 4.7 times the work
the rule allows (least on any block 5.48 S). The first round alone, under every policy, is
0.98 to 1.11 S: one scan of about 1.2 r cells and one rewrite of about 0.7 r letters. After
two rounds the work is 1.75 to 1.79 S (the four policies' medians): past the rule's 1.3 S.

## What the counts say (C unless marked)

- **The claims hold and are loose.** r_new − r is at most m (P), not 4m; on these blocks the
  letter count never exceeded its starting value, and under P2 it fell or stood in every one
  of 24,265 rounds. Fragmentation is not what costs.
- **What costs is rescanning.** Under P2 the rows go fast (0.37 n in round 1, 0.80 n in 4
  rounds, 0.99 n in 16) but the letters go slowly (0.70 r, 0.26 r, 0.085 r left after 1, 4,
  16 rounds), and every round scans all the cells of what is left: Σ cells = 3.9 S, Σ letters
  rewritten = 2.2 S, for Σ m = 0.80 S compositions. A selected cell writes at most 3 new letters;
  the other letters are scanned and copied unchanged.
- **Compositions:** P2 makes 0.805 S of them (median), fewer than the serial induction's steps;
  P1 1.31 S. This is the one figure in the proposal's favour, and it is a count of grammar
  nodes, not of work as the rule defines it.
- **Row order and class order starve on hubs.** Block 1 under P1, round 1,001: 4,261 letters,
  8,516 cells, all with a ≠ b, and a maximal matching of 6 cells; from about round 2,000 on,
  m = 2 a round with 3,500 letters left. By P6 twelve (then four) labels meet every cell: a
  few long letters overlap everything, and a matching can use each once a round. The serial
  induction spends one step per small letter there; the matching spends one round. P2 takes
  the hub's largest cell first and removes most of its rows at once. P3L, which ranks the
  class before the size, has the same starvation (median 4,550 rounds).
- **Conj:** a selection that does not rescan (a worklist of the cells changed by the last
  round) would bring the work towards Σ m plus the first scan (0.80 S + 0.67 S under P2's order,
  before any cost of keeping the worklist and of finding the largest cells); it is the "dynamically maintained" structure the proposal set out to avoid, and it is not
  built or counted here.

## Not done, and limits

- Only Wikidata's admitted blocks. No periodic adversaries or random-run blocks beyond the
  small inputs of the oracle and the cross-check.
- "Work" is cells scanned plus letters written; P2's and P3L's sorts, the hashing and the
  bottom-order pass are not counted (counting them would raise the figures).
- Three policies as named plus P3L; no weighted-optimal matching, no non-maximal policy (for
  instance class −1 and 0 cells only), no worklist.
- No timing of any kind.

## Files

- `feather_ref.py`, `ref_test_output.txt`: the reference, the oracle and its output.
- `sim/` (`Cargo.toml`, `src/main.rs`): the simulator; built with
  `CARGO_TARGET_DIR=DIR/sim/target cargo build --release` in `sim/`.
- `crosscheck_columns.txt`, `crosscheck_python.txt`, `crosscheck_rust.txt`.
- `results/<P>-blocks.jsonl` (one line a block), `results/<P>-rounds.jsonl` (per round:
  r, cells, candidates, m, rows removed, r_new before, r_new after), `results/<P>.log`.
- `summarise.py`, `summary.md`, `cumulative.py`, `cumulative.md`, `SHA256SUMS`.
- `census-wikidata.jsonl` (copy from the census branch), `mibz2-master/` (export of master).
