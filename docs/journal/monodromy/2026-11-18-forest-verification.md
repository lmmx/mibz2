# 2026-11-18 · MONODROMY 59 · Astra's forest sections and boundary parentheses verified independently. All six frozen cuts reproduce exactly, every parenthesis-matched return agrees with dense LF, and B ≤ 2q − c is tight. The arcs summed over levels (≈ 1.76 r) are Cells' Rauzy–Veech step count; the 15× is constants and logs

**The request.** The Coordinator's CHECKPOINT 6981854721, for draft PR #569 at 3832bb12:
`docs/journal/2026-11-18-astra-investigation-3.md` and `2026-11-18-switch-elimination/proof.md`.
- *What I did not read:* the Researcher's verdict.
- *What I did not post:* anything on #569.

**My implementations.** `forest.py.txt` and `parens.py.txt` beside this record, written from
`proof.md`, not from Astra's code:
- the run network;
- descending-width Kruskal, with the primary's arc forced to be a chord;
- the potentials;
- the planar-topological leaf order;
- stack matching per height;
- first returns by dense LF, as ground truth.

**Labels:** **P** proved · **C** counted (mine) · **M** · **Conj**.

## 1. Verdicts

| claim | verdict |
|---|---|
| 1. The presentation is closed under first return | **VERIFIED** (P; C: six levels on the frozen block) |
| 2. The shared coordinate | **VERIFIED** (P): a tree has no cycle to make it inconsistent |
| 3. Boundary uniqueness (the T-join) | **VERIFIED** (P; C) |
| 4. One port order gives parentheses; the marked arrow is kept | **VERIFIED** (P; C: 33,692 matched returns, 0 wrong) |
| 5. B ≤ 2q − c; the excess never increases | **VERIFIED, and tight** (P; C: 0 violations in 52,179 levels; equality attained) |
| 6. Lengths from the second potential; deferred spelling | **Lengths VERIFIED** (C: 0 wrong in 33,692). **The spelling and marked output were not re-derived by me;** they stand on Astra's parity checks. |
| 7. The measured claims | **The frozen cuts reproduce exactly; the corpus and timings were not re-run** |

## 2. The claims

**1–3 (P).** A band from chord i to chord j composes translations, so it is again a translated
labelled slice. The network type is preserved.
- *The potential:* a_v = a_u + x_e − y_e is consistent on a tree, and z = k + a_u is constant along
  interior arcs.
- *The T-join:* at a fixed z, every occupied site has exactly one active in-arc and one active
  out-arc, with chord leaves counted. So the active edges form disjoint paths, and a forest has no
  cycles. Two T-joins with the same odd set T differ by an even subgraph, which in a forest is
  empty.
- *The diamond* (an undirected 4-cycle) shows the forest hypothesis is needed.

**4 (P, C).** The leaf order (in-subtrees, landings, departures, out-subtrees) has two properties.
- *It is planar:* each subtree is contiguous.
- *It is topological:* a directed path goes from an earlier leaf to a later one. A path can't go
  from v into an in-child, because that edge points into v.
- *So:* vertex-disjoint paths in a plane tree can't have interleaved leaf pairs. The matching is
  noncrossing, landings before departures, and the stack algorithm gives it uniquely.
- *The mark:* the primary's arc is excluded from the forest, so the primary is always retained.
- *Checked (C):* 3,000 random columns, half run-structured arbitrary columns, half BWTs of powers.
  Every port matched at every height gives the dense-LF first return: **33,692 checked, 0 wrong.**

**5 (P, C).** One tape a chord. At a site with out-degree b, the b − 1 outgoing boundaries split at
most b − 1 fragments. Σ_v (deg⁺(v) − 1) = E − V, so B ≤ q + E − V = 2q − c. With V′ = q and E′ = B,
E′ − V′ ≤ E − V.
- *Checked (C):* no violation in 52,179 levels.
- **Tight:** `dddaaaddcccccdcbdbbb` with p = 6 gives q = 4, c = 3 and B = 5 = 2q − c.
- *On the frozen block,* the excess falls 10,954 → 5,275 → 2,883 → 760 → 6 → 0, monotone.

**6 (C).** With |W_e| = 1 on the original arcs, the length formula 1 + h(t) − h(s) equals the
dense-LF return length in all 33,692 matched returns.
- *Not checked by me:* the spelling.
- *A note:* for blocks that are powers of a shorter word, the marked lap is shorter than n.
  Repeating its prefix is correct (record 55 §6). My levels don't reach one row there, since
  other LF cycles keep chords; the marked cycle reaches one.

**7.**
- *The frozen block, reproduced exactly (C):* E = 62,138, V = 51,184, c = 1.

  | cut | chord sites | rows after | bands |
  |--:|--:|--:|--:|
  | 1 | 10,955 | 110,273 | 16,230 |
  | 2 | 5,276 | 17,657 | 8,159 |
  | 3 | 2,884 | 3,916 | 3,644 |
  | 4 | 761 | 767 | 767 |
  | 5 | 7 | 7 | 7 |
  | 6 | 1 | 1 | 1 |

  The bands match even with my own tie-break, because the frozen block has none that matter.
- *Arithmetic checked:*
  - 11.819 ÷ 0.748 = 15.8× (paired 15.23×);
  - construction 35.30 of 39.28 ms = 90%;
  - Cachegrind reads 15.7×, writes 11.0×, LLC misses 3.9×;
  - heap 17.60 against 10.35 MB, +70%.
- *Not re-run:* the 233-block 4–6 cuts, the parity over 431 blocks, and the timings (a Xeon VM).

## 3. Theory (M; Conj where marked)

**Against record 57 (open return).**
- *Investigation 2* chose S by width, keeping the narrow runs. Its wide complement could hold
  undirected cycles, so the open map had to be *traced* (U) or *induced* (K).
- *This investigation* chooses S as the complement of a maximum-weight spanning forest: the least
  retained mass for which the complement is acyclic as an undirected graph. Exactly that makes the
  transport *boundary-determined* (the T-join), so no interior is followed.
- *The chords* are a cycle basis: q = E − V + c, the cycle rank. The diamond shows any undirected
  cycle left inside breaks it.

**Against Cells' Rauzy–Veech, and the cost.** Both are towers of first returns: Rauzy–Veech induces
on an interval, and this induces on the chord sources. The work counts line up:

| | forest sections | Cells (right-end census) |
|---|--:|--:|
| arcs over all levels | 62,138 + 16,230 + 8,159 + 3,644 + 767 + 7 = 90,945, ≈ **1.78 r** | **92,570** induction steps, ≈ 1.8 r |
| stored | 27,854 recipes | 28,706 rules |

**So the intrinsic combinatorial work is the same; the 15× is per-item cost.**
- *Implementation:*
  - the slab matcher does 48,647 stack matches for 16,230 bands, where the interval matcher would do
    O((q + B) log²);
  - BTree ranges and allocation;
  - 204,518 child references at output against Cells' direct write;
  - 3.9× the LLC misses.
- *Intrinsic, but optional:*
  - Kruskal's O(E log E) sort. Any spanning forest is correct and only changes the retained mass; a
    BFS forest is O(E), at a mass to measure.
  - The matching's log factors.
- *Linear:* the potentials and the port order.

**E − V, the adjacent-rank gaps and record 30's origami.**
- *What E − V counts (P):* within sites there are M − V internal gaps, and M − E are inside
  outgoing bands. So E − V is the number of gaps with no outgoing continuation. Equivalently, it is
  the number of finite gap paths that start and end *strictly inside* sites.
- *Against investigation 1:* that construction's R − 1 finite gap paths counted every boundary of
  the row line. Here run boundaries are site boundaries and drop out.
- *In record 30's terms:* these are the vertical saddle connections not anchored at run
  boundaries.
- *Monotone (Conj):* E′ − V′ ≤ E − V says first return never creates interior saddle connections.
  It can absorb those ending at coincidences, removable "fake" cone points of angle 2π.
- *A bound by the topology (Conj):* E − V is presentation-dependent, as the depth-3 refinement shows.
  Its least value over presentations should be bounded below by the origami's topology, as an
  interval exchange of d = 2g + s − 1 intervals has E − V = d − 1 in a standard presentation.
  Testing that would mean computing the BWT origami's genus and cone points (record 30's program)
  against the minimum excess on small blocks. Not done here.
