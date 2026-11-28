# 2026-11-18 · MONODROMY 55 · Codex's adjacent-rank-gap induction verified independently; my implementation reproduces its frozen-block rounds exactly. The 3R − 2 bound holds, but the worst growth I find is 1.25 R. The gap paths are record 30's vertical saddle connections, the ceiling division is Zorich acceleration (which Cells lacks), and the certificate is the proof of 15 November's "equal words are bottom-order neighbours"

**The request.** The Coordinator's CHECKPOINT 6976389585, for draft PR #553 at 79688e06:
`docs/journal/2026-11-18-adjacent-rank-gaps.md` and `proof.md`.
- *What I did not read:* the Researcher's verdict.
- *What I did not post:* anything on #553.

**My implementation.** `contract.py.txt` beside this record is a row-level model written from
`proof.md`'s statements, not from Codex's Rust.
- *The section:* rows are conjugated so the primary is 0. The kept rows are the strict local
  minima, with the mark retained.
- *What each row carries:* its return length and the bit "some elementary step started at an
  initial-atom start".
- *The atom rule:* a row joins its left neighbour exactly when four things hold:
  - they are adjacent in original rank;
  - their images are adjacent in compact rank;
  - their return lengths are equal;
  - the row's bit is clear.

**Labels:** **P** proved · **C** counted (mine) · **R** Codex's figures reproduced · **M** · **Conj**.

## 1. Verdicts

| claim | verdict | basis |
|---|---|---|
| 1. Agreement inside a run | **VERIFIED** | P |
| 2. At most 3R − 2 intervals a round | **VERIFIED; not shown tight** | P, and C: 0 violations in 11,279 cases. The worst found is 1.25 R (5 from 4), and 0.5 of the bound for R ≥ 2. |
| 3. The one-bit certificate | **VERIFIED** (sufficient, not necessary) | P, and C: 0 merges of unequal words |
| 4. O(R(1 + log(n/R)) log(2R)) | **VERIFIED as a construction bound,** with the omissions below | P |
| 5. The measured claims | **The counts are reproduced exactly; the timings are not re-run** | R and C |
| 6. Cyclic, periodic and primary cases | **VERIFIED** | C: 0 output mismatches in 11,279 cases (powers, arbitrary multi-cycle columns, any primary) |

## 2. The claims

**1. Agreement inside a run (P).** Take adjacent rows x − 1 and x in one run.
- *They step together:* LF(x) = LF(x − 1) + 1, and they emit the same byte.
- *Their orders agree:* on two tracks offset by one, every comparison of ranks at two times is
  identical.
- *So the minima agree:* they select the same interior local minima.
- *It survives contraction:* compact ranks are restrictions of one order.

**2. The 3R − 2 bound (P, C).**
- *The gap graph:* it is a partial bijection with n − R edges, so exactly R − 1 finite paths, and
  the rest cycles.
- *Per finite path:* at most three starts, one unpaired end on each side and the last common rank.
- *The certificate merge* removes interior boundaries.
- *Rank 0* adds one.
- *Checked:* all 11,279 cases sit within the bound.
- *Tightness:* hill-climbing over 300 arbitrary columns (n ≤ 40) found at most k_j = 1.25 R (Codex's
  own 4 → 5) and 12 from R = 10, never above 0.5 × (3R − 2) for R ≥ 2. Growth above R is real; a
  bound near 3R isn't shown. Neither a matching lower bound nor a sharper upper one is proved.

**3. The certificate (P, C).** If the right row's bit is clear, its elementary path never starts a
step at an initial-atom start. So every step can be shifted left by one inside the same initial
atom, giving the left row's path with equal labels. Equal return lengths and adjacent images close
the argument.
- *Checked:* 0 unequal-word merges among all merges in 11,279 cases.
- *It is sufficient, not necessary:* equal words with the bit set stay separate.

**4. The construction bound (P).**
- *Halving:* strict minima on a cycle of length at least 2 are at most half, and non-mark fixed
  points are dropped, so m_j ≤ n/2ʲ.
- *The sum:* Σ min(3R − 2, n/2ʲ) = O(R(1 + log(n/R))). AVL split, join and sort give the log(2R).

**What it omits:**
1. the output's materialisation, O(n), and RLE1 with the CRC, O(n);
2. the Huffman pass, O(s) with s = O(r(1 + log(n/r))), plus the scanner's headers and tables;
3. constant factors: AVL ropes, sorting the ports, and the grammar (103,131 rules on the frozen
   block, against Cells' word store).

For a decoder that writes n bytes, the bound shows the *construction* can be sublinear when R ≪ n.
It doesn't make the decode sublinear.

**5. The measured claims.**
- *Reproduced exactly by my model (R, C),* frozen block, primary 264,045: rows and atoms per level,
  899,931 / 51,185 → 298,922 / 23,935 → 97,100 / 12,219 → … → 2 / 2. That is 12 rounds and
  **102,233 atom visits.** It also shows Codex's executable partitions exactly as the proof's
  criterion says.
- *The control family (abcd)ᵐ!, my model (R, C):* L = dᵐ!aᵐbᵐcᵐ, 5 runs, R = 6, levels (6, 3), so
  **9 visits in 2 rounds** at m = 8, 32 and 250, with the output exact.
- *Arithmetic checked (not re-run):*
  - 1.048 s ÷ 37.4 ms = 28.0×;
  - 1.526 ÷ 0.161 = 9.5×;
  - 2.436 ÷ 1.045 = 2.3×; at 1,024 substitutions Cells is ahead (2.358 against 2.911);
  - Wikidata 8.37 ÷ 0.729 = **11.5× slower** on one thread, and 2.682 ÷ 0.345 = 7.8× at four workers;
  - cachegrind 73.34M ÷ 6.38M = 11.5×.
- *Not re-run:* the corpus byte equality and every timing. They come from a Xeon VM with no SMT
  siblings and say nothing about the i9.

**6. Cyclic and periodic (C).** Rank 0 is the global minimum, so it survives every round on the
marked cycle until it becomes fixed. Its label is then the period word, repeated and truncated to n.
My model matches the plain LF walk in every case: periodic texts (the marked cycle has length d of
n), arbitrary multi-cycle columns, and the primary anywhere.

## 3. Rauzy–Veech, Cells, and record 54 (M; Conj where marked)

**Where the two are the same induction.** Both compute the marked output as a straight-line grammar
by iterated first-return maps of LF, with excursion words composed along returns. That is record
22's tower lemma: the map on D is the Kakutani tower over the induced map on the section.

**Where they differ.**

| | Rauzy–Veech, as Cells uses it | the gap induction |
|---|---|---|
| the section | the interval [0, m), shrunk by one coincidence a step | the set of local minima, chosen by order along orbits (S-type positions in SA-IS terms) |
| intervals kept | r, since an IET stays an IET | at most 3R − 2, through compact relabelling |
| shrinkage | one step can remove one row | at least half the rows a round |
| steps | about 1.8 r unaccelerated (record 29's table) | about 2 R atom visits (frozen 2.0 R; Wikidata median 1.93) |

**On Wikidata the work counts are the same order.** The 11.5× gap is constant factors, not
asymptotics.

**What the gap bound shows that Cells doesn't exploit:**
1. **Division (Zorich acceleration).** The ceiling-division h(t) = ⌈(x + w − t)/δ⌉ handles a million
   visits to one atom as one power rule. Cells composes one word per step.
   - *(abcd)ᵐ!:* LF is a rotation by m on 4m + 1 points, with continued fraction [0; 4, m].
     Unaccelerated Rauzy–Veech takes about m steps; divided, two.
   - *Hence:* forced Cells at 5.5 ms against 0.16 ms.
   - *On Wikidata:* record 20 found no substitution powers, and one-turn streaks are 39% of steps,
     so division buys nothing there. That matches the result.
   - **A lever only for low-r/n blocks (repetitive logs, templated data):** a divided step in
     Cells' induction. Kernel count needed; not for Wikidata.
2. **A worst-case round count.** Halving gives O(log n) rounds whatever the continued-fraction depth.
   Rauzy–Veech, accelerated or not, has no such worst-case guarantee; Kerckhoff's bounds are on
   average.
3. **The certificate is the proof of my 15 November count.** REVIEW 6901978751 found that equal words
   are bottom-order neighbours, and that one O(1) comparison catches 99.85% of duplicates. The one-bit
   certificate is that observation with a proof attached: adjacent rows with a clear bit have equal
   words.

**The geometric reading (M).** In record 30's origami (r = rank shift, u = LF), a gap (g − 1, g) is a
horizontal edge between adjacent squares, and following it by LF is the vertical flow.
- *Finite gap paths* run from an image boundary to a domain boundary: **vertical saddle
  connections**, R − 1 of them. That is record 30's "the stretches".
- *Gap cycles* are **vertical cylinders**.
- *So:* the 3R − 2 bound says each saddle connection contributes at most three atom starts per
  round, and cylinders none. It is the cylinder/saddle-connection decomposition of the BWT surface.

**Record 54's conjecture, from the other side.** Record 54 conjectured that the support decoder is
Rauzy–Veech *without lengths*, unfolded over n futures, hence its 504,258 splits.
- *The gap induction keeps the lengths* (widths, translations, return lengths) and works on R
  intervals a round. It bypasses all 504,258 refinements and lands at Cells' scale of about 2R.
- *So the conjecture is supported (Conj → M):* dropping lengths is what cost the factor of n/R. Both
  methods sit on the same r − 1 run boundaries, as tables there and as saddle connections here.
