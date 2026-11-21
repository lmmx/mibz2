# 2026-11-18 · MONODROMY 61 · Synthesis of the four investigations. The rank-order constructions all pay about r + 3·(E − V) operations, a block property shared within 4%. They lose to Cells on cost per operation, not on operation count. Only the text-order (copy) descriptions meet a positional obstruction. The global sort is not the forest's cost

**The request.** The Coordinator's CHECKPOINT 6982460711 (QUEUED 6981926437). Louis and the
Coordinator paused new searches until this exists.

**Labels:** **P** proved · **C** counted (here, or cited records) · **M** modelled · **Conj** conjecture ·
**I** interpretation.
- *The frozen block:* n = 899,931, r = 51,184.
- *Timings:* from the cited records' Xeon VM, single thread, unless marked.

## 0. Answers

1. **Two obstructions, by representation and phase (I), not one.**
   - **Rank-order constructions** (Cells, investigations 1–3) never need text positions before the
     final write. **Their structural work is about the same count on every block** (§2). Every
     tested one loses to Cells by its **cost per operation**, a factor of 10–25, not by doing more
     operations. No positional obstruction appears there.
   - **Text-order descriptions** (Pro's copy parses, the BWT macro scheme) need text coordinates,
     which means list-ranking LF. The BWT's own scheme takes 12,406 forward sweeps to evaluate
     from its coordinates (record 60). The short parse needs the decoded text, or at least the
     run heads' text positions, as its seed. **Here the output-order obstruction is real (C, I).**
2. **The shared count (C).** Over all 233 large blocks, the operations per run of three unrelated
   constructions:
   - **Cells' steps,** median 1.837;
   - **investigation 1's atom visits,** 1.929;
   - **investigation 3's arcs over levels,** 1.778.

   They vary 3–4% across blocks, and **correlate 0.91 (Cells–forest), 0.74 and 0.68**, still 0.86 /
   0.68 / 0.60 with run density held fixed. Most of the excess beyond one operation a run tracks a
   block-intrinsic quantity, the **non-coincidence** (E₀ − V₀)/r (§2).
3. **The hypotheses.**
   - *Global against local, for the forest:* **refuted** as the main cost. The sort and union-find
     take about 2 ms of 35.3 ms construction (§4).
   - *A common obstruction in output order:* **supported for text-order descriptions, not for
     rank-order ones.**
4. **No impossibility is established.** The information bound is Θ(r) for the input (§5). The only
   work lower bounds are trivial: Ω(r) to read the input and Ω(n) to write the output. A shared
   count is not a lower bound.

## 1. What each construction avoids and what it pays, on the frozen block (C, from the records)

| construction | avoids | pays (structural count) | cost against Cells | redundancy established |
|---|---|---|---|---|
| **Cells** (right-end Rauzy–Veech) | per-row LF over the block | 92,570 steps (1.81 r); 28,706 rules | 1 (1.58–2.55 ms) | (the baseline) |
| **Investigation 1** (adjacent-rank gaps; records 55, 54) | per-row state; repeated passes through an atom (Zorich division) | 102,233 atom visits (2.00 r), 12 rounds; 103,131 rules; AVL ropes | 10.8× (27.56 ms); **9.5× faster on (abcd)ᵐ!** | **representational** (≤ 3R − 2 atoms a round); **computational only on low-r/n blocks**, through division. Lost from about 1,024 substitutions, and on Wikidata. |
| **Investigation 2** (open return, t = 16; record 57) | induction on narrow runs: 90% of letters, 10% of rows | m = 89,874-row sort; U = 50,709 traced letters; K = 11,215 steps on ≤ 3H + 2 = 7,786 intervals | +17% (1 thread), +9% (4 workers); −7–8% against lazy; heap −39%; instructions −11% | **representational**, and **computational in instructions and memory.** Not in wall time against eager Cells. |
| **Investigation 3** (forest; record 59) | following the interior: transport fixed by the boundary | 90,945 arcs over levels (1.78 r); 27,854 recipes; 48,647 slab matches (3× the bands); 204,518 output child references | 15–25× (39.28 ms) | **representational** (boundary interface, monotone excess); computational **not** established |
| **Pro**, copies (record 60) | spelling return words | **oracle:** 11,145 copies (0.22 r), but the text is the oracle. **The BWT macro scheme:** 62,138 phrases (= E₀), half pointing forward, 12,406 sweeps to evaluate | phase-sampled 8.94 against 2.37 ms (reported, not reproduced) | **representational** (descriptions in text order); computational **not** established |
| **Support** (record 54) | counts and weights | 22,261 bispecial tables, 67,024 B; 504,258 splits | about 190× (decoder alone) | **representational** (identifiability only) |

**Cost per structural operation, frozen block, same host per pair:**

| construction | per operation |
|---|--:|
| Cells | 17–28 ns a step |
| investigation 1 | about 270 ns a visit |
| investigation 3 | about 430 ns an arc |

## 2. The shared count

**The correlations (C).** All 233 large blocks, joined by ordinal, n and r across three
independent evidence files:
- *Cells:* `cells-gap/evidence/i9-census-1.jsonl`;
- *investigation 1:* #553 `adjacent-rank-gaps/evidence/wikidata.csv`;
- *investigation 3:* #569 `switch-elimination/evidence/forest-wikidata.csv`.

| operations ÷ r | median | IQR | range | CV |
|---|--:|--:|--:|--:|
| Cells steps | 1.837 | 1.791–1.879 | 1.29–2.08 | 0.043 |
| investigation 1 visits | 1.929 | 1.899–1.956 | 1.69–2.13 | 0.027 |
| investigation 3 arcs | 1.778 | 1.732–1.800 | 1.41–1.98 | 0.034 |

| pair | Pearson / Spearman | partial, given r/n |
|---|--:|--:|
| Cells ~ forest | **0.906** / 0.820 | **0.859** |
| investigation 1 ~ forest | 0.738 / 0.657 | 0.677 |
| Cells ~ investigation 1 | 0.681 / 0.570 | 0.602 |

The ratios are tight: forest ÷ Cells has median 0.968 (0.93–1.09), and investigation 1 ÷ Cells
1.051 (0.98–1.31).

**What drives it (P, C).** The run network has exactly **E₀ = 2r − 1 − (coincident boundaries)**
arcs. This holds on every one of the 233 blocks: each internal domain boundary that doesn't
coincide with an image boundary adds one arc. So E₀ − V₀ = r − 1 − (coincidences) counts the
interior saddle connections (record 59).
- *Each count's excess over one operation a run* correlates with the non-coincidence (E₀ − V₀)/r:
  0.71 (Cells), 0.63 (investigation 1) and 0.79 (the forest).
- *The fits (M):* excess ≈ 3.6 x (Cells) and 3.1 x (forest), where x = (E₀ − V₀)/r has median 0.223.

**Reading (I, Conj).** Every tested rank-order construction does **about r + 3 (E₀ − V₀)** structural
operations.
- *The r:* one operation a run, which is reading the input.
- *The rest:* resolving the boundaries where LF's images don't line up with the runs, about three
  operations each across the levels.
- *Not a lower bound:* the three share a combinatorial skeleton (iterated first returns on
  translation intervals), so they may share its count without the count being forced. §6 lists
  the proof obligation.

## 3. One obstruction or several?

**Two, separated by what the construction must know (I, C):**

**(a) Positional (text-order) obstruction.** Copy descriptions live in text coordinates.
- *The BWT's own scheme:* 51,184 literals at the run heads' *text positions* plus 10,954 copies, so
  it needs those positions first. Half its copies point forward, and its chains run up to
  26,406 bytes, so evaluating it costs 12,406 sweeps without ranks (record 60).
- *The short parse (11,145):* exists, but needs the text. With rank-order seeds it degrades to
  18,132.
- *What text coordinates amount to:* ranking LF's cycle, which the inverse itself does.
- **This obstruction is genuine for these representations.** It is the coordinate cost Pro's
  sampling met: 6.04 ms of sampling against the 2.37 ms control (reported).

**(b) Per-operation cost in rank-order constructions.** No obstruction of principle shows here.
- *The same count:* the three reach Cells' operation count within ±5% at the median.
- *Their losses* are constant factors: data structures (ropes, BTree slabs, heaps), 3× extra slab
  matches, recipe-following output, and up to 3.9× the LLC misses.
- *Cells' advantage:* it is a tight loop over flat arrays.

**Both hold, for different constructions and phases,** as Louis's distinction anticipated:
- Pro's sampling phase meets (a);
- the forest's and investigation 1's construction phases meet (b).

## 4. The hypotheses, tested

**Global against local (the Coordinator's), for the forest: REFUTED as the main cost (C).**
- *Timed directly* (Rust, this container, best of 200; `kruskal.rs.txt`) on the frozen block's
  62,138 level-0 arc widths:
  - **sort 0.22 ms, union-find 1.21 ms;**
  - later levels add about 46% more arcs, so about 2 ms in all, **about 5% of the 35.3 ms
    construction.**
- *And the global choice isn't needed:* any spanning forest is correct (record 59). The cost sits in
  matching, successive networks and recipes, which are (b).
- *For Pro's LZ-style parse,* the global ingredient is the text itself, which is (a), not a sort.

**A common obstruction in output order (ChatGPT, relayed): PARTLY SUPPORTED (I).**
- *Text-order descriptions (a):* yes.
- *Rank-order constructions (b):* no. They meet output order only in the final write, which every
  decoder pays, and their cost is constants.

## 5. Information against work

**Information (P, C).**
- *The input:* about r(log n + 8) bits.
- *Identifiability without counts* (record 54): 67,024 B, about 10.5 bits a run.
- *Copy descriptions:* Θ(r) phrases (≤ 2r for the BWT's own scheme; 0.22 r copies plus an r-byte
  seed for the oracle parse).

No representation found needs asymptotically less than the input, and **none shows that more
information must be obtained than the input already holds.**

**Work (P).** The only bounds are the trivial Ω(r) and Ω(n). Every rank-order construction is Θ(r)
structural operations plus O(n) writing (§2). Text-order recovery reduces to list ranking: Θ(n)
sequentially and Θ(sort) in the I/O model, which are known general bounds, not specific to the
BWT.

**No general impossibility is established by any of the four.**

## 6. Open possibilities, the cheapest decisive test for each, and proof obligations

**Open possibilities:**

| | possibility | cheapest decisive test |
|---|---|---|
| **O1** | **A rank-order construction at Cells' cost per operation.** The counts match Cells, so a flat-array implementation of the forest's interval matcher, or of investigation 1's rounds, might tie or win where it saves instructions or memory (investigation 2 already saves 11% of instructions). | Microbenchmark ns a structural operation on the frozen block's dumped levels: matcher over flat arrays against Cells' 17–28 ns a step. Decisive if under 2× Cells; about half a day. |
| **O2** | **Division (Zorich) inside Cells,** for low-r/n blocks only. | Count streak lengths per step in Cells' census on a repetitive corpus (logs, the (abcd)ᵐ family). Record 20 says Wikidata has no powers. Decisive for that corpus class; an hour with the existing census. |
| **O3** | **A presentation with smaller E₀ − V₀,** since the excess drives the count. The depth-3 context refinement cut the chord mass from 110,273 to 38,231 but raised the arcs to 76,243 (#569 screens). | Count Σ arcs over levels for the depth-k presentation on the frozen block (offline, with my `forest.py`). Decisive if it falls below 1.78 r; an hour. |
| **O4** | **A text-order route that avoids per-byte ranking.** Its budget is the 2.1–2.3 ms a block of record 60's arm A estimate, and no discovery method exists yet. | None until a discovery method is proposed. |
| **O5** | **Parallelism inside a block** for rank-order constructions: levels and atoms are batch structures, and pointer jumping fits them. Untested. | Count the dependency depth (rounds × per-round critical path) on the frozen block from investigation 1's rounds. Cheap; decisive only for the SMT/i9 question. |

**Proof obligations (no experiment settles these):**

| | obligation |
|---|---|
| **P1** | Is r + Ω(E₀ − V₀) a lower bound for every induction on translation intervals? (Conj) |
| **P2** | A lower bound for recovering text order from the run-length BWT beyond list ranking's. |
| **P3** | The least E − V over presentations, against the origami's genus and cone points (record 59). |
| **P4** | Whether investigation 1's 3R − 2 is tight. The worst found is 1.25 R (record 55). |
