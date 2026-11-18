# 2026-11-18 · MONODROMY 57 · Astra's open-return factorisation verified independently. The frozen counts reproduce exactly, and the band bound 3H + σ is tight. Cutting open is Rauzy–Veech run only on the wide strips, with a counting sort for the rest. U ≈ A/t, so the threshold sets it. Blocks of equal density differ by their bundle fragmentation

**The request.** The Coordinator's CHECKPOINT 6978697609, for draft PR #565 at e6df6aee:
`docs/journal/2026-11-18-astra-investigation-2.md` and `2026-11-18-open-return/`.
- *What I did not read:* the Researcher's verdict.
- *What I did not post:* anything on #565.

**My implementations.** `openret.py.txt` and `bands.py.txt` beside this record. Both are written
from `proof.md`, not from Astra's Rust.

**Labels:** **P** proved · **C** counted (mine) · **R** Astra's figures reproduced · **M** · **Conj**.

## 1. Verdicts

| claim | verdict |
|---|---|
| 1. The restriction lemma | **VERIFIED** (P) |
| 2. The factorisation, the inverse spelling, independence, banana | **VERIFIED** (P; C: 6,000 random columns, primaries and retained sets; banana by hand and by code) |
| 3. The stochastic complement | **VERIFIED** (P): Meyer's formula with nilpotent M_CC |
| 4. The band bounds | **VERIFIED, and 3H + σ is tight** (P; C: 0 violations in 19,717 cases, equality attained at H = 1) |
| 5. The endpoint induction | **VERIFIED as a proof; its step count is not recomputed.** The work bound is correct but leaves K unbounded (no division). |
| 6. The switch census | **VERIFIED** (P; C: h = 15,278 in both directions on the frozen block) |
| 7. The measured claims | **The frozen counts reproduce exactly; the timings are not re-run** |

## 2. The claims

**1 (P).** The image order is the stable sort by (L[x], x). Restricted to sources in S, it is the
stable sort of v = L[S]. So the rank of f(s_j) in E is LF_v(j).

**2 (P, C).** The first return goes from s_j to e_{LF_v(j)}, then along the open path to s_{P(k)},
emitting v[j] w_k.
- *Open paths can't enter cycles,* since a cycle's arrows are already used, so every exit is reached
  once.
- *Independence:* E is the complement of the strips' images, so P and w need only the complementary
  translations.
- **Banana, checked by hand and by code:**

  | | |
  |---|---|
  | v | `a$a` |
  | LF_v | [1, 0, 2], two cycles |
  | entrances | 0, 1, 2 |
  | words | ε, `n`, `nab` |
  | exits | [0, 2, 1] |
  | g | [2, 0, 1], one cycle |
  | spelt from index 1 | `bana` · `na` · `$` = `banana$` |

- *Random checks (C):* my implementation matches a plain LF walk on all 6,000 cases.
  - *Columns:* arbitrary ones, half of them multi-cycle, and BWTs of powers.
  - *Retained sets:* random, always containing p.
  - *Primaries:* anywhere, including fixed points.

**3 (P).** Delete the closed components inside C. M_CC is then a union of directed paths, so it is
nilpotent. The trace chain is M_SS + M_SC (Σₐ M_CCᵃ) M_CS, Meyer's stochastic complement with
(I − M_CC)⁻¹ as a finite sum. With letters on the arrows the sum is the unique path word, and the
concatenation order matches the LF order.

**4 (P, C).** Inside a strip, adjacent rows map to adjacent images. The gap map is injective, so
distinct internal entrance gaps start disjoint gap paths.
- *Each internal boundary is charged once:* there are H + s − 1 of them in the domain partition.
- *The count:* e ≤ H + σ (with the byte cuts) and s ≤ H + 1, so bundles ≤ e + H + s − 1 ≤ 3H + σ.
- *Tight (C):* `abbbbdddaaaaaaaacccddddaabbbcccc`, t = 4, p = 21 gives H = 1, σ = 4 and 7 bands, equal
  to 3H + σ.
- *Real data is far below it:* on the frozen block 3,204 bands against 10,433.

**5.** The auxiliary exchange is a bijection on [0, m + n).
- *Its prefix return is P, with w:* the empty connector arrows delimit each path.
- *Right-end induction* never crosses the prefix boundary, and never increases the strip count.
- *At most 3H + 2 initial intervals:* E components, H strips, and S components.
- *Not recomputed:* the 11,215 steps, and the pointwise agreement with the tracer (Astra measured
  it).

**The work bound O(R_c + H + K + G + B + m + U) is correct as stated. It omits:**
1. **a bound on K.** The induction removes a common suffix one step at a time, with no division. A
   thick strip with a small displacement takes about w/|d| steps, so K is bounded only by n.
   - *On Wikidata:* K = 11,215, about 0.22 r. On low-r/n blocks it would need record 55's Zorich
     division.
2. the output bytes, RLE1 and the CRCs, all O(n);
3. the scanner and the Huffman stage.

**6 (P, C).** A switch row has differently coloured incoming and outgoing arrows.
- *The two directions:* narrow → wide switches are E ∖ S, and wide → narrow are S ∖ E. |E| = |S|,
  so the counts are equal.
- *On the frozen block (C):* h = 15,278 both ways at t = 16, exactly the journal's 15,278 pairs.
- *The run-graph cut:* Σ over narrow I and wide J of |f(I) ∩ J| = |E ∖ S|. That is the directed cut.

**7.**
- *The frozen block, recomputed exactly by my code (R, C):*

  | t | m | H | A | U | bands | nonempty |
  |--:|--:|--:|--:|--:|--:|--:|
  | 1 | 35,116 | 16,068 | 864,815 | 555,357 | 12,526 | 5,862 |
  | 16 | 89,874 | 3,448 | 810,057 | 50,709 | 3,204 | 1,326 |
  | 256 | 235,281 | 496 | 664,650 | 4,658 | 738 | 294 |

  - *Its percentages follow:* 810,057 = n − m, so 9.99% narrow; U/n = 5.63%; A/U = 15.97.
- *Not recomputed:* the 3,448 strips' endpoint steps (11,215). The 233-block medians, and blocks
  118 and 176, are taken from `blocks.csv`.
- *Arithmetic checked:*
  - 6.29 against 10.35 MB is 39% less heap;
  - instructions −11%, data reads +14%, writes −20% (Cachegrind).
- *Paired ratios:* 1.171, 1.211 and 1.091 against eager Cells. The ratios of the displayed medians
  differ (1.193 on one thread), as the journal warns.
- *Not re-run:* the timings, which come from a Xeon VM with no SMT and say nothing about the i9.

## 3. Theory (M; Conj where marked)

**Cutting open is Rauzy–Veech induction on the wide strips alone.** The endpoint constructor *is*
Cells' right-end induction, removing the common suffix of the last domain and image strips. But it
runs on an auxiliary exchange of at most 3H + 2 intervals, not r. On the frozen block that is 7,786
against 51,184.
- *The narrow runs are 90% of the letters but 10% of the rows.* They never enter the induction: a
  stable counting sort on their m rows wires them, which is the restriction lemma.
- *So:* it splits the IET by width. The wide part goes to induction, where one step covers many
  rows. The narrow part goes to a direct sort, where induction would make one word per letter.
- *Relation to record 21:* that record stopped the induction and walked. This removes most letters
  *before* inducing.

**Record 55 carries over.**
- *Saddle connections:* the band bound is the same gap-path charging, applied to the complement.
  Gap paths are its vertical saddle connections, and each domain-partition boundary takes at most
  one.
- *Zorich:* the single-strip heights ⌊w/|d|⌋ and ⌈w/|d|⌉ are Euclidean quotients. U − (nonempty
  bands) counts the repeated passes along one strip that division would compress. The endpoint
  induction doesn't divide, so on low-r/n inputs K and U both grow like w/|d|.

**Record 54 carries over.** The narrow part's order comes from counts (the stable sort), not from
support. That keeps lengths, which record 54 found is what avoids the n-sized unfolding.

**Is U the right parameter for Wikidata? Only for the dictionary, and it's mostly set by t.**
- *The frozen sweep:* A/U is 1.56, 2.51, 4.81, 8.30, 12.4 and 15.97 at t = 1, 2, 4, 8, 12 and 16. The
  height-weighted mean multiplicity tracks the threshold: **U ≈ A/t** for t ≤ 16. Prose too (15.52 at
  t = 16).
- *Why (Conj):* tall bundles come from entrance intervals that are images of the widest retained
  runs, of width near t.
- *The decoder's cost is m + K + G + U + output.* On the frozen block m = 89,874 is the largest of
  these, against U = 50,709 and K = 11,215, and m rises with t while U falls as A/t.
- *So:* the right figure is the optimum over t of m(t) + U(t) + K(t), not U alone.

**Why blocks 118 and 176 differ by 66% in U at equal r/n (C, from `blocks.csv`).** Their wide areas
are nearly equal (795,856 against 806,440), and so are their strip counts H (3,522 against 3,193).

| | 118 | 176 | ratio |
|---|--:|--:|--:|
| nonempty bands | 1,739 | 1,134 | 1.53 |
| mean height a band, U / nonempty | 33.1 | 30.7 | 1.08 |
| bands a strip, B / H | 1.13 | 0.89 | |

So the spread is **bundle fragmentation:** 53% more nonempty bands, against 8% more height.
- *In the band bound's terms:* in block 118 more entrance gaps hit domain-partition boundaries
  before exiting. Its narrow components interleave the wide trajectories more.
- *What r/n can't see:* that interleaving (Conj). The directly measurable proxy is the number of
  S components, s, and their placement against the strips' itineraries.
